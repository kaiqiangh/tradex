// Run against the integration Vite bridge with
// TRADEX_FAKE_APP_SERVER_SCENARIO=research-plan-tools.
// The runtime frames and research rows are local synthetic fixtures only.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const pageErrors = [];
  const consoleErrors = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  const command = async (name, payload) => {
    const requestId = `s33-b5-${name}-${Date.now()}-${Math.random()}`;
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
  const commandCursor = await commandLog(0);
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 B5 research running ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-b5-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Threads', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  const threadTitle = `B5 research running ${stamp}`;
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill(threadTitle);
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
  await page.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: threadTitle, exact: true }).waitFor({ state: 'visible' });

  const request = 'Summarize the synthetic AAPL research result and its limitations.';
  await page.getByLabel('Turn request', { exact: true }).fill(request);
  const researchTool = page.getByRole('combobox', { name: 'Research tool', exact: true });
  const toolId = await researchTool.locator('option').first().getAttribute('value');
  check(toolId === 'public_market_read', `Expected the authorized read-only tool, received ${toolId}.`);
  await page.getByRole('combobox', { name: 'Research focus', exact: true }).selectOption('EQUITY');
  await page.getByRole('button', { name: 'Preview typed research result', exact: true }).press('Enter');
  const preview = page.locator('.research-preview .research-result');
  await preview.getByText('Typed result · AVAILABLE', { exact: true }).waitFor({ state: 'visible' });
  check(await preview.getByText('Synthetic fixture · SYNTHETIC_INTEGRATION_FIXTURE', { exact: true }).isVisible(),
    'The interim result must identify its synthetic-fixture boundary.');
  check(await page.getByRole('status', { name: 'Research lifecycle', exact: true }).innerText() === 'Research status: DONE',
    'A completed typed preview must report DONE.');

  await page.evaluate(() => {
    const transitions = {};
    const sample = () => {
      for (const item of document.querySelectorAll('.timeline-item')) {
        const type = item.querySelector('.timeline-item-heading strong')?.textContent?.trim();
        const status = item.getAttribute('data-item-status');
        if (!type || !status) continue;
        const values = transitions[type] ?? (transitions[type] = []);
        if (values.at(-1) !== status) values.push(status);
      }
    };
    sample();
    const observer = new MutationObserver(sample);
    observer.observe(document.body, { attributes: true, childList: true, characterData: true, subtree: true });
    window.__s33B5Capture = { transitions, observer };
  });
  await page.getByRole('button', { name: 'Send', exact: true }).press('Enter');
  const turnStatus = page.getByRole('status', { name: 'Turn 1 status', exact: true });
  await turnStatus.waitFor({ state: 'visible' });
  await page.waitForFunction(() => document.querySelector('[role="status"][aria-label="Turn 1 status"]')?.textContent?.trim() === 'COMPLETED');
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));

  const stream = await page.evaluate(() => {
    const capture = window.__s33B5Capture;
    capture.observer.disconnect();
    const items = [...document.querySelectorAll('.timeline-item')].map(item => ({
      type: item.querySelector('.timeline-item-heading strong')?.textContent?.trim(),
      status: item.getAttribute('data-item-status'),
      detailLabel: item.querySelector('[role="group"][aria-label]')?.getAttribute('aria-label') ?? null,
      text: item.innerText,
    }));
    return { transitions: capture.transitions, items };
  });
  const item = type => stream.items.find(entry => entry.type === type);
  for (const [type, expectedText, detailLabel] of [
    ['plan', 'SYNTHETIC PLAN: read the typed result; summarize its findings; preserve its source limitations.', 'Plan content'],
    ['tool call', 'SYNTHETIC TOOL EVENT EXAMPLE: public_market_read · EQUITY; not executed by runtime.', 'Tool call details'],
    ['tool result', 'SYNTHETIC TOOL RESULT EXAMPLE: no second research command; local fixture only.', 'Tool result details'],
  ]) {
    check(Boolean(item(type)), `The streamed ${type} item was not rendered: ${JSON.stringify(stream.items)}`);
    check(item(type).text.includes(expectedText), `The streamed ${type} item lost its bounded content.`);
    check(item(type).detailLabel === detailLabel, `${type} should expose its typed accessible label: ${JSON.stringify(item(type))}`);
    check(JSON.stringify(stream.transitions[type]) === JSON.stringify(['STARTED', 'STREAMING', 'COMPLETED']),
      `${type} lifecycle was not rendered as STARTED → STREAMING → COMPLETED: ${JSON.stringify(stream.transitions[type])}`);
  }
  const userMessage = item('user message');
  check(Boolean(userMessage) && userMessage.text.includes(request), 'The exact user request must appear in the timeline.');
  const researchResult = item('research result');
  check(Boolean(researchResult) && researchResult.text.includes('Synthetic fixture · SYNTHETIC_INTEGRATION_FIXTURE'),
    'The typed interim result must persist into the Turn timeline.');
  check(await page.getByText(/Read-only response for: Summarize the synthetic AAPL research result/, { exact: false }).isVisible(),
    'The completed response should remain visible alongside the plan and tool items.');

  const commandResult = await commandLog(commandCursor.cursor);
  const isForbidden = name => name.startsWith('provider.')
    || (name.startsWith('account.') && !['account.list', 'account.get', 'account.activity'].includes(name))
    || name.startsWith('approval.')
    || (name.startsWith('risk.') && !['risk.get_policy', 'risk.decision.list'].includes(name))
    || name.startsWith('model.')
    || name.startsWith('data.')
    || name.startsWith('market.')
    || name.startsWith('trade.')
    || name.startsWith('paper.order.')
    || name.startsWith('order.')
    || name.startsWith('gateway.')
    || name.startsWith('live.');
  const forbidden = commandResult.commands.filter(entry => isForbidden(entry.command));
  check(commandResult.commands.filter(entry => entry.command === 'research.run').length === 1,
    'Exactly one local typed research preview should run.');
  check(commandResult.commands.filter(entry => entry.command === 'turn.start').length === 1,
    'Exactly one Turn should start.');
  check(forbidden.length === 0, `The bounded B5 flow issued forbidden commands: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0 && consoleErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ pageErrors, consoleErrors })}`);

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      turn: document.querySelector('.turn-timeline')?.getBoundingClientRect().toJSON(),
    }));
    check(geometry.scroll <= geometry.viewport, `B5 page overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.turn && geometry.turn.left >= 0 && geometry.turn.right <= geometry.viewport,
      `B5 Turn timeline exceeds ${width}px viewport: ${JSON.stringify(geometry.turn)}`);
    responsive.push({ width, documentScrollWidth: geometry.scroll, turnWidth: geometry.turn.width });
    if (width === 390) {
      await page.locator('.turn-timeline').screenshot({
        path: 'docs/implementation/evidence/s33/b5-research-running-390.png',
        style: '.topbar, .skip-link { visibility: hidden !important; }',
      });
    }
  }
  return {
    result: 'PASS',
    userRequestVisible: true,
    planVisible: true,
    typedToolVisible: true,
    interimTypedResultVisible: true,
    itemLifecycle: ['STARTED', 'STREAMING', 'COMPLETED'],
    researchRunCommands: 1,
    turnStartCommands: 1,
    forbiddenCommands: forbidden.length,
    commandNames: [...new Set(commandResult.commands.map(entry => entry.command))].sort(),
    externalRequests: externalRequests.length,
    pageErrors: pageErrors.length,
    consoleErrors: consoleErrors.length,
    responsive,
    fixtureOnly: true,
  };
}
