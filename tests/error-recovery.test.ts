import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import react from '@vitejs/plugin-react';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { createServer, type ViteDevServer } from 'vite';
import { getRecoveryVariant, resolveCanonicalRecoveryCode } from '../src/errorRecovery.ts';

let vite: ViteDevServer;
let ErrorRecoveryPanel: typeof import('../src/ErrorRecoveryPanel.tsx').ErrorRecoveryPanel;

before(async () => {
  vite = await createServer({
    configFile: false,
    root: process.cwd(),
    plugins: [react()],
    server: { middlewareMode: true },
    appType: 'custom',
  });
  ({ ErrorRecoveryPanel } = await vite.ssrLoadModule('/src/ErrorRecoveryPanel.tsx'));
});

after(async () => { await vite?.close(); });

const revCCodes = [
  'AUTH_ERROR', 'PERMISSION_ERROR', 'RATE_LIMITED', 'NETWORK_ERROR', 'UNSUPPORTED_CAPABILITY',
  'MARKET_CLOSED', 'INSTRUMENT_HALTED', 'INVALID_ORDER', 'INSUFFICIENT_FUNDS', 'RISK_REJECTED',
  'RISK_EVIDENCE_UNAVAILABLE', 'SUBMISSION_REJECTED', 'SUBMISSION_AMBIGUOUS', 'STREAM_DISCONNECTED',
  'STATE_STALE', 'RECONCILIATION_REQUIRED', 'MODEL_UNAVAILABLE', 'QUOTA_EXCEEDED', 'OAUTH_EXPIRED',
  'INTERNAL_ERROR',
] as const;

test('every RevC canonical error has recovery guidance and an explicit next action', () => {
  for (const code of revCCodes) {
    const variant = getRecoveryVariant(code);
    assert.ok(variant.title, `${code} has a title`);
    assert.ok(variant.message, `${code} has recovery guidance`);
    assert.ok(variant.action, `${code} has a next action`);
  }
});

test('authentication recovery directs credential reconnection and keeps Live disabled', () => {
  const variant = getRecoveryVariant('AUTH_ERROR');
  assert.equal(variant.action, 'Reconnect credentials');
  assert.match(variant.message, /Live execution remains disabled/);
});

test('private-stream recovery requires restored account health before Live execution', () => {
  const variant = getRecoveryVariant('STREAM_DISCONNECTED');
  assert.equal(variant.action, 'Reload state');
  assert.match(variant.message, /Live execution remains disabled until account health is restored/);
});

test('risk rejection and unavailable evidence give distinct no-side-effect recovery guidance', () => {
  const rejected = getRecoveryVariant('RISK_REJECTED');
  const unavailable = getRecoveryVariant('RISK_EVIDENCE_UNAVAILABLE');
  assert.notEqual(rejected.title, unavailable.title);
  for (const variant of [rejected, unavailable]) {
    assert.match(variant.message, /No order attempt or provider\/simulator order side effect was created/);
  }
  assert.match(rejected.message, /recorded rejection and its blocking risk or capacity checks/);
  assert.match(unavailable.message, /persisted RiskDecision and execution evidence/);
  assert.match(unavailable.message, /missing or untrusted inputs/);
});

test('provider rate-limit recovery backs off low-priority traffic and preserves reconciliation priority', async () => {
  const variant = getRecoveryVariant('RATE_LIMITED');
  assert.equal(variant.action, 'Retry after cooldown');
  assert.match(variant.message, /Back off low-priority provider traffic until the provider cooldown ends/);
  assert.match(variant.message, /execution reconciliation at highest priority/);

  const { CommandError, explainError } = await vite.ssrLoadModule('/src/client.ts') as typeof import('../src/client.ts');
  const error = new CommandError({
    category: 'RATE_LIMITED', code: 'PROVIDER_RATE_LIMITED',
    message: 'The provider fixture returned a synthetic cooldown.',
    retryable: true, blocking: true, remediationActions: [],
  } as never);
  assert.match(explainError(error), /Back off low-priority provider traffic/);
  assert.match(explainError(error), /execution reconciliation at highest priority/);
  assert.match(renderToStaticMarkup(createElement(ErrorRecoveryPanel, { error })), /execution reconciliation at highest priority/);
});

test('context errors keep their specific remediation under the canonical unsupported category', async () => {
  const { CommandError, explainError } = await vite.ssrLoadModule('/src/client.ts') as typeof import('../src/client.ts');
  const error = new CommandError({
    category: 'INTERNAL_ERROR', code: 'TURN_CONTEXT_INVALID',
    message: 'This mode and execution context cannot be used together.',
  } as never);
  assert.equal(resolveCanonicalRecoveryCode(error), 'UNSUPPORTED_CAPABILITY');
  assert.equal(explainError(error), 'UNSUPPORTED_CAPABILITY · Capability unavailable. This operation is outside the current provider or workspace capability. Next step: Choose a supported context');
  const html = renderToStaticMarkup(createElement(ErrorRecoveryPanel, { error }));
  assert.match(html, /Next step: Choose a supported context/);
  assert.equal(getRecoveryVariant('UNSUPPORTED_CAPABILITY').action, 'Review capability');
});

test('every RevC canonical recovery renders as an accessible UI alert', () => {
  for (const code of revCCodes) {
    const html = renderToStaticMarkup(createElement(ErrorRecoveryPanel, { code }));
    assert.match(html, new RegExp(`role="alert"[^>]*aria-live="polite"[^>]*aria-label="${code} error recovery"`));
    assert.match(html, new RegExp(`data-error-code="${code}"`));
    assert.match(html, /Next step:/);
  }
});

test('provider and model wire errors resolve to their canonical recovery category', () => {
  const cases = [
    [{ category: 'AUTH_ERROR', code: 'PROVIDER_AUTH_FAILED' }, 'AUTH_ERROR'],
    [{ category: 'PERMISSION_ERROR', code: 'PROVIDER_PERMISSION_BLOCKED' }, 'PERMISSION_ERROR'],
    [{ category: 'RATE_LIMITED', code: 'PROVIDER_RATE_LIMITED' }, 'RATE_LIMITED'],
    [{ category: 'NETWORK_ERROR', code: 'PROVIDER_UNAVAILABLE' }, 'NETWORK_ERROR'],
    [{ category: 'UNSUPPORTED_CAPABILITY', code: 'PROVIDER_UNSUPPORTED' }, 'UNSUPPORTED_CAPABILITY'],
    [{ category: 'INTERNAL_ERROR', code: 'TURN_CONTEXT_INVALID' }, 'UNSUPPORTED_CAPABILITY'],
    [{ category: 'POLICY_ERROR', code: 'RISK_EVIDENCE_UNAVAILABLE' }, 'RISK_EVIDENCE_UNAVAILABLE'],
    [{ category: 'PROVIDER_ORDER_REJECTED', code: 'PROVIDER_ORDER_REJECTED' }, 'SUBMISSION_REJECTED'],
    [{ category: 'MODEL_UNAVAILABLE', code: 'MODEL_UNAVAILABLE' }, 'MODEL_UNAVAILABLE'],
    [{ category: 'OAUTH_EXPIRED', code: 'MODEL_OAUTH_EXPIRED' }, 'OAUTH_EXPIRED'],
    [{ category: 'QUOTA_EXCEEDED', code: 'MODEL_QUOTA_EXCEEDED' }, 'QUOTA_EXCEEDED'],
  ] as const;

  for (const [wireError, expected] of cases) assert.equal(resolveCanonicalRecoveryCode(wireError), expected);
});

test('unknown and malformed errors use INTERNAL_ERROR without echoing raw details', () => {
  for (const error of [
    { category: 'UNRECOGNIZED_STATE', code: 'PRIVATE_KEY_LEAK', message: 'Bearer synthetic-secret' },
    { detail: { category: 'RUNTIME_ERROR', code: 'FUTURE_ERROR', message: 'api_key=synthetic-key' } },
    undefined,
  ]) {
    const code = resolveCanonicalRecoveryCode(error);
    assert.equal(code, 'INTERNAL_ERROR');
    const variant = getRecoveryVariant(code);
    assert.equal(variant.title, 'TradeX needs attention');
    assert.doesNotMatch(`${variant.title} ${variant.message} ${variant.action}`, /synthetic|Bearer|api_key/);
  }
});

test('the rendered fallback hides unknown codes and secret-like error fields', () => {
  const html = renderToStaticMarkup(createElement(ErrorRecoveryPanel, {
    error: { category: 'UNRECOGNIZED_STATE', code: 'PRIVATE_KEY_LEAK', message: 'Bearer synthetic-secret api_key=synthetic-key' },
  }));
  assert.match(html, /data-error-code="INTERNAL_ERROR"/);
  assert.match(html, /TradeX needs attention/);
  assert.doesNotMatch(html, /PRIVATE_KEY_LEAK|synthetic-secret|synthetic-key|Bearer|api_key/);
});

test('generic command errors use canonical remediation without echoing raw backend text', async () => {
  const { CommandError, explainError } = await vite.ssrLoadModule('/src/client.ts') as typeof import('../src/client.ts');
  const message = explainError(new CommandError({
    category: 'AUTH_ERROR', code: 'PROVIDER_AUTH_FAILED', message: 'Bearer synthetic-secret',
  } as never));
  assert.equal(message, 'AUTH_ERROR · Authentication needs attention. Reconnect and verify credentials before retrying. Live execution remains disabled until account health is restored. Next step: Reconnect credentials');
  assert.doesNotMatch(message, /Bearer|synthetic-secret/);
});
