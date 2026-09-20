use serde_json::{Value, json};
use std::sync::{Mutex, OnceLock};
use tradex::ControlPlane;

static FIXTURE_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

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
        "parameters": [{"name": "window", "value": "20"}]
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
        "startingCash": "00100000.00",
        "commission": "00.00",
        "slippage": "00.00",
        "parameters": []
    })
}

#[test]
fn backtest_validates_inputs_and_persists_typed_runtime_failure() {
    let _fixture_env_lock = FIXTURE_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    unsafe {
        std::env::remove_var("TRADEX_BACKTEST_FIXTURE");
    }
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

    if !cfg!(feature = "integration-test") {
        let blocked = command(
            &mut control,
            "backtest.run",
            request(workspace_id, version_id),
        );
        assert_eq!(
            blocked["error"]["code"], "MARKET_HISTORY_UNAVAILABLE",
            "{blocked}"
        );
        return;
    }
    unsafe {
        std::env::set_var("TRADEX_BACKTEST_FIXTURE", "1");
    }

    let run = command(
        &mut control,
        "backtest.run",
        request(workspace_id, version_id),
    );
    assert_eq!(run["ok"], true, "{run}");
    assert_eq!(run["data"]["state"], "FAILED");
    assert_eq!(
        run["data"]["failure"]["code"],
        if cfg!(feature = "integration-test") {
            "BACKTEST_FIXTURE_FAILED"
        } else {
            "BACKTEST_RUNTIME_UNAVAILABLE"
        }
    );
    assert_eq!(run["data"]["startingCash"], "100000");
    assert_eq!(run["data"]["commission"], "0");
    assert_eq!(run["data"]["slippage"], "0");
    assert_eq!(
        run["data"]["parameters"],
        json!([{"name": "window", "value": "20"}])
    );
    assert!(
        run["data"]["requestHash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );

    let run_id = run["data"]["runId"].as_str().unwrap();
    let mut unknown_field = request(workspace_id, version_id);
    unknown_field["unknownField"] = json!(true);
    assert_eq!(
        command(&mut control, "backtest.run", unknown_field)["error"]["code"],
        "IPC_PAYLOAD_INVALID"
    );
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
    let mut cas_control = ControlPlane::new(workspace_path.clone());
    assert_eq!(
        command(&mut cas_control, "workspace.open", json!({}))["ok"],
        true
    );
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
    let stale_cancel = command(
        &mut cas_control,
        "backtest.cancel",
        json!({"workspaceId": workspace_id, "runId": run_id, "expectedStateVersion": "stale"}),
    );
    assert_eq!(
        stale_cancel["error"]["code"], "STATE_VERSION_CONFLICT",
        "{stale_cancel}"
    );
    let after_stale = command(
        &mut cas_control,
        "backtest.get",
        json!({"workspaceId": workspace_id, "runId": run_id}),
    );
    assert_eq!(after_stale["data"]["state"], "RUNNING", "{after_stale}");
    let reopened = command(&mut cas_control, "workspace.open", json!({}));
    assert_eq!(reopened["ok"], true, "{reopened}");
    let reconciled = command(
        &mut cas_control,
        "backtest.get",
        json!({"workspaceId": workspace_id, "runId": run_id}),
    );
    assert_eq!(reconciled["data"]["state"], "CANCELLED", "{reconciled}");
    assert_eq!(reconciled["data"]["failure"]["code"], "BACKTEST_CANCELLED");
    let other_path = directory.path().join("other-workspace");
    let mut other = ControlPlane::new(other_path.clone());
    let other_opened = command(
        &mut other,
        "workspace.open",
        json!({"path": other_path.to_string_lossy()}),
    );
    assert_eq!(other_opened["ok"], true, "{other_opened}");
    let cross_workspace = command(
        &mut other,
        "backtest.get",
        json!({"workspaceId": workspace_id, "runId": run_id}),
    );
    assert_eq!(
        cross_workspace["error"]["code"], "IPC_AGGREGATE_NOT_FOUND",
        "{cross_workspace}"
    );
    drop(cas_control);
    drop(other);
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
    let mut uncovered = request(workspace_id, version_id);
    uncovered["startAt"] = json!("1900-01-01T00:00:00Z");
    assert_eq!(
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId": workspace_id}),
        )["ok"],
        true
    );
    let coverage_error = command(&mut control, "backtest.run", uncovered);
    assert_eq!(
        coverage_error["error"]["code"],
        "MARKET_HISTORY_UNAVAILABLE"
    );
    assert_eq!(coverage_error["error"]["field"], "startAt");
}

#[cfg(feature = "integration-test")]
#[test]
fn completed_backtest_persists_deterministic_result_and_typed_guards() {
    let _fixture_env_lock = FIXTURE_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    unsafe {
        std::env::set_var("TRADEX_BACKTEST_FIXTURE", "1");
    }
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

    let mut success_request = request(workspace_id, version_id);
    success_request["fixtureScenario"] = json!("SUCCESS");
    let first = command(&mut control, "backtest.run", success_request.clone());
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(first["data"]["state"], "COMPLETED", "{first}");
    assert_eq!(first["data"]["result"]["historicalSimulation"], true);
    assert_eq!(first["data"]["result"]["metrics"]["return"], "0.015");
    assert_eq!(first["data"]["result"]["metrics"]["sharpe"], "1.5");
    assert_eq!(first["data"]["result"]["metrics"]["sortino"], "2");
    assert_eq!(first["data"]["result"]["metrics"]["maxDrawdown"], "0.005");
    assert_eq!(first["data"]["result"]["metrics"]["winRate"], "1");
    assert_eq!(first["data"]["result"]["metrics"]["profitFactor"], "2");
    assert_eq!(first["data"]["result"]["metrics"]["turnover"], "0.01");
    assert_eq!(first["data"]["result"]["metrics"]["tradeCount"], 1);
    assert_eq!(
        first["data"]["result"]["equityCurve"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        first["data"]["result"]["trades"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        first["data"]["result"]["manifest"]["guardChecks"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert!(
        first["data"]["result"]["resultHash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );

    let second = command(&mut control, "backtest.run", success_request);
    assert_eq!(second["data"]["state"], "COMPLETED", "{second}");
    assert_eq!(second["data"]["requestHash"], first["data"]["requestHash"]);
    assert_eq!(second["data"]["result"], first["data"]["result"]);
    assert_ne!(second["data"]["runId"], first["data"]["runId"]);

    let mut guard_request = request(workspace_id, version_id);
    guard_request["fixtureScenario"] = json!("DATA_GAP");
    let guard = command(&mut control, "backtest.run", guard_request);
    assert_eq!(guard["data"]["state"], "FAILED", "{guard}");
    assert_eq!(guard["data"]["failure"]["code"], "BACKTEST_DATA_GAP");

    let run_id = first["data"]["runId"].as_str().unwrap();
    let database = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    let mut projection: Value = database
        .query_row(
            "SELECT projection FROM backtest_runs WHERE run_id=?1",
            [run_id],
            |row| row.get::<_, String>(0),
        )
        .map(|value| serde_json::from_str(&value).unwrap())
        .unwrap();
    projection["result"]["metrics"]["return"] = json!("0.016");
    database
        .execute(
            "UPDATE backtest_runs SET projection=?1 WHERE run_id=?2",
            rusqlite::params![serde_json::to_string(&projection).unwrap(), run_id],
        )
        .unwrap();
    drop(database);
    assert_eq!(
        command(
            &mut control,
            "backtest.get",
            json!({"workspaceId": workspace_id, "runId": run_id}),
        )["error"]["code"],
        "WORKSPACE_INTEGRITY_FAILED"
    );
}
