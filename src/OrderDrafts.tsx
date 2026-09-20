import { useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type {
  AccountConnection,
  ExecutionContext,
  MarketTier,
  OrderDraft,
  OrderDraftFields,
  OrderDraftSummary,
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

export function OrderDrafts({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const library = useQuery({ queryKey: ['order-drafts', workspaceId], queryFn: () => request('trade.draft.list', { workspaceId }), refetchOnMount: 'always' });
  const accounts = useQuery({ queryKey: ['accounts', workspaceId, 'order-draft'], queryFn: () => request('account.list', { workspaceId }) });
  const catalog = useQuery({ queryKey: ['market-catalog', workspaceId, 'order-draft'], queryFn: () => request('market.catalog', { workspaceId, query: '', tier: marketTier }) });
  const [selectedId, setSelectedId] = useState<string>();
  const [newMode, setNewMode] = useState(false);
  const [form, setForm] = useState<DraftForm>(initialForm);
  const [error, setError] = useState<unknown>();
  const [fieldError, setFieldError] = useState<{ field: DraftField; message: string }>();
  const [notice, setNotice] = useState('');
  const detail = useQuery({ queryKey: ['order-draft', workspaceId, selectedId], queryFn: () => request('trade.draft.get', { workspaceId, draftId: selectedId! }), enabled: Boolean(selectedId) && !newMode });
  const selected = useMemo(() => newMode ? undefined : library.data?.drafts.find(draft => draft.draftId === selectedId), [library.data?.drafts, newMode, selectedId]);
  const detailLoading = Boolean(selectedId) && !newMode && detail.isPending;

  useEffect(() => {
    if (!newMode && !selectedId && library.data?.drafts.length) setSelectedId(library.data.drafts[0].draftId);
    if (selectedId && library.data && !library.data.drafts.some(draft => draft.draftId === selectedId)) setSelectedId(undefined);
  }, [library.data, newMode, selectedId]);
  useEffect(() => { if (detail.data) setForm(fromDraft(detail.data)); }, [detail.data]);

  const instruments = catalog.data?.instruments ?? [];
  const localErrors = [
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
          <div className="form-actions"><button className="primary" disabled={Boolean(localErrors.length) || catalog.isPending || !instruments.length}>Save draft</button></div>
        </form>}
      </section>
    </div>
  </>;
}
