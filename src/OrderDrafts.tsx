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
  AlpacaPaperOrderAttempt,
  AlpacaPaperOrder,
  AlpacaPaperOrderBook,
  OrderQuantityType,
  OrderSide,
  OrderType,
  TimeInForce,
  TradeXError,
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

function isOpenAlpacaOrder(order: AlpacaPaperOrder) {
  return ['new', 'accepted', 'pending_new', 'partially_filled', 'pending_cancel'].includes(order.providerStatus);
}

function canCancelAlpacaOrder(order: AlpacaPaperOrder) {
  return ['new', 'accepted', 'pending_new', 'partially_filled'].includes(order.providerStatus)
    && order.cancelState === 'NONE';
}

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

function localGuardError(code: string, message: string) {
  return new CommandError({
    category: code === 'PROVIDER_UNSUPPORTED' ? 'UNSUPPORTED_CAPABILITY' : 'STATE_STALE',
    code,
    message,
    retryable: false,
    blocking: true,
    remediationActions: [{ id: 'reload_snapshot', label: 'Reload state' }],
  } satisfies TradeXError);
}

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
  const alpacaAccounts = useMemo(() => accounts.data?.accounts.filter(account => account.providerId === 'alpaca' && account.environment === 'PAPER') ?? [], [accounts.data?.accounts]);
  const [ordersConnectionId, setOrdersConnectionId] = useState('');
  const ordersQuery = useQuery({
    queryKey: ['alpaca-paper-orders', workspaceId, ordersConnectionId],
    queryFn: () => request('alpaca.paper.orders.get', { workspaceId, connectionId: ordersConnectionId }),
    enabled: Boolean(ordersConnectionId),
    refetchOnMount: 'always',
  });
  const ordersAccount = alpacaAccounts.find(account => account.connectionId === ordersConnectionId);
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
  const [alpacaIdempotencyKey, setAlpacaIdempotencyKey] = useState<string>();
  const [paperConfirmation, setPaperConfirmation] = useState<'submit' | 'cancel' | 'alpaca-submit' | 'alpaca-cancel'>();
  const [ordersBusy, setOrdersBusy] = useState(false);
  const [cancelReview, setCancelReview] = useState<{ book: AlpacaPaperOrderBook; order: AlpacaPaperOrder }>();
  const confirmationRef = useRef<HTMLDivElement>(null);
  const confirmationTriggerRef = useRef<HTMLElement | null>(null);
  const detail = useQuery({ queryKey: ['order-draft', workspaceId, selectedId], queryFn: () => request('trade.draft.get', { workspaceId, draftId: selectedId! }), enabled: Boolean(selectedId) && !newMode });
  const proposalDetail = useQuery({ queryKey: ['order-proposal', workspaceId, selectedProposalId], queryFn: () => request('trade.proposal.get', { workspaceId, proposalId: selectedProposalId! }), enabled: Boolean(selectedProposalId) });
  const alpacaAttempt = useQuery({
    queryKey: ['alpaca-paper-attempt', workspaceId, selectedProposalId],
    queryFn: () => request('alpaca.paper.order.attempt.get', { workspaceId, proposalId: selectedProposalId! }),
    enabled: Boolean(selectedProposalId) && proposalDetail.data?.fields.environment === 'ALPACA_PAPER',
    refetchOnMount: 'always',
  });
  const selected = useMemo(() => newMode ? undefined : library.data?.drafts.find(draft => draft.draftId === selectedId), [library.data?.drafts, newMode, selectedId]);
  const selectedProposals = useMemo(() => proposals.data?.proposals.filter(proposal => proposal.draftId === selectedId) ?? [], [proposals.data?.proposals, selectedId]);
  const detailLoading = Boolean(selectedId) && !newMode && detail.isPending;
  const orderBook = ordersQuery.data?.book;

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
  useEffect(() => {
    if (!ordersConnectionId && alpacaAccounts.length) setOrdersConnectionId(alpacaAccounts[0].connectionId);
    if (ordersConnectionId && accounts.data && !alpacaAccounts.some(account => account.connectionId === ordersConnectionId)) setOrdersConnectionId('');
  }, [accounts.data, alpacaAccounts, ordersConnectionId]);
  useEffect(() => { if (detail.data) setForm(fromDraft(detail.data)); }, [detail.data]);
  useEffect(() => {
    if (!selectedId || newMode || !selectedProposals.some(proposal => proposal.proposalId === selectedProposalId)) setSelectedProposalId(undefined);
  }, [newMode, selectedId, selectedProposalId, selectedProposals]);
  useEffect(() => {
    setPaperResult(undefined);
    setPaperIdempotencyKey(undefined);
    setPaperCancelIdempotencyKey(undefined);
    setAlpacaIdempotencyKey(undefined);
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

  const submitAlpacaProposal = async () => {
    const proposal = proposalDetail.data;
    if (!proposal || proposal.fields.environment !== 'ALPACA_PAPER' || !proposal.fields.accountId) return;
    setPaperBusy(true); setError(undefined); setNotice('');
    const idempotencyKey = alpacaIdempotencyKey ?? crypto.randomUUID();
    setAlpacaIdempotencyKey(idempotencyKey);
    try {
      const prior = await request('alpaca.paper.order.attempt.get', { workspaceId, proposalId: proposal.proposalId });
      if (prior.attempt) {
        await queryClient.invalidateQueries({ queryKey: ['alpaca-paper-attempt', workspaceId, proposal.proposalId] });
        setNotice('A saved Alpaca Paper attempt already exists. Its status is shown below; no second order was sent.');
        return;
      }
      const [currentProposal, account] = await Promise.all([
        request('trade.proposal.get', { workspaceId, proposalId: proposal.proposalId }),
        request('account.get', { workspaceId, connectionId: proposal.fields.accountId }),
      ]);
      if (currentProposal.proposalHash !== proposal.proposalHash
        || currentProposal.stateVersion !== proposal.stateVersion
        || currentProposal.status !== 'NEEDS_APPROVAL') {
        throw localGuardError('STATE_VERSION_CONFLICT', 'The reviewed Proposal changed. Reload it before submitting.');
      }
      if (account.providerId !== 'alpaca' || account.environment !== 'PAPER') {
        throw localGuardError('PROVIDER_UNSUPPORTED', 'This Proposal is not bound to an Alpaca Paper account.');
      }
      const attempt = await request('alpaca.paper.order.submit', {
        workspaceId,
        connectionId: account.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        proposalId: currentProposal.proposalId,
        expectedProposalStateVersion: currentProposal.stateVersion,
        proposalHash: currentProposal.proposalHash,
        idempotencyKey,
        confirmedPaperOrder: true,
      });
      await queryClient.invalidateQueries({ queryKey: ['alpaca-paper-attempt', workspaceId, proposal.proposalId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposals', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposal', workspaceId, proposal.proposalId] });
      setNotice(`Alpaca Paper attempt ${attempt.state}. A provider acknowledgement does not mean the order filled.`);
    } catch (cause) { setError(cause); }
    finally { setPaperBusy(false); }
  };

  const reconcileAlpacaAttempt = async (attempt: AlpacaPaperOrderAttempt) => {
    setPaperBusy(true); setError(undefined); setNotice('');
    try {
      const account = await request('account.get', { workspaceId, connectionId: attempt.connectionId });
      if (account.providerId !== 'alpaca' || account.environment !== 'PAPER') {
        throw localGuardError('PROVIDER_UNSUPPORTED', 'Reconciliation is limited to the saved Alpaca Paper account.');
      }
      const result = await request('alpaca.paper.order.reconcile', {
        workspaceId,
        connectionId: attempt.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        proposalId: attempt.proposalId,
      });
      await queryClient.invalidateQueries({ queryKey: ['alpaca-paper-attempt', workspaceId, attempt.proposalId] });
      setNotice(`Alpaca Paper attempt ${result.state}. No order was resubmitted.`);
    } catch (cause) { setError(cause); }
    finally { setPaperBusy(false); }
  };

  const refreshAlpacaOrders = async () => {
    if (!ordersAccount) return;
    setOrdersBusy(true); setError(undefined); setNotice('');
    try {
      const account = await request('account.get', { workspaceId, connectionId: ordersAccount.connectionId });
      const result = await request('alpaca.paper.orders.refresh', {
        workspaceId, connectionId: account.connectionId, expectedConnectionStateVersion: account.stateVersion,
      });
      queryClient.setQueryData(['alpaca-paper-orders', workspaceId, account.connectionId], { book: result });
      setNotice(result.status === 'CURRENT'
        ? `Alpaca Paper orders and fills refreshed at ${new Date(result.lastSuccessfulSyncAt ?? result.observedAt).toLocaleString()}.`
        : `Alpaca Paper data is incomplete: ${result.reason ?? 'provider response was not complete'}. Existing observations were kept.`);
    } catch (cause) { setError(cause); }
    finally { setOrdersBusy(false); }
  };

  const reviewAlpacaOrder = async (order: AlpacaPaperOrder) => {
    if (!ordersAccount) return;
    confirmationTriggerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setOrdersBusy(true); setError(undefined); setNotice('');
    try {
      const account = await request('account.get', { workspaceId, connectionId: ordersAccount.connectionId });
      const book = await request('alpaca.paper.order.review', {
        workspaceId, connectionId: account.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        providerOrderId: order.providerOrderId,
      });
      queryClient.setQueryData(['alpaca-paper-orders', workspaceId, account.connectionId], { book });
      if (book.status === 'DEGRADED') throw localGuardError('ORDER_STATUS_UNKNOWN', 'Alpaca could not verify the selected order. Refresh and review again before requesting cancellation.');
      const reviewed = book.orders.find(item => item.providerOrderId === order.providerOrderId);
      if (!reviewed) throw localGuardError('ORDER_STATUS_UNKNOWN', 'The provider order is no longer in the current account history. Refresh and review again.');
      setCancelReview({ book, order: reviewed });
      setPaperConfirmation('alpaca-cancel');
    } catch (cause) { setError(cause); }
    finally { setOrdersBusy(false); }
  };

  const cancelAlpacaOrder = async () => {
    if (!ordersAccount || !cancelReview) return;
    setOrdersBusy(true); setError(undefined); setNotice('');
    try {
      const account = await request('account.get', { workspaceId, connectionId: ordersAccount.connectionId });
      const book = await request('alpaca.paper.order.cancel', {
        workspaceId,
        connectionId: account.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        providerOrderId: cancelReview.order.providerOrderId,
        expectedBookStateVersion: cancelReview.book.stateVersion,
        idempotencyKey: crypto.randomUUID(),
        confirmed: true,
      });
      setCancelReview(undefined);
      queryClient.setQueryData(['alpaca-paper-orders', workspaceId, account.connectionId], { book });
      const updated = book.orders.find(order => order.providerOrderId === cancelReview.order.providerOrderId);
      setNotice(updated?.cancelState === 'PENDING'
        ? 'Alpaca accepted the cancellation request. Status is CANCEL_PENDING until the provider confirms the order state.'
        : `Alpaca Paper provider status: ${updated?.providerStatus ?? 'unknown'}.`);
    } catch (cause) { setError(cause); }
    finally { setOrdersBusy(false); }
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

  const openPaperConfirmation = (action: 'submit' | 'cancel' | 'alpaca-submit' | 'alpaca-cancel') => {
    confirmationTriggerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setPaperConfirmation(action);
  };
  const confirmPaperAction = async () => {
    const action = paperConfirmation;
    if (!action) return;
    if (action === 'submit') await submitProposal();
    else if (action === 'alpaca-submit') await submitAlpacaProposal();
    else if (action === 'alpaca-cancel') await cancelAlpacaOrder();
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
      {selectedProposals.length > 0 && <div className="order-proposal-layout"><div className="order-proposal-list">{selectedProposals.map(proposal => <ProposalRow key={proposal.proposalId} proposal={proposal} selected={proposal.proposalId === selectedProposalId} onSelect={() => setSelectedProposalId(proposal.proposalId)} />)}</div><div className="order-proposal-detail">{!selectedProposalId && <p className="muted">Choose a proposal to inspect its details.</p>}{selectedProposalId && proposalDetail.isPending && <p role="status">Loading proposal…</p>}{selectedProposalId && proposalDetail.isError && <p className="error-text" role="alert">{explainError(proposalDetail.error)}</p>}{proposalDetail.data && <ProposalDetail proposal={proposalDetail.data} onRefresh={refreshProposal} refreshBusy={proposalBusy} onSubmit={() => openPaperConfirmation('submit')} onCancel={() => openPaperConfirmation('cancel')} onAlpacaSubmit={() => openPaperConfirmation('alpaca-submit')} onAlpacaReconcile={reconcileAlpacaAttempt} onReloadAlpacaAttempt={() => void alpacaAttempt.refetch()} alpacaAttempt={alpacaAttempt.data?.attempt ?? undefined} alpacaAttemptLoading={alpacaAttempt.isPending} alpacaAttemptError={alpacaAttempt.error} alpacaAccount={accounts.data?.accounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)} submitBusy={paperBusy} cancelBusy={paperBusy} result={paperResult} />}</div></div>}
    </section>
    <section className="card order-book-panel" aria-labelledby="alpaca-order-book-title">
      <div className="section-heading">
        <div><h2 id="alpaca-order-book-title">Alpaca Paper orders and fills</h2><p className="muted">Provider observations · ALPACA_PAPER only</p></div>
        <div className="order-book-actions">
          <label className="field">Paper account<select aria-label="Alpaca Paper account" value={ordersConnectionId} onChange={event => setOrdersConnectionId(event.target.value)}><option value="">Select an Alpaca Paper account</option>{alpacaAccounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.label} · {account.data?.remoteAccountId ?? account.connectionId}</option>)}</select></label>
          <button type="button" onClick={() => void refreshAlpacaOrders()} disabled={!ordersAccount || ordersAccount.connectionState !== 'CONNECTED' || ordersBusy}>{ordersBusy ? 'Checking Alpaca…' : 'Refresh orders and fills'}</button>
        </div>
      </div>
      {accounts.isPending && <p role="status">Loading linked accounts…</p>}
      {!accounts.isPending && !alpacaAccounts.length && <p className="muted">Connect an Alpaca Paper account to view its provider orders and fills.</p>}
      {ordersAccount && ordersAccount.connectionState !== 'CONNECTED' && <p className="error-text" role="status">This Alpaca Paper account is disconnected. Reconnect it before refreshing provider observations.</p>}
      {ordersQuery.isPending && ordersConnectionId && <p role="status">Loading saved Alpaca Paper observations…</p>}
      {ordersQuery.isError && <div className="error-text" role="alert"><p>Saved Alpaca Paper orders are unavailable.</p><button type="button" onClick={() => void ordersQuery.refetch()}>Reload saved orders</button></div>}
      {orderBook && <>
        <p className={orderBook.status === 'DEGRADED' || orderBook.status === 'STALE' ? 'error-text' : 'muted'} role={orderBook.status === 'DEGRADED' ? 'alert' : 'status'}>
          {orderBook.status === 'CURRENT' ? `Current provider observations · received ${new Date(orderBook.observedAt).toLocaleString()}` : orderBook.status === 'NEVER_SYNCED' ? 'No provider read has completed; refresh to load orders and fills.' : `Provider read is ${orderBook.status.toLowerCase()}: ${orderBook.reason ?? 'showing the last saved observations'}.`}
          {orderBook.lastSuccessfulSyncAt && ` Last complete sync: ${new Date(orderBook.lastSuccessfulSyncAt).toLocaleString()}.`}
        </p>
        <h3>Open orders ({orderBook.orders.filter(isOpenAlpacaOrder).length})</h3>
        {!orderBook.orders.some(isOpenAlpacaOrder) && orderBook.status === 'CURRENT' && <p className="muted">No open Alpaca Paper orders were returned.</p>}
        {orderBook.orders.filter(isOpenAlpacaOrder).map(order => <article className="order-book-order" key={order.providerOrderId}>
          <div><strong>{order.symbol} · {order.side.toUpperCase()} · {order.providerStatus}</strong><small>ALPACA_PAPER · {order.origin === 'TRADE_X' ? 'TradeX proposal' : 'External provider order'} · observed {new Date(order.observedAt).toLocaleString()}</small></div>
          <dl className="order-book-facts"><div><dt>Provider order</dt><dd>{order.providerOrderId}</dd></div><div><dt>Instrument</dt><dd>{order.instrumentId ?? `${order.symbol} · canonical mapping unavailable`}</dd></div><div><dt>Filled / remaining</dt><dd>{order.filledQuantity} / {order.remainingQuantity ?? 'unavailable'}</dd></div><div><dt>Provider state</dt><dd>{order.cancelState === 'PENDING' ? 'CANCEL_PENDING · provider confirmation required' : order.cancelState === 'SUBMITTING' ? 'Cancellation is being checked' : order.providerStatus}</dd></div></dl>
          {order.cancelError && <p className="error-text" role="status">{order.cancelError}</p>}
          {canCancelAlpacaOrder(order) && <button type="button" onClick={() => void reviewAlpacaOrder(order)} disabled={ordersBusy}>{ordersBusy ? 'Refreshing selected order…' : 'Review cancellation'}</button>}
          {order.cancelState !== 'NONE' && <button type="button" disabled>Cancellation pending provider confirmation</button>}
        </article>)}
        <h3>Order history ({orderBook.orders.filter(order => !isOpenAlpacaOrder(order)).length})</h3>
        {orderBook.orders.filter(order => !isOpenAlpacaOrder(order)).map(order => <article className="order-book-order" key={order.providerOrderId}>
          <div><strong>{order.symbol} · {order.side.toUpperCase()} · {order.providerStatus}</strong><small>ALPACA_PAPER · {order.origin === 'TRADE_X' ? 'TradeX proposal' : 'External provider order'} · observed {new Date(order.observedAt).toLocaleString()}</small></div>
          <dl className="order-book-facts"><div><dt>Provider order</dt><dd>{order.providerOrderId}</dd></div><div><dt>Instrument</dt><dd>{order.instrumentId ?? `${order.symbol} · canonical mapping unavailable`}</dd></div><div><dt>Filled / remaining</dt><dd>{order.filledQuantity} / {order.remainingQuantity ?? 'unavailable'}</dd></div></dl>
        </article>)}
        {!orderBook.fills.length && orderBook.status === 'CURRENT' && <p className="muted">No Alpaca FILL activities were returned.</p>}
        <h3>Fills ({orderBook.fills.length})</h3>
        {orderBook.fills.map(fill => <article className="order-book-fill" key={fill.activityId}>
          <strong>{fill.symbol} · {fill.side.toUpperCase()} · {fill.quantity} @ {fill.price}</strong>
          <small>FILL activity {fill.activityId} · order {fill.providerOrderId} · executed {new Date(fill.executedAt).toLocaleString()}</small>
          <small>ALPACA_PAPER · REST activity · received {new Date(fill.observedAt).toLocaleString()}</small>
        </article>)}
      </>}
    </section>
    {paperConfirmation && <div className="picker-backdrop"><div className="picker-dialog" role="dialog" aria-modal="true" aria-labelledby="paper-confirm-title" ref={confirmationRef}><div className="picker-dialog-heading"><div><h2 id="paper-confirm-title">{paperConfirmation === 'alpaca-submit' ? 'Confirm Alpaca Paper submission' : paperConfirmation === 'alpaca-cancel' ? 'Confirm Alpaca Paper cancellation' : paperConfirmation === 'submit' ? 'Confirm Local Paper submission' : 'Confirm Local Paper cancellation'}</h2><p className="muted">{paperConfirmation === 'alpaca-submit' ? 'This sends the exact Proposal to Alpaca Paper simulation only. It never uses a Live endpoint.' : paperConfirmation === 'alpaca-cancel' ? 'The account and order were just reread from Alpaca Paper. Review the exact filled and remaining quantities before confirming.' : 'This changes the TradeX simulation only.'}</p></div></div><p>{paperConfirmation === 'alpaca-submit' ? 'Submit the exact Alpaca Paper Proposal shown below?' : paperConfirmation === 'alpaca-cancel' ? 'Send a cancellation request for this exact Alpaca Paper order?' : paperConfirmation === 'submit' ? 'Submit the selected immutable proposal to Local Paper?' : 'Cancel the remaining quantity of this Local Paper order?'}</p>{paperConfirmation === 'alpaca-cancel' && cancelReview && <dl className="proposal-fields"><div><dt>Environment / account</dt><dd>ALPACA_PAPER · {ordersAccount?.label ?? 'Unavailable'} · {cancelReview.book.remoteAccountId}</dd></div><div><dt>Provider order</dt><dd>{cancelReview.order.providerOrderId}</dd></div><div><dt>Instrument / side</dt><dd>{cancelReview.order.instrumentId ?? cancelReview.order.symbol} · {cancelReview.order.side.toUpperCase()}</dd></div><div><dt>Provider status</dt><dd>{cancelReview.order.providerStatus}</dd></div><div><dt>Filled quantity</dt><dd>{cancelReview.order.filledQuantity}</dd></div><div><dt>Remaining quantity</dt><dd>{cancelReview.order.remainingQuantity ?? 'Unavailable'}</dd></div><div><dt>Last observation</dt><dd>{new Date(cancelReview.order.observedAt).toLocaleString()}</dd></div></dl>}{paperConfirmation === 'alpaca-submit' && proposalDetail.data && <dl className="proposal-fields"><div><dt>Paper account</dt><dd>{accounts.data?.accounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)?.label ?? 'Unavailable'} · {accounts.data?.accounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)?.data?.remoteAccountId ?? 'provider account ID unavailable'}</dd></div><div><dt>Instrument / side</dt><dd>{proposalDetail.data.fields.instrumentId} · {proposalDetail.data.fields.side}</dd></div><div><dt>Quantity / order</dt><dd>{proposalDetail.data.fields.quantity.value} {proposalDetail.data.fields.quantity.type} · {proposalDetail.data.fields.orderType} · {proposalDetail.data.fields.timeInForce}</dd></div><div><dt>Limit</dt><dd>{proposalDetail.data.fields.limitPrice ?? '—'}</dd></div><div><dt>Proposal</dt><dd>{proposalDetail.data.proposalId} · {proposalDetail.data.proposalHash}</dd></div></dl>}<div className="picker-dialog-actions"><button type="button" onClick={() => { setPaperConfirmation(undefined); setCancelReview(undefined); }} disabled={paperBusy || ordersBusy}>Keep reviewing</button><button type="button" className="primary" onClick={() => void confirmPaperAction()} disabled={paperBusy || ordersBusy}>{paperBusy || ordersBusy ? 'Working…' : paperConfirmation === 'alpaca-submit' ? 'Confirm Alpaca Paper submit' : paperConfirmation === 'alpaca-cancel' ? 'Confirm cancellation request' : paperConfirmation === 'submit' ? 'Confirm submit' : 'Confirm cancel'}</button></div></div></div>}
  </>;
}

function ProposalDetail({ proposal, onRefresh, refreshBusy, onSubmit, onCancel, onAlpacaSubmit, onAlpacaReconcile, onReloadAlpacaAttempt, alpacaAttempt, alpacaAttemptLoading, alpacaAttemptError, alpacaAccount, submitBusy, cancelBusy, result }: {
  proposal: OrderProposal;
  onRefresh: () => void;
  refreshBusy: boolean;
  onSubmit: () => void;
  onCancel: () => void;
  onAlpacaSubmit: () => void;
  onAlpacaReconcile: (attempt: AlpacaPaperOrderAttempt) => void;
  onReloadAlpacaAttempt: () => void;
  alpacaAttempt?: AlpacaPaperOrderAttempt;
  alpacaAttemptLoading: boolean;
  alpacaAttemptError: unknown;
  alpacaAccount?: AccountConnection;
  submitBusy: boolean;
  cancelBusy: boolean;
  result?: PaperOrderResult;
}) {
  const alpacaReady = alpacaAccount?.providerId === 'alpaca'
    && alpacaAccount.environment === 'PAPER'
    && alpacaAccount.connectionState === 'CONNECTED';
  return <div className="proposal-read-only" aria-label="Proposal detail">
    <div className="proposal-meta"><strong>{proposal.status}</strong><span>Draft v{proposal.draftVersion}</span><span>{proposal.proposalId}</span><span>{proposal.proposalHash}</span></div>
    <dl className="proposal-fields"><div><dt>Instrument</dt><dd>{proposal.fields.instrumentId}</dd></div><div><dt>Account</dt><dd>{proposal.fields.accountId ?? 'Local Paper account'}</dd></div><div><dt>Side / type</dt><dd>{proposal.fields.side} · {proposal.fields.orderType}</dd></div><div><dt>Quantity</dt><dd>{proposal.fields.quantity.value} {proposal.fields.quantity.type}</dd></div><div><dt>Limit price</dt><dd>{proposal.fields.limitPrice ?? '—'}</dd></div><div><dt>Maximum spend</dt><dd>{proposal.fields.maximumSpend ?? '—'}</dd></div><div><dt>Venue / context</dt><dd>{proposal.fields.venue} · {proposal.fields.environment}</dd></div><div><dt>Time in force</dt><dd>{proposal.fields.timeInForce}</dd></div><div><dt>Client label</dt><dd>{proposal.fields.clientLabel ?? '—'}</dd></div><div><dt>Estimated notional</dt><dd>{proposal.estimatedNotional ? `${proposal.estimatedNotional} ${proposal.estimatedNotionalCurrency ?? ''}` : proposal.estimatedNotionalReason ?? 'Unavailable'}</dd></div></dl>
    <p className="proposal-reference"><strong>Policy:</strong> {proposal.policyStatus} · v{proposal.policyVersion ?? '—'} · state {proposal.policyStateVersion ?? '—'} · {proposal.policyReferenceReason}</p>
    <p className="proposal-reference"><strong>Market:</strong> {proposal.marketStatus} · snapshot {proposal.marketSnapshotId ?? '—'} · {proposal.marketReferenceReason}</p>
    {proposal.invalidationReason && <p className="error-text">{proposal.invalidationReason}</p>}
    {proposal.status === 'NEEDS_APPROVAL' && <button type="button" onClick={onRefresh} disabled={refreshBusy}>{refreshBusy ? 'Refreshing proposal…' : 'Refresh proposal'}</button>}
    {proposal.status === 'NEEDS_APPROVAL' && proposal.fields.environment === 'LOCAL_PAPER' && <button type="button" className="primary" onClick={onSubmit} disabled={submitBusy}>{submitBusy ? 'Submitting Local Paper order…' : 'Submit Local Paper order'}</button>}
    {proposal.fields.environment === 'ALPACA_PAPER' && alpacaAttemptLoading && <p role="status">Loading saved Alpaca Paper attempt…</p>}
    {proposal.fields.environment === 'ALPACA_PAPER' && Boolean(alpacaAttemptError) && <div className="error-text" role="alert"><p>Saved Alpaca Paper attempt is unavailable.</p><button type="button" onClick={onReloadAlpacaAttempt}>Reload attempt</button></div>}
    {proposal.fields.environment === 'ALPACA_PAPER' && alpacaAttempt && <section className="notice" aria-label="Alpaca Paper order attempt"><strong>ALPACA_PAPER · {alpacaAttempt.state}</strong><p>{alpacaAttempt.reason}</p><p>Paper account {alpacaAttempt.remoteAccountId} · client order {alpacaAttempt.clientOrderId}</p>{alpacaAttempt.providerOrderId && <p>Provider order {alpacaAttempt.providerOrderId} · provider status {alpacaAttempt.providerStatus ?? 'Unavailable'}</p>}{alpacaAttempt.state === 'ACKNOWLEDGED' && <p>Alpaca acknowledged the order. This is not fill evidence.</p>}{alpacaAttempt.state === 'SUBMITTING' && <p role="status">Submission is still pending. Reload this attempt to check its saved state.</p>}{alpacaAttempt.state === 'UNKNOWN_RECONCILING' && <button type="button" onClick={() => onAlpacaReconcile(alpacaAttempt)} disabled={submitBusy}>{submitBusy ? 'Checking Alpaca Paper…' : 'Reconcile by client order ID'}</button>}</section>}
    {proposal.status === 'NEEDS_APPROVAL' && proposal.fields.environment === 'ALPACA_PAPER' && !alpacaAttempt && !alpacaAttemptLoading && !alpacaAttemptError && <>{alpacaReady ? <button type="button" className="primary" onClick={onAlpacaSubmit} disabled={submitBusy}>{submitBusy ? 'Submitting Alpaca Paper order…' : 'Submit Alpaca Paper order'}</button> : <p className="muted">Connect and confirm this Alpaca Paper account before submitting the Proposal.</p>}</>}
    {result?.proposalId === proposal.proposalId && <section className="notice" aria-label="Local Paper order result"><strong>TRADEX_SIMULATION · {result.order.state}</strong><p>{result.disclosure}</p><p>Order {result.order.orderId} · {result.order.filledQuantity} filled · {result.order.remainingQuantity} remaining · quote {result.quote.price} {result.quote.currency} · {result.quote.scenarioId}</p>{result.fill && <p>Fill {result.fill.fillId} · {result.fill.quantity} @ {result.fill.price} {result.fill.currency}</p>}<p>Proposal hash: {result.proposalHash}</p>{['ACCEPTED', 'PARTIALLY_FILLED'].includes(result.order.state) && <button type="button" onClick={onCancel} disabled={cancelBusy}>{cancelBusy ? 'Cancelling Local Paper order…' : 'Cancel Local Paper order'}</button>}</section>}
    <h3>History</h3><ol className="proposal-history">{proposal.history.map((entry, index) => <li key={`${entry.event}-${entry.occurredAt}-${index}`}><strong>{entry.event}</strong><time dateTime={entry.occurredAt}>{new Date(entry.occurredAt).toLocaleString()}</time>{entry.reason && <span>{entry.reason}</span>}</li>)}</ol>
  </div>;
}
