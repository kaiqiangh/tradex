import assert from 'node:assert/strict';
import test from 'node:test';
import { applyEvent, fromAlpacaPaperAttemptSnapshot, fromTrading212DemoAttemptSnapshot, fromTrading212DemoOrderBookSnapshot, fromModelSnapshot, fromSnapshot, fromThreadSnapshot, decode } from '../src/projection.ts';

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

test('research venue schema keeps unavailable values nullable and rejects numeric zero', () => {
  const venue = {
    venue: 'BINANCE', state: 'UNAVAILABLE', selected: false,
    bid: null, ask: null, spread: null, depth: null, quoteAge: null,
    provenance: {
      sourceId: 'control-plane:market', provider: 'TradeX Control Plane', status: 'UNAVAILABLE',
      receivedTimestamp: 'UNAVAILABLE', freshness: 'UNAVAILABLE', quality: 'UNAVAILABLE',
      limitation: 'No venue entitlement',
    },
    limitation: 'No venue entitlement',
  };
  assert.deepEqual(decode('ResearchSpotVenue', venue), venue);
  for (const field of ['bid', 'ask', 'spread', 'depth', 'quoteAge']) {
    assert.throws(() => decode('ResearchSpotVenue', { ...venue, [field]: 0 }));
  }
});

test('research payload schema rejects unbounded scenarios, artifact refs and venue rows', () => {
  const payload = {
    state: 'UNAVAILABLE', reason: 'No provider observation', focus: 'EQUITY', conclusion: null,
    findings: [], scenarios: [], evidence: [], limitations: [], instrumentRefs: [], artifactRefs: [], spotVenues: [],
  };
  assert.deepEqual(decode('ResearchToolPayload', payload), payload);
  assert.throws(() => decode('ResearchToolPayload', { ...payload, scenarios: Array.from({ length: 9 }, () => ({ title: 'x', detail: 'y' })) }));
  assert.throws(() => decode('ResearchToolPayload', { ...payload, scenarios: [{ title: 'x'.repeat(121), detail: 'y' }] }));
  assert.throws(() => decode('ResearchToolPayload', { ...payload, scenarios: [{ title: 'x', detail: 'y'.repeat(513) }] }));
  assert.throws(() => decode('ResearchToolPayload', { ...payload, artifactRefs: Array.from({ length: 9 }, (_, index) => `artifact-${index}`) }));
  assert.throws(() => decode('ResearchToolPayload', { ...payload, artifactRefs: ['x'.repeat(129)] }));
  assert.throws(() => decode('ResearchToolPayload', { ...payload, spotVenues: Array.from({ length: 3 }, () => ({ venue: 'BINANCE', state: 'UNAVAILABLE', provenance: { sourceId: 'control-plane:market', provider: 'TradeX', status: 'UNAVAILABLE', receivedTimestamp: 'UNAVAILABLE', freshness: 'UNAVAILABLE', quality: 'UNAVAILABLE' } })) }));
  for (const field of ['bid', 'ask', 'spread', 'depth', 'quoteAge']) {
    assert.throws(() => decode('ResearchToolPayload', { ...payload, spotVenues: [{ venue: 'BINANCE', state: 'UNAVAILABLE', [field]: 'x'.repeat(65), provenance: { sourceId: 'control-plane:market', provider: 'TradeX', status: 'UNAVAILABLE', receivedTimestamp: 'UNAVAILABLE', freshness: 'UNAVAILABLE', quality: 'UNAVAILABLE' } }] }));
  }
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

test('thread projection preserves identity and contiguous event ordering', () => {
  const thread = {
    threadId: 'thread-one', workspaceId: 'workspace-one', title: 'Research',
    createdAt: '2026-09-13T01:00:00Z', updatedAt: '2026-09-13T01:00:00Z', stateVersion: 'thread:thread-one:1',
    defaultAgentMode: 'ASK' as const, defaultExecutionContext: 'NONE_READ_ONLY' as const,
    linkedContexts: [], status: 'ACTIVE' as const, turns: [],
  };
  const initial = fromThreadSnapshot({ aggregateType: 'thread', aggregateId: 'thread-one', projection: thread, lastSequence: 1 });
  const event = {
    eventId: 'thread-two', eventType: 'thread.updated' as const, schemaVersion: 1, occurredAt: thread.updatedAt,
    aggregateType: 'thread' as const, aggregateId: 'thread-one', sequence: 2,
    payload: { ...thread, title: 'Updated research', stateVersion: 'thread:thread-one:2', updatedAt: '2026-09-13T01:01:00Z' },
  };
  const next = applyEvent(initial, event);
  assert.equal(next.snapshot.projection.title, 'Updated research');
  assert.equal(applyEvent(next, structuredClone(event)), next);
  assert.throws(() => applyEvent(next, { ...event, sequence: 4 }));
  assert.throws(() => applyEvent(next, { ...event, payload: { ...event.payload, workspaceId: 'other-workspace' } }));
});

test('Alpaca Paper attempts use their own aggregate identity and preserve order bindings', () => {
  const attempt = {
    attemptId: 'attempt-one', workspaceId: 'workspace-one', connectionId: 'connection-one', remoteAccountId: 'paper-account-one',
    proposalId: 'proposal-one', proposalHash: `sha256:${'a'.repeat(64)}`, clientOrderId: 'tradex-attempt-one',
    state: 'SUBMITTING' as const, providerOrderId: null, providerStatus: null, errorCode: null, reason: 'Submitting',
    stateVersion: 'attempt:attempt-one:1', createdAt: '2026-09-23T01:00:00Z', updatedAt: '2026-09-23T01:00:00Z',
  };
  const initial = fromAlpacaPaperAttemptSnapshot({
    aggregateType: 'alpaca-paper-order-attempt', aggregateId: attempt.attemptId, projection: attempt, lastSequence: 1,
  });
  const event = {
    eventId: 'attempt-two', eventType: 'alpaca.paper.order.attempt.changed' as const, schemaVersion: 1,
    occurredAt: '2026-09-23T01:01:00Z', aggregateType: 'alpaca-paper-order-attempt' as const,
    aggregateId: attempt.attemptId, sequence: 2,
    payload: { ...attempt, state: 'ACKNOWLEDGED' as const, providerOrderId: 'provider-order-one', reason: 'Accepted', stateVersion: 'attempt:attempt-one:2', updatedAt: '2026-09-23T01:01:00Z' },
  };
  const next = applyEvent(initial, event);
  assert.equal(next.snapshot.lastSequence, 2);
  assert.equal(next.snapshot.projection.state, 'ACKNOWLEDGED');
  assert.equal(applyEvent(next, structuredClone(event)), next);
  assert.throws(() => applyEvent(initial, { ...event, payload: { ...event.payload, connectionId: 'connection-two' } }));
  assert.throws(() => applyEvent(initial, { ...event, eventType: 'account.health.changed' }));
});

test('Trading 212 Demo attempts replay by attempt identity without inventing a client order ID', () => {
  const attempt = {
    attemptId: 't212-attempt-one', workspaceId: 'workspace-one', connectionId: 'connection-one', remoteAccountId: '9007199254740993',
    proposalId: 'proposal-one', proposalHash: `sha256:${'b'.repeat(64)}`, state: 'SUBMITTING' as const,
    providerOrderId: null, providerStatus: null, errorCode: null, reason: 'Submitting',
    stateVersion: 'trading212-demo-order-attempt:t212-attempt-one:1',
    createdAt: '2026-09-23T01:00:00Z', updatedAt: '2026-09-23T01:00:00Z',
  };
  const initial = fromTrading212DemoAttemptSnapshot({
    aggregateType: 'trading212-demo-order-attempt', aggregateId: attempt.attemptId, projection: attempt, lastSequence: 1,
  });
  const event = {
    eventId: 't212-attempt-two', eventType: 'trading212.demo.order.attempt.changed' as const, schemaVersion: 1,
    occurredAt: '2026-09-23T01:01:00Z', aggregateType: 'trading212-demo-order-attempt' as const,
    aggregateId: attempt.attemptId, sequence: 2,
    payload: { ...attempt, state: 'ACKNOWLEDGED' as const, providerOrderId: '9007199254740995', reason: 'Accepted', stateVersion: 'trading212-demo-order-attempt:t212-attempt-one:2', updatedAt: '2026-09-23T01:01:00Z' },
  };
  const next = applyEvent(initial, event);
  assert.equal(next.snapshot.lastSequence, 2);
  assert.equal(next.snapshot.projection.providerOrderId, '9007199254740995');
  assert.equal(applyEvent(next, structuredClone(event)), next);
  assert.throws(() => applyEvent(initial, { ...event, payload: { ...event.payload, connectionId: 'connection-two' } }));
  assert.throws(() => applyEvent(initial, { ...event, payload: { ...event.payload, clientOrderId: 'invented-client-order' } }));
  assert.throws(() => applyEvent(initial, { ...event, eventType: 'alpaca.paper.order.attempt.changed' }));
});

test('Trading 212 Demo order books replay exact cumulative observations under Demo account identity', () => {
  const book = {
    workspaceId: 'workspace-one', connectionId: 'connection-one', remoteAccountId: '9007199254740993',
    environment: 'DEMO' as const, status: 'CURRENT' as const,
    stateVersion: 'trading212-demo-order-book:connection-one:1',
    lastSuccessfulSyncAt: '2026-09-23T01:00:00Z', observedAt: '2026-09-23T01:00:00Z',
    historyStarted: true, historyComplete: false, historyPageCount: 1,
    nextPagePath: '/api/v0/equity/history/orders?limit=50&cursor=123',
    historyCursors: ['FIRST'], rateLimits: {},
    orders: [{
      providerOrderId: '9007199254740995', symbol: 'AAPL_US_EQ', side: 'BUY',
      orderType: 'LIMIT', timeInForce: 'DAY', providerStatus: 'PARTIALLY_FILLED',
      normalizedStatus: 'PARTIALLY_FILLED' as const, pending: true, quantity: '5',
      filledQuantity: '1.25', filledValue: '25.125', currency: 'GBP', remainingQuantity: '3.75',
      submittedAt: '2026-09-23T00:59:00Z', observedAt: '2026-09-23T01:00:00Z',
      origin: 'TRADE_X' as const, attemptId: 'attempt-one',
    }],
  };
  const initial = fromTrading212DemoOrderBookSnapshot({
    aggregateType: 'trading212-demo-order-book', aggregateId: book.connectionId, projection: book, lastSequence: 1,
  });
  const event = {
    eventId: 't212-order-book-two', eventType: 'trading212.demo.order.book.changed' as const,
    schemaVersion: 1, occurredAt: '2026-09-23T01:01:00Z',
    aggregateType: 'trading212-demo-order-book' as const, aggregateId: book.connectionId, sequence: 2,
    payload: {
      ...book, stateVersion: 'trading212-demo-order-book:connection-one:2',
      observedAt: '2026-09-23T01:01:00Z',
      orders: [{ ...book.orders[0], filledQuantity: '2.5', filledValue: '50.5', remainingQuantity: '2.5', observedAt: '2026-09-23T01:01:00Z' }],
    },
  };
  const next = applyEvent(initial, event);
  assert.equal(next.snapshot.lastSequence, 2);
  assert.equal(next.snapshot.projection.orders[0].filledQuantity, '2.5');
  assert.equal(next.snapshot.projection.orders[0].filledValue, '50.5');
  assert.equal(next.snapshot.projection.orders[0].currency, 'GBP');
  assert.equal(next.snapshot.projection.orders[0].remainingQuantity, '2.5');
  assert.equal(applyEvent(next, structuredClone(event)), next);
  assert.throws(() => applyEvent(initial, { ...event, eventType: 'alpaca.paper.order.book.changed' }));
  assert.throws(() => applyEvent(initial, { ...event, payload: { ...event.payload, remoteAccountId: 'other-account' } }));
  assert.throws(() => applyEvent(initial, { ...event, payload: { ...event.payload, environment: 'LIVE' } }));
});
