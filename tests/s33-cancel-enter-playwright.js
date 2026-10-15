// Run with `playwright-cli run-code --filename=tests/s33-cancel-enter-playwright.js`
// against a fresh `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser` harness.
// All account/order observations are local synthetic fixtures; no provider is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const isCancellationDismissalMutation = command =>
    /^(provider\.(connect|probe)|account\.refresh|trade\.execution\.|live\.gateway\.|trade\.cancel_(request|approve|reject))/.test(command);
  const command = async (name, payload) => {
    const requestId = `s33-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200, `${name} bridge status ${result.status}`);
    check(result.body.requestId === requestId, `${name} request id mismatch`);
    check(result.body.ok, `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const isolationNotice = page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  );
  await isolationNotice.waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await isolationNotice.waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 Cancel Enter ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-cancel-enter-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'EUR', path: workspacePath });
  const workspaceId = workspace.workspaceId;
  const risk = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: risk.stateVersion,
    policy: { ...risk.policy, maxSingleInstrumentExposurePercent: '10.25' },
  });
  await command('workspace.ready.fixture', { workspaceId });

  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('EUR');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const account = await command('account.cancellation.fixture.seed', {
    workspaceId, providerId: 'binance', label: `Synthetic CANCEL ${Date.now()}`,
  });
  await command('binance.testnet.cancel.fixture', { scenario: 'PREPARE_ORDER' });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  await command('time.revalidate', { workspaceId });
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).click();
  const accountRow = page.locator('.account-row').filter({ hasText: account.label });
  await accountRow.waitFor({ state: 'visible' });
  await accountRow.click();
  await page.getByRole('button', { name: 'Refresh account', exact: true }).click();

  const reviewButtons = page.getByRole('button', { name: 'Review cancellation', exact: true });
  await reviewButtons.first().waitFor({ state: 'visible', timeout: 20000 });
  const candidateRow = reviewButtons.first().locator('xpath=ancestor::tr');
  const orderText = await candidateRow.innerText();
  const brokerOrderId = orderText.match(/([A-Z]+:9007199254740995)/)?.[1];
  check(Boolean(brokerOrderId), `Expected exact synthetic broker order in row: ${orderText}`);
  const exactOrderRow = page.locator('tr').filter({ hasText: brokerOrderId });
  const exactReviewButton = exactOrderRow.getByRole('button', { name: 'Review cancellation', exact: true });

  await exactReviewButton.click();
  const dialog = page.getByRole('dialog', { name: 'Review Live cancellation', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const focused = await page.evaluate(() => document.activeElement?.textContent?.trim());
  check(focused === 'Close without a decision', `CANCEL review safe focus was ${focused}`);
  check(await dialog.getAttribute('aria-modal') === 'true', 'Live CANCEL review is exposed as a modal dialog.');
  const nativeModal = await dialog.evaluate(element => ({ open: element.open, modal: element.matches(':modal') }));
  check(nativeModal.open && nativeModal.modal, `The browser opened a native modal dialog: ${JSON.stringify(nativeModal)}`);
  const focusable = dialog.locator('button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), summary, a[href], [tabindex]:not([tabindex="-1"])');
  const focusableCount = await focusable.count();
  check(focusableCount >= 2, 'The cancellation review exposes keyboard controls.');
  const focusTrace = [];
  for (let index = 0; index < focusableCount + 2; index += 1) {
    await page.keyboard.press('Tab');
    const state = await page.evaluate(() => {
      const active = document.activeElement;
      const dialog = active?.closest('dialog.live-cancel-dialog');
      return {
        open: dialog?.open,
        modal: dialog?.matches(':modal'),
        inside: Boolean(dialog?.contains(active)),
        connected: active?.isConnected,
        activeTag: active?.tagName,
        label: active?.getAttribute('aria-label') || active?.textContent?.trim() || active?.tagName || 'unknown',
      };
    });
    check(state.open && state.modal && state.inside && state.connected,
      `Tab ${index + 1} must keep focus inside the native modal: ${JSON.stringify(state)}`);
    focusTrace.push(state.label);
  }
  for (let index = 0; index < focusableCount + 2; index += 1) {
    await page.keyboard.press('Shift+Tab');
    check(await dialog.evaluate(element => element.contains(document.activeElement)),
      'Shift+Tab focus must remain inside the native modal.');
  }

  await page.setViewportSize({ width: 390, height: 844 });
  await dialog.evaluate(element => { element.scrollTop = 0; });
  const narrowActions = dialog.locator('.picker-dialog-actions button:not(:disabled)');
  for (let index = 0; index < await narrowActions.count(); index += 1) {
    await narrowActions.nth(index).scrollIntoViewIfNeeded();
  }
  await dialog.evaluate(element => { element.scrollTop = element.scrollHeight; });
  const narrowGeometry = await dialog.evaluate(element => {
    const bounds = element.getBoundingClientRect();
    const actions = [...element.querySelectorAll('.picker-dialog-actions button:not(:disabled)')]
      .map(button => { const box = button.getBoundingClientRect(); return { left: box.left, right: box.right, top: box.top, bottom: box.bottom }; });
    return {
      viewport: document.documentElement.clientWidth,
      documentScroll: document.documentElement.scrollWidth,
      dialogWidth: element.clientWidth,
      dialogScroll: element.scrollWidth,
      left: bounds.left,
      right: bounds.right,
      actions,
      height: innerHeight,
    };
  });
  check(narrowGeometry.documentScroll <= narrowGeometry.viewport
    && narrowGeometry.dialogScroll <= narrowGeometry.dialogWidth
    && narrowGeometry.left >= 0 && narrowGeometry.right <= 390
    && narrowGeometry.actions.every(box => box.left >= 0 && box.right <= 390 && box.top >= 0 && box.bottom <= narrowGeometry.height + 1),
    `Live CANCEL dialog controls remain reachable at 390px: ${JSON.stringify(narrowGeometry)}`);
  await dialog.screenshot({ path: 'docs/implementation/evidence/s33/live-cancel-dialog-390.png' });

  await page.setViewportSize({ width: 768, height: 640 });
  const tabletGeometry = await dialog.evaluate(element => {
    const bounds = element.getBoundingClientRect();
    const actions = [...element.querySelectorAll('.picker-dialog-actions button:not(:disabled)')]
      .map(button => { const box = button.getBoundingClientRect(); return { left: box.left, right: box.right, top: box.top, bottom: box.bottom }; });
    return {
      viewport: document.documentElement.clientWidth,
      documentScroll: document.documentElement.scrollWidth,
      dialogWidth: element.clientWidth,
      dialogScroll: element.scrollWidth,
      left: bounds.left,
      right: bounds.right,
      actions,
      height: innerHeight,
    };
  });
  check(tabletGeometry.documentScroll <= tabletGeometry.viewport
    && tabletGeometry.dialogScroll <= tabletGeometry.dialogWidth
    && tabletGeometry.left >= 0 && tabletGeometry.right <= 768
    && tabletGeometry.actions.every(box => box.left >= 0 && box.right <= 768 && box.top >= 0 && box.bottom <= tabletGeometry.height + 1),
    `Live CANCEL dialog controls remain reachable at 768px: ${JSON.stringify(tabletGeometry)}`);
  await page.setViewportSize({ width: 1280, height: 900 });

  const beforeEscape = await command('trade.cancel_approval.list', {
    workspaceId, accountId: account.connectionId, brokerOrderId,
  });
  check(beforeEscape.approvals.length === 0 && beforeEscape.rejections.length === 0,
    'The synthetic CANCEL intent has no prior approval or rejection before Escape dismissal.');
  const beforeEscapeLog = await page.evaluate(async () =>
    (await fetch('/__integration/command-log?after=0')).json());
  await page.keyboard.press('Escape');
  await dialog.waitFor({ state: 'hidden' });
  const afterEscapeLog = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), beforeEscapeLog.cursor);
  const mutationCommands = afterEscapeLog.commands.filter(entry => isCancellationDismissalMutation(entry.command));
  check(mutationCommands.length === 0,
    `Escape dismissal must not mutate cancellation or execution state: ${JSON.stringify(mutationCommands)}`);
  const afterEscape = await command('trade.cancel_approval.list', {
    workspaceId, accountId: account.connectionId, brokerOrderId,
  });
  check(afterEscape.approvals.length === beforeEscape.approvals.length
    && afterEscape.rejections.length === beforeEscape.rejections.length,
    'Escape dismissal leaves cancellation approval and rejection history unchanged.');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Review cancellation',
    'Escape closes the modal and restores focus to the exact cancellation trigger.');
  await exactReviewButton.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Close without a decision',
    'Reopening through the keyboard restores the safe initial focus.');
  const before = await command('trade.cancel_approval.list', {
    workspaceId, accountId: account.connectionId, brokerOrderId,
  });
  check(before.approvals.length === 0 && before.rejections.length === 0,
    'Opening a CANCEL review must not issue approval or rejection.');

  const beforeEnter = await page.evaluate(async () =>
    (await fetch('/__integration/command-log?after=0')).json());
  await dialog.getByRole('button', { name: 'Close without a decision', exact: true }).press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  const afterEnter = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), beforeEnter.cursor);
  const forbiddenCommands = afterEnter.commands.filter(entry => isCancellationDismissalMutation(entry.command));
  check(forbiddenCommands.length === 0,
    `Enter on the safe default must not dispatch provider/gateway/execution/cancellation commands: ${JSON.stringify(forbiddenCommands)}`);
  const after = await command('trade.cancel_approval.list', {
    workspaceId, accountId: account.connectionId, brokerOrderId,
  });
  check(after.approvals.length === before.approvals.length
    && after.rejections.length === before.rejections.length,
    'Enter on the safe default must leave CANCEL approvals and rejections unchanged.');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Review cancellation',
    'Enter on the safe dismissal restores the exact cancellation trigger.');
  const externalRequests = await page.evaluate(() => performance.getEntriesByType('resource')
    .map(entry => { try { return new URL(entry.name); } catch { return null; } })
    .filter(url => url && ['http:', 'https:'].includes(url.protocol)
      && !['127.0.0.1', 'localhost'].includes(url.hostname))
    .map(url => url.origin));
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  return {
    result: 'PASS',
    focus: focused,
    nativeModal,
    focusableCount,
    tabFocusStayedInDialog: focusTrace.length === focusableCount + 2,
    reverseTabStayedInDialog: true,
    responsiveWidth: narrowGeometry.viewport,
    responsiveActionsReachable: true,
    tabletWidth: tabletGeometry.viewport,
    tabletActionsReachable: true,
    brokerOrderId,
    escapeMutationCommands: mutationCommands.length,
    approvalsAfterEscape: afterEscape.approvals.length,
    rejectionsAfterEscape: afterEscape.rejections.length,
    approvalsBefore: before.approvals.length,
    approvalsAfterEnter: after.approvals.length,
    rejectionsAfterEnter: after.rejections.length,
    forbiddenCommands: forbiddenCommands.length,
    externalRequests: externalRequests.length,
  };
}
