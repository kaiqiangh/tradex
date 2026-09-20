use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("strategy-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload,
    }))
}

fn definition(name: &str, source: &str) -> Value {
    json!({
        "name": name,
        "source": source,
        "language": "python",
        "runtime": "sandbox-v1",
        "parameters": [{"name": "window", "value": "20"}]
    })
}

#[test]
fn strategy_versions_are_immutable_workspace_scoped_and_hash_checked() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();

    let saved = command(
        &mut control,
        "strategy.save_version",
        json!({"workspaceId": workspace_id, "definition": definition("Momentum", "return 1")}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let version_id = saved["data"]["strategyVersionId"].as_str().unwrap();
    let strategy_id = saved["data"]["strategyId"].as_str().unwrap();
    let hash = saved["data"]["sourceHash"].as_str().unwrap();
    assert_eq!(saved["data"]["revision"], 1);

    let second = command(
        &mut control,
        "strategy.save_version",
        json!({"workspaceId": workspace_id, "strategyId": strategy_id, "definition": definition("Momentum", "return 2")}),
    );
    assert_eq!(second["ok"], true, "{second}");
    assert_eq!(second["data"]["revision"], 2);
    assert_ne!(second["data"]["sourceHash"], hash);

    let original = command(
        &mut control,
        "strategy.get",
        json!({"workspaceId": workspace_id, "strategyVersionId": version_id}),
    );
    assert_eq!(original["ok"], true, "{original}");
    assert_eq!(original["data"]["definition"]["source"], "return 1");

    let list = command(
        &mut control,
        "strategy.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(list["data"]["versions"].as_array().unwrap().len(), 2);
    let catalog = command(
        &mut control,
        "context.catalog",
        json!({"workspaceId": workspace_id}),
    );
    assert!(
        catalog["data"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["contextRef"]["kind"] == "strategy")
    );
}

#[test]
fn strategy_run_fails_closed_for_tampered_hash_dataset_and_untrusted_time() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let saved = command(
        &mut control,
        "strategy.save_version",
        json!({"workspaceId": workspace_id, "definition": definition("Momentum", "return 1")}),
    );
    let version_id = saved["data"]["strategyVersionId"].as_str().unwrap();
    let base = json!({
        "workspaceId": workspace_id,
        "strategyVersionId": version_id,
        "instrumentId": "equity:US:AAPL",
        "datasetId": "historical:fixture",
        "startAt": "2026-01-01T00:00:00Z",
        "endAt": "2026-01-02T00:00:00Z",
        "parameters": []
    });
    let mut tampered_payload = base.clone();
    tampered_payload["expectedStrategyHash"] = json!("sha256:tampered");
    let tampered = command(&mut control, "strategy.run", tampered_payload);
    assert_eq!(
        tampered["error"]["code"], "STRATEGY_HASH_MISMATCH",
        "{tampered}"
    );
    let mut unknown_dataset_payload = base.clone();
    unknown_dataset_payload["datasetId"] = json!("missing");
    let unknown_dataset = command(&mut control, "strategy.run", unknown_dataset_payload);
    assert_eq!(
        unknown_dataset["error"]["code"], "STRATEGY_DATASET_NOT_FOUND",
        "{unknown_dataset}"
    );
    let mut unknown_historical = base.clone();
    unknown_historical["datasetId"] = json!("historical:unknown");
    let unknown_historical = command(&mut control, "strategy.run", unknown_historical);
    assert_eq!(
        unknown_historical["error"]["code"], "STRATEGY_DATASET_NOT_FOUND",
        "{unknown_historical}"
    );
    let blocked = command(&mut control, "strategy.run", base);
    assert_eq!(
        blocked["error"]["code"], "STRATEGY_TIME_UNTRUSTED",
        "{blocked}"
    );
    assert_eq!(
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId": workspace_id}),
        )["ok"],
        true
    );
    let duplicate = command(
        &mut control,
        "strategy.run",
        json!({
            "workspaceId": workspace_id,
            "strategyVersionId": version_id,
            "instrumentId": "equity:US:AAPL",
            "datasetId": "historical:fixture",
            "startAt": "2026-01-01T00:00:00Z",
            "endAt": "2026-01-02T00:00:00Z",
            "parameters": [{"name":"window","value":"20"},{"name":"window","value":"30"}]
        }),
    );
    assert_eq!(
        duplicate["error"]["code"], "STRATEGY_PARAMETER_INVALID",
        "{duplicate}"
    );
    let runs = command(
        &mut control,
        "strategy.list",
        json!({"workspaceId": workspace_id}),
    );
    assert!(
        runs["data"]["runs"].as_array().unwrap().is_empty(),
        "{runs}"
    );
}

#[test]
fn persisted_strategy_runs_reject_tampering_and_reconcile_active_rows() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_path = directory.path().join("workspace");
    let mut control = ControlPlane::new(workspace_path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let saved = command(
        &mut control,
        "strategy.save_version",
        json!({"workspaceId": workspace_id, "definition": definition("Momentum", "return 1")}),
    );
    let version_id = saved["data"]["strategyVersionId"].as_str().unwrap();
    assert_eq!(
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId": workspace_id}),
        )["ok"],
        true
    );
    let run = command(
        &mut control,
        "strategy.run",
        json!({
            "workspaceId": workspace_id,
            "strategyVersionId": version_id,
            "instrumentId": "equity:US:AAPL",
            "datasetId": "historical:fixture",
            "startAt": "2026-01-01T00:00:00Z",
            "endAt": "2026-01-02T00:00:00Z",
            "parameters": []
        }),
    );
    assert_eq!(run["ok"], true, "{run}");
    let run_id = run["data"]["runId"].as_str().unwrap();
    let database = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    let original: String = database
        .query_row(
            "SELECT projection FROM strategy_runs WHERE run_id=?1",
            [run_id],
            |row| row.get(0),
        )
        .unwrap();
    let mut tampered: Value = serde_json::from_str(&original).unwrap();
    tampered["requestHash"] = json!(format!("sha256:{}", "0".repeat(64)));
    database
        .execute(
            "UPDATE strategy_runs SET projection=?1 WHERE run_id=?2",
            rusqlite::params![serde_json::to_string(&tampered).unwrap(), run_id],
        )
        .unwrap();
    let rejected = command(
        &mut control,
        "strategy.get_run",
        json!({"workspaceId": workspace_id, "runId": run_id}),
    );
    assert_eq!(
        rejected["error"]["code"], "WORKSPACE_INTEGRITY_FAILED",
        "{rejected}"
    );

    let mut active = tampered;
    active["requestHash"] = json!(run["data"]["requestHash"]);
    active["state"] = json!("QUEUED");
    active["failure"] = Value::Null;
    active["signal"] = Value::Null;
    database
        .execute(
            "UPDATE strategy_runs SET projection=?1 WHERE run_id=?2",
            rusqlite::params![serde_json::to_string(&active).unwrap(), run_id],
        )
        .unwrap();
    let reopened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(reopened["ok"], true, "{reopened}");
    let reconciled = command(
        &mut control,
        "strategy.get_run",
        json!({"workspaceId": workspace_id, "runId": run_id}),
    );
    assert_eq!(reconciled["data"]["state"], "CANCELLED", "{reconciled}");
}
