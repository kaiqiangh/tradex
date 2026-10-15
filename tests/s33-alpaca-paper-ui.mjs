// Invoke checkAlpacaPaperS33(page) with a fresh Playwright page against npm run dev:browser.
// React, Rust and SQLite are real; secret entry, HTTP and private-stream producers are fixtures.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { checkProviderUI } from './provider-ui.mjs';

export async function checkAlpacaPaperS33(page) {
  const origin = 'http://127.0.0.1:1420';
  const consoleErrors = [], pageErrors = [], externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) externalRequests.push(url.origin);
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push({ message: message.text(), url: message.location().url }); });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const response = await fetch(`${origin}/__integration/command`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command: name, payload }),
    });
    const result = await response.json();
    assert.ok(response.ok && result.ok, `${name}: ${JSON.stringify(result.error)}`);
    return result.data;
  };
  const log = async after => (await fetch(`${origin}/__integration/command-log?after=${after}`)).json();
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(`${origin}/__integration/command-log?after=0`);
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.goto(origin);
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true }).waitFor();
  const bootstrap = await command('workspace.open', {});
  const stamp = randomUUID();
  const name = `S33 G2 synthetic ${stamp}`;
  const path = `${bootstrap.path}-g2-${stamp}`;
  const workspace = await command('workspace.open', { name, baseCurrency: 'USD', path });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(name);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(path);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor();
  const policy = await command('risk.get_policy', { workspaceId });
  const savedPolicy = await command('risk.save_policy', { workspaceId, expectedStateVersion: policy.stateVersion,
    policy: { ...policy.policy, staleQuoteThresholdSeconds: 120, maxSingleInstrumentExposurePercent: null } });
  const onboarding = await command('onboarding.set_step', { workspaceId, expectedStateVersion: savedPolicy.stateVersion, step: 5 });
  await command('onboarding.complete', { workspaceId, expectedStateVersion: onboarding.stateVersion });
  const refreshMarketFixture = async () => {
    await command('time.revalidate', { workspaceId });
  };
  await refreshMarketFixture();
  const beforePassive = (await log(0)).cursor;
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.locator('.provider-config').waitFor();
  const passive = (await log(beforePassive)).commands.map(item => item.command);
  assert.ok(!passive.some(name => ['provider.connect', 'account.refresh'].includes(name)), 'Passive Accounts navigation cannot refresh/connect');
  const captures = new Set();
  const dialogLayouts = [];
  const viewport = {
    set: async size => {
      await page.setViewportSize(size);
      const subjects = [
        ['submission', page.getByRole('dialog', { name: 'Confirm Alpaca Paper submission', exact: true })],
        ['cancellation', page.getByRole('dialog', { name: 'Confirm Alpaca Paper cancellation', exact: true })],
        ['health', page.locator('.account-health-card').filter({ hasText: 'alpaca/PAPER QA' })],
      ];
      for (const [name, subject] of subjects) if (await subject.isVisible()) {
        if (size.width === 390) {
          const path = `docs/implementation/evidence/s33/g2-alpaca-${name}-390.png`;
          await subject.screenshot({ path }); captures.add(path);
        }
        if (name !== 'health') {
          for (const button of await subject.getByRole('button').all()) {
            await button.scrollIntoViewIfNeeded();
            const box = await button.boundingBox();
            assert.ok(box && box.x >= 0 && box.x + box.width <= size.width + 1
              && box.y >= 0 && box.y + box.height <= size.height + 1, `${name} action must be reachable at ${size.width}`);
          }
          assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth));
          dialogLayouts.push({ dialog: name, width: size.width, actionsReachable: true, noDocumentOverflow: true });
          if (size.width === 390) {
            const path = `docs/implementation/evidence/s33/g2-alpaca-${name}-actions-390.png`;
            await subject.screenshot({ path }); captures.add(path);
          }
        }
      }
    },
    reset: () => page.setViewportSize({ width: 1280, height: 900 }),
  };
  const tab = {
    playwright: page,
    reload: async () => {
      await page.reload();
      await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor();
      await refreshMarketFixture();
    },
    // Browser ARIA snapshot only; no native accessibility or VoiceOver claim.
    getAXState: () => page.locator('body').ariaSnapshot(),
    dev: { logs: async () => [...consoleErrors, ...pageErrors.map(message => ({ message }))] },
  };
  const browser = { capabilities: { get: async name => { assert.equal(name, 'viewport'); return viewport; } } };
  const cursor = (await log(0)).cursor;
  const observations = await checkProviderUI(tab, browser, 'alpaca/PAPER');
  const commands = (await log(cursor)).commands.map(item => item.command);
  const allowed = new Set([
    'account.activity', 'account.get', 'account.list', 'account.refresh', 'agent.capabilities',
    'alpaca.paper.order.attempt.get', 'alpaca.paper.order.cancel', 'alpaca.paper.order.review',
    'alpaca.paper.order.submit', 'alpaca.paper.orders.get', 'alpaca.paper.orders.refresh',
    'alpaca.paper.stream.disconnect.fixture', 'alpaca.paper.stream.fixture', 'context.catalog',
    'data.fx.connection', 'data.fx.requirements', 'data.source.connection', 'domain.snapshot',
    'domain.subscribe', 'market.catalog', 'paper.get', 'provider.connect', 'provider.disconnect',
    'provider.list_definitions', 'risk.decision.list', 'runtime.status', 'thread.list',
    'time.revalidate', 'time.status', 'trade.draft.get', 'trade.draft.list', 'trade.generate_proposal',
    'trade.proposal.get', 'trade.proposal.list', 'trade.save_draft', 'workspace.open',
  ]);
  const forbidden = commands.filter(name => !allowed.has(name));
  assert.deepEqual(forbidden, [], 'Only the explicitly exercised Paper/local commands are permitted');
  for (const [name, count] of [
    ['provider.connect', 2], ['account.refresh', 4], ['provider.disconnect', 2],
    ['alpaca.paper.order.submit', 1], ['alpaca.paper.order.cancel', 1],
    ['alpaca.paper.orders.refresh', 1], ['alpaca.paper.order.review', 3],
    ['alpaca.paper.stream.fixture', 3], ['alpaca.paper.stream.disconnect.fixture', 1],
  ]) assert.equal(commands.filter(command => command === name).length, count, name);
  assert.deepEqual(externalRequests, []);
  assert.deepEqual(consoleErrors, []);
  assert.deepEqual(pageErrors, []);
  return { status: 'PASS_LIMITED', testedAt: new Date().toISOString(), workspaceId, observations, dialogLayouts, captures: [...captures],
    passiveOpen: { providerConnects: 0, accountRefreshes: 0 },
    commandCounts: Object.fromEntries([...new Set(commands)].sort().map(name => [name, commands.filter(item => item === name).length])),
    forbiddenCommands: forbidden.length, externalRequests, consoleErrors, pageErrors,
    limitations: ['Synthetic HTTP/vault/private-stream and market/time fixtures only', 'Browser ARIA is not native accessibility or VoiceOver', 'No real Alpaca authentication, Paper order, stream or entitlement acceptance', 'Full S17/S33 remain open'] };
}
