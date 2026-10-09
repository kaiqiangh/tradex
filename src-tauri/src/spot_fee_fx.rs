//! Owning fee statement and genuinely-required execution-FX statement.
//!
//! This module derives one approval-bound [`SpotFeeFxStatement`] from the delivered read-only
//! seams: the Spot capacity evidence (which now also carries the declared commission projection),
//! the portfolio-scoped FX requirements, the proposal-scoped FX rate observation and the immutable
//! intent. It never grants Arm, consent, dispatch or any other execution authority, it never
//! fabricates a venue number, and it never assumes currency parity.
//!
//! Fail-closed discipline (S29.9 team-lead rulings):
//! - `fee_currency` is `UNKNOWN`: no fixed ordinary public host in the delivered set declares a Spot
//!   account's fee-charging **asset** (the account response declares commission *rates* only), so the
//!   fee statement fails closed with `SPOT_FEE_CURRENCY_UNKNOWN` and no expected-fee amount is
//!   fabricated. The delivered declared rates are still reported verbatim.
//! - A genuinely-required `USDT -> base` conversion can never match the only supported bounded
//!   provider pairs, so the required-FX statement fails closed with
//!   `SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE`. Producers are never relaxed and parity is never assumed.
//! - When both facts hold, the single binding blocker is the route blocker (a structural
//!   impossibility that precedes any fee arithmetic); the fee-currency fact is still reported.
//!
//! All `sha256:` binding versions are declared at exactly **71** characters (`sha256:` + 64 hex).
//! Declaring the shorter 64 makes the generated wire schema reject the real value and the React
//! decoder throws `IPC_SCHEMA_INCOMPATIBLE`; that is the S29.8 lesson and it is enforced here.
use crate::protocol::{
    FxRateEvidence, FxRequirementNeed, FxRequirementPurpose, FxRouteRequirement, FxRequirements,
    OrderProposal, OrderSide, Result,
};
use crate::risk::RiskCheckOutcome;
use crate::spot_capacity::SpotCapacityInputs;
use crate::{provider_io, spot_capacity};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

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
    /// Present only when a supported bounded producer pair exists for this route. Its length bound
    /// mirrors the delivered `FxRouteRequirement.providerPair` exactly (`min = 6, max = 6`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 6, max = 6))]
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
    /// The conservative expected fee, expressed in `fee_currency`. Never fabricated from an unknown
    /// fee currency, so it stays absent while `fee_currency` is `UNKNOWN`.
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

/// Standing limitations the consumed capacity slice always carries. They describe how the evidence
/// was obtained, not what it failed to cover, so they are reported and never treated as gaps.
const CARRIED_CAPACITY_LIMITATIONS: [&str; 2] = [
    "NON_ATOMIC_PROVIDER_SNAPSHOT",
    "DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED",
];

/// The declared rate basis that bounds the fee without ever understating it: the **larger** of the
/// venue's declared maker and taker rates, compared as exact decimals (never `f64`).
fn conservative_basis(
    commission: &spot_capacity::SpotDeclaredCommission,
) -> Result<SpotFeeRateBasis> {
    let ordering = provider_io::decimal_cmp(&commission.maker, &commission.taker)?;
    Ok(match ordering {
        Ordering::Greater => SpotFeeRateBasis::Maker,
        _ => SpotFeeRateBasis::Taker,
    })
}

fn digest(value: &impl Serialize) -> Result<String> {
    let bytes =
        serde_json::to_vec(value).map_err(|_| crate::protocol::TradeXError::new("RISK_EVIDENCE_UNAVAILABLE"))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

/// The standing limitations actually present in an obligation set, in a fixed order.
fn carried(present: &[String], known: &[&str]) -> Vec<String> {
    known
        .iter()
        .filter(|label| present.iter().any(|obligation| obligation == *label))
        .map(|label| (*label).to_string())
        .collect()
}

/// The delivered wire label of an FX evidence quality, without constructing a serialized document.
fn quality_label(quality: &crate::protocol::FinancialEvidenceQuality) -> Option<String> {
    serde_json::to_value(quality)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
}

/// One genuinely-required route qualified against the current proposal-scoped observation.
fn qualify_route(
    purpose: SpotRequiredRoutePurpose,
    requirement: &FxRouteRequirement,
    observation: Option<&FxRateEvidence>,
) -> SpotRequiredRoute {
    let pair = requirement.provider_pair.clone();
    let observed = pair.as_ref().and_then(|pair| {
        observation.and_then(|observation| {
            observation
                .rates
                .iter()
                .find(|rate| &rate.provider_pair == pair)
        })
    });
    let (state, reason) = match requirement.need {
        FxRequirementNeed::Identity => (
            SpotRequiredRouteState::Qualified,
            "The monetary input and target share one currency, so no external rate is required.",
        ),
        FxRequirementNeed::UnknownCurrency => (
            SpotRequiredRouteState::EvidenceUnavailable,
            "A required monetary currency is unavailable, so no conversion pair or zero amount can be inferred.",
        ),
        FxRequirementNeed::ExternalRate => match (&pair, observed) {
            (Some(_), Some(_)) => (
                SpotRequiredRouteState::Qualified,
                "A current bounded directional rate binds this required conversion.",
            ),
            (Some(_), None) => (
                SpotRequiredRouteState::EvidenceUnavailable,
                "This required conversion has a supported producer pair but no current qualified rate is bound to this unchanged intent.",
            ),
            (None, _) => (
                SpotRequiredRouteState::UnsupportedRoute,
                "This required conversion is unsupported by the selected bounded FX producer; currency parity is never assumed.",
            ),
        },
    };
    SpotRequiredRoute {
        purpose,
        from_currency: requirement
            .from_currency
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string()),
        to_currency: requirement
            .to_currency
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string()),
        provider_pair: pair,
        state,
        provider_quality: observation.and_then(|observation| quality_label(&observation.provider_quality)),
        provider_timestamp: observed.map(|rate| rate.provider_timestamp.clone()),
        first_receipt: observation.map(|observation| observation.observed_at.clone()),
        // The conservative cost of a required conversion is never fabricated: a rate read alone does
        // not establish the broker's exact conversion cost.
        conservative_cost: None,
        reason: reason.into(),
    }
}

/// The fee evidence situation, derived only from the already-validated declared commission. There
/// is deliberately no `Qualified` state: no fixed ordinary public host in the delivered set
/// declares the fee-charging asset, so the fee statement always fails closed on `fee_currency`.
enum FeeSituation {
    /// No current declared commission observation is bound to the intent.
    EvidenceUnavailable,
    /// The venue declares commission rates but not the fee-charging asset, so no amount is derivable.
    CurrencyUnknown,
    /// Declared rates are present but not usable as an exact decimal basis.
    RateUnsupported,
}

/// The binding route blocker, if any, in deterministic priority order (structural impossibility
/// precedes any evidence gap, which precedes an unqualified observation).
fn route_blocker(routes: &[SpotRequiredRoute]) -> Option<(SpotFeeFxReasonCode, &'static str)> {
    if routes
        .iter()
        .any(|route| route.state == SpotRequiredRouteState::UnsupportedRoute)
    {
        Some((
            SpotFeeFxReasonCode::SpotRequiredFxUnsupportedRoute,
            "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE",
        ))
    } else if routes
        .iter()
        .any(|route| route.state == SpotRequiredRouteState::EvidenceUnavailable)
    {
        Some((
            SpotFeeFxReasonCode::SpotRequiredFxEvidenceUnavailable,
            "SPOT_REQUIRED_FX_EVIDENCE_UNAVAILABLE",
        ))
    } else if routes
        .iter()
        .any(|route| route.state == SpotRequiredRouteState::Unqualified)
    {
        Some((
            SpotFeeFxReasonCode::SpotRequiredFxUnqualified,
            "SPOT_REQUIRED_FX_UNQUALIFIED",
        ))
    } else {
        None
    }
}

/// Derive the owning fee and genuinely-required execution-FX statement for one immutable intent.
///
/// Returns `Ok(None)` only when the immutable identity cannot be bound at all (the capacity
/// evidence and the intent do not describe the same intent); in that case there is nothing to
/// state. Whenever the identity is established but the evidence is missing, unsupported or
/// unqualified, a statement is returned with `outcome != Pass` so it remains visible to a reviewer.
pub(crate) fn derive(
    capacity: &SpotCapacityInputs,
    fx_requirements: &FxRequirements,
    fx_observation: Option<&FxRateEvidence>,
    proposal: &OrderProposal,
    side: OrderSide,
    base: &str,
) -> Result<Option<SpotFeeFxStatement>> {
    // Identity binding mirrors `spot_owning::same_intent`: capacity and the immutable intent must
    // describe the same account, instrument and assets. A contradiction means no identity to bind.
    let same_intent = capacity.workspace_id == proposal.workspace_id
        && capacity.proposal_id == proposal.proposal_id
        && capacity.proposal_hash == proposal.proposal_hash
        && Some(capacity.account_id.as_str()) == proposal.fields.account_id.as_deref()
        && capacity.instrument_id == proposal.fields.instrument_id;
    if !same_intent {
        return Ok(None);
    }

    // The FX requirements themselves must describe this unchanged intent.
    let requirements_match = fx_requirements.workspace_id == capacity.workspace_id
        && fx_requirements.proposal_id.as_deref() == Some(proposal.proposal_id.as_str())
        && fx_requirements.base_currency == base;

    // Narrow to the genuinely-required routes only: the intent-policy route is always taken, and
    // the intent-funding route is taken only for a buy. The portfolio-scoped account/balance/
    // position/open-order rows are a different projection (`review_context`'s domain) and are not
    // part of this statement.
    let mut routes = Vec::new();
    for requirement in &fx_requirements.requirements {
        let purpose = match requirement.purpose {
            FxRequirementPurpose::IntentPolicy => SpotRequiredRoutePurpose::IntentPolicy,
            FxRequirementPurpose::IntentFunding if side == OrderSide::Buy => {
                SpotRequiredRoutePurpose::IntentFunding
            }
            _ => continue,
        };
        routes.push(qualify_route(purpose, requirement, fx_observation));
    }

    // Fee evidence: consume only the already-validated declared commission projection.
    let observation = capacity.observation.as_ref();
    let commission = observation.and_then(|observation| observation.declared_commission.as_ref());
    // The conservative declared basis is computed as exact decimals (never `f64`). No expected-fee
    // amount is fabricated: the fee currency is never genuinely declared by a delivered host.
    let fee_basis = match commission {
        Some(commission) => conservative_basis(commission).ok(),
        None => None,
    };
    let fee = match commission {
        None => FeeSituation::EvidenceUnavailable,
        Some(_) if fee_basis.is_none() => FeeSituation::RateUnsupported,
        Some(_) => FeeSituation::CurrencyUnknown,
    };

    let fee_code = match fee {
        FeeSituation::EvidenceUnavailable => Some(SpotFeeFxReasonCode::SpotFeeEvidenceUnavailable),
        FeeSituation::CurrencyUnknown => Some(SpotFeeFxReasonCode::SpotFeeCurrencyUnknown),
        FeeSituation::RateUnsupported => Some(SpotFeeFxReasonCode::SpotFeeRateUnsupported),
    };

    // Binding priority: a structurally-unsupported required route precedes any fee-currency fact.
    // The other fact is never silently dropped: it stays in `fee_currency`/`fee_currency_origin`
    // and in the human-readable reason.
    let (reason_code, binding_blocker_code) = match route_blocker(&routes) {
        Some((code, code_label)) => (code, Some(code_label.to_string())),
        None => match fee_code {
            Some(code) => (
                code,
                Some(
                    match code {
                        SpotFeeFxReasonCode::SpotFeeEvidenceUnavailable => {
                            "SPOT_FEE_EVIDENCE_UNAVAILABLE"
                        }
                        SpotFeeFxReasonCode::SpotFeeCurrencyUnknown => "SPOT_FEE_CURRENCY_UNKNOWN",
                        SpotFeeFxReasonCode::SpotFeeRateUnsupported => "SPOT_FEE_RATE_UNSUPPORTED",
                        _ => "SPOT_FEE_FX_QUALIFICATION_BLOCKED",
                    }
                    .to_string(),
                ),
            ),
            None if !requirements_match => (
                SpotFeeFxReasonCode::SpotFeeFxEvidenceMismatch,
                Some("SPOT_FEE_FX_EVIDENCE_MISMATCH".to_string()),
            ),
            None => (SpotFeeFxReasonCode::SpotFeeFxQualified, None),
        },
    };
    let outcome = if binding_blocker_code.is_none() {
        RiskCheckOutcome::Pass
    } else {
        RiskCheckOutcome::Unavailable
    };

    // A single-line, human-readable reason that reports the binding fact and never hides the
    // co-occurring fee-currency fact. It must not contain a newline: the review UI joins blockers
    // with ' · ' on one line.
    let route_fact = routes.iter().find(|route| {
        matches!(
            route.state,
            SpotRequiredRouteState::UnsupportedRoute | SpotRequiredRouteState::EvidenceUnavailable
        )
    });
    let fee_fact = "The venue declares commission rates but not the fee-charging asset, so the fee currency remains UNKNOWN and no fee amount is fabricated.";
    let reason = match route_blocker(&routes) {
        Some(_) => match route_fact {
            Some(route) => format!(
                "The genuinely-required execution conversion {} -> {} is {}; {fee_fact}",
                route.from_currency,
                route.to_currency,
                match route.state {
                    SpotRequiredRouteState::UnsupportedRoute =>
                        "unsupported by the selected bounded FX producer, so no qualified rate can bound this intent",
                    _ =>
                        "unbound because no current proposal-scoped rate qualifies it",
                }
            ),
            None => fee_fact.to_string(),
        },
        None => match reason_code {
            SpotFeeFxReasonCode::SpotFeeFxQualified => {
                "Both the owning fee statement and every genuinely-required execution conversion are current and complete; this carries no execution authority by itself.".to_string()
            }
            SpotFeeFxReasonCode::SpotFeeEvidenceUnavailable => {
                "No current declared commission observation is bound to this unchanged intent, so the owning fee cannot be stated.".to_string()
            }
            SpotFeeFxReasonCode::SpotFeeFxEvidenceMismatch => {
                "The FX requirement evidence does not describe this unchanged intent, so no fee/FX statement can bind it.".to_string()
            }
            _ => fee_fact.to_string(),
        },
    };

    let fee_rate_basis = fee_basis;
    let fee_evidence_version = digest(&(
        capacity.binding_version.as_str(),
        commission,
    ))?;
    let route_evidence_version = digest(&(
        fx_requirements.material_version.as_str(),
        fx_observation.map(|observation| observation.material_version.as_str()),
        &routes,
    ))?;

    Ok(Some(SpotFeeFxStatement {
        workspace_id: capacity.workspace_id.clone(),
        proposal_id: capacity.proposal_id.clone(),
        proposal_hash: capacity.proposal_hash.clone(),
        account_id: capacity.account_id.clone(),
        instrument_id: capacity.instrument_id.clone(),
        base_asset: capacity.base_asset.clone(),
        quote_asset: capacity.quote_asset.clone(),
        side,
        fee_currency: "UNKNOWN".into(),
        fee_currency_origin: "UNKNOWN".into(),
        fee_rate_basis,
        declared_maker_rate: commission.map(|commission| commission.maker.clone()),
        declared_taker_rate: commission.map(|commission| commission.taker.clone()),
        declared_buyer_rate: commission.map(|commission| commission.buyer.clone()),
        declared_seller_rate: commission.map(|commission| commission.seller.clone()),
        expected_fee: None,
        expected_spend_quote: proposal.estimated_notional.clone(),
        maximum_authorized_spend_quote: proposal.fields.maximum_spend.clone(),
        base_currency: base.to_string(),
        workspace_base_expected_spend: None,
        workspace_base_expected_fee: None,
        routes,
        outcome,
        reason_code,
        reason,
        binding_blocker: binding_blocker_code,
        carried_limitations: observation
            .map(|observation| {
                carried(
                    &observation.unresolved_obligations,
                    &CARRIED_CAPACITY_LIMITATIONS,
                )
            })
            .unwrap_or_default(),
        fee_evidence_version,
        route_evidence_version,
    }))
}
