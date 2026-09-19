use crate::capability::{CapabilityDecision, CapabilityQuery, ContextCatalog, ResearchToolId};
use crate::gateway::{GatewayMutation, GatewayState};
use crate::model::{
    ChatgptLogin, ConfigureDeepseek, ModelQuery, ModelState, SetDefaultModel, SetFallbackPolicy,
    VerifyRoute,
};
use crate::providers::*;
use crate::risk::{
    CompleteOnboarding, RiskPolicyState, RiskQuery, SaveRiskPolicy, SetOnboardingStep,
};
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

fn present<'de, D, T>(value: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(value).map(Some)
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeConfidence {
    Trusted,
    ClockUncertain,
    Stale,
}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimeStatus {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub confidence: TimeConfidence,
    #[schemars(length(min = 1, max = 64))]
    pub wall_clock: String,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub monotonic_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_offset_ms: Option<i64>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[schemars(length(min = 1, max = 512))]
    pub reason: String,
    pub remediation: Remediation,
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
    #[schemars(extend("enum" = ["workspace", "account", "model-gateway", "model", "risk", "thread"]))]
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
    Time(TimeStatus),
    Subscription(SubscriptionAck),
    Thread(Box<Thread>),
    Threads(ThreadList),
    Gateway(GatewayState),
    Model(ModelState),
    Risk(RiskPolicyState),
    ProviderCatalog(ProviderCatalog),
    ProviderDefinition(ProviderDefinition),
    Accounts(Accounts),
    Account(Box<AccountConnection>),
    Permissions(PermissionReview),
    Capability(CapabilityDecision),
    ContextCatalog(ContextCatalog),
    DataSourceCatalog(DataSourceCatalog),
    MarketCatalog(MarketCatalog),
    MarketDetail(Box<MarketDetail>),
    Portfolio(Box<PortfolioSnapshot>),
    Watchlist(Box<Watchlist>),
    Watchlists(Watchlists),
    ResearchResult(ResearchToolResult),
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
    pub capability_query: CapabilityQuery,
    pub context_catalog: WorkspaceQuery,
    pub gateway_mutation: GatewayMutation,
    pub model_query: ModelQuery,
    pub chatgpt_login: ChatgptLogin,
    pub configure_deepseek: ConfigureDeepseek,
    pub verify_route: VerifyRoute,
    pub set_default_model: SetDefaultModel,
    pub set_fallback_policy: SetFallbackPolicy,
    pub risk_query: RiskQuery,
    pub save_risk_policy: SaveRiskPolicy,
    pub set_onboarding_step: SetOnboardingStep,
    pub complete_onboarding: CompleteOnboarding,
    pub command: CommandEnvelope,
    pub result: ResultEnvelope,
    pub event: DomainEvent,
    pub workspace_open: OpenWorkspace,
    pub aggregate: Aggregate,
    pub subscribe: Subscribe,
    pub empty: EmptyPayload,
    pub time_status: TimeStatus,
    pub provider_selection: ProviderSelection,
    pub workspace_query: WorkspaceQuery,
    pub account_query: AccountQuery,
    pub account_mutation: AccountMutation,
    pub provider_connect: Connect,
    pub thread_create: ThreadCreate,
    pub thread_query: ThreadQuery,
    pub turn_start: TurnStart,
    pub turn_cancel: TurnCancel,
    pub turn_retry: TurnRetry,
    pub research_request: ResearchToolRequest,
    pub research_invocation: ResearchToolInvocation,
    pub research_result: ResearchToolResult,
    pub data_source_query: DataSourceQuery,
    pub data_source_probe: DataSourceProbe,
    pub market_catalog_query: MarketCatalogQuery,
    pub market_get_query: MarketGetQuery,
    pub portfolio_query: PortfolioQuery,
    pub watchlist_create: WatchlistCreate,
    pub watchlist_rename: WatchlistRename,
    pub watchlist_delete: WatchlistDelete,
    pub watchlist_instrument_mutation: WatchlistInstrumentMutation,
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
    #[schemars(range(min = 1, max = 7))]
    pub storage_schema_version: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentMode {
    Ask,
    Research,
    Backtest,
    Trade,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ExecutionContext {
    #[serde(rename = "NONE_READ_ONLY")]
    NoneReadOnly,
    #[serde(rename = "HISTORICAL_SIMULATION")]
    HistoricalSimulation,
    #[serde(rename = "LOCAL_PAPER")]
    LocalPaper,
    #[serde(rename = "ALPACA_PAPER")]
    AlpacaPaper,
    #[serde(rename = "TRADING212_DEMO")]
    Trading212Demo,
    #[serde(rename = "TRADING212_LIVE")]
    Trading212Live,
    #[serde(rename = "BINANCE_TESTNET")]
    BinanceTestnet,
    #[serde(rename = "BINANCE_LIVE")]
    BinanceLive,
    #[serde(rename = "BITGET_DEMO")]
    BitgetDemo,
    #[serde(rename = "BITGET_LIVE")]
    BitgetLive,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThreadStatus {
    Active,
    Archived,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TurnStatus {
    Running,
    Completed,
    Cancelled,
    Interrupted,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemStatus {
    Started,
    Streaming,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadModel {
    #[schemars(length(min = 1, max = 32))]
    pub provider: String,
    #[schemars(length(min = 1, max = 128))]
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 32))]
    pub thinking_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadContextRef {
    #[schemars(length(min = 1, max = 64))]
    pub kind: String,
    #[schemars(length(min = 1, max = 256))]
    pub id: String,
    #[schemars(length(min = 1, max = 256))]
    pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchResultState {
    Available,
    Degraded,
    Unavailable,
    BlockedExternal,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchFocus {
    General,
    Equity,
    CryptoSpot,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchSpotVenueId {
    Binance,
    Bitget,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchFreshness {
    Healthy,
    Stale,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchQuality {
    Verified,
    Degraded,
    Unknown,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchFinding {
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    #[schemars(length(min = 1, max = 512))]
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchScenario {
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    #[schemars(length(min = 1, max = 512))]
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchProvenance {
    #[schemars(length(min = 1, max = 128))]
    pub source_id: String,
    #[schemars(length(min = 1, max = 160))]
    pub provider: String,
    pub status: DataSourceStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub provider_timestamp: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub received_timestamp: String,
    pub freshness: ResearchFreshness,
    pub quality: ResearchQuality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 512))]
    pub limitation: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchSpotVenue {
    pub venue: ResearchSpotVenueId,
    pub state: ResearchResultState,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    #[schemars(length(min = 1, max = 64))]
    pub bid: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 64))]
    pub ask: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 64))]
    pub spread: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 64))]
    pub depth: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 64))]
    pub quote_age: Option<String>,
    pub provenance: ResearchProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 512))]
    pub limitation: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchToolPayload {
    pub state: ResearchResultState,
    #[schemars(length(min = 1, max = 256))]
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<ResearchFocus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 512))]
    pub conclusion: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub findings: Vec<ResearchFinding>,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub scenarios: Vec<ResearchScenario>,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub evidence: Vec<ResearchProvenance>,
    #[serde(default)]
    #[schemars(length(max = 8), inner(length(min = 1, max = 512)))]
    pub limitations: Vec<String>,
    #[serde(default)]
    #[schemars(length(max = 8), inner(length(min = 1, max = 128)))]
    pub instrument_refs: Vec<String>,
    #[serde(default)]
    #[schemars(length(max = 8), inner(length(min = 1, max = 128)))]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    #[schemars(length(max = 2))]
    pub spot_venues: Vec<ResearchSpotVenue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub fixture_label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchToolRequest {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub agent_mode: AgentMode,
    pub execution_context: ExecutionContext,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<ResearchFocus>,
    #[schemars(length(max = 32))]
    pub attached_contexts: Vec<ThreadContextRef>,
    pub tool_id: ResearchToolId,
    #[schemars(length(min = 1, max = 100_000))]
    pub query: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchToolInvocation {
    pub tool_id: ResearchToolId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<ResearchFocus>,
    #[schemars(length(min = 1, max = 100_000))]
    pub query: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchToolResult {
    #[schemars(length(min = 1, max = 128))]
    pub result_id: String,
    pub tool_id: ResearchToolId,
    #[schemars(length(min = 1, max = 128))]
    pub source_id: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[schemars(length(min = 1, max = 80))]
    pub request_hash: String,
    #[schemars(length(min = 1, max = 96))]
    pub marker: String,
    #[schemars(length(max = 32))]
    pub context_refs: Vec<ThreadContextRef>,
    pub payload: ResearchToolPayload,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataSourceStatus {
    Available,
    Unavailable,
    BlockedExternal,
    Unverified,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataSourceProbeKind {
    PublicMetadata,
    CredentialedMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceEntry {
    #[schemars(length(min = 1, max = 32))]
    pub source_id: String,
    #[schemars(length(min = 1, max = 160))]
    pub provider: String,
    #[schemars(length(max = 8))]
    pub capabilities: Vec<String>,
    #[schemars(length(min = 1, max = 512))]
    pub coverage: String,
    #[schemars(length(min = 1, max = 256))]
    pub latency: String,
    #[schemars(length(min = 1, max = 512))]
    pub entitlement: String,
    #[schemars(length(min = 1, max = 512))]
    pub retention: String,
    #[schemars(length(min = 1, max = 512))]
    pub redistribution: String,
    #[schemars(length(min = 1, max = 256))]
    pub commercial_use: String,
    #[schemars(length(min = 1, max = 256))]
    pub jurisdictions: String,
    #[schemars(length(min = 1, max = 512))]
    pub official_url: String,
    #[schemars(length(min = 1, max = 512))]
    pub terms_url: String,
    #[schemars(length(min = 10, max = 32))]
    pub reviewed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub checked_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub observed_at: Option<String>,
    pub probe_kind: DataSourceProbeKind,
    pub status: DataSourceStatus,
    pub configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub verified_at: Option<String>,
    #[schemars(length(min = 1, max = 512))]
    pub availability_reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceCatalog {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 8))]
    pub sources: Vec<DataSourceEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketTier {
    #[default]
    Census,
    Warm,
    Hot,
    Cold,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketDataStatus {
    Available,
    Unavailable,
    BlockedExternal,
    Unverified,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketSession {
    Open,
    Closed,
    ExtendedHours,
    Halted,
    Maintenance,
    Suspended,
    Degraded,
    Unknown,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdjustmentStatus {
    Adjusted,
    Unadjusted,
    Unknown,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CorporateActionType {
    Split,
    Dividend,
    SymbolChange,
    Delisting,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketEntitlement {
    Realtime,
    Delayed,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketFreshness {
    Healthy,
    Stale,
    ClockUncertain,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetClass {
    Equity,
    CryptoSpot,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstrumentProviderMapping {
    #[schemars(length(min = 1, max = 32))]
    pub provider_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub provider_symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Instrument {
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub asset_class: AssetClass,
    #[schemars(length(min = 1, max = 32))]
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub base: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 16))]
    pub exchange: Option<String>,
    #[schemars(regex(pattern = "^[A-Z][A-Z0-9]{2,7}$"))]
    pub currency: String,
    #[schemars(length(min = 1, max = 160))]
    pub display_name: String,
    #[schemars(length(max = 8))]
    pub providers: Vec<InstrumentProviderMapping>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketSnapshotProvenance {
    #[schemars(length(min = 1, max = 128))]
    pub market_snapshot_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub venue: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub provider_timestamp: String,
    #[schemars(length(min = 1, max = 64))]
    pub received_timestamp: String,
    pub entitlement: MarketEntitlement,
    pub freshness: MarketFreshness,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketSnapshot {
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub provenance: MarketSnapshotProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub last_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub bid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub ask: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketCatalog {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(max = 120))]
    pub query: String,
    pub tier: MarketTier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub source_id: Option<String>,
    pub status: MarketDataStatus,
    #[schemars(length(min = 1, max = 512))]
    pub availability_reason: String,
    #[schemars(length(max = 256))]
    pub instruments: Vec<Instrument>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketState {
    pub session: MarketSession,
    #[schemars(length(min = 1, max = 32))]
    pub venue: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub source_id: Option<String>,
    pub source_status: MarketDataStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub next_open: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub next_close: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub calendar_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub provider_time: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub time_confidence: TimeConfidence,
    #[schemars(length(min = 1, max = 512))]
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorporateAction {
    #[schemars(length(min = 1, max = 128))]
    pub action_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub action_type: CorporateActionType,
    #[schemars(length(min = 1, max = 64))]
    pub effective_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub announced_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub source_id: Option<String>,
    #[schemars(length(min = 1, max = 256))]
    pub description: String,
    pub adjustment_status: AdjustmentStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketDetail {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub instrument: Instrument,
    pub tier: MarketTier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub source_id: Option<String>,
    pub status: MarketDataStatus,
    #[schemars(length(min = 1, max = 512))]
    pub availability_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<MarketSnapshot>,
    pub market_state: MarketState,
    #[schemars(length(max = 16))]
    pub corporate_actions: Vec<CorporateAction>,
    pub adjustment_status: AdjustmentStatus,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceProbe {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub source_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketCatalogQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[serde(default)]
    #[schemars(length(max = 120))]
    pub query: String,
    #[serde(default)]
    pub tier: MarketTier,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketGetQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub tier: MarketTier,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortfolioStatus {
    Available,
    Degraded,
    Unavailable,
    BlockedExternal,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FxFreshness {
    Healthy,
    Stale,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FxQuality {
    Verified,
    Degraded,
    Unknown,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FxProvenance {
    #[schemars(length(min = 1, max = 32))]
    pub source_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub pair_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub rate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub provider_timestamp: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub received_timestamp: String,
    pub freshness: FxFreshness,
    pub quality: FxQuality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub depeg_warning: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub native_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 16))]
    pub native_currency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub account_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 16))]
    pub account_currency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub workspace_value: Option<String>,
    #[schemars(length(min = 1, max = 16))]
    pub workspace_currency: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fx_provenance: Option<FxProvenance>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioHolding {
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 120))]
    pub account_label: String,
    #[schemars(length(min = 1, max = 32))]
    pub provider_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub environment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 32))]
    pub venue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub asset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub quantity: Option<String>,
    pub value: PortfolioValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unrealized_pnl: Option<PortfolioValue>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub health: AccountHealth,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioOrder {
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 120))]
    pub account_label: String,
    #[schemars(length(min = 1, max = 128))]
    pub broker_order_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub asset: String,
    #[schemars(length(min = 1, max = 16))]
    pub side: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub quantity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub notional: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 16))]
    pub currency: Option<String>,
    #[schemars(length(min = 1, max = 32))]
    pub status: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub health: AccountHealth,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioFill {
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 120))]
    pub account_label: String,
    #[schemars(length(min = 1, max = 128))]
    pub fill_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub asset: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub quantity: String,
    pub value: PortfolioValue,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub health: AccountHealth,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioAccount {
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 120))]
    pub label: String,
    #[schemars(length(min = 1, max = 32))]
    pub provider_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub environment: String,
    pub connection_state: ConnectionState,
    pub health: AccountHealth,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 16))]
    pub account_currency: Option<String>,
    pub equity: PortfolioValue,
    pub cash: PortfolioValue,
    #[schemars(range(min = 0, max = 10000))]
    pub positions_count: u32,
    #[schemars(range(min = 0, max = 10000))]
    pub open_orders_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0, max = 10000))]
    pub fills_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioTotals {
    pub equity: PortfolioValue,
    pub cash: PortfolioValue,
    pub unrealized_pnl: PortfolioValue,
    pub realized_pnl: PortfolioValue,
    pub exposure: PortfolioValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioLiveRisk {
    pub eligible: bool,
    #[schemars(length(min = 1, max = 512))]
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortfolioSnapshot {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub base_currency: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub status: PortfolioStatus,
    #[schemars(length(min = 1, max = 512))]
    pub availability_reason: String,
    pub totals: PortfolioTotals,
    #[schemars(length(max = 256))]
    pub accounts: Vec<PortfolioAccount>,
    #[schemars(length(max = 512))]
    pub holdings: Vec<PortfolioHolding>,
    #[schemars(length(max = 512))]
    pub open_orders: Vec<PortfolioOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(max = 512))]
    pub fills: Option<Vec<PortfolioFill>>,
    #[schemars(length(max = 128))]
    pub fx_routes: Vec<FxProvenance>,
    pub live_risk: PortfolioLiveRisk,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WatchlistItem {
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Watchlist {
    #[schemars(length(min = 1, max = 128))]
    pub watchlist_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 256))]
    pub items: Vec<WatchlistItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Watchlists {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 128))]
    pub watchlists: Vec<Watchlist>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WatchlistCreate {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WatchlistRename {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub watchlist_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WatchlistDelete {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub watchlist_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WatchlistInstrumentMutation {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub watchlist_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TurnSnapshot {
    #[schemars(length(min = 1, max = 128))]
    pub turn_id: String,
    pub agent_mode: AgentMode,
    pub execution_context: ExecutionContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 16))]
    pub account_environment: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub capability_level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ThreadModel>,
    pub attached_contexts: Vec<ThreadContextRef>,
    pub started_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadItem {
    #[schemars(length(min = 1, max = 128))]
    pub item_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub item_type: String,
    pub status: ItemStatus,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_result: Option<ResearchToolResult>,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadProviderAttempt {
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub provider: String,
    #[schemars(length(min = 1, max = 128))]
    pub model_id: String,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[schemars(length(min = 1, max = 32))]
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub error_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadTurn {
    #[schemars(length(min = 1, max = 128))]
    pub turn_id: String,
    pub status: TurnStatus,
    pub snapshot: TurnSnapshot,
    pub items: Vec<ThreadItem>,
    pub provider_attempts: Vec<ThreadProviderAttempt>,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_requested_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Thread {
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub codex_thread_id: Option<String>,
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    pub default_agent_mode: AgentMode,
    pub default_execution_context: ExecutionContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ThreadModel>,
    pub linked_contexts: Vec<ThreadContextRef>,
    pub status: ThreadStatus,
    #[serde(default)]
    pub turns: Vec<ThreadTurn>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadSummary {
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    pub updated_at: String,
    pub default_agent_mode: AgentMode,
    pub default_execution_context: ExecutionContext,
    pub status: ThreadStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadList {
    pub threads: Vec<ThreadSummary>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadCreate {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: Option<String>,
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    pub default_agent_mode: AgentMode,
    pub default_execution_context: ExecutionContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ThreadModel>,
    pub linked_contexts: Vec<ThreadContextRef>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThreadQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TurnStart {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    #[schemars(length(min = 1, max = 100_000))]
    pub message: String,
    pub agent_mode: AgentMode,
    pub execution_context: ExecutionContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ThreadModel>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "Vec<ThreadContextRef>", length(max = 32))]
    pub attached_contexts: Option<Vec<ThreadContextRef>>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub research_invocation: Option<ResearchToolInvocation>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub research_result: Option<ResearchToolResult>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TurnCancel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub turn_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TurnRetry {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub turn_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DomainProjection {
    Gateway(GatewayState),
    Model(ModelState),
    Workspace(Workspace),
    Account(Box<AccountConnection>),
    Risk(RiskPolicyState),
    Thread(Box<Thread>),
}

impl DomainProjection {
    pub fn id(&self) -> &str {
        match self {
            Self::Gateway(g) => &g.workspace_id,
            Self::Model(m) => &m.workspace_id,
            Self::Workspace(w) => &w.workspace_id,
            Self::Account(a) => &a.connection_id,
            Self::Risk(r) => &r.workspace_id,
            Self::Thread(t) => &t.thread_id,
        }
    }
    pub fn kind(&self) -> &str {
        match self {
            Self::Gateway(_) => "model-gateway",
            Self::Model(_) => "model",
            Self::Workspace(_) => "workspace",
            Self::Account(_) => "account",
            Self::Risk(_) => "risk",
            Self::Thread(_) => "thread",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DomainEvent {
    #[schemars(length(min = 1))]
    pub event_id: String,
    #[schemars(extend("enum" = ["workspace.opened", "account.health.changed", "model.gateway.changed", "model.provider.changed", "model.provider_attempt.changed", "risk.policy.changed", "thread.created", "thread.updated"]))]
    pub event_type: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    pub occurred_at: String,
    #[schemars(extend("enum" = ["workspace", "account", "model-gateway", "model", "risk", "thread"]))]
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
    #[schemars(extend("enum" = ["workspace", "account", "model-gateway", "model", "risk", "thread"]))]
    pub aggregate_type: String,
    #[schemars(length(min = 1))]
    pub aggregate_id: String,
    pub projection: DomainProjection,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub last_sequence: u64,
}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
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
            "DATA_SOURCE_UNKNOWN" => (
                "That data source is not in the current TradeX policy.",
                "reload_snapshot",
                "Reload data sources",
            ),
            "DATA_SOURCE_PROBE_FAILED" => (
                "The data source probe could not be completed. No source data was changed.",
                "retry_request",
                "Retry probe",
            ),
            "MARKET_INSTRUMENT_INVALID" => (
                "That instrument identifier is not a supported canonical TradeX ID.",
                "reload_snapshot",
                "Reload market catalog",
            ),
            "MARKET_INSTRUMENT_NOT_FOUND" => (
                "The selected instrument is not in the current catalog.",
                "reload_snapshot",
                "Reload market catalog",
            ),
            "MARKET_HISTORY_LIMIT" => (
                "The local historical cache reached its bounded storage limit.",
                "retry_request",
                "Retry after cleanup",
            ),
            "MARKET_HISTORY_UNAVAILABLE" => (
                "The historical market source is unavailable or not entitled; no history was stored.",
                "reload_snapshot",
                "Review data sources",
            ),
            "MARKET_CLOSED" => (
                "The market session is closed or unavailable; live authority remains blocked.",
                "view_market_state",
                "View market state",
            ),
            "INSTRUMENT_HALTED" => (
                "The instrument is halted; live authority remains blocked until the venue recovers.",
                "view_market_state",
                "View market state",
            ),
            "WATCHLIST_NAME_CONFLICT" => (
                "A watchlist with this name already exists in the workspace.",
                "choose_watchlist_name",
                "Choose another name",
            ),
            "WATCHLIST_NOT_FOUND" => (
                "That watchlist is no longer available. Reload the watchlist library.",
                "reload_snapshot",
                "Reload watchlists",
            ),
            "PROVIDER_ALREADY_CONNECTED" => (
                "This account is already connected in this environment. Use the existing connection.",
                "select_account",
                "Open existing connection",
            ),
            "PROVIDER_UNSUPPORTED" => (
                "This provider/environment does not support the requested operation.",
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
                "TradeX time could not be trusted. Live authority decisions remain blocked.",
                "time_revalidate",
                "Synchronize time",
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
            code if code.starts_with("GATEWAY_") => (
                "The model gateway needs attention. Check its status before retrying.",
                "check_gateway",
                "Check model gateway",
            ),
            "MODEL_NATIVE_ENTRY_REQUIRED" => (
                "Open the desktop app to configure this model securely.",
                "open_desktop",
                "Open TradeX",
            ),
            "MODEL_ENTRY_BUSY" => (
                "Finish or cancel the open model credential window before starting another one.",
                "finish_entry",
                "Return to credential entry",
            ),
            "MODEL_ENTRY_CANCELLED" => (
                "Model credential entry was cancelled. The previous configuration was preserved.",
                "configure_model",
                "Configure again",
            ),
            "MODEL_GATEWAY_RUNNING" => (
                "Stop the model gateway before replacing the DeepSeek key, then launch it again.",
                "stop_gateway",
                "Stop gateway",
            ),
            "MODEL_DEFAULT_MISSING" => (
                "Choose a verified default model route before starting a model request.",
                "choose_model",
                "Choose default model",
            ),
            "MODEL_FALLBACK_UNAVAILABLE" => (
                "Automatic fallback needs a verified DeepSeek route in the current workspace.",
                "verify_model",
                "Verify DeepSeek",
            ),
            "MODEL_QUOTA_COOLDOWN" => (
                "The selected model is in its known provider cooldown. Retry after the reset window.",
                "retry_model",
                "Retry after cooldown",
            ),
            "MODEL_KEY_INVALID" => (
                "Enter a valid printable DeepSeek API key, or cancel without saving.",
                "configure_model",
                "Configure again",
            ),
            "MODEL_KEYCHAIN_STORE_FAILED" => (
                "The DeepSeek key could not be saved to OS Keychain. No file fallback was used.",
                "configure_model",
                "Retry secure entry",
            ),
            "MODEL_KEYCHAIN_MISSING" => (
                "The DeepSeek OS Keychain key is unavailable. Configure it again.",
                "configure_model",
                "Configure DeepSeek",
            ),
            "MODEL_KEYCHAIN_DELETE_FAILED" => (
                "The DeepSeek key could not be removed from OS Keychain. Retry the cleanup.",
                "configure_model",
                "Retry cleanup",
            ),
            "MODEL_PLATFORM_UNSUPPORTED" => (
                "Secure model credentials are unavailable on this platform.",
                "open_desktop",
                "Open TradeX",
            ),
            "MODEL_ROUTE_INVALID" => (
                "That provider/model route is not allowed by the current contract.",
                "choose_model",
                "Choose a supported model",
            ),
            "MODEL_UNAVAILABLE" => (
                "The selected model route is unavailable. Retry its probe or choose another verified route.",
                "retry_model",
                "Retry model",
            ),
            "MODEL_OAUTH_EXPIRED" => (
                "ChatGPT authorization expired or was rejected. Re-login before verifying the route.",
                "login_chatgpt",
                "Re-login ChatGPT",
            ),
            "MODEL_QUOTA_EXCEEDED" => (
                "The model provider reported a quota limit. Wait for its known cooldown before retrying.",
                "retry_model",
                "Retry after cooldown",
            ),
            "MODEL_LOGIN_FAILED" => (
                "ChatGPT authorization did not complete. Retry login; no token was imported into TradeX.",
                "login_chatgpt",
                "Login ChatGPT",
            ),
            "MODEL_LOGIN_TIMEOUT" => (
                "ChatGPT authorization timed out. Retry login in the browser.",
                "login_chatgpt",
                "Login ChatGPT",
            ),
            "MODEL_TEST_INFERENCE_FAILED" => (
                "The selected route did not complete the bounded setup inference.",
                "retry_model",
                "Retry model",
            ),
            "MODEL_NATIVE_REQUIRED" => (
                "This model action is available only in the desktop app.",
                "open_desktop",
                "Open TradeX",
            ),
            "TURN_MESSAGE_INVALID" => (
                "Enter a non-empty request without control characters.",
                "retry_request",
                "Review request",
            ),
            "TURN_CONTEXT_INVALID" => (
                "This mode and execution context cannot be used together.",
                "choose_context",
                "Choose a supported context",
            ),
            "UNSUPPORTED_CAPABILITY" => (
                "The requested capability is not available for this Turn.",
                "choose_context",
                "Choose a supported capability",
            ),
            "RESEARCH_RESULT_INVALID" => (
                "The typed research result is missing or no longer matches this Turn.",
                "retry_request",
                "Refresh research result",
            ),
            "TURN_ACCOUNT_REQUIRED" => (
                "Select the account that belongs to this execution context.",
                "select_account",
                "Select account",
            ),
            "TURN_ACCOUNT_INVALID" => (
                "The selected account does not match this execution context.",
                "select_account",
                "Choose a matching account",
            ),
            "TURN_ALREADY_RUNNING" => (
                "This Thread already has a running Turn.",
                "wait_for_turn",
                "Wait for Turn",
            ),
            "TURN_NOT_RUNNING" => (
                "This Turn is no longer running and cannot be cancelled.",
                "reload_snapshot",
                "Reload Turn",
            ),
            "TURN_NOT_RETRYABLE" => (
                "Only failed or interrupted Turns can be retried.",
                "reload_snapshot",
                "Reload Turn",
            ),
            "CODEX_TURN_CANCELLED" => (
                "The Turn was cancelled before completion.",
                "retry_turn",
                "Retry Turn",
            ),
            "CODEX_RUNTIME_NOT_CONFIGURED" => (
                "Codex App Server is not configured for this workspace.",
                "configure_runtime",
                "Configure runtime",
            ),
            "CODEX_RUNTIME_START_FAILED" => (
                "Codex App Server could not be started. The request was preserved.",
                "retry_turn",
                "Retry Turn",
            ),
            "CODEX_RUNTIME_TIMEOUT" => (
                "Codex App Server timed out. The partial Turn was preserved.",
                "retry_turn",
                "Retry Turn",
            ),
            "CODEX_RUNTIME_BACKPRESSURE" => (
                "Codex App Server produced data faster than TradeX could persist it. The partial Turn was preserved.",
                "retry_turn",
                "Retry Turn",
            ),
            "CODEX_PROCESS_EXITED" => (
                "Codex App Server exited before the Turn completed. The partial Turn was preserved.",
                "retry_turn",
                "Retry Turn",
            ),
            "CODEX_FRAME_INVALID" => (
                "Codex App Server returned an invalid stream frame. The Turn was failed safely.",
                "retry_turn",
                "Retry Turn",
            ),
            "CODEX_PROTOCOL_UNSUPPORTED" => (
                "The installed Codex App Server protocol is incompatible with this build.",
                "check_version",
                "Check Codex version",
            ),
            "CODEX_EVENT_GAP" => (
                "The Codex stream was interrupted. Reload the Thread before retrying.",
                "reload_snapshot",
                "Reload Thread",
            ),
            "CODEX_UPSTREAM_ERROR" => (
                "Codex App Server reported a bounded runtime error. The Turn was preserved.",
                "retry_turn",
                "Retry Turn",
            ),
            "RISK_POLICY_INVALID" => (
                "Enter valid risk defaults within the stated decimal and time bounds.",
                "review_risk",
                "Review risk defaults",
            ),
            "RISK_POLICY_NOT_CONFIGURED" => (
                "Save the Risk Defaults step before continuing to Ready.",
                "configure_risk",
                "Configure risk defaults",
            ),
            "ONBOARDING_STEP_INVALID" => (
                "Complete the setup steps in order before continuing.",
                "review_onboarding",
                "Review setup steps",
            ),
            "ONBOARDING_BLOCKED" => (
                "Ready requires a verified default model route and saved risk defaults.",
                "review_onboarding",
                "Review setup",
            ),
            _ => (
                "The control plane could not complete this operation.",
                "reload_snapshot",
                "Reload state",
            ),
        };
        Self {
            category: if code.starts_with("GATEWAY_")
                || matches!(
                    code,
                    "MODEL_UNAVAILABLE"
                        | "MODEL_TEST_INFERENCE_FAILED"
                        | "MODEL_KEYCHAIN_MISSING"
                        | "MODEL_GATEWAY_RUNNING"
                        | "MODEL_DEFAULT_MISSING"
                        | "MODEL_FALLBACK_UNAVAILABLE"
                ) {
                "MODEL_UNAVAILABLE"
            } else if matches!(
                code,
                "MODEL_OAUTH_EXPIRED" | "MODEL_LOGIN_FAILED" | "MODEL_LOGIN_TIMEOUT"
            ) {
                "OAUTH_EXPIRED"
            } else if matches!(code, "MODEL_QUOTA_EXCEEDED" | "MODEL_QUOTA_COOLDOWN") {
                "QUOTA_EXCEEDED"
            } else if matches!(
                code,
                "CODEX_RUNTIME_NOT_CONFIGURED"
                    | "CODEX_RUNTIME_START_FAILED"
                    | "CODEX_RUNTIME_TIMEOUT"
                    | "CODEX_RUNTIME_BACKPRESSURE"
                    | "CODEX_PROCESS_EXITED"
                    | "CODEX_FRAME_INVALID"
                    | "CODEX_PROTOCOL_UNSUPPORTED"
                    | "CODEX_EVENT_GAP"
                    | "CODEX_UPSTREAM_ERROR"
                    | "CODEX_TURN_CANCELLED"
            ) {
                "RUNTIME_ERROR"
            } else if code.starts_with("MODEL_")
                || code == "PROVIDER_AUTH_FAILED"
                || code.starts_with("CREDENTIAL_")
            {
                "AUTH_ERROR"
            } else if code.starts_with("RISK_") || code.starts_with("ONBOARDING_") {
                "POLICY_ERROR"
            } else if code == "PROVIDER_RATE_LIMITED" {
                "RATE_LIMITED"
            } else if matches!(
                code,
                "PROVIDER_UNAVAILABLE" | "DATA_SOURCE_PROBE_FAILED" | "MARKET_HISTORY_UNAVAILABLE"
            ) {
                "NETWORK_ERROR"
            } else if matches!(
                code,
                "PROVIDER_UNSUPPORTED"
                    | "UNSUPPORTED_CAPABILITY"
                    | "RESEARCH_RESULT_INVALID"
                    | "DATA_SOURCE_UNKNOWN"
            ) {
                "UNSUPPORTED_CAPABILITY"
            } else if matches!(code, "MARKET_CLOSED" | "INSTRUMENT_HALTED") {
                code
            } else if code == "PROVIDER_PERMISSION_BLOCKED" {
                "PERMISSION_ERROR"
            } else if matches!(
                code,
                "IPC_AGGREGATE_NOT_FOUND"
                    | "WATCHLIST_NOT_FOUND"
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
            retryable: matches!(
                code,
                "WORKSPACE_BUSY"
                    | "WORKSPACE_OPEN_FAILED"
                    | "MARKET_HISTORY_UNAVAILABLE"
                    | "MARKET_CLOSED"
                    | "INSTRUMENT_HALTED"
                    | "CLOCK_SKEW"
                    | "MODEL_UNAVAILABLE"
                    | "MODEL_OAUTH_EXPIRED"
                    | "MODEL_QUOTA_EXCEEDED"
                    | "MODEL_LOGIN_FAILED"
                    | "MODEL_LOGIN_TIMEOUT"
                    | "MODEL_TEST_INFERENCE_FAILED"
                    | "CODEX_RUNTIME_START_FAILED"
                    | "CODEX_RUNTIME_TIMEOUT"
                    | "CODEX_RUNTIME_BACKPRESSURE"
                    | "CODEX_PROCESS_EXITED"
                    | "CODEX_FRAME_INVALID"
                    | "CODEX_UPSTREAM_ERROR"
                    | "CODEX_TURN_CANCELLED"
            ),
            blocking: true,
            remediation_actions: vec![Remediation {
                id: action.into(),
                label: label.into(),
            }],
        }
    }
}

pub type Result<T> = std::result::Result<T, TradeXError>;
