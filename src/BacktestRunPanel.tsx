import { useEffect, useId, useRef, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { BacktestComparison, BacktestFixtureScenario, BacktestResult, BacktestRun, BacktestRunRequest, StrategyVersion } from '../shared/ipc-types.ts';
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

function BacktestResultView({ result, titleId }: { result: BacktestResult; titleId: string }) {
  return <section aria-labelledby={titleId}>
    <h3 id={titleId}>Completed result · historical simulation</h3>
    <dl>
      <div><dt>Return</dt><dd>{result.metrics.return}</dd></div>
      <div><dt>Sharpe</dt><dd>{result.metrics.sharpe}</dd></div>
      <div><dt>Sortino</dt><dd>{result.metrics.sortino}</dd></div>
      <div><dt>Max drawdown</dt><dd>{result.metrics.maxDrawdown}</dd></div>
      <div><dt>Win rate</dt><dd>{result.metrics.winRate}</dd></div>
      <div><dt>Profit factor</dt><dd>{result.metrics.profitFactor}</dd></div>
      <div><dt>Turnover</dt><dd>{result.metrics.turnover}</dd></div>
      <div><dt>Trades</dt><dd>{result.metrics.tradeCount}</dd></div>
    </dl>
    <p className="form-hint">Historical simulation only. This result cannot place trades or call a provider.</p>
    <h4>Equity curve</h4>
    <div className="table-scroll"><table><caption className="sr-only">Backtest equity curve</caption><thead><tr><th scope="col">Observed</th><th scope="col">Equity</th><th scope="col">Drawdown</th></tr></thead><tbody>{result.equityCurve.map(point => <tr key={`${point.observedAt}-${point.equity}`}><td>{point.observedAt}</td><td>{point.equity}</td><td>{point.drawdown}</td></tr>)}</tbody></table></div>
    <h4>Trade list</h4>
    <div className="table-scroll"><table><caption className="sr-only">Backtest trades</caption><thead><tr><th scope="col">Time</th><th scope="col">Side</th><th scope="col">Quantity</th><th scope="col">Price</th><th scope="col">P&amp;L</th></tr></thead><tbody>{result.trades.map(trade => <tr key={trade.tradeId}><td>{trade.observedAt}</td><td>{trade.side}</td><td>{trade.quantity}</td><td>{trade.price}</td><td>{trade.realizedPnl}</td></tr>)}</tbody></table></div>
    <details><summary>Reproducibility manifest</summary><dl>{Object.entries(result.manifest).map(([key, value]) => <div key={key}><dt>{key}</dt><dd>{Array.isArray(value) ? value.map(item => `${item.name}:${item.state}`).join(' · ') : String(value)}</dd></div>)}</dl></details>
    <ul>{result.limitations.map(limitation => <li key={limitation}>{limitation}</li>)}</ul>
  </section>;
}

function BacktestComparisonView({ comparison, onBack, titleId }: { comparison: BacktestComparison; onBack: () => void; titleId: string }) {
  const metricRows = Object.entries(comparison.metrics) as Array<[string, { left: string; right: string; difference: string }]>;
  const renderDifferences = (title: string, differences: BacktestComparison['inputDifferences']) => <section aria-labelledby={titleId + '-' + title}>
    <h4 id={titleId + '-' + title}>{title}</h4>
    {differences.length ? <div className="table-scroll"><table><caption className="sr-only">{title}</caption><thead><tr><th scope="col">Field</th><th scope="col">Left</th><th scope="col">Right</th></tr></thead><tbody>{differences.map(item => <tr key={item.field}><th scope="row">{item.field}</th><td>{item.left}</td><td>{item.right}</td></tr>)}</tbody></table></div> : <p className="form-hint">No differences.</p>}
  </section>;
  return <section className="strategy-result" aria-labelledby={titleId} aria-live="polite">
    <div className="form-actions"><button type="button" onClick={onBack}>Back to backtest</button></div>
    <h3 id={titleId}>Backtest comparison</h3>
    <p className="form-hint">Historical simulation only. Comparison is read-only and cannot place trades or call a provider.</p>
    <div className="table-scroll"><table><caption className="sr-only">Backtest metric comparison</caption><thead><tr><th scope="col">Metric</th><th scope="col">Left</th><th scope="col">Right</th><th scope="col">Difference</th></tr></thead><tbody>{metricRows.map(([name, value]) => <tr key={name}><th scope="row">{name}</th><td>{value.left}</td><td>{value.right}</td><td>{value.difference}</td></tr>)}</tbody></table></div>
    <section aria-labelledby={titleId + '-identity'}><h4 id={titleId + '-identity'}>Run identity</h4><dl><div><dt>Left run</dt><dd>{comparison.left.runId}</dd></div><div><dt>Right run</dt><dd>{comparison.right.runId}</dd></div><div><dt>Left request</dt><dd>{comparison.left.requestHash}</dd></div><div><dt>Right request</dt><dd>{comparison.right.requestHash}</dd></div><div><dt>Strategy</dt><dd>{comparison.left.strategyVersionId} · {comparison.left.strategyHash} → {comparison.right.strategyVersionId} · {comparison.right.strategyHash}</dd></div><div><dt>Dataset</dt><dd>{comparison.left.instrumentId} · {comparison.left.datasetId} → {comparison.right.instrumentId} · {comparison.right.datasetId}</dd></div><div><dt>Engine</dt><dd>{comparison.left.result?.manifest.engineVersion ?? 'Unavailable'} · {comparison.left.result?.manifest.runtimeVersion ?? 'Unavailable'} → {comparison.right.result?.manifest.engineVersion ?? 'Unavailable'} · {comparison.right.result?.manifest.runtimeVersion ?? 'Unavailable'}</dd></div></dl></section>
    <section aria-labelledby={titleId + '-curve'}><h4 id={titleId + '-curve'}>Equity curve summary</h4><dl><div><dt>Left points</dt><dd>{comparison.leftCurve.pointCount} · {comparison.leftCurve.startEquity} → {comparison.leftCurve.endEquity}</dd></div><div><dt>Right points</dt><dd>{comparison.rightCurve.pointCount} · {comparison.rightCurve.startEquity} → {comparison.rightCurve.endEquity}</dd></div><div><dt>Max drawdown</dt><dd>{comparison.leftCurve.maxDrawdown} → {comparison.rightCurve.maxDrawdown}</dd></div></dl></section>
    {renderDifferences('Input differences', comparison.inputDifferences)}
    {renderDifferences('Manifest differences', comparison.manifestDifferences)}
    <ul>{comparison.limitations.map(limitation => <li key={limitation}>{limitation}</li>)}</ul>
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
  const [leftRunId, setLeftRunId] = useState('');
  const [rightRunId, setRightRunId] = useState('');
  const [compareRequest, setCompareRequest] = useState<{ workspaceId: string; leftRunId: string; rightRunId: string }>();
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
  const backtestLibrary = useQuery({
    queryKey: ['backtest-library', workspaceId],
    queryFn: () => request('backtest.list', { workspaceId }),
    refetchOnMount: 'always',
  });
  const completedRuns = backtestLibrary.data?.runs.filter(item => item.state === 'COMPLETED') ?? [];
  const comparisonQuery = useQuery({
    queryKey: ['backtest-compare', compareRequest],
    queryFn: () => request('backtest.compare', compareRequest!),
    enabled: Boolean(compareRequest),
  });

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
    if (completedRuns.length >= 2) {
      setLeftRunId(current => completedRuns.some(item => item.runId === current) ? current : completedRuns[0].runId);
      setRightRunId(current => completedRuns.some(item => item.runId === current && item.runId !== leftRunId) ? current : completedRuns[1].runId);
    } else {
      setLeftRunId('');
      setRightRunId('');
    }
  }, [backtestLibrary.data?.stateVersion]);
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
      await queryClient.invalidateQueries({ queryKey: ['backtest-library', workspaceId] });
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
      <label className="field">Start date<input type="datetime-local" value={startAt} aria-invalid={Boolean(fieldErrors.startAt)} aria-describedby={fieldErrors.startAt ? errorId('startAt') : undefined} onInput={event => setStartAt(event.currentTarget.value)} onChange={event => setStartAt(event.target.value)} />{fieldMessage('startAt')}</label>
      <label className="field">End date<input type="datetime-local" value={endAt} aria-invalid={Boolean(fieldErrors.endAt)} aria-describedby={fieldErrors.endAt ? errorId('endAt') : undefined} onInput={event => setEndAt(event.currentTarget.value)} onChange={event => setEndAt(event.target.value)} />{fieldMessage('endAt')}</label>
      <label className="field">Bar interval<select value={barInterval} aria-invalid={Boolean(fieldErrors.barInterval)} aria-describedby={fieldErrors.barInterval ? errorId('barInterval') : undefined} onChange={event => setBarInterval(event.target.value)}><option value="1m">1 minute</option><option value="5m">5 minutes</option><option value="15m">15 minutes</option><option value="30m">30 minutes</option><option value="1h">1 hour</option><option value="1d">1 day</option></select>{fieldMessage('barInterval')}</label>
      <label className="field">Starting cash<input inputMode="decimal" value={startingCash} aria-invalid={Boolean(fieldErrors.startingCash)} aria-describedby={fieldErrors.startingCash ? errorId('startingCash') : undefined} onChange={event => setStartingCash(event.target.value)} />{fieldMessage('startingCash')}</label>
      <label className="field">Commission<input inputMode="decimal" value={commission} aria-invalid={Boolean(fieldErrors.commission)} aria-describedby={fieldErrors.commission ? errorId('commission') : undefined} onChange={event => setCommission(event.target.value)} />{fieldMessage('commission')}</label>
      <label className="field">Slippage<input inputMode="decimal" value={slippage} aria-invalid={Boolean(fieldErrors.slippage)} aria-describedby={fieldErrors.slippage ? errorId('slippage') : undefined} onChange={event => setSlippage(event.target.value)} />{fieldMessage('slippage')}</label>
      <label className="field">Portfolio seed<input value={portfolioSeed} aria-invalid={Boolean(fieldErrors.portfolioSeed)} aria-describedby={fieldErrors.portfolioSeed ? errorId('portfolioSeed') : undefined} onChange={event => setPortfolioSeed(event.target.value)} placeholder="Optional" />{fieldMessage('portfolioSeed')}</label>
      {browserIntegration && <label className="field">Backtest integration scenario<select aria-label="Backtest integration scenario" value={fixtureScenario} onChange={event => setFixtureScenario(event.target.value as BacktestFixtureScenario)}><option value="FAILURE">Failure</option><option value="SUCCESS">Success</option><option value="CANCELLED">Cancelled</option><option value="LOOKAHEAD">Look-ahead guard</option><option value="SURVIVORSHIP">Survivorship guard</option><option value="SPLIT">Split guard</option><option value="DIVIDEND">Dividend guard</option><option value="TIMEZONE">Timezone guard</option><option value="DATA_GAP">Data gap guard</option><option value="DATASET_HASH_MISMATCH">Dataset hash guard</option></select></label>}
    </div>
    <div className="form-actions">
      <button type="button" className="primary" disabled={busy || !selected} onFocus={event => { actionRef.current = event.currentTarget; }} onClick={event => { actionRef.current = event.currentTarget; void execute(); }}>Run backtest</button>
      <button type="button" disabled={busy || !queriedRun || !activeStates.includes(queriedRun.state)} onFocus={event => { actionRef.current = event.currentTarget; }} onClick={event => { actionRef.current = event.currentTarget; void cancel(); }}>Cancel backtest</button>
      <button ref={retryRef} type="button" disabled={busy || !lastRequest || !queriedRun || activeStates.includes(queriedRun.state)} onFocus={event => { actionRef.current = event.currentTarget; }} onClick={event => { actionRef.current = event.currentTarget; void execute(lastRequest); }}>Retry backtest</button>
    </div>
    {error && <p className="error-text" role="alert">{error}</p>}
    <section className="strategy-compare" aria-labelledby={'backtest-compare-title-' + panelId}>
      <h3 id={'backtest-compare-title-' + panelId}>Compare completed runs</h3>
      <p className="form-hint">Select two saved completed runs from this workspace. Results stay historical and read-only.</p>
      {backtestLibrary.isPending && <p role="status">Loading saved backtests…</p>}
      {backtestLibrary.isError && <><p className="error-banner" role="alert">Saved backtests are unavailable.</p><button type="button" onClick={() => void backtestLibrary.refetch()}>Reload saved backtests</button></>}
      {!backtestLibrary.isPending && !backtestLibrary.isError && completedRuns.length < 2 && <p className="form-hint">Complete a second backtest to compare runs.</p>}
      {completedRuns.length >= 2 && <div className="form-grid"><label className="field">Left run<select aria-label="Compare left run" value={leftRunId} onChange={event => setLeftRunId(event.target.value)}>{completedRuns.map(item => <option key={item.runId} value={item.runId}>{item.runId} · {item.updatedAt}</option>)}</select></label><label className="field">Right run<select aria-label="Compare right run" value={rightRunId} onChange={event => setRightRunId(event.target.value)}>{completedRuns.map(item => <option key={item.runId} value={item.runId}>{item.runId} · {item.updatedAt}</option>)}</select></label><div className="form-actions"><button type="button" className="primary" disabled={!leftRunId || !rightRunId || leftRunId === rightRunId || comparisonQuery.isFetching} onClick={() => { setError(undefined); setCompareRequest({ workspaceId, leftRunId, rightRunId }); }}>{comparisonQuery.isFetching ? 'Comparing…' : 'Compare runs'}</button></div></div>}
      {comparisonQuery.isError && <p className="error-text" role="alert">{explainError(comparisonQuery.error)} <button type="button" onClick={() => void comparisonQuery.refetch()}>Retry compare</button></p>}
      {comparisonQuery.data && <BacktestComparisonView comparison={comparisonQuery.data} onBack={() => { setCompareRequest(undefined); }} titleId={'backtest-comparison-title-' + panelId} />}
    </section>
    {queriedRun && <div className="strategy-result" aria-live="polite"><strong>{queriedRun.state}</strong><span>Run {queriedRun.runId}</span><span>Config {queriedRun.requestHash}</span><FrozenConfiguration run={queriedRun} titleId={`backtest-frozen-title-${panelId}`} />{queriedRun.result && <BacktestResultView result={queriedRun.result} titleId={`backtest-result-title-${panelId}`} />}{queriedRun.failure && <><p role="alert">{queriedRun.failure.code}: {queriedRun.failure.reason}</p>{queriedRun.failure.remediation?.length ? <ul><li>{queriedRun.failure.remediation.join(' · ')}</li></ul> : null}</>}{queriedRun.fixtureLabel && <p className="form-hint">Integration fixture: {queriedRun.fixtureLabel}</p>}</div>}
    {!selected && <p className="form-hint">Select a saved version before running a backtest.</p>}
  </section>;
}
