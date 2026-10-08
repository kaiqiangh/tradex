import type { MarketDetail } from '../shared/ipc-types.ts';

export function SpotQuoteEvidencePanel({ detail }: { detail: MarketDetail }) {
  const snapshot = detail.snapshot;
  const evidence = snapshot?.provenance.binance;
  if (!snapshot || !evidence) return null;
  const current = detail.status === 'AVAILABLE' && snapshot.provenance.freshness === 'HEALTHY';
  return <section className="spot-depth-evidence" aria-label="Binance Spot depth evidence">
    <h3>Binance Spot depth evidence</h3>
    <p role="status"><strong>{current ? 'Current continuous quote' : 'Retained quote — not current'}</strong></p>
    <p>Known price bands only. Displayed levels do not establish whole-book liquidity or a deeper executable fill.</p>
    <dl className="market-provenance">
      <div><dt>Venue / provider symbol</dt><dd>BINANCE · {evidence.providerSymbol}</dd></div>
      <div><dt>Depth unit</dt><dd>{evidence.baseAsset} BASE · prices in {evidence.quoteAsset}</dd></div>
      <div><dt>Known bid levels / floor</dt><dd>{evidence.knownBidLevels} · {evidence.bidKnownFloor}</dd></div>
      <div><dt>Known ask levels / ceiling</dt><dd>{evidence.knownAskLevels} · {evidence.askKnownCeiling}</dd></div>
      <div><dt>Provider event time (ms)</dt><dd>{evidence.providerEventTimeMs}</dd></div>
      <div><dt>Material update ID</dt><dd>{evidence.bookUpdateId}</dd></div>
      <div><dt>First receipt</dt><dd>{snapshot.provenance.receivedTimestamp}</dd></div>
      <div><dt>Source version</dt><dd>{evidence.sourceVersion}</dd></div>
      <div><dt>Connection generation</dt><dd>{evidence.connectionGeneration}</dd></div>
      <div><dt>Lease / session</dt><dd>{evidence.leaseId} · {evidence.sessionId}</dd></div>
      <div><dt>Time generation</dt><dd>{evidence.timeGeneration}</dd></div>
      <div><dt>Material hash</dt><dd>{evidence.materialHash}</dd></div>
      <div><dt>Financial data use</dt><dd>{evidence.dataUseRights}</dd></div>
    </dl>
    <p className="muted">Technical collection does not verify financial use, permitted retention, redistribution, commercial use or regional eligibility. No last trade, reference price or USD / USDT equivalence is supplied.</p>
    <div className="spot-depth-grid">{([['bid', evidence.bids], ['ask', evidence.asks]] as const).map(([side, levels]) => <table key={side} aria-label={`Known Binance ${side} depth`}>
      <caption>Displayed {side} depth · {levels.length} of {side === 'bid' ? evidence.knownBidLevels : evidence.knownAskLevels} known levels</caption>
      <thead><tr><th scope="col">Price ({evidence.quoteAsset})</th><th scope="col">Quantity ({evidence.baseAsset} BASE)</th></tr></thead>
      <tbody>{levels.map(level => <tr key={level.price}><td>{level.price}</td><td>{level.quantity}</td></tr>)}</tbody>
    </table>)}</div>
  </section>;
}
