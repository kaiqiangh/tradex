// Real React/typed Rust/disposable workspace; only provider HTTP and public market
// HTTP/WS producers are fixtures. This reads current evidence and never arms or sends.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { checkSpotOwningAdmissionUI, waitForUIText } from './spot-owning-admission-ui.mjs';

const LABEL = 'Captured Proposal Spot fee and required FX';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const assertJsonEqual = (actual, expected, message) => assert.equal(JSON.stringify(actual), JSON.stringify(expected), message);

export async function checkSpotDiscountedBnbEstimateUI(tab, browser, { stopAfterCurrentPreview = false } = {}) {
  const state = await checkSpotOwningAdmissionUI(tab, browser, { staleQuoteThresholdSeconds: 30 }), ui = tab.playwright;
  const payload = { workspaceId: state.workspaceId, proposalId: state.proposalId };
  const capacity = ui.getByRole('region', { name: 'Current Proposal Spot capacity inputs', exact: true });
  const commission = ui.getByRole('region', { name: 'Current Proposal symbol commission terms', exact: true });

  // The shared setup selects the public market source through its visible Settings action
  // before reading any account-bound observations. Keep that generation for every binding.
  await state.send('binance.live.commission.fixture', { scenario: 'BNB_SUFFICIENT' });
  const configuredMarket = state.marketSelection;
  await state.refreshSource('COMPLETE_COVERAGE');

  const quote = ui.getByRole('region', { name: 'Trade Hot quote', exact: true });
  const initialMarket = await state.send('market.get', {
    workspaceId: state.workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT',
  });
  const initialTradeLease = initialMarket.snapshot?.provenance?.binance?.leaseId;
  await quote.getByRole('button', { name: 'Retry Proposal quote source', exact: true }).press('Enter');
  let currentInitialMarket;
  const initialQuoteDeadline = Date.now() + 10000;
  do {
    await pause(50);
    currentInitialMarket = await state.send('market.get', {
      workspaceId: state.workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT',
    });
  } while ((currentInitialMarket.status !== 'AVAILABLE'
    || !currentInitialMarket.snapshot?.provenance?.binance?.leaseId
    || currentInitialMarket.snapshot.provenance.binance.leaseId === initialTradeLease)
    && Date.now() < initialQuoteDeadline);
  assert.equal(currentInitialMarket.status, 'AVAILABLE', 'The selected Proposal reacquires its own fresh quote lease');
  await waitForUIText(quote, configuredMarket.stateVersion);
  await waitForUIText(quote, 'STREAMING');
  await waitForUIText(quote, '60000 / 60001');
  assert.ok((await quote.innerText()).includes(configuredMarket.stateVersion), 'The displayed continuous book must belong to the configured source generation');

  // The explicit account refresh follows the final market/rule-source generation,
  // binding the actual BNB balance to the unchanged Proposal and current Rules.
  const oldCapacity = await state.send('trade.spot_capacity.get', payload);
  await capacity.getByRole('button', { name: 'Refresh required capacity inputs', exact: true }).press('Enter');
  let currentCapacity;
  const capacityDeadline = Date.now() + 5000;
  do {
    await pause(50);
    currentCapacity = await state.send('trade.spot_capacity.get', payload);
  } while (currentCapacity.observation?.collectionId === oldCapacity.observation?.collectionId && Date.now() < capacityDeadline);
  assert.notEqual(currentCapacity.observation?.collectionId, oldCapacity.observation?.collectionId, 'The BNB balance must come from a new explicit account read');
  await waitForUIText(capacity, 'observed · Read-only inputs · Execution qualification unavailable');

  const oldCommission = await state.send('trade.spot_commission.get', payload);
  await commission.getByRole('button', { name: 'Read symbol commission terms', exact: true }).press('Enter');
  let currentCommission;
  const commissionDeadline = Date.now() + 5000;
  do {
    await pause(50);
    currentCommission = await state.send('trade.spot_commission.get', payload);
  } while (currentCommission.stateVersion === oldCommission.stateVersion && Date.now() < commissionDeadline);
  assert.notEqual(currentCommission.stateVersion, oldCommission.stateVersion, 'The displayed estimate must bind the latest explicit commission read');
  await waitForUIText(commission, 'observed · Read-only declaration · Execution qualification unavailable');

  const feeSource = ui.getByRole('region', { name: 'Current BNB fee source', exact: true });
  await feeSource.getByRole('button', { name: 'Read BNB fee source', exact: true }).press('Enter');
  await waitForUIText(feeSource, 'STREAMING · Read-only fee inputs · Execution qualification unavailable');
  await waitForUIText(feeSource, '500 / 501');

  await state.evaluate();
  const panel = ui.getByRole('region', { name: LABEL, exact: true }).first();
  const review = await state.send('trade.request_approval', payload);
  assert.equal(review.eligible, false, 'Source and current fee evidence cannot create execution authority');
  assert.equal(review.riskDecision.status, 'UNAVAILABLE', 'The complete Live review remains fail-closed');
  assert.equal(review.riskDecision.spotCommission.status, 'OBSERVED', 'The exact-symbol commission read is bound to this review');
  assert.equal(review.spotFeeFx.reasonCode, 'SPOT_REQUIRED_FX_EVIDENCE_UNAVAILABLE', 'The bounded USDT-to-USD route remains blocked until a selected-location quote is read');
  assert.equal(review.spotFeeFx.feeCurrency, 'UNKNOWN', 'The actual fee asset remains unknown until settlement');
  assert.equal(review.riskDecision.checks.find(check => check.checkId === 'MARKET_DATA_USE')?.outcome, 'UNAVAILABLE', 'User-specific source rights stay an independent blocker');
  const wallet = review.spotFeeFx.walletFundingEvidence;
  assert.equal(wallet.status, 'COMPLETE');
  assertJsonEqual(wallet.requiredAssets, ['BTC', 'USDT', 'BNB']);
  assertJsonEqual(wallet.missingAssets, []);
  assert.equal(wallet.collectionId, currentCapacity.observation.collectionId);
  assert.equal(wallet.originalDigest, currentCapacity.observation.reads.find(read => read.kind === 'ACCOUNT').digest);
  assertJsonEqual(wallet.balances.map(row => [row.asset, row.free, row.locked, row.localClaims]), [
    ['BTC', '0.10000000', '0.02000000', '0'],
    ['USDT', '99999999999999999999.9999999999999999999', '0.0000000000000000002', '0'],
    ['BNB', '1.00000000', '0.00000000', '0'],
  ]);
  const model = review.spotFeeFx.currentFeeModel;
  assert.equal(model.chargingCondition, 'DISCOUNTED_BNB_CURRENT_ESTIMATE');
  assert.equal(model.bnbFundingEvidence.free, '1.00000000');
  assert.equal(model.bnbFee.total, '0.00600042340539');
  const plan = model.fundingPlanPreview;
  assert.equal(plan.principalClaim.asset, 'USDT');
  assert.equal(plan.principalClaim.amount, '60');
  assert.equal(plan.feeDisposition, 'BNB_CURRENT_ESTIMATE');
  assert.equal(plan.currentFeeClaim.asset, 'BNB');
  assert.equal(plan.currentFeeClaim.amount, '0.00600042340539');
  assert.equal(plan.fallbackFeeAsset, 'BTC');
  assert.equal(plan.fallbackFeeAmount, '0.00005000282');
  assert.equal(plan.fallbackNetReceipt, '0.00094999718');
  assertJsonEqual(model.bnbConversionReferences.map(reference => [reference.pair, reference.priceSide]), [['BTCUSDT', 'ASK'], ['BNBUSDT', 'BID']]);
  const text = await panel.innerText();
  assert.match(text, /ACTUAL_FEE_ASSET_PENDING/);
  assert.match(text, /Conditional discounted BNB current estimate/);
  assert.match(text, /discounted bnb current estimate/);
  assert.match(text, /Complete modeled BNB fee/);
  assert.match(text, /BASE \/ QUOTE \/ BNB wallet coverage/);
  assert.match(text, /Coverage complete/);
  assert.match(text, /BTC free \/ locked/);
  assert.match(text, /effective free 1\.00000000 BNB/);
  assert.ok(text.includes(model.bnbFee.total));
  assert.ok(text.includes('BTCUSDT · BTC -> USDT at ASK'));
  assert.ok(text.includes('BNBUSDT · USDT -> BNB at BID'));
  assert.match(text, /BNB_INTERNAL_AVERAGE_EQUIVALENCE_UNPROVEN/);
  assert.match(text, /Native principal claim\s+60 USDT/);
  assert.match(text, /Current fee disposition\s+bnb current estimate · 0\.00600042340539 BNB outgoing fee claim/);
  assert.match(text, /Received-asset fallback fee and net receipt\s+0\.00005000282 BTC · 0\.00094999718 BTC/);
  assert.match(text, /native claims are created only by atomic Prepare after explicit approval/);
  assert.equal(await panel.getByRole('button').count(), 0, 'The saved funding preview exposes no execution or refresh action');


  const viewport = await browser.capabilities.get('viewport'), widths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await panel.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.viewport, width);
      assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
      widths.push(size);
    }
  } finally { await viewport.reset(); }

  if (stopAfterCurrentPreview) {
    const approvals = await state.send('trade.approval.list', payload);
    assertJsonEqual(approvals.approvals, []);
    assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
    assert.equal(await panel.innerText(), text, 'Reading the preview cannot renew or rewrite its saved financial material');
    return { model, text, widths, wallet, sourceAndCapacityWereExplicitlyRead: true,
      approvalEligible: false, completeFinancialAcceptance: false, armingPerformed: false,
      consentPerformed: false, syntheticSupportContextCreated: false };
  }

  // This disposable profile uses a synthetic user-reported Support fact solely to exercise the
  // policy-evaluator branch. The issue permits synthetic external producers only, so this branch
  // is excluded from AC acceptance evidence and must not be treated as the user's real report.
  const testedFxAccount = await state.send('provider.connect', {
    step: 'test', workspaceId: state.workspaceId, providerId: 'alpaca', environment: 'PAPER',
    label: `S29.11 synthetic policy valuation ${randomUUID()}`,
  });
  const fxAccount = await state.send('provider.connect', {
    step: 'confirm', workspaceId: state.workspaceId, connectionId: testedFxAccount.connectionId,
    expectedStateVersion: testedFxAccount.stateVersion, acknowledgeUnverified: true,
  });
  const fxSource = await state.send('data.fx.connection', { workspaceId: state.workspaceId });
  const configuredFx = await state.send('data.fx.configure', {
    workspaceId: state.workspaceId, expectedStateVersion: fxSource.stateVersion,
    connectionId: fxAccount.connectionId, cryptoLocation: 'eu-1',
    sourceUseContext: {
      marketDataPlan: 'BASIC', accountJurisdiction: 'IE', agreementReviewedAt: '2026-10-12',
      countrySupportConfirmation: {
        confirmedJurisdiction: 'IE', confirmedAt: '2026-10-12',
        reference: `SYNTHETIC-SUPPORT-${randomUUID()}`,
      },
      useScope: 'OWN_LOCAL_NONCOMMERCIAL', rawDataDestination: 'LOCAL_ONLY',
    },
  });
  assert.equal(configuredFx.sourceUseAssessment.status, 'SUPPORTED_FOR_DECLARED_SCOPE');
  assert.equal(configuredFx.sourceUseAssessment.provenance, 'USER_REPORTED_PROVIDER_SUPPORT');
  assertJsonEqual(configuredFx.sourceUseAssessment.supportedPurposes, ['POLICY_VALUATION']);
  assert.equal(configuredFx.capabilityStatuses.find(item => item.capability === 'FX_DATA_USE_RIGHTS')?.status, 'UNVERIFIED');

  const currencyEvidence = ui.getByRole('region', { name: 'Currency evidence', exact: true });
  // Rebind the exact rules and account observations after the source setup and baseline layout
  // checks. Their independent freshness window is intentionally enforced by the real review.
  const preFxCapacity = currentCapacity;
  const preFxCommission = currentCommission;
  await state.refreshSource('COMPLETE_COVERAGE');
  await waitForUIText(capacity, 'not observed · Read-only inputs · Execution qualification unavailable');
  await capacity.getByRole('button', { name: 'Refresh required capacity inputs', exact: true }).press('Enter');
  const finalCapacityDeadline = Date.now() + 5000;
  do {
    await pause(50);
    currentCapacity = await state.send('trade.spot_capacity.get', payload);
  } while (currentCapacity.observation?.collectionId === preFxCapacity.observation?.collectionId && Date.now() < finalCapacityDeadline);
  assert.notEqual(currentCapacity.observation?.collectionId, preFxCapacity.observation?.collectionId, 'The positive review uses a new explicit balance and capacity read');
  await waitForUIText(capacity, 'observed · Read-only inputs · Execution qualification unavailable');

  await waitForUIText(commission, 'not observed · Read-only declaration · Execution qualification unavailable');
  await commission.getByRole('button', { name: 'Read symbol commission terms', exact: true }).press('Enter');
  const finalCommissionDeadline = Date.now() + 5000;
  do {
    await pause(50);
    currentCommission = await state.send('trade.spot_commission.get', payload);
  } while (currentCommission.stateVersion === preFxCommission.stateVersion && Date.now() < finalCommissionDeadline);
  assert.notEqual(currentCommission.stateVersion, preFxCommission.stateVersion, 'The positive review uses a new exact-symbol commission read');
  await waitForUIText(commission, 'observed · Read-only declaration · Execution qualification unavailable');

  const feeLeaseId = value => /Lease \/ connection generation\s+([0-9a-f-]+)/.exec(value)?.[1];
  const previousFeeLease = feeLeaseId(await feeSource.innerText());
  assert.ok(previousFeeLease, 'The earlier fee-source read must have a visible lease identity');
  await feeSource.getByRole('button', { name: 'Read BNB fee source', exact: true }).press('Enter');
  const feeLeaseDeadline = Date.now() + 5000;
  let refreshedFeeText = await feeSource.innerText();
  while (feeLeaseId(refreshedFeeText) === previousFeeLease && Date.now() < feeLeaseDeadline) {
    await pause(50);
    refreshedFeeText = await feeSource.innerText();
  }
  assert.notEqual(feeLeaseId(refreshedFeeText), previousFeeLease, 'The positive review uses a newly acquired public BNB fee-source lease');
  await waitForUIText(feeSource, 'STREAMING · Read-only fee inputs · Execution qualification unavailable');
  await waitForUIText(feeSource, '500 / 501');

  // The BNB estimate also consumes the Proposal's received-asset quote. The preceding
  // source-use and account-input reads can outlast that quote's configured freshness window,
  // so reacquire its actual UI-owned lease before reading FX last.
  const previousMarket = await state.send('market.get', {
    workspaceId: state.workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT',
  });
  const previousTradeLease = previousMarket.snapshot?.provenance?.binance?.leaseId;
  await quote.getByRole('button', { name: 'Retry Proposal quote source', exact: true }).press('Enter');
  let currentMarket;
  const tradeQuoteDeadline = Date.now() + 5000;
  do {
    await pause(50);
    currentMarket = await state.send('market.get', {
      workspaceId: state.workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT',
    });
  } while ((currentMarket.status !== 'AVAILABLE'
    || !currentMarket.snapshot?.provenance?.binance?.leaseId
    || currentMarket.snapshot.provenance.binance.leaseId === previousTradeLease)
    && Date.now() < tradeQuoteDeadline);
  assert.equal(currentMarket.status, 'AVAILABLE', 'The positive BNB estimate requires a fresh Proposal market quote');
  assert.notEqual(currentMarket.snapshot.provenance.binance.leaseId, previousTradeLease,
    'The final estimate uses a newly acquired Proposal-owned quote lease');
  await waitForUIText(quote, 'STREAMING');
  await waitForUIText(quote, '60000 / 60001');

  const previousFx = await state.send('data.fx.connection', { workspaceId: state.workspaceId });
  const previousQuote = previousFx.evidence?.cryptoQuotes?.find(item => item.providerSymbol === 'USDT/USD');
  await currencyEvidence.getByRole('button', { name: 'Refresh rates for this proposal', exact: true }).press('Enter');
  let currentFx, currentQuote;
  const fxDeadline = Date.now() + 5000;
  do {
    await pause(50);
    currentFx = await state.send('data.fx.connection', { workspaceId: state.workspaceId });
    currentQuote = currentFx.evidence?.cryptoQuotes?.find(item => item.providerSymbol === 'USDT/USD');
  } while ((currentFx.status !== 'AVAILABLE' || !currentQuote || currentQuote.receivedAt === previousQuote?.receivedAt) && Date.now() < fxDeadline);
  assert.equal(currentFx.status, 'AVAILABLE');
  assert.ok(currentQuote);
  assert.notEqual(currentQuote.receivedAt, previousQuote?.receivedAt, 'The current review must consume a newly read Proposal-bound quote');
  assert.equal(await currencyEvidence.getByRole('button', { name: 'Refresh rates for this proposal', exact: true }).count(), 1);
  assert.equal(currentFx.evidence?.requirements?.proposalId, state.proposalId, 'The fresh source response must be bound to the unchanged Proposal');

  // Derive immediately after the fresh FX quote, before any of the bound observations age out.
  const positive = await state.send('trade.request_approval', payload);
  assert.equal(positive.eligible, false, 'The source-use report does not grant approval authority');
  assert.equal(positive.riskDecision.status, 'UNAVAILABLE');
  assert.equal(positive.spotFeeFx.outcome, 'PASS', JSON.stringify({
    outcome: positive.spotFeeFx.outcome,
    reasonCode: positive.spotFeeFx.reasonCode,
    routes: positive.spotFeeFx.routes?.map(route => ({ purpose: route.purpose, state: route.state, reason: route.reason })),
    fxStatus: currentFx.status,
    fxObservedAt: currentFx.observedAt,
    fxRequirementsProposalId: currentFx.evidence?.requirements?.proposalId,
    quoteProviderTime: currentQuote.providerTimestamp,
    quoteReceivedAt: currentQuote.receivedAt,
    sourceUse: currentFx.sourceUseAssessment?.status,
    feeCurrency: positive.spotFeeFx.feeCurrency,
    currentFeeModelFailure: positive.spotFeeFx.currentFeeModelFailure,
    currentFeeModel: positive.spotFeeFx.currentFeeModel && {
      chargingCondition: positive.spotFeeFx.currentFeeModel.chargingCondition,
      conditionalDiscountAsset: positive.spotFeeFx.currentFeeModel.conditionalDiscountAsset,
      bnbFee: positive.spotFeeFx.currentFeeModel.bnbFee?.total,
      bnbFeeFailure: positive.spotFeeFx.currentFeeModel.bnbFeeFailure,
      bnbFundingEvidence: positive.spotFeeFx.currentFeeModel.bnbFundingEvidence && {
        free: positive.spotFeeFx.currentFeeModel.bnbFundingEvidence.free,
        effectiveFree: positive.spotFeeFx.currentFeeModel.bnbFundingEvidence.effectiveFree,
      },
      preview: positive.spotFeeFx.currentFeeModel.fundingPlanPreview && {
        feeDisposition: positive.spotFeeFx.currentFeeModel.fundingPlanPreview.feeDisposition,
        currentFeeClaim: positive.spotFeeFx.currentFeeModel.fundingPlanPreview.currentFeeClaim,
        fallbackFeeAsset: positive.spotFeeFx.currentFeeModel.fundingPlanPreview.fallbackFeeAsset,
        fallbackFeeAmount: positive.spotFeeFx.currentFeeModel.fundingPlanPreview.fallbackFeeAmount,
      },
    },
    wallet: positive.spotFeeFx.walletFundingEvidence && {
      status: positive.spotFeeFx.walletFundingEvidence.status,
      missingAssets: positive.spotFeeFx.walletFundingEvidence.missingAssets,
    },
  }));
  assert.equal(positive.spotFeeFx.bindingBlocker ?? null, null);
  assert.equal(positive.spotFeeFx.feeCurrency, 'UNKNOWN');
  assert.equal(Object.hasOwn(positive.spotFeeFx, 'expectedFee'), false,
    'An unknown charging asset omits the expected fee instead of serializing a fabricated amount');
  assert.equal(positive.spotFeeFx.workspaceBaseExpectedSpend, '60.006');
  assert.equal(positive.spotFeeFx.currentFeeModel.chargingCondition, 'DISCOUNTED_BNB_CURRENT_ESTIMATE');
  assert.equal(positive.spotFeeFx.currentFeeModel.bnbFee.total, model.bnbFee.total);
  assert.equal(positive.riskDecision.checks.find(check => check.checkId === 'SPOT_FEE_FX')?.outcome, 'PASS');
  for (const checkId of ['ORDER_NOTIONAL', 'RESERVED_CAPITAL']) {
    assert.equal(positive.riskDecision.checks.find(check => check.checkId === checkId)?.outcome, 'PASS', checkId);
  }
  assert.equal(positive.riskDecision.checks.find(check => check.checkId === 'ACCOUNT_ARMING')?.outcome, 'UNAVAILABLE');
  // `trade.request_approval` is read-only and does not refresh React's saved RiskDecision query.
  // Re-evaluate through the visible UI so the current/captured/pre-arm views render the same
  // positive financial checks without creating approval or execution authority.
  await state.evaluate();
  const captures = ui.getByRole('region', { name: LABEL, exact: true });
  const positivePanel = captures.first();
  await positivePanel.waitFor({ state: 'visible' });
  await waitForUIText(positivePanel, /qualified for price-only policy valuation/i);
  const positiveText = await positivePanel.innerText();
  assert.match(positiveText, /ACTUAL_FEE_ASSET_PENDING/i);
  assert.match(positiveText, /does not guarantee the future fee asset or amount/i);
  assert.match(positiveText, /Current received-asset fee scenario/i);
  assert.match(positiveText, /qualified for price-only policy valuation/i);
  assert.ok(positiveText.includes('60.006'));
  assert.ok(positiveText.includes(model.bnbFee.total));
  assert.match(positiveText, /BNB_INTERNAL_AVERAGE_EQUIVALENCE_UNPROVEN/);
  assert.ok((await Promise.all((await captures.all()).map(row => row.innerText()))).includes(text), 'The original blocked statement remains unchanged in history');
  const approvals = await state.send('trade.approval.list', { workspaceId: state.workspaceId, proposalId: state.proposalId });
  assertJsonEqual(approvals.approvals, []);
  assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
  assert.equal(await positivePanel.getByRole('button').count(), 0);

  const positiveWidths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await positivePanel.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.viewport, width);
      assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
      positiveWidths.push(size);
    }
  } finally { await viewport.reset(); }

  await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const prearm = dialog.getByRole('region', { name: LABEL, exact: true });
  await prearm.waitFor({ state: 'visible' });
  const prearmText = await prearm.innerText();
  assert.match(prearmText, /Saved assessment\./);
  assert.ok(prearmText.includes(model.bnbFee.total));
  assert.match(await dialog.innerText(), /Approval remains blocked/);
  assert.equal(await prearm.getByRole('button').count(), 0);
  assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
  const prearmWidths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await prearm.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.viewport, width);
      assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
      prearmWidths.push(size);
    }
  } finally { await viewport.reset(); }
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
  return {
    model, text, widths, positiveText, positiveWidths, prearmText, prearmWidths,
    marketSourceVersion: configuredMarket.stateVersion, fxSourceUse: configuredFx.sourceUseAssessment,
    sourceAndCapacityWereExplicitlyRead: true, approvalEligible: false, armEnabled: false,
    completeFinancialAcceptance: false, armingPerformed: false, consentPerformed: false,
  };
}
