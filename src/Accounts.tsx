import { useEffect, useRef, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { AccountConnection, LocalPaperState } from '../shared/ipc-types.ts';
import { CommandError, desktop, browserIntegration, explainError, request } from './client.ts';
import { fromAccountSnapshot } from './projection.ts';
import { useDomainProjection } from './useDomainProjection.ts';
import { Portfolio } from './Portfolio.tsx';

function mutation(a: AccountConnection) { return { workspaceId: a.workspaceId, connectionId: a.connectionId, expectedStateVersion: a.stateVersion }; }
function money(value: string | null | undefined, currency?: string | null) { return value == null ? 'Unavailable' : `${value}${currency ? ` ${currency}` : ' (currency unavailable)'}`; }
function time(value: string | null | undefined) { return value ? new Date(value).toLocaleString() : 'Not yet'; }

function LocalPaperSummary({ state }: { state: LocalPaperState }) {
  const queryClient = useQueryClient();
  const [scenario, setScenario] = useState(state.profile.scenarioId);
  const [busyOrder, setBusyOrder] = useState<string>();
  const [confirmOrderId, setConfirmOrderId] = useState<string>();
  const [quoteBusy, setQuoteBusy] = useState(false);
  const [notice, setNotice] = useState('');
  const confirmationRef = useRef<HTMLDivElement>(null);
  const confirmationTriggerRef = useRef<HTMLElement | null>(null);
  useEffect(() => setScenario(state.profile.scenarioId), [state.profile.scenarioId]);
  useEffect(() => {
    if (confirmOrderId) {
      queueMicrotask(() => confirmationRef.current?.querySelector<HTMLElement>('button:not(:disabled)')?.focus());
    } else if (confirmationTriggerRef.current) {
      if (confirmationTriggerRef.current.isConnected) confirmationTriggerRef.current.focus();
      else document.getElementById('local-paper-summary-title')?.focus();
      confirmationTriggerRef.current = null;
    }
  }, [confirmOrderId]);
  const applyScenario = async () => {
    try {
      const next = await request('paper.scenario.set', {
        workspaceId: state.workspaceId,
        expectedStateVersion: state.stateVersion,
        profile: { ...state.profile, scenarioId: scenario },
      });
      await queryClient.invalidateQueries({ queryKey: ['paper', state.workspaceId] });
      setNotice(`Scenario ${next.profile.scenarioId} is active.`);
    } catch (failure) { setNotice(explainError(failure)); }
  };
  const cancel = async (orderId: string) => {
    setBusyOrder(orderId); setNotice('');
    try {
      const result = await request('paper.order.cancel', {
        workspaceId: state.workspaceId,
        orderId,
        expectedStateVersion: state.stateVersion,
        idempotencyKey: crypto.randomUUID(),
      });
      await queryClient.invalidateQueries({ queryKey: ['paper', state.workspaceId] });
      setNotice(`Order ${result.order.orderId} is ${result.order.state}.`);
    } catch (failure) { setNotice(explainError(failure)); }
    finally { setBusyOrder(undefined); }
  };
  const requestCancel = (orderId: string) => {
    confirmationTriggerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setConfirmOrderId(orderId);
  };
  const confirmCancel = async () => {
    if (!confirmOrderId) return;
    await cancel(confirmOrderId);
    setConfirmOrderId(undefined);
  };
  const refreshQuote = async () => {
    setQuoteBusy(true); setNotice('');
    try {
      const current = await queryClient.fetchQuery({
        queryKey: ['paper', state.workspaceId],
        queryFn: () => request('paper.get', { workspaceId: state.workspaceId }),
        staleTime: 0,
      });
      const next = await request('paper.quote.refresh', { workspaceId: state.workspaceId, expectedStateVersion: current.stateVersion });
      await queryClient.invalidateQueries({ queryKey: ['paper', state.workspaceId] });
      setNotice(`Simulation quote refreshed at ${time(next.profile.quoteObservedAt)}.`);
    } catch (failure) { setNotice(explainError(failure)); }
    finally { setQuoteBusy(false); }
  };
  return <section className="card local-paper-summary" aria-labelledby="local-paper-summary-title">
    <div className="section-heading"><div><p className="eyebrow">Local Paper · LOCAL_PAPER</p><h2 id="local-paper-summary-title" tabIndex={-1}>TradeX simulation · TRADEX_SIMULATION</h2><p>{state.disclosure}</p></div><span className="badge">LOCAL PAPER</span></div>
    <dl className="health-grid"><div><dt>Scenario</dt><dd>{state.profile.scenarioId}</dd></div><div><dt>Engine</dt><dd>{state.profile.engineVersion}</dd></div><div><dt>Starting cash</dt><dd>{state.profile.startingCash} {state.profile.baseCurrency}</dd></div><div><dt>Cash</dt><dd>{state.cash.value} {state.cash.currency}</dd></div><div><dt>Reserved</dt><dd>{state.reservedCash.value} {state.reservedCash.currency}</dd></div><div><dt>Positions</dt><dd>{state.positions.length}</dd></div><div><dt>Open orders</dt><dd>{state.openOrders.length}</dd></div><div><dt>Fills</dt><dd>{state.fills.length}</dd></div><div><dt>Quote observed</dt><dd>{time(state.profile.quoteObservedAt)}</dd></div></dl>
    <div className="form-actions"><label className="field">Simulation scenario<select aria-label="Simulation scenario" value={scenario} onChange={event => setScenario(event.target.value)}><option value="default-v1">Default full fill</option><option value="partial-v1">Partial fill</option><option value="resting-v1">Resting limit</option><option value="rejected-v1">Rejected</option></select></label><button type="button" onClick={() => void applyScenario()} disabled={scenario === state.profile.scenarioId}>Apply scenario</button><button type="button" onClick={() => void refreshQuote()} disabled={quoteBusy || state.openOrders.length > 0}>{quoteBusy ? 'Refreshing quote…' : 'Refresh simulation quote'}</button></div>
    {notice && <p role="status">{notice}</p>}
    <section aria-label="Local Paper order history"><h3>Local Paper order history</h3>{(state.orders ?? []).length ? <ul>{(state.orders ?? []).map(order => { const fills = state.fills.filter(fill => fill.orderId === order.orderId); const events = (state.events ?? []).filter(event => event.orderId === order.orderId); return <li key={order.orderId}><strong>{order.state}</strong> · {order.instrumentId} · {order.filledQuantity} filled · {order.remainingQuantity} remaining{['ACCEPTED', 'PARTIALLY_FILLED'].includes(order.state) && <button type="button" onClick={() => requestCancel(order.orderId)} disabled={busyOrder === order.orderId}>{busyOrder === order.orderId ? 'Cancelling…' : 'Cancel'}</button>}<details><summary>View fills and events</summary><p>Fills</p>{fills.length ? <ul>{fills.map(fill => <li key={fill.fillId}>{fill.quantity} @ {fill.price} {fill.currency} · {time(fill.observedAt)}</li>)}</ul> : <p>No fills.</p>}<p>Events</p>{events.length ? <ol>{events.map(event => <li key={event.eventId}>{event.kind} · {time(event.occurredAt)} · sequence {event.sequence}</li>)}</ol> : <p>No events.</p>}</details></li>; })}</ul> : <p>No Local Paper orders yet.</p>}</section>
    {confirmOrderId && <div className="picker-backdrop"><div className="picker-dialog" role="dialog" aria-modal="true" aria-labelledby="local-paper-cancel-title" ref={confirmationRef}><div className="picker-dialog-heading"><div><h2 id="local-paper-cancel-title">Confirm Local Paper cancellation</h2><p className="muted">This changes the TradeX simulation only.</p></div></div><p>Cancel the remaining quantity of this Local Paper order?</p><div className="picker-dialog-actions"><button type="button" onClick={() => setConfirmOrderId(undefined)} disabled={Boolean(busyOrder)}>Keep reviewing</button><button type="button" className="primary" onClick={() => void confirmCancel()} disabled={Boolean(busyOrder)}>{busyOrder ? 'Working…' : 'Confirm cancel'}</button></div></div></div>}
    <p className="muted">Quote source: {state.profile.quoteSource} · State {state.stateVersion}</p>
  </section>;
}

function AccountDetail({ account, busy, run }: { account: AccountConnection; busy: boolean; run: (action: () => Promise<AccountConnection>) => void }) {
  const [acknowledgedVersion, setAcknowledgedVersion] = useState<string>();
  const acknowledged = acknowledgedVersion === account.stateVersion;
  const p = account.permissions;
  const blocked = p.forbidden.length > 0 || p.unsupported.length > 0;
  const disconnected = account.connectionState === 'DISCONNECTED';
  const cleanupOnly = disconnected && account.health.credential === 'MISSING';
  const localPaper = account.providerId === 'local-paper' && account.environment === 'LOCAL';
  const disconnectDisabled = busy || (disconnected && !cleanupOnly && account.health.credential !== 'DELETE_PENDING');
  return <section className="card account-detail" aria-labelledby="account-detail-title" data-state-version={account.stateVersion}>
    <div className="account-heading"><div><h2 id="account-detail-title">{account.label}</h2><p>{account.providerId} · {account.environment}{localPaper ? ' · LOCAL_PAPER' : ''} · {account.connectionState}</p></div>
      <div className="account-actions">{!localPaper && <><button disabled={busy || disconnected || account.connectionState === 'CONNECTING' || ['MISSING', 'DELETE_PENDING'].includes(account.health.credential)} onClick={() => run(() => request('account.refresh', mutation(account)))}>Refresh account</button>
        <button disabled={disconnectDisabled} onClick={() => run(() => request('provider.disconnect', mutation(account)))}>{account.health.credential === 'DELETE_PENDING' ? 'Retry Keychain cleanup' : cleanupOnly ? 'Remove local connection' : 'Disconnect'}</button></>}</div>
    </div>
    <p className="notice">{account.health.reason}</p>
    <dl className="health-grid">{Object.entries(account.health).filter(([key]) => key !== 'reason').map(([key, value]) => <div key={key}><dt>{({ connection: 'Connection', authentication: 'Authentication', credential: 'Credential', privateStream: 'Private stream', reconciliation: 'Reconciliation', executionEligibility: 'Execution eligibility', arming: 'Arming' } as Record<string, string>)[key]}</dt><dd>{value}</dd></div>)}
      <div><dt>Last successful sync</dt><dd>{time(account.lastSuccessfulSync)}</dd></div><div><dt>Last reconciliation</dt><dd>Not yet</dd></div><div><dt>Risk policy</dt><dd>Not configured</dd></div>
    </dl>
    {!localPaper && <section className="permission-review" aria-labelledby="permission-title"><h3 id="permission-title">Permission review</h3>
      <p><strong>{blocked ? 'BLOCKED' : p.scope}</strong> · IP allow-list: {p.ipAllowListStatus}</p>
      {p.ipAllowList && p.ipAllowList.length > 0 && <p>Allowed IP addresses: {p.ipAllowList.join(', ')}</p>}
      <p>Observed access: {p.detected.join(', ') || 'No successful permission observations'}</p>
      {p.scope === 'UNVERIFIED' && <p>TradeX cannot fully inspect this API key’s permissions. Successful reads do not verify trading, withdrawal or transfer scope.</p>}
      {blocked && <p role="alert">Remove these permissions at the provider and re-test: {[...p.forbidden, ...p.unsupported].join(', ')}.</p>}
      {p.acknowledged && <p>Unverified scope was explicitly acknowledged for this permission review.</p>}
      {account.connectionState === 'REVIEW_REQUIRED' && <>
        {p.scope === 'UNVERIFIED' && <label className="check-field"><input type="checkbox" checked={acknowledged} onChange={event => setAcknowledgedVersion(event.target.checked ? account.stateVersion : undefined)} />I understand that permission scope is unverified and have checked the key’s permissions at the provider.</label>}
        <button className="primary" disabled={busy || blocked || account.health.authentication !== 'VALID' || account.health.connection !== 'ONLINE' || (p.scope === 'UNVERIFIED' && !acknowledged)} onClick={() => run(() => request('provider.connect', { step: 'confirm', ...mutation(account), acknowledgeUnverified: acknowledged }))}>Confirm connection</button>
      </>}
    </section>}
    {account.data && <><dl className="health-grid"><div><dt>Account type</dt><dd>{account.data.accountType}</dd></div><div><dt>Provider account ID</dt><dd className="identity">{account.data.remoteAccountId}</dd></div><div><dt>Currency</dt><dd>{account.data.currency ?? 'Per asset'}</dd></div>{account.providerId === 'alpaca' && <div><dt>Buying power</dt><dd>{money(account.data.buyingPower, account.data.currency)}</dd></div>}</dl>
      <h3>Balances</h3><div className="table-scroll" tabIndex={0} aria-label="Account balances"><table><thead><tr><th>Asset</th><th>Available / Free</th><th>Equity / Asset total</th><th>{account.providerId === 'bitget' ? 'Reserved / Frozen' : 'Reserved / Locked'}</th>{account.providerId === 'bitget' && <><th>Locked</th><th>Restricted available</th></>}<th>In Pies</th><th>Effective available</th></tr></thead><tbody>{account.data.balances.map(row => <tr key={row.asset}><td>{row.asset}</td><td>{row.available}</td><td>{row.total ?? 'Unavailable'}</td><td>{row.reserved ?? 'Unavailable'}</td>{account.providerId === 'bitget' && <><td>{row.locked ?? 'Unavailable'}</td><td>{row.restrictedAvailable ?? 'Unavailable'}</td></>}<td>{row.inPies ?? 'Unavailable'}</td><td>Not computed</td></tr>)}</tbody></table></div>
      <h3>Positions</h3>{account.data.positions.length ? <div className="table-scroll" tabIndex={0} aria-label="Account positions"><table><thead><tr><th>Symbol</th><th>Quantity</th><th>Market value</th><th>Average entry</th></tr></thead><tbody>{account.data.positions.map(row => <tr key={row.symbol}><td>{row.symbol}</td><td>{row.quantity}</td><td>{money(row.marketValue, row.marketValueCurrency)}</td><td>{money(row.averageEntryPrice, row.instrumentCurrency)}</td></tr>)}</tbody></table></div> : <p>No positions returned by the provider.</p>}
      <h3>Open orders</h3>{account.data.openOrders.length ? <div className="table-scroll" tabIndex={0} aria-label="Open orders"><table><thead><tr><th>Symbol</th><th>Side</th>{account.providerId === 'bitget' && <><th>Kind</th><th>Trigger price</th></>}<th>Quantity / Notional</th><th>Filled</th>{account.providerId === 'bitget' && <><th>Filled quote value</th><th>Limit price</th></>}<th>Status</th></tr></thead><tbody>{account.data.openOrders.map(row => <tr key={row.brokerOrderId}><td>{row.symbol}<small className="identity order-identity">{row.brokerOrderId}</small></td><td>{row.side}</td>{account.providerId === 'bitget' && <><td>{row.kind ?? 'Unavailable'}</td><td>{row.triggerPrice ?? '—'}</td></>}<td>{row.quantity ?? money(row.notional, row.currency)}</td><td>{row.filledQuantity ?? money(row.filledValue, row.currency)}</td>{account.providerId === 'bitget' && <><td>{money(row.filledValue, row.currency)}</td><td>{row.limitPrice ?? '—'}</td></>}<td>{row.status}</td></tr>)}</tbody></table></div> : <p>No open orders returned by the provider.</p>}
      <h3>Capabilities and limitations</h3><p>{account.data.capabilities.join(', ')}</p><ul>{account.data.limitations.map(text => <li key={text}>{text}</li>)}</ul>
    </>}
    <p className="muted">{localPaper ? 'Built-in TradeX simulation. No credential, provider connection or Live order exists for this account.' : 'Disconnect stops local access and removes the stored credential. It does not revoke the provider key or cancel external orders.'}</p>
  </section>;
}

export function Accounts({ workspaceId, healthOnly = false }: { workspaceId: string; healthOnly?: boolean }) {
  const queryClient = useQueryClient();
  const catalog = useQuery({ queryKey: ['providers'], queryFn: () => request('provider.list_definitions', {}) });
  const list = useQuery({ queryKey: ['accounts', workspaceId], queryFn: () => request('account.list', { workspaceId }), refetchInterval: 5000, refetchOnMount: 'always', refetchOnReconnect: 'always' });
  const paper = useQuery({ queryKey: ['paper', workspaceId], queryFn: () => request('paper.get', { workspaceId }), refetchOnMount: 'always', refetchOnReconnect: 'always' });
  const [selection, setSelection] = useState('alpaca/PAPER');
  const [label, setLabel] = useState('Paper research');
  const [selectedId, setSelectedId] = useState<string>();
  const [connectionSource, setConnectionSource] = useState('new');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [notice, setNotice] = useState('');
  const [showPortfolio, setShowPortfolio] = useState(false);
  const restoreFocus = useRef<HTMLElement | null>(null);
  const accounts = list.data?.accounts ?? [];
  const existingAccount = connectionSource === 'new' ? undefined : accounts.find(account => account.connectionId === connectionSource);
  const canReuseExisting = Boolean(existingAccount && existingAccount.connectionState !== 'DISCONNECTED' && !['MISSING', 'DELETE_PENDING'].includes(existingAccount.health.credential));
  useEffect(() => {
    const next = accounts.find(account => account.workspaceId === workspaceId)?.connectionId;
    if (!selectedId || !accounts.some(account => account.connectionId === selectedId && account.workspaceId === workspaceId)) {
      if (next !== selectedId) setSelectedId(next);
    }
    if (connectionSource !== 'new' && !accounts.some(account => account.connectionId === connectionSource)) setConnectionSource('new');
  }, [accounts, connectionSource, selectedId, workspaceId]);
  useEffect(() => {
    if (!busy && restoreFocus.current) {
      if (restoreFocus.current.isConnected) restoreFocus.current.focus();
      restoreFocus.current = null;
    }
  }, [busy]);
  const selected = useDomainProjection('account', selectedId, fromAccountSnapshot);
  const schema = catalog.data?.providers.find(p => `${p.providerId}/${p.environment}` === selection);
  const localPaperSelection = schema?.providerId === 'local-paper';
  const run = async (action: () => Promise<AccountConnection>, focusId?: string) => {
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : undefined;
    let affectedId = selectedId;
    setBusy(true); setError(null); setNotice('');
    try {
      const result = await action();
      if (result.workspaceId !== workspaceId) throw new Error('IPC_IDENTITY_CONFLICT');
      affectedId = result.connectionId;
      await queryClient.invalidateQueries({ queryKey: ['account', result.connectionId] });
    } catch (failure) {
      if (failure instanceof CommandError && failure.detail.code === 'PROVIDER_ENTRY_CANCELLED') setNotice(failure.message);
      else setError(failure);
    } finally {
      await queryClient.invalidateQueries({ queryKey: ['accounts', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['context-catalog', workspaceId] });
      if (affectedId) await queryClient.invalidateQueries({ queryKey: ['account', affectedId] });
      await list.refetch();
      if (affectedId) setSelectedId(affectedId);
      restoreFocus.current = (focusId ? document.getElementById(focusId) : trigger) ?? null;
      setBusy(false);
    }
  };
  const failure = error ?? catalog.error ?? list.error ?? selected.error ?? paper.error;
  return <div className="accounts-panel">
    {failure != null && <div className="error-banner" role="alert"><p>{explainError(failure)}</p><button onClick={() => { setError(null); void list.refetch(); void catalog.refetch(); void paper.refetch(); if (selectedId) void selected.reload(); }}>Reload account state</button></div>}
    {notice && <p role="status">{notice}</p>}{busy && <p role="status">Completing account operation…</p>}
    {!healthOnly && <section className="card provider-config" aria-labelledby="broker-providers"><h2 id="broker-providers">Broker &amp; exchange providers</h2>
      <p>Local Paper is a built-in TradeX simulation and needs no credentials; provider forms below are for external accounts.</p>
      <form onSubmit={event => { event.preventDefault(); if (schema && connectionSource === 'new') void run(() => request('provider.connect', { step: 'test', workspaceId, providerId: schema.providerId, environment: schema.environment, label }), 'connect-account'); }}>
        <label className="field">Connection source<select value={connectionSource} onChange={event => { const value = event.target.value; setConnectionSource(value); const existing = accounts.find(account => account.connectionId === value); if (existing) { setSelectedId(existing.connectionId); setSelection(`${existing.providerId}/${existing.environment}`); setLabel(existing.label); } }} disabled={busy}>
          <option value="new">New account — enter credentials securely</option>
          {accounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.label} · {account.providerId} · {account.environment} · {account.connectionState}</option>)}
        </select></label>
        <label className="field">Provider / environment<select value={selection} onChange={event => setSelection(event.target.value)} disabled={busy || connectionSource !== 'new'}>{catalog.data?.providers.map(p => <option key={`${p.providerId}/${p.environment}`} value={`${p.providerId}/${p.environment}`}>{p.displayName}{p.available ? '' : ' — unavailable'}</option>)}</select></label>
        <label className="field">Connection label<input value={label} onChange={event => setLabel(event.target.value)} maxLength={120} required disabled={busy || connectionSource !== 'new'} /></label>
          {connectionSource === 'new' ? <>
          <p>{localPaperSelection ? 'Local Paper is already provisioned below. It has no credentials and cannot be connected through a broker form.' : schema?.helpText}</p>
          {!localPaperSelection && schema?.available && <><p>Required reads: {schema.requiredPermissions.join(', ')}.</p><p>Secure fields: {schema.fields.map(field => `${field.label}${field.required ? ' (required)' : ''}`).join('; ')}. Enter these only in the native secure window.</p></>}
          <button id="connect-account" className="primary" disabled={busy || localPaperSelection || !schema?.available || !label.trim() || !(desktop || browserIntegration)}>Connect account securely</button>
        </> : <>
          {existingAccount ? <p>{canReuseExisting ? 'Use the stored local credential for this connection. Refresh runs without opening the secure credential window.' : 'This connection has no usable local credential. Choose New account to reconnect; local removal preserves its non-secret audit row.'}</p> : <p>Choose an existing local connection to inspect or refresh it.</p>}
          <button id="use-existing-account" type="button" className="primary" disabled={busy || !canReuseExisting || !(desktop || browserIntegration)} onClick={() => { if (existingAccount && canReuseExisting) void run(() => request('account.refresh', mutation(existingAccount)), 'use-existing-account'); }}>Use existing account</button>
        </>}
      </form>
    </section>}
    {paper.isPending && <p role="status">Loading Local Paper simulation…</p>}
    {paper.data && <LocalPaperSummary state={paper.data} />}
    {!healthOnly && showPortfolio && <Portfolio workspaceId={workspaceId} />}
    <section aria-labelledby="connections-title"><div className="section-heading"><div><h2 id="connections-title">Account connections</h2><p className="muted">Select an account to inspect its provider truth or TradeX simulation state.</p></div>{!healthOnly && <button type="button" onClick={() => setShowPortfolio(value => !value)} aria-expanded={showPortfolio}>{showPortfolio ? 'Hide portfolio' : 'Open portfolio'}</button>}</div>
      {list.isLoading ? <p role="status">Loading local connections…</p> : !accounts.length ? <p>No account observations are available.</p> : <div className="account-list">{accounts.map(account => { const localPaper = account.providerId === 'local-paper' && account.environment === 'LOCAL'; return <button className="account-row" key={account.connectionId} aria-pressed={selectedId === account.connectionId} onClick={() => setSelectedId(account.connectionId)}><strong>{account.label}</strong><span>{account.providerId} · {account.environment}{localPaper ? ' · LOCAL_PAPER' : ''}</span>{localPaper && <small>TRADEX_SIMULATION · {account.health.reason}</small>}<span>{account.connectionState} · {account.health.connection}</span><span>Equity / balance: {account.data?.balances.map(balance => `${balance.asset} ${balance.total ?? balance.available}`).join(' · ') || 'Unavailable'}</span><span>Arming: {account.health.arming}</span><small>Last sync: {time(account.lastSuccessfulSync)}</small></button>; })}</div>}
    </section>
    {selected.data && <AccountDetail key={selected.data.connectionId} account={selected.data} busy={busy} run={action => { void run(action); }} />}
    {selectedId && !selected.data && !selected.error && <p role="status">Restoring account state…</p>}
  </div>;
}
