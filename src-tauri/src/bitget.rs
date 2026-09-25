use super::*;
use crate::{
    protocol::{AssetClass, OrderProposalStatus},
    provider_io,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

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
fn cursor_suffix(value: &str) -> bool {
    value.is_empty()
        || value
            .strip_prefix("&idLessThan=")
            .is_some_and(|cursor| id(&serde_json::json!({"id":cursor}), "id").is_ok())
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
    let allowed = match route {
        "/api/v2/spot/trade/unfilled-orders" => query
            .strip_prefix("limit=100&tpslType=normal")
            .or_else(|| query.strip_prefix("limit=100&tpslType=tpsl"))
            .is_some_and(cursor_suffix),
        "/api/v2/spot/trade/current-plan-order" => {
            query.strip_prefix("limit=100").is_some_and(cursor_suffix)
        }
        _ => false,
    };
    allowed
}
pub(super) fn allows_live_history(path: &str) -> bool {
    let Some((route, query)) = path.split_once('?') else {
        return false;
    };
    match route {
        "/api/v2/spot/trade/history-plan-order" | "/api/v2/spot/trade/fills" => {
            query.strip_prefix("limit=100").is_some_and(cursor_suffix)
        }
        "/api/v2/spot/trade/history-orders" => ["limit=100", "limit=100&tpslType=tpsl"]
            .into_iter()
            .any(|base| query.strip_prefix(base).is_some_and(cursor_suffix)),
        _ => false,
    }
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
    let mut orders = vec![];
    let mut bitget_orders = vec![];
    let mut ids = HashSet::new();
    let mut book_ids = HashSet::new();
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
                let detail = bitget_order(row, kind, &identity, order_id)?;
                let order = open_order(&detail, row)?;
                if !ids.insert(order.broker_order_id.clone()) {
                    return Err(invalid());
                }
                book_ids.insert(format!("{}:{}", detail.kind, detail.provider_order_id));
                orders.push(order);
                bitget_orders.push(detail);
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
    let bitget_order_book = if endpoint == ProviderEndpoint::BitgetLive {
        let mut history = Vec::new();
        for kind in ["normal", "tpsl"] {
            history.extend(history_orders(&signed, &identity, current, kind)?);
        }
        history.extend(history_plan_orders(&signed, current)?);
        let mut currencies = HashMap::new();
        for order in bitget_orders.iter().chain(history.iter()) {
            if let Some(currency) = &order.currency {
                let key = (order.provider_order_id.clone(), order.symbol.clone());
                if currencies
                    .get(&key)
                    .is_some_and(|existing| existing != currency)
                {
                    return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                }
                currencies.insert(key, currency.clone());
            }
        }
        let fills = fills(&signed, &identity, &currencies, current)?;
        for order in history {
            if !book_ids.insert(format!("{}:{}", order.kind, order.provider_order_id)) {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            bitget_orders.push(order);
        }
        bitget_orders.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.created_at.cmp(&left.created_at))
                .then_with(|| {
                    if older(&left.provider_order_id, &right.provider_order_id) {
                        std::cmp::Ordering::Greater
                    } else if older(&right.provider_order_id, &left.provider_order_id) {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
        });
        Some(BitgetSpotOrderBook {
            orders: bitget_orders,
            fills,
            observed_at: crate::storage::timestamp()?,
        })
    } else {
        None
    };
    let assets = signed("/api/v2/spot/account/assets?assetType=all")?;
    let assets = assets.as_array().ok_or_else(invalid)?;
    if assets.len() > 10_000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let mut seen = HashSet::new();
    let mut balances = vec![];
    let mut positions = vec![];
    for asset_value in assets {
        let asset = identifier(asset_value, "coin")?;
        if !seen.insert(asset.to_uppercase()) {
            return Err(invalid());
        }
        let available = positive(&asset_value["available"])?;
        let frozen = positive(&asset_value["frozen"])?;
        let locked = positive(&asset_value["locked"])?;
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
            restricted_available: Some(positive(&asset_value["limitAvailable"])?),
            in_pies: None,
        });
    }
    let mut limitations = vec![
        "Asset totals and holdings are available + frozen + locked. Restricted availability is shown separately and is not added; no FX valuation or cost basis is inferred.".into(),
        "Order quote currency is unavailable until instrument metadata is resolved. Plan and TPSL observations do not enable trigger-order execution.".into(),
        "Private stream, reconciliation, risk policy and execution are not configured; Live remains disarmed.".into(),
    ];
    if endpoint == ProviderEndpoint::BitgetLive {
        limitations.push(
            "Live order history and fills are read-only, limited to the provider's recent 90-day window and 2,000 rows per order category or fill query. Live execution remains disarmed.".into(),
        );
    } else {
        limitations.push("Classic Demo account endpoints may be unsupported.".into());
    }
    Ok(Observation {
        permissions,
        data: AccountData {
            remote_account_id: identity,
            account_type: "BITGET_CLASSIC_SPOT".into(),
            currency: None,
            buying_power: None,
            balances,
            positions,
            open_orders: orders,
            bitget_order_book,
            capabilities: vec![
                "account.read".into(),
                "positions.read".into(),
                "orders.read".into(),
            ],
            limitations,
        },
    })
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
fn timestamp(value: &Value, field: &str) -> Result<Option<String>> {
    let Some(value) = value.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let raw = value.as_str().ok_or_else(invalid)?;
    let parsed = raw.parse::<u64>().map_err(|_| invalid())?;
    let millis = if parsed < 100_000_000_000 {
        parsed.checked_mul(1000).ok_or_else(invalid)?
    } else {
        parsed
    };
    if !valid_time(millis) {
        return Err(invalid());
    }
    let instant = time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(millis) * 1_000_000)
        .map_err(|_| invalid())?;
    Ok(Some(
        instant
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|_| invalid())?,
    ))
}

fn normalized_status(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "new" | "live" | "not_trigger" => "OPEN",
        "partially_filled" => "PARTIALLY_FILLED",
        "filled" => "FILLED",
        "executed" => "TRIGGERED",
        "fail_execute" => "TRIGGER_FAILED",
        "cancelled" | "canceled" => "CANCELED",
        "rejected" => "REJECTED",
        _ => "UNKNOWN",
    }
}

fn optional_decimal(value: &Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(raw)) if raw.is_empty() => Ok(None),
        Some(value) => positive(value).map(Some),
    }
}

fn optional_identifier(value: &Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(raw)) if raw.is_empty() => Ok(None),
        Some(_) => identifier(value, field).map(Some),
    }
}

fn bitget_order(
    v: &Value,
    kind: &str,
    identity: &str,
    order_id: String,
) -> Result<BitgetSpotOrder> {
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
    let filled_quantity = if plan {
        None
    } else {
        optional_decimal(v, "baseVolume")?
    };
    let filled_value = if plan {
        None
    } else {
        optional_decimal(v, "quoteVolume")?
    };
    let quantity = (!quote).then(|| size.clone());
    let notional = quote.then_some(size);
    let remaining_quantity = match (&quantity, &filled_quantity) {
        (Some(quantity), Some(filled)) => Some(provider_io::decimal_subtract(quantity, filled)?),
        _ => None,
    };
    Ok(BitgetSpotOrder {
        provider_order_id: order_id,
        kind: kind.to_uppercase(),
        symbol: identifier(v, "symbol")?,
        side: side.to_uppercase(),
        quantity,
        notional,
        filled_quantity,
        filled_value,
        remaining_quantity,
        currency: optional_identifier(v, "quoteCoin")?,
        provider_status: status.clone(),
        normalized_status: normalized_status(&status).into(),
        origin: "external".into(),
        created_at: timestamp(v, "cTime")?,
        updated_at: timestamp(v, "uTime")?,
    })
}

fn open_order(order: &BitgetSpotOrder, raw: &Value) -> Result<OpenOrder> {
    let limit_price = if raw["orderType"] == "limit" {
        Some(positive(
            &raw[if order.kind == "PLAN" {
                "executePrice"
            } else {
                "priceAvg"
            }],
        )?)
    } else {
        None
    };
    let trigger_price = if order.kind == "NORMAL" {
        None
    } else {
        Some(positive(&raw["triggerPrice"])?)
    };
    Ok(OpenOrder {
        broker_order_id: format!(
            "{}:{}",
            order.kind.to_ascii_lowercase(),
            order.provider_order_id
        ),
        symbol: order.symbol.clone(),
        instrument_id: None,
        side: order.side.clone(),
        quantity: order.quantity.clone(),
        notional: order.notional.clone(),
        filled_quantity: order.filled_quantity.clone(),
        filled_value: order.filled_value.clone(),
        currency: order.currency.clone(),
        status: order.provider_status.to_ascii_uppercase(),
        limit_price,
        kind: Some(order.kind.clone()),
        trigger_price,
    })
}

fn history_orders(
    signed: &impl Fn(&str) -> Result<Value>,
    identity: &str,
    current: &impl Fn() -> bool,
    kind: &str,
) -> Result<Vec<BitgetSpotOrder>> {
    const MAX_PAGES: usize = 20;
    let base = if kind == "tpsl" {
        "/api/v2/spot/trade/history-orders?limit=100&tpslType=tpsl"
    } else {
        "/api/v2/spot/trade/history-orders?limit=100"
    };
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor: Option<String> = None;
    for page_number in 0..MAX_PAGES {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let path = cursor.as_ref().map_or_else(
            || base.to_owned(),
            |cursor| format!("{base}&idLessThan={cursor}"),
        );
        let page = signed(&path)?;
        let rows = page.as_array().ok_or_else(invalid)?;
        if rows.len() > 100 || result.len() + rows.len() > MAX_PAGES * 100 {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        let mut minimum: Option<String> = None;
        let mut prior: Option<String> = None;
        for row in rows {
            let order_id = id(row, "orderId")?;
            if cursor
                .as_ref()
                .is_some_and(|value| !older(&order_id, value))
                || prior.as_ref().is_some_and(|value| !older(&order_id, value))
                || !seen.insert(order_id.clone())
            {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            if minimum.as_ref().is_none_or(|value| older(&order_id, value)) {
                minimum = Some(order_id.clone());
            }
            prior = Some(order_id.clone());
            result.push(bitget_order(row, kind, identity, order_id)?);
        }
        if rows.len() < 100 {
            return Ok(result);
        }
        if page_number + 1 == MAX_PAGES {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        let next = minimum.ok_or_else(invalid)?;
        if cursor.as_ref().is_some_and(|value| !older(&next, value)) {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        cursor = Some(next);
        std::thread::sleep(Duration::from_millis(105));
    }
    Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"))
}

fn history_plan_orders(
    signed: &impl Fn(&str) -> Result<Value>,
    current: &impl Fn() -> bool,
) -> Result<Vec<BitgetSpotOrder>> {
    const MAX_PAGES: usize = 20;
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor: Option<String> = None;
    for page_number in 0..MAX_PAGES {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let path = cursor.as_ref().map_or_else(
            || "/api/v2/spot/trade/history-plan-order?limit=100".to_owned(),
            |cursor| format!("/api/v2/spot/trade/history-plan-order?limit=100&idLessThan={cursor}"),
        );
        let page = signed(&path)?;
        let rows = page["orderList"].as_array().ok_or_else(invalid)?;
        if rows.len() > 100 || result.len() + rows.len() > MAX_PAGES * 100 {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        let mut minimum: Option<String> = None;
        let mut prior: Option<String> = None;
        for row in rows {
            let order_id = id(row, "orderId")?;
            if cursor
                .as_ref()
                .is_some_and(|value| !older(&order_id, value))
                || prior.as_ref().is_some_and(|value| !older(&order_id, value))
                || !seen.insert(order_id.clone())
            {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            if minimum.as_ref().is_none_or(|value| older(&order_id, value)) {
                minimum = Some(order_id.clone());
            }
            prior = Some(order_id.clone());
            result.push(bitget_order(row, "plan", "", order_id)?);
        }
        if !page["nextFlag"].as_bool().ok_or_else(invalid)? {
            return Ok(result);
        }
        if page_number + 1 == MAX_PAGES {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        let next = id(&page, "idLessThan")?;
        let minimum = minimum.ok_or_else(invalid)?;
        if older(&minimum, &next) || cursor.as_ref().is_some_and(|value| !older(&next, value)) {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        cursor = Some(next);
        std::thread::sleep(Duration::from_millis(105));
    }
    Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"))
}

fn fills(
    signed: &impl Fn(&str) -> Result<Value>,
    identity: &str,
    currencies: &HashMap<(String, String), String>,
    current: &impl Fn() -> bool,
) -> Result<Vec<BitgetSpotFill>> {
    const MAX_PAGES: usize = 20;
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor: Option<String> = None;
    for page_number in 0..MAX_PAGES {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let path = cursor.as_ref().map_or_else(
            || "/api/v2/spot/trade/fills?limit=100".to_owned(),
            |cursor| format!("/api/v2/spot/trade/fills?limit=100&idLessThan={cursor}"),
        );
        let page = signed(&path)?;
        let rows = page.as_array().ok_or_else(invalid)?;
        if rows.len() > 100 || result.len() + rows.len() > MAX_PAGES * 100 {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        let mut minimum: Option<String> = None;
        let mut prior: Option<String> = None;
        for row in rows {
            let trade_id = id(row, "tradeId")?;
            if id(row, "userId")? != identity {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
            }
            let order_id = id(row, "orderId")?;
            let key = format!("{order_id}:{trade_id}");
            if cursor
                .as_ref()
                .is_some_and(|value| !older(&trade_id, value))
                || prior.as_ref().is_some_and(|value| !older(&trade_id, value))
                || !seen.insert(key)
            {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            if minimum.as_ref().is_none_or(|value| older(&trade_id, value)) {
                minimum = Some(trade_id.clone());
            }
            prior = Some(trade_id.clone());
            let side = text(row, "side", 4)?;
            if !matches!(side.as_str(), "buy" | "sell") {
                return Err(invalid());
            }
            let symbol = identifier(row, "symbol")?;
            let observed_at = timestamp(row, "uTime")?
                .or(timestamp(row, "cTime")?)
                .unwrap_or(crate::storage::timestamp()?);
            result.push(BitgetSpotFill {
                provider_trade_id: trade_id,
                provider_order_id: order_id.clone(),
                symbol: symbol.clone(),
                side: side.to_uppercase(),
                price: optional_decimal(row, "priceAvg")?,
                quantity: positive(&row["size"])?,
                value: optional_decimal(row, "amount")?,
                currency: currencies.get(&(order_id, symbol)).cloned(),
                observed_at,
            });
        }
        if rows.len() < 100 {
            return Ok(result);
        }
        if page_number + 1 == MAX_PAGES {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        let next = minimum.ok_or_else(invalid)?;
        if cursor.as_ref().is_some_and(|value| !older(&next, value)) {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        cursor = Some(next);
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_currency_matches_order_id_and_symbol() {
        let currencies = HashMap::from([
            (("200".into(), "BTCUSDT".into()), "USDT".into()),
            (("200".into(), "ETHUSDC".into()), "USDC".into()),
        ]);
        let signed = |_: &str| {
            Ok(serde_json::json!([{
                "userId":"9007199254740993", "orderId":"200", "tradeId":"300",
                "symbol":"BTCUSDT", "side":"buy", "priceAvg":"70000.125",
                "size":"0.0002", "amount":"14.000025", "cTime":"1788849500000"
            }]))
        };
        let result = fills(&signed, "9007199254740993", &currencies, &|| true).unwrap();
        assert_eq!(result[0].currency.as_deref(), Some("USDT"));
    }
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
