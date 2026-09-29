import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { Watchlist, Watchlists as WatchlistsState } from '../shared/ipc-types.ts';
import { CommandError, explainError, request } from './client.ts';

function mutationMessage(error: unknown): string {
  if (error instanceof CommandError && error.detail.code === 'STATE_VERSION_CONFLICT') {
    return 'This list changed elsewhere. Reload it before trying again.';
  }
  return explainError(error);
}

function Library({ state, selectedId, onSelect }: { state: WatchlistsState; selectedId?: string; onSelect: (id: string) => void }) {
  return <section className="card watchlist-library" aria-labelledby="watchlist-library-title">
    <div className="watchlist-section-heading"><div><p className="eyebrow">D1 · Watchlist library</p><h2 id="watchlist-library-title">Watchlists</h2></div><span className="badge">{state.watchlists.length}</span></div>
    {state.watchlists.length ? <div className="watchlist-list" role="list" aria-label="Saved watchlists">{state.watchlists.map(list => <button type="button" className="watchlist-row" key={list.watchlistId} aria-pressed={list.watchlistId === selectedId} onClick={() => onSelect(list.watchlistId)}><span><strong>{list.name}</strong><small>{list.items.length} {list.items.length === 1 ? 'instrument' : 'instruments'}</small></span><small className="identity">{list.watchlistId}</small></button>)}</div> : <div className="watchlist-empty"><h3>No watchlists yet</h3><p>Create one to keep a canonical, ordered set of instruments in this workspace.</p></div>}
  </section>;
}

function Detail({
  workspaceId,
  list,
  catalogTerm,
  setCatalogTerm,
  onRename,
  onDelete,
  onAdd,
  onRemove,
  busy,
}: {
  workspaceId: string;
  list: Watchlist;
  catalogTerm: string;
  setCatalogTerm: (value: string) => void;
  onRename: (name: string) => Promise<void>;
  onDelete: () => Promise<void>;
  onAdd: (instrumentId: string) => Promise<void>;
  onRemove: (instrumentId: string) => Promise<void>;
  busy: boolean;
}) {
  const [name, setName] = useState(list.name);
  const [renaming, setRenaming] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const catalog = useQuery({ queryKey: ['market-catalog', workspaceId, 'watchlist', catalogTerm], queryFn: () => request('market.catalog', { workspaceId, query: catalogTerm, tier: 'CENSUS' }) });
  useEffect(() => { setName(list.name); setRenaming(false); setConfirmDelete(false); }, [list.watchlistId, list.name]);
  const submitRename = async (event: FormEvent) => { event.preventDefault(); try { await onRename(name); setRenaming(false); } catch { /* parent keeps the typed error visible */ } };
  const listed = new Set(list.items.map(item => item.instrumentId));
  return <section className="card watchlist-detail" aria-labelledby="watchlist-detail-title">
    <div className="watchlist-section-heading"><div><p className="eyebrow">D2 · Watchlist detail</p><h2 id="watchlist-detail-title">{list.name}</h2><p className="identity">{list.watchlistId}</p></div><span className="badge">{list.stateVersion}</span></div>
    <div className="watchlist-actions">
      {renaming ? <form className="watchlist-inline-form" onSubmit={submitRename}><label htmlFor="watchlist-rename">Rename watchlist</label><div><input id="watchlist-rename" value={name} maxLength={80} onChange={event => setName(event.target.value)} /><button className="primary" type="submit" disabled={busy}>Save</button><button type="button" onClick={() => { setName(list.name); setRenaming(false); }} disabled={busy}>Cancel</button></div></form> : <button type="button" onClick={() => setRenaming(true)} disabled={busy}>Rename</button>}
      {confirmDelete ? <><span className="muted">Delete this list?</span><button type="button" className="danger" onClick={() => { void onDelete(); }} disabled={busy}>Confirm delete</button><button type="button" onClick={() => setConfirmDelete(false)} disabled={busy}>Cancel</button></> : <button type="button" onClick={() => setConfirmDelete(true)} disabled={busy}>Delete</button>}
    </div>
    <section className="watchlist-members" aria-labelledby="watchlist-members-title"><div className="watchlist-section-heading"><div><h3 id="watchlist-members-title">Members</h3><p className="muted">Ordered canonical instrument IDs. Quotes and credentials never enter this projection.</p></div><span className="badge">{list.items.length}</span></div>{list.items.length ? <ol>{list.items.map(item => <li key={item.instrumentId}><code>{item.instrumentId}</code><button type="button" onClick={() => { void onRemove(item.instrumentId); }} disabled={busy}>Remove</button></li>)}</ol> : <p className="watchlist-empty-line">No instruments in this list. Add one from the Census catalog below.</p>}</section>
    <section className="watchlist-add" aria-labelledby="watchlist-add-title"><div className="watchlist-section-heading"><div><p className="eyebrow">D4 · Add instrument</p><h3 id="watchlist-add-title">Markets catalog</h3><p className="muted">Search the canonical registry; adding an existing member is a no-op.</p></div></div><label htmlFor="watchlist-market-search">Search instruments</label><input id="watchlist-market-search" value={catalogTerm} maxLength={120} onChange={event => setCatalogTerm(event.target.value)} placeholder="AAPL, BTC/USDT or company name" />{catalog.isPending ? <p role="status">Loading market catalog…</p> : catalog.error ? <div className="watchlist-inline-error" role="alert"><p>{explainError(catalog.error)}</p><button type="button" onClick={() => { void catalog.refetch(); }}>Reload catalog</button></div> : <div className="watchlist-market-results" role="list" aria-label="Canonical instruments">{(catalog.data?.instruments ?? []).map(instrument => <div className="watchlist-market-row" key={instrument.instrumentId}><span><strong>{instrument.symbol}</strong><small>{instrument.displayName}</small><code>{instrument.instrumentId}</code></span><button type="button" onClick={() => { void onAdd(instrument.instrumentId); }} disabled={busy || listed.has(instrument.instrumentId)}>{listed.has(instrument.instrumentId) ? 'Added' : 'Add'}</button></div>)}</div>}
    </section>
  </section>;
}

export function Watchlists({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const state = useQuery({ queryKey: ['watchlists', workspaceId], queryFn: () => request('watchlist.list', { workspaceId }), refetchOnMount: 'always', refetchOnReconnect: 'always' });
  const [selectedId, setSelectedId] = useState<string>();
  const [newName, setNewName] = useState('');
  const [creating, setCreating] = useState(false);
  const [busy, setBusy] = useState(false);
  const [catalogTerm, setCatalogTerm] = useState('');
  const [error, setError] = useState<unknown>();
  const [notice, setNotice] = useState('');
  useEffect(() => {
    const lists = state.data?.watchlists ?? [];
    if (!lists.some(list => list.watchlistId === selectedId)) setSelectedId(lists[0]?.watchlistId);
  }, [selectedId, state.data?.watchlists]);
  const selected = state.data?.watchlists.find(list => list.watchlistId === selectedId);
  const refresh = async () => { await queryClient.invalidateQueries({ queryKey: ['watchlists', workspaceId] }); };
  const submitCreate = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true); setError(undefined); setNotice('');
    try { const created = await request('watchlist.create', { workspaceId, name: newName }); await refresh(); setSelectedId(created.watchlistId); setNewName(''); setCreating(false); setNotice(`Created “${created.name}”.`); } catch (nextError) { setError(nextError); } finally { setBusy(false); }
  };
  const rename = async (name: string) => {
    if (!selected) return;
    setBusy(true); setError(undefined); setNotice('');
    try { await request('watchlist.rename', { workspaceId, watchlistId: selected.watchlistId, name, expectedStateVersion: selected.stateVersion }); await refresh(); setNotice('Watchlist renamed.'); } catch (nextError) { setError(nextError); throw nextError; } finally { setBusy(false); }
  };
  const removeList = async () => {
    if (!selected) return;
    setBusy(true); setError(undefined); setNotice('');
    try { await request('watchlist.delete', { workspaceId, watchlistId: selected.watchlistId, expectedStateVersion: selected.stateVersion }); await refresh(); setNotice('Watchlist deleted.'); } catch (nextError) { setError(nextError); } finally { setBusy(false); }
  };
  const memberMutation = async (instrumentId: string, command: 'watchlist.add' | 'watchlist.remove') => {
    if (!selected) return;
    setBusy(true); setError(undefined); setNotice('');
    try { await request(command, { workspaceId, watchlistId: selected.watchlistId, instrumentId, expectedStateVersion: selected.stateVersion }); await refresh(); setNotice(command === 'watchlist.add' ? 'Instrument added.' : 'Instrument removed.'); } catch (nextError) { setError(nextError); } finally { setBusy(false); }
  };
  return <><div className="page-heading"><h1>Watchlists</h1><p>Keep ordered canonical instruments in the local workspace. Refresh is on demand and coarse.</p></div>{state.isPending ? <p role="status">Loading watchlists…</p> : state.error ? <div className="error-banner" role="alert"><div><strong>Watchlists need attention</strong><p>{explainError(state.error)}</p></div><button type="button" onClick={() => { void state.refetch(); }}>Reload watchlists</button></div> : state.data ? <><div className="watchlist-toolbar"><button className="primary" type="button" onClick={() => setCreating(value => !value)} disabled={busy}>{creating ? 'Cancel new watchlist' : 'New watchlist'}</button>{creating && <form className="watchlist-create-form" onSubmit={submitCreate}><label htmlFor="watchlist-new-name">New watchlist name</label><div><input id="watchlist-new-name" value={newName} maxLength={80} onChange={event => setNewName(event.target.value)} placeholder="e.g. Core equities" /><button className="primary" type="submit" disabled={busy || !newName.trim()}>Create</button></div></form>}</div>{notice && <p className="watchlist-success" role="status" aria-live="polite">{notice}</p>}{error && <div className="watchlist-error" role="alert"><p>{mutationMessage(error)}</p><button type="button" onClick={() => { setError(undefined); void state.refetch(); }}>Reload</button></div>}<div className="watchlists-layout"><Library state={state.data} selectedId={selectedId} onSelect={setSelectedId} />{selected ? <Detail workspaceId={workspaceId} list={selected} catalogTerm={catalogTerm} setCatalogTerm={setCatalogTerm} onRename={rename} onDelete={removeList} onAdd={instrumentId => memberMutation(instrumentId, 'watchlist.add')} onRemove={instrumentId => memberMutation(instrumentId, 'watchlist.remove')} busy={busy} /> : <section className="card watchlist-detail-empty"><h2>{state.data.watchlists.length ? 'Select a watchlist' : 'Create your first watchlist'}</h2><p>{state.data.watchlists.length ? 'Choose a list from the library to inspect members and manage it.' : 'Your list will be stored in SQLite and available after the workspace reopens.'}</p></section>}</div></> : null}</>;
}
