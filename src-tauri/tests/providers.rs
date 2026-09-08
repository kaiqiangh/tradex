use serde_json::{Value, json};
use tradex::ControlPlane;
use tradex::protocol::Result;
use tradex::provider_io::{CredentialVault, ProviderHttp};

#[path = "support/provider_fixtures.rs"]
mod fixtures;
use fixtures::*;

fn envelope(command: &str, payload: Value) -> Value {
    json!({"requestId":"provider-test","schemaVersion":1,"command":command,"payload":payload})
}
fn begin(workspace: &Value) -> Value {
    envelope(
        "provider.connect",
        json!({"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Paper research"}),
    )
}
fn query(account: &Value) -> Value {
    json!({"workspaceId":account["workspaceId"],"connectionId":account["connectionId"]})
}
fn mutation(account: &Value) -> Value {
    let mut p = query(account);
    p["expectedStateVersion"] = account["stateVersion"].clone();
    p
}
fn execute(
    cp: &mut ControlPlane,
    input: Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let job = cp.prepare_provider(&input).unwrap().unwrap();
    let outcome = job.run(
        vault,
        |_| credentials(),
        http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, vault) {
        cp.record_credential_cleanup(&job, cleanup);
    }
    reply
}

fn command(cp: &mut ControlPlane, command: &str, payload: Value) -> Value {
    cp.dispatch(
        json!({"requestId":"provider-test","schemaVersion":1,"command":command,"payload":payload}),
    )
}

fn lifecycle(vault: &impl CredentialVault) {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let http = Http::default();
    let connected = execute(&mut cp, begin(&workspace), vault, &http);
    assert_eq!(connected["ok"], true, "{connected}");
    let pending = connected["data"].clone();
    assert_eq!(pending["connectionState"], "REVIEW_REQUIRED");
    assert_eq!(pending["data"]["balances"][0]["available"], "1000.25");
    assert_eq!(pending["data"]["openOrders"][0]["notional"], "10.5");
    assert_eq!(pending["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(pending["health"]["privateStream"], "NOT_CONFIGURED");
    assert_eq!(pending["health"]["executionEligibility"], "BLOCKED");
    let mut confirm = mutation(&pending);
    confirm["step"] = "confirm".into();
    confirm["acknowledgeUnverified"] = false.into();
    assert_eq!(
        command(&mut cp, "provider.connect", confirm.clone())["error"]["code"],
        "PROVIDER_REVIEW_REQUIRED"
    );
    confirm["acknowledgeUnverified"] = true.into();
    let confirmed = command(&mut cp, "provider.connect", confirm.clone());
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    assert_eq!(confirmed["data"]["connectionState"], "CONNECTED");
    assert_eq!(
        command(&mut cp, "provider.connect", confirm)["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );

    let snapshot = command(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"account","aggregateId":pending["connectionId"]}),
    );
    assert_eq!(snapshot["data"]["lastSequence"], 3);
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = events.clone();
    let ack=cp.dispatch_with_events(envelope("domain.subscribe",json!({"aggregateType":"account","aggregateId":pending["connectionId"],"afterSequence":0})),"test-window",Some(std::sync::Arc::new(move|event|{sink.lock().unwrap().push(serde_json::to_value(event).unwrap());true})));
    assert_eq!(ack["data"]["replayedCount"], 3);
    assert_eq!(events.lock().unwrap()[2]["payload"], confirmed["data"]);

    http.fail.set(true);
    let failed = execute(
        &mut cp,
        envelope("account.refresh", mutation(&confirmed["data"])),
        vault,
        &http,
    );
    assert_eq!(failed["error"]["code"], "PROVIDER_UNAVAILABLE");
    let stale = command(&mut cp, "account.get", query(&pending))["data"].clone();
    assert_eq!(stale["data"], pending["data"]);
    assert_eq!(stale["lastSuccessfulSync"], pending["lastSuccessfulSync"]);
    assert_eq!(stale["health"]["connection"], "ERROR");
    assert_eq!(events.lock().unwrap().last().unwrap()["payload"], stale);
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    command(&mut cp, "workspace.open", json!({}));
    let restored = command(&mut cp, "account.get", query(&pending))["data"].clone();
    assert_eq!(restored["connectionId"], pending["connectionId"]);
    assert_eq!(restored["health"]["connection"], "STALE");
    assert_eq!(restored["health"]["credential"], "UNCHECKED");
    http.fail.set(false);
    let refreshed = execute(
        &mut cp,
        envelope("provider.probe", mutation(&restored)),
        vault,
        &http,
    );
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    let disconnected = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&refreshed["data"])),
        vault,
        &http,
    );
    assert_eq!(disconnected["data"]["connectionState"], "DISCONNECTED");
    assert_eq!(disconnected["data"]["health"]["credential"], "MISSING");
    let reference = serde_json::from_value::<tradex::providers::AccountConnection>(pending.clone())
        .unwrap()
        .credential_ref();
    assert!(
        vault.get(&reference).is_err(),
        "Disconnect must remove the actual Keychain item"
    );
    let serialized = serde_json::to_vec(&json!([
        pending,
        confirmed,
        stale,
        restored,
        refreshed,
        disconnected,
        *events.lock().unwrap()
    ]))
    .unwrap();
    for secret in [KEY, SECRET] {
        assert!(
            !serialized
                .windows(secret.len())
                .any(|w| w == secret.as_bytes())
        );
        for entry in std::fs::read_dir(folder.path()).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }
            let bytes = std::fs::read(path).unwrap();
            assert!(!bytes.windows(secret.len()).any(|w| w == secret.as_bytes()));
        }
    }
}

#[test]
fn public_connection_lifecycle_preserves_scope_events_decimal_values_and_secret_boundaries() {
    lifecycle(&Vault::default());
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit native OS Keychain integration; creates and deletes disposable synthetic credentials"]
fn native_keychain_roundtrip_drives_the_real_connection_lifecycle() {
    lifecycle(&tradex::provider_io::NativeVault);
}

#[test]
fn disconnected_or_switched_workspaces_reject_late_results_and_cleanup_can_be_retried() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().join("one"));
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let job = cp.prepare_provider(&begin(&workspace)).unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let pending = serde_json::to_value(job.connection()).unwrap();
    vault.fail_remove.set(true);
    let disconnected = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&pending)),
        &vault,
        &http,
    );
    assert_eq!(
        disconnected["data"]["health"]["credential"],
        "DELETE_PENDING"
    );
    assert_eq!(
        cp.complete_provider(&job, outcome)["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    vault.fail_remove.set(false);
    let removed = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&disconnected["data"])),
        &vault,
        &http,
    );
    assert_eq!(removed["data"]["health"]["credential"], "MISSING");
    assert!(vault.present.borrow().is_empty());

    let late = cp.prepare_provider(&begin(&workspace)).unwrap().unwrap();
    let outcome = late.run(&vault, |_| credentials(), &http, || true);
    command(
        &mut cp,
        "workspace.open",
        json!({"path":folder.path().join("two")}),
    );
    assert_eq!(
        command(&mut cp, "runtime.status", json!({}))["data"]["modelAvailable"],
        false
    );
    let result = cp.complete_provider(&late, outcome);
    assert_eq!(result["error"]["code"], "STATE_VERSION_CONFLICT");
    if let Some(cleanup) = late.cleanup_after_failed_commit(&result, &vault) {
        cp.record_credential_cleanup(&late, cleanup);
    }
    assert!(vault.present.borrow().is_empty());
}

#[test]
fn duplicate_and_interrupted_connections_keep_owned_credentials_recoverable() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let first = execute(&mut cp, begin(&workspace), &vault, &http)["data"].clone();
    let duplicate = execute(&mut cp, begin(&workspace), &vault, &http);
    assert_eq!(duplicate["error"]["code"], "PROVIDER_ALREADY_CONNECTED");
    assert_eq!(
        command(&mut cp, "account.get", query(&first))["data"],
        first
    );
    assert_eq!(vault.present.borrow().len(), 1);
    let list = command(&mut cp, "account.list", json!({"workspaceId":workspace}));
    let abandoned = list["data"]["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["connectionId"] != first["connectionId"])
        .unwrap();
    assert_eq!(abandoned["connectionState"], "DISCONNECTED");
    assert_eq!(abandoned["health"]["credential"], "MISSING");

    let interrupted = cp.prepare_provider(&begin(&workspace)).unwrap().unwrap();
    let _outcome = interrupted.run(&vault, |_| credentials(), &http, || true);
    let pending = serde_json::to_value(interrupted.connection()).unwrap();
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    command(&mut cp, "workspace.open", json!({}));
    let restored = command(&mut cp, "account.get", query(&pending))["data"].clone();
    assert_eq!(restored["connectionState"], "DISCONNECTED");
    assert_eq!(restored["health"]["credential"], "DELETE_PENDING");
    assert!(
        cp.prepare_provider(&envelope("provider.probe", mutation(&restored)))
            .is_err()
    );
    let removed = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&restored)),
        &vault,
        &http,
    );
    assert_eq!(removed["data"]["health"]["credential"], "MISSING");
    assert_eq!(
        vault.present.borrow().len(),
        1,
        "Cleanup must retain the original connection's credential"
    );
}

struct Override {
    base: Http,
    path: &'static str,
    body: Vec<u8>,
}

struct Fault(&'static str);
impl ProviderHttp for Fault {
    fn get(
        &self,
        _: tradex::provider_io::ProviderEndpoint,
        _: &str,
        _: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        Err(tradex::protocol::TradeXError::new(self.0))
    }
}

#[test]
fn failed_initial_tests_remove_their_credential_and_keep_failed_cleanup_blocked_after_restart() {
    for code in [
        "PROVIDER_AUTH_FAILED",
        "PROVIDER_UNAVAILABLE",
        "PROVIDER_RATE_LIMITED",
    ] {
        for cleanup_fails in [false, true] {
            let folder = tempfile::tempdir().unwrap();
            let mut cp = ControlPlane::new(folder.path().to_path_buf());
            let workspace =
                command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
            let vault = Vault::default();
            vault.fail_remove.set(cleanup_fails);
            let result = execute(&mut cp, begin(&workspace), &vault, &Fault(code));
            assert_eq!(result["error"]["code"], code);
            let failed = command(&mut cp, "account.list", json!({"workspaceId":workspace}))["data"]
                ["accounts"][0]
                .clone();
            assert_eq!(failed["connectionState"], "FAILED");
            assert!(failed["lastSuccessfulSync"].is_null());
            assert!(failed["data"].is_null());
            assert_eq!(
                failed["health"]["credential"],
                if cleanup_fails {
                    "DELETE_PENDING"
                } else {
                    "MISSING"
                }
            );
            assert!(
                cp.prepare_provider(&envelope("provider.probe", mutation(&failed)))
                    .is_err()
            );
            drop(cp);
            let mut cp = ControlPlane::new(folder.path().to_path_buf());
            command(&mut cp, "workspace.open", json!({}));
            let restored = command(&mut cp, "account.get", query(&failed))["data"].clone();
            assert_eq!(
                restored["health"]["credential"],
                failed["health"]["credential"]
            );
            assert!(
                cp.prepare_provider(&envelope("provider.probe", mutation(&restored)))
                    .is_err()
            );
            vault.fail_remove.set(false);
            let cleaned = execute(
                &mut cp,
                envelope("provider.disconnect", mutation(&restored)),
                &vault,
                &Http::default(),
            );
            assert_eq!(cleaned["data"]["health"]["credential"], "MISSING");
            assert!(vault.present.borrow().is_empty());
        }
    }
}
impl ProviderHttp for Override {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        let normal = self.base.get(
            endpoint,
            if path.contains("&after_order_id=") {
                "/v2/orders?status=open&limit=500&direction=asc&nested=false"
            } else {
                path
            },
            headers,
        )?;
        Ok(if path == self.path {
            self.body.clone()
        } else {
            normal
        })
    }
}

#[test]
fn provider_faults_cannot_publish_unvalidated_or_cross_account_observations() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let result = execute(&mut cp, begin(&workspace), &vault, &Http::default());
    let original = result["data"].clone();
    let mut current = original.clone();
    let wrong = Http {
        identity: "04d1f763-caaa-43d0-a06f-cb20dd65d145".into(),
        ..Http::default()
    };
    let result = execute(
        &mut cp,
        envelope("account.refresh", mutation(&current)),
        &vault,
        &wrong,
    );
    assert_eq!(result["error"]["code"], "PROVIDER_IDENTITY_CHANGED");
    assert_eq!(
        wrong.calls.borrow().len(),
        1,
        "Stop reading when identity differs"
    );
    current = command(&mut cp, "account.get", query(&current))["data"].clone();
    for body in [
        b"not json".to_vec(),
        vec![b' '; 2 * 1024 * 1024 + 1],
        format!("{{\"reflected\":\"{SECRET}\"}}").into_bytes(),
        format!(
            "[{{\"symbol\":\"{}\",\"qty\":\"1\"}}]",
            KEY.replace('-', "\\u002d")
        )
        .into_bytes(),
        serde_json::to_vec(&json!([{"symbol":"AAPL","qty":"NaN"}])).unwrap(),
    ] {
        let http = Override {
            base: Http::default(),
            path: "/v2/positions",
            body,
        };
        let result = execute(
            &mut cp,
            envelope("provider.probe", mutation(&current)),
            &vault,
            &http,
        );
        assert_eq!(
            result["error"]["code"], "PROVIDER_RESPONSE_INVALID",
            "{result}"
        );
        current = command(&mut cp, "account.get", query(&current))["data"].clone();
        assert_eq!(current["data"], original["data"]);
        assert_eq!(
            current["lastSuccessfulSync"],
            original["lastSuccessfulSync"]
        );
        assert_ne!(current["health"]["connection"], "ONLINE");
    }
    let successful = execute(
        &mut cp,
        envelope("provider.probe", mutation(&current)),
        &vault,
        &Http::default(),
    );
    assert_eq!(successful["ok"], true);
    let job = cp
        .prepare_provider(&envelope("account.refresh", mutation(&successful["data"])))
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &Http::default(),
        || true,
    );
    let fault = rusqlite::Connection::open(folder.path().join("workspace.sqlite3")).unwrap();
    fault.execute_batch("CREATE TRIGGER fail_account_event BEFORE INSERT ON outbox WHEN NEW.aggregate_type='account' BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    let failed = cp.complete_provider(&job, outcome);
    assert_eq!(failed["error"]["code"], "WORKSPACE_OPEN_FAILED");
    assert_eq!(
        command(&mut cp, "account.get", query(&current))["data"],
        successful["data"],
        "Projection and outbox roll back together"
    );
    fault
        .execute_batch("DROP TRIGGER fail_account_event")
        .unwrap();
    let reference = job.connection().credential_ref();
    vault.remove(&reference).unwrap();
    let missing = execute(
        &mut cp,
        envelope("provider.probe", mutation(&successful["data"])),
        &vault,
        &Http::default(),
    );
    assert_eq!(missing["error"]["code"], "CREDENTIAL_UNAVAILABLE");
}

struct Pages {
    base: Http,
    repeated: bool,
    calls: std::cell::Cell<usize>,
}
impl ProviderHttp for Pages {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        if !path.starts_with("/v2/orders?") {
            return self.base.get(endpoint, path, headers);
        }
        self.calls.set(self.calls.get() + 1);
        let start = if path.contains("&after_order_id=") {
            assert!(path.ends_with("00000000-0000-4000-8000-000000000500"));
            if self.repeated { 0 } else { 500 }
        } else {
            0
        };
        let count = if start == 0 { 500 } else { 1 };
        let orders=(start+1..start+count+1).map(|n|json!({"id":format!("00000000-0000-4000-8000-{n:012}"),"symbol":"AAPL","side":"buy","qty":"1","filled_qty":"0","status":"new"})).collect::<Vec<_>>();
        Ok(serde_json::to_vec(&orders).unwrap())
    }
}

#[test]
fn order_id_pagination_is_complete_and_repeated_pages_fail_closed() {
    for repeated in [false, true] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let http = Pages {
            base: Http::default(),
            repeated,
            calls: std::cell::Cell::new(0),
        };
        let result = execute(&mut cp, begin(&workspace), &Vault::default(), &http);
        assert_eq!(http.calls.get(), 2);
        if repeated {
            assert_eq!(result["error"]["code"], "PROVIDER_DATA_INCOMPLETE");
        } else {
            assert_eq!(
                result["data"]["data"]["openOrders"]
                    .as_array()
                    .unwrap()
                    .len(),
                501
            );
        }
    }
    let http = tradex::provider_io::BrokerHttp::default();
    assert_eq!(
        http.get(
            tradex::provider_io::ProviderEndpoint::AlpacaPaper,
            "https://example.invalid/",
            reqwest::header::HeaderMap::new()
        )
        .unwrap_err()
        .code,
        "PROVIDER_UNSUPPORTED"
    );
    for value in [
        json!(1.2),
        json!("1e3"),
        json!("NaN"),
        json!("1."),
        json!("+1"),
        json!("1.2.3"),
    ] {
        assert!(tradex::provider_io::decimal(&value).is_err());
    }
    assert_eq!(
        tradex::provider_io::decimal(&json!("-000.000")).unwrap(),
        "0"
    );
}

#[test]
fn provider_schema_precedes_native_entry_and_never_accepts_renderer_secrets() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let schema = command(
        &mut cp,
        "provider.get_schema",
        json!({"providerId":"alpaca","environment":"PAPER"}),
    );
    assert_eq!(schema["ok"], true, "{schema}");
    let fields = schema["data"]["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 2);
    assert!(
        fields
            .iter()
            .all(|f| f["secret"] == true && f["required"] == true)
    );
    assert_eq!(
        command(
            &mut cp,
            "provider.get_schema",
            json!({"providerId":"alpaca","environment":"LIVE"})
        )["error"]["code"],
        "PROVIDER_UNSUPPORTED"
    );
    let mut input = json!({"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Paper research"});
    input["apiKey"] = "synthetic-renderer-secret-must-be-rejected".into();
    assert_eq!(
        command(&mut cp, "provider.connect", input)["error"]["code"],
        "IPC_PAYLOAD_INVALID"
    );
    assert_eq!(
        command(&mut cp, "account.list", json!({"workspaceId":workspace}))["data"]["accounts"],
        json!([])
    );
}
