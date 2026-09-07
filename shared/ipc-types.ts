/* Generated from Rust protocol.rs. Run npm run schema:generate. */

/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DomainProjection".
 */
export type DomainProjection = Workspace | AccountConnection;
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ConnectionState".
 */
export type ConnectionState = "CONNECTING" | "REVIEW_REQUIRED" | "CONNECTED" | "FAILED" | "DISCONNECTED";
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
  | ProviderCatalog
  | ProviderDefinition
  | Accounts
  | AccountConnection
  | PermissionReview;

/**
 * Exported to JSON Schema and TypeScript, and used for renderer runtime validation.
 */
export interface IpcSchema {
  accountMutation: AccountMutation;
  accountQuery: AccountQuery;
  aggregate: Aggregate;
  command: CommandEnvelope;
  empty: EmptyPayload;
  event: DomainEvent;
  providerConnect: Connect;
  providerSelection: ProviderSelection;
  result: ResultEnvelope;
  subscribe: Subscribe;
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
 * via the `definition` "EmptyPayload".
 */
export interface EmptyPayload {}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "DomainEvent".
 */
export interface DomainEvent {
  aggregateId: string;
  aggregateType: "workspace" | "account";
  eventId: string;
  eventType: "workspace.opened" | "account.health.changed";
  occurredAt: string;
  payload: DomainProjection;
  schemaVersion: 1;
  sequence: number;
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
  total?: string | null;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "OpenOrder".
 */
export interface OpenOrder {
  brokerOrderId: string;
  filledQuantity: string;
  limitPrice?: string | null;
  notional?: string | null;
  quantity?: string | null;
  side: string;
  status: string;
  symbol: string;
}
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "Position".
 */
export interface Position {
  averageEntryPrice?: string | null;
  marketValue?: string | null;
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
  ipAllowListStatus: string;
  scope: "VERIFIED" | "UNVERIFIED";
  unsupported: string[];
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
  aggregateType: "workspace" | "account";
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
  aggregateType: "workspace" | "account";
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
 * via the `definition` "Subscribe".
 */
export interface Subscribe {
  afterSequence: number;
  aggregateId: string;
  aggregateType: string;
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
