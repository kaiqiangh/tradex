// Run against the isolated Rust-backed browser harness on port 1423.
// Reopening the temporary workspace exercises local startup invalidation only;
// this CLI-backed harness does not start the desktop provider recovery worker.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const requestUrl = request.url();
    const isHttp = /^https?:\/\//i.test(requestUrl);
    const isLocal = /^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl);
    if (isHttp && !isLocal) externalRequests.push(requestUrl.match(/^https?:\/\/[^/]+/i)?.[0] ?? requestUrl);
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = 's33-startup-' + name + '-' + Date.now() + '-' + Math.random();
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.requestId === requestId && result.body.ok,
      name + ' failed: ' + JSON.stringify(result.body.error));
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await (await fetch('/__integration/command-log?after=' + cursor)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1423/');
  const isolationNotice = page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  );
  await isolationNotice.waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await isolationNotice.waitFor({ state: 'visible' });

  const bootstrap = await command('workspace.open', {});
  const stamp = Date.now() + '-' + Math.random().toString(36).slice(2);
  const workspaceName = 'S33 startup reconciliation ' + stamp;
  const workspacePath = bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/')) + '/s33-startup-' + stamp;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const cursor = (await commandLog(0)).cursor;
  const account = await command('account.arming.fixture.seed', {
    workspaceId: workspace.workspaceId,
    providerId: 'trading212',
    label: 'S33 synthetic startup reconciliation ' + stamp,
  });
  check(account.health.reason.includes('no provider request was made'),
    'The only Live account is an explicitly synthetic, no-request fixture.');
  check(account.health.arming === 'DISARMED', 'The synthetic Live account starts disarmed.');

  // Reopen the same local database. The ControlPlane invalidates persisted
  // provider health; no account.refresh or native startup worker is dispatched.
  const reopened = await command('workspace.open', { path: workspacePath });
  check(reopened.workspaceId === workspace.workspaceId, 'The existing temporary workspace was reopened.');
  const accounts = await command('account.list', { workspaceId: workspace.workspaceId });
  const staleAccount = accounts.accounts.find(item => item.connectionId === account.connectionId);
  check(staleAccount, 'The persisted synthetic account remains available after reopen.');
  check(staleAccount.health.connection === 'STALE'
    && staleAccount.health.authentication === 'UNVERIFIED'
    && staleAccount.health.credential === 'UNCHECKED'
    && staleAccount.health.reconciliation === 'STALE'
    && staleAccount.health.executionEligibility === 'BLOCKED'
    && staleAccount.health.arming === 'DISARMED',
  'Workspace reopen durably invalidates the old observation and keeps Live blocked/disarmed: '
    + JSON.stringify(staleAccount.health));
  check(staleAccount.health.reason.includes('Re-test the connection'),
    'The stale projection retains an explicit local remediation reason.');

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
  const row = page.locator('.account-row').filter({ hasText: account.label });
  await row.waitFor({ state: 'visible' });
  await row.press('Enter');
  const detail = page.locator('.account-detail');
  await detail.getByRole('heading', { name: account.label, exact: true }).waitFor({ state: 'visible' });
  await detail.getByText(staleAccount.health.reason, { exact: true }).waitFor({ state: 'visible' });
  for (const [label, expected] of [
    ['Connection', 'STALE'], ['Authentication', 'UNVERIFIED'], ['Credential', 'UNCHECKED'],
    ['Reconciliation', 'STALE'], ['Execution eligibility', 'BLOCKED'], ['Arming', 'DISARMED'],
  ]) {
    const rowValue = await detail.evaluate((element, name) => {
      const item = [...element.querySelectorAll('.health-grid > div')]
        .find(candidate => candidate.querySelector('dt')?.textContent?.trim() === name);
      return item?.querySelector('dd')?.textContent?.trim() ?? null;
    }, label);
    check(rowValue === expected, label + ' remains visible as ' + expected + ', found ' + rowValue + '.');
  }
  const arm = detail.getByRole('button', { name: 'Arm Live Trading', exact: true });
  check(await arm.isDisabled(), 'A stale, unreconciled account cannot be armed.');

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    const height = width === 390 ? 844 : 900;
    await page.setViewportSize({ width, height });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScroll: document.documentElement.scrollWidth,
      detail: document.querySelector('.account-detail')?.getBoundingClientRect().toJSON() ?? null,
    }));
    check(geometry.documentScroll <= geometry.viewport,
      'Accounts has no document-level horizontal overflow at ' + width + 'px: ' + JSON.stringify(geometry));
    check(geometry.detail && geometry.detail.x >= -1 && geometry.detail.x + geometry.detail.width <= width + 1,
      'The selected account identity and recovery detail fit at ' + width + 'px: ' + JSON.stringify(geometry));
    for (const text of [staleAccount.health.reason, 'STALE', 'UNVERIFIED', 'BLOCKED', 'DISARMED']) {
      check(await detail.getByText(text, { exact: true }).first().isVisible(),
        'The ' + text + ' recovery evidence remains visible at ' + width + 'px.');
    }
    check(await arm.isDisabled(), 'Arm remains unavailable at ' + width + 'px.');
    if (width === 390) {
      await detail.scrollIntoViewIfNeeded();
      await detail.screenshot({ path: 'docs/implementation/evidence/s33/startup-reconciliation-390.png' });
    }
    responsive.push({ width, noHorizontalOverflow: true, staleReasonVisible: true, armDisabled: true });
  }

  const finalLog = await commandLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const allowed = new Set([
    'account.arming.fixture.seed', 'account.get', 'account.list', 'account.activity', 'domain.snapshot',
    'domain.subscribe', 'paper.get', 'provider.list_definitions', 'risk.get_policy', 'runtime.status',
    'trade.cancel_approval.list', 'workspace.open',
  ]);
  const unexpected = commands.filter(name => !allowed.has(name));
  const providerOrAction = commands.filter(name => name === 'provider.connect' || name === 'provider.probe'
    || name === 'provider.permissions' || name === 'provider.disconnect' || name === 'account.refresh'
    || name.startsWith('alpaca.') || name.startsWith('binance.') || name.startsWith('bitget.')
    || name.startsWith('trading212.') || (name.startsWith('trade.') && name !== 'trade.cancel_approval.list') || name.startsWith('live.gateway.')
    || name.startsWith('paper.order.') || name.startsWith('market.hot.') || name.startsWith('data.source.')
    || name === 'model.verify_route' || name === 'account.arm');
  check(unexpected.length === 0, 'Only local projection, workspace, and synthetic fixture commands are allowed: '
    + JSON.stringify(unexpected));
  check(providerOrAction.length === 0, 'No provider, reconciliation-read, or financial action command was issued: '
    + JSON.stringify(providerOrAction));
  check(externalRequests.length === 0, 'Browser made external HTTP requests: ' + JSON.stringify(externalRequests));
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    'Browser errors occurred: ' + JSON.stringify({ consoleErrors, pageErrors }));
  return {
    result: 'PASS', screenId: 'K2', workspace: workspace.workspaceId,
    fixture: 'temporary persisted synthetic Live record; no credential or provider request',
    reopenedHealth: staleAccount.health, remediationReason: staleAccount.health.reason,
    responsive, commands, unexpectedCommands: unexpected, providerOrActionCommands: providerOrAction,
    externalHttpRequests: externalRequests.length, consoleErrors, pageErrors, armActivated: false,
    scope: 'local stale/disarmed startup projection only; desktop provider recovery is intentionally skipped',
  };
}
