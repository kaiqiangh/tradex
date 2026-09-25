// Run through the Codex CUA browser binding against `npm run dev:browser`.
// Every tested account is a synthetic Rust/SQLite fixture; no provider request is made.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';
import { checkWorkspaceUI } from './workspace-ui.mjs';

async function sendIntegrationCommand(command, payload) {
  const requestId = randomUUID();
  const response = await fetch('http://127.0.0.1:1420/__integration/command', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ requestId, schemaVersion: 1, command, payload }),
  });
  assert.equal(response.status, 200, `${command} remains available in the Rust integration bridge`);
  const result = await response.json();
  assert.equal(result.requestId, requestId);
  assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
  return result.data;
}

async function accountState(ui, row, expected) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if ((await row.innerText()).includes(`Arming: ${expected}`)) return;
    await ui.waitForTimeout(50);
  }
  assert.fail(`Account row did not reach ${expected}: ${await row.innerText()}`);
}

async function focusState(ui, expectedId) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (await ui.evaluate(() => document.activeElement?.id) === expectedId) return;
    await ui.waitForTimeout(50);
  }
  assert.fail(`Focus did not return to ${expectedId}`);
}

export async function checkLiveArmingUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await checkWorkspaceUI(tab, browser);
    await viewport.set({ width: 1280, height: 900 });
    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    const identity = await ui.getByRole('complementary', { name: 'Workspace', exact: true }).innerText();
    const workspaceId = identity.match(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i)?.[0];
    assert.ok(workspaceId, `The isolated workspace identity is visible: ${identity}`);

    const currentRisk = await sendIntegrationCommand('risk.get_policy', { workspaceId });
    await sendIntegrationCommand('risk.save_policy', {
      workspaceId,
      expectedStateVersion: currentRisk.stateVersion,
      policy: { ...currentRisk.policy, maxSingleInstrumentExposurePercent: '10.25' },
    });
    const first = await sendIntegrationCommand('account.arming.fixture.seed', {
      workspaceId, providerId: 'binance', label: `Browser Binance ${Date.now()}`,
    });
    const second = await sendIntegrationCommand('account.arming.fixture.seed', {
      workspaceId, providerId: 'bitget', label: `Browser Bitget ${Date.now()}`,
    });
    await tab.reload();
    const time = await sendIntegrationCommand('time.revalidate', { workspaceId });
    assert.equal(time.confidence, 'TRUSTED', 'Revalidate after the workspace restore has reset volatile time trust');
    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    const firstRow = ui.locator('.account-row').filter({ hasText: first.label });
    const secondRow = ui.locator('.account-row').filter({ hasText: second.label });
    await firstRow.waitFor({ state: 'visible' });
    await secondRow.waitFor({ state: 'visible' });
    await firstRow.click();

    const tradeThreadTitle = `Live context stays disarmed ${Date.now()}`;
    await ui.getByRole('button', { name: '+ New Thread', exact: true }).click();
    await ui.getByRole('textbox', { name: 'Thread title', exact: true }).fill(tradeThreadTitle);
    await ui.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('TRADE');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(first.connectionId);
    const createTradeThread = ui.getByRole('button', { name: 'Create Thread', exact: true });
    for (let attempt = 0; attempt < 100 && !(await createTradeThread.isEnabled()); attempt += 1) await ui.waitForTimeout(50);
    assert.equal(await createTradeThread.isEnabled(), true, 'A Live Trade-context thread can be created without arming');
    await createTradeThread.click();
    await ui.getByRole('heading', { name: tradeThreadTitle, exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('button', { name: 'Arm Live Trading', exact: true }).count(), 0, 'Thread creation has no Arm action');
    let list = await sendIntegrationCommand('account.list', { workspaceId });
    assert.equal(list.accounts.find(account => account.connectionId === first.connectionId)?.health.arming, 'DISARMED');
    await tab.reload();
    const recentTradeThread = ui.locator('.thread-history.compact').getByRole('button', { name: tradeThreadTitle, exact: true });
    await recentTradeThread.waitFor({ state: 'visible' });
    await recentTradeThread.click();
    await ui.getByRole('heading', { name: tradeThreadTitle, exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('button', { name: 'Arm Live Trading', exact: true }).count(), 0, 'Resuming a Trade-context thread has no Arm action');
    list = await sendIntegrationCommand('account.list', { workspaceId });
    assert.equal(list.accounts.find(account => account.connectionId === first.connectionId)?.health.arming, 'DISARMED');
    observed.push('Trade mode, a Binance Live context, and thread restoration leave the account DISARMED with no Arm action.');

    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    await firstRow.waitFor({ state: 'visible' });
    await firstRow.click();

    const firstArm = ui.getByRole('button', { name: 'Arm Live Trading', exact: true });
    assert.equal(await firstArm.isEnabled(), true, 'Current trusted time, policy and healthy fixture make this account eligible');
    await firstArm.click();
    const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await dialog.waitFor({ state: 'visible' });
    assert.match(await dialog.innerText(), /binance/);
    assert.match(await dialog.innerText(), new RegExp(first.label.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
    assert.match(await dialog.innerText(), /\bLIVE\b/);
    assert.match(await dialog.innerText(), new RegExp(first.connectionId));
    assert.match(await dialog.innerText(), new RegExp(first.data.remoteAccountId));
    assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Keep reviewing');
    await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Escape');
    await dialog.waitFor({ state: 'hidden' });
    assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Arm Live Trading');
    await accountState(ui, firstRow, 'DISARMED');
    observed.push('Arm dialog names provider, full account identity, label and LIVE; Escape cancels without a state change and returns focus.');

    await firstArm.press('Enter');
    await dialog.waitFor({ state: 'visible' });
    assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Keep reviewing');
    await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Tab');
    assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Arm this Live account');
    await ui.getByRole('button', { name: 'Arm this Live account', exact: true }).press('Enter');
    await dialog.waitFor({ state: 'hidden' });
    await accountState(ui, firstRow, 'ARMED');
    await focusState(ui, 'connections-title');
    await accountState(ui, secondRow, 'DISARMED');
    observed.push('Keyboard Enter explicitly arms only the selected synthetic Binance Live account; Bitget Live stays DISARMED.');

    await ui.getByRole('button', { name: 'Disable Live', exact: true }).click();
    await accountState(ui, firstRow, 'DISARMED');
    list = await sendIntegrationCommand('account.list', { workspaceId });
    const disabledFirst = list.accounts.find(account => account.connectionId === first.connectionId);
    assert.equal(disabledFirst.health.armingReason, 'USER_DISABLED');
    observed.push('Per-account Disable Live persists USER_DISABLED without affecting the other Live account.');

    await firstRow.click();
    await ui.getByRole('button', { name: 'Arm Live Trading', exact: true }).click();
    await dialog.waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Arm this Live account', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    await secondRow.click();
    await ui.getByRole('button', { name: 'Arm Live Trading', exact: true }).click();
    await dialog.waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Arm this Live account', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    await ui.getByRole('button', { name: 'Disable All Live Execution', exact: true }).click();
    list = await sendIntegrationCommand('account.list', { workspaceId });
    const live = list.accounts.filter(account => account.environment === 'LIVE');
    assert.equal(live.length, 2);
    assert.ok(live.every(account => account.health.arming === 'DISARMED'));
    assert.ok(live.every(account => account.health.armingReason === 'USER_DISABLED_ALL'));
    observed.push('Global Disable All Live Execution disarms both accounts independently of current selection and persists the global reason.');

    for (const width of [768, 390]) {
      await viewport.set({ width, height: 844 });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20, `Viewport override did not apply at ${width}: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `Horizontal overflow at ${width}: ${JSON.stringify(size)}`);
      const disableAll = ui.getByRole('button', { name: 'Disable All Live Execution', exact: true });
      assert.equal(await disableAll.isVisible(), true, `Global safety control remains visible at ${width}px`);
      await firstRow.click();
      await ui.getByRole('button', { name: 'Arm Live Trading', exact: true }).click();
      await dialog.waitFor({ state: 'visible' });
      const dialogSize = await dialog.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: window.innerWidth };
      });
      assert.ok(dialogSize.left >= 0 && dialogSize.right <= dialogSize.width, `Arm dialog fits at ${width}px: ${JSON.stringify(dialogSize)}`);
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Escape');
      await dialog.waitFor({ state: 'hidden' });
      observed.push(`Arm eligibility and modal fit ${width}px without horizontal overflow.`);
    }

    const pageErrors = await tab.dev.logs({ levels: ['error'], limit: 20 });
    assert.equal(pageErrors.filter(log => !log.url?.startsWith('chrome-extension://') && !log.message.includes('chrome-extension://')).length, 0);
    return observed;
  } finally {
    await viewport.reset();
  }
}
