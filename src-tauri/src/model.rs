use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelProvider {
    Chatgpt,
    Deepseek,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelHealth {
    NotConfigured,
    Unverified,
    Verifying,
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingType {
    Disabled,
    Enabled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelAttemptOutcome {
    Verified,
    Configured,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelAttemptKind {
    #[default]
    Setup,
    Thread,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSelection {
    pub provider: ModelProvider,
    #[schemars(length(min = 1, max = 128))]
    pub model_id: String,
    pub thinking_type: Option<ThinkingType>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelRoute {
    pub provider: ModelProvider,
    #[schemars(length(min = 1, max = 128))]
    pub model_id: String,
    pub thinking_type: Option<ThinkingType>,
    pub verified_at: Option<String>,
}

impl ModelRoute {
    pub fn selection(&self) -> ModelSelection {
        ModelSelection {
            provider: self.provider.clone(),
            model_id: self.model_id.clone(),
            thinking_type: self.thinking_type.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelProviderState {
    pub provider: ModelProvider,
    pub configured: bool,
    pub status: ModelHealth,
    #[schemars(length(max = 100))]
    pub routes: Vec<ModelRoute>,
    pub last_verified_at: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelQuota {
    pub window: Option<String>,
    pub reset_at: Option<String>,
    #[schemars(range(max = 86_400_u64))]
    pub retry_after_seconds: Option<u64>,
    #[schemars(range(max = 1_000_000_000_u64))]
    pub remaining: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelAttempt {
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: String,
    pub provider: ModelProvider,
    pub model_id: Option<String>,
    pub thinking_type: Option<ThinkingType>,
    pub started_at: String,
    pub ended_at: String,
    #[serde(default)]
    pub kind: ModelAttemptKind,
    pub outcome: ModelAttemptOutcome,
    pub error_category: Option<String>,
    pub quota: Option<ModelQuota>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelState {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    pub chatgpt: ModelProviderState,
    pub deepseek: ModelProviderState,
    pub current_route: Option<ModelRoute>,
    #[serde(default)]
    pub default_route: Option<ModelSelection>,
    #[serde(default)]
    pub automatic_fallback: bool,
    #[serde(default = "default_fallback_policy_version")]
    #[schemars(range(min = 1))]
    pub fallback_policy_version: u64,
    #[schemars(length(max = 100))]
    pub attempts: Vec<ModelAttempt>,
    pub updated_at: String,
}

fn default_fallback_policy_version() -> u64 {
    1
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChatgptLoginAction {
    Login,
    Relogin,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChatgptLogin {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    pub action: ChatgptLoginAction,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigureDeepseek {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerifyRoute {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    pub provider: ModelProvider,
    #[schemars(length(min = 1, max = 128))]
    pub model_id: String,
    pub thinking_type: Option<ThinkingType>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetDefaultModel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    pub provider: ModelProvider,
    #[schemars(length(min = 1, max = 128))]
    pub model_id: String,
    pub thinking_type: Option<ThinkingType>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetFallbackPolicy {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    pub automatic_fallback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelAction {
    LoginChatgpt(ChatgptLoginAction),
    ConfigureDeepseek,
    VerifyRoute {
        provider: ModelProvider,
        model_id: String,
        thinking_type: Option<ThinkingType>,
    },
}

pub struct ModelJob {
    pub(crate) request_id: String,
    pub(crate) session: String,
    pub(crate) action: ModelAction,
    pub(crate) previous: ModelState,
    pub(crate) state: ModelState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRequestPlan {
    pub primary: ModelRoute,
    pub fallback: Option<ModelRoute>,
    pub fallback_policy_version: u64,
}

pub struct ModelOutcome {
    pub(crate) attempt: ModelAttempt,
    pub(crate) routes: Vec<ModelRoute>,
    pub(crate) configured: Option<bool>,
    pub(crate) error: Option<crate::protocol::TradeXError>,
}

#[cfg(target_os = "macos")]
fn discovery_error(provider: &ModelProvider, code: &'static str) -> crate::protocol::TradeXError {
    if *provider == ModelProvider::Chatgpt && code == "GATEWAY_UNAUTHORIZED" {
        crate::protocol::TradeXError::new("MODEL_OAUTH_EXPIRED")
    } else {
        crate::protocol::TradeXError::new(code)
    }
}

pub fn allowed_route(
    provider: &ModelProvider,
    model_id: &str,
    thinking_type: Option<&ThinkingType>,
) -> bool {
    match provider {
        ModelProvider::Chatgpt => {
            thinking_type.is_none()
                && model_id
                    .strip_prefix("gpt-5.6")
                    .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('-'))
                && model_id.len() <= 128
                && !model_id.chars().any(char::is_control)
        }
        ModelProvider::Deepseek => model_id == "deepseek-v4-flash" && thinking_type.is_some(),
    }
}

impl ModelProviderState {
    fn new(provider: ModelProvider) -> Self {
        Self {
            provider,
            configured: false,
            status: ModelHealth::NotConfigured,
            routes: vec![],
            last_verified_at: None,
            error_code: None,
        }
    }

    fn reset_for_session(&mut self) {
        self.routes.clear();
        self.last_verified_at = None;
        self.error_code = None;
        self.status = if self.configured {
            ModelHealth::Unverified
        } else {
            ModelHealth::NotConfigured
        };
    }
}

impl ModelState {
    pub fn new(workspace_id: String) -> Self {
        Self {
            workspace_id,
            state_version: String::new(),
            chatgpt: ModelProviderState::new(ModelProvider::Chatgpt),
            deepseek: ModelProviderState::new(ModelProvider::Deepseek),
            current_route: None,
            default_route: None,
            automatic_fallback: false,
            fallback_policy_version: default_fallback_policy_version(),
            attempts: vec![],
            updated_at: String::new(),
        }
    }

    pub fn reset_for_session(&mut self) {
        self.chatgpt.reset_for_session();
        self.deepseek.reset_for_session();
        self.current_route = None;
    }

    pub fn provider_mut(&mut self, provider: &ModelProvider) -> &mut ModelProviderState {
        match provider {
            ModelProvider::Chatgpt => &mut self.chatgpt,
            ModelProvider::Deepseek => &mut self.deepseek,
        }
    }

    pub fn provider(&self, provider: &ModelProvider) -> &ModelProviderState {
        match provider {
            ModelProvider::Chatgpt => &self.chatgpt,
            ModelProvider::Deepseek => &self.deepseek,
        }
    }

    pub fn append_attempt(&mut self, attempt: ModelAttempt) {
        self.attempts.push(attempt);
        if self.attempts.len() > 100 {
            self.attempts.remove(0);
        }
    }

    pub fn verified_route(&self, selection: &ModelSelection) -> Option<ModelRoute> {
        let provider = self.provider(&selection.provider);
        (provider.status == ModelHealth::Ready)
            .then(|| {
                provider
                    .routes
                    .iter()
                    .find(|route| {
                        allowed_route(
                            &route.provider,
                            &route.model_id,
                            route.thinking_type.as_ref(),
                        ) && route.model_id == selection.model_id
                            && route.provider == selection.provider
                            && route.thinking_type == selection.thinking_type
                            && route.verified_at.is_some()
                    })
                    .cloned()
            })
            .flatten()
    }

    pub fn thread_plan(&self) -> std::result::Result<ModelRequestPlan, &'static str> {
        let selection = self.default_route.as_ref().ok_or("MODEL_DEFAULT_MISSING")?;
        let primary = self.verified_route(selection).ok_or("MODEL_UNAVAILABLE")?;
        Ok(ModelRequestPlan {
            primary,
            fallback: None,
            fallback_policy_version: self.fallback_policy_version,
        })
    }

    pub fn fallback_plan(
        &self,
        primary: &ModelRoute,
        error_category: &str,
    ) -> Option<ModelRequestPlan> {
        if !self.automatic_fallback
            || primary.provider != ModelProvider::Chatgpt
            || !allowed_route(
                &primary.provider,
                &primary.model_id,
                primary.thinking_type.as_ref(),
            )
            || primary.verified_at.is_none()
            || !matches!(
                error_category,
                "MODEL_UNAVAILABLE" | "OAUTH_EXPIRED" | "QUOTA_EXCEEDED"
            )
        {
            return None;
        }
        Some(ModelRequestPlan {
            primary: primary.clone(),
            fallback: Some(self.verified_deepseek_route()?),
            fallback_policy_version: self.fallback_policy_version,
        })
    }

    pub fn verified_deepseek_route(&self) -> Option<ModelRoute> {
        if self.deepseek.status != ModelHealth::Ready {
            return None;
        }
        self.deepseek
            .routes
            .iter()
            .filter(|route| {
                allowed_route(
                    &route.provider,
                    &route.model_id,
                    route.thinking_type.as_ref(),
                ) && route.provider == ModelProvider::Deepseek
                    && route.model_id == "deepseek-v4-flash"
                    && route.verified_at.is_some()
            })
            .min_by_key(|route| {
                if route.thinking_type == Some(ThinkingType::Disabled) {
                    0
                } else {
                    1
                }
            })
            .cloned()
    }

    pub fn fallback_for(&self, primary: &ModelRoute, error_category: &str) -> Option<ModelRoute> {
        self.fallback_plan(primary, error_category)
            .and_then(|plan| plan.fallback)
    }

    pub fn retry_blocked(&self, provider: &ModelProvider, now: time::OffsetDateTime) -> bool {
        self.attempts
            .iter()
            .rev()
            .find(|attempt| {
                &attempt.provider == provider
                    && attempt.error_category.as_deref() == Some("QUOTA_EXCEEDED")
            })
            .and_then(|attempt| {
                let seconds = attempt.quota.as_ref()?.retry_after_seconds?;
                let ended = time::OffsetDateTime::parse(
                    &attempt.ended_at,
                    &time::format_description::well_known::Rfc3339,
                )
                .ok()?;
                Some(ended + time::Duration::seconds(seconds as i64))
            })
            .is_some_and(|until| now < until)
    }
}

#[cfg(target_os = "macos")]
pub fn run_job(
    job: &ModelJob,
    host: &mut crate::gateway_process::GatewayHost,
    vault: &impl crate::model_credentials::ModelVault,
    capture: impl FnOnce() -> crate::protocol::Result<crate::model_credentials::ModelKey>,
    current: impl Fn() -> bool,
) -> ModelOutcome {
    let _ = host.take_last_quota();
    let started_at = crate::storage::timestamp().unwrap_or_default();
    let provider = match &job.action {
        ModelAction::LoginChatgpt(_) => ModelProvider::Chatgpt,
        ModelAction::ConfigureDeepseek
        | ModelAction::VerifyRoute {
            provider: ModelProvider::Deepseek,
            ..
        } => ModelProvider::Deepseek,
        ModelAction::VerifyRoute { provider, .. } => provider.clone(),
    };
    let (model_id, thinking_type) = match &job.action {
        ModelAction::VerifyRoute {
            model_id,
            thinking_type,
            ..
        } => (Some(model_id.clone()), thinking_type.clone()),
        _ => (None, None),
    };
    let mut error = None;
    let mut configured = None;
    let mut routes = vec![];
    let mut outcome = ModelAttemptOutcome::Failed;
    let action_result = (|| -> crate::protocol::Result<()> {
        if !current() {
            return Err(crate::protocol::TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        match &job.action {
            ModelAction::ConfigureDeepseek => {
                let key = capture()?;
                if !current() {
                    return Err(crate::protocol::TradeXError::new("STATE_VERSION_CONFLICT"));
                }
                vault.put_deepseek(&job.state.workspace_id, &key)?;
                host.set_deepseek_key(&job.state.workspace_id, Some(key.as_str()));
                configured = Some(true);
                outcome = ModelAttemptOutcome::Configured;
                Ok(())
            }
            ModelAction::LoginChatgpt(_) => {
                host.login_chatgpt(&current)
                    .map_err(crate::protocol::TradeXError::new)?;
                let discovered = host
                    .discover_models()
                    .map_err(|code| discovery_error(&ModelProvider::Chatgpt, code))?;
                routes = discovered
                    .into_iter()
                    .filter(|id| allowed_route(&ModelProvider::Chatgpt, id, None))
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .take(100)
                    .map(|model_id| ModelRoute {
                        provider: ModelProvider::Chatgpt,
                        model_id,
                        thinking_type: None,
                        verified_at: None,
                    })
                    .collect();
                if routes.is_empty() {
                    return Err(crate::protocol::TradeXError::new("MODEL_UNAVAILABLE"));
                }
                configured = Some(true);
                outcome = ModelAttemptOutcome::Configured;
                Ok(())
            }
            ModelAction::VerifyRoute {
                model_id,
                thinking_type,
                provider,
            } => {
                if *provider == ModelProvider::Deepseek {
                    let key = vault.get_deepseek(&job.state.workspace_id)?;
                    host.set_deepseek_key(&job.state.workspace_id, Some(key.as_str()));
                }
                let discovered = host
                    .discover_models()
                    .map_err(|code| discovery_error(provider, code))?;
                if !discovered.iter().any(|id| id == model_id)
                    || !allowed_route(provider, model_id, thinking_type.as_ref())
                {
                    return Err(crate::protocol::TradeXError::new("MODEL_UNAVAILABLE"));
                }
                let mode = thinking_type.as_ref().map(|mode| match mode {
                    ThinkingType::Disabled => "disabled",
                    ThinkingType::Enabled => "enabled",
                });
                host.test_inference(model_id, mode)
                    .map_err(crate::protocol::TradeXError::new)?;
                routes = vec![ModelRoute {
                    provider: provider.clone(),
                    model_id: model_id.clone(),
                    thinking_type: thinking_type.clone(),
                    verified_at: None,
                }];
                configured = Some(true);
                outcome = ModelAttemptOutcome::Verified;
                Ok(())
            }
        }
    })();
    if let Err(failure) = action_result {
        error = Some(failure);
        outcome = if error
            .as_ref()
            .is_some_and(|failure| failure.code == "MODEL_ENTRY_CANCELLED")
        {
            ModelAttemptOutcome::Cancelled
        } else {
            ModelAttemptOutcome::Failed
        };
    }
    let ended_at = crate::storage::timestamp().unwrap_or_else(|_| started_at.clone());
    let quota = host.take_last_quota();
    ModelOutcome {
        attempt: ModelAttempt {
            attempt_id: uuid::Uuid::new_v4().to_string(),
            provider,
            model_id,
            thinking_type,
            started_at,
            ended_at,
            kind: ModelAttemptKind::Setup,
            outcome,
            error_category: error.as_ref().and_then(|failure| {
                (failure.code != "MODEL_ENTRY_CANCELLED").then(|| failure.category.clone())
            }),
            quota,
        },
        routes,
        configured,
        error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_requires_exact_provider_contracts() {
        assert!(allowed_route(&ModelProvider::Chatgpt, "gpt-5.6-sol", None));
        assert!(allowed_route(&ModelProvider::Chatgpt, "gpt-5.6-luna", None));
        assert!(!allowed_route(
            &ModelProvider::Chatgpt,
            "gpt-5.6-sol",
            Some(&ThinkingType::Enabled)
        ));
        assert!(!allowed_route(&ModelProvider::Chatgpt, "gpt-4o", None));
        assert!(!allowed_route(&ModelProvider::Chatgpt, "gpt-5.6evil", None));
        assert!(!allowed_route(
            &ModelProvider::Deepseek,
            "deepseek-chat",
            Some(&ThinkingType::Disabled)
        ));
        assert!(!allowed_route(
            &ModelProvider::Deepseek,
            "deepseek-v4-flash",
            None
        ));
        assert!(allowed_route(
            &ModelProvider::Deepseek,
            "deepseek-v4-flash",
            Some(&ThinkingType::Disabled)
        ));
        assert!(allowed_route(
            &ModelProvider::Deepseek,
            "deepseek-v4-flash",
            Some(&ThinkingType::Enabled)
        ));
    }

    #[test]
    fn model_state_resets_process_observations_and_bounds_attempt_history() {
        let mut state = ModelState::new("workspace-one".into());
        state.chatgpt.configured = true;
        state.chatgpt.status = ModelHealth::Ready;
        state.current_route = Some(ModelRoute {
            provider: ModelProvider::Chatgpt,
            model_id: "gpt-5.6-sol".into(),
            thinking_type: None,
            verified_at: Some("2026-09-12T00:00:00Z".into()),
        });
        state.default_route = state.current_route.as_ref().map(ModelRoute::selection);
        for index in 0..101 {
            state.append_attempt(ModelAttempt {
                attempt_id: index.to_string(),
                provider: ModelProvider::Chatgpt,
                model_id: Some("gpt-5.6-sol".into()),
                thinking_type: None,
                started_at: "2026-09-12T00:00:00Z".into(),
                ended_at: "2026-09-12T00:00:01Z".into(),
                kind: ModelAttemptKind::Setup,
                outcome: ModelAttemptOutcome::Failed,
                error_category: Some("MODEL_UNAVAILABLE".into()),
                quota: None,
            });
        }
        assert_eq!(state.attempts.len(), 100);
        assert_eq!(state.attempts.first().unwrap().attempt_id, "1");
        state.reset_for_session();
        assert_eq!(state.chatgpt.status, ModelHealth::Unverified);
        assert!(state.chatgpt.routes.is_empty());
        assert!(state.current_route.is_none());
        assert_eq!(
            state.default_route.as_ref().unwrap().model_id,
            "gpt-5.6-sol"
        );
        assert_eq!(state.attempts.len(), 100);
    }

    #[test]
    fn verified_route_requires_matching_provider_identity() {
        let mut state = ModelState::new("workspace-one".into());
        state.chatgpt.status = ModelHealth::Ready;
        state.chatgpt.routes = vec![ModelRoute {
            provider: ModelProvider::Deepseek,
            model_id: "deepseek-v4-flash".into(),
            thinking_type: Some(ThinkingType::Disabled),
            verified_at: Some("2026-09-13T00:00:00Z".into()),
        }];
        state.default_route = Some(ModelSelection {
            provider: ModelProvider::Chatgpt,
            model_id: "deepseek-v4-flash".into(),
            thinking_type: Some(ThinkingType::Disabled),
        });
        assert_eq!(state.thread_plan(), Err("MODEL_UNAVAILABLE"));
    }

    #[test]
    fn thread_plan_requires_default_and_only_offers_opt_in_deepseek_fallback() {
        let mut state = ModelState::new("workspace-one".into());
        let chatgpt = ModelRoute {
            provider: ModelProvider::Chatgpt,
            model_id: "gpt-5.6-sol".into(),
            thinking_type: None,
            verified_at: Some("2026-09-13T00:00:00Z".into()),
        };
        let deepseek = ModelRoute {
            provider: ModelProvider::Deepseek,
            model_id: "deepseek-v4-flash".into(),
            thinking_type: Some(ThinkingType::Disabled),
            verified_at: Some("2026-09-13T00:00:00Z".into()),
        };
        assert_eq!(state.thread_plan(), Err("MODEL_DEFAULT_MISSING"));
        state.chatgpt.status = ModelHealth::Ready;
        state.chatgpt.routes = vec![chatgpt.clone()];
        state.deepseek.status = ModelHealth::Ready;
        state.deepseek.routes = vec![deepseek.clone()];
        state.default_route = Some(chatgpt.selection());
        assert!(state.thread_plan().unwrap().fallback.is_none());
        state.automatic_fallback = true;
        let plan = state.thread_plan().unwrap();
        assert_eq!(plan.primary, chatgpt);
        assert!(plan.fallback.is_none());
        let fallback_plan = state
            .fallback_plan(&plan.primary, "QUOTA_EXCEEDED")
            .unwrap();
        assert_eq!(fallback_plan.primary, plan.primary);
        assert_eq!(fallback_plan.fallback, Some(deepseek.clone()));
        assert_eq!(
            state.fallback_for(&plan.primary, "QUOTA_EXCEEDED"),
            Some(deepseek.clone())
        );
        assert!(
            state
                .fallback_for(&plan.primary, "MODEL_TEST_INFERENCE_FAILED")
                .is_none()
        );
        state.default_route = Some(deepseek.selection());
        assert_eq!(
            state.fallback_for(&plan.primary, "QUOTA_EXCEEDED"),
            Some(deepseek.clone())
        );
        state.deepseek.status = ModelHealth::Unverified;
        assert!(
            state
                .fallback_for(&plan.primary, "QUOTA_EXCEEDED")
                .is_none()
        );
        state.deepseek.status = ModelHealth::Ready;
        state.default_route = Some(ModelSelection {
            provider: ModelProvider::Deepseek,
            model_id: "deepseek-v4-flash".into(),
            thinking_type: Some(ThinkingType::Disabled),
        });
        assert!(state.thread_plan().unwrap().fallback.is_none());
    }

    #[test]
    fn retry_cooldown_is_bounded_by_the_latest_quota_attempt() {
        let mut state = ModelState::new("workspace-one".into());
        let now = time::OffsetDateTime::parse(
            "2026-09-13T00:00:30Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        state.append_attempt(ModelAttempt {
            attempt_id: "quota-1".into(),
            provider: ModelProvider::Chatgpt,
            model_id: Some("gpt-5.6-sol".into()),
            thinking_type: None,
            started_at: "2026-09-13T00:00:00Z".into(),
            ended_at: "2026-09-13T00:00:00Z".into(),
            kind: ModelAttemptKind::Thread,
            outcome: ModelAttemptOutcome::Failed,
            error_category: Some("QUOTA_EXCEEDED".into()),
            quota: Some(ModelQuota {
                window: Some("retry-after:60s".into()),
                reset_at: None,
                retry_after_seconds: Some(60),
                remaining: None,
            }),
        });
        assert!(state.retry_blocked(&ModelProvider::Chatgpt, now));
        assert!(!state.retry_blocked(&ModelProvider::Chatgpt, now + time::Duration::seconds(31)));
        assert!(!state.retry_blocked(&ModelProvider::Deepseek, now));
    }
}
