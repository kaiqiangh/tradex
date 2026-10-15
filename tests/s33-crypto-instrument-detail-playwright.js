// Run against the Rust-backed local integration Vite service with synthetic fixtures only.
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
    const requestId = `s33-c5-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 C5 Crypto Detail ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-c5-${stamp}`;
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
  await page.getByRole('checkbox', { name: 'Crypto', exact: true }).check();
  const result = page.getByRole('list', { name: 'Market results', exact: true });
  await result.getByRole('button', { name: /BTC\/USDT Bitcoin \/ Tether spot/ }).click();

  await page.getByRole('heading', { name: 'Bitcoin / Tether spot', exact: true }).waitFor({ state: 'visible' });
  check(await page.getByText('crypto:BTC/USDT:spot', { exact: true }).count() >= 1, 'The canonical spot instrument identity remains visible.');
  check(await page.getByRole('region', { name: 'Spot-only warning', exact: true }).getByRole('heading', { name: 'Spot only', exact: true }).isVisible(),
    'The selected instrument is explicitly identified as spot only.');
  check(await page.getByText('Spot only in v1.0. Futures, leverage, margin, and transfer actions are unavailable.', { exact: true }).isVisible(),
    'Unsupported futures, leverage, margin, and transfer actions are explicitly unavailable.');
  check(await page.getByRole('region', { name: 'Spot venue state', exact: true }).getByText('Venue state: Maintenance.', { exact: true }).isVisible(),
    'The synthetic Maintenance venue state is visible and distinct from equity calendars.');
  check(await page.getByText('No source selected', { exact: true }).count() >= 1, 'An unselected market source stays explicit.');
  check(await page.getByText('Not verified', { exact: true }).count() >= 1, 'Quote entitlement remains unverified.');
  check(await page.getByText('Unavailable — no validated quote', { exact: true }).count() >= 1, 'No bid, ask, spread or freshness is invented.');
  check(await page.getByText('Unavailable — no validated venue depth evidence', { exact: true }).isVisible(), 'Depth remains unavailable without venue evidence.');
  check(await page.getByRole('heading', { name: 'Price chart unavailable', exact: true }).isVisible(), 'A quote is not treated as historical chart data.');
  check(await page.getByRole('heading', { name: 'Trade action unavailable', exact: true }).isVisible(), 'The screen explains why no trade action is available.');
  check(await page.locator('.market-detail').getByRole('button', { name: /trade/i }).count() === 0,
    'No Trade action is offered without verified account and venue compatibility.');

  const quoteText = await page.getByRole('region', { name: 'Crypto venue quote', exact: true }).innerText();
  check(/Provider time[\s\S]*Unavailable/.test(quoteText) && /TradeX received[\s\S]*Unavailable/.test(quoteText),
    'Provider and receipt timestamps stay unavailable without a snapshot.');
  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => {
      const viewport = document.documentElement.clientWidth;
      const controls = [...document.querySelectorAll('.market-detail button, .market-detail input, .market-detail select')]
        .filter(element => element.getBoundingClientRect().width > 0)
        .map(element => { const box = element.getBoundingClientRect(); return { left: box.left, right: box.right }; });
      return { viewport, documentWidth: document.documentElement.scrollWidth,
        controlsReachable: controls.every(box => box.left >= 0 && box.right <= viewport) };
    });
    check(geometry.documentWidth <= geometry.viewport, `C5 document overflows at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.controlsReachable, `C5 controls are not horizontally reachable at ${width}px: ${JSON.stringify(geometry)}`);
    responsive.push({ width, documentOverflow: false, detailControlsReachable: true });
  }

  const finalLog = await commandLog(cursor.cursor);
  const forbiddenCommands = finalLog.commands.filter(entry =>
    /^(provider\.|account\.(?!list$|get$|activity$)|approval\.|risk\.(?!get_policy$|decision\.list$)|model\.|data\.(?!source\.catalog$|.*\.connection$)|trade\.|paper\.order\.|order\.|gateway\.|live\.)/.test(entry.command));
  check(forbiddenCommands.length === 0, `C5 issued a forbidden provider/financial command: ${JSON.stringify(forbiddenCommands)}`);
  check(externalRequests.length === 0, `C5 contacted external hosts: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0 && consoleErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ pageErrors, consoleErrors })}`);
  return {
    result: 'PASS_LIMITED', screenId: 'C5', workspaceId: workspace.workspaceId, fixtureOnly: true,
    canonicalInstrument: 'crypto:BTC/USDT:spot', venueState: 'MAINTENANCE', spotOnlyWarning: true,
    quoteSpreadDepthAndHistoryUnavailableWithoutValidatedEvidence: true, noIncompatibleTradeAction: true,
    responsive, forbiddenCommands: forbiddenCommands.length, externalRequests: externalRequests.length,
    pageErrors: pageErrors.length, consoleErrors: consoleErrors.length,
    limitations: ['No real market-data source, entitlement, credentials, depth evidence, or account execution compatibility was activated.',
      'Native VoiceOver, the source-backed quote/depth state, and the full S07/S33 gates remain open.'],
  };
}
