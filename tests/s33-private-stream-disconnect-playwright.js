// Run against the isolated Rust-backed browser harness on port 1423.
// Stream health is seeded as synthetic local state; this test never connects to a provider.
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
    const requestId = `s33-stream-disconnect-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.requestId === requestId && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

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
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 private stream disconnect ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-stream-disconnect-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const cursor = (await commandLog(0)).cursor;

  const account = await command('account.private_stream_disconnect.fixture.seed', {
    workspaceId: workspace.workspaceId,
    providerId: 'binance',
    label: `S33 synthetic private stream disconnect ${stamp}`,
  });
  check(account.connectionState === 'CONNECTED' && account.health.connection === 'ONLINE'
    && account.health.authentication === 'VALID' && account.health.credential === 'CONFIGURED',
  'The synthetic stream failure keeps connection and authentication distinct from stream health.');
  check(account.health.privateStream === 'DEGRADED' && account.health.reconciliation === 'DEGRADED'
    && account.health.executionEligibility === 'BLOCKED' && account.health.arming === 'DISARMED',
  'A disconnected private stream is degraded, requires reconciliation, and leaves Live disabled.');
  check(account.health.reason.includes('Private stream disconnected')
    && account.health.reason.includes('no provider request was made'),
  'The local fixture explains the failure and its no-provider boundary.');

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
  const accountRow = page.locator('.account-row').filter({ hasText: account.label });
  await accountRow.waitFor({ state: 'visible' });
  await accountRow.press('Enter');

  const detail = page.locator('.account-detail');
  await detail.getByRole('heading', { name: account.label, exact: true }).waitFor({ state: 'visible' });
  const recovery = detail.getByRole('alert', { name: 'STREAM_DISCONNECTED error recovery', exact: true });
  await recovery.waitFor({ state: 'visible' });
  await recovery.getByText('Stream disconnected', { exact: true }).waitFor({ state: 'visible' });
  await recovery.getByText(/Live execution remains disabled until account health is restored/).waitFor({ state: 'visible' });
  await recovery.getByText('Next step: Reload state', { exact: true }).waitFor({ state: 'visible' });
  const arm = detail.getByRole('button', { name: 'Arm Live Trading', exact: true });
  check(await arm.isDisabled(), 'A degraded private stream cannot be armed.');
  const healthText = await detail.locator('.health-grid').first().innerText();
  check(healthText.includes('Private stream\nDEGRADED')
    && healthText.includes('Reconciliation\nDEGRADED')
    && healthText.includes('Authentication\nVALID'),
  'The account detail keeps connection, authentication, stream and reconciliation dimensions separate.');

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScroll: document.documentElement.scrollWidth,
      panelClient: document.querySelector('.error-recovery-panel')?.clientWidth ?? 0,
      panelScroll: document.querySelector('.error-recovery-panel')?.scrollWidth ?? 0,
    }));
    check(geometry.documentScroll <= geometry.viewport
      && geometry.panelScroll <= geometry.panelClient + 1,
    `Stream recovery has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(await arm.isDisabled(), `Live remains disabled at ${width}px.`);
    if (width === 390) {
      await detail.scrollIntoViewIfNeeded();
      await detail.screenshot({ path: 'docs/implementation/evidence/s33/private-stream-disconnect-390.png' });
    }
    responsive.push({ width, noHorizontalOverflow: true, liveArmDisabled: true });
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  const settingsSections = page.getByRole('group', { name: 'Settings sections', exact: true });
  await settingsSections.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Account Health', exact: true }).waitFor({ state: 'visible' });
  const healthCard = page.locator('.account-health-card').filter({ hasText: account.label });
  await healthCard.waitFor({ state: 'visible' });
  const dimension = async name => healthCard.locator('.account-health-dimensions > div')
    .filter({ has: page.getByText(name, { exact: true }) }).locator('dd').innerText();
  check((await dimension('Authentication')).trim() === 'VALID'
    && (await dimension('Private stream')).trim() === 'DEGRADED'
    && (await dimension('Reconciliation')).trim() === 'DEGRADED'
    && (await dimension('Arming')).trim() === 'DISARMED',
  'Account Health preserves each independent stream and execution state.');

  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScroll: document.documentElement.scrollWidth,
      cardClient: document.querySelector('.account-health-card')?.clientWidth ?? 0,
      cardScroll: document.querySelector('.account-health-card')?.scrollWidth ?? 0,
    }));
    check(geometry.documentScroll <= geometry.viewport && geometry.cardScroll <= geometry.cardClient + 1,
      `Account Health has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await healthCard.scrollIntoViewIfNeeded();
      await healthCard.screenshot({ path: 'docs/implementation/evidence/s33/private-stream-health-390.png' });
    }
  }

  const finalLog = await commandLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const allowed = new Set([
    'account.private_stream_disconnect.fixture.seed', 'account.list', 'account.get', 'account.activity',
    'domain.snapshot', 'domain.subscribe', 'paper.get', 'provider.list_definitions', 'risk.get_policy',
    'runtime.status', 'time.status', 'trade.cancel_approval.list',
  ]);
  const unexpected = commands.filter(name => !allowed.has(name));
  const forbidden = commands.filter(name => name === 'provider.connect' || name === 'provider.probe'
    || name === 'provider.permissions' || name === 'provider.disconnect' || name === 'account.refresh'
    || name === 'account.arm' || name === 'account.disable_all_live' || name.startsWith('alpaca.')
    || name.startsWith('binance.') || name.startsWith('bitget.') || name.startsWith('trading212.')
    || (name.startsWith('trade.') && name !== 'trade.cancel_approval.list')
    || name.startsWith('live.gateway.') || name.startsWith('paper.order.')
    || name.startsWith('market.hot.') || name.startsWith('data.source.') || name === 'model.verify_route');
  check(unexpected.length === 0, `Only the stream-state fixture and local reads are allowed: ${JSON.stringify(unexpected)}.`);
  check(forbidden.length === 0, `No provider, Arm, or financial command was issued: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}.`);
  return {
    result: 'PASS', screenIds: ['J5', 'K4'], workspace: workspace.workspaceId,
    fixture: 'synthetic local private-stream disconnect; no provider request',
    privateStream: account.health.privateStream, reconciliation: account.health.reconciliation,
    arming: account.health.arming, recoveryAction: 'Reload state', responsive, commands,
    unexpectedCommands: unexpected, forbiddenCommands: forbidden,
    externalHttpRequests: externalRequests.length, consoleErrors, pageErrors,
  };
}
