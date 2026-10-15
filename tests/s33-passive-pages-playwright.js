// Run with `playwright-cli run-code --filename=tests/s33-passive-pages-playwright.js`
// against `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser`.
// A1-A4 observations are passive; A5/A6 use only integration-test model fixtures and local risk persistence.
// External HTTP is observed and rejected; no real provider/model operation is activated.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const pageErrors = [];
  const consoleErrors = [];
  const staticIconErrors = [];
  page.on('request', request => {
    const origin = request.url().match(/^https?:\/\/[^/]+/i)?.[0];
    if (origin && !/^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?$/i.test(origin)) externalRequests.push(origin);
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  page.on('console', message => {
    if (message.type() === 'error') {
      if (message.text().includes('/favicon.ico')) staticIconErrors.push(message.text());
      else consoleErrors.push(message.text());
    }
  });
  const fixtureCommands = [];
  const command = async (name, payload) => {
    const requestId = `s33-passive-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok, `${name}: ${JSON.stringify(result.body?.error)}`);
    if (name.endsWith('.fixture')) fixtureCommands.push(name);
    return result.body.data;
  };
  const log = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);
  const forbidden = new Set([
    'provider.connect', 'provider.probe', 'provider.permissions', 'provider.disconnect', 'account.refresh',
    'model.gateway', 'model.login_chatgpt', 'model.configure_deepseek', 'model.verify_route',
    'model.set_default', 'model.set_fallback_policy', 'data.source.probe', 'data.source.disconnect',
    'alpaca.paper.orders.refresh', 'trading212.demo.orders.refresh', 'binance.testnet.orders.refresh',
    'trade.live_order.refresh', 'trade.resolution_evidence.refresh', 'trade.spot_capacity.refresh',
    'trade.spot_commission.refresh', 'trade.spot_order_intervals.refresh', 'trade.spot_rules.refresh',
    'data.fx.refresh', 'data.fx.disconnect', 'data.calendar.refresh', 'data.calendar.disconnect',
    'data.instrument.refresh', 'data.instrument.disconnect', 'data.actions.refresh', 'data.actions.disconnect',
    'data.binance_rules.refresh', 'data.binance_rules.disconnect', 'data.binance_market.disconnect',
    'paper.quote.refresh', 'market.hot.acquire', 'market.hot.release', 'market.screen',
  ]);
  const assertNoProviderIo = async (label, cursor) => {
    const next = await log(cursor);
    const hits = next.commands.filter(entry => forbidden.has(entry.command));
    check(hits.length === 0, `${label} issued a provider/model/data request: ${JSON.stringify(hits)}`);
    return { cursor: next.cursor, commands: next.commands.length };
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 passive pages ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-passive-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });

  const onboardingObservations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const fields = [
      page.getByRole('textbox', { name: 'Workspace name', exact: true }),
      page.getByRole('combobox', { name: 'Base currency', exact: true }),
      page.getByLabel('Local storage', { exact: true }),
    ];
    const boxes = [];
    for (const field of fields) {
      check(await field.isVisible(), `A1 workspace field is visible at ${width}px.`);
      const box = await field.boundingBox();
      check(Boolean(box), `A1 workspace field has a visible box at ${width}px.`);
      boxes.push(box);
    }
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
    }));
    check(geometry.documentScrollWidth <= geometry.viewport,
      `A1 workspace form has no document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    for (const box of boxes) {
      check(box.x >= 0 && box.x + box.width <= width,
        `A1 workspace field stays in the viewport at ${width}px: ${JSON.stringify(box)}`);
    }
    if (width === 390) {
      await page.screenshot({ path: 'docs/implementation/evidence/s33/workspace-onboarding-390.png', fullPage: true });
    }
    onboardingObservations.push({ width, fieldsVisible: 3, fieldsInViewport: true, documentOverflow: false });
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });

  await page.getByRole('button', { name: 'Continue to Providers', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Providers', exact: true }).waitFor({ state: 'visible' });
  await page.waitForFunction(() => {
    const heading = document.getElementById('onboarding-title');
    const fields = [...document.querySelectorAll('.provider-config label.field input, .provider-config label.field select')];
    const stable = heading?.textContent?.trim() === 'Providers' && fields.length === 3 && fields.every(field => {
      const rect = field.getBoundingClientRect();
      const style = getComputedStyle(field);
      return rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden' && style.display !== 'none';
    });
    if (!stable) { delete window.__s33A2StableSince; return false; }
    window.__s33A2StableSince ??= performance.now();
    return performance.now() - window.__s33A2StableSince >= 400;
  }, undefined, { timeout: 10000, polling: 50 });
  const providerOnboardingObservations = [];
  let onboardingCursor = (await log(0)).cursor;
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const fields = [
      ['Connection source', page.getByRole('combobox', { name: 'Connection source', exact: true })],
      ['Provider / environment', page.getByRole('combobox', { name: 'Provider / environment', exact: true })],
      ['Connection label', page.getByRole('textbox', { name: 'Connection label', exact: true })],
    ];
    for (const [name, field] of fields) {
      await field.waitFor({ state: 'visible' });
      const box = await field.boundingBox();
      check(Boolean(box) && box.x >= 0 && box.x + box.width <= width,
        `A2 ${name} field stays in the viewport at ${width}px: ${JSON.stringify(box)}`);
    }
    const connect = page.getByRole('button', { name: 'Connect account securely', exact: true });
    check(await connect.isVisible() && await connect.isEnabled(),
      `A2 secure-connect action is visible and available, but remains unactivated, at ${width}px.`);
    const notice = page.getByText(/Connect read-only broker or data providers\. Live accounts remain DISARMED\./);
    check(await notice.isVisible(), `A2 read-only/Live disclosure is visible at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
    }));
    check(geometry.documentScrollWidth <= geometry.viewport,
      `A2 Providers step has no document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await page.screenshot({ path: 'docs/implementation/evidence/s33/provider-onboarding-390.png' });
    }
    const passive = await assertNoProviderIo(`A2 Providers onboarding at ${width}px`, onboardingCursor);
    onboardingCursor = passive.cursor;
    providerOnboardingObservations.push({ width, fieldsVisible: 3, connectActionVisible: true, disclosureVisible: true, documentOverflow: false });
  }

  await page.getByRole('button', { name: 'Continue to Model', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Model', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'Models · CLIProxyAPI', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'CLIProxyAPI → ChatGPT', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'CLIProxyAPI → DeepSeek', exact: true }).waitFor({ state: 'visible' });
  await page.waitForFunction(() => {
    const title = document.getElementById('model-title');
    const cards = [...document.querySelectorAll('.model-provider-card')];
    const stable = title?.textContent?.trim() === 'Models · CLIProxyAPI' && cards.length === 2 && cards.every(card => {
      const rect = card.getBoundingClientRect();
      const style = getComputedStyle(card);
      return rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden' && style.display !== 'none';
    });
    if (!stable) { delete window.__s33A4StableSince; return false; }
    window.__s33A4StableSince ??= performance.now();
    return performance.now() - window.__s33A4StableSince >= 400;
  }, undefined, { timeout: 10000, polling: 50 });
  const modelOnboardingObservations = [];
  let modelCursor = (await log(0)).cursor;
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const chatgpt = page.getByRole('heading', { name: 'CLIProxyAPI → ChatGPT', exact: true });
    const deepseek = page.getByRole('heading', { name: 'CLIProxyAPI → DeepSeek', exact: true });
    await chatgpt.waitFor({ state: 'visible' });
    await deepseek.waitFor({ state: 'visible' });
    const fallback = page.getByRole('checkbox', { name: /Allow automatic fallback to DeepSeek/ });
    check(await fallback.isVisible() && !(await fallback.isChecked()),
      `A4 automatic fallback is visible and OFF at ${width}px.`);
    check(await page.getByText(/The key never enters the webview, SQLite, events or logs\./).isVisible(),
      `A4 DeepSeek key-storage disclosure is visible at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
    }));
    check(geometry.documentScrollWidth <= geometry.viewport,
      `A4 Model step has no document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo({ top: 0, left: 0, behavior: 'instant' }));
      await page.waitForFunction(() => window.scrollY === 0);
      await page.screenshot({ path: 'docs/implementation/evidence/s33/model-onboarding-390.png' });
    }
    const passive = await assertNoProviderIo(`A4 Model onboarding at ${width}px`, modelCursor);
    modelCursor = passive.cursor;
    modelOnboardingObservations.push({ width, chatgptCardVisible: true, deepseekCardVisible: true, fallbackOff: true,
      keyStorageDisclosureVisible: true, documentOverflow: false, forbiddenCommands: 0 });
  }

  // Only integration-test fixtures seed the synthetic route and gateway. They do not launch a sidecar,
  // contact a model provider, perform OAuth, store credentials, or run inference.
  await command('model.route.fixture', { workspaceId: workspace.workspaceId });
  await command('model.gateway.fixture', { workspaceId: workspace.workspaceId, status: 'RUNNING' });
  await page.getByRole('heading', { name: 'CLIProxyAPI → DeepSeek', exact: true }).waitFor({ state: 'visible' });
  const deepseekCard = page.locator('.model-provider-card').filter({ hasText: 'CLIProxyAPI → DeepSeek' });
  await deepseekCard.getByText('Ready', { exact: true }).waitFor({ state: 'visible' });
  const syntheticRoute = deepseekCard.locator('.model-route').filter({ hasText: 'deepseek-v4-flash · Non-thinking' });
  const useSyntheticDefault = syntheticRoute.getByRole('button', { name: 'Use as default', exact: true });
  check(await useSyntheticDefault.count() === 1 && await useSyntheticDefault.isEnabled(),
    'The fixture route is available for local default selection.');
  await useSyntheticDefault.press('Enter');
  await page.getByText(/Default for next request: deepseek-v4-flash · Non-thinking via DeepSeek official API/).waitFor({ state: 'visible' });
  const routeSetup = await log(onboardingCursor);
  check(routeSetup.commands.filter(entry => entry.command === 'model.set_default').length === 1,
    'Exactly one local model.set_default selects the synthetic route.');
  onboardingCursor = routeSetup.cursor;

  await page.getByRole('button', { name: 'Continue to Risk defaults', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Risk defaults', exact: true }).waitFor({ state: 'visible' });
  const riskDefaultsObservations = [];
  let riskCursor = onboardingCursor;
  const optionalRiskFields = [
    ['Maximum order notional (USD)', page.getByRole('textbox', { name: 'Maximum order notional (USD)', exact: true })],
    ['Maximum single-instrument exposure (%)', page.getByRole('textbox', { name: 'Maximum single-instrument exposure (%)', exact: true })],
    ['Maximum daily traded notional (USD)', page.getByRole('textbox', { name: 'Maximum daily traded notional (USD)', exact: true })],
    ['Maximum daily realized loss (USD)', page.getByRole('textbox', { name: 'Maximum daily realized loss (USD)', exact: true })],
  ];
  const staleThreshold = page.getByRole('spinbutton', { name: 'Stale quote threshold (seconds)', exact: true });
  const marketOrders = page.getByRole('checkbox', { name: /Allow market orders/ });
  const inactivityTimeout = page.getByRole('spinbutton', { name: 'Live inactivity timeout (minutes)', exact: true });
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    for (const [name, field] of optionalRiskFields) {
      await field.waitFor({ state: 'visible' });
      const box = await field.boundingBox();
      check(Boolean(box) && box.x >= 0 && box.x + box.width <= width,
        `A5 ${name} field stays in the viewport at ${width}px: ${JSON.stringify(box)}`);
      check(await field.inputValue() === '' && await field.getAttribute('placeholder') === 'Leave unset',
        `A5 ${name} is unset by default.`);
    }
    check(await staleThreshold.isVisible() && await staleThreshold.inputValue() === '3',
      `A5 stale quote threshold defaults to 3 seconds at ${width}px.`);
    check(await marketOrders.isVisible() && !(await marketOrders.isChecked()),
      `A5 market orders default OFF at ${width}px.`);
    check(await inactivityTimeout.isVisible() && await inactivityTimeout.inputValue() === '20',
      `A5 live inactivity timeout defaults to 20 minutes at ${width}px.`);
    const hardRules = page.locator('.hard-rules');
    check(await hardRules.getByRole('heading', { name: 'Hard safety rules · read only', exact: true }).isVisible()
      && await hardRules.locator('li').count() > 0,
    `A5 hard safety rules are visible and read-only at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
    }));
    check(geometry.documentScrollWidth <= geometry.viewport,
      `A5 Risk defaults has no document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    const save = page.getByRole('button', { name: 'Save and continue', exact: true });
    check(await save.isVisible() && await save.isEnabled(), `A5 save action is available at ${width}px.`);
    if (width === 390) {
      await page.screenshot({ path: 'docs/implementation/evidence/s33/risk-defaults-390.png', fullPage: true });
    }
    const passive = await assertNoProviderIo(`A5 Risk defaults before save at ${width}px`, riskCursor);
    riskCursor = passive.cursor;
    riskDefaultsObservations.push({ width, optionalLimitsUnset: true, staleQuoteThresholdSeconds: 3,
      marketOrdersEnabled: false, liveInactivityTimeoutMinutes: 20, hardRulesReadOnly: true, documentOverflow: false });
  }
  const beforeSave = await log(riskCursor);
  check(!beforeSave.commands.some(entry => entry.command === 'risk.save_policy'),
    'A5 remains unchanged until the explicit Save and continue action.');
  riskCursor = beforeSave.cursor;
  await page.getByRole('button', { name: 'Save and continue', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const saveResult = await log(riskCursor);
  check(saveResult.commands.filter(entry => entry.command === 'risk.save_policy').length === 1,
    'A5 persists the selected defaults exactly once.');
  riskCursor = saveResult.cursor;
  const persistedRisk = await command('risk.get_policy', { workspaceId: workspace.workspaceId });
  check(persistedRisk.configured === true
    && persistedRisk.policy.maxOrderNotional == null
    && persistedRisk.policy.maxSingleInstrumentExposurePercent == null
    && persistedRisk.policy.maxDailyTradedNotional == null
    && persistedRisk.policy.maxDailyRealizedLoss == null
    && persistedRisk.policy.staleQuoteThresholdSeconds === 3
    && persistedRisk.policy.marketOrdersEnabled === false
    && persistedRisk.policy.liveInactivityTimeoutMinutes === 20,
  `A5 persisted policy matches its safe defaults: ${JSON.stringify(persistedRisk.policy)}`);

  const readyObservations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const readySummary = page.locator('.summary-list');
    check(await page.getByText(`${workspaceName} · USD`, { exact: true }).isVisible(),
      `A6 workspace and base currency are visible at ${width}px.`);
    check(await page.getByText('Local Paper · LOCAL PAPER · TradeX simulation · local-paper LOCAL', { exact: true }).isVisible(),
      `A6 provider summary reports only the built-in Local Paper account at ${width}px.`);
    check(await readySummary.getByText(/DEEPSEEK · deepseek-v4-flash/).isVisible(),
      `A6 displays the explicitly synthetic model route at ${width}px.`);
    check(await readySummary.getByText('OFF', { exact: true }).isVisible(),
      `A6 reports automatic fallback OFF at ${width}px.`);
    check(await readySummary.getByText('All DISARMED', { exact: true }).isVisible(),
      `A6 reports Live accounts DISARMED at ${width}px.`);
    check(await page.getByText('Agent turns unavailable', { exact: true }).isVisible(),
      `A6 explains that the agent runtime is unavailable at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
    }));
    check(geometry.documentScrollWidth <= geometry.viewport,
      `A6 Ready has no document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    const boxes = await readySummary.locator('div').evaluateAll(rows => rows.map(row => {
      const rect = row.getBoundingClientRect();
      return { x: rect.x, right: rect.right, width: rect.width };
    }));
    check(boxes.length === 5 && boxes.every(box => box.x >= 0 && box.right <= width),
      `A6 summary rows stay in the viewport at ${width}px: ${JSON.stringify(boxes)}`);
    if (width === 390) {
      await page.screenshot({ path: 'docs/implementation/evidence/s33/ready-onboarding-390.png', fullPage: true });
    }
    const passive = await assertNoProviderIo(`A6 Ready at ${width}px`, riskCursor);
    riskCursor = passive.cursor;
    readyObservations.push({ width, workspaceAndCurrency: true, localPaperOnly: true, noExternalProvider: true,
      syntheticRouteDisclosedInEvidence: true, fallbackOff: true, liveAccountsDisarmed: true,
      agentRuntimeUnavailable: true, documentOverflow: false });
  }
  const complete = page.getByRole('button', { name: 'Complete onboarding', exact: true });
  check(await complete.isEnabled(), 'A6 completion is enabled after local synthetic route setup and saved risk defaults.');
  await complete.press('Enter');
  await page.getByRole('heading', { name: 'What would you like to research?', exact: true }).waitFor({ state: 'visible' });

  const cases = [
    ['+ New Thread', 'What would you like to research?'],
    ['Threads', 'Threads'], ['Markets', 'Markets'], ['Watchlists', 'Watchlists'],
    ['Order Drafts', 'Order Drafts'], ['Accounts', 'Account connections'],
    ['Strategies', 'Strategies'], ['Artifacts', 'Artifacts'], ['Settings', 'Settings'],
  ];
  const viewports = [1280, 768, 390];
  const observed = [];
  let cursor = (await log(0)).cursor;
  for (const width of viewports) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    for (const [destination, heading] of cases) {
      const mobile = page.locator('details.mobile-nav');
      if (width < 900 && await mobile.count() && !(await mobile.evaluate(element => element.open))) {
        await page.getByText('More', { exact: true }).press('Enter');
      }
      await page.getByRole('navigation', { name: 'Primary navigation' })
        .getByRole('button', { name: destination, exact: true }).press('Enter');
      await page.getByRole('heading', { name: heading, exact: true }).first().waitFor({ state: 'visible' });
      await page.waitForTimeout(150);
      const size = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      check(size.scroll <= size.viewport, `${destination} overflows at ${width}px: ${JSON.stringify(size)}`);
      const result = await assertNoProviderIo(`${destination} at ${width}px`, cursor);
      cursor = result.cursor;
      observed.push({ destination, width, commands: result.commands });
    }
    if (await page.getByRole('heading', { name: 'Settings', exact: true }).count()) {
      const tabs = page.getByRole('group', { name: 'Settings sections', exact: true });
      for (const name of ['Providers & Models', 'Risk & Limits', 'Data & Storage', 'Account Health', 'Appearance', 'About']) {
        await tabs.getByRole('button', { name, exact: true }).press('Enter');
        await page.waitForTimeout(150);
        const result = await assertNoProviderIo(`Settings → ${name} at ${width}px`, cursor);
        cursor = result.cursor;
        observed.push({ destination: `Settings → ${name}`, width, commands: result.commands });
      }
    }
  }
  check(externalRequests.length === 0 && pageErrors.length === 0 && consoleErrors.length === 0,
    `No external HTTP requests, page errors, or non-icon console errors: ${JSON.stringify({ externalRequests, pageErrors, consoleErrors })}`);
  const expectedFixtureCommands = ['model.route.fixture', 'model.gateway.fixture'];
  check(expectedFixtureCommands.every(name => fixtureCommands.includes(name)),
    `Both local-only model fixture seams were used: ${JSON.stringify(fixtureCommands)}`);
  return { result: 'PASS', capturedAt: new Date().toISOString(), workspace: workspace.workspaceId,
    providerRequests: 0, externalHttpRequests: externalRequests.length,
    pageErrors: pageErrors.length, consoleErrors: consoleErrors.length, localFavicon404: staticIconErrors.length,
    syntheticModelFixtures: expectedFixtureCommands,
    onboardingObservations, providerOnboardingObservations, modelOnboardingObservations, riskDefaultsObservations,
    readyObservations, riskSavedOnce: true, onboardingCompletedThroughUi: true,
    viewports: viewports.length, surfaces: observed.length, observed };
}
