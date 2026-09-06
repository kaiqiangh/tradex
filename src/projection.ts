import generatedValidators from '../shared/ipc-validators.js';
import type { DomainEvent, Snapshot } from '../shared/ipc-types.ts';

const validators = generatedValidators as Record<string, (value: unknown) => boolean>;

export function decode<T>(definition: string, value: unknown): T {
  const validate = validators[definition];
  if (!validate || !validate(value)) throw new Error('IPC_SCHEMA_INCOMPATIBLE');
  return value as T;
}

export interface Projection {
  snapshot: Snapshot;
  seen: Map<number, string>;
}

export function fromSnapshot(value: unknown): Projection {
  const snapshot = decode<Snapshot>('Snapshot', value);
  if (snapshot.aggregateId !== snapshot.projection.workspaceId) throw new Error('IPC_IDENTITY_CONFLICT');
  return { snapshot, seen: new Map() };
}

export function applyEvent(current: Projection, value: unknown): Projection {
  const event = decode<DomainEvent>('DomainEvent', value);
  if (event.aggregateId !== current.snapshot.aggregateId || event.payload.workspaceId !== event.aggregateId
      || event.payload.createdAt !== current.snapshot.projection.createdAt) throw new Error('IPC_IDENTITY_CONFLICT');
  const fingerprint = JSON.stringify(event);
  if (event.sequence <= current.snapshot.lastSequence) {
    if (current.seen.get(event.sequence) === fingerprint) return current;
    throw new Error('IPC_SEQUENCE_CONFLICT');
  }
  if (event.sequence !== current.snapshot.lastSequence + 1) throw new Error('IPC_SEQUENCE_GAP');
  const seen = new Map(current.seen);
  seen.set(event.sequence, fingerprint);
  if (seen.size > 64) seen.delete(seen.keys().next().value!);
  return { snapshot: { ...current.snapshot, projection: event.payload, lastSequence: event.sequence }, seen };
}
