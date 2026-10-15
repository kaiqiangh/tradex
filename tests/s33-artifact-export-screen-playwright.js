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
    const requestId = `s33-artifact-export-${name}-${Date.now()}-${Math.random()}`;
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
  const workspaceName = `S33 artifact export ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-artifact-export-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const testStartCursor = (await readLog(0)).cursor;
  let cursor = testStartCursor;

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Threads', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill('S33 artifact export source');
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
  await page.getByRole('button', { name: 'Create Thread', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'S33 artifact export source', exact: true }).waitFor({ state: 'visible' });
  await page.getByLabel('Turn request', { exact: true }).fill('S33 local fixture research for export');
  await page.getByRole('combobox', { name: 'Research focus', exact: true }).selectOption('EQUITY');
  await page.getByRole('button', { name: 'Preview typed research result', exact: true }).press('Enter');
  await page.getByText('Typed result · AVAILABLE', { exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Send', exact: true }).press('Enter');
  await page.waitForFunction(() => document.querySelector('[aria-label="Turn 1 status"]')?.textContent?.trim() === 'COMPLETED');
  const researchItem = page.locator('.timeline-item').filter({ has: page.getByText('research result', { exact: true }) });
  await researchItem.getByRole('button', { name: 'Save as artifact', exact: true }).press('Enter');
  await page.getByRole('textbox', { name: 'Artifact title', exact: true }).fill('S33 I4 Local Export');
  await page.getByRole('button', { name: 'Save artifact', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Artifact saved' }).waitFor({ state: 'visible' });

  const library = await command('artifact.list', { workspaceId: workspace.workspaceId });
  const summary = library.artifacts.find(item => item.title === 'S33 I4 Local Export');
  check(Boolean(summary), 'The one explicitly saved synthetic artifact appears in the workspace library.');
  const expected = await command('artifact.get', { workspaceId: workspace.workspaceId, artifactId: summary.artifactId });
  check(expected.workspaceId === workspace.workspaceId && expected.artifactId === summary.artifactId,
    'The export source is bound to the active workspace and exact artifact.');
  check(/^sha256:[0-9a-f]{64}$/.test(expected.contentHash), 'The source artifact has a canonical content hash.');

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Artifacts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Artifacts', exact: true }).waitFor({ state: 'visible' });
  const row = page.getByRole('button', { name: /S33 I4 Local Export/ });
  await row.press('Enter');
  await page.getByRole('heading', { name: 'S33 I4 Local Export', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('heading', { name: 'Saved content', exact: true }).waitFor({ state: 'visible' });

  await page.evaluate(() => {
    const originalFetch = window.fetch.bind(window);
    window.__s33ArtifactExportResponses = [];
    window.fetch = async (input, init) => {
      const response = await originalFetch(input, init);
      const url = new URL(typeof input === 'string' ? input : input.url, location.href);
      if (url.pathname === '/__integration/command' && init?.method === 'POST' && typeof init.body === 'string') {
        try {
          const envelope = JSON.parse(init.body);
          if (envelope.command === 'artifact.export') {
            window.__s33ArtifactExportResponses.push(await response.clone().json());
          }
        } catch { /* Ignore non-export or malformed test requests. */ }
      }
      return response;
    };
  });

  cursor = (await readLog(cursor)).cursor;
  await page.getByRole('button', { name: 'Export JSON', exact: true }).press('Enter');
  const success = page.getByRole('status').filter({ hasText: /^Exported / });
  await success.waitFor({ state: 'visible' });
  const successText = await success.innerText();
  const successMatch = successText.match(/^Exported (\d+) bytes to (.+)$/s);
  check(Boolean(successMatch), `The explicit export reports a local byte count and path: ${successText}`);
  const responseData = await page.waitForFunction(() => window.__s33ArtifactExportResponses?.length === 1)
    .then(() => page.evaluate(() => window.__s33ArtifactExportResponses[0]));
  check(responseData.ok && responseData.data.artifactId === expected.artifactId
    && responseData.data.contentHash === expected.contentHash,
    'The successful IPC export result preserves the selected artifact identity and content hash.');
  check(/^sha256:[0-9a-f]{64}$/.test(responseData.data.manifestHash), 'The export returns a canonical manifest hash.');
  check(Number(successMatch[1]) === responseData.data.bytes && successMatch[2] === responseData.data.path,
    'The UI status matches the successful IPC byte count and destination.');
  const exportPath = responseData.data.path;
  check(exportPath === `${workspacePath}/exports/${expected.artifactId}.json`,
    `The browser export stays under the disposable workspace exports directory: ${exportPath}`);

  const viewportChecks = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    check(await success.isVisible(), `The successful export status remains visible at ${width}px.`);
    const geometry = await page.evaluate(() => ({
      viewport: document.documentElement.clientWidth,
      page: document.documentElement.scrollWidth,
      status: (() => {
        const node = document.querySelector('.artifact-detail [role="status"]');
        if (!node) return null;
        const rect = node.getBoundingClientRect();
        return { left: rect.left, right: rect.right, width: node.clientWidth, scrollWidth: node.scrollWidth };
      })(),
    }));
    check(geometry.page <= geometry.viewport && geometry.status !== null
      && geometry.status.left >= 0 && geometry.status.right <= width + 1
      && geometry.status.scrollWidth <= geometry.status.width + 1,
      `The export feedback fits without horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    viewportChecks.push({ width, noOverflow: true, statusVisible: true });
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/artifact-export-390.png', fullPage: true });
  }

  const successfulLog = await readLog(cursor);
  check(successfulLog.commands.filter(entry => entry.command === 'artifact.export').length === 1,
    'One explicit UI activation issued exactly one successful artifact.export command.');
  cursor = successfulLog.cursor;

  await page.getByRole('button', { name: 'Export JSON', exact: true }).press('Enter');
  const duplicate = page.getByRole('alert');
  await duplicate.waitFor({ state: 'visible' });
  check((await duplicate.innerText()).includes('That export filename already exists. Choose another filename.'),
    'A second explicit export is refused because the generated target already exists.');
  await page.waitForFunction(() => window.__s33ArtifactExportResponses?.length === 2);
  const rejectedResponse = await page.evaluate(() => window.__s33ArtifactExportResponses[1]);
  check(!rejectedResponse.ok && rejectedResponse.error.code === 'ARTIFACT_EXPORT_EXISTS',
    'The existing-file refusal is returned as the typed no-replace IPC error.');
  check(rejectedResponse.error.message === 'That export filename already exists. Choose another filename.',
    'The UI displays the canonical actionable no-replace message returned by the local control plane.');
  const duplicateLog = await readLog(cursor);
  check(duplicateLog.commands.filter(entry => entry.command === 'artifact.export').length === 1,
    'The no-replace check comes from one deliberate second UI activation.');
  const errorViewportChecks = [];
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    check(await duplicate.isVisible()
      && (await duplicate.innerText()).includes('That export filename already exists. Choose another filename.'),
      `The actionable no-replace error remains visible at ${width}px.`);
    const geometry = await page.evaluate(() => {
      const node = document.querySelector('.artifact-detail [role="alert"]');
      if (!node) return null;
      const rect = node.getBoundingClientRect();
      return { viewport: document.documentElement.clientWidth, page: document.documentElement.scrollWidth,
        left: rect.left, right: rect.right, width: node.clientWidth, scrollWidth: node.scrollWidth };
    });
    check(geometry !== null && geometry.page <= geometry.viewport && geometry.left >= 0
      && geometry.right <= width + 1 && geometry.scrollWidth <= geometry.width + 1,
      `The no-replace error fits at ${width}px without document overflow: ${JSON.stringify(geometry)}`);
    errorViewportChecks.push({ width, noOverflow: true, errorVisible: true });
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/artifact-export-existing-390.png', fullPage: true });
  }

  const commands = (await readLog(testStartCursor)).commands.map(entry => entry.command);
  const allowed = new Set(['workspace.open', 'workspace.ready.fixture', 'context.catalog', 'agent.capabilities',
    'domain.snapshot', 'domain.subscribe', 'thread.list', 'thread.create', 'turn.start', 'runtime.get',
    'account.list', 'account.activity', 'research.run', 'artifact.save', 'artifact.list', 'artifact.get', 'artifact.export']);
  const unexpected = commands.filter(name => !allowed.has(name));
  check(unexpected.length === 0, `I4 issued an unexpected provider or financial command: ${JSON.stringify(unexpected)}.`);
  check(commands.filter(name => name === 'artifact.save').length === 1, 'I4 uses one explicitly saved synthetic artifact.');
  check(commands.filter(name => name === 'artifact.export').length === 2,
    'I4 performs one successful export and one explicit no-replace retry.');
  check(externalRequests.length === 0, `Browser made external HTTP requests: ${JSON.stringify(externalRequests)}.`);
  return {
    result: 'PASS', screenId: 'I4', workspace: workspace.workspaceId, workspacePath,
    artifact: expected.artifactId, contentHash: expected.contentHash, exportPath,
    manifestHash: responseData.data.manifestHash, bytes: responseData.data.bytes,
    widths: viewportChecks, errorWidths: errorViewportChecks, commands: commands.length,
    successfulExports: 1, rejectedDuplicateExports: 1,
    unexpectedCommands: unexpected.length, externalHttpRequests: externalRequests.length,
  };
}
