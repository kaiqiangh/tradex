// Run with a Codex CUA tab against `npm run dev:browser`, after checkWorkspaceUI.
// Rust/SQLite/events are real; only provider HTTP and secret entry use explicit test fixtures.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';

export async function checkProviderUI(tab, browser, selection = 'alpaca/PAPER') {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  let alpacaClientOrderId;
  let alpacaConnectionStateVersion;
  const injectPrivateStream = async (connectionId, expectedConnectionStateVersion, frame) => {
    const requestId = randomUUID();
    const response = await fetch('http://127.0.0.1:1420/__integration/command', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        requestId,
        schemaVersion: 1,
        command: 'alpaca.paper.stream.fixture',
        payload: {
          connectionId,
          expectedConnectionStateVersion,
          remoteAccountId: '81161e77-bafd-44bb-b2a0-60b9055e3cd4',
          frame,
        },
      }),
    });
    assert.equal(response.status, 200, 'The isolated Rust stream fixture remains available');
    const result = await response.json();
    assert.equal(result.requestId, requestId);
    assert.equal(result.ok, true, JSON.stringify(result.error));
  };
  const waitForVersionChange = async (detail, previous, message) => {
    for (let attempt = 0; attempt < 100; attempt += 1) {
      const next = await detail.getAttribute('data-state-version');
      if (next && next !== previous) return next;
      await ui.waitForTimeout(50);
    }
    throw new Error(message);
  };
  const waitForButton = async name => {
    const button = ui.getByRole('button', { name, exact: true });
    for (let attempt = 0; attempt < 200; attempt += 1) {
      if (await button.count() && await button.isVisible()) return button;
      await ui.waitForTimeout(50);
    }
    throw new Error(`Button ${name} did not appear after workspace reload`);
  };
  try {
    await viewport.set({ width: 1280, height: 900 });
    await tab.getAXState({ emit: false });
    const previousPath = await ui.locator('.context .path').innerText();
    const label = `${selection} QA ${Date.now()}`;
    await ui.getByRole('button', { name: 'Workspace', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.getByLabel('Workspace name', { exact: true }).fill('S02 isolated account review');
    await ui.getByLabel('Local storage', { exact: true }).fill(join(dirname(previousPath), `accounts-${Date.now()}`));
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    await tab.getAXState({ emit: false });
    assert.equal(await ui.locator('input[type="password"]').count(), 0, 'Secrets must never have renderer inputs');
    await ui.getByRole('combobox', { name: 'Provider / environment', exact: true }).selectOption(selection);
    await ui.getByLabel('Connection label', { exact: true }).fill(label);
    await ui.getByRole('button', { name: 'Connect account securely', exact: true }).click();
    await ui.getByRole('heading', { name: 'Permission review', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const detail = ui.getByRole('region', { name: label, exact: true });
    assert.match(await detail.innerText(), /REVIEW_REQUIRED/);
    const verified = selection === 'binance/LIVE' || selection.startsWith('bitget/');
    assert.match(await detail.innerText(), verified ? /VERIFIED/ : /UNVERIFIED/);
    const text = await detail.innerText();
    if (selection === 'alpaca/PAPER') {
      assert.match(text, /1000\.25/);
      assert.match(text, /1100\.9876543210123456789 USD/);
      assert.match(text, /10\.5 USD/);
    } else if (selection.startsWith('bitget/')) {
      assert.match(text, /1000000000000000002/);
      assert.match(text, /Restricted available/);
      assert.match(text, /127\.0\.0\.1/);
      assert.match(text, /TPSL/);
      assert.match(text, /PLAN/);
      assert.match(text, /plan:200/);
      assert.match(text, /0\.1234567890123456789/);
      assert.match(text, /Filled quote value/);
      if (selection.endsWith('/LIVE')) assert.match(text, /DISARMED/);
    } else if (selection.startsWith('binance/')) {
      assert.match(text, /100000000000000000000\.0000000000000000001/);
      assert.match(text, /USDT/);
      assert.match(text, /测试币/);
      assert.match(text, /BTCUSDT:9007199254740995/);
      assert.match(text, /Per asset/);
      if (verified) assert.match(text, /DISARMED/);
    } else {
      assert.match(text, /1000\.1234567890123456789/);
      assert.match(text, /9007199254740993/);
      assert.match(text, /0\.00000012/);
      assert.match(text, /150\.25 USD/);
      assert.match(text, /200\.34 GBP/);
      assert.match(text, /1\.23 GBP/);
      assert.match(text, /subtype unavailable/);
      assert.match(text, /20\.5/);
      if (selection.endsWith('/LIVE')) assert.match(text, /DISARMED/);
    }
    assert.equal(await ui.getByRole('button', { name: 'Confirm connection', exact: true }).isEnabled(), verified);
    if (!verified) await ui.getByRole('checkbox', { name: 'I understand that permission scope is unverified and have checked the key’s permissions at the provider.' }).press('Space');
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Confirm connection', exact: true }).press('Enter');
    if (!verified) await ui.getByText('Unverified scope was explicitly acknowledged for this permission review.', { exact: true }).waitFor({ state: 'visible' });
    else await ui.getByRole('button', { name: 'Confirm connection', exact: true }).waitFor({ state: 'hidden' });
    await tab.getAXState({ emit: false });
    assert.match(await detail.innerText(), /CONNECTED/);
    assert.match(await detail.innerText(), /BLOCKED/);
    assert.equal(await ui.getByRole('alert').count(), 0);
    observed.push('Schema-driven setup has no renderer secret inputs; scope review (including required UNVERIFIED acknowledgement) precedes confirmation; exact decimal account/order values are displayed.');

    await tab.reload();
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: 'Accounts', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.getByRole('heading', { name: label, exact: true }).waitFor({ state: 'visible' });
    const source = ui.getByRole('combobox', { name: 'Connection source', exact: true });
    const existingValue = await source.locator('option').filter({ hasText: label }).getAttribute('value');
    assert.ok(existingValue, 'Persisted connection should be offered for reuse');
    await source.selectOption(existingValue);
    assert.equal(await ui.getByRole('button', { name: 'Use existing account', exact: true }).isEnabled(), true);
    const beforeReuseVersion = await detail.getAttribute('data-state-version');
    assert.ok(beforeReuseVersion, 'Persisted connection should expose a state version before reuse');
    await ui.getByRole('button', { name: 'Use existing account', exact: true }).press('Enter');
    const afterReuseVersion = await waitForVersionChange(detail, beforeReuseVersion, 'Existing account reuse did not commit a new state');
    const refresh = ui.getByRole('button', { name: 'Refresh account', exact: true });
    for (let attempt = 0; attempt < 600 && !(await refresh.isEnabled()); attempt += 1) await ui.waitForTimeout(50);
    assert.equal(await refresh.isEnabled(), true);
    await refresh.press('Enter');
    await waitForVersionChange(detail, afterReuseVersion, 'Manual account refresh did not commit a new state');
    alpacaConnectionStateVersion = await detail.getAttribute('data-state-version') ?? undefined;
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('alert').count(), 0);
    assert.equal(await ui.getByRole('heading', { name: 'Workspace', exact: true }).isVisible(), true);
    observed.push('Saved connection survives UI reload; account refresh and workspace subscriptions coexist without cross-aggregate errors.');

    if (selection === 'alpaca/PAPER') {
      await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
      await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
      await ui.getByLabel('Execution context').selectOption('ALPACA_PAPER');
      const accountSelect = ui.locator('.order-draft-editor').getByLabel('Account');
      const accountValue = await accountSelect.locator('option').filter({ hasText: label }).getAttribute('value');
      assert.ok(accountValue, 'The saved Alpaca account should be selectable for a new Proposal');
      await accountSelect.selectOption(accountValue);
      await ui.getByLabel('Quantity', { exact: true }).fill('1');
      await ui.getByLabel('Limit price', { exact: true }).fill('10.25');
      await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
      await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
      const submit = ui.getByRole('button', { name: 'Submit Alpaca Paper order', exact: true });
      await submit.waitFor({ state: 'visible' });
      await submit.press('Enter');
      const dialog = ui.getByRole('dialog', { name: 'Confirm Alpaca Paper submission', exact: true });
      await dialog.waitFor({ state: 'visible' });
      assert.equal(await dialog.getAttribute('aria-modal'), 'true');
      const review = await dialog.innerText();
      assert.match(review, /This sends the exact Proposal to Alpaca Paper simulation only/);
      assert.ok(review.includes(label));
      assert.match(review, /81161e77-bafd-44bb-b2a0-60b9055e3cd4/);
      assert.match(review, /equity:US:AAPL · BUY/);
      assert.match(review, /1 BASE · LIMIT · DAY/);
      assert.match(review, /sha256:/);
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
      assert.equal(await dialog.isVisible(), false, 'Review cancellation must not submit the order');
      await submit.press('Enter');
      await dialog.waitFor({ state: 'visible' });
      for (const width of [768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Alpaca confirmation overflow at ${width}px: ${JSON.stringify(size)}`);
        assert.equal(await dialog.isVisible(), true);
      }
      await ui.getByRole('button', { name: 'Confirm Alpaca Paper submit', exact: true }).press('Enter');
      const attempt = ui.locator('section[aria-label="Alpaca Paper order attempt"]');
      await attempt.getByText('ALPACA_PAPER · ACKNOWLEDGED', { exact: true }).waitFor({ state: 'visible' });
      assert.match(await attempt.innerText(), /Alpaca acknowledged the order\. This is not fill evidence\./);
      alpacaClientOrderId = (await attempt.innerText()).match(/client order ([^\s]+)/)?.[1];
      assert.ok(alpacaClientOrderId, 'The saved attempt should expose its exact provider client order ID');
      assert.equal(await attempt.locator('p').evaluateAll(elements => elements.some(element => element.textContent?.startsWith('Fill '))), false);
      assert.equal(await ui.evaluate(() => document.activeElement?.closest('.order-proposal-panel') !== null), true);
      observed.push('Alpaca Paper Proposal review requires explicit confirmation, shows the account/order/hash identity, and renders acknowledgement separately from fills.');

      for (const width of [1280, 768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Alpaca Proposal detail overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      await viewport.set({ width: 1280, height: 900 });
      await tab.reload();
      await tab.getAXState();
      await (await waitForButton('Order Drafts')).press('Enter');
      await ui.locator('.order-proposal-row').first().waitFor({ state: 'visible' });
      await ui.locator('.order-proposal-row').first().press('Enter');
      await ui.locator('section[aria-label="Alpaca Paper order attempt"]')
        .getByText('ALPACA_PAPER · ACKNOWLEDGED', { exact: true }).waitFor({ state: 'visible' });
      assert.equal(await ui.getByRole('button', { name: 'Submit Alpaca Paper order', exact: true }).count(), 0);
      observed.push('Reload restores the saved provider attempt and removes the submit action, preventing a second UI submission.');

      await ui.getByRole('button', { name: 'Refresh orders and fills', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Open orders (1)', exact: true }).waitFor({ state: 'visible' });
      await ui.getByText(/Current provider observations/).waitFor({ state: 'visible' });
      const providerOrder = ui.locator('.order-book-order').filter({ hasText: '18c65e3e-feb0-4576-99e2-36e6f047d84d' });
      await providerOrder.waitFor({ state: 'visible' });
      assert.match(await providerOrder.innerText(), /ALPACA_PAPER · TradeX proposal/);
      assert.match(await providerOrder.innerText(), /Filled \/ remaining\s+0 \/ 1/);
      await providerOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      const cancelDialog = ui.getByRole('dialog', { name: 'Confirm Alpaca Paper cancellation', exact: true });
      await cancelDialog.waitFor({ state: 'visible' });
      await ui.getByText(/Provider read is stale/).waitFor({ state: 'visible' });
      const cancelFacts = await cancelDialog.innerText();
      assert.ok(cancelFacts.includes(label));
      assert.match(cancelFacts, /ALPACA_PAPER/);
      assert.match(cancelFacts, /18c65e3e-feb0-4576-99e2-36e6f047d84d/);
      assert.match(cancelFacts, /Filled quantity\s+0/);
      assert.match(cancelFacts, /Remaining quantity\s+1/);
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
      assert.equal(await cancelDialog.isVisible(), false, 'Closing the review must not request cancellation');
      assert.equal(await providerOrder.getByRole('button', { name: 'Review cancellation', exact: true }).isVisible(), true);

      await providerOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      await cancelDialog.waitFor({ state: 'visible' });
      for (const width of [768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Alpaca cancel confirmation overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      await ui.getByRole('button', { name: 'Confirm cancellation request', exact: true }).press('Enter');
      await providerOrder.getByRole('button', { name: 'Cancellation pending provider confirmation', exact: true }).waitFor({ state: 'visible' });
      assert.match(await providerOrder.innerText(), /CANCEL_PENDING · provider confirmation required/);
      observed.push('Order refresh identifies the TradeX order; cancel review shows account and exact fill remainder, dismissal sends nothing, and HTTP 204 stays pending.');

      const tradeUpdate = (event, status, filledQuantity, timestamp) => ({
        stream: 'trade_updates',
        data: {
          event,
          execution_id: '00000000-0000-4000-8000-000000000003',
          qty: '0.5',
          price: '10.25',
          timestamp,
          order: {
            id: '18c65e3e-feb0-4576-99e2-36e6f047d84d',
            account_id: '81161e77-bafd-44bb-b2a0-60b9055e3cd4',
            client_order_id: alpacaClientOrderId,
            symbol: 'AAPL',
            side: 'buy',
            type: 'limit',
            time_in_force: 'day',
            status,
            qty: '1',
            filled_qty: filledQuantity,
            submitted_at: '2026-09-23T10:00:00Z',
            updated_at: timestamp,
          },
        },
      });
      const partialFill = tradeUpdate('partial_fill', 'partially_filled', '0.5', '2026-09-23T10:02:00Z');
      await injectPrivateStream(existingValue, alpacaConnectionStateVersion, partialFill);
      await ui.getByRole('heading', { name: 'Fills (1)', exact: true }).waitFor({ state: 'visible' });
      await providerOrder.getByText('0.5 / 0.5', { exact: true }).waitFor({ state: 'visible' });
      assert.match(await ui.locator('.order-book-fill').innerText(), /trade_updates stream/);
      await injectPrivateStream(existingValue, alpacaConnectionStateVersion, partialFill);
      await ui.getByRole('heading', { name: 'Fills (1)', exact: true }).waitFor({ state: 'visible' });
      await injectPrivateStream(existingValue, alpacaConnectionStateVersion, tradeUpdate('new', 'new', '0', '2026-09-23T10:01:00Z'));
      await providerOrder.getByText('0.5 / 0.5', { exact: true }).waitFor({ state: 'visible' });
      assert.equal(await ui.locator('.order-book-fill').count(), 1, 'Duplicate and late stream updates must not duplicate the fill or roll back order state');
      assert.match(await providerOrder.innerText(), /partially_filled/);
      assert.match(await ui.getByRole('status').filter({ hasText: 'Private stream:' }).innerText(), /Last event:/);
      observed.push('Rust IPC persists a stream partial fill and health event; React refreshes the order projection, deduplicates repeat executions, and ignores late state rollback.');

      await viewport.set({ width: 1280, height: 900 });
      await tab.reload();
      await tab.getAXState();
      await (await waitForButton('Accounts')).press('Enter');
      await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    }

    for (const width of [768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.width <= width && size.width >= width - 20);
      const rowsFit = await ui.locator('.account-row').evaluateAll(rows => rows.every(row => {
        const box = row.getBoundingClientRect(); const parent = row.parentElement.getBoundingClientRect();
        return box.left >= parent.left - 1 && box.right <= parent.right + 1;
      }));
      assert.ok(rowsFit, `Exact account amounts must wrap within their card at ${width}px`);
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
      assert.match(await detail.innerText(), /Reconciliation\s+REQUIRED/);
      assert.match(await detail.innerText(), /Private stream\s+CONNECTED/);
      assert.match(await detail.innerText(), /Last private stream event\s+\d/);
      observed.push(`Account controls, permission limitations and separate health dimensions remain reachable at ${width}px.`);
    }
    await ui.getByRole('button', { name: 'Disconnect', exact: true }).press('Enter');
    await ui.getByText('MISSING', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.match(await detail.innerText(), /DISCONNECTED/);
    assert.equal(await ui.getByRole('button', { name: 'Refresh account', exact: true }).isEnabled(), false);
    const remove = ui.getByRole('button', { name: 'Remove local connection', exact: true });
    assert.equal(await remove.isEnabled(), true);
    const beforeCleanupVersion = await detail.getAttribute('data-state-version');
    assert.ok(beforeCleanupVersion, 'Disconnected account should expose a state version for the cleanup assertion');
    await remove.press('Enter');
    await waitForVersionChange(detail, beforeCleanupVersion, 'Local cleanup did not commit a new account state');
    await ui.getByText('MISSING', { exact: true }).waitFor({ state: 'visible' });
    assert.match(await detail.innerText(), /DISCONNECTED/);
    assert.equal(await remove.isEnabled(), true);
    assert.equal(await ui.getByRole('alert').count(), 0);
    const browserErrors = await tab.dev.logs({ levels: ['error'], limit: 100 });
    assert.equal(browserErrors.filter(error => !error.message.includes('chrome-extension://')).length, 0);
    observed.push('Disconnect removes local credential access; historical observations stay labeled disconnected and refresh is disabled.');
    return observed;
  } finally { await viewport.reset(); }
}
