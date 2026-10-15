// Run with `playwright-cli run-code --filename=tests/s33-live-approval-focus-playwright.js`
// against the Rust-backed `npm run dev:browser` integration harness.
// Uses a disposable workspace and test-only synthetic Live account; no provider is contacted.
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
    const requestId = `s33-approval-focus-${name}-${Date.now()}-${Math.random()}`;
    const result = await page.evaluate(async envelope => {
      const response = await fetch('/__integration/command', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
      });
      return { status: response.status, body: await response.json() };
    }, { requestId, schemaVersion: 1, command: name, payload });
    check(result.status === 200 && result.body.ok,
      `${name} failed: ${JSON.stringify(result.body.error)}`);
    return result.body.data;
  };
  const commandLog = async cursor => page.evaluate(async after =>
    (await fetch(`/__integration/command-log?after=${after}`)).json(), cursor);

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto('http://127.0.0.1:1420/');
  const bootstrap = await command('workspace.open', {});
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const workspaceName = `S33 Approval Focus ${stamp}`;
  const workspacePath = `${bootstrap.path.slice(0, bootstrap.path.lastIndexOf('/'))}/s33-approval-focus-${stamp}`;
  const workspace = await command('workspace.open', { name: workspaceName, baseCurrency: 'USD', path: workspacePath });
  await command('workspace.ready.fixture', { workspaceId: workspace.workspaceId });
  await page.getByRole('textbox', { name: 'Workspace name', exact: true }).fill(workspaceName);
  await page.getByRole('combobox', { name: 'Base currency', exact: true }).selectOption('USD');
  await page.getByLabel('Local storage', { exact: true }).fill(workspacePath);
  await page.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Ready', exact: true }).waitFor({ state: 'visible' });

  const workspaceId = workspace.workspaceId;
  const risk = await command('risk.get_policy', { workspaceId });
  await command('risk.save_policy', {
    workspaceId,
    expectedStateVersion: risk.stateVersion,
    policy: {
      ...risk.policy,
      marketOrdersEnabled: true,
      maxMarketOrderSlippagePercent: '5',
      staleQuoteThresholdSeconds: 86400,
      maxSingleInstrumentExposurePercent: null,
    },
  });
  const account = await command('account.arming.fixture.seed', {
    workspaceId, providerId: 'trading212', label: `S33 synthetic approval ${stamp}`,
  });
  await command('time.revalidate', { workspaceId });
  await command('account.arm', {
    workspaceId, connectionId: account.connectionId,
    expectedStateVersion: account.stateVersion, confirmed: true,
  });

  await page.getByRole('navigation', { name: 'Primary navigation', exact: true })
    .getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await page.getByRole('heading', { name: 'Order Drafts', exact: true }).waitFor({ state: 'visible' });
  await page.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('TRADING212_LIVE');
  await page.getByRole('combobox', { name: 'Account', exact: true }).selectOption(account.connectionId);
  await page.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('equity:US:AAPL');
  await page.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await page.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await page.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await page.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.01');
  await page.getByRole('textbox', { name: 'Limit price', exact: true }).fill('50000');
  await page.getByRole('textbox', { name: 'Maximum spend (optional)', exact: true }).fill('510');
  await page.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await page.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await page.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  const proposalList = await command('trade.proposal.list', { workspaceId });
  check(proposalList.proposals.length === 1, 'The disposable workspace has one synthetic Live proposal.');
  const expectedInstrumentId = 'equity:US:AAPL';
  const proposal = proposalList.proposals[0];
  const immutableProposal = await command('trade.proposal.get', { workspaceId, proposalId: proposal.proposalId });
  check(typeof immutableProposal.estimatedNotional === 'string', 'The F2 limit-order fixture has a backend estimated notional.');
  await page.locator('.order-proposal-row').first().press('Enter');

  const reviewButton = page.getByRole('button', { name: 'Review Live approval', exact: true });
  await reviewButton.waitFor({ state: 'visible' });
  const beforeReview = await commandLog(0);
  await reviewButton.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Review Live approval', exact: true });
  await dialog.waitFor({ state: 'visible' });
  const keepReviewing = dialog.getByRole('button', { name: 'Keep reviewing', exact: true });
  const reject = dialog.getByRole('button', { name: 'Reject this review', exact: true });
  const approve = dialog.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true });
  const approveEnabled = await approve.isEnabled();
  let enterOnApproveBlocked = false;
  const reviewText = await dialog.innerText();
  for (const expected of [
    'Approval alone does not reserve capacity or place an order.',
    'TRADING212_LIVE', account.label, account.data.remoteAccountId, account.connectionId,
    proposal.proposalId, immutableProposal.proposalHash,
    expectedInstrumentId, immutableProposal.fields.venue,
    'BUY', '0.01 BASE', 'LIMIT · 50000 · GTC',
    `Estimated notional\n${immutableProposal.estimatedNotional} ${immutableProposal.estimatedNotionalCurrency ?? ''}`.trim(),
    'Policy / RiskDecision', 'Backend capacity preview',
    'Provider available', 'TradeX reserved', 'Effective available',
    'Quote provenance', 'Snapshot / source', 'Provider timestamp', 'Received / venue',
    'Entitlement / freshness', 'Quote age', 'Risk checks',
    'Reject this review', 'Approve for up to 30 seconds', 'Keep reviewing',
  ]) check(reviewText.includes(expected), `The F2 review shows ${expected}: ${reviewText}`);
  check(reviewText.includes('Current checks pass. Approval still requires your explicit action.')
    || reviewText.includes('Approval blocked:'),
  'The current risk decision is visible without granting approval.');
  const policyDecision = await dialog.locator('.approval-review-fields').first().evaluate(list => {
    const row = [...list.querySelectorAll(':scope > div')].find(item =>
      item.querySelector('dt')?.innerText.trim() === 'Policy / RiskDecision');
    return row?.querySelector('dd')?.innerText.trim() ?? '';
  });
  check(policyDecision.startsWith(`v${immutableProposal.policyVersion} · `),
    `The review shows the proposal's exact policy version with its risk decision: ${policyDecision}`);
  const quoteRows = await dialog.locator('section[aria-labelledby="approval-quote-title"] dl').evaluate(list =>
    Object.fromEntries([...list.querySelectorAll(':scope > div')].map(row => [
      row.querySelector('dt')?.innerText.trim(), row.querySelector('dd')?.innerText.trim(),
    ])));
  for (const label of ['Snapshot / source', 'Provider timestamp', 'Received / venue', 'Entitlement / freshness', 'Quote age']) {
    check(quoteRows[label] && !quoteRows[label].includes('Unavailable'),
      `The F2 synthetic MarketSnapshot has populated ${label}: ${JSON.stringify(quoteRows)}`);
  }
  check(/^\d+ ms$/.test(quoteRows['Quote age']), `Quote age is shown as a measured value: ${JSON.stringify(quoteRows)}`);
  const capacityRows = await dialog.locator('.live-capacity-summary dl').evaluate(list =>
    Object.fromEntries([...list.querySelectorAll(':scope > div')].map(row => [
      row.querySelector('dt')?.innerText.trim(), row.querySelector('dd')?.innerText.trim(),
    ])));
  for (const label of ['Provider available', 'TradeX reserved', 'Effective available']) {
    check(capacityRows[label] && !capacityRows[label].startsWith('Unavailable'),
      `The F2 synthetic capacity projection has ${label}: ${JSON.stringify(capacityRows)}`);
  }
  const riskChecks = await dialog.locator('.approval-review-checks li').allTextContents();
  check(riskChecks.length > 0 && riskChecks.every(text => text.includes(' · ') && text.includes(': ')),
    `The F2 deterministic risk checks include outcomes and reasons: ${JSON.stringify(riskChecks)}`);
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Keep reviewing',
    'The neutral dismissal action receives initial focus.');
  check(await page.locator('.app-shell').evaluate(element => element.inert),
    'The application shell is inert while the approval dialog is open.');

  const focusable = dialog.locator('button:not(:disabled)');
  const focusableCount = await focusable.count();
  check(focusableCount >= 2, 'The dialog exposes its safe dismissal and review actions.');
  const expectedFocusOrder = ['Reject this review', ...(approveEnabled ? ['Approve for up to 30 seconds'] : []), 'Keep reviewing'];
  check(focusableCount === expectedFocusOrder.length, 'Enabled dialog actions follow the expected visual reading order.');
  const assertFocus = async (expected, message) =>
    check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === expected, message);
  let focusedIndex = expectedFocusOrder.indexOf('Keep reviewing');
  for (const expected of [...expectedFocusOrder.slice(focusedIndex + 1), ...expectedFocusOrder.slice(0, focusedIndex + 1)]) {
    await page.keyboard.press('Tab');
    focusedIndex = (focusedIndex + 1) % expectedFocusOrder.length;
    await assertFocus(expected, `Tab reaches ${expected} in visual order and remains trapped.`);
  }
  for (let step = 0; step < expectedFocusOrder.length; step += 1) {
    await page.keyboard.press('Shift+Tab');
    focusedIndex = (focusedIndex - 1 + expectedFocusOrder.length) % expectedFocusOrder.length;
    await assertFocus(expectedFocusOrder[focusedIndex], `Shift+Tab reaches ${expectedFocusOrder[focusedIndex]} in reverse visual order.`);
  }
  if (approveEnabled) {
    await approve.focus();
    await page.keyboard.press('Enter');
    await dialog.waitFor({ state: 'visible' });
    const approvalsAfterApproveEnter = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
    check(approvalsAfterApproveEnter.approvals.length === 0,
      'Enter on the enabled Approve control does not issue a financial approval.');
    enterOnApproveBlocked = true;
  }

  const responsive = [];
  let identityLayout;
  for (const width of [1280, 768, 390]) {
    await page.setViewportSize({ width, height: 844 });
    if (width === 390) {
      await dialog.evaluate(element => { element.scrollTop = 0; });
      await page.locator('.picker-backdrop').evaluate(element => { element.scrollTop = 0; });
      identityLayout = await dialog.locator('.approval-review-fields').first().evaluate(section => {
        const rows = [...section.querySelectorAll(':scope > div')];
        const value = label => rows.find(row => row.querySelector('dt')?.textContent?.trim() === label)
          ?.querySelector('dd')?.textContent?.trim() ?? null;
        const measurements = ['TradeX connection ID', 'Instrument / venue'].map(label => {
          const row = rows.find(candidate => candidate.querySelector('dt')?.textContent?.trim() === label);
          const element = row?.querySelector('dd');
          if (!element) return { label, missing: true };
          const style = getComputedStyle(element);
          return {
            label,
            text: element.textContent?.trim() ?? '',
            clientWidth: element.clientWidth,
            scrollWidth: element.scrollWidth,
            textOverflow: style.textOverflow,
            whiteSpace: style.whiteSpace,
          };
        });
        return { connectionId: value('TradeX connection ID'), instrument: value('Instrument / venue'), measurements };
      });
      check(identityLayout.connectionId === account.connectionId,
        `Approval review exposes the full exact TradeX connection ID: ${JSON.stringify(identityLayout)}`);
      check(identityLayout.instrument?.startsWith(expectedInstrumentId),
        `Approval review exposes the full canonical instrument ID: ${JSON.stringify(identityLayout)}`);
      check(identityLayout.measurements.every(identity => !identity.missing
        && identity.scrollWidth <= identity.clientWidth + 1 && identity.textOverflow !== 'ellipsis'
        && identity.whiteSpace !== 'nowrap'),
        `Account and instrument identities wrap without clipping at 390px: ${JSON.stringify(identityLayout)}`);
      await page.screenshot({ path: 'docs/implementation/evidence/s33/live-approval-identity-390.png' });
    }
    for (const button of [reject, approve, keepReviewing]) await button.scrollIntoViewIfNeeded();
    await page.locator('.picker-backdrop').evaluate(backdrop => { backdrop.scrollTop = backdrop.scrollHeight; });
    const geometry = await page.evaluate(() => {
      const dialog = document.querySelector('.approval-review-dialog');
      const actions = [...(dialog?.querySelectorAll('.approval-review-actions button') ?? [])]
        .map(button => { const box = button.getBoundingClientRect(); return { left: box.left, right: box.right, top: box.top, bottom: box.bottom }; });
      return {
        viewport: document.documentElement.clientWidth,
        documentScroll: document.documentElement.scrollWidth,
        dialogWidth: dialog?.clientWidth,
        dialogScroll: dialog?.scrollWidth,
        actions,
        height: window.innerHeight,
      };
    });
    check(geometry.documentScroll <= geometry.viewport && geometry.dialogScroll <= geometry.dialogWidth,
      `The approval dialog has no horizontal overflow at ${width}px: ${JSON.stringify(geometry)}`);
    const followsVisualOrder = geometry.actions.every((box, index, all) => index === 0
      || box.top > all[index - 1].top
      || (box.top === all[index - 1].top && box.left >= all[index - 1].left));
    check(followsVisualOrder, `Keyboard action order matches the rendered action order at ${width}px.`);
    check(geometry.actions.every(box => box.left >= 0 && box.right <= width && box.top >= 0 && box.bottom <= geometry.height + 1),
      `Approval decisions remain reachable inside the viewport at ${width}px: ${JSON.stringify(geometry)}`);
    if (width === 390) await page.screenshot({ path: 'docs/implementation/evidence/s33/live-approval-focus-390.png' });
    responsive.push(width);
  }

  await page.setViewportSize({ width: 1280, height: 900 });
  await keepReviewing.press('Enter');
  await dialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Review Live approval',
    'Enter on the neutral action restores focus to its review trigger.');
  check(!(await page.locator('.app-shell').evaluate(element => element.inert)),
    'Dismissing the dialog restores the application shell.');
  const approvalsAfterEnter = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(approvalsAfterEnter.approvals.length === 0, 'Enter on Keep reviewing creates no approval or rejection.');

  await reviewButton.press('Enter');
  await dialog.waitFor({ state: 'visible' });
  await page.keyboard.press('Escape');
  await dialog.waitFor({ state: 'hidden' });
  check(await page.evaluate(() => document.activeElement?.textContent?.trim()) === 'Review Live approval',
    'Escape restores focus to the review trigger.');
  const approvalsAfterEscape = await command('trade.approval.list', { workspaceId, proposalId: proposal.proposalId });
  check(approvalsAfterEscape.approvals.length === 0, 'Escape dismisses without approving or rejecting.');
  const afterReview = await commandLog(beforeReview.cursor);
  const decisions = afterReview.commands.filter(entry =>
    /^(trade\.(approve|reject|execution\.)|provider\.(connect|probe|permissions|disconnect)|account\.refresh|live\.gateway\.)/.test(entry.command));
  check(decisions.length === 0,
    `Dialog review made no provider, approval, Gateway, or execution mutation: ${JSON.stringify(decisions)}`);
  check(externalRequests.length === 0, `The browser contacted external hosts: ${JSON.stringify(externalRequests)}`);

  return {
    result: 'PASS',
    boundary: 'disposable workspace and synthetic Trading 212-labelled Live account; no provider endpoint or credentials',
    initialFocus: 'Keep reviewing',
    approveEnabled,
    inertBackground: true,
    keyboardTrap: `Tab and Shift+Tab traverse visual order and wrap between ${focusableCount} enabled dialog controls`,
    fullIdentityLengths: [identityLayout?.connectionId?.length ?? 0, expectedInstrumentId.length],
    identityWrapsWithoutClippingAt390: true,
    enterAndEscape: 'dismiss without approval/rejection and restore trigger focus',
    enterOnApproveBlocked,
    approvals: approvalsAfterEscape.approvals.length,
    forbiddenCommands: decisions.length,
    externalRequests: externalRequests.length,
    responsiveWidths: responsive,
  };
}
