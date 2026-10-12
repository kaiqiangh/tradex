import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { Workspace, WorkspaceArchiveDataCategoryId, WorkspaceArchivePreview } from '../shared/ipc-types.ts';
import { desktop, explainError, request } from './client.ts';

function daysValue(value: number | null | undefined) {
  return value == null ? '' : String(value);
}

function retentionDays(value: string): number | null {
  return value.trim() === '' ? null : Number(value);
}

function archiveCategoryLabel(categoryId: WorkspaceArchiveDataCategoryId) {
  switch (categoryId) {
    case 'ACCOUNTS': return 'Accounts';
    case 'CONVERSATIONS': return 'Conversations';
    case 'RESEARCH': return 'Research';
    case 'STRATEGIES_AND_BACKTESTS': return 'Strategies and backtests';
    case 'FINANCIAL_HISTORY': return 'Financial history';
  }
}

export function WorkspaceArchive({ workspace, onOpenRestored }: { workspace: Workspace; onOpenRestored: (path: string) => Promise<void> }) {
  const queryClient = useQueryClient();
  const retention = useQuery({
    queryKey: ['workspace-retention', workspace.workspaceId],
    queryFn: () => request('workspace.retention.get', { workspaceId: workspace.workspaceId }),
  });
  const [artifactDays, setArtifactDays] = useState('');
  const [cacheDays, setCacheDays] = useState('');
  const [marketDays, setMarketDays] = useState('');
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string>();
  const [error, setError] = useState<string>();
  const [importOpen, setImportOpen] = useState(false);
  const [archivePath, setArchivePath] = useState<string>();
  const [preview, setPreview] = useState<WorkspaceArchivePreview>();
  const [destinationParent, setDestinationParent] = useState<string>();
  const [confirmRetention, setConfirmRetention] = useState(false);
  const importTriggerRef = useRef<HTMLButtonElement>(null);
  const importDialogRef = useRef<HTMLElement>(null);
  const archiveButtonRef = useRef<HTMLButtonElement>(null);
  const destinationButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!retention.data) return;
    setArtifactDays(daysValue(retention.data.artifactRetentionDays));
    setCacheDays(daysValue(retention.data.cacheRetentionDays));
    setMarketDays(daysValue(retention.data.marketDataRetentionDays));
  }, [retention.data]);

  useEffect(() => {
    if (!importOpen) return;
    const previousFocus = document.activeElement as HTMLElement | null;
    queueMicrotask(() => (preview ? destinationButtonRef.current : archiveButtonRef.current)?.focus());
    const shell = document.querySelector<HTMLElement>('.app-shell');
    if (shell) shell.inert = true;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        if (!busy) setImportOpen(false);
        return;
      }
      if (event.key !== 'Tab') return;
      const focusable = [...(importDialogRef.current?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), [tabindex="0"]') ?? [])];
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      if (shell) shell.inert = false;
      queueMicrotask(() => { if (importTriggerRef.current?.isConnected) importTriggerRef.current.focus(); else if (previousFocus?.isConnected) previousFocus.focus(); });
    };
  }, [importOpen, preview, busy]);

  const saveRetention = async () => {
    if (!retention.data) return;
    setBusy(true); setError(undefined); setStatus(undefined);
    try {
      const saved = await request('workspace.retention.set', {
        workspaceId: workspace.workspaceId,
        expectedStateVersion: retention.data.stateVersion,
        artifactRetentionDays: retentionDays(artifactDays),
        cacheRetentionDays: retentionDays(cacheDays),
        marketDataRetentionDays: retentionDays(marketDays),
      });
      queryClient.setQueryData(['workspace-retention', workspace.workspaceId], saved);
      setStatus('Retention settings saved. They only take effect when you apply them.');
    } catch (failure) {
      setError(explainError(failure));
      await retention.refetch();
    } finally { setBusy(false); }
  };

  const applyRetention = async () => {
    setBusy(true); setError(undefined); setStatus(undefined); setConfirmRetention(false);
    try {
      const applied = await request('workspace.retention.apply', { workspaceId: workspace.workspaceId });
      queryClient.setQueryData(['workspace-retention', workspace.workspaceId], applied);
      await queryClient.invalidateQueries({ queryKey: ['artifacts', workspace.workspaceId] });
      setStatus(`Retention applied: ${applied.lastDeletedArtifacts} ordinary artifacts, ${applied.lastDeletedMarketRows} market-data rows and ${applied.lastDeletedCacheFiles} cache files removed. Financial execution and reconciliation records are retained.`);
    } catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };

  const backupNow = async () => {
    setBusy(true); setError(undefined); setStatus(undefined);
    try {
      const result = await request('workspace.archive.backup', { workspaceId: workspace.workspaceId });
      setStatus(`Backup saved (${result.bytes} bytes): ${result.path}`);
    } catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };

  const exportWorkspace = async () => {
    setError(undefined); setStatus(undefined);
    try {
      const selectedPath = await save({
        defaultPath: `${workspace.name.replace(/[^A-Za-z0-9._-]+/g, '_')}.tradex`,
      });
      if (!selectedPath) return;
      const destinationPath = selectedPath.toLowerCase().endsWith('.tradex')
        ? selectedPath
        : `${selectedPath}.tradex`;
      setBusy(true);
      const result = await request('workspace.archive.export', { workspaceId: workspace.workspaceId, destinationPath });
      setStatus(`Workspace exported (${result.bytes} bytes): ${result.path}`);
    } catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };

  const chooseArchive = async () => {
    setError(undefined); setStatus(undefined); setPreview(undefined); setDestinationParent(undefined);
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: 'Choose a TradeX workspace archive',
        filters: [{ name: 'TradeX workspace archive', extensions: ['tradex'] }],
      });
      if (typeof selected !== 'string') return;
      setBusy(true); setArchivePath(selected);
      const result = await request('workspace.archive.preview', { workspaceId: workspace.workspaceId, archivePath: selected });
      setPreview(result);
    } catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };

  const chooseDestination = async () => {
    setError(undefined);
    try {
      const selected = await open({ directory: true, multiple: false, title: 'Choose where to restore the workspace' });
      if (typeof selected === 'string') setDestinationParent(selected);
    } catch (failure) { setError(explainError(failure)); }
  };

  const restore = async () => {
    if (!archivePath || !preview || !destinationParent) return;
    setBusy(true); setError(undefined); setStatus(undefined);
    try {
      const restored = await request('workspace.archive.restore', {
        workspaceId: workspace.workspaceId,
        archivePath,
        destinationParent,
        manifestHash: preview.manifestHash,
      });
      await onOpenRestored(restored.restoredPath);
      setImportOpen(false); setPreview(undefined); setArchivePath(undefined); setDestinationParent(undefined);
      setStatus(`Restored workspace opened. A pre-restore backup is saved at ${restored.backupPath}`);
    } catch (failure) { setError(explainError(failure)); }
    finally { setBusy(false); }
  };

  const retentionConfigured = retention.data && (retention.data.artifactRetentionDays != null || retention.data.cacheRetentionDays != null || retention.data.marketDataRetentionDays != null);
  const importDialog = importOpen ? createPortal(<div className="picker-backdrop">
    <section ref={importDialogRef} className="picker-dialog workspace-import-dialog" role="dialog" aria-modal="true" aria-labelledby="workspace-import-title">
      <div className="picker-dialog-heading"><div><h2 id="workspace-import-title">Import / Restore Workspace</h2><p className="muted">The archive is validated before you confirm. Restore creates a new workspace and preserves the current one.</p></div><button type="button" aria-label="Close import dialog" onClick={() => setImportOpen(false)} disabled={busy}>×</button></div>
      {!preview ? <button ref={archiveButtonRef} type="button" className="primary" onClick={() => void chooseArchive()} disabled={busy}>{busy ? 'Validating archive…' : 'Choose archive'}</button> : <>
        <p className="workspace-archive-validated" role="status">Archive validated; the stored schema is supported by this TradeX version.</p>
        <dl className="artifact-meta"><div><dt>Workspace</dt><dd>{preview.workspaceName}</dd></div><div><dt>Workspace ID</dt><dd className="identity">{preview.workspaceId}</dd></div><div><dt>Captured</dt><dd><time dateTime={preview.capturedAt}>{new Date(preview.capturedAt).toLocaleString()}</time></dd></div><div><dt>Schema / members</dt><dd>{preview.storageSchemaVersion} · {preview.memberCount}</dd></div><div><dt>Archive size</dt><dd>{preview.bytes.toLocaleString()} bytes</dd></div></dl>
        <section className="workspace-data-categories" aria-labelledby="workspace-data-heading"><h3 id="workspace-data-heading">Included data</h3><dl>{preview.dataCategories.map(category => <div key={category.categoryId}><dt>{archiveCategoryLabel(category.categoryId)}</dt><dd>{category.recordCount.toLocaleString()} records</dd></div>)}</dl></section>
        {preview.credentialChecks.length > 0 && <div className="workspace-credential-checks" aria-label="Credential reference checks"><h3>Saved account credentials</h3><ul>{preview.credentialChecks.map(check => <li key={check.connectionId}><strong>{check.label}</strong> <span>({check.providerId} {check.environment})</span><p>{check.status === 'PRESENT_UNVERIFIED' ? 'Keychain entry exists. The account still needs fresh authentication and reconciliation after restore.' : check.status === 'MISSING' ? 'Keychain entry is missing. The restored account will remain blocked until reconnected.' : check.status === 'UNAVAILABLE' ? 'Keychain could not be checked. The restored account will remain unchecked and blocked.' : 'Keychain entry has not been checked; the restored account will remain unchecked and blocked.'}</p></li>)}</ul></div>}
        <ul>{preview.warnings.map(warning => <li key={warning}>{warning}</li>)}</ul>
        <div className="field"><span>Restore destination</span><div className="folder-input"><span className="path">{destinationParent ?? 'Choose a folder for the restored workspace'}</span><button ref={destinationButtonRef} type="button" onClick={() => void chooseDestination()} disabled={busy}>Choose folder…</button></div></div>
        <div className="notice"><strong>Before restoring</strong><p>TradeX will save a backup of the current workspace, verify the archive again, migrate the restored database, invalidate issued approvals, and leave Live accounts disarmed until they pass fresh health and reconciliation checks. No provider request is made.</p></div>
      </>}
      {error && <p role="alert" className="error-text">{error}</p>}
      <div className="picker-dialog-actions"><button type="button" onClick={() => setImportOpen(false)} disabled={busy}>Cancel</button>{preview && <button type="button" className="primary" onClick={() => void restore()} disabled={busy || !destinationParent}>{busy ? 'Restoring…' : 'Confirm restore'}</button>}</div>
    </section>
  </div>, document.body) : null;

  return <section className="workspace-archive card" aria-labelledby="workspace-archive-heading">
    <div className="account-heading"><div><h3 id="workspace-archive-heading">Workspace backup &amp; retention</h3><p className="muted">SQLite stores workspace state; DuckDB stores historical market data; files store exports and backups.</p></div></div>
    <div className="artifact-actions">
      <button type="button" disabled={!desktop || busy} onClick={() => void backupNow()}>{busy ? 'Creating backup…' : 'Backup Now'}</button>
      <button type="button" disabled={!desktop || busy} onClick={() => void exportWorkspace()}>{busy ? 'Exporting…' : 'Export Workspace'}</button>
      <button ref={importTriggerRef} type="button" className="primary" disabled={!desktop || busy} onClick={() => { setImportOpen(true); setPreview(undefined); setArchivePath(undefined); setDestinationParent(undefined); setError(undefined); }}>Import / Restore</button>
    </div>
    {!desktop && <p className="muted">Backup, export and restore use native local-file dialogs in the TradeX desktop app.</p>}
    <div className="notice"><strong>Privacy and restore safety</strong><p>Archives contain non-secret SQLite workspace state. Broker/API secrets, market.duckdb and cache files are excluded. Restored account references stay unchecked, Live remains disarmed, and accounts need fresh authentication and reconciliation before use. Existing orders, reservations and audit history are retained.</p></div>

    <div className="workspace-retention" aria-labelledby="workspace-retention-heading">
      <h4 id="workspace-retention-heading">Retention limits</h4>
      <p className="muted">Set a number of days or leave a field blank for no automatic limit. Saving never deletes data; Apply retention now performs the removal.</p>
      {retention.isPending ? <p role="status">Loading retention settings…</p> : retention.isError ? <div role="alert"><p>{explainError(retention.error)}</p><button type="button" onClick={() => void retention.refetch()}>Reload retention settings</button></div> : <>
        <div className="workspace-retention-fields">
          <label className="field">Ordinary artifacts (days)<input type="number" min="1" max="3650" value={artifactDays} onChange={event => setArtifactDays(event.target.value)} placeholder="No limit" disabled={busy} /></label>
          <label className="field">Market data (days)<input type="number" min="1" max="3650" value={marketDays} onChange={event => setMarketDays(event.target.value)} placeholder="No limit" disabled={busy} /></label>
          <label className="field">Cache files (days)<input type="number" min="1" max="3650" value={cacheDays} onChange={event => setCacheDays(event.target.value)} placeholder="No limit" disabled={busy} /></label>
        </div>
        <p className="form-hint">Artifacts linked to order IDs, unresolved executions, reservations, settlements, approvals and reconciliation records are kept. Cache cleanup only scans regular files directly inside the workspace’s cache folder.</p>
        <div className="form-actions"><button type="button" onClick={() => void saveRetention()} disabled={busy}>{busy ? 'Saving…' : 'Save retention settings'}</button><button type="button" className="primary" onClick={() => setConfirmRetention(true)} disabled={busy || !retentionConfigured}>{busy ? 'Applying…' : 'Apply retention now'}</button></div>
        {retention.data.lastAppliedAt && <p className="form-hint">Last applied {new Date(retention.data.lastAppliedAt).toLocaleString()}: {retention.data.lastDeletedArtifacts} artifacts, {retention.data.lastDeletedMarketRows} market rows and {retention.data.lastDeletedCacheFiles} cache files removed.</p>}
      </>}
    </div>

    {status && <p role="status" aria-live="polite">{status}</p>}
    {error && !importOpen && <p role="alert" className="error-text">{error}</p>}

    {importDialog}

    {confirmRetention && <section className="notice retention-confirmation" aria-labelledby="retention-confirm-title"><strong id="retention-confirm-title">Apply retention limits?</strong><p>This permanently removes expired ordinary artifacts, market-data rows and cache files. Financial execution, order, reservation, settlement, approval and reconciliation records remain.</p><div className="picker-dialog-actions"><button type="button" onClick={() => setConfirmRetention(false)} disabled={busy}>Cancel</button><button type="button" className="primary" onClick={() => void applyRetention()} disabled={busy}>{busy ? 'Applying…' : 'Confirm apply'}</button></div></section>}
  </section>;
}
