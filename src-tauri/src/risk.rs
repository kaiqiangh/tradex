use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

pub const DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS: u64 = 3;
pub const DEFAULT_LIVE_INACTIVITY_TIMEOUT_MINUTES: u64 = 20;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskPolicy {
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_order_notional: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_single_instrument_exposure_percent: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_daily_traded_notional: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_daily_realized_loss: Option<String>,
    #[schemars(required, range(min = 1, max = 86_400_u64))]
    pub stale_quote_threshold_seconds: u64,
    #[schemars(required)]
    pub market_orders_enabled: bool,
    #[schemars(required, range(min = 1, max = 1_440_u64))]
    pub live_inactivity_timeout_minutes: u64,
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            max_order_notional: None,
            max_single_instrument_exposure_percent: None,
            max_daily_traded_notional: None,
            max_daily_realized_loss: None,
            stale_quote_threshold_seconds: DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS,
            market_orders_enabled: false,
            live_inactivity_timeout_minutes: DEFAULT_LIVE_INACTIVITY_TIMEOUT_MINUTES,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskPolicyInput {
    #[schemars(length(max = 32))]
    pub max_order_notional: RequiredNullableDecimal,
    #[schemars(length(max = 32))]
    pub max_single_instrument_exposure_percent: RequiredNullableDecimal,
    #[schemars(length(max = 32))]
    pub max_daily_traded_notional: RequiredNullableDecimal,
    #[schemars(length(max = 32))]
    pub max_daily_realized_loss: RequiredNullableDecimal,
    #[schemars(range(min = 1, max = 86_400_u64))]
    pub stale_quote_threshold_seconds: u64,
    pub market_orders_enabled: bool,
    #[schemars(range(min = 1, max = 1_440_u64))]
    pub live_inactivity_timeout_minutes: u64,
}

impl From<RiskPolicyInput> for RiskPolicy {
    fn from(input: RiskPolicyInput) -> Self {
        Self {
            max_order_notional: input.max_order_notional.0,
            max_single_instrument_exposure_percent: input.max_single_instrument_exposure_percent.0,
            max_daily_traded_notional: input.max_daily_traded_notional.0,
            max_daily_realized_loss: input.max_daily_realized_loss.0,
            stale_quote_threshold_seconds: input.stale_quote_threshold_seconds,
            market_orders_enabled: input.market_orders_enabled,
            live_inactivity_timeout_minutes: input.live_inactivity_timeout_minutes,
        }
    }
}

pub struct RequiredNullableDecimal(pub Option<String>);

impl<'de> Deserialize<'de> for RequiredNullableDecimal {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<String>::deserialize(deserializer).map(Self)
    }
}

impl JsonSchema for RequiredNullableDecimal {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("RequiredNullableDecimal")
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Option<String>>::json_schema(generator)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HardSafetyRule {
    #[schemars(length(min = 1, max = 64))]
    pub id: String,
    #[schemars(length(min = 1, max = 256))]
    pub description: String,
}

fn hard_safety_rules() -> Vec<HardSafetyRule> {
    [
        (
            "LIVE_DISARMED_BY_DEFAULT",
            "Live accounts remain DISARMED until the later financial authority gates exist.",
        ),
        (
            "APPROVAL_REQUIRED",
            "Every future Live transaction requires a separate TradeX financial approval.",
        ),
        (
            "STALE_DATA_BLOCKS_LIVE",
            "Stale, missing or untrusted market data blocks Live authority.",
        ),
        (
            "AGENT_CANNOT_MODIFY_POLICY",
            "Agent turns cannot change this risk policy.",
        ),
    ]
    .into_iter()
    .map(|(id, description)| HardSafetyRule {
        id: id.into(),
        description: description.into(),
    })
    .collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskPolicyState {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(range(min = 1))]
    pub policy_version: u64,
    pub configured: bool,
    #[schemars(range(min = 1, max = 5))]
    pub onboarding_step: u8,
    pub onboarding_completed: bool,
    pub policy: RiskPolicy,
    pub hard_rules: Vec<HardSafetyRule>,
    pub updated_at: String,
}

impl RiskPolicyState {
    pub fn new(workspace_id: String) -> Self {
        Self {
            workspace_id,
            state_version: String::new(),
            policy_version: 1,
            configured: false,
            onboarding_step: 1,
            onboarding_completed: false,
            policy: RiskPolicy::default(),
            hard_rules: hard_safety_rules(),
            updated_at: String::new(),
        }
    }

    pub fn mark_editing(&mut self) {
        self.onboarding_completed = false;
        self.onboarding_step = self.onboarding_step.min(4);
    }

    pub fn validate_policy(&mut self) -> crate::protocol::Result<()> {
        self.policy.max_order_notional =
            normalize_decimal(self.policy.max_order_notional.take(), 15, 8)?;
        self.policy.max_single_instrument_exposure_percent = normalize_decimal(
            self.policy.max_single_instrument_exposure_percent.take(),
            3,
            4,
        )?;
        self.policy.max_daily_traded_notional =
            normalize_decimal(self.policy.max_daily_traded_notional.take(), 15, 8)?;
        self.policy.max_daily_realized_loss =
            normalize_decimal(self.policy.max_daily_realized_loss.take(), 15, 8)?;
        if let Some(value) = &self.policy.max_single_instrument_exposure_percent
            && !decimal_at_most(value, "100")
        {
            return Err(crate::protocol::TradeXError::new("RISK_POLICY_INVALID"));
        }
        if !(1..=86_400).contains(&self.policy.stale_quote_threshold_seconds)
            || !(1..=1_440).contains(&self.policy.live_inactivity_timeout_minutes)
        {
            return Err(crate::protocol::TradeXError::new("RISK_POLICY_INVALID"));
        }
        self.configured = true;
        Ok(())
    }

    pub fn ready_for_completion(
        &self,
        model: &crate::model::ModelState,
        gateway: &crate::gateway::GatewayState,
    ) -> bool {
        self.configured
            && gateway.status == crate::gateway::GatewayStatus::Running
            && model.thread_plan().is_ok()
    }

    pub fn reopen_after_model_reset(&mut self) {
        if self.onboarding_completed {
            self.onboarding_completed = false;
            self.onboarding_step = 3;
        }
    }

    pub fn validate_persisted(&self, workspace_id: &str) -> crate::protocol::Result<()> {
        if self.workspace_id != workspace_id
            || self.state_version.is_empty()
            || self.updated_at.is_empty()
            || self.policy_version == 0
            || !(1..=5).contains(&self.onboarding_step)
            || (self.onboarding_step == 5 && !self.configured)
            || (self.onboarding_completed && self.onboarding_step != 5)
            || self.hard_rules != hard_safety_rules()
        {
            return Err(crate::protocol::TradeXError::new(
                "WORKSPACE_INTEGRITY_FAILED",
            ));
        }
        let mut checked = self.clone();
        checked
            .validate_policy()
            .map_err(|_| crate::protocol::TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        if checked.policy != self.policy
            || (!self.configured && self.policy != RiskPolicy::default())
        {
            return Err(crate::protocol::TradeXError::new(
                "WORKSPACE_INTEGRITY_FAILED",
            ));
        }
        Ok(())
    }

    pub fn from_persisted_json(data: &str, workspace_id: &str) -> crate::protocol::Result<Self> {
        let value: Value = serde_json::from_str(data)
            .map_err(|_| crate::protocol::TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        let policy = value
            .get("policy")
            .and_then(Value::as_object)
            .ok_or_else(|| crate::protocol::TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        for field in [
            "maxOrderNotional",
            "maxSingleInstrumentExposurePercent",
            "maxDailyTradedNotional",
            "maxDailyRealizedLoss",
            "staleQuoteThresholdSeconds",
            "marketOrdersEnabled",
            "liveInactivityTimeoutMinutes",
        ] {
            if !policy.contains_key(field) {
                return Err(crate::protocol::TradeXError::new(
                    "WORKSPACE_INTEGRITY_FAILED",
                ));
            }
        }
        let state: Self = serde_json::from_value(value)
            .map_err(|_| crate::protocol::TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        state.validate_persisted(workspace_id)?;
        Ok(state)
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRiskPolicy {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    pub policy: RiskPolicyInput,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetOnboardingStep {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    #[schemars(range(min = 1, max = 5))]
    pub step: u8,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteOnboarding {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

fn normalize_decimal(
    value: Option<String>,
    max_integer_digits: usize,
    max_fraction_digits: usize,
) -> crate::protocol::Result<Option<String>> {
    let Some(value) = value else { return Ok(None) };
    let value = value.trim();
    if value.is_empty() {
        return Err(crate::protocol::TradeXError::new("RISK_POLICY_INVALID"));
    }
    let (integer, fraction) = value.split_once('.').unwrap_or((value, ""));
    if integer.is_empty()
        || integer.len() > max_integer_digits
        || fraction.len() > max_fraction_digits
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || integer.trim_start_matches('0').is_empty() && fraction.trim_end_matches('0').is_empty()
    {
        return Err(crate::protocol::TradeXError::new("RISK_POLICY_INVALID"));
    }
    Ok(Some(value.to_owned()))
}

fn decimal_at_most(value: &str, maximum: &str) -> bool {
    let left = value.split_once('.').unwrap_or((value, ""));
    let right = maximum.split_once('.').unwrap_or((maximum, ""));
    let left_integer = left.0.trim_start_matches('0');
    let right_integer = right.0.trim_start_matches('0');
    left_integer.len() < right_integer.len()
        || (left_integer.len() == right_integer.len()
            && (left_integer < right_integer
                || (left_integer == right_integer
                    && left.1.trim_end_matches('0') <= right.1.trim_end_matches('0'))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_leave_money_unset_and_keep_safe_non_money_values() {
        let state = RiskPolicyState::new("workspace".into());
        assert_eq!(state.policy.max_order_notional, None);
        assert_eq!(state.policy.max_daily_realized_loss, None);
        assert_eq!(state.policy.stale_quote_threshold_seconds, 3);
        assert!(!state.policy.market_orders_enabled);
        assert_eq!(state.policy.live_inactivity_timeout_minutes, 20);
    }

    #[test]
    fn decimal_and_boundary_validation_is_exact() {
        let mut state = RiskPolicyState::new("workspace".into());
        state.policy.max_order_notional = Some("1000.125".into());
        state.policy.max_single_instrument_exposure_percent = Some("10.00".into());
        state.validate_policy().unwrap();
        assert_eq!(state.policy.max_order_notional.as_deref(), Some("1000.125"));
        state.policy.max_single_instrument_exposure_percent = Some("100.01".into());
        assert_eq!(
            state.validate_policy().unwrap_err().code,
            "RISK_POLICY_INVALID"
        );
        state.policy.max_single_instrument_exposure_percent = Some("0".into());
        assert_eq!(
            state.validate_policy().unwrap_err().code,
            "RISK_POLICY_INVALID"
        );
    }
}
