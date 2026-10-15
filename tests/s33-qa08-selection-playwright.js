// Build: cargo build --manifest-path src-tauri/Cargo.toml --features integration-test --bin tradex-ipc
// Start: TRADEX_LIVE_APPROVAL_FIXTURE=1 node_modules/.bin/vite --host 127.0.0.1 --mode integration --port 1424
// Run: playwright-cli run-code --filename=tests/s33-qa08-selection-playwright.js
// Disposable workspace; canonical metadata and synthetic screener/market results only.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  page.setDefaultNavigationTimeout(30000);
  const externalRequests = [];
  const pageErrors = [];
  const envelopes = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) externalRequests.push(url.origin);
    if (url.pathname === '/__integration/command' && request.method() === 'POST') envelopes.push(request.postDataJSON());
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-qa08-${Date.now()}-${Math.random()}`;
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
  const geometry = async (label, selector) => {
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
      const size = await page.evaluate(selector => {
        const node = document.querySelector(selector)?.getBoundingClientRect();
        return { width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
          viewportHeight: window.innerHeight,
          node: node ? { left: node.left, right: node.right, top: node.top, bottom: node.bottom } : null };
      }, selector);
      check(size.scroll <= size.width && size.node && size.node.left >= 0 && size.node.right <= size.width,
        `${label} stays within ${width}px: ${JSON.stringify(size)}`);
      if (label.toLowerCase().includes('picker')) {
        check(size.node.top >= 0 && size.node.bottom <= size.viewportHeight,
          `${label} and its actions fit within the ${width}px viewport height: ${JSON.stringify(size)}`);
      }
    }
  };
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1424/');
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true }).waitFor({ state: 'visible' });
  const cursor = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json().then(log => log.cursor));
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const name = `S33 QA08 ${stamp}`;
  const path = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-qa08-${stamp}`;
  const workspace = await command('workspace.open', { name, baseCurrency: 'USD', path });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(name);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(path);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).click();
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).waitFor({ state: 'detached' });
  const nav = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await nav.getByRole('button', { name: 'Threads', exact: true }).click();
  const composer = page.getByRole('region', { name: 'Start a thread', exact: true });
  const picker = composer.getByRole('button', { name: /@ Context/ });
  const catalog = await command('context.catalog', { workspaceId });
  const microsoft = catalog.entries.find(entry => entry.contextRef.id === 'equity:US:MSFT');
  const ethereum = catalog.entries.find(entry => entry.contextRef.id === 'crypto:ETH/USDT:spot');
  check(microsoft?.available && ethereum?.available && microsoft.readOnly && ethereum.readOnly,
    'Canonical catalog instrument references are selectable read-only metadata.');
  await picker.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Choose context', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const msftCheckbox = dialog.getByRole('checkbox', { name: /equity:US:MSFT/ });
  const ethCheckbox = dialog.getByRole('checkbox', { name: /crypto:ETH\/USDT:spot/ });
  await msftCheckbox.check();
  await ethCheckbox.check();
  await ethCheckbox.uncheck();
  await dialog.getByRole('button', { name: 'Attach', exact: true }).press('Enter');
  await composer.getByText('Context references: 1', { exact: true }).waitFor({ state: 'visible' });
  check((await composer.locator('.context-chips').innerText()).includes('equity:US:MSFT')
    && !(await composer.locator('.context-chips').innerText()).includes('crypto:ETH/USDT:spot'),
  'Attach preserves only the checked canonical instrument.');
  await picker.press('Enter');
  await ethCheckbox.check();
  await msftCheckbox.uncheck();
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).press('Enter');
  check((await composer.locator('.context-chips').innerText()).includes('equity:US:MSFT'), 'Cancel discards temporary changes.');
  await picker.press('Enter');
  check(await msftCheckbox.isChecked() && !(await ethCheckbox.isChecked()), 'Reopening restores the committed selection.');
  await geometry('Context picker', '.picker-dialog');
  await dialog.screenshot({ path: 'docs/implementation/evidence/s33/qa08-context-picker-390.png' });
  await ethCheckbox.check();
  await dialog.getByRole('button', { name: 'Attach', exact: true }).press('Enter');
  await composer.getByRole('button', { name: `Remove ${microsoft.label}`, exact: true }).click();
  const threadTitle = `QA08 exact picker ${stamp}`;
  await composer.getByRole('textbox', { name: 'Thread title', exact: true }).fill(threadTitle);
  await composer.getByRole('button', { name: 'Create Thread', exact: true }).click();
  await page.getByRole('heading', { name: threadTitle, exact: true }).waitFor({ state: 'visible' });
  const threads = await command('thread.list', { workspaceId });
  const created = await command('thread.get', { workspaceId, threadId: threads.threads.find(thread => thread.title === threadTitle).threadId });
  check(JSON.stringify(created.linkedContexts) === JSON.stringify([ethereum.contextRef]), 'Thread persists only the remaining exact canonical ID/hash.');
  check(created.turns.length === 0, 'Context selection does not run a model or Turn.');

  await page.setViewportSize({ width: 1280, height: 900 });
  await nav.getByRole('button', { name: 'Markets', exact: true }).click();
  const search = page.getByRole('textbox', { name: 'Search instruments', exact: true });
  await search.fill('MSFT');
  await page.getByRole('button', { name: 'Search', exact: true }).press('Enter');
  const msftRow = page.locator('.market-row').filter({ hasText: 'equity:US:MSFT' });
  await msftRow.waitFor({ state: 'visible' });
  await msftRow.press('Enter');
  await page.getByRole('heading', { name: 'Microsoft Corporation', exact: true }).waitFor({ state: 'visible' });
  check((await page.locator('.market-detail').innerText()).includes('equity:US:MSFT'), 'Market detail retains the selected non-AAPL identity.');
  await geometry('MSFT detail', '.market-detail');
  await page.screenshot({ path: 'docs/implementation/evidence/s33/qa08-msft-detail-390.png' });
  for (const unsupported of ['NVDA', 'AMD']) {
    await search.fill(unsupported);
    await page.getByRole('button', { name: 'Search', exact: true }).press('Enter');
    await page.getByText('No canonical instruments match this search.', { exact: true }).waitFor({ state: 'visible' });
    check(await page.locator('.market-row').count() === 0 && await page.locator('.market-detail').count() === 0,
      `${unsupported} is explicitly absent from this bounded catalog, never mapped to AAPL.`);
  }
  await page.getByRole('button', { name: 'Open screener', exact: true }).press('Enter');
  await page.getByRole('button', { name: 'Parse conditions', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'FilterSpec and RankSpec', exact: true }).waitFor({ state: 'visible' });
  const threshold = page.getByLabel('Condition 1 threshold', { exact: true });
  const run = page.getByRole('button', { name: 'Run screen', exact: true });
  const waitForRun = async () => page.waitForFunction(() => [...document.querySelectorAll('button')].some(button => button.textContent === 'Run screen' && !button.disabled));
  await waitForRun();
  await run.press('Enter');
  await page.getByRole('heading', { name: 'COMPLETED', exact: true }).waitFor({ state: 'visible' });
  check(await page.locator('.screener-candidate').count() === 2, 'The initial 15% fixture threshold yields two candidates.');
  await threshold.fill('0.25');
  check(await run.isDisabled() && await page.locator('.screener-candidate').count() === 0, 'Structured edits invalidate Run and prior results.');
  await page.getByRole('button', { name: 'Recalculate revision', exact: true }).press('Enter');
  await waitForRun();
  await run.press('Enter');
  await page.getByRole('heading', { name: 'EMPTY', exact: true }).waitFor({ state: 'visible' });
  check(await page.locator('.screener-candidate').count() === 0 && (await page.locator('.screener-results').innerText()).includes('0.25'),
    'The displayed 25% threshold drives an empty fixture result and its applied conditions.');
  await geometry('Empty screen', '.screener-results');
  await page.screenshot({ path: 'docs/implementation/evidence/s33/qa08-empty-390.png' });
  await threshold.fill('0.15');
  await page.getByRole('button', { name: 'Recalculate revision', exact: true }).press('Enter');
  await waitForRun();
  await run.press('Enter');
  await page.getByRole('heading', { name: 'COMPLETED', exact: true }).waitFor({ state: 'visible' });
  await geometry('Completed screen', '.screener-results');
  await page.getByLabel('Select equity:US:MSFT', { exact: true }).check();
  check(!(await page.getByLabel('Select equity:US:AAPL', { exact: true }).isChecked()), 'The other candidate stays unchecked.');
  await page.screenshot({ path: 'docs/implementation/evidence/s33/qa08-selected-candidate-390.png' });
  await page.getByRole('combobox', { name: 'Attach selected to', exact: true }).selectOption('new');
  await page.getByRole('button', { name: 'Attach selected (1)', exact: true }).press('Enter');
  const screenerTitle = `QA08 exact screen ${stamp}`;
  await composer.getByRole('textbox', { name: 'Thread title', exact: true }).fill(screenerTitle);
  await composer.getByRole('button', { name: 'Create Thread', exact: true }).click();
  await page.getByRole('heading', { name: screenerTitle, exact: true }).waitFor({ state: 'visible' });
  const finalThreads = await command('thread.list', { workspaceId });
  const attached = await command('thread.get', { workspaceId, threadId: finalThreads.threads.find(thread => thread.title === screenerTitle).threadId });
  check(JSON.stringify(attached.linkedContexts) === JSON.stringify([microsoft.contextRef]), 'Screener attaches only selected MSFT with the canonical workspace hash.');
  for (let version = 1; version <= 252; version += 1) {
    await command('strategy.save_version', { workspaceId, definition: {
      name: `Catalog boundary ${version}`, source: 'signal = HOLD\nreturn signal',
      language: 'python', runtime: 'sandbox-v1', parameters: [],
    } });
  }
  const bounded = await command('context.catalog', { workspaceId });
  check(bounded.entries.length === 256
    && bounded.entries.filter(entry => entry.contextRef.kind === 'strategy').length === 252
    && bounded.emptyStates.some(state => state.kind === 'instrument' && state.availabilityReason.includes('Context catalog limit (256)')),
  'Combined account/strategy/instrument metadata fits the wire bound and explicitly discloses omitted references.');
  await page.setViewportSize({ width: 1280, height: 900 });
  await nav.getByRole('button', { name: 'Settings', exact: true }).click();
  await nav.getByRole('button', { name: 'Threads', exact: true }).click();
  await picker.press('Enter');
  const capacityNotice = dialog.getByText(/Context catalog limit \(256\)/);
  await capacityNotice.waitFor({ state: 'visible' });
  check(await dialog.getByRole('checkbox').count() === 256, 'The schema-decoded bounded Picker remains usable, including existing strategy choices.');
  const lastOption = dialog.getByRole('checkbox').last();
  await lastOption.scrollIntoViewIfNeeded();
  const capacityGeometry = await page.evaluate(() => {
    const dialog = document.querySelector('.picker-dialog');
    const options = document.querySelector('.context-options');
    const actions = document.querySelector('.picker-dialog-actions');
    const inputs = options?.querySelectorAll('input[type="checkbox"]');
    const last = inputs?.[inputs.length - 1];
    const dialogRect = dialog?.getBoundingClientRect();
    const optionsRect = options?.getBoundingClientRect();
    const actionsRect = actions?.getBoundingClientRect();
    const lastRect = last?.getBoundingClientRect();
    return { viewportHeight: window.innerHeight,
      dialog: dialogRect && { top: dialogRect.top, bottom: dialogRect.bottom },
      options: options && { top: optionsRect?.top, bottom: optionsRect?.bottom,
        scrollTop: options.scrollTop, scrollHeight: options.scrollHeight, clientHeight: options.clientHeight },
      actions: actionsRect && { top: actionsRect.top, bottom: actionsRect.bottom },
      last: lastRect && { top: lastRect.top, bottom: lastRect.bottom } };
  });
  check(capacityGeometry.dialog && capacityGeometry.dialog.top >= 0
    && capacityGeometry.dialog.bottom <= capacityGeometry.viewportHeight,
  `The capacity-limited dialog stays within the viewport: ${JSON.stringify(capacityGeometry)}`);
  check(capacityGeometry.options.scrollHeight > capacityGeometry.options.clientHeight
    && capacityGeometry.options.scrollTop > 0,
  `The 256-item list scrolls inside the dialog: ${JSON.stringify(capacityGeometry.options)}`);
  check(capacityGeometry.last.top >= capacityGeometry.options.top
    && capacityGeometry.last.bottom <= capacityGeometry.options.bottom
    && capacityGeometry.actions.top >= capacityGeometry.dialog.top
    && capacityGeometry.actions.bottom <= capacityGeometry.dialog.bottom
    && await capacityNotice.isVisible(),
  `The last context and catalog notice remain reachable while actions stay in view: ${JSON.stringify(capacityGeometry)}`);
  await geometry('Capacity-limited picker', '.picker-dialog');
  await lastOption.scrollIntoViewIfNeeded();
  const mobileCapacityGeometry = await page.evaluate(() => {
    const dialog = document.querySelector('.picker-dialog');
    const options = document.querySelector('.context-options');
    const actions = document.querySelector('.picker-dialog-actions');
    const inputs = options?.querySelectorAll('input[type="checkbox"]');
    const last = inputs?.[inputs.length - 1];
    const d = dialog?.getBoundingClientRect();
    const o = options?.getBoundingClientRect();
    const a = actions?.getBoundingClientRect();
    const l = last?.getBoundingClientRect();
    return { viewportHeight: window.innerHeight,
      dialog: d && { top: d.top, bottom: d.bottom },
      options: options && { top: o?.top, bottom: o?.bottom,
        scrollTop: options.scrollTop, scrollHeight: options.scrollHeight, clientHeight: options.clientHeight },
      actions: a && { top: a.top, bottom: a.bottom },
      last: l && { top: l.top, bottom: l.bottom } };
  });
  check(mobileCapacityGeometry.dialog && mobileCapacityGeometry.dialog.top >= 0
    && mobileCapacityGeometry.dialog.bottom <= mobileCapacityGeometry.viewportHeight
    && mobileCapacityGeometry.options.scrollHeight > mobileCapacityGeometry.options.clientHeight
    && mobileCapacityGeometry.options.scrollTop > 0
    && mobileCapacityGeometry.last.top >= mobileCapacityGeometry.options.top
    && mobileCapacityGeometry.last.bottom <= mobileCapacityGeometry.options.bottom
    && mobileCapacityGeometry.actions.bottom <= mobileCapacityGeometry.dialog.bottom,
  `The 390 px picker scrolls its list while keeping the last item and actions in the bounded dialog: ${JSON.stringify(mobileCapacityGeometry)}`);
  await page.locator('.context-options').evaluate(node => { node.scrollTop = 0; });
  check(await capacityNotice.isVisible() && await dialog.getByRole('button', { name: 'Attach', exact: true }).isVisible(),
    'The catalog notice and actions remain visible after returning the list to its first entries.');
  await dialog.screenshot({ path: 'docs/implementation/evidence/s33/qa08-capacity-limit-390.png' });
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).press('Enter');
  const runs = envelopes.filter(envelope => envelope.command === 'market.screen' && envelope.payload.operation === 'RUN');
  check(runs.length === 3 && runs[1].payload.filterSpec.predicates[0].threshold === '0.25'
    && runs[1].payload.revision !== runs[0].payload.revision, 'The actual RUN request uses the edited validated revision.');
  const log = await page.evaluate(async cursor => (await fetch(`/__integration/command-log?after=${cursor}`)).json(), cursor);
  const forbidden = log.commands.filter(({ command: name }) =>
    /^(trade\.|paper\.order\.|live\.|order\.|turn\.start|backtest\.(run|cancel))/.test(name)
    || (name.startsWith('provider.') && !['provider.list_definitions', 'provider.get_schema'].includes(name))
    || (name.startsWith('account.') && !['account.list', 'account.get', 'account.activity'].includes(name))
    || (name.startsWith('model.') && !['model.get', 'model.get_gateway'].includes(name))
    || (name.startsWith('data.') && /\.(probe|refresh|configure|disconnect|cleanup)$/.test(name)));
  check(forbidden.length === 0 && externalRequests.length === 0 && pageErrors.length === 0,
    `No provider/model/financial activity or browser errors: ${JSON.stringify({ forbidden, externalRequests, pageErrors })}`);
  return { result: 'PASS_LIMITED', pickerContext: created.linkedContexts, screenerContext: attached.linkedContexts,
    runs: runs.length, boundedCatalogEntries: bounded.entries.length, savedBoundaryVersions: 252,
    unsupportedSearches: ['NVDA', 'AMD'], responsiveWidths: [1280, 768, 390],
    capacityPickerViewportBounded: true, capacityListScrollsInternally: true,
    catalogNoticeAndActionsVisible: true, capacityScreenshot: 'qa08-capacity-limit-390.png',
    forbiddenCommands: forbidden.length, externalRequests: externalRequests.length, pageErrors: pageErrors.length };
}
