/* Generated from Rust protocol.rs. Run npm run schema:generate. */

/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ResultEnvelope".
 */
export type ResultEnvelope = SuccessEnvelope | FailureEnvelope;
/**
 * This interface was referenced by `IpcSchema`'s JSON-Schema
 * via the `definition` "ReplyData".
 */
export type ReplyData = Workspace | Snapshot | RuntimeStatus | SubscriptionAck;

/**
 * Exported to JSON Schema and TypeScript, and used for renderer runtime validation.
 */
export interface IpcSchema {
  aggregate: Aggregate;
  command: CommandEnvelope;
  empty: EmptyPayload;
  event: DomainEvent;
  result: ResultEnvelope;
  subscribe: Subscribe;
  workspaceOpen: OpenWorkspace;
  [k: string]: unknown;
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
  aggregateType: "workspace";
  eventId: string;
  eventType: "workspace.opened";
  occurredAt: string;
  payload: Workspace;
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
  storageSchemaVersion: 1;
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
  aggregateType: "workspace";
  lastSequence: number;
  projection: Workspace;
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
  aggregateType: "workspace";
  lastSequence: number;
  replayedCount: number;
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
