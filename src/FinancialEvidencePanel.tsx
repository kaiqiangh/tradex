import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import type { FinancialSourceEvidence, FxReviewEvidence, BrokerInstrumentMetadata, FinancialSourceConnection, FxRequirements, KnownCompanyEvent, MarketFinancialEvidence } from '../shared/ipc-types.ts';

type BinanceSpotRuleEvidence = Extract<FinancialSourceEvidence, { kind: 'BINANCE_SPOT_RULES' }>;

const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');
const pageSize = 25;

function SpotRules({ evidence }: { evidence: BinanceSpotRuleEvidence }) {
  return <section aria-label="Binance exact Spot rule evidence">
    <h4>{evidence.providerSymbol} · {human(evidence.symbolStatus)}</h4>
    <p>Read-only rule collection. The reviewed order, references, current balances/counts, quotes and data rights still need independent checks.</p>
    <dl className="data-source-details">
      <div><dt>Canonical instrument</dt><dd>{evidence.instrumentId}</dd></div>
      <div><dt>BASE / QUOTE</dt><dd>{evidence.baseAsset} / {evidence.quoteAsset} — USDT is not USD</dd></div>
      <div><dt>Native precision</dt><dd>BASE {evidence.baseAssetPrecision} · QUOTE {evidence.quoteAssetPrecision}</dd></div>
      <div><dt>Account type / can trade</dt><dd>{evidence.accountType} · {String(evidence.accountCanTrade)}</dd></div>
      <div><dt>Account permission-set membership</dt><dd>{String(evidence.permissionSetsSatisfied)} · OR inside each set, AND between sets</dd></div>
      <div><dt>Account permissions</dt><dd>{evidence.accountPermissions.join(', ') || 'No permissions supplied'}</dd></div>
      <div><dt>Required permission sets</dt><dd>{evidence.permissionSets.map(set => `(${set.join(' OR ')})`).join(' AND ') || 'No additional sets supplied'}</dd></div>
      <div><dt>Spot / quote Market quantity</dt><dd>{String(evidence.spotTradingAllowed)} / {String(evidence.quoteOrderQtyMarketAllowed)}</dd></div>
      <div><dt>Order forms</dt><dd>{evidence.orderTypes.join(', ')}</dd></div>
      <div><dt>Self-trade prevention</dt><dd>{evidence.defaultSelfTradePreventionMode} · {evidence.allowedSelfTradePreventionModes.join(', ')} · account requirement {evidence.accountRequiresSelfTradePrevention == null ? 'Not supplied' : String(evidence.accountRequiresSelfTradePrevention)}</dd></div>
      <div><dt>System / API lock</dt><dd>{evidence.tradingStatus.systemStatus} / {evidence.tradingStatus.apiTradingLocked ? 'LOCKED' : 'UNLOCKED'}</dd></div>
      <div><dt>Provider clock sample</dt><dd>{evidence.providerClockSample} — not a rule update or quote timestamp</dd></div>
    </dl>
    {evidence.admissionBlockers.length > 0 && <div role="status"><strong>Admission blockers</strong><ul>{evidence.admissionBlockers.map(code => <li key={code}>{human(code)}</li>)}</ul></div>}
    <details><summary>{evidence.constraints.length} scoped constraints</summary>{evidence.constraints.map((constraint, index) => <section key={index}>
      <h5>{constraint.ruleType} · {human(constraint.scope)} · {human(constraint.origin)}</h5>
      <p>{constraint.knownSchema ? 'Known field schema; execution qualification remains pending.' : 'Unsupported schema — cannot qualify an order.'}</p>
      <dl className="data-source-details">{constraint.fields.map(field => <div key={field.name}><dt>{field.name}</dt><dd>{String(field.value.value)}{field.disabled ? ' · disabled component' : ''}</dd></div>)}</dl>
      {constraint.unsupportedFields.length > 0 && <p>Unqualified fields: {constraint.unsupportedFields.join(', ')}</p>}
    </section>)}</details>
    <h5>Unresolved execution obligations</h5><ul>{evidence.unresolvedObligations.map(code => <li key={code}>{human(code)}</li>)}</ul>
  </section>;
}

function CompanyEvents({ actions, instrumentId }: { actions: KnownCompanyEvent[]; instrumentId?: string }) {
  const [page, setPage] = useState(0);
  const rows = instrumentId ? actions.filter(action => action.instrumentIds.includes(instrumentId)) : actions;
  const last = Math.max(0, Math.ceil(rows.length / pageSize) - 1);
  const current = Math.min(page, last);
  return <>
    <p>{rows.length} known events in this query. Dates retain the provider’s date-only precision.</p>
    {!rows.length && <p>No matching processed events were returned. This does not establish that no actions are pending.</p>}
    <ul className="corporate-actions">{rows.slice(current * pageSize, (current + 1) * pageSize).map(action => <li key={action.actionId}>
      <strong>{human(action.category)} · {action.partial ? 'Partial record' : 'Returned fields complete'}</strong>
      <span>Process date {action.processDate} · {action.instrumentIds.join(', ')}</span>
      <small className="identity">Event {action.actionId}</small>
      {action.dates.map(date => <span key={date.name}>{human(date.name)}: {date.value}</span>)}
      {action.securities.map((security, index) => <span key={index}>{human(security.role)} security: {security.symbol ?? 'Symbol unavailable'} · ISIN {security.isin ?? 'Unavailable'} · CUSIP {security.cusip ?? 'Unavailable'}</span>)}
      {action.terms.map(term => <span key={term.name}>{human(term.name)}: {term.value}</span>)}
      {action.stockMovements.map((movement, index) => <span key={index}>Stock movement: {movement.security.symbol ?? 'Unavailable'} · ISIN {movement.security.isin ?? 'Unavailable'} · CUSIP {movement.security.cusip ?? 'Unavailable'} · New rate {movement.newRate ?? 'Unavailable'} · Source rate {movement.sourceRate ?? 'Unavailable'}</span>)}
      <span>Currency {action.currency ?? 'Not supplied'} · Special {action.special === undefined ? 'Not supplied' : String(action.special)} · Foreign {action.foreign === undefined ? 'Not supplied' : String(action.foreign)}</span>
      {action.subType && <span>Subtype: {action.subType}</span>}{action.lotteryType && <span>Lottery: {action.lotteryType}</span>}
    </li>)}</ul>
    {last > 0 && <nav aria-label="Known company event pages"><button type="button" disabled={current === 0} onClick={() => setPage(current - 1)}>Previous events</button><span role="status">Event page {current + 1} of {last + 1}</span><button type="button" disabled={current === last} onClick={() => setPage(current + 1)}>Next events</button></nav>}
  </>;
}

function BrokerInstrument({ instrument }: { instrument: BrokerInstrumentMetadata }) {
  const [page, setPage] = useState(0);
  const last = Math.max(0, Math.ceil(instrument.scheduleEvents.length / pageSize) - 1);
  const current = Math.min(page, last);
  return <section aria-label={`${instrument.providerSymbol} metadata`}>
    <h4>{instrument.displayName} · {instrument.providerSymbol}</h4>
    <dl className="data-source-details">
      <div><dt>Instrument / ISIN</dt><dd>{instrument.instrumentId} · {instrument.isin}</dd></div>
      <div><dt>Canonical security identity</dt><dd>{human(instrument.canonicalSecurityIdentity)} — no authoritative canonical ISIN mapping is available.</dd></div>
      <div><dt>Currency</dt><dd>{instrument.currency}</dd></div>
      <div><dt>Provider exchange / schedule</dt><dd>{instrument.exchangeName} · {instrument.workingScheduleId}. This is not proof of the execution venue or current market state.</dd></div>
      <div><dt>Maximum open quantity</dt><dd>{instrument.maxOpenQuantity ?? 'Not supplied'} — metadata, not a current permission check.</dd></div>
      <div><dt>Extended hours metadata</dt><dd>{instrument.extendedHours === undefined ? 'Not supplied' : String(instrument.extendedHours)}</dd></div>
    </dl>
    <details><summary>{instrument.scheduleEvents.length} provider schedule events</summary>
      <ul>{instrument.scheduleEvents.slice(current * pageSize, (current + 1) * pageSize).map((event, index) => <li key={index}>{human(event.eventType)} · {event.date}</li>)}</ul>
      {last > 0 && <nav aria-label={`${instrument.providerSymbol} schedule pages`}><button type="button" disabled={current === 0} onClick={() => setPage(current - 1)}>Previous schedule events</button><span>Schedule page {current + 1} of {last + 1}</span><button type="button" disabled={current === last} onClick={() => setPage(current + 1)}>Next schedule events</button></nav>}
    </details>
  </section>;
}

export function FinancialEvidencePanel({ source, instrumentId, capturedAt, showRequirements = true }: { source: FinancialSourceConnection; instrumentId?: string; capturedAt?: string; showRequirements?: boolean }) {
  const evidence = source.evidence;
  return <div className="financial-evidence">
    <p role="status"><strong>{capturedAt ? 'Captured status: ' : ''}{human(source.status)}</strong> · {capturedAt ? 'Captured assessment: ' : ''}{source.availabilityReason}</p>
    <dl className="data-source-details">
      <div><dt>Saved account</dt><dd>{source.connectionId ?? 'None selected'}</dd></div>
      {source.instrumentId && <div><dt>Selected rule instrument</dt><dd>{source.instrumentId}</dd></div>}
      <div><dt>First receipt</dt><dd>{source.observedAt ?? 'Not checked'}</dd></div>
      {source.capabilityStatuses.map(capability => <div key={capability.capability}><dt>{human(capability.capability)}</dt><dd>{capturedAt ? 'Captured status: ' : ''}{human(capability.status)} · {capability.reason}</dd></div>)}
    </dl>
    {showRequirements && source.fxRequirements && <FxRequirementsPanel requirements={source.fxRequirements} />}
    {evidence && <>
      {source.status !== 'AVAILABLE' && <p className="error-text">{capturedAt ? 'This observation was unavailable when the review was captured.' : 'Retained observation is unavailable for current decisions.'} Refresh the saved source to obtain new evidence.</p>}
      <dl className="data-source-details"><div><dt>Provider quality</dt><dd>{evidence.providerQuality === 'READ_ONLY_SPOT_RULES' ? 'Read-only Spot rules; per-order execution qualification pending' : evidence.providerQuality === 'TEN_MINUTE_METADATA' ? 'Ten-minute provider metadata' : evidence.providerQuality === 'UNQUALIFIED_FX_RATE' ? 'Read-only FX rate; transaction-grade qualification unavailable' : 'Delayed process-date query'}. A recent receipt does not improve provider quality.</dd></div>
        <div><dt>Provider observation time</dt><dd>{evidence.providerObservedAt ?? 'Not supplied'}</dd></div>
        <div><dt>Account / source version</dt><dd className="identity">{evidence.binding.accountVersion} · {evidence.binding.sourceVersion}</dd></div>
        <div><dt>Material version</dt><dd className="identity">{evidence.materialVersion}</dd></div>
      </dl>
      {evidence.kind === 'BINANCE_SPOT_RULES' ? <SpotRules evidence={evidence} /> : evidence.kind === 'CORPORATE_ACTIONS' ? <>
        <p>Requested process dates: {evidence.coverageStart} – {evidence.coverageEnd}. Query {evidence.queryComplete ? 'exhausted all returned pages' : 'incomplete'}; complete action coverage and historical adjustment remain unavailable.</p>
        <CompanyEvents key={`${evidence.materialVersion}:${instrumentId ?? 'all'}`} actions={evidence.actions} instrumentId={instrumentId} />
      </> : evidence.kind === 'BROKER_INSTRUMENTS' ? <><p>Account currency: {evidence.accountCurrency}. Current account tradability and exchange halts remain unavailable.</p>{evidence.instruments.filter(instrument => !instrumentId || instrument.instrumentId === instrumentId).map(instrument => <BrokerInstrument key={`${evidence.materialVersion}:${instrument.instrumentId}`} instrument={instrument} />)}</> : <><p>Observed routes belong to {evidence.requirements.proposalId ? 'the captured immutable proposal and portfolio context' : 'the captured portfolio context'}. Changing the intent or refreshing in Settings requires a new read for that context.</p><ul>{evidence.rates.map(rate => <li key={rate.providerPair}><strong>{rate.fromCurrency} → {rate.toCurrency}</strong><dl className="data-source-details"><div><dt>Bid / ask</dt><dd>{rate.bid} / {rate.ask}</dd></div><div><dt>Provider mid</dt><dd>{rate.mid} — supplied independently; not a funding conversion</dd></div><div><dt>Provider rate time</dt><dd>{rate.providerTimestamp}</dd></div></dl></li>)}</ul></>}
    </>}
  </div>;
}

export function MarketFinancialEvidencePanel({ evidence, instrumentId, capturedAt }: { evidence: MarketFinancialEvidence; instrumentId: string; capturedAt?: string }) {
  return <section aria-label="Financial prerequisite evidence"><h3>Financial prerequisite evidence</h3>
    {capturedAt && <p role="status">Captured financial evidence · {capturedAt}. This immutable review does not report current eligibility. Evidence can expire or change while this window is open. The backend revalidates current evidence before each protected action; reopen the review to obtain a new assessment.</p>}
    <h4>Known company events</h4><FinancialEvidencePanel source={evidence.companyEvents} instrumentId={instrumentId} capturedAt={capturedAt} />
    <h4>Account instrument metadata</h4><FinancialEvidencePanel source={evidence.brokerInstruments} instrumentId={instrumentId} capturedAt={capturedAt} />
  </section>;
}


export function FxRequirementsPanel({ requirements }: { requirements: FxRequirements }) {
  return <section aria-label="Required currency routes"><h4>Required currency routes</h4>
    <p>{requirements.proposalId ? 'Selected immutable proposal and portfolio context. ' : 'Current portfolio context. '}Workspace base currency: {requirements.baseCurrency}. Currency requirements do not establish complete monetary inputs or conversion eligibility.</p>
    {requirements.requirements.length ? <ul>{requirements.requirements.map((route, index) => <li key={index}>
      <strong>{route.fromCurrency ?? 'Unknown currency'} → {route.toCurrency ?? 'Unknown currency'}</strong> · {human(route.purpose)} · {human(route.need)}
      <p>{route.reason}</p>
    </li>)}</ul> : <p>No monetary currency routes are currently known.</p>}
  </section>;
}

export function FxSourceContext({ workspaceId, proposalId }: { workspaceId: string; proposalId?: string }) {
  const queryClient = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const requirements = useQuery({ queryKey: ['fx-requirements', workspaceId, proposalId], queryFn: () => request('data.fx.requirements', { workspaceId, ...(proposalId ? { proposalId } : {}) }), retry: false, refetchInterval: 1000 });
  const source = useQuery({ queryKey: ['financial-source', workspaceId, 'fx'], queryFn: () => request('data.fx.connection', { workspaceId }), retry: false, refetchInterval: 1000 });
  const refreshIntent = async () => {
    if (!proposalId || !source.data?.configured || busy) return;
    setBusy(true); setError(undefined);
    try {
      const result = await request('data.fx.refresh', { workspaceId, proposalId, expectedStateVersion: source.data.stateVersion });
      queryClient.setQueryData(['financial-source', workspaceId, 'fx'], result);
      if (result.status !== 'AVAILABLE') setError(result.availabilityReason);
    } catch (cause) { setError(explainError(cause)); await source.refetch(); }
    finally { setBusy(false); }
  };
  return <section className="card" aria-label="Currency evidence"><h3>Currency evidence</h3>
    {requirements.isPending ? <p role="status">Loading currency requirements…</p> : requirements.isError ? <div role="alert"><p>{explainError(requirements.error)}</p><button type="button" onClick={() => void requirements.refetch()}>Reload currency requirements</button></div> : <FxRequirementsPanel requirements={requirements.data} />}
    {source.isPending ? <p role="status">Loading saved currency source…</p> : source.isError ? <div role="alert"><p>{explainError(source.error)}</p><button type="button" onClick={() => void source.refetch()}>Reload currency source</button></div> : source.data.configured ? <FinancialEvidencePanel source={source.data} showRequirements={false} /> : <p>No saved currency-rate source is selected. Configure a source in Settings if an external rate is required.</p>}
    {proposalId && source.data?.configured && <button type="button" disabled={busy || requirements.isPending || requirements.isError} onClick={() => void refreshIntent()}>{busy ? 'Reading currency rates…' : 'Refresh rates for this proposal'}</button>}
    {error && <p role="alert">{error}</p>}
  </section>;
}

export function CapturedCurrencyEvidence({ evidence, reviewedAt }: { evidence: FxReviewEvidence; reviewedAt: string }) {
  return <section aria-label="Captured currency evidence"><h3>Captured currency evidence</h3><p className="muted">Captured at: <time dateTime={reviewedAt}>{reviewedAt}</time></p><p>This immutable review retains the original rates and assessment. Polling does not renew consent; the backend checks current material again before a protected action.</p><FxRequirementsPanel requirements={evidence.requirements} /><FinancialEvidencePanel source={evidence.source} capturedAt={reviewedAt} showRequirements={false} /></section>;
}

export function SpotRuleContext({ workspaceId, accountId, instrumentId }: { workspaceId: string; accountId?: string; instrumentId: string }) {
  const source = useQuery({ queryKey: ['binance-rule-source', workspaceId], queryFn: () => request('data.binance_rules.connection', { workspaceId }), retry: false, refetchInterval: 1000 });
  return <section className="card" aria-label="Spot rule evidence for current intent"><h3>Spot rule evidence for current intent</h3>
    {source.isPending ? <p role="status">Loading saved Spot rule source…</p> : source.isError ? <div role="alert"><p>{explainError(source.error)}</p><button type="button" onClick={() => void source.refetch()}>Reload Spot rule evidence</button></div> : source.data.connectionId === accountId && source.data.instrumentId === instrumentId ? <FinancialEvidencePanel source={source.data} instrumentId={instrumentId} /> : <p>No rule source matches this exact account and instrument. Select and refresh it in Settings. Other saved rule observations cannot qualify this intent.</p>}
  </section>;
}
