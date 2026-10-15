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

// Every external-provider scenario gets a fresh process because Binance's
// request and stream budgets, plus provider cooldowns, are process-wide.
fn isolated_hot_scenario(name: &str) -> bool {
    if std::env::var("TRADEX_HOT_SCENARIO").ok().as_deref() == Some(name) {
        return false;
    }
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([name, "--exact", "--nocapture", "--test-threads=1"])
        .env("TRADEX_HOT_SCENARIO", name)
        .output()
        .expect("Run isolated Hot scenario");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "Isolated Hot scenario failed: {name}"
    );
    true
}

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
    if isolated_hot_scenario(
        "configured_public_spot_source_can_acquire_a_hot_lease_without_execution_credentials",
    ) {
        return;
    }
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
    stream_url: String,
    http_status: Arc<std::sync::atomic::AtomicU16>,
    upgrade_status: Arc<std::sync::atomic::AtomicU16>,
    upgrade_attempts: Arc<std::sync::atomic::AtomicUsize>,
    requests: Arc<Mutex<Vec<String>>>,
    fee_metadata: Arc<Mutex<Value>>,
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
        assert!(matches!(symbol, "BTCUSDT" | "ETHUSDT" | "BNBUSDT"));
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
        let stream_url = format!(
            "ws://127.0.0.1:{}/ws/{}@depth@100ms",
            ws_listener.local_addr().unwrap().port(),
            symbol.to_ascii_lowercase()
        );
        let connector = BinanceStreamConnector::for_loopback_test(&stream_url).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let sent_events = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let responses = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let response_ledger = responses.clone();
        let http_status = Arc::new(std::sync::atomic::AtomicU16::new(200));
        let response_status = http_status.clone();
        let halt = stop.clone();
        let ledger = requests.clone();
        let fee_metadata = Arc::new(Mutex::new(
            json!({"symbols":[{"symbol":"BNBUSDT","baseAsset":"BNB","quoteAsset":"USDT","status":"TRADING"}]}),
        ));
        let metadata_response = fee_metadata.clone();
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
                    path if path == format!("/api/v3/exchangeInfo?symbol={symbol}") => {
                        if symbol == "BNBUSDT" {
                            metadata_response.lock().unwrap().clone()
                        } else {
                            json!({"symbols":[{"symbol":symbol,"baseAsset":if symbol=="BTCUSDT" {"BTC"}else {"ETH"},"quoteAsset":"USDT","status":"TRADING"}]})
                        }
                    }
                    path if path == format!("/api/v3/depth?symbol={symbol}&limit=1000") => {
                        if let Some(gate) = snapshot_gate.take() {
                            let _ = gate.recv_timeout(Duration::from_secs(5));
                        }
                        snapshots.pop_front().unwrap_or_else(|| if symbol=="BTCUSDT" {
                            json!({"lastUpdateId":START,"bids":[["60000","1"],["59900","2"]],"asks":[["60001","1"],["60100","2"]]})
                        } else if symbol=="ETHUSDT" {
                            json!({"lastUpdateId":START,"bids":[["3500","1"],["3499","2"]],"asks":[["3501","1"],["3502","2"]]})
                        } else {
                            json!({"lastUpdateId":START,"bids":[["500","1"],["499","2"]],"asks":[["501","1"],["502","2"]]})
                        })
                    }
                    _ => panic!("Unexpected public source request {path}"),
                };
                let body = serde_json::to_vec(&body).unwrap();
                let status = if path == "/api/v3/time" {
                    200
                } else {
                    response_status.load(Ordering::Acquire)
                };
                let response = format!(
                    "HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nRetry-After: 60\r\nLocation: /untrusted-redirect\r\nConnection: close\r\n\r\n",
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
        let upgrade_status = Arc::new(std::sync::atomic::AtomicU16::new(101));
        let upgrade_attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let upgrade_response = upgrade_status.clone();
        let upgrade_ledger = upgrade_attempts.clone();
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
                let upgraded = tungstenite::accept_hdr(
                    tcp,
                    |request: &tungstenite::handshake::server::Request,
                     response: tungstenite::handshake::server::Response| {
                        assert_eq!(
                            request.uri().path(),
                            format!("/ws/{}@depth@100ms", symbol.to_ascii_lowercase())
                        );
                        assert!(!request.headers().contains_key("x-mbx-apikey"));
                        upgrade_ledger.fetch_add(1, Ordering::AcqRel);
                        let status = upgrade_response.load(Ordering::Acquire);
                        if status != 101 {
                            return Err(tungstenite::http::Response::builder()
                                .status(status)
                                .body(None)
                                .unwrap());
                        }
                        Ok(response)
                    },
                );
                let mut socket = match upgraded {
                    Ok(socket) => socket,
                    Err(_) => continue,
                };
                connection_ledger.fetch_add(1, Ordering::AcqRel);
                socket
                    .get_mut()
                    .set_read_timeout(Some(Duration::from_millis(200)))
                    .unwrap();
                let event = initial.clone().unwrap_or_else(||json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":symbol,"U":START,"u":START+1,"b":[[if symbol=="BTCUSDT" {"60000"} else if symbol=="ETHUSDT" {"3500"} else {"500"},"0.5"]],"a":[[if symbol=="BTCUSDT" {"60001"} else if symbol=="ETHUSDT" {"3501"} else {"501"},"0.75"]]}));
                socket
                    .send(tungstenite::Message::Text(event.to_string().into()))
                    .unwrap();
                event_ledger.fetch_add(1, Ordering::AcqRel);
                'connection: while !halt.load(Ordering::Acquire) && Instant::now() < deadline {
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
                            break 'connection;
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
            stream_url,
            http_status,
            upgrade_status,
            upgrade_attempts,
            requests,
            fee_metadata,
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
    if isolated_hot_scenario(
        "unchanged_continuous_updates_advance_health_without_renewing_quote_identity_or_receipts",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "stopping_the_manager_immediately_retires_current_liquidity_without_waiting_for_socket_io",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "cached_public_books_expire_on_trusted_elapsed_time_and_never_renew_when_queried",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "a_missing_update_range_retires_the_old_book_and_rebootstraps_a_new_connection_generation",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "repeated_unbridged_bootstraps_stop_after_three_retries_and_project_unavailability",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "a_snapshot_newer_than_buffered_updates_waits_for_its_bridge_without_reconnecting",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "a_future_dated_unchanged_event_cannot_keep_the_original_connection_usable",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "public_depth_provenance_binds_runtime_scope_and_bounded_known_material",
    ) {
        return;
    }
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
    if isolated_hot_scenario("obsolete_update_ids_do_not_hide_malformed_original_depth_values") {
        return;
    }
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
    if isolated_hot_scenario(
        "a_snapshot_behind_the_first_buffered_update_is_reread_on_the_same_connection",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "provider_ping_payloads_are_echoed_but_control_bursts_stop_before_exceeding_reserved_headroom",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "a_typed_server_shutdown_retires_liquidity_with_a_distinct_bounded_reconnect_reason",
    ) {
        return;
    }
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
fn planned_socket_rotation_retires_the_old_book_before_a_new_bootstrap() {
    if isolated_hot_scenario("planned_socket_rotation_retires_the_old_book_before_a_new_bootstrap")
    {
        return;
    }
    let mut peer = DepthPeer::new();
    // This changes only a transport deadline; all quote material still comes
    // from the actual external HTTP and WebSocket peer.
    peer.connector = BinanceStreamConnector::for_loopback_test_with_rotation(
        &peer.stream_url,
        std::time::Duration::from_millis(750),
    )
    .unwrap();
    let view = HotView::with_peer(peer);
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let retired =
        view.wait(|v| v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]);
    assert!(
        retired["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_STREAM_SHUTDOWN"),
        "{retired}"
    );
    assert_eq!(retired["data"]["detail"]["status"], "UNAVAILABLE");
    assert_eq!(
        retired["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
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
    if isolated_hot_scenario(
        "original_depth_wire_types_unknown_fields_and_duplicate_prices_cannot_replace_good_material",
    ) {
        return;
    }
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
fn a_banned_websocket_upgrade_is_terminal_without_bootstrap_or_reconnect() {
    if isolated_hot_scenario(
        "a_banned_websocket_upgrade_is_terminal_without_bootstrap_or_reconnect",
    ) {
        return;
    }
    let peer = DepthPeer::new();
    peer.upgrade_status
        .store(418, std::sync::atomic::Ordering::Release);
    let view = HotView::with_peer(peer);
    let failed = view.wait(|v| v["data"]["status"] == "FAILED");
    assert!(
        failed["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_IP_BANNED"),
        "{failed}"
    );
    assert_eq!(failed["data"]["reconnectAttempt"], 0);
    assert!(failed["data"]["detail"]["snapshot"].is_null());
    assert_eq!(failed["data"]["detail"]["status"], "UNAVAILABLE");
    assert_eq!(
        view.peer
            .upgrade_attempts
            .load(std::sync::atomic::Ordering::Acquire),
        1
    );
    assert_eq!(
        view.peer
            .connections
            .load(std::sync::atomic::Ordering::Acquire),
        0
    );
    assert!(view.peer.requests.lock().unwrap().is_empty());
}

#[test]
fn http_source_denial_stops_without_retry_or_fallback() {
    if isolated_hot_scenario("http_source_denial_stops_without_retry_or_fallback") {
        return;
    }
    source_http_fault(403, "DATA_SOURCE_FEED_DENIED", false, 1);
}

#[test]
fn http_redirects_never_follow_an_untrusted_source_path() {
    if isolated_hot_scenario("http_redirects_never_follow_an_untrusted_source_path") {
        return;
    }
    source_http_fault(302, "PROVIDER_RESPONSE_INVALID", false, 4);
}

#[test]
fn public_http_ban_stops_and_its_shared_cooldown_survives_source_replacement() {
    if isolated_hot_scenario(
        "public_http_ban_stops_and_its_shared_cooldown_survives_source_replacement",
    ) {
        return;
    }
    source_http_fault(418, "PROVIDER_IP_BANNED", true, 1);
}

#[test]
fn public_http_rate_limit_stops_and_its_shared_cooldown_survives_source_replacement() {
    if isolated_hot_scenario(
        "public_http_rate_limit_stops_and_its_shared_cooldown_survives_source_replacement",
    ) {
        return;
    }
    source_http_fault(429, "PROVIDER_RATE_LIMITED", true, 1);
}

fn source_http_fault(status: u16, reason: &str, cooldown: bool, attempts: usize) {
    let peer = DepthPeer::new();
    peer.http_status
        .store(status, std::sync::atomic::Ordering::Release);
    let mut view = HotView::with_peer(peer);
    let failed = view.wait(|v| v["data"]["status"] == "FAILED");
    assert!(
        failed["data"]["reason"].as_str().unwrap().contains(reason),
        "{failed}"
    );
    assert_eq!(failed["data"]["reconnectAttempt"], attempts - 1);
    assert_eq!(failed["data"]["detail"]["status"], "UNAVAILABLE");
    assert!(failed["data"]["detail"]["snapshot"].is_null());
    assert_eq!(
        view.peer
            .connections
            .load(std::sync::atomic::Ordering::Acquire),
        attempts
    );
    assert_eq!(
        *view.peer.requests.lock().unwrap(),
        (0..attempts)
            .flat_map(|_| [
                "/api/v3/time".to_owned(),
                "/api/v3/depth?symbol=BTCUSDT&limit=1000".to_owned()
            ])
            .collect::<Vec<_>>()
    );
    if cooldown {
        view.peer
            .http_status
            .store(200, std::sync::atomic::Ordering::Release);
        let selected = command(
            &view.control,
            "data.binance_market.configure",
            json!({"workspaceId":view.workspace,"expectedStateVersion":failed["data"]["sourceVersion"]}),
        );
        assert_eq!(selected["ok"], true, "{selected}");
        view.acquire(&selected["data"]["stateVersion"]);
        view.wait(|v| v["data"]["status"] == "AWAITING_QUOTE");
        std::thread::sleep(std::time::Duration::from_millis(300));
        let waiting = command(&view.control, "market.hot.get", view.query.clone());
        assert_eq!(waiting["data"]["detail"]["status"], "UNAVAILABLE");
        assert!(waiting["data"]["detail"]["snapshot"].is_null());
        assert_eq!(
            view.peer.requests.lock().unwrap().len(),
            2,
            "source CAS must not reset provider/IP cooldown"
        );
        assert_eq!(view.release(&view.query)["ok"], true);
    }
}

#[test]
fn snapshot_original_id_types_and_per_side_capacity_never_become_quote_authority() {
    if isolated_hot_scenario(
        "snapshot_original_id_types_and_per_side_capacity_never_become_quote_authority",
    ) {
        return;
    }
    let valid = json!({"lastUpdateId":9_007_199_254_740_992u64,"bids":[["60000","1"],["59900","2"]],"asks":[["60001","1"],["60100","2"]]});
    let mut cases = Vec::new();
    for (name, id) in [
        ("string update id", json!("9007199254740992")),
        ("boolean update id", json!(true)),
        ("floating update id", json!(9_007_199_254_740_992.0f64)),
    ] {
        let mut snapshot = valid.clone();
        snapshot["lastUpdateId"] = id;
        cases.push((name, snapshot));
    }
    let mut over_capacity = valid;
    over_capacity["bids"] = json!(
        (0..1001)
            .map(|n| vec![(60000 - n).to_string(), "1".to_owned()])
            .collect::<Vec<_>>()
    );
    cases.push(("1001 snapshot levels on one side", over_capacity));
    for (name, snapshot) in cases {
        let view = HotView::with_peer(DepthPeer::with_snapshots(None, vec![snapshot; 4]));
        let failed = view.wait(|v| v["data"]["status"] == "FAILED");
        assert_eq!(failed["data"]["reconnectAttempt"], 3, "{name}: {failed}");
        assert!(
            failed["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("PROVIDER_RESPONSE_INVALID"),
            "{name}: {failed}"
        );
        assert_eq!(failed["data"]["detail"]["status"], "UNAVAILABLE", "{name}");
        assert!(
            failed["data"]["detail"]["snapshot"].is_null(),
            "{name}: unqualified material escaped bootstrap"
        );
        assert_eq!(
            view.peer
                .connections
                .load(std::sync::atomic::Ordering::Acquire),
            4,
            "{name}"
        );
    }
}

#[test]
fn an_oversized_actual_websocket_frame_retires_good_depth_before_rebootstrap() {
    if isolated_hot_scenario(
        "an_oversized_actual_websocket_frame_retires_good_depth_before_rebootstrap",
    ) {
        return;
    }
    let view = HotView::open();
    let first = view.wait(|v| v["data"]["status"] == "STREAMING");
    let quantity = format!("0.{}1", "0".repeat(60));
    let event = json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"s":"BTCUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,
        "b":(0..5000).map(|n|vec![(60000-n).to_string(),quantity.clone()]).collect::<Vec<_>>(),
        "a":(0..5000).map(|n|vec![(60001+n).to_string(),quantity.clone()]).collect::<Vec<_>>()});
    assert!(serde_json::to_vec(&event).unwrap().len() > 512 * 1024);
    view.peer.events.send(event).unwrap();
    let retired =
        view.wait(|v| v["data"]["connectionGeneration"] != first["data"]["connectionGeneration"]);
    assert!(
        retired["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_UNAVAILABLE"),
        "{retired}"
    );
    assert_eq!(retired["data"]["detail"]["status"], "UNAVAILABLE");
    assert_eq!(
        retired["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    let recovered = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_eq!(recovered["data"]["reconnectAttempt"], 1);
    assert_ne!(
        recovered["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
}

#[test]
fn material_outside_the_displayed_twenty_levels_still_changes_immutable_depth_identity() {
    if isolated_hot_scenario(
        "material_outside_the_displayed_twenty_levels_still_changes_immutable_depth_identity",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "source_changes_old_releases_and_reopen_cannot_preserve_or_retire_another_current_lease",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "real_snapshot_and_diff_stream_publish_exact_time_bound_continuous_depth_without_a_key",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "eth_depth_uses_its_exact_public_stream_rest_symbol_and_base_liquidity",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "overlapping_updates_delete_levels_without_expanding_known_bands_or_renewing_unchanged_material",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "five_thousand_known_levels_remain_bounded_and_one_extra_level_retires_instead_of_truncating",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "skewed_public_clock_samples_stop_before_depth_reads_or_claiming_quote_collection",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "session_safety_and_time_revalidation_retire_old_books_before_explicit_reacquisition",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "bootstrap_frame_capacity_retires_before_a_late_snapshot_can_publish_into_a_new_source",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "bootstrap_byte_capacity_stops_large_valid_frames_before_the_frame_count_limit",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "exact_account_spot_admission_qualifies_its_own_check_without_an_equity_calendar_or_order_authority",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "current_symbol_halt_rejects_spot_admission_despite_a_continuous_public_book",
    ) {
        return;
    }
    assert_spot_admission_rejects_external_negative("HALT", "SYMBOL_NOT_TRADING", "MARKET_HALTED");
}

#[test]
fn current_symbol_break_rejects_spot_admission_despite_a_continuous_public_book() {
    if isolated_hot_scenario(
        "current_symbol_break_rejects_spot_admission_despite_a_continuous_public_book",
    ) {
        return;
    }
    assert_spot_admission_rejects_external_negative("BREAK", "SYMBOL_NOT_TRADING", "MARKET_HALTED");
}

#[test]
fn current_system_maintenance_rejects_spot_admission_despite_a_continuous_public_book() {
    if isolated_hot_scenario(
        "current_system_maintenance_rejects_spot_admission_despite_a_continuous_public_book",
    ) {
        return;
    }
    assert_spot_admission_rejects_external_negative(
        "MAINTENANCE",
        "SYSTEM_MAINTENANCE",
        "INSTRUMENT_RULES_UNAVAILABLE",
    );
}

#[test]
fn unsatisfied_account_permission_sets_reject_spot_admission_despite_a_continuous_public_book() {
    if isolated_hot_scenario(
        "unsatisfied_account_permission_sets_reject_spot_admission_despite_a_continuous_public_book",
    ) {
        return;
    }
    assert_spot_admission_rejects_external_negative(
        "PERMISSION",
        "PERMISSION_SETS_NOT_SATISFIED",
        "INSTRUMENT_RULES_UNAVAILABLE",
    );
}

#[test]
fn current_api_trading_lock_rejects_spot_admission_despite_a_continuous_public_book() {
    if isolated_hot_scenario(
        "current_api_trading_lock_rejects_spot_admission_despite_a_continuous_public_book",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "displayed_spot_base_liquidity_qualifies_only_its_own_exact_side_size_check",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "exact_spot_spread_comparison_and_missing_last_trade_reference_remain_separate",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "other_symbol_rules_and_expired_rule_receipts_cannot_qualify_spot_admission",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "spot_admission_retires_when_the_public_account_refresh_changes_its_binding",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "authentic_public_spot_depth_satisfies_only_quote_freshness_without_a_fictional_last_trade",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "selected_public_spot_source_is_explicit_in_current_and_retired_market_details",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "public_spot_collection_keeps_an_independent_unverified_data_use_gate_after_reads_and_source_changes",
    ) {
        return;
    }
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
fn current_public_spot_book_cannot_be_handed_to_research_or_artifact_inputs() {
    if isolated_hot_scenario(
        "current_public_spot_book_cannot_be_handed_to_research_or_artifact_inputs",
    ) {
        return;
    }
    // Regression proof of an existing boundary, not a fabricated RED/GREEN repair.
    let view = HotView::open();
    let book = view.wait(|v| v["data"]["status"] == "STREAMING");
    assert_eq!(book["data"]["detail"]["status"], "AVAILABLE");
    let research = command(
        &view.control,
        "research.run",
        json!({"workspaceId":view.workspace,"agentMode":"ASK","executionContext":"NONE_READ_ONLY","attachedContexts":[],"toolId":"public_market_read","focus":"CRYPTO_SPOT","query":"BTC"}),
    );
    assert_eq!(research["ok"], true, "{research}");
    let result = &research["data"];
    assert_eq!(result["payload"]["state"], "UNAVAILABLE");
    assert!(result["payload"]["fixtureLabel"].is_null());
    assert!(result["payload"]["marketSnapshotRefs"].is_null());
    assert!(
        result["payload"]["instrumentRefs"]
            .as_array()
            .unwrap()
            .contains(&json!("crypto:BTC/USDT:spot"))
    );
    for venue in result["payload"]["spotVenues"].as_array().unwrap() {
        for field in ["bid", "ask", "spread", "depth", "quoteAge"] {
            assert!(
                venue[field].is_null(),
                "Live book must not enter research: {venue}"
            );
        }
    }
    let thread = command(
        &view.control,
        "thread.create",
        json!({"workspaceId":view.workspace,"title":"Public book boundary","defaultAgentMode":"ASK","defaultExecutionContext":"NONE_READ_ONLY","model":{"provider":"CHATGPT","modelId":"gpt-5.6-sol"},"linkedContexts":[]}),
    );
    assert_eq!(thread["ok"], true, "{thread}");
    let mut injected = result.clone();
    injected["payload"]["spotVenues"][0]["bid"] = book["data"]["detail"]["snapshot"]["bid"].clone();
    let rejected = command(
        &view.control,
        "turn.start",
        json!({"workspaceId":view.workspace,"threadId":thread["data"]["threadId"],"expectedStateVersion":thread["data"]["stateVersion"],"message":"Use the typed research result","agentMode":"ASK","executionContext":"NONE_READ_ONLY","attachedContexts":[],"researchInvocation":{"toolId":"public_market_read","focus":"CRYPTO_SPOT","query":"BTC"},"researchResult":injected}),
    );
    assert_eq!(
        rejected["error"]["code"], "RESEARCH_RESULT_INVALID",
        "{rejected}"
    );
    // Neither public artifact operation accepts a caller-supplied live snapshot.
    for (name, payload) in [
        (
            "artifact.save",
            json!({"workspaceId":view.workspace,"threadId":thread["data"]["threadId"],"turnId":"no-completed-turn","itemId":"no-completed-item","kind":"RESEARCH","title":"Unlicensed book","snapshot":book["data"]["detail"]["snapshot"]}),
        ),
        (
            "artifact.export",
            json!({"workspaceId":view.workspace,"artifactId":"no-saved-artifact","snapshot":book["data"]["detail"]["snapshot"]}),
        ),
    ] {
        let rejected = command(&view.control, name, payload);
        assert_eq!(
            rejected["error"]["code"], "IPC_PAYLOAD_INVALID",
            "{name}: {rejected}"
        );
    }
}

#[test]
fn captured_spot_review_keeps_raw_depth_ephemeral_while_risk_history_restores_only_references() {
    if isolated_hot_scenario(
        "captured_spot_review_keeps_raw_depth_ephemeral_while_risk_history_restores_only_references",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "protected_market_digest_keeps_unchanged_material_and_retires_on_depth_or_source_change",
    ) {
        return;
    }
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
    if isolated_hot_scenario(
        "financial_quote_ttl_is_stricter_than_collection_age_and_cached_reads_cannot_renew_it",
    ) {
        return;
    }
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

// Only the external signed provider response changes. No financial rows or
// quote/permission authority are seeded into the Control Plane.
struct MarketFeeHttp {
    inner: account_fixtures::Http,
}
impl ProviderHttp for MarketFeeHttp {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        let bytes = self.inner.get(endpoint, path, headers)?;
        if path.split('?').next() != Some("/api/v3/account/commission") {
            return Ok(bytes);
        }
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        value["discount"]["enabledForSymbol"] = json!(false);
        // A higher maker family must not be selected for native MARKET semantics.
        value["specialCommission"]["maker"] = json!("0.90000000");
        Ok(serde_json::to_vec(&value).unwrap())
    }
}

struct DiscountedBnbFeeHttp {
    inner: account_fixtures::Http,
    bnb_free: std::cell::Cell<&'static str>,
}
impl ProviderHttp for DiscountedBnbFeeHttp {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        let bytes = self.inner.get(endpoint, path, headers)?;
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        match path.split('?').next() {
            Some("/api/v3/account") => {
                value["balances"].as_array_mut().unwrap().push(json!({
                    "asset":"BNB", "free":self.bnb_free.get(), "locked":"0"
                }));
            }
            Some("/api/v3/account/commission") => {
                value["standardCommission"] = json!({
                    "maker":"0.01", "taker":"0.001", "buyer":"0", "seller":"0"
                });
                value["taxCommission"] = json!({
                    "maker":"0", "taker":"0.005", "buyer":"0", "seller":"0"
                });
                value["specialCommission"] = json!({
                    "maker":"0", "taker":"0", "buyer":"0", "seller":"0"
                });
                value["discount"] = json!({
                    "enabledForAccount":true, "enabledForSymbol":true,
                    "discountAsset":"BNB", "discount":"0.1"
                });
            }
            _ => return Ok(bytes),
        }
        Ok(serde_json::to_vec(&value).unwrap())
    }
}

fn acquire_fee_bnb_book(view: &HotView, peer: &DepthPeer) -> Value {
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    let acquired=view.supervisor.dispatch_with(
        &view.control,
        &json!({"requestId":"fee-book","schemaVersion":1,"command":"trade.spot_fee_market.acquire","payload":{"workspaceId":view.workspace,"expectedSourceVersion":source["data"]["stateVersion"]}}),
        "main", Arc::new(NoTradingKey), peer.http.clone(),
        QuoteStreamConnectors { stock: StockStreamConnector::default(), binance: peer.connector.clone() },
    );
    assert_eq!(acquired["ok"], true, "{acquired}");
    let query = json!({"workspaceId":view.workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let current = command(&view.control, "trade.spot_fee_market.get", query.clone());
        assert_eq!(current["ok"], true, "{current}");
        if current["data"]["status"] == "STREAMING" {
            return current["data"].clone();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Fee stream did not become continuous: {current}"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn refresh_spot_capacity(
    view: &HotView,
    proposal: &Value,
    vault: &account_fixtures::Vault,
    http: &impl ProviderHttp,
) -> Value {
    let query = json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]});
    let before = command(&view.control, "trade.spot_capacity.get", query.clone());
    tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"fee-capacity","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        vault,
        http,
    )
}

#[test]
fn owning_bnb_fee_estimate_uses_continuous_market_and_compares_the_complete_discounted_branch() {
    if isolated_hot_scenario(
        "owning_bnb_fee_estimate_uses_continuous_market_and_compares_the_complete_discounted_branch",
    ) {
        return;
    }
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (proposal, vault) = connected_binance_proposal(&view);
    let fee_peer = DepthPeer::with_symbol(
        "BNBUSDT",
        Some(
            json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64,"s":"BNBUSDT","U":9_007_199_254_740_992u64,"u":9_007_199_254_740_993u64,"b":[["500","0.5"]],"a":[["501","0.75"]]}),
        ),
        Vec::new(),
    );
    let fee_book = acquire_fee_bnb_book(&view, &fee_peer);
    assert_eq!(fee_book["snapshot"]["bid"], "500");
    let http = DiscountedBnbFeeHttp {
        inner: account_fixtures::Http {
            binance_rules_ui: true,
            binance_capacity_ui: std::cell::Cell::new(
                account_fixtures::BinanceCapacityFixture::CompleteCoverage,
            ),
            ..Default::default()
        },
        bnb_free: std::cell::Cell::new("0.001"),
    };
    collect_spot_admission(
        &view,
        &proposal,
        &vault,
        &http.inner,
        "crypto:BTC/USDT:spot",
    );
    let capacity = refresh_spot_capacity(&view, &proposal, &vault, &http);
    assert_eq!(capacity["data"]["status"], "OBSERVED", "{capacity}");
    let query = json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]});
    let commission = command(&view.control, "trade.spot_commission.get", query.clone());
    let terms = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"discounted-fee-terms","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":commission["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(terms["data"]["status"], "OBSERVED", "{terms}");
    let decision = command(&view.control, "risk.evaluate_proposal", query.clone());
    assert_eq!(decision["ok"], true, "{decision}");
    let wallet = &decision["data"]["spotFeeFx"]["walletFundingEvidence"];
    assert_eq!(wallet["status"], "COMPLETE", "{wallet}");
    assert_eq!(wallet["requiredAssets"], json!(["BTC", "USDT", "BNB"]));
    assert_eq!(wallet["missingAssets"], json!([]));
    assert_eq!(wallet["balances"].as_array().unwrap().len(), 3);
    for (asset, free, locked) in [
        ("BTC", "0.10000000", "0.02000000"),
        ("BNB", "0.001", "0"),
        (
            "USDT",
            "99999999999999999999.9999999999999999999",
            "0.0000000000000000002",
        ),
    ] {
        let row = wallet["balances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["asset"] == asset)
            .unwrap();
        assert_eq!(row["free"], free, "{row}");
        assert_eq!(row["locked"], locked, "{row}");
        assert_eq!(row["localClaims"], "0", "{row}");
        assert_eq!(row["effectiveFree"], free, "{row}");
    }
    let account_read = capacity["data"]["observation"]["reads"]
        .as_array()
        .unwrap()
        .iter()
        .find(|read| read["kind"] == "ACCOUNT")
        .unwrap();
    assert_eq!(
        wallet["collectionId"],
        capacity["data"]["observation"]["collectionId"]
    );
    assert_eq!(wallet["accountId"], capacity["data"]["accountId"]);
    assert_eq!(wallet["proposalHash"], capacity["data"]["proposalHash"]);
    assert_eq!(wallet["sourceVersion"], capacity["data"]["sourceVersion"]);
    assert_eq!(
        wallet["ruleMaterialVersion"],
        capacity["data"]["ruleMaterialVersion"]
    );
    assert_eq!(
        wallet["capacityBindingVersion"],
        capacity["data"]["bindingVersion"]
    );
    assert_eq!(wallet["accountReadStartedAt"], account_read["startedAt"]);
    assert_eq!(wallet["firstReceipt"], account_read["receivedAt"]);
    assert_eq!(wallet["originalDigest"], account_read["digest"]);
    let model = &decision["data"]["spotFeeFx"]["currentFeeModel"];
    assert_eq!(model["fallback"]["role"], "MAKER");
    assert_eq!(model["bnbFee"]["role"], "TAKER");
    assert_eq!(model["bnbFee"]["standard"], "0.0000120002");
    assert_eq!(model["bnbFee"]["tax"], "0.00060001");
    assert_eq!(model["bnbFee"]["special"], "0");
    assert_eq!(model["bnbFee"]["total"], "0.0006120102");
    assert_eq!(
        model["fundingPlanPreview"]["principalClaim"]["asset"],
        "USDT"
    );
    assert_eq!(
        model["fundingPlanPreview"]["principalClaim"]["amount"],
        "60"
    );
    assert_eq!(
        model["fundingPlanPreview"]["feeDisposition"],
        "BNB_CURRENT_ESTIMATE"
    );
    assert_eq!(
        model["fundingPlanPreview"]["currentFeeClaim"]["asset"],
        "BNB"
    );
    assert_eq!(
        model["fundingPlanPreview"]["currentFeeClaim"]["amount"],
        "0.0006120102"
    );
    assert_eq!(model["bnbFundingEvidence"]["effectiveFree"], "0.001");
    assert_eq!(
        model["chargingCondition"],
        "DISCOUNTED_BNB_CURRENT_ESTIMATE"
    );
    assert_eq!(
        model["bnbConversionReferences"].as_array().unwrap().len(),
        2
    );
    assert_eq!(model["bnbFeeCandidates"].as_array().unwrap().len(), 2);
    assert_eq!(model["bnbFeeCandidates"][0]["role"], "MAKER");
    assert_eq!(model["bnbFeeCandidates"][1]["role"], "TAKER");
    assert_eq!(model["bnbConversionReferences"][0]["pair"], "BTCUSDT");
    assert_eq!(model["bnbConversionReferences"][0]["priceSide"], "ASK");
    assert_eq!(model["bnbConversionReferences"][1]["pair"], "BNBUSDT");
    assert_eq!(model["bnbConversionReferences"][1]["priceSide"], "BID");
    assert!(
        model["limitations"]
            .as_array()
            .unwrap()
            .contains(&json!("BNB_INTERNAL_AVERAGE_EQUIVALENCE_UNPROVEN"))
    );
    for reference in model["bnbConversionReferences"].as_array().unwrap() {
        for raw in [
            "price", "quantity", "bid", "ask", "bidSize", "askSize", "bids", "asks",
        ] {
            assert!(
                reference.get(raw).is_none(),
                "Persisted conversion reference contains raw book field {raw}: {reference}"
            );
        }
    }
    assert_eq!(decision["data"]["spotFeeFx"]["outcome"], "UNAVAILABLE");

    // A synthetic, purpose-limited report exercises the conditional policy-valuation path only.
    // It is not evidence about the user's real Alpaca account or provider entitlement.
    let fx_http = account_fixtures::Http {
        fx_ui: true,
        ..Default::default()
    };
    let fx_request = json!({"requestId":"synthetic-policy-fx-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":view.workspace,"providerId":"alpaca","environment":"PAPER","label":"Synthetic policy valuation source"}});
    let fx_job = view
        .control
        .lock()
        .unwrap()
        .prepare_provider_for(&fx_request, "main")
        .unwrap()
        .unwrap();
    let fx_observed = fx_job.run(
        &vault,
        |_| account_fixtures::credentials(),
        &fx_http,
        || view.control.lock().unwrap().provider_job_current(&fx_job),
    );
    let fx_tested = view
        .control
        .lock()
        .unwrap()
        .complete_provider(&fx_job, fx_observed);
    assert_eq!(fx_tested["ok"], true, "{fx_tested}");
    let fx_account = command(
        &view.control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":view.workspace,"connectionId":fx_tested["data"]["connectionId"],"expectedStateVersion":fx_tested["data"]["stateVersion"],"acknowledgeUnverified":true}),
    );
    assert_eq!(fx_account["ok"], true, "{fx_account}");
    let fx_source = command(
        &view.control,
        "data.fx.connection",
        json!({"workspaceId":view.workspace}),
    );
    let fx_configured = command(
        &view.control,
        "data.fx.configure",
        json!({
            "workspaceId":view.workspace,
            "expectedStateVersion":fx_source["data"]["stateVersion"],
            "connectionId":fx_account["data"]["connectionId"],
            "cryptoLocation":"eu-1",
            "sourceUseContext":{
                "marketDataPlan":"BASIC",
                "accountJurisdiction":"IE",
                "agreementReviewedAt":"2026-10-12",
                "countrySupportConfirmation":{
                    "confirmedJurisdiction":"IE",
                    "confirmedAt":"2026-10-12",
                    "reference":"SYNTHETIC-ALPACA-SUPPORT-CASE"
                },
                "useScope":"OWN_LOCAL_NONCOMMERCIAL",
                "rawDataDestination":"LOCAL_ONLY"
            }
        }),
    );
    assert_eq!(fx_configured["ok"], true, "{fx_configured}");
    assert_eq!(
        fx_configured["data"]["sourceUseAssessment"]["status"],
        "SUPPORTED_FOR_DECLARED_SCOPE"
    );
    assert_eq!(
        fx_configured["data"]["sourceUseAssessment"]["provenance"],
        "USER_REPORTED_PROVIDER_SUPPORT"
    );
    assert_eq!(
        fx_configured["data"]["sourceUseAssessment"]["supportedPurposes"],
        json!(["POLICY_VALUATION"]),
        "Synthetic context supports only price-policy valuation"
    );
    assert_eq!(
        fx_configured["data"]["capabilityStatuses"]
            .as_array()
            .unwrap()
            .iter()
            .find(|status| status["capability"] == "FX_DATA_USE_RIGHTS")
            .unwrap()["status"],
        "UNVERIFIED",
        "A synthetic support report does not verify provider entitlement"
    );
    let fx_source = command(
        &view.control,
        "data.fx.connection",
        json!({"workspaceId":view.workspace}),
    );
    let fx_refreshed = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"synthetic-policy-fx-refresh","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":fx_source["data"]["stateVersion"]}}),
        "main",
        &vault,
        &fx_http,
    );
    assert_eq!(fx_refreshed["ok"], true, "{fx_refreshed}");
    assert_eq!(
        fx_refreshed["data"]["status"], "AVAILABLE",
        "{fx_refreshed}"
    );

    let valued_decision = command(&view.control, "risk.evaluate_proposal", query.clone());
    assert_eq!(valued_decision["ok"], true, "{valued_decision}");
    assert_eq!(
        valued_decision["data"]["status"], "UNAVAILABLE",
        "Positive monetary checks must not satisfy unrelated Live authorization gates"
    );
    let valued_statement = &valued_decision["data"]["spotFeeFx"];
    assert_eq!(
        valued_statement["outcome"], "UNAVAILABLE",
        "{valued_statement}"
    );
    assert_eq!(
        valued_statement["bindingBlocker"],
        "SPOT_FEE_CURRENCY_UNKNOWN"
    );
    assert_eq!(
        valued_statement["workspaceBaseExpectedSpend"], "60.006",
        "The BUY liability uses the current USDT/USD ask"
    );
    assert_eq!(
        valued_statement["currentFeeModel"]["chargingCondition"],
        "DISCOUNTED_BNB_CURRENT_ESTIMATE"
    );
    assert_eq!(
        valued_statement["currentFeeModel"]["fundingPlanPreview"]["currentFeeClaim"]["amount"],
        "0.0006120102"
    );
    let find_check = |check_id: &str| {
        valued_decision["data"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["checkId"] == check_id)
            .unwrap_or_else(|| panic!("Missing {check_id}: {valued_decision}"))
    };
    assert_eq!(find_check("SPOT_FEE_FX")["outcome"], "UNAVAILABLE");
    assert_eq!(find_check("ORDER_NOTIONAL")["outcome"], "PASS");
    assert_eq!(find_check("RESERVED_CAPITAL")["outcome"], "PASS");
    assert_eq!(find_check("ACCOUNT_ARMING")["outcome"], "UNAVAILABLE");
    let alpaca_capabilities = valued_statement["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["purpose"] == "INTENT_POLICY")
        .unwrap();
    assert_eq!(alpaca_capabilities["state"], "QUALIFIED");
    let account_state = command(
        &view.control,
        "account.get",
        json!({"workspaceId":view.workspace,"connectionId":proposal["fields"]["accountId"]}),
    );
    assert_eq!(account_state["data"]["health"]["arming"], "DISARMED");
    let approvals = command(
        &view.control,
        "trade.approval.list",
        json!({"workspaceId":view.workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(approvals["ok"], true, "{approvals}");
    assert!(
        approvals["data"]["approvals"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(http.inner.binance_posts.borrow().is_empty());
    assert!(fx_http.alpaca_posts.borrow().is_empty());

    // A separate public observation with a lower real free balance selects the received-asset
    // fallback while retaining the current BNB estimate and its original market references.
    http.bnb_free.set("0.0000001");
    let low_capacity = refresh_spot_capacity(&view, &proposal, &vault, &http);
    assert_eq!(low_capacity["data"]["status"], "OBSERVED", "{low_capacity}");
    let insufficient = command(&view.control, "risk.evaluate_proposal", query);
    let low = &insufficient["data"]["spotFeeFx"]["currentFeeModel"];
    assert_eq!(low["bnbFee"]["total"], "0.0006120102");
    assert_eq!(
        low["chargingCondition"],
        "INSUFFICIENT_BNB_CURRENT_FALLBACK"
    );
    assert_eq!(
        low["fundingPlanPreview"]["feeDisposition"],
        "RECEIVED_ASSET_DEDUCTION"
    );
    assert_eq!(low["fundingPlanPreview"]["currentFeeClaim"], Value::Null);
    assert_eq!(low["fallback"]["role"], "MAKER");

    // SELL receives USDT, so only the fixed BNBUSDT bid is consumed. The BNB role is still
    // selected from the complete discounted three-family total, independently of fallback.
    http.bnb_free.set("1");
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":proposal["fields"]["accountId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"SELL","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let sell = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(sell["ok"], true, "{sell}");
    let sell_query = json!({"workspaceId":view.workspace,"proposalId":sell["data"]["proposalId"]});
    let capacity = refresh_spot_capacity(&view, &sell["data"], &vault, &http);
    assert_eq!(capacity["data"]["status"], "OBSERVED", "{capacity}");
    let commission = command(
        &view.control,
        "trade.spot_commission.get",
        sell_query.clone(),
    );
    let terms = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"sell-fee-terms","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":view.workspace,"proposalId":sell["data"]["proposalId"],"expectedStateVersion":commission["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(terms["data"]["status"], "OBSERVED", "{terms}");
    let sell_decision = command(&view.control, "risk.evaluate_proposal", sell_query.clone());
    let sell_model = &sell_decision["data"]["spotFeeFx"]["currentFeeModel"];
    assert_eq!(sell_model["receivedAsset"], "USDT");
    assert_eq!(sell_model["bnbFee"]["role"], "TAKER");
    assert_eq!(sell_model["bnbFee"]["total"], "0.000612");
    assert_eq!(
        sell_model["fundingPlanPreview"]["principalClaim"]["asset"],
        "BTC"
    );
    assert_eq!(
        sell_model["fundingPlanPreview"]["principalClaim"]["amount"],
        "0.001"
    );
    assert_eq!(
        sell_model["fundingPlanPreview"]["feeDisposition"],
        "BNB_CURRENT_ESTIMATE"
    );
    assert_eq!(
        sell_model["fundingPlanPreview"]["currentFeeClaim"]["amount"],
        "0.000612"
    );
    assert_eq!(
        sell_model["bnbConversionReferences"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(sell_model["bnbConversionReferences"][0]["pair"], "BNBUSDT");

    // Tighten the real loopback book's best-bid size. The estimate now refuses exact coverage.
    fee_peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64,"s":"BNBUSDT","U":9_007_199_254_740_994u64,"u":9_007_199_254_740_994u64,"b":[["500","0.0001"]],"a":[]})).unwrap();
    let fee_query = json!({"workspaceId":view.workspace,"leaseId":fee_book["leaseId"],"generation":fee_book["generation"]});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let current = command(
            &view.control,
            "trade.spot_fee_market.get",
            fee_query.clone(),
        );
        if current["data"]["snapshot"]["bidSize"] == "0.0001" {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Updated BNB depth was not observed: {current}"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let insufficient_depth = command(&view.control, "risk.evaluate_proposal", sell_query);
    let depth_model = &insufficient_depth["data"]["spotFeeFx"]["currentFeeModel"];
    assert!(depth_model["bnbFee"].is_null());
    assert_eq!(
        depth_model["bnbFeeFailure"],
        "SPOT_BNB_MARKET_COVERAGE_INSUFFICIENT"
    );
    assert_eq!(
        depth_model["chargingCondition"],
        "CONDITIONAL_BNB_INPUTS_REQUIRED"
    );
    view.supervisor.stop_all();
}

#[test]
fn owning_market_sell_fee_uses_the_real_current_bid_and_taker_families() {
    if isolated_hot_scenario("owning_market_sell_fee_uses_the_real_current_bid_and_taker_families")
    {
        return;
    }
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (original, vault) = connected_binance_proposal(&view);
    save_spot_quote_policy(&view, "1", None);
    let http = MarketFeeHttp {
        inner: account_fixtures::Http {
            binance_rules_ui: true,
            ..Default::default()
        },
    };
    collect_spot_admission(
        &view,
        &original,
        &vault,
        &http.inner,
        "crypto:BTC/USDT:spot",
    );
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":original["fields"]["accountId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"SELL","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":"61","timeInForce":"DAY"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let query = json!({"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"]});
    let before = command(&view.control, "trade.spot_commission.get", query.clone());
    let terms = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"market-fees","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(terms["data"]["status"], "OBSERVED", "{terms}");
    let result = command(&view.control, "risk.evaluate_proposal", query.clone());
    assert_eq!(result["ok"], true, "{result}");
    let model = &result["data"]["spotFeeFx"]["currentFeeModel"];
    assert_eq!(model["grossReceived"], "60");
    assert_eq!(model["receivedAsset"], "USDT");
    assert_eq!(model["fallback"]["role"], "TAKER");
    assert_eq!(model["fallback"]["standard"], "0.000036");
    assert_eq!(model["fallback"]["tax"], "0.000138");
    assert_eq!(model["fallback"]["special"], "3.6");
    assert_eq!(model["fallback"]["total"], "3.600174");
    assert_eq!(model["fallback"]["netReceived"], "56.399826");
    assert_eq!(model["marketReference"]["priceSide"], "BID");
    assert_eq!(
        model["marketReference"]["marketSnapshotId"],
        current["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_eq!(result["data"]["spotFeeFx"]["outcome"], "UNAVAILABLE");
    let review = command(&view.control, "trade.request_approval", query);
    assert_eq!(review["data"]["eligible"], false);
}

#[test]
fn owning_market_buy_base_fee_is_a_received_base_deduction_at_the_real_ask() {
    if isolated_hot_scenario(
        "owning_market_buy_base_fee_is_a_received_base_deduction_at_the_real_ask",
    ) {
        return;
    }
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (original, vault) = connected_binance_proposal(&view);
    save_spot_quote_policy(&view, "1", None);
    let http = MarketFeeHttp {
        inner: account_fixtures::Http {
            binance_rules_ui: true,
            ..Default::default()
        },
    };
    collect_spot_admission(
        &view,
        &original,
        &vault,
        &http.inner,
        "crypto:BTC/USDT:spot",
    );
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":original["fields"]["accountId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":"61","timeInForce":"DAY"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let query = json!({"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"]});
    let before = command(&view.control, "trade.spot_commission.get", query.clone());
    let terms = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"market-fees","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(terms["data"]["status"], "OBSERVED", "{terms}");
    let result = command(&view.control, "risk.evaluate_proposal", query.clone());
    assert_eq!(result["ok"], true, "{result}");
    let model = &result["data"]["spotFeeFx"]["currentFeeModel"];
    assert_eq!(model["grossReceived"], "0.001");
    assert_eq!(model["receivedAsset"], "BTC");
    assert_eq!(model["fallback"]["role"], "TAKER");
    assert_eq!(model["fallback"]["standard"], "0.0000000005");
    assert_eq!(model["fallback"]["tax"], "0.00000000232");
    assert_eq!(model["fallback"]["special"], "0.00005");
    assert_eq!(model["fallback"]["total"], "0.00005000282");
    assert_eq!(model["fallback"]["netReceived"], "0.00094999718");
    assert_eq!(model["marketReference"]["priceSide"], "ASK");
    assert_eq!(
        model["marketReference"]["marketSnapshotId"],
        current["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_eq!(result["data"]["spotFeeFx"]["outcome"], "UNAVAILABLE");
    let review = command(&view.control, "trade.request_approval", query);
    assert_eq!(review["data"]["eligible"], false);
}

#[test]
fn owning_market_buy_quote_fee_keeps_the_exact_ratio_until_final_family_rounding() {
    if isolated_hot_scenario(
        "owning_market_buy_quote_fee_keeps_the_exact_ratio_until_final_family_rounding",
    ) {
        return;
    }
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (original, vault) = connected_binance_proposal(&view);
    save_spot_quote_policy(&view, "1", None);
    let http = MarketFeeHttp {
        inner: account_fixtures::Http {
            binance_rules_ui: true,
            ..Default::default()
        },
    };
    collect_spot_admission(
        &view,
        &original,
        &vault,
        &http.inner,
        "crypto:BTC/USDT:spot",
    );
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":original["fields"]["accountId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"QUOTE","value":"60"},"limitPrice":null,"maximumSpend":"61","timeInForce":"DAY"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let query = json!({"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"]});
    let before = command(&view.control, "trade.spot_commission.get", query.clone());
    let terms = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"market-fees","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(terms["data"]["status"], "OBSERVED", "{terms}");
    let result = command(&view.control, "risk.evaluate_proposal", query.clone());
    assert_eq!(result["ok"], true, "{result}");
    let model = &result["data"]["spotFeeFx"]["currentFeeModel"];
    // Independent high-precision decimal example: native60QUOTE / actual60001ASK.
    // Retain the exact ratio through all three families, then ceil each liability18.
    assert_eq!(model["grossReceived"], "0.000999983333611106");
    // Canonical rational audit values are independently reduced exact fractions;
    // they survive capture without a raw market price/depth or a renewed receipt.
    assert_eq!(
        model["fallback"]["preRounding"]["standard"],
        json!({"numerator":"3","denominator":"6000100000"})
    );
    assert_eq!(
        model["fallback"]["preRounding"]["tax"],
        json!({"numerator":"3","denominator":"1293125000"})
    );
    assert_eq!(
        model["fallback"]["preRounding"]["special"],
        json!({"numerator":"3","denominator":"60001"})
    );
    assert_eq!(
        model["fallback"]["preRounding"]["total"],
        json!({"numerator":"7500423","denominator":"150002500000"})
    );
    assert_eq!(model["fallbackCandidates"].as_array().unwrap().len(), 1);
    assert_eq!(model["fallbackCandidates"][0]["role"], "TAKER");
    assert_eq!(model["receivedAsset"], "BTC");
    assert_eq!(model["fallback"]["role"], "TAKER");
    assert_eq!(model["fallback"]["standard"], "0.000000000499991667");
    assert_eq!(model["fallback"]["tax"], "0.000000002319961334");
    assert_eq!(model["fallback"]["special"], "0.000049999166680556");
    assert_eq!(model["fallback"]["total"], "0.000050001986633557");
    assert_eq!(model["fallback"]["netReceived"], "0.000949981346977549");
    assert_eq!(model["marketReference"]["priceSide"], "ASK");
    assert_eq!(
        model["marketReference"]["marketSnapshotId"],
        current["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_eq!(result["data"]["spotFeeFx"]["outcome"], "UNAVAILABLE");
    let review = command(&view.control, "trade.request_approval", query.clone());
    assert_eq!(review["data"]["eligible"], false);
    assert_eq!(review["data"]["maximumAuthorizedSpend"], "61");
    assert_eq!(review["data"]["capacityProjection"]["unit"], "USDT");
    assert_eq!(
        review["data"]["capacityProjection"]["requestedAmount"], "60",
        "QUOTE MARKET principal is the native quoteOrderQty; maximumSpend remains its separate local ceiling: {review}"
    );
    let saved = command(&view.control, "risk.decision.list", query.clone());
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(
        saved["data"]["decisions"][0]["spotFeeFx"]["currentFeeModel"]["fallback"]["preRounding"]["standard"],
        json!({"numerator":"3","denominator":"6000100000"})
    );
    let encoded = saved["data"].to_string();
    for raw in ["bids", "asks", "bid", "ask", "bidSize", "askSize"] {
        assert!(
            !encoded.contains(&format!("\"{raw}\":")),
            "A derived fee audit cannot persist raw book fields: {raw}"
        );
    }
    assert_eq!(
        command(&view.control, "workspace.open", json!({}))["ok"],
        true
    );
    assert_eq!(
        command(&view.control, "risk.decision.list", query)["data"],
        saved["data"],
        "Reopen must retain original expression and market receipts, without renewal"
    );
}

#[test]
fn owning_limit_buy_capacity_uses_native_quantity_times_limit_not_maximum_spend() {
    if isolated_hot_scenario(
        "owning_limit_buy_capacity_uses_native_quantity_times_limit_not_maximum_spend",
    ) {
        return;
    }
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (original, vault) = connected_binance_proposal(&view);
    save_spot_quote_policy(&view, "1", None);
    let http = MarketFeeHttp {
        inner: account_fixtures::Http {
            binance_rules_ui: true,
            ..Default::default()
        },
    };
    collect_spot_admission(
        &view,
        &original,
        &vault,
        &http.inner,
        "crypto:BTC/USDT:spot",
    );
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":original["fields"]["accountId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.1"},"limitPrice":"500","maximumSpend":"40","timeInForce":"GTC"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    assert_eq!(proposal["data"]["fields"]["maximumSpend"], "40");
    let review = command(
        &view.control,
        "trade.request_approval",
        json!({"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(review["data"]["eligible"], false);
    assert_eq!(review["data"]["expectedSpend"], "50");
    assert_eq!(review["data"]["maximumAuthorizedSpend"], "40");
    assert!(
        review["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .is_some_and(|text| text.contains("MAXIMUM_AUTHORIZED_SPEND_EXCEEDED"))),
        "An explicitly smaller local ceiling refuses the limit order: {review}"
    );
    assert_eq!(review["data"]["capacityProjection"]["unit"], "USDT");
    assert_eq!(
        review["data"]["capacityProjection"]["requestedAmount"], "50",
        "BUY LIMIT principal is exact quantity times immutable limit price even when the independent maximumSpend is too small; refuse without understating the requested native principal: {review}"
    );
}

#[test]
fn owning_quote_subscale_remainder_is_not_discarded_before_fee_ceiling() {
    if isolated_hot_scenario("owning_quote_subscale_remainder_is_not_discarded_before_fee_ceiling")
    {
        return;
    }
    let view = HotView::open();
    let current = view.wait(|v| v["data"]["status"] == "STREAMING");
    let (original, vault) = connected_binance_proposal(&view);
    save_spot_quote_policy(&view, "1", None);
    let http = MarketFeeHttp {
        inner: account_fixtures::Http {
            binance_rules_ui: true,
            ..Default::default()
        },
    };
    collect_spot_admission(
        &view,
        &original,
        &vault,
        &http.inner,
        "crypto:BTC/USDT:spot",
    );
    let draft = command(
        &view.control,
        "trade.save_draft",
        json!({"workspaceId":view.workspace,"fields":{"accountId":original["fields"]["accountId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"QUOTE","value":"0.060001000000000001"},"limitPrice":null,"maximumSpend":"61","timeInForce":"DAY"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &view.control,
        "trade.generate_proposal",
        json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let query = json!({"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"]});
    let before = command(&view.control, "trade.spot_commission.get", query.clone());
    let terms = tradex::financial_sources::execute_refresh(
        &view.control,
        &json!({"requestId":"market-fees","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(terms["data"]["status"], "OBSERVED", "{terms}");
    let result = command(&view.control, "risk.evaluate_proposal", query.clone());
    assert_eq!(result["ok"], true, "{result}");
    let model = &result["data"]["spotFeeFx"]["currentFeeModel"];
    // Exact native0.060001000000000001QUOTE / actual60001ASK is just above
    // 0.000001BASE. Flooring gross18 first would understate each family by1e-18.
    // Retain the exact ratio through all three families, then ceil each liability18.
    assert_eq!(model["grossReceived"], "0.000001");
    assert_eq!(model["receivedAsset"], "BTC");
    assert_eq!(model["fallback"]["role"], "TAKER");
    assert_eq!(model["fallback"]["standard"], "0.000000000000500001");
    assert_eq!(model["fallback"]["tax"], "0.000000000002320001");
    assert_eq!(model["fallback"]["special"], "0.000000050000000001");
    assert_eq!(model["fallback"]["total"], "0.000000050002820003");
    assert_eq!(model["fallback"]["netReceived"], "0.000000949997179997");
    assert_eq!(model["marketReference"]["priceSide"], "ASK");
    assert_eq!(
        model["marketReference"]["marketSnapshotId"],
        current["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_eq!(result["data"]["spotFeeFx"]["outcome"], "UNAVAILABLE");
    let review = command(&view.control, "trade.request_approval", query);
    assert_eq!(review["data"]["eligible"], false);
}

#[test]
fn owning_quote_market_coverage_compares_native_quote_units_without_rounding() {
    if isolated_hot_scenario(
        "owning_quote_market_coverage_compares_native_quote_units_without_rounding",
    ) {
        return;
    }
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let (original, vault) = connected_binance_proposal(&view);
    save_spot_quote_policy(&view, "1", None);
    let http = MarketFeeHttp {
        inner: account_fixtures::Http {
            binance_rules_ui: true,
            ..Default::default()
        },
    };
    collect_spot_admission(
        &view,
        &original,
        &vault,
        &http.inner,
        "crypto:BTC/USDT:spot",
    );
    // Real ASK60001 * displayed0.75BASE =45000.75QUOTE exactly. Even1e-18
    // QUOTE above that level is uncovered; a rounded derived BASE would hide it.
    for (side, quantity, tif, covered, modeled) in [
        ("BUY", "45000.75", "DAY", true, true),
        ("BUY", "45000.750000000000000001", "DAY", false, false),
        ("SELL", "60", "DAY", false, false),
        ("BUY", "60", "GTC", true, false),
    ] {
        let draft = command(
            &view.control,
            "trade.save_draft",
            json!({"workspaceId":view.workspace,"fields":{"accountId":original["fields"]["accountId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":side,"orderType":"MARKET","quantity":{"type":"QUOTE","value":quantity},"limitPrice":null,"maximumSpend":"45001","timeInForce":tif}}),
        );
        assert_eq!(draft["ok"], true, "{draft}");
        let proposal = command(
            &view.control,
            "trade.generate_proposal",
            json!({"workspaceId":view.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
        );
        assert_eq!(proposal["ok"], true, "{proposal}");
        let query =
            json!({"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"]});
        let before = command(&view.control, "trade.spot_commission.get", query.clone());
        let terms = tradex::financial_sources::execute_refresh(
            &view.control,
            &json!({"requestId":"coverage-fees","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":view.workspace,"proposalId":proposal["data"]["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &vault,
            &http,
        );
        assert_eq!(terms["data"]["status"], "OBSERVED", "{terms}");
        let result = command(&view.control, "risk.evaluate_proposal", query.clone());
        assert_eq!(result["ok"], true, "{result}");
        let slippage = result["data"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["checkId"] == "MARKET_ORDER_SLIPPAGE")
            .unwrap();
        assert_eq!(
            slippage["outcome"],
            if covered { "PASS" } else { "UNAVAILABLE" },
            "{side}/{quantity}/{tif}: {slippage}"
        );
        assert_eq!(
            result["data"]["spotFeeFx"]["currentFeeModel"].is_object(),
            modeled,
            "{side}/{quantity}/{tif}: {result}"
        );
        if modeled {
            assert_eq!(
                result["data"]["spotFeeFx"]["currentFeeModel"]["grossReceived"],
                "0.75"
            );
        }
        assert_ne!(result["data"]["status"], "ALLOWED");
        assert_eq!(
            command(&view.control, "trade.request_approval", query)["data"]["eligible"],
            false
        );
    }
}

#[test]
fn fee_only_bnb_book_is_continuous_without_replacing_the_trading_quote_or_catalogue() {
    if isolated_hot_scenario(
        "fee_only_bnb_book_is_continuous_without_replacing_the_trading_quote_or_catalogue",
    ) {
        return;
    }
    let view = HotView::open();
    let trade = view.wait(|v| v["data"]["status"] == "STREAMING");
    let fee_peer = DepthPeer::with_symbol(
        "BNBUSDT",
        Some(
            json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64,"s":"BNBUSDT","U":9007199254740992u64,"u":9007199254740993u64,"b":[["500","0.5"]],"a":[["501","0.75"]]}),
        ),
        Vec::new(),
    );
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    let acquired = view.supervisor.dispatch_with(&view.control,&json!({"requestId":"fee-bnb","schemaVersion":1,"command":"trade.spot_fee_market.acquire","payload":{"workspaceId":view.workspace,"expectedSourceVersion":source["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),fee_peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:fee_peer.connector.clone()});
    assert_eq!(
        acquired["ok"], true,
        "A fixed fee-only BNB source must be supported: {acquired}"
    );
    let query = json!({"workspaceId":view.workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let current = loop {
        let state = command(&view.control, "trade.spot_fee_market.get", query.clone());
        assert_eq!(state["ok"], true, "{state}");
        if state["data"]["status"] == "STREAMING" {
            break state;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Fee stream did not become continuous: {state}"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(current["data"]["snapshot"]["bid"], "500");
    assert_eq!(current["data"]["snapshot"]["ask"], "501");
    assert_eq!(current["data"]["snapshot"]["bidSize"], "0.5");
    assert_eq!(
        current["data"]["snapshot"]["provenance"]["binance"]["baseAsset"],
        "BNB"
    );
    assert_eq!(
        current["data"]["snapshot"]["provenance"]["binance"]["providerSymbol"],
        "BNBUSDT"
    );
    assert_eq!(current["data"]["metadata"]["baseAsset"], "BNB");
    assert_eq!(current["data"]["metadata"]["quoteAsset"], "USDT");
    assert_eq!(current["data"]["metadata"]["marketStatus"], "TRADING");
    assert_eq!(
        current["data"]["metadata"]["originalDigest"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert_eq!(
        current["data"]["snapshot"]["provenance"]["binance"]["dataUseRights"],
        "UNVERIFIED"
    );
    let trade_after = command(&view.control, "market.hot.get", view.query.clone());
    assert_eq!(trade_after["data"]["leaseId"], trade["data"]["leaseId"]);
    assert_eq!(trade_after["data"]["status"], "STREAMING");
    let catalog = command(
        &view.control,
        "market.catalog",
        json!({"workspaceId":view.workspace,"query":"","tier":"HOT"}),
    );
    assert!(
        !catalog["data"].to_string().contains("BNB"),
        "The fee source cannot become a trading instrument"
    );
    let requested = fee_peer.requests.lock().unwrap().clone();
    assert!(requested.contains(&"/api/v3/exchangeInfo?symbol=BNBUSDT".into()));
    assert!(requested.contains(&"/api/v3/depth?symbol=BNBUSDT&limit=1000".into()));
    let released=view.supervisor.dispatch_with(&view.control,&json!({"requestId":"fee-release","schemaVersion":1,"command":"trade.spot_fee_market.release","payload":query}),"main",Arc::new(NoTradingKey),fee_peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:fee_peer.connector.clone()});
    assert_eq!(released["data"]["released"], true);
    assert_eq!(
        command(&view.control, "market.hot.get", view.query.clone())["data"]["status"],
        "STREAMING"
    );
}

#[test]
fn fee_only_bnb_wrong_metadata_cannot_admit_a_book_or_extend_the_order_surface() {
    if isolated_hot_scenario(
        "fee_only_bnb_wrong_metadata_cannot_admit_a_book_or_extend_the_order_surface",
    ) {
        return;
    }
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let fee_peer = DepthPeer::with_symbol("BNBUSDT", None, Vec::new());
    // This is a real external response with the wrong exact BASE, not a CP setter.
    (*fee_peer.fee_metadata.lock().unwrap())["symbols"][0]["baseAsset"] = json!("BTC");
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    let acquired=view.supervisor.dispatch_with(&view.control,&json!({"requestId":"bad-fee-meta","schemaVersion":1,"command":"trade.spot_fee_market.acquire","payload":{"workspaceId":view.workspace,"expectedSourceVersion":source["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),fee_peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:fee_peer.connector.clone()});
    assert_eq!(acquired["ok"], true, "{acquired}");
    let query = json!({"workspaceId":view.workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let state = command(&view.control, "trade.spot_fee_market.get", query.clone());
        assert_eq!(state["ok"], true, "{state}");
        if state["data"]["status"] == "FAILED" {
            assert!(state["data"]["snapshot"].is_null());
            assert!(state["data"]["metadata"].is_null());
            assert!(
                state["data"]["reason"]
                    .as_str()
                    .unwrap()
                    .contains("PROVIDER_RESPONSE_INVALID")
            );
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{state}");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        !fee_peer
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|p| p.contains("/depth?")),
        "Rejected metadata cannot admit a fee depth read"
    );
    assert_eq!(
        command(&view.control, "market.hot.get", view.query.clone())["data"]["status"],
        "STREAMING"
    );
    for instrument in ["crypto:BNB/USDT:spot", "internal:BNB/USDT:fee"] {
        let draft = command(
            &view.control,
            "trade.save_draft",
            json!({"workspaceId":view.workspace,"fields":{"accountId":null,"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":instrument,"side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"1"},"limitPrice":null,"maximumSpend":"501","timeInForce":"DAY"}}),
        );
        assert_eq!(
            draft["ok"], false,
            "The fee source must not admit a draft: {draft}"
        );
    }
}

#[test]
fn fee_only_bnb_cached_reads_expire_without_receipt_renewal_and_clock_changes_retire_them() {
    if isolated_hot_scenario(
        "fee_only_bnb_cached_reads_expire_without_receipt_renewal_and_clock_changes_retire_them",
    ) {
        return;
    }
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let fee_peer = DepthPeer::with_symbol("BNBUSDT", None, Vec::new());
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    let acquired=view.supervisor.dispatch_with(&view.control,&json!({"requestId":"fee-expiry","schemaVersion":1,"command":"trade.spot_fee_market.acquire","payload":{"workspaceId":view.workspace,"expectedSourceVersion":source["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),fee_peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:fee_peer.connector.clone()});
    assert_eq!(acquired["ok"], true, "{acquired}");
    let query = json!({"workspaceId":view.workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let original = loop {
        let current = command(&view.control, "trade.spot_fee_market.get", query.clone());
        if current["data"]["status"] == "STREAMING" {
            break current;
        }
        assert!(std::time::Instant::now() < deadline, "{current}");
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let requests = fee_peer.requests.lock().unwrap().len();
    // Let real monotonic time pass. A cached public getter is not a new producer.
    std::thread::sleep(std::time::Duration::from_millis(3100));
    let expired = command(&view.control, "trade.spot_fee_market.get", query.clone());
    assert_eq!(expired["data"]["status"], "STALE");
    assert_eq!(
        expired["data"]["snapshot"]["provenance"]["freshness"],
        "STALE"
    );
    assert_eq!(
        expired["data"]["snapshot"]["provenance"]["receivedTimestamp"],
        original["data"]["snapshot"]["provenance"]["receivedTimestamp"]
    );
    assert_eq!(
        expired["data"]["metadata"]["receivedAt"],
        original["data"]["metadata"]["receivedAt"]
    );
    assert_eq!(fee_peer.requests.lock().unwrap().len(), requests);
    assert_eq!(
        command(
            &view.control,
            "time.revalidate",
            json!({"workspaceId":view.workspace})
        )["ok"],
        true
    );
    assert_eq!(
        command(&view.control, "trade.spot_fee_market.get", query.clone())["data"]["status"],
        "STALE"
    );
    let denied=view.control.lock().unwrap().dispatch_with_events(json!({"requestId":"agent-fee","schemaVersion":1,"command":"trade.spot_fee_market.get","payload":query}),"research",None);
    assert_eq!(denied["error"]["code"], "IPC_ACCESS_DENIED");
}

#[test]
fn fee_only_bnb_gap_rebuilds_its_own_generation_and_source_disconnect_retires_it() {
    if isolated_hot_scenario(
        "fee_only_bnb_gap_rebuilds_its_own_generation_and_source_disconnect_retires_it",
    ) {
        return;
    }
    let view = HotView::open();
    view.wait(|v| v["data"]["status"] == "STREAMING");
    let fee_peer = DepthPeer::with_symbol("BNBUSDT", None, Vec::new());
    let source = command(
        &view.control,
        "data.binance_market.connection",
        json!({"workspaceId":view.workspace}),
    );
    let acquired=view.supervisor.dispatch_with(&view.control,&json!({"requestId":"fee-gap","schemaVersion":1,"command":"trade.spot_fee_market.acquire","payload":{"workspaceId":view.workspace,"expectedSourceVersion":source["data"]["stateVersion"]}}),"main",Arc::new(NoTradingKey),fee_peer.http.clone(),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:fee_peer.connector.clone()});
    assert_eq!(acquired["ok"], true, "{acquired}");
    let query = json!({"workspaceId":view.workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let wait = |retry: u64| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let current = command(&view.control, "trade.spot_fee_market.get", query.clone());
            assert_eq!(current["ok"], true, "{current}");
            if current["data"]["status"] == "STREAMING"
                && current["data"]["reconnectAttempt"] == retry
            {
                break current;
            }
            assert!(std::time::Instant::now() < deadline, "{current}");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    };
    let original = wait(0);
    fee_peer.events.send(json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64,"s":"BNBUSDT","U":9007199254741092u64,"u":9007199254741093u64,"b":[["500","0.5"]],"a":[["501","0.75"]]})).unwrap();
    let rebuilt = wait(1);
    assert_ne!(
        rebuilt["data"]["connectionGeneration"],
        original["data"]["connectionGeneration"]
    );
    assert_ne!(
        rebuilt["data"]["snapshot"]["provenance"]["marketSnapshotId"],
        original["data"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_ne!(
        rebuilt["data"]["metadata"]["receivedAt"],
        original["data"]["metadata"]["receivedAt"]
    );
    assert_eq!(
        command(&view.control, "market.hot.get", view.query.clone())["data"]["connectionGeneration"],
        view.query["generation"]
    );
    let disconnected = command(
        &view.control,
        "data.binance_market.disconnect",
        json!({"workspaceId":view.workspace,"expectedStateVersion":source["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    let retired = command(&view.control, "trade.spot_fee_market.get", query.clone());
    assert_eq!(retired["data"]["status"], "STALE");
    assert_eq!(
        retired["data"]["snapshot"]["provenance"]["freshness"],
        "STALE"
    );
    assert_eq!(
        retired["data"]["snapshot"]["provenance"]["receivedTimestamp"],
        rebuilt["data"]["snapshot"]["provenance"]["receivedTimestamp"]
    );
    assert_eq!(
        retired["data"]["metadata"]["receivedAt"],
        rebuilt["data"]["metadata"]["receivedAt"]
    );
}
