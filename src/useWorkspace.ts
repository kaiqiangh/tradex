import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { OpenWorkspace } from '../shared/ipc-types.ts';
import { request, transportAvailable } from './client.ts';
import { fromSnapshot } from './projection.ts';
import { useDomainProjection } from './useDomainProjection.ts';

function rememberedPath(): string | undefined {
  try { return localStorage.getItem('tradex.workspace.path') ?? undefined; } catch { return undefined; }
}

export function useWorkspace() {
  const queryClient = useQueryClient();
  const [workspaceId, setWorkspaceId] = useState<string>();
  const projection = useDomainProjection('workspace', workspaceId, fromSnapshot);
  const runtime = useQuery({ queryKey: ['runtime'], queryFn: () => request('runtime.status', {}), enabled: transportAvailable, refetchInterval: 10_000 });
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


  return {
    workspace: projection.data,
    runtime, opening,
    error: opening.error ?? projection.error ?? runtime.error,
    recover: () => {
      opening.reset();
      if (workspaceId) void projection.reload();
      else { const path = rememberedPath(); if (path) opening.mutate({ path }); }
      void runtime.refetch();
    },
  };
}
