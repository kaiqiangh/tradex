// Run against a Rust-backed local integration Vite service. Catalog and screener state are workspace-local fixtures.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const pageErrors = [];
  const consoleErrors = [];
  page.setDefaultTimeout(10000);
  page.on('request', request => {
    const url = request.url();
    const localOrigin = /^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(url);
    if (/^https?:\/\//i.test(url) && !localOrigin) externalRequests.push(url.split('/').slice(0, 3).join('/'));
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  const command = async (name, payload) => {
    const requestId = `s33-c1-${name}-${Date.now()}-${Math.random()}`;
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
  const commandLog = async cursor => page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), cursor);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1426/');
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const cursor = await commandLog(0);
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 C1 Market Explorer ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-c1-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Markets', exact: true }).click();
  await page.getByRole('heading', { name: 'Market Explorer', exact: true }).waitFor({ state: 'visible' });

  const marketSections = page.getByRole('group', { name: 'Market sections', exact: true });
  const stocksTab = marketSections.getByRole('button', { name: 'US Stocks', exact: true });
  const cryptoTab = marketSections.getByRole('button', { name: 'Crypto', exact: true });
  const screenersTab = marketSections.getByRole('button', { name: 'Screeners', exact: true });
  check(await stocksTab.getAttribute('aria-pressed') === 'true', 'US Stocks is the default selected market section.');
  await page.getByRole('heading', { name: 'Market metrics', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'Top movers', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'Saved screeners', exact: true }).waitFor({ state: 'visible' });
  check(await page.getByText('Catalog counts describe available instrument identities. Price, index and performance metrics require a verified market-data source.', { exact: true }).isVisible(),
    'Market overview distinguishes source-backed catalog counts from unavailable price/index metrics.');
  check(await page.getByText('No prices or percentage changes are synthesized when mover data is unavailable.', { exact: true }).isVisible(),
    'The movers empty state does not invent market values.');
  const equityResults = page.getByRole('list', { name: 'Market results', exact: true });
  await equityResults.getByText('equity:US:AAPL', { exact: true }).waitFor({ state: 'visible' });
  check(await equityResults.getByText('equity:US:AAPL', { exact: true }).isVisible(), 'The stock view shows the canonical AAPL identity.');
  check(await equityResults.getByText('crypto:BTC/USDT:spot', { exact: true }).count() === 0, 'The stock view excludes crypto instruments.');

  await cryptoTab.click();
  check(await cryptoTab.getAttribute('aria-pressed') === 'true', 'Crypto can be selected as a distinct market view.');
  const cryptoResults = page.getByRole('list', { name: 'Market results', exact: true });
  check(await cryptoResults.getByText('crypto:BTC/USDT:spot', { exact: true }).isVisible(), 'The crypto view shows the canonical BTC/USDT identity.');
  check(await cryptoResults.getByText('equity:US:AAPL', { exact: true }).count() === 0, 'The crypto view excludes equities.');
  await page.getByLabel('Search instruments', { exact: true }).fill('BTC/USDT');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await cryptoResults.getByText('crypto:BTC/USDT:spot', { exact: true }).waitFor({ state: 'visible' });
  check(await cryptoResults.getByText('crypto:BTC/USDT:spot', { exact: true }).isVisible(), 'Search preserves the exact canonical crypto result.');
  await cryptoResults.getByRole('button', { name: /BTC\/USDT/ }).click();
  await page.getByRole('button', { name: 'Back to results', exact: true }).waitFor({ state: 'visible' });
  check(await page.getByText('crypto:BTC/USDT:spot', { exact: true }).count() >= 1, 'Opening crypto detail keeps its canonical identity.');
  await page.getByRole('button', { name: 'Back to results', exact: true }).click();
  await page.getByLabel('Search instruments', { exact: true }).fill('');
  await page.getByRole('button', { name: 'Search', exact: true }).click();

  await screenersTab.click();
  check(await screenersTab.getAttribute('aria-pressed') === 'true', 'Screeners opens from the C1 market section control.');
  await page.getByRole('heading', { name: 'Natural-language filter', exact: true }).waitFor({ state: 'visible' });
  const naturalLanguage = 'US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.';
  await page.getByLabel('Describe the market', { exact: true }).fill(naturalLanguage);
  await page.getByRole('button', { name: 'Parse conditions', exact: true }).click();
  await page.getByRole('heading', { name: 'FilterSpec and RankSpec', exact: true }).waitFor({ state: 'visible' });
  await page.getByLabel('Save name', { exact: true }).fill('C1 local saved screen');
  await page.getByRole('button', { name: 'Save screener', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Saved screener C1 local saved screen.' }).waitFor({ state: 'visible' });
  await stocksTab.click();
  const savedScreen = page.getByRole('button', { name: 'Open saved screener C1 local saved screen', exact: true });
  await savedScreen.waitFor({ state: 'visible' });
  await savedScreen.click();
  await page.getByRole('heading', { name: 'Natural-language filter', exact: true }).waitFor({ state: 'visible' });
  check(await page.getByLabel('Describe the market', { exact: true }).inputValue() === naturalLanguage,
    'A saved screener reopens its stored definition in the builder.');
  await stocksTab.click();

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    check(geometry.scroll <= geometry.viewport, `C1 Market Explorer overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(await marketSections.isVisible(), `Market sections remain visible at ${width}px.`);
    check(await page.getByRole('heading', { name: 'Saved screeners', exact: true }).isVisible(), `Saved screeners remain visible at ${width}px.`);
    responsive.push({ width, documentScrollWidth: geometry.scroll });
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator('.market-overview').screenshot({ path: 'docs/implementation/evidence/s33/c1-market-explorer-390.png' });

  const finalLog = await commandLog(cursor.cursor);
  const forbiddenCommands = finalLog.commands.filter(entry =>
    /^(provider\.|account\.(?!list$|get$|activity$)|approval\.|risk\.(?!get_policy$|decision\.list$)|model\.|data\.(?!source\.catalog$|.*\.connection$)|trade\.|paper\.order\.|order\.|gateway\.|live\.)/.test(entry.command));
  check(forbiddenCommands.length === 0, `C1 issued a forbidden provider/financial command: ${JSON.stringify(forbiddenCommands)}`);
  check(externalRequests.length === 0, `C1 contacted external hosts: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0 && consoleErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ pageErrors, consoleErrors })}`);
  return {
    result: 'PASS_LIMITED',
    screenId: 'C1',
    selectedSections: ['US Stocks', 'Crypto', 'Screeners'],
    canonicalIdentity: { equity: 'equity:US:AAPL', crypto: 'crypto:BTC/USDT:spot' },
    sourceGatedMarketMetrics: true,
    moversRemainUnavailableWithoutAggregateFeed: true,
    savedScreenerCreateAndReopen: true,
    commands: finalLog.commands.map(entry => entry.command),
    forbiddenCommands: forbiddenCommands.length,
    externalRequests: externalRequests.length,
    pageErrors: pageErrors.length,
    consoleErrors: consoleErrors.length,
    responsive,
    fixtureOnly: true,
  };
}
