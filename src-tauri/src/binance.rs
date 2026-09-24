use super::*;
use crate::protocol::{
    BinanceTestnetBalance, BinanceTestnetFill, BinanceTestnetHistoryState, BinanceTestnetOrder,
    BinanceTestnetOrderAttempt, BinanceTestnetOrderAttemptState, BinanceTestnetOrderBook,
    BinanceTestnetOrderBookAction, BinanceTestnetOrderBookStatus, BinanceTestnetOrderCancelState,
    BinanceTestnetOrderOrigin, OrderProposal, OrderSide,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    collections::{BTreeMap, HashSet},
    sync::atomic::{AtomicU16, AtomicU64, Ordering},
    time::Instant,
};

// ponytail: process-wide cooldown; use shared OS storage if multiple TradeX processes share one egress IP.
static TESTNET_IP_RETRY_AT: AtomicU64 = AtomicU64::new(0);
static TESTNET_IP_RETRY_STATUS: AtomicU16 = AtomicU16::new(0);

fn time_error() -> TradeXError {
    TradeXError::new("CLOCK_SKEW")
}

fn rate_limit_code(status: u16) -> Option<&'static str> {
    match status {
        418 => Some("PROVIDER_IP_BANNED"),
        429 => Some("PROVIDER_RATE_LIMITED"),
        _ => None,
    }
}

pub(super) fn observe_ip_rate_limit(status: u16, rate_limit: Option<&ProviderRateLimit>) {
    let Some(code) = rate_limit_code(status) else {
        return;
    };
    let fallback = if status == 418 { 120 } else { 60 };
    let seconds = rate_limit
        .and_then(|limit| limit.retry_after_seconds)
        .unwrap_or(fallback)
        .clamp(1, 259_200);
    let retry_at = time::OffsetDateTime::now_utc()
        .unix_timestamp()
        .saturating_add(seconds as i64)
        .max(0) as u64;
    let previous = TESTNET_IP_RETRY_AT.fetch_max(retry_at, Ordering::SeqCst);
    if status == 418 || retry_at >= previous {
        TESTNET_IP_RETRY_STATUS.store(
            if code == "PROVIDER_IP_BANNED" {
                418
            } else {
                429
            },
            Ordering::SeqCst,
        );
    }
}

pub(super) fn check_testnet_ip_cooldown() -> Result<()> {
    let retry_at = TESTNET_IP_RETRY_AT.load(Ordering::SeqCst);
    if retry_at > time::OffsetDateTime::now_utc().unix_timestamp().max(0) as u64 {
        let status = TESTNET_IP_RETRY_STATUS.load(Ordering::SeqCst);
        Err(TradeXError::new(if status == 418 {
            "PROVIDER_IP_BANNED"
        } else {
            "PROVIDER_RATE_LIMITED"
        }))
    } else {
        Ok(())
    }
}

fn testnet_ip_retry_at() -> Option<(String, &'static str)> {
    let retry_at = TESTNET_IP_RETRY_AT.load(Ordering::SeqCst);
    if retry_at <= time::OffsetDateTime::now_utc().unix_timestamp().max(0) as u64 {
        return None;
    }
    let status = TESTNET_IP_RETRY_STATUS.load(Ordering::SeqCst);
    let value = time::OffsetDateTime::from_unix_timestamp(retry_at as i64).ok()?;
    let formatted = value
        .format(&time::format_description::well_known::Rfc3339)
        .ok()?;
    Some((
        formatted,
        if status == 418 {
            "PROVIDER_IP_BANNED"
        } else {
            "PROVIDER_RATE_LIMITED"
        },
    ))
}

pub(super) fn allows(endpoint: ProviderEndpoint, path: &str) -> bool {
    if path == "/api/v3/time" {
        return true;
    }
    if endpoint == ProviderEndpoint::BinanceTestnet
        && matches!(
            path,
            "/api/v3/exchangeInfo?symbol=BTCUSDT"
                | "/api/v3/exchangeInfo?symbol=ETHUSDT"
                | "/api/v3/avgPrice?symbol=BTCUSDT"
                | "/api/v3/avgPrice?symbol=ETHUSDT"
                | "/api/v3/referencePrice?symbol=BTCUSDT"
                | "/api/v3/referencePrice?symbol=ETHUSDT"
                | "/api/v3/ticker/price?symbol=BTCUSDT"
                | "/api/v3/ticker/price?symbol=ETHUSDT"
        )
    {
        return true;
    }
    let Some((route, query)) = path.split_once('?') else {
        return false;
    };
    if !matches!(
        route,
        "/api/v3/account"
            | "/api/v3/openOrders"
            | "/api/v3/order"
            | "/api/v3/allOrders"
            | "/api/v3/myTrades"
    ) && !(endpoint == ProviderEndpoint::BinanceLive
        && route == "/sapi/v1/account/apiRestrictions")
    {
        return false;
    }
    let Some((unsigned, signature)) = query.split_once("&signature=") else {
        return false;
    };
    let mut values = BTreeMap::new();
    for pair in unsigned.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            return false;
        };
        if value.is_empty() || values.insert(key, value).is_some() {
            return false;
        }
    }
    let Some(timestamp) = values.get("timestamp") else {
        return false;
    };
    if values.get("recvWindow") != Some(&"5000")
        || timestamp.len() != 13
        || !timestamp.bytes().all(|b| b.is_ascii_digit())
        || !timestamp.parse::<u64>().is_ok_and(valid_time)
        || signature.len() != 64
        || !signature
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return false;
    }
    let mut keys = values.keys().copied().collect::<HashSet<_>>();
    keys.remove("timestamp");
    keys.remove("recvWindow");
    match route {
        "/api/v3/account" | "/api/v3/openOrders" => keys.is_empty(),
        "/api/v3/allOrders" => {
            endpoint == ProviderEndpoint::BinanceTestnet
                && keys
                    .iter()
                    .all(|key| matches!(*key, "symbol" | "limit" | "orderId"))
                && matches!(values.get("symbol"), Some(&"BTCUSDT" | &"ETHUSDT"))
                && values.get("limit") == Some(&"1000")
                && values.get("orderId").is_none_or(|id| valid_order_id(id))
        }
        "/api/v3/myTrades" => {
            endpoint == ProviderEndpoint::BinanceTestnet
                && keys
                    .iter()
                    .all(|key| matches!(*key, "symbol" | "limit" | "fromId"))
                && matches!(values.get("symbol"), Some(&"BTCUSDT" | &"ETHUSDT"))
                && values.get("limit") == Some(&"1000")
                && values.get("fromId").is_none_or(|id| valid_order_id(id))
        }
        "/api/v3/order" => {
            let symbol = values
                .get("symbol")
                .is_some_and(|value| matches!(*value, "BTCUSDT" | "ETHUSDT"));
            let query_order_by_client = keys
                .iter()
                .all(|key| matches!(*key, "symbol" | "origClientOrderId"))
                && keys.len() == 2
                && symbol
                && values
                    .get("origClientOrderId")
                    .is_some_and(|value| valid_client_order_id(value));
            let query_order_by_id = keys.iter().all(|key| matches!(*key, "symbol" | "orderId"))
                && keys.len() == 2
                && values
                    .get("symbol")
                    .is_some_and(|value| valid_binance_symbol(value))
                && values
                    .get("orderId")
                    .is_some_and(|value| valid_order_id(value));
            let new_order = endpoint == ProviderEndpoint::BinanceTestnet
                && keys.iter().all(|key| {
                    matches!(
                        *key,
                        "symbol"
                            | "side"
                            | "type"
                            | "timeInForce"
                            | "quantity"
                            | "quoteOrderQty"
                            | "price"
                            | "newClientOrderId"
                            | "newOrderRespType"
                    )
                })
                && symbol
                && values
                    .get("side")
                    .is_some_and(|value| matches!(*value, "BUY" | "SELL"))
                && values
                    .get("type")
                    .is_some_and(|value| matches!(*value, "MARKET" | "LIMIT"))
                && values
                    .get("newClientOrderId")
                    .is_some_and(|value| valid_client_order_id(value))
                && values.get("newOrderRespType") == Some(&"RESULT")
                && match values.get("type").copied() {
                    Some("MARKET") => {
                        !values.contains_key("price")
                            && !values.contains_key("timeInForce")
                            && (values.contains_key("quantity")
                                ^ values.contains_key("quoteOrderQty"))
                            && values
                                .get("quantity")
                                .or_else(|| values.get("quoteOrderQty"))
                                .is_some_and(|value| decimal_field(value))
                            && (!values.contains_key("quoteOrderQty")
                                || values.get("side") == Some(&"BUY"))
                    }
                    Some("LIMIT") => {
                        values
                            .get("timeInForce")
                            .is_some_and(|value| matches!(*value, "GTC" | "IOC" | "FOK"))
                            && values.contains_key("quantity")
                            && !values.contains_key("quoteOrderQty")
                            && values
                                .get("quantity")
                                .is_some_and(|value| decimal_field(value))
                            && values
                                .get("price")
                                .is_some_and(|value| decimal_field(value))
                    }
                    _ => false,
                };
            query_order_by_client || query_order_by_id || new_order
        }
        "/sapi/v1/account/apiRestrictions" => {
            endpoint == ProviderEndpoint::BinanceLive && keys.is_empty()
        }
        _ => false,
    }
}

pub(super) fn allows_cancel(path: &str) -> bool {
    let Some((route, query)) = path.split_once('?') else {
        return false;
    };
    if route != "/api/v3/order" {
        return false;
    }
    let Some((unsigned, signature)) = query.split_once("&signature=") else {
        return false;
    };
    let mut values = BTreeMap::new();
    for pair in unsigned.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            return false;
        };
        if values.insert(key, value).is_some() {
            return false;
        }
    }
    values.len() == 5
        && signature.len() == 64
        && signature
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && values
            .get("symbol")
            .is_some_and(|value| matches!(*value, "BTCUSDT" | "ETHUSDT"))
        && values
            .get("orderId")
            .is_some_and(|value| valid_order_id(value))
        && values
            .get("newClientOrderId")
            .is_some_and(|value| valid_client_order_id(value))
        && values.get("recvWindow") == Some(&"5000")
        && values.get("timestamp").is_some_and(|value| {
            value.len() == 13
                && value.bytes().all(|byte| byte.is_ascii_digit())
                && value.parse::<u64>().is_ok_and(valid_time)
        })
}

fn valid_client_order_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 36
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn valid_order_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value
            .parse::<u64>()
            .is_ok_and(|id| id > 0 && id <= i64::MAX as u64)
}

fn valid_binance_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn decimal_field(value: &str) -> bool {
    crate::provider_io::decimal(&Value::String(value.into()))
        .is_ok_and(|normalized| normalized != "0")
}

fn signed_request(
    http: &impl ProviderHttp,
    method: ProviderHttpMethod,
    route: &str,
    params: &[(&str, &str)],
    secrets: &[String],
    server_time: u64,
    sampled: Instant,
    current: &impl Fn() -> bool,
) -> Result<ProviderHttpResponse> {
    check_testnet_ip_cooldown()?;
    if sampled.elapsed() > Duration::from_secs(60) || !current() {
        return Err(time_error());
    }
    let timestamp = server_time
        .checked_add(sampled.elapsed().as_millis() as u64)
        .filter(|value| valid_time(*value))
        .ok_or_else(time_error)?;
    let mut query = params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    if !query.is_empty() {
        query.push('&');
    }
    query.push_str(&format!("timestamp={timestamp}&recvWindow=5000"));
    let signed = Zeroizing::new(signature(&query, &secrets[1])?);
    let path = Zeroizing::new(format!("{route}?{query}&signature={}", signed.as_str()));
    let mut key = HeaderValue::from_str(&secrets[0])
        .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
    key.set_sensitive(true);
    let mut headers = HeaderMap::new();
    headers.insert("X-MBX-APIKEY", key);
    let (response, rate_limit) = http.request_with_rate_limit(
        ProviderEndpoint::BinanceTestnet,
        method,
        &path,
        headers,
        None,
    )?;
    observe_ip_rate_limit(response.status, rate_limit.as_ref());
    Ok(response)
}

fn server_time(http: &impl ProviderHttp, current: &impl Fn() -> bool) -> Result<(u64, Instant)> {
    check_testnet_ip_cooldown()?;
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let started = Instant::now();
    let (response, rate_limit) = http.request_with_rate_limit(
        ProviderEndpoint::BinanceTestnet,
        ProviderHttpMethod::Get,
        "/api/v3/time",
        HeaderMap::new(),
        None,
    )?;
    observe_ip_rate_limit(response.status, rate_limit.as_ref());
    if let Some(code) = rate_limit_code(response.status) {
        return Err(TradeXError::new(code));
    }
    if response.status != 200 || started.elapsed() > Duration::from_secs(2) {
        return Err(time_error());
    }
    let value: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
    let value = value["serverTime"]
        .as_u64()
        .filter(|value| valid_time(*value))
        .ok_or_else(time_error)?;
    Ok((value, Instant::now()))
}

fn signed_book_read(
    http: &impl ProviderHttp,
    route: &str,
    params: &[(&str, &str)],
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<Value> {
    let (server, sampled) = server_time(http, current)?;
    let response = signed_request(
        http,
        ProviderHttpMethod::Get,
        route,
        params,
        secrets,
        server,
        sampled,
        current,
    )?;
    if response.status != 200 {
        return Err(match response.status {
            418 => TradeXError::new("PROVIDER_IP_BANNED"),
            429 => TradeXError::new("PROVIDER_RATE_LIMITED"),
            401 | 403 => TradeXError::new("PROVIDER_AUTHENTICATION_FAILED"),
            404 => TradeXError::new("ORDER_STATUS_UNKNOWN"),
            _ => TradeXError::new("PROVIDER_UNAVAILABLE"),
        });
    }
    response_value(response, secrets)
}

fn iso_after(seconds: i64) -> Result<String> {
    time::OffsetDateTime::now_utc()
        .checked_add(time::Duration::seconds(seconds))
        .ok_or_else(invalid)?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| invalid())
}

fn retry_active(value: Option<&String>) -> bool {
    value
        .and_then(|value| {
            time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()
        })
        .is_some_and(|value| value > time::OffsetDateTime::now_utc())
}

fn provider_id(value: &Value, field: &str) -> Result<String> {
    match &value[field] {
        Value::String(id) if valid_order_id(id) => Ok(id.clone()),
        Value::Number(number) => number
            .as_u64()
            .filter(|id| *id > 0 && *id <= i64::MAX as u64)
            .map(|id| id.to_string())
            .ok_or_else(invalid),
        _ => Err(invalid()),
    }
}

fn provider_time(value: &Value, field: &str) -> Result<u64> {
    value[field]
        .as_u64()
        .filter(|value| *value <= crate::protocol::MAX_SEQUENCE)
        .ok_or_else(invalid)
}

fn optional_nonnegative_decimal(value: &Value) -> Result<Option<String>> {
    let raw = value.as_str().ok_or_else(invalid)?;
    let parsed = decimal(&Value::String(raw.to_owned()))?;
    if parsed.starts_with('-') {
        Ok(None)
    } else {
        Ok(Some(parsed))
    }
}

fn parse_testnet_book_order(value: &Value, observed_at: &str) -> Result<BinanceTestnetOrder> {
    let quantity = optional_nonnegative_decimal(&value["origQty"])?;
    let filled_quantity = positive(&value["executedQty"])?;
    let remaining_quantity = quantity
        .as_deref()
        .map(|quantity| crate::provider_io::decimal_subtract(quantity, &filled_quantity))
        .transpose()?;
    let order = BinanceTestnetOrder {
        provider_order_id: provider_id(value, "orderId")?,
        symbol: text(value, "symbol", 32)?,
        client_order_id: text(value, "clientOrderId", 36)?,
        side: text(value, "side", 16)?,
        order_type: text(value, "type", 32)?,
        time_in_force: value["timeInForce"]
            .as_str()
            .filter(|value| !value.is_empty() && value.len() <= 32)
            .unwrap_or("UNSPECIFIED")
            .to_owned(),
        provider_status: text(value, "status", 64)?,
        price: optional_nonnegative_decimal(&value["price"])?,
        quantity,
        quote_quantity: optional_nonnegative_decimal(&value["origQuoteOrderQty"])?,
        filled_quantity,
        filled_quote_quantity: optional_nonnegative_decimal(&value["cummulativeQuoteQty"])?,
        remaining_quantity,
        submitted_at_ms: provider_time(value, "time")?,
        provider_updated_at_ms: value
            .get("updateTime")
            .and_then(Value::as_u64)
            .filter(|value| *value <= crate::protocol::MAX_SEQUENCE)
            .unwrap_or(provider_time(value, "time")?),
        observed_at: observed_at.to_owned(),
        pending: !matches!(
            value["status"].as_str(),
            Some("FILLED" | "CANCELED" | "REJECTED" | "EXPIRED" | "EXPIRED_IN_MATCH")
        ),
        origin: BinanceTestnetOrderOrigin::External,
        cancel_state: BinanceTestnetOrderCancelState::None,
        cancel_idempotency_key: None,
        cancel_error: None,
        attempt_id: None,
    };
    if !valid_binance_symbol(&order.symbol)
        || !matches!(order.side.as_str(), "BUY" | "SELL")
        || order.provider_status.is_empty()
    {
        return Err(invalid());
    }
    Ok(order)
}

fn parse_testnet_fill(value: &Value, observed_at: &str) -> Result<BinanceTestnetFill> {
    let commission = positive(&value["commission"])?;
    let fill = BinanceTestnetFill {
        trade_id: provider_id(value, "id")?,
        provider_order_id: provider_id(value, "orderId")?,
        symbol: text(value, "symbol", 32)?,
        side: if value["isBuyer"].as_bool().ok_or_else(invalid)? {
            "BUY".into()
        } else {
            "SELL".into()
        },
        price: positive(&value["price"])?,
        quantity: positive(&value["qty"])?,
        quote_quantity: positive(&value["quoteQty"])?,
        commission,
        commission_asset: text(value, "commissionAsset", 32)?,
        executed_at_ms: provider_time(value, "time")?,
        observed_at: observed_at.to_owned(),
    };
    if !valid_binance_symbol(&fill.symbol)
        || fill.price == "0"
        || fill.quantity == "0"
        || fill.quote_quantity == "0"
        || fill.commission_asset.is_empty()
    {
        return Err(invalid());
    }
    Ok(fill)
}

fn merge_testnet_order(
    orders: &mut Vec<BinanceTestnetOrder>,
    fresh: BinanceTestnetOrder,
) -> Result<()> {
    if let Some(index) = orders.iter().position(|old| {
        old.provider_order_id == fresh.provider_order_id && old.symbol == fresh.symbol
    }) {
        let old = &orders[index];
        if old.client_order_id != fresh.client_order_id {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
        }
        let filled_order = decimal_cmp(&fresh.filled_quantity, &old.filled_quantity)?;
        if fresh.provider_updated_at_ms < old.provider_updated_at_ms
            || filled_order == std::cmp::Ordering::Less
            || (!old.pending && fresh.pending)
            || (fresh.provider_updated_at_ms == old.provider_updated_at_ms
                && filled_order == std::cmp::Ordering::Equal
                && (old.provider_status == fresh.provider_status
                    || old.provider_status == "FILLED"))
        {
            return Ok(());
        }
        let mut fresh = fresh;
        if fresh.pending {
            fresh.cancel_state = old.cancel_state;
            fresh.cancel_idempotency_key = old.cancel_idempotency_key.clone();
            fresh.cancel_error = old.cancel_error.clone();
        } else {
            fresh.cancel_state = BinanceTestnetOrderCancelState::None;
            fresh.cancel_idempotency_key = None;
            fresh.cancel_error = None;
        }
        orders[index] = fresh;
    } else {
        orders.push(fresh);
    }
    if orders.len() > 5000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    orders.sort_by(|left, right| {
        right
            .submitted_at_ms
            .cmp(&left.submitted_at_ms)
            .then_with(|| left.symbol.cmp(&right.symbol))
            .then_with(|| left.provider_order_id.cmp(&right.provider_order_id))
    });
    Ok(())
}

pub(super) struct TestnetPrivateStreamUpdate {
    pub event_at: String,
    pub reconciliation_required: bool,
}

fn provider_event_time(value: u64) -> Result<String> {
    let nanos = i128::from(value)
        .checked_mul(1_000_000)
        .ok_or_else(invalid)?;
    time::OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .map_err(|_| invalid())?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| invalid())
}

fn stream_order(value: &Value, observed_at: &str) -> Result<BinanceTestnetOrder> {
    let order = json!({
        "orderId": value["i"],
        "symbol": value["s"],
        "clientOrderId": value["c"],
        "side": value["S"],
        "type": value["o"],
        "timeInForce": value["f"],
        "status": value["X"],
        "price": value["p"],
        "origQty": value["q"],
        "origQuoteOrderQty": value["Q"],
        "executedQty": value["z"],
        "cummulativeQuoteQty": value["Z"],
        "time": value["O"],
        "updateTime": value["T"],
    });
    parse_testnet_book_order(&order, observed_at)
}

fn stream_fill(value: &Value, observed_at: &str) -> Result<BinanceTestnetFill> {
    let is_buyer = match value["S"].as_str() {
        Some("BUY") => true,
        Some("SELL") => false,
        _ => return Err(invalid()),
    };
    let fill = json!({
        "id": value["t"],
        "orderId": value["i"],
        "symbol": value["s"],
        "isBuyer": is_buyer,
        "price": value["L"],
        "qty": value["l"],
        "quoteQty": value["Y"],
        "commission": value["n"],
        "commissionAsset": value["N"],
        "time": value["T"],
    });
    parse_testnet_fill(&fill, observed_at)
}

fn stream_balance(value: &Value) -> Result<BinanceTestnetBalance> {
    let asset = text(value, "a", 32)?;
    let free = positive(&value["f"])?;
    let locked = positive(&value["l"])?;
    Ok(BinanceTestnetBalance {
        asset,
        total: total(&free, &locked)?,
        free,
        locked,
    })
}

pub(super) fn apply_testnet_private_stream_frame(
    book: &mut BinanceTestnetOrderBook,
    frame: &Value,
    subscription_id: u64,
    secrets: &[String],
) -> Result<Option<TestnetPrivateStreamUpdate>> {
    if contains_secret(frame, secrets) || frame["subscriptionId"].as_u64() != Some(subscription_id)
    {
        return Err(invalid());
    }
    let event = frame.get("event").ok_or_else(invalid)?;
    let event_type = text(event, "e", 64)?;
    if event_type == "eventStreamTerminated" {
        return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
    }
    let event_at = provider_event_time(provider_time(event, "E")?)?;
    let before = book.clone();
    let mut reconciliation_required = false;
    match event_type.as_str() {
        "executionReport" => {
            let mut order = stream_order(event, &event_at)?;
            if let Some(attempt) = book.orders.iter().find(|existing| {
                existing.symbol == order.symbol
                    && existing.provider_order_id == order.provider_order_id
                    && existing.client_order_id == order.client_order_id
            }) {
                order.origin = attempt.origin;
                order.attempt_id = attempt.attempt_id.clone();
            }
            let trade = event["x"].as_str() == Some("TRADE")
                && event["t"].as_i64().is_some_and(|id| id > 0);
            merge_testnet_order(&mut book.orders, order)?;
            if trade {
                merge_testnet_fill(&mut book.fills, stream_fill(event, &event_at)?)?;
            }
        }
        "outboundAccountPosition" => {
            let update_at_ms = provider_time(event, "u")?;
            let balances = event["B"].as_array().ok_or_else(invalid)?;
            if balances.len() > 5000 {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            let mut changed = HashSet::new();
            let balances = balances
                .iter()
                .map(|value| {
                    let balance = stream_balance(value)?;
                    if !changed.insert(balance.asset.clone()) {
                        return Err(invalid());
                    }
                    Ok(balance)
                })
                .collect::<Result<Vec<_>>>()?;
            match book.private_stream_balance_update_at_ms {
                Some(previous) if update_at_ms < previous => return Ok(None),
                Some(previous) if update_at_ms == previous => {
                    for fresh in &balances {
                        let old = book.balances.iter().find(|old| old.asset == fresh.asset);
                        if old != Some(fresh) {
                            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
                        }
                    }
                    return Ok(None);
                }
                _ => (),
            }
            for fresh in balances {
                book.balances.retain(|old| old.asset != fresh.asset);
                if fresh.total != "0" {
                    book.balances.push(fresh);
                }
            }
            book.balances
                .sort_by(|left, right| left.asset.cmp(&right.asset));
            book.private_stream_balance_update_at_ms = Some(update_at_ms);
            book.balances_observed_at = Some(event_at.clone());
        }
        "balanceUpdate" => {
            provider_time(event, "T")?;
            text(event, "a", 32)?;
            crate::provider_io::decimal(&event["d"])?;
            reconciliation_required = true;
        }
        _ => reconciliation_required = true,
    }
    if *book == before && !reconciliation_required {
        return Ok(None);
    }
    book.observed_at = crate::storage::timestamp()?;
    if reconciliation_required {
        book.status = BinanceTestnetOrderBookStatus::Stale;
        book.reason = Some("PRIVATE_STREAM_RECONCILIATION_REQUIRED".into());
    } else {
        book.reason = None;
    }
    Ok(Some(TestnetPrivateStreamUpdate {
        event_at,
        reconciliation_required,
    }))
}

fn merge_testnet_fill(
    fills: &mut Vec<BinanceTestnetFill>,
    fresh: BinanceTestnetFill,
) -> Result<()> {
    if let Some(old) = fills
        .iter()
        .find(|old| old.trade_id == fresh.trade_id && old.symbol == fresh.symbol)
    {
        if old.provider_order_id != fresh.provider_order_id
            || old.price != fresh.price
            || old.quantity != fresh.quantity
            || old.quote_quantity != fresh.quote_quantity
            || old.commission != fresh.commission
            || old.commission_asset != fresh.commission_asset
            || old.side != fresh.side
            || old.executed_at_ms != fresh.executed_at_ms
        {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
        }
        return Ok(());
    }
    fills.push(fresh);
    if fills.len() > 5000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    fills.sort_by(|left, right| {
        right
            .executed_at_ms
            .cmp(&left.executed_at_ms)
            .then_with(|| left.symbol.cmp(&right.symbol))
            .then_with(|| left.trade_id.cmp(&right.trade_id))
    });
    Ok(())
}

fn binance_account(
    http: &impl ProviderHttp,
    secrets: &[String],
    current: &impl Fn() -> bool,
    remote_account_id: &str,
) -> Result<Value> {
    let account = signed_book_read(http, "/api/v3/account", &[], secrets, current)?;
    if numeric_id(&account, "uid")? != remote_account_id
        || account["accountType"].as_str() != Some("SPOT")
    {
        return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
    }
    Ok(account)
}

fn account_balances(account: &Value) -> Result<Vec<BinanceTestnetBalance>> {
    let rows = account["balances"].as_array().ok_or_else(invalid)?;
    if rows.len() > 5000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let mut assets = HashSet::new();
    rows.iter()
        .filter_map(|row| {
            let result = (|| -> Result<BinanceTestnetBalance> {
                let asset = text(row, "asset", 32)?;
                if !assets.insert(asset.clone()) {
                    return Err(invalid());
                }
                let free = positive(&row["free"])?;
                let locked = positive(&row["locked"])?;
                let total = total(&free, &locked)?;
                Ok(BinanceTestnetBalance {
                    asset,
                    free,
                    locked,
                    total,
                })
            })();
            Some(result)
        })
        .collect::<Result<Vec<_>>>()
        .map(|mut balances| {
            balances.retain(|balance| balance.total != "0");
            balances.sort_by(|left, right| left.asset.cmp(&right.asset));
            balances
        })
}

fn history_state_mut<'a>(
    book: &'a mut BinanceTestnetOrderBook,
    symbol: &str,
) -> Result<&'a mut BinanceTestnetHistoryState> {
    book.history
        .iter_mut()
        .find(|state| state.symbol == symbol)
        .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))
}

fn fetch_testnet_history(
    book: &mut BinanceTestnetOrderBook,
    symbol: &str,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    observed_at: &str,
) -> Result<()> {
    if !matches!(symbol, "BTCUSDT" | "ETHUSDT") {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let state = book
        .history
        .iter()
        .find(|state| state.symbol == symbol)
        .cloned()
        .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
    let start_from_beginning = !state.started || state.complete;
    let mut order_cursor = state
        .next_order_id
        .clone()
        .or_else(|| start_from_beginning.then(|| "1".into()));
    let mut trade_cursor = state
        .next_trade_id
        .clone()
        .or_else(|| start_from_beginning.then(|| "1".into()));
    if start_from_beginning || order_cursor.is_some() {
        let mut params = vec![("symbol", symbol), ("limit", "1000")];
        if let Some(cursor) = order_cursor.as_deref() {
            params.push(("orderId", cursor));
        }
        let value = signed_book_read(http, "/api/v3/allOrders", &params, secrets, current)?;
        let rows = value.as_array().ok_or_else(invalid)?;
        if rows.len() > 1000 {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        for row in rows {
            if row["symbol"].as_str() != Some(symbol) {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
            }
            merge_testnet_order(
                &mut book.orders,
                parse_testnet_book_order(row, observed_at)?,
            )?;
        }
        order_cursor = if rows.len() == 1000 {
            Some(increment_id(&provider_id(
                rows.last().ok_or_else(invalid)?,
                "orderId",
            )?)?)
        } else {
            None
        };
    }
    if start_from_beginning || trade_cursor.is_some() {
        let mut params = vec![("symbol", symbol), ("limit", "1000")];
        if let Some(cursor) = trade_cursor.as_deref() {
            params.push(("fromId", cursor));
        }
        let value = signed_book_read(http, "/api/v3/myTrades", &params, secrets, current)?;
        let rows = value.as_array().ok_or_else(invalid)?;
        if rows.len() > 1000 {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        for row in rows {
            if row["symbol"].as_str() != Some(symbol) {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
            }
            merge_testnet_fill(&mut book.fills, parse_testnet_fill(row, observed_at)?)?;
        }
        trade_cursor = if rows.len() == 1000 {
            Some(increment_id(&provider_id(
                rows.last().ok_or_else(invalid)?,
                "id",
            )?)?)
        } else {
            None
        };
    }
    let state = history_state_mut(book, symbol)?;
    state.started = true;
    state.last_observed_at = Some(observed_at.to_owned());
    state.page_count = state
        .page_count
        .checked_add(1)
        .filter(|count| *count <= crate::protocol::MAX_SEQUENCE)
        .ok_or_else(invalid)?;
    state.next_order_id = order_cursor;
    state.next_trade_id = trade_cursor;
    state.complete = state.next_order_id.is_none() && state.next_trade_id.is_none();
    Ok(())
}

fn increment_id(value: &str) -> Result<String> {
    if !valid_order_id(value) {
        return Err(invalid());
    }
    let mut digits = value.bytes().collect::<Vec<_>>();
    let mut carry = true;
    for digit in digits.iter_mut().rev() {
        if !carry {
            break;
        }
        if *digit == b'9' {
            *digit = b'0';
        } else {
            *digit += 1;
            carry = false;
        }
    }
    if carry {
        digits.insert(0, b'1');
    }
    let next = String::from_utf8(digits).map_err(|_| invalid())?;
    if valid_order_id(&next) {
        Ok(next)
    } else {
        Err(invalid())
    }
}

pub(super) fn refresh_testnet_order_book(
    book: &mut BinanceTestnetOrderBook,
    action: BinanceTestnetOrderBookAction,
    symbol: Option<&str>,
    provider_order_id: Option<&str>,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<()> {
    let account_retry = book.rate_limits.account_retry_at.as_ref();
    let action_retry = match action {
        BinanceTestnetOrderBookAction::Pending => book.rate_limits.pending_orders_retry_at.as_ref(),
        BinanceTestnetOrderBookAction::Account => account_retry,
        BinanceTestnetOrderBookAction::History => book.rate_limits.history_retry_at.as_ref(),
        BinanceTestnetOrderBookAction::Detail => book.rate_limits.order_detail_retry_at.as_ref(),
    };
    if retry_active(account_retry) || retry_active(action_retry) {
        book.status = BinanceTestnetOrderBookStatus::Degraded;
        book.reason = Some("PROVIDER_RATE_LIMITED".into());
        book.observed_at = crate::storage::timestamp()?;
        return Ok(());
    }
    if let Some((deadline, code)) = testnet_ip_retry_at() {
        book.rate_limits.account_retry_at = Some(deadline.clone());
        match action {
            BinanceTestnetOrderBookAction::Pending => {
                book.rate_limits.pending_orders_retry_at = Some(deadline)
            }
            BinanceTestnetOrderBookAction::Account => (),
            BinanceTestnetOrderBookAction::History => {
                book.rate_limits.history_retry_at = Some(deadline)
            }
            BinanceTestnetOrderBookAction::Detail => {
                book.rate_limits.order_detail_retry_at = Some(deadline)
            }
        }
        book.status = BinanceTestnetOrderBookStatus::Degraded;
        book.reason = Some(code.into());
        book.observed_at = crate::storage::timestamp()?;
        return Ok(());
    }
    let args_valid = match action {
        BinanceTestnetOrderBookAction::Pending | BinanceTestnetOrderBookAction::Account => {
            symbol.is_none() && provider_order_id.is_none()
        }
        BinanceTestnetOrderBookAction::History => {
            matches!(symbol, Some("BTCUSDT" | "ETHUSDT")) && provider_order_id.is_none()
        }
        BinanceTestnetOrderBookAction::Detail => {
            symbol.zip(provider_order_id).is_some_and(|(symbol, id)| {
                valid_order_id(id)
                    && book
                        .orders
                        .iter()
                        .any(|order| order.symbol == symbol && order.provider_order_id == id)
            })
        }
    };
    if !args_valid {
        book.status = BinanceTestnetOrderBookStatus::Degraded;
        book.reason = Some("IPC_PAYLOAD_INVALID".into());
        book.observed_at = crate::storage::timestamp()?;
        return Ok(());
    }
    let mut candidate = book.clone();
    let account_interval = match action {
        BinanceTestnetOrderBookAction::Pending | BinanceTestnetOrderBookAction::Account => 20,
        _ => 30,
    };
    candidate.rate_limits.account_retry_at = Some(iso_after(account_interval)?);
    match action {
        BinanceTestnetOrderBookAction::Pending => {
            candidate.rate_limits.pending_orders_retry_at = Some(iso_after(30)?);
        }
        BinanceTestnetOrderBookAction::Account => (),
        BinanceTestnetOrderBookAction::History => {
            candidate.rate_limits.history_retry_at = Some(iso_after(30)?);
        }
        BinanceTestnetOrderBookAction::Detail => {
            candidate.rate_limits.order_detail_retry_at = Some(iso_after(5)?);
        }
    }
    let observed_at = crate::storage::timestamp()?;
    let pending_reconciliation_required =
        book.reason.as_deref() == Some("ORDER_STATUS_REFRESH_REQUIRED");
    candidate.reason = None;
    let result = (|| -> Result<()> {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let account = binance_account(http, secrets, current, &candidate.remote_account_id)?;
        match action {
            BinanceTestnetOrderBookAction::Pending => {
                let response = signed_book_read(http, "/api/v3/openOrders", &[], secrets, current)?;
                let rows = response.as_array().ok_or_else(invalid)?;
                if rows.len() > 5000 {
                    return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                }
                let mut ids = HashSet::new();
                for row in rows {
                    let fresh = parse_testnet_book_order(row, &observed_at)?;
                    ids.insert((fresh.symbol.clone(), fresh.provider_order_id.clone()));
                    merge_testnet_order(&mut candidate.orders, fresh)?;
                }
                let missing_open = candidate.orders.iter().any(|order| {
                    order.pending
                        && !ids.contains(&(order.symbol.clone(), order.provider_order_id.clone()))
                });
                if missing_open {
                    candidate.status = BinanceTestnetOrderBookStatus::Stale;
                    candidate.reason = Some("ORDER_STATUS_REFRESH_REQUIRED".into());
                }
            }
            BinanceTestnetOrderBookAction::Account => {
                candidate.balances = account_balances(&account)?;
            }
            BinanceTestnetOrderBookAction::History => {
                fetch_testnet_history(
                    &mut candidate,
                    symbol.ok_or_else(invalid)?,
                    secrets,
                    http,
                    current,
                    &observed_at,
                )?;
            }
            BinanceTestnetOrderBookAction::Detail => {
                let symbol = symbol.ok_or_else(invalid)?;
                let provider_order_id = provider_order_id.ok_or_else(invalid)?;
                let query = [("symbol", symbol), ("orderId", provider_order_id)];
                let response = signed_book_read(http, "/api/v3/order", &query, secrets, current)?;
                let fresh = parse_testnet_book_order(&response, &observed_at)?;
                if fresh.provider_order_id != provider_order_id || fresh.symbol != symbol {
                    return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
                }
                merge_testnet_order(&mut candidate.orders, fresh)?;
            }
        }
        Ok(())
    })();
    let observed_at = crate::storage::timestamp()?;
    match result {
        Ok(()) => {
            match action {
                BinanceTestnetOrderBookAction::Pending => {
                    candidate.pending_orders_observed_at = Some(observed_at.clone());
                }
                BinanceTestnetOrderBookAction::Account => {
                    candidate.balances_observed_at = Some(observed_at.clone());
                }
                BinanceTestnetOrderBookAction::History | BinanceTestnetOrderBookAction::Detail => {}
            }
            if pending_reconciliation_required && action != BinanceTestnetOrderBookAction::Pending {
                candidate.status = BinanceTestnetOrderBookStatus::Stale;
                candidate.reason = Some("ORDER_STATUS_REFRESH_REQUIRED".into());
            } else if candidate.reason.as_deref() != Some("ORDER_STATUS_REFRESH_REQUIRED") {
                candidate.status = BinanceTestnetOrderBookStatus::Current;
                candidate.reason = None;
            }
            candidate.last_successful_sync_at = Some(observed_at.clone());
            candidate.observed_at = observed_at;
            *book = candidate;
        }
        Err(error) => {
            book.rate_limits = candidate.rate_limits;
            if matches!(
                error.code.as_str(),
                "PROVIDER_RATE_LIMITED" | "PROVIDER_IP_BANNED"
            ) {
                let deadline = testnet_ip_retry_at()
                    .map(|(deadline, _)| deadline)
                    .unwrap_or(iso_after(if error.code == "PROVIDER_IP_BANNED" {
                        120
                    } else {
                        60
                    })?);
                book.rate_limits.account_retry_at = Some(deadline.clone());
                match action {
                    BinanceTestnetOrderBookAction::Pending => {
                        book.rate_limits.pending_orders_retry_at = Some(deadline)
                    }
                    BinanceTestnetOrderBookAction::Account => (),
                    BinanceTestnetOrderBookAction::History => {
                        book.rate_limits.history_retry_at = Some(deadline)
                    }
                    BinanceTestnetOrderBookAction::Detail => {
                        book.rate_limits.order_detail_retry_at = Some(deadline)
                    }
                }
            }
            book.status = if error.code == "STATE_VERSION_CONFLICT" {
                BinanceTestnetOrderBookStatus::Stale
            } else {
                BinanceTestnetOrderBookStatus::Degraded
            };
            book.reason = Some(error.code);
            book.observed_at = observed_at;
        }
    }
    Ok(())
}

pub(super) fn cancel_testnet_order(
    book: &mut BinanceTestnetOrderBook,
    symbol: &str,
    provider_order_id: &str,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    cancel_may_have_been_sent: &mut bool,
) -> Result<()> {
    let index = book
        .orders
        .iter()
        .position(|order| order.symbol == symbol && order.provider_order_id == provider_order_id)
        .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
    let reviewed = book.orders[index].clone();
    let cancel_id = reviewed
        .cancel_idempotency_key
        .as_deref()
        .filter(|_| reviewed.cancel_state == BinanceTestnetOrderCancelState::Submitting)
        .ok_or_else(|| TradeXError::new("ORDER_CONFIRMATION_REQUIRED"))?;

    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    binance_account(http, secrets, current, &book.remote_account_id)?;
    let order = get_testnet_order(http, &reviewed, secrets, current)?;
    merge_testnet_order(&mut book.orders, order)?;
    book.status = BinanceTestnetOrderBookStatus::Current;
    book.reason = None;
    let fresh = book.orders[index].clone();
    if !fresh.pending {
        clear_cancel(&mut book.orders[index]);
        book.observed_at = crate::storage::timestamp()?;
        book.status = BinanceTestnetOrderBookStatus::Current;
        book.reason = None;
        return Ok(());
    }
    if !same_testnet_order_review(&reviewed, &fresh) {
        finish_cancel_rejected(
            &mut book.orders[index],
            cancel_id,
            "ORDER_CHANGED_REVIEW_AGAIN",
        );
        book.reason = Some("ORDER_CHANGED_REVIEW_AGAIN".into());
        book.status = BinanceTestnetOrderBookStatus::Stale;
        return Ok(());
    }
    if !testnet_cancelable(&fresh) {
        finish_cancel_rejected(&mut book.orders[index], cancel_id, "ORDER_NOT_CANCELABLE");
        book.reason = Some("ORDER_NOT_CANCELABLE".into());
        book.status = BinanceTestnetOrderBookStatus::Stale;
        return Ok(());
    }
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }

    let (server, sampled) = server_time(http, current)?;
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    *cancel_may_have_been_sent = true;
    let params = [
        ("symbol", symbol),
        ("orderId", provider_order_id),
        ("newClientOrderId", cancel_id),
    ];
    let response = signed_request(
        http,
        ProviderHttpMethod::Delete,
        "/api/v3/order",
        &params,
        secrets,
        server,
        sampled,
        current,
    );
    let response = match response {
        Ok(response) if response.status == 200 => response,
        Ok(response) if matches!(response.status, 400 | 401 | 403 | 404 | 418 | 429) => {
            *cancel_may_have_been_sent = false;
            let code = match response.status {
                400 => "PROVIDER_CANCEL_REJECTED",
                401 | 403 => "PROVIDER_AUTHENTICATION_FAILED",
                404 => "ORDER_STATUS_UNKNOWN",
                418 => "PROVIDER_IP_BANNED",
                _ => "PROVIDER_RATE_LIMITED",
            };
            finish_cancel_rejected(&mut book.orders[index], cancel_id, code);
            match get_testnet_order(http, &fresh, secrets, current) {
                Ok(order) => {
                    merge_testnet_order(&mut book.orders, order)?;
                    let updated = &mut book.orders[index];
                    if !updated.pending {
                        clear_cancel(updated);
                    } else if updated.cancel_state == BinanceTestnetOrderCancelState::None {
                        updated.cancel_idempotency_key = Some(cancel_id.to_owned());
                        updated.cancel_error = Some(code.into());
                    }
                    book.status = BinanceTestnetOrderBookStatus::Current;
                    book.reason = Some(code.into());
                }
                Err(_) => {
                    mark_cancel_unknown(book, index, cancel_id);
                }
            }
            book.observed_at = crate::storage::timestamp()?;
            return Ok(());
        }
        Ok(_) | Err(_) => {
            mark_cancel_unknown(book, index, cancel_id);
            book.observed_at = crate::storage::timestamp()?;
            let _ = reconcile_testnet_cancel_order(book, index, &fresh, secrets, http, current);
            return Ok(());
        }
    };

    match parse_testnet_cancel_ack(response, &fresh, secrets) {
        Ok(mut acknowledged) => {
            acknowledged.cancel_state = BinanceTestnetOrderCancelState::Pending;
            acknowledged.cancel_idempotency_key = Some(cancel_id.to_owned());
            acknowledged.cancel_error = None;
            merge_testnet_order(&mut book.orders, acknowledged)?;
            set_cancel_pending(&mut book.orders[index], cancel_id, "");
            match reconcile_testnet_cancel_order(book, index, &fresh, secrets, http, current) {
                Ok(()) => (),
                Err(_) => {
                    mark_cancel_unknown(book, index, cancel_id);
                }
            }
        }
        Err(_) => {
            mark_cancel_unknown(book, index, cancel_id);
            let _ = reconcile_testnet_cancel_order(book, index, &fresh, secrets, http, current);
        }
    }
    book.observed_at = crate::storage::timestamp()?;
    Ok(())
}

pub(crate) fn merge_testnet_cancel_observation(
    latest: &mut BinanceTestnetOrderBook,
    outcome: &BinanceTestnetOrderBook,
    symbol: &str,
    provider_order_id: &str,
) -> Result<()> {
    merge_cancel_observation_into_latest(latest, outcome, symbol, provider_order_id)
}

pub(super) fn merge_cancel_observation_into_latest(
    latest: &mut BinanceTestnetOrderBook,
    outcome: &BinanceTestnetOrderBook,
    symbol: &str,
    provider_order_id: &str,
) -> Result<()> {
    let index = latest
        .orders
        .iter()
        .position(|order| order.symbol == symbol && order.provider_order_id == provider_order_id)
        .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
    let local_key = latest.orders[index].cancel_idempotency_key.clone();
    let fresh = outcome
        .orders
        .iter()
        .find(|order| order.symbol == symbol && order.provider_order_id == provider_order_id)
        .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?
        .clone();
    merge_testnet_order(&mut latest.orders, fresh.clone())?;
    let merged = &mut latest.orders[index];
    if let Some(local_key) = local_key {
        if !merged.pending {
            clear_cancel(merged);
        } else if fresh.pending
            && fresh.cancel_idempotency_key.as_deref() == Some(local_key.as_str())
        {
            merged.cancel_state = fresh.cancel_state;
            merged.cancel_idempotency_key = Some(local_key);
            merged.cancel_error = fresh.cancel_error;
        } else {
            merged.cancel_state = BinanceTestnetOrderCancelState::Pending;
            merged.cancel_idempotency_key = Some(local_key);
            merged.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
        }
    }
    for fill in outcome
        .fills
        .iter()
        .filter(|fill| fill.symbol == symbol && fill.provider_order_id == provider_order_id)
    {
        merge_testnet_fill(&mut latest.fills, fill.clone())?;
    }
    latest.status = BinanceTestnetOrderBookStatus::Stale;
    latest.reason = Some("ORDER_BOOK_CHANGED_DURING_CANCEL".into());
    Ok(())
}

fn reconcile_testnet_cancel_order(
    book: &mut BinanceTestnetOrderBook,
    index: usize,
    reviewed: &BinanceTestnetOrder,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<()> {
    match get_testnet_order(http, reviewed, secrets, current) {
        Ok(order) => {
            merge_testnet_order(&mut book.orders, order)?;
            let order = &mut book.orders[index];
            if !order.pending {
                clear_cancel(order);
                book.status = BinanceTestnetOrderBookStatus::Current;
                book.reason = None;
            } else if matches!(
                order.cancel_state,
                BinanceTestnetOrderCancelState::Submitting
                    | BinanceTestnetOrderCancelState::Pending
            ) {
                order.cancel_state = BinanceTestnetOrderCancelState::Pending;
                book.status = BinanceTestnetOrderBookStatus::Current;
                book.reason = None;
            }
            Ok(())
        }
        Err(error) => {
            book.status = BinanceTestnetOrderBookStatus::Degraded;
            book.reason = Some(error.code.clone());
            Err(error)
        }
    }
}

fn get_testnet_order(
    http: &impl ProviderHttp,
    reviewed: &BinanceTestnetOrder,
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<BinanceTestnetOrder> {
    let params = [
        ("symbol", reviewed.symbol.as_str()),
        ("orderId", reviewed.provider_order_id.as_str()),
    ];
    let value = signed_book_read(http, "/api/v3/order", &params, secrets, current)?;
    let fresh = parse_testnet_book_order(&value, &crate::storage::timestamp()?)?;
    if !same_testnet_order_identity(reviewed, &fresh) {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
    }
    Ok(fresh)
}

fn parse_testnet_cancel_ack(
    response: ProviderHttpResponse,
    reviewed: &BinanceTestnetOrder,
    secrets: &[String],
) -> Result<BinanceTestnetOrder> {
    let mut value = response_value(response, secrets)?;
    if value["symbol"].as_str() != Some(reviewed.symbol.as_str())
        || provider_id(&value, "orderId")? != reviewed.provider_order_id
        || value["origClientOrderId"].as_str() != Some(reviewed.client_order_id.as_str())
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
    }
    let transact_time = provider_time(&value, "transactTime")?;
    value["clientOrderId"] = reviewed.client_order_id.clone().into();
    value["time"] = reviewed.submitted_at_ms.into();
    value["updateTime"] = transact_time.into();
    let mut acknowledged = parse_testnet_book_order(&value, &crate::storage::timestamp()?)?;
    if !same_testnet_order_identity(reviewed, &acknowledged)
        || acknowledged.quantity != reviewed.quantity
        || acknowledged.quote_quantity != reviewed.quote_quantity
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
    }
    acknowledged.provider_status = reviewed.provider_status.clone();
    acknowledged.pending = reviewed.pending;
    Ok(acknowledged)
}

fn same_testnet_order_identity(left: &BinanceTestnetOrder, right: &BinanceTestnetOrder) -> bool {
    left.provider_order_id == right.provider_order_id
        && left.symbol == right.symbol
        && left.client_order_id == right.client_order_id
        && left.side == right.side
        && left.order_type == right.order_type
        && left.time_in_force == right.time_in_force
        && left.quantity == right.quantity
        && left.quote_quantity == right.quote_quantity
        && left.submitted_at_ms == right.submitted_at_ms
}

fn same_testnet_order_review(left: &BinanceTestnetOrder, right: &BinanceTestnetOrder) -> bool {
    same_testnet_order_identity(left, right)
        && left.provider_status == right.provider_status
        && left.filled_quantity == right.filled_quantity
        && left.filled_quote_quantity == right.filled_quote_quantity
        && left.remaining_quantity == right.remaining_quantity
        && left.provider_updated_at_ms == right.provider_updated_at_ms
}

fn testnet_cancelable(order: &BinanceTestnetOrder) -> bool {
    order.pending && matches!(order.provider_status.as_str(), "NEW" | "PARTIALLY_FILLED")
}

fn clear_cancel(order: &mut BinanceTestnetOrder) {
    order.cancel_state = BinanceTestnetOrderCancelState::None;
    order.cancel_idempotency_key = None;
    order.cancel_error = None;
}

fn finish_cancel_rejected(order: &mut BinanceTestnetOrder, key: &str, error: &str) {
    order.cancel_state = BinanceTestnetOrderCancelState::None;
    order.cancel_idempotency_key = Some(key.to_owned());
    order.cancel_error = Some(error.to_owned());
}

fn set_cancel_pending(order: &mut BinanceTestnetOrder, key: &str, error: &str) {
    order.cancel_state = BinanceTestnetOrderCancelState::Pending;
    order.cancel_idempotency_key = Some(key.to_owned());
    order.cancel_error = (!error.is_empty()).then(|| error.to_owned());
}

fn mark_cancel_unknown(book: &mut BinanceTestnetOrderBook, index: usize, key: &str) {
    set_cancel_pending(&mut book.orders[index], key, "ORDER_CANCEL_STATUS_UNKNOWN");
    book.status = BinanceTestnetOrderBookStatus::Degraded;
    book.reason = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
}

pub(super) fn reconcile_testnet_order_book(
    book: &mut BinanceTestnetOrderBook,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<()> {
    let trusted = book.clone();
    let result = (|| -> Result<()> {
        for (action, symbol) in [
            (BinanceTestnetOrderBookAction::Pending, None),
            (BinanceTestnetOrderBookAction::Account, None),
        ] {
            reconcile_action(book, action, symbol, secrets, http, current)?;
        }
        for symbol in ["BTCUSDT", "ETHUSDT"] {
            let mut complete = false;
            for _ in 0..5 {
                reconcile_action(
                    book,
                    BinanceTestnetOrderBookAction::History,
                    Some(symbol),
                    secrets,
                    http,
                    current,
                )?;
                complete = book
                    .history
                    .iter()
                    .find(|history| history.symbol == symbol)
                    .is_some_and(|history| history.complete);
                if complete {
                    break;
                }
            }
            if !complete {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
        }
        book.reason = None;
        reconcile_action(
            book,
            BinanceTestnetOrderBookAction::Pending,
            None,
            secrets,
            http,
            current,
        )?;
        if book.reason.as_deref() == Some("ORDER_STATUS_REFRESH_REQUIRED") {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        book.status = BinanceTestnetOrderBookStatus::Current;
        book.reason = None;
        let observed_at = crate::storage::timestamp()?;
        book.last_successful_sync_at = Some(observed_at.clone());
        book.observed_at = observed_at;
        Ok(())
    })();
    if let Err(error) = result {
        let observed_at = crate::storage::timestamp()?;
        *book = trusted;
        book.status = if error.code == "STATE_VERSION_CONFLICT" {
            BinanceTestnetOrderBookStatus::Stale
        } else {
            BinanceTestnetOrderBookStatus::Degraded
        };
        book.reason = Some(error.code.clone());
        book.observed_at = observed_at;
        return Err(error);
    }
    Ok(())
}

pub(super) fn private_stream_subscription(
    http: &impl ProviderHttp,
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<(String, Value)> {
    if secrets.len() != 2 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let (timestamp, _) = server_time(http, current)?;
    let signed = format!(
        "apiKey={}&recvWindow=5000&timestamp={timestamp}",
        secrets[0]
    );
    let signature = signature(&signed, &secrets[1])?;
    let request_id = uuid::Uuid::new_v4().to_string();
    Ok((
        request_id.clone(),
        json!({
            "id": request_id,
            "method": "userDataStream.subscribe.signature",
            "params": {
                "apiKey": secrets[0],
                "recvWindow": 5000,
                "timestamp": timestamp,
                "signature": signature,
            }
        }),
    ))
}

pub(super) fn verify_private_stream_account(
    http: &impl ProviderHttp,
    secrets: &[String],
    remote_account_id: &str,
    current: &impl Fn() -> bool,
) -> Result<()> {
    binance_account(http, secrets, current, remote_account_id).map(|_| ())
}

fn reconcile_action(
    book: &mut BinanceTestnetOrderBook,
    action: BinanceTestnetOrderBookAction,
    symbol: Option<&str>,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<()> {
    book.rate_limits.account_retry_at = None;
    match action {
        BinanceTestnetOrderBookAction::Pending => {
            book.rate_limits.pending_orders_retry_at = None;
        }
        BinanceTestnetOrderBookAction::Account => (),
        BinanceTestnetOrderBookAction::History => {
            book.rate_limits.history_retry_at = None;
        }
        BinanceTestnetOrderBookAction::Detail => return Err(invalid()),
    }
    refresh_testnet_order_book(book, action, symbol, None, secrets, http, current)?;
    match book.status {
        BinanceTestnetOrderBookStatus::Current => Ok(()),
        BinanceTestnetOrderBookStatus::Stale
            if book.reason.as_deref() == Some("ORDER_STATUS_REFRESH_REQUIRED") =>
        {
            Ok(())
        }
        _ => Err(TradeXError::new(
            book.reason.as_deref().unwrap_or("PROVIDER_DATA_INCOMPLETE"),
        )),
    }
}

fn response_value(response: ProviderHttpResponse, secrets: &[String]) -> Result<Value> {
    if response.body.len() as u64 > MAX_RESPONSE {
        return Err(invalid());
    }
    let value: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
    if contains_secret(&value, secrets) {
        return Err(invalid());
    }
    Ok(value)
}

fn provider_symbol(proposal: &OrderProposal, connection_id: &str) -> Result<String> {
    let fields = &proposal.fields;
    if !matches!(
        proposal.status,
        crate::protocol::OrderProposalStatus::NeedsApproval
            | crate::protocol::OrderProposalStatus::Consumed
    ) || fields.environment != ExecutionContext::BinanceTestnet
        || fields.account_id.as_deref() != Some(connection_id)
        || fields.venue != "BINANCE"
        || fields
            .maximum_spend
            .as_deref()
            .is_some_and(|value| decimal(&Value::String(value.into())).is_err())
    {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    let instrument = market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == fields.instrument_id)
        .filter(|instrument| instrument.asset_class == crate::protocol::AssetClass::CryptoSpot)
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))?;
    let symbol = instrument
        .providers
        .iter()
        .find(|mapping| mapping.provider_id == "binance")
        .map(|mapping| mapping.provider_symbol.clone())
        .filter(|symbol| matches!(symbol.as_str(), "BTCUSDT" | "ETHUSDT"))
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))?;
    let canonical = format!(
        "crypto:{}/{}:spot",
        instrument.base.as_deref().ok_or_else(invalid)?,
        instrument.quote.as_deref().ok_or_else(invalid)?
    );
    if canonical != fields.instrument_id
        || symbol
            != format!(
                "{}{}",
                instrument.base.as_deref().ok_or_else(invalid)?,
                instrument.quote.as_deref().ok_or_else(invalid)?
            )
    {
        return Err(TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"));
    }
    Ok(symbol)
}

pub(crate) fn validate_binance_testnet_proposal(
    proposal: &OrderProposal,
    connection_id: &str,
) -> Result<()> {
    let symbol = provider_symbol(proposal, connection_id)?;
    let fields = &proposal.fields;
    let quantity = decimal(&Value::String(fields.quantity.value.clone()))?;
    if quantity.starts_with('-') || quantity == "0" {
        return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
    }
    match (
        fields.order_type,
        fields.quantity.r#type,
        fields.side,
        fields.time_in_force,
    ) {
        (OrderType::Market, OrderQuantityType::Quote, OrderSide::Buy, TimeInForce::Day)
        | (OrderType::Market, OrderQuantityType::Base, _, TimeInForce::Day)
        | (
            OrderType::Limit,
            OrderQuantityType::Base,
            _,
            TimeInForce::Gtc | TimeInForce::Ioc | TimeInForce::Fok,
        ) => (),
        _ => return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED")),
    }
    if fields.order_type == OrderType::Limit {
        let price = decimal(&Value::String(
            fields
                .limit_price
                .clone()
                .ok_or_else(|| TradeXError::new("ORDER_AMOUNT_INVALID"))?,
        ))?;
        if price.starts_with('-') || price == "0" {
            return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
        }
    } else if fields.limit_price.is_some() {
        return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED"));
    }
    if fields.quantity.r#type == OrderQuantityType::Quote && symbol.is_empty() {
        return Err(invalid());
    }
    Ok(())
}

fn parse_symbol(value: &Value, expected: &str, base: &str, quote: &str) -> Result<Value> {
    let symbols = value["symbols"].as_array().ok_or_else(invalid)?;
    if symbols.len() > 10_000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let mut matches = symbols
        .iter()
        .filter(|symbol| symbol["symbol"].as_str() == Some(expected));
    let symbol = matches
        .next()
        .cloned()
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_UNAVAILABLE"))?;
    if matches.next().is_some()
        || symbol["status"] != "TRADING"
        || symbol["baseAsset"] != base
        || symbol["quoteAsset"] != quote
    {
        return Err(TradeXError::new("ORDER_INSTRUMENT_UNAVAILABLE"));
    }
    Ok(symbol)
}

fn filter<'a>(symbol: &'a Value, name: &str) -> Result<&'a Value> {
    let values = symbol["filters"].as_array().ok_or_else(invalid)?;
    let mut matching = values
        .iter()
        .filter(|value| value["filterType"].as_str() == Some(name));
    let result = matching
        .next()
        .ok_or_else(|| TradeXError::new("ORDER_FILTERS_UNAVAILABLE"))?;
    if matching.next().is_some() {
        return Err(invalid());
    }
    Ok(result)
}

fn scaled(value: &str, scale: usize) -> Result<String> {
    let value = decimal(&Value::String(value.into()))?;
    if value.starts_with('-') {
        return Err(invalid());
    }
    let (whole, fraction) = value.split_once('.').unwrap_or((&value, ""));
    if fraction.len() > scale {
        return Err(invalid());
    }
    let digits = format!("{whole}{fraction}{}", "0".repeat(scale - fraction.len()));
    let digits = digits.trim_start_matches('0').to_owned();
    Ok(if digits.is_empty() {
        "0".into()
    } else {
        digits
    })
}

fn multiple_of(value: &str, step: &str) -> Result<bool> {
    let step = decimal(&Value::String(step.into()))?;
    if step.starts_with('-') || step == "0" {
        return Err(invalid());
    }
    let scale = value
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
        .max(
            step.split_once('.')
                .map_or(0, |(_, fraction)| fraction.len()),
        );
    let numerator = scaled(value, scale)?;
    let divisor = scaled(&step, scale)?;
    if divisor == "0" {
        return Err(invalid());
    }
    let mut remainder = "0".to_owned();
    for digit in numerator.bytes() {
        remainder.push(digit as char);
        remainder = remainder.trim_start_matches('0').to_owned();
        if remainder.is_empty() {
            remainder.push('0');
        }
        for _ in 0..10 {
            if decimal_cmp(&remainder, &divisor)? == std::cmp::Ordering::Less {
                break;
            }
            remainder = decimal_subtract(&remainder, &divisor)?;
        }
    }
    Ok(remainder == "0")
}

fn check_range_step(value: &str, min: &Value, max: &Value, step: &Value) -> Result<()> {
    let min = positive(min)?;
    let max = positive(max)?;
    let step = positive(step)?;
    let value = decimal(&Value::String(value.into()))?;
    if (min != "0" && decimal_cmp(&value, &min)? == std::cmp::Ordering::Less)
        || (max != "0" && decimal_cmp(&value, &max)? == std::cmp::Ordering::Greater)
        || (step != "0" && !multiple_of(&value, &step)?)
    {
        return Err(TradeXError::new("ORDER_FILTER_REJECTED"));
    }
    Ok(())
}

fn multiply(left: &str, right: &str) -> Result<String> {
    let left = decimal(&Value::String(left.into()))?;
    let right = decimal(&Value::String(right.into()))?;
    if left.starts_with('-') || right.starts_with('-') {
        return Err(invalid());
    }
    let parts = |value: &str| {
        let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
        (format!("{whole}{fraction}"), fraction.len())
    };
    let (left, left_scale) = parts(&left);
    let (right, right_scale) = parts(&right);
    if left.len() + right.len() > 256 {
        return Err(invalid());
    }
    let mut product = vec![0u16; left.len() + right.len()];
    for (li, ld) in left.bytes().rev().enumerate() {
        for (ri, rd) in right.bytes().rev().enumerate() {
            let index = product.len() - 1 - li - ri;
            product[index] += u16::from(ld - b'0') * u16::from(rd - b'0');
        }
    }
    for index in (1..product.len()).rev() {
        let carry = product[index] / 10;
        product[index] %= 10;
        product[index - 1] += carry;
    }
    let digits = product
        .into_iter()
        .map(|digit| char::from(b'0' + digit as u8))
        .collect::<String>();
    let scale = left_scale + right_scale;
    let padded = if digits.len() <= scale {
        format!("{}{}", "0".repeat(scale + 1 - digits.len()), digits)
    } else {
        digits
    };
    let result = if scale == 0 {
        padded
    } else {
        format!(
            "{}.{}",
            &padded[..padded.len() - scale],
            &padded[padded.len() - scale..]
        )
    };
    decimal(&Value::String(result)).map_err(|_| invalid())
}

fn validate_exchange_filters(
    symbol: &Value,
    proposal: &OrderProposal,
    reference_price: Option<&str>,
) -> Result<()> {
    let fields = &proposal.fields;
    let amount = decimal(&Value::String(fields.quantity.value.clone()))?;
    let is_market = fields.order_type == OrderType::Market;
    if fields.quantity.r#type == OrderQuantityType::Base {
        let size = filter(
            symbol,
            if is_market {
                "MARKET_LOT_SIZE"
            } else {
                "LOT_SIZE"
            },
        )?;
        check_range_step(&amount, &size["minQty"], &size["maxQty"], &size["stepSize"])?;
    }
    if !is_market {
        let price = decimal(&Value::String(
            fields.limit_price.clone().ok_or_else(invalid)?,
        ))?;
        let price_filter = filter(symbol, "PRICE_FILTER")?;
        check_range_step(
            &price,
            &price_filter["minPrice"],
            &price_filter["maxPrice"],
            &price_filter["tickSize"],
        )?;
        let mut seen = HashSet::new();
        for price_filter in symbol["filters"].as_array().ok_or_else(invalid)? {
            let kind = price_filter["filterType"].as_str().ok_or_else(invalid)?;
            let (down, up) = match kind {
                "PERCENT_PRICE" => (
                    &price_filter["multiplierDown"],
                    &price_filter["multiplierUp"],
                ),
                "PERCENT_PRICE_BY_SIDE" => match fields.side {
                    OrderSide::Buy => (
                        &price_filter["bidMultiplierDown"],
                        &price_filter["bidMultiplierUp"],
                    ),
                    OrderSide::Sell => (
                        &price_filter["askMultiplierDown"],
                        &price_filter["askMultiplierUp"],
                    ),
                },
                _ => continue,
            };
            if !seen.insert(kind) {
                return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
            }
            let reference =
                reference_price.ok_or_else(|| TradeXError::new("ORDER_FILTERS_UNAVAILABLE"))?;
            let minimum = multiply(reference, &positive(down)?)?;
            let maximum = multiply(reference, &positive(up)?)?;
            if decimal_cmp(&price, &minimum)? == std::cmp::Ordering::Less
                || decimal_cmp(&price, &maximum)? == std::cmp::Ordering::Greater
            {
                return Err(TradeXError::new("ORDER_FILTER_REJECTED"));
            }
        }
    }
    let notionals = symbol["filters"]
        .as_array()
        .ok_or_else(invalid)?
        .iter()
        .filter(|value| {
            matches!(
                value["filterType"].as_str(),
                Some("MIN_NOTIONAL" | "NOTIONAL")
            )
        })
        .collect::<Vec<_>>();
    if notionals.is_empty() {
        return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
    }
    let notional = match fields.quantity.r#type {
        OrderQuantityType::Quote => amount,
        OrderQuantityType::Base => multiply(
            &amount,
            if is_market {
                reference_price.ok_or_else(|| TradeXError::new("ORDER_FILTERS_UNAVAILABLE"))?
            } else {
                fields.limit_price.as_deref().ok_or_else(invalid)?
            },
        )?,
    };
    for filter in notionals {
        let min_enabled = !is_market
            || if filter["filterType"] == "MIN_NOTIONAL" {
                filter["applyToMarket"].as_bool().ok_or_else(invalid)?
            } else {
                filter["applyMinToMarket"].as_bool().ok_or_else(invalid)?
            };
        let max_enabled = filter["filterType"] == "NOTIONAL"
            && (!is_market || filter["applyMaxToMarket"].as_bool().ok_or_else(invalid)?);
        let min = positive(&filter["minNotional"])?;
        if min_enabled && min != "0" && decimal_cmp(&notional, &min)? == std::cmp::Ordering::Less {
            return Err(TradeXError::new("ORDER_FILTER_REJECTED"));
        }
        if max_enabled {
            let max = positive(&filter["maxNotional"])?;
            if max != "0" && decimal_cmp(&notional, &max)? == std::cmp::Ordering::Greater {
                return Err(TradeXError::new("ORDER_FILTER_REJECTED"));
            }
        }
    }
    if let Some(maximum_spend) = fields.maximum_spend.as_deref()
        && decimal_cmp(&notional, maximum_spend)? == std::cmp::Ordering::Greater
    {
        return Err(TradeXError::new("ORDER_BUYING_POWER_INSUFFICIENT"));
    }
    Ok(())
}

fn reference_price(
    http: &impl ProviderHttp,
    symbol: &str,
    exchange_symbol: &Value,
    proposal: &OrderProposal,
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<String> {
    let filters = exchange_symbol["filters"].as_array().ok_or_else(invalid)?;
    let is_market = proposal.fields.order_type == OrderType::Market;
    let mut avg_minutes = None;
    let mut conflicting_intervals = false;
    let mut seen = HashSet::new();
    for item in filters.iter().filter(|item| {
        matches!(
            item["filterType"].as_str(),
            Some("MIN_NOTIONAL" | "NOTIONAL") if is_market
        ) || matches!(
            item["filterType"].as_str(),
            Some("PERCENT_PRICE" | "PERCENT_PRICE_BY_SIDE") if !is_market
        )
    }) {
        let kind = item["filterType"].as_str().ok_or_else(invalid)?;
        if !seen.insert(kind) {
            return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
        }
        let value = item["avgPriceMins"]
            .as_u64()
            .ok_or_else(|| TradeXError::new("ORDER_FILTERS_UNAVAILABLE"))?;
        conflicting_intervals |= avg_minutes
            .replace(value)
            .is_some_and(|existing| existing != value);
    }
    let reference_path = format!("/api/v3/referencePrice?symbol={symbol}");
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let response = http.request(
        ProviderEndpoint::BinanceTestnet,
        ProviderHttpMethod::Get,
        &reference_path,
        HeaderMap::new(),
        None,
    )?;
    if response.status == 200 {
        let reference = response_value(response, secrets)?;
        if reference["symbol"].as_str() != Some(symbol)
            || !reference["timestamp"].as_u64().is_some_and(valid_time)
        {
            return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
        }
        match reference.get("referencePrice") {
            Some(Value::Null) => (),
            Some(Value::String(price)) => {
                let price = positive(&Value::String(price.clone()))?;
                if price == "0" {
                    return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
                }
                return Ok(price);
            }
            _ => return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE")),
        }
    } else {
        return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
    }
    if conflicting_intervals {
        return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
    }
    let avg_minutes = avg_minutes.ok_or_else(|| TradeXError::new("ORDER_FILTERS_UNAVAILABLE"))?;
    let endpoint = if avg_minutes == 0 {
        "ticker/price"
    } else {
        "avgPrice"
    };
    let path = format!("/api/v3/{endpoint}?symbol={symbol}");
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let response = http.request(
        ProviderEndpoint::BinanceTestnet,
        ProviderHttpMethod::Get,
        &path,
        HeaderMap::new(),
        None,
    )?;
    if response.status != 200 {
        return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
    }
    let response = response_value(response, secrets)?;
    if avg_minutes > 0 && response["mins"].as_u64() != Some(avg_minutes) {
        return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
    }
    positive(&response["price"])
}

fn account_balance(account: &Value, asset: &str) -> Result<String> {
    let balances = account["balances"].as_array().ok_or_else(invalid)?;
    let mut rows = balances
        .iter()
        .filter(|row| row["asset"].as_str() == Some(asset));
    let row = rows
        .next()
        .ok_or_else(|| TradeXError::new("ORDER_BALANCE_UNAVAILABLE"))?;
    if rows.next().is_some() {
        return Err(invalid());
    }
    positive(&row["free"])
}

fn check_available_balance(
    account: &Value,
    proposal: &OrderProposal,
    symbol: &Value,
    amount: &str,
    notional: &str,
) -> Result<()> {
    let fields = &proposal.fields;
    let asset =
        if fields.side == OrderSide::Sell && fields.quantity.r#type == OrderQuantityType::Base {
            symbol["baseAsset"].as_str().ok_or_else(invalid)?
        } else {
            symbol["quoteAsset"].as_str().ok_or_else(invalid)?
        };
    let needed =
        if fields.side == OrderSide::Sell && fields.quantity.r#type == OrderQuantityType::Base {
            amount
        } else {
            notional
        };
    if decimal_cmp(needed, &account_balance(account, asset)?)? == std::cmp::Ordering::Greater {
        return Err(TradeXError::new("ORDER_BALANCE_INSUFFICIENT"));
    }
    Ok(())
}

fn parse_order_id(value: &Value) -> Result<String> {
    let id = match value.get("orderId") {
        Some(Value::String(id))
            if !id.is_empty() && id.len() <= 20 && id.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            id.clone()
        }
        Some(Value::Number(id)) => id
            .as_u64()
            .filter(|id| *id > 0 && *id <= i64::MAX as u64)
            .map(|id| id.to_string())
            .ok_or_else(invalid)?,
        _ => return Err(invalid()),
    };
    if id
        .parse::<u64>()
        .is_ok_and(|id| id > 0 && id <= i64::MAX as u64)
    {
        Ok(id)
    } else {
        Err(invalid())
    }
}

fn acknowledge(
    value: &Value,
    attempt: &mut BinanceTestnetOrderAttempt,
    proposal: &OrderProposal,
    symbol: &str,
) -> Result<()> {
    let fields = &proposal.fields;
    if text(value, "symbol", 32)? != symbol
        || text(value, "clientOrderId", 36)? != attempt.client_order_id
        || text(value, "side", 4)?
            != match fields.side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            }
        || text(value, "type", 8)?
            != match fields.order_type {
                OrderType::Market => "MARKET",
                OrderType::Limit => "LIMIT",
            }
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
    }
    match fields.quantity.r#type {
        OrderQuantityType::Base
            if positive(&value["origQty"])?
                != decimal(&Value::String(fields.quantity.value.clone()))? =>
        {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
        }
        OrderQuantityType::Quote
            if positive(&value["origQuoteOrderQty"])?
                != decimal(&Value::String(fields.quantity.value.clone()))? =>
        {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
        }
        _ => (),
    }
    if fields.order_type == OrderType::Limit {
        if positive(&value["price"])?
            != decimal(&Value::String(
                fields.limit_price.clone().ok_or_else(invalid)?,
            ))?
            || text(value, "timeInForce", 3)?
                != match fields.time_in_force {
                    TimeInForce::Gtc => "GTC",
                    TimeInForce::Ioc => "IOC",
                    TimeInForce::Fok => "FOK",
                    TimeInForce::Day => return Err(invalid()),
                }
        {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
        }
    }
    attempt.provider_order_id = Some(parse_order_id(value)?);
    attempt.provider_status = Some(text(value, "status", 32)?);
    attempt.state = BinanceTestnetOrderAttemptState::Acknowledged;
    attempt.error_code = None;
    attempt.reason =
        "Binance Spot Testnet acknowledged the order. This response is not fill evidence.".into();
    Ok(())
}

fn query_by_client_id(
    http: &impl ProviderHttp,
    attempt: &mut BinanceTestnetOrderAttempt,
    proposal: &OrderProposal,
    symbol: &str,
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<bool> {
    let (server, sampled) = server_time(http, current)?;
    let params = [
        ("symbol", symbol),
        ("origClientOrderId", attempt.client_order_id.as_str()),
    ];
    let response = signed_request(
        http,
        ProviderHttpMethod::Get,
        "/api/v3/order",
        &params,
        secrets,
        server,
        sampled,
        current,
    )?;
    if response.status == 400 {
        let error: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
        if error["code"].as_i64() == Some(-2013) {
            return Ok(false);
        }
        return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
    }
    if response.status != 200 {
        return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
    }
    let value = response_value(response, secrets)?;
    acknowledge(&value, attempt, proposal, symbol)?;
    Ok(true)
}

pub(super) fn run_testnet_order(
    attempt: &BinanceTestnetOrderAttempt,
    proposal: &OrderProposal,
    reconcile: bool,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> BinanceTestnetOrderAttempt {
    let mut attempt = attempt.clone();
    let mut post_may_have_been_sent = false;
    let result = (|| -> Result<BinanceTestnetOrderAttempt> {
        if secrets.len() != 2
            || attempt.workspace_id != proposal.workspace_id
            || attempt.proposal_id != proposal.proposal_id
            || attempt.proposal_hash != proposal.proposal_hash
            || proposal.workspace_id.is_empty()
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        validate_binance_testnet_proposal(proposal, &attempt.connection_id)?;
        let symbol = provider_symbol(proposal, &attempt.connection_id)?;
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let (server, sampled) = server_time(http, current)?;
        let account_response = signed_request(
            http,
            ProviderHttpMethod::Get,
            "/api/v3/account",
            &[],
            secrets,
            server,
            sampled,
            current,
        )?;
        if account_response.status != 200 {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        let account = response_value(account_response, secrets)?;
        if text(&account, "accountType", 16)? != "SPOT"
            || numeric_id(&account, "uid")? != attempt.remote_account_id
            || !flag(&account, "canTrade")?
        {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
        }
        if reconcile {
            if attempt.state != BinanceTestnetOrderAttemptState::UnknownReconciling {
                return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
            }
            let _ = query_by_client_id(http, &mut attempt, proposal, &symbol, secrets, current);
            if attempt.state != BinanceTestnetOrderAttemptState::Acknowledged {
                attempt.state = BinanceTestnetOrderAttemptState::UnknownReconciling;
                attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
                attempt.reason =
                    "Binance has not confirmed the saved client order ID. Do not resubmit.".into();
            }
            return Ok(attempt.clone());
        }
        let exchange_path = format!("/api/v3/exchangeInfo?symbol={symbol}");
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let exchange_response = http.request(
            ProviderEndpoint::BinanceTestnet,
            ProviderHttpMethod::Get,
            &exchange_path,
            HeaderMap::new(),
            None,
        )?;
        if exchange_response.status != 200 {
            return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
        }
        let exchange = response_value(exchange_response, secrets)?;
        let instrument = market::instruments()
            .into_iter()
            .find(|instrument| instrument.instrument_id == proposal.fields.instrument_id)
            .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))?;
        let exchange_symbol = parse_symbol(
            &exchange,
            &symbol,
            instrument.base.as_deref().ok_or_else(invalid)?,
            instrument.quote.as_deref().ok_or_else(invalid)?,
        )?;
        let needs_reference = (proposal.fields.quantity.r#type == OrderQuantityType::Base
            && proposal.fields.order_type == OrderType::Market)
            || (proposal.fields.order_type == OrderType::Limit
                && exchange_symbol["filters"]
                    .as_array()
                    .is_some_and(|filters| {
                        filters.iter().any(|filter| {
                            matches!(
                                filter["filterType"].as_str(),
                                Some("PERCENT_PRICE" | "PERCENT_PRICE_BY_SIDE")
                            )
                        })
                    }));
        let reference = if needs_reference {
            Some(reference_price(
                http,
                &symbol,
                &exchange_symbol,
                proposal,
                secrets,
                current,
            )?)
        } else {
            None
        };
        validate_exchange_filters(&exchange_symbol, proposal, reference.as_deref())?;
        let amount = decimal(&Value::String(proposal.fields.quantity.value.clone()))?;
        let notional = match proposal.fields.quantity.r#type {
            OrderQuantityType::Quote => amount.clone(),
            OrderQuantityType::Base => multiply(
                &amount,
                if proposal.fields.order_type == OrderType::Market {
                    reference.as_deref().ok_or_else(invalid)?
                } else {
                    proposal.fields.limit_price.as_deref().ok_or_else(invalid)?
                },
            )?,
        };
        check_available_balance(&account, proposal, &exchange_symbol, &amount, &notional)?;
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let mut params = vec![
            ("symbol", symbol.as_str()),
            (
                "side",
                if proposal.fields.side == OrderSide::Buy {
                    "BUY"
                } else {
                    "SELL"
                },
            ),
            (
                "type",
                if proposal.fields.order_type == OrderType::Market {
                    "MARKET"
                } else {
                    "LIMIT"
                },
            ),
        ];
        let amount_key = if proposal.fields.quantity.r#type == OrderQuantityType::Quote {
            "quoteOrderQty"
        } else {
            "quantity"
        };
        params.push((amount_key, proposal.fields.quantity.value.as_str()));
        if proposal.fields.order_type == OrderType::Limit {
            params.push((
                "price",
                proposal.fields.limit_price.as_deref().ok_or_else(invalid)?,
            ));
            params.push((
                "timeInForce",
                match proposal.fields.time_in_force {
                    TimeInForce::Gtc => "GTC",
                    TimeInForce::Ioc => "IOC",
                    TimeInForce::Fok => "FOK",
                    TimeInForce::Day => {
                        return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED"));
                    }
                },
            ));
        }
        params.push(("newClientOrderId", attempt.client_order_id.as_str()));
        params.push(("newOrderRespType", "RESULT"));
        let (server, sampled) = server_time(http, current)?;
        post_may_have_been_sent = true;
        match signed_request(
            http,
            ProviderHttpMethod::Post,
            "/api/v3/order",
            &params,
            secrets,
            server,
            sampled,
            current,
        ) {
            Ok(response) if (200..300).contains(&response.status) => {
                match response_value(response, secrets)
                    .and_then(|value| acknowledge(&value, &mut attempt, proposal, &symbol))
                {
                    Ok(()) => Ok(attempt.clone()),
                    Err(_) => {
                        let _ = query_by_client_id(
                            http,
                            &mut attempt,
                            proposal,
                            &symbol,
                            secrets,
                            current,
                        );
                        if attempt.state != BinanceTestnetOrderAttemptState::Acknowledged {
                            attempt.state = BinanceTestnetOrderAttemptState::UnknownReconciling;
                            attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
                            attempt.reason = "The response could not confirm order identity. Query the saved client order ID; do not resubmit.".into();
                        }
                        Ok(attempt.clone())
                    }
                }
            }
            Ok(response) if matches!(response.status, 400 | 401 | 403 | 418 | 429) => {
                let error: Value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);
                attempt.state = BinanceTestnetOrderAttemptState::Rejected;
                attempt.error_code = Some(
                    match error["code"].as_i64() {
                        Some(-1013) => "ORDER_FILTER_REJECTED",
                        Some(-2010) => "ORDER_PROVIDER_REJECTED",
                        Some(-1022 | -2014 | -2015) => "PROVIDER_AUTH_FAILED",
                        Some(-1003 | -1015) => "PROVIDER_RATE_LIMITED",
                        _ if response.status == 401 || response.status == 403 => {
                            "PROVIDER_AUTH_FAILED"
                        }
                        _ if response.status == 418 || response.status == 429 => {
                            "PROVIDER_RATE_LIMITED"
                        }
                        _ => "PROVIDER_ORDER_REJECTED",
                    }
                    .into(),
                );
                attempt.reason = "Binance rejected the Testnet order request. Review the Proposal before creating another.".into();
                Ok(attempt.clone())
            }
            Ok(_) | Err(_) => {
                let _ = query_by_client_id(http, &mut attempt, proposal, &symbol, secrets, current);
                if attempt.state != BinanceTestnetOrderAttemptState::Acknowledged {
                    attempt.state = BinanceTestnetOrderAttemptState::UnknownReconciling;
                    attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
                    attempt.reason = "The submission result is unknown. Query the saved client order ID; do not resubmit.".into();
                }
                Ok(attempt.clone())
            }
        }
    })();
    match result {
        Ok(result) => result,
        Err(error) => {
            attempt.state = if post_may_have_been_sent {
                BinanceTestnetOrderAttemptState::UnknownReconciling
            } else {
                BinanceTestnetOrderAttemptState::Rejected
            };
            attempt.error_code = Some(
                if post_may_have_been_sent {
                    "ORDER_STATUS_UNKNOWN"
                } else {
                    error.code.as_str()
                }
                .into(),
            );
            attempt.reason = if post_may_have_been_sent {
                "The submission result is unknown. Query the saved client order ID; do not resubmit.".into()
            } else {
                error.message
            };
            attempt
        }
    }
}
fn signature(query: &str, secret: &str) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| invalid())?;
    mac.update(query.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub(super) fn read(
    endpoint: ProviderEndpoint,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    old: Option<&AccountData>,
) -> Result<Observation> {
    let query = |path: &str, auth: HeaderMap| -> Result<Value> {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let bytes = http.get(endpoint, path, auth)?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err(invalid());
        }
        let value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if contains_secret(&value, secrets)
            || path
                .split_once("&signature=")
                .is_some_and(|(_, sig)| contains_secret(&value, &[sig.into()]))
        {
            return Err(invalid());
        }
        Ok(value)
    };
    let started = Instant::now();
    let time = query("/api/v3/time", HeaderMap::new())?;
    let server_time = time["serverTime"]
        .as_u64()
        .filter(|n| valid_time(*n))
        .ok_or_else(time_error)?;
    if started.elapsed() > Duration::from_secs(2) {
        return Err(time_error());
    }
    let sampled = Instant::now();
    let mut key = HeaderValue::from_str(&secrets[0])
        .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
    key.set_sensitive(true);
    let mut auth = HeaderMap::new();
    auth.insert("X-MBX-APIKEY", key);
    let signed = |route: &str| -> Result<Value> {
        let elapsed = sampled.elapsed();
        if elapsed > Duration::from_secs(60) {
            return Err(time_error());
        }
        let timestamp = server_time
            .checked_add(elapsed.as_millis() as u64)
            .filter(|n| valid_time(*n))
            .ok_or_else(time_error)?;
        let params = format!("timestamp={timestamp}&recvWindow=5000");
        let sig = Zeroizing::new(signature(&params, &secrets[1])?);
        let path = Zeroizing::new(format!("{route}?{params}&signature={}", *sig));
        query(&path, auth.clone())
    };
    let account = signed("/api/v3/account")?;
    let identity = numeric_id(&account, "uid")?;
    if old.is_some_and(|old| old.remote_account_id != identity) {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let restrictions = if endpoint == ProviderEndpoint::BinanceLive {
        Some(signed("/sapi/v1/account/apiRestrictions")?)
    } else {
        None
    };
    let result = observe(account, signed("/api/v3/openOrders")?, restrictions)?;
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    Ok(result)
}
fn numeric_id(v: &Value, field: &str) -> Result<String> {
    v[field]
        .as_u64()
        .filter(|n| *n > 0 && *n <= i64::MAX as u64)
        .map(|n| n.to_string())
        .ok_or_else(invalid)
}
const FLAGS: &[(&str, &str)] = &[
    ("enableReading", "read"),
    ("enableWithdrawals", "withdrawal"),
    ("enableInternalTransfer", "transfer.internal"),
    ("permitsUniversalTransfer", "transfer.universal"),
    ("enableMargin", "margin"),
    ("enableFutures", "futures"),
    ("enableVanillaOptions", "options"),
    ("enablePortfolioMarginTrading", "portfolio.margin"),
    ("enableFixApiTrade", "fix.trade"),
    ("enableFixReadOnly", "fix.read"),
    ("enableSpotAndMarginTrading", "spot-and-margin.trade"),
];
fn permissions(restrictions: Option<Value>) -> Result<PermissionReview> {
    let mut p = PermissionReview {
        detected: vec![
            "account.read".into(),
            "positions.read".into(),
            "orders.read".into(),
        ],
        ..Default::default()
    };
    let Some(v) = restrictions else {
        return Ok(p);
    };
    let fields = v.as_object().ok_or_else(invalid)?;
    let mut complete = true;
    for (key, name) in FLAGS {
        match fields.get(*key) {
            Some(Value::Bool(enabled)) => {
                if !enabled {
                    if *key == "enableReading" {
                        p.unsupported.push("reading.disabled".into());
                    }
                    continue;
                }
                p.detected.push((*name).into());
                if matches!(
                    *key,
                    "enableWithdrawals" | "enableInternalTransfer" | "permitsUniversalTransfer"
                ) {
                    p.forbidden.push((*name).into());
                }
                if matches!(
                    *key,
                    "enableMargin"
                        | "enableFutures"
                        | "enableVanillaOptions"
                        | "enablePortfolioMarginTrading"
                        | "enableFixApiTrade"
                ) {
                    p.unsupported.push((*name).into());
                }
            }
            None | Some(Value::Null) => complete = false,
            _ => return Err(invalid()),
        }
    }
    p.ip_allow_list_status = match fields.get("ipRestrict") {
        Some(Value::Bool(true)) => "RESTRICTED",
        Some(Value::Bool(false)) => "UNRESTRICTED",
        None | Some(Value::Null) => {
            complete = false;
            "UNKNOWN"
        }
        _ => return Err(invalid()),
    }
    .into();
    for (key, _) in fields {
        if key != "createTime"
            && key != "ipRestrict"
            && !FLAGS.iter().any(|(known, _)| key == known)
        {
            complete = false;
        }
    }
    if complete {
        p.scope = "VERIFIED".into();
    }
    Ok(p)
}
fn observe(account: Value, orders: Value, restrictions: Option<Value>) -> Result<Observation> {
    if account["accountType"] != "SPOT" {
        return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
    }
    let permissions = permissions(restrictions)?;
    let mut limitations=vec![
        "Spot holdings are free plus locked assets, without cost basis or FX valuation. USDT is not USD.".into(),
        "Order quote currency is unavailable until instrument metadata is resolved. Amounts are not workspace-currency values.".into(),
        "Private stream, reconciliation, risk policy and execution are not configured. Manual full open-order reads consume weight 80; rate limits require a later retry.".into(),
    ];
    if permissions.scope == "UNVERIFIED" {
        limitations.push("API-key scope is incomplete or unavailable (Testnet has no /sapi). Successful reads and account canWithdraw do not verify key permissions.".into());
    }
    for (key, label) in [
        ("canTrade", "trading"),
        ("canWithdraw", "withdrawals"),
        ("canDeposit", "deposits"),
    ] {
        if !flag(&account, key)? {
            limitations.push(format!("The account reports {label} disabled; this is an account capability, not API-key scope."));
        }
    }
    let balances = account["balances"].as_array().ok_or_else(invalid)?;
    let orders = orders.as_array().ok_or_else(invalid)?;
    if balances.len() > 10_000 || orders.len() > 10_000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let mut assets = HashSet::new();
    let mut positions = vec![];
    let balances = balances
        .iter()
        .map(|b| {
            let asset = identifier(b, "asset")?;
            if !assets.insert(asset.clone()) {
                return Err(invalid());
            }
            let free = positive(&b["free"])?;
            let locked = positive(&b["locked"])?;
            let sum = total(&free, &locked)?;
            if sum != "0" {
                positions.push(Position {
                    symbol: asset.clone(),
                    instrument_id: None,
                    quantity: sum.clone(),
                    market_value: None,
                    average_entry_price: None,
                    instrument_currency: None,
                    market_value_currency: None,
                });
            }
            Ok(Balance {
                locked: None,
                restricted_available: None,
                asset,
                available: free,
                total: Some(sum),
                reserved: Some(locked),
                in_pies: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut ids = HashSet::new();
    let orders = orders
        .iter()
        .map(|o| {
            let symbol = identifier(o, "symbol")?;
            let broker_order_id = format!("{symbol}:{}", numeric_id(o, "orderId")?);
            if !ids.insert(broker_order_id.clone()) {
                return Err(invalid());
            }
            let side = text(o, "side", 4)?;
            if !matches!(side.as_str(), "BUY" | "SELL") {
                return Err(invalid());
            }
            let status = text(o, "status", 32)?;
            if !matches!(
                status.as_str(),
                "NEW" | "PENDING_NEW" | "PARTIALLY_FILLED" | "PENDING_CANCEL"
            ) {
                return Err(invalid());
            }
            let price = positive(&o["price"])?;
            let qty = positive(&o["origQty"])?;
            let quote = positive(&o["origQuoteOrderQty"])?;
            if qty == "0" && quote == "0" {
                return Err(invalid());
            }
            Ok(OpenOrder {
                kind: None,
                trigger_price: None,
                broker_order_id,
                symbol,
                instrument_id: None,
                side,
                quantity: if qty == "0" { None } else { Some(qty) },
                notional: if quote == "0" { None } else { Some(quote) },
                filled_quantity: Some(positive(&o["executedQty"])?),
                filled_value: None,
                currency: None,
                status,
                limit_price: if price == "0" { None } else { Some(price) },
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Observation {
        data: AccountData {
            remote_account_id: numeric_id(&account, "uid")?,
            account_type: "BINANCE_SPOT".into(),
            currency: None,
            buying_power: None,
            balances,
            positions,
            open_orders: orders,
            capabilities: vec![
                "account.read".into(),
                "positions.read".into(),
                "orders.read".into(),
            ],
            limitations,
        },
        permissions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream_book() -> BinanceTestnetOrderBook {
        serde_json::from_value(json!({
            "workspaceId": "workspace-test",
            "connectionId": "connection-test",
            "remoteAccountId": "42",
            "environment": "TESTNET",
            "status": "CURRENT",
            "stateVersion": "book-v1",
            "lastSuccessfulSyncAt": "2026-09-24T10:00:00Z",
            "pendingOrdersObservedAt": "2026-09-24T10:00:00Z",
            "balancesObservedAt": "2026-09-24T10:00:00Z",
            "privateStreamBalanceUpdateAtMs": null,
            "observedAt": "2026-09-24T10:00:00Z",
            "reason": null,
            "rateLimits": {
                "pendingOrdersRetryAt": null,
                "accountRetryAt": null,
                "historyRetryAt": null,
                "orderDetailRetryAt": null
            },
            "history": [
                {"symbol":"BTCUSDT","started":false,"complete":false,"pageCount":0,"lastObservedAt":null,"nextOrderId":null,"nextTradeId":null},
                {"symbol":"ETHUSDT","started":false,"complete":false,"pageCount":0,"lastObservedAt":null,"nextOrderId":null,"nextTradeId":null}
            ],
            "orders": [],
            "fills": [],
            "balances": []
        }))
        .unwrap()
    }

    fn execution_report(
        execution: &str,
        status: &str,
        filled: &str,
        quote: &str,
        event_time: u64,
        update_time: u64,
        trade_id: i64,
        last_quantity: &str,
    ) -> Value {
        json!({
            "subscriptionId": 7,
            "event": {
                "e":"executionReport", "E":event_time, "s":"BTCUSDT", "c":"fixture-client-order",
                "S":"BUY", "o":"LIMIT", "f":"GTC", "q":"0.25", "p":"90",
                "x":execution, "X":status, "i":9007199254740995u64,
                "l":last_quantity, "z":filled, "L":"90", "n":"0", "N":"USDT",
                "O":1_788_849_600_000u64, "T":update_time, "t":trade_id, "Q":"0", "Y":quote, "Z":quote
            }
        })
    }

    #[test]
    fn signing_decimal_and_route_bounds_match_external_contracts() {
        // RFC 4231 test case 1, independent of the integration fixture verifier.
        assert_eq!(
            signature("Hi There", &"\u{b}".repeat(20)).unwrap(),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            total("0.999999999999999999999", "0.000000000000000000001").unwrap(),
            "1"
        );
        assert!(total(&"9".repeat(128), "1").is_err());
        assert!(positive(&Value::String("-0.000001".into())).is_err());
        for endpoint in [
            ProviderEndpoint::BinanceTestnet,
            ProviderEndpoint::BinanceLive,
        ] {
            for route in ["/api/v3/account", "/api/v3/openOrders"] {
                let path = format!(
                    "{route}?timestamp=1788849600000&recvWindow=5000&signature={}",
                    "a".repeat(64)
                );
                assert!(allows(endpoint, &path));
                for invalid in [
                    path.replace("recvWindow=5000", "recvWindow=60000"),
                    format!("{path}&symbol=BTCUSDT"),
                    path.replace("1788849600000", "1788849600000000"),
                    path.replace(route, "/api/v3/order"),
                    path.replace(route, "/sapi/v1/asset/transfer"),
                    path.replace(route, "/fapi/v1/account"),
                ] {
                    assert!(!allows(endpoint, &invalid));
                    assert_eq!(
                        BrokerHttp::default()
                            .get(endpoint, &invalid, HeaderMap::new())
                            .unwrap_err()
                            .code,
                        "PROVIDER_UNSUPPORTED"
                    );
                }
            }
        }
        let restriction = format!(
            "/sapi/v1/account/apiRestrictions?timestamp=1788849600000&recvWindow=5000&signature={}",
            "a".repeat(64)
        );
        let reference_price = "/api/v3/referencePrice?symbol=BTCUSDT";
        assert!(allows(ProviderEndpoint::BinanceTestnet, reference_price));
        assert!(!allows(ProviderEndpoint::BinanceLive, reference_price));
        assert!(!allows(
            ProviderEndpoint::BinanceTestnet,
            "/api/v3/referencePrice?symbol=BNBUSDT"
        ));
        assert!(!allows(ProviderEndpoint::BinanceTestnet, &restriction));
        assert!(allows(ProviderEndpoint::BinanceLive, &restriction));
    }

    #[test]
    fn private_stream_deduplicates_fills_and_ignores_late_order_state() {
        let mut book = stream_book();
        let secrets = vec!["fixture-key".into(), "fixture-secret".into()];
        let fill = execution_report(
            "TRADE",
            "PARTIALLY_FILLED",
            "0.1",
            "9",
            1_788_849_700_000,
            1_788_849_700_000,
            9001,
            "0.1",
        );

        let update = apply_testnet_private_stream_frame(&mut book, &fill, 7, &secrets)
            .unwrap()
            .unwrap();
        assert!(!update.reconciliation_required);
        assert_eq!(book.orders.len(), 1);
        assert_eq!(book.orders[0].provider_status, "PARTIALLY_FILLED");
        assert!(book.orders[0].pending);
        assert_eq!(book.orders[0].filled_quantity, "0.1");
        assert_eq!(book.fills.len(), 1);
        assert!(
            apply_testnet_private_stream_frame(&mut book, &fill, 7, &secrets)
                .unwrap()
                .is_none()
        );

        let late = execution_report(
            "NEW",
            "NEW",
            "0",
            "0",
            1_788_849_699_000,
            1_788_849_699_000,
            -1,
            "0",
        );
        assert!(
            apply_testnet_private_stream_frame(&mut book, &late, 7, &secrets)
                .unwrap()
                .is_none()
        );
        assert_eq!(book.orders.len(), 1);
        assert_eq!(book.orders[0].provider_status, "PARTIALLY_FILLED");
        assert_eq!(book.orders[0].filled_quantity, "0.1");
        assert_eq!(book.fills.len(), 1);
    }

    #[test]
    fn private_stream_merges_account_assets_and_requires_reconciliation_for_deltas() {
        let mut book = stream_book();
        let secrets = vec!["fixture-key".into(), "fixture-secret".into()];
        for (updated, asset, free) in [(200, "USDT", "10"), (201, "BTC", "0.5")] {
            let frame = json!({
                "subscriptionId":7,
                "event":{"e":"outboundAccountPosition","E":1_788_849_700_000u64 + updated,"u":updated,
                    "B":[{"a":asset,"f":free,"l":"0"}]}
            });
            assert!(
                !apply_testnet_private_stream_frame(&mut book, &frame, 7, &secrets)
                    .unwrap()
                    .unwrap()
                    .reconciliation_required
            );
        }
        assert_eq!(book.balances.len(), 2);
        assert_eq!(book.private_stream_balance_update_at_ms, Some(201));
        let stale = json!({"subscriptionId":7,"event":{"e":"outboundAccountPosition","E":1_788_849_699_000u64,"u":199,"B":[{"a":"USDT","f":"1","l":"0"}]}});
        assert!(
            apply_testnet_private_stream_frame(&mut book, &stale, 7, &secrets)
                .unwrap()
                .is_none()
        );
        assert_eq!(book.balances[1].free, "10");

        let before = book.balances.clone();
        let delta = json!({"subscriptionId":7,"event":{"e":"balanceUpdate","E":1_788_849_702_000u64,"T":1_788_849_702_000u64,"a":"USDT","d":"-2.5"}});
        let update = apply_testnet_private_stream_frame(&mut book, &delta, 7, &secrets)
            .unwrap()
            .unwrap();
        assert!(update.reconciliation_required);
        assert_eq!(book.status, BinanceTestnetOrderBookStatus::Stale);
        assert_eq!(book.balances, before);
    }

    #[test]
    fn private_stream_rejects_wrong_subscription_and_secret_reflection() {
        let mut book = stream_book();
        let secrets = vec!["fixture-key".into(), "fixture-secret".into()];
        let valid = execution_report(
            "NEW",
            "MYSTERY_STATUS",
            "0",
            "0",
            1_788_849_700_000,
            1_788_849_700_000,
            -1,
            "0",
        );
        assert_eq!(
            apply_testnet_private_stream_frame(&mut book, &valid, 8, &secrets)
                .err()
                .unwrap()
                .code,
            "PROVIDER_RESPONSE_INVALID"
        );
        let reflected = json!({"subscriptionId":7,"event":{"e":"notice","E":1_788_849_700_000u64,"message":"fixture-secret"}});
        assert_eq!(
            apply_testnet_private_stream_frame(&mut book, &reflected, 7, &secrets)
                .err()
                .unwrap()
                .code,
            "PROVIDER_RESPONSE_INVALID"
        );

        let unknown = execution_report(
            "NEW",
            "MYSTERY_STATUS",
            "0",
            "0",
            1_788_849_700_000,
            1_788_849_700_000,
            -1,
            "0",
        );
        apply_testnet_private_stream_frame(&mut book, &unknown, 7, &secrets).unwrap();
        assert_eq!(book.orders[0].provider_status, "MYSTERY_STATUS");
        assert!(book.orders[0].pending);
    }
}
