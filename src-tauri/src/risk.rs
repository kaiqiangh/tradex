use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::HashSet;

pub const DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS: u64 = 3;
pub const DEFAULT_LIVE_INACTIVITY_TIMEOUT_MINUTES: u64 = 20;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskPolicyEnvironment {
    LocalPaper,
    Paper,
    Demo,
    Testnet,
    Live,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskAssetClassLimit {
    pub asset_class: crate::protocol::AssetClass,
    #[schemars(length(max = 32))]
    pub max_exposure_percent: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskPolicy {
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_order_notional: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_order_quantity: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_position_size: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_single_instrument_exposure_percent: Option<String>,
    #[schemars(required, length(max = 2))]
    pub max_asset_class_exposure_percent: Vec<RiskAssetClassLimit>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_daily_traded_notional: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_daily_realized_loss: Option<String>,
    #[schemars(with = "RequiredNullableU64", required)]
    pub max_open_orders: Option<u64>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_reserved_capital: Option<String>,
    #[schemars(required, length(max = 256))]
    pub allowed_instrument_ids: Vec<String>,
    #[schemars(required, length(max = 256))]
    pub blocked_instrument_ids: Vec<String>,
    #[schemars(required, length(max = 256))]
    pub allowed_venues: Vec<String>,
    #[schemars(required, length(max = 256))]
    pub blocked_venues: Vec<String>,
    #[schemars(required, length(max = 256))]
    pub allowed_account_ids: Vec<String>,
    #[schemars(required, length(max = 256))]
    pub blocked_account_ids: Vec<String>,
    #[schemars(required, length(max = 5))]
    pub allowed_environments: Vec<RiskPolicyEnvironment>,
    #[schemars(required, range(min = 1, max = 86_400_u64))]
    pub stale_quote_threshold_seconds: u64,
    #[schemars(required)]
    pub market_orders_enabled: bool,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_market_order_slippage_percent: Option<String>,
    #[schemars(with = "RequiredNullableDecimal", required, length(max = 32))]
    pub max_price_deviation_percent: Option<String>,
    #[schemars(required, range(min = 1, max = 1_440_u64))]
    pub live_inactivity_timeout_minutes: u64,
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            max_order_notional: None,
            max_order_quantity: None,
            max_position_size: None,
            max_single_instrument_exposure_percent: None,
            max_asset_class_exposure_percent: Vec::new(),
            max_daily_traded_notional: None,
            max_daily_realized_loss: None,
            max_open_orders: None,
            max_reserved_capital: None,
            allowed_instrument_ids: Vec::new(),
            blocked_instrument_ids: Vec::new(),
            allowed_venues: Vec::new(),
            blocked_venues: Vec::new(),
            allowed_account_ids: Vec::new(),
            blocked_account_ids: Vec::new(),
            allowed_environments: Vec::new(),
            stale_quote_threshold_seconds: DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS,
            market_orders_enabled: false,
            max_market_order_slippage_percent: None,
            max_price_deviation_percent: None,
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
    pub max_order_quantity: RequiredNullableDecimal,
    #[schemars(length(max = 32))]
    pub max_position_size: RequiredNullableDecimal,
    #[schemars(length(max = 32))]
    pub max_single_instrument_exposure_percent: RequiredNullableDecimal,
    #[schemars(length(max = 2))]
    pub max_asset_class_exposure_percent: Vec<RiskAssetClassLimit>,
    #[schemars(length(max = 32))]
    pub max_daily_traded_notional: RequiredNullableDecimal,
    #[schemars(length(max = 32))]
    pub max_daily_realized_loss: RequiredNullableDecimal,
    pub max_open_orders: RequiredNullableU64,
    #[schemars(length(max = 32))]
    pub max_reserved_capital: RequiredNullableDecimal,
    #[schemars(length(max = 256))]
    pub allowed_instrument_ids: Vec<String>,
    #[schemars(length(max = 256))]
    pub blocked_instrument_ids: Vec<String>,
    #[schemars(length(max = 256))]
    pub allowed_venues: Vec<String>,
    #[schemars(length(max = 256))]
    pub blocked_venues: Vec<String>,
    #[schemars(length(max = 256))]
    pub allowed_account_ids: Vec<String>,
    #[schemars(length(max = 256))]
    pub blocked_account_ids: Vec<String>,
    #[schemars(length(max = 5))]
    pub allowed_environments: Vec<RiskPolicyEnvironment>,
    #[schemars(range(min = 1, max = 86_400_u64))]
    pub stale_quote_threshold_seconds: u64,
    pub market_orders_enabled: bool,
    #[schemars(length(max = 32))]
    pub max_market_order_slippage_percent: RequiredNullableDecimal,
    #[schemars(length(max = 32))]
    pub max_price_deviation_percent: RequiredNullableDecimal,
    #[schemars(range(min = 1, max = 1_440_u64))]
    pub live_inactivity_timeout_minutes: u64,
}

impl From<RiskPolicyInput> for RiskPolicy {
    fn from(input: RiskPolicyInput) -> Self {
        Self {
            max_order_notional: input.max_order_notional.0,
            max_order_quantity: input.max_order_quantity.0,
            max_position_size: input.max_position_size.0,
            max_single_instrument_exposure_percent: input.max_single_instrument_exposure_percent.0,
            max_asset_class_exposure_percent: input.max_asset_class_exposure_percent,
            max_daily_traded_notional: input.max_daily_traded_notional.0,
            max_daily_realized_loss: input.max_daily_realized_loss.0,
            max_open_orders: input.max_open_orders.0,
            max_reserved_capital: input.max_reserved_capital.0,
            allowed_instrument_ids: input.allowed_instrument_ids,
            blocked_instrument_ids: input.blocked_instrument_ids,
            allowed_venues: input.allowed_venues,
            blocked_venues: input.blocked_venues,
            allowed_account_ids: input.allowed_account_ids,
            blocked_account_ids: input.blocked_account_ids,
            allowed_environments: input.allowed_environments,
            stale_quote_threshold_seconds: input.stale_quote_threshold_seconds,
            market_orders_enabled: input.market_orders_enabled,
            max_market_order_slippage_percent: input.max_market_order_slippage_percent.0,
            max_price_deviation_percent: input.max_price_deviation_percent.0,
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

pub struct RequiredNullableU64(pub Option<u64>);

impl<'de> Deserialize<'de> for RequiredNullableU64 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<u64>::deserialize(deserializer).map(Self)
    }
}

impl JsonSchema for RequiredNullableU64 {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("RequiredNullableU64")
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Option<u64>>::json_schema(generator)
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
        self.policy.max_order_quantity =
            normalize_decimal(self.policy.max_order_quantity.take(), 18, 8)?;
        self.policy.max_position_size =
            normalize_decimal(self.policy.max_position_size.take(), 15, 8)?;
        self.policy.max_single_instrument_exposure_percent = normalize_decimal(
            self.policy.max_single_instrument_exposure_percent.take(),
            3,
            4,
        )?;
        for limit in &mut self.policy.max_asset_class_exposure_percent {
            limit.max_exposure_percent =
                normalize_decimal(Some(std::mem::take(&mut limit.max_exposure_percent)), 3, 4)?
                    .ok_or_else(|| crate::protocol::TradeXError::new("RISK_POLICY_INVALID"))?;
        }
        self.policy.max_daily_traded_notional =
            normalize_decimal(self.policy.max_daily_traded_notional.take(), 15, 8)?;
        self.policy.max_daily_realized_loss =
            normalize_decimal(self.policy.max_daily_realized_loss.take(), 15, 8)?;
        self.policy.max_reserved_capital =
            normalize_decimal(self.policy.max_reserved_capital.take(), 15, 8)?;
        self.policy.max_market_order_slippage_percent =
            normalize_decimal(self.policy.max_market_order_slippage_percent.take(), 18, 8)?;
        self.policy.max_price_deviation_percent =
            normalize_decimal(self.policy.max_price_deviation_percent.take(), 18, 8)?;
        if let Some(value) = &self.policy.max_single_instrument_exposure_percent
            && !decimal_at_most(value, "100")
        {
            return Err(crate::protocol::TradeXError::new("RISK_POLICY_INVALID"));
        }
        if self
            .policy
            .max_asset_class_exposure_percent
            .iter()
            .any(|limit| !decimal_at_most(&limit.max_exposure_percent, "100"))
        {
            return Err(crate::protocol::TradeXError::new("RISK_POLICY_INVALID"));
        }
        if self.policy.max_open_orders == Some(0)
            || !valid_instrument_ids(&self.policy.allowed_instrument_ids)
            || !valid_instrument_ids(&self.policy.blocked_instrument_ids)
            || !valid_identifier_list(&self.policy.allowed_venues, 32)
            || !valid_identifier_list(&self.policy.blocked_venues, 32)
            || !valid_identifier_list(&self.policy.allowed_account_ids, 128)
            || !valid_identifier_list(&self.policy.blocked_account_ids, 128)
            || duplicate_asset_class_limits(&self.policy.max_asset_class_exposure_percent)
            || duplicate_values(&self.policy.allowed_environments)
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

    pub fn validate_policy_for_save(&mut self) -> crate::protocol::Result<()> {
        self.validate_policy()?;
        if self.policy.market_orders_enabled
            && self.policy.max_market_order_slippage_percent.is_none()
        {
            return Err(crate::protocol::TradeXError::new("RISK_POLICY_INVALID"));
        }
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
        let mut value: Value = serde_json::from_str(data)
            .map_err(|_| crate::protocol::TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        let policy = value
            .get_mut("policy")
            .and_then(Value::as_object_mut)
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
        for (field, default) in [
            ("maxOrderQuantity", Value::Null),
            ("maxPositionSize", Value::Null),
            ("maxAssetClassExposurePercent", serde_json::json!([])),
            ("maxOpenOrders", Value::Null),
            ("maxReservedCapital", Value::Null),
            ("allowedInstrumentIds", serde_json::json!([])),
            ("blockedInstrumentIds", serde_json::json!([])),
            ("allowedVenues", serde_json::json!([])),
            ("blockedVenues", serde_json::json!([])),
            ("allowedAccountIds", serde_json::json!([])),
            ("blockedAccountIds", serde_json::json!([])),
            ("allowedEnvironments", serde_json::json!([])),
            ("maxMarketOrderSlippagePercent", Value::Null),
            ("maxPriceDeviationPercent", Value::Null),
        ] {
            policy.entry(field).or_insert(default);
        }
        let state: Self = serde_json::from_value(value)
            .map_err(|_| crate::protocol::TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        state.validate_persisted(workspace_id)?;
        Ok(state)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskDecisionStatus {
    Allowed,
    Rejected,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskCheckOutcome {
    Pass,
    Reject,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskCheckId {
    ProposalIdentity,
    PolicyConfigured,
    AccountBinding,
    AccountHealth,
    AllowedAccount,
    AllowedEnvironment,
    AllowedVenue,
    AllowedInstrument,
    BlockedAccount,
    BlockedVenue,
    BlockedInstrument,
    OrderNotional,
    OrderQuantity,
    PositionSize,
    SingleInstrumentExposure,
    AssetClassExposure,
    DailyTradedNotional,
    DailyRealizedLoss,
    OpenOrderCount,
    ReservedCapital,
    MarketOrder,
    MarketOrderSlippage,
    PriceDeviation,
    QuoteFreshness,
    MarketSession,
    InstrumentRules,
    LiveInactivity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskDecisionReasonCode {
    WithinLimit,
    LimitNotConfigured,
    LimitExceeded,
    PolicyUnconfigured,
    ProposalInvalidated,
    EvidenceMissing,
    EvidenceStale,
    EvidenceUntrusted,
    AccountUnavailable,
    AccountMismatch,
    AccountUnhealthy,
    IdentifierNotAllowed,
    IdentifierBlocked,
    EnvironmentNotAllowed,
    MarketOrderDisabled,
    ExecutionQuoteUnavailable,
    MarketClosed,
    MarketHalted,
    InstrumentRulesUnavailable,
    CounterUnavailable,
    ReservationUnavailable,
    CalendarUnavailable,
    ClockUncertain,
    NotApplicable,
    UnsupportedContext,
    InvalidProposalValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskDecisionInputKind {
    Policy,
    Account,
    Portfolio,
    Market,
    Time,
    Calendar,
    DailyCounters,
    Reservations,
    InstrumentRules,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskCheckResult {
    pub check_id: RiskCheckId,
    pub outcome: RiskCheckOutcome,
    pub reason_code: RiskDecisionReasonCode,
    #[schemars(length(min = 1, max = 512))]
    pub reason: String,
}

impl RiskDecisionStatus {
    fn from_checks(checks: &[RiskCheckResult]) -> Self {
        if checks
            .iter()
            .any(|check| check.outcome == RiskCheckOutcome::Reject)
        {
            Self::Rejected
        } else if checks
            .iter()
            .any(|check| check.outcome == RiskCheckOutcome::Unavailable)
        {
            Self::Unavailable
        } else {
            Self::Allowed
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskDecisionInputReference {
    pub kind: RiskDecisionInputKind,
    #[schemars(length(min = 1, max = 128))]
    pub reference_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub observed_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskDecision {
    #[schemars(length(min = 1, max = 128))]
    pub decision_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub account_id: Option<String>,
    pub environment: crate::protocol::ExecutionContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1))]
    pub policy_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub policy_state_version: Option<String>,
    pub status: RiskDecisionStatus,
    #[schemars(length(min = 1, max = 64))]
    pub evaluated_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    pub inputs: Vec<RiskDecisionInputReference>,
    pub checks: Vec<RiskCheckResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskDecisionHistory {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    pub decisions: Vec<RiskDecision>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskDecisionEvaluate {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskDecisionQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

impl RiskDecision {
    pub(crate) fn validate(
        &self,
        workspace_id: &str,
        proposal_id: &str,
        sequence: u64,
    ) -> crate::protocol::Result<()> {
        let status = RiskDecisionStatus::from_checks(&self.checks);
        if self.workspace_id != workspace_id
            || self.proposal_id != proposal_id
            || self.proposal_hash.len() != 71
            || !self.proposal_hash.starts_with("sha256:")
            || !self
                .proposal_hash
                .strip_prefix("sha256:")
                .is_some_and(|hash| {
                    hash.bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                })
            || !self.decision_id.starts_with("risk-decision:")
            || self.evaluated_at.is_empty()
            || self.policy_version == Some(0)
            || self.inputs.len() > 32
            || self.checks.is_empty()
            || self.checks.len() > 64
            || self
                .checks
                .iter()
                .any(|check| check.reason.is_empty() || check.reason.len() > 512)
            || self.checks.iter().enumerate().any(|(index, check)| {
                self.checks[..index]
                    .iter()
                    .any(|previous| previous.check_id == check.check_id)
            })
            || self.inputs.iter().any(|input| {
                input.reference_id.is_empty()
                    || input.reference_id.len() > 128
                    || input.digest.len() != 71
                    || !input.digest.starts_with("sha256:")
                    || !input.digest.strip_prefix("sha256:").is_some_and(|hash| {
                        hash.bytes()
                            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    })
            })
            || self.state_version != format!("risk-decision:{proposal_id}:{sequence}")
            || self.status != status
        {
            return Err(crate::protocol::TradeXError::new(
                "WORKSPACE_INTEGRITY_FAILED",
            ));
        }
        Ok(())
    }
}

pub(crate) fn input_reference<T: Serialize>(
    kind: RiskDecisionInputKind,
    reference_id: String,
    observed_at: Option<String>,
    value: &T,
) -> crate::protocol::Result<RiskDecisionInputReference> {
    let encoded = serde_json::to_vec(value)
        .map_err(|_| crate::protocol::TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    Ok(RiskDecisionInputReference {
        kind,
        reference_id,
        digest: format!("sha256:{}", hex::encode(Sha256::digest(encoded))),
        observed_at,
    })
}

pub(crate) fn evaluate(
    proposal: &crate::protocol::OrderProposal,
    policy_state: Option<&RiskPolicyState>,
    account: Option<&crate::providers::AccountConnection>,
    portfolio: Option<&crate::protocol::PortfolioSnapshot>,
    market: Option<&crate::protocol::MarketDetail>,
    time_status: &crate::protocol::TimeStatus,
    inputs: Vec<RiskDecisionInputReference>,
    evaluated_at: String,
) -> RiskDecision {
    use crate::protocol::{ExecutionContext, MarketSession, OrderSide, OrderType, TimeConfidence};
    use std::cmp::Ordering;

    let policy = policy_state.map(|state| &state.policy);
    let mut checks = Vec::with_capacity(27);
    macro_rules! push {
        ($id:expr, $outcome:expr, $reason_code:expr, $reason:expr $(,)?) => {
            checks.push(RiskCheckResult {
                check_id: $id,
                outcome: $outcome,
                reason_code: $reason_code,
                reason: $reason.into(),
            })
        };
    }
    macro_rules! limit {
        ($id:expr, $value:expr, $maximum:expr, $label:expr $(,)?) => {{
            let (outcome, reason_code, reason) = match $maximum {
                None => (
                    RiskCheckOutcome::Pass,
                    RiskDecisionReasonCode::LimitNotConfigured,
                    format!("{} limit is not configured.", $label),
                ),
                Some(_) if $value.is_none() => (
                    RiskCheckOutcome::Unavailable,
                    RiskDecisionReasonCode::EvidenceMissing,
                    format!("Trusted {} evidence is unavailable.", $label),
                ),
                Some(maximum) => match crate::provider_io::decimal_cmp($value.unwrap(), maximum) {
                    Ok(Ordering::Greater) => (
                        RiskCheckOutcome::Reject,
                        RiskDecisionReasonCode::LimitExceeded,
                        format!("{} exceeds the configured limit.", $label),
                    ),
                    Ok(_) => (
                        RiskCheckOutcome::Pass,
                        RiskDecisionReasonCode::WithinLimit,
                        format!("{} is within the configured limit.", $label),
                    ),
                    Err(_) => (
                        RiskCheckOutcome::Unavailable,
                        RiskDecisionReasonCode::InvalidProposalValue,
                        format!("{} could not be compared exactly.", $label),
                    ),
                },
            };
            push!($id, outcome, reason_code, reason);
        }};
    }

    let proposal_current = proposal.status == crate::protocol::OrderProposalStatus::NeedsApproval
        && proposal.workspace_id == time_status.workspace_id;
    push!(
        RiskCheckId::ProposalIdentity,
        if proposal_current {
            RiskCheckOutcome::Pass
        } else {
            RiskCheckOutcome::Reject
        },
        if proposal_current {
            RiskDecisionReasonCode::WithinLimit
        } else {
            RiskDecisionReasonCode::ProposalInvalidated
        },
        if proposal_current {
            "The immutable proposal is current in this workspace."
        } else {
            "The proposal is no longer eligible for risk evaluation."
        },
    );
    push!(
        RiskCheckId::PolicyConfigured,
        if policy_state.is_some_and(|state| state.configured) {
            RiskCheckOutcome::Pass
        } else {
            RiskCheckOutcome::Unavailable
        },
        if policy_state.is_some_and(|state| state.configured) {
            RiskDecisionReasonCode::WithinLimit
        } else {
            RiskDecisionReasonCode::PolicyUnconfigured
        },
        if policy_state.is_some_and(|state| state.configured) {
            "A persisted risk policy is configured."
        } else {
            "A persisted risk policy is not configured."
        },
    );

    let account_id = proposal.fields.account_id.as_deref();
    let expected = match proposal.fields.environment {
        ExecutionContext::LocalPaper => Some(("local-paper", "LOCAL")),
        ExecutionContext::AlpacaPaper => Some(("alpaca", "PAPER")),
        ExecutionContext::Trading212Demo => Some(("trading212", "DEMO")),
        ExecutionContext::Trading212Live => Some(("trading212", "LIVE")),
        ExecutionContext::BinanceTestnet => Some(("binance", "TESTNET")),
        ExecutionContext::BinanceLive => Some(("binance", "LIVE")),
        ExecutionContext::BitgetDemo => None,
        ExecutionContext::BitgetLive => Some(("bitget", "LIVE")),
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation => None,
    };
    let binding = match (account_id, account, expected) {
        (Some(id), Some(account), Some((provider, environment)))
            if id == account.connection_id
                && account.workspace_id == proposal.workspace_id
                && account.provider_id == provider
                && account.environment == environment =>
        {
            RiskCheckOutcome::Pass
        }
        (Some(_), Some(_), Some(_)) => RiskCheckOutcome::Reject,
        (_, _, None) => RiskCheckOutcome::Unavailable,
        _ => RiskCheckOutcome::Unavailable,
    };
    push!(
        RiskCheckId::AccountBinding,
        binding,
        match binding {
            RiskCheckOutcome::Pass => RiskDecisionReasonCode::WithinLimit,
            RiskCheckOutcome::Reject => RiskDecisionReasonCode::AccountMismatch,
            RiskCheckOutcome::Unavailable => RiskDecisionReasonCode::AccountUnavailable,
        },
        match binding {
            RiskCheckOutcome::Pass => "The proposal is bound to the exact account and environment.",
            RiskCheckOutcome::Reject => "The saved account does not match the proposal context.",
            RiskCheckOutcome::Unavailable => "A supported account identity is not available.",
        },
    );
    let account_healthy = account.is_some_and(|account| {
        account.connection_state == crate::providers::ConnectionState::Connected
            && matches!(account.health.connection.as_str(), "ONLINE" | "CONNECTED")
            && matches!(
                account.health.authentication.as_str(),
                "VALID" | "NOT_REQUIRED"
            )
            && matches!(
                account.health.credential.as_str(),
                "AVAILABLE" | "CONFIGURED" | "NOT_REQUIRED"
            )
    });
    push!(
        RiskCheckId::AccountHealth,
        if account_healthy {
            RiskCheckOutcome::Pass
        } else {
            RiskCheckOutcome::Unavailable
        },
        if account_healthy {
            RiskDecisionReasonCode::WithinLimit
        } else {
            RiskDecisionReasonCode::AccountUnhealthy
        },
        if account_healthy {
            "The saved account connection and authentication are healthy."
        } else {
            "The saved account is disconnected or its health is unverified."
        },
    );

    let environment = match proposal.fields.environment {
        ExecutionContext::LocalPaper => Some(crate::risk::RiskPolicyEnvironment::LocalPaper),
        ExecutionContext::AlpacaPaper | ExecutionContext::Trading212Live => Some(
            if proposal.fields.environment == ExecutionContext::AlpacaPaper {
                crate::risk::RiskPolicyEnvironment::Paper
            } else {
                crate::risk::RiskPolicyEnvironment::Live
            },
        ),
        ExecutionContext::Trading212Demo => Some(crate::risk::RiskPolicyEnvironment::Demo),
        ExecutionContext::BitgetDemo => None,
        ExecutionContext::BinanceTestnet => Some(crate::risk::RiskPolicyEnvironment::Testnet),
        ExecutionContext::BinanceLive | ExecutionContext::BitgetLive => {
            Some(crate::risk::RiskPolicyEnvironment::Live)
        }
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation => None,
    };
    let identity_check = |allowed: &[String], value: &str| {
        if allowed.is_empty() || allowed.iter().any(|candidate| candidate == value) {
            (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit)
        } else {
            (
                RiskCheckOutcome::Reject,
                RiskDecisionReasonCode::IdentifierNotAllowed,
            )
        }
    };
    let (outcome, reason) = match (policy, account_id) {
        (Some(policy), Some(account_id)) => identity_check(&policy.allowed_account_ids, account_id),
        (_, None) if policy.is_some_and(|policy| !policy.allowed_account_ids.is_empty()) => (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::AccountUnavailable,
        ),
        _ => (
            RiskCheckOutcome::Pass,
            RiskDecisionReasonCode::LimitNotConfigured,
        ),
    };
    push!(
        RiskCheckId::AllowedAccount,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The account is outside the configured allow-list."
        } else if outcome == RiskCheckOutcome::Unavailable {
            "The configured account allow-list cannot be checked without an account ID."
        } else {
            "The account allow-list does not block this proposal."
        },
    );
    let (outcome, reason) = match (policy, account_id) {
        (Some(policy), Some(account_id))
            if policy.blocked_account_ids.iter().any(|id| id == account_id) =>
        {
            (
                RiskCheckOutcome::Reject,
                RiskDecisionReasonCode::IdentifierBlocked,
            )
        }
        _ => (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit),
    };
    push!(
        RiskCheckId::BlockedAccount,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The account is in the configured block-list."
        } else {
            "The account is not in the configured block-list."
        },
    );
    let (outcome, reason) = match (policy, environment) {
        (Some(policy), Some(environment))
            if policy.allowed_environments.is_empty()
                || policy.allowed_environments.contains(&environment) =>
        {
            (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit)
        }
        (Some(_), Some(_)) => (
            RiskCheckOutcome::Reject,
            RiskDecisionReasonCode::EnvironmentNotAllowed,
        ),
        _ => (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::UnsupportedContext,
        ),
    };
    push!(
        RiskCheckId::AllowedEnvironment,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The execution environment is outside the configured allow-list."
        } else if outcome == RiskCheckOutcome::Unavailable {
            "This proposal has no supported policy environment mapping."
        } else {
            "The execution environment is allowed by policy."
        },
    );
    let (outcome, reason) = match policy {
        Some(policy)
            if policy.allowed_venues.is_empty()
                || policy.allowed_venues.contains(&proposal.fields.venue) =>
        {
            (
                RiskCheckOutcome::Pass,
                if policy.allowed_venues.is_empty() {
                    RiskDecisionReasonCode::LimitNotConfigured
                } else {
                    RiskDecisionReasonCode::WithinLimit
                },
            )
        }
        Some(_) => (
            RiskCheckOutcome::Reject,
            RiskDecisionReasonCode::IdentifierNotAllowed,
        ),
        None => (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::PolicyUnconfigured,
        ),
    };
    push!(
        RiskCheckId::AllowedVenue,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The venue is outside the configured allow-list."
        } else {
            "The venue is allowed by policy or no venue allow-list is configured."
        },
    );
    let (outcome, reason) = match policy {
        Some(policy)
            if policy.allowed_instrument_ids.is_empty()
                || policy
                    .allowed_instrument_ids
                    .contains(&proposal.fields.instrument_id) =>
        {
            (
                RiskCheckOutcome::Pass,
                if policy.allowed_instrument_ids.is_empty() {
                    RiskDecisionReasonCode::LimitNotConfigured
                } else {
                    RiskDecisionReasonCode::WithinLimit
                },
            )
        }
        Some(_) => (
            RiskCheckOutcome::Reject,
            RiskDecisionReasonCode::IdentifierNotAllowed,
        ),
        None => (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::PolicyUnconfigured,
        ),
    };
    push!(
        RiskCheckId::AllowedInstrument,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The instrument is outside the configured allow-list."
        } else {
            "The instrument is allowed by policy or no instrument allow-list is configured."
        },
    );
    let (outcome, reason) = match policy {
        Some(policy)
            if policy
                .blocked_venues
                .iter()
                .any(|venue| venue == &proposal.fields.venue) =>
        {
            (
                RiskCheckOutcome::Reject,
                RiskDecisionReasonCode::IdentifierBlocked,
            )
        }
        _ => (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit),
    };
    push!(
        RiskCheckId::BlockedVenue,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The venue is in the configured block-list."
        } else {
            "The venue is not in the configured block-list."
        },
    );
    let (outcome, reason) = match policy {
        Some(policy)
            if policy
                .blocked_instrument_ids
                .iter()
                .any(|id| id == &proposal.fields.instrument_id) =>
        {
            (
                RiskCheckOutcome::Reject,
                RiskDecisionReasonCode::IdentifierBlocked,
            )
        }
        _ => (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit),
    };
    push!(
        RiskCheckId::BlockedInstrument,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The instrument is in the configured block-list."
        } else {
            "The instrument is not in the configured block-list."
        },
    );

    let notional = proposal_notional_in_base(proposal, portfolio);
    limit!(
        RiskCheckId::OrderNotional,
        notional.as_deref(),
        policy.and_then(|policy| policy.max_order_notional.as_deref()),
        "Order notional",
    );
    limit!(
        RiskCheckId::OrderQuantity,
        Some(&proposal.fields.quantity.value),
        policy.and_then(|policy| policy.max_order_quantity.as_deref()),
        "Order quantity",
    );
    let current_instrument_value = portfolio.and_then(|portfolio| {
        portfolio_complete(portfolio)
            .then(|| instrument_value(portfolio, &proposal.fields.instrument_id))
            .flatten()
    });
    let position_size = match (current_instrument_value.as_deref(), notional.as_deref()) {
        (Some(current), Some(order)) => {
            let signed_order = if proposal.fields.side == OrderSide::Sell {
                format!("-{order}")
            } else {
                order.to_owned()
            };
            crate::portfolio::decimal_add(current, &signed_order)
                .ok()
                .map(|value| value.strip_prefix('-').unwrap_or(&value).to_owned())
        }
        _ => None,
    };
    limit!(
        RiskCheckId::PositionSize,
        position_size.as_deref(),
        policy.and_then(|policy| policy.max_position_size.as_deref()),
        "Position size",
    );
    let total_equity = portfolio
        .filter(|portfolio| portfolio_complete(portfolio))
        .and_then(|portfolio| portfolio.totals.equity.workspace_value.as_deref());
    let instrument_exposure = match (
        current_instrument_value.as_deref(),
        notional.as_deref(),
        total_equity,
    ) {
        (Some(current), Some(order), Some(equity)) => {
            exposure_percent(current, order, equity, proposal.fields.side)
        }
        _ => None,
    };
    limit!(
        RiskCheckId::SingleInstrumentExposure,
        instrument_exposure.as_deref(),
        policy.and_then(|policy| policy.max_single_instrument_exposure_percent.as_deref()),
        "Single-instrument exposure percent",
    );
    let configured_asset_class_limits = policy
        .map(|policy| &policy.max_asset_class_exposure_percent[..])
        .unwrap_or(&[]);
    let class_limit_exceeded = match (
        portfolio.filter(|portfolio| portfolio_complete(portfolio)),
        market,
        notional.as_deref(),
        total_equity,
    ) {
        (Some(portfolio), Some(market), Some(order), Some(equity)) => configured_asset_class_limits
            .iter()
            .try_fold(false, |exceeded, class_limit| {
                let current = class_exposure(portfolio, &class_limit.asset_class)?;
                let applies = market.instrument.asset_class == class_limit.asset_class;
                let exposure = exposure_percent(
                    &current,
                    if applies { order } else { "0" },
                    equity,
                    if applies {
                        proposal.fields.side
                    } else {
                        OrderSide::Buy
                    },
                )?;
                Some(
                    exceeded
                        || crate::provider_io::decimal_cmp(
                            &exposure,
                            &class_limit.max_exposure_percent,
                        )
                        .ok()?
                            == Ordering::Greater,
                )
            }),
        _ if configured_asset_class_limits.is_empty() => Some(false),
        _ => None,
    };
    let (outcome, reason) = if configured_asset_class_limits.is_empty() {
        (
            RiskCheckOutcome::Pass,
            RiskDecisionReasonCode::LimitNotConfigured,
        )
    } else if let Some(exceeded) = class_limit_exceeded {
        if exceeded {
            (
                RiskCheckOutcome::Reject,
                RiskDecisionReasonCode::LimitExceeded,
            )
        } else {
            (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit)
        }
    } else {
        (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceMissing,
        )
    };
    push!(
        RiskCheckId::AssetClassExposure,
        outcome,
        reason,
        if outcome == RiskCheckOutcome::Reject {
            "The proposed asset-class exposure exceeds its configured limit."
        } else if outcome == RiskCheckOutcome::Unavailable {
            "Workspace exposure or a trusted asset-class mapping is unavailable."
        } else {
            "Asset-class exposure is within limits or no class limit is configured."
        },
    );

    let daily_traded_limit = policy.and_then(|policy| policy.max_daily_traded_notional.as_deref());
    if daily_traded_limit.is_some() && time_status.confidence != TimeConfidence::Trusted {
        push!(
            RiskCheckId::DailyTradedNotional,
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::ClockUncertain,
            "Trusted TimeService evidence is required to determine the current daily trading window.",
        );
    } else {
        let daily_traded = portfolio.and_then(|portfolio| {
            if !portfolio_complete(portfolio) {
                return None;
            }
            let fills = portfolio.fills.as_ref()?;
            let date = time_status.wall_clock.get(..10)?;
            fills
                .iter()
                .filter(|fill| fill.observed_at.get(..10) == Some(date))
                .try_fold("0".to_owned(), |total, fill| {
                    let amount = portfolio_value(&fill.value)?;
                    crate::portfolio::decimal_add(&total, amount).ok()
                })
        });
        limit!(
            RiskCheckId::DailyTradedNotional,
            daily_traded.as_deref(),
            daily_traded_limit,
            "Daily traded notional",
        );
    }
    let daily_loss_limit = policy.and_then(|policy| policy.max_daily_realized_loss.as_deref());
    push!(
        RiskCheckId::DailyRealizedLoss,
        if daily_loss_limit.is_some() {
            RiskCheckOutcome::Unavailable
        } else {
            RiskCheckOutcome::Pass
        },
        if daily_loss_limit.is_some() {
            RiskDecisionReasonCode::CounterUnavailable
        } else {
            RiskDecisionReasonCode::LimitNotConfigured
        },
        if daily_loss_limit.is_some() {
            "Complete daily realized-loss counters are unavailable."
        } else {
            "Daily realized-loss limit is not configured."
        },
    );
    let (outcome, reason) = match policy.and_then(|policy| policy.max_open_orders) {
        None => (
            RiskCheckOutcome::Pass,
            RiskDecisionReasonCode::LimitNotConfigured,
        ),
        Some(maximum) => portfolio
            .filter(|portfolio| portfolio_complete(portfolio))
            .map_or(
                (
                    RiskCheckOutcome::Unavailable,
                    RiskDecisionReasonCode::CounterUnavailable,
                ),
                |portfolio| {
                    let count = portfolio
                        .open_orders
                        .iter()
                        .try_fold(0_u64, |count, order| match order.status.as_str() {
                            "FILLED" | "CANCELLED" | "CANCELED" | "REJECTED" | "EXPIRED" => {
                                Some(count)
                            }
                            "OPEN" | "PENDING" | "ACCEPTED" | "NEW" | "PARTIALLY_FILLED"
                            | "PENDING_CANCEL" | "CANCEL_PENDING" | "PROPOSED" => {
                                count.checked_add(1)
                            }
                            _ => None,
                        });
                    match count.and_then(|count| count.checked_add(1)) {
                        None => (
                            RiskCheckOutcome::Unavailable,
                            RiskDecisionReasonCode::CounterUnavailable,
                        ),
                        Some(projected_count) if projected_count > maximum as u64 => (
                            RiskCheckOutcome::Reject,
                            RiskDecisionReasonCode::LimitExceeded,
                        ),
                        Some(_) => (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit),
                    }
                },
            ),
    };
    push!(
        RiskCheckId::OpenOrderCount,
        outcome,
        reason,
        if reason == RiskDecisionReasonCode::LimitNotConfigured {
            "Open-order limit is not configured."
        } else if reason == RiskDecisionReasonCode::LimitExceeded {
            "Existing workspace open orders plus this proposal exceed the configured limit."
        } else if outcome == RiskCheckOutcome::Unavailable {
            "Complete workspace open-order observations are unavailable."
        } else {
            "Workspace open-order count is within the configured limit."
        },
    );
    let reserved_limit = policy.and_then(|policy| policy.max_reserved_capital.as_deref());
    push!(
        RiskCheckId::ReservedCapital,
        if reserved_limit.is_some() {
            RiskCheckOutcome::Unavailable
        } else {
            RiskCheckOutcome::Pass
        },
        if reserved_limit.is_some() {
            RiskDecisionReasonCode::ReservationUnavailable
        } else {
            RiskDecisionReasonCode::LimitNotConfigured
        },
        if reserved_limit.is_some() {
            "No complete workspace reservation ledger is available."
        } else {
            "Reserved-capital limit is not configured."
        },
    );

    let market_order = proposal.fields.order_type == OrderType::Market;
    let local_simulation = proposal.fields.environment == ExecutionContext::LocalPaper;
    let market_orders_enabled = policy.is_some_and(|policy| policy.market_orders_enabled);
    push!(
        RiskCheckId::MarketOrder,
        if !market_order {
            RiskCheckOutcome::Pass
        } else if market_orders_enabled {
            RiskCheckOutcome::Pass
        } else {
            RiskCheckOutcome::Reject
        },
        if !market_order {
            RiskDecisionReasonCode::NotApplicable
        } else if market_orders_enabled {
            RiskDecisionReasonCode::WithinLimit
        } else {
            RiskDecisionReasonCode::MarketOrderDisabled
        },
        if !market_order {
            "The proposal is a limit order; market-order policy is not applicable."
        } else if market_orders_enabled {
            "Market orders are enabled by policy."
        } else {
            "Market orders are disabled by policy."
        },
    );
    let slippage_limit =
        policy.and_then(|policy| policy.max_market_order_slippage_percent.as_deref());
    let (outcome, reason) = if local_simulation || !market_order || slippage_limit.is_none() {
        (
            RiskCheckOutcome::Pass,
            RiskDecisionReasonCode::NotApplicable,
        )
    } else if !market_orders_enabled {
        (
            RiskCheckOutcome::Reject,
            RiskDecisionReasonCode::MarketOrderDisabled,
        )
    } else {
        (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::ExecutionQuoteUnavailable,
        )
    };
    push!(
        RiskCheckId::MarketOrderSlippage,
        outcome,
        reason,
        if reason == RiskDecisionReasonCode::ExecutionQuoteUnavailable {
            "A size-aware executable quote is unavailable for slippage evaluation."
        } else if reason == RiskDecisionReasonCode::MarketOrderDisabled {
            "Market orders are disabled by policy."
        } else if local_simulation && market_order {
            "The local simulator validates its quote and slippage at submission."
        } else {
            "Market-order slippage is not applicable to this proposal."
        },
    );

    let (quote_outcome, quote_reason) = if local_simulation {
        (
            RiskCheckOutcome::Pass,
            RiskDecisionReasonCode::NotApplicable,
        )
    } else {
        market_freshness(
            market,
            time_status,
            policy.map_or(DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS, |policy| {
                policy.stale_quote_threshold_seconds
            }),
        )
    };
    push!(
        RiskCheckId::QuoteFreshness,
        quote_outcome,
        quote_reason,
        match quote_reason {
            RiskDecisionReasonCode::WithinLimit => {
                "A real-time quote is fresh and the clock is trusted."
            }
            RiskDecisionReasonCode::NotApplicable => {
                "The local simulator validates its quote at submission."
            }
            RiskDecisionReasonCode::EvidenceStale => {
                "The market observation is older than the configured threshold."
            }
            RiskDecisionReasonCode::EvidenceUntrusted => {
                "Market entitlement or time confidence is not trusted."
            }
            RiskDecisionReasonCode::ClockUncertain => {
                "The application clock is not trusted for a freshness check."
            }
            _ => "A required current market quote is unavailable.",
        },
    );
    let (outcome, reason) = if local_simulation {
        (
            RiskCheckOutcome::Pass,
            RiskDecisionReasonCode::NotApplicable,
        )
    } else {
        market
            .filter(|market| {
                market.market_state.source_status == crate::protocol::MarketDataStatus::Available
                    && market.market_state.source_id.is_some()
                    && market.market_state.calendar_version.is_some()
                    && market.market_state.time_confidence
                        == crate::protocol::TimeConfidence::Trusted
            })
            .map_or(
                (
                    RiskCheckOutcome::Unavailable,
                    RiskDecisionReasonCode::CalendarUnavailable,
                ),
                |market| match market.market_state.session {
                    MarketSession::Open => {
                        (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit)
                    }
                    MarketSession::Closed => (
                        RiskCheckOutcome::Reject,
                        RiskDecisionReasonCode::MarketClosed,
                    ),
                    MarketSession::Halted | MarketSession::Suspended => (
                        RiskCheckOutcome::Reject,
                        RiskDecisionReasonCode::MarketHalted,
                    ),
                    _ => (
                        RiskCheckOutcome::Unavailable,
                        RiskDecisionReasonCode::CalendarUnavailable,
                    ),
                },
            )
    };
    push!(
        RiskCheckId::MarketSession,
        outcome,
        reason,
        match reason {
            RiskDecisionReasonCode::MarketClosed => {
                "The authoritative market calendar reports a closed session."
            }
            RiskDecisionReasonCode::MarketHalted => {
                "The authoritative market state reports a halt or suspension."
            }
            RiskDecisionReasonCode::WithinLimit => "The market session is open.",
            _ => "A current authoritative market calendar is unavailable.",
        },
    );
    let deviation = match (
        policy.and_then(|policy| policy.max_price_deviation_percent.as_deref()),
        proposal.fields.limit_price.as_deref(),
        market.and_then(|market| market.snapshot.as_ref()),
    ) {
        (None, _, _) => None,
        (Some(_), Some(limit_price), Some(snapshot)) => snapshot
            .last_price
            .as_deref()
            .and_then(|last| price_deviation(limit_price, last)),
        _ => None,
    };
    let deviation_limit = policy.and_then(|policy| policy.max_price_deviation_percent.as_deref());
    if deviation_limit.is_none() {
        push!(
            RiskCheckId::PriceDeviation,
            RiskCheckOutcome::Pass,
            RiskDecisionReasonCode::LimitNotConfigured,
            "Maximum price deviation is not configured.",
        );
    } else if proposal.fields.order_type == OrderType::Market {
        push!(
            RiskCheckId::PriceDeviation,
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::ExecutionQuoteUnavailable,
            "A market order has no proposal limit price for price-deviation evaluation.",
        );
    } else {
        limit!(
            RiskCheckId::PriceDeviation,
            deviation.as_deref(),
            deviation_limit,
            "Price deviation percent",
        );
    }
    push!(
        RiskCheckId::InstrumentRules,
        if local_simulation {
            RiskCheckOutcome::Pass
        } else {
            RiskCheckOutcome::Unavailable
        },
        if local_simulation {
            RiskDecisionReasonCode::NotApplicable
        } else {
            RiskDecisionReasonCode::InstrumentRulesUnavailable
        },
        if local_simulation {
            "The local simulator validates its built-in order rules at submission."
        } else {
            "No authoritative provider instrument-rule snapshot is available."
        },
    );
    push!(
        RiskCheckId::LiveInactivity,
        RiskCheckOutcome::Pass,
        RiskDecisionReasonCode::NotApplicable,
        "Live inactivity timeout applies to a later Arm session, not this policy evaluation.",
    );

    let status = RiskDecisionStatus::from_checks(&checks);
    let context = proposal.fields.environment.clone();
    RiskDecision {
        decision_id: format!("risk-decision:{}", uuid::Uuid::new_v4()),
        workspace_id: proposal.workspace_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_hash: proposal.proposal_hash.clone(),
        account_id: account_id.map(str::to_owned),
        environment: context,
        policy_version: policy_state.map(|state| state.policy_version),
        policy_state_version: policy_state.map(|state| state.state_version.clone()),
        status,
        evaluated_at,
        state_version: String::new(),
        inputs,
        checks,
    }
}

fn proposal_notional_in_base(
    proposal: &crate::protocol::OrderProposal,
    portfolio: Option<&crate::protocol::PortfolioSnapshot>,
) -> Option<String> {
    let portfolio = portfolio?;
    if !portfolio_complete(portfolio) {
        return None;
    }
    (proposal.estimated_notional_currency.as_deref() == Some(portfolio.base_currency.as_str()))
        .then(|| proposal.estimated_notional.clone())
        .flatten()
}

fn portfolio_complete(portfolio: &crate::protocol::PortfolioSnapshot) -> bool {
    use crate::{protocol::PortfolioStatus, providers::ConnectionState};

    portfolio.status == PortfolioStatus::Available
        && !portfolio.accounts.is_empty()
        && portfolio.totals.equity.workspace_value.is_some()
        && portfolio.accounts.iter().all(|account| {
            account.connection_state == ConnectionState::Connected
                && matches!(account.health.connection.as_str(), "ONLINE" | "CONNECTED")
                && matches!(
                    account.health.authentication.as_str(),
                    "VALID" | "NOT_REQUIRED"
                )
                && matches!(
                    account.health.credential.as_str(),
                    "AVAILABLE" | "CONFIGURED" | "NOT_REQUIRED"
                )
                && account.equity.workspace_value.is_some()
        })
}

fn portfolio_value(value: &crate::protocol::PortfolioValue) -> Option<&str> {
    if value.workspace_currency == value.native_currency.as_deref().unwrap_or("") {
        value.native_value.as_deref()
    } else {
        value.workspace_value.as_deref()
    }
}

fn instrument_value(
    portfolio: &crate::protocol::PortfolioSnapshot,
    instrument_id: &str,
) -> Option<String> {
    portfolio
        .holdings
        .iter()
        .filter(|holding| holding.instrument_id.as_deref() == Some(instrument_id))
        .try_fold("0".to_owned(), |total, holding| {
            crate::portfolio::decimal_add(&total, portfolio_value(&holding.value)?).ok()
        })
}

fn class_exposure(
    portfolio: &crate::protocol::PortfolioSnapshot,
    class: &crate::protocol::AssetClass,
) -> Option<String> {
    let mut total = "0".to_owned();
    for holding in &portfolio.holdings {
        let holding_class = match holding.instrument_id.as_deref() {
            Some(id) if id.starts_with("equity:") => Some(crate::protocol::AssetClass::Equity),
            Some(id) if id.starts_with("crypto:") => Some(crate::protocol::AssetClass::CryptoSpot),
            Some(_) => return None,
            None if portfolio.accounts.iter().any(|account| {
                account.connection_id == holding.connection_id
                    && account.account_currency.as_deref() == Some(&holding.asset)
            }) =>
            {
                None
            }
            None => return None,
        };
        if holding_class.as_ref() == Some(class) {
            total = crate::portfolio::decimal_add(&total, portfolio_value(&holding.value)?).ok()?;
        }
    }
    Some(total)
}

fn exposure_percent(
    current: &str,
    proposed: &str,
    equity: &str,
    side: crate::protocol::OrderSide,
) -> Option<String> {
    let proposed = if side == crate::protocol::OrderSide::Sell {
        format!("-{proposed}")
    } else {
        proposed.to_owned()
    };
    let exposure = crate::portfolio::decimal_add(current, &proposed).ok()?;
    let exposure = exposure.strip_prefix('-').unwrap_or(&exposure);
    crate::portfolio::decimal_mul(
        &crate::portfolio::decimal_div(exposure, equity).ok()?,
        "100",
    )
    .ok()
}

fn price_deviation(limit_price: &str, last_price: &str) -> Option<String> {
    use std::cmp::Ordering;
    if crate::provider_io::decimal_cmp(last_price, "0").ok()? != Ordering::Greater {
        return None;
    }
    let difference = crate::portfolio::decimal_add(limit_price, &format!("-{last_price}")).ok()?;
    let difference = difference.strip_prefix('-').unwrap_or(&difference);
    crate::portfolio::decimal_mul(
        &crate::portfolio::decimal_div(difference, last_price).ok()?,
        "100",
    )
    .ok()
}

fn market_freshness(
    market: Option<&crate::protocol::MarketDetail>,
    time_status: &crate::protocol::TimeStatus,
    maximum_age_seconds: u64,
) -> (RiskCheckOutcome, RiskDecisionReasonCode) {
    use crate::protocol::{MarketDataStatus, MarketEntitlement, MarketFreshness, TimeConfidence};
    use time::format_description::well_known::Rfc3339;

    if time_status.confidence != TimeConfidence::Trusted {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::ClockUncertain,
        );
    }
    let Some(market) = market else {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceMissing,
        );
    };
    if market.status != MarketDataStatus::Available {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceUntrusted,
        );
    }
    let Some(snapshot) = &market.snapshot else {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceMissing,
        );
    };
    if snapshot.provenance.entitlement != MarketEntitlement::Realtime
        || snapshot.provenance.freshness != MarketFreshness::Healthy
        || snapshot.last_price.is_none()
    {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceUntrusted,
        );
    }
    let parse = |value: &str| time::OffsetDateTime::parse(value, &Rfc3339).ok();
    let (Some(now), Some(received), Some(provider)) = (
        parse(&time_status.wall_clock),
        parse(&snapshot.provenance.received_timestamp),
        parse(&snapshot.provenance.provider_timestamp),
    ) else {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceMissing,
        );
    };
    let now = now.unix_timestamp();
    let ages = [
        now - received.unix_timestamp(),
        now - provider.unix_timestamp(),
    ];
    if ages.iter().any(|age| *age < 0) {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceUntrusted,
        );
    }
    if ages.iter().any(|age| *age as u64 > maximum_age_seconds) {
        return (
            RiskCheckOutcome::Unavailable,
            RiskDecisionReasonCode::EvidenceStale,
        );
    }
    (RiskCheckOutcome::Pass, RiskDecisionReasonCode::WithinLimit)
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

fn valid_instrument_ids(values: &[String]) -> bool {
    valid_unique_values(values, 128, crate::market::validate_instrument_id)
}

fn valid_identifier_list(values: &[String], max_length: usize) -> bool {
    valid_unique_values(values, max_length, |value| {
        !value.is_empty()
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':')
            })
    })
}

fn valid_unique_values(values: &[String], max_length: usize, valid: impl Fn(&str) -> bool) -> bool {
    if values.len() > 256 {
        return false;
    }
    let mut seen = HashSet::with_capacity(values.len());
    values
        .iter()
        .all(|value| value.len() <= max_length && valid(value) && seen.insert(value.as_str()))
}

fn duplicate_asset_class_limits(values: &[RiskAssetClassLimit]) -> bool {
    values.iter().enumerate().any(|(index, value)| {
        values[..index]
            .iter()
            .any(|prior| prior.asset_class == value.asset_class)
    })
}

fn duplicate_values<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].iter().any(|prior| prior == value))
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

    #[test]
    fn legacy_persisted_policy_keeps_market_order_preference_with_new_limits_unset() {
        let mut legacy = RiskPolicyState::new("workspace".into());
        legacy.state_version = "legacy-state-version".into();
        legacy.configured = true;
        legacy.policy.market_orders_enabled = true;
        legacy.updated_at = "2026-09-25T00:00:00Z".into();
        let mut value = serde_json::to_value(legacy).unwrap();
        let policy = value["policy"].as_object_mut().unwrap();
        for field in [
            "maxOrderQuantity",
            "maxPositionSize",
            "maxAssetClassExposurePercent",
            "maxOpenOrders",
            "maxReservedCapital",
            "allowedInstrumentIds",
            "blockedInstrumentIds",
            "allowedVenues",
            "blockedVenues",
            "allowedAccountIds",
            "blockedAccountIds",
            "allowedEnvironments",
            "maxMarketOrderSlippagePercent",
            "maxPriceDeviationPercent",
        ] {
            policy.remove(field);
        }

        let restored =
            RiskPolicyState::from_persisted_json(&value.to_string(), "workspace").unwrap();
        assert!(restored.policy.market_orders_enabled);
        assert_eq!(restored.policy.max_market_order_slippage_percent, None);
        assert_eq!(restored.policy.max_order_quantity, None);
        assert!(restored.policy.allowed_instrument_ids.is_empty());

        let mut unsaved = restored;
        assert_eq!(
            unsaved.validate_policy_for_save().unwrap_err().code,
            "RISK_POLICY_INVALID"
        );
    }
}
