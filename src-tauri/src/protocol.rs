use crate::providers::*;
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
    #[schemars(length(min = 1))]
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
    #[schemars(length(min = 1, max = 64))]
    pub aggregate_type: String,
    #[schemars(length(min = 1, max = 128))]
    pub aggregate_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Subscribe {
    #[schemars(length(min = 1, max = 64))]
    pub aggregate_type: String,
    #[schemars(length(min = 1, max = 128))]
    pub aggregate_id: String,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub after_sequence: u64,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubscriptionAck {
    #[schemars(extend("enum" = ["workspace", "account"]))]
    pub aggregate_type: String,
    #[schemars(length(min = 1))]
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
    #[schemars(length(min = 1, max = 128))]
    pub request_id: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    #[schemars(extend("const" = true))]
    pub ok: bool,
    pub data: ReplyData,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1))]
    pub state_version: Option<String>,
}

#[derive(JsonSchema)]
#[serde(untagged)]
pub enum ReplyData {
    Workspace(Workspace),
    Snapshot(Snapshot),
    Runtime(RuntimeStatus),
    Subscription(SubscriptionAck),
    ProviderCatalog(ProviderCatalog),
    ProviderDefinition(ProviderDefinition),
    Accounts(Accounts),
    Account(Box<AccountConnection>),
    Permissions(PermissionReview),
}

#[derive(JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FailureEnvelope {
    #[schemars(length(min = 1, max = 128))]
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
    Success(Box<SuccessEnvelope>),
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
    pub provider_selection: ProviderSelection,
    pub workspace_query: WorkspaceQuery,
    pub account_query: AccountQuery,
    pub account_mutation: AccountMutation,
    pub provider_connect: Connect,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Workspace {
    #[schemars(length(min = 1))]
    pub workspace_id: String,
    pub name: String,
    pub base_currency: String,
    pub path: String,
    pub created_at: String,
    pub last_opened_at: String,
    #[schemars(range(min = 1, max = 2))]
    pub storage_schema_version: u32,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DomainProjection {
    Workspace(Workspace),
    Account(Box<AccountConnection>),
}

impl DomainProjection {
    pub fn id(&self) -> &str {
        match self {
            Self::Workspace(w) => &w.workspace_id,
            Self::Account(a) => &a.connection_id,
        }
    }
    pub fn kind(&self) -> &str {
        match self {
            Self::Workspace(_) => "workspace",
            Self::Account(_) => "account",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DomainEvent {
    #[schemars(length(min = 1))]
    pub event_id: String,
    #[schemars(extend("enum" = ["workspace.opened", "account.health.changed"]))]
    pub event_type: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    pub occurred_at: String,
    #[schemars(extend("enum" = ["workspace", "account"]))]
    pub aggregate_type: String,
    #[schemars(length(min = 1))]
    pub aggregate_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub sequence: u64,
    pub payload: DomainProjection,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snapshot {
    #[schemars(extend("enum" = ["workspace", "account"]))]
    pub aggregate_type: String,
    #[schemars(length(min = 1))]
    pub aggregate_id: String,
    pub projection: DomainProjection,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub last_sequence: u64,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Remediation {
    #[schemars(length(min = 1))]
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
            "PROVIDER_ALREADY_CONNECTED" => (
                "This account is already connected in this environment. Use the existing connection.",
                "select_account",
                "Open existing connection",
            ),
            "PROVIDER_UNSUPPORTED" => (
                "This provider/environment is not supported in this build.",
                "choose_provider",
                "Choose a supported provider",
            ),
            "PROVIDER_NATIVE_ENTRY_REQUIRED" => (
                "Open the desktop app to enter credentials securely.",
                "open_desktop",
                "Open TradeX",
            ),
            "PROVIDER_ENTRY_BUSY" => (
                "Finish or cancel the open credential window before starting another connection.",
                "finish_entry",
                "Return to credential entry",
            ),
            "PROVIDER_ENTRY_CANCELLED" => (
                "Credential entry was cancelled. No connection was confirmed.",
                "connect_provider",
                "Connect again",
            ),
            "PROVIDER_AUTH_FAILED" => (
                "The provider rejected these credentials or their read permissions.",
                "reconnect_provider",
                "Reconnect with valid credentials",
            ),
            "PROVIDER_RATE_LIMITED" => (
                "The provider rate limit was reached. Wait before retrying.",
                "retry_provider",
                "Retry later",
            ),
            "PROVIDER_UNAVAILABLE" => (
                "The provider could not be reached. Saved observations are stale.",
                "retry_provider",
                "Retry connection",
            ),
            "CLOCK_SKEW" => (
                "Provider signing time could not be validated. Saved observations were preserved.",
                "retry_provider",
                "Check connectivity and retry server-time synchronization",
            ),
            "PROVIDER_RESPONSE_INVALID" => (
                "The provider response could not be validated. Saved observations were preserved.",
                "retry_provider",
                "Retry connection",
            ),
            "PROVIDER_DATA_INCOMPLETE" => (
                "The provider result is incomplete. It cannot be treated as a fresh account snapshot.",
                "retry_provider",
                "Retry connection",
            ),
            "PROVIDER_IDENTITY_CHANGED" => (
                "The provider returned a different account identity. Disconnect and reconnect.",
                "reconnect_provider",
                "Reconnect account",
            ),
            "PROVIDER_REVIEW_REQUIRED" => (
                "Test this connection and explicitly review its current permissions before confirming.",
                "review_permissions",
                "Review permissions",
            ),
            "PROVIDER_PERMISSION_BLOCKED" => (
                "Forbidden or unsupported permissions block readiness. Remove them at the provider and test again.",
                "review_permissions",
                "Review permissions",
            ),
            "CREDENTIAL_UNAVAILABLE" => (
                "The OS Keychain credential is missing or unavailable. Reconnect this account.",
                "reconnect_provider",
                "Reconnect account",
            ),
            "CREDENTIAL_STORE_FAILED" => (
                "The credential could not be saved to OS Keychain. No file fallback was used.",
                "retry_provider",
                "Retry secure entry",
            ),
            "CREDENTIAL_DELETE_FAILED" => (
                "Local access is stopped, but Keychain cleanup needs another attempt.",
                "disconnect_provider",
                "Retry Disconnect",
            ),
            _ => (
                "The control plane could not complete this operation.",
                "reload_snapshot",
                "Reload state",
            ),
        };
        Self {
            category: if code == "PROVIDER_AUTH_FAILED" || code.starts_with("CREDENTIAL_") {
                "AUTH_ERROR"
            } else if code == "PROVIDER_RATE_LIMITED" {
                "RATE_LIMITED"
            } else if code == "PROVIDER_UNAVAILABLE" {
                "NETWORK_ERROR"
            } else if code == "PROVIDER_UNSUPPORTED" {
                "UNSUPPORTED_CAPABILITY"
            } else if code == "PROVIDER_PERMISSION_BLOCKED" {
                "PERMISSION_ERROR"
            } else if matches!(
                code,
                "IPC_AGGREGATE_NOT_FOUND"
                    | "STATE_VERSION_CONFLICT"
                    | "IPC_REPLAY_UNAVAILABLE"
                    | "CLOCK_SKEW"
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
