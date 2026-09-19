use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("screener-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload,
    }))
}

#[test]
fn screener_parse_run_is_typed_revisioned_and_read_only() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let aggregate = json!({"aggregateType":"workspace","aggregateId":workspace_id});
    let before = command(&mut control, "domain.snapshot", aggregate.clone());
    let natural_language = "US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.";
    let parsed = command(
        &mut control,
        "market.screen",
        json!({"workspaceId":workspace_id,"operation":"PARSE","naturalLanguage":natural_language,"focus":"EQUITY"}),
    );
    assert_eq!(parsed["ok"], true, "{parsed}");
    assert_eq!(parsed["data"]["state"], "PARSED");
    assert_eq!(
        parsed["data"]["filterSpec"]["predicates"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let run = command(
        &mut control,
        "market.screen",
        json!({
            "workspaceId":workspace_id,
            "operation":"RUN",
            "naturalLanguage":natural_language,
            "focus":"EQUITY",
            "filterSpec":parsed["data"]["filterSpec"],
            "rankSpec":parsed["data"]["rankSpec"],
            "revision":parsed["data"]["revision"],
            "limit":10
        }),
    );
    assert_eq!(run["ok"], true, "{run}");
    assert_eq!(run["data"]["state"], "BLOCKED_EXTERNAL");
    assert!(run["data"]["candidates"].as_array().unwrap().is_empty());
    let stale = command(
        &mut control,
        "market.screen",
        json!({
            "workspaceId":workspace_id,
            "operation":"RUN",
            "naturalLanguage":natural_language,
            "filterSpec":parsed["data"]["filterSpec"],
            "rankSpec":parsed["data"]["rankSpec"],
            "revision":"sha256:stale",
            "limit":10
        }),
    );
    assert_eq!(stale["error"]["code"], "SCREENER_REVISION_STALE");
    assert_eq!(command(&mut control, "domain.snapshot", aggregate), before);
}

#[test]
fn screener_rejects_control_characters_and_unknown_fields() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let control_character = command(
        &mut control,
        "market.screen",
        json!({"workspaceId":workspace_id,"operation":"PARSE","naturalLanguage":"RSI below 70\n"}),
    );
    assert_eq!(control_character["error"]["code"], "IPC_PAYLOAD_INVALID");
    let unknown = command(
        &mut control,
        "market.screen",
        json!({"workspaceId":workspace_id,"operation":"PARSE","naturalLanguage":"RSI below 70","unexpected":true}),
    );
    assert_eq!(unknown["error"]["code"], "IPC_PAYLOAD_INVALID");
}
