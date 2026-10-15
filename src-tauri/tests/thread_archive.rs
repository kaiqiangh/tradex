use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({"requestId":"thread-archive-test","schemaVersion":1,"command":name,"payload":payload}))
}
fn unchanged_fields(mut thread: Value) -> Value {
    for field in ["status", "updatedAt", "stateVersion"] {
        thread.as_object_mut().unwrap().remove(field);
    }
    thread
}

#[test]
fn public_thread_archive_restore_preserves_history_and_replays_after_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let workspace = command(&mut control, "workspace.open", json!({}))["data"].clone();
    let wid = &workspace["workspaceId"];
    let created = command(
        &mut control,
        "thread.create",
        json!({
            "workspaceId":wid,"title":"Saved research","defaultAgentMode":"RESEARCH",
            "defaultExecutionContext":"NONE_READ_ONLY","linkedContexts":[]
        }),
    );
    assert_eq!(created["ok"], true, "{created}");
    let original = created["data"].clone();
    let tid = &original["threadId"];
    let archived = command(
        &mut control,
        "thread.archive",
        json!({
            "workspaceId":wid,"threadId":tid,"expectedStateVersion":original["stateVersion"]
        }),
    );
    assert_eq!(archived["ok"], true, "{archived}");
    assert_eq!(archived["data"]["status"], "ARCHIVED");
    assert_eq!(
        unchanged_fields(archived["data"].clone()),
        unchanged_fields(original.clone())
    );
    assert_ne!(archived["data"]["stateVersion"], original["stateVersion"]);
    drop(control);
    let mut control = ControlPlane::new(path);
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let listed = command(&mut control, "thread.list", json!({"workspaceId":wid}));
    assert_eq!(listed["data"]["threads"][0]["status"], "ARCHIVED");
    let events = Arc::new(Mutex::new(Vec::new()));
    let captured = events.clone();
    let subscribed = control.dispatch_with_events(
        json!({"requestId":"subscribe","schemaVersion":1,"command":"domain.subscribe","payload":{
            "aggregateType":"thread","aggregateId":tid,"afterSequence":0
        }}),
        "archive-test",
        Some(Arc::new(move |event| {
            captured.lock().unwrap().push(event);
            true
        })),
    );
    assert_eq!(subscribed["data"]["replayedCount"], 2, "{subscribed}");
    let restored = command(
        &mut control,
        "thread.restore",
        json!({
            "workspaceId":wid,"threadId":tid,"expectedStateVersion":archived["data"]["stateVersion"]
        }),
    );
    assert_eq!(restored["ok"], true, "{restored}");
    assert_eq!(restored["data"]["status"], "ACTIVE");
    assert_eq!(
        unchanged_fields(restored["data"].clone()),
        unchanged_fields(original.clone())
    );
    assert_eq!(events.lock().unwrap().len(), 3);
    assert_eq!(
        command(
            &mut control,
            "domain.snapshot",
            json!({"aggregateType":"thread","aggregateId":tid})
        )["data"]["lastSequence"],
        3
    );
}

#[test]
fn archived_threads_refuse_sync_and_async_start_retry_without_events() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let wid = command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let created = command(
        &mut control,
        "thread.create",
        json!({"workspaceId":wid,"title":"Read only Saved","defaultAgentMode":"ASK","defaultExecutionContext":"NONE_READ_ONLY","linkedContexts":[]}),
    );
    let tid = created["data"]["threadId"].clone();
    let archived = command(
        &mut control,
        "thread.archive",
        json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":created["data"]["stateVersion"]}),
    );
    assert_eq!(archived["ok"], true, "{archived}");
    let start = json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":archived["data"]["stateVersion"],"message":"Must not run","agentMode":"ASK","executionContext":"NONE_READ_ONLY","attachedContexts":[]});
    let retry = json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":archived["data"]["stateVersion"],"turnId":"missing-turn"});
    for (name, payload) in [("turn.start", start.clone()), ("turn.retry", retry.clone())] {
        let refused = command(&mut control, name, payload);
        assert_eq!(refused["error"]["code"], "THREAD_ARCHIVED", "{refused}");
    }
    let control = Arc::new(Mutex::new(control));
    let supervisor = tradex::RuntimeSupervisor::new();
    for (name, payload) in [("turn.start", start), ("turn.retry", retry)] {
        let request =
            json!({"requestId":"async-archive","schemaVersion":1,"command":name,"payload":payload});
        let refused = if name == "turn.start" {
            supervisor.start(control.clone(), request, None)
        } else {
            supervisor.retry(control.clone(), request, None)
        };
        assert_eq!(refused["error"]["code"], "THREAD_ARCHIVED", "{refused}");
    }
    let mut control = control.lock().unwrap();
    assert_eq!(
        command(
            &mut control,
            "thread.get",
            json!({"workspaceId":wid,"threadId":tid})
        )["data"],
        archived["data"]
    );
    assert_eq!(
        command(
            &mut control,
            "domain.snapshot",
            json!({"aggregateType":"thread","aggregateId":tid})
        )["data"]["lastSequence"],
        2
    );
}

#[test]
fn lifecycle_refusals_and_outbox_failure_leave_public_snapshot_unchanged() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let wid = command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let original=command(&mut control,"thread.create",json!({"workspaceId":wid,"title":"Atomic history","defaultAgentMode":"ASK","defaultExecutionContext":"NONE_READ_ONLY","linkedContexts":[]}))["data"].clone();
    let tid = &original["threadId"];
    let input =
        json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":original["stateVersion"]});
    let snapshot = command(
        &mut control,
        "domain.snapshot",
        json!({"aggregateType":"thread","aggregateId":tid}),
    )["data"]
        .clone();
    let mut cases = vec![("thread.restore", input.clone(), "THREAD_STATUS_CONFLICT")];
    for (field, value, code) in [
        (
            "expectedStateVersion",
            json!("stale"),
            "STATE_VERSION_CONFLICT",
        ),
        ("expectedStateVersion", json!(""), "IPC_PAYLOAD_INVALID"),
        (
            "expectedStateVersion",
            json!("x".repeat(257)),
            "IPC_PAYLOAD_INVALID",
        ),
        (
            "workspaceId",
            json!("other-workspace"),
            "IPC_AGGREGATE_NOT_FOUND",
        ),
        (
            "threadId",
            json!("missing-thread"),
            "IPC_AGGREGATE_NOT_FOUND",
        ),
    ] {
        let mut bad = input.clone();
        bad[field] = value;
        cases.push(("thread.archive", bad, code));
    }
    let mut extra = input.clone();
    extra["unknown"] = json!(true);
    cases.push(("thread.archive", extra, "IPC_PAYLOAD_INVALID"));
    let mut missing = input.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("expectedStateVersion");
    cases.push(("thread.archive", missing, "IPC_PAYLOAD_INVALID"));
    for (name, payload, code) in cases {
        let result = command(&mut control, name, payload);
        assert_eq!(result["error"]["code"], code, "{result}");
        assert_eq!(
            command(
                &mut control,
                "domain.snapshot",
                json!({"aggregateType":"thread","aggregateId":tid})
            )["data"],
            snapshot
        );
    }
    // Fault only the real database's outbox insertion, then verify via public IPC.
    let database = rusqlite::Connection::open(path.join("workspace.sqlite3")).unwrap();
    database.execute_batch("CREATE TRIGGER fail_thread_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'synthetic write fault'); END;").unwrap();
    let refused = command(&mut control, "thread.archive", input.clone());
    assert_eq!(refused["ok"], false, "{refused}");
    assert_eq!(
        command(
            &mut control,
            "domain.snapshot",
            json!({"aggregateType":"thread","aggregateId":tid})
        )["data"],
        snapshot
    );
    database
        .execute_batch("DROP TRIGGER fail_thread_outbox;")
        .unwrap();
    let archived = command(&mut control, "thread.archive", input.clone());
    assert_eq!(archived["ok"], true, "{archived}");
    let duplicate = command(
        &mut control,
        "thread.archive",
        json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":archived["data"]["stateVersion"]}),
    );
    assert_eq!(duplicate["error"]["code"], "THREAD_STATUS_CONFLICT");
    let stale_restore = command(&mut control, "thread.restore", input);
    assert_eq!(stale_restore["error"]["code"], "STATE_VERSION_CONFLICT");
    assert_eq!(
        command(
            &mut control,
            "thread.get",
            json!({"workspaceId":wid,"threadId":tid})
        )["data"],
        archived["data"]
    );
    assert_eq!(
        command(
            &mut control,
            "domain.snapshot",
            json!({"aggregateType":"thread","aggregateId":tid})
        )["data"]["lastSequence"],
        2
    );
}

#[cfg(feature = "integration-test")]
#[test]
fn running_turn_wins_archive_race_and_completed_history_keeps_upstream_identity() {
    assert!(
        std::env::var_os("TRADEX_CODEX_APP_SERVER").is_none(),
        "This test requires the local synthetic runtime, never an external process"
    );
    let directory = tempfile::tempdir().unwrap();
    let mut engine = ControlPlane::new(directory.path().join("workspace"));
    let wid = command(&mut engine, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let original=command(&mut engine,"thread.create",json!({"workspaceId":wid,"title":"Running archive race","defaultAgentMode":"ASK","defaultExecutionContext":"NONE_READ_ONLY","model":{"provider":"CHATGPT","modelId":"gpt-5.6-sol"},"linkedContexts":[]}))["data"].clone();
    let tid = &original["threadId"];
    let control = Arc::new(Mutex::new(engine));
    let supervisor = tradex::RuntimeSupervisor::new();
    let started=supervisor.start(control.clone(),json!({"requestId":"running-first","schemaVersion":1,"command":"turn.start","payload":{
        "workspaceId":wid,"threadId":tid,"expectedStateVersion":original["stateVersion"],"message":"Synthetic read-only timeline","agentMode":"ASK","executionContext":"NONE_READ_ONLY","model":{"provider":"CHATGPT","modelId":"gpt-5.6-sol"},"attachedContexts":[]
    }}),None);
    assert_eq!(started["ok"], true, "{started}");
    {
        let mut engine = control.lock().unwrap();
        let before = command(
            &mut engine,
            "domain.snapshot",
            json!({"aggregateType":"thread","aggregateId":tid}),
        )["data"]
            .clone();
        assert_eq!(before["projection"]["turns"][0]["status"], "RUNNING");
        let refused = command(
            &mut engine,
            "thread.archive",
            json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":before["projection"]["stateVersion"]}),
        );
        assert_eq!(
            refused["error"]["code"], "TURN_ALREADY_RUNNING",
            "{refused}"
        );
        assert_eq!(
            command(
                &mut engine,
                "domain.snapshot",
                json!({"aggregateType":"thread","aggregateId":tid})
            )["data"],
            before
        );
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let completed = loop {
        let current = command(
            &mut control.lock().unwrap(),
            "thread.get",
            json!({"workspaceId":wid,"threadId":tid}),
        )["data"]
            .clone();
        if current["turns"][0]["status"] != "RUNNING" {
            break current;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Synthetic runtime did not finish"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(completed["turns"][0]["status"], "COMPLETED");
    assert!(completed["codexThreadId"].as_str().is_some());
    assert!(
        !completed["turns"][0]["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut engine = control.lock().unwrap();
    let archived = command(
        &mut engine,
        "thread.archive",
        json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":completed["stateVersion"]}),
    );
    assert_eq!(archived["ok"], true, "{archived}");
    assert_eq!(
        unchanged_fields(archived["data"].clone()),
        unchanged_fields(completed.clone())
    );
    let restored = command(
        &mut engine,
        "thread.restore",
        json!({"workspaceId":wid,"threadId":tid,"expectedStateVersion":archived["data"]["stateVersion"]}),
    );
    assert_eq!(restored["ok"], true, "{restored}");
    assert_eq!(
        unchanged_fields(restored["data"].clone()),
        unchanged_fields(completed)
    );
}
