use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": "data-source-test",
        "schemaVersion": 1,
        "command": name,
        "payload": payload
    }))
}

#[test]
fn catalog_and_credentialed_probe_are_scoped_and_read_only() {
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
        "data.source.catalog",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(catalog["ok"], true, "{catalog}");
    assert_eq!(catalog["data"]["sources"].as_array().unwrap().len(), 6);
    let version = catalog["data"]["stateVersion"].as_str().unwrap();
    let alpaca = command(
        &mut control,
        "data.source.probe",
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":version}),
    );
    assert_eq!(alpaca["ok"], true, "{alpaca}");
    assert_eq!(
        alpaca["data"]["sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["sourceId"] == "OD-001")
            .unwrap()["status"],
        "BLOCKED_EXTERNAL"
    );
    assert_eq!(
        command(
            &mut control,
            "data.source.probe",
            json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":"stale"}),
        )["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    assert_eq!(
        command(
            &mut control,
            "data.source.probe",
            json!({"workspaceId":workspace_id,"sourceId":"OD-999","expectedStateVersion":version}),
        )["error"]["code"],
        "DATA_SOURCE_UNKNOWN"
    );
    assert_eq!(
        command(
            &mut control,
            "data.source.catalog",
            json!({"workspaceId":"other"}),
        )["error"]["code"],
        "IPC_AGGREGATE_NOT_FOUND"
    );
    for payload in [
        json!({"workspaceId":workspace_id,"sourceId":"","expectedStateVersion":version}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001\n","expectedStateVersion":version}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":""}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":"x".repeat(257)}),
        json!({"workspaceId":workspace_id,"sourceId":"OD-001","expectedStateVersion":"bad\nversion"}),
    ] {
        assert_eq!(
            command(&mut control, "data.source.probe", payload)["error"]["code"],
            "IPC_PAYLOAD_INVALID"
        );
    }
    let after = command(
        &mut control,
        "domain.snapshot",
        json!({"aggregateType":"workspace","aggregateId":workspace_id}),
    );
    assert_eq!(
        before, after,
        "source catalog/probe must not mutate domain state"
    );
}
