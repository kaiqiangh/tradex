import { useEffect, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { createPortal } from 'react-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { Artifact, ArtifactKind, ArtifactProvenance, ArtifactSummary, ResearchToolResult, ThreadItem, ThreadTurn } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';

function ArtifactSourceList({ result }: { result?: ResearchToolResult | null }) {
  if (!result) return <p className="muted">No typed research result was attached.</p>;
  return <>
    <p><strong>State:</strong> {result.payload.state} · <strong>Tool:</strong> {result.toolId}</p>
    {result.payload.conclusion && <p><strong>Conclusion:</strong> {result.payload.conclusion}</p>}
    {(result.payload.findings ?? []).length > 0 && <ul>{(result.payload.findings ?? []).map(finding => <li key={`${finding.title}:${finding.detail}`}><strong>{finding.title}:</strong> {finding.detail}</li>)}</ul>}
    {(result.payload.limitations ?? []).length > 0 && <p><strong>Limitations:</strong> {(result.payload.limitations ?? []).join(' · ')}</p>}
  </>;
}

function ProvenanceModal({ provenance, onClose }: { provenance: ArtifactProvenance; onClose: () => void }) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    closeRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => { if (event.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKeyDown);
    return () => { window.removeEventListener('keydown', onKeyDown); if (previous?.isConnected) previous.focus(); };
  }, [onClose]);
  useEffect(() => {
    const shell = document.querySelector<HTMLElement>('.app-shell');
    if (!shell) return;
    shell.inert = true;
    return () => { shell.inert = false; };
  }, []);
  return createPortal(<div className="picker-backdrop">
    <div className="picker-dialog provenance-dialog" role="dialog" aria-modal="true" aria-labelledby="artifact-provenance-title" ref={dialogRef}>
      <div className="picker-dialog-heading"><div><h2 id="artifact-provenance-title">Artifact provenance</h2><p className="muted">Historical source identity is independent of current selectors.</p></div><button ref={closeRef} type="button" aria-label="Close provenance" onClick={onClose}>×</button></div>
      <dl className="artifact-provenance-grid">
        <div><dt>Workspace</dt><dd className="identity">{provenance.workspaceId}</dd></div>
        <div><dt>Thread</dt><dd className="identity">{provenance.threadId}</dd></div>
        <div><dt>Turn / Item</dt><dd className="identity">{provenance.turnId} / {provenance.itemId}</dd></div>
        <div><dt>Agent mode</dt><dd>{provenance.turnSnapshot.agentMode}</dd></div>
        <div><dt>Execution context</dt><dd>{provenance.turnSnapshot.executionContext}</dd></div>
        <div><dt>Account</dt><dd>{provenance.turnSnapshot.accountId ?? 'None'}{provenance.turnSnapshot.accountEnvironment ? ` · ${provenance.turnSnapshot.accountEnvironment}` : ''}</dd></div>
        <div><dt>Model</dt><dd>{provenance.turnSnapshot.model ? `${provenance.turnSnapshot.model.provider} · ${provenance.turnSnapshot.model.modelId}` : 'Unavailable'}</dd></div>
        <div><dt>Attached context</dt><dd>{provenance.turnSnapshot.attachedContexts.length ? provenance.turnSnapshot.attachedContexts.map(context => `${context.kind}:${context.id}#${context.hash}`).join(', ') : 'None'}</dd></div>
      </dl>
      <h3>Provider attempts</h3>
      {provenance.providerAttempts.length ? <ul>{provenance.providerAttempts.map(attempt => <li key={attempt.attemptId}>{attempt.provider} · {attempt.modelId} · {attempt.outcome}{attempt.errorCode ? ` · ${attempt.errorCode}` : ''}</li>)}</ul> : <p className="muted">No provider attempts recorded.</p>}
      <h3>Sources</h3>
      {provenance.sources.length ? <ul>{provenance.sources.map(source => <li key={`${source.sourceId}:${source.receivedTimestamp}`}><strong>{source.sourceId}</strong> · {source.provider} · {source.freshness} · {source.quality} · provider {source.providerTimestamp ?? 'unavailable'} · received {source.receivedTimestamp}{source.limitation ? ` · ${source.limitation}` : ''}</li>)}</ul> : <p className="muted">No source provenance was attached.</p>}
      <h3>Related references</h3>
      <ul><li>Market snapshots: {provenance.marketSnapshotHashes.length ? provenance.marketSnapshotHashes.join(', ') : 'None'}</li><li>Datasets: {provenance.datasetHashes.length ? provenance.datasetHashes.join(', ') : 'None'}</li><li>Orders: {provenance.relatedOrderIds.length ? provenance.relatedOrderIds.join(', ') : 'None'}</li></ul>
      <div className="picker-dialog-actions"><button type="button" className="primary" onClick={onClose}>Done</button></div>
    </div>
  </div>, document.body);
}

function ArtifactRow({ artifact, selected, onSelect }: { artifact: ArtifactSummary; selected: boolean; onSelect: () => void }) {
  return <button type="button" className={`artifact-row${selected ? ' selected' : ''}`} aria-current={selected ? 'true' : undefined} onClick={onSelect}>
    <span><strong>{artifact.title}</strong><small>{artifact.kind} · {new Date(artifact.createdAt).toLocaleString()}</small></span>
    <span className="artifact-row-meta"><small>Thread {artifact.threadId.slice(0, 8)}</small><code>{artifact.contentHash.slice(0, 16)}…</code></span>
  </button>;
}

function ArtifactDetail({ workspaceId, artifactId }: { workspaceId: string; artifactId: string }) {
  const queryClient = useQueryClient();
  const [provenanceOpen, setProvenanceOpen] = useState(false);
  const [exportStatus, setExportStatus] = useState<string>();
  const [exportError, setExportError] = useState<string>();
  const detail = useQuery({ queryKey: ['artifact', workspaceId, artifactId], queryFn: () => request('artifact.get', { workspaceId, artifactId }) });
  if (detail.isPending) return <section className="card artifact-detail" role="status">Loading artifact…</section>;
  if (detail.isError) return <section className="card artifact-detail" role="alert"><p>{explainError(detail.error)}</p><button type="button" onClick={() => void detail.refetch()}>Reload artifact</button></section>;
  const artifact: Artifact = detail.data;
  const exportArtifact = async () => {
    setExportStatus(undefined); setExportError(undefined);
    try {
      const result = await request('artifact.export', { workspaceId, artifactId });
      setExportStatus(`Exported ${result.bytes} bytes to ${result.path}`);
      await queryClient.invalidateQueries({ queryKey: ['artifacts', workspaceId] });
    } catch (error) { setExportError(explainError(error)); }
  };
  return <section className="card artifact-detail" aria-labelledby="artifact-detail-title">
    <div className="account-heading"><div><h2 id="artifact-detail-title">{artifact.title}</h2><p className="muted">{artifact.kind} · version {artifact.version}</p></div><span className="badge">Saved locally</span></div>
    <div className="artifact-actions"><button type="button" onClick={() => setProvenanceOpen(true)}>View provenance</button><button type="button" className="primary" onClick={() => void exportArtifact()}>Export JSON</button></div>
    <dl className="artifact-meta"><div><dt>Content hash</dt><dd className="identity">{artifact.contentHash}</dd></div><div><dt>Created</dt><dd><time dateTime={artifact.createdAt}>{new Date(artifact.createdAt).toLocaleString()}</time></dd></div><div><dt>Source Item</dt><dd className="identity">{artifact.provenance.itemId}</dd></div></dl>
    <article className="artifact-content"><h3>Saved content</h3><p>{artifact.content.text}</p><ArtifactSourceList result={artifact.content.researchResult} /></article>
    {exportStatus && <p className="notice" role="status" aria-live="polite">{exportStatus}</p>}
    {exportError && <p className="error-text" role="alert">{exportError}</p>}
    {provenanceOpen && <ProvenanceModal provenance={artifact.provenance} onClose={() => setProvenanceOpen(false)} />}
  </section>;
}

function SaveArtifactForm({ workspaceId, thread, turn, item, onSaved, onCancel }: { workspaceId: string; thread: { threadId: string }; turn: ThreadTurn; item: ThreadItem; onSaved: () => void; onCancel: () => void }) {
  const [title, setTitle] = useState(item.researchResult ? 'Research result' : 'Saved decision');
  const [kind, setKind] = useState<ArtifactKind>(item.researchResult ? 'RESEARCH' : 'DECISION');
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const save = async (event: FormEvent) => {
    event.preventDefault(); setBusy(true); setError(undefined);
    try { await request('artifact.save', { workspaceId, threadId: thread.threadId, turnId: turn.turnId, itemId: item.itemId, kind, title: title.trim() }); onSaved(); }
    catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };
  return <form className="artifact-save-form" onSubmit={save}><label className="field">Artifact title<input value={title} onChange={event => setTitle(event.target.value)} maxLength={120} required disabled={busy} /></label><label className="field">Type<select value={kind} onChange={event => setKind(event.target.value as ArtifactKind)} disabled={busy}><option value="RESEARCH">Research</option><option value="DECISION">Decision</option></select></label>{error && <p className="error-text" role="alert">{error}</p>}<div className="form-actions"><button type="button" onClick={onCancel} disabled={busy}>Cancel</button><button className="primary" type="submit" disabled={busy || !title.trim()}>{busy ? 'Saving…' : 'Save artifact'}</button></div></form>;
}

export function SaveArtifactAction({ workspaceId, thread, turn, item }: { workspaceId: string; thread: { threadId: string }; turn: ThreadTurn; item: ThreadItem }) {
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [saved, setSaved] = useState(false);
  if (item.status !== 'COMPLETED' || saved) return saved ? <span className="badge" role="status">Artifact saved</span> : null;
  return open ? <SaveArtifactForm workspaceId={workspaceId} thread={thread} turn={turn} item={item} onSaved={() => { setSaved(true); setOpen(false); void queryClient.invalidateQueries({ queryKey: ['artifacts', workspaceId] }); }} onCancel={() => setOpen(false)} /> : <button type="button" onClick={() => setOpen(true)}>Save as artifact</button>;
}

export function ArtifactsPage({ workspaceId }: { workspaceId: string }) {
  const library = useQuery({ queryKey: ['artifacts', workspaceId], queryFn: () => request('artifact.list', { workspaceId }) });
  const [selectedId, setSelectedId] = useState<string>();
  useEffect(() => { if (library.data && selectedId && !library.data.artifacts.some(artifact => artifact.artifactId === selectedId)) setSelectedId(undefined); }, [library.data, selectedId]);
  if (library.isPending) return <><div className="page-heading"><h1>Artifacts</h1><p>Saved research and decision provenance.</p></div><p role="status">Loading artifacts…</p></>;
  if (library.isError) return <><div className="page-heading"><h1>Artifacts</h1><p>Saved research and decision provenance.</p></div><section className="card empty-page" role="alert"><h2>Artifacts unavailable</h2><p>{explainError(library.error)}</p><button type="button" onClick={() => void library.refetch()}>Reload artifacts</button></section></>;
  const artifacts = library.data.artifacts;
  return <><div className="page-heading"><div><h1>Artifacts</h1><p>Saved research and decision provenance stays attached to this workspace.</p></div><span className="badge">{artifacts.length} saved</span></div>{artifacts.length === 0 ? <section className="card empty-page"><h2>No artifacts</h2><p>Save a completed research or decision Item from a Thread to build this library.</p></section> : <div className="artifacts-layout"><section className="card artifact-library" aria-label="Artifact library">{artifacts.map(artifact => <ArtifactRow key={artifact.artifactId} artifact={artifact} selected={artifact.artifactId === selectedId} onSelect={() => setSelectedId(artifact.artifactId)} />)}</section>{selectedId ? <ArtifactDetail workspaceId={workspaceId} artifactId={selectedId} /> : <section className="card empty-page"><h2>Select an artifact</h2><p>Open an item to inspect content and provenance.</p></section>}</div>}</>;
}
