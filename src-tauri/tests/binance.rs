use serde_json::{Value, json};
use tradex::{
    ControlPlane,
    provider_io::{CredentialVault, ProviderHttp},
};
#[path = "support/provider_fixtures.rs"]
mod fixtures;
fn request(command: &str, payload: Value) -> Value {
    json!({"requestId":"binance-test","schemaVersion":1,"command":command,"payload":payload})
}
fn call(cp: &mut ControlPlane, command: &str, payload: Value) -> Value {
    cp.dispatch(request(command, payload))
}
fn mutation(a: &Value) -> Value {
    json!({"workspaceId":a["workspaceId"],"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]})
}
fn run(
    cp: &mut ControlPlane,
    req: Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let result = job.run(
        vault,
        |_| fixtures::credentials(),
        http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, result);
    if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, vault) {
        cp.record_credential_cleanup(&job, cleanup);
    }
    reply
}
fn lifecycle(vault: &impl CredentialVault) {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let http = fixtures::Http::default();
    for env in ["TESTNET", "LIVE"] {
        let schema = call(
            &mut cp,
            "provider.get_schema",
            json!({"providerId":"binance","environment":env}),
        );
        assert_eq!(schema["ok"], true, "{schema}");
        let reply = run(
            &mut cp,
            request(
                "provider.connect",
                json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":env,"label":format!("Spot {env}")}),
            ),
            vault,
            &http,
        );
        assert_eq!(reply["ok"], true, "{reply}");
        let a = &reply["data"];
        assert_eq!(a["data"]["remoteAccountId"], "9007199254740993");
        assert_eq!(a["data"]["currency"], Value::Null);
        assert_eq!(a["data"]["balances"][0]["asset"], "USDT");
        assert_eq!(
            a["data"]["balances"][0]["total"],
            "100000000000000000000.0000000000000000001"
        );
        assert_eq!(a["data"]["positions"][0]["marketValue"], Value::Null);
        assert_eq!(a["data"]["openOrders"].as_array().unwrap().len(), 2);
        assert_ne!(
            a["data"]["openOrders"][0]["brokerOrderId"],
            a["data"]["openOrders"][1]["brokerOrderId"]
        );
        assert_eq!(
            a["permissions"]["scope"],
            if env == "LIVE" {
                "VERIFIED"
            } else {
                "UNVERIFIED"
            }
        );
        assert_eq!(
            a["permissions"]["forbidden"],
            json!([]),
            "account canWithdraw must not become key scope"
        );
        assert_eq!(a["health"]["executionEligibility"], "BLOCKED");
        if env == "LIVE" {
            assert_eq!(a["health"]["arming"], "DISARMED");
        }
        let mut confirm = mutation(a);
        confirm["step"] = "confirm".into();
        confirm["acknowledgeUnverified"] = true.into();
        let accepted = call(&mut cp, "provider.connect", confirm);
        assert_eq!(accepted["ok"], true, "{accepted}");
        let disconnected = run(
            &mut cp,
            request("provider.disconnect", mutation(&accepted["data"])),
            vault,
            &http,
        );
        assert_eq!(disconnected["data"]["health"]["credential"], "MISSING");
    }
}

#[test]
fn spot_connections_keep_native_assets_scope_and_environment_separate() {
    lifecycle(&fixtures::Vault::default());
}
#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit OS Keychain check with disposable Binance credentials"]
fn native_keychain_stores_and_removes_spot_credentials() {
    lifecycle(&tradex::provider_io::NativeVault);
}

#[derive(Default)]
struct Response {
    route: &'static str,
    body: Option<Value>,
    error: Option<&'static str>,
    delay_time: bool,
    calls: std::cell::RefCell<Vec<String>>,
}
impl ProviderHttp for Response {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        let route = path.split('?').next().unwrap();
        self.calls.borrow_mut().push(route.into());
        if route == self.route {
            if self.delay_time {
                std::thread::sleep(std::time::Duration::from_millis(2050));
            }
            if let Some(code) = self.error {
                return Err(tradex::protocol::TradeXError::new(code));
            }
            if let Some(body) = &self.body {
                return Ok(serde_json::to_vec(body).unwrap());
            }
        }
        fixtures::Http::default().get(endpoint, path, headers)
    }
}
fn restrictions() -> Value {
    json!({"ipRestrict":true,"enableReading":true,"enableWithdrawals":false,"enableInternalTransfer":false,"permitsUniversalTransfer":false,"enableMargin":false,"enableFutures":false,"enableVanillaOptions":false,"enablePortfolioMarginTrading":false,"enableFixApiTrade":false,"enableFixReadOnly":false,"enableSpotAndMarginTrading":true})
}
#[test]
fn scope_changes_and_forbidden_permissions_cannot_be_acknowledged_away() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = fixtures::Vault::default();
    let mut a=run(&mut cp,request("provider.connect",json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":"LIVE","label":"Permission gate"})),&vault,&fixtures::Http::default())["data"].clone();
    for flag in [
        "enableWithdrawals",
        "enableInternalTransfer",
        "permitsUniversalTransfer",
        "enableMargin",
        "enableFutures",
        "enableVanillaOptions",
        "enablePortfolioMarginTrading",
        "enableFixApiTrade",
        "enableReading",
    ] {
        let mut confirm = mutation(&a);
        confirm["step"] = "confirm".into();
        confirm["acknowledgeUnverified"] = true.into();
        let connected = call(&mut cp, "provider.connect", confirm);
        assert_eq!(connected["ok"], true, "{connected}");
        a = connected["data"].clone();
        let mut changed = restrictions();
        changed[flag] = (flag != "enableReading").into();
        // Incomplete inspection still retains every observed dangerous authority.
        changed.as_object_mut().unwrap().remove("enableFixReadOnly");
        let http = Response {
            route: "/sapi/v1/account/apiRestrictions",
            body: Some(changed),
            ..Default::default()
        };
        let probe = run(
            &mut cp,
            request("provider.probe", mutation(&a)),
            &vault,
            &http,
        );
        assert_eq!(probe["ok"], true, "{probe}");
        a = probe["data"].clone();
        assert_eq!(a["connectionState"], "REVIEW_REQUIRED");
        assert_eq!(a["permissions"]["scope"], "UNVERIFIED");
        assert_eq!(a["permissions"]["acknowledged"], false);
        let mut confirm = mutation(&a);
        confirm["step"] = "confirm".into();
        confirm["acknowledgeUnverified"] = true.into();
        assert_eq!(
            call(&mut cp, "provider.connect", confirm)["error"]["code"],
            "PROVIDER_PERMISSION_BLOCKED",
            "{flag}"
        );
        let recovered = run(
            &mut cp,
            request("provider.probe", mutation(&a)),
            &vault,
            &fixtures::Http::default(),
        );
        a = recovered["data"].clone();
    }
    let mut changed = restrictions();
    changed["futureUnknownPermission"] = true.into();
    changed["ipRestrict"] = false.into();
    let probe = run(
        &mut cp,
        request("provider.probe", mutation(&a)),
        &vault,
        &Response {
            route: "/sapi/v1/account/apiRestrictions",
            body: Some(changed),
            ..Default::default()
        },
    );
    assert_eq!(probe["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(
        probe["data"]["permissions"]["ipAllowListStatus"],
        "UNRESTRICTED"
    );
    let disconnected = run(
        &mut cp,
        request("provider.disconnect", mutation(&probe["data"])),
        &vault,
        &fixtures::Http::default(),
    );
    assert_eq!(disconnected["ok"], true);
}
#[test]
fn time_identity_and_incomplete_observations_fail_without_replacing_saved_data() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = fixtures::Vault::default();
    let original=run(&mut cp,request("provider.connect",json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":"LIVE","label":"Faults"})),&vault,&fixtures::Http::default())["data"].clone();
    let mut a = original.clone();
    for (route, body, code) in [
        (
            "/api/v3/account",
            json!({"uid":9007199254740993u64,"accountType":"SPOT","canTrade":true,"canWithdraw":true,"canDeposit":true,"balances":[{"asset":"BTC","free":"-0.01","locked":"0"}]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/account",
            json!({"uid":9007199254740993u64,"accountType":"SPOT","canTrade":true,"canWithdraw":true,"canDeposit":true,"balances":[{"asset":"BTC","free":"1","locked":"0"},{"asset":"BTC","free":"2","locked":"0"}]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/openOrders",
            json!([{"symbol":"BTCUSDT","orderId":1,"side":"BUY","status":"NEW","price":"2","origQty":"1","origQuoteOrderQty":"0","executedQty":"0"},{"symbol":"BTCUSDT","orderId":1,"side":"BUY","status":"NEW","price":"2","origQty":"1","origQuoteOrderQty":"0","executedQty":"0"}]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        ("/api/v3/time", json!({"serverTime":-1}), "CLOCK_SKEW"),
        (
            "/api/v3/time",
            json!({"serverTime":1788849600000000u64}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v3/time",
            json!({"serverTime":"1788849600000"}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v3/account",
            json!({"uid":123}),
            "PROVIDER_IDENTITY_CHANGED",
        ),
        (
            "/api/v3/account",
            json!({"uid":9007199254740993u64,"accountType":"MARGIN"}),
            "PROVIDER_UNSUPPORTED",
        ),
        (
            "/api/v3/openOrders",
            json!({"items":[],"next":"https://example.invalid"}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/openOrders",
            json!([{"symbol":fixtures::SECRET}]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/sapi/v1/account/apiRestrictions",
            json!({"enableWithdrawals":"false"}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/openOrders",
            Value::Array(vec![json!({}); 10001]),
            "PROVIDER_DATA_INCOMPLETE",
        ),
    ] {
        let http = Response {
            route,
            body: Some(body),
            ..Default::default()
        };
        let reply = run(
            &mut cp,
            request("account.refresh", mutation(&a)),
            &vault,
            &http,
        );
        assert_eq!(reply["error"]["code"], code, "{reply}");
        if route == "/api/v3/time" {
            assert_eq!(http.calls.borrow().len(), 1);
            assert_eq!(reply["error"]["category"], "STATE_STALE");
        }
        if code == "PROVIDER_IDENTITY_CHANGED" {
            assert_eq!(http.calls.borrow().len(), 2);
        }
        a = call(
            &mut cp,
            "account.get",
            json!({"workspaceId":ws,"connectionId":a["connectionId"]}),
        )["data"]
            .clone();
        assert_eq!(a["data"], original["data"]);
        assert_eq!(a["lastSuccessfulSync"], original["lastSuccessfulSync"]);
    }
    for (route, error, delay_time) in [
        ("/api/v3/time", None, true),
        ("/api/v3/account", Some("CLOCK_SKEW"), false),
        (
            "/sapi/v1/account/apiRestrictions",
            Some("PROVIDER_RATE_LIMITED"),
            false,
        ),
        (
            "/sapi/v1/account/apiRestrictions",
            Some("PROVIDER_AUTH_FAILED"),
            false,
        ),
    ] {
        let http = Response {
            route,
            error,
            delay_time,
            ..Default::default()
        };
        let reply = run(
            &mut cp,
            request("provider.probe", mutation(&a)),
            &vault,
            &http,
        );
        assert_eq!(
            reply["error"]["code"],
            error.unwrap_or("CLOCK_SKEW"),
            "{reply}"
        );
        a = call(
            &mut cp,
            "account.get",
            json!({"workspaceId":ws,"connectionId":a["connectionId"]}),
        )["data"]
            .clone();
        assert_eq!(a["data"], original["data"]);
        assert_eq!(a["lastSuccessfulSync"], original["lastSuccessfulSync"]);
    }
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().into());
    call(&mut cp, "workspace.open", json!({}));
    a = call(
        &mut cp,
        "account.get",
        json!({"workspaceId":ws,"connectionId":a["connectionId"]}),
    )["data"]
        .clone();
    assert_eq!(a["health"]["connection"], "STALE");
    let refreshed = run(
        &mut cp,
        request("provider.probe", mutation(&a)),
        &vault,
        &fixtures::Http::default(),
    );
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    let disconnected = run(
        &mut cp,
        request("provider.disconnect", mutation(&refreshed["data"])),
        &vault,
        &fixtures::Http::default(),
    );
    assert_eq!(disconnected["ok"], true);
    for entry in std::fs::read_dir(folder.path()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let bytes = std::fs::read(path).unwrap();
            for secret in [fixtures::KEY, fixtures::SECRET] {
                assert!(!bytes.windows(secret.len()).any(|b| b == secret.as_bytes()));
            }
        }
    }
}

#[test]
fn invalidation_during_server_time_prevents_any_signed_request_and_cleans_new_key() {
    struct Invalidated(std::cell::Cell<bool>);
    impl ProviderHttp for Invalidated {
        fn get(
            &self,
            endpoint: tradex::provider_io::ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            assert_eq!(
                path, "/api/v3/time",
                "No signed read may follow an invalidated job"
            );
            let response = fixtures::Http::default().get(endpoint, path, headers);
            self.0.set(true);
            response
        }
    }
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":"TESTNET","label":"Invalidated time sample"}),
    );
    let vault = fixtures::Vault::default();
    let http = Invalidated(std::cell::Cell::new(false));
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let reference = job.connection().credential_ref();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || !http.0.get() && cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["error"]["code"], "STATE_VERSION_CONFLICT");
    assert!(vault.get(&reference).is_err());
}
