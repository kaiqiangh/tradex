// Run against a fresh `TRADEX_LIVE_APPROVAL_FIXTURE=1 TRADEX_INTEGRATION_ORDER_GATEWAY=1 npm run dev:browser`
// harness. The only PLACE request is explicitly approved and reaches the local fake Gateway.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  const command = async (name, payload) => {
    const requestId = `s33-e4-linked-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200, `${name} bridge status ${result.status}`);
    check(result.body.requestId === requestId, `${name} request ID mismatch`);
    check(result.body.ok, `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 E4 linked Proposal ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-e4-linked-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const policy = await command('risk.get_policy', { workspaceId: workspace.workspaceId });
  await command('risk.save_policy', {
    workspaceId: workspace.workspaceId,
    expectedStateVersion: policy.stateVersion,
    policy: { ...policy.policy, staleQuoteThresholdSeconds: 120, maxSingleInstrumentExposurePercent: null },
  });
  const account = await command('account.arming.fixture.seed', {
    workspaceId: workspace.workspaceId, providerId: 'trading212', label: `Synthetic E4 linked ${Date.now()}`,
  });
  await command('time.revalidate', { workspaceId: workspace.workspaceId });
  const armed = await command('account.arm', {
    workspaceId: workspace.workspaceId,
    connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion,
    confirmed: true,
  });
  check(armed.health.arming === 'ARMED', 'Only the disposable synthetic Live account is armed for the approval fixture.');

  const navigation = page.getByRole('navigation', { name: 'Primary navigation' });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).click();
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
  await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.008');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
  await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await page.getByRole('button', { name: 'Save draft', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const library = await command('trade.proposal.list', { workspaceId: workspace.workspaceId });
  const proposalSummary = library.proposals[0];
  check(proposalSummary, 'The synthetic order Proposal was persisted.');
  const proposal = await command('trade.proposal.get', { workspaceId: workspace.workspaceId, proposalId: proposalSummary.proposalId });
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).click();
  await page.getByRole('button', { name: 'Review Live approval', exact: true }).click();
  const review = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await review.waitFor({ state: 'visible' });
  await page.getByRole('status').filter({ hasText: 'Current checks pass. Approval still requires your explicit action.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).press('Space');
  await review.waitFor({ state: 'hidden' });
  await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).waitFor({ state: 'visible' });
  await command('live.gateway.fixture.set_result', { workspaceId: workspace.workspaceId, scenario: 'ACCEPTED' });
  await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).click();
  const preparationCard = page.getByLabel('TradeX execution preparation', { exact: true });
  await preparationCard.getByText(/ACCEPTED · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
  await preparationCard.getByText('Provider order ID 901', { exact: true }).waitFor({ state: 'visible' });
  const approvalHistory = await command('trade.approval.list', {
    workspaceId: workspace.workspaceId, proposalId: proposal.proposalId,
  });
  const approval = approvalHistory.approvals[0];
  check(approval?.status === 'CONSUMED', 'The accepted PLACE consumed its exact explicit approval.');
  const preparation = (await command('trade.execution.preparation.get', {
    workspaceId: workspace.workspaceId, approvalId: approval.approvalId,
  })).preparation;
  check(preparation.attempt.state === 'ACCEPTED' && preparation.attempt.brokerOrderId === '901',
    'The fake Gateway persisted the accepted order identity.');
  check(preparation.reservation.status === 'ACTIVE', 'The exact PLACE reservation remains active after acceptance.');
  const linked = await command('trade.execution.order_links', {
    workspaceId: workspace.workspaceId, accountId: account.connectionId, brokerOrderIds: ['901', '9010'],
  });
  check(linked.links.length === 1 && linked.links[0].brokerOrderId === '901', 'The read-only query returns only the complete exact broker ID.');
  check(linked.links[0].proposalId === proposal.proposalId && linked.links[0].executionAttemptId === preparation.attempt.attemptId,
    'The exact match retains the persisted Proposal and attempt identities.');
  check(linked.links[0].reservation?.reservationId === preparation.reservation.reservationId,
    'The exact match retains its persisted reservation.');

  await command('account.open_order.fixture.seed', {
    workspaceId: workspace.workspaceId, connectionId: account.connectionId, brokerOrderId: '901',
  });
  const beforeInspection = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());
  const beforeSequence = beforeInspection.commands.at(-1)?.sequence ?? 0;
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).click();
  await page.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
  const accountRow = page.locator('.account-row').filter({ hasText: account.label });
  await accountRow.waitFor({ state: 'visible' });
  await accountRow.click();
  const openOrder = page.locator('.table-scroll[aria-label="Open orders"] tbody tr').filter({ hasText: '901' });
  await openOrder.waitFor({ state: 'visible', timeout: 20000 });
  await openOrder.getByText(`Proposal: ${proposal.proposalId}`, { exact: false }).waitFor({ state: 'visible' });
  const rowText = await openOrder.innerText();
  check(rowText.includes('901') && rowText.includes('PARTIALLY_FILLED'), `Provider identity and state are present: ${rowText}`);
  check(rowText.includes(`Proposal: ${proposal.proposalId}`), `The exact TradeX Proposal is shown: ${rowText}`);
  check(rowText.includes(`Attempt: ${preparation.attempt.attemptId}`), `The exact execution attempt is shown: ${rowText}`);
  check(rowText.includes(`Reservation: ${preparation.reservation.amount} ${preparation.reservation.unit} · ACTIVE`),
    `The active reservation amount/unit/status are shown: ${rowText}`);
  check(!rowText.includes('No exact TradeX PLACE match'), 'An exact match is not rendered as unlinked.');

  const widths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: 900 });
    const geometry = await page.evaluate(() => {
      const scroller = document.querySelector('.table-scroll[aria-label="Open orders"]');
      return { viewport: document.documentElement.clientWidth, documentWidth: document.documentElement.scrollWidth,
        tableClientWidth: scroller?.clientWidth ?? 0, tableScrollWidth: scroller?.scrollWidth ?? 0 };
    });
    check(geometry.documentWidth <= geometry.viewport, `No document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.tableClientWidth > 0, `The open-order table remains present at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      const reachable = await openOrder.locator('xpath=ancestor::div[contains(@class,"table-scroll")]').evaluate(element => {
        element.scrollLeft = element.scrollWidth;
        const bounds = element.getBoundingClientRect();
        const action = Array.from(element.querySelectorAll('button')).find(button => button.textContent?.trim() === 'Review cancellation');
        const actionBounds = action?.getBoundingClientRect();
        return { left: bounds.left, right: bounds.right, actionLeft: actionBounds?.left, actionRight: actionBounds?.right,
          scrollWidth: element.scrollWidth, clientWidth: element.clientWidth };
      });
      check(reachable.scrollWidth > reachable.clientWidth && reachable.actionLeft >= reachable.left && reachable.actionRight <= reachable.right,
        `The order row's cancellation control remains reachable by internal table scrolling: ${JSON.stringify(reachable)}`);
      await page.screenshot({ path: 'docs/implementation/evidence/s33/e4-open-order-linked-390.png', fullPage: true });
      widths.push({ width, ...geometry, cancellationControlReachable: true });
    } else widths.push({ width, ...geometry });
  }

  const log = await page.evaluate(async after => (await fetch(`/__integration/command-log?after=${after}`)).json(), beforeSequence);
  const postApprovalFinancialCommands = log.commands.filter(entry =>
    /^(trade\.(approve|cancel_approve|cancel_reject)|trade\.execution\.prepare|live\.gateway\.)/.test(entry.command));
  check(postApprovalFinancialCommands.length === 0,
    `Inspecting the order link issued a financial action: ${JSON.stringify(postApprovalFinancialCommands)}`);
  const fakeGatewayRequests = (await command('live.gateway.fixture.inspect', {})).requests.filter(entry => entry.method === 'POST');
  check(fakeGatewayRequests.length === 1, `The disposable workspace sent more than its single approved fake PLACE: ${JSON.stringify(fakeGatewayRequests)}`);
  check(externalRequests.length === 0, `The browser contacted an external host: ${JSON.stringify(externalRequests)}`);
  return {
    result: 'PASS',
    brokerOrderId: '901',
    proposalId: proposal.proposalId,
    executionAttemptId: preparation.attempt.attemptId,
    reservation: { amount: preparation.reservation.amount, unit: preparation.reservation.unit, status: preparation.reservation.status },
    rowText,
    widths,
    postApprovalFinancialCommands: postApprovalFinancialCommands.length,
    approvedFakePlaceRequests: fakeGatewayRequests.length,
    externalRequests: externalRequests.length,
  };
}
