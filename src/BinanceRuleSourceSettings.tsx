import { useEffect, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import { FinancialEvidencePanel } from './FinancialEvidencePanel.tsx';

export function BinanceRuleSourceSettings({ workspaceId }: { workspaceId: string }) {
  const client = useQueryClient();
  const key = ['binance-rule-source', workspaceId];
  const source = useQuery({ queryKey: key, queryFn: () => request('data.binance_rules.connection', { workspaceId }), retry: false, refetchInterval: 1000 });
  const [account, setAccount] = useState('');
  const [instrument, setInstrument] = useState('crypto:BTC/USDT:spot');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState<string>();
  useEffect(() => { setAccount(source.data?.connectionId ?? ''); setInstrument(source.data?.instrumentId ?? 'crypto:BTC/USDT:spot'); }, [source.data?.stateVersion]);
  const unsaved = account !== (source.data?.connectionId ?? '') || instrument !== (source.data?.instrumentId ?? 'crypto:BTC/USDT:spot');
  const act = async (operation: 'configure' | 'refresh' | 'disconnect') => {
    if (!source.data || busy) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const base = { workspaceId, expectedStateVersion: source.data.stateVersion };
      const result = operation === 'configure' ? await request('data.binance_rules.configure', { ...base, connectionId: account, instrumentId: instrument })
        : operation === 'refresh' ? await request('data.binance_rules.refresh', base) : await request('data.binance_rules.disconnect', base);
      client.setQueryData(key, result);
      await client.invalidateQueries({ queryKey: ['market-detail', workspaceId] });
      if (operation === 'refresh' && result.status !== 'AVAILABLE') setError(result.availabilityReason);
      else setNotice(operation === 'configure' ? 'Rule-source selection saved. No provider read was made.' : operation === 'disconnect' ? 'Rule source disconnected. Your account and saved key were kept.' : 'Rule observations refreshed. Execution qualification and data rights remain separate.');
    } catch (cause) { setError(explainError(cause)); await source.refetch(); }
    finally { setBusy(false); }
  };
  if (source.isPending) return <p role="status">Loading Binance Spot rule source…</p>;
  if (source.isError) return <div role="alert"><p>{explainError(source.error)}</p><button type="button" onClick={() => void source.refetch()}>Reload Binance rule source</button></div>;
  return <section className="card quote-source-settings" aria-labelledby="binance-rule-heading">
    <h3 id="binance-rule-heading">Binance Spot rule source</h3>
    <p>Read exact Spot rules and account admission using a saved ordinary Binance Live key. These observations do not supply a quote, licence, per-order rule qualification or Live readiness.</p>
    <label>Binance Live rule account<select aria-label="Binance Live rule account" value={account} disabled={busy} onChange={event => { setAccount(event.target.value); setNotice(undefined); }}>
      <option value="">Choose a saved Binance Live account</option>
      {source.data.eligibleAccounts.map(a => <option key={a.connectionId} value={a.connectionId}>{a.displayName}</option>)}
      {source.data.connectionId && !source.data.eligibleAccounts.some(a => a.connectionId === source.data.connectionId) && <option value={source.data.connectionId}>Saved account — reconnect before reading</option>}
    </select></label>
    <label>Spot rule instrument<select aria-label="Spot rule instrument" value={instrument} disabled={busy} onChange={event => { setInstrument(event.target.value); setNotice(undefined); }}>
      <option value="crypto:BTC/USDT:spot">BTC / USDT</option><option value="crypto:ETH/USDT:spot">ETH / USDT</option>
    </select></label>
    {unsaved && <p>Save the account and instrument selection before refreshing. The displayed evidence belongs to the saved selection.</p>}
    <FinancialEvidencePanel source={source.data} />
    {notice && <p role="status">{notice}</p>}
    {error && <div role="alert"><p>{error}</p><button type="button" disabled={busy} onClick={() => void source.refetch()}>Reload Binance rule source</button></div>}
    <button type="button" disabled={busy || !source.data.eligibleAccounts.some(a => a.connectionId === account)} onClick={() => void act('configure')}>{busy ? 'Working…' : 'Save Binance rule source'}</button>
    {source.data.configured && <><button type="button" disabled={busy || unsaved} onClick={() => void act('refresh')}>Refresh Binance rule source</button><button type="button" disabled={busy} onClick={() => void act('disconnect')}>Disconnect Binance rule source</button></>}
  </section>;
}
