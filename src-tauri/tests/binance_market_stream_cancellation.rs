#![cfg(feature = "integration-test")]
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
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

#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod fixtures;
struct Vault(Option<Mutex<fixtures::Vault>>);
impl CredentialVault for Vault {
    fn put(&self, r: &str, c: &Credentials) -> Result<()> {
        self.0
            .as_ref()
            .expect("Public source cannot access a key")
            .lock()
            .unwrap()
            .put(r, c)
    }
    fn get(&self, r: &str) -> Result<Credentials> {
        self.0
            .as_ref()
            .expect("Public source cannot access a key")
            .lock()
            .unwrap()
            .get(r)
    }
    fn remove(&self, r: &str) -> Result<()> {
        self.0
            .as_ref()
            .expect("Public source cannot access a key")
            .lock()
            .unwrap()
            .remove(r)
    }
}
struct NoAccess;
impl CredentialVault for NoAccess {
    fn put(&self, _: &str, _: &Credentials) -> Result<()> {
        panic!("Public source cannot store a key")
    }
    fn get(&self, _: &str) -> Result<Credentials> {
        panic!("Public source cannot read a key")
    }
    fn remove(&self, _: &str) -> Result<()> {
        panic!("Public source cannot delete a key")
    }
}
impl ProviderHttp for NoAccess {
    fn get(&self, _: ProviderEndpoint, _: &str, _: reqwest::header::HeaderMap) -> Result<Vec<u8>> {
        panic!("An incomplete depth frame cannot start REST bootstrap")
    }
}
fn command(control: &Arc<Mutex<ControlPlane>>, name: &str, payload: Value) -> Value {
    control.lock().unwrap().dispatch_with_events(
        json!({"requestId":"fragment-cancel","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}

#[test]
fn releasing_a_trickling_incomplete_frame_allows_the_next_view_to_connect_promptly() {
    trickling_frame_handoff(false, true);
}
#[test]
fn retiring_a_stock_trickling_frame_allows_the_binance_view_to_connect_promptly() {
    trickling_frame_handoff(true, true);
}
#[test]
fn trickling_bootstrap_bytes_cannot_escape_the_total_deadline_or_create_quotes() {
    trickling_frame_handoff(false, false);
}
fn trickling_frame_handoff(stock_first: bool, retire: bool) {
    let folder = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(folder.path().into())));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let source = command(
        &control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &control,
        "data.binance_market.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let vault = Arc::new(Vault(
        stock_first.then(|| Mutex::new(fixtures::Vault::default())),
    ));
    let stock_saved = if stock_first {
        let current = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let result = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"stock-handoff-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":current["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            vault.as_ref(),
            fixtures::credentials,
        );
        assert_eq!(result["ok"], true, "{result}");
        result
    } else {
        Value::Null
    };
    command(
        &control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let connector = BinanceStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/ws/btcusdt@depth@100ms",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let stock_connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let halt = stop.clone();
    let (fragment_started, started) = mpsc::channel();
    let (next_started, next) = mpsc::channel();
    let worker = thread::spawn(move || {
        let accept = || {
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                match listener.accept() {
                    Ok((tcp, _)) => {
                        tcp.set_nonblocking(false).unwrap();
                        tcp.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                        return Some(tcp);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if halt.load(Ordering::Acquire) || Instant::now() > deadline {
                            return None;
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("{e}"),
                }
            }
        };
        let Some(tcp) = accept() else {
            return;
        };
        let mut socket = tungstenite::accept_hdr(
            tcp,
            |request: &tungstenite::handshake::server::Request,
             response: tungstenite::handshake::server::Response| {
                assert_eq!(
                    request.uri().path(),
                    if stock_first {
                        "/v2/sip"
                    } else {
                        "/ws/btcusdt@depth@100ms"
                    }
                );
                assert!(!request.headers().contains_key("x-mbx-apikey"));
                Ok(response)
            },
        )
        .unwrap();
        let raw = socket.get_mut();
        raw.set_read_timeout(Some(Duration::from_millis(20)))
            .unwrap();
        // A valid unmasked text-frame header advertises65535 bytes. Its payload
        // trickles more often than the client's per-read200ms timeout.
        raw.write_all(b"\x81\x7e\xff\xff{").unwrap();
        fragment_started.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(if retire { 2 } else { 12 });
        while !halt.load(Ordering::Acquire) && Instant::now() < deadline {
            let mut byte = [0; 4096];
            match raw.read(&mut byte) {
                Ok(0) => break,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    ()
                }
                Err(_) => break,
                Ok(_) => (),
            }
            if raw.write_all(b" ").is_err() {
                break;
            }
        }
        drop(socket);
        let Some(tcp) = accept() else {
            return;
        };
        let mut socket = tungstenite::accept(tcp).unwrap();
        next_started.send(()).unwrap();
        socket
            .get_mut()
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        while !halt.load(Ordering::Acquire) {
            match socket.read() {
                Ok(tungstenite::Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => {
                    break;
                }
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    ()
                }
                Err(_) => break,
                Ok(_) => (),
            }
        }
    });
    let supervisor = QuoteHotSupervisor::new();
    let send = |name: &str, payload: Value| {
        supervisor.dispatch_with(&control,&json!({"requestId":"fragment-view","schemaVersion":1,"command":name,"payload":payload}),"main",vault.clone(),Arc::new(NoAccess),QuoteStreamConnectors{stock:stock_connector.clone(),binance:connector.clone()})
    };
    let acquire = json!({"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","expectedSourceVersion":saved["data"]["stateVersion"]});
    let first_acquire = if stock_first {
        json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":stock_saved["data"]["stateVersion"]})
    } else {
        acquire.clone()
    };
    let first = send("market.hot.acquire", first_acquire);
    assert_eq!(first["ok"], true, "{first}");
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    let query = json!({"workspaceId":workspace,"leaseId":first["data"]["leaseId"],"generation":first["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(1);
    while command(&control, "market.hot.get", query.clone())["data"]["status"]
        != if stock_first {
            "AUTHENTICATING"
        } else {
            "AWAITING_QUOTE"
        }
    {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    // Allow multiple real payload chunks to arrive after the public connected
    // transition; cancelling during HTTP upgrade would test a different seam.
    thread::sleep(Duration::from_millis(75));
    let waiting = Instant::now();
    let second = if retire {
        let release = send(
            "market.hot.release",
            json!({"workspaceId":workspace,"leaseId":first["data"]["leaseId"],"generation":first["data"]["generation"]}),
        );
        assert_eq!(release["data"]["released"], true, "{release}");
        let second = send("market.hot.acquire", acquire);
        assert_eq!(second["ok"], true, "{second}");
        second
    } else {
        first.clone()
    };
    let connected = next.recv_timeout(if retire {
        Duration::from_millis(750)
    } else {
        Duration::from_secs(11)
    });
    let elapsed = waiting.elapsed();
    let current = command(
        &control,
        "market.hot.get",
        json!({"workspaceId":workspace,"leaseId":second["data"]["leaseId"],"generation":second["data"]["generation"]}),
    );
    supervisor.stop_all();
    stop.store(true, Ordering::Release);
    worker.join().unwrap();
    assert!(
        connected.is_ok(),
        "Trickling payload escaped cancellation/total bootstrap deadline (retire={retire}): {connected:?}"
    );
    if retire {
        assert_ne!(second["data"]["leaseId"], first["data"]["leaseId"]);
    } else {
        assert!(
            elapsed >= Duration::from_millis(9500) && elapsed < Duration::from_millis(10750),
            "The actual total10s bootstrap deadline and bounded backoff must end the trickling frame: {elapsed:?}"
        );
        assert_eq!(current["data"]["reconnectAttempt"], 1, "{current}");
        assert_ne!(
            current["data"]["connectionGeneration"], first["data"]["connectionGeneration"],
            "{current}"
        );
    }
    let detail = command(
        &control,
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"HOT"}),
    );
    assert!(
        detail["data"]["snapshot"].is_null(),
        "Neither handshake nor incomplete bytes establish quote evidence: {detail}"
    );
}
