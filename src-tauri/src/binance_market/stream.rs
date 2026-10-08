//! Fixed ordinary Spot transport adapter; no execution credentials or renderer URLs.
use crate::protocol::{Result, TradeXError};
#[derive(Clone, Default)]
pub struct BinanceStreamConnector {
    #[cfg(feature = "integration-test")]
    loopback: Option<String>,
}
impl BinanceStreamConnector {
    #[cfg(feature = "integration-test")]
    pub fn for_loopback_test(url: &str) -> Result<Self> {
        let parsed =
            reqwest::Url::parse(url).map_err(|_| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
        if parsed.scheme() != "ws"
            || !parsed.host_str().is_some_and(|h| {
                h.parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
            })
            || parsed.port().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || !["/ws/btcusdt@depth@100ms", "/ws/ethusdt@depth@100ms"].contains(&parsed.path())
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        Ok(Self {
            loopback: Some(url.into()),
        })
    }
    pub(crate) fn connect(
        &self,
        symbol: &str,
        current: &(impl Fn() -> bool + Sync),
    ) -> Result<crate::quote_source::transport::Socket> {
        if !matches!(symbol, "BTCUSDT" | "ETHUSDT") {
            return Err(TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        let path = format!("/ws/{}@depth@100ms", symbol.to_ascii_lowercase());
        let url = format!("wss://stream.binance.com:9443{path}");
        #[cfg(feature = "integration-test")]
        let url = self.loopback.clone().unwrap_or(url);
        let parsed =
            reqwest::Url::parse(&url).map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        if parsed.path() != path {
            return Err(TradeXError::new("DATA_SOURCE_FEED_DENIED"));
        }
        crate::provider_io::binance_read_budget::reserve_stream_connection()?;
        crate::quote_source::transport::connect(&url, 524_288, current)
    }
}

use crate::provider_io::ProviderHttp;
use crate::{
    ControlPlane,
    protocol::{HotQuoteAcquire, HotQuoteProjection, HotQuoteStatus, Instrument, MAX_SEQUENCE},
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

fn provider_time(event_time: i64, receipt: &str) -> Result<String> {
    let observed =
        time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(event_time) * 1_000_000)
            .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
    let first =
        time::OffsetDateTime::parse(receipt, &time::format_description::well_known::Rfc3339)
            .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
    if observed > first || first - observed > time::Duration::seconds(30) {
        return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
    }
    observed
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))
}

#[derive(Clone)]
pub(crate) struct Lease {
    session: String,
    clock_generation: String,
    manager_stop: Arc<AtomicBool>,
    pub(crate) projection: HotQuoteProjection,
    pub(crate) stream_snapshot_id: Option<String>,
}
impl Lease {
    pub(crate) fn current(&self, control: &ControlPlane) -> bool {
        !self.manager_stop.load(Ordering::Acquire) && self.owns(control)
    }
    fn owns(&self, control: &ControlPlane) -> bool {
        control.session == self.session
            && control.time.generation() == self.clock_generation
            && control
                .require_workspace(&self.projection.workspace_id)
                .is_ok()
            && control.store.as_ref().is_some_and(|store| {
                store.binance_market_source().is_ok_and(|saved| {
                    saved.configured
                        && saved.state_version(&self.projection.workspace_id)
                            == self.projection.source_version
                })
            })
            && control.hot_quote.as_ref().is_some_and(|owner| {
                let p = owner.projection();
                p.source_id == super::SOURCE_ID
                    && p.lease_id == self.projection.lease_id
                    && p.generation == self.projection.generation
                    && p.connection_generation == self.projection.connection_generation
                    && !matches!(p.status, HotQuoteStatus::Closed | HotQuoteStatus::Failed)
            })
    }
}

pub(crate) struct Job {
    control: Arc<Mutex<ControlPlane>>,
    lease: Lease,
    http: Arc<dyn ProviderHttp + Send + Sync>,
    connector: BinanceStreamConnector,
    symbol: &'static str,
    manager_stop: Arc<AtomicBool>,
}
impl Job {
    pub(crate) fn prepare(
        control: &mut ControlPlane,
        shared: Arc<Mutex<ControlPlane>>,
        input: &HotQuoteAcquire,
        instrument: Instrument,
        http: Arc<dyn ProviderHttp + Send + Sync>,
        connector: BinanceStreamConnector,
        manager_stop: Arc<AtomicBool>,
    ) -> Result<Self> {
        let symbol = match instrument.instrument_id.as_str() {
            "crypto:BTC/USDT:spot" => "BTCUSDT",
            "crypto:ETH/USDT:spot" => "ETHUSDT",
            _ => return Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
        };
        control.require_workspace(&input.workspace_id)?;
        let source = control.store.as_ref().unwrap().binance_market_source()?;
        if source.state_version(&input.workspace_id) != input.expected_source_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if !source.configured {
            return Err(TradeXError::new("DATA_SOURCE_UNKNOWN"));
        }
        let generation = uuid::Uuid::new_v4().to_string();
        let lease = Lease { session: control.session.clone(), clock_generation: control.time.generation().into(), manager_stop: manager_stop.clone(), stream_snapshot_id: None,
            projection: HotQuoteProjection { workspace_id:input.workspace_id.clone(), source_id:super::SOURCE_ID.into(), instrument_id:instrument.instrument_id,
                lease_id:uuid::Uuid::new_v4().to_string(), generation:generation.clone(), connection_generation:generation,
                reconnect_attempt:0,source_version:source.state_version(&input.workspace_id),sequence:0,status:HotQuoteStatus::Connecting,authenticated:false,subscribed:false,
                reason:"Connecting to the selected ordinary public Spot stream. A continuous book is still required.".into(),detail:None } };
        control.hot_quote = Some(crate::quote_source::hot::HotLeaseState::Binance(
            lease.clone(),
        ));
        Ok(Self {
            control: shared,
            lease,
            http,
            connector,
            symbol,
            manager_stop,
        })
    }
    fn current(&self, stop: &AtomicBool) -> bool {
        !stop.load(Ordering::Acquire) && self.control.lock().is_ok_and(|c| self.lease.current(&c))
    }
    fn transition(&self, status: HotQuoteStatus, reason: &str) -> Result<()> {
        let mut control = self
            .control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if !self.lease.current(&control)
            && !(status == HotQuoteStatus::Closed && self.lease.owns(&control))
        {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let Some(crate::quote_source::hot::HotLeaseState::Binance(lease)) =
            control.hot_quote.as_mut()
        else {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        };
        lease.projection.sequence = lease
            .projection
            .sequence
            .checked_add(1)
            .filter(|v| *v <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        lease.projection.status = status;
        lease.projection.authenticated = false;
        lease.projection.subscribed = false;
        lease.projection.reason = reason.into();
        Ok(())
    }
    pub(crate) fn run(&mut self, stop: &AtomicBool) {
        while self.current(stop) {
            let error = match self.run_connection(stop) {
                Ok(()) => return,
                Err(error) => error,
            };
            if !self.current(stop) || error.code == "STATE_VERSION_CONFLICT" {
                return;
            }
            let attempt = self.lease.projection.reconnect_attempt;
            let retryable = matches!(
                error.code.as_str(),
                "PROVIDER_UNAVAILABLE"
                    | "PROVIDER_STREAM_GAP"
                    | "PROVIDER_RESPONSE_INVALID"
                    | "PROVIDER_BACKPRESSURE"
                    | "PROVIDER_STREAM_SHUTDOWN"
            );
            if !retryable || attempt >= 3 {
                let _ = self.transition(HotQuoteStatus::Failed,&format!("Continuous Spot book unavailable ({}). Retry the selected source; retained observations are unavailable.",error.code));
                return;
            }
            if self.reconnect(attempt + 1, &error.code).is_err() {
                return;
            }
            let until = Instant::now() + Duration::from_millis(250 * (1u64 << attempt));
            while Instant::now() < until {
                if !self.current(stop) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    fn reconnect(&mut self, attempt: u32, reason: &str) -> Result<()> {
        let mut control = self
            .control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if !self.lease.current(&control) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let Some(crate::quote_source::hot::HotLeaseState::Binance(lease)) =
            control.hot_quote.as_mut()
        else {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        };
        lease.projection.sequence = lease
            .projection
            .sequence
            .checked_add(1)
            .filter(|v| *v <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        lease.projection.connection_generation = uuid::Uuid::new_v4().to_string();
        lease.projection.reconnect_attempt = attempt;
        lease.projection.status = HotQuoteStatus::Reconnecting;
        lease.projection.authenticated = false;
        lease.projection.subscribed = false;
        lease.stream_snapshot_id = None;
        lease.projection.reason = format!(
            "Retry {attempt} of 3 after {reason}; the new connection requires its own snapshot and continuity proof. Retained liquidity is unavailable."
        );
        self.lease = lease.clone();
        Ok(())
    }
    fn snapshot_job(&self) -> std::sync::mpsc::Receiver<Result<Vec<u8>>> {
        let control = self.control.clone();
        let lease = self.lease.clone();
        let http = self.http.clone();
        let stop = self.manager_stop.clone();
        let symbol = self.symbol;
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let current =
                || !stop.load(Ordering::Acquire) && control.lock().is_ok_and(|c| lease.current(&c));
            let read = |path: &str, weight: u32| -> Result<Vec<u8>> {
                if !current() {
                    return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                }
                crate::provider_io::binance_read_budget::reserve(None, weight)?;
                let (response, rate) = crate::provider_io::with_provider_slot(
                    "binance",
                    crate::provider_io::ProviderPriority::P3,
                    &current,
                    || {
                        http.get_response_with_rate_limit(
                            crate::provider_io::ProviderEndpoint::BinanceLive,
                            path,
                            reqwest::header::HeaderMap::new(),
                        )
                    },
                )??;
                if matches!(response.status, 418 | 429) {
                    crate::provider_io::record_provider_retry_after(
                        "binance",
                        None,
                        rate.and_then(|r| r.retry_after_seconds),
                    );
                }
                if !current() {
                    return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                }
                if response.status != 200 || response.body.len() > 524_288 {
                    return Err(TradeXError::new(match response.status {
                        401 => "PROVIDER_AUTH_FAILED",
                        403 => "DATA_SOURCE_FEED_DENIED",
                        418 => "PROVIDER_IP_BANNED",
                        429 => "PROVIDER_RATE_LIMITED",
                        _ => "PROVIDER_RESPONSE_INVALID",
                    }));
                }
                Ok(response.body)
            };
            let result = (|| {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase", deny_unknown_fields)]
                struct Clock {
                    server_time: i64,
                }
                let started = Instant::now();
                let clock: Clock = crate::provider_json::strict_json(&read("/api/v3/time", 1)?)?;
                let now =
                    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64;
                if started.elapsed() > Duration::from_secs(2)
                    || clock.server_time <= 0
                    || now.abs_diff(clock.server_time) > 5000
                {
                    return Err(TradeXError::new("CLOCK_SKEW"));
                }
                read(&format!("/api/v3/depth?symbol={symbol}&limit=1000"), 50)
            })();
            let _ = sender.send(result);
        });
        receiver
    }
    fn publish(
        &self,
        book: &super::book::Book,
        event_time: i64,
        receipt: &str,
        received: Instant,
    ) -> Result<()> {
        use crate::protocol::{
            BinanceSpotQuoteEvidence, DataSourceStatus, MarketEntitlement, MarketFreshness,
            MarketSnapshot, MarketSnapshotProvenance, QuoteDepthUnit,
        };
        let (bid, ask) = book.best()?;
        let (floor, ceiling, bids, asks) = book.projection();
        let provider = provider_time(event_time, receipt)?;
        let (known_bid_levels, known_ask_levels, material_hash) = book.material(self.symbol);
        let evidence = BinanceSpotQuoteEvidence {
            workspace_id: self.lease.projection.workspace_id.clone(),
            session_id: self.lease.session.clone(),
            time_generation: self.lease.clock_generation.clone(),
            lease_id: self.lease.projection.lease_id.clone(),
            environment: crate::protocol::SpotMarketEnvironment::Ordinary,
            depth_coverage: crate::protocol::SpotDepthCoverage::KnownPriceBands,
            known_bid_levels,
            known_ask_levels,
            material_hash,
            provider_event_time_ms: event_time.to_string(),
            provider_symbol: self.symbol.into(),
            base_asset: if self.symbol == "BTCUSDT" {
                "BTC"
            } else {
                "ETH"
            }
            .into(),
            quote_asset: "USDT".into(),
            source_version: self.lease.projection.source_version.clone(),
            connection_generation: self.lease.projection.connection_generation.clone(),
            book_update_id: book.id.to_string(),
            depth_unit: QuoteDepthUnit::Base,
            bid_known_floor: floor,
            ask_known_ceiling: ceiling,
            bids,
            asks,
            data_use_rights: DataSourceStatus::Unverified,
        };
        let snapshot = MarketSnapshot {
            instrument_id: self.lease.projection.instrument_id.clone(),
            last_price: None,
            bid: Some(bid.price),
            ask: Some(ask.price),
            bid_size: Some(bid.quantity),
            ask_size: Some(ask.quantity),
            provenance: MarketSnapshotProvenance {
                source: super::SOURCE_ID.into(),
                venue: Some("BINANCE".into()),
                provider_timestamp: provider,
                received_timestamp: receipt.into(),
                market_snapshot_id: uuid::Uuid::new_v4().to_string(),
                entitlement: MarketEntitlement::Realtime,
                freshness: MarketFreshness::Healthy,
                alpaca: None,
                binance: Some(evidence),
            },
        };
        let mut control = self
            .control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if !self.lease.current(&control) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let Some(crate::quote_source::hot::HotLeaseState::Binance(lease)) =
            control.hot_quote.as_mut()
        else {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        };
        lease.projection.sequence = lease
            .projection
            .sequence
            .checked_add(1)
            .filter(|s| *s <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        lease.projection.status = HotQuoteStatus::Streaming;
        lease.projection.authenticated = false;
        lease.projection.subscribed = true;
        lease.projection.reason="Verified continuous public Spot book within known price bands; financial data-use rights remain unverified.".into();
        lease.stream_snapshot_id = Some(snapshot.provenance.market_snapshot_id.clone());
        control.binance_market_observations.insert(
            snapshot.instrument_id.clone(),
            Observation {
                lease: self.lease.clone(),
                snapshot,
                received,
            },
        );
        Ok(())
    }
    fn advance_unchanged(&self) -> Result<()> {
        let mut control = self
            .control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if !self.lease.current(&control) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let Some(crate::quote_source::hot::HotLeaseState::Binance(lease)) =
            control.hot_quote.as_mut()
        else {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        };
        lease.projection.sequence = lease
            .projection
            .sequence
            .checked_add(1)
            .filter(|s| *s <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        Ok(())
    }
    fn run_connection(&self, stop: &AtomicBool) -> Result<()> {
        if !self.current(stop) {
            return Ok(());
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut socket = self
            .connector
            .connect(self.symbol, &|| self.current(stop))?;
        let ready = AtomicBool::new(false);
        crate::quote_source::transport::guarded_read(
            &mut socket,
            &|| self.current(stop) && (ready.load(Ordering::Acquire) || Instant::now() < deadline),
            |socket| self.read_connection(socket, stop, deadline, &ready),
        )
    }
    fn read_connection(
        &self,
        socket: &mut crate::quote_source::transport::Socket,
        stop: &AtomicBool,
        deadline: Instant,
        ready: &AtomicBool,
    ) -> Result<()> {
        self.transition(
            HotQuoteStatus::AwaitingQuote,
            "Public stream connected; awaiting verified snapshot and depth continuity.",
        )?;
        let mut receiver: Option<std::sync::mpsc::Receiver<Result<Vec<u8>>>> = None;
        let mut book: Option<super::book::Book> = None;
        let mut candidate: Option<super::book::Book> = None;
        let mut resnapshot_used = false;
        let mut buffer: std::collections::VecDeque<(super::book::Event, String, Instant, usize)> =
            std::collections::VecDeque::new();
        let mut buffered_bytes = 0usize;
        let mut controls = std::collections::VecDeque::<Instant>::new();
        while self.current(stop) {
            if book.is_none() && Instant::now() > deadline {
                return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
            }
            if let Some(rx) = receiver.as_ref() {
                match rx.try_recv() {
                    Ok(result) => {
                        candidate = Some(super::book::Book::snapshot(&result?)?);
                        receiver = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => (),
                    Err(_) => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
                }
            }
            if let Some(pending) = candidate.as_mut() {
                while buffer
                    .front()
                    .is_some_and(|(event, _, _, _)| event.u <= pending.id)
                {
                    if let Some((_, _, _, bytes)) = buffer.pop_front() {
                        buffered_bytes -= bytes;
                    }
                }
                if let Some((first, _, _, _)) = buffer.front() {
                    if !pending.bridge(first) {
                        if resnapshot_used {
                            return Err(TradeXError::new("PROVIDER_STREAM_GAP"));
                        }
                        // A REST snapshot may lag an already-buffering stream.
                        // Keep those original frames/receipts and reread once,
                        // under the same total bootstrap deadline and budgets.
                        resnapshot_used = true;
                        candidate = None;
                        receiver = Some(self.snapshot_job());
                        continue;
                    }
                    let mut published = false;
                    while let Some((event, receipt, received, bytes)) = buffer.pop_front() {
                        buffered_bytes -= bytes;
                        let time = event.time;
                        if let Some(changed) = pending.apply(event)? {
                            if !published || changed {
                                self.publish(pending, time, &receipt, received)?;
                                published = true;
                            } else {
                                self.advance_unchanged()?;
                            }
                        }
                    }
                    book = candidate.take();
                    ready.store(true, Ordering::Release);
                }
            }
            match socket.read() {
                Ok(tungstenite::Message::Text(text)) => {
                    let (receipt, received) = {
                        let mut c = self
                            .control
                            .lock()
                            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
                        (
                            c.time
                                .status(&self.lease.projection.workspace_id)?
                                .wall_clock,
                            Instant::now(),
                        )
                    };
                    let event = match super::book::StreamEvent::parse(text.as_bytes(), self.symbol)?
                    {
                        super::book::StreamEvent::Depth(event) => event,
                        super::book::StreamEvent::Shutdown(time) => {
                            provider_time(time, &receipt)?;
                            return Err(TradeXError::new("PROVIDER_STREAM_SHUTDOWN"));
                        }
                    };
                    provider_time(event.time, &receipt)?;
                    if let Some(book) = book.as_mut() {
                        let time = event.time;
                        if let Some(changed) = book.apply(event)? {
                            if changed {
                                self.publish(book, time, &receipt, received)?;
                            } else {
                                self.advance_unchanged()?;
                            }
                        }
                    } else {
                        buffered_bytes = buffered_bytes.saturating_add(text.len());
                        if buffer.len() >= 256 || buffered_bytes > 4 * 1024 * 1024 {
                            return Err(TradeXError::new("PROVIDER_BACKPRESSURE"));
                        }
                        buffer.push_back((event, receipt, received, text.len()));
                        if receiver.is_none() && candidate.is_none() {
                            receiver = Some(self.snapshot_job());
                        }
                    }
                }
                Ok(tungstenite::Message::Ping(payload)) => {
                    let now = Instant::now();
                    while controls.front().is_some_and(|at| {
                        now.saturating_duration_since(*at) >= Duration::from_secs(1)
                    }) {
                        controls.pop_front();
                    }
                    // Reserve one of the provider's five controls/second. Raw
                    // exact-symbol subscriptions send no JSON controls. Stop
                    // before flushing a queued automatic Pong over this limit.
                    if controls.len() >= 4 {
                        return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
                    }
                    controls.push_back(now);
                    socket
                        .send(tungstenite::Message::Pong(payload))
                        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                }
                Ok(tungstenite::Message::Pong(_)) => (),
                Ok(tungstenite::Message::Close(_)) => {
                    return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
                }
                Ok(_) => return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID")),
                Err(tungstenite::Error::Io(error))
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    ()
                }
                Err(_) => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
            }
        }
        Ok(())
    }
    pub(crate) fn retire_stopped(&self) {
        if self.control.lock().is_ok_and(|c| self.lease.owns(&c)) {
            let _ = self.transition(
                HotQuoteStatus::Closed,
                "Quote connection manager stopped. No current book is available.",
            );
        }
    }
}

#[derive(Clone)]
pub(crate) struct Observation {
    lease: Lease,
    snapshot: crate::protocol::MarketSnapshot,
    received: Instant,
}
pub(crate) fn project_source(
    control: &ControlPlane,
    version: &str,
    source: &mut crate::protocol::DataSourceEntry,
) {
    use crate::protocol::DataSourceStatus;
    let active = control
        .hot_quote
        .as_ref()
        .map(|owner| owner.projection())
        .filter(|p| p.source_id == super::SOURCE_ID && p.source_version == version);
    let observation =
        active.and_then(|p| control.binance_market_observations.get(&p.instrument_id));
    let Some(observation) = observation else {
        if let Some(active) = active {
            source.availability_reason = active.reason.clone();
            if matches!(
                active.status,
                HotQuoteStatus::Failed
                    | HotQuoteStatus::Closed
                    | HotQuoteStatus::Reconnecting
                    | HotQuoteStatus::Stale
            ) {
                source.status = DataSourceStatus::Unavailable;
            }
        }
        return;
    };
    if observation.lease.projection.source_version
        != active.map(|p| p.source_version.as_str()).unwrap_or("")
    {
        return;
    }
    source.observed_at = Some(observation.snapshot.provenance.received_timestamp.clone());
    let time_status = control
        .time
        .preview_status(&observation.lease.projection.workspace_id)
        .ok();
    let now = time_status.as_ref().and_then(|s| {
        time::OffsetDateTime::parse(
            &s.wall_clock,
            &time::format_description::well_known::Rfc3339,
        )
        .ok()
    });
    let fresh = [
        &observation.snapshot.provenance.provider_timestamp,
        &observation.snapshot.provenance.received_timestamp,
    ]
    .iter()
    .all(|value| {
        time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
            .is_ok_and(|t| {
                now.is_some_and(|now| {
                    now - t >= time::Duration::ZERO && now - t <= time::Duration::seconds(30)
                })
            })
    }) && observation.received.elapsed() <= Duration::from_secs(30)
        && time_status.is_some_and(|s| s.confidence == crate::protocol::TimeConfidence::Trusted)
        && observation.lease.current(control)
        && active.is_some_and(|p| p.status == HotQuoteStatus::Streaming);
    source.status = if fresh {
        DataSourceStatus::Available
    } else {
        DataSourceStatus::Unavailable
    };
    source.availability_reason = if fresh {
        "Current continuous public Spot collection within known bands. Financial use, retention, redistribution, commercial use and regional eligibility remain UNVERIFIED."
    } else {
        "Retained Spot observation is unavailable. Open or retry a supported Hot view; cached reads do not renew its first receipt."
    }.into();
}
pub(crate) fn project(
    control: &ControlPlane,
    now: &crate::protocol::TimeStatus,
    detail: &mut crate::protocol::MarketDetail,
) {
    use crate::protocol::{MarketDataStatus, MarketFreshness, MarketTier, TimeConfidence};
    if detail.instrument.asset_class != crate::protocol::AssetClass::CryptoSpot
        || detail.tier == MarketTier::Cold
    {
        return;
    }
    if !super::selected_once(control).unwrap_or(true) {
        return;
    }
    detail.source_id = Some(super::SOURCE_ID.into());
    let Some(observation) = control
        .binance_market_observations
        .get(&detail.instrument.instrument_id)
    else {
        detail.snapshot = None;
        detail.status = MarketDataStatus::Unavailable;
        detail.availability_reason=control.hot_quote.as_ref().and_then(|lease|lease.missing_quote_reason(&detail.instrument.instrument_id)).unwrap_or("The selected public Spot source has no verified continuous book. Open the supported Hot view.").into();
        return;
    };
    let mut snapshot = observation.snapshot.clone();
    let timestamp = |value: &str| {
        time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()
    };
    let fresh = match (
        timestamp(&now.wall_clock),
        timestamp(&snapshot.provenance.provider_timestamp),
        timestamp(&snapshot.provenance.received_timestamp),
    ) {
        (Some(n), Some(p), Some(r)) => [n - p, n - r]
            .iter()
            .all(|age| *age >= time::Duration::ZERO && *age <= time::Duration::seconds(30)),
        _ => false,
    } && observation.received.elapsed() <= Duration::from_secs(30)
        && observation.lease.current(control)
        && control
            .hot_quote
            .as_ref()
            .is_some_and(|lease| lease.projection().status == HotQuoteStatus::Streaming);
    snapshot.provenance.freshness = if now.confidence != TimeConfidence::Trusted {
        MarketFreshness::ClockUncertain
    } else if fresh {
        MarketFreshness::Healthy
    } else {
        MarketFreshness::Stale
    };
    detail.status = if fresh && now.confidence == TimeConfidence::Trusted {
        MarketDataStatus::Available
    } else {
        MarketDataStatus::Unavailable
    };
    detail.availability_reason=if detail.status==MarketDataStatus::Available{"Current continuous public Spot book within known price bands. Data-use rights, rules and all financial authority remain separate."}else{"Retained Spot book is unavailable for current decisions. Reacquire the selected source; cached reads do not renew its first receipt."}.into();
    detail.snapshot = Some(snapshot);
}
