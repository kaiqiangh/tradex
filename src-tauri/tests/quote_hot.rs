#![cfg(feature = "integration-test")]
use serde_json::{Value, json};
use std::{
    net::TcpListener,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};
use tradex::{
    ControlPlane,
    protocol::Result,
    provider_io::{CredentialVault, Credentials},
};
use tungstenite::Message;
#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod fixtures;

struct Vault(Mutex<fixtures::Vault>);
impl CredentialVault for Vault {
    fn put(&self, r: &str, c: &Credentials) -> Result<()> {
        self.0.lock().unwrap().put(r, c)
    }
    fn get(&self, r: &str) -> Result<Credentials> {
        self.0.lock().unwrap().get(r)
    }
    fn remove(&self, r: &str) -> Result<()> {
        self.0.lock().unwrap().remove(r)
    }
}
struct Http(Mutex<fixtures::Http>);
impl tradex::provider_io::ProviderHttp for Http {
    fn get(
        &self,
        e: tradex::provider_io::ProviderEndpoint,
        p: &str,
        h: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        self.0.lock().unwrap().get(e, p, h)
    }
    fn request(
        &self,
        e: tradex::provider_io::ProviderEndpoint,
        m: tradex::provider_io::ProviderHttpMethod,
        p: &str,
        h: reqwest::header::HeaderMap,
        b: Option<&Value>,
    ) -> Result<tradex::provider_io::ProviderHttpResponse> {
        self.0.lock().unwrap().request(e, m, p, h, b)
    }
}
fn command(control: &Arc<Mutex<ControlPlane>>, name: &str, payload: Value) -> Value {
    control.lock().unwrap().dispatch(
        json!({"requestId":"hot-test","schemaVersion":1,"command":name,"payload":payload}),
    )
}

#[test]
fn trickling_upgrade_has_total_deadline_and_release_cancels_the_actual_socket() {
    use std::io::{Read, Write};
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    for release in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace =
            command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let source = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
        let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"upgrade-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            vault.as_ref(),
            fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let connector = StockStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/v2/sip",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let (started, upgrading) = mpsc::channel();
        let (closed, closure) = mpsc::channel();
        let peer = thread::spawn(move || {
            let (mut tcp, _) = listener.accept().unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                tcp.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            let request = String::from_utf8(request).unwrap();
            assert!(request.starts_with("GET /v2/sip HTTP/1.1"));
            assert!(!request.contains(fixtures::KEY));
            assert!(!request.contains(fixtures::SECRET));
            tcp.write_all(b"HTTP/1.1 101 Switching Protocols\r\nX-Trickle: ")
                .unwrap();
            tcp.set_nonblocking(true).unwrap();
            let beginning = Instant::now();
            started.send(()).unwrap();
            let mut bytes = [0; 64];
            let observed = loop {
                match tcp.read(&mut bytes) {
                    Ok(0) => break Some(beginning.elapsed()),
                    Err(error) if error.kind() != std::io::ErrorKind::WouldBlock => {
                        break Some(beginning.elapsed());
                    }
                    _ => (),
                }
                if tcp.write_all(b"a").is_err() {
                    break Some(beginning.elapsed());
                }
                if beginning.elapsed() >= Duration::from_secs(6) {
                    break None;
                }
                thread::sleep(Duration::from_millis(50));
            };
            closed.send(observed).unwrap();
        });
        let supervisor = QuoteHotSupervisor::new();
        let acquired = supervisor.dispatch_with(&control, &json!({"requestId":"upgrade-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}), "main", vault.clone(), http.clone(), connector);
        assert_eq!(acquired["ok"], true, "{acquired}");
        upgrading.recv_timeout(Duration::from_secs(5)).unwrap();
        if release {
            let released = supervisor.dispatch_with(&control, &json!({"requestId":"upgrade-release","schemaVersion":1,"command":"market.hot.release","payload":{"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]}}), "main", vault, http, StockStreamConnector::default());
            assert_eq!(released["data"]["released"], true, "{released}");
        }
        let observed = closure.recv_timeout(Duration::from_secs(7)).unwrap();
        supervisor.stop_all();
        peer.join().unwrap();
        assert!(
            observed.is_some(),
            "Trickling response escaped the total handshake deadline (release={release})"
        );
        assert!(
            observed.unwrap()
                < if release {
                    Duration::from_millis(500)
                } else {
                    Duration::from_secs(5)
                },
            "Socket cleanup exceeded its cancellation/total deadline: {observed:?}"
        );
    }
}

#[test]
fn stream_receipt_precedes_slow_metadata_and_future_on_arrival_stays_rejected() {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use tradex::{
        provider_io::{ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse},
        quote_source::hot::{QuoteHotSupervisor, StockStreamConnector},
    };
    struct SlowMetadata {
        inner: Http,
        entered: Mutex<Option<OffsetDateTime>>,
    }
    impl ProviderHttp for SlowMetadata {
        fn get(
            &self,
            e: ProviderEndpoint,
            p: &str,
            h: reqwest::header::HeaderMap,
        ) -> Result<Vec<u8>> {
            self.request(e, ProviderHttpMethod::Get, p, h, None)
                .map(|r| r.body)
        }
        fn request(
            &self,
            e: ProviderEndpoint,
            m: ProviderHttpMethod,
            p: &str,
            h: reqwest::header::HeaderMap,
            b: Option<&Value>,
        ) -> Result<ProviderHttpResponse> {
            if p.contains("/meta/conditions/") {
                *self.entered.lock().unwrap() = Some(OffsetDateTime::now_utc());
                thread::sleep(Duration::from_millis(1200));
            }
            self.inner.request(e, m, p, h, b)
        }
    }
    for future in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace =
            command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let source = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"receipt-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            vault.as_ref(),
            fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        command(
            &control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let http = Arc::new(SlowMetadata {
            inner: Http(Mutex::new(fixtures::Http::default())),
            entered: Mutex::new(None),
        });
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let connector = StockStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/v2/sip",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let peer = thread::spawn(move || {
            let (tcp, _) = listener.accept().unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept(tcp).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
                "auth"
            );
            socket
                .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
                .unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
                "subscribe"
            );
            socket
                .send(Message::text(r#"[{"T":"subscription","quotes":["AAPL"]}]"#))
                .unwrap();
            let timestamp = (OffsetDateTime::now_utc()
                + if future {
                    time::Duration::milliseconds(600)
                } else {
                    -time::Duration::seconds(1)
                })
            .format(&Rfc3339)
            .unwrap();
            socket.send(Message::text(format!(r#"[{{"T":"q","S":"AAPL","t":"{timestamp}","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":310,"c":["R"],"z":"C"}}]"#))).unwrap();
            assert!(matches!(
                socket.read(),
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
            ));
        });
        let supervisor = QuoteHotSupervisor::new();
        let acquired = supervisor.dispatch_with(&control, &json!({"requestId":"receipt-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}), "main", vault.clone(), http.clone(), connector);
        assert_eq!(acquired["ok"], true, "{acquired}");
        let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        let deadline = Instant::now() + Duration::from_secs(5);
        let read = loop {
            let read = command(&control, "market.hot.get", lease.clone());
            if matches!(
                read["data"]["status"].as_str(),
                Some("STREAMING" | "FAILED")
            ) {
                break read;
            }
            assert!(Instant::now() < deadline, "{read}");
            thread::sleep(Duration::from_millis(5));
        };
        if future {
            assert_eq!(read["data"]["status"], "FAILED", "{read}");
            assert!(read["data"]["detail"]["snapshot"].is_null(), "{read}");
        } else {
            assert_eq!(read["data"]["status"], "STREAMING", "{read}");
            let received = OffsetDateTime::parse(
                read["data"]["detail"]["snapshot"]["provenance"]["receivedTimestamp"]
                    .as_str()
                    .unwrap(),
                &Rfc3339,
            )
            .unwrap();
            assert!(received <= http.entered.lock().unwrap().unwrap(), "{read}");
        }
        supervisor.dispatch_with(&control, &json!({"requestId":"receipt-release","schemaVersion":1,"command":"market.hot.release","payload":lease}), "main", vault, http, StockStreamConnector::default());
        supervisor.stop_all();
        peer.join().unwrap();
    }
}

#[test]
fn identical_stream_events_keep_first_receipt_and_unordered_conflicts_retire_current_evidence() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    for older in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace =
            command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
        let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
        let source = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"ordered-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            vault.as_ref(),
            fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        let now = command(
            &control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let event_time = time::OffsetDateTime::parse(
            now["data"]["wallClock"].as_str().unwrap(),
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap()
            - time::Duration::seconds(1);
        let timestamp = event_time
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let conflict_time = (event_time
            - if older {
                time::Duration::seconds(1)
            } else {
                time::Duration::ZERO
            })
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let connector = StockStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/v2/sip",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let (send_duplicate, duplicate_gate) = mpsc::channel();
        let (send_conflict, conflict_gate) = mpsc::channel();
        let peer = thread::spawn(move || {
            let (tcp, _) = listener.accept().unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept(tcp).unwrap();
            let auth: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(auth["action"], "auth");
            socket
                .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
                .unwrap();
            let subscribe: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(subscribe, json!({"action":"subscribe","quotes":["AAPL"]}));
            socket
                .send(Message::text(r#"[{"T":"subscription","quotes":["AAPL"]}]"#))
                .unwrap();
            let event = format!(
                r#"[{{"T":"q","S":"AAPL","t":"{timestamp}","bx":"P","bp":250.1,"bs":211,"ax":"Q","ap":250.2,"as":365,"c":["R"],"z":"C"}}]"#
            );
            socket.send(Message::text(event.clone())).unwrap();
            duplicate_gate.recv_timeout(Duration::from_secs(5)).unwrap();
            socket.send(Message::text(event)).unwrap();
            conflict_gate.recv_timeout(Duration::from_secs(5)).unwrap();
            socket.send(Message::text(format!(r#"[{{"T":"q","S":"AAPL","t":"{conflict_time}","bx":"P","bp":250.15,"bs":211,"ax":"Q","ap":250.25,"as":365,"c":["R"],"z":"C"}}]"#))).unwrap();
            match socket.read() {
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => (),
                other => panic!("Conflicting quote did not close socket: {other:?}"),
            }
        });
        let supervisor = QuoteHotSupervisor::new();
        let acquired = supervisor.dispatch_with(&control,&json!({"requestId":"ordered-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector);
        assert_eq!(acquired["ok"], true, "{acquired}");
        let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        let wait_for = |predicate: &dyn Fn(&Value) -> bool| {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                let result = command(&control, "market.hot.get", lease.clone());
                if predicate(&result) {
                    break result;
                }
                assert!(
                    Instant::now() < deadline,
                    "Hot projection did not reach expected state: {result}"
                );
                thread::sleep(Duration::from_millis(5));
            }
        };
        let first = wait_for(&|reply| reply["data"]["status"] == "STREAMING");
        assert_eq!(first["data"]["detail"]["status"], "AVAILABLE");
        send_duplicate.send(()).unwrap();
        let duplicate = wait_for(&|reply| {
            reply["data"]["sequence"].as_u64() > first["data"]["sequence"].as_u64()
        });
        assert_eq!(
            duplicate["data"]["detail"]["snapshot"], first["data"]["detail"]["snapshot"],
            "An identical stream event cannot renew receipt time or identity"
        );
        send_conflict.send(()).unwrap();
        let failed = wait_for(&|reply| reply["data"]["status"] == "FAILED");
        assert_eq!(failed["data"]["reconnectAttempt"], 0);
        assert!(
            failed["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("MARKET_QUOTE_CONFLICT")
        );
        assert_eq!(failed["data"]["detail"]["status"], "UNAVAILABLE");
        let snapshot = &failed["data"]["detail"]["snapshot"];
        assert_eq!(snapshot["provenance"]["freshness"], "STALE");
        assert_eq!(snapshot["bid"], "250.1");
        assert_eq!(
            snapshot["provenance"]["marketSnapshotId"],
            first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"]
        );
        peer.join().unwrap();
        supervisor.stop_all();
    }
}

#[test]
fn hot_needs_auth_full_subscription_and_real_quotes_and_release_closes_the_socket() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"hot-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let now = command(
        &control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let provider_time = (time::OffsetDateTime::parse(
        now["data"]["wallClock"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap()
        - time::Duration::seconds(1))
    .format(&time::format_description::well_known::Rfc3339)
    .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let (auth_seen, auth_ready) = mpsc::channel();
    let (allow_auth, auth_gate) = mpsc::channel();
    let (subscription_seen, subscription_ready) = mpsc::channel();
    let (allow_subscription, subscription_gate) = mpsc::channel();
    let (allow_quote, quote_gate) = mpsc::channel();
    let (closed_sender, closed_receiver) = mpsc::channel();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept_hdr(
            tcp,
            |request: &tungstenite::handshake::server::Request,
             response: tungstenite::handshake::server::Response| {
                assert_eq!(request.uri().path(), "/v2/sip");
                Ok(response)
            },
        )
        .unwrap();
        socket
            .send(Message::text(r#"[{"T":"success","msg":"connected"}]"#))
            .unwrap();
        let authentication: Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(
            authentication,
            json!({"action":"auth","key":fixtures::KEY,"secret":fixtures::SECRET})
        );
        auth_seen.send(()).unwrap();
        auth_gate.recv_timeout(Duration::from_secs(5)).unwrap();
        socket
            .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
            .unwrap();
        let subscription: Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(
            subscription,
            json!({"action":"subscribe","quotes":["AAPL"]})
        );
        subscription_seen.send(()).unwrap();
        subscription_gate
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        socket
            .send(Message::text(
                r#"[{"T":"subscription","quotes":["AAPL"],"trades":[],"bars":[]}]"#,
            ))
            .unwrap();
        quote_gate.recv_timeout(Duration::from_secs(5)).unwrap();
        socket.send(Message::text(format!(r#"[{{"T":"q","S":"AAPL","t":"{provider_time}","bx":"P","bp":250.7234567890123456789,"bs":211,"ax":"Q","ap":250.8234567890123456789,"as":365,"c":["R"],"z":"C"}}]"#))).unwrap();
        loop {
            match socket.read() {
                Ok(Message::Close(_)) => break,
                Ok(Message::Ping(payload)) => socket.send(Message::Pong(payload)).unwrap(),
                Ok(_) => (),
                Err(tungstenite::Error::ConnectionClosed) => break,
                Err(error) => panic!("Peer did not observe released connection: {error}"),
            }
        }
        closed_sender.send(()).unwrap();
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"acquire-hot","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector);
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    auth_ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let authenticating = command(&control, "market.hot.get", lease.clone());
    assert_eq!(authenticating["data"]["status"], "AUTHENTICATING");
    assert_eq!(authenticating["data"]["authenticated"], false);
    assert!(authenticating["data"]["detail"]["snapshot"].is_null());
    allow_auth.send(()).unwrap();
    subscription_ready
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let subscribing = command(&control, "market.hot.get", lease.clone());
    assert_eq!(subscribing["data"]["authenticated"], true);
    assert_eq!(subscribing["data"]["subscribed"], false);
    allow_subscription.send(()).unwrap();
    let wait_status = |status: &str| {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let result = command(&control, "market.hot.get", lease.clone());
            if result["data"]["status"] == status {
                break result;
            }
            assert!(Instant::now() < deadline, "Waiting for {status}: {result}");
            thread::sleep(Duration::from_millis(5));
        }
    };
    let awaiting = wait_status("AWAITING_QUOTE");
    assert!(awaiting["data"]["detail"]["snapshot"].is_null());
    assert!(
        awaiting["data"]["detail"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("Subscription confirmed; waiting for an actual provider quote."),
        "Configured and subscribed source must explain missing quote accurately: {awaiting}"
    );
    assert_eq!(awaiting["data"]["detail"]["status"], "UNAVAILABLE");
    allow_quote.send(()).unwrap();
    let streaming = wait_status("STREAMING");
    assert_eq!(
        streaming["data"]["detail"]["snapshot"]["bid"],
        "250.7234567890123456789"
    );
    assert_eq!(
        streaming["data"]["detail"]["snapshot"]["ask"],
        "250.8234567890123456789"
    );
    assert_eq!(streaming["data"]["detail"]["snapshot"]["askSize"], "365");
    assert_eq!(streaming["data"]["detail"]["status"], "AVAILABLE");
    assert_eq!(
        streaming["data"]["detail"]["snapshot"]["provenance"]["alpaca"]["connectionGeneration"],
        lease["generation"]
    );
    let shared = command(
        &control,
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
    );
    assert_eq!(
        shared["data"]["snapshot"],
        streaming["data"]["detail"]["snapshot"]
    );
    for secret in [fixtures::KEY, fixtures::SECRET] {
        assert!(!streaming.to_string().contains(secret));
    }
    // A later HTTP observation is valid for Warm reads but is not a Hot subscription frame.
    let time = command(&control, "time.status", json!({"workspaceId":workspace}));
    let timestamp = time["data"]["wallClock"].as_str().unwrap();
    http.0.lock().unwrap().alpaca_quote_body.replace(Some(format!(r#"{{"quotes":{{"AAPL":{{"t":"{timestamp}","bx":"P","bp":251.1234567890123456789,"bs":205,"ax":"Q","ap":251.2234567890123456789,"as":310,"c":["R"],"z":"C"}}}}}}"#).into_bytes()));
    let warm = tradex::quote_source::execute_market(
        &control,
        &json!({"requestId":"warm-after-hot","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"WARM"}}),
        "main",
        vault.as_ref(),
        http.as_ref(),
    );
    assert_eq!(warm["data"]["status"], "AVAILABLE", "{warm}");
    let hot_after_http = command(&control, "market.hot.get", lease.clone());
    assert_eq!(
        hot_after_http["data"]["detail"]["status"], "UNAVAILABLE",
        "An HTTP refresh cannot be relabelled as a subscription update"
    );
    let released=supervisor.dispatch_with(&control,&json!({"requestId":"release-hot","schemaVersion":1,"command":"market.hot.release","payload":lease.clone()}),"main",vault,http,StockStreamConnector::default());
    assert_eq!(released["data"]["status"], "CLOSED");
    closed_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    peer.join().unwrap();
    supervisor.stop_all();
    assert_eq!(
        command(&control, "market.hot.get", lease)["data"]["status"],
        "CLOSED"
    );
}

#[test]
fn replacement_closes_old_socket_and_late_auth_or_old_release_cannot_activate_or_stop_new_lease() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"hot-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let (first_sender, first_receiver) = mpsc::channel();
    let (late_sender, late_receiver) = mpsc::channel();
    let (second_sender, second_receiver) = mpsc::channel();
    let peer = thread::spawn(move || {
        for index in 0..2 {
            let (tcp, _) = listener.accept().unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept(tcp).unwrap();
            let auth: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(auth["action"], "auth");
            if index == 0 {
                first_sender.send(()).unwrap();
                late_receiver.recv_timeout(Duration::from_secs(5)).unwrap();
                let _ = socket.send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#));
            } else {
                second_sender.send(()).unwrap();
            }
            // The old worker must close before a second TCP connection can authenticate.
            match socket.read() {
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => (),
                other => {
                    panic!("Cancelled lease sent a subscription or left its socket open: {other:?}")
                }
            }
        }
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquire = |symbol: &str| {
        supervisor.dispatch_with(&control,&json!({"requestId":"hot-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":symbol,"expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector.clone())
    };
    let first = acquire("equity:US:AAPL");
    assert_eq!(first["ok"], true, "{first}");
    first_receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    let first_lease = json!({"workspaceId":workspace,"leaseId":first["data"]["leaseId"],"generation":first["data"]["generation"]});
    let second = acquire("equity:US:MSFT");
    assert_eq!(second["ok"], true, "{second}");
    late_sender.send(()).unwrap();
    second_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let second_lease = json!({"workspaceId":workspace,"leaseId":second["data"]["leaseId"],"generation":second["data"]["generation"]});
    let stale_release=supervisor.dispatch_with(&control,&json!({"requestId":"release-old","schemaVersion":1,"command":"market.hot.release","payload":first_lease}),"main",vault.clone(),http.clone(),connector.clone());
    assert_eq!(stale_release["data"]["released"], false);
    let current = command(&control, "market.hot.get", second_lease.clone());
    assert_eq!(current["data"]["status"], "AUTHENTICATING");
    assert_eq!(current["data"]["authenticated"], false);
    assert!(current["data"]["detail"]["snapshot"].is_null());
    let release=supervisor.dispatch_with(&control,&json!({"requestId":"release-new","schemaVersion":1,"command":"market.hot.release","payload":second_lease}),"main",vault,http,connector);
    assert_eq!(release["data"]["released"], true);
    peer.join().unwrap();
    supervisor.stop_all();
}

#[test]
fn transient_reconnect_keeps_view_ownership_but_invalidates_quotes_until_new_acknowledged_frame() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"reconnect-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let time = command(
        &control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let timestamp = (time::OffsetDateTime::parse(
        time["data"]["wallClock"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap()
        - time::Duration::seconds(1))
    .format(&time::format_description::well_known::Rfc3339)
    .unwrap();
    let quote = format!(
        r#"[{{"T":"q","S":"AAPL","t":"{timestamp}","bx":"P","bp":250.7234567890123456789,"bs":211,"ax":"Q","ap":250.8234567890123456789,"as":365,"c":["R"],"z":"C"}}]"#
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let (allow_disconnect, disconnect_gate) = mpsc::channel();
    let (second_auth_seen, second_auth_ready) = mpsc::channel();
    let (allow_second_auth, second_auth_gate) = mpsc::channel();
    let (allow_second_quote, second_quote_gate) = mpsc::channel();
    let peer = thread::spawn(move || {
        for attempt in 0..2 {
            let deadline = Instant::now() + Duration::from_secs(5);
            let tcp = loop {
                match listener.accept() {
                    Ok((tcp, _)) => break tcp,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() > deadline {
                            return;
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("Accept failed: {error}"),
                }
            };
            tcp.set_nonblocking(false).unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept(tcp).unwrap();
            let auth: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(auth["action"], "auth");
            if attempt == 1 {
                second_auth_seen.send(()).unwrap();
                second_auth_gate
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
            }
            socket
                .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
                .unwrap();
            let subscribe: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(subscribe, json!({"action":"subscribe","quotes":["AAPL"]}));
            socket
                .send(Message::text(
                    r#"[{"T":"subscription","quotes":["AAPL"],"trades":[],"bars":[]}]"#,
                ))
                .unwrap();
            if attempt == 1 {
                second_quote_gate
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
            }
            socket.send(Message::text(quote.clone())).unwrap();
            if attempt == 0 {
                disconnect_gate
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
                socket.close(None).unwrap();
            }
            match socket.read() {
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => (),
                other => panic!("Socket did not close: {other:?}"),
            }
        }
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"reconnect-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector.clone());
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let wait = |status: &str| {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let current = command(&control, "market.hot.get", lease.clone());
            if current["data"]["status"] == status {
                break current;
            }
            assert!(Instant::now() < deadline, "Waiting for {status}: {current}");
            thread::sleep(Duration::from_millis(5));
        }
    };
    let first = wait("STREAMING");
    assert_eq!(first["data"]["detail"]["status"], "AVAILABLE");
    let old_snapshot =
        first["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"].clone();
    let old_generation =
        first["data"]["detail"]["snapshot"]["provenance"]["alpaca"]["connectionGeneration"].clone();
    allow_disconnect.send(()).unwrap();
    second_auth_ready
        .recv_timeout(Duration::from_secs(5))
        .expect("Transient closure must automatically reconnect the existing lease");
    let reconnecting = command(&control, "market.hot.get", lease.clone());
    assert_eq!(reconnecting["data"]["authenticated"], false);
    assert_eq!(reconnecting["data"]["subscribed"], false);
    assert_eq!(reconnecting["data"]["generation"], lease["generation"]);
    assert_ne!(reconnecting["data"]["connectionGeneration"], old_generation);
    assert_eq!(reconnecting["data"]["reconnectAttempt"], 1);
    assert_eq!(
        reconnecting["data"]["detail"]["snapshot"]["provenance"]["freshness"],
        "STALE"
    );
    assert_eq!(reconnecting["data"]["detail"]["status"], "UNAVAILABLE");
    allow_second_auth.send(()).unwrap();
    let acknowledged = wait("AWAITING_QUOTE");
    assert_eq!(acknowledged["data"]["detail"]["status"], "UNAVAILABLE");
    assert_eq!(
        acknowledged["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        old_snapshot
    );
    allow_second_quote.send(()).unwrap();
    let recovered = wait("STREAMING");
    assert_eq!(recovered["data"]["detail"]["status"], "AVAILABLE");
    assert_ne!(
        recovered["data"]["detail"]["snapshot"]["provenance"]["marketSnapshotId"],
        old_snapshot
    );
    assert_eq!(
        recovered["data"]["detail"]["snapshot"]["provenance"]["alpaca"]["connectionGeneration"],
        recovered["data"]["connectionGeneration"]
    );
    let released=supervisor.dispatch_with(&control,&json!({"requestId":"reconnect-release","schemaVersion":1,"command":"market.hot.release","payload":lease}),"main",vault,http,connector);
    assert_eq!(released["data"]["released"], true);
    peer.join().unwrap();
    supervisor.stop_all();
}

#[test]
fn repeated_provider_internal_errors_stop_after_three_retries_without_feed_fallback() {
    repeated_stream_error_stops_after_three_retries(500, "PROVIDER_UNAVAILABLE");
}

#[test]
fn slow_client_errors_stop_after_three_retries_without_feed_fallback() {
    repeated_stream_error_stops_after_three_retries(407, "PROVIDER_STREAM_STALE");
}

fn repeated_stream_error_stops_after_three_retries(code: u64, reason: &str) {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"retry-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let (observed_sender, observed_receiver) = mpsc::channel();
    let peer = thread::spawn(move || {
        for connection in 1..=4 {
            let deadline = Instant::now() + Duration::from_secs(5);
            let tcp = loop {
                match listener.accept() {
                    Ok((tcp, _)) => break tcp,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "Expected connection {connection}"
                        );
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("Accept failed: {error}"),
                }
            };
            tcp.set_nonblocking(false).unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept_hdr(
                tcp,
                |request: &tungstenite::handshake::server::Request,
                 response: tungstenite::handshake::server::Response| {
                    assert_eq!(request.uri().path(), "/v2/sip");
                    Ok(response)
                },
            )
            .unwrap();
            let auth: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(auth["action"], "auth");
            socket
                .send(Message::text(
                    json!([{"T":"error","code":code,"msg":fixtures::SECRET}]).to_string(),
                ))
                .unwrap();
            match socket.read() {
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => (),
                other => panic!("Retry did not close socket: {other:?}"),
            }
            observed_sender.send(connection).unwrap();
        }
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"retry-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector.clone());
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(8);
    let failed = loop {
        let projection = command(&control, "market.hot.get", lease.clone());
        if projection["data"]["status"] == "FAILED" {
            break projection;
        }
        assert!(Instant::now() < deadline, "Unbounded retry: {projection}");
        thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(failed["data"]["reconnectAttempt"], 3);
    assert_eq!(failed["data"]["authenticated"], false);
    assert_eq!(failed["data"]["subscribed"], false);
    assert!(failed["data"]["detail"]["snapshot"].is_null());
    assert!(failed["data"]["reason"].as_str().unwrap().contains(reason));
    assert!(!failed.to_string().contains(fixtures::SECRET));
    for connection in 1..=4 {
        assert_eq!(
            observed_receiver
                .recv_timeout(Duration::from_secs(1))
                .unwrap(),
            connection
        );
    }
    peer.join().unwrap();
    let retained = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(retained["data"]["feed"], "sip");
    assert_eq!(
        retained["data"]["stateVersion"],
        saved["data"]["stateVersion"]
    );
    assert_eq!(retained["data"]["status"], "UNAVAILABLE");
    let _=supervisor.dispatch_with(&control,&json!({"requestId":"retry-release","schemaVersion":1,"command":"market.hot.release","payload":lease}),"main",vault,http,connector);
    supervisor.stop_all();
}

#[test]
fn feed_authentication_and_connection_limits_are_terminal_and_cannot_become_acknowledgements() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    for (code, reason) in [
        (402, "PROVIDER_AUTH_FAILED"),
        (404, "PROVIDER_AUTH_TIMEOUT"),
        (405, "PROVIDER_SYMBOL_LIMIT"),
        (409, "DATA_SOURCE_FEED_DENIED"),
        (406, "PROVIDER_CONNECTION_LIMIT"),
        (403, "PROVIDER_ALREADY_AUTHENTICATED"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace =
            command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
        let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
        let source = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"denied-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            vault.as_ref(),
            fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let connector = StockStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/v2/sip",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let peer = thread::spawn(move || {
            let (tcp, _) = listener.accept().unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept(tcp).unwrap();
            let auth: Value =
                serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
            assert_eq!(auth["action"], "auth");
            socket
                .send(Message::text(
                    json!([{"T":"error","code":code,"msg":fixtures::SECRET}]).to_string(),
                ))
                .unwrap();
            match socket.read() {
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => (),
                other => panic!("Permanent error sent another action: {other:?}"),
            }
        });
        let supervisor = QuoteHotSupervisor::new();
        let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"denied-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector.clone());
        assert_eq!(acquired["ok"], true, "{acquired}");
        let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        let deadline = Instant::now() + Duration::from_secs(5);
        let failed = loop {
            let result = command(&control, "market.hot.get", lease.clone());
            if result["data"]["status"] == "FAILED" {
                break result;
            }
            assert!(
                Instant::now() < deadline,
                "Permanent stream error retried: {result}"
            );
            thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(failed["data"]["reconnectAttempt"], 0);
        assert_eq!(failed["data"]["authenticated"], false);
        assert_eq!(failed["data"]["subscribed"], false);
        assert!(failed["data"]["detail"]["snapshot"].is_null());
        assert!(
            failed["data"]["reason"].as_str().unwrap().contains(reason),
            "{failed}"
        );
        assert!(!failed.to_string().contains(fixtures::SECRET));
        let retained = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        assert_eq!(retained["data"]["feed"], "sip");
        assert_eq!(
            retained["data"]["stateVersion"],
            saved["data"]["stateVersion"]
        );
        assert_eq!(retained["data"]["status"], "UNAVAILABLE");
        peer.join().unwrap();
        let _=supervisor.dispatch_with(&control,&json!({"requestId":"denied-release","schemaVersion":1,"command":"market.hot.release","payload":lease}),"main",vault,http,connector);
        supervisor.stop_all();
    }
}

#[test]
fn public_session_safety_trigger_retires_pending_stream_authentication_and_keeps_source_selection()
{
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"session-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let (auth_seen, auth_ready) = mpsc::channel();
    let (allow_late_auth, late_auth_gate) = mpsc::channel();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        let auth: Value = serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(auth["action"], "auth");
        auth_seen.send(()).unwrap();
        late_auth_gate.recv_timeout(Duration::from_secs(5)).unwrap();
        let _ = socket.send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#));
        match socket.read() {
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => (),
            other => panic!("Safety trigger allowed a late subscription: {other:?}"),
        }
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"session-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector.clone());
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    auth_ready.recv_timeout(Duration::from_secs(5)).unwrap();
    // Invoke the same public handler used by native notification observers; no real OS Sleep is asserted.
    control
        .lock()
        .unwrap()
        .disarm_live_for_safety("OS_SLEEP")
        .unwrap();
    let retired = command(&control, "market.hot.get", lease.clone());
    assert_eq!(retired["data"]["status"], "STALE");
    assert_eq!(retired["data"]["authenticated"], false);
    assert_eq!(retired["data"]["subscribed"], false);
    assert!(retired["data"]["detail"]["snapshot"].is_null());
    allow_late_auth.send(()).unwrap();
    peer.join().unwrap();
    let retained = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        retained["data"]["stateVersion"],
        saved["data"]["stateVersion"]
    );
    assert_eq!(retained["data"]["feed"], "sip");
    assert_eq!(retained["data"]["status"], "UNVERIFIED");
    let _=supervisor.dispatch_with(&control,&json!({"requestId":"session-release","schemaVersion":1,"command":"market.hot.release","payload":lease}),"main",vault,http,connector);
    supervisor.stop_all();
}

#[test]
fn oversized_provider_frames_fail_once_without_retrying_or_creating_quotes() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"frame-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        let auth: Value = serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(auth["action"], "auth");
        // The client may reject the frame header before the whole body is sent.
        if let Err(error) = socket.send(Message::text("X".repeat(300_000))) {
            assert!(
                matches!(&error, tungstenite::Error::Io(io) if matches!(
                    io.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe
                )),
                "Unexpected oversized-frame send failure: {error}"
            );
            return;
        }
        let _ = socket.read();
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"frame-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector.clone());
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(5);
    let failed = loop {
        let result = command(&control, "market.hot.get", lease.clone());
        if result["data"]["status"] == "FAILED" {
            break result;
        }
        assert!(
            Instant::now() < deadline,
            "Invalid frame was retried: {result}"
        );
        thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(failed["data"]["reconnectAttempt"], 0);
    assert!(
        failed["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_RESPONSE_INVALID")
    );
    assert!(failed["data"]["detail"]["snapshot"].is_null());
    peer.join().unwrap();
    let _=supervisor.dispatch_with(&control,&json!({"requestId":"frame-release","schemaVersion":1,"command":"market.hot.release","payload":lease}),"main",vault,http,connector);
    supervisor.stop_all();
}

#[test]
fn last_supervisor_owner_closes_the_socket_and_retires_the_public_hot_projection() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"owner-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let (closed_sender, closed_receiver) = mpsc::channel();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        let auth: Value = serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(auth["action"], "auth");
        socket
            .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
            .unwrap();
        let subscription: Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(
            subscription,
            json!({"action":"subscribe","quotes":["AAPL"]})
        );
        socket
            .send(Message::text(r#"[{"T":"subscription","quotes":["AAPL"]}]"#))
            .unwrap();
        match socket.read() {
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => {
                closed_sender.send(()).unwrap()
            }
            other => panic!("Dropping the final owner left the provider socket open: {other:?}"),
        }
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"owner-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault,http,connector);
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let state = command(&control, "market.hot.get", lease.clone());
        if state["data"]["status"] == "AWAITING_QUOTE" {
            break;
        }
        assert!(Instant::now() < deadline, "{state}");
        thread::sleep(Duration::from_millis(5));
    }
    let final_owner = supervisor.clone();
    drop(supervisor);
    assert!(
        matches!(closed_receiver.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "An intermediate owner must not stop the shared supervisor"
    );
    drop(final_owner);
    closed_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("Final supervisor owner must close the actual provider socket");
    peer.join().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let retired = loop {
        let state = command(&control, "market.hot.get", lease.clone());
        if state["data"]["status"] == "CLOSED" {
            break state;
        }
        assert!(
            Instant::now() < deadline,
            "Stopped stream still appeared active: {state}"
        );
        thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(retired["data"]["authenticated"], false);
    assert_eq!(retired["data"]["subscribed"], false);
    assert_eq!(retired["data"]["detail"]["status"], "UNAVAILABLE");
    assert!(retired["data"]["detail"]["snapshot"].is_null());
}

#[test]
fn explicit_stop_closes_the_connection_and_rejected_acquire_preserves_the_retired_lease() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"stop-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        let auth: Value = serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(auth["action"], "auth");
        socket
            .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
            .unwrap();
        let subscribe: Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(subscribe, json!({"action":"subscribe","quotes":["AAPL"]}));
        socket
            .send(Message::text(r#"[{"T":"subscription","quotes":["AAPL"]}]"#))
            .unwrap();
        assert!(matches!(
            socket.read(),
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
        ));
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquire = json!({"requestId":"stop-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}});
    let acquired = supervisor.dispatch_with(
        &control,
        &acquire,
        "main",
        vault.clone(),
        http.clone(),
        connector.clone(),
    );
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let result = command(&control, "market.hot.get", lease.clone());
        if result["data"]["status"] == "AWAITING_QUOTE" {
            break;
        }
        assert!(Instant::now() < deadline, "{result}");
        thread::sleep(Duration::from_millis(5));
    }
    supervisor.stop_all();
    peer.join().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let result = command(&control, "market.hot.get", lease.clone());
        if result["data"]["status"] == "CLOSED" {
            break;
        }
        assert!(Instant::now() < deadline, "{result}");
        thread::sleep(Duration::from_millis(5));
    }
    let rejected = supervisor.dispatch_with(
        &control,
        &acquire,
        "main",
        vault.clone(),
        http.clone(),
        connector.clone(),
    );
    assert_eq!(
        rejected["error"]["code"], "PROVIDER_UNAVAILABLE",
        "{rejected}"
    );
    let retired = command(&control, "market.hot.get", lease);
    assert_eq!(
        retired["ok"], true,
        "Rejected acquire must preserve the current lease: {retired}"
    );
    assert_eq!(retired["data"]["status"], "CLOSED");
    let unused = QuoteHotSupervisor::new();
    unused.stop_all();
    let rejected = unused.dispatch_with(&control, &acquire, "main", vault, http, connector);
    assert_eq!(
        rejected["error"]["code"], "PROVIDER_UNAVAILABLE",
        "An unused stopped manager must reject work: {rejected}"
    );
}

#[test]
fn an_unrequested_subscription_channel_cannot_confirm_the_hot_lease_or_create_quotes() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"channels-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
            "auth"
        );
        socket
            .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap(),
            json!({"action":"subscribe","quotes":["AAPL"]})
        );
        socket
            .send(Message::text(
                r#"[{"T":"subscription","quotes":["AAPL"],"options":["*"]}]"#,
            ))
            .unwrap();
        assert!(matches!(
            socket.read(),
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
        ));
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"channels-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault,http,connector);
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(2);
    let failed = loop {
        let result = command(&control, "market.hot.get", lease.clone());
        if result["data"]["status"] == "FAILED" {
            break result;
        }
        assert!(
            Instant::now() < deadline,
            "Unrequested channel was accepted as a complete subscription: {result}"
        );
        thread::sleep(Duration::from_millis(5));
    };
    assert!(
        failed["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("PROVIDER_SUBSCRIPTION_INVALID")
    );
    assert_eq!(failed["data"]["subscribed"], false);
    assert_eq!(failed["data"]["reconnectAttempt"], 0);
    assert!(failed["data"]["detail"]["snapshot"].is_null());
    supervisor.stop_all();
    peer.join().unwrap();
}

#[test]
fn unrelated_pongs_do_not_acknowledge_our_heartbeat_or_keep_a_dead_connection_current() {
    use std::io::Read;
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"heartbeat-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let (pong_sent, pong_seen) = mpsc::channel();
    let (second_auth_sender, second_auth_seen) = mpsc::channel();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(35))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
            "auth"
        );
        socket
            .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap(),
            json!({"action":"subscribe","quotes":["AAPL"]})
        );
        socket
            .send(Message::text(r#"[{"T":"subscription","quotes":["AAPL"]}]"#))
            .unwrap();
        // Read the wire Ping directly so this adversarial peer does not automatically echo it.
        let mut header = [0u8; 2];
        socket.get_mut().read_exact(&mut header).unwrap();
        assert_eq!(header[0] & 15, 9);
        assert_ne!(header[1] & 128, 0);
        let length = (header[1] & 127) as usize;
        assert!(length <= 125);
        let mut mask = [0u8; 4];
        socket.get_mut().read_exact(&mut mask).unwrap();
        let mut payload = vec![0; length];
        socket.get_mut().read_exact(&mut payload).unwrap();
        socket
            .send(Message::Pong(b"unsolicited-health-pulse".to_vec().into()))
            .unwrap();
        pong_sent.send(()).unwrap();
        assert!(
            matches!(
                socket.read(),
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
            ),
            "Unrelated Pong must not prevent heartbeat timeout"
        );
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
            "auth"
        );
        second_auth_sender.send(()).unwrap();
        assert!(matches!(
            socket.read(),
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
        ));
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"heartbeat-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault,http,connector);
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    pong_seen.recv_timeout(Duration::from_secs(25)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(13);
    let retired = loop {
        let result = command(&control, "market.hot.get", lease.clone());
        if result["data"]["reconnectAttempt"] == 1 {
            break result;
        }
        assert!(
            Instant::now() < deadline,
            "Unrelated Pong kept a dead connection active: {result}"
        );
        thread::sleep(Duration::from_millis(25));
    };
    assert_eq!(
        retired["data"]["generation"],
        acquired["data"]["generation"]
    );
    assert_ne!(
        retired["data"]["connectionGeneration"],
        acquired["data"]["connectionGeneration"]
    );
    assert_eq!(retired["data"]["subscribed"], false);
    assert!(retired["data"]["detail"]["snapshot"].is_null());
    second_auth_seen
        .recv_timeout(Duration::from_secs(3))
        .unwrap();
    supervisor.stop_all();
    peer.join().unwrap();
}

#[test]
fn cancelled_secure_vault_read_preserves_selection_and_never_opens_a_provider_socket() {
    use tradex::{
        protocol::TradeXError,
        quote_source::hot::{QuoteHotSupervisor, StockStreamConnector},
    };
    struct CancelledVault(Mutex<fixtures::Vault>);
    impl CredentialVault for CancelledVault {
        fn put(&self, r: &str, c: &Credentials) -> Result<()> {
            self.0.lock().unwrap().put(r, c)
        }
        fn get(&self, _: &str) -> Result<Credentials> {
            Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"))
        }
        fn remove(&self, r: &str) -> Result<()> {
            self.0.lock().unwrap().remove(r)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(CancelledVault(Mutex::new(fixtures::Vault::default())));
    let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"cancel-read-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let supervisor = QuoteHotSupervisor::new();
    let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"cancel-read-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector);
    assert_eq!(acquired["ok"], true, "{acquired}");
    let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
    let deadline = Instant::now() + Duration::from_secs(2);
    let failed = loop {
        let result = command(&control, "market.hot.get", lease.clone());
        if result["data"]["status"] == "FAILED" {
            break result;
        }
        assert!(Instant::now() < deadline, "{result}");
        thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(failed["data"]["reconnectAttempt"], 0);
    assert_eq!(failed["data"]["authenticated"], false);
    assert_eq!(failed["data"]["subscribed"], false);
    assert!(
        failed["data"]["reason"]
            .as_str()
            .unwrap()
            .contains("CREDENTIAL_UNAVAILABLE")
    );
    assert!(failed["data"]["detail"]["snapshot"].is_null());
    assert!(matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock));
    assert!(http.0.lock().unwrap().calls.borrow().is_empty());
    let retained = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(retained["data"]["feed"], "sip");
    assert_eq!(
        retained["data"]["stateVersion"],
        saved["data"]["stateVersion"]
    );
    assert_eq!(retained["data"]["status"], "UNAVAILABLE");
    let disconnected = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"cancel-read-disconnect","schemaVersion":1,"command":"data.source.disconnect","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        vault.as_ref(),
        || panic!("Disconnect must not open secure entry"),
    );
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    assert_eq!(disconnected["data"]["cleanupPending"], false);
    assert_eq!(disconnected["data"]["configured"], false);
    supervisor.stop_all();
}

#[test]
fn source_disconnect_and_workspace_reopen_close_the_socket_before_late_authentication_can_subscribe()
 {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    for disconnect in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace =
            command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
        let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
        let source = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"retire-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            vault.as_ref(),
            fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let connector = StockStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/v2/sip",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let (auth_seen, auth_ready) = mpsc::channel();
        let (allow_late_auth, late_auth_gate) = mpsc::channel();
        let peer = thread::spawn(move || {
            let (tcp, _) = listener.accept().unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept(tcp).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
                "auth"
            );
            auth_seen.send(()).unwrap();
            late_auth_gate.recv_timeout(Duration::from_secs(5)).unwrap();
            let _ = socket.send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#));
            assert!(
                matches!(
                    socket.read(),
                    Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
                ),
                "Retired source/workspace allowed late authentication to subscribe"
            );
        });
        let supervisor = QuoteHotSupervisor::new();
        let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"retire-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector);
        assert_eq!(acquired["ok"], true, "{acquired}");
        let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        auth_ready.recv_timeout(Duration::from_secs(5)).unwrap();
        if disconnect {
            let result = tradex::quote_source::execute_configuration(
                &control,
                &json!({"requestId":"retire-disconnect","schemaVersion":1,"command":"data.source.disconnect","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
                "main",
                vault.as_ref(),
                || panic!("Disconnect cannot capture credentials"),
            );
            assert_eq!(result["ok"], true, "{result}");
            assert_eq!(result["data"]["configured"], false);
            assert_eq!(result["data"]["cleanupPending"], false);
        } else {
            let reopened = command(&control, "workspace.open", json!({}));
            assert_eq!(reopened["ok"], true, "{reopened}");
            assert_eq!(reopened["data"]["workspaceId"], workspace);
        }
        assert_eq!(
            command(&control, "market.hot.get", lease)["error"]["code"],
            "STATE_VERSION_CONFLICT"
        );
        allow_late_auth.send(()).unwrap();
        peer.join().unwrap();
        let market = command(
            &control,
            "market.get",
            json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
        );
        assert_eq!(market["ok"], true, "{market}");
        assert!(market["data"]["snapshot"].is_null());
        assert_ne!(market["data"]["status"], "AVAILABLE");
        assert!(http.0.lock().unwrap().calls.borrow().is_empty());
        if !disconnect {
            let retained = command(
                &control,
                "data.source.connection",
                json!({"workspaceId":workspace}),
            );
            assert_eq!(retained["data"]["feed"], "sip");
            assert_eq!(
                retained["data"]["stateVersion"],
                saved["data"]["stateVersion"]
            );
            assert_eq!(retained["data"]["status"], "UNVERIFIED");
        }
        supervisor.stop_all();
    }
}

#[test]
fn malformed_crossed_future_and_unbound_stream_quotes_never_create_execution_evidence() {
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    for scenario in [
        "malformed",
        "crossed",
        "zero",
        "wrong-symbol",
        "unknown-condition",
        "wrong-tape",
        "future",
        "before-subscription",
        "mixed-control-quote",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace =
            command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
        let http = Arc::new(Http(Mutex::new(fixtures::Http::default())));
        let source = command(
            &control,
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"invalid-quote-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            vault.as_ref(),
            fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let connector = StockStreamConnector::for_loopback_test(&format!(
            "ws://127.0.0.1:{}/v2/sip",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let peer = thread::spawn(move || {
            let (tcp, _) = listener.accept().unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut socket = tungstenite::accept(tcp).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
                "auth"
            );
            socket
                .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
                .unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap(),
                json!({"action":"subscribe","quotes":["AAPL"]})
            );
            if !matches!(scenario, "before-subscription" | "mixed-control-quote") {
                socket
                    .send(Message::text(r#"[{"T":"subscription","quotes":["AAPL"]}]"#))
                    .unwrap();
            }
            let mut frame = json!({"T":"q","S":"AAPL","t":"2026-09-30T14:10:00.123456789Z","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":310,"c":["R"],"z":"C"});
            match scenario {
                "crossed" => frame["bp"] = json!(300),
                "zero" => frame["ap"] = json!(0),
                "wrong-symbol" => frame["S"] = "MSFT".into(),
                "unknown-condition" => frame["c"] = json!(["ZZ"]),
                "wrong-tape" => frame["z"] = "A".into(),
                "future" => frame["t"] = "2099-01-01T00:00:00.123456789Z".into(),
                _ => (),
            }
            socket
                .send(Message::text(if scenario == "malformed" {
                    "not-json".into()
                } else if scenario == "mixed-control-quote" {
                    json!([{"T":"subscription","quotes":["AAPL"]}, frame]).to_string()
                } else {
                    json!([frame]).to_string()
                }))
                .unwrap();
            assert!(
                matches!(
                    socket.read(),
                    Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
                ),
                "Invalid quote must close without another action: {scenario}"
            );
        });
        let supervisor = QuoteHotSupervisor::new();
        let acquired=supervisor.dispatch_with(&control,&json!({"requestId":"invalid-quote-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault,http,connector);
        assert_eq!(acquired["ok"], true, "{acquired}");
        let lease = json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]});
        let deadline = Instant::now() + Duration::from_secs(3);
        let failed = loop {
            let result = command(&control, "market.hot.get", lease.clone());
            if result["data"]["status"] == "FAILED" {
                break result;
            }
            assert!(
                Instant::now() < deadline,
                "Invalid quote became stream evidence ({scenario}): {result}"
            );
            thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(failed["data"]["reconnectAttempt"], 0);
        assert_eq!(failed["data"]["subscribed"], false);
        assert!(
            failed["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("PROVIDER_RESPONSE_INVALID"),
            "{failed}"
        );
        assert!(failed["data"]["detail"]["snapshot"].is_null());
        assert!(!failed.to_string().contains(fixtures::KEY));
        assert!(!failed.to_string().contains(fixtures::SECRET));
        supervisor.stop_all();
        peer.join().unwrap();
    }
}

#[test]
fn pending_metadata_does_not_block_public_commands_and_old_feed_cannot_publish_after_source_change()
{
    use tradex::provider_io::{
        ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse,
    };
    use tradex::quote_source::hot::{QuoteHotSupervisor, StockStreamConnector};
    struct PausedHttp {
        inner: Http,
        entered: mpsc::Sender<()>,
        gate: Mutex<Option<mpsc::Receiver<()>>>,
    }
    impl ProviderHttp for PausedHttp {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> Result<Vec<u8>> {
            self.request(endpoint, ProviderHttpMethod::Get, path, headers, None)
                .map(|response| response.body)
        }
        fn request(
            &self,
            endpoint: ProviderEndpoint,
            method: ProviderHttpMethod,
            path: &str,
            headers: reqwest::header::HeaderMap,
            body: Option<&Value>,
        ) -> Result<ProviderHttpResponse> {
            if path == "/v2/stocks/meta/conditions/quote?tape=C"
                && let Some(gate) = self.gate.lock().unwrap().take()
            {
                self.entered.send(()).unwrap();
                gate.recv_timeout(Duration::from_secs(5)).unwrap();
            }
            self.inner.request(endpoint, method, path, headers, body)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace = command(&control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Arc::new(Vault(Mutex::new(fixtures::Vault::default())));
    let source = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"paused-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let (entered, reading) = mpsc::channel();
    let (resume, gate) = mpsc::channel();
    let http = Arc::new(PausedHttp {
        inner: Http(Mutex::new(fixtures::Http::default())),
        entered,
        gate: Mutex::new(Some(gate)),
    });
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connector = StockStreamConnector::for_loopback_test(&format!(
        "ws://127.0.0.1:{}/v2/sip",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let peer = thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut socket = tungstenite::accept(tcp).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap()["action"],
            "auth"
        );
        socket
            .send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(socket.read().unwrap().to_text().unwrap()).unwrap(),
            json!({"action":"subscribe","quotes":["AAPL"]})
        );
        socket
            .send(Message::text(r#"[{"T":"subscription","quotes":["AAPL"]}]"#))
            .unwrap();
        socket.send(Message::text(r#"[{"T":"q","S":"AAPL","t":"2026-09-30T14:10:00.123456789Z","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":310,"c":["R"],"z":"C"}]"#)).unwrap();
        assert!(
            matches!(
                socket.read(),
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed)
            ),
            "Old metadata job retained its connection or sent a new action"
        );
    });
    let supervisor = QuoteHotSupervisor::new();
    let acquired = supervisor.dispatch_with(&control,&json!({"requestId":"paused-acquire","schemaVersion":1,"command":"market.hot.acquire","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","expectedSourceVersion":saved["data"]["stateVersion"]}}),"main",vault.clone(),http.clone(),connector);
    assert_eq!(acquired["ok"], true, "{acquired}");
    reading
        .recv_timeout(Duration::from_secs(3))
        .expect("Provider metadata read must reach the external boundary");
    let (responded, response) = mpsc::channel();
    let query_control = control.clone();
    let query_workspace = workspace.clone();
    let query = thread::spawn(move || {
        responded
            .send(command(
                &query_control,
                "time.revalidate",
                json!({"workspaceId":query_workspace}),
            ))
            .unwrap();
    });
    assert_eq!(
        response
            .recv_timeout(Duration::from_secs(2))
            .expect("Metadata I/O must not hold the Control Plane lock")["ok"],
        true
    );
    query.join().unwrap();
    let changed = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"paused-change","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"],"feed":"iex","credential":{"kind":"DEDICATED"}}}),
        "main",
        vault.as_ref(),
        fixtures::credentials,
    );
    assert_eq!(changed["ok"], true, "{changed}");
    resume.send(()).unwrap();
    peer.join().unwrap();
    let market = command(
        &control,
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
    );
    assert_eq!(market["ok"], true, "{market}");
    assert!(
        market["data"]["snapshot"].is_null(),
        "Old feed created a quote after source replacement: {market}"
    );
    assert_ne!(market["data"]["status"], "AVAILABLE");
    let current = command(
        &control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(current["data"]["feed"], "iex");
    assert_eq!(
        current["data"]["stateVersion"],
        changed["data"]["stateVersion"]
    );
    assert_eq!(current["data"]["status"], "UNVERIFIED");
    assert_eq!(
        command(
            &control,
            "market.hot.get",
            json!({"workspaceId":workspace,"leaseId":acquired["data"]["leaseId"],"generation":acquired["data"]["generation"]})
        )["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    supervisor.stop_all();
}
