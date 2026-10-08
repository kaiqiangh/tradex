//! Bounded ordinary Spot metadata; this collector never qualifies a financial intent.
use super::*;
use crate::protocol::{
    BinanceSpotOrderFormFlags, BinanceSpotRuleEvidence, BinanceSpotSymbolStatus,
    SpotRuleConstraint, SpotRuleField, SpotRuleOrigin, SpotRuleScope, SpotRuleValue,
};
use crate::provider_io::{ProviderHttpMethod, contains_secret};
use serde_json::value::RawValue;
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    sync::{Mutex, OnceLock},
};
use zeroize::Zeroizing;

#[derive(Default)]
struct ReadBudget {
    calls: VecDeque<(Instant, String, u32)>,
}
static READ_BUDGET: OnceLock<Mutex<ReadBudget>> = OnceLock::new();
fn charge(account: &str, weight: u32) -> Result<()> {
    let now = Instant::now();
    let mut b = READ_BUDGET
        .get_or_init(|| Mutex::new(ReadBudget::default()))
        .lock()
        .map_err(|_| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
    while b
        .calls
        .front()
        .is_some_and(|(at, _, _)| now.duration_since(*at) >= StdDuration::from_secs(60))
    {
        b.calls.pop_front();
    }
    let total: u32 = b.calls.iter().map(|(_, _, w)| w).sum();
    let own: u32 = b
        .calls
        .iter()
        .filter(|(_, a, _)| a == account)
        .map(|(_, _, w)| w)
        .sum();
    // Reserve ordinary-host P3 metadata headroom rather than assuming the whole
    // provider/IP allowance is ours. Account/source/workspace changes cannot reset it.
    if total.saturating_add(weight) > 3000 || own.saturating_add(weight) > 1500 {
        return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
    }
    b.calls.push_back((now, account.into(), weight));
    Ok(())
}
fn token(s: &str, max: usize) -> Result<String> {
    if s.is_empty()
        || s.len() > max
        || !s
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
    {
        return Err(invalid());
    }
    Ok(s.into())
}
fn tokens(v: Vec<String>, max: usize) -> Result<Vec<String>> {
    if v.len() > max {
        return Err(invalid());
    }
    let mut seen = HashSet::new();
    for s in &v {
        token(s, 32)?;
        if !seen.insert(s) {
            return Err(invalid());
        }
    }
    Ok(v)
}
fn response_body(
    response: crate::provider_io::ProviderHttpResponse,
    secrets: &[String],
    total: &mut usize,
) -> Result<Vec<u8>> {
    match response.status {
        200 => (),
        401 => return Err(TradeXError::new("PROVIDER_AUTH_FAILED")),
        403 | 451 => return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED")),
        418 | 429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
        _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
    }
    *total = total.checked_add(response.body.len()).ok_or_else(invalid)?;
    if response.body.len() > 512 * 1024 || *total > 4 * 1024 * 1024 {
        return Err(invalid());
    }
    let parsed: Value = strict_json(&response.body)?;
    if contains_secret(&parsed, secrets) {
        return Err(invalid());
    }
    Ok(response.body)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Clock {
    server_time: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Account {
    uid: u64,
    account_type: String,
    can_trade: bool,
    permissions: Vec<String>,
    require_self_trade_prevention: Option<bool>,
}
#[derive(Deserialize)]
struct System {
    status: u64,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Trading {
    is_locked: bool,
    update_time: u64,
    planned_recover_time: u64,
}
#[derive(Deserialize, Serialize)]
struct TradingEnvelope {
    data: Trading,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Symbol {
    symbol: String,
    status: String,
    base_asset: String,
    quote_asset: String,
    base_asset_precision: u32,
    quote_asset_precision: u32,
    is_spot_trading_allowed: bool,
    quote_order_qty_market_allowed: bool,
    order_types: Vec<String>,
    iceberg_allowed: Option<bool>,
    oco_allowed: Option<bool>,
    oto_allowed: Option<bool>,
    opo_allowed: Option<bool>,
    allow_trailing_stop: Option<bool>,
    cancel_replace_allowed: Option<bool>,
    amend_allowed: Option<bool>,
    peg_instructions_allowed: Option<bool>,
    permission_sets: Vec<Vec<String>>,
    default_self_trade_prevention_mode: String,
    allowed_self_trade_prevention_modes: Vec<String>,
    filters: Vec<Box<RawValue>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Exchange {
    exchange_filters: Vec<Box<RawValue>>,
    symbols: Vec<Symbol>,
}
#[derive(Deserialize)]
struct ExecutionSymbol {
    symbol: String,
    rules: Vec<Box<RawValue>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Execution {
    symbol_rules: Vec<ExecutionSymbol>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountFilters {
    exchange_filters: Vec<Box<RawValue>>,
    symbol_filters: Vec<Box<RawValue>>,
    asset_filters: Vec<Box<RawValue>>,
}

// D=exact decimal string, I=original integer, B=original boolean, A=asset token.
fn fields_for(kind: &str) -> Option<&'static [(&'static str, char)]> {
    Some(match kind {
        "PRICE_FILTER" => &[("minPrice", 'D'), ("maxPrice", 'D'), ("tickSize", 'D')],
        "LOT_SIZE" | "MARKET_LOT_SIZE" => &[("minQty", 'D'), ("maxQty", 'D'), ("stepSize", 'D')],
        "PERCENT_PRICE" => &[
            ("multiplierUp", 'D'),
            ("multiplierDown", 'D'),
            ("avgPriceMins", 'I'),
        ],
        "PERCENT_PRICE_BY_SIDE" => &[
            ("bidMultiplierUp", 'D'),
            ("bidMultiplierDown", 'D'),
            ("askMultiplierUp", 'D'),
            ("askMultiplierDown", 'D'),
            ("avgPriceMins", 'I'),
        ],
        "MIN_NOTIONAL" => &[
            ("minNotional", 'D'),
            ("applyToMarket", 'B'),
            ("avgPriceMins", 'I'),
        ],
        "NOTIONAL" => &[
            ("minNotional", 'D'),
            ("maxNotional", 'D'),
            ("applyMinToMarket", 'B'),
            ("applyMaxToMarket", 'B'),
            ("avgPriceMins", 'I'),
        ],
        "MAX_NUM_ORDERS" | "EXCHANGE_MAX_NUM_ORDERS" => &[("maxNumOrders", 'I')],
        "MAX_NUM_ALGO_ORDERS" | "EXCHANGE_MAX_NUM_ALGO_ORDERS" => &[("maxNumAlgoOrders", 'I')],
        "MAX_NUM_ICEBERG_ORDERS" | "EXCHANGE_MAX_NUM_ICEBERG_ORDERS" => {
            &[("maxNumIcebergOrders", 'I')]
        }
        "MAX_NUM_ORDER_LISTS" | "EXCHANGE_MAX_NUM_ORDER_LISTS" => &[("maxNumOrderLists", 'I')],
        "MAX_NUM_ORDER_AMENDS" => &[("maxNumOrderAmends", 'I')],
        "ICEBERG_PARTS" => &[("limit", 'I')],
        "MAX_POSITION" => &[("maxPosition", 'D')],
        "MAX_ASSET" => &[("asset", 'A'), ("limit", 'D')],
        "TRAILING_DELTA" => &[
            ("minTrailingAboveDelta", 'I'),
            ("maxTrailingAboveDelta", 'I'),
            ("minTrailingBelowDelta", 'I'),
            ("maxTrailingBelowDelta", 'I'),
        ],
        "PRICE_RANGE" => &[
            ("bidLimitMultUp", 'D'),
            ("bidLimitMultDown", 'D'),
            ("askLimitMultUp", 'D'),
            ("askLimitMultDown", 'D'),
        ],
        _ => return None,
    })
}
fn constraints(
    raw: Vec<Box<RawValue>>,
    scope: SpotRuleScope,
    origin: SpotRuleOrigin,
) -> Result<Vec<SpotRuleConstraint>> {
    if raw.len() > 64 {
        return Err(invalid());
    }
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for raw in raw {
        let mut map: BTreeMap<String, Box<RawValue>> = strict_json(raw.get().as_bytes())?;
        if map.len() > 32 {
            return Err(invalid());
        }
        let tag = if scope == SpotRuleScope::Execution {
            "ruleType"
        } else {
            "filterType"
        };
        let kind: String = serde_json::from_str(map.remove(tag).ok_or_else(invalid)?.get())
            .map_err(|_| invalid())?;
        token(&kind, 64)?;
        let schema = fields_for(&kind);
        if schema.is_some() {
            let expected_scope = if kind == "PRICE_RANGE" {
                SpotRuleScope::Execution
            } else if kind == "MAX_ASSET" {
                SpotRuleScope::Asset
            } else if kind.starts_with("EXCHANGE_") {
                SpotRuleScope::Exchange
            } else {
                SpotRuleScope::Symbol
            };
            if scope != expected_scope {
                return Err(invalid());
            }
        }
        let mut fields = Vec::new();
        if let Some(schema) = schema {
            for &(name, ty) in schema {
                let raw = map.remove(name).ok_or_else(invalid)?;
                let value = match ty {
                    'D' => {
                        let s: String = serde_json::from_str(raw.get()).map_err(|_| invalid())?;
                        if s.len() > 64 || s.starts_with('-') {
                            return Err(invalid());
                        }
                        let s = crate::provider_io::decimal(&Value::String(s))?;
                        if s.len() > 64 {
                            return Err(invalid());
                        }
                        SpotRuleValue::Decimal(s)
                    }
                    'I' => {
                        let n: i64 = serde_json::from_str(raw.get()).map_err(|_| invalid())?;
                        if n < 0 {
                            return Err(invalid());
                        }
                        SpotRuleValue::Integer(n.to_string())
                    }
                    'B' => SpotRuleValue::Boolean(
                        serde_json::from_str(raw.get()).map_err(|_| invalid())?,
                    ),
                    'A' => {
                        let s: String = serde_json::from_str(raw.get()).map_err(|_| invalid())?;
                        SpotRuleValue::Asset(token(&s, 16)?)
                    }
                    _ => unreachable!(),
                };
                let disabled =
                    kind == "PRICE_FILTER" && matches!(&value,SpotRuleValue::Decimal(v) if v=="0");
                fields.push(SpotRuleField {
                    name: name.into(),
                    value,
                    disabled,
                });
            }
        }
        // Only these field/kind pairs are documented by the current REST schema.
        // Other exponent fields stay explicit unsupported obligations.
        let exponent = match kind.as_str() {
            "PRICE_FILTER" => Some("priceExponent"),
            "MAX_ASSET" => Some("qtyExponent"),
            _ => None,
        };
        if let Some(name) = exponent {
            if let Some(raw) = map.remove(name) {
                let n: u32 = serde_json::from_str(raw.get()).map_err(|_| invalid())?;
                if n > 64 {
                    return Err(invalid());
                }
                fields.push(SpotRuleField {
                    name: name.into(),
                    value: SpotRuleValue::Integer(n.to_string()),
                    disabled: false,
                });
            }
        }
        for (minimum, maximum) in [
            ("minPrice", "maxPrice"),
            ("minQty", "maxQty"),
            ("minNotional", "maxNotional"),
            ("multiplierDown", "multiplierUp"),
            ("bidMultiplierDown", "bidMultiplierUp"),
            ("askMultiplierDown", "askMultiplierUp"),
            ("bidLimitMultDown", "bidLimitMultUp"),
            ("askLimitMultDown", "askLimitMultUp"),
            ("minTrailingAboveDelta", "maxTrailingAboveDelta"),
            ("minTrailingBelowDelta", "maxTrailingBelowDelta"),
        ] {
            let numeric = |name: &str| {
                fields
                    .iter()
                    .find(|f| f.name == name && !f.disabled)
                    .and_then(|f| match &f.value {
                        SpotRuleValue::Decimal(v) | SpotRuleValue::Integer(v) => Some(v.as_str()),
                        _ => None,
                    })
            };
            if let (Some(min), Some(max)) = (numeric(minimum), numeric(maximum)) {
                if crate::provider_io::decimal_cmp(min, max)? == std::cmp::Ordering::Greater {
                    return Err(invalid());
                }
            }
        }
        let unsupported_fields = map.keys().cloned().collect::<Vec<_>>();
        if unsupported_fields
            .iter()
            .any(|name| !crate::valid_bounded_text(name, 64))
        {
            return Err(invalid());
        }
        let asset = fields
            .iter()
            .find_map(|f| {
                if let SpotRuleValue::Asset(a) = &f.value {
                    Some(a.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("");
        if !seen.insert((kind.clone(), asset.to_owned())) {
            return Err(invalid());
        }
        out.push(SpotRuleConstraint {
            scope,
            origin,
            rule_type: kind,
            known_schema: schema.is_some() && unsupported_fields.is_empty(),
            fields,
            unsupported_fields,
        });
    }
    Ok(out)
}

pub(super) fn read(
    control: &Arc<Mutex<ControlPlane>>,
    binding: &Binding,
    account: &AccountConnection,
    credentials: &crate::provider_io::Credentials,
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    deadline: Instant,
) -> Result<Observation> {
    let secrets = Zeroizing::new(credentials.values()?);
    if secrets.len() != 2 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let instrument_id = binding.instrument_id.as_deref().ok_or_else(invalid)?;
    let (symbol, base) = match instrument_id {
        "crypto:BTC/USDT:spot" => ("BTCUSDT", "BTC"),
        "crypto:ETH/USDT:spot" => ("ETHUSDT", "ETH"),
        _ => return Err(invalid()),
    };
    let expected = account
        .data
        .as_ref()
        .ok_or_else(invalid)?
        .remote_account_id
        .clone();
    let http_current = || current() && http_budget_remains(deadline);
    let http = crate::provider_io::p3_provider_http(http, &http_current, &binding.reference);
    let mut total = 0usize;
    let public = |path: &str, weight: u32, total: &mut usize| -> Result<Vec<u8>> {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        charge(&expected, weight)?;
        let (r, rate) = http.request_with_rate_limit(
            ProviderEndpoint::BinanceLive,
            ProviderHttpMethod::Get,
            path,
            reqwest::header::HeaderMap::new(),
            None,
        )?;
        if r.status == 418 {
            crate::provider_io::record_provider_retry_after(
                "binance",
                None,
                rate.and_then(|r| r.retry_after_seconds),
            );
        }
        let b = response_body(r, &secrets, total)?;
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        Ok(b)
    };
    let started = Instant::now();
    let clock: Clock = strict_json(&public("/api/v3/time", 1, &mut total)?)?;
    if started.elapsed() > StdDuration::from_secs(2) {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    let sampled = Instant::now();
    let received = control
        .lock()
        .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
        .time
        .status(&binding.workspace)?;
    let local = timestamp(&received.wall_clock)?.unix_timestamp_nanos() / 1_000_000;
    if (i128::from(clock.server_time) - local).abs()
        > i128::from(crate::time::MAX_PROVIDER_OFFSET_MS)
    {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    let signed =
        |route: &str, params: &[(&str, &str)], weight: u32, total: &mut usize| -> Result<Vec<u8>> {
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            charge(&expected, weight)?;
            let r = crate::provider_io::binance::signed_request_for(
                ProviderEndpoint::BinanceLive,
                &http,
                ProviderHttpMethod::Get,
                route,
                params,
                &secrets,
                clock.server_time,
                sampled,
                current,
            )?;
            if r.status == 418 {
                crate::provider_io::record_provider_retry_after("binance", None, None);
            }
            let b = response_body(r, &secrets, total)?;
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            Ok(b)
        };
    let a: Account = strict_json(&signed("/api/v3/account", &[], 20, &mut total)?)?;
    if a.uid == 0 || a.uid > i64::MAX as u64 || a.uid.to_string() != expected {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let account_permissions = tokens(a.permissions, 64)?;
    let restrictions = signed("/sapi/v1/account/apiRestrictions", &[], 1, &mut total)?;
    let original: BTreeMap<String, Box<RawValue>> = strict_json(&restrictions)?;
    for (name, value) in &original {
        if name.starts_with("enable") || name.starts_with("permits") || name == "ipRestrict" {
            serde_json::from_str::<bool>(value.get()).map_err(|_| invalid())?;
        }
    }
    let key_permissions =
        crate::provider_io::binance::permissions(Some(strict_json(&restrictions)?))?;
    let system: System = strict_json(&public("/sapi/v1/system/status", 1, &mut total)?)?;
    let system_received = control
        .lock()
        .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
        .time
        .status(&binding.workspace)?;
    let trading: TradingEnvelope = strict_json(&signed(
        "/sapi/v1/account/apiTradingStatus",
        &[],
        1,
        &mut total,
    )?)?;
    let upper = clock
        .server_time
        .checked_add(sampled.elapsed().as_millis() as u64)
        .ok_or_else(invalid)?;
    let trading_status = crate::provider_io::binance::trading_status(
        &json!({"status":system.status}),
        &serde_json::to_value(trading).map_err(|_| invalid())?,
        upper,
        system_received.wall_clock,
    )?;
    let mut exchange: Exchange = strict_json(&public(
        &format!("/api/v3/exchangeInfo?symbol={symbol}&showPermissionSets=true"),
        20,
        &mut total,
    )?)?;
    if exchange.symbols.len() != 1 {
        return Err(invalid());
    }
    let s = exchange.symbols.pop().ok_or_else(invalid)?;
    if s.symbol != symbol || s.base_asset != base || s.quote_asset != "USDT" {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    if s.base_asset_precision > 64
        || s.quote_asset_precision > 64
        || s.permission_sets.is_empty()
        || s.permission_sets.len() > 64
        || s.permission_sets.iter().any(Vec::is_empty)
    {
        return Err(invalid());
    }
    let order_types = tokens(s.order_types, 32)?;
    let allowed_stp = tokens(s.allowed_self_trade_prevention_modes, 32)?;
    token(&s.default_self_trade_prevention_mode, 32)?;
    token(&s.status, 32)?;
    if !allowed_stp.contains(&s.default_self_trade_prevention_mode) {
        return Err(invalid());
    }
    let permission_sets = s
        .permission_sets
        .into_iter()
        .map(|set| tokens(set, 64))
        .collect::<Result<Vec<_>>>()?;
    let permission_sets_satisfied = permission_sets
        .iter()
        .all(|set| set.iter().any(|p| account_permissions.contains(p)));
    let symbol_status = match s.status.as_str() {
        "TRADING" => BinanceSpotSymbolStatus::Trading,
        "HALT" => BinanceSpotSymbolStatus::Halt,
        "BREAK" => BinanceSpotSymbolStatus::Break,
        _ => BinanceSpotSymbolStatus::Unknown,
    };
    let mut rules = constraints(
        exchange.exchange_filters,
        SpotRuleScope::Exchange,
        SpotRuleOrigin::ExchangeInfo,
    )?;
    rules.extend(constraints(
        s.filters,
        SpotRuleScope::Symbol,
        SpotRuleOrigin::ExchangeInfo,
    )?);
    let mut execution: Execution = strict_json(&public(
        &format!("/api/v3/executionRules?symbol={symbol}"),
        2,
        &mut total,
    )?)?;
    if execution.symbol_rules.len() != 1 || execution.symbol_rules[0].symbol != symbol {
        return Err(invalid());
    }
    rules.extend(constraints(
        execution.symbol_rules.pop().ok_or_else(invalid)?.rules,
        SpotRuleScope::Execution,
        SpotRuleOrigin::ExecutionRules,
    )?);
    let relevant: AccountFilters = strict_json(&signed(
        "/api/v3/myFilters",
        &[("symbol", symbol)],
        40,
        &mut total,
    )?)?;
    rules.extend(constraints(
        relevant.exchange_filters,
        SpotRuleScope::Exchange,
        SpotRuleOrigin::AccountFilters,
    )?);
    rules.extend(constraints(
        relevant.symbol_filters,
        SpotRuleScope::Symbol,
        SpotRuleOrigin::AccountFilters,
    )?);
    rules.extend(constraints(
        relevant.asset_filters,
        SpotRuleScope::Asset,
        SpotRuleOrigin::AccountFilters,
    )?);
    if rules.len() > 256 {
        return Err(invalid());
    }
    let mut blockers = Vec::new();
    for (blocked, code) in [
        (a.account_type != "SPOT", "ACCOUNT_NOT_SPOT"),
        (!a.can_trade, "ACCOUNT_CANNOT_TRADE"),
        (!permission_sets_satisfied, "PERMISSION_SETS_NOT_SATISFIED"),
        (!s.is_spot_trading_allowed, "SPOT_NOT_ALLOWED"),
        (
            symbol_status != BinanceSpotSymbolStatus::Trading,
            "SYMBOL_NOT_TRADING",
        ),
        (key_permissions.scope != "VERIFIED", "KEY_SCOPE_UNVERIFIED"),
        (
            !key_permissions.forbidden.is_empty() || !key_permissions.unsupported.is_empty(),
            "KEY_PERMISSION_BLOCKED",
        ),
        (
            !key_permissions
                .detected
                .iter()
                .any(|p| p == "spot-and-margin.trade"),
            "SPOT_KEY_PERMISSION_MISSING",
        ),
        (
            trading_status.system_status != crate::providers::BinanceSystemStatus::Normal,
            "SYSTEM_MAINTENANCE",
        ),
        (trading_status.api_trading_locked, "API_TRADING_LOCKED"),
    ] {
        if blocked {
            blockers.push(code.into());
        }
    }
    let mut obligations = vec![
        "PROPOSAL_SPECIFIC_RULE_VALIDATION_REQUIRED".into(),
        "QUOTE_DEPTH_AND_RIGHTS_REQUIRED".into(),
    ];
    for rule in &rules {
        let required = match rule.rule_type.as_str() {
            "PRICE_FILTER" => "PRICE_TICK_AND_BOUND_VALIDATION_REQUIRED",
            "LOT_SIZE" => "BASE_QUANTITY_GRID_VALIDATION_REQUIRED",
            "MARKET_LOT_SIZE" => "MARKET_BASE_QUANTITY_GRID_VALIDATION_REQUIRED",
            "MIN_NOTIONAL" | "NOTIONAL" => "NOTIONAL_AND_MARKET_REFERENCE_VALIDATION_REQUIRED",
            "PERCENT_PRICE" | "PERCENT_PRICE_BY_SIDE" => {
                "PROVIDER_REFERENCE_OR_WEIGHTED_AVERAGE_PRICE_REQUIRED"
            }
            "MAX_POSITION" => "CURRENT_FREE_LOCKED_AND_OPEN_BUY_POSITION_REQUIRED",
            "MAX_ASSET" => "EXACT_ORDER_ASSET_AMOUNT_VALIDATION_REQUIRED",
            "PRICE_RANGE" => "BOOK_REFERENCE_PRICE_RANGE_VALIDATION_REQUIRED",
            "TRAILING_DELTA" => "TRAILING_ORDER_FORM_VALIDATION_REQUIRED",
            "ICEBERG_PARTS" => "ICEBERG_ORDER_PART_COUNT_VALIDATION_REQUIRED",
            "MAX_NUM_ORDER_AMENDS" => "CURRENT_ORDER_AMEND_COUNT_REQUIRED",
            "MAX_NUM_ORDERS"
            | "MAX_NUM_ALGO_ORDERS"
            | "MAX_NUM_ICEBERG_ORDERS"
            | "MAX_NUM_ORDER_LISTS"
            | "EXCHANGE_MAX_NUM_ORDERS"
            | "EXCHANGE_MAX_NUM_ALGO_ORDERS"
            | "EXCHANGE_MAX_NUM_ICEBERG_ORDERS"
            | "EXCHANGE_MAX_NUM_ORDER_LISTS" => "CURRENT_ACCOUNT_EXCHANGE_ORDER_COUNTS_REQUIRED",
            _ => continue,
        };
        if !obligations.iter().any(|item| item == required) {
            obligations.push(required.into());
        }
        if rule.rule_type == "MAX_ASSET" && rule.fields.iter().any(|f| matches!(&f.value, SpotRuleValue::Asset(asset) if asset != base && asset != "USDT")) {
            if !obligations.iter().any(|item| item == "NON_CANONICAL_ASSET_CONSTRAINT_UNQUALIFIED") { obligations.push("NON_CANONICAL_ASSET_CONSTRAINT_UNQUALIFIED".into()); }
        }
    }
    if rules.iter().any(|r| !r.known_schema) {
        obligations.push("UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA".into());
    }
    if a.require_self_trade_prevention.is_none() {
        obligations.push("ACCOUNT_SELF_TRADE_PREVENTION_REQUIREMENT_UNOBSERVED".into());
    }
    let order_form_flags = BinanceSpotOrderFormFlags {
        iceberg_allowed: s.iceberg_allowed,
        oco_allowed: s.oco_allowed,
        oto_allowed: s.oto_allowed,
        opo_allowed: s.opo_allowed,
        allow_trailing_stop: s.allow_trailing_stop,
        cancel_replace_allowed: s.cancel_replace_allowed,
        amend_allowed: s.amend_allowed,
        peg_instructions_allowed: s.peg_instructions_allowed,
    };
    if [
        s.iceberg_allowed,
        s.oco_allowed,
        s.oto_allowed,
        s.opo_allowed,
        s.allow_trailing_stop,
        s.cancel_replace_allowed,
        s.amend_allowed,
        s.peg_instructions_allowed,
    ]
    .iter()
    .any(Option::is_none)
    {
        obligations.push("ORDER_FORM_FLAGS_UNOBSERVED".into());
    }
    obligations.push("ADVANCED_ORDER_FORMS_UNSUPPORTED".into());
    let provider_clock_sample =
        OffsetDateTime::from_unix_timestamp_nanos(i128::from(clock.server_time) * 1_000_000)
            .map_err(|_| invalid())?
            .format(&Rfc3339)
            .map_err(|_| invalid())?;
    let projected = binding.projection()?;
    let mut evidence = BinanceSpotRuleEvidence {
        binding: projected,
        material_version: String::new(),
        observed_at: received.wall_clock.clone(),
        provider_observed_at: None,
        provider_quality: FinancialEvidenceQuality::ReadOnlySpotRules,
        provider_clock_sample,
        remote_account_id: expected,
        instrument_id: instrument_id.into(),
        provider_symbol: symbol.into(),
        base_asset: base.into(),
        quote_asset: "USDT".into(),
        symbol_status,
        reported_symbol_status: s.status,
        base_asset_precision: s.base_asset_precision,
        quote_asset_precision: s.quote_asset_precision,
        spot_trading_allowed: s.is_spot_trading_allowed,
        quote_order_qty_market_allowed: s.quote_order_qty_market_allowed,
        order_types,
        order_form_flags,
        default_self_trade_prevention_mode: s.default_self_trade_prevention_mode,
        allowed_self_trade_prevention_modes: allowed_stp,
        account_permissions,
        permission_sets,
        permission_sets_satisfied,
        account_can_trade: a.can_trade,
        account_type: token(&a.account_type, 16)?,
        account_requires_self_trade_prevention: a.require_self_trade_prevention,
        key_permissions,
        trading_status,
        constraints: rules,
        admission_blockers: blockers,
        unresolved_obligations: obligations,
    };
    evidence.material_version = hash(&evidence)?;
    Ok(Observation {
        binding: binding.clone(),
        received,
        evidence: FinancialSourceEvidence::BinanceSpotRules(evidence),
    })
}
