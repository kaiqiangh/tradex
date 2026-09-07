import generatedValidators from '../shared/ipc-validators.js';
import type { AccountConnection, DomainEvent, Snapshot, Workspace } from '../shared/ipc-types.ts';

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

function snapshotOf<T>(value: unknown, kind: 'workspace' | 'account'): Projection<T> {
  const snapshot = decode<Snapshot>('Snapshot', value);
  const p = snapshot.projection;
  const id = 'connectionId' in p ? p.connectionId : p.workspaceId;
  if (snapshot.aggregateType !== kind || snapshot.aggregateId !== id
      || (kind === 'account') !== ('connectionId' in p)) throw new Error('IPC_IDENTITY_CONFLICT');
  return { snapshot: snapshot as Projection<T>['snapshot'], seen: new Map() };
}
export function fromSnapshot(value: unknown): Projection { return snapshotOf(value, 'workspace'); }
export function fromAccountSnapshot(value: unknown): Projection<AccountConnection> { return snapshotOf(value, 'account'); }

export function applyEvent<T extends Workspace | AccountConnection>(current: Projection<T>, value: unknown): Projection<T> {
  const event = decode<DomainEvent>('DomainEvent', value);
  const p = event.payload;
  const old = current.snapshot.projection;
  const account = 'connectionId' in p;
  const id = account ? p.connectionId : p.workspaceId;
  if (event.aggregateId !== current.snapshot.aggregateId || event.aggregateType !== current.snapshot.aggregateType
      || event.eventType !== (account ? 'account.health.changed' : 'workspace.opened')
      || (event.aggregateType === 'account') !== account || id !== event.aggregateId
      || p.createdAt !== old.createdAt || p.workspaceId !== old.workspaceId
      || ('connectionId' in old && (!account || p.providerId !== old.providerId || p.environment !== old.environment))) throw new Error('IPC_IDENTITY_CONFLICT');
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
