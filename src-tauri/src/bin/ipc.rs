use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tradex::{
    BacktestSupervisor, ControlPlane, RuntimeSupervisor, StrategySupervisor, protocol::EventSink,
};

#[cfg(feature = "integration-test")]
#[path = "../../tests/support/provider_fixtures.rs"]
mod fixtures;

fn main() -> io::Result<()> {
    #[cfg(feature = "integration-test")]
    let (vault, http) = (fixtures::Vault::default(), fixtures::Http::default());
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("Usage: tradex-ipc <isolated-workspace-directory>");
        std::process::exit(2);
    };
    let control = Arc::new(Mutex::new(ControlPlane::new(PathBuf::from(path))));
    let supervisor = RuntimeSupervisor::new();
    let strategy_supervisor = StrategySupervisor::new();
    let backtest_supervisor = BacktestSupervisor::new();
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let runtime_path =
        std::env::temp_dir().join(format!("tradex-model-ui-{}", uuid::Uuid::new_v4()));
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let mut gateway = tradex::gateway_process::GatewayHost::new(runtime_path.clone());
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let model_vault = tradex::model_credentials::MemoryModelVault::default();
    let output = Arc::new(Mutex::new(io::stdout()));
    let event_output = output.clone();
    let sink: EventSink = Arc::new(move |event| {
        write_frame(&event_output, &json!({"kind":"event", "event":event})).is_ok()
    });
    let mut input = io::stdin().lock();
    let mut frame = Vec::new();
    let mut oversized = false;
    loop {
        let bytes = input.fill_buf()?;
        if bytes.is_empty() {
            break;
        }
        let newline = bytes.iter().position(|b| *b == b'\n');
        let length = newline.map_or(bytes.len(), |n| n + 1);
        if frame.len() + length > 65_536 {
            oversized = true;
        }
        if !oversized {
            frame.extend_from_slice(&bytes[..length]);
        }
        input.consume(length);
        if newline.is_some() {
            let request: Value = if oversized {
                Value::Null
            } else {
                serde_json::from_slice(&frame).unwrap_or(Value::Null)
            };
            let command = request.get("command").and_then(Value::as_str);
            #[cfg(feature = "integration-test")]
            if command == Some("alpaca.paper.stream.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let remote_account_id = payload.get("remoteAccountId").and_then(Value::as_str);
                let stream_frame = payload.get("frame");
                let reply = match (
                    connection_id,
                    state_version,
                    remote_account_id,
                    stream_frame,
                ) {
                    (
                        Some(connection_id),
                        Some(state_version),
                        Some(remote_account_id),
                        Some(frame),
                    ) => match control.lock() {
                        Ok(mut control) => match control.apply_alpaca_private_stream_frame(
                            connection_id,
                            state_version,
                            remote_account_id,
                            frame,
                            &[fixtures::KEY.into(), fixtures::SECRET.into()],
                        ) {
                            Ok(()) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("alpaca.paper.stream.disconnect.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let reply = match (connection_id, state_version) {
                    (Some(connection_id), Some(state_version)) => match control.lock() {
                        Ok(mut control) => match control.mark_alpaca_private_stream_degraded(
                            connection_id,
                            state_version,
                            "DEGRADED",
                            "Alpaca Paper private stream fixture disconnected.",
                        ) {
                            Ok(()) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.testnet.stream.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let remote_account_id = payload.get("remoteAccountId").and_then(Value::as_str);
                let subscription_id = payload.get("subscriptionId").and_then(Value::as_u64);
                let stream_frame = payload.get("frame");
                let reply = match (
                    connection_id,
                    state_version,
                    remote_account_id,
                    subscription_id,
                    stream_frame,
                ) {
                    (
                        Some(connection_id),
                        Some(state_version),
                        Some(remote_account_id),
                        Some(subscription_id),
                        Some(stream_frame),
                    ) => match control.lock() {
                        Ok(mut control) => match control.apply_binance_private_stream_frame(
                            connection_id,
                            state_version,
                            remote_account_id,
                            stream_frame,
                            subscription_id,
                            &[fixtures::KEY.into(), fixtures::SECRET.into()],
                        ) {
                            Ok(needs_reconciliation) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true,"needsReconciliation":needs_reconciliation}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.testnet.stream.disconnect.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let reply = match (connection_id, state_version) {
                    (Some(connection_id), Some(state_version)) => match control.lock() {
                        Ok(mut control) => match control.update_binance_private_stream_health(
                            connection_id,
                            state_version,
                            "DEGRADED",
                            "DEGRADED",
                            "Binance Spot Testnet private stream fixture disconnected.",
                        ) {
                            Ok(()) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.testnet.stream.reconcile.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let remote_account_id = payload.get("remoteAccountId").and_then(Value::as_str);
                let reply = match (connection_id, state_version, remote_account_id) {
                    (Some(connection_id), Some(state_version), Some(remote_account_id)) => {
                        match control.lock() {
                            Ok(mut control) => {
                                match control.binance_private_stream_order_book(connection_id) {
                                    Ok(book) => {
                                        http.mirror_binance_private_stream_book(&book);
                                        match control.reconcile_binance_private_stream_fixture(
                                            connection_id,
                                            state_version,
                                            remote_account_id,
                                            &[fixtures::KEY.into(), fixtures::SECRET.into()],
                                            &http,
                                        ) {
                                            Ok(()) => json!({
                                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                                "data":{"accepted":true,"reconciled":true}
                                            }),
                                            Err(error) => json!({
                                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                                "error":error
                                            }),
                                        }
                                    }
                                    Err(error) => json!({
                                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                        "error":error
                                    }),
                                }
                            }
                            Err(_) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                            }),
                        }
                    }
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("account.delete.fixture.seed") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let label = payload.get("label").and_then(Value::as_str);
                let reply = match (workspace_id, label) {
                    (Some(workspace_id), Some(label)) => match control.lock() {
                        Ok(mut control) => match control
                            .seed_trading212_demo_deletion_fixture(workspace_id, label)
                        {
                            Ok(account) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":account
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("workspace.open") {
                supervisor.stop_all();
                strategy_supervisor.stop_all();
                backtest_supervisor.stop_all();
            }
            if command == Some("turn.start") || command == Some("turn.retry") {
                let result = if command == Some("turn.start") {
                    supervisor.start(control.clone(), request, None)
                } else {
                    supervisor.retry(control.clone(), request, None)
                };
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("turn.cancel") {
                let result = supervisor.cancel(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("strategy.run") {
                let result = strategy_supervisor.start(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("strategy.cancel") {
                let result = strategy_supervisor.cancel(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("backtest.run") {
                let result = backtest_supervisor.start(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("backtest.cancel") {
                let result = backtest_supervisor.cancel(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("data.source.probe") {
                let prepared = match control.lock() {
                    Ok(mut control) => control.prepare_data_source_probe(&request),
                    Err(_) => Err(tradex::protocol::TradeXError::new(
                        "IPC_CONTROL_PLANE_UNAVAILABLE",
                    )),
                };
                let result = match prepared {
                    Ok(Some(job)) => {
                        let outcome =
                            tradex::data_sources::probe(&job.input.source_id, job.source.clone());
                        match control.lock() {
                            Ok(mut control) => control.complete_data_source_probe(&job, outcome),
                            Err(_) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                            }),
                        }
                    }
                    Ok(None) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_COMMAND_UNKNOWN")
                    }),
                    Err(error) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":error
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            let result = match control.lock() {
                Ok(mut control) => match control.prepare_provider_for(&request, "stdio") {
                    Ok(Some(job)) => {
                        let outcome = job.run(
                            &vault,
                            |definition| {
                                if definition.provider_id == "bitget" {
                                    fixtures::bitget::credentials()
                                } else {
                                    fixtures::credentials()
                                }
                            },
                            &http,
                            || control.provider_job_current(&job),
                        );
                        let reply = control.complete_provider(&job, outcome);
                        if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, &vault) {
                            control.record_credential_cleanup(&job, cleanup);
                        }
                        reply
                    }
                    Ok(None) | Err(_) => {
                        control.dispatch_with_events(request.clone(), "stdio", Some(sink.clone()))
                    }
                },
                Err(_) => json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                    "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                }),
            };
            #[cfg(not(feature = "integration-test"))]
            let result = match control.lock() {
                Ok(mut control) => {
                    control.dispatch_with_events(request.clone(), "stdio", Some(sink.clone()))
                }
                Err(_) => json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                    "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                }),
            };
            #[cfg(all(feature = "integration-test", target_os = "macos"))]
            let result = match request.get("command").and_then(Value::as_str) {
                Some("model.gateway") => match control.lock() {
                    Ok(mut control) => match control.prepare_gateway(&request) {
                        Ok(Some(job)) => {
                            let outcome = gateway.run(&job, || control.gateway_job_current(&job));
                            control.complete_gateway(&job, outcome)
                        }
                        Ok(None) => result,
                        Err(error) => {
                            json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":error})
                        }
                    },
                    Err(_) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                    }),
                },
                Some("model.configure_deepseek") | Some("model.verify_route") => {
                    match control.lock() {
                        Ok(mut control) => match control.prepare_model(&request) {
                            Ok(Some(job)) => {
                                let outcome = tradex::model::run_job(
                                    &job,
                                    &mut gateway,
                                    &model_vault,
                                    || {
                                        tradex::model_credentials::ModelKey::new(
                                            "integration-test-key".into(),
                                        )
                                    },
                                    || control.model_job_current(&job),
                                );
                                control.complete_model(&job, outcome)
                            }
                            Ok(None) => result,
                            Err(error) => {
                                json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":error})
                            }
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    }
                }
                Some("model.login_chatgpt") => json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                    "error":tradex::protocol::TradeXError::new("MODEL_NATIVE_REQUIRED")
                }),
                _ => result,
            };
            write_frame(&output, &json!({"kind":"result", "result":result}))?;
            frame.clear();
            oversized = false;
        }
    }
    supervisor.stop_all();
    backtest_supervisor.stop_all();
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    if gateway.stop() {
        let _ = std::fs::remove_dir_all(runtime_path);
    }
    Ok(())
}

fn write_frame(output: &Mutex<io::Stdout>, frame: &Value) -> io::Result<()> {
    let mut output = output
        .lock()
        .map_err(|_| io::Error::other("output unavailable"))?;
    serde_json::to_writer(&mut *output, frame)?;
    output.write_all(b"\n")?;
    output.flush()
}
