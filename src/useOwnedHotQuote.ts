import { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import type { HotQuoteProjection, HotQuoteQuery } from '../shared/ipc-types.ts';
import { request } from './client.ts';

// Each visible consumer owns its lease, including an acquire completing after cleanup.
export function useOwnedHotQuote(workspaceId: string, instrumentId: string, enabled = true) {
  const equity = instrumentId.startsWith('equity:');
  const spot = ['crypto:BTC/USDT:spot', 'crypto:ETH/USDT:spot'].includes(instrumentId);
  const stockSource = useQuery({ queryKey: ['data-source-connection', workspaceId], queryFn: () => request('data.source.connection', { workspaceId }), enabled: enabled && equity, retry: false, refetchInterval: 1000 });
  const spotSource = useQuery({ queryKey: ['binance-market-source', workspaceId], queryFn: () => request('data.binance_market.connection', { workspaceId }), enabled: enabled && spot, retry: false, refetchInterval: 1000 });
  const sourceVersion = spot ? spotSource.data?.stateVersion : stockSource.data?.stateVersion;
  const configured = enabled && (spot ? Boolean(spotSource.data?.configured) : equity && Boolean(stockSource.data?.feed && stockSource.data?.credentialKind));
  const sourceReady = spot ? spotSource.isSuccess : equity ? stockSource.isSuccess : true;
  const [hot, setHot] = useState<HotQuoteProjection>();
  const [hotError, setHotError] = useState<unknown>();
  const [revision, setRevision] = useState(0);
  const [visible, setVisible] = useState(document.visibilityState !== 'hidden');
  useEffect(() => {
    const changed = () => setVisible(document.visibilityState !== 'hidden');
    document.addEventListener('visibilitychange', changed);
    return () => document.removeEventListener('visibilitychange', changed);
  }, []);
  useEffect(() => {
    setHot(undefined); setHotError(undefined);
    if (!configured || !visible || !sourceVersion) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let lease: HotQuoteQuery | undefined;
    const release = () => {
      if (!lease) return;
      const captured = lease; lease = undefined;
      void request('market.hot.release', captured).catch(() => {});
    };
    const poll = async () => {
      if (cancelled || !lease) return;
      try {
        const current = await request('market.hot.get', lease);
        if (cancelled) return;
        setHot(current);
        if (!['CLOSED', 'FAILED', 'STALE'].includes(current.status)) timer = setTimeout(() => void poll(), 500);
      } catch (error) { if (!cancelled) { setHotError(error); release(); } }
    };
    void request('market.hot.acquire', { workspaceId, instrumentId, expectedSourceVersion: sourceVersion }).then(current => {
      lease = { workspaceId, leaseId: current.leaseId, generation: current.generation };
      if (cancelled) { release(); return; }
      setHot(current); void poll();
    }).catch(error => { if (!cancelled) setHotError(error); });
    return () => { cancelled = true; if (timer) clearTimeout(timer); release(); };
  }, [workspaceId, instrumentId, configured, sourceVersion, visible, revision]);
  const sourceError = spot ? spotSource.error : equity ? stockSource.error : undefined;
  const retry = () => {
    if (spot) void spotSource.refetch(); else if (equity) void stockSource.refetch();
    setRevision(value => value + 1);
  };
  const current = visible && configured && hot?.workspaceId === workspaceId && hot.instrumentId === instrumentId && hot.sourceVersion === sourceVersion ? hot : undefined;
  return { configured, sourceReady, sourceVersion, hot: current, error: sourceError ?? hotError, visible, retry, publicSpot: spot, feed: equity ? stockSource.data?.feed : undefined };
}
