/* Generated from Rust protocol.rs. Run npm run schema:generate. */

/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ChatgptLoginAction".
 */
export type ChatgptLoginAction = "LOGIN" | "RELOGIN";
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DomainProjection".
 */
export type DomainProjection = GatewayState | ModelState | Workspace | AccountConnection | RiskPolicyState;
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
  | GatewayState
  | ModelState
  | RiskPolicyState
  | ProviderCatalog
  | ProviderDefinition
  | Accounts
  | AccountConnection
  | PermissionReview;
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
  chatgptLogin: ChatgptLogin;
  command: CommandEnvelope;
  completeOnboarding: CompleteOnboarding;
  configureDeepseek: ConfigureDeepseek;
  empty: EmptyPayload;
  event: DomainEvent;
  gatewayMutation: GatewayMutation;
  modelQuery: ModelQuery;
  providerConnect: Connect;
  providerSelection: ProviderSelection;
  result: ResultEnvelope;
  riskQuery: RiskQuery;
  saveRiskPolicy: SaveRiskPolicy;
  setDefaultModel: SetDefaultModel;
  setFallbackPolicy: SetFallbackPolicy;
  setOnboardingStep: SetOnboardingStep;
  subscribe: Subscribe;
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
 * via the `definition` "EmptyPayload".
 */
export interface EmptyPayload {}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DomainEvent".
 */
export interface DomainEvent {
  aggregateId: string;
  aggregateType: "workspace" | "account" | "model-gateway" | "model" | "risk";
  eventId: string;
  eventType:
    | "workspace.opened"
    | "account.health.changed"
    | "model.gateway.changed"
    | "model.provider.changed"
    | "model.provider_attempt.changed"
    | "risk.policy.changed";
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
  liveInactivityTimeoutMinutes?: number;
  marketOrdersEnabled?: boolean;
  maxDailyRealizedLoss?: string | null;
  maxDailyTradedNotional?: string | null;
  maxOrderNotional?: string | null;
  maxSingleInstrumentExposurePercent?: string | null;
  staleQuoteThresholdSeconds?: number;
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
  aggregateType: "workspace" | "account" | "model-gateway" | "model" | "risk";
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
  aggregateType: "workspace" | "account" | "model-gateway" | "model" | "risk";
  lastSequence: number;
  replayedCount: number;
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
  policy: RiskPolicy;
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
 * via the `definition` "Subscribe".
 */
export interface Subscribe {
  afterSequence: number;
  aggregateId: string;
  aggregateType: string;
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
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "WorkspaceQuery".
 */
export interface WorkspaceQuery {
  workspaceId: string;
}
