// Run with mcp__playwright__browser_run_code_unsafe against an isolated
// Rust-backed integration Vite page. All account state and Gateway results are
// synthetic; the one read-only reconciliation refresh uses the in-process
// tests/support/provider_fixtures.rs HTTP transport and never contacts a broker.
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
    const requestId = `s33-ambiguous-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 Ambiguous Submission ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-ambiguous-${stamp}`;
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
    workspaceId, providerId: 'trading212', label: `S33 synthetic ambiguous ${stamp}`,
  });
  await command('time.revalidate', { workspaceId });
  const armed = await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });
  check(armed.health.arming === 'ARMED', 'The synthetic account is explicitly armed for the fixture attempt.');
  const market = await command('trade.live_approval.market.fixture.set', {
    workspaceId, instrumentId: 'equity:US:AAPL', revision: 'INITIAL',
  });
  check(market.provenance?.marketSnapshotId === 'synthetic-live-approval-v1',
    'The approved proposal uses a stable, explicitly labelled synthetic market snapshot.');

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
  const proposal = (await command('trade.proposal.list', { workspaceId })).proposals[0];
  check(proposal?.status === 'NEEDS_APPROVAL', 'The synthetic proposal awaits explicit approval.');
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).click();

  const beforeGateway = (await command('live.gateway.fixture.inspect', {})).requests.filter(request => request.method === 'POST').length;
  const beforeLog = await commandLog(0);
  await page.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const approve = dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  check(await approve.isEnabled(), 'The isolated synthetic Live approval is eligible.');
  await approve.press('Space');
  await dialog.waitFor({ state: 'hidden' });
  const issued = (await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId })).approvals;
  check(issued.length === 1 && issued[0].status === 'ISSUED', 'One explicit approval was issued.');

  await command('live.gateway.fixture.set_result', { workspaceId, scenario: 'UNKNOWN' });
  await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).press('Enter');
  const attemptCard = page.getByLabel('TradeX execution preparation', { exact: true });
  await attemptCard.getByText(/UNKNOWN_RECONCILING · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
  await attemptCard.getByRole('status')
    .filter({ hasText: 'The provider outcome is uncertain. Capacity remains held; do not resend this request.' })
    .waitFor({ state: 'visible' });
  const unknownAttemptText = await attemptCard.innerText();
  check(!/\b(?:ACKNOWLEDGED|ACCEPTED|PARTIALLY_FILLED|FILLED|CANCELLED)\b/.test(unknownAttemptText),
    `The unknown attempt contains no fabricated acknowledgement or fill state: ${unknownAttemptText}`);
  check(await attemptCard.getByText(/Provider order ID|Provider status/i).count() === 0,
    'No broker order identity or provider status is added before one is observed.');

  const approval = issued[0];
  const initialPreparation = await command('trade.execution.preparation.get', { workspaceId, approvalId: approval.approvalId });
  check(initialPreparation.preparation.attempt.state === 'UNKNOWN_RECONCILING', 'The uncertain result is persisted.');
  check(initialPreparation.preparation.reservation.status === 'ACTIVE', 'Capacity remains reserved while outcome is unknown.');
  const attempt = initialPreparation.preparation.attempt;
  const evidencePanel = page.getByLabel('Live reconciliation evidence', { exact: true });
  await page.locator('.order-proposal-row').filter({ hasText: attempt.intentId }).click();
  await evidencePanel.getByText(/No exact order observed; absence is not proven|Inconclusive · PROVIDER_RATE_LIMITED/).waitFor({ state: 'visible' });
  const activityTimeline = evidencePanel.getByRole('region', { name: 'Order activity timeline', exact: true });
  await activityTimeline.waitFor({ state: 'visible' });
  const timelineText = await activityTimeline.innerText();
  check(timelineText.includes(attempt.accountId) && timelineText.includes(attempt.attemptId)
    && timelineText.includes(attempt.proposalId),
  'The activity timeline binds the exact Live account, proposal and attempt.');
  check(timelineText.includes('Pending provider evidence · no order ID observed'),
    'Provider order identity remains explicitly pending when no exact order was observed.');
  check(timelineText.includes('APPROVED · Live approval issued')
    && timelineText.includes('RESERVED · TradeX capacity held')
    && timelineText.includes('SUBMITTING · provider request may have been sent')
    && timelineText.includes('Read-only provider observation · INCONCLUSIVE'),
  `The timeline shows only the saved approval, reservation, dispatch and read-only observation: ${timelineText}`);
  check(timelineText.includes('UNKNOWN_RECONCILING')
    && !/\b(?:ACKNOWLEDGED|ACCEPTED|PARTIALLY_FILLED|FILLED|CANCELLED)\b/.test(timelineText),
  `The unknown timeline contains no fabricated acknowledgement, fill or cancellation: ${timelineText}`);
  const timelineTimes = await activityTimeline.locator('li time[datetime]').evaluateAll(nodes =>
    nodes.map(node => Date.parse(node.getAttribute('datetime') ?? '')));
  check(timelineTimes.length === 4 && timelineTimes.every(Number.isFinite)
    && timelineTimes.every((value, index) => index === 0 || timelineTimes[index - 1] <= value),
  `The four persisted activity timestamps are present and chronological: ${JSON.stringify(timelineTimes)}`);
  await page.setViewportSize({ width: 390, height: 844 });
  const timelineGeometry = await activityTimeline.evaluate(element => {
    const rect = element.getBoundingClientRect();
    return { width: rect.width, left: rect.left, right: rect.right,
      viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth };
  });
  check(timelineGeometry.left >= 0 && timelineGeometry.right <= timelineGeometry.viewport
    && timelineGeometry.scroll <= timelineGeometry.viewport,
  `The order activity timeline remains inside the 390px viewport: ${JSON.stringify(timelineGeometry)}`);
  await activityTimeline.screenshot({ path: 'docs/implementation/evidence/s33/qa02-unknown-order-activity-390.png' });
  await page.setViewportSize({ width: 1280, height: 900 });
  const initialEvidence = await command('trade.resolution_evidence', {
    workspaceId, executionAttemptId: attempt.attemptId, accountId: account.connectionId,
  });
  check(initialEvidence.ledger.evidence.length > 0, 'Read-only reconciliation evidence is recorded before any manual decision.');
  check(initialEvidence.ledger.evidence.at(-1).candidateOrders.length === 0, 'No broker order is fabricated as positive evidence.');

  const time = await command('time.status', { workspaceId });
  const deadline = Date.parse(attempt.dispatchStartedAt) + 5 * 60_000;
  await command('time.fixture.advance', {
    workspaceId, elapsedMs: Math.max(1, deadline + 1_000 - Date.parse(time.wallClock)),
  });
  const expiredEvidence = await command('trade.resolution_evidence', {
    workspaceId, executionAttemptId: attempt.attemptId, accountId: account.connectionId,
  });
  check(expiredEvidence.timeTrusted === true && expiredEvidence.automaticWindowExpired === true,
    'The trusted five-minute automatic query window has ended.');
  check(JSON.stringify([...expiredEvidence.allowedDecisions]) === JSON.stringify(['KEEP_RECONCILING']),
    `The synthetic Trading 212 scenario permits only Keep reconciling: ${JSON.stringify(expiredEvidence.allowedDecisions)}`);

  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.locator('.order-proposal-row').filter({ hasText: attempt.intentId }).click();
  await evidencePanel.getByText(/The five-minute query window ended/).waitFor({ state: 'visible' });
  const keep = evidencePanel.getByRole('button', { name: 'Keep reconciling', exact: true });
  await keep.waitFor({ state: 'visible' });
  check(await evidencePanel.getByRole('button', { name: /Confirm not submitted/ }).count() === 0,
    'No unsupported absence confirmation is offered.');
  check(await evidencePanel.getByRole('button', { name: /Confirm submitted order/ }).count() === 0,
    'No submitted-order confirmation is offered without exact identity evidence.');
  check(await evidencePanel.getByRole('button', { name: /release reservation|release capacity|continue live trading/i }).count() === 0,
    'There is no generic release-and-continue action.');

  const accountAtTimeout = (await command('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
  check(accountAtTimeout?.health.arming === 'DISARMED'
    && accountAtTimeout.health.reconciliation === 'STALE'
    && accountAtTimeout.health.reason.includes('remains unknown after the five-minute automatic reconciliation window'),
    `The account is disarmed after the reconciliation window: ${JSON.stringify(accountAtTimeout?.health)}`);
  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await evidencePanel.evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        left: rect.left, right: rect.right, panelScroll: element.scrollWidth, panelClient: element.clientWidth };
    });
    check(geometry.scroll <= geometry.viewport && geometry.left >= 0 && geometry.right <= geometry.viewport
      && geometry.panelScroll <= geometry.panelClient + 1,
    `The reconciliation evidence fits at ${width}px: ${JSON.stringify(geometry)}`);
    responsive.push(width);
    if (width === 390) {
      await evidencePanel.scrollIntoViewIfNeeded();
      await page.screenshot({ path: 'docs/implementation/evidence/s33/ambiguous-submission-390.png' });
    }
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await keep.press('Enter');
  await page.getByRole('status')
    .filter({ hasText: 'Keep reconciling was recorded. The reservation remains frozen' }).waitFor({ state: 'visible' });
  const resolvedEvidence = await command('trade.resolution_evidence', {
    workspaceId, executionAttemptId: attempt.attemptId, accountId: account.connectionId,
  });
  check(resolvedEvidence.ledger.manualResolutions.at(-1).decision === 'KEEP_RECONCILING',
    'The Keep decision is persisted in the manual-resolution ledger.');
  const finalPreparation = await command('trade.execution.preparation.get', { workspaceId, approvalId: approval.approvalId });
  check(finalPreparation.preparation.attempt.state === 'UNKNOWN_RECONCILING', 'Manual Keep does not falsely resolve the outcome.');
  check(finalPreparation.preparation.reservation.status === 'ACTIVE', 'Manual Keep preserves the reservation.');
  const resolvedTimeline = evidencePanel.getByRole('region', { name: 'Order activity timeline', exact: true });
  await resolvedTimeline.getByText('Manual Resolution · KEEP_RECONCILING', { exact: true }).waitFor({ state: 'visible' });
  check(!/\b(?:ACKNOWLEDGED|ACCEPTED|PARTIALLY_FILLED|FILLED|CANCELLED)\b/.test(await resolvedTimeline.innerText()),
    'Persisting Keep reconciling adds only that observed decision and does not invent a provider result.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'The consumed approval cannot be resent.');
  const afterGateway = (await command('live.gateway.fixture.inspect', {})).requests.filter(request => request.method === 'POST');
  check(afterGateway.length === beforeGateway + 1, 'Unknown handling and Manual Resolution issue no additional Gateway POST.');

  const afterLog = await commandLog(beforeLog.cursor);
  const providerManagementCommands = afterLog.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(entry.command));
  check(providerManagementCommands.length === 0,
    `No provider connect/probe/permission/disconnect/account-refresh command was invoked: ${JSON.stringify(providerManagementCommands)}`);
  const reconciliationRefreshes = afterLog.commands.filter(entry => entry.command === 'trade.resolution_evidence.refresh');
  check(reconciliationRefreshes.length === 1,
    `Exactly one read-only reconciliation refresh uses the local HTTP fixture: ${reconciliationRefreshes.length}`);
  check(afterLog.commands.filter(entry => entry.command === 'trade.manual_resolution').length === 1,
    'Exactly one Keep reconciling decision is persisted.');
  check(externalRequests.length === 0, `No external HTTP request was observed: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0, `No browser console errors: ${JSON.stringify(consoleErrors)}`);
  check(pageErrors.length === 0, `No browser page errors: ${JSON.stringify(pageErrors)}`);

  return {
    result: 'PASS_LIMITED', screenId: 'F9 / QA-02', attempt: 'UNKNOWN_RECONCILING',
    observedTimeline: ['APPROVED', 'RESERVED', 'SUBMITTING', 'INCONCLUSIVE', 'KEEP_RECONCILING'],
    fabricatedAcknowledgementOrFill: false, providerOrderIdentity: 'PENDING',
    automaticWindowExpired: true, allowedManualResolution: ['KEEP_RECONCILING'],
    accountArming: accountAtTimeout.health.arming, accountReconciliation: accountAtTimeout.health.reconciliation,
    reservation: finalPreparation.preparation.reservation.status,
    fakeGatewayPosts: afterGateway.length - beforeGateway, resendActions: 0,
    responsiveWidths: responsive, timelineViewport: 390, providerManagementCommands: providerManagementCommands.length,
    syntheticReadOnlyReconciliationRefreshes: reconciliationRefreshes.length,
    externalHttpRequests: externalRequests.length, consoleErrors: consoleErrors.length, pageErrors: pageErrors.length,
  };
}
