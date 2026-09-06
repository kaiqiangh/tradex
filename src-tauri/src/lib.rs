pub mod protocol;
mod storage;

use protocol::{
    Aggregate, CommandEnvelope, EmptyPayload, EventSink, MAX_SEQUENCE, OpenWorkspace, Result,
    RuntimeComponent, RuntimeStatus, Subscribe, TradeXError,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::{collections::HashMap, path::PathBuf};
use storage::Store;

pub struct ControlPlane {
    default_workspace: PathBuf,
    store: Option<Store>,
    subscribers: HashMap<String, EventSink>,
}

impl ControlPlane {
    pub fn new(default_workspace: PathBuf) -> Self {
        Self {
            default_workspace,
            store: None,
            subscribers: HashMap::new(),
        }
    }

    pub fn dispatch(&mut self, request: Value) -> Value {
        self.dispatch_with_events(request, "headless", None)
    }

    pub fn dispatch_with_events(
        &mut self,
        request: Value,
        consumer: &str,
        sink: Option<EventSink>,
    ) -> Value {
        let id = request
            .get("requestId")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty() && s.len() <= 128)
            .unwrap_or("invalid-request")
            .to_owned();
        match self.execute(request, consumer, sink) {
            Ok((data, version)) => {
                let mut reply = json!({"requestId":id,"schemaVersion":1,"ok":true,"data":data});
                if let Some(version) = version {
                    reply["stateVersion"] = version.into();
                }
                reply
            }
            Err(error) => json!({"requestId":id,"schemaVersion":1,"ok":false,"error":error}),
        }
    }

    fn execute(
        &mut self,
        value: Value,
        consumer: &str,
        sink: Option<EventSink>,
    ) -> Result<(Value, Option<String>)> {
        match value.get("schemaVersion").and_then(Value::as_u64) {
            Some(1) => (),
            Some(_) => return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED")),
            None => return Err(TradeXError::new("IPC_PAYLOAD_INVALID")),
        }
        let request: CommandEnvelope = payload(value)?;
        if request.request_id.is_empty() || request.request_id.len() > 128 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        match request.command.as_str() {
            "workspace.open" => {
                let input: OpenWorkspace = payload(request.payload)?;
                if input.name.as_ref().is_some_and(|n| {
                    n.trim().is_empty()
                        || n.chars().count() > 120
                        || n.chars().any(char::is_control)
                }) || input
                    .base_currency
                    .as_ref()
                    .is_some_and(|c| c.len() != 3 || !c.bytes().all(|b| b.is_ascii_uppercase()))
                {
                    return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
                }
                let path = storage::directory(
                    &input
                        .path
                        .as_ref()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| self.default_workspace.clone()),
                )?;
                let same = self.store.as_ref().is_some_and(|store| store.path == path);
                if same {
                    let event = self.store.as_mut().unwrap().record_open()?;
                    self.subscribers.retain(|_, sink| sink(event.clone()));
                    let version = format!("{}:{}", event.aggregate_id, event.sequence);
                    Ok((json!(event.payload), Some(version)))
                } else {
                    let mut store = Store::open(path, &input)?;
                    let event = store.record_open()?;
                    let version = format!("{}:{}", event.aggregate_id, event.sequence);
                    self.store = Some(store);
                    self.subscribers.clear();
                    Ok((json!(event.payload), Some(version)))
                }
            }
            "runtime.status" => {
                let _: EmptyPayload = payload(request.payload)?;
                let runtime = RuntimeStatus {
                    components: vec![
                        RuntimeComponent {
                            id: "control-plane".into(),
                            status: "RUNNING".into(),
                            message: "Local workspace control is available.".into(),
                        },
                        RuntimeComponent {
                            id: "codex".into(),
                            status: "NOT_CONFIGURED".into(),
                            message: "Codex App Server is not configured.".into(),
                        },
                        RuntimeComponent {
                            id: "cliproxyapi".into(),
                            status: "NOT_CONFIGURED".into(),
                            message: "No model route is configured.".into(),
                        },
                        RuntimeComponent {
                            id: "order-gateway".into(),
                            status: "NOT_CONFIGURED".into(),
                            message: "Live execution is unavailable.".into(),
                        },
                    ],
                    model_available: false,
                    live_execution_available: false,
                };
                Ok((json!(runtime), None))
            }
            "domain.snapshot" => {
                let input: Aggregate = payload(request.payload)?;
                validate_aggregate(&input.aggregate_type, &input.aggregate_id)?;
                let store = self
                    .store
                    .as_mut()
                    .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
                let snapshot = store.snapshot()?;
                if input.aggregate_type != snapshot.aggregate_type
                    || input.aggregate_id != snapshot.aggregate_id
                {
                    return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
                }
                let version = format!("{}:{}", snapshot.aggregate_id, snapshot.last_sequence);
                Ok((json!(snapshot), Some(version)))
            }
            "domain.subscribe" => {
                let input: Subscribe = payload(request.payload)?;
                validate_aggregate(&input.aggregate_type, &input.aggregate_id)?;
                if input.after_sequence > MAX_SEQUENCE {
                    return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
                }
                let store = self
                    .store
                    .as_mut()
                    .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
                let snapshot = store.snapshot()?;
                if input.aggregate_type != snapshot.aggregate_type
                    || input.aggregate_id != snapshot.aggregate_id
                {
                    return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
                }
                let sink =
                    sink.ok_or_else(|| TradeXError::new("IPC_SUBSCRIPTION_CHANNEL_REQUIRED"))?;
                match store.replay(input.after_sequence, &sink) {
                    Ok(ack) => {
                        self.subscribers.insert(consumer.to_owned(), sink);
                        Ok((json!(ack), None))
                    }
                    Err(error) => {
                        if error.code == "IPC_SUBSCRIPTION_DELIVERY_FAILED" {
                            self.subscribers.remove(consumer);
                        }
                        Err(error)
                    }
                }
            }
            _ => Err(TradeXError::new("IPC_COMMAND_UNKNOWN")),
        }
    }
}

fn payload<T: DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|_| TradeXError::new("IPC_PAYLOAD_INVALID"))
}

fn validate_aggregate(kind: &str, id: &str) -> Result<()> {
    if kind.is_empty() || kind.len() > 64 || id.is_empty() || id.len() > 128 {
        Err(TradeXError::new("IPC_PAYLOAD_INVALID"))
    } else {
        Ok(())
    }
}
