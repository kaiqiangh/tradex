use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": "integration-request", "schemaVersion": 1,
        "command": name, "payload": payload
    }))
}

#[test]
fn a_workspace_keeps_its_identity_and_events_after_process_state_is_dropped() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("research");
    let mut control = ControlPlane::new(path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
    let created_at = opened["data"]["createdAt"].clone();
    drop(control);

    let mut restarted = ControlPlane::new(path);
    let reopened = command(&mut restarted, "workspace.open", json!({}));
    assert_eq!(reopened["data"]["workspaceId"], id);
    assert_eq!(reopened["data"]["createdAt"], created_at);
    let snapshot = command(
        &mut restarted,
        "domain.snapshot",
        json!({
            "aggregateType": "workspace", "aggregateId": id
        }),
    );
    assert_eq!(snapshot["ok"], true, "{snapshot}");
    assert_eq!(snapshot["data"]["lastSequence"], 2);
    assert_eq!(snapshot["data"]["projection"], reopened["data"]);
}

#[test]
fn creation_settings_survive_reopen_and_invalid_requests_do_not_change_state() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(
        &mut control,
        "workspace.open",
        json!({"name":"Equity research", "baseCurrency":"EUR"}),
    );
    assert_eq!(opened["ok"], true, "{opened}");
    assert_eq!(opened["data"]["name"], "Equity research");
    assert_eq!(opened["data"]["baseCurrency"], "EUR");
    let aggregate =
        json!({"aggregateType":"workspace", "aggregateId":opened["data"]["workspaceId"]});
    let before = command(&mut control, "domain.snapshot", aggregate.clone());
    assert_eq!(
        command(
            &mut control,
            "domain.snapshot",
            json!({"aggregateType":"workspace","aggregateId":"missing"})
        )["error"]["code"],
        "IPC_AGGREGATE_NOT_FOUND"
    );
    for (request, code) in [
        (
            json!({"requestId":"bad", "schemaVersion":2,"command":"workspace.open","payload":{}}),
            "IPC_SCHEMA_UNSUPPORTED",
        ),
        (
            json!({"requestId":"bad", "schemaVersion":1,"command":"trade.approve","payload":{}}),
            "IPC_COMMAND_UNKNOWN",
        ),
        (
            json!({"requestId":"bad", "schemaVersion":1,"command":"workspace.open","payload":{"armed":true}}),
            "IPC_PAYLOAD_INVALID",
        ),
        (
            json!({"requestId":"bad", "schemaVersion":1,"command":"workspace.open","payload":{"name":""}}),
            "IPC_PAYLOAD_INVALID",
        ),
        (
            json!({"requestId":"bad", "schemaVersion":1,"command":"workspace.open","payload":{"path":"relative"}}),
            "WORKSPACE_PATH_INVALID",
        ),
    ] {
        let rejected = control.dispatch(request);
        assert_eq!(rejected["error"]["code"], code, "{rejected}");
        assert!(rejected.get("data").is_none());
        assert_eq!(
            command(&mut control, "domain.snapshot", aggregate.clone()),
            before
        );
    }
    let runtime = command(&mut control, "runtime.status", json!({}));
    assert_eq!(runtime["ok"], true, "{runtime}");
    assert_eq!(runtime["data"]["modelAvailable"], false);
    assert_eq!(runtime["data"]["liveExecutionAvailable"], false);
    let reopened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(reopened["data"]["name"], "Equity research");
    assert_eq!(reopened["data"]["baseCurrency"], "EUR");
    for field in ["path", "name", "baseCurrency"] {
        let rejected = command(&mut control, "workspace.open", json!({field:null}));
        assert_eq!(rejected["error"]["code"], "IPC_PAYLOAD_INVALID");
    }
}

#[test]
fn retained_events_join_live_delivery_and_resubscription_preserves_event_identity() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let id = opened["data"]["workspaceId"].clone();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let target = observed.clone();
    let request = json!({"requestId":"subscribe", "schemaVersion":1, "command":"domain.subscribe", "payload":{
        "aggregateType":"workspace", "aggregateId":id, "afterSequence":0
    }});
    let ack = control.dispatch_with_events(
        request.clone(),
        "window-1",
        Some(Arc::new(move |event| {
            target.lock().unwrap().push(event);
            true
        })),
    );
    assert_eq!(ack["ok"], true, "{ack}");
    assert_eq!(ack["data"]["replayedCount"], 1);
    assert_eq!(observed.lock().unwrap()[0].sequence, 1);
    command(&mut control, "workspace.open", json!({}));
    let event_two = serde_json::to_value(&observed.lock().unwrap()[1]).unwrap();

    let resumed = Arc::new(Mutex::new(Vec::new()));
    let resumed_target = resumed.clone();
    let mut again = request.clone();
    again["payload"]["afterSequence"] = 1.into();
    let replay = control.dispatch_with_events(
        again,
        "window-1",
        Some(Arc::new(move |event| {
            resumed_target.lock().unwrap().push(event);
            true
        })),
    );
    assert_eq!(replay["ok"], true, "{replay}");
    assert_eq!(
        serde_json::to_value(&resumed.lock().unwrap()[0]).unwrap(),
        event_two
    );
    command(&mut control, "workspace.open", json!({}));
    assert_eq!(observed.lock().unwrap().len(), 2, "old channel is replaced");
    assert_eq!(resumed.lock().unwrap()[1].sequence, 3);
    assert_eq!(
        resumed.lock().unwrap()[1].aggregate_id,
        id.as_str().unwrap()
    );

    let mut future = request;
    future["payload"]["afterSequence"] = 99.into();
    let rejected = control.dispatch_with_events(future, "window-2", Some(Arc::new(|_| true)));
    assert_eq!(rejected["error"]["code"], "STATE_VERSION_CONFLICT");

    let subscribe = json!({"requestId":"subscribe", "schemaVersion":1, "command":"domain.subscribe", "payload":{
        "aggregateType":"workspace", "aggregateId":id, "afterSequence":0
    }});
    assert_eq!(
        control.dispatch(subscribe.clone())["error"]["code"],
        "IPC_SUBSCRIPTION_CHANNEL_REQUIRED"
    );
    assert_eq!(
        control.dispatch_with_events(subscribe.clone(), "window-2", Some(Arc::new(|_| false)))["error"]
            ["code"],
        "IPC_SUBSCRIPTION_DELIVERY_FAILED"
    );
    let fault =
        rusqlite::Connection::open(directory.path().join("workspace/workspace.sqlite3")).unwrap();
    fault
        .execute("DELETE FROM outbox WHERE sequence=2", [])
        .unwrap();
    assert_eq!(
        control.dispatch_with_events(subscribe, "window-2", Some(Arc::new(|_| true)))["error"]["code"],
        "IPC_REPLAY_UNAVAILABLE"
    );
}

#[test]
fn failed_storage_switches_and_outbox_writes_preserve_the_active_workspace() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("active");
    let mut control = ControlPlane::new(path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    let aggregate =
        json!({"aggregateType":"workspace", "aggregateId":opened["data"]["workspaceId"]});
    let before = command(&mut control, "domain.snapshot", aggregate.clone());

    let busy = directory.path().join("busy");
    let mut owner = ControlPlane::new(busy.clone());
    assert_eq!(command(&mut owner, "workspace.open", json!({}))["ok"], true);
    let corrupt = directory.path().join("corrupt");
    std::fs::create_dir(&corrupt).unwrap();
    std::fs::write(corrupt.join("workspace.sqlite3"), b"not a database").unwrap();
    let foreign = directory.path().join("foreign");
    std::fs::create_dir(&foreign).unwrap();
    let foreign_db = rusqlite::Connection::open(foreign.join("workspace.sqlite3")).unwrap();
    foreign_db
        .execute_batch(
            "CREATE TABLE unrelated (value TEXT); INSERT INTO unrelated VALUES ('preserve me');",
        )
        .unwrap();
    let inaccessible = directory.path().join("file");
    std::fs::write(&inaccessible, b"preserve me").unwrap();
    for (target, code) in [
        (&busy, "WORKSPACE_BUSY"),
        (&corrupt, "WORKSPACE_INTEGRITY_FAILED"),
        (&foreign, "WORKSPACE_SCHEMA_UNSUPPORTED"),
        (&inaccessible, "WORKSPACE_OPEN_FAILED"),
    ] {
        let rejected = command(&mut control, "workspace.open", json!({"path":target}));
        assert_eq!(rejected["error"]["code"], code, "{rejected}");
        assert_eq!(
            command(&mut control, "domain.snapshot", aggregate.clone()),
            before
        );
    }
    assert_eq!(
        std::fs::read(corrupt.join("workspace.sqlite3")).unwrap(),
        b"not a database"
    );
    assert_eq!(
        foreign_db
            .query_row("SELECT value FROM unrelated", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "preserve me"
    );

    // Force the durable event write to fail after the workspace UPDATE has run.
    let fault = rusqlite::Connection::open(path.join("workspace.sqlite3")).unwrap();
    fault.execute_batch("CREATE TRIGGER reject_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT, 'injected storage fault'); END;").unwrap();
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["error"]["code"],
        "WORKSPACE_OPEN_FAILED"
    );
    assert_eq!(
        command(&mut control, "domain.snapshot", aggregate.clone()),
        before
    );
    fault.execute_batch("DROP TRIGGER reject_outbox;").unwrap();
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    assert_eq!(
        command(&mut control, "domain.snapshot", aggregate)["data"]["lastSequence"],
        2
    );
}

#[test]
fn upgrades_backup_recognized_storage_and_never_downgrade_a_newer_schema() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace.sqlite3");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("PRAGMA application_id=1414682673; PRAGMA user_version=0;")
        .unwrap();
    drop(connection);
    let mut control = ControlPlane::new(directory.path().to_path_buf());
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let backups: Vec<_> = std::fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("before-migration-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = rusqlite::Connection::open(&backups[0]).unwrap();
    assert_eq!(
        backup
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        0
    );
    drop(control);
    let future = rusqlite::Connection::open(path).unwrap();
    future.pragma_update(None, "user_version", 2).unwrap();
    let mut control = ControlPlane::new(directory.path().to_path_buf());
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["error"]["code"],
        "WORKSPACE_SCHEMA_UNSUPPORTED"
    );
    assert_eq!(
        future
            .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
            .unwrap(),
        2
    );
}
