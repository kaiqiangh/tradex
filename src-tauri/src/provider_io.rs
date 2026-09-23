use crate::{
    market,
    protocol::{
        AlpacaPaperCancelState, AlpacaPaperFill, AlpacaPaperFillSource, AlpacaPaperOrder,
        AlpacaPaperOrderAttempt, AlpacaPaperOrderAttemptState, AlpacaPaperOrderBook,
        AlpacaPaperOrderBookStatus, AlpacaPaperOrderOrigin, ExecutionContext, OrderProposal,
        OrderQuantityType, OrderSide, OrderType, Result, TimeInForce, TradeXError,
        Trading212DemoNormalizedOrderStatus, Trading212DemoOrder, Trading212DemoOrderAttempt,
        Trading212DemoOrderAttemptState, Trading212DemoOrderBook, Trading212DemoOrderBookStatus,
        Trading212DemoOrderOrigin,
    },
    providers::*,
};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::Read,
    sync::{Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

#[path = "binance.rs"]
mod binance;
#[path = "bitget.rs"]
mod bitget;
#[path = "trading212.rs"]
mod trading212;

const MAX_RESPONSE: u64 = 2 * 1024 * 1024;
const SERVICE: &str = "com.tradex.broker.credentials";
static TRADING212_READ_LIMITS: OnceLock<Mutex<HashMap<(String, &'static str), i64>>> =
    OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq)]
enum Trading212ReadEndpoint {
    PendingOrders,
    OrderDetail,
    History,
}

impl Trading212ReadEndpoint {
    fn key(self) -> &'static str {
        match self {
            Self::PendingOrders => "orders",
            Self::OrderDetail => "order-detail",
            Self::History => "history-orders",
        }
    }
    fn minimum_interval(self) -> i64 {
        match self {
            Self::PendingOrders => 5,
            Self::OrderDetail => 1,
            Self::History => 10,
        }
    }
    fn deadline(self, book: &Trading212DemoOrderBook) -> &Option<String> {
        match self {
            Self::PendingOrders => &book.rate_limits.pending_orders_retry_at,
            Self::OrderDetail => &book.rate_limits.order_detail_retry_at,
            Self::History => &book.rate_limits.history_retry_at,
        }
    }
    fn deadline_mut(self, book: &mut Trading212DemoOrderBook) -> &mut Option<String> {
        match self {
            Self::PendingOrders => &mut book.rate_limits.pending_orders_retry_at,
            Self::OrderDetail => &mut book.rate_limits.order_detail_retry_at,
            Self::History => &mut book.rate_limits.history_retry_at,
        }
    }
}

fn trading212_order_read_error(status: u16, endpoint: Trading212ReadEndpoint) -> TradeXError {
    match status {
        401 => TradeXError::new("PROVIDER_AUTH_FAILED"),
        403 => TradeXError::new("PROVIDER_PERMISSION_BLOCKED"),
        404 if endpoint == Trading212ReadEndpoint::OrderDetail => {
            TradeXError::new("ORDER_STATUS_UNKNOWN")
        }
        429 => TradeXError::new("PROVIDER_RATE_LIMITED"),
        408 | 500..=599 => TradeXError::new("PROVIDER_UNAVAILABLE"),
        400..=499 => TradeXError::new("PROVIDER_RESPONSE_INVALID"),
        _ => TradeXError::new("PROVIDER_DATA_INCOMPLETE"),
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn unix_timestamp(value: i64) -> Result<String> {
    time::OffsetDateTime::from_unix_timestamp(value)
        .map_err(|_| invalid())?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| invalid())
}

fn parsed_timestamp(value: &str) -> Option<i64> {
    time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .ok()
        .map(|value| value.unix_timestamp())
}

// ponytail: one process-wide map serializes tiny per-account reservations; per-account locks only if request volume requires it.
fn reserve_trading212_read(
    book: &mut Trading212DemoOrderBook,
    endpoint: Trading212ReadEndpoint,
) -> Result<String> {
    let now = unix_now();
    let persisted = endpoint
        .deadline(book)
        .as_deref()
        .and_then(parsed_timestamp)
        .unwrap_or_default();
    let limits = TRADING212_READ_LIMITS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut limits = limits.lock().unwrap_or_else(|error| error.into_inner());
    let key = (book.remote_account_id.clone(), endpoint.key());
    let previous = limits.get(&key).copied().unwrap_or_default();
    let allowed_at = persisted.max(previous);
    if now < allowed_at {
        let retry_at = unix_timestamp(allowed_at)?;
        *endpoint.deadline_mut(book) = Some(retry_at);
        return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
    }
    let next = now.saturating_add(endpoint.minimum_interval());
    limits.insert(key, next);
    let retry_at = unix_timestamp(next)?;
    *endpoint.deadline_mut(book) = Some(retry_at.clone());
    Ok(retry_at)
}

fn record_trading212_rate_limit(
    book: &mut Trading212DemoOrderBook,
    endpoint: Trading212ReadEndpoint,
    headers: Option<ProviderRateLimit>,
) -> Result<()> {
    let mut deadline = endpoint
        .deadline(book)
        .as_deref()
        .and_then(parsed_timestamp)
        .unwrap_or_default();
    if headers.as_ref().and_then(|headers| headers.remaining) == Some(0)
        && let Some(reset) = headers
            .and_then(|headers| headers.reset_at)
            .as_deref()
            .and_then(parsed_timestamp)
    {
        deadline = deadline.max(reset);
    }
    if deadline > 0 {
        *endpoint.deadline_mut(book) = Some(unix_timestamp(deadline)?);
        let limits = TRADING212_READ_LIMITS.get_or_init(|| Mutex::new(HashMap::new()));
        limits
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert((book.remote_account_id.clone(), endpoint.key()), deadline);
    }
    Ok(())
}

// No Debug/Serialize implementation: this value stays inside native capture, Keychain and signing.
pub struct Credentials(Zeroizing<Vec<u8>>);

impl Credentials {
    pub fn new(values: Vec<String>) -> Result<Self> {
        let values = Zeroizing::new(values);
        if !matches!(values.len(), 2 | 3)
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
    pub(crate) fn values(&self) -> Result<Zeroizing<Vec<String>>> {
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
    BitgetDemo,
    BitgetLive,
}
impl ProviderEndpoint {
    pub fn base_url(self) -> &'static str {
        match self {
            Self::AlpacaPaper => "https://paper-api.alpaca.markets",
            Self::Trading212Demo => "https://demo.trading212.com",
            Self::Trading212Live => "https://live.trading212.com",
            Self::BinanceTestnet => "https://testnet.binance.vision",
            Self::BinanceLive => "https://api.binance.com",
            Self::BitgetDemo | Self::BitgetLive => "https://api.bitget.com",
        }
    }
    fn is_binance(self) -> bool {
        matches!(self, Self::BinanceTestnet | Self::BinanceLive)
    }
    fn is_bitget(self) -> bool {
        matches!(self, Self::BitgetDemo | Self::BitgetLive)
    }
    fn allows(self, path: &str) -> bool {
        match self {
            Self::AlpacaPaper => allowed_path(path),
            Self::BitgetDemo | Self::BitgetLive => bitget::allows(path),
            Self::BinanceTestnet | Self::BinanceLive => binance::allows(self, path),
            Self::Trading212Demo => {
                matches!(
                    path,
                    "/api/v0/equity/account/summary"
                        | "/api/v0/equity/positions"
                        | "/api/v0/equity/orders"
                ) || valid_t212_order_detail_path(path)
                    || valid_t212_history_path(path)
            }
            Self::Trading212Live => matches!(
                path,
                "/api/v0/equity/account/summary"
                    | "/api/v0/equity/positions"
                    | "/api/v0/equity/orders"
            ),
        }
    }
    fn allows_method(self, method: ProviderHttpMethod, path: &str) -> bool {
        match method {
            ProviderHttpMethod::Get => self.allows(path),
            ProviderHttpMethod::Post => {
                (self == Self::AlpacaPaper && path == "/v2/orders")
                    || (self == Self::Trading212Demo
                        && matches!(
                            path,
                            "/api/v0/equity/orders/market" | "/api/v0/equity/orders/limit"
                        ))
            }
            ProviderHttpMethod::Delete => {
                self == Self::AlpacaPaper
                    && path
                        .strip_prefix("/v2/orders/")
                        .is_some_and(valid_provider_order_id)
            }
        }
    }
}

fn valid_t212_order_detail_path(path: &str) -> bool {
    path.strip_prefix("/api/v0/equity/orders/")
        .is_some_and(valid_t212_order_id)
}

pub(crate) fn valid_t212_order_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<i64>().is_ok_and(|id| id > 0)
}

pub(crate) fn valid_t212_history_path(path: &str) -> bool {
    let Some((route, query)) = path.split_once('?') else {
        return false;
    };
    if route != "/api/v0/equity/history/orders" || query.is_empty() || path.len() > 512 {
        return false;
    }
    let mut limit = false;
    let mut cursor = false;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            return false;
        };
        match key {
            "limit" if !limit && value == "50" => limit = true,
            "cursor"
                if !cursor
                    && !value.is_empty()
                    && value.len() <= 19
                    && value.bytes().all(|byte| byte.is_ascii_digit())
                    && value.parse::<i64>().is_ok() =>
            {
                cursor = true
            }
            _ => return false,
        }
    }
    limit
}

fn t212_history_cursor(path: &str) -> Option<String> {
    let (_, query) = path.split_once('?')?;
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == "cursor").then(|| value.to_owned())
    })
}

fn rate_limit(headers: &HeaderMap) -> Option<ProviderRateLimit> {
    let remaining = headers
        .get("x-ratelimit-remaining")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());
    let reset_at = headers
        .get("x-ratelimit-reset")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
        .and_then(|value| time::OffsetDateTime::from_unix_timestamp(value).ok())
        .and_then(|value| {
            value
                .format(&time::format_description::well_known::Rfc3339)
                .ok()
        });
    (remaining.is_some() || reset_at.is_some()).then_some(ProviderRateLimit {
        remaining,
        reset_at,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderHttpMethod {
    Get,
    Post,
    Delete,
}

pub struct ProviderHttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct ProviderRateLimit {
    pub remaining: Option<u64>,
    pub reset_at: Option<String>,
}

pub trait ProviderHttp {
    fn get(&self, endpoint: ProviderEndpoint, path: &str, headers: HeaderMap) -> Result<Vec<u8>>;

    fn request(
        &self,
        endpoint: ProviderEndpoint,
        method: ProviderHttpMethod,
        path: &str,
        headers: HeaderMap,
        body: Option<&Value>,
    ) -> Result<ProviderHttpResponse> {
        if method != ProviderHttpMethod::Get || body.is_some() {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        self.get(endpoint, path, headers)
            .map(|body| ProviderHttpResponse { status: 200, body })
    }

    fn request_with_rate_limit(
        &self,
        endpoint: ProviderEndpoint,
        method: ProviderHttpMethod,
        path: &str,
        headers: HeaderMap,
        body: Option<&Value>,
    ) -> Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
        self.request(endpoint, method, path, headers, body)
            .map(|response| (response, None))
    }
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
            400 if endpoint.is_binance() || endpoint.is_bitget() => (),
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
        if status != 200 && endpoint.is_bitget() {
            let value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
            return Err(bitget::business(value).err().unwrap_or_else(invalid));
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

    fn request(
        &self,
        endpoint: ProviderEndpoint,
        method: ProviderHttpMethod,
        path: &str,
        headers: HeaderMap,
        body: Option<&Value>,
    ) -> Result<ProviderHttpResponse> {
        self.request_with_rate_limit(endpoint, method, path, headers, body)
            .map(|(response, _)| response)
    }

    fn request_with_rate_limit(
        &self,
        endpoint: ProviderEndpoint,
        method: ProviderHttpMethod,
        path: &str,
        headers: HeaderMap,
        body: Option<&Value>,
    ) -> Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
        if !endpoint.allows_method(method, path) {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let request = match method {
            ProviderHttpMethod::Get if body.is_none() => self
                .client()?
                .get(format!("{}{path}", endpoint.base_url()))
                .headers(headers),
            ProviderHttpMethod::Post if body.is_some() => {
                let body = serde_json::to_vec(body.unwrap()).map_err(|_| invalid())?;
                if body.len() > 16 * 1024 {
                    return Err(invalid());
                }
                self.client()?
                    .post(format!("{}{path}", endpoint.base_url()))
                    .headers(headers)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body)
            }
            ProviderHttpMethod::Delete if body.is_none() => self
                .client()?
                .delete(format!("{}{path}", endpoint.base_url()))
                .headers(headers),
            _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
        };
        let response = request
            .send()
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE)
        {
            return Err(invalid());
        }
        let status = response.status().as_u16();
        let rate_limit = rate_limit(response.headers());
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err(invalid());
        }
        Ok((
            ProviderHttpResponse {
                status,
                body: bytes,
            },
            rate_limit,
        ))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum JobKind {
    Connect,
    Probe,
    Disconnect,
    Trading212DemoSubmit,
    Trading212DemoOrderBookPending,
    Trading212DemoOrderBookHistory,
    Trading212DemoOrderBookDetail,
    AlpacaPaperSubmit,
    AlpacaPaperReconcile,
    AlpacaPaperOrderBookRefresh,
    AlpacaPaperOrderReview,
    AlpacaPaperOrderCancel,
}

pub struct ProviderJob {
    pub(crate) account: AccountConnection,
    pub(crate) kind: JobKind,
    pub(crate) session: String,
    pub(crate) request_id: String,
    pub(crate) trading212_demo_attempt: Option<Trading212DemoOrderAttempt>,
    pub(crate) trading212_demo_proposal: Option<OrderProposal>,
    pub(crate) trading212_demo_order_book: Option<Trading212DemoOrderBook>,
    pub(crate) trading212_demo_order_id: Option<String>,
    pub(crate) alpaca_attempt: Option<AlpacaPaperOrderAttempt>,
    pub(crate) alpaca_proposal: Option<OrderProposal>,
    pub(crate) alpaca_order_book: Option<AlpacaPaperOrderBook>,
    pub(crate) alpaca_order_id: Option<String>,
    pub(crate) alpaca_expected_order: Option<AlpacaPaperOrder>,
}

pub(crate) struct Observation {
    pub data: AccountData,
    pub permissions: PermissionReview,
}

fn normalize_account_data(data: &mut AccountData, provider_id: &str) {
    for position in &mut data.positions {
        position.instrument_id = market::canonical_instrument_id(provider_id, &position.symbol);
    }
    for order in &mut data.open_orders {
        order.instrument_id = market::canonical_instrument_id(provider_id, &order.symbol);
    }
}
pub struct ProviderOutcome {
    pub(crate) observation: Option<Observation>,
    pub(crate) error: Option<TradeXError>,
    pub(crate) credential: String,
    pub(crate) trading212_demo_attempt: Option<Trading212DemoOrderAttempt>,
    pub(crate) trading212_demo_order_book: Option<Trading212DemoOrderBook>,
    pub(crate) alpaca_paper_attempt: Option<AlpacaPaperOrderAttempt>,
    pub(crate) alpaca_paper_order_book: Option<AlpacaPaperOrderBook>,
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
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
            };
        }
        if matches!(
            self.kind,
            JobKind::AlpacaPaperSubmit | JobKind::AlpacaPaperReconcile
        ) {
            return self.run_alpaca_paper_order(vault, http, current);
        }
        if self.kind == JobKind::Trading212DemoSubmit {
            return self.run_trading212_demo_order(vault, http, current);
        }
        if matches!(
            self.kind,
            JobKind::Trading212DemoOrderBookPending
                | JobKind::Trading212DemoOrderBookHistory
                | JobKind::Trading212DemoOrderBookDetail
        ) {
            return self.run_trading212_demo_order_book(vault, http, current);
        }
        if matches!(
            self.kind,
            JobKind::AlpacaPaperOrderBookRefresh
                | JobKind::AlpacaPaperOrderReview
                | JobKind::AlpacaPaperOrderCancel
        ) {
            return self.run_alpaca_paper_order_book(vault, http, current);
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
                ("bitget", "DEMO") => ProviderEndpoint::BitgetDemo,
                ("bitget", "LIVE") => ProviderEndpoint::BitgetLive,
                _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            };
            if self.kind == JobKind::Connect {
                let secret = capture(&definition)?;
                if secret.values()?.len() != definition.fields.len() {
                    return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
                }
                if !current() {
                    return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                }
                vault.put(&reference, &secret)?;
            }
            let secret = vault.get(&reference)?;
            credential = "CONFIGURED";
            let mut values = secret.values()?;
            if values.len() != definition.fields.len() {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            if endpoint.is_bitget() {
                let mut observation = bitget::read(
                    endpoint,
                    &values,
                    http,
                    &current,
                    self.account.data.as_ref(),
                )?;
                normalize_account_data(&mut observation.data, &self.account.provider_id);
                return Ok(observation);
            }
            if endpoint.is_binance() {
                let mut observation = binance::read(
                    endpoint,
                    &values,
                    http,
                    &current,
                    self.account.data.as_ref(),
                )?;
                normalize_account_data(&mut observation.data, &self.account.provider_id);
                return Ok(observation);
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
            let mut observation = if endpoint == ProviderEndpoint::AlpacaPaper {
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
            normalize_account_data(&mut observation.data, &self.account.provider_id);
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
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
            },
            Err(error) => ProviderOutcome {
                observation: None,
                error: Some(error),
                credential: credential.into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
            },
        }
    }

    fn run_trading212_demo_order(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let Some(mut attempt) = self.trading212_demo_attempt.clone() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
            };
        };
        let mut credential_state = "MISSING";
        let outcome = (|| -> Result<Trading212DemoOrderAttempt> {
            if self.account.provider_id != "trading212"
                || self.account.environment != "DEMO"
                || self.kind != JobKind::Trading212DemoSubmit
            {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            let proposal = self
                .trading212_demo_proposal
                .as_ref()
                .ok_or_else(|| TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"))?;
            if proposal.workspace_id != attempt.workspace_id
                || proposal.proposal_id != attempt.proposal_id
                || proposal.proposal_hash != attempt.proposal_hash
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let (path, body) = trading212_demo_order_request(proposal, &attempt.connection_id)?;
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            credential_state = "CONFIGURED";
            let mut values = secret.values()?;
            if values.len() != 2 || values[0].contains(':') {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            use base64::Engine;
            let combined = Zeroizing::new(format!("{}:{}", values[0], values[1]));
            let encoded = Zeroizing::new(
                base64::engine::general_purpose::STANDARD.encode(combined.as_bytes()),
            );
            let header_text = Zeroizing::new(format!("Basic {}", encoded.as_str()));
            let mut header = HeaderValue::from_str(&header_text)
                .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
            header.set_sensitive(true);
            let mut auth = HeaderMap::new();
            auth.insert("Authorization", header);
            values.push(encoded.to_string());

            let preflight = http.request(
                ProviderEndpoint::Trading212Demo,
                ProviderHttpMethod::Get,
                "/api/v0/equity/account/summary",
                auth.clone(),
                None,
            )?;
            if preflight.status != 200 {
                return Err(match preflight.status {
                    401 => TradeXError::new("PROVIDER_AUTH_FAILED"),
                    403 => TradeXError::new("PROVIDER_PERMISSION_BLOCKED"),
                    429 => TradeXError::new("PROVIDER_RATE_LIMITED"),
                    _ => TradeXError::new("PROVIDER_UNAVAILABLE"),
                });
            }
            let account: Value = serde_json::from_slice(&preflight.body).map_err(|_| invalid())?;
            if contains_secret(&account, &values)
                || trading212::account_id(&account)? != attempt.remote_account_id
            {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
            }
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            match http.request(
                ProviderEndpoint::Trading212Demo,
                ProviderHttpMethod::Post,
                &path,
                auth,
                Some(&body),
            ) {
                Ok(response) if response.status == 200 => {
                    match trading212_demo_acknowledgement(
                        &response.body,
                        &values,
                        &attempt,
                        proposal,
                        &body,
                    ) {
                        Ok(acknowledged) => Ok(acknowledged),
                        Err(_) => Ok(trading212_unknown_attempt(attempt.clone())),
                    }
                }
                Ok(response) if matches!(response.status, 400 | 401 | 403 | 429) => {
                    Err(match response.status {
                        401 => TradeXError::new("PROVIDER_AUTH_FAILED"),
                        403 => TradeXError::new("PROVIDER_PERMISSION_BLOCKED"),
                        429 => TradeXError::new("PROVIDER_RATE_LIMITED"),
                        _ => TradeXError::new("PROVIDER_ORDER_REJECTED"),
                    })
                }
                Ok(_) | Err(_) => Ok(trading212_unknown_attempt(attempt.clone())),
            }
        })();
        match outcome {
            Ok(updated) => attempt = updated,
            Err(error) => {
                attempt.state = Trading212DemoOrderAttemptState::Rejected;
                attempt.provider_order_id = None;
                attempt.provider_status = None;
                attempt.error_code = Some(error.code.clone());
                attempt.reason = error.message;
            }
        }
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential_state.into(),
            trading212_demo_attempt: Some(attempt),
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
        }
    }

    fn run_trading212_demo_order_book(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let Some(mut book) = self.trading212_demo_order_book.clone() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_STATUS_UNKNOWN")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
            };
        };
        let mut credential_state = "MISSING";
        let outcome = (|| -> Result<()> {
            if self.account.provider_id != "trading212"
                || self.account.environment != "DEMO"
                || book.environment != "DEMO"
                || book.connection_id != self.account.connection_id
                || book.workspace_id != self.account.workspace_id
                || self
                    .account
                    .data
                    .as_ref()
                    .map(|data| data.remote_account_id.as_str())
                    != Some(book.remote_account_id.as_str())
                || !matches!(
                    self.kind,
                    JobKind::Trading212DemoOrderBookPending
                        | JobKind::Trading212DemoOrderBookHistory
                        | JobKind::Trading212DemoOrderBookDetail
                )
            {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            credential_state = "CONFIGURED";
            let mut values = secret.values()?;
            if values.len() != 2 || values[0].contains(':') {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            use base64::Engine;
            let combined = Zeroizing::new(format!("{}:{}", values[0], values[1]));
            let encoded = Zeroizing::new(
                base64::engine::general_purpose::STANDARD.encode(combined.as_bytes()),
            );
            let header_text = Zeroizing::new(format!("Basic {}", encoded.as_str()));
            let mut header = HeaderValue::from_str(&header_text)
                .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
            header.set_sensitive(true);
            let mut auth = HeaderMap::new();
            auth.insert("Authorization", header);
            values.push(encoded.to_string());

            let now = crate::storage::timestamp()?;
            let mut candidate = book.clone();
            let endpoint = match self.kind {
                JobKind::Trading212DemoOrderBookPending => Trading212ReadEndpoint::PendingOrders,
                JobKind::Trading212DemoOrderBookHistory => Trading212ReadEndpoint::History,
                JobKind::Trading212DemoOrderBookDetail => Trading212ReadEndpoint::OrderDetail,
                _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            };
            let path = match self.kind {
                JobKind::Trading212DemoOrderBookPending => "/api/v0/equity/orders".to_owned(),
                JobKind::Trading212DemoOrderBookDetail => {
                    let order_id = self
                        .trading212_demo_order_id
                        .as_deref()
                        .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
                    if !valid_t212_order_id(order_id)
                        || !book
                            .orders
                            .iter()
                            .any(|order| order.provider_order_id == order_id && order.pending)
                    {
                        return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
                    }
                    format!("/api/v0/equity/orders/{order_id}")
                }
                JobKind::Trading212DemoOrderBookHistory => {
                    if book.history_page_count >= 100 && !book.history_complete {
                        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                    }
                    if !book.history_started || book.history_complete {
                        candidate.history_started = false;
                        candidate.history_complete = false;
                        candidate.history_page_count = 0;
                        candidate.next_page_path = None;
                        candidate.history_cursors.clear();
                        "/api/v0/equity/history/orders?limit=50".to_owned()
                    } else {
                        book.next_page_path
                            .clone()
                            .filter(|path| valid_t212_history_path(path))
                            .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?
                    }
                }
                _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            };
            let cursor_token = if self.kind == JobKind::Trading212DemoOrderBookHistory {
                t212_history_cursor(&path).unwrap_or_else(|| "FIRST".into())
            } else {
                String::new()
            };
            if self.kind == JobKind::Trading212DemoOrderBookHistory
                && candidate.history_cursors.contains(&cursor_token)
            {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            reserve_trading212_read(&mut book, endpoint)?;
            let (response, limit_headers) = http.request_with_rate_limit(
                ProviderEndpoint::Trading212Demo,
                ProviderHttpMethod::Get,
                &path,
                auth,
                None,
            )?;
            record_trading212_rate_limit(&mut book, endpoint, limit_headers)?;
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            if response.status != 200 {
                return Err(trading212_order_read_error(response.status, endpoint));
            }
            let value: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
            if contains_secret(&value, &values) {
                return Err(invalid());
            }
            match self.kind {
                JobKind::Trading212DemoOrderBookPending => {
                    let rows = value.as_array().ok_or_else(invalid)?;
                    if rows.len() > 500 {
                        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                    }
                    let mut seen = std::collections::HashSet::new();
                    let mut parsed = Vec::with_capacity(rows.len());
                    for row in rows {
                        let order = parse_trading212_order(row, &now, true)?;
                        if !seen.insert(order.provider_order_id.clone()) {
                            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
                        }
                        parsed.push(order);
                    }
                    candidate.orders = merge_trading212_orders(&book.orders, parsed, true, false)?;
                    candidate.observed_at = now.clone();
                    candidate.last_successful_sync_at = Some(now.clone());
                }
                JobKind::Trading212DemoOrderBookDetail => {
                    let order_id = self.trading212_demo_order_id.as_deref().unwrap();
                    let mut order = parse_trading212_order(&value, &now, true)?;
                    if order.provider_order_id != order_id {
                        return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
                    }
                    order.pending = !matches!(
                        order.normalized_status,
                        Trading212DemoNormalizedOrderStatus::Cancelled
                            | Trading212DemoNormalizedOrderStatus::Filled
                            | Trading212DemoNormalizedOrderStatus::Rejected
                            | Trading212DemoNormalizedOrderStatus::Replaced
                    );
                    candidate.orders =
                        merge_trading212_orders(&book.orders, vec![order], false, true)?;
                    candidate.observed_at = now.clone();
                    candidate.last_successful_sync_at = Some(now.clone());
                }
                JobKind::Trading212DemoOrderBookHistory => {
                    let rows = value["items"].as_array().ok_or_else(invalid)?;
                    if rows.len() > 50 {
                        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                    }
                    let mut seen = std::collections::HashSet::new();
                    let mut parsed = Vec::with_capacity(rows.len());
                    for row in rows {
                        let order = parse_trading212_order(row, &now, false)?;
                        if !seen.insert(order.provider_order_id.clone()) {
                            return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
                        }
                        parsed.push(order);
                    }
                    let next = match value.get("nextPagePath") {
                        Some(Value::Null) => None,
                        Some(Value::String(path)) if valid_t212_history_path(path) => {
                            Some(path.clone())
                        }
                        _ => return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE")),
                    };
                    if next.as_deref().is_some_and(|next| {
                        t212_history_cursor(next).is_none_or(|cursor| {
                            cursor == cursor_token || candidate.history_cursors.contains(&cursor)
                        })
                    }) {
                        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                    }
                    candidate.orders = merge_trading212_orders(&book.orders, parsed, false, false)?;
                    candidate.history_cursors.push(cursor_token);
                    candidate.history_page_count = candidate
                        .history_page_count
                        .checked_add(1)
                        .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
                    candidate.history_started = true;
                    candidate.history_complete = next.is_none();
                    candidate.next_page_path = next;
                    candidate.observed_at = now.clone();
                    candidate.last_successful_sync_at = Some(now.clone());
                }
                _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            }
            candidate.status = Trading212DemoOrderBookStatus::Current;
            candidate.reason = None;
            candidate.rate_limits = book.rate_limits.clone();
            book = candidate;
            Ok(())
        })();
        if let Err(error) = outcome {
            book.status = Trading212DemoOrderBookStatus::Degraded;
            book.reason = Some(error.code.clone());
            book.observed_at = crate::storage::timestamp().unwrap_or(book.observed_at.clone());
        }
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential_state.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: Some(book),
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
        }
    }

    fn run_alpaca_paper_order(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let Some(mut attempt) = self.alpaca_attempt.clone() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
            };
        };
        let mut credential_state = "MISSING";
        let outcome = (|| -> Result<AlpacaPaperOrderAttempt> {
            if self.account.provider_id != "alpaca" || self.account.environment != "PAPER" {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            if !matches!(
                self.kind,
                JobKind::AlpacaPaperSubmit | JobKind::AlpacaPaperReconcile
            ) {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            credential_state = "CONFIGURED";
            let values = secret.values()?;
            if values.len() != 2 {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            let auth = alpaca_headers(&values)?;
            let endpoint = ProviderEndpoint::AlpacaPaper;
            let proposal = self
                .alpaca_proposal
                .as_ref()
                .ok_or_else(|| TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"))?;
            if proposal.proposal_id != attempt.proposal_id
                || proposal.proposal_hash != attempt.proposal_hash
                || proposal.workspace_id != attempt.workspace_id
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            if self.kind == JobKind::AlpacaPaperReconcile {
                return Ok(reconcile_alpaca_attempt(
                    http,
                    &auth,
                    attempt.clone(),
                    proposal,
                ));
            }
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let (symbol, body, needs_fractional) = alpaca_order_request(proposal, &attempt)?;
            let account_response = http.request(
                endpoint,
                ProviderHttpMethod::Get,
                "/v2/account",
                auth.clone(),
                None,
            )?;
            if account_response.status != 200 {
                return Err(alpaca_read_error(account_response.status));
            }
            let account: Value =
                serde_json::from_slice(&account_response.body).map_err(|_| invalid())?;
            if id(&account, "id")? != attempt.remote_account_id
                || text(&account, "status", 32)? != "ACTIVE"
                || flag(&account, "account_blocked")?
                || flag(&account, "trading_blocked")?
                || flag(&account, "trade_suspended_by_user")?
            {
                return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
            }
            let currency = text(&account, "currency", 3)?;
            if currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_uppercase()) {
                return Err(invalid());
            }
            if let Some(notional) = body.get("notional").and_then(Value::as_str) {
                if currency != "USD" {
                    return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED"));
                }
                let buying_power = decimal(&account["buying_power"])?;
                if decimal_cmp(notional, &buying_power)? == std::cmp::Ordering::Greater {
                    return Err(TradeXError::new("ORDER_BUYING_POWER_INSUFFICIENT"));
                }
            }
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let asset_path = format!("/v2/assets/{symbol}");
            let asset_response = http.request(
                endpoint,
                ProviderHttpMethod::Get,
                &asset_path,
                auth.clone(),
                None,
            )?;
            if asset_response.status != 200 {
                return Err(if asset_response.status == 404 {
                    TradeXError::new("ORDER_ASSET_UNAVAILABLE")
                } else {
                    alpaca_read_error(asset_response.status)
                });
            }
            let asset: Value =
                serde_json::from_slice(&asset_response.body).map_err(|_| invalid())?;
            let asset_id = validate_alpaca_asset(&asset, &symbol, needs_fractional)?;
            if body.get("side").and_then(Value::as_str) == Some("sell") {
                let quantity = body
                    .get("qty")
                    .and_then(Value::as_str)
                    .ok_or_else(|| TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED"))?;
                let position_response = http.request(
                    endpoint,
                    ProviderHttpMethod::Get,
                    &format!("/v2/positions/{symbol}"),
                    auth.clone(),
                    None,
                )?;
                if position_response.status == 404 {
                    return Err(TradeXError::new("ORDER_INSUFFICIENT_POSITION"));
                }
                if position_response.status != 200 {
                    return Err(alpaca_read_error(position_response.status));
                }
                let position: Value =
                    serde_json::from_slice(&position_response.body).map_err(|_| invalid())?;
                validate_alpaca_sell_position(&position, &symbol, &asset_id, quantity)?;
            }
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            match http.request(
                endpoint,
                ProviderHttpMethod::Post,
                "/v2/orders",
                auth.clone(),
                Some(&body),
            ) {
                Ok(response) if matches!(response.status, 200 | 201) => {
                    match alpaca_acknowledgement(
                        &response.body,
                        &body,
                        &symbol,
                        Some(&asset_id),
                        &attempt,
                    ) {
                        Ok(ack) => Ok(ack),
                        Err(_) => Ok(reconcile_alpaca_attempt(
                            http,
                            &auth,
                            attempt.clone(),
                            proposal,
                        )),
                    }
                }
                Ok(response) if matches!(response.status, 400 | 401 | 403 | 422 | 429) => {
                    Err(alpaca_order_rejection(
                        response.status,
                        &response.body,
                        body.get("side").and_then(Value::as_str).unwrap_or_default(),
                    ))
                }
                Ok(_) | Err(_) => Ok(reconcile_alpaca_attempt(
                    http,
                    &auth,
                    attempt.clone(),
                    proposal,
                )),
            }
        })();
        match outcome {
            Ok(updated) => attempt = updated,
            Err(error) => {
                attempt.state = AlpacaPaperOrderAttemptState::Rejected;
                attempt.error_code = Some(error.code.clone());
                attempt.reason = error.message;
            }
        }
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential_state.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: Some(attempt),
            alpaca_paper_order_book: None,
        }
    }

    fn run_alpaca_paper_order_book(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let Some(mut book) = self.alpaca_order_book.clone() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_STATUS_UNKNOWN")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
            };
        };
        let mut credential_state = "MISSING";
        let mut cancel_may_have_been_sent = false;
        let outcome = (|| -> Result<()> {
            if self.account.provider_id != "alpaca" || self.account.environment != "PAPER" {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            credential_state = "CONFIGURED";
            let values = secret.values()?;
            if values.len() != 2 {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            let auth = alpaca_headers(&values)?;
            verify_alpaca_paper_account(http, &auth, &book.remote_account_id, &values)?;
            match self.kind {
                JobKind::AlpacaPaperOrderBookRefresh => {
                    let (orders, fills) =
                        fetch_alpaca_paper_order_history(http, &auth, &current, &values)?;
                    let orders = merge_order_pages(&book.orders, orders)?;
                    let fills = merge_fill_pages(&book.fills, fills)?;
                    book.orders = orders;
                    book.fills = fills;
                    book.status = AlpacaPaperOrderBookStatus::Current;
                    book.reason = None;
                    let now = crate::storage::timestamp()?;
                    book.observed_at = now.clone();
                    book.last_successful_sync_at = Some(now);
                }
                JobKind::AlpacaPaperOrderReview => {
                    let order_id = self
                        .alpaca_order_id
                        .as_deref()
                        .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
                    let fresh = get_alpaca_order(http, &auth, order_id, &values)?;
                    let Some(previous) = book
                        .orders
                        .iter()
                        .find(|order| order.provider_order_id == order_id)
                        .cloned()
                    else {
                        return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
                    };
                    if !same_order_identity(&previous, &fresh) {
                        return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
                    }
                    replace_order(
                        &mut book.orders,
                        preserve_local_order_state(fresh, &previous),
                    )?;
                    book.observed_at = crate::storage::timestamp()?;
                    book.status = AlpacaPaperOrderBookStatus::Stale;
                    book.reason = Some("FULL_ORDER_BOOK_REFRESH_REQUIRED".into());
                }
                JobKind::AlpacaPaperOrderCancel => {
                    self.cancel_alpaca_order(
                        http,
                        &auth,
                        &values,
                        &mut book,
                        &current,
                        &mut cancel_may_have_been_sent,
                    )?;
                }
                _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            }
            if contains_secret(
                &serde_json::to_value(&book).map_err(|_| invalid())?,
                &values,
            ) {
                return Err(invalid());
            }
            Ok(())
        })();
        if let Err(error) = outcome {
            book.status = AlpacaPaperOrderBookStatus::Degraded;
            book.reason = Some(error.code.clone());
            book.observed_at =
                crate::storage::timestamp().unwrap_or_else(|_| book.observed_at.clone());
            if self.kind == JobKind::AlpacaPaperOrderCancel
                && let Some(order_id) = self.alpaca_order_id.as_deref()
                && let Some(order) = book
                    .orders
                    .iter_mut()
                    .find(|order| order.provider_order_id == order_id)
                && order.cancel_state == AlpacaPaperCancelState::Submitting
            {
                if cancel_may_have_been_sent {
                    // DELETE may have reached Alpaca before a transport failure. Reconcile only; never resend.
                    order.cancel_state = AlpacaPaperCancelState::Pending;
                    order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                } else {
                    order.cancel_state = AlpacaPaperCancelState::None;
                    order.cancel_idempotency_key = None;
                    order.cancel_error = Some(error.code.clone());
                }
            }
        }
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential_state.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: Some(book),
        }
    }

    fn cancel_alpaca_order(
        &self,
        http: &impl ProviderHttp,
        auth: &HeaderMap,
        secrets: &[String],
        book: &mut AlpacaPaperOrderBook,
        current: &impl Fn() -> bool,
        cancel_may_have_been_sent: &mut bool,
    ) -> Result<()> {
        let order_id = self
            .alpaca_order_id
            .as_deref()
            .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
        let reviewed = self
            .alpaca_expected_order
            .as_ref()
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let index = book
            .orders
            .iter()
            .position(|order| order.provider_order_id == order_id)
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let fresh = get_alpaca_order(http, auth, order_id, secrets)?;
        if !same_order_review(reviewed, &fresh) {
            let mut fresh = preserve_local_order_state(fresh, reviewed);
            fresh.cancel_state = AlpacaPaperCancelState::None;
            fresh.cancel_idempotency_key = None;
            fresh.cancel_error = Some("ORDER_CHANGED_REVIEW_AGAIN".into());
            book.orders[index] = fresh;
            book.observed_at = crate::storage::timestamp()?;
            book.reason = Some("ORDER_CHANGED_REVIEW_AGAIN".into());
            return Ok(());
        }
        let fresh = preserve_local_order_state(fresh, reviewed);
        if !cancelable_status(&fresh.provider_status) {
            let mut fresh = fresh;
            fresh.cancel_state = AlpacaPaperCancelState::None;
            fresh.cancel_idempotency_key = None;
            fresh.cancel_error = Some("ORDER_NOT_CANCELABLE".into());
            book.orders[index] = fresh;
            book.observed_at = crate::storage::timestamp()?;
            book.reason = Some("ORDER_NOT_CANCELABLE".into());
            return Ok(());
        }
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        *cancel_may_have_been_sent = true;
        let deleted = http.request(
            ProviderEndpoint::AlpacaPaper,
            ProviderHttpMethod::Delete,
            &format!("/v2/orders/{order_id}"),
            auth.clone(),
            None,
        );
        let status = match deleted {
            Ok(response) if response.status == 204 => {
                *cancel_may_have_been_sent = false;
                204
            }
            Ok(response) if response.status == 422 => {
                *cancel_may_have_been_sent = false;
                422
            }
            Ok(response) if matches!(response.status, 400 | 401 | 403 | 404 | 429) => {
                *cancel_may_have_been_sent = false;
                response.status
            }
            Ok(_) | Err(_) => {
                book.orders[index].cancel_state = AlpacaPaperCancelState::Pending;
                book.orders[index].cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                book.observed_at = crate::storage::timestamp()?;
                book.reason = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                return Ok(());
            }
        };
        if status == 204 {
            book.orders[index].cancel_state = AlpacaPaperCancelState::Pending;
            book.orders[index].cancel_error = None;
        } else {
            book.orders[index].cancel_state = AlpacaPaperCancelState::None;
            book.orders[index].cancel_idempotency_key = None;
            book.orders[index].cancel_error = Some(if status == 422 {
                "PROVIDER_CANCEL_REJECTED".into()
            } else {
                alpaca_read_error(status).code
            });
        }
        let refreshed = get_alpaca_order(http, auth, order_id, secrets);
        match refreshed {
            Ok(refreshed) => {
                let previous = book.orders[index].clone();
                if !same_order_identity(&previous, &refreshed) {
                    return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
                }
                let mut refreshed = preserve_local_order_state(refreshed, &previous);
                if is_terminal_order_status(&refreshed.provider_status) {
                    refreshed.cancel_state = AlpacaPaperCancelState::None;
                    refreshed.cancel_idempotency_key = None;
                    refreshed.cancel_error =
                        (status != 204).then(|| "PROVIDER_CANCEL_REJECTED".into());
                } else if status == 204 {
                    // Acknowledgement only; REST has not confirmed a terminal state.
                    refreshed.cancel_state = AlpacaPaperCancelState::Pending;
                }
                book.orders[index] = refreshed;
            }
            Err(error) => {
                book.status = AlpacaPaperOrderBookStatus::Degraded;
                book.reason = Some(if status == 204 {
                    "ORDER_CANCEL_STATUS_UNKNOWN".into()
                } else {
                    error.code
                });
            }
        }
        let fresh_fills = fetch_alpaca_paper_fills(http, auth, current, secrets)?;
        book.fills = merge_fill_pages(&book.fills, fresh_fills)?;
        book.observed_at = crate::storage::timestamp()?;
        if book.status != AlpacaPaperOrderBookStatus::Degraded {
            book.status = AlpacaPaperOrderBookStatus::Stale;
            book.reason = Some("FULL_ORDER_BOOK_REFRESH_REQUIRED".into());
        }
        Ok(())
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

const ALPACA_ORDER_PAGE_SIZE: usize = 100;
const ALPACA_ORDER_MAX_ORDERS: usize = 500;
const ALPACA_ORDER_MAX_PAGES: usize = ALPACA_ORDER_MAX_ORDERS / ALPACA_ORDER_PAGE_SIZE + 1;
const ALPACA_FILL_MAX: usize = 1000;
const ALPACA_FILL_MAX_PAGES: usize = ALPACA_FILL_MAX / ALPACA_ORDER_PAGE_SIZE + 1;

pub(crate) fn verify_alpaca_paper_account(
    http: &impl ProviderHttp,
    auth: &HeaderMap,
    remote_account_id: &str,
    secrets: &[String],
) -> Result<()> {
    let response = http.request(
        ProviderEndpoint::AlpacaPaper,
        ProviderHttpMethod::Get,
        "/v2/account",
        auth.clone(),
        None,
    )?;
    if response.status != 200 {
        return Err(alpaca_read_error(response.status));
    }
    let account: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
    if contains_secret(&account, secrets) {
        return Err(invalid());
    }
    if id(&account, "id")? != remote_account_id {
        return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
    }
    Ok(())
}

fn fetch_alpaca_paper_order_history(
    http: &impl ProviderHttp,
    auth: &HeaderMap,
    current: &impl Fn() -> bool,
    secrets: &[String],
) -> Result<(Vec<AlpacaPaperOrder>, Vec<AlpacaPaperFill>)> {
    let mut orders = Vec::new();
    let mut before = None;
    let mut cursors = std::collections::HashSet::new();
    for page in 0..ALPACA_ORDER_MAX_PAGES {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let path = before.as_ref().map_or_else(
            || "/v2/orders?status=all&limit=100&direction=desc&nested=false".to_owned(),
            |cursor: &String| format!("/v2/orders?status=all&limit=100&direction=desc&nested=false&before_order_id={cursor}"),
        );
        let response = http.request(
            ProviderEndpoint::AlpacaPaper,
            ProviderHttpMethod::Get,
            &path,
            auth.clone(),
            None,
        )?;
        if response.status != 200 {
            return Err(alpaca_read_error(response.status));
        }
        let values: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
        if contains_secret(&values, secrets) {
            return Err(invalid());
        }
        let rows = values.as_array().ok_or_else(invalid)?;
        if rows.len() > ALPACA_ORDER_PAGE_SIZE {
            return Err(incomplete());
        }
        let observed_at = crate::storage::timestamp()?;
        let mut cursor = None;
        for value in rows {
            let order = parse_alpaca_order(value, &observed_at)?;
            cursor = Some(order.provider_order_id.clone());
            replace_order(&mut orders, order)?;
            if orders.len() > ALPACA_ORDER_MAX_ORDERS {
                return Err(incomplete());
            }
        }
        if rows.len() < ALPACA_ORDER_PAGE_SIZE {
            return Ok((
                orders,
                fetch_alpaca_paper_fills(http, auth, current, secrets)?,
            ));
        }
        let next = cursor.ok_or_else(incomplete)?;
        if !cursors.insert(next.clone()) || before.as_ref() == Some(&next) {
            return Err(incomplete());
        }
        if page + 1 == ALPACA_ORDER_MAX_PAGES {
            return Err(incomplete());
        }
        before = Some(next);
    }
    Err(incomplete())
}

fn fetch_alpaca_paper_fills(
    http: &impl ProviderHttp,
    auth: &HeaderMap,
    current: &impl Fn() -> bool,
    secrets: &[String],
) -> Result<Vec<AlpacaPaperFill>> {
    let mut fills = Vec::new();
    let mut token: Option<String> = None;
    let mut cursors = std::collections::HashSet::new();
    for page in 0..ALPACA_FILL_MAX_PAGES {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let path = token.as_ref().map_or_else(
            || "/v2/account/activities/FILL?page_size=100&direction=desc".to_owned(),
            |cursor| {
                format!(
                    "/v2/account/activities/FILL?page_size=100&direction=desc&page_token={cursor}"
                )
            },
        );
        let response = http.request(
            ProviderEndpoint::AlpacaPaper,
            ProviderHttpMethod::Get,
            &path,
            auth.clone(),
            None,
        )?;
        if response.status != 200 {
            return Err(alpaca_read_error(response.status));
        }
        let values: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
        if contains_secret(&values, secrets) {
            return Err(invalid());
        }
        let rows = values.as_array().ok_or_else(invalid)?;
        if rows.len() > ALPACA_ORDER_PAGE_SIZE {
            return Err(incomplete());
        }
        let observed_at = crate::storage::timestamp()?;
        let mut cursor = None;
        for value in rows {
            let fill = parse_alpaca_fill(value, &observed_at, AlpacaPaperFillSource::RestActivity)?;
            cursor = Some(fill.activity_id.clone());
            insert_fill(&mut fills, fill)?;
            if fills.len() > ALPACA_FILL_MAX {
                return Err(incomplete());
            }
        }
        if rows.len() < ALPACA_ORDER_PAGE_SIZE {
            return Ok(fills);
        }
        let next = cursor.ok_or_else(incomplete)?;
        if !cursors.insert(next.clone()) || token.as_ref() == Some(&next) {
            return Err(incomplete());
        }
        if page + 1 == ALPACA_FILL_MAX_PAGES {
            return Err(incomplete());
        }
        token = Some(next);
    }
    Err(incomplete())
}

fn get_alpaca_order(
    http: &impl ProviderHttp,
    auth: &HeaderMap,
    order_id: &str,
    secrets: &[String],
) -> Result<AlpacaPaperOrder> {
    if !valid_provider_order_id(order_id) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let response = http.request(
        ProviderEndpoint::AlpacaPaper,
        ProviderHttpMethod::Get,
        &format!("/v2/orders/{order_id}"),
        auth.clone(),
        None,
    )?;
    if response.status != 200 {
        return Err(alpaca_read_error(response.status));
    }
    let value: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
    if contains_secret(&value, secrets) {
        return Err(invalid());
    }
    parse_alpaca_order(&value, &crate::storage::timestamp()?)
}

fn parse_trading212_order(
    value: &Value,
    observed_at: &str,
    pending: bool,
) -> Result<Trading212DemoOrder> {
    let provider_order_id = trading212::order_id(value)?;
    let symbol = if value.get("instrument").is_some_and(Value::is_object) {
        text(&value["instrument"], "ticker", 64)?
    } else {
        text(value, "ticker", 64)?
    };
    if value.get("ticker").is_some() && text(value, "ticker", 64)? != symbol {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
    }
    if !symbol
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"._/-".contains(&byte))
    {
        return Err(invalid());
    }
    let side = text(value, "side", 8)?;
    if !matches!(side.as_str(), "BUY" | "SELL") {
        return Err(invalid());
    }
    let order_type = text(value, "type", 32)?;
    if !matches!(
        order_type.as_str(),
        "MARKET" | "LIMIT" | "STOP" | "STOP_LIMIT"
    ) {
        return Err(invalid());
    }
    let time_in_force = text(value, "timeInForce", 32)?;
    if !matches!(time_in_force.as_str(), "DAY" | "GOOD_TILL_CANCEL") {
        return Err(invalid());
    }
    let provider_status = text(value, "status", 32)?;
    let normalized_status = match provider_status.as_str() {
        "LOCAL" => Trading212DemoNormalizedOrderStatus::Local,
        "UNCONFIRMED" => Trading212DemoNormalizedOrderStatus::Pending,
        "CONFIRMED" | "NEW" => Trading212DemoNormalizedOrderStatus::Open,
        "CANCELLING" => Trading212DemoNormalizedOrderStatus::CancelPending,
        "CANCELLED" => Trading212DemoNormalizedOrderStatus::Cancelled,
        "PARTIALLY_FILLED" => Trading212DemoNormalizedOrderStatus::PartiallyFilled,
        "FILLED" => Trading212DemoNormalizedOrderStatus::Filled,
        "REJECTED" => Trading212DemoNormalizedOrderStatus::Rejected,
        "REPLACING" => Trading212DemoNormalizedOrderStatus::Replacing,
        "REPLACED" => Trading212DemoNormalizedOrderStatus::Replaced,
        _ => return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE")),
    };
    let strategy = text(value, "strategy", 16)?;
    let quantity = match strategy.as_str() {
        "QUANTITY" => optional_trading212_decimal(value, "quantity")?,
        "VALUE" => None,
        _ => return Err(invalid()),
    };
    let filled_quantity = optional_trading212_decimal(value, "filledQuantity")?;
    let filled_value = optional_trading212_decimal(value, "filledValue")?;
    let currency = match value.get("currency") {
        None | Some(Value::Null) => None,
        Some(Value::String(currency))
            if currency.len() == 3 && currency.bytes().all(|byte| byte.is_ascii_uppercase()) =>
        {
            Some(currency.clone())
        }
        _ => return Err(invalid()),
    };
    if (strategy == "QUANTITY" && filled_quantity.is_none())
        || (strategy == "VALUE" && filled_value.is_none())
        || quantity.as_deref().is_some_and(|value| value.len() > 64)
        || filled_quantity
            .as_deref()
            .is_some_and(|value| value.len() > 64)
        || filled_value
            .as_deref()
            .is_some_and(|value| value.starts_with('-') || value.len() > 64)
    {
        return Err(invalid());
    }
    let remaining_quantity = match (quantity.as_deref(), filled_quantity.as_deref()) {
        (Some(quantity), Some(filled)) => {
            let quantity = quantity.strip_prefix('-').unwrap_or(quantity);
            let filled = filled.strip_prefix('-').unwrap_or(filled);
            Some(decimal_subtract(quantity, filled)?)
        }
        _ => None,
    };
    Ok(Trading212DemoOrder {
        provider_order_id,
        symbol,
        side,
        order_type,
        time_in_force,
        provider_status,
        normalized_status,
        pending,
        quantity,
        filled_quantity,
        filled_value,
        currency,
        remaining_quantity,
        submitted_at: provider_timestamp(value, "createdAt")?,
        provider_updated_at: None,
        observed_at: observed_at.to_owned(),
        origin: Trading212DemoOrderOrigin::External,
        attempt_id: None,
    })
}

fn merge_trading212_orders(
    existing: &[Trading212DemoOrder],
    incoming: Vec<Trading212DemoOrder>,
    replace_pending_set: bool,
    refresh_exact_order: bool,
) -> Result<Vec<Trading212DemoOrder>> {
    let mut orders = existing.to_vec();
    if replace_pending_set {
        for order in &mut orders {
            order.pending = false;
        }
    }
    for received in incoming {
        if let Some(current) = orders
            .iter_mut()
            .find(|order| order.provider_order_id == received.provider_order_id)
        {
            if current.symbol != received.symbol
                || current.side != received.side
                || current.order_type != received.order_type
                || current.time_in_force != received.time_in_force
                || current.quantity != received.quantity
                || current.submitted_at != received.submitted_at
                || current.currency.is_some()
                    && received.currency.is_some()
                    && current.currency != received.currency
            {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
            }
            if replace_pending_set || refresh_exact_order || received.pending || !current.pending {
                current.provider_status = received.provider_status;
                current.normalized_status = received.normalized_status;
                current.filled_quantity = received.filled_quantity;
                current.filled_value = received.filled_value;
                current.remaining_quantity = received.remaining_quantity;
                current.observed_at = received.observed_at;
            }
            current.currency = received.currency.or_else(|| current.currency.clone());
            if replace_pending_set || refresh_exact_order {
                current.pending = received.pending;
            } else {
                current.pending |= received.pending;
            }
        } else {
            orders.push(received);
        }
    }
    if orders.len() > 5000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    Ok(orders)
}

pub(crate) fn parse_alpaca_order(value: &Value, observed_at: &str) -> Result<AlpacaPaperOrder> {
    let provider_order_id = id(value, "id")?;
    let client_order_id = text(value, "client_order_id", 128)?;
    let symbol = text(value, "symbol", 16)?;
    if !valid_alpaca_symbol(&symbol) {
        return Err(invalid());
    }
    let side = text(value, "side", 16)?;
    if !matches!(side.as_str(), "buy" | "sell") {
        return Err(invalid());
    }
    let order_type = text(value, "type", 32)?;
    let time_in_force = text(value, "time_in_force", 32)?;
    let provider_status = text(value, "status", 64)?;
    let quantity = optional_decimal(value, "qty")?;
    let filled_quantity = decimal(value.get("filled_qty").ok_or_else(invalid)?)?;
    if filled_quantity.starts_with('-')
        || filled_quantity.len() > 64
        || quantity
            .as_deref()
            .is_some_and(|value| value.starts_with('-') || value.len() > 64)
    {
        return Err(invalid());
    }
    let remaining_quantity = quantity
        .as_deref()
        .map(|qty| decimal_subtract(qty, &filled_quantity))
        .transpose()?;
    let submitted_at = provider_timestamp(value, "submitted_at")
        .or_else(|_| provider_timestamp(value, "created_at"))?;
    let provider_updated_at = value
        .get("updated_at")
        .map(|_| provider_timestamp(value, "updated_at"))
        .transpose()?;
    Ok(AlpacaPaperOrder {
        provider_order_id,
        client_order_id,
        symbol: symbol.clone(),
        instrument_id: market::canonical_instrument_id("alpaca", &symbol),
        side,
        order_type,
        time_in_force,
        provider_status,
        quantity,
        filled_quantity,
        remaining_quantity,
        submitted_at,
        provider_updated_at,
        observed_at: observed_at.to_owned(),
        origin: AlpacaPaperOrderOrigin::External,
        cancel_state: AlpacaPaperCancelState::None,
        cancel_idempotency_key: None,
        cancel_error: None,
    })
}

fn parse_alpaca_fill(
    value: &Value,
    observed_at: &str,
    source: AlpacaPaperFillSource,
) -> Result<AlpacaPaperFill> {
    let activity_id = text(value, "id", 128)?;
    if !valid_activity_token(&activity_id) {
        return Err(invalid());
    }
    alpaca_fill_execution_id(&activity_id, source)?;
    let provider_order_id = id(value, "order_id")?;
    let symbol = text(value, "symbol", 16)?;
    if !valid_alpaca_symbol(&symbol) {
        return Err(invalid());
    }
    let side = text(value, "side", 16)?;
    if !matches!(side.as_str(), "buy" | "sell") {
        return Err(invalid());
    }
    let quantity = decimal(value.get("qty").ok_or_else(invalid)?)?;
    let price = decimal(value.get("price").ok_or_else(invalid)?)?;
    if quantity.starts_with('-') || price.starts_with('-') || quantity == "0" || price == "0" {
        return Err(invalid());
    }
    Ok(AlpacaPaperFill {
        activity_id,
        provider_order_id,
        symbol: symbol.clone(),
        instrument_id: market::canonical_instrument_id("alpaca", &symbol),
        side,
        quantity,
        price,
        source,
        executed_at: provider_timestamp(value, "transaction_time")?,
        observed_at: observed_at.to_owned(),
    })
}

pub(crate) fn parse_alpaca_trade_update(
    value: &Value,
    remote_account_id: &str,
    observed_at: &str,
    secrets: &[String],
) -> Result<(AlpacaPaperOrder, Option<AlpacaPaperFill>)> {
    if contains_secret(value, secrets) || text(value, "stream", 64)? != "trade_updates" {
        return Err(invalid());
    }
    let data = value.get("data").ok_or_else(invalid)?;
    let event = text(data, "event", 64)?;
    let raw_order = data.get("order").ok_or_else(invalid)?;
    if raw_order
        .get("account_id")
        .is_some_and(|account| account.as_str() != Some(remote_account_id))
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let mut order = parse_alpaca_order(raw_order, observed_at)?;
    order.provider_updated_at = data
        .get("timestamp")
        .map(|_| provider_timestamp(data, "timestamp"))
        .transpose()?
        .or(order.provider_updated_at);
    let fill = if matches!(event.as_str(), "partial_fill" | "fill") {
        let execution_id = text(data, "execution_id", 128)?;
        if !valid_activity_token(&execution_id) {
            return Err(invalid());
        }
        let mut fill = data.clone();
        let object = fill.as_object_mut().ok_or_else(invalid)?;
        object.insert("id".into(), Value::String(execution_id));
        object.insert(
            "order_id".into(),
            Value::String(order.provider_order_id.clone()),
        );
        object.insert("symbol".into(), Value::String(order.symbol.clone()));
        object.insert("side".into(), Value::String(order.side.clone()));
        object.insert(
            "transaction_time".into(),
            Value::String(text(data, "timestamp", 64)?),
        );
        Some(parse_alpaca_fill(
            &fill,
            observed_at,
            AlpacaPaperFillSource::TradeUpdate,
        )?)
    } else {
        None
    };
    Ok((order, fill))
}

pub(crate) fn apply_alpaca_trade_update(
    book: &mut AlpacaPaperOrderBook,
    value: &Value,
    remote_account_id: &str,
    secrets: &[String],
) -> Result<()> {
    if book.remote_account_id != remote_account_id {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let observed_at = crate::storage::timestamp()?;
    let (order, fill) = parse_alpaca_trade_update(value, remote_account_id, &observed_at, secrets)?;
    replace_order(&mut book.orders, order)?;
    if let Some(fill) = fill {
        insert_fill(&mut book.fills, fill)?;
    }
    if book.orders.len() > ALPACA_ORDER_MAX_ORDERS || book.fills.len() > ALPACA_FILL_MAX {
        return Err(incomplete());
    }
    book.status = AlpacaPaperOrderBookStatus::Stale;
    book.reason = Some("STREAM_UPDATE_REQUIRES_RECONCILIATION".into());
    book.observed_at = observed_at;
    Ok(())
}

fn provider_timestamp(value: &Value, field: &str) -> Result<String> {
    let timestamp = text(value, field, 64)?;
    time::OffsetDateTime::parse(&timestamp, &time::format_description::well_known::Rfc3339)
        .map_err(|_| invalid())?;
    Ok(timestamp)
}

pub(crate) fn decimal_subtract(left: &str, right: &str) -> Result<String> {
    if decimal_cmp(left, right)? == std::cmp::Ordering::Less {
        return Err(invalid());
    }
    let (left_whole, left_fraction) = left.split_once('.').unwrap_or((left, ""));
    let (right_whole, right_fraction) = right.split_once('.').unwrap_or((right, ""));
    let scale = left_fraction.len().max(right_fraction.len());
    let mut left_digits =
        format!("{left_whole}{left_fraction:0<width$}", width = scale).into_bytes();
    let mut right_digits =
        format!("{right_whole}{right_fraction:0<width$}", width = scale).into_bytes();
    left_digits.reverse();
    right_digits.reverse();
    let mut borrow = 0_i16;
    for (index, digit) in left_digits.iter_mut().enumerate() {
        let left_digit = i16::from(*digit - b'0') - borrow;
        let right_digit = right_digits
            .get(index)
            .map_or(0, |digit| i16::from(*digit - b'0'));
        if left_digit < right_digit {
            *digit = (left_digit + 10 - right_digit) as u8 + b'0';
            borrow = 1;
        } else {
            *digit = (left_digit - right_digit) as u8 + b'0';
            borrow = 0;
        }
    }
    if borrow != 0 {
        return Err(invalid());
    }
    left_digits.reverse();
    let mut result = String::from_utf8(left_digits).map_err(|_| invalid())?;
    let whole_len = result.len().saturating_sub(scale);
    if scale > 0 {
        if whole_len == 0 {
            result.insert_str(0, &"0".repeat(scale - result.len()));
        }
        let point = result.len() - scale;
        result.insert(point, '.');
    }
    let normalized = decimal(&Value::String(result))?;
    Ok(normalized)
}

fn same_order_identity(left: &AlpacaPaperOrder, right: &AlpacaPaperOrder) -> bool {
    left.provider_order_id == right.provider_order_id
        && left.client_order_id == right.client_order_id
        && left.symbol == right.symbol
        && left.instrument_id == right.instrument_id
        && left.side == right.side
        && left.order_type == right.order_type
        && left.time_in_force == right.time_in_force
        && left.quantity == right.quantity
        && left.submitted_at == right.submitted_at
}

fn same_order_review(left: &AlpacaPaperOrder, right: &AlpacaPaperOrder) -> bool {
    same_order_identity(left, right)
        && left.provider_status == right.provider_status
        && left.filled_quantity == right.filled_quantity
        && left.remaining_quantity == right.remaining_quantity
}

fn preserve_local_order_state(
    mut fresh: AlpacaPaperOrder,
    previous: &AlpacaPaperOrder,
) -> AlpacaPaperOrder {
    fresh.origin = previous.origin;
    if is_terminal_order_status(&fresh.provider_status) {
        fresh.cancel_state = AlpacaPaperCancelState::None;
        fresh.cancel_idempotency_key = None;
        fresh.cancel_error = None;
    } else {
        fresh.cancel_state = previous.cancel_state;
        fresh.cancel_idempotency_key = previous.cancel_idempotency_key.clone();
        fresh.cancel_error = previous.cancel_error.clone();
    }
    fresh
}

fn replace_order(orders: &mut Vec<AlpacaPaperOrder>, fresh: AlpacaPaperOrder) -> Result<()> {
    if let Some(index) = orders
        .iter()
        .position(|order| order.provider_order_id == fresh.provider_order_id)
    {
        let previous = orders[index].clone();
        if !same_order_identity(&previous, &fresh) {
            return Err(incomplete());
        }
        if let (Some(previous_at), Some(fresh_at)) = (
            previous.provider_updated_at.as_deref(),
            fresh.provider_updated_at.as_deref(),
        ) {
            let previous_at = time::OffsetDateTime::parse(
                previous_at,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| incomplete())?;
            let fresh_at = time::OffsetDateTime::parse(
                fresh_at,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| incomplete())?;
            if previous_at > fresh_at {
                return Ok(());
            }
        }
        orders[index] = preserve_local_order_state(fresh, &previous);
    } else {
        orders.push(fresh);
    }
    Ok(())
}

fn insert_fill(fills: &mut Vec<AlpacaPaperFill>, fresh: AlpacaPaperFill) -> Result<()> {
    let fresh_execution_id = alpaca_fill_execution_id(&fresh.activity_id, fresh.source)?.to_owned();
    for (index, previous) in fills.iter().enumerate() {
        if alpaca_fill_execution_id(&previous.activity_id, previous.source)? != fresh_execution_id {
            continue;
        }
        if previous.provider_order_id != fresh.provider_order_id
            || previous.symbol != fresh.symbol
            || previous.instrument_id != fresh.instrument_id
            || previous.side != fresh.side
            || previous.quantity != fresh.quantity
            || previous.price != fresh.price
            || previous.executed_at != fresh.executed_at
        {
            return Err(incomplete());
        }
        if previous.source == AlpacaPaperFillSource::TradeUpdate
            && fresh.source == AlpacaPaperFillSource::RestActivity
        {
            fills[index] = fresh;
        }
        return Ok(());
    }
    fills.push(fresh);
    Ok(())
}

fn alpaca_fill_execution_id(activity_id: &str, source: AlpacaPaperFillSource) -> Result<&str> {
    let execution_id = match source {
        AlpacaPaperFillSource::RestActivity => {
            let Some((timestamp, execution_id)) = activity_id.split_once("::") else {
                return Err(invalid());
            };
            if timestamp.len() != 17 || !timestamp.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid());
            }
            execution_id
        }
        AlpacaPaperFillSource::TradeUpdate => activity_id,
    };
    uuid::Uuid::parse_str(execution_id).map_err(|_| invalid())?;
    Ok(execution_id)
}

fn merge_order_pages(
    previous: &[AlpacaPaperOrder],
    fresh: Vec<AlpacaPaperOrder>,
) -> Result<Vec<AlpacaPaperOrder>> {
    let mut merged = previous.to_vec();
    for order in fresh {
        replace_order(&mut merged, order)?;
        if merged.len() > ALPACA_ORDER_PAGE_SIZE * ALPACA_ORDER_MAX_PAGES {
            return Err(incomplete());
        }
    }
    merged.sort_by(|left, right| right.submitted_at.cmp(&left.submitted_at));
    Ok(merged)
}

fn merge_fill_pages(
    previous: &[AlpacaPaperFill],
    fresh: Vec<AlpacaPaperFill>,
) -> Result<Vec<AlpacaPaperFill>> {
    let mut merged = previous.to_vec();
    for fill in fresh {
        insert_fill(&mut merged, fill)?;
        if merged.len() > ALPACA_ORDER_PAGE_SIZE * ALPACA_FILL_MAX_PAGES {
            return Err(incomplete());
        }
    }
    merged.sort_by(|left, right| right.executed_at.cmp(&left.executed_at));
    Ok(merged)
}

fn cancelable_status(status: &str) -> bool {
    matches!(
        status,
        "new" | "accepted" | "pending_new" | "partially_filled"
    )
}

fn is_terminal_order_status(status: &str) -> bool {
    matches!(
        status,
        "filled" | "canceled" | "expired" | "rejected" | "replaced"
    )
}

fn incomplete() -> TradeXError {
    TradeXError::new("PROVIDER_RESPONSE_INCOMPLETE")
}

pub(crate) fn alpaca_headers(values: &[String]) -> Result<HeaderMap> {
    if values.len() != 2 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let mut headers = HeaderMap::new();
    for (name, value) in [
        ("APCA-API-KEY-ID", &values[0]),
        ("APCA-API-SECRET-KEY", &values[1]),
    ] {
        let mut header =
            HeaderValue::from_str(value).map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
        header.set_sensitive(true);
        headers.insert(name, header);
    }
    Ok(headers)
}

fn alpaca_order_request(
    proposal: &OrderProposal,
    attempt: &AlpacaPaperOrderAttempt,
) -> Result<(String, Value, bool)> {
    let fields = &proposal.fields;
    if !matches!(
        proposal.status,
        crate::protocol::OrderProposalStatus::NeedsApproval
            | crate::protocol::OrderProposalStatus::Consumed
    ) || fields.environment != ExecutionContext::AlpacaPaper
        || fields.account_id.as_deref() != Some(attempt.connection_id.as_str())
        || attempt.proposal_id != proposal.proposal_id
        || attempt.proposal_hash != proposal.proposal_hash
        || fields.maximum_spend.is_some()
    {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    let instrument = market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == fields.instrument_id)
        .filter(|instrument| instrument.asset_class == crate::protocol::AssetClass::Equity)
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))?;
    let provider_symbol = instrument
        .providers
        .iter()
        .find(|mapping| mapping.provider_id == "alpaca")
        .map(|mapping| mapping.provider_symbol.clone())
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))?;
    if provider_symbol.is_empty()
        || !provider_symbol.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
    {
        return Err(TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"));
    }
    let type_name = match fields.order_type {
        OrderType::Market => "market",
        OrderType::Limit => "limit",
    };
    let time_in_force = match (fields.order_type, fields.time_in_force) {
        (OrderType::Market, TimeInForce::Day) => "day",
        (OrderType::Limit, TimeInForce::Day) => "day",
        (OrderType::Limit, TimeInForce::Gtc) => "gtc",
        _ => return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED")),
    };
    let (amount_field, amount_value, needs_fractional) = match fields.quantity.r#type {
        OrderQuantityType::Base => {
            let fraction = fields
                .quantity
                .value
                .split_once('.')
                .map(|(_, fraction)| fraction)
                .unwrap_or("");
            if fraction.len() > 9 {
                return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
            }
            if !fraction.is_empty()
                && (fields.order_type != OrderType::Market
                    || fields.time_in_force != TimeInForce::Day)
            {
                return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED"));
            }
            (
                "qty",
                fields.quantity.value.as_str(),
                !fraction.trim_end_matches('0').is_empty(),
            )
        }
        OrderQuantityType::Quote => {
            if fields.side != OrderSide::Buy
                || fields.order_type != OrderType::Market
                || fields.time_in_force != TimeInForce::Day
                || fields
                    .quantity
                    .value
                    .split_once('.')
                    .map_or(0, |(_, f)| f.len())
                    > 2
            {
                return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED"));
            }
            ("notional", fields.quantity.value.as_str(), true)
        }
    };
    let mut body = json!({
        "symbol": provider_symbol,
        "side": match fields.side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" },
        "type": type_name,
        "time_in_force": time_in_force,
        "client_order_id": attempt.client_order_id,
    });
    body[amount_field] = Value::String(amount_value.into());
    match fields.order_type {
        OrderType::Limit => {
            let limit_price = fields
                .limit_price
                .as_deref()
                .ok_or_else(|| TradeXError::new("ORDER_AMOUNT_INVALID"))?;
            body["limit_price"] = Value::String(limit_price.into());
        }
        OrderType::Market if fields.limit_price.is_some() => {
            return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED"));
        }
        OrderType::Market => (),
    }
    Ok((provider_symbol, body, needs_fractional))
}

pub(crate) fn validate_alpaca_paper_proposal(
    proposal: &OrderProposal,
    connection_id: &str,
) -> Result<()> {
    let attempt = AlpacaPaperOrderAttempt {
        attempt_id: "preflight".into(),
        workspace_id: proposal.workspace_id.clone(),
        connection_id: connection_id.into(),
        remote_account_id: "preflight".into(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_hash: proposal.proposal_hash.clone(),
        client_order_id: "tradex-preflight".into(),
        state: AlpacaPaperOrderAttemptState::Submitting,
        provider_order_id: None,
        provider_status: None,
        error_code: None,
        reason: "preflight".into(),
        state_version: "preflight".into(),
        created_at: "preflight".into(),
        updated_at: "preflight".into(),
    };
    alpaca_order_request(proposal, &attempt).map(|_| ())
}

pub(crate) fn validate_trading212_demo_proposal(
    proposal: &OrderProposal,
    connection_id: &str,
) -> Result<()> {
    trading212_demo_order_request(proposal, connection_id).map(|_| ())
}

fn trading212_demo_order_request(
    proposal: &OrderProposal,
    connection_id: &str,
) -> Result<(String, Value)> {
    let fields = &proposal.fields;
    if !matches!(
        proposal.status,
        crate::protocol::OrderProposalStatus::NeedsApproval
            | crate::protocol::OrderProposalStatus::Consumed
    ) || fields.environment != ExecutionContext::Trading212Demo
        || fields.account_id.as_deref() != Some(connection_id)
        || fields.quantity.r#type != OrderQuantityType::Base
        || fields.maximum_spend.is_some()
    {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    let instrument = market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == fields.instrument_id)
        .filter(|instrument| instrument.asset_class == crate::protocol::AssetClass::Equity)
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))?;
    if instrument.exchange.as_deref() != Some(fields.venue.as_str()) {
        return Err(TradeXError::new("ORDER_VENUE_INVALID"));
    }
    let ticker = instrument
        .providers
        .iter()
        .find(|mapping| mapping.provider_id == "trading212")
        .map(|mapping| mapping.provider_symbol.as_str())
        .filter(|ticker| {
            !ticker.is_empty()
                && ticker.len() <= 64
                && ticker
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        })
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))?;
    let quantity = decimal(&Value::String(fields.quantity.value.clone()))?;
    if decimal_cmp(&quantity, "0")? != std::cmp::Ordering::Greater {
        return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
    }
    let signed_quantity = match fields.side {
        OrderSide::Buy => quantity,
        OrderSide::Sell => format!("-{quantity}"),
    };
    let quantity = serde_json::from_str::<Value>(&signed_quantity).map_err(|_| invalid())?;
    let mut body = json!({"ticker":ticker,"quantity":quantity});
    let path = match (fields.order_type, fields.time_in_force) {
        (OrderType::Market, TimeInForce::Day) if fields.limit_price.is_none() => {
            body["extendedHours"] = Value::Bool(false);
            "/api/v0/equity/orders/market"
        }
        (OrderType::Limit, TimeInForce::Day | TimeInForce::Gtc) => {
            let raw_price = fields
                .limit_price
                .as_deref()
                .ok_or_else(|| TradeXError::new("ORDER_LIMIT_PRICE_REQUIRED"))?;
            let price = decimal(&Value::String(raw_price.into()))?;
            if decimal_cmp(&price, "0")? != std::cmp::Ordering::Greater {
                return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
            }
            body["limitPrice"] = serde_json::from_str(&price).map_err(|_| invalid())?;
            body["timeValidity"] = Value::String(
                match fields.time_in_force {
                    TimeInForce::Day => "DAY",
                    TimeInForce::Gtc => "GOOD_TILL_CANCEL",
                    _ => unreachable!(),
                }
                .into(),
            );
            "/api/v0/equity/orders/limit"
        }
        _ => return Err(TradeXError::new("ORDER_CAPABILITY_UNSUPPORTED")),
    };
    Ok((path.into(), body))
}

fn trading212_unknown_attempt(
    mut attempt: Trading212DemoOrderAttempt,
) -> Trading212DemoOrderAttempt {
    attempt.state = Trading212DemoOrderAttemptState::UnknownReconciling;
    attempt.provider_order_id = None;
    attempt.provider_status = None;
    attempt.error_code = Some("ORDER_STATUS_UNKNOWN".into());
    attempt.reason = "Trading 212 may have received the order, but its response could not be verified. Do not resubmit; query the account for evidence.".into();
    attempt
}

fn trading212_demo_acknowledgement(
    bytes: &[u8],
    secrets: &[String],
    attempt: &Trading212DemoOrderAttempt,
    proposal: &OrderProposal,
    body: &Value,
) -> Result<Trading212DemoOrderAttempt> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let provider_order_id = trading212::order_id(&value)?;
    if contains_secret(&value, secrets)
        || text(&value, "ticker", 64)? != body["ticker"].as_str().ok_or_else(invalid)?
        || value.get("instrument").is_some_and(|instrument| {
            instrument
                .get("ticker")
                .and_then(Value::as_str)
                .is_some_and(|ticker| ticker != body["ticker"].as_str().unwrap_or_default())
        })
        || text(&value, "side", 8)?
            != match proposal.fields.side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            }
        || text(&value, "strategy", 16)? != "QUANTITY"
        || text(&value, "type", 16)?
            != match proposal.fields.order_type {
                OrderType::Market => "MARKET",
                OrderType::Limit => "LIMIT",
            }
    {
        return Err(invalid());
    }
    let expected_quantity = decimal(&Value::String(proposal.fields.quantity.value.clone()))?;
    let reported_quantity = trading212::number(&value["quantity"])?;
    if decimal_cmp(
        reported_quantity
            .strip_prefix('-')
            .unwrap_or(&reported_quantity),
        &expected_quantity,
    )? != std::cmp::Ordering::Equal
    {
        return Err(invalid());
    }
    let expected_tif = match proposal.fields.time_in_force {
        TimeInForce::Day => "DAY",
        TimeInForce::Gtc => "GOOD_TILL_CANCEL",
        _ => return Err(invalid()),
    };
    if text(&value, "timeInForce", 32)? != expected_tif {
        return Err(invalid());
    }
    if proposal.fields.order_type == OrderType::Market {
        if value.get("extendedHours").and_then(Value::as_bool) != Some(false) {
            return Err(invalid());
        }
    } else {
        let expected_price = decimal(&Value::String(
            proposal.fields.limit_price.clone().ok_or_else(invalid)?,
        ))?;
        let reported_price = trading212::number(&value["limitPrice"])?;
        if decimal_cmp(&reported_price, &expected_price)? != std::cmp::Ordering::Equal {
            return Err(invalid());
        }
    }
    let mut acknowledged = attempt.clone();
    acknowledged.state = Trading212DemoOrderAttemptState::Acknowledged;
    acknowledged.provider_order_id = Some(provider_order_id);
    acknowledged.provider_status = Some(text(&value, "status", 32)?);
    acknowledged.error_code = None;
    acknowledged.reason =
        "Trading 212 returned a matching Demo order record. Acknowledgement is not fill evidence."
            .into();
    Ok(acknowledged)
}

fn validate_alpaca_asset(asset: &Value, symbol: &str, needs_fractional: bool) -> Result<String> {
    let asset_id = id(asset, "id")?;
    if text(asset, "symbol", 64)? != symbol || text(asset, "class", 32)? != "us_equity" {
        return Err(invalid());
    }
    if text(asset, "status", 32)? != "active" || !flag(asset, "tradable")? {
        return Err(TradeXError::new("ORDER_ASSET_UNTRADABLE"));
    }
    if needs_fractional && !flag(asset, "fractionable")? {
        return Err(TradeXError::new("ORDER_ASSET_NOT_FRACTIONABLE"));
    }
    Ok(asset_id)
}

fn validate_alpaca_sell_position(
    position: &Value,
    symbol: &str,
    asset_id: &str,
    quantity: &str,
) -> Result<()> {
    if id(position, "asset_id")? != asset_id
        || text(position, "symbol", 64)? != symbol
        || text(position, "asset_class", 32)? != "us_equity"
    {
        return Err(invalid());
    }
    if text(position, "side", 16)? != "long"
        || decimal_cmp(quantity, &decimal(&position["qty_available"])?)?
            == std::cmp::Ordering::Greater
    {
        return Err(TradeXError::new("ORDER_INSUFFICIENT_POSITION"));
    }
    Ok(())
}

fn alpaca_acknowledgement(
    bytes: &[u8],
    request: &Value,
    symbol: &str,
    expected_asset_id: Option<&str>,
    prior: &AlpacaPaperOrderAttempt,
) -> Result<AlpacaPaperOrderAttempt> {
    let order: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let provider_order_id = id(&order, "id")?;
    if let Some(expected_asset_id) = expected_asset_id
        && id(&order, "asset_id")? != expected_asset_id
    {
        return Err(invalid());
    }
    if text(&order, "client_order_id", 128)? != prior.client_order_id
        || text(&order, "symbol", 64)? != symbol
        || text(&order, "asset_class", 32)? != "us_equity"
        || text(&order, "side", 16)? != request["side"].as_str().ok_or_else(invalid)?
        || order
            .get("type")
            .or_else(|| order.get("order_type"))
            .and_then(Value::as_str)
            != request.get("type").and_then(Value::as_str)
        || text(&order, "time_in_force", 16)?
            != request["time_in_force"].as_str().ok_or_else(invalid)?
    {
        return Err(invalid());
    }
    for field in ["qty", "notional", "limit_price"] {
        if let Some(expected) = request.get(field).and_then(Value::as_str)
            && optional_decimal(&order, field)?.as_deref() != Some(expected)
        {
            return Err(invalid());
        }
    }
    let mut attempt = prior.clone();
    attempt.state = AlpacaPaperOrderAttemptState::Acknowledged;
    attempt.provider_order_id = Some(provider_order_id);
    attempt.provider_status = Some(text(&order, "status", 64)?);
    attempt.error_code = None;
    attempt.reason =
        "Alpaca Paper acknowledged the order. This acknowledgement is not fill evidence.".into();
    Ok(attempt)
}

fn reconcile_alpaca_attempt(
    http: &impl ProviderHttp,
    auth: &HeaderMap,
    attempt: AlpacaPaperOrderAttempt,
    proposal: &OrderProposal,
) -> AlpacaPaperOrderAttempt {
    let account_response = match http.request(
        ProviderEndpoint::AlpacaPaper,
        ProviderHttpMethod::Get,
        "/v2/account",
        auth.clone(),
        None,
    ) {
        Ok(response) if response.status == 200 => response.body,
        Ok(response) => {
            let error = alpaca_read_error(response.status);
            return unknown_attempt(
                attempt,
                &error.code,
                "The saved Alpaca Paper account identity could not be verified. Order status remains unknown; do not resubmit.",
            );
        }
        Err(error) => {
            return unknown_attempt(
                attempt,
                &error.code,
                "The saved Alpaca Paper account identity could not be verified. Order status remains unknown; do not resubmit.",
            );
        }
    };
    let account: Value = match serde_json::from_slice(&account_response) {
        Ok(account) => account,
        Err(_) => {
            return unknown_attempt(
                attempt,
                "PROVIDER_RESPONSE_INVALID",
                "Alpaca returned an invalid account identity. Order status remains unknown; do not resubmit.",
            );
        }
    };
    let remote_account_id = match id(&account, "id") {
        Ok(account_id) => account_id,
        Err(error) => {
            return unknown_attempt(
                attempt,
                &error.code,
                "Alpaca returned an invalid account identity. Order status remains unknown; do not resubmit.",
            );
        }
    };
    if remote_account_id != attempt.remote_account_id {
        return unknown_attempt(
            attempt,
            "PROVIDER_IDENTITY_CHANGED",
            "The current Alpaca Paper credentials resolve to a different account. Restore the saved account identity before reconciling; do not resubmit.",
        );
    }
    let Ok((symbol, request, _)) = alpaca_order_request(proposal, &attempt) else {
        return unknown_attempt(
            attempt,
            "ORDER_STATUS_UNKNOWN",
            "Alpaca Paper order status could not be verified; no retry was sent.",
        );
    };
    let path = format!(
        "/v2/orders:by_client_order_id?client_order_id={}",
        attempt.client_order_id
    );
    match http.request(
        ProviderEndpoint::AlpacaPaper,
        ProviderHttpMethod::Get,
        &path,
        auth.clone(),
        None,
    ) {
        Ok(response) if response.status == 200 => {
            match alpaca_acknowledgement(&response.body, &request, &symbol, None, &attempt) {
                Ok(acknowledged) => acknowledged,
                Err(_) => unknown_attempt(
                    attempt,
                    "PROVIDER_RESPONSE_INVALID",
                    "Alpaca returned an order that did not match the saved client order identity; reconcile before any further action.",
                ),
            }
        }
        Ok(response) if response.status == 404 => unknown_attempt(
            attempt,
            "ORDER_STATUS_UNKNOWN",
            "Alpaca Paper has not returned this client order ID yet. A single empty lookup does not prove the order was not accepted.",
        ),
        Ok(response) => {
            let error = alpaca_read_error(response.status);
            unknown_attempt(
                attempt,
                &error.code,
                "Alpaca Paper order status remains unknown. Retry reconciliation; do not resubmit.",
            )
        }
        Err(error) => unknown_attempt(
            attempt,
            &error.code,
            "Alpaca Paper order status remains unknown. Retry reconciliation; do not resubmit.",
        ),
    }
}

fn unknown_attempt(
    mut attempt: AlpacaPaperOrderAttempt,
    error_code: &str,
    reason: &str,
) -> AlpacaPaperOrderAttempt {
    attempt.state = AlpacaPaperOrderAttemptState::UnknownReconciling;
    attempt.provider_order_id = None;
    attempt.provider_status = None;
    attempt.error_code = Some(error_code.into());
    attempt.reason = reason.into();
    attempt
}

fn alpaca_read_error(status: u16) -> TradeXError {
    TradeXError::new(match status {
        401 => "PROVIDER_AUTH_FAILED",
        403 => "PROVIDER_PERMISSION_BLOCKED",
        418 | 429 => "PROVIDER_RATE_LIMITED",
        404 => "PROVIDER_RESPONSE_INVALID",
        400..=499 => "PROVIDER_RESPONSE_INVALID",
        _ => "PROVIDER_UNAVAILABLE",
    })
}

fn alpaca_order_rejection(status: u16, body: &[u8], side: &str) -> TradeXError {
    match status {
        401 => return TradeXError::new("PROVIDER_AUTH_FAILED"),
        418 | 429 => return TradeXError::new("PROVIDER_RATE_LIMITED"),
        403 => (),
        _ => return TradeXError::new("PROVIDER_ORDER_REJECTED"),
    }

    let response = serde_json::from_slice::<Value>(body).ok();
    let message = response
        .as_ref()
        .and_then(|value| value.get("message").and_then(Value::as_str))
        .map(|message| {
            message
                .chars()
                .take(256)
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .unwrap_or_default();
    let insufficient = message.contains("insufficient")
        || message.contains("not sufficient")
        || message.contains("not enough");

    // ponytail: classify known provider phrases only; changed or unknown 403 messages stay generic until Alpaca documents stable equity rejection codes.
    let account_permission_blocked = (message.contains("not authorized")
        && message.contains("account"))
        || message.contains("account not eligible");
    let error_code = if account_permission_blocked {
        "PROVIDER_PERMISSION_BLOCKED"
    } else if side == "buy"
        && insufficient
        && (message.contains("buying power")
            || message.contains("tradable balance")
            || message.contains("balance"))
    {
        "ORDER_BUYING_POWER_INSUFFICIENT"
    } else if side == "sell"
        && insufficient
        && (message.contains("shares")
            || message.contains("available qty")
            || message.contains("position"))
    {
        "ORDER_INSUFFICIENT_POSITION"
    } else {
        "PROVIDER_ORDER_REJECTED"
    };
    TradeXError::new(error_code)
}

fn decimal_cmp(left: &str, right: &str) -> Result<std::cmp::Ordering> {
    if left.starts_with('-') || right.starts_with('-') {
        return Err(invalid());
    }
    let (left_whole, left_fraction) = left.split_once('.').unwrap_or((left, ""));
    let (right_whole, right_fraction) = right.split_once('.').unwrap_or((right, ""));
    let left_whole = left_whole.trim_start_matches('0');
    let right_whole = right_whole.trim_start_matches('0');
    let left_whole = if left_whole.is_empty() {
        "0"
    } else {
        left_whole
    };
    let right_whole = if right_whole.is_empty() {
        "0"
    } else {
        right_whole
    };
    let whole_order = left_whole
        .len()
        .cmp(&right_whole.len())
        .then_with(|| left_whole.cmp(right_whole));
    if whole_order != std::cmp::Ordering::Equal {
        return Ok(whole_order);
    }
    for index in 0..left_fraction.len().max(right_fraction.len()) {
        let left_digit = left_fraction.as_bytes().get(index).copied().unwrap_or(b'0');
        let right_digit = right_fraction
            .as_bytes()
            .get(index)
            .copied()
            .unwrap_or(b'0');
        match left_digit.cmp(&right_digit) {
            std::cmp::Ordering::Equal => (),
            order => return Ok(order),
        }
    }
    Ok(std::cmp::Ordering::Equal)
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
    if ["/v2/assets/", "/v2/positions/"]
        .iter()
        .any(|prefix| path.strip_prefix(prefix).is_some_and(valid_alpaca_symbol))
    {
        return true;
    }
    if path
        .strip_prefix("/v2/orders:by_client_order_id?client_order_id=")
        .is_some_and(valid_client_order_id)
    {
        return true;
    }
    if path
        .strip_prefix("/v2/orders/")
        .is_some_and(valid_provider_order_id)
    {
        return true;
    }
    if path == "/v2/orders?status=all&limit=100&direction=desc&nested=false"
        || path == "/v2/account/activities/FILL?page_size=100&direction=desc"
    {
        return true;
    }
    path.strip_prefix(
        "/v2/orders?status=all&limit=100&direction=desc&nested=false&before_order_id=",
    )
    .is_some_and(valid_provider_order_id)
        || path
            .strip_prefix("/v2/account/activities/FILL?page_size=100&direction=desc&page_token=")
            .is_some_and(valid_activity_token)
        || path
            .strip_prefix(
                "/v2/orders?status=open&limit=500&direction=asc&nested=false&after_order_id=",
            )
            .is_some_and(valid_provider_order_id)
}

fn valid_provider_order_id(id: &str) -> bool {
    id.len() == 36 && uuid::Uuid::parse_str(id).is_ok()
}

fn valid_activity_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn valid_alpaca_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol.len() <= 16
        && symbol
            .bytes()
            .any(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        && symbol.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}

fn valid_client_order_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
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

#[cfg(test)]
mod trading212_demo_route_tests {
    use super::{
        ProviderEndpoint, ProviderHttpMethod, Trading212ReadEndpoint, merge_trading212_orders,
        parse_trading212_order, trading212_order_read_error, valid_t212_history_path,
    };
    use serde_json::{Value, json};

    #[test]
    fn order_posts_are_restricted_to_the_two_demo_routes() {
        assert_eq!(
            ProviderEndpoint::Trading212Demo.base_url(),
            "https://demo.trading212.com"
        );
        for path in [
            "/api/v0/equity/orders/market",
            "/api/v0/equity/orders/limit",
        ] {
            assert!(ProviderEndpoint::Trading212Demo.allows_method(ProviderHttpMethod::Post, path));
            assert!(
                !ProviderEndpoint::Trading212Live.allows_method(ProviderHttpMethod::Post, path)
            );
        }
        assert!(
            !ProviderEndpoint::Trading212Demo
                .allows_method(ProviderHttpMethod::Post, "/api/v0/equity/orders")
        );
        assert!(
            !ProviderEndpoint::Trading212Demo
                .allows_method(ProviderHttpMethod::Delete, "/api/v0/equity/orders/1")
        );
        assert!(ProviderEndpoint::Trading212Demo.allows("/api/v0/equity/orders"));
        assert!(ProviderEndpoint::Trading212Demo.allows("/api/v0/equity/orders/9007199254740995"));
        assert!(
            ProviderEndpoint::Trading212Demo
                .allows("/api/v0/equity/history/orders?limit=50&cursor=1760346100000")
        );
        assert!(
            !ProviderEndpoint::Trading212Live
                .allows("/api/v0/equity/history/orders?limit=50&cursor=1760346100000")
        );
        for path in [
            "/api/v0/equity/orders/0",
            "/api/v0/equity/orders/9007199254740995?x=1",
            "/api/v0/equity/history/orders?limit=500",
            "/api/v0/equity/history/orders?limit=50&cursor=1&cursor=2",
            "/api/v0/equity/history/orders?limit=50&next=https://evil.test",
            "/api/v0/equity/history/orders/../orders?limit=50",
        ] {
            assert!(!ProviderEndpoint::Trading212Demo.allows(path), "{path}");
        }
        assert!(valid_t212_history_path(
            "/api/v0/equity/history/orders?cursor=1760346100000&limit=50"
        ));
    }

    #[test]
    fn trading212_orders_keep_exact_cumulative_fills_and_reject_unknown_states() {
        let partial = parse_trading212_order(
            &serde_json::from_str::<Value>(
                r#"{
                "id":9007199254740995,"ticker":"AAPL_US_EQ","side":"SELL",
                "type":"LIMIT","timeInForce":"GOOD_TILL_CANCEL","strategy":"QUANTITY",
                "quantity":-5.000,"filledQuantity":-1.25,"filledValue":12.34567890123456789,"currency":"GBP",
                "status":"PARTIALLY_FILLED","createdAt":"2026-09-23T10:00:00Z"
            }"#,
            )
            .unwrap(),
            "2026-09-23T10:01:00Z",
            true,
        )
        .unwrap();
        assert_eq!(partial.provider_order_id, "9007199254740995");
        assert_eq!(partial.provider_status, "PARTIALLY_FILLED");
        assert_eq!(partial.filled_quantity.as_deref(), Some("-1.25"));
        assert_eq!(
            partial.filled_value.as_deref(),
            Some("12.34567890123456789")
        );
        assert_eq!(partial.remaining_quantity.as_deref(), Some("3.75"));
        assert_eq!(partial.currency.as_deref(), Some("GBP"));
        assert!(partial.pending);

        let filled = parse_trading212_order(
            &json!({
                "id":22,"ticker":"MSFT_US_EQ","side":"BUY","type":"MARKET",
                "timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":5,"filledValue":110.5,"status":"FILLED",
                "createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:01:00Z",
            false,
        )
        .unwrap();
        assert_eq!(filled.remaining_quantity.as_deref(), Some("0"));
        assert!(!filled.pending);

        let invalid_currency = json!({
            "id":24,"ticker":"MSFT_US_EQ","side":"BUY","type":"MARKET",
            "timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
            "filledQuantity":5,"filledValue":110.5,"status":"FILLED",
            "currency":"gbp","createdAt":"2026-09-23T10:00:00Z"
        });
        assert_eq!(
            parse_trading212_order(&invalid_currency, "2026-09-23T10:01:00Z", false)
                .unwrap_err()
                .code,
            "PROVIDER_RESPONSE_INVALID"
        );

        let mut unknown = json!({
            "id":23,"ticker":"MSFT_US_EQ","side":"BUY","type":"MARKET",
            "timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
            "filledQuantity":0,"status":"SOMETHING_NEW",
            "createdAt":"2026-09-23T10:00:00Z"
        });
        assert_eq!(
            parse_trading212_order(&unknown, "2026-09-23T10:01:00Z", false)
                .unwrap_err()
                .code,
            "PROVIDER_DATA_INCOMPLETE"
        );
        unknown["status"] = json!("FILLED");
        unknown["filledQuantity"] = json!(6);
        assert_eq!(
            parse_trading212_order(&unknown, "2026-09-23T10:01:00Z", false)
                .unwrap_err()
                .code,
            "PROVIDER_RESPONSE_INVALID"
        );
    }

    #[test]
    fn trading212_order_reads_classify_client_and_server_errors_separately() {
        assert_eq!(
            trading212_order_read_error(418, Trading212ReadEndpoint::PendingOrders).code,
            "PROVIDER_RESPONSE_INVALID"
        );
        assert_eq!(
            trading212_order_read_error(503, Trading212ReadEndpoint::PendingOrders).code,
            "PROVIDER_UNAVAILABLE"
        );
        assert_eq!(
            trading212_order_read_error(404, Trading212ReadEndpoint::OrderDetail).code,
            "ORDER_STATUS_UNKNOWN"
        );
    }

    #[test]
    fn trading212_order_merge_retains_known_currency_and_rejects_conflicts() {
        let known = parse_trading212_order(
            &json!({
                "id":22,"ticker":"MSFT_US_EQ","side":"BUY","type":"MARKET",
                "timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":1,"filledValue":10,"status":"PARTIALLY_FILLED",
                "currency":"GBP","createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:01:00Z",
            true,
        )
        .unwrap();
        let missing = parse_trading212_order(
            &json!({
                "id":22,"ticker":"MSFT_US_EQ","side":"BUY","type":"MARKET",
                "timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":2,"filledValue":20,"status":"PARTIALLY_FILLED",
                "createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:02:00Z",
            true,
        )
        .unwrap();
        let merged =
            merge_trading212_orders(std::slice::from_ref(&known), vec![missing], false, true)
                .unwrap();
        assert_eq!(merged[0].currency.as_deref(), Some("GBP"));

        let conflict = parse_trading212_order(
            &json!({
                "id":22,"ticker":"MSFT_US_EQ","side":"BUY","type":"MARKET",
                "timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":2,"filledValue":20,"status":"PARTIALLY_FILLED",
                "currency":"USD","createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:02:00Z",
            true,
        )
        .unwrap();
        assert_eq!(
            merge_trading212_orders(&[known], vec![conflict], false, true)
                .unwrap_err()
                .code,
            "PROVIDER_IDENTITY_CONFLICT"
        );
    }
}
fn optional_decimal(value: &Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => decimal(v).map(Some),
    }
}
fn optional_trading212_decimal(value: &Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => trading212::number(value).map(Some),
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
                instrument_id: None,
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
                kind: None,
                trigger_price: None,
                broker_order_id,
                symbol: symbol(o)?,
                instrument_id: None,
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
            buying_power: optional_decimal(&account, "buying_power")?,
            balances: vec![Balance {
                locked: None,
                restricted_available: None,
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

fn valid_time(n: u64) -> bool {
    (946684800000..4102444800000).contains(&n)
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

#[cfg(test)]
mod alpaca_trade_update_tests {
    use super::*;

    const ACCOUNT_ID: &str = "81161e77-bafd-44bb-b2a0-60b9055e3cd4";
    const ORDER_ID: &str = "18c65e3e-feb0-4576-99e2-36e6f047d84d";

    fn frame(event: &str, status: &str, filled: &str, timestamp: &str) -> Value {
        serde_json::json!({
            "stream":"trade_updates",
            "data":{
                "event":event,
            "execution_id":"00000000-0000-4000-8000-000000000002",
                "qty":"0.5",
                "price":"10.25",
                "timestamp":timestamp,
                "order":{
                    "id":ORDER_ID,
                    "account_id":ACCOUNT_ID,
                    "client_order_id":"trade-x-order-1",
                    "symbol":"AAPL",
                    "side":"buy",
                    "type":"limit",
                    "time_in_force":"day",
                    "status":status,
                    "qty":"1",
                    "filled_qty":filled,
                    "submitted_at":"2026-09-23T10:00:00Z",
                    "updated_at":timestamp
                }
            }
        })
    }

    #[test]
    fn updates_dedupe_fills_ignore_late_states_and_preserve_unknown_statuses() {
        let mut book = AlpacaPaperOrderBook {
            workspace_id: "workspace-1".into(),
            connection_id: "connection-1".into(),
            remote_account_id: ACCOUNT_ID.into(),
            status: AlpacaPaperOrderBookStatus::NeverSynced,
            state_version: "alpaca-paper-order-book:connection-1:0".into(),
            last_successful_sync_at: None,
            observed_at: "2026-09-23T10:00:00Z".into(),
            reason: None,
            orders: Vec::new(),
            fills: Vec::new(),
        };
        let partial = frame(
            "partial_fill",
            "partially_filled",
            "0.5",
            "2026-09-23T10:02:00Z",
        );
        apply_alpaca_trade_update(&mut book, &partial, ACCOUNT_ID, &[]).unwrap();
        apply_alpaca_trade_update(&mut book, &partial, ACCOUNT_ID, &[]).unwrap();
        assert_eq!(book.fills.len(), 1);
        assert_eq!(book.orders[0].provider_status, "partially_filled");
        assert_eq!(book.fills[0].source, AlpacaPaperFillSource::TradeUpdate);

        let rest_fill = parse_alpaca_fill(
            &serde_json::json!({
                "id":"20260923100200000::00000000-0000-4000-8000-000000000002",
                "order_id":ORDER_ID,
                "symbol":"AAPL",
                "side":"buy",
                "qty":"0.5",
                "price":"10.25",
                "transaction_time":"2026-09-23T10:02:00Z"
            }),
            "2026-09-23T10:03:00Z",
            AlpacaPaperFillSource::RestActivity,
        )
        .unwrap();
        insert_fill(&mut book.fills, rest_fill).unwrap();
        assert_eq!(book.fills.len(), 1);
        assert_eq!(book.fills[0].source, AlpacaPaperFillSource::RestActivity);
        assert!(book.fills[0].activity_id.contains("::"));
        apply_alpaca_trade_update(&mut book, &partial, ACCOUNT_ID, &[]).unwrap();
        assert_eq!(book.fills.len(), 1);
        assert_eq!(book.fills[0].source, AlpacaPaperFillSource::RestActivity);

        let conflicting_rest_fill = parse_alpaca_fill(
            &serde_json::json!({
                "id":"20260923100200000::00000000-0000-4000-8000-000000000002",
                "order_id":ORDER_ID,
                "symbol":"AAPL",
                "side":"buy",
                "qty":"0.4",
                "price":"10.25",
                "transaction_time":"2026-09-23T10:02:00Z"
            }),
            "2026-09-23T10:03:00Z",
            AlpacaPaperFillSource::RestActivity,
        )
        .unwrap();
        assert!(insert_fill(&mut book.fills, conflicting_rest_fill).is_err());

        let late = frame("new", "new", "0", "2026-09-23T10:01:00Z");
        apply_alpaca_trade_update(&mut book, &late, ACCOUNT_ID, &[]).unwrap();
        assert_eq!(book.orders[0].provider_status, "partially_filled");

        let unknown = frame(
            "provider_transition",
            "future_provider_state",
            "0.5",
            "2026-09-23T10:03:00Z",
        );
        apply_alpaca_trade_update(&mut book, &unknown, ACCOUNT_ID, &[]).unwrap();
        assert_eq!(book.orders[0].provider_status, "future_provider_state");
        assert_eq!(book.fills.len(), 1);
    }

    #[test]
    fn stream_updates_preserve_supported_lifecycle_states() {
        for (event, status, filled, has_fill) in [
            ("new", "new", "0", false),
            ("partial_fill", "partially_filled", "0.5", true),
            ("fill", "filled", "1", true),
            ("rejected", "rejected", "0", false),
            ("canceled", "canceled", "0", false),
            ("expired", "expired", "0", false),
        ] {
            let mut book = AlpacaPaperOrderBook {
                workspace_id: "workspace-1".into(),
                connection_id: "connection-1".into(),
                remote_account_id: ACCOUNT_ID.into(),
                status: AlpacaPaperOrderBookStatus::NeverSynced,
                state_version: "alpaca-paper-order-book:connection-1:0".into(),
                last_successful_sync_at: None,
                observed_at: "2026-09-23T10:00:00Z".into(),
                reason: None,
                orders: Vec::new(),
                fills: Vec::new(),
            };
            let update = frame(event, status, filled, "2026-09-23T10:02:00Z");

            apply_alpaca_trade_update(&mut book, &update, ACCOUNT_ID, &[]).unwrap();

            assert_eq!(book.orders[0].provider_status, status);
            assert_eq!(book.fills.len(), usize::from(has_fill));
            assert_eq!(
                book.status,
                AlpacaPaperOrderBookStatus::Stale,
                "every stream event requires REST reconciliation"
            );
        }
    }

    #[test]
    fn stream_updates_reject_cross_account_identity_and_secret_reflection() {
        let event = frame("new", "new", "0", "2026-09-23T10:01:00Z");
        let error = parse_alpaca_trade_update(
            &event,
            "different-remote-account",
            "2026-09-23T10:01:00Z",
            &[],
        )
        .unwrap_err();
        assert_eq!(error.code, "PROVIDER_IDENTITY_CHANGED");

        let mut reflected = event;
        reflected["diagnostic"] = "fixture-private-secret".into();
        let error = parse_alpaca_trade_update(
            &reflected,
            ACCOUNT_ID,
            "2026-09-23T10:01:00Z",
            &["fixture-private-secret".into()],
        )
        .unwrap_err();
        assert_eq!(error.code, "PROVIDER_RESPONSE_INVALID");
    }
}
