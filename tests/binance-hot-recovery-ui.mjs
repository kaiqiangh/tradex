// Faults change external WS frames, never the Control Plane book or authority.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkBinanceHotRecoveryUI(tab, browser) {
  const ui = tab.playwright;
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }) });
    assert.equal(response.status, 200); const result = await response.json(); assert.equal(result.ok, true, JSON.stringify(result.error)); return result.data;
  };
  const viewport = await browser.capabilities.get('viewport');
  try {
    await viewport.set({ width: 1280, height: 900 });
    const workspace = await send('workspace.open', {}), workspaceId = workspace.workspaceId;
    await send('binance.market.fixture.gap', { enabled: false });
    if (await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible()) {
      await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill('S29 stream failure/recovery');
      await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
      await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
    }
    await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name: 'Settings', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).press('Enter');
    await ui.getByText('Public market-source selection saved. No provider read was made.', { exact: true }).waitFor({ state: 'visible' });
    await send('time.revalidate', { workspaceId });
    const baseline = await send('binance.market.fixture.inspect', {});
    await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name: 'Markets', exact: true }).press('Enter');
    await ui.getByRole('button', { name: /^BTC\/USDT / }).press('Enter');
    const stream = ui.getByRole('region', { name: 'Hot quote subscription', exact: true });
    await stream.getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    const before = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(before.status, 'AVAILABLE');
    await send('binance.market.fixture.gap', { enabled: true });
    const failedDeadline = Date.now() + 10000;
    while (!(await stream.getByText('FAILED', { exact: true }).count()) && Date.now() < failedDeadline) {
      await tab.getAXState({ emit: false });
    }
    assert.equal(await stream.getByText('FAILED', { exact: true }).count(), 1, 'Bounded recovery reaches a visible terminal failure');
    await tab.getAXState({ emit: false });
    const failed = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(failed.status, 'UNAVAILABLE');
    assert.equal(failed.snapshot.provenance.marketSnapshotId, before.snapshot.provenance.marketSnapshotId);
    const depth = ui.getByRole('region', { name: 'Binance Spot depth evidence', exact: true });
    await depth.getByText('Retained quote — not current', { exact: true }).waitFor({ state: 'visible' });
    const failurePeer = await send('binance.market.fixture.inspect', {});
    assert.equal(failurePeer.connections, baseline.connections + 4, 'The original connection plus at most3 reconnects');
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await depth.evaluate(element => ({ width: window.innerWidth, scroll: document.documentElement.scrollWidth,
        panelScroll: element.scrollWidth, panelClient: element.clientWidth }));
      assert.equal(size.width, width); assert.ok(size.scroll <= width && size.panelScroll <= size.panelClient + 1, JSON.stringify(size));
      assert.equal(await ui.getByRole('button', { name: 'Retry selected feed', exact: true }).isEnabled(), true);
    }
    await send('binance.market.fixture.gap', { enabled: false });
    await ui.getByRole('button', { name: 'Retry selected feed', exact: true }).press('Enter');
    await stream.getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const recovered = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(recovered.status, 'AVAILABLE');
    assert.notEqual(recovered.snapshot.provenance.marketSnapshotId, before.snapshot.provenance.marketSnapshotId);
    assert.notEqual(recovered.snapshot.provenance.binance.leaseId, before.snapshot.provenance.binance.leaseId);
    const peer = await send('binance.market.fixture.inspect', {}); assert.equal(peer.connections, baseline.connections + 5);
    await viewport.set({ width: 1280, height: 900 });
    await ui.getByRole('button', { name: 'Back to results', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    return { workspaceId, retiredMaterial: before.snapshot.provenance.marketSnapshotId, recoveredMaterial: recovered.snapshot.provenance.marketSnapshotId,
      peer, widths: [1280, 768, 390], seededAuthority: false };
  } finally { await viewport.reset(); }
}
