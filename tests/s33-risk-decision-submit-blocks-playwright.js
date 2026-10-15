// Run with playwright-cli run-code against TRADEX_LIVE_APPROVAL_FIXTURE=1 npm run dev:browser.
// Uses a disposable synthetic Binance Live fixture; missing USDT/USD FX blocks Prepare before Gateway dispatch.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  page.setDefaultTimeout(10000);
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.on('request', request => {
    const url = request.url();
    if (/^https?:\/\//i.test(url) && !/^http:\/\/(127\.0\.0\.1|localhost)(:\d+)?\//i.test(url)) {
      externalRequests.push(url.replace(/^(https?:\/\/[^/]+).*/, '$1'));
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));
  const isUiRiskDecisionRead = request => {
    if (request.method() !== 'POST' || !request.url().endsWith('/__integration/command')) return false;
    try {
      const envelope = request.postDataJSON();
      return envelope.command === 'risk.decision.list'
        && !String(envelope.requestId ?? '').startsWith('s33-risk-submit-');
    } catch { return false; }
  };
  const waitForUiRiskDecisionRefresh = () => page.waitForRequest(isUiRiskDecisionRead);
  const command = async (name, payload) => {
    const requestId = `s33-risk-submit-${name}-${Date.now()}-${Math.random()}`;
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
  const commandLog = async after => page.evaluate(async cursor =>
    (await fetch(`/__integration/command-log?after=${cursor}`)).json(), after);
  const assertResponsive = async (label, screenshot) => {
    const widths = [];
    for (const width of [1280, 768, 390]) {
      await page.setViewportSize({ width, height: width === 390 ? 844 : 900 });
      const geometry = await page.evaluate(() => {
        const root = document.documentElement;
        const error = document.querySelector('.error-recovery-panel')?.getBoundingClientRect();
        const risk = document.querySelector('.risk-decision-panel')?.getBoundingClientRect();
        return { viewport: root.clientWidth, scroll: root.scrollWidth,
          errorLeft: error?.left ?? -1, errorRight: error?.right ?? Infinity,
          riskLeft: risk?.left ?? -1, riskRight: risk?.right ?? Infinity };
      });
      check(geometry.scroll <= width && geometry.errorLeft >= -1 && geometry.errorRight <= width + 1
        && geometry.riskLeft >= -1 && geometry.riskRight <= width + 1,
      `${label} and RiskDecision fit at ${width}px: ${JSON.stringify(geometry)}`);
      widths.push(width);
      if (width === 390 && screenshot) {
        await page.locator('.error-banner').screenshot({ path: screenshot });
        await page.locator('.risk-decision-history > li').first().locator('article')
          .screenshot({ path: screenshot.replace(/\.png$/, '-risk-decision.png') });
      }
    }
    await page.setViewportSize({ width: 1280, height: 900 });
    return widths;
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const isolationNotice = page.getByText(
    'Browser verification · isolated temporary workspace · provider responses are test fixtures', { exact: true },
  );
  await isolationNotice.waitFor({ state: 'visible' });
  await page.evaluate(() => { localStorage.clear(); sessionStorage.clear(); });
  await page.reload();
  await isolationNotice.waitFor({ state: 'visible' });

  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Risk Evidence ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-risk-evidence-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  const workspaceId = workspace.workspaceId;
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  const scenarioCursor = (await commandLog(0)).cursor;

  let policy = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: policy.stateVersion,
    policy: { ...policy.policy, maxOrderNotional: '1', maxSingleInstrumentExposurePercent: null, staleQuoteThresholdSeconds: 120 },
  });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('LOCAL_PAPER');
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('2');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('100');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const localProposalSummary = (await command('trade.proposal.list', { workspaceId })).proposals[0];
  const localProposal = await command('trade.proposal.get', { workspaceId, proposalId: localProposalSummary.proposalId });
  await page.getByRole('button', { name: 'Evaluate risk', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: /RiskDecision REJECTED saved at/ }).waitFor({ state: 'visible' });
  const rejectedDecisionArticle = page.getByRole('article', { name: 'REJECTED RiskDecision', exact: true });
  await rejectedDecisionArticle.waitFor({ state: 'visible' });
  check((await rejectedDecisionArticle.innerText()).includes('ORDER_NOTIONAL · REJECT'),
    'The Local Paper UI displays deterministic rejection and the exact failed policy check.');
  const rejectedDecision = (await command('risk.decision.list', {
    workspaceId, proposalId: localProposal.proposalId,
  })).decisions.at(-1);
  check(rejectedDecision?.status === 'REJECTED' && rejectedDecision.checks.some(check =>
    check.checkId === 'ORDER_NOTIONAL' && check.outcome === 'REJECT'),
  `The rejected decision is persisted: ${JSON.stringify(rejectedDecision)}`);

  await page.getByRole('button', { name: 'Submit Local Paper order', exact: true }).press('Enter');
  const localDialog = page.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true });
  await localDialog.waitFor({ state: 'visible' });
  const rejectedDecisionRefresh = waitForUiRiskDecisionRefresh();
  await localDialog.getByRole('button', { name: 'Confirm submit', exact: true }).press('Enter');
  await rejectedDecisionRefresh;
  const rejectedError = page.locator('.error-recovery-panel[data-error-code="RISK_REJECTED"]');
  await rejectedError.waitFor({ state: 'visible' });
  const rejectedText = await rejectedError.innerText();
  check(rejectedText.includes('No order attempt or provider/simulator order side effect was created.'),
    'RISK_REJECTED explicitly reports the absence of order side effects.');
  check(rejectedText.includes('recorded rejection and its blocking risk or capacity checks'),
    'RISK_REJECTED directs the user to the persisted rejection details.');
  const submittedRejectedDecision = (await command('risk.decision.list', {
    workspaceId, proposalId: localProposal.proposalId,
  })).decisions.at(-1);
  check(submittedRejectedDecision?.status === 'REJECTED'
    && submittedRejectedDecision.decisionId !== rejectedDecision.decisionId
    && submittedRejectedDecision.checks.some(check => check.checkId === 'ORDER_NOTIONAL' && check.outcome === 'REJECT'),
  'The rejected submit appends and persists a fresh blocking decision.');
  await page.getByRole('article', { name: 'REJECTED RiskDecision', exact: true })
    .filter({ hasText: submittedRejectedDecision.decisionId }).waitFor({ state: 'visible' });
  const localPaper = await command('paper.get', { workspaceId });
  check(localPaper.orders.length === 0 && localPaper.fills.length === 0,
    'The rejected Local Paper submission creates no simulator order or fill.');
  check((await command('trade.proposal.get', { workspaceId, proposalId: localProposal.proposalId })).status === 'NEEDS_APPROVAL',
    'The rejected immutable Local Paper proposal remains unconsumed.');
  const rejectedWidths = await assertResponsive('RISK_REJECTED',
    'docs/implementation/evidence/s33/risk-rejected-390.png');

  policy = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: policy.stateVersion,
    policy: { ...policy.policy, maxOrderNotional: null, maxDailyRealizedLoss: '1' },
  });

  await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('LOCAL_PAPER');
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('1');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('100');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const unavailableProposalSummary = (await command('trade.proposal.list', { workspaceId })).proposals[0];
  const unavailableProposal = await command('trade.proposal.get', {
    workspaceId, proposalId: unavailableProposalSummary.proposalId,
  });
  await page.getByRole('button', { name: 'Evaluate risk', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: /RiskDecision UNAVAILABLE saved at/ }).waitFor({ state: 'visible' });
  const unavailableDecision = (await command('risk.decision.list', {
    workspaceId, proposalId: unavailableProposal.proposalId,
  })).decisions.at(-1);
  check(unavailableDecision?.status === 'UNAVAILABLE' && unavailableDecision.checks.some(check =>
    check.checkId === 'DAILY_REALIZED_LOSS' && check.outcome === 'UNAVAILABLE'),
  `The unavailable decision is persisted with its missing counter evidence: ${JSON.stringify(unavailableDecision)}`);
  await page.getByRole('article', { name: 'UNAVAILABLE RiskDecision', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Submit Local Paper order', exact: true }).press('Enter');
  const unavailablePaperDialog = page.getByRole('dialog', { name: 'Confirm Local Paper submission', exact: true });
  await unavailablePaperDialog.waitFor({ state: 'visible' });
  await unavailablePaperDialog.getByRole('button', { name: 'Confirm submit', exact: true }).press('Enter');
  const unavailableSubmitError = page.locator('.error-recovery-panel[data-error-code="RISK_EVIDENCE_UNAVAILABLE"]');
  await unavailableSubmitError.waitFor({ state: 'visible' });
  const unavailableSubmitText = await unavailableSubmitError.innerText();
  check(unavailableSubmitText.includes('No order attempt or provider/simulator order side effect was created.'),
    'Unavailable evidence explicitly reports the absence of order side effects.');
  check(unavailableSubmitText.includes('persisted RiskDecision and execution evidence'),
    'Unavailable evidence directs the user to the saved decision and missing execution evidence.');
  const submittedUnavailableDecision = (await command('risk.decision.list', {
    workspaceId, proposalId: unavailableProposal.proposalId,
  })).decisions.at(-1);
  check(submittedUnavailableDecision?.status === 'UNAVAILABLE'
    && submittedUnavailableDecision.decisionId !== unavailableDecision.decisionId,
  'The failed submit appends and persists a fresh unavailable decision.');
  await page.getByRole('article', { name: 'UNAVAILABLE RiskDecision', exact: true })
    .filter({ hasText: submittedUnavailableDecision.decisionId }).waitFor({ state: 'visible' });
  const simulatorAfterUnavailable = await command('paper.get', { workspaceId });
  check(simulatorAfterUnavailable.orders.length === 0 && simulatorAfterUnavailable.fills.length === 0,
    'The unavailable Local Paper submission creates no simulator order or fill.');
  check((await command('trade.proposal.get', {
    workspaceId, proposalId: unavailableProposal.proposalId,
  })).status === 'NEEDS_APPROVAL', 'The unavailable immutable proposal remains unconsumed.');
  const unavailableWidths = await assertResponsive('RISK_EVIDENCE_UNAVAILABLE',
    'docs/implementation/evidence/s33/risk-evidence-unavailable-local-paper-390.png');

  policy = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: policy.stateVersion,
    policy: { ...policy.policy, maxDailyRealizedLoss: null },
  });
  const account = await command('account.arming.fixture.seed', {
    workspaceId, providerId: 'binance', label: `S33 synthetic FX evidence ${stamp}`,
  });
  const time = await command('time.revalidate', { workspaceId });
  check(time.confidence === 'TRUSTED', 'The synthetic fixture has trusted time.');
  const armed = await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });
  check(armed.health.arming === 'ARMED', 'Only the synthetic Binance-labelled account is armed.');

  const navigation = page.getByRole('navigation', { name: 'Primary navigation', exact: true });
  await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Account connections', exact: true }).waitFor({ state: 'visible' });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
  await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
  await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const proposalList = await command('trade.proposal.list', { workspaceId });
  let liveProposalSummary;
  for (const summary of proposalList.proposals) {
    const detail = await command('trade.proposal.get', { workspaceId, proposalId: summary.proposalId });
    if (detail.fields.environment === 'BINANCE_LIVE') liveProposalSummary = summary;
  }
  check(liveProposalSummary, 'The workspace has a synthetic Binance Live proposal beside the risk-rejected Local Paper proposal.');
  const proposal = await command('trade.proposal.get', { workspaceId, proposalId: liveProposalSummary.proposalId });
  check(proposal.fields.environment === 'BINANCE_LIVE' && proposal.fields.accountId === account.connectionId
    && proposal.fields.instrumentId === 'crypto:BTC/USDT:spot' && proposal.fields.quantity.value === '0.01',
  'The immutable proposal binds the synthetic account and exact BTC/USDT order.');
  await page.locator('.order-proposal-row').filter({ hasText: proposal.proposalId }).press('Enter');
  const gatewayBefore = await command('live.gateway.fixture.inspect', {});
  const postsBefore = gatewayBefore.requests.filter(request => request.method === 'POST').length;

  await page.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const approve = dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  check(await approve.isEnabled(), 'The exact synthetic proposal passes current approval review.');
  await approve.press('Space');
  await dialog.waitFor({ state: 'hidden' });
  const issued = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(issued.approvals.length === 1 && issued.approvals[0].status === 'ISSUED',
    'The user action created one bounded approval before Prepare.');
  const persistedDecisions = (await command('risk.decision.list', { workspaceId, proposalId: proposal.proposalId })).decisions;
  const savedDecision = persistedDecisions.at(-1);
  check(savedDecision && savedDecision.proposalHash === proposal.proposalHash
    && savedDecision.accountId === account.connectionId && savedDecision.environment === 'BINANCE_LIVE',
  `The saved RiskDecision binds this proposal, account and environment: ${JSON.stringify(savedDecision)}`);
  await page.getByRole('article', { name: `${savedDecision.status} RiskDecision`, exact: true }).waitFor({ state: 'visible' });

  const preparationDecisionRefresh = waitForUiRiskDecisionRefresh();
  await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).press('Enter');
  await preparationDecisionRefresh;
  const unavailableError = page.locator('.error-recovery-panel[data-error-code="RISK_EVIDENCE_UNAVAILABLE"]');
  await unavailableError.waitFor({ state: 'visible' });
  const errorText = await unavailableError.innerText();
  check(errorText.includes('Risk evidence is unavailable'), 'The UI names unavailable evidence distinctly.');
  check(errorText.includes('No order attempt or provider/simulator order side effect was created.'),
    'The error explicitly states that no attempt or provider/simulator order side effect was created.');
  check(errorText.includes('persisted RiskDecision and execution evidence'),
    'The error directs review to the saved decision and relevant execution evidence.');
  const decisionsAfterPreparation = (await command('risk.decision.list', {
    workspaceId, proposalId: proposal.proposalId,
  })).decisions;
  const latestPreparationDecision = decisionsAfterPreparation.at(-1);
  check(latestPreparationDecision?.proposalHash === proposal.proposalHash
    && latestPreparationDecision.accountId === account.connectionId
    && latestPreparationDecision.environment === 'BINANCE_LIVE'
    && latestPreparationDecision.status === 'ALLOWED'
    && latestPreparationDecision.decisionId === savedDecision.decisionId,
  'The error-triggered refresh displays the exact persisted ALLOWED review; missing FX remains a downstream execution-evidence blocker.');
  await page.getByRole('article', { name: `${latestPreparationDecision.status} RiskDecision`, exact: true })
    .filter({ hasText: latestPreparationDecision.decisionId }).waitFor({ state: 'visible' });

  const afterApproval = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(afterApproval.approvals[0]?.status === 'ISSUED', 'Missing required FX leaves approval unconsumed.');
  const preparation = await command('trade.execution.preparation.get', {
    workspaceId, approvalId: issued.approvals[0].approvalId,
  });
  check(preparation.preparation === null, `The failed Prepare created no attempt or reservation: ${JSON.stringify(preparation)}`);
  const unchangedProposal = await command('trade.proposal.get', { workspaceId, proposalId: proposal.proposalId });
  check(unchangedProposal.status === 'NEEDS_APPROVAL', 'The proposal remains unchanged after unavailable evidence.');
  const gatewayAfter = await command('live.gateway.fixture.inspect', {});
  const postsAfter = gatewayAfter.requests.filter(request => request.method === 'POST').length;
  check(postsAfter === postsBefore, 'Prepare failed before sending any local fake-Gateway request.');

  const preparationUnavailableWidths = await assertResponsive('RISK_EVIDENCE_UNAVAILABLE',
    'docs/implementation/evidence/s33/preparation-fx-evidence-unavailable-390.png');

  const log = await commandLog(scenarioCursor);
  const commands = log.commands.map(entry => entry.command);
  const providerCommands = commands.filter(name => (name.startsWith('provider.') && name !== 'provider.list_definitions')
    || name.startsWith('binance.')
    || name.startsWith('bitget.') || name.startsWith('trading212.') || name.startsWith('alpaca.'));
  const gatewayCommands = commands.filter(name => name.startsWith('live.gateway.') && name !== 'live.gateway.fixture.inspect');
  const executionCommands = commands.filter(name => name.startsWith('trade.execution.') && !name.endsWith('.preparation.get'));
  check(providerCommands.length === 0, `No provider integration commands were made: ${JSON.stringify(providerCommands)}`);
  check(gatewayCommands.length === 0, `No Order Gateway command was made: ${JSON.stringify(gatewayCommands)}`);
  check(executionCommands.length === 1 && executionCommands[0] === 'trade.execution.prepare',
    `Only the failed Prepare command was attempted: ${JSON.stringify(executionCommands)}`);
  check(commands.filter(name => name === 'paper.order.submit').length === 2,
    'Exactly two deliberately risk-blocked Local Paper submits were attempted.');
  check(externalRequests.length === 0, `Browser made external requests: ${JSON.stringify(externalRequests)}`);
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    `Browser errors occurred: ${JSON.stringify({ consoleErrors, pageErrors })}`);
  return {
    result: 'PASS_LIMITED', screenId: 'F7', workspaceId,
    rejectedProposalId: localProposal.proposalId, rejectedStatus: submittedRejectedDecision.status,
    rejectedSubmitDecisionAppended: true,
    unavailableProposalId: unavailableProposal.proposalId,
    unavailableStatus: unavailableDecision.status, unavailableSubmitError: 'RISK_EVIDENCE_UNAVAILABLE',
    submittedUnavailableDecisionStatus: submittedUnavailableDecision.status,
    preparationProposalId: proposal.proposalId,
    riskDecisionStatus: latestPreparationDecision.status, prepareError: 'RISK_EVIDENCE_UNAVAILABLE',
    preparationRiskDecisionRefreshed: true,
    approval: afterApproval.approvals[0].status, preparation: preparation.preparation,
    proposal: unchangedProposal.status, simulatorOrders: localPaper.orders.length,
    simulatorFills: localPaper.fills.length, gatewayPosts: postsAfter - postsBefore,
    responsiveWidths: { rejected: rejectedWidths, unavailable: unavailableWidths, preparationUnavailable: preparationUnavailableWidths }, providerCommands: providerCommands.length,
    gatewayCommands: gatewayCommands.length, externalHttpRequests: externalRequests.length,
    consoleErrors, pageErrors,
  };
}
