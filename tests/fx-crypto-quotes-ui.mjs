// Actual React -> typed Rust -> disposable storage; only external producers are fake.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

export async function checkFxCryptoQuotesUI(tab, browser) {
  const endpoint = new URL('/__integration/command', await tab.url());
  const send = async (command, payload) => {
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ requestId: randomUUID(), schemaVersion: 1, command, payload }),
    });
    assert.equal(response.status, 200);
    const result = await response.json();
    assert.equal(result.ok, true, `${command}: ${JSON.stringify(result.error)}`);
    return result.data;
  };
  const ui = tab.playwright;
  const inputValue = locator => locator.evaluate(element => element.value);
  const assertJsonEqual = (actual, expected, message) => assert.deepEqual(JSON.parse(JSON.stringify(actual)), expected, message);
  const navigate = async name => {
    const target = ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name, exact: true });
    if (!(await target.isVisible())) await ui.getByText('More', { exact: true }).press('Enter');
    await target.press('Enter');
    await tab.getAXState({ emit: false });
  };
  const bootstrap = await send('workspace.open', {});
  const workspacePath = `${bootstrap.path}-s29-fx-crypto-${randomUUID()}`;
  const workspace = await send('workspace.open', {
    name: 'S29.11 FX crypto quotes', baseCurrency: 'USD', path: workspacePath,
  });
  const workspaceId = workspace.workspaceId;
  const risk = await send('risk.get_policy', { workspaceId });
  await send('risk.save_policy', {
    workspaceId, expectedStateVersion: risk.stateVersion,
    policy: { ...risk.policy, staleQuoteThresholdSeconds: 30 },
  });
  const setupHeading = ui.getByRole('heading', { name: 'Create local workspace', exact: true });
  const workspaceHeading = ui.getByRole('heading', { name: 'Workspace', exact: true }).first();
  await setupHeading.or(workspaceHeading).waitFor({ state: 'visible' });
  if (await ui.getByRole('textbox', { name: 'Workspace name', exact: true }).isVisible()) {
    await ui.getByLabel('Local storage', { exact: true }).fill(workspace.path);
    await ui.getByRole('button', { name: 'Open workspace', exact: true }).press('Enter');
  }
  await workspaceHeading.waitFor({ state: 'visible' });
  await tab.getAXState({ emit: false });
  let accounts = await send('account.list', { workspaceId });
  let ratesAccount = accounts.accounts.find(account => account.providerId === 'alpaca'
    && account.environment === 'PAPER' && account.data?.accountType === 'ALPACA_PAPER');
  if (!ratesAccount) {
    ratesAccount = await send('provider.connect', {
      step: 'test', workspaceId, providerId: 'alpaca', environment: 'PAPER',
      label: `S29.11 FX synthetic ${randomUUID()}`,
    });
  }
  assert.equal(ratesAccount.data?.accountType, 'ALPACA_PAPER', 'The selected account must come from the synthetic Alpaca Paper fixture');
  let spotAccount = accounts.accounts.find(account => account.providerId === 'binance'
    && account.environment === 'LIVE' && account.data?.accountType === 'BINANCE_SPOT');
  if (!spotAccount) {
    spotAccount = await send('provider.connect', {
      step: 'test', workspaceId, providerId: 'binance', environment: 'LIVE',
      label: `S29.11 crypto FX ${randomUUID()}`,
    });
    assert.equal(spotAccount.data?.accountType, 'BINANCE_SPOT', 'The selected account must come from the synthetic Binance fixture');
  }
  const reviewSyntheticAccount = async account => {
    if (account.connectionState !== 'REVIEW_REQUIRED') return account;
    await navigate('Accounts');
    await ui.getByRole('button', { name: account.label, exact: false }).press('Enter');
    await ui.getByRole('button', { name: 'Refresh account', exact: true }).press('Enter');
    let refreshed;
    for (let attempt = 0; attempt < 20; attempt++) {
      await new Promise(resolve => setTimeout(resolve, 50));
      refreshed = await send('account.get', { workspaceId, connectionId: account.connectionId });
      if (refreshed.health.authentication === 'VALID' && refreshed.health.connection === 'ONLINE'
        && refreshed.health.credential === 'CONFIGURED') break;
    }
    assert.equal(refreshed.health.authentication, 'VALID', 'The synthetic credential refresh must succeed before review');
    assert.equal(refreshed.health.connection, 'ONLINE');
    assert.equal(refreshed.health.credential, 'CONFIGURED');
    const permissionReview = ui.getByRole('region', { name: 'Permission review', exact: true });
    const acknowledgement = permissionReview.getByRole('checkbox', {
      name: 'I understand that permission scope is unverified and have checked the key’s permissions at the provider.',
      exact: true,
    });
    if (await acknowledgement.count()) {
      await acknowledgement.setChecked(true);
    }
    const confirm = permissionReview.getByRole('button', { name: 'Confirm connection', exact: true });
    for (let attempt = 0; attempt < 20 && !(await confirm.isEnabled()); attempt++) {
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    assert.ok(await confirm.isEnabled(), 'A fresh synthetic account must allow the explicit permission review');
    await confirm.click();
    let confirmed;
    for (let attempt = 0; attempt < 20; attempt++) {
      await new Promise(resolve => setTimeout(resolve, 50));
      confirmed = await send('account.get', { workspaceId, connectionId: account.connectionId });
      if (confirmed.connectionState === 'CONNECTED') break;
    }
    assert.equal(confirmed.connectionState, 'CONNECTED', 'Only the explicit fixture permission review promotes the synthetic account');
    return confirmed;
  };
  // The isolated integration workspace uses synthetic credentials. Keep its explicit review UI in the test path.
  ratesAccount = await reviewSyntheticAccount(ratesAccount);
  spotAccount = await reviewSyntheticAccount(spotAccount);
  await navigate('Order Drafts');
  await ui.getByRole('button', { name: 'New draft', exact: true }).press('Enter');
  await ui.getByRole('combobox', { name: 'Execution context', exact: true }).selectOption('BINANCE_LIVE');
  await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption(spotAccount.connectionId);
  await ui.getByRole('combobox', { name: 'Instrument', exact: true }).selectOption('crypto:BTC/USDT:spot');
  await ui.getByRole('combobox', { name: 'Side', exact: true }).selectOption('BUY');
  await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
  await ui.getByRole('combobox', { name: 'Quantity type', exact: true }).selectOption('BASE');
  await ui.getByRole('textbox', { name: 'Quantity', exact: true }).fill('0.001');
  await ui.getByRole('textbox', { name: 'Limit price', exact: true }).fill('60000');
  await ui.getByRole('combobox', { name: 'Time in force', exact: true }).selectOption('GTC');
  await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
  await ui.getByRole('status').filter({ hasText: 'Draft saved at version' }).waitFor({ state: 'visible' });
  await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
  await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
  await tab.getAXState({ emit: false });
  const proposals = await send('trade.proposal.list', { workspaceId });
  const proposal = proposals.proposals[0];
  assert.ok(proposal, 'The visible Spot BUY proposal is present in saved proposal history');
  const proposalRow = ui.locator('.order-proposal-row').filter({ hasText: proposal.proposalId.slice(0, 16) });
  const requirements = await send('data.fx.requirements', { workspaceId, proposalId: proposal.proposalId });
  assert.ok(requirements.requirements.some(route => route.purpose === 'INTENT_POLICY'
    && route.fromCurrency === 'USDT' && route.toCurrency === 'USD'
    && route.providerPair === 'USDTUSD' && route.providerDataset === 'CRYPTO_LATEST_QUOTES'));

  await navigate('Settings');
  await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
  await ui.getByRole('heading', { name: 'Alpaca currency rates', exact: true }).waitFor({ state: 'visible' });
  const settings = ui.getByRole('region', { name: 'Alpaca currency rates', exact: true });
  const account = settings.getByRole('combobox', { name: 'Currency rates account', exact: true });
  const accountValue = await account.locator(`option[value="${ratesAccount.connectionId}"]`).getAttribute('value');
  assert.ok(accountValue);
  const location = settings.getByRole('combobox', { name: 'Crypto market-data location', exact: true });
  assert.equal(await inputValue(location), '', 'A crypto location is never selected implicitly');
  await account.selectOption('');
  await account.selectOption(accountValue);
  await location.selectOption('eu-1');
  await settings.getByRole('combobox', { name: 'Alpaca market-data plan', exact: true }).selectOption('BASIC');
  await settings.getByRole('textbox', { name: 'Account jurisdiction', exact: true }).fill('IE');
  const agreementReviewedAt = settings.locator('input[aria-label="Provider agreement reviewed on"]');
  // Native date input fill in CUA sets the DOM value without firing React's
  // change handler; enter the localized date through real keyboard events.
  await agreementReviewedAt.pressSequentially('11102026');
  await agreementReviewedAt.press('Tab');
  await settings.getByRole('combobox', { name: 'Declared use scope', exact: true }).selectOption('OWN_LOCAL_NONCOMMERCIAL');
  await settings.getByRole('combobox', { name: 'Raw market data destination', exact: true }).selectOption('LOCAL_ONLY');
  await settings.getByRole('button', { name: 'Save alpaca currency rates', exact: true }).press('Enter');
  await settings.getByText('Alpaca currency rates selection saved. Read access has not been verified.', { exact: true }).waitFor({ state: 'visible' });
  await tab.getAXState({ emit: false });
  assert.doesNotMatch(await settings.innerText(), /Save this selection before refreshing/, 'A saved source profile is not presented as an unsaved draft after the connection reloads');
  let configuredSource = await send('data.fx.connection', { workspaceId });
  assert.equal(configuredSource.cryptoLocation, 'eu-1');
  assertJsonEqual(configuredSource.sourceUseContext, {
    marketDataPlan: 'BASIC', accountJurisdiction: 'IE', agreementReviewedAt: '2026-10-11',
    useScope: 'OWN_LOCAL_NONCOMMERCIAL', rawDataDestination: 'LOCAL_ONLY',
  });
  await account.selectOption('');
  for (const label of ['Alpaca market-data plan', 'Account jurisdiction', 'Provider agreement reviewed on', 'Country confirmed by Alpaca Support', 'Support confirmation date', 'Support case or message reference', 'Declared use scope', 'Raw market data destination']) {
    assert.equal(await inputValue(settings.locator(`[aria-label="${label}"]`)), '', `Changing accounts clears the unsaved ${label} draft`);
  }
  await account.selectOption(accountValue);
  assert.equal(await inputValue(settings.getByRole('combobox', { name: 'Alpaca market-data plan', exact: true })), 'BASIC');
  assert.equal(await inputValue(settings.getByRole('textbox', { name: 'Account jurisdiction', exact: true })), 'IE');
  assert.doesNotMatch(await settings.innerText(), /Save this selection before refreshing/, 'Reopening the saved account does not mark its restored facts as an unsaved draft');
  assert.equal(configuredSource.sourceUseAssessment.status, 'UNRESOLVED');
  assert.equal(configuredSource.sourceUseAssessment.provenance, 'USER_REPORTED_PROVIDER_UI');
  assert.equal(configuredSource.capabilityStatuses.find(item => item.capability === 'FX_DATA_USE_RIGHTS').status, 'UNVERIFIED');
  const sourceUse = settings.getByRole('region', { name: 'Source-use assessment', exact: true });
  await sourceUse.getByText('Unresolved', { exact: true }).waitFor({ state: 'visible' });
  assert.match(await sourceUse.innerText(), /User-reported provider UI/);
  assert.match(await sourceUse.innerText(), /rights remain unverified/i);
  assert.match(await sourceUse.innerText(), /2dc774d4aeeafbe4c7f0565e7842d932bc8bc10488af805fce43b8734e7b9859/);
  assert.match(await sourceUse.innerText(), /d086d69f063bba1e9693fffbb3185c4469b24b47bda79f587053bd771580fcf6/);
  const jurisdiction = settings.getByRole('textbox', { name: 'Account jurisdiction', exact: true });
  await jurisdiction.fill('');
  assert.equal(await settings.getByRole('button', { name: 'Save alpaca currency rates', exact: true }).isEnabled(), false, 'Partial source-use context cannot be saved');
  assert.match(await settings.innerText(), /Complete all source-use facts or clear them all before saving/);
  await jurisdiction.fill('IE');
  const useScope = settings.getByRole('combobox', { name: 'Declared use scope', exact: true });
  const unresolvedStateVersion = configuredSource.stateVersion;
  await useScope.selectOption('COMMERCIAL_OR_SHARED');
  assert.match(await settings.innerText(), /Save this selection before refreshing/);
  await settings.getByRole('button', { name: 'Save alpaca currency rates', exact: true }).press('Enter');
  let outOfProfile;
  for (let attempt = 0; attempt < 20; attempt++) {
    await new Promise(resolve => setTimeout(resolve, 50));
    outOfProfile = await send('data.fx.connection', { workspaceId });
    if (outOfProfile.stateVersion !== unresolvedStateVersion) break;
  }
  assert.notEqual(outOfProfile.stateVersion, unresolvedStateVersion, 'The changed declared scope is saved as new source material');
  assert.equal(outOfProfile.sourceUseAssessment.status, 'OUT_OF_PROFILE');
  assert.equal(outOfProfile.capabilityStatuses.find(item => item.capability === 'FX_DATA_USE_RIGHTS').status, 'UNVERIFIED');
  const outOfProfileStateVersion = outOfProfile.stateVersion;
  await useScope.selectOption('OWN_LOCAL_NONCOMMERCIAL');
  assert.match(await settings.innerText(), /Save this selection before refreshing/);
  await settings.getByRole('button', { name: 'Save alpaca currency rates', exact: true }).press('Enter');
  for (let attempt = 0; attempt < 20; attempt++) {
    await new Promise(resolve => setTimeout(resolve, 50));
    configuredSource = await send('data.fx.connection', { workspaceId });
    if (configuredSource.stateVersion !== outOfProfileStateVersion) break;
  }
  assert.notEqual(configuredSource.stateVersion, outOfProfileStateVersion, 'The local declared scope is restored as new source material');
  assert.equal(configuredSource.sourceUseAssessment.status, 'UNRESOLVED');
  configuredSource = await send('data.fx.connection', { workspaceId });
  assertJsonEqual(configuredSource.sourceUseContext, {
    marketDataPlan: 'BASIC', accountJurisdiction: 'IE', agreementReviewedAt: '2026-10-11',
    useScope: 'OWN_LOCAL_NONCOMMERCIAL', rawDataDestination: 'LOCAL_ONLY',
  }, 'The saved use facts remain available after reopening the source connection');

  const supportCountry = settings.getByRole('textbox', { name: 'Country confirmed by Alpaca Support', exact: true });
  const supportDate = settings.getByRole('textbox', { name: 'Support confirmation date', exact: true });
  const supportReference = settings.getByRole('textbox', { name: 'Support case or message reference', exact: true });
  const submitSupportFacts = async expectedPreviousVersion => {
    await settings.getByRole('button', { name: 'Save alpaca currency rates', exact: true }).press('Enter');
    let result;
    for (let attempt = 0; attempt < 20; attempt++) {
      await new Promise(resolve => setTimeout(resolve, 50));
      result = await send('data.fx.connection', { workspaceId });
      if (result.stateVersion !== expectedPreviousVersion) break;
    }
    assert.notEqual(result.stateVersion, expectedPreviousVersion, 'Support evidence is saved as new source material');
    return result;
  };
  let beforeSupport = configuredSource.stateVersion;
  // Synthetic user-reported Support facts exercise the policy evaluator only; they are not provider evidence and must not count toward S29.11 acceptance.
  await supportCountry.fill('US');
  await supportDate.pressSequentially('11102026');
  await supportDate.press('Tab');
  await supportReference.fill('SYNTHETIC-SUPPORT-CASE-123');
  let supportAssessment = await submitSupportFacts(beforeSupport);
  assert.equal(supportAssessment.sourceUseAssessment.status, 'UNRESOLVED', 'A country confirmation for another jurisdiction does not qualify policy valuation');
  assert.equal(supportAssessment.sourceUseAssessment.provenance, 'USER_REPORTED_PROVIDER_SUPPORT');
  assertJsonEqual(supportAssessment.sourceUseAssessment.supportedPurposes, []);
  assert.equal(supportAssessment.capabilityStatuses.find(item => item.capability === 'FX_DATA_USE_RIGHTS').status, 'UNVERIFIED');
  assert.match(await settings.getByRole('region', { name: 'Source-use assessment', exact: true }).innerText(), /different jurisdiction/i);

  beforeSupport = supportAssessment.stateVersion;
  await supportCountry.fill('IE');
  supportAssessment = await submitSupportFacts(beforeSupport);
  assert.equal(supportAssessment.sourceUseAssessment.status, 'SUPPORTED_FOR_DECLARED_SCOPE');
  assert.equal(supportAssessment.sourceUseAssessment.provenance, 'USER_REPORTED_PROVIDER_SUPPORT');
  assertJsonEqual(supportAssessment.sourceUseAssessment.supportedPurposes, ['POLICY_VALUATION']);
  assert.equal(supportAssessment.capabilityStatuses.find(item => item.capability === 'FX_DATA_USE_RIGHTS').status, 'UNVERIFIED');
  const supported = settings.getByRole('region', { name: 'Source-use assessment', exact: true });
  await supported.getByText('Supported for declared scope', { exact: true }).waitFor({ state: 'visible' });
  assert.match(await supported.innerText(), /User-reported Alpaca Support confirmation/);
  assert.match(await supported.innerText(), /rights remain unverified/i);
  assert.match(await supported.innerText(), /not provider-verified market-data entitlement/i);
  configuredSource = supportAssessment;
  await send('time.revalidate', { workspaceId });

  await navigate('Order Drafts');
  await proposalRow.press('Enter');
  await tab.getAXState({ emit: false });
  const context = ui.getByRole('region', { name: 'Currency evidence', exact: true });
  assert.match(await context.innerText(), /Selected immutable proposal/);
  await context.getByRole('button', { name: 'Refresh rates for this proposal', exact: true }).press('Enter');
  const quotePanel = context.getByRole('region', { name: 'USDT/USD crypto quote evidence', exact: true });
  await quotePanel.waitFor({ state: 'visible' });
  await tab.getAXState({ emit: false });

  const current = await send('data.fx.connection', { workspaceId });
  assert.equal(current.status, 'AVAILABLE');
  const quote = current.evidence.cryptoQuotes.find(item => item.providerSymbol === 'USDT/USD');
  assert.ok(quote);
  assert.equal(current.evidence.cryptoLocation, 'eu-1');
  assert.equal(quote.bidPrice, '0.9999');
  assert.equal(quote.bidSize, '12.345678901');
  assert.equal(quote.askPrice, '1.0001');
  assert.equal(quote.askSize, '23.456789012');
  assert.match(quote.requestPath, /^\/v1beta3\/crypto\/eu-1\/latest\/quotes\?symbols=USDT%2FUSD$/);
  assert.match(quote.responseSha256, /^[a-f0-9]{64}$/);
  assert.equal(current.capabilityStatuses.find(item => item.capability === 'FX_DATA_USE_RIGHTS').status, 'UNVERIFIED');
  const quoteText = await quotePanel.innerText();
  for (const text of [
    'USDT/USD · eu-1', '0.9999 · 12.345678901', '1.0001 · 23.456789012',
    quote.requestPath, quote.responseSha256, 'Price-only policy valuation input',
  ]) assert.ok(quoteText.includes(text), `Missing quote evidence text: ${text}`);
  assert.match(await context.innerText(), /fx data use rights\s+unverified/i);

  const viewport = await browser.capabilities.get('viewport');
  const layouts = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await quotePanel.evaluate(element => ({
        viewport: window.innerWidth,
        body: document.documentElement.scrollWidth,
        panel: element.scrollWidth,
        client: element.clientWidth,
      }));
      assert.equal(size.viewport, width);
      assert.ok(size.body <= width && size.panel <= size.client + 1, JSON.stringify(size));
      layouts.push(size);
    }
  } finally { await viewport.reset(); }

  // Bind the owning approval snapshot to current, read-only Spot inputs. The approval API only
  // carries the proposal's fee/FX statement when capacity has been observed for that Proposal.
  const savedRules = await send('data.binance_rules.connection', { workspaceId });
  await send('data.binance_rules.configure', {
    workspaceId, connectionId: spotAccount.connectionId,
    instrumentId: 'crypto:BTC/USDT:spot', expectedStateVersion: savedRules.stateVersion,
  });
  await send('binance.live.capacity.fixture', { scenario: 'COMPLETE_COVERAGE' });
  await send('time.revalidate', { workspaceId });
  const rules = await send('data.binance_rules.connection', { workspaceId });
  const refreshedRules = await send('data.binance_rules.refresh', { workspaceId, expectedStateVersion: rules.stateVersion });
  assert.equal(refreshedRules.status, 'AVAILABLE');
  await navigate('Order Drafts');
  await proposalRow.press('Enter');
  await tab.getAXState({ emit: false });
  const capacity = ui.getByRole('region', { name: 'Current Proposal Spot capacity inputs', exact: true });
  const intervals = ui.getByRole('region', { name: 'Current Proposal order interval inputs', exact: true });
  for (const panel of [capacity, intervals]) {
    await panel.getByText(refreshedRules.evidence.materialVersion, { exact: false }).waitFor({ state: 'visible' });
  }
  for (const [panel, button] of [
    [capacity, 'Refresh required capacity inputs'],
    [intervals, 'Refresh account interval inputs'],
  ]) {
    await panel.getByText('not observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
    await panel.getByRole('button', { name: button, exact: true }).press('Enter');
    await panel.getByText('observed · Read-only inputs · Execution qualification unavailable', { exact: true }).waitFor({ state: 'visible' });
  }

  // Read FX last so the proposal-scoped quote is current when the approval snapshot is derived.
  const previousReceipt = quote.receivedAt;
  await context.getByRole('button', { name: 'Refresh rates for this proposal', exact: true }).press('Enter');
  let currentForReview, finalQuote;
  for (let attempt = 0; attempt < 30; attempt++) {
    await new Promise(resolve => setTimeout(resolve, 50));
    currentForReview = await send('data.fx.connection', { workspaceId });
    finalQuote = currentForReview.evidence.cryptoQuotes.find(item => item.providerSymbol === 'USDT/USD');
    if (currentForReview.status === 'AVAILABLE' && finalQuote?.receivedAt !== previousReceipt) break;
  }
  assert.equal(currentForReview.status, 'AVAILABLE');
  assert.ok(finalQuote);
  assert.notEqual(finalQuote.receivedAt, previousReceipt, 'The final approval check binds a newly read Proposal quote');

  const review = await send('trade.request_approval', { workspaceId, proposalId: proposal.proposalId });
  assert.ok(review.spotFeeFx, 'The owning FX statement is captured only after current Spot capacity is read');
  assert.equal(review.eligible, false);
  const policyRoute = review.spotFeeFx.routes.find(route => route.purpose === 'INTENT_POLICY');
  assert.equal(policyRoute?.providerPair, 'USDTUSD');
  assert.equal(policyRoute?.state, 'QUALIFIED', 'A fresh quote in the policy band and matching current source-use assessment can qualify price-only valuation');
  assert.equal(policyRoute?.providerQuality, 'UNQUALIFIED_FX_RATE', 'Provider rate quality stays distinct from purpose qualification');
  assert.equal(policyRoute?.conservativeCost ?? null, null, 'Price-only valuation does not invent execution costs');
  assert.equal(currentForReview.capabilityStatuses.find(item => item.capability === 'FX_DATA_USE_RIGHTS')?.status, 'UNVERIFIED', 'User-reported Support facts do not verify FX market-data rights');
  assert.equal(review.riskDecision.checks.some(check => check.checkId === 'CURRENCY_CONVERSION'), false, 'A matching-USD raw quote does not create a transaction conversion leg');
  assert.equal(review.spotFeeFx.baseCurrency, 'USD');
  assert.ok(review.spotFeeFx.routes.every(route => route.fromCurrency !== 'EUR' && route.toCurrency !== 'EUR'), 'A matching-USD raw quote adds no EUR route or value');
  assert.equal((await send('account.get', { workspaceId, connectionId: spotAccount.connectionId })).health.arming, 'DISARMED');

  // The public pre-Arm review must preserve the same proposal-scoped raw quote
  // in its immutable captured evidence, not only in the current source panel.
  await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  const preArm = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  await preArm.waitFor({ state: 'visible' });
  await tab.getAXState({ emit: false });
  const capturedEvidence = preArm.getByRole('region', { name: 'Captured currency evidence', exact: true });
  const capturedQuotePanel = capturedEvidence.getByRole('region', { name: 'USDT/USD crypto quote evidence', exact: true });
  await capturedQuotePanel.waitFor({ state: 'visible' });
  const capturedQuoteText = await capturedQuotePanel.innerText();
  for (const value of [
    'USDT/USD · eu-1', finalQuote.bidPrice, finalQuote.bidSize,
    finalQuote.askPrice, finalQuote.askSize, finalQuote.providerTimestamp,
    finalQuote.receivedAt, finalQuote.requestPath, finalQuote.responseSha256,
  ]) assert.ok(capturedQuoteText.includes(value), `Captured pre-Arm UI omitted raw quote evidence: ${value}`);
  assert.equal(await preArm.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
  const capturedLayouts = [];
  try {
    for (const width of [1280, 768, 390]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const size = await preArm.evaluate(element => {
        const rect = element.getBoundingClientRect();
        return { viewport: window.innerWidth, body: document.documentElement.scrollWidth, left: rect.left, right: rect.right };
      });
      assert.equal(size.viewport, width);
      assert.ok(size.body <= width && size.left >= 0 && size.right <= width, JSON.stringify(size));
      capturedLayouts.push(size);
    }
  } finally { await viewport.reset(); }
  await preArm.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  await preArm.waitFor({ state: 'hidden' });
  await tab.getAXState({ emit: false });
  return {
    quote: finalQuote,
    layouts,
    capturedLayouts,
    capturedPreArmQuote: true,
    route: {
      state: policyRoute.state,
      providerQuality: policyRoute.providerQuality,
      conservativeExecutionCost: policyRoute.conservativeCost ?? null,
    },
    sourceUse: {
      status: currentForReview.sourceUseAssessment.status,
      provenance: currentForReview.sourceUseAssessment.provenance,
      supportedPurposes: currentForReview.sourceUseAssessment.supportedPurposes,
    },
    rights: 'UNVERIFIED',
    transactionFxQualification: review.riskDecision.checks.find(check => check.checkId === 'CURRENCY_CONVERSION')?.outcome,
    approvalEligible: review.eligible,
    arming: 'DISARMED',
    workspacePath,
  };
}
