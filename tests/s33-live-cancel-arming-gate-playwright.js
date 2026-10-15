// Run with `playwright-cli run-code --filename=tests/s33-live-cancel-arming-gate-playwright.js`
// against the isolated `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser` harness.
// All provider responses and account/order observations are disposable local fixtures.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  await page.addInitScript(() => {
    window.__s33Audit = { externalRequests: [], pageErrors: [] };
    const recordUrl = raw => {
      try {
        const url = new URL(raw instanceof Request ? raw.url : raw, location.href);
        if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
          window.__s33Audit.externalRequests.push(url.origin);
        }
      } catch { /* Relative and non-network requests are not external HTTP. */ }
    };
    const originalFetch = window.fetch.bind(window);
    window.fetch = (...args) => { recordUrl(args[0]); return originalFetch(...args); };
    const originalXhrOpen = XMLHttpRequest.prototype.open;
    XMLHttpRequest.prototype.open = function(method, url, ...rest) {
      recordUrl(url);
      return originalXhrOpen.call(this, method, url, ...rest);
    };
    window.addEventListener('error', event => {
      if (event.error) window.__s33Audit.pageErrors.push(String(event.error.message || event.error));
    });
    window.addEventListener('unhandledrejection', event => {
      window.__s33Audit.pageErrors.push(String(event.reason?.message || event.reason));
    });
  });
  const command = async (name, payload) => {
    const requestId = `s33-live-cancel-${name}-${Date.now()}-${Math.random()}`;
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
  const commandLog = async after => page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), after);

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
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 CANCEL approval ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-cancel-approval-${stamp}`;
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
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const account = await command('account.cancellation.fixture.seed', {
    workspaceId, providerId: 'binance', label: `Synthetic CANCEL approval ${stamp}`,
  });
  await command('binance.testnet.cancel.fixture', { scenario: 'PREPARE_ORDER' });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  await command('time.revalidate', { workspaceId });
  check((await command('time.status', { workspaceId })).confidence === 'TRUSTED',
    'The synthetic fixture clock is explicitly trusted before arming.');

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  const accountRow = page.locator('.account-row').filter({ hasText: account.label });
  await accountRow.waitFor({ state: 'visible' });
  await accountRow.click();
  await page.getByRole('button', { name: 'Refresh account', exact: true }).press('Enter');

  const reviewButtons = page.getByRole('button', { name: 'Review cancellation', exact: true });
  await reviewButtons.first().waitFor({ state: 'visible', timeout: 20000 });
  const orderRow = reviewButtons.first().locator('xpath=ancestor::tr');
  const orderText = await orderRow.innerText();
  const brokerOrderId = orderText.match(/([A-Z]+:9007199254740995)/)?.[1];
  check(Boolean(brokerOrderId), `The exact synthetic open order is visible: ${orderText}`);
  const exactOrderRow = page.locator('tr').filter({ hasText: brokerOrderId });
  const reviewButton = exactOrderRow.getByRole('button', { name: 'Review cancellation', exact: true });

  await reviewButton.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live cancellation', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const initial = await dialog.innerText();
  for (const value of [account.label, account.connectionId, '9007199254740993', brokerOrderId, 'BTCUSDT', 'BUY', 'NEW', '0.1', 'Exact remaining quantity', 'DISARMED']) {
    check(initial.includes(value), `The cancellation review shows ${value}: ${initial}`);
  }
  const initialIntentId = initial.match(/CANCEL intent ID\s+([^\n]+)/)?.[1];
  const initialIntentHash = initial.match(/Intent hash\s+([^\n]+)/)?.[1];
  check(/^cancel:/.test(initialIntentId ?? ''), `The immutable intent ID is visible: ${initialIntentId}`);
  check(/^sha256:[0-9a-f]{64}$/.test(initialIntentHash ?? ''), `The intent hash is visible: ${initialIntentHash}`);
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Close without a decision',
    'Opening CANCEL review starts on the safe dismissal control.');

  const beforeDismissal = await command('trade.cancel_approval.list', {
    workspaceId, accountId: account.connectionId, brokerOrderId,
  });
  check(beforeDismissal.approvals.length === 0 && beforeDismissal.rejections.length === 0,
    'Opening the review alone creates no approval or rejection.');
  const dismissalCursor = (await commandLog(0)).cursor;
  await page.keyboard.press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  const dismissalLog = await commandLog(dismissalCursor);
  const dismissalMutations = dismissalLog.commands.filter(entry =>
    /^(provider\.(connect|probe)|trade\.execution\.|live\.gateway\.|trade\.cancel_(approve|reject))/.test(entry.command));
  check(dismissalMutations.length === 0,
    `Safe Enter dismisses without a cancellation or execution mutation: ${JSON.stringify(dismissalMutations)}`);
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Review cancellation',
    'Safe dismissal returns focus to the exact order-row trigger.');

  await reviewButton.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Close without a decision',
    'Keyboard reopening restores safe initial focus.');
  const reopenedReview = await dialog.innerText();
  const intentId = reopenedReview.match(/CANCEL intent ID\s+([^\n]+)/)?.[1];
  const intentHash = reopenedReview.match(/Intent hash\s+([^\n]+)/)?.[1];
  check(/^cancel:/.test(intentId ?? '') && /^sha256:[0-9a-f]{64}$/.test(intentHash ?? ''),
    `The refreshed review has a valid immutable CANCEL intent: ${intentId} ${intentHash}`);
  check(intentId !== initialIntentId && intentHash === initialIntentHash,
    'Reopening creates a new intent ID while preserving the hash of unchanged order terms.');
  check(reopenedReview.includes(brokerOrderId) && reopenedReview.includes('Exact remaining quantity\n0.1'),
    'The fresh intent still names the exact provider order and unchanged remaining quantity.');
  check(reopenedReview.includes('Risk decision / policy\nUNAVAILABLE')
    && reopenedReview.includes('Account arming\nDISARMED'),
  'The Binance Live fixture discloses unavailable risk evidence and disarmed state before any decision.');
  check(await dialog.getByRole('button', { name: 'Approve cancellation', exact: true }).isDisabled(),
    'Approval remains disabled while reconciliation blocks the risk decision.');

  const viewports = [];
  for (const [width, height] of [[1280, 900], [768, 640], [390, 844]]) {
    await page.setViewportSize({ width, height });
    await dialog.getByRole('button', { name: 'Close without a decision', exact: true }).scrollIntoViewIfNeeded();
    const geometry = await page.evaluate(() => {
      const dialog = document.querySelector('dialog.live-cancel-dialog[open]');
      if (!dialog) return null;
      const rect = dialog.getBoundingClientRect();
      const buttons = [...dialog.querySelectorAll('.picker-dialog-actions button:not(:disabled)')]
        .map(button => { const box = button.getBoundingClientRect(); return { left: box.left, right: box.right, top: box.top, bottom: box.bottom }; });
      return { modal: dialog.matches(':modal'), width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        left: rect.left, right: rect.right, dialogWidth: dialog.clientWidth, dialogScroll: dialog.scrollWidth,
        buttons, height: innerHeight };
    });
    check(Boolean(geometry) && geometry.modal && geometry.width === width && geometry.scroll <= width && geometry.left >= 0 && geometry.right <= width
      && geometry.dialogScroll <= geometry.dialogWidth
      && geometry.buttons.every(box => box.left >= 0 && box.right <= width && box.top >= 0 && box.bottom <= height + 1),
    `The exact cancellation review and actions fit ${width}px: ${JSON.stringify(geometry)}`);
    viewports.push(geometry);
    if (width === 390) await dialog.screenshot({ path: 'docs/implementation/evidence/s33/live-cancel-review-blocked-390.png' });
  }

  const armCheckbox = dialog.getByRole('checkbox');
  await armCheckbox.check();
  check((await command('time.revalidate', { workspaceId })).confidence === 'TRUSTED',
    'The explicit Arm attempt follows a new trusted-time check.');
  const armCursor = (await commandLog(0)).cursor;
  await dialog.getByRole('button', { name: 'Arm account and refresh this intent', exact: true }).press('Enter');
  const reconciliationAlert = dialog.getByRole('alert').filter({ hasText: 'RECONCILIATION_REQUIRED' });
  await reconciliationAlert.waitFor({ state: 'visible', timeout: 20000 });
  await reconciliationAlert.scrollIntoViewIfNeeded();
  const blockedReview = await dialog.innerText();
  check(blockedReview.includes('Account arming\nDISARMED')
    && blockedReview.includes('Risk decision / policy\nUNAVAILABLE'),
  `The account stays disarmed and risk stays unavailable after Arm is correctly blocked: ${blockedReview}`);
  check(await dialog.getByRole('button', { name: 'Approve cancellation', exact: true }).isDisabled(),
    'Reconciliation-required state keeps approval disabled.');
  await dialog.screenshot({ path: 'docs/implementation/evidence/s33/live-cancel-arming-blocked-390.png' });
  const history = await command('trade.cancel_approval.list', {
    workspaceId, accountId: account.connectionId, brokerOrderId,
  });
  check(history.approvals.length === 0 && history.rejections.length === 0,
    `The reconciliation block persists no approval or rejection: ${JSON.stringify(history)}`);

  const finalLog = await commandLog(armCursor);
  const commands = finalLog.commands.map(entry => entry.command);
  check(commands.filter(name => name === 'account.arm').length === 1,
    `The user explicitly requested Arm once: ${JSON.stringify(commands)}`);
  check(!commands.includes('trade.cancel_approve') && !commands.includes('trade.cancel_reject'),
    `The blocked intent produces no approval or rejection: ${JSON.stringify(commands)}`);
  check(!commands.includes('trade.cancel_reject') && !commands.includes('trade.execution.prepare')
    && !commands.some(name => name.startsWith('live.gateway.')),
  `The blocked intent does not prepare or dispatch the cancellation: ${JSON.stringify(commands)}`);
  const audit = await page.evaluate(() => window.__s33Audit);
  check(audit.externalRequests.length === 0, `The browser reached external hosts: ${JSON.stringify(audit.externalRequests)}`);
  const externalResources = await page.evaluate(() => performance.getEntriesByType('resource')
    .map(entry => { try { return new URL(entry.name); } catch { return null; } })
    .filter(url => url && ['http:', 'https:'].includes(url.protocol)
      && !['127.0.0.1', 'localhost'].includes(url.hostname))
    .map(url => url.origin));
  check(externalResources.length === 0, `The browser loaded external resources: ${JSON.stringify(externalResources)}`);
  check(audit.pageErrors.length === 0, `Page errors: ${JSON.stringify(audit.pageErrors)}`);

  return {
    result: 'PASS_LIMITED',
    armBlockedBy: 'RECONCILIATION_REQUIRED',
    riskDecisionStatus: 'UNAVAILABLE',
    approvalCount: history.approvals.length,
    rejectionCount: history.rejections.length,
    exactIntentBound: reopenedReview.includes(intentId) && reopenedReview.includes(intentHash),
    safeEnterDismissalMutationCount: dismissalMutations.length,
    explicitArmCount: commands.filter(name => name === 'account.arm').length,
    explicitApprovalCount: commands.filter(name => name === 'trade.cancel_approve').length,
    prepareOrGatewayMutationCount: commands.filter(name => name === 'trade.execution.prepare' || name.startsWith('live.gateway.')).length,
    viewports: viewports.map(({ width, scroll, left, right }) => ({ width, documentScroll: scroll, dialogLeft: left, dialogRight: right })),
    externalRequests: audit.externalRequests.length,
    externalResources: externalResources.length,
    pageErrors: audit.pageErrors.length,
  };
}
