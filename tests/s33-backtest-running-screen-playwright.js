// Run against the isolated Rust-backed integration server on port 1422.
// Covers H3 with a disposable workspace and a synthetic running backtest only.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  const command = async (name, payload) => {
    const requestId = `s33-backtest-running-${name}-${Date.now()}-${Math.random()}`;
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
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });

  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 backtest running ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-backtest-running-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const cursor = (await readLog(0)).cursor;

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Strategies', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
  const editor = page.getByRole('region', { name: 'Strategy editor', exact: true });
  await editor.getByRole('textbox', { name: 'Name', exact: true }).fill(`S33 running strategy ${stamp}`);
  await editor.getByRole('button', { name: 'Save version', exact: true }).press('Enter');
  await editor.getByText(/Selected .* · v1 · sha256:/).waitFor({ state: 'visible' });

  const backtest = page.getByRole('region', { name: 'Backtest', exact: true });
  const result = backtest.locator('.strategy-result');
  await backtest.getByLabel('Start date', { exact: true }).fill('2026-01-01T00:00');
  await backtest.getByLabel('End date', { exact: true }).fill('2026-01-02T00:00');
  await backtest.getByRole('combobox', { name: 'Backtest integration scenario', exact: true }).selectOption('CANCELLED');
  await backtest.getByRole('button', { name: 'Run backtest', exact: true }).press('Enter');
  await result.getByText('RUNNING', { exact: true }).waitFor({ state: 'visible' });
  check(await result.getAttribute('aria-live') === 'polite', 'H3 running updates use the polite live region.');
  const progress = backtest.getByRole('progressbar', { name: 'Backtest progress', exact: true });
  await progress.waitFor({ state: 'visible' });
  check(await progress.getAttribute('aria-valuenow') === null, 'H3 must not invent a numeric completion percentage.');
  check(await progress.getAttribute('aria-valuetext') === 'Running; completion progress is not estimated.',
    'H3 exposes the honest indeterminate progress state to assistive technology.');
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  const animatedIndicator = await progress.locator('span').evaluate(element => getComputedStyle(element).animationName);
  check(animatedIndicator === 'backtest-progress', `H3 progress animation is missing: ${animatedIndicator}.`);
  await page.emulateMedia({ reducedMotion: 'reduce' });
  const reducedIndicator = await progress.locator('span').evaluate(element => ({
    animationName: getComputedStyle(element).animationName,
    animationDuration: getComputedStyle(element).animationDuration,
  }));
  check(reducedIndicator.animationName === 'none' && reducedIndicator.animationDuration === '0s',
    `Reduced motion must stop H3 animation: ${JSON.stringify(reducedIndicator)}.`);
  await result.getByText('The historical simulation is running; completion progress is not estimated.', { exact: true })
    .waitFor({ state: 'visible' });
  const runText = await result.innerText();
  const runId = /Run ([0-9a-f-]{36})/i.exec(runText)?.[1];
  check(Boolean(runId), `H3 exposes the exact run ID with Running status: ${runText}`);
  check(runText.includes('Frozen configuration'), 'H3 retains the frozen run inputs while running.');
  const cancel = backtest.getByRole('button', { name: 'Cancel backtest', exact: true });
  check(await cancel.isEnabled(), 'H3 keeps cancellation available for the active run.');

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await result.scrollIntoViewIfNeeded();
    await result.getByText('RUNNING', { exact: true }).waitFor({ state: 'visible' });
    check((await result.innerText()).includes(`Run ${runId}`), `Run identity changed at ${width}px.`);
    check(await progress.isVisible(), `Indeterminate progress is not visible at ${width}px.`);
    check(await cancel.isEnabled(), `H3 cancel is disabled at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
    }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `H3 overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    if (width === 390) {
      await result.screenshot({ path: 'docs/implementation/evidence/s33/backtest-running-390.png',
        style: '.topbar { visibility: hidden !important; }' });
    }
    observations.push({ width, running: true, runIdVisible: true, frozenInputsVisible: true,
      cancelEnabled: true, noOverflow: true });
  }

  const commands = (await readLog(cursor)).commands.map(entry => entry.command);
  const forbidden = commands.filter(name => name.startsWith('provider.') || name.startsWith('alpaca.')
    || name.startsWith('binance.') || name.startsWith('bitget.') || name.startsWith('trading212.')
    || name.startsWith('account.refresh') || name.startsWith('model.verify') || name.startsWith('data.source.')
    || name.startsWith('trade.') || name.startsWith('live.gateway.') || name.startsWith('paper.order.')
    || name === 'strategy.run' || name.startsWith('strategy.cancel')
    || name === 'backtest.cancel');
  check(commands.filter(name => name === 'backtest.run').length === 1, 'H3 issues exactly one synthetic backtest run.');
  check(forbidden.length === 0, `H3 browsing issued a provider or financial command: ${JSON.stringify(forbidden)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return { result: 'PASS', screenId: 'H3', workspace: workspace.workspaceId, runId,
    widths: [1280, 768, 390], observations, backtestRuns: 1, backtestCancels: 0,
    providerFinancialCommands: forbidden.length, externalHttpRequests: externalRequests.length };
}
