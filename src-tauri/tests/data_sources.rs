use serde_json::{Value, json};
use tradex::ControlPlane;

#[test]
fn selected_sip_quotes_reach_market_with_exact_provenance_and_identical_reads_keep_receipt() {
    use std::sync::{Arc, Mutex};
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let source = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let vault = provider_fixtures::Vault::default();
    let http = provider_fixtures::Http::default();
    let configured = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"market-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        &vault,
        provider_fixtures::credentials,
    );
    assert_eq!(configured["ok"], true, "{configured}");
    assert!(
        http.calls.borrow().is_empty(),
        "Saving source selection cannot subscribe or fetch the universe"
    );
    let now = command(
        &mut control.lock().unwrap(),
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let timestamp = (OffsetDateTime::parse(now["data"]["wallClock"].as_str().unwrap(), &Rfc3339)
        .unwrap()
        - time::Duration::seconds(1))
    .format(&Rfc3339)
    .unwrap();
    *http.alpaca_quote_body.borrow_mut()=Some(format!(r#"{{"quotes":{{"AAPL":{{"t":"{timestamp}","bx":"P","bp":250.1234567890123456789,"bs":205,"ax":"Q","ap":250.2234567890123456789,"as":310,"c":["R"],"z":"C"}}}}}}"#).into_bytes());
    let request = json!({"requestId":"market-read","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}});
    let read = tradex::quote_source::execute_market(&control, &request, "main", &vault, &http);
    assert_eq!(read["ok"], true, "{read}");
    let snapshot = &read["data"]["snapshot"];
    assert_eq!(snapshot["bid"], "250.1234567890123456789");
    assert_eq!(snapshot["ask"], "250.2234567890123456789");
    assert_eq!(snapshot["bidSize"], "205");
    assert_eq!(snapshot["askSize"], "310");
    assert!(
        snapshot["lastPrice"].is_null(),
        "A quote cannot invent a last trade"
    );
    let provenance = &snapshot["provenance"];
    assert_eq!(provenance["providerTimestamp"], timestamp);
    assert_eq!(provenance["venue"], "US_SIP");
    assert_eq!(provenance["freshness"], "HEALTHY");
    assert_eq!(provenance["alpaca"]["feed"], "sip");
    assert_eq!(provenance["alpaca"]["providerSymbol"], "AAPL");
    assert_eq!(provenance["alpaca"]["listingVenue"], "XNAS");
    assert_eq!(provenance["alpaca"]["bidExchange"], "P");
    assert_eq!(provenance["alpaca"]["askExchange"], "Q");
    assert_eq!(provenance["alpaca"]["depthUnit"], "BASE");
    assert_eq!(provenance["alpaca"]["regularConditions"], true);
    assert_eq!(
        provenance["alpaca"]["sourceVersion"],
        configured["data"]["stateVersion"]
    );
    assert_eq!(
        http.calls.borrow().len(),
        3,
        "One requested quote and authenticated condition/exchange metadata, without polling"
    );
    let repeated = tradex::quote_source::execute_market(&control, &request, "main", &vault, &http);
    assert_eq!(
        repeated["data"]["snapshot"], *snapshot,
        "An identical provider event retains identity and first receipt"
    );
    let cached = command(
        &mut control.lock().unwrap(),
        "market.get",
        request["payload"].clone(),
    );
    assert_eq!(
        cached["data"]["snapshot"], *snapshot,
        "The risk read shares the accepted observation without network I/O"
    );
    http.alpaca_quote_status.set(403);
    let denied = tradex::quote_source::execute_market(&control, &request, "main", &vault, &http);
    assert_eq!(denied["data"]["status"], "UNAVAILABLE");
    assert_eq!(
        denied["data"]["snapshot"]["provenance"]["freshness"],
        "STALE"
    );
    assert_eq!(
        denied["data"]["snapshot"]["provenance"]["marketSnapshotId"],
        provenance["marketSnapshotId"]
    );
    http.alpaca_quote_status.set(200);
    let recovered = tradex::quote_source::execute_market(&control, &request, "main", &vault, &http);
    assert_eq!(recovered["data"]["status"], "AVAILABLE", "{recovered}");
    assert_ne!(
        recovered["data"]["snapshot"]["provenance"]["marketSnapshotId"],
        provenance["marketSnapshotId"],
        "Reauthenticated evidence after failure invalidates prior consent"
    );
    assert_ne!(
        recovered["data"]["snapshot"]["provenance"]["alpaca"]["connectionGeneration"],
        provenance["alpaca"]["connectionGeneration"]
    );
    for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
        assert!(!read.to_string().contains(secret));
    }
    command(&mut control.lock().unwrap(), "workspace.open", json!({}));
    let reopened = command(
        &mut control.lock().unwrap(),
        "market.get",
        request["payload"].clone(),
    );
    assert!(
        reopened["data"]["snapshot"].is_null(),
        "No persisted tick or inherited readiness on reopen"
    );
}

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": "data-source-test",
        "schemaVersion": 1,
        "command": name,
        "payload": payload
    }))
}

#[test]
fn quote_receipt_precedes_slow_metadata_and_future_on_arrival_stays_rejected() {
    use std::{
        cell::Cell,
        sync::{Arc, Mutex},
        time::Duration,
    };
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use tradex::provider_io::{
        ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse,
    };
    struct SlowMetadata {
        inner: provider_fixtures::Http,
        entered: Cell<Option<OffsetDateTime>>,
    }
    impl ProviderHttp for SlowMetadata {
        fn get(
            &self,
            e: ProviderEndpoint,
            p: &str,
            h: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
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
        ) -> tradex::protocol::Result<ProviderHttpResponse> {
            if p.contains("/meta/conditions/") {
                self.entered.set(Some(OffsetDateTime::now_utc()));
                std::thread::sleep(Duration::from_millis(1200));
            }
            self.inner.request(e, m, p, h, b)
        }
    }
    for future in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace = command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]
            ["workspaceId"]
            .clone();
        let source = command(
            &mut control.lock().unwrap(),
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let vault = provider_fixtures::Vault::default();
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"receipt-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            &vault,
            provider_fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        command(
            &mut control.lock().unwrap(),
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let http = SlowMetadata {
            inner: provider_fixtures::Http::default(),
            entered: Cell::new(None),
        };
        let timestamp = (OffsetDateTime::now_utc()
            + if future {
                time::Duration::milliseconds(600)
            } else {
                -time::Duration::seconds(1)
            })
        .format(&Rfc3339)
        .unwrap();
        *http.inner.alpaca_quote_body.borrow_mut() = Some(format!(r#"{{"quotes":{{"AAPL":{{"t":"{timestamp}","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":310,"c":["R"],"z":"C"}}}}}}"#).into_bytes());
        let read = tradex::quote_source::execute_market(
            &control,
            &json!({"requestId":"receipt-read","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}}),
            "main",
            &vault,
            &http,
        );
        assert_eq!(read["ok"], true, "{read}");
        if future {
            assert!(
                read["data"]["snapshot"].is_null(),
                "Metadata delay cannot legalize a future-on-arrival event: {read}"
            );
            assert_eq!(read["data"]["status"], "UNAVAILABLE");
        } else {
            let received = OffsetDateTime::parse(
                read["data"]["snapshot"]["provenance"]["receivedTimestamp"]
                    .as_str()
                    .unwrap(),
                &Rfc3339,
            )
            .unwrap();
            assert!(
                received <= http.entered.get().unwrap(),
                "Receipt must precede metadata enrichment: {read}"
            );
        }
    }
}

#[cfg(feature = "integration-test")]
#[test]
fn quote_http_auth_denial_and_quota_retire_evidence_without_fallback_or_cooldown_storm() {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };
    for (status, reason) in [
        (401, "PROVIDER_AUTH_FAILED"),
        (403, "DATA_SOURCE_FEED_DENIED"),
        (429, "PROVIDER_RATE_LIMITED"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let control = Arc::new(Mutex::new(ControlPlane::new(
            directory.path().join("workspace"),
        )));
        let workspace = command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]
            ["workspaceId"]
            .clone();
        let source = command(
            &mut control.lock().unwrap(),
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let vault = provider_fixtures::Vault::default();
        let saved = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"http-fault-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
            "main",
            &vault,
            provider_fixtures::credentials,
        );
        assert_eq!(saved["ok"], true, "{saved}");
        let time = command(
            &mut control.lock().unwrap(),
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
        let body = format!(
            r#"{{"quotes":{{"AAPL":{{"t":"{timestamp}","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":310,"c":["R"],"z":"C"}}}}}}"#
        );
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let http = tradex::provider_io::BrokerHttp::for_loopback_test(&format!(
            "http://127.0.0.1:{}",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();
        let for_peer = control.clone();
        let peer = std::thread::spawn(move || {
            let mut denied_at: Option<Instant> = None;
            let mut requests = Vec::new();
            for index in 0..7 {
                let deadline = Instant::now() + Duration::from_secs(6);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "Expected read {index}");
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("Accept failed: {error}"),
                    }
                };
                if index == 4 && status == 429 {
                    assert!(
                        denied_at.unwrap().elapsed() >= Duration::from_secs(1),
                        "Provider received a read before its Retry-After cooldown"
                    );
                }
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let expected = match index {
                    0 | 3 | 4 => "/v2/stocks/quotes/latest?symbols=AAPL&feed=sip&currency=USD",
                    1 | 5 => "/v2/stocks/meta/conditions/quote?tape=C",
                    2 | 6 => "/v2/stocks/meta/exchanges",
                    _ => unreachable!(),
                };
                assert_eq!(line, format!("GET {expected} HTTP/1.1\r\n"));
                requests.push(expected.to_owned());
                loop {
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                }
                assert!(
                    for_peer.try_lock().is_ok(),
                    "HTTP read held Control Plane lock"
                );
                let (code, response) = if index == 3 {
                    (
                        status,
                        format!(
                            "private error {} {}",
                            provider_fixtures::KEY,
                            provider_fixtures::SECRET
                        ),
                    )
                } else {
                    (
                        200,
                        match index {
                            0 | 4 => body.clone(),
                            1 | 5 => r#"{"R":"Regular"}"#.into(),
                            2 | 6 => r#"{"P":"NYSE Arca","Q":"Nasdaq"}"#.into(),
                            _ => unreachable!(),
                        },
                    )
                };
                if index == 3 {
                    denied_at = Some(Instant::now());
                }
                let retry = if code == 429 {
                    "Retry-After: 1\r\n"
                } else {
                    ""
                };
                write!(stream,"HTTP/1.1 {code} Result\r\n{retry}Content-Length: {}\r\nConnection: close\r\n\r\n{response}",response.len()).unwrap();
            }
            assert!(listener.accept().is_err(), "Extra retry or feed fallback");
            requests
        });
        let request = json!({"requestId":"http-fault-read","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"WARM"}});
        let first = tradex::quote_source::execute_market(&control, &request, "main", &vault, &http);
        assert_eq!(first["data"]["status"], "AVAILABLE", "{first}");
        let failed =
            tradex::quote_source::execute_market(&control, &request, "main", &vault, &http);
        assert_eq!(failed["data"]["status"], "UNAVAILABLE", "{failed}");
        assert_eq!(
            failed["data"]["snapshot"]["provenance"]["freshness"],
            "STALE"
        );
        assert_eq!(
            failed["data"]["snapshot"]["provenance"]["marketSnapshotId"],
            first["data"]["snapshot"]["provenance"]["marketSnapshotId"]
        );
        let connection = command(
            &mut control.lock().unwrap(),
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        assert_eq!(connection["data"]["feed"], "sip");
        assert_eq!(connection["data"]["status"], "UNAVAILABLE");
        assert!(
            connection["data"]["availabilityReason"]
                .as_str()
                .unwrap()
                .contains(reason)
        );
        assert!(connection["data"]["verifiedAt"].is_null());
        let recovered =
            tradex::quote_source::execute_market(&control, &request, "main", &vault, &http);
        assert_eq!(recovered["data"]["status"], "AVAILABLE", "{recovered}");
        assert_ne!(
            recovered["data"]["snapshot"]["provenance"]["marketSnapshotId"],
            first["data"]["snapshot"]["provenance"]["marketSnapshotId"]
        );
        for reply in [&first, &failed, &connection, &recovered] {
            for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
                assert!(!reply.to_string().contains(secret));
            }
        }
        assert_eq!(peer.join().unwrap().len(), 7);
    }
}

#[test]
fn catalog_and_credentialed_probe_are_scoped_and_read_only() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let before = command(
        &mut control,
        "domain.snapshot",
        json!({"aggregateType":"workspace","aggregateId":workspace_id}),
    );
    let catalog = command(
        &mut control,
        "data.source.catalog",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(catalog["ok"], true, "{catalog}");
    assert_eq!(catalog["data"]["sources"].as_array().unwrap().len(), 6);
    let version = catalog["data"]["stateVersion"].as_str().unwrap();
    let alpaca = command(
        &mut control,
        "data.source.probe",
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":version}),
    );
    assert_eq!(alpaca["ok"], true, "{alpaca}");
    assert_eq!(
        alpaca["data"]["sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["sourceId"] == "OD-001")
            .unwrap()["status"],
        "BLOCKED_EXTERNAL"
    );
    assert_eq!(
        command(
            &mut control,
            "data.source.probe",
            json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":"stale"}),
        )["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    assert_eq!(
        command(
            &mut control,
            "data.source.probe",
            json!({"workspaceId":workspace_id,"sourceId":"OD-999","expectedStateVersion":version}),
        )["error"]["code"],
        "DATA_SOURCE_UNKNOWN"
    );
    assert_eq!(
        command(
            &mut control,
            "data.source.catalog",
            json!({"workspaceId":"other"}),
        )["error"]["code"],
        "IPC_AGGREGATE_NOT_FOUND"
    );
    for payload in [
        json!({"workspaceId":workspace_id,"sourceId":"","expectedStateVersion":version}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001\n","expectedStateVersion":version}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":""}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":"x".repeat(257)}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":"bad\nversion"}),
    ] {
        assert_eq!(
            command(&mut control, "data.source.probe", payload)["error"]["code"],
            "IPC_PAYLOAD_INVALID"
        );
    }
    let after = command(
        &mut control,
        "domain.snapshot",
        json!({"aggregateType":"workspace","aggregateId":workspace_id}),
    );
    assert_eq!(
        before, after,
        "source catalog/probe must not mutate domain state"
    );
}

#[test]
fn selected_quote_source_has_independent_configuration_and_retains_no_live_readiness() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path);
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let connection = command(
        &mut control,
        "data.source.connection",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(connection["ok"], true, "{connection}");
    assert_eq!(connection["data"]["sourceId"], "OD-001");
    assert_eq!(connection["data"]["configured"], false);
    assert_eq!(connection["data"]["status"], "BLOCKED_EXTERNAL");
    assert!(connection["data"]["feed"].is_null());
    assert!(connection["data"]["verifiedAt"].is_null());
    assert_eq!(connection["data"]["eligibleAccounts"], json!([]));
    assert!(
        connection["data"]["stateVersion"]
            .as_str()
            .unwrap()
            .starts_with("data:")
    );
    assert_eq!(
        command(
            &mut control,
            "data.source.connection",
            json!({"workspaceId":"other"})
        )["error"]["code"],
        "IPC_AGGREGATE_NOT_FOUND"
    );
}

#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod provider_fixtures;

#[test]
fn user_can_reuse_an_alpaca_reference_and_reopen_the_explicit_feed_without_entitlement() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let http = provider_fixtures::Http::default();
    let request = json!({"requestId":"setup-alpaca-source","schemaVersion":1,"command":"provider.connect","payload":{
        "step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"My Paper account"
    }});
    let job = control.prepare_provider(&request).unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| provider_fixtures::credentials(),
        &http,
        || control.provider_job_current(&job),
    );
    let tested = control.complete_provider(&job, outcome);
    assert_eq!(tested["ok"], true, "{tested}");
    let confirmed = command(
        &mut control,
        "provider.connect",
        json!({
            "step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],
            "expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":true
        }),
    );
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    let current = command(
        &mut control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        current["data"]["eligibleAccounts"][0]["connectionId"],
        confirmed["data"]["connectionId"]
    );
    let configured = control.dispatch_with_events(json!({"requestId":"choose-source","schemaVersion":1,"command":"data.source.configure","payload":{
        "workspaceId":workspace,"expectedStateVersion":current["data"]["stateVersion"],"feed":"iex",
        "credential":{"kind":"EXISTING_ACCOUNT","connectionId":confirmed["data"]["connectionId"]}
    }}), "main", None);
    assert_eq!(configured["ok"], true, "{configured}");
    assert_eq!(configured["data"]["configured"], true);
    assert_eq!(configured["data"]["feed"], "iex");
    assert_eq!(configured["data"]["status"], "UNVERIFIED");
    let source_catalog = command(
        &mut control,
        "data.source.catalog",
        json!({"workspaceId":workspace}),
    );
    let selected_source = source_catalog["data"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|source| source["sourceId"] == "OD-001")
        .unwrap();
    assert_eq!(selected_source["configured"], true);
    assert_eq!(selected_source["status"], "UNVERIFIED");
    assert!(
        selected_source["coverage"]
            .as_str()
            .unwrap()
            .contains("IEX")
    );
    assert!(configured["data"]["verifiedAt"].is_null());
    for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
        assert!(!configured.to_string().contains(secret));
    }
    let control = std::sync::Arc::new(std::sync::Mutex::new(control));
    let verified = tradex::quote_source::execute_probe(
        &control,
        &json!({"requestId":"verify-reused-key","schemaVersion":1,"command":"data.source.probe","payload":{"workspaceId":workspace,"sourceId":"OD-001","expectedStateVersion":source_catalog["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(verified["ok"], true, "{verified}");
    assert_eq!(
        command(
            &mut control.lock().unwrap(),
            "data.source.connection",
            json!({"workspaceId":workspace})
        )["data"]["status"],
        "AVAILABLE"
    );
    let quote_request = json!({"requestId":"reused-key-quote","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}});
    let before_refresh =
        tradex::quote_source::execute_market(&control, &quote_request, "main", &vault, &http);
    assert_eq!(before_refresh["ok"], true, "{before_refresh}");
    let refresh = json!({"requestId":"refresh-reused-account","schemaVersion":1,"command":"account.refresh","payload":{"workspaceId":workspace,"connectionId":confirmed["data"]["connectionId"],"expectedStateVersion":confirmed["data"]["stateVersion"]}});
    let job = control
        .lock()
        .unwrap()
        .prepare_provider(&refresh)
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| panic!("Saved-key refresh cannot open secure entry"),
        &http,
        || control.lock().unwrap().provider_job_current(&job),
    );
    let refreshed = control.lock().unwrap().complete_provider(&job, outcome);
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    let changed = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        changed["data"]["status"], "UNVERIFIED",
        "Source verification is bound to the reused account version, not an old key observation"
    );
    assert_eq!(
        changed["data"]["stateVersion"],
        configured["data"]["stateVersion"]
    );
    let after_refresh =
        tradex::quote_source::execute_market(&control, &quote_request, "main", &vault, &http);
    assert_eq!(after_refresh["ok"], true, "{after_refresh}");
    assert_ne!(
        before_refresh["data"]["snapshot"]["provenance"]["marketSnapshotId"],
        after_refresh["data"]["snapshot"]["provenance"]["marketSnapshotId"]
    );
    assert_ne!(
        before_refresh["data"]["snapshot"]["provenance"]["alpaca"]["connectionGeneration"],
        after_refresh["data"]["snapshot"]["provenance"]["alpaca"]["connectionGeneration"]
    );
    drop(control);
    let mut reopened = ControlPlane::new(path);
    command(&mut reopened, "workspace.open", json!({}));
    let retained = command(
        &mut reopened,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(retained["data"]["feed"], "iex");
    assert_eq!(
        retained["data"]["stateVersion"],
        configured["data"]["stateVersion"]
    );
    assert_eq!(retained["data"]["status"], "UNVERIFIED");
    assert!(retained["data"]["verifiedAt"].is_null());
    let accounts_before = command(
        &mut reopened,
        "account.list",
        json!({"workspaceId":workspace}),
    );
    let disconnect_request = |version: Value| {
        json!({"requestId":"disconnect-source","schemaVersion":1,"command":"data.source.disconnect","payload":{
            "workspaceId":workspace,"expectedStateVersion":version
        }})
    };
    let stale = reopened.dispatch_with_events(
        disconnect_request(current["data"]["stateVersion"].clone()),
        "main",
        None,
    );
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    let disconnected = reopened.dispatch_with_events(
        disconnect_request(retained["data"]["stateVersion"].clone()),
        "main",
        None,
    );
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    assert_eq!(disconnected["data"]["configured"], false);
    assert_eq!(disconnected["data"]["status"], "BLOCKED_EXTERNAL");
    assert!(disconnected["data"]["feed"].is_null());
    assert_ne!(
        disconnected["data"]["stateVersion"],
        retained["data"]["stateVersion"]
    );
    assert_eq!(
        command(
            &mut reopened,
            "account.list",
            json!({"workspaceId":workspace})
        ),
        accounts_before
    );
}

#[test]
fn dedicated_data_key_is_native_only_and_disconnect_reports_owned_key_cleanup() {
    use std::sync::{Arc, Mutex};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let initial = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let configure = json!({"requestId":"dedicated-data-key","schemaVersion":1,"command":"data.source.configure","payload":{
        "workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}
    }});
    let vault = provider_fixtures::Vault::default();
    let saved =
        tradex::quote_source::execute_configuration(&control, &configure, "main", &vault, || {
            let mut available = control
                .try_lock()
                .expect("Native credential entry must not hold the Control Plane lock");
            assert_eq!(
                command(
                    &mut available,
                    "data.source.connection",
                    json!({"workspaceId":workspace})
                )["ok"],
                true
            );
            provider_fixtures::credentials()
        });
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["configured"], true);
    assert_eq!(saved["data"]["credentialKind"], "DEDICATED");
    assert_eq!(saved["data"]["status"], "UNVERIFIED");
    assert!(saved["data"]["accountId"].is_null());
    assert_eq!(saved["data"]["cleanupPending"], false);
    let accounts = command(
        &mut control.lock().unwrap(),
        "account.list",
        json!({"workspaceId":workspace}),
    );
    assert!(
        !accounts["data"]["accounts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|account| account["providerId"] == "alpaca")
    );
    for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
        assert!(!saved.to_string().contains(secret));
    }
    vault.fail_remove.set(true);
    let disconnect = json!({"requestId":"disconnect-dedicated-key","schemaVersion":1,"command":"data.source.disconnect","payload":{
        "workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]
    }});
    let disconnected =
        tradex::quote_source::execute_configuration(&control, &disconnect, "main", &vault, || {
            panic!("Disconnect cannot open credential entry")
        });
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    assert_eq!(disconnected["data"]["configured"], false);
    assert_eq!(
        disconnected["data"]["cleanupPending"], true,
        "Failed Keychain deletion must remain visibly pending"
    );
    vault.fail_remove.set(false);
    let retry = json!({"requestId":"retry-owned-cleanup","schemaVersion":1,"command":"data.source.cleanup","payload":{"workspaceId":workspace}});
    let cleaned =
        tradex::quote_source::execute_configuration(&control, &retry, "main", &vault, || {
            panic!("Cleanup cannot open credential entry")
        });
    assert_eq!(cleaned["ok"], true, "{cleaned}");
    assert_eq!(cleaned["data"]["cleanupPending"], false);
    assert_eq!(
        cleaned["data"]["stateVersion"],
        disconnected["data"]["stateVersion"]
    );
}

#[test]
fn cancelled_and_stale_secure_entry_preserve_the_current_selection_and_restart_never_verifies_it() {
    use std::sync::{Arc, Mutex};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let control = Arc::new(Mutex::new(ControlPlane::new(path.clone())));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let connection = || {
        command(
            &mut control.lock().unwrap(),
            "data.source.connection",
            json!({"workspaceId":workspace}),
        )
    };
    let configure = |version: Value, feed: &str| {
        json!({"requestId":"replace-data-key","schemaVersion":1,"command":"data.source.configure","payload":{
            "workspaceId":workspace,"expectedStateVersion":version,"feed":feed,"credential":{"kind":"DEDICATED"}
        }})
    };
    let vault = provider_fixtures::Vault::default();
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &configure(connection()["data"]["stateVersion"].clone(), "sip"),
        "main",
        &vault,
        provider_fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let owned_before = vault.present.borrow().clone();
    let cancelled = tradex::quote_source::execute_configuration(
        &control,
        &configure(saved["data"]["stateVersion"].clone(), "iex"),
        "main",
        &vault,
        || {
            Err(tradex::protocol::TradeXError::new(
                "PROVIDER_ENTRY_CANCELLED",
            ))
        },
    );
    assert_eq!(cancelled["error"]["code"], "PROVIDER_ENTRY_CANCELLED");
    assert_eq!(connection()["data"], saved["data"]);
    assert_eq!(*vault.present.borrow(), owned_before);

    let stale = tradex::quote_source::execute_configuration(
        &control,
        &configure(saved["data"]["stateVersion"].clone(), "iex"),
        "main",
        &vault,
        || {
            let newer = tradex::quote_source::execute_configuration(
                &control,
                &configure(saved["data"]["stateVersion"].clone(), "delayed_sip"),
                "main",
                &vault,
                provider_fixtures::credentials,
            );
            assert_eq!(newer["ok"], true, "{newer}");
            provider_fixtures::credentials()
        },
    );
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    let current = connection();
    assert_eq!(current["data"]["feed"], "delayed_sip");
    assert_eq!(
        vault.present.borrow().len(),
        1,
        "Only the current dedicated key may survive a stale entry"
    );
    drop(control);
    let mut reopened = ControlPlane::new(path);
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(
        &mut reopened,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(restored["data"], current["data"]);
    assert_eq!(restored["data"]["status"], "UNVERIFIED");
    assert!(restored["data"]["verifiedAt"].is_null());
}

#[test]
fn interrupted_keychain_write_is_visible_after_restart_and_can_only_clean_its_owned_key() {
    use std::sync::{Arc, Mutex};
    use tradex::provider_io::{CredentialVault, Credentials};
    struct InterruptedVault<'a>(&'a provider_fixtures::Vault);
    impl CredentialVault for InterruptedVault<'_> {
        fn put(&self, reference: &str, credentials: &Credentials) -> tradex::protocol::Result<()> {
            self.0.put(reference, credentials)?;
            panic!("external vault completed its write before process interruption")
        }
        fn get(&self, reference: &str) -> tradex::protocol::Result<Credentials> {
            self.0.get(reference)
        }
        fn remove(&self, reference: &str) -> tradex::protocol::Result<()> {
            self.0.remove(reference)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let control = Arc::new(Mutex::new(ControlPlane::new(path.clone())));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let original = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let vault = provider_fixtures::Vault::default();
    vault
        .put(
            "unrelated-broker-key",
            &provider_fixtures::credentials().unwrap(),
        )
        .unwrap();
    let request = json!({"requestId":"interrupted-data-entry","schemaVersion":1,"command":"data.source.configure","payload":{
        "workspaceId":workspace,"expectedStateVersion":original["data"]["stateVersion"],"feed":"iex","credential":{"kind":"DEDICATED"}
    }});
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tradex::quote_source::execute_configuration(
            &control,
            &request,
            "main",
            &InterruptedVault(&vault),
            provider_fixtures::credentials,
        )
    }));
    assert!(interrupted.is_err());
    drop(control);
    let reopened = Arc::new(Mutex::new(ControlPlane::new(path)));
    assert_eq!(
        command(&mut reopened.lock().unwrap(), "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(
        &mut reopened.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(restored["data"]["configured"], false);
    assert_eq!(restored["data"]["cleanupPending"], true);
    assert_eq!(
        restored["data"]["stateVersion"],
        original["data"]["stateVersion"]
    );
    let cleaned = tradex::quote_source::execute_configuration(
        &reopened,
        &json!({"requestId":"cleanup-after-interruption","schemaVersion":1,"command":"data.source.cleanup","payload":{"workspaceId":workspace}}),
        "main",
        &vault,
        || panic!("Cleanup cannot open credential entry"),
    );
    assert_eq!(cleaned["ok"], true, "{cleaned}");
    assert_eq!(cleaned["data"]["cleanupPending"], false);
    assert_eq!(
        *vault.present.borrow(),
        std::collections::HashSet::from(["unrelated-broker-key".into()])
    );
}

#[test]
fn source_secure_entry_rejects_untrusted_consumers_and_renderer_secrets_before_capture() {
    use std::sync::{Arc, Mutex};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let initial = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let request = json!({"requestId":"untrusted-source-entry","schemaVersion":1,"command":"data.source.configure","payload":{
        "workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}
    }});
    let vault = provider_fixtures::Vault::default();
    let no_entry = || panic!("Rejected requests cannot open native secure entry");
    for consumer in ["agent", "foreign-window"] {
        let rejected = tradex::quote_source::execute_configuration(
            &control, &request, consumer, &vault, no_entry,
        );
        assert_eq!(rejected["error"]["code"], "IPC_ACCESS_DENIED", "{rejected}");
    }
    for injected in [
        json!({"apiKey":"renderer-key"}),
        json!({"secret":"renderer-secret"}),
        json!({"reference":"unrelated-broker-key"}),
        json!({"endpoint":"https://example.invalid"}),
    ] {
        let mut contaminated = request.clone();
        for (key, value) in injected.as_object().unwrap() {
            contaminated["payload"]["credential"][key] = value.clone();
        }
        let rejected = tradex::quote_source::execute_configuration(
            &control,
            &contaminated,
            "main",
            &vault,
            no_entry,
        );
        assert_eq!(
            rejected["error"]["code"], "IPC_PAYLOAD_INVALID",
            "{rejected}"
        );
    }
    assert_eq!(
        command(
            &mut control.lock().unwrap(),
            "data.source.connection",
            json!({"workspaceId":workspace})
        ),
        initial
    );
    assert!(vault.present.borrow().is_empty());
}

#[cfg(feature = "integration-test")]
#[test]
fn selected_feed_is_verified_through_the_actual_read_only_http_boundary_and_reopen_clears_it() {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
    };
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let initial = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let vault = provider_fixtures::Vault::default();
    let configured = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"choose-probe-feed","schemaVersion":1,"command":"data.source.configure","payload":{
            "workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}
        }}),
        "main",
        &vault,
        provider_fixtures::credentials,
    );
    assert_eq!(configured["ok"], true, "{configured}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let http = tradex::provider_io::BrokerHttp::for_loopback_test(&format!(
        "http://127.0.0.1:{}",
        listener.local_addr().unwrap().port()
    ))
    .unwrap();
    let for_provider = control.clone();
    let served = std::thread::spawn(move || {
        for read in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                line,
                match read {
                    0 | 1 =>
                        "GET /v2/stocks/quotes/latest?symbols=AAPL&feed=sip&currency=USD HTTP/1.1\r\n",
                    2 => "GET /v2/stocks/meta/conditions/quote?tape=C HTTP/1.1\r\n",
                    3 => "GET /v2/stocks/meta/exchanges HTTP/1.1\r\n",
                    _ => unreachable!(),
                }
            );
            let mut headers = std::collections::HashMap::new();
            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                let (key, value) = line.trim().split_once(':').unwrap();
                headers.insert(key.to_lowercase(), value.trim().to_owned());
            }
            assert_eq!(
                headers.get("apca-api-key-id").map(String::as_str),
                Some(provider_fixtures::KEY)
            );
            assert_eq!(
                headers.get("apca-api-secret-key").map(String::as_str),
                Some(provider_fixtures::SECRET)
            );
            assert!(
                for_provider.try_lock().is_ok(),
                "The provider read must not hold the Control Plane lock"
            );
            let body = r#"{"quotes":{"AAPL":{"t":"2026-09-30T14:10:00.123456789Z","bx":"P","bp":250.1234567890123456789,"bs":205,"ax":"Q","ap":250.2234567890123456789,"as":310,"c":["R"],"z":"C"}}}"#;
            let body = match read {
                0 | 1 => body,
                2 => r#"{"R":"Regular"}"#,
                3 => r#"{"P":"NYSE Arca","Q":"Nasdaq"}"#,
                _ => unreachable!(),
            };
            write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
        }
    });
    let catalog = command(
        &mut control.lock().unwrap(),
        "data.source.catalog",
        json!({"workspaceId":workspace}),
    );
    let request = json!({"requestId":"verify-selected-feed","schemaVersion":1,"command":"data.source.probe","payload":{"workspaceId":workspace,"sourceId":"OD-001","expectedStateVersion":catalog["data"]["stateVersion"]}});
    let verified = tradex::quote_source::execute_probe(&control, &request, "main", &vault, &http);
    assert_eq!(verified["ok"], true, "{verified}");
    let source = verified["data"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["sourceId"] == "OD-001")
        .unwrap();
    assert_eq!(source["configured"], true);
    assert_eq!(source["status"], "AVAILABLE");
    assert!(source["coverage"].as_str().unwrap().contains("SIP"));
    assert!(
        source["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("Technical access")
    );
    assert!(source["verifiedAt"].as_str().is_some());
    let connection = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(connection["data"]["status"], "AVAILABLE");
    assert_eq!(connection["data"]["feed"], "sip");
    assert_eq!(
        connection["data"]["stateVersion"],
        configured["data"]["stateVersion"]
    );
    for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
        assert!(!verified.to_string().contains(secret));
    }
    let actual_quote = tradex::quote_source::execute_market(
        &control,
        &json!({"requestId":"http-market-read","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"WARM"}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(actual_quote["ok"], true, "{actual_quote}");
    assert_eq!(
        actual_quote["data"]["snapshot"]["bid"],
        "250.1234567890123456789"
    );
    assert_eq!(
        actual_quote["data"]["snapshot"]["provenance"]["alpaca"]["bidExchangeName"],
        "NYSE Arca"
    );
    assert_eq!(
        actual_quote["data"]["snapshot"]["provenance"]["alpaca"]["regularConditions"],
        true
    );
    assert_eq!(
        actual_quote["data"]["snapshot"]["provenance"]["freshness"],
        "CLOCK_UNCERTAIN"
    );
    for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
        assert!(!actual_quote.to_string().contains(secret));
    }
    served.join().unwrap();
    assert_eq!(
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["ok"],
        true
    );
    let reopened = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(reopened["data"]["configured"], true);
    assert_eq!(reopened["data"]["status"], "UNVERIFIED");
    assert_eq!(reopened["data"]["feed"], "sip");
}

#[cfg(feature = "integration-test")]
#[test]
fn a_late_successful_probe_cannot_replace_a_newer_feed_denial_or_trigger_fallback() {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::{Arc, Mutex, mpsc},
    };
    use tradex::provider_io::{CredentialVault, Credentials};
    struct SharedVault(Mutex<provider_fixtures::Vault>);
    impl CredentialVault for SharedVault {
        fn put(&self, r: &str, c: &Credentials) -> tradex::protocol::Result<()> {
            self.0.lock().unwrap().put(r, c)
        }
        fn get(&self, r: &str) -> tradex::protocol::Result<Credentials> {
            self.0.lock().unwrap().get(r)
        }
        fn remove(&self, r: &str) -> tradex::protocol::Result<()> {
            self.0.lock().unwrap().remove(r)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let initial = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let vault = Arc::new(SharedVault(Mutex::new(provider_fixtures::Vault::default())));
    let saved = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"race-source","schemaVersion":1,"command":"data.source.configure","payload":{
            "workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}
        }}),
        "main",
        vault.as_ref(),
        provider_fixtures::credentials,
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let (first_received, first_ready) = mpsc::channel();
    let (denial_committed, allow_late) = mpsc::channel();
    let served = std::thread::spawn(move || {
        let read = |stream: &std::net::TcpStream| {
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                line,
                "GET /v2/stocks/quotes/latest?symbols=AAPL&feed=sip&currency=USD HTTP/1.1\r\n"
            );
            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
            }
        };
        let (mut first, _) = listener.accept().unwrap();
        read(&first);
        first_received.send(()).unwrap();
        let (mut second, _) = listener.accept().unwrap();
        read(&second);
        let body = format!("Forbidden {}", provider_fixtures::SECRET);
        write!(
            second,
            "HTTP/1.1 403 Forbidden\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
        drop(second);
        allow_late
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let body = r#"{"quotes":{"AAPL":{"t":"2026-09-30T14:10:00.123456789Z","bx":"P","bp":250.1234567890123456789,"bs":205,"ax":"Q","ap":250.2234567890123456789,"as":310,"c":["R"],"z":"C"}}}"#;
        write!(
            first,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
        listener.set_nonblocking(true).unwrap();
        assert!(
            listener.accept().is_err(),
            "Denied SIP must not trigger an IEX fallback or retry"
        );
    });
    let catalog = command(
        &mut control.lock().unwrap(),
        "data.source.catalog",
        json!({"workspaceId":workspace}),
    );
    let request = json!({"requestId":"probe-one","schemaVersion":1,"command":"data.source.probe","payload":{"workspaceId":workspace,"sourceId":"OD-001","expectedStateVersion":catalog["data"]["stateVersion"]}});
    let old_control = control.clone();
    let old_vault = vault.clone();
    let old_request = request.clone();
    let old_base = base.clone();
    let pending = std::thread::spawn(move || {
        tradex::quote_source::execute_probe(
            &old_control,
            &old_request,
            "main",
            old_vault.as_ref(),
            &tradex::provider_io::BrokerHttp::for_loopback_test(&old_base).unwrap(),
        )
    });
    first_ready
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let mut newer_request = request.clone();
    newer_request["requestId"] = json!("probe-two");
    let denied = tradex::quote_source::execute_probe(
        &control,
        &newer_request,
        "main",
        vault.as_ref(),
        &tradex::provider_io::BrokerHttp::for_loopback_test(&base).unwrap(),
    );
    assert_eq!(denied["ok"], true, "{denied}");
    assert!(!denied.to_string().contains(provider_fixtures::SECRET));
    denial_committed.send(()).unwrap();
    let late = pending.join().unwrap();
    served.join().unwrap();
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT", "{late}");
    let connection = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(connection["data"]["status"], "UNAVAILABLE");
    assert_eq!(connection["data"]["feed"], "sip");
    assert!(
        connection["data"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("DATA_SOURCE_FEED_DENIED")
    );
}

#[test]
fn configured_sip_quote_only_evidence_reaches_public_risk_and_review_without_granting_authority() {
    use std::sync::{Arc, Mutex};
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    let directory = tempfile::tempdir().unwrap();
    let control = Arc::new(Mutex::new(ControlPlane::new(
        directory.path().join("workspace"),
    )));
    let workspace =
        command(&mut control.lock().unwrap(), "workspace.open", json!({}))["data"]["workspaceId"]
            .clone();
    let vault = provider_fixtures::Vault::default();
    let http = provider_fixtures::Http::default();
    let connect = json!({"requestId":"quote-risk-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"trading212","environment":"LIVE","label":"Synthetic read-only Live account"}});
    let job = control
        .lock()
        .unwrap()
        .prepare_provider(&connect)
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| provider_fixtures::credentials(),
        &http,
        || control.lock().unwrap().provider_job_current(&job),
    );
    let tested = control.lock().unwrap().complete_provider(&job, outcome);
    assert_eq!(tested["ok"], true, "{tested}");
    let account = command(
        &mut control.lock().unwrap(),
        "provider.connect",
        json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":true}),
    );
    assert_eq!(account["ok"], true, "{account}");
    assert_eq!(account["data"]["health"]["arming"], "DISARMED");
    let policy = command(
        &mut control.lock().unwrap(),
        "risk.get_policy",
        json!({"workspaceId":workspace}),
    );
    let policy = command(
        &mut control.lock().unwrap(),
        "risk.save_policy",
        json!({"workspaceId":workspace,"expectedStateVersion":policy["data"]["stateVersion"],"policy":{"maxOrderNotional":null,"maxOrderQuantity":null,"maxPositionSize":null,"maxSingleInstrumentExposurePercent":null,"maxAssetClassExposurePercent":[],"maxDailyTradedNotional":null,"maxDailyRealizedLoss":null,"maxOpenOrders":null,"maxReservedCapital":null,"allowedInstrumentIds":[],"blockedInstrumentIds":[],"allowedVenues":[],"blockedVenues":[],"allowedAccountIds":[],"blockedAccountIds":[],"allowedEnvironments":[],"staleQuoteThresholdSeconds":3,"marketOrdersEnabled":true,"maxMarketOrderSlippagePercent":"5","maxPriceDeviationPercent":null,"liveInactivityTimeoutMinutes":20}}),
    );
    assert_eq!(policy["ok"], true, "{policy}");
    let source = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let configured = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"quote-risk-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        &vault,
        provider_fixtures::credentials,
    );
    assert_eq!(configured["ok"], true, "{configured}");
    let now = command(
        &mut control.lock().unwrap(),
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let timestamp = (OffsetDateTime::parse(now["data"]["wallClock"].as_str().unwrap(), &Rfc3339)
        .unwrap()
        - time::Duration::seconds(1))
    .format(&Rfc3339)
    .unwrap();
    *http.alpaca_quote_body.borrow_mut()=Some(format!(r#"{{"quotes":{{"AAPL":{{"t":"{timestamp}","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":310,"c":["R"],"z":"C"}}}}}}"#).into_bytes());
    let market = tradex::quote_source::execute_market(
        &control,
        &json!({"requestId":"quote-risk-read","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(market["data"]["status"], "AVAILABLE", "{market}");
    assert!(market["data"]["snapshot"]["lastPrice"].is_null());
    let draft = command(
        &mut control.lock().unwrap(),
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":{"accountId":account["data"]["connectionId"],"venue":"XNAS","environment":"TRADING212_LIVE","instrumentId":"equity:US:AAPL","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"2"},"maximumSpend":"600","timeInForce":"DAY"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &mut control.lock().unwrap(),
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let decision = command(
        &mut control.lock().unwrap(),
        "risk.evaluate_proposal",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert_eq!(decision["ok"], true, "{decision}");
    let check = |decision: &Value, id: &str| {
        decision["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["checkId"] == id)
            .unwrap()["outcome"]
            .clone()
    };
    assert_eq!(
        check(&decision["data"], "QUOTE_FRESHNESS"),
        "PASS",
        "A validated current quote does not require an invented last trade: {decision}"
    );
    assert_eq!(
        check(&decision["data"], "MARKET_ORDER_SLIPPAGE"),
        "PASS",
        "Consolidated SIP is eligible for its verified listing mapping: {decision}"
    );
    assert_eq!(check(&decision["data"], "MARKET_SESSION"), "UNAVAILABLE");
    assert_ne!(check(&decision["data"], "ACCOUNT_ARMING"), "PASS");
    assert_ne!(decision["data"]["status"], "ALLOWED");
    let review=control.lock().unwrap().dispatch_with_events(json!({"requestId":"quote-risk-review","schemaVersion":1,"command":"trade.request_approval","payload":{"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}}),"main",None);
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(
        review["data"]["market"]["snapshot"], market["data"]["snapshot"],
        "Risk and review must use the same producer observation and first receipt"
    );
    assert_eq!(review["data"]["eligible"], false);
    assert_eq!(review["data"]["expectedSpend"], "500.4");
    let blocked=control.lock().unwrap().dispatch_with_events(json!({"requestId":"quote-risk-no-authority","schemaVersion":1,"command":"trade.approve","payload":{"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"],"proposalHash":proposal["data"]["proposalHash"],"reviewedRiskDecisionId":review["data"]["riskDecision"]["decisionId"],"reviewDigest":review["data"]["reviewDigest"],"expectedStateVersion":proposal["data"]["stateVersion"]}}),"main",None);
    assert_eq!(
        blocked["ok"], false,
        "Quotes alone cannot grant financial authority: {blocked}"
    );
    let history = control.lock().unwrap().dispatch_with_events(json!({"requestId":"quote-risk-history","schemaVersion":1,"command":"trade.approval.list","payload":{"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}}),"main",None);
    assert_eq!(history["ok"], true, "{history}");
    assert!(history["data"]["approvals"].as_array().unwrap().is_empty());

    // These are actual provider-response variations; none injects a MarketSnapshot or entitlement.
    for (feed, venue, depth, condition, age, freshness, slippage) in [
        ("iex", "XNAS", 310, "R", 1, "UNAVAILABLE", "UNAVAILABLE"),
        (
            "delayed_sip",
            "XNAS",
            310,
            "R",
            1,
            "UNAVAILABLE",
            "UNAVAILABLE",
        ),
        ("sip", "XNYS", 310, "R", 1, "UNAVAILABLE", "UNAVAILABLE"),
        ("sip", "XNAS", 310, "Y", 1, "UNAVAILABLE", "UNAVAILABLE"),
        ("sip", "XNAS", 1, "R", 1, "PASS", "UNAVAILABLE"),
        ("sip", "XNAS", 310, "R", 5, "UNAVAILABLE", "UNAVAILABLE"),
    ] {
        let source = command(
            &mut control.lock().unwrap(),
            "data.source.connection",
            json!({"workspaceId":workspace}),
        );
        let configured = tradex::quote_source::execute_configuration(
            &control,
            &json!({"requestId":"quote-risk-variant","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":feed,"credential":{"kind":"DEDICATED"}}}),
            "main",
            &vault,
            provider_fixtures::credentials,
        );
        assert_eq!(configured["ok"], true, "{configured}");
        let now = command(
            &mut control.lock().unwrap(),
            "time.status",
            json!({"workspaceId":workspace}),
        );
        let timestamp =
            (OffsetDateTime::parse(now["data"]["wallClock"].as_str().unwrap(), &Rfc3339).unwrap()
                - time::Duration::seconds(age))
            .format(&Rfc3339)
            .unwrap();
        *http.alpaca_quote_body.borrow_mut()=Some(format!(r#"{{"quotes":{{"AAPL":{{"t":"{timestamp}","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":{depth},"c":["{condition}"],"z":"C"}}}}}}"#).into_bytes());
        let observed = tradex::quote_source::execute_market(
            &control,
            &json!({"requestId":"quote-risk-variant-read","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}}),
            "main",
            &vault,
            &http,
        );
        assert_eq!(observed["ok"], true, "{observed}");
        let mut fields = proposal["data"]["fields"].clone();
        fields["venue"] = venue.into();
        let draft = command(
            &mut control.lock().unwrap(),
            "trade.save_draft",
            json!({"workspaceId":workspace,"fields":fields}),
        );
        if venue == "XNYS" {
            assert_eq!(
                draft["error"]["code"], "ORDER_VENUE_INVALID",
                "A foreign listing venue must be rejected before proposal creation: {draft}"
            );
            continue;
        }
        assert_eq!(draft["ok"], true, "{draft}");
        let variant = command(
            &mut control.lock().unwrap(),
            "trade.generate_proposal",
            json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
        );
        assert_eq!(variant["ok"], true, "{variant}");
        let risk = command(
            &mut control.lock().unwrap(),
            "risk.evaluate_proposal",
            json!({"workspaceId":workspace,"proposalId":variant["data"]["proposalId"]}),
        );
        assert_eq!(risk["ok"], true, "{risk}");
        assert_eq!(
            check(&risk["data"], "QUOTE_FRESHNESS"),
            freshness,
            "{feed}/{venue}/{condition}/depth={depth}/age={age}: {risk}"
        );
        assert_eq!(
            check(&risk["data"], "MARKET_ORDER_SLIPPAGE"),
            slippage,
            "{feed}/{venue}/{condition}/depth={depth}/age={age}: {risk}"
        );
        assert_ne!(risk["data"]["status"], "ALLOWED");
        let current=control.lock().unwrap().dispatch_with_events(json!({"requestId":"quote-risk-variant-review","schemaVersion":1,"command":"trade.request_approval","payload":{"workspaceId":workspace,"proposalId":variant["data"]["proposalId"]}}),"main",None);
        assert_eq!(current["ok"], true, "{current}");
        assert_eq!(
            current["data"]["market"]["snapshot"],
            observed["data"]["snapshot"]
        );
        assert_eq!(current["data"]["eligible"], false);
        assert_ne!(
            current["data"]["reviewDigest"],
            review["data"]["reviewDigest"]
        );
        if feed == "iex" {
            let same_proposal=control.lock().unwrap().dispatch_with_events(json!({"requestId":"quote-risk-source-changed-review","schemaVersion":1,"command":"trade.request_approval","payload":{"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}}),"main",None);
            assert_eq!(same_proposal["ok"], true, "{same_proposal}");
            assert_eq!(
                same_proposal["data"]["proposal"]["proposalHash"],
                review["data"]["proposal"]["proposalHash"]
            );
            assert_ne!(
                same_proposal["data"]["reviewDigest"], review["data"]["reviewDigest"],
                "Changing the selected source must change review material even for the same immutable proposal"
            );
            assert_ne!(
                same_proposal["data"]["market"]["snapshot"]["provenance"]["marketSnapshotId"],
                market["data"]["snapshot"]["provenance"]["marketSnapshotId"]
            );
            assert_eq!(same_proposal["data"]["eligible"], false);
        }
    }
    let current_policy = command(
        &mut control.lock().unwrap(),
        "risk.get_policy",
        json!({"workspaceId":workspace}),
    );
    let mut limit_policy = current_policy["data"]["policy"].clone();
    limit_policy["maxPriceDeviationPercent"] = "5".into();
    let saved_policy = command(
        &mut control.lock().unwrap(),
        "risk.save_policy",
        json!({"workspaceId":workspace,"expectedStateVersion":current_policy["data"]["stateVersion"],"policy":limit_policy}),
    );
    assert_eq!(saved_policy["ok"], true, "{saved_policy}");
    let source = command(
        &mut control.lock().unwrap(),
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let configured = tradex::quote_source::execute_configuration(
        &control,
        &json!({"requestId":"limit-quote-risk-source","schemaVersion":1,"command":"data.source.configure","payload":{"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"feed":"sip","credential":{"kind":"DEDICATED"}}}),
        "main",
        &vault,
        provider_fixtures::credentials,
    );
    assert_eq!(configured["ok"], true, "{configured}");
    let now = command(
        &mut control.lock().unwrap(),
        "time.status",
        json!({"workspaceId":workspace}),
    );
    let timestamp = (OffsetDateTime::parse(now["data"]["wallClock"].as_str().unwrap(), &Rfc3339)
        .unwrap()
        - time::Duration::seconds(1))
    .format(&Rfc3339)
    .unwrap();
    *http.alpaca_quote_body.borrow_mut()=Some(format!(r#"{{"quotes":{{"AAPL":{{"t":"{timestamp}","bx":"P","bp":250.1,"bs":205,"ax":"Q","ap":250.2,"as":310,"c":["R"],"z":"C"}}}}}}"#).into_bytes());
    let read = tradex::quote_source::execute_market(
        &control,
        &json!({"requestId":"limit-quote-risk-read","schemaVersion":1,"command":"market.get","payload":{"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let mut fields = proposal["data"]["fields"].clone();
    fields["orderType"] = "LIMIT".into();
    fields["limitPrice"] = "250.2".into();
    fields.as_object_mut().unwrap().remove("maximumSpend");
    let draft = command(
        &mut control.lock().unwrap(),
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":fields}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let limit = command(
        &mut control.lock().unwrap(),
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(limit["ok"], true, "{limit}");
    let risk = command(
        &mut control.lock().unwrap(),
        "risk.evaluate_proposal",
        json!({"workspaceId":workspace,"proposalId":limit["data"]["proposalId"]}),
    );
    assert_eq!(risk["ok"], true, "{risk}");
    assert_eq!(check(&risk["data"], "QUOTE_FRESHNESS"), "PASS");
    assert_eq!(
        check(&risk["data"], "PRICE_DEVIATION"),
        "UNAVAILABLE",
        "Quote-only evidence cannot invent the required last-trade reference: {risk}"
    );
    assert_ne!(risk["data"]["status"], "ALLOWED");
    assert!(http.trading212_posts.borrow().is_empty());
    assert!(http.trading212_delete_calls.borrow().is_empty());
}

#[test]
#[cfg(feature = "integration-test")]
fn legacy_live_fixture_cannot_override_an_explicitly_configured_quote_source() {
    // Separate process keeps the legacy opt-in flag scoped to this external-fixture scenario.
    let output=std::process::Command::new(std::env::current_exe().unwrap())
        .args(["configured_sip_quote_only_evidence_reaches_public_risk_and_review_without_granting_authority","--exact"])
        .env("TRADEX_LIVE_APPROVAL_FIXTURE","1")
        .output().unwrap();
    assert!(
        output.status.success(),
        "Selected source was replaced by legacy fixture evidence:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
