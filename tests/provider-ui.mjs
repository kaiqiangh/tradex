// Run with a Codex CUA tab against `npm run dev:browser`, after checkWorkspaceUI.
// Rust/SQLite/events are real; only provider HTTP and secret entry use explicit test fixtures.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { dirname, join } from 'node:path';

async function waitForVersionChange(ui, detail, previous, message) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    const next = await detail.getAttribute('data-state-version');
    if (next && next !== previous) return next;
    await ui.waitForTimeout(50);
  }
  throw new Error(message);
}

async function sendIntegrationCommand(command, payload, expectedOk = true) {
  const requestId = randomUUID();
  const response = await fetch('http://127.0.0.1:1420/__integration/command', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ requestId, schemaVersion: 1, command, payload }),
  });
  assert.equal(response.status, 200, 'The Rust integration command endpoint remains available');
  const result = await response.json();
  assert.equal(result.requestId, requestId);
  const frame = payload?.frame?.event;
  const frameLabel = frame ? ` ${frame.e}/${frame.x ?? '-'}/${frame.X ?? '-'}` : '';
  assert.equal(result.ok, expectedOk, `${command}${frameLabel}: ${JSON.stringify(result.error)}`);
  return result;
}

export async function checkProviderUI(tab, browser, selection = 'alpaca/PAPER') {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  let alpacaClientOrderId;
  let alpacaConnectionStateVersion;
  let binanceConnectionStateVersion;
  const injectPrivateStream = (connectionId, expectedConnectionStateVersion, frame) => sendIntegrationCommand(
    'alpaca.paper.stream.fixture',
    { connectionId, expectedConnectionStateVersion, remoteAccountId: '81161e77-bafd-44bb-b2a0-60b9055e3cd4', frame },
  );
  const injectPrivateStreamDisconnect = (connectionId, expectedConnectionStateVersion) => sendIntegrationCommand(
    'alpaca.paper.stream.disconnect.fixture',
    { connectionId, expectedConnectionStateVersion },
  );
  const injectBinancePrivateStream = (connectionId, expectedConnectionStateVersion, frame) => sendIntegrationCommand(
    'binance.testnet.stream.fixture',
    {
      connectionId,
      expectedConnectionStateVersion,
      remoteAccountId: '9007199254740993',
      subscriptionId: 7,
      frame,
    },
  );
  const disconnectBinancePrivateStream = (connectionId, expectedConnectionStateVersion) => sendIntegrationCommand(
    'binance.testnet.stream.disconnect.fixture',
    { connectionId, expectedConnectionStateVersion },
  );
  const reconcileBinancePrivateStream = (connectionId, expectedConnectionStateVersion) => sendIntegrationCommand(
    'binance.testnet.stream.reconcile.fixture',
    { connectionId, expectedConnectionStateVersion, remoteAccountId: '9007199254740993' },
  );
  const configureBinanceCancelFixture = (scenario, orderUpdatedAtMs) => sendIntegrationCommand(
    'binance.testnet.cancel.fixture',
    { scenario, ...(orderUpdatedAtMs == null ? {} : { orderUpdatedAtMs }) },
  );
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
    const isolatedWorkspaceName = 'S02 isolated account review';
    const isolatedWorkspacePath = join(dirname(previousPath), `accounts-${Date.now()}`);
    await ui.getByRole('button', { name: 'Workspace', exact: true }).click();
    await tab.getAXState({ emit: false });
    await ui.getByLabel('Workspace name', { exact: true }).fill(isolatedWorkspaceName);
    await ui.getByLabel('Local storage', { exact: true }).fill(isolatedWorkspacePath);
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).click();
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).waitFor({ state: 'hidden' });
    await tab.getAXState({ emit: false });
    const openedWorkspace = await sendIntegrationCommand('workspace.open', {
      name: isolatedWorkspaceName,
      baseCurrency: 'USD',
      path: isolatedWorkspacePath,
    });
    await sendIntegrationCommand('workspace.ready.fixture', { workspaceId: openedWorkspace.data.workspaceId });
    await tab.reload();
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
      assert.match(text, /ODDCOIN/);
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
    const afterReuseVersion = await waitForVersionChange(ui, detail, beforeReuseVersion, 'Existing account reuse did not commit a new state');
    const refresh = ui.getByRole('button', { name: 'Refresh account', exact: true });
    for (let attempt = 0; attempt < 600 && !(await refresh.isEnabled()); attempt += 1) await ui.waitForTimeout(50);
    assert.equal(await refresh.isEnabled(), true);
    await refresh.press('Enter');
    await waitForVersionChange(ui, detail, afterReuseVersion, 'Manual account refresh did not commit a new state');
    alpacaConnectionStateVersion = await detail.getAttribute('data-state-version') ?? undefined;
    if (selection === 'binance/TESTNET') binanceConnectionStateVersion = await detail.getAttribute('data-state-version') ?? undefined;
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

      await injectPrivateStreamDisconnect(existingValue, alpacaConnectionStateVersion);
      const streamHealth = ui.getByRole('status').filter({ hasText: 'Private stream:' });
      await ui.getByText(/Provider read is stale:/).waitFor({ state: 'visible' });
      for (let attempt = 0; attempt < 100; attempt += 1) {
        if ((await streamHealth.innerText()).includes('Private stream: DEGRADED')) break;
        await ui.waitForTimeout(50);
      }
      assert.match(await streamHealth.innerText(), /Private stream: DEGRADED · Reconciliation: DEGRADED/);
      observed.push('A Rust-persisted stream disconnect marks the saved order book stale and renders degraded stream/reconciliation text on the Order surface.');

      await viewport.set({ width: 1280, height: 900 });
      await tab.reload();
      await tab.getAXState();
      await (await waitForButton('Accounts')).press('Enter');
      await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    } else if (selection === 'trading212/DEMO') {
      await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
      const orderBook = ui.locator('.order-book-panel');
      await orderBook.getByLabel('Trading 212 Demo account', { exact: true }).selectOption(existingValue);
      await orderBook.getByText(/No provider read has completed/).waitFor({ state: 'visible' });
      await orderBook.getByRole('button', { name: 'Refresh pending orders', exact: true }).press('Enter');
      const preexistingOrder = orderBook.locator('.order-book-order').filter({ hasText: '9007199254740996' });
      await preexistingOrder.waitFor({ state: 'visible' });
      assert.match(await preexistingOrder.innerText(), /External provider order/);
      assert.match(await preexistingOrder.innerText(), /PARTIALLY_FILLED \/ PARTIALLY_FILLED/);
      assert.match(await preexistingOrder.innerText(), /1\.23 GBP/);
      await orderBook.getByRole('button', { name: 'Load order history', exact: true }).press('Enter');
      const externalOrder = orderBook.locator('.order-book-order').filter({ hasText: '8001' });
      await externalOrder.waitFor({ state: 'visible' });
      assert.match(await externalOrder.innerText(), /TRADING212_DEMO · External provider order/);
      assert.match(await externalOrder.innerText(), /FILLED \/ FILLED/);
      assert.match(await externalOrder.innerText(), /201\.234567890123456789 GBP/);
      await orderBook.getByRole('button', { name: 'Refresh order history', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: /PROVIDER_RATE_LIMITED.*Retry after/ }).waitFor({ state: 'visible' });
      await orderBook.getByRole('alert').filter({ hasText: 'PROVIDER_RATE_LIMITED' }).waitFor({ state: 'visible' });
      assert.equal(await orderBook.locator('.order-book-order').filter({ hasText: '8001' }).count(), 1);
      observed.push('Trading 212 Demo order book loads a pre-existing external partial order with its currency and an external completed order; an immediate history refresh renders its retry time and retains the trusted row.');

      await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
      await ui.getByLabel('Execution context').selectOption('TRADING212_DEMO');
      const accountSelect = ui.locator('.order-draft-editor').getByLabel('Account');
      const accountValue = await accountSelect.locator('option').filter({ hasText: label }).getAttribute('value');
      assert.ok(accountValue, 'The saved Trading 212 Demo account should be selectable for a new Proposal');
      await accountSelect.selectOption(accountValue);
      await ui.getByLabel('Order type').selectOption('MARKET');
      await ui.getByLabel('Time in force').selectOption('DAY');
      await ui.getByLabel('Quantity', { exact: true }).fill('1.234567890123456789');
      await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
      await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
      const submit = ui.getByRole('button', { name: 'Submit Trading 212 Demo order', exact: true });
      await submit.waitFor({ state: 'visible' });
      await submit.press('Enter');
      const dialog = ui.getByRole('dialog', { name: 'Confirm Trading 212 Demo submission', exact: true });
      await dialog.waitFor({ state: 'visible' });
      assert.equal(await dialog.getAttribute('aria-modal'), 'true');
      const review = await dialog.innerText();
      assert.match(review, /Trading 212 Demo · TRADING212_DEMO/);
      assert.ok(review.includes(label));
      assert.match(review, /9007199254740993/);
      assert.match(review, /equity:US:AAPL · BUY/);
      assert.match(review, /1\.234567890123456789 BASE · MARKET · DAY/);
      assert.match(review, /Extended hours\s+Off/);
      assert.match(review, /sha256:/);
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
      assert.equal(await dialog.isVisible(), false, 'Review dismissal must not send a provider order');
      await submit.press('Enter');
      await dialog.waitFor({ state: 'visible' });
      for (const width of [768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Trading 212 confirmation overflow at ${width}px: ${JSON.stringify(size)}`);
        assert.equal(await dialog.isVisible(), true);
      }
      await ui.getByRole('button', { name: 'Confirm Trading 212 Demo submit', exact: true }).press('Enter');
      const attempt = ui.locator('section[aria-label="Trading 212 Demo order attempt"]');
      await attempt.getByText('Trading 212 Demo · TRADING212_DEMO · ACKNOWLEDGED', { exact: true }).waitFor({ state: 'visible' });
      assert.match(await attempt.innerText(), /acknowledged the order\. This is not fill evidence\./);
      assert.match(await attempt.innerText(), /Provider order 9007199254740995 · provider status NEW/);
      assert.equal(await attempt.locator('p').evaluateAll(elements => elements.some(element => element.textContent?.startsWith('Fill '))), false);
      assert.equal(await ui.getByRole('button', { name: 'Submit Trading 212 Demo order', exact: true }).count(), 0);
      assert.equal(await ui.evaluate(() => document.activeElement?.closest('.order-proposal-panel') !== null), true);
      observed.push('Trading 212 Demo requires a separate exact-proposal confirmation, visibly disables extended hours, persists acknowledgement without claiming a fill, and restores proposal focus.');

      for (const width of [1280, 768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Trading 212 Proposal detail overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      await viewport.set({ width: 1280, height: 900 });
      await tab.reload();
      await tab.getAXState();
      await (await waitForButton('Order Drafts')).press('Enter');
      await ui.locator('.order-proposal-row').first().waitFor({ state: 'visible' });
      await ui.locator('.order-proposal-row').first().press('Enter');
      await ui.locator('section[aria-label="Trading 212 Demo order attempt"]')
        .getByText('Trading 212 Demo · TRADING212_DEMO · ACKNOWLEDGED', { exact: true }).waitFor({ state: 'visible' });
      assert.equal(await ui.getByRole('button', { name: 'Submit Trading 212 Demo order', exact: true }).count(), 0);
      observed.push('Reload reads the SQLite-backed Trading 212 Demo attempt and prevents a second UI submission.');

      await ui.locator('.order-book-panel').getByLabel('Trading 212 Demo account', { exact: true }).selectOption(existingValue);
      const savedOrderBook = ui.locator('.order-book-panel');
      await ui.waitForTimeout(5100);
      await savedOrderBook.getByRole('button', { name: 'Refresh pending orders', exact: true }).press('Enter');
      const partialOrder = savedOrderBook.locator('.order-book-order').filter({ hasText: '9007199254740995' });
      await partialOrder.waitFor({ state: 'visible' });
      assert.match(await partialOrder.innerText(), /TradeX proposal/);
      assert.match(await partialOrder.innerText(), /PARTIALLY_FILLED \/ PARTIALLY_FILLED/);
      assert.match(await partialOrder.innerText(), /0\.25/);
      assert.match(await partialOrder.innerText(), /33\.125/);
      assert.match(await partialOrder.innerText(), /0\.984567890123456789/);
      await partialOrder.getByRole('button', { name: 'Refresh known order details', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: /Trading 212 Demo order detail refreshed at/ }).waitFor({ state: 'visible' });
      assert.match(await partialOrder.innerText(), /PARTIALLY_FILLED \/ PARTIALLY_FILLED/);
      assert.match(await partialOrder.innerText(), /33\.125/);
      await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      const cancelDialog = ui.getByRole('dialog', { name: 'Confirm Trading 212 Demo cancellation', exact: true });
      await cancelDialog.waitFor({ state: 'visible' });
      assert.equal(await ui.locator('.app-shell').evaluate(element => element.inert), true, 'The background remains inert while reviewing a provider cancellation');
      assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Keep reviewing', 'Focus starts on the safe dismissal action');
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Shift+Tab');
      assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Confirm cancellation request', 'Reverse tab stays inside the confirmation dialog');
      await ui.getByRole('button', { name: 'Confirm cancellation request', exact: true }).press('Tab');
      assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Keep reviewing', 'Forward tab wraps inside the confirmation dialog');
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Escape');
      assert.equal(await cancelDialog.isVisible(), false, 'Escape dismisses the unconfirmed provider action');
      assert.equal(await ui.locator('.app-shell').evaluate(element => element.inert), false, 'Dismissal restores the background');
      assert.equal(await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).count(), 1);
      await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      await cancelDialog.waitFor({ state: 'visible' });
      const cancelReview = await cancelDialog.innerText();
      assert.match(cancelReview, /Trading 212 Demo · TRADING212_DEMO/);
      assert.ok(cancelReview.includes(label));
      assert.match(cancelReview, /9007199254740995/);
      assert.match(cancelReview, /PARTIALLY_FILLED \/ PARTIALLY_FILLED/);
      assert.match(cancelReview, /Filled quantity\s+0\.25/);
      assert.match(cancelReview, /Remaining quantity\s+0\.984567890123456789/);
      assert.match(cancelReview, /Acceptance only; cancellation is not confirmed/);
      for (const width of [768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Trading 212 cancel confirmation overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
      assert.equal(await cancelDialog.isVisible(), false, 'Dismissing the cancel review must leave the provider order unchanged');
      assert.equal(await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).count(), 1);
      await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      await cancelDialog.waitFor({ state: 'visible' });
      await ui.getByRole('button', { name: 'Confirm cancellation request', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: /accepted the cancellation request.*remains pending provider confirmation/ }).waitFor({ state: 'visible' });
      assert.match(await partialOrder.innerText(), /PARTIALLY_FILLED \/ PARTIALLY_FILLED/);
      assert.match(await partialOrder.innerText(), /Cancellation request accepted or outcome unknown/);
      assert.equal(await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).count(), 0);
      assert.equal(await partialOrder.getByRole('button', { name: 'Refresh known order details', exact: true }).count(), 1);
      observed.push('Trading 212 Demo cancellation requires fresh exact-order review and explicit confirmation; dismissal sends no request, acceptance stays pending provider truth, and the dialog fits 768px and 390px.');
      for (const width of [768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Trading 212 order book overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      observed.push('Reload restores the exact TradeX order identity; known-order refresh preserves cumulative partial fills, and the order-book card fits 768px and 390px layouts.');
      await ui.getByText('More', { exact: true }).press('Enter');
      await ui.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    } else if (selection === 'binance/TESTNET') {
      const isolatedWorkspaceId = await ui.locator('.context .identity').innerText();
      await ui.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
      await ui.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
      await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
      const editor = ui.locator('.order-draft-editor');
      await editor.getByRole('combobox', { name: 'Account', exact: true }).selectOption(existingValue);
      await editor.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
      await editor.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('MARKET');
      await editor.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('QUOTE');
      await editor.getByRole('textbox', { name: 'Quantity', exact: true }).fill('25');
      const selectedValues = await ui.evaluate(() => Array.from(document.querySelectorAll('.order-draft-editor select')).map(select => select.value));
      assert.equal(selectedValues[0], 'BINANCE_TESTNET');
      assert.equal(selectedValues[3], 'BINANCE');
      await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
      await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
      const proposalRow = ui.locator('.order-proposal-row').first();
      const proposalId = (await proposalRow.innerText()).split('\n')[1].split('·')[1].trim();
      const submit = ui.getByRole('button', { name: 'Submit Binance Spot Testnet order', exact: true });
      await submit.waitFor({ state: 'visible' });
      await submit.press('Enter');
      const dialog = ui.getByRole('dialog', { name: 'Confirm Binance Spot Testnet submission', exact: true });
      await dialog.waitFor({ state: 'visible' });
      const review = await dialog.innerText();
      assert.match(review, /This sends one order to Binance Spot Testnet only/);
      assert.ok(review.includes(label));
      assert.match(review, /9007199254740993/);
      assert.match(review, /crypto:BTC\/USDT:spot · BUY/);
      assert.match(review, /25 QUOTE · MARKET · DAY/);
      assert.match(review, /BINANCE · BINANCE_TESTNET/);
      assert.match(review, /Proposal \/ hash[\s\S]*sha256:/);
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
      assert.equal(await dialog.isVisible(), false, 'Dismissing review must not submit a provider order');
      assert.equal((await sendIntegrationCommand('binance.testnet.order.attempt.get', {
        workspaceId: isolatedWorkspaceId, proposalId,
      })).data.attempt, null, 'Dismissing confirmation must leave SQLite without an order attempt');
      assert.equal(await submit.isVisible(), true);
      observed.push('The real Trade UI builds a Binance Testnet Proposal; its exact account, remote identity, symbol, quote quantity, order type, TIF and hash are reviewed, and dismissal creates no SQLite attempt.');

      await submit.press('Enter');
      await dialog.waitFor({ state: 'visible' });
      for (const width of [768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Binance Testnet confirmation overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      await ui.getByRole('button', { name: 'Confirm Binance Testnet submit', exact: true }).press('Enter');
      const attempt = ui.getByRole('region', { name: 'Binance Spot Testnet order attempt', exact: true });
      await attempt.getByText('Binance Spot Testnet · TESTNET · ACKNOWLEDGED', { exact: true }).waitFor({ state: 'visible' });
      assert.match(await attempt.innerText(), /9007199254740993/);
      assert.match(await attempt.innerText(), /Provider order 9007199254740997 · provider status NEW/);
      assert.match(await attempt.innerText(), /Binance acknowledged the order\. This is not fill evidence\./);
      assert.equal(await attempt.locator('p').evaluateAll(elements => elements.some(element => element.textContent?.startsWith('Fill '))), false);
      assert.equal(await ui.getByRole('button', { name: 'Submit Binance Spot Testnet order', exact: true }).count(), 0);
      observed.push('Explicit confirmation traverses the Rust IPC/provider fixture and persists one ACKNOWLEDGED SQLite attempt; the UI keeps provider acknowledgement separate from fills.');

      await viewport.set({ width: 1280, height: 900 });
      await tab.reload();
      await tab.getAXState();
      await (await waitForButton('Order Drafts')).press('Enter');
      await ui.locator('.order-proposal-row').first().waitFor({ state: 'visible' });
      await ui.locator('.order-proposal-row').first().press('Enter');
      await ui.getByRole('region', { name: 'Binance Spot Testnet order attempt', exact: true })
        .getByText('Binance Spot Testnet · TESTNET · ACKNOWLEDGED', { exact: true }).waitFor({ state: 'visible' });
      assert.equal(await ui.getByRole('button', { name: 'Submit Binance Spot Testnet order', exact: true }).count(), 0);
      for (const width of [1280, 768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Binance Testnet Proposal overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      observed.push('Reload recovers the saved Testnet attempt and removes the submit action, preventing a second UI write.');

      const binanceOrderBook = ui.locator('.order-book-panel').filter({ has: ui.getByLabel('Binance Testnet account', { exact: true }) });
      await binanceOrderBook.getByLabel('Binance Testnet account', { exact: true }).selectOption(existingValue);
      const privateStreamNow = Date.now();
      const snapshot = {
        subscriptionId: 7,
        event: {
          e: 'outboundAccountPosition', E: privateStreamNow - 75_000, u: privateStreamNow - 75_000,
          B: [{ a: 'USDT', f: '950.5', l: '2.5' }],
        },
      };
      await injectBinancePrivateStream(existingValue, binanceConnectionStateVersion, snapshot);
      let savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
      });
      assert.equal(savedBook.data.book.balances.find(balance => balance.asset === 'USDT')?.free, '950.5');
      assert.equal(savedBook.data.book.balances.find(balance => balance.asset === 'USDT')?.locked, '2.5');
      const streamHealth = ui.getByRole('status').filter({ hasText: 'Private stream:' });
      for (let attempt = 0; attempt < 100; attempt += 1) {
        if ((await streamHealth.innerText()).includes('Private stream: CONNECTED · Reconciliation: REQUIRED')) break;
        await ui.waitForTimeout(50);
      }
      assert.match(await streamHealth.innerText(), /Private stream: CONNECTED · Reconciliation: REQUIRED/);
      await reconcileBinancePrivateStream(existingValue, binanceConnectionStateVersion);
      for (let attempt = 0; attempt < 100; attempt += 1) {
        if ((await streamHealth.innerText()).includes('Private stream: CONNECTED · Reconciliation: CURRENT')) break;
        await ui.waitForTimeout(50);
      }
      assert.match(await streamHealth.innerText(), /Private stream: CONNECTED · Reconciliation: CURRENT/);

      const executionReport = (execution, status, filled, quote, eventTime, updateTime, tradeId, lastQuantity) => ({
        subscriptionId: 7,
        event: {
          e: 'executionReport', E: eventTime, s: 'BTCUSDT', c: 'fixture-private-stream-order', O: privateStreamNow - 70_000,
          S: 'BUY', o: 'LIMIT', f: 'GTC', q: '0.25', p: '90',
          x: execution, X: status, i: 9007199254740998, l: lastQuantity, z: filled,
          L: '90', n: '0', N: 'USDT', T: updateTime, t: tradeId, Q: '0', Y: quote, Z: quote,
        },
      });
      await injectBinancePrivateStream(
        existingValue,
        binanceConnectionStateVersion,
        executionReport('NEW', 'NEW', '0', '0', privateStreamNow - 65_000, privateStreamNow - 65_000, -1, '0'),
      );
      const staleOrder = binanceOrderBook.locator('.order-book-order').filter({ hasText: '9007199254740998' });
      await staleOrder.waitFor({ state: 'visible' });
      assert.equal(await staleOrder.getByRole('button', { name: 'Review cancellation', exact: true }).count(), 0,
        'Orders observed more than 60 seconds ago must not offer cancellation review');
      savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
      });
      const expiredCancel = await sendIntegrationCommand('binance.testnet.orders.cancel', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
        expectedConnectionStateVersion: binanceConnectionStateVersion,
        expectedBookStateVersion: savedBook.data.book.stateVersion,
        symbol: 'BTCUSDT',
        providerOrderId: '9007199254740998',
        idempotencyKey: randomUUID(),
        confirmed: true,
      }, false);
      assert.equal(expiredCancel.error.code, 'ORDER_CONFIRMATION_EXPIRED', 'The IPC bridge must preserve the backend rejection');
      observed.push('A saved order observation older than 60 seconds cannot open the cancel review; Rust rejects a direct stale confirmation with ORDER_CONFIRMATION_EXPIRED through IPC.');

      const partialFill = executionReport('TRADE', 'PARTIALLY_FILLED', '0.1', '9', privateStreamNow, privateStreamNow, 9101, '0.1');
      await injectBinancePrivateStream(existingValue, binanceConnectionStateVersion, partialFill);
      await binanceOrderBook.getByText(/Trade 9101 · order 9007199254740998/).waitFor({ state: 'visible' });
      await injectBinancePrivateStream(existingValue, binanceConnectionStateVersion, partialFill);
      await injectBinancePrivateStream(
        existingValue,
        binanceConnectionStateVersion,
        executionReport('NEW', 'NEW', '0', '0', privateStreamNow - 1_000, privateStreamNow - 1_000, -1, '0'),
      );
      savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
      });
      assert.equal(savedBook.data.book.fills.filter(fill => fill.tradeId === '9101').length, 1);
      assert.equal(savedBook.data.book.orders.find(order => order.providerOrderId === '9007199254740998')?.providerStatus, 'PARTIALLY_FILLED');
      assert.equal(savedBook.data.book.orders.find(order => order.providerOrderId === '9007199254740998')?.filledQuantity, '0.1');
      observed.push('Rust/SQLite persists Binance account and execution reports; repeated trades stay single and a late NEW event cannot roll back cumulative fills.');

      await disconnectBinancePrivateStream(existingValue, binanceConnectionStateVersion);
      for (let attempt = 0; attempt < 100; attempt += 1) {
        if ((await streamHealth.innerText()).includes('Private stream: DEGRADED · Reconciliation: DEGRADED')) break;
        await ui.waitForTimeout(50);
      }
      assert.match(await streamHealth.innerText(), /Private stream: DEGRADED · Reconciliation: DEGRADED/);
      savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
      });
      assert.equal(savedBook.data.book.status, 'STALE');
      await reconcileBinancePrivateStream(existingValue, binanceConnectionStateVersion);
      for (let attempt = 0; attempt < 100; attempt += 1) {
        if ((await streamHealth.innerText()).includes('Private stream: CONNECTED · Reconciliation: CURRENT')) break;
        await ui.waitForTimeout(50);
      }
      assert.match(await streamHealth.innerText(), /Private stream: CONNECTED · Reconciliation: CURRENT/);
      savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
      });
      const accountRetryAt = Date.parse(savedBook.data.book.rateLimits.accountRetryAt ?? '');
      if (Number.isFinite(accountRetryAt)) await ui.waitForTimeout(Math.max(0, accountRetryAt - Date.now() + 50));
      observed.push('A fixture disconnect marks saved observations stale; bounded signed REST fixtures restore CURRENT only after reconciliation succeeds.');

      const waitForBinanceReadCooldown = async () => {
        savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
          workspaceId: isolatedWorkspaceId,
          connectionId: existingValue,
        });
        const retryAt = Math.max(...[
          savedBook.data.book.rateLimits.accountRetryAt,
          savedBook.data.book.rateLimits.pendingOrdersRetryAt,
          savedBook.data.book.rateLimits.orderDetailRetryAt,
        ].map(value => Date.parse(value ?? '')).filter(Number.isFinite));
        if (Number.isFinite(retryAt)) await ui.waitForTimeout(Math.max(0, retryAt - Date.now() + 50));
      };
      const refreshBinanceExactOrder = async providerOrderId => {
        const order = binanceOrderBook.locator('.order-book-order').filter({ hasText: providerOrderId });
        const notice = ui.getByRole('status').filter({ hasText: /Binance Testnet order detail refreshed at/ });
        const previous = (await notice.allTextContents()).join('\n');
        await order.getByRole('button', { name: 'Refresh exact order', exact: true }).press('Enter');
        for (let attempt = 0; attempt < 100; attempt += 1) {
          const current = (await notice.allTextContents()).join('\n');
          if (current && current !== previous) return;
          await ui.waitForTimeout(50);
        }
        throw new Error(`Exact order ${providerOrderId} was not refreshed`);
      };
      await configureBinanceCancelFixture('PREPARE_ORDER', privateStreamNow);
      await configureBinanceCancelFixture('INVALID_QUANTITY');
      await waitForBinanceReadCooldown();
      await binanceOrderBook.getByRole('button', { name: 'Refresh open orders', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: /Binance Testnet open orders refreshed at/ }).waitFor({ state: 'visible' });

      await configureBinanceCancelFixture('REJECTED');
      await waitForBinanceReadCooldown();
      await refreshBinanceExactOrder('9007199254741001');
      const rejectedOrder = binanceOrderBook.locator('.order-book-order').filter({ hasText: '9007199254741001' });
      await waitForBinanceReadCooldown();
      await rejectedOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      let cancelDialog = ui.getByRole('dialog', { name: 'Confirm Binance Spot Testnet cancellation', exact: true });
      await cancelDialog.waitFor({ state: 'visible' });
      await cancelDialog.getByRole('button', { name: 'Confirm Testnet cancellation', exact: true }).press('Enter');
      await cancelDialog.waitFor({ state: 'hidden' });
      await rejectedOrder.getByRole('status').filter({ hasText: /Cancellation request was not accepted: PROVIDER_CANCEL_REJECTED/ }).waitFor({ state: 'visible' });
      await ui.getByRole('status').filter({ hasText: /did not accept cancellation.*PROVIDER_CANCEL_REJECTED.*Provider status: NEW/ }).waitFor({ state: 'visible' });
      await configureBinanceCancelFixture('UNKNOWN');
      await waitForBinanceReadCooldown();
      await refreshBinanceExactOrder('9007199254741002');
      const unknownOrder = binanceOrderBook.locator('.order-book-order').filter({ hasText: '9007199254741002' });
      await waitForBinanceReadCooldown();
      await unknownOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      await cancelDialog.waitFor({ state: 'visible' });
      await cancelDialog.getByRole('button', { name: 'Confirm Testnet cancellation', exact: true }).press('Enter');
      await cancelDialog.waitFor({ state: 'hidden' });
      await ui.getByRole('status').filter({ hasText: /outcome for 9007199254741002 is unknown.*remains pending provider confirmation/ }).waitFor({ state: 'visible' });
      await unknownOrder.getByRole('status').filter({ hasText: /Cancellation request accepted or outcome unknown/ }).waitFor({ state: 'visible' });
      await configureBinanceCancelFixture('RESET');

      const uncertainQuantityOrder = binanceOrderBook.locator('.order-book-order').filter({ hasText: '9007199254740997' });
      await uncertainQuantityOrder.getByText('Unavailable', { exact: true }).waitFor({ state: 'visible' });
      assert.equal(await uncertainQuantityOrder.getByRole('button', { name: 'Review cancellation', exact: true }).count(), 0,
        'An exact order with no usable remaining quantity cannot offer cancellation review');

      await configureBinanceCancelFixture('TERMINAL');
      await waitForBinanceReadCooldown();
      const terminalOrder = binanceOrderBook.locator('.order-book-order').filter({ hasText: '9007199254740999' });
      await terminalOrder.getByRole('button', { name: 'Refresh exact order', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: /Binance Testnet order detail refreshed at/ }).waitFor({ state: 'visible' });
      await terminalOrder.getByText(/ETHUSDT · BUY · FILLED/).waitFor({ state: 'visible' });
      assert.equal(await terminalOrder.getByRole('button', { name: 'Review cancellation', exact: true }).count(), 0,
        'A provider-terminal order remains visible with its terminal status and no cancellation action');

      const partialOrder = binanceOrderBook.locator('.order-book-order').filter({ hasText: '9007199254740998' });
      await waitForBinanceReadCooldown();
      await waitForBinanceReadCooldown();
      await refreshBinanceExactOrder('9007199254740998');
      await waitForBinanceReadCooldown();
      await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: /Binance Testnet order detail refreshed at/ }).waitFor({ state: 'visible' });
      cancelDialog = ui.getByRole('dialog', { name: 'Confirm Binance Spot Testnet cancellation', exact: true });
      await cancelDialog.waitFor({ state: 'visible' });
      const cancelReview = await cancelDialog.innerText();
      assert.match(cancelReview, /saved Testnet order observation/);
      assert.ok(cancelReview.includes(existingValue), 'review must identify the exact TradeX account');
      assert.match(cancelReview, /Binance Spot Testnet · TESTNET/);
      assert.match(cancelReview, /Connection ID/);
      assert.match(cancelReview, /BTCUSDT · 9007199254740998/);
      assert.match(cancelReview, /BUY · PARTIALLY_FILLED/);
      assert.match(cancelReview, /Filled quantity\s+0\.1/);
      assert.match(cancelReview, /Remaining quantity\s+0\.15/);
      for (const width of [390, 768, 1280]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Binance cancellation review overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      await ui.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Escape');
      await cancelDialog.waitFor({ state: 'hidden' });
      savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
      });
      let cancelOrder = savedBook.data.book.orders.find(order => order.providerOrderId === '9007199254740998');
      assert.equal(cancelOrder?.providerStatus, 'PARTIALLY_FILLED', 'dismissal must not cancel the provider order');
      assert.equal(cancelOrder?.cancelState, 'NONE');
      assert.equal(savedBook.data.book.fills.filter(fill => fill.tradeId === '9101').length, 1);
      const retryAtAfterReview = Date.parse(savedBook.data.book.rateLimits.accountRetryAt ?? '');
      if (Number.isFinite(retryAtAfterReview)) await ui.waitForTimeout(Math.max(0, retryAtAfterReview - Date.now() + 50));
      await partialOrder.getByRole('button', { name: 'Review cancellation', exact: true }).press('Enter');
      await ui.getByRole('status').filter({ hasText: /Binance Testnet order detail refreshed at/ }).waitFor({ state: 'visible' });
      cancelDialog = ui.getByRole('dialog', { name: 'Confirm Binance Spot Testnet cancellation', exact: true });
      await cancelDialog.waitFor({ state: 'visible' });
      await cancelDialog.getByRole('button', { name: 'Confirm Testnet cancellation', exact: true }).press('Enter');
      await cancelDialog.waitFor({ state: 'hidden' });
      savedBook = await sendIntegrationCommand('binance.testnet.orders.get', {
        workspaceId: isolatedWorkspaceId,
        connectionId: existingValue,
      });
      cancelOrder = savedBook.data.book.orders.find(order => order.providerOrderId === '9007199254740998');
      assert.equal(cancelOrder?.providerStatus, 'CANCELED');
      assert.equal(cancelOrder?.pending, false);
      assert.equal(savedBook.data.book.fills.filter(fill => fill.tradeId === '9101').length, 1, 'cancellation must retain the partial fill');
      await ui.getByRole('status').filter({ hasText: /Binance Testnet provider status for 9007199254740998: CANCELED/ }).waitFor({ state: 'visible' });
      observed.push('Binance Testnet cancellation exposes accessible rejection, unknown, and terminal outcomes; unusable remaining quantity blocks review; Escape is read-only, explicit confirmation records CANCELED, and the prior fill remains.');


      for (const width of [1280, 768, 390]) {
        await viewport.set({ width, height: 900 });
        const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
        assert.ok(size.scroll <= size.width, `Binance private stream order book overflow at ${width}px: ${JSON.stringify(size)}`);
      }
      observed.push('Binance stream health, balances, fills, and recovery remain accessible without horizontal overflow at 390px, 768px, and 1280px.');
      await viewport.set({ width: 1280, height: 900 });
      await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
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
      if (selection === 'alpaca/PAPER') {
        assert.match(await detail.innerText(), /Reconciliation\s+DEGRADED/);
        assert.match(await detail.innerText(), /Private stream\s+DEGRADED/);
        assert.match(await detail.innerText(), /Last private stream event\s+\d/);
      } else {
        assert.match(await detail.innerText(), /CONNECTED/);
      }
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
    await waitForVersionChange(ui, detail, beforeCleanupVersion, 'Local cleanup did not commit a new account state');
    await ui.getByText('MISSING', { exact: true }).waitFor({ state: 'visible' });
    assert.match(await detail.innerText(), /DISCONNECTED/);
    for (let attempt = 0; attempt < 100 && !(await remove.isEnabled()); attempt += 1) await ui.waitForTimeout(50);
    assert.equal(await remove.isEnabled(), true);
    assert.equal(await ui.getByRole('alert').count(), 0);
    const browserErrors = await tab.dev.logs({ levels: ['error'], limit: 100 });
    assert.equal(browserErrors.filter(error => !error.message.includes('chrome-extension://')).length, 0);
    observed.push('Disconnect removes local credential access; historical observations stay labeled disconnected and refresh is disabled.');
    return observed;
  } finally { await viewport.reset(); }
}

export async function checkTrading212DemoLocalDeleteUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    await viewport.set({ width: 1280, height: 900 });
    const previousDialog = ui.getByRole('dialog', { name: 'Delete local account?', exact: true });
    if (await previousDialog.isVisible()) await previousDialog.getByRole('button', { name: 'Cancel', exact: true }).press('Escape');
    const previousPath = await ui.locator('.path').innerText();
    await ui.getByRole('button', { name: 'Workspace', exact: true }).press('Enter');
    await ui.getByLabel('Workspace name', { exact: true }).fill(`S18 isolated delete ${Date.now()}`);
    await ui.getByLabel('Local storage', { exact: true }).fill(join(dirname(previousPath), `account-delete-${Date.now()}`));
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
    await ui.locator('.identity').waitFor({ state: 'visible' });
    const workspaceId = await ui.locator('.identity').innerText();
    const label = `S18 delete QA ${Date.now()}`;
    const seeded = await sendIntegrationCommand('account.delete.fixture.seed', { workspaceId, label });
    const account = seeded.data;
    const duplicate = await sendIntegrationCommand('account.delete.fixture.seed', { workspaceId, label });
    const third = await sendIntegrationCommand('account.delete.fixture.seed', { workspaceId, label });
    assert.equal(account.providerId, 'trading212');
    assert.equal(account.environment, 'DEMO');
    assert.equal(account.connectionState, 'DISCONNECTED');
    assert.equal(account.health.credential, 'MISSING');
    assert.notEqual(account.connectionId, duplicate.data.connectionId);
    assert.notEqual(account.connectionId, third.data.connectionId);
    assert.notEqual(duplicate.data.connectionId, third.data.connectionId);
    await ui.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
    assert.equal(await ui.locator('input[type="password"]').count(), 0);
    const accounts = await sendIntegrationCommand('account.list', { workspaceId });
    assert.equal(await ui.locator('.account-row').filter({ hasText: label }).count(), 3, 'Same-label Demo records render as separate rows');
    const targetIndex = accounts.data.accounts.findIndex(candidate => candidate.connectionId === account.connectionId);
    assert.ok(targetIndex >= 0, 'The selected deletion target is in the account list');
    await ui.locator('.account-row').nth(targetIndex).press('Enter');
    const detail = ui.getByRole('region', { name: label, exact: true });
    assert.match(await detail.innerText(), /DISCONNECTED/);
    assert.ok(accounts.data.accounts.some(candidate => candidate.connectionId === account.connectionId), 'The isolated disconnected Demo record remains before deletion');
    const accountIdsBeforeDelete = accounts.data.accounts.map(candidate => candidate.connectionId).sort();
    const currentAccountIds = async () => (await sendIntegrationCommand('account.list', { workspaceId })).data.accounts.map(candidate => candidate.connectionId).sort();
    const deleteButton = ui.getByRole('button', { name: 'Delete local account', exact: true });
    await deleteButton.waitFor({ state: 'visible' });
    const dialog = ui.getByRole('dialog', { name: 'Delete local account?', exact: true });
    const review = async () => {
      await deleteButton.press('Enter');
      await dialog.waitFor({ state: 'visible' });
      const text = await dialog.innerText();
      assert.ok(text.includes(label));
      assert.match(text, /trading212 · DEMO/);
      assert.ok(text.includes(account.connectionId), 'The confirmation identifies the exact selected connection');
      assert.match(text, /permanently removes TradeX-local account details and account\/order-book observations/);
      assert.match(text, /does not contact Trading 212, revoke its API key, cancel provider orders, or change any other account/);
      assert.equal(await ui.evaluate(() => document.activeElement?.textContent?.trim()), 'Cancel');
    };
    await review();
    for (const width of [768, 390]) {
      await viewport.set({ width, height: 900 });
      const size = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(size.scroll <= size.width, `Deletion dialog overflow at ${width}px: ${JSON.stringify(size)}`);
    }
    await dialog.getByRole('button', { name: 'Cancel', exact: true }).press('Enter');
    assert.equal(await dialog.isVisible(), false);
    assert.deepEqual(await currentAccountIds(), accountIdsBeforeDelete, 'Cancel preserves every same-label connection');
    await review();
    await dialog.getByRole('button', { name: 'Cancel', exact: true }).press('Escape');
    assert.equal(await dialog.isVisible(), false);
    assert.deepEqual(await currentAccountIds(), accountIdsBeforeDelete, 'Escape preserves every same-label connection');
    observed.push('Dialog names the exact connection ID despite duplicate labels; Cancel and Escape preserve every record; keyboard focus starts on Cancel and the dialog fits 768px/390px.');

    await viewport.set({ width: 1280, height: 900 });
    await review();
    await dialog.getByRole('button', { name: 'Confirm permanent deletion', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: /Deleted local account/ }).waitFor({ state: 'visible' });
    await dialog.waitFor({ state: 'hidden' });
    assert.equal(await ui.getByRole('heading', { name: label, exact: true }).count(), 0);
    assert.equal(await ui.evaluate(() => document.activeElement?.id), 'connections-title', 'Success restores focus to the Account connections heading');
    const afterDelete = await sendIntegrationCommand('account.list', { workspaceId });
    assert.equal(afterDelete.data.accounts.some(candidate => candidate.connectionId === account.connectionId), false);
    assert.deepEqual(afterDelete.data.accounts.map(candidate => candidate.connectionId).sort(), accountIdsBeforeDelete.filter(id => id !== account.connectionId).sort(), 'Other connections remain unchanged');
    for (const aggregateType of ['account', 'trading212-demo-order-book']) {
      const snapshot = await sendIntegrationCommand('domain.snapshot', { aggregateType, aggregateId: account.connectionId }, false);
      assert.equal(snapshot.error.code, 'IPC_AGGREGATE_NOT_FOUND');
    }
    observed.push('Explicit confirmation removes only the selected same-label disposable Demo account through the Account UI and Rust dispatcher.');
    const browserErrors = await tab.dev.logs({ levels: ['error'], limit: 100 });
    assert.equal(browserErrors.filter(error => !error.message.includes('chrome-extension://')).length, 0);
    return observed;
  } finally { await viewport.reset(); }
}
