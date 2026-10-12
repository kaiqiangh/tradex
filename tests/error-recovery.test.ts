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
  assert.equal(message, 'AUTH_ERROR · Authentication needs attention. Next step: Review authentication');
  assert.doesNotMatch(message, /Bearer|synthetic-secret/);
});
