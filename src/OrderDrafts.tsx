import { useEffect, useMemo, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type {
  AccountConnection,
  ExecutionContext,
  MarketTier,
  OrderDraft,
  OrderDraftFields,
  OrderDraftSummary,
  OrderProposal,
  OrderProposalSummary,
  PaperOrderResult,
  OrderQuantityType,
  OrderSide,
  OrderType,
  TimeInForce,
} from '../shared/ipc-types.ts';
import { CommandError, explainError, request } from './client.ts';

const environments: { value: ExecutionContext; label: string }[] = [
  { value: 'LOCAL_PAPER', label: 'Local Paper' },
  { value: 'ALPACA_PAPER', label: 'Alpaca Paper' },
  { value: 'TRADING212_DEMO', label: 'Trading 212 Demo' },
  { value: 'TRADING212_LIVE', label: 'Trading 212 Live' },
  { value: 'BINANCE_TESTNET', label: 'Binance Testnet' },
  { value: 'BINANCE_LIVE', label: 'Binance Live' },
  { value: 'BITGET_DEMO', label: 'Bitget Demo' },
  { value: 'BITGET_LIVE', label: 'Bitget Live' },
];
const marketTier: MarketTier = 'CENSUS';

type DraftForm = {
  accountId: string;
  venue: string;
  environment: ExecutionContext;
  instrumentId: string;
  side: OrderSide;
  orderType: OrderType;
  quantityType: OrderQuantityType;
  quantity: string;
  limitPrice: string;
  maximumSpend: string;
  timeInForce: TimeInForce;
  clientLabel: string;
};
type DraftField = keyof DraftForm;

const initialForm: DraftForm = {
  accountId: '', venue: 'TRADEX_SIM', environment: 'LOCAL_PAPER', instrumentId: 'equity:US:AAPL',
  side: 'BUY', orderType: 'LIMIT', quantityType: 'BASE', quantity: '1', limitPrice: '221.50',
  maximumSpend: '', timeInForce: 'DAY', clientLabel: '',
};

function fromDraft(draft: OrderDraft): DraftForm {
  return {
    accountId: draft.fields.accountId ?? '', venue: draft.fields.venue, environment: draft.fields.environment,
    instrumentId: draft.fields.instrumentId, side: draft.fields.side, orderType: draft.fields.orderType,
    quantityType: draft.fields.quantity.type, quantity: draft.fields.quantity.value,
    limitPrice: draft.fields.limitPrice ?? '', maximumSpend: draft.fields.maximumSpend ?? '',
    timeInForce: draft.fields.timeInForce, clientLabel: draft.fields.clientLabel ?? '',
  };
}

function isPositiveDecimal(value: string) { return /^(?:\d+)(?:\.\d+)?$/.test(value) && value !== '0' && !/^0(?:\.0+)?$/.test(value); }

function contextForAccount(providerId: string, environment: string): ExecutionContext | undefined {
  if (providerId === 'alpaca' && environment === 'PAPER') return 'ALPACA_PAPER';
  if (providerId === 'trading212' && environment === 'DEMO') return 'TRADING212_DEMO';
  if (providerId === 'trading212' && environment === 'LIVE') return 'TRADING212_LIVE';
  if (providerId === 'binance' && environment === 'TESTNET') return 'BINANCE_TESTNET';
  if (providerId === 'binance' && environment === 'LIVE') return 'BINANCE_LIVE';
  if (providerId === 'bitget' && environment === 'DEMO') return 'BITGET_DEMO';
  if (providerId === 'bitget' && environment === 'LIVE') return 'BITGET_LIVE';
  return environment === 'LOCAL' ? 'LOCAL_PAPER' : undefined;
}

function expectedVenue(environment: ExecutionContext, instrumentId: string) {
  if (environment === 'LOCAL_PAPER') return 'TRADEX_SIM';
  if (environment.startsWith('BINANCE')) return 'BINANCE';
  if (environment.startsWith('BITGET')) return 'BITGET';
  return instrumentId.startsWith('crypto:') ? 'BINANCE' : 'XNAS';
}

function DraftRow({ draft, selected, onSelect }: { draft: OrderDraftSummary; selected: boolean; onSelect: () => void }) {
  return <button type="button" className={`order-draft-row${selected ? ' selected' : ''}`} aria-current={selected ? 'true' : undefined} onClick={onSelect}>
    <strong>{draft.instrumentId}</strong><small>{draft.environment} · v{draft.draftVersion}</small><time dateTime={draft.updatedAt}>{new Date(draft.updatedAt).toLocaleString()}</time>
  </button>;
}

function ProposalRow({ proposal, selected, onSelect }: { proposal: OrderProposalSummary; selected: boolean; onSelect: () => void }) {
  return <button type="button" className={`order-proposal-row${selected ? ' selected' : ''}`} aria-current={selected ? 'true' : undefined} onClick={onSelect}>
    <strong>{proposal.status}</strong><small>v{proposal.draftVersion} · {proposal.proposalId}</small><small>{proposal.proposalHash}</small><time dateTime={proposal.createdAt}>{new Date(proposal.createdAt).toLocaleString()}</time>
  </button>;
}

export function OrderDrafts({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const library = useQuery({ queryKey: ['order-drafts', workspaceId], queryFn: () => request('trade.draft.list', { workspaceId }), refetchOnMount: 'always' });
  const proposals = useQuery({ queryKey: ['order-proposals', workspaceId], queryFn: () => request('trade.proposal.list', { workspaceId }), refetchOnMount: 'always' });
  const accounts = useQuery({ queryKey: ['accounts', workspaceId, 'order-draft'], queryFn: () => request('account.list', { workspaceId }) });
  const catalog = useQuery({ queryKey: ['market-catalog', workspaceId, 'order-draft'], queryFn: () => request('market.catalog', { workspaceId, query: '', tier: marketTier }) });
  const [selectedId, setSelectedId] = useState<string>();
  const [selectedProposalId, setSelectedProposalId] = useState<string>();
  const [newMode, setNewMode] = useState(false);
  const [form, setForm] = useState<DraftForm>(initialForm);
  const [error, setError] = useState<unknown>();
  const [fieldError, setFieldError] = useState<{ field: DraftField; message: string }>();
  const [notice, setNotice] = useState('');
  const [proposalBusy, setProposalBusy] = useState(false);
  const [paperBusy, setPaperBusy] = useState(false);
  const [paperResult, setPaperResult] = useState<PaperOrderResult>();
  const [paperIdempotencyKey, setPaperIdempotencyKey] = useState<string>();
  const [paperCancelIdempotencyKey, setPaperCancelIdempotencyKey] = useState<string>();
  const [paperConfirmation, setPaperConfirmation] = useState<'submit' | 'cancel'>();
  const confirmationRef = useRef<HTMLDivElement>(null);
  const confirmationTriggerRef = useRef<HTMLElement | null>(null);
  const detail = useQuery({ queryKey: ['order-draft', workspaceId, selectedId], queryFn: () => request('trade.draft.get', { workspaceId, draftId: selectedId! }), enabled: Boolean(selectedId) && !newMode });
  const proposalDetail = useQuery({ queryKey: ['order-proposal', workspaceId, selectedProposalId], queryFn: () => request('trade.proposal.get', { workspaceId, proposalId: selectedProposalId! }), enabled: Boolean(selectedProposalId) });
  const selected = useMemo(() => newMode ? undefined : library.data?.drafts.find(draft => draft.draftId === selectedId), [library.data?.drafts, newMode, selectedId]);
  const selectedProposals = useMemo(() => proposals.data?.proposals.filter(proposal => proposal.draftId === selectedId) ?? [], [proposals.data?.proposals, selectedId]);
  const detailLoading = Boolean(selectedId) && !newMode && detail.isPending;

  useEffect(() => {
    if (!newMode && !selectedId && library.data?.drafts.length) setSelectedId(library.data.drafts[0].draftId);
    if (selectedId && library.data && !library.isFetching && !library.data.drafts.some(draft => draft.draftId === selectedId)) setSelectedId(undefined);
  }, [library.data, library.isFetching, newMode, selectedId]);
  useEffect(() => {
    if (!form.accountId && accounts.data) {
      const localPaper = accounts.data.accounts.find(account => account.providerId === 'local-paper' && account.environment === 'LOCAL');
      if (localPaper) update('accountId', localPaper.connectionId);
    }
  }, [accounts.data, form.accountId]);
  useEffect(() => { if (detail.data) setForm(fromDraft(detail.data)); }, [detail.data]);
  useEffect(() => {
    if (!selectedId || newMode || !selectedProposals.some(proposal => proposal.proposalId === selectedProposalId)) setSelectedProposalId(undefined);
  }, [newMode, selectedId, selectedProposalId, selectedProposals]);
  useEffect(() => {
    setPaperResult(undefined);
    setPaperIdempotencyKey(undefined);
    setPaperCancelIdempotencyKey(undefined);
  }, [selectedProposalId]);
  useEffect(() => {
    if (paperConfirmation) {
      queueMicrotask(() => confirmationRef.current?.querySelector<HTMLElement>('button:not(:disabled)')?.focus());
      return;
    }
    const trigger = confirmationTriggerRef.current;
    confirmationTriggerRef.current = null;
    queueMicrotask(() => { if (trigger?.isConnected) trigger.focus(); else document.getElementById('order-proposal-title')?.focus(); });
  }, [paperConfirmation]);

  const instruments = catalog.data?.instruments ?? [];
  const localErrors = [
    form.environment === 'LOCAL_PAPER' && !form.accountId ? 'Select the Local Paper account.' : '',
    !form.instrumentId ? 'Choose an instrument.' : '',
    !isPositiveDecimal(form.quantity) ? 'Quantity must be greater than zero.' : '',
    form.orderType === 'LIMIT' && !isPositiveDecimal(form.limitPrice) ? 'Limit price must be greater than zero.' : '',
  ].filter(Boolean);
  const update = <K extends keyof DraftForm>(key: K, value: DraftForm[K]) => {
    setFieldError(current => current?.field === key ? undefined : current);
    setForm(current => ({ ...current, [key]: value }));
  };
  const selectAccount = (accountId: string) => {
    const account = accounts.data?.accounts.find(item => item.connectionId === accountId);
    update('accountId', accountId);
    const environment = account && contextForAccount(account.providerId, account.environment);
    if (environment) update('environment', environment);
    if (environment) update('venue', expectedVenue(environment, form.instrumentId));
  };
  const newDraft = () => { setNewMode(true); setSelectedId(undefined); setForm(initialForm); setError(undefined); setFieldError(undefined); setNotice(''); };
  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (localErrors.length) return;
    setError(undefined); setFieldError(undefined); setNotice('');
    try {
      const fields: OrderDraftFields = {
        accountId: form.accountId || undefined, venue: form.venue, environment: form.environment,
        instrumentId: form.instrumentId, side: form.side, orderType: form.orderType,
        quantity: { type: form.quantityType, value: form.quantity },
        limitPrice: form.orderType === 'LIMIT' ? form.limitPrice : undefined,
        maximumSpend: form.maximumSpend || undefined, timeInForce: form.timeInForce,
        clientLabel: form.clientLabel.trim() || undefined,
      };
      const saved = await request('trade.save_draft', {
        workspaceId, ...(selected ? { draftId: selected.draftId, expectedStateVersion: selected.stateVersion } : {}), fields,
      });
      setNewMode(false); setSelectedId(saved.draftId); setForm(fromDraft(saved));
      await queryClient.invalidateQueries({ queryKey: ['order-drafts', workspaceId] });
      await queryClient.refetchQueries({ queryKey: ['order-drafts', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposals', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposal', workspaceId] });
      setNotice(`Draft saved at version ${saved.draftVersion}.`);
    } catch (cause) {
      setError(cause);
      if (cause instanceof CommandError) {
        const field = (cause.detail.field as DraftField | undefined) ?? ({
          ORDER_CONTEXT_INVALID: 'environment', ORDER_ACCOUNT_REQUIRED: 'accountId', ORDER_ACCOUNT_INVALID: 'accountId',
          ORDER_ACCOUNT_NOT_FOUND: 'accountId', ORDER_INSTRUMENT_NOT_FOUND: 'instrumentId',
          ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED: 'instrumentId', ORDER_VENUE_INVALID: 'venue',
          ORDER_DECIMAL_INVALID: 'quantity', ORDER_AMOUNT_INVALID: 'quantity', ORDER_LIMIT_PRICE_REQUIRED: 'limitPrice',
          ORDER_MARKET_PRICE_FORBIDDEN: 'orderType', ORDER_TIF_INVALID: 'timeInForce',
        } as Partial<Record<string, DraftField>>)[cause.detail.code];
        if (field) setFieldError({ field, message: cause.message });
      }
    }
  };

  const generateProposal = async () => {
    if (!selected) return;
    setError(undefined); setNotice('');
    try {
      const proposal = await request('trade.generate_proposal', { workspaceId, draftId: selected.draftId, expectedDraftVersion: selected.draftVersion });
      await queryClient.invalidateQueries({ queryKey: ['order-proposals', workspaceId] });
      setSelectedProposalId(proposal.proposalId);
      setNotice(`Proposal ${proposal.proposalId.slice(0, 16)}… generated and requires approval.`);
    } catch (cause) { setError(cause); }
  };

  const refreshProposal = async () => {
    if (!proposalDetail.data) return;
    setProposalBusy(true); setError(undefined); setNotice('');
    try {
      const result = await request('trade.refresh_proposal', {
        workspaceId,
        proposalId: proposalDetail.data.proposalId,
        expectedStateVersion: proposalDetail.data.stateVersion,
      });
      await queryClient.invalidateQueries({ queryKey: ['order-proposals', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposal', workspaceId] });
      setSelectedProposalId(result.proposal.proposalId);
      setNotice(`Proposal refreshed (${result.refreshStatus}); ${result.previousProposal.proposalId} → ${result.proposal.proposalId}; ${result.invalidationReason}`);
    } catch (cause) { setError(cause); }
    finally { setProposalBusy(false); }
  };

  const submitProposal = async () => {
    const proposal = proposalDetail.data;
    if (!proposal || proposal.fields.environment !== 'LOCAL_PAPER') return;
    setPaperBusy(true); setError(undefined); setNotice('');
    const idempotencyKey = paperIdempotencyKey ?? crypto.randomUUID();
    setPaperIdempotencyKey(idempotencyKey);
    try {
      const selectedProposal = await request('trade.proposal.get', {
        workspaceId,
        proposalId: proposal.proposalId,
      });
      const result = await request('paper.order.submit', {
        workspaceId,
        proposalId: proposal.proposalId,
        expectedProposalStateVersion: selectedProposal.stateVersion,
        idempotencyKey,
      });
      setPaperResult(result);
      await queryClient.invalidateQueries({ queryKey: ['paper', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['portfolio', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposals', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposal', workspaceId] });
      setNotice(`Local Paper order ${result.order.orderId} is ${result.order.state}.`);
    } catch (cause) { setError(cause); }
    finally { setPaperBusy(false); }
  };

  const cancelPaperOrder = async () => {
    if (!paperResult || !['ACCEPTED', 'PARTIALLY_FILLED'].includes(paperResult.order.state)) return;
    setPaperBusy(true); setError(undefined); setNotice('');
    const idempotencyKey = paperCancelIdempotencyKey ?? crypto.randomUUID();
    setPaperCancelIdempotencyKey(idempotencyKey);
    try {
      const result = await request('paper.order.cancel', {
        workspaceId,
        orderId: paperResult.order.orderId,
        expectedStateVersion: paperResult.stateVersion,
        idempotencyKey,
      });
      setPaperResult(result);
      await queryClient.invalidateQueries({ queryKey: ['paper', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['portfolio', workspaceId] });
      setNotice(`Local Paper order ${result.order.orderId} is ${result.order.state}.`);
    } catch (cause) { setError(cause); }
    finally { setPaperBusy(false); }
  };

  const openPaperConfirmation = (action: 'submit' | 'cancel') => {
    confirmationTriggerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setPaperConfirmation(action);
  };
  const confirmPaperAction = async () => {
    const action = paperConfirmation;
    if (!action) return;
    if (action === 'submit') await submitProposal();
    else await cancelPaperOrder();
    setPaperConfirmation(undefined);
  };

  if (library.isPending) return <p role="status">Loading order drafts…</p>;
  if (library.isError) return <div className="error-banner" role="alert"><div><strong>Order drafts are unavailable.</strong><p>{explainError(library.error)}</p></div><button type="button" onClick={() => void library.refetch()}>Reload drafts</button></div>;
  return <>
    <div className="page-heading"><h1>Order Drafts</h1><p>Save and reopen a versioned order draft. Saving never arms or submits an order.</p></div>
    {notice && <p className="notice" role="status" aria-live="polite">{notice}</p>}
    {error && <div className="error-banner" role="alert"><p>{explainError(error)}</p><button type="button" onClick={() => setError(undefined)}>Dismiss</button></div>}
    <div className="order-drafts-layout">
      <section className="card order-draft-list" aria-labelledby="order-draft-list-title"><div className="section-heading"><div><h2 id="order-draft-list-title">Saved drafts</h2><p className="muted">Workspace-local history</p></div><button type="button" onClick={newDraft}>New draft</button></div>{library.data.drafts.length ? library.data.drafts.map(draft => <DraftRow key={draft.draftId} draft={draft} selected={!newMode && draft.draftId === selectedId} onSelect={() => { setNewMode(false); setSelectedId(draft.draftId); }} />) : <p className="muted">No drafts saved yet.</p>}</section>
      <section className="card order-draft-editor" aria-labelledby="order-draft-editor-title"><div className="section-heading"><div><h2 id="order-draft-editor-title">{selected ? `Edit ${selected.instrumentId}` : 'New order draft'}</h2><p className="muted">Draft changes require the current state version.</p></div>{selected && <span className="badge">v{selected.draftVersion}</span>}</div>
        {detailLoading && <p role="status">Loading draft…</p>}
        {detail.isError && <p className="error-text" role="alert">{explainError(detail.error)}</p>}
        {!detailLoading && !detail.isError && <form onSubmit={save}>
          <div className="order-draft-grid">
            <label className="field">Execution context<select value={form.environment} aria-describedby={fieldError?.field === 'environment' ? 'order-field-error' : undefined} onChange={event => { const environment = event.target.value as ExecutionContext; update('environment', environment); update('venue', expectedVenue(environment, form.instrumentId)); }}>{environments.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}</select></label>
            <label className="field">Account<select value={form.accountId} aria-describedby={fieldError?.field === 'accountId' ? 'order-field-error' : undefined} onChange={event => selectAccount(event.target.value)}><option value="">Local Paper account</option>{(accounts.data?.accounts ?? []).map((account: AccountConnection) => <option key={account.connectionId} value={account.connectionId}>{account.label} · {account.providerId} · {account.environment}</option>)}</select></label>
            <label className="field">Instrument<select value={form.instrumentId} aria-describedby={fieldError?.field === 'instrumentId' ? 'order-field-error' : undefined} onChange={event => { update('instrumentId', event.target.value); update('venue', expectedVenue(form.environment, event.target.value)); }}>{instruments.map(instrument => <option key={instrument.instrumentId} value={instrument.instrumentId}>{instrument.symbol} · {instrument.instrumentId}</option>)}</select></label>
            <label className="field">Venue<select value={form.venue} aria-describedby={fieldError?.field === 'venue' ? 'order-field-error' : undefined} onChange={event => update('venue', event.target.value)}><option value="TRADEX_SIM">TRADEX_SIM</option><option value="XNAS">XNAS</option><option value="BINANCE">BINANCE</option><option value="BITGET">BITGET</option></select></label>
            <label className="field">Side<select value={form.side} onChange={event => update('side', event.target.value as OrderSide)}><option value="BUY">Buy</option><option value="SELL">Sell</option></select></label>
            <label className="field">Order type<select value={form.orderType} aria-describedby={fieldError?.field === 'orderType' ? 'order-field-error' : undefined} onChange={event => update('orderType', event.target.value as OrderType)}><option value="LIMIT">Limit</option><option value="MARKET">Market</option></select></label>
            <label className="field">Quantity type<select value={form.quantityType} onChange={event => update('quantityType', event.target.value as OrderQuantityType)}><option value="BASE">Base</option><option value="QUOTE">Quote</option></select></label>
            <label className="field">Quantity<input inputMode="decimal" value={form.quantity} aria-describedby={fieldError?.field === 'quantity' ? 'order-field-error' : undefined} onChange={event => update('quantity', event.target.value)} aria-invalid={!isPositiveDecimal(form.quantity)} /></label>
            <label className="field">Limit price<input inputMode="decimal" value={form.limitPrice} disabled={form.orderType === 'MARKET'} aria-describedby={fieldError?.field === 'limitPrice' ? 'order-field-error' : undefined} onChange={event => update('limitPrice', event.target.value)} aria-invalid={form.orderType === 'LIMIT' && !isPositiveDecimal(form.limitPrice)} /></label>
            <label className="field">Maximum spend <span className="muted">(optional)</span><input inputMode="decimal" value={form.maximumSpend} onChange={event => update('maximumSpend', event.target.value)} /></label>
            <label className="field">Time in force<select value={form.timeInForce} onChange={event => update('timeInForce', event.target.value as TimeInForce)}>{(['DAY', 'GTC', 'IOC', 'FOK'] as TimeInForce[]).map(value => <option key={value} value={value}>{value}</option>)}</select></label>
            <label className="field">Client label <span className="muted">(optional)</span><input maxLength={80} value={form.clientLabel} onChange={event => update('clientLabel', event.target.value)} /></label>
          </div>
          {fieldError && <p id="order-field-error" className="form-errors" role="alert">{fieldError.message}</p>}
          {localErrors.length > 0 && <ul className="form-errors" role="alert">{localErrors.map(message => <li key={message}>{message}</li>)}</ul>}
          {!catalog.isPending && !instruments.length && <p className="form-hint">Canonical market catalog is unavailable; reload before saving.</p>}
          <div className="form-actions"><button className="primary" disabled={Boolean(localErrors.length) || catalog.isPending || !instruments.length}>Save draft</button>{selected && <button type="button" onClick={() => void generateProposal()}>Generate proposal</button>}</div>
        </form>}
      </section>
    </div>
    <section className="card order-proposal-panel" aria-labelledby="order-proposal-title">
      <div className="section-heading"><div><h2 id="order-proposal-title" tabIndex={-1}>Order proposals</h2><p className="muted">Immutable read-only snapshots awaiting the later approval gate.</p></div>{selected && <span className="badge">{selectedProposals.length} for this draft</span>}</div>
      {proposals.isPending && <p role="status">Loading proposal history…</p>}
      {proposals.isError && <p className="error-text" role="alert">{explainError(proposals.error)}</p>}
      {!proposals.isPending && !proposals.isError && !selectedProposals.length && <p className="muted">Save a draft, then generate a proposal for its immutable snapshot.</p>}
      {selectedProposals.length > 0 && <div className="order-proposal-layout"><div className="order-proposal-list">{selectedProposals.map(proposal => <ProposalRow key={proposal.proposalId} proposal={proposal} selected={proposal.proposalId === selectedProposalId} onSelect={() => setSelectedProposalId(proposal.proposalId)} />)}</div><div className="order-proposal-detail">{!selectedProposalId && <p className="muted">Choose a proposal to inspect its details.</p>}{selectedProposalId && proposalDetail.isPending && <p role="status">Loading proposal…</p>}{selectedProposalId && proposalDetail.isError && <p className="error-text" role="alert">{explainError(proposalDetail.error)}</p>}{proposalDetail.data && <ProposalDetail proposal={proposalDetail.data} onRefresh={refreshProposal} refreshBusy={proposalBusy} onSubmit={() => openPaperConfirmation('submit')} onCancel={() => openPaperConfirmation('cancel')} submitBusy={paperBusy} cancelBusy={paperBusy} result={paperResult} />}</div></div>}
    </section>
    {paperConfirmation && <div className="picker-backdrop"><div className="picker-dialog" role="dialog" aria-modal="true" aria-labelledby="paper-confirm-title" ref={confirmationRef}><div className="picker-dialog-heading"><div><h2 id="paper-confirm-title">{paperConfirmation === 'submit' ? 'Confirm Local Paper submission' : 'Confirm Local Paper cancellation'}</h2><p className="muted">This changes the TradeX simulation only.</p></div></div><p>{paperConfirmation === 'submit' ? 'Submit the selected immutable proposal to Local Paper?' : 'Cancel the remaining quantity of this Local Paper order?'}</p><div className="picker-dialog-actions"><button type="button" onClick={() => setPaperConfirmation(undefined)} disabled={paperBusy}>Keep reviewing</button><button type="button" className="primary" onClick={() => void confirmPaperAction()} disabled={paperBusy}>{paperBusy ? 'Working…' : paperConfirmation === 'submit' ? 'Confirm submit' : 'Confirm cancel'}</button></div></div></div>}
  </>;
}

function ProposalDetail({ proposal, onRefresh, refreshBusy, onSubmit, onCancel, submitBusy, cancelBusy, result }: { proposal: OrderProposal; onRefresh: () => void; refreshBusy: boolean; onSubmit: () => void; onCancel: () => void; submitBusy: boolean; cancelBusy: boolean; result?: PaperOrderResult }) {
  return <div className="proposal-read-only" aria-label="Proposal detail">
    <div className="proposal-meta"><strong>{proposal.status}</strong><span>Draft v{proposal.draftVersion}</span><span>{proposal.proposalId}</span><span>{proposal.proposalHash}</span></div>
    <dl className="proposal-fields"><div><dt>Instrument</dt><dd>{proposal.fields.instrumentId}</dd></div><div><dt>Account</dt><dd>{proposal.fields.accountId ?? 'Local Paper account'}</dd></div><div><dt>Side / type</dt><dd>{proposal.fields.side} · {proposal.fields.orderType}</dd></div><div><dt>Quantity</dt><dd>{proposal.fields.quantity.value} {proposal.fields.quantity.type}</dd></div><div><dt>Limit price</dt><dd>{proposal.fields.limitPrice ?? '—'}</dd></div><div><dt>Maximum spend</dt><dd>{proposal.fields.maximumSpend ?? '—'}</dd></div><div><dt>Venue / context</dt><dd>{proposal.fields.venue} · {proposal.fields.environment}</dd></div><div><dt>Time in force</dt><dd>{proposal.fields.timeInForce}</dd></div><div><dt>Client label</dt><dd>{proposal.fields.clientLabel ?? '—'}</dd></div><div><dt>Estimated notional</dt><dd>{proposal.estimatedNotional ? `${proposal.estimatedNotional} ${proposal.estimatedNotionalCurrency ?? ''}` : proposal.estimatedNotionalReason ?? 'Unavailable'}</dd></div></dl>
    <p className="proposal-reference"><strong>Policy:</strong> {proposal.policyStatus} · v{proposal.policyVersion ?? '—'} · state {proposal.policyStateVersion ?? '—'} · {proposal.policyReferenceReason}</p>
    <p className="proposal-reference"><strong>Market:</strong> {proposal.marketStatus} · snapshot {proposal.marketSnapshotId ?? '—'} · {proposal.marketReferenceReason}</p>
    {proposal.invalidationReason && <p className="error-text">{proposal.invalidationReason}</p>}
    {proposal.status === 'NEEDS_APPROVAL' && <button type="button" onClick={onRefresh} disabled={refreshBusy}>{refreshBusy ? 'Refreshing proposal…' : 'Refresh proposal'}</button>}
    {proposal.status === 'NEEDS_APPROVAL' && proposal.fields.environment === 'LOCAL_PAPER' && <button type="button" className="primary" onClick={onSubmit} disabled={submitBusy}>{submitBusy ? 'Submitting Local Paper order…' : 'Submit Local Paper order'}</button>}
    {result?.proposalId === proposal.proposalId && <section className="notice" aria-label="Local Paper order result"><strong>TRADEX_SIMULATION · {result.order.state}</strong><p>{result.disclosure}</p><p>Order {result.order.orderId} · {result.order.filledQuantity} filled · {result.order.remainingQuantity} remaining · quote {result.quote.price} {result.quote.currency} · {result.quote.scenarioId}</p>{result.fill && <p>Fill {result.fill.fillId} · {result.fill.quantity} @ {result.fill.price} {result.fill.currency}</p>}<p>Proposal hash: {result.proposalHash}</p>{['ACCEPTED', 'PARTIALLY_FILLED'].includes(result.order.state) && <button type="button" onClick={onCancel} disabled={cancelBusy}>{cancelBusy ? 'Cancelling Local Paper order…' : 'Cancel Local Paper order'}</button>}</section>}
    <h3>History</h3><ol className="proposal-history">{proposal.history.map((entry, index) => <li key={`${entry.event}-${entry.occurredAt}-${index}`}><strong>{entry.event}</strong><time dateTime={entry.occurredAt}>{new Date(entry.occurredAt).toLocaleString()}</time>{entry.reason && <span>{entry.reason}</span>}</li>)}</ol>
  </div>;
}
