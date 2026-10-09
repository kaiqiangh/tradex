import type { SpotOwningQualification } from '../shared/ipc-types.ts';

const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');
const declared = (value: string | null | undefined) => value ?? 'Not declared';

// One exact statement of how much of the reviewed intent the venue would currently admit, which
// declared facts bound it, and the single fact that stops it. The number and its blocker are
// rendered on one line so neither can be read without the other at any review width. The owning
// qualification only ever exists as part of a saved RiskDecision, so this is always a saved
// assessment: it has no live re-read, no refresh control and no polling.
export function SpotOwningExplanation({ qualification }: { qualification: SpotOwningQualification }) {
  const qualified = qualification.outcome === 'PASS';
  const blocker = qualification.bindingBlocker;
  return <section className="card spot-owning-admission" aria-label="Captured Proposal Spot owning admission">
    <h3>Captured Proposal Spot owning admission</h3>
    <p className={qualified ? 'muted' : 'error-text'} role={qualified ? 'status' : 'alert'}>
      <strong>{qualified
        ? `The venue's own declared figures admit this intent: ${declared(qualification.remainingOrderSlots)} further ${qualification.remainingOrderSlots === '1' ? 'order' : 'orders'} (${declared(qualification.bindingInterval)}).`
        : `${blocker ? human(blocker) : 'No declared admission available'}: ${qualification.reason}`}</strong>
    </p>
    {!qualified && <p>This is not a smaller number. Until the single fact above is resolved by a fresh, bound observation, no venue-declared slot can be relied on for this intent.</p>}
    <p>Saved assessment. Reading this history never renews the qualification, re-reads the venue or restores consent. Arm, consent, dispatch, fees, required FX, immediate authenticated preflight, private-stream readiness, funding and exact CANCEL/reconciliation remain separate checks.</p>
    <dl className="data-source-details">
      <div><dt>Proposal</dt><dd>{qualification.proposalId} · {qualification.proposalHash}</dd></div>
      <div><dt>Exact account</dt><dd>{qualification.accountId}</dd></div>
      <div><dt>Instrument / declared unit</dt><dd>{qualification.instrumentId} · BASE {qualification.baseAsset} / QUOTE {qualification.quoteAsset}. This statement is read in {qualification.capacityUnit}; USDT is not USD.</dd></div>
      <div><dt>Declared origin · open-order and balance inventory</dt><dd>{human(qualification.inventorySource)}. Covers the declared open-order count, the symbol open-buy quantity, the BASE free/locked figures and the capacity observation time.</dd></div>
      <div><dt>Declared origin · order-rate counters</dt><dd>{human(qualification.quotaSource)}. Covers the remaining slots, the binding declared bucket, any standing provider cooldown and the interval observation time.</dd></div>
      <div><dt>Remaining venue-declared order slots</dt><dd>{qualified ? `${qualification.remainingOrderSlots} · ${qualification.bindingInterval}` : 'None can be relied on'}. Derived from the tighter declared bucket, never stored, never decremented locally and never rolled forward to a new window.</dd></div>
      <div><dt>Declared open orders for this symbol</dt><dd>{declared(qualification.declaredSymbolOpenOrders)}</dd></div>
      <div><dt>Declared symbol open-buy quantity</dt><dd>{declared(qualification.declaredSymbolOpenBuyQuantity)}. Reported as declared exposure only; it is never read as available balance.</dd></div>
      <div><dt>Declared BASE free / locked</dt><dd>{declared(qualification.declaredBaseFree)} / {declared(qualification.declaredBaseLocked)}. Reported verbatim; order locks inside a provider's free figure are never subtracted a second time.</dd></div>
      <div><dt>Provider request cooldown at this assessment</dt><dd>{qualification.declaredProviderWaitSeconds ? `${qualification.declaredProviderWaitSeconds} seconds` : 'None declared'}. This is not an order-rate reset time and is not this qualification's gate.</dd></div>
      <div><dt>Declared capacity / interval observation times</dt><dd>{declared(qualification.capacityObservedAt)} / {declared(qualification.intervalObservedAt)}. The interval counters carry no provider timestamp, so any local time association stays explicitly uncertain.</dd></div>
    </dl>
    <h4>Standing limitations carried from the consumed read-only inputs</h4>
    <ul>{qualification.carriedLimitations.length === 0
      ? <li>No standing limitation reported with the underlying observations.</li>
      : qualification.carriedLimitations.map(limitation => <li key={limitation}>{human(limitation)}</li>)}</ul>
    <p>These describe how the evidence was obtained, not what it failed to cover. Separately read account, order and balance responses are not one atomic provider snapshot, and the venue recalculates its order-rate windows on its own clock, so a future taker phase can still expire. Nothing here is a fill promise, an admission guarantee or an approval.</p>
    <dl className="data-source-details">
      <div><dt>Bound evidence versions</dt><dd>{qualification.bindingVersion} · {qualification.stateVersion}</dd></div>
    </dl>
  </section>;
}
