// Actual React/Rust/temp SQLite; only external HTTP responses and vault are fake.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkSpotOrderIntervalsCurrentUI(tab, browser) {
  const ui = tab.playwright;
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin }, body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }) });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
    return result.data;
  };
  const workspace = await send('workspace.open', {}), workspaceId = workspace.workspaceId;
  if (await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible()) {
    await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  }
  let account = (await send('account.list', { workspaceId })).accounts.find(row => row.label.startsWith('S29.6 external ') && row.connectionState === 'CONNECTED');
  if (!account) {
    const tested = await send('provider.connect', { step: 'test', workspaceId, providerId: 'binance', environment: 'LIVE', label: `S29.6 external ${randomUUID()}` });
    account = await send('provider.connect', { step: 'confirm', workspaceId, connectionId: tested.connectionId, expectedStateVersion: tested.stateVersion, acknowledgeUnverified: false });
  }
  assert.equal(account.health.arming, 'DISARMED');
  const saved = await send('data.binance_rules.connection', { workspaceId });
  await send('data.binance_rules.configure', { workspaceId, connectionId: account.connectionId, instrumentId: 'crypto:BTC/USDT:spot', expectedStateVersion: saved.stateVersion });
  let current;
  const refreshSource = async scenario => {
    await send('binance.live.capacity.fixture', { scenario });
    await send('time.revalidate', { workspaceId });
    const source = await send('data.binance_rules.connection', { workspaceId });
    const refreshed = await send('data.binance_rules.refresh', { workspaceId, expectedStateVersion: source.stateVersion });
    assert.equal(refreshed.status, 'AVAILABLE');
    if (current) await current.getByText(refreshed.evidence.materialVersion, { exact: false }).waitFor({ state: 'visible' });
  };
  await refreshSource('BASE_LISTS');
  const navigate = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name: 'Order Drafts', exact: true });
  if (!await navigate.isVisible()) await ui.getByText('More', { exact: true }).press('Enter');
  await navigate.press('Enter'); await tab.getAXState({ emit: false });
  await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
  await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
  await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
  await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('60000');
  await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  await tab.getAXState({ emit: false });
  current = ui.getByRole('region', { name: 'Current Proposal order interval inputs', exact: true });
  await current.waitFor({ state: 'visible', timeout: 10000 });
  const read = current.getByRole('button', { name: 'Refresh account interval inputs', exact: true });
  await read.press('Enter');
  await current.getByText('observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  const currentText = await current.innerText();
  assert.match(currentText, /9007199254740993/);
  assert.match(currentText, /9223372036854775807/);
  assert.match(currentText, /Across all keys, IPs and APIs/);
  assert.match(currentText, /non-atomic/);
  assert.match(currentText, /REQUEST_WEIGHT/);
  assert.match(currentText, /Original provider counter snapshot time\nNot supplied/);
  await ui.getByRole('button', { name: 'Evaluate risk', exact: true }).press('Enter');
  const capture = ui.getByRole('region', { name: 'Captured Proposal order interval inputs', exact: true }).first();
  await capture.waitFor({ state: 'visible' });
  const capturedText = await capture.innerText();
  assert.match(capturedText, /observed · Read-only inputs · Execution qualification unavailable/);
  assert.equal(await capture.getByRole('button').count(), 0);
  const viewport = await browser.capabilities.get('viewport');
  const widths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      for (const panel of [current, capture]) {
        const size = await panel.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
        assert.equal(size.viewport, width); assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size)); widths.push(size);
      }
    }
  } finally { await viewport.reset(); }
  return { send, refreshSource, workspaceId, accountId: account.connectionId, current, read, capture, capturedText, currentText, widths };
}

export async function checkSpotOrderIntervalsFaultUI(tab, state) {
  const { current, read, capture, capturedText, refreshSource } = state;
  await refreshSource('MALFORMED'); await read.press('Enter');
  await current.getByText('unavailable · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  assert.match(await current.innerText(), /provider response invalid/);
  assert.equal(await capture.innerText(), capturedText);
  await refreshSource('BASE_LISTS'); await read.press('Enter');
  await current.getByText('observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  assert.equal(await capture.innerText(), capturedText);
  return { faultAndRecovery: true, capturedUnchanged: true };
}

export async function checkSpotOrderIntervalsPendingUI(tab, state) {
  const { current, capture, capturedText, refreshSource } = state;
  await refreshSource('DELAYED');
  await current.getByRole('button', { name: 'Refresh account interval inputs', exact: true }).press('Enter');
  const busy = current.getByRole('button', { name: 'Reading account interval inputs…', exact: true });
  await busy.waitFor({ state: 'visible' }); assert.equal(await busy.isEnabled(), false);
  assert.ok(!(await current.innerText()).includes('9007199254740993'));
  assert.equal(await capture.innerText(), capturedText);
  await current.getByText('observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  return { pendingRetiresOld: true, capturedUnchanged: true };
}

export async function checkSpotOrderIntervalsPrearmUI(tab, browser) {
  const ui = tab.playwright;
  await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const captured = dialog.getByRole('region', { name: 'Captured Proposal order interval inputs', exact: true });
  assert.match(await captured.innerText(), /9007199254740993/);
  assert.match(await captured.innerText(), /Original provider counter snapshot time\nNot supplied/);
  assert.equal(await captured.getByRole('button').count(), 0);
  const armEnabled = await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled();
  assert.equal(armEnabled, false);
  const viewport = await browser.capabilities.get('viewport'), widths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await captured.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.viewport, width); assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size)); widths.push(size);
    }
  } finally { await viewport.reset(); }
  const text = await captured.innerText();
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter'); await tab.getAXState({ emit: false });
  return { text, widths, armEnabled, armingPerformed: false, capturedRefreshControls: 0 };
}
