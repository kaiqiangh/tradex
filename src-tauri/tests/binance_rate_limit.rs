use serde_json::{Value, json};
use tradex::{
    ControlPlane,
    provider_io::{CredentialVault, ProviderHttp},
};
#[path = "support/provider_fixtures.rs"]
mod fixtures;

fn request(command: &str, payload: Value) -> Value {
    json!({"requestId":"binance-rate-limit","schemaVersion":1,"command":command,"payload":payload})
}

fn call(cp: &mut ControlPlane, command: &str, payload: Value) -> Value {
    cp.dispatch(request(command, payload))
}

fn run(
    cp: &mut ControlPlane,
    req: Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let job = cp.prepare_provider_for(&req, "main").unwrap().unwrap();
    let outcome = job.run(
        vault,
        |_| fixtures::credentials(),
        http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, vault) {
        cp.record_credential_cleanup(&job, cleanup);
    }
    reply
}

fn connect(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
    label: &str,
) -> Value {
    let tested = run(
        cp,
        request(
            "provider.connect",
            json!({"step":"test","workspaceId":workspace,"providerId":"binance","environment":"TESTNET","label":label}),
        ),
        vault,
        http,
    );
    assert_eq!(tested["ok"], true, "{tested}");
    let accepted = call(
        cp,
        "provider.connect",
        json!({
            "step":"confirm","workspaceId":workspace,
            "connectionId":tested["data"]["connectionId"],
            "expectedStateVersion":tested["data"]["stateVersion"],
            "acknowledgeUnverified":true
        }),
    );
    assert_eq!(accepted["ok"], true, "{accepted}");
    accepted["data"].clone()
}

#[test]
fn binance_testnet_retry_after_blocks_other_connections_on_the_shared_ip() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let first = connect(&mut cp, &vault, &http, &workspace, "Rate limited account");
    http.binance_uid.set(9007199254740994);
    let second = connect(&mut cp, &vault, &http, &workspace, "Other Testnet account");
    http.binance_uid.set(9007199254740993);

    http.binance_read_statuses
        .borrow_mut()
        .insert("/api/v3/allOrders".into(), 429);
    http.binance_retry_after_seconds.set(Some(91));
    let start = time::OffsetDateTime::now_utc().unix_timestamp();
    let limited = run(
        &mut cp,
        request(
            "binance.testnet.orders.refresh",
            json!({
                "workspaceId":workspace,"connectionId":first["connectionId"],
                "expectedConnectionStateVersion":first["stateVersion"],
                "action":"HISTORY","symbol":"BTCUSDT"
            }),
        ),
        &vault,
        &http,
    );
    assert_eq!(limited["ok"], true, "{limited}");
    assert_eq!(limited["data"]["status"], "DEGRADED");
    assert_eq!(limited["data"]["reason"], "PROVIDER_RATE_LIMITED");
    let retry_at = limited["data"]["rateLimits"]["accountRetryAt"]
        .as_str()
        .unwrap();
    let retry_timestamp =
        time::OffsetDateTime::parse(retry_at, &time::format_description::well_known::Rfc3339)
            .unwrap()
            .unix_timestamp();
    assert!((91..=92).contains(&(retry_timestamp - start)));

    let calls_before = http.calls.borrow().len();
    let blocked = run(
        &mut cp,
        request(
            "binance.testnet.orders.refresh",
            json!({
                "workspaceId":workspace,"connectionId":second["connectionId"],
                "expectedConnectionStateVersion":second["stateVersion"],
                "action":"PENDING"
            }),
        ),
        &vault,
        &http,
    );
    assert_eq!(blocked["ok"], true, "{blocked}");
    assert_eq!(blocked["data"]["status"], "DEGRADED");
    assert_eq!(blocked["data"]["reason"], "PROVIDER_RATE_LIMITED");
    assert_eq!(blocked["data"]["rateLimits"]["accountRetryAt"], retry_at);
    assert_eq!(
        http.calls.borrow().len(),
        calls_before,
        "a second connection sharing this process egress must not call Binance during Retry-After"
    );
}
