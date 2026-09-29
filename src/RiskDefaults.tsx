import { useEffect, useState } from 'react';
import type { RiskAssetClassLimit, RiskPolicy, RiskPolicyEnvironment, RiskPolicyInput, RiskPolicyState } from '../shared/ipc-types.ts';
import { CommandError, explainError, request } from './client.ts';

export type RiskDraft = {
  maxOrderNotional: string;
  maxOrderQuantity: string;
  maxPositionSize: string;
  maxSingleInstrumentExposurePercent: string;
  maxEquityExposurePercent: string;
  maxCryptoExposurePercent: string;
  maxDailyTradedNotional: string;
  maxDailyRealizedLoss: string;
  maxOpenOrders: string;
  maxReservedCapital: string;
  allowedInstrumentIds: string;
  blockedInstrumentIds: string;
  allowedVenues: string;
  blockedVenues: string;
  allowedAccountIds: string;
  blockedAccountIds: string;
  allowedEnvironments: RiskPolicyEnvironment[];
  staleQuoteThresholdSeconds: string;
  marketOrdersEnabled: boolean;
  maxMarketOrderSlippagePercent: string;
  maxPriceDeviationPercent: string;
  liveInactivityTimeoutMinutes: string;
};

const environments: { value: RiskPolicyEnvironment; label: string }[] = [
  { value: 'LOCAL_PAPER', label: 'Local Paper' },
  { value: 'PAPER', label: 'Paper' },
  { value: 'DEMO', label: 'Demo' },
  { value: 'TESTNET', label: 'Testnet' },
  { value: 'LIVE', label: 'Live' },
];

const listFromText = (value: string) => value.split('\n').map(item => item.trim()).filter(Boolean);
const textFromList = (value: string[]) => value.join('\n');
const optionalDecimal = (value: string) => value.trim() || null;

export function draftFromPolicy(policy: RiskPolicy): RiskDraft {
  const assetLimit = (assetClass: RiskAssetClassLimit['assetClass']) => policy.maxAssetClassExposurePercent.find(item => item.assetClass === assetClass)?.maxExposurePercent ?? '';
  return {
    maxOrderNotional: policy.maxOrderNotional ?? '',
    maxOrderQuantity: policy.maxOrderQuantity ?? '',
    maxPositionSize: policy.maxPositionSize ?? '',
    maxSingleInstrumentExposurePercent: policy.maxSingleInstrumentExposurePercent ?? '',
    maxEquityExposurePercent: assetLimit('EQUITY'),
    maxCryptoExposurePercent: assetLimit('CRYPTO_SPOT'),
    maxDailyTradedNotional: policy.maxDailyTradedNotional ?? '',
    maxDailyRealizedLoss: policy.maxDailyRealizedLoss ?? '',
    maxOpenOrders: policy.maxOpenOrders == null ? '' : String(policy.maxOpenOrders),
    maxReservedCapital: policy.maxReservedCapital ?? '',
    allowedInstrumentIds: textFromList(policy.allowedInstrumentIds),
    blockedInstrumentIds: textFromList(policy.blockedInstrumentIds),
    allowedVenues: textFromList(policy.allowedVenues),
    blockedVenues: textFromList(policy.blockedVenues),
    allowedAccountIds: textFromList(policy.allowedAccountIds),
    blockedAccountIds: textFromList(policy.blockedAccountIds),
    allowedEnvironments: [...policy.allowedEnvironments],
    staleQuoteThresholdSeconds: String(policy.staleQuoteThresholdSeconds ?? 3),
    marketOrdersEnabled: policy.marketOrdersEnabled ?? false,
    maxMarketOrderSlippagePercent: policy.maxMarketOrderSlippagePercent ?? '',
    maxPriceDeviationPercent: policy.maxPriceDeviationPercent ?? '',
    liveInactivityTimeoutMinutes: String(policy.liveInactivityTimeoutMinutes ?? 20),
  };
}

function toPolicy(draft: RiskDraft): RiskPolicyInput {
  const stale = Number(draft.staleQuoteThresholdSeconds);
  const inactivity = Number(draft.liveInactivityTimeoutMinutes);
  const maxOpenOrders = draft.maxOpenOrders.trim() ? Number(draft.maxOpenOrders) : null;
  if (!Number.isInteger(stale) || !Number.isInteger(inactivity)) throw new Error('Enter whole seconds and minutes.');
  if (maxOpenOrders !== null && (!Number.isSafeInteger(maxOpenOrders) || maxOpenOrders < 1)) throw new Error('Maximum open orders must be a positive whole number.');
  if (draft.marketOrdersEnabled && !draft.maxMarketOrderSlippagePercent.trim()) throw new Error('Set a maximum market-order slippage before enabling market orders.');
  const maxAssetClassExposurePercent: RiskAssetClassLimit[] = [
    ...(draft.maxEquityExposurePercent.trim() ? [{ assetClass: 'EQUITY' as const, maxExposurePercent: draft.maxEquityExposurePercent.trim() }] : []),
    ...(draft.maxCryptoExposurePercent.trim() ? [{ assetClass: 'CRYPTO_SPOT' as const, maxExposurePercent: draft.maxCryptoExposurePercent.trim() }] : []),
  ];
  return {
    maxOrderNotional: optionalDecimal(draft.maxOrderNotional),
    maxOrderQuantity: optionalDecimal(draft.maxOrderQuantity),
    maxPositionSize: optionalDecimal(draft.maxPositionSize),
    maxSingleInstrumentExposurePercent: optionalDecimal(draft.maxSingleInstrumentExposurePercent),
    maxAssetClassExposurePercent: maxAssetClassExposurePercent as RiskPolicyInput['maxAssetClassExposurePercent'],
    maxDailyTradedNotional: optionalDecimal(draft.maxDailyTradedNotional),
    maxDailyRealizedLoss: optionalDecimal(draft.maxDailyRealizedLoss),
    maxOpenOrders,
    maxReservedCapital: optionalDecimal(draft.maxReservedCapital),
    allowedInstrumentIds: listFromText(draft.allowedInstrumentIds) as RiskPolicyInput['allowedInstrumentIds'],
    blockedInstrumentIds: listFromText(draft.blockedInstrumentIds) as RiskPolicyInput['blockedInstrumentIds'],
    allowedVenues: listFromText(draft.allowedVenues) as RiskPolicyInput['allowedVenues'],
    blockedVenues: listFromText(draft.blockedVenues) as RiskPolicyInput['blockedVenues'],
    allowedAccountIds: listFromText(draft.allowedAccountIds) as RiskPolicyInput['allowedAccountIds'],
    blockedAccountIds: listFromText(draft.blockedAccountIds) as RiskPolicyInput['blockedAccountIds'],
    allowedEnvironments: draft.allowedEnvironments as RiskPolicyInput['allowedEnvironments'],
    staleQuoteThresholdSeconds: stale,
    marketOrdersEnabled: draft.marketOrdersEnabled,
    maxMarketOrderSlippagePercent: optionalDecimal(draft.maxMarketOrderSlippagePercent),
    maxPriceDeviationPercent: optionalDecimal(draft.maxPriceDeviationPercent),
    liveInactivityTimeoutMinutes: inactivity,
  };
}

function DecimalField({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return <label className="field">{label}<input type="text" inputMode="decimal" value={value} onChange={event => onChange(event.target.value)} placeholder="Leave unset" maxLength={32} /></label>;
}

function ListField({ label, hint, value, onChange }: { label: string; hint: string; value: string; onChange: (value: string) => void }) {
  return <label className="field">{label}<textarea rows={3} maxLength={33023} value={value} onChange={event => onChange(event.target.value)} placeholder="One identifier per line" /><small className="form-hint">{hint}</small></label>;
}

type Props = {
  workspaceId: string;
  baseCurrency: string;
  state: RiskPolicyState;
  draft: RiskDraft;
  onDraftChange: (draft: RiskDraft) => void;
  onSaved?: (state: RiskPolicyState) => void;
  continueLabel?: string;
  showFullPolicy?: boolean;
};

export function RiskDefaults({ workspaceId, baseCurrency, state, draft, onDraftChange, onSaved, continueLabel = 'Save risk defaults', showFullPolicy = false }: Props) {
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
  const toggleEnvironment = (environment: RiskPolicyEnvironment, checked: boolean) => change(
    'allowedEnvironments',
    checked ? [...draft.allowedEnvironments, environment] : draft.allowedEnvironments.filter(value => value !== environment),
  );
  return <div className="risk-defaults">
    <p className="muted">Money and portfolio exposure use exact decimals in {baseCurrency}. Leave limits unset until you choose them. Quantity uses canonical instrument base units.</p>
    <fieldset className="risk-form" disabled={busy}>
      <legend className="sr-only">Risk policy configuration</legend>
      <DecimalField label={`Maximum order notional (${baseCurrency})`} value={draft.maxOrderNotional} onChange={value => change('maxOrderNotional', value)} />
      {showFullPolicy && <DecimalField label="Maximum order quantity (shares or base units)" value={draft.maxOrderQuantity} onChange={value => change('maxOrderQuantity', value)} />}
      {showFullPolicy && <DecimalField label={`Maximum position size (${baseCurrency})`} value={draft.maxPositionSize} onChange={value => change('maxPositionSize', value)} />}
      <DecimalField label="Maximum single-instrument exposure (%)" value={draft.maxSingleInstrumentExposurePercent} onChange={value => change('maxSingleInstrumentExposurePercent', value)} />
      {showFullPolicy && <>
        <DecimalField label="Maximum EQUITY exposure (%)" value={draft.maxEquityExposurePercent} onChange={value => change('maxEquityExposurePercent', value)} />
        <DecimalField label="Maximum CRYPTO_SPOT exposure (%)" value={draft.maxCryptoExposurePercent} onChange={value => change('maxCryptoExposurePercent', value)} />
      </>}
      <DecimalField label={`Maximum daily traded notional (${baseCurrency})`} value={draft.maxDailyTradedNotional} onChange={value => change('maxDailyTradedNotional', value)} />
      <DecimalField label={`Maximum daily realized loss (${baseCurrency})`} value={draft.maxDailyRealizedLoss} onChange={value => change('maxDailyRealizedLoss', value)} />
      {showFullPolicy && <label className="field">Maximum open orders<input type="number" inputMode="numeric" min={1} step={1} value={draft.maxOpenOrders} onChange={event => change('maxOpenOrders', event.target.value)} placeholder="Leave unset" /></label>}
      {showFullPolicy && <DecimalField label={`Maximum reserved capital (${baseCurrency})`} value={draft.maxReservedCapital} onChange={value => change('maxReservedCapital', value)} />}
      <label className="field">Stale quote threshold (seconds)<input type="number" inputMode="numeric" min={1} max={86400} step={1} value={draft.staleQuoteThresholdSeconds} onChange={event => change('staleQuoteThresholdSeconds', event.target.value)} required /></label>
      <label className="check-field"><input type="checkbox" checked={draft.marketOrdersEnabled} onChange={event => change('marketOrdersEnabled', event.target.checked)} /> Allow market orders <span className="muted">OFF by default</span></label>
      {(showFullPolicy || draft.marketOrdersEnabled) && <DecimalField label="Maximum market-order slippage (%)" value={draft.maxMarketOrderSlippagePercent} onChange={value => change('maxMarketOrderSlippagePercent', value)} />}
      {showFullPolicy && <DecimalField label="Maximum price deviation (%)" value={draft.maxPriceDeviationPercent} onChange={value => change('maxPriceDeviationPercent', value)} />}
      <label className="field">Live inactivity timeout (minutes)<input type="number" inputMode="numeric" min={1} max={1440} step={1} value={draft.liveInactivityTimeoutMinutes} onChange={event => change('liveInactivityTimeoutMinutes', event.target.value)} required /></label>
      {showFullPolicy && <>
        <ListField label="Allowed canonical instruments" hint="Empty means no allow-list restriction. Blocked instruments always reject." value={draft.allowedInstrumentIds} onChange={value => change('allowedInstrumentIds', value)} />
        <ListField label="Blocked canonical instruments" hint="Use canonical IDs such as equity:US:AAPL or crypto:BTC/USDT:spot." value={draft.blockedInstrumentIds} onChange={value => change('blockedInstrumentIds', value)} />
        <ListField label="Allowed venues" hint="Empty means no allow-list restriction; blocked venues take precedence." value={draft.allowedVenues} onChange={value => change('allowedVenues', value)} />
        <ListField label="Blocked venues" hint="Enter canonical venue IDs, one per line." value={draft.blockedVenues} onChange={value => change('blockedVenues', value)} />
        <ListField label="Allowed account IDs" hint="Copy connection IDs from Accounts. Empty means no allow-list restriction." value={draft.allowedAccountIds} onChange={value => change('allowedAccountIds', value)} />
        <ListField label="Blocked account IDs" hint="Matching account IDs are rejected even if listed as allowed." value={draft.blockedAccountIds} onChange={value => change('blockedAccountIds', value)} />
        <fieldset className="risk-environments">
          <legend>Allowed account environments</legend>
          <p className="form-hint">No selection means no additional environment restriction.</p>
          <div>{environments.map(item => <label className="check-field" key={item.value}><input type="checkbox" checked={draft.allowedEnvironments.includes(item.value)} onChange={event => toggleEnvironment(item.value, event.target.checked)} /> {item.label}</label>)}</div>
        </fieldset>
      </>}
    </fieldset>
    <div className="hard-rules" aria-labelledby="hard-rules-title"><h3 id="hard-rules-title">Hard safety rules · read only</h3><ul>{state.hardRules.map(rule => <li key={rule.id}><strong>{rule.id.replaceAll('_', ' ')}</strong><span>{rule.description}</span></li>)}</ul></div>
    {state.lastChange && <section className="notice" role="status" aria-live="polite" aria-atomic="true">
      <h3>Policy change · v{state.lastChange.oldPolicyVersion} → v{state.lastChange.newPolicyVersion}</h3>
      <p>{state.lastChange.scope.kind} scope · {state.lastChange.affectedAccounts.length} affected accounts · {state.lastChange.affectedProposals.length} pending proposals invalidated.</p>
      <p>{state.lastChange.weakened ? `Policy relaxation: ${state.lastChange.weakeningReasons.map(reason => reason.replaceAll('_', ' ').toLowerCase()).join(', ')}.` : 'No policy relaxation detected.'}</p>
      <details><summary>Affected accounts</summary><ul>{state.lastChange.affectedAccounts.map(account => <li key={account.accountId}>{account.environment} · {account.accountId}</li>)}</ul></details>
      <details><summary>Invalidated proposals</summary><ul>{state.lastChange.affectedProposals.map(proposal => <li key={proposal.proposalId}>{proposal.proposalId} · {proposal.invalidationReason}</li>)}</ul></details>
    </section>}
    {error && <p className="error-text" role="alert">{error}</p>}
    {notice && <p className="success-text" role="status">{notice}</p>}
    <button className="primary" type="button" onClick={() => void save()} disabled={busy}>{busy ? 'Saving risk policy…' : continueLabel}</button>
  </div>;
}
