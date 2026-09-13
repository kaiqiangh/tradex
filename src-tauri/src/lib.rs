pub mod gateway;
#[cfg(target_os = "macos")]
pub mod gateway_process;
pub mod model;
#[cfg(target_os = "macos")]
pub mod model_credentials;
#[cfg(all(feature = "desktop", target_os = "macos"))]
pub mod native_credentials;
pub mod protocol;
pub mod provider_io;
pub mod providers;
pub mod risk;
mod storage;

use protocol::{
    Aggregate, CommandEnvelope, DomainProjection, EmptyPayload, EventSink, MAX_SEQUENCE,
    OpenWorkspace, Result, RuntimeComponent, RuntimeStatus, Subscribe, TradeXError,
};
use provider_io::{JobKind, ProviderJob, ProviderOutcome};
use providers::*;
use risk::RiskPolicyState;
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
                    store.save_gateway(gateway::GatewayState::stopped(store.workspace_id()?))?;
                    let mut model = store.model_or_new()?;
                    model.reset_for_session();
                    store.save_model(model, "model.provider.changed")?;
                    match store.risk_or_new()? {
                        Some(mut risk) => {
                            let previous = risk.clone();
                            risk.reopen_after_model_reset();
                            if risk != previous {
                                store.save_risk(risk)?;
                            }
                        }
                        None => {
                            let workspace_id = store.workspace_id()?;
                            store.save_risk(RiskPolicyState::new(workspace_id))?;
                        }
                    }
                    let event = store.record_open()?;
                    let version = format!("{}:{}", event.aggregate_id, event.sequence);
                    self.store = Some(store);
                    self.subscribers.clear();
                    self.session = uuid::Uuid::new_v4().to_string();
                    Ok((json!(event.payload), Some(version)))
                }
            }
            "model.get_gateway" => {
                let input: WorkspaceQuery = payload(request.payload)?;
                self.require_workspace(&input.workspace_id)?;
                let state = self.store.as_ref().unwrap().gateway()?;
                let version = state.state_version.clone();
                Ok((json!(state), Some(version)))
            }
            "model.get" => {
                let input: model::ModelQuery = payload(request.payload)?;
                self.require_workspace(&input.workspace_id)?;
                let state = self.store.as_ref().unwrap().model()?;
                let version = state.state_version.clone();
                Ok((json!(state), Some(version)))
            }
            "model.login_chatgpt" => {
                let _: model::ChatgptLogin = payload(request.payload)?;
                Err(TradeXError::new("MODEL_NATIVE_REQUIRED"))
            }
            "model.configure_deepseek" => {
                let _: model::ConfigureDeepseek = payload(request.payload)?;
                Err(TradeXError::new("MODEL_NATIVE_REQUIRED"))
            }
            "model.verify_route" => {
                let input: model::VerifyRoute = payload(request.payload)?;
                if !model::allowed_route(
                    &input.provider,
                    &input.model_id,
                    input.thinking_type.as_ref(),
                ) {
                    return Err(TradeXError::new("MODEL_ROUTE_INVALID"));
                }
                Err(TradeXError::new("MODEL_NATIVE_REQUIRED"))
            }
            "model.set_default" => {
                let input: model::SetDefaultModel = payload(request.payload)?;
                self.set_default_model(input)
            }
            "model.set_fallback_policy" => {
                let input: model::SetFallbackPolicy = payload(request.payload)?;
                self.set_fallback_policy(input)
            }
            "risk.get_policy" => {
                let input: risk::RiskQuery = payload(request.payload)?;
                self.require_workspace(&input.workspace_id)?;
                let state = self.store.as_ref().unwrap().risk()?;
                Ok((json!(state), Some(state.state_version)))
            }
            "risk.save_policy" => {
                let input: risk::SaveRiskPolicy = payload(request.payload)?;
                self.save_risk_policy(input)
            }
            "onboarding.set_step" => {
                let input: risk::SetOnboardingStep = payload(request.payload)?;
                self.set_onboarding_step(input)
            }
            "onboarding.complete" => {
                let input: risk::CompleteOnboarding = payload(request.payload)?;
                self.complete_onboarding(input)
            }
            "runtime.status" => {
                let _: EmptyPayload = payload(request.payload)?;
                let gateway_running = self
                    .store
                    .as_ref()
                    .and_then(|store| store.gateway().ok())
                    .is_some_and(|gateway| gateway.status == gateway::GatewayStatus::Running);
                let model_available = gateway_running
                    && self
                        .store
                        .as_ref()
                        .and_then(|store| store.model().ok())
                        .is_some_and(|model| model.thread_plan().is_ok());
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
                            status: self
                                .store
                                .as_ref()
                                .and_then(|s| s.gateway().ok())
                                .and_then(|g| serde_json::to_value(g.status).ok())
                                .and_then(|v| v.as_str().map(str::to_owned))
                                .unwrap_or_else(|| "NOT_CONFIGURED".into()),
                            message:
                                "Gateway health is separate from verified model-route availability."
                                    .into(),
                        },
                        RuntimeComponent {
                            id: "order-gateway".into(),
                            status: "NOT_CONFIGURED".into(),
                            message: "Live execution is unavailable.".into(),
                        },
                    ],
                    model_available,
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
    pub fn prepare_gateway(&mut self, value: &Value) -> Result<Option<gateway::GatewayJob>> {
        if value.get("command").and_then(Value::as_str) != Some("model.gateway") {
            return Ok(None);
        }
        if value.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        let request: CommandEnvelope = payload(value.clone())?;
        if request.request_id.is_empty() || request.request_id.len() > 128 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let input: gateway::GatewayMutation = payload(request.payload)?;
        self.require_workspace(&input.workspace_id)?;
        if input.expected_state_version.is_empty() || input.expected_state_version.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let mut state = self.store.as_ref().unwrap().gateway()?;
        if state.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if matches!(
            state.status,
            gateway::GatewayStatus::Installing
                | gateway::GatewayStatus::Starting
                | gateway::GatewayStatus::Stopping
        ) && input.action != gateway::GatewayAction::Stop
        {
            return Err(TradeXError::new("GATEWAY_BUSY"));
        }
        if state.status == gateway::GatewayStatus::Backoff
            && input.action == gateway::GatewayAction::Probe
        {
            return Err(TradeXError::new("GATEWAY_BUSY"));
        }
        state.status = if input.action == gateway::GatewayAction::Stop {
            gateway::GatewayStatus::Stopping
        } else if input.action == gateway::GatewayAction::Launch && !state.installed {
            gateway::GatewayStatus::Installing
        } else {
            gateway::GatewayStatus::Starting
        };
        if input.action != gateway::GatewayAction::Probe {
            state.desired_running = input.action != gateway::GatewayAction::Stop;
        }
        state.model_available = false;
        if input.action != gateway::GatewayAction::Probe {
            state.restart_attempts = 0;
        }
        state.next_retry_at = None;
        state.error_code = None;
        let event = self.store.as_mut().unwrap().save_gateway(state)?;
        self.publish(&event);
        let DomainProjection::Gateway(state) = event.payload else {
            unreachable!()
        };
        Ok(Some(gateway::GatewayJob {
            request_id: request.request_id,
            session: self.session.clone(),
            action: input.action,
            state,
        }))
    }

    pub fn gateway_monitor_job(&self) -> Option<gateway::GatewayJob> {
        let state = self.store.as_ref()?.gateway().ok()?;
        Some(gateway::GatewayJob {
            request_id: "gateway-supervisor".into(),
            session: self.session.clone(),
            action: gateway::GatewayAction::Probe,
            state,
        })
    }

    pub fn gateway_job_current(&self, job: &gateway::GatewayJob) -> bool {
        self.session == job.session
            && self.store.as_ref().is_some_and(|store| {
                store
                    .gateway()
                    .is_ok_and(|state| state.state_version == job.state.state_version)
            })
    }

    fn set_default_model(
        &mut self,
        input: model::SetDefaultModel,
    ) -> Result<(Value, Option<String>)> {
        self.require_workspace(&input.workspace_id)?;
        if input.expected_state_version.is_empty() || input.expected_state_version.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        if !model::allowed_route(
            &input.provider,
            &input.model_id,
            input.thinking_type.as_ref(),
        ) {
            return Err(TradeXError::new("MODEL_ROUTE_INVALID"));
        }
        let previous = self.store.as_ref().unwrap().model()?;
        if previous.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let selection = model::ModelSelection {
            provider: input.provider,
            model_id: input.model_id,
            thinking_type: input.thinking_type,
        };
        let route = previous
            .verified_route(&selection)
            .ok_or_else(|| TradeXError::new("MODEL_UNAVAILABLE"))?;
        let mut state = previous;
        state.default_route = Some(selection);
        state.current_route = Some(route);
        let event = self
            .store
            .as_mut()
            .unwrap()
            .save_model(state, "model.provider.changed")?;
        let DomainProjection::Model(state) = &event.payload else {
            unreachable!()
        };
        let version = state.state_version.clone();
        self.publish(&event);
        Ok((json!(state), Some(version)))
    }

    fn set_fallback_policy(
        &mut self,
        input: model::SetFallbackPolicy,
    ) -> Result<(Value, Option<String>)> {
        self.require_workspace(&input.workspace_id)?;
        if input.expected_state_version.is_empty() || input.expected_state_version.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let previous = self.store.as_ref().unwrap().model()?;
        if previous.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if input.automatic_fallback && previous.verified_deepseek_route().is_none() {
            return Err(TradeXError::new("MODEL_FALLBACK_UNAVAILABLE"));
        }
        let mut state = previous;
        state.automatic_fallback = input.automatic_fallback;
        state.fallback_policy_version = state
            .fallback_policy_version
            .checked_add(1)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        let event = self
            .store
            .as_mut()
            .unwrap()
            .save_model(state, "model.provider.changed")?;
        let DomainProjection::Model(state) = &event.payload else {
            unreachable!()
        };
        let version = state.state_version.clone();
        self.publish(&event);
        Ok((json!(state), Some(version)))
    }

    fn save_risk_policy(&mut self, input: risk::SaveRiskPolicy) -> Result<(Value, Option<String>)> {
        self.require_workspace(&input.workspace_id)?;
        if input.expected_state_version.is_empty() || input.expected_state_version.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let previous = self.store.as_ref().unwrap().risk()?;
        if previous.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let mut state = previous;
        state.policy = input.policy;
        state.validate_policy()?;
        state.mark_editing();
        state.policy_version = state
            .policy_version
            .checked_add(1)
            .ok_or_else(|| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
        self.persist_risk(state)
    }

    fn set_onboarding_step(
        &mut self,
        input: risk::SetOnboardingStep,
    ) -> Result<(Value, Option<String>)> {
        self.require_workspace(&input.workspace_id)?;
        if input.expected_state_version.is_empty() || input.expected_state_version.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        if !(1..=5).contains(&input.step) {
            return Err(TradeXError::new("ONBOARDING_STEP_INVALID"));
        }
        let previous = self.store.as_ref().unwrap().risk()?;
        if previous.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if input.step > previous.onboarding_step.saturating_add(1)
            && !(previous.onboarding_completed && input.step == 1)
        {
            return Err(TradeXError::new("ONBOARDING_STEP_INVALID"));
        }
        if input.step == 5 {
            let model = self.store.as_ref().unwrap().model()?;
            if !previous.ready_for_completion(&model) {
                return Err(TradeXError::new("ONBOARDING_BLOCKED"));
            }
        }
        let mut state = previous;
        state.onboarding_step = input.step;
        if input.step < 5 {
            state.onboarding_completed = false;
        }
        self.persist_risk(state)
    }

    fn complete_onboarding(
        &mut self,
        input: risk::CompleteOnboarding,
    ) -> Result<(Value, Option<String>)> {
        self.require_workspace(&input.workspace_id)?;
        if input.expected_state_version.is_empty() || input.expected_state_version.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let previous = self.store.as_ref().unwrap().risk()?;
        if previous.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if previous.onboarding_step != 5 {
            return Err(TradeXError::new("ONBOARDING_STEP_INVALID"));
        }
        let model = self.store.as_ref().unwrap().model()?;
        if !previous.ready_for_completion(&model) {
            return Err(TradeXError::new("ONBOARDING_BLOCKED"));
        }
        if self
            .store
            .as_ref()
            .unwrap()
            .accounts()?
            .iter()
            .any(|account| account.environment == "LIVE" && account.health.arming != "DISARMED")
        {
            return Err(TradeXError::new("ONBOARDING_BLOCKED"));
        }
        let mut state = previous;
        state.onboarding_completed = true;
        self.persist_risk(state)
    }

    fn persist_risk(&mut self, state: RiskPolicyState) -> Result<(Value, Option<String>)> {
        let event = self.store.as_mut().unwrap().save_risk(state)?;
        let DomainProjection::Risk(state) = &event.payload else {
            unreachable!()
        };
        let version = state.state_version.clone();
        self.publish(&event);
        Ok((json!(state), Some(version)))
    }

    pub fn complete_gateway(
        &mut self,
        job: &gateway::GatewayJob,
        state: gateway::GatewayState,
    ) -> Value {
        let result =
            if self.gateway_job_current(job) && state.workspace_id == job.state.workspace_id {
                self.store.as_mut().unwrap().save_gateway(state)
            } else {
                Err(TradeXError::new("STATE_VERSION_CONFLICT"))
            };
        match result {
            Ok(event) => {
                let DomainProjection::Gateway(ref state) = event.payload else {
                    unreachable!()
                };
                let reply = json!({"requestId":job.request_id,"schemaVersion":1,"ok":true,"stateVersion":state.state_version,"data":state});
                self.publish(&event);
                reply
            }
            Err(error) => {
                json!({"requestId":job.request_id,"schemaVersion":1,"ok":false,"error":error})
            }
        }
    }

    pub fn prepare_model(&mut self, value: &Value) -> Result<Option<model::ModelJob>> {
        let command = value.get("command").and_then(Value::as_str);
        let (request, action, provider) = match command {
            Some("model.login_chatgpt") => {
                let request: CommandEnvelope = payload(value.clone())?;
                let input: model::ChatgptLogin = payload(request.payload.clone())?;
                (
                    request,
                    model::ModelAction::LoginChatgpt(input.action),
                    model::ModelProvider::Chatgpt,
                )
            }
            Some("model.configure_deepseek") => {
                let request: CommandEnvelope = payload(value.clone())?;
                let _input: model::ConfigureDeepseek = payload(request.payload.clone())?;
                (
                    request,
                    model::ModelAction::ConfigureDeepseek,
                    model::ModelProvider::Deepseek,
                )
            }
            Some("model.verify_route") => {
                let request: CommandEnvelope = payload(value.clone())?;
                let input: model::VerifyRoute = payload(request.payload.clone())?;
                if !model::allowed_route(
                    &input.provider,
                    &input.model_id,
                    input.thinking_type.as_ref(),
                ) {
                    return Err(TradeXError::new("MODEL_ROUTE_INVALID"));
                }
                let provider = input.provider.clone();
                (
                    request,
                    model::ModelAction::VerifyRoute {
                        provider: input.provider,
                        model_id: input.model_id,
                        thinking_type: input.thinking_type,
                    },
                    provider,
                )
            }
            _ => return Ok(None),
        };
        if request.schema_version != 1 {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        if request.request_id.is_empty() || request.request_id.len() > 128 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let (workspace_id, expected) = match &action {
            model::ModelAction::LoginChatgpt(_) => {
                let input: model::ChatgptLogin = payload(request.payload.clone())?;
                (input.workspace_id, input.expected_state_version)
            }
            model::ModelAction::ConfigureDeepseek => {
                let input: model::ConfigureDeepseek = payload(request.payload.clone())?;
                (input.workspace_id, input.expected_state_version)
            }
            model::ModelAction::VerifyRoute { .. } => {
                let input: model::VerifyRoute = payload(request.payload.clone())?;
                (input.workspace_id, input.expected_state_version)
            }
        };
        self.require_workspace(&workspace_id)?;
        if expected.is_empty() || expected.len() > 256 {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let previous = self.store.as_ref().unwrap().model()?;
        if previous.state_version != expected {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if matches!(&action, model::ModelAction::VerifyRoute { .. })
            && previous.retry_blocked(&provider, time::OffsetDateTime::now_utc())
        {
            return Err(TradeXError::new("MODEL_QUOTA_COOLDOWN"));
        }
        if matches!(&action, model::ModelAction::ConfigureDeepseek) {
            let gateway = self.store.as_ref().unwrap().gateway()?;
            if gateway.desired_running || gateway.status == gateway::GatewayStatus::Stopping {
                return Err(TradeXError::new("MODEL_GATEWAY_RUNNING"));
            }
        }
        if previous.provider(&provider).status == model::ModelHealth::Verifying {
            return Err(TradeXError::new("MODEL_BUSY"));
        }
        if matches!(&action, model::ModelAction::VerifyRoute { .. })
            && !previous.provider(&provider).configured
        {
            return Err(match provider {
                model::ModelProvider::Chatgpt => TradeXError::new("MODEL_OAUTH_EXPIRED"),
                model::ModelProvider::Deepseek => TradeXError::new("MODEL_KEYCHAIN_MISSING"),
            });
        }
        if let model::ModelAction::VerifyRoute {
            provider: model::ModelProvider::Chatgpt,
            model_id,
            thinking_type,
        } = &action
            && !previous.chatgpt.routes.iter().any(|route| {
                route.model_id == *model_id
                    && route.thinking_type.as_ref() == thinking_type.as_ref()
            })
        {
            return Err(TradeXError::new("MODEL_UNAVAILABLE"));
        }
        let mut state = previous.clone();
        state.provider_mut(&provider).status = model::ModelHealth::Verifying;
        state.provider_mut(&provider).error_code = None;
        let event = self
            .store
            .as_mut()
            .unwrap()
            .save_model(state.clone(), "model.provider.changed")?;
        self.publish(&event);
        let DomainProjection::Model(state) = event.payload else {
            unreachable!()
        };
        Ok(Some(model::ModelJob {
            request_id: request.request_id,
            session: self.session.clone(),
            action,
            previous,
            state,
        }))
    }

    pub fn model_job_current(&self, job: &model::ModelJob) -> bool {
        job.session == self.session
            && self.store.as_ref().is_some_and(|store| {
                store
                    .model()
                    .is_ok_and(|state| state.state_version == job.state.state_version)
            })
    }

    pub fn complete_model(&mut self, job: &model::ModelJob, outcome: model::ModelOutcome) -> Value {
        if !self.model_job_current(job) {
            return json!({"requestId":job.request_id,"schemaVersion":1,"ok":false,"error":TradeXError::new("STATE_VERSION_CONFLICT")});
        }
        let provider = match &job.action {
            model::ModelAction::LoginChatgpt(_) => model::ModelProvider::Chatgpt,
            model::ModelAction::ConfigureDeepseek => model::ModelProvider::Deepseek,
            model::ModelAction::VerifyRoute { provider, .. } => provider.clone(),
        };
        let mut state = if let Some(error) = &outcome.error {
            let mut restored = job.previous.clone();
            let current_route_matches_provider = restored
                .current_route
                .as_ref()
                .is_some_and(|route| route.provider == provider);
            let provider_state = restored.provider_mut(&provider);
            if error.code == "MODEL_ENTRY_CANCELLED" {
                provider_state.error_code = None;
            } else if matches!(&job.action, model::ModelAction::VerifyRoute { .. }) {
                provider_state.status = if provider_state.configured {
                    model::ModelHealth::Unverified
                } else {
                    model::ModelHealth::NotConfigured
                };
                provider_state.error_code = Some(error.code.clone());
                if current_route_matches_provider {
                    restored.current_route = None;
                }
            } else {
                provider_state.status = if provider_state.configured {
                    model::ModelHealth::Failed
                } else {
                    model::ModelHealth::NotConfigured
                };
                provider_state.error_code = Some(error.code.clone());
            }
            restored
        } else {
            let mut completed = job.state.clone();
            if !matches!(&job.action, model::ModelAction::VerifyRoute { .. })
                && completed
                    .current_route
                    .as_ref()
                    .is_some_and(|route| route.provider == provider)
            {
                completed.current_route = None;
            }
            let provider_state = completed.provider_mut(&provider);
            provider_state.configured = outcome.configured.unwrap_or(provider_state.configured);
            provider_state.error_code = None;
            match &job.action {
                model::ModelAction::VerifyRoute {
                    model_id,
                    thinking_type,
                    ..
                } => {
                    let verified_at = Some(outcome.attempt.ended_at.clone());
                    let route = model::ModelRoute {
                        provider: provider.clone(),
                        model_id: model_id.clone(),
                        thinking_type: thinking_type.clone(),
                        verified_at,
                    };
                    if let Some(existing) = provider_state.routes.iter_mut().find(|existing| {
                        existing.model_id == route.model_id
                            && existing.thinking_type == route.thinking_type
                    }) {
                        *existing = route.clone();
                    } else {
                        if provider_state.routes.len() >= 100 {
                            provider_state.routes.truncate(99);
                        }
                        provider_state.routes.push(route.clone());
                    }
                    provider_state.status = model::ModelHealth::Ready;
                    provider_state.last_verified_at = route.verified_at.clone();
                    let selection = route.selection();
                    if completed.default_route.is_none() {
                        completed.default_route = Some(selection.clone());
                    }
                    if completed.default_route.as_ref() == Some(&selection) {
                        completed.current_route = Some(route);
                    }
                }
                _ => {
                    provider_state.routes = outcome.routes;
                    provider_state.status = if provider_state.configured {
                        model::ModelHealth::Unverified
                    } else {
                        model::ModelHealth::NotConfigured
                    };
                    provider_state.last_verified_at = None;
                }
            }
            completed
        };
        state.append_attempt(outcome.attempt);
        let result = self
            .store
            .as_mut()
            .unwrap()
            .save_model(state, "model.provider_attempt.changed")
            .map(|event| {
                self.publish(&event);
                let DomainProjection::Model(state) = event.payload else {
                    unreachable!()
                };
                json!({"requestId":job.request_id,"schemaVersion":1,"ok":true,"stateVersion":state.state_version,"data":state})
            });
        result.unwrap_or_else(
            |error| json!({"requestId":job.request_id,"schemaVersion":1,"ok":false,"error":error}),
        )
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

#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn failed_route_verification_stays_unverified_and_clears_current_route() {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let opened = control.dispatch(json!({
            "requestId":"open",
            "schemaVersion":1,
            "command":"workspace.open",
            "payload":{}
        }));
        let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
        let mut model = control.store.as_ref().unwrap().model().unwrap();
        let route = model::ModelRoute {
            provider: model::ModelProvider::Deepseek,
            model_id: "deepseek-v4-flash".into(),
            thinking_type: Some(model::ThinkingType::Disabled),
            verified_at: Some("2026-09-12T00:00:00Z".into()),
        };
        model.deepseek.configured = true;
        model.deepseek.status = model::ModelHealth::Ready;
        model.deepseek.routes = vec![route.clone()];
        model.deepseek.last_verified_at = route.verified_at.clone();
        model.current_route = Some(route);
        control
            .store
            .as_mut()
            .unwrap()
            .save_model(model, "model.provider.changed")
            .unwrap();
        let current = control.dispatch(json!({
            "requestId":"get",
            "schemaVersion":1,
            "command":"model.get",
            "payload":{"workspaceId":workspace_id}
        }));
        let request = json!({
            "requestId":"verify",
            "schemaVersion":1,
            "command":"model.verify_route",
            "payload":{"workspaceId":workspace_id,"expectedStateVersion":current["data"]["stateVersion"],"provider":"DEEPSEEK","modelId":"deepseek-v4-flash","thinkingType":"disabled"}
        });
        let job = control.prepare_model(&request).unwrap().unwrap();
        let outcome = model::ModelOutcome {
            attempt: model::ModelAttempt {
                attempt_id: "attempt-1".into(),
                provider: model::ModelProvider::Deepseek,
                model_id: Some("deepseek-v4-flash".into()),
                thinking_type: Some(model::ThinkingType::Disabled),
                started_at: "2026-09-12T00:00:00Z".into(),
                ended_at: "2026-09-12T00:00:01Z".into(),
                kind: model::ModelAttemptKind::Setup,
                outcome: model::ModelAttemptOutcome::Failed,
                error_category: Some("MODEL_UNAVAILABLE".into()),
                quota: None,
            },
            routes: vec![],
            configured: None,
            error: Some(TradeXError::new("MODEL_TEST_INFERENCE_FAILED")),
        };
        let reply = control.complete_model(&job, outcome);
        assert_eq!(reply["ok"], true, "{reply}");
        assert_eq!(reply["data"]["deepseek"]["status"], "UNVERIFIED");
        assert_eq!(reply["data"]["currentRoute"], Value::Null);
        assert_eq!(
            reply["data"]["deepseek"]["routes"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            reply["data"]["deepseek"]["errorCode"],
            "MODEL_TEST_INFERENCE_FAILED"
        );
    }

    #[test]
    fn model_routing_commands_persist_default_and_require_verified_fallback() {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let opened = control.dispatch(json!({
            "requestId":"open",
            "schemaVersion":1,
            "command":"workspace.open",
            "payload":{}
        }));
        let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
        let chatgpt = model::ModelRoute {
            provider: model::ModelProvider::Chatgpt,
            model_id: "gpt-5.6-sol".into(),
            thinking_type: None,
            verified_at: Some("2026-09-13T00:00:00Z".into()),
        };
        let deepseek = model::ModelRoute {
            provider: model::ModelProvider::Deepseek,
            model_id: "deepseek-v4-flash".into(),
            thinking_type: Some(model::ThinkingType::Disabled),
            verified_at: Some("2026-09-13T00:00:00Z".into()),
        };
        let mut seeded = control.store.as_ref().unwrap().model().unwrap();
        seeded.chatgpt.configured = true;
        seeded.chatgpt.status = model::ModelHealth::Ready;
        seeded.chatgpt.routes = vec![chatgpt.clone()];
        seeded.deepseek.configured = true;
        seeded.deepseek.status = model::ModelHealth::Ready;
        seeded.deepseek.routes = vec![deepseek.clone()];
        control
            .store
            .as_mut()
            .unwrap()
            .save_model(seeded, "model.provider.changed")
            .unwrap();
        let current = control.dispatch(json!({
            "requestId":"get",
            "schemaVersion":1,
            "command":"model.get",
            "payload":{"workspaceId":workspace_id}
        }));
        let set_default = control.dispatch(json!({
            "requestId":"default",
            "schemaVersion":1,
            "command":"model.set_default",
            "payload":{"workspaceId":workspace_id,"expectedStateVersion":current["data"]["stateVersion"],"provider":"CHATGPT","modelId":"gpt-5.6-sol","thinkingType":null}
        }));
        assert_eq!(set_default["ok"], true, "{set_default}");
        assert_eq!(
            set_default["data"]["defaultRoute"]["modelId"],
            "gpt-5.6-sol"
        );
        assert_eq!(set_default["data"]["automaticFallback"], false);
        assert_eq!(set_default["data"]["fallbackPolicyVersion"], 1);
        let set_fallback = control.dispatch(json!({
            "requestId":"fallback",
            "schemaVersion":1,
            "command":"model.set_fallback_policy",
            "payload":{"workspaceId":workspace_id,"expectedStateVersion":set_default["data"]["stateVersion"],"automaticFallback":true}
        }));
        assert_eq!(set_fallback["ok"], true, "{set_fallback}");
        assert_eq!(set_fallback["data"]["automaticFallback"], true);
        assert_eq!(set_fallback["data"]["fallbackPolicyVersion"], 2);
        let stale = control.dispatch(json!({
            "requestId":"stale",
            "schemaVersion":1,
            "command":"model.set_default",
            "payload":{"workspaceId":workspace_id,"expectedStateVersion":current["data"]["stateVersion"],"provider":"DEEPSEEK","modelId":"deepseek-v4-flash","thinkingType":"disabled"}
        }));
        assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    }

    #[test]
    fn enabling_fallback_without_a_verified_deepseek_route_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let opened = control.dispatch(json!({
            "requestId":"open",
            "schemaVersion":1,
            "command":"workspace.open",
            "payload":{}
        }));
        let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
        let current = control.dispatch(json!({
            "requestId":"get",
            "schemaVersion":1,
            "command":"model.get",
            "payload":{"workspaceId":workspace_id}
        }));
        let reply = control.dispatch(json!({
            "requestId":"fallback",
            "schemaVersion":1,
            "command":"model.set_fallback_policy",
            "payload":{"workspaceId":workspace_id,"expectedStateVersion":current["data"]["stateVersion"],"automaticFallback":true}
        }));
        assert_eq!(reply["error"]["code"], "MODEL_FALLBACK_UNAVAILABLE");
    }

    #[test]
    fn model_verify_is_blocked_during_known_quota_cooldown() {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let opened = control.dispatch(json!({
            "requestId":"open",
            "schemaVersion":1,
            "command":"workspace.open",
            "payload":{}
        }));
        let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
        let route = model::ModelRoute {
            provider: model::ModelProvider::Chatgpt,
            model_id: "gpt-5.6-sol".into(),
            thinking_type: None,
            verified_at: Some("2026-09-13T00:00:00Z".into()),
        };
        let mut seeded = control.store.as_ref().unwrap().model().unwrap();
        seeded.chatgpt.configured = true;
        seeded.chatgpt.status = model::ModelHealth::Ready;
        seeded.chatgpt.routes = vec![route.clone()];
        seeded.default_route = Some(route.selection());
        seeded.current_route = Some(route);
        seeded.append_attempt(model::ModelAttempt {
            attempt_id: "quota-cooldown".into(),
            provider: model::ModelProvider::Chatgpt,
            model_id: Some("gpt-5.6-sol".into()),
            thinking_type: None,
            started_at: "2099-01-01T00:00:00Z".into(),
            ended_at: "2099-01-01T00:00:00Z".into(),
            kind: model::ModelAttemptKind::Thread,
            outcome: model::ModelAttemptOutcome::Failed,
            error_category: Some("QUOTA_EXCEEDED".into()),
            quota: Some(model::ModelQuota {
                window: Some("retry-after:60s".into()),
                reset_at: None,
                retry_after_seconds: Some(60),
                remaining: None,
            }),
        });
        control
            .store
            .as_mut()
            .unwrap()
            .save_model(seeded, "model.provider_attempt.changed")
            .unwrap();
        let current = control.dispatch(json!({
            "requestId":"get",
            "schemaVersion":1,
            "command":"model.get",
            "payload":{"workspaceId":workspace_id}
        }));
        let request = json!({
            "requestId":"verify",
            "schemaVersion":1,
            "command":"model.verify_route",
            "payload":{"workspaceId":workspace_id,"expectedStateVersion":current["data"]["stateVersion"],"provider":"CHATGPT","modelId":"gpt-5.6-sol","thinkingType":null}
        });
        let error = match control.prepare_model(&request) {
            Err(error) => error,
            Ok(_) => panic!("known quota cooldown must block verification"),
        };
        assert_eq!(error.code, "MODEL_QUOTA_COOLDOWN");
    }
}

#[cfg(test)]
mod risk_tests {
    use super::*;

    fn command(control: &mut ControlPlane, command: &str, payload: Value) -> Value {
        control.dispatch(json!({
            "requestId": command,
            "schemaVersion": 1,
            "command": command,
            "payload": payload,
        }))
    }

    #[test]
    fn risk_policy_is_persisted_and_ready_is_gated_by_model_and_order() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace");
        let mut control = ControlPlane::new(path.clone());
        let opened = command(&mut control, "workspace.open", json!({}));
        let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
        let risk = command(
            &mut control,
            "risk.get_policy",
            json!({"workspaceId":workspace_id}),
        );
        assert_eq!(risk["data"]["policy"]["maxOrderNotional"], Value::Null);
        assert_eq!(risk["data"]["policy"]["staleQuoteThresholdSeconds"], 3);
        assert_eq!(risk["data"]["policy"]["marketOrdersEnabled"], false);
        assert_eq!(risk["data"]["policy"]["liveInactivityTimeoutMinutes"], 20);
        let invalid = command(
            &mut control,
            "risk.save_policy",
            json!({"workspaceId":workspace_id,"expectedStateVersion":risk["data"]["stateVersion"],"policy":{"maxOrderNotional":"1e3","staleQuoteThresholdSeconds":3,"marketOrdersEnabled":false,"liveInactivityTimeoutMinutes":20}}),
        );
        assert_eq!(invalid["error"]["code"], "RISK_POLICY_INVALID");
        let saved = command(
            &mut control,
            "risk.save_policy",
            json!({"workspaceId":workspace_id,"expectedStateVersion":risk["data"]["stateVersion"],"policy":{"maxOrderNotional":null,"maxSingleInstrumentExposurePercent":"10.25","maxDailyTradedNotional":null,"maxDailyRealizedLoss":null,"staleQuoteThresholdSeconds":3,"marketOrdersEnabled":false,"liveInactivityTimeoutMinutes":20}}),
        );
        assert_eq!(saved["ok"], true, "{saved}");
        assert_eq!(saved["data"]["configured"], true);
        assert_eq!(saved["data"]["policyVersion"], 2);
        let skipped = command(
            &mut control,
            "onboarding.set_step",
            json!({"workspaceId":workspace_id,"expectedStateVersion":saved["data"]["stateVersion"],"step":3}),
        );
        assert_eq!(skipped["error"]["code"], "ONBOARDING_STEP_INVALID");
        let step_two = command(
            &mut control,
            "onboarding.set_step",
            json!({"workspaceId":workspace_id,"expectedStateVersion":saved["data"]["stateVersion"],"step":2}),
        );
        let step_three = command(
            &mut control,
            "onboarding.set_step",
            json!({"workspaceId":workspace_id,"expectedStateVersion":step_two["data"]["stateVersion"],"step":3}),
        );
        let step_four = command(
            &mut control,
            "onboarding.set_step",
            json!({"workspaceId":workspace_id,"expectedStateVersion":step_three["data"]["stateVersion"],"step":4}),
        );
        let blocked = command(
            &mut control,
            "onboarding.set_step",
            json!({"workspaceId":workspace_id,"expectedStateVersion":step_four["data"]["stateVersion"],"step":5}),
        );
        assert_eq!(blocked["error"]["code"], "ONBOARDING_BLOCKED");
        let route = model::ModelRoute {
            provider: model::ModelProvider::Chatgpt,
            model_id: "gpt-5.6-sol".into(),
            thinking_type: None,
            verified_at: Some("2026-09-13T00:00:00Z".into()),
        };
        let mut model = control.store.as_ref().unwrap().model().unwrap();
        model.chatgpt.configured = true;
        model.chatgpt.status = model::ModelHealth::Ready;
        model.chatgpt.routes = vec![route.clone()];
        model.default_route = Some(route.selection());
        model.current_route = Some(route);
        control
            .store
            .as_mut()
            .unwrap()
            .save_model(model, "model.provider.changed")
            .unwrap();
        let current_risk = command(
            &mut control,
            "risk.get_policy",
            json!({"workspaceId":workspace_id}),
        );
        let ready = command(
            &mut control,
            "onboarding.set_step",
            json!({"workspaceId":workspace_id,"expectedStateVersion":current_risk["data"]["stateVersion"],"step":5}),
        );
        assert_eq!(ready["ok"], true, "{ready}");
        let completed = command(
            &mut control,
            "onboarding.complete",
            json!({"workspaceId":workspace_id,"expectedStateVersion":ready["data"]["stateVersion"]}),
        );
        assert_eq!(completed["data"]["onboardingCompleted"], true);
        drop(control);
        let mut reopened = ControlPlane::new(path);
        let reopened_workspace = command(&mut reopened, "workspace.open", json!({}));
        assert_eq!(reopened_workspace["ok"], true);
        let reopened_risk = command(
            &mut reopened,
            "risk.get_policy",
            json!({"workspaceId":workspace_id}),
        );
        assert_eq!(reopened_risk["data"]["onboardingCompleted"], false);
        assert_eq!(reopened_risk["data"]["onboardingStep"], 3);
    }
}
