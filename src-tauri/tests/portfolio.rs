use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("portfolio-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload,
    }))
}

#[test]
fn portfolio_read_is_workspace_scoped_and_does_not_mutate_state() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let aggregate = json!({"aggregateType":"workspace","aggregateId":workspace_id});
    let before = command(&mut control, "domain.snapshot", aggregate.clone());

    let portfolio = command(
        &mut control,
        "portfolio.get",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(portfolio["ok"], true, "{portfolio}");
    assert_eq!(portfolio["data"]["workspaceId"], workspace_id);
    assert_eq!(portfolio["data"]["status"], "UNAVAILABLE");
    assert_eq!(portfolio["data"]["fills"], Value::Null);
    assert_eq!(portfolio["data"]["liveRisk"]["eligible"], false);
    assert_eq!(
        command(&mut control, "domain.snapshot", aggregate.clone()),
        before,
        "portfolio reads must not mutate domain state"
    );

    let malformed = command(
        &mut control,
        "portfolio.get",
        json!({"workspaceId":workspace_id,"baseCurrency":"EUR"}),
    );
    assert_eq!(malformed["error"]["code"], "IPC_PAYLOAD_INVALID");

    let control_character = command(
        &mut control,
        "portfolio.get",
        json!({"workspaceId":format!("{workspace_id}\n")}),
    );
    assert_eq!(control_character["error"]["code"], "IPC_PAYLOAD_INVALID");
}
