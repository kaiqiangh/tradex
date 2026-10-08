import type { OrderProposal } from '../shared/ipc-types.ts';
import { explainError } from './client.ts';
import { useOwnedHotQuote } from './useOwnedHotQuote.ts';
import { SpotQuoteEvidencePanel } from './SpotQuoteEvidencePanel.tsx';

export function TradeHotQuote({ proposal }: { proposal: OrderProposal }) {
  const fields = proposal.fields;
  const supported = fields.environment === 'BINANCE_LIVE' && fields.venue === 'BINANCE'
    || fields.instrumentId.startsWith('equity:') && ['ALPACA_PAPER', 'TRADING212_DEMO', 'TRADING212_LIVE'].includes(fields.environment);
  const owned = useOwnedHotQuote(proposal.workspaceId, fields.instrumentId, supported);
  if (!supported) return null;
  const detail = owned.hot?.detail;
  const snapshot = detail?.snapshot;
  return <section className="card trade-hot-quote hot-quote-status" aria-label="Trade Hot quote">
    <h3>Current market view for this Proposal</h3>
    <p className="identity">{proposal.proposalId} · {fields.instrumentId} · {fields.venue}</p>
    <strong>{owned.hot?.status ?? (owned.configured ? owned.visible ? 'CONNECTING' : 'PAUSED' : 'UNAVAILABLE')}</strong>
    <p>{owned.hot?.reason ?? (owned.configured ? owned.visible ? 'Acquiring this visible Proposal’s own quote lease.' : 'This hidden Proposal view has released its quote lease.' : 'Save an applicable market source in Settings Data & Storage. An account connection does not configure market data.')}</p>
    <p>No execution authority is granted by this market view. It does not refresh the immutable Proposal’s saved inputs, arm an account, approve or send an order. Captured reviews remain separate.</p>
    {owned.configured && <p className="muted">{owned.publicSpot ? 'Public Spot stream · Authentication not used' : `Feed ${owned.feed?.toUpperCase()} · Authentication ${owned.hot?.authenticated ? 'confirmed' : 'pending'}`} · Continuous quote {owned.hot?.subscribed ? 'confirmed' : 'pending'}.</p>}
    {Boolean(owned.error) && <p role="alert">{explainError(owned.error)}</p>}
    {owned.hot && <p className="identity">Lease {owned.hot.leaseId} · Connection generation {owned.hot.connectionGeneration} · Sequence {owned.hot.sequence}</p>}
    {owned.hot && owned.hot.reconnectAttempt > 0 && <p>Automatic reconnect {owned.hot.reconnectAttempt} of 3.</p>}
    {(owned.error || owned.hot && ['CLOSED', 'FAILED', 'STALE'].includes(owned.hot.status)) && <button type="button" onClick={owned.retry}>Retry Proposal quote source</button>}
    {detail && <p role="status">{detail.status} · {detail.availabilityReason}</p>}
    {snapshot && <dl className="market-provenance">
      <div><dt>Bid / ask</dt><dd>{snapshot.bid ?? 'Unavailable'} / {snapshot.ask ?? 'Unavailable'}</dd></div>
      <div><dt>Bid / ask size (BASE)</dt><dd>{snapshot.bidSize ?? 'Unavailable'} / {snapshot.askSize ?? 'Unavailable'}</dd></div>
      <div><dt>Provider time / first receipt</dt><dd>{snapshot.provenance.providerTimestamp} / {snapshot.provenance.receivedTimestamp}</dd></div>
      <div><dt>Source / snapshot</dt><dd>{snapshot.provenance.source} / {snapshot.provenance.marketSnapshotId}</dd></div>
    </dl>}
    {detail && <SpotQuoteEvidencePanel detail={detail} />}
  </section>;
}
