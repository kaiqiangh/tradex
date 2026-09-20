// Run with a Codex browser tab against `npm run dev:browser` after checkWorkspaceUI.
import assert from 'node:assert/strict';

export async function checkOrderDraftUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    const quantity = ui.getByLabel('Quantity', { exact: true });
    await quantity.fill('0');
    assert.equal(await ui.getByRole('button', { name: 'Save draft', exact: true }).isEnabled(), false);
    assert.equal(await ui.getByRole('alert').count() > 0, true);
    await quantity.fill('1');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText('v1', { exact: true }).count() > 0, true);
    observed.push('Zero quantity disables Save draft; a valid draft saves through the typed Rust dispatcher and shows version 1.');

    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText('NEEDS_APPROVAL', { exact: true }).count() > 0, true);
    assert.equal(await ui.getByText('GENERATED', { exact: true }).count() > 0, true);
    assert.equal(await ui.getByText('221.5 USD', { exact: true }).count() > 0, true);
    observed.push('A saved draft generates an immutable NEEDS_APPROVAL proposal with a visible history entry and estimated notional.');

    await quantity.fill('2');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 2.' }).waitFor({ state: 'visible' });
    await ui.getByText('INVALIDATED', { exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText('DRAFT_CHANGED', { exact: true }).count() > 0, true);
    observed.push('A material draft edit preserves the old proposal snapshot and records DRAFT_CHANGED as INVALIDATED.');

    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'New order draft', exact: true }).waitFor({ state: 'visible' });
    await quantity.fill('1.1234567890123456789');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('alert').filter({ hasText: /decimal|amount/i }).waitFor({ state: 'visible' });
    assert.equal(await quantity.getAttribute('aria-describedby'), 'order-field-error');
    observed.push('Server precision rejection is announced as a field-level quantity error.');

    for (const width of [768, 390]) {
      await viewport.set({ width, height: 900 });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20, `Viewport override did not apply: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `Order Drafts page overflow at ${width}px: ${JSON.stringify(size)}`);
      assert.equal(await ui.getByRole('heading', { name: 'New order draft', exact: true }).isVisible(), true);
    }
    observed.push('Order Drafts remains keyboard reachable and free of horizontal overflow at 768px and 390px.');
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
