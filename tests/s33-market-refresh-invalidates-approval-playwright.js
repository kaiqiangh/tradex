// Run with mcp__playwright__browser_run_code_unsafe against the isolated
// Rust-backed Vite harness on 127.0.0.1:1424 with the explicit Live fixture.
// Uses synthetic local fixtures;
// it never contacts a provider, prepares an order, or dispatches to a Gateway.
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
  page.on('console', message => {
    if (message.type() === 'error') consoleErrors.push(message.text());
  });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = `s33-market-stale-${name}-${Date.now()}-${Math.random()}`;
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
  await page.goto('http://127.0.0.1:1424/');
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Market Refresh ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-market-refresh-${stamp}`;
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
    workspaceId, providerId: 'trading212', label: `S33 synthetic market refresh ${stamp}`,
  });
  await command('time.revalidate', { workspaceId });
  await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });
  const initialMarket = await command('trade.live_approval.market.fixture.set', {
    workspaceId, instrumentId: 'equity:US:AAPL', revision: 'INITIAL',
  });
  check(initialMarket.provenance?.marketSnapshotId === 'synthetic-live-approval-v1',
    'The isolated Live review starts from a named synthetic quote revision.');

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
  await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
  await page.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('510');
  await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const proposalSummary = (await command('trade.proposal.list', { workspaceId })).proposals[0];
  check(proposalSummary?.status === 'NEEDS_APPROVAL', 'The isolated workspace has one synthetic approval proposal.');
  const proposal = await command('trade.proposal.get', {
    workspaceId, proposalId: proposalSummary.proposalId,
  });
  check(proposal.marketSnapshotId === initialMarket.provenance.marketSnapshotId,
    'The immutable proposal records the quote revision used to generate it.');
  await page.locator('.order-proposal-row').first().press('Enter');

  const beforeRefresh = await commandLog(0);
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
    'The synthetic review has one persisted issued approval before refresh.');

  const changedMarket = await command('trade.live_approval.market.fixture.set', {
    workspaceId, instrumentId: 'equity:US:AAPL', revision: 'CHANGED',
  });
  check(changedMarket.provenance?.marketSnapshotId === 'synthetic-live-approval-v2'
      && changedMarket.provenance.marketSnapshotId !== proposal.marketSnapshotId,
    'The local fixture advances to a distinct quote revision before refresh.');
  const quoteChangeApprovals = await command('trade.approval.list', {
    workspaceId, proposalId: proposal.proposalId,
  });
  check(quoteChangeApprovals.approvals.length === 1
      && quoteChangeApprovals.approvals[0].status === 'INVALIDATED',
    'Reading approval history reconciles and invalidates consent after the market revision changes.');
  check(quoteChangeApprovals.approvals[0].invalidationReason?.startsWith('RISK_DECISION_CHANGED:'),
    `The persisted invalidation reason identifies changed risk evidence: ${quoteChangeApprovals.approvals[0].invalidationReason}`);

  await page.getByRole('button', { name: 'Refresh proposal', exact: true }).press('Enter');
  const refreshNotice = page.getByRole('status').filter({ hasText: /Proposal refreshed \(/ });
  await refreshNotice.waitFor({ state: 'visible' });
  const refreshStatus = (await refreshNotice.innerText()).match(/Proposal refreshed \(([^)]+)\)/)?.[1];
  check(refreshStatus === 'REFRESHED', `The refreshed proposal references the current synthetic market (${refreshStatus}).`);
  const library = await command('trade.proposal.list', { workspaceId });
  const replacementSummary = library.proposals.find(item => item.proposalId !== proposal.proposalId);
  check(replacementSummary?.status === 'NEEDS_APPROVAL', 'Refresh creates a distinct proposal that requires new approval.');
  const replacement = await command('trade.proposal.get', {
    workspaceId, proposalId: replacementSummary.proposalId,
  });
  check(replacement.proposalHash !== proposal.proposalHash, 'Refresh creates a new immutable proposal hash.');
  check(replacement.marketSnapshotId === changedMarket.provenance.marketSnapshotId,
    'The replacement records the changed quote revision.');
  const oldProposal = await command('trade.proposal.get', { workspaceId, proposalId: proposal.proposalId });
  const history = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(oldProposal.status === 'INVALIDATED', 'The original proposal remains inspectable as invalidated.');
  check(oldProposal.proposalHash === proposal.proposalHash
      && oldProposal.marketSnapshotId === initialMarket.provenance.marketSnapshotId,
    'Refreshing preserves the original proposal hash and its v1 market binding.');
  check(oldProposal.history.some(event => event.event === 'REFRESHED'), 'The original proposal records REFRESHED history.');
  check(history.approvals.length === 1 && history.approvals[0].status === 'INVALIDATED',
    'Refreshing invalidates the already-issued approval.');
  check(history.approvals[0].invalidationReason?.startsWith('RISK_DECISION_CHANGED:'),
    'The persisted approval history retains the quote-change invalidation reason after proposal refresh.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'No Prepare action remains available after the old approval is invalidated.');

  await page.getByRole('button', { name: /INVALIDATED v1/ }).press('Enter');
  await page.locator('.order-proposal-panel .error-text')
    .filter({ hasText: /Proposal refreshed as proposal:/ }).waitFor({ state: 'visible' });
  await page.getByText(/INVALIDATED · PLACE_ORDER/, { exact: false }).waitFor({ state: 'visible' });
  const reviewOld = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await reviewOld.waitFor({ state: 'visible' });
  check(await reviewOld.isDisabled(), 'An invalidated proposal cannot be reviewed for another approval.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'The invalidated proposal exposes no Prepare action.');
  await page.locator('.order-proposal-row').filter({ hasText: replacement.proposalId }).press('Enter');
  check((await command('trade.approval.list', { workspaceId, proposalId: replacement.proposalId })).approvals.length === 0,
    'The replacement proposal has no inherited approval.');
  const newReview = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await newReview.press('Enter');
  const newDialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await newDialog.waitFor({ state: 'visible' });
  check((await newDialog.innerText()).includes('Current checks pass. Approval still requires your explicit action.'),
    'The replacement requires a fresh explicit review and approval.');
  await newDialog.press('Escape');
  await newDialog.waitFor({ state: 'hidden' });

  const invalidatedResponsive = [];
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).press('Enter');
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      bounds: [...document.querySelectorAll('.order-proposal-panel, .live-approval-record')].map(node => {
        const rect = node.getBoundingClientRect(); return { left: rect.left, right: rect.right };
      }),
    }));
    check(geometry.scroll <= geometry.viewport, `No horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.bounds.every(bounds => bounds.left >= 0 && bounds.right <= geometry.viewport),
      `Invalidation details stay in view at ${width}px: ${JSON.stringify(geometry)}`);
    invalidatedResponsive.push(width);
    if (width === 390) {
      await page.locator('.live-approval-record').first().scrollIntoViewIfNeeded();
      await page.screenshot({ path: 'docs/implementation/evidence/s33/market-refresh-invalidated-approval-390.png' });
    }
  }
  await page.locator('.order-proposal-row').filter({ hasText: replacement.proposalId }).press('Enter');
  const replacementMarketLine = page.locator('.order-proposal-panel .proposal-reference')
    .filter({ hasText: 'synthetic-live-approval-v2' });
  await replacementMarketLine.waitFor({ state: 'visible' });
  check((await replacementMarketLine.innerText()).includes('AVAILABLE'),
    'The refreshed proposal visibly identifies the current available market snapshot.');

  const replacementResponsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      bounds: [...document.querySelectorAll('.order-proposal-panel, .live-approval-record')].map(node => {
        const rect = node.getBoundingClientRect(); return { left: rect.left, right: rect.right };
      }),
    }));
    check(geometry.scroll <= geometry.viewport, `No replacement overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.bounds.every(bounds => bounds.left >= 0 && bounds.right <= geometry.viewport),
      `Replacement details stay in view at ${width}px: ${JSON.stringify(geometry)}`);
    replacementResponsive.push(width);
    if (width === 390) {
      await replacementMarketLine.scrollIntoViewIfNeeded();
      await page.screenshot({ path: 'docs/implementation/evidence/s33/market-refresh-replacement-v2-390.png' });
    }
  }

  await page.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const replacementReview = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await replacementReview.waitFor({ state: 'visible' });
  const approveReplacement = replacementReview.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  check(await approveReplacement.isEnabled(), 'The replacement allows a separate explicit approval action.');
  await approveReplacement.press('Space');
  await replacementReview.waitFor({ state: 'hidden' });
  const replacementApprovals = await command('trade.approval.list', {
    workspaceId, proposalId: replacement.proposalId,
  });
  check(replacementApprovals.approvals.length === 1
      && replacementApprovals.approvals[0].status === 'ISSUED'
      && replacementApprovals.approvals[0].proposalHash === replacement.proposalHash,
    'Only a separate explicit action issues a new approval bound to the replacement proposal.');

  const afterRefresh = await commandLog(beforeRefresh.cursor);
  const forbidden = afterRefresh.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(entry.command)
      || (entry.command.startsWith('trade.execution.') && !entry.command.endsWith('.preparation.get'))
      || entry.command.startsWith('live.gateway.'));
  check(forbidden.length === 0, `No provider, Prepare, execution, or Gateway command: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `No external HTTP request: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0, `No browser console errors: ${JSON.stringify(consoleErrors)}`);
  check(pageErrors.length === 0, `No browser page errors: ${JSON.stringify(pageErrors)}`);

  const reopened = await command('workspace.open', { path: workspacePath });
  check(reopened.workspaceId === workspaceId, 'The same disposable workspace can be reopened from its persisted path.');
  const persistedOldProposal = await command('trade.proposal.get', {
    workspaceId, proposalId: proposal.proposalId,
  });
  const persistedOldApprovals = await command('trade.approval.list', {
    workspaceId, proposalId: proposal.proposalId,
  });
  check(persistedOldProposal.status === 'INVALIDATED'
      && persistedOldProposal.proposalHash === proposal.proposalHash
      && persistedOldProposal.marketSnapshotId === initialMarket.provenance.marketSnapshotId,
    'Workspace reopen preserves the invalidated v1 proposal and its immutable hash.');
  check(persistedOldApprovals.approvals.length === 1
      && persistedOldApprovals.approvals[0].status === 'INVALIDATED'
      && persistedOldApprovals.approvals[0].invalidationReason?.startsWith('RISK_DECISION_CHANGED:'),
    'Workspace reopen preserves the old approval invalidation and exact reason category.');

  return {
    result: 'PASS_LIMITED',
    screenId: 'F5',
    refreshStatus,
    oldProposal: oldProposal.status,
    oldApproval: history.approvals[0].status,
    marketChangeInvalidationReason: history.approvals[0].invalidationReason,
    replacementProposal: replacement.status,
    originalMarketSnapshotId: proposal.marketSnapshotId,
    replacementMarketSnapshotId: replacement.marketSnapshotId,
    replacementApprovals: replacementApprovals.approvals.length,
    prepareActionsAfterRefresh: 0,
    invalidatedProposalResponsiveWidths: invalidatedResponsive,
    replacementProposalResponsiveWidths: replacementResponsive,
    reopenedWorkspace: true,
    forbiddenCommands: forbidden.length,
    externalHttpRequests: externalRequests.length,
    consoleErrors: consoleErrors.length,
    pageErrors: pageErrors.length,
    quoteBoundary: `synthetic quote revisions ${proposal.marketSnapshotId} -> ${replacement.marketSnapshotId}; no provider source configured`,
  };
}
