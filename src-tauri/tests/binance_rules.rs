use serde_json::{Value, json};
use tradex::{
    ControlPlane,
    provider_io::{CredentialVault, ProviderHttp},
};
#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod fixtures;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch_with_events(
        json!({"requestId":"spot-rules","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}

// Independent external-provider scenarios each own one application process.
// The parent serializes them; production IP/UID quotas are unchanged inside a
// scenario. This also keeps a ban/fault scenario from poisoning unrelated cases.
fn isolated_rule_scenario(name: &str) -> bool {
    if std::env::var("TRADEX_RULE_SCENARIO").ok().as_deref() == Some(name) {
        return false;
    }
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([name, "--exact", "--nocapture", "--test-threads=1"])
        .env("TRADEX_RULE_SCENARIO", name)
        .output()
        .expect("Run isolated rule scenario");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "Isolated scenario failed: {name}");
    true
}

fn interval_get(rig: &RuleRig, proposal: &Value) -> Value {
    command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}),
    )
}
fn interval_refresh(rig: &RuleRig, proposal: &Value, external: &RuleHttp) -> Value {
    let before = interval_get(rig, proposal);
    tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"interval-collection","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":rig.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        external,
    )
}
fn rules_get(rig: &RuleRig, proposal: &Value) -> Value {
    command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}),
    )
}
fn rules_refresh(rig: &RuleRig, proposal: &Value, external: &RuleHttp) -> Value {
    let before = rules_get(rig, proposal);
    tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"rules-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        external,
    )
}
#[test]
fn interval_eth_proposal_uses_derived_eth_definitions_and_the_same_account_wide_counter_scope() {
    if isolated_rule_scenario(
        "interval_eth_proposal_uses_derived_eth_definitions_and_the_same_account_wide_counter_scope",
    ) {
        return;
    }
    struct RecordingHttp {
        inner: fixtures::Http,
        calls: std::cell::RefCell<Vec<String>>,
    }
    impl ProviderHttp for RecordingHttp {
        fn get(
            &self,
            endpoint: tradex::provider_io::ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            self.calls.borrow_mut().push(path.into());
            self.inner.get(endpoint, path, headers)
        }
    }
    let rig = RuleRig::new();
    let source = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    let configured = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.configure",
        json!({"workspaceId":rig.workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":rig.account["connectionId"],"instrumentId":"crypto:ETH/USDT:spot"}),
    );
    assert_eq!(configured["ok"], true, "{configured}");
    let mut external = fixtures::Http::default();
    external.binance_rules_ui = true;
    external.binance_uid.set(
        rig.account["data"]["remoteAccountId"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let source = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    let external = RecordingHttp {
        inner: external,
        calls: Default::default(),
    };
    let source = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"eth-rule-source","schemaVersion":1,"command":"data.binance_rules.refresh","payload":{"workspaceId":rig.workspace,"expectedStateVersion":source["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(source["data"]["status"], "AVAILABLE", "{source}");
    let proposal=rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:ETH/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"3000","maximumSpend":null,"timeInForce":"GTC"}));
    let before = interval_get(&rig, &proposal);
    let first = external.calls.borrow().len();
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"eth-intervals","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":rig.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    assert_eq!(result["data"]["instrumentId"], "crypto:ETH/USDT:spot");
    assert_eq!(result["data"]["baseAsset"], "ETH");
    assert_eq!(result["data"]["quoteAsset"], "USDT");
    assert_eq!(result["data"]["scope"], "ACCOUNT_ALL_KEYS_IPS_APIS");
    assert_eq!(result["data"]["observation"]["coverageComplete"], true);
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
    let calls = external.calls.borrow();
    let fresh = &calls[first..];
    assert!(
        fresh
            .iter()
            .any(|p| p == "/api/v3/exchangeInfo?symbol=ETHUSDT&showPermissionSets=true")
    );
    assert!(
        fresh
            .iter()
            .any(|p| p.starts_with("/api/v3/rateLimit/order?timestamp=") && !p.contains("symbol="))
    );
    assert!(fresh.iter().all(|p| !p.contains("BTCUSDT")
        && !p.contains("openOrders")
        && !p.contains("openOrderList")));
}

#[test]
fn interval_exact_response_bounds_and_eight_transient_slots_are_visible_through_public_queries() {
    if isolated_rule_scenario(
        "interval_exact_response_bounds_and_eight_transient_slots_are_visible_through_public_queries",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let first_proposal = rig.decision();
    external.edit.set(Some(|route,body| match route {
        "/api/v3/exchangeInfo"=>body["rateLimits"]=(1..=32).map(|n|json!({"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":n,"limit":50})).collect::<Vec<_>>().into(),
        "/api/v3/rateLimit/order"=>*body=(1..=32).map(|n|json!({"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":n,"limit":50,"count":50+n})).collect::<Vec<_>>().into(),
        _=>{}
    }));
    let maximum = interval_refresh(&rig, &first_proposal, &external);
    assert_eq!(maximum["data"]["status"], "OBSERVED", "{maximum}");
    assert_eq!(
        maximum["data"]["observation"]["counters"]
            .as_array()
            .unwrap()
            .len(),
        32
    );
    assert_eq!(maximum["data"]["observation"]["coverageComplete"], true);
    assert_eq!(
        maximum["data"]["observation"]["counters"][31]["count"],
        "82"
    );
    assert_eq!(maximum["data"]["qualification"], "UNAVAILABLE");
    let mut oversized =
        json!([{"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":1,"limit":50,"count":0}]);
    oversized[0]["futurePadding"] = json!("x".repeat(512 * 1024));
    *external.raw.borrow_mut() = Some((
        "/api/v3/rateLimit/order".into(),
        serde_json::to_vec(&oversized).unwrap(),
    ));
    let rejected = interval_refresh(&rig, &first_proposal, &external);
    assert_eq!(rejected["data"]["status"], "UNAVAILABLE");
    assert!(rejected["data"]["observation"].is_null());
    *external.raw.borrow_mut() = None;
    external.edit.set(None);
    assert_eq!(
        interval_refresh(&rig, &first_proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let mut latest = first_proposal.clone();
    for _ in 0..8 {
        latest = rig.decision();
        assert_eq!(
            interval_refresh(&rig, &latest, &external)["data"]["status"],
            "OBSERVED"
        );
    }
    let retired = interval_get(&rig, &first_proposal);
    assert_eq!(
        retired["data"]["status"], "NOT_OBSERVED",
        "The oldest collection must retire when a ninth Proposal owns a slot: {retired}"
    );
    assert!(retired["data"]["observation"].is_null());
    assert_eq!(interval_get(&rig, &latest)["data"]["status"], "OBSERVED");
    assert_ne!(latest["status"], "ALLOWED");
}

#[test]
fn interval_definition_scope_cannot_borrow_a_different_symbol_or_asset_context() {
    if isolated_rule_scenario(
        "interval_definition_scope_cannot_borrow_a_different_symbol_or_asset_context",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/exchangeInfo" {
                body["symbols"][0]["symbol"] = json!("ETHUSDT");
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/exchangeInfo" {
                body["symbols"][0]["baseAsset"] = json!("ETH");
            }
        },
        |route, body| {
            if route == "/api/v3/exchangeInfo" {
                body["symbols"][0]["quoteAsset"] = json!("USD");
            }
        },
        |route, body| {
            if route == "/api/v3/exchangeInfo" {
                let row = body["symbols"][0].clone();
                body["symbols"].as_array_mut().unwrap().push(row);
            }
        },
    ] {
        external.edit.set(Some(edit));
        let first = external.calls.borrow().len();
        let result = interval_refresh(&rig, &proposal, &external);
        assert_eq!(
            result["data"]["status"], "UNAVAILABLE",
            "Filtered definitions must match the immutable canonical scope: {result}"
        );
        assert_eq!(result["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(result["data"]["observation"].is_null());
        assert!(
            external.calls.borrow()[first..]
                .iter()
                .all(|p| p == "/api/v3/time" || p.starts_with("/api/v3/exchangeInfo?")),
            "Mismatched public scope cannot cause private reads"
        );
    }
}

#[test]
fn interval_authentication_wait_retires_current_success_and_preserves_remaining_http_deadline() {
    if isolated_rule_scenario(
        "interval_authentication_wait_retires_current_success_and_preserves_remaining_http_deadline",
    ) {
        return;
    }
    #[derive(Clone)]
    struct WaitingVault {
        inner: fixtures::Vault,
        started: std::sync::mpsc::Sender<()>,
    }
    impl CredentialVault for WaitingVault {
        fn put(
            &self,
            reference: &str,
            credentials: &tradex::provider_io::Credentials,
        ) -> tradex::protocol::Result<()> {
            self.inner.put(reference, credentials)
        }
        fn remove(&self, reference: &str) -> tradex::protocol::Result<()> {
            self.inner.remove(reference)
        }
        fn get(
            &self,
            reference: &str,
        ) -> tradex::protocol::Result<tradex::provider_io::Credentials> {
            self.started.send(()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(22));
            self.inner.get(reference)
        }
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query.clone(),
    );
    let request = |version: Value| json!({"requestId":"capacity-vault-deadline","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}});
    let first = tradex::financial_sources::execute_refresh(
        &rig.control,
        &request(before["data"]["stateVersion"].clone()),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(first["data"]["status"], "OBSERVED");
    let (started, receiving) = std::sync::mpsc::channel();
    let waiting = WaitingVault {
        inner: rig.vault.clone(),
        started,
    };
    let cp = rig.control.clone();
    let uid = external.uid.get();
    let req = request(first["data"]["stateVersion"].clone());
    let worker = std::thread::spawn(move || {
        let http = RuleHttp::default();
        http.uid.set(uid);
        let at = std::time::Instant::now();
        let result = tradex::financial_sources::execute_refresh(&cp, &req, "main", &waiting, &http);
        (result, http.calls.into_inner(), at.elapsed())
    });
    receiving
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let during = command(
        &mut rig
            .control
            .try_lock()
            .expect("Authentication cannot hold the CP lock"),
        "trade.spot_order_intervals.get",
        query,
    );
    assert_eq!(
        during["data"]["status"], "NOT_OBSERVED",
        "Pending authentication retained current success: {during}"
    );
    assert!(during["data"]["observation"].is_null());
    let (failed, calls, elapsed) = worker.join().unwrap();
    assert_eq!(failed["data"]["status"], "UNAVAILABLE", "{failed}");
    assert_eq!(
        failed["data"]["failure"], "PROVIDER_UNAVAILABLE",
        "Insufficient remaining deadline is not a binding change: {failed}"
    );
    assert!(
        calls.len() == 2
            && calls
                .iter()
                .all(|p| p == "/api/v3/time" || p.starts_with("/api/v3/exchangeInfo?")),
        "Only public purpose discovery precedes authentication; no signed HTTP starts without the existing 12-second completion margin"
    );
    assert!(elapsed < std::time::Duration::from_secs(30));
}

#[test]
fn interval_newer_public_refresh_owns_the_observation_and_late_collection_cannot_replace_it() {
    if isolated_rule_scenario(
        "interval_newer_public_refresh_owns_the_observation_and_late_collection_cannot_replace_it",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query.clone(),
    );
    let winner = std::sync::Arc::new(std::sync::Mutex::new(None::<Value>));
    let saved = winner.clone();
    let cp = rig.control.clone();
    let q = query.clone();
    let vault = rig.vault.clone();
    let uid = external.uid.get();
    *external.hook.borrow_mut() = Some((
        "/api/v3/rateLimit/order".into(),
        Box::new(move || {
            let current = command(
                &mut cp
                    .try_lock()
                    .expect("In-flight provider read cannot own the CP lock"),
                "trade.spot_order_intervals.get",
                q.clone(),
            );
            let next = RuleHttp::default();
            next.uid.set(uid);
            let fresh = tradex::financial_sources::execute_refresh(
                &cp,
                &json!({"requestId":"capacity-newer","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":q["workspaceId"],"proposalId":q["proposalId"],"expectedStateVersion":current["data"]["stateVersion"]}}),
                "main",
                &vault,
                &next,
            );
            assert_eq!(fresh["data"]["status"], "OBSERVED", "{fresh}");
            *saved.lock().unwrap() = Some(fresh);
        }),
    ));
    let late = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-older","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT", "{late}");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query,
    );
    let fresh = winner.lock().unwrap().clone().unwrap();
    assert_eq!(current["data"]["observation"], fresh["data"]["observation"]);
    assert_eq!(current["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn interval_authentication_and_original_identity_faults_retire_inputs_with_sanitized_recovery() {
    if isolated_rule_scenario(
        "interval_authentication_and_original_identity_faults_retire_inputs_with_sanitized_recovery",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_order_intervals.get",
            query.clone(),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-auth-faults","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let initial = refresh();
    assert_eq!(initial["data"]["status"], "OBSERVED");
    *external.response.borrow_mut() = Some(("/api/v3/rateLimit/order".into(), 401, 0));
    let denied = refresh();
    assert_eq!(denied["data"]["status"], "UNAVAILABLE");
    assert_eq!(denied["data"]["failure"], "PROVIDER_AUTHENTICATION_FAILED");
    assert!(denied["data"]["observation"].is_null());
    let calls = external.calls.borrow().len();
    let retained = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query.clone(),
    );
    assert_eq!(retained["data"], denied["data"]);
    assert_eq!(external.calls.borrow().len(), calls);
    *external.response.borrow_mut() = None;
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/account" {
                body["uid"] = json!("9007199254740993");
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/account" {
                body["uid"] = json!(9007199254740993.0f64);
            }
        },
        |route, body| {
            if route == "/api/v3/account" {
                body["uid"] = json!(42);
            }
        },
    ] {
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Original identity coercion accepted: {failed}"
        );
        assert_eq!(failed["data"]["failure"], "PROVIDER_IDENTITY_CHANGED");
        assert!(failed["data"]["observation"].is_null());
    }
    external.edit.set(None);
    external.reflect_key.set(true);
    let reflected = refresh();
    assert_eq!(reflected["data"]["status"], "UNAVAILABLE");
    assert!(!reflected.to_string().contains(fixtures::KEY));
    external.reflect_key.set(false);
    let recovered = refresh();
    assert_eq!(recovered["data"]["status"], "OBSERVED", "{recovered}");
    assert_ne!(
        recovered["data"]["observation"]["collectionId"],
        initial["data"]["observation"]["collectionId"]
    );
    assert_eq!(recovered["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn interval_refresh_rejects_renderer_authority_and_late_changed_binding_without_holding_control_lock()
 {
    if isolated_rule_scenario(
        "interval_refresh_rejects_renderer_authority_and_late_changed_binding_without_holding_control_lock",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query.clone(),
    );
    let payload = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]});
    let calls = external.calls.borrow().len();
    for field in [
        "endpoint",
        "symbol",
        "accountId",
        "remoteUid",
        "free",
        "limit",
        "interval",
        "serverTime",
        "count",
        "qualification",
    ] {
        let mut injected = payload.clone();
        injected[field] = json!("renderer-value");
        let result = tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-injection","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":injected}),
            "main",
            &rig.vault,
            &external,
        );
        assert_eq!(result["error"]["code"], "IPC_PAYLOAD_INVALID", "{result}");
    }
    let denied = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-consumer","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":payload}),
        "agent-untrusted",
        &rig.vault,
        &external,
    );
    assert_eq!(denied["error"]["code"], "IPC_ACCESS_DENIED");
    let mut stale = payload.clone();
    stale["expectedStateVersion"] = json!("stale");
    let stale = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-cas","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":stale}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    assert_eq!(external.calls.borrow().len(), calls);
    let control = rig.control.clone();
    let workspace = rig.workspace.clone();
    let account = rig.account["connectionId"].clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/rateLimit/order".into(),
        Box::new(move || {
            let mut cp = control
                .try_lock()
                .expect("Capacity HTTP held the global Control Plane lock");
            let source = command(
                &mut cp,
                "data.binance_rules.connection",
                json!({"workspaceId":workspace}),
            );
            let changed = command(
                &mut cp,
                "data.binance_rules.configure",
                json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account,"instrumentId":"crypto:ETH/USDT:spot"}),
            );
            assert_eq!(changed["ok"], true, "{changed}");
        }),
    ));
    let late = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-late","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":payload}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT", "{late}");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query,
    );
    assert!(current["data"]["observation"].is_null());
    assert_eq!(current["data"]["proposalHash"], decision["proposalHash"]);
}

#[test]
fn interval_public_definition_reflection_cannot_expose_a_saved_key_as_an_unknown_term() {
    if isolated_rule_scenario(
        "interval_public_definition_reflection_cannot_expose_a_saved_key_as_an_unknown_term",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["rateLimits"][0]["rateLimitType"] = json!(fixtures::KEY);
        }
    }));
    let first = external.calls.borrow().len();
    let result = interval_refresh(&rig, &proposal, &external);
    assert_eq!(
        result["data"]["status"], "UNAVAILABLE",
        "Reflected authentication text cannot become a retained unknown interval term: {result}"
    );
    assert_eq!(result["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
    assert!(!result.to_string().contains(fixtures::KEY));
    assert!(
        external.calls.borrow()[first..]
            .iter()
            .all(|p| p == "/api/v3/time" || p.starts_with("/api/v3/exchangeInfo?")),
        "No signed read is needed after invalid public material"
    );
}

#[test]
fn interval_ip_ban_retires_inputs_and_exposes_the_actual_shared_provider_wait() {
    if isolated_rule_scenario(
        "interval_ip_ban_retires_inputs_and_exposes_the_actual_shared_provider_wait",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    *external.response.borrow_mut() = Some(("/api/v3/rateLimit/order".into(), 418, 120));
    let result = interval_refresh(&rig, &proposal, &external);
    assert_eq!(result["data"]["status"], "UNAVAILABLE");
    assert_eq!(result["data"]["failure"], "PROVIDER_IP_BANNED");
    assert!(result["data"]["observation"].is_null());
    assert_eq!(
        result["data"]["providerWaitSeconds"], "120",
        "Original provider request cooldown must remain visible, distinct from interval reset: {result}"
    );
    assert_eq!(interval_get(&rig, &proposal)["data"], result["data"]);
}

#[test]
fn interval_saved_risk_assessment_keeps_bounded_original_inputs_after_reopen_without_refresh() {
    if isolated_rule_scenario(
        "interval_saved_risk_assessment_keeps_bounded_original_inputs_after_reopen_without_refresh",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let workspace = rig.workspace.clone();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    let observed = interval_refresh(&rig, &proposal, &external);
    assert_eq!(observed["data"]["status"], "OBSERVED");
    let captured = command(
        &mut rig.control.lock().unwrap(),
        "risk.evaluate_proposal",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(
        captured["data"]["spotOrderIntervals"], observed["data"],
        "Risk assessment must retain the reviewed interval observation: {captured}"
    );
    assert_ne!(captured["data"]["status"], "ALLOWED");
    assert!(
        captured["data"]["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["kind"] == "SPOT_ORDER_INTERVALS")
    );
    let reads = external.calls.borrow().len();
    let workspace = rig.workspace.clone();
    let RuleRig {
        _folder, control, ..
    } = rig;
    drop(control);
    let mut reopened = ControlPlane::new(_folder.path().into());
    let opened = command(&mut reopened, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let history = command(
        &mut reopened,
        "risk.decision.list",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(
        history["ok"], true,
        "Reopening must deserialize approved interval aggregates: {history}"
    );
    let saved = history["data"]["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["decisionId"] == captured["data"]["decisionId"])
        .unwrap();
    assert_eq!(
        saved["spotOrderIntervals"],
        captured["data"]["spotOrderIntervals"]
    );
    let current = command(
        &mut reopened,
        "trade.spot_order_intervals.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert!(
        current["data"]["observation"].is_null(),
        "Runtime counter observations must not revive: {current}"
    );
    assert_eq!(external.calls.borrow().len(), reads);
    assert!(!saved.to_string().contains("balances"));
    assert!(!saved.to_string().contains("X-MBX"));
    assert!(!saved.to_string().contains("signature="));
    assert!(!saved.to_string().contains("private-capacity-client"));
}

#[test]
fn interval_cached_get_retires_possible_utc_rollover_without_renewing_or_resetting_usage() {
    if isolated_rule_scenario(
        "interval_cached_get_retires_possible_utc_rollover_without_renewing_or_resetting_usage",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    external.edit.set(Some(|route,body|match route {
        "/api/v3/time" => { let now=(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64;body["serverTime"]=json!(now/1000*1000+300); },
        "/api/v3/exchangeInfo"=>body["rateLimits"]=json!([{"rateLimitType":"ORDERS","interval":"SECOND","intervalNum":1,"limit":50}]),
        "/api/v3/rateLimit/order"=>*body=json!([{"rateLimitType":"ORDERS","interval":"SECOND","intervalNum":1,"limit":50,"count":7}]),
        _=>{}
    }));
    let observed = interval_refresh(&rig, &proposal, &external);
    assert_eq!(
        observed["data"]["status"], "OBSERVED",
        "The initial read stays inside the derived uncertain interval: {observed}"
    );
    let cached = interval_get(&rig, &proposal);
    assert_eq!(
        cached["data"]["observation"],
        observed["data"]["observation"]
    );
    let reads = external.calls.borrow().len();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let retired = interval_get(&rig, &proposal);
    assert_eq!(
        retired["data"]["status"], "STALE",
        "A possible rollover must retire before default3s age: {retired}"
    );
    assert_eq!(
        retired["data"]["retirementReason"],
        "POSSIBLE_INTERVAL_BOUNDARY"
    );
    assert!(retired["data"]["observation"].is_null());
    assert_eq!(
        external.calls.borrow().len(),
        reads,
        "Cached get cannot query or renew provider clock/counters"
    );
    assert_eq!(retired["data"]["qualification"], "UNAVAILABLE");
    assert_eq!(observed["data"]["observation"]["counters"][0]["count"], "7");
    assert!(observed["data"]["observation"]["providerObservedAt"].is_null());
    assert!(!observed.to_string().contains("resetAt"));
}

#[test]
fn interval_original_integers_and_duration_math_cannot_be_coerced_or_overflow() {
    if isolated_rule_scenario(
        "interval_original_integers_and_duration_math_cannot_be_coerced_or_overflow",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    let original =
        json!({"rateLimitType":"ORDERS","interval":"DAY","intervalNum":1,"limit":50,"count":0});
    for (field, value) in [
        ("intervalNum", json!(9223372036854775807i64)),
        ("intervalNum", json!(0)),
        ("intervalNum", json!(1.0)),
        ("intervalNum", json!("1")),
        ("limit", json!(-1)),
        ("limit", json!(50.0)),
        ("limit", json!("50")),
        ("count", json!(-1)),
        ("count", json!(0.0)),
        ("count", json!("0")),
        ("count", json!(9223372036854775808u64)),
    ] {
        let mut row = original.clone();
        row[field] = value;
        *external.raw.borrow_mut() = Some((
            "/api/v3/rateLimit/order".into(),
            serde_json::to_vec(&json!([row])).unwrap(),
        ));
        let result = interval_refresh(&rig, &proposal, &external);
        assert_eq!(
            result["data"]["status"], "UNAVAILABLE",
            "Original integer/duration must fail closed at {field}: {result}"
        );
        assert_eq!(result["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(result["data"]["observation"].is_null());
    }
    let mut row = original;
    row["limit"] = json!(0);
    row["count"] = json!(9007199254740993u64);
    *external.raw.borrow_mut() = Some((
        "/api/v3/rateLimit/order".into(),
        serde_json::to_vec(&json!([row])).unwrap(),
    ));
    let result = interval_refresh(&rig, &proposal, &external);
    assert_eq!(result["data"]["status"], "OBSERVED");
    assert_eq!(result["data"]["observation"]["counters"][0]["limit"], "0");
    assert_eq!(
        result["data"]["observation"]["counters"][0]["count"],
        "9007199254740993"
    );
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
    assert!(!result.to_string().contains("remainingSlots"));
}

#[test]
fn interval_coverage_retains_unknown_rows_and_explains_missing_extra_and_contradictory_terms() {
    if isolated_rule_scenario(
        "interval_coverage_retains_unknown_rows_and_explains_missing_extra_and_contradictory_terms",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    let normal = json!([{"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":1,"limit":50,"count":0},{"rateLimitType":"ORDERS","interval":"DAY","intervalNum":1,"limit":9223372036854775807i64,"count":9007199254740993u64}]);
    let mut unknown_field = normal.clone();
    unknown_field[0]["futureAdmissionFlag"] = json!("private-counter-field");
    let mut missing = normal.clone();
    missing.as_array_mut().unwrap().pop();
    let mut extra = normal.clone();
    extra.as_array_mut().unwrap().push(
        json!({"rateLimitType":"ORDERS","interval":"MINUTE","intervalNum":7,"limit":25,"count":4}),
    );
    let mut contradiction = normal.clone();
    contradiction[0]["limit"] = json!(49);
    let mut unknown_unit = normal.clone();
    unknown_unit[0]["interval"] = json!("FORTNIGHT");
    let mut unknown_type = normal.clone();
    unknown_type[0]["rateLimitType"] = json!("FUTURE_ORDERS");
    for (body, reason) in [
        (unknown_field, "UNKNOWN_ACTIVE_INTERVAL_FIELDS_UNRESOLVED"),
        (missing, "DECLARED_INTERVALS_MISSING"),
        (extra, "UNDECLARED_COUNTER_INTERVALS"),
        (contradiction, "INTERVAL_LIMIT_CONTRADICTION"),
        (unknown_unit, "UNKNOWN_INTERVAL_UNIT_UNRESOLVED"),
        (unknown_type, "UNKNOWN_COUNTER_TYPE_UNRESOLVED"),
    ] {
        *external.raw.borrow_mut() = Some((
            "/api/v3/rateLimit/order".into(),
            serde_json::to_vec(&body).unwrap(),
        ));
        let result = interval_refresh(&rig, &proposal, &external);
        assert_eq!(
            result["data"]["status"], "OBSERVED",
            "Bounded original unresolved inputs remain reviewable: {result}"
        );
        let observation = &result["data"]["observation"];
        assert_eq!(
            observation["coverageComplete"], false,
            "Unknown or incomplete intervals cannot claim complete coverage: {result}"
        );
        assert_eq!(
            observation["counters"].as_array().unwrap().len(),
            body.as_array().unwrap().len()
        );
        assert!(
            observation["unresolvedObligations"]
                .as_array()
                .unwrap()
                .contains(&json!(reason)),
            "Missing precise reason {reason}: {result}"
        );
        assert!(!result.to_string().contains("private-counter-field"));
        assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
    }
}

#[test]
fn interval_original_collections_are_unique_bounded_and_never_truncated_to_success() {
    if isolated_rule_scenario(
        "interval_original_collections_are_unique_bounded_and_never_truncated_to_success",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    let row =
        json!({"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":1,"limit":50,"count":0});
    let mut declaration = fixtures::binance_spot_exchange_info("BTCUSDT");
    let mut term = row.clone();
    term.as_object_mut().unwrap().remove("count");
    declaration["rateLimits"] = json!([term.clone(), term]);
    for (route,body) in [
        ("/api/v3/rateLimit/order",serde_json::to_vec(&vec![row.clone();33]).unwrap()),
        ("/api/v3/rateLimit/order",serde_json::to_vec(&vec![row.clone();2]).unwrap()),
        ("/api/v3/exchangeInfo",serde_json::to_vec(&declaration).unwrap()),
        ("/api/v3/rateLimit/order",b"[{\"rateLimitType\":\"ORDERS\",\"interval\":\"HOUR\",\"intervalNum\":1,\"limit\":50,\"count\":0,\"count\":1}]".to_vec()),
    ] {
        *external.raw.borrow_mut()=Some((route.into(),body));
        let result=interval_refresh(&rig,&proposal,&external);
        assert_eq!(result["data"]["status"],"UNAVAILABLE","Malformed interval inventory cannot succeed: {result}");
        assert_eq!(result["data"]["failure"],"PROVIDER_RESPONSE_INVALID","{result}");
        assert!(result["data"]["observation"].is_null());
    }
    *external.raw.borrow_mut() = None;
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
}

#[test]
fn interval_definitions_without_an_order_purpose_cannot_collect_private_usage() {
    if isolated_rule_scenario(
        "interval_definitions_without_an_order_purpose_cannot_collect_private_usage",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    external.edit.set(Some(|route,body| { if route=="/api/v3/exchangeInfo" { body["rateLimits"]=json!([{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":6000}]); } }));
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let first = external.calls.borrow().len();
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"no-interval-purpose","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        result["data"]["status"], "UNAVAILABLE",
        "Missing ORDERS definitions are not unlimited: {result}"
    );
    assert_eq!(
        result["data"]["failure"],
        "INTERVAL_DEFINITIONS_UNAVAILABLE"
    );
    assert!(result["data"]["observation"].is_null());
    let calls = external.calls.borrow();
    let fresh = &calls[first..];
    assert_eq!(
        fresh.len(),
        2,
        "No private purpose means no account/counter read: {fresh:?}"
    );
    assert!(
        fresh
            .iter()
            .all(|p| p == "/api/v3/time" || p.starts_with("/api/v3/exchangeInfo?symbol=BTCUSDT&"))
    );
}

#[test]
fn explicit_interval_read_preserves_original_account_usage_and_declared_units_without_capacity() {
    if isolated_rule_scenario(
        "explicit_interval_read_preserves_original_account_usage_and_declared_units_without_capacity",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query.clone(),
    );
    let first = external.calls.borrow().len();
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"interval-read","schemaVersion":1,"command":"trade.spot_order_intervals.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        result["ok"], true,
        "No authenticated interval observation: {result}"
    );
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    let observed = &result["data"]["observation"];
    assert_eq!(
        observed["declarations"],
        json!([{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":"1","limit":"6000"},{"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":"1","limit":"50"},{"rateLimitType":"ORDERS","interval":"DAY","intervalNum":"1","limit":"9223372036854775807"}])
    );
    assert_eq!(
        observed["counters"],
        json!([{"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":"1","limit":"50","count":"0"},{"rateLimitType":"ORDERS","interval":"DAY","intervalNum":"1","limit":"9223372036854775807","count":"9007199254740993"}])
    );
    assert_eq!(observed["coverageComplete"], true);
    assert_eq!(observed["atomic"], false);
    assert!(observed["providerObservedAt"].is_null());
    assert_eq!(observed["reads"][0]["kind"], "INTERVAL_DEFINITIONS");
    assert_eq!(observed["reads"][1]["kind"], "ACCOUNT_IDENTITY");
    assert_eq!(observed["reads"][2]["kind"], "ACCOUNT_INTERVAL_COUNTERS");
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
    let calls = external.calls.borrow();
    let fresh = &calls[first..];
    assert_eq!(
        fresh.len(),
        4,
        "Only clock, filtered definitions, identity and account-wide counter reads: {fresh:?}"
    );
    assert!(
        fresh
            .iter()
            .any(|p| p == "/api/v3/exchangeInfo?symbol=BTCUSDT&showPermissionSets=true")
    );
    assert!(
        fresh
            .iter()
            .any(|p| p.starts_with("/api/v3/rateLimit/order?timestamp=") && !p.contains("symbol="))
    );
    assert!(fresh.iter().all(|p| !p.contains("openOrders")
        && !p.contains("openOrderList")
        && !p.contains("order/test")
        && !p.contains("myTrades")));
    drop(calls);
    let cached = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        query,
    );
    assert_eq!(cached["data"]["observation"], *observed);
    assert_ne!(rig.decision()["status"], "ALLOWED");
}

#[test]
fn stored_spot_proposal_explains_interval_inputs_without_collecting_or_qualifying() {
    if isolated_rule_scenario(
        "stored_spot_proposal_explains_interval_inputs_without_collecting_or_qualifying",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let reads = external.calls.borrow().len();
    let result = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_order_intervals.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    assert_eq!(
        result["ok"], true,
        "No immutable interval-input explanation: {result}"
    );
    assert_eq!(result["data"]["proposalHash"], decision["proposalHash"]);
    assert_eq!(result["data"]["accountId"], rig.account["connectionId"]);
    assert_eq!(result["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(result["data"]["status"], "NOT_OBSERVED");
    assert_eq!(result["data"]["scope"], "ACCOUNT_ALL_KEYS_IPS_APIS");
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
    assert_eq!(
        result["data"]["qualificationReason"],
        "INTERVAL_INPUTS_NOT_EXECUTION_QUALIFIED"
    );
    assert!(result["data"]["observation"].is_null());
    assert_eq!(
        external.calls.borrow().len(),
        reads,
        "local get cannot collect counters"
    );
    assert_ne!(rig.decision()["status"], "ALLOWED");
}

#[test]
fn stored_spot_proposal_names_required_capacity_inputs_without_reading_or_qualifying() {
    if isolated_rule_scenario(
        "stored_spot_proposal_names_required_capacity_inputs_without_reading_or_qualifying",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let reads = external.calls.borrow().len();
    let result = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    assert_eq!(
        result["ok"], true,
        "The immutable Proposal has no capacity-input explanation: {result}"
    );
    assert_eq!(result["data"]["proposalHash"], decision["proposalHash"]);
    assert_eq!(result["data"]["accountId"], rig.account["connectionId"]);
    assert_eq!(result["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(result["data"]["baseAsset"], "BTC");
    assert_eq!(result["data"]["quoteAsset"], "USDT");
    assert_eq!(result["data"]["status"], "NOT_OBSERVED");
    assert_eq!(result["data"]["purposes"], json!(["OPEN_ORDERS_ACCOUNT"]));
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
    assert_eq!(
        result["data"]["qualificationReason"],
        "DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED"
    );
    assert!(result["data"]["observation"].is_null());
    assert_eq!(
        external.calls.borrow().len(),
        reads,
        "get must not collect private input"
    );
    assert_ne!(decision["status"], "ALLOWED");
}

#[test]
fn explicit_spot_capacity_read_observes_all_account_orders_without_financial_qualification() {
    if isolated_rule_scenario(
        "explicit_spot_capacity_read_observes_all_account_orders_without_financial_qualification",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    let first = external.calls.borrow().len();
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-read","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        result["ok"], true,
        "Authenticated capacity input read is unavailable: {result}"
    );
    assert_eq!(result["data"]["status"], "OBSERVED");
    let observed = &result["data"]["observation"];
    assert_eq!(observed["quality"], "READ_ONLY_SPOT_CAPACITY");
    assert_eq!(observed["atomic"], false);
    assert!(observed["providerObservedAt"].is_null());
    assert_eq!(observed["counts"]["accountOpenOrders"], "2");
    assert_eq!(observed["counts"]["symbolOpenOrders"], "1");
    assert_eq!(observed["reads"][0]["kind"], "ACCOUNT");
    assert_eq!(observed["reads"][1]["kind"], "OPEN_ORDERS_ACCOUNT");
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
    assert!(!result.to_string().contains("private-capacity-client"));
    let calls = external.calls.borrow();
    let fresh = &calls[first..];
    assert!(
        fresh
            .iter()
            .any(|path| path.starts_with("/api/v3/account?timestamp="))
    );
    assert!(
        fresh
            .iter()
            .any(|path| path.starts_with("/api/v3/openOrders?timestamp=")),
        "Must collect the whole account without symbol: {fresh:?}"
    );
    assert!(fresh.iter().all(|p| !p.contains("/rateLimit/order")
        && !p.contains("/openOrderList")
        && !p.contains("/order/test")));
    drop(calls);
    let cached = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query,
    );
    assert_eq!(cached["data"]["observation"], *observed);
    assert_ne!(rig.decision()["status"], "ALLOWED");
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "account.get",
            json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]})
        )["data"]["health"]["arming"],
        "DISARMED"
    );
}

#[test]
fn max_position_inputs_keep_original_base_balances_and_foreign_exposure_unresolved() {
    if isolated_rule_scenario(
        "max_position_inputs_keep_original_base_balances_and_foreign_exposure_unresolved",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body|match route {
        "/api/v3/exchangeInfo"=>body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"MAX_POSITION","maxPosition":"1.00000000"})),
        "/api/v3/account"=> {body["updateTime"]=json!(1700000000000u64);body["balances"]=json!([{"asset":"BTC","free":"0.10000000","locked":"0.02000000"},{"asset":"ETH","free":"3.00000000","locked":"0.00000000"}]);},
        _=>{}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query,
    );
    assert_eq!(
        before["data"]["purposes"],
        json!(["OPEN_ORDERS_ACCOUNT", "BASE_BALANCES"]),
        "Required position inputs are not derived from the actual rule: {before}"
    );
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"position-input","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    let observed = &result["data"]["observation"];
    assert_eq!(
        observed["baseBalance"],
        json!({"asset":"BTC","free":"0.10000000","locked":"0.02000000"})
    );
    assert_eq!(
        observed["position"]["selectedSymbolOpenBuyOriginalQuantity"],
        "0.1"
    );
    assert_eq!(
        observed["position"]["selectedSymbolOpenBuyExecutedQuantity"],
        "0"
    );
    assert_eq!(observed["position"]["assetExposureComplete"], false);
    assert!(
        observed["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("FOREIGN_SYMBOL_ASSET_ATTRIBUTION_UNAVAILABLE"))
    );
    assert_eq!(
        observed["reads"][0]["oldestProviderUpdateTimeMs"],
        "1700000000000"
    );
    assert!(observed["providerObservedAt"].is_null());
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn capacity_counts_distinguish_foreign_algorithmic_and_iceberg_orders() {
    if isolated_rule_scenario("capacity_counts_distinguish_foreign_algorithmic_and_iceberg_orders")
    {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/openOrders" {
            body[1]["type"] = json!("STOP_LOSS_LIMIT");
            body[1]["icebergQty"] = json!("0.01000000");
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query,
    );
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-classifications","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    let counts = &result["data"]["observation"]["counts"];
    assert_eq!(counts["accountOpenOrders"], "2");
    assert_eq!(counts["symbolOpenOrders"], "1");
    assert_eq!(
        counts["accountAlgoOrders"], "1",
        "Missing account-wide algorithmic count: {result}"
    );
    assert_eq!(counts["symbolAlgoOrders"], "0");
    assert_eq!(counts["accountIcebergOrders"], "1");
    assert_eq!(counts["symbolIcebergOrders"], "0");
    assert_eq!(counts["classificationsComplete"], true);
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn malformed_capacity_order_refresh_retires_previously_observed_inventory() {
    if isolated_rule_scenario(
        "malformed_capacity_order_refresh_retires_previously_observed_inventory",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-malformed","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/openOrders" {
                body[0]["side"] = json!("UNKNOWN_SIDE");
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["isWorking"] = json!("true");
            }
        },
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["status"] = json!("FILLED");
            }
        },
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["origQty"] = json!(0.1);
            }
        },
    ] {
        external.edit.set(None);
        assert_eq!(refresh()["data"]["status"], "OBSERVED");
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Malformed inventory was accepted: {failed}"
        );
        assert!(failed["data"]["observation"].is_null());
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert_eq!(failed["data"]["qualification"], "UNAVAILABLE");
    }
}

#[test]
fn capacity_list_inventory_exposes_missing_pending_legs_without_guessing_reserved_counts() {
    if isolated_rule_scenario(
        "capacity_list_inventory_exposes_missing_pending_legs_without_guessing_reserved_counts",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| match route {
        "/api/v3/myFilters" => body["exchangeFilters"].as_array_mut().unwrap().push(json!({"filterType":"EXCHANGE_MAX_NUM_ORDER_LISTS","maxNumOrderLists":20})),
        "/api/v3/openOrders" => body[0]["orderListId"] = json!(9007199254740995u64),
        "/api/v3/openOrderList" => *body = json!([{"orderListId":9007199254740995u64,"contingencyType":"OTO","listStatusType":"EXEC_STARTED","listOrderStatus":"EXECUTING","listClientOrderId":"private-capacity-list","transactionTime":1700000000000u64,"symbol":"BTCUSDT","orders":[{"symbol":"BTCUSDT","orderId":9007199254740993u64,"clientOrderId":"private-capacity-client-0"},{"symbol":"BTCUSDT","orderId":9007199254740996u64,"clientOrderId":"private-pending-leg"}]}]),
        _=>{}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    assert_eq!(
        before["data"]["purposes"],
        json!(["OPEN_ORDERS_ACCOUNT", "OPEN_ORDER_LISTS"]),
        "List source purpose is absent: {before}"
    );
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-lists","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    let observation = &result["data"]["observation"];
    assert_eq!(observation["counts"]["accountOpenOrderLists"], "1");
    assert_eq!(observation["counts"]["symbolOpenOrderLists"], "1");
    assert_eq!(observation["counts"]["missingListLegs"], "1");
    assert_eq!(observation["counts"]["listCoverageComplete"], false);
    assert_eq!(observation["reads"][2]["kind"], "OPEN_ORDER_LISTS");
    assert!(
        observation["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("PENDING_OR_MISSING_LIST_LEGS_UNRESOLVED"))
    );
    assert!(!result.to_string().contains("private-pending-leg"));
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn capacity_ip_ban_preserves_original_retry_after_in_shared_cooldown() {
    if isolated_rule_scenario("capacity_ip_ban_preserves_original_retry_after_in_shared_cooldown") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    *external.response.borrow_mut() = Some(("/api/v3/openOrders".into(), 418, 120));
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-ban","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "UNAVAILABLE");
    assert_eq!(result["data"]["failure"], "PROVIDER_IP_BANNED");
    assert!(result["data"]["observation"].is_null());
    let remaining = tradex::provider_io::provider_retry_after_seconds(
        "binance",
        &external.uid.get().to_string(),
    );
    assert!(
        remaining.is_some_and(|s| s >= 119),
        "Provider 120s ban was shortened: {remaining:?}"
    );
}

#[test]
fn capacity_age_starts_at_first_private_receipt_and_cached_get_cannot_renew_it() {
    if isolated_rule_scenario(
        "capacity_age_starts_at_first_private_receipt_and_cached_get_cannot_renew_it",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    *external.hook.borrow_mut() = Some((
        "/api/v3/openOrders".into(),
        Box::new(|| std::thread::sleep(std::time::Duration::from_millis(3200))),
    ));
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-age","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        result["data"]["status"], "STALE",
        "Finishing the final HTTP read renewed the earlier account receipt: {result}"
    );
    assert!(result["data"]["observation"].is_null());
    let calls = external.calls.borrow().len();
    let cached = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query,
    );
    assert_eq!(cached["data"]["status"], "STALE");
    assert!(cached["data"]["observation"].is_null());
    assert_eq!(external.calls.borrow().len(), calls);
}

#[test]
fn capacity_purpose_reads_only_symbol_scope_and_refuses_unnecessary_private_collection() {
    if isolated_rule_scenario(
        "capacity_purpose_reads_only_symbol_scope_and_refuses_unnecessary_private_collection",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["exchangeFilters"] = json!([]);
            body["symbolFilters"]
                .as_array_mut()
                .unwrap()
                .push(json!({"filterType":"MAX_NUM_ORDERS","maxNumOrders":25}));
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let get = || {
        command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        )
    };
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-scope","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let before = get();
    assert_eq!(
        before["data"]["purposes"],
        json!(["OPEN_ORDERS_SYMBOL"]),
        "Symbol purpose missing: {before}"
    );
    let result = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    assert_eq!(
        result["data"]["observation"]["counts"]["symbolOpenOrders"],
        "1"
    );
    assert!(result["data"]["observation"]["counts"]["accountOpenOrders"].is_null());
    assert!(
        external
            .calls
            .borrow()
            .iter()
            .any(|path| path.starts_with("/api/v3/openOrders?symbol=BTCUSDT&timestamp="))
    );
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["exchangeFilters"] = json!([]);
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let no_purpose = get();
    assert_eq!(no_purpose["data"]["purposes"], json!([]));
    assert!(no_purpose["data"]["observation"].is_null());
    let calls = external.calls.borrow().len();
    let refused = refresh(no_purpose["data"]["stateVersion"].clone());
    assert_eq!(refused["error"]["code"], "CAPACITY_INPUTS_NOT_REQUIRED");
    assert_eq!(external.calls.borrow().len(), calls);
}

#[test]
fn capacity_partial_fill_and_unknown_active_fields_remain_explicitly_unresolved() {
    if isolated_rule_scenario(
        "capacity_partial_fill_and_unknown_active_fields_remain_explicitly_unresolved",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => body["symbols"][0]["filters"]
            .as_array_mut()
            .unwrap()
            .push(json!({"filterType":"MAX_POSITION","maxPosition":"1.00000000"})),
        "/api/v3/account" => {
            body["balances"] = json!([{"asset":"BTC","free":"0.10000000","locked":"0.02000000"}])
        }
        "/api/v3/openOrders" => {
            body.as_array_mut().unwrap().truncate(1);
            body[0]["status"] = json!("PARTIALLY_FILLED");
            body[0]["executedQty"] = json!("0.02000000");
            body[0]["futureCapacityReservation"] = json!("private-future-field");
        }
        _ => {}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-partial","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    let observation = &result["data"]["observation"];
    assert_eq!(
        observation["position"]["selectedSymbolOpenBuyOriginalQuantity"],
        "0.1"
    );
    assert_eq!(
        observation["position"]["selectedSymbolOpenBuyExecutedQuantity"],
        "0.02"
    );
    let unresolved = observation["unresolvedObligations"].as_array().unwrap();
    assert!(
        unresolved.contains(&json!("PARTIAL_FILL_QUANTITY_SEMANTICS_UNRESOLVED")),
        "Partial quantity ambiguity disappeared: {result}"
    );
    assert!(unresolved.contains(&json!("UNKNOWN_ACTIVE_ORDER_FIELDS_UNRESOLVED")));
    assert_eq!(observation["counts"]["orderCoverageComplete"], false);
    assert!(!result.to_string().contains("private-future-field"));
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn malformed_account_and_balance_inventory_is_rejected_even_without_position_purpose() {
    if isolated_rule_scenario(
        "malformed_account_and_balance_inventory_is_rejected_even_without_position_purpose",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-account-fields","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/account" {
                body["canTrade"] = json!("true");
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/account" {
                body["balances"] = json!([{"asset":"BTC","free":"1.0000","locked":"0"},{"asset":"BTC","free":"2.0000","locked":"0"}]);
            }
        },
        |route, body| {
            if route == "/api/v3/account" {
                body["balances"] = json!([{"asset":"BTC","free":1.25,"locked":"0"}]);
            }
        },
        |route, body| {
            if route == "/api/v3/account" {
                body["balances"] =
                    json!(vec![json!({"asset":"BTC","free":"1","locked":"0"}); 1025]);
            }
        },
    ] {
        external.edit.set(None);
        assert_eq!(refresh()["data"]["status"], "OBSERVED");
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Invalid account fields accepted: {failed}"
        );
        assert!(failed["data"]["observation"].is_null());
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
    }
}

#[test]
fn capacity_original_order_quantities_identity_and_future_times_are_not_coerced() {
    if isolated_rule_scenario(
        "capacity_original_order_quantities_identity_and_future_times_are_not_coerced",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/openOrders" {
                body[0]["executedQty"] = json!("0.20000000");
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["price"] = json!(60000);
            }
        },
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["orderListId"] = json!(-1.0);
            }
        },
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["updateTime"] = json!(
                    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64
                        + 60_000
                );
            }
        },
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["clientOrderId"] = json!("x".repeat(37));
            }
        },
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["timeInForce"] = json!(true);
            }
        },
    ] {
        external.edit.set(Some(edit));
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        );
        let result = tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-originals","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        );
        assert_eq!(
            result["data"]["status"], "UNAVAILABLE",
            "Original order field was coerced or ignored: {result}"
        );
        assert_eq!(result["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(result["data"]["observation"].is_null());
    }
}

#[test]
fn capacity_capture_retains_aggregate_observation_without_raw_inventory_or_current_renewal() {
    if isolated_rule_scenario(
        "capacity_capture_retains_aggregate_observation_without_raw_inventory_or_current_renewal",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-capture","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["data"]["status"], "OBSERVED");
    let capture = command(
        &mut rig.control.lock().unwrap(),
        "risk.evaluate_proposal",
        query.clone(),
    );
    assert_eq!(capture["ok"], true);
    assert_eq!(
        capture["data"]["spotCapacity"]["observation"], read["data"]["observation"],
        "Risk review omitted the observed capacity inputs: {capture}"
    );
    assert_eq!(
        capture["data"]["spotCapacity"]["qualification"],
        "UNAVAILABLE"
    );
    assert_ne!(capture["data"]["status"], "ALLOWED");
    assert!(
        capture["data"]["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|input| input["kind"] == "SPOT_CAPACITY")
    );
    assert!(!capture.to_string().contains("private-capacity-client"));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    assert!(current["data"]["observation"].is_null());
    let calls = external.calls.borrow().len();
    let RuleRig {
        _folder, control, ..
    } = rig;
    drop(control);
    let mut reopened = ControlPlane::new(_folder.path().into());
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let history = command(&mut reopened, "risk.decision.list", query.clone());
    let saved = history["data"]["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["decisionId"] == capture["data"]["decisionId"])
        .unwrap();
    assert_eq!(saved["spotCapacity"], capture["data"]["spotCapacity"]);
    assert!(!history.to_string().contains("private-capacity-client"));
    let reopened_inputs = command(&mut reopened, "trade.spot_capacity.get", query);
    assert!(reopened_inputs["data"]["observation"].is_null());
    assert_eq!(external.calls.borrow().len(), calls);
}

#[test]
fn capacity_refresh_rejects_renderer_authority_and_late_changed_binding_without_holding_control_lock()
 {
    if isolated_rule_scenario(
        "capacity_refresh_rejects_renderer_authority_and_late_changed_binding_without_holding_control_lock",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    let payload = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]});
    let calls = external.calls.borrow().len();
    for field in [
        "endpoint",
        "symbol",
        "accountId",
        "remoteUid",
        "free",
        "count",
        "qualification",
    ] {
        let mut injected = payload.clone();
        injected[field] = json!("renderer-value");
        let result = tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-injection","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":injected}),
            "main",
            &rig.vault,
            &external,
        );
        assert_eq!(result["error"]["code"], "IPC_PAYLOAD_INVALID", "{result}");
    }
    let denied = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-consumer","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":payload}),
        "agent-untrusted",
        &rig.vault,
        &external,
    );
    assert_eq!(denied["error"]["code"], "IPC_ACCESS_DENIED");
    let mut stale = payload.clone();
    stale["expectedStateVersion"] = json!("stale");
    let stale = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-cas","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":stale}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    assert_eq!(external.calls.borrow().len(), calls);
    let control = rig.control.clone();
    let workspace = rig.workspace.clone();
    let account = rig.account["connectionId"].clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/openOrders".into(),
        Box::new(move || {
            let mut cp = control
                .try_lock()
                .expect("Capacity HTTP held the global Control Plane lock");
            let source = command(
                &mut cp,
                "data.binance_rules.connection",
                json!({"workspaceId":workspace}),
            );
            let changed = command(
                &mut cp,
                "data.binance_rules.configure",
                json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account,"instrumentId":"crypto:ETH/USDT:spot"}),
            );
            assert_eq!(changed["ok"], true, "{changed}");
        }),
    ));
    let late = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-late","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":payload}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT", "{late}");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query,
    );
    assert!(current["data"]["observation"].is_null());
    assert_eq!(current["data"]["proposalHash"], decision["proposalHash"]);
}

#[test]
fn capacity_list_original_transaction_times_and_unknown_fields_do_not_become_complete_snapshots() {
    if isolated_rule_scenario(
        "capacity_list_original_transaction_times_and_unknown_fields_do_not_become_complete_snapshots",
    ) {
        return;
    }
    fn complete_lists(route: &str, body: &mut Value) {
        match route {
            "/api/v3/myFilters" => body["exchangeFilters"]
                .as_array_mut()
                .unwrap()
                .push(json!({"filterType":"EXCHANGE_MAX_NUM_ORDER_LISTS","maxNumOrderLists":20})),
            "/api/v3/openOrders" => {
                for row in body.as_array_mut().unwrap() {
                    row["symbol"] = json!("BTCUSDT");
                    row["orderListId"] = json!(9007199254740995u64);
                }
            }
            "/api/v3/openOrderList" => {
                *body = json!([{"orderListId":9007199254740995u64,"contingencyType":"OCO","listStatusType":"EXEC_STARTED","listOrderStatus":"EXECUTING","listClientOrderId":"private-capacity-list","transactionTime":1700000000000u64,"symbol":"BTCUSDT","orders":[{"symbol":"BTCUSDT","orderId":9007199254740993u64,"clientOrderId":"private-capacity-client-0"},{"symbol":"BTCUSDT","orderId":9007199254740994u64,"clientOrderId":"private-capacity-client-1"}]}])
            }
            _ => {}
        }
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(complete_lists));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-list-times","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let observed = refresh();
    assert_eq!(observed["data"]["status"], "OBSERVED", "{observed}");
    assert_eq!(
        observed["data"]["observation"]["counts"]["listCoverageComplete"],
        true
    );
    assert_eq!(
        observed["data"]["observation"]["reads"][2]["oldestProviderTransactionTimeMs"],
        "1700000000000",
        "Original list transaction time was omitted: {observed}"
    );
    assert!(observed["data"]["observation"]["providerObservedAt"].is_null());
    external.edit.set(Some(|route, body| {
        complete_lists(route, body);
        if route == "/api/v3/openOrderList" {
            body[0]["futurePendingReservation"] = json!(true);
        }
    }));
    let unknown = refresh();
    assert_eq!(unknown["data"]["status"], "OBSERVED");
    assert_eq!(
        unknown["data"]["observation"]["counts"]["listCoverageComplete"],
        false
    );
    assert!(
        unknown["data"]["observation"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("UNKNOWN_ACTIVE_LIST_FIELDS_UNRESOLVED"))
    );
    external.edit.set(Some(|route, body| {
        complete_lists(route, body);
        if route == "/api/v3/openOrderList" {
            body[0]["transactionTime"] = json!(
                (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64
                    + 60_000
            );
        }
    }));
    let future = refresh();
    assert_eq!(future["data"]["status"], "UNAVAILABLE", "{future}");
    assert!(future["data"]["observation"].is_null());
}

#[test]
fn capacity_account_extensions_are_unresolved_and_optional_original_fields_are_validated() {
    if isolated_rule_scenario(
        "capacity_account_extensions_are_unresolved_and_optional_original_fields_are_validated",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-account-extensions","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/account" {
            body["futureSharedBalanceMode"] = json!("private-account-extension");
            body["balances"] = json!([{"asset":"BTC","free":"1","locked":"0","futureReserved":"private-balance-extension"}]);
        }
    }));
    let unknown = refresh();
    assert_eq!(unknown["data"]["status"], "OBSERVED", "{unknown}");
    let reasons = unknown["data"]["observation"]["unresolvedObligations"]
        .as_array()
        .unwrap();
    assert!(
        reasons.contains(&json!("UNKNOWN_ACTIVE_ACCOUNT_FIELDS_UNRESOLVED")),
        "Unknown account field silently disappeared: {unknown}"
    );
    assert!(reasons.contains(&json!("UNKNOWN_ACTIVE_BALANCE_FIELDS_UNRESOLVED")));
    assert!(!unknown.to_string().contains("private-account-extension"));
    assert!(!unknown.to_string().contains("private-balance-extension"));
    assert_eq!(unknown["data"]["qualification"], "UNAVAILABLE");
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/account" {
                body["requireSelfTradePrevention"] = json!("false");
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/account" {
                body["permissions"] = json!(["SPOT", 1]);
            }
        },
        |route, body| {
            if route == "/api/v3/account" {
                body["makerCommission"] = json!("15");
            }
        },
        |route, body| {
            if route == "/api/v3/account" {
                body["commissionRates"] =
                    json!({"maker":0.0015,"taker":"0.0015","buyer":"0","seller":"0"});
            }
        },
    ] {
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Malformed optional original fields accepted: {failed}"
        );
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(failed["data"]["observation"].is_null());
    }
}

#[test]
fn capacity_list_leg_extensions_and_identity_contradictions_cannot_claim_complete_coverage() {
    if isolated_rule_scenario(
        "capacity_list_leg_extensions_and_identity_contradictions_cannot_claim_complete_coverage",
    ) {
        return;
    }
    fn complete(route: &str, body: &mut Value) {
        match route {
            "/api/v3/myFilters" => body["exchangeFilters"]
                .as_array_mut()
                .unwrap()
                .push(json!({"filterType":"EXCHANGE_MAX_NUM_ORDER_LISTS","maxNumOrderLists":20})),
            "/api/v3/openOrders" => {
                for row in body.as_array_mut().unwrap() {
                    row["symbol"] = json!("BTCUSDT");
                    row["orderListId"] = json!(9007199254740995u64);
                }
            }
            "/api/v3/openOrderList" => {
                *body = json!([{"orderListId":9007199254740995u64,"contingencyType":"OCO","listStatusType":"EXEC_STARTED","listOrderStatus":"EXECUTING","listClientOrderId":"private-list","transactionTime":1700000000000u64,"symbol":"BTCUSDT","orders":[{"symbol":"BTCUSDT","orderId":9007199254740993u64,"clientOrderId":"private-capacity-client-0"},{"symbol":"BTCUSDT","orderId":9007199254740994u64,"clientOrderId":"private-capacity-client-1"}]}])
            }
            _ => {}
        }
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(complete));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-leg-identity","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    assert_eq!(
        refresh()["data"]["observation"]["counts"]["listCoverageComplete"],
        true
    );
    external.edit.set(Some(|route, body| {
        complete(route, body);
        if route == "/api/v3/openOrderList" {
            body[0]["orders"][0]["clientOrderId"] = json!("private-different-order");
            body[0]["orders"][1]["futurePendingLeg"] = json!("private-leg-extension");
        }
    }));
    let contradiction = refresh();
    assert_eq!(
        contradiction["data"]["status"], "OBSERVED",
        "{contradiction}"
    );
    assert_eq!(
        contradiction["data"]["observation"]["counts"]["listCoverageComplete"], false,
        "Client identity contradiction hidden: {contradiction}"
    );
    let reasons = contradiction["data"]["observation"]["unresolvedObligations"]
        .as_array()
        .unwrap();
    assert!(reasons.contains(&json!("LIST_LEG_IDENTITY_CONTRADICTION_UNRESOLVED")));
    assert!(reasons.contains(&json!("UNKNOWN_ACTIVE_LIST_LEG_FIELDS_UNRESOLVED")));
    assert!(!contradiction.to_string().contains("private-leg-extension"));
    for edit in [
        (|route: &str, body: &mut Value| {
            complete(route, body);
            if route == "/api/v3/openOrderList" {
                body[0]["listClientOrderId"] = json!(1);
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            complete(route, body);
            if route == "/api/v3/openOrderList" {
                body[0]["orders"][0]["clientOrderId"] = json!("a".repeat(37));
            }
        },
        |route, body| {
            complete(route, body);
            if route == "/api/v3/openOrderList" {
                body[0]["contingencyType"] = json!(42);
            }
        },
        |route, body| {
            complete(route, body);
            if route == "/api/v3/openOrderList" {
                body[0]["transactionTime"] = json!(0);
            }
        },
    ] {
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Malformed list/leg accepted: {failed}"
        );
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(failed["data"]["observation"].is_null());
    }
}

#[test]
fn capacity_order_state_times_preserve_absence_and_reject_reverse_or_future_history() {
    if isolated_rule_scenario(
        "capacity_order_state_times_preserve_absence_and_reject_reverse_or_future_history",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-time-history","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/openOrders" {
            body[0].as_object_mut().unwrap().remove("updateTime");
        }
    }));
    let absent = refresh();
    assert_eq!(absent["data"]["status"], "OBSERVED", "{absent}");
    assert!(
        absent["data"]["observation"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("ORIGINAL_ORDER_TIMES_INCOMPLETE")),
        "Missing row time hidden by another row's timestamp: {absent}"
    );
    assert_eq!(
        absent["data"]["observation"]["reads"][1]["latestProviderUpdateTimeMs"],
        "1700000000000"
    );
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/openOrders" {
                body[0]["time"] = json!(1700000000001u64);
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["workingTime"] = json!(
                    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64
                        + 60_000
                );
            }
        },
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["workingTime"] = json!("1700000000000");
            }
        },
    ] {
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Contradictory original time accepted: {failed}"
        );
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(failed["data"]["observation"].is_null());
    }
}

#[test]
fn capacity_unknown_order_flags_remain_unresolved_and_original_flag_types_are_bounded() {
    if isolated_rule_scenario(
        "capacity_unknown_order_flags_remain_unresolved_and_original_flag_types_are_bounded",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-original-flags","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/openOrders" {
            body[0]["timeInForce"] = json!("FUTURE_TIF");
            body[0]["selfTradePreventionMode"] = json!("FUTURE_STP");
            body[0]["isWorking"] = json!(false);
        }
    }));
    let unknown = refresh();
    assert_eq!(unknown["data"]["status"], "OBSERVED", "{unknown}");
    assert_eq!(
        unknown["data"]["observation"]["counts"]["orderCoverageComplete"], false,
        "Unknown active flags silently counted as complete: {unknown}"
    );
    assert!(
        unknown["data"]["observation"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("ACTIVE_ORDER_FLAGS_UNRESOLVED"))
    );
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/openOrders" {
                body[0]["selfTradePreventionMode"] = json!(123);
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/openOrders" {
                body[0]["type"] = json!("a".repeat(33));
            }
        },
    ] {
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Malformed original flag accepted: {failed}"
        );
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
    }
}

#[test]
fn capacity_bounded_inventory_is_complete_only_when_original_rows_fit_without_truncation() {
    if isolated_rule_scenario(
        "capacity_bounded_inventory_is_complete_only_when_original_rows_fit_without_truncation",
    ) {
        return;
    }
    fn inventory(count: usize) -> Vec<Value> {
        (0..count).map(|i| json!({"symbol":"BTCUSDT","orderId":9007199254740993u64+i as u64,"orderListId":-1,"clientOrderId":format!("capacity-{i}"),"price":"1","origQty":"1","executedQty":"0","cummulativeQuoteQty":"0","status":"NEW","timeInForce":"GTC","type":"LIMIT","side":"BUY","stopPrice":"0","icebergQty":"0","isWorking":true,"origQuoteOrderQty":"0"})).collect()
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-inventory-bounds","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let maximum = serde_json::to_vec(&inventory(1000)).unwrap();
    assert!(maximum.len() < 512 * 1024);
    *external.raw.borrow_mut() = Some(("/api/v3/openOrders".into(), maximum));
    let complete = refresh();
    assert_eq!(complete["data"]["status"], "OBSERVED", "{complete}");
    assert_eq!(
        complete["data"]["observation"]["counts"]["accountOpenOrders"],
        "1000"
    );
    let mut duplicate = inventory(2);
    duplicate[1] = duplicate[0].clone();
    for invalid in [
        serde_json::to_vec(&inventory(1001)).unwrap(),
        serde_json::to_vec(&duplicate).unwrap(),
        b"[{\"symbol\":\"BTCUSDT\",\"symbol\":\"ETHUSDT\"}]".to_vec(),
        [b"[]".as_slice(), vec![b' '; 512 * 1024].as_slice()].concat(),
    ] {
        *external.raw.borrow_mut() = Some(("/api/v3/openOrders".into(), invalid));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Incomplete inventory accepted: {failed}"
        );
        assert!(failed["data"]["observation"].is_null());
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
    }
    *external.raw.borrow_mut() = Some(("/api/v3/openOrders".into(), b"[]".to_vec()));
    let empty = refresh();
    assert_eq!(empty["data"]["status"], "OBSERVED");
    assert_eq!(
        empty["data"]["observation"]["counts"]["accountOpenOrders"],
        "0"
    );
    assert_ne!(
        empty["data"]["observation"]["collectionId"],
        complete["data"]["observation"]["collectionId"]
    );
    assert_eq!(empty["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn capacity_list_bounds_and_duplicates_cannot_be_truncated_into_complete_coverage() {
    if isolated_rule_scenario(
        "capacity_list_bounds_and_duplicates_cannot_be_truncated_into_complete_coverage",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["exchangeFilters"]
                .as_array_mut()
                .unwrap()
                .push(json!({"filterType":"EXCHANGE_MAX_NUM_ORDER_LISTS","maxNumOrderLists":20}));
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-list-bounds","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let lists = |count: usize| {
        (0..count).map(|i| json!({"orderListId":10000+i,"contingencyType":"OTO","listStatusType":"EXEC_STARTED","listOrderStatus":"EXECUTING","listClientOrderId":format!("capacity-list-{i}"),"transactionTime":1700000000000u64,"symbol":"BTCUSDT","orders":[{"symbol":"BTCUSDT","orderId":20000+i,"clientOrderId":format!("capacity-leg-{i}")}]})).collect::<Vec<_>>()
    };
    *external.raw.borrow_mut() = Some((
        "/api/v3/openOrderList".into(),
        serde_json::to_vec(&lists(256)).unwrap(),
    ));
    let maximum = refresh();
    assert_eq!(maximum["data"]["status"], "OBSERVED", "{maximum}");
    assert_eq!(
        maximum["data"]["observation"]["counts"]["accountOpenOrderLists"],
        "256"
    );
    assert_eq!(
        maximum["data"]["observation"]["counts"]["missingListLegs"],
        "256"
    );
    assert_eq!(
        maximum["data"]["observation"]["counts"]["listCoverageComplete"],
        false
    );
    let mut duplicate = lists(2);
    duplicate[1] = duplicate[0].clone();
    let mut duplicate_leg = lists(1);
    duplicate_leg[0]["orders"] =
        json!([duplicate_leg[0]["orders"][0], duplicate_leg[0]["orders"][0]]);
    for invalid in [lists(257), duplicate, duplicate_leg] {
        *external.raw.borrow_mut() = Some((
            "/api/v3/openOrderList".into(),
            serde_json::to_vec(&invalid).unwrap(),
        ));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "List bounds or duplicate identity accepted: {failed}"
        );
        assert_eq!(failed["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(failed["data"]["observation"].is_null());
    }
}

#[test]
fn capacity_known_list_forms_do_not_invent_a_missing_leg_shape() {
    if isolated_rule_scenario("capacity_known_list_forms_do_not_invent_a_missing_leg_shape") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| { match route {
        "/api/v3/myFilters" => body["exchangeFilters"].as_array_mut().unwrap().push(json!({"filterType":"EXCHANGE_MAX_NUM_ORDER_LISTS","maxNumOrderLists":20})),
        "/api/v3/openOrders" => { *body = json!([body[0]]); body[0]["orderListId"] = json!(9007199254740995u64); },
        "/api/v3/openOrderList" => *body = json!([{"orderListId":9007199254740995u64,"contingencyType":"OCO","listStatusType":"EXEC_STARTED","listOrderStatus":"EXECUTING","listClientOrderId":"private-list-shape","transactionTime":1700000000000u64,"symbol":"BTCUSDT","orders":[{"symbol":"BTCUSDT","orderId":9007199254740993u64,"clientOrderId":"private-capacity-client-0"}]}]),
        _=>{}
    } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-list-shape","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "OBSERVED", "{result}");
    assert_eq!(
        result["data"]["observation"]["counts"]["listCoverageComplete"], false,
        "One returned leg cannot establish a complete OCO shape: {result}"
    );
    assert!(
        result["data"]["observation"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("LIST_LEG_SHAPE_UNRESOLVED"))
    );
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn capacity_authentication_and_original_identity_faults_retire_inputs_with_sanitized_recovery() {
    if isolated_rule_scenario(
        "capacity_authentication_and_original_identity_faults_retire_inputs_with_sanitized_recovery",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-auth-faults","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let initial = refresh();
    assert_eq!(initial["data"]["status"], "OBSERVED");
    *external.response.borrow_mut() = Some(("/api/v3/openOrders".into(), 401, 0));
    let denied = refresh();
    assert_eq!(denied["data"]["status"], "UNAVAILABLE");
    assert_eq!(denied["data"]["failure"], "PROVIDER_AUTHENTICATION_FAILED");
    assert!(denied["data"]["observation"].is_null());
    let calls = external.calls.borrow().len();
    let retained = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    assert_eq!(retained["data"], denied["data"]);
    assert_eq!(external.calls.borrow().len(), calls);
    *external.response.borrow_mut() = None;
    for edit in [
        (|route: &str, body: &mut Value| {
            if route == "/api/v3/account" {
                body["uid"] = json!("9007199254740993");
            }
        }) as fn(&str, &mut Value),
        |route, body| {
            if route == "/api/v3/account" {
                body["uid"] = json!(9007199254740993.0f64);
            }
        },
        |route, body| {
            if route == "/api/v3/account" {
                body["uid"] = json!(42);
            }
        },
    ] {
        external.edit.set(Some(edit));
        let failed = refresh();
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "Original identity coercion accepted: {failed}"
        );
        assert_eq!(failed["data"]["failure"], "PROVIDER_IDENTITY_CHANGED");
        assert!(failed["data"]["observation"].is_null());
    }
    external.edit.set(None);
    external.reflect_key.set(true);
    let reflected = refresh();
    assert_eq!(reflected["data"]["status"], "UNAVAILABLE");
    assert!(!reflected.to_string().contains(fixtures::KEY));
    external.reflect_key.set(false);
    let recovered = refresh();
    assert_eq!(recovered["data"]["status"], "OBSERVED", "{recovered}");
    assert_ne!(
        recovered["data"]["observation"]["collectionId"],
        initial["data"]["observation"]["collectionId"]
    );
    assert_eq!(recovered["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn capacity_rate_limit_wait_is_shared_with_existing_signed_account_reads() {
    if isolated_rule_scenario(
        "capacity_rate_limit_wait_is_shared_with_existing_signed_account_reads",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = || {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-429","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    assert_eq!(refresh()["data"]["status"], "OBSERVED");
    *external.response.borrow_mut() = Some(("/api/v3/openOrders".into(), 429, 120));
    let refused = refresh();
    assert_eq!(refused["data"]["status"], "UNAVAILABLE");
    assert_eq!(refused["data"]["failure"], "PROVIDER_RATE_LIMITED");
    assert!(refused["data"]["observation"].is_null());
    let uid = external.uid.get().to_string();
    assert_eq!(
        tradex::provider_io::provider_retry_after_seconds("binance", &uid),
        Some(120)
    );
    let calls = external.calls.borrow().len();
    *external.response.borrow_mut() = None;
    let retry = refresh();
    assert_eq!(retry["data"]["status"], "UNAVAILABLE", "{retry}");
    assert_eq!(
        external.calls.borrow().len(),
        calls,
        "Cooldown must prevent an additional request before its deadline"
    );
}

#[test]
fn capacity_newer_public_refresh_owns_the_observation_and_late_collection_cannot_replace_it() {
    if isolated_rule_scenario(
        "capacity_newer_public_refresh_owns_the_observation_and_late_collection_cannot_replace_it",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    let winner = std::sync::Arc::new(std::sync::Mutex::new(None::<Value>));
    let saved = winner.clone();
    let cp = rig.control.clone();
    let q = query.clone();
    let vault = rig.vault.clone();
    let uid = external.uid.get();
    *external.hook.borrow_mut() = Some((
        "/api/v3/openOrders".into(),
        Box::new(move || {
            let current = command(
                &mut cp
                    .try_lock()
                    .expect("In-flight provider read cannot own the CP lock"),
                "trade.spot_capacity.get",
                q.clone(),
            );
            let next = RuleHttp::default();
            next.uid.set(uid);
            let fresh = tradex::financial_sources::execute_refresh(
                &cp,
                &json!({"requestId":"capacity-newer","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":q["workspaceId"],"proposalId":q["proposalId"],"expectedStateVersion":current["data"]["stateVersion"]}}),
                "main",
                &vault,
                &next,
            );
            assert_eq!(fresh["data"]["status"], "OBSERVED", "{fresh}");
            *saved.lock().unwrap() = Some(fresh);
        }),
    ));
    let late = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-older","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT", "{late}");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query,
    );
    let fresh = winner.lock().unwrap().clone().unwrap();
    assert_eq!(current["data"]["observation"], fresh["data"]["observation"]);
    assert_eq!(current["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn capacity_proposal_changes_cannot_reset_shared_exact_uid_read_headroom() {
    if isolated_rule_scenario(
        "capacity_proposal_changes_cannot_reset_shared_exact_uid_read_headroom",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let first = rig.decision();
    let refresh = |proposal: &Value| {
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}),
        );
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-uid-budget","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let mut successes = 0;
    for _ in 0..30 {
        let result = refresh(&first);
        if result["data"]["status"] == "OBSERVED" {
            successes += 1;
        } else {
            assert_eq!(result["data"]["status"], "UNAVAILABLE", "{result}");
            assert_eq!(result["data"]["failure"], "PROVIDER_RATE_LIMITED");
            break;
        }
    }
    assert!(
        (1..15).contains(&successes),
        "The shared 1500-weight UID headroom must stop these signed 100-weight collections: {successes}"
    );
    let second = rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60001","maximumSpend":null,"timeInForce":"GTC"}));
    assert_ne!(second["proposalId"], first["proposalId"]);
    let order_reads = external
        .calls
        .borrow()
        .iter()
        .filter(|path| path.starts_with("/api/v3/openOrders?"))
        .count();
    let blocked = refresh(&second);
    assert_eq!(
        blocked["data"]["status"], "UNAVAILABLE",
        "New Proposal reset shared signed UID capacity: {blocked}"
    );
    assert_eq!(blocked["data"]["failure"], "PROVIDER_RATE_LIMITED");
    assert_eq!(
        external
            .calls
            .borrow()
            .iter()
            .filter(|path| path.starts_with("/api/v3/openOrders?"))
            .count(),
        order_reads
    );
}

#[test]
fn capacity_authentication_wait_retires_current_success_and_preserves_remaining_http_deadline() {
    if isolated_rule_scenario(
        "capacity_authentication_wait_retires_current_success_and_preserves_remaining_http_deadline",
    ) {
        return;
    }
    #[derive(Clone)]
    struct WaitingVault {
        inner: fixtures::Vault,
        started: std::sync::mpsc::Sender<()>,
    }
    impl CredentialVault for WaitingVault {
        fn put(
            &self,
            reference: &str,
            credentials: &tradex::provider_io::Credentials,
        ) -> tradex::protocol::Result<()> {
            self.inner.put(reference, credentials)
        }
        fn remove(&self, reference: &str) -> tradex::protocol::Result<()> {
            self.inner.remove(reference)
        }
        fn get(
            &self,
            reference: &str,
        ) -> tradex::protocol::Result<tradex::provider_io::Credentials> {
            self.started.send(()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(22));
            self.inner.get(reference)
        }
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        query.clone(),
    );
    let request = |version: Value| json!({"requestId":"capacity-vault-deadline","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}});
    let first = tradex::financial_sources::execute_refresh(
        &rig.control,
        &request(before["data"]["stateVersion"].clone()),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(first["data"]["status"], "OBSERVED");
    let (started, receiving) = std::sync::mpsc::channel();
    let waiting = WaitingVault {
        inner: rig.vault.clone(),
        started,
    };
    let cp = rig.control.clone();
    let uid = external.uid.get();
    let req = request(first["data"]["stateVersion"].clone());
    let worker = std::thread::spawn(move || {
        let http = RuleHttp::default();
        http.uid.set(uid);
        let at = std::time::Instant::now();
        let result = tradex::financial_sources::execute_refresh(&cp, &req, "main", &waiting, &http);
        (result, http.calls.into_inner(), at.elapsed())
    });
    receiving
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let during = command(
        &mut rig
            .control
            .try_lock()
            .expect("Authentication cannot hold the CP lock"),
        "trade.spot_capacity.get",
        query,
    );
    assert_eq!(
        during["data"]["status"], "NOT_OBSERVED",
        "Pending authentication retained current success: {during}"
    );
    assert!(during["data"]["observation"].is_null());
    let (failed, calls, elapsed) = worker.join().unwrap();
    assert_eq!(failed["data"]["status"], "UNAVAILABLE", "{failed}");
    assert_eq!(
        failed["data"]["failure"], "PROVIDER_UNAVAILABLE",
        "Insufficient remaining deadline is not a binding change: {failed}"
    );
    assert!(
        calls.is_empty(),
        "No HTTP starts without the existing 12-second completion margin"
    );
    assert!(elapsed < std::time::Duration::from_secs(30));
}

#[test]
fn capacity_public_clock_ip_ban_preserves_retry_after_for_other_account_reads() {
    if isolated_rule_scenario(
        "capacity_public_clock_ip_ban_preserves_retry_after_for_other_account_reads",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    *external.response.borrow_mut() = Some(("/api/v3/time".into(), 418, 120));
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-clock-ban","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(result["data"]["status"], "UNAVAILABLE");
    assert_eq!(result["data"]["failure"], "PROVIDER_IP_BANNED");
    assert!(result["data"]["observation"].is_null());
    assert_eq!(
        tradex::provider_io::provider_retry_after_seconds("binance", "another-signed-uid"),
        Some(120),
        "The public clock ban must retain original IP-wide wait metadata"
    );
}

#[test]
fn capacity_documented_unicode_foreign_inventory_is_counted_without_inventing_base_membership() {
    if isolated_rule_scenario(
        "capacity_documented_unicode_foreign_inventory_is_counted_without_inventing_base_membership",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { match route {
        "/api/v3/account"=>body["balances"]=json!([{"asset":"BTC","free":"0.10000000","locked":"0.02000000"},{"asset":"币安人生","free":"3.00000000","locked":"0.00000000"}]),
        "/api/v3/myFilters"=>body["symbolFilters"].as_array_mut().unwrap().push(json!({"filterType":"MAX_POSITION","maxPosition":"1.00000000"})),
        "/api/v3/openOrders"=>body[1]["symbol"]=json!("币安人生USDT"),
        _=>{}
    }}));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let result = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-unicode-inventory","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        result["data"]["status"], "OBSERVED",
        "Documented non-ASCII foreign inventory was misclassified as malformed: {result}"
    );
    let observation = &result["data"]["observation"];
    assert_eq!(observation["counts"]["accountOpenOrders"], "2");
    assert_eq!(observation["counts"]["symbolOpenOrders"], "1");
    assert_eq!(observation["position"]["assetExposureComplete"], false);
    assert_eq!(
        observation["position"]["selectedSymbolOpenBuyOriginalQuantity"],
        "0.1"
    );
    assert!(!result.to_string().contains("币安人生"));
    assert_eq!(result["data"]["qualification"], "UNAVAILABLE");
}

#[test]
fn capacity_public_clock_original_json_is_bounded_and_duplicate_times_are_not_last_wins() {
    if isolated_rule_scenario(
        "capacity_public_clock_original_json_is_bounded_and_duplicate_times_are_not_last_wins",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    external.calls.borrow_mut().clear();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let now = (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64;
    for raw in [
        format!("{{\"serverTime\":{now},\"serverTime\":{now}}}").into_bytes(),
        [
            format!("{{\"serverTime\":{now}}}").as_bytes(),
            vec![b' '; 512 * 1024].as_slice(),
        ]
        .concat(),
    ] {
        *external.raw.borrow_mut() = Some(("/api/v3/time".into(), raw));
        let before = command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_capacity.get",
            query.clone(),
        );
        let result = tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"capacity-clock-original-json","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
            "main",
            &rig.vault,
            &external,
        );
        assert_eq!(
            result["data"]["status"], "UNAVAILABLE",
            "Ambiguous/oversized original clock response reached private inputs: {result}"
        );
        assert_eq!(result["data"]["failure"], "PROVIDER_RESPONSE_INVALID");
        assert!(result["data"]["observation"].is_null());
    }
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|route| route.starts_with("/api/v3/account?")
                || route.starts_with("/api/v3/openOrders?")),
        "Invalid clock responses must stop before private HTTP"
    );
}

#[test]
fn immutable_spot_proposal_explains_static_rules_without_hiding_dynamic_obligations() {
    if isolated_rule_scenario(
        "immutable_spot_proposal_explains_static_rules_without_hiding_dynamic_obligations",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let result = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    assert_eq!(
        result["ok"], true,
        "Per-Proposal rule explanation is unavailable: {result}"
    );
    assert_eq!(result["data"]["proposalHash"], decision["proposalHash"]);
    assert_eq!(result["data"]["quoteAsset"], "USDT");
    let rows = result["data"]["rules"].as_array().unwrap();
    for kind in ["PRICE_FILTER", "LOT_SIZE", "MIN_NOTIONAL", "MAX_ASSET"] {
        assert_eq!(
            rows.iter().find(|r| r["ruleType"] == kind).unwrap()["outcome"],
            "PASS",
            "{result}"
        );
    }
    assert_eq!(
        rows.iter()
            .find(|r| r["ruleType"] == "PRICE_RANGE")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
    assert_eq!(result["data"]["outcome"], "UNAVAILABLE");
    assert_ne!(decision["status"], "ALLOWED");
}

#[test]
fn current_static_price_violation_rejects_risk_and_preserves_its_proposal_binding() {
    if isolated_rule_scenario(
        "current_static_price_violation_rejects_risk_and_preserves_its_proposal_binding",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"][0]["tickSize"] = json!("7");
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    assert_eq!(
        rule_check(&decision)["outcome"],
        "REJECT",
        "The exact 60000 limit is not a multiple of7: {decision}"
    );
    assert_eq!(
        decision["spotRules"]["proposalHash"],
        decision["proposalHash"]
    );
    assert_eq!(decision["spotRules"]["outcome"], "REJECT");
    assert_eq!(
        rule_check(&decision)["reasonCode"],
        "INSTRUMENT_RULES_REJECTED"
    );
}

#[test]
fn empty_collected_constraints_cannot_qualify_an_ordinary_spot_proposal() {
    if isolated_rule_scenario(
        "empty_collected_constraints_cannot_qualify_an_ordinary_spot_proposal",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => body["symbols"][0]["filters"] = json!([]),
        "/api/v3/executionRules" => body["symbolRules"][0]["rules"] = json!([]),
        "/api/v3/myFilters" => {
            body["exchangeFilters"] = json!([]);
            body["symbolFilters"] = json!([]);
            body["assetFilters"] = json!([]);
        }
        _ => {}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    assert_eq!(
        rule_check(&decision)["outcome"],
        "UNAVAILABLE",
        "Metadata without required rule coverage is not qualification: {decision}"
    );
    assert!(
        decision["spotRules"]["unresolvedObligations"]
            .as_array()
            .is_some_and(|v| v.contains(&json!("REQUIRED_STATIC_RULE_COVERAGE_UNAVAILABLE")))
    );
}

#[test]
fn explicit_required_reference_read_qualifies_only_its_bound_percentage_rule() {
    if isolated_rule_scenario(
        "explicit_required_reference_read_qualifies_only_its_bound_percentage_rule",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"proposal-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        read["ok"], true,
        "Public required reference refresh is unavailable: {read}"
    );
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_eq!(read["data"]["references"][0]["kind"], "PROVIDER_REFERENCE");
    assert_eq!(read["data"]["references"][0]["price"], "60000");
    assert_eq!(
        read["data"]["outcome"], "UNAVAILABLE",
        "Dynamic rules remain required"
    );
}

#[test]
fn explicit_null_reference_uses_only_matching_average_interval() {
    if isolated_rule_scenario("explicit_null_reference_uses_only_matching_average_interval") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));
        }
        if route == "/api/v3/referencePrice" { body["referencePrice"] = Value::Null; }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"reference-null","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"][0]["kind"], "AVERAGE_PRICE",
        "Explicit null permits a genuine average, not an error fallback: {read}"
    );
    assert_eq!(read["data"]["references"][0]["intervalMinutes"], 5);
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_eq!(read["data"]["outcome"], "UNAVAILABLE");
}

#[test]
fn base_market_notional_requests_its_required_reference_and_keeps_base_units() {
    if isolated_rule_scenario(
        "base_market_notional_requests_its_required_reference_and_keeps_base_units",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision=rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":null,"timeInForce":"DAY"}));
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"market-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        read["ok"], true,
        "Market BASE requires notional valuation: {read}"
    );
    let rows = read["data"]["rules"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .find(|r| r["ruleType"] == "MIN_NOTIONAL")
            .unwrap()["outcome"],
        "PASS",
        "0.001 BTC at60000USDT is60USDT"
    );
    assert_eq!(
        rows.iter().find(|r| r["ruleType"] == "LOT_SIZE").unwrap()["outcome"],
        "PASS"
    );
    assert_eq!(
        rows.iter()
            .find(|r| r["ruleType"] == "PRICE_FILTER")
            .unwrap()["applicable"],
        false
    );
    assert_ne!(read["data"]["outcome"], "PASS");
}

#[test]
fn unsupported_provider_order_form_rejects_the_unchanged_proposal() {
    if isolated_rule_scenario("unsupported_provider_order_form_rejects_the_unchanged_proposal") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["orderTypes"] = json!(["MARKET"]);
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    assert_eq!(
        rule_check(&decision)["outcome"],
        "REJECT",
        "A LIMIT Proposal cannot use a MARKET-only symbol: {decision}"
    );
    assert!(
        decision["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["ruleType"] == "ORDER_FORM" && r["outcome"] == "REJECT")
    );
}

#[test]
fn refused_reference_read_retires_previous_price_and_never_calls_average_fallback() {
    if isolated_rule_scenario(
        "refused_reference_read_retires_previous_price_and_never_calls_average_fallback",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"reference-failure","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let good = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(good["data"]["references"][0]["price"], "60000");
    *external.raw.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        br#"{"code":-2043,"msg":"This symbol doesn't have a reference price."}"#.to_vec(),
    ));
    let failed = refresh(good["data"]["stateVersion"].clone());
    assert_eq!(
        failed["ok"], true,
        "A bounded unavailable projection permits UI recovery: {failed}"
    );
    assert_eq!(
        failed["data"]["references"],
        json!([]),
        "Failed refresh must not keep prior reference eligible"
    );
    assert_eq!(
        failed["data"]["referenceFailure"], "PROVIDER_RESPONSE_INVALID",
        "Refusal retains a sanitized specific reason: {failed}"
    );
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice")),
        "Error is not explicit null"
    );
    let risk = command(
        &mut rig.control.lock().unwrap(),
        "risk.evaluate_proposal",
        query,
    );
    assert_eq!(rule_check(&risk["data"])["outcome"], "UNAVAILABLE");
}

#[test]
fn quote_asset_market_limit_rejects_exact_notional_without_changing_base_quantity() {
    if isolated_rule_scenario(
        "quote_asset_market_limit_rejects_exact_notional_without_changing_base_quantity",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["assetFilters"] = json!([{"filterType":"MAX_ASSET","asset":"USDT","limit":"50"}]);
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision=rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":null,"timeInForce":"DAY"}));
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"asset-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let read = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "MAX_ASSET")
            .unwrap()["outcome"],
        "REJECT",
        "60USDT exceeds50USDT; no conversion toUSD: {read}"
    );
    assert_eq!(read["data"]["outcome"], "REJECT");
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "MAX_ASSET")
            .unwrap()["unit"],
        "USDT",
        "Asset constraint must retain its actual unit"
    );
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "LOT_SIZE")
            .unwrap()["unit"],
        "BTC"
    );
    let repeated = refresh(read["data"]["stateVersion"].clone());
    assert_eq!(
        repeated["ok"], true,
        "Required input remains refreshable after a complete read: {repeated}"
    );
}

#[test]
fn reference_deadline_retires_prior_input_without_blocking_control_plane_reads() {
    if isolated_rule_scenario(
        "reference_deadline_retires_prior_input_without_blocking_control_plane_reads",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"reference-deadline","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let good = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(good["data"]["references"][0]["price"], "60000");
    let cp = rig.control.clone();
    let q = query.clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        Box::new(move || {
            let during = command(
                &mut cp
                    .try_lock()
                    .expect("Network I/O must not hold the CP lock"),
                "trade.spot_rules.get",
                q.clone(),
            );
            assert_eq!(during["ok"], true);
            std::thread::sleep(std::time::Duration::from_millis(10020));
        }),
    ));
    let late = refresh(good["data"]["stateVersion"].clone());
    assert_eq!(
        late["ok"], true,
        "A same-binding expired read must retire prior eligibility and expose recovery: {late}"
    );
    assert_eq!(late["data"]["references"], json!([]));
    assert_eq!(late["data"]["referenceFailure"], "PROVIDER_UNAVAILABLE");
    let after = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query,
    );
    assert_eq!(after["data"]["references"], json!([]));
}

#[test]
fn identical_reference_cannot_renew_first_receipt_or_financial_freshness() {
    if isolated_rule_scenario(
        "identical_reference_cannot_renew_first_receipt_or_financial_freshness",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let raw = json!({"symbol":"BTCUSDT","referencePrice":"60000","timestamp":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64});
    *external.raw.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        serde_json::to_vec(&raw).unwrap(),
    ));
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"reference-age","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let first = refresh(before["data"]["stateVersion"].clone());
    let second = refresh(first["data"]["stateVersion"].clone());
    assert_eq!(
        first["data"]["references"], second["data"]["references"],
        "Identical provider material preserves its first receipt"
    );
    std::thread::sleep(std::time::Duration::from_millis(3100));
    let old = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    assert_eq!(
        old["data"]["references"],
        json!([]),
        "Default3-second financial freshness is stricter than30-second rule metadata: {old}"
    );
    assert_eq!(old["data"]["referenceFailure"], "REFERENCE_PRICE_STALE");
    let repeated = refresh(old["data"]["stateVersion"].clone());
    assert_eq!(
        repeated["data"]["references"],
        json!([]),
        "An unchanged cached original time cannot renew reference eligibility"
    );
}

#[test]
fn zero_interval_uses_original_last_trade_time_only_after_explicit_null() {
    if isolated_rule_scenario(
        "zero_interval_uses_original_last_trade_time_only_after_explicit_null",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":0})); }
        if route == "/api/v3/referencePrice" { body["referencePrice"]=Value::Null; }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"zero-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"][0]["kind"], "LAST_TRADE",
        "Zero interval needs original trade time, not a ticker receipt or5-minute average: {read}"
    );
    assert_eq!(read["data"]["references"][0]["intervalMinutes"], 0);
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE")
            .unwrap()["outcome"],
        "PASS"
    );
    assert!(
        external
            .calls
            .borrow()
            .iter()
            .any(|p| p == "/api/v3/trades?symbol=BTCUSDT&limit=1")
    );
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|p| p.starts_with("/api/v3/avgPrice") || p.starts_with("/api/v3/ticker"))
    );
}

#[test]
fn mixed_reference_intervals_qualify_only_their_own_rules() {
    if isolated_rule_scenario("mixed_reference_intervals_qualify_only_their_own_rules") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"].as_array_mut().unwrap().extend([
                json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":0}),
                json!({"filterType":"PERCENT_PRICE_BY_SIDE","bidMultiplierDown":"0.8","bidMultiplierUp":"1.2","askMultiplierDown":"0.8","askMultiplierUp":"1.2","avgPriceMins":5})]);
        }
        if route == "/api/v3/referencePrice" { body["referencePrice"] = Value::Null; }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"mixed-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"].as_array().unwrap().len(),
        2,
        "Different rule intervals cannot be substituted: {read}"
    );
    for kind in ["PERCENT_PRICE", "PERCENT_PRICE_BY_SIDE"] {
        assert_eq!(
            read["data"]["rules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["ruleType"] == kind)
                .unwrap()["outcome"],
            "PASS"
        );
    }
    assert_eq!(read["data"]["outcome"], "UNAVAILABLE");
}

#[test]
fn future_reference_time_is_not_accepted_as_a_new_receipt() {
    if isolated_rule_scenario("future_reference_time_is_not_accepted_as_a_new_receipt") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); }
        if route == "/api/v3/referencePrice" { body["timestamp"]=json!((time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64 +1000); }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"future-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"],
        json!([]),
        "Future provider time must not qualify: {read}"
    );
    assert_eq!(read["data"]["referenceFailure"], "REFERENCE_PRICE_FUTURE");
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice"))
    );
}

#[test]
fn public_reference_ip_ban_retires_price_and_honors_shared_cooldown() {
    if isolated_rule_scenario("public_reference_ip_ban_retires_price_and_honors_shared_cooldown") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"ban-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query,
    );
    let good = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(good["data"]["references"].as_array().unwrap().len(), 1);
    *external.response.borrow_mut() = Some(("/api/v3/referencePrice".into(), 418, 1));
    let banned = refresh(good["data"]["stateVersion"].clone());
    assert_eq!(banned["data"]["references"], json!([]));
    *external.response.borrow_mut() = None;
    let started = std::time::Instant::now();
    let recovered = refresh(banned["data"]["stateVersion"].clone());
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(900),
        "An IP ban must delay the next shared-IP reference read"
    );
    assert_eq!(banned["data"]["referenceFailure"], "PROVIDER_IP_BANNED");
    assert_eq!(recovered["data"]["references"].as_array().unwrap().len(), 1);
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice"))
    );
}

#[test]
fn proposal_explanation_distinguishes_missing_reference_dynamic_and_unknown_rules() {
    if isolated_rule_scenario(
        "proposal_explanation_distinguishes_missing_reference_dynamic_and_unknown_rules",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" {
        body["symbols"][0]["filters"].as_array_mut().unwrap().extend([
            json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}),
            json!({"filterType":"FUTURE_FILTER","threshold":"10"})]);
    } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let rows = decision["spotRules"]["rules"].as_array().unwrap();
    for (kind, reason) in [
        ("PERCENT_PRICE", "REFERENCE_PRICE_MISSING"),
        (
            "PRICE_RANGE",
            "EXECUTION_REFERENCE_PRICE_MISSING",
        ),
        (
            "EXCHANGE_MAX_NUM_ORDERS",
            "CURRENT_ACCOUNT_EXCHANGE_ORDER_COUNTS_REQUIRED",
        ),
        ("FUTURE_FILTER", "UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA"),
    ] {
        let row = rows.iter().find(|r| r["ruleType"] == kind).unwrap();
        assert_eq!(row["outcome"], "UNAVAILABLE");
        assert_eq!(row["reasonCode"], reason, "{row}");
    }
}

#[test]
fn proposal_price_range_explains_selected_side_snapshot_without_placement_authority() {
    if isolated_rule_scenario(
        "proposal_price_range_explains_selected_side_snapshot_without_placement_authority",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = rules_get(&rig, &decision);
    assert_eq!(before["ok"], true, "{before}");
    assert!(
        before["data"]["referencePurposes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|purpose| purpose == "ExecutionRules:Execution:PRICE_RANGE"),
        "A stated limit price must require the genuine execution reference: {before}"
    );
    let preview = &before["data"]["priceRangePreview"];
    assert_eq!(preview["state"], "REFERENCE_MISSING", "{before}");
    assert_eq!(preview["side"], "BUY");
    assert_eq!(preview["unit"], "USDT/BTC");
    let bid = &preview["directions"][0];
    assert_eq!(bid["direction"], "BID");
    assert_eq!(bid["lowerMultiplier"], "0.9999");
    assert_eq!(bid["upperMultiplier"], "1.0001");
    assert_eq!(bid["enforced"], true);
    let ask = &preview["directions"][1];
    assert_eq!(ask["direction"], "ASK");
    assert_eq!(ask["enforced"], true);
    assert!(preview["lowerBound"].is_null() && preview["upperBound"].is_null());
    assert!(preview["reference"].is_null());
    let row = before["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PRICE_RANGE")
        .unwrap()
        .clone();
    assert_eq!(row["outcome"], "UNAVAILABLE", "{row}");
    assert_eq!(row["reasonCode"], "EXECUTION_REFERENCE_PRICE_MISSING", "{row}");
    assert_eq!(before["data"]["outcome"], "UNAVAILABLE");

    let refreshed = rules_refresh(&rig, &decision, &external);
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    let preview = &refreshed["data"]["priceRangePreview"];
    assert_eq!(preview["state"], "SNAPSHOT_AVAILABLE", "{refreshed}");
    assert_eq!(preview["lowerBound"], "59994");
    assert_eq!(preview["upperBound"], "60006");
    let buy_digest = preview["boundsDigest"]
        .as_str()
        .expect("A snapshot bound must be traceable by digest after capture: {refreshed}")
        .to_owned();
    assert!(buy_digest.starts_with("sha256:"), "{refreshed}");
    assert_eq!(preview["reference"]["kind"], "EXECUTION_REFERENCE");
    assert_eq!(preview["reference"]["price"], "60000");
    assert!(preview["reference"]["providerObservedAt"].is_string());
    assert!(preview["reference"]["receivedAt"].is_string());
    let row = refreshed["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PRICE_RANGE")
        .unwrap()
        .clone();
    assert_eq!(row["outcome"], "UNAVAILABLE", "{row}");
    assert_eq!(row["reasonCode"], "PRICE_RANGE_SNAPSHOT_BOUNDS_ONLY", "{row}");
    assert_eq!(
        refreshed["data"]["outcome"], "UNAVAILABLE",
        "A price-range snapshot preview is not placement authority: {refreshed}"
    );
    assert!(
        refreshed["data"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "EXECUTION_PRICE_RANGE_UNQUALIFIED"),
        "{refreshed}"
    );
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "account.get",
            json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]})
        )["data"]["health"]["arming"],
        "DISARMED"
    );

    // The same immutable binding explains SELL through its own ask multipliers.
    let sell = rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"SELL","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}));
    let ask = rules_refresh(&rig, &sell, &external);
    assert_eq!(ask["data"]["priceRangePreview"]["side"], "SELL", "{ask}");
    assert_eq!(ask["data"]["priceRangePreview"]["lowerBound"], "59994", "{ask}");
    assert_eq!(ask["data"]["priceRangePreview"]["upperBound"], "60006", "{ask}");
    assert_ne!(
        ask["data"]["priceRangePreview"]["boundsDigest"], buy_digest,
        "The bound digest must bind the selected side, not only the numbers: {ask}"
    );
    assert_eq!(
        ask["data"]["priceRangePreview"]["directions"][1]["direction"],
        "ASK"
    );
}

#[test]
fn proposal_price_range_distinguishes_empty_partial_null_and_unsupported_configuration() {
    if isolated_rule_scenario(
        "proposal_price_range_distinguishes_empty_partial_null_and_unsupported_configuration",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    // Documented partial configuration: only one BUY multiplier is reported.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/executionRules" {
            body["symbolRules"][0]["rules"][0] =
                json!({"ruleType":"PRICE_RANGE","bidLimitMultUp":"1.0001"});
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let partial = rules_get(&rig, &decision);
    let preview = &partial["data"]["priceRangePreview"];
    assert_eq!(preview["state"], "NOT_ENFORCED_SELECTED_SIDE", "{partial}");
    assert_eq!(preview["directions"][0]["upperMultiplier"], "1.0001");
    assert!(preview["directions"][0]["lowerMultiplier"].is_null());
    assert_eq!(preview["directions"][0]["enforced"], false);
    assert_eq!(preview["directions"][1]["enforced"], false);
    assert!(preview["lowerBound"].is_null() && preview["upperBound"].is_null());
    let row = partial["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PRICE_RANGE")
        .unwrap()
        .clone();
    assert_eq!(row["applicable"], false, "{row}");
    assert_eq!(row["outcome"], "PASS", "{row}");
    assert_eq!(
        row["reasonCode"], "PRICE_RANGE_NOT_ENFORCED_FOR_SELECTED_SIDE",
        "{row}"
    );
    assert_eq!(partial["data"]["referencePurposes"], json!([]));
    let calls = external.calls.borrow().len();
    let unnecessary = rules_refresh(&rig, &decision, &external);
    assert_eq!(unnecessary["error"]["code"], "REFERENCE_PRICE_NOT_REQUIRED");
    assert_eq!(external.calls.borrow().len(), calls);

    // A configured rule with an explicit null reference price is not enforced.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/referencePrice" {
            body["referencePrice"] = json!(null);
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let nulled = rules_refresh(&rig, &decision, &external);
    assert_eq!(nulled["ok"], true, "{nulled}");
    let preview = &nulled["data"]["priceRangePreview"];
    assert_eq!(preview["state"], "REFERENCE_EXPLICIT_NULL", "{nulled}");
    assert!(preview["reference"]["price"].is_null());
    assert!(preview["lowerBound"].is_null() && preview["upperBound"].is_null());
    let row = nulled["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PRICE_RANGE")
        .unwrap()
        .clone();
    assert_eq!(row["applicable"], false, "{row}");
    assert_eq!(row["reasonCode"], "PRICE_RANGE_NOT_ENFORCED_WITHOUT_REFERENCE", "{row}");
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|call| call.starts_with("/api/v3/avgPrice")
                || call.starts_with("/api/v3/trades")),
        "An explicit null execution reference must not fall back to average or last trade"
    );

    // An unknown active execution-rule field keeps the configuration explicitly unsupported.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/executionRules" {
            body["symbolRules"][0]["rules"][0]["futureMultiplier"] = json!("1.0100");
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let unsupported = rules_get(&rig, &decision);
    assert_eq!(
        unsupported["data"]["priceRangePreview"]["state"], "UNSUPPORTED_CONFIGURATION",
        "{unsupported}"
    );
    let row = unsupported["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PRICE_RANGE")
        .unwrap()
        .clone();
    assert_eq!(row["outcome"], "UNAVAILABLE", "{row}");
    assert_eq!(row["reasonCode"], "UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA", "{row}");
    assert!(
        unsupported["data"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str().unwrap().contains("PRICE_RANGE")),
        "{unsupported}"
    );

    // A scoped, complete execution-rule source that reports no PRICE_RANGE rule at all
    // is documented non-enforcement: no rule, no reference read, no invented per-rule row.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/executionRules" {
            body["symbolRules"][0]["rules"] = json!([]);
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let absent = rules_get(&rig, &decision);
    let preview = &absent["data"]["priceRangePreview"];
    assert_eq!(preview["state"], "NO_RULE", "{absent}");
    assert_eq!(preview["explanation"], "PRICE_RANGE_NOT_CONFIGURED", "{absent}");
    assert!(preview["reference"].is_null());
    assert!(preview["lowerBound"].is_null() && preview["boundsDigest"].is_null());
    assert_eq!(absent["data"]["referencePurposes"], json!([]), "{absent}");
    assert!(
        !absent["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["ruleType"] == "PRICE_RANGE"),
        "A symbol without a PRICE_RANGE rule must not manufacture a per-rule row: {absent}"
    );
    let calls = external.calls.borrow().len();
    let unneeded = rules_refresh(&rig, &decision, &external);
    assert_eq!(unneeded["error"]["code"], "REFERENCE_PRICE_NOT_REQUIRED", "{unneeded}");
    assert_eq!(
        external.calls.borrow().len(),
        calls,
        "Verified absence of a PRICE_RANGE rule must not read a reference"
    );
}

#[test]
fn proposal_price_range_preserves_exact_zero_precision_and_retires_on_binding_change() {
    if isolated_rule_scenario(
        "proposal_price_range_preserves_exact_zero_precision_and_retires_on_binding_change",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/executionRules" {
            body["symbolRules"][0]["rules"][0] = json!({
                "ruleType":"PRICE_RANGE",
                "bidLimitMultUp":"1.0001",
                "bidLimitMultDown":"0",
                "askLimitMultUp":"1.0001",
                "askLimitMultDown":"0.00000000000000000000000000000001"
            });
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refreshed = rules_refresh(&rig, &decision, &external);
    let preview = &refreshed["data"]["priceRangePreview"];
    assert_eq!(preview["state"], "SNAPSHOT_AVAILABLE", "{refreshed}");
    assert_eq!(
        preview["directions"][0]["lowerMultiplier"], "0",
        "An exact reported zero stays a real multiplier: {preview}"
    );
    assert_eq!(preview["directions"][1]["lowerMultiplier"], "0.00000000000000000000000000000001");
    assert_eq!(preview["lowerBound"], "0", "{refreshed}");
    assert_eq!(preview["upperBound"], "60006", "{refreshed}");
    // The same intent bound to SELL uses the ask multipliers with exact precision.
    let sell = rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"SELL","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}));
    let exact = rules_refresh(&rig, &sell, &external);
    assert_eq!(exact["data"]["priceRangePreview"]["state"], "SNAPSHOT_AVAILABLE", "{exact}");
    assert_eq!(
        exact["data"]["priceRangePreview"]["lowerBound"], "0.0000000000000000000000000006",
        "Exact decimal multiplication must not round: {exact}"
    );
    assert_eq!(exact["data"]["priceRangePreview"]["upperBound"], "60006");
    // A source generation change retires the snapshot instead of renewing it.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"][0]["tickSize"] = json!("0.01000000");
        }
        if route == "/api/v3/executionRules" {
            body["symbolRules"][0]["rules"][0] = json!({
                "ruleType":"PRICE_RANGE",
                "bidLimitMultUp":"1.02",
                "bidLimitMultDown":"0.98",
                "askLimitMultUp":"1.02",
                "askLimitMultDown":"0.98"
            });
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let retired = rules_get(&rig, &decision);
    assert_eq!(
        retired["data"]["priceRangePreview"]["state"], "REFERENCE_MISSING",
        "A changed source generation must require a new execution reference: {retired}"
    );
    assert!(retired["data"]["priceRangePreview"]["lowerBound"].is_null());
}

#[test]
fn proposal_price_range_retires_a_stale_execution_reference_instead_of_renewing_it() {
    if isolated_rule_scenario(
        "proposal_price_range_retires_a_stale_execution_reference_instead_of_renewing_it",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let refreshed = rules_refresh(&rig, &decision, &external);
    let preview = &refreshed["data"]["priceRangePreview"];
    assert_eq!(preview["state"], "SNAPSHOT_AVAILABLE", "{refreshed}");
    assert_eq!(preview["reference"]["price"], "60000");
    let reference_digest = preview["reference"]["digest"].as_str().unwrap().to_owned();
    let bounds_digest = preview["boundsDigest"].as_str().unwrap().to_owned();

    // The default (and any configured) freshness window is bounded by the documented
    // 30-second ceiling and the default 3-second policy threshold, so an untouched
    // observation expires on the clock alone.
    std::thread::sleep(std::time::Duration::from_secs(4));

    let later = rules_get(&rig, &decision);
    let preview = &later["data"]["priceRangePreview"];
    assert_eq!(
        preview["state"], "REFERENCE_UNAVAILABLE",
        "An expired observation is unavailable, never non-enforcement: {later}"
    );
    assert_eq!(preview["explanation"], "REFERENCE_PRICE_STALE", "{later}");
    assert!(
        preview["reference"]["price"].is_null(),
        "A plain get must not renew an expired reference price: {later}"
    );
    assert_eq!(
        preview["reference"]["digest"], reference_digest,
        "The retired observation keeps its provenance: {later}"
    );
    assert!(
        preview["lowerBound"].is_null() && preview["upperBound"].is_null(),
        "{later}"
    );
    assert!(
        preview["boundsDigest"].is_null(),
        "A retired snapshot has no bounds digest to claim: {later}"
    );
    assert_ne!(bounds_digest, "", "{refreshed}");
    let row = later["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PRICE_RANGE")
        .unwrap()
        .clone();
    assert_eq!(row["outcome"], "UNAVAILABLE", "{row}");
    assert_eq!(row["reasonCode"], "REFERENCE_PRICE_STALE", "{row}");
    assert_eq!(later["data"]["outcome"], "UNAVAILABLE", "{later}");
}

#[test]
fn fully_observed_applicable_static_rules_do_not_invent_an_absent_notional_filter() {
    if isolated_rule_scenario(
        "fully_observed_applicable_static_rules_do_not_invent_an_absent_notional_filter",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| match route {
        "/api/v3/account" => body["requireSelfTradePrevention"] = json!(false),
        "/api/v3/myFilters" => {
            body["exchangeFilters"] = json!([]);
            body["symbolFilters"] = json!([]);
            body["assetFilters"] = json!([]);
        }
        "/api/v3/executionRules" => body["symbolRules"][0]["rules"] = json!([]),
        _ => {}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let selected = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_market.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "data.binance_market.configure",
            json!({"workspaceId":rig.workspace,"expectedStateVersion":selected["data"]["stateVersion"]})
        )["ok"],
        true
    );
    let decision = rig.decision();
    assert_eq!(
        decision["spotRules"]["outcome"], "PASS",
        "Completeness evaluates actually returned applicable constraints: {decision}"
    );
    assert_eq!(
        decision["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["checkId"] == "MARKET_DATA_USE")
            .unwrap()["outcome"],
        "UNAVAILABLE",
        "Static success cannot grant public-market financial-use rights"
    );
    assert_eq!(rule_check(&decision)["outcome"], "PASS");
    assert_ne!(decision["status"], "ALLOWED");
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "account.get",
            json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]})
        )["data"]["health"]["arming"],
        "DISARMED"
    );
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/referencePrice"))
    );
}

#[test]
fn proposal_rule_guards_keep_exact_intent_units_and_reference_faults_closed() {
    if isolated_rule_scenario(
        "proposal_rule_guards_keep_exact_intent_units_and_reference_faults_closed",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE_BY_SIDE","bidMultiplierDown":"0.9","bidMultiplierUp":"1.1","askMultiplierDown":"1.1","askMultiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let mut fields = json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"});
    for (quantity, price, kind, outcome) in [
        ("0.0000011", "60000", "LOT_SIZE", "REJECT"),
        ("0.000001000000000001", "60000", "LOT_SIZE", "REJECT"),
        ("0.001", "60000.001", "PRICE_FILTER", "REJECT"),
        ("0.001", "1000", "MIN_NOTIONAL", "REJECT"),
        ("0.3", "60000", "MAX_ASSET", "REJECT"),
    ] {
        fields["quantity"]["value"] = json!(quantity);
        fields["limitPrice"] = json!(price);
        let decision = rig.decision_with(fields.clone());
        let row = decision["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == kind)
            .unwrap();
        assert_eq!(row["outcome"], outcome, "{row}");
    }
    fields["quantity"]["value"] = json!("0.001");
    fields["limitPrice"] = json!("60000");
    fields["side"] = json!("SELL");
    let decision = rig.decision_with(fields);
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"guards-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let get = || {
        command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_rules.get",
            query.clone(),
        )
    };
    let good = refresh(get()["data"]["stateVersion"].clone());
    assert_eq!(
        good["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE_BY_SIDE")
            .unwrap()["outcome"],
        "REJECT",
        "SELL must use ask multipliers"
    );
    for body in [
        r#"{"symbol":"BTCUSDT","referencePrice":60000,"timestamp":1}"#,
        r#"{"symbol":"BTCUSDT","referencePrice":"60000","timestamp":1.5}"#,
        r#"{"symbol":"BTCUSDT","referencePrice":"60000"}"#,
        r#"{"symbol":"BTCUSDT","referencePrice":null,"referencePrice":"60000","timestamp":1}"#,
        r#"{"symbol":"ETHUSDT","referencePrice":"60000","timestamp":1}"#,
        r#"{"code":-2043,"msg":"NO_REFERENCE_PRICE"}"#,
    ] {
        *external.raw.borrow_mut() =
            Some(("/api/v3/referencePrice".into(), body.as_bytes().to_vec()));
        let failed = refresh(get()["data"]["stateVersion"].clone());
        assert_eq!(failed["ok"], true, "{failed}");
        assert_eq!(failed["data"]["references"], json!([]), "{failed}");
        assert!(failed["data"]["referenceFailure"].is_string());
    }
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice"))
    );
    let mut untrusted = query.clone();
    untrusted["price"] = json!("60000");
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_rules.get",
            untrusted
        )["ok"],
        false
    );
    let forbidden=rig.control.lock().unwrap().dispatch_with_events(json!({"requestId":"foreign-consumer","schemaVersion":1,"command":"trade.spot_rules.get","payload":query}),"order-gateway",None);
    assert_eq!(forbidden["error"]["code"], "IPC_ACCESS_DENIED");
    let hash = command(
        &mut rig.control.lock().unwrap(),
        "trade.proposal.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    assert_eq!(hash["data"]["proposalHash"], decision["proposalHash"]);
}

#[test]
fn late_reference_selection_change_cannot_publish_into_new_rule_binding() {
    if isolated_rule_scenario(
        "late_reference_selection_change_cannot_publish_into_new_rule_binding",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {if route == "/api/v3/exchangeInfo" {body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));}}));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let control = rig.control.clone();
    let workspace = rig.workspace.clone();
    let account = rig.account["connectionId"].clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        Box::new(move || {
            let mut cp = control
                .try_lock()
                .expect("Reference I/O must leave Control Plane available");
            let source = command(
                &mut cp,
                "data.binance_rules.connection",
                json!({"workspaceId":workspace}),
            );
            assert_eq!(
                command(
                    &mut cp,
                    "data.binance_rules.configure",
                    json!({"workspaceId":workspace,"connectionId":account,"instrumentId":"crypto:ETH/USDT:spot","expectedStateVersion":source["data"]["stateVersion"]})
                )["ok"],
                true
            );
        }),
    ));
    let late = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"late-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query,
    );
    assert_eq!(current["data"]["references"], json!([]));
    assert!(current["data"]["materialVersion"].is_null());
}

#[test]
fn reopen_preserves_only_captured_digests_and_never_renews_public_references() {
    if isolated_rule_scenario(
        "reopen_preserves_only_captured_digests_and_never_renews_public_references",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {if route == "/api/v3/exchangeInfo" {body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));}}));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capture-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["data"]["references"].as_array().unwrap().len(), 1);
    let capture = command(
        &mut rig.control.lock().unwrap(),
        "risk.evaluate_proposal",
        query.clone(),
    );
    assert_eq!(capture["ok"], true);
    assert!(capture["data"]["spotRules"]["references"][0]["price"].is_null());
    assert_eq!(
        capture["data"]["spotRules"]["references"][0]["digest"],
        read["data"]["references"][0]["digest"]
    );
    let calls = external.calls.borrow().len();
    let RuleRig {
        _folder, control, ..
    } = rig;
    drop(control);
    let mut reopened = ControlPlane::new(_folder.path().into());
    let opened = command(&mut reopened, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let history = command(&mut reopened, "risk.decision.list", query.clone());
    assert_eq!(history["ok"], true, "{history}");
    assert_eq!(
        history["data"]["decisions"]
            .as_array()
            .unwrap()
            .last()
            .unwrap(),
        &capture["data"]
    );
    let current = command(&mut reopened, "trade.spot_rules.get", query);
    assert_eq!(current["data"]["references"], json!([]));
    assert_eq!(current["data"]["outcome"], "UNAVAILABLE");
    assert_eq!(external.calls.borrow().len(), calls);
}

#[test]
fn unmatched_average_is_explained_as_interval_mismatch_without_substitution() {
    if isolated_rule_scenario(
        "unmatched_average_is_explained_as_interval_mismatch_without_substitution",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":1})); } if route == "/api/v3/referencePrice" {body["referencePrice"]=Value::Null;} }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"mismatched-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["data"]["references"][0]["intervalMinutes"], 5);
    let row = read["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PERCENT_PRICE")
        .unwrap();
    assert_eq!(row["outcome"], "UNAVAILABLE");
    assert_eq!(
        row["reasonCode"], "REFERENCE_PRICE_INTERVAL_MISMATCH",
        "{row}"
    );
}

#[test]
fn notional_market_flags_and_quote_quantity_never_invent_base_amounts() {
    if isolated_rule_scenario("notional_market_flags_and_quote_quantity_never_invent_base_amounts")
    {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"NOTIONAL","minNotional":"5","maxNotional":"50","applyMinToMarket":false,"applyMaxToMarket":true,"avgPriceMins":5})); }
        if route == "/api/v3/myFilters" {body["symbolFilters"]=json!([]);}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let limit = rig.decision();
    assert_eq!(
        limit["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "NOTIONAL")
            .unwrap()["outcome"],
        "REJECT",
        "Limit notional60 exceeds50USDT"
    );
    let market = json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":null,"timeInForce":"DAY"});
    let base = rig.decision_with(market.clone());
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":base["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"notional-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":base["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "NOTIONAL")
            .unwrap()["outcome"],
        "REJECT",
        "Base0.001 at original60000 has notional60USDT"
    );
    let mut quote = market;
    quote["quantity"] = json!({"type":"QUOTE","value":"60"});
    let decision = rig.decision_with(quote.clone());
    let rows = decision["spotRules"]["rules"].as_array().unwrap();
    assert_eq!(
        rows.iter().find(|r| r["ruleType"] == "NOTIONAL").unwrap()["outcome"],
        "REJECT"
    );
    for kind in ["LOT_SIZE", "MAX_ASSET"] {
        assert_eq!(
            rows.iter().find(|r| r["ruleType"] == kind).unwrap()["outcome"],
            "UNAVAILABLE",
            "QUOTE60 is not a known BASE amount"
        );
    }
    assert_eq!(decision["spotRules"]["referencePurposes"], json!([]));
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"NOTIONAL","minNotional":"5","maxNotional":"50","applyMinToMarket":false,"applyMaxToMarket":false,"avgPriceMins":5})); }
        if route == "/api/v3/myFilters" {body["symbolFilters"]=json!([]);}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let exempt = rig.decision_with(quote);
    let row = exempt["spotRules"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "NOTIONAL")
        .unwrap();
    assert_eq!(row["applicable"], false);
    assert_eq!(row["outcome"], "PASS");
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":exempt["proposalId"]}),
    );
    let calls = external.calls.borrow().len();
    let unnecessary = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"not-required","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":exempt["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(unnecessary["error"]["code"], "REFERENCE_PRICE_NOT_REQUIRED");
    assert_eq!(external.calls.borrow().len(), calls);
}

#[test]
fn only_documented_price_zero_fields_disable_an_obligation() {
    if isolated_rule_scenario("only_documented_price_zero_fields_disable_an_obligation") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            for name in ["minPrice", "maxPrice", "tickSize"] {
                body["symbols"][0]["filters"][0][name] = json!("0.00000000");
            }
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision=rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000.000001","maximumSpend":null,"timeInForce":"GTC"}));
    assert_eq!(
        decision["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PRICE_FILTER")
            .unwrap()["outcome"],
        "PASS"
    );
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"][1]["stepSize"] = json!("0.00000000");
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let zero_quantity = rig.decision();
    assert_eq!(
        zero_quantity["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "LOT_SIZE")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
}

#[test]
fn public_rule_source_selection_is_versioned_metadata_and_keeps_the_borrowed_key() {
    if isolated_rule_scenario(
        "public_rule_source_selection_is_versioned_metadata_and_keeps_the_borrowed_key",
    ) {
        return;
    }
    let folder = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(folder.path().into());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let request = json!({"requestId":"rules-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"binance","environment":"LIVE","label":"Spot rules source"}});
    let job = control
        .prepare_provider_for(&request, "main")
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || control.provider_job_current(&job),
    );
    let tested = control.complete_provider(&job, outcome);
    assert_eq!(tested["ok"], true, "{tested}");
    let confirmed = command(
        &mut control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":false}),
    );
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    assert_eq!(confirmed["data"]["health"]["arming"], "DISARMED");
    let account: tradex::providers::AccountConnection =
        serde_json::from_value(confirmed["data"].clone()).unwrap();
    let before = http.calls.borrow().len();
    let source = command(
        &mut control,
        "data.binance_rules.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        source["ok"], true,
        "Public Binance rule-source selection is unavailable: {source}"
    );
    assert_eq!(source["data"]["configured"], false);
    assert_eq!(
        source["data"]["eligibleAccounts"][0]["connectionId"],
        account.connection_id
    );
    let payload = json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account.connection_id,"instrumentId":"crypto:BTC/USDT:spot"});
    let saved = command(
        &mut control,
        "data.binance_rules.configure",
        payload.clone(),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(saved["data"]["status"], "UNVERIFIED");
    assert_eq!(saved["data"]["kind"], "BINANCE_SPOT_RULES");
    assert!(saved["data"]["evidence"].is_null());
    assert_eq!(
        http.calls.borrow().len(),
        before,
        "Save must not read the provider"
    );
    let stale = command(&mut control, "data.binance_rules.configure", payload);
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    drop(control);
    let mut control = ControlPlane::new(folder.path().into());
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let reopened = command(
        &mut control,
        "data.binance_rules.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(reopened["data"]["connectionId"], account.connection_id);
    assert_eq!(reopened["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(reopened["data"]["status"], "UNVERIFIED");
    assert!(reopened["data"]["evidence"].is_null());
    let disconnected = command(
        &mut control,
        "data.binance_rules.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":reopened["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["data"]["configured"], false);
    assert!(vault.get(&account.credential_ref()).is_ok());
    let retained = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account.connection_id}),
    );
    assert_eq!(retained["data"]["connectionState"], "CONNECTED");
    assert_eq!(retained["data"]["health"]["arming"], "DISARMED");
}

#[derive(Default)]
struct RuleHttp {
    calls: std::cell::RefCell<Vec<String>>,
    uid: std::cell::Cell<u64>,
    reflect_signature: std::cell::Cell<bool>,
    reflect_key: std::cell::Cell<bool>,
    edit: std::cell::Cell<Option<fn(&str, &mut Value)>>,
    raw: std::cell::RefCell<Option<(String, Vec<u8>)>>,
    response: std::cell::RefCell<Option<(String, u16, u64)>>,
    hook: std::cell::RefCell<Option<(String, Box<dyn Fn()>)>>,
}
impl tradex::provider_io::ProviderHttp for RuleHttp {
    fn get_response_with_rate_limit(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<(
        tradex::provider_io::ProviderHttpResponse,
        Option<tradex::provider_io::ProviderRateLimit>,
    )> {
        let body = self.get(endpoint, path, headers)?;
        let (status, limit) = self
            .response
            .borrow()
            .as_ref()
            .filter(|(route, _, _)| path.split('?').next() == Some(route.as_str()))
            .map_or((200, None), |(_, status, seconds)| {
                (
                    *status,
                    Some(tradex::provider_io::ProviderRateLimit {
                        retry_after_seconds: Some(*seconds),
                        ..Default::default()
                    }),
                )
            });
        Ok((
            tradex::provider_io::ProviderHttpResponse { status, body },
            limit,
        ))
    }
    fn request_with_rate_limit(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        method: tradex::provider_io::ProviderHttpMethod,
        path: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<&Value>,
    ) -> tradex::protocol::Result<(
        tradex::provider_io::ProviderHttpResponse,
        Option<tradex::provider_io::ProviderRateLimit>,
    )> {
        assert_eq!(method, tradex::provider_io::ProviderHttpMethod::Get);
        assert!(body.is_none());
        self.get_response_with_rate_limit(endpoint, path, headers)
    }
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        assert_eq!(endpoint, tradex::provider_io::ProviderEndpoint::BinanceLive);
        let (route, query) = path.split_once('?').unwrap_or((path, ""));
        self.calls.borrow_mut().push(path.into());
        let public = matches!(
            route,
            "/api/v3/time"
                | "/sapi/v1/system/status"
                | "/api/v3/exchangeInfo"
                | "/api/v3/executionRules"
                | "/api/v3/referencePrice"
                | "/api/v3/avgPrice"
                | "/api/v3/trades"
        );
        if public {
            assert!(headers.is_empty());
        } else {
            assert_eq!(headers["X-MBX-APIKEY"], fixtures::KEY);
            assert!(headers["X-MBX-APIKEY"].is_sensitive());
            let (params, signature) = query.split_once("&signature=").unwrap();
            use hmac::Mac;
            let mut mac =
                hmac::Hmac::<sha2::Sha256>::new_from_slice(fixtures::SECRET.as_bytes()).unwrap();
            mac.update(params.as_bytes());
            mac.verify_slice(&hex::decode(signature).unwrap()).unwrap();
            assert!(params.contains("recvWindow=5000"));
        }
        let mut body = match route {
            "/api/v3/trades" => {
                assert_eq!(query, "symbol=BTCUSDT&limit=1");
                json!([{"id":9007199254740993u64,"price":"60000.00000000","qty":"0.001","quoteQty":"60","time":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"isBuyerMaker":false,"isBestMatch":true}])
            }
            "/api/v3/avgPrice" => {
                assert_eq!(query, "symbol=BTCUSDT");
                json!({"mins":5,"price":"60000.00000000","closeTime":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})
            }
            "/api/v3/referencePrice" => {
                assert_eq!(query, "symbol=BTCUSDT");
                json!({"symbol":"BTCUSDT","referencePrice":"60000.00000000","timestamp":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})
            }
            "/api/v3/time" => {
                json!({"serverTime":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})
            }
            "/api/v3/account" => {
                json!({"uid":self.uid.get(),"accountType":"SPOT","canTrade":true,"permissions":["SPOT"]})
            }
            "/api/v3/account/commission" => {
                assert!(query.starts_with("symbol=BTCUSDT&timestamp="));
                json!({"symbol":"BTCUSDT",
                    "standardCommission":{"maker":"0.00000010","taker":"0.00000020","buyer":"0.00000030","seller":"0.00000040"},
                    "taxCommission":{"maker":"0.00000112","taker":"0.00000114","buyer":"0.00000118","seller":"0.00000116"},
                    "specialCommission":{"maker":"0.01000000","taker":"0.02000000","buyer":"0.03000000","seller":"0.04000000"},
                    "discount":{"enabledForAccount":true,"enabledForSymbol":true,"discountAsset":"BNB","discount":"0.75000000"}})
            }
            "/api/v3/rateLimit/order" => {
                assert!(query.starts_with("timestamp=") && !query.contains("symbol="));
                json!([{"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":1,"limit":50,"count":0},{"rateLimitType":"ORDERS","interval":"DAY","intervalNum":1,"limit":9223372036854775807i64,"count":9007199254740993u64}])
            }
            "/api/v3/openOrders" => {
                assert!(
                    query.starts_with("timestamp=")
                        || query.starts_with("symbol=BTCUSDT&timestamp=")
                );
                ["BTCUSDT","ETHUSDT"].into_iter().filter(|symbol| !query.starts_with("symbol=") || *symbol == "BTCUSDT").enumerate().map(|(index,symbol)|json!({"symbol":symbol,"orderId":9007199254740993u64+index as u64,"orderListId":-1,"clientOrderId":format!("private-capacity-client-{index}"),"price":"60000.00000000","origQty":"0.10000000","executedQty":"0.00000000","cummulativeQuoteQty":"0.00000000","status":"NEW","timeInForce":"GTC","type":"LIMIT","side":"BUY","stopPrice":"0.00000000","icebergQty":"0.00000000","time":1700000000000u64,"updateTime":1700000000000u64,"isWorking":true,"workingTime":1700000000000u64,"origQuoteOrderQty":"0.00000000","selfTradePreventionMode":"NONE"})).collect::<Vec<_>>().into()
            }
            "/api/v3/openOrderList" => {
                assert!(query.starts_with("timestamp=") && !query.contains("symbol="));
                json!([])
            }
            "/sapi/v1/account/apiRestrictions" => {
                json!({"enableReading":true,"enableWithdrawals":false,"enableInternalTransfer":false,"permitsUniversalTransfer":false,"enableMargin":false,"enableFutures":false,"enableVanillaOptions":false,"enablePortfolioMarginTrading":false,"enableFixApiTrade":false,"enableFixReadOnly":false,"enableSpotAndMarginTrading":true,"ipRestrict":true})
            }
            "/sapi/v1/system/status" => json!({"status":0}),
            "/sapi/v1/account/apiTradingStatus" => {
                json!({"data":{"isLocked":false,"updateTime":1547630471725u64,"plannedRecoverTime":0}})
            }
            "/api/v3/exchangeInfo" => {
                assert_eq!(query, "symbol=BTCUSDT&showPermissionSets=true");
                json!({"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":6000},{"rateLimitType":"ORDERS","interval":"HOUR","intervalNum":1,"limit":50},{"rateLimitType":"ORDERS","interval":"DAY","intervalNum":1,"limit":9223372036854775807i64}],"exchangeFilters":[],"symbols":[{"symbol":"BTCUSDT","status":"TRADING","baseAsset":"BTC","quoteAsset":"USDT","baseAssetPrecision":8,"quoteAssetPrecision":8,"isSpotTradingAllowed":true,"quoteOrderQtyMarketAllowed":true,"orderTypes":["LIMIT","MARKET"],"defaultSelfTradePreventionMode":"NONE","allowedSelfTradePreventionModes":["NONE"],"permissionSets":[["SPOT","MARGIN"]],"filters":[{"filterType":"PRICE_FILTER","minPrice":"0.00000000","maxPrice":"999999.00000000","tickSize":"0.01000000"},{"filterType":"LOT_SIZE","minQty":"0.00000100","maxQty":"100.00000000","stepSize":"0.00000100"}]}]})
            }
            "/api/v3/executionRules" => {
                assert_eq!(query, "symbol=BTCUSDT");
                json!({"symbolRules":[{"symbol":"BTCUSDT","rules":[{"ruleType":"PRICE_RANGE","bidLimitMultUp":"1.0001","bidLimitMultDown":"0.9999","askLimitMultUp":"1.0001","askLimitMultDown":"0.9999"}]}]})
            }
            "/api/v3/myFilters" => {
                assert!(query.starts_with("symbol=BTCUSDT&timestamp="));
                json!({"exchangeFilters":[{"filterType":"EXCHANGE_MAX_NUM_ORDERS","maxNumOrders":1000}],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":"5.00000000","applyToMarket":true,"avgPriceMins":5}],"assetFilters":[{"filterType":"MAX_ASSET","asset":"BTC","limit":"0.25000000"}]})
            }
            _ => return Err(tradex::protocol::TradeXError::new("PROVIDER_UNSUPPORTED")),
        };
        if !public && self.reflect_signature.get() {
            body["message"] = json!(query.split_once("&signature=").unwrap().1);
        }
        if self.reflect_key.get() {
            body["message"] = json!(fixtures::KEY);
        }
        let run_hook = self
            .hook
            .borrow()
            .as_ref()
            .is_some_and(|(target, _)| target == route);
        if run_hook {
            if let Some((_, hook)) = self.hook.borrow_mut().take() {
                hook();
            }
        }
        if let Some(edit) = self.edit.get() {
            edit(route, &mut body);
        }
        if let Some((target, bytes)) = self.raw.borrow().as_ref() {
            if target == route {
                return Ok(bytes.clone());
            }
        }
        Ok(serde_json::to_vec(&body).unwrap())
    }
}

#[test]
fn public_rule_refresh_collects_exact_account_and_scoped_constraints_without_execution_authority() {
    if isolated_rule_scenario(
        "public_rule_refresh_collects_exact_account_and_scoped_constraints_without_execution_authority",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let control = &rig.control;
    let workspace = &rig.workspace;
    let account = &rig.account;
    let external = RuleHttp::default();
    let read = rig.refresh(&external);
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let evidence = &read["data"]["evidence"];
    assert_eq!(evidence["kind"], "BINANCE_SPOT_RULES");
    assert_eq!(evidence["providerQuality"], "READ_ONLY_SPOT_RULES");
    assert_eq!(
        evidence["remoteAccountId"],
        account["data"]["remoteAccountId"]
    );
    assert_eq!(evidence["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(evidence["providerSymbol"], "BTCUSDT");
    assert_eq!(evidence["baseAsset"], "BTC");
    assert_eq!(evidence["quoteAsset"], "USDT");
    assert_eq!(evidence["permissionSetsSatisfied"], true);
    assert!(
        evidence["constraints"]
            .as_array()
            .is_some_and(|rows| rows.len() >= 6)
    );
    let qualifications = read["data"]["capabilityStatuses"].as_array().unwrap();
    assert_eq!(
        qualifications
            .iter()
            .find(|c| c["capability"] == "SPOT_EXECUTION_QUALIFICATION")
            .unwrap()["status"],
        "UNVERIFIED"
    );
    assert_eq!(
        qualifications
            .iter()
            .find(|c| c["capability"] == "SPOT_DATA_USE_RIGHTS")
            .unwrap()["status"],
        "UNVERIFIED"
    );
    let mut control = control.lock().unwrap();
    let retained = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(retained["data"]["health"]["arming"], "DISARMED");
    let market = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"WARM"}),
    );
    assert_eq!(market["ok"], true, "{market}");
    assert!(
        market["data"]["snapshot"].is_null(),
        "Rule reads cannot manufacture a quote: {market}"
    );
    assert_eq!(
        market["data"]["spotRuleEvidence"]["evidence"]["providerSymbol"], "BTCUSDT",
        "Markets must expose the real exact-symbol rule evidence: {market}"
    );
    let other = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"crypto:ETH/USDT:spot","tier":"WARM"}),
    );
    assert!(
        other["data"]["spotRuleEvidence"].is_null(),
        "BTC rules must not qualify ETH: {other}"
    );
    for path in [
        "/api/v3/account?",
        "/api/v3/exchangeInfo?symbol=BTCUSDT&showPermissionSets=true",
        "/api/v3/executionRules?symbol=BTCUSDT",
        "/api/v3/myFilters?symbol=BTCUSDT&",
    ] {
        assert!(
            external
                .calls
                .borrow()
                .iter()
                .any(|call| call.starts_with(path)),
            "Missing {path}"
        );
    }
}

struct RuleRig {
    _folder: tempfile::TempDir,
    control: std::sync::Arc<std::sync::Mutex<ControlPlane>>,
    workspace: Value,
    account: Value,
    vault: fixtures::Vault,
}
impl RuleRig {
    fn new() -> Self {
        let folder = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(folder.path().into());
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = fixtures::Vault::default();
        let http = fixtures::Http::default();
        static NEXT_UID: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(9007199254740993);
        http.binance_uid
            .set(NEXT_UID.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        let request = json!({"requestId":"collect-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"binance","environment":"LIVE","label":"Collect rules"}});
        let job = control
            .prepare_provider_for(&request, "main")
            .unwrap()
            .unwrap();
        let observed = job.run(
            &vault,
            |_| fixtures::credentials(),
            &http,
            || control.provider_job_current(&job),
        );
        let tested = control.complete_provider(&job, observed);
        assert_eq!(tested["ok"], true, "{tested}");
        let account = command(&mut control,"provider.connect",json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":false}))["data"].clone();
        let source = command(
            &mut control,
            "data.binance_rules.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.binance_rules.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"],"instrumentId":"crypto:BTC/USDT:spot"}),
        );
        assert_eq!(saved["ok"], true, "{saved}");
        assert_eq!(
            command(
                &mut control,
                "time.revalidate",
                json!({"workspaceId":workspace})
            )["data"]["confidence"],
            "TRUSTED"
        );
        let control = std::sync::Arc::new(std::sync::Mutex::new(control));
        Self {
            _folder: folder,
            control,
            workspace,
            account,
            vault,
        }
    }
    fn decision(&self) -> Value {
        self.decision_with(json!({"accountId":self.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}))
    }
    fn decision_with(&self, fields: Value) -> Value {
        let mut control = self.control.lock().unwrap();
        let draft = command(
            &mut control,
            "trade.save_draft",
            json!({"workspaceId":self.workspace,"fields":fields}),
        );
        assert_eq!(draft["ok"], true, "{draft}");
        let proposal = command(
            &mut control,
            "trade.generate_proposal",
            json!({"workspaceId":self.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
        );
        assert_eq!(proposal["ok"], true, "{proposal}");
        let decision = command(
            &mut control,
            "risk.evaluate_proposal",
            json!({"workspaceId":self.workspace,"proposalId":proposal["data"]["proposalId"]}),
        );
        assert_eq!(decision["ok"], true, "{decision}");
        decision["data"].clone()
    }
    fn refresh(&self, external: &RuleHttp) -> Value {
        external.uid.set(
            self.account["data"]["remoteAccountId"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
        );
        let saved = command(
            &mut self.control.lock().unwrap(),
            "data.binance_rules.connection",
            json!({"workspaceId":self.workspace}),
        );
        tradex::financial_sources::execute_refresh(
            &self.control,
            &json!({"requestId":"read-rules","schemaVersion":1,"command":"data.binance_rules.refresh","payload":{"workspaceId":self.workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
            "main",
            &self.vault,
            external,
        )
    }
}

#[test]
fn malformed_rule_range_retires_current_evidence_and_retains_the_last_good_observation() {
    if isolated_rule_scenario(
        "malformed_rule_range_retires_current_evidence_and_retains_the_last_good_observation",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"][1]["minQty"] = json!("200");
        }
    }));
    let failed = rig.refresh(&external);
    assert_eq!(
        failed["data"]["status"], "UNAVAILABLE",
        "Inverted quantity limits must fail closed: {failed}"
    );
    assert_eq!(failed["data"]["evidence"], good["data"]["evidence"]);
    assert_eq!(failed["data"]["observedAt"], good["data"]["observedAt"]);
    external.edit.set(None);
    let recovered = rig.refresh(&external);
    assert_eq!(recovered["data"]["status"], "AVAILABLE", "{recovered}");
    let account = command(
        &mut rig.control.lock().unwrap(),
        "account.get",
        json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]}),
    );
    assert_eq!(account["data"]["health"]["arming"], "DISARMED");
}

fn rule_check(decision: &Value) -> &Value {
    decision["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["checkId"] == "INSTRUMENT_RULES")
        .unwrap()
}
#[test]
fn actual_halt_evidence_rejects_new_order_rules_and_normal_recovery_never_grants_execution() {
    if isolated_rule_scenario(
        "actual_halt_evidence_rejects_new_order_rules_and_normal_recovery_never_grants_execution",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["status"] = json!("HALT");
        }
    }));
    let read = rig.refresh(&external);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    assert_eq!(
        read["data"]["evidence"]["admissionBlockers"],
        json!(["SYMBOL_NOT_TRADING"])
    );
    let halted = rig.decision();
    assert_eq!(
        rule_check(&halted)["outcome"],
        "REJECT",
        "Actual HALT must reach Trade's protected decision: {halted}"
    );
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => {
            body["symbols"][0]["status"] = json!("BREAK");
            body["symbols"][0]["isSpotTradingAllowed"] = json!(false);
        }
        "/api/v3/account" => {
            body["accountType"] = json!("MARGIN");
            body["canTrade"] = json!(false);
        }
        "/sapi/v1/account/apiRestrictions" => body["enableWithdrawals"] = json!(true),
        "/sapi/v1/system/status" => body["status"] = json!(1),
        "/sapi/v1/account/apiTradingStatus" => body["data"]["isLocked"] = json!(true),
        _ => (),
    }));
    let blocked = rig.refresh(&external);
    assert_eq!(blocked["data"]["status"], "AVAILABLE", "{blocked}");
    assert_eq!(blocked["data"]["evidence"]["symbolStatus"], "BREAK");
    let blockers = blocked["data"]["evidence"]["admissionBlockers"]
        .as_array()
        .unwrap();
    for expected in [
        "ACCOUNT_NOT_SPOT",
        "ACCOUNT_CANNOT_TRADE",
        "SPOT_NOT_ALLOWED",
        "SYMBOL_NOT_TRADING",
        "KEY_PERMISSION_BLOCKED",
        "SYSTEM_MAINTENANCE",
        "API_TRADING_LOCKED",
    ] {
        assert!(
            blockers.contains(&json!(expected)),
            "Missing distinct admission diagnosis {expected}: {blocked}"
        );
    }
    assert_eq!(rule_check(&rig.decision())["outcome"], "REJECT");
    external.edit.set(None);
    let normal = rig.refresh(&external);
    assert_eq!(normal["data"]["status"], "AVAILABLE", "{normal}");
    let recovered = rig.decision();
    assert_eq!(
        rule_check(&recovered)["outcome"],
        "UNAVAILABLE",
        "Normal metadata is not complete per-Proposal qualification: {recovered}"
    );
    assert_ne!(recovered["status"], "ALLOWED");
}

#[test]
fn original_wire_types_and_duplicate_fields_cannot_replace_a_good_observation() {
    if isolated_rule_scenario(
        "original_wire_types_and_duplicate_fields_cannot_replace_a_good_observation",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    for (route, bytes) in [
        ("/api/v3/account", br#"{"uid":7,"accountType":"SPOT","canTrade":true,"permissions":["SPOT"]}"#.to_vec()),
        ("/api/v3/account", br#"{"uid":9007199254740993,"accountType":"SPOT","canTrade":1,"permissions":["SPOT"]}"#.to_vec()),
        ("/api/v3/account", br#"{"uid":{"$serde_json::private::Number":"9007199254740993"},"accountType":"SPOT","canTrade":true,"permissions":["SPOT"]}"#.to_vec()),
        ("/sapi/v1/account/apiRestrictions", br#"{"enableReading":true,"enableSpotAndMarginTrading":{"$serde_json::private::Number":"1"}}"#.to_vec()),
        ("/sapi/v1/system/status", br#"{"status":0,"status":1}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":5,"applyToMarket":true,"avgPriceMins":5}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":"5","applyToMarket":true,"avgPriceMins":5.0}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"LOT_SIZE","minQty":"-0.01","maxQty":"10","stepSize":"0.01"}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"LOT_SIZE","minQty":"0.01","maxQty":"10","stepSize":"0.01"},{"filterType":"LOT_SIZE","minQty":"0.01","maxQty":"10","stepSize":"0.01"}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":"5","applyToMarket":true,"avgPriceMins":{"$serde_json::private::Number":"5"}}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[{"filterType":"LOT_SIZE","minQty":"0.01","maxQty":"10","stepSize":"0.01"}],"symbolFilters":[],"assetFilters":[]}"#.to_vec()),
    ] {
        *external.raw.borrow_mut() = Some((route.into(), bytes));
        let failed = rig.refresh(&external);
        assert_eq!(failed["data"]["status"], "UNAVAILABLE", "Invalid original types/scopes at {route}: {failed}");
        assert_eq!(failed["data"]["evidence"], good["data"]["evidence"], "Prior evidence must remain explicitly retained, not refreshed");
        assert_ne!(rule_check(&rig.decision())["outcome"], "PASS");
    }
}

#[test]
fn permission_sets_apply_or_within_and_between_sets_without_granting_per_order_qualification() {
    if isolated_rule_scenario(
        "permission_sets_apply_or_within_and_between_sets_without_granting_per_order_qualification",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["permissionSets"] = json!([["SPOT", "MARGIN"], ["MARGIN"]]);
        }
    }));
    let unmet = rig.refresh(&external);
    assert_eq!(unmet["data"]["status"], "AVAILABLE", "{unmet}");
    assert_eq!(unmet["data"]["evidence"]["permissionSetsSatisfied"], false);
    assert_eq!(rule_check(&rig.decision())["outcome"], "REJECT");
    external.edit.set(None);
    let accepted = rig.refresh(&external);
    assert_eq!(
        accepted["data"]["evidence"]["permissionSetsSatisfied"], true,
        "OR permits SPOT without MARGIN: {accepted}"
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/executionRules" {
            body["symbolRules"][0]["rules"] =
                json!([{"ruleType":"FUTURE_ACTIVE_RULE","limit":"1"}]);
        }
    }));
    let unknown = rig.refresh(&external);
    assert_eq!(unknown["data"]["status"], "AVAILABLE", "{unknown}");
    assert!(
        unknown["data"]["evidence"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA"))
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn changing_the_selected_instrument_during_http_prevents_late_publication() {
    if isolated_rule_scenario(
        "changing_the_selected_instrument_during_http_prevents_late_publication",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let cp = rig.control.clone();
    let workspace = rig.workspace.clone();
    let account = rig.account.clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/account".into(),
        Box::new(move || {
            let mut control = cp.lock().unwrap();
            let source = command(
                &mut control,
                "data.binance_rules.connection",
                json!({"workspaceId":workspace}),
            );
            let changed = command(
                &mut control,
                "data.binance_rules.configure",
                json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"],"instrumentId":"crypto:ETH/USDT:spot"}),
            );
            assert_eq!(changed["ok"], true, "{changed}");
        }),
    ));
    let late = rig.refresh(&external);
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT", "{late}");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(current["data"]["instrumentId"], "crypto:ETH/USDT:spot");
    assert_eq!(current["data"]["status"], "UNVERIFIED");
    assert!(current["data"]["evidence"].is_null());
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|p| p.starts_with("/api/v3/myFilters"))
    );
}

#[test]
fn elapsed_receipt_expires_rules_without_renewing_retained_evidence() {
    if isolated_rule_scenario("elapsed_receipt_expires_rules_without_renewing_retained_evidence") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    std::thread::sleep(std::time::Duration::from_millis(30_100));
    let expired = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(expired["data"]["status"], "UNAVAILABLE", "{expired}");
    assert_eq!(expired["data"]["evidence"], good["data"]["evidence"]);
    assert_eq!(expired["data"]["observedAt"], good["data"]["observedAt"]);
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn documented_price_and_asset_exponents_remain_original_bounded_integers() {
    if isolated_rule_scenario(
        "documented_price_and_asset_exponents_remain_original_bounded_integers",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => body["symbols"][0]["filters"][0]["priceExponent"] = json!(8),
        "/api/v3/myFilters" => body["assetFilters"][0]["qtyExponent"] = json!(8),
        _ => (),
    }));
    let read = rig.refresh(&external);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    for (kind, name) in [
        ("PRICE_FILTER", "priceExponent"),
        ("MAX_ASSET", "qtyExponent"),
    ] {
        let rule = read["data"]["evidence"]["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == kind)
            .unwrap();
        assert_eq!(
            rule["knownSchema"], true,
            "Documented exponent must be typed: {rule}"
        );
        let field = rule["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == name)
            .unwrap();
        assert_eq!(field["value"], json!({"type":"INTEGER","value":"8"}));
    }
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["assetFilters"][0]["qtyExponent"] = json!(65);
        }
    }));
    let invalid = rig.refresh(&external);
    assert_eq!(invalid["data"]["status"], "UNAVAILABLE", "{invalid}");
}

#[test]
fn authentication_reflections_are_rejected_before_typed_projection() {
    if isolated_rule_scenario("authentication_reflections_are_rejected_before_typed_projection") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    external.reflect_signature.set(true);
    let reflected = rig.refresh(&external);
    assert_eq!(reflected["data"]["status"], "UNAVAILABLE", "{reflected}");
    assert_eq!(reflected["data"]["evidence"], good["data"]["evidence"]);
    external.reflect_signature.set(false);
    external.reflect_key.set(true);
    let leaked = rig.refresh(&external);
    assert_eq!(leaked["data"]["status"], "UNAVAILABLE", "{leaked}");
    assert!(!leaked.to_string().contains(fixtures::KEY));
    assert!(!leaked.to_string().contains(fixtures::SECRET));
}

#[test]
fn clock_revalidation_and_reopen_retire_active_rule_evidence() {
    if isolated_rule_scenario("clock_revalidation_and_reopen_retire_active_rule_evidence") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    let clock = command(
        &mut rig.control.lock().unwrap(),
        "time.revalidate",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(clock["data"]["confidence"], "TRUSTED");
    let retired = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(retired["data"]["status"], "UNAVAILABLE");
    assert_eq!(retired["data"]["evidence"], good["data"]["evidence"]);
    let fresh = rig.refresh(&external);
    assert_eq!(fresh["data"]["status"], "AVAILABLE", "{fresh}");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/time" {
            body["serverTime"] = json!(
                (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64
                    + 60_000
            );
        }
    }));
    let future = rig.refresh(&external);
    assert_eq!(future["data"]["status"], "UNAVAILABLE", "{future}");
    assert_eq!(future["data"]["evidence"], fresh["data"]["evidence"]);
    let RuleRig {
        _folder,
        control,
        workspace,
        account,
        vault,
    } = rig;
    drop(control);
    let mut reopened = ControlPlane::new(_folder.path().into());
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let source = command(
        &mut reopened,
        "data.binance_rules.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(source["data"]["status"], "UNVERIFIED");
    assert_eq!(source["data"]["connectionId"], account["connectionId"]);
    assert_eq!(source["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert!(source["data"]["evidence"].is_null());
    assert!(source["data"]["observedAt"].is_null());
    let stored: tradex::providers::AccountConnection = serde_json::from_value(account).unwrap();
    assert!(vault.get(&stored.credential_ref()).is_ok());
}

#[test]
fn collected_constraints_name_the_unresolved_per_order_obligations() {
    if isolated_rule_scenario("collected_constraints_name_the_unresolved_per_order_obligations") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let read = rig.refresh(&external);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let obligations = read["data"]["evidence"]["unresolvedObligations"]
        .as_array()
        .unwrap();
    for expected in [
        "PRICE_TICK_AND_BOUND_VALIDATION_REQUIRED",
        "BASE_QUANTITY_GRID_VALIDATION_REQUIRED",
        "NOTIONAL_AND_MARKET_REFERENCE_VALIDATION_REQUIRED",
        "CURRENT_ACCOUNT_EXCHANGE_ORDER_COUNTS_REQUIRED",
        "EXACT_ORDER_ASSET_AMOUNT_VALIDATION_REQUIRED",
        "EXECUTION_PRICE_RANGE_UNQUALIFIED",
    ] {
        assert!(
            obligations.contains(&json!(expected)),
            "Unresolved constraint is not individually represented: {expected}: {obligations:?}"
        );
    }
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn rule_counts_preserve_large_integers_but_reject_values_outside_documented_int64() {
    if isolated_rule_scenario(
        "rule_counts_preserve_large_integers_but_reject_values_outside_documented_int64",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["exchangeFilters"][0]["maxNumOrders"] = json!(9007199254740993u64);
        }
    }));
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    let constraint = good["data"]["evidence"]["constraints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "EXCHANGE_MAX_NUM_ORDERS")
        .unwrap();
    assert_eq!(
        constraint["fields"][0]["value"],
        json!({"type":"INTEGER","value":"9007199254740993"})
    );
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["exchangeFilters"][0]["maxNumOrders"] = json!(u64::MAX);
        }
    }));
    let failed = rig.refresh(&external);
    assert_eq!(
        failed["data"]["status"], "UNAVAILABLE",
        "Out-of-range int64 must fail closed: {failed}"
    );
    assert_eq!(failed["data"]["evidence"], good["data"]["evidence"]);
    assert_ne!(rule_check(&rig.decision())["outcome"], "PASS");
}

#[test]
fn unpopulated_permission_sets_never_grant_account_admission() {
    if isolated_rule_scenario("unpopulated_permission_sets_never_grant_account_admission") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => body["symbols"][0]["permissionSets"] = json!([]),
        "/api/v3/account" => body["permissions"] = json!([]),
        _ => (),
    }));
    let empty = rig.refresh(&external);
    assert_eq!(
        empty["data"]["status"], "UNAVAILABLE",
        "Unpopulated permissions are not admission evidence: {empty}"
    );
    assert_eq!(empty["data"]["evidence"], good["data"]["evidence"]);
    assert!(
        empty["data"]["capabilityStatuses"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] != "AVAILABLE")
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["permissionSets"] = json!([[]]);
        }
    }));
    let empty_inner = rig.refresh(&external);
    assert_eq!(empty_inner["data"]["status"], "UNAVAILABLE");
    external.edit.set(None);
    let recovered = rig.refresh(&external);
    assert_eq!(recovered["data"]["status"], "AVAILABLE", "{recovered}");
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn original_order_form_flags_are_preserved_or_explicitly_unobserved_and_never_grant_advanced_forms()
{
    if isolated_rule_scenario(
        "original_order_form_flags_are_preserved_or_explicitly_unobserved_and_never_grant_advanced_forms",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            for (name, enabled) in [
                ("icebergAllowed", true),
                ("ocoAllowed", false),
                ("otoAllowed", true),
                ("opoAllowed", false),
                ("allowTrailingStop", true),
                ("cancelReplaceAllowed", false),
                ("amendAllowed", true),
                ("pegInstructionsAllowed", false),
            ] {
                body["symbols"][0][name] = json!(enabled);
            }
        }
    }));
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE");
    assert_eq!(
        good["data"]["evidence"]["orderFormFlags"],
        json!({"icebergAllowed":true,"ocoAllowed":false,"otoAllowed":true,"opoAllowed":false,"allowTrailingStop":true,"cancelReplaceAllowed":false,"amendAllowed":true,"pegInstructionsAllowed":false})
    );
    assert!(
        good["data"]["evidence"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("ADVANCED_ORDER_FORMS_UNSUPPORTED"))
    );
    let raw = external
        .get(
            tradex::provider_io::ProviderEndpoint::BinanceLive,
            "/api/v3/exchangeInfo?symbol=BTCUSDT&showPermissionSets=true",
            reqwest::header::HeaderMap::new(),
        )
        .unwrap();
    for name in [
        "icebergAllowed",
        "ocoAllowed",
        "otoAllowed",
        "opoAllowed",
        "allowTrailingStop",
        "cancelReplaceAllowed",
        "amendAllowed",
        "pegInstructionsAllowed",
    ] {
        let mut malformed: Value = serde_json::from_slice(&raw).unwrap();
        malformed["symbols"][0][name] = json!(1);
        *external.raw.borrow_mut() = Some((
            "/api/v3/exchangeInfo".into(),
            serde_json::to_vec(&malformed).unwrap(),
        ));
        let rejected = rig.refresh(&external);
        assert_eq!(
            rejected["data"]["status"], "UNAVAILABLE",
            "Nonboolean {name} must fail closed: {rejected}"
        );
        assert_eq!(rejected["data"]["evidence"], good["data"]["evidence"]);
    }
    *external.raw.borrow_mut() = None;
    external.edit.set(None);
    let absent = rig.refresh(&external);
    assert_eq!(absent["data"]["status"], "AVAILABLE");
    assert!(
        absent["data"]["evidence"]["orderFormFlags"]
            .as_object()
            .unwrap()
            .values()
            .all(Value::is_null)
    );
    assert!(
        absent["data"]["evidence"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("ORDER_FORM_FLAGS_UNOBSERVED"))
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

// S29.8 — owning Spot Live PLACE qualification derived from the delivered capacity and
// interval inputs. Every case drives the public account/source/draft/Proposal/rules/capacity/
// interval/risk/approval flow with only the external HTTP producer fake.
fn capacity_get(rig: &RuleRig, proposal: &Value) -> Value {
    command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_capacity.get",
        json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}),
    )
}
fn capacity_refresh(rig: &RuleRig, proposal: &Value, external: &RuleHttp) -> Value {
    let before = capacity_get(rig, proposal);
    tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capacity-collection","schemaVersion":1,"command":"trade.spot_capacity.refresh","payload":{"workspaceId":rig.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        external,
    )
}
fn risk_evaluate(rig: &RuleRig, proposal: &Value) -> Value {
    command(
        &mut rig.control.lock().unwrap(),
        "risk.evaluate_proposal",
        json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}),
    )["data"]
        .clone()
}
fn request_approval(rig: &RuleRig, proposal: &Value) -> Value {
    command(
        &mut rig.control.lock().unwrap(),
        "trade.request_approval",
        json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}),
    )
}
fn sell_proposal(rig: &RuleRig) -> Value {
    rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"SELL","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}))
}

#[test]
fn owning_spot_qualification_derives_current_declared_headroom_without_granting_authority() {
    if isolated_rule_scenario(
        "owning_spot_qualification_derives_current_declared_headroom_without_granting_authority",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let decision = risk_evaluate(&rig, &proposal);
    let owning = &decision["spotOwning"];
    assert_eq!(owning["outcome"], "PASS", "{decision}");
    assert_eq!(owning["reasonCode"], "SPOT_OWNING_QUALIFIED");
    assert!(owning["bindingBlocker"].is_null(), "{owning}");
    assert_eq!(owning["proposalId"], proposal["proposalId"]);
    assert_eq!(owning["proposalHash"], proposal["proposalHash"]);
    assert_eq!(owning["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(owning["baseAsset"], "BTC");
    assert_eq!(owning["quoteAsset"], "USDT");
    assert_eq!(
        owning["capacityUnit"], "USDT",
        "A buy is funded in the quote asset: {owning}"
    );
    // Derived, never stored: limit 50 minus count 0, with the tighter of the two declared
    // buckets binding and its declared origin named.
    assert_eq!(owning["remainingOrderSlots"], "50");
    assert_eq!(owning["bindingInterval"], "ORDERS 50/1 HOUR");
    assert_eq!(owning["declaredSymbolOpenOrders"], "1");
    assert!(owning["declaredSymbolOpenBuyQuantity"].is_null());
    assert!(owning["declaredBaseFree"].is_null());
    assert!(owning["declaredBaseLocked"].is_null());
    assert_eq!(owning["inventorySource"], "PROVIDER_DECLARED_OPEN_ORDERS");
    assert_eq!(owning["quotaSource"], "PROVIDER_DECLARED_ORDER_RATE_COUNTERS");
    assert!(owning["bindingVersion"].as_str().unwrap().starts_with("sha256:"));
    assert!(owning["stateVersion"].as_str().unwrap().starts_with("sha256:"));
    // The standing provenance labels of both consumed slices are carried forward verbatim
    // instead of being promoted into a second, hidden admission veto.
    for label in [
        "NON_ATOMIC_PROVIDER_SNAPSHOT",
        "DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED",
        "COUNTER_SNAPSHOT_TIME_UNAVAILABLE",
        "DERIVED_WINDOW_ASSOCIATION_UNCERTAIN",
        "INTERVAL_INPUTS_NOT_EXECUTION_QUALIFIED",
    ] {
        assert!(
            owning["carriedLimitations"]
                .as_array()
                .unwrap()
                .contains(&json!(label)),
            "{label} must be carried: {owning}"
        );
    }
    // Execution-unqualified: the statement itself names no arming, consent or dispatch.
    assert!(
        owning
            .as_object()
            .unwrap()
            .keys()
            .all(|key| !["arm", "consent", "dispatch", "execute"]
                .iter()
                .any(|word| key.to_lowercase().contains(word))),
        "{owning}"
    );
    let retained = command(
        &mut rig.control.lock().unwrap(),
        "account.get",
        json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]}),
    );
    assert_eq!(
        retained["data"]["health"]["arming"], "DISARMED",
        "A qualified own admission must never arm: {retained}"
    );
}

#[test]
fn owning_spot_qualification_shows_exactly_one_slot_until_the_venue_counter_advances() {
    if isolated_rule_scenario(
        "owning_spot_qualification_shows_exactly_one_slot_until_the_venue_counter_advances",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/rateLimit/order" {
            body[0]["count"] = json!(49);
        }
    }));
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let owning = risk_evaluate(&rig, &proposal)["spotOwning"].clone();
    assert_eq!(owning["outcome"], "PASS", "{owning}");
    assert_eq!(
        owning["remainingOrderSlots"], "1",
        "49 of a declared 50 leaves exactly one: {owning}"
    );
    // The venue's own counter is the authority for that one slot: once it reaches the limit the
    // same intent is refused. Nothing here decremented, reset or rolled the window locally.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/rateLimit/order" {
            body[0]["count"] = json!(50);
        }
    }));
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let spent = risk_evaluate(&rig, &proposal)["spotOwning"].clone();
    assert_eq!(spent["outcome"], "REJECT", "{spent}");
    assert_eq!(spent["reasonCode"], "SPOT_INTERVAL_QUOTA_EXHAUSTED");
    assert_eq!(spent["bindingBlocker"], "SPOT_INTERVAL_QUOTA_EXHAUSTED");
    assert!(spent["remainingOrderSlots"].is_null());
}

#[test]
fn owning_spot_qualification_blocks_an_exhausted_order_rate_window_from_being_approved() {
    if isolated_rule_scenario(
        "owning_spot_qualification_blocks_an_exhausted_order_rate_window_from_being_approved",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/rateLimit/order" {
            body[0]["count"] = json!(50);
        }
    }));
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let review = request_approval(&rig, &proposal);
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(
        review["data"]["eligible"], false,
        "An exhausted venue window must not be eligible: {review}"
    );
    assert_eq!(
        review["data"]["riskDecision"]["spotOwning"]["outcome"],
        "REJECT"
    );
    assert!(
        review["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .unwrap()
                .starts_with("SPOT_INTERVAL_QUOTA_EXHAUSTED")),
        "The single binding fact must be named: {review}"
    );
}

#[test]
fn owning_spot_qualification_fails_closed_on_incomplete_coverage_and_unbound_evidence() {
    if isolated_rule_scenario(
        "owning_spot_qualification_fails_closed_on_incomplete_coverage_and_unbound_evidence",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();

    // A declared order-rate interval with no matching counter is unknown coverage, not headroom.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["rateLimits"]
                .as_array_mut()
                .unwrap()
                .push(json!({"rateLimitType":"ORDERS","interval":"MINUTE","intervalNum":1,"limit":10}));
        }
    }));
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let gapped = risk_evaluate(&rig, &proposal)["spotOwning"].clone();
    assert_eq!(gapped["outcome"], "UNAVAILABLE", "{gapped}");
    assert_eq!(gapped["reasonCode"], "SPOT_INTERVAL_EVIDENCE_UNAVAILABLE");
    assert_eq!(gapped["bindingBlocker"], "SPOT_INTERVAL_EVIDENCE_UNAVAILABLE");
    assert!(gapped["remainingOrderSlots"].is_null());

    // A declared count that exceeds its own declared limit is the venue's numbers disagreeing.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/rateLimit/order" {
            body[0]["count"] = json!(60);
        }
    }));
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let over = risk_evaluate(&rig, &proposal)["spotOwning"].clone();
    assert_eq!(over["outcome"], "UNAVAILABLE", "{over}");
    assert_eq!(over["reasonCode"], "SPOT_INTERVAL_EVIDENCE_UNAVAILABLE");
    assert_eq!(over["bindingBlocker"], "SPOT_INTERVAL_EVIDENCE_UNAVAILABLE");

    // An order row the venue's own classification cannot account for is unknown coverage too.
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/openOrders" {
            body[0]["unclassifiedProviderField"] = json!(1);
        }
    }));
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let partial = risk_evaluate(&rig, &proposal)["spotOwning"].clone();
    assert_eq!(partial["outcome"], "UNAVAILABLE", "{partial}");
    assert_eq!(partial["reasonCode"], "SPOT_CAPACITY_COVERAGE_INCOMPLETE");
    assert_eq!(partial["bindingBlocker"], "SPOT_CAPACITY_COVERAGE_INCOMPLETE");

    // A fresh intent with no bound interval observation at all is equally unavailable.
    external.edit.set(None);
    let unbound = sell_proposal(&rig);
    assert_eq!(
        capacity_refresh(&rig, &unbound, &external)["data"]["status"],
        "OBSERVED"
    );
    let unobserved = risk_evaluate(&rig, &unbound)["spotOwning"].clone();
    assert_eq!(unobserved["outcome"], "UNAVAILABLE", "{unobserved}");
    assert_eq!(
        unobserved["reasonCode"], "SPOT_INTERVAL_EVIDENCE_UNAVAILABLE"
    );
    assert_eq!(
        unobserved["capacityUnit"], "BTC",
        "A sell is funded in the base asset: {unobserved}"
    );
}

#[test]
fn owning_spot_qualification_surfaces_an_interval_window_that_cannot_be_shown_to_hold() {
    if isolated_rule_scenario(
        "owning_spot_qualification_surfaces_an_interval_window_that_cannot_be_shown_to_hold",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    // One-second buckets make the conservative local association lapse within a second: the
    // counter has no provider timestamp, so the window it was read in can no longer be shown.
    external.edit.set(Some(|route, body| {
        match route {
            "/api/v3/exchangeInfo" => body["rateLimits"]
                .as_array_mut()
                .unwrap()
                .push(json!({"rateLimitType":"ORDERS","interval":"SECOND","intervalNum":1,"limit":10})),
            "/api/v3/rateLimit/order" => body
                .as_array_mut()
                .unwrap()
                .push(json!({"rateLimitType":"ORDERS","interval":"SECOND","intervalNum":1,"limit":10,"count":0})),
            _ => {}
        }
    }));
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let retired = interval_get(&rig, &proposal);
    assert_eq!(
        retired["data"]["status"], "STALE",
        "A lapsed association must retire the counters: {retired}"
    );
    assert_eq!(
        retired["data"]["retirementReason"], "POSSIBLE_INTERVAL_BOUNDARY",
        "{retired}"
    );
    let owning = risk_evaluate(&rig, &proposal)["spotOwning"].clone();
    assert_eq!(owning["outcome"], "UNAVAILABLE", "{owning}");
    assert_eq!(owning["reasonCode"], "SPOT_INTERVAL_WINDOW_UNCERTAIN");
    assert_eq!(owning["bindingBlocker"], "SPOT_INTERVAL_WINDOW_UNCERTAIN");
    assert!(owning["remainingOrderSlots"].is_null());
}

// S29.9 — owning fee statement and genuinely-required execution-FX statement. These cases drive the
// same delivered flow as the S29.8 rig above and assert the statement fails closed without relaxing
// any producer, assuming any parity or fabricating any fee amount.
#[test]
fn owning_fee_fx_statement_binds_the_intent_and_fails_closed_without_relaxing_producers() {
    if isolated_rule_scenario(
        "owning_fee_fx_statement_binds_the_intent_and_fails_closed_without_relaxing_producers",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let decision = risk_evaluate(&rig, &proposal);
    let statement = &decision["spotFeeFx"];
    assert!(
        !statement.is_null(),
        "The fee/FX statement must be present: {decision}"
    );
    // Identity is bound to the immutable intent, never a renderer-supplied value.
    assert_eq!(statement["proposalId"], proposal["proposalId"]);
    assert_eq!(statement["proposalHash"], proposal["proposalHash"]);
    assert_eq!(statement["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(statement["baseAsset"], "BTC");
    assert_eq!(statement["quoteAsset"], "USDT");
    assert_eq!(statement["baseCurrency"], "USD");
    assert_eq!(statement["side"], "BUY");
    // The fee currency is never genuinely declared by a delivered host: fail closed, never an amount.
    assert_eq!(statement["feeCurrency"], "UNKNOWN", "{statement}");
    assert_eq!(statement["feeCurrencyOrigin"], "UNKNOWN", "{statement}");
    assert!(statement["expectedFee"].is_null(), "{statement}");
    // The genuinely-required USDT -> base conversion is unsupported by the bounded producer, so it
    // is the single binding fact; the route is named verbatim and parity is never inferred.
    assert_eq!(statement["outcome"], "UNAVAILABLE", "{statement}");
    assert_eq!(
        statement["reasonCode"], "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE",
        "{statement}"
    );
    assert_eq!(
        statement["bindingBlocker"], "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE",
        "{statement}"
    );
    let routes = statement["routes"].as_array().unwrap();
    let intent = routes
        .iter()
        .find(|route| route["purpose"] == "INTENT_POLICY")
        .unwrap_or_else(|| panic!("The intent-policy route must be named: {statement}"));
    assert_eq!(intent["fromCurrency"], "USDT", "{statement}");
    assert_eq!(intent["toCurrency"], "USD", "{statement}");
    assert_eq!(intent["state"], "UNSUPPORTED_ROUTE", "{statement}");
    assert!(intent["providerPair"].is_null(), "{statement}");
    // Both bound evidence versions are full 71-character `sha256:` digests.
    for field in ["proposalHash", "feeEvidenceVersion", "routeEvidenceVersion"] {
        let value = statement[field].as_str().unwrap_or_default();
        assert!(value.starts_with("sha256:"), "{field}: {value}");
        assert_eq!(value.len(), 71, "{field}: {value}");
    }
    // The reason is single-line: the review UI joins blockers with ' · ' on one line.
    assert!(
        !statement["reason"].as_str().unwrap().contains('\n'),
        "{statement}"
    );
    // Execution-unqualified: the statement names no arming, consent or dispatch.
    assert!(
        statement.as_object().unwrap().keys().all(|key| ![
            "arm", "consent", "dispatch", "execute"
        ]
        .iter()
        .any(|word| key.to_lowercase().contains(word))),
        "{statement}"
    );
}

#[test]
fn owning_fee_fx_gate_blocks_approval_and_names_the_single_binding_fact() {
    if isolated_rule_scenario(
        "owning_fee_fx_gate_blocks_approval_and_names_the_single_binding_fact",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    // The owning gate itself passes on this rig; only the fee/FX gate withholds the review.
    let owning = risk_evaluate(&rig, &proposal)["spotOwning"].clone();
    assert_eq!(owning["outcome"], "PASS", "{owning}");
    let review = request_approval(&rig, &proposal);
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(review["data"]["eligible"], false, "{review}");
    // The statement is surfaced verbatim on the review and on its bound decision.
    assert_eq!(
        review["data"]["spotFeeFx"], review["data"]["riskDecision"]["spotFeeFx"],
        "{review}"
    );
    assert!(
        review["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .unwrap()
                .starts_with("SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE")),
        "The single binding fact must be named: {review}"
    );
    // Every blocker stays single-line so the review UI can join them with ' · '.
    assert!(
        review["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|blocker| !blocker.as_str().unwrap().contains('\n')),
        "{review}"
    );
}

#[test]
fn owning_fee_fx_stays_visible_when_capacity_evidence_is_absent() {
    if isolated_rule_scenario("owning_fee_fx_stays_visible_when_capacity_evidence_is_absent") {
        return;
    }
    let rig = RuleRig::new();
    // The owning path is selected (its source is configured) but no capacity observation is bound.
    // The statement must still be produced and shown (never silently omitted): evidence absence is
    // reported as a non-PASS statement, not as a missing one.
    let proposal = rig.decision();
    let review = request_approval(&rig, &proposal);
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(review["data"]["eligible"], false, "{review}");
    let statement = &review["data"]["spotFeeFx"];
    assert!(
        !statement.is_null(),
        "A missing observation must still surface a statement, never silence it: {review}"
    );
    assert_eq!(statement["feeCurrency"], "UNKNOWN", "{statement}");
    assert_eq!(
        statement["bindingBlocker"], "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE",
        "{statement}"
    );
    assert!(
        review["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .unwrap()
                .starts_with("SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE")),
        "The single binding fact must be named: {review}"
    );
}

#[test]
fn owning_fee_fx_binding_blocker_prefers_the_route_and_keeps_the_fee_fact_visible() {
    if isolated_rule_scenario(
        "owning_fee_fx_binding_blocker_prefers_the_route_and_keeps_the_fee_fact_visible",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let decision = risk_evaluate(&rig, &proposal);
    let statement = &decision["spotFeeFx"];
    // P2 priority: the unsupported required route is the single binding blocker, because a
    // structural impossibility precedes any fee arithmetic.
    assert_eq!(
        statement["reasonCode"], "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE",
        "{statement}"
    );
    assert_eq!(
        statement["bindingBlocker"], "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE",
        "{statement}"
    );
    // The co-occurring fee-currency fact is still reported and never silently dropped.
    assert_eq!(statement["feeCurrency"], "UNKNOWN", "{statement}");
    assert_eq!(statement["feeCurrencyOrigin"], "UNKNOWN", "{statement}");
    assert!(
        statement["reason"]
            .as_str()
            .unwrap()
            .contains("fee currency remains UNKNOWN"),
        "The fee-currency fact must survive in the reason: {statement}"
    );
    // The binding route is the intent-policy USDT -> base conversion that has no supported pair.
    let intent = statement["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["purpose"] == "INTENT_POLICY")
        .expect("The intent-policy route must be named");
    assert_eq!(intent["state"], "UNSUPPORTED_ROUTE", "{statement}");
    assert!(intent["providerPair"].is_null(), "{statement}");
}

#[test]
fn owning_fee_fx_declared_fee_change_updates_the_approval_bound_material_without_authority() {
    if isolated_rule_scenario(
        "owning_fee_fx_declared_fee_change_updates_the_approval_bound_material_without_authority",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/account" {
            body["commissionRates"] =
                json!({"maker":"0.0010","taker":"0.0015","buyer":"0.0000","seller":"0.0000"});
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let first = request_approval(&rig, &proposal);
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(first["data"]["eligible"], false, "{first}");
    assert_eq!(
        first["data"]["spotFeeFx"]["declaredMakerRate"], "0.0010",
        "{first}"
    );
    assert_eq!(
        first["data"]["spotFeeFx"]["feeRateBasis"], "TAKER",
        "{first}"
    );
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/account" {
            body["commissionRates"] =
                json!({"maker":"0.0020","taker":"0.0015","buyer":"0.0000","seller":"0.0000"});
        }
    }));
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let second = request_approval(&rig, &proposal);
    assert_eq!(second["ok"], true, "{second}");
    assert_eq!(second["data"]["eligible"], false, "{second}");
    assert_eq!(
        second["data"]["spotFeeFx"]["declaredMakerRate"], "0.0020",
        "{second}"
    );
    assert_eq!(
        second["data"]["spotFeeFx"]["feeRateBasis"], "MAKER",
        "{second}"
    );
    assert_ne!(
        first["data"]["spotFeeFx"]["feeEvidenceVersion"],
        second["data"]["spotFeeFx"]["feeEvidenceVersion"]
    );
    assert_ne!(
        first["data"]["reviewDigest"],
        second["data"]["reviewDigest"]
    );
    assert!(second["data"]["spotFeeFx"]["expectedFee"].is_null());
    assert_eq!(second["data"]["spotFeeFx"]["feeCurrency"], "UNKNOWN");
    assert_eq!(
        second["data"]["spotFeeFx"]["bindingBlocker"],
        "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE"
    );
}

#[test]
fn owning_fee_fx_gate_refuses_approval_and_leaves_no_partial_state() {
    if isolated_rule_scenario("owning_fee_fx_gate_refuses_approval_and_leaves_no_partial_state") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        interval_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let review = request_approval(&rig, &proposal);
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(review["data"]["eligible"], false, "{review}");
    // The approval action carries the immutable proposal identity, not the risk decision's.
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.proposal.get",
        json!({"workspaceId": rig.workspace, "proposalId": proposal["proposalId"]}),
    );
    assert_eq!(current["ok"], true, "{current}");
    // The fee/FX gate refuses issuance itself: the exact reviewed material is carried and the
    // approval is still refused with RISK_EVIDENCE_UNAVAILABLE before any approval exists.
    let approve = command(
        &mut rig.control.lock().unwrap(),
        "trade.approve",
        json!({
            "workspaceId": rig.workspace,
            "proposalId": proposal["proposalId"],
            "proposalHash": current["data"]["proposalHash"],
            "reviewedRiskDecisionId": review["data"]["riskDecision"]["decisionId"],
            "reviewDigest": review["data"]["reviewDigest"],
            "expectedStateVersion": current["data"]["stateVersion"],
        }),
    );
    assert_eq!(approve["ok"], false, "{approve}");
    assert_eq!(
        approve["error"]["code"], "RISK_EVIDENCE_UNAVAILABLE",
        "{approve}"
    );
    // No partial state: no approval is issued and the proposal never left NEEDS_APPROVAL.
    let approvals = command(
        &mut rig.control.lock().unwrap(),
        "trade.approval.list",
        json!({"workspaceId": rig.workspace, "proposalId": proposal["proposalId"]}),
    );
    assert_eq!(approvals["ok"], true, "{approvals}");
    assert!(
        approvals["data"]["approvals"]
            .as_array()
            .unwrap()
            .is_empty(),
        "No approval may be issued while the fee/FX gate withholds: {approvals}"
    );
    let retained = command(
        &mut rig.control.lock().unwrap(),
        "trade.proposal.get",
        json!({"workspaceId": rig.workspace, "proposalId": proposal["proposalId"]}),
    );
    assert_eq!(retained["ok"], true, "{retained}");
    assert_eq!(
        retained["data"]["status"], "NEEDS_APPROVAL",
        "The proposal must remain NEEDS_APPROVAL: {retained}"
    );
}

#[test]
fn capacity_seventeen_unknown_commission_keys_stay_an_obligation_and_are_bounded_and_sorted() {
    if isolated_rule_scenario(
        "capacity_seventeen_unknown_commission_keys_stay_an_obligation_and_are_bounded_and_sorted",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/account" {
            let mut rates =
                json!({"maker":"0.0010","taker":"0.0015","buyer":"0.0000","seller":"0.0000"});
            if let Some(object) = rates.as_object_mut() {
                for index in 0..20 {
                    object.insert(format!("futureKey{index:02}"), json!("0.0001"));
                }
            }
            body["commissionRates"] = rates;
        }
    }));
    let observed = capacity_refresh(&rig, &proposal, &external);
    assert_eq!(observed["data"]["status"], "OBSERVED", "{observed}");
    let observation = &observed["data"]["observation"];
    // The delivered unknown-key obligation still holds even though the projection is bounded.
    assert!(
        observation["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("UNKNOWN_ACTIVE_ACCOUNT_FIELDS_UNRESOLVED")),
        "Unknown commission keys silently disappeared: {observed}"
    );
    let commission = &observation["declaredCommission"];
    assert_eq!(commission["maker"], "0.0010", "{observation}");
    assert_eq!(commission["taker"], "0.0015", "{observation}");
    let keys = commission["extensionKeys"].as_array().unwrap();
    assert_eq!(
        keys.len(),
        16,
        "At most 16 unknown commission keys are reported: {commission}"
    );
    let mut sorted = keys.clone();
    sorted.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    assert_eq!(
        *keys, sorted,
        "The reported unknown keys must be sorted: {commission}"
    );
    let first = keys.first().unwrap().as_str().unwrap();
    let last = keys.last().unwrap().as_str().unwrap();
    assert_eq!(first, "futureKey00", "{commission}");
    assert_eq!(last, "futureKey15", "{commission}");
    // The unknown key VALUES are never projected into a rate or leaked into the observation.
    assert!(
        !observed.to_string().contains("0.0001"),
        "An unknown commission value leaked into the projection: {observed}"
    );
}

#[test]
fn capacity_commission_extension_names_must_fit_the_declared_wire_bound() {
    if isolated_rule_scenario(
        "capacity_commission_extension_names_must_fit_the_declared_wire_bound",
    ) {
        return;
    }
    let cases: [fn(&str, &mut Value); 2] = [
        |route, body| {
            if route == "/api/v3/account" {
                body["commissionRates"] = json!({"maker":"0.0010","taker":"0.0015","buyer":"0.0000","seller":"0.0000", "": "not a projected rate"});
            }
        },
        |route, body| {
            if route == "/api/v3/account" {
                body["commissionRates"] =
                    json!({"maker":"0.0010","taker":"0.0015","buyer":"0.0000","seller":"0.0000"});
                body["commissionRates"]
                    .as_object_mut()
                    .unwrap()
                    .insert("x".repeat(65), json!("not a projected rate"));
            }
        },
    ];
    for mutate in cases {
        let rig = RuleRig::new();
        let external = RuleHttp::default();
        assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
        let proposal = rig.decision();
        external.edit.set(Some(mutate));
        let result = capacity_refresh(&rig, &proposal, &external);
        assert_eq!(result["data"]["status"], "UNAVAILABLE", "{result}");
        assert!(
            result["data"]["observation"].is_null(),
            "Malformed disclosure must not leave observed data: {result}"
        );
    }
}

#[test]
fn owning_fee_fx_declared_basis_compares_actual_rates_numerically_without_a_fee_estimate() {
    if isolated_rule_scenario(
        "owning_fee_fx_declared_basis_compares_actual_rates_numerically_without_a_fee_estimate",
    ) {
        return;
    }
    let cases: [(fn(&str, &mut Value), &str); 3] = [
        (
            |route, body| {
                if route == "/api/v3/account" {
                    body["commissionRates"] = json!({"maker":"0.00100","taker":"0.0010","buyer":"0.0000","seller":"0.0000"});
                }
            },
            "TAKER",
        ),
        (
            |route, body| {
                if route == "/api/v3/account" {
                    body["commissionRates"] = json!({"maker":"0.0020","taker":"0.0010","buyer":"0.0000","seller":"0.0000"});
                }
            },
            "MAKER",
        ),
        (
            |route, body| {
                if route == "/api/v3/account" {
                    body["commissionRates"] = json!({"maker":"0.0010","taker":"0.0020","buyer":"0.0000","seller":"0.0000"});
                }
            },
            "TAKER",
        ),
    ];
    for (mutate, expected_basis) in cases {
        let rig = RuleRig::new();
        let external = RuleHttp::default();
        assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
        let proposal = rig.decision();
        external.edit.set(Some(mutate));
        assert_eq!(
            capacity_refresh(&rig, &proposal, &external)["data"]["status"],
            "OBSERVED"
        );
        let decision = risk_evaluate(&rig, &proposal);
        let statement = &decision["spotFeeFx"];
        assert_eq!(statement["feeRateBasis"], expected_basis, "{statement}");
        assert!(statement["declaredMakerRate"].is_string());
        assert!(statement["declaredTakerRate"].is_string());
        assert_eq!(statement["feeCurrency"], "UNKNOWN");
        assert!(statement["expectedFee"].is_null());
        assert_eq!(statement["outcome"], "UNAVAILABLE");
    }
}

#[test]
fn owning_fee_fx_route_blocker_does_not_invent_a_missing_commission_observation() {
    if isolated_rule_scenario(
        "owning_fee_fx_route_blocker_does_not_invent_a_missing_commission_observation",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/account" {
            body.as_object_mut().unwrap().remove("commissionRates");
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let decision = risk_evaluate(&rig, &proposal);
    let statement = &decision["spotFeeFx"];
    assert_eq!(
        statement["bindingBlocker"],
        "SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE"
    );
    assert!(statement["declaredMakerRate"].is_null());
    assert!(statement["declaredTakerRate"].is_null());
    assert!(
        statement["reason"]
            .as_str()
            .unwrap()
            .contains("No current declared commission observation"),
        "Missing rates cannot be described as declared: {statement}"
    );
    assert!(
        !statement["reason"]
            .as_str()
            .unwrap()
            .contains("The venue declares commission rates")
    );
}

#[test]
fn owning_fee_fx_captured_history_survives_refresh_and_reopen_without_a_provider_read() {
    if isolated_rule_scenario(
        "owning_fee_fx_captured_history_survives_refresh_and_reopen_without_a_provider_read",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/account" {
            body["commissionRates"] =
                json!({"maker":"0.0010","taker":"0.0015","buyer":"0.0000","seller":"0.0000"});
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let captured = risk_evaluate(&rig, &proposal);
    assert_eq!(captured["spotFeeFx"]["declaredMakerRate"], "0.0010");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/account" {
            body["commissionRates"] =
                json!({"maker":"0.0020","taker":"0.0015","buyer":"0.0000","seller":"0.0000"});
        }
    }));
    assert_eq!(
        capacity_refresh(&rig, &proposal, &external)["data"]["status"],
        "OBSERVED"
    );
    let query = json!({"workspaceId": rig.workspace, "proposalId": proposal["proposalId"]});
    let history = command(
        &mut rig.control.lock().unwrap(),
        "risk.decision.list",
        query.clone(),
    );
    let saved = history["data"]["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["decisionId"] == captured["decisionId"])
        .unwrap();
    assert_eq!(saved["spotFeeFx"], captured["spotFeeFx"]);
    let calls = external.calls.borrow().len();
    let RuleRig {
        _folder, control, ..
    } = rig;
    drop(control);
    let mut reopened = ControlPlane::new(_folder.path().into());
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(&mut reopened, "risk.decision.list", query.clone());
    let saved = restored["data"]["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["decisionId"] == captured["decisionId"])
        .unwrap();
    assert_eq!(saved["spotFeeFx"], captured["spotFeeFx"]);
    let current = command(&mut reopened, "trade.spot_capacity.get", query);
    assert!(current["data"]["observation"].is_null());
    assert_eq!(external.calls.borrow().len(), calls);
}

// S29.10 public source path: normal application commands; only external HTTP/vault are fake.
fn commission_get(rig: &RuleRig, proposal: &Value) -> Value {
    command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_commission.get",
        json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}),
    )
}
fn commission_refresh(rig: &RuleRig, proposal: &Value, external: &RuleHttp) -> Value {
    let before = commission_get(rig, proposal);
    tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"commission-collection","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":rig.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        external,
    )
}
#[test]
fn symbol_commission_collects_all_declared_families_for_the_exact_account_without_authority() {
    if isolated_rule_scenario(
        "symbol_commission_collects_all_declared_families_for_the_exact_account_without_authority",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let proposal = rig.decision();
    let missing = commission_get(&rig, &proposal);
    assert_eq!(
        missing["ok"], true,
        "A user must be able to inspect the exact Proposal fee-source state: {missing}"
    );
    assert_eq!(missing["data"]["status"], "NOT_OBSERVED");
    let first = external.calls.borrow().len();
    let collected = commission_refresh(&rig, &proposal, &external);
    assert_eq!(collected["data"]["status"], "OBSERVED", "{collected}");
    assert_eq!(collected["data"]["qualification"], "UNAVAILABLE");
    assert_eq!(collected["data"]["proposalHash"], proposal["proposalHash"]);
    assert_eq!(collected["data"]["accountId"], rig.account["connectionId"]);
    let terms = &collected["data"]["observation"];
    assert_eq!(terms["symbol"], "BTCUSDT");
    assert_eq!(terms["standardCommission"]["maker"], "0.00000010");
    assert_eq!(terms["taxCommission"]["seller"], "0.00000116");
    assert_eq!(terms["specialCommission"]["buyer"], "0.03000000");
    assert_eq!(terms["discount"]["discount"], "0.75000000");
    assert_eq!(terms["termsComplete"], true);
    assert!(terms["providerObservedAt"].is_null());
    assert_eq!(
        terms["receivedAsset"], "BTC",
        "BUY fees come from the received BASE branch"
    );
    assert_eq!(terms["conditionalDiscountAsset"], "BNB");
    assert_eq!(terms["chargingAssetQualified"], false);
    assert!(terms["feeAmount"].is_null());
    let calls = external.calls.borrow();
    let fresh = &calls[first..];
    assert_eq!(
        fresh.len(),
        3,
        "One bounded explicit collection uses time/account/commission GETs: {fresh:?}"
    );
    assert!(
        fresh[0] == "/api/v3/time"
            && fresh[1].starts_with("/api/v3/account?timestamp=")
            && fresh[2].starts_with("/api/v3/account/commission?symbol=BTCUSDT&timestamp=")
    );
    drop(calls);
    let retained = commission_get(&rig, &proposal);
    assert_eq!(
        retained["data"], collected["data"],
        "Reading cached evidence must not renew its receipt"
    );
    assert_eq!(external.calls.borrow().len(), first + 3);
    let review = request_approval(&rig, &proposal);
    assert_eq!(review["data"]["eligible"], false);
    let account = command(
        &mut rig.control.lock().unwrap(),
        "account.get",
        json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]}),
    );
    assert_eq!(account["data"]["health"]["arming"], "DISARMED");
}

#[test]
fn symbol_commission_changes_bind_review_and_preserve_captured_history_without_qualification() {
    if isolated_rule_scenario("symbol_commission_changes_bind_review_and_preserve_captured_history_without_qualification") { return; }
    let rig=RuleRig::new();let external=RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"],"AVAILABLE");
    let proposal=rig.decision();assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["status"],"OBSERVED");
    let captured=risk_evaluate(&rig,&proposal);
    assert_eq!(captured["spotCommission"]["observation"]["standardCommission"]["maker"],"0.00000010","Risk history must carry the symbol-specific declaration: {captured}");
    let first=request_approval(&rig,&proposal);assert_eq!(first["data"]["eligible"],false);
    external.edit.set(Some(|route,body| {if route=="/api/v3/account/commission" {body["standardCommission"]["maker"]=json!("0.00000025");body["discount"]["enabledForSymbol"]=json!(false);}}));
    let changed=commission_refresh(&rig,&proposal,&external);assert_eq!(changed["data"]["status"],"OBSERVED");
    assert!(changed["data"]["observation"]["conditionalDiscountAsset"].is_null());
    let second=request_approval(&rig,&proposal);
    assert_eq!(second["data"]["riskDecision"]["spotCommission"]["observation"]["standardCommission"]["maker"],"0.00000025");
    assert_ne!(first["data"]["reviewDigest"],second["data"]["reviewDigest"]);
    assert_eq!(second["data"]["eligible"],false);
    assert_eq!(second["data"]["riskDecision"]["spotFeeFx"]["feeCurrency"],"UNKNOWN");
    let calls=external.calls.borrow().len();
    let history=command(&mut rig.control.lock().unwrap(),"risk.decision.list",json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]}));
    let saved=history["data"]["decisions"].as_array().unwrap().iter().find(|d|d["decisionId"]==captured["decisionId"]).unwrap();
    assert_eq!(saved["spotCommission"],captured["spotCommission"]);
    assert_eq!(external.calls.borrow().len(),calls,"Captured reads do not call the provider");
}

#[test]
fn symbol_commission_invalid_discount_and_incomplete_declarations_retire_the_previous_source() {
    if isolated_rule_scenario("symbol_commission_invalid_discount_and_incomplete_declarations_retire_the_previous_source") {return;}
    let rig=RuleRig::new();let external=RuleHttp::default();assert_eq!(rig.refresh(&external)["data"]["status"],"AVAILABLE");let proposal=rig.decision();
    assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["status"],"OBSERVED");
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["discount"]["discount"]=json!("1.00000001");}}));
    let failed=commission_refresh(&rig,&proposal,&external);
    assert_eq!(failed["data"]["status"],"UNAVAILABLE","An out-of-range discount is not a valid complete source: {failed}");
    assert_eq!(failed["data"]["failure"],"PROVIDER_RESPONSE_INVALID");
    assert!(failed["data"]["observation"].is_null());
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["taxCommission"].as_object_mut().unwrap().remove("seller");}}));
    let missing=commission_refresh(&rig,&proposal,&external);
    assert_eq!(missing["data"]["status"],"UNAVAILABLE");assert!(missing["data"]["observation"].is_null());
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["standardCommission"]["maker"]=json!(0.001);}}));
    assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["failure"],"PROVIDER_RESPONSE_INVALID");
    external.edit.set(None);
    *external.raw.borrow_mut()=Some(("/api/v3/account/commission".into(),b"{\"symbol\":\"BTCUSDT\",\"symbol\":\"ETHUSDT\"}".to_vec()));
    assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["failure"],"PROVIDER_RESPONSE_INVALID");
    *external.raw.borrow_mut()=None;
    assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["status"],"OBSERVED");
}

#[test]
fn symbol_commission_unknown_active_terms_and_discount_conditions_stay_unqualified() {
    if isolated_rule_scenario("symbol_commission_unknown_active_terms_and_discount_conditions_stay_unqualified") {return;}
    let rig=RuleRig::new();let external=RuleHttp::default();assert_eq!(rig.refresh(&external)["data"]["status"],"AVAILABLE");let proposal=sell_proposal(&rig);
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["specialCommission"]["futureFee"]=json!("private-undeclared-fee-term");}}));
    let source=commission_refresh(&rig,&proposal,&external);
    assert_eq!(source["data"]["status"],"OBSERVED");let observation=&source["data"]["observation"];
    assert_eq!(observation["receivedAsset"],"USDT","SELL receives the quote asset");
    assert_eq!(observation["termsComplete"],false);
    assert_eq!(observation["extensionKeys"],json!(["specialCommission.futureFee"]));
    assert!(observation["unresolvedObligations"].as_array().unwrap().contains(&json!("UNKNOWN_ACTIVE_COMMISSION_FIELDS_UNRESOLVED")));
    assert!(!source.to_string().contains("private-undeclared-fee-term"));
    assert_eq!(observation["chargingAssetQualified"],false);
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["discount"]["enabledForAccount"]=json!(false);}}));
    let disabled=commission_refresh(&rig,&proposal,&external);
    assert_eq!(disabled["data"]["status"],"OBSERVED");assert!(disabled["data"]["observation"]["conditionalDiscountAsset"].is_null());
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["discount"]["discountAsset"]=json!("FUTURETOKEN");}}));
    let unsupported=commission_refresh(&rig,&proposal,&external);assert_eq!(unsupported["data"]["observation"]["termsComplete"],false);
    assert!(unsupported["data"]["observation"]["unresolvedObligations"].as_array().unwrap().contains(&json!("UNSUPPORTED_DISCOUNT_ASSET")));
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["standardCommission"]["x".repeat(65)]=json!("0.1");}}));
    assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["failure"],"PROVIDER_RESPONSE_INVALID");
}

#[test]
fn symbol_commission_identity_and_provider_failures_never_reuse_an_older_success() {
    if isolated_rule_scenario("symbol_commission_identity_and_provider_failures_never_reuse_an_older_success") {return;}
    let rig=RuleRig::new();let external=RuleHttp::default();assert_eq!(rig.refresh(&external)["data"]["status"],"AVAILABLE");let proposal=rig.decision();
    assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["status"],"OBSERVED");
    external.edit.set(Some(|route,body|{if route=="/api/v3/account/commission" {body["symbol"]=json!("ETHUSDT");}}));
    assert_eq!(commission_refresh(&rig,&proposal,&external)["data"]["failure"],"PROVIDER_IDENTITY_CHANGED");
    external.edit.set(None);external.uid.set(1);
    let first=external.calls.borrow().len();let mismatched=commission_refresh(&rig,&proposal,&external);
    assert_eq!(mismatched["data"]["failure"],"PROVIDER_IDENTITY_CHANGED");assert_eq!(external.calls.borrow().len()-first,2,"A wrong UID stops before the symbol read");
    external.uid.set(rig.account["data"]["remoteAccountId"].as_str().unwrap().parse().unwrap());
    for (status,reason) in [(401,"PROVIDER_AUTHENTICATION_FAILED"),(403,"PROVIDER_PERMISSION_BLOCKED"),(500,"PROVIDER_UNAVAILABLE"),(429,"PROVIDER_RATE_LIMITED")] {
        *external.response.borrow_mut()=Some(("/api/v3/account/commission".into(),status,2));
        let result=commission_refresh(&rig,&proposal,&external);assert_eq!(result["data"]["status"],"UNAVAILABLE","{result}");assert_eq!(result["data"]["failure"],reason);assert!(result["data"]["observation"].is_null());
    }
}

#[test]
fn symbol_commission_late_source_generation_and_renderer_authority_are_refused() {
    if isolated_rule_scenario("symbol_commission_late_source_generation_and_renderer_authority_are_refused") {return;}
    let rig=RuleRig::new();let external=RuleHttp::default();assert_eq!(rig.refresh(&external)["data"]["status"],"AVAILABLE");let proposal=rig.decision();
    let before=commission_get(&rig,&proposal);let first=external.calls.borrow().len();
    let forged=tradex::financial_sources::execute_refresh(&rig.control,&json!({"requestId":"forged-fee","schemaVersion":1,"command":"trade.spot_commission.refresh","payload":{"workspaceId":rig.workspace,"proposalId":proposal["proposalId"],"expectedStateVersion":before["data"]["stateVersion"],"qualification":"PASS"}}),"main",&rig.vault,&external);
    assert_eq!(forged["ok"],false);assert_eq!(external.calls.borrow().len(),first);
    let control=rig.control.clone();let workspace=rig.workspace.clone();let account_id=rig.account["connectionId"].clone();
    *external.hook.borrow_mut()=Some(("/api/v3/account/commission".into(),Box::new(move || {
        let mut cp=control.lock().unwrap();let source=command(&mut cp,"data.binance_rules.connection",json!({"workspaceId":workspace}));
        let changed=command(&mut cp,"data.binance_rules.configure",json!({"workspaceId":workspace,"connectionId":account_id,"instrumentId":"crypto:BTC/USDT:spot","expectedStateVersion":source["data"]["stateVersion"]}));assert_eq!(changed["ok"],true);
    })));
    let late=commission_refresh(&rig,&proposal,&external);assert_eq!(late["error"]["code"],"STATE_VERSION_CONFLICT","{late}");
    *external.hook.borrow_mut()=None;
    assert!(commission_get(&rig,&proposal)["data"]["observation"].is_null());
    let unavailable=commission_refresh(&rig,&proposal,&external);assert_eq!(unavailable["error"]["code"],"ORDER_FILTERS_UNAVAILABLE");
}

#[test]
fn symbol_commission_expires_without_renewal_and_captured_history_survives_reopen() {
    if isolated_rule_scenario("symbol_commission_expires_without_renewal_and_captured_history_survives_reopen") {return;}
    let rig=RuleRig::new();let external=RuleHttp::default();assert_eq!(rig.refresh(&external)["data"]["status"],"AVAILABLE");let proposal=rig.decision();
    let collected=commission_refresh(&rig,&proposal,&external);assert_eq!(collected["data"]["status"],"OBSERVED");let captured=risk_evaluate(&rig,&proposal);
    let calls=external.calls.borrow().len();std::thread::sleep(std::time::Duration::from_secs(6));
    let expired=commission_get(&rig,&proposal);assert_eq!(expired["data"]["status"],"STALE");assert!(expired["data"]["observation"].is_null());assert_eq!(external.calls.borrow().len(),calls);
    let query=json!({"workspaceId":rig.workspace,"proposalId":proposal["proposalId"]});
    let RuleRig{_folder,control,..}=rig;drop(control);
    let mut reopened=ControlPlane::new(_folder.path().into());assert_eq!(command(&mut reopened,"workspace.open",json!({}))["ok"],true);
    let history=command(&mut reopened,"risk.decision.list",query);
    let saved=history["data"]["decisions"].as_array().unwrap().iter().find(|d|d["decisionId"]==captured["decisionId"]).unwrap();
    assert_eq!(saved["spotCommission"],captured["spotCommission"]);assert_eq!(external.calls.borrow().len(),calls);
}
