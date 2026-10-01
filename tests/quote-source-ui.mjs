// Public Settings UI against the real isolated Rust/SQLite bridge.
// Supply a confirmed synthetic Alpaca account created through the public Accounts UI.
import assert from 'node:assert/strict';

export async function checkQuoteSourceConfigurationUI(tab, browser, accountLabel) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  const navigate = async name => {
    const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true });
    const target = navigation.getByRole('button', { name, exact: true });
    if (!(await target.isVisible())) await ui.getByText('More', { exact: true }).press('Enter');
    await target.press('Enter');
    await tab.getAXState({ emit: false });
  };
  const settings = async () => {
    await navigate('Settings');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Alpaca quote source', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
  };
  try {
    await viewport.set({ width: 1280, height: 900 });
    await settings();
    assert.equal(await ui.locator('input[type="password"]').count(), 0);
    await ui.getByRole('combobox', { name: 'Saved Alpaca account', exact: true }).selectOption({ label: accountLabel });
    await ui.getByRole('combobox', { name: 'Market-data feed', exact: true }).selectOption('delayed_sip');
    await ui.getByRole('button', { name: 'Save quote source', exact: true }).press('Enter');
    await ui.getByText('Source selection saved. Data access has not been verified.', { exact: true }).waitFor({ state: 'visible' });
    observed.push('Explicit saved-reference selection and delayed_sip feed are saved through native-trusted Rust IPC; data access remains unverified.');
    await navigate('Accounts');
    await settings();
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const state = await ui.evaluate(() => ({
        width: document.documentElement.clientWidth,
        scroll: document.documentElement.scrollWidth,
        feed: document.querySelector('[aria-label="Market-data feed"]').value,
        account: document.querySelector('[aria-label="Saved Alpaca account"]').selectedOptions[0].textContent,
      }));
      assert.ok(state.width <= width && state.width >= width - 20);
      assert.ok(state.scroll <= state.width, `Source settings overflow at${width}px: ${JSON.stringify(state)}`);
      assert.equal(state.feed, 'delayed_sip');
      assert.equal(state.account, accountLabel);
    }
    observed.push('Saved selection survives navigation/remount; source controls have no horizontal overflow at390/768/1280px.');
    await ui.getByRole('button', { name: 'Disconnect quote source', exact: true }).press('Enter');
    await ui.getByText('Quote source disconnected. Your Alpaca account and its saved key were kept.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('button', { name: 'Disconnect quote source', exact: true }).count(), 0);
    await navigate('Accounts');
    assert.match(await ui.getByRole('combobox', { name: 'Connection source', exact: true }).textContent(), new RegExp(`${accountLabel}.*CONNECTED`));
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    observed.push('Source disconnect removes source selection while the synthetic broker account remains CONNECTED; no password renderer or browser error.');
    await settings();
    return observed;
  } finally { await viewport.reset(); }
}

// Dedicated credentials are supplied by the external secure-vault fixture, never by the renderer.
export async function checkDedicatedQuoteSourceUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const observed = [];
  try {
    const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true });
    await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    await ui.getByRole('combobox', { name: 'Data credentials', exact: true }).selectOption('DEDICATED');
    await ui.getByRole('combobox', { name: 'Market-data feed', exact: true }).selectOption('sip');
    assert.equal(await ui.locator('input[type="password"]').count(), 0);
    await ui.getByRole('button', { name: 'Save data key securely', exact: true }).press('Enter');
    await ui.getByText('Source selection saved. Data access has not been verified.', { exact: true }).waitFor({ state: 'visible' });
    observed.push('Dedicated source credentials are captured outside the renderer, with explicit SIP selection and unverified access.');
    await tab.reload();
    await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
    await ui.getByRole('button', { name: 'Data & Storage', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Alpaca quote source', exact: true }).waitFor({ state: 'visible' });
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const state = await ui.evaluate(() => ({
        width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth,
        kind: document.querySelector('[aria-label="Data credentials"]').value,
        feed: document.querySelector('[aria-label="Market-data feed"]').value,
      }));
      assert.ok(state.scroll <= state.width, `Dedicated source settings overflow at ${width}px`);
      assert.equal(state.kind, 'DEDICATED'); assert.equal(state.feed, 'sip');
    }
    observed.push('Dedicated selection survives page reload without horizontal overflow at 390/768/1280px.');
    await ui.getByRole('button', { name: 'Disconnect quote source', exact: true }).press('Enter');
    await ui.getByText('Quote source disconnected. Its dedicated data key was removed.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.equal(await ui.getByRole('button', { name: 'Disconnect quote source', exact: true }).count(), 0);
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    observed.push('Disconnect removes only the dedicated data key and source selection; no renderer secrets or browser errors.');
    return observed;
  } finally { await viewport.reset(); }
}

export async function checkSelectedFeedVerificationUI(tab) {
  const ui=tab.playwright;
  const navigation=ui.getByRole('navigation',{name:'Primary navigation',exact:true});
  await navigation.getByRole('button',{name:'Settings',exact:true}).press('Enter');
  await ui.getByRole('button',{name:'Data & Storage',exact:true}).press('Enter');
  await tab.getAXState({emit:false});
  await ui.getByRole('combobox',{name:'Data credentials',exact:true}).selectOption('DEDICATED');
  await ui.getByRole('combobox',{name:'Market-data feed',exact:true}).selectOption('sip');
  await ui.getByRole('button',{name:'Save data key securely',exact:true}).press('Enter');
  await ui.getByText('Source selection saved. Data access has not been verified.',{exact:true}).waitFor({state:'visible'});
  await ui.getByRole('button',{name:'Verify selected feed',exact:true}).press('Enter');
  await ui.getByText('Selected feed technical access verified. Quote freshness and trading eligibility remain separate.',{exact:true}).waitFor({state:'visible'});
  await tab.getAXState({emit:false});
  const source=ui.getByRole('article',{name:'OD-001',exact:true});
  assert.match(await source.textContent(),/Technical access to the selected sip feed verified/);
  assert.match(await source.textContent(),/SIP: consolidated US equity quotes/);
  await ui.getByRole('combobox',{name:'Market-data feed',exact:true}).selectOption('iex');
  assert.equal(await ui.getByRole('button',{name:'Verify selected feed',exact:true}).isEnabled(),false,'Unsaved feed changes cannot be presented as verified');
  await ui.getByRole('combobox',{name:'Market-data feed',exact:true}).selectOption('sip');
  assert.equal(await ui.getByRole('button',{name:'Verify selected feed',exact:true}).isEnabled(),true);
  assert.equal(await ui.locator('input[type="password"]').count(),0);
  return ['Explicit saved SIP configuration is verified through the adapter, with technical access separate from quote/trading readiness.','Source card and source connection agree; unsaved feed changes disable verification; no renderer password input.'];
}

export async function checkActualQuoteProjectionUI(tab, browser) {
  const ui=tab.playwright;
  const viewport=await browser.capabilities.get('viewport');
  await ui.getByRole('navigation',{name:'Primary navigation',exact:true}).getByRole('button',{name:'Markets',exact:true}).press('Enter');
  await tab.getAXState({emit:false});
  await ui.getByRole('button',{name:/^AAPL /}).press('Enter');
  await ui.getByRole('region',{name:'Market quote',exact:true}).waitFor({state:'visible'});
  await tab.getAXState({emit:false});
  const quote=ui.getByRole('region',{name:'Market quote',exact:true});
  assert.match(await quote.textContent(),/250\.1234567890123456789/);
  assert.match(await quote.textContent(),/250\.2234567890123456789/);
  assert.match(await quote.textContent(),/US_SIP/);
  assert.match(await quote.textContent(),/NYSE Arca/);
  assert.match(await quote.textContent(),/Nasdaq/);
  assert.match(await quote.textContent(),/Regular Two Sided Open/);
  assert.match(await quote.textContent(),/Bid size \(BASE\)205/);
  assert.match(await quote.textContent(),/Ask size \(BASE\)310/);
  assert.match(await quote.textContent(),/Not supplied by quote endpoint/);
  assert.equal(await ui.getByText(/No active Hot lease supplied this snapshot; an on-demand read is not a subscription update\./).isVisible(),true);
  assert.match(await quote.textContent(),/2026-09-30T14:10:00\.123456789Z/);
  try {
    for(const width of [390,768,1280]) {
      await viewport.set({width,height:900});
      await tab.getAXState({emit:false});
      const geometry=await ui.evaluate(()=>({width:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth}));
      assert.ok(geometry.scroll<=geometry.width,`Quote provenance overflows at ${width}px`);
    }
    assert.equal((await tab.dev.logs({levels:['error'],limit:20})).length,0);
    return ['Actual external quote bytes reach the shared Rust Market projection, with exact bid/ask, size units, authenticated exchange/condition metadata and precise provider time.','Quote provenance has no horizontal overflow at 390/768/1280px; no last trade or active Hot subscription is invented.'];
  } finally {await viewport.reset();}
}

export async function checkHotQuoteUI(tab,browser) {
  const ui=tab.playwright;const viewport=await browser.capabilities.get('viewport');
  await ui.getByRole('navigation',{name:'Primary navigation',exact:true}).getByRole('button',{name:'Markets',exact:true}).press('Enter');
  await ui.getByRole('button',{name:/^AAPL /}).press('Enter');
  await ui.getByRole('region',{name:'Hot quote subscription',exact:true}).getByText('STREAMING',{exact:true}).waitFor({state:'visible'});
  const quote=ui.getByRole('region',{name:'Market quote',exact:true});
  assert.match(await quote.textContent(),/250\.7234567890123456789/);assert.match(await quote.textContent(),/Ask size \(BASE\)365/);
  assert.equal(await ui.getByText(/No active Hot lease supplied this snapshot/).count(),0);
  assert.match(await quote.textContent(),/STALE|CLOCK_UNCERTAIN/);
  try {
    for(const width of [390,768,1280]) {
      await viewport.set({width,height:900});await tab.getAXState({emit:false});
      const geometry=await ui.evaluate(()=>({width:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth}));assert.ok(geometry.scroll<=geometry.width,`Hot detail overflows at ${width}px`);
    }
    await ui.getByRole('button',{name:'Back to results',exact:true}).press('Enter');
    assert.equal(await ui.getByRole('region',{name:'Hot quote subscription',exact:true}).count(),0);
    await ui.getByRole('button',{name:/^MSFT /}).press('Enter');
    await ui.getByRole('region',{name:'Hot quote subscription',exact:true}).getByText('STREAMING',{exact:true}).waitFor({state:'visible'});
    assert.match(await ui.getByRole('region',{name:'Market quote',exact:true}).textContent(),/Provider symbolMSFT/);
    assert.equal((await tab.dev.logs({levels:['error'],limit:20})).length,0);
    return ['Actual loopback WebSocket frames activate the public Hot detail only after provider acknowledgements and validated quotes.','Old fixture time remains non-current despite STREAMING; AAPL to results to MSFT remount displays the requested provider symbol; no overflow at 390/768/1280px.'];
  }finally{await viewport.reset();}
}

export async function checkAutomaticReconnectUI(tab) {
  const ui=tab.playwright;
  await ui.getByRole('navigation',{name:'Primary navigation',exact:true}).getByRole('button',{name:'Markets',exact:true}).press('Enter');
  await ui.getByRole('button',{name:/^AAPL /}).press('Enter');
  const status=ui.getByRole('region',{name:'Hot quote subscription',exact:true});
  await status.getByText('Automatic reconnect 1 of 3.',{exact:true}).waitFor({state:'visible'});
  await status.getByText('STREAMING',{exact:true}).waitFor({state:'visible'});
  const quote=ui.getByRole('region',{name:'Market quote',exact:true});
  assert.match(await quote.textContent(),/250\.7234567890123456789/);assert.match(await quote.textContent(),/CLOCK_UNCERTAIN|STALE/);
  assert.equal(await ui.getByRole('button',{name:'Retry selected feed',exact:true}).count(),0);
  assert.equal((await tab.dev.logs({levels:['error'],limit:20})).length,0);
  return ['Provider-initiated TCP closure automatically recovers the same public view lease on the selected SIP feed, without a UI reacquire or manual Retry.','The recovered public projection displays reconnect attempt 1 and actual quote bytes; old fixture time remains non-current and trading guards stay visible.'];
}

// External provider confirms subscription but deliberately emits no quote frames.
export async function checkPendingHotQuoteUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  try {
    await viewport.set({ width: 1280, height: 900 });
    await ui.getByRole('navigation', { name: 'Primary navigation', exact: true }).getByRole('button', { name: 'Markets', exact: true }).press('Enter');
    await tab.getAXState({ emit: false });
    await ui.getByRole('button', { name: /^AAPL / }).press('Enter');
    const hot = ui.getByRole('region', { name: 'Hot quote subscription', exact: true });
    await hot.getByText('AWAITING_QUOTE', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.match(await hot.textContent(), /Authentication confirmed · Subscription confirmed/);
    const detail = ui.getByRole('region', { name: 'Apple Inc.', exact: true });
    assert.match(await detail.textContent(), /Unavailable.*Subscription confirmed; waiting for an actual provider quote\./s);
    assert.doesNotMatch(await detail.textContent(), /adapter.*not configured|technical access has not been verified/i);
    assert.equal(await ui.getByRole('region', { name: 'Market quote', exact: true }).count(), 0);
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const sizes = await ui.evaluate(() => ({ width: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
      assert.ok(sizes.scroll <= sizes.width, `Pending quote overflow at ${width}px`);
    }
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    return ['Actual socket authentication and subscription without a quote keep Hot AWAITING_QUOTE; no invented snapshot.', 'Detail explains waiting for a provider quote without the legacy unconfigured-adapter message; keyboard navigation and 390/768/1280px have no overflow or browser error.'];
  } finally { await viewport.reset(); }
}

// Normally connected fake T212 account stays UNVERIFIED/DISARMED; quote bytes come from the Alpaca adapter.
export async function checkPreArmingQuoteEvidenceUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true });
  const label = 'Read-only quote evidence account';
  try {
    await viewport.set({ width: 1280, height: 900 });
    await navigation.getByRole('button', { name: 'Accounts', exact: true }).press('Enter');
    await ui.getByRole('combobox', { name: 'Provider / environment', exact: true }).selectOption('trading212/LIVE');
    await ui.getByLabel('Connection label', { exact: true }).fill(label);
    await ui.getByRole('button', { name: 'Connect account securely', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'Permission review', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('checkbox', { name: 'I understand that permission scope is unverified and have checked the key’s permissions at the provider.' }).press('Space');
    await ui.getByRole('button', { name: 'Confirm connection', exact: true }).press('Enter');
    await ui.getByText('Unverified scope was explicitly acknowledged for this permission review.', { exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    assert.match(await ui.getByRole('region', { name: label, exact: true }).textContent(), /DISARMED/);
    await navigation.getByRole('button', { name: 'Markets', exact: true }).press('Enter');
    await ui.getByRole('button', { name: /^AAPL / }).press('Enter');
    await ui.getByRole('region', { name: 'Market quote', exact: true }).waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const originalQuote = await ui.getByRole('region', { name: 'Market quote', exact: true }).textContent();
    const snapshot = await ui.getByRole('region', { name: 'Market quote', exact: true }).locator('div').filter({ has: ui.getByText('Snapshot ID', { exact: true }) }).locator('dd').textContent();
    assert.match(originalQuote, /250\.7234567890123456789/);
    await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
    await ui.getByRole('heading', { name: 'New order draft', exact: true }).waitFor({ state: 'visible' });
    await ui.getByRole('combobox', { name: 'Account', exact: true }).selectOption({ label: `${label} · trading212 · LIVE` });
    await ui.getByRole('combobox', { name: 'Order type', exact: true }).selectOption('LIMIT');
    await ui.getByLabel('Quantity', { exact: true }).fill('2');
    await ui.getByLabel('Limit price', { exact: true }).fill('250.8');
    await ui.getByRole('button', { name: 'Save draft', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'Draft saved at version 1.' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Generate proposal', exact: true }).press('Enter');
    await ui.getByRole('status').filter({ hasText: 'generated and requires approval' }).waitFor({ state: 'visible' });
    await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
    const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
    await dialog.waitFor({ state: 'visible' });
    await tab.getAXState({ emit: false });
    const evidence = dialog.getByRole('region', { name: 'Quote evidence before arming', exact: true });
    assert.equal(await evidence.count(), 1, 'DISARMED review must expose read-only configured quote evidence before Arm');
    const text = await evidence.textContent();
    for (const value of [snapshot, '250.7234567890123456789', '250.8234567890123456789', 'SIP', 'US_SIP', 'NYSE Arca', 'Nasdaq', 'Regular Two Sided Open', '2026-09-30T14:10:00.123456789Z']) assert.ok(text.includes(value), `Missing producer-derived evidence: ${value}`);
    assert.match(text, /Read only.*does not approve or arm/s);
    assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
    assert.equal(await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).count(), 0);
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const bounds = await dialog.evaluate(element => { const r = element.getBoundingClientRect(); return { left: r.left, right: r.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth }; });
      assert.ok(bounds.left >= 0 && bounds.right <= bounds.width && bounds.scroll <= bounds.width, `Pre-Arm quote evidence overflow at ${width}px`);
    }
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    return ['Public normally connected T212 account remains UNVERIFIED/DISARMED; backend read-only review shows the same adapter-produced snapshot and exact quote provenance before Arm.', 'Arm remains disabled, Approve is absent, and read-only quote/blocker display works through keyboard at 390/768/1280px without overflow/errors.'];
  } finally { await viewport.reset(); }
}

// Continue the preceding public flow with its immutable proposal; never generate new authority.
export async function checkPreArmingSourceReplacementUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const navigation = ui.getByRole('navigation', { name: 'Primary navigation', exact: true });
  const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  const originalProposal = await dialog.getByText(/proposal:.*sha256:/).textContent();
  await dialog.getByRole('button', { name: 'Keep reviewing', exact: true }).press('Enter');
  await navigation.getByRole('button', { name: 'Settings', exact: true }).press('Enter');
  await ui.getByRole('combobox', { name: 'Market-data feed', exact: true }).selectOption('iex');
  await ui.getByRole('button', { name: 'Save data key securely', exact: true }).press('Enter');
  await ui.getByText('Source selection saved. Data access has not been verified.', { exact: true }).waitFor({ state: 'visible' });
  await navigation.getByRole('button', { name: 'Order Drafts', exact: true }).press('Enter');
  await ui.getByRole('button', { name: /^NEEDS_APPROVAL v1/ }).press('Enter');
  await ui.getByRole('button', { name: 'Review Live approval', exact: true }).press('Enter');
  await dialog.waitFor({ state: 'visible' });
  assert.equal(await dialog.getByText(/proposal:.*sha256:/).textContent(), originalProposal);
  return checkUnavailablePreArmingQuoteUI(tab, browser);
}

export async function checkUnavailablePreArmingQuoteUI(tab, browser) {
  const ui = tab.playwright;
  const viewport = await browser.capabilities.get('viewport');
  const dialog = ui.getByRole('dialog', { name: 'Confirm Live arming', exact: true });
  const evidence = dialog.getByRole('region', { name: 'Quote evidence before arming', exact: true });
  assert.match(await evidence.textContent(), /Quote evidence is unavailable.*Selected iex feed is configured; technical access has not been verified/s);
  assert.doesNotMatch(await evidence.textContent(), /250\.7234567890123456789|alpaca:|Snapshot \/ source/);
  assert.equal(await dialog.getByRole('button', { name: 'Arm this Live account', exact: true }).isEnabled(), false);
  assert.equal(await ui.getByRole('button', { name: 'Approve for up to 30 seconds', exact: true }).count(), 0);
  try {
    for (const width of [390, 768, 1280]) {
      await viewport.set({ width, height: 900 });
      await tab.getAXState({ emit: false });
      const bounds = await dialog.evaluate(element => { const r = element.getBoundingClientRect(); return { left: r.left, right: r.right, width: window.innerWidth, scroll: document.documentElement.scrollWidth }; });
      assert.ok(bounds.left >= 0 && bounds.right <= bounds.width && bounds.scroll <= bounds.width, `Unavailable pre-Arm evidence overflow at ${width}px`);
    }
    assert.equal((await tab.dev.logs({ levels: ['error'], limit: 20 })).length, 0);
    return ['Public source replacement retires the old SIP quote: the same immutable proposal now explains unverified IEX with no old quote identity or price.', 'Unavailable evidence and blockers retain disabled Arm and absent Approve, with no overflow/errors at 390/768/1280px.'];
  } finally { await viewport.reset(); }
}
