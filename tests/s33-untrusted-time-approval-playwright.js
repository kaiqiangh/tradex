// Run with the Playwright tool against the isolated Rust-backed integration harness.
// Uses a disposable workspace and a synthetic Live fixture; no provider is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const requestUrl = request.url();
    const isHttp = /^https?:\/\//i.test(requestUrl);
    const isLocal = /^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl);
    if (isHttp && !isLocal) externalRequests.push(requestUrl.match(/^https?:\/\/[^/]+/i)?.[0] ?? requestUrl);
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-clock-uncertain-${name}-${Date.now()}-${Math.random()}`;
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
  await page.goto('http://127.0.0.1:1423/');
  const isolationNotice = page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  );
  await isolationNotice.waitFor({ state: 'visible' });
  const cursor = (await commandLog(0)).cursor;
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Clock Uncertain ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-clock-uncertain-${stamp}`;
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
    workspaceId, providerId: 'trading212', label: `S33 synthetic clock ${stamp}`,
  });
  const untrustedTime = await command('time.status', { workspaceId });
  check(untrustedTime.confidence === 'CLOCK_UNCERTAIN', `A new workspace starts with untrusted time: ${JSON.stringify(untrustedTime)}`);
  const accountProjection = await command('account.list', { workspaceId });
  const armingEligibility = accountProjection.liveArmingEligibility.find(candidate => candidate.connectionId === account.connectionId);
  check(armingEligibility?.canArm === false && armingEligibility.reasonCode === 'CLOCK_SKEW',
    `Trusted-time uncertainty is the synthetic account's specific arming blocker: ${JSON.stringify(armingEligibility)}`);

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
  check(proposal.status === 'NEEDS_APPROVAL',
    `The disposable workspace has one reviewable proposal: ${JSON.stringify(proposal)}`);

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Settings', exact: true }).waitFor({ state: 'visible' });
  const settingsSections = page.getByRole('group', { name: 'Settings sections', exact: true });
  await settingsSections.getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
  const timePanel = page.locator('.time-health');
  await timePanel.waitFor({ state: 'visible' });
  await timePanel.getByText('CLOCK UNCERTAIN', { exact: true }).waitFor({ state: 'visible' });
  const timeReason = await timePanel.locator('.notice').innerText();
  check(/synchroni[sz]e time/i.test(timeReason),
    `Account Health explains how to recover untrusted time: ${timeReason}`);
  const synchronize = timePanel.getByRole('button', { name: 'Synchronize time', exact: true });
  check(await synchronize.isVisible() && await synchronize.isEnabled(),
    'The explicit trusted-time recovery action is available without running automatically.');

  const timeViewports = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      panel: document.querySelector('.time-health')?.getBoundingClientRect(),
    }));
    check(geometry.scroll <= geometry.viewport,
      `Untrusted-time Account Health has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    const bounds = await synchronize.boundingBox();
    check(bounds && bounds.x >= -1 && bounds.x + bounds.width <= width + 1,
      `Synchronize time stays visible at ${width}px: ${JSON.stringify(bounds)}`);
    if (width === 390) {
      await timePanel.scrollIntoViewIfNeeded();
      await timePanel.screenshot({ path: 'docs/implementation/evidence/s33/clock-uncertain-health-390.png' });
    }
    timeViewports.push({ width, noHorizontalOverflow: true, recoveryVisible: true });
  }

  await page.getByText('More', { exact: true }).click();
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.locator('.order-proposal-row').first().press('Enter');
  const reviewButton = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await reviewButton.waitFor({ state: 'visible' });
  check(await reviewButton.isEnabled(), 'The user can inspect why the proposal is blocked.');
  await reviewButton.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const arm = dialog.getByRole('button', { name: 'Arm this Live account', exact: true });
  check(!(await arm.isEnabled()), 'Untrusted time disables the explicit Live arming action.');
  const reviewText = await dialog.innerText();
  check(/CLOCK_UNTRUSTED|clock|time|freshness/i.test(reviewText),
    `The arming review explains the trusted-time blocker: ${reviewText}`);
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'The safe dismissal action receives initial focus while time is untrusted.');

  const approvals = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(approvals.approvals.length === 0, 'No approval is persisted while trusted time is unavailable.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'Untrusted time exposes no Prepare action.');

  const reviewViewports = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth,
      dialog: document.querySelector('[role="dialog"]')?.getBoundingClientRect(),
    }));
    check(geometry.scroll <= geometry.viewport,
      `Untrusted approval review has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    for (const name of ['Keep reviewing', 'Arm this Live account']) {
      const button = dialog.getByRole('button', { name, exact: true });
      const bounds = await button.boundingBox();
      check(bounds && bounds.x >= -1 && bounds.x + bounds.width <= width + 1,
        `${name} stays within ${width}px: ${JSON.stringify(bounds)}`);
    }
    if (width === 390) {
      await arm.scrollIntoViewIfNeeded();
      await dialog.screenshot({ path: 'docs/implementation/evidence/s33/clock-uncertain-arming-390.png' });
    }
    reviewViewports.push({ width, noHorizontalOverflow: true, blockerAndSafeActionVisible: true });
  }

  const after = await commandLog(cursor);
  const forbidden = after.commands.filter(entry => {
    const name = entry.command;
    return /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh)$/.test(name)
      || name.startsWith('alpaca.') || name.startsWith('binance.') || name.startsWith('bitget.')
      || name.startsWith('trading212.') || name.startsWith('live.gateway.')
      || name === 'account.arm' || name === 'account.disable_all_live'
      || name.startsWith('trade.prepare') || name === 'trade.approval.issue'
      || name === 'trade.approve' || name === 'trade.reject'
      || (name.startsWith('trade.execution.') && !name.endsWith('.preparation.get'));
  });
  check(forbidden.length === 0,
    `Clock failure makes no provider, approval, Prepare, execution, or Gateway call: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0, `No external HTTP requests occurred: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}`);

  return {
    result: 'PASS',
    screenId: 'K6',
    workspace: workspace.workspaceId,
    boundary: 'disposable Rust-backed workspace with local synthetic account and proposal; no credentials or provider requests',
    timeConfidence: untrustedTime.confidence,
    armingReasonCode: armingEligibility.reasonCode,
    armEnabled: await arm.isEnabled(),
    persistedApprovals: approvals.approvals.length,
    prepareActions: 0,
    timeViewports,
    reviewViewports,
    commands: after.commands.map(entry => entry.command),
    commandCount: after.commands.length,
    forbiddenCommands: forbidden,
    externalHttpRequests: externalRequests.length,
    consoleErrors,
    pageErrors,
  };
}
