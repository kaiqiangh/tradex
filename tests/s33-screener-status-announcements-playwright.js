// Run against the isolated Rust-backed integration server on port 1422.
// Uses a disposable workspace and local screener fixture; no provider is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  const command = async (name, payload) => {
    const requestId = `s33-screener-status-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.requestId === requestId && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const readLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1422/');
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true })
    .waitFor({ state: 'visible' });
  const startingLog = await readLog(0);
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 screener announcements ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-screener-status-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Markets', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Markets', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Open screener', exact: true }).press('Enter');
  const query = page.getByLabel('Describe the market', { exact: true });
  await query.fill('US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.');
  await page.getByRole('button', { name: 'Parse conditions', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'FilterSpec and RankSpec', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Recalculate revision', exact: true }).press('Enter');
  const runButton = page.getByRole('button', { name: 'Run screen', exact: true });
  for (let attempt = 0; attempt < 100 && !(await runButton.isEnabled()); attempt += 1) await page.waitForTimeout(50);
  check(await runButton.isEnabled(), 'A reviewed local fixture revision enables Run screen.');

  const status = page.getByRole('status', { name: 'Screener status', exact: true });
  check(await status.count() === 1, 'A single persistent result status region exists before the run.');
  check(await status.getAttribute('aria-live') === 'polite', 'The result status uses polite announcements.');
  check(await status.getAttribute('aria-atomic') === 'true', 'The concise status is announced as one unit.');
  check((await status.innerText()).trim() === '', 'The persistent status is empty before a result is ready.');
  await page.evaluate(() => {
    const node = document.querySelector('[role="status"][aria-label="Screener status"]');
    const capture = { messages: [], observer: null };
    const sample = () => {
      const message = node?.textContent?.trim();
      if (message && capture.messages.at(-1) !== message) capture.messages.push(message);
    };
    capture.observer = new MutationObserver(sample);
    capture.observer.observe(node, { childList: true, characterData: true, subtree: true });
    window.__s33ScreenerAnnouncementCapture = capture;
    window.__s33ObservedScreenOperations = [];
    window.__s33OriginalFetch = window.fetch.bind(window);
    window.fetch = async (input, init) => {
      const response = await window.__s33OriginalFetch(input, init);
      const url = input instanceof Request ? input.url : String(input);
      if (new URL(url, window.location.href).pathname === '/__integration/command' && init?.method === 'POST') {
        let envelope;
        try { envelope = JSON.parse(init.body); } catch { envelope = undefined; }
        if (envelope?.command === 'market.screen') window.__s33ObservedScreenOperations.push(envelope.payload?.operation);
        if (envelope?.command === 'market.screen' && envelope.payload?.operation === 'RUN') {
          await new Promise(resolve => setTimeout(resolve, 300));
        }
      }
      return response;
    };
  });
  await runButton.press('Enter');
  await page.getByRole('heading', { name: 'COMPLETED', exact: true }).waitFor({ state: 'visible' });
  await page.waitForFunction(() => /^COMPLETED · \d+ candidates?$/.test(
    document.querySelector('[role="status"][aria-label="Screener status"]')?.textContent?.trim() ?? '',
  ));
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  const announcement = await status.innerText();
  const takeAnnouncements = async () => page.evaluate(() => {
    const capture = window.__s33ScreenerAnnouncementCapture;
    const messages = [...capture.messages];
    capture.messages.length = 0;
    return messages;
  });
  const announcements = await takeAnnouncements();
  check(/^COMPLETED · \d+ candidates?$/.test(announcement), `Unexpected concise status: ${announcement}`);
  check(JSON.stringify(announcements) === JSON.stringify(['RUNNING · Screening candidates', announcement]),
    `The screen should announce one Running transition followed by one concise final state: ${JSON.stringify(announcements)}`);
  check(!/AAPL|Source|Provider|Received|Conditions/.test(announcement),
    `Candidate/provenance detail must not be included in the live announcement: ${announcement}`);

  const resultSection = page.locator('.screener-results');
  check(await resultSection.getAttribute('aria-live') === null, 'The full result section is not a live region.');
  const candidates = page.getByRole('list', { name: 'Screener candidates', exact: true });
  check(await candidates.count() === 1 && await candidates.getByRole('listitem').count() > 0,
    'Full candidate details remain available through the normal named list.');
  const completedCandidateCount = await candidates.getByRole('listitem').count();
  check(await page.getByRole('button', { name: 'Open equity:US:AAPL market detail', exact: true }).isVisible(),
    'The candidate action remains available outside the live region.');
  const candidateLiveAncestor = await page.locator('.screener-candidates').evaluate(element =>
    element.closest('[aria-live], [role="status"], [role="alert"]')?.getAttribute('aria-label') ?? null);
  check(candidateLiveAncestor === null, 'Candidate details are outside all live/status/alert regions.');

  const responsiveWidths = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    await resultSection.scrollIntoViewIfNeeded();
    const geometry = await page.evaluate(() => {
      const result = document.querySelector('.screener-results');
      const heading = document.querySelector('#screener-results-title');
      const cards = [...document.querySelectorAll('.screener-candidate')];
      return { viewport: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        result: result?.getBoundingClientRect().toJSON(), heading: heading?.getBoundingClientRect().toJSON(),
        candidateCardsFit: cards.every(card => card.scrollWidth <= card.clientWidth) };
    });
    check(geometry.scroll <= geometry.viewport, `Screener results overflow at ${width} px: ${JSON.stringify(geometry)}`);
    check(geometry.heading && geometry.heading.left >= 0 && geometry.heading.right <= geometry.viewport,
      `The result heading remains within the ${width} px viewport: ${JSON.stringify(geometry.heading)}`);
    check(geometry.candidateCardsFit, `Candidate content overflows its own card at ${width} px: ${JSON.stringify(geometry)}`);

    const selectCandidate = page.getByRole('checkbox', { name: 'Select equity:US:MSFT', exact: true });
    await selectCandidate.scrollIntoViewIfNeeded();
    const selectBox = await selectCandidate.boundingBox();
    check(selectBox && selectBox.x >= 0 && selectBox.x + selectBox.width <= geometry.viewport,
      `The candidate selection control remains horizontally reachable at ${width} px: ${JSON.stringify(selectBox)}`);
    const openCandidate = page.getByRole('button', { name: 'Open equity:US:AAPL market detail', exact: true });
    await openCandidate.scrollIntoViewIfNeeded();
    const openBox = await openCandidate.boundingBox();
    check(openBox && openBox.x >= 0 && openBox.x + openBox.width <= geometry.viewport,
      `The candidate detail action remains horizontally reachable at ${width} px: ${JSON.stringify(openBox)}`);
    responsiveWidths.push({ width, documentOverflow: false, candidateCardsFit: geometry.candidateCardsFit,
      selectionReachable: true, detailActionReachable: true });
    if (width === 390) await resultSection.screenshot({ path: 'docs/implementation/evidence/s33/screener-results-announcement-390.png' });
  }
  await page.setViewportSize({ width: 1280, height: 900 });

  const waitForFinalAnnouncement = async state => {
    await page.waitForFunction(expected => new RegExp(`^${expected} · \\d+ candidates?$`).test(
      document.querySelector('[role="status"][aria-label="Screener status"]')?.textContent?.trim() ?? '',
    ), state);
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    return status.innerText();
  };
  const assertRunTransitions = async (state, finalText) => {
    const observed = await takeAnnouncements();
    check(JSON.stringify(observed) === JSON.stringify(['RUNNING · Screening candidates', finalText]),
      `${state} should follow one Running update with one concise result update: ${JSON.stringify(observed)}`);
  };

  await page.getByLabel('Condition 1 threshold', { exact: true }).fill('1');
  await page.getByRole('button', { name: 'Recalculate revision', exact: true }).press('Enter');
  for (let attempt = 0; attempt < 100 && !(await runButton.isEnabled()); attempt += 1) await page.waitForTimeout(50);
  check(await runButton.isEnabled(), 'A reviewed local fixture revision enables the Empty-state run.');
  const emptyPreparationAnnouncements = await takeAnnouncements();
  check(emptyPreparationAnnouncements.includes('PARSING · Checking filter conditions'),
    `Recalculating the reviewed conditions announces a single parsing state: ${JSON.stringify(emptyPreparationAnnouncements)}`);
  await runButton.press('Enter');
  await page.getByRole('heading', { name: 'EMPTY', exact: true }).waitFor({ state: 'visible' });
  const emptyAnnouncement = await waitForFinalAnnouncement('EMPTY');
  check(emptyAnnouncement === 'EMPTY · 0 candidates', `Unexpected Empty status: ${emptyAnnouncement}`);
  await assertRunTransitions('EMPTY', emptyAnnouncement);

  await query.fill('Find stocks with P/E below 20.');
  await page.getByRole('button', { name: 'Parse conditions', exact: true }).press('Enter');
  await page.getByRole('alert').filter({ hasText: 'Unsupported filter field' }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'FAILED', exact: true }).waitFor({ state: 'visible' });
  const failedAnnouncement = await waitForFinalAnnouncement('FAILED');
  check(failedAnnouncement === 'FAILED · 0 candidates', `Unexpected Failed status: ${failedAnnouncement}`);
  const failedTransitions = await takeAnnouncements();
  check(failedTransitions.at(-1) === failedAnnouncement,
    `Failed result status should be announced once: ${JSON.stringify(failedTransitions)}`);

  await query.fill('US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.');
  const beforeBlockedParse = await page.evaluate(() => window.__s33ObservedScreenOperations.length);
  await page.getByRole('button', { name: 'Parse conditions', exact: true }).press('Enter');
  await page.waitForFunction(previousCount => window.__s33ObservedScreenOperations.length > previousCount, beforeBlockedParse);
  for (let attempt = 0; attempt < 100 && !(await runButton.isEnabled()); attempt += 1) await page.waitForTimeout(50);
  check(await runButton.isEnabled(), 'Parsing the reviewed input produces a fresh revision before recalculation.');
  await page.getByRole('heading', { name: 'FilterSpec and RankSpec', exact: true }).waitFor({ state: 'visible' });
  const beforeBlockedRevision = await page.evaluate(() => window.__s33ObservedScreenOperations.length);
  await page.getByRole('button', { name: 'Recalculate revision', exact: true }).press('Enter');
  await page.waitForFunction(previousCount => window.__s33ObservedScreenOperations.length > previousCount, beforeBlockedRevision);
  for (let attempt = 0; attempt < 100 && !(await runButton.isEnabled()); attempt += 1) await page.waitForTimeout(50);
  check(await runButton.isEnabled(), 'A reviewed local fixture revision enables the blocked-external run.');
  await takeAnnouncements();
  const mode = await page.evaluate(async enabled => {
    const response = await fetch('/__integration/screener-mode', {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ enabled }),
    });
    return response.status;
  }, true);
  check(mode === 204, 'The isolated harness enables its no-network blocked-external mode.');
  try {
    await runButton.press('Enter');
    await page.getByRole('heading', { name: 'BLOCKED EXTERNAL', exact: true }).waitFor({ state: 'visible' });
    const blockedAnnouncement = await waitForFinalAnnouncement('BLOCKED EXTERNAL');
    check(blockedAnnouncement === 'BLOCKED EXTERNAL · 0 candidates', `Unexpected blocked-external status: ${blockedAnnouncement}`);
    await assertRunTransitions('BLOCKED EXTERNAL', blockedAnnouncement);
    check(await page.locator('.screener-candidate').count() === 0, 'Blocked external results contain no candidates.');
  } finally {
    const resetMode = await page.evaluate(async () => (await fetch('/__integration/screener-mode', {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ enabled: false }),
    })).status);
    check(resetMode === 204, 'The isolated harness returns to normal mode.');
  }

  const commands = (await readLog(startingLog.cursor)).commands;
  const forbidden = commands.filter(entry => entry.command.startsWith('provider.')
    || entry.command === 'account.refresh'
    || entry.command.startsWith('model.')
    || entry.command.startsWith('data.')
    || entry.command.startsWith('trade.')
    || entry.command.startsWith('paper.order.')
    || entry.command.startsWith('order.')
    || entry.command.startsWith('gateway.')
    || entry.command.startsWith('market.hot.'));
  check(forbidden.length === 0, `The local screener status test issued forbidden commands: ${JSON.stringify(forbidden)}`);
  const marketScreenCommands = commands.filter(entry => entry.command === 'market.screen').length;
  check(marketScreenCommands === 9,
    'Only the intended local screener parse, revision, and run commands are sent for Completed, Empty, Failed, and Blocked External coverage.');
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  await page.evaluate(() => {
    window.__s33ScreenerAnnouncementCapture.observer.disconnect();
    window.fetch = window.__s33OriginalFetch;
    delete window.__s33OriginalFetch;
  });
  return { result: 'PASS', announcement, announcements, emptyAnnouncement, failedAnnouncement,
    blockedExternal: 'BLOCKED EXTERNAL · 0 candidates', candidates: completedCandidateCount, forbiddenCommands: forbidden.length,
    externalRequests: externalRequests.length, responsiveWidths,
    marketScreenCommands,
    nativeVoiceOver: 'not exercised by this browser test' };
}
