import { useState } from 'react';
import type { ReactNode } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import type { SpotProposalRules, SpotPriceRangePreview } from '../shared/ipc-types.ts';
const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');

const priceRangeMeaning = (preview: SpotPriceRangePreview) => {
  switch (preview.state) {
    case 'NO_RULE': return 'No PRICE_RANGE execution rule is configured for this symbol, so the documented contract enforces no execution price range here.';
    case 'UNSUPPORTED_CONFIGURATION': return 'This execution rule uses fields this build does not recognise, so the range stays unresolved instead of being assumed.';
    case 'NOT_ENFORCED_SELECTED_SIDE': return 'At least one multiplier for this order side is not set, so the documented contract enforces no range for that direction. A reported zero is a real multiplier, not a disabled marker.';
    case 'NO_STATED_EXECUTION_PRICE': return 'A market form states no execution price, so no placement-time comparison exists; the venue applies the rule during the taker phase.';
    case 'REFERENCE_MISSING': return 'This order side is enforced, but no current genuine provider reference price has been read for this unchanged Proposal yet.';
    case 'REFERENCE_UNAVAILABLE': return 'The genuine execution reference price is unavailable, stale or cannot be multiplied exactly. A failed read is never treated as non-enforcement.';
    case 'REFERENCE_EXPLICIT_NULL': return 'The provider reported an explicit null reference price, so the documented contract enforces no range. This is a verified fact, not a failed lookup.';
    case 'SNAPSHOT_AVAILABLE': return 'A genuine current provider reference price bounded this snapshot at the times below.';
    default: return 'The execution rule state is not explained by this build.';
  }
};

function PriceRangeExplanation({ preview, captured }: { preview: SpotPriceRangePreview; captured: boolean }) {
  const bounded = preview.lowerBound != null && preview.upperBound != null;
  return <section className="price-range-preview" aria-label={captured ? 'Captured price range execution preview' : 'Current price range execution preview'}>
    <h4>Price range execution preview · {human(preview.state)}</h4>
    <p>{priceRangeMeaning(preview)}</p>
    <dl className="data-source-details">
      <div><dt>Selected side / unit</dt><dd>{preview.side} · {preview.unit}. Bounds are quoted in the QUOTE asset per BASE unit.</dd></div>
      {preview.directions.map(direction => <div key={direction.direction}><dt>{direction.direction === 'BID' ? 'BUY (bid)' : 'SELL (ask)'} multipliers</dt>
        <dd>{direction.lowerMultiplier ?? 'lower multiplier not set'} · {direction.upperMultiplier ?? 'upper multiplier not set'} · {direction.enforced ? 'Enforced for this direction' : 'Not enforced for this direction'}</dd></div>)}
      <div><dt>Selected-side snapshot bound</dt>
        <dd>{bounded ? `${preview.lowerBound} – ${preview.upperBound} ${preview.unit}` : preview.state === 'SNAPSHOT_AVAILABLE' ? `Snapshot bounds retained as digests only · ${preview.boundsDigest ?? 'digest unavailable'}` : 'No snapshot bound is established'}</dd></div>
    </dl>
    {preview.reference && <dl className="data-source-details">
      <div><dt>Execution reference</dt><dd>{preview.reference.price != null
        ? `${preview.reference.price} ${preview.unit}`
        : preview.state === 'REFERENCE_EXPLICIT_NULL'
          ? 'Provider reported an explicit null reference price'
          : captured
            ? 'Price retained as digest only'
            : 'Price withheld: this reference is not current'} · {preview.reference.digest}</dd></div>
      <div><dt>Provider time</dt><dd>{preview.reference.providerObservedAt}</dd></div>
      <div><dt>First receipt</dt><dd>{preview.reference.receivedAt}</dd></div>
    </dl>}
    <p>Configuration: <strong>{human(preview.explanation)}</strong>. This snapshot only explains today&apos;s execution-rule inputs. The venue recalculates the reference price when the order enters its taker phase, an execution outside the range expires the order, and nothing here approves, places or promises a fill.</p>
  </section>;
}

export function SpotRulesExplanation({ rules, captured = false, actions }: { rules: SpotProposalRules; captured?: boolean; actions?: ReactNode }) {
  return <section className="card spot-proposal-rules" aria-label={captured ? 'Captured Proposal Spot rules' : 'Current Proposal Spot rules'}>
    <h3>{captured ? 'Captured Proposal Spot rules' : 'Current Proposal Spot rules'}</h3>
    <p><strong>Complete rules · {rules.outcome}</strong></p>
    <p>{captured ? 'Saved assessment. These outcomes and original receipts do not renew current evidence or consent.' : 'Current assessment of the saved, unchanged Proposal. Missing inputs remain unavailable.'} Rights, quotes, permissions, fees, FX and immediate preflight need their own checks.</p>
    <dl className="data-source-details">
      <div><dt>Proposal</dt><dd>{rules.proposalId} · {rules.proposalHash}</dd></div>
      <div><dt>Exact account</dt><dd>{rules.accountId}</dd></div>
      <div><dt>Instrument / units</dt><dd>{rules.instrumentId} · BASE {rules.baseAsset} / QUOTE {rules.quoteAsset}. USDT is not USD.</dd></div>
      <div><dt>Rule source / material</dt><dd>{rules.sourceVersion} · {rules.materialVersion ?? 'Current exact rules unavailable'}</dd></div>
      <div><dt>Rule first receipt</dt><dd>{rules.observedAt ?? 'Not checked'}</dd></div>
    </dl>
    <ul className="risk-decision-checks">{rules.rules.map((rule, index) => <li key={index}>
      <strong>{rule.ruleType} · {rule.outcome}</strong>
      <span>{rule.origin} · {rule.scope} · {rule.applicable ? 'Applies to this intent' : 'Not applicable to this intent'} · {human(rule.reasonCode)}{rule.unit && ` · Unit: ${rule.unit}`}</span>
    </li>)}</ul>
    {rules.priceRangePreview && <PriceRangeExplanation preview={rules.priceRangePreview} captured={captured} />}
    <h4>Required reference purposes</h4><ul>{rules.referencePurposes.map(purpose => <li key={purpose}>{purpose}</li>)}</ul>
    {!rules.referencePurposes.length && <p>{rules.materialVersion ? 'This intent needs no public rule reference read.' : 'Required reference purposes are unavailable until current exact rules are refreshed.'}</p>}
    {rules.references.map(reference => <dl className="data-source-details" key={reference.digest}>
      <div><dt>Reference kind</dt><dd>{reference.kind} · {reference.intervalMinutes == null ? 'Provider reference; no average interval' : reference.intervalMinutes === 0 ? 'Original last trade; zero interval' : `${reference.intervalMinutes} minute average`}</dd></div>
      <div><dt>Rule reference</dt><dd>{reference.price == null ? 'Price retained as digest only' : `${reference.price} ${rules.quoteAsset}`} · {reference.digest}</dd></div>
      <div><dt>Provider time</dt><dd>{reference.providerObservedAt}</dd></div>
      <div><dt>First receipt</dt><dd>{reference.receivedAt}</dd></div>
    </dl>)}
    {rules.referenceFailure && <p role="status">Reference unavailable: {human(rules.referenceFailure)}. Refresh the required inputs to recover.</p>}
    {actions}
    {rules.unresolvedObligations.length > 0 && <><h4>Unresolved obligations</h4><ul>{rules.unresolvedObligations.map((reason, index) => <li key={index}>{reason}</li>)}</ul></>}
  </section>;
}
export function CurrentSpotRules({ workspaceId, proposalId }: { workspaceId: string; proposalId: string }) {
  const key = ['proposal-spot-rules', workspaceId, proposalId];
  const queryClient = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const query = useQuery({ queryKey: key, queryFn: () => request('trade.spot_rules.get', { workspaceId, proposalId }), refetchInterval: 1000, retry: false });
  const refresh = async () => {
    if (busy || !query.data) return;
    setBusy(true); setError(undefined);
    try {
      const result = await request('trade.spot_rules.refresh', { workspaceId, proposalId, expectedStateVersion: query.data.stateVersion });
      queryClient.setQueryData(key, result);
    } catch (cause) { setError(explainError(cause)); await query.refetch(); }
    finally { setBusy(false); }
  };
  return <>
    {query.isPending && <p role="status">Loading Proposal Spot rules…</p>}
    {query.isError && <div role="alert"><p>Proposal Spot rules unavailable: {explainError(query.error)}</p><button type="button" onClick={() => void query.refetch()}>Reload Proposal Spot rules</button></div>}
    {query.data && !query.isError && <SpotRulesExplanation rules={query.data} actions={<div>
      <button type="button" disabled={busy || query.isError || !query.data.materialVersion || !query.data.referencePurposes.length} onClick={() => void refresh()}>{busy ? 'Reading required rule references…' : 'Refresh required rule references'}</button>
      <p>Only the required public inputs are read. No account key is sent, and this action does not approve or place an order.</p>
      {!query.data.materialVersion && <p>Select and refresh the exact account and instrument rule source in Settings first.</p>}
    </div>} />}
    {error && <p role="alert">{error}</p>}
  </>;
}
