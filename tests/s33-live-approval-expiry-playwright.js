// Run with `playwright-cli run-code --filename=tests/s33-live-approval-expiry-playwright.js`
// against the Rust-backed `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser` harness.
// Uses a disposable workspace and a synthetic Live fixture; no provider is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const requestUrl = request.url();
    const externalHttpOrigin = requestUrl.match(/^https?:\/\/[^/?#]+/i)?.[0];
    if (externalHttpOrigin && !/^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl)) {
      externalRequests.push(externalHttpOrigin);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-approval-expiry-${name}-${Date.now()}-${Math.random()}`;
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
  const selectDraft = async draftId => {
    const library = await command('trade.draft.list', { workspaceId });
    const index = library.drafts.findIndex(draft => draft.draftId === draftId);
    check(index >= 0, `Draft ${draftId} remains in the local draft library.`);
    await page.locator('.order-draft-row').nth(index).press('Enter');
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Approval Expiry ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-approval-expiry-${stamp}`;
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
    workspaceId, providerId: 'trading212', label: `S33 synthetic expiry ${stamp}`,
  });
  await command('time.revalidate', { workspaceId });
  await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  const createLiveProposal = async maximumSpend => {
    await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
    await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await page.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill(maximumSpend);
    await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  };
  await createLiveProposal('510');
  const proposals = await command('trade.proposal.list', { workspaceId });
  check(proposals.proposals.length === 1, 'The disposable workspace has one synthetic Live proposal.');
  const proposal = proposals.proposals[0];
  await page.locator('.order-proposal-row').first().press('Enter');
  await page.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const approve = dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  check(await approve.isEnabled(), 'The synthetic proposal is eligible for its explicit approval action.');
  const gatewayBefore = await command('live.gateway.fixture.inspect', {});
  const beforeApproval = await commandLog(0);
  await approve.press('Space');
  await dialog.waitFor({ state: 'hidden' });
  const issued = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  const approval = issued.approvals[0];
  check(issued.approvals.length === 1 && approval.status === 'ISSUED'
    && approval.proposalId === proposal.proposalId && approval.accountId === account.connectionId
    && approval.environment === 'TRADING212_LIVE' && approval.operation === 'PLACE_ORDER',
  'The explicit Space action issues one approval bound to the exact Live proposal, account, environment and operation.');
  const issuedAt = Date.parse(approval.issuedAt);
  const expiresAt = Date.parse(approval.expiresAt);
  check(Number.isFinite(issuedAt) && Number.isFinite(expiresAt)
    && expiresAt > Date.now() && expiresAt - issuedAt <= 30000,
  'The issued approval has a future expiry no later than 30 seconds after issue.');

  await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await createLiveProposal('520');
  const proposalList = await command('trade.proposal.list', { workspaceId });
  check(proposalList.proposals.length === 2, 'Two independent proposals cover unprepared and RESERVED expiry paths.');
  const reservedProposal = proposalList.proposals.find(item => item.proposalId !== proposal.proposalId);
  check(reservedProposal?.status === 'NEEDS_APPROVAL', 'The second proposal is independent and still requires approval.');
  await page.locator('.order-proposal-row').filter({ hasText: reservedProposal.proposalId }).press('Enter');
  await page.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const reservedApprovalDialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await reservedApprovalDialog.waitFor({ state: 'visible' });
  const reservedApprove = reservedApprovalDialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  check(await reservedApprove.isEnabled(), 'The second proposal receives its own explicit approval before time advances.');
  await reservedApprove.press('Space');
  await reservedApprovalDialog.waitFor({ state: 'hidden' });
  const reservedApprovalHistory = await command('trade.approval.list', {
    workspaceId, proposalId: reservedProposal.proposalId,
  });
  const reservedApproval = reservedApprovalHistory.approvals[0];
  check(reservedApprovalHistory.approvals.length === 1 && reservedApproval.status === 'ISSUED'
    && reservedApproval.proposalId === reservedProposal.proposalId,
  'The second approval is persisted against only its own proposal.');

  const reservePayload = {
    workspaceId,
    approvalId: reservedApproval.approvalId,
    expectedApprovalStateVersion: reservedApproval.stateVersion,
    idempotencyKey: `s33-approval-expiry-${stamp}`,
    confirmed: true,
  };
  const reserved = await command('trade.execution.fixture.prepare_reserved', reservePayload);
  check(reserved.attempt?.state === 'RESERVED' && reserved.reservation?.status === 'ACTIVE',
    `The integration-only gate holds a real domain preparation at RESERVED: ${JSON.stringify(reserved)}`);
  const reservationAmount = reserved.reservation.amount;
  const reservationUnit = reserved.reservation.unit;

  await command('time.fixture.advance', {
    workspaceId,
    elapsedMs: Math.max(
      30001,
      expiresAt - issuedAt + 1,
      Date.parse(reservedApproval.expiresAt) - Date.parse(reservedApproval.issuedAt) + 1,
    ),
  });
  const expired = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(expired.approvals[0]?.status === 'EXPIRED'
    && expired.approvals[0]?.invalidationReason === 'APPROVAL_EXPIRED',
  'Trusted synthetic time expiry is persisted with the canonical reason.');
  const unusedExpiry = expired.approvals[0].expiry;
  check(unusedExpiry?.scope === 'approval' && unusedExpiry.authority === 'TRUSTED_TIME_SERVICE'
    && unusedExpiry.authorityReference === approval.approvalId
    && unusedExpiry.expiresAt === approval.expiresAt
    && Date.parse(unusedExpiry.observedAt) >= expiresAt
    && unusedExpiry.reservationDisposition === 'NONE' && !unusedExpiry.reservationId,
  'Unused expiry records its exact approval and trusted deadline without fabricating a reservation.');
  await selectDraft(proposal.draftId);
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).press('Enter');
  await page.getByRole('button', { name: 'Reload history', exact: true }).press('Enter');
  const expiredRecord = page.locator('.live-approval-record').filter({ hasText: 'EXPIRED · PLACE_ORDER' });
  await expiredRecord.waitFor({ state: 'visible' });
  await expiredRecord.getByText('Reason: APPROVAL_EXPIRED', { exact: true }).waitFor({ state: 'visible' });
  check((await page.getByLabel('Live approval history', { exact: true }).innerText())
    .includes('Approval alone sends nothing.'),
  'The approval screen keeps the no-provider-request disclosure.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'The expired approval exposes no Prepare action.');

  const expiryStatus = page.getByRole('status', { name: 'Approval expiry status', exact: true });
  await expiryStatus.getByText('Approval authority expired. This approval held no new reservation.', { exact: true }).waitFor({ state: 'visible' });
  check(await expiryStatus.getAttribute('aria-live') === 'polite' && await expiryStatus.getAttribute('aria-atomic') === 'true',
    'The expiry status provides a concise polite, atomic DOM announcement.');

  const expiredResponsiveWidths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await expiredRecord.scrollIntoViewIfNeeded();
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      bounds: [...document.querySelectorAll('.order-proposal-panel, .live-approval-record')]
        .map(node => { const rect = node.getBoundingClientRect(); return { left: rect.left, right: rect.right }; }),
    }));
    check(geometry.scroll <= geometry.viewport, `Expired approval has no overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.bounds.every(bounds => bounds.left >= 0 && bounds.right <= geometry.viewport),
      `Expired approval remains in view at ${width}px: ${JSON.stringify(geometry)}`);
    expiredResponsiveWidths.push(width);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/live-approval-expired-390.png' });
  }

  check((await command('trade.proposal.get', { workspaceId, proposalId: proposal.proposalId })).status === 'NEEDS_APPROVAL',
    'Expiry does not rewrite or consume the proposal itself.');
  const stopped = await command('trade.execution.preparation.get', {
    workspaceId, approvalId: reservedApproval.approvalId,
  });
  check(stopped.preparation?.attempt.state === 'INVALIDATED'
    && stopped.preparation.attempt.invalidationReason === 'APPROVAL_EXPIRED'
    && stopped.preparation.attempt.dispatchDisposition === 'STOPPED_BEFORE_DISPATCH',
  `Trusted expiry stops the RESERVED attempt before dispatch: ${JSON.stringify(stopped.preparation?.attempt)}`);
  check(stopped.preparation.reservation?.status === 'RELEASED'
    && stopped.preparation.reservation.amount === reservationAmount
    && stopped.preparation.reservation.unit === reservationUnit,
  'The exact active PLACE reservation is released when expiry wins before dispatch.');
  const consumedHistory = await command('trade.approval.list', {
    workspaceId, proposalId: reservedProposal.proposalId,
  });
  check(consumedHistory.approvals.find(item => item.approvalId === reservedApproval.approvalId)?.status === 'CONSUMED',
    'The stopped attempt retains its consumed approval for audit without restoring consent.');
  const reservedExpiry = consumedHistory.approvals.find(item => item.approvalId === reservedApproval.approvalId).expiry;
  check(reservedExpiry?.scope === 'approval' && reservedExpiry.authority === 'TRUSTED_TIME_SERVICE'
    && reservedExpiry.authorityReference === reservedApproval.approvalId
    && reservedExpiry.reservationDisposition === 'RELEASED'
    && reservedExpiry.executionAttemptId === reserved.attempt.attemptId
    && reservedExpiry.reservationId === reserved.reservation.reservationId
    && reservedExpiry.reservationStateVersion === stopped.preparation.reservation.stateVersion,
  'Reserved expiry records the exact stopped attempt and released reservation evidence.');
  const replayed = await command('trade.execution.fixture.prepare_reserved', reservePayload);
  check(replayed.attempt?.attemptId === reserved.attempt.attemptId
    && replayed.attempt.state === 'INVALIDATED',
  'Same-key replay restores the stopped attempt instead of creating or dispatching another one.');
  const stillReleased = await command('trade.execution.preparation.get', {
    workspaceId, approvalId: reservedApproval.approvalId,
  });
  check(stillReleased.preparation?.reservation?.status === 'RELEASED'
    && stillReleased.preparation.reservation.reservationId === reserved.reservation.reservationId
    && stillReleased.preparation.reservation.amount === reservationAmount
    && stillReleased.preparation.reservation.stateVersion === stopped.preparation.reservation.stateVersion
    && stillReleased.preparation.attempt.stateVersion === stopped.preparation.attempt.stateVersion,
  'Replay leaves the exact stopped attempt and released reservation unchanged.');
  await selectDraft(reservedProposal.draftId);
  await page.locator('.order-proposal-row').filter({ hasText: reservedProposal.proposalId }).press('Enter');
  const stoppedCard = page.getByLabel('TradeX execution preparation', { exact: true });
  await stoppedCard.getByText('Stopped before provider dispatch. No order request was sent.', { exact: true })
    .waitFor({ state: 'visible' });
  await stoppedCard.getByText(`Reservation released: ${reservationAmount} ${reservationUnit}. This capacity is no longer held by TradeX.`, { exact: true })
    .waitFor({ state: 'visible' });
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'The invalidated attempt exposes no new Prepare action.');
  const stoppedResponsiveWidths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await stoppedCard.scrollIntoViewIfNeeded();
    const stoppedGeometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      card: document.querySelector('[aria-label="TradeX execution preparation"]')?.getBoundingClientRect(),
    }));
    check(stoppedGeometry.scroll <= stoppedGeometry.viewport
      && stoppedGeometry.card && stoppedGeometry.card.left >= 0 && stoppedGeometry.card.right <= stoppedGeometry.viewport,
    `The stopped-before-dispatch card fits at ${width}px: ${JSON.stringify(stoppedGeometry)}`);
    stoppedResponsiveWidths.push(width);
    if (width === 390) {
      await page.screenshot({ path: 'docs/implementation/evidence/s33/live-approval-expired-reserved-released-390.png' });
    }
  }

  const gatewayAfter = await command('live.gateway.fixture.inspect', {});
  check(gatewayAfter.requests.length === gatewayBefore.requests.length,
    'Approval expiry and same-key replay add no local fake-Gateway request.');
  const afterExpiry = await commandLog(beforeApproval.cursor);
  const forbidden = afterExpiry.commands.filter(entry => {
    const name = entry.command;
    return /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(name)
      || (name.startsWith('trade.execution.')
        && !['trade.execution.preparation.get', 'trade.execution.fixture.prepare_reserved'].includes(name))
      || name.startsWith('trade.prepare')
      || (name.startsWith('live.gateway.') && name !== 'live.gateway.fixture.inspect');
  });
  check(forbidden.length === 0,
    `Approval expiry issued no provider, Prepare, execution, or Gateway command: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `No external HTTP request: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0, `No browser console errors: ${JSON.stringify(consoleErrors)}`);
  check(pageErrors.length === 0, `No browser page errors: ${JSON.stringify(pageErrors)}`);

  await command('workspace.open', { path: workspacePath });
  const persistedExpired = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  const persistedStopped = await command('trade.execution.preparation.get', {
    workspaceId, approvalId: reservedApproval.approvalId,
  });
  check(persistedExpired.approvals.find(item => item.approvalId === approval.approvalId)?.status === 'EXPIRED'
    && persistedExpired.approvals.find(item => item.approvalId === approval.approvalId)?.invalidationReason === 'APPROVAL_EXPIRED',
  'The expired approval and canonical reason persist after same-process workspace reopen.');
  check(persistedStopped.preparation?.attempt.attemptId === reserved.attempt.attemptId
    && persistedStopped.preparation.attempt.state === 'INVALIDATED'
    && persistedStopped.preparation.reservation?.status === 'RELEASED',
  'The stopped attempt and released reservation persist after same-process workspace reopen.');
  const persistedConsumed = await command('trade.approval.list', { workspaceId, proposalId: reservedProposal.proposalId });
  check(JSON.stringify(persistedExpired.approvals[0].expiry) === JSON.stringify(unusedExpiry)
    && JSON.stringify(persistedConsumed.approvals[0].expiry) === JSON.stringify(reservedExpiry),
  'The exact expiry evidence persists after workspace reopen.');

  return {
    result: 'PASS',
    boundary: 'disposable workspace and synthetic Trading 212-labelled Live account; no provider endpoint or credentials',
    approvalBeforeExpiry: approval.status,
    approvalAfterExpiry: expired.approvals[0].status,
    approvalExpiryReason: expired.approvals[0].invalidationReason,
    expiryReservationDispositions: [unusedExpiry.reservationDisposition, reservedExpiry.reservationDisposition],
    preparedAttemptExpiry: stopped.preparation.attempt.state,
    preparedApprovalAfterExpiry: consumedHistory.approvals.find(item => item.approvalId === reservedApproval.approvalId).status,
    preparedAttemptReason: stopped.preparation.attempt.invalidationReason,
    reservationAfterExpiry: stopped.preparation.reservation.status,
    reservationAmountReleased: reservationAmount,
    sameKeyReplayAttemptId: replayed.attempt.attemptId,
    expiredApprovalResponsiveWidths: expiredResponsiveWidths,
    stoppedAttemptResponsiveWidths: stoppedResponsiveWidths,
    workspaceReopened: true,
    reservedFixturePreparations: afterExpiry.commands.filter(entry => entry.command === 'trade.execution.fixture.prepare_reserved').length,
    directPrepareCommands: afterExpiry.commands.filter(entry => entry.command === 'trade.execution.prepare').length,
    fakeGatewayRequestsAdded: gatewayAfter.requests.length - gatewayBefore.requests.length,
    forbiddenCommands: forbidden.length,
    externalHttpRequests: externalRequests.length,
    consoleErrors: consoleErrors.length,
    pageErrors: pageErrors.length,
  };
}
