// Run against the Rust-backed browser integration harness on port 1420.
// Holds or aborts only read transports; successful data comes from real Rust.
// Does not create ARCHIVED rows or claim an actual storage failure.
export async function checkThreadRecoveryS33(page) {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const consoleErrors = [];
  const expectedTransportErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const requestUrl = request.url();
    if (/^https?:\/\//i.test(requestUrl)
      && !/^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl)) {
      externalRequests.push(requestUrl.match(/^https?:\/\/[^/]+/i)?.[0] ?? requestUrl);
    }
  });
  page.on('console', message => { if (message.type() === 'error') {
    if (message.text() === 'Failed to load resource: net::ERR_FAILED') expectedTransportErrors.push(message.text());
    else consoleErrors.push(message.text());
  } });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = `s33-b2-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok, `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  await page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  ).waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 B2 history ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-b2-history-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Complete onboarding', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'What would you like to research?', exact: true }).waitFor({ state: 'visible' });

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  const navigate = async destination => {
    const mobile = page.locator('details.mobile-nav');
    if (await mobile.isVisible()) {
      if (!(await mobile.evaluate(element => element.open))) await mobile.locator('summary').press('Enter');
      await mobile.getByRole('navigation', { name: 'Primary navigation', exact: true })
        .getByRole('button', { name: destination, exact: true }).press('Enter');
    } else {
      await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
        .getByRole('button', { name: destination, exact: true }).press('Enter');
    }
  };
  const title = 'B2 recovery Research MSFT';
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill(title);
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('button', { name: /@ Context/ }).click();
  const picker = page.getByRole('dialog');
  await picker.getByRole('checkbox', { name: /equity:US:MSFT/ }).check();
  await picker.getByRole('button', { name: 'Attach', exact: true }).click();
  await page.getByRole('button', { name: 'Create Thread', exact: true }).click();
  await page.getByRole('heading', { name: title, exact: true }).waitFor({ state: 'visible' });
  const thread = (await command('thread.list', { workspaceId: workspace.workspaceId })).threads.find(item => item.title === title);
  check(Boolean(thread), 'Created Thread has its own public identity.');
  const before = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
  const cursor = (await commandLog(0)).cursor;
  let target;
  let release;
  const intercepted = [];
  await page.route('**/__integration/command', async route => {
    const envelope = route.request().postDataJSON();
    const matches = target === 'history' ? envelope.command === 'thread.list'
      : target === 'resume' && envelope.command === 'domain.snapshot'
        && envelope.payload.aggregateType === 'thread' && envelope.payload.aggregateId === thread.threadId;
    if (!matches) { await route.continue(); return; }
    intercepted.push({ target, command: envelope.command });
    await new Promise(resolve => { release = resolve; });
    await route.abort('failed');
  });
  const observations = [];
  const observe = async (scope, phase, width, locator) => {
    await locator.waitFor({ state: 'visible' });
    await locator.scrollIntoViewIfNeeded();
    const rect = await locator.boundingBox();
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth, height: innerHeight }));
    check(rect && rect.x >= 0 && rect.x + rect.width <= geometry.viewport + 1
      && rect.y >= 0 && rect.y + rect.height <= geometry.height + 1, `${scope} ${phase} reachable at ${width}`);
    check(geometry.scroll <= geometry.viewport, `${scope} ${phase} overflow at ${width}`);
    observations.push({ scope, phase, width, reachable: true, documentOverflow: false });
  };
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    target = 'history'; release = undefined;
    await page.reload();
    await navigate('Threads');
    const list = page.locator('.threads-list-card');
    await observe('history', 'loading', width, list.getByText('Loading thread history…', { exact: true }));
    check(typeof release === 'function', 'History read is held before transport failure.');
    target = undefined; release();
    await observe('history', 'failed', width, list.getByRole('alert'));
    check(await list.getByText('Thread history is unavailable.', { exact: true }).isVisible(), 'History failure is explicit.');
    const reloadHistory = list.getByRole('button', { name: 'Reload history', exact: true });
    await observe('history', 'retry', width, reloadHistory);
    if (width === 390) await list.screenshot({ path: 'docs/implementation/evidence/s33/b2-history-transport-failed-390.png' });
    await reloadHistory.press('Enter');
    const entry = list.getByRole('button', { name: thread.title, exact: true });
    await entry.waitFor({ state: 'visible' });
    target = 'resume'; release = undefined;
    await entry.press('Enter');
    await observe('resume', 'loading', width, page.locator('.threads-main').getByText('Loading thread…', { exact: true }));
    check(typeof release === 'function', 'Exact Thread snapshot is held before transport failure.');
    target = undefined; release();
    const failure = page.locator('.threads-main .error-banner');
    await observe('resume', 'failed', width, failure);
    check(await failure.getByText('Thread needs attention', { exact: true }).isVisible(), 'Resume failure is explicit.');
    const reloadThread = failure.getByRole('button', { name: 'Reload thread', exact: true });
    await observe('resume', 'retry', width, reloadThread);
    if (width === 390) await failure.screenshot({ path: 'docs/implementation/evidence/s33/b2-resume-transport-failed-390.png' });
    await reloadThread.press('Enter');
    const detail = page.locator('.thread-detail');
    await detail.getByRole('heading', { name: thread.title, exact: true }).waitFor({ state: 'visible' });
    check(await detail.getByRole('combobox', { name: 'Agent mode', exact: true }).inputValue() === 'RESEARCH', 'Recovered exact saved mode.');
    check(await detail.getByRole('button', { name: /Remove .*MSFT/ }).isVisible(), 'Recovered canonical MSFT context.');
    check(await detail.getByText('No turns have started. Send a request to begin the read-only timeline.', { exact: true }).isVisible(), 'Recovery starts no Turn.');
    await navigate('+ New Thread');
    await page.getByRole('heading', { name: 'What would you like to research?', exact: true }).waitFor({ state: 'visible' });
  }
  await page.unroute('**/__integration/command');
  const after = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
  check(JSON.stringify(before) === JSON.stringify(after), 'Read recovery preserves complete saved Thread.');
  const log = await commandLog(cursor);
  const allowed = new Set(['workspace.open', 'domain.snapshot', 'domain.subscribe', 'runtime.status', 'thread.list', 'thread.get', 'time.status', 'provider.list_definitions', 'context.catalog', 'account.list', 'account.activity', 'agent.capabilities']);
  check(log.commands.every(entry => allowed.has(entry.command)), `Unexpected recovery command: ${JSON.stringify(log.commands.filter(entry => !allowed.has(entry.command)))}`);
  check(intercepted.length === 6, `Exactly one failed history and resume read per width: ${intercepted.length}`);
  check(expectedTransportErrors.length === 6, `Exactly six expected aborted-read diagnostics: ${expectedTransportErrors.length}`);
  check(externalRequests.length === 0 && pageErrors.length === 0 && consoleErrors.length === 0, 'No unexpected browser error or external HTTP.');
  return { status: 'PASS_LIMITED', verifiedAt: new Date().toISOString(), workspaceId: workspace.workspaceId,
    threadId: thread.threadId, observations, intercepted, expectedTransportErrors: expectedTransportErrors.length,
    savedThreadUnchanged: true, commandCounts: Object.fromEntries([...allowed].map(command => [command, log.commands.filter(entry => entry.command === command).length])),
    unexpectedConsoleErrors: consoleErrors, pageErrors, externalRequests,
    limits: 'Read transport delay/abort only; real Rust data after retry. No actual storage failure, populated ARCHIVED/Saved, native accessibility or complete S33 acceptance.' };
}
