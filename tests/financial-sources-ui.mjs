// Real Rust/SQLite public workflow; only vault and official-shaped HTTP are external fixtures.
// No verified account, tradability, action-coverage or adjustment fixture is injected.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

async function command(command, payload) {
  const response = await fetch('http://127.0.0.1:1420/__integration/command', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }),
  });
  assert.equal(response.status, 200);
  const result = await response.json();
  assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
  return result.data;
}

export async function checkFinancialSourcesUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
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
    await ui.getByRole('heading', { name: 'Alpaca known company events', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('heading', { name: 'Trading 212 account instruments', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
  };
  const sourceRegion = title => ui.getByRole('region', { name: title, exact: true });
  try {
    await viewport.set({ width: 1280, height: 900 });
    await tab.reload();
    await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).or(ui.getByRole('complementary', { name: 'Workspace', exact: true })).waitFor({ state: 'visible' });
    if (await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).isVisible()) {
      await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
      await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    }
    await ui.getByRole('heading', { name: 'Workspace', exact: true }).first().waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    for (const [provider, label] of [['alpaca/PAPER', 'Company events QA'], ['trading212/LIVE', 'Instrument metadata QA']]) {
      await navigate('Accounts');
      if (await ui.getByRole('combobox', { name: 'Connection source', exact: true }).locator('option').filter({ hasText: label }).count()) continue;
      await ui.getByRole('combobox', { name: 'Connection source', exact: true }).selectOption('new');
      await ui.getByRole('combobox', { name: 'Provider / environment', exact: true }).selectOption(provider);
      await ui.getByLabel('Connection label', { exact: true }).fill(label);
      await ui.getByRole('button', { name: 'Connect account securely', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Permission review', exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
      assert.match(await sourceRegion(label).innerText(), /UNVERIFIED/);
      await ui.getByRole('checkbox', { name: 'I understand that permission scope is unverified and have checked the key’s permissions at the provider.' }).press('Space');
      await ui.getByRole('button', { name: 'Confirm connection', exact: true }).press('Enter');
      await ui.getByText('Unverified scope was explicitly acknowledged for this permission review.', { exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
      assert.match(await sourceRegion(label).innerText(), /CONNECTED/);
    }
    observed.push('Both saved accounts were connected through public UI, acknowledged as UNVERIFIED; Live remained DISARMED.');
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Synchronize time', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    // Real five-second summary quota after the public account connection; no quota-reset fixture.
    await ui.waitForTimeout(5100);
    await settings();
    for (const [title, field, label] of [
      ['Alpaca known company events', 'Company events account', 'Company events QA'],
      ['Trading 212 account instruments', 'Account instruments account', 'Instrument metadata QA'],
    ]) {
      const region = sourceRegion(title);
      assert.equal(await region.locator('input').count(), 0);
      const select = region.getByRole('combobox', { name: field, exact: true });
      const option = select.locator('option').filter({ hasText: label });
      await select.selectOption(await option.getAttribute('value'));
      await region.getByRole('button', { name: `Save ${title.toLowerCase()}`, exact: true }).press('Enter');
      await region.getByText(`${title} selection saved. Read access has not been verified.`, { exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
      await select.selectOption('');
      assert.equal(await region.getByRole('button', { name: `Refresh ${title.toLowerCase()}`, exact: true }).isEnabled(), false);
      await select.selectOption(await option.getAttribute('value'));
      await region.getByRole('button', { name: `Refresh ${title.toLowerCase()}`, exact: true }).press('Enter');
      await region.getByText(`${title} refreshed. Only the listed capabilities are current.`, { exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
    }
    const company = sourceRegion('Alpaca known company events');
    assert.match(await company.innerText(), /26 known events/);
    assert.equal(await company.locator('li').count(), 25);
    await company.getByRole('button', { name: 'Next events', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    assert.equal(await company.locator('li').count(), 1);
    assert.match(await company.innerText(), /Partial record/i);
    assert.match(await company.innerText(), /0\.1234567890123456789/);
    assert.match(await company.innerText(), /Delayed process-date query/);
    const first = await command('data.actions.connection', { workspaceId: workspace.workspaceId });
    const broker = sourceRegion('Trading 212 account instruments');
    assert.match(await broker.innerText(), /Ten-minute provider metadata/);
    assert.match(await broker.innerText(), /US0378331005/);
    assert.match(await broker.innerText(), /current account tradability/i);
    observed.push('Saved-reference CAS, unsaved refresh lock, 26-row paging, partial date-only records and exact decimals are visible; independent blockers remain.');
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20);
      assert.ok(size.scroll <= size.width, `Financial source overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    observed.push('Both source widgets and evidence stay within 390/768/1280px layouts.');
    await broker.getByRole('button', { name: 'Refresh trading 212 account instruments', exact: true }).press('Enter');
    await broker.getByRole('alert').waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.match(await broker.getByRole('alert').innerText(), /quota|rate|wait/i);
    assert.equal(await broker.getByRole('button', { name: 'Reload trading 212 account instruments', exact: true }).isEnabled(), true);
    observed.push('Repeated broker refresh is denied by the actual-account monotonic quota; sanitized error and Reload are available.');
    await command('time.fixture.advance', { workspaceId: workspace.workspaceId, elapsedMs: 30001 });
    await company.getByText('Retained observation is unavailable for current decisions. Refresh the saved source to obtain new evidence.', { exact: true }).waitFor({ state: 'visible' });
    const expired = await command('data.actions.connection', { workspaceId: workspace.workspaceId });
    assert.equal(expired.status, 'UNAVAILABLE');
    assert.deepEqual(expired.evidence, first.evidence);
    await tab.reload();
    await ui.getByRole('heading', { name: 'Workspace', exact: true }).first().waitFor({ state: 'visible' });
    await settings();
    assert.match(await sourceRegion('Alpaca known company events').getByRole('combobox', { name: 'Company events account', exact: true }).textContent(), /Company events QA/);
    const reopened = await command('data.actions.connection', { workspaceId: workspace.workspaceId });
    assert.equal(reopened.status, 'UNVERIFIED');
    assert.equal(reopened.evidence, undefined);
    assert.match(await sourceRegion('Alpaca known company events').innerText(), /First receipt[\s\S]*Not checked/);
    observed.push('Trusted-time expiry preserves original receipt/material as unavailable; reload reopens the workspace, keeps saved selection and retires process evidence without renewing authority.');
    for (const title of ['Alpaca known company events', 'Trading 212 account instruments']) {
      const region = sourceRegion(title);
      await region.getByRole('button', { name: `Disconnect ${title.toLowerCase()}`, exact: true }).press('Enter');
      await region.getByText(`${title} disconnected. Your account and saved key were kept.`, { exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
    }
    await navigate('Accounts');
    const accounts = await command('account.list', { workspaceId: workspace.workspaceId });
    assert.ok(JSON.stringify(accounts).includes('Company events QA'));
    assert.ok(JSON.stringify(accounts).includes('Instrument metadata QA'));
    const liveOption = ui.getByRole('combobox', { name: 'Connection source', exact: true }).locator('option').filter({ hasText: 'Instrument metadata QA' });
    await ui.getByRole('combobox', { name: 'Connection source', exact: true }).selectOption(await liveOption.getAttribute('value'));
    await tab.getAXState({ emit: false });
    assert.match(await sourceRegion('Instrument metadata QA').innerText(), /UNVERIFIED/);
    assert.match(await sourceRegion('Instrument metadata QA').innerText(), /DISARMED/);
    observed.push('Disconnect keeps borrowed account records and credentials; no permission promotion, arming or order mutation occurred.');
    return observed;
  } finally { await viewport.reset(); }
}

export async function checkFinancialEvidenceApprovalUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const workspace = await command('workspace.open', {});
  const navigate = async name => {
    const target = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true });
    if (!(await target.isVisible())) await ui.getByText('More', { exact: true }).press('Enter');
    await target.press('Enter'); await tab.getAXState({ emit: false });
  };
  try {
    await viewport.set({ width: 1280, height: 900 });
    await tab.reload();
    await ui.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Synchronize time', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    const source = ui.getByRole('region', { name: 'Alpaca known company events', exact: true });
    const choice = source.getByRole('combobox', { name: 'Company events account', exact: true });
    await choice.selectOption(await choice.locator('option').filter({ hasText: 'Company events QA' }).getAttribute('value'));
    await source.getByRole('button', { name: 'Save alpaca known company events', exact: true }).press('Enter');
    await source.getByRole('button', { name: 'Refresh alpaca known company events', exact: true }).press('Enter');
    await source.getByText('Alpaca known company events refreshed. Only the listed capabilities are current.', { exact: true }).waitFor({ state: 'visible' });
    const original = await command('data.actions.connection', { workspaceId: workspace.workspaceId });
    await navigate('Order Drafts');
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    const account = ui.getByRole('combobox', { name: 'Account', exact: true });
    await account.selectOption(await account.locator('option').filter({ hasText: 'Instrument metadata QA' }).getAttribute('value'));
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
    await ui.locator('.order-proposal-row').first().press('Enter');
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await dialog.waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const text = await dialog.innerText();
    assert.ok(text.includes(original.evidence.materialVersion));
    assert.ok(text.includes(original.evidence.observedAt));
    assert.match(text, /26 known events/);
    assert.match(text, /CorporateActionCoverage/);
    assert.match(text, /HistoricalAdjustment/);
    assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
    assert.match(text, /Captured financial evidence/);
    assert.match(text, /Captured status: available/);
    assert.match(text, /does not report current eligibility/);
    await command('time.fixture.advance', { workspaceId: workspace.workspaceId, elapsedMs: 30_001 });
    await tab.getAXState({ emit: false });
    const expired = await command('data.actions.connection', { workspaceId: workspace.workspaceId });
    assert.equal(expired.status, 'UNAVAILABLE');
    assert.equal(expired.evidence.observedAt, original.evidence.observedAt);
    assert.equal(expired.evidence.materialVersion, original.evidence.materialVersion);
    const frozenText = await dialog.innerText();
    assert.match(frozenText, /Captured status: available/);
    assert.match(frozenText, /does not report current eligibility/);
    assert.ok(frozenText.includes(original.evidence.materialVersion));
    assert.ok(frozenText.includes(original.evidence.observedAt));
    assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
    const layouts = [];
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await dialog.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
      });
      assert.ok(size.left >= 0 && size.right <= size.width && size.scroll <= size.width, JSON.stringify(size));
      layouts.push(size);
    }
    const proposals = await command('trade.proposal.list', { workspaceId: workspace.workspaceId });
    const approvals = await command('trade.approval.list', { workspaceId: workspace.workspaceId, proposalId: proposals.proposals.at(-1).proposalId });
    assert.equal(approvals.approvals.length, 0);
    await dialog.press('Escape'); await dialog.waitFor({ state: 'hidden' });
    return { originalReceipt: original.evidence.observedAt, materialVersion: original.evidence.materialVersion, layouts, approvalCount: 0, financialMutation: false, summary: 'Public unverified/disarmed account; original known-event evidence and independent coverage/adjustment blockers in the disabled pre-arm review. No positive authority seed or financial action.' };
  } finally { await viewport.reset(); }
}
