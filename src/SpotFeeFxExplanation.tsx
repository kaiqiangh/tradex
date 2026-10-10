import type { SpotFeeFxStatement, SpotRequiredRoute } from '../shared/ipc-types.ts';

const human = (value: string) => value.toLowerCase().replaceAll('_', ' ');
const declared = (value: string | null | undefined) => value ?? 'Not declared';

// The exact literal is shared by the accessible name and the visible heading so neither can drift
// from the other.
const SURFACE_LABEL = 'Captured Proposal Spot fee and required execution FX';

// One genuinely-required execution conversion, rendered on a single line so its state and evidence
// can never be read apart from the currency pair they qualify.
const routeLine = (route: SpotRequiredRoute) =>
  `${human(route.purpose)} · ${route.fromCurrency} -> ${route.toCurrency} · ${human(route.state)} · provider pair ${route.providerPair ?? 'no supported pair'} · rate quality ${declared(route.providerQuality)} · provider time ${declared(route.providerTimestamp)} · first receipt ${declared(route.firstReceipt)} · declared cost ${route.conservativeCost ?? 'no declared cost'} · ${route.reason}`;

// One exact statement of the reviewed intent's owning fee and genuinely-required execution FX.
// It reports two independent facts that must both remain readable: the single binding blocker and
// the fee currency. The owning statement only ever exists as part of a saved RiskDecision, so this
// is always a saved assessment: it has no live re-read, no refresh control and no polling.
export function SpotFeeFxExplanation({ statement }: { statement: SpotFeeFxStatement }) {
  const qualified = statement.outcome === 'PASS';
  const blocker = statement.bindingBlocker;
  return <section className="card spot-fee-fx-statement" aria-label={SURFACE_LABEL}>
    <h3>{SURFACE_LABEL}</h3>
    <p className={qualified ? 'muted' : 'error-text'} role={qualified ? 'status' : 'alert'}>
      <strong>{qualified
        ? 'The owning fee statement and every genuinely-required execution conversion are current and complete; this carries no execution authority by itself.'
        : `${blocker ? human(blocker) : 'No fee or required-FX statement available'}: ${statement.reason}`}</strong>
    </p>
    <p>Saved assessment. Reading this history never renews the statement, re-reads a venue or restores consent. Arm, consent, dispatch, immediate authenticated preflight, private-stream readiness, funding and exact CANCEL/reconciliation remain separate checks.</p>
    <dl className="data-source-details">
      <div><dt>Reason code</dt><dd>{statement.reasonCode}</dd></div>
      <div><dt>Binding blocker</dt><dd>{blocker ?? 'None'}</dd></div>
      <div><dt>Fee currency · declared origin</dt><dd>{statement.feeCurrency} · {statement.feeCurrencyOrigin}. This is an independent fact, reported even when the binding blocker above is the required execution route.</dd></div>
      <div><dt>Fee rate basis · declared origin ACCOUNT_DECLARED_COMMISSION_RATES</dt><dd>{declared(statement.feeRateBasis)}. The larger declared maker/taker comparison basis, not a complete fee estimate. The account rates do not establish the complete symbol/side commission or fee-charging asset.</dd></div>
      <div><dt>Declared maker / taker rate · origin ACCOUNT_DECLARED_COMMISSION_RATES</dt><dd>{declared(statement.declaredMakerRate)} / {declared(statement.declaredTakerRate)}. Reported verbatim, never promoted into a hidden gate.</dd></div>
      <div><dt>Declared buyer / seller rate · origin ACCOUNT_DECLARED_COMMISSION_RATES</dt><dd>{declared(statement.declaredBuyerRate)} / {declared(statement.declaredSellerRate)}</dd></div>
      <div><dt>Expected fee · declared origin {statement.feeCurrencyOrigin}</dt><dd>{declared(statement.expectedFee)}. Never fabricated from an unknown fee currency.</dd></div>
      <div><dt>Intent notional · declared origin INTENT_PROPOSAL</dt><dd>{declared(statement.expectedSpendQuote)} {statement.quoteAsset} · maximum authorized {declared(statement.maximumAuthorizedSpendQuote)} {statement.quoteAsset}</dd></div>
      <div><dt>Workspace base currency · declared origin WORKSPACE_BASE_CURRENCY</dt><dd>{statement.baseCurrency}. The workspace base is a three-letter ISO code, so USDT is never read as USD: USDT ≠ USD.</dd></div>
      <div><dt>Workspace-base expected spend / fee · declared origin REQUIRED_EXECUTION_FX</dt><dd>{declared(statement.workspaceBaseExpectedSpend)} / {declared(statement.workspaceBaseExpectedFee)}. Present only when the required conversion is genuinely qualified; no conversion cost is fabricated.</dd></div>
      <div><dt>Exact account · instrument</dt><dd>{statement.accountId} · {statement.instrumentId} · BASE {statement.baseAsset} / QUOTE {statement.quoteAsset}</dd></div>
      <div><dt>Proposal · hash</dt><dd>{statement.proposalId} · {statement.proposalHash}</dd></div>
    </dl>
    <h4>Genuinely-required execution FX routes</h4>
    <ul>{statement.routes.length === 0
      ? <li>No genuinely-required execution conversion applies to this intent.</li>
      : statement.routes.map(route => <li key={`${route.purpose}-${route.fromCurrency}-${route.toCurrency}`}>{routeLine(route)}</li>)}</ul>
    <h4>Standing limitations carried from the consumed read-only inputs</h4>
    <ul>{statement.carriedLimitations.length === 0
      ? <li>No standing limitation reported with the underlying observations.</li>
      : statement.carriedLimitations.map(limitation => <li key={limitation}>{human(limitation)}</li>)}</ul>
    <p>These describe how the evidence was obtained, not what it failed to cover. Separately read account, order and balance responses are not one atomic provider snapshot. Nothing here is a fill promise, a fee promise or an approval.</p>
    <dl className="data-source-details">
      <div><dt>Bound evidence versions</dt><dd>{statement.feeEvidenceVersion} · {statement.routeEvidenceVersion}</dd></div>
    </dl>
  </section>;
}
