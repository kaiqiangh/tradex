// Run against the isolated Rust-backed browser harness on port 1423.
// About reports local build/runtime facts; this test performs no provider I/O.
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
    const requestId = `s33-about-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok, `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1423/');
  await page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  ).waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 About ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-about-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const cursor = (await commandLog(0)).cursor;
  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  const tabs = page.getByRole('group', { name: 'Settings sections', exact: true });
  const aboutTab = tabs.getByRole('button', { name: 'About', exact: true });
  await aboutTab.press('Enter');
  await page.getByRole('heading', { name: 'About', exact: true }).waitFor({ state: 'visible' });
  check(await aboutTab.getAttribute('aria-pressed') === 'true', 'Keyboard Enter selects the About section.');

  const about = page.locator('.about-settings');
  await page.getByRole('heading', { name: 'About TradeX', exact: true }).waitFor({ state: 'visible' });
  const facts = page.locator('.about-facts dd');
  const version = (await facts.nth(0).innerText()).trim();
  const application = (await facts.nth(1).innerText()).trim();
  const baseline = (await facts.nth(2).innerText()).trim();
  const prototypeDisclaimer = (await facts.nth(3).innerText()).trim();
  check(/^\d+\.\d+\.\d+(?:[-+][\w.-]+)?$/.test(version), `Application version is visible: ${version}`);
  check(application === 'Native TradeX desktop workspace', `Application identity is accurate: ${application}`);
  check(baseline === 'TradeX v1.0 · Revision C', `The documentation baseline is visible: ${baseline}`);
  check(prototypeDisclaimer === 'Historical clickable fixture; it is not the running application.',
    `The historical prototype is clearly distinguished: ${prototypeDisclaimer}`);

  await page.getByRole('heading', { name: 'Local runtime', exact: true }).waitFor({ state: 'visible' });
  const components = page.locator('.component-list > li');
  await components.first().waitFor({ state: 'visible' });
  check(await components.count() === 4, 'About renders all four local runtime component rows.');
  const runtimeRows = await components.evaluateAll(rows => rows.map(row => ({
    component: row.querySelector('strong')?.textContent?.trim() ?? '',
    message: row.querySelector('p')?.textContent?.trim() ?? '',
    status: row.querySelector('.badge')?.textContent?.trim() ?? '',
  })));
  check(runtimeRows.every(row => row.component && row.message && row.status),
    `Each runtime row shows a component, status, and explanation: ${JSON.stringify(runtimeRows)}`);
  for (const component of ['Control Plane', 'Codex App Server', 'CLIProxyAPI', 'Order Gateway']) {
    check(runtimeRows.some(row => row.component === component), `Runtime status includes ${component}.`);
  }

  const refresh = page.getByRole('button', { name: 'Refresh runtime status', exact: true });
  await refresh.press('Enter');
  await page.waitForFunction(() => !document.querySelector('.about-section-heading button:disabled'));
  check(await refresh.isEnabled(), 'The runtime refresh control returns to its ready state.');
  check(await components.count() === 4, 'Refreshing local runtime status preserves the component snapshot.');

  const requiredFacts = [
    [page.getByRole('heading', { name: 'About TradeX', exact: true }), 'About title'],
    [page.getByText('Application version', { exact: true }), 'application version label'],
    [page.getByText('TradeX v1.0 · Revision C', { exact: true }), 'documentation baseline'],
    [page.getByText('Historical clickable fixture; it is not the running application.', { exact: true }), 'prototype disclaimer'],
    [page.getByRole('heading', { name: 'Local runtime', exact: true }), 'local runtime heading'],
    [refresh, 'runtime refresh control'],
  ];
  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    for (const [fact, label] of requiredFacts) {
      check(await fact.isVisible(), `About keeps the ${label} visible at ${width}px.`);
    }
    const visibleRows = await components.evaluateAll(rows => rows.every(row => {
      const box = row.getBoundingClientRect();
      return getComputedStyle(row).visibility !== 'hidden' && box.width > 0 && box.height > 0;
    }));
    check(await components.count() === 4 && visibleRows, `All runtime component rows remain visible at ${width}px.`);
    const geometry = await page.evaluate(() => {
      const root = document.documentElement;
      const surface = document.querySelector('.about-settings')?.getBoundingClientRect();
      return { viewport: root.clientWidth, documentScroll: root.scrollWidth, left: surface?.left, right: surface?.right };
    });
    check(geometry.documentScroll <= geometry.viewport && geometry.left >= -1 && geometry.right <= width + 1,
      `About stays within ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await about.screenshot({ path: 'docs/implementation/evidence/s33/about-390.png' });
    }
    responsive.push({ width, requiredFactsVisible: true, allRuntimeRowsRendered: true, noHorizontalOverflow: true });
  }

  const finalLog = await commandLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const allowedCommands = new Set([
    'account.activity', 'account.list', 'domain.snapshot', 'domain.subscribe',
    'paper.get', 'provider.list_definitions', 'runtime.status', 'thread.list', 'time.status',
  ]);
  const unexpectedCommands = commands.filter(name => !allowedCommands.has(name));
  check(unexpectedCommands.length === 0, `About emitted commands outside the local status allowlist: ${JSON.stringify(unexpectedCommands)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}.`);
  return {
    result: 'PASS', screenId: 'J7', workspace: workspace.workspaceId,
    version, application, documentationBaseline: baseline, prototypeDisclaimer,
    runtimeRows, responsive, observedCommands: [...new Set(commands)], commandCount: commands.length, unexpectedCommands,
    externalHttpRequests: externalRequests.length, consoleErrors, pageErrors,
  };
}
