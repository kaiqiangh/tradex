// Actual React/Rust/disposable SQLite. Only external HTTP responses and the vault are fake.
// This replay covers the current required-FX blocked path and captured pre-arm statement. Exact-
// symbol commission collection and changed-source history have their own focused UI replay.
import assert from 'node:assert/strict';
import { checkSpotOwningAdmissionUI } from './spot-owning-admission-ui.mjs';

const LABEL = 'Captured Proposal Spot fee and required FX';
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
  assert.match(missingText, /SPOT_REQUIRED_FX_EVIDENCE_UNAVAILABLE/);
  assert.match(missingText, /No current exact-symbol commission observation/);
  assert.match(missingText, /Fee currency · declared origin\nUNKNOWN · UNKNOWN/);
  assert.match(missingText, /USDT ≠ USD/);
  assert.match(missingText, /Saved assessment/);
  assert.match(missingText, /owning Live Spot risk never substitutes historical portfolio FX/);
  assert.match(missingText, /Not declared \/ Not declared/);
  assert.match(missingText, /Current fee scenario unavailable: SPOT_FEE_MODEL_EVIDENCE_UNAVAILABLE/);
  assert.equal(await first.getByRole('button').count(), 0);

  const review = await state.send('trade.request_approval', { workspaceId: state.workspaceId, proposalId: state.proposalId });
  assert.equal(review.eligible, false);
  assert.ok(review.blockers.some(reason => reason.startsWith('SPOT_REQUIRED_FX_EVIDENCE_UNAVAILABLE')));
  assert.equal(review.spotFeeFx.feeCurrency, 'UNKNOWN');
  assert.equal(review.spotFeeFx.expectedFee ?? null, null);
  assert.equal(review.spotFeeFx.workspaceBaseExpectedSpend ?? null, null);
  assert.equal(review.riskDecision.status, 'UNAVAILABLE');

  const widths = await measureWidths(tab, browser, first);
  await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const captured = dialog.getByRole('region', { name: LABEL, exact: true });
  const prearmText = await captured.innerText();
  assert.match(prearmText, /SPOT_REQUIRED_FX_EVIDENCE_UNAVAILABLE/);
  assert.match(prearmText, /No current exact-symbol commission observation/);
  assert.match(prearmText, /owning Live Spot risk never substitutes historical portfolio FX/);
  assert.equal(await captured.getByRole('button').count(), 0);
  assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
  const prearmWidths = await measureWidths(tab, browser, captured);
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  await tab.getAXState({ emit: false });
  assert.equal((await state.send('account.get', { workspaceId: state.workspaceId, connectionId: state.accountId })).health.arming, 'DISARMED');
  return { ...state, missingText, prearmText, widths, prearmWidths, eligible: false, armingPerformed: false, consentPerformed: false, capturedUnchanged: true };
}
