import { FinancialEvidencePanel, MarketFinancialEvidencePanel } from './FinancialEvidencePanel.tsx';
import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { AdjustmentStatus, CorporateAction, FilterSpec, Instrument, MarketDataStatus, MarketDetail, MarketSession, MarketState, RankSpec, ScreenerAttachment, ScreenerDefinition, ScreenerDirection, ScreenerFeature, ScreenerFeatureField, ScreenerLibrary, ScreenerOperator, ScreenerPredicateField, ScreenerResult, ScreenerResultState, ScreenerUniverse, ThreadContextRef } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';
import { ErrorRecoveryPanel } from './ErrorRecoveryPanel.tsx';
import { useOwnedHotQuote } from './useOwnedHotQuote.ts';
import { SpotQuoteEvidencePanel } from './SpotQuoteEvidencePanel.tsx';
import { calculateExactSpread } from './marketSpread.ts';

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

const equityDetailSections = ['Overview', 'Financials', 'News', 'Filings', 'Analysis'] as const;
type EquityDetailSection = typeof equityDetailSections[number];

function InstrumentWatchlist({ workspaceId, instrument }: { workspaceId: string; instrument: Instrument }) {
  const queryClient = useQueryClient();
  const watchlists = useQuery({ queryKey: ['watchlists', workspaceId], queryFn: () => request('watchlist.list', { workspaceId }), retry: false });
  const [selectedId, setSelectedId] = useState('');
  const [newName, setNewName] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState('');
  const lists = watchlists.data?.watchlists ?? [];
  useEffect(() => {
    if (!lists.some(list => list.watchlistId === selectedId)) setSelectedId(lists[0]?.watchlistId ?? '');
  }, [lists, selectedId]);
  const selected = lists.find(list => list.watchlistId === selectedId);
  const alreadyAdded = selected?.items.some(item => item.instrumentId === instrument.instrumentId) ?? false;
  const refresh = async () => {
    await queryClient.invalidateQueries({ queryKey: ['watchlists', workspaceId] });
    await watchlists.refetch();
  };
  const create = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true); setError(undefined); setNotice('');
    try {
      const created = await request('watchlist.create', { workspaceId, name: newName });
      await refresh(); setSelectedId(created.watchlistId); setNewName(''); setNotice(`Created “${created.name}”.`);
    } catch (cause) { setError(explainError(cause)); }
    finally { setBusy(false); }
  };
  const add = async () => {
    if (!selected || alreadyAdded || busy) return;
    setBusy(true); setError(undefined); setNotice('');
    try {
      await request('watchlist.add', { workspaceId, watchlistId: selected.watchlistId, instrumentId: instrument.instrumentId, expectedStateVersion: selected.stateVersion });
      await refresh(); setNotice(`${instrument.symbol} added to “${selected.name}”.`);
    } catch (cause) { setError(explainError(cause)); }
    finally { setBusy(false); }
  };
  return <section className="market-watchlist" aria-labelledby="market-watchlist-title">
    <div className="market-panel-heading"><div><p className="eyebrow">Workspace collection</p><h3 id="market-watchlist-title">Watchlist</h3></div><span className="badge">{lists.length}</span></div>
    {watchlists.isPending ? <p role="status">Loading watchlists…</p> : watchlists.error ? <div role="alert"><p>{explainError(watchlists.error)}</p><button type="button" onClick={() => { void watchlists.refetch(); }}>Reload watchlists</button></div> : <>
      {lists.length > 0 ? <><label htmlFor="market-watchlist-select">Choose a watchlist</label><select id="market-watchlist-select" value={selectedId} onChange={event => setSelectedId(event.target.value)}>{lists.map(list => <option key={list.watchlistId} value={list.watchlistId}>{list.name} · {list.items.length}</option>)}</select><button type="button" onClick={() => { void add(); }} disabled={!selected || alreadyAdded || busy}>{alreadyAdded ? 'Already in watchlist' : busy ? 'Adding…' : 'Add to watchlist'}</button></> : <p className="muted">No watchlists exist in this workspace yet.</p>}
      <form className="market-watchlist-create" onSubmit={event => { void create(event); }}><label htmlFor="market-watchlist-new-name">Create a watchlist</label><div><input id="market-watchlist-new-name" value={newName} maxLength={80} onChange={event => setNewName(event.target.value)} placeholder="e.g. Core equities" /><button type="submit" disabled={busy || !newName.trim()}>Create</button></div></form>
    </>}
    {notice && <p role="status" aria-live="polite">{notice}</p>}{error && <div role="alert"><p>{error}</p><button type="button" onClick={() => { setError(undefined); void watchlists.refetch(); }}>Reload</button></div>}
  </section>;
}

function EquityQuoteAndChart({ detail, onOpenDataSources }: { detail: MarketDetail; onOpenDataSources: () => void }) {
  const quote = detail.snapshot;
  const evidence = quote?.provenance.alpaca;
  return <>
    {quote ? <section className="market-quote" aria-label="Market quote">
      <h3>Bid / ask quote</h3>
      <p className="muted">{quote.provenance.entitlement} · {quote.provenance.freshness}</p>
      <dl className="market-provenance">
        <div><dt>Bid</dt><dd>{quote.bid ?? 'Unavailable'}</dd></div><div><dt>Ask</dt><dd>{quote.ask ?? 'Unavailable'}</dd></div>
        <div><dt>Bid size (BASE)</dt><dd>{quote.bidSize ?? 'Unavailable'}</dd></div><div><dt>Ask size (BASE)</dt><dd>{quote.askSize ?? 'Unavailable'}</dd></div>
        <div><dt>Last trade</dt><dd>{quote.lastPrice ?? 'Not supplied by quote endpoint'}</dd></div><div><dt>Source</dt><dd>{quote.provenance.source}</dd></div>
        <div><dt>Provider time</dt><dd>{quote.provenance.providerTimestamp}</dd></div><div><dt>Received</dt><dd>{quote.provenance.receivedTimestamp}</dd></div>
        <div><dt>Quote coverage</dt><dd>{quote.provenance.venue ?? 'Not supplied'}</dd></div>
        {evidence && <>
          <div><dt>Feed</dt><dd>{evidence.feed === 'delayed_sip' ? 'Delayed SIP' : evidence.feed.toUpperCase()}</dd></div>
          <div><dt>Listing venue</dt><dd>{evidence.listingVenue}</dd></div><div><dt>Provider symbol</dt><dd>{evidence.providerSymbol}</dd></div>
          <div><dt>Bid exchange</dt><dd>{evidence.bidExchange} · {evidence.bidExchangeName}</dd></div><div><dt>Ask exchange</dt><dd>{evidence.askExchange} · {evidence.askExchangeName}</dd></div>
          <div><dt>Tape</dt><dd>{evidence.tape}</dd></div><div><dt>Quote conditions</dt><dd>{evidence.conditions.map(condition => `${condition.code} · ${condition.name}`).join('; ')}</dd></div>
          <div><dt>Depth unit</dt><dd>{evidence.depthUnit === 'BASE' ? 'BASE — shares' : 'Unavailable'}</dd></div>
          <div><dt>Source version</dt><dd>{evidence.sourceVersion}</dd></div><div><dt>Connection generation</dt><dd>{evidence.connectionGeneration}</dd></div>
        </>}
        <div><dt>Snapshot ID</dt><dd>{quote.provenance.marketSnapshotId}</dd></div>
      </dl>
    </section> : <section className="market-unavailable" aria-label="Market quote unavailable"><h4>Quote unavailable</h4><p className="muted">No quote is shown until a selected source is entitled and its adapter returns a validated snapshot.</p><button type="button" onClick={onOpenDataSources}>Open Data &amp; Storage settings</button></section>}
    <section className="market-chart-unavailable" aria-label="Price chart"><h4>Price chart</h4><p className="muted">No validated historical series is available for this instrument. A quote snapshot is not treated as chart history.</p></section>
  </>;
}

function EquityDetailViews({ workspaceId, detail, onOpenDataSources }: { workspaceId: string; detail: MarketDetail; onOpenDataSources: () => void }) {
  const [activeSection, setActiveSection] = useState<EquityDetailSection>('Overview');
  const instrument = detail.instrument;
  const tabId = (section: EquityDetailSection) => `market-detail-tab-${section.toLowerCase()}`;
  const panelId = (section: EquityDetailSection) => `market-detail-panel-${section.toLowerCase()}`;
  return <>
    <div className="market-detail-tabs" role="tablist" aria-label="Equity instrument detail sections">
      {equityDetailSections.map(section => <button key={section} id={tabId(section)} type="button" role="tab" aria-selected={activeSection === section} aria-controls={panelId(section)} tabIndex={activeSection === section ? 0 : -1} onClick={() => setActiveSection(section)} onKeyDown={event => {
        const current = equityDetailSections.indexOf(section);
        const next = event.key === 'ArrowRight' ? (current + 1) % equityDetailSections.length : event.key === 'ArrowLeft' ? (current + equityDetailSections.length - 1) % equityDetailSections.length : event.key === 'Home' ? 0 : event.key === 'End' ? equityDetailSections.length - 1 : -1;
        if (next < 0) return;
        event.preventDefault();
        const target = equityDetailSections[next];
        setActiveSection(target);
        document.getElementById(tabId(target))?.focus();
      }}>{section}</button>)}
    </div>
    <section className="market-detail-panel" id={panelId(activeSection)} role="tabpanel" aria-labelledby={tabId(activeSection)} tabIndex={0}>
      <h3>{activeSection}</h3>
      {activeSection === 'Overview' && <>
        <MarketStatePanel state={detail.marketState} adjustmentStatus={detail.adjustmentStatus} actions={detail.corporateActions} onOpenDataSources={onOpenDataSources} />
        <InstrumentWatchlist workspaceId={workspaceId} instrument={instrument} />
        <EquityQuoteAndChart detail={detail} onOpenDataSources={onOpenDataSources} />
      </>}
      {activeSection === 'Financials' && <>
        {detail.financialEvidence ? <><p className="muted">Available rows are source evidence, not a complete issuer financial statement or valuation.</p><MarketFinancialEvidencePanel evidence={detail.financialEvidence} instrumentId={instrument.instrumentId} /></> : <div className="market-unavailable"><h4>Financial data unavailable</h4><p className="muted">No verified company financial dataset is available for this instrument. No values are synthesized.</p></div>}
      </>}
      {activeSection === 'News' && <div className="market-unavailable"><h4>News unavailable</h4><p className="muted">No verified news source or instrument news payload is available. No stories are synthesized.</p></div>}
      {activeSection === 'Filings' && <div className="market-unavailable"><h4>Filings unavailable</h4><p className="muted">No supported filing payload is available for this instrument in the current data contract.</p></div>}
      {activeSection === 'Analysis' && <div className="market-unavailable"><h4>Analysis unavailable</h4><p className="muted">No source-backed analysis is available without validated market and financial inputs.</p></div>}
    </section>
  </>;
}

function CryptoSpotDetail({ detail, onOpenDataSources }: { detail: MarketDetail; onOpenDataSources: () => void }) {
  const quote = detail.snapshot;
  const spread = calculateExactSpread(quote?.bid, quote?.ask);
  const depthEvidence = quote?.provenance.binance;
  return <>
    <section className="crypto-spot-policy" aria-label="Spot-only warning">
      <div><p className="eyebrow">Product scope</p><h3>Spot only</h3></div>
      <p>Spot only in v1.0. Futures, leverage, margin, and transfer actions are unavailable.</p>
    </section>
    <section className="crypto-venue-state" aria-label="Spot venue state" role="status">
      <div><p className="eyebrow">Spot venue</p><h3>{detail.instrument.exchange ?? detail.marketState.venue ?? 'Venue not selected'}</h3></div>
      <p>Venue state: {sessionLabel[detail.marketState.session]}. A venue state does not establish quote entitlement or trading compatibility.</p>
      <p className="muted">Account-specific symbol admission, quote collection, complete order rules, and execution authority are separate checks.</p>
    </section>
    <section className="market-quote crypto-venue-quote" aria-label="Crypto venue quote">
      <h3>Venue quote</h3>
      {quote && <p className="muted">{quote.provenance.entitlement} · {quote.provenance.freshness}{detail.status === 'AVAILABLE' && quote.provenance.freshness === 'HEALTHY' ? ' · Current validated snapshot' : ' · Retained snapshot — not current'}</p>}
      {!quote && <p className="muted">No validated venue snapshot is available. Values remain unavailable until a selected source is entitled and returns a validated quote.</p>}
      <dl className="market-provenance">
        <div><dt>Venue</dt><dd>{quote?.provenance.venue ?? detail.instrument.exchange ?? 'Unavailable'}</dd></div>
        <div><dt>Bid</dt><dd>{quote?.bid ?? 'Unavailable — no validated quote'}</dd></div>
        <div><dt>Ask</dt><dd>{quote?.ask ?? 'Unavailable — no validated quote'}</dd></div>
        <div><dt>Spread (ask − bid)</dt><dd>{spread.status === 'AVAILABLE' ? spread.value : quote ? spread.reason : 'Unavailable — no validated quote'}</dd></div>
        <div><dt>Bid size (BASE)</dt><dd>{quote?.bidSize ?? 'Unavailable — no validated quote'}</dd></div>
        <div><dt>Ask size (BASE)</dt><dd>{quote?.askSize ?? 'Unavailable — no validated quote'}</dd></div>
        <div><dt>Depth</dt><dd>{depthEvidence ? `${depthEvidence.knownBidLevels} known bid levels · ${depthEvidence.knownAskLevels} known ask levels` : 'Unavailable — no validated venue depth evidence'}</dd></div>
        <div><dt>Last trade</dt><dd>{quote?.lastPrice ?? 'Not supplied'}</dd></div>
        <div><dt>Source</dt><dd>{quote?.provenance.source ?? detail.sourceId ?? 'No source selected'}</dd></div>
        <div><dt>Entitlement</dt><dd>{quote?.provenance.entitlement ?? 'Not verified'}</dd></div>
        <div><dt>Freshness</dt><dd>{quote?.provenance.freshness ?? 'Unavailable — no validated quote'}</dd></div>
        <div><dt>Provider time</dt><dd>{quote?.provenance.providerTimestamp ?? 'Unavailable'}</dd></div>
        <div><dt>TradeX received</dt><dd>{quote?.provenance.receivedTimestamp ?? 'Unavailable'}</dd></div>
        {quote && <div><dt>Snapshot ID</dt><dd>{quote.provenance.marketSnapshotId}</dd></div>}
      </dl>
      {!quote && <button type="button" onClick={onOpenDataSources}>Open Data &amp; Storage settings</button>}
    </section>
    {depthEvidence ? <SpotQuoteEvidencePanel detail={detail} /> : <section className="crypto-depth-unavailable" aria-label="Venue depth unavailable">
      <h3>Venue depth unavailable</h3>
      <p className="muted">No validated depth payload is available for the selected venue. Bid and ask quotes alone do not establish book depth or executable liquidity.</p>
    </section>}
    <section className="market-chart-unavailable crypto-chart-unavailable" aria-label="Crypto price chart">
      <h3>Price chart unavailable</h3>
      <p className="muted">No validated historical series is available for this instrument. A quote snapshot is not treated as chart history.</p>
    </section>
    <section className="crypto-trade-unavailable" aria-label="Trade availability" role="status">
      <h3>Trade action unavailable</h3>
      <p className="muted">A trade action is shown only when this venue, the exact account, and the complete spot order rules are compatible and verified.</p>
    </section>
  </>;
}

function Detail({ workspaceId, detail, onBack, onOpenDataSources }: { workspaceId: string; detail: MarketDetail; onBack: () => void; onOpenDataSources: () => void }) {
  const { instrument } = detail;
  const quote = detail.snapshot;
  return <section className="card market-detail" aria-labelledby="market-detail-title">
    <div className="market-detail-heading"><div><p className="eyebrow">Instrument detail</p><h2 id="market-detail-title">{instrument.displayName}</h2><p className="identity">{instrument.instrumentId}</p></div><button type="button" onClick={onBack}>Back to results</button></div>
    <dl className="market-identity"><div><dt>Symbol</dt><dd>{instrument.symbol}</dd></div><div><dt>Asset class</dt><dd>{instrument.assetClass === 'EQUITY' ? 'US equity' : 'Crypto spot'}</dd></div><div><dt>Venue</dt><dd>{instrument.exchange ?? quote?.provenance.venue ?? detail.marketState.venue ?? 'Unavailable'}</dd></div><div><dt>Currency</dt><dd>{instrument.currency}</dd></div><div><dt>Access tier</dt><dd>{detail.tier}</dd></div></dl>
    <div className={`market-status market-status-${detail.status.toLowerCase()}`} role="status"><strong>{statusLabel[detail.status]}</strong><p>{detail.availabilityReason}</p><small>Source: {detail.sourceId ?? 'No source selected'}</small></div>
    <dl className="market-source-freshness" aria-label="Market data source and freshness"><div><dt>Source</dt><dd>{quote?.provenance.source ?? detail.sourceId ?? 'No source selected'}</dd></div><div><dt>Entitlement</dt><dd>{quote?.provenance.entitlement ?? 'Not verified'}</dd></div><div><dt>Freshness</dt><dd>{quote?.provenance.freshness ?? 'Unavailable — no validated quote'}</dd></div><div><dt>Provider time</dt><dd>{quote?.provenance.providerTimestamp ?? 'Unavailable'}</dd></div><div><dt>TradeX received</dt><dd>{quote?.provenance.receivedTimestamp ?? 'Unavailable'}</dd></div></dl>
    {instrument.assetClass === 'EQUITY' ? <EquityDetailViews workspaceId={workspaceId} detail={detail} onOpenDataSources={onOpenDataSources} /> : <>
      <CryptoSpotDetail detail={detail} onOpenDataSources={onOpenDataSources} />
      {detail.spotRuleEvidence && <section aria-label="Spot rule evidence"><h3>Spot rule evidence</h3><FinancialEvidencePanel source={detail.spotRuleEvidence} instrumentId={instrument.instrumentId} /></section>}
      {detail.financialEvidence && <MarketFinancialEvidencePanel evidence={detail.financialEvidence} instrumentId={instrument.instrumentId} />}
    </>}
  </section>;
}

// A view owns its quote lease. Cleanup also covers acquire completing after navigation.
function InstrumentDetail({ workspaceId, instrumentId, onBack, onOpenDataSources }: { workspaceId: string; instrumentId: string; onBack: () => void; onOpenDataSources: () => void }) {
  const owned = useOwnedHotQuote(workspaceId, instrumentId);
  const { configured, hot, visible } = owned;
  const detail = useQuery({ queryKey: ['market-detail', workspaceId, instrumentId], queryFn: () => request('market.get', { workspaceId, instrumentId, tier: 'HOT' }), enabled: !configured && owned.sourceReady, retry: false, refetchInterval: query => instrumentId.startsWith('equity:') || query.state.data?.spotRuleEvidence ? 1000 : false });
  const error = owned.error ?? detail.error;
  const retry = () => { owned.retry(); if (!configured) void detail.refetch(); };
  if (error) return <div className="error-banner" role="alert"><p>{explainError(error)}</p><button type="button" onClick={retry}>Retry instrument data</button><button type="button" onClick={onOpenDataSources}>Review quote source</button></div>;
  const current = configured ? hot?.detail : detail.data;
  return <div className="market-detail-container">
    {configured && <section className="card hot-quote-status" aria-label="Hot quote subscription">
      <h3>Hot quote subscription</h3><strong>{hot?.status ?? (visible ? 'CONNECTING' : 'PAUSED')}</strong>
      <p>{hot?.reason ?? (visible ? 'Acquiring a quote subscription for this instrument.' : 'This hidden view has released its quote subscription.')}</p>
      <p className="muted">{owned.publicSpot ? `Public Spot stream · Authentication not used · Continuous book ${hot?.subscribed ? 'confirmed' : 'pending'}` : `Feed ${owned.feed?.toUpperCase()} · Authentication ${hot?.authenticated ? 'confirmed' : 'pending'} · Subscription ${hot?.subscribed ? 'confirmed' : 'pending'}`}. Quote freshness and trading eligibility are evaluated separately.</p>
      {hot && <p className="identity">Connection generation {hot.connectionGeneration} · Sequence {hot.sequence}</p>}
      {hot && hot.reconnectAttempt > 0 && <p>Automatic reconnect {hot.reconnectAttempt} of 3.</p>}
      {hot && ['FAILED', 'STALE', 'CLOSED'].includes(hot.status) && <button type="button" onClick={retry}>Retry selected feed</button>}
    </section>}
    {current ? <Detail workspaceId={workspaceId} detail={current} onBack={onBack} onOpenDataSources={onOpenDataSources} /> : <p role="status">Loading {instrumentId}…</p>}
  </div>;
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

function ScreenerBuilder({ workspaceId, onBack, onOpenInstrument, onAttachContexts, hasCurrentThread, initialScreener }: { workspaceId: string; onBack: () => void; onOpenInstrument: (instrumentId: string) => void; onAttachContexts: (contexts: ThreadContextRef[], target: ScreenerAttachTarget) => void; hasCurrentThread: boolean; initialScreener?: ScreenerLibrary['screeners'][number] }) {
  const queryClient = useQueryClient();
  const library = useQuery({ queryKey: ['screeners', workspaceId], queryFn: () => request('screener.list', { workspaceId }), retry: false });
  const [naturalLanguage, setNaturalLanguage] = useState(initialScreener?.definition.naturalLanguage ?? 'US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70.');
  const [spec, setSpec] = useState<FilterSpec | undefined>(initialScreener?.definition.filterSpec);
  const [rankSpec, setRankSpec] = useState<RankSpec | undefined>(initialScreener?.definition.rankSpec);
  const [revision, setRevision] = useState<string | undefined>(initialScreener?.definition.revision);
  const [limit, setLimit] = useState(initialScreener?.definition.limit ?? 10);
  const [result, setResult] = useState<ScreenerResult>();
  const [phase, setPhase] = useState<'idle' | 'parsing' | 'running'>('idle');
  const resultAnnouncement = phase === 'parsing'
    ? 'PARSING · Checking filter conditions'
    : phase === 'running'
      ? 'RUNNING · Screening candidates'
      : result
        ? `${result.state.replaceAll('_', ' ')} · ${result.candidateCount} ${result.candidateCount === 1 ? 'candidate' : 'candidates'}`
        : '';
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [selectedSavedId, setSelectedSavedId] = useState<string | undefined>(initialScreener?.screenerId);
  const [saveName, setSaveName] = useState(initialScreener?.name ?? '');
  const [attachTarget, setAttachTarget] = useState<ScreenerAttachTarget>('new');
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
      const parsed = response.state === 'PARSED';
      setSpec(parsed ? response.filterSpec ?? undefined : undefined);
      setRankSpec(parsed ? response.rankSpec ?? undefined : undefined);
      setRevision(parsed ? response.revision ?? undefined : undefined);
      setStale(!parsed);
      setResult(parsed ? undefined : response);
      setPhase('idle');
      if (!parsed) setError(response.availabilityReason);
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
    <p className="sr-only" role="status" aria-live="polite" aria-atomic="true" aria-label="Screener status">{resultAnnouncement}</p>
    <div className="market-detail-heading"><div><p className="eyebrow">Market screener</p><h2 id="screener-title">Natural-language filter</h2><p className="muted">Parse first, review the typed conditions, then run the read-only candidate query.</p></div><button type="button" onClick={onBack}>Back to explorer</button></div>
    <section className="screener-library" aria-labelledby="screener-library-title"><div className="market-panel-heading"><div><p className="eyebrow">Saved definitions</p><h3 id="screener-library-title">Screener library</h3></div>{library.data && <span className="badge">{library.data.screeners.length} saved</span>}</div>{library.isPending && <p role="status">Loading saved screeners…</p>}{library.isError && <p className="error-text" role="alert">Saved screeners are unavailable.</p>}{library.data?.screeners.length ? <div className="screener-library-list" role="list" aria-label="Saved screeners">{library.data.screeners.map(saved => <article key={saved.screenerId} role="listitem" className={saved.screenerId === selectedSavedId ? 'selected' : ''}><button type="button" onClick={() => reopen(saved)}><strong>{saved.name}</strong><span className="badge">{saved.state.replaceAll('_', ' ')}</span><small>{saved.definition.revision}</small></button></article>)}</div> : library.data && <p className="muted">No saved screeners yet.</p>}</section>
    <ol className="screener-stages" aria-label="Screener stages"><li className={stage === 1 ? 'active' : ''}>1. Describe</li><li className={stage === 2 ? 'active' : ''}>2. Parse</li><li className={stage === 3 ? 'active' : ''}>3. Inspect</li><li className={stage === 4 ? 'active' : ''}>4. Run</li><li className={stage === 5 ? 'active' : ''}>5. Results</li></ol>
    <form className="screener-query" onSubmit={event => { event.preventDefault(); void parse(); }}><label htmlFor="screener-natural-language">Describe the market</label><textarea id="screener-natural-language" value={naturalLanguage} onChange={event => { setNaturalLanguage(event.target.value); setRevision(undefined); setStale(true); setResult(undefined); setSelectedIds([]); }} maxLength={4000} rows={4} /><div className="screener-actions"><button className="primary" type="submit" disabled={phase !== 'idle' || !naturalLanguage.trim()}>Parse conditions</button>{phase === 'parsing' && <span aria-hidden="true">Parsing…</span>}</div></form>
    {spec && rankSpec && <section className="screener-inspection" aria-labelledby="screener-inspection-title"><div className="market-panel-heading"><div><p className="eyebrow">Review before run</p><h3 id="screener-inspection-title">FilterSpec and RankSpec</h3></div>{revision && <span className="badge">Revision ready</span>}</div><label htmlFor="screener-universe">Universe</label><select id="screener-universe" value={spec.universe} onChange={event => editSpec({ ...spec, universe: event.target.value as ScreenerUniverse })}>{Object.entries(screenerUniverseLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><div className="screener-predicates"><span className="market-facet-label">Conditions</span>{(spec.predicates ?? []).map((predicate, index) => <div className="screener-predicate" key={`${predicate.field}-${index}`}><select aria-label={`Condition ${index + 1} field`} value={predicate.field} onChange={event => { const predicates = [...(spec.predicates ?? [])]; predicates[index] = { ...predicate, field: event.target.value as ScreenerPredicateField }; editSpec({ ...spec, predicates: predicates as FilterSpec['predicates'] }); }}>{Object.entries(screenerFieldLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><select aria-label={`Condition ${index + 1} operator`} value={predicate.operator} onChange={event => { const predicates = [...(spec.predicates ?? [])]; predicates[index] = { ...predicate, operator: event.target.value as ScreenerOperator }; editSpec({ ...spec, predicates: predicates as FilterSpec['predicates'] }); }}>{Object.entries(screenerOperatorLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><input aria-label={`Condition ${index + 1} threshold`} value={predicate.threshold} onChange={event => { const predicates = [...(spec.predicates ?? [])]; predicates[index] = { ...predicate, threshold: event.target.value }; editSpec({ ...spec, predicates: predicates as FilterSpec['predicates'] }); }} maxLength={64} /></div>)}</div><div className="screener-rank"><label htmlFor="screener-rank-field">Rank by</label><select id="screener-rank-field" value={rankSpec.field} onChange={event => editRank({ ...rankSpec, field: event.target.value as RankSpec['field'] })}>{Object.entries(screenerRankLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><select aria-label="Rank direction" value={rankSpec.direction} onChange={event => editRank({ ...rankSpec, direction: event.target.value as ScreenerDirection })}><option value="DESC">Highest first</option><option value="ASC">Lowest first</option></select><label htmlFor="screener-limit">Limit</label><input id="screener-limit" type="number" min={1} max={50} value={limit} onChange={event => { setLimit(Math.max(1, Math.min(50, Number(event.target.value) || 1))); setRevision(undefined); setStale(true); setResult(undefined); setSelectedIds([]); }} /></div><div className="screener-actions"><button type="button" onClick={() => void parse(true)} disabled={phase !== 'idle'}>Recalculate revision</button><button className="primary" type="button" onClick={() => void run()} disabled={phase !== 'idle' || stale || !revision}>Run screen</button>{stale && <span role="status">Conditions changed; recalculate the revision before running.</span>}{phase === 'running' && <span aria-hidden="true">Running…</span>}</div></section>}
    {spec && rankSpec && revision && !stale && library.data && <div className="screener-save"><label htmlFor="screener-name">Save name<input id="screener-name" value={saveName} onChange={event => setSaveName(event.target.value)} maxLength={80} placeholder="e.g. Growth leaders" /></label><button type="button" className="primary" onClick={() => void save()} disabled={saveBusy || !saveName.trim() || library.isPending}>{saveBusy ? 'Saving…' : selectedSavedId ? 'Save changes' : 'Save screener'}</button></div>}
    {notice && <p className="form-hint" role="status">{notice}</p>}
    {error && <p className="error-banner" role="alert">{error}</p>}
    {result && <section className={`screener-results screener-results-${result.state.toLowerCase()}`} aria-labelledby="screener-results-title"><div className="market-panel-heading"><div><p className="eyebrow">Screen result</p><h3 id="screener-results-title">{result.state.replaceAll('_', ' ')}</h3></div><span className="badge">{result.candidateCount} candidates</span></div><p>{result.availabilityReason}</p><p className="muted">Conditions: {(result.appliedConditions ?? []).join(' · ')}</p>{result.limitations?.map(limitation => <p className="muted" key={limitation}>{limitation}</p>)}{result.fixtureLabel && <p className="muted">Fixture: {result.fixtureLabel}</p>}{result.candidates?.length ? <div className="screener-candidates" role="list" aria-label="Screener candidates">{result.candidates.map(candidate => <article className="screener-candidate" role="listitem" key={candidate.instrumentId}><label className="screener-candidate-select"><input type="checkbox" aria-label={`Select ${candidate.instrumentId}`} checked={selectedIds.includes(candidate.instrumentId)} onChange={() => setSelectedIds(current => current.includes(candidate.instrumentId) ? current.filter(id => id !== candidate.instrumentId) : [...current, candidate.instrumentId])} /><span>Select</span></label><button type="button" className="screener-candidate-button" aria-label={`Open ${candidate.instrumentId} market detail`} onClick={() => onOpenInstrument(candidate.instrumentId)}><div><strong>{candidate.rank}. {candidate.symbol}</strong><small className="identity">{candidate.instrumentId}</small></div><div className="screener-feature-list"><span>Quality {featureValue(candidate, 'QUALITY')}</span><span>Revision {featureValue(candidate, 'REVISION_STRENGTH')}</span><span>Momentum {featureValue(candidate, 'MOMENTUM')}</span></div><small>Source {candidate.provenance.sourceId} · Provider {candidate.provenance.providerTimestamp ?? 'Unavailable'} · Received {candidate.provenance.receivedTimestamp} · {candidate.provenance.freshness} · {candidate.provenance.quality} · {candidate.limitation}</small></button></article>)}</div> : <p className="muted">No candidates matched the reviewed conditions.</p>}{result.candidates?.length ? <div className="screener-attach"><label htmlFor="screener-attach-target">Attach selected to<select id="screener-attach-target" value={attachTarget} onChange={event => setAttachTarget(event.target.value as ScreenerAttachTarget)}><option value="new">New Thread</option><option value="current" disabled={!hasCurrentThread}>Current Thread next Turn{hasCurrentThread ? '' : ' (select a Thread first)'}</option></select></label><button type="button" className="primary" onClick={() => void attach()} disabled={attachBusy || selectedIds.length === 0 || !result.revision}>{attachBusy ? 'Attaching…' : `Attach selected (${selectedIds.length})`}</button></div> : null}<button type="button" onClick={() => void (result.state === 'FAILED' ? parse() : run())} disabled={phase !== 'idle'}>Retry screen</button></section>}
  </section>;
}

export function Markets({ workspaceId, onOpenDataSources, onAttachContexts, hasCurrentThread }: { workspaceId: string; onOpenDataSources: () => void; onAttachContexts: (contexts: ThreadContextRef[], target: ScreenerAttachTarget) => void; hasCurrentThread: boolean }) {
  const [term, setTerm] = useState('');
  const [query, setQuery] = useState('');
  const [selectedId, setSelectedId] = useState<string>();
  const [screenerOpen, setScreenerOpen] = useState(false);
  const [marketView, setMarketView] = useState<'stocks' | 'crypto'>('stocks');
  const [returnView, setReturnView] = useState<'stocks' | 'crypto'>('stocks');
  const [initialScreener, setInitialScreener] = useState<ScreenerLibrary['screeners'][number]>();
  const catalog = useQuery({ queryKey: ['market-catalog', workspaceId, query], queryFn: () => request('market.catalog', { workspaceId, query, tier: 'CENSUS' }) });
  const screeners = useQuery({ queryKey: ['screeners', workspaceId], queryFn: () => request('screener.list', { workspaceId }), retry: false });
  useEffect(() => {
    if (selectedId && catalog.data?.instruments.every(instrument => instrument.instrumentId !== selectedId)) setSelectedId(undefined);
  }, [catalog.data?.instruments, selectedId]);
  const submit = (event: FormEvent) => { event.preventDefault(); setQuery(term.trim()); };
  const instruments = catalog.data?.instruments ?? [];
  const equityCount = instruments.filter(instrument => instrument.assetClass === 'EQUITY').length;
  const cryptoCount = instruments.filter(instrument => instrument.assetClass === 'CRYPTO_SPOT').length;
  const visibleInstruments = instruments.filter(instrument => marketView === 'stocks' ? instrument.assetClass === 'EQUITY' : instrument.assetClass === 'CRYPTO_SPOT');
  const assetFacets = [...new Set(visibleInstruments.map(instrument => instrument.assetClass === 'EQUITY' ? 'US equity' : 'Crypto spot'))];
  const venueFacets = [...new Set(visibleInstruments.map(instrument => instrument.exchange ?? 'Provider venue'))];
  const catalogStatus = catalog.data ? statusLabel[catalog.data.status] : catalog.isError ? 'Unavailable' : 'Loading';
  const catalogReason = catalog.data?.availabilityReason ?? (catalog.error ? explainError(catalog.error) : 'Waiting for the local market catalog.');
  const openScreeners = (saved?: ScreenerLibrary['screeners'][number]) => {
    setReturnView(marketView);
    setInitialScreener(saved);
    setScreenerOpen(true);
  };
  const selectMarketView = (view: 'stocks' | 'crypto') => {
    setMarketView(view);
    setSelectedId(undefined);
    setScreenerOpen(false);
  };
  return <>
    <div className="page-heading"><div><h1>Markets</h1><p>Search canonical instruments and inspect source-backed market availability.</p></div><button type="button" onClick={() => screenerOpen ? setScreenerOpen(false) : openScreeners()}>{screenerOpen ? 'Close screener' : 'Open screener'}</button></div>
    <div className="market-view-tabs" role="group" aria-label="Market sections">
      <button type="button" aria-pressed={!screenerOpen && marketView === 'stocks'} onClick={() => selectMarketView('stocks')}>US Stocks</button>
      <button type="button" aria-pressed={!screenerOpen && marketView === 'crypto'} onClick={() => selectMarketView('crypto')}>Crypto</button>
      <button type="button" aria-pressed={screenerOpen} onClick={() => openScreeners()}>Screeners</button>
    </div>
    {screenerOpen && <ScreenerBuilder workspaceId={workspaceId} hasCurrentThread={hasCurrentThread} onAttachContexts={onAttachContexts} initialScreener={initialScreener} onBack={() => { setMarketView(returnView); setScreenerOpen(false); }} onOpenInstrument={instrumentId => { setMarketView(instrumentId.startsWith('crypto:') ? 'crypto' : 'stocks'); setScreenerOpen(false); setSelectedId(instrumentId); }} />}
    {screenerOpen ? null : <>
    <div className="market-overview">
      <section className="card market-overview-card" aria-labelledby="market-metrics-title">
        <div className="market-panel-heading"><div><p className="eyebrow">Source-backed overview</p><h2 id="market-metrics-title">Market metrics</h2></div><span className="badge">{catalogStatus}</span></div>
        <dl className="market-metric-grid"><div><dt>US equity matches</dt><dd>{catalog.isPending ? 'Loading' : equityCount}</dd></div><div><dt>Crypto spot matches</dt><dd>{catalog.isPending ? 'Loading' : cryptoCount}</dd></div><div><dt>Selected source</dt><dd>{catalog.data?.sourceId ?? 'None selected'}</dd></div></dl>
        <p className="muted">Catalog counts describe available instrument identities. Price, index and performance metrics require a verified market-data source.</p>
        {catalog.data && catalog.data.status !== 'AVAILABLE' && <p className="market-availability-reason">{catalogReason}</p>}
        {catalog.isError && <div className="error-banner" role="alert"><p>{catalogReason}</p><button type="button" onClick={() => { void catalog.refetch(); }}>Reload markets</button></div>}
      </section>
      <section className="card market-overview-card" aria-labelledby="market-movers-title">
        <div className="market-panel-heading"><div><p className="eyebrow">Current session</p><h2 id="market-movers-title">Top movers</h2></div><span className="badge">{catalogStatus}</span></div>
        <p>{catalog.data?.status === 'AVAILABLE' ? 'The current market catalog does not include a market-wide movers snapshot.' : catalogReason}</p>
        <p className="muted">No prices or percentage changes are synthesized when mover data is unavailable.</p>
        {(!catalog.data || catalog.data.status !== 'AVAILABLE') && <button type="button" onClick={onOpenDataSources}>Review market-data sources</button>}
      </section>
      <section className="card market-overview-card market-saved-screeners" aria-labelledby="market-saved-screeners-title">
        <div className="market-panel-heading"><div><p className="eyebrow">Workspace library</p><h2 id="market-saved-screeners-title">Saved screeners</h2></div><span className="badge">{screeners.data?.screeners.length ?? (screeners.isPending ? 'Loading' : 'Unavailable')}</span></div>
        {screeners.isError && <p className="error-text" role="alert">Saved screeners are unavailable.</p>}
        {screeners.isPending && <p role="status">Loading saved screeners…</p>}
        {screeners.data?.screeners.length ? <><ul className="market-saved-list">{screeners.data.screeners.slice(0, 4).map(saved => <li key={saved.screenerId}><button type="button" aria-label={`Open saved screener ${saved.name}`} onClick={() => openScreeners(saved)}><span><strong>{saved.name}</strong><small>{saved.definition.focus?.replaceAll('_', ' ') ?? 'Market'} · {saved.state.replaceAll('_', ' ')}</small></span><span aria-hidden="true">Open</span></button></li>)}</ul>{screeners.data.screeners.length > 4 && <p className="muted">Showing 4 of {screeners.data.screeners.length} saved screeners. Open Screeners to view the full library.</p>}</> : screeners.data && <p className="muted">No saved screeners yet.</p>}
        {screeners.data && screeners.data.screeners.length === 0 && <button type="button" onClick={() => openScreeners()}>Create a screener</button>}
      </section>
    </div>
    <section className="card market-explorer" aria-labelledby="market-explorer-title">
      <div className="market-explorer-heading"><div><h2 id="market-explorer-title">Market Explorer</h2><p className="muted">{marketView === 'stocks' ? 'US Stocks' : 'Crypto Spot'} · Census search is coarse and on demand. Select a result to use the Hot detail path.</p></div>{catalog.data && <span className={`badge market-status-badge market-status-${catalog.data.status.toLowerCase()}`}>{statusLabel[catalog.data.status]}</span>}</div>
      <form className="market-search" onSubmit={submit}><label htmlFor="market-search-input">Search instruments</label><div><input id="market-search-input" value={term} onChange={event => setTerm(event.target.value)} placeholder="AAPL, BTC/USDT or company name" maxLength={120} /><button className="primary" type="submit">Search</button></div></form>
      {catalog.isPending ? <p role="status">Loading market catalog…</p> : <><div className="market-facets" aria-label="Market facets"><span className="market-facet-label">Asset class</span>{assetFacets.map(facet => <span className="badge" key={facet}>{facet}</span>)}<span className="market-facet-label">Venue</span>{venueFacets.map(facet => <span className="badge" key={facet}>{facet}</span>)}</div><p className="market-result-count" role="status">{visibleInstruments.length} {visibleInstruments.length === 1 ? 'instrument' : 'instruments'} found</p><div className="market-explorer-body"><div className="market-results" role="list" aria-label="Market results">{visibleInstruments.length ? visibleInstruments.map(instrument => <InstrumentRow key={instrument.instrumentId} instrument={instrument} selected={instrument.instrumentId === selectedId} onSelect={() => setSelectedId(instrument.instrumentId)} />) : <p className="muted">No canonical instruments match this search.</p>}</div>{selectedId && visibleInstruments.some(instrument => instrument.instrumentId === selectedId) && <InstrumentDetail key={`${workspaceId}:${selectedId}`} workspaceId={workspaceId} instrumentId={selectedId} onBack={() => setSelectedId(undefined)} onOpenDataSources={onOpenDataSources} />}</div></>}
    </section>
    </>}
  </>;
}
