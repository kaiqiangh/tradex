// Run against the isolated Rust-backed integration server on port 1422.
// J3 coverage is read-only and uses one disposable, fixture-ready workspace.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-data-storage-${name}-${Date.now()}-${Math.random()}`;
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
  const isolationNotice = page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  );
  await isolationNotice.waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await isolationNotice.waitFor({ state: 'visible' });

  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 data storage ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-data-storage-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const cursor = (await readLog(0)).cursor;

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  const tabs = page.getByRole('group', { name: 'Settings sections', exact: true });
  await tabs.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Data & Storage', exact: true }).waitFor({ state: 'visible' });

  const archive = page.locator('.workspace-archive');
  await page.getByRole('heading', { name: 'Workspace backup & retention', exact: true }).waitFor({ state: 'visible' });
  const storageProfile = page.getByRole('list', { name: 'Storage responsibilities', exact: true });
  await storageProfile.waitFor({ state: 'visible' });
  const storageText = (await storageProfile.innerText()).replace(/\s+/g, ' ');
  for (const expected of [
    'SQLite — transactional/domain state and financial audit.',
    'DuckDB — persistent 1-minute+ OHLCV, analytics, and MVP backtest data.',
    'Filesystem — artifacts, strategies, exports, and backups.',
    'Parquet — optional Phase 2+ for large immutable historical datasets and interchange; not required for v1.0 MVP correctness.',
  ]) check(storageText.includes(expected), `Storage responsibility is missing: ${expected}`);

  const privacy = page.getByText('Privacy and restore safety', { exact: true });
  await privacy.waitFor({ state: 'visible' });
  const privacyText = await privacy.locator('..').innerText();
  check(privacyText.includes('Broker/API secrets, market.duckdb and cache files are excluded.'),
    'The archive privacy boundary is visible.');
  check(privacyText.includes('Restored account references stay unchecked, Live remains disarmed'),
    'Restore safety remains explicit.');

  const retentionInputs = [
    ['Ordinary artifacts (days)', 'No limit'],
    ['Market data (days)', 'No limit'],
    ['Cache files (days)', 'No limit'],
  ];
  for (const [name, placeholder] of retentionInputs) {
    const input = page.getByRole('spinbutton', { name, exact: true });
    await input.waitFor({ state: 'visible' });
    check(await input.inputValue() === '', `${name} should have no default retention limit.`);
    check(await input.getAttribute('placeholder') === placeholder, `${name} should disclose the blank/no-limit default.`);
  }
  await page.getByText('Saving never deletes data; Apply retention now performs the removal.', { exact: false }).waitFor({ state: 'visible' });
  await page.getByText('Artifacts linked to order IDs, unresolved executions, reservations, settlements, approvals and reconciliation records are kept.', { exact: false }).waitFor({ state: 'visible' });
  const saveRetention = page.getByRole('button', { name: 'Save retention settings', exact: true });
  const applyRetention = page.getByRole('button', { name: 'Apply retention now', exact: true });
  await saveRetention.waitFor({ state: 'visible' });
  await applyRetention.waitFor({ state: 'visible' });
  check(await applyRetention.isDisabled(), 'Apply retention remains disabled when every limit is unset.');

  const fileActions = ['Backup Now', 'Export Workspace', 'Import / Restore'];
  for (const name of fileActions) {
    const button = page.getByRole('button', { name, exact: true });
    await button.waitFor({ state: 'visible' });
    check(await button.isDisabled(), `${name} should remain disabled in the browser harness without native file dialogs.`);
  }
  await page.getByText('Backup, export and restore use native local-file dialogs in the TradeX desktop app.', { exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'Data sources', exact: true }).waitFor({ state: 'visible' });

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await storageProfile.scrollIntoViewIfNeeded();
    for (const element of [storageProfile, ...retentionInputs.map(([name]) => page.getByRole('spinbutton', { name, exact: true }))]) {
      await element.scrollIntoViewIfNeeded();
      const rect = await element.evaluate(node => {
        const box = node.getBoundingClientRect();
        return { left: box.left, right: box.right, width: box.width };
      });
      check(rect.width > 0 && rect.left >= -1 && rect.right <= width + 1,
        `Storage content is clipped at ${width}px: ${JSON.stringify(rect)}.`);
    }
    const geometry = await page.evaluate(() => {
      const root = document.querySelector('.workspace-archive');
      return {
        viewport: document.documentElement.clientWidth,
        documentScroll: document.documentElement.scrollWidth,
        archiveClient: root?.clientWidth ?? 0,
        archiveScroll: root?.scrollWidth ?? 0,
      };
    });
    check(geometry.documentScroll <= geometry.viewport && geometry.archiveScroll <= geometry.archiveClient + 1,
      `Data & Storage overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: 'docs/implementation/evidence/s33/data-storage-390.png', fullPage: true });
    }
    observations.push({ width, storageResponsibilities: 4, retentionFields: 3, archiveOverflow: false });
  }

  const commands = (await readLog(cursor)).commands.map(entry => entry.command);
  const allowedCommands = new Set([
    'account.activity', 'account.list', 'domain.snapshot', 'domain.subscribe', 'paper.get',
    'provider.list_definitions', 'risk.get_policy', 'runtime.status', 'workspace.retention.get',
    'data.source.catalog', 'data.source.connection', 'data.calendar.connection',
    'data.actions.connection', 'data.instrument.connection', 'data.fx.connection',
    'data.binance_rules.connection', 'data.binance_market.connection',
  ]);
  const unexpected = commands.filter(name => !allowedCommands.has(name));
  check(unexpected.length === 0, `J3 browsing issued a command outside its passive-read allowlist: ${JSON.stringify(unexpected)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}.`);
  return {
    result: 'PASS', screenId: 'J3', workspace: workspace.workspaceId, widths: [1280, 768, 390],
    observations, commands, allowedCommands: [...allowedCommands], unexpectedCommands: unexpected,
    externalHttpRequests: externalRequests.length, consoleErrors, pageErrors,
    retentionSaveCommands: commands.filter(name => name === 'workspace.retention.set').length,
    retentionApplyCommands: commands.filter(name => name === 'workspace.retention.apply').length,
    archiveCommands: commands.filter(name => name.startsWith('workspace.archive.')),
    providerMutationsOrProbes: commands.filter(name => /\.(configure|refresh|disconnect|probe)$/.test(name)
      || name === 'account.refresh' || name.startsWith('alpaca.') || name.startsWith('binance.')
      || name.startsWith('bitget.') || name.startsWith('trading212.')),
  };
}
