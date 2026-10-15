// Run with `/tmp/run-tradex-playwright.mjs` against the Rust-backed browser harness.
// Uses a disposable workspace and synthetic Binance-labelled account/market quote;
// no Binance credential, provider endpoint, approval, or order is used.
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
    const requestId = `s33-market-approval-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const commandLog = async cursor => page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), cursor);

  await page.setViewportSize({ width: 1280, height: 900 });
  const currentUrl = page.url();
  const localOrigin = currentUrl.startsWith('http://127.0.0.1:')
    ? new URL(currentUrl).origin
    : 'http://127.0.0.1:1420';
  await page.goto(`${localOrigin}/`);
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Market Approval ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-market-approval-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const workspaceId = workspace.workspaceId;
  const policy = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: policy.stateVersion,
    policy: {
      ...policy.policy,
      marketOrdersEnabled: true,
      maxMarketOrderSlippagePercent: '5',
      maxSingleInstrumentExposurePercent: null,
      staleQuoteThresholdSeconds: 120,
    },
  });
  const account = await command('account.arming.fixture.seed', {
    workspaceId, providerId: 'binance', label: `S33 synthetic market approval ${stamp}`,
  });
  const time = await command('time.revalidate', { workspaceId });
  check(time.confidence === 'TRUSTED', 'The disposable workspace has trusted synthetic time.');
  const armed = await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });
  check(armed.health.arming === 'ARMED', 'Only the synthetic Binance-labelled account is armed.');

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
  await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('MARKET');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
  await page.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('500');
  await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const proposals = await command('trade.proposal.list', { workspaceId });
  check(proposals.proposals.length === 1, 'The workspace has one synthetic Binance Live proposal.');
  const proposal = await command('trade.proposal.get', { workspaceId, proposalId: proposals.proposals[0].proposalId });
  check(proposal.fields.environment === 'BINANCE_LIVE'
    && proposal.fields.instrumentId === 'crypto:BTC/USDT:spot'
    && proposal.fields.side === 'BUY' && proposal.fields.orderType === 'MARKET'
    && proposal.fields.quantity.value === '0.01' && proposal.fields.maximumSpend === '500',
  'The immutable transaction is the intended bounded BTC market order.');
  check(typeof proposal.estimatedNotional === 'string', 'The proposal has a backend estimated notional.');
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).press('Enter');

  const beforeReview = await commandLog(0);
  const reviewButton = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await reviewButton.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const approve = dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  const reject = dialog.getByRole('button', { name: 'Reject this review', exact: true });
  const keepReviewing = dialog.getByRole('button', { name: 'Keep reviewing', exact: true });
  const approveEnabled = await approve.isEnabled();
  check(!approveEnabled, 'Required spend above the immutable maximum hard-blocks approval.');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'The safe dismissal action receives initial focus.');

  const text = await dialog.innerText();
  const expectedNotional = `${proposal.estimatedNotional} ${proposal.estimatedNotionalCurrency ?? ''}`.trim();
  for (const expected of [
    proposal.proposalId, proposal.proposalHash, account.connectionId, account.data.remoteAccountId,
    'BINANCE_LIVE', 'binance', 'crypto:BTC/USDT:spot', 'BINANCE',
    'BUY', '0.01 BASE', 'MARKET · No limit · GTC',
    `Estimated notional\n${expectedNotional}`,
    'Expected spend\n500.01 USDT', 'Maximum authorized spend\n500 USDT',
    'Backend capacity preview', 'Provider available', 'TradeX reserved', 'Effective available',
    'Quote provenance', 'SYNTHETIC_INTEGRATION_FIXTURE', 'Provider timestamp', 'Received / venue',
    'Entitlement / freshness', 'Quote age', 'Bid / ask / spread\n49999 / 50001 / 2',
    'Risk checks', 'MARKET_MAXIMUM_AUTHORIZATION_EXCEEDED',
    'Reject this review', 'Approve for up to 30 seconds', 'Keep reviewing',
  ]) check(text.includes(expected), `The F4 review shows ${expected}: ${text}`);
  check(text.includes('Approval blocked:'), 'The approval blocker is exposed as accessible status text.');
  const quoteRows = await dialog.locator('section[aria-labelledby="approval-quote-title"] dl').evaluate(list =>
    Object.fromEntries([...list.querySelectorAll(':scope > div')].map(row => [
      row.querySelector('dt')?.innerText.trim(), row.querySelector('dd')?.innerText.trim(),
    ])));
  for (const label of ['Snapshot / source', 'Provider timestamp', 'Received / venue', 'Entitlement / freshness', 'Quote age']) {
    check(quoteRows[label] && !quoteRows[label].includes('Unavailable'),
      `The F4 synthetic quote provenance includes ${label}: ${JSON.stringify(quoteRows)}`);
  }
  check(/^\d+ ms$/.test(quoteRows['Quote age']), `The F4 review shows a measured quote age: ${JSON.stringify(quoteRows)}`);
  const expectedSpendRow = await dialog.locator('.approval-review-fields').first().evaluate(list => {
    const row = [...list.querySelectorAll(':scope > div')].find(item => item.querySelector('dt')?.innerText.trim() === 'Expected spend');
    return row?.querySelector('dd')?.innerText.trim() ?? '';
  });
  check(expectedSpendRow === '500.01 USDT', `Expected spend remains separate from the maximum: ${expectedSpendRow}`);
  check(text.includes('Estimated fees\nUnavailable') || /Estimated fees\n\S/.test(text),
    'Estimated fees are either explicitly shown or labeled unavailable.');
  check(text.includes('Estimated slippage\nUnavailable') || /Estimated slippage\nApproximately /.test(text),
    'Estimated slippage is either shown from evidence or labeled unavailable.');
  check((await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId })).approvals.length === 0,
    'Opening the blocked review creates no approval.');

  const focusables = dialog.locator('button:not(:disabled)');
  check(await focusables.count() === 2, 'The blocked review keeps Reject and safe dismissal available while Approve is disabled.');
  await page.keyboard.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Reject this review',
    'Tab reaches Reject before the safe dismissal action.');
  await page.keyboard.press('Shift+Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'Shift+Tab returns to the safe dismissal action.');

  const responsiveWidths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    for (const button of [reject, keepReviewing]) await button.scrollIntoViewIfNeeded();
    const geometry = await page.evaluate(() => {
      const dialog = document.querySelector('.approval-review-dialog');
      const actions = [...(dialog?.querySelectorAll('.approval-review-actions button') ?? [])].map(button => {
        const box = button.getBoundingClientRect();
        return { left: box.left, right: box.right, top: box.top, bottom: box.bottom };
      });
      return { viewport: document.documentElement.clientWidth, documentWidth: document.documentElement.scrollWidth,
        dialogWidth: dialog?.clientWidth, dialogScroll: dialog?.scrollWidth, actions, height: window.innerHeight };
    });
    check(geometry.documentWidth <= width && geometry.dialogScroll <= geometry.dialogWidth
      && geometry.actions.every(box => box.left >= 0 && box.right <= width && box.top >= 0 && box.bottom <= geometry.height + 1),
    `F4 review and actions fit at ${width}px: ${JSON.stringify(geometry)}`);
    responsiveWidths.push(width);
    if (width === 390) {
      await dialog.evaluate(element => { element.scrollTop = 0; });
      await page.locator('.picker-backdrop').evaluate(element => { element.scrollTop = 0; });
      await page.screenshot({ path: 'docs/implementation/evidence/s33/f4-market-approval-390.png' });
    }
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await keepReviewing.press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  const afterReview = await commandLog(beforeReview.cursor);
  const mutations = afterReview.commands.filter(entry =>
    /^(trade\.(approve|reject|execution\.)|provider\.(connect|probe|permissions|disconnect)|account\.refresh|live\.gateway\.)/.test(entry.command));
  check(mutations.length === 0, `The blocked review caused no approval, rejection, execution, provider, or Gateway mutation: ${JSON.stringify(mutations)}`);
  check(externalRequests.length === 0, `No external HTTP request was issued: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `No browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}`);

  return {
    result: 'PASS_LIMITED', screenId: 'F4', instrument: 'crypto:BTC/USDT:spot',
    expectedSpend: '500.01 USDT', maximumAuthorizedSpend: '500 USDT',
    hardBlock: 'MARKET_MAXIMUM_AUTHORIZATION_EXCEEDED', approvalEnabled: approveEnabled,
    approvals: 0, mutations: mutations.length, externalRequests: externalRequests.length, responsiveWidths,
  };
}
