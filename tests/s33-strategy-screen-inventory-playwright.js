// Run against the isolated Rust-backed integration server on port 1422.
// Covers H1/H2 with a disposable workspace and local strategy persistence only.
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
    const requestId = `s33-strategy-inventory-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 strategy screens ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-strategy-screens-${stamp}`;
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
  const boundary = page.getByRole('region', { name: 'Blocked capabilities', exact: true });
  await editor.waitFor({ state: 'visible' });
  await savedVersions.waitFor({ state: 'visible' });
  await boundary.waitFor({ state: 'visible' });

  // H1 representative empty-library state and H2 editable sandbox surface.
  await savedVersions.getByText('No saved strategy versions.', { exact: true }).waitFor({ state: 'visible' });
  for (const name of ['Name', 'Language', 'Runtime', 'Source']) {
    await editor.getByRole('textbox', { name, exact: true }).waitFor({ state: 'visible' });
  }
  const run = page.getByRole('button', { name: 'Run selected version', exact: true });
  check(await run.isDisabled(), 'H2 cannot run an unsaved/unselected strategy version.');
  for (const capability of ['Broker credentials', 'OS keychain', 'Privileged Order Gateway', 'Unrestricted network']) {
    await boundary.getByText(capability, { exact: true }).waitFor({ state: 'visible' });
  }
  await boundary.getByText('Strategy output is a signal only. Trade, approval, reservation and provider actions are unavailable here.', { exact: true })
    .waitFor({ state: 'visible' });

  const strategyName = `S33 local strategy ${stamp}`;
  await editor.getByRole('textbox', { name: 'Name', exact: true }).fill(strategyName);
  await editor.getByRole('textbox', { name: 'Language', exact: true }).fill('python');
  await editor.getByRole('textbox', { name: 'Runtime', exact: true }).fill('sandbox-v1');
  await editor.getByRole('textbox', { name: 'Source', exact: true }).fill('signal = HOLD\nreturn signal');
  await editor.getByRole('button', { name: 'Save version', exact: true }).press('Enter');
  await editor.getByText(/Selected .* · v1 · sha256:/).waitFor({ state: 'visible' });
  const saved = savedVersions.getByRole('button').filter({ hasText: strategyName }).filter({ hasText: 'v1 · python' });
  await saved.waitFor({ state: 'visible' });
  check(await run.isEnabled(), 'Saving a local immutable version makes the selected signal-only run control available.');
  check(await savedVersions.getByText('Saved versions are immutable. Save again to create a new revision.', { exact: true }).isVisible(),
    'H1 discloses immutable saved versions.');

  const firstSource = 'signal = HOLD\nreturn signal';
  const secondSource = '# revision 2\nsignal = HOLD\nreturn signal';
  await editor.getByRole('textbox', { name: 'Source', exact: true }).fill(secondSource);
  await editor.getByRole('button', { name: 'Save version', exact: true }).press('Enter');
  await editor.getByText(/Selected .* · v2 · sha256:/).waitFor({ state: 'visible' });
  const firstRevision = savedVersions.getByRole('button').filter({ hasText: strategyName }).filter({ hasText: 'v1 · python' });
  const secondRevision = savedVersions.getByRole('button').filter({ hasText: strategyName }).filter({ hasText: 'v2 · python' });
  await firstRevision.waitFor({ state: 'visible' });
  await secondRevision.waitFor({ state: 'visible' });
  check(await secondRevision.getAttribute('aria-pressed') === 'true', 'The newest immutable revision is selected after saving.');
  const savedLibrary = await command('strategy.list', { workspaceId: workspace.workspaceId });
  const savedV2 = savedLibrary.versions.find(version => version.definition.name === strategyName && version.revision === 2);
  check(savedV2, 'The immutable v2 metadata is present in the workspace library.');
  const parameterVersion = await command('strategy.save_version', {
    workspaceId: workspace.workspaceId,
    strategyId: savedV2.strategyId,
    definition: { ...savedV2.definition, parameters: [{ name: 'window', value: '20' }] },
  });
  check(parameterVersion.revision === 3, 'A fixture-only parameterized v3 is stored as a new immutable revision.');
  await nav.getByRole('button', { name: 'Threads', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  await nav.getByRole('button', { name: 'Strategies', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
  const thirdRevision = savedVersions.getByRole('button').filter({ hasText: strategyName }).filter({ hasText: 'v3 · python' });
  await thirdRevision.waitFor({ state: 'visible' });
  check(await thirdRevision.getByText('Parameters: window=20', { exact: true }).isVisible(),
    'H1 renders the stored parameter names and values.');
  check(await firstRevision.getByText('Parameters: None', { exact: true }).isVisible(),
    'H1 clearly labels versions with no parameters.');

  const listedLibrary = await command('strategy.list', { workspaceId: workspace.workspaceId });
  const listedVersions = [1, 2, 3].map(revision => listedLibrary.versions
    .find(version => version.definition.name === strategyName && version.revision === revision));
  check(listedVersions.every(Boolean), 'All immutable revisions remain in the persisted strategy list.');
  for (const [index, row] of [firstRevision, secondRevision, thirdRevision].entries()) {
    const version = listedVersions[index];
    const displayedHash = (await row.innerText()).match(/sha256:[0-9a-f]{64}/)?.[0];
    check(displayedHash === version.sourceHash, `H1 shows the canonical hash for v${version.revision}.`);
    check(await row.locator('time').getAttribute('datetime') === version.createdAt,
      `H1 exposes the exact persisted creation time for v${version.revision}.`);
  }
  check(new Set(listedVersions.map(version => version.sourceHash)).size === 3,
    'H1 shows distinct canonical SHA-256 identities for all immutable revisions.');
  await firstRevision.press('Enter');
  await editor.getByText(/Selected .* · v1 · sha256:/).waitFor({ state: 'visible' });
  check(await firstRevision.getAttribute('aria-pressed') === 'true'
    && await secondRevision.getAttribute('aria-pressed') === 'false',
  'H1 reselects the historical v1 version without changing either immutable list entry.');
  check(await editor.getByRole('textbox', { name: 'Source', exact: true }).inputValue() === firstSource,
    'Selecting v1 restores its saved source instead of the newer draft.');

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await page.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
    await editor.getByRole('heading', { name: 'Strategy editor', exact: true }).waitFor({ state: 'visible' });
    await savedVersions.getByRole('heading', { name: 'Saved versions', exact: true }).waitFor({ state: 'visible' });
    await savedVersions.scrollIntoViewIfNeeded();
    for (const revision of [firstRevision, secondRevision, thirdRevision]) {
      const geometry = await revision.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { top: rect.top, bottom: rect.bottom, viewportHeight: window.innerHeight };
      });
      check(geometry.top >= 0 && geometry.bottom <= geometry.viewportHeight,
        `H1 list entry is fully visible at ${width}px: ${JSON.stringify(geometry)}.`);
    }
    await boundary.scrollIntoViewIfNeeded();
    const boundaryGeometry = await boundary.evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { top: rect.top, bottom: rect.bottom, viewportHeight: window.innerHeight };
    });
    check(boundaryGeometry.top >= 0 && boundaryGeometry.bottom <= boundaryGeometry.viewportHeight,
      `H1/H2 blocked capabilities are not fully in the viewport at ${width}px: ${JSON.stringify(boundaryGeometry)}.`);
    for (const capability of ['Broker credentials', 'OS keychain', 'Privileged Order Gateway', 'Unrestricted network']) {
      check(await boundary.getByText(capability, { exact: true }).isVisible(),
        `${capability} stays visibly blocked at ${width}px.`);
    }
    for (const name of ['Name', 'Language', 'Runtime', 'Source']) {
      check(await editor.getByRole('textbox', { name, exact: true }).isVisible(), `H2 ${name} is reachable at ${width}px.`);
    }
    const sourceGeometry = await editor.getByRole('textbox', { name: 'Source', exact: true }).evaluate(element => ({
      controlWidth: element.getBoundingClientRect().width,
      fieldWidth: element.closest('.field').getBoundingClientRect().width,
    }));
    check(sourceGeometry.controlWidth >= sourceGeometry.fieldWidth - 2,
      `H2 source editor fills its field at ${width}px: ${JSON.stringify(sourceGeometry)}.`);
    check(await saved.isVisible() && await saved.getAttribute('aria-pressed') === 'true',
      `H1 retains the saved and selected version at ${width}px.`);
    check(await boundary.getByText('Strategy output is a signal only. Trade, approval, reservation and provider actions are unavailable here.', { exact: true }).isVisible(),
      `The no-trading capability boundary remains visible at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      editor: document.querySelector('.strategy-editor')?.getBoundingClientRect().toJSON() ?? null,
      surface: document.querySelector('.strategy-surface')?.getBoundingClientRect().toJSON() ?? null,
    }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `Strategy inventory screens overflow at ${width}px: ${JSON.stringify(geometry)}.`);
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await boundary.screenshot({ path: 'docs/implementation/evidence/s33/strategy-boundary-390.png' });
      await savedVersions.screenshot({ path: 'docs/implementation/evidence/s33/strategy-list-390.png' });
      await page.locator('.strategy-grid').screenshot({
        path: 'docs/implementation/evidence/s33/strategy-inventory-390.png',
        style: '.topbar { visibility: hidden !important; }',
      });
    }
    observations.push({ screenIds: ['H1', 'H2'], width,
      savedVersionVisible: true, editorFieldsReachable: true, blockedCapabilitiesVisible: true,
      signalOnlyBoundaryVisible: true, historicalVersionReselected: true, noOverflow: true });
  }

  const finalLog = await readLog(cursor);
  const commands = finalLog.commands.map(entry => entry.command);
  const versionSaves = commands.filter(name => name === 'strategy.save_version');
  const forbidden = commands.filter(name => name.startsWith('provider.') || name.startsWith('alpaca.')
    || name.startsWith('binance.') || name.startsWith('bitget.') || name.startsWith('trading212.')
    || name.startsWith('account.refresh') || name.startsWith('model.verify') || name.startsWith('data.source.')
    || name.startsWith('trade.') || name.startsWith('live.gateway.') || name.startsWith('paper.order.')
    || name === 'strategy.run' || name.startsWith('strategy.cancel')
    || name.startsWith('backtest.run') || name.startsWith('backtest.cancel'));
  check(forbidden.length === 0, `H1/H2 browsing issued a provider or financial command: ${JSON.stringify(forbidden)}.`);
  check(versionSaves.length === 3, `The H1 list scenario saves two UI revisions and one parameterized list fixture: ${JSON.stringify(versionSaves)}.`);
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return { result: 'PASS', screenIds: ['H1', 'H2'], workspace: workspace.workspaceId,
    widths: [1280, 768, 390], emptyLibraryObservedAt: 1280, observations,
    strategyVersionSaves: versionSaves.length, uiVersionSaves: 2, parameterizedListFixtureSaves: 1,
    strategyRuns: 0, backtests: 0,
    providerFinancialCommands: forbidden.length, externalHttpRequests: externalRequests.length };
}
