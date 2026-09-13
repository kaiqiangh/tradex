import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery } from '@tanstack/react-query';
import { open } from '@tauri-apps/plugin-dialog';
import type { ModelRoute, ModelState, RiskPolicyState } from '../shared/ipc-types.ts';
import { browserIntegration, desktop, explainError, request, transportAvailable } from './client.ts';
import { Accounts } from './Accounts.tsx';
import { Models } from './Models.tsx';
import { RiskDefaults, draftFromPolicy, type RiskDraft } from './RiskDefaults.tsx';
import { fromModelSnapshot, fromRiskSnapshot } from './projection.ts';
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

function Onboarding({ workspace, risk, model, onCompleted }: { workspace: Workspace; risk?: RiskPolicyState; model?: ModelState; onCompleted: () => void }) {
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
  const route = verifiedDefaultRoute(model);
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
    if (step === 3 && !route) { setError('Verify a default model route before continuing.'); return; }
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
    {step === 3 && <section className="card onboarding-card"><h1 id="onboarding-title">Model</h1><p className="muted">Verify at least one real model route before Ready.</p><Models workspaceId={workspace.workspaceId} /></section>}
    {step === 4 && <section className="card onboarding-card"><h1 id="onboarding-title">Risk defaults</h1><RiskDefaults workspaceId={workspace.workspaceId} baseCurrency={workspace.baseCurrency} state={risk} draft={draft} onDraftChange={setDraft} onSaved={riskSaved} continueLabel="Save and continue" /></section>}
    {step === 5 && <section className="card onboarding-card"><h1 id="onboarding-title">Ready</h1><p className="muted">Review the current setup before completing onboarding.</p><dl className="summary-list"><div><dt>Workspace / currency</dt><dd>{workspace.name} · {workspace.baseCurrency}</dd></div><div><dt>Providers</dt><dd>{providerSummary}</dd></div><div><dt>Model route</dt><dd>{route ? `${route.provider} · ${route.modelId}${route.thinkingType ? ` · ${route.thinkingType}` : ''}` : 'Unavailable — verify a default route'}</dd></div><div><dt>Automatic fallback</dt><dd>{model?.automaticFallback ? 'ON · DeepSeek fallback disclosed' : 'OFF'}</dd></div><div><dt>Live accounts</dt><dd>{liveAccountSummary}</dd></div></dl><div className="notice"><strong>Agent turns unavailable</strong><p>Codex App Server is not configured; Send remains disabled until its later runtime slice.</p></div></section>}
    {error && <p className="error-text" role="alert">{error}</p>}
    <div className="onboarding-actions">{step > 1 && <button type="button" onClick={() => void setStep(step - 1)} disabled={busy}>Back</button>}{step < 5 ? <button className="primary" type="button" onClick={next} disabled={busy || (step === 3 && !route) || (step === 4 && !risk.configured)}>{busy ? 'Saving…' : `Continue to ${['', 'Providers', 'Model', 'Risk defaults', 'Ready'][step]}`}</button> : <><button type="button" onClick={() => void setStep(4)} disabled={busy}>Edit setup</button><button className="primary" type="button" onClick={next} disabled={busy || !route || !risk.configured || !allLiveDisarmed}>{busy ? 'Completing…' : 'Complete onboarding'}</button></>}</div>
  </section>;
}

export default function App() {
  const [page, setPage] = useState<Page>('New Thread');
  const [setup, setSetup] = useState(false);
  const [settingsTab, setSettingsTab] = useState('Providers & Models');
  const state = useWorkspace();
  const workspace = state.workspace;
  const riskProjection = useDomainProjection('risk', workspace?.workspaceId, fromRiskSnapshot);
  const modelProjection = useDomainProjection('model', workspace?.workspaceId, fromModelSnapshot);
  const risk = riskProjection.data;
  const model = modelProjection.data;
  const navigate = (destination: Page) => {
    setPage(destination); setSetup(false);
    document.querySelectorAll('details[open]').forEach(details => details.removeAttribute('open'));
  };
  const submit = async (options: OpenWorkspace) => {
    try { await state.opening.mutateAsync(options); setSetup(true); setPage('New Thread'); } catch { /* Render the canonical error below. */ }
  };
  const onboardingVisible = setup || Boolean(workspace && risk && page === 'New Thread' && (!risk.onboardingCompleted || !verifiedDefaultRoute(model)));
  const title = setup || (!workspace && page === 'New Thread') ? 'Workspace setup' : page;
  return <div className="app-shell">
    <a className="skip-link" href="#main">Skip to content</a>
    <aside className="sidebar">
      <div className="brand">Trade<b>X</b></div>
      <Navigation page={page} navigate={navigate} />
      <div className="sidebar-divider" />
      <div className="recent-heading">Recent threads</div><p className="sidebar-empty">No threads yet</p>
      <div className="runtime-summary"><strong>Model not configured</strong><p>Choose a provider to begin.</p></div>
    </aside>
    <div className="shell">
      <header className="topbar">
        <details className="mobile-nav"><summary>More</summary><Navigation page={page} navigate={navigate} /></details>
        <span className="topbar-title">{title}</span>
        <span className="badge readonly">Read only</span>
        <button className="workspace-button" onClick={() => { setPage('New Thread'); setSetup(true); }}>Workspace</button>
      </header>
      <main id="main" tabIndex={-1}>
        {browserIntegration && <div className="integration-notice">Browser verification · isolated temporary workspace · provider responses are test fixtures</div>}
        {state.error != null && <div className="error-banner" role="alert"><div><strong>Workspace needs attention</strong><p>{explainError(state.error)}</p></div><button onClick={state.recover}>Retry connection</button></div>}
        {state.opening.isPending && !workspace ? <p role="status">Opening local workspace…</p> :
          (!workspace && page === 'New Thread') ? <WorkspaceSetup busy={state.opening.isPending} submit={submit} /> :
          (workspace && onboardingVisible) ? <Onboarding workspace={workspace} risk={risk} model={model} onCompleted={() => { setSetup(false); setPage('New Thread'); }} /> :
          <div className="workspace-layout"><section className="content">
            {page === 'New Thread' && <>
              <div className="thread-welcome"><h1>What would you like to research?</h1><p>Ask a question, explore an opportunity, or review your portfolio.</p></div>
              <section className="composer" aria-label="Thread composer">
                <div className="composer-context"><span className="badge">Agent mode: Ask</span><span className="badge">Execution: Read only</span><span className="muted">No account selected</span></div>
                <textarea aria-label="Message" placeholder="Configure a model provider to start a thread" disabled />
                <div className="composer-footer"><span>Model not configured</span><button className="primary" disabled>Send</button></div>
              </section>
              <div className="notice model-notice"><div><strong>Connect a model provider</strong><p>Agent turns are unavailable until a model route is configured and verified.</p></div><button onClick={() => navigate('Settings')}>Providers &amp; Models</button></div>
              <div className="empty-activity"><h2>Thread activity</h2><p>No agent turns have started in this workspace.</p></div>
            </>}
            {page === 'Settings' && <>
              <div className="page-heading"><h1>Settings</h1><p>Manage your local workspace and connected services.</p></div>
              <div className="settings-tabs" role="group" aria-label="Settings sections">{['Providers & Models', 'Risk & Limits', 'Data & Storage', 'Account Health', 'Appearance', 'About'].map(tab =>
                <button key={tab} aria-pressed={settingsTab === tab} onClick={() => setSettingsTab(tab)}>{tab}</button>)}</div>
              <section className="card settings-section"><h2>{settingsTab}</h2>
                {workspace && (settingsTab === 'Providers & Models' || settingsTab === 'Account Health') && <Accounts key={workspace.workspaceId} workspaceId={workspace.workspaceId} healthOnly={settingsTab === 'Account Health'} />}
                {workspace && settingsTab === 'Providers & Models' && <Models key={`models:${workspace.workspaceId}`} workspaceId={workspace.workspaceId} />}
                {settingsTab === 'Providers & Models' || settingsTab === 'About' ? <>
                  <p className="muted">{settingsTab === 'About' ? 'TradeX 0.1.0 · local desktop workspace' : 'No model provider is configured. Agent turns and onboarding Ready remain unavailable.'}</p>
                  <ul className="component-list">{state.runtime.data?.components.map(component => <li key={component.id}><div><strong>{component.id === 'cliproxyapi' ? 'CLIProxyAPI' : component.id === 'codex' ? 'Codex App Server' : component.id === 'control-plane' ? 'Control Plane' : 'Order Gateway'}</strong><p>{component.message}</p></div><span className="badge">{state.runtime.isError ? 'Unavailable' : component.status === 'RUNNING' ? 'Available' : component.status.replaceAll('_', ' ')}</span></li>)}</ul>
                  <button onClick={() => { void state.runtime.refetch(); }} disabled={state.runtime.isFetching}>Refresh runtime status</button>
                </> : settingsTab === 'Risk & Limits' && workspace ? <RiskSettings workspace={workspace} risk={risk} /> : settingsTab === 'Data & Storage' && workspace ? <><p className="muted">Local workspace folder</p><p className="path">{workspace.path}</p><button onClick={() => setSetup(true)}>Open another workspace</button></> :
                  <p className="muted">{settingsTab === 'Account Health' ? 'Connection, authentication, stream, reconciliation and execution are separate checks.' : 'The workspace currently uses the RevC light theme.'}</p>}
              </section>
            </>}
            {page === 'Accounts' && <><div className="page-heading"><h1>Accounts</h1><p>Connect and inspect your provider accounts.</p></div>{workspace ? <Accounts key={workspace.workspaceId} workspaceId={workspace.workspaceId} /> : <p>Open a workspace to manage accounts.</p>}</>}
            {page !== 'New Thread' && page !== 'Settings' && page !== 'Accounts' && <>
              <div className="page-heading"><h1>{page}</h1></div>
              <section className="card empty-page"><h2>{page === 'Threads' ? 'No saved threads' : page === 'Markets' ? 'Market data is not connected' : page === 'Watchlists' ? 'No watchlists' : page === 'Strategies' ? 'No saved strategies' : 'No artifacts'}</h2>
                <p>{page === 'Threads' ? 'Configure a model provider before starting a persistent thread.' : page === 'Markets' ? 'Provider connections are not available in this build. No account or market data has been loaded.' : 'This workflow is not available in this build. Your local workspace is ready for the next setup steps.'}</p>
                <button onClick={() => navigate('Settings')}>Open settings</button>
              </section>
            </>}
          </section>{workspace && <WorkspaceDetails workspace={workspace} />}</div>}
      </main>
    </div>
  </div>;
}
