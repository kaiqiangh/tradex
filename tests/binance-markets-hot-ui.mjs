// Visible React view → real Rust producer → external HTTP/WS; no authority seeds.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkBinanceMarketsHotUI(tab, browser, symbol = 'BTC') {
  assert.ok(['BTC', 'ETH'].includes(symbol));
  const instrumentId = `crypto:${symbol}/USDT:spot`;
  const providerSymbol = `${symbol}USDT`;
  const displayName = symbol === 'BTC' ? 'Bitcoin / Tether spot' : 'Ethereum / Tether spot';
  const row = symbol === 'BTC' ? /BTC\/USDT.*Bitcoin/ : /ETH\/USDT.*Ethereum/;
  const bid = symbol === 'BTC' ? '60000' : '3500';
  const ui = tab.playwright;
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }) });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.equal(result.ok, true, JSON.stringify(result.error));
    return result.data;
  };
  const navigate = async name => {
    let targets = await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true }).all();
    let visible;
    for (const target of targets) if (await target.isVisible()) { visible = target; break; }
    if (!visible) {
      await ui.getByText('More', { exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
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
      await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill('S29 public Hot market UI');
      await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
      await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
      await tab.getAXState({ emit: false });
    }
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Save Binance market source', exact: true }).press('Enter');
    await ui.getByText('Public market-source selection saved. No provider read was made.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    await send('time.revalidate', { workspaceId });
    const before = await send('binance.market.fixture.inspect', {});
    await navigate('Markets');
    await ui.getByRole('button', { name: row }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: row }).press('Enter');
    await ui.getByRole('heading', { name: displayName, exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('region', { name: 'Hot quote subscription', exact: true }).count(), 1,
      'The visible canonical Spot detail must own a real Hot lease instead of reading an unowned cached detail');
    const stream = ui.getByRole('region', { name: 'Hot quote subscription', exact: true });
    await stream.getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    assert.match(await stream.innerText(), /Authentication not used/);
    const quote = ui.getByRole('region', { name: 'Market quote', exact: true });
    await quote.getByText(bid, { exact: true }).waitFor({ state: 'visible' });
    assert.equal(await quote.getByText('0.75', { exact: true }).isVisible(), true);
    const depth = ui.getByRole('region', { name: 'Binance Spot depth evidence', exact: true });
    await depth.getByText('UNVERIFIED', { exact: true }).waitFor({ state: 'visible' });
    assert.match(await depth.innerText(), /Known price bands/);
    assert.equal(await depth.getByRole('table').count(), 2);
    const detail = await send('market.get', { workspaceId, instrumentId, tier: 'HOT' });
    assert.equal(detail.status, 'AVAILABLE');
    assert.equal(detail.snapshot.provenance.binance.providerSymbol, providerSymbol);
    const collected = await send('binance.market.fixture.inspect', {});
    assert.equal(collected.connections, before.connections + 1);
    assert.ok(collected.frames > before.frames);
    assert.ok(collected.requests.includes(`/api/v3/depth?symbol=${providerSymbol}&limit=1000`));
    assert.equal(collected.streams.at(-1), `/ws/${providerSymbol.toLowerCase()}@depth@100ms`);
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 }); await tab.getAXState({ emit: false });
      const size = await depth.evaluate(element => ({ width: window.innerWidth, scroll: document.documentElement.scrollWidth,
        left: element.getBoundingClientRect().left, right: element.getBoundingClientRect().right,
        panelScroll: element.scrollWidth, panelClient: element.clientWidth }));
      assert.equal(size.width, width);
      assert.ok(size.scroll <= width && size.left >= 0 && size.right <= width && size.panelScroll <= size.panelClient + 1, JSON.stringify(size));
    }
    await navigate('Settings');
    let closed;
    const deadline = Date.now() + 2000;
    do { closed = await send('binance.market.fixture.inspect', {}); if (closed.closedConnections === closed.connections) break;
      await new Promise(resolve => setTimeout(resolve, 20)); } while (Date.now() < deadline);
    assert.equal(closed.closedConnections, closed.connections, 'Navigation actually closes the external socket');
    const retired = await send('market.get', { workspaceId, instrumentId, tier: 'HOT' });
    assert.equal(retired.status, 'UNAVAILABLE');
    return { workspaceId, instrumentId, materialId: detail.snapshot.provenance.marketSnapshotId, peer: closed, widths: [1280, 768, 390], seededAuthority: false };
  } finally { await viewport.reset(); }
}

export async function checkSpotInstrumentReplacementUI(tab, browser) {
  const ui = tab.playwright;
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: endpoint.origin },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }) });
    const result = await response.json(); assert.equal(result.ok, true, JSON.stringify(result.error)); return result.data;
  };
  const viewport = await browser.capabilities.get('viewport');
  try {
    await viewport.set({ width: 1280, height: 900 });
    const workspace = await send('workspace.open', {}), workspaceId = workspace.workspaceId;
    await send('time.revalidate', { workspaceId });
    await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name: 'Markets', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: /^BTC\/USDT / }).press('Enter');
    await tab.getAXState({ emit: false });
    await ui.getByRole('region', { name: 'Hot quote subscription', exact: true }).getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    const btc = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(btc.status, 'AVAILABLE');
    const before = await send('binance.market.fixture.inspect', {});
    await ui.getByRole('button', { name: /^ETH\/USDT / }).press('Enter');
    await tab.getAXState({ emit: false });
    await ui.getByRole('heading', { name: 'Ethereum / Tether spot', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('region', { name: 'Hot quote subscription', exact: true }).getByText('STREAMING', { exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('region', { name: 'Market quote', exact: true }).getByText('3500', { exact: true }).waitFor({ state: 'visible' });
    const eth = await send('market.get', { workspaceId, instrumentId: 'crypto:ETH/USDT:spot', tier: 'HOT' });
    assert.equal(eth.status, 'AVAILABLE');
    assert.notEqual(eth.snapshot.provenance.binance.leaseId, btc.snapshot.provenance.binance.leaseId);
    assert.notEqual(eth.snapshot.provenance.binance.connectionGeneration, btc.snapshot.provenance.binance.connectionGeneration);
    assert.notEqual(eth.snapshot.provenance.marketSnapshotId, btc.snapshot.provenance.marketSnapshotId);
    assert.equal(eth.snapshot.provenance.binance.baseAsset, 'ETH');
    const retired = await send('market.get', { workspaceId, instrumentId: 'crypto:BTC/USDT:spot', tier: 'HOT' });
    assert.equal(retired.status, 'UNAVAILABLE');
    const after = await send('binance.market.fixture.inspect', {});
    assert.equal(after.connections, before.connections + 1);
    assert.equal(after.closedConnections, before.closedConnections + 1);
    assert.equal(after.streams.at(-1), '/ws/ethusdt@depth@100ms');
    assert.equal(await ui.getByRole('region', { name: 'Market quote', exact: true }).getByText('60000', { exact: true }).count(), 0);
    await ui.getByRole('button', { name: 'Back to results', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('region', { name: 'Hot quote subscription', exact: true }).count(), 0);
    return { workspaceId, retiredLease: btc.snapshot.provenance.binance.leaseId, newLease: eth.snapshot.provenance.binance.leaseId, peer: after, seededAuthority: false };
  } finally { await viewport.reset(); }
}
