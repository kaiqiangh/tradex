import { FinancialSourceSettings } from './FinancialSourceSettings.tsx';
import { BinanceRuleSourceSettings } from './BinanceRuleSourceSettings.tsx';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import type { AlpacaFeed, DataSourceCredentialKind, DataSourceEntry, DataSourceStatus } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';
import { CalendarSourceSettings } from './CalendarSourceSettings.tsx';

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

function QuoteSourceSettings({ workspaceId, onSourceChange }: { workspaceId: string; onSourceChange: () => void }) {
  const queryClient = useQueryClient();
  const connection = useQuery({ queryKey: ['quote-source-connection', workspaceId], queryFn: () => request('data.source.connection', { workspaceId }), retry: false });
  const [accountId, setAccountId] = useState('');
  const [credentialKind, setCredentialKind] = useState<DataSourceCredentialKind>('EXISTING_ACCOUNT');
  const [feed, setFeed] = useState<AlpacaFeed>('iex');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState<string>();
  useEffect(() => {
    setAccountId(connection.data?.accountId ?? '');
    setCredentialKind(connection.data?.credentialKind ?? 'EXISTING_ACCOUNT');
    setFeed(connection.data?.feed ?? 'iex');
  }, [connection.data?.stateVersion, workspaceId]);
  const reloadSelection = async () => {
    const result = await connection.refetch();
    if (result.data) {
      setAccountId(result.data.accountId ?? '');
      setCredentialKind(result.data.credentialKind ?? 'EXISTING_ACCOUNT');
      setFeed(result.data.feed ?? 'iex');
    }
    onSourceChange();
    await queryClient.invalidateQueries({ queryKey: ['data-source-catalog', workspaceId] });
  };
  const save = async () => {
    if (!connection.data || (credentialKind === 'EXISTING_ACCOUNT' && !accountId) || busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      await request('data.source.configure', { workspaceId, expectedStateVersion: connection.data.stateVersion, feed,
        credential: credentialKind === 'DEDICATED' ? { kind: 'DEDICATED' } : { kind: 'EXISTING_ACCOUNT', connectionId: accountId } });
      await reloadSelection();
      setNotice('Source selection saved. Data access has not been verified.');
    } catch (cause) { setError(explainError(cause)); await reloadSelection(); } finally { setBusy(false); }
  };
  const disconnect = async () => {
    if (!connection.data || busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const result = await request('data.source.disconnect', { workspaceId, expectedStateVersion: connection.data.stateVersion });
      await reloadSelection();
      setNotice(connection.data.credentialKind === 'DEDICATED'
        ? result.cleanupPending ? 'Quote source disconnected. Data-key cleanup needs another attempt.' : 'Quote source disconnected. Its dedicated data key was removed.'
        : 'Quote source disconnected. Your Alpaca account and its saved key were kept.');
    } catch (cause) { setError(explainError(cause)); } finally { setBusy(false); }
  };
  const cleanup = async () => {
    if (busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const result = await request('data.source.cleanup', { workspaceId });
      await reloadSelection();
      setNotice(result.cleanupPending ? 'Data-key cleanup needs another attempt.' : 'Pending data-key cleanup completed.');
    } catch (cause) { setError(explainError(cause)); } finally { setBusy(false); }
  };
  const verify = async () => {
    if (!connection.data?.configured || busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const catalog = await request('data.source.catalog', { workspaceId });
      const result = await request('data.source.probe', { workspaceId, sourceId: 'OD-001', expectedStateVersion: catalog.stateVersion });
      await reloadSelection();
      const source = result.sources.find(source => source.sourceId === 'OD-001');
      if (source?.status === 'AVAILABLE') setNotice('Selected feed technical access verified. Quote freshness and trading eligibility remain separate.');
      else setError(source?.availabilityReason ?? 'Selected feed access could not be verified.');
    } catch (cause) { setError(explainError(cause)); await reloadSelection(); } finally { setBusy(false); }
  };
  if (connection.isPending) return <p role="status">Loading quote source…</p>;
  if (connection.isError) return <div role="alert"><p>{explainError(connection.error)}</p><button type="button" onClick={() => void reloadSelection()}>Reload quote source</button></div>;
  return <section className="card quote-source-settings" aria-labelledby="quote-source-heading">
    <h3 id="quote-source-heading">Alpaca quote source</h3>
    <p>Use an existing saved Alpaca key or enter a dedicated data key in the native secure window. Choose the feed explicitly; saving a key does not verify data access.</p>
    <label>Data credentials<select aria-label="Data credentials" value={credentialKind} onChange={event => setCredentialKind(event.target.value as DataSourceCredentialKind)} disabled={busy}>
      <option value="EXISTING_ACCOUNT">Use a saved Alpaca account key</option><option value="DEDICATED">Dedicated market-data key</option>
    </select></label>
    {credentialKind === 'EXISTING_ACCOUNT' && <label>Saved Alpaca account<select aria-label="Saved Alpaca account" value={accountId} onChange={event => setAccountId(event.target.value)} disabled={busy}>
      <option value="">Choose an account</option>
      {connection.data.eligibleAccounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.displayName}</option>)}
      {connection.data.accountId && !connection.data.eligibleAccounts.some(account => account.connectionId === connection.data.accountId) && <option value={connection.data.accountId}>Saved account — reconnect to change selection</option>}
    </select></label>}
    {credentialKind === 'DEDICATED' && <p className="form-hint">Saving opens native secure entry for a new key. Disconnect removes only this source's dedicated key; brokerage account keys are kept.</p>}
    <label>Market-data feed<select aria-label="Market-data feed" value={feed} onChange={event => setFeed(event.target.value as AlpacaFeed)} disabled={busy}>
      <option value="iex">IEX — single-venue realtime</option><option value="sip">SIP — consolidated realtime, entitlement required</option><option value="delayed_sip">Delayed SIP — consolidated delayed</option>
    </select></label>
    <p className="form-hint">A denied feed stays denied. Coverage, freshness and licensing remain separate from connectivity.</p>
    <p role="status">{connection.data.availabilityReason}</p>
    {notice && <p className="success-text" role="status">{notice}</p>}
    {error && <div role="alert"><p className="error-text">{error}</p><button type="button" onClick={() => void reloadSelection()} disabled={busy}>Reload quote source</button></div>}
    {connection.data.cleanupPending && <div role="status"><p>Access to discarded data keys has stopped. Their Keychain cleanup is still pending.</p><button type="button" onClick={() => void cleanup()} disabled={busy}>Retry data-key cleanup</button></div>}
    <button type="button" onClick={() => void save()} disabled={busy || (credentialKind === 'EXISTING_ACCOUNT' && !connection.data.eligibleAccounts.some(account => account.connectionId === accountId))}>{busy ? 'Working…' : credentialKind === 'DEDICATED' ? 'Save data key securely' : 'Save quote source'}</button>
    <button type="button" onClick={() => void verify()} disabled={busy || !connection.data.configured || feed !== connection.data.feed || credentialKind !== connection.data.credentialKind || (credentialKind === 'EXISTING_ACCOUNT' && accountId !== connection.data.accountId)}>Verify selected feed</button>
    {connection.data.configured && <button type="button" onClick={() => void disconnect()} disabled={busy}>Disconnect quote source</button>}
  </section>;
}

export function DataSources({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
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
      if (source.sourceId === 'OD-001') await queryClient.invalidateQueries({ queryKey: ['quote-source-connection', workspaceId] });
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
    <QuoteSourceSettings key={workspaceId} workspaceId={workspaceId} onSourceChange={() => {
      setOverrides(current => { const next={...current}; delete next['OD-001']; return next; });
      setStaleSources(current => ({ ...current, 'OD-001': false }));
      setProbeErrors(current => ({ ...current, 'OD-001': '' }));
    }} />
    <CalendarSourceSettings key={`calendar-${workspaceId}`} workspaceId={workspaceId} onSourceChange={() => {
      setOverrides(current => { const next = { ...current }; delete next['OD-005']; return next; });
      setStaleSources(current => ({ ...current, 'OD-005': false }));
      setProbeErrors(current => ({ ...current, 'OD-005': '' }));
    }} />
    {(['actions', 'instrument', 'fx'] as const).map(kind => <FinancialSourceSettings key={`${kind}-${workspaceId}`} workspaceId={workspaceId} kind={kind} onSourceChange={() => {
      setOverrides(current => { const next = { ...current }; delete next['OD-005']; return next; });
      setStaleSources(current => ({ ...current, 'OD-005': false }));
      setProbeErrors(current => ({ ...current, 'OD-005': '' }));
    }} />)}
    <BinanceRuleSourceSettings key={`binance-rules-${workspaceId}`} workspaceId={workspaceId} />
    <div className="data-source-grid">{catalog.data.sources.map(source => { const current = overrides[source.sourceId] ?? source; const stale = staleSources[source.sourceId] === true || sourceIsStale(current); return <SourceCard key={source.sourceId} source={current} stale={stale} busy={probing !== undefined} onProbe={() => void probe(current)} error={probeErrors[source.sourceId] || undefined} />; })}</div>
  </section>;
}
