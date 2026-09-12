import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import type { GatewayAction } from '../shared/ipc-types.ts';
import { request, explainError } from './client.ts';
import { fromGatewaySnapshot } from './projection.ts';
import { useDomainProjection } from './useDomainProjection.ts';

const states = { STOPPED: 'Stopped', INSTALLING: 'Installing', STARTING: 'Starting / probing', RUNNING: 'Running', PORT_CONFLICT: 'Port conflict', UNAUTHORIZED: 'Unauthorized', BACKOFF: 'Waiting to restart', FAILED: 'Failed', STOPPING: 'Stopping' };
const reasons: Record<string, string> = {
  GATEWAY_CLEANUP_FAILED: 'The process has stopped, but its private configuration could not be removed. Check directory permissions and retry Stop.',
  GATEWAY_BUSY: 'Another TradeX process is managing the gateway. Stop it there before launching here.',
  GATEWAY_PORT_CONFLICT: 'Port 8317 is occupied. Close or reconfigure that service, then launch again.',
  GATEWAY_UNAUTHORIZED: 'The owned gateway rejected authentication. Restart to regenerate its local configuration.',
  GATEWAY_BINARY_INVALID: 'The gateway does not match the pinned release. A verified installation is required.',
  GATEWAY_NOT_INSTALLED: 'The pinned gateway has not been installed.',
  GATEWAY_INSTALL_FAILED: 'Installation failed. Check network access and storage, then retry Launch.',
  GATEWAY_STORAGE_FAILED: 'The private model directory could not be used. Check its access permissions.',
  GATEWAY_PLATFORM_UNSUPPORTED: 'This gateway build is unavailable for the current platform.',
  GATEWAY_STOPPED: 'Launch the gateway before probing it.',
  GATEWAY_PROCESS_FAILED: 'The gateway process stopped or could not start. Try Restart.',
  GATEWAY_PROBE_FAILED: 'The gateway health probe failed. Try Probe again or Restart.',
};

export function Models({ workspaceId }: { workspaceId: string }) {
  const gateway = useDomainProjection('model-gateway', workspaceId, fromGatewaySnapshot);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const state = gateway.data;
  const queryClient = useQueryClient();
  useEffect(() => { void queryClient.invalidateQueries({queryKey:['runtime']}); },[state?.stateVersion,queryClient]);
  async function act(action: GatewayAction) {
    if (!state || busy) return;
    setBusy(true); setError(null);
    try { await request('model.gateway', { workspaceId, expectedStateVersion: state.stateVersion, action }); }
    catch (error) { setError(error); }
    finally { setBusy(false); await gateway.reload(); }
  }
  return <section className="model-settings" aria-labelledby="model-title">
    <h3 id="model-title">Models · CLIProxyAPI</h3>
    <p>Model inference uses an external provider through your local gateway. Broker credentials stay separate.</p>
    {(error || gateway.error) != null && <p role="alert">{explainError(error || gateway.error)}</p>}
    {state ? <>
      <p role="status"><strong>{states[state.status]}</strong> · Pinned version {state.pinnedVersion}</p>
      <p>Endpoint: <code>{state.endpoint}</code></p>
      <p>{state.installed ? 'Pinned binary verified' : 'Pinned binary not yet verified'} · Discovered models: {state.discoveredModelCount}</p>
      <p>Last successful probe: {state.lastProbeAt ?? 'Not verified'}</p>
      {state.nextRetryAt && <p>Next restart: {state.nextRetryAt} · Attempt {state.restartAttempts} of 3</p>}
      {state.errorCode && <p role="alert">{reasons[state.errorCode] ?? 'The model gateway needs attention. Retry after checking its configuration.'}</p>}
      <div className="model-actions">{(['LAUNCH', 'PROBE', 'RESTART', 'STOP'] as const).map(action =>
        <button key={action} disabled={busy} onClick={() => { void act(action); }}>{action === 'LAUNCH' ? (state.installed ? 'Launch' : 'Install & Launch') : action === 'PROBE' ? 'Probe' : action === 'RESTART' ? 'Restart' : 'Stop'}</button>)}</div>
      {busy && <p role="status">Updating model gateway…</p>}
      <p>No verified model route. Agent turns and onboarding Ready remain unavailable.</p>
    </> : <p role="status">Loading model gateway…</p>}
    <button onClick={() => { void gateway.reload(); }}>Reload gateway state</button>
  </section>;
}
