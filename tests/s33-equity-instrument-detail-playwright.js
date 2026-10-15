// Run against a Rust-backed local integration Vite service with synthetic fixtures only.
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
    const requestId = `s33-c4-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 C4 Equity Detail ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-c4-${stamp}`;
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
  await page.getByLabel('Search instruments', { exact: true }).fill('MSFT');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  const result = page.getByRole('list', { name: 'Market results', exact: true });
  await result.getByRole('button', { name: /MSFT Microsoft Corporation/ }).click();

  const tabs = ['Overview', 'Financials', 'News', 'Filings', 'Analysis'];
  const tablist = page.getByRole('tablist', { name: 'Equity instrument detail sections', exact: true });
  for (const name of tabs) check(await tablist.getByRole('tab', { name, exact: true }).isVisible(), `${name} detail tab is visible.`);
  check(await page.getByRole('tab', { name: 'Overview', exact: true }).getAttribute('aria-selected') === 'true',
    'Overview is the default equity detail view.');
  check(await page.getByText('equity:US:MSFT', { exact: true }).count() >= 1, 'The exact canonical instrument identity remains visible.');
  check(await page.getByText('Market-data entitlement is not configured or verified in TradeX.', { exact: true }).isVisible(),
    'Unverified market access is explicitly blocked.');
  check(await page.getByText('Unavailable — no validated quote', { exact: true }).isVisible(),
    'Market freshness remains unavailable without a validated quote.');
  check(await page.getByRole('alert', { name: 'MARKET_CLOSED error recovery', exact: true }).isVisible(),
    'The synthetic closed session displays the deterministic MARKET_CLOSED block.');
  check(await page.getByText('4 recorded', { exact: true }).isVisible(), 'The bounded corporate-action fixture is visible.');
  check(await page.getByText('No validated historical series is available for this instrument. A quote snapshot is not treated as chart history.', { exact: true }).isVisible(),
    'The screen does not infer chart history from a quote snapshot.');

  await page.getByRole('tab', { name: 'Overview', exact: true }).press('ArrowRight');
  check(await page.getByRole('tab', { name: 'Financials', exact: true }).getAttribute('aria-selected') === 'true',
    'ArrowRight moves the selected tab and the focused tab to Financials.');
  check(await page.getByRole('heading', { name: 'Financial data unavailable', exact: true }).isVisible(),
    'Missing company financial inputs stay unavailable.');
  for (const [name, emptyHeading] of [['News', 'News unavailable'], ['Filings', 'Filings unavailable'], ['Analysis', 'Analysis unavailable']]) {
    await page.getByRole('tab', { name, exact: true }).click();
    check(await page.getByRole('heading', { name: emptyHeading, exact: true }).isVisible(), `${name} has an honest unavailable state.`);
  }

  await page.getByRole('tab', { name: 'Overview', exact: true }).click();
  await page.getByLabel('Create a watchlist', { exact: true }).fill('C4 core equities');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByText('Created “C4 core equities”.', { exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Add to watchlist', exact: true }).click();
  await page.getByText('MSFT added to “C4 core equities”.', { exact: true }).waitFor({ state: 'visible' });
  check(await page.getByRole('button', { name: 'Already in watchlist', exact: true }).isDisabled(),
    'The selected list reflects persisted exact-instrument membership and prevents duplicate add.');

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => {
      const viewport = document.documentElement.clientWidth;
      const tabs = document.querySelector('.market-detail-tabs');
      const controls = [...document.querySelectorAll('.market-detail button, .market-detail input, .market-detail select')]
        .filter(element => element.getBoundingClientRect().width > 0)
        .map(element => { const box = element.getBoundingClientRect(); return { left: box.left, right: box.right }; });
      return { viewport, documentWidth: document.documentElement.scrollWidth, tablistWidth: tabs?.clientWidth,
        tablistScrollWidth: tabs?.scrollWidth, controlsReachable: controls.every(box => box.left >= 0 && box.right <= viewport) };
    });
    check(geometry.documentWidth <= geometry.viewport, `C4 document overflows at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.tablistScrollWidth <= geometry.tablistWidth, `C4 tabs overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.controlsReachable, `C4 controls are not horizontally reachable at ${width}px: ${JSON.stringify(geometry)}`);
    responsive.push({ width, documentOverflow: false, tabOverflow: false, controlsReachable: true });
  }

  const finalLog = await commandLog(cursor.cursor);
  const forbiddenCommands = finalLog.commands.filter(entry =>
    /^(provider\.|account\.(?!list$|get$|activity$)|approval\.|risk\.(?!get_policy$|decision\.list$)|model\.|data\.(?!source\.catalog$|.*\.connection$)|trade\.|paper\.order\.|order\.|gateway\.|live\.)/.test(entry.command));
  check(forbiddenCommands.length === 0, `C4 issued a forbidden provider/financial command: ${JSON.stringify(forbiddenCommands)}`);
  check(externalRequests.length === 0, `C4 contacted external hosts: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0 && consoleErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ pageErrors, consoleErrors })}`);
  return {
    result: 'PASS_LIMITED', screenId: 'C4', workspaceId: workspace.workspaceId, fixtureOnly: true,
    tabs, keyboardTabNavigation: true, canonicalIdentity: 'equity:US:MSFT',
    marketClosedAndCorporateActions: true, sourceFreshnessUnavailableWithoutEvidence: true,
    quoteAndChartNotFabricated: true, newsFinancialsFilingsAnalysisEmptyStates: true,
    watchlistCreateAndCanonicalAdd: true, responsive, commands: finalLog.commands.map(entry => entry.command),
    forbiddenCommands: forbiddenCommands.length, externalRequests: externalRequests.length,
    pageErrors: pageErrors.length, consoleErrors: consoleErrors.length,
    limitations: ['No real provider/source entitlement was configured.', 'INSTRUMENT_HALTED is covered by the Rust market-eligibility contract test; the browser fixture observed the MARKET_CLOSED case.'],
  };
}
