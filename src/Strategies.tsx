import { useEffect, useRef, useState } from 'react';
import type { FocusEvent } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { BacktestFixtureScenario, BacktestRun, BacktestRunRequest, StrategyFixtureScenario, StrategyRun, StrategyRunRequest, StrategySave, StrategyVersion, ThreadContextRef } from '../shared/ipc-types.ts';
import { browserIntegration, explainError, request } from './client.ts';

const activeStates = ['QUEUED', 'RUNNING'];
const backtestActiveStates = ['QUEUED', 'RUNNING'];
const localDateTime = (value: Date) => {
  const pad = (part: number) => String(part).padStart(2, '0');
  return `${value.getFullYear()}-${pad(value.getMonth() + 1)}-${pad(value.getDate())}T${pad(value.getHours())}:${pad(value.getMinutes())}`;
};
const requestTimestamp = (value: string) => value ? new Date(value).toISOString() : '';
const decimalInput = /^(?:\d+)(?:\.\d{0,18})?$/;
export type StrategyEntryContext = { source: 'RESEARCH' | 'BACKTEST'; contextRefs: ThreadContextRef[] };

export function Strategies({ workspaceId, entryContext }: { workspaceId: string; entryContext?: StrategyEntryContext }) {
  const queryClient = useQueryClient();
  const library = useQuery({ queryKey: ['strategies', workspaceId], queryFn: () => request('strategy.list', { workspaceId }) });
  const [selected, setSelected] = useState<StrategyVersion>();
  const [name, setName] = useState('My strategy');
  const [source, setSource] = useState('signal = HOLD\nreturn signal');
  const [language, setLanguage] = useState('python');
  const [runtime, setRuntime] = useState('sandbox-v1');
  const contextInstrumentId = entryContext?.contextRefs.find(context => context.kind === 'instrument')?.id;
  const [instrumentId, setInstrumentId] = useState(contextInstrumentId ?? 'equity:US:AAPL');
  const [datasetId, setDatasetId] = useState('historical:fixture');
  const [fixtureScenario, setFixtureScenario] = useState<StrategyFixtureScenario>('SUCCESS');
  const [run, setRun] = useState<StrategyRun>();
  const [lastRequest, setLastRequest] = useState<StrategyRunRequest>();
  const [barInterval, setBarInterval] = useState('1d');
  const [startAt, setStartAt] = useState(localDateTime(new Date(Date.now() - 86_400_000)));
  const [endAt, setEndAt] = useState(localDateTime(new Date()));
  const [startingCash, setStartingCash] = useState('100000');
  const [commission, setCommission] = useState('0');
  const [slippage, setSlippage] = useState('0');
  const [portfolioSeed, setPortfolioSeed] = useState('');
  const [backtestScenario, setBacktestScenario] = useState<BacktestFixtureScenario>('FAILURE');
  const [backtestRun, setBacktestRun] = useState<BacktestRun>();
  const [lastBacktestRequest, setLastBacktestRequest] = useState<BacktestRunRequest>();
  const [backtestFieldErrors, setBacktestFieldErrors] = useState<Record<string, string>>({});
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const focusedAction = useRef<HTMLButtonElement | null>(null);
  const retryAction = useRef<HTMLButtonElement | null>(null);
  const backtestFocusedAction = useRef<HTMLButtonElement | null>(null);
  const backtestRetryAction = useRef<HTMLButtonElement | null>(null);

  const rememberActionFocus = (event: FocusEvent<HTMLButtonElement>) => {
    focusedAction.current = event.currentTarget;
  };
  useEffect(() => {
    if (contextInstrumentId) setInstrumentId(contextInstrumentId);
  }, [contextInstrumentId]);

  const runQuery = useQuery({
    queryKey: ['strategy-run', workspaceId, run?.runId],
    queryFn: () => request('strategy.get_run', { workspaceId, runId: run!.runId }),
    enabled: Boolean(run && activeStates.includes(run.state)),
    refetchInterval: 250,
  });
  const backtestQuery = useQuery({
    queryKey: ['backtest-run', workspaceId, backtestRun?.runId],
    queryFn: () => request('backtest.get', { workspaceId, runId: backtestRun!.runId }),
    enabled: Boolean(backtestRun && backtestActiveStates.includes(backtestRun.state)),
    refetchInterval: 250,
  });
  useEffect(() => {
    if (runQuery.data) setRun(runQuery.data);
  }, [runQuery.data]);
  useEffect(() => {
    if (backtestQuery.data) setBacktestRun(backtestQuery.data);
  }, [backtestQuery.data]);
  const displayedRun = runQuery.data ?? run;
  useEffect(() => {
    if (!busy && focusedAction.current && displayedRun && !activeStates.includes(displayedRun.state)) {
      const target = focusedAction.current.disabled ? retryAction.current : focusedAction.current;
      target?.focus();
      focusedAction.current = null;
    }
  }, [busy, displayedRun?.state]);
  const displayedBacktestRun = backtestQuery.data ?? backtestRun;
  useEffect(() => {
    if (!busy && backtestFocusedAction.current && displayedBacktestRun && !backtestActiveStates.includes(displayedBacktestRun.state)) {
      const target = backtestFocusedAction.current.disabled ? backtestRetryAction.current : backtestFocusedAction.current;
      target?.focus();
      backtestFocusedAction.current = null;
    }
  }, [busy, displayedBacktestRun?.state]);

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
    ...(browserIntegration ? { fixtureScenario } : {}),
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
  const loadRun = async (runId: string) => {
    setBusy(true); setError(undefined);
    try {
      const loaded = await request('strategy.get_run', { workspaceId, runId });
      setRun(loaded);
      setInstrumentId(loaded.instrumentId);
      setDatasetId(loaded.datasetId);
      setLastRequest({
        workspaceId,
        strategyVersionId: loaded.strategyVersionId,
        expectedStrategyHash: loaded.strategyHash,
        instrumentId: loaded.instrumentId,
        datasetId: loaded.datasetId,
        startAt: loaded.startAt,
        endAt: loaded.endAt,
        parameters: loaded.parameters,
      });
      const version = library.data?.versions.find(item => item.strategyVersionId === loaded.strategyVersionId);
      if (version) edit(version);
    } catch (cause) { setError(explainError(cause)); }
    finally { setBusy(false); }
  };
  const cancel = async () => {
    if (!displayedRun || !activeStates.includes(displayedRun.state)) return;
    setBusy(true); setError(undefined);
    try {
      const cancelled = await request('strategy.cancel', { workspaceId, runId: displayedRun.runId });
      setRun(cancelled);
      await queryClient.invalidateQueries({ queryKey: ['strategy-run', workspaceId, displayedRun.runId] });
      await queryClient.invalidateQueries({ queryKey: ['strategies', workspaceId] });
    } catch (cause) {
      setError(explainError(cause));
      await runQuery.refetch();
    }
    finally { setBusy(false); }
  };
  const buildBacktestRequest = (): BacktestRunRequest | undefined => selected ? ({
    workspaceId,
    strategyVersionId: selected.strategyVersionId,
    expectedStrategyHash: selected.sourceHash,
    instrumentId,
    datasetId,
    startAt: requestTimestamp(startAt),
    endAt: requestTimestamp(endAt),
    barInterval,
    startingCash,
    commission,
    slippage,
    ...(portfolioSeed.trim() ? { portfolioSeed: portfolioSeed.trim() } : {}),
    parameters: [],
    ...(browserIntegration ? { fixtureScenario: backtestScenario } : {}),
  }) : undefined;
  const validateBacktestFields = () => {
    const next: Record<string, string> = {};
    if (!selected) next.strategy = 'Select a saved strategy version.';
    if (!instrumentId.trim()) next.instrument = 'Enter an instrument.';
    if (!datasetId.trim()) next.dataset = 'Enter a historical dataset.';
    if (!startAt) next.startAt = 'Choose a start date.';
    if (!endAt) next.endAt = 'Choose an end date.';
    if (startAt && endAt && new Date(startAt) >= new Date(endAt)) next.endAt = 'End date must be after the start date.';
    if (!decimalInput.test(startingCash) || startingCash === '0') next.startingCash = 'Starting cash must be a positive decimal with at most 18 fraction digits.';
    if (!decimalInput.test(commission)) next.commission = 'Commission must be a non-negative decimal with at most 18 fraction digits.';
    if (!decimalInput.test(slippage)) next.slippage = 'Slippage must be a non-negative decimal with at most 18 fraction digits.';
    setBacktestFieldErrors(next);
    if (Object.keys(next).length) { setError('Fix the highlighted backtest fields before running.'); return false; }
    return true;
  };
  const executeBacktest = async (input?: BacktestRunRequest) => {
    if (!input && !validateBacktestFields()) return;
    const requestInput = input ?? buildBacktestRequest();
    if (!requestInput) { setError('Select a saved version before running a backtest.'); return; }
    setBusy(true); setError(undefined); setLastBacktestRequest(requestInput);
    try {
      setBacktestRun(await request('backtest.run', requestInput));
    } catch (cause) { setError(explainError(cause)); }
    finally { setBusy(false); }
  };
  const cancelBacktest = async () => {
    if (!displayedBacktestRun || !backtestActiveStates.includes(displayedBacktestRun.state)) return;
    setBusy(true); setError(undefined);
    try {
      setBacktestRun(await request('backtest.cancel', { workspaceId, runId: displayedBacktestRun.runId }));
      await backtestQuery.refetch();
    } catch (cause) {
      setError(explainError(cause));
      await backtestQuery.refetch();
    }
    finally { setBusy(false); }
  };

  if (library.isPending) return <p role="status">Loading strategies…</p>;
  if (library.isError) return <div className="error-banner" role="alert"><p>Strategies are unavailable.</p><button type="button" onClick={() => void library.refetch()}>Reload strategies</button></div>;
  return <div className="strategy-surface">
    <div className="page-heading"><h1>Strategies</h1><p>Save immutable versions and inspect signal-only sandbox runs.</p></div>
    {entryContext && <section className="notice" aria-label="Strategy entry context"><strong>Opened from {entryContext.source} context</strong><p>{entryContext.contextRefs.length ? entryContext.contextRefs.map(context => `${context.kind}:${context.id}#${context.hash}`).join(' · ') : 'No context reference attached; choose an instrument before running.'}</p></section>}
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
        {!!library.data.runs.length && <><h3>Recent runs</h3><ul className="strategy-run-list">{library.data.runs.map(item => <li key={item.runId}><button type="button" onClick={() => void loadRun(item.runId)} disabled={busy}><code>{item.runId}</code><span>{item.state}</span></button></li>)}</ul></>}
      </section>
    </div>
    <section className="card strategy-run" aria-labelledby="strategy-run-title">
      <h2 id="strategy-run-title">Run signal-only strategy</h2>
      <div className="form-grid">
        <label className="field">Instrument<input value={instrumentId} onChange={event => setInstrumentId(event.target.value)} /></label>
        <label className="field">Dataset<input value={datasetId} onChange={event => setDatasetId(event.target.value)} /></label>
        {browserIntegration && <label className="field">Integration scenario<select value={fixtureScenario} onChange={event => setFixtureScenario(event.target.value as StrategyFixtureScenario)}><option value="SUCCESS">Success</option><option value="FAILURE">Failure</option><option value="CANCELLED">Cancelled</option></select></label>}
      </div>
      <div className="form-actions">
        <button type="button" className="primary" disabled={busy || !selected} onFocus={rememberActionFocus} onClick={event => { focusedAction.current = event.currentTarget; void execute(); }}>Run selected version</button>
        <button type="button" disabled={busy || !displayedRun || !activeStates.includes(displayedRun.state)} onFocus={rememberActionFocus} onClick={event => { focusedAction.current = event.currentTarget; void cancel(); }}>Cancel run</button>
        <button ref={retryAction} type="button" disabled={busy || !lastRequest || !displayedRun || activeStates.includes(displayedRun.state)} onFocus={rememberActionFocus} onClick={event => { focusedAction.current = event.currentTarget; void execute(lastRequest); }}>Retry run</button>
      </div>
      {displayedRun && <div className="strategy-result" aria-live="polite"><strong>{displayedRun.state}</strong><span>Run {displayedRun.runId}</span>{displayedRun.failure && <><p role="alert">{displayedRun.failure.code}: {displayedRun.failure.reason}</p>{displayedRun.failure.remediation?.length ? <ul><li>{displayedRun.failure.remediation.join(' · ')}</li></ul> : null}</>}{displayedRun.signal && <dl><div><dt>Instrument</dt><dd>{displayedRun.signal.instrumentId}</dd></div><div><dt>Direction</dt><dd>{displayedRun.signal.direction}</dd></div><div><dt>Exposure</dt><dd>{displayedRun.signal.desiredExposure}</dd></div><div><dt>Observed</dt><dd>{displayedRun.signal.observedAt}</dd></div><div><dt>Source</dt><dd>{displayedRun.signal.sourceRef}</dd></div><div><dt>Dataset</dt><dd>{displayedRun.signal.datasetId}</dd></div></dl>}{displayedRun.fixtureLabel && <p className="form-hint">Integration fixture: {displayedRun.fixtureLabel}</p>}</div>}
      {!selected && <p className="form-hint">Select a saved version before running.</p>}
    </section>
    <section className="card strategy-run" aria-labelledby="backtest-title">
      <h2 id="backtest-title">Backtest</h2>
      <p className="form-hint">Historical simulation only. Backtests never place trades or call a provider.</p>
      <div className="form-grid">
        <label className="field">Start date<input type="datetime-local" value={startAt} aria-invalid={Boolean(backtestFieldErrors.startAt)} aria-describedby={backtestFieldErrors.startAt ? 'backtest-start-error' : undefined} onChange={event => setStartAt(event.target.value)} />{backtestFieldErrors.startAt && <span id="backtest-start-error" className="error-text">{backtestFieldErrors.startAt}</span>}</label>
        <label className="field">End date<input type="datetime-local" value={endAt} aria-invalid={Boolean(backtestFieldErrors.endAt)} aria-describedby={backtestFieldErrors.endAt ? 'backtest-end-error' : undefined} onChange={event => setEndAt(event.target.value)} />{backtestFieldErrors.endAt && <span id="backtest-end-error" className="error-text">{backtestFieldErrors.endAt}</span>}</label>
        <label className="field">Bar interval<select value={barInterval} onChange={event => setBarInterval(event.target.value)}><option value="1m">1 minute</option><option value="5m">5 minutes</option><option value="15m">15 minutes</option><option value="30m">30 minutes</option><option value="1h">1 hour</option><option value="1d">1 day</option></select></label>
        <label className="field">Starting cash<input inputMode="decimal" value={startingCash} aria-invalid={Boolean(backtestFieldErrors.startingCash)} aria-describedby={backtestFieldErrors.startingCash ? 'backtest-cash-error' : undefined} onChange={event => setStartingCash(event.target.value)} />{backtestFieldErrors.startingCash && <span id="backtest-cash-error" className="error-text">{backtestFieldErrors.startingCash}</span>}</label>
        <label className="field">Commission<input inputMode="decimal" value={commission} aria-invalid={Boolean(backtestFieldErrors.commission)} aria-describedby={backtestFieldErrors.commission ? 'backtest-commission-error' : undefined} onChange={event => setCommission(event.target.value)} />{backtestFieldErrors.commission && <span id="backtest-commission-error" className="error-text">{backtestFieldErrors.commission}</span>}</label>
        <label className="field">Slippage<input inputMode="decimal" value={slippage} aria-invalid={Boolean(backtestFieldErrors.slippage)} aria-describedby={backtestFieldErrors.slippage ? 'backtest-slippage-error' : undefined} onChange={event => setSlippage(event.target.value)} />{backtestFieldErrors.slippage && <span id="backtest-slippage-error" className="error-text">{backtestFieldErrors.slippage}</span>}</label>
        <label className="field">Portfolio seed<input value={portfolioSeed} onChange={event => setPortfolioSeed(event.target.value)} placeholder="Optional" /></label>
        {browserIntegration && <label className="field">Backtest integration scenario<select aria-label="Backtest integration scenario" value={backtestScenario} onChange={event => setBacktestScenario(event.target.value as BacktestFixtureScenario)}><option value="FAILURE">Failure</option><option value="CANCELLED">Cancelled</option></select></label>}
      </div>
      <div className="form-actions">
        <button type="button" className="primary" disabled={busy || !selected} onFocus={event => { backtestFocusedAction.current = event.currentTarget; }} onClick={event => { backtestFocusedAction.current = event.currentTarget; void executeBacktest(); }}>Run backtest</button>
        <button type="button" disabled={busy || !displayedBacktestRun || !backtestActiveStates.includes(displayedBacktestRun.state)} onFocus={event => { backtestFocusedAction.current = event.currentTarget; }} onClick={event => { backtestFocusedAction.current = event.currentTarget; void cancelBacktest(); }}>Cancel backtest</button>
        <button ref={backtestRetryAction} type="button" disabled={busy || !lastBacktestRequest || !displayedBacktestRun || backtestActiveStates.includes(displayedBacktestRun.state)} onFocus={event => { backtestFocusedAction.current = event.currentTarget; }} onClick={event => { backtestFocusedAction.current = event.currentTarget; void executeBacktest(lastBacktestRequest); }}>Retry backtest</button>
      </div>
      {displayedBacktestRun && <div className="strategy-result" aria-live="polite"><strong>{displayedBacktestRun.state}</strong><span>Run {displayedBacktestRun.runId}</span><span>Config {displayedBacktestRun.requestHash}</span>{displayedBacktestRun.failure && <><p role="alert">{displayedBacktestRun.failure.code}: {displayedBacktestRun.failure.reason}</p>{displayedBacktestRun.failure.remediation?.length ? <ul><li>{displayedBacktestRun.failure.remediation.join(' · ')}</li></ul> : null}</>}{displayedBacktestRun.fixtureLabel && <p className="form-hint">Integration fixture: {displayedBacktestRun.fixtureLabel}</p>}</div>}
      {!selected && <p className="form-hint">Select a saved version before running a backtest.</p>}
    </section>
    <p className="notice">Strategy output is a signal only. Trade, approval, reservation and provider actions are unavailable here.</p>
  </div>;
}
