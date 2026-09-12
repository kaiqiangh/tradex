use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tradex::{ControlPlane, protocol::EventSink};

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
    let mut control = ControlPlane::new(PathBuf::from(path));
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
            #[cfg(feature = "integration-test")]
            let result = match control.prepare_provider(&request) {
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
            };
            #[cfg(not(feature = "integration-test"))]
            let result = control.dispatch_with_events(request.clone(), "stdio", Some(sink.clone()));
            #[cfg(all(feature = "integration-test", target_os = "macos"))]
            let result = match request.get("command").and_then(Value::as_str) {
                Some("model.gateway") => match control.prepare_gateway(&request) {
                    Ok(Some(job)) => {
                        let outcome = gateway.run(&job, || control.gateway_job_current(&job));
                        control.complete_gateway(&job, outcome)
                    }
                    Ok(None) => result,
                    Err(error) => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":error})
                    }
                },
                Some("model.configure_deepseek") | Some("model.verify_route") => {
                    match control.prepare_model(&request) {
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
