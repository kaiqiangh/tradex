// Run with the Rust-backed integration Vite service on port 1422.
// Uses a disposable local workspace and a synthetic completed Research Item only.
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
    const requestId = `s33-artifact-library-${name}-${Date.now()}-${Math.random()}`;
    const response = await page.evaluate(async envelope => {
      const result = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: result.status, body: await result.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(response.status === 200 && response.body.requestId === requestId && response.body.ok,
      `${name} failed: ${JSON.stringify(response.body.error)}`);
    return response.body.data;
  };
  const readLog = async after => page.evaluate(async cursor =>
    (await (await fetch(`/__integration/command-log?after=${cursor}`)).json()), after);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1422/');
  const isolationNotice = page.getByText('Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true });
  await isolationNotice.waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await isolationNotice.waitFor({ state: 'visible' });

  const bootstrap = await command('workspace.open', {});
  const stamp = await page.evaluate(() => crypto.randomUUID());
  const workspaceName = `S33 artifact library ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-artifact-library-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const cursor = (await readLog(0)).cursor;

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Artifacts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Artifacts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'No artifacts', exact: true }).waitFor({ state: 'visible' });
  check((await page.getByRole('heading', { name: 'No artifacts', exact: true }).innerText()) === 'No artifacts',
    'I1 presents the empty library state in the active workspace.');

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Threads', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill('S33 artifact library source');
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
  await page.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'S33 artifact library source', exact: true }).waitFor({ state: 'visible' });
  await page.getByLabel('Turn request', { exact: true }).fill('S33 local fixture research');
  await page.getByRole('combobox', { name: 'Research focus', exact: true }).selectOption('EQUITY');
  await page.getByRole('button', { name: 'Preview typed research result', exact: true }).press('Enter');
  await page.getByText('Typed result · AVAILABLE', { exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Send', exact: true }).press('Enter');
  const turnStatus = page.getByRole('status', { name: 'Turn 1 status', exact: true });
  await turnStatus.waitFor({ state: 'visible' });
  await page.waitForFunction(() => document.querySelector('[aria-label="Turn 1 status"]')?.textContent?.trim() === 'COMPLETED');
  await page.getByRole('button', { name: 'Save as artifact', exact: true }).first().press('Enter');
  await page.getByRole('textbox', { name: 'Artifact title', exact: true }).fill('S33 Library Research Evidence');
  await page.getByRole('button', { name: 'Save artifact', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Artifact saved' }).waitFor({ state: 'visible' });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Artifacts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Artifacts', exact: true }).waitFor({ state: 'visible' });
  const library = page.getByRole('region', { name: 'Artifact library', exact: true });
  const row = library.getByRole('button', { name: /S33 Library Research Evidence/ });
  await row.waitFor({ state: 'visible' });
  const rowText = await row.innerText();
  check(/\b(?:RESEARCH|DECISION)\b/.test(rowText) && rowText.includes('Thread') && rowText.includes('sha256:'),
    `I1 shows the artifact title, type, source Thread and content identity: ${rowText}`);
  check(await page.getByText('1 saved', { exact: true }).isVisible(), 'I1 reports the workspace artifact count.');
  check(await row.getAttribute('aria-current') === null, 'I1 keeps the library row unselected until the user opens it.');
  await page.getByRole('heading', { name: 'Select an artifact', exact: true }).waitFor({ state: 'visible' });

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    check(await row.isVisible(), `I1 artifact row is unavailable at ${width}px.`);
    check((await row.innerText()).includes('S33 Library Research Evidence'), `I1 row identity changed at ${width}px.`);
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `I1 overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/artifact-library-390.png',
      style: '.topbar, .skip-link { display: none !important; }' });
    observations.push({ width, rowVisible: true, noPageOverflow: true });
  }

  const commands = (await readLog(cursor)).commands.map(entry => entry.command);
  const allowed = new Set(['workspace.open', 'workspace.ready.fixture', 'context.catalog', 'agent.capabilities',
    'domain.snapshot', 'domain.subscribe', 'thread.list', 'thread.create', 'turn.start',
    'runtime.get', 'account.list', 'account.activity', 'research.run', 'artifact.save', 'artifact.list']);
  const unexpected = commands.filter(name => !allowed.has(name));
  check(unexpected.length === 0, `I1 issued an unexpected provider, detail, export, or financial command: ${JSON.stringify(unexpected)}.`);
  check(commands.filter(name => name === 'artifact.save').length === 1, 'I1 creates exactly one local synthetic artifact.');
  check(commands.filter(name => name === 'artifact.list').length >= 2, 'I1 reads both empty and populated workspace-library states.');
  check(commands.filter(name => name === 'artifact.get').length === 0, 'I1 does not open a detail before the user selects a row.');
  check(commands.filter(name => name === 'artifact.export').length === 0, 'I1 does not export from the library screen.');
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return { result: 'PASS', screenId: 'I1', workspace: workspace.workspaceId, artifacts: 1,
    widths: [1280, 768, 390], observations, commands: commands.length,
    unexpectedCommands: unexpected.length, externalHttpRequests: externalRequests.length };
}
