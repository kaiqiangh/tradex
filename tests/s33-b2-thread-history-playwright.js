// Run against the Rust-backed browser integration harness on port 1420.
// Exercises local Thread history grouping and restores the selected Thread's
// own defaults/context without starting another Turn.
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
    const requestId = `s33-b2-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 B2 history ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-b2-history-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'What would you like to research?', exact: true }).waitFor({ state: 'visible' });

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  const navigate = async destination => {
    const mobile = page.locator('details.mobile-nav');
    if (await mobile.isVisible()) {
      if (!(await mobile.evaluate(element => element.open))) await mobile.locator('summary').press('Enter');
      await mobile.getByRole('navigation', { name: 'Primary navigation', exact: true })
        .getByRole('button', { name: destination, exact: true }).press('Enter');
    } else {
      await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
        .getByRole('button', { name: destination, exact: true }).press('Enter');
    }
  };
  await navigation.getByRole('button', { name: 'Threads', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  const history = page.locator('.threads-list-card .thread-history');
  const groups = history.locator('.thread-history-group');
  check(await groups.count() === 3, 'Empty history exposes Today, Earlier, and Saved groups.');
  for (const label of ['Today', 'Earlier', 'Saved']) {
    check(await history.getByRole('heading', { name: label, exact: true }).isVisible(), `Empty history shows ${label}.`);
  }
  check(await history.getByText('No saved threads', { exact: true }).isVisible(), 'Empty Saved group has a clear state.');

  const createThread = async (title, mode, instrumentId) => {
    await navigation.getByRole('button', { name: '+ New Thread', exact: true }).press('Enter');
    await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill(title);
    await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption(mode);
    if (instrumentId) {
      await page.getByRole('button', { name: /@ Context/, exact: false }).click();
      const dialog = page.getByRole('dialog');
      await dialog.waitFor({ state: 'visible' });
      await dialog.getByRole('checkbox', { name: new RegExp(instrumentId.replaceAll(':', '\\:')) }).check();
      await dialog.getByRole('button', { name: 'Attach', exact: true }).click();
      await dialog.waitFor({ state: 'hidden' });
    }
    await page.getByRole('button', { name: 'Create Thread', exact: true }).click();
    await page.getByRole('heading', { name: title, exact: true }).waitFor({ state: 'visible' });
    const listed = await command('thread.list', { workspaceId: workspace.workspaceId });
    const thread = listed.threads.find(item => item.title === title);
    check(Boolean(thread?.threadId), `Thread history includes ${title}.`);
    return thread;
  };

  const researchTitle = 'B2 saved research defaults';
  const researchThread = await createThread(researchTitle, 'RESEARCH', 'equity:US:MSFT');
  const askTitle = 'B2 separate Ask defaults';
  const askThread = await createThread(askTitle, 'ASK');
  const researchSnapshot = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: researchThread.threadId });
  const askSnapshot = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: askThread.threadId });
  check(researchSnapshot.defaultAgentMode === 'RESEARCH' && researchSnapshot.linkedContexts.some(context => context.kind === 'instrument' && context.id === 'equity:US:MSFT'),
    'The first Thread persists its own Research mode and canonical MSFT context.');
  check(askSnapshot.defaultAgentMode === 'ASK' && askSnapshot.linkedContexts.length === 0,
    'The second Thread persists its separate Ask mode and empty context.');

  const widths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await navigate('Threads');
    await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
    const todayGroup = history.locator('.thread-history-group').filter({ has: page.getByRole('heading', { name: 'Today', exact: true }) });
    check(await todayGroup.getByRole('button', { name: researchTitle, exact: true }).isVisible(), `Today contains the Research Thread at ${width}px.`);
    check(await todayGroup.getByRole('button', { name: askTitle, exact: true }).isVisible(), `Today contains the Ask Thread at ${width}px.`);
    check(await history.getByRole('heading', { name: 'Earlier', exact: true }).isVisible()
      && await history.getByRole('heading', { name: 'Saved', exact: true }).isVisible(), `All history groups remain visible at ${width}px.`);
    check(await history.getByText('No saved threads', { exact: true }).isVisible(),
      `The Saved group remains clear when only active Threads exist at ${width}px.`);
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, documentScroll: document.documentElement.scrollWidth }));
    check(geometry.documentScroll <= geometry.viewport, `Thread history overflows at ${width}px: ${JSON.stringify(geometry)}`);
    widths.push({ width, todayThreads: 2, documentOverflow: false });
  }
  if (widths.some(item => item.width === 390)) {
    await page.locator('.threads-list-card').screenshot({ path: 'docs/implementation/evidence/s33/b2-thread-history-today-390.png' });
  }

  await page.clock.install({ time: new Date(Date.now() + 86_400_000) });
  await navigate('+ New Thread');
  await navigate('Threads');
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  const earlier = history.locator('.thread-history-group').filter({ has: page.getByRole('heading', { name: 'Earlier', exact: true }) });
  check(await earlier.getByRole('button', { name: researchTitle, exact: true }).isVisible()
    && await earlier.getByRole('button', { name: askTitle, exact: true }).isVisible(),
  'Threads from the preceding local date are grouped under Earlier.');
  check(await history.getByText('No threads in this period', { exact: true }).isVisible(),
    'Today has a readable empty state when there are no Threads for that local date.');

  const resumeCursor = (await commandLog(0)).cursor;
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const earlierAtWidth = history.locator('.thread-history-group').filter({ has: page.getByRole('heading', { name: 'Earlier', exact: true }) });
    const entry = earlierAtWidth.getByRole('button', { name: researchTitle, exact: true });
    await entry.press('Enter');
    check(await entry.getAttribute('aria-current') === 'true', `Keyboard Enter selects the exact history item at ${width}px.`);
    const detail = page.locator('.thread-detail');
    await detail.waitFor({ state: 'visible' });
    const resumedMode = detail.getByRole('combobox', { name: 'Agent mode', exact: true });
    await resumedMode.waitFor({ state: 'visible' });
    check(await resumedMode.inputValue() === 'RESEARCH',
      `Resume restores the selected Thread's saved mode at ${width}px.`);
    check(await detail.getByText('Context references: 1', { exact: true }).isVisible()
      && await detail.getByRole('button', { name: /Remove .*MSFT/ }).isVisible(),
    `Resume restores the canonical instrument context at ${width}px.`);
    check(await detail.getByText('No turns have started. Send a request to begin the read-only timeline.', { exact: true }).isVisible(),
      'Resume restores the empty timeline without creating a Turn.');
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, documentScroll: document.documentElement.scrollWidth }));
    check(geometry.documentScroll <= geometry.viewport, `Resumed Thread overflows at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: 'docs/implementation/evidence/s33/b2-thread-resume-context-390.png', fullPage: true });
    }
  }
  const resumeLog = await commandLog(resumeCursor);
  const turnStarts = resumeLog.commands.filter(entry => entry.command === 'turn.start');
  const providerMutations = resumeLog.commands.filter(entry => entry.command.startsWith('provider.')
    || entry.command.startsWith('alpaca.') || entry.command.startsWith('binance.')
    || entry.command.startsWith('bitget.') || entry.command.startsWith('trading212.')
    || entry.command.startsWith('trade.') || entry.command.startsWith('live.gateway.')
    || entry.command === 'account.refresh');
  check(turnStarts.length === 0, `History selection does not start a Turn: ${turnStarts.length}`);
  check(providerMutations.length === 0, `Resume does not mutate a provider/account/financial state: ${JSON.stringify(providerMutations)}`);
  check(externalRequests.length === 0, `No external HTTP requests occurred: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0, `No page errors occurred: ${JSON.stringify(pageErrors)}`);
  check(consoleErrors.length === 0, `No console errors occurred: ${JSON.stringify(consoleErrors)}`);
  return {
    result: 'PASS',
    verifiedAt: new Date().toISOString(),
    fixtureBoundary: 'One disposable Rust-backed workspace; workspace.ready.fixture synthetic ready state only',
    workspaceId: workspace.workspaceId,
    threadIds: { research: researchThread.threadId, ask: askThread.threadId },
    widths,
    earlierThreads: 2,
    savedGroupEmptyState: true,
    resumed: { title: researchTitle, mode: 'RESEARCH', linkedInstrument: 'equity:US:MSFT', turnsPersisted: researchSnapshot.turns.length },
    turnStartCommandsAfterResume: turnStarts.length,
    providerMutationCommandsAfterResume: providerMutations.length,
    externalHttpRequests: externalRequests.length,
    pageErrors: pageErrors.length,
    consoleErrors: consoleErrors.length,
  };
}
