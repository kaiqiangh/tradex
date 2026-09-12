import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import type { ChatgptLoginAction, GatewayAction, ModelProviderState, ModelRoute, ThinkingType } from '../shared/ipc-types.ts';
import { request, explainError } from './client.ts';
import { fromGatewaySnapshot, fromModelSnapshot } from './projection.ts';
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
  MODEL_NATIVE_REQUIRED: 'Open the desktop app to configure this model securely.',
  MODEL_NATIVE_ENTRY_REQUIRED: 'Open the desktop app to configure this model securely.',
  MODEL_KEYCHAIN_MISSING: 'The DeepSeek OS Keychain key is unavailable. Configure it again.',
  MODEL_UNAVAILABLE: 'The selected model route is unavailable. Retry its probe or choose another route.',
  MODEL_OAUTH_EXPIRED: 'ChatGPT authorization expired or was rejected. Re-login before verifying the route.',
  MODEL_QUOTA_EXCEEDED: 'The model provider reported a quota limit. Wait before retrying.',
  MODEL_TEST_INFERENCE_FAILED: 'The selected route did not complete the bounded setup inference.',
};
const modelHealth: Record<string, string> = {
  NOT_CONFIGURED: 'Not configured',
  UNVERIFIED: 'Configured · verification required',
  VERIFYING: 'Verifying',
  READY: 'Ready',
  FAILED: 'Failed',
};

function routeLabel(route: ModelRoute) {
  return `${route.modelId}${route.thinkingType ? ` · ${route.thinkingType === 'enabled' ? 'Thinking' : 'Non-thinking'}` : ''}`;
}

function providerError(provider: ModelProviderState) {
  return provider.errorCode ? reasons[provider.errorCode] ?? `The ${provider.provider === 'CHATGPT' ? 'ChatGPT' : 'DeepSeek'} route needs attention.` : null;
}

export function Models({ workspaceId }: { workspaceId: string }) {
  const gateway = useDomainProjection('model-gateway', workspaceId, fromGatewaySnapshot);
  const model = useDomainProjection('model', workspaceId, fromModelSnapshot);
  const [busy, setBusy] = useState(false);
  const [modelBusy, setModelBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const state = gateway.data;
  const modelState = model.data;
  const queryClient = useQueryClient();
  useEffect(() => { void queryClient.invalidateQueries({ queryKey: ['runtime'] }); }, [state?.stateVersion, modelState?.stateVersion, queryClient]);
  async function act(action: GatewayAction) {
    if (!state || busy) return;
    setBusy(true); setError(null);
    try { await request('model.gateway', { workspaceId, expectedStateVersion: state.stateVersion, action }); }
    catch (error) { setError(error); }
    finally { setBusy(false); await gateway.reload(); }
  }
  async function modelAct(command: 'model.login_chatgpt' | 'model.configure_deepseek' | 'model.verify_route', payload: { action: ChatgptLoginAction } | { provider: 'CHATGPT' | 'DEEPSEEK'; modelId: string; thinkingType: ThinkingType | null } | null) {
    if (!modelState || modelBusy) return;
    setModelBusy(true); setError(null);
    try {
      if (command === 'model.login_chatgpt') await request(command, { workspaceId, expectedStateVersion: modelState.stateVersion, action: (payload as { action: ChatgptLoginAction }).action });
      else if (command === 'model.configure_deepseek') await request(command, { workspaceId, expectedStateVersion: modelState.stateVersion });
      else await request(command, { workspaceId, expectedStateVersion: modelState.stateVersion, ...(payload as { provider: 'CHATGPT' | 'DEEPSEEK'; modelId: string; thinkingType: ThinkingType | null }) });
    } catch (error) { setError(error); }
    finally { setModelBusy(false); await model.reload(); await gateway.reload(); }
  }
  const gatewayRunning = state?.status === 'RUNNING';
  const verify = (provider: 'CHATGPT' | 'DEEPSEEK', modelId: string, thinkingType: ThinkingType | null) => { void modelAct('model.verify_route', { provider, modelId, thinkingType }); };
  return <section className="model-settings" aria-labelledby="model-title">
    <h3 id="model-title">Models · CLIProxyAPI</h3>
    <p>Model inference uses an external provider through your local gateway. Broker credentials stay separate.</p>
    {(error || gateway.error || model.error) != null && <p role="alert">{explainError(error || gateway.error || model.error)}</p>}
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
      {modelState ? <div className="model-providers" aria-label="Model providers">
        <article className="model-provider-card">
          <div className="model-provider-heading"><div><h4>CLIProxyAPI → ChatGPT</h4><p>ChatGPT subscription OAuth · GPT-5.6 series</p></div><span className="badge">{modelHealth[modelState.chatgpt.status]}</span></div>
          {providerError(modelState.chatgpt) && <p className="model-error" role="alert">{providerError(modelState.chatgpt)}</p>}
          <div className="model-provider-actions"><button disabled={modelBusy || !gatewayRunning} onClick={() => { void modelAct('model.login_chatgpt', { action: modelState.chatgpt.configured ? 'RELOGIN' : 'LOGIN' }); }}>{modelState.chatgpt.configured ? 'Re-login ChatGPT' : 'Login ChatGPT'}</button></div>
          {!gatewayRunning && <p className="form-hint">Launch the pinned gateway before starting OAuth or verifying a route.</p>}
          <div className="model-routes">
            {modelState.chatgpt.routes.length === 0 ? <p className="form-hint">No allowed GPT-5.6 models discovered yet.</p> : modelState.chatgpt.routes.map(route => <div className="model-route" key={route.modelId}>
              <div><strong>{routeLabel(route)}</strong><small>{route.verifiedAt ? `Verified ${route.verifiedAt}` : 'Discovered · verification required'}</small></div>
              <button disabled={modelBusy || !gatewayRunning} onClick={() => verify('CHATGPT', route.modelId, null)}>Verify route</button>
            </div>)}
          </div>
        </article>
        <article className="model-provider-card">
          <div className="model-provider-heading"><div><h4>CLIProxyAPI → DeepSeek</h4><p>Official API · key stored only in macOS Keychain</p></div><span className="badge">{modelHealth[modelState.deepseek.status]}</span></div>
          {providerError(modelState.deepseek) && <p className="model-error" role="alert">{providerError(modelState.deepseek)}</p>}
          <div className="model-provider-actions"><button disabled={modelBusy || gatewayRunning} onClick={() => { void modelAct('model.configure_deepseek', null); }}>{modelState.deepseek.configured ? 'Replace DeepSeek key' : 'Configure DeepSeek key'}</button></div>
          <p className="form-hint">The native secure dialog writes directly to Keychain. The key never enters the webview, SQLite, events or logs. Stop the gateway before changing the key, then launch it again to render the new key into its private config.</p>
          <div className="model-routes">{(['disabled', 'enabled'] as const).map(mode => {
            const route = modelState.deepseek.routes.find(item => item.modelId === 'deepseek-v4-flash' && item.thinkingType === mode);
            return <div className="model-route" key={mode}>
              <div><strong>deepseek-v4-flash · {mode === 'enabled' ? 'Thinking' : 'Non-thinking'}</strong><small>{route?.verifiedAt ? `Verified ${route.verifiedAt}` : 'Explicit mode · verification required'}</small></div>
              <button disabled={modelBusy || !gatewayRunning || !modelState.deepseek.configured} onClick={() => verify('DEEPSEEK', 'deepseek-v4-flash', mode)}>Verify route</button>
            </div>;
          })}</div>
        </article>
      </div> : <p role="status">Loading model provider state…</p>}
      {modelState?.currentRoute ? <p className="model-route-current" role="status">Current verified route: <strong>{routeLabel(modelState.currentRoute)}</strong> via {modelState.currentRoute.provider === 'CHATGPT' ? 'ChatGPT subscription' : 'DeepSeek official API'}</p> : <p>No verified model route. Agent turns and onboarding Ready remain unavailable.</p>}
      {modelState && modelState.attempts.length > 0 && <details className="model-attempts"><summary>Setup attempts ({modelState.attempts.length})</summary><ul>{[...modelState.attempts].reverse().slice(0, 5).map(attempt => <li key={attempt.attemptId}><strong>{attempt.provider}</strong>{attempt.modelId ? ` · ${attempt.modelId}` : ''}{attempt.thinkingType ? ` · ${attempt.thinkingType}` : ''} · {attempt.outcome}{attempt.errorCategory ? ` · ${attempt.errorCategory}` : ''}<small>{attempt.endedAt}</small></li>)}</ul></details>}
      {modelBusy && <p role="status">Updating model provider…</p>}
    </> : <p role="status">Loading model gateway…</p>}
    <button onClick={() => { void gateway.reload(); void model.reload(); }}>Reload model state</button>
  </section>;
}
