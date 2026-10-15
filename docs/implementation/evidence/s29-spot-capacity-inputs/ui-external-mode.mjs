import {randomUUID} from 'node:crypto';
const scenario=process.argv[2];
const r=await(await fetch('http://127.0.0.1:1427/__integration/command',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({requestId:randomUUID(),schemaVersion:1,command:'binance.live.capacity.fixture',payload:{scenario}})})).json();
if(!r.ok)throw Error(JSON.stringify(r.error));
console.log(JSON.stringify({externalHttpScenario:scenario,controlPlaneSeeds:false}));
