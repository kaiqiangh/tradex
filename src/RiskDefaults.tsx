import { useEffect, useState } from 'react';
import type { RiskPolicy, RiskPolicyInput, RiskPolicyState } from '../shared/ipc-types.ts';
import { CommandError, explainError, request } from './client.ts';

export type RiskDraft = {
  maxOrderNotional: string;
  maxSingleInstrumentExposurePercent: string;
  maxDailyTradedNotional: string;
  maxDailyRealizedLoss: string;
  staleQuoteThresholdSeconds: string;
  marketOrdersEnabled: boolean;
  liveInactivityTimeoutMinutes: string;
};

export function draftFromPolicy(policy: RiskPolicy): RiskDraft {
  return {
    maxOrderNotional: policy.maxOrderNotional ?? '',
    maxSingleInstrumentExposurePercent: policy.maxSingleInstrumentExposurePercent ?? '',
    maxDailyTradedNotional: policy.maxDailyTradedNotional ?? '',
    maxDailyRealizedLoss: policy.maxDailyRealizedLoss ?? '',
    staleQuoteThresholdSeconds: String(policy.staleQuoteThresholdSeconds ?? 3),
    marketOrdersEnabled: policy.marketOrdersEnabled ?? false,
    liveInactivityTimeoutMinutes: String(policy.liveInactivityTimeoutMinutes ?? 20),
  };
}

function toPolicy(draft: RiskDraft): RiskPolicyInput {
  const stale = Number(draft.staleQuoteThresholdSeconds);
  const inactivity = Number(draft.liveInactivityTimeoutMinutes);
  if (!Number.isInteger(stale) || !Number.isInteger(inactivity)) throw new Error('Enter whole seconds and minutes.');
  return {
    maxOrderNotional: draft.maxOrderNotional.trim() || null,
    maxSingleInstrumentExposurePercent: draft.maxSingleInstrumentExposurePercent.trim() || null,
    maxDailyTradedNotional: draft.maxDailyTradedNotional.trim() || null,
    maxDailyRealizedLoss: draft.maxDailyRealizedLoss.trim() || null,
    staleQuoteThresholdSeconds: stale,
    marketOrdersEnabled: draft.marketOrdersEnabled,
    liveInactivityTimeoutMinutes: inactivity,
  };
}

type Props = {
  workspaceId: string;
  baseCurrency: string;
  state: RiskPolicyState;
  draft: RiskDraft;
  onDraftChange: (draft: RiskDraft) => void;
  onSaved?: (state: RiskPolicyState) => void;
  continueLabel?: string;
};

export function RiskDefaults({ workspaceId, baseCurrency, state, draft, onDraftChange, onSaved, continueLabel = 'Save risk defaults' }: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState('');
  useEffect(() => { setError(undefined); }, [state.stateVersion]);
  const change = <K extends keyof RiskDraft>(key: K, value: RiskDraft[K]) => onDraftChange({ ...draft, [key]: value });
  const save = async () => {
    setBusy(true); setError(undefined); setNotice('');
    try {
      const saved = await request('risk.save_policy', { workspaceId, expectedStateVersion: state.stateVersion, policy: toPolicy(draft) });
      setNotice(`Saved policy v${state.policyVersion + 1}.`);
      onSaved?.(saved);
    } catch (failure) {
      setError(failure instanceof Error && !(failure instanceof CommandError) ? failure.message : explainError(failure));
    } finally { setBusy(false); }
  };
  return <div className="risk-defaults">
    <p className="muted">Money uses exact decimal values in {baseCurrency}. Leave monetary limits blank until you choose them.</p>
    <div className="risk-form">
      <label className="field">Maximum order notional ({baseCurrency})
        <input type="text" inputMode="decimal" value={draft.maxOrderNotional} onChange={event => change('maxOrderNotional', event.target.value)} placeholder="Leave blank" maxLength={32} aria-label={`Maximum order notional in ${baseCurrency}`} disabled={busy} />
      </label>
      <label className="field">Maximum single-instrument exposure (%)
        <input type="text" inputMode="decimal" value={draft.maxSingleInstrumentExposurePercent} onChange={event => change('maxSingleInstrumentExposurePercent', event.target.value)} placeholder="Leave blank" maxLength={32} aria-label="Maximum single-instrument exposure percent" disabled={busy} />
      </label>
      <label className="field">Maximum daily traded notional ({baseCurrency})
        <input type="text" inputMode="decimal" value={draft.maxDailyTradedNotional} onChange={event => change('maxDailyTradedNotional', event.target.value)} placeholder="Leave blank" maxLength={32} aria-label={`Maximum daily traded notional in ${baseCurrency}`} disabled={busy} />
      </label>
      <label className="field">Maximum daily realized loss ({baseCurrency})
        <input type="text" inputMode="decimal" value={draft.maxDailyRealizedLoss} onChange={event => change('maxDailyRealizedLoss', event.target.value)} placeholder="Leave blank" maxLength={32} aria-label={`Maximum daily realized loss in ${baseCurrency}`} disabled={busy} />
      </label>
      <label className="field">Stale quote threshold (seconds)
        <input type="number" inputMode="numeric" min={1} max={86400} step={1} value={draft.staleQuoteThresholdSeconds} onChange={event => change('staleQuoteThresholdSeconds', event.target.value)} required aria-label="Stale quote threshold in seconds" disabled={busy} />
      </label>
      <label className="check-field"><input type="checkbox" checked={draft.marketOrdersEnabled} onChange={event => change('marketOrdersEnabled', event.target.checked)} disabled={busy} /> Allow market orders <span className="muted">OFF by default</span></label>
      <label className="field">Live inactivity timeout (minutes)
        <input type="number" inputMode="numeric" min={1} max={1440} step={1} value={draft.liveInactivityTimeoutMinutes} onChange={event => change('liveInactivityTimeoutMinutes', event.target.value)} required aria-label="Live inactivity timeout in minutes" disabled={busy} />
      </label>
    </div>
    <div className="hard-rules" aria-labelledby="hard-rules-title"><h3 id="hard-rules-title">Hard safety rules · read only</h3><ul>{state.hardRules.map(rule => <li key={rule.id}><strong>{rule.id.replaceAll('_', ' ')}</strong><span>{rule.description}</span></li>)}</ul></div>
    {error && <p className="error-text" role="alert">{error}</p>}
    {notice && <p className="success-text" role="status">{notice}</p>}
    <button className="primary" type="button" onClick={() => void save()} disabled={busy}>{busy ? 'Saving risk defaults…' : continueLabel}</button>
  </div>;
}
