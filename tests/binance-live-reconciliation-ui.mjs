import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';

export async function checkBinanceLiveReconciliationUI(tab, browser) {
  const ui = tab.playwright;
  const commandEndpoint = new URL('/__integration/command', await tab.url()).toString();
  const send = async (command, payload) => {
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
  };
  const viewport = await browser.capabilities.get('viewport');
  try {
    await viewport.set({ width: 1280, height: 900 });
    const bootstrap = await send('workspace.open', {});
    const workspaceName = 'S25.2 Binance Live reconciliation UI';
    const workspacePath = join(dirname(bootstrap.path), `binance-reconciliation-${randomUUID()}`);
    const workspace = await send('workspace.open', {
      name: workspaceName, baseCurrency: 'USD', path: workspacePath,
    });
    await send('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
    if (!(await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible())) {
      await ui.getByRole('button', { name: 'Workspace', exact: true }).press('Enter');
    }
    await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
    await ui.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
    await ui.getByLabel('Local storage', { exact: true }).fill(workspacePath);
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

    const workspaceId = workspace.workspaceId;
    const policy = await send('risk.get_policy', { workspaceId });
    await send('risk.save_policy', {
      workspaceId,
      expectedStateVersion: policy.stateVersion,
      policy: { ...policy.policy, staleQuoteThresholdSeconds: 120, maxSingleInstrumentExposurePercent: null },
    });
    const account = await send('binance.live.reconciliation.fixture.account.seed', {
      workspaceId, label: `Binance reconciliation ${Date.now()}`,
    });
    assert.equal(account.providerId, 'binance');
    assert.equal(account.environment, 'LIVE');
    await send('time.revalidate', { workspaceId });
    const armed = await send('account.arm', {
      workspaceId, connectionId: account.connectionId,
      expectedStateVersion: account.stateVersion, confirmed: true,
    });
    assert.equal(armed.health.arming, 'ARMED');

    const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).first();
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    const proposals = await send('trade.proposal.list', { workspaceId });
    const proposal = await send('trade.proposal.get', {
      workspaceId, proposalId: proposals.proposals[0].proposalId,
    });
    await ui.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).click();
    const review = await send('trade.request_approval', { workspaceId, proposalId: proposal.proposalId });
    assert.equal(review.eligible, true, JSON.stringify(review));
    const approval = await send('trade.approve', {
      workspaceId,
      proposalId: proposal.proposalId,
      proposalHash: review.proposal.proposalHash,
      reviewedRiskDecisionId: review.riskDecision.decisionId,
      reviewDigest: review.reviewDigest,
      expectedStateVersion: review.proposal.stateVersion,
    });
    const seeded = await send('binance.live.reconciliation.fixture.attempt.seed', {
      workspaceId,
      proposalId: proposal.proposalId,
      approvalId: approval.approvalId,
      capacityProjection: review.capacityProjection,
    });
    const attemptId = seeded.attempt.attemptId;
    const providerClientOrderId = seeded.attempt.providerClientOrderId;
    assert.equal(providerClientOrderId, `tx-${attemptId.replaceAll('-', '')}`);
    assert.equal(seeded.attempt.state, 'UNKNOWN_RECONCILING');
    await ui.getByRole('button', { name: 'Reload history', exact: true }).press('Enter');
    await ui.getByText(/UNKNOWN_RECONCILING · TradeX execution attempt/, { exact: false }).waitFor({ state: 'visible' });

    const evidence = ui.getByLabel('Live reconciliation evidence', { exact: true });
    await evidence.getByRole('heading', { name: 'Binance Live reconciliation evidence', exact: true }).waitFor({ state: 'visible' });
    await evidence.getByText('Similar orders observed; candidates only', { exact: true }).waitFor({ state: 'visible' });
    await evidence.getByText(`Provider client order ID ${providerClientOrderId}`, { exact: true }).waitFor({ state: 'visible' });
    await evidence.getByText('Exact-order lookup complete', { exact: true }).waitFor({ state: 'visible' });
    await evidence.getByText(/Candidate only · provider order 987654321/).waitFor({ state: 'visible' });

    const preparation = await send('trade.execution.preparation.get', {
      workspaceId, approvalId: approval.approvalId,
    });
    assert.equal(preparation.preparation.attempt.state, 'UNKNOWN_RECONCILING');
    assert.equal(preparation.preparation.reservation.status, 'ACTIVE');
    const saved = await send('trade.resolution_evidence', {
      workspaceId, executionAttemptId: attemptId, accountId: account.connectionId,
    });
    assert.equal(saved.ledger.providerId, 'binance');
    assert.equal(saved.ledger.historyPagesRead, 0);
    assert.equal(saved.ledger.evidence.at(-1).outcome, 'CANDIDATES_FOUND');
    assert.equal(saved.ledger.evidence.at(-1).candidateOrders[0].providerClientId, providerClientOrderId);
    assert.equal(saved.ledger.evidence.at(-1).candidateOrders[0].providerStatus, 'NEW');
    assert.equal((await send('binance.live.reconciliation.fixture.inspect', {})).providerOrderWrites, 0);

    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: width === 390 ? 844 : 900 });
      const size = await evidence.evaluate(element => ({
        left: element.getBoundingClientRect().left,
        right: element.getBoundingClientRect().right,
        width: window.innerWidth,
        documentScroll: document.documentElement.scrollWidth,
        panelScroll: element.scrollWidth,
        panelClient: element.clientWidth,
      }));
      assert.ok(size.left >= 0 && size.right <= size.width, `Evidence panel fits at ${width}px: ${JSON.stringify(size)}`);
      assert.ok(size.documentScroll <= size.width, `Evidence view has no horizontal overflow at ${width}px: ${JSON.stringify(size)}`);
      assert.ok(size.panelScroll <= size.panelClient + 1, `Evidence panel has no local overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    const pageErrors = await tab.dev.logs({ levels: ['error'], limit: 20 });
    assert.equal(pageErrors.filter(log => !log.url?.startsWith('chrome-extension://') && !log.message.includes('chrome-extension://')).length, 0);
    return { attemptId, candidateClientOrderId: providerClientOrderId, providerOrderWrites: 0 };
  } finally {
    await viewport.reset();
  }
}
