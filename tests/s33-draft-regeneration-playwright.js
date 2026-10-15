// Run with `playwright-cli run-code --filename=tests/s33-draft-regeneration-playwright.js`
// Build the isolated IPC first: cargo build --manifest-path src-tauri/Cargo.toml --features integration-test --bin tradex-ipc
// Start port 1424: TRADEX_LIVE_APPROVAL_FIXTURE=1 node_modules/.bin/vite --host 127.0.0.1 --mode integration --port 1424
// Each run creates a disposable workspace; no provider is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const pageErrors = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-draft-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.requestId === requestId && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body?.error)}`);
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), after);

  const runScenario = async allowConsent => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto('http://127.0.0.1:1424/');
    await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
      .waitFor({ state: 'visible' });
    const cursor = await page.evaluate(async () =>
      (await fetch('/__integration/command-log?after=0')).json().then(log => log.cursor));
    const bootstrap = await command('workspace.open', {});
    const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const workspaceName = `S33 draft regeneration ${stamp}`;
    const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-draft-${stamp}`;
    const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
    const workspaceId = workspace.workspaceId;
    await command('workspace.ready.fixture', { workspaceId });
    await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
    await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
    await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
    await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
    await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
    if (allowConsent) {
      // The optional limit is configured through the UI in this disposable workspace only.
      await page.getByRole('button', { name: 'Edit setup', exact: true }).click();
      await page.getByRole('heading', { name: 'Risk defaults', exact: true }).waitFor({ state: 'visible' });
      await page.getByRole('textbox', { name: 'Maximum single-instrument exposure (%)', exact: true }).fill('');
      await page.getByRole('button', { name: 'Save and continue', exact: true }).click();
      await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
    }
    await page.getByRole('button', { name: 'Complete onboarding', exact: true }).click();
    await page.getByRole('button', { name: 'Complete onboarding', exact: true }).waitFor({ state: 'detached' });
    const account = await command('account.arming.fixture.seed', {
      workspaceId, providerId: 'trading212', label: `S33 draft edit ${stamp}`,
    });
    await command('time.revalidate', { workspaceId });
    const armed = await command('account.arm', {
      workspaceId, connectionId: account.connectionId,
      expectedStateVersion: account.stateVersion, confirmed: true,
    });
    check(armed.health.arming === 'ARMED', `The synthetic account must arm before review: ${JSON.stringify(armed.health)}`);

    const navigation = page.getByRole('navigation', { name: 'Primary navigation' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    const quantity = page.getByRole('textbox', { name: 'Quantity', exact: true });
    await quantity.fill('0.01');
    await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await page.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('510');
    await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await page.getByRole('button', { name: 'Save draft', exact: true }).click();
    await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await page.getByRole('button', { name: 'Generate proposal', exact: true }).click();
    await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });

    const firstList = await command('trade.proposal.list', { workspaceId });
    check(firstList.proposals.length === 1, `Expected one initial Proposal: ${JSON.stringify(firstList.proposals)}`);
    const original = await command('trade.proposal.get', {
      workspaceId, proposalId: firstList.proposals[0].proposalId,
    });
    check(original.status === 'NEEDS_APPROVAL' && original.fields.quantity.value === '0.01' && original.draftVersion === 1,
      `Initial Proposal must freeze version 1 / quantity 1: ${JSON.stringify(original)}`);

    await page.locator('.order-proposal-row').first().press('Enter');
    const review = page.getByRole('button', { name: 'Review Live approval', exact: true });
    await review.press('Enter');
    const reviewDialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await reviewDialog.waitFor({ state: 'visible' });
    const approve = reviewDialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
    if (allowConsent) {
      check(await approve.isEnabled(), 'The explicitly configured synthetic policy permits local consent.');
      await approve.press('Enter');
      check((await command('trade.approval.list', { workspaceId, proposalId: original.proposalId })).approvals.length === 0,
        'Enter on Approve must not implicitly issue financial consent.');
      await approve.press('Space');
      await reviewDialog.waitFor({ state: 'hidden' });
    } else {
      check(await approve.isDisabled()
        && (await reviewDialog.innerText()).includes('Single-instrument exposure percent evidence is unavailable'),
      'The default policy remains fail-closed without trusted portfolio exposure evidence.');
      await reviewDialog.press('Escape');
      await reviewDialog.waitFor({ state: 'hidden' });
    }
    const approvalBeforeEdit = await command('trade.approval.list', { workspaceId, proposalId: original.proposalId });
    check(allowConsent
      ? approvalBeforeEdit.approvals.length === 1 && approvalBeforeEdit.approvals[0].status === 'ISSUED'
      : approvalBeforeEdit.approvals.length === 0,
    'Only explicit consent in the configured synthetic case creates an issued approval.');

    await quantity.fill('0.009');
    await page.getByRole('button', { name: 'Save draft', exact: true }).click();
    await page.getByRole('status').filter({ hasText: 'Draft saved at version 2.' }).waitFor({ state: 'visible' });
    const invalidated = await command('trade.proposal.get', {
      workspaceId, proposalId: original.proposalId,
    });
    check(invalidated.status === 'INVALIDATED' && invalidated.fields.quantity.value === '0.01'
      && invalidated.history.some(event => event.event === 'DRAFT_CHANGED'),
    `Editing the draft must preserve and invalidate its original snapshot: ${JSON.stringify(invalidated)}`);
    const approvalAfterEdit = await command('trade.approval.list', { workspaceId, proposalId: original.proposalId });
    check(allowConsent
      ? approvalAfterEdit.approvals.length === 1
        && approvalAfterEdit.approvals[0].approvalId === approvalBeforeEdit.approvals[0].approvalId
        && approvalAfterEdit.approvals[0].status === 'INVALIDATED'
        && approvalAfterEdit.approvals[0].invalidationReason === 'PROPOSAL_CHANGED'
      : approvalAfterEdit.approvals.length === 0,
    'Editing invalidates the exact old approval or preserves the empty risk-blocked history.');

    await page.getByRole('button', { name: 'Generate proposal', exact: true }).click();
    await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const proposals = await command('trade.proposal.list', { workspaceId });
    check(proposals.proposals.length === 2, `Regeneration must append a new Proposal: ${JSON.stringify(proposals.proposals)}`);
    const regeneratedSummary = proposals.proposals.find(item => item.proposalId !== original.proposalId);
    check(Boolean(regeneratedSummary), 'Regenerated Proposal must have a new immutable identity.');
    const regenerated = await command('trade.proposal.get', {
      workspaceId, proposalId: regeneratedSummary.proposalId,
    });
    check(regenerated.status === 'NEEDS_APPROVAL' && regenerated.fields.quantity.value === '0.009' && regenerated.draftVersion === 2,
      `Regenerated Proposal must freeze version 2 / quantity 2: ${JSON.stringify(regenerated)}`);
    check(regenerated.proposalId !== original.proposalId && regenerated.proposalHash !== original.proposalHash,
      'Regeneration must produce a distinct proposal identity and content hash.');
    const originalAfterRegeneration = await command('trade.proposal.get', {
      workspaceId, proposalId: original.proposalId,
    });
    check(originalAfterRegeneration.status === 'INVALIDATED'
      && originalAfterRegeneration.fields.quantity.value === '0.01'
      && originalAfterRegeneration.proposalHash === original.proposalHash,
    'Regeneration must preserve the old Proposal snapshot and invalidation state.');

    const regeneratedApprovals = await command('trade.approval.list', { workspaceId, proposalId: regenerated.proposalId });
    check(regeneratedApprovals.approvals.length === 0,
      'A regenerated Proposal has no approval inherited from the prior identity.');
    await page.locator('.order-proposal-row').filter({ hasText: original.proposalId }).press('Enter');
    await page.locator('.proposal-read-only').filter({ hasText: original.proposalId })
      .locator('.proposal-meta').getByText('INVALIDATED', { exact: true }).waitFor({ state: 'visible' });
    const oldReview = page.getByRole('button', { name: 'Review Live approval', exact: true });
    check(await oldReview.isDisabled(), 'An invalidated Proposal cannot start a new review.');
    if (allowConsent) {
      await page.locator('.live-approval-record').filter({ hasText: approvalBeforeEdit.approvals[0].approvalId })
        .filter({ hasText: 'INVALIDATED' }).filter({ hasText: 'PROPOSAL_CHANGED' }).waitFor({ state: 'visible' });
    }
    check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
      'The invalidated Proposal exposes no Prepare action.');
    await page.locator('.order-proposal-row').filter({ hasText: regenerated.proposalId }).press('Enter');
    await page.locator('.proposal-read-only').filter({ hasText: regenerated.proposalId })
      .locator('.proposal-meta').getByText('NEEDS_APPROVAL', { exact: true }).waitFor({ state: 'visible' });
    const newReview = page.getByRole('button', { name: 'Review Live approval', exact: true });
    check(await newReview.isEnabled(), 'The replacement Proposal exposes its own review path.');

    const responsive = [];
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
      const geometry = await page.evaluate(() => {
        const panel = document.querySelector('.order-proposal-panel')?.getBoundingClientRect();
        return {
          width: document.documentElement.clientWidth,
          scroll: document.documentElement.scrollWidth,
          panel: panel ? { left: panel.left, right: panel.right } : null,
        };
      });
      check(geometry.scroll <= geometry.width, `The proposal editor has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
      check(Boolean(geometry.panel) && geometry.panel.left >= 0 && geometry.panel.right <= geometry.width,
        `The immutable proposal panel stays in view at ${width}px: ${JSON.stringify(geometry)}`);
      responsive.push(width);
      if (width === 390) await page.screenshot({ path: `docs/implementation/evidence/s33/qa07-${allowConsent ? 'consent' : 'draft'}-regeneration-390.png` });
    }
    await newReview.press('Enter');
    const newReviewDialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await newReviewDialog.waitFor({ state: 'visible' });
    const newApprove = newReviewDialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
    await newReviewDialog.screenshot({ path: `docs/implementation/evidence/s33/qa07-${allowConsent ? 'fresh-consent' : 'blocked-review'}-390.png` });
    if (allowConsent) {
      check(await newApprove.isEnabled(), 'The replacement requires its own explicit, eligible review.');
      await newApprove.press('Space');
      await newReviewDialog.waitFor({ state: 'hidden' });
    } else {
      check(await newApprove.isDisabled()
        && (await newReviewDialog.innerText()).includes('Single-instrument exposure percent evidence is unavailable'),
      'The default policy also blocks the replacement review without exposure evidence.');
      await newReviewDialog.press('Escape');
      await newReviewDialog.waitFor({ state: 'hidden' });
    }
    const finalApprovals = await command('trade.approval.list', { workspaceId, proposalId: regenerated.proposalId });
    check(allowConsent
      ? finalApprovals.approvals.length === 1 && finalApprovals.approvals[0].status === 'ISSUED'
        && finalApprovals.approvals[0].approvalId !== approvalBeforeEdit.approvals[0].approvalId
        && finalApprovals.approvals[0].proposalHash === regenerated.proposalHash
      : finalApprovals.approvals.length === 0,
    'Only fresh explicit consent can issue a distinct approval bound to the regenerated hash.');

    const log = await commandLog(cursor);
    const forbiddenCommands = log.commands.filter(entry =>
      /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh|data\..*\.(probe|refresh)|model\.verify_route|trade\.(reject|execution\.prepare|cancel\.)|paper\.order\.submit|live\.gateway\.(submit|fixture\.set_result))/.test(entry.command));
    check(forbiddenCommands.length === 0,
      `Draft regeneration must not call a provider or prepare/execute a financial action: ${JSON.stringify(forbiddenCommands)}`);
    check(log.commands.filter(entry => entry.command === 'trade.request_approval').length === 2,
      'Only the original and regenerated Proposals receive local review requests.');
    check(log.commands.filter(entry => entry.command === 'trade.approve').length === (allowConsent ? 2 : 0),
      'The configured case has exactly two explicit local approvals; the default case has zero.');
    check(externalRequests.length === 0, `No external HTTP requests: ${JSON.stringify(externalRequests)}`);
    check(pageErrors.length === 0, `No browser errors: ${JSON.stringify(pageErrors)}`);
    return {
      result: 'PASS_LIMITED',
      scenario: allowConsent ? 'OPTIONAL_EXPOSURE_LIMIT_UNSET' : 'DEFAULT_EXPOSURE_GUARD',
      workspaceId,
      originalProposal: { proposalId: original.proposalId, status: invalidated.status,
        quantity: original.fields.quantity.value, approvalBeforeEdit: approvalBeforeEdit.approvals.length,
        approvalAfterEdit: approvalAfterEdit.approvals.length },
      regeneratedProposal: { proposalId: regenerated.proposalId, status: regenerated.status, draftVersion: regenerated.draftVersion, quantity: regenerated.fields.quantity.value },
      replacementApprovals: regeneratedApprovals.approvals.length,
      oldApprovalStatus: approvalAfterEdit.approvals[0]?.status ?? 'NONE',
      newApprovalStatus: finalApprovals.approvals[0]?.status ?? 'NONE',
      approvalBlockedBy: allowConsent ? null : 'SINGLE_INSTRUMENT_EXPOSURE',
      responsiveWidths: responsive,
      forbiddenCommands: forbiddenCommands.length,
      externalRequests: externalRequests.length,
      pageErrors: pageErrors.length,
    };
  };
  const guarded = await runScenario(false);
  const consent = await runScenario(true);
  return { result: 'PASS_LIMITED', guarded, consent, externalRequests: externalRequests.length, pageErrors: pageErrors.length };
}
