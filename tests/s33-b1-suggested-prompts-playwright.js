// Run against the Rust-backed browser integration harness on port 1420.
// Suggested prompts remain local drafts until the user creates an empty Thread
// and explicitly sends the first Turn.
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
    const requestId = `s33-b1-${name}-${Date.now()}-${Math.random()}`;
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
  await page.goto('http://127.0.0.1:1420/');
  await page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  ).waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 B1 prompts ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-b1-prompts-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'What would you like to research?', exact: true }).waitFor({ state: 'visible' });

  const prompts = [
    'Compare NVDA, AMD and AVGO',
    'Review my live portfolio risk',
    'Backtest BTC momentum strategy',
    'Compare BTC liquidity on Binance and Bitget',
  ];
  const cursor = (await commandLog(0)).cursor;
  const observed = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const indexes = width === 1280 ? [0] : width === 768 ? [1] : [2, 3];
    for (const index of indexes) {
      const prompt = prompts[index];
      const button = page.getByRole('button', { name: prompt, exact: true });
      check(await button.isVisible() && await button.isEnabled(), `Prompt is visible and enabled at ${width}px: ${prompt}`);
      await button.press('Enter');
      check(await button.getAttribute('aria-pressed') === 'true', `Keyboard selection is announced for: ${prompt}`);
      check(await page.locator('.suggested-prompt[aria-pressed="true"]').count() === 1,
        'Selecting a suggestion leaves exactly one prompt selected.');
      const geometry = await page.evaluate(() => ({
        viewport: document.documentElement.clientWidth,
        documentScroll: document.documentElement.scrollWidth,
      }));
      check(geometry.documentScroll <= geometry.viewport, `New Thread overflows at ${width}px: ${JSON.stringify(geometry)}`);
      observed.push({ width, prompt, selected: true, overflow: false });
    }
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: 'docs/implementation/evidence/s33/b1-suggested-prompts-selected-390.png' });
    }
  }

  const promptSelectionLog = await commandLog(cursor);
  const promptSelectionMutations = promptSelectionLog.commands.filter(entry =>
    ['thread.create', 'turn.start'].includes(entry.command)
    || entry.command.startsWith('provider.') || entry.command.startsWith('alpaca.')
    || entry.command.startsWith('binance.') || entry.command.startsWith('bitget.')
    || entry.command.startsWith('trading212.') || entry.command.startsWith('trade.')
    || entry.command.startsWith('live.gateway.') || entry.command.startsWith('model.verify'));
  check(promptSelectionMutations.length === 0,
    `Choosing prompts must remain local and must not create a Thread, start a Turn, or call a provider: ${JSON.stringify(promptSelectionMutations)}`);

  const create = page.getByRole('button', { name: 'Create Thread', exact: true });
  await page.waitForFunction(() => {
    const button = [...document.querySelectorAll('button')].find(node => node.textContent?.trim() === 'Create Thread');
    return Boolean(button && !button.disabled);
  });
  await create.press('Enter');
  await page.getByRole('heading', { name: 'New research thread', exact: true }).waitFor({ state: 'visible' });
  const requestField = page.getByRole('textbox', { name: 'Turn request', exact: true });
  await requestField.waitFor({ state: 'visible' });
  check(await requestField.inputValue() === prompts[3], 'The selected suggestion is available to edit as the first Turn draft.');
  await page.getByText('No turns have started. Send a request to begin the read-only timeline.', { exact: true })
    .waitFor({ state: 'visible' });

  const threads = await command('thread.list', { workspaceId: workspace.workspaceId });
  const created = threads.threads.find(thread => thread.title === 'New research thread');
  check(Boolean(created?.threadId), 'Create Thread persists the local Thread.');
  const detail = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: created.threadId });
  check(detail.turns.length === 0, 'The created Thread remains empty until Send is explicitly activated.');
  const afterCreate = await commandLog(cursor);
  const threadCreates = afterCreate.commands.filter(entry => entry.command === 'thread.create');
  const turnStarts = afterCreate.commands.filter(entry => entry.command === 'turn.start');
  check(threadCreates.length === 1, `Create Thread issues exactly one thread.create: ${threadCreates.length}`);
  check(turnStarts.length === 0, `Creating a Thread does not issue turn.start: ${turnStarts.length}`);
  check(externalRequests.length === 0, `No external HTTP requests occurred: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0, `No page errors occurred: ${JSON.stringify(pageErrors)}`);
  check(consoleErrors.length === 0, `No console errors occurred: ${JSON.stringify(consoleErrors)}`);

  await requestField.scrollIntoViewIfNeeded();
  await page.locator('.turn-composer label.field').first()
    .screenshot({ path: 'docs/implementation/evidence/s33/b1-empty-thread-with-draft-390.png' });
  return {
    result: 'PASS',
    verifiedAt: new Date().toISOString(),
    fixtureBoundary: 'one disposable local workspace; synthetic ready fixture only',
    promptTexts: prompts,
    observed,
    selectedPrompt: prompts[3],
    promptSelectionMutationCommands: promptSelectionMutations.length,
    workspaceId: workspace.workspaceId,
    createdThreadId: created.threadId,
    persistedTurns: detail.turns.length,
    threadCreateCommands: threadCreates.length,
    turnStartCommands: turnStarts.length,
    externalHttpRequests: externalRequests.length,
    pageErrors: pageErrors.length,
    consoleErrors: consoleErrors.length,
  };
}
