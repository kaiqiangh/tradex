import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import type { DataSourceEntry, DataSourceStatus } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';

const statusLabels: Record<DataSourceStatus, string> = {
  AVAILABLE: 'Available',
  UNAVAILABLE: 'Unavailable',
  BLOCKED_EXTERNAL: 'Blocked by external setup',
  UNVERIFIED: 'Not verified',
};
const staleReason = 'Last successful observation retained; this result is stale.';

function sourceIsStale(source: DataSourceEntry) {
  return source.status === 'UNAVAILABLE' && source.availabilityReason.includes(staleReason);
}

function statusClass(status: DataSourceStatus, stale: boolean) {
  return `badge data-source-status data-source-status-${status.toLowerCase()}${stale ? ' data-source-status-stale' : ''}`;
}

function SourceCard({ source, stale, busy, onProbe, error }: { source: DataSourceEntry; stale: boolean; busy: boolean; onProbe: () => void; error?: string }) {
  return <article className="data-source-card" aria-labelledby={`data-source-${source.sourceId}`}>
    <div className="data-source-heading">
      <div><h3 id={`data-source-${source.sourceId}`}>{source.sourceId}</h3><p>{source.provider}</p></div>
      <span className={statusClass(source.status, stale)} role="status">{statusLabels[source.status]}{stale ? ' · stale' : ''}</span>
    </div>
    <div className="data-source-capabilities" aria-label={`${source.sourceId} capabilities`}>{source.capabilities.map(capability => <span className="badge" key={capability}>{capability}</span>)}</div>
    <dl className="data-source-details">
      <div><dt>Coverage</dt><dd>{source.coverage}</dd></div>
      <div><dt>Latency</dt><dd>{source.latency}</dd></div>
      <div><dt>Entitlement</dt><dd>{source.entitlement}</dd></div>
      <div><dt>Retention</dt><dd>{source.retention}</dd></div>
      <div><dt>Redistribution / commercial use</dt><dd>{source.redistribution} {source.commercialUse}</dd></div>
      <div><dt>Jurisdictions</dt><dd>{source.jurisdictions}</dd></div>
    </dl>
    <div className="data-source-footer">
      <span className="muted">Reviewed {source.reviewedAt} · checked {source.checkedAt ? new Date(source.checkedAt).toLocaleString() : 'Not checked'}{source.observedAt ? ` · observed ${new Date(source.observedAt).toLocaleString()}` : ''}</span>
      <div className="data-source-links"><a href={source.officialUrl} target="_blank" rel="noreferrer noopener">Official docs</a><a href={source.termsUrl} target="_blank" rel="noreferrer noopener">Terms / policy</a></div>
    </div>
    <p className={source.status === 'AVAILABLE' ? 'success-text' : 'data-source-reason'}>{source.availabilityReason}{stale && !source.availabilityReason.includes(staleReason) ? ` ${staleReason}` : ''}</p>
    {error && <p className="error-text" role="alert">{error}</p>}
    <button type="button" onClick={onProbe} disabled={busy}>{busy ? 'Checking…' : source.probeKind === 'PUBLIC_METADATA' ? 'Check public endpoint' : 'Check entitlement'}</button>
  </article>;
}

export function DataSources({ workspaceId }: { workspaceId: string }) {
  const catalog = useQuery({ queryKey: ['data-source-catalog', workspaceId], queryFn: () => request('data.source.catalog', { workspaceId }), retry: false });
  const [probing, setProbing] = useState<string>();
  const [probeErrors, setProbeErrors] = useState<Record<string, string>>({});
  const [overrides, setOverrides] = useState<Record<string, DataSourceEntry>>({});
  const [staleSources, setStaleSources] = useState<Record<string, boolean>>({});
  const stateVersion = catalog.data?.stateVersion;
  useEffect(() => { setOverrides({}); setStaleSources({}); setProbeErrors({}); }, [stateVersion]);
  const probe = async (source: DataSourceEntry) => {
    if (!catalog.data || probing) return;
    const previous = overrides[source.sourceId] ?? source;
    setProbing(source.sourceId);
    setProbeErrors(current => ({ ...current, [source.sourceId]: '' }));
    try {
      const result = await request('data.source.probe', { workspaceId, sourceId: source.sourceId, expectedStateVersion: catalog.data.stateVersion });
      const updated = result.sources.find(item => item.sourceId === source.sourceId);
      if (updated) {
        const stale = updated.status === 'UNAVAILABLE' && previous.status === 'AVAILABLE' && Boolean(previous.observedAt);
        const displayed = stale ? { ...updated, observedAt: previous.observedAt } : updated;
        setStaleSources(current => ({ ...current, [source.sourceId]: stale }));
        setOverrides(current => ({ ...current, [source.sourceId]: displayed }));
      }
    } catch (error) {
      if (previous.status === 'AVAILABLE' && previous.observedAt) setStaleSources(current => ({ ...current, [source.sourceId]: true }));
      setProbeErrors(current => ({ ...current, [source.sourceId]: explainError(error) }));
    } finally { setProbing(undefined); }
  };
  if (catalog.isPending) return <p role="status">Loading data-source policy…</p>;
  if (catalog.isError) return <div className="error-banner" role="alert"><div><strong>Data-source policy is unavailable.</strong><p>{explainError(catalog.error)}</p></div><button type="button" onClick={() => void catalog.refetch()}>Reload data sources</button></div>;
  return <section className="data-sources" aria-labelledby="data-sources-title">
    <h3 id="data-sources-title">Data sources</h3>
    <p className="muted">These are read-only source gates. Provider account links do not grant market-data entitlement, and no secrets are entered here.</p>
    <p className="form-hint">A public endpoint check proves reachability only. Coverage, freshness, terms and commercial use remain visible with the result.</p>
    <div className="data-source-grid">{catalog.data.sources.map(source => { const current = overrides[source.sourceId] ?? source; const stale = staleSources[source.sourceId] === true || sourceIsStale(current); return <SourceCard key={source.sourceId} source={current} stale={stale} busy={probing !== undefined} onProbe={() => void probe(current)} error={probeErrors[source.sourceId] || undefined} />; })}</div>
  </section>;
}
