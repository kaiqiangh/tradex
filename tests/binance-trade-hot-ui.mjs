// Public UI/Rust and external provider protocols only; no positive authority seed.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkBinanceTradeHotUI(tab, browser) {
  const ui = tab.playwright;
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }) });
    assert.equal(response.status, 200);
    const result = await response.json(); assert.equal(result.ok, true, JSON.stringify(result.error)); return result.data;
  };
  const navigate = async name => {
    let targets = await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true }).all();
    let visible;
    for (const target of targets) if (await target.isVisible()) { visible = target; break; }
    if (!visible) {
      await ui.getByText('More', { exact: true }).press('Enter'); await tab.getAXState({ emit: false });
      targets = await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true }).all();
      for (const target of targets) if (await target.isVisible()) { visible = target; break; }
    }
    assert.ok(visible); await visible.press('Enter'); await tab.getAXState({ emit: false });
  };
  const viewport = await browser.capabilities.get('viewport');
  try {
    await viewport.set({ width: 1280, height: 900 });
    const workspace = await send('workspace.open', {}), workspaceId = workspace.workspaceId;
    if (await ui.getByRole('heading', { name: 'Create local workspace', exact: true }).isVisible()) {
      await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill('S29 Trade owned quote');
      await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
      await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
    }
    const accounts = await send('account.list', { workspaceId });
    let account = accounts.accounts.find(row => row.providerId === 'binance' && row.environment === 'LIVE' && row.connectionState === 'CONNECTED' && row.label.startsWith('S29 external account '));
    if (!account) {
      const tested = await send('provider.connect', { step: 'test', workspaceId, providerId: 'binance', environment: 'LIVE', label: `S29 external account ${randomUUID()}` });
      account = await send('provider.connect', { step: 'confirm', workspaceId, connectionId: tested.connectionId, expectedStateVersion: tested.stateVersion, acknowledgeUnverified: false });
    }
    assert.equal(account.health.arming, 'DISARMED');
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).press('Enter');
    await ui.getByText('Public market-source selection saved. No provider read was made.', { exact: true }).waitFor({ state: 'visible' });
    await send('time.revalidate', { workspaceId });
    await navigate('Markets');
    await ui.getByRole('button', { name: /^BTC\/USDT / }).press('Enter');
    await ui.getByRole('region', { name: 'Hot quote subscription', exact: true }).getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    const markets = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(markets.status, 'AVAILABLE');
    await navigate('Order Drafts');
    const retired = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(retired.status, 'UNAVAILABLE');
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
    await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('60000');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('region', { name: 'Trade Hot quote', exact: true }).count(), 1,
      'The selected immutable Proposal must own a new visible Hot lease after Markets releases its lease');
    const live = ui.getByRole('region', { name: 'Trade Hot quote', exact: true });
    await live.getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    const current = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(current.status, 'AVAILABLE');
    assert.notEqual(current.snapshot.provenance.binance.leaseId, markets.snapshot.provenance.binance.leaseId);
    assert.notEqual(current.snapshot.provenance.marketSnapshotId, markets.snapshot.provenance.marketSnapshotId);
    assert.match(await live.innerText(), /No execution authority/);
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await live.evaluate(element => ({ width: window.innerWidth, scroll: document.documentElement.scrollWidth,
        left: element.getBoundingClientRect().left, right: element.getBoundingClientRect().right, panelScroll: element.scrollWidth, panelClient: element.clientWidth }));
      assert.equal(size.width, width); assert.ok(size.scroll <= width && size.left >= 0 && size.right <= width && size.panelScroll <= size.panelClient + 1, JSON.stringify(size));
    }
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const review = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await review.waitFor({ state: 'visible' }); await tab.getAXState({ emit: false });
    await review.getByText('Captured quote', { exact: true }).waitFor({ state: 'visible' });
    const captured = review.getByRole('region', { name: 'Binance Spot depth evidence', exact: true });
    const text = await captured.innerText();
    assert.ok(text.includes(current.snapshot.provenance.marketSnapshotId));
    assert.equal(await review.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
    assert.equal(await review.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).count(), 0);
    const source = await send('data.binance_market.connection', { workspaceId });
    await send('data.binance_market.configure', { workspaceId, expectedStateVersion: source.stateVersion });
    let replaced;
    const replacementDeadline = Date.now() + 5000;
    do {
      replaced = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
      if (replaced.status === 'AVAILABLE' && replaced.snapshot.provenance.marketSnapshotId !== current.snapshot.provenance.marketSnapshotId) break;
      await new Promise(resolve => setTimeout(resolve, 20));
    } while (Date.now() < replacementDeadline);
    assert.equal(replaced.status, 'AVAILABLE');
    assert.notEqual(replaced.snapshot.provenance.marketSnapshotId, current.snapshot.provenance.marketSnapshotId);
    assert.equal(await captured.innerText(), text, 'Polling and new source material must not overwrite this immutable captured review');
    assert.equal(await review.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
    await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
    await navigate('Settings');
    const unavailable = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(unavailable.status, 'UNAVAILABLE');
    return { workspaceId, marketsLease: markets.snapshot.provenance.binance.leaseId, tradeLease: current.snapshot.provenance.binance.leaseId,
      capturedMaterial: current.snapshot.provenance.marketSnapshotId, replacementMaterial: replaced.snapshot.provenance.marketSnapshotId,
      widths: [1280, 768, 390], seededAuthority: false, orderWrites: false };
  } finally { await viewport.reset(); }
}
