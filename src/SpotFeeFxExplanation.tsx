import type { FeeConversionReference, SpotFeeFxStatement, SpotRequiredRoute } from '../shared/ipc-types.ts';

const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');
const declared = (value: string | null | undefined) => value ?? 'Not declared';

// The exact literal is shared by the accessible name and the visible heading so neither can drift
// from the other.
const SURFACE_LABEL = 'Captured Proposal Spot fee and required FX';

// Keep purpose, rate quality and the exact qualification boundary together. A price-only
// valuation can qualify without pretending to establish executable depth or transaction cost.
const routeLine = (route: SpotRequiredRoute) => {
  const state = route.purpose === 'INTENT_POLICY' && route.state === 'QUALIFIED'
    ? 'qualified for price-only policy valuation'
    : human(route.state);
  const cost = route.purpose === 'INTENT_POLICY'
    ? 'execution depth/cost not applicable to price-only policy valuation'
    : `declared execution cost ${route.conservativeCost ?? 'not established'}`;
  return `${human(route.purpose)} FX route · ${route.fromCurrency} -> ${route.toCurrency} · ${state} · provider pair ${route.providerPair ?? 'no supported pair'} · provider rate quality ${declared(route.providerQuality)} · provider time ${declared(route.providerTimestamp)} · first receipt ${declared(route.firstReceipt)} · ${cost} · ${route.reason}`;
};

const bnbReferenceLine = (reference: FeeConversionReference) =>
  `${reference.pair} · ${reference.fromAsset} -> ${reference.toAsset} at ${reference.priceSide} · exact ${reference.coverageAsset} coverage ${reference.coverage.numerator}/${reference.coverage.denominator} · ${reference.source} snapshot ${reference.marketSnapshotId} · ${reference.sourceVersion} · ${reference.connectionGeneration} · update ${reference.bookUpdateId} · provider ${reference.providerTimestamp} · first receipt ${reference.firstReceipt} · use ${reference.dataUseRights} · material ${reference.materialHash}`;

// One exact statement of the reviewed intent's owning fee and genuinely-required FX routes.
// It reports two independent facts that must both remain readable: the single binding blocker and
// the fee currency. The owning statement only ever exists as part of a saved RiskDecision, so this
// is always a saved assessment: it has no live re-read, no refresh control and no polling.
export function SpotFeeFxExplanation({ statement }: { statement: SpotFeeFxStatement }) {
  const qualified = statement.outcome === 'PASS';
  const blocker = statement.bindingBlocker;
  const model = statement.currentFeeModel;
  const wallet = statement.walletFundingEvidence;
  const hasHistoricalAccountComparison = Boolean(
    statement.feeRateBasis || statement.declaredMakerRate || statement.declaredTakerRate ||
    statement.declaredBuyerRate || statement.declaredSellerRate,
  );
  return <section className="card spot-fee-fx-statement" aria-label={SURFACE_LABEL}>
    <h3>{SURFACE_LABEL}</h3>
    <p className={qualified ? 'muted' : 'error-text'} role={qualified ? 'status' : 'alert'}>
      <strong>{qualified
        ? 'The owning fee statement and every genuinely-required FX route are current and complete for their declared purposes; this carries no execution authority by itself.'
        : `${blocker ? human(blocker) : 'No fee or required-FX statement available'}: ${statement.reason}`}</strong>
    </p>
    <p>Saved assessment. Reading this history never renews the statement, re-reads a venue or restores consent. Arm, consent, dispatch, immediate authenticated preflight, private-stream readiness, funding and exact CANCEL/reconciliation remain separate checks.</p>
    {wallet && <>
      <h4>BASE / QUOTE / BNB wallet coverage</h4>
      <p className={wallet.status === 'COMPLETE' ? 'muted' : 'error-text'}>
        Coverage {human(wallet.status)}. This is bound read-only evidence, not approval or execution authority.
        {wallet.reason ? ` ${human(wallet.reason)}.` : ''}
      </p>
      <dl className="data-source-details">
        <div><dt>Required assets · missing assets</dt><dd>{wallet.requiredAssets.join(' · ')} · {wallet.missingAssets.length ? wallet.missingAssets.join(' · ') : 'none'}</dd></div>
        <div><dt>Wallet account · Proposal</dt><dd>{wallet.accountId} · {wallet.proposalId} · {wallet.proposalHash}</dd></div>
        <div><dt>Source · rules · capacity binding</dt><dd>{wallet.sourceVersion} · {declared(wallet.ruleMaterialVersion)} · {wallet.capacityBindingVersion}</dd></div>
        <div><dt>Collection · account-read start / first receipt</dt><dd>{declared(wallet.collectionId)} · {declared(wallet.accountReadStartedAt)} / {declared(wallet.firstReceipt)}</dd></div>
        <div><dt>Original account-read digest</dt><dd>{declared(wallet.originalDigest)}</dd></div>
        {wallet.balances.map(row => <div key={row.asset}>
          <dt>{row.asset} free / locked</dt>
          <dd>{row.free} / {row.locked} {row.asset}. Free already excludes broker locks; local claims {row.localClaims}; effective free {declared(row.effectiveFree)} {row.asset}.</dd>
        </div>)}
      </dl>
    </>}
    {model && <>
      <h4>Current received-asset fee scenario</h4>
      <p><strong>ACTUAL_FEE_ASSET_PENDING.</strong> This current model estimates a full fill from the recorded intent and complete symbol commission terms; it does not guarantee the future fee asset or amount. A complete, account-bound estimate may qualify review and Prepare only when every independent source, wallet, market, risk and permission guard passes. Settlement uses the broker-reported fee asset and debit; deviations freeze for reconciliation. A SELL limit models proceeds at its minimum price, not the largest possible proceeds or fee.</p>
      <p>Fee calculations use the full modeled receipt before display rounding. The displayed gross and net receipt are rounded down to18 decimal places; each final fee family is rounded up separately.</p>
      {model.discountedFeeIsExactlyZero && <p>Known zero discounted expression: the complete recorded discounted fee expression is exactly zero. No paid asset, BNB wallet amount or conversion is invented. The undiscounted received-asset fallback below remains a separate conditional scenario, not a claim about the current charge or a future fee guarantee.</p>}
      <dl className="data-source-details">
        <div><dt>Model method · class</dt><dd>{model.method} · {model.class}</dd></div>
        <div><dt>Modeled gross receipt</dt><dd>{model.grossReceived} {model.receivedAsset}</dd></div>
        <div><dt>Complete fallback role</dt><dd>{model.fallback.role}. Selected by the full rounded three-family total, not independent family maxima.</dd></div>
        <div><dt>Modeled standard fee</dt><dd>{model.fallback.standard} {model.receivedAsset}</dd></div>
        <div><dt>Modeled tax fee</dt><dd>{model.fallback.tax} {model.receivedAsset}</dd></div>
        <div><dt>Modeled special fee</dt><dd>{model.fallback.special} {model.receivedAsset}</dd></div>
        <div><dt>Modeled received-asset fee total</dt><dd>{model.fallback.total} {model.receivedAsset}</dd></div>
        <div><dt>Modeled net receipt</dt><dd>{model.fallback.netReceived} {model.receivedAsset}</dd></div>
        <div><dt>Current charging condition</dt><dd>{model.chargingCondition}. This is a current condition, not a promised future charging asset.</dd></div>
        <div><dt>Model limitations</dt><dd>{model.limitations.join(' · ')}</dd></div>
      </dl>
      {model.fundingPlanPreview && <>
        <h4>Current funding plan preview</h4>
        <p>This backend-derived preview is captured with the reviewed financial material. It is not approval or a future fee guarantee; native claims are created only by atomic Prepare after explicit approval.</p>
        <dl className="data-source-details">
          <div><dt>Native principal claim</dt><dd>{model.fundingPlanPreview.principalClaim.amount} {model.fundingPlanPreview.principalClaim.asset}</dd></div>
          <div><dt>Current fee disposition</dt><dd>{human(model.fundingPlanPreview.feeDisposition)}{model.fundingPlanPreview.currentFeeClaim
            ? ` · ${model.fundingPlanPreview.currentFeeClaim.amount} ${model.fundingPlanPreview.currentFeeClaim.asset} outgoing fee claim`
            : ''}</dd></div>
          <div><dt>Received-asset fallback fee and net receipt</dt><dd>{model.fundingPlanPreview.fallbackFeeAmount} {model.fundingPlanPreview.fallbackFeeAsset} · {model.fundingPlanPreview.fallbackNetReceipt} {model.fundingPlanPreview.fallbackFeeAsset}</dd></div>
        </dl>
      </>}
      {model.marketReference && <>
        <h4>Current market scenario reference</h4>
        <p>The MARKET scenario uses the recorded {model.marketReference.priceSide} side and TAKER families. It is a current modeled receipt, not an actual fill prediction or a future proceeds or fee bound. Reading this capture does not renew the original observation.</p>
        <dl className="data-source-details">
          <div><dt>Price side · data-use rights</dt><dd>{model.marketReference.priceSide} · {model.marketReference.dataUseRights}</dd></div>
          <div><dt>Source · snapshot</dt><dd>{model.marketReference.source} · {model.marketReference.marketSnapshotId}</dd></div>
          <div><dt>Original provider time · first receipt</dt><dd>{model.marketReference.providerTimestamp} · {model.marketReference.firstReceipt}</dd></div>
          <div><dt>Source version · connection generation</dt><dd>{model.marketReference.sourceVersion} · {model.marketReference.connectionGeneration}</dd></div>
          <div><dt>Book update · material hash</dt><dd>{model.marketReference.bookUpdateId} · {model.marketReference.materialHash}</dd></div>
        </dl>
      </>}
      {model.bnbFundingEvidence && !wallet && <dl className="data-source-details">
        <div><dt>Observed BNB free / locked</dt><dd>{model.bnbFundingEvidence.free} / {model.bnbFundingEvidence.locked} {model.bnbFundingEvidence.asset}. Free already excludes provider locks.</dd></div>
        <div><dt>Known local BNB claims</dt><dd>{model.bnbFundingEvidence.localClaims} {model.bnbFundingEvidence.asset}</dd></div>
        <div><dt>Effective available BNB</dt><dd>{model.bnbFundingEvidence.effectiveFree} {model.bnbFundingEvidence.asset}. Exact free minus retained local claims; locked is not deducted again.</dd></div>
        <div><dt>Original account-read start / first receipt</dt><dd>{model.bnbFundingEvidence.startedAt} / {model.bnbFundingEvidence.firstReceipt}</dd></div>
        <div><dt>Funding collection / binding</dt><dd>{model.bnbFundingEvidence.collectionId} · {model.bnbFundingEvidence.capacityBindingVersion}</dd></div>
        <div><dt>Original account-read digest</dt><dd>{model.bnbFundingEvidence.originalDigest}</dd></div>
      </dl>}
      {model.bnbFee && <>
        <h4>Conditional discounted BNB current estimate</h4>
        <p>This is a current estimate under the displayed complete role and exact price legs. It does not prove Binance’s internal average conversion rate or guarantee the future charging asset. Current branch: {human(model.chargingCondition)}.</p>
        <dl className="data-source-details">
          <div><dt>Selected discounted role</dt><dd>{model.bnbFee.role}</dd></div>
          <div><dt>Discounted standard fee</dt><dd>{model.bnbFee.standard} BNB</dd></div>
          <div><dt>Undiscounted tax fee</dt><dd>{model.bnbFee.tax} BNB</dd></div>
          <div><dt>Undiscounted special fee</dt><dd>{model.bnbFee.special} BNB</dd></div>
          <div><dt>Complete modeled BNB fee</dt><dd>{model.bnbFee.total} BNB</dd></div>
          <div><dt>Complete discounted role candidates</dt><dd>{(model.bnbFeeCandidates ?? []).map(candidate => `${candidate.role} ${candidate.total} BNB`).join(' · ')}</dd></div>
        </dl>
        <h4>Consumed BNB conversion legs</h4>
        <ul>{(model.bnbConversionReferences ?? []).map(reference => <li key={`${reference.pair}-${reference.fromAsset}-${reference.toAsset}`}>{bnbReferenceLine(reference)}</li>)}</ul>
      </>}
      {model.bnbFeeFailure && <p>Current discounted BNB estimate unavailable: {human(model.bnbFeeFailure)}. The received-asset scenario remains visible; incomplete price or exact depth coverage cannot be treated as sufficient.</p>}
    </>}
    {statement.currentFeeModelFailure && <p>Current fee scenario unavailable: {statement.currentFeeModelFailure}. Missing inputs are not replaced by an older amount.</p>}
    <dl className="data-source-details">
      <div><dt>Reason code</dt><dd>{statement.reasonCode}</dd></div>
      <div><dt>Binding blocker</dt><dd>{blocker ?? 'None'}</dd></div>
      <div><dt>Fee currency · declared origin</dt><dd>{statement.feeCurrency} · {statement.feeCurrencyOrigin}. This is an independent fact, reported even when the binding blocker above is the required execution route.</dd></div>
      {hasHistoricalAccountComparison && <>
        <div><dt>Historical account-level commission comparison</dt><dd>Basis {declared(statement.feeRateBasis)} · maker/taker {declared(statement.declaredMakerRate)} / {declared(statement.declaredTakerRate)} · buyer/seller {declared(statement.declaredBuyerRate)} / {declared(statement.declaredSellerRate)}. Retained only for older captures; current fee scenarios use the separately bound exact-symbol commission declaration.</dd></div>
      </>}
      <div><dt>Expected fee · declared origin {statement.feeCurrencyOrigin}</dt><dd>{declared(statement.expectedFee)}. Never fabricated from an unknown fee currency.</dd></div>
      <div><dt>Intent notional · declared origin INTENT_PROPOSAL</dt><dd>{declared(statement.expectedSpendQuote)} {statement.quoteAsset} · maximum authorized {declared(statement.maximumAuthorizedSpendQuote)} {statement.quoteAsset}</dd></div>
      <div><dt>Workspace base currency · declared origin WORKSPACE_BASE_CURRENCY</dt><dd>{statement.baseCurrency}. The workspace base is a three-letter ISO code, so USDT is never read as USD: USDT ≠ USD.</dd></div>
      <div><dt>Workspace-base expected spend / fee · declared origin REQUIRED_FX</dt><dd>{declared(statement.workspaceBaseExpectedSpend)} / {declared(statement.workspaceBaseExpectedFee)}. The exact qualified policy valuation supplies the Proposal amount used by owning monetary risk checks; when unavailable, owning Live Spot risk never substitutes historical portfolio FX. This price-only estimate adds no execution cost.</dd></div>
      <div><dt>Exact account · instrument</dt><dd>{statement.accountId} · {statement.instrumentId} · BASE {statement.baseAsset} / QUOTE {statement.quoteAsset}</dd></div>
      <div><dt>Proposal · hash</dt><dd>{statement.proposalId} · {statement.proposalHash}</dd></div>
    </dl>
    <h4>Genuinely-required FX routes</h4>
    <ul>{statement.routes.length === 0
      ? <li>No genuinely-required FX conversion applies to this intent.</li>
      : statement.routes.map(route => <li key={`${route.purpose}-${route.fromCurrency}-${route.toCurrency}`}>{routeLine(route)}</li>)}</ul>
    <h4>Standing limitations carried from the consumed read-only inputs</h4>
    <ul>{statement.carriedLimitations.length === 0
      ? <li>No standing limitation reported with the underlying observations.</li>
      : statement.carriedLimitations.map(limitation => <li key={limitation}>{human(limitation)}</li>)}</ul>
    <p>These describe how the evidence was obtained, not what it failed to cover. Separately read account, order and balance responses are not one atomic provider snapshot. Nothing here is a fill promise, a fee promise or an approval.</p>
    <dl className="data-source-details">
      <div><dt>Exact-symbol fee material · required FX material</dt><dd>{statement.feeEvidenceVersion} · {statement.routeEvidenceVersion}</dd></div>
    </dl>
  </section>;
}
