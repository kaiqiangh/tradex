// Run with `playwright-cli run-code --filename=tests/s33-artifact-provenance-dialog-playwright.js`
// against the Rust-backed `npm run dev:browser` integration harness.
// The research result and Turn use local test fixtures; no external model or broker is contacted.
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
    const requestId = `s33-artifact-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200, `${name} bridge status ${result.status}`);
    check(result.body.requestId === requestId, `${name} request id mismatch`);
    check(result.body.ok, `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const waitForCompleted = async status => {
    for (let attempt = 0; attempt < 100 && (await status.innerText()) !== 'COMPLETED'; attempt += 1) {
      await page.waitForTimeout(25);
    }
    check(await status.innerText() === 'COMPLETED', 'The synthetic local Turn completes.');
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const commandCursor = await page.evaluate(async () => (await fetch('/__integration/command-log?after=0')).json());
  const bootstrap = await command('workspace.open', {});
  const workspaceName = `S33 Artifact ${Date.now()}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-artifact-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });

  await page.getByRole('button', { name: 'Threads', exact: true }).click();
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('textbox', { name: 'Thread title', exact: true }).fill('S33 artifact provenance fixture');
  await page.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('NONE_READ_ONLY');
  await page.getByRole('button', { name: 'Create Thread', exact: true }).click();
  await page.getByRole('heading', { name: 'S33 artifact provenance fixture', exact: true }).waitFor({ state: 'visible' });
  await page.getByLabel('Turn request', { exact: true }).fill('AAPL');
  await page.getByRole('combobox', { name: 'Research focus', exact: true }).selectOption('EQUITY');
  await page.getByRole('button', { name: 'Preview typed research result', exact: true }).click();
  await page.getByText('Typed result · AVAILABLE', { exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Send', exact: true }).click();
  const turnStatus = page.getByRole('status', { name: 'Turn 1 status', exact: true });
  await turnStatus.waitFor({ state: 'visible' });
  await waitForCompleted(turnStatus);
  await page.getByRole('button', { name: 'Save as artifact', exact: true }).first().click();
  await page.getByRole('textbox', { name: 'Artifact title', exact: true }).fill('S33 synthetic provenance');
  await page.getByRole('button', { name: 'Save artifact', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Artifact saved' }).waitFor({ state: 'visible' });
  const saved = await command('artifact.list', { workspaceId });
  const savedArtifact = saved.artifacts.find(artifact => artifact.title === 'S33 synthetic provenance');
  check(Boolean(savedArtifact), 'The local research artifact is saved in the disposable workspace.');
  const artifact = await command('artifact.get', { workspaceId, artifactId: savedArtifact.artifactId });
  const expectedIdentity = [
    artifact.provenance.workspaceId,
    artifact.provenance.threadId,
    `${artifact.provenance.turnId} / ${artifact.provenance.itemId}`,
  ];
  check(expectedIdentity.every(identity => identity.length >= 32),
    `The provenance fixture uses full identifiers: ${expectedIdentity.map(identity => identity.length)}`);

  await page.getByRole('button', { name: 'Artifacts', exact: true }).click();
  await page.getByRole('heading', { name: 'Artifacts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: /S33 synthetic provenance/ }).click();
  await page.getByRole('heading', { name: 'S33 synthetic provenance', exact: true }).waitFor({ state: 'visible' });
  const trigger = page.getByRole('button', { name: 'View provenance', exact: true });
  const openDialog = async width => {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    if (width === 390) {
      await trigger.focus();
      await page.keyboard.press('Enter');
    } else await trigger.click();
    const dialog = page.getByRole('dialog', { name: 'Artifact provenance', exact: true });
    await dialog.waitFor({ state: 'visible' });
    check(await dialog.getAttribute('aria-modal') === 'true', 'Provenance is exposed as a modal dialog.');
    check(await page.evaluate(() => document.querySelector('.app-shell')?.inert) === true,
      'The background is inert while provenance is open.');
    check(await page.evaluate(() => document.activeElement?.getAttribute('aria-label')) === 'Close provenance',
      `The safe close control receives focus at ${width}px.`);
    const close = dialog.getByRole('button', { name: 'Close provenance', exact: true });
    const done = dialog.getByRole('button', { name: 'Done', exact: true });
    check(await close.count() === 1 && await done.count() === 1,
      'Close and Done expose accessible names.');
    return { dialog, close, done };
  };
  const dismissAndCheckReturn = async dialog => {
    await dialog.press('Escape');
    await dialog.waitFor({ state: 'hidden' });
    check(await page.evaluate(() => document.querySelector('.app-shell')?.inert) === false,
      'The background is restored after dismissal.');
    check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'View provenance',
      'Focus returns to the provenance trigger.');
    check(await page.getByRole('heading', { name: 'S33 synthetic provenance', exact: true }).isVisible(),
      'Dismissing provenance keeps the artifact detail unchanged.');
  };

  const first = await openDialog(1280);
  await first.close.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Done',
    'Tab moves from Close to Done.');
  await page.keyboard.press('Tab');
  check(await page.evaluate(() => document.activeElement?.getAttribute('aria-label')) === 'Close provenance',
    'Tab wraps from Done to Close.');
  await page.keyboard.press('Shift+Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Done',
    'Shift+Tab wraps from Close to Done.');
  await dismissAndCheckReturn(first.dialog);

  const narrow = await openDialog(390);
  const identityValues = await narrow.dialog.locator('.artifact-provenance-grid dd.identity').allInnerTexts();
  check(JSON.stringify(identityValues) === JSON.stringify(expectedIdentity),
    `The evidence view exposes each complete workspace/thread/turn/item identity: ${JSON.stringify(identityValues.map(value => value.length))}`);
  const identityLayout = await narrow.dialog.locator('.artifact-provenance-grid dd.identity').evaluateAll(nodes => nodes.map(node => {
    const style = getComputedStyle(node);
    const bounds = node.getBoundingClientRect();
    return {
      textLength: node.textContent?.trim().length ?? 0,
      clientWidth: node.clientWidth,
      scrollWidth: node.scrollWidth,
      textOverflow: style.textOverflow,
      whiteSpace: style.whiteSpace,
      inViewport: bounds.left >= 0 && bounds.right <= innerWidth && bounds.height > 0,
    };
  }));
  check(identityLayout.length === 3 && identityLayout.every(identity => identity.textLength >= 32
    && identity.scrollWidth <= identity.clientWidth + 1 && identity.textOverflow !== 'ellipsis'
    && identity.whiteSpace !== 'nowrap' && identity.inViewport),
    `Full evidence identities wrap without clipping at 390px: ${JSON.stringify(identityLayout)}`);
  const geometry = await page.evaluate(() => {
    const dialog = document.querySelector('[role="dialog"]');
    const rect = dialog?.getBoundingClientRect();
    const actions = [...(dialog?.querySelectorAll('.picker-dialog-actions button, .picker-dialog-heading button') ?? [])]
      .map(button => button.getBoundingClientRect());
    return {
      viewport: document.documentElement.clientWidth,
      page: document.documentElement.scrollWidth,
      left: rect?.left ?? -1,
      right: rect?.right ?? Infinity,
      actionsInsideWidth: actions.every(button => button.left >= 0 && button.right <= window.innerWidth),
    };
  });
  check(geometry.page <= geometry.viewport && geometry.left >= 0 && geometry.right <= 390 && geometry.actionsInsideWidth,
    `The narrow provenance dialog has no horizontal overflow: ${JSON.stringify(geometry)}`);
  await narrow.dialog.screenshot({ path: 'docs/implementation/evidence/s33/artifact-provenance-390.png' });
  await narrow.close.press('Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Done',
    'Tab reaches the Done action in the tall narrow dialog.');
  const doneBounds = await narrow.done.boundingBox();
  check(Boolean(doneBounds && doneBounds.y >= 0 && doneBounds.y + doneBounds.height <= 844),
    `Keyboard focus scrolls Done into view at 390px: ${JSON.stringify(doneBounds)}`);
  await page.keyboard.press('Tab');
  check(await page.evaluate(() => document.activeElement?.getAttribute('aria-label')) === 'Close provenance',
    'Tab wraps to Close in the narrow dialog.');
  await page.keyboard.press('Shift+Tab');
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Done',
    'Shift+Tab wraps to Done in the narrow dialog.');
  await dismissAndCheckReturn(narrow.dialog);

  const afterDialog = await page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), commandCursor.cursor);
  const forbiddenCommands = afterDialog.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh|data\..*\.(probe|refresh)|model\.verify_route|trade\.|live\.gateway\.|paper\.order\.|artifact\.export)/.test(entry.command));
  check(forbiddenCommands.length === 0, `The provenance dialog issues no provider or financial command: ${JSON.stringify(forbiddenCommands)}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  return {
    result: 'PASS',
    workspaceId,
    artifactId: artifact.artifactId,
    boundary: 'isolated workspace and locally synthesized research artifact; no provider integration, broker, export, or financial action',
    initialFocus: 'Close provenance',
    keyboardTrap: 'Tab and Shift+Tab wrap between Close and Done',
    dismissal: 'Escape restores background and trigger focus',
    responsiveWidth: 390,
    narrowDoneReachable: true,
    fullIdentityLengths: expectedIdentity.map(identity => identity.length),
    provenanceIdentities: expectedIdentity,
    identityWrapsWithoutClippingAt390: true,
    totalCommands: afterDialog.commands.length,
    commandCounts: Object.fromEntries([...new Set(afterDialog.commands.map(entry => entry.command))]
      .map(name => [name, afterDialog.commands.filter(entry => entry.command === name).length])),
    forbiddenCommands: forbiddenCommands.length,
    externalRequests: externalRequests.length,
  };
}
