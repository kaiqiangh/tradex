import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery } from '@tanstack/react-query';
import type { Instrument, MarketDataStatus, MarketDetail } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';

const statusLabel: Record<MarketDataStatus, string> = {
  AVAILABLE: 'Available',
  UNAVAILABLE: 'Unavailable',
  BLOCKED_EXTERNAL: 'Blocked by external setup',
  UNVERIFIED: 'Unverified',
};

function InstrumentRow({ instrument, selected, onSelect }: { instrument: Instrument; selected: boolean; onSelect: () => void }) {
  return <button type="button" className="market-row" aria-current={selected ? 'true' : undefined} onClick={onSelect}>
    <span className="market-row-main"><strong>{instrument.symbol}</strong><small>{instrument.displayName}</small></span>
    <span className="market-row-meta"><span className="badge">{instrument.assetClass === 'EQUITY' ? 'US equity' : 'Crypto spot'}</span><small className="identity">{instrument.instrumentId}</small></span>
  </button>;
}

function Detail({ detail, onBack, onOpenDataSources }: { detail: MarketDetail; onBack: () => void; onOpenDataSources: () => void }) {
  const { instrument } = detail;
  return <section className="card market-detail" aria-labelledby="market-detail-title">
    <div className="market-detail-heading"><div><p className="eyebrow">Instrument detail</p><h2 id="market-detail-title">{instrument.displayName}</h2><p className="identity">{instrument.instrumentId}</p></div><button type="button" onClick={onBack}>Back to results</button></div>
    <dl className="market-identity"><div><dt>Symbol</dt><dd>{instrument.symbol}</dd></div><div><dt>Asset class</dt><dd>{instrument.assetClass === 'EQUITY' ? 'US equity' : 'Crypto spot'}</dd></div><div><dt>Venue</dt><dd>{instrument.exchange ?? 'Provider venue selected at fetch'}</dd></div><div><dt>Currency</dt><dd>{instrument.currency}</dd></div><div><dt>Access tier</dt><dd>{detail.tier}</dd></div></dl>
    <div className={`market-status market-status-${detail.status.toLowerCase()}`} role="status"><strong>{statusLabel[detail.status]}</strong><p>{detail.availabilityReason}</p><small>Source: {detail.sourceId ?? 'No source selected'}</small></div>
    {detail.snapshot ? <section className="market-quote" aria-label="Market quote"><h3>Quote</h3><p className="market-price">{detail.snapshot.lastPrice ?? 'No last price'}</p><p className="muted">{detail.snapshot.provenance.entitlement} · {detail.snapshot.provenance.freshness}</p><dl className="market-provenance"><div><dt>Provider time</dt><dd>{detail.snapshot.provenance.providerTimestamp}</dd></div><div><dt>Received</dt><dd>{detail.snapshot.provenance.receivedTimestamp}</dd></div><div><dt>Venue</dt><dd>{detail.snapshot.provenance.venue ?? 'Not supplied'}</dd></div></dl></section> : <div className="market-unavailable"><p className="muted">No quote or chart is shown until the selected source is entitled and the adapter returns a validated snapshot.</p><button type="button" onClick={onOpenDataSources}>Open Data &amp; Storage settings</button></div>}
  </section>;
}

export function Markets({ workspaceId, onOpenDataSources }: { workspaceId: string; onOpenDataSources: () => void }) {
  const [term, setTerm] = useState('');
  const [query, setQuery] = useState('');
  const [selectedId, setSelectedId] = useState<string>();
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
    <div className="page-heading"><h1>Markets</h1><p>Search canonical instruments and inspect source-backed market availability.</p></div>
    <section className="card market-explorer" aria-labelledby="market-explorer-title">
      <div className="market-explorer-heading"><div><h2 id="market-explorer-title">Market Explorer</h2><p className="muted">Census search is coarse and on demand. Select a result to use the Hot detail path.</p></div>{catalog.data && <span className={`badge market-status-badge market-status-${catalog.data.status.toLowerCase()}`}>{statusLabel[catalog.data.status]}</span>}</div>
      <form className="market-search" onSubmit={submit}><label htmlFor="market-search-input">Search instruments</label><div><input id="market-search-input" value={term} onChange={event => setTerm(event.target.value)} placeholder="AAPL, BTC/USDT or company name" maxLength={120} /><button className="primary" type="submit">Search</button></div></form>
      {catalog.isPending ? <p role="status">Loading market catalog…</p> : <><div className="market-facets" aria-label="Market facets"><span className="market-facet-label">Asset class</span>{assetFacets.map(facet => <span className="badge" key={facet}>{facet}</span>)}<span className="market-facet-label">Venue</span>{venueFacets.map(facet => <span className="badge" key={facet}>{facet}</span>)}</div><p className="market-result-count" role="status">{instruments.length} {instruments.length === 1 ? 'instrument' : 'instruments'} found</p><div className="market-explorer-body"><div className="market-results" role="list" aria-label="Market results">{instruments.length ? instruments.map(instrument => <InstrumentRow key={instrument.instrumentId} instrument={instrument} selected={instrument.instrumentId === selectedId} onSelect={() => setSelectedId(instrument.instrumentId)} />) : <p className="muted">No canonical instruments match this search.</p>}</div>{selectedId && detail.isPending && <p role="status">Loading {selectedId}…</p>}{selectedId && detail.data && <Detail detail={detail.data} onBack={() => setSelectedId(undefined)} onOpenDataSources={onOpenDataSources} />}</div></>}
    </section>
  </>;
}
