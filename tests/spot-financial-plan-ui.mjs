// Actual React -> typed Rust -> disposable storage; only external producers are fake.
import assert from 'node:assert/strict';
import { checkSpotOwningAdmissionUI } from './spot-owning-admission-ui.mjs';
const LABEL = 'Captured Proposal Spot fee and required FX';
export async function checkSpotFinancialPlanUI(tab, browser) {
  const state = await checkSpotOwningAdmissionUI(tab, browser), ui = tab.playwright;
  const source = ui.getByRole('region', { name: 'Current Proposal symbol commission terms', exact: true });
  await state.send('binance.live.commission.fixture', { scenario: 'CHANGED' });
  await source.getByRole('button', { name: 'Read symbol commission terms', exact: true }).press('Enter');
  await source.getByText('observed · Read-only declaration · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  await state.evaluate();
  const panel = ui.getByRole('region', { name: LABEL, exact: true }).first();
  const text = await panel.innerText();
  assert.match(text, /Current received-asset fee scenario/);
  assert.match(text, /SPOT_THREE_FAMILY_RATIONAL_CEIL18_V1/);
  assert.match(text, /0\.0000000005 BTC/);
  assert.match(text, /0\.00000000232 BTC/);
  assert.match(text, /0\.00005 BTC/);
  assert.match(text, /0\.00005000282 BTC/);
  assert.match(text, /0\.00094999718 BTC/);
  assert.match(text, /DISCOUNT_DISABLED_CURRENT_FALLBACK/);
  assert.match(text, /Current funding plan preview/);
  assert.match(text, /Native principal claim\s+60 USDT/);
  assert.match(text, /received asset deduction/);
  assert.match(text, /0\.00005000282 BTC · 0\.00094999718 BTC/);
  assert.match(text, /current model.*future fee guarantee/i);
  assert.match(text, /SPOT_REQUIRED_FX_EVIDENCE_UNAVAILABLE/);
  assert.match(text, /Genuinely-required FX routes/);
  assert.match(text, /execution depth\/cost not applicable to price-only policy valuation/i);
  const review = await state.send('trade.request_approval', { workspaceId: state.workspaceId, proposalId: state.proposalId });
  const wallet = review.spotFeeFx.walletFundingEvidence;
  assert.equal(wallet.status, 'INCOMPLETE');
  assert.deepEqual(wallet.requiredAssets, ['BTC', 'USDT', 'BNB']);
  assert.deepEqual(wallet.missingAssets, ['BNB']);
  assert.match(text, /BASE \/ QUOTE \/ BNB wallet coverage/);
  assert.match(text, /Coverage incomplete/);
  assert.doesNotMatch(text, /BNB free \/ locked/);
  assert.doesNotMatch(text, /declared cost no declared/);
  assert.equal(await panel.getByRole('button').count(), 0);
  assert.equal(review.eligible, false);
  assert.equal(review.spotFeeFx.currentFeeModel.fallback.total, '0.00005000282');
  const plan = review.spotFeeFx.currentFeeModel.fundingPlanPreview;
  assert.equal(plan.principalClaim.asset, 'USDT');
  assert.equal(plan.principalClaim.amount, '60');
  assert.equal(plan.feeDisposition, 'RECEIVED_ASSET_DEDUCTION');
  assert.equal(Object.hasOwn(plan, 'currentFeeClaim'), false, 'Received-asset deduction creates no additional outgoing fee claim');
  assert.equal(plan.fallbackFeeAsset, 'BTC');
  assert.equal(plan.fallbackFeeAmount, '0.00005000282');
  assert.equal(plan.fallbackNetReceipt, '0.00094999718');
  assert.doesNotMatch(text, /BNB outgoing fee claim/);

  const viewport = await browser.capabilities.get('viewport'), widths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await panel.evaluate(el => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: el.scrollWidth, client: el.clientWidth }));
      assert.equal(size.viewport, width);
      assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
      widths.push(size);
    }
  } finally { await viewport.reset(); }
  await state.send('binance.live.commission.fixture', { scenario: 'INCOMPLETE' });
  await source.getByRole('button', { name: 'Read symbol commission terms', exact: true }).press('Enter');
  await source.getByText('unavailable · Read-only declaration · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  await state.evaluate();
  const captures = ui.getByRole('region', { name: LABEL, exact: true });
  assert.doesNotMatch(await captures.first().innerText(), /Current received-asset fee scenario/);
  assert.ok((await Promise.all((await captures.all()).map(row => row.innerText()))).includes(text), 'Retired current terms cannot erase or rewrite the earlier monetary capture');
  assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
  return { text, widths, fundingPlanPreview: plan, capturedUnchanged: true, incompleteSourceHasNoCurrentModel: true, currentModelPassed: true, completeFinancialAcceptance: false, armingPerformed: false, consentPerformed: false, syntheticSupportContextCreated: false };
}

export async function checkSpotZeroFeeBranchesUI(tab, browser) {
  const state = await checkSpotOwningAdmissionUI(tab, browser), ui = tab.playwright;
  const source = ui.getByRole('region', { name: 'Current Proposal symbol commission terms', exact: true });
  const captures = ui.getByRole('region', { name: LABEL, exact: true });
  const viewport = await browser.capabilities.get('viewport'), widths = [];
  const collect = async (scenario, readWallet) => {
    await state.send('binance.live.commission.fixture', { scenario });
    if (readWallet) {
      // The source binding is unchanged, so the previous observation is still
      // OBSERVED. Refresh it through its real control and wait for that receipt
      // to disappear; the owning helper's first-read NOT_OBSERVED assumption
      // does not apply to this repeat collection.
      const before = await state.send('trade.spot_capacity.get', { workspaceId: state.workspaceId, proposalId: state.proposalId });
      const capacity = ui.getByRole('region', { name: 'Current Proposal Spot capacity inputs', exact: true });
      await capacity.getByRole('button', { name: 'Refresh required capacity inputs', exact: true }).press('Enter');
      if (before.observation) await capacity.getByText(before.observation.collectionId, { exact: false }).waitFor({ state: 'hidden' });
      await capacity.getByText('observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
    }
    await source.getByRole('button', { name: 'Read symbol commission terms', exact: true }).press('Enter');
    await source.getByText('observed · Read-only declaration · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
    await state.evaluate();
    const current = captures.first(), text = await current.innerText();
    try {
      for (const width of [1280, 768, 390]) {
        await viewport.set({ width, height: 900 });
        await tab.getAXState({ emit: false });
        const size = await current.evaluate(el => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: el.scrollWidth, client: el.clientWidth }));
        assert.equal(size.viewport, width);
        assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
        widths.push({ scenario, ...size });
      }
    } finally { await viewport.reset(); }
    return text;
  };
  const zeroBnb = await collect('ZERO_BNB', true);
  assert.match(zeroBnb, /ZERO_BNB_CURRENT_FALLBACK/);
  assert.match(zeroBnb, /Observed BNB free \/ locked/);
  assert.match(zeroBnb, /0\.00000000 \/ 2\.00000000 BNB/);
  assert.match(zeroBnb, /Known local BNB claims/);
  assert.match(zeroBnb, /Effective available BNB/);
  const tinyBnb = await collect('TINY_BNB', true);
  assert.match(tinyBnb, /CONDITIONAL_BNB_INPUTS_REQUIRED/);
  assert.match(tinyBnb, /0\.000000000000000000001 BNB/);
  assert.doesNotMatch(tinyBnb, /ZERO_BNB_CURRENT_FALLBACK/);
  const zeroExpression = await collect('ZERO_EXPRESSION', false);
  assert.match(zeroExpression, /ZERO_DISCOUNTED_FEE_EXPRESSION/);
  assert.match(zeroExpression, /Known zero discounted expression/);
  assert.doesNotMatch(zeroExpression, /Observed BNB free \/ locked/);
  const texts = await Promise.all((await captures.all()).map(row => row.innerText()));
  assert.ok(texts.includes(zeroBnb) && texts.includes(tinyBnb), 'Later conditional models cannot rewrite their original captured funding evidence');
  // Re-collect only the actually consumed terms before the current review;
  // old captures and their receipts remain unchanged, never renewed.
  await source.getByRole('button', { name: 'Read symbol commission terms', exact: true }).press('Enter');
  await source.getByText('observed · Read-only declaration · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  const review = await state.send('trade.request_approval', { workspaceId: state.workspaceId, proposalId: state.proposalId });
  assert.equal(review.eligible, false);
  assert.equal(review.spotFeeFx.currentFeeModel.discountedFeeIsExactlyZero, true);
  assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
  return { zeroBnb, tinyBnb, zeroExpression, widths, capturedUnchanged: true, completeFinancialAcceptance: false, armingPerformed: false, consentPerformed: false };
}

export async function checkSpotMarketFeeUI(tab, browser, form = { side: 'SELL', type: 'BASE', quantity: '0.001', sidePrice: 'BID', gross: '60 USDT', fee: '3.600174 USDT', net: '56.399826 USDT' }) {
  const state = await checkSpotOwningAdmissionUI(tab, browser), ui = tab.playwright;
  const policy = await state.send('risk.get_policy', { workspaceId: state.workspaceId });
    await state.send('risk.save_policy', { workspaceId: state.workspaceId, expectedStateVersion: policy.stateVersion, policy: {
      ...policy.policy, marketOrdersEnabled: true, maxMarketOrderSlippagePercent: '1', maxPriceDeviationPercent: null,
    } });
  const marketSource = await state.send('data.binance_market.connection', { workspaceId: state.workspaceId });
  await state.send('data.binance_market.configure', { workspaceId: state.workspaceId, expectedStateVersion: marketSource.stateVersion });
  // The actual visible Proposal owns its quote lease. A second harness lease
  // would be replaced by React's acquire when the Proposal changes.
  {
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(state.accountId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
    await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption(form.side);
    await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption(form.type);
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('MARKET');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill(form.quantity);
    await ui.getByRole('textbox', { name: form.side === 'SELL' ? /Maximum sale value/ : /Maximum spend/ }).fill('61');
    await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('DAY');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const source = ui.getByRole('region', { name: 'Current Proposal symbol commission terms', exact: true });
    await state.send('binance.live.commission.fixture', { scenario: 'CHANGED' });
    await source.getByRole('button', { name: 'Read symbol commission terms', exact: true }).press('Enter');
    await source.getByText('observed · Read-only declaration · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
    // The continuous public protocol producer, never a seeded market snapshot.
    const quote = ui.getByRole('region', { name: 'Trade Hot quote', exact: true });
    await quote.getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    await quote.getByText('60000 / 60001', { exact: true }).waitFor({ state: 'visible' });
    await state.evaluate();
    const panel = ui.getByRole('region', { name: LABEL, exact: true }).first(), text = await panel.innerText();
    assert.match(text, /Current market scenario reference/);
    assert.ok(text.includes(form.sidePrice));
    for (const amount of [form.gross, form.fee, form.net]) assert.ok(text.includes(amount), amount);
    assert.match(text, /UNVERIFIED/);
    assert.match(text, /TAKER/);
    const viewport = await browser.capabilities.get('viewport'), widths = [];
    try {
      for (const width of [1280, 768, 390]) {
        await viewport.set({ width, height: 900 });
        await tab.getAXState({ emit: false });
        const size = await panel.evaluate(el => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: el.scrollWidth, client: el.clientWidth }));
        assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
        widths.push(size);
      }
    } finally { await viewport.reset(); }
    assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
    const currentProposal = (await state.send('trade.proposal.list', { workspaceId: state.workspaceId })).proposals[0];
    const review = await state.send('trade.request_approval', { workspaceId: state.workspaceId, proposalId: currentProposal.proposalId });
    assert.equal(review.eligible, false);
    assert.equal(review.spotFeeFx.currentFeeModel.marketReference.priceSide, form.sidePrice);
    assert.equal(review.spotFeeFx.currentFeeModel.fallbackCandidates.length, 1);
    const originalAudit = review.spotFeeFx.currentFeeModel.fallback.preRounding;
    assert.ok(originalAudit, 'The actual IPC producer retains original exact liability expressions');
    if (form.type === 'QUOTE') {
      assert.equal(originalAudit.standard.numerator, '3');
      assert.equal(originalAudit.standard.denominator, '6000100000');
      assert.equal(originalAudit.total.numerator, '7500423');
      assert.equal(originalAudit.total.denominator, '150002500000');
    }
    assert.equal(await panel.innerText(), text, 'Reading the current review cannot renew or rewrite a saved scenario');
    assert.equal(await panel.getByRole('button').count(), 0);
    return { form, text, widths, originalAudit, capturedUnchanged: true, completeFinancialAcceptance: false, armingPerformed: false, consentPerformed: false };
  }
}

