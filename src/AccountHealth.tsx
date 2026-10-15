import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import type { AccountConnection } from '../shared/ipc-types.ts';
import { ErrorRecoveryPanel } from './ErrorRecoveryPanel.tsx';
import { request } from './client.ts';

type HealthDimension = { id: string; label: string; value: string };

function dimensions(account: AccountConnection): HealthDimension[] {
  return [
    { id: 'connection', label: 'Connection', value: `${account.connectionState} · ${account.health.connection}` },
    { id: 'authentication', label: 'Authentication', value: account.health.authentication },
    { id: 'private-stream', label: 'Private stream', value: account.health.privateStream },
    { id: 'reconciliation', label: 'Reconciliation', value: account.health.reconciliation },
    { id: 'execution-eligibility', label: 'Execution eligibility', value: account.health.executionEligibility },
    { id: 'arming', label: 'Arming', value: account.health.arming },
  ];
}

function timestamp(value: string | null | undefined): string {
  if (!value) return 'No observation recorded';
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? 'Timestamp unavailable' : date.toLocaleString();
}

export function AccountHealth({ workspaceId }: { workspaceId: string }) {
  const [actionError, setActionError] = useState<unknown>();
  const accounts = useQuery({
    queryKey: ['accounts', workspaceId],
    queryFn: () => request('account.list', { workspaceId }),
    refetchOnMount: 'always',
    refetchOnReconnect: 'always',
  });
  const liveAccounts = accounts.data?.accounts.filter(account => account.environment === 'LIVE') ?? [];

  const disableAllLive = async () => {
    setActionError(undefined);
    try {
      await request('account.disable_all_live', { workspaceId });
      await accounts.refetch();
    } catch (failure) {
      setActionError(failure);
    }
  };

  return <div className="account-health-surface" aria-label="Account health observations">
    <div className="account-health-heading">
      <p className="muted">Health is shown as separate local observations. Opening this page does not contact a provider.</p>
      <button type="button" onClick={() => { void accounts.refetch(); }} disabled={accounts.isFetching}>Reload health snapshot</button>
    </div>
    {liveAccounts.length > 0 && <button type="button" id="disable-all-live" onClick={() => { void disableAllLive(); }} disabled={accounts.isFetching}>
      Disable All Live Execution
    </button>}
    {Boolean(accounts.error || actionError) && <ErrorRecoveryPanel error={actionError ?? accounts.error} />}
    {accounts.isPending ? <p role="status">Loading local account health…</p> : accounts.error ? null : !accounts.data?.accounts.length ?
      <section className="card account-health-empty"><h3>No account health observations</h3><p>Connect an account from Accounts to record its local health state.</p></section> :
      <div className="account-health-list">{accounts.data.accounts.map(account => <section className="card account-health-card" key={account.connectionId} aria-labelledby={`health-${account.connectionId}`}>
        <div className="account-health-card-heading">
          <div><h3 id={`health-${account.connectionId}`}>{account.label}</h3><p>{account.providerId} · {account.environment}</p></div>
          <span className="badge">{account.connectionState}</span>
        </div>
        <dl className="account-health-dimensions">{dimensions(account).map(dimension => <div key={dimension.id}>
          <dt>{dimension.label}</dt><dd><code>{dimension.value}</code></dd>
        </div>)}</dl>
        <dl className="account-health-observations">
          <div><dt>Last successful sync</dt><dd>{timestamp(account.lastSuccessfulSync)}</dd></div>
          <div><dt>Last private stream event</dt><dd>{timestamp(account.lastPrivateStreamEventAt)}</dd></div>
        </dl>
      </section>)}</div>}
  </div>;
}
