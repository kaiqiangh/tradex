import { useState } from 'react';
import type { ReactNode } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import type { SpotOrderIntervalInputs } from '../shared/ipc-types.ts';

const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');

export function SpotOrderIntervalExplanation({ inputs, captured = false, actions }: { inputs: SpotOrderIntervalInputs; captured?: boolean; actions?: ReactNode }) {
  const observed = inputs.observation;
  return <section className="card spot-order-interval-inputs" aria-label={captured ? 'Captured Proposal order interval inputs' : 'Current Proposal order interval inputs'}>
    <h3>{captured ? 'Captured Proposal order interval inputs' : 'Current Proposal order interval inputs'}</h3>
    <p><strong>{human(inputs.status)} · Read-only inputs · Execution qualification unavailable</strong></p>
    <p>{captured ? 'Saved assessment. History does not refresh counters, renew receipts or restore consent.' : 'Current inputs for the saved, unchanged Proposal. Explicit refresh reads the actual definitions and authenticated account usage.'} These observations do not approve or reserve an order.</p>
    <dl className="data-source-details">
      <div><dt>Proposal</dt><dd>{inputs.proposalId} · {inputs.proposalHash}</dd></div>
      <div><dt>Exact account</dt><dd>{inputs.accountId}</dd></div>
      <div><dt>Account-wide interval scope</dt><dd>Across all keys, IPs and APIs for this account. Changing keys, workspaces or sources does not reset usage.</dd></div>
      <div><dt>Instrument / units</dt><dd>{inputs.instrumentId} · BASE {inputs.baseAsset} / QUOTE {inputs.quoteAsset}. USDT is not USD.</dd></div>
      <div><dt>Rule source / material</dt><dd>{inputs.sourceVersion} · {inputs.ruleMaterialVersion ?? 'Current exact rules unavailable'}</dd></div>
    </dl>
    <p>Open-order filter inventory, account interval usage, HTTP request budgets and local reservations are separate. A local fill, cancellation, expiry or absent response header cannot supply decrement credit or reset an observed counter. Rights, quotes, permissions, health, fees, FX, funding, Arm, consent and immediate authenticated preflight need their own checks.</p>
    {observed && <>
      <p>Definition, identity and counter responses are non-atomic. Original provider clock samples and local read receipts are not a provider counter snapshot time. Derived time association is uncertain; a possible UTC interval boundary retires the current observation. No reset time, remaining slots or execution-qualified interval is inferred.</p>
      <dl className="data-source-details">
        <div><dt>Collection</dt><dd>{observed.collectionId}</dd></div>
        <div><dt>Original provider counter snapshot time</dt><dd>{observed.providerObservedAt ?? 'Not supplied'}</dd></div>
        <div><dt>Declared interval coverage</dt><dd>{observed.coverageComplete ? 'Complete bounded tuple/limit coverage; execution qualification remains unavailable' : 'Incomplete or unresolved; missing counters are not zero or unlimited'}</dd></div>
        <div><dt>Original clock sample (milliseconds)</dt><dd>{observed.clock.serverTimeMs}</dd></div>
        <div><dt>Clock read start / original receipt</dt><dd>{observed.clock.startedAt} / {observed.clock.receivedAt}</dd></div>
        <div><dt>Derived local clock round-trip bound (milliseconds)</dt><dd>{observed.clock.localRoundTripBoundMs}. This bound is not a counter watermark or an admission promise.</dd></div>
      </dl>
      <h4>Actual declared rate limits</h4>
      <p>ORDERS describes account interval usage. REQUEST_WEIGHT and RAW_REQUESTS describe transport budgets, rather than available order slots.</p>
      {observed.declarations.map(row => <dl className="data-source-details" key={`${row.rateLimitType}:${row.interval}:${row.intervalNum}`}>
        <div><dt>Declared type / interval</dt><dd>{row.rateLimitType} · Every {row.intervalNum} {row.interval}</dd></div>
        <div><dt>Original declared limit</dt><dd>{row.limit}</dd></div>
      </dl>)}
      <h4>Original account interval counters</h4>
      {observed.counters.length === 0 && <p>No counters returned. Missing usage is not zero.</p>}
      {observed.counters.map(row => <dl className="data-source-details" key={`${row.rateLimitType}:${row.interval}:${row.intervalNum}`}>
        <div><dt>Observed type / interval</dt><dd>{row.rateLimitType} · Every {row.intervalNum} {row.interval}</dd></div>
        <div><dt>Original count / reported limit</dt><dd>{row.count} / {row.limit}. A reported zero limit stays zero; at/above-limit usage is not clamped.</dd></div>
      </dl>)}
      {observed.reads.map(read => <dl className="data-source-details" key={read.kind}>
        <div><dt>Read scope</dt><dd>{human(read.kind)}</dd></div>
        <div><dt>Read start / original receipt</dt><dd>{read.startedAt} / {read.receivedAt}</dd></div>
        <div><dt>Material digest</dt><dd>{read.digest}</dd></div>
      </dl>)}
      <h4>Unresolved interval obligations</h4><ul>{observed.unresolvedObligations.map(reason => <li key={reason}>{human(reason)}</li>)}</ul>
    </>}
    {!observed && <p>{inputs.status === 'STALE' ? 'The original observation retired. Explicitly refresh for a new read.' : 'No current interval observation is available. Missing usage is not zero or unlimited.'}</p>}
    {inputs.retirementReason && <p role="status">Observation retired: {human(inputs.retirementReason)}.</p>}
    {inputs.failure && <p role="status">Interval read unavailable: {human(inputs.failure)}. Resolve authentication, source, time or provider wait, then explicitly refresh.</p>}
    {inputs.providerWaitSeconds && <p>Provider request cooldown: {inputs.providerWaitSeconds} seconds remaining at this assessment. This is not an order interval reset time.</p>}
    {actions}
  </section>;
}

export function CurrentSpotOrderIntervals({ workspaceId, proposalId }: { workspaceId: string; proposalId: string }) {
  const key = ['proposal-spot-order-intervals', workspaceId, proposalId];
  const client = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const query = useQuery({ queryKey: key, queryFn: () => request('trade.spot_order_intervals.get', { workspaceId, proposalId }), refetchInterval: 1000, retry: false });
  const refresh = async () => {
    if (busy || !query.data) return;
    setBusy(true); setError(undefined);
    try {
      const result = await request('trade.spot_order_intervals.refresh', { workspaceId, proposalId, expectedStateVersion: query.data.stateVersion });
      client.setQueryData(key, result);
    } catch (cause) { setError(explainError(cause)); await query.refetch(); }
    finally { setBusy(false); }
  };
  return <>
    {query.isPending && <p role="status">Loading Proposal interval inputs…</p>}
    {query.isError && <div role="alert"><p>Proposal interval inputs unavailable: {explainError(query.error)}</p><button type="button" onClick={() => void query.refetch()}>Reload Proposal interval inputs</button></div>}
    {query.data && !query.isError && <SpotOrderIntervalExplanation inputs={busy ? { ...query.data, status: 'NOT_OBSERVED', observation: null, failure: null, retirementReason: null } : query.data} actions={<div>
      <button type="button" disabled={busy || !query.data.ruleMaterialVersion} onClick={() => void refresh()}>{busy ? 'Reading account interval inputs…' : 'Refresh account interval inputs'}</button>
      <p>This authenticated read uses your saved account. It does not arm, approve, reserve or send an order.</p>
      {!query.data.ruleMaterialVersion && <p>Refresh the exact account and instrument rule source in Settings first.</p>}
    </div>} />}
    {error && <p role="alert">{error}</p>}
  </>;
}
