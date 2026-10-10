// Actual React/Rust/disposable SQLite. Only the external HTTP/vault producers are fake.
import assert from 'node:assert/strict';
import { checkSpotOwningAdmissionUI } from './spot-owning-admission-ui.mjs';

const LABEL = 'Captured Proposal Spot fee and required execution FX';
const panels = ui => ui.getByRole('region', { name: LABEL, exact: true });

async function measureWidths(tab, browser, panel) {
  const viewport = await browser.capabilities.get('viewport');
  const measurements = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await panel.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.viewport, width);
      assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
      measurements.push(size);
    }
  } finally { await viewport.reset(); }
  return measurements;
}

export async function checkSpotFeeRequiredFxUI(tab, browser) {
  const state = await checkSpotOwningAdmissionUI(tab, browser);
  const ui = tab.playwright;
  const first = panels(ui).first();
  await first.waitFor({ state: 'visible' });
  const missingText = await first.innerText();
  assert.match(missingText, /SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE/);
  assert.match(missingText, /No current declared commission observation/);
  assert.match(missingText, /Fee currency · declared origin\nUNKNOWN · UNKNOWN/);
  assert.match(missingText, /USDT ≠ USD/);
  assert.match(missingText, /Saved assessment/);
  assert.equal(await first.getByRole('button').count(), 0);

  const assess = async scenario => {
    await state.refreshSource(scenario);
    await state.readCapacity();
    await state.readIntervals();
    await state.evaluate();
    const latest = panels(ui).first();
    await latest.waitFor({ state: 'visible' });
    return latest;
  };
  const declared = await assess('FEE_DECLARED');
  const declaredText = await declared.innerText();
  assert.match(declaredText, /0\.0010 \/ 0\.0015/);
  assert.match(declaredText, /TAKER/);
  assert.match(declaredText, /The venue declares commission rates but not the fee-charging asset/);
  assert.match(declaredText, /60 USDT/);
  assert.match(declaredText, /provider time Not declared/);
  assert.match(declaredText, /Intent notional · declared origin INTENT_PROPOSAL/);
  assert.match(declaredText, /sha256:[0-9a-f]{64} · sha256:[0-9a-f]{64}/);
  assert.equal(await panels(ui).last().innerText(), missingText);
  const review = await state.send('trade.request_approval', { workspaceId: state.workspaceId, proposalId: state.proposalId });
  assert.equal(review.eligible, false);
  assert.ok(review.blockers.some(reason => reason.startsWith('SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE')));
  assert.equal(review.spotFeeFx.feeCurrency, 'UNKNOWN');
  assert.equal(review.spotFeeFx.expectedFee ?? null, null);

  const widths = await measureWidths(tab, browser, declared);

  const changed = await assess('FEE_CHANGED');
  const changedText = await changed.innerText();
  assert.match(changedText, /0\.0020 \/ 0\.0015/);
  assert.match(changedText, /MAKER/);
  assert.equal(await panels(ui).last().innerText(), missingText);
  const texts = await Promise.all((await panels(ui).all()).map(panel => panel.innerText()));
  assert.ok(texts.includes(declaredText), 'Changing the external commission response cannot rewrite the earlier assessment');

  // Refresh both source inputs immediately before opening review; captured content has no read.
  await assess('FEE_CHANGED');
  await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const captured = dialog.getByRole('region', { name: LABEL, exact: true });
  const prearmText = await captured.innerText();
  assert.match(prearmText, /0\.0020 \/ 0\.0015/);
  assert.match(prearmText, /SPOT_REQUIRED_FX_UNSUPPORTED_ROUTE/);
  assert.equal(await captured.getByRole('button').count(), 0);
  assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
  const prearmWidths = await measureWidths(tab, browser, captured);
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  await tab.getAXState({ emit: false });
  assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
  return { ...state, missingText, declaredText, changedText, prearmText, widths, prearmWidths, eligible: false, armingPerformed: false, consentPerformed: false, capturedUnchanged: true };
}
