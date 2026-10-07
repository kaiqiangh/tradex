import { useEffect, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import { FinancialEvidencePanel } from './FinancialEvidencePanel.tsx';

const sources = {
  fx: { title: 'Alpaca currency rates', account: 'Currency rates account', selection: 'Paper', explanation: 'Read only the currency pairs required by current account and workspace inputs, using a saved Alpaca Paper key. Supplied rates do not establish transaction-grade qualification or exact broker conversion costs.' },
  actions: { title: 'Alpaca known company events', account: 'Company events account', selection: 'Paper', explanation: 'Read processed company events using a saved Alpaca Paper key. A completed query does not establish complete action coverage or adjusted history.' },
  instrument: { title: 'Trading 212 account instruments', account: 'Account instruments account', selection: 'Live', explanation: 'Read account identity, instrument metadata and schedules using the same saved Trading 212 Live key. These reads do not establish current account tradability, exchange halts or full key permissions.' },
} as const;

export function FinancialSourceSettings({ workspaceId, kind, onSourceChange }: { workspaceId: string; kind: keyof typeof sources; onSourceChange: () => void }) {
  const config = sources[kind];
  const cacheKey = ['financial-source', workspaceId, kind];
  const queryClient = useQueryClient();
  const source = useQuery({ queryKey: cacheKey, queryFn: () => request(`data.${kind}.connection`, { workspaceId }), retry: false, refetchInterval: 1000 });
  const [accountId, setAccountId] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState<string>();
  useEffect(() => { setAccountId(source.data?.connectionId ?? ''); }, [source.data?.stateVersion]);
  const act = async (operation: 'configure' | 'refresh' | 'disconnect') => {
    if (!source.data || busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const payload = { workspaceId, expectedStateVersion: source.data.stateVersion };
      const result = operation === 'configure'
        ? await request(`data.${kind}.configure`, { ...payload, connectionId: accountId })
        : operation === 'refresh'
          ? await request(`data.${kind}.refresh`, payload)
          : await request(`data.${kind}.disconnect`, payload);
      queryClient.setQueryData(cacheKey, result);
      onSourceChange();
      await queryClient.invalidateQueries({ queryKey: ['data-source-catalog', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['market-detail', workspaceId] });
      if (operation === 'refresh' && result.status !== 'AVAILABLE') setError(result.availabilityReason);
      else setNotice(operation === 'configure' ? `${config.title} selection saved. Read access has not been verified.`
        : operation === 'disconnect' ? `${config.title} disconnected. Your account and saved key were kept.`
          : `${config.title} refreshed. Only the listed capabilities are current.`);
    } catch (cause) { setError(explainError(cause)); await source.refetch(); }
    finally { setBusy(false); }
  };
  if (source.isPending) return <p role="status">Loading {config.title.toLowerCase()}…</p>;
  if (source.isError) return <div role="alert"><p>{explainError(source.error)}</p><button type="button" onClick={() => void source.refetch()}>Reload {config.title.toLowerCase()}</button></div>;
  return <section className="card quote-source-settings" aria-labelledby={`${kind}-source-heading`}>
    <h3 id={`${kind}-source-heading`}>{config.title}</h3>
    <p>{config.explanation}</p>
    <label>{config.account}<select aria-label={config.account} value={accountId} disabled={busy} onChange={event => { setAccountId(event.target.value); setNotice(undefined); }}>
      <option value="">Choose a saved {config.selection} account</option>
      {source.data.eligibleAccounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.displayName}</option>)}
      {source.data.connectionId && !source.data.eligibleAccounts.some(account => account.connectionId === source.data.connectionId) && <option value={source.data.connectionId}>Saved account — reconnect to use this key</option>}
    </select></label>
    {accountId !== (source.data.connectionId ?? '') && <p>Save this selection before refreshing. The current evidence belongs to the saved selection.</p>}
    <FinancialEvidencePanel source={source.data} />
    {notice && <p role="status">{notice}</p>}
    {error && <div role="alert"><p>{error}</p><button type="button" disabled={busy} onClick={() => void source.refetch()}>Reload {config.title.toLowerCase()}</button></div>}
    <button type="button" disabled={busy || !source.data.eligibleAccounts.some(account => account.connectionId === accountId)} onClick={() => void act('configure')}>{busy ? 'Working…' : `Save ${config.title.toLowerCase()}`}</button>
    {source.data.configured && <><button type="button" disabled={busy || accountId !== source.data.connectionId} onClick={() => void act('refresh')}>Refresh {config.title.toLowerCase()}</button><button type="button" disabled={busy} onClick={() => void act('disconnect')}>Disconnect {config.title.toLowerCase()}</button></>}
  </section>;
}
