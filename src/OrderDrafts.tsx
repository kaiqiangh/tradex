import { useEffect, useMemo, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { createPortal } from 'react-dom';
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
  Trading212DemoOrderAttempt,
  Trading212DemoOrder,
  Trading212DemoOrderBook,
  Trading212DemoOrderBookAction,
  AlpacaPaperOrderAttempt,
  AlpacaPaperOrder,
  AlpacaPaperOrderBook,
  BinanceTestnetOrderAttempt,
  BinanceTestnetOrder,
  BinanceTestnetOrderBookAction,
  OrderQuantityType,
  OrderSide,
  OrderType,
  TimeInForce,
  TradeXError,
} from '../shared/ipc-types.ts';
import { CommandError, explainError, request } from './client.ts';
import { fromAccountSnapshot } from './projection.ts';
import { useDomainProjection } from './useDomainProjection.ts';

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

function retryLabel(value?: string | null) {
  if (value == null) return 'ready';
  const time = Date.parse(value);
  return Number.isFinite(time) ? time <= Date.now() ? 'ready' : 'after ' + new Date(time).toLocaleTimeString() : 'unavailable';
}

function isOpenAlpacaOrder(order: AlpacaPaperOrder) {
  return ['new', 'accepted', 'pending_new', 'partially_filled', 'pending_cancel'].includes(order.providerStatus);
}

function canCancelAlpacaOrder(order: AlpacaPaperOrder) {
  return ['new', 'accepted', 'pending_new', 'partially_filled'].includes(order.providerStatus)
    && order.cancelState === 'NONE';
}

function canCancelTrading212Order(order: Trading212DemoOrder) {
  return ['CONFIRMED', 'NEW', 'PARTIALLY_FILLED'].includes(order.providerStatus)
    && (order.cancelState ?? 'NONE') === 'NONE';
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
type PaperConfirmation = 'submit' | 'cancel' | 'alpaca-submit' | 'alpaca-cancel' | 'trading212-submit' | 'trading212-cancel' | 'binance-testnet-submit';

const paperConfirmationCopy: Record<PaperConfirmation, { title: string; explanation: string; prompt: string; confirmLabel: string }> = {
  submit: {
    title: 'Confirm Local Paper submission', explanation: 'This changes the TradeX simulation only.',
    prompt: 'Submit the selected immutable proposal to Local Paper?', confirmLabel: 'Confirm submit',
  },
  cancel: {
    title: 'Confirm Local Paper cancellation', explanation: 'This changes the TradeX simulation only.',
    prompt: 'Cancel the remaining quantity of this Local Paper order?', confirmLabel: 'Confirm cancel',
  },
  'alpaca-submit': {
    title: 'Confirm Alpaca Paper submission', explanation: 'This sends the exact Proposal to Alpaca Paper simulation only. It never uses a Live endpoint.',
    prompt: 'Submit the exact Alpaca Paper Proposal shown below?', confirmLabel: 'Confirm Alpaca Paper submit',
  },
  'alpaca-cancel': {
    title: 'Confirm Alpaca Paper cancellation', explanation: 'The account and order were just reread from Alpaca Paper. Review the exact filled and remaining quantities before confirming.',
    prompt: 'Send a cancellation request for this exact Alpaca Paper order?', confirmLabel: 'Confirm cancellation request',
  },
  'trading212-submit': {
    title: 'Confirm Trading 212 Demo submission', explanation: 'This sends one order to the connected Trading 212 Demo account. It never uses a Live endpoint; an acknowledgement is not a fill.',
    prompt: 'Submit this exact immutable Proposal to Trading 212 Demo?', confirmLabel: 'Confirm Trading 212 Demo submit',
  },
  'trading212-cancel': {
    title: 'Confirm Trading 212 Demo cancellation', explanation: 'The exact Trading 212 Demo account and order were just reread. A provider acknowledgement is not proof of cancellation; fills may race this request.',
    prompt: 'Send one cancellation request for this exact Trading 212 Demo order?', confirmLabel: 'Confirm cancellation request',
  },
  'binance-testnet-submit': {
    title: 'Confirm Binance Spot Testnet submission', explanation: 'This sends one order to Binance Spot Testnet only. The endpoint is fixed to Testnet; an acknowledgement is not a fill.',
    prompt: 'Submit this exact immutable Proposal to Binance Spot Testnet?', confirmLabel: 'Confirm Binance Testnet submit',
  },
};

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
  const trading212Accounts = useMemo(() => accounts.data?.accounts.filter(account => account.providerId === 'trading212' && account.environment === 'DEMO') ?? [], [accounts.data?.accounts]);
  const binanceTestnetAccounts = useMemo(() => accounts.data?.accounts.filter(account => account.providerId === 'binance' && account.environment === 'TESTNET') ?? [], [accounts.data?.accounts]);
  const [ordersConnectionId, setOrdersConnectionId] = useState('');
  const ordersQuery = useQuery({
    queryKey: ['alpaca-paper-orders', workspaceId, ordersConnectionId],
    queryFn: () => request('alpaca.paper.orders.get', { workspaceId, connectionId: ordersConnectionId }),
    enabled: Boolean(ordersConnectionId),
    refetchOnMount: 'always',
  });
  const streamAccount = useDomainProjection('account', ordersConnectionId || undefined, fromAccountSnapshot);
  useEffect(() => {
    if (ordersConnectionId && streamAccount.data) void queryClient.invalidateQueries({ queryKey: ['alpaca-paper-orders', workspaceId, ordersConnectionId] });
  }, [ordersConnectionId, queryClient, streamAccount.data?.health.privateStream, streamAccount.data?.health.reconciliation, streamAccount.data?.lastPrivateStreamEventAt, streamAccount.data?.updatedAt, workspaceId]);
  const ordersAccount = alpacaAccounts.find(account => account.connectionId === ordersConnectionId);
  const [trading212ConnectionId, setTrading212ConnectionId] = useState('');
  const trading212OrdersQuery = useQuery({
    queryKey: ['trading212-demo-orders', workspaceId, trading212ConnectionId],
    queryFn: () => request('trading212.demo.orders.get', { workspaceId, connectionId: trading212ConnectionId }),
    enabled: Boolean(trading212ConnectionId),
    refetchOnMount: 'always',
  });
  const trading212OrderBook = trading212OrdersQuery.data?.book;
  const trading212OrdersAccount = trading212Accounts.find(account => account.connectionId === trading212ConnectionId);
  const [binanceTestnetOrdersConnectionId, setBinanceTestnetOrdersConnectionId] = useState('');
  const binanceTestnetOrdersQuery = useQuery({
    queryKey: ['binance-testnet-orders', workspaceId, binanceTestnetOrdersConnectionId],
    queryFn: () => request('binance.testnet.orders.get', { workspaceId, connectionId: binanceTestnetOrdersConnectionId }),
    enabled: Boolean(binanceTestnetOrdersConnectionId),
    refetchOnMount: 'always',
  });
  const binanceTestnetOrderBook = binanceTestnetOrdersQuery.data?.book;
  const binanceTestnetOrdersAccount = binanceTestnetAccounts.find(account => account.connectionId === binanceTestnetOrdersConnectionId);
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
  const [trading212IdempotencyKey, setTrading212IdempotencyKey] = useState<string>();
  const [binanceTestnetIdempotencyKey, setBinanceTestnetIdempotencyKey] = useState<string>();
  const [binanceTestnetReview, setBinanceTestnetReview] = useState<{ connectionId: string; accountLabel: string; remoteAccountId: string }>();
  const [paperConfirmation, setPaperConfirmation] = useState<PaperConfirmation>();
  const [ordersBusy, setOrdersBusy] = useState(false);
  const [trading212OrdersBusy, setTrading212OrdersBusy] = useState(false);
  const [binanceTestnetOrdersBusy, setBinanceTestnetOrdersBusy] = useState(false);
  const [cancelReview, setCancelReview] = useState<{ book: AlpacaPaperOrderBook; order: AlpacaPaperOrder }>();
  const [trading212CancelReview, setTrading212CancelReview] = useState<{ book: Trading212DemoOrderBook; order: Trading212DemoOrder; connectionId: string; accountLabel: string }>();
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
  const trading212Attempt = useQuery({
    queryKey: ['trading212-demo-attempt', workspaceId, selectedProposalId],
    queryFn: () => request('trading212.demo.order.attempt.get', { workspaceId, proposalId: selectedProposalId! }),
    enabled: Boolean(selectedProposalId) && proposalDetail.data?.fields.environment === 'TRADING212_DEMO',
    refetchOnMount: 'always',
  });
  const binanceTestnetAttempt = useQuery({
    queryKey: ['binance-testnet-attempt', workspaceId, selectedProposalId],
    queryFn: () => request('binance.testnet.order.attempt.get', { workspaceId, proposalId: selectedProposalId! }),
    enabled: Boolean(selectedProposalId) && proposalDetail.data?.fields.environment === 'BINANCE_TESTNET',
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
  useEffect(() => {
    if (!trading212ConnectionId && trading212Accounts.length) setTrading212ConnectionId(trading212Accounts[0].connectionId);
    if (trading212ConnectionId && accounts.data && !trading212Accounts.some(account => account.connectionId === trading212ConnectionId)) setTrading212ConnectionId('');
  }, [accounts.data, trading212Accounts, trading212ConnectionId]);
  useEffect(() => {
    if (!binanceTestnetOrdersConnectionId && binanceTestnetAccounts.length) {
      setBinanceTestnetOrdersConnectionId((binanceTestnetAccounts.find(account => account.connectionState === 'CONNECTED') ?? binanceTestnetAccounts[0]).connectionId);
    }
    if (binanceTestnetOrdersConnectionId && accounts.data && !binanceTestnetAccounts.some(account => account.connectionId === binanceTestnetOrdersConnectionId)) setBinanceTestnetOrdersConnectionId('');
  }, [accounts.data, binanceTestnetAccounts, binanceTestnetOrdersConnectionId]);
  useEffect(() => { if (detail.data) setForm(fromDraft(detail.data)); }, [detail.data]);
  useEffect(() => {
    if (!selectedId || newMode || !selectedProposals.some(proposal => proposal.proposalId === selectedProposalId)) setSelectedProposalId(undefined);
  }, [newMode, selectedId, selectedProposalId, selectedProposals]);
  useEffect(() => {
    setPaperResult(undefined);
    setPaperIdempotencyKey(undefined);
    setPaperCancelIdempotencyKey(undefined);
    setAlpacaIdempotencyKey(undefined);
    setTrading212IdempotencyKey(undefined);
    setBinanceTestnetIdempotencyKey(undefined);
    setBinanceTestnetReview(undefined);
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
  useEffect(() => {
    if (!paperConfirmation) return;
    const shell = document.querySelector<HTMLElement>('.app-shell');
    if (shell) shell.inert = true;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        if (!paperBusy && !ordersBusy && !trading212OrdersBusy) {
          setPaperConfirmation(undefined);
          setCancelReview(undefined);
          setTrading212CancelReview(undefined);
          setBinanceTestnetReview(undefined);
        }
        return;
      }
      if (event.key !== 'Tab') return;
      const focusable = [...(confirmationRef.current?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex="0"]') ?? [])];
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && (document.activeElement === first || !confirmationRef.current?.contains(document.activeElement))) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && (document.activeElement === last || !confirmationRef.current?.contains(document.activeElement))) { event.preventDefault(); first.focus(); }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      if (shell) shell.inert = false;
    };
  }, [paperConfirmation, paperBusy, ordersBusy, trading212OrdersBusy]);

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
      setNotice(proposal.status === 'NEEDS_APPROVAL'
        ? `Proposal ${proposal.proposalId.slice(0, 16)}… generated and requires approval.`
        : `No new proposal was created; the matching Proposal is ${proposal.status}.`);
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

  const submitTrading212Proposal = async () => {
    const proposal = proposalDetail.data;
    if (!proposal || proposal.fields.environment !== 'TRADING212_DEMO' || !proposal.fields.accountId) return;
    setPaperBusy(true); setError(undefined); setNotice('');
    const idempotencyKey = trading212IdempotencyKey ?? crypto.randomUUID();
    setTrading212IdempotencyKey(idempotencyKey);
    try {
      const prior = await request('trading212.demo.order.attempt.get', { workspaceId, proposalId: proposal.proposalId });
      if (prior.attempt) {
        queryClient.setQueryData(['trading212-demo-attempt', workspaceId, proposal.proposalId], prior);
        setNotice('A saved Trading 212 Demo attempt already exists. Its status is shown below; no second order was sent.');
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
      if (account.providerId !== 'trading212' || account.environment !== 'DEMO') {
        throw localGuardError('PROVIDER_UNSUPPORTED', 'This Proposal is not bound to a Trading 212 Demo account.');
      }
      const attempt = await request('trading212.demo.order.submit', {
        workspaceId,
        connectionId: account.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        proposalId: currentProposal.proposalId,
        expectedProposalStateVersion: currentProposal.stateVersion,
        proposalHash: currentProposal.proposalHash,
        idempotencyKey,
        confirmedDemoOrder: true,
      });
      queryClient.setQueryData(['trading212-demo-attempt', workspaceId, proposal.proposalId], { attempt });
      await queryClient.invalidateQueries({ queryKey: ['order-proposals', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposal', workspaceId, proposal.proposalId] });
      setNotice(attempt.state === 'UNKNOWN_RECONCILING'
        ? 'Trading 212 Demo returned an unknown result. Do not resubmit; query the account for order evidence.'
        : `Trading 212 Demo attempt ${attempt.state}. A provider acknowledgement is not a fill.`);
    } catch (cause) { setError(cause); }
    finally { setPaperBusy(false); }
  };

  const submitBinanceTestnetProposal = async () => {
    const proposal = proposalDetail.data;
    const reviewedAccount = binanceTestnetReview;
    if (!proposal || proposal.fields.environment !== 'BINANCE_TESTNET' || !proposal.fields.accountId
      || !reviewedAccount || reviewedAccount.connectionId !== proposal.fields.accountId) return;
    setPaperBusy(true); setError(undefined); setNotice('');
    const idempotencyKey = binanceTestnetIdempotencyKey ?? crypto.randomUUID();
    setBinanceTestnetIdempotencyKey(idempotencyKey);
    try {
      const prior = await request('binance.testnet.order.attempt.get', { workspaceId, proposalId: proposal.proposalId });
      if (prior.attempt) {
        queryClient.setQueryData(['binance-testnet-attempt', workspaceId, proposal.proposalId], prior);
        setNotice('A saved Binance Testnet attempt already exists. Its status is shown below; no second order was sent.');
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
      if (account.connectionId !== reviewedAccount.connectionId || account.providerId !== 'binance'
        || account.environment !== 'TESTNET' || account.connectionState !== 'CONNECTED'
        || account.data?.remoteAccountId !== reviewedAccount.remoteAccountId) {
        throw localGuardError('PROVIDER_UNSUPPORTED', 'Connect and confirm this exact Binance Testnet account before submitting.');
      }
      const attempt = await request('binance.testnet.order.submit', {
        workspaceId,
        connectionId: account.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        proposalId: currentProposal.proposalId,
        expectedProposalStateVersion: currentProposal.stateVersion,
        proposalHash: currentProposal.proposalHash,
        idempotencyKey,
        confirmedTestnetOrder: true,
      });
      if (attempt.connectionId !== reviewedAccount.connectionId || attempt.remoteAccountId !== reviewedAccount.remoteAccountId) {
        throw localGuardError('STATE_VERSION_CONFLICT', 'The Binance Testnet account identity changed during submission. Reload the saved attempt before taking another action.');
      }
      queryClient.setQueryData(['binance-testnet-attempt', workspaceId, proposal.proposalId], { attempt });
      await queryClient.invalidateQueries({ queryKey: ['order-proposals', workspaceId] });
      await queryClient.invalidateQueries({ queryKey: ['order-proposal', workspaceId, proposal.proposalId] });
      setNotice(attempt.state === 'UNKNOWN_RECONCILING'
        ? 'Binance Testnet returned an unknown result. Do not resubmit; query this saved client order ID.'
        : `Binance Testnet attempt ${attempt.state}. Provider acknowledgement is not fill evidence.`);
    } catch (cause) { setError(cause); }
    finally { setPaperBusy(false); }
  };

  const reconcileBinanceTestnetAttempt = async (attempt: BinanceTestnetOrderAttempt) => {
    setPaperBusy(true); setError(undefined); setNotice('');
    try {
      const account = await request('account.get', { workspaceId, connectionId: attempt.connectionId });
      if (account.providerId !== 'binance' || account.environment !== 'TESTNET'
        || account.data?.remoteAccountId !== attempt.remoteAccountId) {
        throw localGuardError('PROVIDER_UNSUPPORTED', 'Reconciliation is limited to the saved Binance Testnet account identity.');
      }
      const result = await request('binance.testnet.order.reconcile', {
        workspaceId,
        connectionId: attempt.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        proposalId: attempt.proposalId,
      });
      queryClient.setQueryData(['binance-testnet-attempt', workspaceId, attempt.proposalId], { attempt: result });
      setNotice(`Binance Testnet attempt ${result.state}. No order was resubmitted.`);
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

  const refreshTrading212Orders = async (action: Trading212DemoOrderBookAction, providerOrderId?: string, targetConnectionId = trading212ConnectionId, expectedRemoteAccountId?: string) => {
    if (!targetConnectionId) return;
    setTrading212OrdersBusy(true); setError(undefined); setNotice('');
    try {
      const account = await request('account.get', { workspaceId, connectionId: targetConnectionId });
      if (account.connectionId !== targetConnectionId || account.providerId !== 'trading212' || account.environment !== 'DEMO'
        || account.connectionState !== 'CONNECTED' || (expectedRemoteAccountId && account.data?.remoteAccountId !== expectedRemoteAccountId)) {
        throw localGuardError('ORDER_STATUS_UNKNOWN', 'The selected Trading 212 Demo account changed. Select it again and review the order before requesting cancellation.');
      }
      const result = await request('trading212.demo.orders.refresh', {
        workspaceId,
        connectionId: account.connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        action,
        ...(providerOrderId ? { providerOrderId } : {}),
      });
      queryClient.setQueryData(['trading212-demo-orders', workspaceId, account.connectionId], { book: result });
      if (result.status === 'CURRENT') {
        setNotice('Trading 212 Demo ' + (action === 'PENDING' ? 'pending orders' : action === 'HISTORY' ? 'order history' : 'order detail') + ' refreshed at ' + new Date(result.observedAt).toLocaleString() + '.');
      } else {
        const retryAt = action === 'PENDING' ? result.rateLimits.pendingOrdersRetryAt : action === 'DETAIL' ? result.rateLimits.orderDetailRetryAt : result.rateLimits.historyRetryAt;
        setNotice('Trading 212 Demo read is degraded: ' + (result.reason ?? 'provider data was incomplete') + '. Existing observations were kept.' + (retryAt ? ' Retry ' + retryLabel(retryAt) + '.' : ''));
      }
      return result;
    } catch (cause) { setError(cause); }
    finally { setTrading212OrdersBusy(false); }
  };

  const refreshBinanceTestnetOrders = async (action: BinanceTestnetOrderBookAction, symbol?: string, providerOrderId?: string) => {
    const targetConnectionId = binanceTestnetOrdersConnectionId;
    if (!targetConnectionId) return;
    setBinanceTestnetOrdersBusy(true); setError(undefined); setNotice('');
    try {
      const account = await request('account.get', { workspaceId, connectionId: targetConnectionId });
      if (account.connectionId !== targetConnectionId || account.providerId !== 'binance' || account.environment !== 'TESTNET'
        || account.connectionState !== 'CONNECTED' || (binanceTestnetOrderBook && account.data?.remoteAccountId !== binanceTestnetOrderBook.remoteAccountId)) {
        throw localGuardError('ORDER_STATUS_UNKNOWN', 'The selected Binance Testnet account changed or is disconnected. Refresh linked accounts before reading provider data.');
      }
      const result = await request('binance.testnet.orders.refresh', {
        workspaceId, connectionId: account.connectionId, expectedConnectionStateVersion: account.stateVersion, action,
        ...(symbol ? { symbol } : {}), ...(providerOrderId ? { providerOrderId } : {}),
      });
      queryClient.setQueryData(['binance-testnet-orders', workspaceId, account.connectionId], { book: result });
      const retryAt = result.rateLimits.accountRetryAt ?? (action === 'PENDING' ? result.rateLimits.pendingOrdersRetryAt : action === 'HISTORY' ? result.rateLimits.historyRetryAt : result.rateLimits.orderDetailRetryAt);
      setNotice(result.status === 'CURRENT'
        ? `Binance Testnet ${action === 'PENDING' ? 'open orders' : action === 'ACCOUNT' ? 'balances' : action === 'HISTORY' ? `${symbol} history` : 'order detail'} refreshed at ${new Date(result.observedAt).toLocaleString()}.`
        : `Binance Testnet read is ${result.status.toLowerCase()}: ${result.reason ?? 'provider data is incomplete'}. Existing observations were kept.${retryAt ? ` Retry ${retryLabel(retryAt)}.` : ''}`);
      return result;
    } catch (cause) { setError(cause); }
    finally { setBinanceTestnetOrdersBusy(false); }
  };

  const reviewTrading212Order = async (order: Trading212DemoOrder) => {
    const connectionId = trading212ConnectionId;
    const accountSummary = trading212Accounts.find(account => account.connectionId === connectionId);
    const remoteAccountId = accountSummary?.data?.remoteAccountId;
    if (!accountSummary || !remoteAccountId || !canCancelTrading212Order(order)) return;
    confirmationTriggerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setError(undefined); setNotice('');
    try {
      const book = await refreshTrading212Orders('DETAIL', order.providerOrderId, connectionId, remoteAccountId);
      if (!book || book.status !== 'CURRENT') throw localGuardError('ORDER_STATUS_UNKNOWN', 'Trading 212 could not verify the selected order. Refresh and review again before requesting cancellation.');
      const reviewed = book.orders.find(item => item.providerOrderId === order.providerOrderId);
      if (!reviewed || !reviewed.pending || !canCancelTrading212Order(reviewed)) {
        throw localGuardError('ORDER_NOT_CANCELABLE', 'The provider order is no longer cancelable. Refresh its current provider status.');
      }
      setTrading212CancelReview({ book, order: reviewed, connectionId, accountLabel: accountSummary.label });
      setPaperConfirmation('trading212-cancel');
    } catch (cause) { setError(cause); }
  };

  const cancelTrading212Order = async () => {
    if (!trading212CancelReview) return;
    setTrading212OrdersBusy(true); setError(undefined); setNotice('');
    try {
      const { book: reviewedBook, order, connectionId } = trading212CancelReview;
      const account = await request('account.get', { workspaceId, connectionId });
      if (account.connectionId !== connectionId || account.providerId !== 'trading212' || account.environment !== 'DEMO'
        || account.connectionState !== 'CONNECTED' || account.data?.remoteAccountId !== reviewedBook.remoteAccountId) {
        throw localGuardError('ORDER_STATUS_UNKNOWN', 'The reviewed Trading 212 Demo account identity changed. Refresh the account and order before trying again.');
      }
      const updatedBook = await request('trading212.demo.orders.cancel', {
        workspaceId,
        connectionId,
        expectedConnectionStateVersion: account.stateVersion,
        providerOrderId: order.providerOrderId,
        expectedBookStateVersion: reviewedBook.stateVersion,
        idempotencyKey: crypto.randomUUID(),
        confirmed: true,
      });
      queryClient.setQueryData(['trading212-demo-orders', workspaceId, connectionId], { book: updatedBook });
      const updatedOrder = updatedBook.orders.find(item => item.providerOrderId === order.providerOrderId);
      setNotice(updatedOrder?.cancelError === 'ORDER_CANCEL_STATUS_UNKNOWN'
        ? `Trading 212 Demo cancellation outcome for ${order.providerOrderId} is unknown. It remains pending provider confirmation; refresh this exact order before taking further action.`
        : updatedOrder?.cancelState === 'PENDING'
          ? `Trading 212 Demo accepted the cancellation request for ${order.providerOrderId}. The order remains pending provider confirmation.`
          : updatedOrder?.cancelError
            ? `Trading 212 Demo did not accept cancellation for ${order.providerOrderId}: ${updatedOrder.cancelError}. The provider order remains ${updatedOrder.providerStatus}.`
            : `Trading 212 Demo cancellation state for ${order.providerOrderId}: ${updatedOrder?.providerStatus ?? 'unknown'}.`);
      setTrading212CancelReview(undefined);
    } catch (cause) { setError(cause); }
    finally { setTrading212OrdersBusy(false); }
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

  const openPaperConfirmation = (action: 'submit' | 'cancel' | 'alpaca-submit' | 'alpaca-cancel' | 'trading212-submit' | 'trading212-cancel') => {
    confirmationTriggerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setPaperConfirmation(action);
  };
  const openBinanceTestnetConfirmation = () => {
    const proposal = proposalDetail.data;
    const account = proposal?.fields.accountId && binanceTestnetAccounts.find(item => item.connectionId === proposal.fields.accountId);
    if (!proposal || proposal.fields.environment !== 'BINANCE_TESTNET' || !account
      || account.connectionState !== 'CONNECTED' || !account.data?.remoteAccountId) {
      setError(localGuardError('STATE_STALE', 'Reload the connected Binance Testnet account before reviewing this Proposal.'));
      return;
    }
    confirmationTriggerRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setBinanceTestnetReview({ connectionId: account.connectionId, accountLabel: account.label, remoteAccountId: account.data.remoteAccountId });
    setPaperConfirmation('binance-testnet-submit');
  };
  const confirmPaperAction = async () => {
    const action = paperConfirmation;
    if (!action) return;
    if (action === 'submit') await submitProposal();
    else if (action === 'alpaca-submit') await submitAlpacaProposal();
    else if (action === 'trading212-submit') await submitTrading212Proposal();
    else if (action === 'binance-testnet-submit') await submitBinanceTestnetProposal();
    else if (action === 'alpaca-cancel') await cancelAlpacaOrder();
    else if (action === 'trading212-cancel') await cancelTrading212Order();
    else await cancelPaperOrder();
    setPaperConfirmation(undefined);
    setCancelReview(undefined);
    setTrading212CancelReview(undefined);
    setBinanceTestnetReview(undefined);
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
      {selectedProposals.length > 0 && <div className="order-proposal-layout"><div className="order-proposal-list">{selectedProposals.map(proposal => <ProposalRow key={proposal.proposalId} proposal={proposal} selected={proposal.proposalId === selectedProposalId} onSelect={() => setSelectedProposalId(proposal.proposalId)} />)}</div><div className="order-proposal-detail">{!selectedProposalId && <p className="muted">Choose a proposal to inspect its details.</p>}{selectedProposalId && proposalDetail.isPending && <p role="status">Loading proposal…</p>}{selectedProposalId && proposalDetail.isError && <p className="error-text" role="alert">{explainError(proposalDetail.error)}</p>}{proposalDetail.data && <ProposalDetail proposal={proposalDetail.data} onRefresh={refreshProposal} refreshBusy={proposalBusy} onSubmit={() => openPaperConfirmation('submit')} onCancel={() => openPaperConfirmation('cancel')} onAlpacaSubmit={() => openPaperConfirmation('alpaca-submit')} onTrading212Submit={() => openPaperConfirmation('trading212-submit')} onBinanceTestnetSubmit={openBinanceTestnetConfirmation} onAlpacaReconcile={reconcileAlpacaAttempt} onReloadAlpacaAttempt={() => void alpacaAttempt.refetch()} alpacaAttempt={alpacaAttempt.data?.attempt ?? undefined} alpacaAttemptLoading={alpacaAttempt.isPending} alpacaAttemptError={alpacaAttempt.error} alpacaAccount={alpacaAccounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)} trading212Attempt={trading212Attempt.data?.attempt ?? undefined} trading212AttemptLoading={trading212Attempt.isPending} trading212AttemptError={trading212Attempt.error} trading212Account={trading212Accounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)} onReloadTrading212Attempt={() => void trading212Attempt.refetch()} binanceTestnetAttempt={binanceTestnetAttempt.data?.attempt ?? undefined} binanceTestnetAttemptLoading={binanceTestnetAttempt.isPending} binanceTestnetAttemptError={binanceTestnetAttempt.error} binanceTestnetAccount={binanceTestnetAccounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)} onReloadBinanceTestnetAttempt={() => void binanceTestnetAttempt.refetch()} onBinanceTestnetReconcile={reconcileBinanceTestnetAttempt} submitBusy={paperBusy} cancelBusy={paperBusy} result={paperResult} />}</div></div>}
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
      {ordersAccount && <p className="muted" role="status">Private stream: {streamAccount.data?.health.privateStream ?? ordersAccount.health.privateStream} · Reconciliation: {streamAccount.data?.health.reconciliation ?? ordersAccount.health.reconciliation} · Last event: {streamAccount.data?.lastPrivateStreamEventAt ? new Date(streamAccount.data.lastPrivateStreamEventAt).toLocaleString() : 'Not yet'}</p>}
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
          <small>ALPACA_PAPER · {fill.source === 'TRADE_UPDATE' ? 'trade_updates stream' : 'REST FILL activity'} · received {new Date(fill.observedAt).toLocaleString()}</small>
        </article>)}
      </>}
    </section>
    <section className="card order-book-panel" aria-labelledby="trading212-order-book-title">
      <div className="section-heading">
        <div><h2 id="trading212-order-book-title">Trading 212 Demo orders</h2><p className="muted">Provider observations · TRADING212_DEMO · manual REST refresh</p></div>
        <div className="order-book-actions">
          <label className="field">Demo account<select aria-label="Trading 212 Demo account" value={trading212ConnectionId} onChange={event => setTrading212ConnectionId(event.target.value)}><option value="">Select a Trading 212 Demo account</option>{trading212Accounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.label} · {account.data?.remoteAccountId ?? account.connectionId}</option>)}</select></label>
          <button type="button" onClick={() => void refreshTrading212Orders('PENDING')} disabled={!trading212OrdersAccount || trading212OrdersAccount.connectionState !== 'CONNECTED' || trading212OrdersBusy}>{trading212OrdersBusy ? 'Checking Trading 212…' : 'Refresh pending orders'}</button>
        </div>
      </div>
      {accounts.isPending && <p role="status">Loading linked accounts…</p>}
      {!accounts.isPending && !trading212Accounts.length && <p className="muted">Connect a Trading 212 Demo account to view its pending and historical provider orders.</p>}
      {trading212OrdersAccount && trading212OrdersAccount.connectionState !== 'CONNECTED' && <p className="error-text" role="status">This Trading 212 Demo account is disconnected. Reconnect it before refreshing provider observations.</p>}
      {trading212OrdersQuery.isPending && trading212ConnectionId && <p role="status">Loading saved Trading 212 Demo observations…</p>}
      {trading212OrdersQuery.isError && <div className="error-text" role="alert"><p>Saved Trading 212 Demo orders are unavailable.</p><button type="button" onClick={() => void trading212OrdersQuery.refetch()}>Reload saved orders</button></div>}
      {trading212OrderBook && <>
        <p className={trading212OrderBook.status === 'DEGRADED' || trading212OrderBook.status === 'STALE' ? 'error-text' : 'muted'} role={trading212OrderBook.status === 'DEGRADED' ? 'alert' : 'status'}>
          {trading212OrderBook.status === 'CURRENT' ? 'Current provider observations · received ' + new Date(trading212OrderBook.observedAt).toLocaleString() : trading212OrderBook.status === 'NEVER_SYNCED' ? 'No provider read has completed; refresh pending orders or load history.' : 'Provider read is ' + trading212OrderBook.status.toLowerCase() + ': ' + (trading212OrderBook.reason ?? 'showing the last saved observations') + '.'}
          {trading212OrderBook.lastSuccessfulSyncAt && ' Last successful read: ' + new Date(trading212OrderBook.lastSuccessfulSyncAt).toLocaleString() + '.'}
        </p>
        <h3>Pending orders ({trading212OrderBook.orders.filter(order => order.pending).length})</h3>
        {!trading212OrderBook.orders.some(order => order.pending) && trading212OrderBook.status === 'CURRENT' && <p className="muted">No pending Trading 212 Demo orders were returned.</p>}
        {trading212OrderBook.orders.filter(order => order.pending).map(order => <Trading212OrderCard key={order.providerOrderId} order={order} pending onRefresh={() => void refreshTrading212Orders('DETAIL', order.providerOrderId)} onReviewCancel={trading212OrderBook.status === 'CURRENT' && trading212OrdersAccount?.connectionState === 'CONNECTED' && canCancelTrading212Order(order) ? () => void reviewTrading212Order(order) : undefined} busy={trading212OrdersBusy} />)}
        <div className="section-heading">
          <h3>Order history ({trading212OrderBook.orders.filter(order => !order.pending).length})</h3>
          <button type="button" onClick={() => void refreshTrading212Orders('HISTORY')} disabled={!trading212OrdersAccount || trading212OrdersAccount.connectionState !== 'CONNECTED' || trading212OrdersBusy}>
            {trading212OrdersBusy ? 'Loading history…' : !trading212OrderBook.historyStarted ? 'Load order history' : trading212OrderBook.historyComplete ? 'Refresh order history' : 'Load more history'}
          </button>
        </div>
        {!trading212OrderBook.orders.some(order => !order.pending) && trading212OrderBook.historyStarted && trading212OrderBook.status === 'CURRENT' && <p className="muted">No historical Trading 212 Demo orders were returned.</p>}
        {trading212OrderBook.orders.filter(order => !order.pending).map(order => <Trading212OrderCard key={order.providerOrderId} order={order} pending={false} busy={trading212OrdersBusy} />)}
        <p className="muted">History pages loaded: {trading212OrderBook.historyPageCount} · {trading212OrderBook.historyComplete ? 'complete' : 'more may be available'}</p>
      <p className="muted">Endpoint limits: pending {retryLabel(trading212OrderBook.rateLimits.pendingOrdersRetryAt)} · detail {retryLabel(trading212OrderBook.rateLimits.orderDetailRetryAt)} · history {retryLabel(trading212OrderBook.rateLimits.historyRetryAt)} · cancel {retryLabel(trading212OrderBook.rateLimits.cancelOrderRetryAt)}.</p>
      </>}
    </section>
    <section className="card order-book-panel" aria-labelledby="binance-testnet-order-book-title">
      <div className="section-heading">
        <div><h2 id="binance-testnet-order-book-title">Binance Spot Testnet orders, fills and balances</h2><p className="muted">Provider observations · TESTNET only · manual REST refresh</p></div>
        <div className="order-book-actions">
          <label className="field">Testnet account<select aria-label="Binance Testnet account" value={binanceTestnetOrdersConnectionId} onChange={event => setBinanceTestnetOrdersConnectionId(event.target.value)}><option value="">Select a Binance Testnet account</option>{binanceTestnetAccounts.map(account => <option key={account.connectionId} value={account.connectionId}>{account.label} · {account.data?.remoteAccountId ?? account.connectionId}</option>)}</select></label>
          <button type="button" onClick={() => void refreshBinanceTestnetOrders('PENDING')} disabled={!binanceTestnetOrdersAccount || binanceTestnetOrdersAccount.connectionState !== 'CONNECTED' || binanceTestnetOrdersBusy}>{binanceTestnetOrdersBusy ? 'Checking Binance…' : 'Refresh open orders'}</button>
          <button type="button" onClick={() => void refreshBinanceTestnetOrders('ACCOUNT')} disabled={!binanceTestnetOrdersAccount || binanceTestnetOrdersAccount.connectionState !== 'CONNECTED' || binanceTestnetOrdersBusy}>{binanceTestnetOrdersBusy ? 'Checking Binance…' : 'Refresh balances'}</button>
        </div>
      </div>
      {accounts.isPending && <p role="status">Loading linked accounts…</p>}
      {!accounts.isPending && !binanceTestnetAccounts.length && <p className="muted">Connect a Binance Testnet account to view its provider orders, fills and balances.</p>}
      {binanceTestnetOrdersAccount && binanceTestnetOrdersAccount.connectionState !== 'CONNECTED' && <p className="error-text" role="status">This Binance Testnet account is disconnected. Saved observations remain available; reconnect it before refreshing provider data.</p>}
      {binanceTestnetOrdersQuery.isPending && binanceTestnetOrdersConnectionId && <p role="status">Loading saved Binance Testnet observations…</p>}
      {binanceTestnetOrdersQuery.isError && <div className="error-text" role="alert"><p>Saved Binance Testnet observations are unavailable.</p><button type="button" onClick={() => void binanceTestnetOrdersQuery.refetch()}>Reload saved observations</button></div>}
      {binanceTestnetOrderBook && <>
        <p className={binanceTestnetOrderBook.status === 'DEGRADED' || binanceTestnetOrderBook.status === 'STALE' ? 'error-text' : 'muted'} role={binanceTestnetOrderBook.status === 'DEGRADED' ? 'alert' : 'status'}>
          {binanceTestnetOrderBook.status === 'CURRENT' ? `Current provider observations · received ${new Date(binanceTestnetOrderBook.observedAt).toLocaleString()}` : binanceTestnetOrderBook.status === 'NEVER_SYNCED' ? 'No provider read has completed; refresh orders or balances to load observations.' : `Provider read is ${binanceTestnetOrderBook.status.toLowerCase()}: ${binanceTestnetOrderBook.reason ?? 'showing the last saved observations'}.`}
          {binanceTestnetOrderBook.lastSuccessfulSyncAt && ` Last successful read: ${new Date(binanceTestnetOrderBook.lastSuccessfulSyncAt).toLocaleString()}.`}
        </p>
        <h3>Spot balances ({binanceTestnetOrderBook.balances.length})</h3>
        {!binanceTestnetOrderBook.balances.length && binanceTestnetOrderBook.status === 'CURRENT' && <p className="muted">No nonzero Spot balances were returned. No USD valuation is inferred.</p>}
        {binanceTestnetOrderBook.balances.map(balance => <article className="order-book-fill" key={balance.asset}>
          <strong>{balance.asset}</strong><dl className="order-book-facts"><div><dt>Free</dt><dd>{balance.free}</dd></div><div><dt>Locked</dt><dd>{balance.locked}</dd></div><div><dt>Total</dt><dd>{balance.total}</dd></div></dl><small>TESTNET provider balance · no fiat valuation</small>
        </article>)}
        <h3>Open orders ({binanceTestnetOrderBook.orders.filter(order => order.pending).length})</h3>
        {!binanceTestnetOrderBook.orders.some(order => order.pending) && binanceTestnetOrderBook.status === 'CURRENT' && <p className="muted">No open Binance Testnet orders were returned.</p>}
        {binanceTestnetOrderBook.orders.filter(order => order.pending).map(order => <BinanceTestnetOrderCard key={`${order.symbol}:${order.providerOrderId}`} order={order} busy={binanceTestnetOrdersBusy} onRefresh={() => void refreshBinanceTestnetOrders('DETAIL', order.symbol, order.providerOrderId)} />)}
        <h3>Order history ({binanceTestnetOrderBook.orders.filter(order => !order.pending).length})</h3>
        {!binanceTestnetOrderBook.orders.some(order => !order.pending) && binanceTestnetOrderBook.status === 'CURRENT' && <p className="muted">No historical Binance Testnet orders have been loaded yet.</p>}
        {binanceTestnetOrderBook.orders.filter(order => !order.pending).map(order => <BinanceTestnetOrderCard key={`${order.symbol}:${order.providerOrderId}`} order={order} busy={binanceTestnetOrdersBusy} onRefresh={() => void refreshBinanceTestnetOrders('DETAIL', order.symbol, order.providerOrderId)} />)}
        <div className="section-heading"><h3>Supported order and trade history</h3><span className="muted">Pages are bounded to 1,000 records per endpoint.</span></div>
        {binanceTestnetOrderBook.history.map(history => <div className="order-book-actions" key={history.symbol}>
          <button type="button" onClick={() => void refreshBinanceTestnetOrders('HISTORY', history.symbol)} disabled={!binanceTestnetOrdersAccount || binanceTestnetOrdersAccount.connectionState !== 'CONNECTED' || binanceTestnetOrdersBusy}>{binanceTestnetOrdersBusy ? 'Loading…' : history.complete ? `Restart ${history.symbol} history scan` : history.started ? `Load next ${history.symbol} page` : `Load ${history.symbol} history`}</button>
          <span className="muted">{history.symbol}: {history.started ? `${history.pageCount} page(s) loaded · ${history.complete ? 'complete' : 'more provider records remain'}` : 'not loaded'}</span>
        </div>)}
        <h3>Provider fills and fees ({binanceTestnetOrderBook.fills.length})</h3>
        {!binanceTestnetOrderBook.fills.length && binanceTestnetOrderBook.history.some(history => history.started) && binanceTestnetOrderBook.status === 'CURRENT' && <p className="muted">No trade rows were returned for the loaded history pages.</p>}
        {binanceTestnetOrderBook.fills.map(fill => <article className="order-book-fill" key={`${fill.symbol}:${fill.tradeId}`}>
          <strong>{fill.symbol} · {fill.side} · {fill.quantity} @ {fill.price}</strong>
          <small>Trade {fill.tradeId} · order {fill.providerOrderId} · quote {fill.quoteQuantity} · executed {new Date(fill.executedAtMs).toLocaleString()}</small>
          <small>Fee {fill.commission} {fill.commissionAsset} · provider observation {new Date(fill.observedAt).toLocaleString()}</small>
        </article>)}
        <p className="muted">Endpoint cooldowns: open orders {retryLabel(binanceTestnetOrderBook.rateLimits.pendingOrdersRetryAt)} · account {retryLabel(binanceTestnetOrderBook.rateLimits.accountRetryAt)} · history {retryLabel(binanceTestnetOrderBook.rateLimits.historyRetryAt)} · exact order {retryLabel(binanceTestnetOrderBook.rateLimits.orderDetailRetryAt)}.</p>
      </>}
    </section>
    {paperConfirmation && createPortal(<div className="picker-backdrop"><div className="picker-dialog" role="dialog" aria-modal="true" aria-labelledby="paper-confirm-title" ref={confirmationRef}>
      <div className="picker-dialog-heading"><div>
        <h2 id="paper-confirm-title">{paperConfirmationCopy[paperConfirmation].title}</h2>
        <p className="muted">{paperConfirmationCopy[paperConfirmation].explanation}</p>
      </div></div>
      <p>{paperConfirmationCopy[paperConfirmation].prompt}</p>
      {paperConfirmation === 'alpaca-cancel' && cancelReview && <dl className="proposal-fields"><div><dt>Environment / account</dt><dd>ALPACA_PAPER · {ordersAccount?.label ?? 'Unavailable'} · {cancelReview.book.remoteAccountId}</dd></div><div><dt>Provider order</dt><dd>{cancelReview.order.providerOrderId}</dd></div><div><dt>Instrument / side</dt><dd>{cancelReview.order.instrumentId ?? cancelReview.order.symbol} · {cancelReview.order.side.toUpperCase()}</dd></div><div><dt>Provider status</dt><dd>{cancelReview.order.providerStatus}</dd></div><div><dt>Filled quantity</dt><dd>{cancelReview.order.filledQuantity}</dd></div><div><dt>Remaining quantity</dt><dd>{cancelReview.order.remainingQuantity ?? 'Unavailable'}</dd></div><div><dt>Last observation</dt><dd>{new Date(cancelReview.order.observedAt).toLocaleString()}</dd></div></dl>}
      {paperConfirmation === 'trading212-cancel' && trading212CancelReview && <dl className="proposal-fields"><div><dt>Environment / account</dt><dd>Trading 212 Demo · TRADING212_DEMO · {trading212CancelReview.accountLabel} · {trading212CancelReview.book.remoteAccountId}</dd></div><div><dt>Provider order</dt><dd>{trading212CancelReview.order.providerOrderId}</dd></div><div><dt>Instrument / side</dt><dd>{trading212CancelReview.order.symbol} · {trading212CancelReview.order.side}</dd></div><div><dt>Provider / normalized status</dt><dd>{trading212CancelReview.order.providerStatus} / {trading212CancelReview.order.normalizedStatus}</dd></div><div><dt>Filled quantity</dt><dd>{trading212CancelReview.order.filledQuantity ?? 'Unavailable'}</dd></div><div><dt>Remaining quantity</dt><dd>{trading212CancelReview.order.remainingQuantity ?? 'Unavailable'}</dd></div><div><dt>Filled value</dt><dd>{trading212CancelReview.order.filledValue == null ? 'Unavailable' : `${trading212CancelReview.order.filledValue} ${trading212CancelReview.order.currency ?? 'currency unavailable'}`}</dd></div><div><dt>Last provider observation</dt><dd>{new Date(trading212CancelReview.order.observedAt).toLocaleString()}</dd></div><div><dt>Provider acknowledgement</dt><dd>Acceptance only; cancellation is not confirmed until a later provider observation.</dd></div></dl>}
      {(paperConfirmation === 'alpaca-submit' || paperConfirmation === 'trading212-submit' || paperConfirmation === 'binance-testnet-submit') && proposalDetail.data && <dl className="proposal-fields">
        <div><dt>Environment / account</dt><dd>{paperConfirmation === 'binance-testnet-submit' ? `Binance Spot Testnet · TESTNET · ${binanceTestnetReview?.accountLabel ?? 'Unavailable'}` : paperConfirmation === 'trading212-submit' ? 'Trading 212 Demo · TRADING212_DEMO' : 'Alpaca Paper'} · {paperConfirmation === 'binance-testnet-submit' ? binanceTestnetReview?.remoteAccountId ?? 'provider account ID unavailable' : accounts.data?.accounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)?.label ?? 'Unavailable'}{paperConfirmation === 'binance-testnet-submit' ? '' : ` · ${accounts.data?.accounts.find(account => account.connectionId === proposalDetail.data?.fields.accountId)?.data?.remoteAccountId ?? 'provider account ID unavailable'}`}</dd></div>
        {paperConfirmation === 'binance-testnet-submit' && <div><dt>Connection ID</dt><dd>{binanceTestnetReview?.connectionId ?? 'Unavailable'}</dd></div>}
        <div><dt>Instrument / side</dt><dd>{proposalDetail.data.fields.instrumentId} · {proposalDetail.data.fields.side}</dd></div>
        <div><dt>Quantity / order</dt><dd>{proposalDetail.data.fields.quantity.value} {proposalDetail.data.fields.quantity.type} · {proposalDetail.data.fields.orderType} · {proposalDetail.data.fields.timeInForce}</dd></div>
        <div><dt>Venue / context</dt><dd>{proposalDetail.data.fields.venue} · {proposalDetail.data.fields.environment}</dd></div>
        <div><dt>Maximum spend</dt><dd>{proposalDetail.data.fields.maximumSpend ?? '—'}</dd></div>
        <div><dt>Limit</dt><dd>{proposalDetail.data.fields.limitPrice ?? '—'}</dd></div>
        <div><dt>Client label</dt><dd>{proposalDetail.data.fields.clientLabel ?? '—'}</dd></div>
        {paperConfirmation === 'trading212-submit' && proposalDetail.data.fields.orderType === 'MARKET' && <div><dt>Extended hours</dt><dd>Off</dd></div>}
        <div><dt>Proposal / hash</dt><dd>{proposalDetail.data.proposalId} · {proposalDetail.data.proposalHash}</dd></div>
      </dl>}
      <div className="picker-dialog-actions"><button type="button" onClick={() => { setPaperConfirmation(undefined); setCancelReview(undefined); setTrading212CancelReview(undefined); setBinanceTestnetReview(undefined); }} disabled={paperBusy || ordersBusy || trading212OrdersBusy}>Keep reviewing</button><button type="button" className="primary" onClick={() => void confirmPaperAction()} disabled={paperBusy || ordersBusy || trading212OrdersBusy}>{paperBusy || ordersBusy || trading212OrdersBusy ? 'Working…' : paperConfirmationCopy[paperConfirmation].confirmLabel}</button></div>
    </div></div>, document.body)}
  </>;
}

function Trading212OrderCard({ order, pending, onRefresh, onReviewCancel, busy }: {
  order: Trading212DemoOrder;
  pending: boolean;
  onRefresh?: () => void;
  onReviewCancel?: () => void;
  busy: boolean;
}) {
  return <article className="order-book-order">
    <div><strong>{order.symbol} · {order.side} · {order.providerStatus}</strong><small>TRADING212_DEMO · {order.origin === 'TRADE_X' ? 'TradeX proposal' : 'External provider order'} · observed {new Date(order.observedAt).toLocaleString()}</small></div>
    <dl className="order-book-facts">
      <div><dt>Provider order</dt><dd>{order.providerOrderId}</dd></div>
      <div><dt>Provider / normalized status</dt><dd>{order.providerStatus} / {order.normalizedStatus}</dd></div>
      <div><dt>Order type / time in force</dt><dd>{order.orderType} · {order.timeInForce}</dd></div>
      <div><dt>Quantity / remaining</dt><dd>{order.quantity ?? 'Unavailable'} / {order.remainingQuantity ?? 'Unavailable'}</dd></div>
      <div><dt>Cumulative filled quantity</dt><dd>{order.filledQuantity ?? 'Unavailable'}</dd></div>
      <div><dt>Cumulative filled value</dt><dd>{order.filledValue == null ? 'Unavailable' : `${order.filledValue} ${order.currency ?? 'currency unavailable'}`}</dd></div>
      <div><dt>Submitted</dt><dd>{new Date(order.submittedAt).toLocaleString()}</dd></div>
      {order.attemptId && <div><dt>TradeX attempt</dt><dd>{order.attemptId}</dd></div>}
    </dl>
    {order.cancelState === 'SUBMITTING' && <p className="muted" role="status">Cancellation request is being sent. The provider order remains pending verification.</p>}
    {order.cancelState === 'PENDING' && <p className="muted" role="status">Cancellation request accepted or outcome unknown; refresh this exact order for provider confirmation.</p>}
    {order.cancelError && order.cancelError !== 'ORDER_CANCEL_STATUS_UNKNOWN' && <p className="error-text" role="status">Cancellation request was not accepted: {order.cancelError}</p>}
    {pending && onReviewCancel && <button type="button" onClick={onReviewCancel} disabled={busy}>Review cancellation</button>}
    {pending && onRefresh && <button type="button" onClick={onRefresh} disabled={busy}>{busy ? 'Refreshing order…' : 'Refresh known order details'}</button>}
    {!pending && !order.pending && order.normalizedStatus === 'OPEN' && <p className="muted" role="status">This order was not returned in the latest pending-order read; the displayed status is its last provider observation.</p>}
  </article>;
}

function BinanceTestnetOrderCard({ order, onRefresh, busy }: {
  order: BinanceTestnetOrder;
  onRefresh: () => void;
  busy: boolean;
}) {
  return <article className="order-book-order">
    <div><strong>{order.symbol} · {order.side} · {order.providerStatus}</strong><small>BINANCE_TESTNET · {order.origin === 'TRADE_X' ? 'TradeX proposal' : 'External provider order'} · observed {new Date(order.observedAt).toLocaleString()}</small></div>
    <dl className="order-book-facts">
      <div><dt>Provider order / client ID</dt><dd>{order.providerOrderId} / {order.clientOrderId}</dd></div>
      <div><dt>Order type / time in force</dt><dd>{order.orderType} · {order.timeInForce}</dd></div>
      <div><dt>Quantity / quote target</dt><dd>{order.quantity ?? 'Unavailable'} / {order.quoteQuantity ?? 'Unavailable'}</dd></div>
      <div><dt>Cumulative filled base / quote</dt><dd>{order.filledQuantity} / {order.filledQuoteQuantity ?? 'Unavailable'}</dd></div>
      <div><dt>Remaining base quantity</dt><dd>{order.remainingQuantity ?? 'Unavailable'}</dd></div>
      <div><dt>Provider updated</dt><dd>{new Date(order.providerUpdatedAtMs).toLocaleString()}</dd></div>
      <div><dt>Submitted</dt><dd>{new Date(order.submittedAtMs).toLocaleString()}</dd></div>
      {order.attemptId && <div><dt>TradeX attempt</dt><dd>{order.attemptId}</dd></div>}
    </dl>
    <button type="button" onClick={onRefresh} disabled={busy}>{busy ? 'Refreshing…' : 'Refresh exact order'}</button>
  </article>;
}

function ProposalDetail({ proposal, onRefresh, refreshBusy, onSubmit, onCancel, onAlpacaSubmit, onAlpacaReconcile, onReloadAlpacaAttempt, alpacaAttempt, alpacaAttemptLoading, alpacaAttemptError, alpacaAccount, onTrading212Submit, onReloadTrading212Attempt, trading212Attempt, trading212AttemptLoading, trading212AttemptError, trading212Account, onBinanceTestnetSubmit, onBinanceTestnetReconcile, onReloadBinanceTestnetAttempt, binanceTestnetAttempt, binanceTestnetAttemptLoading, binanceTestnetAttemptError, binanceTestnetAccount, submitBusy, cancelBusy, result }: {
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
  onTrading212Submit: () => void;
  onReloadTrading212Attempt: () => void;
  trading212Attempt?: Trading212DemoOrderAttempt;
  trading212AttemptLoading: boolean;
  trading212AttemptError: unknown;
  trading212Account?: AccountConnection;
  onBinanceTestnetSubmit: () => void;
  onBinanceTestnetReconcile: (attempt: BinanceTestnetOrderAttempt) => void;
  onReloadBinanceTestnetAttempt: () => void;
  binanceTestnetAttempt?: BinanceTestnetOrderAttempt;
  binanceTestnetAttemptLoading: boolean;
  binanceTestnetAttemptError: unknown;
  binanceTestnetAccount?: AccountConnection;
  submitBusy: boolean;
  cancelBusy: boolean;
  result?: PaperOrderResult;
}) {
  const alpacaReady = alpacaAccount?.providerId === 'alpaca'
    && alpacaAccount.environment === 'PAPER'
    && alpacaAccount.connectionState === 'CONNECTED';
  const trading212Ready = trading212Account?.providerId === 'trading212'
    && trading212Account.environment === 'DEMO'
    && trading212Account.connectionState === 'CONNECTED';
  const binanceTestnetReady = binanceTestnetAccount?.providerId === 'binance'
    && binanceTestnetAccount.environment === 'TESTNET'
    && binanceTestnetAccount.connectionState === 'CONNECTED';
  return <div className="proposal-read-only" aria-label="Proposal detail">
    <div className="proposal-meta"><strong>{proposal.status}</strong><span>Draft v{proposal.draftVersion}</span><span>{proposal.proposalId}</span><span>{proposal.proposalHash}</span></div>
    <dl className="proposal-fields"><div><dt>Instrument</dt><dd>{proposal.fields.instrumentId}</dd></div><div><dt>Account</dt><dd>{proposal.fields.accountId ?? 'Local Paper account'}</dd></div><div><dt>Side / type</dt><dd>{proposal.fields.side} · {proposal.fields.orderType}</dd></div><div><dt>Quantity</dt><dd>{proposal.fields.quantity.value} {proposal.fields.quantity.type}</dd></div><div><dt>Limit price</dt><dd>{proposal.fields.limitPrice ?? '—'}</dd></div><div><dt>Maximum spend</dt><dd>{proposal.fields.maximumSpend ?? '—'}</dd></div><div><dt>Venue / context</dt><dd>{proposal.fields.venue} · {proposal.fields.environment === 'TRADING212_DEMO' ? 'Trading 212 Demo · TRADING212_DEMO' : proposal.fields.environment}</dd></div><div><dt>Time in force</dt><dd>{proposal.fields.timeInForce}</dd></div><div><dt>Client label</dt><dd>{proposal.fields.clientLabel ?? '—'}</dd></div><div><dt>Estimated notional</dt><dd>{proposal.estimatedNotional ? `${proposal.estimatedNotional} ${proposal.estimatedNotionalCurrency ?? ''}` : proposal.estimatedNotionalReason ?? 'Unavailable'}</dd></div></dl>
    <p className="proposal-reference"><strong>Policy:</strong> {proposal.policyStatus} · v{proposal.policyVersion ?? '—'} · state {proposal.policyStateVersion ?? '—'} · {proposal.policyReferenceReason}</p>
    <p className="proposal-reference"><strong>Market:</strong> {proposal.marketStatus} · snapshot {proposal.marketSnapshotId ?? '—'} · {proposal.marketReferenceReason}</p>
    {proposal.invalidationReason && <p className="error-text">{proposal.invalidationReason}</p>}
    {proposal.status === 'NEEDS_APPROVAL' && <button type="button" onClick={onRefresh} disabled={refreshBusy}>{refreshBusy ? 'Refreshing proposal…' : 'Refresh proposal'}</button>}
    {proposal.status === 'NEEDS_APPROVAL' && proposal.fields.environment === 'LOCAL_PAPER' && <button type="button" className="primary" onClick={onSubmit} disabled={submitBusy}>{submitBusy ? 'Submitting Local Paper order…' : 'Submit Local Paper order'}</button>}
    {proposal.fields.environment === 'ALPACA_PAPER' && alpacaAttemptLoading && <p role="status">Loading saved Alpaca Paper attempt…</p>}
    {proposal.fields.environment === 'ALPACA_PAPER' && Boolean(alpacaAttemptError) && <div className="error-text" role="alert"><p>Saved Alpaca Paper attempt is unavailable.</p><button type="button" onClick={onReloadAlpacaAttempt}>Reload attempt</button></div>}
    {proposal.fields.environment === 'ALPACA_PAPER' && alpacaAttempt && <section className="notice" aria-label="Alpaca Paper order attempt"><strong>ALPACA_PAPER · {alpacaAttempt.state}</strong><p>{alpacaAttempt.reason}</p><p>Paper account {alpacaAttempt.remoteAccountId} · client order {alpacaAttempt.clientOrderId}</p>{alpacaAttempt.providerOrderId && <p>Provider order {alpacaAttempt.providerOrderId} · provider status {alpacaAttempt.providerStatus ?? 'Unavailable'}</p>}{alpacaAttempt.state === 'ACKNOWLEDGED' && <p>Alpaca acknowledged the order. This is not fill evidence.</p>}{alpacaAttempt.state === 'SUBMITTING' && <p role="status">Submission is still pending. Reload this attempt to check its saved state.</p>}{alpacaAttempt.state === 'UNKNOWN_RECONCILING' && <button type="button" onClick={() => onAlpacaReconcile(alpacaAttempt)} disabled={submitBusy}>{submitBusy ? 'Checking Alpaca Paper…' : 'Reconcile by client order ID'}</button>}</section>}
    {proposal.status === 'NEEDS_APPROVAL' && proposal.fields.environment === 'ALPACA_PAPER' && !alpacaAttempt && !alpacaAttemptLoading && !alpacaAttemptError && <>{alpacaReady ? <button type="button" className="primary" onClick={onAlpacaSubmit} disabled={submitBusy}>{submitBusy ? 'Submitting Alpaca Paper order…' : 'Submit Alpaca Paper order'}</button> : <p className="muted">Connect and confirm this Alpaca Paper account before submitting the Proposal.</p>}</>}
    {proposal.fields.environment === 'TRADING212_DEMO' && trading212AttemptLoading && <p role="status">Loading saved Trading 212 Demo attempt…</p>}
    {proposal.fields.environment === 'TRADING212_DEMO' && Boolean(trading212AttemptError) && <div className="error-text" role="alert"><p>Saved Trading 212 Demo attempt is unavailable. Reload it before taking another action.</p><button type="button" onClick={onReloadTrading212Attempt}>Reload attempt</button></div>}
    {proposal.fields.environment === 'TRADING212_DEMO' && trading212Attempt && <section className="notice" aria-label="Trading 212 Demo order attempt"><strong>Trading 212 Demo · TRADING212_DEMO · {trading212Attempt.state}</strong><p>{trading212Attempt.reason}</p><p>Account {trading212Attempt.remoteAccountId} · Proposal {trading212Attempt.proposalId} · {trading212Attempt.proposalHash}</p>{trading212Attempt.providerOrderId && <p>Provider order {trading212Attempt.providerOrderId} · provider status {trading212Attempt.providerStatus ?? 'Unavailable'}</p>}<p>Updated {new Date(trading212Attempt.updatedAt).toLocaleString()}</p>{trading212Attempt.state === 'ACKNOWLEDGED' && <p>Trading 212 acknowledged the order. This is not fill evidence.</p>}{trading212Attempt.state === 'SUBMITTING' && <p role="status">Submission is still pending. Reload this attempt to check its saved state.</p>}{trading212Attempt.state === 'UNKNOWN_RECONCILING' && <p className="error-text">Result unknown. Do not resubmit; the attempt remains frozen until later provider-order reconciliation.</p>}{trading212Attempt.state === 'REJECTED' && <p className="error-text">This Proposal's attempt was rejected and cannot be submitted again. Refresh the Proposal before a new reviewed attempt.</p>}</section>}
    {proposal.status === 'NEEDS_APPROVAL' && proposal.fields.environment === 'TRADING212_DEMO' && !trading212Attempt && !trading212AttemptLoading && !trading212AttemptError && <>{trading212Ready ? <button type="button" className="primary" onClick={onTrading212Submit} disabled={submitBusy}>{submitBusy ? 'Submitting Trading 212 Demo order…' : 'Submit Trading 212 Demo order'}</button> : <p className="muted">Connect and confirm this Trading 212 Demo account before submitting the Proposal.</p>}</>}
    {proposal.fields.environment === 'BINANCE_TESTNET' && binanceTestnetAttemptLoading && <p role="status">Loading saved Binance Testnet attempt…</p>}
    {proposal.fields.environment === 'BINANCE_TESTNET' && Boolean(binanceTestnetAttemptError) && <div className="error-text" role="alert"><p>Saved Binance Testnet attempt is unavailable. Reload it before taking another action.</p><button type="button" onClick={onReloadBinanceTestnetAttempt}>Reload attempt</button></div>}
    {proposal.fields.environment === 'BINANCE_TESTNET' && binanceTestnetAttempt && <section className="notice" aria-label="Binance Spot Testnet order attempt"><strong>Binance Spot Testnet · TESTNET · {binanceTestnetAttempt.state}</strong><p>{binanceTestnetAttempt.reason}</p><p>Account {binanceTestnetAttempt.remoteAccountId} · client order {binanceTestnetAttempt.clientOrderId}</p><p>Proposal {binanceTestnetAttempt.proposalId} · {binanceTestnetAttempt.proposalHash}</p>{binanceTestnetAttempt.providerOrderId && <p>Provider order {binanceTestnetAttempt.providerOrderId} · provider status {binanceTestnetAttempt.providerStatus ?? 'Unavailable'}</p>}<p>Updated {new Date(binanceTestnetAttempt.updatedAt).toLocaleString()}</p>{binanceTestnetAttempt.state === 'ACKNOWLEDGED' && <p>Binance acknowledged the order. This is not fill evidence.</p>}{binanceTestnetAttempt.state === 'SUBMITTING' && <><p role="status">Submission is still pending. Reload this saved attempt before taking further action.</p><button type="button" onClick={onReloadBinanceTestnetAttempt} disabled={submitBusy}>Reload saved attempt</button></>}{binanceTestnetAttempt.state === 'UNKNOWN_RECONCILING' && <><p className="error-text">Result unknown. Do not resubmit; the attempt remains frozen until the saved client order ID is found or reconciled.</p><button type="button" onClick={() => onBinanceTestnetReconcile(binanceTestnetAttempt)} disabled={submitBusy}>{submitBusy ? 'Checking Binance Testnet…' : 'Reconcile by client order ID'}</button></>}{binanceTestnetAttempt.state === 'REJECTED' && <p className="error-text">This Proposal's attempt was rejected and cannot be submitted again. Refresh the Proposal before a new reviewed attempt.</p>}</section>}
    {proposal.status === 'NEEDS_APPROVAL' && proposal.fields.environment === 'BINANCE_TESTNET' && !binanceTestnetAttempt && !binanceTestnetAttemptLoading && !binanceTestnetAttemptError && <>{binanceTestnetReady ? <button type="button" className="primary" onClick={onBinanceTestnetSubmit} disabled={submitBusy}>{submitBusy ? 'Submitting Binance Testnet order…' : 'Submit Binance Spot Testnet order'}</button> : <p className="muted">Connect and confirm this Binance Testnet account before submitting the Proposal.</p>}</>}
    {result?.proposalId === proposal.proposalId && <section className="notice" aria-label="Local Paper order result"><strong>TRADEX_SIMULATION · {result.order.state}</strong><p>{result.disclosure}</p><p>Order {result.order.orderId} · {result.order.filledQuantity} filled · {result.order.remainingQuantity} remaining · quote {result.quote.price} {result.quote.currency} · {result.quote.scenarioId}</p>{result.fill && <p>Fill {result.fill.fillId} · {result.fill.quantity} @ {result.fill.price} {result.fill.currency}</p>}<p>Proposal hash: {result.proposalHash}</p>{['ACCEPTED', 'PARTIALLY_FILLED'].includes(result.order.state) && <button type="button" onClick={onCancel} disabled={cancelBusy}>{cancelBusy ? 'Cancelling Local Paper order…' : 'Cancel Local Paper order'}</button>}</section>}
    <h3>History</h3><ol className="proposal-history">{proposal.history.map((entry, index) => <li key={`${entry.event}-${entry.occurredAt}-${index}`}><strong>{entry.event}</strong><time dateTime={entry.occurredAt}>{new Date(entry.occurredAt).toLocaleString()}</time>{entry.reason && <span>{entry.reason}</span>}</li>)}</ol>
  </div>;
}
