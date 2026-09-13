#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tauri::{Manager, ipc::Channel};
use tradex::{
    ControlPlane, RuntimeSupervisor, data_sources,
    gateway_process::GatewayHost,
    model,
    model_credentials::{ModelVault, NativeModelVault},
    native_credentials,
    protocol::{DomainEvent, TradeXError},
    provider_io::{BrokerHttp, NativeVault},
};

struct Service(
    Arc<Mutex<ControlPlane>>,
    Arc<Mutex<GatewayHost>>,
    RuntimeSupervisor,
    Arc<AtomicBool>,
);

#[tauri::command]
async fn control(
    request: Value,
    events: Channel<DomainEvent>,
    window: tauri::WebviewWindow,
    service: tauri::State<'_, Service>,
) -> Result<Value, ()> {
    let consumer = window.label().to_owned();
    let trusted = window.url().is_ok_and(|url| {
        (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
            || (url.scheme() == "http" && url.host_str() == Some("tauri.localhost"))
            || (cfg!(debug_assertions)
                && url.scheme() == "http"
                && url.host_str() == Some("127.0.0.1")
                && url.port() == Some(1420))
    });
    if consumer != "main" || !trusted {
        return Ok(failed(&request, "IPC_ACCESS_DENIED"));
    }
    let engine = service.0.clone();
    let gateway = service.1.clone();
    let supervisor = service.2.clone();
    let fallback = request.clone();
    Ok(tauri::async_runtime::spawn_blocking(move || {
        if request.get("command").and_then(Value::as_str) == Some("data.source.probe") {
            let job = match engine.lock() {
                Ok(mut engine) => match engine.prepare_data_source_probe(&request) {
                    Ok(Some(job)) => job,
                    Ok(None) => return failed(&request, "IPC_COMMAND_UNKNOWN"),
                    Err(error) => return failed(&request, &error.code),
                },
                Err(_) => return failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
            };
            let outcome = data_sources::probe(&job.input.source_id, job.source.clone());
            return match engine.lock() {
                Ok(mut engine) => engine.complete_data_source_probe(&job, outcome),
                Err(_) => failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
            };
        }
        let model_job = match engine.lock() {
            Ok(mut engine) => match engine.prepare_model(&request) {
                Ok(job) => job,
                Err(error) => return failed(&request, &error.code),
            },
            Err(_) => return failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
        };
        if let Some(job) = model_job {
            let mut host = match gateway.lock() {
                Ok(host) => host,
                Err(_) => return failed(&request, "GATEWAY_PROCESS_FAILED"),
            };
            let outcome = model::run_job(
                &job,
                &mut host,
                &NativeModelVault,
                || tradex::model_credentials::capture(window.app_handle()),
                || {
                    engine
                        .lock()
                        .is_ok_and(|engine| engine.model_job_current(&job))
                },
            );
            let reply = match engine.lock() {
                Ok(mut engine) => engine.complete_model(&job, outcome),
                Err(_) => failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
            };
            if reply["ok"] != true {
                host.stop();
            }
            return reply;
        }
        let gateway_job = match engine.lock() {
            Ok(mut engine) => match engine.prepare_gateway(&request) {
                Ok(job) => job,
                Err(error) => return failed(&request, &error.code),
            },
            Err(_) => return failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
        };
        if let Some(job) = gateway_job {
            let mut host = match gateway.lock() {
                Ok(host) => host,
                Err(_) => return failed(&request, "GATEWAY_PROCESS_FAILED"),
            };
            if let Ok(key) = NativeModelVault.get_deepseek(job.workspace_id()) {
                host.set_deepseek_key(job.workspace_id(), Some(key.as_str()));
            } else {
                host.set_deepseek_key(job.workspace_id(), None);
            }
            let outcome = host.run(&job, || {
                engine
                    .lock()
                    .is_ok_and(|engine| engine.gateway_job_current(&job))
            });
            let reply = match engine.lock() {
                Ok(mut engine) => engine.complete_gateway(&job, outcome),
                Err(_) => failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
            };
            if reply["ok"] != true {
                host.stop();
            }
            return reply;
        }
        if request.get("command").and_then(Value::as_str) == Some("workspace.open") {
            supervisor.stop_all();
        }
        let command = request.get("command").and_then(Value::as_str);
        if command == Some("turn.start") || command == Some("turn.retry") {
            let runtime_access = request
                .get("payload")
                .and_then(|payload| payload.get("workspaceId"))
                .and_then(Value::as_str)
                .and_then(|workspace_id| {
                    gateway
                        .lock()
                        .ok()
                        .and_then(|host| host.codex_runtime_access(workspace_id))
                });
            return if command == Some("turn.start") {
                supervisor.start(engine, request, runtime_access)
            } else {
                supervisor.retry(engine, request, runtime_access)
            };
        }
        if command == Some("turn.cancel") {
            return supervisor.cancel(engine, request);
        }
        let prepared = match engine.lock() {
            Ok(mut engine) => match engine.prepare_provider(&request) {
                Ok(Some(job)) => job,
                Ok(None) => {
                    return engine.dispatch_with_runtime(
                        request,
                        &consumer,
                        Some(Arc::new(move |event| events.send(event).is_ok())),
                        None,
                    );
                }
                Err(error) => return failed(&request, &error.code),
            },
            Err(_) => return failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
        };
        let outcome = prepared.run(
            &NativeVault,
            |schema| native_credentials::capture(window.app_handle(), schema),
            &BrokerHttp::default(),
            || {
                engine
                    .lock()
                    .is_ok_and(|engine| engine.provider_job_current(&prepared))
            },
        );
        let reply = match engine.lock() {
            Ok(mut engine) => engine.complete_provider(&prepared, outcome),
            Err(_) => failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
        };
        if let Some(cleanup) = prepared.cleanup_after_failed_commit(&reply, &NativeVault)
            && let Ok(mut engine) = engine.lock()
        {
            engine.record_credential_cleanup(&prepared, cleanup);
        }
        reply
    })
    .await
    .unwrap_or_else(|_| failed(&fallback, "IPC_CONTROL_PLANE_UNAVAILABLE")))
}

fn failed(request: &Value, code: &str) -> Value {
    json!({"requestId":request.get("requestId").and_then(Value::as_str).unwrap_or("invalid-request"), "schemaVersion":1, "ok":false, "error":TradeXError::new(code)})
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let default = app.path().home_dir()?.join(".tradex/workspaces/default");
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            let engine = Arc::new(Mutex::new(ControlPlane::new(default)));
            let gateway = Arc::new(Mutex::new(GatewayHost::new(app_data.join("models"))));
            let exiting = Arc::new(AtomicBool::new(false));
            app.manage(Service(
                engine.clone(),
                gateway.clone(),
                RuntimeSupervisor::new(),
                exiting.clone(),
            ));
            std::thread::spawn(move || {
                while !exiting.load(Ordering::Acquire) {
                    if let Ok(mut host) = gateway.try_lock() {
                        let job = engine
                            .lock()
                            .ok()
                            .and_then(|engine| engine.gateway_monitor_job());
                        if let Some(job) = job {
                            if host.needs_deepseek_key_reload() {
                                if let Ok(key) = NativeModelVault.get_deepseek(job.workspace_id()) {
                                    host.set_deepseek_key(job.workspace_id(), Some(key.as_str()));
                                } else {
                                    host.set_deepseek_key(job.workspace_id(), None);
                                }
                            }
                            let outcome = host.monitor(&job, || {
                                engine
                                    .lock()
                                    .is_ok_and(|engine| engine.gateway_job_current(&job))
                            });
                            if let Some(state) = outcome {
                                let committed = engine.lock().is_ok_and(|mut engine| {
                                    engine.complete_gateway(&job, state)["ok"] == true
                                });
                                if !committed {
                                    host.stop();
                                }
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![control])
        .build(tauri::generate_context!())
        .expect("TradeX could not start its desktop shell")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                app.state::<Service>().2.stop_all();
                app.state::<Service>().3.store(true, Ordering::Release);
                if let Ok(mut gateway) = app.state::<Service>().1.lock() {
                    gateway.stop();
                }
            }
        });
}
