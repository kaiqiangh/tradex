import type { BinanceLocalUseAttestation } from '../shared/ipc-types.ts';

export function BinanceLocalUseSummary({ attestation, captured = false }: {
  attestation: BinanceLocalUseAttestation;
  captured?: boolean;
}) {
  return <section className="local-use-attestation-summary" aria-label={captured ? 'Captured Binance local-use confirmation' : 'Current Binance local-use confirmation'}>
    <h4>{captured ? 'Captured local-use confirmation' : 'Current local-use confirmation'}</h4>
    <p><strong>Confirmed by you for local-only decision support.</strong> Provider entitlement remains independently UNVERIFIED. This is your reported context, not Binance verification, legal approval, or permission for commercial or external use.</p>
    <dl className="health-grid">
      <div><dt>Bound source</dt><dd>{attestation.providerId} · {attestation.sourceId} · generation {attestation.sourceGeneration}</dd></div>
      <div><dt>Environment / dataset profile</dt><dd>{attestation.environment} · {attestation.datasetProfile}</dd></div>
      <div><dt>Instruments</dt><dd>{attestation.symbols.map(symbol => symbol.replace(/([A-Z]+)(USDT)$/, '$1 / $2')).join(', ')}</dd></div>
      <div><dt>Fee valuation scope</dt><dd>{attestation.feeSymbolScope === 'BNB_USDT_ONLY_WHEN_REQUIRED_FOR_FEE_VALUATION' ? 'BNB / USDT only when required for fee valuation' : attestation.feeSymbolScope}</dd></div>
      <div><dt>Declared jurisdiction</dt><dd>{attestation.declaredJurisdiction}</dd></div>
      <div><dt>Purpose / destination</dt><dd>{attestation.scope === 'LOCAL_TRADING_DECISION_SUPPORT' ? 'Own local, non-commercial trading decision support' : attestation.scope} · {attestation.destination}</dd></div>
      <div><dt>Retention boundary</dt><dd>{attestation.retentionBoundary}</dd></div>
      <div><dt>Policy version</dt><dd>{attestation.policyVersion}</dd></div>
      <div><dt>Terms / review date</dt><dd><a href={attestation.termsUrl} target="_blank" rel="noreferrer">Current Binance terms</a> · {attestation.termsReviewedAt}</dd></div>
      <div><dt>Confirmation / provenance</dt><dd>{attestation.confirmedAt} · {attestation.provenance}</dd></div>
      <div className="local-use-digest"><dt>Binding digest</dt><dd><code>{attestation.bindingDigest}</code></dd></div>
    </dl>
  </section>;
}
