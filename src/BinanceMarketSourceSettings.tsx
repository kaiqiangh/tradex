import { useEffect, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import { BinanceLocalUseSummary } from './BinanceLocalUseSummary.tsx';

export function BinanceMarketSourceSettings({ workspaceId }: { workspaceId: string }) {
  const client = useQueryClient();
  const key = ['binance-market-source', workspaceId];
  const connection = useQuery({ queryKey: key, queryFn: () => request('data.binance_market.connection', { workspaceId }), retry: false, refetchInterval: 1000 });
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string>();
  const [error, setError] = useState<string>();
  const [declaredJurisdiction, setDeclaredJurisdiction] = useState('');
  const [confirmLocalUse, setConfirmLocalUse] = useState(false);

  useEffect(() => {
    if (!connection.data) return;
    setDeclaredJurisdiction(connection.data.localUseAttestation?.declaredJurisdiction ?? '');
    setConfirmLocalUse(false);
  }, [connection.data?.stateVersion]);

  const act = async (operation: 'configure' | 'disconnect' | 'attest-local-use' | 'withdraw-local-use') => {
    if (!connection.data || busy) return;
    setBusy(true); setNotice(undefined); setError(undefined);
    try {
      const state = connection.data;
      const result = operation === 'attest-local-use'
        ? await request('data.binance_market.attest_local_use', {
            workspaceId,
            expectedStateVersion: state.stateVersion,
            declaredJurisdiction,
            confirmLocalTradingDecisionSupport: true,
          })
        : operation === 'withdraw-local-use'
          ? await request('data.binance_market.withdraw_local_use', {
              workspaceId,
              expectedStateVersion: state.stateVersion,
            })
          : await request(operation === 'configure' ? 'data.binance_market.configure' : 'data.binance_market.disconnect', {
              workspaceId,
              expectedStateVersion: state.stateVersion,
            });
      client.setQueryData(key, result);
      await client.invalidateQueries({ queryKey: ['data-source-catalog', workspaceId] });
      await client.invalidateQueries({ queryKey: ['market-detail', workspaceId] });
      setNotice(operation === 'configure' ? 'Public market-source selection saved. No provider read was made; the source generation changed.'
        : operation === 'disconnect' ? 'Public market source disconnected. Account connections and saved keys were kept.'
          : operation === 'attest-local-use' ? 'Your local-use confirmation was recorded. It does not verify provider entitlement or satisfy another risk, approval, Arm, or order gate.'
            : 'Your local-use confirmation was withdrawn. Current market evidence was retired with the source generation change.');
    } catch (cause) {
      setError(explainError(cause));
      await connection.refetch();
    } finally { setBusy(false); }
  };

  if (connection.isPending) return <p role="status">Loading Binance market source…</p>;
  if (connection.isError) return <div role="alert"><p>{explainError(connection.error)}</p><button type="button" onClick={() => void connection.refetch()}>Reload Binance market source</button></div>;
  const state = connection.data;
  const source = state.source;
  const jurisdictionValid = /^[A-Z]{2}$/.test(declaredJurisdiction);
  const assessmentLabel: Record<typeof state.localUseAssessment.state, string> = {
    SOURCE_NOT_SELECTED: 'Source not selected',
    NOT_RECORDED: 'No current confirmation',
    CURRENT: 'Current user confirmation',
    STALE: 'Confirmation is stale',
    MISMATCHED: 'Confirmation binding does not match',
    UNSUPPORTED: 'Confirmation profile is unsupported',
    RESTRICTED: 'Declared jurisdiction is restricted',
    POLICY_UNRESOLVED: 'Policy or terms revision is unresolved',
  };

  return <section className="card quote-source-settings" aria-labelledby="binance-market-heading">
    <h3 id="binance-market-heading">Binance Spot market source</h3>
    <p>Public BTC / USDT and ETH / USDT depth from the ordinary Binance Spot service. No trading account or key is needed for this market source.</p>
    <dl className="data-source-details">
      <div><dt>Selection</dt><dd>{state.configured ? 'Saved' : 'Not selected'}</dd></div>
      <div><dt>Technical collection</dt><dd>{source.status === 'AVAILABLE' ? 'Available in the current Hot view' : source.status === 'UNVERIFIED' ? 'Not verified' : 'Unavailable'}</dd></div>
      <div><dt>Provider entitlement</dt><dd>UNVERIFIED — access does not verify financial use, retention, redistribution, commercial use, or regional permission.</dd></div>
    </dl>
    <p role="status">{source.availabilityReason}</p>
    <p className="form-hint">Save records the source selection only. A visible BTC / USDT or ETH / USDT Hot view owns its bounded stream; a connected socket is not a quote, authentication, or permission. Saving again or disconnecting retires current market evidence.</p>

    <section className="source-use-assessment" aria-label="Binance local-use assessment">
      <h4>Local-use confirmation status</h4>
      <dl className="health-grid">
        <div><dt>Status</dt><dd>{assessmentLabel[state.localUseAssessment.state]}</dd></div>
        <div><dt>Control Plane reason</dt><dd>{state.localUseAssessment.reason}</dd></div>
      </dl>
      <p><strong>Provider entitlement and <code>BinanceSpotQuoteEvidence.dataUseRights</code> remain UNVERIFIED in every state.</strong> This user confirmation is not Binance verification or legal approval, and does not permit external or commercial use.</p>
    </section>

    {state.localUseAttestation && <BinanceLocalUseSummary attestation={state.localUseAttestation} />}

    {state.source.termsUrl && <section className="source-use-fields" aria-label="Current Binance terms">
      <h4>Current terms for review</h4>
      <p><a href={source.termsUrl} target="_blank" rel="noreferrer">Open Binance terms</a> · reviewed {source.reviewedAt}</p>
    </section>}

    <fieldset className="source-use-fields" disabled={busy || !state.configured}>
      <legend>Confirm your own local use</legend>
      <p className="form-hint">This records only your statement that the displayed ordinary public Spot dataset is for your own local, non-commercial trading decision support in TradeX, including owning-plan and risk review and Prepare. It does not grant approval, Arm, PLACE, order submission, provider entitlement, legal permission, or external/commercial use.</p>
      <label>Declared jurisdiction<input aria-label="Declared jurisdiction" value={declaredJurisdiction} maxLength={2} autoComplete="off" placeholder="Two-letter uppercase country code" onChange={event => {
        setDeclaredJurisdiction(event.target.value.replace(/[^a-z]/gi, '').toUpperCase());
        setNotice(undefined);
      }} /></label>
      <div className="local-use-attestation-confirmation"><label><input type="checkbox" checked={confirmLocalUse} onChange={event => setConfirmLocalUse(event.target.checked)} /> I reviewed the current Binance terms above and believe they permit only this local, non-commercial use in my declared jurisdiction.</label></div>
      <button type="button" disabled={busy || !state.configured || !jurisdictionValid || !confirmLocalUse} onClick={() => void act('attest-local-use')}>{busy ? 'Working…' : 'Record my local-use confirmation'}</button>
      <p className="form-hint">The confirmation covers BTC / USDT and ETH / USDT ordinary Spot quotes and bounded depth. BNB / USDT is included only when required for fee valuation. Raw or derived market data stays local and is not sent to model/tool prompts, research, exports, or external consumers.</p>
    </fieldset>

    {notice && <p role="status" className="success-text">{notice}</p>}
    {error && <div role="alert"><p>{error}</p><button type="button" disabled={busy} onClick={() => void connection.refetch()}>Reload Binance market source</button></div>}
    <div className="quote-source-settings-actions">
      <button type="button" disabled={busy} onClick={() => void act('configure')}>{busy ? 'Working…' : 'Save Binance market source'}</button>
      {state.localUseAttestation && <button type="button" disabled={busy} onClick={() => void act('withdraw-local-use')}>Withdraw local-use confirmation</button>}
      {state.configured && <button type="button" disabled={busy} onClick={() => void act('disconnect')}>Disconnect Binance market source</button>}
    </div>
  </section>;
}
