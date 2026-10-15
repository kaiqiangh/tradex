// Run against the isolated Rust-backed browser harness with
// TRADEX_LIVE_APPROVAL_FIXTURE=1 and TRADEX_S26_2_CANCEL_FIXTURE=1.
// Trading 212 account, order, and Gateway responses are synthetic local fixtures.
async page => {
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  let workspaceId;
  let account;
  let brokerOrderId;
  const externalRequests = [];
  const consoleErrors = [];
  const pageErrors = [];
  page.setDefaultTimeout(15000);
  page.on('request', request => {
    const url = new URL(request.url());
    if (['http:', 'https:'].includes(url.protocol) && !['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.origin);
    }
  });
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', error => pageErrors.push(error.message));

  const command = async (name, payload) => {
    const requestId = 's33-qa01-' + name + '-' + Date.now() + '-' + Math.random();
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200, name + ' bridge status ' + result.status);
    check(result.body.requestId === requestId, name + ' request ID mismatch');
    check(result.body.ok, name + ' failed: ' + JSON.stringify(result.body.error));
    return result.body.data;
  };
  const commandLog = async after => page.evaluate(async cursor =>
    (await fetch('/__integration/command-log?after=' + cursor)).json(), after);
  const cancellationHistory = () => command('trade.cancel_approval.list', {
    workspaceId,
    accountId: account.connectionId,
    brokerOrderId,
  });
  const readIntent = text => ({
    id: text.match(/CANCEL intent ID\s+([^\n]+)/)?.[1],
    hash: text.match(/Intent hash\s+([^\n]+)/)?.[1],
  });
  const waitForProviderCooldown = async () => {
    await page.waitForTimeout(5200);
    const time = await command('time.revalidate', { workspaceId });
    check(time.confidence === 'TRUSTED', 'Trusted synthetic time remains current after the Trading 212 read cooldown.');
  };
  const openAccountsAndSelect = async () => {
    await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
      .getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    const accountRow = page.locator('.account-row').filter({ hasText: account.label });
    await accountRow.waitFor({ state: 'visible' });
    await accountRow.click();
    const orderRow = page.locator('.table-scroll[aria-label="Open orders"] tbody tr')
      .filter({ hasText: brokerOrderId });
    await orderRow.waitFor({ state: 'visible' });
    return { accountRow, orderRow, reviewButton: orderRow.getByRole('button', { name: 'Review cancellation', exact: true }) };
  };

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1425/');
  const bootstrap = await command('workspace.open', {});
  const stamp = Date.now() + '-' + Math.random().toString(36).slice(2);
  const workspaceName = 'S33 QA-01 CANCEL arming ' + stamp;
  const workspacePath = bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/')) + '/s33-qa01-cancel-' + stamp;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'EUR', path: workspacePath });
  workspaceId = workspace.workspaceId;
  const risk = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: risk.stateVersion,
    policy: { ...risk.policy, maxSingleInstrumentExposurePercent: '10.25' },
  });
  await command('workspace.ready.fixture', { workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('EUR');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });
  await page.reload();
  await page.getByRole('complementary', { name: 'Workspace', exact: true }).waitFor({ state: 'visible' });

  account = await command('account.cancellation.fixture.seed', {
    workspaceId,
    providerId: 'trading212',
    label: 'S26.2 T212 Live S33 QA-01 ' + stamp,
  });
  brokerOrderId = '9007199254740996';
  await command('trading212.live.cancel.fixture.seed_order', {});
  await command('account.open_order.fixture.seed', {
    workspaceId,
    connectionId: account.connectionId,
    brokerOrderId,
  });
  await command('time.revalidate', { workspaceId });

  // A fill between the first review and explicit arming invalidates the same intent.
  let selection = await openAccountsAndSelect();
  await selection.reviewButton.press('Enter');
  let dialog = page.getByRole('dialog', { name: 'Review Live cancellation', exact: true });
  await dialog.waitFor({ state: 'visible' });
  let firstReview = await dialog.innerText();
  let firstIntent = readIntent(firstReview);
  check(/^cancel:/.test(firstIntent.id || ''), 'The first immutable CANCEL intent is present.');
  check(/^sha256:[0-9a-f]{64}$/.test(firstIntent.hash || ''), 'The first intent has a canonical semantic hash.');
  for (const value of [account.label, account.connectionId, brokerOrderId, 'MSFT_US_EQ', 'PARTIALLY_FILLED', '0.25', 'Exact remaining quantity\n0.75', 'Account arming\nDISARMED']) {
    check(firstReview.includes(value), 'The first review binds ' + value + ': ' + firstReview);
  }
  check(await dialog.getByRole('button', { name: 'Approve cancellation', exact: true }).isDisabled(),
    'A disarmed account cannot approve the initial cancellation review.');
  const fillBeforeApprovalCursor = (await commandLog(0)).cursor;
  await waitForProviderCooldown();
  const accountBeforeFillArm = await command('account.get', {
    workspaceId,
    connectionId: account.connectionId,
  });
  await command('trading212.live.cancel.fixture.fill_race', {});
  await dialog.getByRole('checkbox').check();
  await dialog.getByRole('button', { name: 'Arm account and refresh this intent', exact: true }).press('Enter');
  await page.waitForFunction(() => {
    const dialog = document.querySelector('dialog.live-cancel-dialog');
    const row = [...document.querySelectorAll('.table-scroll[aria-label="Open orders"] tbody tr')]
      .find(candidate => candidate.innerText.includes('9007199254740996'));
    return Boolean(dialog?.querySelector('[role="alert"]')) || !row;
  }, null, { timeout: 20000 });
  const fillDialogCount = await page.locator('dialog.live-cancel-dialog').count();
  const fillBeforeApprovalReview = fillDialogCount ? await dialog.innerText() : '';
  if (fillDialogCount) {
    check(fillBeforeApprovalReview.includes(firstIntent.id) && fillBeforeApprovalReview.includes(firstIntent.hash),
      'The failed refresh preserves the original intent in the open dialog.');
    check(await dialog.getByRole('button', { name: 'Approve cancellation', exact: true }).isDisabled(),
      'The terminal order cannot be approved after the post-arm refresh detects its fill.');
    check(await dialog.getByRole('alert').count() === 1,
      'The UI presents the terminal-order refresh failure inside the cancellation review.');
  } else {
    check(await selection.orderRow.count() === 0,
      'Once the exact read confirms a terminal fill, the open-order row and its stale approval control disappear.');
  }
  const filledAccount = await command('account.get', {
    workspaceId,
    connectionId: account.connectionId,
  });
  const fillBranchLog = await commandLog(fillBeforeApprovalCursor);
  const fillBranchCommands = fillBranchLog.commands.map(entry => entry.command);
  check(filledAccount.health.arming === 'ARMED',
    'Explicit Arm succeeds before the exact order refresh observes the fill: '
    + JSON.stringify({
      health: filledAccount.health,
      state: filledAccount.connectionState,
      dialog: fillBeforeApprovalReview || 'order row removed after terminal provider observation',
      beforeArm: { health: accountBeforeFillArm.health, state: accountBeforeFillArm.connectionState },
      commands: fillBranchLog.commands,
    }));
  check(!filledAccount.data.openOrders.some(order => order.brokerOrderId === brokerOrderId),
    'The provider-confirmed terminal order is removed from the open-order snapshot.');
  let history = await cancellationHistory();
  const invalidatedIntent = history.intents.find(intent => intent.cancellationIntentId === firstIntent.id);
  check(invalidatedIntent?.status === 'INVALIDATED'
    && invalidatedIntent.invalidationReason === 'ORDER_NOT_CANCELLABLE',
  'The filled-before-approval branch invalidates the exact original intent.');
  check(history.approvals.length === 0 && history.rejections.length === 0,
    'The terminal-order branch records no approval or rejection.');
  const preApprovalHistory = history;
  check(fillBranchCommands.filter(name => name === 'account.arm').length === 1,
    'The fill branch contains one explicit account Arm.');
  check(!fillBranchCommands.includes('trade.cancel_approve')
    && !fillBranchCommands.includes('trade.cancel_reject')
    && !fillBranchCommands.includes('trade.execution.prepare'),
  'The fill-before-approval branch creates no authorization or preparation.');
  check((await command('live.gateway.fixture.inspect', {})).requests.length === 0,
    'The fill-before-approval branch makes no Gateway request.');
  if (fillDialogCount) {
    await page.setViewportSize({ width: 390, height: 844 });
    await dialog.screenshot({ path: 'docs/implementation/evidence/s33/qa01-fill-before-approval-390.png' });
    await page.setViewportSize({ width: 1280, height: 900 });
    await dialog.getByRole('button', { name: 'Close without a decision', exact: true }).press('Enter');
    await dialog.waitFor({ state: 'hidden' });
  }

  // Reset only the disposable fixture, then prove the complete Arm-to-CANCEL path.
  await command('account.disarm', {
    workspaceId,
    connectionId: account.connectionId,
    expectedStateVersion: filledAccount.stateVersion,
  });
  await command('trading212.live.cancel.fixture.seed_order', {});
  await command('account.open_order.fixture.seed', {
    workspaceId,
    connectionId: account.connectionId,
    brokerOrderId,
  });
  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  selection = await openAccountsAndSelect();
  await waitForProviderCooldown();
  await selection.reviewButton.press('Enter');
  dialog = page.getByRole('dialog', { name: 'Review Live cancellation', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const disarmedReview = await dialog.innerText();
  const armedPathIntent = readIntent(disarmedReview);
  check(/^cancel:/.test(armedPathIntent.id || '')
    && /^sha256:[0-9a-f]{64}$/.test(armedPathIntent.hash || ''),
  'The reset synthetic order receives a fresh immutable cancellation intent.');
  check(disarmedReview.includes(brokerOrderId)
    && disarmedReview.includes('Exact remaining quantity\n0.75')
    && disarmedReview.includes('Account arming\nDISARMED'),
  'The reset review binds the exact provider order and remaining quantity while disarmed.');
  await dialog.getByRole('checkbox').check();
  await waitForProviderCooldown();
  await dialog.getByRole('button', { name: 'Arm account and refresh this intent', exact: true }).press('Enter');
  try {
    await page.waitForFunction(() => {
      const dialog = document.querySelector('dialog.live-cancel-dialog');
      const button = dialog?.querySelector('button.primary');
      return Boolean(dialog && button && button.textContent.trim() === 'Approve cancellation'
        && dialog.innerText.includes('Risk decision / policy\nALLOWED')
        && dialog.innerText.includes('Account arming\nARMED'));
    }, null, { timeout: 10000 });
  } catch (error) {
    const current = await command('account.get', { workspaceId, connectionId: account.connectionId });
    const log = await commandLog(0);
    const dialogCount = await page.locator('dialog.live-cancel-dialog').count();
    throw new Error('Positive Arm and refresh did not settle: ' + JSON.stringify({
      failure: String(error),
      account: { state: current.connectionState, health: current.health, stateVersion: current.stateVersion },
      dialog: dialogCount ? await page.locator('dialog.live-cancel-dialog').innerText() : null,
      commands: log.commands.slice(-16),
    }));
  }
  const armedReview = await dialog.innerText();
  check(armedReview.includes(armedPathIntent.id) && armedReview.includes(armedPathIntent.hash),
    'Explicit Arm returns to the same cancellation intent and semantic hash.');
  for (const value of [brokerOrderId, 'Exact remaining quantity\n0.75', 'Account arming\nARMED', 'Risk decision / policy\nALLOWED']) {
    check(armedReview.includes(value), 'The refreshed approval review shows ' + value + ': ' + armedReview);
  }
  check(await dialog.getByRole('button', { name: 'Approve cancellation', exact: true }).isEnabled(),
    'Approval is available only after the same intent has fresh order evidence and an ALLOWED risk decision.');

  // Rejection is an explicit, durable decision and still has no Gateway side effect.
  const rejectionCursor = (await commandLog(0)).cursor;
  await dialog.getByRole('button', { name: 'Reject cancellation', exact: true }).press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  await page.getByRole('status').filter({ hasText: 'Cancellation review rejected and recorded.' }).waitFor({ state: 'visible' });
  history = await cancellationHistory();
  check(history.approvals.length === 0 && history.rejections.length === 1
    && history.rejections[0].cancellationIntentId === armedPathIntent.id
    && history.rejections[0].intentHash === armedPathIntent.hash,
  'Explicit rejection is recorded against the exact armed intent without creating an approval.');
  const rejectionLog = await commandLog(rejectionCursor);
  const rejectionCommands = rejectionLog.commands.map(entry => entry.command);
  check(rejectionCommands.filter(name => name === 'trade.cancel_reject').length === 1
    && !rejectionCommands.includes('trade.execution.prepare'),
  'The rejection path records one reject and no preparation.');
  check((await command('live.gateway.fixture.inspect', {})).requests.length === 0,
    'Rejection makes no Gateway request.');

  // A new explicit review after rejection receives a new intent and can be approved.
  await waitForProviderCooldown();
  await selection.reviewButton.press('Enter');
  dialog = page.getByRole('dialog', { name: 'Review Live cancellation', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const postRejectReview = await dialog.innerText();
  const approvedIntent = readIntent(postRejectReview);
  check(approvedIntent.id !== armedPathIntent.id && approvedIntent.hash === armedPathIntent.hash,
    'A new review creates a new intent ID while preserving unchanged provider-order terms.');
  check(postRejectReview.includes(brokerOrderId)
    && postRejectReview.includes('Exact remaining quantity\n0.75')
    && postRejectReview.includes('Account arming\nARMED')
    && postRejectReview.includes('Risk decision / policy\nALLOWED'),
  'The fresh approval review still binds the exact armed order and remaining quantity.');
  await dialog.getByRole('button', { name: 'Approve cancellation', exact: true }).press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  const approvalCard = page.getByRole('article', { name: 'Saved Trading 212 Live cancellation attempt', exact: true });
  await approvalCard.waitFor({ state: 'visible' });
  const approvalCardText = await approvalCard.innerText();
  check(approvalCardText.includes(approvedIntent.id)
    && approvalCardText.includes(approvedIntent.hash)
    && approvalCardText.includes(brokerOrderId)
    && approvalCardText.includes('remaining quantity 0.75')
    && approvalCardText.includes(account.connectionId),
  'The account-level saved history shows the short-lived CANCEL approval bound to the exact account, intent, provider order and remaining quantity.');
  history = await cancellationHistory();
  const issuedApproval = history.approvals.find(approval => approval.cancellationIntentId === approvedIntent.id);
  check(issuedApproval?.operation === 'CANCEL' && issuedApproval.status === 'ISSUED'
    && issuedApproval.brokerOrderId === brokerOrderId
    && issuedApproval.intentHash === approvedIntent.hash
    && issuedApproval.remainingQuantity === '0.75',
  'The durable approval is operation CANCEL and preserves exact immutable order terms.');
  check(await page.getByRole('button', { name: 'Prepare and send approved PLACE', exact: true }).count() === 0,
    'The cancellation path exposes no PLACE preparation action.');

  // The Gateway's required T212 account-summary read shares the five-second broker read budget
  // with the exact order refresh that issued this approval.
  await waitForProviderCooldown();
  await command('live.gateway.fixture.set_result', { scenario: 'ACCEPTED' });
  const prepareCursor = (await commandLog(0)).cursor;
  await page.getByRole('button', { name: 'Prepare and send approved cancellation', exact: true }).press('Enter');
  const preparationCard = page.getByRole('article', { name: 'Saved Trading 212 Live cancellation attempt', exact: true });
  await page.waitForFunction(() => [...document.querySelectorAll('article[aria-label="Saved Trading 212 Live cancellation attempt"]')]
    .some(article => article.innerText.includes('CANCEL_PENDING')),
  null, { timeout: 20000 });
  await page.setViewportSize({ width: 390, height: 844 });
  await preparationCard.scrollIntoViewIfNeeded();
  const preparedCardBounds = await preparationCard.boundingBox();
  const preparedPageGeometry = await page.evaluate(() => ({
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: window.innerWidth,
  }));
  check(preparedCardBounds && preparedCardBounds.x >= 0 && preparedCardBounds.x + preparedCardBounds.width <= 390,
    'The prepared cancellation state fits the 390 px viewport: ' + JSON.stringify({ preparedCardBounds, preparedPageGeometry }));
  check(preparedPageGeometry.documentWidth <= preparedPageGeometry.viewportWidth,
    'The prepared cancellation page has no document-level horizontal overflow: ' + JSON.stringify(preparedPageGeometry));
  await preparationCard.screenshot({ path: 'docs/implementation/evidence/s33/qa01-cancel-pending-390.png' });
  await page.setViewportSize({ width: 1280, height: 900 });
  const preparationText = await preparationCard.innerText();
  const savedPreparation = await command('trade.execution.preparation.get', {
    workspaceId,
    approvalId: issuedApproval.approvalId,
  });
  const attempt = savedPreparation.preparation?.attempt;
  const gatewayAfterCancel = await command('live.gateway.fixture.inspect', {});
  const accountAtPrepare = await command('account.get', { workspaceId, connectionId: account.connectionId });
  const prepareLog = await commandLog(prepareCursor);
  check(preparationText.includes('CANCEL_PENDING')
    && preparationText.includes(brokerOrderId)
    && preparationText.includes(approvedIntent.id)
    && preparationText.includes('No new PLACE capacity reservation was created.'),
  'The separate preparation sends only the exact CANCEL intent and records CANCEL_PENDING: '
    + JSON.stringify({
      preparationText,
      attempt,
      account: { stateVersion: accountAtPrepare.stateVersion, health: accountAtPrepare.health },
      approval: issuedApproval,
      commands: prepareLog.commands.slice(-16),
      gatewayRequests: gatewayAfterCancel.requests,
    }));
  check(attempt?.state === 'CANCEL_PENDING' && attempt.operation === 'CANCEL'
    && attempt.brokerOrderId === brokerOrderId
    && attempt.intentId === approvedIntent.id
    && attempt.intentHash === approvedIntent.hash
    && attempt.proposalId == null && attempt.reservationId == null
    && savedPreparation.preparation.reservation == null,
  'The persisted attempt is CANCEL_PENDING with no PLACE proposal or capacity reservation.');
  const preparationCommands = prepareLog.commands.map(entry => entry.command);
  check(preparationCommands.filter(name => name === 'trade.execution.prepare').length === 1,
    'The explicit cancellation preparation is sent once.');
  const deleteRequests = gatewayAfterCancel.requests.filter(request =>
    request.method === 'DELETE' && request.path === '/api/v0/equity/orders/' + brokerOrderId);
  check(deleteRequests.length === 1,
    'The local fake Gateway records exactly one DELETE for the reviewed provider order.');

  // Provider acknowledgement stays pending until exact provider evidence confirms the fill.
  await command('trading212.live.cancel.fixture.fill_race', {});
  await waitForProviderCooldown();
  const refreshButton = page.getByRole('button', { name: 'Refresh exact order', exact: true });
  await refreshButton.waitFor({ state: 'visible' });
  await refreshButton.press('Enter');
  await page.waitForFunction(() => [...document.querySelectorAll('article[aria-label="Saved Trading 212 Live cancellation attempt"]')]
    .some(article => article.innerText.includes('Latest exact order: FILLED · TERMINAL')
      && article.innerText.includes('remaining quantity 0')),
  null, { timeout: 20000 });
  const savedAfterFill = await command('trade.execution.preparation.get', {
    workspaceId,
    approvalId: issuedApproval.approvalId,
  });
  const finalAttempt = savedAfterFill.preparation?.attempt;
  const exactObservation = finalAttempt?.trading212LiveOrderObservation;
  check(finalAttempt?.state === 'CANCEL_PENDING'
    && exactObservation?.providerStatus === 'FILLED'
    && exactObservation.disposition === 'TERMINAL'
    && exactObservation.remainingQuantity === '0',
  'The saved attempt remains CANCEL_PENDING while a later exact provider read records FILLED · TERMINAL and zero remaining.');
  const finalGateway = await command('live.gateway.fixture.inspect', {});
  check(finalGateway.requests.filter(request =>
    request.method === 'DELETE' && request.path === '/api/v0/equity/orders/' + brokerOrderId).length === 1,
  'The post-dispatch fill observation performs no second cancellation write.');
  history = await cancellationHistory();
  const consumedApproval = history.approvals.find(approval => approval.approvalId === issuedApproval.approvalId);
  check(consumedApproval?.status === 'CONSUMED',
    'The exact approved CANCEL authorization is consumed once and remains in history.');
  const finalLog = await commandLog(0);
  const allCommands = finalLog.commands.map(entry => entry.command);
  check(!allCommands.includes('trade.approve') && !allCommands.includes('trade.reject'),
    'No PLACE approval or rejection path was used.');
  check(externalRequests.length === 0, 'The browser issued no external HTTP requests: ' + JSON.stringify(externalRequests));
  check(consoleErrors.length === 0 && pageErrors.length === 0,
    'No browser errors occurred: ' + JSON.stringify({ consoleErrors, pageErrors }));

  return {
    result: 'PASS',
    scope: 'synthetic local Trading 212 fixture and local fake Order Gateway only',
    workspaceId,
    accountId: account.connectionId,
    providerOrderId: brokerOrderId,
    preApprovalFill: {
      intentId: firstIntent.id,
      invalidationReason: invalidatedIntent.invalidationReason,
      approvals: preApprovalHistory.approvals.length,
      rejections: preApprovalHistory.rejections.length,
      terminalOrderRemoved: true,
      gatewayRequests: 0,
    },
    armAndReject: {
      sameIntentIdAfterArm: true,
      sameIntentHashAfterArm: true,
      rejectedIntentId: armedPathIntent.id,
      persistedRejections: history.rejections.length,
    },
    approvedCancellation: {
      intentId: approvedIntent.id,
      intentHash: approvedIntent.hash,
      exactRemainingQuantity: issuedApproval.remainingQuantity,
      preparationState: attempt.state,
      operation: attempt.operation,
      proposalId: attempt.proposalId ?? null,
      reservationId: attempt.reservationId ?? null,
      deleteRequests: deleteRequests.length,
      mobileScreenshot: 'docs/implementation/evidence/s33/qa01-cancel-pending-390.png',
    },
    providerConfirmedFill: {
      attemptState: finalAttempt.state,
      providerStatus: exactObservation.providerStatus,
      disposition: exactObservation.disposition,
      remainingQuantity: exactObservation.remainingQuantity,
      deleteRequestsAfterRefresh: finalGateway.requests.filter(request =>
        request.method === 'DELETE' && request.path === '/api/v0/equity/orders/' + brokerOrderId).length,
    },
    externalRequests: externalRequests.length,
    consoleErrors: consoleErrors.length,
    pageErrors: pageErrors.length,
  };
}
