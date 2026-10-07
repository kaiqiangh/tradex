// Real Rust/SQLite public workflow; external vault and official-shaped HTTP only.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkFxSourcesUI(tab, browser, baseUrl = 'http://127.0.0.1:1430') {
  const command = async (command, payload) => {
    const response = await fetch(`${baseUrl}/__integration/command`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }),
    });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
    return result.data;
  };
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observations = [];
  const workspace = await command('workspace.open', {});
  const navigate = async name => {
    const target = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true });
    if (!(await target.isVisible())) await ui.getByText('More', { exact: true }).press('Enter');
    await target.press('Enter');
    await tab.getAXState({ emit: false });
  };
  const settings = async () => {
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Alpaca currency rates', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
  };
  const region = () => ui.getByRole('region', { name: 'Alpaca currency rates', exact: true });
  try {
    await viewport.set({ width: 1280, height: 900 });
    await tab.reload();
    await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).or(ui.getByRole('heading', { name: 'Workspace', exact: true }).first()).waitFor({ state: 'visible' });
    if (await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).isVisible()) {
      await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
      await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    }
    await ui.getByRole('heading', { name: 'Workspace', exact: true }).first().waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    for (const [provider, label] of [['alpaca/PAPER', 'FX rates QA'], ['trading212/LIVE', 'FX account QA']]) {
      await navigate('Accounts');
      if (await ui.getByRole('combobox', { name: 'Connection source', exact: true }).locator('option').filter({ hasText: label }).count()) continue;
      await ui.getByRole('combobox', { name: 'Connection source', exact: true }).selectOption('new');
      await ui.getByRole('combobox', { name: 'Provider / environment', exact: true }).selectOption(provider);
      await ui.getByLabel('Connection label', { exact: true }).fill(label);
      await ui.getByRole('button', { name: 'Connect account securely', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Permission review', exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
      await ui.getByRole('checkbox', { name: 'I understand that permission scope is unverified and have checked the key’s permissions at the provider.' }).press('Space');
      await ui.getByRole('button', { name: 'Confirm connection', exact: true }).press('Enter');
      await ui.getByText('Unverified scope was explicitly acknowledged for this permission review.', { exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
      assert.match(await ui.getByRole('region', { name: label, exact: true }).innerText(), /CONNECTED/);
    }
    observations.push('Public connection UI retains UNVERIFIED permission scope and Live DISARMED.');
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Synchronize time', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    await settings();
    assert.equal(await region().locator('input').count(), 0);
    const select = region().getByRole('combobox', { name: 'Currency rates account', exact: true });
    const value = await select.locator('option').filter({ hasText: 'FX rates QA' }).getAttribute('value');
    await select.selectOption(value);
    await region().getByRole('button', { name: 'Save alpaca currency rates', exact: true }).press('Enter');
    await region().getByText('Alpaca currency rates selection saved. Read access has not been verified.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    await select.selectOption('');
    assert.equal(await region().getByRole('button', { name: 'Refresh alpaca currency rates', exact: true }).isEnabled(), false);
    await select.selectOption(value);
    await region().getByRole('button', { name: 'Refresh alpaca currency rates', exact: true }).press('Enter');
    await region().getByText('Alpaca currency rates refreshed. Only the listed capabilities are current.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const first = await command('data.fx.connection', { workspaceId: workspace.workspaceId });
    assert.ok(first.fxRequirements.requirements.some(route => route.purpose === 'BALANCE_WORKSPACE' && route.fromCurrency === 'EUR' && route.providerPair === 'EURUSD'));
    assert.match(await region().innerText(), /balance workspace/i);
    assert.equal(first.status, 'AVAILABLE');
    assert.ok((await region().innerText()).includes('1.01234567890123456789012345678901234567890123456789012345678901 / 1.11234567891123456789112345678911234567891123456789112345678901'), 'Bounded64character prices must remain exact and readable');
    assert.match(await region().innerText(), /Read-only FX rate; transaction-grade qualification unavailable/);
    assert.match(await region().innerText(), /broker conversion costs/i);
    observations.push('Saved-key selection, unsaved refresh lock, independent provider mid and exact bid/ask decimals are displayed with conversion/cost blockers.');
    const layouts = [];
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.scroll <= size.width, `FX overflow at ${width}: ${JSON.stringify(size)}`);
      layouts.push({ requestedWidth: width, ...size });
    }
    await navigate('Accounts');
    await ui.getByRole('button', { name: 'Open portfolio', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    assert.match(await ui.getByRole('region', { name: 'Currency evidence', exact: true }).innerText(), /Workspace base currency: USD/);
    observations.push('Portfolio displays actual currency routes; selection does not restore synthetic monetary authority.');
    await navigate('Order Drafts');
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    const account = ui.getByRole('combobox', { name: 'Account', exact: true });
    await account.selectOption(await account.locator('option').filter({ hasText: 'FX account QA' }).getAttribute('value'));
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('150');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('DAY');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await ui.locator('.order-proposal-row').last().press('Enter'); await tab.getAXState({ emit: false });
    const context = ui.getByRole('region', { name: 'Currency evidence', exact: true });
    assert.match(await context.innerText(), /Selected immutable proposal/);
    await context.getByRole('button', { name: 'Refresh rates for this proposal', exact: true }).press('Enter');
    await context.getByText('USD → EUR', { exact: true }).last().waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const scoped = await command('data.fx.connection', { workspaceId: workspace.workspaceId });
    assert.equal(scoped.status, 'AVAILABLE');
    assert.ok(scoped.evidence.requirements.proposalId);
    assert.ok(scoped.evidence.rates.some(rate => rate.providerPair === 'USDEUR'));
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await dialog.waitFor({ state: 'visible' }); await tab.getAXState({ emit: false });
    const capturedTime = dialog.getByRole('region', { name: 'Captured currency evidence', exact: true }).locator('time');
    await capturedTime.waitFor({ state: 'visible' });
    const capturedTimestamp = await capturedTime.evaluate(element => element.getAttribute('datetime'));
    assert.match(capturedTimestamp, /^\d{4}-\d{2}-\d{2}T/);
    assert.equal(await capturedTime.innerText(), capturedTimestamp);
    const text = await dialog.innerText();
    assert.ok(text.includes(scoped.evidence.materialVersion));
    assert.match(text, /Captured status: available/);
    assert.match(text, /CurrencyConversion/);
    assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
    const reviewLayouts = [];
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await dialog.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
      });
      assert.ok(size.left >= 0 && size.right <= size.width && size.scroll <= size.width, JSON.stringify(size));
      reviewLayouts.push({ requestedWidth: width, ...size });
    }
    await command('time.fixture.advance', { workspaceId: workspace.workspaceId, elapsedMs: 30001 });
    const expired = await command('data.fx.connection', { workspaceId: workspace.workspaceId });
    assert.equal(expired.status, 'UNAVAILABLE');
    assert.deepEqual(expired.evidence, scoped.evidence);
    await tab.getAXState({ emit: false });
    assert.match(await dialog.innerText(), /Captured status: available/);
    assert.equal(await capturedTime.evaluate(element => element.getAttribute('datetime')), capturedTimestamp);
    assert.equal(await capturedTime.innerText(), capturedTimestamp);
    assert.ok((await dialog.innerText()).includes(scoped.evidence.materialVersion));
    await dialog.press('Escape'); await dialog.waitFor({ state: 'hidden' }); await tab.getAXState({ emit: false });
    await context.getByText('Retained observation is unavailable for current decisions. Refresh the saved source to obtain new evidence.', { exact: true }).waitFor({ state: 'visible' });
    observations.push('Immutable proposal refresh obtains the reverse route. Disabled pre-arm review retains original evidence after expiry; current UI marks it unavailable. No Arm or approval action.');
    const approvals = await command('trade.approval.list', { workspaceId: workspace.workspaceId, proposalId: scoped.evidence.requirements.proposalId });
    assert.equal(approvals.approvals.length, 0);
    await settings();
    await region().getByRole('button', { name: 'Refresh alpaca currency rates', exact: true }).press('Enter');
    await region().getByRole('alert').waitFor({ state: 'visible' }); await tab.getAXState({ emit: false });
    assert.match(await region().getByRole('alert').innerText(), /provider denied access/i);
    assert.equal(await region().getByRole('button', { name: 'Reload alpaca currency rates', exact: true }).isEnabled(), true);
    await region().getByRole('button', { name: 'Reload alpaca currency rates', exact: true }).press('Enter'); await tab.getAXState({ emit: false });
    observations.push('The external HTTP fixture denies the third actual read; the widget exposes a sanitized error and Reload without a financial action.');
    await tab.reload(); await ui.getByRole('heading', { name: 'Workspace', exact: true }).first().waitFor({ state: 'visible' });
    await settings();
    const reopened = await command('data.fx.connection', { workspaceId: workspace.workspaceId });
    assert.equal(reopened.status, 'UNVERIFIED');
    assert.equal(reopened.evidence, undefined);
    assert.equal(await region().getByRole('combobox', { name: 'Currency rates account', exact: true }).evaluate(element => element.value), value);
    assert.match(await region().innerText(), /FX rates QA/);
    await region().getByRole('button', { name: 'Disconnect alpaca currency rates', exact: true }).press('Enter');
    await region().getByText('Alpaca currency rates disconnected. Your account and saved key were kept.', { exact: true }).waitFor({ state: 'visible' }); await tab.getAXState({ emit: false });
    const accounts = await command('account.list', { workspaceId: workspace.workspaceId });
    assert.ok(JSON.stringify(accounts).includes('FX rates QA'));
    assert.ok(JSON.stringify(accounts).includes('FX account QA'));
    observations.push('Reload retains saved selection only. Disconnect preserves both account records and borrowed credentials.');
    return { observations, layouts, reviewLayouts, sourceKind: first.kind, approvalCount: 0, providerMutation: false };
  } finally { await viewport.reset(); }
}
