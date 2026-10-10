// Actual React/typed Rust/disposable storage; only external provider/vault responses change.
import assert from 'node:assert/strict';
import { checkSpotOwningAdmissionUI } from './spot-owning-admission-ui.mjs';
const CURRENT = 'Current Proposal symbol commission terms';
const CAPTURED = 'Captured Proposal symbol commission terms';
const captures = ui => ui.getByRole('region', { name: CAPTURED, exact: true });
async function widths(tab, browser, panel, prepare) {
  const viewport = await browser.capabilities.get('viewport'), sizes = [];
  try {
    for (const width of [1280,768,390]) {
      await viewport.set({width,height:900});
      if (prepare) await prepare();
      await tab.getAXState({emit:false});
      const size = await panel.evaluate(el => ({viewport:window.innerWidth,body:document.documentElement.scrollWidth,panel:el.scrollWidth,client:el.clientWidth}));
      assert.equal(size.viewport,width);assert.ok(size.body<=width && size.panel<=size.client+1,JSON.stringify(size));sizes.push(size);
    }
  } finally { await viewport.reset(); }
  return sizes;
}
export async function checkSpotSymbolCommissionUI(tab, browser) {
  const state = await checkSpotOwningAdmissionUI(tab,browser), ui = tab.playwright;
  const current = ui.getByRole('region',{name:CURRENT,exact:true});
  await current.waitFor({state:'visible',timeout:10000});
  assert.match(await current.innerText(),/not observed/);
  const refresh = async scenario => {
    await state.send('binance.live.commission.fixture',{scenario});
    await current.getByRole('button',{name:'Read symbol commission terms',exact:true}).press('Enter');
    await current.getByText('observed · Read-only declaration · Execution qualification unavailable',{exact:true}).waitFor({state:'visible'});
  };
  await refresh('NORMAL');
  const currentText = await current.innerText();
  for(const term of ['standard commission','tax commission','special commission','0.00000010','0.00000116','0.03000000','0.75000000','BUY received asset','BTC','BNB','not supplied','USDT ≠ USD']) assert.ok(currentText.includes(term),term);
  assert.match(currentText,/balance and conversion/);
  const currentWidths = await widths(tab,browser,current,()=>refresh('NORMAL'));
  await refresh('NORMAL');await state.evaluate();
  const saved = captures(ui).first(), capturedText = await saved.innerText();
  assert.match(capturedText,/0.00000010/);assert.equal(await saved.getByRole('button').count(),0);
  const measurements = await widths(tab,browser,saved);
  await refresh('CHANGED');await state.evaluate();
  const changedText = await captures(ui).first().innerText();assert.match(changedText,/0.00000025/);
  assert.ok((await Promise.all((await captures(ui).all()).map(p=>p.innerText()))).includes(capturedText),'A new collection cannot rewrite saved terms');
  await state.send('binance.live.commission.fixture',{scenario:'INCOMPLETE'});
  await current.getByRole('button',{name:'Read symbol commission terms',exact:true}).press('Enter');
  await current.getByText('unavailable · Read-only declaration · Execution qualification unavailable',{exact:true}).waitFor({state:'visible'});
  assert.ok((await Promise.all((await captures(ui).all()).map(p=>p.innerText()))).includes(capturedText));
  // Rebind all current read-only inputs before pre-arm; this performs no financial action.
  await state.refreshSource('COMPLETE_COVERAGE');await state.readCapacity();await state.readIntervals();await refresh('NORMAL');
  await ui.getByRole('button',{name:'Review Live approval',exact:true}).press('Enter');
  const dialog=ui.getByRole('dialog',{name:'Confirm Live arming',exact:true});await dialog.waitFor({state:'visible'});
  const prearm=dialog.getByRole('region',{name:CAPTURED,exact:true});const prearmText=await prearm.innerText();assert.match(prearmText,/0.00000010/);
  assert.equal(await prearm.getByRole('button').count(),0);assert.equal(await dialog.getByRole('button',{name:'Arm this Live account',exact:true}).isEnabled(),false);
  const prearmWidths=await widths(tab,browser,prearm);
  await dialog.getByRole('button',{name:'Keep reviewing',exact:true}).press('Enter');await tab.getAXState({emit:false});
  assert.equal((await state.send('account.get',{workspaceId:state.workspaceId,connectionId:state.accountId})).health.arming,'DISARMED');
  return {currentText,capturedText,changedText,prearmText,currentWidths,widths:measurements,prearmWidths,armingPerformed:false,consentPerformed:false,capturesUnchanged:true};
}
