// Actual React/Rust/temp SQLite; only external HTTP responses and the vault are fake.
// Drives the owning Spot Live admission statement through the public UI: the saved RiskDecision
// history, the pre-arm captured evidence and the blocked-then-recovered path, at keyboard
// 1280/768/390. The saved history renders newest first, so .first() is the newest statement and
// .last() is the original one.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

const OWNING = 'Captured Proposal Spot owning admission';
const owning = ui => ui.getByRole('region', { name: OWNING, exact: true });
// The saved history renders newest first, so the first statement is the newest assessment.
const statements = ui => owning(ui).allInnerTexts();
const occurrences = (texts, text) => texts.filter(item => item === text).length;

export async function checkSpotOwningAdmissionUI(tab, browser) {
  const ui = tab.playwright;
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin }, body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }) });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
    return result.data;
  };
  const workspace = await send('workspace.open', {}), workspaceId = workspace.workspaceId;
  if (await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible()) {
    await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  }
  let account = (await send('account.list', { workspaceId })).accounts.find(row => row.label.startsWith('S29.8 external ') && row.connectionState === 'CONNECTED');
  if (!account) {
    const tested = await send('provider.connect', { step: 'test', workspaceId, providerId: 'binance', environment: 'LIVE', label: `S29.8 external ${randomUUID()}` });
    account = await send('provider.connect', { step: 'confirm', workspaceId, connectionId: tested.connectionId, expectedStateVersion: tested.stateVersion, acknowledgeUnverified: false });
  }
  assert.equal(account.health.arming, 'DISARMED');
  const saved = await send('data.binance_rules.connection', { workspaceId });
  await send('data.binance_rules.configure', { workspaceId, connectionId: account.connectionId, instrumentId: 'crypto:BTC/USDT:spot', expectedStateVersion: saved.stateVersion });
  let capacityPanel, intervalsPanel;
  const refreshSource = async scenario => {
    // A completely covered inventory is the only declared external state that can qualify; an
    // exhausted order-rate counter is the venue's own refusal. Both are external HTTP producers.
    await send('binance.live.capacity.fixture', { scenario });
    await send('time.revalidate', { workspaceId });
    const source = await send('data.binance_rules.connection', { workspaceId });
    const refreshed = await send('data.binance_rules.refresh', { workspaceId, expectedStateVersion: source.stateVersion });
    assert.equal(refreshed.status, 'AVAILABLE');
    // Replacing the rule source retires both consumed observations, so each panel must first be
    // seen to have polled the new binding before its compare-and-set refresh can be offered.
    for (const panel of [capacityPanel, intervalsPanel]) {
      if (panel) await panel.getByText(refreshed.evidence.materialVersion, { exact: false }).waitFor({ state: 'visible' });
    }
  };
  await refreshSource('COMPLETE_COVERAGE');
  const navigate = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name: 'Order Drafts', exact: true });
  if (!await navigate.isVisible()) await ui.getByText('More', { exact: true }).press('Enter');
  await navigate.press('Enter'); await tab.getAXState({ emit: false });
  await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
  await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
  await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
  await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('60000');
  await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  await tab.getAXState({ emit: false });
  const proposals = await send('trade.proposal.list', { workspaceId });
  const proposalId = proposals.proposals[0].proposalId;
  capacityPanel = ui.getByRole('region', { name: 'Current Proposal Spot capacity inputs', exact: true });
  await capacityPanel.waitFor({ state: 'visible', timeout: 10000 });
  intervalsPanel = ui.getByRole('region', { name: 'Current Proposal order interval inputs', exact: true });
  await intervalsPanel.waitFor({ state: 'visible', timeout: 10000 });
  // Both read-only inputs must be current before the owning statement can be derived, and both are
  // read through their own public controls rather than through any renderer-supplied figure. Each
  // read is only offered once the panel has polled the binding it will refresh.
  const read = async (panel, button) => {
    await panel.getByText('not observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
    await panel.getByRole('button', { name: button, exact: true }).press('Enter');
    await panel.getByText('observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  };
  const readCapacity = () => read(capacityPanel, 'Refresh required capacity inputs');
  const readIntervals = () => read(intervalsPanel, 'Refresh account interval inputs');
  const evaluate = async () => {
    const again = ui.getByRole('button', { name: 'Evaluate again', exact: true });
    if (await again.count()) await again.press('Enter');
    else await ui.getByRole('button', { name: 'Evaluate risk', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
  };
  await readCapacity();
  await readIntervals();
  await evaluate();
  const admission = owning(ui).first();
  await admission.waitFor({ state: 'visible' });
  assert.equal(await owning(ui).count(), 1, 'One saved assessment renders one owning statement');
  const admissionText = await admission.innerText();
  assert.match(admissionText, /The venue's own declared figures admit this intent: 50 further orders \(ORDERS 50\/1 HOUR\)\./, 'The declared number and its binding bucket must be stated together');
  assert.match(admissionText, /Declared origin · open-order and balance inventory\nprovider declared open orders\. Covers the declared open-order count, the symbol open-buy quantity, the BASE free\/locked figures and the capacity observation time\./, 'Every reported number must name which venue read declared it');
  assert.match(admissionText, /Declared origin · order-rate counters\nprovider declared order rate counters\. Covers the remaining slots, the binding declared bucket, any standing provider cooldown and the interval observation time\./, 'Every reported number must name which venue read declared it');
  assert.match(admissionText, /This statement is read in USDT; USDT is not USD\./);
  assert.match(admissionText, /Remaining venue-declared order slots\n50 · ORDERS 50\/1 HOUR\./);
  assert.match(admissionText, /never decremented locally and never rolled forward to a new window/);
  assert.match(admissionText, /Declared open orders for this symbol\n1/);
  assert.match(admissionText, /Declared symbol open-buy quantity\n0\.1\. Reported as declared exposure only; it is never read as available balance\./);
  assert.match(admissionText, /Declared BASE free \/ locked\n0\.10000000 \/ 0\.02000000\. Reported verbatim; order locks inside a provider's free figure are never subtracted a second time\./);
  assert.match(admissionText, /Provider request cooldown at this assessment\nNone declared\. This is not an order-rate reset time and is not this qualification's gate\./);
  assert.match(admissionText, /The interval counters carry no provider timestamp, so any local time association stays explicitly uncertain\./);
  for (const limitation of ['non atomic provider snapshot', 'dynamic inputs not execution qualified', 'counter snapshot time unavailable', 'derived window association uncertain', 'interval inputs not execution qualified']) {
    assert.match(admissionText, new RegExp(limitation), `${limitation} must be carried as a standing limitation`);
  }
  assert.match(admissionText, /describe how the evidence was obtained, not what it failed to cover/);
  assert.match(admissionText, /Bound evidence versions\nsha256:[0-9a-f]{64} · sha256:[0-9a-f]{64}/);
  assert.match(admissionText, /Saved assessment\. Reading this history never renews the qualification, re-reads the venue or restores consent\./);
  assert.match(admissionText, /Nothing here is a fill promise, an admission guarantee or an approval\./);
  assert.equal(await admission.getByRole('button').count(), 0, 'A saved admission statement exposes no refresh control');
  assert.equal(await admission.getByRole('textbox').count(), 0);
  // A qualified statement is not authority: the exact account is still disarmed and the statement
  // itself offers no arming, consent or dispatch control.
  assert.equal((await send('account.get', { workspaceId, connectionId: account.connectionId })).health.arming, 'DISARMED');
  const viewport = await browser.capabilities.get('viewport'), widths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await admission.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.viewport, width); assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size)); widths.push(size);
    }
  } finally { await viewport.reset(); }
  return { send, refreshSource, readCapacity, readIntervals, evaluate, workspaceId, proposalId, accountId: account.connectionId, admission, admissionText, widths };
}

export async function checkSpotOwningBlockedUI(tab, browser, state) {
  const { send, readCapacity, readIntervals, evaluate, workspaceId, proposalId, accountId, admissionText } = state;
  const ui = tab.playwright;
  // The venue's own declared window is already at its limit while inventory coverage stays
  // complete. Nothing local decremented it, and nothing local may invent a slot: the single
  // binding fact is named instead.
  await state.refreshSource('ORDER_RATE_EXHAUSTED');
  await readCapacity();
  await readIntervals();
  await evaluate();
  const blocked = owning(ui).first();
  await blocked.waitFor({ state: 'visible' });
  assert.equal(occurrences(await statements(ui), admissionText), 1, 'A refusal is appended without rewriting the saved statement it follows');
  const blockedText = await blocked.innerText();
  assert.match(blockedText, /spot interval quota exhausted: The venue's declared order-rate window has no remaining slot, so it would not admit another order now\./, 'The single typed blocker must be stated in plain language');
  assert.match(blockedText, /This is not a smaller number\. Until the single fact above is resolved by a fresh, bound observation, no venue-declared slot can be relied on for this intent\./);
  assert.match(blockedText, /Remaining venue-declared order slots\nNone can be relied on\./);
  assert.match(blockedText, /Declared open orders for this symbol\n1\n/, 'The declared facts stay reported beside the blocker');
  assert.doesNotMatch(blockedText, /further orders/);
  assert.equal(await blocked.getByRole('button').count(), 0);
  assert.equal(await owning(ui).last().innerText(), admissionText, 'A later assessment cannot renew or rewrite the saved statement it follows');
  const review = await send('trade.request_approval', { workspaceId, proposalId });
  assert.equal(review.eligible, false, 'An intent the venue would refuse must not be eligible for approval');
  assert.ok(review.blockers.some(blocker => blocker.startsWith('SPOT_INTERVAL_QUOTA_EXHAUSTED')), JSON.stringify(review.blockers));
  assert.equal((await send('account.get', { workspaceId, connectionId: accountId })).health.arming, 'DISARMED');

  // Pre-arm captured evidence must show the same typed blocker without renewing or repairing it.
  await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const prearm = dialog.getByRole('region', { name: OWNING, exact: true });
  await prearm.waitFor({ state: 'visible' });
  const prearmText = await prearm.innerText();
  assert.match(prearmText, /spot interval quota exhausted: The venue's declared order-rate window has no remaining slot/);
  assert.match(prearmText, /Saved assessment\./);
  assert.match(await dialog.innerText(), /Approval remains blocked: .*SPOT_INTERVAL_QUOTA_EXHAUSTED/);
  assert.equal(await prearm.getByRole('button').count(), 0);
  assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
  const viewport = await browser.capabilities.get('viewport'), widths = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await prearm.evaluate(element => ({ viewport: window.innerWidth, body: document.documentElement.scrollWidth, panel: element.scrollWidth, client: element.clientWidth }));
      assert.equal(size.viewport, width); assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size)); widths.push(size);
    }
  } finally { await viewport.reset(); }
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  await tab.getAXState({ emit: false });
  return { blocked, prearm, blockedText, prearmText, widths, eligible: false, armEnabled: false, armingPerformed: false, savedStatementPreserved: true };
}

export async function checkSpotOwningRecoveryUI(tab, state) {
  const { send, readCapacity, readIntervals, evaluate, workspaceId, accountId, admissionText } = state;
  const ui = tab.playwright;
  const blockedText = await owning(ui).first().innerText();
  // The window advances at the venue, not locally: fresh bound observations of the venue's own
  // figures restore the number, and the saved blocked assessment stays exactly as it was.
  await state.refreshSource('COMPLETE_COVERAGE');
  await readCapacity();
  await readIntervals();
  await evaluate();
  const recovered = owning(ui).first();
  await recovered.waitFor({ state: 'visible' });
  const recoveredText = await recovered.innerText();
  assert.match(recoveredText, /50 further orders \(ORDERS 50\/1 HOUR\)/);
  assert.doesNotMatch(recoveredText, /spot interval quota exhausted/);
  // Recovery appends a new saved assessment; it never rewrites or drops the ones before it. The
  // exact account stays disarmed throughout, so nothing here granted or restored any authority.
  const saved = await statements(ui);
  assert.ok(occurrences(saved, blockedText) >= 1, 'The blocked assessment remains readable as history');
  assert.ok(occurrences(saved, admissionText) >= 1, 'The original saved statement remains readable as history');
  assert.equal(await owning(ui).last().innerText(), admissionText, 'The oldest saved statement is the one first relied on');
  assert.equal((await send('account.get', { workspaceId, connectionId: accountId })).health.arming, 'DISARMED');
  return { recovered, recoveredText, blockedAssessmentPreserved: true, armPerformed: false };
}
