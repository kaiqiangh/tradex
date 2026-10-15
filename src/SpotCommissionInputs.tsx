import { useState } from 'react';
import type { ReactNode } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import type { SpotCommissionInputs } from '../shared/ipc-types.ts';
const human = (value: string) => value.toLowerCase().replaceAll('_',' ');

export function SpotCommissionExplanation({inputs,captured=false,actions}:{inputs:SpotCommissionInputs;captured?:boolean;actions?:ReactNode}) {
  const observed=inputs.observation;
  return <section className="card" aria-label={`${captured?'Captured':'Current'} Proposal symbol commission terms`}>
    <h3>{captured?'Captured':'Current'} Proposal symbol commission terms</h3>
    <p><strong>{human(inputs.status)} · Read-only declaration · Execution qualification unavailable</strong></p>
    <p>{captured?'Saved assessment. Reading this history never renews its receipts, calls Binance or restores consent.':'Declared terms for this saved Proposal. Only an explicit read collects new account and symbol responses.'} Source validity is separate from complete monetary fees and required execution FX. This is not an approval or a fill-time fee promise.</p>
    <dl className="data-source-details">
      <div><dt>Exact account · Proposal</dt><dd>{inputs.accountId} · {inputs.proposalId} · {inputs.proposalHash}</dd></div>
      <div><dt>Instrument · units</dt><dd>{inputs.instrumentId} · BASE {inputs.baseAsset} / QUOTE {inputs.quoteAsset}. USDT ≠ USD.</dd></div>
      <div><dt>Rule source · material</dt><dd>{inputs.sourceVersion} · {inputs.ruleMaterialVersion??'Current exact rules unavailable'}</dd></div>
    </dl>
    {observed && <>
      <p>Origin: signed Binance account identity and symbol commission declaration. These database responses are separate and non-atomic; local receipts do not prove execution freshness.</p>
      <dl className="data-source-details">
        <div><dt>Declared symbol · remote account</dt><dd>{observed.symbol} · {observed.remoteAccountId}</dd></div>
        <div><dt>Declared provider observation time</dt><dd>{observed.providerObservedAt??'not supplied'}. No local time is substituted.</dd></div>
        <div><dt>Known commission terms</dt><dd>{observed.termsComplete?'All three known families declared':'Unresolved active fee terms; no completeness inferred'}. Fee amount and charging asset remain unqualified.</dd></div>
      </dl>
      {[
        ['standard commission',observed.standardCommission],['tax commission',observed.taxCommission],['special commission',observed.specialCommission],
      ].map(([label,rates])=><div key={label as string}>
        <h4>Declared {label as string}</h4>
        <p>Origin: Binance symbol {label as string} declaration. Original rates per received-asset unit; no rounding or fee amount.</p>
        <dl className="data-source-details">{Object.entries(rates).map(([role,rate])=><div key={role}><dt>{role}</dt><dd>{rate}</dd></div>)}</dl>
      </div>)}
      <dl className="data-source-details">
        <div><dt>Declared discount account · symbol flags</dt><dd>{String(observed.discount.enabledForAccount)} / {String(observed.discount.enabledForSymbol)} · origin: symbol discount declaration</dd></div>
        <div><dt>Declared discount asset · value</dt><dd>{observed.discount.discountAsset} / {observed.discount.discount} · origin: symbol discount declaration. Value is preserved verbatim; no discount arithmetic.</dd></div>
        <div><dt>{inputs.side} received asset</dt><dd>{observed.receivedAsset} · origin: venue received-asset charging rule, applied to this Proposal side and canonical BASE/QUOTE. A conditional charging branch, not a promised fee currency.</dd></div>
        <div><dt>Conditional discount payment</dt><dd>{observed.conditionalDiscountAsset?`${observed.conditionalDiscountAsset} requires sufficient balance and conversion evidence; otherwise fees fall back to received ${observed.receivedAsset}.`:`Discount payment is not enabled for both account and symbol; the declared received-asset branch is ${observed.receivedAsset}.`} Future charging asset and fee amount are not execution-qualified.</dd></div>
        <div><dt>Unknown active declaration fields</dt><dd>{observed.extensionKeys.length?observed.extensionKeys.join(' · '):'None reported'}</dd></div>
      </dl>
      {observed.reads.map(read=><dl className="data-source-details" key={read.kind}>
        <div><dt>Origin · read scope</dt><dd>{human(read.kind)} · original signed response</dd></div>
        <div><dt>Original local start · receipt</dt><dd>{read.startedAt} / {read.receivedAt}. Read-only collection times, not provider observation times.</dd></div>
        <div><dt>Original material digest</dt><dd>{read.digest}</dd></div>
      </dl>)}
      <h4>Carried limitations and unresolved obligations</h4><ul>{observed.unresolvedObligations.map(reason=><li key={reason}>{human(reason)}</li>)}</ul>
    </>}
    {!observed && <p>{inputs.status==='STALE'?'The original collection expired. Its saved assessments are unchanged.':'No current complete declaration is available. Missing terms are never zero fees.'}</p>}
    {inputs.failure && <p role="status">Commission read unavailable: {human(inputs.failure)}. Resolve the source, authentication or provider wait before explicitly reading again.</p>}
    {actions}
  </section>;
}

export function CurrentSpotCommission({workspaceId,proposalId}:{workspaceId:string;proposalId:string}) {
  const key=['proposal-spot-commission',workspaceId,proposalId],client=useQueryClient();
  const [busy,setBusy]=useState(false),[error,setError]=useState<string>();
  const query=useQuery({queryKey:key,queryFn:()=>request('trade.spot_commission.get',{workspaceId,proposalId}),refetchInterval:1000,retry:false});
  const refresh=async()=>{
    if(busy||!query.data)return;setBusy(true);setError(undefined);
    try{const value=await request('trade.spot_commission.refresh',{workspaceId,proposalId,expectedStateVersion:query.data.stateVersion});client.setQueryData(key,value);}
    catch(cause){setError(explainError(cause));await query.refetch();}finally{setBusy(false);}
  };
  return <>
    {query.isPending&&<p role="status">Loading Proposal commission terms…</p>}
    {query.isError&&<div role="alert"><p>Commission terms unavailable: {explainError(query.error)}</p><button type="button" onClick={()=>void query.refetch()}>Reload commission state</button></div>}
    {query.data&&!query.isError&&<SpotCommissionExplanation inputs={busy?{...query.data,status:'NOT_OBSERVED',observation:null,failure:null}:query.data} actions={<>
      <button type="button" disabled={busy||!query.data.ruleMaterialVersion} onClick={()=>void refresh()}>{busy?'Reading symbol commission terms…':'Read symbol commission terms'}</button>
      <p>This authenticated read uses the existing account connection. It does not arm, approve, reserve or send an order.</p>
    </>}/>}
    {error&&<p role="alert">{error}</p>}
  </>;
}
