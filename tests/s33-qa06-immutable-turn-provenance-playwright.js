// Run against the Rust-backed `npm run dev:browser` integration harness.
// Both model routes and research results are local synthetic fixtures; no model or broker endpoint is contacted.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const externalRequests = [];
  const pageErrors = [];
  page.on('request', request => {
    const url = request.url();
    const localOrigin = /^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?(?:\/|$)/i.test(url);
    if (/^https?:\/\//i.test(url) && !localOrigin) externalRequests.push(url.split('/').slice(0, 3).join('/'));
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  const command = async (name, payload) => {
    const requestId = `s33-qa06-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body?.error)}`);
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), after);
  const waitForCompleted = async status => {
    for (let attempt = 0; attempt < 100 && (await status.innerText()) !== 'COMPLETED'; attempt += 1) {
      await page.waitForTimeout(25);
    }
    check(await status.innerText() === 'COMPLETED', 'The local synthetic Turn completes.');
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1425/');
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  const cursor = await commandLog(0);
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 QA06 ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-qa06-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await command('model.route.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).click();
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  check(await page.evaluate(() => localStorage.getItem('tradex.workspace.path')) === workspacePath,
    'Opening the local workspace stores its path for reload recovery.');
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });

  await page.getByRole('button', { name: 'Threads', exact: true }).click();
  await page.getByRole('heading', { name: 'Threads', exact: true }).waitFor({ state: 'visible' });
  const newThreadComposer = page.getByRole('region', { name: 'Start a thread', exact: true });
  await newThreadComposer.getByRole('textbox', { name: 'Thread title', exact: true }).fill('US tech Research');
  await newThreadComposer.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('RESEARCH');
  const threadModel = newThreadComposer.getByRole('combobox', { name: 'Model', exact: true });
  const options = await threadModel.locator('option').evaluateAll(nodes => nodes.map(option => ({
    value: option.value,
    label: option.textContent?.trim() ?? '',
  })));
  const chatgptRoute = options.find(option => option.label.includes('gpt-5.6-sol'));
  const deepseekRoute = options.find(option => option.label.includes('DeepSeek API · deepseek-v4-flash'));
  check(Boolean(chatgptRoute && deepseekRoute), `Both local verified model routes are available: ${JSON.stringify(options)}`);
  check(await threadModel.inputValue() === chatgptRoute.value,
    'A new Thread starts with the workspace default model route.');
  const accounts = await command('account.list', { workspaceId });
  const localAccount = accounts.accounts.find(account => account.environment === 'LOCAL');
  check(Boolean(localAccount), 'The isolated workspace exposes only its Local Paper fixture.');
  await newThreadComposer.getByRole('combobox', { name: 'Account', exact: true }).selectOption(localAccount.connectionId);
  const contextTrigger = newThreadComposer.getByRole('button', { name: /@ Context/, exact: false });
  await contextTrigger.click();
  const contextDialog = page.getByRole('dialog');
  await contextDialog.waitFor({ state: 'visible' });
  await contextDialog.getByRole('checkbox', { name: /Local Paper/, exact: false }).check();
  await contextDialog.getByRole('button', { name: 'Attach', exact: true }).click();
  await page.getByText('Context references: 1', { exact: true }).waitFor({ state: 'visible' });
  await newThreadComposer.getByRole('button', { name: 'Create Thread', exact: true }).click();
  await page.getByRole('heading', { name: 'US tech Research', exact: true }).waitFor({ state: 'visible' });

  const turnComposer = page.getByRole('region', { name: 'Ask this Thread', exact: true });
  const turnModel = turnComposer.getByRole('combobox', { name: 'Model', exact: true });
  check((await turnModel.inputValue()) === chatgptRoute.value,
    'The Thread composer restores its own saved next-Turn model default.');
  await turnComposer.getByLabel('Turn request', { exact: true }).fill('Review AAPL using the local equity research fixture.');
  await turnComposer.getByRole('combobox', { name: 'Research focus', exact: true }).selectOption('EQUITY');
  await turnComposer.getByRole('button', { name: 'Preview typed research result', exact: true }).click();
  await page.getByText('Typed result · AVAILABLE', { exact: true }).waitFor({ state: 'visible' });
  await turnComposer.getByRole('button', { name: 'Send', exact: true }).click();
  const status = page.getByRole('status', { name: 'Turn 1 status', exact: true });
  await status.waitFor({ state: 'visible' });
  await waitForCompleted(status);

  const listedThreads = await command('thread.list', { workspaceId });
  const threadSummary = listedThreads.threads.find(thread => thread.title === 'US tech Research');
  check(Boolean(threadSummary?.threadId), 'The completed research Thread remains in local history.');
  const thread = await command('thread.get', { workspaceId, threadId: threadSummary.threadId });
  check(thread.turns.length === 1, 'Exactly one Turn exists before changing the next-Turn selections.');
  const originalTurn = thread.turns[0];
  const originalSnapshot = JSON.stringify(originalTurn.snapshot);
  const originalAttempts = JSON.stringify(originalTurn.providerAttempts);
  const originalTurnCard = page.locator('.turn-timeline').filter({ hasText: 'Turn 1' });
  check(originalTurn.snapshot.agentMode === 'RESEARCH', 'The historical Turn retains Research mode.');
  check(originalTurn.snapshot.executionContext === 'NONE_READ_ONLY', 'The historical Turn retains its read-only execution context.');
  check(originalTurn.snapshot.accountId === localAccount.connectionId, 'The historical Turn retains the Local Paper account identity.');
  check(originalTurn.snapshot.accountEnvironment === localAccount.environment, 'The historical Turn retains its account environment.');
  check(originalTurn.snapshot.model?.provider === 'CHATGPT' && originalTurn.snapshot.model.modelId === 'gpt-5.6-sol',
    'The historical Turn retains the model and provider selected when it started.');
  check(originalTurn.snapshot.attachedContexts.length === 1
    && originalTurn.snapshot.attachedContexts[0].kind === 'account'
    && originalTurn.snapshot.attachedContexts[0].id === localAccount.connectionId
    && originalTurn.snapshot.attachedContexts[0].hash.length > 0,
  'The historical Turn retains the canonical attached account context and hash.');
  const visibleAttempts = originalTurnCard.locator('.turn-attempt');
  check(await visibleAttempts.count() === originalTurn.providerAttempts.length && originalTurn.providerAttempts.length === 1,
    'The completed Turn exposes every persisted provider attempt in order.');
  const visibleAttempt = await visibleAttempts.first().innerText();
  check(visibleAttempt.includes(`Provider attempt: 1 · ${originalTurn.providerAttempts[0].provider} · ${originalTurn.providerAttempts[0].modelId} · ${originalTurn.providerAttempts[0].outcome}`),
    'The provider-attempt entry exposes its ordinal, provider, model, and outcome.');
  await page.setViewportSize({ width: 390, height: 844 });
  await originalTurnCard.evaluate(element => element.scrollIntoView({ block: 'start', behavior: 'instant' }));
  await page.evaluate(() => {
    const topbar = document.querySelector('.topbar');
    window.scrollBy(0, -((topbar?.getBoundingClientRect().height ?? 52) + 12));
  });
  const turnCapturePosition = await originalTurnCard.evaluate(element => ({
    cardTop: element.getBoundingClientRect().top,
    topbarBottom: document.querySelector('.topbar')?.getBoundingClientRect().bottom ?? 0,
  }));
  check(turnCapturePosition.cardTop >= turnCapturePosition.topbarBottom,
    `The Turn provenance screenshot is not overlapped by the sticky header: ${JSON.stringify(turnCapturePosition)}`);
  await originalTurnCard.screenshot({
    path: 'docs/implementation/evidence/s33/b9-turn-provenance-390.png',
    style: '.topbar, .skip-link { visibility: hidden !important; }',
  });
  await page.setViewportSize({ width: 1280, height: 900 });

  const resultItem = page.locator('.timeline-item').filter({ hasText: 'research result' }).last();
  await resultItem.getByRole('button', { name: 'Save as artifact', exact: true }).click();
  await page.getByRole('textbox', { name: 'Artifact title', exact: true }).fill('QA-06 frozen research');
  await page.getByRole('button', { name: 'Save artifact', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Artifact saved' }).waitFor({ state: 'visible' });
  const saved = await command('artifact.list', { workspaceId });
  const savedArtifact = saved.artifacts.find(artifact => artifact.title === 'QA-06 frozen research');
  check(Boolean(savedArtifact), 'The completed research item creates a local Artifact.');
  const artifactBeforeChange = await command('artifact.get', { workspaceId, artifactId: savedArtifact.artifactId });
  check(JSON.stringify(artifactBeforeChange.provenance.turnSnapshot) === originalSnapshot,
    'Artifact provenance is copied from the immutable Turn snapshot.');

  await turnModel.selectOption(deepseekRoute.value);
  await turnComposer.getByRole('combobox', { name: 'Agent mode', exact: true }).selectOption('BACKTEST');
  check(await turnModel.inputValue() === deepseekRoute.value,
    'Selecting another provider changes only the next-Turn model picker.');
  check(await turnComposer.getByRole('combobox', { name: 'Account', exact: true }).inputValue() === localAccount.connectionId,
    'Changing to Backtest preserves the selected account as optional read-only context.');
  check(await turnComposer.getByText('Context references: 1', { exact: true }).isVisible()
    && await turnComposer.locator('.context-chip').count() === 1,
  'Changing the next-Turn mode preserves its selected canonical account context.');
  check(await page.getByText('Mode: BACKTEST', { exact: true }).isVisible(), 'The composer shows the new next-Turn mode.');
  const historicalTurnCard = originalTurnCard;
  check(await historicalTurnCard.getByText('Model: CHATGPT · gpt-5.6-sol', { exact: true }).isVisible(),
    'The existing Turn card keeps its original model after changing the composer.');
  check((await historicalTurnCard.innerText()).includes('RESEARCH · NONE_READ_ONLY'),
    'The existing Turn card keeps its original mode and execution context.');
  check((await historicalTurnCard.innerText()).includes(`${localAccount.connectionId} · ${localAccount.environment}`),
    'The existing Turn card keeps its account identity.');
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    check(geometry.scroll <= geometry.width, `Turn provenance has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(await visibleAttempts.first().isVisible(), `The provider-attempt history remains visible at ${width}px.`);
  }
  await page.setViewportSize({ width: 1280, height: 900 });

  const unchangedThread = await command('thread.get', { workspaceId, threadId: threadSummary.threadId });
  check(unchangedThread.turns.length === 1, 'Changing the composer does not create another Turn.');
  check(JSON.stringify(unchangedThread.turns[0].snapshot) === originalSnapshot,
    'The persisted historical snapshot is byte-for-byte unchanged.');
  check(JSON.stringify(unchangedThread.turns[0].providerAttempts) === originalAttempts,
    'No provider attempt is appended until a Turn is sent.');
  check(unchangedThread.model?.provider === 'CHATGPT' && unchangedThread.defaultAgentMode === 'RESEARCH',
    'The temporary picker change does not rewrite the Thread defaults.');

  await page.getByRole('button', { name: 'Artifacts', exact: true }).click();
  await page.getByRole('heading', { name: 'Artifacts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: /QA-06 frozen research/ }).click();
  await page.getByRole('heading', { name: 'QA-06 frozen research', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'View provenance', exact: true }).click();
  const provenance = page.getByRole('dialog', { name: 'Artifact provenance', exact: true });
  await provenance.waitFor({ state: 'visible' });
  check(await provenance.getByText('RESEARCH', { exact: true }).isVisible(), 'Artifact provenance retains the original mode.');
  check(await provenance.getByText('NONE_READ_ONLY', { exact: true }).isVisible(), 'Artifact provenance retains the original execution context.');
  check(await provenance.getByText(`CHATGPT · gpt-5.6-sol`, { exact: true }).isVisible(), 'Artifact provenance retains the original model provider.');
  check(await provenance.getByText(`${localAccount.connectionId} · ${localAccount.environment}`, { exact: true }).isVisible(),
    'Artifact provenance retains the original account identity and environment.');
  const contextText = await provenance.locator('.artifact-provenance-grid').innerText();
  check(contextText.includes(`${originalTurn.snapshot.attachedContexts[0].kind}:${localAccount.connectionId}#${originalTurn.snapshot.attachedContexts[0].hash}`),
    'Artifact provenance retains the original canonical context ID and hash.');
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
    const geometry = await page.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
    check(geometry.scroll <= geometry.width, `Artifact provenance has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    check(await provenance.isVisible(), `The historical provenance remains visible at ${width}px.`);
  }
  await provenance.screenshot({ path: 'docs/implementation/evidence/s33/qa06-immutable-provenance-390.png' });

  const finalLog = await commandLog(cursor.cursor);
  const forbiddenCommands = finalLog.commands.filter(entry =>
    /^(provider\.(connect|probe|permissions|disconnect)|account\.refresh|data\..*\.(probe|refresh)|model\.verify_route|trade\.|live\.gateway\.|paper\.order\.|artifact\.export)/.test(entry.command));
  check(forbiddenCommands.length === 0, `The scenario issues no provider or financial command: ${JSON.stringify(forbiddenCommands)}`);
  const turnStarts = finalLog.commands.filter(entry => entry.command === 'turn.start');
  check(turnStarts.length === 1, `Exactly the synthetic research Turn is sent; selector changes send nothing: ${turnStarts.length}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);
  check(pageErrors.length === 0, `The page raised errors: ${JSON.stringify(pageErrors)}`);
  return {
    result: 'PASS_LIMITED',
    screenId: 'B4/B9/I3',
    qa: 'QA-06',
    originalTurn: { mode: originalTurn.snapshot.agentMode, executionContext: originalTurn.snapshot.executionContext,
      model: originalTurn.snapshot.model, accountId: originalTurn.snapshot.accountId,
      contextRefs: originalTurn.snapshot.attachedContexts.length },
    nextTurnSelection: { mode: 'BACKTEST', modelProvider: 'DEEPSEEK', accountPreserved: true },
    artifactProvenanceUnchanged: true,
    responsiveWidths: [1280, 768, 390],
    providerAndFinancialCommands: forbiddenCommands.length,
    turnStarts,
    externalRequests: externalRequests.length,
    pageErrors: pageErrors.length,
  };
}
