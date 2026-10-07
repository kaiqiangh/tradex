use serde_json::{Value, json};
use tradex::{ControlPlane, provider_io::CredentialVault};
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

#[test]
fn public_rule_source_selection_is_versioned_metadata_and_keeps_the_borrowed_key() {
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
