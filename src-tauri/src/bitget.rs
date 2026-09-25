use super::*;
use crate::{
    protocol::{AssetClass, OrderProposalStatus},
    provider_io,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{collections::HashSet, time::Instant};

fn id(v: &Value, key: &str) -> Result<String> {
    let s = v[key].as_str().ok_or_else(invalid)?;
    if s.is_empty() || s.len() > 40 || s.starts_with('0') || !s.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid());
    }
    Ok(s.into())
}
fn older(a: &str, b: &str) -> bool {
    (a.len(), a) < (b.len(), b)
}
pub(super) fn allows(path: &str) -> bool {
    if matches!(
        path,
        "/api/v2/public/time"
            | "/api/v2/spot/account/info"
            | "/api/v2/spot/account/assets?assetType=all"
            | "/api/v2/spot/public/symbols?symbol=BTCUSDT"
            | "/api/v2/spot/public/symbols?symbol=ETHUSDT"
            | "/api/v2/spot/market/tickers?symbol=BTCUSDT"
            | "/api/v2/spot/market/tickers?symbol=ETHUSDT"
    ) {
        return true;
    }
    if let Some(client_oid) = path.strip_prefix("/api/v2/spot/trade/orderInfo?clientOid=") {
        return valid_client_oid(client_oid);
    }
    let Some((route, query)) = path.split_once('?') else {
        return false;
    };
    let rest = match route {
        "/api/v2/spot/trade/unfilled-orders" => query
            .strip_prefix("limit=100&tpslType=normal")
            .or_else(|| query.strip_prefix("limit=100&tpslType=tpsl")),
        "/api/v2/spot/trade/current-plan-order" => query.strip_prefix("limit=100"),
        _ => None,
    };
    rest.is_some_and(|r| {
        r.is_empty()
            || r.strip_prefix("&idLessThan=")
                .is_some_and(|s| id(&serde_json::json!({"id":s}), "id").is_ok())
    })
}
fn valid_client_oid(value: &str) -> bool {
    value.starts_with("tx-")
        && value.len() <= 50
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
pub(super) fn business(v: Value) -> Result<Value> {
    match v["code"].as_str() {
        Some("00000") => Ok(v["data"].clone()),
        Some(code) => Err(TradeXError::new(match code {
            "40005" | "40008" | "40078" => "CLOCK_SKEW",
            "40081" | "40105" | "40110" => "PROVIDER_UNSUPPORTED",
            "429" => "PROVIDER_RATE_LIMITED",
            "40001" | "40002" | "40003" | "40006" | "40009" | "40011" | "40012" | "40014"
            | "40018" | "40025" | "40036" | "40037" | "40038" | "40040" | "40041" => {
                "PROVIDER_AUTH_FAILED"
            }
            _ => "PROVIDER_UNAVAILABLE",
        })),
        None => Err(invalid()),
    }
}
pub(super) fn read(
    endpoint: ProviderEndpoint,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    old: Option<&AccountData>,
) -> Result<Observation> {
    let query = |path: &str, headers: HeaderMap, signature: Option<&str>| -> Result<Value> {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let bytes = http.get(endpoint, path, headers)?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err(invalid());
        }
        let value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if contains_secret(&value, secrets)
            || signature.is_some_and(|s| contains_secret(&value, &[s.into()]))
        {
            return Err(invalid());
        }
        business(value)
    };
    let started = Instant::now();
    let time = query("/api/v2/public/time", HeaderMap::new(), None)?;
    let server = time["serverTime"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|n| valid_time(*n))
        .ok_or_else(|| TradeXError::new("CLOCK_SKEW"))?;
    if started.elapsed() > Duration::from_secs(2) {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    let sampled = Instant::now();
    let signed = |path: &str| -> Result<Value> {
        if sampled.elapsed() > Duration::from_secs(60) {
            return Err(TradeXError::new("CLOCK_SKEW"));
        }
        let timestamp = server
            .checked_add(sampled.elapsed().as_millis() as u64)
            .filter(|n| valid_time(*n))
            .ok_or_else(|| TradeXError::new("CLOCK_SKEW"))?
            .to_string();
        let mut mac =
            Hmac::<Sha256>::new_from_slice(secrets[1].as_bytes()).map_err(|_| invalid())?;
        mac.update(format!("{timestamp}GET{path}").as_bytes());
        let signature = Zeroizing::new(STANDARD.encode(mac.finalize().into_bytes()));
        let mut headers = HeaderMap::new();
        for (name, value) in [
            ("ACCESS-KEY", secrets[0].as_str()),
            ("ACCESS-PASSPHRASE", secrets[2].as_str()),
            ("ACCESS-SIGN", signature.as_str()),
        ] {
            let mut header = HeaderValue::from_str(value)
                .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
            header.set_sensitive(true);
            headers.insert(name, header);
        }
        headers.insert(
            "ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp).map_err(|_| invalid())?,
        );
        if endpoint == ProviderEndpoint::BitgetDemo {
            headers.insert("paptrading", HeaderValue::from_static("1"));
        }
        query(path, headers, Some(&signature))
    };
    let account = signed("/api/v2/spot/account/info")?;
    let identity = id(&account, "userId")?;
    if old.is_some_and(|a| a.remote_account_id != identity) {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let permissions = permissions(&account)?;
    let assets = signed("/api/v2/spot/account/assets?assetType=all")?;
    let assets = assets.as_array().ok_or_else(invalid)?;
    if assets.len() > 10_000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let mut seen = HashSet::new();
    let mut balances = vec![];
    let mut positions = vec![];
    for a in assets {
        let asset = identifier(a, "coin")?;
        if !seen.insert(asset.to_uppercase()) {
            return Err(invalid());
        }
        let available = positive(&a["available"])?;
        let frozen = positive(&a["frozen"])?;
        let locked = positive(&a["locked"])?;
        let sum = total(&total(&available, &frozen)?, &locked)?;
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
        balances.push(Balance {
            asset,
            available,
            total: Some(sum),
            reserved: Some(frozen),
            locked: Some(locked),
            restricted_available: Some(positive(&a["limitAvailable"])?),
            in_pies: None,
        });
    }
    let mut orders = vec![];
    let mut ids = HashSet::new();
    for kind in ["normal", "tpsl", "plan"] {
        let base = if kind == "plan" {
            "/api/v2/spot/trade/current-plan-order?limit=100".into()
        } else {
            format!("/api/v2/spot/trade/unfilled-orders?limit=100&tpslType={kind}")
        };
        let mut cursor: Option<String> = None;
        loop {
            let path = match &cursor {
                Some(c) => format!("{base}&idLessThan={c}"),
                None => base.clone(),
            };
            let page = signed(&path)?;
            let rows = if kind == "plan" {
                &page["orderList"]
            } else {
                &page
            }
            .as_array()
            .ok_or_else(invalid)?;
            if rows.len() > 100 || orders.len() + rows.len() > 10_000 {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            let mut minimum: Option<String> = None;
            for row in rows {
                let order_id = id(row, "orderId")?;
                if cursor.as_ref().is_some_and(|c| !older(&order_id, c)) {
                    return Err(invalid());
                }
                if minimum.as_ref().is_none_or(|c| older(&order_id, c)) {
                    minimum = Some(order_id.clone());
                }
                let order = order(row, kind, &identity, order_id)?;
                if !ids.insert(order.broker_order_id.clone()) {
                    return Err(invalid());
                }
                orders.push(order);
            }
            let more = if kind == "plan" {
                page["nextFlag"].as_bool().ok_or_else(invalid)?
            } else {
                rows.len() == 100
            };
            if !more {
                break;
            }
            let minimum = minimum.ok_or_else(invalid)?;
            let next = if kind == "plan" {
                id(&page, "idLessThan")?
            } else {
                minimum.clone()
            };
            if next != minimum || cursor.as_ref().is_some_and(|c| !older(&next, c)) {
                return Err(invalid());
            }
            cursor = Some(next);
            // Keep each paginated endpoint below its documented 20 requests/second UID limit.
            std::thread::sleep(Duration::from_millis(55));
        }
    }
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    Ok(Observation { permissions, data: AccountData { remote_account_id: identity, account_type: "BITGET_CLASSIC_SPOT".into(), currency: None, buying_power: None, balances, positions, open_orders: orders, capabilities: vec!["account.read".into(),"positions.read".into(),"orders.read".into()], limitations: vec!["Asset totals and holdings are available + frozen + locked. Restricted availability is shown separately and is not added; no FX valuation or cost basis is inferred.".into(), "Order quote currency is unavailable until instrument metadata is resolved. Plan and TPSL observations do not enable trigger-order execution.".into(), "Classic Demo account endpoints may be unsupported. Private stream, reconciliation, risk policy and execution are not configured; Live remains disarmed.".into()] } })
}
fn permissions(account: &Value) -> Result<PermissionReview> {
    let mut p = PermissionReview::default();
    let mut complete = true;
    match account.get("authorities") {
        None | Some(Value::Null) => complete = false,
        Some(v) => {
            let values = v.as_array().ok_or_else(invalid)?;
            if values.len() > 100 {
                return Err(invalid());
            }
            for value in values {
                let code = value.as_str().ok_or_else(invalid)?;
                if code.is_empty()
                    || code.len() > 64
                    || !code
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                {
                    return Err(invalid());
                }
                if p.detected.iter().any(|s| s == code) {
                    return Err(invalid());
                }
                p.detected.push(code.into());
                match code {
                    "wtow" | "wwow" | "chow" => p.forbidden.push(code.into()),
                    "coow" | "cpow" | "smow" | "ttow" | "p2p" | "pllw" | "taxw" => {
                        p.unsupported.push(code.into())
                    }
                    "coor" | "cpor" | "stor" | "smor" | "ttor" | "wtor" | "taxr" | "chor"
                    | "p2pr" | "pllr" | "stow" => (),
                    _ => complete = false,
                }
            }
        }
    }
    p.ip_allow_list_status = match account.get("ips") {
        None | Some(Value::Null) => {
            complete = false;
            "UNKNOWN"
        }
        Some(Value::String(s)) if s.is_empty() => {
            p.ip_allow_list = Some(vec![]);
            "UNRESTRICTED"
        }
        Some(Value::String(s))
            if s.len() <= 4096
                && s.split(',')
                    .all(|ip| ip.trim().parse::<std::net::IpAddr>().is_ok()) =>
        {
            let mut addresses = s
                .split(',')
                .map(|ip| {
                    ip.trim()
                        .parse::<std::net::IpAddr>()
                        .map(|ip| ip.to_string())
                })
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|_| invalid())?;
            addresses.sort();
            addresses.dedup();
            p.ip_allow_list = Some(addresses);
            "RESTRICTED"
        }
        Some(Value::String(_)) => {
            complete = false;
            "UNKNOWN"
        }
        _ => return Err(invalid()),
    }
    .into();
    p.detected.sort();
    p.forbidden.sort();
    p.unsupported.sort();
    if complete {
        p.scope = "VERIFIED".into();
    }
    Ok(p)
}
fn order(v: &Value, kind: &str, identity: &str, order_id: String) -> Result<OpenOrder> {
    let plan = kind == "plan";
    if !plan && id(v, "userId")? != identity {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let side = text(v, "side", 4)?;
    let order_type = text(v, "orderType", 6)?;
    if !matches!(side.as_str(), "buy" | "sell")
        || !matches!(order_type.as_str(), "limit" | "market")
    {
        return Err(invalid());
    }
    let status = text(v, "status", 32)?;
    if (plan && status != "not_trigger")
        || (!plan && !matches!(status.as_str(), "new" | "live" | "partially_filled"))
    {
        return Err(invalid());
    }
    if !plan && v["tpslType"] != kind {
        return Err(invalid());
    }
    let quote = if plan {
        match v["planType"].as_str() {
            Some("amount") => false,
            Some("total") => true,
            _ => return Err(invalid()),
        }
    } else {
        order_type == "market" && side == "buy"
    };
    let size = positive(&v["size"])?;
    if size == "0" {
        return Err(invalid());
    }
    Ok(OpenOrder {
        broker_order_id: format!("{}:{order_id}", if plan { "plan" } else { "order" }),
        symbol: identifier(v, "symbol")?,
        instrument_id: None,
        side: side.to_uppercase(),
        quantity: (!quote).then(|| size.clone()),
        notional: quote.then_some(size),
        filled_quantity: if plan {
            None
        } else {
            Some(positive(&v["baseVolume"])?)
        },
        filled_value: if plan {
            None
        } else {
            Some(positive(&v["quoteVolume"])?)
        },
        currency: None,
        status: status.to_uppercase(),
        limit_price: if order_type == "limit" {
            Some(positive(
                &v[if plan { "executePrice" } else { "priceAvg" }],
            )?)
        } else {
            None
        },
        kind: Some(kind.to_uppercase()),
        trigger_price: if kind == "normal" {
            None
        } else {
            Some(positive(&v["triggerPrice"])?)
        },
    })
}

struct DemoClock {
    sampled_at: Instant,
    server_time: u64,
}

struct DemoOrderIntent {
    symbol: String,
    side: &'static str,
    order_type: &'static str,
    size: String,
    price: Option<String>,
    force: Option<&'static str>,
    amount_is_quote: bool,
}

fn demo_clock(
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    secrets: &[String],
) -> Result<DemoClock> {
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let started = Instant::now();
    let bytes = http.get(
        ProviderEndpoint::BitgetDemo,
        "/api/v2/public/time",
        HeaderMap::new(),
    )?;
    if bytes.len() as u64 > MAX_RESPONSE {
        return Err(invalid());
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if contains_secret(&value, secrets) || started.elapsed() > Duration::from_secs(2) {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    let server_time = business(value)?["serverTime"]
        .as_str()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| valid_time(*value))
        .ok_or_else(|| TradeXError::new("CLOCK_SKEW"))?;
    Ok(DemoClock {
        sampled_at: Instant::now(),
        server_time,
    })
}

fn demo_signed_headers(
    clock: &DemoClock,
    secrets: &[String],
    method: &str,
    path: &str,
    body: Option<&Value>,
) -> Result<(HeaderMap, Zeroizing<String>)> {
    if secrets.len() != 3 || clock.sampled_at.elapsed() > Duration::from_secs(60) {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    let timestamp = clock
        .server_time
        .checked_add(clock.sampled_at.elapsed().as_millis() as u64)
        .filter(|value| valid_time(*value))
        .ok_or_else(|| TradeXError::new("CLOCK_SKEW"))?
        .to_string();
    let body = body
        .map(serde_json::to_string)
        .transpose()
        .map_err(|_| invalid())?
        .unwrap_or_default();
    let mut mac = Hmac::<Sha256>::new_from_slice(secrets[1].as_bytes()).map_err(|_| invalid())?;
    mac.update(format!("{timestamp}{method}{path}{body}").as_bytes());
    let signature = Zeroizing::new(STANDARD.encode(mac.finalize().into_bytes()));
    let mut headers = HeaderMap::new();
    for (name, value) in [
        ("ACCESS-KEY", secrets[0].as_str()),
        ("ACCESS-PASSPHRASE", secrets[2].as_str()),
        ("ACCESS-SIGN", signature.as_str()),
    ] {
        let mut header =
            HeaderValue::from_str(value).map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
        header.set_sensitive(true);
        headers.insert(name, header);
    }
    headers.insert(
        "ACCESS-TIMESTAMP",
        HeaderValue::from_str(&timestamp).map_err(|_| invalid())?,
    );
    headers.insert("paptrading", HeaderValue::from_static("1"));
    Ok((headers, signature))
}

fn demo_get(
    clock: &DemoClock,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    path: &str,
) -> Result<Value> {
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let (headers, signature) = demo_signed_headers(clock, secrets, "GET", path, None)?;
    let bytes = http.get(ProviderEndpoint::BitgetDemo, path, headers)?;
    if bytes.len() as u64 > MAX_RESPONSE {
        return Err(invalid());
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if contains_secret(&value, secrets)
        || contains_secret(&value, std::slice::from_ref(&*signature))
    {
        return Err(invalid());
    }
    business(value)
}

fn bitget_symbol(proposal: &OrderProposal, connection_id: &str) -> Result<String> {
    let fields = &proposal.fields;
    if !matches!(
        proposal.status,
        OrderProposalStatus::NeedsApproval | OrderProposalStatus::Consumed
    ) || fields.environment != ExecutionContext::BitgetDemo
        || fields.account_id.as_deref() != Some(connection_id)
        || fields.venue != "BITGET"
    {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == fields.instrument_id)
        .filter(|instrument| instrument.asset_class == AssetClass::CryptoSpot)
        .and_then(|instrument| {
            instrument
                .providers
                .into_iter()
                .find(|mapping| mapping.provider_id == "bitget")
                .map(|mapping| mapping.provider_symbol)
        })
        .filter(|symbol| matches!(symbol.as_str(), "BTCUSDT" | "ETHUSDT"))
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))
}

fn demo_order_intent(
    proposal: &OrderProposal,
    attempt: &BitgetDemoOrderAttempt,
) -> Result<DemoOrderIntent> {
    let fields = &proposal.fields;
    if attempt.environment != "DEMO"
        || attempt.proposal_id != proposal.proposal_id
        || attempt.proposal_hash != proposal.proposal_hash
        || fields.maximum_spend.is_some()
    {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    let symbol = bitget_symbol(proposal, &attempt.connection_id)?;
    let size = provider_io::decimal(&json!(fields.quantity.value))?;
    if provider_io::decimal_cmp(&size, "0")? != std::cmp::Ordering::Greater {
        return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
    }
    match (
        fields.order_type,
        fields.time_in_force,
        fields.quantity.r#type,
        fields.side,
    ) {
        (
            OrderType::Limit,
            TimeInForce::Gtc | TimeInForce::Ioc | TimeInForce::Fok,
            OrderQuantityType::Base,
            _,
        ) => {
            let price = fields
                .limit_price
                .as_deref()
                .ok_or_else(|| TradeXError::new("ORDER_AMOUNT_INVALID"))?;
            let price = provider_io::decimal(&json!(price))?;
            if provider_io::decimal_cmp(&price, "0")? != std::cmp::Ordering::Greater {
                return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
            }
            Ok(DemoOrderIntent {
                symbol,
                side: if fields.side == OrderSide::Buy {
                    "buy"
                } else {
                    "sell"
                },
                order_type: "limit",
                size,
                price: Some(price),
                force: Some(match fields.time_in_force {
                    TimeInForce::Gtc => "gtc",
                    TimeInForce::Ioc => "ioc",
                    TimeInForce::Fok => "fok",
                    TimeInForce::Day => unreachable!(),
                }),
                amount_is_quote: false,
            })
        }
        (OrderType::Market, TimeInForce::Day, OrderQuantityType::Quote, OrderSide::Buy)
            if fields.limit_price.is_none() =>
        {
            Ok(DemoOrderIntent {
                symbol,
                side: "buy",
                order_type: "market",
                size,
                price: None,
                force: None,
                amount_is_quote: true,
            })
        }
        (OrderType::Market, TimeInForce::Day, OrderQuantityType::Base, OrderSide::Sell)
            if fields.limit_price.is_none() =>
        {
            Ok(DemoOrderIntent {
                symbol,
                side: "sell",
                order_type: "market",
                size,
                price: None,
                force: None,
                amount_is_quote: false,
            })
        }
        _ => Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED")),
    }
}

pub(super) fn validate_demo_proposal(proposal: &OrderProposal, connection_id: &str) -> Result<()> {
    let symbol = bitget_symbol(proposal, connection_id)?;
    let attempt = BitgetDemoOrderAttempt {
        attempt_id: "preflight".into(),
        workspace_id: proposal.workspace_id.clone(),
        connection_id: connection_id.into(),
        remote_account_id: "1".into(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_hash: proposal.proposal_hash.clone(),
        environment: "DEMO".into(),
        client_oid: "tx-preflight".into(),
        state: BitgetDemoOrderAttemptState::Submitting,
        provider_order_id: None,
        provider_status: None,
        error_code: None,
        reason: "preflight".into(),
        state_version: "preflight".into(),
        created_at: "preflight".into(),
        updated_at: "preflight".into(),
    };
    let intent = demo_order_intent(proposal, &attempt)?;
    if intent.symbol != symbol {
        return Err(invalid());
    }
    Ok(())
}

fn symbol_number(value: &Value, name: &str) -> Result<u32> {
    let number = value
        .get(name)
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(invalid)?;
    if number > 18 {
        return Err(invalid());
    }
    Ok(number)
}

fn decimal_places(value: &str) -> usize {
    value
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
}

fn checked_order_body(
    proposal: &OrderProposal,
    attempt: &BitgetDemoOrderAttempt,
    http: &impl ProviderHttp,
    clock: &DemoClock,
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<Value> {
    let intent = demo_order_intent(proposal, attempt)?;
    let symbols = demo_get(
        clock,
        secrets,
        http,
        current,
        &format!("/api/v2/spot/public/symbols?symbol={}", intent.symbol),
    )?;
    let symbol_rows = symbols
        .as_array()
        .filter(|rows| rows.len() == 1)
        .ok_or_else(invalid)?;
    let symbol = &symbol_rows[0];
    if symbol["symbol"].as_str() != Some(intent.symbol.as_str())
        || symbol["baseCoin"]
            .as_str()
            .is_none_or(|value| value.is_empty())
        || symbol["quoteCoin"]
            .as_str()
            .is_none_or(|value| value.is_empty())
        || symbol["status"].as_str() != Some("online")
    {
        return Err(TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"));
    }
    let price_precision = symbol_number(symbol, "pricePrecision")? as usize;
    let base_precision = symbol_number(symbol, "quantityPrecision")? as usize;
    let quote_precision = symbol_number(symbol, "quotePrecision")? as usize;
    let minimum = provider_io::decimal(&symbol["minTradeUSDT"])?;
    let ticker = demo_get(
        clock,
        secrets,
        http,
        current,
        &format!("/api/v2/spot/market/tickers?symbol={}", intent.symbol),
    )?;
    let ticker_rows = ticker
        .as_array()
        .filter(|rows| rows.len() == 1)
        .ok_or_else(invalid)?;
    let row = &ticker_rows[0];
    if row["symbol"].as_str() != Some(intent.symbol.as_str()) {
        return Err(invalid());
    }
    let bid = provider_io::decimal(&row["bidPr"])?;
    let ask = provider_io::decimal(&row["askPr"])?;
    if provider_io::decimal_cmp(&bid, "0")? != std::cmp::Ordering::Greater
        || provider_io::decimal_cmp(&ask, "0")? != std::cmp::Ordering::Greater
    {
        return Err(invalid());
    }
    let amount_precision = if intent.amount_is_quote {
        quote_precision
    } else {
        base_precision
    };
    if decimal_places(&intent.size) > amount_precision
        || intent
            .price
            .as_deref()
            .is_some_and(|price| decimal_places(price) > price_precision)
    {
        return Err(TradeXError::new("ORDER_FILTER_REJECTED"));
    }
    let notional = match (&intent.price, intent.amount_is_quote, intent.side) {
        (Some(price), _, _) => binance::multiply(&intent.size, price)?,
        (None, true, "buy") => intent.size.clone(),
        (None, false, "sell") => binance::multiply(&intent.size, &bid)?,
        _ => return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED")),
    };
    if provider_io::decimal_cmp(&notional, &minimum)? == std::cmp::Ordering::Less {
        return Err(TradeXError::new("ORDER_FILTER_REJECTED"));
    }
    let mut body = json!({
        "symbol":intent.symbol,
        "side":intent.side,
        "orderType":intent.order_type,
        "size":intent.size,
        "clientOid":attempt.client_oid
    });
    if let Some(price) = intent.price {
        body["price"] = Value::String(price);
    }
    if let Some(force) = intent.force {
        body["force"] = Value::String(force.into());
    }
    Ok(body)
}

fn set_demo_attempt_error(
    mut attempt: BitgetDemoOrderAttempt,
    state: BitgetDemoOrderAttemptState,
    code: &str,
    reason: &str,
) -> BitgetDemoOrderAttempt {
    attempt.state = state;
    attempt.provider_order_id = None;
    attempt.provider_status = None;
    attempt.error_code = Some(code.into());
    attempt.reason = reason.chars().take(256).collect();
    attempt
}

fn acknowledgement(
    attempt: &BitgetDemoOrderAttempt,
    data: &Value,
    expected_user_id: Option<&str>,
) -> Result<BitgetDemoOrderAttempt> {
    if data["clientOid"].as_str() != Some(attempt.client_oid.as_str())
        || expected_user_id.is_some_and(|expected| data["userId"].as_str() != Some(expected))
    {
        return Err(invalid());
    }
    let order_id = data["orderId"]
        .as_str()
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 40
                && !value.starts_with('0')
                && value.bytes().all(|byte| byte.is_ascii_digit())
        })
        .ok_or_else(invalid)?;
    let mut acknowledged = attempt.clone();
    acknowledged.state = BitgetDemoOrderAttemptState::Acknowledged;
    acknowledged.provider_order_id = Some(order_id.into());
    acknowledged.provider_status = data
        .get("status")
        .and_then(Value::as_str)
        .filter(|status| {
            matches!(
                *status,
                "live" | "partially_filled" | "filled" | "cancelled"
            )
        })
        .map(str::to_owned);
    acknowledged.error_code = None;
    acknowledged.reason = if acknowledged.provider_status.is_some() {
        "Bitget returned order state for this clientOid; fill quantities remain provider evidence."
            .into()
    } else {
        "Bitget accepted the order request. Acknowledgement is not evidence of a fill.".into()
    };
    Ok(acknowledged)
}

pub(super) fn run_demo_order(
    attempt: &BitgetDemoOrderAttempt,
    proposal: &OrderProposal,
    reconcile: bool,
    reviewed_permissions: &PermissionReview,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> BitgetDemoOrderAttempt {
    if attempt.environment != "DEMO"
        || attempt.connection_id != proposal.fields.account_id.as_deref().unwrap_or_default()
        || attempt.proposal_id != proposal.proposal_id
        || attempt.proposal_hash != proposal.proposal_hash
        || !valid_client_oid(&attempt.client_oid)
    {
        return set_demo_attempt_error(
            attempt.clone(),
            BitgetDemoOrderAttemptState::Rejected,
            "ORDER_PROPOSAL_NOT_ELIGIBLE",
            "The saved Bitget Demo order no longer matches its immutable Proposal.",
        );
    }
    let prepared = (|| -> Result<(DemoClock, Value)> {
        let clock = demo_clock(http, current, secrets)?;
        let account = demo_get(&clock, secrets, http, current, "/api/v2/spot/account/info")?;
        if id(&account, "userId")? != attempt.remote_account_id {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
        }
        if !reconcile {
            let observed = permissions(&account)?;
            if !observed.forbidden.is_empty() || !observed.unsupported.is_empty() {
                return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED"));
            }
            let mut reviewed = reviewed_permissions.clone();
            reviewed.acknowledged = false;
            if observed != reviewed {
                return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
            }
        }
        let body = if reconcile {
            Value::Null
        } else {
            checked_order_body(proposal, attempt, http, &clock, secrets, current)?
        };
        Ok((clock, body))
    })();
    let (clock, body) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            return set_demo_attempt_error(
                attempt.clone(),
                if reconcile {
                    BitgetDemoOrderAttemptState::UnknownReconciling
                } else {
                    BitgetDemoOrderAttemptState::Rejected
                },
                &error.code,
                &error.message,
            );
        }
    };
    if !current() {
        return set_demo_attempt_error(
            attempt.clone(),
            if reconcile {
                BitgetDemoOrderAttemptState::UnknownReconciling
            } else {
                BitgetDemoOrderAttemptState::Rejected
            },
            "STATE_VERSION_CONFLICT",
            "The saved connection state changed before the Bitget request.",
        );
    }
    if reconcile {
        let path = format!(
            "/api/v2/spot/trade/orderInfo?clientOid={}",
            attempt.client_oid
        );
        let result = demo_get(&clock, secrets, http, current, &path);
        return match result {
            Ok(data) => {
                let rows = data.as_array();
                match rows.filter(|rows| rows.len() == 1).and_then(|rows| rows.first()) {
                    Some(row) => acknowledgement(attempt, row, Some(&attempt.remote_account_id))
                        .unwrap_or_else(|_| set_demo_attempt_error(
                            attempt.clone(),
                            BitgetDemoOrderAttemptState::UnknownReconciling,
                            "ORDER_STATUS_UNKNOWN",
                            "Bitget order lookup did not match the saved clientOid and account.",
                        )),
                    None => set_demo_attempt_error(
                        attempt.clone(),
                        BitgetDemoOrderAttemptState::UnknownReconciling,
                        "ORDER_STATUS_UNKNOWN",
                        "Bitget has not returned an order for the saved clientOid yet.",
                    ),
                }
            }
            Err(error) => set_demo_attempt_error(
                attempt.clone(),
                BitgetDemoOrderAttemptState::UnknownReconciling,
                &error.code,
                &error.message,
            ),
        };
    }
    let (headers, signature) = match demo_signed_headers(
        &clock,
        secrets,
        "POST",
        "/api/v2/spot/trade/place-order",
        Some(&body),
    ) {
        Ok(value) => value,
        Err(error) => {
            return set_demo_attempt_error(
                attempt.clone(),
                BitgetDemoOrderAttemptState::Rejected,
                &error.code,
                &error.message,
            );
        }
    };
    if !current() {
        return set_demo_attempt_error(
            attempt.clone(),
            BitgetDemoOrderAttemptState::Rejected,
            "STATE_VERSION_CONFLICT",
            "The saved connection state changed before the Bitget Demo write.",
        );
    }
    match http.request(
        ProviderEndpoint::BitgetDemo,
        ProviderHttpMethod::Post,
        "/api/v2/spot/trade/place-order",
        headers,
        Some(&body),
    ) {
        Err(error) => set_demo_attempt_error(
            attempt.clone(),
            BitgetDemoOrderAttemptState::UnknownReconciling,
            "ORDER_STATUS_UNKNOWN",
            &error.message,
        ),
        Ok(response) if response.status == 200 && response.body.len() as u64 <= MAX_RESPONSE => {
            let value: Value = match serde_json::from_slice(&response.body) {
                Ok(value) => value,
                Err(_) => {
                    return set_demo_attempt_error(
                        attempt.clone(),
                        BitgetDemoOrderAttemptState::UnknownReconciling,
                        "ORDER_STATUS_UNKNOWN",
                        "Bitget returned an unreadable response after the order request.",
                    );
                }
            };
            if contains_secret(&value, secrets)
                || contains_secret(&value, std::slice::from_ref(&*signature))
            {
                return set_demo_attempt_error(
                    attempt.clone(),
                    BitgetDemoOrderAttemptState::UnknownReconciling,
                    "ORDER_STATUS_UNKNOWN",
                    "Bitget response could not be safely verified; query the saved clientOid.",
                );
            }
            if value["code"].as_str() != Some("00000") {
                let code = value["code"].as_str().unwrap_or_default();
                return if code == "429" {
                    set_demo_attempt_error(
                        attempt.clone(),
                        BitgetDemoOrderAttemptState::UnknownReconciling,
                        "ORDER_STATUS_UNKNOWN",
                        "Bitget rate-limited the request; query the saved clientOid before acting.",
                    )
                } else {
                    let error = business(value).err().unwrap_or_else(invalid);
                    set_demo_attempt_error(
                        attempt.clone(),
                        BitgetDemoOrderAttemptState::Rejected,
                        &error.code,
                        &error.message,
                    )
                };
            }
            acknowledgement(attempt, &value["data"], None).unwrap_or_else(|_| {
                set_demo_attempt_error(
                    attempt.clone(),
                    BitgetDemoOrderAttemptState::UnknownReconciling,
                    "ORDER_STATUS_UNKNOWN",
                    "Bitget acknowledgement did not echo the saved clientOid; query before acting.",
                )
            })
        }
        Ok(response)
            if response.status == 408
                || response.status == 429
                || response.status >= 500
                || (300..400).contains(&response.status)
                || response.body.len() as u64 > MAX_RESPONSE =>
        {
            set_demo_attempt_error(
                attempt.clone(),
                BitgetDemoOrderAttemptState::UnknownReconciling,
                "ORDER_STATUS_UNKNOWN",
                "Bitget did not return a verifiable result; query the saved clientOid before acting.",
            )
        }
        Ok(_) => set_demo_attempt_error(
            attempt.clone(),
            BitgetDemoOrderAttemptState::Rejected,
            "PROVIDER_ORDER_REJECTED",
            "Bitget rejected the order request before creating an order.",
        ),
    }
}
