import generatedValidators from '../shared/ipc-validators.js';
import type { AccountConnection, AlpacaPaperOrderAttempt, DomainEvent, DomainProjection, GatewayState, ModelState, RiskPolicyState, Snapshot, Thread, Workspace } from '../shared/ipc-types.ts';

const validators = generatedValidators as Record<string, (value: unknown) => boolean>;

export function decode<T>(definition: string, value: unknown): T {
  const validate = validators[definition];
  if (!validate || !validate(value)) throw new Error('IPC_SCHEMA_INCOMPATIBLE');
  return value as T;
}

export interface Projection<T = Workspace> {
  snapshot: Omit<Snapshot, 'projection'> & { projection: T };
  seen: Map<number, string>;
}

function projectionKind(p: DomainProjection) { return 'threadId' in p ? 'thread' : 'attemptId' in p ? 'alpaca-paper-order-attempt' : 'connectionId' in p ? 'account' : 'pinnedVersion' in p ? 'model-gateway' : 'chatgpt' in p ? 'model' : 'hardRules' in p ? 'risk' : 'workspace'; }

function snapshotOf<T>(value: unknown, kind: 'workspace' | 'account' | 'model-gateway' | 'model' | 'risk' | 'thread' | 'alpaca-paper-order-attempt'): Projection<T> {
  const snapshot = decode<Snapshot>('Snapshot', value);
  const p = snapshot.projection;
  const id = 'threadId' in p ? p.threadId : 'attemptId' in p ? p.attemptId : 'connectionId' in p ? p.connectionId : p.workspaceId;
  if (snapshot.aggregateType !== kind || snapshot.aggregateId !== id
      || projectionKind(p) !== kind) throw new Error('IPC_IDENTITY_CONFLICT');
  return { snapshot: snapshot as Projection<T>['snapshot'], seen: new Map() };
}
export function fromSnapshot(value: unknown): Projection { return snapshotOf(value, 'workspace'); }
export function fromAccountSnapshot(value: unknown): Projection<AccountConnection> { return snapshotOf(value, 'account'); }

export function fromGatewaySnapshot(value: unknown): Projection<GatewayState> { return snapshotOf(value, 'model-gateway'); }
export function fromModelSnapshot(value: unknown): Projection<ModelState> { return snapshotOf(value, 'model'); }
export function fromRiskSnapshot(value: unknown): Projection<RiskPolicyState> { return snapshotOf(value, 'risk'); }
export function fromThreadSnapshot(value: unknown): Projection<Thread> { return snapshotOf(value, 'thread'); }
export function fromAlpacaPaperAttemptSnapshot(value: unknown): Projection<AlpacaPaperOrderAttempt> { return snapshotOf(value, 'alpaca-paper-order-attempt'); }

export function applyEvent<T extends DomainProjection>(current: Projection<T>, value: unknown): Projection<T> {
  const event = decode<DomainEvent>('DomainEvent', value);
  const p = event.payload;
  const old = current.snapshot.projection;
  const thread = 'threadId' in p;
  const attempt = 'attemptId' in p;
  const account = 'connectionId' in p && !attempt;
  const id = thread ? p.threadId : attempt ? p.attemptId : account ? p.connectionId : p.workspaceId;
  const kind = projectionKind(p);
  const eventTypeValid = thread
    ? event.eventType === 'thread.created' || event.eventType === 'thread.updated'
    : attempt
    ? event.eventType === 'alpaca.paper.order.attempt.changed'
    : account
    ? event.eventType === 'account.health.changed'
    : kind === 'model-gateway'
      ? event.eventType === 'model.gateway.changed'
      : kind === 'model'
      ? event.eventType === 'model.provider.changed' || event.eventType === 'model.provider_attempt.changed'
        : kind === 'risk'
          ? event.eventType === 'risk.policy.changed'
      : event.eventType === 'workspace.opened';
  if (event.aggregateId !== current.snapshot.aggregateId || event.aggregateType !== current.snapshot.aggregateType
      || !eventTypeValid
      || event.aggregateType !== kind || kind !== projectionKind(old) || id !== event.aggregateId
      || ('createdAt' in p && 'createdAt' in old && p.createdAt !== old.createdAt) || p.workspaceId !== old.workspaceId
      || ('connectionId' in old && 'attemptId' in old && (!attempt || p.attemptId !== old.attemptId || p.connectionId !== old.connectionId || p.remoteAccountId !== old.remoteAccountId || p.proposalId !== old.proposalId || p.proposalHash !== old.proposalHash || p.clientOrderId !== old.clientOrderId))
      || ('connectionId' in old && !('attemptId' in old) && (!account || !('providerId' in p) || p.connectionId !== old.connectionId || p.providerId !== old.providerId || p.environment !== old.environment))
      || ('threadId' in old && (!thread || p.workspaceId !== old.workspaceId))) throw new Error('IPC_IDENTITY_CONFLICT');
  const fingerprint = JSON.stringify(event);
  if (event.sequence <= current.snapshot.lastSequence) {
    if (current.seen.get(event.sequence) === fingerprint) return current;
    throw new Error('IPC_SEQUENCE_CONFLICT');
  }
  if (event.sequence !== current.snapshot.lastSequence + 1) throw new Error('IPC_SEQUENCE_GAP');
  const seen = new Map(current.seen);
  seen.set(event.sequence, fingerprint);
  if (seen.size > 64) seen.delete(seen.keys().next().value!);
  return { snapshot: { ...current.snapshot, projection: p as T, lastSequence: event.sequence }, seen };
}
