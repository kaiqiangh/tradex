// Run against the isolated Rust-backed integration server on port 1422.
// Covers H2 parameter editing, immutable versions, and signal-only run recovery.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const consoleErrors = [];
  const directCommands = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => consoleErrors.push(error.message));
  const command = async (name, payload) => {
    directCommands.push(name);
    const requestId = `s33-strategy-sandbox-${name}-${Date.now()}-${Math.random()}`;
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
  const readLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1422/');
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 strategy sandbox ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-strategy-sandbox-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  let cursor = (await readLog(0)).cursor;

  const nav = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await nav.getByRole('button', { name: 'Strategies', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
  const editor = page.getByRole('region', { name: 'Strategy editor', exact: true });
  const savedVersions = page.getByRole('region', { name: 'Saved versions', exact: true });
  const runPanel = page.getByRole('region', { name: 'Run signal-only strategy', exact: true });
  const parameters = editor.locator('fieldset.strategy-parameters');
  const save = editor.getByRole('button', { name: 'Save version', exact: true });
  const name = `S33 parameter strategy ${stamp}`;
  await editor.getByRole('textbox', { name: 'Name', exact: true }).fill(name);
  await editor.getByRole('textbox', { name: 'Source', exact: true }).fill('signal = HOLD\nreturn signal');

  await parameters.getByRole('button', { name: 'Add parameter', exact: true }).press('Enter');
  check(await save.isDisabled(), 'A blank parameter name keeps Save version disabled.');
  await parameters.getByRole('alert').getByText(/Each parameter needs a name/).waitFor({ state: 'visible' });
  await parameters.getByRole('textbox', { name: 'Parameter name 1', exact: true }).fill('window');
  await parameters.getByRole('textbox', { name: 'Parameter value 1', exact: true }).fill('20');
  check(await save.isEnabled(), 'A bounded named parameter allows saving.');
  await parameters.getByRole('button', { name: 'Add parameter', exact: true }).press('Enter');
  await parameters.getByRole('textbox', { name: 'Parameter name 2', exact: true }).fill('window');
  await parameters.getByRole('textbox', { name: 'Parameter value 2', exact: true }).fill('5');
  check(await save.isDisabled(), 'Duplicate trimmed parameter names keep Save version disabled.');
  await parameters.getByRole('alert').getByText('Parameter names must be unique.', { exact: true }).waitFor({ state: 'visible' });
  check(await parameters.getByRole('textbox', { name: 'Parameter name 2', exact: true }).getAttribute('aria-invalid') === 'true',
    'The duplicate parameter input is associated with the accessible validation error.');
  await parameters.getByRole('button', { name: 'Remove parameter 2', exact: true }).press('Enter');
  check(await save.isEnabled(), 'Removing the duplicate parameter clears validation.');
  await save.press('Enter');
  await editor.getByText(/Selected .* · v1 · sha256:/).waitFor({ state: 'visible' });
  const firstRevision = savedVersions.getByRole('button').filter({ hasText: name }).filter({ hasText: 'v1 · python' });
  await firstRevision.waitFor({ state: 'visible' });
  check((await firstRevision.innerText()).includes('Parameters: window=20'), 'H2 stores the entered parameter in v1.');

  await editor.getByRole('textbox', { name: 'Parameter value 1', exact: true }).fill('50');
  await save.press('Enter');
  await editor.getByText(/Selected .* · v2 · sha256:/).waitFor({ state: 'visible' });
  const secondRevision = savedVersions.getByRole('button').filter({ hasText: name }).filter({ hasText: 'v2 · python' });
  await secondRevision.waitFor({ state: 'visible' });
  check((await secondRevision.innerText()).includes('Parameters: window=50'), 'Saving an edit creates immutable v2 with its own parameter value.');
  await firstRevision.press('Enter');
  check(await editor.getByRole('textbox', { name: 'Parameter value 1', exact: true }).inputValue() === '20',
    'Selecting historical v1 restores its frozen parameter values.');
  await secondRevision.press('Enter');
  check(await editor.getByRole('textbox', { name: 'Parameter value 1', exact: true }).inputValue() === '50',
    'Reselecting v2 restores its own frozen parameter values.');

  const runButton = runPanel.getByRole('button', { name: 'Run selected version', exact: true });
  const cancelButton = runPanel.getByRole('button', { name: 'Cancel run', exact: true });
  const retryButton = runPanel.getByRole('button', { name: 'Retry run', exact: true });
  const scenario = runPanel.getByRole('combobox', { name: 'Integration scenario', exact: true });
  const result = runPanel.locator('.strategy-result');
  const announcement = result.getByRole('status');
  await scenario.selectOption('SUCCESS');
  await runButton.press('Enter');
  await announcement.getByText('COMPLETED', { exact: true }).waitFor({ state: 'visible' });
  await result.getByText('HOLD', { exact: true }).waitFor({ state: 'visible' });
  check(await retryButton.isDisabled(), 'Retry is disabled when a run completed successfully.');
  check(await announcement.getAttribute('aria-live') === 'polite',
    'Only the concise state and run ID are announced through the polite status region.');
  const successRunId = await result.locator('span').innerText().then(text => text.replace(/^Run\s+/, ''));
  const successRun = await command('strategy.get_run', { workspaceId: workspace.workspaceId, runId: successRunId });
  check(successRun.signal?.direction === 'HOLD' && successRun.parameters?.[0]?.name === 'window'
    && successRun.parameters[0].value === '50', 'The signal-only run freezes the selected version parameters.');
  check(successRun.strategyVersionId && /^sha256:[0-9a-f]{64}$/.test(successRun.strategyHash)
    && /^sha256:[0-9a-f]{64}$/.test(successRun.requestHash), 'The run carries stable strategy and request identities.');
  check(successRun.contextRefs?.some(reference => reference.kind === 'instrument'
    && reference.id === successRun.instrumentId && /^sha256:[0-9a-f]{64}$/.test(reference.hash)),
  'The Control Plane adds and persists the canonical selected-instrument context reference.');
  for (const label of ['Strategy version', 'Strategy hash', 'Request hash', 'Parameters', 'Context references']) {
    await result.getByText(label, { exact: true }).waitFor({ state: 'visible' });
  }
  await result.getByText('Integration fixture: TRADEX_STRATEGY_FIXTURE', { exact: true }).waitFor({ state: 'visible' });
  check(await result.getByText(/window=50/, { exact: false }).isVisible(), 'The result announces the frozen parameter value.');
  check(await runPanel.getByRole('button', { name: 'Trade', exact: true }).count() === 0,
    'The strategy surface exposes no direct trade action.');

  await scenario.selectOption('FAILURE');
  await runButton.press('Enter');
  await announcement.getByText('FAILED', { exact: true }).waitFor({ state: 'visible' });
  check(await retryButton.isEnabled(), 'A failed run enables its retry action.');
  await result.getByText('STRATEGY_FIXTURE_FAILED: The integration strategy fixture returned a typed failure.', { exact: true })
    .waitFor({ state: 'visible' });
  await result.getByText('retry_strategy_run', { exact: true }).waitFor({ state: 'visible' });
  const failedRunId = await result.locator('span').innerText().then(text => text.replace(/^Run\s+/, ''));
  const failedRun = await command('strategy.get_run', { workspaceId: workspace.workspaceId, runId: failedRunId });
  check(failedRun.parameters?.[0]?.value === '50' && failedRun.strategyVersionId === successRun.strategyVersionId,
    'The typed failure retains the same version and frozen parameters.');
  await retryButton.press('Enter');
  await page.waitForFunction(previousId => {
    const resultNode = document.querySelector('.strategy-result');
    return resultNode && !resultNode.textContent?.includes(`Run ${previousId}`);
  }, failedRunId);
  await announcement.getByText('FAILED', { exact: true }).waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Retry run',
    'Keyboard retry returns focus to Retry run after completion.');
  const retriedRunId = await result.locator('span').innerText().then(text => text.replace(/^Run\s+/, ''));
  const retriedRun = await command('strategy.get_run', { workspaceId: workspace.workspaceId, runId: retriedRunId });
  check(retriedRunId !== failedRunId && retriedRun.requestHash === failedRun.requestHash
    && JSON.stringify(retriedRun.parameters) === JSON.stringify(failedRun.parameters),
  'Retry creates a new run identity with the same frozen request hash and parameters.');

  await scenario.selectOption('CANCELLED');
  await runButton.press('Enter');
  await page.waitForFunction(() => {
    const button = [...document.querySelectorAll('button')].find(item => item.textContent?.trim() === 'Cancel run');
    return button && !button.disabled;
  });
  await cancelButton.press('Enter');
  await announcement.getByText('CANCELLED', { exact: true }).waitFor({ state: 'visible' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Retry run',
    'Cancelling an active run moves keyboard focus to the available recovery action.');
  check(await retryButton.isEnabled(), 'A cancelled run offers retry with the frozen request.');
  const cancelledRunId = await result.locator('span').innerText().then(text => text.replace(/^Run\s+/, ''));
  const cancelledRun = await command('strategy.get_run', { workspaceId: workspace.workspaceId, runId: cancelledRunId });
  check(cancelledRun.parameters?.[0]?.value === '50' && cancelledRun.contextRefs?.length === successRun.contextRefs.length,
    'Cancellation preserves the selected version parameters and context references.');

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await page.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      editor: document.querySelector('.strategy-grid')?.getBoundingClientRect().toJSON() ?? null,
      result: document.querySelector('.strategy-result')?.getBoundingClientRect().toJSON() ?? null,
    }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `H2 overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    check(await parameters.getByRole('textbox', { name: 'Parameter name 1', exact: true }).isVisible()
      && await parameters.getByRole('textbox', { name: 'Parameter value 1', exact: true }).isVisible(),
    `H2 parameter editor remains visible at ${width}px.`);
    check(await result.getByText('Context references', { exact: true }).isVisible(),
      `The run identity remains visible at ${width}px.`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.locator('.strategy-surface').screenshot({ path: 'docs/implementation/evidence/s33/strategy-sandbox-390.png' });
    }
    observations.push({ screenId: 'H2', width, noOverflow: true, parameterEditorVisible: true, frozenRunIdentityVisible: true });
  }

  const finalLog = await readLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const forbidden = commands.filter(command => command.startsWith('provider.') || command.startsWith('alpaca.')
    || command.startsWith('binance.') || command.startsWith('bitget.') || command.startsWith('trading212.')
    || command.startsWith('account.refresh') || command.startsWith('model.verify') || command.startsWith('data.source.')
    || command.startsWith('trade.') || command.startsWith('live.gateway.') || command.startsWith('paper.order.')
    || command.startsWith('backtest.run') || command.startsWith('backtest.cancel'));
  check(commands.filter(command => command === 'strategy.save_version').length === 2,
    'H2 creates exactly two immutable versions through the editor.');
  check(commands.filter(command => command === 'strategy.run').length === 4
    && commands.filter(command => command === 'strategy.cancel').length === 1,
  'H2 runs success, failure, retry, and cancellation scenarios once each.');
  check(forbidden.length === 0, `H2 issued a provider or financial command: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0, `Browser reported console errors: ${JSON.stringify(consoleErrors)}.`);
  return { result: 'PASS', screenIds: ['H2'], workspace: workspace.workspaceId,
    widths: [1280, 768, 390], observations,
    parameterValidation: true, immutableVersions: 2, parameterizedSignalRun: true,
    failureAndRetrySameRequestHash: true, cancellationAndFocusRecovery: true,
    strategyRuns: 4, strategyCancellations: 1, providerFinancialCommands: forbidden.length,
    directStrategyRunReads: directCommands.filter(command => command === 'strategy.get_run').length,
    totalStrategyRunReads: commands.filter(command => command === 'strategy.get_run').length,
    externalHttpRequests: externalRequests.length, consoleErrors: consoleErrors.length };
}
