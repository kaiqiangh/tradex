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
            "ws://127.0.0.1:{}/ws/btcusdt@depth@100ms",
            ws_listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
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
                        json!({"serverTime":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})
                    }
                    "/api/v3/depth?symbol=BTCUSDT&limit=1000" => {
                        snapshots.pop_front().unwrap_or_else(||json!({"lastUpdateId":START,"bids":[["60000","1"],["59900","2"]],"asks":[["60001","1"],["60100","2"]]}))
                    }
                    _ => panic!("Unexpected public source request {path}"),
                };
                let body = serde_json::to_vec(&body).unwrap();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                socket.write_all(response.as_bytes()).unwrap();
                socket.write_all(&body).unwrap();
            }
        });
        let halt = stop.clone();
        let connection_ledger = connections.clone();
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
                        assert_eq!(request.uri().path(), "/ws/btcusdt@depth@100ms");
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
                let event = initial.clone().unwrap_or_else(||json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":START,"u":START+1,"b":[["60000","0.5"]],"a":[["60001","0.75"]]}));
                socket
                    .send(tungstenite::Message::Text(event.to_string().into()))
                    .unwrap();
                while !halt.load(Ordering::Acquire) && Instant::now() < deadline {
                    while let Ok(ping) = next_ping.try_recv() {
                        socket
                            .send(tungstenite::Message::Ping(ping.into()))
                            .unwrap();
                    }
                    while let Ok(event) = next_event.try_recv() {
                        socket
                            .send(tungstenite::Message::Text(event.to_string().into()))
                            .unwrap();
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
        let acquired = supervisor.dispatch_with(&control,&json!({"requestId":"depth-view","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","expectedSourceVersion":selected["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:peer.connector.clone()});
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
