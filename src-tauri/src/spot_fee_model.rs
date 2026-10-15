//! Current three-family fee scenarios, distinct from funding and execution qualification.
use crate::protocol::{
    DataSourceStatus, ExecutionReservation, MarketDetail, OrderProposal, OrderQuantityType,
    OrderSide, OrderType, Result, TimeInForce, TimeStatus, TradeXError,
};
use crate::spot_capacity::{SpotCapacityInputs, SpotCapacityReadKind, SpotCapacityStatus};
use crate::spot_commission::{SpotCommissionInputs, SpotCommissionRates, SpotCommissionStatus};
use crate::spot_fee_fx::SpotFeeRateBasis;
use num_bigint::BigUint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeeModelMethod {
    SpotThreeFamilyRationalCeil18V1,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeeModelClass {
    CurrentOrderScenario,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChargingCondition {
    DiscountDisabledCurrentFallback,
    ConditionalBnbInputsRequired,
    ZeroBnbCurrentFallback,
    ZeroDiscountedFeeExpression,
    DiscountedBnbCurrentEstimate,
    InsufficientBnbCurrentFallback,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeeFundingDisposition {
    ReceivedAssetDeduction,
    BnbCurrentEstimate,
    NoCurrentFee,
    ConditionalInputsRequired,
}

/// One backend-derived native claim shown with the current scenario. It is not an approval and
/// never comes from renderer-supplied amounts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeNativeClaimPreview {
    #[schemars(length(min = 1, max = 16))]
    pub asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub amount: String,
}

/// Review-bound preview of the native principal and the currently modeled fee branch.
/// The eventual reservation is still created only by explicit approval and atomic Prepare.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotFundingPlanPreview {
    pub principal_claim: FeeNativeClaimPreview,
    pub fee_disposition: FeeFundingDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_fee_claim: Option<FeeNativeClaimPreview>,
    #[schemars(length(min = 1, max = 16))]
    pub fallback_fee_asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub fallback_fee_amount: String,
    #[schemars(length(min = 1, max = 64))]
    pub fallback_net_receipt: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeePriceSide {
    Bid,
    Ask,
}

/// Immutable market identity/receipts, without persisting raw depth or quote prices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeMarketReference {
    pub price_side: FeePriceSide,
    #[schemars(length(min = 1, max = 128))]
    pub market_snapshot_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub source: String,
    #[schemars(length(min = 64, max = 64))]
    pub material_hash: String,
    #[schemars(length(min = 1, max = 256))]
    pub source_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_generation: String,
    #[schemars(length(min = 1, max = 20))]
    pub book_update_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub provider_timestamp: String,
    #[schemars(length(min = 1, max = 64))]
    pub first_receipt: String,
    pub data_use_rights: DataSourceStatus,
}

/// One exact purpose-bound price leg used only by the current BNB fee estimate.
/// Price and size remain ephemeral; the saved reference contains identity and exact consumed
/// amount only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeConversionReference {
    #[schemars(length(min = 1, max = 32))]
    pub pair: String,
    #[schemars(length(min = 1, max = 16))]
    pub from_asset: String,
    #[schemars(length(min = 1, max = 16))]
    pub to_asset: String,
    pub price_side: FeePriceSide,
    #[schemars(length(min = 1, max = 16))]
    pub coverage_asset: String,
    pub coverage: FeeExactExpression,
    #[schemars(length(min = 1, max = 128))]
    pub market_snapshot_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub source: String,
    #[schemars(length(min = 64, max = 64))]
    pub material_hash: String,
    #[schemars(length(min = 1, max = 256))]
    pub source_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_generation: String,
    #[schemars(length(min = 1, max = 20))]
    pub book_update_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub provider_timestamp: String,
    #[schemars(length(min = 1, max = 64))]
    pub first_receipt: String,
    pub data_use_rights: DataSourceStatus,
}

pub(crate) struct MarketInputs<'a> {
    pub detail: Option<&'a MarketDetail>,
    pub fee_market: Option<&'a crate::spot_fee_market::SpotFeeMarketProjection>,
    pub clock: &'a TimeStatus,
    pub policy: Option<&'a crate::risk::RiskPolicy>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeFundingEvidence {
    #[schemars(length(min = 1, max = 16))]
    pub asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub free: String,
    #[schemars(length(min = 1, max = 64))]
    pub locked: String,
    #[schemars(length(min = 1, max = 64))]
    pub local_claims: String,
    #[schemars(length(min = 1, max = 64))]
    pub effective_free: String,
    #[schemars(length(min = 1, max = 128))]
    pub collection_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub capacity_binding_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub started_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub first_receipt: String,
    #[schemars(length(min = 64, max = 64))]
    pub original_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotWalletFundingStatus {
    NotObserved,
    Complete,
    Incomplete,
    Unavailable,
    Retired,
}

/// One original account wallet row with active same-account claims for this asset applied once.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotWalletBalanceEvidence {
    #[schemars(length(min = 1, max = 16))]
    pub asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub free: String,
    #[schemars(length(min = 1, max = 64))]
    pub locked: String,
    #[schemars(length(min = 1, max = 64))]
    pub local_claims: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub effective_free: Option<String>,
}

/// Proposal/account-bound exact BASE/QUOTE/BNB wallet coverage from one account-read receipt.
/// Missing holdings remain missing. This is descriptive local evidence, not execution authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotWalletFundingEvidence {
    pub status: SpotWalletFundingStatus,
    #[schemars(length(max = 3), inner(length(min = 1, max = 16)))]
    pub required_assets: Vec<String>,
    #[schemars(length(max = 3), inner(length(min = 1, max = 16)))]
    pub missing_assets: Vec<String>,
    #[schemars(length(max = 3))]
    pub balances: Vec<SpotWalletBalanceEvidence>,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 71, max = 71))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub account_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub source_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 64, max = 64))]
    pub rule_material_version: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub capacity_binding_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub collection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub account_read_started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub first_receipt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 64, max = 64))]
    pub original_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub reason: Option<String>,
}

pub(crate) fn reservation_native_claims(
    reservation: &ExecutionReservation,
) -> Result<Vec<crate::protocol::ExecutionReservationAssetClaim>> {
    use crate::protocol::{
        ExecutionReservationAssetClaim as Claim,
        ExecutionReservationClaimPurpose::{Fee, Principal},
    };

    if reservation.asset_claims.is_empty() {
        if reservation.unit.is_empty()
            || crate::provider_io::decimal_cmp(&reservation.amount, "0")?
                != std::cmp::Ordering::Greater
        {
            return Err(unavailable("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE"));
        }
        return Ok(vec![Claim {
            asset: reservation.unit.clone(),
            amount: reservation.amount.clone(),
            purpose: Principal,
        }]);
    }
    if reservation.asset_claims.len() > 16 {
        return Err(unavailable("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE"));
    }

    let mut claims: Vec<Claim> = Vec::with_capacity(reservation.asset_claims.len());
    let mut principal_count = 0;
    for claim in &reservation.asset_claims {
        let amount_order = crate::provider_io::decimal_cmp(&claim.amount, "0")?;
        if claim.asset.is_empty()
            || claim.asset.len() > 16
            || !claim
                .asset
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
            || amount_order == std::cmp::Ordering::Less
            || (amount_order == std::cmp::Ordering::Equal
                && claim.purpose == Principal
                && reservation.status != crate::protocol::ExecutionReservationStatus::Released)
        {
            return Err(unavailable("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE"));
        }
        if claims.iter().any(|existing| {
            existing.asset.eq_ignore_ascii_case(&claim.asset) && existing.purpose == claim.purpose
        }) {
            return Err(unavailable("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE"));
        }
        if claim.purpose == Principal {
            principal_count += 1;
            if claim.asset != reservation.unit || claim.amount != reservation.amount {
                return Err(unavailable("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE"));
            }
        } else if claim.purpose != Fee {
            return Err(unavailable("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE"));
        }
        claims.push(claim.clone());
    }
    if principal_count != 1 {
        return Err(unavailable("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE"));
    }
    Ok(claims)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeExactExpression {
    #[schemars(length(min = 1, max = 1234), regex(pattern = "^(0|[1-9][0-9]*)$"))]
    pub numerator: String,
    #[schemars(length(min = 1, max = 1234), regex(pattern = "^[1-9][0-9]*$"))]
    pub denominator: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeUnroundedFamilies {
    pub standard: FeeExactExpression,
    pub tax: FeeExactExpression,
    pub special: FeeExactExpression,
    pub total: FeeExactExpression,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeFamilyScenario {
    pub role: SpotFeeRateBasis,
    #[schemars(length(min = 1, max = 64))]
    pub standard: String,
    #[schemars(length(min = 1, max = 64))]
    pub tax: String,
    #[schemars(length(min = 1, max = 64))]
    pub special: String,
    #[schemars(length(min = 1, max = 64))]
    pub total: String,
    #[schemars(length(min = 1, max = 64))]
    pub net_received: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_rounding: Option<FeeUnroundedFamilies>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeBnbScenario {
    pub role: SpotFeeRateBasis,
    #[schemars(length(min = 1, max = 64))]
    pub standard: String,
    #[schemars(length(min = 1, max = 64))]
    pub tax: String,
    #[schemars(length(min = 1, max = 64))]
    pub special: String,
    #[schemars(length(min = 1, max = 64))]
    pub total: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_rounding: Option<FeeUnroundedFamilies>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCurrentFeeModel {
    pub method: FeeModelMethod,
    pub class: FeeModelClass,
    #[schemars(length(min = 1, max = 16))]
    pub received_asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub gross_received: String,
    /// Backend-derived, current-branch plan preview. It is captured in the same financial digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_plan_preview: Option<SpotFundingPlanPreview>,
    pub fallback: FeeFamilyScenario,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(length(max = 2))]
    pub fallback_candidates: Vec<FeeFamilyScenario>,
    pub charging_condition: ChargingCondition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bnb_funding_evidence: Option<FeeFundingEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discounted_fee_is_exactly_zero: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bnb_fee: Option<FeeBnbScenario>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(length(max = 2))]
    pub bnb_fee_candidates: Vec<FeeBnbScenario>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(length(max = 2))]
    pub bnb_conversion_references: Vec<FeeConversionReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub bnb_fee_failure: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_reference: Option<FeeMarketReference>,
    #[schemars(length(max = 2), inner(length(min = 1, max = 128)))]
    pub limitations: Vec<String>,
}

fn funding_plan_preview(
    proposal: &OrderProposal,
    capacity: &SpotCapacityInputs,
    charging_condition: &ChargingCondition,
    fallback: &FeeFamilyScenario,
    bnb_fee: Option<&FeeBnbScenario>,
) -> Result<SpotFundingPlanPreview> {
    let (principal_asset, principal_amount) = match proposal.fields.side {
        OrderSide::Sell => (
            capacity.base_asset.clone(),
            proposal.fields.quantity.value.clone(),
        ),
        OrderSide::Buy => match (proposal.fields.order_type, proposal.fields.quantity.r#type) {
            (OrderType::Market, OrderQuantityType::Quote) => (
                capacity.quote_asset.clone(),
                proposal.fields.quantity.value.clone(),
            ),
            (OrderType::Market, OrderQuantityType::Base) => (
                capacity.quote_asset.clone(),
                proposal
                    .fields
                    .maximum_spend
                    .clone()
                    .ok_or_else(|| unavailable("SPOT_FINANCIAL_PLAN_UNAVAILABLE"))?,
            ),
            (OrderType::Limit, OrderQuantityType::Base) => (
                capacity.quote_asset.clone(),
                crate::portfolio::decimal_mul(
                    &proposal.fields.quantity.value,
                    proposal
                        .fields
                        .limit_price
                        .as_deref()
                        .ok_or_else(|| unavailable("SPOT_FINANCIAL_PLAN_UNAVAILABLE"))?,
                )?,
            ),
            _ => return Err(unavailable("SPOT_FINANCIAL_PLAN_UNAVAILABLE")),
        },
    };
    let principal_amount =
        crate::provider_io::decimal(&serde_json::Value::String(principal_amount))?;
    if crate::provider_io::decimal_cmp(&principal_amount, "0")? != std::cmp::Ordering::Greater {
        return Err(unavailable("SPOT_FINANCIAL_PLAN_UNAVAILABLE"));
    }
    let (fee_disposition, current_fee_claim) = match charging_condition {
        ChargingCondition::DiscountDisabledCurrentFallback
        | ChargingCondition::ZeroBnbCurrentFallback
        | ChargingCondition::InsufficientBnbCurrentFallback => {
            (FeeFundingDisposition::ReceivedAssetDeduction, None)
        }
        ChargingCondition::DiscountedBnbCurrentEstimate => {
            let fee = bnb_fee.ok_or_else(|| unavailable("SPOT_FINANCIAL_PLAN_UNAVAILABLE"))?;
            if crate::provider_io::decimal_cmp(&fee.total, "0")? != std::cmp::Ordering::Greater {
                return Err(unavailable("SPOT_FINANCIAL_PLAN_UNAVAILABLE"));
            }
            (
                FeeFundingDisposition::BnbCurrentEstimate,
                Some(FeeNativeClaimPreview {
                    asset: "BNB".into(),
                    amount: fee.total.clone(),
                }),
            )
        }
        ChargingCondition::ZeroDiscountedFeeExpression => {
            (FeeFundingDisposition::NoCurrentFee, None)
        }
        ChargingCondition::ConditionalBnbInputsRequired => {
            (FeeFundingDisposition::ConditionalInputsRequired, None)
        }
    };
    Ok(SpotFundingPlanPreview {
        principal_claim: FeeNativeClaimPreview {
            asset: principal_asset,
            amount: principal_amount,
        },
        fee_disposition,
        current_fee_claim,
        fallback_fee_asset: match proposal.fields.side {
            OrderSide::Buy => capacity.base_asset.clone(),
            OrderSide::Sell => capacity.quote_asset.clone(),
        },
        fallback_fee_amount: fallback.total.clone(),
        fallback_net_receipt: fallback.net_received.clone(),
    })
}

fn unavailable(code: &str) -> TradeXError {
    TradeXError::new(code)
}

fn wallet_evidence_shell(
    proposal: &OrderProposal,
    capacity: &SpotCapacityInputs,
    required_assets: Vec<String>,
    status: SpotWalletFundingStatus,
    reason: Option<&str>,
) -> SpotWalletFundingEvidence {
    SpotWalletFundingEvidence {
        status,
        missing_assets: required_assets.clone(),
        required_assets,
        balances: Vec::new(),
        workspace_id: capacity.workspace_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_hash: proposal.proposal_hash.clone(),
        account_id: capacity.account_id.clone(),
        source_version: capacity.source_version.clone(),
        rule_material_version: capacity.rule_material_version.clone(),
        capacity_binding_version: capacity.binding_version.clone(),
        collection_id: None,
        account_read_started_at: None,
        first_receipt: None,
        original_digest: None,
        reason: reason.map(str::to_owned),
    }
}

/// Project the exact BASE/QUOTE/BNB rows from one current account read and subtract only active
/// same-account claims denominated in each row's asset. Invalid or retired inputs never retain prior balances.
pub(crate) fn wallet_funding_evidence(
    proposal: &OrderProposal,
    capacity: &SpotCapacityInputs,
    reservations: &[ExecutionReservation],
) -> SpotWalletFundingEvidence {
    let mut required_assets = Vec::with_capacity(3);
    for asset in [&capacity.base_asset, &capacity.quote_asset] {
        if !required_assets.iter().any(|required| required == asset) {
            required_assets.push(asset.clone());
        }
    }
    if !required_assets.iter().any(|required| required == "BNB") {
        required_assets.push("BNB".to_owned());
    }

    let identity_matches = capacity.workspace_id == proposal.workspace_id
        && capacity.proposal_id == proposal.proposal_id
        && capacity.proposal_hash == proposal.proposal_hash
        && Some(capacity.account_id.as_str()) == proposal.fields.account_id.as_deref()
        && capacity.instrument_id == proposal.fields.instrument_id;
    if !identity_matches {
        return wallet_evidence_shell(
            proposal,
            capacity,
            required_assets,
            SpotWalletFundingStatus::Unavailable,
            Some("SPOT_WALLET_INPUT_BINDING_MISMATCH"),
        );
    }

    let initial_status = match capacity.status {
        SpotCapacityStatus::NotObserved => Some((
            SpotWalletFundingStatus::NotObserved,
            "SPOT_WALLET_INPUT_NOT_OBSERVED",
        )),
        SpotCapacityStatus::Unavailable => Some((
            SpotWalletFundingStatus::Unavailable,
            "SPOT_WALLET_INPUT_UNAVAILABLE",
        )),
        SpotCapacityStatus::Stale => Some((
            SpotWalletFundingStatus::Retired,
            "SPOT_WALLET_INPUT_RETIRED",
        )),
        SpotCapacityStatus::Observed => None,
    };
    if let Some((status, reason)) = initial_status {
        return wallet_evidence_shell(proposal, capacity, required_assets, status, Some(reason));
    }
    let Some(observation) = capacity.observation.as_ref() else {
        return wallet_evidence_shell(
            proposal,
            capacity,
            required_assets,
            SpotWalletFundingStatus::Unavailable,
            Some("SPOT_WALLET_OBSERVATION_MISSING"),
        );
    };

    let mut evidence = wallet_evidence_shell(
        proposal,
        capacity,
        required_assets,
        SpotWalletFundingStatus::Incomplete,
        None,
    );
    evidence.collection_id = Some(observation.collection_id.clone());
    if observation.unresolved_obligations.iter().any(|reason| {
        matches!(
            reason.as_str(),
            "UNKNOWN_ACTIVE_ACCOUNT_FIELDS_UNRESOLVED" | "UNKNOWN_ACTIVE_BALANCE_FIELDS_UNRESOLVED"
        )
    }) {
        evidence.status = SpotWalletFundingStatus::Unavailable;
        evidence.reason = Some("SPOT_WALLET_INPUT_UNKNOWN_FIELDS".into());
        return evidence;
    }

    let account_reads = observation
        .reads
        .iter()
        .filter(|read| read.kind == SpotCapacityReadKind::Account)
        .collect::<Vec<_>>();
    if account_reads.len() != 1 {
        evidence.status = SpotWalletFundingStatus::Unavailable;
        evidence.reason = Some("SPOT_WALLET_ACCOUNT_RECEIPT_INCOMPLETE".into());
        return evidence;
    }
    let account_read = account_reads[0];
    evidence.account_read_started_at = Some(account_read.started_at.clone());
    evidence.first_receipt = Some(account_read.received_at.clone());
    evidence.original_digest = Some(account_read.digest.clone());

    let duplicate_asset = observation
        .wallet_balances
        .iter()
        .enumerate()
        .any(|(index, row)| {
            observation.wallet_balances[..index]
                .iter()
                .any(|prior| prior.asset.eq_ignore_ascii_case(&row.asset))
        });
    if duplicate_asset {
        evidence.status = SpotWalletFundingStatus::Unavailable;
        evidence.reason = Some("SPOT_WALLET_DUPLICATE_ASSET".into());
        return evidence;
    }

    let mut invalid_reason = None;
    let mut missing = Vec::new();
    for asset in &evidence.required_assets {
        let rows = observation
            .wallet_balances
            .iter()
            .filter(|row| row.asset == *asset)
            .collect::<Vec<_>>();
        if rows.len() != 1 {
            missing.push(asset.clone());
            continue;
        }
        let row = rows[0];
        let parsed_free = crate::provider_io::decimal(&serde_json::Value::String(row.free.clone()));
        let parsed_locked =
            crate::provider_io::decimal(&serde_json::Value::String(row.locked.clone()));
        let (Ok(free), Ok(_locked)) = (parsed_free, parsed_locked) else {
            invalid_reason.get_or_insert("SPOT_WALLET_BALANCE_UNAVAILABLE");
            missing.push(asset.clone());
            continue;
        };

        let mut claims = "0".to_owned();
        let mut claims_valid = true;
        for reservation in reservations.iter().filter(|reservation| {
            reservation.account_id == capacity.account_id
                && reservation.status == crate::protocol::ExecutionReservationStatus::Active
        }) {
            let native_claims = match reservation_native_claims(reservation) {
                Ok(claims) => claims,
                Err(_) => {
                    claims_valid = false;
                    break;
                }
            };
            for claim in native_claims
                .into_iter()
                .filter(|claim| claim.asset.eq_ignore_ascii_case(asset))
            {
                let amount =
                    match crate::provider_io::decimal(&serde_json::Value::String(claim.amount)) {
                        Ok(amount) => amount,
                        Err(_) => {
                            claims_valid = false;
                            break;
                        }
                    };
                claims = match crate::portfolio::decimal_add(&claims, &amount) {
                    Ok(total) if total.len() <= 64 => total,
                    _ => {
                        claims_valid = false;
                        break;
                    }
                };
            }
            if !claims_valid {
                break;
            }
        }
        if !claims_valid {
            invalid_reason.get_or_insert("SPOT_WALLET_LOCAL_CLAIM_UNAVAILABLE");
            missing.push(asset.clone());
            continue;
        }
        let effective_free = match crate::provider_io::decimal_cmp(&free, &claims) {
            Ok(std::cmp::Ordering::Less) => {
                invalid_reason.get_or_insert("SPOT_WALLET_LOCAL_CLAIM_EXCEEDS_FREE");
                None
            }
            Ok(_) if claims == "0" => Some(row.free.clone()),
            Ok(_) => match crate::provider_io::decimal_subtract(&row.free, &claims) {
                Ok(effective) if effective.len() <= 64 => Some(effective),
                _ => {
                    invalid_reason.get_or_insert("SPOT_WALLET_BALANCE_UNAVAILABLE");
                    None
                }
            },
            Err(_) => {
                invalid_reason.get_or_insert("SPOT_WALLET_BALANCE_UNAVAILABLE");
                None
            }
        };
        if effective_free.is_none() {
            missing.push(asset.clone());
        }
        evidence.balances.push(SpotWalletBalanceEvidence {
            asset: row.asset.clone(),
            free: row.free.clone(),
            locked: row.locked.clone(),
            local_claims: claims,
            effective_free,
        });
    }

    let has_unmapped_active_claim = reservations.iter().any(|reservation| {
        reservation.account_id == capacity.account_id
            && reservation.status == crate::protocol::ExecutionReservationStatus::Active
            && reservation_native_claims(reservation).map_or(true, |claims| {
                claims.iter().any(|claim| {
                    !evidence
                        .required_assets
                        .iter()
                        .any(|asset| asset.eq_ignore_ascii_case(&claim.asset))
                })
            })
    });
    evidence.missing_assets = missing;
    if let Some(reason) = invalid_reason {
        evidence.status = SpotWalletFundingStatus::Unavailable;
        evidence.reason = Some(reason.into());
    } else if has_unmapped_active_claim {
        evidence.status = SpotWalletFundingStatus::Incomplete;
        evidence.reason = Some("SPOT_WALLET_ACTIVE_CLAIM_OUTSIDE_REQUIRED_ASSETS".into());
    } else if evidence.missing_assets.is_empty() {
        evidence.status = SpotWalletFundingStatus::Complete;
    } else {
        evidence.status = SpotWalletFundingStatus::Incomplete;
        evidence.reason = Some("SPOT_WALLET_REQUIRED_ASSET_MISSING".into());
    }
    evidence
}

/// Exact nonnegative rational expressions; the only truncation is the final model output.
/// Bounded wire inputs and a 4096-bit intermediate limit bound allocation and arithmetic work.
struct Rational {
    numerator: BigUint,
    denominator: BigUint,
}
impl Rational {
    fn checked(numerator: BigUint, denominator: BigUint) -> Result<Self> {
        if denominator == BigUint::from(0u8) || numerator.bits() > 4096 || denominator.bits() > 4096
        {
            return Err(unavailable("SPOT_FEE_MODEL_ARITHMETIC_UNAVAILABLE"));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
    fn decimal(raw: &str) -> Result<Self> {
        if raw.is_empty() || raw.len() > 64 || raw.starts_with('-') {
            return Err(unavailable("SPOT_FEE_MODEL_ARITHMETIC_UNAVAILABLE"));
        }
        let normalized = crate::provider_io::decimal(&serde_json::Value::String(raw.into()))?;
        let (whole, fraction) = normalized.split_once('.').unwrap_or((&normalized, ""));
        let digits = format!("{whole}{fraction}");
        let numerator = BigUint::parse_bytes(digits.as_bytes(), 10)
            .ok_or_else(|| unavailable("SPOT_FEE_MODEL_ARITHMETIC_UNAVAILABLE"))?;
        Self::checked(numerator, BigUint::from(10u8).pow(fraction.len() as u32))
    }
    fn add(&self, other: &Self) -> Result<Self> {
        Self::checked(
            &self.numerator * &other.denominator + &other.numerator * &self.denominator,
            &self.denominator * &other.denominator,
        )
    }
    fn multiply(&self, other: &Self) -> Result<Self> {
        Self::checked(
            &self.numerator * &other.numerator,
            &self.denominator * &other.denominator,
        )
    }
    fn divide(&self, other: &Self) -> Result<Self> {
        Self::checked(
            &self.numerator * &other.denominator,
            &self.denominator * &other.numerator,
        )
    }
    fn subtract(&self, other: &Self) -> Result<Self> {
        let left = &self.numerator * &other.denominator;
        let right = &other.numerator * &self.denominator;
        if left < right {
            return Err(unavailable("SPOT_FEE_EXCEEDS_MODELED_RECEIPT"));
        }
        Self::checked(left - right, &self.denominator * &other.denominator)
    }
    fn compare(&self, other: &Self) -> Result<std::cmp::Ordering> {
        let left = &self.numerator * &other.denominator;
        let right = &other.numerator * &self.denominator;
        if left.bits() > 4096 || right.bits() > 4096 {
            return Err(unavailable("SPOT_FEE_MODEL_ARITHMETIC_UNAVAILABLE"));
        }
        Ok(left.cmp(&right))
    }
    fn output(&self, outward: bool) -> Result<String> {
        let scaled = &self.numerator * BigUint::from(10u8).pow(18);
        let quotient = &scaled / &self.denominator;
        let remainder = &scaled % &self.denominator;
        let rounded = if outward && remainder != BigUint::from(0u8) {
            quotient + BigUint::from(1u8)
        } else {
            quotient
        };
        let digits = rounded.to_str_radix(10);
        let padded = format!("{:0>19}", digits);
        let split = padded.len() - 18;
        let (whole, fraction) = padded.split_at(split);
        let fraction = fraction.trim_end_matches('0');
        let result = if fraction.is_empty() {
            whole.into()
        } else {
            format!("{whole}.{fraction}")
        };
        if result.len() > 64 {
            return Err(unavailable("SPOT_FEE_MODEL_ARITHMETIC_UNAVAILABLE"));
        }
        Ok(result)
    }
    fn audit(&self) -> FeeExactExpression {
        // Canonical reduced fractions express mathematical values rather than
        // the implementation's multiplication order. Zero is always0/1.
        let mut left = self.numerator.clone();
        let mut right = self.denominator.clone();
        while right != BigUint::from(0u8) {
            let remainder = &left % &right;
            left = right;
            right = remainder;
        }
        FeeExactExpression {
            numerator: (&self.numerator / &left).to_str_radix(10),
            denominator: (&self.denominator / &left).to_str_radix(10),
        }
    }
}

fn family_expression(
    gross: &Rational,
    rates: &SpotCommissionRates,
    role: SpotFeeRateBasis,
    side: OrderSide,
) -> Result<Rational> {
    let role_rate = match role {
        SpotFeeRateBasis::Maker => &rates.maker,
        SpotFeeRateBasis::Taker => &rates.taker,
    };
    let side_rate = match side {
        OrderSide::Buy => &rates.buyer,
        OrderSide::Sell => &rates.seller,
    };
    gross.multiply(&Rational::decimal(role_rate)?.add(&Rational::decimal(side_rate)?)?)
}

fn scenario(
    gross: &Rational,
    inputs: &crate::spot_commission::SpotCommissionObservation,
    role: SpotFeeRateBasis,
    side: OrderSide,
) -> Result<FeeFamilyScenario> {
    let standard_expression = family_expression(gross, &inputs.standard_commission, role, side)?;
    let tax_expression = family_expression(gross, &inputs.tax_commission, role, side)?;
    let special_expression = family_expression(gross, &inputs.special_commission, role, side)?;
    let pre_rounding = FeeUnroundedFamilies {
        standard: standard_expression.audit(),
        tax: tax_expression.audit(),
        special: special_expression.audit(),
        total: standard_expression
            .add(&tax_expression)?
            .add(&special_expression)?
            .audit(),
    };
    let standard = standard_expression.output(true)?;
    let tax = tax_expression.output(true)?;
    let special = special_expression.output(true)?;
    let total = Rational::decimal(&standard)?
        .add(&Rational::decimal(&tax)?)?
        .add(&Rational::decimal(&special)?)?;
    let net_received = gross.subtract(&total)?.output(false)?;
    Ok(FeeFamilyScenario {
        role,
        standard,
        tax,
        special,
        total: total.output(true)?,
        net_received,
        pre_rounding: Some(pre_rounding),
    })
}

fn discounted_bnb_scenario(
    gross: &Rational,
    observation: &crate::spot_commission::SpotCommissionObservation,
    role: SpotFeeRateBasis,
    side: OrderSide,
    conversion: &Rational,
) -> Result<(FeeBnbScenario, Rational)> {
    let discount = Rational::decimal(&observation.discount.discount)?;
    let standard_received = family_expression(gross, &observation.standard_commission, role, side)?
        .multiply(&discount)?;
    let tax_received = family_expression(gross, &observation.tax_commission, role, side)?;
    let special_received = family_expression(gross, &observation.special_commission, role, side)?;
    let received_total = standard_received
        .add(&tax_received)?
        .add(&special_received)?;
    let standard = standard_received.multiply(conversion)?;
    let tax = tax_received.multiply(conversion)?;
    let special = special_received.multiply(conversion)?;
    let pre_rounding = FeeUnroundedFamilies {
        standard: standard.audit(),
        tax: tax.audit(),
        special: special.audit(),
        total: standard.add(&tax)?.add(&special)?.audit(),
    };
    let standard_amount = standard.output(true)?;
    let tax_amount = tax.output(true)?;
    let special_amount = special.output(true)?;
    let rounded_total = Rational::decimal(&standard_amount)?
        .add(&Rational::decimal(&tax_amount)?)?
        .add(&Rational::decimal(&special_amount)?)?;
    Ok((
        FeeBnbScenario {
            role,
            standard: standard_amount,
            tax: tax_amount,
            special: special_amount,
            total: rounded_total.output(true)?,
            pre_rounding: Some(pre_rounding),
        },
        received_total,
    ))
}

fn conversion_reference(
    snapshot: &crate::protocol::MarketSnapshot,
    evidence: &crate::protocol::BinanceSpotQuoteEvidence,
    from_asset: &str,
    to_asset: &str,
    price_side: FeePriceSide,
    coverage_asset: &str,
    coverage: &Rational,
) -> FeeConversionReference {
    FeeConversionReference {
        pair: evidence.provider_symbol.clone(),
        from_asset: from_asset.into(),
        to_asset: to_asset.into(),
        price_side,
        coverage_asset: coverage_asset.into(),
        coverage: coverage.audit(),
        market_snapshot_id: snapshot.provenance.market_snapshot_id.clone(),
        source: snapshot.provenance.source.clone(),
        material_hash: evidence.material_hash.clone(),
        source_version: evidence.source_version.clone(),
        connection_generation: evidence.connection_generation.clone(),
        book_update_id: evidence.book_update_id.clone(),
        provider_timestamp: snapshot.provenance.provider_timestamp.clone(),
        first_receipt: snapshot.provenance.received_timestamp.clone(),
        data_use_rights: evidence.data_use_rights.clone(),
    }
}

fn discounted_bnb_estimate(
    gross: &Rational,
    spot_inputs: &SpotCommissionInputs,
    observation: &crate::spot_commission::SpotCommissionObservation,
    proposal: &OrderProposal,
    roles: &[SpotFeeRateBasis],
    market: &MarketInputs<'_>,
) -> Result<(
    FeeBnbScenario,
    Vec<FeeBnbScenario>,
    Vec<FeeConversionReference>,
)> {
    use crate::protocol::{HotQuoteStatus, MarketFreshness, TimeConfidence};
    use crate::risk::RiskCheckOutcome;
    use std::cmp::Ordering;

    let fee_market = market
        .fee_market
        .filter(|projection| {
            projection.workspace_id == proposal.workspace_id
                && projection.source_id == crate::binance_market::SOURCE_ID
                && projection.status == HotQuoteStatus::Streaming
                && projection.metadata.as_ref().is_some_and(|metadata| {
                    metadata.provider_symbol == "BNBUSDT"
                        && metadata.base_asset == "BNB"
                        && metadata.quote_asset == "USDT"
                        && metadata.market_status == "TRADING"
                        && !metadata.received_at.is_empty()
                })
        })
        .ok_or_else(|| unavailable("SPOT_BNB_MARKET_INPUT_UNAVAILABLE"))?;
    let fee_snapshot = fee_market
        .snapshot
        .as_ref()
        .filter(|snapshot| {
            snapshot.instrument_id == crate::spot_fee_market::INTERNAL_ID
                && snapshot.provenance.source == crate::binance_market::SOURCE_ID
                && snapshot.provenance.freshness == MarketFreshness::Healthy
        })
        .ok_or_else(|| unavailable("SPOT_BNB_MARKET_INPUT_UNAVAILABLE"))?;
    let fee_evidence = fee_snapshot
        .provenance
        .binance
        .as_ref()
        .filter(|evidence| {
            evidence.provider_symbol == "BNBUSDT"
                && evidence.base_asset == "BNB"
                && evidence.quote_asset == "USDT"
                && evidence.source_version == fee_market.source_version
                && evidence.connection_generation == fee_market.connection_generation
        })
        .ok_or_else(|| unavailable("SPOT_BNB_MARKET_INPUT_UNAVAILABLE"))?;
    let bnb_bid = Rational::decimal(
        fee_snapshot
            .bid
            .as_deref()
            .ok_or_else(|| unavailable("SPOT_BNB_MARKET_INPUT_UNAVAILABLE"))?,
    )?;
    let bnb_ask = Rational::decimal(
        fee_snapshot
            .ask
            .as_deref()
            .ok_or_else(|| unavailable("SPOT_BNB_MARKET_INPUT_UNAVAILABLE"))?,
    )?;
    if market.clock.confidence != TimeConfidence::Trusted
        || bnb_bid.numerator == BigUint::from(0u8)
        || bnb_bid.compare(&bnb_ask)? == Ordering::Greater
    {
        return Err(unavailable("SPOT_BNB_MARKET_INPUT_UNAVAILABLE"));
    }
    let bnb_bid_size = Rational::decimal(
        fee_snapshot
            .bid_size
            .as_deref()
            .ok_or_else(|| unavailable("SPOT_BNB_MARKET_INPUT_UNAVAILABLE"))?,
    )?;
    if bnb_bid_size.numerator == BigUint::from(0u8) {
        return Err(unavailable("SPOT_BNB_MARKET_COVERAGE_INSUFFICIENT"));
    }

    let (conversion, instrument_leg) = if observation.received_asset == spot_inputs.base_asset {
        let detail = market
            .detail
            .filter(|detail| crate::risk::execution_quote_matches(proposal, detail))
            .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?;
        let policy = market
            .policy
            .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_POLICY_UNAVAILABLE"))?;
        let maximum_age = policy.stale_quote_threshold_seconds.min(30);
        let (freshness, _) = crate::risk::market_freshness(Some(detail), market.clock, maximum_age);
        if freshness != RiskCheckOutcome::Pass {
            return Err(unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"));
        }
        let snapshot = detail
            .snapshot
            .as_ref()
            .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?;
        let evidence = snapshot
            .provenance
            .binance
            .as_ref()
            .filter(|evidence| {
                evidence.base_asset == observation.received_asset
                    && evidence.base_asset == spot_inputs.base_asset
                    && evidence.quote_asset == "USDT"
                    && evidence.provider_symbol.ends_with("USDT")
            })
            .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?;
        let ask = Rational::decimal(
            snapshot
                .ask
                .as_deref()
                .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?,
        )?;
        if ask.numerator == BigUint::from(0u8) {
            return Err(unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"));
        }
        let ask_size = Rational::decimal(
            snapshot
                .ask_size
                .as_deref()
                .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?,
        )?;
        let leg = (snapshot, evidence, ask_size);
        (ask.divide(&bnb_bid)?, Some(leg))
    } else if observation.received_asset == spot_inputs.quote_asset
        && observation.received_asset == "USDT"
    {
        (
            Rational::checked(BigUint::from(1u8), BigUint::from(1u8))?.divide(&bnb_bid)?,
            None,
        )
    } else {
        return Err(unavailable("SPOT_BNB_CONVERSION_ROUTE_UNSUPPORTED"));
    };

    let mut candidates = Vec::with_capacity(roles.len());
    for &role in roles {
        candidates.push(discounted_bnb_scenario(
            gross,
            observation,
            role,
            proposal.fields.side,
            &conversion,
        )?);
    }
    let mut selected_index = 0usize;
    for index in 1..candidates.len() {
        match crate::provider_io::decimal_cmp(
            &candidates[index].0.total,
            &candidates[selected_index].0.total,
        )? {
            Ordering::Greater => selected_index = index,
            Ordering::Equal
                if candidates[index].0.role == SpotFeeRateBasis::Taker
                    && candidates[selected_index].0.role != SpotFeeRateBasis::Taker =>
            {
                selected_index = index;
            }
            _ => {}
        }
    }
    let (selected, _) = &candidates[selected_index];
    let bnb_total = Rational::decimal(&selected.total)?;
    if bnb_bid_size.compare(&bnb_total)? == Ordering::Less {
        return Err(unavailable("SPOT_BNB_MARKET_COVERAGE_INSUFFICIENT"));
    }
    let mut references = Vec::with_capacity(2);
    if let Some((snapshot, evidence, ask_size)) = instrument_leg {
        let mut maximum_received_amount = &candidates[0].1;
        for (_, amount) in candidates.iter().skip(1) {
            if amount.compare(maximum_received_amount)? == Ordering::Greater {
                maximum_received_amount = amount;
            }
        }
        if ask_size.compare(maximum_received_amount)? == Ordering::Less {
            return Err(unavailable("SPOT_FEE_MODEL_MARKET_COVERAGE_INSUFFICIENT"));
        }
        references.push(conversion_reference(
            snapshot,
            evidence,
            &observation.received_asset,
            "USDT",
            FeePriceSide::Ask,
            &observation.received_asset,
            maximum_received_amount,
        ));
    }
    references.push(conversion_reference(
        fee_snapshot,
        fee_evidence,
        "USDT",
        "BNB",
        FeePriceSide::Bid,
        "BNB",
        &bnb_total,
    ));
    let candidate_models = candidates
        .into_iter()
        .map(|(scenario, _)| scenario)
        .collect::<Vec<_>>();
    Ok((
        candidate_models[selected_index].clone(),
        candidate_models,
        references,
    ))
}

fn discounted_expression_positive(
    gross: &Rational,
    observation: &crate::spot_commission::SpotCommissionObservation,
    side: OrderSide,
    roles: &[SpotFeeRateBasis],
) -> Result<bool> {
    let discount = Rational::decimal(&observation.discount.discount)?;
    for &role in roles {
        let total = family_expression(gross, &observation.standard_commission, role, side)?
            .multiply(&discount)?
            .add(&family_expression(
                gross,
                &observation.tax_commission,
                role,
                side,
            )?)?
            .add(&family_expression(
                gross,
                &observation.special_commission,
                role,
                side,
            )?)?;
        if total.numerator != BigUint::from(0u8) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn market_base_scenario(
    proposal: &OrderProposal,
    quantity: Rational,
    inputs: &MarketInputs<'_>,
) -> Result<(Rational, FeeMarketReference)> {
    let policy = inputs
        .policy
        .filter(|policy| policy.market_orders_enabled)
        .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_POLICY_UNAVAILABLE"))?;
    let limit = policy
        .max_market_order_slippage_percent
        .as_deref()
        .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_POLICY_UNAVAILABLE"))?;
    let age = policy.stale_quote_threshold_seconds.min(30);
    let (numerator, denominator) =
        crate::risk::market_order_slippage(proposal, inputs.detail, inputs.clock, age)
            .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?;
    let maximum = crate::portfolio::decimal_mul(limit, &denominator)?;
    if crate::provider_io::decimal_cmp(&numerator, &maximum)? == std::cmp::Ordering::Greater {
        return Err(unavailable("SPOT_FEE_MODEL_SLIPPAGE_EXCEEDED"));
    }
    let snapshot = inputs
        .detail
        .and_then(|detail| detail.snapshot.as_ref())
        .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?;
    let evidence = snapshot
        .provenance
        .binance
        .as_ref()
        .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?;
    let (gross, side) = match proposal.fields.side {
        OrderSide::Buy => (
            if proposal.fields.quantity.r#type == OrderQuantityType::Quote {
                quantity.divide(&Rational::decimal(snapshot.ask.as_deref().ok_or_else(
                    || unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"),
                )?)?)?
            } else {
                quantity
            },
            FeePriceSide::Ask,
        ),
        OrderSide::Sell => (
            quantity.multiply(&Rational::decimal(
                snapshot
                    .bid
                    .as_deref()
                    .ok_or_else(|| unavailable("SPOT_FEE_MODEL_MARKET_QUOTE_UNAVAILABLE"))?,
            )?)?,
            FeePriceSide::Bid,
        ),
    };
    Ok((
        gross,
        FeeMarketReference {
            price_side: side,
            market_snapshot_id: snapshot.provenance.market_snapshot_id.clone(),
            source: snapshot.provenance.source.clone(),
            material_hash: evidence.material_hash.clone(),
            source_version: evidence.source_version.clone(),
            connection_generation: evidence.connection_generation.clone(),
            book_update_id: evidence.book_update_id.clone(),
            provider_timestamp: snapshot.provenance.provider_timestamp.clone(),
            first_receipt: snapshot.provenance.received_timestamp.clone(),
            data_use_rights: evidence.data_use_rights.clone(),
        },
    ))
}

fn current_bnb_funds(
    inputs: &SpotCommissionInputs,
    proposal: &OrderProposal,
    capacity: &SpotCapacityInputs,
    reservations: &[ExecutionReservation],
    wallet: &SpotWalletFundingEvidence,
) -> Result<Option<FeeFundingEvidence>> {
    if capacity.status != SpotCapacityStatus::Observed
        || !inputs.matches_capacity_binding(proposal, capacity, proposal.fields.side)
        || matches!(
            wallet.status,
            SpotWalletFundingStatus::NotObserved
                | SpotWalletFundingStatus::Unavailable
                | SpotWalletFundingStatus::Retired
        )
    {
        return Ok(None);
    }
    let balances = wallet
        .balances
        .iter()
        .filter(|row| row.asset == "BNB")
        .collect::<Vec<_>>();
    if balances.len() != 1 {
        return Ok(None);
    }
    let balance = balances[0];
    // A legacy PLACE reservation records only its principal unit. Until the fee and fill
    // lifecycle has settled, a reservation in another asset cannot prove that the attempt will
    // not also consume BNB. Do not treat the observed BNB free balance as available in that case.
    if reservations.iter().any(|reservation| {
        reservation.account_id == inputs.account_id
            && reservation.status == crate::protocol::ExecutionReservationStatus::Active
            && reservation_native_claims(reservation).map_or(true, |claims| {
                claims
                    .iter()
                    .any(|claim| !claim.asset.eq_ignore_ascii_case("BNB"))
            })
    }) {
        return Ok(None);
    }
    let Some(effective) = balance.effective_free.as_ref() else {
        return Ok(None);
    };
    let (Some(collection_id), Some(started_at), Some(first_receipt), Some(original_digest)) = (
        wallet.collection_id.as_ref(),
        wallet.account_read_started_at.as_ref(),
        wallet.first_receipt.as_ref(),
        wallet.original_digest.as_ref(),
    ) else {
        return Ok(None);
    };
    Ok(Some(FeeFundingEvidence {
        asset: balance.asset.clone(),
        free: balance.free.clone(),
        locked: balance.locked.clone(),
        local_claims: balance.local_claims.clone(),
        effective_free: effective.clone(),
        collection_id: collection_id.clone(),
        capacity_binding_version: capacity.binding_version.clone(),
        started_at: started_at.clone(),
        first_receipt: first_receipt.clone(),
        original_digest: original_digest.clone(),
    }))
}

pub(crate) fn current_fallback(
    inputs: &SpotCommissionInputs,
    proposal: &OrderProposal,
    capacity: &SpotCapacityInputs,
    reservations: &[ExecutionReservation],
    wallet: &SpotWalletFundingEvidence,
    market: MarketInputs<'_>,
) -> Result<SpotCurrentFeeModel> {
    if !inputs.matches_capacity_binding(proposal, capacity, proposal.fields.side)
        || inputs.status != SpotCommissionStatus::Observed
    {
        return Err(unavailable("SPOT_FEE_MODEL_EVIDENCE_UNAVAILABLE"));
    }
    let observation = inputs
        .observation
        .as_ref()
        .filter(|observation| {
            observation.terms_complete
                && observation.extension_keys.is_empty()
                && observation.matches_capacity_symbol(capacity, proposal.fields.side)
                && !observation.unresolved_obligations.iter().any(|obligation| {
                    !matches!(
                        obligation.as_str(),
                        "NON_ATOMIC_PROVIDER_SNAPSHOT"
                            | "FEE_AMOUNT_AND_CHARGING_ASSET_NOT_EXECUTION_QUALIFIED"
                    )
                })
        })
        .ok_or_else(|| unavailable("SPOT_FEE_MODEL_EVIDENCE_UNAVAILABLE"))?;
    if proposal.fields.quantity.r#type != OrderQuantityType::Base
        && !(proposal.fields.quantity.r#type == OrderQuantityType::Quote
            && proposal.fields.side == OrderSide::Buy
            && proposal.fields.order_type == OrderType::Market
            && proposal.fields.time_in_force == TimeInForce::Day)
    {
        return Err(unavailable("SPOT_FEE_MODEL_SCENARIO_UNAVAILABLE"));
    }
    let quantity = Rational::decimal(&proposal.fields.quantity.value)?;
    let (gross, market_reference) =
        match proposal.fields.order_type {
            OrderType::Limit
                if matches!(
                    proposal.fields.time_in_force,
                    TimeInForce::Gtc | TimeInForce::Ioc | TimeInForce::Fok
                ) =>
            {
                (
                    match proposal.fields.side {
                        OrderSide::Buy => quantity,
                        OrderSide::Sell => quantity.multiply(&Rational::decimal(
                            proposal.fields.limit_price.as_deref().ok_or_else(|| {
                                unavailable("SPOT_FEE_MODEL_SCENARIO_UNAVAILABLE")
                            })?,
                        )?)?,
                    },
                    None,
                )
            }
            OrderType::Market if proposal.fields.time_in_force == TimeInForce::Day => {
                let (gross, reference) = market_base_scenario(proposal, quantity, &market)?;
                (gross, Some(reference))
            }
            _ => return Err(unavailable("SPOT_FEE_MODEL_SCENARIO_UNAVAILABLE")),
        };
    let taker = scenario(
        &gross,
        observation,
        SpotFeeRateBasis::Taker,
        proposal.fields.side,
    )?;
    let roles: &[SpotFeeRateBasis] = if proposal.fields.order_type == OrderType::Limit {
        &[SpotFeeRateBasis::Maker, SpotFeeRateBasis::Taker]
    } else {
        &[SpotFeeRateBasis::Taker]
    };
    let mut fallback_candidates = Vec::with_capacity(roles.len());
    let fallback = if proposal.fields.order_type == OrderType::Limit {
        let maker = scenario(
            &gross,
            observation,
            SpotFeeRateBasis::Maker,
            proposal.fields.side,
        )?;
        fallback_candidates.push(maker.clone());
        fallback_candidates.push(taker.clone());
        if crate::provider_io::decimal_cmp(&maker.total, &taker.total)?
            == std::cmp::Ordering::Greater
        {
            maker
        } else {
            taker
        }
    } else {
        fallback_candidates.push(taker.clone());
        taker
    };
    let discount_active =
        observation.discount.enabled_for_account && observation.discount.enabled_for_symbol;
    let discounted_fee_is_exactly_zero = if discount_active {
        Some(!discounted_expression_positive(
            &gross,
            observation,
            proposal.fields.side,
            roles,
        )?)
    } else {
        None
    };
    let bnb_funding_evidence = if discounted_fee_is_exactly_zero == Some(false) {
        current_bnb_funds(inputs, proposal, capacity, reservations, wallet)?
    } else {
        None
    };
    let bnb_funds_vs_zero = bnb_funding_evidence
        .as_ref()
        .map(|funds| crate::provider_io::decimal_cmp(&funds.effective_free, "0"))
        .transpose()?;
    let bnb_funds_are_positive = bnb_funds_vs_zero == Some(std::cmp::Ordering::Greater);
    let bnb_funds_are_zero = bnb_funds_vs_zero == Some(std::cmp::Ordering::Equal);
    let mut bnb_fee = None;
    let mut bnb_fee_candidates = Vec::new();
    let mut bnb_conversion_references = Vec::new();
    let mut bnb_fee_failure = None;
    if discount_active && discounted_fee_is_exactly_zero == Some(false) && bnb_funds_are_positive {
        match discounted_bnb_estimate(&gross, inputs, observation, proposal, roles, &market) {
            Ok((selected, candidates, references)) => {
                bnb_fee = Some(selected);
                bnb_fee_candidates = candidates;
                bnb_conversion_references = references;
            }
            Err(error) => bnb_fee_failure = Some(error.code),
        }
    }
    let charging_condition = if !discount_active {
        ChargingCondition::DiscountDisabledCurrentFallback
    } else if discounted_fee_is_exactly_zero == Some(true) {
        // Algebraic zero needs no invented paid asset, rate or wallet amount.
        ChargingCondition::ZeroDiscountedFeeExpression
    } else if bnb_funds_are_zero {
        ChargingCondition::ZeroBnbCurrentFallback
    } else if let (Some(funds), Some(fee)) = (&bnb_funding_evidence, &bnb_fee) {
        if crate::provider_io::decimal_cmp(&funds.effective_free, &fee.total)?
            != std::cmp::Ordering::Less
        {
            ChargingCondition::DiscountedBnbCurrentEstimate
        } else {
            ChargingCondition::InsufficientBnbCurrentFallback
        }
    } else {
        ChargingCondition::ConditionalBnbInputsRequired
    };
    let mut limitations = vec!["VENUE_PER_FILL_ROUNDING_UNPROVEN".into()];
    if bnb_fee.is_some() {
        limitations.push("BNB_INTERNAL_AVERAGE_EQUIVALENCE_UNPROVEN".into());
    }
    let funding_plan_preview = funding_plan_preview(
        proposal,
        capacity,
        &charging_condition,
        &fallback,
        bnb_fee.as_ref(),
    )?;
    Ok(SpotCurrentFeeModel {
        method: FeeModelMethod::SpotThreeFamilyRationalCeil18V1,
        class: FeeModelClass::CurrentOrderScenario,
        received_asset: observation.received_asset.clone(),
        gross_received: gross.output(false)?,
        funding_plan_preview: Some(funding_plan_preview),
        fallback,
        fallback_candidates,
        charging_condition,
        bnb_funding_evidence,
        discounted_fee_is_exactly_zero,
        bnb_fee,
        bnb_fee_candidates,
        bnb_conversion_references,
        bnb_fee_failure,
        market_reference,
        limitations,
    })
}
