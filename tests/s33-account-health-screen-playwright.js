// Run against the isolated Rust-backed browser harness on port 1423.
// The only account is inserted by the integration-test fixture seam; it uses
// no credentials and makes no provider request. UI reads remain local.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = `s33-account-health-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 account health ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-account-health-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  const tabs = page.getByRole('group', { name: 'Settings sections', exact: true });
  await tabs.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Account Health', exact: true }).waitFor({ state: 'visible' });
  await page.getByText('Health is shown as separate local observations. Opening this page does not contact a provider.', { exact: true }).waitFor({ state: 'visible' });
  const existingCards = page.locator('.account-health-card');
  await existingCards.first().waitFor({ state: 'visible' });
  check(await existingCards.count() === 1
    && await existingCards.first().getByRole('heading', { name: /Local Paper · LOCAL PAPER · TradeX simulation/ }).isVisible(),
  'The ready fixture exposes only its local simulation observation before the synthetic test account is seeded.');
  const reload = page.getByRole('button', { name: 'Reload health snapshot', exact: true });
  await reload.waitFor({ state: 'visible' });
  let cursor = (await commandLog(0)).cursor;

  const account = await command('account.arming.fixture.seed', {
    workspaceId: workspace.workspaceId,
    providerId: 'trading212',
    label: `S33 synthetic Account Health ${stamp}`,
  });
  check(account.health.reason.includes('no provider request was made'), 'The fixture explicitly records that it made no provider request.');
  await reload.focus();
  check(await reload.evaluate(element => document.activeElement === element), 'Reload health snapshot receives keyboard focus.');
  await page.keyboard.press('Enter');
  const card = page.locator('.account-health-card').filter({ hasText: account.label });
  await card.waitFor({ state: 'visible' });
  check(await page.locator('.account-health-card').count() === 2,
    'The explicit local snapshot reload adds the seeded record alongside Local Paper.');

  const expectedDimensions = [
    ['Connection', `${account.connectionState} · ${account.health.connection}`],
    ['Authentication', account.health.authentication],
    ['Private stream', account.health.privateStream],
    ['Reconciliation', account.health.reconciliation],
    ['Execution eligibility', account.health.executionEligibility],
    ['Arming', account.health.arming],
  ];
  check(new Set(expectedDimensions.map(([, value]) => value)).size > 1,
    'The local fixture carries distinct states instead of one collapsed health value.');
  const readRow = (selector, label) => card.evaluate((element, args) => {
    const row = [...element.querySelectorAll(args.selector)]
      .find(candidate => candidate.querySelector('dt')?.textContent?.trim() === args.label);
    return row?.querySelector('dd')?.textContent?.trim() ?? null;
  }, { selector, label });
  for (const [label, expected] of expectedDimensions) {
    const actual = await readRow('.account-health-dimensions > div', label);
    check(actual !== null, `The ${label} dimension has its own labeled row.`);
    check(actual === expected,
      `${label} shows its independent local state (${expected}).`);
  }
  check(await card.getByRole('heading', { name: account.label, exact: true }).isVisible(), 'The health record identifies the synthetic local account.');
  check((await card.getByText(`${account.providerId} · ${account.environment}`, { exact: true }).innerText()) === `${account.providerId} · ${account.environment}`,
    'Provider name and environment are visibly scoped to the record.');

  const timestampValue = label => card.evaluate((element, name) => {
    const row = [...element.querySelectorAll('.account-health-observations > div')]
      .find(candidate => candidate.querySelector('dt')?.textContent?.trim() === name);
    return row?.querySelector('dd')?.textContent?.trim() ?? null;
  }, label);
  const lastSync = await timestampValue('Last successful sync');
  check(lastSync && lastSync !== 'No observation recorded' && lastSync !== 'Timestamp unavailable',
    'The synthetic successful-sync observation is rendered as a timestamp.');
  check(await timestampValue('Last private stream event') === 'No observation recorded',
    'An absent private-stream timestamp remains explicitly unobserved.');
  const disableAll = page.getByRole('button', { name: 'Disable All Live Execution', exact: true });
  check(await disableAll.isVisible(), 'The separate explicit Disable All control remains available for a Live record.');

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    const height = width === 390 ? 844 : 900;
    await page.setViewportSize({ width, height });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScroll: document.documentElement.scrollWidth,
      cardClient: document.querySelector('.account-health-card')?.clientWidth ?? 0,
      cardScroll: document.querySelector('.account-health-card')?.scrollWidth ?? 0,
    }));
    check(geometry.documentScroll <= geometry.viewport && geometry.cardScroll <= geometry.cardClient + 1,
      `Account Health has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    const dimensionBounds = await card.evaluate((element, labels) => labels.map(label => {
      const row = [...element.querySelectorAll('.account-health-dimensions > div')]
        .find(candidate => candidate.querySelector('dt')?.textContent?.trim() === label);
      const bounds = row?.getBoundingClientRect();
      return { label, x: bounds?.x, width: bounds?.width };
    }), expectedDimensions.map(([label]) => label));
    for (const bounds of dimensionBounds) check(Number.isFinite(bounds.x) && Number.isFinite(bounds.width)
      && bounds.x >= -1 && bounds.x + bounds.width <= width + 1,
    `${bounds.label} stays within ${width}px: ${JSON.stringify(bounds)}`);
    for (const name of ['Reload health snapshot', 'Disable All Live Execution']) {
      const control = page.getByRole('button', { name, exact: true });
      await control.scrollIntoViewIfNeeded();
      const bounds = await control.boundingBox();
      check(bounds && bounds.x >= -1 && bounds.x + bounds.width <= width + 1,
        `${name} stays within ${width}px: ${JSON.stringify(bounds)}`);
    }
    if (width === 390) {
      await card.scrollIntoViewIfNeeded();
      await card.screenshot({ path: 'docs/implementation/evidence/s33/account-health-390.png' });
    }
    responsive.push({ width, noHorizontalOverflow: true, sixDimensionsReachable: true, actionsReachable: true });
  }

  const finalLog = await commandLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const allowed = new Set([
    'account.arming.fixture.seed', 'account.list', 'account.activity', 'domain.snapshot',
    'domain.subscribe', 'paper.get', 'provider.list_definitions', 'runtime.status',
  ]);
  const unexpected = commands.filter(name => !allowed.has(name));
  const forbidden = commands.filter(name => name === 'provider.connect' || name === 'provider.probe'
    || name === 'provider.permissions' || name === 'provider.disconnect' || name === 'account.refresh'
    || name === 'account.disable_all_live' || name.startsWith('alpaca.') || name.startsWith('binance.')
    || name.startsWith('bitget.') || name.startsWith('trading212.') || name.startsWith('trade.')
    || name.startsWith('live.gateway.') || name.startsWith('paper.order.') || name.startsWith('market.hot.')
    || name.startsWith('data.source.') || name === 'model.verify_route');
  check(unexpected.length === 0, `Only local account-health reads and the synthetic seed are allowed: ${JSON.stringify(unexpected)}.`);
  check(forbidden.length === 0, `No provider or action command was issued: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}.`);
  return {
    result: 'PASS', screenId: 'J5', workspace: workspace.workspaceId, fixture: 'synthetic local account; no credentials or provider request',
    dimensions: expectedDimensions, timestamps: { lastSuccessfulSync: lastSync, lastPrivateStreamEvent: 'No observation recorded' },
    responsive, commands, unexpectedCommands: unexpected, forbiddenCommands: forbidden,
    externalHttpRequests: externalRequests.length, consoleErrors, pageErrors, disableAllActivated: false,
  };
}
