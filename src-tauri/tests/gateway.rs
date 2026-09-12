use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(cp: &mut ControlPlane, name: &str, input: Value) -> Value {
    cp.dispatch(json!({"requestId":"gateway-test", "schemaVersion":1,
        "command":name, "payload":input}))
}

#[test]
fn gateway_state_is_workspace_scoped_persistent_and_not_model_readiness() {
    let dir = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(dir.path().into());
    let opened = command(&mut cp, "workspace.open", json!({}));
    let workspace = opened["data"]["workspaceId"].as_str().unwrap();
    let state = command(
        &mut cp,
        "model.get_gateway",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(state["ok"], true, "{state}");
    assert_eq!(state["data"]["status"], "STOPPED");
    assert_eq!(state["data"]["pinnedVersion"], "7.2.155");
    assert_eq!(state["data"]["endpoint"], "http://127.0.0.1:8317");
    assert_eq!(state["data"]["modelAvailable"], false);
    assert_eq!(state["data"]["installed"], false);
    let snapshot = command(
        &mut cp,
        "domain.snapshot",
        json!({
        "aggregateType":"model-gateway", "aggregateId":workspace}),
    );
    assert_eq!(snapshot["data"]["projection"], state["data"]);
    let replayed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let replay_sink = replayed.clone();
    let ack = cp.dispatch_with_events(
        json!({
            "requestId":"gateway-subscribe", "schemaVersion":1,
            "command":"domain.subscribe",
            "payload":{"aggregateType":"model-gateway","aggregateId":workspace,"afterSequence":0}
        }),
        "gateway-test",
        Some(std::sync::Arc::new(move |event| {
            replay_sink.lock().unwrap().push(event);
            true
        })),
    );
    assert_eq!(ack["ok"], true, "{ack}");
    assert_eq!(ack["data"]["replayedCount"], 1);
    assert_eq!(
        replayed.lock().unwrap()[0].event_type,
        "model.gateway.changed"
    );
    let invalid = command(
        &mut cp,
        "model.get_gateway",
        json!({"workspaceId":workspace,"key":"not-allowed"}),
    );
    assert_eq!(invalid["error"]["code"], "IPC_PAYLOAD_INVALID");
    let foreign = command(
        &mut cp,
        "model.get_gateway",
        json!({"workspaceId":"foreign"}),
    );
    assert_eq!(foreign["error"]["code"], "IPC_AGGREGATE_NOT_FOUND");
    drop(cp);
    let mut cp = ControlPlane::new(dir.path().into());
    let again = command(&mut cp, "workspace.open", json!({}));
    assert_eq!(again["data"]["workspaceId"], workspace);
    let restored = command(
        &mut cp,
        "model.get_gateway",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(restored["data"]["modelAvailable"], false);
    assert_eq!(restored["data"]["lastProbeAt"], Value::Null);
    assert_eq!(restored["data"]["status"], "STOPPED");
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "runs the pinned native gateway on port 8317; requires isolated serial execution"]
fn pinned_gateway_lifecycle_uses_public_commands_without_model_readiness() {
    use tradex::gateway_process::GatewayHost;
    let workspace_dir = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let pinned = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../.artifacts/s03-planning/upstream/cli-proxy-api");
    std::fs::copy(pinned, runtime.path().join("cli-proxy-api-7.2.155")).unwrap();
    let mut cp = ControlPlane::new(workspace_dir.path().into());
    let opened = command(&mut cp, "workspace.open", json!({}));
    let id = opened["data"]["workspaceId"].as_str().unwrap();
    let mut host = GatewayHost::new(runtime.path().into());
    let mut run = |action: &str| {
        let state = command(&mut cp, "model.get_gateway", json!({"workspaceId":id}));
        let request = json!({"schemaVersion":1,"requestId":"native-gateway", "command":"model.gateway",
            "payload":{"workspaceId":id,"expectedStateVersion":state["data"]["stateVersion"],"action":action}});
        let job = cp.prepare_gateway(&request).unwrap().unwrap();
        let outcome = host.run(&job, || cp.gateway_job_current(&job));
        cp.complete_gateway(&job, outcome)
    };
    let occupied = std::net::TcpListener::bind("127.0.0.1:8317").unwrap();
    let blocked = run("LAUNCH");
    assert_eq!(blocked["data"]["status"], "PORT_CONFLICT", "{blocked}");
    assert_eq!(blocked["data"]["modelAvailable"], false);
    assert!(occupied.local_addr().is_ok());
    drop(occupied);
    let started = run("LAUNCH");
    assert_eq!(started["data"]["status"], "RUNNING", "{started}");
    assert_eq!(started["data"]["discoveredModelCount"], 0);
    assert_eq!(started["data"]["modelAvailable"], false);
    assert!(started["data"]["lastProbeAt"].is_string());
    let probed = run("PROBE");
    assert_eq!(probed["data"]["status"], "RUNNING");
    let stopped = run("STOP");
    assert_eq!(stopped["data"]["status"], "STOPPED");
    assert_eq!(stopped["data"]["lastProbeAt"], Value::Null);
    assert_eq!(stopped["data"]["discoveredModelCount"], 0);
    assert!(!runtime.path().join(id).join("gateway.json").exists());
    assert!(std::net::TcpListener::bind("127.0.0.1:8317").is_ok());
    std::fs::write(runtime.path().join(id).join("gateway.json"), "stale").unwrap();
    let occupied = std::net::TcpListener::bind("127.0.0.1:8317").unwrap();
    let blocked = run("LAUNCH");
    assert_eq!(blocked["data"]["status"], "PORT_CONFLICT");
    assert!(!runtime.path().join(id).join("gateway.json").exists());
    drop(occupied);
    assert_eq!(run("LAUNCH")["data"]["status"], "RUNNING");
    for crash in 0..4 {
        let pid = std::process::Command::new("/usr/sbin/lsof")
            .args(["-t", "-iTCP:8317", "-sTCP:LISTEN"])
            .output()
            .unwrap();
        let pid = String::from_utf8(pid.stdout).unwrap();
        let pid = pid.trim();
        assert!(!pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()));
        let image = std::process::Command::new("/usr/sbin/lsof")
            .args(["-a", "-p", pid, "-d", "txt", "-Fn"])
            .output()
            .unwrap();
        let expected = runtime
            .path()
            .join("cli-proxy-api-7.2.155")
            .canonicalize()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&image.stdout)
                .lines()
                .any(|line| line == format!("n{}", expected.display()))
        );
        assert!(
            std::process::Command::new("/bin/kill")
                .args(["-KILL", pid])
                .status()
                .unwrap()
                .success()
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
        let job = cp.gateway_monitor_job().unwrap();
        let outcome = host.monitor(&job, || cp.gateway_job_current(&job)).unwrap();
        let changed = cp.complete_gateway(&job, outcome);
        assert_eq!(changed["data"]["modelAvailable"], false);
        if crash == 3 {
            assert_eq!(changed["data"]["status"], "FAILED");
            assert_eq!(changed["data"]["restartAttempts"], 3);
            assert_eq!(changed["data"]["nextRetryAt"], Value::Null);
            break;
        }
        assert_eq!(changed["data"]["status"], "BACKOFF");
        assert!(changed["data"]["nextRetryAt"].is_string());
        // Account/control-plane reads do not wait on the recovery timer.
        assert_eq!(
            command(&mut cp, "account.list", json!({"workspaceId":id}))["ok"],
            true
        );
        std::thread::sleep(std::time::Duration::from_millis((1 << crash) * 1000 + 50));
        let job = cp.gateway_monitor_job().unwrap();
        let starting = host.monitor(&job, || cp.gateway_job_current(&job)).unwrap();
        assert_eq!(
            cp.complete_gateway(&job, starting)["data"]["status"],
            "STARTING"
        );
        let job = cp.gateway_monitor_job().unwrap();
        let running = host.monitor(&job, || cp.gateway_job_current(&job)).unwrap();
        let reply = cp.complete_gateway(&job, running);
        assert_eq!(reply["data"]["status"], "RUNNING", "{reply}");
        assert_eq!(reply["data"]["errorCode"], Value::Null);
        assert_eq!(reply["data"]["nextRetryAt"], Value::Null);
    }
    assert!(!runtime.path().join(id).join("gateway.json").exists());
}
