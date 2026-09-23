import generatedValidators from '../shared/ipc-validators.js';
import type { AccountConnection, AlpacaPaperOrderAttempt, AlpacaPaperOrderBook, DomainEvent, DomainProjection, GatewayState, ModelState, RiskPolicyState, Snapshot, Thread, Trading212DemoOrderAttempt, Trading212DemoOrderBook, Workspace } from '../shared/ipc-types.ts';

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

function projectionKind(p: DomainProjection) { return 'threadId' in p ? 'thread' : 'attemptId' in p ? ('clientOrderId' in p ? 'alpaca-paper-order-attempt' : 'trading212-demo-order-attempt') : 'orders' in p ? ('environment' in p ? 'trading212-demo-order-book' : 'alpaca-paper-order-book') : 'connectionId' in p ? 'account' : 'pinnedVersion' in p ? 'model-gateway' : 'chatgpt' in p ? 'model' : 'hardRules' in p ? 'risk' : 'workspace'; }

function snapshotOf<T>(value: unknown, kind: 'workspace' | 'account' | 'model-gateway' | 'model' | 'risk' | 'thread' | 'alpaca-paper-order-attempt' | 'trading212-demo-order-attempt' | 'alpaca-paper-order-book' | 'trading212-demo-order-book'): Projection<T> {
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
export function fromTrading212DemoAttemptSnapshot(value: unknown): Projection<Trading212DemoOrderAttempt> { return snapshotOf(value, 'trading212-demo-order-attempt'); }
export function fromAlpacaPaperOrderBookSnapshot(value: unknown): Projection<AlpacaPaperOrderBook> { return snapshotOf(value, 'alpaca-paper-order-book'); }
export function fromTrading212DemoOrderBookSnapshot(value: unknown): Projection<Trading212DemoOrderBook> { return snapshotOf(value, 'trading212-demo-order-book'); }

export function applyEvent<T extends DomainProjection>(current: Projection<T>, value: unknown): Projection<T> {
  const event = decode<DomainEvent>('DomainEvent', value);
  const p = event.payload;
  const old = current.snapshot.projection;
  const thread = 'threadId' in p;
  const attempt = 'attemptId' in p;
  const alpacaAttempt = attempt && 'clientOrderId' in p;
  const book = 'orders' in p;
  const trading212Book = book && 'environment' in p;
  const account = 'connectionId' in p && !attempt && !book;
  const id = thread ? p.threadId : attempt ? p.attemptId : account || book ? p.connectionId : p.workspaceId;
  const kind = projectionKind(p);
  const eventTypeValid = thread
    ? event.eventType === 'thread.created' || event.eventType === 'thread.updated'
    : attempt
    ? event.eventType === (alpacaAttempt ? 'alpaca.paper.order.attempt.changed' : 'trading212.demo.order.attempt.changed')
    : book
    ? event.eventType === (trading212Book ? 'trading212.demo.order.book.changed' : 'alpaca.paper.order.book.changed')
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
      || ('connectionId' in old && 'attemptId' in old && (!attempt || p.attemptId !== old.attemptId || p.connectionId !== old.connectionId || p.remoteAccountId !== old.remoteAccountId || p.proposalId !== old.proposalId || p.proposalHash !== old.proposalHash || ('clientOrderId' in old && (!('clientOrderId' in p) || p.clientOrderId !== old.clientOrderId)) || (!('clientOrderId' in old) && 'clientOrderId' in p)))
      || ('connectionId' in old && 'orders' in old && (!book || p.connectionId !== old.connectionId || p.remoteAccountId !== old.remoteAccountId || ('environment' in old ? !('environment' in p) || p.environment !== old.environment : 'environment' in p)))
      || ('connectionId' in old && !('attemptId' in old) && !('orders' in old) && (!account || !('providerId' in p) || p.connectionId !== old.connectionId || p.providerId !== old.providerId || p.environment !== old.environment))
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
