// Actual React -> Rust Control Plane -> disposable SQLite on npm run dev:browser.
// Native secret entry and provider HTTP use integration-only fixtures.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [], consoleErrors = [], pageErrors = [];
  page.on('request', request => {
    const url = request.url();
    if (/^https?:/i.test(url) && !/^https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?\//i.test(url)) {
      externalRequests.push(new URL(url).origin);
    }
  });
  page.on('console', message => {
    if (message.type() === 'error') consoleErrors.push({ message: message.text(), url: message.location().url });
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const envelope = { requestId: `a3-${Date.now()}-${Math.random()}`, schemaVersion: 1, command: name, payload };
    const result = await page.evaluate(async input => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input),
      });
      return { status: response.status, body: await response.json() };
    }, envelope);
    check(result.status === 200 && result.body.ok && result.body.requestId === envelope.requestId,
      `${name}: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const log = after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);
  await page.setViewportSize({ width: 1280, height: 900 });
  // Establish this origin without booting React with a previous harness's expired workspace path.
  await page.goto('http://127.0.0.1:1420/__integration/command-log?after=0');
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.goto('http://127.0.0.1:1420/');
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true }).waitFor();
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const name = `S33 A3 permission review ${stamp}`;
  const path = `${bootstrap.path}-a3-${stamp}`;
  const workspace = await command('workspace.open', { name, baseCurrency: 'USD', path });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(name);
  await page.getByLabel('Local storage', { exact: true }).fill(path);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor();
  const cursor = (await log(0)).cursor;
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  const form = page.locator('.provider-config');
  await form.waitFor({ state: 'visible' });
  const passiveCommands = (await log(cursor)).commands.map(item => item.command);
  check(!passiveCommands.some(name => name === 'provider.connect' || name === 'account.refresh'),
    'Opening Accounts never connects or refreshes a provider');
  const catalog = await command('provider.list_definitions', {});
  const observations = [];
  const blockedLayouts = [];
  const cases = [
    ['alpaca', 'PAPER', 'UNVERIFIED', ['apiKey', 'secret']],
    ['trading212', 'LIVE', 'UNVERIFIED', ['apiKey', 'secret']],
    ['binance', 'LIVE', 'VERIFIED', ['apiKey', 'secret']],
    ['bitget', 'LIVE', 'VERIFIED', ['apiKey', 'secret', 'passphrase']],
  ];
  for (const [providerId, environment, scope, fields] of cases) {
    const expectedArming = environment === 'LIVE' ? 'DISARMED' : 'NOT_APPLICABLE';
    await form.getByRole('combobox', { name: 'Connection source', exact: true }).selectOption('new');
    await form.getByRole('combobox', { name: 'Provider / environment', exact: true }).selectOption(`${providerId}/${environment}`);
    const schema = catalog.providers.find(item => item.providerId === providerId && item.environment === environment);
    check(schema?.available, `The ${providerId}/${environment} fixture schema is available`);
    check(JSON.stringify(schema.fields.map(field => field.id)) === JSON.stringify(fields), `${providerId} has its own credential shape`);
    const formText = await form.innerText();
    for (const field of schema.fields) check(formText.includes(`${field.label} (required)`), `Missing secure field hint ${field.label}`);
    check(await page.locator('input[type="password"]').count() === 0, 'Secret inputs stay outside the renderer');
    const label = `S33 A3 synthetic ${providerId} ${stamp}`;
    await form.getByRole('textbox', { name: 'Connection label', exact: true }).fill(label);
    if (providerId === 'binance') await command('binance.live.permission.fixture', { scenario: 'WITHDRAWAL' });
    await form.getByRole('button', { name: 'Connect account securely', exact: true }).press('Enter');
    const detail = page.locator('.account-detail');
    await detail.getByRole('heading', { name: label, exact: true }).waitFor();
    const permissions = detail.locator('.permission-review').filter({ has: page.getByRole('heading', { name: 'Permission review', exact: true }) });
    if (providerId === 'binance') {
      await permissions.getByRole('alert').filter({ hasText: 'withdrawal' }).waitFor();
      const blocked = (await command('account.list', { workspaceId })).accounts.find(item => item.label === label);
      check(blocked.permissions.forbidden.includes('withdrawal') && blocked.health.arming === 'DISARMED',
        'The actual permission parser blocks the synthetic withdrawal-enabled key');
      check(await permissions.getByRole('button', { name: 'Confirm connection', exact: true }).isDisabled(),
        'Forbidden permissions cannot be confirmed');
      for (const width of [1280, 768, 390]) {
        await page.setViewportSize({ width, height: 900 });
        await permissions.evaluate(element => element.scrollIntoView({ block: 'center' }));
        check(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth),
          `Forbidden-permission remediation overflows at ${width}`);
        blockedLayouts.push({ width, noHorizontalOverflow: true, confirmationDisabled: true });
        if (width === 390) await permissions.screenshot({ path: 'docs/implementation/evidence/s33/a3-forbidden-permission-390.png' });
      }
      await command('binance.live.permission.fixture', { scenario: 'SAFE' });
      // Observe the local provider read cooldown before the user's explicit re-test.
      await page.waitForTimeout(5100);
      await detail.getByRole('button', { name: 'Refresh account', exact: true }).press('Enter');
      await permissions.getByRole('alert').waitFor({ state: 'hidden' });
      await page.setViewportSize({ width: 1280, height: 900 });
    }
    await permissions.getByText(new RegExp(`^${scope} · IP allow-list:`)).waitFor();
    const account = (await command('account.list', { workspaceId })).accounts.find(item => item.label === label);
    check(account?.connectionState === 'REVIEW_REQUIRED', 'Testing does not implicitly confirm a connection');
    check(account.permissions.scope === scope && account.health.arming === expectedArming,
      `Detected scope remains distinct from arming: ${JSON.stringify({ scope: account.permissions.scope, arming: account.health.arming })}`);
    const confirm = permissions.getByRole('button', { name: 'Confirm connection', exact: true });
    if (scope === 'UNVERIFIED') {
      check(await confirm.isDisabled(), 'An unverified review requires explicit acknowledgement');
      const acknowledgement = permissions.getByRole('checkbox');
      await acknowledgement.focus();
      await acknowledgement.press('Space');
      check(await acknowledgement.isChecked(), 'Space operates the permission acknowledgement');
      await acknowledgement.press('Tab');
      check(await confirm.evaluate(element => element === document.activeElement), 'Tab reaches the explicit confirm control');
    }
    const layouts = [];
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: 900 });
      await permissions.evaluate(element => element.scrollIntoView({ block: 'center' }));
      check(await confirm.isEnabled(), 'The reviewed local fixture is confirmable');
      const geometry = await page.evaluate(() => ({
        viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
      }));
      check(geometry.scroll <= geometry.viewport, `${providerId} permission review overflows at ${width}`);
      layouts.push({ width, noHorizontalOverflow: true });
      if (width === 390) await detail.screenshot({ path: `docs/implementation/evidence/s33/a3-${providerId}-permission-390.png` });
    }
    await confirm.press('Enter');
    await detail.getByText(new RegExp(`${providerId} · ${environment} · CONNECTED`)).waitFor();
    const confirmed = await command('account.get', { workspaceId, connectionId: account.connectionId });
    check(confirmed.permissions.scope === scope, 'Acknowledgement cannot promote permission scope');
    check(confirmed.health.arming === expectedArming, 'Connection confirmation never arms Live execution');
    if (scope === 'UNVERIFIED') check(confirmed.permissions.acknowledged, 'The explicit acknowledgement persists');
    if (environment === 'LIVE' && scope === 'UNVERIFIED') {
      check(await detail.getByRole('button', { name: 'Arm Live Trading', exact: true }).isDisabled(), 'Unverified Live scope cannot arm');
    }
    observations.push({ providerId, environment, scope, state: confirmed.connectionState,
      arming: confirmed.health.arming, acknowledged: confirmed.permissions.acknowledged, layouts });
    await page.setViewportSize({ width: 1280, height: 900 });
  }
  await page.reload();
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  const reopened = await command('account.list', { workspaceId });
  for (const observed of observations) {
    const account = reopened.accounts.find(item => item.providerId === observed.providerId && item.environment === observed.environment);
    check(account?.connectionState === 'CONNECTED' && account.permissions.scope === observed.scope
      && account.health.arming === observed.arming, 'Reload preserves connection/scope without arming');
  }
  const commands = (await log(cursor)).commands.map(item => item.command);
  check(commands.filter(item => item === 'provider.connect').length === 8, 'Exactly four explicit test/confirm pairs occur');
  check(commands.filter(item => item === 'account.refresh').length === 1, 'Only the explicit permission-removal re-test refreshes an account');
  const allowed = new Set(['account.activity', 'account.get', 'account.list', 'account.refresh',
    'binance.live.permission.fixture', 'domain.snapshot', 'domain.subscribe', 'paper.get',
    'provider.connect', 'provider.list_definitions', 'runtime.status', 'thread.list',
    'trade.cancel_approval.list', 'trade.execution.order_links', 'workspace.open', 'risk.get_policy']);
  const forbidden = commands.filter(item => !allowed.has(item));
  check(forbidden.length === 0, `Unexpected activation: ${JSON.stringify(forbidden)}`);
  const applicationConsoleErrors = consoleErrors.filter(item => !(item.url.endsWith('/favicon.ico') && item.message.includes('404')));
  check(externalRequests.length === 0 && applicationConsoleErrors.length === 0 && pageErrors.length === 0,
    JSON.stringify({ externalRequests, consoleErrors, pageErrors }));
  return { status: 'PASS_LIMITED', testedAt: new Date().toISOString(), workspaceId, observations, blockedLayouts,
    passiveOpen: { providerConnects: 0, accountRefreshes: 0 },
    explicitFixtureConnections: 4, forbiddenCommands: forbidden.length,
    commandCounts: Object.fromEntries([...new Set(commands)].sort().map(name => [name, commands.filter(item => item === name).length])),
    externalRequests: externalRequests.length, consoleErrors, applicationConsoleErrors: applicationConsoleErrors.length, pageErrors: pageErrors.length,
    limitations: ['Synthetic HTTP and secret entry only', 'Native secret window, native accessibility and other forbidden/unsupported permission variants remain unverified', 'No real provider entitlement or S33 completion'] };
}
