// Run against the Rust-backed browser integration harness on port 1420.
// Verifies Ask's lightweight/read-only surface and explicit mode promotion
// controls without sending a Turn or invoking a provider/model runtime.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const requestUrl = request.url();
    if (/^https?:\/\//i.test(requestUrl)
      && !/^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl)) {
      externalRequests.push(requestUrl.match(/^https?:\/\/[^/]+/i)?.[0] ?? requestUrl);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = `s33-b3-${name}-${Date.now()}-${Math.random()}`;
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

  await page.clock.install({ time: new Date() });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  await page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  ).waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 B3 Ask mode ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-b3-ask-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'What would you like to research?', exact: true }).waitFor({ state: 'visible' });

  const newComposer = page.locator('.thread-composer');
  const newMode = newComposer.getByRole('combobox', { name: 'Agent mode', exact: true });
  check(await newMode.inputValue() === 'ASK', 'New Threads default to Ask mode.');
  await newComposer.getByRole('textbox', { name: 'Thread title', exact: true }).fill('B3 read-only Ask with MSFT');
  await newComposer.getByRole('button', { name: /@ Context/ }).press('Enter');
  const picker = page.getByRole('dialog');
  await picker.waitFor({ state: 'visible' });
  await picker.getByRole('checkbox', { name: /equity:US:MSFT/ }).check();
  await picker.getByRole('button', { name: 'Attach', exact: true }).press('Enter');
  await picker.waitFor({ state: 'hidden' });
  await newComposer.locator('.capability-summary strong').filter({ hasText: 'Capability: C0' }).waitFor({ state: 'visible' });

  const cursor = (await commandLog(0)).cursor;
  await newComposer.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  const title = 'B3 read-only Ask with MSFT';
  await page.getByRole('heading', { name: title, exact: true }).waitFor({ state: 'visible' });
  const listed = await command('thread.list', { workspaceId: workspace.workspaceId });
  const thread = listed.threads.find(item => item.title === title);
  check(Boolean(thread?.threadId), 'Ask Thread appears in local history.');
  const snapshot = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
  check(snapshot.defaultAgentMode === 'ASK' && snapshot.defaultExecutionContext === 'NONE_READ_ONLY',
    'Persisted Thread defaults remain Ask and read-only.');
  check(snapshot.turns.length === 0 && snapshot.linkedContexts.some(context =>
    context.kind === 'instrument' && context.id === 'equity:US:MSFT'),
  'The Thread retains its explicit MSFT context and has no Turn.');

  const composer = page.locator('.thread-detail .turn-composer');
  const detail = page.locator('.thread-detail');
  const mode = composer.getByRole('combobox', { name: 'Agent mode', exact: true });
  const execution = composer.getByRole('combobox', { name: 'Execution context', exact: true });
  const account = composer.getByRole('combobox', { name: 'Account', exact: true });
  const askState = async width => {
    check(await mode.inputValue() === 'ASK', `Ask remains selected at ${width}px.`);
    check(await execution.inputValue() === 'NONE_READ_ONLY', `Ask remains read-only at ${width}px.`);
    check(await account.inputValue() === '', `No execution account is silently attached at ${width}px.`);
    check(await composer.getByRole('heading', { name: 'Ask this Thread', exact: true }).isVisible(),
      `Ask composer is visible at ${width}px.`);
    check(await composer.getByText('No financial approval', { exact: true }).isVisible(),
      `Ask disclosure is visible at ${width}px.`);
    check(await composer.locator('.capability-summary strong').innerText() === 'Capability: C0',
      `Instrument-only Ask remains at read-only C0 at ${width}px.`);
    check(await detail.getByText('No turns have started. Send a request to begin the read-only timeline.', { exact: true }).isVisible(),
      'Ask starts without a heavy plan or an automatic research Turn.');
    check(await composer.getByRole('button', { name: /Remove .*MSFT/ }).isVisible(),
      `The explicitly attached instrument remains visible at ${width}px.`);
    check(await composer.getByRole('button', { name: /Prepare order|Submit order|Create proposal|Approve|Arm account/i }).count() === 0,
      `Ask exposes no trade-preparation or financial-approval action at ${width}px.`);
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, documentScroll: document.documentElement.scrollWidth }));
    check(geometry.documentScroll <= geometry.viewport, `Ask overflows at ${width}px: ${JSON.stringify(geometry)}`);
    return { width, mode: 'ASK', executionContext: 'NONE_READ_ONLY', capability: 'C0', documentOverflow: false };
  };

  const widths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await askState(width);
    widths.push({ width, mode: 'ASK', executionContext: 'NONE_READ_ONLY', capability: 'C0', documentOverflow: false });
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: 'docs/implementation/evidence/s33/b3-ask-mode-390.png', fullPage: true });
    }
  }

  const options = await mode.locator('option').evaluateAll(elements => elements.map(option => ({ value: option.value, label: option.textContent?.trim() })));
  check(['ASK', 'RESEARCH', 'BACKTEST', 'TRADE'].every(value => options.some(option => option.value === value)),
    'The explicit mode control offers Ask, Research, Backtest, and Trade upgrades.');
  check(options.find(option => option.value === 'TRADE')?.label?.includes('approval'),
    'Trade mode option explicitly discloses later approval.');

  await mode.selectOption('RESEARCH');
  await composer.getByRole('button', { name: 'Open Strategies from RESEARCH context', exact: true }).waitFor({ state: 'visible' });
  await composer.locator('.capability-summary strong').filter({ hasText: 'Capability: C0' }).waitFor({ state: 'visible' });
  check(await execution.inputValue() === 'NONE_READ_ONLY', 'Promoting to Research does not change the execution context.');

  await mode.selectOption('BACKTEST');
  await execution.selectOption('HISTORICAL_SIMULATION');
  await composer.getByRole('button', { name: 'Open Strategies from BACKTEST context', exact: true }).waitFor({ state: 'visible' });
  await composer.locator('.capability-summary strong').filter({ hasText: 'Capability: C2' }).waitFor({ state: 'visible' });
  check(await execution.inputValue() === 'HISTORICAL_SIMULATION', 'Backtest promotion is explicitly paired with historical simulation.');
  check(await composer.getByRole('heading', { name: 'Backtest', exact: true }).isVisible(),
    'Backtest-specific controls render without invoking a run.');

  await execution.selectOption('NONE_READ_ONLY');
  await mode.selectOption('TRADE');
  const invalidModeAlert = composer.getByRole('alert');
  await invalidModeAlert.filter({ hasText: 'UNSUPPORTED_CAPABILITY' }).waitFor({ state: 'visible' });
  check((await invalidModeAlert.innerText()).includes('Choose a supported context'),
    'An invalid mode/context combination gives an actionable explanation.');
  check(await execution.inputValue() === 'NONE_READ_ONLY' && await account.inputValue() === '',
    'Selecting Trade without an account fails closed and does not silently change authority.');
  check(await composer.getByRole('button', { name: /Prepare order|Submit order|Create proposal|Approve|Arm account/i }).count() === 0,
    'An invalid Trade combination does not expose preparation or approval actions.');

  await mode.selectOption('ASK');
  await composer.locator('.capability-summary strong').filter({ hasText: 'Capability: C0' }).waitFor({ state: 'visible' });
  const after = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
  check(after.defaultAgentMode === 'ASK' && after.defaultExecutionContext === 'NONE_READ_ONLY'
    && after.turns.length === 0 && after.linkedContexts.some(context => context.kind === 'instrument' && context.id === 'equity:US:MSFT'),
  'Exploring upgrade options leaves persisted Ask defaults, context, and empty timeline unchanged.');

  const log = await commandLog(cursor);
  const forbidden = log.commands.filter(entry => entry.command === 'turn.start' || entry.command === 'research.run'
    || entry.command.startsWith('provider.') || entry.command.startsWith('alpaca.')
    || entry.command.startsWith('binance.') || entry.command.startsWith('bitget.')
    || entry.command.startsWith('trading212.') || entry.command.startsWith('trade.')
    || entry.command === 'account.arm' || entry.command.startsWith('approval.'));
  check(forbidden.length === 0, `Ask and its uncommitted mode changes issue no runtime or financial command: ${JSON.stringify(forbidden)}`);
  check(log.commands.filter(entry => entry.command === 'thread.create').length === 1,
    'Exactly one explicit Ask Thread is created.');
  check(externalRequests.length === 0, `No external HTTP requests occurred: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0, `No page errors occurred: ${JSON.stringify(pageErrors)}`);
  check(consoleErrors.length === 0, `No console errors occurred: ${JSON.stringify(consoleErrors)}`);
  return {
    result: 'PASS_LIMITED',
    verifiedAt: new Date().toISOString(),
    fixtureBoundary: 'One disposable Rust-backed workspace; workspace.ready.fixture synthetic ready state only; no model/provider runtime or inference.',
    workspaceId: workspace.workspaceId,
    threadId: thread.threadId,
    widths,
    ask: { mode: 'ASK', executionContext: 'NONE_READ_ONLY', capability: 'C0', linkedInstrument: 'equity:US:MSFT', persistedTurns: 0, tradePreparationControls: 0 },
    upgradeOptions: options,
    exploredModes: ['RESEARCH', 'BACKTEST+HISTORICAL_SIMULATION', 'TRADE+NONE_READ_ONLY_BLOCKED'],
    threadCreates: 1,
    turnsStarted: 0,
    researchRuns: 0,
    providerMutationCommands: 0,
    externalHttpRequests: externalRequests.length,
    pageErrors: pageErrors.length,
    consoleErrors: consoleErrors.length,
    limitations: ['No model route/runtime was configured, so no answer or model inference was started.', 'Native VoiceOver and full S33 acceptance remain open.'],
  };
}
