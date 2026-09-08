use super::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{collections::HashSet, time::Instant};

fn time_error() -> TradeXError {
    TradeXError::new("CLOCK_SKEW")
}
fn valid_time(n: u64) -> bool {
    (946684800000..4102444800000).contains(&n)
}

pub(super) fn allows(endpoint: ProviderEndpoint, path: &str) -> bool {
    if path == "/api/v3/time" {
        return true;
    }
    let Some((route, query)) = path.split_once('?') else {
        return false;
    };
    if !matches!(route, "/api/v3/account" | "/api/v3/openOrders")
        && !(endpoint == ProviderEndpoint::BinanceLive
            && route == "/sapi/v1/account/apiRestrictions")
    {
        return false;
    }
    let Some((timestamp, signature)) = query
        .strip_prefix("timestamp=")
        .and_then(|q| q.split_once("&recvWindow=5000&signature="))
    else {
        return false;
    };
    timestamp.len() == 13
        && timestamp.bytes().all(|b| b.is_ascii_digit())
        && timestamp.parse().is_ok_and(valid_time)
        && signature.len() == 64
        && signature
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
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
fn identifier(v: &Value, field: &str) -> Result<String> {
    let s = v[field].as_str().ok_or_else(invalid)?;
    if s.is_empty() || s.len() > 128 || !s.chars().all(|c| c.is_alphanumeric() || "._-".contains(c))
    {
        return Err(invalid());
    }
    Ok(s.into())
}
fn positive(v: &Value) -> Result<String> {
    let s = decimal(v)?;
    if s.starts_with('-') {
        return Err(invalid());
    }
    Ok(s)
}
// Exact bounded decimal addition; no binary float or fixed-width monetary integer.
fn total(a: &str, b: &str) -> Result<String> {
    let parts = |s: &str| {
        let (w, f) = s.split_once('.').unwrap_or((s, ""));
        (w.to_owned(), f.to_owned())
    };
    let (aw, af) = parts(a);
    let (bw, bf) = parts(b);
    let scale = af.len().max(bf.len());
    let digits =
        |w: String, f: String| format!("{w}{f}{}", "0".repeat(scale - f.len())).into_bytes();
    let a = digits(aw, af);
    let b = digits(bw, bf);
    let mut result = Vec::new();
    let mut carry = 0u8;
    for i in 0..a.len().max(b.len()) {
        let x = a.iter().rev().nth(i).map_or(0, |c| c - b'0');
        let y = b.iter().rev().nth(i).map_or(0, |c| c - b'0');
        let sum = x + y + carry;
        result.push(b'0' + sum % 10);
        carry = sum / 10;
    }
    if carry > 0 {
        result.push(b'0' + carry);
    }
    result.reverse();
    if scale > 0 {
        result.insert(result.len() - scale, b'.');
    }
    decimal(&Value::String(
        String::from_utf8(result).map_err(|_| invalid())?,
    ))
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
                    quantity: sum.clone(),
                    market_value: None,
                    average_entry_price: None,
                    instrument_currency: None,
                    market_value_currency: None,
                });
            }
            Ok(Balance {
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
                broker_order_id,
                symbol,
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
