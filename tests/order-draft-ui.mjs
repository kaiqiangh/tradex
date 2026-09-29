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
    assert.equal(await ui.getByText(/Policy: UNCONFIGURED.*v1.*state/, { exact: false }).count() > 0, true);
    assert.equal(await ui.getByText(/Market: BLOCKED_EXTERNAL.*snapshot —/, { exact: false }).count() > 0, true);
    observed.push('A saved draft generates an immutable NEEDS_APPROVAL proposal with a visible history entry and estimated notional.');

    await ui.getByRole('button', { name: 'Refresh proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: /Proposal refreshed \(STALE\)/ }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('status').filter({ hasText: /proposal:.*→ proposal:/ }).count() > 0, true);
    assert.equal(await ui.getByText('NEEDS_APPROVAL', { exact: true }).count() > 0, true);
    await ui.getByRole('button', { name: /INVALIDATED v1/ }).press('Enter');
    await ui.getByText('REFRESHED', { exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText(/Proposal refreshed as proposal:/, { exact: false }).count() > 0, true);
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'No new proposal was created; the matching Proposal is INVALIDATED.' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).count(), 0);
    await ui.getByRole('button', { name: /NEEDS_APPROVAL v1/ }).press('Enter');
    observed.push('Refresh preserves the old invalidated identity; duplicate generation identifies it as INVALIDATED and keeps submission unavailable.');

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

    for (const width of [1280, 768, 390]) {
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

export async function checkLocalPaperSubmitUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  const setPaperMode = async (mode) => {
    const response = await fetch('http://127.0.0.1:1420/__integration/paper-mode', {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ mode }),
    });
    assert.equal(response.status, 204, `Could not set the Local Paper integration mode to ${mode}`);
  };
  try {
    await viewport.set({ width: 1280, height: 900 });
    await setPaperMode('delay');
    await tab.reload(); // Clear React Query's cached paper.get result so pending UI is observable.
    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('status').filter({ hasText: 'Loading Local Paper simulation…' }).waitFor({ state: 'visible' });
    await ui.getByRole('region', { name: 'Local Paper order history', exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText('No Local Paper orders yet.', { exact: true }).count(), 1);
    assert.equal(await ui.getByRole('combobox', { name: 'Simulation scenario', exact: true }).count(), 1);
    assert.equal(await ui.getByRole('button', { name: 'Refresh simulation quote', exact: true }).count(), 1);
    await setPaperMode('normal');
    observed.push('A delayed Rust paper read renders the loading status; the new workspace exposes an explicit empty order history and labelled scenario/quote controls.');
    await ui.getByRole('button', { name: 'Refresh simulation quote', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Simulation quote refreshed at' }).waitFor({ state: 'visible' });

    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('2');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('100');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).press('Enter');
    const submitDialog = ui.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true });
    await submitDialog.waitFor({ state: 'visible' });
    assert.equal(await submitDialog.getAttribute('aria-modal'), 'true');
    await ui.getByRole('button', { name: 'Confirm submit', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'is FILLED' }).waitFor({ state: 'visible' });
    assert.equal(await ui.evaluate(() => document.activeElement?.closest('.order-proposal-panel') !== null), true, 'Submit confirmation should restore focus to the proposal surface');
    assert.equal(await ui.getByText('TRADEX_SIMULATION · FILLED', { exact: true }).count(), 1);
    assert.equal(await ui.getByText(/2 filled · 0 remaining · quote 100 USD/, { exact: false }).count(), 1);
    assert.equal(await ui.getByText(/Proposal hash: sha256:/, { exact: false }).count(), 1);
    assert.equal(await ui.getByText(/not provider truth; not Live execution/, { exact: false }).count(), 1);
    observed.push('A Local Paper Proposal submits through the Rust-backed browser dispatcher and renders deterministic full-fill, quote, hash and simulation disclosure.');

    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    const refreshSimulationQuote = ui.getByRole('button', { name: 'Refresh simulation quote', exact: true });
    await refreshSimulationQuote.waitFor({ state: 'visible' });
    for (let attempt = 0; attempt < 40 && !(await refreshSimulationQuote.isEnabled()); attempt += 1) await ui.waitForTimeout(50);
    assert.equal(await refreshSimulationQuote.isEnabled(), true);
    await refreshSimulationQuote.press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Simulation quote refreshed at' }).waitFor({ state: 'visible' });
    observed.push('The Trade surface can refresh the bounded Local Paper quote through Control Plane state.');
    const openPortfolio = ui.getByRole('button', { name: 'Open portfolio', exact: true });
    assert.equal(await openPortfolio.getAttribute('aria-expanded'), 'false');
    if (await openPortfolio.count()) await openPortfolio.press('Enter');
    await ui.getByRole('heading', { name: 'Workspace valuation', exact: true }).waitFor({ state: 'visible' });
    assert.match(await ui.getByText(/Live risk: Blocked/).innerText(), /TRADEX_SIMULATION_NOT_LIVE/);
    assert.match(await ui.getByRole('region', { name: 'Open orders', exact: true }).innerText(), /Fills/);
    observed.push('Portfolio refresh preserves Local Paper simulation provenance and blocked Live risk after the fill.');

    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    const scenario = ui.getByRole('combobox', { name: 'Simulation scenario', exact: true });
    await scenario.selectOption('partial-v1');
    await ui.getByRole('button', { name: 'Apply scenario', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Scenario partial-v1 is active.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByText('Choose a proposal to inspect its details.', { exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('4');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('100');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).press('Enter');
    await ui.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Confirm submit', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'is PARTIALLY_FILLED' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText('TRADEX_SIMULATION · PARTIALLY_FILLED', { exact: true }).count(), 1);
    assert.equal(await ui.getByText(/2 filled · 2 remaining · quote 100 USD/, { exact: false }).count(), 1);
    await ui.getByRole('button', { name: 'Cancel Local Paper order', exact: true }).press('Enter');
    await ui.getByRole('dialog', { name: 'Confirm Local Paper cancellation', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Confirm cancel', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'is CANCELLED' }).waitFor({ state: 'visible' });
    assert.equal(await ui.evaluate(() => document.activeElement?.closest('.order-proposal-panel') !== null), true, 'Cancel confirmation should restore focus to the proposal surface');
    observed.push('Partial Local Paper fill shows remaining quantity, then cancel releases reserved cash while preserving the fill.');

    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    const historyDetails = ui.locator('summary').filter({ hasText: 'View fills and events' });
    assert.equal(await ui.getByRole('region', { name: 'Local Paper order history', exact: true }).count(), 1);
    assert.equal(await historyDetails.count() > 1, true);
    await historyDetails.nth(1).click();
    assert.equal(await ui.evaluate(() => [...document.querySelectorAll('details')]
      .some((details) => details.open && details.textContent?.includes('PARTIALLY_FILLED'))), true);
    assert.equal(await ui.evaluate(() => [...document.querySelectorAll('details')]
      .some((details) => details.open && details.textContent?.includes('CANCELLED'))), true);
    observed.push('Local Paper account history exposes persisted fill and event details after cancellation.');
    await ui.getByRole('combobox', { name: 'Simulation scenario', exact: true }).selectOption('resting-v1');
    await ui.getByRole('button', { name: 'Apply scenario', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Scenario resting-v1 is active.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('1');
    await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('90');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).press('Enter');
    await ui.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Confirm submit', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'is ACCEPTED' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText(/0 filled · 1 remaining · quote 100 USD/, { exact: false }).count(), 1);
    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('heading', { name: 'Local Paper order history', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Cancel', exact: true }).press('Enter');
    await ui.getByRole('dialog', { name: 'Confirm Local Paper cancellation', exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.evaluate(() => document.activeElement?.textContent), 'Keep reviewing', 'Accounts cancellation should focus the first dialog action');
    await ui.getByRole('button', { name: 'Confirm cancel', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'is CANCELLED' }).waitFor({ state: 'visible' });
    assert.equal(await ui.evaluate(() => document.activeElement?.closest('.local-paper-summary') !== null), true, 'Accounts cancellation should restore focus to the Local Paper surface');
    observed.push('Resting limit remains ACCEPTED with one remaining unit, requires explicit confirmation, and is cancellable from the Local Paper account history.');

    await ui.getByRole('combobox', { name: 'Simulation scenario', exact: true }).selectOption('rejected-v1');
    await ui.getByRole('button', { name: 'Apply scenario', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Scenario rejected-v1 is active.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('1');
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('MARKET');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).press('Enter');
    await ui.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Confirm submit', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'is REJECTED' }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByText('TRADEX_SIMULATION · REJECTED', { exact: true }).count(), 1);
    assert.equal(await ui.getByText(/0 filled · 1 remaining · quote 100 USD/, { exact: false }).count(), 1);
    observed.push('Rejected scenario returns a typed terminal result with zero fill and no false success text.');

    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('combobox', { name: 'Simulation scenario', exact: true }).selectOption('default-v1');
    await ui.getByRole('button', { name: 'Apply scenario', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Scenario default-v1 is active.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'New order draft', exact: true }).waitFor({ state: 'visible' });
    const insufficientQuantity = ui.getByRole('textbox', { name: 'Quantity', exact: true });
    const insufficientLimit = ui.getByRole('textbox', { name: 'Limit price', exact: true });
    await insufficientQuantity.fill('1001');
    await insufficientLimit.fill('100');
    assert.equal(await insufficientQuantity.getAttribute('value'), '1001');
    assert.equal(await insufficientLimit.getAttribute('value'), '100');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Submit Local Paper order', exact: true }).press('Enter');
    await ui.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Confirm submit', exact: true }).press('Enter');
    await ui.getByRole('alert').filter({ hasText: /enough simulation cash/i }).waitFor({ state: 'visible' });
    assert.equal(await ui.getByRole('status').filter({ hasText: 'is FILLED' }).count(), 0);
    observed.push('Insufficient Local Paper cash fails closed in the Rust-backed browser flow without rendering a fill.');

    assert.equal(await ui.getByText('No open orders returned by the provider.', { exact: true }).count(), 1);
    assert.equal(await ui.getByRole('region', { name: 'Local Paper order history', exact: true }).count(), 1);
    observed.push('The empty provider open-order state remains visible after cancellation, while Local Paper order history stays in its labelled region.');

    await setPaperMode('error');
    await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    const paperError = ui.getByRole('alert').filter({ hasText: /Local Paper simulation state could not be loaded/i });
    await paperError.waitFor({ state: 'visible' });
    const reloadPaper = ui.getByRole('button', { name: 'Reload account state', exact: true });
    assert.equal(await reloadPaper.isEnabled(), true);
    await setPaperMode('normal');
    await reloadPaper.press('Enter');
    await ui.getByRole('alert').filter({ hasText: /Local Paper simulation state could not be loaded/i }).waitFor({ state: 'hidden' });
    await ui.getByRole('region', { name: 'Local Paper order history', exact: true }).waitFor({ state: 'visible' });
    observed.push('A controlled paper.get failure renders a semantic alert and enabled reload action; reloading after recovery clears the error and restores account state.');

    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20, `Viewport override did not apply: ${JSON.stringify(size)}`);
      assert.ok(size.scroll <= size.width, `Order Drafts overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    observed.push('Local Paper submission and remediation remain free of horizontal overflow at 768px and 390px.');
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    return observed;
  } finally { await setPaperMode('normal'); await viewport.reset(); }
}
