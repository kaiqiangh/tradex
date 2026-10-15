// Run with mcp__playwright__browser_run_code_unsafe against an isolated
// Rust-backed integration Vite page already open at 127.0.0.1.
// The only submission target is TradeX's local fake Gateway; no provider is contacted.
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
    const requestId = `s33-broker-rejected-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 Broker Rejected ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-broker-rejected-${stamp}`;
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
    policy: { ...policy.policy, maxSingleInstrumentExposurePercent: null, staleQuoteThresholdSeconds: 120 },
  });
  const account = await command('account.arming.fixture.seed', {
    workspaceId, providerId: 'trading212', label: `S33 synthetic rejection ${stamp}`,
  });
  await command('time.revalidate', { workspaceId });
  await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
  await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
  await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const proposal = (await command('trade.proposal.list', { workspaceId })).proposals[0];
  check(proposal?.status === 'NEEDS_APPROVAL', 'The synthetic proposal is awaiting explicit approval.');
  await page.locator('.order-proposal-row').first().press('Enter');

  const gatewayBefore = await command('live.gateway.fixture.inspect', {});
  const postsBefore = gatewayBefore.requests.filter(request => request.method === 'POST').length;
  const beforeAttempt = await commandLog(0);
  const review = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await review.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const approve = dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  check(await approve.isEnabled(), 'The explicitly synthetic Live review is eligible.');
  await approve.press('Space');
  await dialog.waitFor({ state: 'hidden' });
  const issued = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(issued.approvals.length === 1 && issued.approvals[0].status === 'ISSUED',
    'An explicit action issued one bounded approval.');

  await command('live.gateway.fixture.set_result', { workspaceId, scenario: 'REJECTED' });
  await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).press('Enter');
  const rejectedCard = page.getByLabel('TradeX execution preparation', { exact: true });
  await rejectedCard.getByText(/REJECTED · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
  await rejectedCard.getByRole('status')
    .filter({ hasText: 'The provider rejected the order. TradeX released its reservation.' }).waitFor({ state: 'visible' });
  const approval = (await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId })).approvals[0];
  const preparation = await command('trade.execution.preparation.get', { workspaceId, approvalId: approval.approvalId });
  check(preparation.preparation?.attempt.state === 'REJECTED', 'The rejection is persisted as a terminal attempt.');
  check(preparation.preparation?.reservation.status === 'RELEASED', 'The synthetic capacity reservation is released.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'A rejected, consumed approval cannot expose a resend action.');
  const errorCode = preparation.preparation.attempt.errorCode;
  check(errorCode === 'PROVIDER_ORDER_REJECTED', `The raw outcome code is retained (${errorCode}).`);
  check(await rejectedCard.getByText('Outcome category SUBMISSION_REJECTED', { exact: true }).count() === 1,
    'The UI presents the RevC canonical SUBMISSION_REJECTED category.');

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await rejectedCard.scrollIntoViewIfNeeded();
    const geometry = await page.evaluate(() => {
      const card = document.querySelector('[aria-label="TradeX execution preparation"]');
      const rect = card?.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        left: rect?.left ?? null, right: rect?.right ?? null };
    });
    check(geometry.scroll <= geometry.viewport && geometry.left >= 0 && geometry.right <= geometry.viewport,
      `The rejected attempt fits at ${width}px: ${JSON.stringify(geometry)}`);
    responsive.push(width);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/broker-rejected-390.png' });
  }

  const postsAfter = (await command('live.gateway.fixture.inspect', {})).requests.filter(request => request.method === 'POST');
  check(postsAfter.length === postsBefore + 1, 'Exactly one local fake-Gateway POST was issued.');
  const afterAttempt = await commandLog(beforeAttempt.cursor);
  const providerCommands = afterAttempt.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(entry.command));
  const executionMutations = afterAttempt.commands.filter(entry =>
    entry.command.startsWith('trade.execution.') && !entry.command.endsWith('.preparation.get'));
  check(providerCommands.length === 0, `No provider integration command occurred: ${JSON.stringify(providerCommands)}`);
  check(executionMutations.length === 1 && executionMutations[0].command === 'trade.execution.prepare',
    `Only the expected local Prepare command occurred: ${JSON.stringify(executionMutations)}`);
  check(externalRequests.length === 0, `No external HTTP request: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0, `No browser console errors: ${JSON.stringify(consoleErrors)}`);
  check(pageErrors.length === 0, `No browser page errors: ${JSON.stringify(pageErrors)}`);

  return {
    result: 'PASS_LIMITED', screenId: 'F8', attempt: 'REJECTED',
    errorCode, errorCategory: 'SUBMISSION_REJECTED', reservation: 'RELEASED',
    fakeGatewayPosts: postsAfter.length - postsBefore, resendActions: 0,
    responsiveWidths: responsive, providerCommands: providerCommands.length,
    externalHttpRequests: externalRequests.length, consoleErrors: consoleErrors.length, pageErrors: pageErrors.length,
  };
}
