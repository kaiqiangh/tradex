//! Owning fee statement and genuinely-required execution-FX statement.
//!
//! This module declares the *contract* for S29.9 only. It is a single, pure derivation that
//! consumes the delivered read-only seams (Spot capacity evidence, the portfolio-scoped FX
//! requirements, the proposal-scoped FX observation and the immutable intent) and produces one
//! approval-bound [`SpotFeeFxStatement`]. It never grants Arm, consent, dispatch or any other
//! execution authority, it never fabricates a venue number, and it never assumes currency parity.
//!
//! T01 (this slice's contract layer) ships the type surface and a deliberately-empty derivation so
//! the approval-bound plumbing can be reviewed on its own, with **provably zero behaviour change**:
//! [`derive`] returns `Ok(None)` for every input until T02 fills in the real statement. The field
//! is serialized only when present, so every delivered suite that never sees a statement is
//! byte-for-byte unchanged.
//!
//! All `sha256:` binding versions are declared at exactly **71** characters (`sha256:` + 64 hex).
//! Declaring the shorter 64 makes the generated wire schema reject the real value and the React
//! decoder throws `IPC_SCHEMA_INCOMPATIBLE`; that is the S29.8 lesson and it is enforced here.
use crate::protocol::{OrderSide, Result};
use crate::risk::RiskCheckOutcome;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The single typed reason a fee/FX statement carries. The wire value is screaming-snake.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotFeeFxReasonCode {
    /// Both statements are current and complete.
    SpotFeeFxQualified,
    /// The fee and FX evidence do not describe the same immutable intent.
    SpotFeeFxEvidenceMismatch,
    /// No current declared commission observation is bound to the intent.
    SpotFeeEvidenceUnavailable,
    /// The venue does not declare the fee-charging asset, so the statement fails closed.
    SpotFeeCurrencyUnknown,
    /// Declared rates are present but not usable as an exact decimal basis.
    SpotFeeRateUnsupported,
    /// No current proposal-scoped FX observation qualifies the required routes.
    SpotRequiredFxEvidenceUnavailable,
    /// A required route's rate is missing, stale or otherwise unqualified.
    SpotRequiredFxUnqualified,
    /// A required route has no supported provider pair.
    SpotRequiredFxUnsupportedRoute,
}

/// The declared commission basis actually used for the expected fee. Always the conservative one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotFeeRateBasis {
    Maker,
    Taker,
}

/// Why a required FX route exists for this intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotRequiredRoutePurpose {
    IntentPolicy,
    IntentFunding,
}

/// The declared state of one required FX route's evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotRequiredRouteState {
    Qualified,
    Unqualified,
    UnsupportedRoute,
    EvidenceUnavailable,
}

/// One genuinely-required execution FX route. A statement carries at most two of these.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotRequiredRoute {
    pub purpose: SpotRequiredRoutePurpose,
    #[schemars(length(min = 1, max = 16))]
    pub from_currency: String,
    #[schemars(length(min = 1, max = 16))]
    pub to_currency: String,
    /// Present only when a supported bounded producer pair exists for this route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 6))]
    pub provider_pair: Option<String>,
    pub state: SpotRequiredRouteState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub provider_quality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub provider_timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub first_receipt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub conservative_cost: Option<String>,
    #[schemars(length(min = 1, max = 256))]
    pub reason: String,
}

/// One exact statement of the reviewed intent's owning fee and genuinely-required execution FX.
/// It is explicitly execution-unqualified: it carries no Arm, consent, dispatch or execution
/// authority even when fully qualified, and it is not a substitute for authenticated preflight.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotFeeFxStatement {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    /// The bound `sha256:` intent hash — a full 71 characters on the wire.
    #[schemars(length(min = 71, max = 71))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub account_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub base_asset: String,
    #[schemars(length(min = 1, max = 16))]
    pub quote_asset: String,
    pub side: OrderSide,
    /// `UNKNOWN` when the venue does not genuinely declare the fee-charging asset.
    #[schemars(length(min = 1, max = 16))]
    pub fee_currency: String,
    /// The declared origin of the fee currency, e.g. `ACCOUNT_DECLARED_FEE_ASSET` or `UNKNOWN`.
    #[schemars(length(min = 1, max = 128))]
    pub fee_currency_origin: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_rate_basis: Option<SpotFeeRateBasis>,
    /// The venue's declared rates, reported verbatim, never promoted into a hidden gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub declared_maker_rate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub declared_taker_rate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub declared_buyer_rate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub declared_seller_rate: Option<String>,
    /// The conservative expected fee, expressed in `fee_currency`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub expected_fee: Option<String>,
    /// The intent notional in its own `QUOTE` unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub expected_spend_quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub maximum_authorized_spend_quote: Option<String>,
    #[schemars(length(min = 1, max = 16))]
    pub base_currency: String,
    /// Present only when the corresponding required conversion is genuinely qualified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub workspace_base_expected_spend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub workspace_base_expected_fee: Option<String>,
    #[schemars(length(max = 2))]
    pub routes: Vec<SpotRequiredRoute>,
    pub outcome: RiskCheckOutcome,
    pub reason_code: SpotFeeFxReasonCode,
    #[schemars(length(min = 1, max = 512))]
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub binding_blocker: Option<String>,
    /// Standing provenance limitations of the consumed slices, carried verbatim.
    #[schemars(length(max = 5), inner(length(min = 1, max = 128)))]
    pub carried_limitations: Vec<String>,
    /// Bound fee-evidence version: a full 71-character `sha256:` digest.
    #[schemars(length(min = 71, max = 71))]
    pub fee_evidence_version: String,
    /// Bound route-evidence version: a full 71-character `sha256:` digest.
    #[schemars(length(min = 71, max = 71))]
    pub route_evidence_version: String,
}

/// Derive the owning fee and genuinely-required execution-FX statement.
///
/// T02 fills this in. Until then the statement is deliberately absent so that T01 is provably
/// behaviour-neutral for every delivered suite: `Some` is never produced, so `RiskDecision` and
/// `ApprovalReview` serialize exactly as they did before this slice and no gate can fire.
///
// TODO(T02): expand this signature once the FX accessors exist. T02 needs the portfolio-scoped
// `financial_sources::fx_requirements` and the proposal-scoped `financial_sources` FX observation
// (see design §1.3 `spot_fee_fx::derive(capacity, fx_requirements, fx_observation, proposal, side,
// base)`). T01 deliberately keeps only the two parameters it can bind today rather than freezing a
// shape it does not yet understand.
pub(crate) fn derive(
    _capacity: &crate::spot_capacity::SpotCapacityInputs,
    _proposal: &crate::protocol::OrderProposal,
) -> Result<Option<SpotFeeFxStatement>> {
    Ok(None)
}
