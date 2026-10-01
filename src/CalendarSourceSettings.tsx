import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { explainError, request } from './client.ts';

export function CalendarSourceSettings({ workspaceId, onSourceChange }: { workspaceId: string; onSourceChange: () => void }) {
  const queryClient = useQueryClient();
  const source = useQuery({ queryKey: ['calendar-source', workspaceId], queryFn: () => request('data.calendar.connection', { workspaceId }), retry: false, refetchInterval: 1000 });
  const [accountId, setAccountId] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState<string>();
  useEffect(() => { setAccountId(source.data?.connectionId ?? ''); setError(undefined); setNotice(undefined); }, [source.data?.stateVersion, workspaceId]);
  const mutate = async (disconnect: boolean) => {
    if (!source.data || busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const expectedStateVersion = source.data.stateVersion;
      if (disconnect) await request('data.calendar.disconnect', { workspaceId, expectedStateVersion });
      else await request('data.calendar.configure', { workspaceId, expectedStateVersion, connectionId: accountId });
      await source.refetch();
      onSourceChange();
      await queryClient.invalidateQueries({ queryKey: ['data-source-catalog', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['market-detail', workspaceId] });
      setNotice(disconnect ? 'Calendar source disconnected. Your account and its saved key were kept.' : 'Calendar source selection saved. Calendar access has not been verified.');
    } catch (cause) { setError(explainError(cause)); await source.refetch(); }
    finally { setBusy(false); }
  };
  const refresh = async () => {
    if (!source.data || busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const result = await request('data.calendar.refresh', { workspaceId, expectedStateVersion: source.data.stateVersion });
      queryClient.setQueryData(['calendar-source', workspaceId], result);
      onSourceChange();
      await queryClient.invalidateQueries({ queryKey: ['data-source-catalog', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['market-detail', workspaceId] });
      if (result.status === 'AVAILABLE') setNotice('XNAS calendar refreshed. Only its calendar capability is current.');
      else setError(result.availabilityReason);
    } catch (cause) { setError(explainError(cause)); await source.refetch(); }
    finally { setBusy(false); }
  };
  if (source.isPending) return <p role="status">Loading calendar source…</p>;
  if (source.isError) return <div role="alert"><p>{explainError(source.error)}</p><button type="button" onClick={() => void source.refetch()}>Reload calendar source</button></div>;
  return <section className="card quote-source-settings" aria-labelledby="calendar-source-heading">
    <h3 id="calendar-source-heading">Alpaca XNAS calendar</h3>
    <p>Select a saved Alpaca Paper key for read-only calendar access on its fixed Paper host. Calendar, corporate actions and trading eligibility are separate checks.</p>
    <label>Calendar account<select aria-label="Calendar account" value={accountId} disabled={busy} onChange={event => setAccountId(event.target.value)}>
      <option value="">Choose a saved Paper account</option>
      {source.data.eligibleAccounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.displayName}</option>)}
      {source.data.connectionId && !source.data.eligibleAccounts.some(account => account.connectionId === source.data.connectionId) && <option value={source.data.connectionId}>Saved account — reconnect to use this key</option>}
    </select></label>
    <p role="status">{source.data.availabilityReason}</p>
    <dl className="data-source-details">
      <div><dt>Calendar receipt</dt><dd>{source.data.observedAt ?? 'Not checked'}</dd></div>
      <div><dt>Calendar version</dt><dd>{source.data.calendarVersion ?? 'Unavailable'}</dd></div>
      <div><dt>Requested coverage</dt><dd>{source.data.coverageStart && source.data.coverageEnd ? `${source.data.coverageStart} – ${source.data.coverageEnd}` : 'Unavailable'}</dd></div>
      {source.data.capabilityStatuses.map(capability => <div key={capability.capability}><dt>{capability.capability.replaceAll('_', ' ')}</dt><dd>{capability.status.replaceAll('_', ' ')} · {capability.reason}</dd></div>)}
    </dl>
    {notice && <p role="status">{notice}</p>}
    {error && <div role="alert"><p>{error}</p><button type="button" disabled={busy} onClick={() => void source.refetch()}>Reload calendar source</button></div>}
    <button type="button" disabled={busy || !source.data.eligibleAccounts.some(account => account.connectionId === accountId)} onClick={() => void mutate(false)}>{busy ? 'Working…' : 'Save calendar source'}</button>
    {source.data.configured && <><button type="button" disabled={busy || accountId !== source.data.connectionId} onClick={() => void refresh()}>Refresh selected calendar</button><button type="button" disabled={busy} onClick={() => void mutate(true)}>Disconnect calendar source</button></>}
  </section>;
}
