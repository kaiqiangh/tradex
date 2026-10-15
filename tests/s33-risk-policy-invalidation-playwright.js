// Run with mcp__playwright__browser_run_code_unsafe against an isolated
// Rust-backed integration Vite page. Synthetic Live accounts and market
// evidence are local fixtures; this test never prepares an order or contacts a provider.
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
    const requestId = `s33-policy-invalidation-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 Policy Invalidation ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-policy-invalidation-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const workspaceId = workspace.workspaceId;
  const beforeLog = await commandLog(0);

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Risk & Limits', exact: true }).press('Enter');
  const riskForm = page.locator('.risk-defaults');
  const orderLimit = riskForm.getByRole('textbox', { name: 'Maximum order notional (USD)', exact: true });
  const exposureLimit = riskForm.getByRole('textbox', { name: 'Maximum single-instrument exposure (%)', exact: true });
  await orderLimit.fill('100');
  await exposureLimit.fill('');
  await riskForm.getByRole('button', { name: 'Save risk defaults', exact: true }).press('Enter');
  await riskForm.getByRole('status').filter({ hasText: /^Saved policy v\d+\.$/ }).waitFor({ state: 'visible' });
  let policy = await command('risk.get_policy', { workspaceId });
  check(policy.policy.maxOrderNotional === '100', 'The initial workspace policy caps each test order at 100 USD.');

  const accounts = [];
  for (const suffix of ['A', 'B']) {
    const account = await command('account.arming.fixture.seed', {
      workspaceId, providerId: 'trading212', label: `S33 policy fixture ${suffix} ${stamp}`,
    });
    await command('time.revalidate', { workspaceId });
    const armed = await command('account.arm', {
      workspaceId, connectionId: account.connectionId,
      expectedStateVersion: account.stateVersion, confirmed: true,
    });
    check(armed.health.arming === 'ARMED', `Synthetic account ${suffix} is explicitly armed.`);
    accounts.push(account);
  }

  const makeApprovedProposal = async (account, suffix, limitPrice) => {
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    if (suffix === 'B') await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
    await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill(String(limitPrice));
    await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    const draftsBefore = (await command('trade.draft.list', { workspaceId })).drafts.map(item => item.draftId);
    await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await page.getByRole('status').filter({ hasText: /Draft saved at version \d+\./ }).waitFor({ state: 'visible' });
    await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const proposalSummaries = (await command('trade.proposal.list', { workspaceId })).proposals;
    let proposal;
    for (const summary of proposalSummaries) {
      const detail = await command('trade.proposal.get', { workspaceId, proposalId: summary.proposalId });
      if (detail.fields.accountId === account.connectionId && detail.status === 'NEEDS_APPROVAL'
        && detail.fields.quantity.value === '0.001' && detail.fields.side === 'BUY'
        && detail.fields.limitPrice === String(limitPrice)) proposal = detail;
    }
    check(proposal, `Synthetic proposal ${suffix} is persisted for the exact account and draft.`);
    check(!draftsBefore.includes(proposal.draftId), `Proposal ${suffix} uses a newly saved draft.`);
    const review = await command('trade.request_approval', { workspaceId, proposalId: proposal.proposalId });
    check(review.eligible === true, `Proposal ${suffix} passes the explicit synthetic risk review: ${JSON.stringify(review.blockers)}`);
    const issued = await command('trade.approve', {
      workspaceId,
      proposalId: proposal.proposalId,
      proposalHash: review.proposal.proposalHash,
      reviewedRiskDecisionId: review.riskDecision.decisionId,
      reviewDigest: review.reviewDigest,
      expectedStateVersion: review.proposal.stateVersion,
    });
    check(issued.status === 'ISSUED', `Proposal ${suffix} has an explicit, persisted approval.`);
    return { proposal, approval: issued };
  };

  const first = await makeApprovedProposal(accounts[0], 'A', 50000);
  const second = await makeApprovedProposal(accounts[1], 'B', 40000);
  policy = await command('risk.get_policy', { workspaceId });
  const oldPolicyVersion = policy.policyVersion;
  const accountsBeforeChange = (await command('account.list', { workspaceId })).accounts;
  const initialApprovals = await Promise.all([first, second].map(async item =>
    (await command('trade.approval.list', { workspaceId, proposalId: item.proposal.proposalId })).approvals[0]));
  check(initialApprovals.every(approval => approval.status === 'ISSUED'), 'Both prior decisions remain issued before policy change.');

  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Risk & Limits', exact: true }).press('Enter');
  await orderLimit.waitFor({ state: 'visible' });
  await orderLimit.fill('');
  await riskForm.getByRole('button', { name: 'Save risk defaults', exact: true }).press('Enter');
  await riskForm.getByRole('status').filter({ hasText: /^Saved policy v\d+\.$/ }).waitFor({ state: 'visible' });

  const updatedPolicy = await command('risk.get_policy', { workspaceId });
  const change = updatedPolicy.lastChange;
  check(updatedPolicy.policyVersion === oldPolicyVersion + 1, 'The shared policy version increments exactly once.');
  check(change?.scope.kind === 'WORKSPACE' && change.scope.workspaceId === workspaceId,
    'The persisted change is explicitly workspace scoped.');
  check(change.weakened === true && change.weakeningReasons.length > 0,
    `Removing the order cap is classified as a policy weakening: ${JSON.stringify(change)}`);
  check(change.affectedAccounts.length === accountsBeforeChange.length
    && accountsBeforeChange.every(account => change.affectedAccounts.some(item =>
      item.accountId === account.connectionId && item.environment === account.environment)),
  `The change includes every persisted workspace account, not just the selected account: ${JSON.stringify(change.affectedAccounts)}`);
  check(change.affectedProposals.length === 2
    && [first.proposal.proposalId, second.proposal.proposalId].every(id =>
      change.affectedProposals.some(item => item.proposalId === id && item.policyVersion === oldPolicyVersion)),
  `The change lists both old-version pending proposals: ${JSON.stringify(change.affectedProposals)}`);

  const changeRegion = riskForm.getByRole('status').filter({ hasText: `Policy change · v${oldPolicyVersion} → v${updatedPolicy.policyVersion}` });
  await changeRegion.waitFor({ state: 'visible' });
  check(await changeRegion.getAttribute('aria-live') === 'polite', 'The policy-change summary is a polite live region.');
  check(await changeRegion.getAttribute('aria-atomic') === 'true', 'The policy-change summary is atomic.');
  check((await changeRegion.innerText()).includes(`WORKSPACE scope · ${accountsBeforeChange.length} affected accounts · 2 pending proposals invalidated.`),
    `The live summary names workspace scope and both affected counts: ${await changeRegion.innerText()}`);
  const details = changeRegion.locator('details');
  await details.nth(0).locator('summary').press('Enter');
  await details.nth(1).locator('summary').press('Enter');
  const detailsText = await changeRegion.innerText();
  for (const account of accountsBeforeChange) check(detailsText.includes(account.connectionId), `Affected account ${account.connectionId} is visible.`);
  const invalidationReason = `Risk policy version changed from ${oldPolicyVersion} to ${updatedPolicy.policyVersion}; this proposal decision is stale.`;
  for (const item of [first, second]) {
    check(detailsText.includes(item.proposal.proposalId), `Invalidated proposal ${item.proposal.proposalId} is visible.`);
    check(detailsText.includes(`${item.proposal.proposalId} · ${invalidationReason}`), 'The exact policy invalidation reason is visible beside each proposal.');
  }
  check(detailsText.includes('Policy relaxation:'), 'The weakening is disclosed in the live region.');

  const accountList = await command('account.list', { workspaceId });
  for (const account of accounts) {
    const updated = accountList.accounts.find(item => item.connectionId === account.connectionId);
    check(updated?.health.arming === 'DISARMED', `Weakening disarms affected account ${account.connectionId}.`);
  }
  for (const item of [first, second]) {
    const proposal = await command('trade.proposal.get', { workspaceId, proposalId: item.proposal.proposalId });
    check(proposal.status === 'INVALIDATED', `Old proposal ${item.proposal.proposalId} is invalidated.`);
    const approvals = await command('trade.approval.list', { workspaceId, proposalId: item.proposal.proposalId });
    check(approvals.approvals[0]?.status === 'INVALIDATED', 'A prior issued approval cannot be reused.');
    const history = await command('risk.decision.list', { workspaceId, proposalId: item.proposal.proposalId });
    const decision = history.decisions.at(-1);
    const policyVersionCheck = decision?.checks.find(entry => entry.checkId === 'POLICY_VERSION');
    check(decision?.proposalId === item.proposal.proposalId
      && policyVersionCheck?.outcome === 'REJECT'
      && policyVersionCheck.reasonCode === 'POLICY_VERSION_STALE'
      && policyVersionCheck.reason === 'The proposal was generated under a stale policy version.',
    `Persisted RiskDecision history preserves POLICY_VERSION_STALE and its reason: ${JSON.stringify({ decision, policyVersionCheck })}`);
  }

  const responsive = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await changeRegion.evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        left: rect.left, right: rect.right };
    });
    check(geometry.scroll <= geometry.viewport && geometry.left >= 0 && geometry.right <= geometry.viewport,
      `Policy change evidence fits at ${width}px: ${JSON.stringify(geometry)}`);
    responsive.push(width);
    if (width === 390) {
      await changeRegion.scrollIntoViewIfNeeded();
      await page.screenshot({ path: 'docs/implementation/evidence/s33/risk-policy-invalidation-390.png' });
    }
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  const draftLibrary = await command('trade.draft.list', { workspaceId });
  const firstDraftIndex = draftLibrary.drafts.findIndex(item => item.draftId === first.proposal.draftId);
  check(firstDraftIndex >= 0, 'The invalidated proposal draft remains in workspace history.');
  await page.locator('.order-draft-row').nth(firstDraftIndex).click();
  await page.locator('.order-proposal-row').filter({ hasText: first.proposal.proposalId }).click();
  const reviewAction = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await reviewAction.waitFor({ state: 'visible' });
  await page.waitForFunction(() => [...document.querySelectorAll('button')]
    .find(button => button.textContent?.trim() === 'Review Live approval')?.disabled === true);
  check(await reviewAction.isDisabled(), 'The invalidated proposal cannot request a replacement approval.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'No Prepare action remains for the invalidated proposal.');

  const afterLog = await commandLog(beforeLog.cursor);
  const forbidden = afterLog.commands.filter(entry => {
    const name = entry.command;
    return /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(name)
      || name === 'trade.execution.prepare'
      || name === 'trade.manual_resolution'
      || name.startsWith('live.gateway.');
  });
  check(forbidden.length === 0, `No provider management, execution, or Gateway command ran: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `No external HTTP request was observed: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0, `No browser console errors: ${JSON.stringify(consoleErrors)}`);
  check(pageErrors.length === 0, `No browser page errors: ${JSON.stringify(pageErrors)}`);

  return {
    result: 'PASS_LIMITED', screenId: 'F12', scope: change.scope.kind,
    policyVersions: [change.oldPolicyVersion, change.newPolicyVersion], weakened: change.weakened,
    affectedAccounts: change.affectedAccounts.length, affectedLiveAccountsDisarmed: accounts.length,
    invalidatedProposals: change.affectedProposals.length,
    priorApprovalsInvalidated: true, affectedAccountsDisarmed: true,
    responsiveWidths: responsive, forbiddenCommands: forbidden.length,
    externalHttpRequests: externalRequests.length, consoleErrors: consoleErrors.length, pageErrors: pageErrors.length,
  };
}
