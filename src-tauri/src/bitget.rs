use super::*;
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
    ) {
        return true;
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
    Ok(Observation { permissions, data: AccountData { remote_account_id: identity, account_type: "BITGET_CLASSIC_SPOT".into(), currency: None, balances, positions, open_orders: orders, capabilities: vec!["account.read".into(),"positions.read".into(),"orders.read".into()], limitations: vec!["Asset totals and holdings are available + frozen + locked. Restricted availability is shown separately and is not added; no FX valuation or cost basis is inferred.".into(), "Order quote currency is unavailable until instrument metadata is resolved. Plan and TPSL observations do not enable trigger-order execution.".into(), "Classic Demo account endpoints may be unsupported. Private stream, reconciliation, risk policy and execution are not configured; Live remains disarmed.".into()] } })
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
