import assert from 'node:assert/strict';
import test from 'node:test';
import { applyEvent, fromModelSnapshot, fromSnapshot, decode } from '../src/projection.ts';

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

test('model projection accepts both provider event types and rejects foreign aggregates', () => {
  const model = {
    workspaceId: 'workspace-one', stateVersion: 'model:workspace-one:1', updatedAt: '2026-09-06T01:00:00Z', attempts: [], currentRoute: null,
    chatgpt: { provider: 'CHATGPT', configured: false, status: 'NOT_CONFIGURED', routes: [], lastVerifiedAt: null, errorCode: null },
    deepseek: { provider: 'DEEPSEEK', configured: false, status: 'NOT_CONFIGURED', routes: [], lastVerifiedAt: null, errorCode: null },
  } as const;
  const initial = fromModelSnapshot({ aggregateType: 'model', aggregateId: 'workspace-one', projection: model, lastSequence: 1 });
  const provider = { eventId: 'model-two', eventType: 'model.provider.changed', schemaVersion: 1, occurredAt: model.updatedAt, aggregateType: 'model', aggregateId: 'workspace-one', sequence: 2, payload: { ...model, stateVersion: 'model:workspace-one:2' } };
  const attempt = { ...provider, eventId: 'model-three', eventType: 'model.provider_attempt.changed', sequence: 3, payload: { ...provider.payload, stateVersion: 'model:workspace-one:3' } };
  const next = applyEvent(initial, provider);
  assert.equal(next.snapshot.lastSequence, 2);
  assert.equal(applyEvent(next, attempt).snapshot.lastSequence, 3);
  assert.throws(() => applyEvent(initial, { ...provider, eventType: 'model.gateway.changed' }));
});
