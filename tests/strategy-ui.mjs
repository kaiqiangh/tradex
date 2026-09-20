import assert from 'node:assert/strict';

export async function checkStrategyUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  await viewport.set({ width: 1280, height: 900 });
  await ui.getByRole('button', { name: 'Strategies', exact: true }).press('Enter');
  await ui.getByRole('heading', { name: 'Strategies', exact: true }).waitFor({ state: 'visible' });
  await ui.getByRole('button', { name: 'Save version', exact: true }).press('Enter');
  await ui.getByText(/Selected .* · v1 · sha256:/).waitFor({ state: 'visible' });
  await ui.getByRole('button', { name: 'Run selected version', exact: true }).press('Enter');
  await ui.getByText('COMPLETED', { exact: true }).waitFor({ state: 'visible' });
  assert.equal(await ui.getByText('HOLD', { exact: true }).count(), 1);
  assert.equal(await ui.getByRole('button', { name: /Trade|Approve|Reserve/ }).count(), 0);
  assert.equal(await ui.locator('.strategy-result').getAttribute('aria-live'), 'polite');
  for (const width of [768, 390]) {
    await viewport.set({ width, height: 900 });
    const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    assert.ok(size.scroll <= size.width, `Strategy page overflow at ${width}px: ${JSON.stringify(size)}`);
  }
  return ['strategy draft saved as immutable version', 'fixture run returned signal-only HOLD', 'strategy surface stayed within 768px/390px viewport'];
}
