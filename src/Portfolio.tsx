import { useQuery } from '@tanstack/react-query';
import type { FxProvenance, PortfolioSnapshot, PortfolioValue } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';

const statusLabel: Record<PortfolioSnapshot['status'], string> = {
  AVAILABLE: 'Available',
  DEGRADED: 'Degraded',
  UNAVAILABLE: 'Unavailable',
  BLOCKED_EXTERNAL: 'Blocked by external setup',
};

function valueText(value: PortfolioValue) {
  if (value.workspaceValue != null) return `${value.workspaceValue} ${value.workspaceCurrency}`;
  if (value.nativeValue != null) return `${value.nativeValue} ${value.nativeCurrency ?? 'native'} · unavailable in ${value.workspaceCurrency}`;
  return `Unavailable in ${value.workspaceCurrency}`;
}

function provenance(route: FxProvenance) {
  return <li key={`${route.sourceId}:${route.pairPath}`} className="portfolio-provenance-row">
    <div><strong>{route.pairPath}</strong><small>{route.sourceId} · {route.quality} · {route.freshness}</small></div>
    <dl><div><dt>Rate</dt><dd>{route.rate ?? 'Unavailable'}</dd></div><div><dt>Provider timestamp</dt><dd>{route.providerTimestamp ?? 'Unavailable'}</dd></div><div><dt>TradeX received</dt><dd>{route.receivedTimestamp}</dd></div></dl>
    {route.depegWarning && <p className="notice" role="alert">{route.depegWarning}</p>}
  </li>;
}

function PortfolioTables({ snapshot }: { snapshot: PortfolioSnapshot }) {
  return <>
    <section className="card portfolio-section" aria-labelledby="portfolio-accounts-title">
      <div className="section-heading"><div><p className="eyebrow">Accounts</p><h2 id="portfolio-accounts-title">Cross-account view</h2></div><span className="badge">{snapshot.accounts.length} accounts</span></div>
      {snapshot.accounts.length ? <div className="table-scroll" tabIndex={0} aria-label="Portfolio accounts"><table><thead><tr><th>Account</th><th>Provider / environment</th><th>Currency</th><th>Equity</th><th>Cash</th><th>Positions</th><th>Open orders</th></tr></thead><tbody>{snapshot.accounts.map(account => <tr key={account.connectionId}><td><strong>{account.label}</strong><small className="identity">{account.connectionId}</small></td><td>{account.providerId} · {account.environment}</td><td>{account.accountCurrency ?? 'Unavailable'}</td><td>{valueText(account.equity)}</td><td>{valueText(account.cash)}</td><td>{account.positionsCount}</td><td>{account.openOrdersCount}</td></tr>)}</tbody></table></div> : <p className="muted">No account observations are available.</p>}
    </section>
    <section className="card portfolio-section" aria-labelledby="portfolio-holdings-title">
      <div className="section-heading"><div><p className="eyebrow">Holdings</p><h2 id="portfolio-holdings-title">Positions and balances</h2></div><span className="badge">{snapshot.holdings.length} holdings</span></div>
      {snapshot.holdings.length ? <div className="table-scroll" tabIndex={0} aria-label="Portfolio holdings"><table><thead><tr><th>Asset</th><th>Account</th><th>Quantity</th><th>Value</th><th>Unrealized P&amp;L</th></tr></thead><tbody>{snapshot.holdings.map((holding, index) => <tr key={`${holding.connectionId}:${holding.asset}:${index}`}><td><strong>{holding.asset}</strong><small className="identity">{holding.instrumentId ?? 'No canonical instrument'}</small></td><td>{holding.accountLabel}</td><td>{holding.quantity ?? 'Unavailable'}</td><td>{valueText(holding.value)}</td><td>{holding.unrealizedPnl ? valueText(holding.unrealizedPnl) : 'Unavailable'}</td></tr>)}</tbody></table></div> : <p className="muted">No positions or balances are available.</p>}
    </section>
    <section className="card portfolio-section" aria-labelledby="portfolio-orders-title">
      <div className="section-heading"><div><p className="eyebrow">Orders and fills</p><h2 id="portfolio-orders-title">Open orders</h2></div><span className="badge">{snapshot.openOrders.length} open</span></div>
      {snapshot.openOrders.length ? <div className="table-scroll" tabIndex={0} aria-label="Portfolio open orders"><table><thead><tr><th>Asset</th><th>Account</th><th>Side</th><th>Quantity / notional</th><th>Broker state</th></tr></thead><tbody>{snapshot.openOrders.map(order => <tr key={order.brokerOrderId}><td>{order.asset}<small className="identity">{order.brokerOrderId}</small></td><td>{order.accountLabel}</td><td>{order.side}</td><td>{order.quantity ?? order.notional ?? 'Unavailable'}{order.currency ? ` ${order.currency}` : ''}</td><td>{order.status}</td></tr>)}</tbody></table></div> : <p className="muted">No open orders are available.</p>}
      <h3>Fills</h3>{snapshot.fills?.length ? <div className="table-scroll" tabIndex={0} aria-label="Portfolio fills"><table><thead><tr><th>Asset</th><th>Account</th><th>Quantity</th><th>Value</th><th>Observed</th></tr></thead><tbody>{snapshot.fills.map(fill => <tr key={fill.fillId}><td>{fill.asset}</td><td>{fill.accountLabel}</td><td>{fill.quantity}</td><td>{valueText(fill.value)}</td><td>{fill.observedAt}</td></tr>)}</tbody></table></div> : <p className="muted">Fills are unavailable from the connected providers.</p>}
    </section>
  </>;
}

export function Portfolio({ workspaceId }: { workspaceId: string }) {
  const query = useQuery({ queryKey: ['portfolio', workspaceId], queryFn: () => request('portfolio.get', { workspaceId }), refetchOnMount: 'always', refetchOnReconnect: 'always' });
  if (query.isPending) return <section className="card portfolio-section" aria-live="polite"><p role="status">Loading portfolio…</p></section>;
  if (query.isError) return <section className="card portfolio-section error-banner" role="alert"><div><strong>Portfolio needs attention</strong><p>{explainError(query.error)}</p></div><button type="button" onClick={() => void query.refetch()}>Reload portfolio</button></section>;
  const snapshot = query.data;
  return <section className="portfolio-panel" aria-labelledby="portfolio-title">
    <div className={`card portfolio-summary portfolio-status-${snapshot.status.toLowerCase()}`} role="status" aria-live="polite">
      <div className="section-heading"><div><p className="eyebrow">Portfolio</p><h2 id="portfolio-title">Workspace valuation</h2><p>{snapshot.availabilityReason}</p></div><span className="badge">{statusLabel[snapshot.status]}</span></div>
      <dl className="portfolio-totals"><div><dt>Workspace equity</dt><dd>{valueText(snapshot.totals.equity)}</dd></div><div><dt>Cash</dt><dd>{valueText(snapshot.totals.cash)}</dd></div><div><dt>Exposure</dt><dd>{valueText(snapshot.totals.exposure)}</dd></div><div><dt>Unrealized P&amp;L</dt><dd>{valueText(snapshot.totals.unrealizedPnl)}</dd></div><div><dt>Realized P&amp;L</dt><dd>{valueText(snapshot.totals.realizedPnl)}</dd></div></dl>
      <p className="muted">Base currency: <strong>{snapshot.baseCurrency}</strong> · Observed {snapshot.observedAt}</p>
      <p className="notice">Live risk: {snapshot.liveRisk.eligible ? 'Eligible' : 'Blocked'} — {snapshot.liveRisk.reason}</p>
    </div>
    <section className="card portfolio-section" aria-labelledby="portfolio-fx-title"><div className="section-heading"><div><p className="eyebrow">Provenance</p><h2 id="portfolio-fx-title">FX and stablecoin routes</h2></div><span className="badge">{snapshot.fxRoutes.length} routes</span></div>{snapshot.fxRoutes.length ? <ul className="portfolio-provenance">{snapshot.fxRoutes.map(provenance)}</ul> : <p className="muted">No conversion routes were required or observed.</p>}</section>
    <PortfolioTables snapshot={snapshot} />
  </section>;
}
