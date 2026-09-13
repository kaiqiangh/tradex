/* Generated from Rust protocol.rs. Run npm run schema:generate. */

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
export type DomainProjection = GatewayState | ModelState | Workspace | AccountConnection | RiskPolicyState | Thread;
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
 * via the `definition` "GatewayAction".
 */
export type GatewayAction = "LAUNCH" | "PROBE" | "RESTART" | "STOP";
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
 * Data-plane tools are intentionally separate from financial authority IDs.
 *
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchToolId".
 */
export type ResearchToolId = "public_market_read" | "account_read" | "historical_simulation";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResearchResultState".
 */
export type ResearchResultState = "UNAVAILABLE";
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
  | ResearchToolResult;
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
 * via the `definition` "DataSourceStatus".
 */
export type DataSourceStatus = "AVAILABLE" | "UNAVAILABLE" | "BLOCKED_EXTERNAL" | "UNVERIFIED";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ModelAttemptKind".
 */
export type ModelAttemptKind = "SETUP" | "THREAD";

/**
 * Exported to JSON Schema and TypeScript, and used for renderer runtime validation.
 */
export interface IpcSchema {
  accountMutation: AccountMutation;
  accountQuery: AccountQuery;
  aggregate: Aggregate;
  capabilityQuery: CapabilityQuery;
  chatgptLogin: ChatgptLogin;
  command: CommandEnvelope;
  completeOnboarding: CompleteOnboarding;
  configureDeepseek: ConfigureDeepseek;
  contextCatalog: WorkspaceQuery;
  dataSourceProbe: DataSourceProbe;
  dataSourceQuery: DataSourceQuery;
  empty: EmptyPayload;
  event: DomainEvent;
  gatewayMutation: GatewayMutation;
  modelQuery: ModelQuery;
  providerConnect: Connect;
  providerSelection: ProviderSelection;
  researchInvocation: ResearchToolInvocation;
  researchRequest: ResearchToolRequest;
  researchResult: ResearchToolResult;
  result: ResultEnvelope;
  riskQuery: RiskQuery;
  saveRiskPolicy: SaveRiskPolicy;
  setDefaultModel: SetDefaultModel;
  setFallbackPolicy: SetFallbackPolicy;
  setOnboardingStep: SetOnboardingStep;
  subscribe: Subscribe;
  threadCreate: ThreadCreate;
  threadQuery: ThreadQuery;
  turnCancel: TurnCancel;
  turnRetry: TurnRetry;
  turnStart: TurnStart;
  verifyRoute: VerifyRoute;
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
  aggregateType: "workspace" | "account" | "model-gateway" | "model" | "risk" | "thread";
  eventId: string;
  eventType:
    | "workspace.opened"
    | "account.health.changed"
    | "model.gateway.changed"
    | "model.provider.changed"
    | "model.provider_attempt.changed"
    | "risk.policy.changed"
    | "thread.created"
    | "thread.updated";
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
  sourceId?: string | null;
  startedAt: string;
  status: ItemStatus;
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
 * via the `definition` "GatewayMutation".
 */
export interface GatewayMutation {
  action: GatewayAction;
  expectedStateVersion: string;
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
  query: string;
  toolId: ResearchToolId;
  workspaceId: string;
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
  reason: string;
  state: ResearchResultState;
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
  aggregateType: "workspace" | "account" | "model-gateway" | "model" | "risk" | "thread";
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
 * via the `definition` "SubscriptionAck".
 */
export interface SubscriptionAck {
  afterSequence: number;
  aggregateId: string;
  aggregateType: "workspace" | "account" | "model-gateway" | "model" | "risk" | "thread";
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
  message: string;
  remediationActions: Remediation[];
  retryable: boolean;
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
 * via the `definition` "OpenWorkspace".
 */
export interface OpenWorkspace {
  baseCurrency?: string;
  name?: string;
  path?: string;
}
