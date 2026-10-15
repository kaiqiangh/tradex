import { useEffect, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { explainError, request } from './client.ts';
import { FinancialEvidencePanel } from './FinancialEvidencePanel.tsx';
import type { AlpacaCryptoLocation, AlpacaMarketDataPlan, FxDeclaredUseScope, FxRawDataDestination, FxSourceUseContext } from '../shared/ipc-types.ts';

type FxSourceUseDraft = {
  marketDataPlan: AlpacaMarketDataPlan | '';
  accountJurisdiction: string;
  agreementReviewedAt: string;
  supportConfirmedJurisdiction: string;
  supportConfirmedAt: string;
  supportReference: string;
  useScope: FxDeclaredUseScope | '';
  rawDataDestination: FxRawDataDestination | '';
};

const EMPTY_FX_SOURCE_USE_DRAFT: FxSourceUseDraft = {
  marketDataPlan: '', accountJurisdiction: '', agreementReviewedAt: '', supportConfirmedJurisdiction: '', supportConfirmedAt: '', supportReference: '', useScope: '', rawDataDestination: '',
};

function sourceUseDraft(context?: FxSourceUseContext | null): FxSourceUseDraft {
  return context ? {
    ...context,
    supportConfirmedJurisdiction: context.countrySupportConfirmation?.confirmedJurisdiction ?? '',
    supportConfirmedAt: context.countrySupportConfirmation?.confirmedAt ?? '',
    supportReference: context.countrySupportConfirmation?.reference ?? '',
  } : { ...EMPTY_FX_SOURCE_USE_DRAFT };
}

function sameSourceUseContext(left: FxSourceUseContext | null | undefined, right: FxSourceUseContext | null | undefined): boolean {
  if (!left || !right) return left === right || (!left && !right);
  return left.marketDataPlan === right.marketDataPlan
    && left.accountJurisdiction === right.accountJurisdiction
    && left.agreementReviewedAt === right.agreementReviewedAt
    && left.useScope === right.useScope
    && left.rawDataDestination === right.rawDataDestination
    && left.countrySupportConfirmation?.confirmedJurisdiction === right.countrySupportConfirmation?.confirmedJurisdiction
    && left.countrySupportConfirmation?.confirmedAt === right.countrySupportConfirmation?.confirmedAt
    && left.countrySupportConfirmation?.reference === right.countrySupportConfirmation?.reference;
}

function sourceUseContext(draft: FxSourceUseDraft): FxSourceUseContext | null | undefined {
  const required = [draft.marketDataPlan, draft.accountJurisdiction, draft.agreementReviewedAt, draft.useScope, draft.rawDataDestination];
  const support = [draft.supportConfirmedJurisdiction, draft.supportConfirmedAt, draft.supportReference];
  if ([...required, ...support].every(value => value === '')) return null;
  if (required.some(value => value === '') || (support.some(value => value === '') && support.some(value => value !== ''))) return undefined;
  return {
    marketDataPlan: draft.marketDataPlan as AlpacaMarketDataPlan,
    accountJurisdiction: draft.accountJurisdiction.trim().toUpperCase(),
    agreementReviewedAt: draft.agreementReviewedAt,
    useScope: draft.useScope as FxDeclaredUseScope,
    rawDataDestination: draft.rawDataDestination as FxRawDataDestination,
    ...(support.every(value => value !== '') ? { countrySupportConfirmation: {
      confirmedJurisdiction: draft.supportConfirmedJurisdiction.trim().toUpperCase(),
      confirmedAt: draft.supportConfirmedAt,
      reference: draft.supportReference.trim(),
    } } : {}),
  };
}

const sources = {
  fx: { title: 'Alpaca currency rates', account: 'Currency rates account', selection: 'Paper', explanation: 'Read only the currency pairs required by current account and workspace inputs, using a saved Alpaca Paper key. Crypto location must be selected explicitly. Authentication and source-use facts do not prove provider data-use rights, transaction qualification or exact broker conversion costs.' },
  actions: { title: 'Alpaca known company events', account: 'Company events account', selection: 'Paper', explanation: 'Read processed company events using a saved Alpaca Paper key. A completed query does not establish complete action coverage or adjusted history.' },
  instrument: { title: 'Trading 212 account instruments', account: 'Account instruments account', selection: 'Live', explanation: 'Read account identity, instrument metadata and schedules using the same saved Trading 212 Live key. These reads do not establish current account tradability, exchange halts or full key permissions.' },
} as const;

export function FinancialSourceSettings({ workspaceId, kind, onSourceChange }: { workspaceId: string; kind: keyof typeof sources; onSourceChange: () => void }) {
  const config = sources[kind];
  const cacheKey = ['financial-source', workspaceId, kind];
  const queryClient = useQueryClient();
  const source = useQuery({ queryKey: cacheKey, queryFn: () => request(`data.${kind}.connection`, { workspaceId }), retry: false, refetchInterval: 1000 });
  const [accountId, setAccountId] = useState('');
  const [cryptoLocation, setCryptoLocation] = useState<AlpacaCryptoLocation | ''>('');
  const [fxSourceUseDraft, setFxSourceUseDraft] = useState<FxSourceUseDraft>({ ...EMPTY_FX_SOURCE_USE_DRAFT });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState<string>();
  useEffect(() => {
    setAccountId(source.data?.connectionId ?? '');
    setCryptoLocation(source.data?.cryptoLocation ?? '');
    setFxSourceUseDraft(sourceUseDraft(source.data?.sourceUseContext));
  }, [source.data?.stateVersion]);
  const nextSourceUseContext = kind === 'fx' ? sourceUseContext(fxSourceUseDraft) : null;
  const incompleteSourceUseContext = nextSourceUseContext === undefined;
  const sourceUseContextChanged = kind === 'fx' && (incompleteSourceUseContext
    || !sameSourceUseContext(nextSourceUseContext, source.data?.sourceUseContext ?? null));
  const profileChanged = accountId !== (source.data?.connectionId ?? '')
    || (kind === 'fx' && cryptoLocation !== (source.data?.cryptoLocation ?? ''))
    || sourceUseContextChanged;
  const act = async (operation: 'configure' | 'refresh' | 'disconnect') => {
    if (!source.data || busy || (operation === 'configure' && incompleteSourceUseContext)) return;
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const payload = { workspaceId, expectedStateVersion: source.data.stateVersion };
      const result = operation === 'configure'
        ? await request(`data.${kind}.configure`, { ...payload, connectionId: accountId,
          ...(kind === 'fx' ? { cryptoLocation: cryptoLocation || null, sourceUseContext: nextSourceUseContext ?? null } : {}) })
        : operation === 'refresh'
          ? await request(`data.${kind}.refresh`, payload)
          : await request(`data.${kind}.disconnect`, payload);
      queryClient.setQueryData(cacheKey, result);
      onSourceChange();
      await queryClient.invalidateQueries({ queryKey: ['data-source-catalog', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['market-detail', workspaceId] });
      if (operation === 'refresh' && result.status !== 'AVAILABLE') setError(result.availabilityReason);
      else setNotice(operation === 'configure' ? `${config.title} selection saved. Read access has not been verified.`
        : operation === 'disconnect' ? `${config.title} disconnected. Your account and saved key were kept.`
          : `${config.title} refreshed. Only the listed capabilities are current.`);
    } catch (cause) { setError(explainError(cause)); await source.refetch(); }
    finally { setBusy(false); }
  };
  if (source.isPending) return <p role="status">Loading {config.title.toLowerCase()}…</p>;
  if (source.isError) return <div role="alert"><p>{explainError(source.error)}</p><button type="button" onClick={() => void source.refetch()}>Reload {config.title.toLowerCase()}</button></div>;
  return <section className="card quote-source-settings" aria-labelledby={`${kind}-source-heading`}>
    <h3 id={`${kind}-source-heading`}>{config.title}</h3>
    <p>{config.explanation}</p>
    <label>{config.account}<select aria-label={config.account} value={accountId} disabled={busy} onChange={event => {
      const selected = event.target.value;
      setAccountId(selected);
      setNotice(undefined);
      setFxSourceUseDraft(selected === source.data.connectionId ? sourceUseDraft(source.data.sourceUseContext) : { ...EMPTY_FX_SOURCE_USE_DRAFT });
    }}>
      <option value="">Choose a saved {config.selection} account</option>
      {source.data.eligibleAccounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.displayName}</option>)}
      {source.data.connectionId && !source.data.eligibleAccounts.some(account => account.connectionId === source.data.connectionId) && <option value={source.data.connectionId}>Saved account — reconnect to use this key</option>}
    </select></label>
    {kind === 'fx' && <>
      <label>Crypto market-data location<select aria-label="Crypto market-data location" value={cryptoLocation} disabled={busy} onChange={event => { setCryptoLocation(event.target.value as AlpacaCryptoLocation | ''); setNotice(undefined); }}>
        <option value="">Choose a location</option><option value="us">Alpaca US</option><option value="us-1">Kraken US through Alpaca</option><option value="eu-1">Kraken EU through Alpaca</option>
      </select></label>
      <p className="form-hint">This selection applies only to crypto market data. Forex access is separate; an unavailable location is never replaced automatically.</p>
      <fieldset className="source-use-fields" disabled={busy}>
        <legend>Alpaca source-use facts</legend>
        <p className="form-hint">Record the plan and agreement details shown in your provider account. These user-reported facts are evaluated against a local policy and do not prove market-data entitlement.</p>
        <label>Market-data plan<select aria-label="Alpaca market-data plan" value={fxSourceUseDraft.marketDataPlan} onChange={event => { setFxSourceUseDraft(current => ({ ...current, marketDataPlan: event.target.value as FxSourceUseDraft['marketDataPlan'] })); setNotice(undefined); }}>
          <option value="">Choose a plan</option><option value="BASIC">Basic</option><option value="ALGO_TRADER_PLUS">Algo Trader Plus</option><option value="UNKNOWN">Not recorded / unsure</option>
        </select></label>
        <label>Account jurisdiction<input aria-label="Account jurisdiction" value={fxSourceUseDraft.accountJurisdiction} maxLength={2} autoComplete="off" placeholder="Two-letter country code" onChange={event => { setFxSourceUseDraft(current => ({ ...current, accountJurisdiction: event.target.value.replace(/[^a-z]/gi, '').toUpperCase() })); setNotice(undefined); }} /></label>
        <label>Provider agreement reviewed on<input aria-label="Provider agreement reviewed on" type="date" value={fxSourceUseDraft.agreementReviewedAt} onChange={event => { setFxSourceUseDraft(current => ({ ...current, agreementReviewedAt: event.target.value })); setNotice(undefined); }} /></label>
        <label>Country confirmed by Alpaca Support<input aria-label="Country confirmed by Alpaca Support" value={fxSourceUseDraft.supportConfirmedJurisdiction} maxLength={2} autoComplete="off" placeholder="Leave blank without confirmation" onChange={event => { setFxSourceUseDraft(current => ({ ...current, supportConfirmedJurisdiction: event.target.value.replace(/[^a-z]/gi, '').toUpperCase() })); setNotice(undefined); }} /></label>
        <label>Support confirmation date<input aria-label="Support confirmation date" type="date" value={fxSourceUseDraft.supportConfirmedAt} onChange={event => { setFxSourceUseDraft(current => ({ ...current, supportConfirmedAt: event.target.value })); setNotice(undefined); }} /></label>
        <label>Support case or message reference<input aria-label="Support case or message reference" value={fxSourceUseDraft.supportReference} maxLength={128} autoComplete="off" placeholder="Case number or message subject" onChange={event => { setFxSourceUseDraft(current => ({ ...current, supportReference: event.target.value })); setNotice(undefined); }} /></label>
        <p className="form-hint">Only enter a country-specific Support reference if you have one. This human-reported evidence can support local policy valuation only; it never verifies data rights or enables a transaction.</p>
        <label>Declared use scope<select aria-label="Declared use scope" value={fxSourceUseDraft.useScope} onChange={event => { setFxSourceUseDraft(current => ({ ...current, useScope: event.target.value as FxSourceUseDraft['useScope'] })); setNotice(undefined); }}>
          <option value="">Choose a use scope</option><option value="OWN_LOCAL_NONCOMMERCIAL">Own local, non-commercial use</option><option value="COMMERCIAL_OR_SHARED">Commercial or shared use</option>
        </select></label>
        <label>Raw market data destination<select aria-label="Raw market data destination" value={fxSourceUseDraft.rawDataDestination} onChange={event => { setFxSourceUseDraft(current => ({ ...current, rawDataDestination: event.target.value as FxSourceUseDraft['rawDataDestination'] })); setNotice(undefined); }}>
          <option value="">Choose a destination</option><option value="LOCAL_ONLY">Local TradeX only</option><option value="EXTERNAL">External service or recipient</option>
        </select></label>
        {incompleteSourceUseContext && <p role="status">Complete all source-use facts or clear them all before saving.</p>}
        {accountId !== (source.data.connectionId ?? '') && <p className="form-hint">Source-use facts are account-specific. Review and enter facts for the selected account before saving them.</p>}
      </fieldset>
      {source.data.sourceUseAssessment && <section className="source-use-assessment" aria-label="Source-use assessment">
        <h4>Source-use assessment</h4>
        <dl className="health-grid">
          <div><dt>Result</dt><dd>{source.data.sourceUseAssessment.status === 'SUPPORTED_FOR_DECLARED_SCOPE' ? 'Supported for declared scope' : source.data.sourceUseAssessment.status === 'OUT_OF_PROFILE' ? 'Outside local profile' : 'Unresolved'}</dd></div>
          <div><dt>Evidence origin</dt><dd>{source.data.sourceUseAssessment.provenance === 'USER_REPORTED_PROVIDER_SUPPORT' ? 'User-reported Alpaca Support confirmation' : source.data.sourceUseAssessment.provenance === 'USER_REPORTED_PROVIDER_UI' ? 'User-reported provider UI' : 'Not recorded'}</dd></div>
          <div><dt>Policy version</dt><dd>{source.data.sourceUseAssessment.policyVersion}</dd></div>
        </dl>
        {accountId !== (source.data.connectionId ?? '') && <p role="status">This assessment belongs to the saved account. The selected account has no assessment until you save its own facts.</p>}
        <p>{source.data.sourceUseAssessment.reason}</p>
        <p><strong>Rights remain unverified.</strong> “Supported for declared scope” is a local policy result, not provider entitlement evidence. Do not send raw market data outside this local TradeX process.</p>
        <h5>Pinned provider documents</h5>
        <ul>{source.data.sourceUseAssessment.documents.map(document => <li key={document.documentId}>
          <a href={document.url} target="_blank" rel="noreferrer">{document.title}</a>
          <code>SHA-256 {document.sha256}</code>
        </li>)}</ul>
      </section>}
    </>}
    {profileChanged && <p role="status">Save this selection before refreshing. The current evidence and policy assessment belong to the saved profile.</p>}
    <FinancialEvidencePanel source={source.data} />
    {notice && <p role="status">{notice}</p>}
    {error && <div role="alert"><p>{error}</p><button type="button" disabled={busy} onClick={() => void source.refetch()}>Reload {config.title.toLowerCase()}</button></div>}
    <button type="button" disabled={busy || incompleteSourceUseContext || !source.data.eligibleAccounts.some(account => account.connectionId === accountId)} onClick={() => void act('configure')}>{busy ? 'Working…' : `Save ${config.title.toLowerCase()}`}</button>
    {source.data.configured && <><button type="button" disabled={busy || accountId !== source.data.connectionId} onClick={() => void act('refresh')}>Refresh {config.title.toLowerCase()}</button><button type="button" disabled={busy} onClick={() => void act('disconnect')}>Disconnect {config.title.toLowerCase()}</button></>}
  </section>;
}
