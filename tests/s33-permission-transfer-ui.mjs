// Public React -> Rust -> disposable SQLite seam on npm run dev:browser.
// External permission HTTP and native secret entry are integration fixtures only.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkPermissionReviewS33(page, scenario = 'INTERNAL_TRANSFER') {
  const cases = {
    INTERNAL_TRANSFER: { forbidden: ['transfer.internal'], unsupported: [] },
    UNIVERSAL_TRANSFER: { forbidden: ['transfer.universal'], unsupported: [] },
    READING_DISABLED: { forbidden: [], unsupported: ['reading.disabled'] },
    FIX_TRADE: { forbidden: [], unsupported: ['fix.trade'] },
    PORTFOLIO_MARGIN: { forbidden: [], unsupported: ['portfolio.margin'] },
    OPTIONS: { forbidden: [], unsupported: ['options'] },
    FUTURES: { forbidden: [], unsupported: ['futures'] },
    MARGIN: { forbidden: [], unsupported: ['margin'] },
  };
  const expected = cases[scenario];
  assert.ok(expected, `Unknown permission test case ${scenario}`);
  const blocker = [...expected.forbidden, ...expected.unsupported][0];
  const origin = 'http://127.0.0.1:1420';
  const consoleErrors = [], pageErrors = [], externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) externalRequests.push(url.origin);
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push({ message: message.text(), url: message.location().url }); });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload, expectedCode) => {
    const requestId = randomUUID();
    const response = await fetch(`${origin}/__integration/command`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ requestId, schemaVersion: 1, command: name, payload }),
    });
    const result = await response.json();
    assert.equal(response.status, 200);
    assert.equal(result.requestId, requestId);
    if (expectedCode) { assert.equal(result.ok, false); assert.equal(result.error?.code, expectedCode); }
    else assert.equal(result.ok, true, `${name}: ${JSON.stringify(result.error)}`);
    return result.data;
  };
  const log = async after => (await fetch(`${origin}/__integration/command-log?after=${after}`)).json();
  await page.goto(`${origin}/__integration/command-log?after=0`);
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(origin);
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true }).waitFor();
  const bootstrap = await command('workspace.open', {});
  const stamp = randomUUID();
  const name = `S33 ${scenario} permission ${stamp}`;
  const path = `${bootstrap.path}-permission-transfer-${stamp}`;
  const opened = await command('workspace.open', { name, path, baseCurrency: 'USD' });
  const workspaceId = opened.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(name);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(path);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor();
  const cursor = (await log(0)).cursor;
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  const form = page.locator('.provider-config');
  await form.waitFor();
  assert.ok(!(await log(cursor)).commands.some(item => ['provider.connect', 'account.refresh'].includes(item.command)));
  await form.getByRole('combobox', { name: 'Provider / environment', exact: true }).selectOption('binance/LIVE');
  const label = `S33 synthetic ${scenario} ${stamp}`;
  await form.getByRole('textbox', { name: 'Connection label', exact: true }).fill(label);
  await command('binance.live.permission.fixture', { scenario });
  await form.getByRole('button', { name: 'Connect account securely', exact: true }).press('Enter');
  const detail = page.getByRole('region', { name: label, exact: true });
  const permission = detail.locator('.permission-review').filter({ has: page.getByRole('heading', { name: 'Permission review', exact: true }) });
  await permission.getByRole('alert').filter({ hasText: blocker }).waitFor();
  const account = (await command('account.list', { workspaceId })).accounts.find(item => item.label === label);
  assert.deepEqual(account.permissions.forbidden, expected.forbidden);
  assert.deepEqual(account.permissions.unsupported, expected.unsupported);
  assert.equal(account.connectionState, 'REVIEW_REQUIRED');
  assert.equal(account.health.arming, 'DISARMED');
  assert.equal(await permission.getByRole('button', { name: 'Confirm connection', exact: true }).isDisabled(), true);
  assert.equal(await detail.getByRole('button', { name: 'Arm Live Trading', exact: true }).isDisabled(), true);
  await command('provider.connect', {
    step: 'confirm', workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, acknowledgeUnverified: true,
  }, 'PROVIDER_PERMISSION_BLOCKED');
  const rejected = await command('account.get', { workspaceId, connectionId: account.connectionId });
  assert.equal(rejected.stateVersion, account.stateVersion);
  assert.equal(rejected.connectionState, 'REVIEW_REQUIRED');
  assert.equal(rejected.health.arming, 'DISARMED');
  const layouts = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: 900 });
    await permission.getByRole('alert').scrollIntoViewIfNeeded();
    const box = await permission.getByRole('alert').boundingBox();
    assert.ok(box && box.x >= 0 && box.x + box.width <= width + 1 && box.y >= 0 && box.y + box.height <= 901);
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth));
    layouts.push({ width, blockerReachable: true, noDocumentOverflow: true });
    if (width === 390) await permission.screenshot({ path: `docs/implementation/evidence/s33/a3-variants-${scenario.toLowerCase().replaceAll('_', '-')}-blocked-390.png` });
  }
  await command('binance.live.permission.fixture', { scenario: 'SAFE' });
  assert.equal(await permission.getByRole('button', { name: 'Confirm connection', exact: true }).isDisabled(), true,
    'Changing the producer alone cannot replace the saved permission observation');
  await page.waitForTimeout(5100);
  await detail.getByRole('button', { name: 'Refresh account', exact: true }).press('Enter');
  await permission.getByRole('alert').waitFor({ state: 'hidden' });
  const confirm = permission.getByRole('button', { name: 'Confirm connection', exact: true });
  for (let attempt = 0; attempt < 200 && !(await confirm.isEnabled()); attempt += 1) await page.waitForTimeout(50);
  assert.equal(await confirm.isEnabled(), true, 'Explicit re-test must finish before keyboard confirmation');
  await confirm.press('Enter');
  await detail.getByText('binance · LIVE · CONNECTED', { exact: true }).waitFor();
  const confirmed = await command('account.get', { workspaceId, connectionId: account.connectionId });
  assert.deepEqual(confirmed.permissions.forbidden, []);
  assert.deepEqual(confirmed.permissions.unsupported, []);
  assert.equal(confirmed.permissions.scope, 'VERIFIED');
  assert.equal(confirmed.health.arming, 'DISARMED');
  const commands = (await log(cursor)).commands.map(item => item.command);
  const allowed = new Set(['account.activity', 'account.get', 'account.list', 'account.refresh',
    'binance.live.permission.fixture', 'domain.snapshot', 'domain.subscribe', 'paper.get',
    'provider.connect', 'provider.list_definitions', 'runtime.status', 'thread.list',
    'trade.cancel_approval.list', 'trade.execution.order_links', 'risk.get_policy']);
  assert.deepEqual(commands.filter(name => !allowed.has(name)), []);
  assert.equal(commands.filter(name => name === 'provider.connect').length, 3);
  assert.equal(commands.filter(name => name === 'account.refresh').length, 1);
  assert.deepEqual(externalRequests, []); assert.deepEqual(consoleErrors, []); assert.deepEqual(pageErrors, []);
  return { status: 'PASS_LIMITED', testedAt: new Date().toISOString(), workspaceId, scenario, expected, layouts,
    rejectedConfirmation: 'PROVIDER_PERMISSION_BLOCKED', rejectedConfirmationPreservedState: true,
    producerChangeRequiresExplicitRetest: true, finalArming: confirmed.health.arming,
    commandCounts: Object.fromEntries([...new Set(commands)].sort().map(name => [name, commands.filter(item => item === name).length])),
    externalRequests, consoleErrors, pageErrors,
    limitations: ['Synthetic HTTP and secret entry only', 'Other forbidden/unsupported variants, native secure entry, VoiceOver and real-provider acceptance remain open'] };
}
