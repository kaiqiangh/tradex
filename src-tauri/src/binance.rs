use super::*;
use crate::protocol::{BinanceTestnetOrderAttempt, BinanceTestnetOrderAttemptState, OrderProposal};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    collections::{BTreeMap, HashSet},
    time::Instant,
};

fn time_error() -> TradeXError {
    TradeXError::new("CLOCK_SKEW")
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
        "/api/v3/account" | "/api/v3/openOrders" | "/api/v3/order"
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
        "/api/v3/order" => {
            let symbol = values
                .get("symbol")
                .is_some_and(|value| matches!(*value, "BTCUSDT" | "ETHUSDT"));
            let query_order = keys
                .iter()
                .all(|key| matches!(*key, "symbol" | "origClientOrderId"))
                && symbol
                && values
                    .get("origClientOrderId")
                    .is_some_and(|value| valid_client_order_id(value));
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
            query_order || new_order
        }
        "/sapi/v1/account/apiRestrictions" => {
            endpoint == ProviderEndpoint::BinanceLive && keys.is_empty()
        }
        _ => false,
    }
}

fn valid_client_order_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 36
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
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
    http.request(
        ProviderEndpoint::BinanceTestnet,
        method,
        &path,
        headers,
        None,
    )
}

fn server_time(http: &impl ProviderHttp, current: &impl Fn() -> bool) -> Result<(u64, Instant)> {
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let started = Instant::now();
    let response = http.request(
        ProviderEndpoint::BinanceTestnet,
        ProviderHttpMethod::Get,
        "/api/v3/time",
        HeaderMap::new(),
        None,
    )?;
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
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<String> {
    let filters = exchange_symbol["filters"].as_array().ok_or_else(invalid)?;
    let mut avg_minutes = None;
    for item in filters.iter().filter(|item| {
        matches!(
            item["filterType"].as_str(),
            Some("MIN_NOTIONAL" | "NOTIONAL")
        )
    }) {
        if let Some(value) = item.get("avgPriceMins").and_then(Value::as_u64) {
            if avg_minutes
                .replace(value)
                .is_some_and(|existing| existing != value)
            {
                return Err(invalid());
            }
        }
    }
    let endpoint = if avg_minutes.unwrap_or(0) == 0 {
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
    positive(&response_value(response, secrets)?["price"])
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
        let needs_reference = proposal.fields.quantity.r#type == OrderQuantityType::Base
            && proposal.fields.order_type == OrderType::Market;
        let reference = if needs_reference {
            Some(reference_price(
                http,
                &symbol,
                &exchange_symbol,
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
        assert!(!allows(ProviderEndpoint::BinanceTestnet, &restriction));
        assert!(allows(ProviderEndpoint::BinanceLive, &restriction));
    }
}
