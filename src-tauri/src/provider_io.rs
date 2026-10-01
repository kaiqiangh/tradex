use crate::{
    market,
    protocol::{
        AlpacaPaperCancelState, AlpacaPaperFill, AlpacaPaperFillSource, AlpacaPaperOrder,
        AlpacaPaperOrderAttempt, AlpacaPaperOrderAttemptState, AlpacaPaperOrderBook,
        AlpacaPaperOrderBookStatus, AlpacaPaperOrderOrigin, BinanceTestnetOrderAttempt,
        BinanceTestnetOrderAttemptState, BinanceTestnetOrderBook, BinanceTestnetOrderBookAction,
        BinanceTestnetOrderBookStatus, BinanceTestnetOrderCancelState, BitgetDemoOrderAttempt,
        BitgetDemoOrderAttemptState, CancellationIntent, ExecutionAttempt, ExecutionAttemptState,
        ExecutionContext, OrderProposal, OrderQuantityType, OrderSide, OrderType,
        ProviderOrderCandidate, ResolutionEvidence, ResolutionEvidenceLedger,
        ResolutionEvidenceOutcome, ResolutionEvidenceRefresh, Result, TimeInForce, TradeXError,
        Trading212DemoCancelState, Trading212DemoNormalizedOrderStatus, Trading212DemoOrder,
        Trading212DemoOrderAttempt, Trading212DemoOrderAttemptState, Trading212DemoOrderBook,
        Trading212DemoOrderBookStatus, Trading212DemoOrderOrigin,
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
    sync::{Condvar, Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
static TRADING212_ENDPOINT_LIMITS: OnceLock<Mutex<HashMap<(String, &'static str), i64>>> =
    OnceLock::new();
static PROVIDER_SCHEDULER: OnceLock<ProviderScheduler> = OnceLock::new();

pub(crate) fn binance_live_client_order_id(attempt_id: &str) -> Result<String> {
    binance::live_client_order_id(attempt_id)
}

pub(crate) fn bitget_live_client_order_id(attempt_id: &str) -> Result<String> {
    bitget::live_client_order_id(attempt_id)
}

pub(crate) fn binance_testnet_private_stream_subscription(
    http: &impl ProviderHttp,
    secrets: &[String],
    current: &impl Fn() -> bool,
) -> Result<(String, Value)> {
    binance::private_stream_subscription(http, secrets, current)
}

pub(crate) fn verify_binance_testnet_private_stream_account(
    http: &impl ProviderHttp,
    secrets: &[String],
    remote_account_id: &str,
    current: &impl Fn() -> bool,
) -> Result<()> {
    binance::verify_private_stream_account(http, secrets, remote_account_id, current)
}

pub(crate) fn merge_binance_testnet_cancel_observation(
    latest: &mut BinanceTestnetOrderBook,
    outcome: &BinanceTestnetOrderBook,
    symbol: &str,
    provider_order_id: &str,
) -> Result<()> {
    binance::merge_testnet_cancel_observation(latest, outcome, symbol, provider_order_id)
}

pub(crate) fn apply_binance_testnet_private_stream_frame(
    book: &mut BinanceTestnetOrderBook,
    frame: &Value,
    subscription_id: u64,
    secrets: &[String],
) -> Result<Option<(String, bool)>> {
    binance::apply_testnet_private_stream_frame(book, frame, subscription_id, secrets)
        .map(|update| update.map(|update| (update.event_at, update.reconciliation_required)))
}

pub(crate) fn reconcile_binance_testnet_private_stream(
    book: &mut BinanceTestnetOrderBook,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<()> {
    binance::reconcile_testnet_order_book(book, secrets, http, current)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Trading212Endpoint {
    PendingOrders,
    OrderDetail,
    History,
    CancelOrder,
}

impl Trading212Endpoint {
    fn key(self) -> &'static str {
        match self {
            Self::PendingOrders => "orders",
            Self::OrderDetail => "order-detail",
            Self::History => "history-orders",
            Self::CancelOrder => "cancel-order",
        }
    }
    fn minimum_interval(self) -> i64 {
        match self {
            Self::PendingOrders => 5,
            Self::OrderDetail => 1,
            Self::History => 10,
            Self::CancelOrder => 2,
        }
    }
    fn deadline(self, book: &Trading212DemoOrderBook) -> &Option<String> {
        match self {
            Self::PendingOrders => &book.rate_limits.pending_orders_retry_at,
            Self::OrderDetail => &book.rate_limits.order_detail_retry_at,
            Self::History => &book.rate_limits.history_retry_at,
            Self::CancelOrder => &book.rate_limits.cancel_order_retry_at,
        }
    }
    fn deadline_mut(self, book: &mut Trading212DemoOrderBook) -> &mut Option<String> {
        match self {
            Self::PendingOrders => &mut book.rate_limits.pending_orders_retry_at,
            Self::OrderDetail => &mut book.rate_limits.order_detail_retry_at,
            Self::History => &mut book.rate_limits.history_retry_at,
            Self::CancelOrder => &mut book.rate_limits.cancel_order_retry_at,
        }
    }
}

fn trading212_order_read_error(status: u16, endpoint: Trading212Endpoint) -> TradeXError {
    match status {
        401 => TradeXError::new("PROVIDER_AUTH_FAILED"),
        403 => TradeXError::new("PROVIDER_PERMISSION_BLOCKED"),
        404 if endpoint == Trading212Endpoint::OrderDetail => {
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
fn reserve_trading212_endpoint(
    book: &mut Trading212DemoOrderBook,
    endpoint: Trading212Endpoint,
) -> Result<String> {
    let now = unix_now();
    let persisted = endpoint
        .deadline(book)
        .as_deref()
        .and_then(parsed_timestamp)
        .unwrap_or_default();
    let limits = TRADING212_ENDPOINT_LIMITS.get_or_init(|| Mutex::new(HashMap::new()));
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
    endpoint: Trading212Endpoint,
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
        let limits = TRADING212_ENDPOINT_LIMITS.get_or_init(|| Mutex::new(HashMap::new()));
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
    AlpacaMarketData,
    Trading212Demo,
    Trading212Live,
    BinanceTestnet,
    BinanceLive,
    BitgetDemo,
    BitgetLive,
}
impl ProviderEndpoint {
    fn scheduler_provider(self) -> &'static str {
        match self {
            Self::AlpacaPaper | Self::AlpacaMarketData => "alpaca",
            Self::Trading212Demo | Self::Trading212Live => "trading212",
            Self::BinanceTestnet | Self::BinanceLive => "binance",
            Self::BitgetDemo | Self::BitgetLive => "bitget",
        }
    }

    pub fn base_url(self) -> &'static str {
        match self {
            Self::AlpacaPaper => "https://paper-api.alpaca.markets",
            Self::AlpacaMarketData => "https://data.alpaca.markets",
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
            Self::AlpacaMarketData => crate::quote_source::allowed_latest_path(path),
            Self::BitgetDemo => bitget::allows(path),
            Self::BitgetLive => {
                bitget::allows(path)
                    || bitget::allows_live_history(path)
                    || bitget::allows_live_cancel_observation(path)
            }
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
            Self::Trading212Live => {
                matches!(
                    path,
                    "/api/v0/equity/account/summary"
                        | "/api/v0/equity/positions"
                        | "/api/v0/equity/orders"
                ) || valid_t212_order_detail_path(path)
                    || valid_t212_history_path(path)
            }
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
                    || (self == Self::BinanceTestnet
                        && path.starts_with("/api/v3/order?")
                        && self.allows(path))
                    || (self == Self::BitgetDemo && path == "/api/v2/spot/trade/place-order")
                    || (cfg!(feature = "order-gateway-runtime")
                        && self == Self::Trading212Live
                        && matches!(
                            path,
                            "/api/v0/equity/orders/market" | "/api/v0/equity/orders/limit"
                        ))
                    || (cfg!(feature = "order-gateway-runtime")
                        && self == Self::BitgetLive
                        && path == "/api/v2/spot/trade/cancel-order")
            }
            ProviderHttpMethod::Delete => {
                self == Self::AlpacaPaper
                    && path
                        .strip_prefix("/v2/orders/")
                        .is_some_and(valid_provider_order_id)
                    || self == Self::Trading212Demo && valid_t212_order_detail_path(path)
                    || self == Self::BinanceTestnet && binance::allows_cancel(path)
                    || (cfg!(feature = "order-gateway-runtime")
                        && self == Self::BinanceLive
                        && binance::allows_live_cancel(path))
                    || (cfg!(feature = "order-gateway-runtime")
                        && self == Self::Trading212Live
                        && valid_t212_order_detail_path(path))
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

pub(crate) fn valid_live_cancel_order_id(provider_id: &str, value: &str) -> bool {
    match provider_id {
        "trading212" => valid_t212_order_id(value),
        "binance" => value.split_once(':').is_some_and(|(symbol, id)| {
            binance::valid_binance_symbol(symbol) && binance::valid_order_id(id)
        }),
        "bitget" => value
            .strip_prefix("normal:")
            .is_some_and(bitget::valid_live_cancel_number),
        _ => false,
    }
}

pub(crate) fn valid_binance_order_id(value: &str) -> bool {
    binance::valid_order_id(value)
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
    let mut ticker = false;
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
            "ticker"
                if !ticker
                    && !value.is_empty()
                    && value.len() <= 64
                    && value.bytes().all(|byte| {
                        byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'
                    }) =>
            {
                ticker = true
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
    let retry_after_seconds = headers
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    (remaining.is_some() || reset_at.is_some() || retry_after_seconds.is_some()).then_some(
        ProviderRateLimit {
            remaining,
            reset_at,
            retry_after_seconds,
        },
    )
}

fn classify_get_response(
    endpoint: ProviderEndpoint,
    status: u16,
    bytes: Vec<u8>,
) -> Result<Vec<u8>> {
    match status {
        200 => (),
        400 if endpoint.is_binance() || endpoint.is_bitget() => (),
        401 | 403 => return Err(TradeXError::new("PROVIDER_AUTH_FAILED")),
        418 | 429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
        _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
    }
    if status != 200 && endpoint.is_bitget() {
        let value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        return Err(bitget::business(value).err().unwrap_or_else(invalid));
    }
    if status != 200 {
        let code = serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|value| value["code"].as_i64());
        return Err(TradeXError::new(match code {
            Some(-1021) => "CLOCK_SKEW",
            Some(-1022 | -2014 | -2015) => "PROVIDER_AUTH_FAILED",
            Some(-1003 | -1015) => "PROVIDER_RATE_LIMITED",
            _ => "PROVIDER_RESPONSE_INVALID",
        }));
    }
    Ok(bytes)
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
    pub retry_after_seconds: Option<u64>,
}

pub trait ProviderHttp {
    fn get(&self, endpoint: ProviderEndpoint, path: &str, headers: HeaderMap) -> Result<Vec<u8>>;

    fn get_response_with_rate_limit(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: HeaderMap,
    ) -> Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
        self.get(endpoint, path, headers)
            .map(|body| (ProviderHttpResponse { status: 200, body }, None))
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum ProviderPriority {
    P0,
    P1,
    P2,
    P3,
}

const PROVIDER_CONCURRENCY: usize = 4;
const PROVIDER_QUEUE_LIMIT: usize = 32;
const PROVIDER_LOW_QUEUE_LIMIT: usize = 24;
const PROVIDER_WAIT_POLL: Duration = Duration::from_millis(100);

#[derive(Clone)]
struct ProviderWaiter {
    ticket: u64,
    priority: ProviderPriority,
    account_id: Option<String>,
}

#[derive(Clone, Copy, Default)]
struct ProviderCooldown {
    until: Option<Instant>,
    unrepresentable: bool,
}

#[derive(Default)]
struct ProviderQueue {
    active: usize,
    active_non_p0: usize,
    next_ticket: u64,
    waiters: Vec<ProviderWaiter>,
    provider_cooldown: ProviderCooldown,
    account_cooldowns: HashMap<String, ProviderCooldown>,
}

#[derive(Default)]
struct ProviderSchedulerState {
    providers: HashMap<&'static str, ProviderQueue>,
}

struct ProviderScheduler {
    state: Mutex<ProviderSchedulerState>,
    available: Condvar,
    concurrency: usize,
    queue_limit: usize,
    low_queue_limit: usize,
}

impl ProviderScheduler {
    fn with_limits(concurrency: usize, queue_limit: usize, low_queue_limit: usize) -> Self {
        Self {
            state: Mutex::new(ProviderSchedulerState::default()),
            available: Condvar::new(),
            concurrency: concurrency.max(1),
            queue_limit,
            low_queue_limit: low_queue_limit.min(queue_limit),
        }
    }

    fn acquire(
        &self,
        provider: &'static str,
        priority: ProviderPriority,
        current: &dyn Fn() -> bool,
    ) -> Result<ProviderPermit<'_>> {
        self.acquire_for_account(provider, None, priority, current)
    }

    fn acquire_for_account(
        &self,
        provider: &'static str,
        account_id: Option<&str>,
        priority: ProviderPriority,
        current: &dyn Fn() -> bool,
    ) -> Result<ProviderPermit<'_>> {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let queue = state.providers.entry(provider).or_default();
        let low_waiters = queue
            .waiters
            .iter()
            .filter(|waiter| waiter.priority >= ProviderPriority::P2)
            .count();
        if queue.waiters.len() >= self.queue_limit
            || (priority >= ProviderPriority::P2 && low_waiters >= self.low_queue_limit)
        {
            return Err(TradeXError::new("PROVIDER_BACKPRESSURE"));
        }
        let ticket = queue.next_ticket;
        queue.next_ticket = queue.next_ticket.wrapping_add(1);
        queue.waiters.push(ProviderWaiter {
            ticket,
            priority,
            account_id: account_id.map(str::to_owned),
        });

        loop {
            drop(state);
            let is_current = current();
            state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            let queue = state
                .providers
                .get_mut(provider)
                .expect("provider queue exists");
            if !is_current {
                queue.waiters.retain(|waiter| waiter.ticket != ticket);
                self.available.notify_all();
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            if queue.provider_cooldown.unrepresentable
                || account_id.is_some_and(|account_id| {
                    queue
                        .account_cooldowns
                        .get(account_id)
                        .is_some_and(|cooldown| cooldown.unrepresentable)
                })
            {
                queue.waiters.retain(|waiter| waiter.ticket != ticket);
                self.available.notify_all();
                return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
            }
            let now = Instant::now();
            if queue
                .provider_cooldown
                .until
                .is_some_and(|deadline| deadline <= now)
            {
                queue.provider_cooldown.until = None;
            }
            queue.account_cooldowns.retain(|_, cooldown| {
                cooldown.unrepresentable || cooldown.until.is_some_and(|deadline| deadline > now)
            });
            let next = queue
                .waiters
                .iter()
                .filter(|waiter| {
                    !provider_cooldown_active(queue, waiter.account_id.as_deref(), now)
                })
                .min_by_key(|waiter| (waiter.priority, waiter.ticket))
                .map(|waiter| waiter.ticket);
            if next == Some(ticket)
                && queue.active < self.concurrency
                && (priority == ProviderPriority::P0
                    || queue.active_non_p0 < self.concurrency.saturating_sub(1).max(1))
                && !provider_cooldown_active(queue, account_id, now)
            {
                queue.waiters.retain(|waiter| waiter.ticket != ticket);
                queue.active += 1;
                if priority != ProviderPriority::P0 {
                    queue.active_non_p0 += 1;
                }
                return Ok(ProviderPermit {
                    scheduler: self,
                    provider,
                    priority,
                });
            }
            let wait = queue
                .provider_cooldown
                .until
                .into_iter()
                .chain(
                    account_id
                        .and_then(|account_id| queue.account_cooldowns.get(account_id))
                        .and_then(|cooldown| cooldown.until),
                )
                .max()
                .map(|deadline| {
                    deadline
                        .saturating_duration_since(now)
                        .min(PROVIDER_WAIT_POLL)
                })
                .unwrap_or(PROVIDER_WAIT_POLL);
            let (next_state, _) = self
                .available
                .wait_timeout(state, wait)
                .unwrap_or_else(|error| error.into_inner());
            state = next_state;
        }
    }

    fn delay_provider(&self, provider: &'static str, account_id: Option<&str>, delay: Duration) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let queue = state.providers.entry(provider).or_default();
        let cooldown = match account_id {
            Some(account_id) => queue
                .account_cooldowns
                .entry(account_id.to_owned())
                .or_default(),
            None => &mut queue.provider_cooldown,
        };
        match Instant::now().checked_add(delay) {
            Some(deadline) if !cooldown.unrepresentable => {
                cooldown.until = Some(
                    cooldown
                        .until
                        .map_or(deadline, |current| current.max(deadline)),
                );
            }
            None => cooldown.unrepresentable = true,
            _ => (),
        }
        self.available.notify_all();
    }

    #[cfg(test)]
    fn queued(&self, provider: &'static str) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .providers
            .get(provider)
            .map_or(0, |queue| queue.waiters.len())
    }
}

fn provider_cooldown_active(queue: &ProviderQueue, account_id: Option<&str>, now: Instant) -> bool {
    queue
        .provider_cooldown
        .until
        .is_some_and(|deadline| deadline > now)
        || queue.provider_cooldown.unrepresentable
        || account_id
            .and_then(|account_id| queue.account_cooldowns.get(account_id))
            .is_some_and(|cooldown| {
                cooldown.unrepresentable || cooldown.until.is_some_and(|deadline| deadline > now)
            })
}

pub(crate) struct ProviderPermit<'a> {
    scheduler: &'a ProviderScheduler,
    provider: &'static str,
    priority: ProviderPriority,
}

impl Drop for ProviderPermit<'_> {
    fn drop(&mut self) {
        let mut state = self
            .scheduler
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(queue) = state.providers.get_mut(self.provider) {
            queue.active = queue.active.saturating_sub(1);
            if self.priority != ProviderPriority::P0 {
                queue.active_non_p0 = queue.active_non_p0.saturating_sub(1);
            }
        }
        self.scheduler.available.notify_all();
    }
}

fn provider_scheduler() -> &'static ProviderScheduler {
    PROVIDER_SCHEDULER.get_or_init(|| {
        ProviderScheduler::with_limits(
            PROVIDER_CONCURRENCY,
            PROVIDER_QUEUE_LIMIT,
            PROVIDER_LOW_QUEUE_LIMIT,
        )
    })
}

pub(crate) fn with_provider_slot<T>(
    provider: &'static str,
    priority: ProviderPriority,
    current: &dyn Fn() -> bool,
    operation: impl FnOnce() -> T,
) -> Result<T> {
    let _permit = provider_scheduler().acquire(provider, priority, current)?;
    Ok(operation())
}

pub(crate) fn acquire_p0_provider_slot(
    provider: &'static str,
    account_id: &str,
) -> Result<ProviderPermit<'static>> {
    provider_scheduler().acquire_for_account(
        provider,
        Some(account_id),
        ProviderPriority::P0,
        &|| true,
    )
}

// Only sanitized cooldown metadata crosses the internal Gateway channel.
pub fn provider_retry_after_seconds(provider: &str, account_id: &str) -> Option<u64> {
    let state = provider_scheduler()
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let queue = state.providers.get(provider)?;
    let cooldowns = [
        Some(&queue.provider_cooldown),
        queue.account_cooldowns.get(account_id),
    ];
    if cooldowns
        .iter()
        .flatten()
        .any(|cooldown| cooldown.unrepresentable)
    {
        return Some(u64::MAX);
    }
    cooldowns
        .iter()
        .flatten()
        .filter_map(|cooldown| cooldown.until)
        .filter_map(|deadline| deadline.checked_duration_since(Instant::now()))
        .map(|duration| {
            duration
                .as_secs()
                .saturating_add(u64::from(duration.subsec_nanos() > 0))
        })
        .max()
}

pub(crate) fn record_provider_retry_after(
    provider: &'static str,
    account_id: Option<&str>,
    retry_after_seconds: Option<u64>,
) {
    provider_scheduler().delay_provider(
        provider,
        account_id,
        Duration::from_secs(retry_after_seconds.unwrap_or(1)),
    );
}

fn record_provider_rate_limit(
    endpoint: ProviderEndpoint,
    account_id: Option<&str>,
    rate_limit: Option<&ProviderRateLimit>,
) {
    let retry_after = rate_limit
        .and_then(|limit| limit.retry_after_seconds)
        .or_else(|| {
            rate_limit
                .and_then(|limit| limit.reset_at.as_deref())
                .and_then(parsed_timestamp)
                .map(|reset| reset.saturating_sub(unix_now()).max(1) as u64)
        })
        .unwrap_or(1);
    provider_scheduler().delay_provider(
        endpoint.scheduler_provider(),
        if matches!(
            endpoint,
            ProviderEndpoint::BinanceLive | ProviderEndpoint::BinanceTestnet
        ) {
            None // Binance request-weight limits apply to the shared IP.
        } else {
            account_id
        },
        Duration::from_secs(retry_after),
    );
}

struct PrioritizedProviderHttp<'a, H> {
    inner: &'a H,
    priority: ProviderPriority,
    current: &'a dyn Fn() -> bool,
    account_id: &'a str,
}

pub fn p0_provider_http<'a, H: ProviderHttp>(
    inner: &'a H,
    current: &'a dyn Fn() -> bool,
    account_id: &'a str,
) -> impl ProviderHttp + 'a {
    prioritized_provider_http(inner, current, account_id, ProviderPriority::P0)
}

pub fn p1_provider_http<'a, H: ProviderHttp>(
    inner: &'a H,
    current: &'a dyn Fn() -> bool,
    account_id: &'a str,
) -> impl ProviderHttp + 'a {
    prioritized_provider_http(inner, current, account_id, ProviderPriority::P1)
}

pub(crate) fn p3_provider_http<'a, H: ProviderHttp>(
    inner: &'a H,
    current: &'a dyn Fn() -> bool,
    account_id: &'a str,
) -> impl ProviderHttp + 'a {
    prioritized_provider_http(inner, current, account_id, ProviderPriority::P3)
}

fn prioritized_provider_http<'a, H: ProviderHttp>(
    inner: &'a H,
    current: &'a dyn Fn() -> bool,
    account_id: &'a str,
    priority: ProviderPriority,
) -> PrioritizedProviderHttp<'a, H> {
    PrioritizedProviderHttp {
        inner,
        priority,
        current,
        account_id,
    }
}

impl<H: ProviderHttp> ProviderHttp for PrioritizedProviderHttp<'_, H> {
    fn get(&self, endpoint: ProviderEndpoint, path: &str, headers: HeaderMap) -> Result<Vec<u8>> {
        let _permit = provider_scheduler().acquire_for_account(
            endpoint.scheduler_provider(),
            Some(self.account_id),
            self.priority,
            self.current,
        )?;
        let result = self
            .inner
            .get_response_with_rate_limit(endpoint, path, headers)
            .and_then(|(response, limit)| {
                if response.status == 429 {
                    record_provider_rate_limit(endpoint, Some(self.account_id), limit.as_ref());
                }
                classify_get_response(endpoint, response.status, response.body)
            });
        if result
            .as_ref()
            .is_err_and(|error| error.code == "PROVIDER_RATE_LIMITED")
        {
            record_provider_rate_limit(endpoint, Some(self.account_id), None);
        }
        result
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
        let _permit = provider_scheduler().acquire_for_account(
            endpoint.scheduler_provider(),
            Some(self.account_id),
            self.priority,
            self.current,
        )?;
        let result = self
            .inner
            .request_with_rate_limit(endpoint, method, path, headers, body);
        match &result {
            Ok((response, limit)) if response.status == 429 => {
                record_provider_rate_limit(endpoint, Some(self.account_id), limit.as_ref());
            }
            Err(error) if error.code == "PROVIDER_RATE_LIMITED" => {
                record_provider_rate_limit(endpoint, Some(self.account_id), None);
            }
            _ => (),
        }
        result
    }
}

pub enum PrivilegedLiveOperation<'a> {
    Place(&'a OrderProposal),
    Cancel(&'a CancellationIntent),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LiveDispatchOutcome {
    pub state: crate::protocol::ExecutionAttemptState,
    pub broker_order_id: Option<String>,
    pub provider_status: Option<String>,
    pub error_code: Option<String>,
}

pub struct PreparedLiveMutation {
    endpoint: ProviderEndpoint,
    path: String,
    method: ProviderHttpMethod,
    body: Option<Value>,
    headers: HeaderMap,
    secrets: Zeroizing<Vec<String>>,
    expected_ticker: Option<String>,
    order_id: Option<String>,
    cancel_provider_status: Option<String>,
    binance_live_cancel: Option<(String, String)>,
    bitget_live_cancel: Option<(String, String)>,
}

pub fn prepare_trading212_live_mutation(
    account: &AccountConnection,
    credential_reference: &str,
    operation: PrivilegedLiveOperation<'_>,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Result<PreparedLiveMutation> {
    if account.provider_id != "trading212"
        || account.environment != "LIVE"
        || credential_reference != account.credential_ref()
        || account.data.is_none()
    {
        return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
    }
    let (path, method, body, expected_ticker, order_id, cancel_intent) = match operation {
        PrivilegedLiveOperation::Place(proposal) => {
            if proposal.workspace_id != account.workspace_id
                || proposal.fields.account_id.as_deref() != Some(&account.connection_id)
                || proposal.fields.environment != ExecutionContext::Trading212Live
                || proposal.status != crate::protocol::OrderProposalStatus::Consumed
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let (path, body) = trading212_live_order_request(proposal, &account.connection_id)?;
            (
                path,
                ProviderHttpMethod::Post,
                Some(body),
                Some(body_ticker_for_live_proposal(proposal)?),
                None,
                None,
            )
        }
        PrivilegedLiveOperation::Cancel(intent) => {
            if intent.workspace_id != account.workspace_id
                || intent.account_id != account.connection_id
                || intent.environment != ExecutionContext::Trading212Live
                || !valid_t212_order_id(&intent.provider_order_id)
            {
                return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
            }
            (
                format!("/api/v0/equity/orders/{}", intent.provider_order_id),
                ProviderHttpMethod::Delete,
                None,
                None,
                Some(intent.provider_order_id.clone()),
                Some(intent.clone()),
            )
        }
    };
    let credential = vault.get(credential_reference)?;
    let mut secrets = credential.values()?;
    if secrets.len() != 2 || secrets[0].contains(':') {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    use base64::Engine;
    let combined = Zeroizing::new(format!("{}:{}", secrets[0], secrets[1]));
    let encoded =
        Zeroizing::new(base64::engine::general_purpose::STANDARD.encode(combined.as_bytes()));
    let authorization = Zeroizing::new(format!("Basic {}", encoded.as_str()));
    let mut header = HeaderValue::from_str(authorization.as_str())
        .map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
    header.set_sensitive(true);
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", header);
    secrets.push(encoded.to_string());

    let identity = http.request(
        ProviderEndpoint::Trading212Live,
        ProviderHttpMethod::Get,
        "/api/v0/equity/account/summary",
        headers.clone(),
        None,
    )?;
    if identity.status != 200 {
        return Err(trading212_order_read_error(
            identity.status,
            Trading212Endpoint::OrderDetail,
        ));
    }
    let identity: Value = serde_json::from_slice(&identity.body).map_err(|_| invalid())?;
    let expected_remote_id = account
        .data
        .as_ref()
        .map(|data| data.remote_account_id.as_str())
        .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
    if contains_secret(&identity, &secrets)
        || trading212::account_id(&identity)? != expected_remote_id
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let cancel_provider_status = if let Some(intent) = cancel_intent.as_ref() {
        let response = http.request(
            ProviderEndpoint::Trading212Live,
            ProviderHttpMethod::Get,
            &format!("/api/v0/equity/orders/{}", intent.provider_order_id),
            headers.clone(),
            None,
        )?;
        if response.status != 200 {
            return Err(if response.status == 404 {
                TradeXError::new("ORDER_NOT_CANCELLABLE")
            } else {
                trading212_order_read_error(response.status, Trading212Endpoint::OrderDetail)
            });
        }
        if response.body.len() as u64 > MAX_RESPONSE {
            return Err(invalid());
        }
        let order: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
        if contains_secret(&order, &secrets) {
            return Err(invalid());
        }
        Some(validate_trading212_live_cancel_snapshot(&order, intent)?)
    } else {
        None
    };
    Ok(PreparedLiveMutation {
        endpoint: ProviderEndpoint::Trading212Live,
        path,
        method,
        body,
        headers,
        secrets,
        expected_ticker,
        order_id,
        cancel_provider_status,
        binance_live_cancel: None,
        bitget_live_cancel: None,
    })
}

pub fn prepare_binance_live_cancel_mutation(
    account: &AccountConnection,
    credential_reference: &str,
    intent: &CancellationIntent,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Result<PreparedLiveMutation> {
    if account.provider_id != "binance"
        || account.environment != "LIVE"
        || credential_reference != account.credential_ref()
        || intent.workspace_id != account.workspace_id
        || intent.account_id != account.connection_id
        || intent.environment != ExecutionContext::BinanceLive
    {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    }
    let Some((symbol, order_id)) = intent.provider_order_id.split_once(':') else {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    };
    if !binance::valid_order_id(order_id)
        || !binance::valid_binance_symbol(symbol)
        || symbol != intent.symbol
    {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    }
    let expected_remote_id = account
        .data
        .as_ref()
        .map(|data| data.remote_account_id.as_str())
        .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
    let credential = vault.get(credential_reference)?;
    let secrets = credential.values()?;
    if secrets.len() != 2 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let current = || true;
    let (server_time, sampled) =
        binance::server_time_for(ProviderEndpoint::BinanceLive, http, &current)?;
    let identity = binance::signed_request_for(
        ProviderEndpoint::BinanceLive,
        http,
        ProviderHttpMethod::Get,
        "/api/v3/account",
        &[],
        &secrets,
        server_time,
        sampled,
        &current,
    )?;
    if identity.status != 200 {
        return Err(binance_live_order_read_error(identity.status, false));
    }
    let identity = binance::response_value(identity, &secrets)?;
    if identity["accountType"].as_str() != Some("SPOT")
        || identity["uid"]
            .as_u64()
            .map(|value| value.to_string())
            .as_deref()
            != Some(expected_remote_id)
    {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
    }
    let order = binance::signed_request_for(
        ProviderEndpoint::BinanceLive,
        http,
        ProviderHttpMethod::Get,
        "/api/v3/order",
        &[("symbol", symbol), ("orderId", order_id)],
        &secrets,
        server_time,
        sampled,
        &current,
    )?;
    if order.status != 200 {
        return Err(binance_live_order_read_error(order.status, true));
    }
    let order = binance::response_value(order, &secrets)?;
    let cancel_provider_status = binance::validate_live_cancel_snapshot(&order, intent)?;
    Ok(PreparedLiveMutation {
        endpoint: ProviderEndpoint::BinanceLive,
        path: String::new(),
        method: ProviderHttpMethod::Delete,
        body: None,
        headers: HeaderMap::new(),
        secrets,
        expected_ticker: None,
        order_id: Some(intent.provider_order_id.clone()),
        cancel_provider_status: Some(cancel_provider_status),
        binance_live_cancel: Some((symbol.to_owned(), order_id.to_owned())),
        bitget_live_cancel: None,
    })
}

pub fn prepare_bitget_live_cancel_mutation(
    account: &AccountConnection,
    credential_reference: &str,
    intent: &CancellationIntent,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Result<PreparedLiveMutation> {
    if account.provider_id != "bitget"
        || account.environment != "LIVE"
        || credential_reference != account.credential_ref()
        || intent.workspace_id != account.workspace_id
        || intent.account_id != account.connection_id
        || intent.environment != ExecutionContext::BitgetLive
        || !valid_live_cancel_order_id("bitget", &intent.provider_order_id)
    {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    }
    let order_id = intent
        .provider_order_id
        .strip_prefix("normal:")
        .ok_or_else(|| TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"))?;
    let credential = vault.get(credential_reference)?;
    let secrets = credential.values()?;
    let cancel_provider_status =
        bitget::prepare_live_cancel_preflight(account, intent, &secrets, http)?;
    Ok(PreparedLiveMutation {
        endpoint: ProviderEndpoint::BitgetLive,
        path: "/api/v2/spot/trade/cancel-order".into(),
        method: ProviderHttpMethod::Post,
        body: Some(json!({"symbol":intent.symbol,"orderId":order_id})),
        headers: HeaderMap::new(),
        secrets,
        expected_ticker: None,
        order_id: Some(intent.provider_order_id.clone()),
        cancel_provider_status: Some(cancel_provider_status),
        binance_live_cancel: None,
        bitget_live_cancel: Some((intent.symbol.clone(), order_id.into())),
    })
}

fn binance_live_order_read_error(status: u16, exact_order: bool) -> TradeXError {
    TradeXError::new(match status {
        401 | 403 => "PROVIDER_AUTH_FAILED",
        418 | 429 => "PROVIDER_RATE_LIMITED",
        404 if exact_order => "ORDER_STATUS_UNKNOWN",
        400..=499 => "PROVIDER_RESPONSE_INVALID",
        _ => "PROVIDER_UNAVAILABLE",
    })
}

impl PreparedLiveMutation {
    pub fn send(self, http: &impl ProviderHttp) -> LiveDispatchOutcome {
        let response = if let Some((symbol, order_id)) = self.binance_live_cancel.as_ref() {
            let current = || true;
            let (server_time, sampled) =
                match binance::server_time_for(ProviderEndpoint::BinanceLive, http, &current) {
                    Ok(time) => time,
                    Err(_) => return unknown_live_dispatch(),
                };
            binance::signed_request_for(
                ProviderEndpoint::BinanceLive,
                http,
                ProviderHttpMethod::Delete,
                "/api/v3/order",
                &[("symbol", symbol), ("orderId", order_id)],
                &self.secrets,
                server_time,
                sampled,
                &current,
            )
        } else if let Some((symbol, order_id)) = self.bitget_live_cancel.as_ref() {
            bitget::send_live_cancel(&self.secrets, symbol, order_id, http)
        } else {
            http.request(
                self.endpoint,
                self.method,
                &self.path,
                self.headers,
                self.body.as_ref(),
            )
        };
        let response = match response {
            Ok(response) => response,
            Err(_) => return unknown_live_dispatch(),
        };
        if matches!(response.status, 400 | 401 | 403 | 418 | 429) {
            return LiveDispatchOutcome {
                state: crate::protocol::ExecutionAttemptState::Rejected,
                broker_order_id: self.order_id,
                provider_status: None,
                error_code: Some(
                    match response.status {
                        400 => "PROVIDER_ORDER_REJECTED",
                        401 => "PROVIDER_AUTH_FAILED",
                        403 => "PROVIDER_PERMISSION_BLOCKED",
                        418 => "PROVIDER_RATE_LIMITED",
                        _ => "PROVIDER_RATE_LIMITED",
                    }
                    .into(),
                ),
            };
        }
        if !(200..300).contains(&response.status) {
            return unknown_live_dispatch();
        }
        if let Some((_, order_id)) = self.bitget_live_cancel.as_ref() {
            match bitget::live_cancel_acknowledgement(&response, &self.secrets, order_id) {
                bitget::LiveCancelAcknowledgement::Accepted => (),
                bitget::LiveCancelAcknowledgement::Rejected(code) => {
                    return LiveDispatchOutcome {
                        state: crate::protocol::ExecutionAttemptState::Rejected,
                        broker_order_id: self.order_id,
                        provider_status: None,
                        error_code: Some(code.into()),
                    };
                }
                bitget::LiveCancelAcknowledgement::Unknown => return unknown_live_dispatch(),
            }
        }
        if let Some((symbol, numeric_order_id)) = self.binance_live_cancel.as_ref() {
            let acknowledged = (|| -> Result<()> {
                if response.body.len() as u64 > MAX_RESPONSE {
                    return Err(invalid());
                }
                let value: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
                if contains_secret(&value, &self.secrets)
                    || value["symbol"].as_str() != Some(symbol)
                    || binance::provider_id(&value, "orderId")? != *numeric_order_id
                    || value["orderListId"].as_i64() != Some(-1)
                    || value
                        .get("status")
                        .and_then(Value::as_str)
                        .is_none_or(|status| status.is_empty() || status.len() > 64)
                {
                    return Err(invalid());
                }
                Ok(())
            })();
            if acknowledged.is_err() {
                return unknown_live_dispatch();
            }
        }
        if let Some(order_id) = self.order_id {
            return LiveDispatchOutcome {
                state: crate::protocol::ExecutionAttemptState::CancelPending,
                broker_order_id: Some(order_id),
                provider_status: self.cancel_provider_status,
                error_code: None,
            };
        }
        let acknowledged = (|| -> Result<(String, Option<String>)> {
            let value: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
            if contains_secret(&value, &self.secrets)
                || self.expected_ticker.as_deref() != value.get("ticker").and_then(Value::as_str)
            {
                return Err(invalid());
            }
            let provider_order_id = trading212::order_id(&value)?;
            let provider_status = value
                .get("status")
                .and_then(Value::as_str)
                .filter(|status| {
                    !status.is_empty()
                        && status.len() <= 64
                        && status.bytes().all(|byte| byte.is_ascii_graphic())
                })
                .map(str::to_owned);
            Ok((provider_order_id, provider_status))
        })();
        match acknowledged {
            Ok((broker_order_id, provider_status)) => LiveDispatchOutcome {
                state: crate::protocol::ExecutionAttemptState::Accepted,
                broker_order_id: Some(broker_order_id),
                provider_status,
                error_code: None,
            },
            Err(_) => unknown_live_dispatch(),
        }
    }
}

fn validate_trading212_live_cancel_snapshot(
    order: &Value,
    intent: &CancellationIntent,
) -> Result<String> {
    let order_id = trading212::order_id(order)?;
    let symbol = if order.get("instrument").is_some_and(Value::is_object) {
        text(&order["instrument"], "ticker", 64)?
    } else {
        text(order, "ticker", 64)?
    };
    let side = text(order, "side", 8)?.to_ascii_uppercase();
    let status = text(order, "status", 32)?;
    if !matches!(
        status.as_str(),
        "UNCONFIRMED" | "CONFIRMED" | "NEW" | "PARTIALLY_FILLED"
    ) {
        return Err(TradeXError::new("ORDER_NOT_CANCELLABLE"));
    }
    if text(order, "strategy", 16)? != "QUANTITY" {
        return Err(TradeXError::new("ORDER_NOT_CANCELLABLE"));
    }
    let quantity = decimal_magnitude(&trading212::number(&order["quantity"])?)?;
    let filled_quantity = decimal_magnitude(&trading212::number(&order["filledQuantity"])?)?;
    let remaining = decimal_subtract(&quantity, &filled_quantity)
        .map_err(|_| TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"))?;
    let instrument_id = market::canonical_instrument_id("trading212", &symbol);
    if order_id != intent.provider_order_id
        || symbol != intent.symbol
        || side != intent.side
        || instrument_id.as_deref() != Some(intent.instrument_id.as_str())
        || quantity != intent.quantity
        || filled_quantity != intent.filled_quantity
        || remaining != intent.remaining_quantity
        || status != intent.provider_status
    {
        return Err(TradeXError::new("ORDER_CHANGED_REVIEW_AGAIN"));
    }
    Ok(status)
}

pub(crate) fn decimal_magnitude(value: &str) -> Result<String> {
    let normalized = decimal(&Value::String(value.into()))?;
    Ok(normalized.strip_prefix('-').unwrap_or(&normalized).into())
}

fn parse_t212_recent_orders(history: Value) -> Result<Vec<OpenOrder>> {
    let items = history["items"].as_array().ok_or_else(invalid)?;
    if items.len() > 50 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    match history.get("nextPagePath") {
        Some(Value::Null) => (),
        Some(Value::String(path)) if valid_t212_history_path(path) => (),
        _ => return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE")),
    }
    t212_history_unique_orders(items)?
        .iter()
        .map(|order| trading212::open_order(order))
        .collect()
}

fn t212_history_order(item: &Value) -> Result<&Value> {
    item.get("order")
        .filter(|value| value.is_object())
        .ok_or_else(invalid)
}

fn t212_history_unique_orders(items: &[Value]) -> Result<Vec<&Value>> {
    let mut indexes = HashMap::new();
    let mut orders = Vec::with_capacity(items.len());
    for item in items {
        let order = t212_history_order(item)?;
        let id = trading212::order_id(order)?;
        if let Some(index) = indexes.get(&id).copied() {
            if orders[index] != order {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            continue;
        }
        indexes.insert(id, orders.len());
        orders.push(order);
    }
    Ok(orders)
}

fn read_trading212_live_order(
    order_id: &str,
    auth: &HeaderMap,
    secrets: &[String],
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<(Value, &'static str)> {
    if !valid_t212_order_id(order_id) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let path = format!("/api/v0/equity/orders/{order_id}");
    if !current() {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let detail = http.request(
        ProviderEndpoint::Trading212Live,
        ProviderHttpMethod::Get,
        &path,
        auth.clone(),
        None,
    )?;
    let (body, source) = match detail.status {
        200 => (detail.body, "trading212.live.order-detail"),
        404 => {
            let history = http.request(
                ProviderEndpoint::Trading212Live,
                ProviderHttpMethod::Get,
                "/api/v0/equity/history/orders?limit=50",
                auth.clone(),
                None,
            )?;
            if history.status != 200 {
                return Err(trading212_order_read_error(
                    history.status,
                    Trading212Endpoint::History,
                ));
            }
            if history.body.len() as u64 > MAX_RESPONSE {
                return Err(invalid());
            }
            let page: Value = serde_json::from_slice(&history.body).map_err(|_| invalid())?;
            if contains_secret(&page, secrets) {
                return Err(invalid());
            }
            let items = page["items"].as_array().ok_or_else(invalid)?;
            if items.len() > 50 {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            let mut exact = t212_history_unique_orders(items)?
                .into_iter()
                .filter(|item| trading212::order_id(item).is_ok_and(|id| id == order_id));
            let Some(order) = exact.next() else {
                return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
            };
            if exact.next().is_some() {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
            }
            let bytes = serde_json::to_vec(order).map_err(|_| invalid())?;
            (bytes, "trading212.live.order-history")
        }
        status => {
            return Err(trading212_order_read_error(
                status,
                Trading212Endpoint::OrderDetail,
            ));
        }
    };
    if body.len() as u64 > MAX_RESPONSE {
        return Err(invalid());
    }
    let order: Value = serde_json::from_slice(&body).map_err(|_| invalid())?;
    if contains_secret(&order, secrets) || trading212::order_id(&order)? != order_id {
        return Err(TradeXError::new("PROVIDER_IDENTITY_CONFLICT"));
    }
    Ok((order, source))
}

fn unknown_live_dispatch() -> LiveDispatchOutcome {
    LiveDispatchOutcome {
        state: crate::protocol::ExecutionAttemptState::UnknownReconciling,
        broker_order_id: None,
        provider_status: None,
        error_code: Some("ORDER_STATUS_UNKNOWN".into()),
    }
}

fn body_ticker_for_live_proposal(proposal: &OrderProposal) -> Result<String> {
    market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == proposal.fields.instrument_id)
        .and_then(|instrument| {
            instrument
                .providers
                .into_iter()
                .find(|mapping| mapping.provider_id == "trading212")
                .map(|mapping| mapping.provider_symbol)
        })
        .ok_or_else(|| TradeXError::new("ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"))
}

pub struct BrokerHttp {
    client: OnceLock<Result<Client>>,
    #[cfg(feature = "integration-test")]
    local_test_base_url: Option<String>,
}

impl BrokerHttp {
    fn client_builder(https_only: bool) -> reqwest::blocking::ClientBuilder {
        Client::builder()
            .https_only(https_only)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(12))
            .connect_timeout(Duration::from_secs(4))
    }

    fn client(&self) -> Result<&Client> {
        self.client
            .get_or_init(|| {
                Self::client_builder(self.local_test_base_url().is_none())
                    .build()
                    .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    #[cfg(feature = "integration-test")]
    pub fn for_loopback_test(base_url: &str) -> Result<Self> {
        let port = base_url
            .strip_prefix("http://127.0.0.1:")
            .and_then(|value| value.parse::<u16>().ok());
        if !port.is_some_and(|port| port > 0) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        Ok(Self {
            client: OnceLock::new(),
            local_test_base_url: Some(base_url.trim_end_matches('/').to_owned()),
        })
    }

    fn local_test_base_url(&self) -> Option<&str> {
        #[cfg(feature = "integration-test")]
        {
            self.local_test_base_url.as_deref()
        }
        #[cfg(not(feature = "integration-test"))]
        {
            None
        }
    }

    fn url(&self, endpoint: ProviderEndpoint, path: &str) -> String {
        let base = if (endpoint == ProviderEndpoint::AlpacaPaper
            && crate::calendar_source::allowed_path(path))
            || matches!(
                endpoint,
                ProviderEndpoint::Trading212Live
                    | ProviderEndpoint::AlpacaMarketData
                    | ProviderEndpoint::BinanceLive
                    | ProviderEndpoint::BitgetLive
            ) {
            self.local_test_base_url()
                .unwrap_or_else(|| endpoint.base_url())
        } else {
            endpoint.base_url()
        };
        format!("{base}{path}")
    }
}

impl Default for BrokerHttp {
    fn default() -> Self {
        Self {
            client: OnceLock::new(),
            #[cfg(feature = "integration-test")]
            local_test_base_url: None,
        }
    }
}

#[cfg(test)]
#[path = "../tests/support/provider_http.rs"]
mod http_tests;

impl ProviderHttp for BrokerHttp {
    fn get_response_with_rate_limit(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: HeaderMap,
    ) -> Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
        self.request_with_rate_limit(endpoint, ProviderHttpMethod::Get, path, headers, None)
    }

    fn get(&self, endpoint: ProviderEndpoint, path: &str, headers: HeaderMap) -> Result<Vec<u8>> {
        self.get_response_with_rate_limit(endpoint, path, headers)
            .and_then(|(response, _)| {
                classify_get_response(endpoint, response.status, response.body)
            })
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
        if endpoint == ProviderEndpoint::BinanceTestnet {
            binance::check_testnet_ip_cooldown()?;
        }
        if !endpoint.allows_method(method, path) {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        if endpoint == ProviderEndpoint::BitgetDemo
            && method == ProviderHttpMethod::Post
            && headers
                .get("paptrading")
                .and_then(|value| value.to_str().ok())
                != Some("1")
        {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let request = match method {
            ProviderHttpMethod::Get if body.is_none() => self
                .client()?
                .get(self.url(endpoint, path))
                .headers(headers),
            ProviderHttpMethod::Post if body.is_some() => {
                let body = serde_json::to_vec(body.unwrap()).map_err(|_| invalid())?;
                if body.len() > 16 * 1024 {
                    return Err(invalid());
                }
                self.client()?
                    .post(self.url(endpoint, path))
                    .headers(headers)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body)
            }
            ProviderHttpMethod::Post
                if endpoint == ProviderEndpoint::BinanceTestnet && body.is_none() =>
            {
                self.client()?
                    .post(self.url(endpoint, path))
                    .headers(headers)
            }
            ProviderHttpMethod::Delete if body.is_none() => self
                .client()?
                .delete(self.url(endpoint, path))
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
        if endpoint == ProviderEndpoint::BinanceTestnet {
            binance::observe_ip_rate_limit(status, rate_limit.as_ref());
        }
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

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum JobKind {
    Connect,
    Probe,
    Disconnect,
    Trading212DemoSubmit,
    Trading212DemoOrderBookPending,
    Trading212DemoOrderBookHistory,
    Trading212DemoOrderBookDetail,
    Trading212DemoOrderCancel,
    AlpacaPaperSubmit,
    AlpacaPaperReconcile,
    AlpacaPaperOrderBookRefresh,
    AlpacaPaperOrderReview,
    AlpacaPaperOrderCancel,
    BinanceTestnetSubmit,
    BinanceTestnetReconcile,
    BinanceTestnetOrderBookRefresh,
    BinanceTestnetOrderCancel,
    BitgetDemoSubmit,
    BitgetDemoReconcile,
    CancellationIntentRefresh(Box<crate::protocol::CancellationIntentRequest>),
    LiveOrderRefresh {
        order_id: String,
        attempt_id: String,
    },
    LiveOrderReconcile {
        input: Box<ResolutionEvidenceRefresh>,
        attempt: Box<ExecutionAttempt>,
        proposal: Box<OrderProposal>,
        ledger: Option<Box<ResolutionEvidenceLedger>>,
        queried_at: String,
        automatic_window_started_at: String,
        automatic_window_ends_at: String,
    },
}

impl JobKind {
    fn provider_priority(&self) -> ProviderPriority {
        match self {
            Self::Connect | Self::Probe | Self::AlpacaPaperOrderReview => ProviderPriority::P1,
            Self::Trading212DemoOrderBookPending
            | Self::AlpacaPaperOrderBookRefresh
            | Self::BinanceTestnetOrderBookRefresh => ProviderPriority::P2,
            Self::Trading212DemoOrderBookHistory | Self::Trading212DemoOrderBookDetail => {
                ProviderPriority::P3
            }
            _ => ProviderPriority::P0,
        }
    }
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
    pub(crate) binance_testnet_attempt: Option<BinanceTestnetOrderAttempt>,
    pub(crate) binance_testnet_proposal: Option<OrderProposal>,
    pub(crate) binance_testnet_order_book: Option<BinanceTestnetOrderBook>,
    pub(crate) binance_testnet_book_action: Option<BinanceTestnetOrderBookAction>,
    pub(crate) binance_testnet_book_symbol: Option<String>,
    pub(crate) binance_testnet_book_order_id: Option<String>,
    pub(crate) bitget_demo_attempt: Option<BitgetDemoOrderAttempt>,
    pub(crate) bitget_demo_proposal: Option<OrderProposal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LiveOrderObservation {
    pub provider_order_id: String,
    pub raw_status: String,
    pub disposition: crate::protocol::LiveOrderDisposition,
    pub order_quantity: Option<String>,
    pub filled_quantity: Option<String>,
    pub remaining_quantity: Option<String>,
    pub filled_value: Option<String>,
    pub fees: Option<Vec<crate::protocol::LiveOrderFee>>,
    pub trade_facts_complete: bool,
    pub trade_facts: Vec<crate::protocol::LiveOrderTradeFact>,
    pub provider_observed_at: Option<String>,
    pub source: String,
}

pub(crate) struct Observation {
    pub data: AccountData,
    pub permissions: PermissionReview,
    pub live_order_settlements: Vec<LiveOrderObservation>,
}

fn normalize_account_data(data: &mut AccountData, provider_id: &str) {
    for position in &mut data.positions {
        position.instrument_id = market::canonical_instrument_id(provider_id, &position.symbol);
    }
    for order in &mut data.open_orders {
        order.instrument_id = market::canonical_instrument_id(provider_id, &order.symbol);
    }
    for order in &mut data.recent_orders {
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
    pub(crate) binance_testnet_attempt: Option<BinanceTestnetOrderAttempt>,
    pub(crate) binance_testnet_order_book: Option<BinanceTestnetOrderBook>,
    pub(crate) bitget_demo_attempt: Option<BitgetDemoOrderAttempt>,
    pub(crate) resolution_evidence: Option<ResolutionEvidence>,
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
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            };
        }
        let http = prioritized_provider_http(
            http,
            &current,
            &self.account.connection_id,
            self.kind.provider_priority(),
        );
        if matches!(&self.kind, JobKind::LiveOrderReconcile { .. }) {
            return match self.account.provider_id.as_str() {
                "binance" => self.run_binance_live_reconciliation(vault, &http, &current),
                "bitget" => self.run_bitget_live_reconciliation(vault, &http, &current),
                _ => self.run_trading212_live_reconciliation(vault, &http, &current),
            };
        }
        if matches!(
            &self.kind,
            JobKind::AlpacaPaperSubmit | JobKind::AlpacaPaperReconcile
        ) {
            return self.run_alpaca_paper_order(vault, &http, &current);
        }
        if matches!(
            &self.kind,
            JobKind::BinanceTestnetSubmit | JobKind::BinanceTestnetReconcile
        ) {
            return self.run_binance_testnet_order(vault, &http, &current);
        }
        if matches!(
            &self.kind,
            JobKind::BitgetDemoSubmit | JobKind::BitgetDemoReconcile
        ) {
            return self.run_bitget_demo_order(vault, &http, &current);
        }
        if self.kind == JobKind::BinanceTestnetOrderBookRefresh {
            return self.run_binance_testnet_order_book(vault, &http, &current);
        }
        if self.kind == JobKind::BinanceTestnetOrderCancel {
            return self.run_binance_testnet_order_book(vault, &http, &current);
        }
        if self.kind == JobKind::Trading212DemoSubmit {
            return self.run_trading212_demo_order(vault, &http, &current);
        }
        if matches!(
            &self.kind,
            JobKind::Trading212DemoOrderBookPending
                | JobKind::Trading212DemoOrderBookHistory
                | JobKind::Trading212DemoOrderBookDetail
                | JobKind::Trading212DemoOrderCancel
        ) {
            return self.run_trading212_demo_order_operation(vault, &http, &current);
        }
        if matches!(
            &self.kind,
            JobKind::AlpacaPaperOrderBookRefresh
                | JobKind::AlpacaPaperOrderReview
                | JobKind::AlpacaPaperOrderCancel
        ) {
            return self.run_alpaca_paper_order_book(vault, &http, &current);
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
                let exact_cancel_order_id = match &self.kind {
                    JobKind::CancellationIntentRefresh(input)
                        if endpoint == ProviderEndpoint::BitgetLive =>
                    {
                        Some(input.broker_order_id.as_str())
                    }
                    JobKind::LiveOrderRefresh { order_id, .. }
                        if endpoint == ProviderEndpoint::BitgetLive =>
                    {
                        Some(order_id.as_str())
                    }
                    _ => None,
                };
                let mut observation = bitget::read(
                    endpoint,
                    &values,
                    &http,
                    &current,
                    self.account.data.as_ref(),
                    exact_cancel_order_id,
                )?;
                normalize_account_data(&mut observation.data, &self.account.provider_id);
                return Ok(observation);
            }
            if endpoint.is_binance() {
                let exact_order = match &self.kind {
                    JobKind::CancellationIntentRefresh(input)
                        if endpoint == ProviderEndpoint::BinanceLive =>
                    {
                        Some((input.broker_order_id.as_str(), false))
                    }
                    JobKind::LiveOrderRefresh { order_id, .. }
                        if endpoint == ProviderEndpoint::BinanceLive =>
                    {
                        Some((order_id.as_str(), true))
                    }
                    _ => None,
                };
                let mut observation = binance::read(
                    endpoint,
                    &values,
                    &http,
                    &current,
                    self.account.data.as_ref(),
                    exact_order,
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
                let mut observation = trading212::observe(
                    account,
                    query("/api/v0/equity/positions")?,
                    query("/api/v0/equity/orders")?,
                    endpoint == ProviderEndpoint::Trading212Live,
                )?;
                if endpoint == ProviderEndpoint::Trading212Live {
                    if matches!(&self.kind, JobKind::Connect | JobKind::Probe) {
                        observation.data.recent_orders = parse_t212_recent_orders(query(
                            "/api/v0/equity/history/orders?limit=50",
                        )?)?;
                    } else {
                        observation.data.recent_orders = self
                            .account
                            .data
                            .as_ref()
                            .map(|data| data.recent_orders.clone())
                            .unwrap_or_default();
                    }
                }
                #[cfg(feature = "integration-test")]
                if endpoint == ProviderEndpoint::Trading212Live
                    && std::env::var_os("TRADEX_S26_2_CANCEL_FIXTURE").is_some()
                    && self
                        .account
                        .label
                        .starts_with("SYNTHETIC · S26.2 T212 Live")
                    && self.account.permissions.scope == "VERIFIED"
                {
                    observation.permissions.scope = "VERIFIED".into();
                }
                let target_order_id = match &self.kind {
                    JobKind::CancellationIntentRefresh(input) => {
                        Some(input.broker_order_id.as_str())
                    }
                    JobKind::LiveOrderRefresh { order_id, .. } => Some(order_id.as_str()),
                    _ => None,
                };
                if endpoint == ProviderEndpoint::Trading212Live
                    && let Some(order_id) = target_order_id
                {
                    let (order, source) =
                        read_trading212_live_order(order_id, &auth, &values, &http, &current)?;
                    let exact = trading212::open_order(&order)?;
                    let terminal = matches!(
                        exact.status.as_str(),
                        "CANCELLED" | "FILLED" | "REJECTED" | "REPLACED" | "EXPIRED"
                    );
                    observation
                        .data
                        .open_orders
                        .retain(|saved| saved.broker_order_id != order_id);
                    if !terminal {
                        observation.data.open_orders.push(exact);
                    }
                    observation
                        .live_order_settlements
                        .retain(|saved| saved.provider_order_id != order_id);
                    observation
                        .live_order_settlements
                        .push(trading212::live_order_observation(&order, source)?);
                }
                observation
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
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            },
            Err(error) => ProviderOutcome {
                observation: None,
                error: Some(error),
                credential: credential.into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            },
        }
    }

    fn run_binance_live_reconciliation(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let JobKind::LiveOrderReconcile {
            input,
            attempt,
            proposal,
            queried_at,
            automatic_window_started_at,
            automatic_window_ends_at,
            ..
        } = &self.kind
        else {
            unreachable!();
        };
        let mut credential = "MISSING";
        let empty_outcome = |error: TradeXError| ProviderOutcome {
            observation: None,
            error: Some(error),
            credential: credential.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
        };
        if !current() {
            return empty_outcome(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let expected_client_order_id = binance::live_client_order_id(&attempt.attempt_id);
        let provider_symbol = binance::provider_symbol(
            proposal,
            &self.account.connection_id,
            ExecutionContext::BinanceLive,
        );
        let query_scope = match (&provider_symbol, &attempt.provider_client_order_id) {
            (Ok(symbol), Some(client_order_id)) => format!(
                "GET /api/v3/time; GET /api/v3/account; GET /api/v3/order?symbol={symbol}&origClientOrderId={client_order_id}"
            ),
            _ => "GET /api/v3/time; GET /api/v3/account; GET /api/v3/order using the saved Binance symbol and client-order identity".into(),
        };
        let result = (|| -> Result<Option<crate::protocol::ProviderOrderCandidate>> {
            if self.account.provider_id != "binance"
                || self.account.environment != "LIVE"
                || self.account.connection_state != ConnectionState::Connected
                || attempt.state != ExecutionAttemptState::UnknownReconciling
                || attempt.operation != crate::protocol::FinancialOperation::PlaceOrder
                || attempt.environment != ExecutionContext::BinanceLive
                || attempt.account_id != self.account.connection_id
                || attempt.attempt_id != input.execution_attempt_id
                || attempt.workspace_id != input.workspace_id
                || attempt.state_version != input.expected_attempt_state_version
                || input.account_id != self.account.connection_id
                || expected_client_order_id
                    .as_ref()
                    .ok()
                    .zip(attempt.provider_client_order_id.as_ref())
                    .is_none_or(|(expected, saved)| expected != saved)
                || provider_symbol.is_err()
                || proposal.workspace_id != attempt.workspace_id
                || proposal.proposal_id != attempt.intent_id
                || proposal.proposal_id.as_str()
                    != attempt.proposal_id.as_deref().unwrap_or_default()
                || proposal.proposal_hash != attempt.intent_hash
                || proposal.fields.account_id.as_deref() != Some(attempt.account_id.as_str())
                || proposal.fields.environment != ExecutionContext::BinanceLive
                || !current()
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let remote_account_id = self
                .account
                .data
                .as_ref()
                .map(|data| data.remote_account_id.as_str())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
            let secrets = vault.get(&self.account.credential_ref())?;
            credential = "CONFIGURED";
            let values = secrets.values()?;
            if values.len() != 2 {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            let candidate = binance::query_live_reconciliation_candidate(
                proposal,
                &attempt.attempt_id,
                attempt
                    .provider_client_order_id
                    .as_deref()
                    .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?,
                &self.account.connection_id,
                remote_account_id,
                automatic_window_started_at,
                automatic_window_ends_at,
                &values,
                http,
                &current,
            )?;
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            Ok(candidate)
        })();
        if result
            .as_ref()
            .is_err_and(|error| error.code == "PROVIDER_AUTH_FAILED")
        {
            credential = "INVALID";
        }
        let (candidate_orders, error_code) = match result {
            Ok(candidate) => (candidate.into_iter().collect::<Vec<_>>(), None),
            Err(error) if error.code == "STATE_VERSION_CONFLICT" => return empty_outcome(error),
            Err(error) => (Vec::new(), Some(error.code)),
        };
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: Some(ResolutionEvidence {
                evidence_id: uuid::Uuid::new_v4().to_string(),
                execution_attempt_id: attempt.attempt_id.clone(),
                account_id: self.account.connection_id.clone(),
                provider_id: "binance".into(),
                account_observation_version: None,
                queried_at: queried_at.clone(),
                query_scope,
                coverage_from: Some(automatic_window_started_at.clone()),
                coverage_to: Some(queried_at.clone()),
                outcome: if candidate_orders.is_empty() {
                    ResolutionEvidenceOutcome::Inconclusive
                } else {
                    ResolutionEvidenceOutcome::CandidatesFound
                },
                pagination_complete: error_code.is_none(),
                candidate_orders,
                next_page_path: None,
                error_code,
            }),
        }
    }

    fn run_bitget_live_reconciliation(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let JobKind::LiveOrderReconcile {
            input,
            attempt,
            proposal,
            queried_at,
            automatic_window_started_at,
            automatic_window_ends_at,
            ..
        } = &self.kind
        else {
            unreachable!();
        };
        let mut credential = "MISSING";
        let empty_outcome = |error: TradeXError| ProviderOutcome {
            observation: None,
            error: Some(error),
            credential: credential.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
        };
        if !current() {
            return empty_outcome(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let expected_client_order_id = bitget::live_client_order_id(&attempt.attempt_id);
        let query_scope = attempt.provider_client_order_id.as_ref().map_or_else(
            || "GET /api/v2/public/time; GET /api/v2/spot/account/info; GET /api/v2/spot/trade/orderInfo using the saved Bitget clientOid".into(),
            |client_oid| format!(
                "GET /api/v2/public/time; GET /api/v2/spot/account/info; GET /api/v2/spot/trade/orderInfo?clientOid={client_oid}"
            ),
        );
        let result = (|| -> Result<Option<crate::protocol::ProviderOrderCandidate>> {
            if self.account.provider_id != "bitget"
                || self.account.environment != "LIVE"
                || self.account.connection_state != ConnectionState::Connected
                || attempt.state != ExecutionAttemptState::UnknownReconciling
                || attempt.operation != crate::protocol::FinancialOperation::PlaceOrder
                || attempt.environment != ExecutionContext::BitgetLive
                || attempt.account_id != self.account.connection_id
                || attempt.attempt_id != input.execution_attempt_id
                || attempt.workspace_id != input.workspace_id
                || attempt.state_version != input.expected_attempt_state_version
                || input.account_id != self.account.connection_id
                || expected_client_order_id
                    .as_ref()
                    .ok()
                    .zip(attempt.provider_client_order_id.as_ref())
                    .is_none_or(|(expected, saved)| expected != saved)
                || proposal.workspace_id != attempt.workspace_id
                || proposal.proposal_id != attempt.intent_id
                || proposal.proposal_id.as_str()
                    != attempt.proposal_id.as_deref().unwrap_or_default()
                || proposal.proposal_hash != attempt.intent_hash
                || proposal.fields.account_id.as_deref() != Some(attempt.account_id.as_str())
                || proposal.fields.environment != ExecutionContext::BitgetLive
                || !current()
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let remote_account_id = self
                .account
                .data
                .as_ref()
                .map(|data| data.remote_account_id.as_str())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?;
            let secrets = vault.get(&self.account.credential_ref())?;
            credential = "CONFIGURED";
            let values = secrets.values()?;
            if values.len() != 3 {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            let candidate = bitget::query_live_reconciliation_candidate(
                proposal,
                &attempt.attempt_id,
                attempt
                    .provider_client_order_id
                    .as_deref()
                    .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?,
                &self.account.connection_id,
                remote_account_id,
                automatic_window_started_at,
                automatic_window_ends_at,
                &values,
                http,
                &current,
            )?;
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            Ok(candidate)
        })();
        if result
            .as_ref()
            .is_err_and(|error| error.code == "PROVIDER_AUTH_FAILED")
        {
            credential = "INVALID";
        }
        let (candidate_orders, error_code) = match result {
            Ok(candidate) => (candidate.into_iter().collect::<Vec<_>>(), None),
            Err(error) if error.code == "STATE_VERSION_CONFLICT" => return empty_outcome(error),
            Err(error) => (Vec::new(), Some(error.code)),
        };
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: Some(ResolutionEvidence {
                evidence_id: uuid::Uuid::new_v4().to_string(),
                execution_attempt_id: attempt.attempt_id.clone(),
                account_id: self.account.connection_id.clone(),
                provider_id: "bitget".into(),
                account_observation_version: None,
                queried_at: queried_at.clone(),
                query_scope,
                coverage_from: Some(automatic_window_started_at.clone()),
                coverage_to: Some(queried_at.clone()),
                outcome: if candidate_orders.is_empty() {
                    ResolutionEvidenceOutcome::Inconclusive
                } else {
                    ResolutionEvidenceOutcome::CandidatesFound
                },
                pagination_complete: error_code.is_none(),
                candidate_orders,
                next_page_path: None,
                error_code,
            }),
        }
    }

    fn run_trading212_live_reconciliation(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let JobKind::LiveOrderReconcile {
            input,
            attempt,
            proposal,
            ledger,
            queried_at,
            automatic_window_started_at,
            automatic_window_ends_at,
        } = &self.kind
        else {
            unreachable!();
        };
        let mut credential = "MISSING";
        let empty_outcome = |error: TradeXError| ProviderOutcome {
            observation: None,
            error: Some(error),
            credential: credential.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
        };
        if !current() {
            return empty_outcome(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let (_, submit_body) =
            match trading212_live_order_request(proposal, &self.account.connection_id) {
                Ok(value) => value,
                Err(error) => return empty_outcome(error),
            };
        let Some(ticker) = submit_body["ticker"].as_str() else {
            return empty_outcome(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        };
        let history_path = ledger
            .as_deref()
            .and_then(|ledger| ledger.next_page_path.clone())
            .unwrap_or_else(|| format!("/api/v0/equity/history/orders?limit=50&ticker={ticker}"));
        let query_scope = format!(
            "GET /api/v0/equity/account/summary; GET /api/v0/equity/orders; GET {history_path}"
        );
        let result = (|| -> Result<(Vec<ProviderOrderCandidate>, Option<String>, bool)> {
            if self.account.provider_id != "trading212"
                || self.account.environment != "LIVE"
                || self.account.connection_state != ConnectionState::Connected
                || attempt.state != ExecutionAttemptState::UnknownReconciling
                || attempt.operation != crate::protocol::FinancialOperation::PlaceOrder
                || attempt.environment != ExecutionContext::Trading212Live
                || attempt.account_id != self.account.connection_id
                || attempt.attempt_id != input.execution_attempt_id
                || attempt.workspace_id != input.workspace_id
                || attempt.state_version != input.expected_attempt_state_version
                || input.account_id != self.account.connection_id
                || proposal.workspace_id != attempt.workspace_id
                || proposal.proposal_id != attempt.intent_id
                || proposal.proposal_id.as_str()
                    != attempt.proposal_id.as_deref().unwrap_or_default()
                || proposal.proposal_hash != attempt.intent_hash
                || proposal.fields.account_id.as_deref() != Some(attempt.account_id.as_str())
                || proposal.fields.environment != ExecutionContext::Trading212Live
                || !valid_t212_history_path(&history_path)
                || !current()
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            if ledger.as_deref().is_some_and(|ledger| {
                ledger.history_pages_read >= 100 && ledger.next_page_path.is_some()
            }) {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            credential = "CONFIGURED";
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
            let query = |path: &str| -> Result<Value> {
                if !current() {
                    return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                }
                let response = http.request(
                    ProviderEndpoint::Trading212Live,
                    ProviderHttpMethod::Get,
                    path,
                    auth.clone(),
                    None,
                )?;
                if response.status != 200 {
                    return Err(trading212_order_read_error(
                        response.status,
                        Trading212Endpoint::History,
                    ));
                }
                if response.body.len() as u64 > MAX_RESPONSE {
                    return Err(invalid());
                }
                let value: Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
                if contains_secret(&value, &values) {
                    return Err(invalid());
                }
                Ok(value)
            };
            let account = query("/api/v0/equity/account/summary")?;
            let remote_account_id = trading212::account_id(&account)?;
            if self
                .account
                .data
                .as_ref()
                .is_none_or(|data| data.remote_account_id != remote_account_id)
            {
                return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
            }
            let pending = query("/api/v0/equity/orders")?;
            let pending = pending.as_array().ok_or_else(invalid)?;
            if pending.len() > 500 {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            let mut rows = Vec::with_capacity(pending.len() + 50);
            for row in pending {
                rows.push(parse_trading212_order(row, queried_at, true)?);
            }
            let history = query(&history_path)?;
            let items = history["items"].as_array().ok_or_else(invalid)?;
            if items.len() > 50 {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            for row in t212_history_unique_orders(items)? {
                rows.push(parse_trading212_order(row, queried_at, false)?);
            }
            let next_page_path = match history.get("nextPagePath") {
                Some(Value::Null) => None,
                Some(Value::String(path)) if valid_t212_history_path(path) => Some(path.clone()),
                _ => return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE")),
            };
            if next_page_path.as_deref().is_some_and(|next| {
                next == history_path
                    || ledger.as_deref().is_some_and(|ledger| {
                        ledger
                            .evidence
                            .iter()
                            .any(|evidence| evidence.query_scope.ends_with(&format!("GET {next}")))
                    })
            }) {
                return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
            }
            let expected_side = match proposal.fields.side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            };
            let window_started = ::time::OffsetDateTime::parse(
                automatic_window_started_at,
                &::time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            let window_ends = ::time::OffsetDateTime::parse(
                automatic_window_ends_at,
                &::time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            let proposal_quantity = proposal.fields.quantity.value.as_str();
            let mut seen = std::collections::HashSet::new();
            let candidates = rows
                .into_iter()
                .filter(|order| order.symbol == ticker && order.side == expected_side)
                .filter(|order| {
                    order.quantity.as_deref().is_some_and(|quantity| {
                        decimal_cmp(quantity, proposal_quantity)
                            .is_ok_and(|comparison| comparison == std::cmp::Ordering::Equal)
                    })
                })
                .filter(|order| {
                    ::time::OffsetDateTime::parse(
                        &order.submitted_at,
                        &::time::format_description::well_known::Rfc3339,
                    )
                    .is_ok_and(|submitted_at| {
                        submitted_at >= window_started && submitted_at <= window_ends
                    })
                })
                .filter(|order| seen.insert(order.provider_order_id.clone()))
                .map(|order| ProviderOrderCandidate {
                    provider_order_id: order.provider_order_id,
                    provider_symbol: order.symbol,
                    side: proposal.fields.side,
                    provider_status: order.provider_status,
                    order_type: order.order_type,
                    quantity: order.quantity,
                    quote_quantity: None,
                    limit_price: None,
                    time_in_force: None,
                    force: None,
                    tpsl_type: None,
                    submitted_at: Some(order.submitted_at),
                    provider_client_id: None,
                })
                .collect::<Vec<_>>();
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            Ok((candidates, next_page_path.clone(), next_page_path.is_none()))
        })();
        if result
            .as_ref()
            .is_err_and(|error| error.code == "PROVIDER_AUTH_FAILED")
        {
            credential = "INVALID";
        }
        let evidence = match result {
            Ok((candidate_orders, next_page_path, pagination_complete)) => ResolutionEvidence {
                evidence_id: uuid::Uuid::new_v4().to_string(),
                execution_attempt_id: attempt.attempt_id.clone(),
                account_id: self.account.connection_id.clone(),
                provider_id: "trading212".into(),
                account_observation_version: None,
                queried_at: queried_at.clone(),
                query_scope,
                coverage_from: Some(automatic_window_started_at.clone()),
                coverage_to: Some(queried_at.clone()),
                outcome: if candidate_orders.is_empty() {
                    ResolutionEvidenceOutcome::Inconclusive
                } else {
                    ResolutionEvidenceOutcome::CandidatesFound
                },
                pagination_complete,
                candidate_orders,
                next_page_path,
                error_code: None,
            },
            Err(error) if error.code == "STATE_VERSION_CONFLICT" => {
                return empty_outcome(error);
            }
            Err(error) => ResolutionEvidence {
                evidence_id: uuid::Uuid::new_v4().to_string(),
                execution_attempt_id: attempt.attempt_id.clone(),
                account_id: self.account.connection_id.clone(),
                provider_id: "trading212".into(),
                account_observation_version: None,
                queried_at: queried_at.clone(),
                query_scope,
                coverage_from: Some(automatic_window_started_at.clone()),
                coverage_to: Some(queried_at.clone()),
                outcome: ResolutionEvidenceOutcome::Inconclusive,
                pagination_complete: false,
                candidate_orders: Vec::new(),
                next_page_path: None,
                error_code: Some(error.code),
            },
        };
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: Some(evidence),
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
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
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
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
        }
    }

    fn run_trading212_demo_order_operation(
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
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            };
        };
        let mut credential_state = "MISSING";
        let mut cancel_may_have_been_sent = false;
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
                    &self.kind,
                    JobKind::Trading212DemoOrderBookPending
                        | JobKind::Trading212DemoOrderBookHistory
                        | JobKind::Trading212DemoOrderBookDetail
                        | JobKind::Trading212DemoOrderCancel
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

            if self.kind == JobKind::Trading212DemoOrderCancel {
                self.cancel_trading212_order(
                    http,
                    &auth,
                    &values,
                    &mut book,
                    &current,
                    &mut cancel_may_have_been_sent,
                )?;
                return Ok(());
            }

            let now = crate::storage::timestamp()?;
            let mut candidate = book.clone();
            let endpoint = match &self.kind {
                JobKind::Trading212DemoOrderBookPending => Trading212Endpoint::PendingOrders,
                JobKind::Trading212DemoOrderBookHistory => Trading212Endpoint::History,
                JobKind::Trading212DemoOrderBookDetail => Trading212Endpoint::OrderDetail,
                _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            };
            let path = match &self.kind {
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
            reserve_trading212_endpoint(&mut book, endpoint)?;
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
            match &self.kind {
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
                            | Trading212DemoNormalizedOrderStatus::Expired
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
                        let order = parse_trading212_order(t212_history_order(row)?, &now, false)?;
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
            if self.kind == JobKind::Trading212DemoOrderCancel
                && let Some(order_id) = self.trading212_demo_order_id.as_deref()
                && let Some(order) = book
                    .orders
                    .iter_mut()
                    .find(|order| order.provider_order_id == order_id)
                && order.cancel_state == Trading212DemoCancelState::Submitting
            {
                if cancel_may_have_been_sent {
                    order.cancel_state = Trading212DemoCancelState::Pending;
                    order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                } else {
                    order.cancel_state = Trading212DemoCancelState::None;
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
            trading212_demo_order_book: Some(book),
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
        }
    }

    fn cancel_trading212_order(
        &self,
        http: &impl ProviderHttp,
        auth: &HeaderMap,
        secrets: &[String],
        book: &mut Trading212DemoOrderBook,
        current: &impl Fn() -> bool,
        cancel_may_have_been_sent: &mut bool,
    ) -> Result<()> {
        let order_id = self
            .trading212_demo_order_id
            .as_deref()
            .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
        if !valid_t212_order_id(order_id) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let index = book
            .orders
            .iter()
            .position(|order| order.provider_order_id == order_id)
            .ok_or_else(|| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let order = &book.orders[index];
        if order.cancel_state != Trading212DemoCancelState::Submitting {
            return Err(TradeXError::new("ORDER_STATUS_UNKNOWN"));
        }
        if book.status != Trading212DemoOrderBookStatus::Current
            || !order.pending
            || !matches!(
                order.provider_status.as_str(),
                "CONFIRMED" | "NEW" | "PARTIALLY_FILLED"
            )
        {
            return Err(TradeXError::new("ORDER_NOT_CANCELABLE"));
        }
        let observed_at = time::OffsetDateTime::parse(
            &order.observed_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let age = time::OffsetDateTime::now_utc() - observed_at;
        if age.is_negative() || age > time::Duration::seconds(60) {
            return Err(TradeXError::new("ORDER_CONFIRMATION_EXPIRED"));
        }
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }

        let identity = http.request(
            ProviderEndpoint::Trading212Demo,
            ProviderHttpMethod::Get,
            "/api/v0/equity/account/summary",
            auth.clone(),
            None,
        )?;
        if identity.status != 200 {
            return Err(trading212_order_read_error(
                identity.status,
                Trading212Endpoint::OrderDetail,
            ));
        }
        let identity: Value = serde_json::from_slice(&identity.body).map_err(|_| invalid())?;
        if contains_secret(&identity, secrets)
            || trading212::account_id(&identity)? != book.remote_account_id
        {
            return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
        }
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let observed_at = time::OffsetDateTime::parse(
            &book.orders[index].observed_at,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| TradeXError::new("ORDER_STATUS_UNKNOWN"))?;
        let age = time::OffsetDateTime::now_utc() - observed_at;
        if age.is_negative() || age > time::Duration::seconds(60) {
            return Err(TradeXError::new("ORDER_CONFIRMATION_EXPIRED"));
        }
        let endpoint = Trading212Endpoint::CancelOrder;
        reserve_trading212_endpoint(book, endpoint)?;
        *cancel_may_have_been_sent = true;
        let response = http.request_with_rate_limit(
            ProviderEndpoint::Trading212Demo,
            ProviderHttpMethod::Delete,
            &format!("/api/v0/equity/orders/{order_id}"),
            auth.clone(),
            None,
        );
        let (response, limit_headers) = match response {
            Ok(response) => response,
            Err(_) => {
                let order = &mut book.orders[index];
                order.cancel_state = Trading212DemoCancelState::Pending;
                order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                book.status = Trading212DemoOrderBookStatus::Stale;
                book.reason = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                book.observed_at = crate::storage::timestamp()?;
                return Ok(());
            }
        };
        record_trading212_rate_limit(book, endpoint, limit_headers)?;
        let order = &mut book.orders[index];
        match response.status {
            200 => {
                order.cancel_state = Trading212DemoCancelState::Pending;
                order.cancel_error = None;
                book.reason = Some("ORDER_CANCEL_PENDING".into());
            }
            400 | 401 | 403 | 429 => {
                *cancel_may_have_been_sent = false;
                order.cancel_state = Trading212DemoCancelState::None;
                order.cancel_idempotency_key = None;
                order.cancel_error = Some(
                    match response.status {
                        400 => "ORDER_NOT_CANCELABLE",
                        401 => "PROVIDER_AUTH_FAILED",
                        403 => "PROVIDER_PERMISSION_BLOCKED",
                        _ => "PROVIDER_RATE_LIMITED",
                    }
                    .into(),
                );
                book.reason = order.cancel_error.clone();
            }
            _ => {
                order.cancel_state = Trading212DemoCancelState::Pending;
                order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                book.reason = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
            }
        }
        book.status = Trading212DemoOrderBookStatus::Stale;
        book.observed_at = crate::storage::timestamp()?;
        Ok(())
    }

    fn run_binance_testnet_order(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let Some(attempt) = self.binance_testnet_attempt.clone() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            };
        };
        let Some(proposal) = self.binance_testnet_proposal.as_ref() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            };
        };
        let result = (|| -> Result<BinanceTestnetOrderAttempt> {
            if self.account.provider_id != "binance"
                || self.account.environment != "TESTNET"
                || !matches!(
                    &self.kind,
                    JobKind::BinanceTestnetSubmit | JobKind::BinanceTestnetReconcile
                )
            {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            let values = secret.values()?;
            let reconcile = self.kind == JobKind::BinanceTestnetReconcile;
            Ok(binance::run_testnet_order(
                &attempt, proposal, reconcile, &values, http, &current,
            ))
        })();
        let attempt = result.unwrap_or_else(|error| {
            let mut attempt = attempt;
            attempt.state = if self.kind == JobKind::BinanceTestnetReconcile {
                BinanceTestnetOrderAttemptState::UnknownReconciling
            } else {
                BinanceTestnetOrderAttemptState::Rejected
            };
            attempt.error_code = Some(error.code.clone());
            attempt.reason = error.message;
            attempt
        });
        ProviderOutcome {
            observation: None,
            error: None,
            credential: "CONFIGURED".into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: Some(attempt),
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
        }
    }

    fn run_bitget_demo_order(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let Some(attempt) = self.bitget_demo_attempt.clone() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            };
        };
        let reconcile = self.kind == JobKind::BitgetDemoReconcile;
        let result = (|| -> Result<BitgetDemoOrderAttempt> {
            let proposal = self
                .bitget_demo_proposal
                .as_ref()
                .ok_or_else(|| TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"))?;
            if self.account.provider_id != "bitget" || self.account.environment != "DEMO" {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            let values = secret.values()?;
            Ok(bitget::run_demo_order(
                &attempt,
                proposal,
                reconcile,
                &self.account.permissions,
                &values,
                http,
                &current,
            ))
        })();
        let attempt = result.unwrap_or_else(|error| {
            let mut attempt = attempt;
            attempt.state = if reconcile {
                BitgetDemoOrderAttemptState::UnknownReconciling
            } else {
                BitgetDemoOrderAttemptState::Rejected
            };
            attempt.provider_order_id = None;
            attempt.provider_status = None;
            attempt.error_code = Some(error.code.clone());
            attempt.reason = error.message.chars().take(256).collect();
            attempt
        });
        ProviderOutcome {
            observation: None,
            error: None,
            credential: "CONFIGURED".into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: Some(attempt),
            resolution_evidence: None,
        }
    }

    fn run_binance_testnet_order_book(
        &self,
        vault: &impl CredentialVault,
        http: &impl ProviderHttp,
        current: impl Fn() -> bool,
    ) -> ProviderOutcome {
        let Some(mut book) = self.binance_testnet_order_book.clone() else {
            return ProviderOutcome {
                observation: None,
                error: Some(TradeXError::new("ORDER_STATUS_UNKNOWN")),
                credential: "MISSING".into(),
                trading212_demo_attempt: None,
                trading212_demo_order_book: None,
                alpaca_paper_attempt: None,
                alpaca_paper_order_book: None,
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            };
        };
        let mut credential = "MISSING";
        let mut cancel_may_have_been_sent = false;
        let result = (|| -> Result<BinanceTestnetOrderBook> {
            if self.account.provider_id != "binance"
                || self.account.environment != "TESTNET"
                || self.account.connection_state != ConnectionState::Connected
                || self.account.health.credential != "CONFIGURED"
            {
                return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
            }
            let secret = vault.get(&self.account.credential_ref())?;
            let values = secret.values()?;
            if values.len() != 2 {
                return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
            }
            credential = "CONFIGURED";
            if self.kind == JobKind::BinanceTestnetOrderCancel {
                binance::cancel_testnet_order(
                    &mut book,
                    self.binance_testnet_book_symbol
                        .as_deref()
                        .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?,
                    self.binance_testnet_book_order_id
                        .as_deref()
                        .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?,
                    &values,
                    http,
                    &current,
                    &mut cancel_may_have_been_sent,
                )?;
            } else {
                let action = self
                    .binance_testnet_book_action
                    .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
                binance::refresh_testnet_order_book(
                    &mut book,
                    action,
                    self.binance_testnet_book_symbol.as_deref(),
                    self.binance_testnet_book_order_id.as_deref(),
                    &values,
                    http,
                    &current,
                )?;
            }
            Ok(book.clone())
        })();
        if let Err(error) = result {
            book.status = if error.code == "STATE_VERSION_CONFLICT" {
                BinanceTestnetOrderBookStatus::Stale
            } else {
                BinanceTestnetOrderBookStatus::Degraded
            };
            book.reason = Some(error.code.clone());
            book.observed_at = crate::storage::timestamp().unwrap_or(book.observed_at);
            let cancel_error = book.reason.clone().unwrap_or_else(|| error.code.clone());
            if self.kind == JobKind::BinanceTestnetOrderCancel
                && let Some(order_id) = self.binance_testnet_book_order_id.as_deref()
                && let Some(symbol) = self.binance_testnet_book_symbol.as_deref()
                && let Some(order) = book
                    .orders
                    .iter_mut()
                    .find(|order| order.symbol == symbol && order.provider_order_id == order_id)
                && order.cancel_state == BinanceTestnetOrderCancelState::Submitting
            {
                if cancel_may_have_been_sent {
                    order.cancel_state = BinanceTestnetOrderCancelState::Pending;
                    order.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
                } else {
                    order.cancel_state = BinanceTestnetOrderCancelState::None;
                    order.cancel_error = Some(cancel_error);
                }
            }
        }
        ProviderOutcome {
            observation: None,
            error: None,
            credential: credential.into(),
            trading212_demo_attempt: None,
            trading212_demo_order_book: None,
            alpaca_paper_attempt: None,
            alpaca_paper_order_book: None,
            binance_testnet_attempt: None,
            binance_testnet_order_book: Some(book),
            bitget_demo_attempt: None,
            resolution_evidence: None,
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
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
            };
        };
        let mut credential_state = "MISSING";
        let outcome = (|| -> Result<AlpacaPaperOrderAttempt> {
            if self.account.provider_id != "alpaca" || self.account.environment != "PAPER" {
                return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
            }
            if !matches!(
                &self.kind,
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
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
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
                binance_testnet_attempt: None,
                binance_testnet_order_book: None,
                bitget_demo_attempt: None,
                resolution_evidence: None,
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
            match &self.kind {
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
            binance_testnet_attempt: None,
            binance_testnet_order_book: None,
            bitget_demo_attempt: None,
            resolution_evidence: None,
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
        "EXPIRED" => Trading212DemoNormalizedOrderStatus::Expired,
        _ => return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE")),
    };
    let pending = pending
        && !matches!(
            normalized_status,
            Trading212DemoNormalizedOrderStatus::Cancelled
                | Trading212DemoNormalizedOrderStatus::Filled
                | Trading212DemoNormalizedOrderStatus::Rejected
                | Trading212DemoNormalizedOrderStatus::Replaced
                | Trading212DemoNormalizedOrderStatus::Expired
        );
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
        cancel_state: Trading212DemoCancelState::None,
        cancel_idempotency_key: None,
        cancel_error: None,
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
            let terminal = trading212_terminal_order_status(&received.provider_status);
            if replace_pending_set
                || refresh_exact_order
                || received.pending
                || !current.pending
                || terminal
            {
                current.provider_status = received.provider_status;
                current.normalized_status = received.normalized_status;
                current.filled_quantity = received.filled_quantity;
                current.filled_value = received.filled_value;
                current.remaining_quantity = received.remaining_quantity;
                current.observed_at = received.observed_at;
            }
            current.currency = received.currency.or_else(|| current.currency.clone());
            if terminal {
                current.pending = false;
            } else if replace_pending_set || refresh_exact_order {
                current.pending = received.pending;
            } else {
                current.pending |= received.pending;
            }
            if terminal {
                current.cancel_state = Trading212DemoCancelState::None;
                current.cancel_idempotency_key = None;
                current.cancel_error = None;
            } else if current.cancel_state == Trading212DemoCancelState::None {
                current.cancel_idempotency_key = None;
                current.cancel_error = None;
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

fn trading212_terminal_order_status(status: &str) -> bool {
    matches!(
        status,
        "CANCELLED" | "FILLED" | "REJECTED" | "REPLACED" | "EXPIRED"
    )
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

pub(crate) fn validate_binance_testnet_proposal(
    proposal: &OrderProposal,
    connection_id: &str,
) -> Result<()> {
    binance::validate_binance_testnet_proposal(proposal, connection_id)
}

pub(crate) fn validate_bitget_demo_proposal(
    proposal: &OrderProposal,
    connection_id: &str,
) -> Result<()> {
    bitget::validate_demo_proposal(proposal, connection_id)
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
    trading212_order_request(proposal, connection_id, ExecutionContext::Trading212Demo)
}

pub(crate) fn trading212_live_order_request(
    proposal: &OrderProposal,
    connection_id: &str,
) -> Result<(String, Value)> {
    trading212_order_request(proposal, connection_id, ExecutionContext::Trading212Live)
}

fn trading212_order_request(
    proposal: &OrderProposal,
    connection_id: &str,
    environment: ExecutionContext,
) -> Result<(String, Value)> {
    let fields = &proposal.fields;
    if !matches!(
        proposal.status,
        crate::protocol::OrderProposalStatus::NeedsApproval
            | crate::protocol::OrderProposalStatus::Consumed
    ) || fields.environment != environment
        || fields.account_id.as_deref() != Some(connection_id)
        || fields.quantity.r#type != OrderQuantityType::Base
        || (fields.maximum_spend.is_some()
            && (environment != ExecutionContext::Trading212Live
                || fields.order_type != OrderType::Market))
    {
        return Err(TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"));
    }
    if environment == ExecutionContext::Trading212Live && fields.order_type == OrderType::Market {
        let maximum = fields
            .maximum_spend
            .as_ref()
            .ok_or_else(|| TradeXError::new("ORDER_PROPOSAL_NOT_ELIGIBLE"))?;
        let maximum = decimal(&Value::String(maximum.clone()))?;
        if decimal_cmp(&maximum, "0")? != std::cmp::Ordering::Greater {
            return Err(TradeXError::new("ORDER_AMOUNT_INVALID"));
        }
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

pub(super) fn decimal_cmp(left: &str, right: &str) -> Result<std::cmp::Ordering> {
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
    if crate::calendar_source::allowed_path(path) {
        return true;
    }
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

pub(crate) fn valid_provider_order_id(id: &str) -> bool {
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
mod bitget_live_cancel_route_tests {
    use super::{ProviderEndpoint, ProviderHttpMethod, valid_live_cancel_order_id};

    #[test]
    fn bitget_live_cancel_and_exact_read_routes_are_narrow() {
        assert!(valid_live_cancel_order_id(
            "bitget",
            "normal:9223372036854775807"
        ));
        for order_id in [
            "tpsl:12",
            "plan:12",
            "normal:0",
            "normal:012",
            "normal:abc",
            "normal:9223372036854775808",
        ] {
            assert!(
                !valid_live_cancel_order_id("bitget", order_id),
                "{order_id}"
            );
        }

        for path in [
            "/api/v2/spot/trade/orderInfo?orderId=9223372036854775807",
            "/api/v2/spot/trade/fills?limit=100&orderId=9223372036854775807",
            "/api/v2/spot/trade/fills?limit=100&orderId=9223372036854775807&idLessThan=9223372036854775806",
        ] {
            assert!(ProviderEndpoint::BitgetLive.allows(path), "{path}");
        }
        for path in [
            "/api/v2/spot/trade/orderInfo?orderId=0",
            "/api/v2/spot/trade/orderInfo?orderId=9223372036854775808",
            "/api/v2/spot/trade/orderInfo?clientOid=tx-0123456789abcdef&orderId=1",
            "/api/v2/spot/trade/fills?limit=100&orderId=9223372036854775808",
            "/api/v2/spot/trade/fills?limit=100&orderId=1&symbol=ETHUSDT",
            "/api/v2/spot/trade/fills?limit=100&orderId=1&idLessThan=0",
        ] {
            assert!(!ProviderEndpoint::BitgetLive.allows(path), "{path}");
        }
        assert_eq!(
            ProviderEndpoint::BitgetLive
                .allows_method(ProviderHttpMethod::Post, "/api/v2/spot/trade/cancel-order"),
            cfg!(feature = "order-gateway-runtime")
        );
        assert!(
            !ProviderEndpoint::BitgetDemo
                .allows_method(ProviderHttpMethod::Post, "/api/v2/spot/trade/cancel-order")
        );
    }
}

#[cfg(test)]
mod trading212_demo_route_tests {
    use super::{
        ProviderEndpoint, ProviderHttpMethod, Trading212Endpoint, merge_trading212_orders,
        parse_t212_recent_orders, parse_trading212_order, trading212_order_read_error,
        valid_t212_history_path,
    };
    use crate::protocol::Trading212DemoCancelState;
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
            ProviderEndpoint::Trading212Demo
                .allows_method(ProviderHttpMethod::Delete, "/api/v0/equity/orders/1")
        );
        assert!(ProviderEndpoint::Trading212Demo.allows_method(
            ProviderHttpMethod::Delete,
            "/api/v0/equity/orders/9007199254740995"
        ));
        assert!(!ProviderEndpoint::Trading212Live.allows_method(
            ProviderHttpMethod::Delete,
            "/api/v0/equity/orders/9007199254740995"
        ));
        assert!(ProviderEndpoint::Trading212Demo.allows("/api/v0/equity/orders"));
        assert!(ProviderEndpoint::Trading212Demo.allows("/api/v0/equity/orders/9007199254740995"));
        assert!(
            ProviderEndpoint::Trading212Demo
                .allows("/api/v0/equity/history/orders?limit=50&cursor=1760346100000")
        );
        assert!(
            ProviderEndpoint::Trading212Live
                .allows("/api/v0/equity/history/orders?limit=50&cursor=1760346100000")
        );
        assert!(ProviderEndpoint::Trading212Live.allows_method(
            ProviderHttpMethod::Get,
            "/api/v0/equity/orders/9007199254740995"
        ));
        for method in [ProviderHttpMethod::Post, ProviderHttpMethod::Delete] {
            assert!(!ProviderEndpoint::Trading212Live.allows_method(
                method,
                "/api/v0/equity/history/orders?limit=50&cursor=1760346100000"
            ));
        }
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
        for path in ["/api/v0/equity/orders/0", "/api/v0/equity/orders/01?x=1"] {
            assert!(
                !ProviderEndpoint::Trading212Demo.allows_method(ProviderHttpMethod::Delete, path),
                "{path}"
            );
        }
        assert!(valid_t212_history_path(
            "/api/v0/equity/history/orders?cursor=1760346100000&limit=50"
        ));
    }

    #[test]
    fn t212_recent_history_parses_official_order_fill_rows_and_rejects_conflicts() {
        let order = json!({
            "id":9001,"instrument":{"ticker":"AAPL_US_EQ"},"side":"BUY","strategy":"QUANTITY",
            "quantity":1,"filledQuantity":1,"filledValue":182.5,"currency":"USD",
            "status":"FILLED"
        });
        let row = |fill_id| {
            json!({
                "fill": {"id":fill_id,"filledAt":"2026-09-29T10:00:00Z","price":182.5,
                    "quantity":1,"tradingMethod":"TOTV","type":"TRADE"},
                "order":order.clone()
            })
        };
        let page = json!({
            "items":[row(1),row(2)],
            "nextPagePath":"/api/v0/equity/history/orders?limit=50&cursor=123"
        });
        let rows = parse_t212_recent_orders(page).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].broker_order_id, "9001");
        assert_eq!(rows[0].status, "FILLED");

        let conflicting = json!({
            "items":[row(1), {"fill":{"id":2}, "order":{
                "id":9001,"ticker":"AAPL_US_EQ","side":"BUY","strategy":"QUANTITY",
                "quantity":2,"filledQuantity":1,"filledValue":182.5,"currency":"USD",
                "status":"PARTIALLY_FILLED"
            }}],
            "nextPagePath":null
        });
        assert_eq!(
            parse_t212_recent_orders(conflicting).unwrap_err().code,
            "PROVIDER_DATA_INCOMPLETE"
        );
        assert_eq!(
            parse_t212_recent_orders(json!({"items":[],"nextPagePath":"https://evil.test"}))
                .unwrap_err()
                .code,
            "PROVIDER_DATA_INCOMPLETE"
        );
        assert_eq!(
            parse_t212_recent_orders(json!({"items":[order],"nextPagePath":null}))
                .unwrap_err()
                .code,
            "PROVIDER_RESPONSE_INVALID"
        );
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

        let expired = parse_trading212_order(
            &json!({
                "id":25,"ticker":"MSFT_US_EQ","side":"BUY","type":"MARKET",
                "timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":0,"status":"EXPIRED",
                "createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:01:00Z",
            true,
        )
        .unwrap();
        assert_eq!(
            expired.normalized_status,
            crate::protocol::Trading212DemoNormalizedOrderStatus::Expired
        );
        assert!(!expired.pending);

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
    fn order_merge_preserves_cancel_pending_until_provider_terminal_state() {
        let mut reviewed = parse_trading212_order(
            &json!({
                "id":9007199254740995u64,"ticker":"AAPL_US_EQ","side":"BUY",
                "type":"LIMIT","timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":1,"filledValue":25,"status":"PARTIALLY_FILLED",
                "createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:01:00Z",
            true,
        )
        .unwrap();
        reviewed.cancel_state = Trading212DemoCancelState::Pending;
        reviewed.cancel_idempotency_key = Some("cancel-key".into());
        reviewed.cancel_error = Some("ORDER_CANCEL_STATUS_UNKNOWN".into());
        let filled_again = parse_trading212_order(
            &json!({
                "id":9007199254740995u64,"ticker":"AAPL_US_EQ","side":"BUY",
                "type":"LIMIT","timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":2,"filledValue":50,"status":"PARTIALLY_FILLED",
                "createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:02:00Z",
            true,
        )
        .unwrap();
        let merged = merge_trading212_orders(&[reviewed], vec![filled_again], false, true).unwrap();
        assert_eq!(merged[0].filled_quantity.as_deref(), Some("2"));
        assert_eq!(merged[0].cancel_state, Trading212DemoCancelState::Pending);
        assert_eq!(
            merged[0].cancel_error.as_deref(),
            Some("ORDER_CANCEL_STATUS_UNKNOWN")
        );

        let cancelled = parse_trading212_order(
            &json!({
                "id":9007199254740995u64,"ticker":"AAPL_US_EQ","side":"BUY",
                "type":"LIMIT","timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
                "filledQuantity":2,"filledValue":50,"status":"CANCELLED",
                "createdAt":"2026-09-23T10:00:00Z"
            }),
            "2026-09-23T10:03:00Z",
            false,
        )
        .unwrap();
        let merged = merge_trading212_orders(&merged, vec![cancelled], false, true).unwrap();
        assert_eq!(merged[0].cancel_state, Trading212DemoCancelState::None);
        assert_eq!(merged[0].cancel_idempotency_key, None);
        assert_eq!(merged[0].cancel_error, None);
        assert!(!merged[0].pending);
    }

    #[test]
    fn trading212_order_reads_classify_client_and_server_errors_separately() {
        assert_eq!(
            trading212_order_read_error(418, Trading212Endpoint::PendingOrders).code,
            "PROVIDER_RESPONSE_INVALID"
        );
        assert_eq!(
            trading212_order_read_error(503, Trading212Endpoint::PendingOrders).code,
            "PROVIDER_UNAVAILABLE"
        );
        assert_eq!(
            trading212_order_read_error(404, Trading212Endpoint::OrderDetail).code,
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

#[cfg(test)]
mod trading212_live_cancel_preflight_tests {
    use super::{ProviderEndpoint, ProviderHttpMethod, validate_trading212_live_cancel_snapshot};
    use crate::protocol::{CancellationIntent, ExecutionContext};
    use serde_json::json;

    fn intent() -> CancellationIntent {
        CancellationIntent {
            cancellation_intent_id: "cancel:1".into(),
            intent_hash: format!("sha256:{}", "a".repeat(64)),
            workspace_id: "workspace".into(),
            account_id: "account".into(),
            environment: ExecutionContext::Trading212Live,
            provider_order_id: "123".into(),
            instrument_id: "equity:US:AAPL".into(),
            symbol: "AAPL_US_EQ".into(),
            side: "SELL".into(),
            provider_status: "PARTIALLY_FILLED".into(),
            quantity: "5".into(),
            filled_quantity: "1.25".into(),
            remaining_quantity: "3.75".into(),
            created_at: "2026-09-28T10:00:00Z".into(),
        }
    }

    fn order() -> serde_json::Value {
        json!({
            "id": 123,
            "ticker": "AAPL_US_EQ",
            "side": "SELL",
            "type": "LIMIT",
            "timeInForce": "GOOD_TILL_CANCEL",
            "strategy": "QUANTITY",
            "quantity": -5,
            "filledQuantity": -1.25,
            "status": "PARTIALLY_FILLED"
        })
    }

    #[test]
    fn live_cancel_preflight_accepts_exact_signed_sell_snapshot_and_rejects_changes() {
        let intent = intent();
        assert_eq!(
            validate_trading212_live_cancel_snapshot(&order(), &intent).unwrap(),
            "PARTIALLY_FILLED"
        );

        let mut changed = order();
        changed["filledQuantity"] = json!(-2);
        assert_eq!(
            validate_trading212_live_cancel_snapshot(&changed, &intent)
                .unwrap_err()
                .code,
            "ORDER_CHANGED_REVIEW_AGAIN"
        );
        let mut terminal = order();
        terminal["status"] = json!("FILLED");
        assert_eq!(
            validate_trading212_live_cancel_snapshot(&terminal, &intent)
                .unwrap_err()
                .code,
            "ORDER_NOT_CANCELLABLE"
        );
        assert!(
            ProviderEndpoint::Trading212Live
                .allows_method(ProviderHttpMethod::Get, "/api/v0/equity/orders/123")
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
            recent_orders: Vec::new(),
            bitget_order_book: None,
            capabilities: permissions.detected.clone(),
            limitations,
        },
        permissions,
        live_order_settlements: Vec::new(),
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

#[cfg(test)]
mod provider_scheduler_tests {
    use super::{JobKind, ProviderPriority, ProviderScheduler};
    use std::{
        sync::{Arc, mpsc},
        thread,
        time::{Duration, Instant},
    };

    fn wait_until(mut ready: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !ready() {
            assert!(Instant::now() < deadline, "provider waiter did not queue");
            thread::yield_now();
        }
    }

    #[test]
    fn provider_jobs_use_the_documented_priority_classes() {
        assert_eq!(
            JobKind::AlpacaPaperReconcile.provider_priority(),
            ProviderPriority::P0
        );
        assert_eq!(JobKind::Probe.provider_priority(), ProviderPriority::P1);
        assert_eq!(
            JobKind::BinanceTestnetOrderBookRefresh.provider_priority(),
            ProviderPriority::P2
        );
        assert_eq!(
            JobKind::Trading212DemoOrderBookHistory.provider_priority(),
            ProviderPriority::P3
        );
    }

    #[test]
    fn p0_bypasses_a_saturated_p3_queue_and_overflow_is_explicit() {
        let scheduler = Arc::new(ProviderScheduler::with_limits(1, 2, 1));
        let occupied = scheduler
            .acquire("alpaca", ProviderPriority::P0, &|| true)
            .unwrap();
        let (order_tx, order_rx) = mpsc::channel();
        let (p3_release_tx, p3_release_rx) = mpsc::channel();
        let p3_scheduler = Arc::clone(&scheduler);
        let p3 = thread::spawn(move || {
            let _permit = p3_scheduler
                .acquire("alpaca", ProviderPriority::P3, &|| true)
                .unwrap();
            order_tx.send("p3").unwrap();
            p3_release_rx.recv().unwrap();
        });
        wait_until(|| scheduler.queued("alpaca") == 1);

        let error = match scheduler.acquire("alpaca", ProviderPriority::P3, &|| true) {
            Err(error) => error,
            Ok(_) => panic!("the bounded low-priority queue accepted overflow"),
        };
        assert_eq!(error.code, "PROVIDER_BACKPRESSURE");
        assert_eq!(error.category, "RATE_LIMITED");
        assert!(error.retryable);

        let (p0_tx, p0_rx) = mpsc::channel();
        let (p0_release_tx, p0_release_rx) = mpsc::channel();
        let p0_scheduler = Arc::clone(&scheduler);
        let p0 = thread::spawn(move || {
            let _permit = p0_scheduler
                .acquire("alpaca", ProviderPriority::P0, &|| true)
                .unwrap();
            p0_tx.send(()).unwrap();
            p0_release_rx.recv().unwrap();
        });
        wait_until(|| scheduler.queued("alpaca") == 2);
        drop(occupied);

        p0_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(
            order_rx.try_recv().is_err(),
            "P3 ran ahead of queued P0 work"
        );
        p0_release_tx.send(()).unwrap();
        assert_eq!(order_rx.recv_timeout(Duration::from_secs(2)).unwrap(), "p3");
        p3_release_tx.send(()).unwrap();
        p0.join().unwrap();
        p3.join().unwrap();
    }

    #[test]
    fn p1_runs_before_queued_p2_and_p3_work() {
        let scheduler = Arc::new(ProviderScheduler::with_limits(1, 4, 4));
        let occupied = scheduler
            .acquire("binance", ProviderPriority::P0, &|| true)
            .unwrap();
        let (order_tx, order_rx) = mpsc::channel();
        let (p2_release_tx, p2_release_rx) = mpsc::channel();
        let (p3_release_tx, p3_release_rx) = mpsc::channel();
        let (p1_release_tx, p1_release_rx) = mpsc::channel();
        let p2_scheduler = Arc::clone(&scheduler);
        let p2_tx = order_tx.clone();
        let p2 = thread::spawn(move || {
            let _permit = p2_scheduler
                .acquire("binance", ProviderPriority::P2, &|| true)
                .unwrap();
            p2_tx.send("p2").unwrap();
            p2_release_rx.recv().unwrap();
        });
        wait_until(|| scheduler.queued("binance") == 1);
        let p3_scheduler = Arc::clone(&scheduler);
        let p3_tx = order_tx.clone();
        let p3 = thread::spawn(move || {
            let _permit = p3_scheduler
                .acquire("binance", ProviderPriority::P3, &|| true)
                .unwrap();
            p3_tx.send("p3").unwrap();
            p3_release_rx.recv().unwrap();
        });
        wait_until(|| scheduler.queued("binance") == 2);
        let p1_scheduler = Arc::clone(&scheduler);
        let p1 = thread::spawn(move || {
            let _permit = p1_scheduler
                .acquire("binance", ProviderPriority::P1, &|| true)
                .unwrap();
            order_tx.send("p1").unwrap();
            p1_release_rx.recv().unwrap();
        });
        wait_until(|| scheduler.queued("binance") == 3);
        drop(occupied);

        assert_eq!(order_rx.recv_timeout(Duration::from_secs(2)).unwrap(), "p1");
        p1_release_tx.send(()).unwrap();
        assert_eq!(order_rx.recv_timeout(Duration::from_secs(2)).unwrap(), "p2");
        p2_release_tx.send(()).unwrap();
        assert_eq!(order_rx.recv_timeout(Duration::from_secs(2)).unwrap(), "p3");
        p3_release_tx.send(()).unwrap();
        p1.join().unwrap();
        p2.join().unwrap();
        p3.join().unwrap();
    }

    #[test]
    fn p0_keeps_one_provider_slot_when_low_priority_work_is_active() {
        let scheduler = Arc::new(ProviderScheduler::with_limits(2, 2, 1));
        let active_p3 = scheduler
            .acquire("trading212", ProviderPriority::P3, &|| true)
            .unwrap();
        let (p3_tx, p3_rx) = mpsc::channel();
        let p3_scheduler = Arc::clone(&scheduler);
        let p3 = thread::spawn(move || {
            let _permit = p3_scheduler
                .acquire("trading212", ProviderPriority::P3, &|| true)
                .unwrap();
            p3_tx.send(()).unwrap();
        });
        wait_until(|| scheduler.queued("trading212") == 1);

        let p0 = scheduler
            .acquire("trading212", ProviderPriority::P0, &|| true)
            .unwrap();
        assert_eq!(scheduler.queued("trading212"), 1);
        drop(p0);
        drop(active_p3);
        p3_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        p3.join().unwrap();
    }

    #[test]
    fn provider_concurrency_never_exceeds_its_limit() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let scheduler = Arc::new(ProviderScheduler::with_limits(2, 8, 8));
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let workers = (0..8)
            .map(|_| {
                let scheduler = Arc::clone(&scheduler);
                let active = Arc::clone(&active);
                let maximum = Arc::clone(&maximum);
                thread::spawn(move || {
                    let _permit = scheduler
                        .acquire("bitget", ProviderPriority::P0, &|| true)
                        .unwrap();
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    maximum.fetch_max(current, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(10));
                    active.fetch_sub(1, Ordering::SeqCst);
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
        assert_eq!(maximum.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn provider_retry_after_delays_the_next_eligible_request() {
        let scheduler = ProviderScheduler::with_limits(1, 2, 1);
        scheduler.delay_provider("alpaca", None, Duration::from_millis(40));
        let started = Instant::now();
        let _permit = scheduler
            .acquire("alpaca", ProviderPriority::P0, &|| true)
            .unwrap();
        assert!(started.elapsed() >= Duration::from_millis(30));
    }

    #[test]
    fn cooled_account_does_not_block_another_accounts_eligible_work() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let scheduler = Arc::new(ProviderScheduler::with_limits(1, 2, 1));
        scheduler.delay_provider("alpaca", Some("account-a"), Duration::from_secs(5));
        let current = Arc::new(AtomicBool::new(true));
        let waiting_scheduler = Arc::clone(&scheduler);
        let waiting_current = Arc::clone(&current);
        let waiter = thread::spawn(move || {
            let result = waiting_scheduler.acquire_for_account(
                "alpaca",
                Some("account-a"),
                ProviderPriority::P0,
                &|| waiting_current.load(Ordering::Acquire),
            );
            assert_eq!(result.err().unwrap().code, "STATE_VERSION_CONFLICT");
        });
        wait_until(|| scheduler.queued("alpaca") == 1);
        let (tx, rx) = mpsc::channel();
        let eligible_scheduler = Arc::clone(&scheduler);
        let eligible = thread::spawn(move || {
            let _permit = eligible_scheduler
                .acquire_for_account("alpaca", Some("account-b"), ProviderPriority::P1, &|| true)
                .unwrap();
            tx.send(()).unwrap();
        });
        let progressed = rx.recv_timeout(Duration::from_secs(1));
        current.store(false, Ordering::Release);
        waiter.join().unwrap();
        eligible.join().unwrap();
        progressed.unwrap();
    }
}
