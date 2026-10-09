// Actual React/Rust/temp SQLite; only external provider responses and vault are fake.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
export async function checkSpotProposalRulesUI(tab, browser) {
  const ui=tab.playwright, endpoint=new URL('/__integration/command',await tab.url());
  const send=async(command,payload)=>{
    const response=await fetch(endpoint,{method:'POST',headers:{'Content-Type':'application/json',Origin:endpoint.origin},body:JSON.stringify({requestId:randomUUID(),schemaVersion:1,command,payload})});
    assert.equal(response.status,200); const result=await response.json(); assert.equal(result.ok,true,`${command}: ${JSON.stringify(result.error)}`); return result.data;
  };
  const workspace=await send('workspace.open',{}), workspaceId=workspace.workspaceId;
  if(await ui.getByRole('heading',{name:'Create local workspace',exact:true}).isVisible()){
    await ui.getByLabel('Local storage',{exact:true}).fill(workspace.path);
    await ui.getByRole('button',{name:'Open workspace',exact:true}).press('Enter');
    await ui.getByRole('heading',{name:'Workspace',exact:true}).waitFor({state:'visible'});
  }
  const navigate=async name=>{
    const target=ui.getByRole('navigation',{name:'Primary navigation',exact:true}).getByRole('button',{name,exact:true});
    if(!await target.isVisible()) await ui.getByText('More',{exact:true}).press('Enter');
    await target.press('Enter'); await tab.getAXState({emit:false});
  };
  let account=(await send('account.list',{workspaceId})).accounts.find(row=>row.label.startsWith('S29.4 external ') && row.connectionState === 'CONNECTED');
  if(!account){
    const tested=await send('provider.connect',{step:'test',workspaceId,providerId:'binance',environment:'LIVE',label:`S29.4 external ${randomUUID()}`});
    account=await send('provider.connect',{step:'confirm',workspaceId,connectionId:tested.connectionId,expectedStateVersion:tested.stateVersion,acknowledgeUnverified:false});
  }
  assert.equal(account.health.arming,'DISARMED');
  await send('time.revalidate',{workspaceId});
  // A persisted policy pins the documented freshness window at its 30-second ceiling so the
  // public reference stays current for the whole interaction; the default 3-second window and
  // expiry itself are covered deterministically by the Rust staleness test.
  const unconfigured=await send('risk.get_policy',{workspaceId});
  if(unconfigured.configured!==true){
    await send('risk.save_policy',{workspaceId,expectedStateVersion:unconfigured.stateVersion,policy:{
      maxOrderNotional:null,maxOrderQuantity:null,maxPositionSize:null,
      maxSingleInstrumentExposurePercent:'10.25',maxAssetClassExposurePercent:[],
      maxDailyTradedNotional:null,maxDailyRealizedLoss:null,maxOpenOrders:null,maxReservedCapital:null,
      allowedInstrumentIds:[],blockedInstrumentIds:[],allowedVenues:[],blockedVenues:[],
      allowedAccountIds:[],blockedAccountIds:[],allowedEnvironments:[],
      staleQuoteThresholdSeconds:30,marketOrdersEnabled:false,maxMarketOrderSlippagePercent:null,
      maxPriceDeviationPercent:null,liveInactivityTimeoutMinutes:20}});
  }
  const saved=await send('data.binance_rules.connection',{workspaceId});
  await send('data.binance_rules.configure',{workspaceId,connectionId:account.connectionId,instrumentId:'crypto:BTC/USDT:spot',expectedStateVersion:saved.stateVersion});
  await send('binance.live.rules.fixture',{scenario:'PERCENT_REFERENCE',symbol:'BTCUSDT'});
  const source=await send('data.binance_rules.connection',{workspaceId});
  assert.equal((await send('data.binance_rules.refresh',{workspaceId,expectedStateVersion:source.stateVersion})).status,'AVAILABLE');
  await navigate('Order Drafts');
  await ui.getByRole('button',{name:'New draft',exact:true}).press('Enter');
  await ui.getByRole('combobox',{name:'Execution context',exact:true}).selectOption('BINANCE_LIVE');
  await ui.getByRole('combobox',{name:'Account',exact:true}).selectOption(account.connectionId);
  await ui.getByRole('combobox',{name:'Instrument',exact:true}).selectOption('crypto:BTC/USDT:spot');
  await ui.getByRole('textbox',{name:'Quantity',exact:true}).fill('0.001');
  await ui.getByRole('textbox',{name:'Limit price',exact:true}).fill('60000');
  await ui.getByRole('combobox',{name:'Time in force',exact:true}).selectOption('GTC');
  await ui.getByRole('button',{name:'Save draft',exact:true}).press('Enter');
  await ui.getByRole('status').filter({hasText:'Draft saved at version 1.'}).waitFor({state:'visible'});
  await ui.getByRole('button',{name:'Generate proposal',exact:true}).press('Enter');
  await ui.getByRole('status').filter({hasText:'generated and requires approval'}).waitFor({state:'visible'});
  await tab.getAXState({emit:false});
  const current=ui.getByRole('region',{name:'Current Proposal Spot rules',exact:true});
  assert.equal(await current.count(),1,'Selected immutable Proposal must display per-rule qualification');
  await current.getByText('PERCENT_PRICE · UNAVAILABLE',{exact:true}).waitFor({state:'visible'});
  const range=ui.getByRole('region',{name:'Current price range execution preview',exact:true});
  const clickRefresh=async()=>{
    await ui.waitForFunction(()=>{
      const node=[...document.querySelectorAll('button')].find(item=>item.textContent.trim().startsWith('Refresh required rule references'));
      return Boolean(node)&&!node.disabled;
    },undefined,{timeout:15000});
    await current.getByRole('button',{name:'Refresh required rule references',exact:true}).press('Enter');
  };
  // The same control becomes "Evaluate again" once a decision exists, so both labels are reached
  // through the public surface rather than a private setter.
  const clickEvaluate=async()=>{
    const again=ui.getByRole('button',{name:'Evaluate again',exact:true});
    if(await again.count()) await again.press('Enter');
    else await ui.getByRole('button',{name:'Evaluate risk',exact:true}).press('Enter');
  };
  await range.getByText(/reference missing/).waitFor({state:'visible'});
  assert.match(await range.innerText(),/BUY \(bid\) multipliers/);
  assert.match(await range.innerText(),/SELL \(ask\) multipliers/);
  assert.match(await range.innerText(),/1\.0001/);
  assert.match(await range.innerText(),/No snapshot bound is established/);
  await clickRefresh();
  await current.getByText('PERCENT_PRICE · PASS',{exact:true}).waitFor({state:'visible'});
  assert.match(await current.innerText(),/PROVIDER_REFERENCE/); assert.match(await current.innerText(),/60000 USDT/);
  assert.match(await current.innerText(),/Provider time/); assert.match(await current.innerText(),/First receipt/);
  assert.match(await current.innerText(),/PRICE_RANGE · UNAVAILABLE/);
  // The same genuine referencePrice read also qualifies the execution purpose this limit
  // actually needs, so the read-only snapshot becomes available without a second read.
  await range.getByText(/snapshot available/).waitFor({state:'visible'});
  assert.match(await range.innerText(),/59994 – 60006 USDT\/BTC/);
  await ui.getByRole('button',{name:'Evaluate risk',exact:true}).press('Enter');
  const capture=ui.getByRole('region',{name:'Captured Proposal Spot rules',exact:true}).first();
  await capture.waitFor({state:'visible'}); assert.match(await capture.innerText(),/PERCENT_PRICE · PASS/);
  assert.match(await capture.innerText(),/Price retained as digest only/);
  const captured=await capture.innerText();
  const viewport=await browser.capabilities.get('viewport');
  try {
    for(const width of [1280,768,390]){
      await viewport.set({width,height:900}); await tab.getAXState({emit:false});
      const size=await current.evaluate(element=>({width:window.innerWidth,scroll:document.documentElement.scrollWidth,panel:element.scrollWidth,client:element.clientWidth}));
      assert.equal(size.width,width); assert.ok(size.scroll<=width && size.panel<=size.client+1,JSON.stringify(size));
      const preview=await range.evaluate(element=>({scroll:element.scrollWidth,client:element.clientWidth}));
      assert.ok(preview.scroll<=preview.client+1,`price range preview at ${width}: ${JSON.stringify(preview)}`);
    }
  } finally { await viewport.reset(); }
  await send('binance.live.rules.fixture',{scenario:'PRICE_REJECT',symbol:'BTCUSDT'});
  const oldSource=await send('data.binance_rules.connection',{workspaceId});
  await send('data.binance_rules.refresh',{workspaceId,expectedStateVersion:oldSource.stateVersion});
  await current.getByText('PRICE_FILTER · REJECT',{exact:true}).waitFor({state:'visible'});
  assert.equal(await capture.innerText(),captured,'Current source replacement cannot renew a captured decision');
  assert.equal((await send('account.get',{workspaceId,connectionId:account.connectionId})).health.arming,'DISARMED');

  // S29.7 — the immutable Proposal explains its actual PRICE_RANGE execution rule.
  const switchSource=async scenario=>{
    await send('binance.live.rules.fixture',{scenario,symbol:'BTCUSDT'});
    const saved=await send('data.binance_rules.connection',{workspaceId});
    await send('data.binance_rules.refresh',{workspaceId,expectedStateVersion:saved.stateVersion});
  };
  await switchSource('PRICE_RANGE_PARTIAL');
  await range.getByText(/not enforced selected side/).waitFor({state:'visible'});
  const partial=await range.innerText();
  assert.match(partial,/lower multiplier not set/,'A documented omitted multiplier must not be invented');
  assert.match(partial,/1\.0001/);
  assert.match(partial,/Not enforced for this direction/);
  assert.match(partial,/price range not enforced for selected side/);
  assert.match(partial,/No snapshot bound is established/);

  await switchSource('NORMAL');
  // A new source generation retires the previous observation; waiting for that retirement also
  // proves the panel polled the new state version before the CAS refresh is offered.
  await range.getByText(/reference missing/).waitFor({state:'visible'});
  await clickRefresh();
  await range.getByText(/snapshot available/).waitFor({state:'visible'});
  const snapshot=await range.innerText();
  await clickEvaluate();
  const capturedRange=ui.getByRole('region',{name:'Captured price range execution preview',exact:true}).last();
  await capturedRange.waitFor({state:'visible'});
  const capturedRangeText=await capturedRange.innerText();
  assert.match(snapshot,/59994 – 60006 USDT\/BTC/);
  assert.match(snapshot,/BUY \(bid\) multipliers/); assert.match(snapshot,/SELL \(ask\) multipliers/);
  assert.match(snapshot,/Enforced for this direction/);
  assert.match(snapshot,/Provider time/); assert.match(snapshot,/First receipt/);
  assert.match(snapshot,/taker phase/); assert.match(snapshot,/expires the order/);
  assert.match(snapshot,/nothing here approves, places or promises a fill/);
  assert.match(snapshot,/sha256:[0-9a-f]{64}/,'A displayed bound must be traceable by digest');
  assert.match(capturedRangeText,/Snapshot bounds retained as digests only · sha256:[0-9a-f]{64}/,'A captured review must still name the bound digest it was made against');
  assert.ok(!capturedRangeText.includes('59994')&&!capturedRangeText.includes('60006'),'A captured review must not persist snapshot prices');
  assert.ok(!capturedRangeText.includes('60000'),'A captured review must not persist the execution reference price');

  await switchSource('PRICE_RANGE_NULL');
  await range.getByText(/reference missing/).waitFor({state:'visible'});
  await clickRefresh();
  // The same sentence appears in the prose explanation and in the execution reference row; the row
  // is the one that must carry the digest, so target the term definition value explicitly.
  const explicitNullReference=range.locator('dd',{hasText:/^Provider reported an explicit null reference price · sha256:[0-9a-f]{64}$/});
  await explicitNullReference.waitFor({state:'visible'});
  assert.match(await explicitNullReference.innerText(),/^Provider reported an explicit null reference price · sha256:[0-9a-f]{64}$/,'An explicit null must be reported as a verified provider fact traceable by digest');
  assert.match(await range.innerText(),/reference explicit null/);
  assert.equal(await capturedRange.innerText(),capturedRangeText,'A later current read cannot renew a captured review');
  assert.equal((await send('account.get',{workspaceId,connectionId:account.connectionId})).health.arming,'DISARMED');
  return {workspaceId,widths:[1280,768,390],currentRejected:true,captureFrozen:true,rawReferencePersisted:false,positiveAuthoritySeed:false,providerOrderWrites:false,priceRangePartial:true,priceRangeSnapshot:true,priceRangeExplicitNull:true,capturedBoundsStripped:true,capturedBoundsDigested:true};
}

export async function checkSpotRuleReviewUI(tab) {
  const ui=tab.playwright;
  await ui.getByRole('button',{name:'Review Live approval',exact:true}).press('Enter');
  const dialog=ui.getByRole('dialog',{name:'Confirm Live arming',exact:true});
  await dialog.waitFor({state:'visible'});
  const captured=dialog.getByRole('region',{name:'Captured Proposal Spot rules',exact:true});
  assert.equal(await captured.count(),1,'Read-only pre-arm review must explain its captured immutable Proposal rules');
  assert.match(await captured.innerText(),/Saved assessment/);
  assert.match(await captured.innerText(),/Exact account/);
  assert.equal(await captured.getByRole('button',{name:'Refresh required rule references',exact:true}).count(),0);
  const frozen=await captured.innerText();await tab.getAXState({emit:false});assert.equal(await captured.innerText(),frozen);
  await dialog.getByRole('button',{name:'Keep reviewing',exact:true}).press('Enter');
  await tab.getAXState({emit:false});
  return {preArmCapturedRules:true,referenceRefreshInCapture:false,armingPerformed:false};
}
