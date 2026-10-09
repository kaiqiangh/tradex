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
  await current.getByRole('button',{name:'Refresh required rule references',exact:true}).press('Enter');
  await current.getByText('PERCENT_PRICE · PASS',{exact:true}).waitFor({state:'visible'});
  assert.match(await current.innerText(),/PROVIDER_REFERENCE/); assert.match(await current.innerText(),/60000 USDT/);
  assert.match(await current.innerText(),/Provider time/); assert.match(await current.innerText(),/First receipt/);
  assert.match(await current.innerText(),/PRICE_RANGE · UNAVAILABLE/);
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
    }
  } finally { await viewport.reset(); }
  await send('binance.live.rules.fixture',{scenario:'PRICE_REJECT',symbol:'BTCUSDT'});
  const oldSource=await send('data.binance_rules.connection',{workspaceId});
  await send('data.binance_rules.refresh',{workspaceId,expectedStateVersion:oldSource.stateVersion});
  await current.getByText('PRICE_FILTER · REJECT',{exact:true}).waitFor({state:'visible'});
  assert.equal(await capture.innerText(),captured,'Current source replacement cannot renew a captured decision');
  assert.equal((await send('account.get',{workspaceId,connectionId:account.connectionId})).health.arming,'DISARMED');
  return {workspaceId,widths:[1280,768,390],currentRejected:true,captureFrozen:true,rawReferencePersisted:false,positiveAuthoritySeed:false,providerOrderWrites:false};
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
