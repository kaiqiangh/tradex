// Run with a Codex CUA tab against `npm run dev:browser`, after checkWorkspaceUI.
// Rust/SQLite/events are real; only provider HTTP and secret entry use explicit test fixtures.
import assert from 'node:assert/strict';
import { dirname, join } from 'node:path';

export async function checkProviderUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    await tab.getAXState({ emit: false });
    const previousPath = await ui.locator('.context .path').innerText();
    const label = `Alpaca QA ${Date.now()}`;
    await ui.getByRole('button', { name: 'Workspace', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.getByLabel('Workspace name', { exact: true }).fill('S02 isolated account review');
    await ui.getByLabel('Local storage', { exact: true }).fill(join(dirname(previousPath), `accounts-${Date.now()}`));
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    await tab.getAXState({ emit: false });
    assert.equal(await ui.locator('input[type="password"]').count(), 0, 'Secrets must never have renderer inputs');
    await ui.getByRole('combobox', { name: 'Provider / environment', exact: true }).selectOption('alpaca/PAPER');
    await ui.getByLabel('Connection label', { exact: true }).fill(label);
    await ui.getByRole('button', { name: 'Connect account securely', exact: true }).click();
    await ui.getByRole('heading', { name: 'Permission review', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const detail = ui.getByRole('region', { name: label, exact: true });
    assert.match(await detail.innerText(), /REVIEW_REQUIRED/);
    assert.match(await detail.innerText(), /UNVERIFIED/);
    assert.match(await detail.innerText(), /1000\.25/);
    assert.match(await detail.innerText(), /10\.5 USD/);
    assert.equal(await ui.getByRole('button', { name: 'Confirm connection', exact: true }).isEnabled(), false);
    await ui.getByRole('checkbox', { name: 'I understand that permission scope is unverified and have checked the key’s permissions at the provider.' }).press('Space');
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Confirm connection', exact: true }).press('Enter');
    await ui.getByText('Unverified scope was explicitly acknowledged for this permission review.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.match(await detail.innerText(), /CONNECTED/);
    assert.match(await detail.innerText(), /BLOCKED/);
    assert.equal(await ui.getByRole('alert').count(), 0);
    observed.push('Schema-driven setup has no renderer secret inputs; explicit UNVERIFIED acknowledgement precedes confirmation; exact decimal account/order values are displayed.');

    await tab.reload();
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.locator('.account-row').filter({ hasText: label }).press('Enter');
    await ui.getByRole('heading', { name: label, exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Refresh account', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    await ui.getByRole('heading', { name: label, exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('alert').count(), 0);
    assert.equal(await ui.getByRole('complementary', { name: 'Workspace', exact: true }).isVisible(), true);
    observed.push('Saved connection survives UI reload; account refresh and workspace subscriptions coexist without cross-aggregate errors.');

    for (const width of [768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20);
      assert.ok(size.scroll <= size.width, `Account page overflow at ${width}: ${JSON.stringify(size)}`);
      assert.equal(await ui.getByRole('button', { name: 'Refresh account', exact: true }).isVisible(), true);
      assert.equal(await ui.getByRole('button', { name: 'Disconnect', exact: true }).isVisible(), true);
      await ui.getByText('More', { exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
      await ui.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Settings', exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
      await ui.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
      await tab.getAXState({ emit: false });
      await ui.locator('.account-row').filter({ hasText: label }).press('Enter');
      await ui.getByRole('heading', { name: label, exact: true }).waitFor({ state: 'visible' });
      assert.match(await detail.innerText(), /NOT_RUN/);
      assert.match(await detail.innerText(), /NOT_CONFIGURED/);
      observed.push(`Account controls, permission limitations and separate health dimensions remain reachable at ${width}px.`);
    }
    await ui.getByRole('button', { name: 'Disconnect', exact: true }).press('Enter');
    await ui.getByText('MISSING', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.match(await detail.innerText(), /DISCONNECTED/);
    assert.equal(await ui.getByRole('button', { name: 'Refresh account', exact: true }).isEnabled(), false);
    assert.equal(await ui.getByRole('alert').count(), 0);
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 30 })).length, 0);
    observed.push('Disconnect removes local credential access; historical observations stay labeled disconnected and refresh is disabled.');
    return observed;
  } finally { await viewport.reset(); }
}
