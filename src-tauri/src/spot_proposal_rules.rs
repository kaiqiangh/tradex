//! Intent-bound read-only rule explanations. Static success is not financial authority.
use crate::provider_io::{ProviderEndpoint, ProviderHttp, ProviderHttpMethod};
use crate::{ControlPlane, protocol::*, risk::RiskCheckOutcome};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotRulesQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotRulesRefresh {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub expected_state_version: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotRuleReference {
    #[schemars(length(min = 1, max = 32))]
    pub kind: String,
    #[schemars(length(min = 1, max = 64))]
    pub price: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub provider_observed_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub received_at: String,
    #[schemars(length(min = 1, max = 128))]
    pub digest: String,
    #[schemars(range(min = 0, max = 1440))]
    pub interval_minutes: Option<u64>,
}
/// Genuine-current-only execution reference observation. The digest survives capture;
/// the price does not.
pub(crate) const EXECUTION_REFERENCE_KIND: &str = "EXECUTION_REFERENCE";
/// Documented PRICE_RANGE state for one immutable Proposal. Every state is a snapshot
/// explanation, never placement authority or a promised fill.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotPriceRangeState {
    /// Scoped, complete execution-rule evidence has no PRICE_RANGE rule: not enforced.
    NoRule,
    /// The execution rule is present but its active schema is unknown or unsupported.
    UnsupportedConfiguration,
    /// The selected side is missing at least one documented multiplier: not enforced.
    NotEnforcedSelectedSide,
    /// A market form states no execution price, so no placement-time comparison exists.
    NoStatedExecutionPrice,
    /// The selected side is enforced but no genuine execution reference was observed.
    ReferenceMissing,
    /// The execution reference lookup failed, is stale or cannot be multiplied exactly.
    ReferenceUnavailable,
    /// The provider reported an explicit null reference price: not enforced.
    ReferenceExplicitNull,
    /// A genuine current reference bounded this snapshot.
    SnapshotAvailable,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotPriceRangeDirection {
    #[schemars(length(min = 1, max = 4))]
    pub direction: String,
    #[schemars(length(min = 1, max = 64))]
    pub lower_multiplier: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub upper_multiplier: Option<String>,
    pub enforced: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotPriceRangePreview {
    pub state: SpotPriceRangeState,
    pub side: OrderSide,
    #[schemars(length(min = 1, max = 32))]
    pub unit: String,
    #[schemars(length(max = 2))]
    pub directions: Vec<SpotPriceRangeDirection>,
    #[schemars(length(min = 1, max = 64))]
    pub lower_bound: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub upper_bound: Option<String>,
    /// Digest of the snapshot bounds. Like `SpotRuleReference::digest`, it survives
    /// capture so a saved review can still be traced to the bounds it was made against
    /// even though the bounds themselves are withheld.
    #[schemars(length(min = 1, max = 128))]
    pub bounds_digest: Option<String>,
    pub reference: Option<SpotRuleReference>,
    #[schemars(length(min = 1, max = 128))]
    pub explanation: String,
}
#[derive(Default)]
pub(crate) struct Runtime {
    sequence: u64,
    slots: HashMap<String, Slot>,
}
struct Slot {
    sequence: u64,
    binding: String,
    references: Vec<(SpotRuleReference, Instant)>,
    execution: Option<(SpotRuleReference, Instant)>,
    failure: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotRuleEvaluation {
    pub scope: SpotRuleScope,
    pub origin: SpotRuleOrigin,
    #[schemars(length(min = 1, max = 64))]
    pub rule_type: String,
    pub applicable: bool,
    pub outcome: RiskCheckOutcome,
    #[schemars(length(min = 1, max = 128))]
    pub reason_code: String,
    #[schemars(length(min = 1, max = 64))]
    pub unit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotProposalRules {
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
    #[schemars(length(min = 1, max = 128))]
    pub source_version: String,
    #[schemars(length(min = 64, max = 64))]
    pub material_version: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: Option<String>,
    pub outcome: RiskCheckOutcome,
    #[schemars(length(max = 272))]
    pub rules: Vec<SpotRuleEvaluation>,
    #[schemars(length(max = 272), inner(length(min = 1, max = 128)))]
    pub unresolved_obligations: Vec<String>,
    #[schemars(length(max = 2))]
    pub references: Vec<SpotRuleReference>,
    #[schemars(length(min = 1, max = 128))]
    pub reference_failure: Option<String>,
    #[schemars(length(max = 256), inner(length(min = 1, max = 128)))]
    pub reference_purposes: Vec<String>,
    pub price_range_preview: Option<SpotPriceRangePreview>,
    #[schemars(length(min = 1, max = 128))]
    pub binding_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub state_version: String,
}

fn unavailable() -> TradeXError {
    TradeXError::new("ORDER_FILTERS_UNAVAILABLE")
}
fn field<'a>(rule: &'a SpotRuleConstraint, name: &str) -> Result<&'a SpotRuleValue> {
    let mut matching = rule.fields.iter().filter(|f| f.name == name);
    let value = &matching.next().ok_or_else(unavailable)?.value;
    if matching.next().is_some() {
        return Err(unavailable());
    }
    Ok(value)
}
fn decimal<'a>(rule: &'a SpotRuleConstraint, name: &str) -> Result<&'a str> {
    match field(rule, name)? {
        SpotRuleValue::Decimal(value) if !value.starts_with('-') => Ok(value),
        _ => Err(unavailable()),
    }
}
fn boolean(rule: &SpotRuleConstraint, name: &str) -> Result<bool> {
    match field(rule, name)? {
        SpotRuleValue::Boolean(value) => Ok(*value),
        _ => Err(unavailable()),
    }
}
fn cmp(a: &str, b: &str) -> Result<Ordering> {
    crate::provider_io::decimal_cmp(a, b)
}
fn range(value: &str, minimum: &str, maximum: Option<&str>) -> Result<bool> {
    Ok(cmp(value, minimum)? != Ordering::Less
        && maximum.is_none_or(|maximum| cmp(value, maximum).is_ok_and(|o| o != Ordering::Greater)))
}
fn multiplier(rule: &SpotRuleConstraint, name: &str) -> Option<String> {
    match field(rule, name) {
        Ok(SpotRuleValue::Decimal(value)) => Some(value.clone()),
        _ => None,
    }
}
fn price_range_multipliers(fields: &OrderDraftFields) -> (&'static str, &'static str) {
    if fields.side == OrderSide::Buy {
        ("bidLimitMultDown", "bidLimitMultUp")
    } else {
        ("askLimitMultDown", "askLimitMultUp")
    }
}
fn price_range_direction(
    rule: &SpotRuleConstraint,
    direction: &str,
    down: &str,
    up: &str,
) -> SpotPriceRangeDirection {
    let lower = multiplier(rule, down);
    let upper = multiplier(rule, up);
    SpotPriceRangeDirection {
        direction: direction.into(),
        enforced: lower.is_some() && upper.is_some(),
        lower_multiplier: lower,
        upper_multiplier: upper,
    }
}
/// The documented execution reference is read only when a stated limit price makes the
/// selected-side snapshot bound comparable at placement time.
fn execution_reference_required(rule: &SpotRuleConstraint, fields: &OrderDraftFields) -> bool {
    if rule.rule_type != "PRICE_RANGE"
        || fields.order_type != OrderType::Limit
        || !rule.known_schema
        || !rule.unsupported_fields.is_empty()
    {
        return false;
    }
    let (down, up) = price_range_multipliers(fields);
    multiplier(rule, down).is_some() && multiplier(rule, up).is_some()
}
fn requires_reference(rule: &SpotRuleConstraint, fields: &OrderDraftFields) -> bool {
    if !rule.known_schema || !rule.unsupported_fields.is_empty() {
        return false;
    }
    if rule.rule_type == "PRICE_RANGE" {
        return execution_reference_required(rule, fields);
    }
    if fields.order_type == OrderType::Limit {
        return matches!(
            rule.rule_type.as_str(),
            "PERCENT_PRICE" | "PERCENT_PRICE_BY_SIDE"
        );
    }
    if fields.quantity.r#type != OrderQuantityType::Base {
        return false;
    }
    match rule.rule_type.as_str() {
        "MIN_NOTIONAL" => boolean(rule, "applyToMarket").unwrap_or(false),
        "NOTIONAL" => {
            boolean(rule, "applyMinToMarket").unwrap_or(false)
                || boolean(rule, "applyMaxToMarket").unwrap_or(false)
        }
        "MAX_ASSET" => {
            matches!(field(rule,"asset"), Ok(SpotRuleValue::Asset(asset)) if asset == "USDT")
        }
        _ => false,
    }
}
fn rule_reference<'a>(
    rule: &SpotRuleConstraint,
    reference: Option<&'a SpotRuleReference>,
) -> Result<&'a str> {
    let reference = reference.ok_or_else(|| TradeXError::new("REFERENCE_PRICE_MISSING"))?;
    if matches!(reference.kind.as_str(), "AVERAGE_PRICE" | "LAST_TRADE") {
        let minutes = match field(rule, "avgPriceMins")? {
            SpotRuleValue::Integer(minutes) => minutes.parse::<u64>().map_err(|_| unavailable())?,
            _ => return Err(unavailable()),
        };
        if reference.interval_minutes != Some(minutes)
            || (reference.kind == "LAST_TRADE") != (minutes == 0)
        {
            return Err(TradeXError::new("REFERENCE_PRICE_INTERVAL_MISMATCH"));
        }
    }
    reference.price.as_deref().ok_or_else(unavailable)
}
fn static_result(
    rule: &SpotRuleConstraint,
    fields: &OrderDraftFields,
    reference: Option<&SpotRuleReference>,
) -> Result<Option<bool>> {
    let market = fields.order_type == OrderType::Market;
    let base = fields.quantity.r#type == OrderQuantityType::Base;
    match rule.rule_type.as_str() {
        "PRICE_FILTER" if market => Ok(None),
        "PRICE_FILTER" => {
            let price = fields.limit_price.as_deref().ok_or_else(unavailable)?;
            let min = decimal(rule, "minPrice")?;
            let max = decimal(rule, "maxPrice")?;
            let tick = decimal(rule, "tickSize")?;
            Ok(Some(
                (min == "0" || cmp(price, min)? != Ordering::Less)
                    && (max == "0" || cmp(price, max)? != Ordering::Greater)
                    && (tick == "0" || crate::provider_io::binance::multiple_of(price, tick)?),
            ))
        }
        "MARKET_LOT_SIZE" if !market => Ok(None),
        "LOT_SIZE" | "MARKET_LOT_SIZE" => {
            if !base {
                return Err(unavailable());
            }
            let min = decimal(rule, "minQty")?;
            let max = decimal(rule, "maxQty")?;
            let step = decimal(rule, "stepSize")?;
            // Price-filter zero disabling must not be generalized to quantity rules.
            if step == "0" || max == "0" {
                return Err(unavailable());
            }
            Ok(Some(
                range(&fields.quantity.value, min, Some(max))?
                    && crate::provider_io::binance::multiple_of(&fields.quantity.value, step)?,
            ))
        }
        "MIN_NOTIONAL" | "NOTIONAL" => {
            let min_enabled = !market
                || boolean(
                    rule,
                    if rule.rule_type == "MIN_NOTIONAL" {
                        "applyToMarket"
                    } else {
                        "applyMinToMarket"
                    },
                )?;
            let max_enabled =
                rule.rule_type == "NOTIONAL" && (!market || boolean(rule, "applyMaxToMarket")?);
            if !min_enabled && !max_enabled {
                return Ok(None);
            }
            let amount = if base {
                crate::provider_io::binance::multiply(
                    &fields.quantity.value,
                    if market {
                        rule_reference(rule, reference)?
                    } else {
                        fields.limit_price.as_deref().ok_or_else(unavailable)?
                    },
                )?
            } else {
                fields.quantity.value.clone()
            };
            Ok(Some(
                (!min_enabled || cmp(&amount, decimal(rule, "minNotional")?)? != Ordering::Less)
                    && (!max_enabled
                        || cmp(&amount, decimal(rule, "maxNotional")?)? != Ordering::Greater),
            ))
        }
        "PERCENT_PRICE" | "PERCENT_PRICE_BY_SIDE" if market => Ok(None),
        "PERCENT_PRICE" | "PERCENT_PRICE_BY_SIDE" => {
            let reference = rule_reference(rule, reference)?;
            let price = fields.limit_price.as_deref().ok_or_else(unavailable)?;
            let (down, up) = if rule.rule_type == "PERCENT_PRICE" {
                ("multiplierDown", "multiplierUp")
            } else if fields.side == OrderSide::Buy {
                ("bidMultiplierDown", "bidMultiplierUp")
            } else {
                ("askMultiplierDown", "askMultiplierUp")
            };
            let min = crate::provider_io::binance::multiply(reference, decimal(rule, down)?)?;
            let max = crate::provider_io::binance::multiply(reference, decimal(rule, up)?)?;
            Ok(Some(range(price, &min, Some(&max))?))
        }
        "MAX_ASSET" => {
            let asset = match field(rule, "asset")? {
                SpotRuleValue::Asset(value) => value,
                _ => return Err(unavailable()),
            };
            let (base_asset, quote_asset) = match fields.instrument_id.as_str() {
                "crypto:BTC/USDT:spot" => ("BTC", "USDT"),
                "crypto:ETH/USDT:spot" => ("ETH", "USDT"),
                _ => return Err(unavailable()),
            };
            let amount = if asset == base_asset {
                if !base {
                    return Err(unavailable());
                }
                fields.quantity.value.clone()
            } else if asset == quote_asset {
                if base {
                    crate::provider_io::binance::multiply(
                        &fields.quantity.value,
                        if market {
                            rule_reference(rule, reference)?
                        } else {
                            fields.limit_price.as_deref().ok_or_else(unavailable)?
                        },
                    )?
                } else {
                    fields.quantity.value.clone()
                }
            } else {
                return Err(unavailable());
            };
            Ok(Some(
                cmp(&amount, decimal(rule, "limit")?)? != Ordering::Greater,
            ))
        }
        "PRICE_RANGE" => Err(TradeXError::new("UNSUPPORTED_RULE_EVALUATOR")),
        "MAX_POSITION" => Err(TradeXError::new(
            "CURRENT_FREE_LOCKED_AND_OPEN_BUY_POSITION_REQUIRED",
        )),
        "MAX_NUM_ORDER_AMENDS" => Err(TradeXError::new("CURRENT_ORDER_AMEND_COUNT_REQUIRED")),
        "MAX_NUM_ORDERS"
        | "EXCHANGE_MAX_NUM_ORDERS"
        | "MAX_NUM_ALGO_ORDERS"
        | "EXCHANGE_MAX_NUM_ALGO_ORDERS"
        | "MAX_NUM_ICEBERG_ORDERS"
        | "EXCHANGE_MAX_NUM_ICEBERG_ORDERS"
        | "MAX_NUM_ORDER_LISTS"
        | "EXCHANGE_MAX_NUM_ORDER_LISTS" => Err(TradeXError::new(
            "CURRENT_ACCOUNT_EXCHANGE_ORDER_COUNTS_REQUIRED",
        )),
        _ => Err(TradeXError::new("UNSUPPORTED_RULE_EVALUATOR")),
    }
}

fn price_range_preview(
    evidence: &BinanceSpotRuleEvidence,
    fields: &OrderDraftFields,
    base_asset: &str,
    slot: Option<&Slot>,
    maximum_reference_age: u64,
    time: &TimeStatus,
) -> SpotPriceRangePreview {
    let unit = format!("USDT/{base_asset}");
    let mut preview = SpotPriceRangePreview {
        state: SpotPriceRangeState::NoRule,
        side: fields.side,
        unit,
        directions: Vec::new(),
        lower_bound: None,
        upper_bound: None,
        bounds_digest: None,
        reference: None,
        explanation: "PRICE_RANGE_NOT_CONFIGURED".into(),
    };
    let rule = evidence
        .constraints
        .iter()
        .find(|r| r.rule_type == "PRICE_RANGE" && r.scope == SpotRuleScope::Execution);
    let Some(rule) = rule else {
        return preview;
    };
    preview.directions = vec![
        price_range_direction(rule, "BID", "bidLimitMultDown", "bidLimitMultUp"),
        price_range_direction(rule, "ASK", "askLimitMultDown", "askLimitMultUp"),
    ];
    if !rule.known_schema || !rule.unsupported_fields.is_empty() {
        preview.state = SpotPriceRangeState::UnsupportedConfiguration;
        preview.explanation = "UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA".into();
        return preview;
    }
    if fields.order_type != OrderType::Limit {
        preview.state = SpotPriceRangeState::NoStatedExecutionPrice;
        preview.explanation = "PRICE_RANGE_EXECUTION_TIME_ONLY".into();
        return preview;
    }
    let (down, up) = price_range_multipliers(fields);
    let selected = if fields.side == OrderSide::Buy {
        &preview.directions[0]
    } else {
        &preview.directions[1]
    };
    if !selected.enforced {
        preview.state = SpotPriceRangeState::NotEnforcedSelectedSide;
        preview.explanation = "PRICE_RANGE_NOT_ENFORCED_FOR_SELECTED_SIDE".into();
        return preview;
    }
    let observation = slot
        .and_then(|slot| slot.execution.as_ref())
        .filter(|(reference, _)| reference.kind == EXECUTION_REFERENCE_KIND);
    let fresh = observation.filter(|(reference, at)| {
        at.elapsed() <= Duration::from_secs(maximum_reference_age)
            && valid_age(&reference.provider_observed_at, time, maximum_reference_age).is_ok()
    });
    let Some((reference, _)) = fresh else {
        let stale = observation.is_some();
        // A stale or expired observation keeps its provenance so the review stays
        // traceable, but never its price: expired input is not current evidence and
        // must not be renewed into the current preview by a plain get.
        preview.reference = observation.map(|(reference, _)| SpotRuleReference {
            price: None,
            ..reference.clone()
        });
        let failure = slot.and_then(|slot| slot.failure.clone());
        preview.state = if failure.is_some() || stale {
            SpotPriceRangeState::ReferenceUnavailable
        } else {
            SpotPriceRangeState::ReferenceMissing
        };
        preview.explanation = if stale {
            "REFERENCE_PRICE_STALE".into()
        } else {
            failure.unwrap_or_else(|| "EXECUTION_REFERENCE_PRICE_MISSING".into())
        };
        return preview;
    };
    preview.reference = Some(reference.clone());
    let Some(price) = reference.price.as_deref() else {
        preview.state = SpotPriceRangeState::ReferenceExplicitNull;
        preview.explanation = "PRICE_RANGE_NOT_ENFORCED_WITHOUT_REFERENCE".into();
        return preview;
    };
    let bound = |name: &str| match multiplier(rule, name).as_deref() {
        Some(multiplier) => crate::provider_io::binance::multiply(price, multiplier).ok(),
        None => None,
    };
    match (bound(down), bound(up)) {
        (Some(lower), Some(upper)) => {
            preview.bounds_digest = digest(&json!([
                reference.digest,
                preview.side,
                preview.unit,
                lower,
                upper
            ]))
            .ok();
            preview.lower_bound = Some(lower);
            preview.upper_bound = Some(upper);
            preview.state = SpotPriceRangeState::SnapshotAvailable;
            preview.explanation = "PRICE_RANGE_SNAPSHOT_BOUNDS_ONLY".into();
        }
        _ => {
            preview.state = SpotPriceRangeState::ReferenceUnavailable;
            preview.explanation = "PRICE_RANGE_ARITHMETIC_UNAVAILABLE".into();
        }
    }
    preview
}
fn price_range_row(
    preview: &SpotPriceRangePreview,
    unresolved: &mut Vec<String>,
) -> SpotRuleEvaluation {
    let (applicable, outcome, reason) = match preview.state {
        SpotPriceRangeState::SnapshotAvailable => {
            unresolved.push("EXECUTION_PRICE_RANGE_UNQUALIFIED".into());
            (
                true,
                RiskCheckOutcome::Unavailable,
                "PRICE_RANGE_SNAPSHOT_BOUNDS_ONLY",
            )
        }
        SpotPriceRangeState::ReferenceMissing => {
            unresolved.push("EXECUTION_PRICE_RANGE_UNQUALIFIED".into());
            (
                true,
                RiskCheckOutcome::Unavailable,
                "EXECUTION_REFERENCE_PRICE_MISSING",
            )
        }
        SpotPriceRangeState::ReferenceUnavailable => {
            unresolved.push("EXECUTION_PRICE_RANGE_UNQUALIFIED".into());
            (true, RiskCheckOutcome::Unavailable, preview.explanation.as_str())
        }
        SpotPriceRangeState::NoStatedExecutionPrice => (
            false,
            RiskCheckOutcome::Pass,
            "PRICE_RANGE_EXECUTION_TIME_ONLY",
        ),
        SpotPriceRangeState::NotEnforcedSelectedSide => (
            false,
            RiskCheckOutcome::Pass,
            "PRICE_RANGE_NOT_ENFORCED_FOR_SELECTED_SIDE",
        ),
        SpotPriceRangeState::ReferenceExplicitNull => (
            false,
            RiskCheckOutcome::Pass,
            "PRICE_RANGE_NOT_ENFORCED_WITHOUT_REFERENCE",
        ),
        SpotPriceRangeState::NoRule | SpotPriceRangeState::UnsupportedConfiguration => (
            true,
            RiskCheckOutcome::Unavailable,
            "UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA",
        ),
    };
    SpotRuleEvaluation {
        scope: SpotRuleScope::Execution,
        origin: SpotRuleOrigin::ExecutionRules,
        rule_type: "PRICE_RANGE".into(),
        applicable,
        outcome,
        reason_code: reason.into(),
        unit: Some(preview.unit.clone()),
    }
}

pub(crate) fn get(control: &mut ControlPlane, input: &SpotRulesQuery) -> Result<SpotProposalRules> {
    control.require_workspace(&input.workspace_id)?;
    crate::storage::validate_order_proposal_id(&input.proposal_id)?;
    let store = control.store.as_ref().ok_or_else(unavailable)?;
    let proposal = store.order_proposal(&input.proposal_id)?;
    if proposal.workspace_id != input.workspace_id {
        return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
    }
    let fields = &proposal.fields;
    if fields.environment != ExecutionContext::BinanceLive || fields.venue != "BINANCE" {
        return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
    }
    let base_asset = match fields.instrument_id.as_str() {
        "crypto:BTC/USDT:spot" => "BTC",
        "crypto:ETH/USDT:spot" => "ETH",
        _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
    };
    let account = store.account(fields.account_id.as_deref().ok_or_else(unavailable)?)?;
    if account.provider_id != "binance" || account.environment != "LIVE" {
        return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
    }
    let policy = store.risk_or_new()?;
    let maximum_reference_age = reference_age(
        policy
            .as_ref()
            .map(|p| p.policy.stale_quote_threshold_seconds),
    );
    let market_version = store
        .binance_market_source()?
        .state_version(&input.workspace_id);
    let source = crate::financial_sources::connection(
        control,
        &input.workspace_id,
        FinancialSourceKind::BinanceSpotRules,
    )?;
    let evidence = match &source.evidence {
        Some(FinancialSourceEvidence::BinanceSpotRules(e))
            if source.status == DataSourceStatus::Available
                && source.connection_id.as_deref() == Some(account.connection_id.as_str())
                && e.binding.connection_id == account.connection_id
                && e.binding.account_version == account.state_version
                && e.binding.source_version == source.state_version
                && e.instrument_id == fields.instrument_id
                && e.base_asset == base_asset
                && e.quote_asset == "USDT"
                && account
                    .data
                    .as_ref()
                    .is_some_and(|d| d.remote_account_id == e.remote_account_id) =>
        {
            Some(e)
        }
        _ => None,
    };
    let binding_version = digest(&json!([
        input.workspace_id,
        proposal.proposal_hash,
        source.state_version,
        evidence.map(|e| (&e.material_version, &e.observed_at)),
        account.state_version,
        market_version,
        policy.as_ref().map(|p| &p.state_version),
        maximum_reference_age,
        control.session,
        control.time.generation()
    ]))?;
    let time = control.time.status(&input.workspace_id)?;
    let slot = control
        .spot_rule_runtime
        .slots
        .get(&input.proposal_id)
        .filter(|slot| slot.binding == binding_version);
    let references: Vec<&SpotRuleReference> = slot
        .into_iter()
        .flat_map(|slot| slot.references.iter())
        .filter(|(r, at)| {
            at.elapsed() <= Duration::from_secs(maximum_reference_age)
                && valid_age(&r.provider_observed_at, &time, maximum_reference_age).is_ok()
        })
        .map(|(r, _)| r)
        .collect();
    let price_range_preview = evidence.map(|e| {
        price_range_preview(e, fields, base_asset, slot, maximum_reference_age, &time)
    });
    let mut rules = Vec::new();
    let mut unresolved = Vec::new();
    if let Some(e) = evidence {
        let has = |kind: &str| {
            e.constraints
                .iter()
                .any(|r| r.rule_type == kind && r.known_schema)
        };
        if !has("LOT_SIZE") || (fields.order_type == OrderType::Limit && !has("PRICE_FILTER")) {
            unresolved.push("REQUIRED_STATIC_RULE_COVERAGE_UNAVAILABLE".into());
        }
        let supported_form = match (
            fields.order_type,
            fields.quantity.r#type,
            fields.time_in_force,
        ) {
            (
                OrderType::Limit,
                OrderQuantityType::Base,
                TimeInForce::Gtc | TimeInForce::Ioc | TimeInForce::Fok,
            ) => e.order_types.iter().any(|t| t == "LIMIT"),
            (OrderType::Market, OrderQuantityType::Base, TimeInForce::Day) => {
                e.order_types.iter().any(|t| t == "MARKET")
            }
            (OrderType::Market, OrderQuantityType::Quote, TimeInForce::Day) => {
                fields.side == OrderSide::Buy
                    && e.quote_order_qty_market_allowed
                    && e.order_types.iter().any(|t| t == "MARKET")
            }
            _ => false,
        };
        let form_outcome = if !supported_form {
            RiskCheckOutcome::Reject
        } else if e.account_requires_self_trade_prevention.is_none() {
            unresolved.push("ACCOUNT_SELF_TRADE_PREVENTION_REQUIREMENT_UNOBSERVED".into());
            RiskCheckOutcome::Unavailable
        } else if e.account_requires_self_trade_prevention == Some(true)
            && e.default_self_trade_prevention_mode == "NONE"
        {
            RiskCheckOutcome::Reject
        } else {
            RiskCheckOutcome::Pass
        };
        rules.push(SpotRuleEvaluation {
            scope: SpotRuleScope::Symbol,
            origin: SpotRuleOrigin::ExchangeInfo,
            rule_type: "ORDER_FORM".into(),
            applicable: true,
            reason_code: match form_outcome {
                RiskCheckOutcome::Pass => "ORDINARY_ORDER_FORM_SUPPORTED",
                RiskCheckOutcome::Reject => "ORDER_FORM_UNSUPPORTED",
                RiskCheckOutcome::Unavailable => {
                    "ACCOUNT_SELF_TRADE_PREVENTION_REQUIREMENT_UNOBSERVED"
                }
            }
            .into(),
            outcome: form_outcome,
            unit: None,
        });
        if !e.admission_blockers.is_empty() {
            rules.push(SpotRuleEvaluation {
                scope: SpotRuleScope::Symbol,
                origin: SpotRuleOrigin::ExchangeInfo,
                rule_type: "ADMISSION".into(),
                applicable: true,
                outcome: RiskCheckOutcome::Reject,
                reason_code: "SPOT_ADMISSION_REJECTED".into(),
                unit: None,
            });
        }
        for rule in &e.constraints {
            if rule.rule_type == "PRICE_RANGE"
                && rule.known_schema
                && rule.unsupported_fields.is_empty()
            {
                if let Some(preview) = &price_range_preview {
                    rules.push(price_range_row(preview, &mut unresolved));
                }
                continue;
            }
            let evaluated = if rule.known_schema && rule.unsupported_fields.is_empty() {
                let reference = references
                    .iter()
                    .copied()
                    .find(|r| rule_reference(rule, Some(r)).is_ok())
                    .or_else(|| references.first().copied());
                static_result(rule, fields, reference)
            } else {
                Err(TradeXError::new("UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA"))
            };
            let (applicable, outcome, reason) = match evaluated {
                Ok(None) => (false, RiskCheckOutcome::Pass, "NOT_APPLICABLE".to_owned()),
                Ok(Some(true)) => (
                    true,
                    RiskCheckOutcome::Pass,
                    "STATIC_RULE_SATISFIED".to_owned(),
                ),
                Ok(Some(false)) => (
                    true,
                    RiskCheckOutcome::Reject,
                    "ORDER_FILTER_REJECTED".to_owned(),
                ),
                Err(error) => {
                    unresolved.push(format!(
                        "{:?}:{:?}:{}",
                        rule.origin, rule.scope, rule.rule_type
                    ));
                    (true, RiskCheckOutcome::Unavailable, error.code)
                }
            };
            rules.push(SpotRuleEvaluation {
                scope: rule.scope,
                origin: rule.origin,
                rule_type: rule.rule_type.clone(),
                applicable,
                outcome,
                reason_code: reason.into(),
                unit: match rule.rule_type.as_str() {
                    "LOT_SIZE" | "MARKET_LOT_SIZE" => Some(base_asset.into()),
                    "PRICE_FILTER" | "PERCENT_PRICE" | "PERCENT_PRICE_BY_SIDE" => {
                        Some(format!("USDT/{base_asset}"))
                    }
                    "MIN_NOTIONAL" | "NOTIONAL" => Some("USDT".into()),
                    "MAX_ASSET" => match field(rule, "asset") {
                        Ok(SpotRuleValue::Asset(asset)) => Some(asset.clone()),
                        _ => None,
                    },
                    _ => None,
                },
            });
        }
    } else {
        unresolved.push("CURRENT_EXACT_ACCOUNT_RULES_UNAVAILABLE".into());
    }
    let outcome = if rules.iter().any(|r| r.outcome == RiskCheckOutcome::Reject) {
        RiskCheckOutcome::Reject
    } else if !unresolved.is_empty() {
        RiskCheckOutcome::Unavailable
    } else {
        RiskCheckOutcome::Pass
    };
    let mut result = SpotProposalRules {
        workspace_id: input.workspace_id.clone(),
        proposal_id: proposal.proposal_id,
        proposal_hash: proposal.proposal_hash,
        account_id: account.connection_id,
        instrument_id: fields.instrument_id.clone(),
        base_asset: base_asset.into(),
        quote_asset: "USDT".into(),
        source_version: source.state_version,
        material_version: evidence.map(|e| e.material_version.clone()),
        observed_at: evidence.map(|e| e.observed_at.clone()),
        outcome,
        rules,
        unresolved_obligations: unresolved,
        references: references.iter().copied().cloned().collect(),
        reference_failure: slot.and_then(|s| {
            s.failure.clone().or_else(|| {
                (!s.references.is_empty() && references.len() != s.references.len())
                    .then(|| "REFERENCE_PRICE_STALE".into())
            })
        }),
        reference_purposes: evidence
            .into_iter()
            .flat_map(|e| e.constraints.iter())
            .filter(|r| requires_reference(r, fields))
            .map(|r| format!("{:?}:{:?}:{}", r.origin, r.scope, r.rule_type))
            .collect(),
        price_range_preview,
        binding_version,
        state_version: String::new(),
    };
    result.state_version = format!(
        "sha256:{}",
        hex::encode(Sha256::digest(
            serde_json::to_vec(&json!([
                result,
                account.state_version,
                control.session,
                control.time.generation()
            ]))
            .map_err(|_| unavailable())?
        ))
    );
    Ok(result)
}

fn digest(value: &Value) -> Result<String> {
    Ok(format!(
        "sha256:{}",
        hex::encode(Sha256::digest(
            serde_json::to_vec(value).map_err(|_| unavailable())?
        ))
    ))
}
/// The single derived freshness window shared by the current preview and the
/// observation that feeds it. The 30-second cap is a documented ceiling; a stricter
/// policy threshold always narrows it.
fn reference_age(policy_threshold_seconds: Option<u64>) -> u64 {
    policy_threshold_seconds
        .unwrap_or(crate::risk::DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS)
        .min(30)
}
/// Provider-original unix milliseconds as the exact RFC3339 instant that is both
/// reported and aged. A replacement for the local receipt time, never a proxy for it.
fn provider_timestamp(timestamp: i64) -> Result<String> {
    time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(timestamp) * 1_000_000)
        .map_err(|_| unavailable())?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| unavailable())
}
fn valid_age(provider: &str, now: &TimeStatus, maximum_age: u64) -> Result<()> {
    if now.confidence != TimeConfidence::Trusted {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    let parse = |s: &str| {
        time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
            .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))
    };
    let age = parse(&now.wall_clock)? - parse(provider)?;
    if age < time::Duration::ZERO {
        return Err(TradeXError::new("REFERENCE_PRICE_FUTURE"));
    }
    if age > time::Duration::seconds(maximum_age as i64) {
        return Err(TradeXError::new("REFERENCE_PRICE_STALE"));
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderReference {
    symbol: String,
    reference_price: Value,
    timestamp: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AveragePrice {
    mins: u64,
    price: String,
    close_time: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecentTrade {
    id: u64,
    price: String,
    qty: String,
    quote_qty: String,
    time: i64,
    is_buyer_maker: bool,
    is_best_match: bool,
}
fn public_reference_body(http: &impl ProviderHttp, path: &str) -> Result<Vec<u8>> {
    let (response, limit) = http.request_with_rate_limit(
        ProviderEndpoint::BinanceLive,
        ProviderHttpMethod::Get,
        path,
        reqwest::header::HeaderMap::new(),
        None,
    )?;
    match response.status {
        200 => (),
        418 | 429 => {
            crate::provider_io::record_provider_retry_after(
                "binance",
                None,
                limit.and_then(|l| l.retry_after_seconds),
            );
            return Err(TradeXError::new(if response.status == 418 {
                "PROVIDER_IP_BANNED"
            } else {
                "PROVIDER_RATE_LIMITED"
            }));
        }
        403 | 451 => return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED")),
        _ => return Err(TradeXError::new("REFERENCE_PRICE_UNAVAILABLE")),
    }
    if response.body.len() > 512 * 1024 {
        return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
    }
    Ok(response.body)
}
pub(crate) fn execute_refresh(
    control: &Arc<Mutex<ControlPlane>>,
    request: &Value,
    consumer: &str,
    http: &impl ProviderHttp,
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    let prepare = (|| {
        if !crate::provider_order_consumer_allowed(consumer) {
            return Err(TradeXError::new("IPC_ACCESS_DENIED"));
        }
        let envelope: CommandEnvelope = crate::payload(request.clone())?;
        if envelope.schema_version != 1 {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        if envelope.command != "trade.spot_rules.refresh"
            || !crate::valid_bounded_text(&envelope.request_id, 128)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let input: SpotRulesRefresh = crate::payload(envelope.payload)?;
        let mut cp = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        let query = SpotRulesQuery {
            workspace_id: input.workspace_id,
            proposal_id: input.proposal_id,
        };
        let before = get(&mut cp, &query)?;
        if before.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if before.material_version.is_none() {
            return Err(unavailable());
        }
        if before.reference_purposes.is_empty() {
            return Err(TradeXError::new("REFERENCE_PRICE_NOT_REQUIRED"));
        }
        let proposal = cp
            .store
            .as_ref()
            .ok_or_else(unavailable)?
            .order_proposal(&query.proposal_id)?;
        let maximum_reference_age = reference_age(
            cp.store
                .as_ref()
                .ok_or_else(unavailable)?
                .risk_or_new()?
                .as_ref()
                .map(|p| p.policy.stale_quote_threshold_seconds),
        );
        let source = crate::financial_sources::connection(
            &mut cp,
            &query.workspace_id,
            FinancialSourceKind::BinanceSpotRules,
        )?;
        let (needs_last_trade, needs_average, needs_execution, shared_reference) =
            match &source.evidence {
                Some(FinancialSourceEvidence::BinanceSpotRules(e)) => {
                    let mut last = false;
                    let mut average = false;
                    for rule in e
                        .constraints
                        .iter()
                        .filter(|r| requires_reference(r, &proposal.fields))
                    {
                        if let Ok(SpotRuleValue::Integer(minutes)) = field(rule, "avgPriceMins") {
                            if minutes == "0" {
                                last = true;
                            } else {
                                average = true;
                            }
                        }
                    }
                    let execution = e
                        .constraints
                        .iter()
                        .any(|r| execution_reference_required(r, &proposal.fields));
                    // The genuine referencePrice response is read once; it is reported as a
                    // shared static reference only for the purposes that actually need one.
                    let shared = e.constraints.iter().any(|r| {
                        r.rule_type != "PRICE_RANGE" && requires_reference(r, &proposal.fields)
                    });
                    (last, average, execution, shared)
                }
                _ => (false, false, false, false),
            };
        cp.spot_rule_runtime.sequence = cp
            .spot_rule_runtime
            .sequence
            .checked_add(1)
            .filter(|v| *v <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        let sequence = cp.spot_rule_runtime.sequence;
        if cp.spot_rule_runtime.slots.len() >= 8
            && !cp.spot_rule_runtime.slots.contains_key(&query.proposal_id)
        {
            if let Some(oldest) = cp
                .spot_rule_runtime
                .slots
                .iter()
                .min_by_key(|(_, s)| s.sequence)
                .map(|(id, _)| id.clone())
            {
                cp.spot_rule_runtime.slots.remove(&oldest);
            }
        }
        let previous = cp
            .spot_rule_runtime
            .slots
            .remove(&query.proposal_id)
            .filter(|s| s.binding == before.binding_version)
            .map(|s| s.references)
            .unwrap_or_default();
        cp.spot_rule_runtime.slots.insert(
            query.proposal_id.clone(),
            Slot {
                sequence,
                binding: before.binding_version.clone(),
                references: previous,
                execution: None,
                failure: None,
            },
        );
        Ok((
            query,
            before.binding_version,
            sequence,
            format!("{}USDT", before.base_asset),
            needs_last_trade,
            needs_average,
            needs_execution,
            shared_reference,
            maximum_reference_age,
        ))
    })();
    let (
        query,
        binding,
        sequence,
        symbol,
        needs_last_trade,
        needs_average,
        needs_execution,
        shared_reference,
        maximum_reference_age,
    ) = match prepare {
        Ok(v) => v,
        Err(e) => return crate::failure_reply(crate::request_id(request), e),
    };
    let binding_current = || {
        control.lock().is_ok_and(|mut cp| {
            cp.spot_rule_runtime
                .slots
                .get(&query.proposal_id)
                .is_some_and(|s| s.sequence == sequence && s.binding == binding)
                && get(&mut cp, &query).is_ok_and(|r| r.binding_version == binding)
        })
    };
    let current = || Instant::now() < deadline && binding_current();
    let read = (|| {
        crate::provider_io::binance_read_budget::reserve(None, 2)?;
        let http =
            crate::provider_io::p3_provider_http(http, &current, "spot-rules-public-reference");
        let body =
            public_reference_body(&http, &format!("/api/v3/referencePrice?symbol={symbol}"))?;
        let received = Instant::now();
        let now = control
            .lock()
            .map_err(|_| unavailable())?
            .time
            .status(&query.workspace_id)?;
        let raw: ProviderReference = crate::provider_json::strict_json(&body)?;
        if raw.symbol != symbol || raw.timestamp <= 0 {
            return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
        }
        let capture = |kind: &str,
                       original: String,
                       timestamp: i64,
                       interval_minutes: Option<u64>,
                       received: Instant,
                       now: TimeStatus| {
            if original.len() > 64 || timestamp <= 0 {
                return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
            }
            let price = crate::provider_io::decimal(&json!(original))?;
            if cmp(&price, "0")? != Ordering::Greater {
                return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
            }
            let provider = provider_timestamp(timestamp)?;
            valid_age(&provider, &now, maximum_reference_age)?;
            let reference = SpotRuleReference {
                kind: kind.into(),
                interval_minutes,
                digest: digest(&json!([
                    binding,
                    symbol,
                    kind,
                    price,
                    provider,
                    interval_minutes
                ]))?,
                price: Some(price),
                provider_observed_at: provider,
                received_at: now.wall_clock,
            };
            Ok((reference, received))
        };
        // An explicit null referencePrice is a documented, non-error observation for the
        // execution purpose: the rule is simply not enforced.
        let capture_absent = |kind: &str, timestamp: i64, received: Instant, now: TimeStatus| {
            if timestamp <= 0 {
                return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
            }
            let provider = provider_timestamp(timestamp)?;
            valid_age(&provider, &now, maximum_reference_age)?;
            Ok((
                SpotRuleReference {
                    kind: kind.into(),
                    interval_minutes: None,
                    digest: digest(&json!([binding, symbol, kind, Value::Null, provider]))?,
                    price: None,
                    provider_observed_at: provider,
                    received_at: now.wall_clock,
                },
                received,
            ))
        };
        if !raw.reference_price.is_null() {
            let price = raw
                .reference_price
                .as_str()
                .ok_or_else(|| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?
                .to_owned();
            let mut references = Vec::new();
            if !needs_execution || shared_reference {
                references.push(capture(
                    "PROVIDER_REFERENCE",
                    price.clone(),
                    raw.timestamp,
                    None,
                    received,
                    now.clone(),
                )?);
            }
            let execution = if needs_execution {
                Some(capture(
                    EXECUTION_REFERENCE_KIND,
                    price,
                    raw.timestamp,
                    None,
                    received,
                    now,
                )?)
            } else {
                None
            };
            return Ok((references, execution));
        }
        let execution = if needs_execution {
            Some(capture_absent(
                EXECUTION_REFERENCE_KIND,
                raw.timestamp,
                received,
                now,
            )?)
        } else {
            None
        };
        let mut references = Vec::new();
        for last_trade in [true, false] {
            if (last_trade && !needs_last_trade) || (!last_trade && !needs_average) {
                continue;
            }
            crate::provider_io::binance_read_budget::reserve(
                None,
                if last_trade { 25 } else { 2 },
            )?;
            let path = if last_trade {
                format!("/api/v3/trades?symbol={symbol}&limit=1")
            } else {
                format!("/api/v3/avgPrice?symbol={symbol}")
            };
            let body = public_reference_body(&http, &path)?;
            let received = Instant::now();
            let now = control
                .lock()
                .map_err(|_| unavailable())?
                .time
                .status(&query.workspace_id)?;
            if last_trade {
                let mut trades: Vec<RecentTrade> = crate::provider_json::strict_json(&body)?;
                if trades.len() != 1 {
                    return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
                }
                let trade = trades.remove(0);
                let _ = (trade.id, trade.is_buyer_maker, trade.is_best_match);
                crate::provider_io::decimal(&json!(trade.qty))?;
                crate::provider_io::decimal(&json!(trade.quote_qty))?;
                references.push(capture(
                    "LAST_TRADE",
                    trade.price,
                    trade.time,
                    Some(0),
                    received,
                    now,
                )?);
            } else {
                let average: AveragePrice = crate::provider_json::strict_json(&body)?;
                if average.mins == 0 || average.mins > 1440 {
                    return Err(TradeXError::new("REFERENCE_PRICE_INTERVAL_UNAVAILABLE"));
                }
                references.push(capture(
                    "AVERAGE_PRICE",
                    average.price,
                    average.close_time,
                    Some(average.mins),
                    received,
                    now,
                )?);
            }
        }
        if references.is_empty() && execution.is_none() {
            return Err(TradeXError::new("REFERENCE_PRICE_PURPOSE_UNAVAILABLE"));
        }
        Ok((references, execution))
    })();
    let finish = (|| {
        if !binding_current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let mut cp = control.lock().map_err(|_| unavailable())?;
        if get(&mut cp, &query)?.binding_version != binding {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let slot = cp
            .spot_rule_runtime
            .slots
            .get_mut(&query.proposal_id)
            .filter(|s| s.sequence == sequence)
            .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?;
        let read = if Instant::now() >= deadline {
            Err(TradeXError::new("PROVIDER_UNAVAILABLE"))
        } else {
            read
        };
        match read {
            Ok((references, execution)) => {
                let previous = std::mem::take(&mut slot.references);
                slot.references = references
                    .into_iter()
                    .map(|new| {
                        previous
                            .iter()
                            .find(|(old, _)| old.digest == new.0.digest)
                            .cloned()
                            .unwrap_or(new)
                    })
                    .collect();
                let previous = slot.execution.take();
                slot.execution = execution.map(|new| {
                    previous
                        .filter(|(old, _)| old.digest == new.0.digest)
                        .unwrap_or(new)
                });
                slot.failure = None;
            }
            Err(e) => {
                slot.references.clear();
                slot.execution = None;
                slot.failure = Some(e.code);
            }
        }
        get(&mut cp, &query)
    })();
    match finish {
        Ok(v) => crate::success_reply(crate::request_id(request), json!(v), None),
        Err(e) => crate::failure_reply(crate::request_id(request), e),
    }
}
