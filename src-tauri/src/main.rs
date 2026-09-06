#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tauri::{Manager, ipc::Channel};
use tradex::{
    ControlPlane,
    protocol::{DomainEvent, TradeXError},
};

struct Service(Arc<Mutex<ControlPlane>>);

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
    let fallback = request.clone();
    Ok(
        tauri::async_runtime::spawn_blocking(move || match engine.lock() {
            Ok(mut engine) => engine.dispatch_with_events(
                request,
                &consumer,
                Some(Arc::new(move |event| events.send(event).is_ok())),
            ),
            Err(_) => failed(&request, "IPC_CONTROL_PLANE_UNAVAILABLE"),
        })
        .await
        .unwrap_or_else(|_| failed(&fallback, "IPC_CONTROL_PLANE_UNAVAILABLE")),
    )
}

fn failed(request: &Value, code: &str) -> Value {
    json!({"requestId":request.get("requestId").and_then(Value::as_str).unwrap_or("invalid-request"), "schemaVersion":1, "ok":false, "error":TradeXError::new(code)})
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let default = app.path().home_dir()?.join(".tradex/workspaces/default");
            app.manage(Service(Arc::new(Mutex::new(ControlPlane::new(default)))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![control])
        .run(tauri::generate_context!())
        .expect("TradeX could not start its desktop shell");
}
