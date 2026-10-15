// Real React/typed Rust/disposable workspace; only external HTTP/vault/WS producers are fake.
import assert from 'node:assert/strict';
import { checkSpotOwningAdmissionUI } from './spot-owning-admission-ui.mjs';
export async function checkSpotFeeMarketUI(tab, browser) {
  const state=await checkSpotOwningAdmissionUI(tab,browser), ui=tab.playwright;
  const source=await state.send('data.binance_market.connection',{workspaceId:state.workspaceId});
  await state.send('data.binance_market.configure',{workspaceId:state.workspaceId,expectedStateVersion:source.stateVersion});
  const panel=ui.getByRole('region',{name:'Current BNB fee source',exact:true});
  await panel.waitFor({state:'visible',timeoutMs:3000});
  await panel.getByText('Ordinary Binance public source selected.',{exact:true}).waitFor({state:'visible'});
  await panel.getByRole('button',{name:'Read BNB fee source',exact:true}).press('Enter');
  await panel.getByText('STREAMING · Read-only fee inputs · Execution qualification unavailable',{exact:true}).waitFor({state:'visible'});
  await tab.getAXState({emit:false});
  const text=await panel.innerText();
  assert.match(text,/500 \/ 501/);
  assert.match(text,/0\.5 \/ 0\.75 BNB/);
  assert.match(text,/BNBUSDT · BNB \/ USDT · TRADING/);
  assert.match(text,/UNVERIFIED/);
  assert.match(text,/never.*tradable/i);
  const trade=ui.getByRole('region',{name:'Trade Hot quote',exact:true});
  assert.match(await trade.innerText(),/STREAMING/);
  const viewport=await browser.capabilities.get('viewport'),widths=[];
  try {
    for(const width of [1280,768,390]) {
      await viewport.set({width,height:900});
      await tab.getAXState({emit:false});
      const size=await panel.evaluate(el=>({viewport:window.innerWidth,body:document.documentElement.scrollWidth,panel:el.scrollWidth,client:el.clientWidth}));
      assert.ok(size.body<=width&&size.panel<=size.client+1,JSON.stringify(size));
      widths.push(size);
    }
  } finally {await viewport.reset();}
  await panel.getByRole('button',{name:'Release BNB fee source',exact:true}).press('Enter');
  await panel.getByText('Fee source released.',{exact:true}).waitFor({state:'visible'});
  assert.match(await trade.innerText(),/STREAMING/);
  assert.equal((await state.send('account.get',{workspaceId:state.workspaceId,connectionId:state.accountId})).health.arming,'DISARMED');
  return {text,widths,tradingLeaseUnchanged:true,completeFinancialAcceptance:false,armingPerformed:false,consentPerformed:false};
}
