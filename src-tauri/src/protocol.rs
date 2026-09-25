use crate::capability::{CapabilityDecision, CapabilityQuery, ContextCatalog, ResearchToolId};
use crate::gateway::{GatewayMutation, GatewayState};
use crate::model::{
    ChatgptLogin, ConfigureDeepseek, ModelQuery, ModelState, SetDefaultModel, SetFallbackPolicy,
    VerifyRoute,
};
use crate::providers::*;
use crate::risk::{
    CompleteOnboarding, RiskDecision, RiskDecisionEvaluate, RiskDecisionHistory, RiskDecisionQuery,
    RiskPolicyState, RiskQuery, SaveRiskPolicy, SetOnboardingStep,
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
    #[schemars(extend("enum" = ["workspace", "account", "model-gateway", "model", "risk", "risk-decision", "thread", "trading212-demo-order-attempt", "trading212-demo-order-book", "alpaca-paper-order-attempt", "alpaca-paper-order-book", "binance-testnet-order-attempt", "binance-testnet-order-book", "bitget-demo-order-attempt"]))]
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
    Risk(Box<RiskPolicyState>),
    RiskDecision(Box<RiskDecision>),
    RiskDecisionHistory(RiskDecisionHistory),
    ProviderCatalog(ProviderCatalog),
    ProviderDefinition(ProviderDefinition),
    Accounts(Accounts),
    Account(Box<AccountConnection>),
    AccountDeletion(Box<AccountDeletionReceipt>),
    Permissions(PermissionReview),
    Capability(CapabilityDecision),
    ContextCatalog(ContextCatalog),
    DataSourceCatalog(DataSourceCatalog),
    MarketCatalog(MarketCatalog),
    MarketDetail(Box<MarketDetail>),
    Portfolio(Box<PortfolioSnapshot>),
    Paper(Box<LocalPaperState>),
    Watchlist(Box<Watchlist>),
    Watchlists(Watchlists),
    ResearchResult(ResearchToolResult),
    ScreenerResult(ScreenerResult),
    ScreenerLibrary(ScreenerLibrary),
    ScreenerAttachment(ScreenerAttachment),
    OrderDraft(Box<OrderDraft>),
    OrderDraftLibrary(OrderDraftLibrary),
    OrderProposal(Box<OrderProposal>),
    OrderProposalLibrary(OrderProposalLibrary),
    OrderProposalRefresh(Box<OrderProposalRefreshResult>),
    Trading212DemoOrderAttempt(Box<Trading212DemoOrderAttempt>),
    Trading212DemoOrderAttemptQuery(Box<Trading212DemoOrderAttemptQueryResult>),
    Trading212DemoOrderBook(Box<Trading212DemoOrderBook>),
    Trading212DemoOrderBookQuery(Box<Trading212DemoOrderBookQueryResult>),
    AlpacaPaperOrderAttempt(Box<AlpacaPaperOrderAttempt>),
    AlpacaPaperOrderAttemptQuery(Box<AlpacaPaperOrderAttemptQueryResult>),
    AlpacaPaperOrderBook(Box<AlpacaPaperOrderBook>),
    AlpacaPaperOrderBookQuery(Box<AlpacaPaperOrderBookQueryResult>),
    BinanceTestnetOrderAttempt(Box<BinanceTestnetOrderAttempt>),
    BinanceTestnetOrderAttemptQuery(Box<BinanceTestnetOrderAttemptQueryResult>),
    BitgetDemoOrderAttempt(Box<BitgetDemoOrderAttempt>),
    BitgetDemoOrderAttemptQuery(Box<BitgetDemoOrderAttemptQueryResult>),
    BinanceTestnetOrderBook(Box<BinanceTestnetOrderBook>),
    BinanceTestnetOrderBookQuery(Box<BinanceTestnetOrderBookQueryResult>),
    PaperOrderResult(Box<PaperOrderResult>),
    Artifact(Box<Artifact>),
    ArtifactLibrary(ArtifactLibrary),
    ArtifactExport(ArtifactExportResult),
    StrategyLibrary(StrategyLibrary),
    StrategyVersion(Box<StrategyVersion>),
    StrategyRun(Box<StrategyRun>),
    BacktestRun(Box<BacktestRun>),
    BacktestLibrary(BacktestLibrary),
    BacktestComparison(Box<BacktestComparison>),
}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountDeletionReceipt {
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
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
    pub risk_decision_evaluate: RiskDecisionEvaluate,
    pub risk_decision_query: RiskDecisionQuery,
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
    pub screener_request: ScreenerRequest,
    pub screener_save: ScreenerSave,
    pub screener_update: ScreenerUpdate,
    pub screener_attach: ScreenerAttach,
    pub order_draft_save: OrderDraftSave,
    pub order_draft_query: OrderDraftQuery,
    pub order_draft: OrderDraft,
    pub order_draft_library: OrderDraftLibrary,
    pub order_proposal_generate: OrderProposalGenerate,
    pub order_proposal_query: OrderProposalQuery,
    pub order_proposal: OrderProposal,
    pub order_proposal_library: OrderProposalLibrary,
    pub order_proposal_refresh_request: OrderProposalRefresh,
    pub order_proposal_refresh: OrderProposalRefreshResult,
    pub trading212_demo_order_attempt: Trading212DemoOrderAttempt,
    pub trading212_demo_order_submit: Trading212DemoOrderSubmit,
    pub trading212_demo_order_attempt_query: Trading212DemoOrderAttemptQuery,
    pub trading212_demo_order_attempt_query_result: Trading212DemoOrderAttemptQueryResult,
    pub trading212_demo_order_book_query: Trading212DemoOrderBookQuery,
    pub trading212_demo_order_book_query_result: Trading212DemoOrderBookQueryResult,
    pub trading212_demo_order_book_refresh: Trading212DemoOrderBookRefresh,
    pub trading212_demo_order_cancel: Trading212DemoOrderCancel,
    pub trading212_demo_order_book: Trading212DemoOrderBook,
    pub trading212_demo_order: Trading212DemoOrder,
    pub alpaca_paper_order_submit: AlpacaPaperOrderSubmit,
    pub alpaca_paper_order_attempt_query: AlpacaPaperOrderAttemptQuery,
    pub alpaca_paper_order_attempt_query_result: AlpacaPaperOrderAttemptQueryResult,
    pub alpaca_paper_order_reconcile: AlpacaPaperOrderReconcile,
    pub alpaca_paper_order_attempt: AlpacaPaperOrderAttempt,
    pub alpaca_paper_order_book_query: AlpacaPaperOrderBookQuery,
    pub alpaca_paper_order_book_query_result: AlpacaPaperOrderBookQueryResult,
    pub alpaca_paper_order_book_refresh: AlpacaPaperOrderBookRefresh,
    pub alpaca_paper_order_review: AlpacaPaperOrderReview,
    pub alpaca_paper_order_cancel: AlpacaPaperOrderCancel,
    pub alpaca_paper_order_book: AlpacaPaperOrderBook,
    pub alpaca_paper_order: AlpacaPaperOrder,
    pub alpaca_paper_fill: AlpacaPaperFill,
    pub binance_testnet_order_submit: BinanceTestnetOrderSubmit,
    pub binance_testnet_order_attempt_query: BinanceTestnetOrderAttemptQuery,
    pub binance_testnet_order_attempt_query_result: BinanceTestnetOrderAttemptQueryResult,
    pub binance_testnet_order_reconcile: BinanceTestnetOrderReconcile,
    pub binance_testnet_order_attempt: BinanceTestnetOrderAttempt,
    pub bitget_demo_order_submit: BitgetDemoOrderSubmit,
    pub bitget_demo_order_attempt_query: BitgetDemoOrderAttemptQuery,
    pub bitget_demo_order_attempt_query_result: BitgetDemoOrderAttemptQueryResult,
    pub bitget_demo_order_reconcile: BitgetDemoOrderReconcile,
    pub bitget_demo_order_attempt: BitgetDemoOrderAttempt,
    pub binance_testnet_order_book_query: BinanceTestnetOrderBookQuery,
    pub binance_testnet_order_book_query_result: BinanceTestnetOrderBookQueryResult,
    pub binance_testnet_order_book_refresh: BinanceTestnetOrderBookRefresh,
    pub binance_testnet_order_cancel: BinanceTestnetOrderCancel,
    pub binance_testnet_order_book: BinanceTestnetOrderBook,
    pub binance_testnet_order: BinanceTestnetOrder,
    pub binance_testnet_fill: BinanceTestnetFill,
    pub binance_testnet_balance: BinanceTestnetBalance,
    pub binance_testnet_history_state: BinanceTestnetHistoryState,
    pub binance_testnet_order_book_rate_limits: BinanceTestnetOrderBookRateLimits,
    pub artifact_save: ArtifactSave,
    pub artifact_query: ArtifactQuery,
    pub artifact_export: ArtifactExport,
    pub portfolio_query: PortfolioQuery,
    pub local_paper_state: LocalPaperState,
    pub paper_order_submit: PaperOrderSubmit,
    pub paper_order_cancel: PaperOrderCancel,
    pub paper_quote_refresh: PaperQuoteRefresh,
    pub paper_scenario_set: PaperScenarioSet,
    pub paper_order_result: PaperOrderResult,
    pub strategy_definition: StrategyDefinition,
    pub strategy_version: StrategyVersion,
    pub strategy_save: StrategySave,
    pub strategy_query: StrategyQuery,
    pub strategy_run_query: StrategyRunQuery,
    pub strategy_run_request: StrategyRunRequest,
    pub strategy_cancel: StrategyCancel,
    pub strategy_library: StrategyLibrary,
    pub strategy_run: StrategyRun,
    pub strategy_signal: StrategySignal,
    pub strategy_failure: StrategyFailure,
    pub backtest_failure: BacktestFailure,
    pub backtest_guard_check: BacktestGuardCheck,
    pub backtest_manifest: BacktestManifest,
    pub backtest_metrics: BacktestMetrics,
    pub equity_point: EquityPoint,
    pub trade_record: TradeRecord,
    pub backtest_result: BacktestResult,
    pub backtest_run: BacktestRun,
    pub backtest_run_summary: BacktestRunSummary,
    pub backtest_library: BacktestLibrary,
    pub backtest_compare_request: BacktestCompareRequest,
    pub backtest_field_difference: BacktestFieldDifference,
    pub backtest_metric_delta: BacktestMetricDelta,
    pub backtest_metric_comparison: BacktestMetricComparison,
    pub backtest_curve_summary: BacktestCurveSummary,
    pub backtest_comparison: BacktestComparison,
    pub backtest_run_query: BacktestRunQuery,
    pub backtest_run_request: BacktestRunRequest,
    pub backtest_cancel: BacktestCancel,
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
    #[schemars(range(min = 1, max = 23))]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(length(max = 8), inner(length(min = 1, max = 128)))]
    pub market_snapshot_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(length(max = 8), inner(length(min = 1, max = 128)))]
    pub dataset_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(length(max = 8), inner(length(min = 1, max = 128)))]
    pub order_refs: Vec<String>,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerOperation {
    Parse,
    Run,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerResultState {
    Parsed,
    Running,
    Empty,
    Completed,
    BlockedExternal,
    Failed,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerUniverse {
    UsEquities,
    UsLargeCapTechnology,
    CryptoSpot,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerPredicateField {
    RevenueGrowth,
    EstimateRevision,
    Rsi,
    PriceChange,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerOperator {
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerPredicate {
    pub field: ScreenerPredicateField,
    pub operator: ScreenerOperator,
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub threshold: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilterSpec {
    pub universe: ScreenerUniverse,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub predicates: Vec<ScreenerPredicate>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerRankField {
    Quality,
    RevisionStrength,
    Momentum,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RankSpec {
    pub field: ScreenerRankField,
    pub direction: ScreenerDirection,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenerFeatureField {
    RevenueGrowth,
    EstimateRevision,
    Rsi,
    PriceChange,
    Quality,
    RevisionStrength,
    Momentum,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerFeature {
    pub field: ScreenerFeatureField,
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerProvenance {
    #[schemars(length(min = 1, max = 32))]
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub provider_timestamp: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub received_timestamp: String,
    pub freshness: ResearchFreshness,
    pub quality: ResearchQuality,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerCandidate {
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub symbol: String,
    #[schemars(range(min = 1, max = 50))]
    pub rank: u32,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub features: Vec<ScreenerFeature>,
    pub provenance: ScreenerProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 512))]
    pub limitation: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerRequest {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub operation: ScreenerOperation,
    #[schemars(length(min = 1, max = 4_000))]
    pub natural_language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<ResearchFocus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_spec: Option<FilterSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_spec: Option<RankSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 80))]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerResult {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub operation: ScreenerOperation,
    pub state: ScreenerResultState,
    #[schemars(length(min = 1, max = 4_000))]
    pub natural_language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<ResearchFocus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_spec: Option<FilterSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_spec: Option<RankSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 80))]
    pub revision: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 16), inner(length(min = 1, max = 256)))]
    pub applied_conditions: Vec<String>,
    #[serde(default)]
    #[schemars(length(max = 50))]
    pub candidates: Vec<ScreenerCandidate>,
    #[schemars(range(min = 0, max = 50))]
    pub candidate_count: u32,
    #[serde(default)]
    #[schemars(length(max = 8), inner(length(min = 1, max = 32)))]
    pub source_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub provider_timestamp: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub received_timestamp: String,
    #[schemars(length(min = 1, max = 512))]
    pub availability_reason: String,
    #[serde(default)]
    #[schemars(length(max = 8), inner(length(min = 1, max = 512)))]
    pub limitations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub fixture_label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerDefinition {
    #[schemars(length(min = 1, max = 4_000))]
    pub natural_language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<ResearchFocus>,
    pub filter_spec: FilterSpec,
    pub rank_spec: RankSpec,
    #[schemars(with = "String", length(min = 1, max = 80))]
    pub revision: String,
    #[schemars(range(min = 1, max = 50))]
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedScreener {
    #[schemars(length(min = 1, max = 128))]
    pub screener_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    pub definition: ScreenerDefinition,
    pub state: ScreenerResultState,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerLibrary {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 128))]
    pub screeners: Vec<SavedScreener>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerSave {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    pub definition: ScreenerDefinition,
    pub state: ScreenerResultState,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerUpdate {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub screener_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    pub definition: ScreenerDefinition,
    pub state: ScreenerResultState,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerAttach {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(with = "String", length(min = 1, max = 80))]
    pub revision: String,
    #[schemars(length(min = 1, max = 32), inner(length(min = 1, max = 128)))]
    pub selected_instrument_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenerAttachment {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(with = "String", length(min = 1, max = 80))]
    pub revision: String,
    #[schemars(length(max = 32))]
    pub context_refs: Vec<ThreadContextRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrategyRunState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrategyDirection {
    Buy,
    Sell,
    Hold,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrategyFixtureScenario {
    Success,
    Failure,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyParameter {
    #[schemars(length(min = 1, max = 64))]
    pub name: String,
    #[schemars(length(max = 256))]
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyDefinition {
    #[schemars(length(min = 1, max = 120))]
    pub name: String,
    #[schemars(length(min = 1, max = 100_000))]
    pub source: String,
    #[schemars(length(min = 1, max = 32))]
    pub language: String,
    #[schemars(length(min = 1, max = 64))]
    pub runtime: String,
    #[serde(default)]
    #[schemars(length(max = 32))]
    pub parameters: Vec<StrategyParameter>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyVersion {
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub revision: u64,
    pub definition: StrategyDefinition,
    #[schemars(length(min = 1, max = 80))]
    pub source_hash: String,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategySignal {
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub direction: StrategyDirection,
    #[schemars(length(min = 1, max = 64))]
    pub desired_exposure: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub source_ref: String,
    #[schemars(length(min = 1, max = 80))]
    pub strategy_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub dataset_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyFailure {
    #[schemars(length(min = 1, max = 64))]
    pub code: String,
    #[schemars(length(min = 1, max = 512))]
    pub reason: String,
    #[serde(default)]
    #[schemars(length(max = 4), inner(length(min = 1, max = 256)))]
    pub remediation: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyRun {
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub strategy_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub dataset_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub start_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub end_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[serde(default)]
    #[schemars(length(max = 32))]
    pub parameters: Vec<StrategyParameter>,
    pub state: StrategyRunState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<StrategySignal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<StrategyFailure>,
    #[schemars(length(min = 1, max = 80))]
    pub request_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub fixture_label: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyRunSummary {
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    pub state: StrategyRunState,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyLibrary {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 256))]
    pub versions: Vec<StrategyVersion>,
    #[schemars(length(max = 256))]
    pub runs: Vec<StrategyRunSummary>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategySave {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub strategy_id: Option<String>,
    pub definition: StrategyDefinition,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyRunQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyRunRequest {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 80))]
    pub expected_strategy_hash: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub dataset_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub start_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub end_at: String,
    #[serde(default)]
    #[schemars(length(max = 32))]
    pub parameters: Vec<StrategyParameter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_scenario: Option<StrategyFixtureScenario>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrategyCancel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BacktestRunState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BacktestFixtureScenario {
    Success,
    Failure,
    Cancelled,
    Lookahead,
    Survivorship,
    Split,
    Dividend,
    Timezone,
    DataGap,
    DatasetHashMismatch,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestFailure {
    #[schemars(length(min = 1, max = 64))]
    pub code: String,
    #[schemars(length(min = 1, max = 512))]
    pub reason: String,
    #[serde(default)]
    #[schemars(length(max = 4), inner(length(min = 1, max = 256)))]
    pub remediation: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BacktestGuardState {
    Passed,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestGuardCheck {
    #[schemars(length(min = 1, max = 64))]
    pub name: String,
    pub state: BacktestGuardState,
    #[schemars(length(min = 1, max = 256))]
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestManifest {
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version: String,
    #[schemars(length(min = 71, max = 71))]
    pub strategy_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub dataset_id: String,
    #[schemars(length(min = 71, max = 71))]
    pub dataset_hash: String,
    #[schemars(length(min = 1, max = 160))]
    pub data_provider: String,
    #[schemars(length(min = 1, max = 64))]
    pub retrieved_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub start_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub end_at: String,
    #[schemars(length(min = 1, max = 128))]
    pub adjustment_method: String,
    #[schemars(length(min = 1, max = 64))]
    pub timezone: String,
    #[schemars(length(min = 1, max = 128))]
    pub market_calendar_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub commission_model: String,
    #[schemars(length(min = 1, max = 128))]
    pub commission: String,
    #[schemars(length(min = 1, max = 128))]
    pub slippage_model: String,
    #[schemars(length(min = 1, max = 128))]
    pub slippage: String,
    #[schemars(length(min = 1, max = 128))]
    pub starting_cash: String,
    #[schemars(length(min = 1, max = 128))]
    pub seed: String,
    #[schemars(length(max = 32), inner())]
    pub parameters: Vec<StrategyParameter>,
    #[schemars(length(min = 1, max = 128))]
    pub engine_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub runtime_version: String,
    #[schemars(length(min = 6, max = 6), inner(length(min = 1, max = 256)))]
    pub guard_checks: Vec<BacktestGuardCheck>,
    #[schemars(length(min = 71, max = 71))]
    pub manifest_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestMetrics {
    #[serde(rename = "return")]
    #[schemars(length(min = 1, max = 128))]
    pub return_pct: String,
    #[schemars(length(min = 1, max = 128))]
    pub sharpe: String,
    #[schemars(length(min = 1, max = 128))]
    pub sortino: String,
    #[schemars(length(min = 1, max = 128))]
    pub max_drawdown: String,
    #[schemars(length(min = 1, max = 128))]
    pub win_rate: String,
    #[schemars(length(min = 1, max = 128))]
    pub profit_factor: String,
    #[schemars(length(min = 1, max = 128))]
    pub turnover: String,
    #[schemars(range(min = 0, max = 100_000))]
    pub trade_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EquityPoint {
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[schemars(length(min = 1, max = 128))]
    pub equity: String,
    #[schemars(length(min = 1, max = 128))]
    pub drawdown: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TradeRecord {
    #[schemars(length(min = 1, max = 128))]
    pub trade_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub side: String,
    #[schemars(length(min = 1, max = 128))]
    pub quantity: String,
    #[schemars(length(min = 1, max = 128))]
    pub price: String,
    #[schemars(length(min = 1, max = 128))]
    pub gross_value: String,
    #[schemars(length(min = 1, max = 128))]
    pub commission: String,
    #[schemars(length(min = 1, max = 128))]
    pub slippage: String,
    #[schemars(length(min = 1, max = 128))]
    pub realized_pnl: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestResult {
    pub metrics: BacktestMetrics,
    #[schemars(length(min = 1, max = 5_000), inner())]
    pub equity_curve: Vec<EquityPoint>,
    #[schemars(length(max = 5_000), inner())]
    pub trades: Vec<TradeRecord>,
    pub manifest: BacktestManifest,
    pub historical_simulation: bool,
    #[schemars(length(min = 1, max = 8), inner(length(min = 1, max = 256)))]
    pub limitations: Vec<String>,
    #[schemars(length(min = 71, max = 71))]
    pub result_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestRun {
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub strategy_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub dataset_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub start_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub end_at: String,
    #[schemars(length(min = 1, max = 32))]
    pub bar_interval: String,
    #[schemars(length(min = 1, max = 128))]
    pub starting_cash: String,
    #[schemars(length(min = 1, max = 128))]
    pub commission: String,
    #[schemars(length(min = 1, max = 128))]
    pub slippage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub portfolio_seed: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 32))]
    pub parameters: Vec<StrategyParameter>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub state: BacktestRunState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<BacktestFailure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<BacktestResult>,
    #[schemars(length(min = 1, max = 80))]
    pub request_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub fixture_label: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestRunSummary {
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    #[schemars(length(min = 1, max = 80))]
    pub strategy_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub dataset_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub start_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub end_at: String,
    #[schemars(length(min = 1, max = 32))]
    pub bar_interval: String,
    pub state: BacktestRunState,
    #[schemars(length(min = 1, max = 80))]
    pub request_hash: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 71, max = 71))]
    pub result_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0, max = 100_000))]
    pub trade_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestLibrary {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 256))]
    pub runs: Vec<BacktestRunSummary>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestCompareRequest {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub left_run_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub right_run_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestFieldDifference {
    #[schemars(length(min = 1, max = 128))]
    pub field: String,
    #[schemars(length(min = 1, max = 16384))]
    pub left: String,
    #[schemars(length(min = 1, max = 16384))]
    pub right: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestMetricDelta {
    #[schemars(length(min = 1, max = 128))]
    pub left: String,
    #[schemars(length(min = 1, max = 128))]
    pub right: String,
    #[schemars(length(min = 1, max = 128))]
    pub difference: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestMetricComparison {
    #[serde(rename = "return")]
    pub return_pct: BacktestMetricDelta,
    pub sharpe: BacktestMetricDelta,
    pub sortino: BacktestMetricDelta,
    pub max_drawdown: BacktestMetricDelta,
    pub win_rate: BacktestMetricDelta,
    pub profit_factor: BacktestMetricDelta,
    pub turnover: BacktestMetricDelta,
    pub trade_count: BacktestMetricDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestCurveSummary {
    #[schemars(range(min = 1, max = 5_000))]
    pub point_count: u32,
    #[schemars(length(min = 1, max = 64))]
    pub start_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub end_at: String,
    #[schemars(length(min = 1, max = 128))]
    pub start_equity: String,
    #[schemars(length(min = 1, max = 128))]
    pub end_equity: String,
    #[schemars(length(min = 1, max = 128))]
    pub max_drawdown: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestComparison {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub left: Box<BacktestRun>,
    pub right: Box<BacktestRun>,
    pub left_curve: BacktestCurveSummary,
    pub right_curve: BacktestCurveSummary,
    #[schemars(length(max = 64))]
    pub input_differences: Vec<BacktestFieldDifference>,
    #[schemars(length(max = 64))]
    pub manifest_differences: Vec<BacktestFieldDifference>,
    pub metrics: BacktestMetricComparison,
    pub historical_simulation: bool,
    #[schemars(length(min = 1, max = 8), inner(length(min = 1, max = 256)))]
    pub limitations: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestRunRequest {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub strategy_version_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 80))]
    pub expected_strategy_hash: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub dataset_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub start_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub end_at: String,
    #[schemars(length(min = 1, max = 32))]
    pub bar_interval: String,
    #[schemars(length(min = 1, max = 128))]
    pub starting_cash: String,
    #[schemars(length(min = 1, max = 128))]
    pub commission: String,
    #[schemars(length(min = 1, max = 128))]
    pub slippage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub portfolio_seed: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 32))]
    pub parameters: Vec<StrategyParameter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_scenario: Option<BacktestFixtureScenario>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestRunQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacktestCancel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub run_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderQuantityType {
    Base,
    Quote,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    Day,
    Gtc,
    Ioc,
    Fok,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderQuantity {
    pub r#type: OrderQuantityType,
    #[schemars(length(min = 1, max = 128))]
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderDraftFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[schemars(length(min = 1, max = 32))]
    pub venue: String,
    pub environment: ExecutionContext,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: OrderQuantity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub limit_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub maximum_spend: Option<String>,
    pub time_in_force: TimeInForce,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 80))]
    pub client_label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderDraft {
    #[schemars(length(min = 1, max = 128))]
    pub draft_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub draft_version: u64,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    pub fields: OrderDraftFields,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderDraftSummary {
    #[schemars(length(min = 1, max = 128))]
    pub draft_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub draft_version: u64,
    #[schemars(length(min = 1, max = 64))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub environment: ExecutionContext,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderDraftLibrary {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 256))]
    pub drafts: Vec<OrderDraftSummary>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderDraftSave {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub draft_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: Option<String>,
    pub fields: OrderDraftFields,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderDraftQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub draft_id: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderProposalStatus {
    NeedsApproval,
    Consumed,
    Invalidated,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalReferenceStatus {
    Available,
    Unconfigured,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderProposalHistoryEvent {
    Generated,
    DraftChanged,
    Refreshed,
    PolicyChanged,
    Consumed,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderProposalRefreshStatus {
    Refreshed,
    Stale,
    Blocked,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposalHistoryEntry {
    pub event: OrderProposalHistoryEvent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub reason: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub occurred_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposal {
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub draft_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub draft_version: u64,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    pub fields: OrderDraftFields,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub estimated_notional: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 16))]
    pub estimated_notional_currency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub estimated_notional_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1))]
    pub policy_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub policy_state_version: Option<String>,
    pub policy_status: ProposalReferenceStatus,
    #[schemars(length(min = 1, max = 256))]
    pub policy_reference_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub market_snapshot_id: Option<String>,
    pub market_status: MarketDataStatus,
    #[schemars(length(min = 1, max = 256))]
    pub market_reference_reason: String,
    pub status: OrderProposalStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub invalidation_reason: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 32))]
    pub history: Vec<OrderProposalHistoryEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposalSummary {
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub draft_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub draft_version: u64,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    pub status: OrderProposalStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub invalidation_reason: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposalLibrary {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 256))]
    pub proposals: Vec<OrderProposalSummary>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposalGenerate {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub draft_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub expected_draft_version: u64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposalQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposalRefresh {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderProposalRefreshResult {
    pub previous_proposal: Box<OrderProposal>,
    pub proposal: Box<OrderProposal>,
    pub refresh_status: OrderProposalRefreshStatus,
    #[schemars(length(min = 1, max = 256))]
    pub invalidation_reason: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Trading212DemoOrderAttemptState {
    Submitting,
    Acknowledged,
    UnknownReconciling,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderAttempt {
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 20))]
    pub remote_account_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    pub state: Trading212DemoOrderAttemptState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 20))]
    pub provider_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 32))]
    pub provider_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub error_code: Option<String>,
    #[schemars(length(min = 1, max = 256))]
    pub reason: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderSubmit {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_proposal_state_version: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
    pub confirmed_demo_order: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderAttemptQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderAttemptQueryResult {
    pub attempt: Option<Trading212DemoOrderAttempt>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Trading212DemoOrderBookStatus {
    NeverSynced,
    Current,
    Degraded,
    Stale,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Trading212DemoOrderOrigin {
    TradeX,
    External,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Trading212DemoNormalizedOrderStatus {
    Local,
    Pending,
    Open,
    CancelPending,
    Cancelled,
    PartiallyFilled,
    Filled,
    Rejected,
    Replacing,
    Replaced,
    Expired,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Trading212DemoCancelState {
    #[default]
    None,
    Submitting,
    Pending,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Trading212DemoOrderBookAction {
    Pending,
    History,
    Detail,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoRateLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub pending_orders_retry_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub order_detail_retry_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub history_retry_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub cancel_order_retry_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrder {
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub symbol: String,
    #[schemars(length(min = 1, max = 8))]
    pub side: String,
    #[schemars(length(min = 1, max = 32))]
    pub order_type: String,
    #[schemars(length(min = 1, max = 32))]
    pub time_in_force: String,
    #[schemars(length(min = 1, max = 32))]
    pub provider_status: String,
    pub normalized_status: Trading212DemoNormalizedOrderStatus,
    pub pending: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub quantity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub filled_quantity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub filled_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub currency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub remaining_quantity: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub submitted_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub provider_updated_at: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub origin: Trading212DemoOrderOrigin,
    #[serde(default)]
    pub cancel_state: Trading212DemoCancelState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub cancel_idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub cancel_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderBook {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub remote_account_id: String,
    #[schemars(extend("const" = "DEMO"))]
    pub environment: String,
    pub status: Trading212DemoOrderBookStatus,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub last_successful_sync_at: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub reason: Option<String>,
    pub history_started: bool,
    pub history_complete: bool,
    #[schemars(range(max = 100))]
    pub history_page_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 512))]
    pub next_page_path: Option<String>,
    #[schemars(length(max = 100))]
    pub history_cursors: Vec<String>,
    pub rate_limits: Trading212DemoRateLimits,
    #[schemars(length(max = 5000))]
    pub orders: Vec<Trading212DemoOrder>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderBookQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderBookQueryResult {
    pub book: Option<Trading212DemoOrderBook>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderBookRefresh {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    pub action: Trading212DemoOrderBookAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub provider_order_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trading212DemoOrderCancel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_book_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
    pub confirmed: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlpacaPaperOrderAttemptState {
    Submitting,
    Acknowledged,
    UnknownReconciling,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderAttempt {
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub remote_account_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub client_order_id: String,
    pub state: AlpacaPaperOrderAttemptState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 36))]
    pub provider_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub provider_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub error_code: Option<String>,
    #[schemars(length(min = 1, max = 256))]
    pub reason: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderSubmit {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_proposal_state_version: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
    pub confirmed_paper_order: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderAttemptQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderAttemptQueryResult {
    pub attempt: Option<AlpacaPaperOrderAttempt>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceTestnetOrderAttemptState {
    Submitting,
    Acknowledged,
    UnknownReconciling,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderAttempt {
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub remote_account_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(extend("const" = "TESTNET"))]
    pub environment: String,
    #[schemars(length(min = 1, max = 36))]
    pub client_order_id: String,
    pub state: BinanceTestnetOrderAttemptState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 20))]
    pub provider_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub provider_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub error_code: Option<String>,
    #[schemars(length(min = 1, max = 256))]
    pub reason: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderSubmit {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_proposal_state_version: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
    pub confirmed_testnet_order: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderAttemptQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderAttemptQueryResult {
    pub attempt: Option<BinanceTestnetOrderAttempt>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderReconcile {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BitgetDemoOrderAttemptState {
    Submitting,
    Acknowledged,
    UnknownReconciling,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BitgetDemoOrderAttempt {
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub remote_account_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(extend("const" = "DEMO"))]
    pub environment: String,
    #[schemars(length(min = 1, max = 50))]
    pub client_oid: String,
    pub state: BitgetDemoOrderAttemptState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 40))]
    pub provider_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub provider_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub error_code: Option<String>,
    #[schemars(length(min = 1, max = 256))]
    pub reason: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BitgetDemoOrderSubmit {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_proposal_state_version: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
    pub confirmed_demo_order: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BitgetDemoOrderAttemptQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BitgetDemoOrderAttemptQueryResult {
    pub attempt: Option<BitgetDemoOrderAttempt>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BitgetDemoOrderReconcile {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceTestnetOrderBookStatus {
    NeverSynced,
    Current,
    Degraded,
    Stale,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceTestnetOrderOrigin {
    TradeX,
    External,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceTestnetOrderCancelState {
    #[default]
    None,
    Submitting,
    Pending,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceTestnetOrderBookAction {
    Pending,
    Account,
    History,
    Detail,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrder {
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub symbol: String,
    #[schemars(length(min = 1, max = 36))]
    pub client_order_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub side: String,
    #[schemars(length(min = 1, max = 32))]
    pub order_type: String,
    #[schemars(length(min = 1, max = 32))]
    pub time_in_force: String,
    #[schemars(length(min = 1, max = 64))]
    pub provider_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub quantity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub quote_quantity: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub filled_quantity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub filled_quote_quantity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub remaining_quantity: Option<String>,
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub submitted_at_ms: u64,
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub provider_updated_at_ms: u64,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub pending: bool,
    pub origin: BinanceTestnetOrderOrigin,
    #[serde(default)]
    pub cancel_state: BinanceTestnetOrderCancelState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 36))]
    pub cancel_idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub cancel_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub attempt_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetFill {
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub trade_id: String,
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub symbol: String,
    #[schemars(length(min = 1, max = 16))]
    pub side: String,
    #[schemars(length(min = 1, max = 64))]
    pub price: String,
    #[schemars(length(min = 1, max = 64))]
    pub quantity: String,
    #[schemars(length(min = 1, max = 64))]
    pub quote_quantity: String,
    #[schemars(length(min = 1, max = 64))]
    pub commission: String,
    #[schemars(length(min = 1, max = 32))]
    pub commission_asset: String,
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub executed_at_ms: u64,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetBalance {
    #[schemars(length(min = 1, max = 32))]
    pub asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub free: String,
    #[schemars(length(min = 1, max = 64))]
    pub locked: String,
    #[schemars(length(min = 1, max = 64))]
    pub total: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetHistoryState {
    #[schemars(regex(pattern = "^(BTCUSDT|ETHUSDT)$"))]
    pub symbol: String,
    pub started: bool,
    pub complete: bool,
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub page_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub last_observed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub next_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub next_trade_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderBookRateLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub pending_orders_retry_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub account_retry_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub history_retry_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub order_detail_retry_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderBook {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub remote_account_id: String,
    #[schemars(extend("const" = "TESTNET"))]
    pub environment: String,
    pub status: BinanceTestnetOrderBookStatus,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub last_successful_sync_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub pending_orders_observed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub balances_observed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub private_stream_balance_update_at_ms: Option<u64>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub reason: Option<String>,
    pub rate_limits: BinanceTestnetOrderBookRateLimits,
    #[schemars(length(max = 2))]
    pub history: Vec<BinanceTestnetHistoryState>,
    #[schemars(length(max = 5000))]
    pub orders: Vec<BinanceTestnetOrder>,
    #[schemars(length(max = 5000))]
    pub fills: Vec<BinanceTestnetFill>,
    #[schemars(length(max = 5000))]
    pub balances: Vec<BinanceTestnetBalance>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderBookQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderBookQueryResult {
    pub book: Option<BinanceTestnetOrderBook>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderBookRefresh {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    pub action: BinanceTestnetOrderBookAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^[A-Z0-9]{1,32}$"))]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub provider_order_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BinanceTestnetOrderCancel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(regex(pattern = "^(BTCUSDT|ETHUSDT)$"))]
    pub symbol: String,
    #[schemars(regex(pattern = "^[0-9]{1,20}$"))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_book_state_version: String,
    #[schemars(regex(pattern = "^[A-Za-z0-9_-]{36}$"))]
    pub idempotency_key: String,
    pub confirmed: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderReconcile {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlpacaPaperOrderBookStatus {
    NeverSynced,
    Current,
    Degraded,
    Stale,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlpacaPaperOrderOrigin {
    TradeX,
    External,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlpacaPaperCancelState {
    None,
    Submitting,
    Pending,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrder {
    #[schemars(length(min = 36, max = 36))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub client_order_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: Option<String>,
    #[schemars(length(min = 1, max = 16))]
    pub side: String,
    #[schemars(length(min = 1, max = 32))]
    pub order_type: String,
    #[schemars(length(min = 1, max = 32))]
    pub time_in_force: String,
    #[schemars(length(min = 1, max = 64))]
    pub provider_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub quantity: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub filled_quantity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub remaining_quantity: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub submitted_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub provider_updated_at: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    pub origin: AlpacaPaperOrderOrigin,
    pub cancel_state: AlpacaPaperCancelState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub cancel_idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub cancel_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperFill {
    #[schemars(length(min = 1, max = 128))]
    pub activity_id: String,
    #[schemars(length(min = 36, max = 36))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: Option<String>,
    #[schemars(length(min = 1, max = 16))]
    pub side: String,
    #[schemars(length(min = 1, max = 64))]
    pub quantity: String,
    #[schemars(length(min = 1, max = 64))]
    pub price: String,
    pub source: AlpacaPaperFillSource,
    #[schemars(length(min = 1, max = 64))]
    pub executed_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlpacaPaperFillSource {
    RestActivity,
    TradeUpdate,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderBook {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub remote_account_id: String,
    pub status: AlpacaPaperOrderBookStatus,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub last_successful_sync_at: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 256))]
    pub reason: Option<String>,
    #[schemars(length(max = 500))]
    pub orders: Vec<AlpacaPaperOrder>,
    #[schemars(length(max = 1000))]
    pub fills: Vec<AlpacaPaperFill>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderBookQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderBookQueryResult {
    pub book: Option<AlpacaPaperOrderBook>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderBookRefresh {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderReview {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 36, max = 36))]
    pub provider_order_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlpacaPaperOrderCancel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_connection_state_version: String,
    #[schemars(length(min = 36, max = 36))]
    pub provider_order_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_book_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
    pub confirmed: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactKind {
    Research,
    Decision,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactContent {
    #[schemars(length(min = 1, max = 32_768))]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_result: Option<ResearchToolResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactProvenance {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub turn_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub item_id: String,
    pub turn_snapshot: TurnSnapshot,
    #[schemars(length(max = 16))]
    pub provider_attempts: Vec<ThreadProviderAttempt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_tool_id: Option<ResearchToolId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub research_result_id: Option<String>,
    #[schemars(length(max = 16))]
    pub sources: Vec<ResearchProvenance>,
    #[schemars(length(max = 16), inner(length(min = 1, max = 128)))]
    pub market_snapshot_hashes: Vec<String>,
    #[schemars(length(max = 16), inner(length(min = 1, max = 128)))]
    pub dataset_hashes: Vec<String>,
    #[schemars(length(max = 16), inner(length(min = 1, max = 128)))]
    pub related_order_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Artifact {
    #[schemars(length(min = 1, max = 128))]
    pub artifact_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub kind: ArtifactKind,
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    #[schemars(range(min = 1, max = 100))]
    pub version: u32,
    #[schemars(length(min = 1, max = 80))]
    pub content_hash: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    pub content: ArtifactContent,
    pub provenance: ArtifactProvenance,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactSummary {
    #[schemars(length(min = 1, max = 128))]
    pub artifact_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub kind: ArtifactKind,
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
    #[schemars(length(min = 1, max = 80))]
    pub content_hash: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub turn_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub item_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactLibrary {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(max = 256))]
    pub artifacts: Vec<ArtifactSummary>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactSave {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub thread_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub turn_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub item_id: String,
    pub kind: ArtifactKind,
    #[schemars(length(min = 1, max = 120))]
    pub title: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub artifact_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactExport {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub artifact_id: String,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub file_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", length(min = 1, max = 4096))]
    pub destination_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactExportResult {
    #[schemars(length(min = 1, max = 128))]
    pub artifact_id: String,
    #[schemars(length(min = 1, max = 4096))]
    pub path: String,
    #[schemars(length(min = 1, max = 80))]
    pub content_hash: String,
    #[schemars(length(min = 1, max = 80))]
    pub manifest_hash: String,
    #[schemars(range(min = 1, max = 10_000_000))]
    pub bytes: u64,
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
    Risk(Box<RiskPolicyState>),
    RiskDecision(Box<RiskDecision>),
    Thread(Box<Thread>),
    Trading212DemoOrderAttempt(Box<Trading212DemoOrderAttempt>),
    AlpacaPaperOrderAttempt(Box<AlpacaPaperOrderAttempt>),
    AlpacaPaperOrderBook(Box<AlpacaPaperOrderBook>),
    BinanceTestnetOrderAttempt(Box<BinanceTestnetOrderAttempt>),
    BitgetDemoOrderAttempt(Box<BitgetDemoOrderAttempt>),
    Trading212DemoOrderBook(Box<Trading212DemoOrderBook>),
    BinanceTestnetOrderBook(Box<BinanceTestnetOrderBook>),
}

impl DomainProjection {
    pub fn id(&self) -> &str {
        match self {
            Self::Gateway(g) => &g.workspace_id,
            Self::Model(m) => &m.workspace_id,
            Self::Workspace(w) => &w.workspace_id,
            Self::Account(a) => &a.connection_id,
            Self::Risk(r) => &r.workspace_id,
            Self::RiskDecision(d) => &d.proposal_id,
            Self::Thread(t) => &t.thread_id,
            Self::Trading212DemoOrderAttempt(a) => &a.attempt_id,
            Self::AlpacaPaperOrderAttempt(a) => &a.attempt_id,
            Self::AlpacaPaperOrderBook(b) => &b.connection_id,
            Self::BinanceTestnetOrderAttempt(a) => &a.attempt_id,
            Self::BitgetDemoOrderAttempt(a) => &a.attempt_id,
            Self::Trading212DemoOrderBook(b) => &b.connection_id,
            Self::BinanceTestnetOrderBook(b) => &b.connection_id,
        }
    }
    pub fn kind(&self) -> &str {
        match self {
            Self::Gateway(_) => "model-gateway",
            Self::Model(_) => "model",
            Self::Workspace(_) => "workspace",
            Self::Account(_) => "account",
            Self::Risk(_) => "risk",
            Self::RiskDecision(_) => "risk-decision",
            Self::Thread(_) => "thread",
            Self::Trading212DemoOrderAttempt(_) => "trading212-demo-order-attempt",
            Self::AlpacaPaperOrderAttempt(_) => "alpaca-paper-order-attempt",
            Self::AlpacaPaperOrderBook(_) => "alpaca-paper-order-book",
            Self::BinanceTestnetOrderAttempt(_) => "binance-testnet-order-attempt",
            Self::BitgetDemoOrderAttempt(_) => "bitget-demo-order-attempt",
            Self::Trading212DemoOrderBook(_) => "trading212-demo-order-book",
            Self::BinanceTestnetOrderBook(_) => "binance-testnet-order-book",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DomainEvent {
    #[schemars(length(min = 1))]
    pub event_id: String,
    #[schemars(extend("enum" = ["workspace.opened", "account.health.changed", "model.gateway.changed", "model.provider.changed", "model.provider_attempt.changed", "risk.policy.changed", "risk.decision.evaluated", "thread.created", "thread.updated", "trading212.demo.order.attempt.changed", "trading212.demo.order.book.changed", "alpaca.paper.order.attempt.changed", "alpaca.paper.order.book.changed", "binance.testnet.order.attempt.changed", "binance.testnet.order.book.changed", "bitget.demo.order.attempt.changed"]))]
    pub event_type: String,
    #[schemars(extend("const" = 1))]
    pub schema_version: u32,
    pub occurred_at: String,
    #[schemars(extend("enum" = ["workspace", "account", "model-gateway", "model", "risk", "risk-decision", "thread", "trading212-demo-order-attempt", "trading212-demo-order-book", "alpaca-paper-order-attempt", "alpaca-paper-order-book", "binance-testnet-order-attempt", "binance-testnet-order-book", "bitget-demo-order-attempt"]))]
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
    #[schemars(extend("enum" = ["workspace", "account", "model-gateway", "model", "risk", "risk-decision", "thread", "trading212-demo-order-attempt", "trading212-demo-order-book", "alpaca-paper-order-attempt", "alpaca-paper-order-book", "binance-testnet-order-attempt", "binance-testnet-order-book", "bitget-demo-order-attempt"]))]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub field: Option<Box<str>>,
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
            "ORDER_CONTEXT_INVALID" => (
                "This execution context cannot be used for an order draft or does not match the account.",
                "select_execution_context",
                "Choose a compatible context",
            ),
            "ORDER_ACCOUNT_REQUIRED" => (
                "Select an existing provider account for this execution context.",
                "select_account",
                "Select an account",
            ),
            "ORDER_ACCOUNT_INVALID" => (
                "The selected account is not valid for this workspace or context.",
                "select_account",
                "Choose another account",
            ),
            "ORDER_ACCOUNT_NOT_FOUND" => (
                "That account is no longer available. Reload the account list.",
                "select_account",
                "Reload accounts",
            ),
            "ORDER_INSTRUMENT_NOT_FOUND" => (
                "The selected instrument is no longer in the current catalog.",
                "reload_snapshot",
                "Reload market catalog",
            ),
            "ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED" => (
                "The selected provider does not support this instrument mapping.",
                "select_instrument",
                "Choose another instrument",
            ),
            "ORDER_VENUE_INVALID" => (
                "The venue does not match the selected instrument and execution context.",
                "select_venue",
                "Choose a compatible venue",
            ),
            "ORDER_DECIMAL_INVALID" => (
                "Enter a decimal amount without exponent notation.",
                "edit_order_amount",
                "Review order amount",
            ),
            "ORDER_AMOUNT_INVALID" => (
                "Order quantity and prices must be greater than zero.",
                "edit_order_amount",
                "Review order amount",
            ),
            "ORDER_LIMIT_PRICE_REQUIRED" => (
                "A limit price is required for a limit order.",
                "edit_order_amount",
                "Enter a limit price",
            ),
            "ORDER_MARKET_PRICE_FORBIDDEN" => (
                "Market orders cannot include a limit price.",
                "edit_order_amount",
                "Remove the limit price",
            ),
            "ORDER_TIF_INVALID" => (
                "This time in force is not supported for the selected order type.",
                "edit_order_type",
                "Choose a compatible time in force",
            ),
            "ORDER_SUBMIT_FORBIDDEN" => (
                "Confirm Alpaca Paper orders from the main Trade surface. Agents and research tools cannot submit orders.",
                "open_trade_surface",
                "Open Trade",
            ),
            "ORDER_ATTEMPT_NOT_FOUND" => (
                "No saved Alpaca Paper attempt exists for this proposal. Reload its authoritative state.",
                "reload_snapshot",
                "Reload proposal",
            ),
            "ORDER_PROPOSAL_CONSUMED" => (
                "This proposal already has a saved Alpaca Paper submission attempt. Check its status; do not submit it again.",
                "reload_snapshot",
                "Reload attempt",
            ),
            "ORDER_PROPOSAL_NOT_ELIGIBLE" => (
                "Only a current Alpaca Paper proposal bound to its connected account can be submitted.",
                "reload_snapshot",
                "Reload proposal",
            ),
            "ORDER_ASSET_UNAVAILABLE" => (
                "Alpaca could not find this canonical equity symbol for Paper trading.",
                "select_instrument",
                "Choose another instrument",
            ),
            "ORDER_ASSET_UNTRADABLE" => (
                "Alpaca reports this asset as inactive or not tradable. Choose an eligible instrument.",
                "select_instrument",
                "Review instrument",
            ),
            "ORDER_ASSET_NOT_FRACTIONABLE" => (
                "Alpaca does not allow fractional orders for this asset. Use a whole-share quantity or another instrument.",
                "edit_order_amount",
                "Review quantity",
            ),
            "ORDER_INSUFFICIENT_POSITION" => (
                "The Alpaca Paper account does not have enough available long shares for this sell. Opening short positions is disabled.",
                "reload_snapshot",
                "Refresh positions",
            ),
            "ORDER_BUYING_POWER_INSUFFICIENT" => (
                "Alpaca reports insufficient buying power for this Paper order. Review the account and order size, then create a new proposal.",
                "reload_snapshot",
                "Refresh account",
            ),
            "ORDER_CAPABILITY_UNSUPPORTED" => (
                "TradeX cannot verify this Alpaca Paper order combination for this account. Review the order terms and account permissions.",
                "edit_order_type",
                "Review order terms",
            ),
            "ORDER_STATUS_UNKNOWN" => (
                "Alpaca has not established this order's status. Reconcile by client order ID; do not submit it again.",
                "retry_provider",
                "Reconcile order",
            ),
            "PROVIDER_ORDER_REJECTED" => (
                "Alpaca rejected the Paper order. Review the order terms and account restrictions before creating a new proposal.",
                "reload_snapshot",
                "Review provider status",
            ),
            "ORDER_DRAFT_NOT_FOUND" => (
                "That order draft is no longer available. Reload the draft library.",
                "reload_snapshot",
                "Reload drafts",
            ),
            "ORDER_PROPOSAL_NOT_FOUND" => (
                "That order proposal is no longer available. Reload proposal history.",
                "reload_snapshot",
                "Reload proposals",
            ),
            "ORDER_PROPOSAL_NOT_REFRESHABLE" => (
                "This proposal is already invalidated and must be regenerated from the current draft.",
                "reload_snapshot",
                "Reload proposal history",
            ),
            "PAPER_PROPOSAL_INVALID" => (
                "Only a current Local Paper proposal can be submitted by the simulation engine.",
                "reload_snapshot",
                "Reload proposal",
            ),
            "PAPER_PROPOSAL_NOT_SELECTED" => (
                "Select the Local Paper proposal from the Trade surface before submitting it.",
                "reload_snapshot",
                "Select proposal",
            ),
            "PAPER_PROPOSAL_CONSUMED" => (
                "This proposal has already been consumed by a Local Paper order.",
                "reload_snapshot",
                "Reload order history",
            ),
            "PAPER_IDEMPOTENCY_CONFLICT" => (
                "That idempotency key belongs to another Local Paper order.",
                "retry_request",
                "Retry with a new key",
            ),
            "PAPER_QUOTE_UNAVAILABLE" => (
                "The deterministic Local Paper quote is unavailable for this proposal.",
                "retry_request",
                "Retry submission",
            ),
            "PAPER_LIMIT_NOT_CROSSED" => (
                "The Local Paper limit order remains open because its limit does not cross the deterministic simulation quote.",
                "reload_snapshot",
                "Review open order",
            ),
            "PAPER_ORDER_NOT_FOUND" => (
                "That Local Paper order is no longer available in this workspace.",
                "reload_snapshot",
                "Reload simulation",
            ),
            "PAPER_ORDER_NOT_CANCELLABLE" => (
                "This Local Paper order is already terminal and cannot be cancelled.",
                "reload_snapshot",
                "Reload order state",
            ),
            "PAPER_SCENARIO_INVALID" => (
                "The Local Paper scenario or profile is outside the bounded simulation contract.",
                "edit_order_amount",
                "Choose a supported scenario",
            ),
            "PAPER_SCENARIO_ORDER_OPEN" => (
                "Close open Local Paper orders before changing the simulation scenario.",
                "reload_snapshot",
                "Review open orders",
            ),
            "PAPER_SCENARIO_AGENT_FORBIDDEN" => (
                "Agents cannot mutate the Local Paper simulation profile. Use the Trade surface.",
                "select_proposal",
                "Open Trade surface",
            ),
            "PAPER_QUOTE_REFRESH_AGENT_FORBIDDEN" => (
                "Agents cannot refresh the Local Paper quote. Use the Trade surface.",
                "select_proposal",
                "Open Trade surface",
            ),
            "PAPER_MAXIMUM_SPEND_EXCEEDED" => (
                "The deterministic Local Paper fill exceeds the proposal maximum spend.",
                "edit_order_amount",
                "Review maximum spend",
            ),
            "PAPER_INSUFFICIENT_CASH" => (
                "The Local Paper account does not have enough simulation cash for this order.",
                "reload_snapshot",
                "Review simulation balance",
            ),
            "PAPER_INSUFFICIENT_POSITION" => (
                "The Local Paper account does not have enough simulated position to sell.",
                "reload_snapshot",
                "Review simulation position",
            ),
            "PAPER_QUANTITY_UNREPRESENTABLE" => (
                "The requested quote quantity cannot be represented at the simulation quote precision.",
                "edit_order_amount",
                "Review order quantity",
            ),
            "PAPER_ACCOUNT_INVALID" => (
                "The Local Paper account is not valid for this workspace.",
                "reload_snapshot",
                "Reload account state",
            ),
            "PAPER_STATE_LIMIT" => (
                "The Local Paper event history reached its bounded limit.",
                "reload_snapshot",
                "Reload simulation state",
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
            "ACCOUNT_DELETE_BLOCKED" => (
                "This local connection cannot be deleted while it is connected, has a credential, or has open or unresolved TradeX activity. Reload account state after resolving it.",
                "reload_snapshot",
                "Reload account state",
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
            "ARTIFACT_NOT_FOUND" => (
                "That artifact is not available in the active workspace.",
                "reload_snapshot",
                "Reload artifacts",
            ),
            "ARTIFACT_SOURCE_INVALID" => (
                "The selected completed Thread Item is not a valid artifact source.",
                "reload_snapshot",
                "Choose another source",
            ),
            "ARTIFACT_REDACTION_FAILED" => (
                "The artifact contains unsupported sensitive content and was not saved.",
                "retry_request",
                "Review content",
            ),
            "ARTIFACT_EXPORT_PATH_INVALID" => (
                "Choose a safe local JSON filename inside the TradeX exports folder.",
                "retry_request",
                "Choose another filename",
            ),
            "ARTIFACT_EXPORT_EXISTS" => (
                "That export filename already exists. Choose another filename.",
                "retry_request",
                "Choose another filename",
            ),
            "ARTIFACT_EXPORT_FAILED" => (
                "The artifact could not be exported; no partial file was kept.",
                "retry_request",
                "Retry export",
            ),
            "STRATEGY_DEFINITION_INVALID" => (
                "The strategy definition is outside the bounded sandbox contract.",
                "edit_strategy",
                "Review strategy",
            ),
            "STRATEGY_PARAMETER_INVALID" => (
                "Strategy parameters must be unique, bounded and declared by the saved version.",
                "edit_strategy",
                "Review parameters",
            ),
            "STRATEGY_VERSION_INVALID" | "STRATEGY_VERSION_LIMIT" => (
                "The strategy version could not be saved. Review the bounded version fields.",
                "edit_strategy",
                "Review strategy",
            ),
            "STRATEGY_VERSION_NOT_FOUND" => (
                "That immutable strategy version is no longer available in this workspace.",
                "reload_snapshot",
                "Reload strategies",
            ),
            "STRATEGY_HASH_MISMATCH" => (
                "The selected strategy version hash changed. Reload it before running.",
                "reload_snapshot",
                "Reload strategy",
            ),
            "STRATEGY_RUN_INVALID" => (
                "The strategy run inputs do not match the bounded run contract.",
                "review_strategy_run",
                "Review run inputs",
            ),
            "STRATEGY_DATASET_NOT_FOUND" => (
                "The selected historical dataset is not a canonical TradeX reference.",
                "select_dataset",
                "Choose a dataset",
            ),
            "STRATEGY_TIME_UNTRUSTED" => (
                "TradeX time is not trusted, so this strategy run is blocked.",
                "time_revalidate",
                "Revalidate time",
            ),
            "STRATEGY_RUN_NOT_FOUND" => (
                "That strategy run is no longer available in this workspace.",
                "reload_snapshot",
                "Reload runs",
            ),
            "STRATEGY_RUN_NOT_CANCELLABLE" => (
                "This strategy run has already finished and cannot be cancelled.",
                "reload_snapshot",
                "Reload run",
            ),
            "STRATEGY_RUNTIME_UNAVAILABLE" => (
                "No production strategy runtime is configured, so this run failed closed without synthetic output.",
                "retry_strategy_run",
                "Retry strategy run",
            ),
            "STRATEGY_WORKER_FAILED" => (
                "The restricted strategy worker failed safely without exposing provider secrets.",
                "retry_strategy_run",
                "Retry strategy run",
            ),
            "BACKTEST_CONFIG_INVALID"
            | "BACKTEST_RUN_INVALID"
            | "BACKTEST_DECIMAL_INVALID"
            | "BACKTEST_BAR_INTERVAL_INVALID"
            | "BACKTEST_DATE_RANGE_INVALID"
            | "BACKTEST_PARAMETER_INVALID"
            | "BACKTEST_PORTFOLIO_SEED_INVALID" => (
                "Review the bounded historical backtest configuration.",
                "review_backtest",
                "Review backtest",
            ),
            "BACKTEST_INSTRUMENT_NOT_FOUND" => (
                "Choose a canonical instrument from the market catalog.",
                "select_instrument",
                "Choose instrument",
            ),
            "BACKTEST_DATASET_NOT_FOUND" => (
                "Choose a supported historical dataset.",
                "select_dataset",
                "Choose dataset",
            ),
            "BACKTEST_STRATEGY_HASH_INVALID" | "BACKTEST_STRATEGY_HASH_MISMATCH" => (
                "The selected strategy version changed. Reload it before running.",
                "reload_snapshot",
                "Reload strategy",
            ),
            "BACKTEST_TIME_UNTRUSTED" => (
                "TradeX time is not trusted, so this backtest is blocked.",
                "time_revalidate",
                "Revalidate time",
            ),
            "BACKTEST_FIXTURE_UNAVAILABLE" => (
                "The browser integration fixture is unavailable in this runtime.",
                "retry_request",
                "Retry backtest",
            ),
            "BACKTEST_RUNTIME_UNAVAILABLE" | "BACKTEST_FIXTURE_FAILED" => (
                "The backtest runtime failed closed without producing synthetic results.",
                "retry_backtest_run",
                "Retry backtest",
            ),
            "BACKTEST_RESULT_INVALID" | "BACKTEST_DATASET_HASH_MISMATCH" => (
                "The backtest result or approved dataset identity could not be validated safely.",
                "retry_backtest_run",
                "Retry backtest",
            ),
            "BACKTEST_LOOKAHEAD_DETECTED"
            | "BACKTEST_SURVIVORSHIP_BIAS"
            | "BACKTEST_SPLIT_UNVERIFIED"
            | "BACKTEST_DIVIDEND_UNVERIFIED"
            | "BACKTEST_TIMEZONE_MISMATCH"
            | "BACKTEST_DATA_GAP" => (
                "The backtest data guard rejected this historical input.",
                "review_backtest",
                "Review backtest data",
            ),
            "BACKTEST_CANCELLED" => (
                "The backtest was cancelled before completion.",
                "retry_backtest_run",
                "Retry backtest",
            ),
            "BACKTEST_FAILURE_INVALID" | "BACKTEST_IDENTITY_FAILED" => (
                "The backtest result could not be validated safely.",
                "retry_backtest_run",
                "Retry backtest",
            ),
            "BACKTEST_COMPARE_INVALID" => (
                "Choose two valid completed backtest runs to compare.",
                "select_backtest_runs",
                "Choose runs",
            ),
            "BACKTEST_COMPARE_SAME_RUN" => (
                "Choose two different completed backtest runs.",
                "select_backtest_runs",
                "Choose different runs",
            ),
            "BACKTEST_COMPARE_NOT_COMPLETED" => (
                "Only completed backtest runs can be compared.",
                "select_backtest_runs",
                "Choose completed runs",
            ),
            "BACKTEST_RUN_NOT_FOUND" => (
                "That backtest run is no longer available in this workspace.",
                "reload_snapshot",
                "Reload runs",
            ),
            "BACKTEST_RUN_NOT_CANCELLABLE" | "BACKTEST_RUN_TERMINAL_IMMUTABLE" => (
                "This backtest run has already reached a terminal state.",
                "reload_snapshot",
                "Reload run",
            ),
            "SCREENER_REVISION_STALE" => (
                "The reviewed screener conditions changed. Parse them again before running.",
                "retry_request",
                "Recalculate screener",
            ),
            "SCREENER_FILTER_UNSUPPORTED" => (
                "This screener contains a condition that TradeX cannot verify yet.",
                "retry_request",
                "Review screener",
            ),
            "SCREENER_REVISION_FAILED" => (
                "The screener revision could not be created. Review the conditions and retry.",
                "retry_request",
                "Retry screener",
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
            "RISK_REJECTED" => (
                "The current risk policy rejected this proposal. Review its saved RiskDecision before retrying.",
                "review_risk",
                "Review risk decision",
            ),
            "RISK_EVIDENCE_UNAVAILABLE" => (
                "Required risk evidence is missing or untrusted. No order was submitted.",
                "review_risk",
                "Review risk evidence",
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
            } else if matches!(code, "STRATEGY_TIME_UNTRUSTED" | "BACKTEST_TIME_UNTRUSTED")
                || code == "ACCOUNT_DELETE_BLOCKED"
            {
                "STATE_STALE"
            } else if matches!(
                code,
                "STRATEGY_WORKER_FAILED"
                    | "STRATEGY_RUNTIME_UNAVAILABLE"
                    | "BACKTEST_RUNTIME_UNAVAILABLE"
                    | "BACKTEST_FIXTURE_FAILED"
            ) {
                "RUNTIME_ERROR"
            } else if code.starts_with("MODEL_")
                || code == "PROVIDER_AUTH_FAILED"
                || code.starts_with("CREDENTIAL_")
            {
                "AUTH_ERROR"
            } else if code.starts_with("RISK_") || code.starts_with("ONBOARDING_") {
                "POLICY_ERROR"
            } else if code.starts_with("PAPER_") {
                "SIMULATION_ERROR"
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
                    | "ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"
                    | "ORDER_CAPABILITY_UNSUPPORTED"
                    | "ORDER_ASSET_UNAVAILABLE"
                    | "ORDER_ASSET_UNTRADABLE"
                    | "ORDER_ASSET_NOT_FRACTIONABLE"
            ) {
                "UNSUPPORTED_CAPABILITY"
            } else if matches!(code, "MARKET_CLOSED" | "INSTRUMENT_HALTED") {
                code
            } else if matches!(
                code,
                "PROVIDER_PERMISSION_BLOCKED" | "ORDER_SUBMIT_FORBIDDEN"
            ) {
                "PERMISSION_ERROR"
            } else if matches!(
                code,
                "PROVIDER_ORDER_REJECTED"
                    | "ORDER_BUYING_POWER_INSUFFICIENT"
                    | "ORDER_INSUFFICIENT_POSITION"
            ) {
                "PROVIDER_ORDER_REJECTED"
            } else if matches!(
                code,
                "IPC_AGGREGATE_NOT_FOUND"
                    | "ARTIFACT_NOT_FOUND"
                    | "ORDER_PROPOSAL_NOT_FOUND"
                    | "ORDER_PROPOSAL_NOT_REFRESHABLE"
                    | "ORDER_ATTEMPT_NOT_FOUND"
                    | "ORDER_PROPOSAL_CONSUMED"
                    | "ORDER_PROPOSAL_NOT_ELIGIBLE"
                    | "ORDER_STATUS_UNKNOWN"
                    | "WATCHLIST_NOT_FOUND"
                    | "STATE_VERSION_CONFLICT"
                    | "IPC_REPLAY_UNAVAILABLE"
                    | "CLOCK_SKEW"
                    | "BACKTEST_RUN_NOT_FOUND"
                    | "BACKTEST_RUN_NOT_CANCELLABLE"
                    | "BACKTEST_RUN_TERMINAL_IMMUTABLE"
                    | "BACKTEST_CANCELLED"
                    | "BACKTEST_COMPARE_SAME_RUN"
                    | "BACKTEST_COMPARE_NOT_COMPLETED"
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
                    | "PAPER_QUOTE_UNAVAILABLE"
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
                    | "STRATEGY_WORKER_FAILED"
                    | "STRATEGY_RUNTIME_UNAVAILABLE"
                    | "BACKTEST_RUNTIME_UNAVAILABLE"
                    | "BACKTEST_FIXTURE_FAILED"
                    | "BACKTEST_FIXTURE_UNAVAILABLE"
                    | "ARTIFACT_EXPORT_FAILED"
            ),
            blocking: true,
            remediation_actions: vec![Remediation {
                id: action.into(),
                label: label.into(),
            }],
            field: None,
        }
    }

    pub fn with_field(mut self, field: &str) -> Self {
        self.field = Some(field.to_owned().into_boxed_str());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperMoney {
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub value: String,
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub currency: String,
}

fn default_local_paper_scenario_seed() -> String {
    "s16-default".into()
}

fn default_local_paper_scenario_version() -> String {
    "s16-v1".into()
}

fn default_local_paper_fee_policy() -> String {
    "ZERO".into()
}

fn default_local_paper_slippage_policy() -> String {
    "NONE".into()
}

fn default_local_paper_fill_policy() -> String {
    "BOUNDED_V1".into()
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperProfile {
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub base_currency: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub starting_cash: String,
    #[schemars(length(min = 1, max = 64))]
    pub quote_source: String,
    #[schemars(length(min = 1, max = 128))]
    pub scenario_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub engine_version: String,
    #[serde(default = "default_local_paper_scenario_seed")]
    #[schemars(required, length(min = 1, max = 64))]
    pub scenario_seed: String,
    #[serde(default = "default_local_paper_scenario_version")]
    #[schemars(required, length(min = 1, max = 64))]
    pub scenario_version: String,
    #[serde(default = "default_local_paper_fee_policy")]
    #[schemars(required, length(min = 1, max = 64))]
    pub fee_policy: String,
    #[serde(default = "default_local_paper_slippage_policy")]
    #[schemars(required, length(min = 1, max = 64))]
    pub slippage_policy: String,
    #[serde(default = "default_local_paper_fill_policy")]
    #[schemars(required, length(min = 1, max = 64))]
    pub fill_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub quote_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 32))]
    pub quote_freshness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 64))]
    pub quote_observed_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperQuote {
    #[schemars(length(min = 1, max = 128))]
    pub quote_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub price: String,
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub currency: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
    #[schemars(length(min = 1, max = 128))]
    pub scenario_id: String,
    #[schemars(length(min = 1, max = 64))]
    pub source: String,
    #[schemars(length(min = 1, max = 32))]
    pub freshness: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalPaperOrderState {
    Proposed,
    Accepted,
    PartiallyFilled,
    Filled,
    Rejected,
    CancelPending,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperBalance {
    #[schemars(length(min = 1, max = 64))]
    pub asset: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub available: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub total: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub reserved: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperPosition {
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub quantity: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub average_entry_price: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub market_value: Option<String>,
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub currency: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub unrealized_pnl: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperOrder {
    #[schemars(length(min = 1, max = 128))]
    pub order_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub state: LocalPaperOrderState,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub requested_quantity: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub filled_quantity: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub remaining_quantity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity_type: Option<OrderQuantityType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub limit_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub average_fill_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<LocalPaperQuote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub cancel_idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub event_sequence: Option<u64>,
    #[schemars(length(min = 1, max = 64))]
    pub created_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperFill {
    #[schemars(length(min = 1, max = 128))]
    pub fill_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub order_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    pub side: OrderSide,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub quantity: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub price: String,
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub value: String,
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub currency: String,
    #[schemars(length(min = 1, max = 64))]
    pub observed_at: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalPaperEventKind {
    Accepted,
    PartiallyFilled,
    Filled,
    Rejected,
    Cancelled,
    ScenarioChanged,
    QuoteRefreshed,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperEvent {
    #[schemars(length(min = 1, max = 128))]
    pub event_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub sequence: u64,
    pub kind: LocalPaperEventKind,
    #[schemars(length(min = 1, max = 128))]
    pub order_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1, max = 128))]
    pub fill_id: Option<String>,
    #[schemars(length(min = 1, max = 64))]
    pub occurred_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPaperState {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub account_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub provider_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub environment: String,
    #[schemars(length(min = 1, max = 120))]
    pub account_label: String,
    pub profile: LocalPaperProfile,
    pub cash: LocalPaperMoney,
    pub reserved_cash: LocalPaperMoney,
    pub equity: LocalPaperMoney,
    pub realized_pnl: LocalPaperMoney,
    pub unrealized_pnl: LocalPaperMoney,
    pub exposure: LocalPaperMoney,
    #[schemars(length(max = 512))]
    pub balances: Vec<LocalPaperBalance>,
    #[serde(default)]
    #[schemars(length(max = 512))]
    pub orders: Vec<LocalPaperOrder>,
    #[schemars(length(max = 512))]
    pub positions: Vec<LocalPaperPosition>,
    #[schemars(length(max = 512))]
    pub open_orders: Vec<LocalPaperOrder>,
    #[schemars(length(max = 512))]
    pub fills: Vec<LocalPaperFill>,
    #[serde(default)]
    #[schemars(length(max = 1024))]
    pub events: Vec<LocalPaperEvent>,
    #[schemars(range(min = 0, max = 9_007_199_254_740_991_u64))]
    pub event_cursor: u64,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(length(min = 1, max = 64))]
    pub updated_at: String,
    #[schemars(length(min = 1, max = 256))]
    pub disclosure: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperOrderSubmit {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_proposal_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperOrderCancel {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub order_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub idempotency_key: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperQuoteRefresh {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperScenarioSet {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
    pub profile: LocalPaperProfile,
}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaperOrderResult {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub account_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub environment: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(regex(pattern = "^sha256:[0-9a-f]{64}$"))]
    pub proposal_hash: String,
    pub order: LocalPaperOrder,
    pub fill: Option<LocalPaperFill>,
    pub quote: LocalPaperQuote,
    #[schemars(length(min = 1, max = 256))]
    pub proposal_state_version: String,
    #[schemars(length(min = 1, max = 256))]
    pub state_version: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub event_sequence: u64,
    #[schemars(length(min = 1, max = 256))]
    pub disclosure: String,
    pub paper_state: Box<LocalPaperState>,
}

pub type Result<T> = std::result::Result<T, TradeXError>;
