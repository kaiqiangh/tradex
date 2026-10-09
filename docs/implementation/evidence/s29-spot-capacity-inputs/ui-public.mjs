import { randomUUID } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
const endpoint = 'http://127.0.0.1:1427/__integration/command';
async function call(command, payload) {
 const response = await fetch(endpoint,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({requestId:randomUUID(),schemaVersion:1,command,payload})});
 const result = await response.json();
 if(!result.ok) throw new Error(`${command}: ${JSON.stringify(result.error)}`);
 return result.data;
}
const mode=process.argv[2];
let data;
if(mode==='setup') {
 const workspace=await call('workspace.open',{});
 const workspaceId=workspace.workspaceId;
 const tested=await call('provider.connect',{step:'test',workspaceId,providerId:'binance',environment:'LIVE',label:'S29.5 isolated capacity account'});
 const account=await call('provider.connect',{step:'confirm',workspaceId,connectionId:tested.connectionId,expectedStateVersion:tested.stateVersion,acknowledgeUnverified:false});
 const current=await call('data.binance_rules.connection',{workspaceId});
 await call('data.binance_rules.configure',{workspaceId,expectedStateVersion:current.stateVersion,connectionId:account.connectionId,instrumentId:'crypto:BTC/USDT:spot'});
 await call('time.revalidate',{workspaceId});
 const draft=await call('trade.save_draft',{workspaceId,fields:{accountId:account.connectionId,venue:'BINANCE',environment:'BINANCE_LIVE',instrumentId:'crypto:BTC/USDT:spot',side:'BUY',orderType:'LIMIT',quantity:{type:'BASE',value:'0.001'},limitPrice:'60000',maximumSpend:null,timeInForce:'GTC'}});
 const proposal=await call('trade.generate_proposal',{workspaceId,draftId:draft.draftId,expectedDraftVersion:1});
 data={workspaceId,accountId:account.connectionId,draftId:draft.draftId,proposalId:proposal.proposalId};
 await writeFile('/tmp/s29-5-ui-owned.json',JSON.stringify(data,null,2));
 console.log(JSON.stringify(data));
} else {
 data=JSON.parse(await readFile('/tmp/s29-5-ui-owned.json','utf8'));
 await call('time.revalidate',{workspaceId:data.workspaceId});
 const source=await call('data.binance_rules.connection',{workspaceId:data.workspaceId});
 const result=await call('data.binance_rules.refresh',{workspaceId:data.workspaceId,expectedStateVersion:source.stateVersion});
 console.log(JSON.stringify({status:result.status,workspaceId:data.workspaceId}));
}
