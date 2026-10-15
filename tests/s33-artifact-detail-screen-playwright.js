// Run with the isolated Rust-backed integration Vite service on port 1422.
// Uses a disposable workspace and one local synthetic completed Research Item only.
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
    const requestId = `s33-artifact-detail-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 artifact detail ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-artifact-detail-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const cursor = (await readLog(0)).cursor;

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Threads', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill('S33 artifact detail source');
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
  await page.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'S33 artifact detail source', exact: true }).waitFor({ state: 'visible' });
  await page.getByLabel('Turn request', { exact: true }).fill('S33 local fixture research');
  await page.getByRole('combobox', { name: 'Research focus', exact: true }).selectOption('EQUITY');
  await page.getByRole('button', { name: 'Preview typed research result', exact: true }).press('Enter');
  await page.getByText('Typed result · AVAILABLE', { exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Send', exact: true }).press('Enter');
  await page.waitForFunction(() => document.querySelector('[aria-label="Turn 1 status"]')?.textContent?.trim() === 'COMPLETED');
  const researchItem = page.locator('.timeline-item').filter({ has: page.getByText('research result', { exact: true }) });
  await researchItem.getByRole('button', { name: 'Save as artifact', exact: true }).press('Enter');
  await page.getByRole('textbox', { name: 'Artifact title', exact: true }).fill('S33 I2 Immutable Detail');
  await page.getByRole('button', { name: 'Save artifact', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Artifact saved' }).waitFor({ state: 'visible' });

  const library = await command('artifact.list', { workspaceId: workspace.workspaceId });
  const summary = library.artifacts.find(item => item.title === 'S33 I2 Immutable Detail');
  check(Boolean(summary), 'The saved synthetic artifact appears in the workspace library.');
  const expected = await command('artifact.get', { workspaceId: workspace.workspaceId, artifactId: summary.artifactId });
  check(expected.workspaceId === workspace.workspaceId && expected.artifactId === summary.artifactId,
    'The detail projection is bound to the active workspace and selected artifact.');
  check(/^sha256:[0-9a-f]{64}$/.test(expected.contentHash), 'The detail projection exposes a canonical content hash.');

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Artifacts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Artifacts', exact: true }).waitFor({ state: 'visible' });
  const row = page.getByRole('region', { name: 'Artifact library', exact: true })
    .getByRole('button', { name: /S33 I2 Immutable Detail/ });
  await row.waitFor({ state: 'visible' });
  check(await row.getAttribute('aria-current') === null, 'I2 detail stays closed until the user selects a library row.');
  await row.press('Enter');
  await page.getByRole('heading', { name: 'S33 I2 Immutable Detail', exact: true }).waitFor({ state: 'visible' });
  check(await row.getAttribute('aria-current') === 'true', 'Keyboard selection marks only the chosen artifact current.');

  const detail = page.locator('section.artifact-detail');
  await detail.waitFor({ state: 'visible' });
  check(expected.kind === 'RESEARCH' && await detail.getByText('RESEARCH · version 1', { exact: true }).isVisible(),
    'I2 shows the saved Research kind and immutable version.');
  check(await detail.getByText('Saved locally', { exact: true }).isVisible(), 'I2 identifies the workspace-local artifact.');
  const metadata = detail.locator('.artifact-meta');
  check(JSON.stringify(await metadata.locator('dt').allInnerTexts()) === JSON.stringify(['Content hash', 'Created', 'Source Item']),
    'I2 labels the saved content identity, creation time, and source item.');
  const identities = await metadata.locator('dd.identity').allInnerTexts();
  check(JSON.stringify(identities) === JSON.stringify([expected.contentHash, expected.provenance.itemId]),
    'I2 shows complete immutable content-hash and source-item identities.');
  check(await metadata.locator('time').getAttribute('datetime') === expected.createdAt,
    'I2 exposes the saved creation timestamp without substituting current state.');

  const content = detail.locator('.artifact-content');
  check(await content.locator('p').first().innerText() === expected.content.text,
    'I2 renders the saved content projection verbatim as text.');
  const displayedResearch = await content.innerText();
  const research = expected.content.researchResult;
  check(Boolean(research), 'The synthetic fixture includes its typed research projection.');
  check(displayedResearch.includes(`State: ${research.payload.state} · Tool: ${research.toolId}`),
    'I2 shows the saved typed research state and tool identity.');
  if (research.payload.conclusion) check(displayedResearch.includes(research.payload.conclusion), 'I2 shows the saved conclusion.');
  for (const finding of research.payload.findings ?? []) {
    check(displayedResearch.includes(finding.title) && displayedResearch.includes(finding.detail),
      `I2 shows saved finding ${finding.title}.`);
  }
  for (const limitation of research.payload.limitations ?? []) {
    check(displayedResearch.includes(limitation), `I2 shows saved limitation ${limitation}.`);
  }
  check(await detail.getByRole('button', { name: 'Export JSON', exact: true }).isVisible(),
    'I2 exposes the explicit export action without activating it.');

  const observations = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    check(await detail.getByRole('heading', { name: 'S33 I2 Immutable Detail', exact: true }).isVisible(),
      `I2 detail remains visible at ${width}px.`);
    const geometry = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth,
      scroll: document.documentElement.scrollWidth }));
    check(geometry.viewport <= width && geometry.scroll <= geometry.viewport,
      `I2 overflows at ${width}px: ${JSON.stringify(geometry)}.`);
    const identityLayout = await metadata.locator('dd.identity').evaluateAll(nodes => nodes.map(node => {
      const style = getComputedStyle(node);
      return { text: node.textContent?.trim(), clientWidth: node.clientWidth, scrollWidth: node.scrollWidth,
        whiteSpace: style.whiteSpace, textOverflow: style.textOverflow };
    }));
    check(identityLayout.length === 2 && identityLayout.every(identity => identity.scrollWidth <= identity.clientWidth + 1
      && identity.whiteSpace !== 'nowrap' && identity.textOverflow !== 'ellipsis'),
      `I2 full identities wrap without truncation at ${width}px: ${JSON.stringify(identityLayout)}.`);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/artifact-detail-390.png', fullPage: true,
      style: '.topbar, .skip-link { display: none !important; }' });
    observations.push({ width, detailVisible: true, fullIdentities: true, noPageOverflow: true });
  }

  const commands = (await readLog(cursor)).commands.map(entry => entry.command);
  const allowed = new Set(['workspace.open', 'workspace.ready.fixture', 'context.catalog', 'agent.capabilities',
    'domain.snapshot', 'domain.subscribe', 'thread.list', 'thread.create', 'turn.start', 'runtime.get',
    'account.list', 'account.activity', 'research.run', 'artifact.save', 'artifact.list', 'artifact.get']);
  const unexpected = commands.filter(name => !allowed.has(name));
  check(unexpected.length === 0, `I2 issued an unexpected provider, export, or financial command: ${JSON.stringify(unexpected)}.`);
  check(commands.filter(name => name === 'artifact.save').length === 1, 'I2 uses one explicitly saved local synthetic artifact.');
  check(commands.filter(name => name === 'artifact.get').length === 2,
    `I2 performs one expected direct fixture read and one UI detail read: ${JSON.stringify(commands.filter(name => name === 'artifact.get'))}.`);
  check(commands.filter(name => name === 'artifact.export').length === 0, 'I2 does not activate the I4 export action.');
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return { result: 'PASS', screenId: 'I2', workspace: workspace.workspaceId, artifact: expected.artifactId,
    contentHash: expected.contentHash, sourceItemId: expected.provenance.itemId,
    widths: [1280, 768, 390], observations, commands: commands.length,
    unexpectedCommands: unexpected.length, artifactGetCommands: 2, exportCommands: 0, externalHttpRequests: 0 };
}
