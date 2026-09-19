import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { AdjustmentStatus, CorporateAction, FilterSpec, Instrument, MarketDataStatus, MarketDetail, MarketSession, MarketState, RankSpec, ScreenerAttachment, ScreenerDefinition, ScreenerDirection, ScreenerFeature, ScreenerFeatureField, ScreenerLibrary, ScreenerOperator, ScreenerPredicateField, ScreenerResult, ScreenerResultState, ScreenerUniverse, ThreadContextRef } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';
import { ErrorRecoveryPanel } from './ErrorRecoveryPanel.tsx';

const statusLabel: Record<MarketDataStatus, string> = {
  AVAILABLE: 'Available',
  UNAVAILABLE: 'Unavailable',
  BLOCKED_EXTERNAL: 'Blocked by external setup',
  UNVERIFIED: 'Unverified',
};

const sessionLabel: Record<MarketSession, string> = {
  OPEN: 'Open',
  CLOSED: 'Closed',
  EXTENDED_HOURS: 'Extended hours',
  HALTED: 'Halted',
  MAINTENANCE: 'Maintenance',
  SUSPENDED: 'Suspended',
  DEGRADED: 'Degraded',
  UNKNOWN: 'Unknown',
};

const adjustmentLabel: Record<AdjustmentStatus, string> = {
  ADJUSTED: 'Adjusted',
  UNADJUSTED: 'Unadjusted',
  UNKNOWN: 'Unknown',
  UNAVAILABLE: 'Unavailable',
};

function InstrumentRow({ instrument, selected, onSelect }: { instrument: Instrument; selected: boolean; onSelect: () => void }) {
  return <button type="button" className="market-row" aria-current={selected ? 'true' : undefined} onClick={onSelect}>
    <span className="market-row-main"><strong>{instrument.symbol}</strong><small>{instrument.displayName}</small></span>
    <span className="market-row-meta"><span className="badge">{instrument.assetClass === 'EQUITY' ? 'US equity' : 'Crypto spot'}</span><small className="identity">{instrument.instrumentId}</small></span>
  </button>;
}

function MarketStatePanel({ state, adjustmentStatus, actions, onOpenDataSources }: { state: MarketState; adjustmentStatus: AdjustmentStatus; actions: CorporateAction[]; onOpenDataSources: () => void }) {
  return <>
    <section className={`market-session market-session-${state.session.toLowerCase()}`} aria-labelledby="market-session-title" role="status" aria-live="polite">
      <div className="market-panel-heading"><div><p className="eyebrow">Market session</p><h3 id="market-session-title" tabIndex={-1}>{sessionLabel[state.session]}</h3></div><span className="badge">{state.timeConfidence}</span></div>
      <p>{state.reason}</p>
      <ErrorRecoveryPanel code={state.session === 'CLOSED' ? 'MARKET_CLOSED' : state.session === 'HALTED' ? 'INSTRUMENT_HALTED' : undefined} onAction={() => document.getElementById('market-session-title')?.focus()} />
      <dl className="market-state-details"><div><dt>Venue</dt><dd>{state.venue}</dd></div><div><dt>Source status</dt><dd>{statusLabel[state.sourceStatus]}</dd></div><div><dt>Next open</dt><dd>{state.nextOpen ?? 'Unavailable'}</dd></div><div><dt>Next close</dt><dd>{state.nextClose ?? 'Unavailable'}</dd></div><div><dt>Calendar version</dt><dd>{state.calendarVersion ?? 'Unavailable'}</dd></div><div><dt>Provider time</dt><dd>{state.providerTime ?? 'Unavailable'}</dd></div><div><dt>Observed</dt><dd>{state.observedAt}</dd></div></dl>
      {(state.sourceStatus === 'BLOCKED_EXTERNAL' || state.sourceStatus === 'UNAVAILABLE') && <button type="button" onClick={onOpenDataSources}>Review calendar source</button>}
    </section>
    <section className="market-actions-panel" aria-labelledby="corporate-actions-title">
      <div className="market-panel-heading"><div><p className="eyebrow">Corporate actions</p><h3 id="corporate-actions-title">History adjustment: {adjustmentLabel[adjustmentStatus]}</h3></div><span className="badge">{actions.length} recorded</span></div>
      {actions.length ? <ul className="corporate-actions">{actions.map(action => <li key={action.actionId}><strong>{action.actionType.replaceAll('_', ' ')}</strong><span>{action.description}</span><small>Effective {action.effectiveAt}{action.announcedAt ? ` · Announced ${action.announcedAt}` : ''}{action.sourceId ? ` · Source ${action.sourceId}` : ''} · Adjustment {adjustmentLabel[action.adjustmentStatus]}</small></li>)}</ul> : <p className="muted">No authoritative corporate-action records are available. Historical data is not marked adjusted.</p>}
    </section>
  </>;
}

function Detail({ detail, onBack, onOpenDataSources }: { detail: MarketDetail; onBack: () => void; onOpenDataSources: () => void }) {
  const { instrument } = detail;
  return <section className="card market-detail" aria-labelledby="market-detail-title">
    <div className="market-detail-heading"><div><p className="eyebrow">Instrument detail</p><h2 id="market-detail-title">{instrument.displayName}</h2><p className="identity">{instrument.instrumentId}</p></div><button type="button" onClick={onBack}>Back to results</button></div>
    <dl className="market-identity"><div><dt>Symbol</dt><dd>{instrument.symbol}</dd></div><div><dt>Asset class</dt><dd>{instrument.assetClass === 'EQUITY' ? 'US equity' : 'Crypto spot'}</dd></div><div><dt>Venue</dt><dd>{instrument.exchange ?? 'Provider venue selected at fetch'}</dd></div><div><dt>Currency</dt><dd>{instrument.currency}</dd></div><div><dt>Access tier</dt><dd>{detail.tier}</dd></div></dl>
    <div className={`market-status market-status-${detail.status.toLowerCase()}`} role="status"><strong>{statusLabel[detail.status]}</strong><p>{detail.availabilityReason}</p><small>Source: {detail.sourceId ?? 'No source selected'}</small></div>
    <MarketStatePanel state={detail.marketState} adjustmentStatus={detail.adjustmentStatus} actions={detail.corporateActions} onOpenDataSources={onOpenDataSources} />
    {detail.snapshot ? <section className="market-quote" aria-label="Market quote"><h3>Quote</h3><p className="market-price">{detail.snapshot.lastPrice ?? 'No last price'}</p><p className="muted">{detail.snapshot.provenance.entitlement} · {detail.snapshot.provenance.freshness}</p><dl className="market-provenance"><div><dt>Provider time</dt><dd>{detail.snapshot.provenance.providerTimestamp}</dd></div><div><dt>Received</dt><dd>{detail.snapshot.provenance.receivedTimestamp}</dd></div><div><dt>Venue</dt><dd>{detail.snapshot.provenance.venue ?? 'Not supplied'}</dd></div></dl></section> : <div className="market-unavailable"><p className="muted">No quote or chart is shown until the selected source is entitled and the adapter returns a validated snapshot.</p><button type="button" onClick={onOpenDataSources}>Open Data &amp; Storage settings</button></div>}
  </section>;
}

const screenerFieldLabels: Record<ScreenerPredicateField, string> = {
  REVENUE_GROWTH: 'Revenue growth', ESTIMATE_REVISION: 'Estimate revision', RSI: 'RSI', PRICE_CHANGE: 'Price change',
};
const screenerOperatorLabels: Record<ScreenerOperator, string> = {
  GREATER_THAN: 'above', GREATER_OR_EQUAL: 'at least', LESS_THAN: 'below', LESS_OR_EQUAL: 'at most',
};
const screenerUniverseLabels: Record<ScreenerUniverse, string> = {
  US_EQUITIES: 'US equities', US_LARGE_CAP_TECHNOLOGY: 'US large-cap technology', CRYPTO_SPOT: 'Crypto spot',
};
const screenerRankLabels: Record<RankSpec['field'], string> = {
  QUALITY: 'Quality', REVISION_STRENGTH: 'Revision strength', MOMENTUM: 'Momentum',
};

function featureValue(candidate: NonNullable<ScreenerResult['candidates']>[number], field: ScreenerFeatureField) {
  return (candidate.features ?? []).find((feature: ScreenerFeature) => feature.field === field)?.value ?? 'Unavailable';
}

type ScreenerAttachTarget = 'new' | 'current';

function ScreenerBuilder({ workspaceId, onBack, onOpenInstrument, onAttachContexts, hasCurrentThread }: { workspaceId: string; onBack: () => void; onOpenInstrument: (instrumentId: string) => void; onAttachContexts: (contexts: ThreadContextRef[], target: ScreenerAttachTarget) => void; hasCurrentThread: boolean }) {
  const queryClient = useQueryClient();
  const library = useQuery({ queryKey: ['screeners', workspaceId], queryFn: () => request('screener.list', { workspaceId }), retry: false });
  const [naturalLanguage, setNaturalLanguage] = useState('US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.');
  const [spec, setSpec] = useState<FilterSpec>();
  const [rankSpec, setRankSpec] = useState<RankSpec>();
  const [revision, setRevision] = useState<string>();
  const [limit, setLimit] = useState(10);
  const [result, setResult] = useState<ScreenerResult>();
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [selectedSavedId, setSelectedSavedId] = useState<string>();
  const [saveName, setSaveName] = useState('');
  const [attachTarget, setAttachTarget] = useState<ScreenerAttachTarget>('new');
  const [phase, setPhase] = useState<'idle' | 'parsing' | 'running'>('idle');
  const [saveBusy, setSaveBusy] = useState(false);
  const [attachBusy, setAttachBusy] = useState(false);
  const [stale, setStale] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState<string>();

  const parse = async (edited = false) => {
    setPhase('parsing'); setError(undefined); setNotice(undefined); setResult(undefined); setSelectedIds([]);
    try {
      const response = await request('market.screen', {
        workspaceId, operation: 'PARSE', naturalLanguage, focus: 'EQUITY',
        ...(edited && spec && rankSpec ? { filterSpec: spec, rankSpec, limit } : {}),
      });
      setSpec(response.filterSpec ?? undefined); setRankSpec(response.rankSpec ?? undefined);
      setRevision(response.revision ?? undefined); setStale(response.state !== 'PARSED');
      setResult(response.state === 'PARSED' ? undefined : response);
      setPhase('idle');
      if (response.state !== 'PARSED') setError(response.availabilityReason);
    } catch (cause) { setPhase('idle'); setError(explainError(cause)); }
  };

  const run = async () => {
    if (!spec || !rankSpec || !revision || stale) return;
    setPhase('running'); setError(undefined); setNotice(undefined); setResult(undefined); setSelectedIds([]);
    try {
      const response = await request('market.screen', {
        workspaceId, operation: 'RUN', naturalLanguage, focus: 'EQUITY', filterSpec: spec, rankSpec, revision, limit,
      });
      setResult(response); setPhase('idle');
    } catch (cause) { setPhase('idle'); setError(explainError(cause)); }
  };

  const editSpec = (next: FilterSpec) => { setSpec(next); setRevision(undefined); setStale(true); setResult(undefined); setSelectedIds([]); };
  const editRank = (next: RankSpec) => { setRankSpec(next); setRevision(undefined); setStale(true); setResult(undefined); setSelectedIds([]); };
  const reopen = (saved: ScreenerLibrary['screeners'][number]) => {
    setNaturalLanguage(saved.definition.naturalLanguage);
    setSpec(saved.definition.filterSpec);
    setRankSpec(saved.definition.rankSpec);
    setRevision(saved.definition.revision);
    setLimit(saved.definition.limit);
    setSelectedSavedId(saved.screenerId);
    setSaveName(saved.name);
    setResult(undefined);
    setSelectedIds([]);
    setStale(false);
    setError(undefined);
    setNotice(`Reopened ${saved.name}; run it again to obtain current source-gated results.`);
  };
  const save = async () => {
    if (!spec || !rankSpec || !revision || stale || !library.data || !saveName.trim() || saveBusy) return;
    const definition: ScreenerDefinition = { naturalLanguage, focus: 'EQUITY', filterSpec: spec, rankSpec, revision, limit };
    const state: ScreenerResultState = result?.state ?? 'PARSED';
    setSaveBusy(true); setError(undefined); setNotice(undefined);
    try {
      const response = selectedSavedId
        ? await request('screener.update', { workspaceId, screenerId: selectedSavedId, name: saveName.trim(), definition, state, expectedStateVersion: library.data.stateVersion })
        : await request('screener.save', { workspaceId, name: saveName.trim(), definition, state, expectedStateVersion: library.data.stateVersion });
      queryClient.setQueryData(['screeners', workspaceId], response);
      const saved = response.screeners.find(item => selectedSavedId ? item.screenerId === selectedSavedId : item.name === saveName.trim());
      if (saved) { setSelectedSavedId(saved.screenerId); setSaveName(saved.name); }
      setNotice(`${selectedSavedId ? 'Updated' : 'Saved'} screener ${saved?.name ?? saveName.trim()}.`);
    } catch (cause) { setError(explainError(cause)); }
    finally { setSaveBusy(false); }
  };
  const attach = async () => {
    if (!result?.revision || selectedIds.length === 0 || attachBusy) return;
    setAttachBusy(true); setError(undefined); setNotice(undefined);
    try {
      const selectedInstrumentIds = [selectedIds[0], ...selectedIds.slice(1)] as [string, ...string[]];
      const response: ScreenerAttachment = await request('screener.attach', { workspaceId, revision: result.revision, selectedInstrumentIds });
      onAttachContexts(response.contextRefs, attachTarget);
      setNotice(`Attached ${response.contextRefs.length} selected candidate${response.contextRefs.length === 1 ? '' : 's'} to ${attachTarget === 'new' ? 'a new Thread' : 'the current Thread next Turn'}.`);
      setSelectedIds([]);
    } catch (cause) { setError(explainError(cause)); }
    finally { setAttachBusy(false); }
  };
  const stage = result ? 5 : phase === 'running' ? 4 : phase === 'parsing' ? 2 : spec ? 3 : 1;
  return <section className="card screener-builder" aria-labelledby="screener-title">
    <div className="market-detail-heading"><div><p className="eyebrow">Market screener</p><h2 id="screener-title">Natural-language filter</h2><p className="muted">Parse first, review the typed conditions, then run the read-only candidate query.</p></div><button type="button" onClick={onBack}>Back to explorer</button></div>
    <section className="screener-library" aria-labelledby="screener-library-title"><div className="market-panel-heading"><div><p className="eyebrow">Saved definitions</p><h3 id="screener-library-title">Screener library</h3></div>{library.data && <span className="badge">{library.data.screeners.length} saved</span>}</div>{library.isPending && <p role="status">Loading saved screeners…</p>}{library.isError && <p className="error-text" role="alert">Saved screeners are unavailable.</p>}{library.data?.screeners.length ? <div className="screener-library-list" role="list" aria-label="Saved screeners">{library.data.screeners.map(saved => <article key={saved.screenerId} role="listitem" className={saved.screenerId === selectedSavedId ? 'selected' : ''}><button type="button" onClick={() => reopen(saved)}><strong>{saved.name}</strong><span className="badge">{saved.state.replaceAll('_', ' ')}</span><small>{saved.definition.revision}</small></button></article>)}</div> : library.data && <p className="muted">No saved screeners yet.</p>}</section>
    <ol className="screener-stages" aria-label="Screener stages"><li className={stage === 1 ? 'active' : ''}>1. Describe</li><li className={stage === 2 ? 'active' : ''}>2. Parse</li><li className={stage === 3 ? 'active' : ''}>3. Inspect</li><li className={stage === 4 ? 'active' : ''}>4. Run</li><li className={stage === 5 ? 'active' : ''}>5. Results</li></ol>
    <form className="screener-query" onSubmit={event => { event.preventDefault(); void parse(); }}><label htmlFor="screener-natural-language">Describe the market</label><textarea id="screener-natural-language" value={naturalLanguage} onChange={event => { setNaturalLanguage(event.target.value); setRevision(undefined); setStale(true); setResult(undefined); setSelectedIds([]); }} maxLength={4000} rows={4} /><div className="screener-actions"><button className="primary" type="submit" disabled={phase !== 'idle' || !naturalLanguage.trim()}>Parse conditions</button>{phase === 'parsing' && <span role="status">Parsing…</span>}</div></form>
    {spec && rankSpec && <section className="screener-inspection" aria-labelledby="screener-inspection-title"><div className="market-panel-heading"><div><p className="eyebrow">Review before run</p><h3 id="screener-inspection-title">FilterSpec and RankSpec</h3></div>{revision && <span className="badge">Revision ready</span>}</div><label htmlFor="screener-universe">Universe</label><select id="screener-universe" value={spec.universe} onChange={event => editSpec({ ...spec, universe: event.target.value as ScreenerUniverse })}>{Object.entries(screenerUniverseLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><div className="screener-predicates"><span className="market-facet-label">Conditions</span>{(spec.predicates ?? []).map((predicate, index) => <div className="screener-predicate" key={`${predicate.field}-${index}`}><select aria-label={`Condition ${index + 1} field`} value={predicate.field} onChange={event => { const predicates = [...(spec.predicates ?? [])]; predicates[index] = { ...predicate, field: event.target.value as ScreenerPredicateField }; editSpec({ ...spec, predicates: predicates as FilterSpec['predicates'] }); }}>{Object.entries(screenerFieldLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><select aria-label={`Condition ${index + 1} operator`} value={predicate.operator} onChange={event => { const predicates = [...(spec.predicates ?? [])]; predicates[index] = { ...predicate, operator: event.target.value as ScreenerOperator }; editSpec({ ...spec, predicates: predicates as FilterSpec['predicates'] }); }}>{Object.entries(screenerOperatorLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><input aria-label={`Condition ${index + 1} threshold`} value={predicate.threshold} onChange={event => { const predicates = [...(spec.predicates ?? [])]; predicates[index] = { ...predicate, threshold: event.target.value }; editSpec({ ...spec, predicates: predicates as FilterSpec['predicates'] }); }} maxLength={64} /></div>)}</div><div className="screener-rank"><label htmlFor="screener-rank-field">Rank by</label><select id="screener-rank-field" value={rankSpec.field} onChange={event => editRank({ ...rankSpec, field: event.target.value as RankSpec['field'] })}>{Object.entries(screenerRankLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><select aria-label="Rank direction" value={rankSpec.direction} onChange={event => editRank({ ...rankSpec, direction: event.target.value as ScreenerDirection })}><option value="DESC">Highest first</option><option value="ASC">Lowest first</option></select><label htmlFor="screener-limit">Limit</label><input id="screener-limit" type="number" min={1} max={50} value={limit} onChange={event => { setLimit(Math.max(1, Math.min(50, Number(event.target.value) || 1))); setRevision(undefined); setStale(true); setResult(undefined); setSelectedIds([]); }} /></div><div className="screener-actions"><button type="button" onClick={() => void parse(true)} disabled={phase !== 'idle'}>Recalculate revision</button><button className="primary" type="button" onClick={() => void run()} disabled={phase !== 'idle' || stale || !revision}>Run screen</button>{stale && <span role="status">Conditions changed; recalculate the revision before running.</span>}{phase === 'running' && <span role="status">Running…</span>}</div></section>}
    {spec && rankSpec && revision && !stale && library.data && <div className="screener-save"><label htmlFor="screener-name">Save name<input id="screener-name" value={saveName} onChange={event => setSaveName(event.target.value)} maxLength={80} placeholder="e.g. Growth leaders" /></label><button type="button" className="primary" onClick={() => void save()} disabled={saveBusy || !saveName.trim() || library.isPending}>{saveBusy ? 'Saving…' : selectedSavedId ? 'Save changes' : 'Save screener'}</button></div>}
    {notice && <p className="form-hint" role="status">{notice}</p>}
    {error && <p className="error-banner" role="alert">{error}</p>}
    {result && <section className={`screener-results screener-results-${result.state.toLowerCase()}`} aria-live="polite" aria-labelledby="screener-results-title"><div className="market-panel-heading"><div><p className="eyebrow">Screen result</p><h3 id="screener-results-title">{result.state.replaceAll('_', ' ')}</h3></div><span className="badge">{result.candidateCount} candidates</span></div><p>{result.availabilityReason}</p><p className="muted">Conditions: {(result.appliedConditions ?? []).join(' · ')}</p>{result.limitations?.map(limitation => <p className="muted" key={limitation}>{limitation}</p>)}{result.fixtureLabel && <p className="muted">Fixture: {result.fixtureLabel}</p>}{result.candidates?.length ? <div className="screener-candidates" role="list" aria-label="Screener candidates">{result.candidates.map(candidate => <article className="screener-candidate" role="listitem" key={candidate.instrumentId}><label className="screener-candidate-select"><input type="checkbox" aria-label={`Select ${candidate.instrumentId}`} checked={selectedIds.includes(candidate.instrumentId)} onChange={() => setSelectedIds(current => current.includes(candidate.instrumentId) ? current.filter(id => id !== candidate.instrumentId) : [...current, candidate.instrumentId])} /><span>Select</span></label><button type="button" className="screener-candidate-button" aria-label={`Open ${candidate.instrumentId} market detail`} onClick={() => onOpenInstrument(candidate.instrumentId)}><div><strong>{candidate.rank}. {candidate.symbol}</strong><small className="identity">{candidate.instrumentId}</small></div><div className="screener-feature-list"><span>Quality {featureValue(candidate, 'QUALITY')}</span><span>Revision {featureValue(candidate, 'REVISION_STRENGTH')}</span><span>Momentum {featureValue(candidate, 'MOMENTUM')}</span></div><small>Source {candidate.provenance.sourceId} · Provider {candidate.provenance.providerTimestamp ?? 'Unavailable'} · Received {candidate.provenance.receivedTimestamp} · {candidate.provenance.freshness} · {candidate.provenance.quality} · {candidate.limitation}</small></button></article>)}</div> : <p className="muted">No candidates matched the reviewed conditions.</p>}{result.candidates?.length ? <div className="screener-attach"><label htmlFor="screener-attach-target">Attach selected to<select id="screener-attach-target" value={attachTarget} onChange={event => setAttachTarget(event.target.value as ScreenerAttachTarget)}><option value="new">New Thread</option><option value="current" disabled={!hasCurrentThread}>Current Thread next Turn{hasCurrentThread ? '' : ' (select a Thread first)'}</option></select></label><button type="button" className="primary" onClick={() => void attach()} disabled={attachBusy || selectedIds.length === 0 || !result.revision}>{attachBusy ? 'Attaching…' : `Attach selected (${selectedIds.length})`}</button></div> : null}<button type="button" onClick={() => void (result.state === 'FAILED' ? parse() : run())} disabled={phase !== 'idle'}>Retry screen</button></section>}
  </section>;
}

export function Markets({ workspaceId, onOpenDataSources, onAttachContexts, hasCurrentThread }: { workspaceId: string; onOpenDataSources: () => void; onAttachContexts: (contexts: ThreadContextRef[], target: ScreenerAttachTarget) => void; hasCurrentThread: boolean }) {
  const [term, setTerm] = useState('');
  const [query, setQuery] = useState('');
  const [selectedId, setSelectedId] = useState<string>();
  const [screenerOpen, setScreenerOpen] = useState(false);
  const catalog = useQuery({ queryKey: ['market-catalog', workspaceId, query], queryFn: () => request('market.catalog', { workspaceId, query, tier: 'CENSUS' }) });
  const detail = useQuery({ queryKey: ['market-detail', workspaceId, selectedId], queryFn: () => request('market.get', { workspaceId, instrumentId: selectedId!, tier: 'HOT' }), enabled: Boolean(selectedId) });
  useEffect(() => {
    if (selectedId && catalog.data?.instruments.every(instrument => instrument.instrumentId !== selectedId)) setSelectedId(undefined);
  }, [catalog.data?.instruments, selectedId]);
  const submit = (event: FormEvent) => { event.preventDefault(); setQuery(term.trim()); };
  const error = catalog.error ?? detail.error;
  if (error) return <div className="error-banner" role="alert"><div><strong>Market catalog needs attention</strong><p>{explainError(error)}</p></div><button type="button" onClick={() => { void catalog.refetch(); if (selectedId) void detail.refetch(); }}>Reload markets</button></div>;
  const instruments = catalog.data?.instruments ?? [];
  const assetFacets = [...new Set(instruments.map(instrument => instrument.assetClass === 'EQUITY' ? 'US equity' : 'Crypto spot'))];
  const venueFacets = [...new Set(instruments.map(instrument => instrument.exchange ?? 'Provider venue'))];
  return <>
    <div className="page-heading"><div><h1>Markets</h1><p>Search canonical instruments and inspect source-backed market availability.</p></div><button type="button" onClick={() => setScreenerOpen(value => !value)}>{screenerOpen ? 'Close screener' : 'Open screener'}</button></div>
    {screenerOpen && <ScreenerBuilder workspaceId={workspaceId} hasCurrentThread={hasCurrentThread} onAttachContexts={onAttachContexts} onBack={() => setScreenerOpen(false)} onOpenInstrument={instrumentId => { setScreenerOpen(false); setSelectedId(instrumentId); }} />}
    {screenerOpen ? null : <>
    <section className="card market-explorer" aria-labelledby="market-explorer-title">
      <div className="market-explorer-heading"><div><h2 id="market-explorer-title">Market Explorer</h2><p className="muted">Census search is coarse and on demand. Select a result to use the Hot detail path.</p></div>{catalog.data && <span className={`badge market-status-badge market-status-${catalog.data.status.toLowerCase()}`}>{statusLabel[catalog.data.status]}</span>}</div>
      <form className="market-search" onSubmit={submit}><label htmlFor="market-search-input">Search instruments</label><div><input id="market-search-input" value={term} onChange={event => setTerm(event.target.value)} placeholder="AAPL, BTC/USDT or company name" maxLength={120} /><button className="primary" type="submit">Search</button></div></form>
      {catalog.isPending ? <p role="status">Loading market catalog…</p> : <><div className="market-facets" aria-label="Market facets"><span className="market-facet-label">Asset class</span>{assetFacets.map(facet => <span className="badge" key={facet}>{facet}</span>)}<span className="market-facet-label">Venue</span>{venueFacets.map(facet => <span className="badge" key={facet}>{facet}</span>)}</div><p className="market-result-count" role="status">{instruments.length} {instruments.length === 1 ? 'instrument' : 'instruments'} found</p><div className="market-explorer-body"><div className="market-results" role="list" aria-label="Market results">{instruments.length ? instruments.map(instrument => <InstrumentRow key={instrument.instrumentId} instrument={instrument} selected={instrument.instrumentId === selectedId} onSelect={() => setSelectedId(instrument.instrumentId)} />) : <p className="muted">No canonical instruments match this search.</p>}</div>{selectedId && detail.isPending && <p role="status">Loading {selectedId}…</p>}{selectedId && detail.data && <Detail detail={detail.data} onBack={() => setSelectedId(undefined)} onOpenDataSources={onOpenDataSources} />}</div></>}
    </section>
    </>}
  </>;
}
