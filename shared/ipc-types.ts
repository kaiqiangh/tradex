/* Generated from Rust protocol.rs. Run npm run schema:generate. */

/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperFillSource".
 */
export type AlpacaPaperFillSource = "REST_ACTIVITY" | "TRADE_UPDATE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperCancelState".
 */
export type AlpacaPaperCancelState = "NONE" | "SUBMITTING" | "PENDING";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderOrigin".
 */
export type AlpacaPaperOrderOrigin = "TRADE_X" | "EXTERNAL";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderAttemptState".
 */
export type AlpacaPaperOrderAttemptState = "SUBMITTING" | "ACKNOWLEDGED" | "UNKNOWN_RECONCILING" | "REJECTED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderBookStatus".
 */
export type AlpacaPaperOrderBookStatus = "NEVER_SYNCED" | "CURRENT" | "DEGRADED" | "STALE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactKind".
 */
export type ArtifactKind = "RESEARCH" | "DECISION";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestGuardState".
 */
export type BacktestGuardState = "PASSED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestRunState".
 */
export type BacktestRunState = "QUEUED" | "RUNNING" | "COMPLETED" | "FAILED" | "CANCELLED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestFixtureScenario".
 */
export type BacktestFixtureScenario =
  | "SUCCESS"
  | "FAILURE"
  | "CANCELLED"
  | "LOOKAHEAD"
  | "SURVIVORSHIP"
  | "SPLIT"
  | "DIVIDEND"
  | "TIMEZONE"
  | "DATA_GAP"
  | "DATASET_HASH_MISMATCH";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AgentMode".
 */
export type AgentMode = "ASK" | "RESEARCH" | "BACKTEST" | "TRADE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ExecutionContext".
 */
export type ExecutionContext =
  | "NONE_READ_ONLY"
  | "HISTORICAL_SIMULATION"
  | "LOCAL_PAPER"
  | "ALPACA_PAPER"
  | "TRADING212_DEMO"
  | "TRADING212_LIVE"
  | "BINANCE_TESTNET"
  | "BINANCE_LIVE"
  | "BITGET_DEMO"
  | "BITGET_LIVE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "CapabilityLevel".
 */
export type CapabilityLevel = "C0" | "C1" | "C2" | "C3" | "C4" | "C5" | "C6";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ChatgptLoginAction".
 */
export type ChatgptLoginAction = "LOGIN" | "RELOGIN";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DomainProjection".
 */
export type DomainProjection =
  | GatewayState
  | ModelState
  | Workspace
  | AccountConnection
  | RiskPolicyState
  | Thread
  | Trading212DemoOrderAttempt
  | AlpacaPaperOrderAttempt
  | AlpacaPaperOrderBook
  | Trading212DemoOrderBook;
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "GatewayStatus".
 */
export type GatewayStatus =
  | "STOPPED"
  | "INSTALLING"
  | "STARTING"
  | "RUNNING"
  | "PORT_CONFLICT"
  | "UNAUTHORIZED"
  | "BACKOFF"
  | "FAILED"
  | "STOPPING";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelAttemptOutcome".
 */
export type ModelAttemptOutcome = "VERIFIED" | "CONFIGURED" | "CANCELLED" | "FAILED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelProvider".
 */
export type ModelProvider = "CHATGPT" | "DEEPSEEK";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThinkingType".
 */
export type ThinkingType = "disabled" | "enabled";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelHealth".
 */
export type ModelHealth = "NOT_CONFIGURED" | "UNVERIFIED" | "VERIFYING" | "READY" | "FAILED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ConnectionState".
 */
export type ConnectionState = "CONNECTING" | "REVIEW_REQUIRED" | "CONNECTED" | "FAILED" | "DISCONNECTED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadStatus".
 */
export type ThreadStatus = "ACTIVE" | "ARCHIVED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchFreshness".
 */
export type ResearchFreshness = "HEALTHY" | "STALE" | "UNAVAILABLE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchQuality".
 */
export type ResearchQuality = "VERIFIED" | "DEGRADED" | "UNKNOWN" | "UNAVAILABLE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DataSourceStatus".
 */
export type DataSourceStatus = "AVAILABLE" | "UNAVAILABLE" | "BLOCKED_EXTERNAL" | "UNVERIFIED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchFocus".
 */
export type ResearchFocus = "GENERAL" | "EQUITY" | "CRYPTO_SPOT";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchResultState".
 */
export type ResearchResultState = "AVAILABLE" | "DEGRADED" | "UNAVAILABLE" | "BLOCKED_EXTERNAL" | "FAILED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchSpotVenueId".
 */
export type ResearchSpotVenueId = "BINANCE" | "BITGET";
/**
 * Data-plane tools are intentionally separate from financial authority IDs.
 *
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchToolId".
 */
export type ResearchToolId = "public_market_read" | "account_read" | "historical_simulation";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ItemStatus".
 */
export type ItemStatus = "STARTED" | "STREAMING" | "COMPLETED" | "FAILED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TurnStatus".
 */
export type TurnStatus = "RUNNING" | "COMPLETED" | "CANCELLED" | "INTERRUPTED" | "FAILED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderAttemptState".
 */
export type Trading212DemoOrderAttemptState = "SUBMITTING" | "ACKNOWLEDGED" | "UNKNOWN_RECONCILING" | "REJECTED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoNormalizedOrderStatus".
 */
export type Trading212DemoNormalizedOrderStatus =
  | "LOCAL"
  | "PENDING"
  | "OPEN"
  | "CANCEL_PENDING"
  | "CANCELLED"
  | "PARTIALLY_FILLED"
  | "FILLED"
  | "REJECTED"
  | "REPLACING"
  | "REPLACED"
  | "EXPIRED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderOrigin".
 */
export type Trading212DemoOrderOrigin = "TRADE_X" | "EXTERNAL";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderBookStatus".
 */
export type Trading212DemoOrderBookStatus = "NEVER_SYNCED" | "CURRENT" | "DEGRADED" | "STALE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "GatewayAction".
 */
export type GatewayAction = "LAUNCH" | "PROBE" | "RESTART" | "STOP";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperEventKind".
 */
export type LocalPaperEventKind =
  "ACCEPTED" | "PARTIALLY_FILLED" | "FILLED" | "REJECTED" | "CANCELLED" | "SCENARIO_CHANGED" | "QUOTE_REFRESHED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderSide".
 */
export type OrderSide = "BUY" | "SELL";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderType".
 */
export type OrderType = "MARKET" | "LIMIT";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderQuantityType".
 */
export type OrderQuantityType = "BASE" | "QUOTE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperOrderState".
 */
export type LocalPaperOrderState =
  "PROPOSED" | "ACCEPTED" | "PARTIALLY_FILLED" | "FILLED" | "REJECTED" | "CANCEL_PENDING" | "CANCELLED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TimeInForce".
 */
export type TimeInForce = "DAY" | "GTC" | "IOC" | "FOK";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketTier".
 */
export type MarketTier = "CENSUS" | "WARM" | "HOT" | "COLD";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalHistoryEvent".
 */
export type OrderProposalHistoryEvent = "GENERATED" | "DRAFT_CHANGED" | "REFRESHED" | "CONSUMED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketDataStatus".
 */
export type MarketDataStatus = "AVAILABLE" | "UNAVAILABLE" | "BLOCKED_EXTERNAL" | "UNVERIFIED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ProposalReferenceStatus".
 */
export type ProposalReferenceStatus = "AVAILABLE" | "UNCONFIGURED" | "UNAVAILABLE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalStatus".
 */
export type OrderProposalStatus = "NEEDS_APPROVAL" | "CONSUMED" | "INVALIDATED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalRefreshStatus".
 */
export type OrderProposalRefreshStatus = "REFRESHED" | "STALE" | "BLOCKED" | "UNAVAILABLE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Connect".
 */
export type Connect =
  | {
      environment: string;
      label: string;
      providerId: string;
      step: "test";
      workspaceId: string;
    }
  | {
      acknowledgeUnverified: boolean;
      connectionId: string;
      expectedStateVersion: string;
      step: "confirm";
      workspaceId: string;
    };
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResultEnvelope".
 */
export type ResultEnvelope = SuccessEnvelope | FailureEnvelope;
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ReplyData".
 */
export type ReplyData =
  | Workspace
  | Snapshot
  | RuntimeStatus
  | TimeStatus
  | SubscriptionAck
  | Thread
  | ThreadList
  | GatewayState
  | ModelState
  | RiskPolicyState
  | ProviderCatalog
  | ProviderDefinition
  | Accounts
  | AccountConnection
  | PermissionReview
  | CapabilityDecision
  | ContextCatalog
  | DataSourceCatalog
  | MarketCatalog
  | MarketDetail
  | PortfolioSnapshot
  | LocalPaperState
  | Watchlist
  | Watchlists
  | ResearchToolResult
  | ScreenerResult
  | ScreenerLibrary
  | ScreenerAttachment
  | OrderDraft
  | OrderDraftLibrary
  | OrderProposal
  | OrderProposalLibrary
  | OrderProposalRefreshResult
  | Trading212DemoOrderAttempt
  | Trading212DemoOrderAttemptQueryResult
  | Trading212DemoOrderBook
  | Trading212DemoOrderBookQueryResult
  | AlpacaPaperOrderAttempt
  | AlpacaPaperOrderAttemptQueryResult
  | AlpacaPaperOrderBook
  | AlpacaPaperOrderBookQueryResult
  | PaperOrderResult
  | Artifact
  | ArtifactLibrary
  | ArtifactExportResult
  | StrategyLibrary
  | StrategyVersion
  | StrategyRun
  | BacktestRun
  | BacktestLibrary
  | BacktestComparison;
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TimeConfidence".
 */
export type TimeConfidence = "TRUSTED" | "CLOCK_UNCERTAIN" | "STALE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ToolId".
 */
export type ToolId =
  | "public_market_read"
  | "account_read"
  | "historical_simulation"
  | "paper_demo_testnet_execution"
  | "live_order_proposal";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DataSourceProbeKind".
 */
export type DataSourceProbeKind = "PUBLIC_METADATA" | "CREDENTIALED_METADATA";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AssetClass".
 */
export type AssetClass = "EQUITY" | "CRYPTO_SPOT";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AdjustmentStatus".
 */
export type AdjustmentStatus = "ADJUSTED" | "UNADJUSTED" | "UNKNOWN" | "UNAVAILABLE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "CorporateActionType".
 */
export type CorporateActionType = "SPLIT" | "DIVIDEND" | "SYMBOL_CHANGE" | "DELISTING";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketSession".
 */
export type MarketSession =
  "OPEN" | "CLOSED" | "EXTENDED_HOURS" | "HALTED" | "MAINTENANCE" | "SUSPENDED" | "DEGRADED" | "UNKNOWN";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketEntitlement".
 */
export type MarketEntitlement = "REALTIME" | "DELAYED" | "UNKNOWN";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketFreshness".
 */
export type MarketFreshness = "HEALTHY" | "STALE" | "CLOCK_UNCERTAIN";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "FxFreshness".
 */
export type FxFreshness = "HEALTHY" | "STALE" | "UNAVAILABLE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "FxQuality".
 */
export type FxQuality = "VERIFIED" | "DEGRADED" | "UNKNOWN" | "UNAVAILABLE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioStatus".
 */
export type PortfolioStatus = "AVAILABLE" | "DEGRADED" | "UNAVAILABLE" | "BLOCKED_EXTERNAL";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerFeatureField".
 */
export type ScreenerFeatureField =
  "REVENUE_GROWTH" | "ESTIMATE_REVISION" | "RSI" | "PRICE_CHANGE" | "QUALITY" | "REVISION_STRENGTH" | "MOMENTUM";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerPredicateField".
 */
export type ScreenerPredicateField = "REVENUE_GROWTH" | "ESTIMATE_REVISION" | "RSI" | "PRICE_CHANGE";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerOperator".
 */
export type ScreenerOperator = "GREATER_THAN" | "GREATER_OR_EQUAL" | "LESS_THAN" | "LESS_OR_EQUAL";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerUniverse".
 */
export type ScreenerUniverse = "US_EQUITIES" | "US_LARGE_CAP_TECHNOLOGY" | "CRYPTO_SPOT";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerOperation".
 */
export type ScreenerOperation = "PARSE" | "RUN";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerDirection".
 */
export type ScreenerDirection = "ASC" | "DESC";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerRankField".
 */
export type ScreenerRankField = "QUALITY" | "REVISION_STRENGTH" | "MOMENTUM";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerResultState".
 */
export type ScreenerResultState = "PARSED" | "RUNNING" | "EMPTY" | "COMPLETED" | "BLOCKED_EXTERNAL" | "FAILED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyRunState".
 */
export type StrategyRunState = "QUEUED" | "RUNNING" | "COMPLETED" | "FAILED" | "CANCELLED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyDirection".
 */
export type StrategyDirection = "BUY" | "SELL" | "HOLD";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyFixtureScenario".
 */
export type StrategyFixtureScenario = "SUCCESS" | "FAILURE" | "CANCELLED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderBookAction".
 */
export type Trading212DemoOrderBookAction = "PENDING" | "HISTORY" | "DETAIL";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelAttemptKind".
 */
export type ModelAttemptKind = "SETUP" | "THREAD";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoCancelState".
 */
export type Trading212DemoCancelState = "NONE" | "SUBMITTING" | "PENDING";

/**
 * Exported to JSON Schema and TypeScript, and used for renderer runtime validation.
 */
export interface IpcSchema {
  accountMutation: AccountMutation;
  accountQuery: AccountQuery;
  aggregate: Aggregate;
  alpacaPaperFill: AlpacaPaperFill;
  alpacaPaperOrder: AlpacaPaperOrder;
  alpacaPaperOrderAttempt: AlpacaPaperOrderAttempt;
  alpacaPaperOrderAttemptQuery: AlpacaPaperOrderAttemptQuery;
  alpacaPaperOrderAttemptQueryResult: AlpacaPaperOrderAttemptQueryResult;
  alpacaPaperOrderBook: AlpacaPaperOrderBook;
  alpacaPaperOrderBookQuery: AlpacaPaperOrderBookQuery;
  alpacaPaperOrderBookQueryResult: AlpacaPaperOrderBookQueryResult;
  alpacaPaperOrderBookRefresh: AlpacaPaperOrderBookRefresh;
  alpacaPaperOrderCancel: AlpacaPaperOrderCancel;
  alpacaPaperOrderReconcile: AlpacaPaperOrderReconcile;
  alpacaPaperOrderReview: AlpacaPaperOrderReview;
  alpacaPaperOrderSubmit: AlpacaPaperOrderSubmit;
  artifactExport: ArtifactExport;
  artifactQuery: ArtifactQuery;
  artifactSave: ArtifactSave;
  backtestCancel: BacktestCancel;
  backtestCompareRequest: BacktestCompareRequest;
  backtestComparison: BacktestComparison;
  backtestCurveSummary: BacktestCurveSummary;
  backtestFailure: BacktestFailure;
  backtestFieldDifference: BacktestFieldDifference;
  backtestGuardCheck: BacktestGuardCheck;
  backtestLibrary: BacktestLibrary;
  backtestManifest: BacktestManifest;
  backtestMetricComparison: BacktestMetricComparison;
  backtestMetricDelta: BacktestMetricDelta;
  backtestMetrics: BacktestMetrics;
  backtestResult: BacktestResult;
  backtestRun: BacktestRun;
  backtestRunQuery: BacktestRunQuery;
  backtestRunRequest: BacktestRunRequest;
  backtestRunSummary: BacktestRunSummary;
  capabilityQuery: CapabilityQuery;
  chatgptLogin: ChatgptLogin;
  command: CommandEnvelope;
  completeOnboarding: CompleteOnboarding;
  configureDeepseek: ConfigureDeepseek;
  contextCatalog: WorkspaceQuery;
  dataSourceProbe: DataSourceProbe;
  dataSourceQuery: DataSourceQuery;
  empty: EmptyPayload;
  equityPoint: EquityPoint;
  event: DomainEvent;
  gatewayMutation: GatewayMutation;
  localPaperState: LocalPaperState;
  marketCatalogQuery: MarketCatalogQuery;
  marketGetQuery: MarketGetQuery;
  modelQuery: ModelQuery;
  orderDraft: OrderDraft;
  orderDraftLibrary: OrderDraftLibrary;
  orderDraftQuery: OrderDraftQuery;
  orderDraftSave: OrderDraftSave;
  orderProposal: OrderProposal;
  orderProposalGenerate: OrderProposalGenerate;
  orderProposalLibrary: OrderProposalLibrary;
  orderProposalQuery: OrderProposalQuery;
  orderProposalRefresh: OrderProposalRefreshResult;
  orderProposalRefreshRequest: OrderProposalRefresh;
  paperOrderCancel: PaperOrderCancel;
  paperOrderResult: PaperOrderResult;
  paperOrderSubmit: PaperOrderSubmit;
  paperQuoteRefresh: PaperQuoteRefresh;
  paperScenarioSet: PaperScenarioSet;
  portfolioQuery: PortfolioQuery;
  providerConnect: Connect;
  providerSelection: ProviderSelection;
  researchInvocation: ResearchToolInvocation;
  researchRequest: ResearchToolRequest;
  researchResult: ResearchToolResult;
  result: ResultEnvelope;
  riskQuery: RiskQuery;
  saveRiskPolicy: SaveRiskPolicy;
  screenerAttach: ScreenerAttach;
  screenerRequest: ScreenerRequest;
  screenerSave: ScreenerSave;
  screenerUpdate: ScreenerUpdate;
  setDefaultModel: SetDefaultModel;
  setFallbackPolicy: SetFallbackPolicy;
  setOnboardingStep: SetOnboardingStep;
  strategyCancel: StrategyCancel;
  strategyDefinition: StrategyDefinition;
  strategyFailure: StrategyFailure;
  strategyLibrary: StrategyLibrary;
  strategyQuery: StrategyQuery;
  strategyRun: StrategyRun;
  strategyRunQuery: StrategyRunQuery;
  strategyRunRequest: StrategyRunRequest;
  strategySave: StrategySave;
  strategySignal: StrategySignal;
  strategyVersion: StrategyVersion;
  subscribe: Subscribe;
  threadCreate: ThreadCreate;
  threadQuery: ThreadQuery;
  timeStatus: TimeStatus;
  tradeRecord: TradeRecord;
  trading212DemoOrder: Trading212DemoOrder;
  trading212DemoOrderAttempt: Trading212DemoOrderAttempt;
  trading212DemoOrderAttemptQuery: Trading212DemoOrderAttemptQuery;
  trading212DemoOrderAttemptQueryResult: Trading212DemoOrderAttemptQueryResult;
  trading212DemoOrderBook: Trading212DemoOrderBook;
  trading212DemoOrderBookQuery: Trading212DemoOrderBookQuery;
  trading212DemoOrderBookQueryResult: Trading212DemoOrderBookQueryResult;
  trading212DemoOrderBookRefresh: Trading212DemoOrderBookRefresh;
  trading212DemoOrderCancel: Trading212DemoOrderCancel;
  trading212DemoOrderSubmit: Trading212DemoOrderSubmit;
  turnCancel: TurnCancel;
  turnRetry: TurnRetry;
  turnStart: TurnStart;
  verifyRoute: VerifyRoute;
  watchlistCreate: WatchlistCreate;
  watchlistDelete: WatchlistDelete;
  watchlistInstrumentMutation: WatchlistInstrumentMutation;
  watchlistRename: WatchlistRename;
  workspaceOpen: OpenWorkspace;
  workspaceQuery: WorkspaceQuery;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AccountMutation".
 */
export interface AccountMutation {
  connectionId: string;
  expectedStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AccountQuery".
 */
export interface AccountQuery {
  connectionId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Aggregate".
 */
export interface Aggregate {
  aggregateId: string;
  aggregateType: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperFill".
 */
export interface AlpacaPaperFill {
  activityId: string;
  executedAt: string;
  instrumentId?: string | null;
  observedAt: string;
  price: string;
  providerOrderId: string;
  quantity: string;
  side: string;
  source: AlpacaPaperFillSource;
  symbol: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrder".
 */
export interface AlpacaPaperOrder {
  cancelError?: string | null;
  cancelIdempotencyKey?: string | null;
  cancelState: AlpacaPaperCancelState;
  clientOrderId: string;
  filledQuantity: string;
  instrumentId?: string | null;
  observedAt: string;
  orderType: string;
  origin: AlpacaPaperOrderOrigin;
  providerOrderId: string;
  providerStatus: string;
  providerUpdatedAt?: string | null;
  quantity?: string | null;
  remainingQuantity?: string | null;
  side: string;
  submittedAt: string;
  symbol: string;
  timeInForce: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderAttempt".
 */
export interface AlpacaPaperOrderAttempt {
  attemptId: string;
  clientOrderId: string;
  connectionId: string;
  createdAt: string;
  errorCode?: string | null;
  proposalHash: string;
  proposalId: string;
  providerOrderId?: string | null;
  providerStatus?: string | null;
  reason: string;
  remoteAccountId: string;
  state: AlpacaPaperOrderAttemptState;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderAttemptQuery".
 */
export interface AlpacaPaperOrderAttemptQuery {
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderAttemptQueryResult".
 */
export interface AlpacaPaperOrderAttemptQueryResult {
  attempt?: AlpacaPaperOrderAttempt | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderBook".
 */
export interface AlpacaPaperOrderBook {
  connectionId: string;
  /**
   * @maxItems 1000
   */
  fills: AlpacaPaperFill[];
  lastSuccessfulSyncAt?: string | null;
  observedAt: string;
  /**
   * @maxItems 500
   */
  orders: AlpacaPaperOrder[];
  reason?: string | null;
  remoteAccountId: string;
  stateVersion: string;
  status: AlpacaPaperOrderBookStatus;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderBookQuery".
 */
export interface AlpacaPaperOrderBookQuery {
  connectionId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderBookQueryResult".
 */
export interface AlpacaPaperOrderBookQueryResult {
  book?: AlpacaPaperOrderBook | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderBookRefresh".
 */
export interface AlpacaPaperOrderBookRefresh {
  connectionId: string;
  expectedConnectionStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderCancel".
 */
export interface AlpacaPaperOrderCancel {
  confirmed: boolean;
  connectionId: string;
  expectedBookStateVersion: string;
  expectedConnectionStateVersion: string;
  idempotencyKey: string;
  providerOrderId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderReconcile".
 */
export interface AlpacaPaperOrderReconcile {
  connectionId: string;
  expectedConnectionStateVersion: string;
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderReview".
 */
export interface AlpacaPaperOrderReview {
  connectionId: string;
  expectedConnectionStateVersion: string;
  providerOrderId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AlpacaPaperOrderSubmit".
 */
export interface AlpacaPaperOrderSubmit {
  confirmedPaperOrder: boolean;
  connectionId: string;
  expectedConnectionStateVersion: string;
  expectedProposalStateVersion: string;
  idempotencyKey: string;
  proposalHash: string;
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactExport".
 */
export interface ArtifactExport {
  artifactId: string;
  destinationPath?: string;
  fileName?: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactQuery".
 */
export interface ArtifactQuery {
  artifactId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactSave".
 */
export interface ArtifactSave {
  itemId: string;
  kind: ArtifactKind;
  threadId: string;
  title: string;
  turnId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestCancel".
 */
export interface BacktestCancel {
  expectedStateVersion: string;
  runId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestCompareRequest".
 */
export interface BacktestCompareRequest {
  leftRunId: string;
  rightRunId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestComparison".
 */
export interface BacktestComparison {
  historicalSimulation: boolean;
  /**
   * @maxItems 64
   */
  inputDifferences: BacktestFieldDifference[];
  left: BacktestRun;
  leftCurve: BacktestCurveSummary;
  /**
   * @minItems 1
   * @maxItems 8
   */
  limitations:
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  /**
   * @maxItems 64
   */
  manifestDifferences: BacktestFieldDifference[];
  metrics: BacktestMetricComparison;
  right: BacktestRun;
  rightCurve: BacktestCurveSummary;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestFieldDifference".
 */
export interface BacktestFieldDifference {
  field: string;
  left: string;
  right: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestRun".
 */
export interface BacktestRun {
  barInterval: string;
  commission: string;
  createdAt: string;
  datasetId: string;
  endAt: string;
  failure?: BacktestFailure | null;
  fixtureLabel?: string | null;
  instrumentId: string;
  observedAt: string;
  /**
   * @maxItems 32
   */
  parameters?: StrategyParameter[];
  portfolioSeed?: string | null;
  requestHash: string;
  result?: BacktestResult | null;
  runId: string;
  slippage: string;
  startAt: string;
  startingCash: string;
  state: BacktestRunState;
  stateVersion: string;
  strategyHash: string;
  strategyVersionId: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestFailure".
 */
export interface BacktestFailure {
  code: string;
  reason: string;
  /**
   * @maxItems 4
   */
  remediation?: [] | [string] | [string, string] | [string, string, string] | [string, string, string, string];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyParameter".
 */
export interface StrategyParameter {
  name: string;
  value: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestResult".
 */
export interface BacktestResult {
  /**
   * @minItems 1
   * @maxItems 5000
   */
  equityCurve: [EquityPoint, ...EquityPoint[]];
  historicalSimulation: boolean;
  /**
   * @minItems 1
   * @maxItems 8
   */
  limitations:
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  manifest: BacktestManifest;
  metrics: BacktestMetrics;
  resultHash: string;
  /**
   * @maxItems 5000
   */
  trades: TradeRecord[];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "EquityPoint".
 */
export interface EquityPoint {
  drawdown: string;
  equity: string;
  observedAt: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestManifest".
 */
export interface BacktestManifest {
  adjustmentMethod: string;
  commission: string;
  commissionModel: string;
  dataProvider: string;
  datasetHash: string;
  datasetId: string;
  endAt: string;
  engineVersion: string;
  /**
   * @minItems 6
   * @maxItems 6
   */
  guardChecks: [
    BacktestGuardCheck,
    BacktestGuardCheck,
    BacktestGuardCheck,
    BacktestGuardCheck,
    BacktestGuardCheck,
    BacktestGuardCheck
  ];
  manifestHash: string;
  marketCalendarVersion: string;
  /**
   * @maxItems 32
   */
  parameters: StrategyParameter[];
  retrievedAt: string;
  runtimeVersion: string;
  seed: string;
  slippage: string;
  slippageModel: string;
  startAt: string;
  startingCash: string;
  strategyHash: string;
  strategyVersion: string;
  timezone: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestGuardCheck".
 */
export interface BacktestGuardCheck {
  detail: string;
  name: string;
  state: BacktestGuardState;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestMetrics".
 */
export interface BacktestMetrics {
  maxDrawdown: string;
  profitFactor: string;
  return: string;
  sharpe: string;
  sortino: string;
  tradeCount: number;
  turnover: string;
  winRate: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TradeRecord".
 */
export interface TradeRecord {
  commission: string;
  grossValue: string;
  instrumentId: string;
  observedAt: string;
  price: string;
  quantity: string;
  realizedPnl: string;
  side: string;
  slippage: string;
  tradeId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestCurveSummary".
 */
export interface BacktestCurveSummary {
  endAt: string;
  endEquity: string;
  maxDrawdown: string;
  pointCount: number;
  startAt: string;
  startEquity: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestMetricComparison".
 */
export interface BacktestMetricComparison {
  maxDrawdown: BacktestMetricDelta;
  profitFactor: BacktestMetricDelta;
  return: BacktestMetricDelta;
  sharpe: BacktestMetricDelta;
  sortino: BacktestMetricDelta;
  tradeCount: BacktestMetricDelta;
  turnover: BacktestMetricDelta;
  winRate: BacktestMetricDelta;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestMetricDelta".
 */
export interface BacktestMetricDelta {
  difference: string;
  left: string;
  right: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestLibrary".
 */
export interface BacktestLibrary {
  /**
   * @maxItems 256
   */
  runs: BacktestRunSummary[];
  stateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestRunSummary".
 */
export interface BacktestRunSummary {
  barInterval: string;
  datasetId: string;
  endAt: string;
  instrumentId: string;
  requestHash: string;
  resultHash?: string | null;
  runId: string;
  startAt: string;
  state: BacktestRunState;
  strategyHash: string;
  strategyVersionId: string;
  tradeCount?: number | null;
  updatedAt: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestRunQuery".
 */
export interface BacktestRunQuery {
  runId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "BacktestRunRequest".
 */
export interface BacktestRunRequest {
  barInterval: string;
  commission: string;
  datasetId: string;
  endAt: string;
  expectedStrategyHash?: string | null;
  fixtureScenario?: BacktestFixtureScenario | null;
  instrumentId: string;
  /**
   * @maxItems 32
   */
  parameters?: StrategyParameter[];
  portfolioSeed?: string | null;
  slippage: string;
  startAt: string;
  startingCash: string;
  strategyVersionId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "CapabilityQuery".
 */
export interface CapabilityQuery {
  accountId?: string;
  agentMode: AgentMode;
  /**
   * @maxItems 32
   */
  attachedContexts?: ThreadContextRef[];
  executionContext: ExecutionContext;
  requestedLevel?: CapabilityLevel;
  requestedTool?: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadContextRef".
 */
export interface ThreadContextRef {
  hash: string;
  id: string;
  kind: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ChatgptLogin".
 */
export interface ChatgptLogin {
  action: ChatgptLoginAction;
  expectedStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "CommandEnvelope".
 */
export interface CommandEnvelope {
  command: string;
  payload: unknown;
  requestId: string;
  schemaVersion: 1;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "CompleteOnboarding".
 */
export interface CompleteOnboarding {
  expectedStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ConfigureDeepseek".
 */
export interface ConfigureDeepseek {
  expectedStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "WorkspaceQuery".
 */
export interface WorkspaceQuery {
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DataSourceProbe".
 */
export interface DataSourceProbe {
  expectedStateVersion: string;
  sourceId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DataSourceQuery".
 */
export interface DataSourceQuery {
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "EmptyPayload".
 */
export interface EmptyPayload {}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DomainEvent".
 */
export interface DomainEvent {
  aggregateId: string;
  aggregateType:
    | "workspace"
    | "account"
    | "model-gateway"
    | "model"
    | "risk"
    | "thread"
    | "trading212-demo-order-attempt"
    | "trading212-demo-order-book"
    | "alpaca-paper-order-attempt"
    | "alpaca-paper-order-book";
  eventId: string;
  eventType:
    | "workspace.opened"
    | "account.health.changed"
    | "model.gateway.changed"
    | "model.provider.changed"
    | "model.provider_attempt.changed"
    | "risk.policy.changed"
    | "thread.created"
    | "thread.updated"
    | "trading212.demo.order.attempt.changed"
    | "trading212.demo.order.book.changed"
    | "alpaca.paper.order.attempt.changed"
    | "alpaca.paper.order.book.changed";
  occurredAt: string;
  payload: DomainProjection;
  schemaVersion: 1;
  sequence: number;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "GatewayState".
 */
export interface GatewayState {
  desiredRunning: boolean;
  discoveredModelCount: number;
  endpoint: "http://127.0.0.1:8317";
  errorCode?: string | null;
  installed: boolean;
  lastProbeAt?: string | null;
  modelAvailable: boolean;
  nextRetryAt?: string | null;
  pinnedVersion: "7.2.155";
  restartAttempts: number;
  stateVersion: string;
  status: GatewayStatus;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelState".
 */
export interface ModelState {
  /**
   * @maxItems 100
   */
  attempts: ModelAttempt[];
  automaticFallback?: boolean;
  chatgpt: ModelProviderState;
  currentRoute?: ModelRoute | null;
  deepseek: ModelProviderState;
  defaultRoute?: ModelSelection | null;
  fallbackPolicyVersion?: number;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelAttempt".
 */
export interface ModelAttempt {
  attemptId: string;
  endedAt: string;
  errorCategory?: string | null;
  kind?: "SETUP" | "THREAD";
  modelId?: string | null;
  outcome: ModelAttemptOutcome;
  provider: ModelProvider;
  quota?: ModelQuota | null;
  startedAt: string;
  thinkingType?: ThinkingType | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelQuota".
 */
export interface ModelQuota {
  remaining?: number | null;
  resetAt?: string | null;
  retryAfterSeconds?: number | null;
  window?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelProviderState".
 */
export interface ModelProviderState {
  configured: boolean;
  errorCode?: string | null;
  lastVerifiedAt?: string | null;
  provider: ModelProvider;
  /**
   * @maxItems 100
   */
  routes: ModelRoute[];
  status: ModelHealth;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelRoute".
 */
export interface ModelRoute {
  modelId: string;
  provider: ModelProvider;
  thinkingType?: ThinkingType | null;
  verifiedAt?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelSelection".
 */
export interface ModelSelection {
  modelId: string;
  provider: ModelProvider;
  thinkingType?: ThinkingType | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Workspace".
 */
export interface Workspace {
  baseCurrency: string;
  createdAt: string;
  lastOpenedAt: string;
  name: string;
  path: string;
  storageSchemaVersion: number;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AccountConnection".
 */
export interface AccountConnection {
  connectionId: string;
  connectionState: ConnectionState;
  createdAt: string;
  data?: AccountData | null;
  environment: string;
  health: AccountHealth;
  label: string;
  lastPrivateStreamEventAt?: string | null;
  lastSuccessfulSync?: string | null;
  permissions: PermissionReview;
  providerId: string;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AccountData".
 */
export interface AccountData {
  accountType: string;
  balances: Balance[];
  buyingPower?: string | null;
  capabilities: string[];
  currency?: string | null;
  limitations: string[];
  openOrders: OpenOrder[];
  positions: Position[];
  remoteAccountId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Balance".
 */
export interface Balance {
  asset: string;
  available: string;
  inPies?: string | null;
  locked?: string | null;
  reserved?: string | null;
  restrictedAvailable?: string | null;
  total?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OpenOrder".
 */
export interface OpenOrder {
  brokerOrderId: string;
  currency?: string | null;
  filledQuantity?: string | null;
  filledValue?: string | null;
  instrumentId?: string | null;
  kind?: "NORMAL" | "TPSL" | "PLAN" | null;
  limitPrice?: string | null;
  notional?: string | null;
  quantity?: string | null;
  side: string;
  status: string;
  symbol: string;
  triggerPrice?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Position".
 */
export interface Position {
  averageEntryPrice?: string | null;
  instrumentCurrency?: string | null;
  instrumentId?: string | null;
  marketValue?: string | null;
  marketValueCurrency?: string | null;
  quantity: string;
  symbol: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "AccountHealth".
 */
export interface AccountHealth {
  arming: string;
  authentication: string;
  connection: string;
  credential: string;
  executionEligibility: string;
  privateStream: string;
  reason: string;
  reconciliation: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PermissionReview".
 */
export interface PermissionReview {
  acknowledged: boolean;
  detected: string[];
  forbidden: string[];
  ipAllowList?: string[] | null;
  ipAllowListStatus: string;
  scope: "VERIFIED" | "UNVERIFIED";
  unsupported: string[];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "RiskPolicyState".
 */
export interface RiskPolicyState {
  configured: boolean;
  hardRules: HardSafetyRule[];
  onboardingCompleted: boolean;
  onboardingStep: number;
  policy: RiskPolicy;
  policyVersion: number;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "HardSafetyRule".
 */
export interface HardSafetyRule {
  description: string;
  id: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "RiskPolicy".
 */
export interface RiskPolicy {
  liveInactivityTimeoutMinutes: number;
  marketOrdersEnabled: boolean;
  maxDailyRealizedLoss: string | null;
  maxDailyTradedNotional: string | null;
  maxOrderNotional: string | null;
  maxSingleInstrumentExposurePercent: string | null;
  staleQuoteThresholdSeconds: number;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Thread".
 */
export interface Thread {
  accountId?: string | null;
  codexThreadId?: string | null;
  createdAt: string;
  defaultAgentMode: AgentMode;
  defaultExecutionContext: ExecutionContext;
  linkedContexts: ThreadContextRef[];
  model?: ThreadModel | null;
  stateVersion: string;
  status: ThreadStatus;
  threadId: string;
  title: string;
  turns?: ThreadTurn[];
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadModel".
 */
export interface ThreadModel {
  modelId: string;
  provider: string;
  thinkingType?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadTurn".
 */
export interface ThreadTurn {
  cancelRequestedAt?: string | null;
  completedAt?: string | null;
  items: ThreadItem[];
  providerAttempts: ThreadProviderAttempt[];
  snapshot: TurnSnapshot;
  startedAt: string;
  status: TurnStatus;
  turnId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadItem".
 */
export interface ThreadItem {
  completedAt?: string | null;
  content: string;
  itemId: string;
  itemType: string;
  researchResult?: ResearchToolResult | null;
  sourceId?: string | null;
  startedAt: string;
  status: ItemStatus;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchToolResult".
 */
export interface ResearchToolResult {
  accountId?: string;
  /**
   * @maxItems 32
   */
  contextRefs: ThreadContextRef[];
  marker: string;
  payload: ResearchToolPayload;
  requestHash: string;
  resultId: string;
  sourceId: string;
  toolId: ResearchToolId;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchToolPayload".
 */
export interface ResearchToolPayload {
  /**
   * @maxItems 8
   */
  artifactRefs?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  conclusion?: string | null;
  /**
   * @maxItems 8
   */
  datasetRefs?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  /**
   * @maxItems 8
   */
  evidence?:
    | []
    | [ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance, ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance, ResearchProvenance, ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance, ResearchProvenance, ResearchProvenance, ResearchProvenance]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ];
  /**
   * @maxItems 8
   */
  findings?:
    | []
    | [ResearchFinding]
    | [ResearchFinding, ResearchFinding]
    | [ResearchFinding, ResearchFinding, ResearchFinding]
    | [ResearchFinding, ResearchFinding, ResearchFinding, ResearchFinding]
    | [ResearchFinding, ResearchFinding, ResearchFinding, ResearchFinding, ResearchFinding]
    | [ResearchFinding, ResearchFinding, ResearchFinding, ResearchFinding, ResearchFinding, ResearchFinding]
    | [
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding
      ]
    | [
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding,
        ResearchFinding
      ];
  fixtureLabel?: string;
  focus?: ResearchFocus | null;
  /**
   * @maxItems 8
   */
  instrumentRefs?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  /**
   * @maxItems 8
   */
  limitations?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  /**
   * @maxItems 8
   */
  marketSnapshotRefs?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  /**
   * @maxItems 8
   */
  orderRefs?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  reason: string;
  /**
   * @maxItems 8
   */
  scenarios?:
    | []
    | [ResearchScenario]
    | [ResearchScenario, ResearchScenario]
    | [ResearchScenario, ResearchScenario, ResearchScenario]
    | [ResearchScenario, ResearchScenario, ResearchScenario, ResearchScenario]
    | [ResearchScenario, ResearchScenario, ResearchScenario, ResearchScenario, ResearchScenario]
    | [ResearchScenario, ResearchScenario, ResearchScenario, ResearchScenario, ResearchScenario, ResearchScenario]
    | [
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario
      ]
    | [
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario,
        ResearchScenario
      ];
  /**
   * @maxItems 2
   */
  spotVenues?: [] | [ResearchSpotVenue] | [ResearchSpotVenue, ResearchSpotVenue];
  state: ResearchResultState;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchProvenance".
 */
export interface ResearchProvenance {
  freshness: ResearchFreshness;
  limitation?: string | null;
  provider: string;
  providerTimestamp?: string;
  quality: ResearchQuality;
  receivedTimestamp: string;
  sourceId: string;
  status: DataSourceStatus;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchFinding".
 */
export interface ResearchFinding {
  detail: string;
  title: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchScenario".
 */
export interface ResearchScenario {
  detail: string;
  title: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchSpotVenue".
 */
export interface ResearchSpotVenue {
  ask?: string | null;
  bid?: string | null;
  depth?: string | null;
  limitation?: string | null;
  provenance: ResearchProvenance;
  quoteAge?: string | null;
  selected?: boolean;
  spread?: string | null;
  state: ResearchResultState;
  venue: ResearchSpotVenueId;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadProviderAttempt".
 */
export interface ThreadProviderAttempt {
  attemptId: string;
  endedAt?: string | null;
  errorCode?: string | null;
  modelId: string;
  outcome: string;
  provider: string;
  startedAt: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TurnSnapshot".
 */
export interface TurnSnapshot {
  accountEnvironment?: string | null;
  accountId?: string | null;
  agentMode: AgentMode;
  attachedContexts: ThreadContextRef[];
  capabilityLevel: string;
  executionContext: ExecutionContext;
  model?: ThreadModel | null;
  startedAt: string;
  turnId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderAttempt".
 */
export interface Trading212DemoOrderAttempt {
  attemptId: string;
  connectionId: string;
  createdAt: string;
  errorCode?: string | null;
  proposalHash: string;
  proposalId: string;
  providerOrderId?: string | null;
  providerStatus?: string | null;
  reason: string;
  remoteAccountId: string;
  state: Trading212DemoOrderAttemptState;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderBook".
 */
export interface Trading212DemoOrderBook {
  connectionId: string;
  environment: "DEMO";
  historyComplete: boolean;
  /**
   * @maxItems 100
   */
  historyCursors: string[];
  historyPageCount: number;
  historyStarted: boolean;
  lastSuccessfulSyncAt?: string | null;
  nextPagePath?: string | null;
  observedAt: string;
  /**
   * @maxItems 5000
   */
  orders: Trading212DemoOrder[];
  rateLimits: Trading212DemoRateLimits;
  reason?: string | null;
  remoteAccountId: string;
  stateVersion: string;
  status: Trading212DemoOrderBookStatus;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrder".
 */
export interface Trading212DemoOrder {
  attemptId?: string | null;
  cancelError?: string | null;
  cancelIdempotencyKey?: string | null;
  cancelState?: "NONE" | "SUBMITTING" | "PENDING";
  currency?: string | null;
  filledQuantity?: string | null;
  filledValue?: string | null;
  normalizedStatus: Trading212DemoNormalizedOrderStatus;
  observedAt: string;
  orderType: string;
  origin: Trading212DemoOrderOrigin;
  pending: boolean;
  providerOrderId: string;
  providerStatus: string;
  providerUpdatedAt?: string | null;
  quantity?: string | null;
  remainingQuantity?: string | null;
  side: string;
  submittedAt: string;
  symbol: string;
  timeInForce: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoRateLimits".
 */
export interface Trading212DemoRateLimits {
  cancelOrderRetryAt?: string | null;
  historyRetryAt?: string | null;
  orderDetailRetryAt?: string | null;
  pendingOrdersRetryAt?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "GatewayMutation".
 */
export interface GatewayMutation {
  action: GatewayAction;
  expectedStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperState".
 */
export interface LocalPaperState {
  accountId: string;
  accountLabel: string;
  /**
   * @maxItems 512
   */
  balances: LocalPaperBalance[];
  cash: LocalPaperMoney;
  disclosure: string;
  environment: string;
  equity: LocalPaperMoney;
  eventCursor: number;
  /**
   * @maxItems 1024
   */
  events?: LocalPaperEvent[];
  exposure: LocalPaperMoney;
  /**
   * @maxItems 512
   */
  fills: LocalPaperFill[];
  /**
   * @maxItems 512
   */
  openOrders: LocalPaperOrder[];
  /**
   * @maxItems 512
   */
  orders?: LocalPaperOrder[];
  /**
   * @maxItems 512
   */
  positions: LocalPaperPosition[];
  profile: LocalPaperProfile;
  providerId: string;
  realizedPnl: LocalPaperMoney;
  reservedCash: LocalPaperMoney;
  stateVersion: string;
  unrealizedPnl: LocalPaperMoney;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperBalance".
 */
export interface LocalPaperBalance {
  asset: string;
  available: string;
  reserved: string;
  total: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperMoney".
 */
export interface LocalPaperMoney {
  currency: string;
  value: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperEvent".
 */
export interface LocalPaperEvent {
  eventId: string;
  fillId?: string | null;
  kind: LocalPaperEventKind;
  occurredAt: string;
  orderId: string;
  sequence: number;
  stateVersion: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperFill".
 */
export interface LocalPaperFill {
  currency: string;
  fillId: string;
  instrumentId: string;
  observedAt: string;
  orderId: string;
  price: string;
  quantity: string;
  side: OrderSide;
  value: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperOrder".
 */
export interface LocalPaperOrder {
  averageFillPrice?: string;
  cancelIdempotencyKey?: string | null;
  createdAt: string;
  eventSequence?: number | null;
  filledQuantity: string;
  idempotencyKey?: string | null;
  instrumentId: string;
  limitPrice?: string;
  orderId: string;
  orderType: OrderType;
  proposalHash: string;
  proposalId: string;
  quantityType?: OrderQuantityType | null;
  quote?: LocalPaperQuote | null;
  remainingQuantity: string;
  requestedQuantity: string;
  side: OrderSide;
  state: LocalPaperOrderState;
  timeInForce?: TimeInForce | null;
  updatedAt: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperQuote".
 */
export interface LocalPaperQuote {
  currency: string;
  freshness: string;
  instrumentId: string;
  observedAt: string;
  price: string;
  quoteId: string;
  scenarioId: string;
  source: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperPosition".
 */
export interface LocalPaperPosition {
  averageEntryPrice: string;
  currency: string;
  instrumentId: string;
  marketValue?: string;
  quantity: string;
  unrealizedPnl?: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "LocalPaperProfile".
 */
export interface LocalPaperProfile {
  baseCurrency: string;
  engineVersion: string;
  feePolicy?: string;
  fillPolicy?: string;
  quoteFreshness?: string | null;
  quoteObservedAt?: string | null;
  quotePrice?: string;
  quoteSource: string;
  scenarioId: string;
  scenarioSeed?: string;
  scenarioVersion?: string;
  slippagePolicy?: string;
  startingCash: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketCatalogQuery".
 */
export interface MarketCatalogQuery {
  query?: string;
  tier?: "CENSUS" | "WARM" | "HOT" | "COLD";
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketGetQuery".
 */
export interface MarketGetQuery {
  instrumentId: string;
  tier: MarketTier;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelQuery".
 */
export interface ModelQuery {
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderDraft".
 */
export interface OrderDraft {
  draftId: string;
  draftVersion: number;
  fields: OrderDraftFields;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderDraftFields".
 */
export interface OrderDraftFields {
  accountId?: string | null;
  clientLabel?: string | null;
  environment: ExecutionContext;
  instrumentId: string;
  limitPrice?: string;
  maximumSpend?: string;
  orderType: OrderType;
  quantity: OrderQuantity;
  side: OrderSide;
  timeInForce: TimeInForce;
  venue: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderQuantity".
 */
export interface OrderQuantity {
  type: OrderQuantityType;
  value: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderDraftLibrary".
 */
export interface OrderDraftLibrary {
  /**
   * @maxItems 256
   */
  drafts: OrderDraftSummary[];
  stateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderDraftSummary".
 */
export interface OrderDraftSummary {
  draftId: string;
  draftVersion: number;
  environment: ExecutionContext;
  instrumentId: string;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderDraftQuery".
 */
export interface OrderDraftQuery {
  draftId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderDraftSave".
 */
export interface OrderDraftSave {
  draftId?: string | null;
  expectedStateVersion?: string | null;
  fields: OrderDraftFields;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposal".
 */
export interface OrderProposal {
  createdAt: string;
  draftId: string;
  draftVersion: number;
  estimatedNotional?: string;
  estimatedNotionalCurrency?: string | null;
  estimatedNotionalReason?: string | null;
  fields: OrderDraftFields;
  /**
   * @maxItems 32
   */
  history: OrderProposalHistoryEntry[];
  invalidationReason?: string | null;
  marketReferenceReason: string;
  marketSnapshotId?: string | null;
  marketStatus: MarketDataStatus;
  policyReferenceReason: string;
  policyStateVersion?: string | null;
  policyStatus: ProposalReferenceStatus;
  policyVersion?: number | null;
  proposalHash: string;
  proposalId: string;
  stateVersion: string;
  status: OrderProposalStatus;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalHistoryEntry".
 */
export interface OrderProposalHistoryEntry {
  event: OrderProposalHistoryEvent;
  occurredAt: string;
  reason?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalGenerate".
 */
export interface OrderProposalGenerate {
  draftId: string;
  expectedDraftVersion: number;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalLibrary".
 */
export interface OrderProposalLibrary {
  /**
   * @maxItems 256
   */
  proposals: OrderProposalSummary[];
  stateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalSummary".
 */
export interface OrderProposalSummary {
  createdAt: string;
  draftId: string;
  draftVersion: number;
  invalidationReason?: string | null;
  proposalHash: string;
  proposalId: string;
  stateVersion: string;
  status: OrderProposalStatus;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalQuery".
 */
export interface OrderProposalQuery {
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalRefreshResult".
 */
export interface OrderProposalRefreshResult {
  invalidationReason: string;
  previousProposal: OrderProposal;
  proposal: OrderProposal;
  refreshStatus: OrderProposalRefreshStatus;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OrderProposalRefresh".
 */
export interface OrderProposalRefresh {
  expectedStateVersion: string;
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PaperOrderCancel".
 */
export interface PaperOrderCancel {
  expectedStateVersion: string;
  idempotencyKey: string;
  orderId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PaperOrderResult".
 */
export interface PaperOrderResult {
  accountId: string;
  disclosure: string;
  environment: string;
  eventSequence: number;
  fill?: LocalPaperFill | null;
  order: LocalPaperOrder;
  paperState: LocalPaperState;
  proposalHash: string;
  proposalId: string;
  proposalStateVersion: string;
  quote: LocalPaperQuote;
  stateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PaperOrderSubmit".
 */
export interface PaperOrderSubmit {
  expectedProposalStateVersion: string;
  idempotencyKey: string;
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PaperQuoteRefresh".
 */
export interface PaperQuoteRefresh {
  expectedStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PaperScenarioSet".
 */
export interface PaperScenarioSet {
  expectedStateVersion: string;
  profile: LocalPaperProfile;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioQuery".
 */
export interface PortfolioQuery {
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ProviderSelection".
 */
export interface ProviderSelection {
  environment: string;
  providerId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchToolInvocation".
 */
export interface ResearchToolInvocation {
  focus?: ResearchFocus | null;
  query: string;
  toolId: ResearchToolId;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchToolRequest".
 */
export interface ResearchToolRequest {
  accountId?: string;
  agentMode: AgentMode;
  /**
   * @maxItems 32
   */
  attachedContexts: ThreadContextRef[];
  executionContext: ExecutionContext;
  focus?: ResearchFocus | null;
  query: string;
  toolId: ResearchToolId;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "SuccessEnvelope".
 */
export interface SuccessEnvelope {
  data: ReplyData;
  ok: true;
  requestId: string;
  schemaVersion: 1;
  stateVersion?: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Snapshot".
 */
export interface Snapshot {
  aggregateId: string;
  aggregateType:
    | "workspace"
    | "account"
    | "model-gateway"
    | "model"
    | "risk"
    | "thread"
    | "trading212-demo-order-attempt"
    | "trading212-demo-order-book"
    | "alpaca-paper-order-attempt"
    | "alpaca-paper-order-book";
  lastSequence: number;
  projection: DomainProjection;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "RuntimeStatus".
 */
export interface RuntimeStatus {
  components: RuntimeComponent[];
  liveExecutionAvailable: boolean;
  modelAvailable: boolean;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "RuntimeComponent".
 */
export interface RuntimeComponent {
  id: string;
  message: string;
  status: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TimeStatus".
 */
export interface TimeStatus {
  confidence: TimeConfidence;
  monotonicMs: number;
  observedAt: string;
  providerOffsetMs?: number | null;
  reason: string;
  remediation: Remediation;
  wallClock: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Remediation".
 */
export interface Remediation {
  id: string;
  label: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "SubscriptionAck".
 */
export interface SubscriptionAck {
  afterSequence: number;
  aggregateId: string;
  aggregateType:
    | "workspace"
    | "account"
    | "model-gateway"
    | "model"
    | "risk"
    | "thread"
    | "trading212-demo-order-attempt"
    | "trading212-demo-order-book"
    | "alpaca-paper-order-attempt"
    | "alpaca-paper-order-book";
  lastSequence: number;
  replayedCount: number;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadList".
 */
export interface ThreadList {
  threads: ThreadSummary[];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadSummary".
 */
export interface ThreadSummary {
  defaultAgentMode: AgentMode;
  defaultExecutionContext: ExecutionContext;
  status: ThreadStatus;
  threadId: string;
  title: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ProviderCatalog".
 */
export interface ProviderCatalog {
  providers: ProviderDefinition[];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ProviderDefinition".
 */
export interface ProviderDefinition {
  available: boolean;
  displayName: string;
  environment: string;
  fields: ProviderField[];
  forbiddenPermissions: string[];
  helpText: string;
  optionalPermissions: string[];
  providerId: string;
  requiredPermissions: string[];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ProviderField".
 */
export interface ProviderField {
  environment: string;
  helpText: string;
  id: string;
  inputType: string;
  label: string;
  maxLength: number;
  required: boolean;
  secret: boolean;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Accounts".
 */
export interface Accounts {
  accounts: AccountConnection[];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "CapabilityDecision".
 */
export interface CapabilityDecision {
  allowedTools: ToolId[];
  executionAllowed: boolean;
  level: CapabilityLevel;
  reason?: string | null;
  /**
   * @maxItems 3
   */
  researchTools:
    | []
    | [ResearchToolDefinition]
    | [ResearchToolDefinition, ResearchToolDefinition]
    | [ResearchToolDefinition, ResearchToolDefinition, ResearchToolDefinition];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchToolDefinition".
 */
export interface ResearchToolDefinition {
  description: string;
  id: ResearchToolId;
  label: string;
  readOnly: boolean;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ContextCatalog".
 */
export interface ContextCatalog {
  /**
   * @maxItems 5
   */
  emptyStates:
    | []
    | [ContextCatalogEmptyState]
    | [ContextCatalogEmptyState, ContextCatalogEmptyState]
    | [ContextCatalogEmptyState, ContextCatalogEmptyState, ContextCatalogEmptyState]
    | [ContextCatalogEmptyState, ContextCatalogEmptyState, ContextCatalogEmptyState, ContextCatalogEmptyState]
    | [
        ContextCatalogEmptyState,
        ContextCatalogEmptyState,
        ContextCatalogEmptyState,
        ContextCatalogEmptyState,
        ContextCatalogEmptyState
      ];
  /**
   * @maxItems 256
   */
  entries: ContextCatalogEntry[];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ContextCatalogEmptyState".
 */
export interface ContextCatalogEmptyState {
  availabilityReason: string;
  kind: "instrument" | "account" | "strategy" | "backtest" | "artifact";
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ContextCatalogEntry".
 */
export interface ContextCatalogEntry {
  availabilityReason?: string;
  available: boolean;
  contextRef: ThreadContextRef;
  environment?: string;
  label: string;
  providerId?: string;
  readOnly: boolean;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DataSourceCatalog".
 */
export interface DataSourceCatalog {
  /**
   * @minItems 1
   * @maxItems 8
   */
  sources:
    | [DataSourceEntry]
    | [DataSourceEntry, DataSourceEntry]
    | [DataSourceEntry, DataSourceEntry, DataSourceEntry]
    | [DataSourceEntry, DataSourceEntry, DataSourceEntry, DataSourceEntry]
    | [DataSourceEntry, DataSourceEntry, DataSourceEntry, DataSourceEntry, DataSourceEntry]
    | [DataSourceEntry, DataSourceEntry, DataSourceEntry, DataSourceEntry, DataSourceEntry, DataSourceEntry]
    | [
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry
      ]
    | [
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry,
        DataSourceEntry
      ];
  stateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DataSourceEntry".
 */
export interface DataSourceEntry {
  availabilityReason: string;
  /**
   * @maxItems 8
   */
  capabilities:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  checkedAt?: string;
  commercialUse: string;
  configured: boolean;
  coverage: string;
  entitlement: string;
  jurisdictions: string;
  latency: string;
  observedAt?: string;
  officialUrl: string;
  probeKind: DataSourceProbeKind;
  provider: string;
  redistribution: string;
  retention: string;
  reviewedAt: string;
  sourceId: string;
  status: DataSourceStatus;
  termsUrl: string;
  verifiedAt?: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketCatalog".
 */
export interface MarketCatalog {
  availabilityReason: string;
  /**
   * @maxItems 256
   */
  instruments: Instrument[];
  query: string;
  sourceId?: string;
  status: MarketDataStatus;
  tier: MarketTier;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Instrument".
 */
export interface Instrument {
  assetClass: AssetClass;
  base?: string;
  currency: string;
  displayName: string;
  exchange?: string;
  instrumentId: string;
  /**
   * @maxItems 8
   */
  providers:
    | []
    | [InstrumentProviderMapping]
    | [InstrumentProviderMapping, InstrumentProviderMapping]
    | [InstrumentProviderMapping, InstrumentProviderMapping, InstrumentProviderMapping]
    | [InstrumentProviderMapping, InstrumentProviderMapping, InstrumentProviderMapping, InstrumentProviderMapping]
    | [
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping
      ]
    | [
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping
      ]
    | [
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping
      ]
    | [
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping,
        InstrumentProviderMapping
      ];
  quote?: string;
  symbol: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "InstrumentProviderMapping".
 */
export interface InstrumentProviderMapping {
  providerId: string;
  providerSymbol: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketDetail".
 */
export interface MarketDetail {
  adjustmentStatus: AdjustmentStatus;
  availabilityReason: string;
  /**
   * @maxItems 16
   */
  corporateActions:
    | []
    | [CorporateAction]
    | [CorporateAction, CorporateAction]
    | [CorporateAction, CorporateAction, CorporateAction]
    | [CorporateAction, CorporateAction, CorporateAction, CorporateAction]
    | [CorporateAction, CorporateAction, CorporateAction, CorporateAction, CorporateAction]
    | [CorporateAction, CorporateAction, CorporateAction, CorporateAction, CorporateAction, CorporateAction]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ]
    | [
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction,
        CorporateAction
      ];
  instrument: Instrument;
  marketState: MarketState;
  snapshot?: MarketSnapshot | null;
  sourceId?: string;
  status: MarketDataStatus;
  tier: MarketTier;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "CorporateAction".
 */
export interface CorporateAction {
  actionId: string;
  actionType: CorporateActionType;
  adjustmentStatus: AdjustmentStatus;
  announcedAt?: string;
  description: string;
  effectiveAt: string;
  instrumentId: string;
  sourceId?: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketState".
 */
export interface MarketState {
  calendarVersion?: string;
  nextClose?: string;
  nextOpen?: string;
  observedAt: string;
  providerTime?: string;
  reason: string;
  session: MarketSession;
  sourceId?: string;
  sourceStatus: MarketDataStatus;
  timeConfidence: TimeConfidence;
  venue: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketSnapshot".
 */
export interface MarketSnapshot {
  ask?: string;
  bid?: string;
  instrumentId: string;
  lastPrice?: string;
  provenance: MarketSnapshotProvenance;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "MarketSnapshotProvenance".
 */
export interface MarketSnapshotProvenance {
  entitlement: MarketEntitlement;
  freshness: MarketFreshness;
  marketSnapshotId: string;
  providerTimestamp: string;
  receivedTimestamp: string;
  source: string;
  venue?: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioSnapshot".
 */
export interface PortfolioSnapshot {
  /**
   * @maxItems 256
   */
  accounts: PortfolioAccount[];
  availabilityReason: string;
  baseCurrency: string;
  /**
   * @maxItems 512
   */
  fills?: PortfolioFill[] | null;
  /**
   * @maxItems 128
   */
  fxRoutes: FxProvenance[];
  /**
   * @maxItems 512
   */
  holdings: PortfolioHolding[];
  liveRisk: PortfolioLiveRisk;
  observedAt: string;
  /**
   * @maxItems 512
   */
  openOrders: PortfolioOrder[];
  status: PortfolioStatus;
  totals: PortfolioTotals;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioAccount".
 */
export interface PortfolioAccount {
  accountCurrency?: string | null;
  cash: PortfolioValue;
  connectionId: string;
  connectionState: ConnectionState;
  environment: string;
  equity: PortfolioValue;
  fillsCount?: number | null;
  health: AccountHealth;
  label: string;
  openOrdersCount: number;
  positionsCount: number;
  providerId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioValue".
 */
export interface PortfolioValue {
  accountCurrency?: string | null;
  accountValue?: string;
  fxProvenance?: FxProvenance | null;
  nativeCurrency?: string | null;
  nativeValue?: string;
  workspaceCurrency: string;
  workspaceValue?: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "FxProvenance".
 */
export interface FxProvenance {
  depegWarning?: string | null;
  freshness: FxFreshness;
  pairPath: string;
  providerTimestamp?: string;
  quality: FxQuality;
  rate?: string;
  receivedTimestamp: string;
  sourceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioFill".
 */
export interface PortfolioFill {
  accountLabel: string;
  asset: string;
  connectionId: string;
  fillId: string;
  health: AccountHealth;
  instrumentId?: string | null;
  observedAt: string;
  quantity: string;
  value: PortfolioValue;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioHolding".
 */
export interface PortfolioHolding {
  accountLabel: string;
  asset: string;
  connectionId: string;
  environment: string;
  health: AccountHealth;
  instrumentId?: string | null;
  observedAt: string;
  providerId: string;
  quantity?: string;
  unrealizedPnl?: PortfolioValue | null;
  value: PortfolioValue;
  venue?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioLiveRisk".
 */
export interface PortfolioLiveRisk {
  eligible: boolean;
  reason: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioOrder".
 */
export interface PortfolioOrder {
  accountLabel: string;
  asset: string;
  brokerOrderId: string;
  connectionId: string;
  currency?: string | null;
  health: AccountHealth;
  instrumentId?: string | null;
  notional?: string;
  observedAt: string;
  quantity?: string;
  side: string;
  status: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "PortfolioTotals".
 */
export interface PortfolioTotals {
  cash: PortfolioValue;
  equity: PortfolioValue;
  exposure: PortfolioValue;
  realizedPnl: PortfolioValue;
  unrealizedPnl: PortfolioValue;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Watchlist".
 */
export interface Watchlist {
  /**
   * @maxItems 256
   */
  items: WatchlistItem[];
  name: string;
  stateVersion: string;
  watchlistId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "WatchlistItem".
 */
export interface WatchlistItem {
  instrumentId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Watchlists".
 */
export interface Watchlists {
  stateVersion: string;
  /**
   * @maxItems 128
   */
  watchlists: Watchlist[];
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerResult".
 */
export interface ScreenerResult {
  /**
   * @maxItems 16
   */
  appliedConditions?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ];
  availabilityReason: string;
  candidateCount: number;
  /**
   * @maxItems 50
   */
  candidates?: ScreenerCandidate[];
  filterSpec?: FilterSpec | null;
  fixtureLabel?: string;
  focus?: ResearchFocus | null;
  /**
   * @maxItems 8
   */
  limitations?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  naturalLanguage: string;
  operation: ScreenerOperation;
  providerTimestamp?: string;
  rankSpec?: RankSpec | null;
  receivedTimestamp: string;
  revision?: string;
  /**
   * @maxItems 8
   */
  sourceIds?:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string];
  state: ScreenerResultState;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerCandidate".
 */
export interface ScreenerCandidate {
  /**
   * @maxItems 8
   */
  features?:
    | []
    | [ScreenerFeature]
    | [ScreenerFeature, ScreenerFeature]
    | [ScreenerFeature, ScreenerFeature, ScreenerFeature]
    | [ScreenerFeature, ScreenerFeature, ScreenerFeature, ScreenerFeature]
    | [ScreenerFeature, ScreenerFeature, ScreenerFeature, ScreenerFeature, ScreenerFeature]
    | [ScreenerFeature, ScreenerFeature, ScreenerFeature, ScreenerFeature, ScreenerFeature, ScreenerFeature]
    | [
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature
      ]
    | [
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature,
        ScreenerFeature
      ];
  instrumentId: string;
  limitation?: string | null;
  provenance: ScreenerProvenance;
  rank: number;
  symbol: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerFeature".
 */
export interface ScreenerFeature {
  field: ScreenerFeatureField;
  value: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerProvenance".
 */
export interface ScreenerProvenance {
  freshness: ResearchFreshness;
  providerTimestamp?: string;
  quality: ResearchQuality;
  receivedTimestamp: string;
  sourceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "FilterSpec".
 */
export interface FilterSpec {
  /**
   * @maxItems 8
   */
  predicates?:
    | []
    | [ScreenerPredicate]
    | [ScreenerPredicate, ScreenerPredicate]
    | [ScreenerPredicate, ScreenerPredicate, ScreenerPredicate]
    | [ScreenerPredicate, ScreenerPredicate, ScreenerPredicate, ScreenerPredicate]
    | [ScreenerPredicate, ScreenerPredicate, ScreenerPredicate, ScreenerPredicate, ScreenerPredicate]
    | [ScreenerPredicate, ScreenerPredicate, ScreenerPredicate, ScreenerPredicate, ScreenerPredicate, ScreenerPredicate]
    | [
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate
      ]
    | [
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate,
        ScreenerPredicate
      ];
  universe: ScreenerUniverse;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerPredicate".
 */
export interface ScreenerPredicate {
  field: ScreenerPredicateField;
  operator: ScreenerOperator;
  threshold: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "RankSpec".
 */
export interface RankSpec {
  direction: ScreenerDirection;
  field: ScreenerRankField;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerLibrary".
 */
export interface ScreenerLibrary {
  /**
   * @maxItems 128
   */
  screeners: SavedScreener[];
  stateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "SavedScreener".
 */
export interface SavedScreener {
  createdAt: string;
  definition: ScreenerDefinition;
  name: string;
  screenerId: string;
  state: ScreenerResultState;
  stateVersion: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerDefinition".
 */
export interface ScreenerDefinition {
  filterSpec: FilterSpec;
  focus?: ResearchFocus | null;
  limit: number;
  naturalLanguage: string;
  rankSpec: RankSpec;
  revision: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerAttachment".
 */
export interface ScreenerAttachment {
  /**
   * @maxItems 32
   */
  contextRefs: ThreadContextRef[];
  revision: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderAttemptQueryResult".
 */
export interface Trading212DemoOrderAttemptQueryResult {
  attempt?: Trading212DemoOrderAttempt | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderBookQueryResult".
 */
export interface Trading212DemoOrderBookQueryResult {
  book?: Trading212DemoOrderBook | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Artifact".
 */
export interface Artifact {
  artifactId: string;
  content: ArtifactContent;
  contentHash: string;
  createdAt: string;
  kind: ArtifactKind;
  provenance: ArtifactProvenance;
  stateVersion: string;
  title: string;
  updatedAt: string;
  version: number;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactContent".
 */
export interface ArtifactContent {
  researchResult?: ResearchToolResult | null;
  text: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactProvenance".
 */
export interface ArtifactProvenance {
  /**
   * @maxItems 16
   */
  datasetHashes:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ];
  itemId: string;
  /**
   * @maxItems 16
   */
  marketSnapshotHashes:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ];
  /**
   * @maxItems 16
   */
  providerAttempts:
    | []
    | [ThreadProviderAttempt]
    | [ThreadProviderAttempt, ThreadProviderAttempt]
    | [ThreadProviderAttempt, ThreadProviderAttempt, ThreadProviderAttempt]
    | [ThreadProviderAttempt, ThreadProviderAttempt, ThreadProviderAttempt, ThreadProviderAttempt]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ]
    | [
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt,
        ThreadProviderAttempt
      ];
  /**
   * @maxItems 16
   */
  relatedOrderIds:
    | []
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string, string, string]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ]
    | [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ];
  researchResultId?: string | null;
  researchToolId?: ResearchToolId | null;
  /**
   * @maxItems 16
   */
  sources:
    | []
    | [ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance, ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance, ResearchProvenance, ResearchProvenance]
    | [ResearchProvenance, ResearchProvenance, ResearchProvenance, ResearchProvenance, ResearchProvenance]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ]
    | [
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance,
        ResearchProvenance
      ];
  threadId: string;
  turnId: string;
  turnSnapshot: TurnSnapshot;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactLibrary".
 */
export interface ArtifactLibrary {
  /**
   * @maxItems 256
   */
  artifacts: ArtifactSummary[];
  stateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactSummary".
 */
export interface ArtifactSummary {
  artifactId: string;
  contentHash: string;
  createdAt: string;
  itemId: string;
  kind: ArtifactKind;
  stateVersion: string;
  threadId: string;
  title: string;
  turnId: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ArtifactExportResult".
 */
export interface ArtifactExportResult {
  artifactId: string;
  bytes: number;
  contentHash: string;
  manifestHash: string;
  path: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyLibrary".
 */
export interface StrategyLibrary {
  /**
   * @maxItems 256
   */
  runs: StrategyRunSummary[];
  stateVersion: string;
  /**
   * @maxItems 256
   */
  versions: StrategyVersion[];
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyRunSummary".
 */
export interface StrategyRunSummary {
  failureCode?: string | null;
  runId: string;
  state: StrategyRunState;
  strategyVersionId: string;
  updatedAt: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyVersion".
 */
export interface StrategyVersion {
  createdAt: string;
  definition: StrategyDefinition;
  revision: number;
  sourceHash: string;
  stateVersion: string;
  strategyId: string;
  strategyVersionId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyDefinition".
 */
export interface StrategyDefinition {
  language: string;
  name: string;
  /**
   * @maxItems 32
   */
  parameters?: StrategyParameter[];
  runtime: string;
  source: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyRun".
 */
export interface StrategyRun {
  createdAt: string;
  datasetId: string;
  endAt: string;
  failure?: StrategyFailure | null;
  fixtureLabel?: string | null;
  instrumentId: string;
  observedAt: string;
  /**
   * @maxItems 32
   */
  parameters?: StrategyParameter[];
  requestHash: string;
  runId: string;
  signal?: StrategySignal | null;
  startAt: string;
  state: StrategyRunState;
  stateVersion: string;
  strategyHash: string;
  strategyVersionId: string;
  updatedAt: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyFailure".
 */
export interface StrategyFailure {
  code: string;
  reason: string;
  /**
   * @maxItems 4
   */
  remediation?: [] | [string] | [string, string] | [string, string, string] | [string, string, string, string];
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategySignal".
 */
export interface StrategySignal {
  datasetId: string;
  desiredExposure: string;
  direction: StrategyDirection;
  instrumentId: string;
  observedAt: string;
  sourceRef: string;
  strategyHash: string;
  strategyVersionId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "FailureEnvelope".
 */
export interface FailureEnvelope {
  error: TradeXError;
  ok: false;
  requestId: string;
  schemaVersion: 1;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TradeXError".
 */
export interface TradeXError {
  blocking: boolean;
  category: string;
  code: string;
  field?: string;
  message: string;
  remediationActions: Remediation[];
  retryable: boolean;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "RiskQuery".
 */
export interface RiskQuery {
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "SaveRiskPolicy".
 */
export interface SaveRiskPolicy {
  expectedStateVersion: string;
  policy: RiskPolicyInput;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "RiskPolicyInput".
 */
export interface RiskPolicyInput {
  liveInactivityTimeoutMinutes: number;
  marketOrdersEnabled: boolean;
  maxDailyRealizedLoss: string | null;
  maxDailyTradedNotional: string | null;
  maxOrderNotional: string | null;
  maxSingleInstrumentExposurePercent: string | null;
  staleQuoteThresholdSeconds: number;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerAttach".
 */
export interface ScreenerAttach {
  revision: string;
  /**
   * @minItems 1
   * @maxItems 32
   */
  selectedInstrumentIds: [string, ...string[]];
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerRequest".
 */
export interface ScreenerRequest {
  filterSpec?: FilterSpec | null;
  focus?: ResearchFocus | null;
  limit?: number | null;
  naturalLanguage: string;
  operation: ScreenerOperation;
  rankSpec?: RankSpec | null;
  revision?: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerSave".
 */
export interface ScreenerSave {
  definition: ScreenerDefinition;
  expectedStateVersion: string;
  name: string;
  state: ScreenerResultState;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ScreenerUpdate".
 */
export interface ScreenerUpdate {
  definition: ScreenerDefinition;
  expectedStateVersion: string;
  name: string;
  screenerId: string;
  state: ScreenerResultState;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "SetDefaultModel".
 */
export interface SetDefaultModel {
  expectedStateVersion: string;
  modelId: string;
  provider: ModelProvider;
  thinkingType?: ThinkingType | null;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "SetFallbackPolicy".
 */
export interface SetFallbackPolicy {
  automaticFallback: boolean;
  expectedStateVersion: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "SetOnboardingStep".
 */
export interface SetOnboardingStep {
  expectedStateVersion: string;
  step: number;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyCancel".
 */
export interface StrategyCancel {
  runId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyQuery".
 */
export interface StrategyQuery {
  strategyVersionId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyRunQuery".
 */
export interface StrategyRunQuery {
  runId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategyRunRequest".
 */
export interface StrategyRunRequest {
  datasetId: string;
  endAt: string;
  expectedStrategyHash?: string | null;
  fixtureScenario?: StrategyFixtureScenario | null;
  instrumentId: string;
  /**
   * @maxItems 32
   */
  parameters?: StrategyParameter[];
  startAt: string;
  strategyVersionId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "StrategySave".
 */
export interface StrategySave {
  definition: StrategyDefinition;
  strategyId?: string | null;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Subscribe".
 */
export interface Subscribe {
  afterSequence: number;
  aggregateId: string;
  aggregateType: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadCreate".
 */
export interface ThreadCreate {
  accountId?: string | null;
  defaultAgentMode: AgentMode;
  defaultExecutionContext: ExecutionContext;
  expectedStateVersion?: string | null;
  linkedContexts: ThreadContextRef[];
  model?: ThreadModel | null;
  title: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ThreadQuery".
 */
export interface ThreadQuery {
  threadId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderAttemptQuery".
 */
export interface Trading212DemoOrderAttemptQuery {
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderBookQuery".
 */
export interface Trading212DemoOrderBookQuery {
  connectionId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderBookRefresh".
 */
export interface Trading212DemoOrderBookRefresh {
  action: Trading212DemoOrderBookAction;
  connectionId: string;
  expectedConnectionStateVersion: string;
  providerOrderId?: string | null;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderCancel".
 */
export interface Trading212DemoOrderCancel {
  confirmed: boolean;
  connectionId: string;
  expectedBookStateVersion: string;
  expectedConnectionStateVersion: string;
  idempotencyKey: string;
  providerOrderId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Trading212DemoOrderSubmit".
 */
export interface Trading212DemoOrderSubmit {
  confirmedDemoOrder: boolean;
  connectionId: string;
  expectedConnectionStateVersion: string;
  expectedProposalStateVersion: string;
  idempotencyKey: string;
  proposalHash: string;
  proposalId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TurnCancel".
 */
export interface TurnCancel {
  expectedStateVersion: string;
  threadId: string;
  turnId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TurnRetry".
 */
export interface TurnRetry {
  expectedStateVersion: string;
  threadId: string;
  turnId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "TurnStart".
 */
export interface TurnStart {
  accountId?: string | null;
  agentMode: AgentMode;
  /**
   * @maxItems 32
   */
  attachedContexts?: ThreadContextRef[];
  executionContext: ExecutionContext;
  expectedStateVersion: string;
  message: string;
  model?: ThreadModel | null;
  researchInvocation?: ResearchToolInvocation | null;
  researchResult?: ResearchToolResult | null;
  threadId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "VerifyRoute".
 */
export interface VerifyRoute {
  expectedStateVersion: string;
  modelId: string;
  provider: ModelProvider;
  thinkingType?: ThinkingType | null;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "WatchlistCreate".
 */
export interface WatchlistCreate {
  name: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "WatchlistDelete".
 */
export interface WatchlistDelete {
  expectedStateVersion: string;
  watchlistId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "WatchlistInstrumentMutation".
 */
export interface WatchlistInstrumentMutation {
  expectedStateVersion: string;
  instrumentId: string;
  watchlistId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "WatchlistRename".
 */
export interface WatchlistRename {
  expectedStateVersion: string;
  name: string;
  watchlistId: string;
  workspaceId: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OpenWorkspace".
 */
export interface OpenWorkspace {
  baseCurrency?: string;
  name?: string;
  path?: string;
}
