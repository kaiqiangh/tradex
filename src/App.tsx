import { useEffect, useId, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { open } from '@tauri-apps/plugin-dialog';
import { createPortal } from 'react-dom';
import type { AgentMode, CapabilityDecision, ContextCatalog, ContextCatalogEntry, ExecutionContext, ModelRoute, ModelState, ResearchFocus, ResearchToolId, ResearchToolInvocation, ResearchToolResult, RiskPolicyState, RuntimeStatus, Thread, ThreadContextRef, ThreadCreate, ThreadItem, ThreadModel, ThreadTurn, TimeStatus, TurnCancel, TurnRetry, TurnStart } from '../shared/ipc-types.ts';
import { browserIntegration, desktop, explainError, request, transportAvailable } from './client.ts';
import { Accounts } from './Accounts.tsx';
import { Models } from './Models.tsx';
import { RiskDefaults, draftFromPolicy, type RiskDraft } from './RiskDefaults.tsx';
import { DataSources } from './DataSources.tsx';
import { Markets } from './Markets.tsx';
import { Watchlists } from './Watchlists.tsx';
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

const toolLabels: Record<string, string> = {
  public_market_read: 'Public market read',
  account_read: 'Account read',
  historical_simulation: 'Historical simulation',
  paper_demo_testnet_execution: 'Paper / Demo / Testnet',
  live_order_proposal: 'Live order proposal',
};

function formatContextRefs(contexts: ThreadContextRef[]) {
  return contexts.length ? contexts.map(context => `${context.kind}:${context.id}#${context.hash}`).join(', ') : 'None';
}

function mergeContextRefs(...lists: ThreadContextRef[][]) {
  const seen = new Set<string>();
  return lists.flat().filter(context => {
    const key = `${context.kind}:${context.id}:${context.hash}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function CapabilitySummary({ decision, loading, error }: { decision?: CapabilityDecision; loading?: boolean; error?: unknown }) {
  if (loading) return <span className="muted" role="status">Checking tool capability…</span>;
  if (error) return <span className="error-text" role="alert">{explainError(error)}</span>;
  if (!decision) return <span className="muted">Capability unavailable</span>;
  const tools = decision.allowedTools.map(tool => toolLabels[tool] ?? tool).join(' · ');
  const researchTools = decision.researchTools.map(tool => toolLabels[tool.id] ?? tool.id).join(' · ') || 'None';
  const reason = decision.reason === 'LIVE_READ_ONLY'
    ? 'Live account is read only'
    : decision.reason === 'LIVE_PROPOSAL_REQUIRES_ARMING_APPROVAL'
      ? 'Live proposal only; arming and approval are separate'
      : decision.reason === 'HISTORICAL_SIMULATION_ONLY'
        ? 'Historical simulation only'
        : decision.reason === 'LOCAL_PAPER_SIMULATION'
          ? 'TradeX simulation'
          : undefined;
  return <span className="capability-summary"><strong>Capability: {decision.level}</strong><span>{tools}</span><span className="muted">Data-plane research: {researchTools}</span>{reason && <span className="muted">{reason}</span>}</span>;
}

const modeOptions: { value: AgentMode; label: string }[] = [
  { value: 'ASK', label: 'Ask · read only' },
  { value: 'RESEARCH', label: 'Research · read only' },
  { value: 'BACKTEST', label: 'Backtest · historical simulation' },
  { value: 'TRADE', label: 'Trade · later approval required' },
];

const executionOptions: { value: ExecutionContext; label: string }[] = [
  { value: 'NONE_READ_ONLY', label: 'None · read only' },
  { value: 'HISTORICAL_SIMULATION', label: 'Historical simulation' },
  { value: 'LOCAL_PAPER', label: 'Local Paper' },
  { value: 'ALPACA_PAPER', label: 'Alpaca Paper' },
  { value: 'TRADING212_DEMO', label: 'Trading 212 Demo' },
  { value: 'TRADING212_LIVE', label: 'Trading 212 Live' },
  { value: 'BINANCE_TESTNET', label: 'Binance Testnet' },
  { value: 'BINANCE_LIVE', label: 'Binance Live' },
  { value: 'BITGET_DEMO', label: 'Bitget Demo' },
  { value: 'BITGET_LIVE', label: 'Bitget Live' },
];

function liveReadOnly(mode: AgentMode, environment: string) {
  return environment === 'LIVE' && (mode === 'ASK' || mode === 'RESEARCH');
}

function catalogAccountAvailable(catalog: ContextCatalog | undefined, accountId: string) {
  return catalog?.entries.some(entry => entry.contextRef.kind === 'account' && entry.contextRef.id === accountId && entry.available) ?? false;
}

function accountOptionLabel(account: { label: string; providerId: string; environment: string }, mode: AgentMode, available = true) {
  const liveDisclosure = liveReadOnly(mode, account.environment) ? ' · READ-ONLY' : '';
  return `${account.label} · ${account.providerId} · ${account.environment}${liveDisclosure}${available ? '' : ' · unavailable'}`;
}

function ContextPicker({ workspaceId, pending, onAttach, mode }: { workspaceId: string; pending: ThreadContextRef[]; onAttach: (contexts: ThreadContextRef[]) => void; mode: AgentMode }) {
  const catalog = useQuery({ queryKey: ['context-catalog', workspaceId], queryFn: () => request('context.catalog', { workspaceId }), retry: false });
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<ThreadContextRef[]>(pending);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const pickerId = useId().replaceAll(':', '');
  const titleId = `context-picker-title-${pickerId}`;
  useEffect(() => {
    if (open) {
      setDraft(pending);
      window.setTimeout(() => dialogRef.current?.querySelector<HTMLElement>('h3, input, button')?.focus(), 0);
    } else if (triggerRef.current) triggerRef.current.focus();
  }, [open, pending]);
  useEffect(() => {
    if (!open) return;
    const shell = document.querySelector<HTMLElement>('.app-shell');
    if (!shell) return;
    shell.inert = true;
    return () => { shell.inert = false; };
  }, [open]);
  useEffect(() => {
    if (!open) return;
    const handleDialogKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') { setOpen(false); return; }
      if (event.key !== 'Tab') return;
      const focusable = [...(dialogRef.current?.querySelectorAll<HTMLElement>('h3[tabindex="-1"], button:not(:disabled), input:not(:disabled), [tabindex="0"]') ?? [])];
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    window.addEventListener('keydown', handleDialogKey);
    return () => window.removeEventListener('keydown', handleDialogKey);
  }, [open]);
  const entries = catalog.data?.entries ?? [];
  const selected = (entry: ContextCatalogEntry) => draft.some(context => context.kind === entry.contextRef.kind && context.id === entry.contextRef.id && context.hash === entry.contextRef.hash);
  const toggle = (entry: ContextCatalogEntry) => setDraft(current => {
    const isSelected = current.some(context => context.kind === entry.contextRef.kind && context.id === entry.contextRef.id && context.hash === entry.contextRef.hash);
    return isSelected
      ? current.filter(context => !(context.kind === entry.contextRef.kind && context.id === entry.contextRef.id && context.hash === entry.contextRef.hash))
      : [...current, entry.contextRef];
  });
  const entryLabel = (context: ThreadContextRef) => entries.find(entry => entry.contextRef.kind === context.kind && entry.contextRef.id === context.id && entry.contextRef.hash === context.hash)?.label ?? `${context.kind}:${context.id}`;
  const pickerDialog = open ? createPortal(<div className="picker-backdrop">
    <div className="picker-dialog" role="dialog" aria-modal="true" aria-labelledby={titleId} ref={dialogRef}>
      <div className="picker-dialog-heading"><div><h3 id={titleId} tabIndex={-1}>Choose context</h3><p className="muted">Attach references for the next Turn only.</p></div><button type="button" aria-label="Close context picker" onClick={() => setOpen(false)}>×</button></div>
      {catalog.isPending && <p role="status">Loading context catalog…</p>}
      {catalog.isError && <div className="error-banner" role="alert"><p>Context catalog is unavailable.</p><button type="button" onClick={() => void catalog.refetch()}>Reload catalog</button></div>}
      {catalog.data && <>
        <fieldset className="context-options"><legend>Available contexts</legend>{entries.length ? entries.map(entry => <label className="context-option" key={`${entry.contextRef.kind}:${entry.contextRef.id}:${entry.contextRef.hash}`}><input type="checkbox" checked={selected(entry)} disabled={!entry.available} onChange={() => toggle(entry)} /><span><strong>{entry.label}</strong><small>{entry.providerId ? `${entry.providerId} · ` : ''}{entry.environment ?? entry.contextRef.kind}{liveReadOnly(mode, entry.environment ?? '') ? ' · READ-ONLY' : ''}{entry.readOnly ? ' · read-only context' : ''}</small>{!entry.available && <em>{entry.availabilityReason}</em>}</span></label>) : <p>No account contexts are available.</p>}</fieldset>
        {catalog.data.emptyStates.length > 0 && <section className="context-empty" aria-label="Unavailable context catalogs"><h4>Future context catalogs</h4><ul>{catalog.data.emptyStates.map(state => <li key={state.kind}><strong>{state.kind}</strong> — {state.availabilityReason}</li>)}</ul></section>}
      </>}
      <div className="picker-dialog-actions"><button type="button" onClick={() => setOpen(false)}>Cancel</button><button type="button" className="primary" disabled={catalog.isPending || catalog.isError} onClick={() => { onAttach(draft); setOpen(false); }}>Attach</button></div>
    </div>
  </div>, document.body) : null;
  return <div className="context-picker">
    <button ref={triggerRef} type="button" aria-haspopup="dialog" aria-expanded={open} onClick={() => setOpen(true)}>@ Context{pending.length ? ` · ${pending.length}` : ''}</button>
    {pending.length > 0 && <div className="context-chips" aria-label="Attached contexts">{pending.map(context => <span className="context-chip" key={`${context.kind}:${context.id}:${context.hash}`}><span>{entryLabel(context)}{context.kind === 'account' && context.hash ? ` · ${context.id.slice(0, 8)}` : ''}</span><button type="button" aria-label={`Remove ${entryLabel(context)}`} onClick={() => onAttach(pending.filter(item => item !== context))}>×</button></span>)}</div>}
    {pickerDialog}
  </div>;
}

function ThreadComposer({ workspaceId, model, onCreated, initialContexts = [] }: { workspaceId: string; model?: ModelState; onCreated: (thread: Thread) => void; initialContexts?: ThreadContextRef[] }) {
  const accounts = useQuery({ queryKey: ['accounts', workspaceId, 'thread-composer'], queryFn: () => request('account.list', { workspaceId }) });
  const contextCatalog = useQuery({ queryKey: ['context-catalog', workspaceId], queryFn: () => request('context.catalog', { workspaceId }), retry: false });
  const queryClient = useQueryClient();
  const [title, setTitle] = useState('New research thread');
  const [mode, setMode] = useState<AgentMode>('ASK');
  const [execution, setExecution] = useState<ExecutionContext>('NONE_READ_ONLY');
  const [accountId, setAccountId] = useState('');
  const [pendingContexts, setPendingContexts] = useState<ThreadContextRef[]>(initialContexts);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const capability = useQuery({
    queryKey: ['agent-capability', workspaceId, mode, execution, accountId, pendingContexts],
    queryFn: () => request('agent.capabilities', {
      workspaceId,
      agentMode: mode,
      executionContext: execution,
      ...(accountId ? { accountId } : {}),
      attachedContexts: pendingContexts,
    }),
    retry: false,
  });
  const route = verifiedDefaultRoute(model);
  const selectedModel: ThreadModel | undefined = route
    ? { provider: route.provider, modelId: route.modelId, ...(route.thinkingType ? { thinkingType: route.thinkingType } : {}) }
    : browserIntegration ? { provider: 'CHATGPT', modelId: 'gpt-5.6-sol' } : undefined;
  const create = async (event: FormEvent) => {
    event.preventDefault(); setBusy(true); setError(undefined);
    const payload: ThreadCreate = { workspaceId, title: title.trim(), defaultAgentMode: mode, defaultExecutionContext: execution, linkedContexts: pendingContexts, ...(accountId ? { accountId } : {}), ...(selectedModel ? { model: selectedModel } : {}) };
    try { const thread = await request('thread.create', payload); await queryClient.invalidateQueries({ queryKey: ['threads', workspaceId] }); onCreated(thread); setTitle('New research thread'); setPendingContexts([]); }
    catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };
  return <section className="composer thread-composer" aria-labelledby="thread-composer-title">
    <div className="composer-heading"><div><h2 id="thread-composer-title">Start a thread</h2><p className="muted">The selections below become this Thread's defaults.</p></div><span className="badge readonly">No financial approval</span></div>
    <form onSubmit={create}>
      <label className="field">Thread title<input value={title} onChange={event => setTitle(event.target.value)} maxLength={120} required /></label>
      <div className="thread-picker-grid">
        <label className="field">Agent mode<select value={mode} onChange={event => setMode(event.target.value as AgentMode)}>{modeOptions.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}</select></label>
        <label className="field">Execution context<select value={execution} onChange={event => setExecution(event.target.value as ExecutionContext)}>{executionOptions.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}</select></label>
        <label className="field">Account<select value={accountId} onChange={event => setAccountId(event.target.value)} disabled={accounts.isPending || contextCatalog.isPending || contextCatalog.isError}><option value="">No account selected</option>{accounts.data?.accounts.map(account => { const available = catalogAccountAvailable(contextCatalog.data, account.connectionId); return <option key={account.connectionId} value={account.connectionId} disabled={!available}>{accountOptionLabel(account, mode, available)}</option>; })}</select></label>
      </div>
      <ContextPicker workspaceId={workspaceId} pending={pendingContexts} onAttach={setPendingContexts} mode={mode} />
      <div className="composer-context"><span className="badge">Mode: {mode}</span><span className="badge">Execution: {execution}</span><span className="muted">Model: {selectedModel ? `${selectedModel.provider} · ${selectedModel.modelId}` : 'Selected when the next Turn starts'}</span><CapabilitySummary decision={capability.data} loading={capability.isPending} error={capability.error} /></div>
      {error && <p className="error-text" role="alert">{error}</p>}
      <div className="composer-footer"><span>Context references: {pendingContexts.length}</span><button className="primary" type="submit" disabled={busy || !title.trim() || accounts.isError || capability.isPending || capability.isError || !capability.data}>{busy ? 'Creating…' : 'Create Thread'}</button></div>
    </form>
  </section>;
}

function runtimeReady(runtime?: RuntimeStatus) {
  return runtime?.components.some(component => component.id === 'codex' && component.status === 'READY') === true
    && (browserIntegration || runtime.modelAvailable === true);
}

function routeAsThreadModel(route?: ModelRoute): ThreadModel | undefined {
  return route ? { provider: route.provider, modelId: route.modelId, ...(route.thinkingType ? { thinkingType: route.thinkingType } : {}) } : undefined;
}

function ResearchResultCard({ result, persisted = false, agentMode }: { result: ResearchToolResult; persisted?: boolean; agentMode?: AgentMode }) {
  const payload = result.payload;
  const findings = payload.findings ?? [];
  const scenarios = payload.scenarios ?? [];
  const evidence = payload.evidence ?? [];
  const limitations = payload.limitations ?? [];
  const instrumentRefs = payload.instrumentRefs ?? [];
  const artifactRefs = payload.artifactRefs ?? [];
  const spotVenues = payload.spotVenues ?? [];
  return <section className="research-result" tabIndex={0} aria-label={persisted ? 'Persisted typed research result' : 'Typed research result'}>
    <div className="research-result-heading"><strong>Typed result · {payload.state}</strong>{payload.fixtureLabel && <span className="badge">Synthetic fixture · {payload.fixtureLabel}</span>}</div>
    {payload.focus && <span role="status">Focus: {payload.focus}</span>}
    <p className="research-result-reason">{payload.reason}</p>
    {payload.conclusion && <p>{payload.conclusion}</p>}
    {instrumentRefs.length > 0 && <section aria-label="Canonical instruments"><h4>Canonical instruments</h4><ul>{instrumentRefs.map(instrument => <li key={instrument}><code>{instrument}</code></li>)}</ul></section>}
    {findings.length > 0 && <section aria-label="Key findings"><h4>Key findings</h4><ul>{findings.map(finding => <li key={finding.title}><b>{finding.title}:</b> {finding.detail}</li>)}</ul></section>}
    {scenarios.length > 0 && <section aria-label="Scenarios"><h4>Scenarios</h4><ul>{scenarios.map(scenario => <li key={scenario.title}><b>{scenario.title}:</b> {scenario.detail}</li>)}</ul></section>}
    {artifactRefs.length > 0 && <section aria-label="Artifact references"><h4>Artifact references</h4><ul>{artifactRefs.map(artifact => <li key={artifact}><code>{artifact}</code></li>)}</ul></section>}
    {payload.focus === 'CRYPTO_SPOT' && <section className="research-spot" aria-label="Crypto spot venue evidence"><h4>Venue evidence</h4>{spotVenues.length === 0 && <p>No venue evidence is available.</p>}{spotVenues.map(venue => <article className="research-venue" key={venue.venue}><div className="research-venue-heading"><strong>{venue.venue}</strong><span className="badge" role="status">{venue.state}{venue.selected ? ' · SELECTED' : ''}</span></div>{venue.bid && <span>Bid: {venue.bid}</span>}{venue.ask && <span>Ask: {venue.ask}</span>}{venue.spread && <span>Spread: {venue.spread}</span>}{venue.depth && <span>Depth: {venue.depth}</span>}{venue.quoteAge && <span>Quote age: {venue.quoteAge}</span>}<small>Source: {venue.provenance.sourceId} · {venue.provenance.provider} · {venue.provenance.status} · received {venue.provenance.receivedTimestamp} · freshness {venue.provenance.freshness} · quality {venue.provenance.quality}</small>{venue.limitation && <p>Limit: {venue.limitation}</p>}</article>)}</section>}
    {evidence.length > 0 && <section aria-label="Evidence provenance"><h4>Evidence provenance</h4>{evidence.map(source => <article className="research-evidence" key={`${source.sourceId}:${source.receivedTimestamp}`}><small>Source: {source.sourceId} · {source.provider} · {source.status} · received {source.receivedTimestamp}{source.providerTimestamp ? ` · provider ${source.providerTimestamp}` : ''} · freshness {source.freshness} · quality {source.quality}</small>{source.limitation && <p>Limit: {source.limitation}</p>}</article>)}</section>}
    {limitations.length > 0 && <section aria-label="Limitations"><h4>Limitations</h4><ul>{limitations.map(limitation => <li key={limitation}>{limitation}</li>)}</ul></section>}
    {agentMode === 'TRADE' && <div className="research-trade-cta"><button type="button" disabled>Review read-only proposal</button><small>Trade mode only; this card does not create or submit an order.</small></div>}
    <code data-research-marker tabIndex={0} aria-label="Research result marker">{result.marker}</code>
    <small>Request hash: {result.requestHash}</small>
    <small>Context refs: {formatContextRefs(result.contextRefs)}</small>
  </section>;
}

function TurnComposer({ thread, model, runtime, initialContexts = [], onContextsConsumed }: { thread: Thread; model?: ModelState; runtime?: RuntimeStatus; initialContexts?: ThreadContextRef[]; onContextsConsumed?: () => void }) {
  const accounts = useQuery({ queryKey: ['accounts', thread.workspaceId, 'turn-composer'], queryFn: () => request('account.list', { workspaceId: thread.workspaceId }) });
  const contextCatalog = useQuery({ queryKey: ['context-catalog', thread.workspaceId], queryFn: () => request('context.catalog', { workspaceId: thread.workspaceId }), retry: false });
  const queryClient = useQueryClient();
  const [message, setMessage] = useState('');
  const [mode, setMode] = useState<AgentMode>(thread.defaultAgentMode);
  const [execution, setExecution] = useState<ExecutionContext>(thread.defaultExecutionContext);
  const [accountId, setAccountId] = useState(thread.accountId ?? '');
  const [researchToolId, setResearchToolId] = useState<ResearchToolId>('public_market_read');
  const [researchFocus, setResearchFocus] = useState<ResearchFocus>('GENERAL');
  const [pendingContexts, setPendingContexts] = useState<ThreadContextRef[]>(mergeContextRefs(thread.linkedContexts, initialContexts));
  const [researchPreview, setResearchPreview] = useState<{ invocation: ResearchToolInvocation; result: ResearchToolResult }>();
  const [researchBusy, setResearchBusy] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  useEffect(() => {
    setMode(thread.defaultAgentMode);
    setExecution(thread.defaultExecutionContext);
    setAccountId(thread.accountId ?? '');
    setResearchToolId('public_market_read');
    setResearchFocus('GENERAL');
    setPendingContexts(mergeContextRefs(thread.linkedContexts, initialContexts));
    setResearchPreview(undefined);
    setError(undefined);
  }, [thread.threadId, thread.stateVersion, initialContexts]);
  useEffect(() => {
    setResearchPreview(undefined);
  }, [mode, execution, accountId, pendingContexts, researchToolId, researchFocus]);
  const selectedModel = thread.model ?? routeAsThreadModel(verifiedDefaultRoute(model));
  const capability = useQuery({
    queryKey: ['agent-capability', thread.workspaceId, mode, execution, accountId, pendingContexts],
    queryFn: () => request('agent.capabilities', {
      workspaceId: thread.workspaceId,
      agentMode: mode,
      executionContext: execution,
      ...(accountId ? { accountId } : {}),
      attachedContexts: pendingContexts,
    }),
    retry: false,
  });
  const selectedResearchTool = capability.data?.researchTools.find(tool => tool.id === researchToolId)
    ?? capability.data?.researchTools[0];
  const ready = runtimeReady(runtime) && Boolean(selectedModel) && Boolean(capability.data) && !capability.isError;
  const previewResearch = async () => {
    const definition = selectedResearchTool;
    if (!definition || researchBusy || busy) return;
    const query = message.trim() || 'Preview typed research context';
    setResearchBusy(true); setError(undefined);
    try {
      const invocation: ResearchToolInvocation = { toolId: definition.id, focus: researchFocus, query };
      const result = await request('research.run', {
        workspaceId: thread.workspaceId,
        agentMode: mode,
        executionContext: execution,
        ...(accountId ? { accountId } : {}),
        focus: researchFocus,
        attachedContexts: pendingContexts,
        toolId: invocation.toolId,
        query: invocation.query,
      });
      setResearchPreview({ invocation, result });
    } catch (failure) { setError(explainError(failure)); }
    finally { setResearchBusy(false); }
  };
  const send = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready || !message.trim() || busy) return;
    setBusy(true); setError(undefined);
    const payload: TurnStart = {
      workspaceId: thread.workspaceId,
      threadId: thread.threadId,
      expectedStateVersion: thread.stateVersion,
      message: message.trim(),
      agentMode: mode,
      executionContext: execution,
      attachedContexts: pendingContexts,
      ...(accountId ? { accountId } : {}),
      ...(selectedModel ? { model: selectedModel } : {}),
      ...(researchPreview ? { researchInvocation: researchPreview.invocation, researchResult: researchPreview.result } : {}),
    };
    try {
      await request('turn.start', payload);
      await queryClient.invalidateQueries({ queryKey: ['thread', thread.threadId] });
      setMessage('');
      setResearchPreview(undefined);
      onContextsConsumed?.();
    } catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };
  return <section className="composer turn-composer" aria-labelledby="turn-composer-title">
    <div className="composer-heading"><div><h3 id="turn-composer-title">Ask this Thread</h3><p className="muted">Choose context for this Turn; the saved Thread defaults stay unchanged.</p></div><span className="badge readonly">No financial approval</span></div>
    <form onSubmit={send}>
      <label className="field">Request<textarea aria-label="Turn request" value={message} onChange={event => { setMessage(event.target.value); setResearchPreview(undefined); }} maxLength={100000} placeholder="Ask a read-only question…" disabled={busy} /></label>
      <div className="thread-picker-grid">
        <label className="field">Agent mode<select value={mode} onChange={event => setMode(event.target.value as AgentMode)} disabled={busy}>{modeOptions.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}</select></label>
        <label className="field">Execution context<select value={execution} onChange={event => setExecution(event.target.value as ExecutionContext)} disabled={busy}>{executionOptions.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}</select></label>
        <label className="field">Account<select value={accountId} onChange={event => setAccountId(event.target.value)} disabled={busy || accounts.isPending || contextCatalog.isPending || contextCatalog.isError}><option value="">No account selected</option>{accounts.data?.accounts.map(account => { const available = catalogAccountAvailable(contextCatalog.data, account.connectionId); return <option key={account.connectionId} value={account.connectionId} disabled={!available}>{accountOptionLabel(account, mode, available)}</option>; })}</select></label>
      </div>
      <ContextPicker workspaceId={thread.workspaceId} pending={pendingContexts} onAttach={contexts => { setPendingContexts(contexts); setResearchPreview(undefined); }} mode={mode} />
      <div className="composer-context"><span className="badge">Mode: {mode}</span><span className="badge">Execution: {execution}</span><span className="muted">Model: {selectedModel ? `${selectedModel.provider} · ${selectedModel.modelId}` : 'Verified route required'}</span><CapabilitySummary decision={capability.data} loading={capability.isPending} error={capability.error} /></div>
      {(capability.data?.researchTools?.length ?? 0) > 0 && <section className="research-preview" aria-label="Typed research result"><label className="field">Research tool<select value={selectedResearchTool?.id ?? ''} onChange={event => { setResearchToolId(event.target.value as ResearchToolId); setResearchPreview(undefined); }} disabled={busy}>{capability.data?.researchTools.map(tool => <option key={tool.id} value={tool.id}>{tool.label}</option>)}</select></label><label className="field">Research focus<select value={researchFocus} onChange={event => { setResearchFocus(event.target.value as ResearchFocus); setResearchPreview(undefined); }} disabled={busy}><option value="GENERAL">General</option><option value="EQUITY">Equities</option><option value="CRYPTO_SPOT">Crypto spot</option></select></label><button type="button" onClick={() => void previewResearch()} disabled={!ready || busy || researchBusy}>{researchBusy ? 'Preparing typed result…' : 'Preview typed research result'}</button>{researchPreview && <div role="status"><ResearchResultCard result={researchPreview.result} agentMode={mode} /></div>}</section>}
      {!runtimeReady(runtime) && <p className="form-hint">{runtime?.modelAvailable === false ? 'Model gateway is unavailable; the draft remains local until it is ready.' : 'Codex App Server is unavailable; the draft remains local until the runtime is ready.'}</p>}
      {!selectedModel && <p className="form-hint">Choose and verify a model route before sending.</p>}
      {error && <p className="error-text" role="alert">{error}</p>}
      <div className="composer-footer"><span>Context references: {pendingContexts.length}</span><button className="primary" type="submit" disabled={!ready || busy || !message.trim()}>{busy ? 'Running…' : 'Send'}</button></div>
    </form>
  </section>;
}

function TimelineItem({ item, contextRefs, agentMode }: { item: ThreadItem; contextRefs: ThreadContextRef[]; agentMode: AgentMode }) {
  const result = item.researchResult;
  return <article className={`timeline-item timeline-${item.status.toLowerCase()}`} data-item-status={item.status}>
    <div className="timeline-item-heading"><strong>{item.itemType.replaceAll('_', ' ')}</strong><span className="badge">{item.status}</span></div>
    {item.sourceId && <small className="timeline-item-provenance">Source: {item.sourceId} · Context refs: {formatContextRefs(contextRefs)}</small>}
    <p>{item.content || 'Waiting for stream content…'}</p>
    {result && <ResearchResultCard result={result} persisted agentMode={agentMode} />}
  </article>;
}

function TurnTimeline({ turn, index, busy, onCancel, onRetry }: { turn: ThreadTurn; index: number; busy: boolean; onCancel: () => void; onRetry: () => void }) {
  const attempt = turn.providerAttempts[turn.providerAttempts.length - 1];
  return <article className="turn-timeline" aria-labelledby={`turn-${turn.turnId}`}>
    <div className="turn-heading"><div><h3 id={`turn-${turn.turnId}`}>Turn {index + 1}</h3><small>{turn.snapshot.agentMode} · {turn.snapshot.executionContext}</small></div><div className="turn-heading-actions"><span className="badge" role="status" aria-label={`Turn ${index + 1} status`} aria-live="polite">{turn.status}</span>{turn.status === 'RUNNING' && <button type="button" onClick={onCancel} disabled={busy} aria-label={`Cancel Turn ${index + 1}`}>{busy ? 'Cancelling…' : 'Cancel'}</button>}{['FAILED', 'CANCELLED', 'INTERRUPTED'].includes(turn.status) && <button type="button" onClick={onRetry} disabled={busy} aria-label={`Retry Turn ${index + 1}`}>{busy ? 'Retrying…' : 'Retry'}</button>}</div></div>
    {turn.cancelRequestedAt && turn.status === 'RUNNING' && <p className="form-hint" role="status">Cancellation requested…</p>}
    <div className="turn-provenance"><span>Model: {turn.snapshot.model ? `${turn.snapshot.model.provider} · ${turn.snapshot.model.modelId}` : 'Unavailable'}</span><span>Account: {turn.snapshot.accountId ? `${turn.snapshot.accountId} · ${turn.snapshot.accountEnvironment ?? 'environment unavailable'}` : 'None'}</span><span>Capability: {turn.snapshot.capabilityLevel}</span><span>Context: {turn.snapshot.attachedContexts.length ? turn.snapshot.attachedContexts.map(context => `${context.kind}:${context.id}#${context.hash}`).join(', ') : 'None'}</span></div>
    <div className="timeline-items">{turn.items.map(item => <TimelineItem key={item.itemId} item={item} contextRefs={turn.snapshot.attachedContexts} agentMode={turn.snapshot.agentMode} />)}</div>
    {attempt && <p className="turn-attempt" data-provider-outcome={attempt.outcome}>Provider attempt: {attempt.provider} · {attempt.modelId} · {attempt.outcome}{attempt.errorCode ? ` · ${attempt.errorCode}` : ''}</p>}
  </article>;
}

function ThreadDetail({ threadId, model, runtime, initialContexts = [], onContextsConsumed }: { threadId: string; model?: ModelState; runtime?: RuntimeStatus; initialContexts?: ThreadContextRef[]; onContextsConsumed?: () => void }) {
  const queryClient = useQueryClient();
  const projection = useDomainProjection('thread', threadId, fromThreadSnapshot);
  const [actionTurnId, setActionTurnId] = useState<string>();
  const [actionError, setActionError] = useState<string>();
  if (projection.error) return <div className="error-banner" role="alert"><div><strong>Thread needs attention</strong><p>{explainError(projection.error)}</p></div><button type="button" onClick={() => void projection.reload()}>Reload thread</button></div>;
  const thread = projection.data;
  if (!thread) return <p role="status">Loading thread…</p>;
  const act = async (turn: ThreadTurn, command: 'turn.cancel' | 'turn.retry') => {
    setActionTurnId(turn.turnId); setActionError(undefined);
    try {
      const payload = command === 'turn.cancel'
        ? { workspaceId: thread.workspaceId, threadId: thread.threadId, turnId: turn.turnId, expectedStateVersion: thread.stateVersion } satisfies TurnCancel
        : { workspaceId: thread.workspaceId, threadId: thread.threadId, turnId: turn.turnId, expectedStateVersion: thread.stateVersion } satisfies TurnRetry;
      await request(command, payload);
      await queryClient.invalidateQueries({ queryKey: ['thread', thread.threadId] });
      await projection.reload();
    } catch (failure) { setActionError(explainError(failure)); }
    finally { setActionTurnId(undefined); }
  };
  return <section className="card thread-detail" aria-labelledby="thread-detail-title">
    <div className="account-heading"><div><h2 id="thread-detail-title">{thread.title}</h2><p className="muted">Thread {thread.threadId}</p></div><span className="badge">{thread.status}</span></div>
    <div className="composer-context"><span className="badge">Mode: {thread.defaultAgentMode}</span><span className="badge">Execution: {thread.defaultExecutionContext}</span><span className="muted">Account: {thread.accountId ?? 'None selected'}</span><span className="muted">Model: {thread.model ? `${thread.model.provider} · ${thread.model.modelId}` : 'Not selected'}</span></div>
    <dl className="thread-provenance"><div><dt>Created</dt><dd><time dateTime={thread.createdAt}>{new Date(thread.createdAt).toLocaleString()}</time></dd></div><div><dt>Updated</dt><dd><time dateTime={thread.updatedAt}>{new Date(thread.updatedAt).toLocaleString()}</time></dd></div><div><dt>Context references</dt><dd>{thread.linkedContexts.length ? thread.linkedContexts.map(context => `${context.kind}:${context.id}`).join(', ') : 'None'}</dd></div></dl>
    <TurnComposer thread={thread} model={model} runtime={runtime} initialContexts={initialContexts} onContextsConsumed={onContextsConsumed} />
    {actionError && <p className="error-text" role="alert">{actionError}</p>}
    {thread.turns?.length ? <section className="thread-timeline" aria-label="Turn timeline">{thread.turns.map((turn, index) => <TurnTimeline key={turn.turnId} turn={turn} index={index} busy={actionTurnId === turn.turnId} onCancel={() => void act(turn, 'turn.cancel')} onRetry={() => void act(turn, 'turn.retry')} />)}</section> : <div className="empty-activity" role="status"><h3>Thread timeline</h3><p>No turns have started. Send a request to begin the read-only timeline.</p></div>}
  </section>;
}

function ThreadsPage({ workspaceId, model, runtime, selectedThreadId, onSelect, onCreated, newThreadContexts = [], selectedThreadContexts = [], onContextsConsumed }: { workspaceId: string; model?: ModelState; runtime?: RuntimeStatus; selectedThreadId?: string; onSelect: (threadId: string) => void; onCreated: (thread: Thread) => void; newThreadContexts?: ThreadContextRef[]; selectedThreadContexts?: ThreadContextRef[]; onContextsConsumed?: () => void }) {
  return <><div className="page-heading"><h1>Threads</h1><p>Local history restores each Thread's own defaults and context.</p></div><div className="threads-layout"><section className="card threads-list-card"><h2>History</h2><ThreadHistory workspaceId={workspaceId} selectedThreadId={selectedThreadId} onSelect={onSelect} /></section><section className="threads-main"><ThreadComposer workspaceId={workspaceId} model={model} onCreated={onCreated} initialContexts={newThreadContexts} />{selectedThreadId && <ThreadDetail threadId={selectedThreadId} model={model} runtime={runtime} initialContexts={selectedThreadContexts} onContextsConsumed={onContextsConsumed} />}</section></div></>;
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

function TimeHealth({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const actionRef = useRef<HTMLButtonElement>(null);
  const restoreFocus = useRef(false);
  const status = useQuery({
    queryKey: ['time-status', workspaceId],
    queryFn: () => request('time.status', { workspaceId }),
    refetchInterval: 10_000,
  });
  const current = status.data as TimeStatus | undefined;
  const revalidate = async () => {
    restoreFocus.current = document.activeElement === actionRef.current;
    setBusy(true); setError(undefined);
    try {
      const refreshed = await request('time.revalidate', { workspaceId });
      queryClient.setQueryData(['time-status', workspaceId], refreshed);
    } catch (failure) {
      setError(explainError(failure));
    } finally {
      setBusy(false);
    }
  };
  useEffect(() => {
    if (!busy && restoreFocus.current) {
      restoreFocus.current = false;
      actionRef.current?.focus();
    }
  }, [busy]);
  if (status.isPending) return <p role="status">Checking trusted time…</p>;
  if (status.isError || !current) return <div className="error-banner" role="alert"><div><strong>Trusted time needs attention</strong><p>{explainError(status.error)}</p></div><button type="button" onClick={() => void status.refetch()}>Reload time status</button></div>;
  return <section className="time-health" aria-labelledby="time-health-title">
    <div className="account-heading"><div><h3 id="time-health-title">Trusted time</h3><p className="muted">Live freshness and TTL decisions use this Control Plane reading.</p></div><span className={`badge time-confidence-${current.confidence.toLowerCase()}`} role="status" aria-live="polite">{current.confidence.replaceAll('_', ' ')}</span></div>
    <p className="notice" role="status" aria-live="polite">{current.reason}</p>
    <dl className="health-grid"><div><dt>Wall clock</dt><dd><time dateTime={current.wallClock}>{current.wallClock}</time></dd></div><div><dt>Monotonic reading</dt><dd>{current.monotonicMs} ms</dd></div><div><dt>Provider offset</dt><dd>{current.providerOffsetMs == null ? 'Unavailable' : `${current.providerOffsetMs} ms`}</dd></div><div><dt>Observed</dt><dd><time dateTime={current.observedAt}>{current.observedAt}</time></dd></div></dl>
    <button ref={actionRef} type="button" className="primary" onClick={() => void revalidate()} disabled={busy}>{busy ? 'Synchronizing…' : 'Synchronize time'}</button>
    {error && <p className="error-text" role="alert">{error}</p>}
  </section>;
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
  const [newThreadScreenerContexts, setNewThreadScreenerContexts] = useState<ThreadContextRef[]>([]);
  const [currentThreadScreenerContexts, setCurrentThreadScreenerContexts] = useState<{ threadId: string; contexts: ThreadContextRef[] }>();
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
  useEffect(() => { setSelectedThreadId(undefined); setNewThreadScreenerContexts([]); setCurrentThreadScreenerContexts(undefined); }, [workspace?.workspaceId]);
  const navigate = (destination: Page) => {
    setPage(destination); setSetup(false); setWorkspacePicker(false);
    if (destination === 'New Thread') setSelectedThreadId(undefined);
    document.querySelectorAll('details[open]').forEach(details => details.removeAttribute('open'));
  };
  const selectThread = (threadId: string) => { setSelectedThreadId(threadId); setNewThreadScreenerContexts([]); setCurrentThreadScreenerContexts(previous => previous?.threadId === threadId ? previous : undefined); setPage('Threads'); setSetup(false); setWorkspacePicker(false); };
  const createdThread = (thread: Thread) => { setNewThreadScreenerContexts([]); setCurrentThreadScreenerContexts(undefined); setSelectedThreadId(thread.threadId); setPage('Threads'); };
  const attachScreenerContexts = (contexts: ThreadContextRef[], target: 'new' | 'current') => {
    setSetup(false); setWorkspacePicker(false);
    if (target === 'current' && selectedThreadId) {
      setCurrentThreadScreenerContexts({ threadId: selectedThreadId, contexts });
      setNewThreadScreenerContexts([]);
      setPage('Threads');
    } else {
      setNewThreadScreenerContexts(contexts);
      setCurrentThreadScreenerContexts(undefined);
      setSelectedThreadId(undefined);
      setPage('New Thread');
    }
  };
  const submit = async (options: OpenWorkspace) => {
    try { await state.opening.mutateAsync(options); setSelectedThreadId(undefined); setWorkspacePicker(false); setSetup(true); setPage('New Thread'); } catch { /* Render the canonical error below. */ }
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
              {workspace && <ThreadComposer workspaceId={workspace.workspaceId} model={model} onCreated={createdThread} initialContexts={newThreadScreenerContexts} />}
              <div className="notice model-notice"><div><strong>{modelReady ? 'Agent turns unavailable' : defaultModelRoute ? 'Model gateway unavailable' : 'Connect a model provider'}</strong><p>{modelState.reason}</p></div><button onClick={() => navigate('Settings')}>Providers &amp; Models</button></div>
              <div className="empty-activity"><h2>Thread activity</h2><p>No agent turns have started in this workspace.</p></div>
            </>}
            {page === 'Threads' && workspace && <ThreadsPage workspaceId={workspace.workspaceId} model={model} runtime={state.runtime.data} selectedThreadId={selectedThreadId} onSelect={selectThread} onCreated={createdThread} newThreadContexts={newThreadScreenerContexts} selectedThreadContexts={currentThreadScreenerContexts?.threadId === selectedThreadId ? currentThreadScreenerContexts?.contexts ?? [] : []} onContextsConsumed={() => setCurrentThreadScreenerContexts(undefined)} />}
            {page === 'Settings' && <>
              <div className="page-heading"><h1>Settings</h1><p>Manage your local workspace and connected services.</p></div>
              <div className="settings-tabs" role="group" aria-label="Settings sections">{['Providers & Models', 'Risk & Limits', 'Data & Storage', 'Account Health', 'Appearance', 'About'].map(tab =>
                <button key={tab} aria-pressed={settingsTab === tab} onClick={() => setSettingsTab(tab)}>{tab}</button>)}</div>
              <section className="card settings-section"><h2>{settingsTab}</h2>
                {workspace && (settingsTab === 'Providers & Models' || settingsTab === 'Account Health') && <Accounts key={workspace.workspaceId} workspaceId={workspace.workspaceId} healthOnly={settingsTab === 'Account Health'} />}
                {workspace && settingsTab === 'Account Health' && <TimeHealth workspaceId={workspace.workspaceId} />}
                {workspace && settingsTab === 'Providers & Models' && <Models key={`models:${workspace.workspaceId}`} workspaceId={workspace.workspaceId} model={model} modelError={modelProjection.error} reloadModel={modelProjection.reload} />}
                {settingsTab === 'Providers & Models' || settingsTab === 'About' ? <>
                  <p className="muted">{settingsTab === 'About' ? 'TradeX 0.1.0 · local desktop workspace' : modelReady ? 'A verified model route is available; agent turns remain disabled until Codex App Server is configured.' : modelState.reason}</p>
                  <ul className="component-list">{state.runtime.data?.components.map(component => <li key={component.id}><div><strong>{component.id === 'cliproxyapi' ? 'CLIProxyAPI' : component.id === 'codex' ? 'Codex App Server' : component.id === 'control-plane' ? 'Control Plane' : 'Order Gateway'}</strong><p>{component.message}</p></div><span className="badge">{state.runtime.isError ? 'Unavailable' : component.status === 'RUNNING' ? 'Available' : component.status.replaceAll('_', ' ')}</span></li>)}</ul>
                  <button onClick={() => { void state.runtime.refetch(); }} disabled={state.runtime.isFetching}>Refresh runtime status</button>
                </> : settingsTab === 'Risk & Limits' && workspace ? <RiskSettings workspace={workspace} risk={risk} /> : settingsTab === 'Data & Storage' && workspace ? <><p className="muted">Local workspace folder</p><p className="path">{workspace.path}</p><button onClick={() => { setWorkspacePicker(true); setSetup(false); }}>Open another workspace</button><DataSources workspaceId={workspace.workspaceId} /></> :
                  <p className="muted">{settingsTab === 'Account Health' ? 'Connection, authentication, stream, reconciliation and execution are separate checks.' : 'The workspace currently uses the RevC light theme.'}</p>}
              </section>
            </>}
            {page === 'Accounts' && <><div className="page-heading"><h1>Accounts</h1><p>Connect and inspect your provider accounts.</p></div>{workspace ? <Accounts key={workspace.workspaceId} workspaceId={workspace.workspaceId} /> : <p>Open a workspace to manage accounts.</p>}</>}
            {page === 'Markets' && (workspace ? <Markets workspaceId={workspace.workspaceId} hasCurrentThread={Boolean(selectedThreadId)} onAttachContexts={attachScreenerContexts} onOpenDataSources={() => { setSettingsTab('Data & Storage'); navigate('Settings'); }} /> : <><div className="page-heading"><h1>Markets</h1><p>Search canonical instruments and inspect source-backed market availability.</p></div><section className="card empty-page"><h2>Open a workspace to browse markets</h2><p>Market catalogs and source status are scoped to a local workspace.</p><button type="button" onClick={() => { setPage('New Thread'); setWorkspacePicker(true); }}>Open workspace</button></section></>)}
            {page === 'Watchlists' && (workspace ? <Watchlists workspaceId={workspace.workspaceId} /> : <><div className="page-heading"><h1>Watchlists</h1><p>Keep ordered canonical instruments in a local workspace.</p></div><section className="card empty-page"><h2>Open a workspace to manage watchlists</h2><p>Watchlists are stored in the selected local workspace.</p><button type="button" onClick={() => { setPage('New Thread'); setWorkspacePicker(true); }}>Open workspace</button></section></>)}
            {(page === 'Strategies' || page === 'Artifacts') && <>
              <div className="page-heading"><h1>{page}</h1></div>
              <section className="card empty-page"><h2>{page === 'Strategies' ? 'No saved strategies' : 'No artifacts'}</h2>
                <p>This workflow is not available in this build. Your local workspace is ready for the next setup steps.</p>
                <button onClick={() => navigate('Settings')}>Open settings</button>
              </section>
            </>}
          </section>{workspace && <WorkspaceDetails workspace={workspace} />}</div>}
      </main>
    </div>
  </div>;
}
