//! Owning Spot Live PLACE qualification derived from the delivered read-only inputs.
//!
//! This module only decides whether the venue's own declared evidence currently admits a
//! new order. It never grants Arm, consent, dispatch or any other execution authority, and
//! it never adjusts the provider's numbers.
//!
//! It deliberately consumes the delivered capacity and interval inputs *as they are*. Those
//! slices ship honest, permanent limitations — the capacity collection is built from separate
//! HTTP reads and is therefore labelled non-atomic, and the interval counters carry no provider
//! timestamp so their local clock association stays explicitly uncertain. Those labels are
//! carried forward as declared limitations instead of being promoted into a second, hidden
//! admission veto; what fails the admission question is *unknown coverage*, never the honest
//! labelling of a known one. This is the "later synced contract" the capacity slice reserved to
//! distinguish an ordinary intent's actually-applicable capacity from authenticated execution
//! qualification: this module answers the former and never the latter.
use crate::protocol::{OrderSide, Result};
use crate::risk::RiskCheckOutcome;
use crate::spot_capacity::SpotCapacityInputs;
use crate::spot_order_intervals::SpotOrderIntervalInputs;
use crate::provider_io;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

/// Standing limitations the delivered slices always carry. They describe how the evidence was
/// obtained, not what it failed to cover, so they are reported and never treated as gaps.
const CARRIED_CAPACITY_LIMITATIONS: [&str; 2] = [
    "NON_ATOMIC_PROVIDER_SNAPSHOT",
    "DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED",
];
const CARRIED_INTERVAL_LIMITATIONS: [&str; 3] = [
    "COUNTER_SNAPSHOT_TIME_UNAVAILABLE",
    "DERIVED_WINDOW_ASSOCIATION_UNCERTAIN",
    "INTERVAL_INPUTS_NOT_EXECUTION_QUALIFIED",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotOwningReasonCode {
    /// Every declared capacity and interval fact is current, complete and consistent.
    SpotOwningQualified,
    /// The two evidence sets do not describe the same immutable intent.
    SpotOwningEvidenceMismatch,
    /// No current declared capacity observation qualifies this intent.
    SpotCapacityEvidenceUnavailable,
    /// The declared inventory is present but its coverage is not complete.
    SpotCapacityCoverageIncomplete,
    /// No current declared interval observation qualifies this intent.
    SpotIntervalEvidenceUnavailable,
    /// A declared order-rate bucket has no remaining slot.
    SpotIntervalQuotaExhausted,
    /// The interval observation was retired because its window association lapsed.
    SpotIntervalWindowUncertain,
}

/// One exact statement of whether, and how much of, the reviewed intent the venue would
/// currently admit. It is explicitly execution-unqualified: the venue still recalculates its
/// order-rate windows, and fees, required FX, authenticated preflight and private-stream
/// readiness remain separate owning work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotOwningQualification {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
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
    /// The exact declared unit the capacity numbers are read in: QUOTE for a buy, BASE for a sell.
    #[schemars(length(min = 1, max = 16))]
    pub capacity_unit: String,
    pub outcome: RiskCheckOutcome,
    pub reason_code: SpotOwningReasonCode,
    #[schemars(length(min = 1, max = 256))]
    pub reason: String,
    #[schemars(length(min = 1, max = 128))]
    pub binding_blocker: Option<String>,
    /// The declared fact each number was read from, so a reviewer can name the origin.
    #[schemars(length(min = 1, max = 128))]
    pub inventory_source: String,
    #[schemars(length(min = 1, max = 128))]
    pub quota_source: String,
    /// Exact remaining order-rate slots of the tightest declared bucket, when one binds.
    #[schemars(length(min = 1, max = 20))]
    pub remaining_order_slots: Option<String>,
    /// The declared bucket those remaining slots were read from.
    #[schemars(length(min = 1, max = 128))]
    pub binding_interval: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub declared_symbol_open_orders: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub declared_symbol_open_buy_quantity: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub declared_base_free: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub declared_base_locked: Option<String>,
    /// A standing venue-imposed cooldown, reported but never treated as this slice's gate.
    #[schemars(length(min = 1, max = 20))]
    pub declared_provider_wait_seconds: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub capacity_observed_at: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub interval_observed_at: Option<String>,
    /// Standing provenance limitations of the two consumed slices, carried verbatim.
    #[schemars(length(max = 5), inner(length(min = 1, max = 128)))]
    pub carried_limitations: Vec<String>,
    /// Both bound evidence versions are `sha256:` digests, so the generated wire schema must
    /// admit the full 71 characters the same way `proposal_hash` does.
    #[schemars(length(min = 71, max = 71))]
    pub binding_version: String,
    #[schemars(length(min = 71, max = 71))]
    pub state_version: String,
}

fn digest(value: &impl Serialize) -> Result<String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| crate::protocol::TradeXError::new("RISK_EVIDENCE_UNAVAILABLE"))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn same_intent(capacity: &SpotCapacityInputs, intervals: &SpotOrderIntervalInputs) -> bool {
    capacity.workspace_id == intervals.workspace_id
        && capacity.proposal_id == intervals.proposal_id
        && capacity.proposal_hash == intervals.proposal_hash
        && capacity.account_id == intervals.account_id
        && capacity.instrument_id == intervals.instrument_id
        && capacity.base_asset == intervals.base_asset
        && capacity.quote_asset == intervals.quote_asset
        && capacity.source_version == intervals.source_version
        && capacity.rule_material_version == intervals.rule_material_version
}

/// The standing limitations actually present in an obligation set, in a fixed order.
fn carried(present: &[String], known: &[&str]) -> Vec<String> {
    known
        .iter()
        .filter(|label| present.iter().any(|obligation| obligation == *label))
        .map(|label| (*label).to_string())
        .collect()
}

fn skeleton(
    capacity: &SpotCapacityInputs,
    intervals: &SpotOrderIntervalInputs,
    capacity_unit: &str,
) -> Result<SpotOwningQualification> {
    Ok(SpotOwningQualification {
        workspace_id: capacity.workspace_id.clone(),
        proposal_id: capacity.proposal_id.clone(),
        proposal_hash: capacity.proposal_hash.clone(),
        account_id: capacity.account_id.clone(),
        instrument_id: capacity.instrument_id.clone(),
        base_asset: capacity.base_asset.clone(),
        quote_asset: capacity.quote_asset.clone(),
        capacity_unit: capacity_unit.to_string(),
        outcome: RiskCheckOutcome::Unavailable,
        reason_code: SpotOwningReasonCode::SpotOwningEvidenceMismatch,
        reason: String::new(),
        binding_blocker: None,
        inventory_source: "PROVIDER_DECLARED_OPEN_ORDERS".into(),
        quota_source: "PROVIDER_DECLARED_ORDER_RATE_COUNTERS".into(),
        remaining_order_slots: None,
        binding_interval: None,
        declared_symbol_open_orders: None,
        declared_symbol_open_buy_quantity: None,
        declared_base_free: None,
        declared_base_locked: None,
        declared_provider_wait_seconds: intervals.provider_wait_seconds.clone(),
        capacity_observed_at: capacity
            .observation
            .as_ref()
            .and_then(|observation| observation.provider_observed_at.clone()),
        interval_observed_at: intervals
            .observation
            .as_ref()
            .and_then(|observation| observation.provider_observed_at.clone()),
        carried_limitations: capacity
            .observation
            .as_ref()
            .map(|observation| {
                let mut labels = carried(
                    &observation.unresolved_obligations,
                    &CARRIED_CAPACITY_LIMITATIONS,
                );
                if let Some(interval) = intervals.observation.as_ref() {
                    labels.extend(carried(
                        &interval.unresolved_obligations,
                        &CARRIED_INTERVAL_LIMITATIONS,
                    ));
                }
                labels
            })
            .unwrap_or_default(),
        binding_version: digest(&(
            capacity.binding_version.as_str(),
            intervals.binding_version.as_str(),
        ))?,
        state_version: digest(&(
            capacity.state_version.as_str(),
            intervals.state_version.as_str(),
        ))?,
    })
}

fn blocked(
    mut qualification: SpotOwningQualification,
    outcome: RiskCheckOutcome,
    reason_code: SpotOwningReasonCode,
    reason: &str,
    blocker: Option<&str>,
) -> SpotOwningQualification {
    qualification.outcome = outcome;
    qualification.reason_code = reason_code;
    qualification.reason = reason.into();
    qualification.binding_blocker = blocker.map(str::to_owned);
    qualification
}

/// Derive the owning qualification from one read-only capacity observation and one read-only
/// interval observation. Nothing here is decremented, reset or rolled forward.
pub(crate) fn qualify(
    capacity: &SpotCapacityInputs,
    intervals: &SpotOrderIntervalInputs,
    side: OrderSide,
) -> Result<SpotOwningQualification> {
    let unit = match side {
        OrderSide::Buy => capacity.quote_asset.clone(),
        OrderSide::Sell => capacity.base_asset.clone(),
    };
    let mut qualification = skeleton(capacity, intervals, &unit)?;
    if !same_intent(capacity, intervals) {
        return Ok(blocked(
            qualification,
            RiskCheckOutcome::Unavailable,
            SpotOwningReasonCode::SpotOwningEvidenceMismatch,
            "The declared capacity and interval evidence do not describe the same immutable intent, so neither can qualify it.",
            Some("SPOT_OWNING_EVIDENCE_MISMATCH"),
        ));
    }
    let Some(capacity_observation) = capacity.observation.as_ref() else {
        return Ok(blocked(
            qualification,
            RiskCheckOutcome::Unavailable,
            SpotOwningReasonCode::SpotCapacityEvidenceUnavailable,
            "No current declared capacity observation is bound to this unchanged intent.",
            Some("SPOT_CAPACITY_EVIDENCE_UNAVAILABLE"),
        ));
    };
    qualification.declared_symbol_open_orders =
        Some(capacity_observation.counts.symbol_open_orders.clone());
    qualification.declared_symbol_open_buy_quantity = capacity_observation
        .position
        .as_ref()
        .map(|position| position.selected_symbol_open_buy_original_quantity.clone());
    qualification.declared_base_free = capacity_observation
        .base_balance
        .as_ref()
        .map(|balance| balance.free.clone());
    qualification.declared_base_locked = capacity_observation
        .base_balance
        .as_ref()
        .map(|balance| balance.locked.clone());

    // The declared inventory is only usable as capacity if the venue's own coverage flags say
    // every open order was seen and classified. The standing "non-atomic" and "not execution
    // qualified" labels are not coverage gaps and stay in carried_limitations.
    let counts = &capacity_observation.counts;
    let capacity_gaps = capacity_observation
        .unresolved_obligations
        .iter()
        .filter(|obligation| {
            !CARRIED_CAPACITY_LIMITATIONS.contains(&obligation.as_str())
        })
        .count();
    if !counts.classifications_complete
        || !counts.order_coverage_complete
        || counts.list_coverage_complete == Some(false)
        || capacity_gaps > 0
    {
        return Ok(blocked(
            qualification,
            RiskCheckOutcome::Unavailable,
            SpotOwningReasonCode::SpotCapacityCoverageIncomplete,
            "The declared open-order inventory is not complete or not fully classified, so its numbers cannot be read as free capacity.",
            Some("SPOT_CAPACITY_COVERAGE_INCOMPLETE"),
        ));
    }

    // The symbol-scoped open-buy quantity and the declared base balance are reported verbatim
    // exactly as the venue declared them. They are never netted against each other and never
    // subtracted from anything: a provider balance whose `free` already excludes order locks
    // must not have the same orders removed from it a second time here.
    let Some(interval_observation) = intervals.observation.as_ref() else {
        // A retired observation is not the same statement as a missing one: the venue reported
        // counters whose window association could not be shown to hold any longer.
        return Ok(match intervals.retirement_reason.as_deref() {
            Some("POSSIBLE_INTERVAL_BOUNDARY") => blocked(
                qualification,
                RiskCheckOutcome::Unavailable,
                SpotOwningReasonCode::SpotIntervalWindowUncertain,
                "The declared order-rate counters were retired because the local association can no longer be shown to remain inside the window they were read in, so no remaining slot can be derived.",
                Some("SPOT_INTERVAL_WINDOW_UNCERTAIN"),
            ),
            _ => blocked(
                qualification,
                RiskCheckOutcome::Unavailable,
                SpotOwningReasonCode::SpotIntervalEvidenceUnavailable,
                "No current declared interval-quota observation is bound to this unchanged intent.",
                Some("SPOT_INTERVAL_EVIDENCE_UNAVAILABLE"),
            ),
        });
    };
    if !interval_observation.coverage_complete {
        return Ok(blocked(
            qualification,
            RiskCheckOutcome::Unavailable,
            SpotOwningReasonCode::SpotIntervalEvidenceUnavailable,
            "The declared order-rate counters are not a complete set for every key, IP and API, so no remaining slot can be derived.",
            Some("SPOT_INTERVAL_EVIDENCE_UNAVAILABLE"),
        ));
    }

    let mut binding: Option<(String, String)> = None;
    for counter in &interval_observation.counters {
        let Some(declaration) = interval_observation
            .declarations
            .iter()
            .find(|declaration| {
                declaration.rate_limit_type == counter.definition.rate_limit_type
                    && declaration.interval == counter.definition.interval
                    && declaration.interval_num == counter.definition.interval_num
            })
        else {
            return Ok(blocked(
                qualification,
                RiskCheckOutcome::Unavailable,
                SpotOwningReasonCode::SpotIntervalEvidenceUnavailable,
                "A declared order-rate counter has no matching declaration, so its remaining slots are unknown.",
                Some("SPOT_INTERVAL_EVIDENCE_UNAVAILABLE"),
            ));
        };
        if declaration.limit != counter.definition.limit {
            return Ok(blocked(
                qualification,
                RiskCheckOutcome::Unavailable,
                SpotOwningReasonCode::SpotIntervalEvidenceUnavailable,
                "A declared order-rate limit disagrees between its definition and its counter, so the remaining slots are unknown.",
                Some("SPOT_INTERVAL_EVIDENCE_UNAVAILABLE"),
            ));
        }
        let limit = provider_io::decimal(&json!(declaration.limit))?;
        let count = provider_io::decimal(&json!(counter.count))?;
        if provider_io::decimal_cmp(&count, &limit)? == std::cmp::Ordering::Greater {
            return Ok(blocked(
                qualification,
                RiskCheckOutcome::Unavailable,
                SpotOwningReasonCode::SpotIntervalEvidenceUnavailable,
                "A declared order-rate count exceeds its declared limit, so the venue's own numbers disagree and no remaining slot can be derived.",
                Some("SPOT_INTERVAL_EVIDENCE_UNAVAILABLE"),
            ));
        }
        let remaining = provider_io::decimal_subtract(&limit, &count)?;
        let label = format!(
            "{} {}/{} {}",
            declaration.rate_limit_type,
            declaration.limit,
            declaration.interval_num,
            declaration.interval
        );
        if provider_io::decimal_cmp(&remaining, "0")? == std::cmp::Ordering::Equal {
            return Ok(blocked(
                qualification,
                RiskCheckOutcome::Reject,
                SpotOwningReasonCode::SpotIntervalQuotaExhausted,
                "The venue's declared order-rate window has no remaining slot, so it would not admit another order now.",
                Some("SPOT_INTERVAL_QUOTA_EXHAUSTED"),
            ));
        }
        let tighter = match binding.as_ref() {
            None => true,
            Some((current, _)) => {
                matches!(
                    provider_io::decimal_cmp(&remaining, current),
                    Ok(std::cmp::Ordering::Less)
                )
            }
        };
        if tighter {
            binding = Some((remaining, label));
        }
    }
    let Some((remaining, label)) = binding else {
        return Ok(blocked(
            qualification,
            RiskCheckOutcome::Unavailable,
            SpotOwningReasonCode::SpotIntervalEvidenceUnavailable,
            "The venue declared no order-rate counter for this account, so no remaining slot can be derived.",
            Some("SPOT_INTERVAL_EVIDENCE_UNAVAILABLE"),
        ));
    };

    qualification.outcome = RiskCheckOutcome::Pass;
    qualification.reason_code = SpotOwningReasonCode::SpotOwningQualified;
    qualification.reason = "Every declared capacity and order-rate fact is current, complete and consistent; the venue's own figures bound this intent. Execution authority, fees, required FX and authenticated preflight remain separate.".into();
    qualification.remaining_order_slots = Some(remaining);
    qualification.binding_interval = Some(label);
    Ok(qualification)
}
