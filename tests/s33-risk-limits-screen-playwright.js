// Run against the isolated Rust-backed integration server on port 1422.
// J2 coverage is read-only and uses one disposable, fixture-ready workspace.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
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
    const requestId = `s33-risk-limits-${name}-${Date.now()}-${Math.random()}`;
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
  const readLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1422/');
  const isolationNotice = page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  );
  await isolationNotice.waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await isolationNotice.waitFor({ state: 'visible' });

  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 risk limits ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-risk-limits-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  let cursor = (await readLog(0)).cursor;

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  const tabs = page.getByRole('group', { name: 'Settings sections', exact: true });
  await tabs.getByRole('button', { name: 'Risk & Limits', exact: true }).press('Enter');

  const summary = page.getByText(
    'Money and portfolio exposure use exact decimals in USD. Leave limits unset until you choose them. Quantity uses canonical instrument base units.',
    { exact: true },
  );
  await summary.waitFor({ state: 'visible' });
  const decimalFields = [
    'Maximum order notional (USD)',
    'Maximum order quantity (shares or base units)',
    'Maximum position size (USD)',
    'Maximum single-instrument exposure (%)',
    'Maximum EQUITY exposure (%)',
    'Maximum CRYPTO_SPOT exposure (%)',
    'Maximum daily traded notional (USD)',
    'Maximum daily realized loss (USD)',
    'Maximum reserved capital (USD)',
    'Maximum market-order slippage (%)',
    'Maximum price deviation (%)',
  ];
  const listFields = [
    'Allowed canonical instruments', 'Blocked canonical instruments',
    'Allowed venues', 'Blocked venues', 'Allowed account IDs', 'Blocked account IDs',
  ];
  const field = (role, name) => page.getByRole(role, { name: new RegExp(`^${name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}(?:\\s|$)`) });
  const blankDecimalFields = decimalFields.filter(name => name !== 'Maximum single-instrument exposure (%)');
  for (const name of decimalFields) await field('textbox', name).waitFor({ state: 'visible' });
  for (const name of listFields) await field('textbox', name).waitFor({ state: 'visible' });
  await page.getByRole('spinbutton', { name: 'Maximum open orders', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('spinbutton', { name: 'Stale quote threshold (seconds)', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('spinbutton', { name: 'Live inactivity timeout (minutes)', exact: true }).waitFor({ state: 'visible' });

  for (const name of blankDecimalFields) {
    check(await page.getByRole('textbox', { name, exact: true }).inputValue() === '',
      `${name} should stay unset in the disposable default policy.`);
  }
  for (const name of listFields) {
    check((await field('textbox', name).inputValue()) === '', `${name} should have no list restriction by default.`);
  }
  check(await page.getByRole('textbox', { name: 'Maximum single-instrument exposure (%)', exact: true }).inputValue() === '10.25',
    'The deterministic fixture risk policy should display its explicit single-instrument exposure value.');
  check(await page.getByRole('spinbutton', { name: 'Maximum open orders', exact: true }).inputValue() === '',
    'Maximum open orders should remain unset.');
  check(await page.getByRole('spinbutton', { name: 'Stale quote threshold (seconds)', exact: true }).inputValue() === '3',
    'The fixture displays the configured stale quote threshold in seconds.');
  check(await page.getByRole('spinbutton', { name: 'Live inactivity timeout (minutes)', exact: true }).inputValue() === '20',
    'The fixture displays the configured Live inactivity timeout in minutes.');
  check(!(await page.getByRole('checkbox', { name: /Allow market orders/ }).isChecked()),
    'Market orders must remain disabled in the fixture.');
  await page.getByText('Empty means no allow-list restriction. Blocked instruments always reject.', { exact: true }).waitFor({ state: 'visible' });
  await page.getByText('Matching account IDs are rejected even if listed as allowed.', { exact: true }).waitFor({ state: 'visible' });
  await page.getByText('No selection means no additional environment restriction.', { exact: true }).waitFor({ state: 'visible' });
  for (const environment of ['Local Paper', 'Paper', 'Demo', 'Testnet', 'Live']) {
    const checkbox = page.getByRole('checkbox', { name: environment, exact: true });
    await checkbox.waitFor({ state: 'visible' });
    check(!(await checkbox.isChecked()), `No allowed environment should mean no added restriction: ${environment}.`);
  }
  const hardRules = page.locator('.hard-rules');
  await page.getByRole('heading', { name: 'Hard safety rules · read only', exact: true }).waitFor({ state: 'visible' });
  check(await hardRules.locator('li').count() > 0, 'Backend-owned read-only hard rules are present.');
  await page.getByRole('button', { name: 'Save risk defaults', exact: true }).waitFor({ state: 'visible' });

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
    await summary.scrollIntoViewIfNeeded();
    const visibleFields = [];
    for (const name of [...decimalFields, ...listFields, 'Maximum open orders', 'Stale quote threshold (seconds)', 'Live inactivity timeout (minutes)']) {
      const control = field(name.includes('orders') || name.includes('seconds') || name.includes('minutes') ? 'spinbutton' : 'textbox', name);
      await control.scrollIntoViewIfNeeded();
      const rect = await control.evaluate(element => {
        const box = element.getBoundingClientRect();
        return { left: box.left, right: box.right, width: box.width };
      });
      check(rect.width > 0 && rect.left >= -1 && rect.right <= width + 1,
        `${name} is clipped horizontally at ${width}px: ${JSON.stringify(rect)}.`);
      visibleFields.push(name);
    }
    for (const name of ['Local Paper', 'Paper', 'Demo', 'Testnet', 'Live']) {
      const checkbox = page.getByRole('checkbox', { name, exact: true });
      await checkbox.scrollIntoViewIfNeeded();
      const rect = await checkbox.evaluate(element => {
        const box = element.getBoundingClientRect();
        return { left: box.left, right: box.right, width: box.width };
      });
      check(rect.width > 0 && rect.left >= -1 && rect.right <= width + 1,
        `${name} environment control is clipped at ${width}px: ${JSON.stringify(rect)}.`);
    }
    const marketOrders = page.getByRole('checkbox', { name: /Allow market orders/ });
    await marketOrders.scrollIntoViewIfNeeded();
    const marketOrderRect = await marketOrders.evaluate(element => {
      const box = element.getBoundingClientRect();
      return { left: box.left, right: box.right, width: box.width };
    });
    check(marketOrderRect.width > 0 && marketOrderRect.left >= -1 && marketOrderRect.right <= width + 1,
      `Market-order toggle is clipped at ${width}px: ${JSON.stringify(marketOrderRect)}.`);
    await hardRules.scrollIntoViewIfNeeded();
    const rulesRect = await hardRules.evaluate(element => {
      const box = element.getBoundingClientRect();
      return { left: box.left, right: box.right, width: box.width };
    });
    check(rulesRect.width > 0 && rulesRect.left >= -1 && rulesRect.right <= width + 1,
      `Read-only hard rules are clipped at ${width}px: ${JSON.stringify(rulesRect)}.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      riskFormClient: document.querySelector('.risk-form')?.clientWidth ?? 0,
      riskFormScroll: document.querySelector('.risk-form')?.scrollWidth ?? 0,
    }));
    check(geometry.scroll <= geometry.viewport && geometry.riskFormScroll <= geometry.riskFormClient + 1,
      `Risk & Limits overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: 'docs/implementation/evidence/s33/risk-limits-390.png', fullPage: true });
    }
    observations.push({ width, fieldsReachable: visibleFields.length, environmentControlsReachable: 5,
      marketOrderToggleReachable: true, hardRulesVisible: true, noOverflow: true });
  }

  const finalLog = await readLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const allowedCommands = new Set([
    'account.activity', 'account.list', 'domain.snapshot', 'domain.subscribe',
    'paper.get', 'provider.list_definitions', 'risk.get_policy', 'runtime.status',
  ]);
  const unexpected = commands.filter(name => !allowedCommands.has(name));
  check(unexpected.length === 0, `J2 browsing issued a command outside its passive-read allowlist: ${JSON.stringify(unexpected)}.`);
  const forbidden = commands.filter(name => name === 'risk.save_policy' || name === 'provider.connect'
    || name === 'provider.probe' || name === 'provider.permissions' || name === 'provider.disconnect'
    || name === 'account.refresh' || name.startsWith('alpaca.') || name.startsWith('binance.')
    || name.startsWith('bitget.') || name.startsWith('trading212.') || name.startsWith('trade.')
    || name.startsWith('live.gateway.') || name.startsWith('paper.order.') || name.startsWith('market.hot.')
    || name === 'model.verify_route' || name.startsWith('data.source.'));
  check(forbidden.length === 0, `J2 browsing issued a prohibited provider or mutation command: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}.`);
  return {
    result: 'PASS', screenId: 'J2', workspace: workspace.workspaceId, widths: [1280, 768, 390],
    observations, commands, allowedCommands: [...allowedCommands], unexpectedCommands: unexpected,
    forbiddenCommands: forbidden, externalHttpRequests: externalRequests.length,
    consoleErrors, pageErrors, policySaveCommands: 0,
  };
}
