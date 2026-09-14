use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("market-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload
    }))
}

#[test]
fn market_catalog_and_detail_preserve_identity_and_gate_entitlement() {
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
        "market.catalog",
        json!({"workspaceId":workspace_id,"query":"aapl","tier":"CENSUS"}),
    );
    assert_eq!(catalog["ok"], true, "{catalog}");
    assert_eq!(catalog["data"]["status"], "BLOCKED_EXTERNAL");
    assert_eq!(catalog["data"]["sourceId"], "OD-001");
    assert_eq!(
        catalog["data"]["instruments"][0]["instrumentId"],
        "equity:US:AAPL"
    );
    let default_tier_catalog = command(
        &mut control,
        "market.catalog",
        json!({"workspaceId":workspace_id,"query":"aapl"}),
    );
    assert_eq!(default_tier_catalog["ok"], true, "{default_tier_catalog}");
    assert_eq!(default_tier_catalog["data"]["tier"], "CENSUS");
    let crypto_catalog = command(
        &mut control,
        "market.catalog",
        json!({"workspaceId":workspace_id,"query":"BTC/USDT","tier":"CENSUS"}),
    );
    assert_eq!(crypto_catalog["ok"], true, "{crypto_catalog}");
    assert_eq!(crypto_catalog["data"]["status"], "UNAVAILABLE");
    assert!(crypto_catalog["data"]["sourceId"].is_null());
    let detail = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace_id,"instrumentId":"equity:US:AAPL","tier":"HOT"}),
    );
    assert_eq!(detail["ok"], true, "{detail}");
    assert_eq!(
        detail["data"]["instrument"]["instrumentId"],
        "equity:US:AAPL"
    );
    assert_eq!(detail["data"]["status"], "BLOCKED_EXTERNAL");
    assert!(detail["data"]["snapshot"].is_null());
    let crypto_detail = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace_id,"instrumentId":"crypto:BTC/USDT:spot","tier":"HOT"}),
    );
    assert_eq!(crypto_detail["ok"], true, "{crypto_detail}");
    assert_eq!(crypto_detail["data"]["status"], "UNAVAILABLE");
    assert!(crypto_detail["data"]["sourceId"].is_null());
    assert!(crypto_detail["data"]["snapshot"].is_null());
    let unknown = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace_id,"instrumentId":"equity:US:TSLA","tier":"HOT"}),
    );
    assert_eq!(unknown["error"]["code"], "MARKET_INSTRUMENT_NOT_FOUND");
    let malformed = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace_id,"instrumentId":"AAPL","tier":"HOT"}),
    );
    assert_eq!(malformed["error"]["code"], "MARKET_INSTRUMENT_INVALID");
    let after = command(
        &mut control,
        "domain.snapshot",
        json!({"aggregateType":"workspace","aggregateId":workspace_id}),
    );
    assert_eq!(before, after, "market reads must not mutate domain state");
}

#[test]
fn market_query_rejects_control_characters_without_network_or_mutation() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let result = command(
        &mut control,
        "market.catalog",
        json!({"workspaceId":workspace_id,"query":"AAPL\n","tier":"CENSUS"}),
    );
    assert_eq!(result["error"]["code"], "IPC_PAYLOAD_INVALID");
}
