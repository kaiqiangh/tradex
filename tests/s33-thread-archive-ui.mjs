// Run against the Rust-backed browser integration harness on port 1420.
// Public archive/Saved/restore operations; no trusted row injection.
export async function checkThreadArchiveS33(page, withTimeline = false) {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const requestUrl = request.url();
    if (/^https?:\/\//i.test(requestUrl)
      && !/^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(requestUrl)) {
      externalRequests.push(requestUrl.match(/^https?:\/\/[^/]+/i)?.[0] ?? requestUrl);
    }
  });
  page.on('console', message => { if (message.type() === 'error') {
    consoleErrors.push(message.text());
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
  if (withTimeline) {
    const current = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
    await command('turn.start', { workspaceId: workspace.workspaceId, threadId: thread.threadId,
      expectedStateVersion: current.stateVersion, message: 'Synthetic read-only archive history',
      agentMode: 'RESEARCH', executionContext: 'NONE_READ_ONLY', model: current.model, attachedContexts: current.linkedContexts });
    await page.waitForFunction(() => document.querySelector('.thread-lifecycle button')?.disabled === true);
    await page.getByText('A Turn is running. Wait for it to finish or explicitly cancel it before archiving.', { exact: true }).waitFor({ state: 'visible' });
    await page.waitForFunction(() => document.querySelector('.turn-timeline [aria-label="Turn 1 status"]')?.textContent === 'COMPLETED');
  }
  const original = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
  if (withTimeline) check(original.codexThreadId && original.turns.length === 1 && original.turns[0].items.length > 0, 'Synthetic completed history has upstream identity and original items.');
  const stable = value => { const { status, updatedAt, stateVersion, ...rest } = value; return JSON.stringify(rest); };
  const accounts = await command('account.list', { workspaceId: workspace.workspaceId });
  const cursor = (await commandLog(0)).cursor;
  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await navigate('Threads');
    await page.locator('.threads-list-card').getByRole('button', { name: thread.title, exact: true }).press('Enter');
    const detail = page.locator('.thread-detail');
    const action = detail.getByRole('button', { name: 'Archive Thread', exact: true });
    await action.waitFor({ state: 'visible' });
    await action.press('Enter');
    const restore = detail.getByRole('button', { name: 'Restore to active', exact: true });
    await restore.waitFor({ state: 'visible' });
    await page.waitForFunction(() => document.activeElement?.textContent === 'Restore to active');
    const saved = page.locator('.threads-list-card .thread-history-group').filter({ has: page.getByRole('heading', { name: 'Saved', exact: true }) });
    await saved.getByRole('button', { name: thread.title, exact: true }).waitFor({ state: 'visible' });
    check(await detail.getByText('Archived Thread · read only', { exact: true }).isVisible(), 'Saved explains read-only status.');
    check(await detail.getByRole('button', { name: /Send|Retry Turn/ }).count() === 0, 'Saved exposes no Turn start/retry.');
    check(await detail.getByText('instrument:equity:US:MSFT', { exact: true }).isVisible(), 'Canonical MSFT context remains inspectable.');
    const captured = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
    check(captured.status === 'ARCHIVED' && stable(captured) === stable(original), 'Archive preserves exact original Thread.');
    await page.reload();
    await navigate('Threads');
    await page.locator('.threads-list-card').getByRole('button', { name: thread.title, exact: true }).press('Enter');
    await restore.waitFor({ state: 'visible' });
    await restore.scrollIntoViewIfNeeded();
    const rect = await restore.boundingBox();
    const geometry = await page.evaluate(() => ({ width: innerWidth, height: innerHeight, scroll: document.documentElement.scrollWidth }));
    check(rect && rect.x >= 0 && rect.x + rect.width <= geometry.width + 1 && rect.y >= 0 && rect.y + rect.height <= geometry.height + 1 && geometry.scroll <= geometry.width, 'Saved controls reachable without overflow.');
    const identity = detail.getByText(`Thread ${thread.threadId}`, { exact: true });
    await identity.evaluate(element => element.scrollIntoView({ block: 'center' }));
    const identityRect = await identity.boundingBox();
    const headerBottom = await page.locator('.topbar').evaluate(element => element.getBoundingClientRect().bottom);
    check(identityRect && identityRect.x >= 0 && identityRect.x + identityRect.width <= geometry.width + 1 && identityRect.y >= Math.max(0, headerBottom) && identityRect.y + identityRect.height <= geometry.height, 'Full Thread identity is reachable below sticky navigation.');
    if (width === 390) {
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: `docs/implementation/evidence/s33/b2-saved-${withTimeline ? 'timeline' : 'archive'}-390.png`, fullPage: true });
    }
    await restore.press('Enter');
    await action.waitFor({ state: 'visible' });
    await page.waitForFunction(() => document.activeElement?.textContent === 'Archive Thread');
    const active = await command('thread.get', { workspaceId: workspace.workspaceId, threadId: thread.threadId });
    check(active.status === 'ACTIVE' && stable(active) === stable(original), 'Restore preserves original identity/context/timeline.');
    const today = page.locator('.threads-list-card .thread-history-group').filter({ has: page.getByRole('heading', { name: 'Today', exact: true }) });
    await today.getByRole('button', { name: thread.title, exact: true }).waitFor({ state: 'visible' });
    observations.push({ width, savedAfterReload: true, restoredToToday: true, focusPreserved: true, completeIdentityReachable: true, documentOverflow: false });
  }
  // Hold a real UI command while another public command wins its state version.
  let held;
  let releaseStale;
  await page.route('**/__integration/command', async route => {
    const envelope = route.request().postDataJSON();
    if (envelope.command !== 'thread.archive' || held) { await route.continue(); return; }
    held = envelope.payload;
    await new Promise(resolve => { releaseStale = resolve; });
    await route.continue();
  });
  const detail = page.locator('.thread-detail');
  await detail.getByRole('button', { name: 'Archive Thread', exact: true }).press('Enter');
  await page.waitForFunction(() => document.querySelector('.thread-lifecycle button')?.textContent === 'Saving Thread status…');
  check(Boolean(held), 'UI lifecycle request held at public transport.');
  await command('thread.archive', held);
  releaseStale();
  await detail.getByRole('alert').waitFor({ state: 'visible' });
  const reload = detail.getByRole('button', { name: 'Reload thread', exact: true });
  await reload.waitFor({ state: 'visible' });
  await reload.press('Enter');
  await page.waitForFunction(() => { const b = document.querySelector('.thread-lifecycle button'); return b?.textContent === 'Restore to active' && !b.disabled; });
  await detail.getByRole('button', { name: 'Restore to active', exact: true }).press('Enter');
  await detail.getByRole('button', { name: 'Archive Thread', exact: true }).waitFor({ state: 'visible' });
  await page.unroute('**/__integration/command');
  const finalAccounts = await command('account.list', { workspaceId: workspace.workspaceId });
  check(JSON.stringify(finalAccounts) === JSON.stringify(accounts), 'Lifecycle leaves account health/arming unchanged.');
  const log = await commandLog(cursor);
  const allowed = new Set(['thread.archive','thread.restore','thread.get','thread.list','workspace.open','domain.snapshot','domain.subscribe','runtime.status','context.catalog','account.list','account.activity','agent.capabilities']);
  check(log.commands.every(entry => allowed.has(entry.command)), `Unexpected action: ${JSON.stringify(log.commands.filter(entry => !allowed.has(entry.command)))}`);
  check(log.commands.filter(entry => entry.command === 'thread.archive').length === 5 && log.commands.filter(entry => entry.command === 'thread.restore').length === 4, 'Three width cycles plus concurrent winner, refused stale archive and explicit recovery.');
  check(externalRequests.length === 0 && consoleErrors.length === 0 && pageErrors.length === 0, 'No external HTTP or browser errors.');
  return { status: 'PASS_LIMITED', verifiedAt: new Date().toISOString(), workspaceId: workspace.workspaceId, threadId: thread.threadId, observations,
    withTimeline, originalTurnCount: original.turns.length, archivedAndRestored: 4, staleUiRecovery: true, unchangedThreadFields: true, unchangedAccounts: true, externalRequests, consoleErrors, pageErrors,
    commandCounts: Object.fromEntries([...allowed].map(command => [command, log.commands.filter(entry => entry.command === command).length])),
    limits: 'Local UI/real Rust/isolated SQLite; synthetic runtime only when withTimeline; runtime refusal and reopen/event checks recorded separately. Not native VoiceOver or full S33 acceptance.' };
}
