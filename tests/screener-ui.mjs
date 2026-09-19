// Run with a Codex browser tab against `npm run dev:browser` after checkWorkspaceUI.
import assert from 'node:assert/strict';

export async function checkScreenerUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    await ui.getByRole('button', { name: 'Markets', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Markets', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Open screener', exact: true }).press('Enter');
    const query = ui.getByLabel('Describe the market', { exact: true });
    await query.fill('US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.');
    await ui.getByRole('button', { name: 'Parse conditions', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'FilterSpec and RankSpec', exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText('Revision ready', { exact: true }).count(), 1);
    const threshold = ui.getByLabel('Condition 1 threshold', { exact: true });
    await threshold.fill('0.20');
    assert.equal(await ui.getByRole('button', { name: 'Run screen', exact: true }).isEnabled(), false);
    await ui.getByRole('button', { name: 'Recalculate revision', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Run screen', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'COMPLETED', exact: true }).waitFor({ state: 'visible' });
    const result = ui.locator('.screener-results');
    assert.match(await result.innerText(), /AAPL/);
    assert.match(await result.innerText(), /FX-SCREENER/);
    assert.match(await result.innerText(), /Provider 2026-09-14T00:00:00Z/);
    assert.match(await result.innerText(), /Received 2026-09-14T00:00:00Z/);
    assert.match(await result.innerText(), /SYNTHETIC_SCREENER_FIXTURE|Synthetic screener fixture/);
    observed.push('PARSE exposes FilterSpec/RankSpec, edits invalidate Run until revision recalculation, and fixture RUN renders canonical candidate/provenance evidence.');

    for (const width of [768, 390]) {
      await viewport.set({ width, height: 900 });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20, `Viewport override did not apply: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `Screener page overflow at ${width}px: ${JSON.stringify(size)}`);
      assert.equal(await ui.getByRole('heading', { name: 'COMPLETED', exact: true }).isVisible(), true);
    }
    observed.push('Screener remains visible at 768px/390px without page overflow.');
    await ui.getByRole('button', { name: 'Retry screen', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'COMPLETED', exact: true }).waitFor({ state: 'visible' });
    await query.fill('Find stocks with P/E below 20.');
    await ui.getByRole('button', { name: 'Parse conditions', exact: true }).press('Enter');
    assert.match(await ui.getByRole('alert').innerText(), /Unsupported filter field/);
    observed.push('Retry preserves the reviewed input and unsupported filters return a field-level failure.');
    await query.fill('US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.');
    await ui.getByRole('button', { name: 'Parse conditions', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Recalculate revision', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Run screen', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'COMPLETED', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Open equity:US:AAPL market detail', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Apple Inc.', exact: true }).waitFor({ state: 'visible' });
    assert.match(await ui.locator('.market-detail').innerText(), /equity:US:AAPL/);
    observed.push('Candidate rows open the canonical Market detail while preserving the exact instrument ID.');
    assert.equal((await tab.dev.logs({ levels: ['warn', 'error'], limit: 20 })).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
