// Run `npm run dev:browser -- --port 1423` and then
// `playwright-cli run-code --filename=tests/s33-workspace-import-dialog-playwright.js`.
// The test talks only to that disposable Rust-backed integration harness.
// The harness disables desktop-only controls. The test-only React handler below
// opens the in-app dialog; a narrow Tauri dialog shim supplies a synthetic archive
// path while the Rust-backed workspace.archive.preview command remains real.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const pageErrors = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-import-dialog-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok, `${name} failed: ${JSON.stringify(result.body?.error)}`);
    return result.body.data;
  };
  const commandLog = async cursor => page.evaluate(async after =>
    (await (await fetch(`/__integration/command-log?after=${after}`)).json()), cursor);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1423/');
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 import dialog ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-import-dialog-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'EUR', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  const backup = await command('workspace.archive.backup', { workspaceId: workspace.workspaceId });
  const destinationParent = bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'));
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('EUR');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('group', { name: 'Settings sections', exact: true }).getByRole('button', { name: 'Data & Storage', exact: true }).click();
  await page.getByRole('heading', { name: 'Data & Storage', exact: true }).waitFor({ state: 'visible' });
  const trigger = page.getByRole('button', { name: 'Import / Restore', exact: true });
  check(await trigger.isDisabled(), 'The native file-picker action stays disabled in the browser harness.');
  const returnTarget = page.getByRole('group', { name: 'Settings sections', exact: true })
    .getByRole('button', { name: 'Data & Storage', exact: true });
  await returnTarget.focus();
  check(await returnTarget.evaluate(element => document.activeElement === element), 'A stable prior focus target is set for the component-level check.');
  const before = await commandLog(0);

  const openDialog = async () => {
    // The integration harness intentionally disables desktop-only controls.
    // Call only the component's open handler; all subsequent UI actions use
    // normal locators and keyboard input.
    await trigger.evaluate(button => {
      const propsKey = Object.keys(button).find(key => key.startsWith('__reactProps$'));
      const onClick = propsKey ? button[propsKey]?.onClick : undefined;
      if (typeof onClick !== 'function') throw new Error('React click handler is unavailable.');
      onClick();
    });
  };
  const dialog = page.getByRole('dialog', { name: 'Import / Restore Workspace', exact: true });
  await openDialog();
  await dialog.waitFor({ state: 'visible' });
  check(await dialog.getAttribute('aria-modal') === 'true', 'Import/Restore is exposed as a modal dialog.');
  check(await dialog.getByRole('button', { name: 'Choose archive', exact: true }).isVisible(), 'Archive chooser is present.');
  check(await page.evaluate(() => document.querySelector('.app-shell')?.inert === true), 'The app shell is inert while the dialog is open.');
  check(await dialog.getByRole('button', { name: 'Choose archive', exact: true }).evaluate(element => document.activeElement === element),
    'Opening the dialog places focus on the safe archive-selection action.');

  const focusable = dialog.locator('button:not(:disabled), input:not(:disabled), summary, a[href], [tabindex]:not([tabindex="-1"])');
  const focusableCount = await focusable.count();
  check(focusableCount === 3, `Expected Close, Choose archive, and Cancel controls: ${focusableCount}`);
  for (let index = 0; index < focusableCount + 1; index += 1) {
    await page.keyboard.press('Tab');
    check(await dialog.evaluate(element => element.contains(document.activeElement)), `Tab ${index + 1} keeps focus in the dialog.`);
  }
  for (let index = 0; index < focusableCount + 1; index += 1) {
    await page.keyboard.press('Shift+Tab');
    check(await dialog.evaluate(element => element.contains(document.activeElement)), `Shift+Tab ${index + 1} keeps focus in the dialog.`);
  }
  await page.keyboard.press('Escape');
  await dialog.waitFor({ state: 'hidden' });
  for (let attempt = 0; attempt < 20 && !(await returnTarget.evaluate(element => document.activeElement === element)); attempt += 1) {
    await page.waitForTimeout(10);
  }
  check(await returnTarget.evaluate(element => document.activeElement === element), 'Escape returns focus to the previous Settings control.');

  await returnTarget.focus();
  await openDialog();
  await dialog.waitFor({ state: 'visible' });

  // The browser harness cannot display native file panels. Keep the boundary
  // narrow: only the Tauri dialog plugin call is stubbed; archive validation,
  // summary projection and command logging still run through the Rust process.
  await page.evaluate(({ archivePath, destinationPath }) => {
    window.__TRADEX_TEST_DIALOG_CALLS__ = [];
    window.__TAURI_INTERNALS__ = {
      invoke: async (command, args) => {
        if (command !== 'plugin:dialog|open') throw new Error(`Unexpected Tauri command: ${command}`);
        const options = args?.options ?? {};
        const selected = options.directory ? destinationPath : archivePath;
        window.__TRADEX_TEST_DIALOG_CALLS__.push({ command, options, selected });
        return selected;
      },
    };
  }, { archivePath: backup.path, destinationPath: destinationParent });

  const chooseArchive = dialog.getByRole('button', { name: 'Choose archive', exact: true });
  await chooseArchive.click();
  await dialog.getByRole('status').filter({ hasText: 'Archive validated; the stored schema is supported by this TradeX version.' }).waitFor({ state: 'visible' });
  check(await dialog.getByText(workspaceName, { exact: true }).isVisible(), 'The validated preview shows the archived workspace name.');
  check(await dialog.getByText(workspace.workspaceId, { exact: true }).isVisible(), 'The validated preview shows the archived workspace identity.');
  check(await dialog.getByText('Schema / members', { exact: true }).isVisible(), 'The validated preview shows schema/member metadata.');
  check(await dialog.getByRole('heading', { name: 'Included data', exact: true }).isVisible(), 'The preview provides an import summary.');
  for (const category of ['Accounts', 'Conversations', 'Research', 'Strategies and backtests', 'Financial history']) {
    check(await dialog.getByText(category, { exact: true }).isVisible(), `The archive summary includes ${category}.`);
  }
  const warningList = dialog.locator('ul').filter({ hasText: 'Broker/API secrets are not included' });
  await warningList.scrollIntoViewIfNeeded();
  const warnings = await warningList.innerText();
  for (const expected of [
    'Broker/API secrets are not included',
    'Market history (market.duckdb) and cache files are not included',
    'Restored accounts remain stale and disarmed',
  ]) check(warnings.includes(expected), `The preview states its restore boundary: ${expected}`);
  const accountSummary = dialog.locator('.workspace-data-categories dl > div').filter({ hasText: 'Accounts' });
  check((await accountSummary.innerText()).replace(/\s+/g, ' ').includes('Accounts 1 record'),
    'A single archived account uses the singular record label.');
  const beforeRestoreLabel = dialog.getByText('Before restoring', { exact: true });
  check(await beforeRestoreLabel.isVisible(), 'Restore safeguards are disclosed before confirmation.');
  const restoreNotice = await beforeRestoreLabel.locator('..').innerText();
  check(restoreNotice.includes('backup of the current workspace') && restoreNotice.includes('Live accounts disarmed')
    && restoreNotice.includes('fresh health and reconciliation checks') && restoreNotice.includes('No provider request is made.'),
  `Restore safety disclosure is complete: ${restoreNotice}`);
  const confirmRestore = dialog.getByRole('button', { name: 'Confirm restore', exact: true });
  check(await confirmRestore.isDisabled(), 'Restore cannot be confirmed before a destination is selected.');
  const chooseFolder = dialog.getByRole('button', { name: /Choose folder/ });
  await chooseFolder.click();
  check(await confirmRestore.isEnabled(), 'A selected destination enables explicit restore confirmation.');
  check(await chooseFolder.evaluate(element => document.activeElement === element), 'The preview stage focuses the destination-selection control.');

  const previewFocusable = dialog.locator('button:not(:disabled), input:not(:disabled), summary, a[href], [tabindex]:not([tabindex="-1"])');
  const previewFocusableCount = await previewFocusable.count();
  check(previewFocusableCount === 4, `Expected Close, Choose folder, Cancel, and Confirm restore controls: ${previewFocusableCount}`);
  for (let index = 0; index < previewFocusableCount + 1; index += 1) {
    await page.keyboard.press('Tab');
    check(await dialog.evaluate(element => element.contains(document.activeElement)), `Preview Tab ${index + 1} stays inside the dialog.`);
  }
  for (let index = 0; index < previewFocusableCount + 1; index += 1) {
    await page.keyboard.press('Shift+Tab');
    check(await dialog.evaluate(element => element.contains(document.activeElement)), `Preview Shift+Tab ${index + 1} stays inside the dialog.`);
  }

  const dialogCalls = await page.evaluate(() => window.__TRADEX_TEST_DIALOG_CALLS__);
  check(dialogCalls.length === 2 && dialogCalls[0].options.directory === false
    && dialogCalls[0].options.filters?.some(filter => filter.extensions?.includes('tradex'))
    && dialogCalls[1].options.directory === true,
  `The UI requests a single .tradex archive and a restore folder: ${JSON.stringify(dialogCalls)}`);

  const observations = [];
  for (const width of [1280, 768, 390]) {
    const height = width === 390 ? 844 : 900;
    await page.setViewportSize({ width, height });
    await dialog.evaluate(element => { element.scrollTop = 0; });
    const geometry = await dialog.evaluate(element => {
      const bounds = element.getBoundingClientRect();
      return {
        viewport: document.documentElement.clientWidth,
        documentScroll: document.documentElement.scrollWidth,
        dialogWidth: element.clientWidth,
        dialogScroll: element.scrollWidth,
        left: bounds.left,
        right: bounds.right,
      };
    });
    check(geometry.documentScroll <= geometry.viewport && geometry.dialogScroll <= geometry.dialogWidth
      && geometry.left >= 0 && geometry.right <= geometry.viewport,
    `The dialog has no horizontal clipping at ${width}px: ${JSON.stringify(geometry)}`);
    for (const control of [
      dialog.getByRole('button', { name: 'Close import dialog', exact: true }),
      chooseFolder,
      dialog.getByRole('button', { name: 'Cancel', exact: true }),
      confirmRestore,
    ]) {
      await control.scrollIntoViewIfNeeded();
      const bounds = await control.evaluate(element => {
        const box = element.getBoundingClientRect();
        return { left: box.left, right: box.right, top: box.top, bottom: box.bottom };
      });
      check(bounds.left >= 0 && bounds.right <= width && bounds.top >= 0 && bounds.bottom <= height + 1,
        `A dialog action can be scrolled fully into view at ${width}px: ${JSON.stringify(bounds)}`);
    }
    if (width === 390) {
      await dialog.evaluate(element => { element.scrollTop = 0; });
      await dialog.screenshot({ path: 'docs/implementation/evidence/s33/workspace-import-preview-390.png' });
      await beforeRestoreLabel.scrollIntoViewIfNeeded();
      await dialog.screenshot({ path: 'docs/implementation/evidence/s33/workspace-import-safeguards-390.png' });
    }
    observations.push({ width, noHorizontalOverflow: true, actionsScrollableIntoView: true });
  }

  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  await dialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.querySelector('.app-shell')?.inert === false), 'Dismissal restores the app shell.');

  const after = await commandLog(before.cursor);
  const allowedCommands = new Set([
    'account.activity', 'account.list', 'domain.snapshot', 'domain.subscribe', 'paper.get',
    'provider.list_definitions', 'risk.get_policy', 'runtime.status', 'workspace.retention.get',
    'data.source.catalog', 'data.source.connection', 'data.calendar.connection',
    'data.actions.connection', 'data.instrument.connection', 'data.fx.connection',
    'data.binance_rules.connection', 'data.binance_market.connection', 'workspace.archive.preview',
  ]);
  const unexpectedCommands = after.commands.filter(entry => !allowedCommands.has(entry.command));
  const previews = after.commands.filter(entry => entry.command === 'workspace.archive.preview');
  check(previews.length === 1 && unexpectedCommands.length === 0,
    `The UI issues one validated preview and only passive local reads: ${JSON.stringify(after.commands)}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0, `Uncaught browser errors occurred: ${JSON.stringify(pageErrors)}`);
  return {
    result: 'PASS',
    workspaceId: workspace.workspaceId,
    backupArchivePath: backup.path,
    boundary: 'harness-only trigger bypass and Tauri dialog shim; real Rust archive preview; no restore confirmation',
    initialFocusSafe: true,
    appShellInertWhileOpen: true,
    forwardAndReverseTabStayedInside: true,
    escapeRestoredPriorFocus: true,
    limitation: 'The browser harness disables the desktop-only trigger and native file panel; the test bypasses only the trigger and shims the Tauri dialog API, so it does not verify the real OS picker or restore publication.',
    previewSummary: 'workspace identity, schema/member metadata, all five record categories, and pre-restore safeguards shown',
    dialogCalls: dialogCalls.map(call => ({ command: call.command, directory: call.options.directory, filters: call.options.filters })),
    previewCommands: previews.length,
    restoreCommands: after.commands.filter(entry => entry.command === 'workspace.archive.restore').length,
    totalCommandsAfterBaseline: after.commands.length,
    commandCountsAfterBaseline: Object.fromEntries([...new Set(after.commands.map(entry => entry.command))]
      .map(name => [name, after.commands.filter(entry => entry.command === name).length])),
    observedCommands: after.commands.map(entry => entry.command),
    unexpectedCommands: unexpectedCommands.map(entry => entry.command),
    observations,
    controlsReachableAt390: true,
    pageErrors,
    externalRequests: externalRequests.length,
  };
}
