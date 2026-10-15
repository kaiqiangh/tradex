// Start the isolated Rust-backed integration server with:
// ./node_modules/.bin/vite --host 127.0.0.1 --mode integration --port 1422 --strictPort
// Then run this callback with mcp__playwright__browser_run_code_unsafe, filename:
// tests/s33-backtest-compare-screen-playwright.js
// Covers H6 using two local synthetic runs with different saved strategy parameters.
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
    const requestId = `s33-backtest-compare-${name}-${Date.now()}-${Math.random()}`;
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
  const stamp = await page.evaluate(() => crypto.randomUUID());
  const workspaceName = `S33 backtest compare ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-backtest-compare-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const cursor = (await readLog(0)).cursor;

  const fastName = `S33 H6 Fast ${stamp}`;
  const slowName = `S33 H6 Slow ${stamp}`;
  const fastVersion = await command('strategy.save_version', { workspaceId: workspace.workspaceId,
    definition: { name: fastName, source: 'signal = HOLD\nreturn signal', language: 'python', runtime: 'sandbox-v1',
      parameters: [{ name: 'window', value: '10' }] } });
  const slowVersion = await command('strategy.save_version', { workspaceId: workspace.workspaceId,
    definition: { name: slowName, source: 'signal = HOLD\nreturn signal', language: 'python', runtime: 'sandbox-v1',
      parameters: [{ name: 'window', value: '20' }] } });
  check(fastVersion.strategyVersionId !== slowVersion.strategyVersionId,
    'H6 compares distinct immutable strategy versions.');

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Strategies', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
  const editor = page.getByRole('region', { name: 'Strategy editor', exact: true });
  const backtest = page.getByRole('region', { name: 'Backtest', exact: true });
  const result = backtest.locator('.strategy-result');
  const runIds = [];

  for (const [versionName, version, startingCash, seed] of [
    [fastName, fastVersion, '125000', 'H6 fast seed'],
    [slowName, slowVersion, '150000', 'H6 slow seed'],
  ]) {
    await page.getByRole('button').filter({ hasText: versionName }).first().click();
    await editor.getByText(new RegExp(`Selected ${version.strategyVersionId} · v1 · sha256:`)).waitFor({ state: 'visible' });
    await backtest.getByLabel('Instrument', { exact: true }).fill('equity:US:AAPL');
    await backtest.getByLabel('Dataset', { exact: true }).fill('historical:fixture');
    await backtest.getByLabel('Start date', { exact: true }).fill('2026-01-01T00:00');
    await backtest.getByLabel('End date', { exact: true }).fill('2026-01-02T00:00');
    await backtest.getByLabel('Starting cash', { exact: true }).fill(startingCash);
    await backtest.getByLabel('Commission', { exact: true }).fill('0.25');
    await backtest.getByLabel('Slippage', { exact: true }).fill('5.5');
    await backtest.getByLabel('Portfolio seed', { exact: true }).fill(seed);
    await backtest.getByRole('combobox', { name: 'Backtest integration scenario', exact: true }).selectOption('SUCCESS');
    const previousRunId = runIds.at(-1) ?? null;
    await backtest.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
    await page.waitForFunction(previous => {
      const text = document.querySelector('[aria-labelledby^="backtest-title-"] .strategy-result')?.innerText ?? '';
      const id = /Run ([0-9a-f-]{36})/i.exec(text)?.[1];
      return text.includes('COMPLETED') && Boolean(id) && id !== previous;
    }, previousRunId);
    const resultText = await result.innerText();
    const runId = /Run ([0-9a-f-]{36})/i.exec(resultText)?.[1];
    check(Boolean(runId), `H6 local fixture run is completed with an identity: ${resultText}`);
    check(!runIds.includes(runId), 'H6 records a new completed run ID for each parameter set.');
    runIds.push(runId);
  }
  check(runIds.length === 2 && runIds[0] !== runIds[1], 'H6 prepares exactly two distinct completed runs.');

  const leftSelector = page.getByLabel('Compare left run', { exact: true });
  const rightSelector = page.getByLabel('Compare right run', { exact: true });
  await leftSelector.waitFor({ state: 'visible' });
  await rightSelector.waitFor({ state: 'visible' });
  await leftSelector.selectOption(runIds[0]);
  await rightSelector.selectOption(runIds[0]);
  const compareButton = page.getByRole('button', { name: 'Compare runs', exact: true });
  check(await compareButton.isDisabled(), 'H6 prevents selecting one run twice.');
  await rightSelector.selectOption(runIds[1]);
  check(await compareButton.isEnabled(), 'H6 enables comparison after selecting two distinct completed runs.');
  await compareButton.press('Enter');

  const comparison = page.locator('.strategy-compare .strategy-result');
  await comparison.getByRole('heading', { name: 'Backtest comparison', exact: true }).waitFor({ state: 'visible' });
  check(await comparison.getAttribute('aria-live') === 'polite', 'H6 result changes use the polite live region.');
  const comparisonText = await comparison.innerText();
  check(comparisonText.includes('Historical simulation only. Comparison is read-only and cannot place trades or call a provider.'),
    'H6 clearly keeps comparison historical and read-only.');
  const metricTable = comparison.getByRole('table', { name: 'Backtest metric comparison', exact: true });
  const metricRows = await metricTable.locator('tbody tr').evaluateAll(rows => rows.map(row =>
    [...row.querySelectorAll('th, td')].map(cell => cell.textContent?.trim() ?? '')));
  const metricNames = ['maxDrawdown', 'profitFactor', 'return', 'sharpe', 'sortino', 'tradeCount', 'turnover', 'winRate'];
  check(metricRows.length === metricNames.length && metricRows.every((row, index) =>
    row.length === 4 && row[0] === metricNames[index] && row[1] !== '' && row[2] !== '' && Number.isFinite(Number(row[3]))),
  `H6 shows eight exact metric deltas: ${JSON.stringify(metricRows)}`);

  const inputRegion = comparison.getByRole('region', { name: 'Input differences', exact: true });
  check(await inputRegion.isVisible(), 'H6 exposes input differences as a named region.');
  const inputTable = inputRegion.getByRole('table', { name: 'Input differences', exact: true });
  const inputRows = await inputTable.locator('tbody tr').evaluateAll(rows => rows.map(row =>
    [...row.querySelectorAll('th, td')].map(cell => cell.textContent?.trim() ?? '')));
  const parameterRow = inputRows.find(row => row[0] === 'parameters');
  check(Boolean(parameterRow && parameterRow[1].includes('"value":"10"') && parameterRow[2].includes('"value":"20"')),
    `H6 exposes the saved parameter difference: ${JSON.stringify(parameterRow)}`);
  for (const [field, left, right] of [
    ['startingCash', '125000', '150000'],
    ['portfolioSeed', 'H6 fast seed', 'H6 slow seed'],
  ]) {
    check(inputRows.some(row => row[0] === field && row[1] === left && row[2] === right),
      `H6 input differences preserve ${field}: ${JSON.stringify(inputRows)}`);
  }

  const identity = comparison.getByRole('region', { name: 'Run identity', exact: true });
  const identityText = await identity.innerText();
  check(identityText.includes(runIds[0]) && identityText.includes(runIds[1])
    && identityText.includes(fastVersion.strategyVersionId) && identityText.includes(slowVersion.strategyVersionId)
    && identityText.includes(fastVersion.sourceHash) && identityText.includes(slowVersion.sourceHash)
    && identityText.includes('Data manifest') && identityText.includes('Result hash'),
  `H6 binds both compared runs to their immutable strategy and result identities: ${identityText}`);
  const manifestRegion = comparison.getByRole('region', { name: 'Manifest differences', exact: true });
  check(await manifestRegion.isVisible(), 'H6 exposes manifest differences as a named region.');
  const manifestTable = manifestRegion.getByRole('table', { name: 'Manifest differences', exact: true });
  const manifestRows = await manifestTable.locator('tbody tr').evaluateAll(rows => rows.map(row =>
    [...row.querySelectorAll('th, td')].map(cell => cell.textContent?.trim() ?? '')));
  check(manifestRows.some(row => row[0] === 'strategyVersion')
    && manifestRows.some(row => row[0] === 'parameters')
    && manifestRows.some(row => row[0] === 'manifestHash'),
  `H6 exposes immutable version and manifest differences: ${JSON.stringify(manifestRows)}`);
  for (const [name, regionName] of [
    ['Open left strategy version', 'Left strategy version'],
    ['Open right strategy version', 'Right strategy version'],
  ]) {
    const link = comparison.getByRole('link', { name, exact: true });
    const href = await link.getAttribute('href');
    const target = href ? page.locator(href) : page.locator('html');
    const region = comparison.getByRole('region', { name: regionName, exact: true });
    check(Boolean(href?.startsWith('#')) && await target.count() === 1 && await region.isVisible()
      && href?.slice(1) === await region.getAttribute('id'),
      `H6 provides a working in-page link to ${regionName}.`);
    await link.press('Enter');
    check(new URL(page.url()).hash === href, `H6 link navigates to ${regionName}.`);
  }
  for (const [name, label] of [
    ['Open left manifest', 'Left reproducibility manifest'],
    ['Open right manifest', 'Right reproducibility manifest'],
  ]) {
    const link = comparison.getByRole('link', { name, exact: true });
    const href = await link.getAttribute('href');
    const target = href ? page.locator(href) : page.locator('html');
    const summary = target.getByText(label, { exact: true });
    check(Boolean(href?.startsWith('#')) && await target.count() === 1 && await summary.isVisible()
      && href?.slice(1) === await target.getAttribute('id')
      && await target.evaluate(element => element instanceof HTMLDetailsElement && !element.open),
    `H6 provides a link to the collapsed ${label}.`);
    await link.press('Enter');
    check(new URL(page.url()).hash === href, `H6 link navigates to ${label}.`);
    await summary.press('Enter');
    check(await target.evaluate(element => element instanceof HTMLDetailsElement && element.open)
      && (await target.innerText()).includes('manifestHash'),
    `H6 expands the complete ${label}.`);
    await summary.press('Enter');
  }
  check(comparisonText.includes('HISTORICAL_SIMULATION_ONLY')
    && comparisonText.includes('COMPARISON_READ_ONLY')
    && comparisonText.includes('SYNTHETIC_FIXTURE_NOT_PROVIDER_DATA'),
  'H6 exposes the comparison limitations.');

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await comparison.scrollIntoViewIfNeeded();
    check(await comparison.isVisible(), `H6 comparison is not visible at ${width}px.`);
    check((await comparison.innerText()).includes(runIds[0]) && (await comparison.innerText()).includes(runIds[1]),
      `H6 identities changed at ${width}px.`);
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `H6 overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    const scrollRegions = await comparison.locator('.table-scroll').count();
    check(scrollRegions >= 3, `H6 keeps wide comparison tables in internal scroll regions at ${width}px.`);
    if (width === 390) {
      const inputScroll = inputTable.locator('xpath=..');
      const initialScroll = await inputScroll.evaluate(element => ({ scrollWidth: element.scrollWidth,
        clientWidth: element.clientWidth, scrollLeft: element.scrollLeft }));
      check(initialScroll.scrollWidth > initialScroll.clientWidth,
        `H6 wide difference table should scroll inside its region at 390px: ${JSON.stringify(initialScroll)}.`);
      await inputScroll.evaluate(element => { element.scrollLeft = element.scrollWidth; });
      const movedScroll = await inputScroll.evaluate(element => ({ scrollWidth: element.scrollWidth,
        clientWidth: element.clientWidth, scrollLeft: element.scrollLeft }));
      check(movedScroll.scrollLeft > 0 && movedScroll.scrollLeft + movedScroll.clientWidth >= movedScroll.scrollWidth - 1,
        `H6 difference table cannot reach its trailing columns at 390px: ${JSON.stringify(movedScroll)}.`);
      await inputScroll.evaluate(element => { element.scrollLeft = 0; });
      await comparison.screenshot({ path: 'docs/implementation/evidence/s33/backtest-compare-390.png',
        style: '.topbar, .skip-link { display: none !important; }' });
    }
    observations.push({ width, comparisonVisible: true, noPageOverflow: true, tableScrollRegions: scrollRegions });
  }

  await page.getByRole('button', { name: 'Back to backtest', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Backtest', exact: true }).waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Compare runs',
    'H6 returns focus to the compare trigger after leaving results.');

  const commands = (await readLog(cursor)).commands.map(entry => entry.command);
  const allowed = new Set(['account.activity', 'strategy.list', 'backtest.list', 'strategy.save_version',
    'backtest.run', 'backtest.get', 'backtest.compare']);
  const unexpected = commands.filter(name => !allowed.has(name));
  check(unexpected.length === 0, `H6 issued an unexpected, provider, or financial command: ${JSON.stringify(unexpected)}.`);
  check(commands.filter(name => name === 'strategy.save_version').length === 2,
    'H6 creates only two immutable local strategy versions for the comparison.');
  check(commands.filter(name => name === 'backtest.run').length === 2,
    'H6 creates exactly two synthetic backtest runs.');
  check(commands.filter(name => name === 'backtest.compare').length === 1,
    'H6 makes exactly one read-only comparison request.');
  check(commands.filter(name => name === 'backtest.cancel').length === 0,
    'H6 never cancels or mutates either completed run.');
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return { result: 'PASS', screenId: 'H6', workspace: workspace.workspaceId, runIds,
    strategyVersionIds: [fastVersion.strategyVersionId, slowVersion.strategyVersionId],
    parameterValues: ['10', '20'], metrics: metricRows.length, parameterDifferences: parameterRow,
    manifestDifferences: manifestRows.length, widths: [1280, 768, 390], observations,
    commands: commands.length, unexpectedCommands: unexpected.length, externalHttpRequests: externalRequests.length };
}
