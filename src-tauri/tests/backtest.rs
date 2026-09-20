use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("backtest-{name}"),
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
        "parameters": []
    })
}

fn request(workspace_id: &str, version_id: &str) -> Value {
    json!({
        "workspaceId": workspace_id,
        "strategyVersionId": version_id,
        "instrumentId": "equity:US:AAPL",
        "datasetId": "historical:fixture",
        "startAt": "2026-01-01T00:00:00Z",
        "endAt": "2026-01-02T00:00:00Z",
        "barInterval": "1d",
        "startingCash": "100000",
        "commission": "0",
        "slippage": "0",
        "parameters": []
    })
}

#[test]
fn backtest_validates_inputs_and_persists_typed_runtime_failure() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_path = directory.path().join("workspace");
    let mut control = ControlPlane::new(workspace_path.clone());
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

    let mut tampered_hash = request(workspace_id, version_id);
    tampered_hash["expectedStrategyHash"] = json!("sha256:tampered");
    assert_eq!(
        command(&mut control, "backtest.run", tampered_hash)["error"]["code"],
        "BACKTEST_STRATEGY_HASH_INVALID"
    );

    let mut unknown_dataset = request(workspace_id, version_id);
    unknown_dataset["datasetId"] = json!("historical:unknown");
    assert_eq!(
        command(&mut control, "backtest.run", unknown_dataset)["error"]["code"],
        "BACKTEST_DATASET_NOT_FOUND"
    );

    let blocked = command(
        &mut control,
        "backtest.run",
        request(workspace_id, version_id),
    );
    assert_eq!(
        blocked["error"]["code"], "BACKTEST_TIME_UNTRUSTED",
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

    let run = command(
        &mut control,
        "backtest.run",
        request(workspace_id, version_id),
    );
    assert_eq!(run["ok"], true, "{run}");
    assert_eq!(run["data"]["state"], "FAILED");
    assert_eq!(
        run["data"]["failure"]["code"],
        "BACKTEST_RUNTIME_UNAVAILABLE"
    );
    assert_eq!(run["data"]["startingCash"], "100000");
    assert!(
        run["data"]["requestHash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );

    let run_id = run["data"]["runId"].as_str().unwrap();
    let loaded = command(
        &mut control,
        "backtest.get",
        json!({"workspaceId": workspace_id, "runId": run_id}),
    );
    assert_eq!(loaded["data"], run["data"], "{loaded}");
    let repeated = command(
        &mut control,
        "backtest.run",
        request(workspace_id, version_id),
    );
    assert_eq!(repeated["ok"], true, "{repeated}");
    assert_eq!(repeated["data"]["requestHash"], run["data"]["requestHash"]);
    assert_ne!(repeated["data"]["runId"], run["data"]["runId"]);
    let cancelled = command(
        &mut control,
        "backtest.cancel",
        json!({"workspaceId": workspace_id, "runId": run_id, "expectedStateVersion": run["data"]["stateVersion"]}),
    );
    assert_eq!(
        cancelled["error"]["code"], "BACKTEST_RUN_NOT_CANCELLABLE",
        "{cancelled}"
    );

    drop(control);
    let database = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    let mut projection: Value = database
        .query_row(
            "SELECT projection FROM backtest_runs WHERE run_id=?1",
            [run_id],
            |row| row.get::<_, String>(0),
        )
        .map(|value| serde_json::from_str(&value).unwrap())
        .unwrap();
    projection["state"] = json!("RUNNING");
    projection["failure"] = Value::Null;
    database
        .execute(
            "UPDATE backtest_runs SET projection=?1 WHERE run_id=?2",
            rusqlite::params![serde_json::to_string(&projection).unwrap(), run_id],
        )
        .unwrap();
    drop(database);
    let mut restarted = ControlPlane::new(workspace_path.clone());
    let reopened = command(&mut restarted, "workspace.open", json!({}));
    assert_eq!(reopened["ok"], true, "{reopened}");
    let reconciled = command(
        &mut restarted,
        "backtest.get",
        json!({"workspaceId": workspace_id, "runId": run_id}),
    );
    assert_eq!(reconciled["data"]["state"], "CANCELLED", "{reconciled}");
    assert_eq!(reconciled["data"]["failure"]["code"], "BACKTEST_CANCELLED");
    drop(restarted);
    let database = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    let count: i64 = database
        .query_row("SELECT COUNT(*) FROM backtest_runs", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn backtest_rejects_invalid_numeric_interval_and_date_values() {
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
    for (field, value, code) in [
        ("startingCash", "0", "BACKTEST_DECIMAL_INVALID"),
        ("commission", "1e-3", "BACKTEST_DECIMAL_INVALID"),
        ("barInterval", "2d", "BACKTEST_BAR_INTERVAL_INVALID"),
        (
            "endAt",
            "2025-01-01T00:00:00Z",
            "BACKTEST_DATE_RANGE_INVALID",
        ),
    ] {
        let mut input = request(workspace_id, version_id);
        input[field] = json!(value);
        assert_eq!(
            command(&mut control, "backtest.run", input)["error"]["code"],
            code,
            "field={field}"
        );
    }
}
