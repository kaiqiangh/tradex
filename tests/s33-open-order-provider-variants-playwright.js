// Run against a fresh `TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser`
// harness. All provider responses below come from local synthetic fixtures.
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
    const requestId = `s33-e4-variants-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 E4 provider variants ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-e4-variants-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'EUR', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });

  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('EUR');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const trading212 = await command('account.cancellation.fixture.seed', {
    workspaceId: workspace.workspaceId, providerId: 'trading212', label: `Synthetic E4 T212 ${Date.now()}`,
  });
  await command('trading212.live.cancel.fixture.seed_order', {});
  await command('time.revalidate', { workspaceId: workspace.workspaceId });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).click();

  const checkLayout = async ({ tableSelector, row, button, screenshot }) => {
    const widths = [];
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: 900 });
      const geometry = await page.evaluate(selector => {
        const scroller = document.querySelector(selector);
        return {
          viewport: document.documentElement.clientWidth,
          documentWidth: document.documentElement.scrollWidth,
          tableClientWidth: scroller?.clientWidth ?? 0,
          tableScrollWidth: scroller?.scrollWidth ?? 0,
        };
      }, tableSelector);
      check(geometry.documentWidth <= geometry.viewport, `No document overflow at ${width}px: ${JSON.stringify(geometry)}`);
      check(geometry.tableClientWidth > 0, `The order table remains present at ${width}px: ${JSON.stringify(geometry)}`);
      if (width === 390) {
        const reachable = await row.locator('xpath=ancestor::div[contains(@class,"table-scroll")]').evaluate(element => {
          element.scrollLeft = element.scrollWidth;
          const bounds = element.getBoundingClientRect();
          const action = Array.from(element.querySelectorAll('button'))
            .find(candidate => candidate.textContent?.trim() === 'Review cancellation');
          const actionBounds = action?.getBoundingClientRect();
          return { scrollWidth: element.scrollWidth, clientWidth: element.clientWidth, left: bounds.left, right: bounds.right,
            actionLeft: actionBounds?.left, actionRight: actionBounds?.right };
        });
        check(reachable.scrollWidth > reachable.clientWidth, `The wide order table scrolls in its own region: ${JSON.stringify(reachable)}`);
        check(reachable.actionLeft >= reachable.left && reachable.actionRight <= reachable.right,
          `The cancellation control is reachable after scrolling: ${JSON.stringify(reachable)}`);
        await page.screenshot({ path: screenshot, fullPage: true });
        widths.push({ width, ...geometry, cancellationControlReachable: true });
      } else {
        widths.push({ width, ...geometry });
      }
    }
    return widths;
  };

  const openAccount = async account => {
    const accountRow = page.locator('.account-row').filter({ hasText: account.label });
    await accountRow.waitFor({ state: 'visible' });
    await accountRow.click();
    await page.getByRole('button', { name: 'Refresh account', exact: true }).click();
  };
  await openAccount(trading212);
  const t212Id = '9007199254740996';
  const t212Row = page.locator('.table-scroll[aria-label="Open orders"] tbody tr').filter({ hasText: t212Id });
  await t212Row.waitFor({ state: 'visible', timeout: 20000 });
  await t212Row.getByText(/No exact TradeX PLACE match/).waitFor({ state: 'visible' });
  const t212Text = await t212Row.innerText();
  check(t212Text.includes(t212Id) && t212Text.includes('PARTIALLY_FILLED'), `Trading 212 identity/state: ${t212Text}`);
  check(t212Text.includes('1') && t212Text.includes('0.25') && t212Text.includes('0.75'), `Trading 212 exact quantities: ${t212Text}`);
  check(t212Text.includes('Reservation: Not linked'), `Trading 212 has no false reservation link: ${t212Text}`);
  const t212Cancel = t212Row.locator('button').filter({ hasText: /^Review cancellation$/ });
  await t212Cancel.waitFor({ state: 'visible' });
  const t212Current = await command('account.get', { workspaceId: workspace.workspaceId, connectionId: trading212.connectionId });
  check(t212Current.connectionState === 'REVIEW_REQUIRED' && await t212Cancel.isDisabled(),
    'Trading 212 cancellation review stays disabled while provider permission review is required.');
  const t212Widths = await checkLayout({
    tableSelector: '.table-scroll[aria-label="Open orders"]',
    row: t212Row,
    button: t212Cancel,
    screenshot: 'docs/implementation/evidence/s33/e4-open-order-trading212-390.png',
  });

  await page.setViewportSize({ width: 1280, height: 900 });
  const bitget = await command('account.cancellation.fixture.seed', {
    workspaceId: workspace.workspaceId, providerId: 'bitget', label: `Synthetic E4 Bitget ${Date.now()}`,
  });
  await command('time.revalidate', { workspaceId: workspace.workspaceId });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('button', { name: 'Accounts', exact: true }).click();
  await openAccount(bitget);

  const bitgetTable = page.locator('.table-scroll[aria-label="Bitget Live orders"]');
  const bitgetRow = bitgetTable.locator('tbody tr').filter({ hasText: '200' }).filter({ hasText: 'NORMAL' });
  await bitgetRow.waitFor({ state: 'visible', timeout: 20000 });
  await bitgetRow.getByText(/No exact TradeX PLACE match/).waitFor({ state: 'visible' });
  const bitgetText = await bitgetRow.innerText();
  check(bitgetText.includes('200') && bitgetText.includes('live / OPEN'), `Bitget exact order identity/state: ${bitgetText}`);
  check(bitgetText.includes('0.123456') && bitgetText.includes('0.01') && bitgetText.includes('0.113456'),
    `Bitget exact base quantities: ${bitgetText}`);
  check(bitgetText.includes('Reservation: Not linked'), `Bitget has no false reservation link: ${bitgetText}`);
  const bitgetCancel = bitgetRow.locator('button').filter({ hasText: /^Review cancellation$/ });
  await bitgetCancel.waitFor({ state: 'visible' });
  const bitgetCurrent = await command('account.get', { workspaceId: workspace.workspaceId, connectionId: bitget.connectionId });
  check(bitgetCurrent.connectionState === 'REVIEW_REQUIRED' && await bitgetCancel.isDisabled(),
    'Bitget cancellation review stays disabled while provider permission review is required.');
  const bitgetWidths = await checkLayout({
    tableSelector: '.table-scroll[aria-label="Bitget Live orders"]',
    row: bitgetRow,
    button: bitgetCancel,
    screenshot: 'docs/implementation/evidence/s33/e4-open-order-bitget-390.png',
  });

  const bridgeLog = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());
  const financialCommands = bridgeLog.commands.filter(entry =>
    /^(provider\.(connect|probe)|trade\.execution\.prepare|trade\.cancel_(approve|reject)|live\.gateway\.)/.test(entry.command));
  check(financialCommands.length === 0, `The provider variants issued no connection or financial action: ${JSON.stringify(financialCommands)}`);
  check(externalRequests.length === 0, `The browser contacted an external host: ${JSON.stringify(externalRequests)}`);
  return {
    result: 'PASS',
    trading212: { brokerOrderId: t212Id, row: t212Text, cancelDisabledForReview: true, widths: t212Widths },
    bitget: { brokerOrderId: 'normal:200', row: bitgetText, cancelDisabledForReview: true, widths: bitgetWidths },
    accountRefreshes: bridgeLog.commands.filter(entry => entry.command === 'account.refresh').length,
    financialCommands: financialCommands.length,
    externalRequests: externalRequests.length,
  };
}
