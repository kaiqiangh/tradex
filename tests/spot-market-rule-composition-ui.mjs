// Actual React → typed Rust → disposable SQLite. External HTTP/vault/WS are synthetic.
// Run in the browser harness with TRADEX_LIVE_APPROVAL_FIXTURE=1; never seed authority.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';
import { waitForUIText } from './spot-owning-admission-ui.mjs';

export async function checkSpotMarketRuleCompositionUI(page, options = {}) {
  page.setDefaultTimeout(15000);
  const failures = [];
  const consoleErrors = [];
  const external = [];
  page.on('pageerror', error => failures.push(error.message));
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) external.push(url.origin);
  });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(options.origin ?? 'http://127.0.0.1:1420');
  const endpoint = new URL('/__integration/command', page.url());
  const traceEndpoint = new URL('/__integration/command-log', endpoint);
  const traceStart = await (await fetch(traceEndpoint)).json();
  const send = async (command, payload) => {
    const response = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin }, body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }) });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
    return result.data;
  };
  const bootstrap = await send('workspace.open', {});
  const workspace = await send('workspace.open', { name: 'MARKET rule composition verification', baseCurrency: 'USD', path: join(dirname(bootstrap.path), `market-rules-${randomUUID()}`) });
  const workspaceId = workspace.workspaceId;
  await page.getByLabel('Local storage', { exact: true }).fill(workspace.path);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Workspace', exact: true }).waitFor();
  const navigate = async name => {
    let buttons = page.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true });
    let visible;
    for (const button of await buttons.all()) if (await button.isVisible()) { visible = button; break; }
    if (!visible) {
      await page.getByText('More', { exact: true }).press('Enter');
      for (const button of await buttons.all()) if (await button.isVisible()) { visible = button; break; }
    }
    assert.ok(visible, `${name} navigation`);
    await visible.press('Enter');
  };
  const tested = await send('provider.connect', { step: 'test', workspaceId, providerId: 'binance', environment: 'LIVE', label: 'Synthetic external MARKET account' });
  const account = await send('provider.connect', { step: 'confirm', workspaceId, connectionId: tested.connectionId, expectedStateVersion: tested.stateVersion, acknowledgeUnverified: false });
  await send('binance.live.capacity.fixture', { scenario: 'MARKET_EMPTY_INVENTORY' });
  const policy = await send('risk.get_policy', { workspaceId });
  await send('risk.save_policy', { workspaceId, expectedStateVersion: policy.stateVersion, policy: { ...policy.policy, staleQuoteThresholdSeconds: 30, marketOrdersEnabled: true, maxMarketOrderSlippagePercent: '1' } });
  await send('time.revalidate', { workspaceId });
  await navigate('Settings');
  await page.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
  await page.getByRole('button', { name: 'Save Binance market source', exact: true }).press('Enter');
  await waitForUIText(page.locator('main'), 'Public market-source selection saved.');
  const source = await send('data.binance_rules.connection', { workspaceId });
  const configured = await send('data.binance_rules.configure', { workspaceId, expectedStateVersion: source.stateVersion, connectionId: account.connectionId, instrumentId: 'crypto:BTC/USDT:spot' });
  assert.equal((await send('data.binance_rules.refresh', { workspaceId, expectedStateVersion: configured.stateVersion })).status, 'AVAILABLE');
  await navigate('Markets');
  await page.getByRole('button', { name: 'Crypto', exact: true }).press('Enter');
  await page.getByRole('button', { name: /^BTC\/USDT / }).press('Enter');
  await waitForUIText(page.getByRole('region', { name: 'Hot quote subscription', exact: true }), 'STREAMING');
  await navigate('Order Drafts');
  await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  for (const [label, value] of [['Execution context', 'BINANCE_LIVE'], ['Account', account.connectionId], ['Instrument', 'crypto:BTC/USDT:spot'], ['Order type', 'MARKET'], ['Quantity type', 'BASE'], ['Time in force', 'DAY']]) {
    await page.getByRole('combobox', { name: label, exact: true }).selectOption(value);
  }
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
  await page.getByRole('textbox', { name: /^Maximum spend/ }).fill('61');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor();
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor();
  const proposal = (await send('trade.proposal.list', { workspaceId })).proposals[0];
  const query = { workspaceId, proposalId: proposal.proposalId };
  const current = page.getByRole('region', { name: 'Current Proposal Spot rules', exact: true });
  const references = await send('trade.spot_rules.get', query);
  await send('trade.spot_rules.refresh', { ...query, expectedStateVersion: references.stateVersion });
  for (const [region, button] of [['Current Proposal Spot capacity inputs', 'Refresh required capacity inputs'], ['Current Proposal order interval inputs', 'Refresh account interval inputs']]) {
    const panel = page.getByRole('region', { name: region, exact: true });
    await panel.getByRole('button', { name: button, exact: true }).press('Enter');
    await waitForUIText(panel, 'observed · Read-only inputs · Execution qualification unavailable');
  }
  await page.getByRole('button', { name: 'Evaluate risk', exact: true }).press('Enter');
  const capture = page.getByRole('region', { name: 'Captured Proposal Spot rules', exact: true }).first();
  await capture.waitFor();
  const decisions = await send('risk.decision.list', query);
  const saved = decisions.decisions[0];
  assert.equal(saved.spotRules.outcome, 'PASS', JSON.stringify(saved.spotRules));
  assert.equal((await send('trade.spot_rules.get', query)).outcome, 'UNAVAILABLE');
  await waitForUIText(current, 'Source-only assessment');
  const composition = capture.getByRole('region', { name: 'Local MARKET rule composition', exact: true });
  await composition.waitFor();
  const comparison = saved.spotRules.rules.find(row => row.ruleType === 'MAX_POSITION').localComparison;
  const comparisonText = `Local comparison: ${comparison.observed} + ${comparison.increment} = ${comparison.total}; limit ${comparison.limit} BTC`;
  await waitForUIText(capture, comparisonText);
  const capturedText = await capture.innerText();
  assert.match(capturedText, /Conservative local policy/);
  assert.match(capturedText, /venue applies PRICE_RANGE during execution/);
  assert.match(capturedText, /non atomic provider snapshot/);
  assert.ok(capturedText.includes(saved.spotRules.marketComposition.capacityVersion));
  assert.ok(capturedText.includes(saved.spotRules.marketComposition.intervalVersion));
  assert.equal(await composition.getByRole('button').count(), 0);
  const widths = [];
  const checkWidths = async panel => {
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: 900 });
      await panel.scrollIntoViewIfNeeded();
      const size = await panel.evaluate(element => ({ width: innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.width, width);
      assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
      widths.push(size);
      if (options.screenshot) await options.screenshot(page, width);
    }
  };
  await checkWidths(capture);
  await page.setViewportSize({ width: 1280, height: 900 });
  const oldSource = await send('data.binance_rules.connection', { workspaceId });
  await send('data.binance_rules.refresh', { workspaceId, expectedStateVersion: oldSource.stateVersion });
  assert.equal(await capture.innerText(), capturedText, 'A source refresh preserves captured comparisons');
  const review = await send('trade.request_approval', query);
  assert.equal(review.eligible, false);
  await page.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await dialog.waitFor();
  const prearm = dialog.getByRole('region', { name: 'Local MARKET rule composition', exact: true });
  await waitForUIText(prearm, 'UNAVAILABLE');
  await checkWidths(prearm);
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  assert.equal((await send('account.get', { workspaceId, connectionId: account.connectionId })).health.arming, 'DISARMED');
  assert.equal((await send('trade.approval.list', query)).approvals.length, 0);
  traceEndpoint.searchParams.set('after', String(traceStart.cursor));
  const trace = await (await fetch(traceEndpoint)).json();
  const forbiddenCommands = trace.commands.filter(entry => ['account.arming.fixture.seed', 'workspace.ready.fixture', 'account.arm', 'trade.approve', 'trade.execution.prepare', 'trade.execution.dispatch'].includes(entry.command));
  assert.deepEqual(forbiddenCommands, [], 'Inspect actual backend commands from both React and this driver');
  assert.deepEqual(failures, []);
  assert.deepEqual(consoleErrors, []);
  assert.deepEqual(external, []);
  return { workspaceId, proposalId: proposal.proposalId, composedOutcome: saved.spotRules.outcome, standaloneOutcome: 'UNAVAILABLE', currentCaptureAndPrearm: true, widths, noAuthorityIssued: true, observedBackendCommands: trace.commands.length, forbiddenCommands, pageErrors: failures, consoleErrors, externalRequests: external };
}
