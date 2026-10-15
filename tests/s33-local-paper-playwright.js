// Run with `playwright-cli run-code --filename=tests/s33-local-paper-playwright.js`
// against `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser`.
// All orders are disposable TRADEX_SIMULATION state; no provider is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
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
  const setPaperMode = async mode => {
    const status = await page.evaluate(async value => {
      const response = await fetch('/__integration/paper-mode', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ mode: value }),
      });
      return response.status;
    }, mode);
    check(status === 204, `Local Paper fixture mode ${mode} was rejected.`);
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await setPaperMode('normal');
  await page.goto('http://127.0.0.1:1420/');
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 Local Paper ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-paper-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });

  await setPaperMode('delay');
  const navigation = page.getByRole('navigation', { name: 'Primary navigation' });
  const accountWidths = [];
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('status').filter({ hasText: 'Loading Local Paper simulation…' }).waitFor({ state: 'visible' });
  const paperHistory = page.getByRole('region', { name: 'Local Paper order history', exact: true });
  await paperHistory.waitFor({ state: 'visible' });
  check(await paperHistory.getByText('No Local Paper orders yet.', { exact: true }).count() === 1,
    'A fresh simulation has an explicit empty order-history state.');
  check(await page.getByRole('heading', { name: 'TradeX simulation · TRADEX_SIMULATION', exact: true }).isVisible(),
    'The Local Paper history stays identified as TradeX simulation.');
  for (const width of [768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    const bounds = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    check(bounds.scroll <= bounds.viewport, `Local Paper account has no page overflow at ${width}px: ${JSON.stringify(bounds)}`);
    check(await page.getByRole('heading', { name: 'TradeX simulation · TRADEX_SIMULATION', exact: true }).isVisible(),
      `Simulation identity remains visible at ${width}px.`);
    accountWidths.push(width);
  }
  await page.setViewportSize({ width: 1280, height: 900 });
  await setPaperMode('normal');
  await page.getByRole('button', { name: 'Refresh simulation quote', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Simulation quote refreshed at' }).waitFor({ state: 'visible' });

  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('2');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('100');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Evaluate risk', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: /RiskDecision ALLOWED saved at/ }).waitFor({ state: 'visible' });
  await page.getByRole('article', { name: 'ALLOWED RiskDecision', exact: true }).waitFor({ state: 'visible' });
  const submit = page.getByRole('button', { name: 'Submit Local Paper order', exact: true });
  await submit.waitFor({ state: 'visible' });
  const dialog = page.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true });
  const beforeSafeEnter = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());
  await submit.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  check(await dialog.getAttribute('aria-modal') === 'true', 'Submission confirmation is modal.');
  check((await dialog.innerText()).includes('This changes the TradeX simulation only.'),
    'Submission confirmation names the simulation boundary.');
  const safeFocus = await page.evaluate(() => document.activeElement?.textContent?.trim());
  check(safeFocus === 'Keep reviewing', `Local Paper confirmation safe focus was ${safeFocus}`);
  check(await page.locator('.app-shell').evaluate(element => element.inert),
    'The application background is inert while the submission dialog is open.');
  const keepReviewingSubmit = dialog.getByRole('button', { name: 'Keep reviewing', exact: true });
  const confirmSubmit = dialog.getByRole('button', { name: 'Confirm submit', exact: true });
  await keepReviewingSubmit.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Confirm submit',
    'Tab moves from the safe dismissal action to explicit Local Paper confirmation.');
  await confirmSubmit.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'Tab wraps to the safe dismissal action.');
  await keepReviewingSubmit.press('Shift+Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Confirm submit',
    'Shift+Tab wraps to the explicit confirmation action.');
  await page.keyboard.press('Escape');
  await dialog.waitFor({ state: 'hidden' });
  check(!(await page.locator('.app-shell').evaluate(element => element.inert)),
    'Escape restores the application background.');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Submit Local Paper order',
    'Escape returns focus to the Local Paper submission trigger.');
  const afterSafeEnter = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), beforeSafeEnter.cursor);
  check(!afterSafeEnter.commands.some(entry => entry.command === 'paper.order.submit'),
    'Tab navigation and Escape do not submit a simulation order.');

  const submissionDialogWidths = [];
  for (const width of [768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await submit.press('Enter');
    await dialog.waitFor({ state: 'visible' });
    check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
      `Local Paper submission has safe focus at ${width}px.`);
    const geometry = await page.evaluate(() => {
      const dialog = document.querySelector('[role="dialog"]');
      const rect = dialog?.getBoundingClientRect();
      const actions = [...(dialog?.querySelectorAll('.picker-dialog-actions button') ?? [])]
        .map(button => button.getBoundingClientRect());
      return { viewport: document.documentElement.clientWidth, page: document.documentElement.scrollWidth,
        height: document.documentElement.clientHeight, left: rect?.left ?? -1, right: rect?.right ?? Infinity,
        top: rect?.top ?? -1, bottom: rect?.bottom ?? Infinity,
        actionsInside: actions.every(button => button.left >= 0 && button.right <= window.innerWidth
          && button.top >= 0 && button.bottom <= window.innerHeight) };
    });
    check(geometry.page <= geometry.viewport && geometry.left >= 0 && geometry.right <= width
      && geometry.top >= 0 && geometry.bottom <= geometry.height && geometry.actionsInside,
    `Local Paper submission dialog and actions fit at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await dialog.screenshot({ path: 'docs/implementation/evidence/s33/local-paper-submit-390.png' });
    }
    await page.keyboard.press('Escape');
    await dialog.waitFor({ state: 'hidden' });
    check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Submit Local Paper order',
      `Escape restores the submission trigger at ${width}px.`);
    submissionDialogWidths.push(width);
  }
  await page.setViewportSize({ width: 1280, height: 900 });
  await submit.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Submit Local Paper order',
    'Enter on Keep reviewing dismisses and restores focus without approval.');
  const afterSafeEnterAgain = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), beforeSafeEnter.cursor);
  check(!afterSafeEnterAgain.commands.some(entry => entry.command === 'paper.order.submit'),
    'Enter on the safe dismissal action does not submit a simulation order.');
  let paper = await command('paper.get', { workspaceId });
  check(paper.orders.length === 0 && paper.fills.length === 0,
    'Safe dismissal leaves Local Paper orders and fills unchanged.');

  await submit.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  await dialog.getByRole('button', { name: 'Confirm submit', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'is FILLED' }).waitFor({ state: 'visible' });
  paper = await command('paper.get', { workspaceId });
  check(paper.providerId === 'local-paper' && paper.environment === 'LOCAL',
    `The result remains scoped to Local Paper: ${paper.providerId}/${paper.environment}`);
  check(paper.orders.length === 1 && paper.orders[0].state === 'FILLED'
    && paper.orders[0].filledQuantity === '2' && paper.orders[0].remainingQuantity === '0',
  `Explicit confirmation produces the deterministic full-fill simulation state: ${JSON.stringify(paper.orders)}`);
  check(paper.fills.length === 1 && paper.fills[0].quantity === '2',
    `The synthetic fill matches the confirmed quantity: ${JSON.stringify(paper.fills)}`);
  check((await page.getByText('TRADEX_SIMULATION · FILLED', { exact: true }).count()) === 1,
    'The page labels the outcome as a TradeX simulation fill.');

  const afterSubmit = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), beforeSafeEnter.cursor);
  const providerCommands = afterSubmit.commands.filter(entry =>
    /^(provider\.(connect|probe)|account\.refresh|trade\.execution\.|live\.gateway\.)/.test(entry.command));
  check(providerCommands.length === 0, `Local Paper flow issued no provider or Gateway command: ${JSON.stringify(providerCommands)}`);

  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('combobox', { name: 'Simulation scenario', exact: true }).selectOption('partial-v1');
  await page.getByRole('button', { name: 'Apply scenario', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Scenario partial-v1 is active.' }).waitFor({ state: 'visible' });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('button', { name: 'New draft', exact: true }).click();
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('4');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('100');
  await page.getByRole('button', { name: 'Save draft', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Evaluate risk', exact: true }).click();
  await page.getByRole('status').filter({ hasText: /RiskDecision ALLOWED saved at/ }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Submit Local Paper order', exact: true }).click();
  const partialSubmitDialog = page.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true });
  await partialSubmitDialog.waitFor({ state: 'visible' });
  await partialSubmitDialog.getByRole('button', { name: 'Confirm submit', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'is PARTIALLY_FILLED' }).waitFor({ state: 'visible' });
  const cancel = page.getByRole('button', { name: 'Cancel Local Paper order', exact: true });
  await cancel.waitFor({ state: 'visible' });
  const beforeCancelDecision = await page.evaluate(async () =>
    (await fetch('/__integration/command-log?after=0')).json());
  const allProviderCommands = beforeCancelDecision.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh|model\.verify_route|data\..*\.(probe|refresh)|trade\.execution\.|live\.gateway\.)/.test(entry.command));
  check(allProviderCommands.length === 0,
    `The full Local Paper run issued no provider/model/data/Gateway command: ${JSON.stringify(allProviderCommands)}`);
  await cancel.click();
  const cancelDialog = page.getByRole('dialog', { name: 'Confirm Local Paper cancellation', exact: true });
  await cancelDialog.waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'Local Paper cancellation opens on its safe dismissal action.');
  check(await page.locator('.app-shell').evaluate(element => element.inert),
    'The application background is inert while the cancellation dialog is open.');
  const keepReviewing = cancelDialog.getByRole('button', { name: 'Keep reviewing', exact: true });
  const confirmCancel = cancelDialog.getByRole('button', { name: 'Confirm cancel', exact: true });
  check(await keepReviewing.count() === 1 && await confirmCancel.count() === 1,
    'The safe dismissal and explicit cancellation controls have accessible names.');
  await page.keyboard.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Confirm cancel',
    'Tab moves to the explicit cancellation control.');
  await page.keyboard.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'Tab wraps to the safe dismissal control.');
  await page.keyboard.press('Shift+Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Confirm cancel',
    'Shift+Tab wraps to the explicit cancellation control.');
  await page.keyboard.press('Escape');
  await cancelDialog.waitFor({ state: 'hidden' });
  check(!(await page.locator('.app-shell').evaluate(element => element.inert)),
    'The application background is restored after dismissal.');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Cancel Local Paper order',
    'Focus returns to the cancellation trigger.');
  const afterSafeCancelEnter = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), beforeCancelDecision.cursor);
  check(!afterSafeCancelEnter.commands.some(entry => entry.command === 'paper.order.cancel'),
    'Dialog keyboard navigation and Escape must not cancel the simulated order.');
  const afterSafeCancelPaper = await command('paper.get', { workspaceId });
  const partialOrder = afterSafeCancelPaper.orders.find(order => order.state === 'PARTIALLY_FILLED');
  check(Boolean(partialOrder && partialOrder.filledQuantity === '2' && partialOrder.remainingQuantity === '2'),
    `Safe dismissal must preserve the partial simulation fill and remaining quantity: ${JSON.stringify(afterSafeCancelPaper.orders)}`);
  check(afterSafeCancelPaper.fills.some(fill => fill.orderId === partialOrder.orderId && fill.quantity === '2'),
    `The partial simulation fill remains recorded after safe dismissal: ${JSON.stringify(afterSafeCancelPaper.fills)}`);

  const dialogWidths = [];
  for (const width of [768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await cancel.click();
    const responsiveCancelDialog = page.getByRole('dialog', { name: 'Confirm Local Paper cancellation', exact: true });
    await responsiveCancelDialog.waitFor({ state: 'visible' });
    check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
      `Safe cancellation focus is retained at ${width}px.`);
    const geometry = await page.evaluate(() => {
      const dialog = document.querySelector('[role="dialog"]');
      const rect = dialog?.getBoundingClientRect();
      const actions = [...(dialog?.querySelectorAll('.picker-dialog-actions button') ?? [])]
        .map(button => button.getBoundingClientRect());
      return {
        viewport: document.documentElement.clientWidth,
        page: document.documentElement.scrollWidth,
        left: rect?.left ?? -1,
        right: rect?.right ?? Infinity,
        actionsInside: actions.every(button => button.left >= 0 && button.right <= window.innerWidth),
      };
    });
    check(geometry.page <= geometry.viewport && geometry.left >= 0 && geometry.right <= width && geometry.actionsInside,
      `Local Paper cancellation dialog fits at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await responsiveCancelDialog.screenshot({
        path: 'docs/implementation/evidence/s33/local-paper-cancel-390.png',
      });
    }
    await responsiveCancelDialog.getByRole('button', { name: 'Keep reviewing', exact: true }).click();
    await responsiveCancelDialog.waitFor({ state: 'hidden' });
    check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Cancel Local Paper order',
      `Dismissal restores the trigger focus at ${width}px.`);
    dialogWidths.push(width);
  }

  return {
    result: 'PASS',
    boundary: 'Local Paper / TRADEX_SIMULATION only',
    safeFocus,
    approvals: 'none',
    orders: afterSafeCancelPaper.orders.length,
    fills: afterSafeCancelPaper.fills.length,
    safeCancelDialog: 'inert background; Tab/Shift+Tab wrap; Escape restores focus without paper.order.cancel; partial order unchanged',
    safeSubmitDialog: 'inert background; Tab/Shift+Tab wrap; Escape and Enter on Keep reviewing do not submit; focus returns; 768/390 geometry',
    providerCommands: providerCommands.length,
    responsiveWidths: { account: accountWidths, submissionDialog: submissionDialogWidths, cancellationDialog: dialogWidths },
  };
}
