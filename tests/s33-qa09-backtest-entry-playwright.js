// Build: cargo build --manifest-path src-tauri/Cargo.toml --features integration-test --bin tradex-ipc
// Start: env -u TRADEX_CODEX_APP_SERVER TRADEX_LIVE_APPROVAL_FIXTURE=1 node_modules/.bin/vite --host 127.0.0.1 --mode integration --port 1424
// Run: playwright-cli run-code --filename=tests/s33-qa09-backtest-entry-playwright.js
// Disposable Rust-backed workspace. Local Codex and historical backtest fixtures only.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  page.setDefaultNavigationTimeout(30000);
  const externalRequests = [], pageErrors = [], envelopes = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) externalRequests.push(url.origin);
    if (url.pathname === '/__integration/command' && request.method() === 'POST') envelopes.push(request.postDataJSON());
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-qa09-${Date.now()}-${Math.random()}`;
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
  const requests = name => envelopes.filter(item => item.command === name);
  const waitValue = async (locator, value) => {
    for (let attempt = 0; attempt < 400; attempt += 1) {
      if (await locator.inputValue() === value) return;
      await page.waitForTimeout(25);
    }
    throw new Error(`Input did not settle to ${value}.`);
  };
  const geometry = async (label, locator, screenshot) => {
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: 900 });
      await locator.scrollIntoViewIfNeeded();
      const size = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      check(size.scroll <= size.viewport, `${label} overflows at ${width}: ${JSON.stringify(size)}`);
      if (width === 390) await locator.screenshot({ path: `docs/implementation/evidence/s33/${screenshot}`,
        style: '.topbar { visibility: hidden !important; }' });
    }
    await page.setViewportSize({ width: 1280, height: 900 });
  };
  const configure = async (panel, scenario, cash = '125000') => {
    for (const [label, value] of [
      ['Instrument', 'equity:US:MSFT'], ['Dataset', 'historical:fixture'],
      ['Start date', '2026-01-01T00:00'], ['End date', '2026-01-02T00:00'],
      ['Starting cash', cash], ['Commission', '0.25'], ['Slippage', '5.5'], ['Portfolio seed', 'QA09 read-only seed'],
    ]) await panel.getByLabel(label, { exact: label !== 'Instrument' }).fill(value);
    await panel.getByRole('combobox', { name: 'Bar interval', exact: true }).selectOption('1d');
    await panel.getByRole('combobox', { name: 'Backtest integration scenario', exact: true }).selectOption(scenario);
  };
  const assertFrozen = async (panel, run) => {
    const frozen = panel.getByRole('region', { name: 'Frozen configuration', exact: true });
    const text = await frozen.innerText();
    for (const value of [run.strategyVersionId, run.strategyHash, run.instrumentId, run.datasetId,
      run.startAt, run.endAt, run.barInterval, run.startingCash, run.commission, run.slippage, run.portfolioSeed]) {
      check(text.includes(value), `Frozen inputs retain ${value}.`);
    }
    return frozen;
  };
  const waitRun = async (panel, state) => {
    const result = panel.locator('.strategy-result');
    await result.getByText(state, { exact: true }).waitFor({ state: 'visible' });
    const runId = /Run ([0-9a-f-]{36})/i.exec(await result.innerText())?.[1];
    check(Boolean(runId), `${state} has a distinct persisted run identity.`);
    return command('backtest.get', { workspaceId, runId });
  };
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1424/');
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true }).waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true }).waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const name = `S33 QA09 ${stamp}`;
  const path = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-qa09-${stamp}`;
  const workspace = await command('workspace.open', { name, baseCurrency: 'USD', path });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(name);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(path);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).click();
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).waitFor({ state: 'detached' });
  const nav = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await nav.getByRole('button', { name: 'Threads', exact: true }).click();
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  const newComposer = page.getByRole('region', { name: 'Start a thread', exact: true });
  await newComposer.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('BACKTEST');
  await newComposer.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('HISTORICAL_SIMULATION');
  let panel = newComposer.getByRole('region', { name: 'Backtest', exact: true });
  await panel.getByRole('heading', { name: 'Backtest', exact: true }).waitFor({ state: 'visible' });
  check(await panel.getByRole('button', { name: 'Run backtest', exact: true }).isDisabled(), 'No saved strategy blocks deterministic execution.');
  await newComposer.getByRole('button', { name: /@ Context/ }).press('Enter');
  const picker = page.getByRole('dialog', { name: 'Choose context', exact: true });
  await picker.getByRole('checkbox', { name: /equity:US:MSFT/ }).check();
  await picker.getByRole('button', { name: 'Attach', exact: true }).press('Enter');
  await waitValue(panel.getByLabel('Instrument', { exact: false }), 'equity:US:MSFT');
  check(await panel.getByLabel('Instrument', { exact: false }).inputValue() === 'equity:US:MSFT', 'Late context selection preserves MSFT in the mounted Backtest panel.');
  await newComposer.getByRole('button', { name: 'Open Strategies from BACKTEST context', exact: true }).press('Enter');
  await page.getByText('Opened from BACKTEST context', { exact: true }).waitFor({ state: 'visible' });
  const editor = page.getByRole('region', { name: 'Strategy editor', exact: true });
  const strategyName = `QA09 saved strategy ${stamp}`;
  await editor.getByLabel('Name', { exact: true }).fill(strategyName);
  await editor.getByRole('button', { name: 'Save version', exact: true }).press('Enter');
  await editor.getByText(/Selected .* · v1 · sha256:/).waitFor({ state: 'visible' });
  const library = await command('strategy.list', { workspaceId });
  const version = library.versions.find(item => item.definition.name === strategyName);
  check(Boolean(version), 'Explicit Save produces a selected immutable version.');
  panel = page.getByRole('region', { name: 'Backtest', exact: true });
  await waitValue(panel.getByLabel('Instrument', { exact: false }), 'equity:US:MSFT');
  check(await panel.getByLabel('Instrument', { exact: false }).inputValue() === 'equity:US:MSFT', 'Strategy editor receives the exact Backtest context.');

  await nav.getByRole('button', { name: 'Threads', exact: true }).click();
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  await newComposer.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('BACKTEST');
  await newComposer.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('HISTORICAL_SIMULATION');
  const threadTitle = `QA09 Backtest entry ${stamp}`;
  await newComposer.getByLabel('Thread title', { exact: true }).fill(threadTitle);
  await newComposer.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: threadTitle, exact: true }).waitFor({ state: 'visible' });
  const turnComposer = page.getByRole('region', { name: 'Ask this Thread', exact: true });
  await turnComposer.getByLabel('Turn request', { exact: true }).fill('Review a historical strategy configuration; do not place an order.');
  await turnComposer.getByRole('button', { name: 'Send', exact: true }).press('Enter');
  await page.getByRole('status', { name: 'Turn 1 status', exact: true }).getByText('COMPLETED', { exact: true }).waitFor({ state: 'visible' });
  check(requests('turn.start').length === 1 && requests('turn.start')[0].payload.agentMode === 'BACKTEST'
    && requests('turn.start')[0].payload.executionContext === 'HISTORICAL_SIMULATION', 'Backtest Send uses the historical agent route.');
  check(requests('backtest.run').length === 0 && requests('research.run').length === 0, 'Send does not substitute a research preview or execute an unconfigured backtest.');
  panel = turnComposer.getByRole('region', { name: 'Backtest', exact: true });
  await panel.getByRole('combobox', { name: 'Saved strategy version', exact: true }).selectOption(version.strategyVersionId);
  await configure(panel, 'CANCELLED');
  await panel.getByLabel('Instrument', { exact: false }).fill('');
  await panel.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
  await panel.getByText('Enter an instrument.', { exact: true }).waitFor({ state: 'visible' });
  check(await panel.getByLabel('Instrument', { exact: false }).getAttribute('aria-invalid') === 'true'
    && requests('backtest.run').length === 0, 'Invalid configuration is field-labelled and never dispatched.');
  await panel.getByLabel('Instrument', { exact: false }).fill('equity:US:MSFT');
  await panel.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
  const running = await waitRun(panel, 'RUNNING');
  await assertFrozen(panel, running);
  await panel.getByLabel('Starting cash', { exact: true }).fill('150000');
  await panel.getByLabel('Instrument', { exact: false }).fill('equity:US:AAPL');
  check(JSON.stringify(await command('backtest.get', { workspaceId, runId: running.runId })) === JSON.stringify(running), 'Editing future configuration cannot mutate the active frozen run.');
  await geometry('Running backtest', panel.locator('.strategy-result'), 'qa09-running-390.png');
  await panel.getByRole('button', { name: 'Cancel backtest', exact: true }).press('Enter');
  const cancelled = await waitRun(panel, 'CANCELLED');
  check(cancelled.runId === running.runId && cancelled.requestHash === running.requestHash && cancelled.startingCash === '125000', 'Explicit Cancel stops the same frozen run.');
  await configure(panel, 'DATA_GAP');
  await panel.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
  const failed = await waitRun(panel, 'FAILED');
  await assertFrozen(panel, failed);
  await panel.getByLabel('Starting cash', { exact: true }).fill('175000');
  await panel.getByRole('button', { name: 'Retry backtest', exact: true }).press('Enter');
  await page.waitForFunction(oldId => !document.querySelector('.turn-composer .strategy-result')?.textContent?.includes(oldId), failed.runId);
  const retry = await waitRun(panel, 'FAILED');
  check(retry.runId !== failed.runId && retry.requestHash === failed.requestHash && retry.startingCash === '125000', 'Retry has a distinct run ID and exact old frozen inputs, even after editable fields change.');
  await configure(panel, 'SUCCESS');
  await panel.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
  const threadRun = await waitRun(panel, 'COMPLETED');
  const threadSnapshot = JSON.stringify(threadRun);
  const frozen = await assertFrozen(panel, threadRun);
  await geometry('Completed Thread backtest', frozen, 'qa09-thread-frozen-390.png');
  await turnComposer.getByRole('button', { name: 'Open Strategies from BACKTEST context', exact: true }).press('Enter');
  // Saved library selection is public keyboard UI, not a fixture replacement of configuration.
  await page.getByRole('button', { name: new RegExp(`${strategyName}.*v1`), exact: false }).press('Enter');
  panel = page.getByRole('region', { name: 'Backtest', exact: true });
  await configure(panel, 'SUCCESS', '150000');
  await panel.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
  const editorRun = await waitRun(panel, 'COMPLETED');
  check(editorRun.runId !== threadRun.runId && editorRun.requestHash !== threadRun.requestHash
    && editorRun.strategyVersionId === threadRun.strategyVersionId, 'Both entries converge on version-bound configuration and distinct immutable runs.');
  check(JSON.stringify(await command('backtest.get', { workspaceId, runId: threadRun.runId })) === threadSnapshot, 'Later editor execution leaves the old completed result unchanged.');
  await panel.getByRole('combobox', { name: 'Compare left run', exact: true }).selectOption(threadRun.runId);
  await panel.getByRole('combobox', { name: 'Compare right run', exact: true }).selectOption(editorRun.runId);
  await panel.getByRole('button', { name: 'Compare runs', exact: true }).press('Enter');
  const comparison = panel.getByRole('region', { name: 'Backtest comparison', exact: true });
  await comparison.getByRole('heading', { name: 'Backtest comparison', exact: true }).waitFor({ state: 'visible' });
  const comparisonText = await comparison.innerText();
  check(comparisonText.includes(threadRun.runId) && comparisonText.includes(editorRun.runId)
    && comparisonText.includes('125000') && comparisonText.includes('150000'), 'Comparison preserves both exact run identities and capital difference.');
  await geometry('Comparison', comparison, 'qa09-comparison-390.png');

  await command('model.gateway.fixture', { workspaceId, status: 'STOPPED' });
  await page.reload();
  await nav.getByRole('button', { name: 'Threads', exact: true }).click();
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  await page.locator('.sidebar').getByRole('button', { name: new RegExp(threadTitle), exact: false }).press('Enter');
  await turnComposer.getByLabel('Turn request', { exact: true }).fill('Generate another strategy');
  await turnComposer.getByText('Model gateway is unavailable; the draft remains local until it is ready.', { exact: true }).waitFor({ state: 'visible' });
  check(await turnComposer.getByRole('button', { name: 'Send', exact: true }).isDisabled(), 'Unavailable model blocks strategy-generation Send.');
  panel = turnComposer.getByRole('region', { name: 'Backtest', exact: true });
  await panel.getByRole('combobox', { name: 'Saved strategy version', exact: true }).selectOption(version.strategyVersionId);
  await configure(panel, 'SUCCESS', '200000');
  await panel.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
  const offlineRun = await waitRun(panel, 'COMPLETED');
  check(offlineRun.startingCash === '200000' && offlineRun.runId !== editorRun.runId,
    'Configured deterministic backtest still runs with unavailable model.');
  await geometry('Unavailable model and deterministic result', turnComposer, 'qa09-model-unavailable-390.png');
  // Reproduce the stale previous-workspace runtime projection through a second explicit switch.
  await page.getByRole('button', { name: 'Workspace', exact: true }).click();
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).waitFor({ state: 'visible' });
  const nextName = `QA09 ready after stopped ${stamp}`;
  const nextPath = `${path}-ready`;
  const nextWorkspace = await command('workspace.open', { name: nextName, baseCurrency: 'USD', path: nextPath });
  await command('workspace.ready.fixture', { workspaceId: nextWorkspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(nextName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(nextPath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.locator('.runtime-summary').getByText('Model ready · CHATGPT', { exact: true }).waitFor({ state: 'visible', timeout: 2000 });
  await page.waitForFunction(() => [...document.querySelectorAll('button')].some(button => button.textContent === 'Complete onboarding' && !button.disabled), undefined, { timeout: 2000 });
  check(await page.getByRole('button', { name: 'Complete onboarding', exact: true }).isEnabled(),
    'A ready destination workspace replaces the stopped source runtime without waiting for the ten-second poll.');
  const names = envelopes.map(item => item.command);
  const forbidden = names.filter(name => /^(trade\.|paper\.order\.|live\.|order\.|alpaca\.|binance\.|bitget\.|trading212\.|market\.|research\.|strategy\.run)/.test(name)
    || (name.startsWith('provider.') && !['provider.list_definitions', 'provider.get_schema'].includes(name))
    || (name.startsWith('account.') && !['account.list', 'account.get', 'account.activity'].includes(name))
    || (name.startsWith('model.') && !['model.get', 'model.get_gateway', 'model.gateway.fixture'].includes(name))
    || (name.startsWith('data.') && /probe|refresh|configure|disconnect|cleanup/.test(name)));
  check(forbidden.length === 0, `Forbidden provider or financial commands: ${JSON.stringify(forbidden)}`);
  check(requests('backtest.run').length === 6 && requests('backtest.cancel').length === 1
    && requests('backtest.compare').length === 1 && requests('turn.start').length === 1, 'Only six explicit fixture runs, one cancel, one comparison and one synthetic Turn occur.');
  check(externalRequests.length === 0 && pageErrors.length === 0,
    `No external HTTP/page errors: ${JSON.stringify({ externalRequests, pageErrors })}`);
  return { result: 'PASS_LIMITED', backtestRuns: 6, cancels: 1, comparisons: 1, syntheticTurns: 1,
    frozenVersion: version.strategyVersionId, runIds: [running.runId, failed.runId, retry.runId, threadRun.runId, editorRun.runId, offlineRun.runId],
    responsiveWidths: [1280, 768, 390], workspaceRuntimeRefresh: true, forbiddenCommands: 0, externalRequests: 0, pageErrors: 0 };
}
