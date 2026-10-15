//! Owning fee statement and genuinely-required execution-FX statement.
//!
//! This module derives one approval-bound [`SpotFeeFxStatement`] from the delivered read-only
//! seams: proposal-bound exact-symbol commission inputs, Spot capacity, portfolio-scoped FX
//! requirements, the proposal-scoped FX rate observation, the bound source-use assessment and the
//! immutable intent. Account-level comparison rates stay in their historical capacity projection;
//! they do not supply owning fee evidence. This module never grants Arm, consent, dispatch or other
//! execution authority, fabricates a venue number, or assumes currency parity.
//!
//! Fail-closed discipline for incomplete/conflicting S29.11 evidence:
//! - `fee_currency` remains `UNKNOWN`: the exact-symbol commission response does not establish the
//!   future fee-charging **asset**, and no expected fee is fabricated in an undeclared asset. A
//!   complete backend-derived current branch plus account-bound BASE/QUOTE/BNB wallet coverage may
//!   qualify the current estimate under the approved conditional contract, while actual asset and
//!   amount remain unguaranteed. Incomplete branches remain unavailable.
//! - A genuinely-required `USDT -> base` conversion can never match the only supported bounded
//!   provider pairs, so the required-FX statement fails closed with
//!   `SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE`. Producers are never relaxed and parity is never assumed.
//! - When both facts hold, the single binding blocker is the route blocker (a structural
//!   impossibility that precedes any fee arithmetic); the fee-currency fact is still reported.
//!
//! The approval gate in `lib.rs` also carries a `None` arm that fails closed with
//! `SPOT_FEE_FX_QUALIFICATION_UNAVAILABLE`. That arm is **defensive only and unreachable on the
//! delivered seam**: `spot_capacity` is always `Some` for a valid Spot intent (its rules are
//! captured lazily), so `derive` is always reached and a visible statement is always produced. It
//! mirrors the equally-unreachable `SPOT_OWNING_QUALIFICATION_UNAVAILABLE` arm of the sibling S29.8
//! gate; keeping it is cheap insurance if a future seam ever withholds the capacity inputs.
//!
//! All `sha256:` binding versions are declared at exactly **71** characters (`sha256:` + 64 hex).
//! Declaring the shorter 64 makes the generated wire schema reject the real value and the React
//! decoder throws `IPC_SCHEMA_INCOMPATIBLE`; that is the S29.8 lesson and it is enforced here.
use crate::protocol::{
    FxRateEvidence, FxRequirementNeed, FxRequirementPurpose, FxRequirements, FxRouteRequirement,
    FxSourceUseAssessment, FxSourceUseAssessmentStatus, OrderProposal, OrderSide, Result,
};
use crate::risk::RiskCheckOutcome;
use crate::spot_capacity::SpotCapacityInputs;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The single typed reason a fee/FX statement carries. The wire value is screaming-snake.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotFeeFxReasonCode {
    /// Both statements are current and complete.
    SpotFeeFxQualified,
    /// The fee and FX evidence do not describe the same immutable intent.
    SpotFeeFxEvidenceMismatch,
    /// No current exact-symbol commission observation is bound to the intent.
    SpotFeeEvidenceUnavailable,
    /// No complete current conditional estimate qualifies while the fee asset is undeclared.
    SpotFeeCurrencyUnknown,
    /// Declared rates are present but not usable as an exact decimal basis.
    SpotFeeRateUnsupported,
    /// No current proposal-scoped FX observation qualifies the required routes.
    SpotRequiredFxEvidenceUnavailable,
    /// A required route's rate is missing, stale or otherwise unqualified.
    SpotRequiredFxUnqualified,
    /// A required route has no supported provider pair.
    SpotRequiredFxUnsupportedRoute,
    /// The provider result denomination differs from the workspace base currency.
    ValuationCurrencyMismatch,
}

/// A maker/taker role label. Legacy `feeRateBasis` values on captured statements describe the
/// historical account-comparison calculation; new owning statements leave that field absent.
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
    CurrencyMismatch,
}

/// One genuinely-required, purpose-scoped FX leg. A statement carries at most three: two explicit
/// policy-valuation legs plus a separate funding leg. `QUALIFIED` applies only to `purpose` and
/// never means provider permission or execution authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotRequiredRoute {
    pub purpose: SpotRequiredRoutePurpose,
    #[schemars(length(min = 1, max = 16))]
    pub from_currency: String,
    #[schemars(length(min = 1, max = 16))]
    pub to_currency: String,
    /// Present only when a supported bounded producer pair exists for this route. Its length bound
    /// mirrors `FxRouteRequirement.providerPair`, including the seven-character `USDTUSD` pair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 6, max = 7))]
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

/// One exact statement of the reviewed intent's owning fee and genuinely-required FX.
/// Price-only policy valuation and funding/execution routes have separate qualification rules.
/// This statement carries no Arm, consent, dispatch or execution authority even when complete,
/// and it is not a substitute for authenticated preflight.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotFeeFxStatement {
    /// Exact BASE/QUOTE/BNB wallet rows bound to the current account-read receipt.
    /// Older captures omit this additive evidence; retirement never reuses prior rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_funding_evidence: Option<crate::spot_fee_model::SpotWalletFundingEvidence>,
    /// S29.11 current scenario; it does not qualify absent funding or required FX.
    /// Absent on older persisted captures, whose source-only meaning is preserved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_fee_model: Option<crate::spot_fee_model::SpotCurrentFeeModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub current_fee_model_failure: Option<String>,
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
    /// Historical account-level comparison basis, retained only when reading older captures.
    /// New statements leave it absent; current fee modeling is bound to exact-symbol terms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_rate_basis: Option<SpotFeeRateBasis>,
    /// Historical account-level comparison rates from older captures. New statements leave these
    /// absent; exact-symbol terms are preserved by the bound SpotCommission evidence input.
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
    #[schemars(length(max = 3))]
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

/// Current source-derived fee and source-use facts for one owning statement.
pub(crate) struct SpotFeeFxSources<'a> {
    pub commission: Option<&'a crate::spot_commission::SpotCommissionInputs>,
    pub source_use_assessment: Option<&'a FxSourceUseAssessment>,
    pub current_fee_model: Option<&'a crate::spot_fee_model::SpotCurrentFeeModel>,
    pub wallet_funding_evidence: Option<&'a crate::spot_fee_model::SpotWalletFundingEvidence>,
}

fn fee_situation(
    capacity: &SpotCapacityInputs,
    commission: Option<&crate::spot_commission::SpotCommissionInputs>,
    current_fee_model: Option<&crate::spot_fee_model::SpotCurrentFeeModel>,
    wallet_funding_evidence: Option<&crate::spot_fee_model::SpotWalletFundingEvidence>,
    proposal: &OrderProposal,
    side: OrderSide,
) -> FeeSituation {
    use crate::spot_commission::SpotCommissionStatus;

    let Some(commission) = commission else {
        return FeeSituation::EvidenceUnavailable;
    };
    if !commission.matches_capacity_binding(proposal, capacity, side) {
        return FeeSituation::EvidenceMismatch;
    }
    if commission.status != SpotCommissionStatus::Observed {
        return FeeSituation::EvidenceUnavailable;
    }
    let Some(observation) = commission.observation.as_ref() else {
        return FeeSituation::EvidenceUnavailable;
    };
    if !observation.matches_capacity_symbol(capacity, side) {
        return FeeSituation::EvidenceMismatch;
    }
    if !observation.terms_complete
        || !observation.extension_keys.is_empty()
        || observation.unresolved_obligations.iter().any(|obligation| {
            matches!(
                obligation.as_str(),
                "UNKNOWN_ACTIVE_COMMISSION_FIELDS_UNRESOLVED" | "UNSUPPORTED_DISCOUNT_ASSET"
            )
        })
    {
        return FeeSituation::RateUnsupported;
    }
    if let Some(model) = current_fee_model {
        if !supports_current_fee_branch(model, observation) {
            return FeeSituation::CurrencyUnknown;
        }
        if !wallet_funding_evidence.is_some_and(|wallet| {
            wallet.status == crate::spot_fee_model::SpotWalletFundingStatus::Complete
                && wallet.workspace_id == proposal.workspace_id
                && wallet.proposal_id == proposal.proposal_id
                && wallet.proposal_hash == proposal.proposal_hash
                && wallet.account_id == capacity.account_id
                && wallet.required_assets.contains(&capacity.base_asset)
                && wallet.required_assets.contains(&capacity.quote_asset)
                && wallet.required_assets.contains(&"BNB".to_owned())
                && wallet.missing_assets.is_empty()
                && wallet.required_assets.iter().all(|asset| {
                    wallet
                        .balances
                        .iter()
                        .filter(|row| row.asset == *asset)
                        .count()
                        == 1
                        && wallet
                            .balances
                            .iter()
                            .find(|row| row.asset == *asset)
                            .is_some_and(|row| row.effective_free.is_some())
                })
        }) {
            return FeeSituation::FundingUnavailable;
        }
        // The current estimate can qualify under the approved conditional contract. The wire fee
        // currency intentionally stays UNKNOWN until settlement reports the actual asset.
        FeeSituation::ConditionalEstimate
    } else {
        FeeSituation::CurrencyUnknown
    }
}

/// Validate the selected current scenario while the actual future commission asset remains
/// unknown. Only a complete, account-bound model and wallet may qualify the present estimate;
/// incomplete conditional inputs and inconsistent model evidence remain unavailable.
fn supports_current_fee_branch(
    model: &crate::spot_fee_model::SpotCurrentFeeModel,
    observation: &crate::spot_commission::SpotCommissionObservation,
) -> bool {
    use crate::spot_fee_model::{ChargingCondition, FeeFundingDisposition};

    if model.received_asset != observation.received_asset {
        return false;
    }
    let Some(preview) = model.funding_plan_preview.as_ref() else {
        return false;
    };
    let fallback_is_consistent = preview.fallback_fee_asset == observation.received_asset
        && preview.fallback_fee_amount == model.fallback.total
        && preview.fallback_net_receipt == model.fallback.net_received;
    match model.charging_condition {
        ChargingCondition::DiscountDisabledCurrentFallback => {
            (!observation.discount.enabled_for_account || !observation.discount.enabled_for_symbol)
                && preview.fee_disposition == FeeFundingDisposition::ReceivedAssetDeduction
                && fallback_is_consistent
        }
        ChargingCondition::ZeroBnbCurrentFallback
        | ChargingCondition::InsufficientBnbCurrentFallback => {
            observation.discount.enabled_for_account
                && observation.discount.enabled_for_symbol
                && observation.discount.discount_asset == "BNB"
                && preview.fee_disposition == FeeFundingDisposition::ReceivedAssetDeduction
                && fallback_is_consistent
                && model.bnb_funding_evidence.is_some()
                && (model.charging_condition != ChargingCondition::InsufficientBnbCurrentFallback
                    || model.bnb_fee.is_some())
        }
        ChargingCondition::DiscountedBnbCurrentEstimate => {
            observation.discount.enabled_for_account
                && observation.discount.enabled_for_symbol
                && observation.discount.discount_asset == "BNB"
                && observation.conditional_discount_asset.as_deref() == Some("BNB")
                && preview.fee_disposition == FeeFundingDisposition::BnbCurrentEstimate
                && preview
                    .current_fee_claim
                    .as_ref()
                    .is_some_and(|claim| claim.asset == "BNB")
                && model.bnb_funding_evidence.is_some()
                && model.bnb_fee.is_some()
        }
        ChargingCondition::ZeroDiscountedFeeExpression => {
            model.discounted_fee_is_exactly_zero == Some(true)
                && preview.fee_disposition == FeeFundingDisposition::NoCurrentFee
        }
        ChargingCondition::ConditionalBnbInputsRequired => false,
    }
}

/// Receipt and collection identifiers remain in captured provenance, but do not change current
/// financial material when the exact terms and source binding are unchanged.
fn fee_evidence_material(
    commission: Option<&crate::spot_commission::SpotCommissionInputs>,
) -> Result<serde_json::Value> {
    let Some(commission) = commission else {
        return Ok(serde_json::Value::Null);
    };
    let mut material = serde_json::to_value(commission)
        .map_err(|_| crate::protocol::TradeXError::new("RISK_EVIDENCE_UNAVAILABLE"))?;
    if let Some(object) = material.as_object_mut() {
        object.remove("stateVersion");
        if let Some(observation) = object
            .get_mut("observation")
            .and_then(serde_json::Value::as_object_mut)
        {
            observation.remove("collectionId");
            observation.remove("reads");
        }
    }
    Ok(material)
}

fn digest(value: &impl Serialize) -> Result<String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| crate::protocol::TradeXError::new("RISK_EVIDENCE_UNAVAILABLE"))?;
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

/// One genuinely-required route assessed for its declared purpose against the current
/// proposal-scoped observation and, for policy valuation, the exact source-use assessment.
fn qualify_route(
    purpose: SpotRequiredRoutePurpose,
    requirement: &FxRouteRequirement,
    observation: Option<&FxRateEvidence>,
    policy_valuation_supported: bool,
    workspace_base: &str,
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
    let (state, reason, conservative_cost) = match requirement.need {
        FxRequirementNeed::Identity => (
            SpotRequiredRouteState::Qualified,
            "The monetary input and target share one currency, so no external rate is required.",
            None,
        ),
        FxRequirementNeed::UnknownCurrency => (
            SpotRequiredRouteState::EvidenceUnavailable,
            "A required monetary currency is unavailable, so no conversion pair or zero amount can be inferred.",
            None,
        ),
        FxRequirementNeed::ExternalRate => match (pair.as_deref(), observed) {
            (Some(_), Some(_))
                if purpose == SpotRequiredRoutePurpose::IntentPolicy
                    && requirement.to_currency.as_deref() != Some(workspace_base) =>
            {
                (
                    SpotRequiredRouteState::CurrencyMismatch,
                    "The raw provider quote is visible, but its result currency differs from the workspace base; no converted amount or extra FX leg is used.",
                    None,
                )
            }
            (Some(_), Some(_))
                if purpose == SpotRequiredRoutePurpose::IntentPolicy
                    && !policy_valuation_supported =>
            {
                (
                    SpotRequiredRouteState::Unqualified,
                    "The current source-use assessment does not support POLICY_VALUATION for this FX source; an observed quote cannot be relied on for the owning valuation.",
                    None,
                )
            }
            (Some("USDTUSD"), Some(rate)) if purpose == SpotRequiredRoutePurpose::IntentPolicy => {
                let bid = crate::financial_sources::normalize_fx_numeric_text(&rate.bid).ok();
                let ask = crate::financial_sources::normalize_fx_numeric_text(&rate.ask).ok();
                let in_reviewed_band = bid.as_deref().is_some_and(|bid| {
                    crate::provider_io::decimal_cmp(bid, "0.99")
                        .is_ok_and(|ordering| ordering != std::cmp::Ordering::Less)
                }) && ask.as_deref().is_some_and(|ask| {
                    crate::provider_io::decimal_cmp(ask, "1.01")
                        .is_ok_and(|ordering| ordering != std::cmp::Ordering::Greater)
                });
                if in_reviewed_band {
                    (
                        SpotRequiredRouteState::Qualified,
                        "The current USDT/USD quote is within the reviewed one-percent policy-valuation band. Execution depth and conversion-transaction cost do not apply to this price-only valuation; source-use rights and execution qualification remain separate.",
                        None,
                    )
                } else {
                    (
                        SpotRequiredRouteState::Unqualified,
                        "The current USDT/USD quote is outside the reviewed one-percent policy-valuation band.",
                        None,
                    )
                }
            }
            (Some(_), Some(_)) if purpose == SpotRequiredRoutePurpose::IntentPolicy => (
                SpotRequiredRouteState::Qualified,
                "The current directional quote qualifies for price-only policy valuation. Execution depth and conversion-transaction cost are not applicable; source-use rights and execution qualification remain separate.",
                None,
            ),
            (Some(_), Some(_)) => (
                SpotRequiredRouteState::Unqualified,
                "A directional read-only rate is present, but execution quality and conservative execution cost are not established.",
                None,
            ),
            (Some(_), None) => (
                SpotRequiredRouteState::EvidenceUnavailable,
                "This required conversion has a supported producer pair but no current qualified rate is bound to this unchanged intent.",
                None,
            ),
            (None, _) => (
                SpotRequiredRouteState::UnsupportedRoute,
                "This required conversion is unsupported by the selected bounded FX producer; currency parity is never assumed.",
                None,
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
        provider_quality: observation
            .and_then(|observation| quality_label(&observation.provider_quality)),
        provider_timestamp: observed.map(|rate| rate.provider_timestamp.clone()),
        first_receipt: observation.map(|observation| observation.observed_at.clone()),
        // A policy valuation is price-only, so transaction cost is explicitly not applicable.
        // Funding/execution routes never infer cost from a read-only rate.
        conservative_cost,
        reason: reason.into(),
    }
}

/// Convert the immutable Proposal notional through its exact current, purpose-qualified policy
/// route. This value is policy valuation only; it neither upgrades the source quality nor proves
/// fee currency, funding, execution cost or provider permission.
fn workspace_base_expected_spend(
    proposal: &OrderProposal,
    base: &str,
    requirements: &FxRequirements,
    requirements_match: bool,
    routes: &[SpotRequiredRoute],
    observation: Option<&FxRateEvidence>,
    side: OrderSide,
) -> Option<String> {
    if !requirements_match {
        return None;
    }
    let amount = proposal.estimated_notional.as_deref()?;
    let currency = proposal.estimated_notional_currency.as_deref()?;
    // A local valuation consumes only one provider-returned pair. Its result denomination must
    // already be the workspace base; never append a bridge leg to manufacture that denomination.
    let mut matches = requirements.requirements.iter().filter(|requirement| {
        requirement.purpose == FxRequirementPurpose::IntentPolicy
            && requirement.from_currency.as_deref() == Some(currency)
    });
    let requirement = matches.next()?;
    if matches.next().is_some() || requirement.to_currency.as_deref() != Some(base) {
        return None;
    }
    let route = routes.iter().find(|route| {
        route.purpose == SpotRequiredRoutePurpose::IntentPolicy
            && route.from_currency == currency
            && route.to_currency == base
            && route.provider_pair == requirement.provider_pair
    })?;
    if route.state != SpotRequiredRouteState::Qualified {
        return None;
    }

    if requirement.need == FxRequirementNeed::Identity {
        return (currency == base
            && requirement.provider_pair.is_none()
            && requirement.from_currency.as_deref() == Some(currency)
            && requirement.to_currency.as_deref() == Some(base))
        .then(|| amount.to_owned());
    }
    let observation = observation?;
    if observation.requirements != *requirements {
        return None;
    }
    if requirement.need != FxRequirementNeed::ExternalRate {
        return None;
    }
    let pair = requirement.provider_pair.as_deref()?;
    let from = requirement.from_currency.as_deref()?;
    let to = requirement.to_currency.as_deref()?;
    let rate = observation.rates.iter().find(|rate| {
        rate.provider_pair == pair && rate.from_currency == from && rate.to_currency == to
    })?;
    // Crypto routes are supported only by the exact quote object from the same FX response.
    // Compare both original numeric lexemes by decimal value; retain those lexemes for display.
    if pair == "USDTUSD" {
        let crypto_location = match observation.crypto_location? {
            crate::protocol::AlpacaCryptoLocation::Us => "us",
            crate::protocol::AlpacaCryptoLocation::Us1 => "us-1",
            crate::protocol::AlpacaCryptoLocation::Eu1 => "eu-1",
        };
        let request_path =
            format!("/v1beta3/crypto/{crypto_location}/latest/quotes?symbols=USDT%2FUSD");
        let bid = crate::financial_sources::normalize_fx_numeric_text(&rate.bid).ok()?;
        let ask = crate::financial_sources::normalize_fx_numeric_text(&rate.ask).ok()?;
        if !observation.crypto_quotes.iter().any(|quote| {
            quote.provider_symbol == "USDT/USD"
                && quote.request_path == request_path
                && quote.provider_timestamp == rate.provider_timestamp
                && crate::financial_sources::normalize_fx_numeric_text(&quote.bid_price)
                    .is_ok_and(|value| value == bid)
                && crate::financial_sources::normalize_fx_numeric_text(&quote.ask_price)
                    .is_ok_and(|value| value == ask)
                && quote.response_sha256.len() == 64
        }) {
            return None;
        }
    }
    let direction = match side {
        OrderSide::Buy => crate::financial_sources::normalize_fx_numeric_text(&rate.ask).ok()?,
        OrderSide::Sell => crate::financial_sources::normalize_fx_numeric_text(&rate.bid).ok()?,
    };
    crate::portfolio::decimal_mul(amount, &direction).ok()
}

/// The fee evidence situation, derived from the exact-symbol declaration and its backend-owned
/// current branch. A complete current branch plus complete account-bound wallet evidence qualifies
/// only the present estimate; it does not change the separate `fee_currency=UNKNOWN` fact about a
/// future fill.
enum FeeSituation {
    /// The commission observation does not bind the same account, Proposal, symbol or rules.
    EvidenceMismatch,
    /// No current exact-symbol commission observation is bound to the intent.
    EvidenceUnavailable,
    /// The venue declares commission rates but not the fee-charging asset, so the statement is unavailable.
    CurrencyUnknown,
    /// A complete account-bound current branch is eligible, but actual settlement remains unknown.
    ConditionalEstimate,
    /// Declared rates are present but not usable as an exact decimal basis.
    RateUnsupported,
    /// The current branch lacks complete exact-account BASE/QUOTE/BNB rows.
    FundingUnavailable,
}

/// The binding route blocker, if any, in deterministic priority order (structural impossibility
/// precedes any evidence gap, which precedes an unqualified observation).
fn route_blocker(routes: &[SpotRequiredRoute]) -> Option<(SpotFeeFxReasonCode, &'static str)> {
    if routes
        .iter()
        .any(|route| route.state == SpotRequiredRouteState::CurrencyMismatch)
    {
        Some((
            SpotFeeFxReasonCode::ValuationCurrencyMismatch,
            "SPOT_VALUATION_CURRENCY_MISMATCH",
        ))
    } else if routes
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
/// Policy valuation is unqualified unless the current source-use assessment supports that purpose.
///
/// Returns `Ok(None)` only when the immutable identity cannot be bound at all (the capacity
/// evidence and the intent do not describe the same intent); in that case there is nothing to
/// state. Whenever the identity is established but the evidence is missing, unsupported or
/// unqualified, a statement is returned with `outcome != Pass` so it remains visible to a reviewer.
pub(crate) fn derive(
    capacity: &SpotCapacityInputs,
    sources: SpotFeeFxSources<'_>,
    fx_requirements: &FxRequirements,
    fx_observation: Option<&FxRateEvidence>,
    proposal: &OrderProposal,
    side: OrderSide,
    base: &str,
) -> Result<Option<SpotFeeFxStatement>> {
    let SpotFeeFxSources {
        commission,
        source_use_assessment,
        current_fee_model,
        wallet_funding_evidence,
    } = sources;
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
    let policy_valuation_supported = source_use_assessment.is_some_and(|assessment| {
        assessment.status == FxSourceUseAssessmentStatus::SupportedForDeclaredScope
            && assessment
                .supported_purposes
                .iter()
                .any(|purpose| purpose == "POLICY_VALUATION")
    });
    let mut routes = Vec::new();
    for requirement in &fx_requirements.requirements {
        let purpose = match requirement.purpose {
            FxRequirementPurpose::IntentPolicy => SpotRequiredRoutePurpose::IntentPolicy,
            FxRequirementPurpose::IntentFunding if side == OrderSide::Buy => {
                SpotRequiredRoutePurpose::IntentFunding
            }
            _ => continue,
        };
        routes.push(qualify_route(
            purpose,
            requirement,
            fx_observation,
            policy_valuation_supported,
            base,
        ));
    }
    routes.sort_by_key(|route| {
        let purpose_order = match route.purpose {
            SpotRequiredRoutePurpose::IntentPolicy => 0,
            SpotRequiredRoutePurpose::IntentFunding => 1,
        };
        let leg_order = match (
            route.purpose,
            route.from_currency.as_str(),
            route.to_currency.as_str(),
        ) {
            (SpotRequiredRoutePurpose::IntentPolicy, "USDT", "USD") => 0,
            (SpotRequiredRoutePurpose::IntentPolicy, "USD", _) => 1,
            _ => 0,
        };
        (
            purpose_order,
            leg_order,
            route.from_currency.clone(),
            route.to_currency.clone(),
        )
    });

    // Fee evidence comes only from the exact-symbol declaration bound to this account, Proposal,
    // selected rule material and source generation. Account-level comparisons remain historical.
    let observation = capacity.observation.as_ref();
    let fee = fee_situation(
        capacity,
        commission,
        current_fee_model,
        wallet_funding_evidence,
        proposal,
        side,
    );

    let fee_code = match fee {
        FeeSituation::EvidenceMismatch => Some(SpotFeeFxReasonCode::SpotFeeFxEvidenceMismatch),
        FeeSituation::EvidenceUnavailable => Some(SpotFeeFxReasonCode::SpotFeeEvidenceUnavailable),
        FeeSituation::CurrencyUnknown => Some(SpotFeeFxReasonCode::SpotFeeCurrencyUnknown),
        FeeSituation::ConditionalEstimate => None,
        FeeSituation::RateUnsupported => Some(SpotFeeFxReasonCode::SpotFeeRateUnsupported),
        FeeSituation::FundingUnavailable => Some(SpotFeeFxReasonCode::SpotFeeEvidenceUnavailable),
    };

    // Binding priority, most fundamental first: **identity -> route -> fee**.
    //   1. identity: the FX requirement evidence must describe THIS immutable intent (workspace,
    //      Proposal and base currency); a foreign requirement set is never reconciled by assumption.
    //   2. route: a genuinely-required conversion must be supported by the bounded producer.
    //   3. fee: only then does the current fee-estimate contract bind the statement. The actual
    //      future fee currency remains UNKNOWN even when a complete conditional estimate qualifies.
    // All three tiers are reachable. A complete conditional estimate can qualify only after the
    // route tier passes; the non-binding facts are never silently dropped: routes stay in
    // `routes`/`reason` and actual fee currency stays `UNKNOWN` in the statement.
    let route_block = route_blocker(&routes);
    let (reason_code, binding_blocker_code) = if !requirements_match {
        (
            SpotFeeFxReasonCode::SpotFeeFxEvidenceMismatch,
            Some("SPOT_FEE_FX_EVIDENCE_MISMATCH".to_string()),
        )
    } else if let Some((code, code_label)) = route_block {
        (code, Some(code_label.to_string()))
    } else if let Some(code) = fee_code {
        (
            code,
            Some(
                match code {
                    SpotFeeFxReasonCode::SpotFeeFxEvidenceMismatch => {
                        "SPOT_FEE_FX_EVIDENCE_MISMATCH"
                    }
                    SpotFeeFxReasonCode::SpotFeeEvidenceUnavailable => {
                        "SPOT_FEE_EVIDENCE_UNAVAILABLE"
                    }
                    SpotFeeFxReasonCode::SpotFeeCurrencyUnknown => "SPOT_FEE_CURRENCY_UNKNOWN",
                    SpotFeeFxReasonCode::SpotFeeRateUnsupported => "SPOT_FEE_RATE_UNSUPPORTED",
                    _ => "SPOT_FEE_FX_QUALIFICATION_BLOCKED",
                }
                .to_string(),
            ),
        )
    } else {
        // Complete conditional estimate and every genuinely-required route pass. This still grants
        // no execution authority; the public Prepare path revalidates the whole current plan.
        (SpotFeeFxReasonCode::SpotFeeFxQualified, None)
    };
    let outcome = if binding_blocker_code.is_none() {
        RiskCheckOutcome::Pass
    } else {
        RiskCheckOutcome::Unavailable
    };

    // A single-line, human-readable reason that reports the binding fact under the same
    // identity -> route -> fee priority and never hides a co-occurring fact. It must not contain a
    // newline: the review UI joins blockers with ' · ' on one line.
    let route_fact = routes
        .iter()
        .find(|route| route.state == SpotRequiredRouteState::UnsupportedRoute)
        .or_else(|| {
            routes
                .iter()
                .find(|route| route.state == SpotRequiredRouteState::CurrencyMismatch)
        })
        .or_else(|| {
            routes.iter().find(|route| {
                matches!(
                    route.state,
                    SpotRequiredRouteState::EvidenceUnavailable
                        | SpotRequiredRouteState::Unqualified
                )
            })
        });
    let fee_fact = match fee {
        FeeSituation::EvidenceMismatch => {
            "The exact-symbol commission evidence does not match the current account, Proposal, source or rules; fee currency remains UNKNOWN and no fee amount is fabricated."
        }
        FeeSituation::EvidenceUnavailable => {
            "No current exact-symbol commission observation is bound to this unchanged intent; fee currency remains UNKNOWN and no fee amount is fabricated."
        }
        FeeSituation::RateUnsupported => {
            "The exact-symbol commission terms are incomplete or contain an unresolved active field; fee currency remains UNKNOWN and no fee amount is fabricated."
        }
        FeeSituation::CurrencyUnknown => {
            "The exact-symbol commission terms are current, but the venue does not declare the future fee-charging asset, so fee currency remains UNKNOWN and no fee amount is fabricated."
        }
        FeeSituation::ConditionalEstimate => {
            "The complete account-bound current fee estimate is eligible for review subject to independent gates; the actual fee asset and amount remain pending and are not guaranteed."
        }
        FeeSituation::FundingUnavailable => {
            "The current fee branch is modeled, but exact BASE/QUOTE/BNB wallet coverage is incomplete or does not match this account and Proposal; the owning financial plan remains unavailable."
        }
    };
    let reason = if !requirements_match {
        "The FX requirement evidence does not describe this unchanged intent, so no fee/FX statement can bind it.".to_string()
    } else if route_block.is_some() {
        match route_fact {
            Some(route) => format!(
                "The genuinely-required {} FX route {} -> {} is {}; {fee_fact}",
                match route.purpose {
                    SpotRequiredRoutePurpose::IntentPolicy => "price-only policy-valuation",
                    SpotRequiredRoutePurpose::IntentFunding => "funding/execution",
                },
                route.from_currency,
                route.to_currency,
                match (route.purpose, route.state) {
                    (_, SpotRequiredRouteState::UnsupportedRoute) =>
                        "unsupported by the selected bounded FX producer, so no rate can be used",
                    (_, SpotRequiredRouteState::CurrencyMismatch) =>
                        "a raw quote is visible but its result currency differs from the workspace base, so no monetary amount is derived",
                    (
                        SpotRequiredRoutePurpose::IntentPolicy,
                        SpotRequiredRouteState::EvidenceUnavailable,
                    ) => "unavailable because no current proposal-scoped quote is bound",
                    (
                        SpotRequiredRoutePurpose::IntentFunding,
                        SpotRequiredRouteState::EvidenceUnavailable,
                    ) => "unavailable because no current proposal-scoped execution rate is bound",
                    (
                        SpotRequiredRoutePurpose::IntentPolicy,
                        SpotRequiredRouteState::Unqualified,
                    ) => "outside the reviewed price-only policy-valuation constraints",
                    (
                        SpotRequiredRoutePurpose::IntentFunding,
                        SpotRequiredRouteState::Unqualified,
                    ) =>
                        "execution-unqualified because the read-only rate establishes no conservative execution cost",
                    (_, SpotRequiredRouteState::Qualified) => "qualified for its declared purpose",
                }
            ),
            None => fee_fact.to_string(),
        }
    } else {
        match reason_code {
            SpotFeeFxReasonCode::SpotFeeFxQualified => "The owning current fee estimate and every genuinely-required purpose-scoped FX route are complete for their declared conditions; the future fee asset and amount remain unguaranteed, and this carries no execution authority by itself.".to_string(),
            _ => fee_fact.to_string(),
        }
    };

    let fee_evidence_version = digest(&fee_evidence_material(commission)?)?;
    let route_evidence_version = digest(&(
        fx_requirements.material_version.as_str(),
        fx_observation.map(|observation| observation.material_version.as_str()),
        &routes,
    ))?;
    let workspace_base_expected_spend = workspace_base_expected_spend(
        proposal,
        base,
        fx_requirements,
        requirements_match,
        &routes,
        fx_observation,
        side,
    );

    Ok(Some(SpotFeeFxStatement {
        wallet_funding_evidence: None,
        current_fee_model: None,
        current_fee_model_failure: None,
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
        fee_rate_basis: None,
        declared_maker_rate: None,
        declared_taker_rate: None,
        declared_buyer_rate: None,
        declared_seller_rate: None,
        expected_fee: None,
        expected_spend_quote: proposal.estimated_notional.clone(),
        maximum_authorized_spend_quote: proposal.fields.maximum_spend.clone(),
        base_currency: base.to_string(),
        workspace_base_expected_spend,
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

#[cfg(test)]
mod tests {
    //! Contract tests for [`super::derive`]'s decision table.
    //!
    //! Each test builds a **fully supported** intent route (an `INTENT_POLICY` `EUR -> USD`
    //! exact EUR identity to isolate fee facts; supported EURUSD qualifies only for price-only
    //! policy valuation while its provider quality remains unqualified. Identity requires no
    //! external rate; only the declared-commission evidence varies in fee cases.
    //! They pin the four fee-side outcomes of `derive` and prove that identity is bound rather than
    //! silently succeeding.
    //!
    //! These are contract tests, not end-to-end reachability tests: synthetic inputs can exercise
    //! a current branch while the future fee asset and provider rights remain unverified. Full
    //! positive public-producer reachability remains mandatory later work.
    use super::*;
    use crate::protocol::{
        ExecutionContext, ExecutionReservation, ExecutionReservationStatus,
        FinancialEvidenceBinding, FinancialEvidenceQuality, FxObservedRate, FxSourceUseAssessment,
        FxSourceUseAssessmentStatus, FxSourceUseEvidenceProvenance, MarketDataStatus,
        OrderDraftFields, OrderProposalStatus, OrderQuantity, OrderQuantityType, OrderType,
        ProposalReferenceStatus, Remediation, TimeConfidence, TimeInForce, TimeStatus,
    };
    use crate::risk::RiskCheckOutcome;
    use crate::spot_capacity::{
        SpotCapacityBalance, SpotCapacityCounts, SpotCapacityObservation, SpotCapacityPurpose,
        SpotCapacityQuality, SpotCapacityRead, SpotCapacityReadKind, SpotCapacityStatus,
        SpotDeclaredCommission,
    };
    use crate::spot_commission::{
        SpotCommissionDiscount, SpotCommissionInputs, SpotCommissionObservation,
        SpotCommissionQuality, SpotCommissionRates, SpotCommissionReadKind, SpotCommissionReceipt,
        SpotCommissionStatus,
    };

    /// A full 71-character `sha256:` binding version (`sha256:` + 64 identical hex digits), so the
    /// declared wire bound and the real value agree in the test exactly as they must on the wire.
    fn hash(fill: char) -> String {
        format!("sha256:{}", fill.to_string().repeat(64))
    }

    fn proposal() -> OrderProposal {
        OrderProposal {
            proposal_id: "proposal-1".into(),
            workspace_id: "workspace-1".into(),
            draft_id: "draft-1".into(),
            draft_version: 1,
            proposal_hash: hash('1'),
            fields: OrderDraftFields {
                account_id: Some("account-1".into()),
                venue: "binance".into(),
                environment: ExecutionContext::BinanceLive,
                instrument_id: "binance:sp:EUR/USD".into(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: OrderQuantity {
                    r#type: OrderQuantityType::Quote,
                    value: "100".into(),
                },
                limit_price: None,
                maximum_spend: Some("100".into()),
                time_in_force: TimeInForce::Ioc,
                client_label: None,
            },
            estimated_notional: Some("100".into()),
            estimated_notional_currency: Some("USD".into()),
            estimated_notional_reason: None,
            policy_version: None,
            policy_state_version: None,
            policy_status: ProposalReferenceStatus::Available,
            policy_reference_reason: "Policy is available.".into(),
            market_snapshot_id: None,
            market_status: MarketDataStatus::Available,
            market_reference_reason: "Market is available.".into(),
            status: OrderProposalStatus::NeedsApproval,
            invalidation_reason: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            state_version: "state-1".into(),
            history: Vec::new(),
        }
    }

    fn requirements() -> FxRequirements {
        FxRequirements {
            workspace_id: "workspace-1".into(),
            base_currency: "USD".into(),
            proposal_id: Some("proposal-1".into()),
            material_version: "2".repeat(64),
            requirements: vec![FxRouteRequirement {
                purpose: FxRequirementPurpose::IntentPolicy,
                connection_id: None,
                account_version: None,
                from_currency: Some("EUR".into()),
                to_currency: Some("USD".into()),
                need: FxRequirementNeed::ExternalRate,
                provider_pair: Some("EURUSD".into()),
                provider_dataset: Some(crate::protocol::FxProviderDataset::ForexLatestRates),
                reason: "The intent policy requires the EUR input to settle in USD.".into(),
            }],
        }
    }

    fn evidence() -> FxRateEvidence {
        FxRateEvidence {
            binding: FinancialEvidenceBinding {
                connection_id: "connection-1".into(),
                account_version: "1".into(),
                source_version: "1".into(),
                binding_version: "3".repeat(64),
            },
            material_version: "4".repeat(64),
            provider_quality: FinancialEvidenceQuality::UnqualifiedFxRate,
            observed_at: "2024-01-01T00:00:01Z".into(),
            provider_observed_at: None,
            requirements: requirements(),
            crypto_location: None,
            crypto_quotes: Vec::new(),
            rates: vec![FxObservedRate {
                provider_pair: "EURUSD".into(),
                from_currency: "EUR".into(),
                to_currency: "USD".into(),
                bid: "1.0800".into(),
                ask: "1.0801".into(),
                mid: Some("1.08005".into()),
                provider_timestamp: "2024-01-01T00:00:00Z".into(),
            }],
        }
    }

    fn source_use_assessment(supported: bool) -> FxSourceUseAssessment {
        FxSourceUseAssessment {
            policy_version: "ALPACA_PERSONAL_LOCAL_V1".into(),
            status: if supported {
                FxSourceUseAssessmentStatus::SupportedForDeclaredScope
            } else {
                FxSourceUseAssessmentStatus::Unresolved
            },
            provenance: if supported {
                FxSourceUseEvidenceProvenance::UserReportedProviderSupport
            } else {
                FxSourceUseEvidenceProvenance::UserReportedProviderUi
            },
            context: None,
            documents: Vec::new(),
            supported_purposes: if supported {
                vec!["POLICY_VALUATION".into()]
            } else {
                Vec::new()
            },
            reason: "Synthetic source-use assessment for a contract test.".into(),
        }
    }

    fn commission(maker: &str, taker: &str) -> SpotDeclaredCommission {
        SpotDeclaredCommission {
            maker: maker.into(),
            taker: taker.into(),
            buyer: "0.0000".into(),
            seller: "0.0000".into(),
            extension_keys: Vec::new(),
        }
    }

    fn symbol_commission(maker: &str, taker: &str) -> SpotCommissionInputs {
        let rates = |maker: &str, taker: &str| SpotCommissionRates {
            maker: maker.into(),
            taker: taker.into(),
            buyer: "0".into(),
            seller: "0".into(),
        };
        SpotCommissionInputs {
            workspace_id: "workspace-1".into(),
            proposal_id: "proposal-1".into(),
            proposal_hash: hash('1'),
            account_id: "account-1".into(),
            instrument_id: "binance:sp:EUR/USD".into(),
            base_asset: "EUR".into(),
            quote_asset: "USD".into(),
            side: OrderSide::Buy,
            source_version: "1".into(),
            rule_material_version: None,
            status: SpotCommissionStatus::Observed,
            qualification: RiskCheckOutcome::Unavailable,
            observation: Some(SpotCommissionObservation {
                collection_id: "commission:1".into(),
                remote_account_id: "123456".into(),
                symbol: "EURUSD".into(),
                quality: SpotCommissionQuality::ReadOnlySymbolCommission,
                standard_commission: rates(maker, taker),
                tax_commission: rates("0", "0"),
                special_commission: rates("0", "0"),
                discount: SpotCommissionDiscount {
                    enabled_for_account: true,
                    enabled_for_symbol: true,
                    discount_asset: "BNB".into(),
                    discount: "0.75".into(),
                },
                terms_complete: true,
                received_asset: "EUR".into(),
                conditional_discount_asset: Some("BNB".into()),
                charging_asset_qualified: false,
                provider_observed_at: None,
                reads: vec![
                    SpotCommissionReceipt {
                        kind: SpotCommissionReadKind::AccountIdentity,
                        started_at: "2024-01-01T00:00:00Z".into(),
                        received_at: "2024-01-01T00:00:01Z".into(),
                        digest: "a".repeat(64),
                    },
                    SpotCommissionReceipt {
                        kind: SpotCommissionReadKind::SymbolCommission,
                        started_at: "2024-01-01T00:00:01Z".into(),
                        received_at: "2024-01-01T00:00:02Z".into(),
                        digest: "b".repeat(64),
                    },
                ],
                extension_keys: Vec::new(),
                unresolved_obligations: vec![
                    "NON_ATOMIC_PROVIDER_SNAPSHOT".into(),
                    "FEE_AMOUNT_AND_CHARGING_ASSET_NOT_EXECUTION_QUALIFIED".into(),
                ],
            }),
            failure: None,
            binding_version: "binding-1".into(),
            state_version: "commission-state-1".into(),
        }
    }

    fn capacity(declared_commission: Option<SpotDeclaredCommission>) -> SpotCapacityInputs {
        SpotCapacityInputs {
            workspace_id: "workspace-1".into(),
            proposal_id: "proposal-1".into(),
            proposal_hash: hash('1'),
            account_id: "account-1".into(),
            instrument_id: "binance:sp:EUR/USD".into(),
            base_asset: "EUR".into(),
            quote_asset: "USD".into(),
            source_version: "1".into(),
            rule_material_version: None,
            purposes: vec![SpotCapacityPurpose::OpenOrdersAccount],
            status: SpotCapacityStatus::Observed,
            qualification: RiskCheckOutcome::Pass,
            qualification_reason: "The declared capacity is observed.".into(),
            observation: Some(SpotCapacityObservation {
                wallet_balances: Vec::new(),
                collection_id: "collection-1".into(),
                quality: SpotCapacityQuality::ReadOnlySpotCapacity,
                atomic: false,
                provider_observed_at: Some("2024-01-01T00:00:00Z".into()),
                counts: SpotCapacityCounts {
                    account_open_orders: Some("0".into()),
                    symbol_open_orders: "0".into(),
                    account_algo_orders: None,
                    symbol_algo_orders: None,
                    account_iceberg_orders: None,
                    symbol_iceberg_orders: "0".into(),
                    classifications_complete: true,
                    order_coverage_complete: true,
                    account_open_order_lists: None,
                    symbol_open_order_lists: None,
                    missing_list_legs: None,
                    list_coverage_complete: None,
                },
                base_balance: None,
                position: None,
                declared_commission,
                unresolved_obligations: vec![
                    "NON_ATOMIC_PROVIDER_SNAPSHOT".into(),
                    "DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED".into(),
                ],
                reads: vec![SpotCapacityRead {
                    kind: SpotCapacityReadKind::Account,
                    started_at: "2024-01-01T00:00:00Z".into(),
                    received_at: "2024-01-01T00:00:01Z".into(),
                    digest: "a".repeat(64),
                    oldest_provider_update_time_ms: None,
                    latest_provider_update_time_ms: None,
                    oldest_provider_transaction_time_ms: None,
                    latest_provider_transaction_time_ms: None,
                }],
            }),
            failure: None,
            binding_version: "binding-1".into(),
            state_version: "state-1".into(),
        }
    }

    /// Derive against the supported `EUR -> USD` route, returning the always-visible statement.
    fn derive_supported_with_model(
        symbol_evidence: Option<SpotCommissionInputs>,
        current_fee_model: Option<&crate::spot_fee_model::SpotCurrentFeeModel>,
        wallet_funding_evidence: Option<&crate::spot_fee_model::SpotWalletFundingEvidence>,
    ) -> SpotFeeFxStatement {
        let capacity = capacity(None);
        let requirements = requirements();
        let evidence = evidence();
        let source_use = source_use_assessment(true);
        derive(
            &capacity,
            SpotFeeFxSources {
                commission: symbol_evidence.as_ref(),
                source_use_assessment: Some(&source_use),
                current_fee_model,
                wallet_funding_evidence,
            },
            &requirements,
            Some(&evidence),
            &proposal(),
            OrderSide::Buy,
            "USD",
        )
        .expect("derive never fails for a bound identity")
        .expect("a bound identity always yields a visible statement")
    }

    fn derive_supported(symbol_evidence: Option<SpotCommissionInputs>) -> SpotFeeFxStatement {
        derive_supported_with_model(symbol_evidence, None, None)
    }

    fn complete_wallet_funding_evidence() -> crate::spot_fee_model::SpotWalletFundingEvidence {
        use crate::spot_fee_model::{
            SpotWalletBalanceEvidence, SpotWalletFundingEvidence, SpotWalletFundingStatus,
        };
        let balance = |asset: &str| SpotWalletBalanceEvidence {
            asset: asset.into(),
            free: "100".into(),
            locked: "0".into(),
            local_claims: "0".into(),
            effective_free: Some("100".into()),
        };
        SpotWalletFundingEvidence {
            status: SpotWalletFundingStatus::Complete,
            required_assets: vec!["EUR".into(), "USD".into(), "BNB".into()],
            missing_assets: Vec::new(),
            balances: vec![balance("EUR"), balance("USD"), balance("BNB")],
            workspace_id: "workspace-1".into(),
            proposal_id: "proposal-1".into(),
            proposal_hash: hash('1'),
            account_id: "account-1".into(),
            source_version: "source-1".into(),
            rule_material_version: None,
            capacity_binding_version: "binding-1".into(),
            collection_id: Some("collection-1".into()),
            account_read_started_at: Some("2024-01-01T00:00:00Z".into()),
            first_receipt: Some("2024-01-01T00:00:01Z".into()),
            original_digest: Some("a".repeat(64)),
            reason: None,
        }
    }

    fn current_received_asset_model() -> crate::spot_fee_model::SpotCurrentFeeModel {
        serde_json::from_value(serde_json::json!({
            "method": "SPOT_THREE_FAMILY_RATIONAL_CEIL18_V1",
            "class": "CURRENT_ORDER_SCENARIO",
            "receivedAsset": "EUR",
            "grossReceived": "100",
            "fundingPlanPreview": {
                "principalClaim": {"asset": "USD", "amount": "100"},
                "feeDisposition": "RECEIVED_ASSET_DEDUCTION",
                "fallbackFeeAsset": "EUR",
                "fallbackFeeAmount": "0.1",
                "fallbackNetReceipt": "99.9"
            },
            "fallback": {
                "role": "TAKER",
                "standard": "0.1",
                "tax": "0",
                "special": "0",
                "total": "0.1",
                "netReceived": "99.9"
            },
            "chargingCondition": "DISCOUNT_DISABLED_CURRENT_FALLBACK",
            "limitations": ["VENUE_PER_FILL_ROUNDING_UNPROVEN"]
        }))
        .expect("complete typed current fee model")
    }

    /// Isolate fee facts using exact EUR identity; retained contract evidence, not product acceptance.
    fn derive_identity(symbol_evidence: Option<SpotCommissionInputs>) -> SpotFeeFxStatement {
        let capacity = capacity(None);
        let mut requirements = requirements();
        requirements.base_currency = "EUR".into();
        requirements.requirements[0].to_currency = Some("EUR".into());
        requirements.requirements[0].need = FxRequirementNeed::Identity;
        requirements.requirements[0].provider_pair = None;
        derive(
            &capacity,
            SpotFeeFxSources {
                commission: symbol_evidence.as_ref(),
                source_use_assessment: None,
                current_fee_model: None,
                wallet_funding_evidence: None,
            },
            &requirements,
            None,
            &proposal(),
            OrderSide::Buy,
            "EUR",
        )
        .expect("identity contract")
        .expect("bound statement")
    }

    #[test]
    fn qualified_policy_spend_uses_ask_for_buy_and_bid_for_sell() {
        let mut proposal = proposal();
        proposal.estimated_notional = Some("100".into());
        proposal.estimated_notional_currency = Some("EUR".into());
        let requirements = requirements();
        let observation = evidence();
        let policy_requirement = requirements
            .requirements
            .iter()
            .find(|requirement| requirement.purpose == FxRequirementPurpose::IntentPolicy)
            .expect("policy requirement");
        let route = qualify_route(
            SpotRequiredRoutePurpose::IntentPolicy,
            policy_requirement,
            Some(&observation),
            true,
            "USD",
        );
        assert_eq!(route.state, SpotRequiredRouteState::Qualified);
        assert_eq!(
            workspace_base_expected_spend(
                &proposal,
                "USD",
                &requirements,
                true,
                std::slice::from_ref(&route),
                Some(&observation),
                OrderSide::Buy,
            )
            .as_deref(),
            Some("108.01")
        );
        assert_eq!(
            workspace_base_expected_spend(
                &proposal,
                "USD",
                &requirements,
                true,
                std::slice::from_ref(&route),
                Some(&observation),
                OrderSide::Sell,
            )
            .as_deref(),
            Some("108")
        );
    }

    #[test]
    fn identity_policy_spend_preserves_the_proposal_amount_without_fx_evidence() {
        let proposal = proposal();
        let mut requirements = requirements();
        let requirement = &mut requirements.requirements[0];
        requirement.from_currency = Some("USD".into());
        requirement.to_currency = Some("USD".into());
        requirement.need = FxRequirementNeed::Identity;
        requirement.provider_pair = None;
        requirement.provider_dataset = None;
        let route = qualify_route(
            SpotRequiredRoutePurpose::IntentPolicy,
            requirement,
            None,
            false,
            "USD",
        );
        assert_eq!(route.state, SpotRequiredRouteState::Qualified);
        assert_eq!(
            workspace_base_expected_spend(
                &proposal,
                "USD",
                &requirements,
                true,
                std::slice::from_ref(&route),
                None,
                OrderSide::Buy,
            )
            .as_deref(),
            Some("100")
        );
    }

    #[test]
    fn active_non_bnb_place_reservation_keeps_discounted_bnb_funding_unresolved() {
        let mut proposal = proposal();
        proposal.fields.order_type = OrderType::Limit;
        proposal.fields.quantity = OrderQuantity {
            r#type: OrderQuantityType::Base,
            value: "0.001".into(),
        };
        proposal.fields.limit_price = Some("60000".into());
        proposal.fields.time_in_force = TimeInForce::Gtc;

        let mut capacity = capacity(None);
        capacity.observation.as_mut().unwrap().wallet_balances = vec![
            SpotCapacityBalance {
                asset: "EUR".into(),
                free: "2.000".into(),
                locked: "0".into(),
            },
            SpotCapacityBalance {
                asset: "USD".into(),
                free: "500".into(),
                locked: "300".into(),
            },
            SpotCapacityBalance {
                asset: "BNB".into(),
                free: "1.00000000".into(),
                locked: "0".into(),
            },
        ];
        let reservation = ExecutionReservation {
            reservation_id: "reservation-1".into(),
            workspace_id: "workspace-1".into(),
            account_id: "account-1".into(),
            attempt_id: "attempt-1".into(),
            proposal_id: "proposal-1".into(),
            proposal_hash: hash('1'),
            instrument_id: "binance:sp:EUR/USD".into(),
            side: OrderSide::Buy,
            capacity_key: "cash:USD".into(),
            amount: "100".into(),
            unit: "USD".into(),
            asset_claims: Vec::new(),
            fee_settlement_plan: None,
            asset_capacities: Vec::new(),
            notional: "100".into(),
            notional_currency: "USD".into(),
            workspace_notional: Some("100".into()),
            workspace_currency: "USD".into(),
            broker_available: "500".into(),
            existing_reservations: "0".into(),
            effective_available: "400".into(),
            capacity_projection: None,
            account_state_version: "account-state-1".into(),
            status: ExecutionReservationStatus::Active,
            created_at: "2024-01-01T00:00:00Z".into(),
            state_version: "reservation-state-1".into(),
        };
        let legacy_projection = serde_json::to_value(&reservation).unwrap();
        assert!(legacy_projection.get("assetClaims").is_none());
        let restored_legacy: ExecutionReservation =
            serde_json::from_value(legacy_projection).unwrap();
        assert!(restored_legacy.asset_claims.is_empty());
        let mut second_claim = reservation.clone();
        second_claim.reservation_id = "reservation-2".into();
        second_claim.amount = "20".into();
        let mut vector_claim = reservation.clone();
        vector_claim.reservation_id = "reservation-vector".into();
        vector_claim.amount = "20".into();
        vector_claim.asset_claims = vec![
            crate::protocol::ExecutionReservationAssetClaim {
                asset: "USD".into(),
                amount: "20".into(),
                purpose: crate::protocol::ExecutionReservationClaimPurpose::Principal,
            },
            crate::protocol::ExecutionReservationAssetClaim {
                asset: "BNB".into(),
                amount: "0.25".into(),
                purpose: crate::protocol::ExecutionReservationClaimPurpose::Fee,
            },
        ];
        let mut released_claim = reservation.clone();
        released_claim.reservation_id = "reservation-released".into();
        released_claim.amount = "1000".into();
        released_claim.status = ExecutionReservationStatus::Released;
        let mut other_account_claim = reservation.clone();
        other_account_claim.reservation_id = "reservation-other-account".into();
        other_account_claim.account_id = "another-account".into();
        other_account_claim.amount = "1000".into();
        let clock = TimeStatus {
            workspace_id: "workspace-1".into(),
            confidence: TimeConfidence::Trusted,
            wall_clock: "2024-01-01T00:00:03Z".into(),
            monotonic_ms: 3,
            provider_offset_ms: None,
            observed_at: "2024-01-01T00:00:03Z".into(),
            reason: "Test clock is trusted.".into(),
            remediation: Remediation {
                id: "time.revalidate".into(),
                label: "Revalidate time".into(),
            },
        };

        let reservations = [
            reservation.clone(),
            second_claim,
            vector_claim,
            released_claim,
            other_account_claim,
        ];
        let wallet =
            crate::spot_fee_model::wallet_funding_evidence(&proposal, &capacity, &reservations);
        assert_eq!(
            wallet.status,
            crate::spot_fee_model::SpotWalletFundingStatus::Complete
        );
        let usd = wallet
            .balances
            .iter()
            .find(|balance| balance.asset == "USD")
            .unwrap();
        assert_eq!(usd.free, "500");
        assert_eq!(usd.locked, "300");
        assert_eq!(usd.local_claims, "140");
        assert_eq!(usd.effective_free.as_deref(), Some("360"));
        let bnb = wallet
            .balances
            .iter()
            .find(|balance| balance.asset == "BNB")
            .unwrap();
        assert_eq!(bnb.free, "1.00000000");
        assert_eq!(bnb.locked, "0");
        assert_eq!(bnb.local_claims, "0.25");
        assert_eq!(bnb.effective_free.as_deref(), Some("0.75"));
        let model = crate::spot_fee_model::current_fallback(
            &symbol_commission("0.001", "0.002"),
            &proposal,
            &capacity,
            &reservations,
            &wallet,
            crate::spot_fee_model::MarketInputs {
                detail: None,
                fee_market: None,
                clock: &clock,
                policy: None,
            },
        )
        .expect("an uncertain BNB balance remains a visible current fee model");

        assert!(model.bnb_funding_evidence.is_none());
        assert_eq!(
            model.charging_condition,
            crate::spot_fee_model::ChargingCondition::ConditionalBnbInputsRequired
        );
        assert!(model.bnb_fee.is_none());

        let mut bnb_primary_with_usd_fee = reservation.clone();
        bnb_primary_with_usd_fee.capacity_key = "cash:BNB".into();
        bnb_primary_with_usd_fee.amount = "0.1".into();
        bnb_primary_with_usd_fee.unit = "BNB".into();
        bnb_primary_with_usd_fee.asset_claims = vec![
            crate::protocol::ExecutionReservationAssetClaim {
                asset: "BNB".into(),
                amount: "0.1".into(),
                purpose: crate::protocol::ExecutionReservationClaimPurpose::Principal,
            },
            crate::protocol::ExecutionReservationAssetClaim {
                asset: "USD".into(),
                amount: "10".into(),
                purpose: crate::protocol::ExecutionReservationClaimPurpose::Fee,
            },
        ];
        let bnb_primary_reservations = [bnb_primary_with_usd_fee];
        let bnb_primary_wallet = crate::spot_fee_model::wallet_funding_evidence(
            &proposal,
            &capacity,
            &bnb_primary_reservations,
        );
        assert_eq!(
            bnb_primary_wallet.status,
            crate::spot_fee_model::SpotWalletFundingStatus::Complete
        );
        let bnb_primary = bnb_primary_wallet
            .balances
            .iter()
            .find(|balance| balance.asset == "BNB")
            .unwrap();
        assert_eq!(bnb_primary.local_claims, "0.1");
        let usd_primary = bnb_primary_wallet
            .balances
            .iter()
            .find(|balance| balance.asset == "USD")
            .unwrap();
        assert_eq!(usd_primary.local_claims, "10");
        let uncertain_model = crate::spot_fee_model::current_fallback(
            &symbol_commission("0.001", "0.002"),
            &proposal,
            &capacity,
            &bnb_primary_reservations,
            &bnb_primary_wallet,
            crate::spot_fee_model::MarketInputs {
                detail: None,
                fee_market: None,
                clock: &clock,
                policy: None,
            },
        )
        .expect("An outstanding non-BNB fee claim keeps BNB charging uncertain");
        assert!(uncertain_model.bnb_funding_evidence.is_none());
        assert_eq!(
            uncertain_model.charging_condition,
            crate::spot_fee_model::ChargingCondition::ConditionalBnbInputsRequired
        );
    }

    #[test]
    fn supported_local_use_and_read_only_fx_qualify_policy_valuation_but_not_execution() {
        let requirements = requirements();
        let evidence = evidence();
        let route = qualify_route(
            SpotRequiredRoutePurpose::IntentPolicy,
            &requirements.requirements[0],
            Some(&evidence),
            true,
            "USD",
        );
        assert_eq!(route.state, SpotRequiredRouteState::Qualified);
        assert_eq!(
            route.provider_quality.as_deref(),
            Some("UNQUALIFIED_FX_RATE")
        );
        assert_eq!(
            route.provider_timestamp.as_deref(),
            Some("2024-01-01T00:00:00Z")
        );
        assert!(route.conservative_cost.is_none());
        assert!(route.reason.contains("price-only policy valuation"));
        assert!(
            route
                .reason
                .contains("execution qualification remain separate")
        );
        let funding_route = qualify_route(
            SpotRequiredRoutePurpose::IntentFunding,
            &requirements.requirements[0],
            Some(&evidence),
            false,
            "USD",
        );
        assert_eq!(funding_route.state, SpotRequiredRouteState::Unqualified);
        assert!(
            funding_route
                .reason
                .contains("execution quality and conservative execution cost")
        );
        let statement = derive_supported(Some(symbol_commission("0.0010", "0.0015")));
        assert_eq!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotFeeCurrencyUnknown
        );
        assert_eq!(
            statement.binding_blocker.as_deref(),
            Some("SPOT_FEE_CURRENCY_UNKNOWN")
        );
        assert!(statement.reason.contains("fee currency remains UNKNOWN"));
        assert!(statement.reason.contains("fee currency remains UNKNOWN"));
    }

    #[test]
    fn raw_provider_exponent_tokens_are_normalized_only_for_monetary_math() {
        let mut proposal = proposal();
        proposal.estimated_notional = Some("100".into());
        proposal.estimated_notional_currency = Some("EUR".into());
        let requirements = requirements();
        let mut observation = evidence();
        observation.rates[0].bid = "1.08e0".into();
        observation.rates[0].ask = "1.0801e0".into();
        let policy_requirement = requirements
            .requirements
            .iter()
            .find(|requirement| requirement.purpose == FxRequirementPurpose::IntentPolicy)
            .expect("policy requirement");
        let route = qualify_route(
            SpotRequiredRoutePurpose::IntentPolicy,
            policy_requirement,
            Some(&observation),
            true,
            "USD",
        );
        assert_eq!(route.state, SpotRequiredRouteState::Qualified);
        assert_eq!(observation.rates[0].bid, "1.08e0");
        assert_eq!(observation.rates[0].ask, "1.0801e0");
        assert_eq!(
            workspace_base_expected_spend(
                &proposal,
                "USD",
                &requirements,
                true,
                std::slice::from_ref(&route),
                Some(&observation),
                OrderSide::Buy,
            )
            .as_deref(),
            Some("108.01")
        );
    }

    #[test]
    fn shared_commission_binding_rejects_changed_capacity_and_symbol_identity() {
        let proposal = proposal();
        let capacity = capacity(None);
        let commission = symbol_commission("0.0010", "0.0015");

        assert!(commission.matches_capacity_binding(&proposal, &capacity, OrderSide::Buy));
        assert!(
            commission
                .observation
                .as_ref()
                .expect("fixture has an observation")
                .matches_capacity_symbol(&capacity, OrderSide::Buy)
        );

        let mut changed_capacity = capacity.clone();
        changed_capacity.account_id = "another-account".into();
        assert!(!commission.matches_capacity_binding(&proposal, &changed_capacity, OrderSide::Buy));

        let mut changed_commission = commission;
        changed_commission.observation.as_mut().unwrap().symbol = "BTCUSDT".into();
        assert!(
            !changed_commission
                .observation
                .as_ref()
                .unwrap()
                .matches_capacity_symbol(&capacity, OrderSide::Buy)
        );
    }

    #[test]
    fn policy_valuation_route_requires_a_current_supported_source_use_purpose() {
        let capacity = capacity(None);
        let requirements = requirements();
        let evidence = evidence();
        let unresolved = source_use_assessment(false);
        let statement = derive(
            &capacity,
            SpotFeeFxSources {
                commission: Some(&symbol_commission("0.0010", "0.0015")),
                source_use_assessment: Some(&unresolved),
                current_fee_model: None,
                wallet_funding_evidence: None,
            },
            &requirements,
            Some(&evidence),
            &proposal(),
            OrderSide::Buy,
            "USD",
        )
        .expect("derive never fails for a bound identity")
        .expect("a bound identity always yields a visible statement");

        assert_eq!(statement.routes.len(), 1);
        assert_eq!(
            statement.routes[0].state,
            SpotRequiredRouteState::Unqualified
        );
        assert!(
            statement.routes[0]
                .reason
                .contains("does not support POLICY_VALUATION")
        );
        assert_eq!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotRequiredFxUnqualified
        );
        assert_eq!(
            statement.binding_blocker.as_deref(),
            Some("SPOT_REQUIRED_FX_UNQUALIFIED")
        );

        let supported = derive_supported(Some(symbol_commission("0.0010", "0.0015")));
        assert_eq!(supported.routes[0].state, SpotRequiredRouteState::Qualified);
    }

    #[test]
    fn fails_closed_with_currency_unknown_when_rates_are_valid_but_the_fee_asset_is_undeclared() {
        let statement = derive_identity(Some(symbol_commission("0.0010", "0.0015")));
        assert_eq!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotFeeCurrencyUnknown
        );
        assert_eq!(statement.outcome, RiskCheckOutcome::Unavailable);
        assert_eq!(
            statement.binding_blocker.as_deref(),
            Some("SPOT_FEE_CURRENCY_UNKNOWN")
        );
        assert_eq!(statement.fee_currency, "UNKNOWN");
        assert_eq!(statement.fee_currency_origin, "UNKNOWN");
        // Account-level comparisons are not the owning fee basis.
        assert!(statement.fee_rate_basis.is_none());
        // No amount is fabricated from an undeclared fee asset.
        assert!(statement.expected_fee.is_none());
        assert!(statement.declared_maker_rate.is_none());
        assert!(statement.declared_taker_rate.is_none());
        // Exact identity requires no external rate and isolates the fee-class fact.
        assert_eq!(statement.routes.len(), 1);
        assert_eq!(statement.routes[0].state, SpotRequiredRouteState::Qualified);
    }

    #[test]
    fn fails_closed_with_rate_unsupported_when_the_declared_rates_are_not_exact_decimals() {
        // Incomplete exact-symbol terms cannot provide the owning fee estimate.
        let mut symbol_evidence = symbol_commission("0.0010", "0.0015");
        let observation = symbol_evidence.observation.as_mut().unwrap();
        observation.terms_complete = false;
        observation
            .unresolved_obligations
            .push("UNKNOWN_ACTIVE_COMMISSION_FIELDS_UNRESOLVED".into());
        let statement = derive_identity(Some(symbol_evidence));
        assert_eq!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotFeeRateUnsupported
        );
        assert_eq!(statement.outcome, RiskCheckOutcome::Unavailable);
        assert_eq!(
            statement.binding_blocker.as_deref(),
            Some("SPOT_FEE_RATE_UNSUPPORTED")
        );
        assert!(statement.fee_rate_basis.is_none());
        // Historical account rates remain outside the owning statement.
        assert!(statement.declared_maker_rate.is_none());
    }

    #[test]
    fn fails_closed_with_evidence_unavailable_when_no_current_commission_is_bound() {
        let statement = derive_identity(None);
        assert_eq!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotFeeEvidenceUnavailable
        );
        assert_eq!(statement.outcome, RiskCheckOutcome::Unavailable);
        assert_eq!(
            statement.binding_blocker.as_deref(),
            Some("SPOT_FEE_EVIDENCE_UNAVAILABLE")
        );
        assert!(statement.fee_rate_basis.is_none());
        assert!(statement.declared_maker_rate.is_none());
    }

    #[test]
    fn a_supported_route_and_commission_without_a_current_model_remain_unavailable() {
        // Exact-symbol commission alone does not state the current branch. The future fee asset
        // remains unknown, and source-use support alone does not establish provider entitlement.
        let statement = derive_supported(Some(symbol_commission("0.0010", "0.0015")));
        assert_ne!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotFeeFxQualified
        );
        assert_ne!(statement.outcome, RiskCheckOutcome::Pass);
        assert!(statement.binding_blocker.is_some());
        assert_eq!(statement.proposal_hash.len(), 71);
        assert_eq!(statement.fee_evidence_version.len(), 71);
        assert_eq!(statement.route_evidence_version.len(), 71);
    }

    #[test]
    fn a_complete_current_branch_qualifies_review_while_the_actual_fee_asset_stays_unknown() {
        let mut commission = symbol_commission("0.0010", "0.0015");
        commission
            .observation
            .as_mut()
            .expect("symbol observation")
            .discount
            .enabled_for_account = false;
        let model = current_received_asset_model();
        let wallet = complete_wallet_funding_evidence();
        let statement = derive_supported_with_model(Some(commission), Some(&model), Some(&wallet));

        assert_eq!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotFeeFxQualified
        );
        assert_eq!(statement.outcome, RiskCheckOutcome::Pass);
        assert!(statement.binding_blocker.is_none());
        assert_eq!(statement.fee_currency, "UNKNOWN");
        assert_eq!(statement.fee_currency_origin, "UNKNOWN");
        assert!(statement.expected_fee.is_none());
        assert!(
            statement
                .reason
                .contains("future fee asset and amount remain unguaranteed")
        );

        let incomplete = derive_supported_with_model(
            Some({
                let mut commission = symbol_commission("0.0010", "0.0015");
                commission
                    .observation
                    .as_mut()
                    .expect("symbol observation")
                    .discount
                    .enabled_for_account = false;
                commission
            }),
            Some(&model),
            None,
        );
        assert_eq!(incomplete.outcome, RiskCheckOutcome::Unavailable);
        assert_eq!(
            incomplete.binding_blocker.as_deref(),
            Some("SPOT_FEE_EVIDENCE_UNAVAILABLE")
        );
        assert!(incomplete.reason.contains("wallet coverage is incomplete"));
    }

    #[test]
    fn current_model_with_unresolved_branch_inputs_does_not_qualify_review() {
        let mut commission = symbol_commission("0.0010", "0.0015");
        let observation = commission.observation.as_mut().expect("symbol observation");
        observation.discount.enabled_for_account = true;
        observation.discount.enabled_for_symbol = true;
        let model: crate::spot_fee_model::SpotCurrentFeeModel =
            serde_json::from_value(serde_json::json!({
                "method": "SPOT_THREE_FAMILY_RATIONAL_CEIL18_V1",
                "class": "CURRENT_ORDER_SCENARIO",
                "receivedAsset": "EUR",
                "grossReceived": "100",
                "fundingPlanPreview": {
                    "principalClaim": {"asset": "USD", "amount": "100"},
                    "feeDisposition": "CONDITIONAL_INPUTS_REQUIRED",
                    "fallbackFeeAsset": "EUR",
                    "fallbackFeeAmount": "0.1",
                    "fallbackNetReceipt": "99.9"
                },
                "fallback": {
                    "role": "TAKER", "standard": "0.1", "tax": "0", "special": "0",
                    "total": "0.1", "netReceived": "99.9"
                },
                "chargingCondition": "CONDITIONAL_BNB_INPUTS_REQUIRED",
                "limitations": ["VENUE_PER_FILL_ROUNDING_UNPROVEN"]
            }))
            .expect("conditional typed current fee model");
        let wallet = complete_wallet_funding_evidence();
        let statement = derive_supported_with_model(Some(commission), Some(&model), Some(&wallet));

        assert_eq!(
            statement.reason_code,
            SpotFeeFxReasonCode::SpotFeeCurrencyUnknown
        );
        assert_eq!(statement.outcome, RiskCheckOutcome::Unavailable);
        assert_eq!(
            statement.binding_blocker.as_deref(),
            Some("SPOT_FEE_CURRENCY_UNKNOWN")
        );
    }

    #[test]
    fn returns_none_when_the_capacity_does_not_describe_the_same_immutable_intent() {
        let mut capacity = capacity(Some(commission("0.0010", "0.0015")));
        // A differing proposal hash means the capacity and the intent are not the same object, so
        // there is no identity to bind and nothing to state.
        capacity.proposal_hash = hash('9');
        let requirements = requirements();
        let evidence = evidence();
        let bound = derive(
            &capacity,
            SpotFeeFxSources {
                commission: None,
                source_use_assessment: None,
                current_fee_model: None,
                wallet_funding_evidence: None,
            },
            &requirements,
            Some(&evidence),
            &proposal(),
            OrderSide::Buy,
            "USD",
        )
        .expect("derive never fails for a mismatched identity");
        assert!(bound.is_none());
    }

    #[test]
    fn reports_evidence_mismatch_when_the_fx_requirements_describe_another_intent() {
        // Identity mismatch binds before the independently policy-qualified supported rate.
        // Foreign workspace, Proposal or base requirements are never reused by assumption.
        let cases: [fn(&mut FxRequirements); 3] = [
            |requirements| requirements.workspace_id = "workspace-other".into(),
            |requirements| requirements.proposal_id = Some("proposal-other".into()),
            |requirements| requirements.base_currency = "EUR".into(),
        ];
        for mutate in cases {
            let capacity = capacity(Some(commission("0.0010", "0.0015")));
            let mut requirements = requirements();
            mutate(&mut requirements);
            let evidence = evidence();
            let source_use = source_use_assessment(true);
            let symbol_evidence = symbol_commission("0.0010", "0.0015");
            let statement = derive(
                &capacity,
                SpotFeeFxSources {
                    commission: Some(&symbol_evidence),
                    source_use_assessment: Some(&source_use),
                    current_fee_model: None,
                    wallet_funding_evidence: None,
                },
                &requirements,
                Some(&evidence),
                &proposal(),
                OrderSide::Buy,
                "USD",
            )
            .expect("derive never fails for a bound identity")
            .expect("a bound identity always yields a visible statement");
            assert_eq!(
                statement.reason_code,
                SpotFeeFxReasonCode::SpotFeeFxEvidenceMismatch
            );
            assert_eq!(statement.outcome, RiskCheckOutcome::Unavailable);
            assert_eq!(
                statement.binding_blocker.as_deref(),
                Some("SPOT_FEE_FX_EVIDENCE_MISMATCH")
            );
            // Preserve the independent policy-valuation route while identity mismatch binds first.
            assert_eq!(statement.routes.len(), 1);
            assert_eq!(statement.routes[0].state, SpotRequiredRouteState::Qualified);
        }
    }
}
