#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
use block2::RcBlock;
#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSApplicationDidBecomeActiveNotification, NSApplicationDidResignActiveNotification,
    NSWorkspace, NSWorkspaceDidWakeNotification, NSWorkspaceSessionDidResignActiveNotification,
    NSWorkspaceWillSleepNotification,
};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSNotification, NSNotificationCenter, NSNotificationName};
use serde_json::{Value, json};
#[cfg(target_os = "macos")]
use std::ptr::NonNull;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tauri::{Manager, ipc::Channel};
#[cfg(target_os = "macos")]
use tradex::alpaca_stream::AlpacaPrivateStreamSupervisor;
#[cfg(target_os = "macos")]
use tradex::binance_stream::{BinancePrivateStreamSupervisor, mark_connected_accounts_degraded};
#[cfg(unix)]
use tradex::order_gateway::OrderGatewayHost;
use tradex::{
    BacktestSupervisor, ControlPlane, RuntimeSupervisor, StrategySupervisor, data_sources,
    gateway_process::GatewayHost,
    model,
    model_credentials::{ModelVault, NativeModelVault},
    native_credentials,
    protocol::{DomainEvent, TradeXError},
    provider_io::{BrokerHttp, NativeVault, ProviderJob},
};

struct Service(
    Arc<Mutex<ControlPlane>>,
    Arc<Mutex<GatewayHost>>,
    RuntimeSupervisor,
    StrategySupervisor,
    BacktestSupervisor,
    Arc<AtomicBool>,
    #[cfg(target_os = "macos")] AlpacaPrivateStreamSupervisor,
    #[cfg(target_os = "macos")] BinancePrivateStreamSupervisor,
    #[cfg(unix)] Arc<Mutex<OrderGatewayHost>>,
);

fn read_startup_provider_job(
    engine: &Arc<Mutex<ControlPlane>>,
    job: &ProviderJob,
) -> tradex::provider_io::ProviderOutcome {
    job.run(
        &NativeVault,
        |_| Err(TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED")),
        &BrokerHttp::default(),
        || {
            engine
                .lock()
                .is_ok_and(|control| control.provider_job_current(job))
        },
    )
}

fn start_startup_live_recovery(engine: Arc<Mutex<ControlPlane>>, workspace_id: String) {
    #[cfg(feature = "integration-test")]
    if std::env::var_os("TRADEX_INTEGRATION_STARTUP_RECOVERY").is_none() {
        return;
    }

    std::thread::spawn(move || {
        let plan = match engine.lock() {
            Ok(mut control) => match control.prepare_startup_live_recovery_plan() {
                Ok(Some(plan)) if plan.workspace_id == workspace_id => plan,
                Ok(_) => return,
                Err(error) => {
                    eprintln!(
                        "TradeX startup recovery could not be prepared: {}",
                        error.code
                    );
                    return;
                }
            },
            Err(_) => return,
        };

        let mut order_jobs = Vec::new();
        for input in &plan.unresolved_attempts {
            let job = match engine.lock() {
                Ok(mut control) => match control.prepare_startup_live_order_reconciliation(input) {
                    Ok(Some(job)) => job,
                    Ok(None) => continue,
                    Err(error) => {
                        eprintln!(
                            "TradeX startup order reconciliation skipped: {}",
                            error.code
                        );
                        continue;
                    }
                },
                Err(_) => return,
            };
            order_jobs.push(job);
        }

        let mut account_jobs = Vec::new();
        for connection_id in &plan.account_ids {
            let job = match engine.lock() {
                Ok(mut control) => {
                    match control
                        .prepare_startup_live_account_refresh(&plan.workspace_id, connection_id)
                    {
                        Ok(Some(job)) => job,
                        Ok(None) => continue,
                        Err(error) => {
                            eprintln!("TradeX startup account refresh skipped: {}", error.code);
                            continue;
                        }
                    }
                }
                Err(_) => return,
            };
            account_jobs.push((connection_id.clone(), job));
        }

        // Dispatch exact-attempt P0 evidence reads before P1 account refreshes.
        // Both read sets overlap, but P0 results are committed before P1 results.
        let account_outcomes = std::thread::scope(|scope| {
            let order_reads = order_jobs
                .into_iter()
                .map(|job| {
                    let worker_engine = Arc::clone(&engine);
                    scope.spawn(move || {
                        let outcome = read_startup_provider_job(&worker_engine, &job);
                        (job, outcome)
                    })
                })
                .collect::<Vec<_>>();

            let account_reads = account_jobs
                .into_iter()
                .map(|(connection_id, job)| {
                    let worker_engine = Arc::clone(&engine);
                    scope.spawn(move || {
                        let outcome = read_startup_provider_job(&worker_engine, &job);
                        (connection_id, job, outcome)
                    })
                })
                .collect::<Vec<_>>();

            for read in order_reads {
                match read.join() {
                    Ok((job, outcome)) => {
                        let completed = engine.lock().is_ok_and(|mut control| {
                            control.complete_provider(&job, outcome)["ok"] == true
                        });
                        if !completed {
                            eprintln!("TradeX startup order reconciliation remains pending.");
                        }
                    }
                    Err(_) => {
                        eprintln!("TradeX startup order reconciliation worker failed.");
                    }
                }
            }

            account_reads
                .into_iter()
                .filter_map(|read| match read.join() {
                    Ok(result) => Some(result),
                    Err(_) => {
                        eprintln!("TradeX startup account refresh worker failed.");
                        None
                    }
                })
                .collect::<Vec<_>>()
        });

        for (connection_id, job, outcome) in account_outcomes {
            let refreshed = engine
                .lock()
                .is_ok_and(|mut control| control.complete_provider(&job, outcome)["ok"] == true);
            if !refreshed {
                continue;
            }
            if let Ok(mut control) = engine.lock()
                && let Err(error) =
                    control.finish_startup_live_reconciliation(&plan.workspace_id, &connection_id)
            {
                eprintln!("TradeX startup recovery remains blocked: {}", error.code);
            }
        }
    });
}

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
    let strategy_supervisor = service.3.clone();
    let backtest_supervisor = service.4.clone();
    #[cfg(target_os = "macos")]
    let private_stream_supervisor = service.6.clone();
    #[cfg(target_os = "macos")]
    let binance_stream_supervisor = service.7.clone();
    #[cfg(target_os = "macos")]
    let order_gateway_host = service.8.clone();
    #[cfg(all(unix, not(target_os = "macos")))]
    let order_gateway_host = service.6.clone();
    let fallback = request.clone();
    Ok(tauri::async_runtime::spawn_blocking(move || {
        #[cfg(unix)]
        if request.get("command").and_then(Value::as_str) == Some("trade.execution.prepare")
            && !order_gateway_host
                .lock()
                .is_ok_and(|mut host| host.is_running())
        {
            return failed(&request, "GATEWAY_UNAVAILABLE");
        }
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
        let opening_workspace =
            request.get("command").and_then(Value::as_str) == Some("workspace.open");
        if opening_workspace {
            supervisor.stop_all();
            strategy_supervisor.stop_all();
            backtest_supervisor.stop_all();
            #[cfg(target_os = "macos")]
            private_stream_supervisor.pause();
            #[cfg(target_os = "macos")]
            binance_stream_supervisor.pause();
            #[cfg(target_os = "macos")]
            mark_connected_accounts_degraded(&engine);
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
        if command == Some("strategy.run") {
            return strategy_supervisor.start(engine, request);
        }
        if command == Some("strategy.cancel") {
            return strategy_supervisor.cancel(engine, request);
        }
        if command == Some("backtest.run") {
            return backtest_supervisor.start(engine, request);
        }
        if command == Some("backtest.cancel") {
            return backtest_supervisor.cancel(engine, request);
        }
        let execution_engine = engine.clone();
        let prepared = match engine.lock() {
            Ok(mut engine) => match engine.prepare_provider_for(&request, &consumer) {
                Ok(Some(job)) => job,
                Ok(None) => {
                    #[cfg(feature = "integration-test")]
                    let request = if opening_workspace
                        && std::env::var_os("TRADEX_INTEGRATION_ARMING_FIXTURE").is_some()
                    {
                        let Some(path) = std::env::var_os("TRADEX_INTEGRATION_WORKSPACE") else {
                            return failed(&request, "IPC_PAYLOAD_INVALID");
                        };
                        let mut request = request;
                        request["payload"]["path"] = json!(path.to_string_lossy());
                        request
                    } else {
                        request
                    };
                    #[cfg(feature = "integration-test")]
                    let fixture_request = request.clone();
                    let is_live_execution_prepare = request.get("command").and_then(Value::as_str)
                        == Some("trade.execution.prepare");
                    let reply = engine.dispatch_with_runtime(
                        request,
                        &consumer,
                        Some(Arc::new(move |event| events.send(event).is_ok())),
                        None,
                    );
                    #[cfg(feature = "integration-test")]
                    let reply = if opening_workspace
                        && std::env::var_os("TRADEX_INTEGRATION_ARMING_FIXTURE").is_some()
                        && reply["ok"] == true
                    {
                        match reply["data"]["workspaceId"].as_str() {
                            Some(workspace_id) => {
                                if let Err(error) =
                                    engine.seed_browser_workspace_ready(workspace_id)
                                {
                                    return failed(&fixture_request, &error.code);
                                }
                                match engine.seed_live_arming_fixture(
                                    workspace_id,
                                    "trading212",
                                    "Isolated macOS Lock QA",
                                ) {
                                    Ok(_) => reply,
                                    Err(error) => failed(&fixture_request, &error.code),
                                }
                            }
                            None => failed(&fixture_request, "IPC_PAYLOAD_INVALID"),
                        }
                    } else {
                        reply
                    };
                    drop(engine);
                    if opening_workspace
                        && reply["ok"] == true
                        && let Some(workspace_id) = reply["data"]["workspaceId"].as_str()
                    {
                        start_startup_live_recovery(
                            execution_engine.clone(),
                            workspace_id.to_owned(),
                        );
                    }
                    #[cfg(unix)]
                    if is_live_execution_prepare
                        && reply["ok"] == true
                        && reply["data"]["attempt"]["state"] == "RESERVED"
                    {
                        let attempt_id = reply["data"]["attempt"]["attemptId"]
                            .as_str()
                            .unwrap_or_default()
                            .to_owned();
                        let dispatch_result = order_gateway_host.lock().map(|mut host| {
                            host.dispatch_attempt(
                                &attempt_id,
                                {
                                    let engine = execution_engine.clone();
                                    move |attempt_id, session_id| {
                                        engine
                                            .lock()
                                            .map_err(|_| {
                                                "IPC_CONTROL_PLANE_UNAVAILABLE".to_owned()
                                            })?
                                            .live_dispatch_package(attempt_id, session_id)
                                            .map_err(|error| error.code)
                                    }
                                },
                                {
                                    let engine = execution_engine.clone();
                                    move |grant_id, session_id| {
                                        engine
                                            .lock()
                                            .map_err(|_| {
                                                "IPC_CONTROL_PLANE_UNAVAILABLE".to_owned()
                                            })?
                                            .begin_live_execution_submission(grant_id, session_id)
                                            .map(|_| ())
                                            .map_err(|error| error.code)
                                    }
                                },
                                {
                                    let engine = execution_engine.clone();
                                    move |attempt_id, code| {
                                        if let Ok(mut engine) = engine.lock() {
                                            let _ = engine.stop_live_dispatch_before_submission(
                                                attempt_id, code,
                                            );
                                        }
                                    }
                                },
                                {
                                    let engine = execution_engine.clone();
                                    move |attempt_id, outcome| {
                                        engine
                                            .lock()
                                            .map_err(|_| {
                                                "IPC_CONTROL_PLANE_UNAVAILABLE".to_owned()
                                            })?
                                            .complete_live_execution_submission(attempt_id, outcome)
                                            .map(|_| ())
                                            .map_err(|error| error.code)
                                    }
                                },
                            )
                        });
                        let failed = !matches!(dispatch_result, Ok(Ok(())));
                        if failed {
                            if let Ok(mut host) = order_gateway_host.lock() {
                                host.stop();
                            }
                            if let Ok(mut engine) = execution_engine.lock() {
                                let _ = engine.stop_live_dispatch_before_submission(
                                    &attempt_id,
                                    "GATEWAY_PROCESS_FAILED",
                                );
                            }
                        }
                    }
                    #[cfg(target_os = "macos")]
                    if opening_workspace {
                        private_stream_supervisor.resume();
                    }
                    #[cfg(target_os = "macos")]
                    if opening_workspace {
                        binance_stream_supervisor.resume();
                    }
                    return reply;
                }
                Err(error) => {
                    #[cfg(target_os = "macos")]
                    if opening_workspace {
                        private_stream_supervisor.resume();
                    }
                    #[cfg(target_os = "macos")]
                    if opening_workspace {
                        binance_stream_supervisor.resume();
                    }
                    return failed(&request, &error.code);
                }
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

#[cfg(target_os = "macos")]
fn disarm_live_for_safety_trigger(engine: &Arc<Mutex<ControlPlane>>, reason: &'static str) {
    if let Ok(mut engine) = engine.lock()
        && let Err(error) = engine.disarm_live_for_safety(reason)
    {
        eprintln!(
            "TradeX could not disarm Live accounts after {reason}: {}",
            error.code
        );
    }
}

#[cfg(target_os = "macos")]
fn register_live_safety_observer(
    center: &NSNotificationCenter,
    name: &NSNotificationName,
    reason: &'static str,
    engine: Arc<Mutex<ControlPlane>>,
) {
    let observer = RcBlock::new(move |_notification: NonNull<NSNotification>| {
        disarm_live_for_safety_trigger(&engine, reason);
    });
    // These observers last for the app process lifetime; the center retains the blocks.
    let token = unsafe {
        center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &observer)
    };
    std::mem::forget(token);
}

#[cfg(target_os = "macos")]
fn register_live_safety_observers(engine: Arc<Mutex<ControlPlane>>) {
    let workspace_center = NSWorkspace::sharedWorkspace().notificationCenter();
    // SAFETY: These extern constants are AppKit's documented notification names.
    let notifications = unsafe {
        [
            (NSWorkspaceWillSleepNotification, "OS_SLEEP"),
            (
                NSWorkspaceSessionDidResignActiveNotification,
                "SESSION_INACTIVE",
            ),
        ]
    };
    for (name, reason) in notifications {
        register_live_safety_observer(&workspace_center, name, reason, engine.clone());
    }

    // AppKit posts this public process-local event when another app takes active status.
    // The lock screen activates loginwindow, so this also fails closed on screen lock.
    let application_center = NSNotificationCenter::defaultCenter();
    register_live_safety_observer(
        &application_center,
        unsafe { NSApplicationDidResignActiveNotification },
        "SESSION_INACTIVE",
        engine,
    );
}

#[cfg(target_os = "macos")]
fn recover_live_after_resume(
    engine: Arc<Mutex<ControlPlane>>,
    private_stream_supervisor: AlpacaPrivateStreamSupervisor,
    binance_stream_supervisor: BinancePrivateStreamSupervisor,
    last_trigger: Arc<Mutex<Option<std::time::Instant>>>,
) {
    if let Ok(mut previous) = last_trigger.lock() {
        let now = std::time::Instant::now();
        if previous
            .as_ref()
            .is_some_and(|last| now.duration_since(*last) < std::time::Duration::from_millis(500))
        {
            return;
        }
        *previous = Some(now);
    } else {
        return;
    }

    let workspace_id = match engine.lock() {
        Ok(mut control) => match control.resume() {
            Ok(workspace_id) => workspace_id,
            Err(error) => {
                eprintln!(
                    "TradeX could not durably invalidate Live state after resume: {}",
                    error.code
                );
                return;
            }
        },
        Err(_) => return,
    };
    private_stream_supervisor.restart_all();
    binance_stream_supervisor.restart_all();
    mark_connected_accounts_degraded(&engine);
    if let Some(workspace_id) = workspace_id {
        start_startup_live_recovery(engine, workspace_id);
    }
}

#[cfg(target_os = "macos")]
fn register_live_recovery_observer(
    center: &NSNotificationCenter,
    name: &NSNotificationName,
    engine: Arc<Mutex<ControlPlane>>,
    private_stream_supervisor: AlpacaPrivateStreamSupervisor,
    binance_stream_supervisor: BinancePrivateStreamSupervisor,
    last_trigger: Arc<Mutex<Option<std::time::Instant>>>,
) {
    let observer = RcBlock::new(move |_notification: NonNull<NSNotification>| {
        recover_live_after_resume(
            engine.clone(),
            private_stream_supervisor.clone(),
            binance_stream_supervisor.clone(),
            last_trigger.clone(),
        );
    });
    let token = unsafe {
        center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &observer)
    };
    std::mem::forget(token);
}

#[cfg(target_os = "macos")]
fn register_live_recovery_observers(
    engine: Arc<Mutex<ControlPlane>>,
    private_stream_supervisor: AlpacaPrivateStreamSupervisor,
    binance_stream_supervisor: BinancePrivateStreamSupervisor,
) {
    let last_trigger = Arc::new(Mutex::new(None));
    let workspace_center = NSWorkspace::sharedWorkspace().notificationCenter();
    register_live_recovery_observer(
        &workspace_center,
        unsafe { NSWorkspaceDidWakeNotification },
        engine.clone(),
        private_stream_supervisor.clone(),
        binance_stream_supervisor.clone(),
        last_trigger.clone(),
    );

    let application_center = NSNotificationCenter::defaultCenter();
    register_live_recovery_observer(
        &application_center,
        unsafe { NSApplicationDidBecomeActiveNotification },
        engine,
        private_stream_supervisor,
        binance_stream_supervisor,
        last_trigger,
    );
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(feature = "integration-test")]
            let default = std::env::var_os("TRADEX_INTEGRATION_WORKSPACE")
                .map(std::path::PathBuf::from)
                .unwrap_or(app.path().home_dir()?.join(".tradex/workspaces/default"));
            #[cfg(not(feature = "integration-test"))]
            let default = app.path().home_dir()?.join(".tradex/workspaces/default");
            #[cfg(feature = "integration-test")]
            let app_data = std::env::var_os("TRADEX_INTEGRATION_APP_DATA")
                .map(std::path::PathBuf::from)
                .unwrap_or(app.path().app_data_dir()?);
            #[cfg(not(feature = "integration-test"))]
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            let engine = Arc::new(Mutex::new(ControlPlane::new(default)));
            let gateway = Arc::new(Mutex::new(GatewayHost::new(app_data.join("models"))));
            #[cfg(unix)]
            let order_gateway_host = {
                let pinned = option_env!("TRADEX_ORDER_GATEWAY_SHA256").unwrap_or_default();
                let mut host = OrderGatewayHost::from_current_executable(pinned)
                    .map_err(std::io::Error::other)?;
                if !pinned.is_empty()
                    && let Err(code) = host.start()
                {
                    eprintln!("TradeX Order Gateway startup failed: {code}");
                }
                Arc::new(Mutex::new(host))
            };
            let exiting = Arc::new(AtomicBool::new(false));
            #[cfg(target_os = "macos")]
            register_live_safety_observers(engine.clone());
            #[cfg(target_os = "macos")]
            let private_stream_supervisor = AlpacaPrivateStreamSupervisor::new();
            #[cfg(target_os = "macos")]
            let binance_stream_supervisor = BinancePrivateStreamSupervisor::new();
            #[cfg(target_os = "macos")]
            register_live_recovery_observers(
                engine.clone(),
                private_stream_supervisor.clone(),
                binance_stream_supervisor.clone(),
            );
            #[cfg(target_os = "macos")]
            app.manage(Service(
                engine.clone(),
                gateway.clone(),
                RuntimeSupervisor::new(),
                StrategySupervisor::new(),
                BacktestSupervisor::new(),
                exiting.clone(),
                private_stream_supervisor.clone(),
                binance_stream_supervisor.clone(),
                order_gateway_host.clone(),
            ));
            #[cfg(not(target_os = "macos"))]
            app.manage(Service(
                engine.clone(),
                gateway.clone(),
                RuntimeSupervisor::new(),
                StrategySupervisor::new(),
                BacktestSupervisor::new(),
                exiting.clone(),
                #[cfg(unix)]
                order_gateway_host.clone(),
            ));
            #[cfg(target_os = "macos")]
            let stream_engine = engine.clone();
            #[cfg(target_os = "macos")]
            let stream_supervisor = private_stream_supervisor.clone();
            #[cfg(target_os = "macos")]
            let binance_stream_engine = engine.clone();
            #[cfg(target_os = "macos")]
            let binance_supervisor = binance_stream_supervisor.clone();
            #[cfg(target_os = "macos")]
            let stream_exiting = exiting.clone();
            #[cfg(target_os = "macos")]
            std::thread::spawn(move || {
                while !stream_exiting.load(Ordering::Acquire) {
                    stream_supervisor.sync(stream_engine.clone());
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                stream_supervisor.stop_all();
            });
            #[cfg(target_os = "macos")]
            let binance_stream_exiting = exiting.clone();
            #[cfg(target_os = "macos")]
            std::thread::spawn(move || {
                while !binance_stream_exiting.load(Ordering::Acquire) {
                    binance_supervisor.sync(binance_stream_engine.clone());
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                binance_supervisor.stop_all();
            });
            let arming_engine = engine.clone();
            let arming_exiting = exiting.clone();
            std::thread::spawn(move || {
                while !arming_exiting.load(Ordering::Acquire) {
                    if let Ok(mut engine) = arming_engine.try_lock()
                        && let Err(error) = engine.expire_live_arming()
                    {
                        eprintln!(
                            "TradeX could not enforce Live safety deadlines: {}",
                            error.code
                        );
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });
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
            #[cfg(not(target_os = "macos"))]
            if matches!(event, tauri::RunEvent::Resumed)
                && let Ok(mut engine) = app.state::<Service>().0.lock()
            {
                match engine.resume() {
                    Ok(Some(workspace_id)) => {
                        start_startup_live_recovery(app.state::<Service>().0.clone(), workspace_id);
                    }
                    Ok(None) => (),
                    Err(error) => {
                        eprintln!(
                            "TradeX could not durably invalidate Live state after resume: {}",
                            error.code
                        );
                    }
                }
            }
            if matches!(event, tauri::RunEvent::Exit) {
                app.state::<Service>().2.stop_all();
                app.state::<Service>().3.stop_all();
                app.state::<Service>().4.stop_all();
                app.state::<Service>().5.store(true, Ordering::Release);
                #[cfg(target_os = "macos")]
                app.state::<Service>().6.stop_all();
                #[cfg(target_os = "macos")]
                app.state::<Service>().7.stop_all();
                #[cfg(target_os = "macos")]
                {
                    let service = app.state::<Service>();
                    mark_connected_accounts_degraded(&service.0);
                }
                if let Ok(mut gateway) = app.state::<Service>().1.lock() {
                    gateway.stop();
                }
                #[cfg(target_os = "macos")]
                if let Ok(mut gateway) = app.state::<Service>().8.lock() {
                    gateway.stop();
                }
                #[cfg(all(unix, not(target_os = "macos")))]
                if let Ok(mut gateway) = app.state::<Service>().6.lock() {
                    gateway.stop();
                }
            }
        });
}
