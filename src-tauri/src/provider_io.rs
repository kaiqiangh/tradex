use crate::{
    protocol::{Result, TradeXError},
    providers::*,
};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use serde_json::Value;
use std::{io::Read, time::Duration};
use zeroize::Zeroizing;

#[path = "binance.rs"]
mod binance;
#[path = "trading212.rs"]
mod trading212;

const MAX_RESPONSE: u64 = 2 * 1024 * 1024;
const SERVICE: &str = "com.tradex.broker.credentials";

// No Debug/Serialize implementation: this value stays inside native capture, Keychain and signing.
pub struct Credentials(Zeroizing<Vec<u8>>);

impl Credentials {
    pub fn new(values: Vec<String>) -> Result<Self> {
        let values = Zeroizing::new(values);
        if values.len() != 2
            || values
                .iter()
                .any(|v| v.is_empty() || v.len() > 512 || !v.bytes().all(|b| b.is_ascii_graphic()))
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        Ok(Self(Zeroizing::new(serde_json::to_vec(&*values).map_err(
            |_| TradeXError::new("CREDENTIAL_STORE_FAILED"),
        )?)))
    }
    fn values(&self) -> Result<Zeroizing<Vec<String>>> {
        serde_json::from_slice(&self.0)
            .map(Zeroizing::new)
            .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))
    }
}

/// External boundary shared by the native OS store and isolated fault tests; never an IPC service.
pub trait CredentialVault {
    fn put(&self, reference: &str, credentials: &Credentials) -> Result<()>;
    fn get(&self, reference: &str) -> Result<Credentials>;
    fn remove(&self, reference: &str) -> Result<()>;
}

pub struct NativeVault;

#[cfg(target_os = "macos")]
impl CredentialVault for NativeVault {
    fn put(&self, reference: &str, credentials: &Credentials) -> Result<()> {
        security_framework::passwords::set_generic_password(SERVICE, reference, &credentials.0)
            .map_err(|_| TradeXError::new("CREDENTIAL_STORE_FAILED"))
    }
    fn get(&self, reference: &str) -> Result<Credentials> {
        let bytes = Zeroizing::new(
            security_framework::passwords::get_generic_password(SERVICE, reference)
                .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?,
        );
        if bytes.len() > 4096 {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        let values = serde_json::from_slice(&bytes)
            .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
        Credentials::new(values).map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))
    }
    fn remove(&self, reference: &str) -> Result<()> {
        match security_framework::passwords::delete_generic_password(SERVICE, reference) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25300 => Ok(()), // errSecItemNotFound: idempotent own-item cleanup.
            Err(_) => Err(TradeXError::new("CREDENTIAL_DELETE_FAILED")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderEndpoint {
    AlpacaPaper,
    Trading212Demo,
    Trading212Live,
    BinanceTestnet,
    BinanceLive,
}
impl ProviderEndpoint {
    pub fn base_url(self) -> &'static str {
        match self {
            Self::AlpacaPaper => "https://paper-api.alpaca.markets",
            Self::Trading212Demo => "https://demo.trading212.com",
            Self::Trading212Live => "https://live.trading212.com",
            Self::BinanceTestnet => "https://testnet.binance.vision",
            Self::BinanceLive => "https://api.binance.com",
        }
    }
    fn is_binance(self) -> bool {
        matches!(self, Self::BinanceTestnet | Self::BinanceLive)
    }
    fn allows(self, path: &str) -> bool {
        match self {
            Self::AlpacaPaper => allowed_path(path),
            Self::BinanceTestnet | Self::BinanceLive => binance::allows(self, path),
            _ => matches!(
                path,
                "/api/v0/equity/account/summary"
                    | "/api/v0/equity/positions"
                    | "/api/v0/equity/orders"
            ),
        }
    }
}
pub trait ProviderHttp {
    fn get(&self, endpoint: ProviderEndpoint, path: &str, headers: HeaderMap) -> Result<Vec<u8>>;
}

#[derive(Default)]
pub struct BrokerHttp(std::cell::OnceCell<Result<Client>>);

impl BrokerHttp {
    fn client_builder() -> reqwest::blocking::ClientBuilder {
        Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(12))
            .connect_timeout(Duration::from_secs(4))
    }

    fn client(&self) -> Result<&Client> {
        self.0
            .get_or_init(|| {
                Self::client_builder()
                    .build()
                    .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

#[cfg(test)]
#[path = "../tests/support/provider_http.rs"]
mod http_tests;

impl ProviderHttp for BrokerHttp {
    fn get(&self, endpoint: ProviderEndpoint, path: &str, headers: HeaderMap) -> Result<Vec<u8>> {
        if !endpoint.allows(path) {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let response = self
            .client()?
            .get(format!("{}{path}", endpoint.base_url()))
            .headers(headers)
            .send()
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        let status = response.status().as_u16();
        match status {
            200 => (),
            400 if endpoint.is_binance() => (),
            401 | 403 => return Err(TradeXError::new("PROVIDER_AUTH_FAILED")),
            418 | 429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
            _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
        }
        if response.content_length().is_some_and(|n| n > MAX_RESPONSE) {
            return Err(invalid());
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err(invalid());
        }
        if status != 200 {
            let code = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| v["code"].as_i64());
            return Err(TradeXError::new(match code {
                Some(-1021) => "CLOCK_SKEW",
                Some(-1022 | -2014 | -2015) => "PROVIDER_AUTH_FAILED",
                Some(-1003 | -1015) => "PROVIDER_RATE_LIMITED",
                _ => "PROVIDER_RESPONSE_INVALID",
            }));
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum JobKind {
    Connect,
    Probe,
    Disconnect,
}

pub struct ProviderJob {
    pub(crate) account: AccountConnection,
    pub(crate) kind: JobKind,
    pub(crate) session: String,
    pub(crate) request_id: String,
}

pub(crate) struct Observation {
    pub data: AccountData,
    pub permissions: PermissionReview,
}
pub struct ProviderOutcome {
    pub(crate) observation: Option<Observation>,
    pub(crate) error: Option<TradeXError>,
    pub(crate) credential: String,
}

impl ProviderJob {
    pub fn connection(&self) -> &AccountConnection {
        &self.account
    }

    pub fn run(
        &self,
        vault: &impl CredentialVault,
        capture: impl FnOnce(&ProviderDefinition) -> Result<Credentials>,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let reference = self.account.credential_ref();
        if self.kind == JobKind::Disconnect {
            let result = vault.remove(&reference);
            return ProviderOutcome {
                observation: None,
                credential: if result.is_ok() {
                    "MISSING"
                } else {
                    "DELETE_PENDING"
                }
                .into(),
                error: result.err(),
            };
        }
        let mut credential = "MISSING";
        let result = (|| -> Result<Observation> {
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let definition = definition(&self.account.provider_id, &self.account.environment)?;
            let endpoint = match (
                self.account.provider_id.as_str(),
                self.account.environment.as_str(),
            ) {
                ("alpaca", "PAPER") => ProviderEndpoint::AlpacaPaper,
                ("trading212", "DEMO") => ProviderEndpoint::Trading212Demo,
                ("trading212", "LIVE") => ProviderEndpoint::Trading212Live,
                ("binance", "TESTNET") => ProviderEndpoint::BinanceTestnet,
                ("binance", "LIVE") => ProviderEndpoint::BinanceLive,
                _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            };
            if self.kind == JobKind::Connect {
                let secret = capture(&definition)?;
                if !current() {
                    return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                }
                vault.put(&reference, &secret)?;
            }
            let secret = vault.get(&reference)?;
            credential = "CONFIGURED";
            let mut values = secret.values()?;
            if endpoint.is_binance() {
                return binance::read(
                    endpoint,
                    &values,
                    http,
                    &current,
                    self.account.data.as_ref(),
                );
            }
            let mut auth = HeaderMap::new();
            if endpoint == ProviderEndpoint::AlpacaPaper {
                for (name, value) in [
                    ("APCA-API-KEY-ID", &values[0]),
                    ("APCA-API-SECRET-KEY", &values[1]),
                ] {
                    let mut header = HeaderValue::from_str(value)
                        .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
                    header.set_sensitive(true);
                    auth.insert(name, header);
                }
            } else {
                use base64::Engine;
                if values[0].contains(':') {
                    return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
                }
                let combined = Zeroizing::new(format!("{}:{}", values[0], values[1]));
                let encoded = base64::engine::general_purpose::STANDARD.encode(combined.as_bytes());
                let header_text = Zeroizing::new(format!("Basic {encoded}"));
                let mut header = HeaderValue::from_str(&header_text)
                    .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
                header.set_sensitive(true);
                auth.insert("Authorization", header);
                values.push(encoded); // Encoded credentials must not be reflected into observations either.
            }
            let query = |path: &str| -> Result<Value> {
                if !current() {
                    return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                }
                let bytes = http.get(endpoint, path, auth.clone())?;
                if bytes.len() as u64 > MAX_RESPONSE {
                    return Err(invalid());
                }
                let value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
                if contains_secret(&value, &values) {
                    return Err(invalid());
                }
                Ok(value)
            };
            let observation = if endpoint == ProviderEndpoint::AlpacaPaper {
                let account = query("/v2/account")?;
                let remote_id = id(&account, "id")?;
                if self
                    .account
                    .data
                    .as_ref()
                    .is_some_and(|old| old.remote_account_id != remote_id)
                {
                    return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
                }
                let positions = query("/v2/positions")?;
                let mut orders = Vec::new();
                let mut cursor = None;
                let mut seen = std::collections::HashSet::new();
                loop {
                    let path = format!(
                        "/v2/orders?status=open&limit=500&direction=asc&nested=false{}",
                        cursor
                            .as_ref()
                            .map(|id| format!("&after_order_id={id}"))
                            .unwrap_or_default()
                    );
                    let value = query(&path)?;
                    let page = value.as_array().ok_or_else(invalid)?;
                    if page.len() > 500 {
                        return Err(invalid());
                    }
                    for order in page {
                        let order_id = id(order, "id")?;
                        if !seen.insert(order_id.clone()) {
                            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                        }
                        cursor = Some(order_id);
                        orders.push(order.clone());
                    }
                    if page.len() < 500 {
                        break;
                    }
                    if orders.len() >= 10_000 {
                        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                    }
                }
                alpaca(account, positions, Value::Array(orders))?
            } else {
                let account = query("/api/v0/equity/account/summary")?;
                let remote_id = trading212::account_id(&account)?;
                if self
                    .account
                    .data
                    .as_ref()
                    .is_some_and(|old| old.remote_account_id != remote_id)
                {
                    return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
                }
                trading212::observe(
                    account,
                    query("/api/v0/equity/positions")?,
                    query("/api/v0/equity/orders")?,
                )?
            };
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            Ok(observation)
        })();
        if self.kind == JobKind::Connect && !current() {
            credential = if vault.remove(&reference).is_ok() {
                "MISSING"
            } else {
                "DELETE_PENDING"
            };
        }
        if result
            .as_ref()
            .is_err_and(|e| e.code == "PROVIDER_AUTH_FAILED")
        {
            credential = "INVALID";
        }
        match result {
            Ok(observation) => ProviderOutcome {
                observation: Some(observation),
                error: None,
                credential: credential.into(),
            },
            Err(error) => ProviderOutcome {
                observation: None,
                error: Some(error),
                credential: credential.into(),
            },
        }
    }

    pub fn cleanup_after_failed_commit(
        &self,
        reply: &Value,
        vault: &impl CredentialVault,
    ) -> Option<Result<()>> {
        if self.kind == JobKind::Connect && reply["ok"] == false {
            // The durable pending row owns this exact reference, even if cleanup must be retried after restart.
            return Some(vault.remove(&self.account.credential_ref()));
        }
        None
    }
}

fn allowed_path(path: &str) -> bool {
    if matches!(
        path,
        "/v2/account"
            | "/v2/positions"
            | "/v2/orders?status=open&limit=500&direction=asc&nested=false"
    ) {
        return true;
    }
    path.strip_prefix("/v2/orders?status=open&limit=500&direction=asc&nested=false&after_order_id=")
        .is_some_and(|id| id.len() == 36 && uuid::Uuid::parse_str(id).is_ok())
}

fn invalid() -> TradeXError {
    TradeXError::new("PROVIDER_RESPONSE_INVALID")
}

fn contains_secret(value: &Value, secrets: &[String]) -> bool {
    let contains = |text: &str| secrets.iter().any(|secret| text.contains(secret));
    match value {
        Value::String(text) => contains(text),
        Value::Number(number) => contains(&number.to_string()),
        Value::Array(values) => values.iter().any(|value| contains_secret(value, secrets)),
        Value::Object(fields) => fields
            .iter()
            .any(|(key, value)| contains(key) || contains_secret(value, secrets)),
        _ => false,
    }
}
fn text(value: &Value, field: &str, max: usize) -> Result<String> {
    let s = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    if s.is_empty() || s.len() > max || !s.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(invalid());
    }
    Ok(s.into())
}
fn id(value: &Value, field: &str) -> Result<String> {
    let id = text(value, field, 36)?;
    uuid::Uuid::parse_str(&id).map_err(|_| invalid())?;
    Ok(id)
}
fn flag(value: &Value, field: &str) -> Result<bool> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(invalid)
}

/// Exact decimal normalization; no binary float conversion, exponent, NaN or silent missing-to-zero.
pub fn decimal(value: &Value) -> Result<String> {
    let raw = value.as_str().ok_or_else(invalid)?;
    if raw.is_empty() || raw.len() > 128 {
        return Err(invalid());
    }
    let (negative, s) = raw.strip_prefix('-').map_or((false, raw), |s| (true, s));
    let (whole, fraction) = s.split_once('.').unwrap_or((s, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || (s.contains('.') && fraction.is_empty())
    {
        return Err(invalid());
    }
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let fraction = fraction.trim_end_matches('0');
    Ok(format!(
        "{}{}{}{}",
        if negative && (whole != "0" || !fraction.is_empty()) {
            "-"
        } else {
            ""
        },
        whole,
        if fraction.is_empty() { "" } else { "." },
        fraction
    ))
}
fn optional_decimal(value: &Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => decimal(v).map(Some),
    }
}
fn symbol(value: &Value) -> Result<String> {
    let s = text(value, "symbol", 64)?;
    if !s
        .bytes()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b"./_-".contains(&b))
    {
        return Err(invalid());
    }
    Ok(s)
}

fn alpaca(account: Value, positions: Value, orders: Value) -> Result<Observation> {
    let remote_account_id = id(&account, "id")?;
    let currency = text(&account, "currency", 3)?;
    if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(invalid());
    }
    let status = text(&account, "status", 32)?;
    if !matches!(
        status.as_str(),
        "ACTIVE"
            | "ONBOARDING"
            | "SUBMISSION_FAILED"
            | "SUBMITTED"
            | "ACCOUNT_UPDATED"
            | "APPROVAL_PENDING"
            | "REJECTED"
            | "DISABLED"
            | "ACTION_REQUIRED"
    ) {
        return Err(invalid());
    }
    let mut limitations=vec!["API-key permission scope cannot be fully inspected. Successful reads do not verify trading or transfer permissions.".into()];
    let account_blocked = flag(&account, "account_blocked")?;
    let trading_blocked = flag(&account, "trading_blocked")?;
    if status != "ACTIVE" || account_blocked || trading_blocked {
        limitations.push(
            "Alpaca reports an inactive or restricted account; execution is unavailable.".into(),
        );
    }
    if flag(&account, "shorting_enabled")? {
        limitations.push(
            "The account supports short selling. TradeX has not enabled short-sale execution."
                .into(),
        );
    }
    let positions = positions.as_array().ok_or_else(invalid)?;
    let orders = orders.as_array().ok_or_else(invalid)?;
    if orders.len() > 10_000 || positions.len() > 10_000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let positions = positions
        .iter()
        .map(|p| {
            Ok(Position {
                symbol: symbol(p)?,
                quantity: decimal(&p["qty"])?,
                market_value: optional_decimal(p, "market_value")?,
                average_entry_price: optional_decimal(p, "avg_entry_price")?,
                instrument_currency: Some(currency.clone()),
                market_value_currency: Some(currency.clone()),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut order_ids = std::collections::HashSet::new();
    let orders = orders
        .iter()
        .map(|o| {
            let broker_order_id = id(o, "id")?;
            if !order_ids.insert(broker_order_id.clone()) {
                return Err(invalid());
            }
            let side = text(o, "side", 4)?;
            if !matches!(side.as_str(), "buy" | "sell") {
                return Err(invalid());
            }
            let status = text(o, "status", 32)?;
            if !matches!(
                status.as_str(),
                "new"
                    | "partially_filled"
                    | "accepted"
                    | "pending_new"
                    | "accepted_for_bidding"
                    | "pending_cancel"
                    | "pending_replace"
                    | "held"
                    | "stopped"
                    | "suspended"
                    | "calculated"
            ) {
                return Err(invalid());
            }
            let quantity = optional_decimal(o, "qty")?;
            let notional = optional_decimal(o, "notional")?;
            if quantity.is_none() && notional.is_none() {
                return Err(invalid());
            }
            Ok(OpenOrder {
                broker_order_id,
                symbol: symbol(o)?,
                side,
                quantity,
                notional,
                filled_quantity: Some(decimal(&o["filled_qty"])?),
                filled_value: None,
                currency: Some(currency.clone()),
                status,
                limit_price: optional_decimal(o, "limit_price")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let permissions = PermissionReview {
        detected: vec![
            "account.read".into(),
            "positions.read".into(),
            "orders.read".into(),
        ],
        ..PermissionReview::default()
    };
    Ok(Observation {
        data: AccountData {
            remote_account_id,
            account_type: "ALPACA_PAPER".into(),
            currency: Some(currency.clone()),
            balances: vec![Balance {
                asset: currency,
                available: decimal(&account["cash"])?,
                total: optional_decimal(&account, "equity")?,
                reserved: None,
                in_pies: None,
            }],
            positions,
            open_orders: orders,
            capabilities: permissions.detected.clone(),
            limitations,
        },
        permissions,
    })
}
