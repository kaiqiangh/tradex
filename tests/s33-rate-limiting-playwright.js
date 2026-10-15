// Run against the isolated Rust-backed browser harness on port 1423.
// The account.refresh UI request is intercepted before Rust; no provider call is made.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  const intercepted = [];
  page.on('request', request => {
    const requestUrl = request.url();
    const isHttp = /^https?:\/\//i.test(requestUrl);
    const isLocal = /^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl);
    if (isHttp && !isLocal) externalRequests.push(requestUrl.match(/^https?:\/\/[^/]+/i)?.[0] ?? requestUrl);
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = `s33-rate-limit-${name}-${Date.now()}-${Math.random()}`;
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

  await page.unroute('**/__integration/command');
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
  const workspaceName = `S33 rate limiting ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-rate-limit-${stamp}`;
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
    label: `S33 synthetic rate limit ${stamp}`,
  });
  check(account.health.reason.includes('no provider request was made'),
    'The provider-shaped account is only a local fixture and made no provider request.');

  await page.route('**/__integration/command', async route => {
    const request = route.request();
    const envelope = request.postDataJSON();
    if (envelope.command !== 'account.refresh') { await route.continue(); return; }
    intercepted.push({ command: envelope.command, connectionId: envelope.payload.connectionId });
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        requestId: envelope.requestId,
        schemaVersion: 1,
        ok: false,
        error: {
          category: 'RATE_LIMITED',
          code: 'PROVIDER_RATE_LIMITED',
          message: 'Synthetic rate-limit response; provider integration was skipped.',
          retryable: true,
          blocking: true,
          remediationActions: [],
        },
      }),
    });
  });

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
  const accountRow = page.locator('.account-row').filter({ hasText: account.label });
  await accountRow.waitFor({ state: 'visible' });
  await accountRow.press('Enter');
  const detail = page.locator('.account-detail');
  await detail.getByRole('heading', { name: account.label, exact: true }).waitFor({ state: 'visible' });
  await detail.getByRole('button', { name: 'Refresh account', exact: true }).click();

  const banner = page.locator('.error-banner');
  await banner.getByText(/RATE_LIMITED · Provider rate limit reached/).waitFor({ state: 'visible' });
  await banner.getByText(/Back off low-priority provider traffic until the provider cooldown ends/).waitFor({ state: 'visible' });
  await banner.getByText(/keep execution reconciliation at highest priority/).waitFor({ state: 'visible' });
  await banner.getByText(/Next step: Retry after cooldown/).waitFor({ state: 'visible' });
  check(!(await banner.innerText()).includes('Synthetic rate-limit response'),
    'The UI does not echo the raw synthetic error message.');
  check(intercepted.some(item => item.command === 'account.refresh' && item.connectionId === account.connectionId),
    `The explicit Refresh account UI received the synthetic rate-limit response before Rust: ${JSON.stringify(intercepted)}.`);
  const liveArm = detail.getByRole('button', { name: 'Arm Live Trading', exact: true });
  check(await liveArm.isDisabled(), 'The existing Live gate remains disabled while the synthetic rate-limit error is shown.');

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => {
      const banner = document.querySelector('.error-banner');
      return {
        viewport: document.documentElement.clientWidth,
        documentScroll: document.documentElement.scrollWidth,
        bannerClient: banner?.clientWidth ?? 0,
        bannerScroll: banner?.scrollWidth ?? 0,
      };
    });
    check(geometry.documentScroll <= geometry.viewport && geometry.bannerScroll <= geometry.bannerClient + 1,
      `Rate-limit recovery fits at ${width}px: ${JSON.stringify(geometry)}.`);
    await banner.getByText(/keep execution reconciliation at highest priority/).waitFor({ state: 'visible' });
    const identity = detail.getByRole('heading', { name: account.label, exact: true });
    check(await identity.isVisible() && await identity.innerText() === account.label,
      `The selected account identity remains visible at ${width}px.`);
    const identityBounds = await identity.boundingBox();
    check(identityBounds && identityBounds.x >= -1 && identityBounds.x + identityBounds.width <= width + 1,
      `The selected account identity fits at ${width}px: ${JSON.stringify(identityBounds)}.`);
    check(await liveArm.isDisabled(), `Live remains disabled at ${width}px.`);
    check(await banner.getByRole('button', { name: 'Reload account state', exact: true }).isVisible(),
      `The local account-state reload remains reachable at ${width}px.`);
    check(await detail.getByRole('button', { name: 'Refresh account', exact: true }).isVisible(),
      `The explicit provider refresh control remains visible at ${width}px.`);
    if (width === 390) {
      await banner.scrollIntoViewIfNeeded();
      await banner.screenshot({ path: 'docs/implementation/evidence/s33/rate-limit-390.png' });
    }
    responsive.push({ width, noHorizontalOverflow: true, recoveryVisible: true });
  }

  const finalLog = await commandLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const allowed = new Set([
    'account.arming.fixture.seed', 'account.list', 'account.get', 'account.activity', 'domain.snapshot',
    'domain.subscribe', 'paper.get', 'provider.list_definitions', 'risk.get_policy', 'runtime.status',
    'time.status', 'trade.cancel_approval.list',
  ]);
  const unexpected = commands.filter(name => !allowed.has(name));
  const forbidden = commands.filter(name => name === 'provider.connect' || name === 'provider.probe'
    || name === 'provider.permissions' || name === 'provider.disconnect' || name === 'account.refresh'
    || name === 'account.arm' || name === 'account.disable_all_live' || name.startsWith('alpaca.')
    || name.startsWith('binance.') || name.startsWith('bitget.') || name.startsWith('trading212.')
    || (name.startsWith('trade.') && name !== 'trade.cancel_approval.list')
    || name.startsWith('live.gateway.') || name.startsWith('paper.order.')
    || name.startsWith('market.hot.') || name.startsWith('data.source.') || name === 'model.verify_route');
  check(unexpected.length === 0, `Only synthetic local state and passive reads are allowed: ${JSON.stringify(unexpected)}.`);
  check(forbidden.length === 0, `No provider or financial operation was issued: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}.`);
  return {
    result: 'PASS', screenId: 'K5', workspace: workspace.workspaceId,
    fixture: 'account.refresh UI request intercepted before Rust with RATE_LIMITED; no provider request',
    interceptedCommands: intercepted, recovery: 'back off low-priority traffic; execution reconciliation highest priority',
    responsive, commands, unexpectedCommands: unexpected, forbiddenCommands: forbidden,
    externalHttpRequests: externalRequests.length, consoleErrors, pageErrors,
  };
}
