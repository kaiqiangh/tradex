use serde_json::{Value, json};
use tradex::{
    ControlPlane,
    provider_io::{CredentialVault, ProviderHttp},
};
#[path = "support/provider_fixtures.rs"]
mod fixtures;

fn request(command: &str, payload: Value) -> Value {
    json!({"requestId":"trading212-test","schemaVersion":1,"command":command,"payload":payload})
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
    let mut accounts = vec![];
    for env in ["DEMO", "LIVE"] {
        let schema = call(
            &mut cp,
            "provider.get_schema",
            json!({"providerId":"trading212","environment":env}),
        );
        assert_eq!(schema["ok"], true, "{schema}");
        assert_eq!(schema["data"]["fields"].as_array().unwrap().len(), 2);
        let reply = run(
            &mut cp,
            request(
                "provider.connect",
                json!({"step":"test","workspaceId":ws,"providerId":"trading212","environment":env,"label":format!("T212 {env}")}),
            ),
            vault,
            &http,
        );
        assert_eq!(reply["ok"], true, "{reply}");
        let host = if env == "DEMO" {
            "https://demo.trading212.com"
        } else {
            "https://live.trading212.com"
        };
        assert!(
            http.calls
                .borrow()
                .iter()
                .rev()
                .take(3)
                .all(|url| url.starts_with(host)),
            "The job must route every request to its immutable environment"
        );
        let a = &reply["data"];
        assert_eq!(a["data"]["remoteAccountId"], "9007199254740993");
        assert_eq!(
            a["data"]["balances"][0]["available"],
            "1000.1234567890123456789"
        );
        assert_eq!(a["data"]["balances"][0]["reserved"], "20.5");
        assert_eq!(a["data"]["positions"][0]["quantity"], "0.00000012");
        assert_eq!(a["data"]["positions"][0]["instrumentCurrency"], "USD");
        assert_eq!(a["data"]["positions"][0]["marketValueCurrency"], "GBP");
        assert_eq!(
            a["data"]["openOrders"][0]["brokerOrderId"],
            "9007199254740995"
        );
        assert_eq!(a["data"]["openOrders"][0]["filledQuantity"], Value::Null);
        assert_eq!(a["data"]["openOrders"][0]["filledValue"], "1.23");
        assert_eq!(a["permissions"]["scope"], "UNVERIFIED");
        assert_eq!(
            a["health"]["arming"],
            if env == "LIVE" {
                "DISARMED"
            } else {
                "NOT_APPLICABLE"
            }
        );
        assert_eq!(a["health"]["executionEligibility"], "BLOCKED");
        let mut confirm = mutation(a);
        confirm["step"] = "confirm".into();
        confirm["acknowledgeUnverified"] = false.into();
        assert_eq!(
            call(&mut cp, "provider.connect", confirm.clone())["error"]["code"],
            "PROVIDER_REVIEW_REQUIRED"
        );
        confirm["acknowledgeUnverified"] = true.into();
        let accepted = call(&mut cp, "provider.connect", confirm);
        assert_eq!(accepted["ok"], true);
        accounts.push(accepted["data"].clone());
    }
    let refs: Vec<_> = accounts
        .iter()
        .map(|a| {
            serde_json::from_value::<tradex::providers::AccountConnection>(a.clone())
                .unwrap()
                .credential_ref()
        })
        .collect();
    assert_ne!(refs[0], refs[1]);
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().into());
    call(&mut cp, "workspace.open", json!({}));
    for a in &mut accounts {
        let mut get = mutation(a);
        get.as_object_mut().unwrap().remove("expectedStateVersion");
        let restored = call(&mut cp, "account.get", get)["data"].clone();
        assert_eq!(restored["health"]["connection"], "STALE");
        let refreshed = run(
            &mut cp,
            request("provider.probe", mutation(&restored)),
            vault,
            &http,
        );
        assert_eq!(refreshed["ok"], true, "{refreshed}");
        *a = refreshed["data"].clone();
    }
    for (i, a) in accounts.iter().enumerate() {
        let reply = run(
            &mut cp,
            request("provider.disconnect", mutation(a)),
            vault,
            &http,
        );
        assert_eq!(reply["data"]["health"]["credential"], "MISSING");
        assert!(vault.get(&refs[i]).is_err());
        if i == 0 {
            assert!(
                vault.get(&refs[1]).is_ok(),
                "Demo cleanup must not remove Live credentials"
            );
        }
    }
    for secret in [
        fixtures::KEY,
        fixtures::SECRET,
        "UzAyLUZBS0UtS0VZLTU5NDc5MTQ1MzpTMDItRkFLRS1TRUNSRVQtNzA0NTU2OTIx",
    ] {
        assert!(!serde_json::to_string(&accounts).unwrap().contains(secret));
        for entry in std::fs::read_dir(folder.path()).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                assert!(
                    !std::fs::read(path)
                        .unwrap()
                        .windows(secret.len())
                        .any(|b| b == secret.as_bytes())
                );
            }
        }
    }
}
#[test]
fn demo_and_live_preserve_exact_observations_and_independent_credentials() {
    lifecycle(&fixtures::Vault::default());
}
#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit OS Keychain integration with disposable Trading 212 credentials"]
fn native_keychain_keeps_demo_and_live_items_separate() {
    lifecycle(&tradex::provider_io::NativeVault);
}

struct Response {
    path: &'static str,
    body: Vec<u8>,
    error: Option<&'static str>,
    calls: std::cell::RefCell<Vec<String>>,
}
impl ProviderHttp for Response {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        self.calls.borrow_mut().push(path.into());
        if let Some(code) = self.error {
            return Err(tradex::protocol::TradeXError::new(code));
        }
        if path == self.path {
            Ok(self.body.clone())
        } else {
            fixtures::Http::default().get(endpoint, path, headers)
        }
    }
}
#[test]
fn failed_or_hostile_reads_preserve_identity_data_and_last_success() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = fixtures::Vault::default();
    let connected = run(
        &mut cp,
        request(
            "provider.connect",
            json!({"step":"test","workspaceId":ws,"providerId":"trading212","environment":"DEMO","label":"Fault check"}),
        ),
        &vault,
        &fixtures::Http::default(),
    );
    let original = connected["data"].clone();
    let mut current = original.clone();
    for (path,body,code) in [
        ("/api/v0/equity/account/summary",br#"{"id":123}"#.to_vec(),"PROVIDER_IDENTITY_CHANGED"),
        ("/api/v0/equity/account/summary",b"not json".to_vec(),"PROVIDER_RESPONSE_INVALID"),
        ("/api/v0/equity/positions",br#"[{"instrument":{"ticker":"AAPL_US_EQ"},"quantity":1,"averagePricePaid":50}]"#.to_vec(),"PROVIDER_RESPONSE_INVALID"),
        ("/api/v0/equity/positions",br#"[{"instrument":{"ticker":"AAPL_US_EQ","currency":"USD"},"quantity":1e-2147483648}]"#.to_vec(),"PROVIDER_RESPONSE_INVALID"),
        ("/api/v0/equity/orders",br#"{"items":[],"nextPagePath":"https://example.invalid"}"#.to_vec(),"PROVIDER_RESPONSE_INVALID"),
        ("/api/v0/equity/positions",format!(r#"[{{"instrument":{{"ticker":"{}","currency":"USD"}},"quantity":1}}]"#,fixtures::KEY.replace('-',"\\u002d")).into_bytes(),"PROVIDER_RESPONSE_INVALID"),
        ("/api/v0/equity/positions",br#"[{"instrument":{"ticker":"UzAyLUZBS0UtS0VZLTU5NDc5MTQ1MzpTMDItRkFLRS1TRUNSRVQtNzA0NTU2OTIx","currency":"USD"},"quantity":1}]"#.to_vec(),"PROVIDER_RESPONSE_INVALID"),
        ("/api/v0/equity/orders",br#"[{"id":12,"ticker":"MSFT_US_EQ","strategy":"VALUE","side":"BUY","status":"NEW","currency":"GBP","filledValue":0}]"#.to_vec(),"PROVIDER_RESPONSE_INVALID"),
        ("/api/v0/equity/orders",vec![b' ';2*1024*1024+1],"PROVIDER_RESPONSE_INVALID"),
    ] {
        let http=Response{path,body,error:None,calls:Default::default()};
        let reply=run(&mut cp,request("account.refresh",mutation(&current)),&vault,&http);
        assert_eq!(reply["error"]["code"],code,"{reply}");
        if code == "PROVIDER_IDENTITY_CHANGED" { assert_eq!(http.calls.borrow().len(),1); }
        current=call(&mut cp,"account.get",json!({"workspaceId":ws,"connectionId":original["connectionId"]}))["data"].clone();
        assert_eq!(current["data"],original["data"]);
        assert_eq!(current["lastSuccessfulSync"],original["lastSuccessfulSync"]);
        assert_eq!(current["health"]["executionEligibility"],"BLOCKED");
    }
    for code in [
        "PROVIDER_AUTH_FAILED",
        "PROVIDER_RATE_LIMITED",
        "PROVIDER_UNAVAILABLE",
    ] {
        let http = Response {
            path: "",
            body: vec![],
            error: Some(code),
            calls: Default::default(),
        };
        let reply = run(
            &mut cp,
            request("provider.probe", mutation(&current)),
            &vault,
            &http,
        );
        assert_eq!(reply["error"]["code"], code);
        current = call(
            &mut cp,
            "account.get",
            json!({"workspaceId":ws,"connectionId":original["connectionId"]}),
        )["data"]
            .clone();
        assert_eq!(current["data"], original["data"]);
        assert_eq!(
            current["lastSuccessfulSync"],
            original["lastSuccessfulSync"]
        );
    }
    use tradex::provider_io::{BrokerHttp, ProviderEndpoint};
    for endpoint in [
        ProviderEndpoint::Trading212Demo,
        ProviderEndpoint::Trading212Live,
    ] {
        for path in [
            "/v2/account",
            "https://example.invalid",
            "/api/v0/equity/orders/123",
            "/api/v0/equity/orders?cursor=1",
        ] {
            assert_eq!(
                BrokerHttp::default()
                    .get(endpoint, path, Default::default())
                    .unwrap_err()
                    .code,
                "PROVIDER_UNSUPPORTED"
            );
        }
    }
}
