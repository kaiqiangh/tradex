// Start the isolated Rust-backed integration server with:
// ./node_modules/.bin/vite --host 127.0.0.1 --mode integration --port 1422 --strictPort
// Then run this callback with mcp__playwright__browser_run_code_unsafe, filename:
// tests/s33-backtest-result-screen-playwright.js
// Covers H5 with a disposable workspace and one synthetic successful backtest.
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
    const requestId = `s33-backtest-result-${name}-${Date.now()}-${Math.random()}`;
    const response = await page.evaluate(async envelope => {
      const result = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: result.status, body: await result.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(response.status === 200 && response.body.requestId === requestId && response.body.ok,
      `${name} failed: ${JSON.stringify(response.body.error)}`);
    return response.body.data;
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
  const workspaceName = `S33 backtest result ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-backtest-result-${stamp}`;
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
  await editor.getByRole('textbox', { name: 'Name', exact: true }).fill(`S33 result strategy ${stamp}`);
  await editor.getByRole('button', { name: 'Save version', exact: true }).press('Enter');
  const selectedVersion = editor.getByText(/Selected .* · v1 · sha256:/);
  await selectedVersion.waitFor({ state: 'visible' });
  const strategyText = await selectedVersion.innerText();
  const strategyVersionId = /Selected ([0-9a-f-]{36})/i.exec(strategyText)?.[1];
  check(Boolean(strategyVersionId), `H5 has a saved immutable strategy version: ${strategyText}`);

  const backtest = page.getByRole('region', { name: 'Backtest', exact: true });
  const result = backtest.locator('.strategy-result');
  await backtest.getByLabel('Instrument', { exact: true }).fill('equity:US:AAPL');
  await backtest.getByLabel('Dataset', { exact: true }).fill('historical:fixture');
  await backtest.getByLabel('Start date', { exact: true }).fill('2026-01-01T00:00');
  await backtest.getByLabel('End date', { exact: true }).fill('2026-01-02T00:00');
  await backtest.getByLabel('Starting cash', { exact: true }).fill('125000');
  await backtest.getByLabel('Commission', { exact: true }).fill('0.25');
  await backtest.getByLabel('Slippage', { exact: true }).fill('5.5');
  await backtest.getByLabel('Portfolio seed', { exact: true }).fill('S33 result seed');
  await backtest.getByRole('combobox', { name: 'Backtest integration scenario', exact: true }).selectOption('SUCCESS');
  await backtest.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');

  await result.getByText('COMPLETED', { exact: true }).waitFor({ state: 'visible' });
  check(await result.getAttribute('aria-live') === 'polite', 'H5 completion updates use the polite live region.');
  const resultText = await result.innerText();
  const runId = /Run ([0-9a-f-]{36})/i.exec(resultText)?.[1];
  const requestHash = /Config sha256:([0-9a-f]{64})/i.exec(resultText)?.[1];
  check(Boolean(runId && requestHash), `H5 exposes run and request identities: ${resultText}`);
  check(resultText.includes('Historical simulation only. This result cannot place trades or call a provider.'),
    'H5 retains its historical-only/no-provider disclosure.');
  const completed = result.getByRole('region', { name: 'Completed result · historical simulation', exact: true });
  await completed.waitFor({ state: 'visible' });
  const metrics = await completed.locator('dl').first().evaluate(dl => Object.fromEntries(
    [...dl.querySelectorAll('div')].map(row => [
      row.querySelector('dt')?.textContent?.trim() ?? '', row.querySelector('dd')?.textContent?.trim() ?? '',
    ]),
  ));
  for (const [label, expected] of Object.entries({
    Return: '0.015', Sharpe: '1.5', Sortino: '2', 'Max drawdown': '0.005', 'Win rate': '1',
    'Profit factor': '2', Turnover: '0.01', Trades: '1',
  })) {
    check(metrics[label] === expected, `H5 metric ${label} should be ${expected}, got ${metrics[label]}.`);
  }

  const curve = completed.getByRole('table', { name: 'Backtest equity curve', exact: true });
  const curveRows = await curve.locator('tbody tr').allInnerTexts();
  check(curveRows.length === 2 && curveRows[0].includes('125000') && curveRows[1].includes('126875')
    && curveRows[1].includes('0.005'), `H5 renders the frozen equity curve: ${JSON.stringify(curveRows)}`);
  const trades = completed.getByRole('table', { name: 'Backtest trades', exact: true });
  const tradeRows = await trades.locator('tbody tr').allInnerTexts();
  check(tradeRows.length === 1 && tradeRows[0].includes('BUY') && tradeRows[0].includes('1')
    && tradeRows[0].includes('100') && tradeRows[0].includes('1875'),
  `H5 renders the completed trade list: ${JSON.stringify(tradeRows)}`);

  const manifestDetails = completed.locator('details');
  await manifestDetails.getByText('Reproducibility manifest', { exact: true }).click();
  const manifestText = await manifestDetails.innerText();
  for (const value of [
    'datasetHash', 'TradeX deterministic fixture', 'historical:fixture', '2026-01-01T00:00:00Z',
    '2026-01-02T00:00:00Z', 'split-dividend-adjusted', 'UTC', 'tradex-calendar-v1',
    'decimal-rate', '125000', '0.25', '5.5', 'S33 result seed', 'fixture-runtime-v1',
    'lookAhead: PASSED', 'survivorship: PASSED', 'split: PASSED', 'dividend: PASSED',
    'timezone: PASSED', 'dataGaps: PASSED', 'manifestHash',
    'No future bars are read before their observation time.',
    'The fixture universe is fixed at dataset creation.',
    'Split adjustment metadata is present and verified.',
    'Dividend adjustment metadata is present and verified.',
    'All observations are normalized to UTC.',
    'The fixture range has complete daily coverage.',
  ]) {
    check(manifestText.includes(value), `H5 reproducibility manifest is missing ${value}: ${manifestText}`);
  }
  check(resultText.includes('HISTORICAL_SIMULATION_ONLY')
    && resultText.includes('SYNTHETIC_FIXTURE_NOT_PROVIDER_DATA'),
  'H5 exposes the fixture limitations.');

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await completed.scrollIntoViewIfNeeded();
    check((await result.innerText()).includes(`Run ${runId}`), `H5 run identity changed at ${width}px.`);
    check(await completed.isVisible(), `H5 completed metrics are not visible at ${width}px.`);
    check(await manifestDetails.isVisible(), `H5 manifest is not visible at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
    }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `H5 overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    if (width === 390) {
      await result.screenshot({ path: 'docs/implementation/evidence/s33/backtest-result-390.png',
        style: '.topbar, .skip-link { display: none !important; }' });
    }
    observations.push({ width, completed: true, metricsVisible: true, curveRows: 2,
      tradeRows: 1, manifestVisible: true, noOverflow: true });
  }

  const commands = (await readLog(cursor)).commands.map(entry => entry.command);
  const expectedCommandCounts = {
    'account.activity': 1,
    'backtest.get': 1,
    'backtest.list': 2,
    'backtest.run': 1,
    'strategy.list': 3,
    'strategy.save_version': 1,
  };
  const actualCommandCounts = Object.fromEntries([...new Set(commands)].sort()
    .map(name => [name, commands.filter(commandName => commandName === name).length]));
  check(JSON.stringify(actualCommandCounts) === JSON.stringify(expectedCommandCounts),
    `H5 used only the expected fixture-backed workspace, strategy, and backtest commands: ${JSON.stringify(actualCommandCounts)}.`);
  check(commands.filter(name => name === 'backtest.run').length === 1, 'H5 issues exactly one synthetic backtest run.');
  check(commands.filter(name => name === 'strategy.save_version').length === 1,
    'H5 uses one local immutable strategy version.');
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return { result: 'PASS', screenId: 'H5', workspace: workspace.workspaceId,
    strategyVersionId, runId, requestHash, metrics, equityCurveRows: curveRows.length, tradeRows: tradeRows.length,
    widths: [1280, 768, 390], observations, backtestRuns: 1, backtestCancels: 0,
    commandCounts: actualCommandCounts, unexpectedCommands: 0, externalHttpRequests: externalRequests.length };
}
