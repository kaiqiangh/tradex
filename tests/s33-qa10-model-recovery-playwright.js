// Run against the Rust-backed integration harness with model.gateway.fixture only.
// No sidecar, OAuth flow, Keychain entry, route probe, or model inference is activated.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  page.setDefaultNavigationTimeout(30000);
  const externalRequests = [];
  const pageErrors = [];
  const envelopes = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) externalRequests.push(url.origin);
    if (url.pathname === '/__integration/command' && request.method() === 'POST') envelopes.push(request.postDataJSON());
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  const checkOk = async (command, payload) => {
    const requestId = `s33-qa10-${command}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command, payload });
    check(result.status === 200 && result.body.requestId === requestId && result.body.ok,
      `${command} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const navigate = async name => {
    const button = page.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true });
    if (!(await button.isVisible())) await page.getByText('More', { exact: true }).press('Enter');
    await button.press('Enter');
  };
  const noOverflow = async width => {
    const size = await page.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    check(size.width <= width && size.width >= width - 20 && size.scroll <= size.width,
      `No document overflow at ${width}px: ${JSON.stringify(size)}`);
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1424/');
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true }).waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const bootstrap = await checkOk('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const name = `S33 QA10 ${stamp}`;
  const path = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-qa10-${stamp}`;
  const workspace = await checkOk('workspace.open', { name, baseCurrency: 'USD', path });
  const workspaceId = workspace.workspaceId;
  await checkOk('workspace.ready.fixture', { workspaceId });
  const accounts = [];
  for (const providerId of ['binance', 'trading212']) {
    accounts.push(await checkOk('account.arming.fixture.seed', { workspaceId, providerId, label: `QA10 ${providerId} ${stamp}` }));
  }
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(name);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(path);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).click();
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).waitFor({ state: 'detached' });
  const modelBefore = await checkOk('model.get', { workspaceId });
  const accountsBefore = await checkOk('account.list', { workspaceId });

  const expectedStates = {
    STOPPED: ['Stopped', 'The selected model route is unavailable. Retry its probe or choose another route.'],
    UNAUTHORIZED: ['Unauthorized', 'The owned gateway rejected authentication. Restart to regenerate its local configuration.'],
    PORT_CONFLICT: ['Port conflict', 'Port 8317 is occupied. Close or reconfigure that service, then launch again.'],
    RUNNING: ['Running', null],
  };
  const observations = [];
  for (const [status, [label, remediation]] of Object.entries(expectedStates)) {
    await checkOk('model.gateway.fixture', { workspaceId, status });
    const accountsAfterFixture = await checkOk('account.list', { workspaceId });
    check(JSON.stringify(accountsAfterFixture.accounts) === JSON.stringify(accountsBefore.accounts),
      `Seeding model status ${status} does not mutate the account snapshot.`);
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
      await navigate('Settings');
      await page.getByRole('button', { name: 'Providers & Models', exact: true }).press('Enter');
      const models = page.locator('.model-settings');
      await models.getByRole('status').filter({ hasText: label }).first().waitFor({ state: 'visible' });
      if (remediation) check(await models.getByText(remediation, { exact: true }).isVisible(), `${status} remediation is visible.`);
      check(await models.getByRole('button', { name: 'Restart', exact: true }).isEnabled(), 'Restart is presented without auto-activation.');
      check(await page.locator('input[type="password"]').count() === 0, 'No provider secret is rendered into the webview.');
      const fallback = models.getByRole('checkbox', { name: /Allow automatic fallback to DeepSeek/ });
      check(!(await fallback.isChecked()) && await fallback.isDisabled(), 'Cross-provider fallback stays OFF and unavailable without a verified target.');
      await models.getByRole('button', { name: 'Reload model state', exact: true }).press('Enter');
      await models.getByRole('status').filter({ hasText: label }).first().waitFor({ state: 'visible' });
      await noOverflow(width);

      await page.getByRole('group', { name: 'Settings sections', exact: true })
        .getByRole('button', { name: 'Account Health', exact: true }).press('Enter');
      for (const account of accounts) {
        const card = page.locator('.account-health-card').filter({ hasText: account.label });
        await card.waitFor({ state: 'visible' });
        const text = await card.innerText();
        check(text.includes('Arming\nDISARMED'),
          `Pure model status ${status} keeps Account Health accessible with execution disarmed: ${text}`);
      }
      await noOverflow(width);
      observations.push(`${status}@${width}`);
    }
    const currentModel = await checkOk('model.get', { workspaceId });
    check(JSON.stringify(currentModel.defaultRoute) === JSON.stringify(modelBefore.defaultRoute)
      && currentModel.automaticFallback === false
      && JSON.stringify(currentModel.attempts) === JSON.stringify(modelBefore.attempts),
    `Changing gateway status preserves route, provider-attempt history, and fallback OFF: ${status}`);
  }

  await checkOk('model.gateway.fixture', { workspaceId, status: 'STOPPED' });
  const accountsAfterStop = await checkOk('account.list', { workspaceId });
  check(JSON.stringify(accountsAfterStop.accounts) === JSON.stringify(accountsBefore.accounts),
    'The final stopped fixture preserves the account snapshot.');
  await navigate('Settings');
  await page.getByRole('group', { name: 'Settings sections', exact: true })
    .getByRole('button', { name: 'Providers & Models', exact: true }).press('Enter');
  await page.locator('.model-settings').getByRole('status').filter({ hasText: 'Stopped' }).waitFor({ state: 'visible' });
  await page.setViewportSize({ width: 1280, height: 900 });
  await navigate('Threads');
  const newThread = page.getByRole('region', { name: 'Start a thread', exact: true });
  const title = `QA10 preserved draft ${stamp}`;
  await newThread.getByRole('textbox', { name: 'Thread title', exact: true }).fill(title);
  await newThread.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: title, exact: true }).waitFor({ state: 'visible' });
  const turn = page.getByRole('region', { name: 'Ask this Thread', exact: true });
  const draft = 'Keep this local draft while the model gateway is stopped.';
  await turn.getByLabel('Turn request', { exact: true }).fill(draft);
  await page.getByText('Model gateway is unavailable; the draft remains local until it is ready.', { exact: true }).waitFor({ state: 'visible' });
  check(await turn.getByRole('button', { name: 'Send', exact: true }).isDisabled(), 'The unavailable gateway blocks Send.');
  check(await turn.getByLabel('Turn request', { exact: true }).inputValue() === draft,
    'The unsent draft remains in the composer while the model gateway is unavailable.');
  check(await page.getByText('No turns have started. Send a request to begin the read-only timeline.', { exact: true }).isVisible(),
    'The unavailable model does not add a Turn to thread history.');

  const forbidden = envelopes.filter(item => /^(model\.gateway$|model\.(login_chatgpt|configure_deepseek|verify_route|set_default|set_fallback_policy)|provider\.(connect|probe|permissions|disconnect)|account\.refresh|paper\.order\.|live\.|order\.)/.test(item.command)
    || (item.command.startsWith('trade.') && item.command !== 'trade.cancel_approval.list'));
  check(forbidden.length === 0, `No gateway process, OAuth, key setup, provider, or financial action was activated: ${JSON.stringify(forbidden)}`);
  check(externalRequests.length === 0 && pageErrors.length === 0,
    `No external HTTP or browser errors: ${JSON.stringify({ externalRequests, pageErrors })}`);
  const stateCounts = Object.fromEntries(Object.keys(expectedStates).map(status => [status, 3]));
  return { result: 'PASS_LIMITED', statesChecked: stateCounts, viewportObservations: observations,
    defaultRoutePreserved: true, fallbackRemainsOff: true, accountHealthReachable: true,
    stoppedSendDisabled: true, unsentDraftPreserved: true, restartActivated: false,
    realOAuthOrModelProbe: false, forbiddenCommands: forbidden.length,
    externalRequests: externalRequests.length, pageErrors: pageErrors.length };
}
