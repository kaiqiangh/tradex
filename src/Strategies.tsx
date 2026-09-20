import { useEffect, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { StrategyRun, StrategyRunRequest, StrategySave, StrategyVersion } from '../shared/ipc-types.ts';
import { explainError, request } from './client.ts';

const activeStates = ['QUEUED', 'RUNNING'];

export function Strategies({ workspaceId }: { workspaceId: string }) {
  const queryClient = useQueryClient();
  const library = useQuery({ queryKey: ['strategies', workspaceId], queryFn: () => request('strategy.list', { workspaceId }) });
  const [selected, setSelected] = useState<StrategyVersion>();
  const [name, setName] = useState('My strategy');
  const [source, setSource] = useState('signal = HOLD\nreturn signal');
  const [language, setLanguage] = useState('python');
  const [runtime, setRuntime] = useState('sandbox-v1');
  const [instrumentId, setInstrumentId] = useState('equity:US:AAPL');
  const [datasetId, setDatasetId] = useState('historical:fixture');
  const [run, setRun] = useState<StrategyRun>();
  const [lastRequest, setLastRequest] = useState<StrategyRunRequest>();
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);

  const runQuery = useQuery({
    queryKey: ['strategy-run', workspaceId, run?.runId],
    queryFn: () => request('strategy.get_run', { workspaceId, runId: run!.runId }),
    enabled: Boolean(run && activeStates.includes(run.state)),
    refetchInterval: 250,
  });
  useEffect(() => {
    if (runQuery.data) setRun(runQuery.data);
  }, [runQuery.data]);
  const displayedRun = runQuery.data ?? run;

  const edit = (version: StrategyVersion) => {
    setSelected(version);
    setName(version.definition.name);
    setSource(version.definition.source);
    setLanguage(version.definition.language);
    setRuntime(version.definition.runtime);
    setError(undefined);
  };
  const save = async () => {
    setBusy(true); setError(undefined);
    try {
      const input: StrategySave = {
        workspaceId,
        ...(selected ? { strategyId: selected.strategyId } : {}),
        definition: { name: name.trim(), source, language: language.trim(), runtime: runtime.trim(), parameters: [] },
      };
      const version = await request('strategy.save_version', input);
      edit(version);
      await queryClient.invalidateQueries({ queryKey: ['strategies', workspaceId] });
    } catch (cause) { setError(explainError(cause)); }
    finally { setBusy(false); }
  };
  const buildRequest = (): StrategyRunRequest | undefined => selected ? ({
    workspaceId,
    strategyVersionId: selected.strategyVersionId,
    expectedStrategyHash: selected.sourceHash,
    instrumentId,
    datasetId,
    startAt: new Date(Date.now() - 86_400_000).toISOString(),
    endAt: new Date().toISOString(),
    parameters: [],
  }) : undefined;
  const execute = async (input = buildRequest()) => {
    if (!input) return;
    setBusy(true); setError(undefined); setLastRequest(input);
    try {
      setRun(await request('strategy.run', input));
      await queryClient.invalidateQueries({ queryKey: ['strategies', workspaceId] });
    } catch (cause) { setError(explainError(cause)); }
    finally { setBusy(false); }
  };
  const cancel = async () => {
    if (!displayedRun || !activeStates.includes(displayedRun.state)) return;
    setBusy(true); setError(undefined);
    try { setRun(await request('strategy.cancel', { workspaceId, runId: displayedRun.runId })); }
    catch (cause) { setError(explainError(cause)); }
    finally { setBusy(false); }
  };

  if (library.isPending) return <p role="status">Loading strategies…</p>;
  if (library.isError) return <div className="error-banner" role="alert"><p>Strategies are unavailable.</p><button type="button" onClick={() => void library.refetch()}>Reload strategies</button></div>;
  return <div className="strategy-surface">
    <div className="page-heading"><h1>Strategies</h1><p>Save immutable versions and inspect signal-only sandbox runs.</p></div>
    {error && <p className="error-banner" role="alert">{error}</p>}
    <div className="strategy-grid">
      <section className="card" aria-labelledby="strategy-editor-title">
        <h2 id="strategy-editor-title">Strategy editor</h2>
        <label className="field">Name<input value={name} maxLength={120} onChange={event => setName(event.target.value)} /></label>
        <label className="field">Language<input value={language} maxLength={32} onChange={event => setLanguage(event.target.value)} /></label>
        <label className="field">Runtime<input value={runtime} maxLength={64} onChange={event => setRuntime(event.target.value)} /></label>
        <label className="field">Source<textarea value={source} maxLength={100_000} rows={8} onChange={event => setSource(event.target.value)} /></label>
        <div className="form-actions"><button className="primary" type="button" disabled={busy || !name.trim() || !source.trim()} onClick={() => void save()}>Save version</button></div>
        {selected && <p className="form-hint">Selected {selected.strategyVersionId} · v{selected.revision} · {selected.sourceHash}</p>}
      </section>
      <section className="card" aria-labelledby="strategy-list-title">
        <h2 id="strategy-list-title">Saved versions</h2>
        {!library.data.versions.length && <p className="muted">No saved strategy versions.</p>}
        <ul className="strategy-list">{library.data.versions.map(version => <li key={version.strategyVersionId}>
          <button type="button" onClick={() => edit(version)} aria-pressed={selected?.strategyVersionId === version.strategyVersionId}><strong>{version.definition.name}</strong><span>v{version.revision} · {version.definition.language}</span><code>{version.sourceHash}</code></button>
        </li>)}</ul>
        <p className="form-hint">Saved versions are immutable. Save again to create a new revision.</p>
        {!!library.data.runs.length && <><h3>Recent runs</h3><ul className="strategy-run-list">{library.data.runs.map(item => <li key={item.runId}><code>{item.runId}</code><span>{item.state}</span></li>)}</ul></>}
      </section>
    </div>
    <section className="card strategy-run" aria-labelledby="strategy-run-title">
      <h2 id="strategy-run-title">Run signal-only strategy</h2>
      <div className="form-grid">
        <label className="field">Instrument<input value={instrumentId} onChange={event => setInstrumentId(event.target.value)} /></label>
        <label className="field">Dataset<input value={datasetId} onChange={event => setDatasetId(event.target.value)} /></label>
      </div>
      <div className="form-actions">
        <button type="button" className="primary" disabled={busy || !selected} onClick={() => void execute()}>Run selected version</button>
        <button type="button" disabled={busy || !displayedRun || !activeStates.includes(displayedRun.state)} onClick={() => void cancel()}>Cancel run</button>
        <button type="button" disabled={busy || !lastRequest || !displayedRun || activeStates.includes(displayedRun.state)} onClick={() => void execute(lastRequest)}>Retry run</button>
      </div>
      {displayedRun && <div className="strategy-result" aria-live="polite"><strong>{displayedRun.state}</strong><span>Run {displayedRun.runId}</span>{displayedRun.failure && <><p role="alert">{displayedRun.failure.code}: {displayedRun.failure.reason}</p>{displayedRun.failure.remediation?.length ? <ul><li>{displayedRun.failure.remediation.join(' · ')}</li></ul> : null}</>}{displayedRun.signal && <dl><div><dt>Instrument</dt><dd>{displayedRun.signal.instrumentId}</dd></div><div><dt>Direction</dt><dd>{displayedRun.signal.direction}</dd></div><div><dt>Exposure</dt><dd>{displayedRun.signal.desiredExposure}</dd></div><div><dt>Observed</dt><dd>{displayedRun.signal.observedAt}</dd></div><div><dt>Source</dt><dd>{displayedRun.signal.sourceRef}</dd></div><div><dt>Dataset</dt><dd>{displayedRun.signal.datasetId}</dd></div></dl>}{displayedRun.fixtureLabel && <p className="form-hint">Integration fixture: {displayedRun.fixtureLabel}</p>}</div>}
      {!selected && <p className="form-hint">Select a saved version before running.</p>}
    </section>
    <p className="notice">Strategy output is a signal only. Trade, approval, reservation and provider actions are unavailable here.</p>
  </div>;
}
