// Build the integration binary with `cargo build --manifest-path src-tauri/Cargo.toml --features 'integration-test order-gateway-runtime' --bin tradex-order-gateway --bin tradex-ipc`.
// Start isolated UI/IPC service: `./node_modules/.bin/vite --host 127.0.0.1 --mode integration --port 1422 --strictPort`.
// Then run `playwright-cli -s=s33-research run-code --filename=tests/s33-research-result-cards-playwright.js`.
// Exercises local synthetic research fixtures only; no market provider or model is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  page.on('request', request => {
    const url = request.url();
    if (/^https?:\/\//i.test(url) && !/^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(url)) {
      externalRequests.push(url);
    }
  });
  const command = async (name, payload) => {
    const requestId = `s33-research-${name}-${Date.now()}-${Math.random()}`;
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
  await page.goto('http://127.0.0.1:1422/');
  const commandCursor = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 synthetic research cards ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-research-cards-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Threads', exact: true }).click();
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });

  const threadTitle = `S33 synthetic research ${stamp}`;
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill(threadTitle);
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
  await page.getByRole('button', { name: /^@ Context/ }).click();
  const contextDialog = page.getByRole('dialog', { name: 'Choose context', exact: true });
  await contextDialog.getByLabel(/Synthetic research artifact/).check();
  await contextDialog.getByRole('button', { name: 'Attach', exact: true }).click();
  await page.getByRole('button', { name: /@ Context · 1/ }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Create Thread', exact: true }).click();
  await page.getByRole('heading', { name: threadTitle, exact: true }).waitFor({ state: 'visible' });

  const observations = [];
  const preview = async ({ focus, query, assertions, width, tradeActionVisible = false }) => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const detail = page.locator('.thread-detail');
    await detail.getByRole('combobox', { name: 'Research focus', exact: true }).selectOption(focus);
    await detail.getByLabel('Turn request', { exact: true }).fill(query);
    await detail.getByRole('button', { name: 'Preview typed research result', exact: true }).click();
    const card = detail.locator('.research-result');
    await card.getByText('Typed result · AVAILABLE', { exact: true }).waitFor({ state: 'visible' });
    await card.scrollIntoViewIfNeeded();
    const text = await card.innerText();
    for (const expected of assertions) {
      check(text.includes(expected), `${focus} result at ${width}px is missing ${JSON.stringify(expected)}.`);
    }
    const tradeAction = card.getByRole('button', { name: 'Review read-only proposal', exact: true });
    if (tradeActionVisible) {
      check(await tradeAction.count() === 1 && await tradeAction.isVisible(), `${focus} Trade result did not expose its documented proposal action.`);
      check(await tradeAction.isDisabled(), `${focus} synthetic preview unexpectedly enabled a proposal action.`);
    } else {
      check(await tradeAction.count() === 0, `${focus} Research result exposed a Trade-mode action.`);
    }
    const geometry = await page.evaluate(() => {
      const result = document.querySelector('.research-result');
      const box = result?.getBoundingClientRect();
      return {
        viewport: document.documentElement.clientWidth,
        documentScrollWidth: document.documentElement.scrollWidth,
        cardClientWidth: result?.clientWidth ?? 0,
        cardScrollWidth: result?.scrollWidth ?? 0,
        left: box?.left ?? -1,
        right: box?.right ?? Infinity,
      };
    });
    check(geometry.documentScrollWidth <= geometry.viewport,
      `${focus} result caused document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.cardScrollWidth <= geometry.cardClientWidth && geometry.left >= 0 && geometry.right <= geometry.viewport,
      `${focus} result card exceeds its viewport at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      const modeName = tradeActionVisible ? 'trade' : 'research';
      await page.evaluate(() => {
        const topbar = document.querySelector('.topbar');
        window.scrollBy(0, -((topbar?.getBoundingClientRect().height ?? 52) + 12));
      });
      const capturePosition = await page.evaluate(() => ({
        cardTop: document.querySelector('.thread-detail .research-result')?.getBoundingClientRect().top ?? -1,
        topbarBottom: document.querySelector('.topbar')?.getBoundingClientRect().bottom ?? 0,
      }));
      check(capturePosition.cardTop >= capturePosition.topbarBottom,
        `${focus} screenshot card heading is overlapped by the sticky Thread header: ${JSON.stringify(capturePosition)}.`);
      await card.screenshot({
        path: `docs/implementation/evidence/s33/research-${focus.toLowerCase()}-${modeName}-390.png`,
        style: '.topbar { visibility: hidden !important; }',
      });
    }
    observations.push({ focus, mode: tradeActionVisible ? 'TRADE' : 'RESEARCH', width, cardWidth: geometry.cardClientWidth, documentOverflow: false });
  };

  for (const width of [1280, 768, 390]) {
    await preview({
      focus: 'EQUITY',
      query: 'AAPL',
      width,
      assertions: [
        'Synthetic fixture · SYNTHETIC_INTEGRATION_FIXTURE',
        'Integration fixture only: typed research data is available for contract verification.',
        'Canonical instruments', 'equity:US:AAPL',
        'Key findings', 'Scenarios', 'Evidence provenance',
        'Artifact references', 'artifact-1',
        'control-plane:market', 'TradeX synthetic research fixture',
        'synthetic and cannot establish provider entitlement or execution authority',
      ],
    });
  }
  for (const width of [1280, 768, 390]) {
    await preview({
      focus: 'CRYPTO_SPOT',
      query: 'BTC/USDT',
      width,
      assertions: [
        'Synthetic fixture · SYNTHETIC_INTEGRATION_FIXTURE',
        'Venue evidence', 'BINANCE', 'BITGET', 'SELECTED',
        'Bid: 60000.00', 'Ask: 60010.00', 'Spread: 10.00', 'Depth: 1250000', 'Quote age: 2s',
        'Bid: 59990.00', 'Ask: 60020.00', 'Spread: 30.00', 'Depth: 840000', 'Quote age: 4s',
        'Evidence provenance', 'control-plane:market', 'TradeX synthetic research fixture',
        'synthetic and cannot establish provider entitlement or execution authority',
      ],
    });
  }

  const detail = page.locator('.thread-detail');
  const accountSelect = detail.getByRole('combobox', { name: 'Account', exact: true });
  const localPaperOption = accountSelect.locator('option').filter({ hasText: 'Local Paper' });
  await localPaperOption.waitFor({ state: 'attached' });
  const localPaperConnectionId = await localPaperOption.getAttribute('value');
  check(Boolean(localPaperConnectionId), 'The isolated fixture workspace must expose its Local Paper account.');
  await detail.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('LOCAL_PAPER');
  await accountSelect.selectOption(localPaperConnectionId);
  await detail.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('TRADE');
  await detail.getByRole('combobox', { name: 'Research focus', exact: true }).waitFor({ state: 'visible' });
  for (const width of [1280, 768, 390]) {
    await preview({
      focus: 'EQUITY',
      query: 'AAPL',
      width,
      tradeActionVisible: true,
      assertions: ['Synthetic fixture · SYNTHETIC_INTEGRATION_FIXTURE', 'equity:US:AAPL', 'Evidence provenance'],
    });
  }
  for (const width of [1280, 768, 390]) {
    await preview({
      focus: 'CRYPTO_SPOT',
      query: 'BTC/USDT',
      width,
      tradeActionVisible: true,
      assertions: ['Synthetic fixture · SYNTHETIC_INTEGRATION_FIXTURE', 'Venue evidence', 'BINANCE', 'BITGET', 'SELECTED'],
    });
  }

  const isForbiddenCommand = name => name.startsWith('provider.')
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
  const commands = await commandLog(commandCursor.cursor);
  const forbidden = commands.commands.filter(entry => isForbiddenCommand(entry.command));
  check(forbidden.length === 0, `The synthetic result-card test issued forbidden commands: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  const researchRuns = commands.commands.filter(entry => entry.command === 'research.run');
  check(researchRuns.length === 12, `Expected one local preview per focus/mode/viewport combination, got ${researchRuns.length}.`);

  return {
    result: 'PASS',
    cases: observations.length,
    observations,
    researchRunCommands: researchRuns.length,
    forbiddenCommands: forbidden.length,
    commandNames: [...new Set(commands.commands.map(entry => entry.command))].sort(),
    externalRequests: externalRequests.length,
    fixtureOnly: true,
  };
}
