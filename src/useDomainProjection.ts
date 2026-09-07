import { useEffect, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { AccountConnection, Workspace } from '../shared/ipc-types.ts';
import { request, subscribe } from './client.ts';
import { applyEvent } from './projection.ts';
import type { Projection } from './projection.ts';

export function useDomainProjection<T extends Workspace | AccountConnection>(kind: 'workspace' | 'account', id: string | undefined, read: (value: unknown) => Projection<T>) {
  const queryClient = useQueryClient();
  const [projection, setProjection] = useState<Projection<T> | null>(null);
  const [streamError, setStreamError] = useState<unknown>(null);
  const snapshot = useQuery({
    queryKey: [kind, id], enabled: Boolean(id),
    queryFn: () => request('domain.snapshot', { aggregateType: kind, aggregateId: id! }),
  });
  useEffect(() => {
    if (!snapshot.data || snapshot.isError) { setProjection(null); return; }
    let current: Projection<T>;
    let active = true;
    let ready = false;
    let stop = () => {};
    const fail = (error: unknown) => {
      if (!active) return;
      active = false; stop(); setProjection(null); setStreamError(error);
    };
    try { current = read(snapshot.data); } catch (error) { fail(error); return; }
    setProjection(null); setStreamError(null);
    void subscribe({ aggregateType: kind, aggregateId: current.snapshot.aggregateId, afterSequence: current.snapshot.lastSequence }, value => {
      if (!active) return;
      try { current = applyEvent(current, value); if (ready) setProjection(current); } catch (error) {
        fail(error);
        if (error instanceof Error && error.message.startsWith('IPC_SEQUENCE')) void queryClient.invalidateQueries({ queryKey: [kind, id] });
      }
    }, fail).then(close => {
      if (active) { stop = close; ready = true; setProjection(current); } else close();
    }).catch(fail);
    return () => { active = false; stop(); };
  }, [snapshot.data, snapshot.dataUpdatedAt, snapshot.isError, kind, id, queryClient, read]);
  return { data: projection?.snapshot.aggregateId === id ? projection?.snapshot.projection : undefined, error: snapshot.error ?? streamError, reload: () => snapshot.refetch() };
}
