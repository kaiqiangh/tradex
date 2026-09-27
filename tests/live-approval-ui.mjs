// Run with a Codex CUA browser binding against `npm run dev:browser`.
// All Live account, quote, and proposal data is synthetic integration-fixture state.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';

let commandEndpoint = 'http://127.0.0.1:1420/__integration/command';

async function sendIntegrationCommand(command, payload) {
  const requestId = randomUUID();
  const response = await fetch(commandEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Origin: new URL(commandEndpoint).origin },
    body: JSON.stringify({ requestId, schemaVersion: 1, command, payload }),
  });
  assert.equal(response.status, 200, `${command} is available in the Rust integration bridge`);
  const result = await response.json();
  assert.equal(result.requestId, requestId);
  assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
  return result.data;
}

async function expectFocus(ui, expected) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (await ui.evaluate(() => document.activeElement?.textContent?.trim()) === expected) return;
    await ui.waitForTimeout(50);
  }
  assert.fail(`Expected focus on ${expected}`);
}

export async function checkLiveApprovalUI(tab, browser) {
  const ui = tab.playwright;
  commandEndpoint = new URL('/__integration/command', await tab.url()).toString();
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    assert.ok(await ui.evaluate(() => window.innerWidth >= 1280), 'Desktop viewport override applies');
    const bootstrap = await sendIntegrationCommand('workspace.open', {});
    const workspaceName = 'S23 Live Approval UI';
    const workspacePath = join(dirname(bootstrap.path), `live-approval-${randomUUID()}`);
    const workspace = await sendIntegrationCommand('workspace.open', {
      name: workspaceName, baseCurrency: 'USD', path: workspacePath,
    });
    await sendIntegrationCommand('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
    if (!(await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible())) {
      await ui.getByRole('button', { name: 'Workspace', exact: true }).press('Enter');
    }
    await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
    await ui.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
    await ui.getByLabel('Local storage', { exact: true }).fill(workspacePath);
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
    const workspaceId = workspace.workspaceId;
    await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).first()
      .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    const identity = await ui.getByRole('complementary', { name: 'Workspace', exact: true }).innerText();
    assert.ok(identity.includes(workspaceId), `The isolated workspace identity is visible: ${identity}`);

    const currentRisk = await sendIntegrationCommand('risk.get_policy', { workspaceId });
    await sendIntegrationCommand('risk.save_policy', {
      workspaceId,
      expectedStateVersion: currentRisk.stateVersion,
      policy: { ...currentRisk.policy, marketOrdersEnabled: true, maxMarketOrderSlippagePercent: '5', staleQuoteThresholdSeconds: 120, maxSingleInstrumentExposurePercent: null },
    });
    const account = await sendIntegrationCommand('account.arming.fixture.seed', {
      workspaceId, providerId: 'trading212', label: `Approval fixture ${Date.now()}`,
    });
    const time = await sendIntegrationCommand('time.revalidate', { workspaceId });
    assert.equal(time.confidence, 'TRUSTED');
    const armed = await sendIntegrationCommand('account.arm', {
      workspaceId,
      connectionId: account.connectionId,
      expectedStateVersion: account.stateVersion,
      confirmed: true,
    });
    assert.equal(armed.health.arming, 'ARMED');
    const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).first();
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });

    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('MARKET');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
    await ui.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('500');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const marketLibrary = await sendIntegrationCommand('trade.proposal.list', { workspaceId });
    assert.equal(marketLibrary.proposals.length, 1, 'The isolated workspace contains only this generated proposal');
    const marketProposal = marketLibrary.proposals[0];
    await ui.locator('.order-proposal-row').first().press('Enter');

    const reviewButton = ui.getByRole('button', { name: 'Review Live approval', exact: true });
    await reviewButton.press('Enter');
    const dialog = ui.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await dialog.waitFor({ state: 'visible' });
    const reviewText = await dialog.innerText();
    for (const expected of ['TRADING212_LIVE', account.label, 'equity:US:AAPL', 'BUY', '0.01 BASE', 'MARKET · No limit · GTC', 'Expected spend\n500.01', 'Maximum authorized spend\n500', 'SYNTHETIC_INTEGRATION_FIXTURE', '49999 / 50001 / 2', 'TRADABLE', 'Estimated fees\nUnavailable', 'Estimated slippage\nUnavailable']) {
      assert.ok(reviewText.includes(expected), `Approval review includes ${expected}: ${reviewText}`);
    }
    await ui.getByRole('alert').filter({ hasText: 'MARKET_MAXIMUM_AUTHORIZATION_EXCEEDED' }).waitFor({ state: 'visible' });
    await ui.getByRole('alert').filter({ hasText: 'MarketOrderSlippage' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).isEnabled(), false);
    const marketHistory = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: marketProposal.proposalId });
    assert.equal(marketHistory.approvals.length, 0, 'Unavailable size-aware slippage keeps market approval absent');
    assert.match(reviewText, /Quote age\n\d+ ms/);
    await expectFocus(ui, 'Reject this review');
    await viewport.set({ width: 390, height: 844 });
    const dialogSize = await dialog.evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
    });
    assert.ok(dialogSize.left >= 0 && dialogSize.right <= dialogSize.width, `Approval review fits on mobile: ${JSON.stringify(dialogSize)}`);
    assert.ok(dialogSize.scroll <= dialogSize.width, `Approval review has no horizontal overflow: ${JSON.stringify(dialogSize)}`);
    await dialog.press('Escape');
    await dialog.waitFor({ state: 'hidden' });
    await expectFocus(ui, 'Review Live approval');
    observed.push('Market BUY review shows ask-derived expected spend, the 500 cap and quote provenance; unavailable size-aware slippage blocks approval, and Escape cancels safely in a 390px viewport.');

    const armedAccount = (await sendIntegrationCommand('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
    assert.ok(armedAccount);
    const disarmedForApproval = await sendIntegrationCommand('account.disarm', {
      workspaceId, connectionId: account.connectionId, expectedStateVersion: armedAccount.stateVersion,
    });
    assert.equal(disarmedForApproval.health.arming, 'DISARMED');
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await ui.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('510');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const proposalLibrary = await sendIntegrationCommand('trade.proposal.list', { workspaceId });
    assert.equal(proposalLibrary.proposals.length, 2);
    const proposalRow = ui.locator('.order-proposal-row').first();
    const selectedProposalId = (await proposalRow.innerText()).match(/proposal:[0-9a-f-]{36}/)?.[0];
    assert.ok(selectedProposalId, 'The visible limit proposal has a canonical identity');
    const selectedProposal = proposalLibrary.proposals.find(item => item.proposalId === selectedProposalId);
    assert.ok(selectedProposal, 'The visible limit proposal is present in backend history');
    await proposalRow.press('Enter');
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const armDialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await armDialog.waitFor({ state: 'visible' });
    const armText = await armDialog.innerText();
    for (const expected of [account.providerId, account.label, 'LIVE', account.connectionId, account.data.remoteAccountId]) {
      assert.ok(armText.includes(expected), `Arm confirmation identifies ${expected}: ${armText}`);
    }
    const armDialogSize = await armDialog.evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
    });
    assert.ok(armDialogSize.left >= 0 && armDialogSize.right <= armDialogSize.width, `Arm confirmation fits on mobile: ${JSON.stringify(armDialogSize)}`);
    assert.ok(armDialogSize.scroll <= armDialogSize.width, `Arm confirmation has no horizontal overflow: ${JSON.stringify(armDialogSize)}`);
    assert.equal(await ui.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), true);
    const beforeArm = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: selectedProposal.proposalId });
    assert.equal(beforeArm.approvals.length, 0, 'Opening an Arm confirmation does not issue approval');
    await ui.getByRole('button', { name: 'Arm this Live account', exact: true }).press('Enter');
    await armDialog.waitFor({ state: 'hidden' });
    await reviewButton.waitFor({ state: 'visible' });
    await dialog.waitFor({ state: 'visible' });
    const armedReviewText = await dialog.innerText();
    assert.ok(armedReviewText.includes(selectedProposal.proposalId), 'Arming returns to the same proposal');
    assert.ok(armedReviewText.includes(selectedProposal.proposalHash), 'Arming returns to the same proposal hash');
    assert.ok(armedReviewText.includes(account.connectionId), 'Arming retains the exact selected account');
    observed.push('A DISARMED Live PLACE opens an exact Arm confirmation; explicit Arm returns to the same proposal, hash and account for a fresh review without issuing approval.');
    const beforeExplicitReview = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: selectedProposal.proposalId });
    assert.equal(beforeExplicitReview.approvals.length, 0, 'Arming and opening the fresh review do not issue approval');
    await ui.getByRole('status').filter({ hasText: 'Current checks pass. Approval still requires your explicit action.' }).waitFor({ state: 'visible' });

    await expectFocus(ui, 'Reject this review');
    await ui.getByRole('button', { name: 'Reject this review', exact: true }).press('Enter');
    await dialog.waitFor({ state: 'hidden' });
    await ui.getByText(/REJECTED · USER_REJECTED/, { exact: false }).waitFor({ state: 'visible' });
    observed.push('Enter on the review trigger focuses the safe Reject action; explicit Enter records a rejection without an order.');

    await reviewButton.click();
    await dialog.waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Reject this review', exact: true }).press('Tab');
    assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Approve for up to 30 seconds');
    const approveButton = ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
    await approveButton.press('Enter');
    await dialog.waitFor({ state: 'visible' });
    const beforeExplicitAction = await sendIntegrationCommand('trade.approval.list', {
      workspaceId,
      proposalId: selectedProposal.proposalId,
    });
    assert.equal(beforeExplicitAction.approvals.length, 0, 'Enter does not issue an approval');
    await approveButton.press('Space');
    await dialog.waitFor({ state: 'hidden' });
    await ui.getByText(/ISSUED · PLACE_ORDER/, { exact: false }).waitFor({ state: 'visible' });
    const issuedHistory = await sendIntegrationCommand('trade.approval.list', {
      workspaceId,
      proposalId: selectedProposal.proposalId,
    });
    assert.equal(issuedHistory.approvals[0]?.status, 'ISSUED', 'Approving does not consume the approval or reserve capacity');
    await ui.getByRole('button', { name: 'Prepare PLACE and reserve capacity', exact: true }).waitFor({ state: 'visible' });
    assert.match(await ui.getByRole('status').innerText(), /No order was placed/);
    observed.push('Enter is inert on Approve; Space explicitly issues the approval, while preparation remains a separate action and no order has been placed or reserved.');

    const accounts = await sendIntegrationCommand('account.list', { workspaceId });
    const currentAccount = accounts.accounts.find(item => item.connectionId === account.connectionId);
    const disarmed = await sendIntegrationCommand('account.disarm', {
      workspaceId,
      connectionId: account.connectionId,
      expectedStateVersion: currentAccount.stateVersion,
    });
    assert.equal(disarmed.health.arming, 'DISARMED');
    await ui.getByRole('button', { name: 'Reload history', exact: true }).press('Enter');
    await ui.getByText(/INVALIDATED · PLACE_ORDER/, { exact: false }).waitFor({ state: 'visible' });
    observed.push('Disarming the selected Live account invalidates its issued approval when history is re-read.');

    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('SELL');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('MARKET');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
    await ui.getByRole('textbox', { name: 'Maximum sale value (optional)', exact: true }).fill('510');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const sellLibrary = await sendIntegrationCommand('trade.proposal.list', { workspaceId });
    const sellProposalId = (await ui.locator('.order-proposal-row').first().innerText()).match(/proposal:[0-9a-f-]{36}/)?.[0];
    assert.ok(sellProposalId);
    const sellProposal = sellLibrary.proposals.find(item => item.proposalId === sellProposalId);
    assert.ok(sellProposal, 'The visible market SELL proposal is present in backend history');
    await ui.locator('.order-proposal-row').first().press('Enter');
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const changedAccountArmDialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await changedAccountArmDialog.waitFor({ state: 'visible' });
    const disarmedAccount = (await sendIntegrationCommand('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
    assert.ok(disarmedAccount);
    const concurrentArm = await sendIntegrationCommand('account.arm', {
      workspaceId, connectionId: account.connectionId, expectedStateVersion: disarmedAccount.stateVersion, confirmed: true,
    });
    assert.equal(concurrentArm.health.arming, 'ARMED');
    await ui.getByRole('button', { name: 'Arm this Live account', exact: true }).press('Enter');
    await changedAccountArmDialog.waitFor({ state: 'hidden' });
    await ui.getByRole('status').filter({ hasText: 'Account state changed while confirming. No Arm or approval was issued' }).waitFor({ state: 'visible' });
    const sellHistory = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: sellProposal.proposalId });
    assert.equal(sellHistory.approvals.length, 0, 'A concurrent account change cannot issue approval');
    observed.push('A concurrent account-state change during Arm confirmation fails closed and leaves the disarmed proposal without approval.');

    const rearmedAccount = (await sendIntegrationCommand('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
    assert.ok(rearmedAccount);
    const disarmedForChangedProposal = await sendIntegrationCommand('account.disarm', {
      workspaceId, connectionId: account.connectionId, expectedStateVersion: rearmedAccount.stateVersion,
    });
    assert.equal(disarmedForChangedProposal.health.arming, 'DISARMED');
    await reviewButton.press('Enter');
    const changedProposalArmDialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await changedProposalArmDialog.waitFor({ state: 'visible' });
    const approvalHistoryBeforeRefresh = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: sellProposal.proposalId });
    assert.equal(approvalHistoryBeforeRefresh.approvals.length, 0);
    const proposalBeforeRefresh = await sendIntegrationCommand('trade.proposal.get', { workspaceId, proposalId: sellProposal.proposalId });
    const refreshedProposal = await sendIntegrationCommand('trade.refresh_proposal', {
      workspaceId,
      proposalId: proposalBeforeRefresh.proposalId,
      expectedStateVersion: proposalBeforeRefresh.stateVersion,
    });
    assert.notEqual(refreshedProposal.proposal.proposalId, sellProposal.proposalId, 'Refreshing changes the proposal identity');
    await ui.getByRole('button', { name: 'Arm this Live account', exact: true }).press('Enter');
    await changedProposalArmDialog.waitFor({ state: 'hidden' });
    await ui.getByRole('status').filter({ hasText: 'The proposal changed while confirming. No account was armed' }).waitFor({ state: 'visible' });
    const newProposalHistory = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: refreshedProposal.proposal.proposalId });
    assert.equal(newProposalHistory.approvals.length, 0, 'A changed proposal cannot inherit or receive approval');
    const stillDisarmed = (await sendIntegrationCommand('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
    assert.equal(stillDisarmed?.health.arming, 'DISARMED', 'A changed proposal is rejected before Account Arm');
    observed.push('A proposal refreshed during Arm confirmation fails closed; the changed proposal stays DISARMED and neither the original nor replacement proposal receives approval.');

    const latestAccount = (await sendIntegrationCommand('account.list', { workspaceId })).accounts.find(item => item.connectionId === account.connectionId);
    assert.ok(latestAccount);
    const readyAccount = await sendIntegrationCommand('account.arm', {
      workspaceId, connectionId: account.connectionId, expectedStateVersion: latestAccount.stateVersion, confirmed: true,
    });
    assert.equal(readyAccount.health.arming, 'ARMED');
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.008');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await ui.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('410');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    let preparedProposal;
    for (const summary of (await sendIntegrationCommand('trade.proposal.list', { workspaceId })).proposals) {
      const detail = await sendIntegrationCommand('trade.proposal.get', { workspaceId, proposalId: summary.proposalId });
      if (detail.fields.quantity.value === '0.008') { preparedProposal = detail; break; }
    }
    assert.ok(preparedProposal, 'The explicit-reservation proposal was persisted');
    await ui.locator('.order-proposal-row').filter({ hasText: preparedProposal.proposalId }).press('Enter');
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const prepareReview = ui.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await prepareReview.waitFor({ state: 'visible' });
    await ui.getByRole('status').filter({ hasText: 'Current checks pass. Approval still requires your explicit action.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).press('Space');
    await prepareReview.waitFor({ state: 'hidden' });
    await ui.getByRole('button', { name: 'Prepare PLACE and reserve capacity', exact: true }).waitFor({ state: 'visible' });
    const preparationButton = ui.getByRole('button', { name: 'Prepare PLACE and reserve capacity', exact: true });
    await preparationButton.press('Enter');
    await ui.getByRole('status').filter({ hasText: 'TradeX reserved 400 USD; no provider request was sent.' }).waitFor({ state: 'visible' });
    await ui.getByText(/RESERVED · TradeX capacity reservation/, { exact: false }).waitFor({ state: 'visible' });
    await ui.getByText(/No order request was sent to the provider\./, { exact: false }).waitFor({ state: 'visible' });
    const liveReviewButton = ui.getByRole('button', { name: 'Review Live approval', exact: true });
    for (let attempt = 0; attempt < 40 && await liveReviewButton.isEnabled(); attempt += 1) await ui.waitForTimeout(50);
    assert.equal(await liveReviewButton.isEnabled(), false, 'A consumed proposal cannot open a second live review');
    const preparedHistory = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: preparedProposal.proposalId });
    assert.equal(preparedHistory.approvals[0]?.status, 'CONSUMED', 'The explicit preparation consumes the exact approval');
    const durablePreparation = await sendIntegrationCommand('trade.execution.preparation.get', {
      workspaceId,
      approvalId: preparedHistory.approvals[0].approvalId,
    });
    assert.ok(durablePreparation.preparation?.attempt.attemptId, 'The saved attempt is queryable without its original idempotency key');
    const consumedProposal = await sendIntegrationCommand('trade.proposal.get', { workspaceId, proposalId: preparedProposal.proposalId });
    assert.equal(consumedProposal.status, 'CONSUMED', 'The same transaction consumes the exact immutable proposal');
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.locator('.order-proposal-row').filter({ hasText: preparedProposal.proposalId }).press('Enter');
    const recoveredCard = ui.getByLabel('TradeX execution preparation', { exact: true });
    await recoveredCard.waitFor({ state: 'visible' });
    assert.ok((await recoveredCard.innerText()).includes(durablePreparation.preparation.attempt.attemptId), 'Reopening Order Drafts restores the persisted preparation');
    observed.push('A second explicit action consumes the issued approval, shows a durable RESERVED attempt and exact capacity, and confirms no provider request was sent.');
    observed.push('Reopening Order Drafts restores the preparation through its approval ID without renderer-held idempotency state.');

    const pageErrors = await tab.dev.logs({ levels: ['error'], limit: 20 });
    assert.equal(pageErrors.filter(log => !log.url?.startsWith('chrome-extension://') && !log.message.includes('chrome-extension://')).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
