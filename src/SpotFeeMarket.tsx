import { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import type { HotQuoteQuery, SpotFeeMarketProjection } from '../shared/ipc-types.ts';

export function SpotFeeMarket({ workspaceId }: { workspaceId: string }) {
  const source = useQuery({ queryKey: ['binance-market-source', workspaceId], queryFn: () => request('data.binance_market.connection', { workspaceId }), refetchInterval: 1000, retry: false });
  const [read, setRead] = useState(0);
  const [released, setReleased] = useState(false);
  const [current, setCurrent] = useState<SpotFeeMarketProjection>();
  const [failure, setFailure] = useState<unknown>();
  const [visible, setVisible] = useState(document.visibilityState !== 'hidden');
  useEffect(() => {
    const changed = () => setVisible(document.visibilityState !== 'hidden');
    document.addEventListener('visibilitychange', changed);
    return () => document.removeEventListener('visibilitychange', changed);
  }, []);
  const configured = source.data?.configured;
  const version = source.data?.stateVersion;
  useEffect(() => {
    setCurrent(undefined); setFailure(undefined);
    if (!read || released || !visible || !configured || !version) return;
    let cancelled = false;
    let lease: HotQuoteQuery | undefined;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const release = () => {
      if (!lease) return;
      const token = lease; lease = undefined;
      void request('trade.spot_fee_market.release', token).catch(() => {});
    };
    const poll = async () => {
      if (cancelled || !lease) return;
      try {
        const value = await request('trade.spot_fee_market.get', lease);
        if (cancelled) return;
        setCurrent(value);
        if (!['CLOSED', 'FAILED', 'STALE'].includes(value.status)) timer = setTimeout(() => void poll(), 250);
        else release();
      } catch (error) { if (!cancelled) { setFailure(error); release(); } }
    };
    void request('trade.spot_fee_market.acquire', { workspaceId, expectedSourceVersion: version }).then(value => {
      lease = { workspaceId, leaseId: value.leaseId, generation: value.generation };
      if (cancelled) { release(); return; }
      setCurrent(value); void poll();
    }).catch(error => { if (!cancelled) setFailure(error); });
    return () => { cancelled = true; if (timer) clearTimeout(timer); release(); };
  }, [workspaceId, configured, version, read, released, visible]);
  const snapshot = current?.snapshot;
  const metadata = current?.metadata;
  const rights = snapshot?.provenance.binance?.dataUseRights;
  return <section className="card" aria-label="Current BNB fee source">
    <h3>Current BNB fee source</h3>
    <p><strong>{current?.status ?? (released ? 'CLOSED' : read && !visible ? 'PAUSED' : read ? 'CONNECTING' : 'NOT_READ')} · Read-only fee inputs · Execution qualification unavailable</strong></p>
    <p>This fixed BNB/USDT source supports fee estimates and BNB policy valuation only. It never becomes a tradable instrument or sends an order. Zero-discounted-expression and proven-zero-BNB branches need no unused conversion.</p>
    <p>{configured ? 'Ordinary Binance public source selected.' : 'Select the ordinary Binance public market source in Settings Data & Storage before reading.'}</p>
    <button type="button" disabled={!configured || !visible} onClick={() => { setReleased(false); setRead(value => value + 1); }}>Read BNB fee source</button>
    {read > 0 && !released && <button type="button" onClick={() => setReleased(true)}>Release BNB fee source</button>}
    {released && <p role="status">Fee source released.</p>}
    <p>{current?.reason ?? (released ? 'Read again to obtain new current evidence. The released source provides no current fee input.' : 'No fee book has been read. Public access and a continuous book do not establish financial data-use rights, funding, approval or execution.')}</p>
    {Boolean(failure || source.error) && <p role="alert">{explainError(failure ?? source.error)}</p>}
    {metadata && <dl className="data-source-details">
      <div><dt>Declared fee pair · market status</dt><dd>{metadata.providerSymbol} · {metadata.baseAsset} / {metadata.quoteAsset} · {metadata.marketStatus}</dd></div>
      <div><dt>Metadata response completion</dt><dd>{metadata.receivedAt}. No provider observation timestamp was supplied.</dd></div>
      <div><dt>Original metadata digest</dt><dd>{metadata.originalDigest}</dd></div>
    </dl>}
    {snapshot && <dl className="data-source-details">
      <div><dt>BNB bid / ask in USDT</dt><dd>{snapshot.bid} / {snapshot.ask}</dd></div>
      <div><dt>Displayed bid / ask size in BASE</dt><dd>{snapshot.bidSize} / {snapshot.askSize} BNB</dd></div>
      <div><dt>Provider time / first receipt</dt><dd>{snapshot.provenance.providerTimestamp} / {snapshot.provenance.receivedTimestamp}</dd></div>
      <div><dt>Financial data-use status</dt><dd>{rights}. Source use and every financial gate remain separate.</dd></div>
      <div><dt>Source / snapshot</dt><dd>{snapshot.provenance.source} · {snapshot.provenance.marketSnapshotId}</dd></div>
      <div><dt>Lease / connection generation</dt><dd>{current?.leaseId} · {current?.connectionGeneration}</dd></div>
    </dl>}
  </section>;
}
