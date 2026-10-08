#![cfg(feature = "integration-test")]
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tradex::{
    ControlPlane,
    binance_market::stream::BinanceStreamConnector,
    protocol::Result,
    provider_io::{CredentialVault, Credentials, ProviderEndpoint, ProviderHttp},
    quote_source::hot::{QuoteHotSupervisor, QuoteStreamConnectors, StockStreamConnector},
};

struct NoAccess;
impl CredentialVault for NoAccess {
    fn put(&self, _: &str, _: &Credentials) -> Result<()> {
        panic!("Public stream cannot store a credential")
    }
    fn get(&self, _: &str) -> Result<Credentials> {
        panic!("Public stream cannot read a credential")
    }
    fn remove(&self, _: &str) -> Result<()> {
        panic!("Public stream cannot delete a credential")
    }
}
impl ProviderHttp for NoAccess {
    fn get(&self, _: ProviderEndpoint, _: &str, _: reqwest::header::HeaderMap) -> Result<Vec<u8>> {
        panic!("An actual denied upgrade must not trigger REST bootstrap")
    }
}
fn command(control: &Arc<Mutex<ControlPlane>>, name: &str, payload: Value) -> Value {
    control.lock().unwrap().dispatch_with_events(
        json!({"requestId":"stream-budget","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}
struct DeniedPeer {
    stop: Arc<AtomicBool>,
    accepted: Arc<AtomicUsize>,
    thread: Option<thread::JoinHandle<()>>,
    connector: BinanceStreamConnector,
}
impl DeniedPeer {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let connector = BinanceStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/ws/btcusdt@depth@100ms",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let accepted = Arc::new(AtomicUsize::new(0));
        let halt = stop.clone();
        let ledger = accepted.clone();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(30);
            while !halt.load(Ordering::Acquire) && Instant::now() < deadline {
                let (mut socket, _) = match listener.accept() {
                    Ok(v) => v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(e) => panic!("{e}"),
                };
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    socket.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                    assert!(request.len() < 16384);
                }
                let request = String::from_utf8(request).unwrap();
                assert!(request.starts_with("GET /ws/btcusdt@depth@100ms HTTP/1.1\r\n"));
                assert!(!request.to_ascii_lowercase().contains("x-mbx-apikey"));
                ledger.fetch_add(1, Ordering::AcqRel);
                socket
                    .write_all(
                        b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .unwrap();
            }
        });
        Self {
            stop,
            accepted,
            thread: Some(worker),
            connector,
        }
    }
}
impl Drop for DeniedPeer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.thread.take() {
            let result = worker.join();
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }
}

#[test]
fn public_stream_connection_headroom_is_shared_across_source_and_workspace_changes() {
    let folder = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(folder.path().into())));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let source = command(
        &control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        command(
            &control,
            "data.binance_market.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"]})
        )["ok"],
        true
    );
    command(
        &control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let peer = DeniedPeer::new();
    let supervisor = QuoteHotSupervisor::new();
    let mut last = json!(null);
    for n in 0..31 {
        if n == 1 {
            let current = command(
                &control,
                "data.binance_market.connection",
                json!({"workspaceId":workspace}),
            );
            assert_eq!(
                command(
                    &control,
                    "data.binance_market.configure",
                    json!({"workspaceId":workspace,"expectedStateVersion":current["data"]["stateVersion"]})
                )["ok"],
                true
            );
        }
        if n == 2 {
            assert_eq!(
                command(&control, "workspace.open", json!({}))["data"]["workspaceId"],
                workspace
            );
            command(
                &control,
                "time.revalidate",
                json!({"workspaceId":workspace}),
            );
        }
        let source = command(
            &control,
            "data.binance_market.connection",
            json!({"workspaceId":workspace}),
        );
        let acquired=supervisor.dispatch_with(&control,&json!({"requestId":format!("budget-{n}"),"schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","expectedSourceVersion":source["data"]["stateVersion"]}}),"main",Arc::new(NoAccess),Arc::new(NoAccess),QuoteStreamConnectors{stock:StockStreamConnector::default(),binance:peer.connector.clone()});
        assert_eq!(acquired["ok"], true, "{acquired}");
        let query = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            last = command(&control, "market.hot.get", query.clone());
            if last["data"]["status"] == "FAILED" {
                break;
            }
            assert!(Instant::now() < deadline, "{last}");
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            last["data"]["reconnectAttempt"], 0,
            "Actual feed denial or local allowance exhaustion cannot cause automatic retries"
        );
        assert!(last["data"]["detail"]["snapshot"].is_null());
    }
    supervisor.stop_all();
    assert_eq!(
        peer.accepted.load(Ordering::Acquire),
        30,
        "The31st actual connection must be blocked before I/O even after reselect/reopen: {last}"
    );
    assert!(
        last["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_RATE_LIMITED"),
        "{last}"
    );
}
