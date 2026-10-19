// Public React → real Rust/temporary SQLite. Run with TRADEX_SOURCE_ONLY_FIXTURE=1.
// No market/financial authority seed, provider read, account key or mutation.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkBinanceMarketSourceSettingsUI(tab, browser) {
  const ui = tab.playwright;
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, {
      method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }),
    });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.equal(result.ok, true, JSON.stringify(result.error));
    return result.data;
  };
  const viewport = await browser.capabilities.get('viewport');
  const navigate = async (name) => {
    let buttons = await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true }).all();
    let visible;
    for (const button of buttons) if (await button.isVisible()) { visible = button; break; }
    if (!visible) {
      await ui.getByText('More', { exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
      buttons = await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true }).all();
      for (const button of buttons) if (await button.isVisible()) { visible = button; break; }
    }
    assert.ok(visible, `${name} navigation is visible`);
    await visible.press('Enter');
    await tab.getAXState({ emit: false });
  };
  const settings = async () => {
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Alpaca quote source', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
  };
  try {
    await viewport.set({ width: 1280, height: 900 });
    if (await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible()) {
      await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill('S29 public source UI');
      await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
    }
    const workspace = await send('workspace.open', {});
    const workspaceId = workspace.workspaceId;
    const before = await send('data.source.connection', { workspaceId });
    await settings();
    const panel = ui.getByRole('region', { name: 'Binance Spot market source', exact: true });
    assert.equal(await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).count(), 1,
      'An explicit public source Save action must exist independently of rule/trading-key configuration');
    await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).press('Enter');
    await ui.getByText('Public market-source selection saved. No provider read was made; the source generation changed.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const selected = await send('data.binance_market.connection', { workspaceId });
    assert.equal(selected.configured, true);
    assert.equal(selected.source.status, 'UNVERIFIED');
    assert.equal(selected.source.checkedAt ?? null, null);
    assert.equal(selected.source.observedAt ?? null, null);
    assert.equal(selected.localUseAssessment.state, 'NOT_RECORDED');
    assert.equal(await panel.getByRole('checkbox').isChecked(), false, 'The local-use confirmation is unchecked by default');
    await panel.getByRole('textbox', { name: 'Declared jurisdiction', exact: true }).fill('CA');
    const recordButton = panel.getByRole('button', { name: 'Record my local-use confirmation', exact: true });
    assert.equal(await recordButton.isEnabled(), false, 'A local-use confirmation cannot be recorded before the explicit checkbox');
    await panel.getByRole('checkbox').check();
    await recordButton.press('Enter');
    await ui.getByText('Your local-use confirmation was recorded. It does not verify provider entitlement or satisfy another risk, approval, Arm, or order gate.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const confirmed = await send('data.binance_market.connection', { workspaceId });
    assert.equal(confirmed.localUseAssessment.state, 'CURRENT');
    assert.equal(confirmed.localUseAttestation.declaredJurisdiction, 'CA');
    assert.equal(await panel.getByText('Confirmed by you for local-only decision support.', { exact: false }).count(), 1);
    assert.match(await panel.innerText(), /Provider entitlement remains independently UNVERIFIED/);
    assert.equal(await ui.locator('input[type="password"]').count(), 0);
    assert.equal(await ui.getByRole('article', { name: 'BINANCE_SPOT_PUBLIC', exact: true }).getByRole('button').count(), 0,
      'The registry card must not offer an unsupported generic endpoint probe');
    await tab.reload();
    await tab.getAXState({ emit: false });
    await settings();
    const restored = await send('data.binance_market.connection', { workspaceId });
    assert.equal(restored.configured, true);
    assert.equal(restored.stateVersion, confirmed.stateVersion);
    assert.equal(restored.source.status, 'UNVERIFIED');
    assert.equal(restored.source.observedAt ?? null, null);
    assert.match(await panel.innerText(), /UNVERIFIED/);
    assert.equal(await panel.getByRole('checkbox').isChecked(), false, "Reload does not replay the user's confirmation action");
    const restoredConfirmed = await send('data.binance_market.connection', { workspaceId });
    assert.equal(restoredConfirmed.localUseAttestation.bindingDigest, confirmed.localUseAttestation.bindingDigest);
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await panel.evaluate(element => ({
        left: element.getBoundingClientRect().left, right: element.getBoundingClientRect().right,
        width: window.innerWidth, documentScroll: document.documentElement.scrollWidth,
        panelScroll: element.scrollWidth, panelClient: element.clientWidth,
      }));
      assert.equal(size.width, width, 'The requested responsive width actually applies');
      assert.ok(size.left >= 0 && size.right <= size.width, JSON.stringify(size));
      assert.ok(size.documentScroll <= size.width && size.panelScroll <= size.panelClient + 1, JSON.stringify(size));
    }
    await navigate('Markets');
    await settings();
    await ui.getByRole('button', { name: 'Withdraw local-use confirmation', exact: true }).press('Enter');
    await ui.getByText('Your local-use confirmation was withdrawn. Current market evidence was retired with the source generation change.', { exact: true }).waitFor({ state: 'visible' });
    const withdrawn = await send('data.binance_market.connection', { workspaceId });
    assert.equal(withdrawn.localUseAttestation ?? null, null);
    assert.equal(withdrawn.localUseAssessment.state, 'NOT_RECORDED');
    assert.equal(await ui.getByRole('button', { name: 'Disconnect Binance market source', exact: true }).count(), 1);
    await ui.getByRole('button', { name: 'Disconnect Binance market source', exact: true }).press('Enter');
    await ui.getByText('Public market source disconnected. Account connections and saved keys were kept.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const disconnected = await send('data.binance_market.connection', { workspaceId });
    assert.equal(disconnected.configured, false);
    assert.equal(disconnected.source.status, 'BLOCKED_EXTERNAL');
    assert.notEqual(disconnected.stateVersion, selected.stateVersion);
    const after = await send('data.source.connection', { workspaceId });
    for (const field of ['configured', 'stateVersion', 'feed', 'accountId', 'credentialKind']) assert.deepEqual(after[field], before[field]);
    return { workspaceId, selection: selected.stateVersion, confirmed: confirmed.stateVersion, withdrawn: withdrawn.stateVersion, disconnected: disconnected.stateVersion, widths: [1280, 768, 390], seededAuthority: false };
  } finally { await viewport.reset(); }
}
