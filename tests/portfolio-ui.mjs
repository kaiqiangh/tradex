// Run with a Codex CUA tab against `npm run dev:browser`, after checkWorkspaceUI.
import assert from 'node:assert/strict';

export async function checkPortfolioUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    const openPortfolio = ui.getByRole('button', { name: 'Open portfolio', exact: true });
    if (await openPortfolio.count()) await openPortfolio.press('Enter');
    await ui.getByRole('heading', { name: 'Workspace valuation', exact: true }).waitFor({ state: 'visible' });

    const summary = ui.locator('.portfolio-summary');
    assert.match(await summary.innerText(), /Degraded/);
    assert.match(await summary.innerText(), /3867\.6 USD/);
    assert.match(await ui.getByText(/Live risk: Blocked/).innerText(), /Live risk: Blocked/);
    const provenance = ui.locator('.portfolio-provenance');
    assert.match(await provenance.innerText(), /USDT -> USD/);
    assert.match(await provenance.innerText(), /USDT is not USD/);
    const holdings = ui.locator('section[aria-labelledby="portfolio-holdings-title"]');
    assert.match(await holdings.innerText(), /equity:US:AAPL/);
    assert.match(await holdings.innerText(), /crypto:BTC\/USDT:spot/);
    assert.match(await holdings.innerText(), /UNAVAILABLE/);
    assert.ok((await ui.locator('.table-scroll').count()) >= 3);
    observed.push('Fixture portfolio totals, canonical identities, unavailable balance, FX provenance and blocked Live risk render in Accounts.');

    for (const width of [768, 390]) {
      await viewport.set({ width, height: 900 });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20, `Viewport override did not apply: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `Portfolio page overflow at ${width}px: ${JSON.stringify(size)}`);
      assert.equal(await ui.getByRole('heading', { name: 'Workspace valuation', exact: true }).isVisible(), true);
      assert.ok((await ui.locator('.table-scroll').count()) >= 3);
    }
    observed.push('Portfolio remains keyboard-visible at 768px/390px with horizontally scrollable tables, no page overflow and no console warnings/errors.');
    assert.equal((await tab.dev.logs({ levels: ['warn', 'error'], limit: 20 })).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
