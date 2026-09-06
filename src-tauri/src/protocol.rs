use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_SEQUENCE: u64 = 9_007_199_254_740_991;

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandEnvelope {
    #[schemars(length(min = 1, max = 128))]
    pub request_id: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    pub command: String,
    pub payload: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenWorkspace {
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String")]
    pub path: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", length(min = 1, max = 120))]
    pub name: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", regex(pattern = "^[A-Z]{3}$"))]
    pub base_currency: Option<String>,
}

fn present<'de, D: serde::Deserializer<'de>>(
    value: D,
) -> std::result::Result<Option<String>, D::Error> {
    String::deserialize(value).map(Some)
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EmptyPayload {}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeComponent {
    pub id: String,
    pub status: String,
    pub message: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeStatus {
    pub components: Vec<RuntimeComponent>,
    pub model_available: bool,
    pub live_execution_available: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Aggregate {
    pub aggregate_type: String,
    pub aggregate_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Subscribe {
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub after_sequence: u64,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubscriptionAck {
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub after_sequence: u64,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub last_sequence: u64,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub replayed_count: u64,
}

pub type EventSink = std::sync::Arc<dyn Fn(DomainEvent) -> bool + Send + Sync>;

#[derive(JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SuccessEnvelope {
    pub request_id: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    #[schemars(extend("const" = true))]
    pub ok: bool,
    pub data: ReplyData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_version: Option<String>,
}

#[derive(JsonSchema)]
#[serde(untagged)]
pub enum ReplyData {
    Workspace(Workspace),
    Snapshot(Snapshot),
    Runtime(RuntimeStatus),
    Subscription(SubscriptionAck),
}

#[derive(JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FailureEnvelope {
    pub request_id: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    #[schemars(extend("const" = false))]
    pub ok: bool,
    pub error: TradeXError,
}

#[derive(JsonSchema)]
#[serde(untagged)]
pub enum ResultEnvelope {
    Success(SuccessEnvelope),
    Failure(FailureEnvelope),
}

/// Exported to JSON Schema and TypeScript, and used for renderer runtime validation.
#[derive(JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IpcSchema {
    pub command: CommandEnvelope,
    pub result: ResultEnvelope,
    pub event: DomainEvent,
    pub workspace_open: OpenWorkspace,
    pub aggregate: Aggregate,
    pub subscribe: Subscribe,
    pub empty: EmptyPayload,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Workspace {
    pub workspace_id: String,
    pub name: String,
    pub base_currency: String,
    pub path: String,
    pub created_at: String,
    pub last_opened_at: String,
    #[schemars(extend("const" = 1))]
    pub storage_schema_version: u32,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DomainEvent {
    pub event_id: String,
    #[schemars(extend("const" = "workspace.opened"))]
    pub event_type: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    pub occurred_at: String,
    #[schemars(extend("const" = "workspace"))]
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub sequence: u64,
    pub payload: Workspace,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snapshot {
    #[schemars(extend("const" = "workspace"))]
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub projection: Workspace,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub last_sequence: u64,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Remediation {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TradeXError {
    pub category: String,
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub blocking: bool,
    pub remediation_actions: Vec<Remediation>,
}

impl TradeXError {
    pub fn new(code: &str) -> Self {
        let (message, action, label) = match code {
            "WORKSPACE_PATH_INVALID" => (
                "Choose an absolute local folder path.",
                "choose_workspace",
                "Choose a folder",
            ),
            "WORKSPACE_BUSY" => (
                "This workspace is open in another process.",
                "retry_workspace",
                "Retry opening",
            ),
            "WORKSPACE_SCHEMA_UNSUPPORTED" => (
                "This workspace needs a compatible TradeX version.",
                "check_version",
                "Check application version",
            ),
            "WORKSPACE_INTEGRITY_FAILED" => (
                "Workspace integrity could not be verified. Existing data was preserved.",
                "choose_workspace",
                "Choose another workspace",
            ),
            "WORKSPACE_OPEN_FAILED" => (
                "The workspace could not be opened. Check folder access and available storage.",
                "retry_workspace",
                "Retry opening",
            ),
            "IPC_SCHEMA_UNSUPPORTED" => (
                "The application and control plane use incompatible schemas.",
                "check_version",
                "Check application version",
            ),
            "IPC_COMMAND_UNKNOWN" => (
                "This command is not supported by the current control plane.",
                "check_version",
                "Check application version",
            ),
            "IPC_AGGREGATE_NOT_FOUND" => (
                "The requested state is not available in the active workspace.",
                "reload_snapshot",
                "Reload workspace",
            ),
            "STATE_VERSION_CONFLICT" | "IPC_REPLAY_UNAVAILABLE" => (
                "The state cursor cannot be resumed. Reload authoritative state.",
                "reload_snapshot",
                "Reload state",
            ),
            "IPC_PAYLOAD_INVALID" => (
                "The request does not match the supported command schema.",
                "retry_request",
                "Review and retry",
            ),
            _ => (
                "The control plane could not complete this operation.",
                "reload_snapshot",
                "Reload state",
            ),
        };
        Self {
            category: if matches!(
                code,
                "IPC_AGGREGATE_NOT_FOUND" | "STATE_VERSION_CONFLICT" | "IPC_REPLAY_UNAVAILABLE"
            ) {
                "STATE_STALE"
            } else {
                "INTERNAL_ERROR"
            }
            .into(),
            code: code.into(),
            message: message.into(),
            retryable: matches!(code, "WORKSPACE_BUSY" | "WORKSPACE_OPEN_FAILED"),
            blocking: true,
            remediation_actions: vec![Remediation {
                id: action.into(),
                label: label.into(),
            }],
        }
    }
}

pub type Result<T> = std::result::Result<T, TradeXError>;
