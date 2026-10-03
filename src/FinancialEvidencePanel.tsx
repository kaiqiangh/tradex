import { useState } from 'react';
import type { BrokerInstrumentMetadata, FinancialSourceConnection, KnownCompanyEvent, MarketFinancialEvidence } from '../shared/ipc-types.ts';

const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');
const pageSize = 25;

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

export function FinancialEvidencePanel({ source, instrumentId, capturedAt }: { source: FinancialSourceConnection; instrumentId?: string; capturedAt?: string }) {
  const evidence = source.evidence;
  return <div className="financial-evidence">
    <p role="status"><strong>{capturedAt ? 'Captured status: ' : ''}{human(source.status)}</strong> · {capturedAt ? 'Captured assessment: ' : ''}{source.availabilityReason}</p>
    <dl className="data-source-details">
      <div><dt>Saved account</dt><dd>{source.connectionId ?? 'None selected'}</dd></div>
      <div><dt>First receipt</dt><dd>{source.observedAt ?? 'Not checked'}</dd></div>
      {source.capabilityStatuses.map(capability => <div key={capability.capability}><dt>{human(capability.capability)}</dt><dd>{capturedAt ? 'Captured status: ' : ''}{human(capability.status)} · {capability.reason}</dd></div>)}
    </dl>
    {evidence && <>
      {source.status !== 'AVAILABLE' && <p className="error-text">{capturedAt ? 'This observation was unavailable when the review was captured.' : 'Retained observation is unavailable for current decisions.'} Refresh the saved source to obtain new evidence.</p>}
      <dl className="data-source-details"><div><dt>Provider quality</dt><dd>{evidence.providerQuality === 'TEN_MINUTE_METADATA' ? 'Ten-minute provider metadata' : 'Delayed process-date query'}. A recent receipt does not improve provider quality.</dd></div>
        <div><dt>Provider observation time</dt><dd>{evidence.providerObservedAt ?? 'Not supplied'}</dd></div>
        <div><dt>Account / source version</dt><dd className="identity">{evidence.binding.accountVersion} · {evidence.binding.sourceVersion}</dd></div>
        <div><dt>Material version</dt><dd className="identity">{evidence.materialVersion}</dd></div>
      </dl>
      {evidence.kind === 'CORPORATE_ACTIONS' ? <>
        <p>Requested process dates: {evidence.coverageStart} – {evidence.coverageEnd}. Query {evidence.queryComplete ? 'exhausted all returned pages' : 'incomplete'}; complete action coverage and historical adjustment remain unavailable.</p>
        <CompanyEvents key={`${evidence.materialVersion}:${instrumentId ?? 'all'}`} actions={evidence.actions} instrumentId={instrumentId} />
      </> : <><p>Account currency: {evidence.accountCurrency}. Current account tradability and exchange halts remain unavailable.</p>{evidence.instruments.filter(instrument => !instrumentId || instrument.instrumentId === instrumentId).map(instrument => <BrokerInstrument key={`${evidence.materialVersion}:${instrument.instrumentId}`} instrument={instrument} />)}</>}
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
