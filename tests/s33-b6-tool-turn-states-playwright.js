// Run against the Rust-backed local integration bridge with
// TRADEX_FAKE_APP_SERVER_SCENARIO=tool-failure-once. The failure and recovery
// are local integration fixtures; no provider, model endpoint, or tool runs.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const pageErrors = [];
  const consoleErrors = [];
  page.on('request', request => {
    const url = request.url();
    const localOrigin = /^https?:\/\/(127\.0\.0\.1|localhost)(:|\/|$)/.test(url);
    if (/^https?:\/\//.test(url) && !localOrigin) externalRequests.push(url.split('/').slice(0, 3).join('/'));
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  const command = async (name, payload) => {
    const requestId = `s33-b6-${name}-${Date.now()}-${Math.random()}`;
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

  await page.addInitScript(() => window.localStorage.clear());
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1426/');
  const commandCursor = await commandLog(0);
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 B6 tool states ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-b6-${stamp}`;
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
  const threadTitle = `B6 tool states ${stamp}`;
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill(threadTitle);
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
  await page.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: threadTitle, exact: true }).waitFor({ state: 'visible' });

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
    const observer = new MutationObserver(sample);
    observer.observe(document.body, { attributes: true, childList: true, characterData: true, subtree: true });
    window.__s33B6Capture = { transitions, observer };
  });
  const request = 'Read the synthetic availability check and report any limitations.';
  await page.getByLabel('Turn request', { exact: true }).fill(request);
  await page.getByRole('button', { name: 'Send', exact: true }).press('Enter');
  const failedStatus = page.getByRole('status', { name: 'Turn 1 status', exact: true });
  await page.waitForFunction(() => document.querySelector('[role="status"][aria-label="Turn 1 status"]')?.textContent?.trim() === 'FAILED');
  check(await failedStatus.innerText() === 'FAILED', 'A failed read-only tool request must leave an explicit FAILED Turn.');
  const failedTool = page.locator('.timeline-item[data-item-status="FAILED"]').filter({ hasText: 'tool call' });
  check(await failedTool.getAttribute('data-item-status') === 'FAILED', 'The interrupted tool call must be marked FAILED.');
  check(await failedTool.innerText().then(text => text.includes('Synthetic read-only tool fixture')), 'The failed tool card must explain its local read-only fixture boundary.');
  check(await page.getByText('MODEL_UNAVAILABLE', { exact: false }).count() > 0, 'The recorded failure code must remain visible.');
  const recoveryPanel = page.getByRole('alert', { name: 'MODEL_UNAVAILABLE error recovery', exact: true });
  check(await recoveryPanel.isVisible(), 'A failed Turn must expose its canonical model recovery guidance.');
  check(await recoveryPanel.getByText('Next step: Review model', { exact: true }).isVisible(), 'MODEL_UNAVAILABLE must explain the next recovery action.');
  check(await page.getByRole('button', { name: 'Retry Turn 1', exact: true }).isVisible(), 'Failed Turns must offer a retry action.');

  await page.setViewportSize({ width: 390, height: 844 });
  const failedGeometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
  check(failedGeometry.scroll <= failedGeometry.viewport, `Failed Turn overflow at 390px: ${JSON.stringify(failedGeometry)}`);
  await page.locator('.turn-timeline').screenshot({
    path: 'docs/implementation/evidence/s33/b6-tool-turn-failed-390.png',
    style: '.topbar, .skip-link { visibility: hidden !important; }',
  });

  await page.setViewportSize({ width: 1280, height: 900 });
  const retryButton = page.getByRole('button', { name: 'Retry Turn 1', exact: true });
  await retryButton.press('Enter');
  await retryButton.getByText('Retrying…', { exact: true }).waitFor({ state: 'visible' });
  const retriedStatus = page.getByRole('status', { name: 'Turn 2 status', exact: true });
  await retriedStatus.waitFor({ state: 'visible' });
  await page.waitForFunction(() => document.querySelector('[role="status"][aria-label="Turn 2 status"]')?.textContent?.trim() === 'COMPLETED');
  check(await failedStatus.innerText() === 'FAILED', 'Retry must preserve the original failed Turn.');
  check(await retriedStatus.innerText() === 'COMPLETED', 'Retry must create a distinct Turn that can complete.');

  await page.getByLabel('Turn request', { exact: true }).fill('Cancel this synthetic read-only Turn while it is running.');
  await page.getByRole('button', { name: 'Send', exact: true }).press('Enter');
  const cancelButton = page.getByRole('button', { name: 'Cancel Turn 3', exact: true });
  await cancelButton.waitFor({ state: 'visible' });
  await cancelButton.press('Enter');
  const cancelledStatus = page.getByRole('status', { name: 'Turn 3 status', exact: true });
  await page.waitForFunction(() => document.querySelector('[role="status"][aria-label="Turn 3 status"]')?.textContent?.trim() === 'CANCELLED');
  check(await cancelledStatus.innerText() === 'CANCELLED', 'An active Turn must end in the distinct CANCELLED state.');
  check(await page.getByRole('button', { name: 'Retry Turn 3', exact: true }).isVisible(), 'Cancelled Turns must keep a retry action.');

  const stream = await page.evaluate(() => {
    const capture = window.__s33B6Capture;
    capture.observer.disconnect();
    return {
      transitions: capture.transitions,
      turns: [...document.querySelectorAll('.turn-timeline')].map(turn => ({
        status: turn.querySelector('.turn-heading [role="status"]')?.textContent?.trim(),
        text: turn.innerText,
      })),
    };
  });
  check(JSON.stringify(stream.transitions['tool call']) === JSON.stringify(['STARTED', 'STREAMING', 'FAILED']),
    `Failed tool lifecycle should be STARTED → STREAMING → FAILED: ${JSON.stringify(stream.transitions)}`);
  check(stream.turns.length === 3 && stream.turns[0].status === 'FAILED' && stream.turns[1].status === 'COMPLETED' && stream.turns[2].status === 'CANCELLED',
    `Retry and cancellation should preserve history and append distinct Turns: ${JSON.stringify(stream.turns)}`);

  const commandResult = await commandLog(commandCursor.cursor);
  const isForbidden = name => name.startsWith('provider.')
    || (name.startsWith('account.') && !['account.list', 'account.get', 'account.activity'].includes(name))
    || name.startsWith('approval.') || name.startsWith('risk.') || name.startsWith('model.')
    || name.startsWith('data.') || name.startsWith('market.') || name.startsWith('trade.')
    || name.startsWith('paper.order.') || name.startsWith('order.') || name.startsWith('gateway.')
    || name.startsWith('live.');
  const forbidden = commandResult.commands.filter(entry => isForbidden(entry.command));
  check(commandResult.commands.filter(entry => entry.command === 'turn.start').length === 2, 'The test should start two Turns.');
  check(commandResult.commands.filter(entry => entry.command === 'turn.retry').length === 1, 'The test should retry once.');
  check(commandResult.commands.filter(entry => entry.command === 'turn.cancel').length === 1, 'The test should cancel one running Turn.');
  check(forbidden.length === 0, `The bounded B6 flow issued forbidden commands: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0 && consoleErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ pageErrors, consoleErrors })}`);

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    check(geometry.scroll <= geometry.viewport, `B6 page overflow at ${width}px: ${JSON.stringify(geometry)}`);
    responsive.push({ width, documentScrollWidth: geometry.scroll });
  }
  return {
    result: 'PASS',
    toolLifecycle: stream.transitions['tool call'],
    turnStates: stream.turns.map(turn => turn.status),
    retryCreatesNewTurn: true,
    failedTurnRetained: true,
    cancelledTurnVisible: true,
    failedItemBoundary: 'synthetic read-only tool; no financial-state mutation',
    turnStartCommands: 2,
    turnRetryCommands: 1,
    turnCancelCommands: 1,
    forbiddenCommands: forbidden.length,
    commandNames: [...new Set(commandResult.commands.map(entry => entry.command))].sort(),
    externalRequests: externalRequests.length,
    pageErrors: pageErrors.length,
    consoleErrors: consoleErrors.length,
    responsive,
    fixtureOnly: true,
  };
}
