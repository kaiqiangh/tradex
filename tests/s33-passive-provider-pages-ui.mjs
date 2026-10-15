// S33 passive-page check: local test bridge only; no provider action is activated.
import assert from 'node:assert/strict';

export async function checkS33PassiveProviderPagesUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const origin = new URL(await tab.url()).origin;
  const forbidden = new Set([
    'provider.connect', 'provider.probe', 'provider.permissions', 'provider.disconnect',
    'account.refresh', 'model.verify_route', 'data.source.probe', 'data.source.disconnect',
    'alpaca.paper.orders.refresh', 'trading212.demo.orders.refresh',
    'binance.testnet.orders.refresh', 'trade.live_order.refresh', 'trade.resolution_evidence.refresh',
    'trade.spot_capacity.refresh', 'trade.spot_commission.refresh', 'trade.spot_order_intervals.refresh',
    'trade.spot_rules.refresh', 'data.fx.refresh',
    'data.fx.disconnect', 'data.calendar.refresh', 'data.calendar.disconnect',
    'data.instrument.refresh', 'data.instrument.disconnect', 'data.actions.refresh', 'data.actions.disconnect',
    'data.binance_rules.refresh', 'paper.quote.refresh',
    'data.binance_rules.disconnect', 'data.binance_market.disconnect',
    'market.hot.acquire', 'market.hot.release', 'market.screen',
  ]);
  const commandLog = async (after = 0) => {
    const response = await fetch(`${origin}/__integration/command-log?after=${after}`, {
      headers: { Origin: origin },
    });
    assert.equal(response.status, 200, 'The isolated integration bridge exposes its local command log');
    return response.json();
  };
  const observed = [];
  const assertPassiveCommands = async (label, after) => {
    const next = await commandLog(after);
    const violations = next.commands.filter(entry => forbidden.has(entry.command));
    assert.equal(violations.length, 0, `${label} must not connect, probe, or refresh provider data: ${JSON.stringify(next.commands)}`);
    observed.push(`${label} issued no provider connect/probe/account/order/data refresh command (${next.commands.length} local commands observed).`);
    return next.cursor;
  };

  try {
    await viewport.set({ width: 1280, height: 900 });
    const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).first();
    await navigation.getByRole('button', { name: 'Threads', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
    let cursor = (await commandLog()).cursor;

    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    cursor = await assertPassiveCommands('Opening Order Drafts', cursor);

    await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
    cursor = await assertPassiveCommands('Opening Settings → Providers & Models', cursor);
    assert.match(await ui.getByRole('heading', { name: 'Providers & Models', exact: true }).innerText(), /Providers & Models/);
    assert.equal(await ui.getByRole('button', { name: 'Connect account securely', exact: true }).isVisible(), true);

    await ui.getByRole('group', { name: 'Settings sections', exact: true }).getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Data sources', exact: true }).waitFor({ state: 'visible' });
    cursor = await assertPassiveCommands('Opening Settings → Data & Storage', cursor);

    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    cursor = await assertPassiveCommands('Opening Accounts', cursor);

    await navigation.getByRole('button', { name: 'Markets', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Markets', exact: true }).waitFor({ state: 'visible' });
    await assertPassiveCommands('Opening Markets', cursor);

    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    return observed;
  } finally {
    await viewport.reset();
  }
}
