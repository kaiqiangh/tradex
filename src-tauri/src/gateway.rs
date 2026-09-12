use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GatewayAction {
    Launch,
    Probe,
    Restart,
    Stop,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GatewayMutation {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    pub action: GatewayAction,
}

pub struct GatewayJob {
    pub(crate) request_id: String,
    pub(crate) session: String,
    pub(crate) action: GatewayAction,
    pub(crate) state: GatewayState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GatewayStatus {
    Stopped,
    Installing,
    Starting,
    Running,
    PortConflict,
    Unauthorized,
    Backoff,
    Failed,
    Stopping,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GatewayState {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(extend("const" = "7.2.155"))]
    pub pinned_version: String,
    #[schemars(extend("const" = "http://127.0.0.1:8317"))]
    pub endpoint: String,
    pub status: GatewayStatus,
    pub desired_running: bool,
    pub installed: bool,
    pub model_available: bool,
    #[schemars(range(max = 1000))]
    pub discovered_model_count: u32,
    pub last_probe_at: Option<String>,
    pub next_retry_at: Option<String>,
    #[schemars(range(max = 3))]
    pub restart_attempts: u32,
    pub error_code: Option<String>,
    pub updated_at: String,
}

impl GatewayState {
    pub fn stopped(workspace_id: String) -> Self {
        Self {
            workspace_id,
            state_version: String::new(),
            pinned_version: "7.2.155".into(),
            endpoint: "http://127.0.0.1:8317".into(),
            status: GatewayStatus::Stopped,
            desired_running: false,
            installed: false,
            model_available: false,
            discovered_model_count: 0,
            last_probe_at: None,
            next_retry_at: None,
            restart_attempts: 0,
            error_code: None,
            updated_at: String::new(),
        }
    }
}
