#![cfg(feature = "integration-test")]
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tradex::binance_market::stream::BinanceStreamConnector;
use tradex::quote_source::hot::{QuoteHotSupervisor, QuoteStreamConnectors, StockStreamConnector};
use tradex::{
    ControlPlane,
    protocol::Result,
    provider_io::{CredentialVault, Credentials, ProviderEndpoint, ProviderHttp},
};

#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod account_fixtures;

struct NoTradingKey;
impl CredentialVault for NoTradingKey {
    fn put(&self, _: &str, _: &Credentials) -> Result<()> {
        panic!("Public Spot source must never store a trading credential")
    }
    fn get(&self, _: &str) -> Result<Credentials> {
        panic!("Public Spot source must never read a trading credential")
    }
    fn remove(&self, _: &str) -> Result<()> {
        panic!("Public Spot source must never delete a trading credential")
    }
}
struct NoPrivateHttp;
impl ProviderHttp for NoPrivateHttp {
    fn get(&self, _: ProviderEndpoint, _: &str, _: reqwest::header::HeaderMap) -> Result<Vec<u8>> {
        Err(tradex::protocol::TradeXError::new("PROVIDER_UNAVAILABLE"))
    }
}
fn command(control: &Arc<Mutex<ControlPlane>>, name: &str, payload: Value) -> Value {
    control.lock().unwrap().dispatch_with_events(
        json!({"requestId":"spot-hot","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}

#[test]
fn configured_public_spot_source_can_acquire_a_hot_lease_without_execution_credentials() {
    let folder = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(folder.path().into())));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let source = command(
        &control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    let selected = command(
        &control,
        "data.binance_market.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"]}),
    );
    assert_eq!(selected["ok"], true, "{selected}");
    assert_eq!(
        command(
            &control,
            "time.revalidate",
            json!({"workspaceId":workspace})
        )["ok"],
        true
    );
    // Every supported transport path in this test is explicitly loopback-only.
    let peer = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let connectors = QuoteStreamConnectors {
        stock: StockStreamConnector::default(),
        binance: BinanceStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/ws/btcusdt@depth@100ms",
            peer.local_addr().unwrap().port()
        ))
        .unwrap(),
    };
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"spot-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","expectedSourceVersion":selected["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),Arc::new(NoPrivateHttp),connectors);
    assert_eq!(
        acquired["ok"], true,
        "Configured public Spot Hot leases are unsupported: {acquired}"
    );
    assert_eq!(acquired["data"]["sourceId"], "BINANCE_SPOT_PUBLIC");
    assert_eq!(
        acquired["data"]["authenticated"], false,
        "Public access must not invent authentication"
    );
    assert!(
        acquired["data"]["detail"]["snapshot"].is_null(),
        "A lease without a continuous book is not a quote"
    );
    supervisor.stop_all();
}

struct DepthPeer {
    symbol: &'static str,
    sent_events: Arc<std::sync::atomic::AtomicUsize>,
    responses: Arc<std::sync::atomic::AtomicUsize>,
    stop: Arc<std::sync::atomic::AtomicBool>,
    threads: Vec<std::thread::JoinHandle<()>>,
    http: Arc<tradex::provider_io::BrokerHttp>,
    connector: BinanceStreamConnector,
    requests: Arc<Mutex<Vec<String>>>,
    events: std::sync::mpsc::Sender<Value>,
    connections: Arc<std::sync::atomic::AtomicUsize>,
    pings: std::sync::mpsc::Sender<Vec<u8>>,
    pongs: Arc<Mutex<Vec<Vec<u8>>>>,
}
impl DepthPeer {
    fn new() -> Self {
        Self::with_initial(None)
    }
    fn with_initial(initial: Option<Value>) -> Self {
        Self::with_snapshots(initial, Vec::new())
    }
    fn with_snapshots(initial: Option<Value>, snapshots: Vec<Value>) -> Self {
        Self::with_symbol("BTCUSDT", initial, snapshots)
    }
    fn with_symbol(symbol: &'static str, initial: Option<Value>, snapshots: Vec<Value>) -> Self {
        Self::with_clock(symbol, initial, snapshots, 0)
    }
    fn with_clock(
        symbol: &'static str,
        initial: Option<Value>,
        snapshots: Vec<Value>,
        clock_offset_ms: i64,
    ) -> Self {
        Self::with_protocol(symbol, initial, snapshots, clock_offset_ms, None)
    }
    fn with_snapshot_gate() -> (Self, std::sync::mpsc::Sender<()>) {
        let (release, gate) = std::sync::mpsc::channel();
        (
            Self::with_protocol("BTCUSDT", None, Vec::new(), 0, Some(gate)),
            release,
        )
    }
    fn with_protocol(
        symbol: &'static str,
        initial: Option<Value>,
        snapshots: Vec<Value>,
        clock_offset_ms: i64,
        mut snapshot_gate: Option<std::sync::mpsc::Receiver<()>>,
    ) -> Self {
        assert!(matches!(symbol, "BTCUSDT" | "ETHUSDT"));
        use std::{
            io::{Read, Write},
            sync::atomic::{AtomicBool, Ordering},
            time::{Duration, Instant},
        };
        const START: u64 = 9_007_199_254_740_992;
        let http_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let ws_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        http_listener.set_nonblocking(true).unwrap();
        ws_listener.set_nonblocking(true).unwrap();
        let http = Arc::new(
            tradex::provider_io::BrokerHttp::for_loopback_test(&format!(
                "http://127.0.0.1:{}",
                http_listener.local_addr().unwrap().port()
            ))
            .unwrap(),
        );
        let connector = BinanceStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/ws/{}@depth@100ms",
            ws_listener.local_addr().unwrap().port(),
            symbol.to_ascii_lowercase()
        ))
        .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let sent_events = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let responses = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let response_ledger = responses.clone();
        let halt = stop.clone();
        let ledger = requests.clone();
        let rest = std::thread::spawn(move || {
            let mut snapshots = std::collections::VecDeque::from(snapshots);
            let deadline = Instant::now() + Duration::from_secs(8);
            while !halt.load(Ordering::Acquire) && Instant::now() < deadline {
                let (mut socket, _) = match http_listener.accept() {
                    Ok(v) => v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(e) => panic!("{e}"),
                };
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_millis(500)))
                    .unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    socket.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                    assert!(request.len() < 16_384);
                }
                let request = String::from_utf8(request).unwrap();
                assert!(!request.to_ascii_lowercase().contains("x-mbx-apikey"));
                let path = request
                    .lines()
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_owned();
                ledger.lock().unwrap().push(path.clone());
                let body = match path.as_str() {
                    "/api/v3/time" => {
                        json!({"serverTime":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64+clock_offset_ms})
                    }
                    path if path == format!("/api/v3/depth?symbol={symbol}&limit=1000") => {
                        if let Some(gate) = snapshot_gate.take() {
                            let _ = gate.recv_timeout(Duration::from_secs(5));
                        }
                        snapshots.pop_front().unwrap_or_else(|| if symbol=="BTCUSDT" {
                            json!({"lastUpdateId":START,"bids":[["60000","1"],["59900","2"]],"asks":[["60001","1"],["60100","2"]]})
                        } else {
                            json!({"lastUpdateId":START,"bids":[["3500","1"],["3499","2"]],"asks":[["3501","1"],["3502","2"]]})
                        })
                    }
                    _ => panic!("Unexpected public source request {path}"),
                };
                let body = serde_json::to_vec(&body).unwrap();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                if socket
                    .write_all(response.as_bytes())
                    .and_then(|_| socket.write_all(&body))
                    .is_err()
                {
                    continue;
                }
                response_ledger.fetch_add(1, Ordering::AcqRel);
            }
        });
        let halt = stop.clone();
        let connection_ledger = connections.clone();
        let event_ledger = sent_events.clone();
        let (events, next_event) = std::sync::mpsc::channel::<Value>();
        let (pings, next_ping) = std::sync::mpsc::channel::<Vec<u8>>();
        let pongs = Arc::new(Mutex::new(Vec::new()));
        let pong_ledger = pongs.clone();
        let stream = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(8);
            while !halt.load(Ordering::Acquire) && Instant::now() < deadline {
                let (tcp, _) = loop {
                    match ws_listener.accept() {
                        Ok(v) => break v,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            if halt.load(Ordering::Acquire) || Instant::now() > deadline {
                                return;
                            }
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(e) => panic!("{e}"),
                    }
                };
                tcp.set_nonblocking(false).unwrap();
                tcp.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut socket = tungstenite::accept_hdr(
                    tcp,
                    |request: &tungstenite::handshake::server::Request,
                     response: tungstenite::handshake::server::Response| {
                        assert_eq!(
                            request.uri().path(),
                            format!("/ws/{}@depth@100ms", symbol.to_ascii_lowercase())
                        );
                        assert!(!request.headers().contains_key("x-mbx-apikey"));
                        Ok(response)
                    },
                )
                .unwrap();
                connection_ledger.fetch_add(1, Ordering::AcqRel);
                socket
                    .get_mut()
                    .set_read_timeout(Some(Duration::from_millis(200)))
                    .unwrap();
                let event = initial.clone().unwrap_or_else(||json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":symbol,"U":START,"u":START+1,"b":[[if symbol=="BTCUSDT" {"60000"} else {"3500"},"0.5"]],"a":[[if symbol=="BTCUSDT" {"60001"} else {"3501"},"0.75"]]}));
                socket
                    .send(tungstenite::Message::Text(event.to_string().into()))
                    .unwrap();
                event_ledger.fetch_add(1, Ordering::AcqRel);
                while !halt.load(Ordering::Acquire) && Instant::now() < deadline {
                    while let Ok(ping) = next_ping.try_recv() {
                        socket
                            .send(tungstenite::Message::Ping(ping.into()))
                            .unwrap();
                    }
                    while let Ok(event) = next_event.try_recv() {
                        if socket
                            .send(tungstenite::Message::Text(event.to_string().into()))
                            .is_err()
                        {
                            return;
                        }
                        event_ledger.fetch_add(1, Ordering::AcqRel);
                    }
                    match socket.read() {
                        Ok(tungstenite::Message::Close(_)) => break,
                        Ok(tungstenite::Message::Pong(payload)) => {
                            pong_ledger.lock().unwrap().push(payload.to_vec())
                        }
                        Ok(_) => (),
                        Err(tungstenite::Error::Io(e))
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) =>
                        {
                            ()
                        }
                        Err(_) => break,
                    }
                }
            }
        });
        Self {
            symbol,
            sent_events,
            responses,
            stop,
            threads: vec![rest, stream],
            http,
            connector,
            requests,
            events,
            connections,
            pings,
            pongs,
        }
    }
}

struct HotView {
    _folder: tempfile::TempDir,
    control: Arc<Mutex<ControlPlane>>,
    supervisor: QuoteHotSupervisor,
    peer: DepthPeer,
    workspace: Value,
    query: Value,
}
impl HotView {
    fn open() -> Self {
        Self::with_peer(DepthPeer::new())
    }
    fn with_peer(peer: DepthPeer) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(folder.path().into())));
        let workspace =
            command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let source = command(
            &control,
            "data.binance_market.connection",
            json!({"workspaceId":workspace}),
        );
        let selected = command(
            &control,
            "data.binance_market.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"]}),
        );
        assert_eq!(selected["ok"], true, "{selected}");
        command(
            &control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let supervisor = QuoteHotSupervisor::new();
        let instrument = if peer.symbol == "BTCUSDT" {
            "crypto:BTC/USDT:spot"
        } else {
            "crypto:ETH/USDT:spot"
        };
        let acquired = supervisor.dispatch_with(&control,&json!({"requestId":"depth-view","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":instrument,"expectedSourceVersion":selected["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:peer.connector.clone()});
        assert_eq!(acquired["ok"], true, "{acquired}");
        let query = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        Self {
            _folder: folder,
            control,
            supervisor,
            peer,
            workspace,
            query,
        }
    }
    fn wait(&self, ready: impl Fn(&Value) -> bool) -> Value {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let state = command(&self.control, "market.hot.get", self.query.clone());
            assert_eq!(state["ok"], true, "{state}");
            if ready(&state) {
                return state;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Hot condition not reached: {state}"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    fn acquire(&mut self, version: &Value) -> Value {
        let acquired=self.supervisor.dispatch_with(&self.control,&json!({"requestId":"replace-view","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":self.workspace,"instrumentId":"crypto:BTC/USDT:spot","expectedSourceVersion":version}}),"main",Arc::new(NoTradingKey),self.peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:self.peer.connector.clone()});
        assert_eq!(acquired["ok"], true, "{acquired}");
        self.query = json!({"workspaceId":self.workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        acquired
    }
    fn release(&self, query: &Value) -> Value {
        self.supervisor.dispatch_with(&self.control,&json!({"requestId":"release-view","schemaVersion":1,"command":"market.hot.release","payload":query}),"main",Arc::new(NoTradingKey),self.peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:self.peer.connector.clone()})
    }
}
impl Drop for HotView {
    fn drop(&mut self) {
        self.supervisor.stop_all();
    }
}

#[test]
fn unchanged_continuous_updates_advance_health_without_renewing_quote_identity_or_receipts() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let sequence = first["data"]["sequence"].as_u64().unwrap();
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,"b":[["60000.00","0.5000"]],"a":[["60001.00","0.7500"]]})).unwrap();
    let after = view.wait(|v| v["data"]["sequence"].as_u64().unwrap_or(0) > sequence);
    assert_eq!(
        after["data"]["detail"]["snapshot"], first["data"]["detail"]["snapshot"],
        "Unchanged material must retain its original identity, provider event time and first receipt"
    );
    let cached = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(
        cached["data"]["detail"]["snapshot"],
        first["data"]["detail"]["snapshot"]
    );
}

#[test]
fn stopping_the_manager_immediately_retires_current_liquidity_without_waiting_for_socket_io() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    view.supervisor.stop_all();
    let after = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(
        after["data"]["detail"]["status"], "UNAVAILABLE",
        "Stopped ownership cannot keep liquidity available while its socket read is finishing: {after}"
    );
    assert_eq!(
        after["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    assert_eq!(source["data"]["source"]["status"], "UNAVAILABLE");
}

#[test]
fn cached_public_books_expire_on_trusted_elapsed_time_and_never_renew_when_queried() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let workspace = view.workspace.as_str().unwrap();
    view.control
        .lock()
        .unwrap()
        .advance_test_clock_fixture(workspace, 31_001)
        .unwrap();
    let after = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(after["data"]["detail"]["status"], "UNAVAILABLE");
    assert_eq!(
        after["data"]["detail"]["snapshot"]["provenance"]["freshness"],
        "STALE"
    );
    for field in ["marketSnapshotId", "receivedTimestamp", "providerTimestamp"] {
        assert_eq!(
            after["data"]["detail"]["snapshot"]["provenance"][field],
            first["data"]["detail"]["snapshot"]["provenance"][field]
        );
    }
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    assert_eq!(
        source["data"]["source"]["status"], "UNAVAILABLE",
        "The same source registry cannot keep an expired material observation available: {source}"
    );
}

#[test]
fn a_missing_update_range_retires_the_old_book_and_rebootstraps_a_new_connection_generation() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_997u64,"u":9_007_199_254_740_997u64,"b":[["60000","0.25"]],"a":[]})).unwrap();
    let after = view.wait(|v| {
        v["data"]["status"] == "STREAMING"
            && v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]
    });
    assert_eq!(after["data"]["reconnectAttempt"], 1);
    assert_eq!(
        after["data"]["generation"], first["data"]["generation"],
        "View ownership is stable; each actual connection/bootstrap gets its own generation"
    );
    assert_ne!(
        after["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_eq!(
        after["data"]["detail"]["snapshot"]["bidSize"], "0.5",
        "The unbridged gap event cannot be applied to retained liquidity"
    );
    assert_eq!(
        view.peer
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.as_str() == "/api/v3/depth?symbol=BTCUSDT&limit=1000")
            .count(),
        2
    );
}

#[test]
fn repeated_unbridged_bootstraps_stop_after_three_retries_and_project_unavailability() {
    let peer = DepthPeer::with_initial(Some(
        json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_997u64,"u":9_007_199_254_740_997u64,"b":[],"a":[]}),
    ));
    let view = HotView::with_peer(peer);
    let failed = view.wait(|v| v["data"]["status"] == "FAILED");
    assert_eq!(failed["data"]["reconnectAttempt"], 3);
    assert!(
        failed["data"]["detail"]["snapshot"].is_null(),
        "No unbridged snapshot may become a quote: {failed}"
    );
    assert_eq!(
        view.peer
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.as_str() == "/api/v3/depth?symbol=BTCUSDT&limit=1000")
            .count(),
        8
    );
    assert_eq!(
        view.peer
            .connections
            .load(std::sync::atomic::Ordering::Acquire),
        4,
        "Each of four connections permits only one bounded extra REST snapshot"
    );
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    assert_eq!(
        source["data"]["source"]["status"], "UNAVAILABLE",
        "A failed acquisition must report its failure even when no quote was ever accepted: {source}"
    );
}

#[test]
fn a_snapshot_newer_than_buffered_updates_waits_for_its_bridge_without_reconnecting() {
    let view = HotView::with_peer(DepthPeer::with_initial(Some(
        json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_991u64,"u":9_007_199_254_740_992u64,"b":[],"a":[]}),
    )));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while !view
        .peer
        .requests
        .lock()
        .unwrap()
        .iter()
        .any(|p| p.as_str() == "/api/v3/depth?symbol=BTCUSDT&limit=1000")
    {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let pending = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(
        pending["data"]["reconnectAttempt"], 0,
        "Obsolete buffered events do not imply a stream gap: {pending}"
    );
    assert_eq!(pending["data"]["status"], "AWAITING_QUOTE");
    assert!(pending["data"]["detail"]["snapshot"].is_null());
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_992u64,"u":9_007_199_254_740_993u64,"b":[["60000","0.25"]],"a":[]})).unwrap();
    let ready = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_eq!(ready["data"]["detail"]["snapshot"]["bidSize"], "0.25");
    assert_eq!(ready["data"]["reconnectAttempt"], 0);
    assert_eq!(
        view.peer
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.as_str() == "/api/v3/depth?symbol=BTCUSDT&limit=1000")
            .count(),
        1
    );
}

#[test]
fn a_future_dated_unchanged_event_cannot_keep_the_original_connection_usable() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64+60_000,"s":"BTCUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,"b":[["60000","0.5"]],"a":[]})).unwrap();
    let retired = view.wait(|v| {
        v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]
            || v["data"]["status"] == "FAILED"
    });
    assert_eq!(
        retired["data"]["detail"]["status"], "UNAVAILABLE",
        "Future provider time must invalidate continuity even if the material is unchanged: {retired}"
    );
    assert_eq!(
        retired["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
}

#[test]
fn public_depth_provenance_binds_runtime_scope_and_bounded_known_material() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let evidence = &first["data"]["detail"]["snapshot"]["provenance"]["binance"];
    assert_eq!(
        evidence["workspaceId"], view.workspace,
        "Public quote provenance must bind its workspace: {evidence}"
    );
    assert_eq!(evidence["leaseId"], view.query["leaseId"]);
    assert_eq!(evidence["environment"], "ORDINARY");
    assert_eq!(evidence["depthCoverage"], "KNOWN_PRICE_BANDS");
    assert_eq!(evidence["knownBidLevels"], 2);
    assert_eq!(evidence["knownAskLevels"], 2);
    for field in ["sessionId", "timeGeneration"] {
        assert!(!evidence[field].as_str().unwrap_or("").is_empty());
    }
    let initial_hash = evidence["materialHash"].as_str().unwrap();
    assert_eq!(initial_hash.len(), 64);
    assert!(initial_hash.bytes().all(|b| b.is_ascii_hexdigit()));
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,"b":[["59900","0"]],"a":[]})).unwrap();
    let after = view.wait(|v| {
        v["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
            != first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    });
    let changed = &after["data"]["detail"]["snapshot"]["provenance"]["binance"];
    assert_eq!(changed["knownBidLevels"], 1);
    assert_eq!(
        changed["bidKnownFloor"], "59900",
        "Deleting the band boundary must not fabricate newly known deeper liquidity"
    );
    assert_ne!(changed["materialHash"], evidence["materialHash"]);
    assert_eq!(changed["dataUseRights"], "UNVERIFIED");
}

#[test]
fn obsolete_update_ids_do_not_hide_malformed_original_depth_values() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_993u64,"u":9_007_199_254_740_993u64,"b":[["60000","-1"]],"a":[]})).unwrap();
    let retired =
        view.wait(|v| v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]);
    assert_eq!(
        retired["data"]["detail"]["status"], "UNAVAILABLE",
        "Original malformed quantities must not be hidden by an obsolete cursor: {retired}"
    );
}

#[test]
fn a_snapshot_behind_the_first_buffered_update_is_reread_on_the_same_connection() {
    let event = json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_997u64,"u":9_007_199_254_740_998u64,"b":[["60000","0.5"]],"a":[]});
    let snapshots = [9_007_199_254_740_992u64,9_007_199_254_740_997u64].map(|id|json!({"lastUpdateId":id,"bids":[["60000","1"],["59900","2"]],"asks":[["60001","1"],["60100","2"]]}));
    let view = HotView::with_peer(DepthPeer::with_snapshots(Some(event), snapshots.into()));
    let ready = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_eq!(
        ready["data"]["reconnectAttempt"], 0,
        "A lagging snapshot needs bounded REST resnapshot on the already-buffering stream: {ready}"
    );
    assert_eq!(
        view.peer
            .connections
            .load(std::sync::atomic::Ordering::Acquire),
        1
    );
    assert_eq!(ready["data"]["detail"]["snapshot"]["bidSize"], "0.5");
    assert_eq!(
        ready["data"]["detail"]["snapshot"]["provenance"]["binance"]["bookUpdateId"],
        "9007199254740998"
    );
    assert_eq!(
        view.peer
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.as_str() == "/api/v3/depth?symbol=BTCUSDT&limit=1000")
            .count(),
        2
    );
}

#[test]
fn provider_ping_payloads_are_echoed_but_control_bursts_stop_before_exceeding_reserved_headroom() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let payloads = (0..6)
        .map(|n| format!("binance-ping-{n}").into_bytes())
        .collect::<Vec<_>>();
    for payload in &payloads {
        view.peer.pings.send(payload.clone()).unwrap();
    }
    let failed = view.wait(|v| v["data"]["status"] == "FAILED");
    assert_eq!(
        failed["data"]["reconnectAttempt"], 0,
        "A control limit must not automatically hammer a new connection"
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while view.peer.pongs.lock().unwrap().len() < 4 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(
        *view.peer.pongs.lock().unwrap(),
        payloads[..4],
        "Actual wire Pongs must preserve each payload and reserve headroom below5 controls/second"
    );
    assert_eq!(failed["data"]["detail"]["status"], "UNAVAILABLE");
    assert_eq!(
        failed["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        "Ping health cannot renew quote material"
    );
}

#[test]
fn a_typed_server_shutdown_retires_liquidity_with_a_distinct_bounded_reconnect_reason() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    view.peer.events.send(json!({"e":"serverShutdown","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})).unwrap();
    let retired =
        view.wait(|v| v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]);
    assert!(
        retired["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_STREAM_SHUTDOWN"),
        "A documented shutdown is distinct from malformed provider data: {retired}"
    );
    assert_eq!(retired["data"]["detail"]["status"], "UNAVAILABLE");
    let recovered = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_eq!(recovered["data"]["reconnectAttempt"], 1);
    assert_eq!(
        view.peer
            .connections
            .load(std::sync::atomic::Ordering::Acquire),
        2
    );
    assert_ne!(
        recovered["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
}

#[test]
fn original_depth_wire_types_unknown_fields_and_duplicate_prices_cannot_replace_good_material() {
    let cases: [(&str, fn(&mut Value)); 6] = [
        ("event time string", |v| v["E"] = json!("1791450000000")),
        ("floating update id", |v| {
            v["U"] = json!(9_007_199_254_740_994.0f64)
        }),
        ("update id string", |v| v["u"] = json!("9007199254740994")),
        ("numeric quantity", |v| v["b"] = json!([["60000", 0.5]])),
        ("duplicate canonical price", |v| {
            v["a"] = json!([["60001", "1"], ["60001.00", "2"]])
        }),
        ("unknown provider field", |v| {
            v["unsupportedAuthority"] = json!(true)
        }),
    ];
    for (name, corrupt) in cases {
        let view = HotView::open();
        let first = view.wait(|v| v["data"]["status"] == "STREAMING");
        let mut event = json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,"b":[["60000","0.25"]],"a":[]});
        corrupt(&mut event);
        view.peer.events.send(event).unwrap();
        let retired = view
            .wait(|v| v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]);
        assert!(
            retired["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("PROVIDER_RESPONSE_INVALID"),
            "{name} must fail because of original provider data, not a quota or unrelated fault: {retired}"
        );
        assert_eq!(retired["data"]["detail"]["status"], "UNAVAILABLE", "{name}");
        assert_eq!(
            retired["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
            first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
            "{name}"
        );
    }
}

#[test]
fn material_outside_the_displayed_twenty_levels_still_changes_immutable_depth_identity() {
    let bids = (0..21)
        .map(|n| json!([(60000 - n).to_string(), "2"]))
        .collect::<Vec<_>>();
    let snapshot = json!({"lastUpdateId":9_007_199_254_740_992u64,"bids":bids,"asks":[["60001","2"],["60100","2"]]});
    let view = HotView::with_peer(DepthPeer::with_snapshots(None, vec![snapshot]));
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let first_evidence = &first["data"]["detail"]["snapshot"]["provenance"]["binance"];
    assert_eq!(first_evidence["knownBidLevels"], 21);
    assert_eq!(first_evidence["bids"].as_array().unwrap().len(), 20);
    assert_eq!(first_evidence["bidKnownFloor"], "59980");
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,"b":[["59980","3"]],"a":[]})).unwrap();
    let after = view.wait(|v| {
        v["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
            != first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    });
    let changed = &after["data"]["detail"]["snapshot"]["provenance"]["binance"];
    assert_eq!(
        changed["bids"], first_evidence["bids"],
        "Displayed levels do not include the changed21st level"
    );
    assert_ne!(
        changed["materialHash"], first_evidence["materialHash"],
        "Identity covers all retained known depth, not only the UI slice"
    );
    assert_eq!(changed["depthCoverage"], "KNOWN_PRICE_BANDS");
    assert_eq!(changed["knownBidLevels"], 21);
}

#[test]
fn source_changes_old_releases_and_reopen_cannot_preserve_or_retire_another_current_lease() {
    let mut view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let old_query = view.query.clone();
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    let changed = command(
        &view.control,
        "data.binance_market.configure",
        json!({"workspaceId":view.workspace,"expectedStateVersion":source["data"]["stateVersion"]}),
    );
    assert_eq!(changed["ok"], true, "{changed}");
    assert_eq!(changed["data"]["source"]["status"], "UNVERIFIED");
    let retired = command(&view.control, "market.hot.get", old_query.clone());
    assert_eq!(retired["data"]["status"], "STALE");
    assert_eq!(retired["data"]["detail"]["status"], "UNAVAILABLE");
    view.acquire(&changed["data"]["stateVersion"]);
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_ne!(current["data"]["leaseId"], first["data"]["leaseId"]);
    assert_eq!(
        current["data"]["detail"]["snapshot"]["provenance"]["binance"]["sourceVersion"],
        changed["data"]["stateVersion"]
    );
    let late = view.release(&old_query);
    assert_eq!(late["ok"], true, "{late}");
    assert_eq!(late["data"]["released"], false);
    let after = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(
        after["data"]["status"], "STREAMING",
        "Old view cleanup cannot release its replacement: {after}"
    );
    assert_eq!(
        command(&view.control, "workspace.open", json!({}))["data"]["workspaceId"],
        view.workspace
    );
    assert_eq!(
        command(&view.control, "market.hot.get", view.query.clone())["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    let detail = command(
        &view.control,
        "market.get",
        json!({"workspaceId":view.workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"HOT"}),
    );
    assert_eq!(detail["ok"], true, "{detail}");
    assert!(
        detail["data"]["snapshot"].is_null(),
        "Reopen restores selection only: {detail}"
    );
    assert_eq!(
        command(
            &view.control,
            "data.binance_market.connection",
            json!({"workspaceId":view.workspace})
        )["data"]["configured"],
        true
    );
}
impl Drop for DepthPeer {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        for thread in self.threads.drain(..) {
            let result = thread.join();
            if !std::thread::panicking() {
                result.unwrap();
            }
        }
    }
}

#[test]
fn real_snapshot_and_diff_stream_publish_exact_time_bound_continuous_depth_without_a_key() {
    use std::time::{Duration, Instant};
    let folder = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(folder.path().into())));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let source = command(
        &control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    let selected = command(
        &control,
        "data.binance_market.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"]}),
    );
    assert_eq!(selected["ok"], true);
    command(
        &control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let peer = DepthPeer::new();
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"continuous-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","expectedSourceVersion":selected["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:peer.connector.clone()});
    assert_eq!(acquired["ok"], true, "{acquired}");
    let query = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut seen;
    loop {
        seen = command(&control, "market.hot.get", query.clone());
        assert_eq!(seen["ok"], true, "{seen}");
        if seen["data"]["detail"]["snapshot"].is_object() || Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let source_state = command(
        &control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    supervisor.stop_all();
    assert_eq!(
        source_state["data"]["source"]["status"], "AVAILABLE",
        "Actual continuous collection must be reflected by the same saved-source projection: {source_state}"
    );
    assert!(
        source_state["data"]["source"]["redistribution"]
            .as_str()
            .unwrap()
            .contains("UNVERIFIED")
    );
    let snapshot = &seen["data"]["detail"]["snapshot"];
    assert!(
        snapshot.is_object(),
        "An actual HTTP snapshot and diff event did not produce a current continuous book: {seen}"
    );
    assert_eq!(seen["data"]["status"], "STREAMING");
    assert_eq!(seen["data"]["authenticated"], false);
    assert_eq!(seen["data"]["subscribed"], true);
    assert_eq!(snapshot["bid"], "60000");
    assert_eq!(snapshot["ask"], "60001");
    assert_eq!(snapshot["bidSize"], "0.5");
    assert_eq!(snapshot["askSize"], "0.75");
    assert!(snapshot["lastPrice"].is_null());
    assert_eq!(snapshot["provenance"]["source"], "BINANCE_SPOT_PUBLIC");
    assert_eq!(snapshot["provenance"]["venue"], "BINANCE");
    assert_eq!(
        snapshot["provenance"]["binance"]["bookUpdateId"],
        "9007199254740993"
    );
    assert_eq!(snapshot["provenance"]["binance"]["depthUnit"], "BASE");
    assert_eq!(snapshot["provenance"]["binance"]["bidKnownFloor"], "59900");
    assert_eq!(
        snapshot["provenance"]["binance"]["askKnownCeiling"],
        "60100"
    );
    assert!(
        peer.requests
            .lock()
            .unwrap()
            .iter()
            .any(|p| p == "/api/v3/depth?symbol=BTCUSDT&limit=1000")
    );
}

#[test]
fn eth_depth_uses_its_exact_public_stream_rest_symbol_and_base_liquidity() {
    let view = HotView::with_peer(DepthPeer::with_symbol("ETHUSDT", None, Vec::new()));
    let state = view.wait(|v| v["data"]["status"] == "STREAMING");
    let snapshot = &state["data"]["detail"]["snapshot"];
    let evidence = &snapshot["provenance"]["binance"];
    assert_eq!(state["data"]["instrumentId"], "crypto:ETH/USDT:spot");
    assert_eq!(snapshot["bid"], "3500");
    assert_eq!(snapshot["ask"], "3501");
    assert_eq!(snapshot["bidSize"], "0.5");
    assert_eq!(snapshot["askSize"], "0.75");
    assert_eq!(evidence["providerSymbol"], "ETHUSDT");
    assert_eq!(evidence["baseAsset"], "ETH");
    assert_eq!(evidence["quoteAsset"], "USDT");
    assert_eq!(evidence["depthUnit"], "BASE");
    assert_eq!(evidence["bidKnownFloor"], "3499");
    assert_eq!(evidence["askKnownCeiling"], "3502");
    assert_eq!(evidence["bookUpdateId"], "9007199254740993");
    assert_eq!(snapshot["provenance"]["venue"], "BINANCE");
    assert_eq!(evidence["dataUseRights"], "UNVERIFIED");
    assert_eq!(state["data"]["detail"]["status"], "AVAILABLE");
    assert!(snapshot["lastPrice"].is_null());
    let paths = view.peer.requests.lock().unwrap();
    assert_eq!(
        paths.as_slice(),
        ["/api/v3/time", "/api/v3/depth?symbol=ETHUSDT&limit=1000"]
    );
}

#[test]
fn overlapping_updates_delete_levels_without_expanding_known_bands_or_renewing_unchanged_material()
{
    const START: u64 = 9_007_199_254_740_992;
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START,"u":START+2,"b":[["60000","0"],["59950","3"],["59800","900"]],"a":[["60001","0"],["60050","4"],["60200","900"]]})).unwrap();
    let replaced = view.wait(|v| {
        v["data"]["detail"]["snapshot"]["provenance"]["binance"]["bookUpdateId"]
            == "9007199254740994"
    });
    let snapshot = &replaced["data"]["detail"]["snapshot"];
    assert_eq!(snapshot["bid"], "59950");
    assert_eq!(snapshot["ask"], "60050");
    assert_eq!(snapshot["bidSize"], "3");
    assert_eq!(snapshot["askSize"], "4");
    assert_eq!(snapshot["provenance"]["binance"]["knownBidLevels"], 2);
    assert_eq!(snapshot["provenance"]["binance"]["knownAskLevels"], 2);
    assert_eq!(
        snapshot["provenance"]["binance"]["bids"],
        json!([{"price":"59950","quantity":"3"},{"price":"59900","quantity":"2"}])
    );
    assert_eq!(
        snapshot["provenance"]["binance"]["asks"],
        json!([{"price":"60050","quantity":"4"},{"price":"60100","quantity":"2"}])
    );
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START+3,"u":START+3,"b":[["59900","0"]],"a":[["60100","0"]]})).unwrap();
    let narrowed = view.wait(|v| {
        v["data"]["detail"]["snapshot"]["provenance"]["binance"]["bookUpdateId"]
            == "9007199254740995"
    });
    let material = &narrowed["data"]["detail"]["snapshot"];
    let evidence = &material["provenance"]["binance"];
    assert_eq!(evidence["knownBidLevels"], 1);
    assert_eq!(evidence["knownAskLevels"], 1);
    assert_eq!(evidence["bidKnownFloor"], "59900");
    assert_eq!(evidence["askKnownCeiling"], "60100");
    assert_ne!(
        material["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START+4,"u":START+4,"b":[["59800","1"]],"a":[["60200","1"]]})).unwrap();
    let unchanged =
        view.wait(|v| v["data"]["sequence"].as_u64() > narrowed["data"]["sequence"].as_u64());
    assert_eq!(
        unchanged["data"]["detail"]["snapshot"], *material,
        "Unproved outside bands cannot expand coverage or freshen known material"
    );
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START+5,"u":START+5,"b":[["59950","0"]],"a":[["60050","0"]]})).unwrap();
    let exhausted =
        view.wait(|v| v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]);
    assert_eq!(exhausted["data"]["detail"]["status"], "UNAVAILABLE");
    assert!(
        exhausted["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_RESPONSE_INVALID"),
        "{exhausted}"
    );
    assert_eq!(
        exhausted["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        material["provenance"]["marketSnapshotId"]
    );
}

#[test]
fn five_thousand_known_levels_remain_bounded_and_one_extra_level_retires_instead_of_truncating() {
    const START: u64 = 9_007_199_254_740_992;
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let additions = (1..=4998)
        .map(|i| json!([format!("59950.{i:04}"), "1"]))
        .collect::<Vec<_>>();
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START+2,"u":START+2,"b":additions,"a":[]})).unwrap();
    let full = view.wait(|v| {
        v["data"]["detail"]["snapshot"]["provenance"]["binance"]["knownBidLevels"] == 5000
    });
    assert_eq!(full["data"]["detail"]["status"], "AVAILABLE");
    let material = &full["data"]["detail"]["snapshot"];
    let evidence = &material["provenance"]["binance"];
    assert_eq!(evidence["bids"].as_array().unwrap().len(), 20);
    assert_eq!(evidence["knownAskLevels"], 2);
    assert_eq!(evidence["bidKnownFloor"], "59900");
    assert_eq!(material["bid"], "60000");
    assert_eq!(evidence["bids"][1]["price"], "59950.4998");
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START+3,"u":START+3,"b":[["59950.4999","1"]],"a":[]})).unwrap();
    let exhausted =
        view.wait(|v| v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]);
    assert!(
        exhausted["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_BACKPRESSURE"),
        "{exhausted}"
    );
    assert_eq!(exhausted["data"]["detail"]["status"], "UNAVAILABLE");
    assert_eq!(
        exhausted["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        material["provenance"]["marketSnapshotId"]
    );
    assert_eq!(
        exhausted["data"]["detail"]["snapshot"]["provenance"]["binance"]["knownBidLevels"], 5000,
        "No silent truncation may hide the capacity fault"
    );
}

#[test]
fn skewed_public_clock_samples_stop_before_depth_reads_or_claiming_quote_collection() {
    for offset in [-10_000, 10_000] {
        let view = HotView::with_peer(DepthPeer::with_clock("BTCUSDT", None, Vec::new(), offset));
        let state = view.wait(|v| v["data"]["status"] == "FAILED");
        assert!(
            state["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("CLOCK_SKEW"),
            "{state}"
        );
        assert_eq!(state["data"]["reconnectAttempt"], 0);
        assert_eq!(state["data"]["detail"]["status"], "UNAVAILABLE");
        assert!(state["data"]["detail"]["snapshot"].is_null());
        assert_eq!(
            view.peer.requests.lock().unwrap().as_slice(),
            ["/api/v3/time"],
            "A bad provider clock cannot start depth bootstrap"
        );
        assert_eq!(
            view.peer
                .connections
                .load(std::sync::atomic::Ordering::Acquire),
            1
        );
        let source = command(
            &view.control,
            "data.binance_market.connection",
            json!({"workspaceId":view.workspace}),
        );
        assert_eq!(source["data"]["configured"], true);
        assert_eq!(source["data"]["source"]["status"], "UNAVAILABLE");
        assert!(
            source["data"]["source"]["availabilityReason"]
                .as_str()
                .unwrap()
                .contains("CLOCK_SKEW"),
            "{source}"
        );
    }
}

#[test]
fn session_safety_and_time_revalidation_retire_old_books_before_explicit_reacquisition() {
    for reason in [
        "OS_SLEEP",
        "SESSION_INACTIVE",
        "SESSION_RESUMED",
        "TIME_REVALIDATE",
    ] {
        let mut view = HotView::open();
        let first = view.wait(|v| v["data"]["status"] == "STREAMING");
        let version = first["data"]["sourceVersion"].clone();
        let old_query = view.query.clone();
        if reason == "TIME_REVALIDATE" {
            assert_eq!(
                command(
                    &view.control,
                    "time.revalidate",
                    json!({"workspaceId":view.workspace})
                )["ok"],
                true
            );
        } else {
            // Public native safety handler; this does not assert physical OS Sleep/Wake.
            view.control
                .lock()
                .unwrap()
                .disarm_live_for_safety(reason)
                .unwrap();
        }
        let retired = command(&view.control, "market.hot.get", old_query.clone());
        assert_eq!(retired["data"]["status"], "STALE", "{reason}: {retired}");
        assert_eq!(retired["data"]["detail"]["status"], "UNAVAILABLE");
        assert_eq!(
            retired["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
            first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
        );
        let source = command(
            &view.control,
            "data.binance_market.connection",
            json!({"workspaceId":view.workspace}),
        );
        assert_eq!(source["data"]["stateVersion"], version);
        assert_eq!(source["data"]["configured"], true);
        assert_eq!(source["data"]["source"]["status"], "UNAVAILABLE");
        assert_eq!(
            command(
                &view.control,
                "time.revalidate",
                json!({"workspaceId":view.workspace})
            )["ok"],
            true
        );
        let new = view.acquire(&version);
        assert_ne!(new["data"]["leaseId"], first["data"]["leaseId"]);
        let recovered = view.wait(|v| v["data"]["status"] == "STREAMING");
        let material = &recovered["data"]["detail"]["snapshot"];
        assert_ne!(
            material["provenance"]["marketSnapshotId"],
            first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
        );
        assert_ne!(
            material["provenance"]["binance"]["timeGeneration"],
            first["data"]["detail"]["snapshot"]["provenance"]["binance"]["timeGeneration"]
        );
        assert_eq!(
            view.peer
                .connections
                .load(std::sync::atomic::Ordering::Acquire),
            2
        );
        assert_eq!(view.release(&old_query)["data"]["released"], false);
        let current = command(&view.control, "market.hot.get", view.query.clone());
        assert_eq!(current["data"]["detail"]["status"], "AVAILABLE");
    }
}

#[test]
fn bootstrap_frame_capacity_retires_before_a_late_snapshot_can_publish_into_a_new_source() {
    use std::{
        sync::atomic::Ordering,
        time::{Duration, Instant},
    };
    const START: u64 = 9_007_199_254_740_992;
    let (peer, resume) = DepthPeer::with_snapshot_gate();
    let view = HotView::with_peer(peer);
    let deadline = Instant::now() + Duration::from_secs(2);
    while view.peer.requests.lock().unwrap().len() < 2 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    for id in START + 2..=START + 256 {
        view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":id,"u":id,"b":[],"a":[]})).unwrap();
    }
    while view.peer.sent_events.load(Ordering::Acquire) < 256 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    std::thread::sleep(Duration::from_millis(150));
    let bounded = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(
        bounded["data"]["status"], "AWAITING_QUOTE",
        "256 real buffered frames are permitted: {bounded}"
    );
    assert!(bounded["data"]["detail"]["snapshot"].is_null());
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START+257,"u":START+257,"b":[],"a":[]})).unwrap();
    let exhausted = view.wait(|v| v["data"]["reconnectAttempt"] == 1);
    assert!(
        exhausted["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_BACKPRESSURE"),
        "{exhausted}"
    );
    assert_eq!(exhausted["data"]["detail"]["status"], "UNAVAILABLE");
    assert!(exhausted["data"]["detail"]["snapshot"].is_null());
    let started = Instant::now();
    let changed = command(
        &view.control,
        "data.binance_market.configure",
        json!({"workspaceId":view.workspace,"expectedStateVersion":bounded["data"]["sourceVersion"]}),
    );
    assert_eq!(changed["ok"], true, "{changed}");
    assert!(
        started.elapsed() < Duration::from_millis(750),
        "Pending HTTP must not hold the Control Plane lock"
    );
    view.supervisor.stop_all();
    resume.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while view.peer.responses.load(Ordering::Acquire) < 2 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    std::thread::sleep(Duration::from_millis(100));
    let detail = command(
        &view.control,
        "market.get",
        json!({"workspaceId":view.workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"HOT"}),
    );
    assert!(
        detail["data"]["snapshot"].is_null(),
        "Late actual HTTP cannot publish through retired ownership: {detail}"
    );
    assert_eq!(detail["data"]["status"], "UNAVAILABLE");
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    assert_eq!(
        source["data"]["stateVersion"],
        changed["data"]["stateVersion"]
    );
    assert_eq!(source["data"]["source"]["status"], "UNVERIFIED");
}

#[test]
fn bootstrap_byte_capacity_stops_large_valid_frames_before_the_frame_count_limit() {
    use std::{
        sync::atomic::Ordering,
        time::{Duration, Instant},
    };
    const START: u64 = 9_007_199_254_740_992;
    let (peer, resume) = DepthPeer::with_snapshot_gate();
    let view = HotView::with_peer(peer);
    let deadline = Instant::now() + Duration::from_secs(2);
    while view.peer.requests.lock().unwrap().len() < 2 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    let levels = (1..=500)
        .map(|i| {
            json!([
                format!("59950.{i:04}"),
                "1.0000000000000000000000000000000000001"
            ])
        })
        .collect::<Vec<_>>();
    let event = |id: u64| json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":id,"u":id,"b":levels,"a":[]});
    let frame_bytes = event(START + 2).to_string().len();
    assert!(frame_bytes < 512 * 1024);
    assert!(frame_bytes * 110 < 4 * 1024 * 1024);
    assert!(frame_bytes * 150 > 4 * 1024 * 1024);
    for id in START + 2..=START + 111 {
        view.peer.events.send(event(id)).unwrap();
    }
    while view.peer.sent_events.load(Ordering::Acquire) < 111 {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    std::thread::sleep(Duration::from_millis(250));
    let bounded = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(
        bounded["data"]["status"], "AWAITING_QUOTE",
        "Below4MiB and256frames remains a pending bootstrap: {bounded}"
    );
    assert!(bounded["data"]["detail"]["snapshot"].is_null());
    for id in START + 112..=START + 151 {
        view.peer.events.send(event(id)).unwrap();
    }
    let exhausted = view.wait(|v| v["data"]["reconnectAttempt"] == 1);
    let frames = view.peer.sent_events.load(Ordering::Acquire);
    assert!(
        frames < 256,
        "The byte limit must be independent of the256 frame limit: {frames}"
    );
    assert!(
        exhausted["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_BACKPRESSURE"),
        "{exhausted}"
    );
    assert_eq!(exhausted["data"]["detail"]["status"], "UNAVAILABLE");
    assert!(exhausted["data"]["detail"]["snapshot"].is_null());
    view.supervisor.stop_all();
    resume.send(()).unwrap();
}

fn connected_binance_proposal(view: &HotView) -> (Value, account_fixtures::Vault) {
    let vault = account_fixtures::Vault::default();
    let http = account_fixtures::Http::default();
    let request = json!({"requestId":"quote-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":view.workspace,"providerId":"binance","environment":"LIVE","label":"Quote consumer account"}});
    let job = view
        .control
        .lock()
        .unwrap()
        .prepare_provider_for(&request, "main")
        .unwrap()
        .unwrap();
    let observed = job.run(
        &vault,
        |_| account_fixtures::credentials(),
        &http,
        || view.control.lock().unwrap().provider_job_current(&job),
    );
    let tested = view
        .control
        .lock()
        .unwrap()
        .complete_provider(&job, observed);
    assert_eq!(tested["ok"], true, "{tested}");
    let account = command(
        &view.control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":view.workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":false}),
    );
    assert_eq!(account["ok"], true, "{account}");
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":account["data"]["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    (proposal["data"].clone(), vault)
}

fn collect_spot_admission(
    view: &HotView,
    proposal: &Value,
    vault: &account_fixtures::Vault,
    http: &account_fixtures::Http,
    instrument: &str,
) -> Value {
    let source = command(
        &view.control,
        "data.binance_rules.connection",
        json!({"workspaceId":view.workspace}),
    );
    let saved = command(
        &view.control,
        "data.binance_rules.configure",
        json!({"workspaceId":view.workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":proposal["fields"]["accountId"],"instrumentId":instrument}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let read = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"quote-admission","schemaVersion":1,"command":"data.binance_rules.refresh","payload":{"workspaceId":view.workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        vault,
        http,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    read["data"].clone()
}

#[test]
fn exact_account_spot_admission_qualifies_its_own_check_without_an_equity_calendar_or_order_authority()
 {
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, vault) = connected_binance_proposal(&view);
    let http = account_fixtures::Http {
        binance_rules_ui: true,
        ..Default::default()
    };
    let collected = collect_spot_admission(&view, &proposal, &vault, &http, "crypto:BTC/USDT:spot");
    assert!(
        collected["evidence"]["admissionBlockers"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        collected["capabilityStatuses"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["capability"] == "SPOT_ACCOUNT_ADMISSION")
            .unwrap()["status"],
        "AVAILABLE"
    );
    let detail = command(
        &view.control,
        "market.get",
        json!({"workspaceId":view.workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"HOT"}),
    );
    assert!(detail["data"]["marketState"]["calendarVersion"].is_null());
    assert_eq!(detail["data"]["marketState"]["session"], "UNKNOWN");
    let decision = command(
        &view.control,
        "risk.evaluate_proposal",
        json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    let checks = decision["data"]["checks"].as_array().unwrap();
    let admission = checks
        .iter()
        .find(|c| c["checkId"] == "MARKET_SESSION")
        .unwrap();
    assert_eq!(
        admission["outcome"], "PASS",
        "Actual exact-account Spot admission must not demand a fictional equity calendar: {admission}"
    );
    assert!(admission["reason"].as_str().unwrap().contains("admission"));
    for check in ["INSTRUMENT_RULES", "MARKET_DATA_USE", "ACCOUNT_ARMING"] {
        assert_eq!(
            checks.iter().find(|c| c["checkId"] == check).unwrap()["outcome"],
            "UNAVAILABLE",
            "{check}"
        );
    }
    assert_ne!(decision["data"]["status"], "ALLOWED");
}

fn assert_spot_admission_rejects_external_negative(mode: &str, blocker: &str, reason: &str) {
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, vault) = connected_binance_proposal(&view);
    let http = account_fixtures::Http {
        binance_rules_ui: true,
        ..Default::default()
    };
    match mode {
        "HALT" | "BREAK" => {
            let mut info = account_fixtures::binance_spot_exchange_info("BTCUSDT");
            info["symbols"][0]["status"] = json!(mode);
            *http.binance_exchange_info.borrow_mut() = Some(info);
        }
        "MAINTENANCE" => *http.binance_system_status.borrow_mut() = Some(json!({"status":1})),
        "API_LOCK" => {
            *http.binance_api_trading_status.borrow_mut() = Some(
                json!({"data":{"isLocked":true,"plannedRecoverTime":0,"updateTime":1547630471725u64}}),
            )
        }
        "PERMISSION" => {
            let mut info = account_fixtures::binance_spot_exchange_info("BTCUSDT");
            info["symbols"][0]["permissionSets"] = json!([["MARGIN"]]);
            *http.binance_exchange_info.borrow_mut() = Some(info);
        }
        _ => panic!("Unsupported external protocol case"),
    }
    let collected = collect_spot_admission(&view, &proposal, &vault, &http, "crypto:BTC/USDT:spot");
    assert!(
        collected["evidence"]["admissionBlockers"]
            .as_array()
            .unwrap()
            .contains(&json!(blocker))
    );
    let decision = command(
        &view.control,
        "risk.evaluate_proposal",
        json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    let checks = decision["data"]["checks"].as_array().unwrap();
    let admission = checks
        .iter()
        .find(|c| c["checkId"] == "MARKET_SESSION")
        .unwrap();
    assert_eq!(admission["outcome"], "REJECT", "{admission}");
    assert_eq!(admission["reasonCode"], reason, "{admission}");
    assert!(admission["reason"].as_str().unwrap().contains(blocker));
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_ne!(decision["data"]["status"], "ALLOWED");
}

#[test]
fn current_symbol_halt_rejects_spot_admission_despite_a_continuous_public_book() {
    assert_spot_admission_rejects_external_negative("HALT", "SYMBOL_NOT_TRADING", "MARKET_HALTED");
}

#[test]
fn current_symbol_break_rejects_spot_admission_despite_a_continuous_public_book() {
    assert_spot_admission_rejects_external_negative("BREAK", "SYMBOL_NOT_TRADING", "MARKET_HALTED");
}

#[test]
fn current_system_maintenance_rejects_spot_admission_despite_a_continuous_public_book() {
    assert_spot_admission_rejects_external_negative(
        "MAINTENANCE",
        "SYSTEM_MAINTENANCE",
        "INSTRUMENT_RULES_UNAVAILABLE",
    );
}

#[test]
fn unsatisfied_account_permission_sets_reject_spot_admission_despite_a_continuous_public_book() {
    assert_spot_admission_rejects_external_negative(
        "PERMISSION",
        "PERMISSION_SETS_NOT_SATISFIED",
        "INSTRUMENT_RULES_UNAVAILABLE",
    );
}

#[test]
fn current_api_trading_lock_rejects_spot_admission_despite_a_continuous_public_book() {
    assert_spot_admission_rejects_external_negative(
        "API_LOCK",
        "API_TRADING_LOCKED",
        "INSTRUMENT_RULES_UNAVAILABLE",
    );
}

fn save_spot_quote_policy(view: &HotView, slippage: &str, deviation: Option<&str>) {
    let current = command(
        &view.control,
        "risk.get_policy",
        json!({"workspaceId":view.workspace}),
    );
    let saved = command(
        &view.control,
        "risk.save_policy",
        json!({"workspaceId":view.workspace,"expectedStateVersion":current["data"]["stateVersion"],"policy":{"maxOrderNotional":null,"maxOrderQuantity":null,"maxPositionSize":null,"maxSingleInstrumentExposurePercent":null,"maxAssetClassExposurePercent":[],"maxDailyTradedNotional":null,"maxDailyRealizedLoss":null,"maxOpenOrders":null,"maxReservedCapital":null,"allowedInstrumentIds":[],"blockedInstrumentIds":[],"allowedVenues":[],"blockedVenues":[],"allowedAccountIds":[],"blockedAccountIds":[],"allowedEnvironments":[],"staleQuoteThresholdSeconds":3,"marketOrdersEnabled":true,"maxMarketOrderSlippagePercent":slippage,"maxPriceDeviationPercent":deviation,"liveInactivityTimeoutMinutes":20}}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
}

fn evaluate_spot_market_order(
    view: &HotView,
    account: &Value,
    side: &str,
    quantity: &str,
) -> Value {
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":account,"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":side,"orderType":"MARKET","quantity":{"type":"BASE","value":quantity},"limitPrice":null,"maximumSpend":null,"timeInForce":"GTC"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let decision = command(
        &view.control,
        "risk.evaluate_proposal",
        json!({"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    decision["data"].clone()
}

#[test]
fn displayed_spot_base_liquidity_qualifies_only_its_own_exact_side_size_check() {
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_eq!(current["data"]["detail"]["snapshot"]["bidSize"], "0.5");
    assert_eq!(current["data"]["detail"]["snapshot"]["askSize"], "0.75");
    let (proposal, _vault) = connected_binance_proposal(&view);
    save_spot_quote_policy(&view, "1", Some("1"));
    // Displayed best-side BASE quantities, not quote-currency amounts or deeper fills.
    for (side, quantity, expected) in [
        ("BUY", "0.75", "PASS"),
        ("BUY", "0.750000000000000001", "UNAVAILABLE"),
        ("SELL", "0.5", "PASS"),
        ("SELL", "0.500000000000000001", "UNAVAILABLE"),
    ] {
        let decision =
            evaluate_spot_market_order(&view, &proposal["fields"]["accountId"], side, quantity);
        let checks = decision["checks"].as_array().unwrap();
        let slippage = checks
            .iter()
            .find(|c| c["checkId"] == "MARKET_ORDER_SLIPPAGE")
            .unwrap();
        assert_eq!(
            slippage["outcome"], expected,
            "{side} {quantity}: {slippage}"
        );
        assert_eq!(
            checks
                .iter()
                .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
                .unwrap()["outcome"],
            "PASS"
        );
        for check in [
            "PRICE_DEVIATION",
            "INSTRUMENT_RULES",
            "MARKET_DATA_USE",
            "ACCOUNT_ARMING",
        ] {
            assert_eq!(
                checks.iter().find(|c| c["checkId"] == check).unwrap()["outcome"],
                "UNAVAILABLE",
                "{check}"
            );
        }
        assert_ne!(decision["status"], "ALLOWED");
    }
}

#[test]
fn exact_spot_spread_comparison_and_missing_last_trade_reference_remain_separate() {
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, _vault) = connected_binance_proposal(&view);
    // Bid60000/ask60001 gives adverse midpoint percentage100/120001,
    // strictly between0.0008 and0.0009. No float or rounded midpoint is needed.
    for (limit, expected) in [("0.0008", "REJECT"), ("0.0009", "PASS")] {
        save_spot_quote_policy(&view, limit, Some("1"));
        let decision =
            evaluate_spot_market_order(&view, &proposal["fields"]["accountId"], "BUY", "0.75");
        let checks = decision["checks"].as_array().unwrap();
        let slippage = checks
            .iter()
            .find(|c| c["checkId"] == "MARKET_ORDER_SLIPPAGE")
            .unwrap();
        assert_eq!(slippage["outcome"], expected, "{limit}: {slippage}");
        assert_ne!(decision["status"], "ALLOWED");
    }
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":proposal["fields"]}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let limit_proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(limit_proposal["ok"], true, "{limit_proposal}");
    let decision = command(
        &view.control,
        "risk.evaluate_proposal",
        json!({"workspaceId":view.workspace,"proposalId":limit_proposal["data"]["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    let checks = decision["data"]["checks"].as_array().unwrap();
    let deviation = checks
        .iter()
        .find(|c| c["checkId"] == "PRICE_DEVIATION")
        .unwrap();
    assert_eq!(
        deviation["outcome"], "UNAVAILABLE",
        "Bid/ask/midpoint are not an actual last-trade reference: {deviation}"
    );
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_ne!(decision["data"]["status"], "ALLOWED");
}

#[test]
fn other_symbol_rules_and_expired_rule_receipts_cannot_qualify_spot_admission() {
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, vault) = connected_binance_proposal(&view);
    let http = account_fixtures::Http {
        binance_rules_ui: true,
        ..Default::default()
    };
    let evaluate = || {
        command(
            &view.control,
            "risk.evaluate_proposal",
            json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
        )
    };
    let check = |decision: &Value| {
        decision["data"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["checkId"] == "MARKET_SESSION")
            .unwrap()
            .clone()
    };
    collect_spot_admission(&view, &proposal, &vault, &http, "crypto:ETH/USDT:spot");
    let mismatch = evaluate();
    assert_eq!(mismatch["ok"], true, "{mismatch}");
    assert_eq!(check(&mismatch)["outcome"], "UNAVAILABLE", "{mismatch}");
    collect_spot_admission(&view, &proposal, &vault, &http, "crypto:BTC/USDT:spot");
    assert_eq!(check(&evaluate())["outcome"], "PASS");
    view.control
        .lock()
        .unwrap()
        .advance_test_clock_fixture(view.workspace.as_str().unwrap(), 31_001)
        .unwrap();
    let expired = evaluate();
    assert_eq!(expired["ok"], true, "{expired}");
    assert_eq!(check(&expired)["outcome"], "UNAVAILABLE");
    assert_eq!(
        check(&expired)["reasonCode"],
        "INSTRUMENT_RULES_UNAVAILABLE"
    );
    assert_ne!(expired["data"]["status"], "ALLOWED");
}

#[test]
fn spot_admission_retires_when_the_public_account_refresh_changes_its_binding() {
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, vault) = connected_binance_proposal(&view);
    let http = account_fixtures::Http {
        binance_rules_ui: true,
        ..Default::default()
    };
    let collected = collect_spot_admission(&view, &proposal, &vault, &http, "crypto:BTC/USDT:spot");
    let account = command(
        &view.control,
        "account.get",
        json!({"workspaceId":view.workspace,"connectionId":proposal["fields"]["accountId"]}),
    );
    let request = json!({"requestId":"refresh-admitted-account","schemaVersion":1,"command":"provider.probe","payload":{"workspaceId":view.workspace,"connectionId":account["data"]["connectionId"],"expectedStateVersion":account["data"]["stateVersion"]}});
    let job = view
        .control
        .lock()
        .unwrap()
        .prepare_provider_for(&request, "main")
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| account_fixtures::credentials(),
        &http,
        || view.control.lock().unwrap().provider_job_current(&job),
    );
    let refreshed = view
        .control
        .lock()
        .unwrap()
        .complete_provider(&job, outcome);
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    assert_ne!(
        refreshed["data"]["stateVersion"],
        collected["evidence"]["binding"]["accountVersion"]
    );
    let decision = command(
        &view.control,
        "risk.evaluate_proposal",
        json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    let checks = decision["data"]["checks"].as_array().unwrap();
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "MARKET_SESSION")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_ne!(decision["data"]["status"], "ALLOWED");
}

#[test]
fn authentic_public_spot_depth_satisfies_only_quote_freshness_without_a_fictional_last_trade() {
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert!(current["data"]["detail"]["snapshot"]["lastPrice"].is_null());
    let (proposal, _vault) = connected_binance_proposal(&view);
    let decision = command(
        &view.control,
        "risk.evaluate_proposal",
        json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    let checks = decision["data"]["checks"].as_array().unwrap();
    let quote = checks
        .iter()
        .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
        .unwrap();
    assert_eq!(
        quote["outcome"], "PASS",
        "A actual continuous depth source needs no last trade to establish quote freshness: {quote}"
    );
    assert_eq!(quote["reasonCode"], "WITHIN_LIMIT");
    assert_ne!(
        decision["data"]["status"], "ALLOWED",
        "Technical collection does not establish financial authority"
    );
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "ACCOUNT_ARMING")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
    assert_ne!(
        checks
            .iter()
            .find(|c| c["checkId"] == "INSTRUMENT_RULES")
            .unwrap()["outcome"],
        "PASS"
    );
}

#[test]
fn selected_public_spot_source_is_explicit_in_current_and_retired_market_details() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_eq!(
        first["data"]["detail"]["sourceId"], "BINANCE_SPOT_PUBLIC",
        "Current detail must name its selected source independently of raw snapshot presence"
    );
    let changed = command(
        &view.control,
        "data.binance_market.configure",
        json!({"workspaceId":view.workspace,"expectedStateVersion":first["data"]["sourceVersion"]}),
    );
    assert_eq!(changed["ok"], true, "{changed}");
    let retired = command(
        &view.control,
        "market.get",
        json!({"workspaceId":view.workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"CENSUS"}),
    );
    assert_eq!(retired["data"]["sourceId"], "BINANCE_SPOT_PUBLIC");
    assert_eq!(retired["data"]["status"], "UNAVAILABLE");
    assert_eq!(
        retired["data"]["snapshot"]["provenance"]["freshness"],
        "STALE"
    );
}

#[test]
fn public_spot_collection_keeps_an_independent_unverified_data_use_gate_after_reads_and_source_changes()
 {
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, _vault) = connected_binance_proposal(&view);
    let evaluate = || {
        command(
            &view.control,
            "risk.evaluate_proposal",
            json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
        )
    };
    let decision = evaluate();
    assert_eq!(decision["ok"], true, "{decision}");
    let checks = decision["data"]["checks"].as_array().unwrap();
    let rights = checks
        .iter()
        .find(|c| c["checkId"] == "MARKET_DATA_USE")
        .expect("Technical access must have a separate financial data-use gate");
    assert_eq!(rights["outcome"], "UNAVAILABLE");
    assert_eq!(rights["reasonCode"], "DATA_USE_RIGHTS_UNVERIFIED");
    assert!(rights["reason"].as_str().unwrap().contains("UNVERIFIED"));
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_ne!(decision["data"]["status"], "ALLOWED");
    let changed = command(
        &view.control,
        "data.binance_market.configure",
        json!({"workspaceId":view.workspace,"expectedStateVersion":current["data"]["sourceVersion"]}),
    );
    assert_eq!(changed["ok"], true, "{changed}");
    let retired = evaluate();
    assert_eq!(retired["ok"], true, "{retired}");
    let checks = retired["data"]["checks"].as_array().unwrap();
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "MARKET_DATA_USE")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
}

#[test]
fn captured_spot_review_keeps_raw_depth_ephemeral_while_risk_history_restores_only_references() {
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, _vault) = connected_binance_proposal(&view);
    let query = json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]});
    let review = command(&view.control, "trade.request_approval", query.clone());
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(review["data"]["eligible"], false);
    assert_eq!(
        review["data"]["market"]["snapshot"]["provenance"]["marketSnapshotId"],
        current["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_eq!(
        review["data"]["market"]["snapshot"]["provenance"]["binance"]["dataUseRights"],
        "UNVERIFIED"
    );
    let before = command(&view.control, "risk.decision.list", query.clone());
    assert_eq!(before["ok"], true, "{before}");
    assert_eq!(before["data"]["decisions"].as_array().unwrap().len(), 1);
    let encoded = before["data"].to_string();
    for raw_field in [
        "bids",
        "asks",
        "bid",
        "ask",
        "bidSize",
        "askSize",
        "snapshot",
        "provenance",
    ] {
        assert!(
            !encoded.contains(&format!("\"{raw_field}\":")),
            "Persisted risk history must not retain raw public book fields: {raw_field}"
        );
    }
    let market_reference = before["data"]["decisions"][0]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "MARKET")
        .unwrap();
    assert!(
        market_reference["digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert_eq!(
        command(&view.control, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(&view.control, "risk.decision.list", query.clone());
    assert_eq!(restored["ok"], true, "{restored}");
    assert_eq!(
        restored["data"], before["data"],
        "Captured decision history remains the originally evaluated metadata"
    );
    let detail = command(
        &view.control,
        "market.get",
        json!({"workspaceId":view.workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"HOT"}),
    );
    assert_eq!(detail["data"]["sourceId"], "BINANCE_SPOT_PUBLIC");
    assert!(detail["data"]["snapshot"].is_null());
    command(
        &view.control,
        "time.revalidate",
        json!({"workspaceId":view.workspace}),
    );
    let fresh = command(&view.control, "risk.evaluate_proposal", query);
    assert_eq!(fresh["ok"], true, "{fresh}");
    let checks = fresh["data"]["checks"].as_array().unwrap();
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
    assert_eq!(
        checks
            .iter()
            .find(|c| c["checkId"] == "MARKET_DATA_USE")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
}

#[test]
fn protected_market_digest_keeps_unchanged_material_and_retires_on_depth_or_source_change() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, _vault) = connected_binance_proposal(&view);
    let evaluate = || {
        let reply = command(
            &view.control,
            "risk.evaluate_proposal",
            json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
        );
        assert_eq!(reply["ok"], true, "{reply}");
        reply["data"].clone()
    };
    let digest = |decision: &Value| {
        decision["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["kind"] == "MARKET")
            .unwrap()["digest"]
            .clone()
    };
    let initial = evaluate();
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,"b":[["60000","0.5"]],"a":[["60001","0.75"]]})).unwrap();
    let unchanged =
        view.wait(|v| v["data"]["sequence"].as_u64() > first["data"]["sequence"].as_u64());
    assert_eq!(
        unchanged["data"]["detail"]["snapshot"],
        first["data"]["detail"]["snapshot"]
    );
    assert_eq!(
        digest(&evaluate()),
        digest(&initial),
        "Advancing stream health cannot renew protected material"
    );
    view.peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_995u64,"u":9_007_199_254_740_995u64,"b":[["60000","0.4"]],"a":[]})).unwrap();
    view.wait(|v| {
        v["data"]["detail"]["snapshot"]["provenance"]["binance"]["bookUpdateId"]
            == "9007199254740995"
    });
    let changed = evaluate();
    assert_ne!(digest(&changed), digest(&initial));
    let configured = command(
        &view.control,
        "data.binance_market.configure",
        json!({"workspaceId":view.workspace,"expectedStateVersion":first["data"]["sourceVersion"]}),
    );
    assert_eq!(configured["ok"], true, "{configured}");
    let retired = evaluate();
    assert_ne!(digest(&retired), digest(&changed));
    assert_eq!(
        retired["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
}

#[test]
fn financial_quote_ttl_is_stricter_than_collection_age_and_cached_reads_cannot_renew_it() {
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, _vault) = connected_binance_proposal(&view);
    // Existing integration clock boundary advances both wall and monotonic time;
    // this is freshness logic proof, not physical clock or Sleep/Wake evidence.
    view.control
        .lock()
        .unwrap()
        .advance_test_clock_fixture(view.workspace.as_str().unwrap(), 4000)
        .unwrap();
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    assert_eq!(
        source["data"]["source"]["status"], "AVAILABLE",
        "Collection remains within30s"
    );
    let detail = command(
        &view.control,
        "market.get",
        json!({"workspaceId":view.workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"HOT"}),
    );
    assert_eq!(detail["data"]["status"], "AVAILABLE");
    assert_eq!(
        detail["data"]["snapshot"], first["data"]["detail"]["snapshot"],
        "Cache reads preserve the original event and receipt"
    );
    let decision = command(
        &view.control,
        "risk.evaluate_proposal",
        json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    let check = decision["data"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["checkId"] == "QUOTE_FRESHNESS")
        .unwrap();
    assert_eq!(check["outcome"], "UNAVAILABLE");
    assert_eq!(
        check["reasonCode"], "EVIDENCE_STALE",
        "Default financial TTL is3s, independent of collection30s: {check}"
    );
    let cached = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(
        cached["data"]["detail"]["snapshot"],
        first["data"]["detail"]["snapshot"]
    );
}
