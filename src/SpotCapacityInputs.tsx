import { useState } from 'react';
import type { ReactNode } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import type { SpotCapacityInputs } from '../shared/ipc-types.ts';

const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');
const count = (value: string | null | undefined) => value ?? 'Not collected';

export function SpotCapacityExplanation({ inputs, captured = false, actions }: { inputs: SpotCapacityInputs; captured?: boolean; actions?: ReactNode }) {
  const observed = inputs.observation;
  const counts = observed?.counts;
  return <section className="card spot-capacity-inputs" aria-label={captured ? 'Captured Proposal Spot capacity inputs' : 'Current Proposal Spot capacity inputs'}>
    <h3>{captured ? 'Captured Proposal Spot capacity inputs' : 'Current Proposal Spot capacity inputs'}</h3>
    <p><strong>{human(inputs.status)} · Read-only inputs · Execution qualification unavailable</strong></p>
    <p>{captured ? 'Saved assessment. Reading this history never renews its collection receipts or consent.' : 'Current observations for the saved, unchanged Proposal. Refresh reads only its required account inputs.'} These inputs cannot approve an order. Rights, quotes, permissions, fees, FX, funding, arming and immediate preflight need their own checks.</p>
    <dl className="data-source-details">
      <div><dt>Proposal</dt><dd>{inputs.proposalId} · {inputs.proposalHash}</dd></div>
      <div><dt>Exact account</dt><dd>{inputs.accountId}</dd></div>
      <div><dt>Instrument / units</dt><dd>{inputs.instrumentId} · BASE {inputs.baseAsset} / QUOTE {inputs.quoteAsset}. USDT is not USD.</dd></div>
      <div><dt>Rule source / material</dt><dd>{inputs.sourceVersion} · {inputs.ruleMaterialVersion ?? 'Current exact rules unavailable'}</dd></div>
      <div><dt>Required reads</dt><dd>{inputs.purposes.length ? inputs.purposes.map(human).join(' · ') : inputs.ruleMaterialVersion ? 'No capacity read required' : 'Required purposes unavailable until exact rules are refreshed'}</dd></div>
    </dl>
    {observed && <>
      <p>Separate account, order and list responses are non-atomic. Local receipts do not establish a common provider snapshot or execution freshness.</p>
      <dl className="data-source-details">
        <div><dt>Collection / quality</dt><dd>{observed.collectionId} · Read-only Spot capacity</dd></div>
        <div><dt>Aggregate provider observation time</dt><dd>{observed.providerObservedAt ?? 'Not supplied'}</dd></div>
        <div><dt>Returned order coverage</dt><dd>{counts?.orderCoverageComplete ? 'Complete bounded response for the indicated read scope' : 'Incomplete or unresolved'}</dd></div>
        <div><dt>Account / selected-symbol open orders</dt><dd>{count(counts?.accountOpenOrders)} / {count(counts?.symbolOpenOrders)}</dd></div>
        <div><dt>Account / selected-symbol algorithmic orders</dt><dd>{count(counts?.accountAlgoOrders)} / {count(counts?.symbolAlgoOrders)} · Classification {counts?.classificationsComplete ? 'observed' : 'unresolved'}</dd></div>
        <div><dt>Account / selected-symbol iceberg orders</dt><dd>{count(counts?.accountIcebergOrders)} / {count(counts?.symbolIcebergOrders)}</dd></div>
        <div><dt>Account / selected-symbol order lists</dt><dd>{count(counts?.accountOpenOrderLists)} / {count(counts?.symbolOpenOrderLists)} · Leg coverage {counts?.listCoverageComplete == null ? 'not collected' : counts.listCoverageComplete ? 'observed' : 'unresolved'}</dd></div>
        <div><dt>Missing or pending list legs</dt><dd>{count(counts?.missingListLegs)}. Missing legs are not a zero reservation.</dd></div>
        {observed.baseBalance && <div><dt>Original BASE free / locked</dt><dd>{observed.baseBalance.free} / {observed.baseBalance.locked} {inputs.baseAsset}</dd></div>}
        {observed.position && <>
          <div><dt>Selected-symbol open BUY original / executed quantity</dt><dd>{observed.position.selectedSymbolOpenBuyOriginalQuantity} / {observed.position.selectedSymbolOpenBuyExecutedQuantity} {inputs.baseAsset}. No remaining quantity or position qualification is inferred.</dd></div>
          <div><dt>Foreign-symbol asset coverage</dt><dd>{observed.position.assetExposureComplete ? 'Observed scope; qualification remains unavailable' : 'Unresolved. No BASE membership is inferred from a ticker prefix.'}</dd></div>
        </>}
      </dl>
      {observed.reads.map(read => <dl className="data-source-details" key={read.kind}>
        <div><dt>Read scope</dt><dd>{human(read.kind)}</dd></div>
        <div><dt>Collection start / original receipt</dt><dd>{read.startedAt} / {read.receivedAt}</dd></div>
        <div><dt>Original provider update-time range (milliseconds)</dt><dd>{read.oldestProviderUpdateTimeMs ?? 'Not supplied'} / {read.latestProviderUpdateTimeMs ?? 'Not supplied'}. State-change times are not collection receipts.</dd></div>
        {read.kind === 'OPEN_ORDER_LISTS' && <div><dt>Original provider transaction-time range (milliseconds)</dt><dd>{read.oldestProviderTransactionTimeMs ?? 'Not supplied'} / {read.latestProviderTransactionTimeMs ?? 'Not supplied'}. List transaction times are not collection receipts.</dd></div>}
        <div><dt>Material digest</dt><dd>{read.digest}</dd></div>
      </dl>)}
      <h4>Unresolved capacity obligations</h4><ul>{observed.unresolvedObligations.map(reason => <li key={reason}>{human(reason)}</li>)}</ul>
    </>}
    {!observed && <p>{inputs.status === 'STALE' ? 'The original collection expired. Explicitly refresh the required inputs for a new observation.' : 'No current capacity observation is available. A missing observation cannot be treated as zero orders or zero balances.'}</p>}
    {inputs.failure && <p role="status">Capacity read unavailable: {human(inputs.failure)}. Resolve the source, authentication, time or provider wait, then explicitly refresh.</p>}
    {actions}
  </section>;
}

export function CurrentSpotCapacity({ workspaceId, proposalId }: { workspaceId: string; proposalId: string }) {
  const key = ['proposal-spot-capacity', workspaceId, proposalId];
  const client = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const query = useQuery({ queryKey: key, queryFn: () => request('trade.spot_capacity.get', { workspaceId, proposalId }), refetchInterval: 1000, retry: false });
  const refresh = async () => {
    if (busy || !query.data) return;
    setBusy(true); setError(undefined);
    try {
      const result = await request('trade.spot_capacity.refresh', { workspaceId, proposalId, expectedStateVersion: query.data.stateVersion });
      client.setQueryData(key, result);
    } catch (cause) { setError(explainError(cause)); await query.refetch(); }
    finally { setBusy(false); }
  };
  return <>
    {query.isPending && <p role="status">Loading Proposal capacity inputs…</p>}
    {query.isError && <div role="alert"><p>Proposal capacity inputs unavailable: {explainError(query.error)}</p><button type="button" onClick={() => void query.refetch()}>Reload Proposal capacity inputs</button></div>}
    {query.data && !query.isError && <SpotCapacityExplanation inputs={busy ? { ...query.data, status: 'NOT_OBSERVED', observation: null, failure: null } : query.data} actions={<div>
      <button type="button" disabled={busy || !query.data.ruleMaterialVersion || !query.data.purposes.length} onClick={() => void refresh()}>{busy ? 'Reading required capacity inputs…' : 'Refresh required capacity inputs'}</button>
      <p>This authenticated read uses your existing account connection. It does not arm, approve, reserve or send an order.</p>
      {!query.data.ruleMaterialVersion && <p>Refresh the exact account and instrument rule source in Settings first.</p>}
    </div>} />}
    {error && <p role="alert">{error}</p>}
  </>;
}
