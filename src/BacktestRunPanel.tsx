import { useEffect, useId, useRef, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { BacktestFixtureScenario, BacktestRun, BacktestRunRequest, StrategyVersion } from '../shared/ipc-types.ts';
import { browserIntegration, CommandError, explainError, request } from './client.ts';

const activeStates = ['QUEUED', 'RUNNING'];
const decimalInput = /^(?:\d+)(?:\.\d{0,18})?$/;

const localDateTime = (value: Date) => {
  const pad = (part: number) => String(part).padStart(2, '0');
  return `${value.getFullYear()}-${pad(value.getMonth() + 1)}-${pad(value.getDate())}T${pad(value.getHours())}:${pad(value.getMinutes())}`;
};

const requestTimestamp = (value: string) => value ? new Date(value).toISOString() : '';

function FrozenConfiguration({ run, titleId }: { run: BacktestRun; titleId: string }) {
  return <section aria-labelledby={titleId}>
    <h3 id={titleId}>Frozen configuration</h3>
    <dl>
      <div><dt>Strategy version</dt><dd>{run.strategyVersionId}</dd></div>
      <div><dt>Strategy hash</dt><dd>{run.strategyHash}</dd></div>
      <div><dt>Instrument</dt><dd>{run.instrumentId}</dd></div>
      <div><dt>Dataset</dt><dd>{run.datasetId}</dd></div>
      <div><dt>Date range</dt><dd>{run.startAt} → {run.endAt}</dd></div>
      <div><dt>Bar interval</dt><dd>{run.barInterval}</dd></div>
      <div><dt>Starting cash</dt><dd>{run.startingCash}</dd></div>
      <div><dt>Commission</dt><dd>{run.commission}</dd></div>
      <div><dt>Slippage</dt><dd>{run.slippage}</dd></div>
      <div><dt>Portfolio seed</dt><dd>{run.portfolioSeed ?? 'None'}</dd></div>
      <div><dt>Parameters</dt><dd>{run.parameters?.length ? run.parameters.map(parameter => `${parameter.name}=${parameter.value}`).join(', ') : 'Saved defaults'}</dd></div>
    </dl>
  </section>;
}

type Props = {
  workspaceId: string;
  strategy?: StrategyVersion;
  strategyOptions?: StrategyVersion[];
  onStrategyChange?: (version: StrategyVersion) => void;
  instrumentId?: string;
  datasetId?: string;
  onInstrumentChange?: (value: string) => void;
  onDatasetChange?: (value: string) => void;
  initialInstrumentId?: string;
};

export function BacktestRunPanel({
  workspaceId,
  strategy,
  strategyOptions,
  onStrategyChange,
  instrumentId: controlledInstrumentId,
  datasetId: controlledDatasetId,
  onInstrumentChange,
  onDatasetChange,
  initialInstrumentId,
}: Props) {
  const queryClient = useQueryClient();
  const panelId = useId().replaceAll(':', '');
  const strategyLibrary = useQuery({
    queryKey: ['strategies', workspaceId, 'backtest-panel'],
    queryFn: () => request('strategy.list', { workspaceId }),
    enabled: strategyOptions == null,
  });
  const options = strategyOptions ?? strategyLibrary.data?.versions ?? [];
  const [selectedStrategyId, setSelectedStrategyId] = useState(strategy?.strategyVersionId ?? '');
  const [localInstrumentId, setLocalInstrumentId] = useState(initialInstrumentId ?? 'equity:US:AAPL');
  const [localDatasetId, setLocalDatasetId] = useState('historical:fixture');
  const [barInterval, setBarInterval] = useState('1d');
  const [startAt, setStartAt] = useState(localDateTime(new Date(Date.now() - 86_400_000)));
  const [endAt, setEndAt] = useState(localDateTime(new Date()));
  const [startingCash, setStartingCash] = useState('100000');
  const [commission, setCommission] = useState('0');
  const [slippage, setSlippage] = useState('0');
  const [portfolioSeed, setPortfolioSeed] = useState('');
  const [fixtureScenario, setFixtureScenario] = useState<BacktestFixtureScenario>('FAILURE');
  const [run, setRun] = useState<BacktestRun>();
  const [lastRequest, setLastRequest] = useState<BacktestRunRequest>();
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const actionRef = useRef<HTMLButtonElement | null>(null);
  const retryRef = useRef<HTMLButtonElement | null>(null);

  const selected = strategy ?? options.find(item => item.strategyVersionId === selectedStrategyId);
  const instrumentId = controlledInstrumentId ?? localInstrumentId;
  const datasetId = controlledDatasetId ?? localDatasetId;
  const displayedRun = run;
  const runQuery = useQuery({
    queryKey: ['backtest-run', workspaceId, run?.runId],
    queryFn: () => request('backtest.get', { workspaceId, runId: run!.runId }),
    enabled: Boolean(run && activeStates.includes(run.state)),
    refetchInterval: 250,
  });
  const queriedRun = runQuery.data ?? displayedRun;

  useEffect(() => {
    if (strategy) setSelectedStrategyId(strategy.strategyVersionId);
  }, [strategy?.strategyVersionId]);
  useEffect(() => {
    if (initialInstrumentId && controlledInstrumentId == null) setLocalInstrumentId(initialInstrumentId);
  }, [initialInstrumentId, controlledInstrumentId]);
  useEffect(() => {
    if (runQuery.data) setRun(runQuery.data);
  }, [runQuery.data]);
  useEffect(() => {
    if (!busy && actionRef.current && queriedRun && !activeStates.includes(queriedRun.state)) {
      const target = actionRef.current.disabled ? retryRef.current : actionRef.current;
      target?.focus();
      actionRef.current = null;
    }
  }, [busy, queriedRun?.state]);

  const setInstrument = (value: string) => {
    setLocalInstrumentId(value);
    onInstrumentChange?.(value);
  };
  const setDataset = (value: string) => {
    setLocalDatasetId(value);
    onDatasetChange?.(value);
  };
  const selectStrategy = (versionId: string) => {
    setSelectedStrategyId(versionId);
    const version = options.find(item => item.strategyVersionId === versionId);
    if (version) onStrategyChange?.(version);
  };
  const validate = () => {
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
    setFieldErrors(next);
    if (Object.keys(next).length) {
      setError('Fix the highlighted backtest fields before running.');
      return false;
    }
    return true;
  };
  const buildRequest = (): BacktestRunRequest | undefined => selected ? ({
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
    ...(browserIntegration ? { fixtureScenario } : {}),
  }) : undefined;
  const execute = async (input?: BacktestRunRequest) => {
    if (!input && !validate()) return;
    const requestInput = input ?? buildRequest();
    if (!requestInput) {
      setFieldErrors({ strategy: 'Select a saved strategy version.' });
      setError('Select a saved version before running a backtest.');
      return;
    }
    setBusy(true); setError(undefined); setLastRequest(requestInput);
    try {
      setRun(await request('backtest.run', requestInput));
      await queryClient.invalidateQueries({ queryKey: ['strategies', workspaceId] });
    } catch (cause) {
      if (cause instanceof CommandError && cause.detail.field) {
        const field = ({ instrumentId: 'instrument', datasetId: 'dataset', strategyVersionId: 'strategy' } as Record<string, string>)[cause.detail.field] ?? cause.detail.field;
        setFieldErrors(current => ({ ...current, [field]: cause.detail.message }));
      }
      setError(explainError(cause));
    } finally { setBusy(false); }
  };
  const cancel = async () => {
    if (!queriedRun || !activeStates.includes(queriedRun.state)) return;
    setBusy(true); setError(undefined);
    try {
      setRun(await request('backtest.cancel', { workspaceId, runId: queriedRun.runId, expectedStateVersion: queriedRun.stateVersion }));
      await runQuery.refetch();
    } catch (cause) {
      setError(explainError(cause));
      await runQuery.refetch();
    } finally { setBusy(false); }
  };
  const errorId = (field: string) => `backtest-${panelId}-${field}-error`;
  const fieldMessage = (field: string) => fieldErrors[field] && <span id={errorId(field)} className="error-text">{fieldErrors[field]}</span>;
  if (strategyOptions == null && strategyLibrary.isPending) return <section className="card strategy-run" aria-labelledby={`backtest-title-${panelId}`}><p role="status">Loading strategies…</p></section>;
  if (strategyOptions == null && strategyLibrary.isError) return <section className="card strategy-run" aria-labelledby={`backtest-title-${panelId}`}><p className="error-banner" role="alert">Strategies are unavailable.</p><button type="button" onClick={() => void strategyLibrary.refetch()}>Reload strategies</button></section>;
  return <section className="card strategy-run" aria-labelledby={`backtest-title-${panelId}`}>
    <h2 id={`backtest-title-${panelId}`}>Backtest</h2>
    <p className="form-hint">Historical simulation only. Backtests never place trades or call a provider.</p>
    {(fieldErrors.strategy || fieldErrors.expectedStrategyHash) && <p className="error-text" role="alert">{fieldErrors.strategy ?? fieldErrors.expectedStrategyHash}</p>}
    {!strategy && <label className="field">Saved strategy version<select value={selected?.strategyVersionId ?? ''} aria-invalid={Boolean(fieldErrors.strategy)} aria-describedby={fieldErrors.strategy ? errorId('strategy') : undefined} onChange={event => selectStrategy(event.target.value)}><option value="">Select a saved version</option>{options.map(version => <option key={version.strategyVersionId} value={version.strategyVersionId}>{version.definition.name} · v{version.revision}</option>)}</select>{fieldMessage('strategy')}</label>}
    <div className="form-grid">
      <label className="field">Instrument<input value={instrumentId} aria-invalid={Boolean(fieldErrors.instrument)} aria-describedby={fieldErrors.instrument ? errorId('instrument') : undefined} onChange={event => setInstrument(event.target.value)} />{fieldMessage('instrument')}</label>
      <label className="field">Dataset<input value={datasetId} aria-invalid={Boolean(fieldErrors.dataset)} aria-describedby={fieldErrors.dataset ? errorId('dataset') : undefined} onChange={event => setDataset(event.target.value)} />{fieldMessage('dataset')}</label>
      <label className="field">Start date<input type="datetime-local" value={startAt} aria-invalid={Boolean(fieldErrors.startAt)} aria-describedby={fieldErrors.startAt ? errorId('startAt') : undefined} onChange={event => setStartAt(event.target.value)} />{fieldMessage('startAt')}</label>
      <label className="field">End date<input type="datetime-local" value={endAt} aria-invalid={Boolean(fieldErrors.endAt)} aria-describedby={fieldErrors.endAt ? errorId('endAt') : undefined} onChange={event => setEndAt(event.target.value)} />{fieldMessage('endAt')}</label>
      <label className="field">Bar interval<select value={barInterval} aria-invalid={Boolean(fieldErrors.barInterval)} aria-describedby={fieldErrors.barInterval ? errorId('barInterval') : undefined} onChange={event => setBarInterval(event.target.value)}><option value="1m">1 minute</option><option value="5m">5 minutes</option><option value="15m">15 minutes</option><option value="30m">30 minutes</option><option value="1h">1 hour</option><option value="1d">1 day</option></select>{fieldMessage('barInterval')}</label>
      <label className="field">Starting cash<input inputMode="decimal" value={startingCash} aria-invalid={Boolean(fieldErrors.startingCash)} aria-describedby={fieldErrors.startingCash ? errorId('startingCash') : undefined} onChange={event => setStartingCash(event.target.value)} />{fieldMessage('startingCash')}</label>
      <label className="field">Commission<input inputMode="decimal" value={commission} aria-invalid={Boolean(fieldErrors.commission)} aria-describedby={fieldErrors.commission ? errorId('commission') : undefined} onChange={event => setCommission(event.target.value)} />{fieldMessage('commission')}</label>
      <label className="field">Slippage<input inputMode="decimal" value={slippage} aria-invalid={Boolean(fieldErrors.slippage)} aria-describedby={fieldErrors.slippage ? errorId('slippage') : undefined} onChange={event => setSlippage(event.target.value)} />{fieldMessage('slippage')}</label>
      <label className="field">Portfolio seed<input value={portfolioSeed} aria-invalid={Boolean(fieldErrors.portfolioSeed)} aria-describedby={fieldErrors.portfolioSeed ? errorId('portfolioSeed') : undefined} onChange={event => setPortfolioSeed(event.target.value)} placeholder="Optional" />{fieldMessage('portfolioSeed')}</label>
      {browserIntegration && <label className="field">Backtest integration scenario<select aria-label="Backtest integration scenario" value={fixtureScenario} onChange={event => setFixtureScenario(event.target.value as BacktestFixtureScenario)}><option value="FAILURE">Failure</option><option value="CANCELLED">Cancelled</option></select></label>}
    </div>
    <div className="form-actions">
      <button type="button" className="primary" disabled={busy || !selected} onFocus={event => { actionRef.current = event.currentTarget; }} onClick={event => { actionRef.current = event.currentTarget; void execute(); }}>Run backtest</button>
      <button type="button" disabled={busy || !queriedRun || !activeStates.includes(queriedRun.state)} onFocus={event => { actionRef.current = event.currentTarget; }} onClick={event => { actionRef.current = event.currentTarget; void cancel(); }}>Cancel backtest</button>
      <button ref={retryRef} type="button" disabled={busy || !lastRequest || !queriedRun || activeStates.includes(queriedRun.state)} onFocus={event => { actionRef.current = event.currentTarget; }} onClick={event => { actionRef.current = event.currentTarget; void execute(lastRequest); }}>Retry backtest</button>
    </div>
    {error && <p className="error-text" role="alert">{error}</p>}
    {queriedRun && <div className="strategy-result" aria-live="polite"><strong>{queriedRun.state}</strong><span>Run {queriedRun.runId}</span><span>Config {queriedRun.requestHash}</span><FrozenConfiguration run={queriedRun} titleId={`backtest-frozen-title-${panelId}`} />{queriedRun.failure && <><p role="alert">{queriedRun.failure.code}: {queriedRun.failure.reason}</p>{queriedRun.failure.remediation?.length ? <ul><li>{queriedRun.failure.remediation.join(' · ')}</li></ul> : null}</>}{queriedRun.fixtureLabel && <p className="form-hint">Integration fixture: {queriedRun.fixtureLabel}</p>}</div>}
    {!selected && <p className="form-hint">Select a saved version before running a backtest.</p>}
  </section>;
}
