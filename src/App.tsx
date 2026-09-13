import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery } from '@tanstack/react-query';
import { open } from '@tauri-apps/plugin-dialog';
import type { AgentMode, ExecutionContext, ModelRoute, ModelState, RiskPolicyState, RuntimeStatus, Thread, ThreadCreate, ThreadModel } from '../shared/ipc-types.ts';
import { browserIntegration, desktop, explainError, request, transportAvailable } from './client.ts';
import { Accounts } from './Accounts.tsx';
import { Models } from './Models.tsx';
import { RiskDefaults, draftFromPolicy, type RiskDraft } from './RiskDefaults.tsx';
import { fromModelSnapshot, fromRiskSnapshot, fromThreadSnapshot } from './projection.ts';
import { useWorkspace } from './useWorkspace.ts';
import { useDomainProjection } from './useDomainProjection.ts';
import type { OpenWorkspace, Workspace } from '../shared/ipc-types.ts';

const pages = ['New Thread', 'Threads', 'Markets', 'Watchlists', 'Accounts', 'Strategies', 'Artifacts', 'Settings'] as const;
type Page = typeof pages[number];

function Navigation({ page, navigate }: { page: Page; navigate: (page: Page) => void }) {
  return <nav aria-label="Primary navigation">{pages.map(destination =>
    <button key={destination} className={destination === 'New Thread' ? 'new-thread' : ''}
      aria-current={destination === page ? 'page' : undefined} onClick={() => navigate(destination)}>
      {destination === 'New Thread' ? '+ New Thread' : destination}
    </button>)}</nav>;
}

function WorkspaceSetup({ busy, submit }: { busy: boolean; submit: (options: OpenWorkspace) => Promise<void> }) {
  const [name, setName] = useState('My Trading Workspace');
  const [currency, setCurrency] = useState('USD');
  const [path, setPath] = useState('');
  const [pickerError, setPickerError] = useState(false);
  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    void submit({ name: name.trim(), baseCurrency: currency, ...(path.trim() ? { path: path.trim() } : {}) });
  };
  return <section className="setup" aria-labelledby="setup-title">
    <ol className="steps" aria-label="Workspace setup progress">{['Workspace', 'Providers', 'Model', 'Risk defaults', 'Ready'].map((step, index) =>
      <li key={step} aria-current={index === 0 ? 'step' : undefined}><span>{index + 1}</span>{step}</li>)}</ol>
    <div className="card setup-form">
      <h1 id="setup-title">Create local workspace</h1>
      <p className="muted">Your research, strategies and activity stay in your local workspace.</p>
      <form onSubmit={handleSubmit}>
        <label className="field">Workspace name
          <input value={name} onChange={event => setName(event.target.value)} required maxLength={120} autoComplete="off" />
        </label>
        <label className="field">Base currency
          <select value={currency} onChange={event => setCurrency(event.target.value)}>
            <option value="USD">USD — US Dollar</option><option value="EUR">EUR — Euro</option><option value="GBP">GBP — British Pound</option>
            <option value="CAD">CAD — Canadian Dollar</option><option value="AUD">AUD — Australian Dollar</option><option value="CHF">CHF — Swiss Franc</option><option value="JPY">JPY — Japanese Yen</option>
          </select>
        </label>
        <div className="field"><label htmlFor="workspace-path">Local storage</label>
          <div className="folder-input"><input id="workspace-path" value={path} onChange={event => setPath(event.target.value)} placeholder="Use the default local workspace folder" autoComplete="off" />
            <button type="button" disabled={!desktop || busy} onClick={async () => {
              try { const selected = await open({ directory: true, multiple: false, title: 'Choose TradeX workspace' }); if (typeof selected === 'string') setPath(selected); }
              catch { setPickerError(true); }
            }}>Browse…</button></div>
        </div>
        <p className="form-hint">Opening an existing folder restores its saved name and base currency.</p>
        {pickerError && <p role="alert">The folder picker is unavailable. Enter an absolute local folder path.</p>}
        <div className="form-actions"><button className="primary" disabled={busy || !transportAvailable || !name.trim()}>
          {busy ? 'Opening workspace…' : 'Open workspace'}
        </button></div>
      </form>
      {!transportAvailable && <p className="notice">Open TradeX in the desktop app to use a local workspace.</p>}
    </div>
    <p className="setup-note">Live trading requires separate account setup, risk limits and explicit approval.</p>
  </section>;
}

function WorkspaceDetails({ workspace }: { workspace: Workspace }) {
  return <aside className="context" aria-labelledby="workspace-details">
    <h2 id="workspace-details">Workspace</h2><p className="muted">{workspace.name}</p>
    <dl><div><dt>Base currency</dt><dd>{workspace.baseCurrency}</dd></div>
      <div><dt>Storage</dt><dd className="path">{workspace.path}</dd></div>
      <div><dt>Workspace ID</dt><dd className="identity">{workspace.workspaceId}</dd></div>
      <div><dt>Created</dt><dd><time dateTime={workspace.createdAt}>{new Date(workspace.createdAt).toLocaleString()}</time></dd></div>
    </dl>
    <div className="notice"><strong>No live execution</strong><p>Connect an account and configure its risk policy before requesting live access.</p></div>
  </aside>;
}

function verifiedDefaultRoute(model?: ModelState): ModelRoute | undefined {
  const selection = model?.defaultRoute;
  if (!model || !selection) return undefined;
  const provider = selection.provider === 'CHATGPT' ? model.chatgpt : model.deepseek;
  if (provider.status !== 'READY') return undefined;
  return provider.routes.find(route => route.provider === selection.provider && route.modelId === selection.modelId && route.thinkingType === selection.thinkingType && route.verifiedAt != null);
}

function modelGate(model: ModelState | undefined, runtime: RuntimeStatus | undefined) {
  const route = verifiedDefaultRoute(model);
  const gateway = runtime?.components.find(component => component.id === 'cliproxyapi');
  if (!route) return { route, ready: false, reason: 'Agent turns are unavailable until a model route is configured and verified.' };
  if (!runtime) return { route, ready: false, reason: 'Model gateway status is unavailable; launch and verify the gateway before continuing.' };
  if (!runtime.modelAvailable) {
    const status = gateway?.status.replaceAll('_', ' ').toLowerCase() ?? 'unavailable';
    return { route, ready: false, reason: `Model gateway is ${status}; launch and verify the gateway before continuing.` };
  }
  const codex = runtime.components.find(component => component.id === 'codex');
  return {
    route,
    ready: true,
    reason: codex?.status === 'NOT_CONFIGURED'
      ? 'Codex App Server is not configured; Send remains disabled until its later runtime slice.'
      : 'Codex thread runtime is not available in this build.',
  };
}

function ThreadHistory({ workspaceId, selectedThreadId, onSelect, compact = false }: { workspaceId: string; selectedThreadId?: string; onSelect: (threadId: string) => void; compact?: boolean }) {
  const threads = useQuery({ queryKey: ['threads', workspaceId], queryFn: () => request('thread.list', { workspaceId }) });
  if (threads.isPending) return <p className="sidebar-empty" role="status">Loading thread history…</p>;
  if (threads.isError) return <div className="thread-history-error"><p role="alert">Thread history is unavailable.</p><button type="button" onClick={() => void threads.refetch()}>Reload history</button></div>;
  if (!threads.data.threads.length) return <p className="sidebar-empty">No saved threads</p>;
  return <div className={compact ? 'thread-history compact' : 'thread-history'}>{threads.data.threads.map(thread =>
    <button key={thread.threadId} type="button" className="thread-history-row" aria-label={thread.title} aria-current={thread.threadId === selectedThreadId ? 'true' : undefined} onClick={() => onSelect(thread.threadId)}>
      <strong>{thread.title}</strong><small>{thread.defaultAgentMode} · {thread.defaultExecutionContext}</small><time dateTime={thread.updatedAt}>{new Date(thread.updatedAt).toLocaleString()}</time>
    </button>)}</div>;
}

function ThreadComposer({ workspaceId, model, onCreated }: { workspaceId: string; model?: ModelState; onCreated: (thread: Thread) => void }) {
  const accounts = useQuery({ queryKey: ['accounts', workspaceId, 'thread-composer'], queryFn: () => request('account.list', { workspaceId }) });
  const [title, setTitle] = useState('New research thread');
  const [mode, setMode] = useState<AgentMode>('ASK');
  const [execution, setExecution] = useState<ExecutionContext>('NONE_READ_ONLY');
  const [accountId, setAccountId] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const route = model?.defaultRoute;
  const selectedModel: ThreadModel | undefined = route ? { provider: route.provider, modelId: route.modelId, ...(route.thinkingType ? { thinkingType: route.thinkingType } : {}) } : undefined;
  const create = async (event: FormEvent) => {
    event.preventDefault(); setBusy(true); setError(undefined);
    const payload: ThreadCreate = { workspaceId, title: title.trim(), defaultAgentMode: mode, defaultExecutionContext: execution, linkedContexts: [], ...(accountId ? { accountId } : {}), ...(selectedModel ? { model: selectedModel } : {}) };
    try { onCreated(await request('thread.create', payload)); setTitle('New research thread'); }
    catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };
  return <section className="composer thread-composer" aria-labelledby="thread-composer-title">
    <div className="composer-heading"><div><h2 id="thread-composer-title">Start a thread</h2><p className="muted">The selections below become this Thread's defaults.</p></div><span className="badge readonly">No financial approval</span></div>
    <form onSubmit={create}>
      <label className="field">Thread title<input value={title} onChange={event => setTitle(event.target.value)} maxLength={120} required /></label>
      <div className="thread-picker-grid">
        <label className="field">Agent mode<select value={mode} onChange={event => setMode(event.target.value as AgentMode)}><option value="ASK">Ask · read only</option><option value="RESEARCH">Research · read only</option><option value="BACKTEST">Backtest · historical simulation</option><option value="TRADE">Trade · later approval required</option></select></label>
        <label className="field">Execution context<select value={execution} onChange={event => setExecution(event.target.value as ExecutionContext)}><option value="NONE_READ_ONLY">None · read only</option><option value="HISTORICAL_SIMULATION">Historical simulation</option><option value="LOCAL_PAPER">Local Paper</option><option value="ALPACA_PAPER">Alpaca Paper</option><option value="TRADING212_DEMO">Trading 212 Demo</option><option value="TRADING212_LIVE">Trading 212 Live</option><option value="BINANCE_TESTNET">Binance Testnet</option><option value="BINANCE_LIVE">Binance Live</option><option value="BITGET_DEMO">Bitget Demo</option><option value="BITGET_LIVE">Bitget Live</option></select></label>
        <label className="field">Account<select value={accountId} onChange={event => setAccountId(event.target.value)} disabled={accounts.isPending}><option value="">No account selected</option>{accounts.data?.accounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.label} · {account.environment}</option>)}</select></label>
      </div>
      <div className="composer-context"><span className="badge">Mode: {mode}</span><span className="badge">Execution: {execution}</span><span className="muted">Model: {selectedModel ? `${selectedModel.provider} · ${selectedModel.modelId}` : 'Selected when the next Turn starts'}</span></div>
      {error && <p className="error-text" role="alert">{error}</p>}
      <div className="composer-footer"><span>Context references: none</span><button className="primary" type="submit" disabled={busy || !title.trim() || accounts.isError}>{busy ? 'Creating…' : 'Create Thread'}</button></div>
    </form>
  </section>;
}

function ThreadDetail({ threadId }: { threadId: string }) {
  const projection = useDomainProjection('thread', threadId, fromThreadSnapshot);
  if (projection.error) return <div className="error-banner" role="alert"><div><strong>Thread needs attention</strong><p>{explainError(projection.error)}</p></div><button type="button" onClick={() => void projection.reload()}>Reload thread</button></div>;
  const thread = projection.data;
  if (!thread) return <p role="status">Loading thread…</p>;
  return <section className="card thread-detail" aria-labelledby="thread-detail-title">
    <div className="account-heading"><div><h2 id="thread-detail-title">{thread.title}</h2><p className="muted">Thread {thread.threadId}</p></div><span className="badge">{thread.status}</span></div>
    <div className="composer-context"><span className="badge">Mode: {thread.defaultAgentMode}</span><span className="badge">Execution: {thread.defaultExecutionContext}</span><span className="muted">Account: {thread.accountId ?? 'None selected'}</span><span className="muted">Model: {thread.model ? `${thread.model.provider} · ${thread.model.modelId}` : 'Not selected'}</span></div>
    <dl className="thread-provenance"><div><dt>Created</dt><dd><time dateTime={thread.createdAt}>{new Date(thread.createdAt).toLocaleString()}</time></dd></div><div><dt>Updated</dt><dd><time dateTime={thread.updatedAt}>{new Date(thread.updatedAt).toLocaleString()}</time></dd></div><div><dt>Context references</dt><dd>{thread.linkedContexts.length ? thread.linkedContexts.map(context => `${context.kind}:${context.id}`).join(', ') : 'None'}</dd></div></dl>
    <div className="empty-activity" role="status"><h3>Thread timeline</h3><p>No turns have started. Send remains unavailable until the Codex runtime slice is complete.</p></div>
  </section>;
}

function ThreadsPage({ workspaceId, model, selectedThreadId, onSelect, onCreated }: { workspaceId: string; model?: ModelState; selectedThreadId?: string; onSelect: (threadId: string) => void; onCreated: (thread: Thread) => void }) {
  return <><div className="page-heading"><h1>Threads</h1><p>Local history restores each Thread's own defaults and context.</p></div><div className="threads-layout"><section className="card threads-list-card"><h2>History</h2><ThreadHistory workspaceId={workspaceId} selectedThreadId={selectedThreadId} onSelect={onSelect} /></section><section className="threads-main"><ThreadComposer workspaceId={workspaceId} model={model} onCreated={onCreated} />{selectedThreadId && <ThreadDetail threadId={selectedThreadId} />}</section></div></>;
}

function RiskSettings({ workspace, risk }: { workspace: Workspace; risk?: RiskPolicyState }) {
  const [draft, setDraft] = useState<RiskDraft>();
  const [draftVersion, setDraftVersion] = useState('');
  useEffect(() => {
    if (risk && draftVersion !== risk.stateVersion) {
      setDraft(draftFromPolicy(risk.policy));
      setDraftVersion(risk.stateVersion);
    }
  }, [risk?.stateVersion, risk, draftVersion]);
  if (!risk || !draft) return <p role="status">Loading risk policy…</p>;
  return <RiskDefaults workspaceId={workspace.workspaceId} baseCurrency={workspace.baseCurrency} state={risk} draft={draft} onDraftChange={setDraft} />;
}

function Onboarding({ workspace, risk, model, modelError, reloadModel, runtime, onCompleted }: { workspace: Workspace; risk?: RiskPolicyState; model?: ModelState; modelError?: unknown; reloadModel: () => Promise<unknown>; runtime?: RuntimeStatus; onCompleted: () => void }) {
  const accounts = useQuery({ queryKey: ['accounts', workspace.workspaceId, 'onboarding'], queryFn: () => request('account.list', { workspaceId: workspace.workspaceId }) });
  const [draft, setDraft] = useState<RiskDraft>();
  const [draftVersion, setDraftVersion] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  useEffect(() => {
    if (risk && draftVersion !== risk.stateVersion) {
      setDraft(draftFromPolicy(risk.policy));
      setDraftVersion(risk.stateVersion);
    }
  }, [risk?.stateVersion, risk, draftVersion]);
  if (!risk || !draft) return <section className="onboarding" aria-labelledby="onboarding-title"><p role="status">Loading onboarding state…</p></section>;
  const step = risk.onboardingCompleted ? 5 : risk.onboardingStep;
  const gate = modelGate(model, runtime);
  const route = gate.route;
  const accountReady = accounts.status === 'success' && !accounts.isFetching && accounts.dataUpdatedAt > 0;
  const liveAccounts = accountReady ? accounts.data.accounts.filter(account => account.environment === 'LIVE') : undefined;
  const allLiveDisarmed = accountReady && liveAccounts!.every(account => account.health.arming === 'DISARMED');
  const liveAccountSummary = !accountReady
    ? (accounts.isError ? 'Unavailable — reload account health' : accounts.isFetching ? 'Loading account health…' : 'Unavailable — reload account health')
    : allLiveDisarmed ? 'All DISARMED' : 'A Live account needs to be DISARMED';
  const setStep = async (next: number, expectedStateVersion = risk.stateVersion) => {
    setBusy(true); setError(undefined);
    try { await request('onboarding.set_step', { workspaceId: workspace.workspaceId, expectedStateVersion, step: next }); }
    catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };
  const complete = async () => {
    setBusy(true); setError(undefined);
    try { await request('onboarding.complete', { workspaceId: workspace.workspaceId, expectedStateVersion: risk.stateVersion }); onCompleted(); }
    catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };
  const next = () => {
    if (step === 3 && !gate.ready) { setError(gate.reason); return; }
    if (step === 4 && !risk.configured) { setError('Save risk defaults before continuing.'); return; }
    if (step < 5) void setStep(step + 1);
    else void complete();
  };
  const riskSaved = (saved: RiskPolicyState) => {
    setDraftVersion(saved.stateVersion);
    void setStep(5, saved.stateVersion);
  };
  const providerSummary = !accountReady ? (accounts.isError ? 'Unavailable — reload provider connections' : accounts.isFetching ? 'Loading provider connections…' : 'Unavailable — reload provider connections') : accounts.data.accounts.length ? accounts.data.accounts.map(account => `${account.label} · ${account.providerId} ${account.environment}`).join(' · ') : 'No external broker/data provider connected';
  return <section className="onboarding" aria-labelledby="onboarding-title">
    <ol className="steps" aria-label="Workspace setup progress">{['Workspace', 'Providers', 'Model', 'Risk defaults', 'Ready'].map((label, index) => <li key={label} aria-current={index + 1 === step ? 'step' : undefined}><span>{index + 1}</span>{label}</li>)}</ol>
    {step === 1 && <section className="card onboarding-card"><h1 id="onboarding-title">Workspace</h1><p className="muted">{workspace.name} · base currency {workspace.baseCurrency}</p><dl className="summary-list"><div><dt>Local storage</dt><dd className="path">{workspace.path}</dd></div><div><dt>Workspace ID</dt><dd className="identity">{workspace.workspaceId}</dd></div></dl><p>Continue to reuse the existing provider connection and model setup surfaces.</p></section>}
    {step === 2 && <section className="card onboarding-card"><h1 id="onboarding-title">Providers</h1><p className="muted">Connect read-only broker or data providers. Live accounts remain DISARMED.</p><Accounts workspaceId={workspace.workspaceId} /></section>}
    {step === 3 && <section className="card onboarding-card"><h1 id="onboarding-title">Model</h1><p className="muted">Verify at least one real model route before Ready.</p><Models workspaceId={workspace.workspaceId} model={model} modelError={modelError} reloadModel={reloadModel} /></section>}
    {step === 4 && <section className="card onboarding-card"><h1 id="onboarding-title">Risk defaults</h1><RiskDefaults workspaceId={workspace.workspaceId} baseCurrency={workspace.baseCurrency} state={risk} draft={draft} onDraftChange={setDraft} onSaved={riskSaved} continueLabel="Save and continue" /></section>}
    {step === 5 && <section className="card onboarding-card"><h1 id="onboarding-title">Ready</h1><p className="muted">Review the current setup before completing onboarding.</p><dl className="summary-list"><div><dt>Workspace / currency</dt><dd>{workspace.name} · {workspace.baseCurrency}</dd></div><div><dt>Providers</dt><dd>{providerSummary}</dd></div><div><dt>Model route</dt><dd>{route ? `${route.provider} · ${route.modelId}${route.thinkingType ? ` · ${route.thinkingType}` : ''}` : 'Unavailable — verify a default route'}</dd></div><div><dt>Automatic fallback</dt><dd>{model?.automaticFallback ? 'ON · DeepSeek fallback disclosed' : 'OFF'}</dd></div><div><dt>Live accounts</dt><dd>{liveAccountSummary}</dd></div></dl><div className="notice"><strong>{gate.ready ? 'Agent turns unavailable' : 'Model route unavailable'}</strong><p>{gate.reason}</p></div></section>}
    {error && <p className="error-text" role="alert">{error}</p>}
    <div className="onboarding-actions">{step > 1 && <button type="button" onClick={() => void setStep(step - 1)} disabled={busy}>Back</button>}{step < 5 ? <button className="primary" type="button" onClick={next} disabled={busy || (step === 3 && !gate.ready) || (step === 4 && !risk.configured)}>{busy ? 'Saving…' : `Continue to ${['', 'Providers', 'Model', 'Risk defaults', 'Ready'][step]}`}</button> : <><button type="button" onClick={() => void setStep(4)} disabled={busy}>Edit setup</button><button className="primary" type="button" onClick={next} disabled={busy || !gate.ready || !risk.configured || !allLiveDisarmed}>{busy ? 'Completing…' : 'Complete onboarding'}</button></>}</div>
  </section>;
}

export default function App() {
  const [page, setPage] = useState<Page>('New Thread');
  const [selectedThreadId, setSelectedThreadId] = useState<string>();
  const [setup, setSetup] = useState(false);
  const [workspacePicker, setWorkspacePicker] = useState(false);
  const [settingsTab, setSettingsTab] = useState('Providers & Models');
  const state = useWorkspace();
  const workspace = state.workspace;
  const riskProjection = useDomainProjection('risk', workspace?.workspaceId, fromRiskSnapshot);
  const modelProjection = useDomainProjection('model', workspace?.workspaceId, fromModelSnapshot);
  const risk = riskProjection.data;
  const model = modelProjection.data;
  const modelState = modelGate(model, state.runtime.data);
  const defaultModelRoute = modelState.route;
  const modelReady = modelState.ready;
  const projectionError = riskProjection.error ?? modelProjection.error;
  const reloadProjections = () => { void riskProjection.reload(); void modelProjection.reload(); };
  const navigate = (destination: Page) => {
    setPage(destination); setSetup(false); setWorkspacePicker(false);
    if (destination === 'New Thread') setSelectedThreadId(undefined);
    document.querySelectorAll('details[open]').forEach(details => details.removeAttribute('open'));
  };
  const selectThread = (threadId: string) => { setSelectedThreadId(threadId); setPage('Threads'); setSetup(false); setWorkspacePicker(false); };
  const createdThread = (thread: Thread) => { setSelectedThreadId(thread.threadId); setPage('Threads'); };
  const submit = async (options: OpenWorkspace) => {
    try { await state.opening.mutateAsync(options); setWorkspacePicker(false); setSetup(true); setPage('New Thread'); } catch { /* Render the canonical error below. */ }
  };
  const onboardingVisible = !workspacePicker && (setup || Boolean(workspace && risk && page === 'New Thread' && (!risk.onboardingCompleted || !modelReady)));
  const title = workspacePicker || setup || (!workspace && page === 'New Thread') ? 'Workspace setup' : page;
  return <div className="app-shell">
    <a className="skip-link" href="#main">Skip to content</a>
    <aside className="sidebar">
      <div className="brand">Trade<b>X</b></div>
      <Navigation page={page} navigate={navigate} />
      <div className="sidebar-divider" />
      <div className="recent-heading">Recent threads</div>{workspace ? <ThreadHistory workspaceId={workspace.workspaceId} selectedThreadId={selectedThreadId} onSelect={selectThread} compact /> : <p className="sidebar-empty">No threads yet</p>}
      <div className="runtime-summary"><strong>{modelReady ? `Model ready · ${defaultModelRoute?.provider}` : defaultModelRoute ? 'Model gateway unavailable' : 'Model not configured'}</strong><p>{modelReady ? 'A verified route is available.' : modelState.reason}</p></div>
    </aside>
    <div className="shell">
      <header className="topbar">
        <details className="mobile-nav"><summary>More</summary><Navigation page={page} navigate={navigate} /></details>
        <span className="topbar-title">{title}</span>
        <span className="badge readonly">Read only</span>
        <button className="workspace-button" onClick={() => { setPage('New Thread'); setSetup(false); setWorkspacePicker(true); }}>Workspace</button>
      </header>
      <main id="main" tabIndex={-1}>
        {browserIntegration && <div className="integration-notice">Browser verification · isolated temporary workspace · provider responses are test fixtures</div>}
        {state.error != null && <div className="error-banner" role="alert"><div><strong>Workspace needs attention</strong><p>{explainError(state.error)}</p></div><button onClick={state.recover}>Retry connection</button></div>}
        {projectionError != null && <div className="error-banner" role="alert"><div><strong>Workspace state needs attention</strong><p>{explainError(projectionError)}</p></div><button onClick={reloadProjections}>Reload workspace state</button></div>}
        {state.opening.isPending && !workspace ? <p role="status">Opening local workspace…</p> :
          (workspacePicker || (!workspace && page === 'New Thread')) ? <WorkspaceSetup busy={state.opening.isPending} submit={submit} /> :
          (workspace && onboardingVisible) ? <Onboarding workspace={workspace} risk={risk} model={model} modelError={modelProjection.error} reloadModel={modelProjection.reload} runtime={state.runtime.data} onCompleted={() => { setSetup(false); setPage('New Thread'); }} /> :
          <div className="workspace-layout"><section className="content">
            {page === 'New Thread' && <>
              <div className="thread-welcome"><h1>What would you like to research?</h1><p>Ask a question, explore an opportunity, or review your portfolio.</p></div>
              {workspace && <ThreadComposer workspaceId={workspace.workspaceId} model={model} onCreated={createdThread} />}
              <div className="notice model-notice"><div><strong>{modelReady ? 'Agent turns unavailable' : defaultModelRoute ? 'Model gateway unavailable' : 'Connect a model provider'}</strong><p>{modelState.reason}</p></div><button onClick={() => navigate('Settings')}>Providers &amp; Models</button></div>
              <div className="empty-activity"><h2>Thread activity</h2><p>No agent turns have started in this workspace.</p></div>
            </>}
            {page === 'Threads' && workspace && <ThreadsPage workspaceId={workspace.workspaceId} model={model} selectedThreadId={selectedThreadId} onSelect={selectThread} onCreated={createdThread} />}
            {page === 'Settings' && <>
              <div className="page-heading"><h1>Settings</h1><p>Manage your local workspace and connected services.</p></div>
              <div className="settings-tabs" role="group" aria-label="Settings sections">{['Providers & Models', 'Risk & Limits', 'Data & Storage', 'Account Health', 'Appearance', 'About'].map(tab =>
                <button key={tab} aria-pressed={settingsTab === tab} onClick={() => setSettingsTab(tab)}>{tab}</button>)}</div>
              <section className="card settings-section"><h2>{settingsTab}</h2>
                {workspace && (settingsTab === 'Providers & Models' || settingsTab === 'Account Health') && <Accounts key={workspace.workspaceId} workspaceId={workspace.workspaceId} healthOnly={settingsTab === 'Account Health'} />}
                {workspace && settingsTab === 'Providers & Models' && <Models key={`models:${workspace.workspaceId}`} workspaceId={workspace.workspaceId} model={model} modelError={modelProjection.error} reloadModel={modelProjection.reload} />}
                {settingsTab === 'Providers & Models' || settingsTab === 'About' ? <>
                  <p className="muted">{settingsTab === 'About' ? 'TradeX 0.1.0 · local desktop workspace' : modelReady ? 'A verified model route is available; agent turns remain disabled until Codex App Server is configured.' : modelState.reason}</p>
                  <ul className="component-list">{state.runtime.data?.components.map(component => <li key={component.id}><div><strong>{component.id === 'cliproxyapi' ? 'CLIProxyAPI' : component.id === 'codex' ? 'Codex App Server' : component.id === 'control-plane' ? 'Control Plane' : 'Order Gateway'}</strong><p>{component.message}</p></div><span className="badge">{state.runtime.isError ? 'Unavailable' : component.status === 'RUNNING' ? 'Available' : component.status.replaceAll('_', ' ')}</span></li>)}</ul>
                  <button onClick={() => { void state.runtime.refetch(); }} disabled={state.runtime.isFetching}>Refresh runtime status</button>
                </> : settingsTab === 'Risk & Limits' && workspace ? <RiskSettings workspace={workspace} risk={risk} /> : settingsTab === 'Data & Storage' && workspace ? <><p className="muted">Local workspace folder</p><p className="path">{workspace.path}</p><button onClick={() => { setWorkspacePicker(true); setSetup(false); }}>Open another workspace</button></> :
                  <p className="muted">{settingsTab === 'Account Health' ? 'Connection, authentication, stream, reconciliation and execution are separate checks.' : 'The workspace currently uses the RevC light theme.'}</p>}
              </section>
            </>}
            {page === 'Accounts' && <><div className="page-heading"><h1>Accounts</h1><p>Connect and inspect your provider accounts.</p></div>{workspace ? <Accounts key={workspace.workspaceId} workspaceId={workspace.workspaceId} /> : <p>Open a workspace to manage accounts.</p>}</>}
            {page !== 'New Thread' && page !== 'Threads' && page !== 'Settings' && page !== 'Accounts' && <>
              <div className="page-heading"><h1>{page}</h1></div>
              <section className="card empty-page"><h2>{page === 'Markets' ? 'Market data is not connected' : page === 'Watchlists' ? 'No watchlists' : page === 'Strategies' ? 'No saved strategies' : 'No artifacts'}</h2>
                <p>{page === 'Markets' ? 'Provider connections are not available in this build. No account or market data has been loaded.' : 'This workflow is not available in this build. Your local workspace is ready for the next setup steps.'}</p>
                <button onClick={() => navigate('Settings')}>Open settings</button>
              </section>
            </>}
          </section>{workspace && <WorkspaceDetails workspace={workspace} />}</div>}
      </main>
    </div>
  </div>;
}
