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

async function selectDraft(ui, workspaceId, draftId, expectedQuantity) {
  const drafts = await sendIntegrationCommand('trade.draft.list', { workspaceId });
  const index = drafts.drafts.findIndex(draft => draft.draftId === draftId);
  assert.ok(index >= 0, `Draft ${draftId} is present in workspace history`);
  await ui.locator('.order-draft-row').nth(index).click();
  const quantity = () => ui.evaluate(() => [...document.querySelectorAll('input[inputmode="decimal"]')][0]?.value);
  for (let attempt = 0; attempt < 40 && await quantity() !== expectedQuantity; attempt += 1) await ui.waitForTimeout(50);
  assert.equal(await quantity(), expectedQuantity, 'Selecting the draft loads its saved quantity');
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
    for (const expected of ['TRADING212_LIVE', account.label, 'equity:US:AAPL', 'BUY', '0.01 BASE', 'MARKET · No limit · GTC', 'Expected spend\n500.01', 'Maximum authorized spend\n500', 'Backend capacity preview', 'Provider available\n1000 USD', 'Provider committed\n0 USD', 'TradeX reserved\n0 USD', 'Effective available\n1000 USD', 'SYNTHETIC_INTEGRATION_FIXTURE', '49999 / 50001 / 2', 'TRADABLE', 'Estimated fees\nUnavailable', 'Estimated slippage\nUnavailable']) {
      assert.ok(reviewText.includes(expected), `Approval review includes ${expected}: ${reviewText}`);
    }
    await ui.getByRole('alert').filter({ hasText: 'MARKET_MAXIMUM_AUTHORIZATION_EXCEEDED' }).waitFor({ state: 'visible' });
    await ui.getByRole('alert').filter({ hasText: 'MarketOrderSlippage' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).isEnabled(), false);
    const marketHistory = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: marketProposal.proposalId });
    assert.equal(marketHistory.approvals.length, 0, 'Unavailable size-aware slippage keeps market approval absent');
    assert.match(reviewText, /Quote age\n\d+ ms/);
    await expectFocus(ui, 'Reject this review');
    await viewport.set({ width: 768, height: 900 });
    const tabletDialogSize = await dialog.evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
    });
    assert.ok(tabletDialogSize.left >= 0 && tabletDialogSize.right <= tabletDialogSize.width, `Approval review fits at 768px: ${JSON.stringify(tabletDialogSize)}`);
    assert.ok(tabletDialogSize.scroll <= tabletDialogSize.width, `Approval review has no horizontal overflow at 768px: ${JSON.stringify(tabletDialogSize)}`);
    await viewport.set({ width: 390, height: 844 });
    const dialogSize = await dialog.evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
    });
    assert.ok(dialogSize.left >= 0 && dialogSize.right <= dialogSize.width, `Approval review fits on mobile: ${JSON.stringify(dialogSize)}`);
    assert.ok(dialogSize.scroll <= dialogSize.width, `Approval review has no horizontal overflow: ${JSON.stringify(dialogSize)}`);
    await dialog.press('Escape');
    await dialog.waitFor({ state: 'hidden' });
    await viewport.set({ width: 1280, height: 900 });
    await expectFocus(ui, 'Review Live approval');
    observed.push('Market BUY review shows backend available/committed/reserved/effective capacity with quote provenance; unavailable size-aware slippage blocks approval, and the review fits at 1280/768/390px.');

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
    await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).waitFor({ state: 'visible' });
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
    const executionAccount = await sendIntegrationCommand('account.arming.fixture.seed', {
      workspaceId, providerId: 'trading212', label: `Gateway dispatch fixture ${Date.now()}`,
    });
    await sendIntegrationCommand('time.revalidate', { workspaceId });
    const executionArm = await sendIntegrationCommand('account.arm', {
      workspaceId, connectionId: executionAccount.connectionId,
      expectedStateVersion: executionAccount.stateVersion, confirmed: true,
    });
    assert.equal(executionArm.health.arming, 'ARMED');
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(executionAccount.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.008');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
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
    await selectDraft(ui, workspaceId, preparedProposal.draftId, '0.008');
    const preparedProposalRow = ui.locator('.order-proposal-row').filter({ hasText: preparedProposal.proposalId });
    await preparedProposalRow.waitFor({ state: 'visible' });
    await preparedProposalRow.click();
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const prepareReview = ui.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await prepareReview.waitFor({ state: 'visible' });
    await ui.getByRole('status').filter({ hasText: 'Current checks pass. Approval still requires your explicit action.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).press('Space');
    await prepareReview.waitFor({ state: 'hidden' });
    await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).waitFor({ state: 'visible' });
    const preparationButton = ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true });
    await sendIntegrationCommand('live.gateway.fixture.set_result', { workspaceId, scenario: 'ACCEPTED' });
    await preparationButton.press('Enter');
    await ui.getByText(/ACCEPTED · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
    const acceptedCard = ui.getByLabel('TradeX execution preparation', { exact: true });
    await acceptedCard.getByRole('status').filter({ hasText: 'The provider accepted the order. This is not a fill.' }).waitFor({ state: 'visible' });
    await acceptedCard.getByText('Provider order ID 901', { exact: true }).waitFor({ state: 'visible' });
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
    assert.equal(durablePreparation.preparation.attempt.state, 'ACCEPTED');
    assert.equal(durablePreparation.preparation.attempt.brokerOrderId, '901');
    assert.equal(durablePreparation.preparation.attempt.providerStatus, 'NEW');
    assert.equal(durablePreparation.preparation.reservation.status, 'ACTIVE', 'Provider acceptance does not release capacity before provider evidence does.');
    const gatewayRequests = (await sendIntegrationCommand('live.gateway.fixture.inspect', {})).requests;
    const placeRequests = gatewayRequests.filter(request => request.method === 'POST');
    assert.equal(placeRequests.length, 1, 'The approved attempt causes exactly one provider mutation.');
    assert.equal(placeRequests[0].path, '/api/v0/equity/orders/limit');
    assert.deepStrictEqual(JSON.parse(JSON.stringify(placeRequests[0].body)), {
      ticker: 'AAPL_US_EQ', quantity: 0.008, limitPrice: 50000, timeValidity: 'GOOD_TILL_CANCEL',
    });
    assert.equal(placeRequests[0].authorizationPresent, true, 'The synthetic provider receives credentials only in its authorization header.');
    const consumedProposal = await sendIntegrationCommand('trade.proposal.get', { workspaceId, proposalId: preparedProposal.proposalId });
    assert.equal(consumedProposal.status, 'CONSUMED', 'The same transaction consumes the exact immutable proposal');
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await selectDraft(ui, workspaceId, preparedProposal.draftId, '0.008');
    await ui.locator('.order-proposal-row').filter({ hasText: preparedProposal.proposalId }).click();
    const recoveredCard = ui.getByLabel('TradeX execution preparation', { exact: true });
    await recoveredCard.waitFor({ state: 'visible' });
    assert.ok((await recoveredCard.innerText()).includes(durablePreparation.preparation.attempt.attemptId), 'Reopening Order Drafts restores the persisted preparation');
    assert.match(await recoveredCard.innerText(), /ACCEPTED · TradeX execution attempt/);
    assert.match(await recoveredCard.innerText(), /Provider order ID 901/);
    assert.ok((await recoveredCard.innerText()).includes('Capacity after TradeX reservation'), 'The recovered reservation restores its backend capacity projection');
    observed.push('The explicit Prepare action consumes the approval, dispatches one exact fake-provider POST, and persists ACCEPTED with provider order 901 while keeping capacity held.');
    observed.push('Reopening Order Drafts restores the preparation through its approval ID without renderer-held idempotency state.');

    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(executionAccount.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.014');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    let conflictProposal;
    for (const summary of (await sendIntegrationCommand('trade.proposal.list', { workspaceId })).proposals) {
      const detail = await sendIntegrationCommand('trade.proposal.get', { workspaceId, proposalId: summary.proposalId });
      if (detail.fields.quantity.value === '0.014' && detail.fields.side === 'BUY') { conflictProposal = detail; break; }
    }
    assert.ok(conflictProposal, 'The F11 conflict proposal is persisted');
    await ui.locator('.order-proposal-row').filter({ hasText: conflictProposal.proposalId }).press('Enter');
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const conflictDialog = ui.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await conflictDialog.waitFor({ state: 'visible' });
    const conflictText = await conflictDialog.innerText();
    for (const expected of ['Provider available\n1000 USD', 'Provider committed\n0 USD', 'TradeX reserved\n400 USD', 'Effective available\n600 USD', 'Requested capacity\n700 USD', 'CURRENT']) {
      assert.ok(conflictText.includes(expected), `F11 preview includes ${expected}: ${conflictText}`);
    }
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: width === 390 ? 844 : 900 });
      const size = await conflictDialog.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
      });
      assert.ok(size.left >= 0 && size.right <= size.width, `F11 review fits at ${width}px: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `F11 review has no horizontal overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    await expectFocus(ui, 'Reject this review');
    await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).press('Space');
    await conflictDialog.waitFor({ state: 'hidden' });
    await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).press('Enter');
    await ui.getByRole('alert').filter({ hasText: 'RISK_REJECTED · RESERVED_CAPACITY' }).waitFor({ state: 'visible' });
    await ui.getByText(/Next step: Reduce the request or wait for earlier reservations/).waitFor({ state: 'visible' });
    const refusalSummary = ui.getByLabel('Capacity evidence at refusal', { exact: true });
    await refusalSummary.waitFor({ state: 'visible' });
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: width === 390 ? 844 : 900 });
      const size = await refusalSummary.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
      });
      assert.ok(size.left >= 0 && size.right <= size.width, `F11 refusal evidence fits at ${width}px: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `F11 refusal evidence has no horizontal overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    await viewport.set({ width: 1280, height: 900 });
    const conflictHistory = await sendIntegrationCommand('trade.approval.list', { workspaceId, proposalId: conflictProposal.proposalId });
    assert.equal(conflictHistory.approvals[0]?.status, 'ISSUED', 'Capacity refusal preserves the approval for a deliberate retry');
    assert.equal((await sendIntegrationCommand('trade.proposal.get', { workspaceId, proposalId: conflictProposal.proposalId })).status, 'NEEDS_APPROVAL');
    const conflictPreparation = await sendIntegrationCommand('trade.execution.preparation.get', {
      workspaceId, approvalId: conflictHistory.approvals[0].approvalId,
    });
    assert.equal(conflictPreparation.preparation, null, 'Over-limit preparation creates no execution attempt');
    assert.equal(conflictPreparation.rejections[0].capacityContext.effectiveAvailable, '600');
    assert.equal(conflictPreparation.rejections[0].capacityContext.remediation, 'REDUCE_REQUEST_OR_WAIT_FOR_RESERVATIONS');
    assert.equal(await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).isEnabled(), true);
    observed.push('F11 shows $1,000 provider capacity, $400 reserved by TradeX, and $600 effective before a $700 request; refusal evidence fits at 1280/768/390px, preserves the approval, records remediation, and creates no attempt.');

    const staleAccount = await sendIntegrationCommand('account.capacity.fixture.stale', {
      workspaceId, connectionId: account.connectionId,
    });
    assert.equal(staleAccount.health.reconciliation, 'CURRENT');
    assert.equal(staleAccount.health.arming, 'ARMED', 'Stale capacity evidence does not silently alter account arming');
    const drafts = await sendIntegrationCommand('trade.draft.list', { workspaceId });
    const marketDraftIndex = drafts.drafts.findIndex(draft => draft.draftId === marketProposal.draftId);
    assert.ok(marketDraftIndex >= 0, 'The stale-preview proposal still has a saved draft');
    await ui.locator('.order-draft-row').nth(marketDraftIndex).press('Enter');
    const marketProposalRow = ui.locator('.order-proposal-row').filter({ hasText: marketProposal.proposalId });
    await marketProposalRow.waitFor({ state: 'visible' });
    await marketProposalRow.click();
    const staleReviewButton = ui.getByRole('button', { name: 'Review Live approval', exact: true });
    await staleReviewButton.waitFor({ state: 'visible' });
    await staleReviewButton.press('Enter');
    const staleDialog = ui.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await staleDialog.waitFor({ state: 'visible' });
    const staleSummary = ui.getByLabel('Backend capacity preview', { exact: true });
    await staleSummary.waitFor({ state: 'visible' });
    const staleText = await staleSummary.innerText();
    for (const expected of [
      'Provider available\nUnavailable', 'Provider committed\nUnavailable',
      'TradeX reserved\nUnavailable', 'Effective available\nUnavailable',
      'Account evidence\nSTALE ·',
      'Account evidence is stale; refresh the account before relying on these values.',
    ]) assert.ok(staleText.includes(expected), `Stale capacity is explicit and hides amounts: ${staleText}`);
    assert.equal(await staleSummary.getByRole('alert').getAttribute('aria-live'), 'polite');
    assert.equal(await staleDialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).isEnabled(), false);
    await expectFocus(ui, 'Reject this review');
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: width === 390 ? 844 : 900 });
      const size = await staleDialog.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth };
      });
      assert.ok(size.left >= 0 && size.right <= size.width, `F11 stale review fits at ${width}px: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `F11 stale review has no horizontal overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    await staleDialog.press('Escape');
    await staleDialog.waitFor({ state: 'hidden' });
    await viewport.set({ width: 1280, height: 900 });
    observed.push('F11 stale capacity review hides every amount, announces the stale state to assistive technology, blocks approval, and fits at 1280/768/390px.');

    await selectDraft(ui, workspaceId, preparedProposal.draftId, '0.008');
    await ui.locator('.order-proposal-row').filter({ hasText: preparedProposal.proposalId }).click();
    const preparationCard = ui.getByLabel('TradeX execution preparation', { exact: true });
    await ui.getByText(/ACCEPTED · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
    const policy = await sendIntegrationCommand('risk.get_policy', { workspaceId });
    const stoppedPolicy = { ...policy.policy, staleQuoteThresholdSeconds: Math.max(1, policy.policy.staleQuoteThresholdSeconds - 1) };
    await sendIntegrationCommand('risk.save_policy', {
      workspaceId, expectedStateVersion: policy.stateVersion, policy: stoppedPolicy,
    });
    const acceptedAfterPolicyChange = await sendIntegrationCommand('trade.execution.preparation.get', {
      workspaceId, approvalId: preparedHistory.approvals[0].approvalId,
    });
    assert.equal(acceptedAfterPolicyChange.preparation.attempt.state, 'ACCEPTED', 'A post-dispatch policy update cannot claim it prevented the provider request.');
    assert.equal(acceptedAfterPolicyChange.preparation.reservation.status, 'ACTIVE');
    assert.equal(((await sendIntegrationCommand('live.gateway.fixture.inspect', {})).requests.filter(request => request.method === 'POST')).length, 1);
    assert.match(await preparationCard.innerText(), /ACCEPTED · TradeX execution attempt/);
    observed.push('A risk-policy update after provider acceptance preserves the persisted ACCEPTED attempt and its active capacity reservation.');

    const mountedAccount = await sendIntegrationCommand('account.arming.fixture.seed', {
      workspaceId, providerId: 'trading212', label: `Mounted refresh fixture ${Date.now()}`,
    });
    await sendIntegrationCommand('time.revalidate', { workspaceId });
    const armedMountedAccount = await sendIntegrationCommand('account.arm', {
      workspaceId, connectionId: mountedAccount.connectionId,
      expectedStateVersion: mountedAccount.stateVersion, confirmed: true,
    });
    assert.equal(armedMountedAccount.health.arming, 'ARMED');
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(mountedAccount.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    let mountedProposal;
    for (const summary of (await sendIntegrationCommand('trade.proposal.list', { workspaceId })).proposals) {
      const detail = await sendIntegrationCommand('trade.proposal.get', { workspaceId, proposalId: summary.proposalId });
      if (detail.fields.quantity.value === '0.001') { mountedProposal = detail; break; }
    }
    assert.ok(mountedProposal, 'The mounted-refresh proposal was persisted');
    await selectDraft(ui, workspaceId, mountedProposal.draftId, '0.001');
    await ui.locator('.order-proposal-row').filter({ hasText: mountedProposal.proposalId }).click();
    const mountedReview = await sendIntegrationCommand('trade.request_approval', {
      workspaceId, proposalId: mountedProposal.proposalId,
    });
    assert.equal(mountedReview.eligible, true, 'The mounted-refresh proposal passes its explicit review');
    const mountedApproval = await sendIntegrationCommand('trade.approve', {
      workspaceId,
      proposalId: mountedProposal.proposalId,
      proposalHash: mountedReview.proposal.proposalHash,
      reviewedRiskDecisionId: mountedReview.riskDecision.decisionId,
      reviewDigest: mountedReview.reviewDigest,
      expectedStateVersion: mountedReview.proposal.stateVersion,
    });
    assert.equal(mountedApproval.status, 'ISSUED');
    await ui.getByRole('button', { name: 'Reload history', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).waitFor({ state: 'visible' });
    await sendIntegrationCommand('live.gateway.fixture.set_result', { workspaceId, scenario: 'UNKNOWN' });
    await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).press('Enter');
    await ui.getByText(/UNKNOWN_RECONCILING · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
    const unknownCard = ui.getByLabel('TradeX execution preparation', { exact: true });
    await unknownCard.getByRole('status').filter({ hasText: 'The provider outcome is uncertain. Capacity remains held; do not resend this request.' }).waitFor({ state: 'visible' });
    const evidencePanel = ui.getByLabel('Trading 212 Live reconciliation evidence', { exact: true });
    await evidencePanel.getByText('No similar order observed; absence is not proven', { exact: true }).waitFor({ state: 'visible' });
    const mountedPolicy = await sendIntegrationCommand('risk.get_policy', { workspaceId });
    await sendIntegrationCommand('risk.save_policy', {
      workspaceId,
      expectedStateVersion: mountedPolicy.stateVersion,
      policy: { ...mountedPolicy.policy, staleQuoteThresholdSeconds: Math.max(1, mountedPolicy.policy.staleQuoteThresholdSeconds - 1) },
    });
    const unknownPreparation = await sendIntegrationCommand('trade.execution.preparation.get', {
      workspaceId, approvalId: mountedApproval.approvalId,
    });
    assert.equal(unknownPreparation.preparation.attempt.state, 'UNKNOWN_RECONCILING');
    assert.equal(unknownPreparation.preparation.reservation.status, 'ACTIVE', 'Unknown provider outcome keeps capacity frozen after a policy update.');
    const persistedEvidence = await sendIntegrationCommand('trade.resolution_evidence', {
      workspaceId,
      executionAttemptId: unknownPreparation.preparation.attempt.attemptId,
      accountId: account.connectionId,
    });
    assert.equal(persistedEvidence.ledger.evidence.at(-1).outcome, 'INCONCLUSIVE');
    assert.deepEqual(persistedEvidence.ledger.evidence.at(-1).candidateOrders, []);
    assert.match(persistedEvidence.ledger.evidence.at(-1).queryScope, /AAPL/);
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: width === 390 ? 844 : 900 });
      await evidencePanel.getByText('No similar order observed; absence is not proven', { exact: true }).waitFor({ state: 'visible' });
      const size = await evidencePanel.evaluate(element => ({
        left: element.getBoundingClientRect().left,
        right: element.getBoundingClientRect().right,
        width: window.innerWidth,
        documentScroll: document.documentElement.scrollWidth,
        panelScroll: element.scrollWidth,
        panelClient: element.clientWidth,
      }));
      assert.ok(size.left >= 0 && size.right <= size.width, `Evidence panel fits at ${width}px: ${JSON.stringify(size)}`);
      assert.ok(size.documentScroll <= size.width, `Evidence view has no page overflow at ${width}px: ${JSON.stringify(size)}`);
      assert.ok(size.panelScroll <= size.panelClient + 1, `Evidence panel has no local overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    await viewport.set({ width: 1280, height: 900 });
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await evidencePanel.getByText('No similar order observed; absence is not proven', { exact: true }).waitFor({ state: 'visible' });
    const reopenedPreparation = await sendIntegrationCommand('trade.execution.preparation.get', {
      workspaceId, approvalId: mountedApproval.approvalId,
    });
    assert.equal(reopenedPreparation.preparation.attempt.state, 'UNKNOWN_RECONCILING');
    assert.equal(reopenedPreparation.preparation.reservation.status, 'ACTIVE', 'Leaving and reopening the evidence surface preserves frozen capacity.');
    assert.equal(((await sendIntegrationCommand('live.gateway.fixture.inspect', {})).requests.filter(request => request.method === 'POST')).length, 2, 'Unknown outcome is never resent.');
    const mountedStoppedText = await unknownCard.innerText();
    assert.match(mountedStoppedText, /UNKNOWN_RECONCILING · TradeX execution attempt/);
    assert.match(mountedStoppedText, /Capacity remains held; do not resend this request/);
    assert.equal(await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count(), 0, 'An uncertain consumed approval exposes no resend action.');
    observed.push('An ambiguous provider outcome remains UNKNOWN_RECONCILING with active capacity after policy change and navigation; inconclusive evidence is accessible at 1280/768/390px and cannot be resent.');

    const rejectedAccount = await sendIntegrationCommand('account.arming.fixture.seed', {
      workspaceId, providerId: 'trading212', label: `Rejected gateway fixture ${Date.now()}`,
    });
    await sendIntegrationCommand('time.revalidate', { workspaceId });
    const armedRejectedAccount = await sendIntegrationCommand('account.arm', {
      workspaceId, connectionId: rejectedAccount.connectionId,
      expectedStateVersion: rejectedAccount.stateVersion, confirmed: true,
    });
    assert.equal(armedRejectedAccount.health.arming, 'ARMED');
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(rejectedAccount.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    const proposalsBeforeRejection = new Set((await sendIntegrationCommand('trade.proposal.list', { workspaceId })).proposals.map(item => item.proposalId));
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const rejectedProposal = (await sendIntegrationCommand('trade.proposal.list', { workspaceId })).proposals.find(item => !proposalsBeforeRejection.has(item.proposalId));
    assert.ok(rejectedProposal, 'The definitive-rejection proposal was persisted');
    await ui.locator('.order-proposal-row').filter({ hasText: rejectedProposal.proposalId }).click();
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const rejectedDialog = ui.getByRole('dialog', { name: 'Review Live approval', exact: true });
    await rejectedDialog.waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).press('Space');
    await rejectedDialog.waitFor({ state: 'hidden' });
    await sendIntegrationCommand('live.gateway.fixture.set_result', { workspaceId, scenario: 'REJECTED' });
    await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).press('Enter');
    const rejectedCard = ui.getByLabel('TradeX execution preparation', { exact: true });
    await rejectedCard.getByText(/REJECTED · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });
    await rejectedCard.getByRole('status').filter({ hasText: 'The provider rejected the order. TradeX released its reservation.' }).waitFor({ state: 'visible' });
    const rejectedApproval = (await sendIntegrationCommand('trade.approval.list', {
      workspaceId, proposalId: rejectedProposal.proposalId,
    })).approvals[0];
    const rejectedPreparation = await sendIntegrationCommand('trade.execution.preparation.get', {
      workspaceId, approvalId: rejectedApproval.approvalId,
    });
    assert.equal(rejectedPreparation.preparation.attempt.state, 'REJECTED');
    assert.equal(rejectedPreparation.preparation.reservation.status, 'RELEASED');
    assert.equal(((await sendIntegrationCommand('live.gateway.fixture.inspect', {})).requests.filter(request => request.method === 'POST').length), 3, 'Each explicit attempt causes one POST; rejection is not replayed.');
    assert.equal(await ui.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count(), 0, 'A rejected consumed approval exposes no resend action.');
    observed.push('A definitive provider rejection is persisted, releases only its reservation, displays accurate status, and cannot be resent.');

    const pageErrors = await tab.dev.logs({ levels: ['error'], limit: 20 });
    assert.equal(pageErrors.filter(log => !log.url?.startsWith('chrome-extension://') && !log.message.includes('chrome-extension://')).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
