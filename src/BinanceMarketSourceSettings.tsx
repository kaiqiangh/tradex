import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';

export function BinanceMarketSourceSettings({ workspaceId }: { workspaceId: string }) {
  const client = useQueryClient();
  const key = ['binance-market-source', workspaceId];
  const connection = useQuery({ queryKey: key, queryFn: () => request('data.binance_market.connection', { workspaceId }), retry: false, refetchInterval: 1000 });
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string>();
  const [error, setError] = useState<string>();
  const act = async (operation: 'configure' | 'disconnect') => {
    if (!connection.data || busy) return;
    setBusy(true); setNotice(undefined); setError(undefined);
    try {
      const payload = { workspaceId, expectedStateVersion: connection.data.stateVersion };
      const result = operation === 'configure' ? await request('data.binance_market.configure', payload)
        : await request('data.binance_market.disconnect', payload);
      client.setQueryData(key, result);
      await client.invalidateQueries({ queryKey: ['data-source-catalog', workspaceId] });
      await client.invalidateQueries({ queryKey: ['market-detail', workspaceId] });
      setNotice(operation === 'configure' ? 'Public market-source selection saved. No provider read was made.'
        : 'Public market source disconnected. Account connections and saved keys were kept.');
    } catch (cause) {
      setError(explainError(cause));
      await connection.refetch();
    } finally { setBusy(false); }
  };
  if (connection.isPending) return <p role="status">Loading Binance market source…</p>;
  if (connection.isError) return <div role="alert"><p>{explainError(connection.error)}</p><button type="button" onClick={() => void connection.refetch()}>Reload Binance market source</button></div>;
  const source = connection.data.source;
  return <section className="card quote-source-settings" aria-labelledby="binance-market-heading">
    <h3 id="binance-market-heading">Binance Spot market source</h3>
    <p>Public BTC / USDT and ETH / USDT depth from the ordinary Binance Spot service. No trading account or key is needed for this market source.</p>
    <dl className="data-source-details">
      <div><dt>Selection</dt><dd>{connection.data.configured ? 'Saved' : 'Not selected'}</dd></div>
      <div><dt>Technical collection</dt><dd>{source.status === 'AVAILABLE' ? 'Available in the current Hot view' : source.status === 'UNVERIFIED' ? 'Not verified' : 'Unavailable'}</dd></div>
      <div><dt>Financial data use</dt><dd>UNVERIFIED — financial use, retention, redistribution, commercial use and regional permission remain unverified.</dd></div>
    </dl>
    <p role="status">{source.availabilityReason}</p>
    <p className="form-hint">Save records the selection only. A visible BTC / USDT or ETH / USDT Hot view owns its bounded stream; a connected socket is not a quote, authentication or permission. Saving again or disconnecting retires current market evidence.</p>
    {notice && <p role="status" className="success-text">{notice}</p>}
    {error && <div role="alert"><p>{error}</p><button type="button" disabled={busy} onClick={() => void connection.refetch()}>Reload Binance market source</button></div>}
    <button type="button" disabled={busy} onClick={() => void act('configure')}>{busy ? 'Working…' : 'Save Binance market source'}</button>
    {connection.data.configured && <button type="button" disabled={busy} onClick={() => void act('disconnect')}>Disconnect Binance market source</button>}
  </section>;
}
