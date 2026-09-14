use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("request-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload,
    }))
}

#[test]
fn time_commands_are_workspace_scoped_and_do_not_mutate_domain_state() {
    let directory = tempfile::tempdir().unwrap();
    let first_path = directory.path().join("first");
    let second_path = directory.path().join("second");
    let mut control = ControlPlane::new(first_path);
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
    let aggregate = json!({"aggregateType":"workspace","aggregateId":workspace_id});
    let before = command(&mut control, "domain.snapshot", aggregate.clone());

    let status = command(
        &mut control,
        "time.status",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(status["ok"], true, "{status}");
    assert_eq!(status["data"]["confidence"], "CLOCK_UNCERTAIN");
    assert_eq!(status["data"]["remediation"]["id"], "time_revalidate");
    let trusted = command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(trusted["ok"], true, "{trusted}");
    assert_eq!(trusted["data"]["confidence"], "TRUSTED");
    assert_eq!(
        command(&mut control, "domain.snapshot", aggregate.clone()),
        before
    );

    control.resume();
    let after_resume = command(
        &mut control,
        "time.status",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(after_resume["data"]["confidence"], "CLOCK_UNCERTAIN");
    assert_eq!(
        command(&mut control, "domain.snapshot", aggregate.clone()),
        before
    );

    let malformed = command(
        &mut control,
        "time.status",
        json!({"workspaceId":workspace_id,"wallClock":"override"}),
    );
    assert_eq!(malformed["error"]["code"], "IPC_PAYLOAD_INVALID");
    assert_eq!(
        command(&mut control, "domain.snapshot", aggregate.clone()),
        before
    );

    let reopened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(reopened["data"]["workspaceId"], workspace_id);
    let after_reopen = command(
        &mut control,
        "time.status",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(after_reopen["data"]["confidence"], "CLOCK_UNCERTAIN");

    let switched = command(&mut control, "workspace.open", json!({"path":second_path}));
    let second_id = switched["data"]["workspaceId"].as_str().unwrap();
    let wrong_workspace = command(
        &mut control,
        "time.status",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(wrong_workspace["error"]["code"], "IPC_AGGREGATE_NOT_FOUND");
    let second_status = command(
        &mut control,
        "time.status",
        json!({"workspaceId":second_id}),
    );
    assert_eq!(second_status["data"]["confidence"], "CLOCK_UNCERTAIN");
}
