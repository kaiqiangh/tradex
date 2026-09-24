// Run with the Codex CUA browser binding against `npm run dev:browser`.
// All interaction stays on the public UI and the real, isolated Rust dispatcher.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkWorkspaceUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 860 });
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Workspace', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).fill('S01 Browser Research');
    await ui.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('EUR');
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).click();
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).waitFor({ state: 'hidden' });
    await tab.getAXState({ emit: false });
    const openResponse = await fetch('http://127.0.0.1:1420/__integration/command', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command: 'workspace.open', payload: { name: 'S01 Browser Research', baseCurrency: 'EUR' } }),
    });
    assert.equal(openResponse.status, 200, 'The Rust integration bridge should reopen only this temporary workspace');
    const openResult = await openResponse.json();
    assert.equal(openResult.ok, true, JSON.stringify(openResult.error));
    const fixtureReady = await fetch('http://127.0.0.1:1420/__integration/command', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command: 'workspace.ready.fixture', payload: { workspaceId: openResult.data.workspaceId } }),
    });
    assert.equal(fixtureReady.status, 200, 'The Rust integration bridge should seed only this temporary workspace');
    const fixtureResult = await fixtureReady.json();
    assert.equal(fixtureResult.ok, true, JSON.stringify(fixtureResult.error));
    await tab.reload();
    await tab.getAXState({ emit: false });
    const identity = await ui.getByRole('complementary', { name: 'Workspace', exact: true }).innerText();
    assert.match(identity, /S01 Browser Research/);
    assert.match(identity, /EUR/);
    assert.equal(await ui.getByRole('button', { name: 'Create Thread', exact: true }).isEnabled(), true);
    await tab.reload();
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('complementary', { name: 'Workspace', exact: true }).innerText(), identity);
    assert.equal(await ui.getByRole('alert').count(), 0);
    observed.push('Workspace name, currency, identity and creation time survive UI reload. The ready fixture leaves thread creation available.');

    // The development bridge closes only event transports; the Rust state is unchanged.
    assert.equal((await fetch('http://127.0.0.1:1420/__integration/disconnect', { method: 'POST' })).status, 204);
    await ui.getByRole('alert').first().waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('complementary', { name: 'Workspace', exact: true }).count(), 0);
    await ui.getByRole('button', { name: 'Retry connection', exact: true }).click();
    await ui.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('complementary', { name: 'Workspace', exact: true }).innerText(), identity);
    assert.equal(await ui.getByRole('alert').count(), 0);
    observed.push('After an actual event-stream disconnect, Retry restores the unchanged snapshot and live subscription.');

    await ui.getByRole('button', { name: 'Settings', exact: true }).click();
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByText('Local workspace control is available.', { exact: true }).isVisible(), true);
    assert.equal(await ui.getByRole('heading', { name: 'CLIProxyAPI → ChatGPT', exact: true }).isVisible(), true);
    assert.equal(await ui.getByRole('heading', { name: 'CLIProxyAPI → DeepSeek', exact: true }).isVisible(), true);
    assert.equal(await ui.locator('input[type="password"]').count(), 0, 'Model keys must never have renderer inputs');
    observed.push('The Rust-backed browser fixture completes onboarding locally, exposes the Settings surface, and keeps model keys out of renderer inputs.');

    const destinations = ['+ New Thread', 'Threads', 'Markets', 'Watchlists', 'Order Drafts', 'Accounts', 'Strategies', 'Artifacts', 'Settings'];
    for (const width of [768, 390]) {
      await viewport.set({ width, height: 860 });
      await tab.getAXState({ emit: false });
      const actualWidth = await ui.evaluate(() => document.documentElement.clientWidth);
      assert.ok(actualWidth <= width && actualWidth >= width - 20, `Browser viewport override did not apply: requested ${width}, observed ${actualWidth}`);
      for (const name of destinations) {
        await ui.getByText('More', { exact: true }).press('Enter');
        await tab.getAXState({ emit: false });
        assert.equal(await ui.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name, exact: true }).isVisible(), true);
        await ui.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name, exact: true }).press('Enter');
        await tab.getAXState({ emit: false });
        const heading = name === '+ New Thread' ? 'What would you like to research?' : name;
        assert.equal(await ui.getByRole('heading', { name: heading, exact: true }).first().isVisible(), true);
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Horizontal overflow at ${width}: ${JSON.stringify(size)}`);
      }
      observed.push(`All nine navigation destinations work with Enter at ${width}px, without horizontal overflow.`);
    }
    const pageErrors = await tab.dev.logs({ levels: ['error'], limit: 20 });
    assert.equal(pageErrors.filter(log => !log.url?.startsWith('chrome-extension://') && !log.message.includes('chrome-extension://')).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
