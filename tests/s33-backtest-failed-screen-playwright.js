// Start the isolated Rust-backed integration server with:
// ./node_modules/.bin/vite --host 127.0.0.1 --mode integration --port 1422 --strictPort
// Then run this callback with mcp__playwright__browser_run_code_unsafe, filename:
// tests/s33-backtest-failed-screen-playwright.js
// Covers H4 with a disposable workspace, a typed synthetic failure, and one local retry.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  const command = async (name, payload) => {
    const requestId = `s33-backtest-failed-${name}-${Date.now()}-${Math.random()}`;
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
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });

  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 backtest failed ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-backtest-failed-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const cursor = (await readLog(0)).cursor;

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Strategies', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
  const editor = page.getByRole('region', { name: 'Strategy editor', exact: true });
  await editor.getByRole('textbox', { name: 'Name', exact: true }).fill(`S33 failed strategy ${stamp}`);
  await editor.getByRole('button', { name: 'Save version', exact: true }).press('Enter');
  await editor.getByText(/Selected .* · v1 · sha256:/).waitFor({ state: 'visible' });
  const selectedStrategy = await editor.getByText(/Selected .* · v1 · sha256:/).innerText();

  const backtest = page.getByRole('region', { name: 'Backtest', exact: true });
  const result = backtest.locator('.strategy-result');
  await backtest.getByLabel('Instrument', { exact: true }).fill('equity:US:AAPL');
  await backtest.getByLabel('Dataset', { exact: true }).fill('historical:fixture');
  await backtest.getByLabel('Start date', { exact: true }).fill('2026-01-01T00:00');
  await backtest.getByLabel('End date', { exact: true }).fill('2026-01-02T00:00');
  await backtest.getByLabel('Starting cash', { exact: true }).fill('125000');
  await backtest.getByLabel('Commission', { exact: true }).fill('0.25');
  await backtest.getByLabel('Slippage', { exact: true }).fill('5.5');
  await backtest.getByLabel('Portfolio seed', { exact: true }).fill('S33 synthetic seed');
  await backtest.getByRole('combobox', { name: 'Backtest integration scenario', exact: true }).selectOption('DATA_GAP');
  const expectedDateRange = (await page.evaluate(() => [
    new Date('2026-01-01T00:00').toISOString(), new Date('2026-01-02T00:00').toISOString(),
  ])).map(value => value.replace('.000Z', 'Z'));
  await backtest.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');

  await result.getByText('FAILED', { exact: true }).waitFor({ state: 'visible' });
  check(await result.getAttribute('aria-live') === 'polite', 'H4 failure updates use the polite live region.');
  const firstText = await result.innerText();
  const firstRunId = /Run ([0-9a-f-]{36})/i.exec(firstText)?.[1];
  const firstRequestHash = /Config sha256:([0-9a-f]{64})/i.exec(firstText)?.[1];
  check(Boolean(firstRunId && firstRequestHash), `H4 exposes run and request identities: ${firstText}`);
  const failure = result.getByRole('alert');
  const failureText = await failure.innerText();
  check(failureText.includes('BACKTEST_DATA_GAP') && failureText.includes('data gap inside the requested range'),
    `H4 shows a concrete typed failure: ${failureText}`);
  check(await result.getByText('review_backtest_data_guards', { exact: true }).isVisible(),
    'H4 retains guard-specific remediation beside the failure.');
  const frozen = backtest.getByRole('region', { name: 'Frozen configuration', exact: true });
  await frozen.waitFor({ state: 'visible' });
  const firstFrozenText = await frozen.innerText();
  check(firstFrozenText.includes('equity:US:AAPL') && firstFrozenText.includes('historical:fixture')
    && firstFrozenText.includes('125000') && firstFrozenText.includes('0.25')
    && firstFrozenText.includes('5.5') && firstFrozenText.includes('S33 synthetic seed'),
  `H4 retains the frozen request inputs: ${firstFrozenText}`);
  check(firstFrozenText.includes(`${expectedDateRange[0]} → ${expectedDateRange[1]}`),
    `H4 retains the exact frozen date range: ${firstFrozenText}`);
  const retry = backtest.getByRole('button', { name: 'Retry backtest', exact: true });
  check(await retry.isEnabled(), 'H4 exposes an enabled, concrete retry action.');
  for (const [label, expected] of [
    ['Instrument', 'equity:US:AAPL'], ['Dataset', 'historical:fixture'],
    ['Start date', '2026-01-01T00:00'], ['End date', '2026-01-02T00:00'],
    ['Starting cash', '125000'], ['Commission', '0.25'], ['Slippage', '5.5'],
    ['Portfolio seed', 'S33 synthetic seed'],
  ]) {
    check(await backtest.getByLabel(label, { exact: true }).inputValue() === expected,
      `Failed inputs remain editable and unchanged for ${label}.`);
  }

  await retry.press('Enter');
  await page.waitForFunction(oldRunId => {
    const text = document.querySelector('.strategy-result')?.textContent ?? '';
    return !text.includes(`Run ${oldRunId}`) && text.includes('BACKTEST_DATA_GAP');
  }, firstRunId);
  await result.getByText('FAILED', { exact: true }).waitFor({ state: 'visible' });
  const retriedText = await result.innerText();
  const retriedRunId = /Run ([0-9a-f-]{36})/i.exec(retriedText)?.[1];
  const retriedRequestHash = /Config sha256:([0-9a-f]{64})/i.exec(retriedText)?.[1];
  check(Boolean(retriedRunId && retriedRunId !== firstRunId), 'Retry creates a distinct backtest run ID.');
  check(retriedRequestHash === firstRequestHash, 'Retry preserves the exact reviewed request identity.');
  check((await frozen.innerText()) === firstFrozenText, 'Retry preserves the displayed frozen configuration.');
  check((await result.getByRole('alert').innerText()).includes('BACKTEST_DATA_GAP'), 'Retry retains the typed failure result.');

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await result.scrollIntoViewIfNeeded();
    await result.getByText('FAILED', { exact: true }).waitFor({ state: 'visible' });
    await retry.scrollIntoViewIfNeeded();
    check((await result.innerText()).includes(`Run ${retriedRunId}`), `Retried run identity changed at ${width}px.`);
    check(await frozen.isVisible(), `H4 frozen inputs are not visible at ${width}px.`);
    check(await retry.isVisible() && await retry.isEnabled(), `H4 retry action is not visible and enabled at ${width}px.`);
    check((await result.getByRole('alert').innerText()).includes('BACKTEST_DATA_GAP'),
      `H4 failure detail is not visible at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
    }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `H4 overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    if (width === 390) {
      await result.screenshot({ path: 'docs/implementation/evidence/s33/backtest-failed-390.png',
        style: '.topbar { visibility: hidden !important; }' });
    }
    observations.push({ width, failed: true, runIdVisible: true, frozenInputsVisible: true,
      failureAndRemediationVisible: true, retryEnabled: true, noOverflow: true });
  }

  const commands = (await readLog(cursor)).commands.map(entry => entry.command);
  const forbidden = commands.filter(name => name.startsWith('provider.') || name.startsWith('alpaca.')
    || name.startsWith('binance.') || name.startsWith('bitget.') || name.startsWith('trading212.')
    || name.startsWith('account.refresh') || name.startsWith('model.verify') || name.startsWith('data.source.')
    || name.startsWith('trade.') || name.startsWith('live.gateway.') || name.startsWith('paper.order.')
    || name === 'strategy.run' || name.startsWith('strategy.cancel')
    || name === 'backtest.cancel');
  check(commands.filter(name => name === 'backtest.run').length === 2,
    'H4 issues exactly one synthetic run and one explicit retry.');
  check(commands.filter(name => name === 'strategy.save_version').length === 1,
    'H4 uses one local immutable strategy version.');
  check(forbidden.length === 0, `H4 browsing issued a provider or financial command: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return { result: 'PASS', screenId: 'H4', workspace: workspace.workspaceId, strategy: selectedStrategy,
    failedRunId: firstRunId, retryRunId: retriedRunId, requestHashPreserved: true,
    widths: [1280, 768, 390], observations, backtestRuns: 2, backtestCancels: 0,
    providerFinancialCommands: forbidden.length, externalHttpRequests: externalRequests.length };
}
