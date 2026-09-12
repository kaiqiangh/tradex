use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": "model-test", "schemaVersion": 1,
        "command": name, "payload": payload
    }))
}

#[test]
fn model_state_is_workspace_scoped_and_replayed_as_sanitized_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
    let state = command(
        &mut control,
        "model.get",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(state["ok"], true, "{state}");
    assert_eq!(state["data"]["chatgpt"]["status"], "NOT_CONFIGURED");
    assert_eq!(state["data"]["deepseek"]["status"], "NOT_CONFIGURED");
    assert_eq!(state["data"]["attempts"].as_array().unwrap().len(), 0);
    let snapshot = command(
        &mut control,
        "domain.snapshot",
        json!({"aggregateType":"model", "aggregateId":workspace_id}),
    );
    assert_eq!(snapshot["data"]["projection"], state["data"]);
    let events = Arc::new(Mutex::new(Vec::new()));
    let target = events.clone();
    let ack = control.dispatch_with_events(
        json!({"requestId":"model-subscribe", "schemaVersion":1, "command":"domain.subscribe",
            "payload":{"aggregateType":"model","aggregateId":workspace_id,"afterSequence":0}}),
        "model-test",
        Some(Arc::new(move |event| {
            target.lock().unwrap().push(event);
            true
        })),
    );
    assert_eq!(ack["ok"], true, "{ack}");
    assert_eq!(ack["data"]["replayedCount"], 1);
    let event = &events.lock().unwrap()[0];
    assert_eq!(event.event_type, "model.provider.changed");
    let encoded = serde_json::to_string(event).unwrap();
    assert!(!encoded.contains("integration-test-key"));
    assert!(!encoded.contains("api.deepseek.com"));
    let foreign = command(&mut control, "model.get", json!({"workspaceId":"foreign"}));
    assert_eq!(foreign["error"]["code"], "IPC_AGGREGATE_NOT_FOUND");
    let invalid = command(
        &mut control,
        "model.get",
        json!({"workspaceId": workspace_id, "key":"secret"}),
    );
    assert_eq!(invalid["error"]["code"], "IPC_PAYLOAD_INVALID");
}

#[test]
fn model_mutations_require_native_boundary_and_exact_routes() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let state = command(
        &mut control,
        "model.get",
        json!({"workspaceId":workspace_id}),
    );
    let version = state["data"]["stateVersion"].as_str().unwrap();
    let native = command(
        &mut control,
        "model.configure_deepseek",
        json!({"workspaceId":workspace_id,"expectedStateVersion":version}),
    );
    assert_eq!(native["error"]["code"], "MODEL_NATIVE_REQUIRED");
    let native_login = command(
        &mut control,
        "model.login_chatgpt",
        json!({"workspaceId":workspace_id,"expectedStateVersion":version,"action":"LOGIN"}),
    );
    assert_eq!(native_login["error"]["code"], "MODEL_NATIVE_REQUIRED");
    let invalid_payload = command(
        &mut control,
        "model.configure_deepseek",
        json!({"workspaceId":workspace_id,"expectedStateVersion":version,"key":"secret"}),
    );
    assert_eq!(invalid_payload["error"]["code"], "IPC_PAYLOAD_INVALID");
    let invalid = control.prepare_model(&json!({
        "requestId":"route", "schemaVersion":1, "command":"model.verify_route",
        "payload":{"workspaceId":workspace_id,"expectedStateVersion":version,"provider":"DEEPSEEK","modelId":"deepseek-chat","thinkingType":"disabled"}
    }));
    assert_eq!(invalid.err().unwrap().code, "MODEL_ROUTE_INVALID");
    let stale = control.prepare_model(&json!({
        "requestId":"route", "schemaVersion":1, "command":"model.verify_route",
        "payload":{"workspaceId":workspace_id,"expectedStateVersion":"stale","provider":"DEEPSEEK","modelId":"deepseek-v4-flash","thinkingType":"disabled"}
    }));
    assert_eq!(stale.err().unwrap().code, "STATE_VERSION_CONFLICT");
}

#[test]
fn model_failures_keep_canonical_attempt_categories() {
    assert_eq!(
        tradex::protocol::TradeXError::new("MODEL_TEST_INFERENCE_FAILED").category,
        "MODEL_UNAVAILABLE"
    );
    assert_eq!(
        tradex::protocol::TradeXError::new("MODEL_LOGIN_TIMEOUT").category,
        "OAUTH_EXPIRED"
    );
    assert_eq!(
        tradex::protocol::TradeXError::new("MODEL_QUOTA_EXCEEDED").category,
        "QUOTA_EXCEEDED"
    );
    assert_eq!(
        tradex::protocol::TradeXError::new("MODEL_GATEWAY_RUNNING").category,
        "MODEL_UNAVAILABLE"
    );
}

#[test]
fn deepseek_replacement_requires_a_stopped_gateway() {
    use tradex::gateway::{GatewayState, GatewayStatus};

    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let gateway = command(
        &mut control,
        "model.get_gateway",
        json!({"workspaceId":workspace_id}),
    );
    let request = json!({
        "requestId":"gateway-start",
        "schemaVersion":1,
        "command":"model.gateway",
        "payload":{"workspaceId":workspace_id,"expectedStateVersion":gateway["data"]["stateVersion"],"action":"LAUNCH"}
    });
    let job = control.prepare_gateway(&request).unwrap().unwrap();
    let mut running: GatewayState = serde_json::from_value(
        command(
            &mut control,
            "model.get_gateway",
            json!({"workspaceId":workspace_id}),
        )["data"]
            .clone(),
    )
    .unwrap();
    running.status = GatewayStatus::Running;
    running.desired_running = true;
    assert_eq!(control.complete_gateway(&job, running)["ok"], true);

    let model = command(
        &mut control,
        "model.get",
        json!({"workspaceId":workspace_id}),
    );
    let rejected = json!({
        "requestId":"replace",
        "schemaVersion":1,
        "command":"model.configure_deepseek",
        "payload":{"workspaceId":workspace_id,"expectedStateVersion":model["data"]["stateVersion"]}
    });
    let error = match control.prepare_model(&rejected) {
        Err(error) => error,
        Ok(_) => panic!("running gateway must block DeepSeek replacement"),
    };
    assert_eq!(error.code, "MODEL_GATEWAY_RUNNING");

    let gateway = command(
        &mut control,
        "model.get_gateway",
        json!({"workspaceId":workspace_id}),
    );
    let stop_request = json!({
        "requestId":"gateway-stop",
        "schemaVersion":1,
        "command":"model.gateway",
        "payload":{"workspaceId":workspace_id,"expectedStateVersion":gateway["data"]["stateVersion"],"action":"STOP"}
    });
    let _stop_job = control.prepare_gateway(&stop_request).unwrap().unwrap();
    let model = command(
        &mut control,
        "model.get",
        json!({"workspaceId":workspace_id}),
    );
    let stopping_rejected = json!({
        "requestId":"replace-while-stopping",
        "schemaVersion":1,
        "command":"model.configure_deepseek",
        "payload":{"workspaceId":workspace_id,"expectedStateVersion":model["data"]["stateVersion"]}
    });
    let error = match control.prepare_model(&stopping_rejected) {
        Err(error) => error,
        Ok(_) => panic!("stopping gateway must block DeepSeek replacement"),
    };
    assert_eq!(error.code, "MODEL_GATEWAY_RUNNING");
}

#[test]
#[cfg(target_os = "macos")]
fn cancelled_deepseek_entry_does_not_write_keychain_double() {
    use tradex::gateway_process::GatewayHost;
    use tradex::model_credentials::{MemoryModelVault, ModelVault};
    use tradex::protocol::TradeXError;

    let directory = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
    let current = command(
        &mut control,
        "model.get",
        json!({"workspaceId":workspace_id}),
    );
    let request = json!({
        "requestId":"cancel-configure",
        "schemaVersion":1,
        "command":"model.configure_deepseek",
        "payload":{"workspaceId":workspace_id,"expectedStateVersion":current["data"]["stateVersion"]}
    });
    let job = control.prepare_model(&request).unwrap().unwrap();
    let vault = MemoryModelVault::default();
    let mut host = GatewayHost::new(runtime.path().into());
    let outcome = tradex::model::run_job(
        &job,
        &mut host,
        &vault,
        || Err(TradeXError::new("MODEL_ENTRY_CANCELLED")),
        || control.model_job_current(&job),
    );
    assert!(vault.get_deepseek(&workspace_id).is_err());
    let reply = control.complete_model(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["deepseek"]["status"], "NOT_CONFIGURED");
    assert_eq!(
        reply["data"]["attempts"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["outcome"],
        "CANCELLED"
    );
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit native Keychain integration with a disposable synthetic model key"]
fn native_model_keychain_roundtrip_is_workspace_scoped() {
    use tradex::model_credentials::{ModelKey, ModelVault, NativeModelVault};

    let workspace = format!("model-test-{}", uuid::Uuid::new_v4());
    let other = format!("model-test-{}", uuid::Uuid::new_v4());
    let key = ModelKey::new("integration-test-model-key".into()).unwrap();
    NativeModelVault.put_deepseek(&workspace, &key).unwrap();
    assert_eq!(
        NativeModelVault.get_deepseek(&workspace).unwrap().as_str(),
        key.as_str()
    );
    assert!(NativeModelVault.get_deepseek(&other).is_err());
    NativeModelVault.remove_deepseek(&workspace).unwrap();
    assert!(NativeModelVault.get_deepseek(&workspace).is_err());
}
