// Run with `playwright-cli run-code --filename=tests/s33-account-delete-dialog-playwright.js`
// against `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser`.
// Uses disconnected Demo records and a test-only eligible Live fixture in a disposable workspace;
// no provider credentials or external requests are used.
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
    check(result.status === 200 && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 Delete Dialog ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-delete-dialog-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const label = `S33 synthetic delete ${Date.now()}`;
  const target = await command('account.delete.fixture.seed', { workspaceId: workspace.workspaceId, label });
  const sibling = await command('account.delete.fixture.seed', { workspaceId: workspace.workspaceId, label });
  check(target.connectionId !== sibling.connectionId, 'Duplicate labels have distinct local connection IDs.');
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).click();
  const seededAccounts = await command('account.list', { workspaceId: workspace.workspaceId });
  const targetIndex = seededAccounts.accounts.findIndex(account => account.connectionId === target.connectionId);
  check(targetIndex >= 0, 'The exact synthetic deletion target is in the account list.');
  const accountRow = page.locator('.account-row').nth(targetIndex);
  await accountRow.waitFor({ state: 'visible' });
  await accountRow.press('Enter');
  const deleteTrigger = page.getByRole('button', { name: 'Delete local account', exact: true });
  await deleteTrigger.waitFor({ state: 'visible' });
  const dialog = page.getByRole('dialog', { name: 'Delete local account?', exact: true });
  const cursor = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());

  await deleteTrigger.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  const dialogText = await dialog.innerText();
  check(dialogText.includes(label) && dialogText.includes(target.connectionId),
    'The confirmation identifies the exact selected record despite its duplicate label.');
  check(dialogText.includes('does not contact Trading 212'),
    'The confirmation states that local deletion does not contact the provider.');
  check(await page.evaluate(() => document.querySelector('dialog.account-delete-dialog')?.matches(':modal')),
    'The native dialog is in the browser modal top layer.');
  const cancel = dialog.getByRole('button', { name: 'Cancel', exact: true });
  const confirm = dialog.getByRole('button', { name: 'Confirm permanent deletion', exact: true });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Cancel',
    'The safe Cancel action receives initial focus.');
  await cancel.press('Shift+Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Confirm permanent deletion',
    'Shift+Tab wraps within the native confirmation.');
  await confirm.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Cancel',
    'Tab wraps within the native confirmation.');

  const responsive = [];
  for (const width of [768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    const geometry = await page.evaluate(() => {
      const dialog = document.querySelector('dialog.account-delete-dialog');
      const button = dialog?.querySelector('.picker-dialog-actions button.primary');
      const box = button?.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        left: box?.left, right: box?.right, width: window.innerWidth };
    });
    check(geometry.scroll <= geometry.viewport, `Account deletion has no document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.left >= 0 && geometry.right <= geometry.width,
      `The explicit delete action fits within ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/account-delete-390.png' });
    responsive.push(width);
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await cancel.press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Delete local account',
    'Safe dismissal restores focus to its invoking action.');
  const afterEnter = await page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), cursor.cursor);
  check(!afterEnter.commands.some(entry => entry.command === 'account.delete'),
    'Enter on the safe default does not issue account.delete.');
  let accounts = await command('account.list', { workspaceId: workspace.workspaceId });
  check(accounts.accounts.filter(account => account.label === label).length === 2
    && accounts.accounts.some(account => account.connectionId === target.connectionId)
    && accounts.accounts.some(account => account.connectionId === sibling.connectionId),
    'Safe Enter dismissal preserves both duplicate-label records.');

  await deleteTrigger.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  await page.keyboard.press('Escape');
  await dialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Delete local account',
    'Escape dismissal restores focus to the invoking action.');
  accounts = await command('account.list', { workspaceId: workspace.workspaceId });
  check(accounts.accounts.filter(account => account.label === label).length === 2
    && accounts.accounts.some(account => account.connectionId === target.connectionId)
    && accounts.accounts.some(account => account.connectionId === sibling.connectionId),
    'Escape dismissal also leaves both local records unchanged.');
  const runLog = await page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), cursor.cursor);
  const deletionForbiddenCommands = runLog.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.(refresh|delete)$|trade\.execution\.|live\.gateway\.)/.test(entry.command));
  check(deletionForbiddenCommands.length === 0,
    `Deletion-dialog keyboard checks issued no delete/provider/Gateway/execution command: ${JSON.stringify(deletionForbiddenCommands)}`);

  const positiveDeleteCursor = (await page.evaluate(async () =>
    (await fetch('/__integration/command-log?after=0')).json())).cursor;
  await deleteTrigger.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Cancel',
    'The explicit positive path still opens on the safe Cancel action.');
  await confirm.click();
  await dialog.waitFor({ state: 'hidden' });
  await page.getByRole('status').filter({ hasText: `Deleted local account ${label} (trading212 · DEMO).` })
    .waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.id) === 'connections-title',
    'After the selected record is removed, focus returns to the stable Account connections heading.');
  accounts = await command('account.list', { workspaceId: workspace.workspaceId });
  check(!accounts.accounts.some(account => account.connectionId === target.connectionId)
    && accounts.accounts.some(account => account.connectionId === sibling.connectionId)
    && accounts.accounts.some(account => account.providerId === 'local-paper')
    && accounts.accounts.length === 2,
  'Confirmation removes only the selected synthetic Demo record, preserving its same-label sibling and Local Paper.');
  const afterPositiveDelete = await page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), positiveDeleteCursor);
  const deleteCommands = afterPositiveDelete.commands.filter(entry => entry.command === 'account.delete');
  check(deleteCommands.length === 1,
    `The positive path issues exactly one account.delete after confirming the selected connection: ${JSON.stringify(deleteCommands)}`);
  const unexpectedPositiveCommands = afterPositiveDelete.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh|trade\.execution\.|live\.gateway\.)/.test(entry.command));
  check(unexpectedPositiveCommands.length === 0,
    `Positive local deletion does not contact providers or financial execution: ${JSON.stringify(unexpectedPositiveCommands)}`);

  const workspaceId = workspace.workspaceId;
  const risk = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: risk.stateVersion,
    policy: { ...risk.policy, maxSingleInstrumentExposurePercent: '10.25' },
  });
  await command('workspace.ready.fixture', { workspaceId });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  const trustedTime = await command('time.revalidate', { workspaceId });
  check(trustedTime.confidence === 'TRUSTED', 'The isolated fixture uses revalidated synthetic time.');
  const armAccount = await command('account.arming.fixture.seed', {
    workspaceId, providerId: 'trading212', label: `S33 synthetic Arm ${Date.now()}`,
  });
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).click();
  const armAccounts = await command('account.list', { workspaceId });
  const armIndex = armAccounts.accounts.findIndex(account => account.connectionId === armAccount.connectionId);
  check(armIndex >= 0, 'The synthetic Live arming fixture is in the account list.');
  const armEligibility = armAccounts.liveArmingEligibility.find(item => item.connectionId === armAccount.connectionId);
  const armState = armAccounts.accounts[armIndex];
  check(armEligibility?.canArm, `The synthetic fixture follows the normal Arm gates: ${JSON.stringify({ armEligibility, health: armState.health, connectionState: armState.connectionState, permissions: armState.permissions })}`);
  await page.locator('.account-row').nth(armIndex).press('Enter');
  const armTrigger = page.getByRole('button', { name: 'Arm Live Trading', exact: true });
  await armTrigger.waitFor({ state: 'visible' });
  check(await armTrigger.isEnabled(), `The synthetic Live account can open its explicit Arm review: ${JSON.stringify(armEligibility)}`);
  const armDialog = page.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  const beforeArmReview = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());
  await armTrigger.press('Enter');
  await armDialog.waitFor({ state: 'visible' });
  const armDialogText = await armDialog.innerText();
  for (const identity of [armAccount.providerId, armAccount.label, 'LIVE', armAccount.connectionId, armAccount.data.remoteAccountId]) {
    check(armDialogText.includes(identity), `The Arm review names the exact account identity ${identity}.`);
  }
  check(armDialogText.includes('does not approve or submit an order'),
    'The Arm review states that arming is separate from order approval and submission.');
  check(await page.evaluate(() => document.querySelector('dialog.live-arm-dialog')?.matches(':modal')),
    'The Live Arm confirmation uses the native modal top layer.');
  const keepReviewing = armDialog.getByRole('button', { name: 'Keep reviewing', exact: true });
  const confirmArm = armDialog.getByRole('button', { name: 'Arm this Live account', exact: true });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'The safe Live Arm dismissal action receives initial focus.');
  await keepReviewing.press('Shift+Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Arm this Live account',
    'Shift+Tab wraps from the safe action to the explicit Arm action.');
  await confirmArm.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'Tab wraps back to the safe Live Arm dismissal action.');
  const armResponsive = [];
  for (const width of [768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    const geometry = await page.evaluate(() => {
      const dialog = document.querySelector('dialog.live-arm-dialog');
      const button = dialog?.querySelector('.picker-dialog-actions button.primary');
      const box = button?.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        left: box?.left, right: box?.right, width: window.innerWidth };
    });
    check(geometry.scroll <= geometry.viewport, `Live Arm has no document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.left >= 0 && geometry.right <= geometry.width,
      `The explicit Arm action fits within ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/live-arm-390.png' });
    armResponsive.push(width);
  }
  await page.setViewportSize({ width: 1280, height: 900 });
  await keepReviewing.press('Enter');
  await armDialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Arm Live Trading',
    'Safe Live Arm dismissal restores focus to its invoking action.');
  const afterArmDismiss = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), beforeArmReview.cursor);
  check(!afterArmDismiss.commands.some(entry => entry.command === 'account.arm'),
    'Enter on the safe Live Arm action does not arm the account.');
  await armTrigger.press('Enter');
  await armDialog.waitFor({ state: 'visible' });
  await page.keyboard.press('Escape');
  await armDialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Arm Live Trading',
    'Allowed Escape dismissal restores focus to the Live Arm trigger.');
  const unchangedArmAccount = (await command('account.list', { workspaceId })).accounts
    .find(account => account.connectionId === armAccount.connectionId);
  check(unchangedArmAccount?.health.arming === 'DISARMED', 'Safe Live Arm dismissal leaves the account disarmed.');
  const finalLog = await page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), cursor.cursor);
  const forbiddenCommands = finalLog.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.(refresh|arm)$|trade\.(approve|cancel_approve|cancel_reject|execution\.)|live\.gateway\.)/.test(entry.command));
  check(forbiddenCommands.length === 0,
    `Both dialog keyboard checks issued no delete/arm/provider/Gateway/execution command: ${JSON.stringify(forbiddenCommands)}`);

  return {
    result: 'PASS',
    boundary: 'disposable disconnected Demo records and test-only eligible Live fixture; no provider credentials or external requests',
    initialFocus: 'Cancel',
    keyboardTrap: 'Tab and Shift+Tab wrap inside native modal',
    enter: 'dismissed without account.delete; records unchanged',
    escape: 'dismissed; records unchanged',
    focusReturn: 'Delete local account',
    positiveDelete: 'one confirmed synthetic account removed; same-label sibling and Local Paper preserved; no provider/Gateway/execution commands',
    forbiddenCommands: forbiddenCommands.length + unexpectedPositiveCommands.length,
    liveArm: 'Tab and Shift+Tab wrap; Enter on Keep reviewing leaves the account DISARMED',
    responsiveWidths: { accountDelete: responsive, liveArm: armResponsive },
  };
}
