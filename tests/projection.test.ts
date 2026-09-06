import assert from 'node:assert/strict';
import test from 'node:test';
import { applyEvent, fromSnapshot, decode } from '../src/projection.ts';

const workspace = {
  workspaceId: 'workspace-one', name: 'Equity research', baseCurrency: 'EUR', path: '/workspace',
  createdAt: '2026-09-06T01:00:00Z', lastOpenedAt: '2026-09-06T01:00:00Z', storageSchemaVersion: 1 as const,
};
const snapshot = { aggregateType: 'workspace' as const, aggregateId: 'workspace-one', projection: workspace, lastSequence: 1 };
const event = {
  eventId: 'event-two', eventType: 'workspace.opened', schemaVersion: 1, occurredAt: '2026-09-06T01:01:00Z',
  aggregateType: 'workspace', aggregateId: 'workspace-one', sequence: 2,
  payload: { ...workspace, lastOpenedAt: '2026-09-06T01:01:00Z' },
};

test('the UI applies only contiguous supported events and ignores exact duplicates', () => {
  const initial = fromSnapshot(snapshot);
  const next = applyEvent(initial, event);
  assert.equal(next.snapshot.lastSequence, 2);
  assert.equal(next.snapshot.projection.lastOpenedAt, '2026-09-06T01:01:00Z');
  assert.equal(applyEvent(next, structuredClone(event)), next);
  for (const invalid of [
    { ...event, sequence: 4 },
    { ...event, eventId: 'conflicting-event' },
    { ...event, payload: { ...workspace, name: 'conflicting payload' } },
    { ...event, schemaVersion: 2 },
    { ...event, eventType: 'trade.order.filled' },
    { ...event, aggregateId: 'another-workspace' },
  ]) assert.throws(() => applyEvent(next, invalid));
  assert.equal(next.snapshot.lastSequence, 2, 'rejected events cannot advance the cursor');
});

test('generated result schema rejects false success, mixed envelopes and foreign payloads', () => {
  const good = { requestId: 'one', schemaVersion: 1, ok: true, data: workspace };
  assert.deepEqual(decode('ResultEnvelope', good), good);
  for (const invalid of [
    { ...good, schemaVersion: 2 }, { ...good, ok: false },
    { ...good, error: { message: 'unexpected' } },
    { ...good, data: { ...workspace, armed: true } },
    { ...good, data: { ...workspace, workspaceId: null } },
    { ...good, data: { ...workspace, workspaceId: '' } },
    { ...good, stateVersion: null },
  ]) assert.throws(() => decode('ResultEnvelope', invalid));
});
