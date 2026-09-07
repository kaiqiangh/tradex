#[cfg(all(feature = "desktop", target_os = "macos"))]
pub mod native_credentials;
pub mod protocol;
pub mod provider_io;
pub mod providers;
mod storage;

use protocol::{
    Aggregate, CommandEnvelope, DomainProjection, EmptyPayload, EventSink, MAX_SEQUENCE,
    OpenWorkspace, Result, RuntimeComponent, RuntimeStatus, Subscribe, TradeXError,
};
use provider_io::{JobKind, ProviderJob, ProviderOutcome};
use providers::*;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::{collections::HashMap, path::PathBuf};
use storage::Store;

pub struct ControlPlane {
    default_workspace: PathBuf,
    store: Option<Store>,
    subscribers: HashMap<(String, String, String), EventSink>,
    session: String,
}

impl ControlPlane {
    pub fn new(default_workspace: PathBuf) -> Self {
        Self {
            default_workspace,
            store: None,
            subscribers: HashMap::new(),
            session: uuid::Uuid::new_v4().to_string(),
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
                    self.publish(&event);
                    let version = format!("{}:{}", event.aggregate_id, event.sequence);
                    Ok((json!(event.payload), Some(version)))
                } else {
                    let mut store = Store::open(path, &input)?;
                    store.mark_accounts_stale()?;
                    let event = store.record_open()?;
                    let version = format!("{}:{}", event.aggregate_id, event.sequence);
                    self.store = Some(store);
                    self.subscribers.clear();
                    self.session = uuid::Uuid::new_v4().to_string();
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
                let snapshot = store.snapshot_for(&input.aggregate_type, &input.aggregate_id)?;
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
                let snapshot = store.snapshot_for(&input.aggregate_type, &input.aggregate_id)?;
                if input.aggregate_type != snapshot.aggregate_type
                    || input.aggregate_id != snapshot.aggregate_id
                {
                    return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
                }
                let sink =
                    sink.ok_or_else(|| TradeXError::new("IPC_SUBSCRIPTION_CHANNEL_REQUIRED"))?;
                match store.replay(
                    &input.aggregate_type,
                    &input.aggregate_id,
                    input.after_sequence,
                    &sink,
                ) {
                    Ok(ack) => {
                        self.subscribers.insert(
                            (
                                consumer.to_owned(),
                                input.aggregate_type,
                                input.aggregate_id,
                            ),
                            sink,
                        );
                        Ok((json!(ack), None))
                    }
                    Err(error) => {
                        if error.code == "IPC_SUBSCRIPTION_DELIVERY_FAILED" {
                            self.subscribers.remove(&(
                                consumer.to_owned(),
                                input.aggregate_type,
                                input.aggregate_id,
                            ));
                        }
                        Err(error)
                    }
                }
            }
            "provider.list_definitions" => {
                let _: EmptyPayload = payload(request.payload)?;
                Ok((json!(catalog()), None))
            }
            "provider.get_schema" => {
                let p: ProviderSelection = payload(request.payload)?;
                Ok((json!(definition(&p.provider_id, &p.environment)?), None))
            }
            "account.list" => {
                let p: WorkspaceQuery = payload(request.payload)?;
                self.require_workspace(&p.workspace_id)?;
                Ok((
                    json!(Accounts {
                        accounts: self.store.as_ref().unwrap().accounts()?
                    }),
                    None,
                ))
            }
            "account.get" | "provider.permissions" => {
                let p: AccountQuery = payload(request.payload)?;
                self.require_workspace(&p.workspace_id)?;
                validate_aggregate("account", &p.connection_id)?;
                let a = self.store.as_ref().unwrap().account(&p.connection_id)?;
                let data = if request.command == "provider.permissions" {
                    json!(a.permissions)
                } else {
                    json!(a)
                };
                Ok((data, Some(a.state_version)))
            }
            "provider.connect" => match payload::<Connect>(request.payload)? {
                Connect::Test {
                    workspace_id,
                    provider_id,
                    environment,
                    label,
                } => {
                    self.validate_connection(&workspace_id, &provider_id, &environment, &label)?;
                    Err(TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED"))
                }
                Connect::Confirm {
                    workspace_id,
                    connection_id,
                    expected_state_version,
                    acknowledge_unverified,
                } => {
                    let mut a = self.current_account(
                        &workspace_id,
                        &connection_id,
                        &expected_state_version,
                    )?;
                    if a.connection_state != ConnectionState::ReviewRequired
                        || a.health.authentication != "VALID"
                        || a.health.connection != "ONLINE"
                    {
                        return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
                    }
                    if a.blocked_permissions() {
                        return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED"));
                    }
                    if a.permissions.scope == "UNVERIFIED" && !acknowledge_unverified {
                        return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
                    }
                    a.permissions.acknowledged = acknowledge_unverified;
                    a.connection_state = ConnectionState::Connected;
                    let a = self.persist_account(a)?;
                    Ok((json!(a), Some(a.state_version)))
                }
            },
            "provider.probe" | "account.refresh" | "provider.disconnect" => {
                let p: AccountMutation = payload(request.payload)?;
                self.current_account(&p.workspace_id, &p.connection_id, &p.expected_state_version)?;
                Err(TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED"))
            }
            _ => Err(TradeXError::new("IPC_COMMAND_UNKNOWN")),
        }
    }
    fn publish(&mut self, event: &protocol::DomainEvent) {
        self.subscribers.retain(|(_, kind, id), sink| {
            kind != &event.aggregate_type || id != &event.aggregate_id || sink(event.clone())
        });
    }

    fn require_workspace(&self, id: &str) -> Result<()> {
        validate_aggregate("workspace", id)?;
        if self
            .store
            .as_ref()
            .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?
            .workspace_id()?
            != id
        {
            return Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"));
        }
        Ok(())
    }

    fn validate_connection(
        &self,
        workspace: &str,
        provider: &str,
        environment: &str,
        label: &str,
    ) -> Result<()> {
        self.require_workspace(workspace)?;
        definition(provider, environment)?;
        if label.trim().is_empty()
            || label.chars().count() > 120
            || label.chars().any(char::is_control)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        Ok(())
    }

    fn current_account(
        &self,
        workspace: &str,
        id: &str,
        version: &str,
    ) -> Result<AccountConnection> {
        self.require_workspace(workspace)?;
        validate_aggregate("account", id)?;
        if version.is_empty() || version.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let a = self.store.as_ref().unwrap().account(id)?;
        if a.state_version != version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        Ok(a)
    }

    fn persist_account(&mut self, account: AccountConnection) -> Result<AccountConnection> {
        let event = self.store.as_mut().unwrap().save_account(account)?;
        self.publish(&event);
        match event.payload {
            DomainProjection::Account(account) => Ok(*account),
            _ => unreachable!(),
        }
    }

    /// Prepare under the domain lock, perform native/provider work outside it, then commit with the same session/version.
    pub fn prepare_provider(&mut self, value: &Value) -> Result<Option<ProviderJob>> {
        match value.get("schemaVersion").and_then(Value::as_u64) {
            Some(1) => (),
            Some(_) => return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED")),
            None => return Err(TradeXError::new("IPC_PAYLOAD_INVALID")),
        }
        let request: CommandEnvelope = payload(value.clone())?;
        if request.request_id.is_empty() || request.request_id.len() > 128 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let (account, kind) = match request.command.as_str() {
            "provider.connect" => match payload::<Connect>(request.payload)? {
                Connect::Test {
                    workspace_id,
                    provider_id,
                    environment,
                    label,
                } => {
                    self.validate_connection(&workspace_id, &provider_id, &environment, &label)?;
                    let account = AccountConnection::new(
                        workspace_id,
                        provider_id,
                        environment,
                        label.trim().into(),
                    )?;
                    (self.persist_account(account)?, JobKind::Connect)
                }
                Connect::Confirm { .. } => return Ok(None),
            },
            "provider.probe" | "account.refresh" | "provider.disconnect" => {
                let p: AccountMutation = payload(request.payload)?;
                let mut a = self.current_account(
                    &p.workspace_id,
                    &p.connection_id,
                    &p.expected_state_version,
                )?;
                if request.command == "provider.disconnect" {
                    a.connection_state = ConnectionState::Disconnected;
                    a.permissions.acknowledged = false;
                    a.health.connection = "DISCONNECTED".into();
                    a.health.authentication = "UNVERIFIED".into();
                    a.health.credential = "DELETE_PENDING".into();
                    a.health.reason = "Local access stopped. External orders and the provider API key are unchanged.".into();
                    (self.persist_account(a)?, JobKind::Disconnect)
                } else {
                    if matches!(
                        a.connection_state,
                        ConnectionState::Disconnected | ConnectionState::Connecting
                    ) {
                        return Err(TradeXError::new("PROVIDER_REVIEW_REQUIRED"));
                    }
                    if matches!(a.health.credential.as_str(), "MISSING" | "DELETE_PENDING") {
                        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
                    }
                    (a, JobKind::Probe)
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(ProviderJob {
            account,
            kind,
            session: self.session.clone(),
            request_id: request.request_id,
        }))
    }

    pub fn provider_job_current(&self, job: &ProviderJob) -> bool {
        job.session == self.session
            && self
                .current_account(
                    &job.account.workspace_id,
                    &job.account.connection_id,
                    &job.account.state_version,
                )
                .is_ok()
    }

    pub fn record_credential_cleanup(&mut self, job: &ProviderJob, cleanup: Result<()>) {
        if job.session != self.session || self.require_workspace(&job.account.workspace_id).is_err()
        {
            return;
        }
        if let Ok(mut account) = self
            .store
            .as_ref()
            .unwrap()
            .account(&job.account.connection_id)
        {
            if !matches!(
                account.connection_state,
                ConnectionState::Connecting
                    | ConnectionState::Disconnected
                    | ConnectionState::Failed
            ) {
                return;
            }
            let failed = account.connection_state == ConnectionState::Failed;
            if !failed {
                account.connection_state = ConnectionState::Disconnected;
                account.health.connection = "DISCONNECTED".into();
                account.health.authentication = "UNVERIFIED".into();
            }
            account.health.credential = if cleanup.is_ok() {
                "MISSING"
            } else {
                "DELETE_PENDING"
            }
            .into();
            account.health.reason = if cleanup.is_ok() && failed {
                "The connection test failed and its credential was removed. Start a new connection."
            } else if cleanup.is_ok() {
                "The abandoned connection was disconnected and its credential removed."
            } else {
                "Local access is stopped. Retry Disconnect to complete Keychain cleanup."
            }
            .into();
            let _ = self.persist_account(account); // On storage failure the durable pending reference is checked again on reopen.
        }
    }

    pub fn complete_provider(&mut self, job: &ProviderJob, outcome: ProviderOutcome) -> Value {
        let result = (|| -> Result<AccountConnection> {
            if !self.provider_job_current(job) {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let mut a = job.account.clone();
            a.health.credential = outcome.credential;
            if job.kind == JobKind::Disconnect {
                if outcome.error.is_some() {
                    a.health.reason =
                        "Local access is stopped, but Keychain cleanup failed. Retry Disconnect."
                            .into();
                }
                return self.persist_account(a);
            }
            if let Some(error) = outcome.error {
                if job.kind == JobKind::Connect {
                    a.health.credential = "DELETE_PENDING".into();
                }
                if error.code == "PROVIDER_ENTRY_CANCELLED" {
                    a.connection_state = ConnectionState::Disconnected;
                    a.health.connection = "DISCONNECTED".into();
                } else {
                    a.connection_state = ConnectionState::Failed;
                    a.health.connection = "ERROR".into();
                }
                a.health.authentication = if error.code == "PROVIDER_AUTH_FAILED" {
                    "INVALID"
                } else {
                    "UNVERIFIED"
                }
                .into();
                a.health.reason = error.message.clone();
                self.persist_account(a)?;
                return Err(error);
            }
            let observed = outcome
                .observation
                .ok_or_else(|| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
            if a.data
                .as_ref()
                .is_some_and(|old| old.remote_account_id != observed.data.remote_account_id)
            {
                a.connection_state = ConnectionState::Failed;
                a.health.connection = "ERROR".into();
                a.health.authentication = "UNVERIFIED".into();
                a.health.reason =
                    "Provider returned a different account identity. Disconnect and reconnect."
                        .into();
                self.persist_account(a)?;
                return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
            }
            if self
                .store
                .as_ref()
                .unwrap()
                .accounts()?
                .iter()
                .any(|other| {
                    other.connection_id != a.connection_id
                        && other.provider_id == a.provider_id
                        && other.environment == a.environment
                        && other.connection_state != ConnectionState::Disconnected
                        && other.data.as_ref().is_some_and(|data| {
                            data.remote_account_id == observed.data.remote_account_id
                        })
                })
            {
                a.connection_state = ConnectionState::Disconnected;
                a.health.connection = "DISCONNECTED".into();
                a.health.authentication = "UNVERIFIED".into();
                a.health.credential = "DELETE_PENDING".into();
                a.health.reason="This provider account is already connected in this environment. Use the existing connection.".into();
                self.persist_account(a)?;
                return Err(TradeXError::new("PROVIDER_ALREADY_CONNECTED"));
            }
            let mut old_scope = a.permissions.clone();
            old_scope.acknowledged = false;
            let unchanged_scope = old_scope == observed.permissions;
            let acknowledged = a.permissions.acknowledged && unchanged_scope;
            a.permissions = observed.permissions;
            a.permissions.acknowledged = acknowledged;
            if a.connection_state != ConnectionState::Connected || !unchanged_scope {
                a.connection_state = ConnectionState::ReviewRequired;
            }
            a.data = Some(observed.data);
            a.last_successful_sync = Some(storage::timestamp()?);
            a.health.connection = "ONLINE".into();
            a.health.authentication = "VALID".into();
            a.health.reason = "Read-only account data loaded. Trading, private streams and reconciliation are not configured.".into();
            self.persist_account(a)
        })();
        match result {
            Ok(a) => {
                json!({"requestId":job.request_id,"schemaVersion":1,"ok":true,"stateVersion":a.state_version,"data":a})
            }
            Err(error) => {
                json!({"requestId":job.request_id,"schemaVersion":1,"ok":false,"error":error})
            }
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
