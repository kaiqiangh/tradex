import { useState } from 'react';
import type { ReactNode } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import type { SpotProposalRules } from '../shared/ipc-types.ts';
const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');

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
