//! Local ordinary BASE MARKET admission comparisons. No provider I/O or execution authority.
use crate::{
    ControlPlane,
    portfolio::decimal_add,
    protocol::*,
    provider_io::decimal_cmp,
    risk::RiskCheckOutcome,
    spot_capacity::SpotCapacityInputs,
    spot_order_intervals::SpotOrderIntervalInputs,
    spot_owning::SpotOwningQualification,
    spot_proposal_rules::{SpotProposalRules, SpotRuleEvaluation, digest, field},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;

const POLICY: &str = "ORDINARY_BASE_MARKET_PLACE_EMPTY_POSITION_CONSERVATIVE_CATEGORY_V1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotMarketRuleComparison {
    #[schemars(length(min = 1, max = 64))]
    pub observed: String,
    #[schemars(length(min = 1, max = 64))]
    pub increment: String,
    #[schemars(length(min = 1, max = 64))]
    pub total: String,
    #[schemars(length(min = 1, max = 64))]
    pub limit: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotMarketRuleComposition {
    #[schemars(length(min = 71, max = 71))]
    pub capacity_version: String,
    #[schemars(length(min = 71, max = 71))]
    pub interval_version: String,
    pub admission_outcome: RiskCheckOutcome,
    #[schemars(length(min = 1, max = 256))]
    pub admission_reason: String,
    #[schemars(length(max = 5), inner(length(min = 1, max = 128)))]
    pub carried_limitations: Vec<String>,
}

fn unavailable() -> TradeXError {
    TradeXError::new("CURRENT_MARKET_RULE_COMPOSITION_UNAVAILABLE")
}

fn compare(
    row: &mut SpotRuleEvaluation,
    observed: &str,
    increment: &str,
    limit: &str,
    unit: &str,
) -> Result<()> {
    let total = decimal_add(observed, increment)?;
    if [observed, increment, total.as_str(), limit]
        .iter()
        .any(|value| {
            value.is_empty()
                || value.len() > 64
                || decimal_cmp(value, "0")
                    .ok()
                    .is_none_or(|o| o == Ordering::Less)
        })
    {
        return Err(unavailable());
    }
    row.outcome = if decimal_cmp(&total, limit)? == Ordering::Greater {
        RiskCheckOutcome::Reject
    } else {
        RiskCheckOutcome::Pass
    };
    row.reason_code = if row.outcome == RiskCheckOutcome::Pass {
        "LOCAL_MARKET_RULE_SATISFIED"
    } else {
        "LOCAL_MARKET_RULE_REJECTED"
    }
    .into();
    row.unit = Some(unit.into());
    row.local_comparison = Some(SpotMarketRuleComparison {
        observed: observed.into(),
        increment: increment.into(),
        total,
        limit: limit.into(),
    });
    Ok(())
}

fn not_applicable(row: &mut SpotRuleEvaluation) {
    row.applicable = false;
    row.outcome = RiskCheckOutcome::Pass;
    row.reason_code = "NOT_APPLICABLE_TO_NEW_MARKET_PLACE".into();
}

fn dynamic_row(
    row: &mut SpotRuleEvaluation,
    constraint: &SpotRuleConstraint,
    proposal: &OrderProposal,
    capacity: &SpotCapacityInputs,
) -> Result<bool> {
    let Some(observation) = capacity.observation.as_ref() else {
        return Err(unavailable());
    };
    let counts = &observation.counts;
    let kind = constraint.rule_type.as_str();
    if kind == "MAX_POSITION" {
        let position = observation.position.as_ref().ok_or_else(unavailable)?;
        let balance = observation
            .base_balance
            .as_ref()
            .filter(|balance| balance.asset == capacity.base_asset)
            .ok_or_else(unavailable)?;
        // The provider does not document how partial fills or foreign symbols map into
        // this position obligation. Initially only a complete empty account inventory qualifies.
        if counts.account_open_orders.as_deref() != Some("0")
            || !position.asset_exposure_complete
            || decimal_cmp(&position.selected_symbol_open_buy_original_quantity, "0")?
                != Ordering::Equal
            || decimal_cmp(&position.selected_symbol_open_buy_executed_quantity, "0")?
                != Ordering::Equal
        {
            return Err(unavailable());
        }
        let SpotRuleValue::Decimal(limit) = field(constraint, "maxPosition")? else {
            return Err(unavailable());
        };
        compare(
            row,
            &decimal_add(&balance.free, &balance.locked)?,
            &proposal.fields.quantity.value,
            limit,
            &capacity.base_asset,
        )?;
        return Ok(true);
    }
    let (observed, increment, field_name) = match kind {
        "MAX_NUM_ORDERS" => (
            Some(counts.symbol_open_orders.as_str()),
            "1",
            "maxNumOrders",
        ),
        "EXCHANGE_MAX_NUM_ORDERS" => (counts.account_open_orders.as_deref(), "1", "maxNumOrders"),
        "MAX_NUM_ALGO_ORDERS" => (
            counts.symbol_algo_orders.as_deref(),
            "0",
            "maxNumAlgoOrders",
        ),
        "EXCHANGE_MAX_NUM_ALGO_ORDERS" => (
            counts.account_algo_orders.as_deref(),
            "0",
            "maxNumAlgoOrders",
        ),
        "MAX_NUM_ICEBERG_ORDERS" => (
            Some(counts.symbol_iceberg_orders.as_str()),
            "0",
            "maxNumIcebergOrders",
        ),
        "EXCHANGE_MAX_NUM_ICEBERG_ORDERS" => (
            counts.account_iceberg_orders.as_deref(),
            "0",
            "maxNumIcebergOrders",
        ),
        "MAX_NUM_ORDER_LISTS" => (
            counts.symbol_open_order_lists.as_deref(),
            "0",
            "maxNumOrderLists",
        ),
        "EXCHANGE_MAX_NUM_ORDER_LISTS" => (
            counts.account_open_order_lists.as_deref(),
            "0",
            "maxNumOrderLists",
        ),
        _ => return Ok(false),
    };
    let SpotRuleValue::Integer(limit) = field(constraint, field_name)? else {
        return Err(unavailable());
    };
    compare(
        row,
        observed.ok_or_else(unavailable)?,
        increment,
        limit,
        if kind.ends_with("ORDER_LISTS") {
            "ORDER_LISTS"
        } else {
            "ORDERS"
        },
    )?;
    Ok(true)
}

pub(crate) fn compose(
    control: &mut ControlPlane,
    proposal: &OrderProposal,
    mut rules: SpotProposalRules,
    capacity: Option<&SpotCapacityInputs>,
    intervals: Option<&SpotOrderIntervalInputs>,
    owning: Option<&SpotOwningQualification>,
) -> Result<SpotProposalRules> {
    if proposal.fields.order_type != OrderType::Market
        || proposal.fields.quantity.r#type != OrderQuantityType::Base
        || proposal.fields.time_in_force != TimeInForce::Day
    {
        return Ok(rules);
    }
    let source = crate::financial_sources::connection(
        control,
        &rules.workspace_id,
        FinancialSourceKind::BinanceSpotRules,
    )
    .ok();
    let evidence = source.as_ref().and_then(|source| match &source.evidence {
        Some(FinancialSourceEvidence::BinanceSpotRules(e))
            if source.status == DataSourceStatus::Available
                && source.state_version == rules.source_version
                && rules.material_version.as_ref() == Some(&e.material_version)
                && e.instrument_id == rules.instrument_id
                && e.base_asset == rules.base_asset
                && e.quote_asset == rules.quote_asset =>
        {
            Some(e)
        }
        _ => None,
    });
    let bound = match (capacity, intervals, owning) {
        (Some(capacity), Some(intervals), Some(owning)) => {
            rules.workspace_id == proposal.workspace_id
                && rules.proposal_id == proposal.proposal_id
                && rules.proposal_hash == proposal.proposal_hash
                && capacity.workspace_id == rules.workspace_id
                && capacity.proposal_id == rules.proposal_id
                && capacity.proposal_hash == rules.proposal_hash
                && capacity.account_id == rules.account_id
                && capacity.instrument_id == rules.instrument_id
                && capacity.base_asset == rules.base_asset
                && capacity.quote_asset == rules.quote_asset
                && capacity.source_version == rules.source_version
                && capacity.rule_material_version == rules.material_version
                && intervals.workspace_id == capacity.workspace_id
                && intervals.proposal_id == capacity.proposal_id
                && intervals.proposal_hash == capacity.proposal_hash
                && intervals.account_id == capacity.account_id
                && intervals.instrument_id == capacity.instrument_id
                && intervals.base_asset == capacity.base_asset
                && intervals.quote_asset == capacity.quote_asset
                && intervals.source_version == capacity.source_version
                && intervals.rule_material_version == capacity.rule_material_version
                && owning.workspace_id == rules.workspace_id
                && owning.proposal_id == rules.proposal_id
                && owning.proposal_hash == rules.proposal_hash
                && owning.account_id == rules.account_id
                && owning.instrument_id == rules.instrument_id
        }
        _ => false,
    } && evidence.is_some();
    let admission = if bound {
        owning
            .map(|o| o.outcome)
            .unwrap_or(RiskCheckOutcome::Unavailable)
    } else {
        RiskCheckOutcome::Unavailable
    };
    if let (Some(capacity), Some(intervals), Some(owning)) = (capacity, intervals, owning) {
        rules.market_composition = Some(SpotMarketRuleComposition {
            capacity_version: capacity.state_version.clone(),
            interval_version: intervals.state_version.clone(),
            admission_outcome: admission,
            admission_reason: if bound {
                owning.reason.clone()
            } else {
                "Current exact MARKET composition inputs are not bound to this Proposal and rule source.".into()
            },
            carried_limitations: owning.carried_limitations.clone(),
        });
    }
    if admission != RiskCheckOutcome::Pass {
        rules
            .unresolved_obligations
            .push("CURRENT_MARKET_ADMISSION_NOT_QUALIFIED".into());
    }
    if let Some(evidence) = evidence {
        let qualified_capacity = capacity.filter(|_| bound).filter(|capacity| {
            capacity
                .observation
                .as_ref()
                .is_some_and(crate::spot_owning::capacity_coverage_complete)
        });
        for row in &mut rules.rules {
            if row.rule_type == "ORDER_FORM"
                && row.outcome != RiskCheckOutcome::Reject
                && (!matches!(
                    evidence.default_self_trade_prevention_mode.as_str(),
                    "NONE" | "EXPIRE_MAKER" | "EXPIRE_TAKER" | "EXPIRE_BOTH"
                ) || !evidence
                    .allowed_self_trade_prevention_modes
                    .contains(&evidence.default_self_trade_prevention_mode))
            {
                row.outcome = RiskCheckOutcome::Unavailable;
                row.reason_code = "UNSUPPORTED_EFFECTIVE_SELF_TRADE_PREVENTION".into();
                rules.unresolved_obligations.push(row.reason_code.clone());
            }
            let Some(constraint) = evidence.constraints.iter().find(|c| {
                c.scope == row.scope
                    && c.origin == row.origin
                    && c.rule_type == row.rule_type
                    && c.known_schema
                    && c.unsupported_fields.is_empty()
            }) else {
                continue;
            };
            let resolved = if constraint.rule_type == "MAX_NUM_ORDER_AMENDS"
                || (constraint.rule_type == "MAX_POSITION"
                    && proposal.fields.side == OrderSide::Sell)
            {
                not_applicable(row);
                true
            } else {
                qualified_capacity.is_some_and(|capacity| {
                    dynamic_row(row, constraint, proposal, capacity).unwrap_or(false)
                })
            };
            if resolved {
                let key = format!("{:?}:{:?}:{}", row.origin, row.scope, row.rule_type);
                rules
                    .unresolved_obligations
                    .retain(|obligation| obligation != &key);
            }
        }
    }
    rules.outcome = if admission == RiskCheckOutcome::Reject
        || rules
            .rules
            .iter()
            .any(|r| r.outcome == RiskCheckOutcome::Reject)
    {
        RiskCheckOutcome::Reject
    } else if admission != RiskCheckOutcome::Pass
        || !rules.unresolved_obligations.is_empty()
        || rules
            .rules
            .iter()
            .any(|r| r.outcome == RiskCheckOutcome::Unavailable)
    {
        RiskCheckOutcome::Unavailable
    } else {
        RiskCheckOutcome::Pass
    };
    rules.binding_version = digest(&json!([
        POLICY,
        rules.binding_version,
        rules.state_version,
        rules.rules,
        rules.market_composition,
        rules.unresolved_obligations,
        capacity.map(|c| (&c.binding_version, &c.state_version)),
        intervals.map(|i| (&i.binding_version, &i.state_version)),
        owning.map(|o| (&o.binding_version, &o.state_version))
    ]))?;
    rules.state_version = digest(&json!([
        POLICY,
        rules,
        capacity.map(|c| &c.state_version),
        intervals.map(|i| &i.state_version),
        owning.map(|o| &o.state_version)
    ]))?;
    Ok(rules)
}
