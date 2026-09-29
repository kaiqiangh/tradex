// Run with a Codex CUA browser binding against `npm run dev:browser`.
// The isolated Rust bridge uses synthetic credentials and provider HTTP fixtures only.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';

let commandEndpoint = 'http://127.0.0.1:1420/__integration/command';

async function command(name, payload) {
  const requestId = randomUUID();
  const response = await fetch(commandEndpoint, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ requestId, schemaVersion: 1, command: name, payload }),
  });
  assert.equal(response.status, 200, `${name} is available in the Rust integration bridge`);
  const result = await response.json();
  assert.equal(result.requestId, requestId);
  assert.equal(result.ok, true, `${name}: ${JSON.stringify(result.error)}`);
  return result.data;
}

async function waitFor(ui, predicate, message) {
  for (let attempt = 0; attempt < 120; attempt += 1) {
    if (await predicate()) return;
    await ui.waitForTimeout(50);
  }
  assert.fail(message);
}

async function assertReachableInDialog(ui, tab, selector, text, description) {
  const measure = () => ui.evaluate(({ selector, text }) => {
    const dialog = document.querySelector('dialog.live-cancel-dialog');
    const element = Array.from(dialog?.querySelectorAll(selector) ?? []).find(candidate => candidate.textContent?.trim() === text);
    if (!dialog || !element) return null;
    const target = element.getBoundingClientRect();
    const bounds = dialog.getBoundingClientRect();
    return { left: target.left, right: target.right, top: target.top, bottom: target.bottom, dialogLeft: bounds.left, dialogRight: bounds.right, dialogTop: bounds.top, dialogBottom: bounds.bottom };
  }, { selector, text });
  let bounds = await measure();
  assert.ok(bounds, `${description} is present in the cancellation dialog`);
  if (bounds.top < bounds.dialogTop || bounds.bottom > bounds.dialogBottom) {
    await tab.scroll([(bounds.dialogLeft + bounds.dialogRight) / 2, (bounds.dialogTop + bounds.dialogBottom) / 2], bounds.top < bounds.dialogTop ? 'up' : 'down', 4);
    await tab.getAXState({ emit: false });
    bounds = await measure();
  }
  assert.ok(bounds, `${description} remains present in the cancellation dialog`);
  assert.ok(bounds.left >= bounds.dialogLeft && bounds.right <= bounds.dialogRight && bounds.top >= bounds.dialogTop && bounds.bottom <= bounds.dialogBottom, `${description} remains reachable inside its dialog: ${JSON.stringify(bounds)}`);
}

export async function checkLiveCancellationApprovalUI(tab, browser) {
  const ui = tab.playwright;
  commandEndpoint = new URL('/__integration/command', await tab.url()).toString();
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    const bootstrap = await command('workspace.open', {});
    const workspaceName = `S22 CANCEL ${Date.now()}`;
    const workspacePath = join(dirname(bootstrap.path), `live-cancel-${randomUUID()}`);
    await ui.getByRole('button', { name: 'Workspace', exact: true }).click();
    const createWorkspace = ui.getByRole('heading', { name: 'Create local workspace', exact: true });
    let workspace;
    if (await createWorkspace.count()) {
      await createWorkspace.waitFor({ state: 'visible' });
      await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
      await ui.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('EUR');
      await ui.getByLabel('Local storage', { exact: true }).fill(workspacePath);
      await ui.getByRole('button', { name: 'Open workspace', exact: true }).click();
      await ui.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
      workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'EUR', path: workspacePath });
    } else {
      const currentWorkspacePanel = ui.getByRole('complementary', { name: 'Workspace', exact: true });
      await currentWorkspacePanel.waitFor({ state: 'visible' });
      const currentWorkspaceText = (await currentWorkspacePanel.innerText()).replace(/\n+/g, '\n').trim();
      const currentName = currentWorkspaceText.match(/Workspace\n([^\n]+)\nBase currency/)?.[1];
      const currentCurrency = currentWorkspaceText.match(/Base currency\n([^\n]+)/)?.[1];
      const currentPath = currentWorkspaceText.match(/Storage\n([\s\S]*?)\nWorkspace ID/)?.[1];
      assert.ok(currentName && currentCurrency && currentPath, `Current isolated workspace details are visible: ${currentWorkspaceText}`);
      workspace = await command('workspace.open', { name: currentName, baseCurrency: currentCurrency, path: currentPath });
    }
    const workspaceId = workspace.workspaceId;
    const currentRisk = await command('risk.get_policy', { workspaceId });
    await command('risk.save_policy', {
      workspaceId,
      expectedStateVersion: currentRisk.stateVersion,
      policy: { ...currentRisk.policy, maxSingleInstrumentExposurePercent: '10.25' },
    });
    await command('workspace.ready.fixture', { workspaceId });
    await tab.reload();
    await ui.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
    const account = await command('account.cancellation.fixture.seed', {
      workspaceId, providerId: 'binance', label: `Synthetic CANCEL ${Date.now()}`,
    });
    await command('binance.testnet.cancel.fixture', { scenario: 'PREPARE_ORDER' });
    await tab.reload();
    await command('time.revalidate', { workspaceId });
    assert.equal((await command('time.status', { workspaceId })).confidence, 'TRUSTED', 'The browser fixture has a trusted clock before Live cancellation review.');
    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    const accountRow = ui.locator('.account-row').filter({ hasText: account.label });
    await accountRow.waitFor({ state: 'visible' });
    await accountRow.click();
    await ui.getByRole('button', { name: 'Refresh account', exact: true }).click();
    const reviewButtons = ui.getByRole('button', { name: 'Review cancellation', exact: true });
    await waitFor(ui, async () => await reviewButtons.count() > 0, 'Synthetic provider refresh did not expose an open Live order.');
  const unknownInstrumentRow = ui.locator('tr').filter({ hasText: '9007199254740996' });
  assert.equal(await unknownInstrumentRow.getByRole('button', { name: 'Review cancellation', exact: true }).count(), 0, 'Unmapped provider instruments do not expose an unusable cancellation action.');
  assert.match(await unknownInstrumentRow.innerText(), /Unavailable for cancellation review/);
  const firstOrderRow = reviewButtons.first().locator('xpath=ancestor::tr');
  const firstOrderText = await firstOrderRow.innerText();
  assert.match(firstOrderText, /9007199254740995/);
  const firstBrokerOrderId = firstOrderText.match(/([A-Z]+:9007199254740995)/)?.[1];
  assert.ok(firstBrokerOrderId, `TradeX shows the full provider order identity: ${firstOrderText}`);

  await reviewButtons.first().click();
  const dialog = ui.getByRole('dialog', { name: 'Review Live cancellation', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const initial = await dialog.innerText();
  for (const value of [account.label, account.connectionId, '9007199254740993', firstBrokerOrderId, 'BTCUSDT', 'BUY', 'NEW', '0.1', 'Exact remaining quantity', 'DISARMED']) {
    assert.ok(initial.includes(value), `Cancellation review contains ${value}: ${initial}`);
  }
  let originalIntentId = initial.match(/CANCEL intent ID\s+([^\n]+)/)?.[1];
  let originalIntentHash = initial.match(/Intent hash\s+([^\n]+)/)?.[1];
  assert.match(originalIntentId ?? '', /^cancel:/);
  assert.match(originalIntentHash ?? '', /^sha256:[0-9a-f]{64}$/);
  assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Close without a decision');
  await dialog.getByRole('button', { name: 'Close without a decision', exact: true }).press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Review cancellation', 'Closing restores focus to the order-row invoker.');
  let history = await command('trade.cancel_approval.list', { workspaceId, accountId: account.connectionId, brokerOrderId: firstBrokerOrderId });
  assert.equal(history.approvals.length, 0, 'Enter does not approve the cancellation.');
  observed.push('Enter activates the focused safe-close action and records no cancellation approval.');

  await reviewButtons.first().click();
  await dialog.waitFor({ state: 'visible' });
  const reopened = await dialog.innerText();
  originalIntentId = reopened.match(/CANCEL intent ID\s+([^\n]+)/)?.[1];
  originalIntentHash = reopened.match(/Intent hash\s+([^\n]+)/)?.[1];
  assert.match(originalIntentId ?? '', /^cancel:/);
  assert.match(originalIntentHash ?? '', /^sha256:[0-9a-f]{64}$/);
  assert.ok(reopened.includes(account.connectionId));
  assert.ok(reopened.includes(firstBrokerOrderId));
  observed.push('A fresh explicit review displays the exact disarmed Binance Live account and provider order.');

  await dialog.getByRole('checkbox').check();
  assert.equal((await command('time.revalidate', { workspaceId })).confidence, 'TRUSTED', 'Explicit arming follows a fresh trusted-time check.');
  await dialog.getByRole('button', { name: 'Arm account and refresh this intent', exact: true }).click();
  await waitFor(ui, async () => (await dialog.innerText()).includes('Account arming\nARMED'), 'Arm and same-intent provider refresh did not complete.');
  const armedReview = await dialog.innerText();
  assert.ok(armedReview.includes(originalIntentId), 'Arm and fresh order evidence retain the original CANCEL intent ID.');
  assert.ok(armedReview.includes(originalIntentHash), 'Arm and fresh order evidence retain the original semantic intent hash.');
  assert.match(armedReview, /Risk decision \/ policy\s+ALLOWED/);
  assert.equal(await ui.getByRole('button', { name: 'Approve cancellation', exact: true }).isEnabled(), true);
  observed.push('Explicit Arm returns to the same CANCEL intent and fresh review; it does not navigate to or create a PLACE proposal.');

  await viewport.set({ width: 768, height: 640 });
  const tabletSize = await dialog.evaluate(element => {
    const rect = element.getBoundingClientRect();
    return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
  });
  assert.ok(tabletSize.left >= 0 && tabletSize.right <= tabletSize.width, `CANCEL review fits a 768px viewport: ${JSON.stringify(tabletSize)}`);
  assert.ok(tabletSize.scroll <= tabletSize.width, `CANCEL review has no page-level horizontal overflow at 768px: ${JSON.stringify(tabletSize)}`);
  await assertReachableInDialog(ui, tab, 'dd.identity', account.connectionId, 'The full account identity');
  await assertReachableInDialog(ui, tab, '.picker-dialog-actions button', 'Close without a decision', 'The safe-close action');
  await assertReachableInDialog(ui, tab, '.picker-dialog-actions button', 'Reject cancellation', 'The reject action');
  await assertReachableInDialog(ui, tab, '.picker-dialog-actions button', 'Approve cancellation', 'The approval action');

  await viewport.set({ width: 390, height: 844 });
  const dialogSize = await dialog.evaluate(element => {
    const rect = element.getBoundingClientRect();
    return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
  });
  assert.ok(dialogSize.left >= 0 && dialogSize.right <= dialogSize.width, `CANCEL review fits on a narrow viewport: ${JSON.stringify(dialogSize)}`);
  assert.ok(dialogSize.scroll <= dialogSize.width, `CANCEL review has no page-level horizontal overflow: ${JSON.stringify(dialogSize)}`);
  await assertReachableInDialog(ui, tab, 'dd.identity', account.connectionId, 'The full account identity on a narrow viewport');
  await assertReachableInDialog(ui, tab, '.picker-dialog-actions button', 'Close without a decision', 'The safe-close action on a narrow viewport');
  await assertReachableInDialog(ui, tab, '.picker-dialog-actions button', 'Reject cancellation', 'The reject action on a narrow viewport');
  await assertReachableInDialog(ui, tab, '.picker-dialog-actions button', 'Approve cancellation', 'The approval action on a narrow viewport');
  await ui.getByRole('button', { name: 'Approve cancellation', exact: true }).click();
  await ui.getByRole('status').filter({ hasText: 'The broker order was not cancelled.' }).waitFor({ state: 'visible' });
  history = await command('trade.cancel_approval.list', { workspaceId, accountId: account.connectionId, brokerOrderId: firstBrokerOrderId });
  assert.equal(history.approvals.length, 1);
  assert.equal(history.approvals[0].operation, 'CANCEL');
  assert.equal(history.approvals[0].cancellationIntentId, originalIntentId);
  assert.equal(history.approvals[0].intentHash, originalIntentHash);
  assert.equal(history.approvals[0].brokerOrderId, firstBrokerOrderId);
  assert.equal(history.approvals[0].remainingQuantity, '0.1');
  assert.equal(history.approvals[0].status, 'ISSUED');
  observed.push('Explicit approval is durably recorded as CANCEL for the same exact intent, with a short expiry; the UI states the broker order was not cancelled.');

  await viewport.set({ width: 1280, height: 900 });
  await ui.getByRole('button', { name: 'Order Drafts', exact: true }).click();
  await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
  const approvalRestoreAccountRow = ui.locator('.account-row').filter({ hasText: account.label });
  await approvalRestoreAccountRow.waitFor({ state: 'visible' });
  await approvalRestoreAccountRow.click();
  const approvalRestoreRow = ui.locator('tr').filter({ hasText: '9007199254740995' });
  await approvalRestoreRow.getByRole('button', { name: 'Review cancellation', exact: true }).click();
  const restoredApprovalCard = approvalRestoreRow.getByRole('article', { name: 'Approved Live cancellation preparation', exact: true });
  await restoredApprovalCard.waitFor({ state: 'visible' });
  const restoredApprovalText = await restoredApprovalCard.innerText();
  assert.ok(restoredApprovalText.includes(originalIntentId));
  assert.ok(restoredApprovalText.includes(originalIntentHash));
  assert.equal(await approvalRestoreRow.getByRole('button', { name: 'Prepare cancellation in TradeX', exact: true }).count(), 1);
  assert.equal(await approvalRestoreRow.getByRole('article', { name: 'TradeX cancellation preparation', exact: true }).count(), 0);
  observed.push('An issued CANCEL approval survives navigation and returns as the same exact intent with a separate preparation action.');

  await viewport.set({ width: 1280, height: 900 });
  await ui.getByRole('button', { name: 'Order Drafts', exact: true }).click();
  await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
  const restoredAccountRow = ui.locator('.account-row').filter({ hasText: account.label });
  await restoredAccountRow.waitFor({ state: 'visible' });
  await restoredAccountRow.click();
  const restoredOrderRow = ui.locator('tr').filter({ hasText: '9007199254740995' });
  const restoreButton = restoredOrderRow.getByRole('button', { name: 'Review cancellation', exact: true });
  await restoreButton.click();
  const restoredAgain = restoredOrderRow.getByRole('article', { name: 'Approved Live cancellation preparation', exact: true });
  await restoredAgain.waitFor({ state: 'visible' });
  const restoredText = await restoredAgain.innerText();
  assert.ok(restoredText.includes(originalIntentId), `Restored approval retains its intent: ${restoredText}`);
  assert.ok(restoredText.includes(firstBrokerOrderId), `Restored approval retains its provider order: ${restoredText}`);
  assert.equal(await restoredOrderRow.getByRole('button', { name: 'Prepare cancellation in TradeX', exact: true }).count(), 1);
  history = await command('trade.cancel_approval.list', { workspaceId, accountId: account.connectionId, brokerOrderId: firstBrokerOrderId });
  assert.equal(history.approvals[0].status, 'ISSUED', 'The approval remains unconsumed until its explicit prepare action.');
  observed.push('Leaving Accounts and returning restores the issued cancellation approval; this Binance fixture does not activate the Trading 212-only Order Gateway.');

  await viewport.set({ width: 1280, height: 900 });
  await waitFor(ui, () => reviewButtons.first().isEnabled(), 'The account snapshot did not settle after the cancellation decision.');
  const rejectedRow = ui.locator('tr').filter({ hasText: '9007199254741001' });
  const rejectReviewButton = rejectedRow.getByRole('button', { name: 'Review cancellation', exact: true });
  await rejectReviewButton.click();
  const secondDialog = ui.getByRole('dialog', { name: 'Review Live cancellation', exact: true });
  await secondDialog.waitFor({ state: 'visible' });
  await secondDialog.getByRole('button', { name: 'Reject cancellation', exact: true }).click();
  await ui.getByRole('status').filter({ hasText: 'Cancellation review rejected and recorded.' }).waitFor({ state: 'visible' });
  await waitFor(ui, () => rejectReviewButton.isEnabled(), 'The account snapshot did not settle after rejecting the separate order.');
  const secondOrderId = (await rejectedRow.innerText()).match(/([A-Z]+:9007199254741001)/)?.[1];
  assert.ok(secondOrderId, 'Second provider order identity remains visible after the first decision.');
  history = await command('trade.cancel_approval.list', { workspaceId, accountId: account.connectionId, brokerOrderId: secondOrderId });
  assert.equal(history.rejections.length, 1, 'Reject is stored in durable, order-scoped authorization history.');
  assert.equal(history.approvals.length, 0);
  observed.push('Explicit Reject is durably recorded for a separate Live order without creating an approval.');

  let currentAccount = (await command('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
  assert.ok(currentAccount, 'The synthetic Live account remains available for refresh.');
  await command('account.refresh', { workspaceId, connectionId: account.connectionId, expectedStateVersion: currentAccount.stateVersion });
  const disappearingRow = ui.locator('tr').filter({ hasText: '9007199254740999' });
  const disappearingReviewButton = disappearingRow.getByRole('button', { name: 'Review cancellation', exact: true });
  await waitFor(ui, async () => await disappearingReviewButton.count() === 1, 'Prepared synthetic order did not appear in the Live account snapshot.');
  await disappearingReviewButton.click();
  await dialog.waitFor({ state: 'visible' });
  await command('binance.testnet.cancel.fixture', { scenario: 'REMOVE_ORDER' });
  currentAccount = (await command('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
  assert.ok(currentAccount, 'The Live account remains available for the removal refresh.');
  await command('account.refresh', { workspaceId, connectionId: account.connectionId, expectedStateVersion: currentAccount.stateVersion });
  await waitFor(ui, async () => await disappearingRow.count() === 0, 'Removed provider order stayed in the Live account snapshot.');
  assert.equal(await ui.getByRole('dialog', { name: 'Review Live cancellation', exact: true }).count(), 0);
  assert.equal(await ui.evaluate(() => document.activeElement?.id), 'connections-title', 'Removing the dialog row restores focus to the stable Account connections heading.');
  observed.push('When provider refresh removes an order row with its review open, focus returns to the stable Account connections heading.');
    return observed;
  } finally { await viewport.reset(); }
}
