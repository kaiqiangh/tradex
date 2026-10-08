//! Ephemeral, single-worker stock quote leases. Provider bytes enter the shared quote cache.
use super::transport::Socket;
use super::*;
use crate::protocol::{
    HotQuoteAcquire, HotQuoteProjection, HotQuoteQuery, HotQuoteRelease, HotQuoteStatus,
    Instrument, MarketTier,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use tungstenite::Message;
use zeroize::Zeroizing;

#[derive(Clone, Default)]
pub struct StockStreamConnector {
    #[cfg(feature = "integration-test")]
    loopback: Option<String>,
}
#[derive(Clone, Default)]
pub struct QuoteStreamConnectors {
    pub stock: StockStreamConnector,
    pub binance: crate::binance_market::stream::BinanceStreamConnector,
}
impl From<StockStreamConnector> for QuoteStreamConnectors {
    fn from(stock: StockStreamConnector) -> Self {
        Self {
            stock,
            binance: Default::default(),
        }
    }
}

impl StockStreamConnector {
    #[cfg(feature = "integration-test")]
    pub fn for_loopback_test(url: &str) -> Result<Self> {
        let parsed =
            reqwest::Url::parse(url).map_err(|_| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
        if parsed.scheme() != "ws"
            || !parsed.host_str().is_some_and(|host| {
                host.parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
            })
            || parsed.port().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || !["/v2/iex", "/v2/sip", "/v2/delayed_sip"].contains(&parsed.path())
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        Ok(Self {
            loopback: Some(url.into()),
        })
    }
    fn connect(&self, feed: &AlpacaFeed, current: &(impl Fn() -> bool + Sync)) -> Result<Socket> {
        let url = format!("wss://stream.data.alpaca.markets/v2/{}", feed_name(feed));
        #[cfg(feature = "integration-test")]
        let url = self.loopback.clone().unwrap_or(url);
        let parsed =
            reqwest::Url::parse(&url).map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        if parsed.path() != format!("/v2/{}", feed_name(feed)) {
            return Err(TradeXError::new("DATA_SOURCE_FEED_DENIED"));
        }
        super::transport::connect(&url, 262_144, current)
    }
}

#[derive(Clone)]
pub(crate) struct StockLeaseState {
    binding: SourceReadBinding,
    instrument: Instrument,
    projection: HotQuoteProjection,
    stream_snapshot_id: Option<String>,
}
impl StockLeaseState {
    fn current(&self, control: &ControlPlane) -> bool {
        self.binding.current_source(control)
            && self.binding.connection_generation == control.quote_connection_generation
            && control.hot_quote.as_ref().is_some_and(|lease| {
                lease.projection().lease_id == self.projection.lease_id
                    && lease.projection().generation == self.projection.generation
                    && !matches!(
                        lease.projection().status,
                        HotQuoteStatus::Closed | HotQuoteStatus::Failed
                    )
            })
    }
    pub(crate) fn streaming_for(
        &self,
        instrument: &str,
        generation: &str,
        control: &ControlPlane,
    ) -> bool {
        self.current(control)
            && self.projection.instrument_id == instrument
            && self.binding.connection_generation == generation
            && self.projection.status == HotQuoteStatus::Streaming
            && self.projection.authenticated
            && self.projection.subscribed
            && control
                .quote_observations
                .get(instrument)
                .is_some_and(|accepted| {
                    self.stream_snapshot_id.as_ref()
                        == Some(&accepted.snapshot.provenance.market_snapshot_id)
                })
    }
}

#[derive(Clone)]
pub(crate) enum HotLeaseState {
    Stock(StockLeaseState),
    Binance(crate::binance_market::stream::Lease),
}
impl HotLeaseState {
    pub(crate) fn projection(&self) -> &HotQuoteProjection {
        match self {
            Self::Stock(v) => &v.projection,
            Self::Binance(v) => &v.projection,
        }
    }
    fn projection_mut(&mut self) -> &mut HotQuoteProjection {
        match self {
            Self::Stock(v) => &mut v.projection,
            Self::Binance(v) => &mut v.projection,
        }
    }
    fn stock_mut(&mut self) -> Option<&mut StockLeaseState> {
        match self {
            Self::Stock(v) => Some(v),
            _ => None,
        }
    }
    pub(crate) fn current(&self, control: &ControlPlane) -> bool {
        match self {
            Self::Stock(v) => v.current(control),
            Self::Binance(v) => v.current(control),
        }
    }
    pub(crate) fn missing_quote_reason(&self, instrument: &str) -> Option<&str> {
        (self.projection().instrument_id == instrument).then_some(self.projection().reason.as_str())
    }
    pub(crate) fn streaming_for(
        &self,
        instrument: &str,
        generation: &str,
        control: &ControlPlane,
    ) -> bool {
        match self {
            Self::Stock(v) => v.streaming_for(instrument, generation, control),
            Self::Binance(_) => false,
        }
    }
    fn clear_snapshot(&mut self) {
        match self {
            Self::Stock(v) => v.stream_snapshot_id = None,
            Self::Binance(v) => v.stream_snapshot_id = None,
        }
    }
}

pub(crate) fn projection(
    control: &mut ControlPlane,
    input: HotQuoteQuery,
) -> Result<HotQuoteProjection> {
    control.require_workspace(&input.workspace_id)?;
    let lease = control
        .hot_quote
        .as_ref()
        .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?
        .clone();
    if input.workspace_id != lease.projection().workspace_id
        || input.lease_id != lease.projection().lease_id
        || input.generation != lease.projection().generation
    {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let mut result = lease.projection().clone();
    if !matches!(
        result.status,
        HotQuoteStatus::Closed | HotQuoteStatus::Failed
    ) && !lease.current(control)
    {
        result.status = HotQuoteStatus::Stale;
        result.authenticated = false;
        result.subscribed = false;
        result.reason = "Source, workspace or account changed; reacquire the quote lease.".into();
    }
    result.detail = Some(control.market_detail_for(
        &input.workspace_id,
        &result.instrument_id,
        &MarketTier::Hot,
        false,
    )?);
    Ok(result)
}

pub(crate) fn retire_for_session(control: &mut ControlPlane) {
    invalidate_quotes(control, "DATA_SOURCE_SESSION_CHANGED");
    control.quote_source_access_binding = None;
    if let Some(lease) = control.hot_quote.as_mut() {
        lease.clear_snapshot();
        let lease = lease.projection_mut();
        lease.status = HotQuoteStatus::Stale;
        lease.authenticated = false;
        lease.subscribed = false;
        lease.reason = "Session safety changed; reacquire and verify the selected source.".into();
        lease.sequence = lease.sequence.saturating_add(1).min(MAX_SEQUENCE);
    }
}

struct Job {
    control: Arc<Mutex<ControlPlane>>,
    lease: StockLeaseState,
    vault: Arc<dyn CredentialVault + Send + Sync>,
    http: Arc<dyn ProviderHttp + Send + Sync>,
    connector: StockStreamConnector,
}
enum PendingJob {
    Stock(Job),
    Binance(crate::binance_market::stream::Job),
}
impl PendingJob {
    fn run(&mut self, stop: &AtomicBool) {
        match self {
            Self::Stock(v) => run(v, stop),
            Self::Binance(v) => v.run(stop),
        }
    }
    fn retire(&self) {
        match self {
            Self::Stock(v) => retire_stopped(v),
            Self::Binance(v) => v.retire_stopped(),
        }
    }
}
struct Worker {
    pending: Arc<(Mutex<Option<PendingJob>>, std::sync::Condvar)>,
    stop: Arc<AtomicBool>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.pending.1.notify_all();
    }
}
#[derive(Clone, Default)]
pub struct QuoteHotSupervisor {
    worker: Arc<Mutex<Option<Worker>>>,
    stopped: Arc<AtomicBool>,
}
impl QuoteHotSupervisor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn stop_all(&self) {
        if let Ok(worker) = self.worker.lock() {
            self.stopped.store(true, Ordering::Release);
            if let Some(worker) = worker.as_ref() {
                worker.pending.1.notify_all();
            }
        }
    }
    pub fn dispatch_with<
        V: CredentialVault + Send + Sync + 'static,
        H: ProviderHttp + Send + Sync + 'static,
    >(
        &self,
        control: &Arc<Mutex<ControlPlane>>,
        request: &Value,
        consumer: &str,
        vault: Arc<V>,
        http: Arc<H>,
        connectors: impl Into<QuoteStreamConnectors>,
    ) -> Value {
        let connectors = connectors.into();
        let result = (|| {
            if !provider_order_consumer_allowed(consumer) {
                return Err(TradeXError::new("IPC_ACCESS_DENIED"));
            }
            let envelope: CommandEnvelope = payload(request.clone())?;
            if envelope.schema_version != 1 {
                return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
            }
            if !crate::valid_bounded_text(&envelope.request_id, 128) {
                return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
            }
            if envelope.command == "market.hot.release" {
                let input: HotQuoteQuery = payload(envelope.payload)?;
                let mut control = control
                    .lock()
                    .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
                control.require_workspace(&input.workspace_id)?;
                let released = if let Some(lease) = control.hot_quote.as_mut().filter(|lease| {
                    lease.projection().lease_id == input.lease_id
                        && lease.projection().generation == input.generation
                        && lease.projection().workspace_id == input.workspace_id
                }) {
                    lease.clear_snapshot();
                    let lease = lease.projection_mut();
                    let changed = lease.status != HotQuoteStatus::Closed;
                    lease.status = HotQuoteStatus::Closed;
                    lease.authenticated = false;
                    lease.subscribed = false;
                    lease.reason = "Quote lease released.".into();
                    lease.sequence = lease.sequence.saturating_add(1).min(MAX_SEQUENCE);
                    changed
                } else {
                    false
                };
                return Ok(json!(HotQuoteRelease {
                    workspace_id: input.workspace_id,
                    lease_id: input.lease_id,
                    generation: input.generation,
                    released,
                    status: HotQuoteStatus::Closed
                }));
            }
            if envelope.command != "market.hot.acquire" {
                return Err(TradeXError::new("IPC_COMMAND_UNKNOWN"));
            }
            let input: HotQuoteAcquire = payload(envelope.payload)?;
            // Serialize stop and acquire before modifying any public lease or quote binding.
            let mut worker = self
                .worker
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
            if self.stopped.load(Ordering::Acquire) {
                return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
            }
            let mut locked = control
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
            let detail = locked.market_detail_for(
                &input.workspace_id,
                &input.instrument_id,
                &MarketTier::Hot,
                false,
            )?;
            let pending_job = match detail.instrument.asset_class {
                crate::protocol::AssetClass::Equity => {
                    let mut binding = SourceReadBinding::capture(&mut locked, &input.workspace_id)?;
                    if binding.source_version != input.expected_source_version {
                        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                    }
                    if binding.reference.is_none() || binding.saved.feed.is_none() {
                        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
                    }
                    let generation = uuid::Uuid::new_v4().to_string();
                    locked.quote_connection_generation = generation.clone();
                    binding.connection_generation = generation.clone();
                    let lease = StockLeaseState {
                        binding: binding.clone(),
                        instrument: detail.instrument,
                        stream_snapshot_id: None,
                        projection: HotQuoteProjection {
                            workspace_id: input.workspace_id.clone(),
                            source_id: "OD-001".into(),
                            instrument_id: input.instrument_id,
                            lease_id: uuid::Uuid::new_v4().to_string(),
                            connection_generation: generation.clone(),
                            reconnect_attempt: 0,
                            generation,
                            source_version: binding.source_version,
                            sequence: 0,
                            status: HotQuoteStatus::Connecting,
                            authenticated: false,
                            subscribed: false,
                            reason: "Connecting to the explicitly selected stock feed.".into(),
                            detail: None,
                        },
                    };
                    locked.hot_quote = Some(HotLeaseState::Stock(lease.clone()));
                    let pending_job = PendingJob::Stock(Job {
                        control: control.clone(),
                        lease,
                        vault,
                        http,
                        connector: connectors.stock,
                    });
                    pending_job
                }
                crate::protocol::AssetClass::CryptoSpot => {
                    PendingJob::Binance(crate::binance_market::stream::Job::prepare(
                        &mut locked,
                        control.clone(),
                        &input,
                        detail.instrument,
                        http,
                        connectors.binance,
                        self.stopped.clone(),
                    )?)
                }
            };
            let owner = locked
                .hot_quote
                .as_ref()
                .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?
                .clone();
            let result = projection(
                &mut locked,
                HotQuoteQuery {
                    workspace_id: input.workspace_id.clone(),
                    lease_id: owner.projection().lease_id.clone(),
                    generation: owner.projection().generation.clone(),
                },
            )?;
            drop(locked);
            // Concurrent acquire completion must not replace a newer pending job with an old lease.
            if !control.lock().is_ok_and(|control| owner.current(&control)) {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            if worker.is_none() {
                let pending = Arc::new((Mutex::new(None::<PendingJob>), std::sync::Condvar::new()));
                let stop = self.stopped.clone();
                let stopped = stop.clone();
                let jobs = pending.clone();
                thread::spawn(move || {
                    loop {
                        let mut next = match jobs.0.lock() {
                            Ok(next) => next,
                            Err(_) => break,
                        };
                        while next.is_none() && !stopped.load(Ordering::Acquire) {
                            next = match jobs.1.wait(next) {
                                Ok(next) => next,
                                Err(_) => return,
                            };
                        }
                        if stopped.load(Ordering::Acquire) {
                            let cancelled = next.take();
                            drop(next);
                            if let Some(job) = cancelled {
                                job.retire();
                            }
                            break;
                        }
                        let mut job = next.take().unwrap();
                        drop(next);
                        job.run(&stopped);
                        if stopped.load(Ordering::Acquire) {
                            job.retire();
                        }
                    }
                });
                *worker = Some(Worker { pending, stop });
            }
            let worker = worker.as_ref().unwrap();
            // Only the newest pending lease survives. The active worker closes before replacement.
            *worker
                .pending
                .0
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))? =
                Some(pending_job);
            worker.pending.1.notify_one();
            Ok(json!(result))
        })();
        match result {
            Ok(value) => success_reply(request_id(request), value, None),
            Err(error) => failure_reply(request_id(request), error),
        }
    }
}

fn retire_stopped(job: &Job) {
    if let Ok(mut control) = job.control.lock()
        && job.lease.current(&control)
        && let Some(lease) = control
            .hot_quote
            .as_mut()
            .and_then(HotLeaseState::stock_mut)
    {
        lease.stream_snapshot_id = None;
        lease.projection.status = HotQuoteStatus::Closed;
        lease.projection.authenticated = false;
        lease.projection.subscribed = false;
        lease.projection.reason = "Quote connection manager stopped.".into();
        lease.projection.sequence = lease
            .projection
            .sequence
            .saturating_add(1)
            .min(MAX_SEQUENCE);
    }
}

struct SharedHttp(Arc<dyn ProviderHttp + Send + Sync>);
impl ProviderHttp for SharedHttp {
    fn get(&self, e: ProviderEndpoint, p: &str, h: HeaderMap) -> Result<Vec<u8>> {
        self.0.get(e, p, h)
    }
    fn request(
        &self,
        e: ProviderEndpoint,
        m: ProviderHttpMethod,
        p: &str,
        h: HeaderMap,
        b: Option<&Value>,
    ) -> Result<crate::provider_io::ProviderHttpResponse> {
        self.0.request(e, m, p, h, b)
    }
    fn request_with_rate_limit(
        &self,
        e: ProviderEndpoint,
        m: ProviderHttpMethod,
        p: &str,
        h: HeaderMap,
        b: Option<&Value>,
    ) -> Result<(
        crate::provider_io::ProviderHttpResponse,
        Option<crate::provider_io::ProviderRateLimit>,
    )> {
        self.0.request_with_rate_limit(e, m, p, h, b)
    }
}
fn current(job: &Job, stop: &AtomicBool) -> bool {
    !stop.load(Ordering::Acquire)
        && job
            .control
            .lock()
            .is_ok_and(|control| job.lease.current(&control))
}
fn transition(
    job: &Job,
    status: HotQuoteStatus,
    authenticated: bool,
    subscribed: bool,
    reason: &str,
) -> Result<()> {
    let mut control = job
        .control
        .lock()
        .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
    if !job.lease.current(&control) {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let lease = control
        .hot_quote
        .as_mut()
        .and_then(HotLeaseState::stock_mut)
        .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?;
    lease.projection.sequence = lease
        .projection
        .sequence
        .checked_add(1)
        .filter(|n| *n <= MAX_SEQUENCE)
        .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
    lease.projection.status = status;
    lease.projection.authenticated = authenticated;
    lease.projection.subscribed = subscribed;
    lease.projection.reason = reason.into();
    Ok(())
}
fn run_connection(job: &Job, stop: &AtomicBool) -> Result<()> {
    if !current(&job, stop) {
        return Ok(());
    }
    let mut socket = None;
    let result = (|| {
        let credentials = job.vault.get(
            job.lease
                .binding
                .reference
                .as_deref()
                .ok_or_else(|| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?,
        )?;
        if !current(&job, stop) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let values = Zeroizing::new(credentials.values()?);
        if values.len() != 2 {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        let connected = job
            .connector
            .connect(job.lease.binding.saved.feed.as_ref().unwrap(), &|| {
                current(job, stop)
            })?;
        socket = Some(connected);
        let socket = socket.as_mut().unwrap();
        if !current(&job, stop) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        transition(
            &job,
            HotQuoteStatus::Authenticating,
            false,
            false,
            "Waiting for provider authentication confirmation.",
        )?;
        let authentication =
            Zeroizing::new(json!({"action":"auth","key":values[0],"secret":values[1]}).to_string());
        socket
            .send(Message::text(authentication.as_str()))
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        let symbol = job
            .lease
            .instrument
            .providers
            .iter()
            .find(|p| p.provider_id == "alpaca")
            .ok_or_else(|| TradeXError::new("MARKET_INSTRUMENT_NOT_FOUND"))?
            .provider_symbol
            .clone();
        let mut authenticated = false;
        let mut subscribed = false;
        let mut deadline = Instant::now() + Duration::from_secs(10);
        let mut ping_at = Instant::now();
        let mut pending_ping: Option<(Instant, Vec<u8>)> = None;
        let mut dictionaries = None;
        while current(&job, stop) {
            if !subscribed && Instant::now() > deadline {
                return Err(TradeXError::new("PROVIDER_AUTH_TIMEOUT"));
            }
            if pending_ping
                .as_ref()
                .is_some_and(|(deadline, _)| Instant::now() > *deadline)
            {
                return Err(TradeXError::new("PROVIDER_STREAM_STALE"));
            }
            if ping_at.elapsed() > Duration::from_secs(20) && pending_ping.is_none() {
                let nonce = uuid::Uuid::new_v4().as_bytes().to_vec();
                socket
                    .send(Message::Ping(nonce.clone().into()))
                    .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                pending_ping = Some((Instant::now() + Duration::from_secs(10), nonce));
                ping_at = Instant::now();
            }
            let message = match socket.read() {
                Ok(message) => message,
                Err(tungstenite::Error::Io(error))
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    continue;
                }
                Err(
                    tungstenite::Error::Capacity(_)
                    | tungstenite::Error::Protocol(_)
                    | tungstenite::Error::Utf8(_)
                    | tungstenite::Error::AttackAttempt,
                ) => return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID")),
                Err(tungstenite::Error::WriteBufferFull(_)) => {
                    return Err(TradeXError::new("PROVIDER_BACKPRESSURE"));
                }
                Err(_) => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
            };
            let text = match message {
                Message::Text(text) => text,
                Message::Ping(bytes) => {
                    socket
                        .send(Message::Pong(bytes))
                        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                    continue;
                }
                Message::Pong(bytes) => {
                    if pending_ping
                        .as_ref()
                        .is_some_and(|(_, nonce)| nonce.as_slice() == bytes.as_ref())
                    {
                        pending_ping = None;
                    }
                    continue;
                }
                Message::Close(_) => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
                _ => return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID")),
            };
            let received = job
                .control
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
                .time
                .status(&job.lease.binding.workspace_id)?;
            let raw_frames: Vec<Box<serde_json::value::RawValue>> = serde_json::from_str(&text)
                .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
            let value: Value = serde_json::from_str(&text)
                .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
            let frames = value
                .as_array()
                .filter(|frames| !frames.is_empty() && frames.len() <= 64)
                .ok_or_else(|| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
            if frames.len() != 1
                && frames.iter().any(|frame| {
                    matches!(
                        frame["T"].as_str(),
                        Some("success" | "subscription" | "error")
                    )
                })
            {
                return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
            }
            for (frame, raw_frame) in frames.iter().zip(&raw_frames) {
                match frame["T"].as_str() {
                    Some("success") if frame["msg"] == "connected" && !authenticated => (),
                    Some("success") if frame["msg"] == "authenticated" && !authenticated => {
                        authenticated = true;
                        deadline = Instant::now() + Duration::from_secs(10);
                        transition(
                            &job,
                            HotQuoteStatus::Subscribing,
                            true,
                            false,
                            "Authenticated; waiting for complete quote subscription confirmation.",
                        )?;
                        socket
                            .send(Message::text(
                                json!({"action":"subscribe","quotes":[symbol]}).to_string(),
                            ))
                            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                    }
                    Some("subscription") if authenticated => {
                        if frame["quotes"] != json!([symbol])
                            || frame.as_object().is_none_or(|fields| {
                                fields.iter().any(|(field, items)| {
                                    !matches!(field.as_str(), "T" | "quotes")
                                        && (!matches!(
                                            field.as_str(),
                                            "trades"
                                                | "bars"
                                                | "updatedBars"
                                                | "dailyBars"
                                                | "statuses"
                                                | "lulds"
                                                | "corrections"
                                                | "cancelErrors"
                                        ) || items
                                            .as_array()
                                            .is_none_or(|items| !items.is_empty()))
                                })
                            })
                        {
                            return Err(TradeXError::new("PROVIDER_SUBSCRIPTION_INVALID"));
                        }
                        subscribed = true;
                        transition(
                            &job,
                            HotQuoteStatus::AwaitingQuote,
                            true,
                            true,
                            "Subscription confirmed; waiting for an actual provider quote.",
                        )?;
                    }
                    Some("q") if authenticated && subscribed => {
                        if frame["S"] != symbol {
                            return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
                        }
                        validate_quote_numeric_tokens(raw_frame.get().as_bytes())?;
                        let body = serde_json::to_vec(&json!({"quotes":{symbol.clone():frame}}))
                            .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
                        let quote = parse_latest_response(&body, &symbol)?;
                        if quote.tape != "C"
                            || job.lease.instrument.exchange.as_deref() != Some("XNAS")
                        {
                            return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
                        }
                        if dictionaries.is_none() {
                            let live = || current(&job, stop);
                            let shared = SharedHttp(job.http.clone());
                            let http = crate::provider_io::p3_provider_http(
                                &shared,
                                &live,
                                job.lease.binding.reference.as_deref().unwrap(),
                            );
                            let headers = source_headers(&credentials)?;
                            dictionaries = Some((
                                metadata(
                                    &read_data(
                                        &http,
                                        "/v2/stocks/meta/conditions/quote?tape=C",
                                        headers.clone(),
                                    )?,
                                    &values,
                                )?,
                                metadata(
                                    &read_data(&http, "/v2/stocks/meta/exchanges", headers)?,
                                    &values,
                                )?,
                            ));
                        }
                        let (conditions, exchanges) = dictionaries.as_ref().unwrap();
                        let evidence = quote_evidence(
                            &job.lease.binding,
                            &job.lease.instrument,
                            &quote,
                            conditions,
                            exchanges,
                        )?;
                        let mut control = job
                            .control
                            .lock()
                            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
                        if !job.lease.current(&control) {
                            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                        }
                        let now = control.time.status(&job.lease.binding.workspace_id)?;
                        let snapshot = accept_snapshot(
                            &control,
                            &job.lease.binding,
                            &job.lease.projection.instrument_id,
                            quote,
                            evidence,
                            &received,
                        )?;
                        control
                            .hot_quote
                            .as_mut()
                            .and_then(HotLeaseState::stock_mut)
                            .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?
                            .stream_snapshot_id =
                            Some(snapshot.provenance.market_snapshot_id.clone());
                        control.quote_source_access_binding = Some(job.lease.binding.clone());
                        control.quote_observations.insert(
                            job.lease.projection.instrument_id.clone(),
                            AcceptedQuote {
                                binding: job.lease.binding.clone(),
                                snapshot,
                                failure: None,
                            },
                        );
                        let mut source =
                            configured_entry(&control, &job.lease.binding.workspace_id)?;
                        source.status = crate::protocol::DataSourceStatus::Available;
                        source.checked_at = Some(now.wall_clock.clone());
                        source.observed_at = Some(now.wall_clock.clone());
                        source.verified_at = Some(now.wall_clock);
                        source.availability_reason="Authenticated quote stream and metadata verified. Freshness and financial guards remain separate.".into();
                        control.data_source_observations.insert(
                            (job.lease.binding.workspace_id.clone(), "OD-001".into()),
                            source,
                        );
                        drop(control);
                        transition(
                            &job,
                            HotQuoteStatus::Streaming,
                            true,
                            true,
                            "Actual quote received on the authenticated, confirmed subscription.",
                        )?;
                    }
                    Some("error") => {
                        return Err(TradeXError::new(match frame["code"].as_u64() {
                            Some(401 | 402) => "PROVIDER_AUTH_FAILED",
                            Some(403) => "PROVIDER_ALREADY_AUTHENTICATED",
                            Some(404) => "PROVIDER_AUTH_TIMEOUT",
                            Some(406) => "PROVIDER_CONNECTION_LIMIT",
                            Some(405) => "PROVIDER_SYMBOL_LIMIT",
                            Some(407) => "PROVIDER_STREAM_STALE",
                            Some(409) => "DATA_SOURCE_FEED_DENIED",
                            Some(500) => "PROVIDER_UNAVAILABLE",
                            _ => "PROVIDER_RESPONSE_INVALID",
                        }));
                    }
                    _ => return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID")),
                }
            }
        }
        Ok(())
    })();
    if let Some(mut socket) = socket {
        let _ = socket.close(None);
    }
    result
}
fn mark_failed(job: &Job, error: &TradeXError) {
    // A terminal failure never promotes retained data or silently changes the selected feed.
    if let Ok(mut control) = job.control.lock()
        && job.lease.current(&control)
    {
        control.quote_source_access_binding = Some(job.lease.binding.clone());
        if let Ok(mut source) = configured_entry(&control, &job.lease.binding.workspace_id) {
            source.status = crate::protocol::DataSourceStatus::Unavailable;
            source.verified_at = None;
            source.availability_reason = format!(
                "Selected quote stream failed ({}). No feed fallback occurred.",
                error.code
            );
            control.data_source_observations.insert(
                (job.lease.binding.workspace_id.clone(), "OD-001".into()),
                source,
            );
        }
        for accepted in control.quote_observations.values_mut() {
            accepted.failure = Some(error.code.clone());
        }
        if let Some(lease) = control
            .hot_quote
            .as_mut()
            .and_then(HotLeaseState::stock_mut)
        {
            lease.projection.status = HotQuoteStatus::Failed;
            lease.projection.authenticated = false;
            lease.projection.subscribed = false;
            lease.projection.reason = format!(
                "Quote stream failed ({}); release and retry the selected feed.",
                error.code
            );
            lease.projection.sequence = lease
                .projection
                .sequence
                .saturating_add(1)
                .min(MAX_SEQUENCE);
        }
    }
}

fn run(job: &mut Job, stop: &AtomicBool) {
    while current(&job, stop) {
        let error = match run_connection(&job, stop) {
            Ok(()) => return,
            Err(error) => error,
        };
        if error.code == "STATE_VERSION_CONFLICT" || !current(&job, stop) {
            return;
        }
        let attempt = job.lease.projection.reconnect_attempt;
        let retryable = matches!(
            error.code.as_str(),
            "PROVIDER_UNAVAILABLE" | "PROVIDER_STREAM_STALE"
        );
        if !retryable || attempt >= 3 {
            mark_failed(&job, &error);
            return;
        }
        let rebound = (|| -> Result<()> {
            let mut control = job
                .control
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
            if !job.lease.current(&control) {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            control.quote_source_access_binding = Some(job.lease.binding.clone());
            if let Ok(mut source) = configured_entry(&control, &job.lease.binding.workspace_id) {
                source.status = crate::protocol::DataSourceStatus::Unavailable;
                source.verified_at = None;
                source.availability_reason = format!(
                    "Selected stream disconnected ({}); reconnecting the same feed. Retained quotes are stale.",
                    error.code
                );
                control.data_source_observations.insert(
                    (job.lease.binding.workspace_id.clone(), "OD-001".into()),
                    source,
                );
            }
            for accepted in control.quote_observations.values_mut() {
                accepted.failure = Some(error.code.clone());
            }
            let generation = uuid::Uuid::new_v4().to_string();
            control.quote_connection_generation = generation.clone();
            let lease = control
                .hot_quote
                .as_mut()
                .and_then(HotLeaseState::stock_mut)
                .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?;
            lease.binding.connection_generation = generation.clone();
            lease.stream_snapshot_id = None;
            lease.projection.connection_generation = generation;
            lease.projection.reconnect_attempt = attempt + 1;
            lease.projection.authenticated = false;
            lease.projection.subscribed = false;
            lease.projection.status = HotQuoteStatus::Reconnecting;
            lease.projection.sequence = lease
                .projection
                .sequence
                .checked_add(1)
                .filter(|n| *n <= MAX_SEQUENCE)
                .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
            lease.projection.reason = format!(
                "Retry {} of 3 on the selected feed after {}. Retained quotes are stale.",
                attempt + 1,
                error.code
            );
            job.lease = lease.clone();
            Ok(())
        })();
        if let Err(error) = rebound {
            if error.code != "STATE_VERSION_CONFLICT" {
                mark_failed(&job, &error);
            }
            return;
        }
        // One worker owns all attempts; cancellation is checked while waiting, with no engine lock held.
        let deadline = Instant::now() + Duration::from_millis(500u64 << attempt);
        while Instant::now() < deadline {
            if !current(&job, stop) {
                return;
            }
            thread::sleep(
                Duration::from_millis(50).min(deadline.saturating_duration_since(Instant::now())),
            );
        }
    }
}
