// Run against a fresh `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser`
// harness. Every account/order observation is a local synthetic fixture.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  const command = async (name, payload) => {
    const requestId = `s33-e4-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200, `${name} bridge status ${result.status}`);
    check(result.body.requestId === requestId, `${name} request ID mismatch`);
    check(result.body.ok, `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 E4 Open Order ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-e4-open-order-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'EUR', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });

  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('EUR');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const account = await command('account.cancellation.fixture.seed', {
    workspaceId: workspace.workspaceId, providerId: 'binance', label: `Synthetic E4 ${Date.now()}`,
  });
  await command('binance.testnet.cancel.fixture', { scenario: 'PREPARE_ORDER' });
  await command('time.revalidate', { workspaceId: workspace.workspaceId });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).click();
  const accountRow = page.locator('.account-row').filter({ hasText: account.label });
  await accountRow.waitFor({ state: 'visible' });
  await accountRow.click();
  await page.getByRole('button', { name: 'Refresh account', exact: true }).click();

  const brokerOrderId = 'BTCUSDT:9007199254740998';
  const orderRow = page.locator('tr').filter({ hasText: brokerOrderId });
  await orderRow.waitFor({ state: 'visible', timeout: 20000 });
  await orderRow.getByText(/No exact TradeX PLACE match/).waitFor({ state: 'visible' });
  const orderText = await orderRow.innerText();
  check(orderText.includes(brokerOrderId), `Exact broker identity is visible: ${orderText}`);
  check(orderText.includes('PARTIALLY_FILLED'), `Broker state is visible: ${orderText}`);
  check(orderText.includes('0.25') && orderText.includes('0.1'), `Original and filled quantities are visible: ${orderText}`);
  check(orderText.includes('0.15'), `Exact remaining quantity is visible: ${orderText}`);
  check(orderText.includes('No exact TradeX PLACE match'), `Unlinked synthetic provider order is identified accurately: ${orderText}`);
  check(orderText.includes('Reservation: Not linked'), `No reservation is attributed without an exact TradeX PLACE match: ${orderText}`);
  const cancel = orderRow.locator('button').filter({ hasText: /^Review cancellation$/ });
  await cancel.waitFor({ state: 'visible', timeout: 10000 });
  check(await cancel.count() === 1, 'An eligible Live order exposes its exact cancellation review action.');
  check(await cancel.isEnabled(), 'The eligible Live cancellation review action is enabled.');

  const widthResults = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: 900 });
    const geometry = await page.evaluate(() => {
      const scroller = document.querySelector('.table-scroll[aria-label="Open orders"]');
      return {
        viewport: document.documentElement.clientWidth,
        documentWidth: document.documentElement.scrollWidth,
        tableClientWidth: scroller?.clientWidth ?? 0,
        tableScrollWidth: scroller?.scrollWidth ?? 0,
      };
    });
    check(geometry.documentWidth <= geometry.viewport, `No document overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(geometry.tableClientWidth > 0, `Open-order table remains present at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) {
      const reachable = await orderRow.locator('xpath=ancestor::div[contains(@class,"table-scroll")]').evaluate(element => {
        element.scrollLeft = element.scrollWidth;
        const bounds = element.getBoundingClientRect();
        const button = Array.from(element.querySelectorAll('button'))
          .find(candidate => candidate.textContent?.trim() === 'Review cancellation');
        const buttonBounds = button?.getBoundingClientRect();
        return { scrollWidth: element.scrollWidth, clientWidth: element.clientWidth, left: bounds.left, right: bounds.right,
          buttonLeft: buttonBounds?.left, buttonRight: buttonBounds?.right };
      });
      check(reachable.scrollWidth > reachable.clientWidth, `The wide order table scrolls in its own region: ${JSON.stringify(reachable)}`);
      check(reachable.buttonLeft >= reachable.left && reachable.buttonRight <= reachable.right,
        `The cancellation action is reachable after scrolling the table: ${JSON.stringify(reachable)}`);
      await page.screenshot({ path: 'docs/implementation/evidence/s33/e4-open-order-390.png', fullPage: true });
      widthResults.push({ width, ...geometry, cancelReachable: true });
    } else {
      widthResults.push({ width, ...geometry });
    }
  }

  const bridgeLog = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());
  const executionWrites = bridgeLog.commands.filter(entry =>
    /^(provider\.(connect|probe)|trade\.execution\.(prepare|order_links_fixture)|trade\.(approve|cancel_approve|cancel_reject)|live\.gateway\.)/.test(entry.command));
  check(executionWrites.length === 0, `Opening and inspecting E4 sends no connection or financial action: ${JSON.stringify(executionWrites)}`);
  check(externalRequests.length === 0, `The browser contacted an external host: ${JSON.stringify(externalRequests)}`);
  return {
    result: 'PASS',
    brokerOrderId,
    orderText,
    remainingQuantity: '0.15',
    eligibleCancellation: true,
    widths: widthResults,
    executionWrites: executionWrites.length,
    externalRequests: externalRequests.length,
  };
}
