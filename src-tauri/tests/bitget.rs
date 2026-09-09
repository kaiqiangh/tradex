use serde_json::{Value, json};
use std::cell::RefCell;
use tradex::{
    ControlPlane,
    protocol::{Result, TradeXError},
    provider_io::{CredentialVault, Credentials, ProviderEndpoint, ProviderHttp},
};

#[path = "support/bitget_fixtures.rs"]
mod bitget_fixtures;
use bitget_fixtures::{Http as LivePages, KEY, PASSPHRASE, SECRET, credentials};
#[derive(Default)]
struct Vault(RefCell<Option<String>>);
impl CredentialVault for Vault {
    fn put(&self, reference: &str, _: &Credentials) -> Result<()> {
        assert!(self.0.borrow().is_none());
        *self.0.borrow_mut() = Some(reference.into());
        Ok(())
    }
    fn get(&self, reference: &str) -> Result<Credentials> {
        if self.0.borrow().as_deref() != Some(reference) {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        credentials()
    }
    fn remove(&self, reference: &str) -> Result<()> {
        if self.0.borrow().as_deref() == Some(reference) {
            self.0.borrow_mut().take();
        }
        Ok(())
    }
}
struct DemoUnsupported(RefCell<Vec<String>>);
impl ProviderHttp for DemoUnsupported {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        assert_eq!(endpoint.base_url(), "https://api.bitget.com");
        self.0.borrow_mut().push(path.into());
        if path == "/api/v2/public/time" {
            assert!(headers.is_empty());
            return Ok(br#"{"code":"00000","data":{"serverTime":"1788849600000"}}"#.to_vec());
        }
        assert_eq!(path, "/api/v2/spot/account/info");
        assert_eq!(headers["paptrading"], "1");
        assert_eq!(headers["ACCESS-KEY"], KEY);
        assert_eq!(headers["ACCESS-PASSPHRASE"], PASSPHRASE);
        for name in ["ACCESS-KEY", "ACCESS-PASSPHRASE", "ACCESS-SIGN"] {
            assert!(headers[name].is_sensitive());
        }
        let timestamp = headers["ACCESS-TIMESTAMP"].to_str().unwrap();
        assert!((1788849600000..1788849660000).contains(&timestamp.parse::<u64>().unwrap()));
        use base64::Engine;
        use hmac::Mac;
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(SECRET.as_bytes()).unwrap();
        mac.update(format!("{timestamp}GET{path}").as_bytes());
        mac.verify_slice(
            &base64::engine::general_purpose::STANDARD
                .decode(headers["ACCESS-SIGN"].as_bytes())
                .unwrap(),
        )
        .unwrap();
        Ok(
            br#"{"code":"40081","msg":"synthetic Demo account endpoint restriction","data":null}"#
                .to_vec(),
        )
    }
}
fn request(command: &str, payload: Value) -> Value {
    json!({"requestId":"bitget-test","schemaVersion":1,"command":command,"payload":payload})
}
#[test]
fn demo_account_rejection_keeps_three_secrets_native_and_never_falls_back_to_live() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let schema = cp.dispatch(request(
        "provider.get_schema",
        json!({"providerId":"bitget","environment":"DEMO"}),
    ));
    assert_eq!(schema["ok"], true, "{schema}");
    let fields = schema["data"]["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 3);
    assert_eq!(
        fields
            .iter()
            .map(|f| f["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["apiKey", "secret", "passphrase"]
    );
    assert!(
        fields
            .iter()
            .all(|f| f["secret"] == true && f["required"] == true)
    );
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"DEMO","label":"Disposable Demo rejection"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let vault = Vault::default();
    let http = DemoUnsupported(RefCell::new(vec![]));
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, &vault) {
        cp.record_credential_cleanup(&job, cleanup);
    }
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(reply["error"]["code"], "PROVIDER_UNSUPPORTED", "{reply}");
    assert!(vault.0.borrow().is_none());
    assert_eq!(
        *http.0.borrow(),
        ["/api/v2/public/time", "/api/v2/spot/account/info"]
    );
    let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
    let account = &list["data"]["accounts"][0];
    assert_eq!(account["environment"], "DEMO");
    assert_eq!(account["connectionState"], "FAILED");
    assert_eq!(account["health"]["credential"], "MISSING");
    assert!(account["data"].is_null());
    assert!(account["lastSuccessfulSync"].is_null());
    for secret in [KEY, SECRET, PASSPHRASE] {
        assert!(!format!("{schema}{reply}{list}").contains(secret));
    }
}

fn live_lifecycle(vault: &impl CredentialVault) {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Live contract fixture"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let http = LivePages(RefCell::new(vec![]));
    let outcome = job.run(
        vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    let a = &reply["data"];
    assert_eq!(a["permissions"]["scope"], "VERIFIED");
    assert_eq!(a["health"]["arming"], "DISARMED");
    assert_eq!(a["health"]["executionEligibility"], "BLOCKED");
    assert_eq!(a["data"]["balances"][0]["total"], "1000000000000000002");
    assert_eq!(a["data"]["balances"][0]["locked"], "2");
    assert_eq!(a["data"]["balances"][0]["restrictedAvailable"], "7");
    let orders = a["data"]["openOrders"].as_array().unwrap();
    assert_eq!(orders.len(), 103);
    assert_eq!(orders[100]["quantity"], "0.1234567890123456789");
    assert!(orders[100]["notional"].is_null());
    assert_eq!(orders[101]["kind"], "TPSL");
    assert!(orders[101]["quantity"].is_null());
    assert_eq!(orders[101]["notional"], "0.1234567890123456789");
    assert_eq!(orders[101]["triggerPrice"], "12000");
    assert_eq!(orders[102]["kind"], "PLAN");
    assert_eq!(orders[102]["notional"], "20");
    assert_ne!(orders[0]["brokerOrderId"], orders[102]["brokerOrderId"]);
    let confirm=cp.dispatch(request("provider.connect",json!({"step":"confirm","workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"],"acknowledgeUnverified":false})));
    assert_eq!(confirm["ok"], true, "{confirm}");
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().into());
    assert_eq!(
        cp.dispatch(request("workspace.open", json!({})))["ok"],
        true
    );
    let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
    let a = &list["data"]["accounts"][0];
    assert_eq!(a["data"]["openOrders"].as_array().unwrap().len(), 103);
    let req = request(
        "provider.disconnect",
        json!({"workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let outcome = job.run(
        vault,
        |_| panic!("Disconnect must not capture credentials"),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["health"]["credential"], "MISSING");
}

#[test]
fn live_connection_preserves_all_order_pages_asset_buckets_and_review() {
    let vault = Vault::default();
    live_lifecycle(&vault);
    assert!(vault.0.borrow().is_none());
}
#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit native Keychain integration with three disposable Bitget credential fields"]
fn native_keychain_stores_and_removes_three_bitget_fields() {
    live_lifecycle(&tradex::provider_io::NativeVault);
}

struct ChangedResponse {
    path: &'static str,
    data: Value,
}
impl ProviderHttp for ChangedResponse {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        let bytes = LivePages(RefCell::new(vec![])).get(endpoint, path, headers)?;
        if path == self.path {
            Ok(serde_json::to_vec(&json!({"code":"00000","data":self.data})).unwrap())
        } else {
            Ok(bytes)
        }
    }
}
#[test]
fn dangerous_or_unknown_scope_and_failed_refresh_do_not_gain_authority() {
    for authority in [
        "wtow",
        "wwow",
        "chow",
        "coow",
        "cpow",
        "smow",
        "ttow",
        "p2p",
        "pllw",
        "taxw",
        "future_permission",
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let req = request(
            "provider.connect",
            json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Scope fixture"}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let vault = Vault::default();
        let http = ChangedResponse {
            path: "/api/v2/spot/account/info",
            data: json!({"userId":"9007199254740993","ips":"127.0.0.1","authorities":["stor",authority]}),
        };
        let outcome = job.run(
            &vault,
            |_| credentials(),
            &http,
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, outcome);
        assert_eq!(reply["ok"], true, "{reply}");
        let a = &reply["data"];
        let req = request(
            "provider.connect",
            json!({"step":"confirm","workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"],"acknowledgeUnverified":true}),
        );
        let confirmed = cp.dispatch(req);
        assert_eq!(
            confirmed["ok"],
            authority == "future_permission",
            "{authority}: {confirmed}"
        );
        if authority == "future_permission" {
            assert_eq!(confirmed["data"]["permissions"]["scope"], "UNVERIFIED");
        }
        let a = if confirmed["ok"] == true {
            &confirmed["data"]
        } else {
            a
        };
        let previous = a["data"].clone();
        let stamp = a["lastSuccessfulSync"].clone();
        let req = request(
            "account.refresh",
            json!({"workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let http = if authority == "future_permission" {
            ChangedResponse {
                path: "/api/v2/spot/account/info",
                data: json!({"userId":"2","ips":"127.0.0.1","authorities":["stor"]}),
            }
        } else {
            ChangedResponse {
                path: "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal&idLessThan=101",
                data: json!({"orders":[]}),
            }
        };
        let outcome = job.run(
            &vault,
            |_| panic!("Refresh must use existing credentials"),
            &http,
            || cp.provider_job_current(&job),
        );
        let failed = cp.complete_provider(&job, outcome);
        assert_eq!(failed["ok"], false);
        if authority == "future_permission" {
            assert_eq!(failed["error"]["code"], "PROVIDER_IDENTITY_CHANGED");
        }
        let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
        let a = &list["data"]["accounts"][0];
        assert_eq!(a["data"], previous);
        assert_eq!(a["lastSuccessfulSync"], stamp);
        assert_eq!(a["health"]["executionEligibility"], "BLOCKED");
    }
}
#[test]
fn provider_schema_field_count_is_checked_before_storage_and_http() {
    for (provider, environment, values) in [
        (
            "alpaca",
            "PAPER",
            vec![KEY.into(), SECRET.into(), PASSPHRASE.into()],
        ),
        ("bitget", "LIVE", vec![KEY.into(), SECRET.into()]),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let req = request(
            "provider.connect",
            json!({"step":"test","workspaceId":ws,"providerId":provider,"environment":environment,"label":"Invalid field count"}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let vault = Vault::default();
        let http = LivePages(RefCell::new(vec![]));
        let outcome = job.run(
            &vault,
            |_| Credentials::new(values),
            &http,
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, outcome);
        assert_eq!(reply["ok"], false, "{reply}");
        assert_eq!(reply["error"]["code"], "IPC_PAYLOAD_INVALID");
        assert!(vault.0.borrow().is_none());
        assert!(http.0.borrow().is_empty());
    }
}

#[test]
fn changing_restricted_ip_addresses_requires_a_new_permission_review() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"IP review fixture"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let http = LivePages(RefCell::new(vec![]));
    let result = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, result);
    assert_eq!(reply["ok"], true, "{reply}");
    let a = &reply["data"];
    let confirmed = cp.dispatch(request("provider.connect", json!({"step":"confirm","workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"],"acknowledgeUnverified":true})));
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    let a = &confirmed["data"];
    let req = request(
        "account.refresh",
        json!({"workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let changed = ChangedResponse {
        path: "/api/v2/spot/account/info",
        data: json!({"userId":"9007199254740993","ips":"127.0.0.2","authorities":["stor","stow"]}),
    };
    let result = job.run(
        &vault,
        |_| panic!("No capture on refresh"),
        &changed,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, result);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["connectionState"], "REVIEW_REQUIRED");
    assert_eq!(reply["data"]["permissions"]["acknowledged"], false);
}

#[test]
fn malformed_identity_time_assets_pages_and_reflected_secrets_fail_without_a_snapshot() {
    let info = "/api/v2/spot/account/info";
    let assets = "/api/v2/spot/account/assets?assetType=all";
    let plan = "/api/v2/spot/trade/current-plan-order?limit=100";
    let normal = "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal";
    let coin = json!({"coin":"BTC","available":"1","frozen":"0","locked":"0","limitAvailable":"0"});
    let mut negative = coin.clone();
    negative["locked"] = "-1".into();
    let bad_order = json!({"userId":"9007199254740993","orderId":"1","symbol":"BTCUSDT","size":"1","orderType":"limit","side":"buy","status":"live","tpslType":"normal","priceAvg":"1","baseVolume":"0","quoteVolume":"0"});
    let mut wrong_user = bad_order.clone();
    wrong_user["userId"] = "2".into();
    let cases = vec![
        (
            "/api/v2/public/time",
            json!({"serverTime":"-1"}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v2/public/time",
            json!({"serverTime":1788849600000u64}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v2/public/time",
            json!({"serverTime":"1788849600000000"}),
            "CLOCK_SKEW",
        ),
        (
            info,
            json!({"userId":"../1","authorities":[],"ips":""}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            info,
            json!({"userId":"1","authorities":true,"ips":""}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            info,
            json!({"userId":"1","authorities":["stor","stor"],"ips":""}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            info,
            json!({"userId":"1","authorities":["stor"],"ips":"","unexpected":PASSPHRASE}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            assets,
            json!([coin.clone(), coin.clone()]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (assets, json!([negative]), "PROVIDER_RESPONSE_INVALID"),
        (
            assets,
            json!({"list":[coin.clone()]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            assets,
            Value::Array(vec![coin; 10_001]),
            "PROVIDER_DATA_INCOMPLETE",
        ),
        (normal, json!([wrong_user]), "PROVIDER_IDENTITY_CHANGED"),
        (
            normal,
            json!([bad_order.clone(), bad_order]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            plan,
            json!({"nextFlag":true,"idLessThan":"1","orderList":[]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            plan,
            json!({"nextFlag":"false","idLessThan":"1","orderList":[]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (plan, json!([]), "PROVIDER_RESPONSE_INVALID"),
    ];
    for (path, data, expected) in cases {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let req = request(
            "provider.connect",
            json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Rejected observation"}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let vault = Vault::default();
        let result = job.run(
            &vault,
            |_| credentials(),
            &ChangedResponse { path, data },
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, result);
        if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, &vault) {
            cp.record_credential_cleanup(&job, cleanup);
        }
        assert_eq!(reply["ok"], false, "{path}: {reply}");
        assert_eq!(reply["error"]["code"], expected, "{path}: {reply}");
        assert!(vault.0.borrow().is_none());
        let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
        let a = &list["data"]["accounts"][0];
        assert!(a["data"].is_null());
        assert!(a["lastSuccessfulSync"].is_null());
        assert_eq!(a["health"]["credential"], "MISSING");
        for secret in [KEY, SECRET, PASSPHRASE] {
            assert!(!format!("{reply}{list}").contains(secret));
        }
    }
}

#[test]
fn invalidation_during_a_page_stops_further_io_and_cleans_the_new_key() {
    struct Invalidated {
        current: std::cell::Cell<bool>,
        calls: RefCell<Vec<String>>,
    }
    impl ProviderHttp for Invalidated {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> Result<Vec<u8>> {
            assert!(self.current.get(), "No read may follow invalidation");
            self.calls.borrow_mut().push(path.into());
            let result = LivePages(RefCell::new(vec![])).get(endpoint, path, headers);
            if path == "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal" {
                self.current.set(false);
            }
            result
        }
    }
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Invalidated page"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let vault = Vault::default();
    let http = Invalidated {
        current: std::cell::Cell::new(true),
        calls: RefCell::new(vec![]),
    };
    let result = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job) && http.current.get(),
    );
    let reply = cp.complete_provider(&job, result);
    assert_eq!(reply["error"]["code"], "STATE_VERSION_CONFLICT", "{reply}");
    assert_eq!(http.calls.borrow().len(), 4);
    assert!(vault.0.borrow().is_none());
}
