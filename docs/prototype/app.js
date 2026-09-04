const state = {
  view: 'onboarding',
  onboardingStep: 1,
  mode: 'Research',
  account: 'No account',
  model: 'GPT-5.6 Sol',
  threadStatus: 'empty',
  threadId: 'us-tech-earnings',
  modal: null,
  liveArmed: false,
  pendingOrder: null,
  currentOrder: null,
  orderState: null,
  selectedInstrument: 'AAPL',
  instrumentTab: 'overview',
  watchlist: 'Tech Giants',
  selectedAccount: 'Trading 212 Live',
  strategy: 'list',
  artifact: null,
  settings: 'providers',
  recoveryState: 'sleep',
  toolFailure: false,
  screenerResult: false,
  toast: null,
  connectedProviders: new Set(['Alpaca Paper','Trading 212 Live','Binance Live','Bitget Live','US Market Data']),
  risk: {
    maxOrderNotional: '1000',
    maxExposure: '10',
    maxDailyNotional: '5000',
    maxDailyLoss: '300',
    staleQuote: '3',
    marketOrders: 'Disabled',
    inactivity: '20'
  }
};

const threadHistory = [
  ['us-tech-earnings','US tech earnings analysis','Today · 14:38'],
  ['btc-liquidity','BTC liquidity setup','Today · 11:02'],
  ['portfolio-risk','Portfolio risk review','Yesterday'],
  ['aapl-thesis','AAPL thesis refresh','Yesterday']
];

const accounts = [
  {name:'Local Paper', provider:'TradeX', env:'Paper', equity:'€100,000', pnl:'+€2,341', status:'Active', type:'Paper'},
  {name:'Alpaca Paper', provider:'Alpaca', env:'Paper', equity:'€50,000', pnl:'+€1,234', status:'Healthy', type:'Paper'},
  {name:'Trading 212 Demo', provider:'Trading 212', env:'Demo', equity:'€25,000', pnl:'+€188', status:'Healthy', type:'Demo'},
  {name:'Trading 212 Live', provider:'Trading 212', env:'Live', equity:'€12,432', pnl:'+€432', status:'Healthy', type:'Live'},
  {name:'Binance Testnet', provider:'Binance', env:'Testnet', equity:'€20,000', pnl:'+€520', status:'Healthy', type:'Demo'},
  {name:'Binance Live', provider:'Binance', env:'Live', equity:'€8,234', pnl:'+€321', status:'Healthy', type:'Live'},
  {name:'Bitget Demo', provider:'Bitget', env:'Demo', equity:'€15,000', pnl:'+€410', status:'Healthy', type:'Demo'},
  {name:'Bitget Live', provider:'Bitget', env:'Live', equity:'€6,543', pnl:'+€210', status:'Healthy', type:'Live'}
];

const positions = [
  ['AAPL','Trading 212 Live','€8,143','+€164','6.5%'],
  ['NVDA','Trading 212 Live','€7,202','+€428','5.7%'],
  ['BTC','Binance Live','€5,836','+€184','4.7%'],
  ['ETH','Bitget Live','€3,744','-€62','3.0%'],
  ['MSFT','Alpaca Paper','€7,942','+€312','6.3%']
];

const marketRows = [
  ['NVDA','138.12','+4.71%','45.2M','Earnings strength'],
  ['AMD','162.34','+5.12%','38.1M','Momentum'],
  ['TSLA','248.21','+4.83%','62.4M','High volatility'],
  ['META','512.21','+1.34%','13.0M','Trend continuation'],
  ['NFLX','612.21','-2.84%','8.2M','Weak breadth'],
  ['BTC/USDT','62,418','+2.18%','1.8B','Liquidity improving']
];

const watchRows = [
  ['AAPL','221.42','+1.24%','3.41T','Quality compounder','—'],
  ['MSFT','412.34','+0.87%','3.06T','AI platform','Price > 420'],
  ['GOOGL','162.24','+1.56%','1.98T','Search + cloud','—'],
  ['AMZN','186.45','+0.92%','1.94T','Margin expansion','Earnings'],
  ['META','512.21','+1.34%','1.30T','Ad efficiency','—'],
  ['TSLA','248.21','+4.83%','0.79T','High variance','Price < 220'],
  ['NVDA','138.12','+4.71%','3.41T','AI compute leader','Exposure > 10%']
];

const strategies = [
  ['MA Crossover','Technical','BTC','v12','2 days ago','Active'],
  ['RSI Mean Reversion','Technical','ETH','v8','5 days ago','Draft'],
  ['Pairs Trading','Statistical','Stocks','v4','1 week ago','Active'],
  ['Earnings Momentum','Fundamental','Stocks','v9','1 week ago','Draft'],
  ['Crypto Arbitrage','Arbitrage','Crypto','v3','2 weeks ago','Archived']
];

const artifacts = [
  ['NVDA vs AMD analysis','Research','US tech earnings','2h ago','2.4 MB'],
  ['BTC liquidity comparison','Research','BTC venue setup','5h ago','1.1 MB'],
  ['MA Crossover v12','Backtest','Strategy research','1d ago','1.8 MB'],
  ['Tech Sector Screener','Dataset','Screening run','2d ago','0.5 MB'],
  ['Portfolio Risk Review','Report','Portfolio review','3d ago','1.2 MB'],
  ['AAPL live trade review','Trade review','AAPL position','4d ago','0.8 MB']
];

const providers = [
  ['Alpaca Paper','US equities · Paper','Paper'],
  ['Trading 212 Demo','US equities · Demo','Demo'],
  ['Trading 212 Live','US equities · Live','Live'],
  ['Binance Testnet','Crypto spot · Testnet','Demo'],
  ['Binance Live','Crypto spot · Live','Live'],
  ['Bitget Demo','Crypto spot · Demo','Demo'],
  ['Bitget Live','Crypto spot · Live','Live'],
  ['US Market Data','Quotes · OHLCV · Fundamentals','Data']
];

function esc(s){return String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]))}
function fmtEnv(env){const cls=(env==='Live')?'live':(env==='Paper'||env==='Demo'||env==='Testnet')?'paper':'research';return `<span class="status ${cls==='paper'?'ok':cls}">${env}</span>`}
function chart(color='#16a34a', fill='#dcfce7'){
  const id='g'+Math.random().toString(36).slice(2,7);
  return `<svg viewBox="0 0 300 140" preserveAspectRatio="none"><defs><linearGradient id="${id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${fill}" stop-opacity=".75"/><stop offset="1" stop-color="${fill}" stop-opacity="0"/></linearGradient></defs><path d="M0 116 L18 102 L32 108 L50 79 L70 88 L88 65 L105 70 L122 48 L138 58 L156 43 L174 52 L192 36 L208 42 L226 28 L242 34 L260 21 L276 28 L300 16 L300 140 L0 140 Z" fill="url(#${id})"/><path d="M0 116 L18 102 L32 108 L50 79 L70 88 L88 65 L105 70 L122 48 L138 58 L156 43 L174 52 L192 36 L208 42 L226 28 L242 34 L260 21 L276 28 L300 16" fill="none" stroke="${color}" stroke-width="2"/></svg>`;
}
function metric(label,value,delta,color=''){return `<div class="metric"><div class="label">${label}</div><div class="value ${color}">${value}</div><div class="delta" ${color==='negative'?'style="color:var(--red)"':''}>${delta}</div></div>`}
function table(headers, rows, clickFn=''){
  return `<table class="table"><thead><tr>${headers.map(h=>`<th>${h}</th>`).join('')}</tr></thead><tbody>${rows.map(r=>`<tr class="${clickFn?'clickable':''}" ${clickFn?`onclick="${clickFn}('${esc(r[0])}')"`:''}>${r.map(v=>`<td class="${String(v).startsWith('+')?'positive':String(v).startsWith('-')?'negative':''}">${v}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
}
function button(label, action, kind=''){return `<button class="btn ${kind}" onclick="${action}">${label}</button>`}
function pill(label, cls=''){return `<span class="pill ${cls}">${label}</span>`}

function navItem(name,label){return `<button class="${state.view===name?'active':''}" onclick="go('${name}')">${label}</button>`}
function sidebar(){
  const history = state.view==='thread' ? `<div class="thread-history"><div class="thread-history-label">Today</div>${threadHistory.slice(0,2).map(r=>`<button class="${state.threadId===r[0]?'active':''}" onclick="openThread('${r[0]}')"><span>${r[1]}</span><small>${r[2]}</small></button>`).join('')}<div class="thread-history-label">Earlier</div>${threadHistory.slice(2).map(r=>`<button class="${state.threadId===r[0]?'active':''}" onclick="openThread('${r[0]}')"><span>${r[1]}</span><small>${r[2]}</small></button>`).join('')}</div>`:'';
  return `<aside class="sidebar"><div class="brand">Trade<b>X</b></div><button class="new-thread" onclick="newThread()">+ New Thread</button><div class="nav">${navItem('thread','Threads')}${navItem('markets','Markets')}${navItem('watchlists','Watchlists')}${navItem('accounts','Accounts')}${navItem('strategies','Strategies')}${navItem('artifacts','Artifacts')}</div>${history}<div class="sidebar-rule"></div><div class="settings-nav"><button class="${state.view==='settings'&&state.settings==='providers'?'active':''}" onclick="goSettings('providers')">Providers & Models</button><button class="${state.view==='settings'&&state.settings==='risk'?'active':''}" onclick="goSettings('risk')">Risk & Limits</button><button class="${state.view==='settings'&&state.settings==='data'?'active':''}" onclick="goSettings('data')">Data & Storage</button><button class="${state.view==='recovery'?'active':''}" onclick="go('recovery')">Account Health</button></div><div class="runtime"><strong><span class="dot"></span>Local workspace</strong><small>Codex runtime connected</small></div></aside>`;
}
function mobileNav(){return `<nav class="mobile-nav"><button onclick="go('thread')" class="${state.view==='thread'?'active':''}">Chat</button><button onclick="go('markets')" class="${state.view==='markets'||state.view==='instrument'?'active':''}">Markets</button><button onclick="go('watchlists')" class="${state.view==='watchlists'?'active':''}">Watch</button><button onclick="go('accounts')" class="${state.view==='accounts'||state.view==='portfolio'||state.view==='accountDetail'?'active':''}">Accounts</button><button onclick="go('strategies')" class="${state.view==='strategies'?'active':''}">Strategy</button></nav>`}
function titleFor(){
  const m={
    onboarding:'Welcome to TradeX',thread:state.threadStatus==='empty'?'New thread':state.threadId==='btc-liquidity'?'BTC liquidity setup':'US tech earnings analysis',
    markets:'Market Explorer',instrument:state.selectedInstrument==='BTC/USDT'?'BTC / USDT — Binance':'AAPL — Apple Inc.',watchlists:'Watchlists',accounts:'Accounts',portfolio:'Portfolio',accountDetail:state.selectedAccount,
    strategies:'Strategies',artifacts:'Artifacts',settings:state.settings==='providers'?'Providers & Models':state.settings==='risk'?'Risk & Limits':'Data & Storage',orderMonitor:'Order activity',recovery:'Account Health'
  };return m[state.view]||'TradeX';
}
function topbar(){
  if(state.view==='onboarding') return '';
  let mode=state.view==='thread'?state.mode:(state.view==='orderMonitor'||state.view==='accountDetail'?'Live':'Research');
  let account=state.view==='thread'?state.account:(state.view==='orderMonitor'&&state.currentOrder?state.currentOrder.account:state.view==='accountDetail'?state.selectedAccount:'No account');
  return `${state.liveArmed?`<div class="live-banner"><span>LIVE EXECUTION ARMED · Every live transaction still requires one-time approval.</span><button class="btn danger" style="height:28px" onclick="disarmLive()">Disable Live</button></div>`:''}<header class="topbar"><div class="title">${esc(titleFor())}</div><div class="top-actions"><span class="pill ${mode.toLowerCase()}">${mode}</span>${account!=='No account'?`<span class="pill">${account}</span>`:''}<button class="btn ghost" onclick="showToast('Share link copied for prototype review')">Share</button></div></header>`;
}
function shell(content){return `<div class="app">${sidebar()}<section class="shell">${topbar()}<main class="main">${content}</main></section>${mobileNav()}${modal()}${toast()}</div>`}
function toast(){if(!state.toast)return '';return `<div class="toast">${esc(state.toast)}</div>`}

function contextPanel(){
  const crypto=state.selectedInstrument==='BTC/USDT';
  return `<aside class="context"><h3>${crypto?'BTC / USDT':'AAPL'}</h3><div class="sub">${crypto?'Binance · Spot':'NASDAQ · Equity'}</div><div class="quote-row"><div class="quote">${crypto?'62,418.20':'221.42'}</div><div class="positive"><b>${crypto?'+2.18%':'+1.24%'}</b></div></div><div class="mini-chart">${chart()}</div><div class="section-label">Position</div><div class="kv"><span>${crypto?'Balance':'Shares'}</span><span>${crypto?'0.083 BTC':'12'}</span></div><div class="kv"><span>Unrealized P&L</span><span class="positive">${crypto?'+€184.22':'+$164.40'}</span></div><div class="section-label">Market context</div><div class="kv"><span>Spread</span><span>${crypto?'1.8 bps':'$0.03'}</span></div><div class="kv"><span>Quote age</span><span>${crypto?'280 ms':'420 ms'}</span></div><div class="kv"><span>${crypto?'Venue':'Session'}</span><span>${crypto?'Binance':'Open'}</span></div><div class="kv"><span>Open orders</span><span>1</span></div><div class="section-label">Evidence</div><div class="small">3 sources attached<br/>Latest refresh 12 sec ago</div>${crypto?`<button class="btn full" style="margin-top:14px" onclick="openMarketOrder()">Prepare Market Buy</button>`:''}</aside>`;
}
function composer(){
  return `<div class="composer"><textarea id="prompt" placeholder="Ask TradeX to research, compare, backtest, or prepare an order…"></textarea><div class="composer-footer"><div class="composer-left"><button class="pill" onclick="openContextPicker()" style="border:0;cursor:pointer">@ Context</button><button class="pill ${state.mode.toLowerCase()}" onclick="cycleMode()" style="border:0;cursor:pointer">${state.mode}</button><button class="pill" onclick="openAccountPicker()" style="border:0;cursor:pointer">${state.account}</button><button class="pill" onclick="openModelPicker()" style="border:0;cursor:pointer">${state.model}</button></div><button class="btn primary" onclick="sendPrompt()">Send</button></div></div>`;
}

function onboarding(){
  const steps=['Workspace','Providers','Model','Risk defaults','Ready'];
  const stepChips=steps.map((x,i)=>`<span class="pill ${i+1===state.onboardingStep?'research':''}">${i+1} ${x}</span>`).join('');
  let body='';
  if(state.onboardingStep===1){
    body=`<h2>Create local workspace</h2><div class="field"><label>Workspace name</label><input id="workspaceName" value="My Trading Workspace"/></div><div class="field"><label>Base currency</label><select><option>EUR — Euro</option><option>USD — US Dollar</option></select></div><div class="field"><label>Local storage</label><input value="~/.tradex/workspaces/default"/></div><div class="modal-actions">${button('Continue to providers','nextOnboarding()','primary')}</div>`;
  } else if(state.onboardingStep===2){
    body=`<h2>Connect providers</h2><p class="muted">Demo/testnet and live environments are separate connections. You can finish setup without connecting a live account.</p><div class="provider-grid">${providers.slice(0,7).map(p=>`<div class="provider-card"><div><strong>${p[0]}</strong><small>${p[1]}</small></div>${fmtEnv(p[2])}<button class="btn" onclick="connectProvider('${p[0]}')">${state.connectedProviders.has(p[0])?'Connected':'Connect'}</button></div>`).join('')}</div><div class="modal-actions">${button('Back','prevOnboarding()')}${button('Continue to model','nextOnboarding()','primary')}</div>`;
  } else if(state.onboardingStep===3){
    body=`<h2>Choose agent model</h2><p class="muted">The Codex App Server is local; model inference may use the configured provider.</p><div class="selection-list">${['GPT-5.6 Sol','GPT-5.6 Luna','Local model (future)'].map((m,i)=>`<button class="selection-row ${state.model===m?'selected':''}" onclick="selectModel('${m}')"><div><strong>${m}</strong><small>${i===0?'Recommended for complex research and trading workflows':i===1?'Fast everyday reasoning':'Not enabled in this prototype'}</small></div><span>${state.model===m?'✓':''}</span></button>`).join('')}</div><div class="modal-actions">${button('Back','prevOnboarding()')}${button('Continue to risk defaults','nextOnboarding()','primary')}</div>`;
  } else if(state.onboardingStep===4){
    body=`<h2>Set conservative live-trading defaults</h2><p class="muted">The agent cannot modify these controls. Weakening a limit invalidates pending approvals.</p>${riskForm(true)}<div class="modal-actions">${button('Back','prevOnboarding()')}${button('Review setup','nextOnboarding()','primary')}</div>`;
  } else {
    body=`<h2>Workspace ready</h2><p class="muted">Your local workspace, model and trading safety defaults are configured.</p><div class="ready-grid"><div class="ready-item"><b>Workspace</b><span>My Trading Workspace · EUR</span></div><div class="ready-item"><b>Providers</b><span>${state.connectedProviders.size} connected</span></div><div class="ready-item"><b>Model</b><span>${state.model}</span></div><div class="ready-item"><b>Live trading</b><span>DISARMED by default</span></div></div><div class="warning-box">Live trading remains disabled after onboarding. Arming requires a separate explicit action, and every live order still requires transaction-specific approval.</div><div class="modal-actions">${button('Back','prevOnboarding()')}${button('Create first thread','finishOnboarding()','primary')}</div>`;
  }
  return `<div class="onboarding-full"><div class="onboard-left"><div class="brand" style="margin:0">Trade<b>X</b></div><h1>Your markets.<br>Your agent.<br>Your workspace.</h1><p>Research, strategy and execution stay connected in one local workspace.</p><small>Primary data stays local. Broker credentials remain in the OS credential vault.</small></div><div class="onboard-main"><h1>Welcome to TradeX</h1><p class="muted">Set up your workspace in a few steps.</p><div class="steps">${stepChips}</div><div class="onboard-card">${body}</div></div>${modal()}${toast()}</div>`;
}

function thread(){
  if(state.threadStatus==='empty') return shell(`<div class="workspace"><section class="content"><div class="hero-empty"><h1>What do you want to investigate?</h1><p>Research a market, review a portfolio, test a strategy, or prepare an order. Complex tasks show their plan, tools, evidence and trading states directly in the thread.</p><div class="suggestions"><button class="suggestion" onclick="startResearch()"><strong>Compare NVDA, AMD and AVGO</strong><span class="pill research">Research</span></button><button class="suggestion" onclick="portfolioReview()"><strong>Review my live portfolio risk</strong><span class="pill">Portfolio</span></button><button class="suggestion" onclick="openStrategy()"><strong>Backtest BTC momentum strategy</strong><span class="pill">Backtest</span></button><button class="suggestion" onclick="startCryptoResearch()"><strong>Compare BTC liquidity on Binance and Bitget</strong><span class="pill">Crypto</span></button></div></div>${composer()}</section>${contextPanel()}</div>`);
  if(state.threadStatus==='running') return researchRunning();
  if(state.threadStatus==='toolError') return toolErrorView();
  return researchResult();
}
function toolCard(name,detail,status,type='ok'){
  const color=type==='error'?'var(--red)':type==='run'?'var(--blue)':type==='retry'?'var(--amber)':'var(--green)';
  return `<div class="tool-card"><span style="color:${color}">●</span><div><strong>${name}</strong><small>${detail}</small></div><div class="state" style="color:${color}">${status}</div></div>`;
}
function researchRunning(){
  return shell(`<div class="workspace"><section class="content"><div class="timeline-title">Compare NVDA, AMD and AVGO after earnings. Check sentiment, valuation and my exposure.</div><div class="plan"><b>Agent plan</b>${[['✓','Fetch earnings and guidance','Done'],['✓','Compare financial metrics','Done'],['✓','Review news and sentiment','Done'],['●','Check portfolio exposure','Running'],['○','Synthesize bull/base/bear cases','Queued']].map(r=>`<div class="plan-row"><span class="${r[0]==='✓'?'positive':''}">${r[0]}</span><span>${r[1]}</span><span class="state">${r[2]}</span></div>`).join('')}</div>${toolCard('market.earnings','NVDA, AMD, AVGO · filings + fundamentals','Completed')}${toolCard('research.news_search','12 articles · source timestamps preserved','Completed')}${toolCard('portfolio.exposure','Trading 212 Live · read-only account context','Running…','run')}<div class="card"><b>Interim findings</b><p style="line-height:1.65">NVDA shows the strongest post-earnings revenue revision and margin profile, while AMD trades at a lower valuation. Your existing semiconductor exposure is being checked before any position recommendation.</p><div class="toolbar-actions">${button('Simulate tool failure','simulateToolFailure()')}</div></div>${composer()}</section>${contextPanel()}</div>`);
}
function toolErrorView(){
  return shell(`<div class="workspace"><section class="content"><div class="timeline-title">US tech earnings analysis</div><div class="plan"><b>Agent plan</b><div class="plan-row"><span class="positive">✓</span><span>Fetch earnings and guidance</span><span class="state">Done</span></div><div class="plan-row"><span>×</span><span>Check portfolio exposure</span><span class="state negative">Failed</span></div></div>${toolCard('portfolio.exposure','Trading 212 returned a temporary network error','Failed','error')}<div class="error-box"><b>NETWORK_ERROR</b><br/>The failed read did not change any account state. You can retry the tool or continue without portfolio context.</div><div class="modal-actions">${button('Cancel turn','cancelTurn()')}${button('Retry tool','retryTool()','primary')}</div>${composer()}</section>${contextPanel()}</div>`);
}
function researchResult(){
  const crypto=state.threadId==='btc-liquidity';
  if(crypto) return cryptoResearchResult();
  return shell(`<div class="workspace"><section class="content"><div class="toolbar"><div><h1 style="font-size:18px">Research complete</h1><p>6 sources · market data and portfolio state timestamped</p></div><span class="status ok">Completed</span></div><div class="card analysis-result"><h2>NVDA remains the strongest setup, but current exposure limits incremental size.</h2><div class="section-label">Key findings</div>${['Revenue estimate revisions remain strongest for NVDA.','AMD valuation is cheaper but estimate momentum is weaker.','AVGO has lower beta but slower near-term growth.','Current semiconductor exposure is 8.6% of portfolio.'].map(v=>`<div class="bullet">${v}</div>`).join('')}<div class="section-label">Scenario view</div><div class="scenario-grid"><div class="scenario"><strong class="positive">Bull</strong><div class="big">+18% upside</div><small>$152</small></div><div class="scenario"><strong style="color:var(--blue)">Base</strong><div class="big">+7%</div><small>$138</small></div><div class="scenario"><strong class="negative">Bear</strong><div class="big">Multiple compression</div><small>$112</small></div></div><div class="section-label">Artifacts created</div><div class="artifact-row"><span class="artifact-chip blue">Research report.md</span><span class="artifact-chip">Peer comparison.csv</span><span class="artifact-chip">Scenario chart</span></div><div class="action-strip"><button class="btn" onclick="openPaperModal()">Prepare Paper Order</button><button class="btn" onclick="openRiskRejected()">Simulate Risk Reject</button><button class="btn danger" onclick="requestLiveOrder('aaplLimit')">Prepare Live Order</button></div></div>${composer()}</section>${contextPanel()}</div>`);
}
function cryptoResearchResult(){
  state.selectedInstrument='BTC/USDT';
  return shell(`<div class="workspace"><section class="content"><div class="toolbar"><div><h1 style="font-size:18px">BTC venue comparison complete</h1><p>Binance and Bitget · realtime venue snapshots</p></div><span class="status ok">Completed</span></div><div class="card analysis-result"><h2>Binance currently offers the tighter spread and deeper top-of-book liquidity.</h2><div class="venue-compare"><div class="venue-card"><b>Binance</b><span class="status ok">Preferred</span><div class="kv"><span>Spread</span><span>1.8 bps</span></div><div class="kv"><span>Top 10 depth</span><span>$18.2M</span></div><div class="kv"><span>Quote age</span><span>280 ms</span></div></div><div class="venue-card"><b>Bitget</b><div class="kv"><span>Spread</span><span>2.6 bps</span></div><div class="kv"><span>Top 10 depth</span><span>$9.7M</span></div><div class="kv"><span>Quote age</span><span>340 ms</span></div></div></div><div class="action-strip"><button class="btn" onclick="openCryptoInstrument()">Open BTC detail</button><button class="btn danger" onclick="requestLiveOrder('btcMarket')">Prepare Binance Market Buy</button></div></div>${composer()}</section>${contextPanel()}</div>`);
}

function markets(){
  const rows=state.screenerResult?[
    ['NVDA','138.12','+4.71%','45.2M','Quality + revisions'],['AVGO','1,652','+1.82%','3.6M','Quality growth'],['META','512.21','+1.34%','13.0M','Strong margins'],['MSFT','412.34','+0.87%','21.2M','AI platform'],['AMD','162.34','+5.12%','38.1M','Revision improving']
  ]:marketRows;
  return shell(`<div class="toolbar"><div><h1>Market Explorer</h1><p>Structured screeners reduce the universe before deep agent research.</p></div></div><input class="search" placeholder="Search symbols, companies, sectors, or crypto pairs…" onkeydown="if(event.key==='Enter') openInstrument(this.value||'AAPL')"/><div class="filter-row"><button class="pill research" onclick="showToast('US Stocks filter active')" style="border:0">US Stocks</button><button class="pill" onclick="openCryptoInstrument()" style="border:0">Crypto</button><button class="pill" onclick="openScreenerBuilder()" style="border:0">Screeners</button></div><div class="grid3">${metric('S&P 500','5,473.23','+0.55%')}${metric('NASDAQ','17,321.12','+0.92%')}${metric('BTC','62,418','+2.18%')}</div><div class="market-layout" style="margin-top:20px"><div class="table-wrap"><div class="table-head"><h3>${state.screenerResult?'Screener results · 5 candidates':'Top movers'}</h3><small class="muted">Realtime · source timestamped</small></div>${table(['Symbol','Price','Change','Volume','Signal'],rows,'openInstrument')}</div><div class="card screener-list"><h3>Saved screeners</h3>${['US Quality Growth','Earnings Revisions','Crypto OI Expansion','Crowded Longs','Low-vol Portfolio Candidates'].map((v,i)=>`<div class="row"><strong>${v}</strong><small>${i<2?'US equities':'Crypto / cross-market'}</small></div>`).join('')}<button class="btn full" style="margin-top:14px" onclick="openScreenerBuilder()">+ New screener</button></div></div>`);
}
function instrument(){
  return state.selectedInstrument==='BTC/USDT'?cryptoInstrument():equityInstrument();
}
function equityInstrument(){
  const tabCopy={overview:'Apple remains a high-quality compounder with resilient services growth. Near-term attention is on device demand, gross-margin durability and valuation sensitivity to rates.',financials:'Revenue growth is supported by services mix and installed-base monetization. Margin and cash conversion remain key quality indicators.',news:'Recent coverage focuses on product cycle expectations, services momentum and AI feature adoption. Sources retain publication timestamps.',filings:'Latest filing evidence is summarized with source links and reporting-period metadata.',analysis:'Agent view: quality remains high, but valuation sensitivity and portfolio concentration should constrain position sizing.'};
  return shell(`<div class="toolbar"><div><div class="instrument-header"><h1>AAPL</h1><span class="muted">Apple Inc. · NASDAQ</span></div><div class="price">221.42 <span class="positive" style="font-size:12px">+1.24%</span></div></div><button class="btn" onclick="openAddWatchlist()">+ Watchlist</button></div><div class="chart-large">${chart()}</div><div class="card" style="padding:0"><div class="tabs">${['overview','financials','news','filings','analysis'].map(t=>`<button class="${state.instrumentTab===t?'active':''}" onclick="instrumentTab('${t}')">${t[0].toUpperCase()+t.slice(1)}</button>`).join('')}</div><div style="padding:20px"><h3>${state.instrumentTab[0].toUpperCase()+state.instrumentTab.slice(1)}</h3><p style="line-height:1.7">${tabCopy[state.instrumentTab]}</p><div class="artifact-row"><span class="artifact-chip blue">AAPL quality compounder</span><span class="artifact-chip">Quote source: MarketData-X · 420 ms</span><span class="artifact-chip">Next earnings: Oct 29</span></div></div></div>`);
}
function cryptoInstrument(){
  return shell(`<div class="toolbar"><div><div class="instrument-header"><h1>BTC / USDT</h1><span class="muted">Crypto Spot · Binance</span></div><div class="price">62,418.20 <span class="positive" style="font-size:12px">+2.18%</span></div></div><div class="toolbar-actions"><button class="btn" onclick="state.threadId='btc-liquidity';state.threadStatus='result';go('thread')">Open research thread</button><button class="btn danger" onclick="requestLiveOrder('btcMarket')">Prepare Market Buy</button></div></div><div class="chart-large">${chart('#2563eb','#dbeafe')}</div><div class="split"><div class="card"><h3>Venue market context</h3>${[['Best bid','62,416.80'],['Best ask','62,418.20'],['Spread','1.8 bps'],['24h volume','$1.8B'],['Top 10 book depth','$18.2M'],['Quote age','280 ms']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}</div><div class="card"><h3>Cross-venue comparison</h3>${[['Binance spread','1.8 bps'],['Bitget spread','2.6 bps'],['Binance top-10 depth','$18.2M'],['Bitget top-10 depth','$9.7M'],['My Binance balance','0.083 BTC']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}<div class="warning-box">Spot trading only in v1.0. Futures, leverage and margin actions are not available.</div></div></div>`);
}

function watchlists(){
  return shell(`<div class="toolbar"><div><h1>Watchlists</h1><p>Watchlists drive warm subscriptions and thesis monitoring.</p></div><button class="btn primary" onclick="openNewWatchlist()">+ New Watchlist</button></div><div class="watchlist-layout"><div class="watchlist-menu">${['Tech Giants','AI Semiconductors','Crypto Top 10','Earnings Watch','Swing Trading','Long-term'].map(v=>`<button class="${state.watchlist===v?'active':''}" onclick="setWatchlist('${v}')">${v}</button>`).join('')}</div><div class="table-wrap"><div class="table-head"><h3>${state.watchlist}</h3><small class="muted">Warm market-data tier</small></div>${table(['Symbol','Price','Change','Market Cap','Thesis','Alert'],watchRows,'openInstrument')}</div></div>`);
}

function accountsView(){
  const rows=accounts.map(a=>[a.name,a.env,a.equity,a.pnl,a.status]);
  return shell(`<div class="toolbar"><div><h1>Accounts</h1><p>Demo/testnet and live environments are always separate connections.</p></div><div class="toolbar-actions"><button class="btn" onclick="go('portfolio')">Portfolio</button><button class="btn primary" onclick="openConnectAccount()">+ Connect Account</button></div></div><div class="grid3">${metric('Total equity','€125,432.21','+2.34% today')}${metric('Unrealized P&L','+€8,432.21','+7.21%','positive')}${metric('Available cash','€32,118.43','Across connected accounts')}</div><div class="table-wrap" style="margin-top:20px"><div class="table-head"><h3>Connected accounts</h3><small class="muted">Broker state authoritative</small></div>${table(['Account','Environment','Equity','P&L','Status'],rows,'openAccount')}</div>`);
}
function portfolio(){
  return shell(`<div class="toolbar"><div><h1>Portfolio</h1><p>Cross-account exposure normalized to EUR with timestamped FX.</p></div><span class="pill">Base: EUR</span></div><div class="grid3">${metric('Portfolio value','€125,432','+2.34%')}${metric('1D P&L','+€2,864','+2.34%','positive')}${metric('Cash','€32,118','25.6%')}</div><div class="allocation" style="margin-top:20px"><div class="card"><h3>Allocation</h3>${[['US Equities',58],['Crypto',16],['Cash',26]].map(r=>`<div style="margin-top:20px"><div class="kv"><span>${r[0]}</span><span>${r[1]}%</span></div><div class="bar"><span style="width:${r[1]}%"></span></div></div>`).join('')}<div class="section-label">Risk concentration</div><div class="kv"><span>Technology</span><span>32.4%</span></div><div class="kv"><span>Semiconductors</span><span>8.6%</span></div><div class="kv"><span>BTC</span><span>6.4%</span></div></div><div class="table-wrap"><div class="table-head"><h3>Positions</h3><small class="muted">FX source · 12 sec ago</small></div>${table(['Instrument','Account','Value (EUR)','P&L','Weight'],positions)}</div></div>`);
}
function accountDetail(){
  const a=accounts.find(x=>x.name===state.selectedAccount)||accounts[3];
  const isLive=a.env==='Live';
  const isCrypto=a.provider==='Binance'||a.provider==='Bitget';
  const perms=isLive?['Account read','Positions read','Order read','Place order','Cancel order']:['Account read','Positions read','Order read','Paper/Demo execute'];
  const title=`${a.provider} ${fmtEnv(a.env)} <span class="status ok">Healthy</span>`;
  return shell(`<div class="toolbar"><div><h1>${title}</h1><p>${isCrypto?'Crypto spot':'US equities'} · ${a.env} · reconciled 12 sec ago</p></div>${isLive?`<button class="btn ${state.liveArmed?'danger':'primary'}" onclick="${state.liveArmed?'disarmLive()':'openArmLive()'}">${state.liveArmed?'Disable Live':'Arm Live Trading'}</button>`:''}</div><div class="grid3">${metric('Equity',a.equity,a.pnl+' today')}${metric('Available cash',isCrypto?'€3,884.12':'€3,212.43','Primary currency EUR')}${metric('Open orders',isLive?'2':'1','Broker reconciled')}</div><div class="split" style="margin-top:20px"><div class="card"><h3>Connection health</h3>${[['REST','Healthy · 142 ms'],['Order reconciliation','Healthy · 12 sec ago'],['Credentials','Valid · Keychain'],['Capability',isLive?'Live orders · approval gated':'Demo/Paper execution']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}<div class="section-label">Permissions</div><div class="artifact-row">${perms.map(p=>`<span class="artifact-chip">✓ ${p}</span>`).join('')}</div><div class="warning-box">Withdrawals, transfers, leverage changes and custody actions are not implemented by TradeX.</div></div><div><div class="card"><h3>Open orders</h3><div class="open-order"><div><b>${isCrypto?'BTC/USDT':'AAPL'}</b><small>${isCrypto?'BUY 0.005 BTC · LIMIT':'BUY 2 AAPL · LIMIT $220.50'}</small></div><span class="status warn">PARTIAL</span></div><div class="kv"><span>Filled</span><span>${isCrypto?'0.002 / 0.005 BTC':'1 / 2 shares'}</span></div>${isLive?`<button class="btn full" style="margin-top:12px" onclick="requestCancel()">Cancel remaining</button>`:''}</div><div class="card"><h3>Account diagnostics</h3><button class="btn full" onclick="showRecovery('auth')">Simulate auth error</button><button class="btn full" style="margin-top:8px" onclick="showRecovery('stream')">Simulate stream disconnect</button><button class="btn full" style="margin-top:8px" onclick="showRecovery('rate')">Simulate rate limit</button></div></div></div>`);
}

function strategiesView(){
  if(state.strategy==='editor') return strategyEditor();
  if(state.strategy==='running') return strategyRunning();
  if(state.strategy==='results') return backtestResults();
  if(state.strategy==='failed') return strategyFailed();
  if(state.strategy==='compare') return strategyCompare();
  return shell(`<div class="toolbar"><div><h1>Strategies</h1><p>Versioned local strategies execute inside a restricted sandbox.</p></div><button class="btn primary" onclick="newStrategy()">+ New Strategy</button></div><div class="table-wrap"><div class="table-head"><h3>My strategies</h3><small class="muted">Local workspace</small></div>${table(['Name','Type','Assets','Version','Last run','Status'],strategies,'selectStrategy')}</div>`);
}
function strategyEditor(){
  return shell(`<div class="toolbar"><div><h1>MA Crossover · v12</h1><p>Agent-assisted strategy code · restricted sandbox</p></div><div class="toolbar-actions"><button class="btn" onclick="askAgentReview()">Ask agent to review</button><button class="btn primary" onclick="runBacktest()">Run backtest</button></div></div><div class="code-shell"><div class="files"><strong>MA Crossover</strong><div class="active">strategy.py</div><div>parameters.yaml</div><div>README.md</div><div>runs/</div><div>&nbsp;&nbsp;2026-09-04.json</div></div><textarea class="editor editable-code" spellcheck="false">class MovingAverageCrossover(Strategy):
    fast = 20
    slow = 50

    def on_bar(self, bar):
        if self.fast_ma > self.slow_ma:
            return Signal.buy(exposure=0.05)
        elif self.fast_ma < self.slow_ma:
            return Signal.sell()

# Network, keychain and broker gateway access are blocked.</textarea><div class="inspector"><h3>Strategy Inspector</h3><div class="section-label">Sandbox</div><span class="status ok">Restricted</span>${[['Network','Blocked'],['Keychain','Blocked'],['Order Gateway','Blocked'],['Historical data','Allowed']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}<div class="section-label">Parameters</div>${[['Fast MA','20'],['Slow MA','50'],['Exposure','5%'],['Slippage','5 bps']].map(r=>`<div class="field compact"><label>${r[0]}</label><input value="${r[1]}"/></div>`).join('')}<button class="btn full primary" onclick="runBacktest()">Run backtest</button><button class="btn full" style="margin-top:8px" onclick="failBacktest()">Simulate failure</button></div></div>`);
}
function strategyRunning(){
  return shell(`<div class="reconnect"><span class="status research">BACKTEST RUNNING</span><h2>MA Crossover · v12</h2><p class="muted">Historical data loaded. TradeX is simulating deterministic order fills locally.</p>${[['Load dataset','Complete'],['Validate calendar & adjustments','Complete'],['Execute strategy','Running'],['Compute metrics','Waiting'],['Persist manifest','Waiting']].map((r,i)=>`<div class="reconnect-row"><span>${r[0]}</span><b style="color:${i<2?'var(--green)':i===2?'var(--blue)':'var(--muted)'}">${r[1]}</b></div>`).join('')}<div class="modal-actions">${button('Cancel run','cancelBacktest()')}${button('Complete now','completeBacktest()','primary')}</div></div>`);
}
function strategyFailed(){
  return shell(`<div class="reconnect"><span class="status live">BACKTEST FAILED</span><h2>Dataset validation failed</h2><p class="muted">A corporate-action gap was detected in the requested historical period. No result artifact was produced.</p><div class="error-box"><b>DATA_QUALITY_ERROR</b><br/>Missing split adjustment metadata for one constituent.</div><div class="modal-actions">${button('Back to editor','backToEditor()')}${button('Retry with validated dataset','runBacktest()','primary')}</div></div>`);
}
function backtestResults(){
  return shell(`<div class="toolbar"><div><h1>Backtest Results</h1><p>MA Crossover · deterministic local run</p></div><div class="toolbar-actions"><span class="status ok">Reproducible</span><button class="btn" onclick="compareBacktests()">Compare v11 / v12</button></div></div><div class="grid4">${metric('Total return','+28.34%','vs +16.12% benchmark')}${metric('Sharpe','1.42','Risk adjusted')}${metric('Max drawdown','-18.21%','Worst peak-to-trough','negative')}${metric('Win rate','62.3%','24 trades')}</div><div class="split" style="margin-top:20px"><div class="backtest-chart">${chart('#2563eb','#dbeafe')}</div><div class="manifest"><h3>Reproducibility</h3>${[['Strategy','v12 · 83d8…'],['Dataset','sha256:a71c…'],['Provider','MarketData-X'],['Adjustment','Split/dividend'],['Timezone','America/New_York'],['Slippage','5 bps'],['Engine','tradex-bt 1.0']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}</div></div><div class="table-wrap" style="margin-top:20px"><div class="table-head"><h3>Recent trades</h3></div>${table(['Entry','Exit','Side','Return','Duration'],[['2026-08-12','2026-08-18','Long','+4.8%','6d'],['2026-07-29','2026-08-03','Long','-1.6%','5d']])}</div>`);
}
function strategyCompare(){
  return shell(`<div class="toolbar"><div><h1>Compare Backtest Runs</h1><p>MA Crossover · v11 vs v12</p></div><button class="btn" onclick="state.strategy='results';render()">Back to result</button></div><div class="compare-grid"><div class="card"><h3>v11</h3>${[['Return','23.1%'],['Sharpe','1.18'],['Max drawdown','-21.4%'],['Trades','28'],['Fast / Slow','15 / 50']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}</div><div class="card compare-highlight"><h3>v12</h3>${[['Return','28.34%'],['Sharpe','1.42'],['Max drawdown','-18.21%'],['Trades','24'],['Fast / Slow','20 / 50']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}</div></div><div class="card" style="margin-top:20px"><h3>Parameter diff</h3><div class="kv"><span>Fast moving average</span><span>15 → 20</span></div><div class="kv"><span>Other parameters</span><span>No change</span></div><p class="muted">TradeX does not infer that stronger backtest performance guarantees stronger live performance.</p></div>`);
}

function artifactsView(){
  if(state.artifact) return artifactDetail();
  return shell(`<div class="toolbar"><div><h1>Artifacts</h1><p>Research, datasets, backtests and trade reviews remain attached to local threads.</p></div><input class="search" style="width:330px" placeholder="Search artifacts…"/></div><div class="filter-row"><button class="pill research" style="border:0">All</button><button class="pill" style="border:0">Research</button><button class="pill" style="border:0">Backtests</button><button class="pill" style="border:0">Trade reviews</button></div><div class="table-wrap"><div class="table-head"><h3>Local artifacts</h3><small class="muted">6 items</small></div>${table(['Name','Type','Linked thread','Created','Size'],artifacts,'openArtifact')}</div>`);
}
function artifactDetail(){
  return shell(`<div class="toolbar"><div><h1>${esc(state.artifact)}</h1><p>Research artifact · local workspace</p></div><div class="toolbar-actions"><button class="btn" onclick="openProvenance()">Show provenance</button><button class="btn primary" onclick="exportArtifact()">Export</button></div></div><div class="settings-layout"><div class="card artifact-doc"><h2>NVDA vs AMD: post-earnings comparison</h2><p class="muted">Linked to “US tech earnings analysis” · created 2 hours ago</p><div class="section-label">Summary</div><p>NVDA retains the strongest earnings-revision profile and superior margin trajectory. AMD remains attractive on valuation, but current estimate momentum is weaker.</p><div class="section-label">Evidence</div>${['Company filing · NVDA · 2026-09-04','Company filing · AMD · 2026-09-04','Market data snapshot · realtime','Portfolio exposure · Trading 212 Live'].map(v=>`<div class="evidence-row">${v}</div>`).join('')}<div class="section-label">Decision context</div><p>No live order was executed from this artifact. A paper-order suggestion was generated separately.</p></div><div class="card"><h3>Artifact details</h3>${[['Format','Markdown'],['Size','2.4 MB'],['Sources','6'],['Thread','US tech earnings'],['Model',state.model],['Local path','artifacts/...']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}<button class="btn full" style="margin-top:14px" onclick="openThread('us-tech-earnings')">Open linked thread</button></div></div>`);
}

function settingsView(){
  if(state.settings==='providers') return providersSettings();
  if(state.settings==='risk') return riskSettings();
  return dataSettings();
}
function providersSettings(){
  return shell(`<div class="toolbar"><div><h1>Providers & Models</h1><p>Connection health, provider capabilities and pinned agent runtime.</p></div><button class="btn primary" onclick="openConnectAccount()">+ Add Provider</button></div><div class="card"><h3>Broker & data providers</h3>${providers.map(p=>`<div class="provider-row"><div><b>${p[0]}</b><small>${p[1]}</small></div><span>${fmtEnv(p[2])}</span><span class="status ${state.connectedProviders.has(p[0])?'ok':'neutral'}">${state.connectedProviders.has(p[0])?'Connected':'Not connected'}</span><button class="btn" onclick="configureProvider('${p[0]}')">Configure</button></div>`).join('')}</div><div class="card"><h3>Agent runtime</h3>${[['Codex App Server','Pinned runtime','Connected'],['Primary model',state.model,'Ready'],['Protocol schema','Generated against pinned version','Compatible']].map(r=>`<div class="provider-row"><b>${r[0]}</b><span>${r[1]}</span><span class="status ok">${r[2]}</span><button class="btn" onclick="openModelPicker()">Configure</button></div>`).join('')}</div>`);
}
function riskForm(onboarding=false){
  return `<div class="risk-form"><div class="field"><label>Maximum order notional (€)</label><input id="risk-maxOrderNotional" value="${state.risk.maxOrderNotional}"/></div><div class="field"><label>Maximum single-instrument exposure (%)</label><input id="risk-maxExposure" value="${state.risk.maxExposure}"/></div><div class="field"><label>Maximum daily traded notional (€)</label><input id="risk-maxDailyNotional" value="${state.risk.maxDailyNotional}"/></div><div class="field"><label>Maximum daily realized loss (€)</label><input id="risk-maxDailyLoss" value="${state.risk.maxDailyLoss}"/></div><div class="field"><label>Stale quote threshold (seconds)</label><input id="risk-staleQuote" value="${state.risk.staleQuote}"/></div><div class="field"><label>Market orders</label><select id="risk-marketOrders"><option ${state.risk.marketOrders==='Disabled'?'selected':''}>Disabled</option><option ${state.risk.marketOrders==='Enabled'?'selected':''}>Enabled</option></select></div><div class="field"><label>Live inactivity timeout (minutes)</label><input id="risk-inactivity" value="${state.risk.inactivity}"/></div></div>`;
}
function riskSettings(){
  return shell(`<div class="toolbar"><div><h1>Risk & Limits</h1><p>The agent cannot change these settings.</p></div></div><div class="settings-layout"><div class="card"><h3>Live trading guardrails</h3>${riskForm()}<div class="modal-actions"><button class="btn primary" onclick="saveRiskSettings()">Save limits</button></div></div><div class="card"><h3>Security behavior</h3><div class="bullet">Weakening a limit invalidates pending approvals.</div><div class="bullet">Pre-execution validation runs again after user approval.</div><div class="bullet">Concurrent orders reserve cash and exposure atomically.</div><div class="bullet">Market orders require a maximum approved notional.</div><div class="warning-box">Risk policy changes cannot be made through an agent tool call.</div></div></div>`);
}
function dataSettings(){
  return shell(`<div class="toolbar"><div><h1>Data & Storage</h1><p>Local-first persistence, retention and privacy boundaries.</p></div></div><div class="settings-layout"><div class="card"><h3>Local workspace storage</h3>${[['Workspace database','SQLite','284 MB'],['Analytics','DuckDB','418 MB'],['Historical bars','Parquet','6.2 GB'],['Artifacts','Filesystem','1.8 GB'],['Broker cache','Filesystem','192 MB']].map(r=>`<div class="storage-row"><span>${r[0]}</span><span>${r[1]}</span><b>${r[2]}</b></div>`).join('')}<div class="section-label">Retention</div>${[['Raw order book','Memory only'],['1m OHLCV','90 days'],['Daily bars','Keep'],['Artifacts','Keep'],['Execution audit','Keep']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}<div class="modal-actions"><button class="btn" onclick="exportWorkspace()">Export workspace</button><button class="btn primary" onclick="backupWorkspace()">Backup now</button></div></div><div class="card"><h3>Privacy boundaries</h3>${[['LOCAL ONLY','Workspace files'],['AGENT CONTEXT','Selected thread context'],['BROKER ONLY','Orders + account calls'],['SECRET','Never sent to model']].map((r,i)=>`<div class="privacy-item"><span class="status ${i===3?'live':i===1?'research':'ok'}">${r[0]}</span><p>${r[1]}</p></div>`).join('')}</div></div>`);
}

function recoveryView(){
  const type=state.recoveryState;
  if(type==='auth') return shell(`<div class="reconnect"><span class="status live">AUTH_ERROR</span><h2>${state.selectedAccount} authentication failed</h2><p class="muted">The provider rejected the saved credential. Live execution is disabled for this account.</p><div class="error-box">PERMISSION / AUTH ERROR · Secrets remain in the OS credential vault and are never shown to the model.</div><div class="modal-actions"><button class="btn" onclick="go('accountDetail')">Keep disconnected</button><button class="btn primary" onclick="openConnectAccount()">Reconnect</button></div></div>`);
  if(type==='stream') return shell(`<div class="reconnect"><span class="status warn">STREAM_DISCONNECTED</span><h2>Private account stream disconnected</h2><p class="muted">Fast order updates are degraded. TradeX is using REST reconciliation until the stream recovers.</p>${[['Private stream','Disconnected'],['REST reconciliation','Active'],['Open-order state','Fresh · 3 sec'],['New live orders','Disabled until healthy']].map(r=>`<div class="reconnect-row"><span>${r[0]}</span><b>${r[1]}</b></div>`).join('')}<div class="modal-actions"><button class="btn" onclick="go('accountDetail')">Back</button><button class="btn primary" onclick="showToast('Stream reconnected; account healthy')">Reconnect stream</button></div></div>`);
  if(type==='rate') return shell(`<div class="reconnect"><span class="status warn">RATE_LIMITED</span><h2>Provider request budget is temporarily exhausted</h2><p class="muted">TradeX has paused low-priority research requests. Execution reconciliation remains highest priority.</p>${[['P0','Execution reconciliation','Active'],['P1','Account refresh','Queued'],['P2','Market monitoring','Reduced'],['P3','Historical research','Paused']].map((r,i)=>`<div class="priority-grid"><b>${r[0]}</b><span>${r[1]}</span><b style="color:${['var(--green)','var(--blue)','var(--amber)','var(--red)'][i]}">${r[2]}</b></div>`).join('')}<div class="warning-box">Error category: RATE_LIMITED · Retry strategy: backoff + provider reset window</div><button class="btn full" style="margin-top:16px" onclick="go('accountDetail')">Back to account</button></div>`);
  if(type==='startup') return shell(`<div class="reconnect"><span class="status research">STARTUP RECONCILIATION</span><h2>Restoring live account state</h2><p class="muted">TradeX is rebuilding its local projection from authoritative broker state. Live execution remains disabled.</p>${[['Load local projection','Complete'],['Connect Trading 212','Complete'],['Reconcile 2 open orders','Running'],['Refresh positions','Waiting'],['Validate reservations','Waiting']].map((r,i)=>`<div class="reconnect-row"><span>${r[0]}</span><b style="color:${i<2?'var(--green)':i===2?'var(--blue)':'var(--muted)'}">${r[1]}</b></div>`).join('')}<div class="warning-box">No live order can be submitted until reconciliation succeeds.</div><div class="modal-actions"><button class="btn primary" onclick="showToast('Reconciliation complete; account healthy')">Complete reconciliation</button></div></div>`);
  return shell(`<div class="reconnect"><span class="status warn">LIVE DISABLED</span><h2>Refreshing account state after system resume</h2><p class="muted">TradeX detected that the device resumed from sleep. Live submissions remain disabled until streams, server time, balances and open orders are reconciled.</p>${[['Reconnect market streams','Complete'],['Synchronize provider time','Complete'],['Refresh balances','Complete'],['Reconcile open orders','Running'],['Validate risk reservations','Waiting']].map((r,i)=>`<div class="reconnect-row"><span>${r[0]}</span><b style="color:${i<3?'var(--green)':i===3?'var(--blue)':'var(--muted)'}">${r[1]}</b></div>`).join('')}<div class="modal-actions"><button class="btn" onclick="go('accountDetail')">Keep Live Disabled</button><button class="btn primary" onclick="showRecovery('rate')">Simulate provider issue</button></div></div>`);
}

function orderMonitor(){
  const o=state.currentOrder||defaultAaplOrder();
  const isBtc=o.instrument==='BTC/USDT';
  const status=state.orderState||'FILLED';
  let events=[];
  if(status==='BROKER_REJECTED') events=[['14:42:18.021','Order approved','Exact proposal approved by user'],['14:42:18.109','Pre-execution check','Risk and quote freshness passed'],['14:42:18.221','Submitted',`Request sent to ${o.provider}`],['14:42:18.364','Rejected','Broker rejected order: insufficient available buying power after external account change']];
  else if(status==='SUBMITTING') events=[['14:42:18.021','Order approved','Exact proposal approved by user'],['14:42:18.109','Pre-execution check','Risk and quote freshness passed'],['14:42:18.221','Submitted',`Request sent to ${o.provider}`],['14:42:18.720','Waiting','Broker acknowledgement pending']];
  else events=[['14:42:18.021','Order approved','Exact proposal approved by user'],['14:42:18.109','Pre-execution check','Risk and quote freshness passed'],['14:42:18.221','Submitted',`Request sent to ${o.provider}`],['14:42:18.364','Accepted','Broker acknowledged order'],['14:42:19.041','Partially filled',isBtc?'0.006 BTC @ 62,410':'1 AAPL @ $221.48'],['14:42:19.612','Filled',isBtc?'0.004 BTC @ 62,416':'1 AAPL @ $221.50'],['14:42:19.820','Reconciled','Broker + local projection converged']];
  const statusClass=status==='BROKER_REJECTED'?'live':status==='SUBMITTING'?'warn':'ok';
  return shell(`<div class="toolbar"><div><h1>Order activity</h1><p>Broker acknowledgement is not treated as a fill.</p></div><span class="status ${statusClass}">${status}</span></div><div class="split"><div class="card event-timeline">${events.map(r=>`<div class="event"><span class="time">${r[0]}</span><span class="circle"></span><div><div class="event-title">${r[1]}</div><div class="event-desc">${r[2]}</div></div></div>`).join('')}</div><div><div class="card"><span class="status ${statusClass}">${status}</span><h2>${o.side} ${o.quantity} ${o.instrument}</h2>${[['Order',o.orderType+' · '+o.price],['Account',o.account],['Broker ID',status==='SUBMITTING'?'Pending':'8942…312'],['TradeX ID','txo_01…8d']].map(r=>`<div class="kv"><span>${r[0]}</span><span>${r[1]}</span></div>`).join('')}<div class="action-stack"><button class="btn full" onclick="showAmbiguous()">View ambiguous submission</button><button class="btn full" onclick="simulateBrokerReject()">Simulate broker rejection</button>${status==='FILLED'?'<button class="btn full" onclick="requestCancelFilledDemo()">Show cancel-order flow</button>':''}</div></div><div class="card"><h3>Audit chain</h3>${['User intent','Research evidence','Order proposal','Risk evaluation','User approval','Pre-execution revalidation','Execution attempt','Broker state'].map(v=>`<div class="bullet">${v}</div>`).join('')}</div></div></div>`);
}
function defaultAaplOrder(){return {provider:'Trading 212',account:'Trading 212 Live',instrument:'AAPL',side:'BUY',quantity:'2',orderType:'LIMIT',price:'$221.50',notional:'$443.00',quote:'$221.42',quoteAge:'420 ms'}}
function btcOrder(){return {provider:'Binance',account:'Binance Live',instrument:'BTC/USDT',side:'BUY',quantity:'0.01 BTC',orderType:'MARKET',price:'Expected €620 · max €630',notional:'€620 expected',quote:'62,418.20',quoteAge:'280 ms'}}

function modal(){
  if(!state.modal) return '';
  if(state.modal==='contextPicker') return modalShell('Attach context','Select context objects that may be included in the agent turn.',`<div class="picker-groups"><h4>Instruments</h4>${['AAPL','NVDA','BTC/USDT'].map(x=>checkRow(x,x==='AAPL')).join('')}<h4>Accounts</h4>${['Trading 212 Live','Binance Live'].map(x=>checkRow(x,x==='Trading 212 Live')).join('')}<h4>Strategies / Artifacts</h4>${['MA Crossover v12','NVDA vs AMD analysis'].map(x=>checkRow(x,false)).join('')}</div>`,`${button('Cancel','closeModal()')}${button('Attach selected','attachContext()','primary')}`);
  if(state.modal==='accountPicker') return modalShell('Choose account','Account context is read-only unless the current mode permits paper/live proposals.',`<div class="selection-list">${accounts.map(a=>`<button class="selection-row ${state.account===a.name?'selected':''}" onclick="chooseAccount('${a.name}')"><div><strong>${a.name}</strong><small>${a.provider} · ${a.env}</small></div>${fmtEnv(a.env)}</button>`).join('')}</div>`,'');
  if(state.modal==='modelPicker') return modalShell('Choose model','Model choice affects agent reasoning only; it does not change trading authority.',`<div class="selection-list">${['GPT-5.6 Sol','GPT-5.6 Luna','Local model (future)'].map(m=>`<button class="selection-row ${state.model===m?'selected':''}" onclick="selectModelAndClose('${m}')"><div><strong>${m}</strong><small>${m===state.model?'Current model':'Available option'}</small></div><span>${m===state.model?'✓':''}</span></button>`).join('')}</div>`,'');
  if(state.modal==='connectProvider') return connectProviderModal();
  if(state.modal==='connectSuccess') return modalShell('Provider connected','Credential was validated and stored in the OS credential vault.',`<div class="risk-box"><b class="positive">Connection test passed</b><div class="risk-row"><span>Account read</span><span>✓ Allowed</span></div><div class="risk-row"><span>Positions read</span><span>✓ Allowed</span></div><div class="risk-row"><span>Order read</span><span>✓ Allowed</span></div><div class="risk-row"><span>Place/cancel order</span><span>✓ Allowed</span></div></div><div class="warning-box">Withdrawal and transfer capabilities are not requested or implemented.</div>`,button('Done','closeModal()','primary'));
  if(state.modal==='armLive') return armLiveModal();
  if(state.modal==='liveApproval') return liveApprovalModal(state.pendingOrder||defaultAaplOrder());
  if(state.modal==='marketApproval') return marketApprovalModal();
  if(state.modal==='approvalInvalid') return invalidApprovalModal();
  if(state.modal==='approvalExpired') return modalShell('Approval expired','The one-time approval window closed before execution.',`<div class="order-summary"><h3>${state.pendingOrder?.side||'BUY'} ${state.pendingOrder?.quantity||'2'} ${state.pendingOrder?.instrument||'AAPL'}</h3><p class="muted">No broker request was sent.</p></div><div class="warning-box">Generate and approve a fresh proposal before any live submission.</div>`,`${button('Cancel','closeModal()')}${button('Refresh proposal','refreshProposal()','primary')}`);
  if(state.modal==='riskRejected') return modalShell('Order blocked by risk policy','The deterministic Risk Engine rejected the proposal before user approval.',`<div class="error-box"><b>RISK_REJECTED</b><br/>Projected AAPL concentration: 12.4% · Limit: 10.0%</div><div class="kv"><span>Agent action</span><span>Cannot override</span></div><div class="kv"><span>Live order submitted</span><span>No</span></div>`,button('Close','closeModal()'));
  if(state.modal==='paper') return paperModal();
  if(state.modal==='paperSuccess') return modalShell('Paper order completed','Alpaca Paper reported the simulated fill.',`<span class="status ok">PAPER FILLED</span><div class="warning-box">Paper results are not equivalent to expected live execution, market impact or queue priority.</div>`,button('Done','closeModal()','primary'));
  if(state.modal==='ambiguous') return ambiguousModal();
  if(state.modal==='cancelApproval') return cancelApprovalModal();
  if(state.modal==='cancelPending') return modalShell('Cancellation pending','The cancel request was accepted locally and is waiting for broker confirmation.',`<span class="status warn">CANCEL_PENDING</span><div class="kv"><span>Order</span><span>${state.selectedInstrument==='BTC/USDT'?'BTC/USDT':'AAPL'} remaining quantity</span></div><div class="warning-box">The order may still fill until the broker confirms cancellation.</div>`,button('Confirm broker cancellation','confirmCancelled()','primary'));
  if(state.modal==='cancelled') return modalShell('Order cancelled','Broker state confirms the remaining quantity is cancelled.',`<span class="status ok">CANCELLED</span><div class="kv"><span>Broker state</span><span>Confirmed</span></div><div class="kv"><span>Local projection</span><span>Reconciled</span></div>`,button('Done','closeModal()','primary'));
  if(state.modal==='screenerBuilder') return screenerBuilderModal();
  if(state.modal==='newWatchlist') return modalShell('Create watchlist','Create a local watchlist and warm data subscription group.',`<div class="field"><label>Name</label><input id="newWatchName" value="AI Infrastructure"/></div><div class="field"><label>Market</label><select><option>US Stocks</option><option>Crypto</option></select></div>`,`${button('Cancel','closeModal()')}${button('Create watchlist','createWatchlist()','primary')}`);
  if(state.modal==='addWatchlist') return modalShell('Add AAPL to watchlist','Choose a destination watchlist.',`<div class="selection-list">${['Tech Giants','AI Semiconductors','Earnings Watch','Long-term'].map(x=>`<button class="selection-row" onclick="addToWatchlist('${x}')"><strong>${x}</strong><span>›</span></button>`).join('')}</div>`,'');
  if(state.modal==='providerConfig') return connectProviderModal(true);
  if(state.modal==='provenance') return provenanceModal();
  if(state.modal==='success') return modalShell('Completed',state.toast||'Action completed.',`<span class="status ok">SUCCESS</span>`,button('Done','closeModal()','primary'));
  return '';
}
function modalShell(title,sub,body,actions=''){return `<div class="modal-layer"><div class="modal"><h2>${title}</h2>${sub?`<p class="muted">${sub}</p>`:''}${body}${actions?`<div class="modal-actions">${actions}</div>`:''}</div></div>`}
function checkRow(label,checked){return `<label class="check-row"><input type="checkbox" ${checked?'checked':''}/><span>${label}</span></label>`}
function connectProviderModal(config=false){
  const name=state.pendingProvider||'Trading 212 Live';
  const live=name.includes('Live');
  const crypto=name.includes('Binance')||name.includes('Bitget');
  return modalShell(config?`Configure ${name}`:`Connect ${name}`,live?'This is a live account connection. TradeX will verify permissions but keep execution disarmed.':'Credentials are stored only in the OS credential vault.',`<div class="field"><label>Environment</label><input value="${live?'Live':name.includes('Testnet')?'Testnet':name.includes('Demo')?'Demo':'Paper'}" disabled/></div><div class="field"><label>API Key</label><input value="••••••••••••••••"/></div>${crypto?'<div class="field"><label>API Secret / Passphrase</label><input value="••••••••••••••••"/></div>':''}<div class="risk-box"><b>Requested permissions</b>${['Account read','Positions read','Order read','Place order','Cancel order'].map(p=>`<div class="risk-row"><span>${p}</span><span>✓ Required</span></div>`).join('')}</div><div class="warning-box">TradeX does not request withdrawal, transfer, custody, margin borrowing or leverage-management permissions.</div>`,`${button('Cancel','closeModal()')}${button('Test connection','testConnection()','primary')}`);
}
function armLiveModal(){
  const p=state.pendingOrder;
  return modalShell('Arm Live Trading','Live execution is currently DISARMED. Arming is an explicit user action and does not approve any order.',`<div class="order-summary"><div class="kv"><span>Account</span><span>${p?.account||state.selectedAccount}</span></div><div class="kv"><span>Risk policy</span><span>Conservative defaults</span></div><div class="kv"><span>Session timeout</span><span>${state.risk.inactivity} min</span></div></div><div class="warning-box">After arming, every live order and live cancellation still requires a separate transaction-specific approval.</div>`,`${button('Cancel','cancelPendingOrder()')}${button('Arm Live Trading','confirmArmLive()','danger')}`);
}
function liveApprovalModal(o){
  return modalShell('Approve live order','Review the exact immutable proposal. Approval is single-use and expires shortly.',`<span class="status live">LIVE</span><div class="order-summary"><h3>${o.side} ${o.quantity} ${o.instrument}</h3><p class="muted">${o.orderType} · ${o.price}</p><div class="order-grid"><div><label>Estimated value</label><strong>${o.notional}</strong></div><div><label>Account</label><strong>${o.account}</strong></div><div><label>Quote</label><strong>${o.quote} · ${o.quoteAge}</strong></div></div></div><div class="risk-box"><b class="positive">Risk checks passed</b>${[['Account reconciled','Healthy'],['Instrument tradable','Yes'],['Projected concentration','8.4% < 10%'],['Daily traded notional','€1,264 < €5,000'],['Quote freshness',o.quoteAge+' < 3 sec']].map(r=>`<div class="risk-row"><span>${r[0]}</span><span>✓ ${r[1]}</span></div>`).join('')}</div><div class="warning-box">Approval expires in 00:42. Any material proposal change invalidates this approval.</div>`,`${button('Simulate stale quote','invalidateApproval()')}${button('Simulate expiry','expireApproval()')}${button('Reject','cancelPendingOrder()')}${button('Approve & Place','approveLive()','danger')}`);
}
function marketApprovalModal(){
  const o=state.pendingOrder||btcOrder();
  return modalShell('Approve market order','Market orders require an explicit maximum approved spend.',`<span class="status live">BINANCE LIVE</span><div class="order-summary"><h3>BUY BTC MARKET</h3><div class="order-grid"><div><label>Expected spend</label><strong>€620</strong></div><div><label>Maximum authorized</label><strong>€630</strong></div><div><label>Best ask</label><strong>62,418.20</strong></div><div><label>Spread</label><strong>1.8 bps</strong></div><div><label>Quote age</label><strong>280 ms</strong></div><div><label>Estimated fee</label><strong>€0.62</strong></div></div></div><div class="risk-box"><b class="positive">Pre-approval checks passed</b><div class="risk-row"><span>Available USDT</span><span>✓ Sufficient</span></div><div class="risk-row"><span>Max order notional</span><span>✓ Within limit</span></div><div class="risk-row"><span>Market orders policy</span><span>✓ Explicitly allowed for this proposal</span></div></div><div class="warning-box">The gateway will reject execution if required spend exceeds €630.</div>`,`${button('Reject','cancelPendingOrder()')}${button('Approve up to €630','approveLive()','danger')}`);
}
function invalidApprovalModal(){
  const o=state.pendingOrder||defaultAaplOrder();
  return modalShell('Approval is no longer valid','The market moved beyond the configured freshness / deviation threshold after approval.',`<span class="status warn">REFRESH REQUIRED</span><div class="split"><div class="order-summary"><b>Approved proposal</b><h3>${o.side} ${o.quantity} ${o.instrument} · ${o.price}</h3><div class="kv"><span>Quote at approval</span><span>${o.quote}</span></div><div class="kv"><span>Age</span><span class="negative">4.8 sec</span></div></div><div class="order-summary" style="background:var(--blue2)"><b style="color:var(--blue)">Refreshed market</b><h3>${o.instrument} · ${o.instrument==='AAPL'?'$223.18':'62,910'}</h3><div class="kv"><span>Change</span><span class="negative">+0.79%</span></div><div class="kv"><span>Approval</span><span>Invalid</span></div></div></div><div class="warning-box">TradeX will not silently update and submit the order. A refreshed proposal requires a new approval.</div>`,`${button('Cancel','cancelPendingOrder()')}${button('Refresh proposal','refreshProposal()','primary')}`);
}
function paperModal(){return modalShell('Place paper order','Alpaca Paper · simulated execution environment',`<span class="status ok">PAPER</span><div class="order-summary"><h3>BUY 3 AAPL</h3><p class="muted">Limit · $220.80 · DAY</p><div class="order-grid"><div><label>Estimated notional</label><strong>$662.40</strong></div><div><label>Available paper cash</label><strong>$48,210</strong></div><div><label>Projected exposure</label><strong>5.1%</strong></div></div></div>`,`${button('Cancel','closeModal()')}${button('Place Paper Order','paperSuccess()','primary')}`)}
function ambiguousModal(){const o=state.currentOrder||defaultAaplOrder();return modalShell('TradeX cannot confirm whether the broker received the order.','The submission timed out after leaving TradeX. TradeX will not blindly send it again.',`<span class="status warn">UNKNOWN · RECONCILING</span><div class="warning-box">Do not place another equivalent order manually until reconciliation completes.</div><div class="order-grid"><div><label>Provider</label><strong>${o.provider}</strong></div><div><label>Local state</label><strong>SUBMITTING</strong></div><div><label>Current state</label><strong>UNKNOWN_RECONCILING</strong></div><div><label>Automatic retry</label><strong>Blocked</strong></div></div>`,`${button('Open broker','showToast(\'Broker opened in external app\')')}${button('Continue reconciling','closeModal()','primary')}`)}
function cancelApprovalModal(){return modalShell('Approve live cancellation','Cancellation is a live account action and requires transaction-specific approval.',`<span class="status live">LIVE CANCEL</span><div class="order-summary"><h3>Cancel remaining ${state.selectedInstrument==='BTC/USDT'?'0.003 BTC':'1 AAPL'}</h3><p class="muted">${state.selectedAccount}</p><div class="kv"><span>Current order state</span><span>PARTIALLY_FILLED</span></div><div class="kv"><span>Broker state fresh</span><span>2 sec ago</span></div></div><div class="warning-box">The order may fill before the broker confirms cancellation.</div>`,`${button('Reject','closeModal()')}${button('Approve Cancellation','submitCancel()','danger')}`)}
function screenerBuilderModal(){return modalShell('New natural-language screener','TradeX converts intent into a visible structured FilterSpec before execution.',`<div class="field"><label>Describe what you want to find</label><textarea class="large-input">US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.</textarea></div><div class="filter-spec"><b>Parsed FilterSpec</b><div class="kv"><span>Universe</span><span>US Large Cap Technology</span></div><div class="kv"><span>Revenue growth</span><span>&gt; 15%</span></div><div class="kv"><span>Estimate revision</span><span>&gt; 0</span></div><div class="kv"><span>RSI</span><span>&lt; 70</span></div><div class="kv"><span>Rank</span><span>Revision strength + quality</span></div></div>`,`${button('Cancel','closeModal()')}${button('Run screener','runScreener()','primary')}`)}
function provenanceModal(){return modalShell('Artifact provenance','Inspect how this artifact was produced.',`<div class="provenance"><div class="kv"><span>Thread</span><span>US tech earnings analysis</span></div><div class="kv"><span>Created by</span><span>Agent Turn #4</span></div><div class="kv"><span>Model</span><span>${state.model}</span></div><div class="kv"><span>Sources</span><span>6</span></div><div class="section-label">Tool calls</div>${['market.earnings','research.news_search','portfolio.exposure'].map(x=>`<div class="evidence-row">${x}</div>`).join('')}<div class="section-label">Dataset hashes</div><div class="evidence-row">market_snapshot sha256:8ad1…</div><div class="section-label">Related order</div><div class="evidence-row">None · research only</div></div>`,button('Close','closeModal()'))}

function render(){
  const app=document.getElementById('app');let html='';
  switch(state.view){
    case'onboarding':html=onboarding();break;case'thread':html=thread();break;case'markets':html=markets();break;case'instrument':html=instrument();break;case'watchlists':html=watchlists();break;case'accounts':html=accountsView();break;case'portfolio':html=portfolio();break;case'accountDetail':html=accountDetail();break;case'strategies':html=strategiesView();break;case'artifacts':html=artifactsView();break;case'settings':html=settingsView();break;case'recovery':html=recoveryView();break;case'orderMonitor':html=orderMonitor();break;default:html=thread();
  }
  app.innerHTML=html;
}

// navigation / onboarding
window.go=v=>{state.view=v;state.modal=null;render()};
window.goSettings=v=>{state.view='settings';state.settings=v;state.modal=null;render()};
window.nextOnboarding=()=>{captureRiskInputs();state.onboardingStep=Math.min(5,state.onboardingStep+1);render()};
window.prevOnboarding=()=>{state.onboardingStep=Math.max(1,state.onboardingStep-1);render()};
window.finishOnboarding=()=>{state.view='thread';state.threadStatus='empty';state.modal=null;render()};
window.selectModel=m=>{state.model=m;render()};
window.selectModelAndClose=m=>{state.model=m;state.modal=null;render()};
window.newThread=()=>{state.view='thread';state.threadStatus='empty';state.threadId='us-tech-earnings';state.mode='Research';state.account='No account';state.selectedInstrument='AAPL';render()};
window.openThread=id=>{state.threadId=id;state.view='thread';state.threadStatus=id==='btc-liquidity'?'result':'result';state.account=id==='btc-liquidity'?'Binance Live':'Trading 212 Live';state.selectedInstrument=id==='btc-liquidity'?'BTC/USDT':'AAPL';render()};

// agent
window.startResearch=()=>{state.threadId='us-tech-earnings';state.threadStatus='running';state.account='Trading 212 Live';state.selectedInstrument='AAPL';render();setTimeout(()=>{if(state.threadStatus==='running'){state.threadStatus='result';render()}},1800)};
window.startCryptoResearch=()=>{state.threadId='btc-liquidity';state.threadStatus='result';state.account='Binance Live';state.selectedInstrument='BTC/USDT';render()};
window.simulateToolFailure=()=>{state.threadStatus='toolError';render()};
window.retryTool=()=>{state.threadStatus='running';render();setTimeout(()=>{state.threadStatus='result';render()},1000)};
window.cancelTurn=()=>{state.threadStatus='empty';render()};
window.portfolioReview=()=>go('portfolio');
window.openStrategy=()=>{state.view='strategies';state.strategy='editor';render()};
window.cycleMode=()=>{const modes=['Research','Backtest','Paper','Live'];state.mode=modes[(modes.indexOf(state.mode)+1)%modes.length];if(state.mode==='Live')state.account='Trading 212 Live';else if(state.mode==='Paper')state.account='Alpaca Paper';render()};
window.sendPrompt=()=>{const p=document.getElementById('prompt');if(p&&p.value.trim())startResearch()};
window.openContextPicker=()=>{state.modal='contextPicker';render()};window.attachContext=()=>{state.modal=null;showToast('Context attached: AAPL + Trading 212 Live');render()};
window.openAccountPicker=()=>{state.modal='accountPicker';render()};window.chooseAccount=n=>{state.account=n;state.modal=null;render()};
window.openModelPicker=()=>{state.modal='modelPicker';render()};

// markets / watchlists
window.openInstrument=s=>{state.selectedInstrument=String(s).includes('BTC')?'BTC/USDT':'AAPL';state.view='instrument';state.instrumentTab='overview';render()};
window.openCryptoInstrument=()=>{state.selectedInstrument='BTC/USDT';state.view='instrument';render()};
window.instrumentTab=t=>{state.instrumentTab=t;render()};
window.openScreenerBuilder=()=>{state.modal='screenerBuilder';render()};window.runScreener=()=>{state.screenerResult=true;state.modal=null;showToast('Screener completed: 5 candidates');render()};
window.setWatchlist=v=>{state.watchlist=v;render()};
window.openNewWatchlist=()=>{state.modal='newWatchlist';render()};window.createWatchlist=()=>{const v=document.getElementById('newWatchName')?.value||'New Watchlist';state.watchlist=v;state.modal=null;showToast('Watchlist created: '+v);render()};
window.openAddWatchlist=()=>{state.modal='addWatchlist';render()};window.addToWatchlist=v=>{state.watchlist=v;state.modal=null;showToast('AAPL added to '+v);render()};

// providers / accounts
window.openConnectAccount=()=>{state.pendingProvider='Trading 212 Live';state.modal='connectProvider';render()};
window.connectProvider=n=>{state.pendingProvider=n;state.modal='connectProvider';render()};
window.configureProvider=n=>{state.pendingProvider=n;state.modal='providerConfig';render()};
window.testConnection=()=>{state.connectedProviders.add(state.pendingProvider||'Trading 212 Live');state.modal='connectSuccess';render()};
window.openAccount=n=>{state.selectedAccount=n;state.selectedInstrument=(n.includes('Binance')||n.includes('Bitget'))?'BTC/USDT':'AAPL';state.view='accountDetail';render()};

// live trading safety
window.openArmLive=()=>{state.pendingOrder=null;state.modal='armLive';render()};
window.requestLiveOrder=type=>{state.pendingOrder=type==='btcMarket'?btcOrder():defaultAaplOrder();if(!state.liveArmed){state.modal='armLive'}else{state.modal=type==='btcMarket'?'marketApproval':'liveApproval'}render()};
window.openMarketOrder=()=>requestLiveOrder('btcMarket');
window.confirmArmLive=()=>{state.liveArmed=true;const p=state.pendingOrder;if(p)state.modal=p.instrument==='BTC/USDT'?'marketApproval':'liveApproval';else state.modal=null;render()};
window.disarmLive=()=>{state.liveArmed=false;state.modal=null;showToast('Live execution disabled');render()};
window.cancelPendingOrder=()=>{state.pendingOrder=null;state.modal=null;render()};
window.invalidateApproval=()=>{state.modal='approvalInvalid';render()};
window.expireApproval=()=>{state.modal='approvalExpired';render()};
window.refreshProposal=()=>{state.modal=(state.pendingOrder?.instrument==='BTC/USDT')?'marketApproval':'liveApproval';render()};
window.openRiskRejected=()=>{state.modal='riskRejected';render()};
window.approveLive=()=>{state.currentOrder={...(state.pendingOrder||defaultAaplOrder())};state.pendingOrder=null;state.modal=null;state.orderState='SUBMITTING';state.view='orderMonitor';render();setTimeout(()=>{if(state.view==='orderMonitor'&&state.orderState==='SUBMITTING'){state.orderState='FILLED';render()}},1400)};
window.showAmbiguous=()=>{state.modal='ambiguous';render()};
window.simulateBrokerReject=()=>{state.orderState='BROKER_REJECTED';render()};

// paper / cancel
window.openPaperModal=()=>{state.modal='paper';render()};window.paperSuccess=()=>{state.modal='paperSuccess';render()};
window.requestCancel=()=>{if(!state.liveArmed){state.pendingOrder={account:state.selectedAccount,instrument:state.selectedInstrument,operation:'CANCEL'};state.modal='armLive'}else state.modal='cancelApproval';render()};
window.requestCancelFilledDemo=()=>{state.selectedAccount=state.currentOrder?.account||'Trading 212 Live';state.selectedInstrument=state.currentOrder?.instrument||'AAPL';state.modal='cancelApproval';render()};
window.submitCancel=()=>{state.modal='cancelPending';render()};window.confirmCancelled=()=>{state.modal='cancelled';render()};

// strategy
window.selectStrategy=()=>{state.strategy='editor';render()};window.newStrategy=()=>{state.strategy='editor';showToast('New local strategy draft created');render()};
window.runBacktest=()=>{state.strategy='running';render();setTimeout(()=>{if(state.strategy==='running'){state.strategy='results';render()}},1500)};window.completeBacktest=()=>{state.strategy='results';render()};window.cancelBacktest=()=>{state.strategy='editor';render()};window.failBacktest=()=>{state.strategy='failed';render()};window.backToEditor=()=>{state.strategy='editor';render()};window.compareBacktests=()=>{state.strategy='compare';render()};window.askAgentReview=()=>showToast('Agent review attached to strategy thread');

// artifacts / settings / recovery
window.openArtifact=v=>{state.artifact=v;render()};window.openProvenance=()=>{state.modal='provenance';render()};window.exportArtifact=()=>{state.modal='success';state.toast='Artifact exported to local workspace exports folder';render()};
window.exportWorkspace=()=>{state.modal='success';state.toast='Workspace export created locally';render()};window.backupWorkspace=()=>{state.modal='success';state.toast='Local backup completed';render()};
window.saveRiskSettings=()=>{captureRiskInputs();state.liveArmed=false;showToast('Risk limits saved; live session remains/disarms for safety');render()};
window.captureRiskInputs=captureRiskInputs;
window.showRecovery=t=>{state.recoveryState=t;state.view='recovery';state.liveArmed=false;render()};
window.closeModal=()=>{state.modal=null;render()};
window.showToast=showToast;

function captureRiskInputs(){
  const map=['maxOrderNotional','maxExposure','maxDailyNotional','maxDailyLoss','staleQuote','inactivity'];
  map.forEach(k=>{const e=document.getElementById('risk-'+k);if(e)state.risk[k]=e.value});const m=document.getElementById('risk-marketOrders');if(m)state.risk.marketOrders=m.value;
}
function showToast(msg){state.toast=msg;render();setTimeout(()=>{if(state.toast===msg){state.toast=null;render()}},1700)}

render();
