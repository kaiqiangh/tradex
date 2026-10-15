// Run with `playwright-cli run-code --filename=tests/s33-global-disable-before-prepare-playwright.js`
// against the Rust-backed `npm run dev:browser` integration harness.
// Uses a disposable workspace and a synthetic Live fixture; no provider is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const command = async (name, payload) => {
    const requestId = `s33-disable-all-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const commandLog = async cursor => page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), cursor);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Disable All ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-disable-all-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const workspaceId = workspace.workspaceId;
  const policy = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: policy.stateVersion,
    policy: { ...policy.policy, maxSingleInstrumentExposurePercent: null, staleQuoteThresholdSeconds: 120 },
  });
  const account = await command('account.arming.fixture.seed', {
    workspaceId, providerId: 'trading212', label: `S33 synthetic disable ${stamp}`,
  });
  await command('time.revalidate', { workspaceId });
  await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
  await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
  await page.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('510');
  await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const proposals = await command('trade.proposal.list', { workspaceId });
  check(proposals.proposals.length === 1, 'The disposable workspace has one synthetic Live proposal.');
  const proposal = proposals.proposals[0];
  await page.locator('.order-proposal-row').first().press('Enter');
  await page.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const approvalDialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await approvalDialog.waitFor({ state: 'visible' });
  const approve = approvalDialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  check(await approve.isEnabled(), 'The synthetic, eligible proposal exposes its explicit approval action.');
  const gatewayBefore = await command('live.gateway.fixture.inspect', {});
  const beforeDisable = await commandLog(0);
  await approve.press('Space');
  await approvalDialog.waitFor({ state: 'hidden' });
  const issued = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(issued.approvals.length === 1 && issued.approvals[0].status === 'ISSUED',
    'The synthetic approval is issued explicitly, before Prepare and dispatch.');

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Accounts', exact: true }).waitFor({ state: 'visible' });
  const disableAll = page.getByRole('button', { name: 'Disable All Live Execution', exact: true });
  await disableAll.waitFor({ state: 'visible' });
  await disableAll.press('Enter');
  await page.getByRole('status').filter({ hasText: 'All Live accounts are disarmed.' }).waitFor({ state: 'visible' });

  const accounts = await command('account.list', { workspaceId });
  const disabled = accounts.accounts.find(item => item.connectionId === account.connectionId);
  check(disabled?.health.arming === 'DISARMED' && disabled.health.armingReason === 'USER_DISABLED_ALL',
    'Disable All persists the global disarm reason on the exact synthetic account.');
  const approvalsAfterDisable = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(approvalsAfterDisable.approvals[0]?.status === 'INVALIDATED',
    'Global disarm invalidates the unconsumed approval before Prepare.');
  const gatewayAfter = await command('live.gateway.fixture.inspect', {});
  check(gatewayAfter.requests.length === gatewayBefore.requests.length,
    'Global disarm before Prepare creates no local fake Gateway request.');

  const afterDisable = await commandLog(beforeDisable.cursor);
  const forbidden = afterDisable.commands.filter(entry => {
    const name = entry.command;
    return /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(name)
      || (name.startsWith('trade.execution.') && !name.endsWith('.preparation.get'))
      || name.startsWith('trade.prepare')
      || (name.startsWith('live.gateway.') && !name.endsWith('.fixture.inspect'));
  });
  check(forbidden.length === 0,
    `The scenario made no provider, Prepare, execution, or Gateway command: ${JSON.stringify(forbidden)}`);

  const responsive = [];
  for (const width of [768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await disableAll.scrollIntoViewIfNeeded();
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      control: document.querySelector('#disable-all-live')?.getBoundingClientRect(),
      height: window.innerHeight,
    }));
    check(geometry.scroll <= geometry.viewport, `Accounts has no horizontal overflow at ${width}px.`);
    check(geometry.control && geometry.control.left >= 0 && geometry.control.right <= width
      && geometry.control.top >= 0 && geometry.control.bottom <= geometry.height,
    `Disable All remains visible at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/disable-all-live-390.png' });
    responsive.push(width);
  }

  return {
    result: 'PASS',
    boundary: 'disposable workspace and synthetic Trading 212-labelled Live account; no provider endpoint or credentials',
    approvalBeforeDisable: issued.approvals[0].status,
    accountAfterDisable: disabled.health.arming,
    approvalAfterDisable: approvalsAfterDisable.approvals[0].status,
    fakeGatewayRequestsAdded: gatewayAfter.requests.length - gatewayBefore.requests.length,
    forbiddenCommands: forbidden.length,
    responsiveWidths: responsive,
  };
}
