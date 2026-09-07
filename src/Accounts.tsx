import { useEffect, useRef, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { AccountConnection } from '../shared/ipc-types.ts';
import { CommandError, desktop, browserIntegration, explainError, request } from './client.ts';
import { fromAccountSnapshot } from './projection.ts';
import { useDomainProjection } from './useDomainProjection.ts';

function mutation(a: AccountConnection) { return { workspaceId: a.workspaceId, connectionId: a.connectionId, expectedStateVersion: a.stateVersion }; }
function time(value: string | null | undefined) { return value ? new Date(value).toLocaleString() : 'Not yet'; }

function AccountDetail({ account, busy, run }: { account: AccountConnection; busy: boolean; run: (action: () => Promise<AccountConnection>) => void }) {
  const [acknowledgedVersion, setAcknowledgedVersion] = useState<string>();
  const acknowledged = acknowledgedVersion === account.stateVersion;
  const p = account.permissions;
  const blocked = p.forbidden.length > 0 || p.unsupported.length > 0;
  const disconnected = account.connectionState === 'DISCONNECTED';
  return <section className="card account-detail" aria-labelledby="account-detail-title">
    <div className="account-heading"><div><h2 id="account-detail-title">{account.label}</h2><p>{account.providerId} · {account.environment} · {account.connectionState}</p></div>
      <div className="account-actions"><button disabled={busy || disconnected || account.connectionState === 'CONNECTING' || ['MISSING', 'DELETE_PENDING'].includes(account.health.credential)} onClick={() => run(() => request('account.refresh', mutation(account)))}>Refresh account</button>
        <button disabled={busy || (disconnected && account.health.credential === 'MISSING')} onClick={() => run(() => request('provider.disconnect', mutation(account)))}>{account.health.credential === 'DELETE_PENDING' ? 'Retry Keychain cleanup' : 'Disconnect'}</button></div>
    </div>
    <p className="notice">{account.health.reason}</p>
    <dl className="health-grid">{Object.entries(account.health).filter(([key]) => key !== 'reason').map(([key, value]) => <div key={key}><dt>{({ connection: 'Connection', authentication: 'Authentication', credential: 'Credential', privateStream: 'Private stream', reconciliation: 'Reconciliation', executionEligibility: 'Execution eligibility', arming: 'Arming' } as Record<string, string>)[key]}</dt><dd>{value}</dd></div>)}
      <div><dt>Last successful sync</dt><dd>{time(account.lastSuccessfulSync)}</dd></div><div><dt>Last reconciliation</dt><dd>Not yet</dd></div><div><dt>Risk policy</dt><dd>Not configured</dd></div>
    </dl>
    <section className="permission-review" aria-labelledby="permission-title"><h3 id="permission-title">Permission review</h3>
      <p><strong>{blocked ? 'BLOCKED' : p.scope}</strong> · IP allow-list: {p.ipAllowListStatus}</p>
      <p>Observed access: {p.detected.join(', ') || 'No successful permission observations'}</p>
      {p.scope === 'UNVERIFIED' && <p>TradeX cannot fully inspect this API key’s permissions. Successful reads do not verify trading, withdrawal or transfer scope.</p>}
      {blocked && <p role="alert">Remove these permissions at the provider and re-test: {[...p.forbidden, ...p.unsupported].join(', ')}.</p>}
      {p.acknowledged && <p>Unverified scope was explicitly acknowledged for this permission review.</p>}
      {account.connectionState === 'REVIEW_REQUIRED' && <>
        {p.scope === 'UNVERIFIED' && <label className="check-field"><input type="checkbox" checked={acknowledged} onChange={event => setAcknowledgedVersion(event.target.checked ? account.stateVersion : undefined)} />I understand that permission scope is unverified and have checked the key’s permissions at the provider.</label>}
        <button className="primary" disabled={busy || blocked || account.health.authentication !== 'VALID' || account.health.connection !== 'ONLINE' || (p.scope === 'UNVERIFIED' && !acknowledged)} onClick={() => run(() => request('provider.connect', { step: 'confirm', ...mutation(account), acknowledgeUnverified: acknowledged }))}>Confirm connection</button>
      </>}
    </section>
    {account.data && <><dl className="health-grid"><div><dt>Account type</dt><dd>{account.data.accountType}</dd></div><div><dt>Provider account ID</dt><dd className="identity">{account.data.remoteAccountId}</dd></div><div><dt>Currency</dt><dd>{account.data.currency ?? 'Per asset'}</dd></div></dl>
      <h3>Balances</h3><div className="table-scroll" tabIndex={0} aria-label="Account balances"><table><thead><tr><th>Asset</th><th>Cash available</th><th>Equity</th></tr></thead><tbody>{account.data.balances.map(row => <tr key={row.asset}><td>{row.asset}</td><td>{row.available}</td><td>{row.total ?? 'Unavailable'}</td></tr>)}</tbody></table></div>
      <h3>Positions</h3>{account.data.positions.length ? <div className="table-scroll" tabIndex={0} aria-label="Account positions"><table><thead><tr><th>Symbol</th><th>Quantity</th><th>Market value</th><th>Average entry</th></tr></thead><tbody>{account.data.positions.map(row => <tr key={row.symbol}><td>{row.symbol}</td><td>{row.quantity}</td><td>{row.marketValue ?? 'Unavailable'}</td><td>{row.averageEntryPrice ?? 'Unavailable'}</td></tr>)}</tbody></table></div> : <p>No positions returned by the provider.</p>}
      <h3>Open orders</h3>{account.data.openOrders.length ? <div className="table-scroll" tabIndex={0} aria-label="Open orders"><table><thead><tr><th>Symbol</th><th>Side</th><th>Quantity / Notional</th><th>Filled</th><th>Status</th></tr></thead><tbody>{account.data.openOrders.map(row => <tr key={row.brokerOrderId}><td>{row.symbol}<small className="identity order-identity">{row.brokerOrderId}</small></td><td>{row.side}</td><td>{row.quantity ?? `${row.notional} ${account.data!.currency ?? ''}`}</td><td>{row.filledQuantity}</td><td>{row.status}</td></tr>)}</tbody></table></div> : <p>No open orders returned by the provider.</p>}
      <h3>Capabilities and limitations</h3><p>{account.data.capabilities.join(', ')}</p><ul>{account.data.limitations.map(text => <li key={text}>{text}</li>)}</ul>
    </>}
    <p className="muted">Disconnect stops local access and removes the stored credential. It does not revoke the provider key or cancel external orders.</p>
  </section>;
}

export function Accounts({ workspaceId, healthOnly = false }: { workspaceId: string; healthOnly?: boolean }) {
  const queryClient = useQueryClient();
  const catalog = useQuery({ queryKey: ['providers'], queryFn: () => request('provider.list_definitions', {}) });
  const list = useQuery({ queryKey: ['accounts', workspaceId], queryFn: () => request('account.list', { workspaceId }), refetchInterval: 5000 });
  const [selection, setSelection] = useState('alpaca/PAPER');
  const [label, setLabel] = useState('Paper research');
  const [selectedId, setSelectedId] = useState<string>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [notice, setNotice] = useState('');
  const restoreFocus = useRef<HTMLElement | null>(null);
  useEffect(() => {
    if (!busy && restoreFocus.current) {
      if (restoreFocus.current.isConnected) restoreFocus.current.focus();
      restoreFocus.current = null;
    }
  }, [busy]);
  const selected = useDomainProjection('account', selectedId, fromAccountSnapshot);
  const schema = catalog.data?.providers.find(p => `${p.providerId}/${p.environment}` === selection);
  const run = async (action: () => Promise<AccountConnection>, focusId?: string) => {
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : undefined;
    setBusy(true); setError(null); setNotice('');
    try {
      const result = await action();
      if (result.workspaceId !== workspaceId) throw new Error('IPC_IDENTITY_CONFLICT');
      setSelectedId(result.connectionId);
      await queryClient.invalidateQueries({ queryKey: ['account', result.connectionId] });
    } catch (failure) {
      if (failure instanceof CommandError && failure.detail.code === 'PROVIDER_ENTRY_CANCELLED') setNotice(failure.message);
      else setError(failure);
    } finally {
      await queryClient.invalidateQueries({ queryKey: ['accounts', workspaceId] });
      if (selectedId) await queryClient.invalidateQueries({ queryKey: ['account', selectedId] });
      restoreFocus.current = (focusId ? document.getElementById(focusId) : trigger) ?? null;
      setBusy(false);
    }
  };
  const failure = error ?? catalog.error ?? list.error ?? selected.error;
  return <div className="accounts-panel">
    {failure != null && <div className="error-banner" role="alert"><p>{explainError(failure)}</p><button onClick={() => { setError(null); void list.refetch(); void catalog.refetch(); if (selectedId) void selected.reload(); }}>Reload account state</button></div>}
    {notice && <p role="status">{notice}</p>}{busy && <p role="status">Completing account operation…</p>}
    {!healthOnly && <section className="card provider-config" aria-labelledby="broker-providers"><h2 id="broker-providers">Broker &amp; exchange providers</h2>
      <p>Local Paper is built-in and needs no credentials. Its simulation engine is not configured yet.</p>
      <form onSubmit={event => { event.preventDefault(); if (schema) void run(() => request('provider.connect', { step: 'test', workspaceId, providerId: schema.providerId, environment: schema.environment, label }), 'connect-account'); }}>
        <label className="field">Provider / environment<select value={selection} onChange={event => setSelection(event.target.value)} disabled={busy}>{catalog.data?.providers.map(p => <option key={`${p.providerId}/${p.environment}`} value={`${p.providerId}/${p.environment}`}>{p.displayName}{p.available ? '' : ' — unavailable'}</option>)}</select></label>
        <label className="field">Connection label<input value={label} onChange={event => setLabel(event.target.value)} maxLength={120} required disabled={busy} /></label>
        <p>{schema?.helpText}</p>
        {schema?.available && <><p>Required reads: {schema.requiredPermissions.join(', ')}.</p><p>Secure fields: {schema.fields.map(field => `${field.label}${field.required ? ' (required)' : ''}`).join('; ')}. Enter these only in the native secure window.</p></>}
        <button id="connect-account" className="primary" disabled={busy || !schema?.available || !label.trim() || !(desktop || browserIntegration)}>Connect account securely</button>
      </form>
    </section>}
    <section aria-labelledby="connections-title"><h2 id="connections-title">Account connections</h2>
      {list.isLoading ? <p role="status">Loading local connections…</p> : !list.data?.accounts.length ? <p>No external accounts are connected.</p> : <div className="account-list">{list.data.accounts.map(account => <button className="account-row" key={account.connectionId} aria-pressed={selectedId === account.connectionId} onClick={() => setSelectedId(account.connectionId)}><strong>{account.label}</strong><span>{account.providerId} · {account.environment}</span><span>{account.connectionState} · {account.health.connection}</span><span>Equity / balance: {account.data?.balances.map(balance => `${balance.asset} ${balance.total ?? balance.available}`).join(' · ') || 'Unavailable'}</span><span>Arming: {account.health.arming}</span><small>Last sync: {time(account.lastSuccessfulSync)}</small></button>)}</div>}
    </section>
    {selected.data && <AccountDetail key={selected.data.connectionId} account={selected.data} busy={busy} run={action => { void run(action); }} />}
    {selectedId && !selected.data && !selected.error && <p role="status">Restoring account state…</p>}
  </div>;
}
