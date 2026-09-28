use super::*;
use crate::{
    protocol::{
        AssetClass, LiveOrderDisposition, LiveOrderFee, LiveOrderTradeFact, OrderProposalStatus,
    },
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
    if !valid_order_id(s) {
        return Err(invalid());
    }
    Ok(s.into())
}
pub(super) fn valid_order_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 40
        && !value.starts_with('0')
        && value.bytes().all(|byte| byte.is_ascii_digit())
}
pub(super) fn valid_live_cancel_number(value: &str) -> bool {
    valid_order_id(value) && value.parse::<i64>().is_ok_and(|order_id| order_id > 0)
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
pub(super) fn allows_live_cancel_observation(path: &str) -> bool {
    if let Some(order_id) = path.strip_prefix("/api/v2/spot/trade/orderInfo?orderId=") {
        return valid_live_cancel_number(order_id);
    }
    let Some((route, query)) = path.split_once('?') else {
        return false;
    };
    if route != "/api/v2/spot/trade/fills" {
        return false;
    }
    let Some(query) = query.strip_prefix("limit=100&orderId=") else {
        return false;
    };
    let Some((order_id, cursor)) = query.split_once("&idLessThan=") else {
        return valid_live_cancel_number(query);
    };
    valid_live_cancel_number(order_id) && valid_order_id(cursor)
}
fn valid_client_oid(value: &str) -> bool {
    value.starts_with("tx-")
        && value.len() <= 50
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

pub(super) fn live_client_order_id(attempt_id: &str) -> Result<String> {
    let attempt_id = uuid::Uuid::parse_str(attempt_id).map_err(|_| invalid())?;
    let client_oid = format!("tx-{}", attempt_id.simple());
    if valid_client_oid(&client_oid) {
        Ok(client_oid)
    } else {
        Err(invalid())
    }
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

fn response_json(
    endpoint: ProviderEndpoint,
    path: &str,
    headers: HeaderMap,
    signature: Option<&str>,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<Value> {
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let bytes = http.get(endpoint, path, headers)?;
    if bytes.len() as u64 > MAX_RESPONSE {
        return Err(invalid());
    }
    let value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if contains_secret(&value, secrets)
        || signature.is_some_and(|signature| contains_secret(&value, &[signature.into()]))
    {
        return Err(invalid());
    }
    business(value)
}

fn server_clock(
    endpoint: ProviderEndpoint,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<(u64, Instant)> {
    let started = Instant::now();
    let time = response_json(
        endpoint,
        "/api/v2/public/time",
        HeaderMap::new(),
        None,
        secrets,
        http,
        current,
    )?;
    let server = time["serverTime"]
        .as_str()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| valid_time(*value))
        .ok_or_else(|| TradeXError::new("CLOCK_SKEW"))?;
    if started.elapsed() > Duration::from_secs(2) {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    Ok((server, Instant::now()))
}

fn signed_get(
    endpoint: ProviderEndpoint,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    server: u64,
    sampled: Instant,
    path: &str,
) -> Result<Value> {
    if secrets.len() != 3 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let (headers, signature) = signed_headers(
        server,
        sampled,
        secrets,
        "GET",
        path,
        None,
        endpoint == ProviderEndpoint::BitgetDemo,
    )?;
    response_json(
        endpoint,
        path,
        headers,
        Some(&signature),
        secrets,
        http,
        current,
    )
}

fn signed_headers(
    server: u64,
    sampled: Instant,
    secrets: &[String],
    method: &str,
    path: &str,
    body: Option<&Value>,
    demo: bool,
) -> Result<(HeaderMap, Zeroizing<String>)> {
    if sampled.elapsed() > Duration::from_secs(60) {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    let timestamp = server
        .checked_add(sampled.elapsed().as_millis() as u64)
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
    if demo {
        headers.insert("paptrading", HeaderValue::from_static("1"));
    }
    Ok((headers, signature))
}

pub(super) fn prepare_live_cancel_preflight(
    account: &AccountConnection,
    intent: &CancellationIntent,
    secrets: &[String],
    http: &impl ProviderHttp,
) -> Result<String> {
    let order_id = intent
        .provider_order_id
        .strip_prefix("normal:")
        .filter(|value| valid_live_cancel_number(value))
        .ok_or_else(|| TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"))?;
    if account.provider_id != "bitget"
        || account.environment != "LIVE"
        || account.workspace_id != intent.workspace_id
        || account.connection_id != intent.account_id
        || intent.environment != ExecutionContext::BitgetLive
        || secrets.len() != 3
    {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    }
    crate::storage::validate_cancellation_snapshot(account, intent)?;
    let data = account
        .data
        .as_ref()
        .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
    let saved = data
        .bitget_order_book
        .as_ref()
        .and_then(|book| {
            book.orders
                .iter()
                .find(|order| order.kind == "NORMAL" && order.provider_order_id == order_id)
        })
        .ok_or_else(|| TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"))?;
    let current = || true;
    let (server, sampled) = server_clock(ProviderEndpoint::BitgetLive, secrets, http, &current)?;
    let signed = |path: &str| {
        signed_get(
            ProviderEndpoint::BitgetLive,
            secrets,
            http,
            &current,
            server,
            sampled,
            path,
        )
    };
    let identity = signed("/api/v2/spot/account/info")?;
    if id(&identity, "userId")? != data.remote_account_id {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let rows = signed(&format!("/api/v2/spot/trade/orderInfo?orderId={order_id}"))?;
    let rows = rows.as_array().ok_or_else(invalid)?;
    if rows.len() != 1 {
        return Err(TradeXError::new("ORDER_NOT_CANCELLABLE"));
    }
    let row = &rows[0];
    if id(row, "userId")? != data.remote_account_id {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let order = bitget_order(row, "normal", &data.remote_account_id, order_id.to_owned())?;
    if text(row, "tpslType", 16)? != "normal"
        || order.symbol != intent.symbol
        || order.side != intent.side
        || market::canonical_instrument_id("bitget", &order.symbol).as_deref()
            != Some(intent.instrument_id.as_str())
        || !matches!(
            order.normalized_status.as_str(),
            "OPEN" | "PARTIALLY_FILLED"
        )
    {
        return Err(TradeXError::new("ORDER_NOT_CANCELLABLE"));
    }
    if order.quantity.as_deref() != Some(intent.quantity.as_str())
        || order.filled_quantity.as_deref() != Some(intent.filled_quantity.as_str())
        || order.remaining_quantity.as_deref() != Some(intent.remaining_quantity.as_str())
        || order.provider_status.to_ascii_uppercase() != intent.provider_status
    {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    }
    if saved.symbol != order.symbol
        || saved.side != order.side
        || saved.quantity != order.quantity
        || saved.notional != order.notional
        || saved.filled_quantity != order.filled_quantity
        || saved.filled_value != order.filled_value
        || saved.remaining_quantity != order.remaining_quantity
        || saved.currency != order.currency
        || saved.provider_status != order.provider_status
        || saved.normalized_status != order.normalized_status
        || saved.created_at != order.created_at
        || saved.updated_at != order.updated_at
    {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    }
    Ok(intent.provider_status.clone())
}

pub(super) enum LiveCancelAcknowledgement {
    Accepted,
    Rejected(&'static str),
    Unknown,
}

pub(super) fn send_live_cancel(
    secrets: &[String],
    symbol: &str,
    order_id: &str,
    http: &impl ProviderHttp,
) -> Result<ProviderHttpResponse> {
    if !valid_live_cancel_number(order_id) || !matches!(symbol, "BTCUSDT" | "ETHUSDT") {
        return Err(invalid());
    }
    let current = || true;
    let (server, sampled) = server_clock(ProviderEndpoint::BitgetLive, secrets, http, &current)?;
    let path = "/api/v2/spot/trade/cancel-order";
    let body = serde_json::json!({"symbol":symbol,"orderId":order_id});
    let headers = live_signed_headers(server, sampled, secrets, "POST", path, Some(&body))?;
    http.request(
        ProviderEndpoint::BitgetLive,
        ProviderHttpMethod::Post,
        path,
        headers,
        Some(&body),
    )
}

pub(super) fn live_cancel_acknowledgement(
    response: &ProviderHttpResponse,
    secrets: &[String],
    expected_order_id: &str,
) -> LiveCancelAcknowledgement {
    if response.body.len() as u64 > MAX_RESPONSE {
        return LiveCancelAcknowledgement::Unknown;
    }
    let Ok(value) = serde_json::from_slice::<Value>(&response.body) else {
        return LiveCancelAcknowledgement::Unknown;
    };
    if contains_secret(&value, secrets) {
        return LiveCancelAcknowledgement::Unknown;
    }
    match value["code"].as_str() {
        Some("00000") => {
            if value["data"]["orderId"].as_str() == Some(expected_order_id) {
                LiveCancelAcknowledgement::Accepted
            } else {
                LiveCancelAcknowledgement::Unknown
            }
        }
        Some(code) => LiveCancelAcknowledgement::Rejected(match code {
            "40001" | "40002" | "40003" | "40006" | "40009" | "40011" | "40012" | "40014"
            | "40018" | "40025" | "40036" | "40037" | "40038" | "40040" | "40041" => {
                "PROVIDER_AUTH_FAILED"
            }
            "40010" | "40022" | "40026" | "40027" => "PROVIDER_PERMISSION_BLOCKED",
            "429" => "PROVIDER_RATE_LIMITED",
            "40005" | "40008" | "40078" => "CLOCK_SKEW",
            _ => "PROVIDER_ORDER_REJECTED",
        }),
        None => LiveCancelAcknowledgement::Unknown,
    }
}

fn live_signed_headers(
    server: u64,
    sampled: Instant,
    secrets: &[String],
    method: &str,
    path: &str,
    body: Option<&Value>,
) -> Result<HeaderMap> {
    if secrets.len() != 3 {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    signed_headers(server, sampled, secrets, method, path, body, false).map(|(headers, _)| headers)
}

pub(super) fn read(
    endpoint: ProviderEndpoint,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    old: Option<&AccountData>,
    exact_cancel_order_id: Option<&str>,
) -> Result<Observation> {
    let (server, sampled) = server_clock(endpoint, secrets, http, current)?;
    let signed = |path: &str| signed_get(endpoint, secrets, http, current, server, sampled, path);
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
    let mut bitget_order_book = if endpoint == ProviderEndpoint::BitgetLive {
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
    let mut live_order_settlements = Vec::new();
    if let Some(broker_order_id) = exact_cancel_order_id {
        if endpoint != ProviderEndpoint::BitgetLive {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let order_id = broker_order_id
            .strip_prefix("normal:")
            .filter(|value| valid_order_id(value))
            .ok_or_else(|| TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"))?;
        let saved = old
            .and_then(|data| data.bitget_order_book.as_ref())
            .and_then(|book| {
                book.orders
                    .iter()
                    .find(|order| order.kind == "NORMAL" && order.provider_order_id == order_id)
            })
            .ok_or_else(|| TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"))?;
        let (exact_order, raw, observation, exact_fills) =
            exact_live_cancel_order(&signed, &identity, broker_order_id, &saved.symbol, current)?;
        if exact_order.symbol != saved.symbol {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
        }
        let book = bitget_order_book
            .as_mut()
            .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
        if book.orders.iter().any(|order| {
            order.kind == "NORMAL"
                && order.provider_order_id == exact_order.provider_order_id
                && order.symbol != exact_order.symbol
        }) {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
        }
        book.orders.retain(|order| {
            order.kind != "NORMAL" || order.provider_order_id != exact_order.provider_order_id
        });
        book.orders.push(exact_order.clone());
        book.fills.retain(|fill| fill.provider_order_id != order_id);
        book.fills.extend(exact_fills);
        book.fills
            .sort_by(|left, right| right.observed_at.cmp(&left.observed_at));
        book.fills.truncate(2_000);
        let exact_open_order = open_order(&exact_order, &raw)?;
        orders.retain(|order| order.broker_order_id != broker_order_id);
        if matches!(
            exact_order.normalized_status.as_str(),
            "OPEN" | "PARTIALLY_FILLED"
        ) {
            orders.push(exact_open_order);
        }
        live_order_settlements.push(observation);
    }
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
        "Private stream unavailable · REST reconciliation: Bitget Classic Spot v2 account and order observations update only through explicit signed REST connection/refresh requests. Stream health remains NOT_CONFIGURED.".into(),
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
        live_order_settlements,
    })
}

fn exact_live_cancel_order(
    signed: &impl Fn(&str) -> Result<Value>,
    identity: &str,
    broker_order_id: &str,
    expected_symbol: &str,
    current: &impl Fn() -> bool,
) -> Result<(
    BitgetSpotOrder,
    Value,
    LiveOrderObservation,
    Vec<BitgetSpotFill>,
)> {
    let order_id = broker_order_id
        .strip_prefix("normal:")
        .filter(|value| valid_live_cancel_number(value))
        .ok_or_else(|| TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"))?;
    let rows = signed(&format!("/api/v2/spot/trade/orderInfo?orderId={order_id}"))?;
    let rows = rows.as_array().ok_or_else(invalid)?;
    if rows.len() != 1 {
        return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
    }
    let raw = rows[0].clone();
    if id(&raw, "userId")? != identity {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    if id(&raw, "orderId")? != order_id
        || text(&raw, "tpslType", 16)? != "normal"
        || identifier(&raw, "symbol")? != expected_symbol
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let order = bitget_order(&raw, "normal", identity, order_id.to_owned())?;
    let (trade_facts, exact_fills, fees) =
        exact_live_order_fills(signed, identity, &order, current)?;
    let filled_quantity = order
        .filled_quantity
        .as_deref()
        .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
    let filled_value = order
        .filled_value
        .as_deref()
        .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
    let provider_observed_at =
        timestamp(&raw, "uTime")?.ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
    let disposition = match order.normalized_status.as_str() {
        "OPEN" | "PARTIALLY_FILLED" => LiveOrderDisposition::Working,
        "FILLED" | "CANCELED" | "REJECTED" => LiveOrderDisposition::Terminal,
        _ => LiveOrderDisposition::Unknown,
    };
    let observation = LiveOrderObservation {
        provider_order_id: broker_order_id.into(),
        raw_status: order.provider_status.clone(),
        disposition,
        order_quantity: order.quantity.clone(),
        filled_quantity: Some(filled_quantity.into()),
        remaining_quantity: order.remaining_quantity.clone(),
        filled_value: Some(filled_value.into()),
        fees: Some(fees),
        trade_facts_complete: true,
        trade_facts,
        provider_observed_at: Some(provider_observed_at),
        source: "bitget.live.exact-order".into(),
    };
    Ok((order, raw, observation, exact_fills))
}

fn exact_live_order_fills(
    signed: &impl Fn(&str) -> Result<Value>,
    identity: &str,
    order: &BitgetSpotOrder,
    current: &impl Fn() -> bool,
) -> Result<(
    Vec<LiveOrderTradeFact>,
    Vec<BitgetSpotFill>,
    Vec<LiveOrderFee>,
)> {
    const MAX_PAGES: usize = 20;
    let mut cursor: Option<String> = None;
    let mut seen = HashSet::new();
    let mut facts = Vec::new();
    let mut fills = Vec::new();
    let mut fee_totals = HashMap::<String, String>::new();
    for page_number in 0..MAX_PAGES {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let path = cursor.as_ref().map_or_else(
            || {
                format!(
                    "/api/v2/spot/trade/fills?limit=100&orderId={}",
                    order.provider_order_id
                )
            },
            |cursor| {
                format!(
                    "/api/v2/spot/trade/fills?limit=100&orderId={}&idLessThan={cursor}",
                    order.provider_order_id
                )
            },
        );
        let page = signed(&path)?;
        let rows = page.as_array().ok_or_else(invalid)?;
        if rows.len() > 100 || facts.len() + rows.len() > MAX_PAGES * 100 {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        let mut minimum: Option<String> = None;
        let mut prior: Option<String> = None;
        for row in rows {
            let trade_id = id(row, "tradeId")?;
            let provider_order_id = id(row, "orderId")?;
            let symbol = identifier(row, "symbol")?;
            if id(row, "userId")? != identity {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
            }
            if provider_order_id != order.provider_order_id
                || symbol != order.symbol
                || text(row, "side", 4)?.to_ascii_uppercase() != order.side
                || cursor
                    .as_ref()
                    .is_some_and(|value| !older(&trade_id, value))
                || prior.as_ref().is_some_and(|value| !older(&trade_id, value))
                || !seen.insert(trade_id.clone())
            {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            if minimum.as_ref().is_none_or(|value| older(&trade_id, value)) {
                minimum = Some(trade_id.clone());
            }
            prior = Some(trade_id.clone());
            let quantity = positive(&row["size"])?;
            let value = positive(&row["amount"])?;
            let fees = live_fee_detail(row)?;
            let executed_at = timestamp(row, "cTime")?
                .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
            let observed_at = timestamp(row, "uTime")?.unwrap_or_else(|| executed_at.clone());
            for fee in &fees {
                let total = fee_totals
                    .entry(fee.asset.clone())
                    .or_insert_with(|| "0".into());
                *total = crate::portfolio::decimal_add(total, &fee.amount)?;
            }
            facts.push(LiveOrderTradeFact {
                provider_trade_id: trade_id.clone(),
                quantity: quantity.clone(),
                value: value.clone(),
                fees: fees.clone(),
                provider_executed_at: Some(executed_at.clone()),
            });
            fills.push(BitgetSpotFill {
                provider_trade_id: trade_id,
                provider_order_id,
                symbol,
                side: order.side.clone(),
                price: Some(positive(&row["priceAvg"])?),
                quantity,
                value: Some(value),
                currency: order.currency.clone(),
                observed_at,
                provider_executed_at: Some(executed_at),
                fees: Some(fees),
            });
        }
        if rows.len() < 100 {
            let mut quantity_total = "0".to_owned();
            let mut value_total = "0".to_owned();
            for fact in &facts {
                quantity_total = crate::portfolio::decimal_add(&quantity_total, &fact.quantity)?;
                value_total = crate::portfolio::decimal_add(&value_total, &fact.value)?;
            }
            if order.filled_quantity.as_deref() != Some(quantity_total.as_str())
                || order.filled_value.as_deref() != Some(value_total.as_str())
            {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            let fees = fee_totals
                .into_iter()
                .map(|(asset, amount)| LiveOrderFee { asset, amount })
                .collect();
            return Ok((facts, fills, fees));
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

fn live_fee_detail(fill: &Value) -> Result<Vec<LiveOrderFee>> {
    let detail = fill.get("feeDetail").ok_or_else(invalid)?;
    let asset = identifier(detail, "feeCoin")?;
    let amount = crate::provider_io::decimal(&Value::String(text(detail, "totalFee", 128)?))?;
    let amount = amount.strip_prefix('-').unwrap_or(&amount).to_owned();
    Ok(vec![LiveOrderFee { asset, amount }])
}

fn permissions(account: &Value) -> Result<PermissionReview> {
    let mut p = PermissionReview {
        detected: vec![
            "account.read".into(),
            "positions.read".into(),
            "orders.read".into(),
        ],
        ..Default::default()
    };
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
            let provider_executed_at = timestamp(row, "cTime")?;
            let observed_at = timestamp(row, "uTime")?
                .or(provider_executed_at.clone())
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
                provider_executed_at,
                fees: row.get("feeDetail").and_then(|_| live_fee_detail(row).ok()),
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

    #[test]
    fn exact_live_fills_require_complete_order_matched_fees_and_totals() {
        let order = BitgetSpotOrder {
            provider_order_id: "200".into(),
            kind: "NORMAL".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            quantity: Some("0.0002".into()),
            notional: None,
            filled_quantity: Some("0.0002".into()),
            filled_value: Some("14.000025".into()),
            remaining_quantity: Some("0".into()),
            currency: Some("USDT".into()),
            provider_status: "filled".into(),
            normalized_status: "FILLED".into(),
            origin: "external".into(),
            created_at: None,
            updated_at: None,
        };
        let signed = |path: &str| {
            assert_eq!(path, "/api/v2/spot/trade/fills?limit=100&orderId=200");
            Ok(serde_json::json!([{
                "userId":"9007199254740993", "orderId":"200", "tradeId":"300",
                "symbol":"BTCUSDT", "side":"buy", "priceAvg":"70000.125",
                "size":"0.0002", "amount":"14.000025",
                "feeDetail":{"feeCoin":"BTC","totalFee":"-0.0000007"},
                "cTime":"1788849500", "uTime":"1788849600"
            }]))
        };
        let (facts, fills, fees) =
            exact_live_order_fills(&signed, "9007199254740993", &order, &|| true).unwrap();
        assert_eq!(facts[0].provider_trade_id, "300");
        assert_eq!(facts[0].fees[0].asset, "BTC");
        assert_eq!(facts[0].fees[0].amount, "0.0000007");
        assert_eq!(
            fills[0].provider_executed_at.as_deref(),
            Some("2026-09-08T06:38:20Z")
        );
        assert_eq!(fees[0].amount, "0.0000007");

        let incomplete = |_: &str| {
            Ok(serde_json::json!([{
                "userId":"9007199254740993", "orderId":"201", "tradeId":"301",
                "symbol":"BTCUSDT", "side":"buy", "priceAvg":"70000.125",
                "size":"0.0002", "amount":"14.000025",
                "cTime":"1788849500"
            }]))
        };
        assert_eq!(
            exact_live_order_fills(&incomplete, "9007199254740993", &order, &|| true)
                .unwrap_err()
                .code,
            "PROVIDER_DATA_INCOMPLETE"
        );
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
    if secrets.len() != 3 {
        return Err(TradeXError::new("CLOCK_SKEW"));
    }
    signed_headers(
        clock.server_time,
        clock.sampled_at,
        secrets,
        method,
        path,
        body,
        true,
    )
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

fn bitget_symbol(
    proposal: &OrderProposal,
    connection_id: &str,
    environment: ExecutionContext,
) -> Result<String> {
    let fields = &proposal.fields;
    if !matches!(
        proposal.status,
        OrderProposalStatus::NeedsApproval | OrderProposalStatus::Consumed
    ) || fields.environment != environment
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
    if attempt.environment != "DEMO"
        || attempt.proposal_id != proposal.proposal_id
        || attempt.proposal_hash != proposal.proposal_hash
    {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    bitget_order_intent(
        proposal,
        &attempt.connection_id,
        ExecutionContext::BitgetDemo,
    )
}

fn bitget_order_intent(
    proposal: &OrderProposal,
    connection_id: &str,
    environment: ExecutionContext,
) -> Result<DemoOrderIntent> {
    let fields = &proposal.fields;
    if fields.maximum_spend.is_some() {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    let symbol = bitget_symbol(proposal, connection_id, environment)?;
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
    let symbol = bitget_symbol(proposal, connection_id, ExecutionContext::BitgetDemo)?;
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

pub(super) fn query_live_reconciliation_candidate(
    proposal: &OrderProposal,
    attempt_id: &str,
    provider_client_order_id: &str,
    connection_id: &str,
    remote_account_id: &str,
    window_started_at: &str,
    window_ends_at: &str,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<Option<ProviderOrderCandidate>> {
    if secrets.len() != 3
        || proposal.status != OrderProposalStatus::Consumed
        || proposal.fields.environment != ExecutionContext::BitgetLive
        || remote_account_id.is_empty()
        || live_client_order_id(attempt_id)? != provider_client_order_id
    {
        return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
    }
    let intent = bitget_order_intent(proposal, connection_id, ExecutionContext::BitgetLive)?;
    let endpoint = ProviderEndpoint::BitgetLive;
    let (server, sampled) = server_clock(endpoint, secrets, http, current)?;
    let signed = |path: &str| signed_get(endpoint, secrets, http, current, server, sampled, path);
    let account = signed("/api/v2/spot/account/info")?;
    if id(&account, "userId")? != remote_account_id {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let path = format!("/api/v2/spot/trade/orderInfo?clientOid={provider_client_order_id}");
    let response = signed(&path)?;
    let rows = response.as_array().ok_or_else(invalid)?;
    if rows.is_empty() {
        return Ok(None);
    }
    if rows.len() != 1 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let row = &rows[0];
    let account_id = id(row, "userId")?;
    let provider_symbol = text(row, "symbol", 32)?;
    let order_id = id(row, "orderId")?;
    let client_oid = text(row, "clientOid", 50)?;
    let side = text(row, "side", 8)?;
    let order_type = text(row, "orderType", 8)?;
    let status = text(row, "status", 32)?;
    let quantity = positive(&row["size"])?;
    let submitted_at =
        timestamp(row, "cTime")?.ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
    let submitted = time::OffsetDateTime::parse(
        &submitted_at,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| invalid())?;
    let window_started = time::OffsetDateTime::parse(
        window_started_at,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| invalid())?;
    let window_ends = time::OffsetDateTime::parse(
        window_ends_at,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| invalid())?;
    let quantity_matches =
        provider_io::decimal_cmp(&quantity, &intent.size)? == std::cmp::Ordering::Equal;
    let limit_price = (intent.order_type == "limit")
        .then(|| positive(&row["price"]))
        .transpose()?;
    let force = (intent.order_type == "limit")
        .then(|| text(row, "force", 16))
        .transpose()?;
    let tpsl_type = text(row, "tpslType", 16)?;
    let limit_fields_match = intent.order_type != "limit"
        || (provider_io::decimal_cmp(
            limit_price.as_deref().ok_or_else(invalid)?,
            intent.price.as_deref().ok_or_else(invalid)?,
        )? == std::cmp::Ordering::Equal
            && force.as_deref() == intent.force);
    if account_id != remote_account_id
        || provider_symbol != intent.symbol
        || client_oid != provider_client_order_id
        || side != intent.side
        || order_type != intent.order_type
        || normalized_status(&status) == "UNKNOWN"
        || tpsl_type != "normal"
        || !quantity_matches
        || !limit_fields_match
        || submitted < window_started
        || submitted > window_ends
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    Ok(Some(ProviderOrderCandidate {
        provider_order_id: order_id,
        provider_symbol,
        side: proposal.fields.side,
        provider_status: status,
        order_type,
        quantity: Some(quantity),
        quote_quantity: None,
        limit_price,
        time_in_force: None,
        force,
        tpsl_type: Some(tpsl_type),
        submitted_at: Some(submitted_at),
        provider_client_id: Some(client_oid),
    }))
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
