// Run with `playwright-cli run-code --filename=tests/s33-live-order-submission-monitoring-playwright.js`
// against `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser`.
// Uses one disposable workspace and synthetic Trading 212-labelled account; the
// only order request is an explicitly approved PLACE to the local fake Gateway.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const requestUrl = request.url();
    const origin = requestUrl.match(/^https?:\/\/[^/?#]+/i)?.[0];
    if (origin && !/^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl)) {
      externalRequests.push(origin);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = `s33-live-monitor-${name}-${Date.now()}-${Math.random()}`;
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
  const localOrigin = currentUrl.match(/^http:\/\/127\.0\.0\.1:\d+/)?.[0] ?? 'http://127.0.0.1:1420';
  await page.goto(`${localOrigin}/`);
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Live Monitoring ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-live-monitor-${stamp}`;
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
    workspaceId, providerId: 'trading212', label: `S33 synthetic monitoring ${stamp}`,
  });
  const time = await command('time.revalidate', { workspaceId });
  check(time.confidence === 'TRUSTED', 'The disposable workspace has trusted synthetic time.');
  const armed = await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });
  check(armed.health.arming === 'ARMED', 'Only the synthetic account is armed for this fixture.');

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
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

  const proposalList = await command('trade.proposal.list', { workspaceId });
  check(proposalList.proposals.length === 1, 'The disposable workspace has one immutable Live proposal.');
  const proposal = await command('trade.proposal.get', {
    workspaceId, proposalId: proposalList.proposals[0].proposalId,
  });
  check(proposal.fields.environment === 'TRADING212_LIVE'
    && proposal.fields.instrumentId === 'equity:US:AAPL'
    && proposal.fields.side === 'BUY' && proposal.fields.orderType === 'LIMIT'
    && proposal.fields.quantity.value === '0.001' && proposal.fields.limitPrice === '50000'
    && proposal.fields.timeInForce === 'GTC', 'The persisted Proposal is the exact F3 AAPL limit order.');
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).press('Enter');

  const gatewayBefore = await command('live.gateway.fixture.inspect', {});
  const postsBefore = gatewayBefore.requests.filter(request => request.method === 'POST').length;
  const beforeActions = await commandLog(0);
  const approvalsBefore = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(approvalsBefore.approvals.length === 0, 'A new proposal begins without approval.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'Prepare is unavailable before explicit approval.');

  const review = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await review.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await dialog.waitFor({ state: 'visible' });
  await dialog.getByRole('status').filter({ hasText: 'Current checks pass. Approval still requires your explicit action.' }).waitFor({ state: 'visible' });
  await dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).press('Space');
  await dialog.waitFor({ state: 'hidden' });
  const issued = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(issued.approvals.length === 1 && issued.approvals[0].status === 'ISSUED',
    'One explicit approval is issued for the immutable Proposal.');
  check((await command('live.gateway.fixture.inspect', {})).requests.filter(request => request.method === 'POST').length === postsBefore,
    'Approval alone does not dispatch a provider request.');

  const prepare = page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true });
  await prepare.waitFor({ state: 'visible' });
  await command('live.gateway.fixture.set_result', { workspaceId, scenario: 'ACCEPTED' });
  await prepare.press('Enter');
  const card = page.getByLabel('TradeX execution preparation', { exact: true });
  await card.getByText(/ACCEPTED · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
  await card.getByRole('status').filter({ hasText: 'The provider accepted the order. This is not a fill.' }).waitFor({ state: 'visible' });
  await card.getByText('Provider order ID 901', { exact: true }).waitFor({ state: 'visible' });

  const consumed = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(consumed.approvals.length === 1 && consumed.approvals[0].status === 'CONSUMED',
    'The exact approval is consumed by Prepare.');
  const approval = consumed.approvals[0];
  const saved = (await command('trade.execution.preparation.get', { workspaceId, approvalId: approval.approvalId })).preparation;
  check(saved.attempt.state === 'ACCEPTED' && saved.attempt.brokerOrderId === '901'
    && saved.attempt.accountId === account.connectionId
    && saved.attempt.proposalId === proposal.proposalId
    && saved.attempt.environment === 'TRADING212_LIVE', 'The durable accepted attempt retains exact account and Proposal identity.');
  check(saved.reservation.status === 'ACTIVE' && saved.reservation.amount === '50'
    && saved.reservation.unit === 'USD' && saved.reservation.proposalId === proposal.proposalId
    && saved.reservation.proposalHash === proposal.proposalHash,
  `The accepted order retains its exact active reservation: ${JSON.stringify(saved.reservation)}`);
  check(!saved.liveOrderSettlement, 'Acceptance alone creates no provider fill or settlement evidence.');
  check(await card.getByLabel('Live order fill and fee evidence', { exact: true }).count() === 0,
    'No fill evidence is rendered before a read-only provider observation.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'The consumed approval has no resend action.');
  const currentProposal = await command('trade.proposal.get', { workspaceId, proposalId: proposal.proposalId });
  check(currentProposal.proposalHash === proposal.proposalHash && currentProposal.status === 'CONSUMED',
    'The submitted Proposal hash is unchanged and its terminal state is persisted.');

  const expiryClock = await command('time.status', { workspaceId });
  await command('time.fixture.advance', {
    workspaceId, elapsedMs: Math.max(1, Date.parse(approval.expiresAt) + 1 - Date.parse(expiryClock.wallClock)),
  });
  const expiredHistory = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  const expiredApproval = expiredHistory.approvals.find(item => item.approvalId === approval.approvalId);
  const expiry = expiredApproval?.expiry;
  check(expiredApproval?.status === 'CONSUMED' && expiredApproval.consumedAt === approval.consumedAt
    && expiry?.scope === 'approval' && expiry.authority === 'TRUSTED_TIME_SERVICE'
    && expiry.authorityReference === approval.approvalId && expiry.expiresAt === approval.expiresAt
    && Date.parse(expiry.observedAt) >= Date.parse(approval.expiresAt)
    && expiry.reservationDisposition === 'RETAINED'
    && expiry.executionAttemptId === saved.attempt.attemptId
    && expiry.reservationId === saved.reservation.reservationId
    && expiry.reservationStateVersion === saved.reservation.stateVersion,
  'Post-dispatch TTL creates a trusted expiry audit while preserving consumption and exact held-capacity evidence.');
  const afterExpiry = (await command('trade.execution.preparation.get', { workspaceId, approvalId: approval.approvalId })).preparation;
  check(afterExpiry.attempt.state === 'ACCEPTED' && afterExpiry.attempt.stateVersion === saved.attempt.stateVersion
    && JSON.stringify(afterExpiry.reservation) === JSON.stringify(saved.reservation),
  'Approval expiry leaves the accepted order and its exact active reservation unchanged.');
  await page.getByRole('button', { name: 'Reload history', exact: true }).press('Enter');
  const expiredRecord = page.locator('.live-approval-record:not([aria-label])').filter({ hasText: approval.approvalId });
  await expiredRecord.getByText(/Approval authority expired:/).waitFor({ state: 'visible' });
  await expiredRecord.getByText('Capacity remains held until authoritative reconciliation.', { exact: true })
    .waitFor({ state: 'visible' });

  const expiryStatus = page.getByRole('status', { name: 'Approval expiry status', exact: true });
  await expiryStatus.getByText('Approval authority expired. Capacity remains held until authoritative reconciliation.', { exact: true }).waitFor({ state: 'visible' });
  check(await expiryStatus.getAttribute('aria-live') === 'polite' && await expiryStatus.getAttribute('aria-atomic') === 'true',
    'The expiry status provides a concise polite, atomic DOM announcement.');

  const responsiveWidths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await card.scrollIntoViewIfNeeded();
    const geometry = await card.evaluate(element => {
      const box = element.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth, documentWidth: document.documentElement.scrollWidth,
        left: box.left, right: box.right, statusVisible: box.height > 0 };
    });
    check(geometry.documentWidth <= width && geometry.left >= 0 && geometry.right <= width && geometry.statusVisible,
      `The accepted attempt is readable without horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    const expiryBounds = await expiredRecord.evaluate(element => {
      const box = element.getBoundingClientRect();
      return { left: box.left, right: box.right };
    });
    check(expiryBounds.left >= 0 && expiryBounds.right <= width,
      `The persisted expiry and retained-capacity disclosure fits at ${width}px: ${JSON.stringify(expiryBounds)}`);
    responsiveWidths.push(width);
    if (width === 390) {
      await page.screenshot({ path: 'docs/implementation/evidence/s33/f3-live-accepted-390.png' });
      await expiredRecord.scrollIntoViewIfNeeded();
      await page.screenshot({ path: 'docs/implementation/evidence/s33/f6-consumed-expiry-retained-390.png' });
    }
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).press('Enter');
  const restored = page.getByLabel('TradeX execution preparation', { exact: true });
  await restored.getByText(/ACCEPTED · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
  check((await restored.innerText()).includes(saved.attempt.attemptId), 'Navigation restores the saved attempt identity.');
  check((await restored.innerText()).includes('Capacity after TradeX reservation'),
    'Navigation restores the active reservation projection.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'Navigation does not restore a resend action.');
  const gatewayAfter = await command('live.gateway.fixture.inspect', {});
  const postsAfter = gatewayAfter.requests.filter(request => request.method === 'POST').length;
  check(postsAfter === postsBefore + 1, `Exactly one local fake-Gateway request was sent: ${postsAfter - postsBefore}`);
  const actions = (await commandLog(beforeActions.cursor)).commands;
  const approvals = actions.filter(entry => entry.command === 'trade.approve');
  const preparations = actions.filter(entry => entry.command === 'trade.execution.prepare');
  const providerMutations = actions.filter(entry => /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(entry.command));
  check(approvals.length === 1 && preparations.length === 1 && providerMutations.length === 0,
    `One explicit approval and one Prepare, with no provider management: ${JSON.stringify({ approvals, preparations, providerMutations })}`);
  check(externalRequests.length === 0, `No external HTTP was issued: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `No browser errors: ${JSON.stringify({ consoleErrors, pageErrors })}`);

  return {
    result: 'PASS_LIMITED', screenId: 'F3', attempt: 'ACCEPTED',
    proposalId: proposal.proposalId, proposalHashUnchanged: true,
    reservation: { amount: saved.reservation.amount, unit: saved.reservation.unit, status: saved.reservation.status },
    acceptedIsNotFill: true, restoredAfterNavigation: true, fakeGatewayPosts: postsAfter - postsBefore,
    approvalAuthorityExpired: true, expiryReservationDisposition: expiry.reservationDisposition,
    expiryPreservesAttemptAndReservation: true,
    explicitApprovals: approvals.length, prepares: preparations.length, providerMutations: providerMutations.length,
    externalRequests: externalRequests.length, responsiveWidths,
  };
}
