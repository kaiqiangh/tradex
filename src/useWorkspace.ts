import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { OpenWorkspace } from '../shared/ipc-types.ts';
import { request, subscribe, transportAvailable } from './client.ts';
import { applyEvent, fromSnapshot } from './projection.ts';
import type { Projection } from './projection.ts';

function rememberedPath(): string | undefined {
  try { return localStorage.getItem('tradex.workspace.path') ?? undefined; } catch { return undefined; }
}

export function useWorkspace() {
  const queryClient = useQueryClient();
  const [workspaceId, setWorkspaceId] = useState<string>();
  const [projection, setProjection] = useState<Projection | null>(null);
  const [streamError, setStreamError] = useState<unknown>(null);
  const runtime = useQuery({ queryKey: ['runtime'], queryFn: () => request('runtime.status', {}), enabled: transportAvailable, refetchInterval: 10_000 });
  const snapshot = useQuery({
    queryKey: ['workspace', workspaceId], enabled: Boolean(workspaceId),
    queryFn: () => request('domain.snapshot', { aggregateType: 'workspace', aggregateId: workspaceId! }),
  });
  const opening = useMutation({
    mutationFn: (options: OpenWorkspace) => request('workspace.open', options),
    onSuccess: workspace => {
      try { localStorage.setItem('tradex.workspace.path', workspace.path); } catch { /* Workspace persistence belongs to Rust; this is only a navigation hint. */ }
      setWorkspaceId(workspace.workspaceId);
      void queryClient.invalidateQueries({ queryKey: ['workspace', workspace.workspaceId] });
    },
  });
  useEffect(() => {
    const path = rememberedPath();
    if (path && transportAvailable) opening.mutate({ path });
    // One restore attempt per app mount; subsequent retries are explicit user actions.
  }, []);

  useEffect(() => {
    if (!snapshot.data || snapshot.isError) { setProjection(null); return; }
    let current: Projection;
    let active = true;
    let ready = false;
    let stop = () => {};
    const fail = (error: unknown) => {
      if (!active) return;
      active = false;
      stop();
      setProjection(null);
      setStreamError(error);
    };
    try { current = fromSnapshot(snapshot.data); } catch (error) { fail(error); return; }
    setProjection(null);
    setStreamError(null);
    void subscribe({ aggregateType: 'workspace', aggregateId: current.snapshot.aggregateId, afterSequence: current.snapshot.lastSequence }, value => {
      if (!active) return;
      try { current = applyEvent(current, value); if (ready) setProjection(current); } catch (error) {
        fail(error);
        if (error instanceof Error && error.message.startsWith('IPC_SEQUENCE')) {
          void queryClient.invalidateQueries({ queryKey: ['workspace', workspaceId] });
        }
      }
    }, fail).then(close => {
      if (active) { stop = close; ready = true; setProjection(current); }
      else close();
    }).catch(fail);
    return () => { active = false; stop(); };
  }, [snapshot.data, snapshot.dataUpdatedAt, snapshot.isError, workspaceId, queryClient]);

  return {
    workspace: projection && projection.snapshot.aggregateId === workspaceId ? projection.snapshot.projection : undefined,
    runtime, opening,
    error: opening.error ?? snapshot.error ?? streamError ?? runtime.error,
    recover: () => {
      opening.reset();
      if (workspaceId) void snapshot.refetch();
      else { const path = rememberedPath(); if (path) opening.mutate({ path }); }
      void runtime.refetch();
    },
  };
}
