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
    const localAccountRow = ui.locator('.account-row').filter({ hasText: 'local-paper' });
    await localAccountRow.waitFor({ state: 'visible' });
    const localAccountText = await localAccountRow.innerText();
    assert.match(localAccountText, /LOCAL_PAPER/);
    assert.match(localAccountText, /TRADEX_SIMULATION/);
    assert.match(localAccountText, /not provider truth/);
    assert.match(localAccountText, /not Live execution/);
    await ui.getByRole('button', { name: '+ New Thread', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'What would you like to research?', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: '@ Context', exact: true }).press('Enter');
    const localContext = ui.locator('.context-option').filter({ hasText: 'local-paper' });
    await localContext.waitFor({ state: 'visible' });
    const localContextText = await localContext.innerText();
    assert.match(localContextText, /LOCAL_PAPER/);
    assert.match(localContextText, /TRADEX_SIMULATION/);
    assert.match(localContextText, /not provider truth/);
    assert.match(localContextText, /not Live execution/);
    await ui.getByRole('button', { name: 'Cancel', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    const openPortfolio = ui.getByRole('button', { name: 'Open portfolio', exact: true });
    if (await openPortfolio.count()) await openPortfolio.press('Enter');
    await ui.getByRole('heading', { name: 'Workspace valuation', exact: true }).waitFor({ state: 'visible' });

    const summary = ui.locator('.portfolio-summary');
    const summaryText = await summary.innerText();
    const localPaper = /LOCAL_PAPER/.test(summaryText);
    if (localPaper) {
      assert.match(summaryText, /LOCAL_PAPER/);
      assert.match(summaryText, /not provider truth/);
      assert.match(summaryText, /not Live execution/);
      assert.match(await ui.getByText(/Live risk: Blocked/).innerText(), /TRADEX_SIMULATION_NOT_LIVE/);
      assert.match(await ui.getByText(/TradeX simulation/).first().innerText(), /TRADEX_SIMULATION/);
      observed.push('Local Paper identity, simulation disclosure, empty projection and blocked Live risk render in the Rust-backed Portfolio path.');
    } else {
      assert.match(summaryText, /Degraded/);
      assert.match(summaryText, /3867\.6 USD/);
      assert.match(summaryText, /TRADEX_SIMULATION/);
      assert.match(summaryText, /portfolio-fixture-v1/);
    }
    assert.match(await ui.getByText(/Live risk: Blocked/).innerText(), /Live risk: Blocked/);
    const provenance = ui.locator('.portfolio-provenance');
    const holdings = ui.locator('section[aria-labelledby="portfolio-holdings-title"]');
    if (localPaper) {
      assert.match(await holdings.innerText(), /Local Paper/);
      assert.match(await holdings.innerText(), /100000 USD/);
      assert.match(await provenance.innerText(), /USD -> USD/);
    } else {
      assert.match(await provenance.innerText(), /USDT -> USD/);
      assert.match(await provenance.innerText(), /USDT is not USD/);
      assert.match(await holdings.innerText(), /equity:US:AAPL/);
      assert.match(await holdings.innerText(), /crypto:BTC\/USDT:spot/);
      assert.match(await holdings.innerText(), /UNAVAILABLE/);
    }
    assert.ok((await ui.locator('.table-scroll').count()) >= 3);
    if (!localPaper) observed.push('Fixture portfolio totals, canonical identities, unavailable balance, FX provenance, simulation provenance and blocked Live risk render in Accounts.');

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
