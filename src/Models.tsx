import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import type { ChatgptLoginAction, GatewayAction, ModelProviderState, ModelRoute, SetDefaultModel, SetFallbackPolicy, ThinkingType } from '../shared/ipc-types.ts';
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
  MODEL_GATEWAY_RUNNING: 'Stop the model gateway before replacing the DeepSeek key, then launch it again.',
  MODEL_DEFAULT_MISSING: 'Choose a verified default model route before starting a model request.',
  MODEL_FALLBACK_UNAVAILABLE: 'Automatic fallback needs a verified DeepSeek route in this workspace.',
  MODEL_QUOTA_COOLDOWN: 'The selected model is in its known provider cooldown. Retry after the reset window.',
  MODEL_KEYCHAIN_MISSING: 'The DeepSeek OS Keychain key is unavailable. Configure it again.',
  MODEL_UNAVAILABLE: 'The selected model route is unavailable. Retry its probe or choose another route.',
  MODEL_OAUTH_EXPIRED: 'ChatGPT authorization expired or was rejected. Re-login before verifying the route.',
  MODEL_LOGIN_FAILED: 'ChatGPT authorization did not complete. Retry login; no token was imported into TradeX.',
  MODEL_LOGIN_TIMEOUT: 'ChatGPT authorization timed out. Retry login in the browser.',
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

type ModelCommand = 'model.login_chatgpt' | 'model.configure_deepseek' | 'model.verify_route' | 'model.set_default' | 'model.set_fallback_policy';
type ModelPayload = { action: ChatgptLoginAction } | { provider: 'CHATGPT' | 'DEEPSEEK'; modelId: string; thinkingType: ThinkingType | null } | SetDefaultModel | SetFallbackPolicy | null;

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
  async function modelAct(command: ModelCommand, payload: ModelPayload) {
    if (!modelState || modelBusy) return;
    setModelBusy(true); setError(null);
    try {
      if (command === 'model.login_chatgpt') await request(command, { workspaceId, expectedStateVersion: modelState.stateVersion, action: (payload as { action: ChatgptLoginAction }).action });
      else if (command === 'model.configure_deepseek') await request(command, { workspaceId, expectedStateVersion: modelState.stateVersion });
      else if (command === 'model.verify_route') await request(command, { workspaceId, expectedStateVersion: modelState.stateVersion, ...(payload as { provider: 'CHATGPT' | 'DEEPSEEK'; modelId: string; thinkingType: ThinkingType | null }) });
      else if (command === 'model.set_default') await request(command, { ...(payload as SetDefaultModel), workspaceId, expectedStateVersion: modelState.stateVersion });
      else await request(command, { ...(payload as SetFallbackPolicy), workspaceId, expectedStateVersion: modelState.stateVersion });
    } catch (error) { setError(error); }
    finally { setModelBusy(false); await model.reload(); await gateway.reload(); }
  }
  const gatewayRunning = state?.status === 'RUNNING';
  const verify = (provider: 'CHATGPT' | 'DEEPSEEK', modelId: string, thinkingType: ThinkingType | null) => { void modelAct('model.verify_route', { provider, modelId, thinkingType }); };
  const defaultRoute = modelState?.defaultRoute ?? null;
  const fallbackTarget = modelState?.deepseek.status === 'READY' ? modelState.deepseek.routes.find(route => route.provider === 'DEEPSEEK' && route.verifiedAt && route.modelId === 'deepseek-v4-flash' && route.thinkingType != null) ?? null : null;
  const cooldownFor = (provider: 'CHATGPT' | 'DEEPSEEK') => {
    const attempt = modelState ? [...modelState.attempts].reverse().find(item => item.provider === provider && item.errorCategory === 'QUOTA_EXCEEDED' && item.quota?.retryAfterSeconds != null) : undefined;
    const seconds = attempt?.quota?.retryAfterSeconds;
    return seconds != null && Number.isFinite(Date.parse(attempt?.endedAt ?? '')) && Date.parse(attempt?.endedAt ?? '') + seconds * 1000 > Date.now();
  };
  const chatgptCooldownActive = cooldownFor('CHATGPT');
  const deepseekCooldownActive = cooldownFor('DEEPSEEK');
  const setDefault = (route: ModelRoute) => { void modelAct('model.set_default', { provider: route.provider, modelId: route.modelId, thinkingType: route.thinkingType ?? null } as SetDefaultModel); };
  const switchToDeepSeek = () => { if (fallbackTarget) setDefault(fallbackTarget); };
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
          <div className="model-provider-actions">
            <button disabled={modelBusy || !gatewayRunning} onClick={() => { void modelAct('model.login_chatgpt', { action: modelState.chatgpt.configured ? 'RELOGIN' : 'LOGIN' }); }}>{modelState.chatgpt.configured ? 'Re-login ChatGPT' : 'Login ChatGPT'}</button>
            {fallbackTarget && modelState.chatgpt.errorCode && ['MODEL_UNAVAILABLE', 'MODEL_OAUTH_EXPIRED', 'MODEL_QUOTA_EXCEEDED'].includes(modelState.chatgpt.errorCode) && <button disabled={modelBusy} onClick={switchToDeepSeek}>Switch to DeepSeek</button>}
          </div>
          {!gatewayRunning && <p className="form-hint">Launch the pinned gateway before starting OAuth or verifying a route.</p>}
          <div className="model-routes">
            {modelState.chatgpt.routes.length === 0 ? <p className="form-hint">No allowed GPT-5.6 models discovered yet.</p> : modelState.chatgpt.routes.map(route => <div className="model-route" key={route.modelId}>
              <div><strong>{routeLabel(route)}</strong><small>{route.verifiedAt ? `Verified ${route.verifiedAt}` : 'Discovered · verification required'}</small></div>
              <div className="model-route-actions"><button disabled={modelBusy || modelState.chatgpt.status !== 'READY' || !route.verifiedAt || (defaultRoute?.provider === route.provider && defaultRoute.modelId === route.modelId && defaultRoute.thinkingType === route.thinkingType)} onClick={() => setDefault(route)}>Use as default</button><button disabled={modelBusy || !gatewayRunning || chatgptCooldownActive} onClick={() => verify('CHATGPT', route.modelId, null)}>{chatgptCooldownActive ? 'Cooldown' : 'Verify route'}</button></div>
            </div>)}
          </div>
        </article>
        <article className="model-provider-card">
          <div className="model-provider-heading"><div><h4>CLIProxyAPI → DeepSeek</h4><p>Official API · key stored only in macOS Keychain</p></div><span className="badge">{modelHealth[modelState.deepseek.status]}</span></div>
          {providerError(modelState.deepseek) && <p className="model-error" role="alert">{providerError(modelState.deepseek)}</p>}
          <div className="model-provider-actions"><button disabled={modelBusy || state.desiredRunning || state.status === 'STOPPING'} onClick={() => { void modelAct('model.configure_deepseek', null); }}>{modelState.deepseek.configured ? 'Replace DeepSeek key' : 'Configure DeepSeek key'}</button></div>
          <p className="form-hint">The native secure dialog writes directly to Keychain. The key never enters the webview, SQLite, events or logs. Stop the gateway before changing the key, then launch it again to render the new key into its private config.</p>
          <div className="model-routes">{(['disabled', 'enabled'] as const).map(mode => {
            const route = modelState.deepseek.routes.find(item => item.modelId === 'deepseek-v4-flash' && item.thinkingType === mode);
            return <div className="model-route" key={mode}>
              <div><strong>deepseek-v4-flash · {mode === 'enabled' ? 'Thinking' : 'Non-thinking'}</strong><small>{route?.verifiedAt ? `Verified ${route.verifiedAt}` : 'Explicit mode · verification required'}</small></div>
              <div className="model-route-actions"><button disabled={modelBusy || modelState.deepseek.status !== 'READY' || !route?.verifiedAt || (defaultRoute?.provider === 'DEEPSEEK' && defaultRoute.modelId === 'deepseek-v4-flash' && defaultRoute.thinkingType === mode)} onClick={() => { if (route) setDefault(route); }}>Use as default</button><button disabled={modelBusy || !gatewayRunning || !modelState.deepseek.configured || deepseekCooldownActive} onClick={() => verify('DEEPSEEK', 'deepseek-v4-flash', mode)}>{deepseekCooldownActive ? 'Cooldown' : 'Verify route'}</button></div>
            </div>;
          })}</div>
          <div className="model-policy">
            <label><input type="checkbox" checked={Boolean(modelState.automaticFallback)} disabled={modelBusy || !fallbackTarget} onChange={event => { void modelAct('model.set_fallback_policy', { automaticFallback: event.target.checked } as SetFallbackPolicy); }} /> Allow automatic fallback to DeepSeek</label>
            <p className="form-hint">OFF by default. When enabled, only an eligible failed ChatGPT attempt may retry through this verified DeepSeek route; the provider change remains visible and audited.</p>
            {!fallbackTarget && <p className="form-hint">Verify a DeepSeek route before enabling automatic fallback.</p>}
            {modelState.automaticFallback && <small>Consent version {modelState.fallbackPolicyVersion ?? 1}</small>}
          </div>
        </article>
      </div> : <p role="status">Loading model provider state…</p>}
      {defaultRoute ? <p className="model-route-current" role="status">Default for next request: <strong>{routeLabel(defaultRoute)}</strong> via {defaultRoute.provider === 'CHATGPT' ? 'ChatGPT subscription' : 'DeepSeek official API'}</p> : <p>No default model route selected. Choose a verified route before starting an agent turn.</p>}
      {(chatgptCooldownActive || deepseekCooldownActive) && <p className="model-error" role="status">Retry is blocked until the known provider cooldown expires.</p>}
      {modelState && modelState.attempts.length > 0 && <details className="model-attempts"><summary>Provider attempts ({modelState.attempts.length})</summary><ul>{[...modelState.attempts].reverse().slice(0, 5).map(attempt => <li key={attempt.attemptId}><strong>{attempt.kind ?? 'SETUP'} · {attempt.provider}</strong>{attempt.modelId ? ` · ${attempt.modelId}` : ''}{attempt.thinkingType ? ` · ${attempt.thinkingType}` : ''} · {attempt.outcome}{attempt.errorCategory ? ` · ${attempt.errorCategory}` : ''}<small>{attempt.endedAt}</small>{attempt.quota && <small>{[attempt.quota.remaining != null ? `Quota remaining ${attempt.quota.remaining}` : '', attempt.quota.window ?? '', attempt.quota.retryAfterSeconds != null ? `Retry after ${attempt.quota.retryAfterSeconds}s` : '', attempt.quota.resetAt ? `Reset ${attempt.quota.resetAt}` : ''].filter(Boolean).join(' · ')}</small>}</li>)}</ul></details>}
      {modelBusy && <p role="status">Updating model provider…</p>}
    </> : <p role="status">Loading model gateway…</p>}
    <button onClick={() => { void gateway.reload(); void model.reload(); }}>Reload model state</button>
  </section>;
}
