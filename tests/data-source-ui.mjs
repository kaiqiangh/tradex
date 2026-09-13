// Run with a Codex CUA tab against `npm run dev:browser`, after checkWorkspaceUI.
import assert from 'node:assert/strict';

export async function checkDataSourceUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    const primaryNavigation = ui.getByRole('navigation', { name: 'Primary navigation' });
    const desktopSettings = primaryNavigation.getByRole('button', { name: 'Settings', exact: true });
    if (await desktopSettings.isVisible()) await desktopSettings.press('Enter');
    else {
      await ui.getByText('More', { exact: true }).press('Enter');
      await primaryNavigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
    }
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Data sources', exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('heading', { name: /^OD-00[1-6]$/, exact: true }).count(), 6);
    assert.equal(await ui.locator('input[type="password"]').count(), 0, 'Data source settings must not expose secret inputs');
    assert.match(await ui.getByRole('article', { name: /OD-001/ }).innerText(), /Blocked by external setup/);
    assert.match(await ui.getByRole('article', { name: /OD-004/ }).innerText(), /general-news|general news/);
    await ui.getByRole('article', { name: /OD-001/ }).getByRole('button', { name: 'Check entitlement', exact: true }).press('Enter');
    await ui.getByRole('article', { name: /OD-001/ }).getByText(/TradeX did not read or infer credentials/).waitFor({ state: 'visible' });
    observed.push('Six OD source cards disclose the policy and the credentialed Alpaca gate stays blocked without secret access.');
    for (const width of [768, 390]) {
      await viewport.set({ width, height: 900 });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20, `Viewport override did not apply: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `Data source page overflow at ${width}px: ${JSON.stringify(size)}`);
      assert.equal(await ui.getByRole('heading', { name: 'Data sources', exact: true }).isVisible(), true);
      assert.equal(await ui.getByRole('button', { name: 'Check entitlement', exact: true }).count(), 3);
    }
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    observed.push('Data source cards, status actions and terms remain keyboard reachable at 768px/390px with no overflow or browser errors.');
    return observed;
  } finally { await viewport.reset(); }
}
