use crate::market;
use crate::protocol::{
    BacktestFailure, BacktestFixtureScenario, BacktestGuardCheck, BacktestGuardState,
    BacktestManifest, BacktestMetrics, BacktestResult, BacktestRun, BacktestRunRequest,
    EquityPoint, StrategyParameter, StrategyVersion, TradeRecord, TradeXError,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub const ENGINE_VERSION: &str = "tradex-backtest-engine-v1";
const FIXTURE_HISTORY_START: &str = "2000-01-01T00:00:00Z";
const FIXTURE_HISTORY_END: &str = "2100-01-01T00:00:00Z";

#[derive(Debug)]
pub enum RunOutcome {
    Completed(Box<BacktestResult>),
    Failed(BacktestFailure),
    Cancelled(BacktestFailure),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalRun<'a> {
    workspace_id: &'a str,
    strategy_version_id: &'a str,
    strategy_hash: &'a str,
    instrument_id: &'a str,
    dataset_id: &'a str,
    start_at: &'a str,
    end_at: &'a str,
    bar_interval: &'a str,
    starting_cash: &'a str,
    commission: &'a str,
    slippage: &'a str,
    portfolio_seed: Option<&'a str>,
    parameters: &'a [StrategyParameter],
    dataset_hash: String,
    adjustment_method: &'static str,
    timezone: &'static str,
    market_calendar_version: &'static str,
    runtime_version: &'static str,
    engine_version: &'static str,
}

pub fn validate_run_request(request: &BacktestRunRequest) -> Result<(), TradeXError> {
    if request.workspace_id.trim().is_empty()
        || request.workspace_id.len() > 128
        || request.workspace_id.chars().any(char::is_control)
        || request.strategy_version_id.trim().is_empty()
        || request.strategy_version_id.len() > 128
        || request.instrument_id.len() > 128
        || request.instrument_id.chars().any(char::is_control)
        || request.start_at.len() > 64
        || request.end_at.len() > 64
        || request.start_at.chars().any(char::is_control)
        || request.end_at.chars().any(char::is_control)
        || request.bar_interval.len() > 32
        || request.bar_interval.chars().any(char::is_control)
    {
        return Err(TradeXError::new("BACKTEST_CONFIG_INVALID"));
    }
    if !market::validate_instrument_id(&request.instrument_id)
        || !market::instruments()
            .iter()
            .any(|instrument| instrument.instrument_id == request.instrument_id)
    {
        return Err(TradeXError::new("BACKTEST_INSTRUMENT_NOT_FOUND").with_field("instrumentId"));
    }
    if request.dataset_id != "historical:fixture" {
        return Err(TradeXError::new("BACKTEST_DATASET_NOT_FOUND").with_field("datasetId"));
    }
    if !matches!(
        request.bar_interval.as_str(),
        "1m" | "5m" | "15m" | "30m" | "1h" | "1d"
    ) {
        return Err(TradeXError::new("BACKTEST_BAR_INTERVAL_INVALID").with_field("barInterval"));
    }
    let start = parse_timestamp(&request.start_at).map_err(|error| error.with_field("startAt"))?;
    let end = parse_timestamp(&request.end_at).map_err(|error| error.with_field("endAt"))?;
    if start > end {
        return Err(TradeXError::new("BACKTEST_DATE_RANGE_INVALID").with_field("endAt"));
    }
    let _starting_cash = normalize_decimal(&request.starting_cash, true)
        .map_err(|error| error.with_field("startingCash"))?;
    let _commission = normalize_decimal(&request.commission, false)
        .map_err(|error| error.with_field("commission"))?;
    let _slippage = normalize_decimal(&request.slippage, false)
        .map_err(|error| error.with_field("slippage"))?;
    if let Some(seed) = &request.portfolio_seed
        && (seed.trim().is_empty() || seed.len() > 128 || seed.chars().any(char::is_control))
    {
        return Err(TradeXError::new("BACKTEST_PORTFOLIO_SEED_INVALID").with_field("portfolioSeed"));
    }
    if request.parameters.len() > 32 {
        return Err(TradeXError::new("BACKTEST_PARAMETER_INVALID"));
    }
    for parameter in &request.parameters {
        validate_parameter(parameter).map_err(|error| error.with_field("parameters"))?;
    }
    if let Some(hash) = request.expected_strategy_hash.as_deref()
        && !valid_hash(hash)
    {
        return Err(
            TradeXError::new("BACKTEST_STRATEGY_HASH_INVALID").with_field("expectedStrategyHash")
        );
    }
    if request.fixture_scenario.is_some() && !fixture_enabled() {
        return Err(TradeXError::new("BACKTEST_FIXTURE_UNAVAILABLE"));
    }
    Ok(())
}

pub fn run_identity_hash(
    version: &StrategyVersion,
    request: &BacktestRunRequest,
    parameters: &[StrategyParameter],
) -> Result<String, TradeXError> {
    let starting_cash = normalize_decimal(&request.starting_cash, true)?;
    let commission = normalize_decimal(&request.commission, false)?;
    let slippage = normalize_decimal(&request.slippage, false)?;
    let input = CanonicalRun {
        workspace_id: &request.workspace_id,
        strategy_version_id: &version.strategy_version_id,
        strategy_hash: &version.source_hash,
        instrument_id: &request.instrument_id,
        dataset_id: &request.dataset_id,
        start_at: &request.start_at,
        end_at: &request.end_at,
        bar_interval: &request.bar_interval,
        starting_cash: &starting_cash,
        commission: &commission,
        slippage: &slippage,
        portfolio_seed: request.portfolio_seed.as_deref(),
        parameters,
        dataset_hash: fixture_dataset_hash(),
        adjustment_method: "split-dividend-adjusted",
        timezone: "UTC",
        market_calendar_version: "tradex-calendar-v1",
        runtime_version: "fixture-runtime-v1",
        engine_version: ENGINE_VERSION,
    };
    let bytes =
        serde_json::to_vec(&input).map_err(|_| TradeXError::new("BACKTEST_IDENTITY_FAILED"))?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

pub fn fixture_enabled() -> bool {
    cfg!(feature = "integration-test")
        && std::env::var("TRADEX_BACKTEST_FIXTURE").ok().as_deref() == Some("1")
}

pub fn validate_history_coverage(
    request: &BacktestRunRequest,
    fixture: bool,
    historical_source_available: bool,
) -> Result<(), TradeXError> {
    let start = parse_timestamp(&request.start_at).map_err(|error| error.with_field("startAt"))?;
    let end = parse_timestamp(&request.end_at).map_err(|error| error.with_field("endAt"))?;
    let fixture_start =
        parse_timestamp(FIXTURE_HISTORY_START).map_err(|error| error.with_field("startAt"))?;
    let fixture_end =
        parse_timestamp(FIXTURE_HISTORY_END).map_err(|error| error.with_field("endAt"))?;
    if start < fixture_start {
        return Err(TradeXError::new("MARKET_HISTORY_UNAVAILABLE").with_field("startAt"));
    }
    if end > fixture_end {
        return Err(TradeXError::new("MARKET_HISTORY_UNAVAILABLE").with_field("endAt"));
    }
    if fixture || historical_source_available {
        return Ok(());
    }
    Err(TradeXError::new("MARKET_HISTORY_UNAVAILABLE").with_field("datasetId"))
}

pub fn execute(
    request: &BacktestRunRequest,
    run: &BacktestRun,
    cancel: Option<&AtomicBool>,
) -> RunOutcome {
    if cancel.is_some_and(|flag| flag.load(Ordering::Acquire)) {
        return RunOutcome::Cancelled(cancelled_failure());
    }
    if !fixture_enabled() {
        return RunOutcome::Failed(BacktestFailure {
            code: "BACKTEST_RUNTIME_UNAVAILABLE".into(),
            reason: "No production backtest runtime is configured for this worker.".into(),
            remediation: vec!["configure_backtest_runtime".into()],
        });
    }
    match request
        .fixture_scenario
        .clone()
        .unwrap_or(BacktestFixtureScenario::Failure)
    {
        BacktestFixtureScenario::Success => match fixture_result(request, run) {
            Ok(result) => RunOutcome::Completed(Box::new(result)),
            Err(error) => RunOutcome::Failed(BacktestFailure {
                code: error.code,
                reason: error.message,
                remediation: vec!["retry_backtest_run".into()],
            }),
        },
        BacktestFixtureScenario::Failure => RunOutcome::Failed(BacktestFailure {
            code: "BACKTEST_FIXTURE_FAILED".into(),
            reason: "The integration backtest fixture returned a typed failure.".into(),
            remediation: vec!["retry_backtest_run".into()],
        }),
        BacktestFixtureScenario::Cancelled => {
            for _ in 0..500 {
                if cancel.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                    return RunOutcome::Cancelled(cancelled_failure());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            RunOutcome::Cancelled(cancelled_failure())
        }
        scenario => RunOutcome::Failed(fixture_guard_failure(scenario)),
    }
}

fn fixture_guard_failure(scenario: BacktestFixtureScenario) -> BacktestFailure {
    let (code, reason) = match scenario {
        BacktestFixtureScenario::Lookahead => (
            "BACKTEST_LOOKAHEAD_DETECTED",
            "The historical fixture contains a look-ahead reference and was rejected.",
        ),
        BacktestFixtureScenario::Survivorship => (
            "BACKTEST_SURVIVORSHIP_BIAS",
            "The historical fixture does not contain a survivorship-safe universe.",
        ),
        BacktestFixtureScenario::Split => (
            "BACKTEST_SPLIT_UNVERIFIED",
            "The fixture split adjustment could not be verified.",
        ),
        BacktestFixtureScenario::Dividend => (
            "BACKTEST_DIVIDEND_UNVERIFIED",
            "The fixture dividend adjustment could not be verified.",
        ),
        BacktestFixtureScenario::Timezone => (
            "BACKTEST_TIMEZONE_MISMATCH",
            "The fixture timezone does not match the requested calendar.",
        ),
        BacktestFixtureScenario::DataGap => (
            "BACKTEST_DATA_GAP",
            "The fixture contains a data gap inside the requested range.",
        ),
        BacktestFixtureScenario::DatasetHashMismatch => (
            "BACKTEST_DATASET_HASH_MISMATCH",
            "The approved dataset hash does not match the fixture contents.",
        ),
        BacktestFixtureScenario::Success
        | BacktestFixtureScenario::Failure
        | BacktestFixtureScenario::Cancelled => (
            "BACKTEST_FIXTURE_FAILED",
            "The integration backtest fixture returned a typed failure.",
        ),
    };
    BacktestFailure {
        code: code.into(),
        reason: reason.into(),
        remediation: vec!["review_backtest_data_guards".into()],
    }
}

fn fixture_result(
    request: &BacktestRunRequest,
    run: &BacktestRun,
) -> Result<BacktestResult, TradeXError> {
    let starting_cash = normalize_decimal(&request.starting_cash, true)?;
    let gain = crate::portfolio::decimal_mul(&starting_cash, "0.015")
        .map_err(|_| TradeXError::new("BACKTEST_RESULT_INVALID"))?;
    let end_equity = crate::portfolio::decimal_add(&starting_cash, &gain)
        .map_err(|_| TradeXError::new("BACKTEST_RESULT_INVALID"))?;
    let dataset_hash = fixture_dataset_hash();
    let mut manifest = BacktestManifest {
        strategy_version: run.strategy_version_id.clone(),
        strategy_hash: run.strategy_hash.clone(),
        dataset_id: request.dataset_id.clone(),
        dataset_hash,
        data_provider: "TradeX deterministic fixture".into(),
        retrieved_at: "2026-01-01T00:00:00Z".into(),
        start_at: request.start_at.clone(),
        end_at: request.end_at.clone(),
        adjustment_method: "split-dividend-adjusted".into(),
        timezone: "UTC".into(),
        market_calendar_version: "tradex-calendar-v1".into(),
        commission_model: "decimal-rate".into(),
        commission: request.commission.clone(),
        slippage_model: "decimal-rate".into(),
        slippage: request.slippage.clone(),
        starting_cash: starting_cash.clone(),
        seed: request
            .portfolio_seed
            .clone()
            .unwrap_or_else(|| "default".into()),
        parameters: request.parameters.clone(),
        engine_version: ENGINE_VERSION.into(),
        runtime_version: "fixture-runtime-v1".into(),
        guard_checks: fixture_guard_checks(),
        manifest_hash: String::new(),
    };
    manifest.manifest_hash = manifest_hash(&manifest)?;
    let metrics = BacktestMetrics {
        return_pct: "0.015".into(),
        sharpe: "1.5".into(),
        sortino: "2".into(),
        max_drawdown: "0.005".into(),
        win_rate: "1".into(),
        profit_factor: "2".into(),
        turnover: "0.01".into(),
        trade_count: 1,
    };
    let equity_curve = vec![
        EquityPoint {
            observed_at: request.start_at.clone(),
            equity: starting_cash,
            drawdown: "0".into(),
        },
        EquityPoint {
            observed_at: request.end_at.clone(),
            equity: end_equity,
            drawdown: "0.005".into(),
        },
    ];
    let trades = vec![TradeRecord {
        trade_id: "fixture-trade-1".into(),
        instrument_id: request.instrument_id.clone(),
        side: "BUY".into(),
        quantity: "1".into(),
        price: "100".into(),
        gross_value: "100".into(),
        commission: request.commission.clone(),
        slippage: request.slippage.clone(),
        realized_pnl: gain,
        observed_at: request.end_at.clone(),
    }];
    let mut result = BacktestResult {
        metrics,
        equity_curve,
        trades,
        manifest,
        historical_simulation: true,
        limitations: vec![
            "HISTORICAL_SIMULATION_ONLY".into(),
            "SYNTHETIC_FIXTURE_NOT_PROVIDER_DATA".into(),
        ],
        result_hash: String::new(),
    };
    result.result_hash = result_hash(&result)?;
    validate_result(&result, run)?;
    Ok(result)
}

fn fixture_guard_checks() -> Vec<BacktestGuardCheck> {
    [
        (
            "lookAhead",
            "No future bars are read before their observation time.",
        ),
        (
            "survivorship",
            "The fixture universe is fixed at dataset creation.",
        ),
        (
            "split",
            "Split adjustment metadata is present and verified.",
        ),
        (
            "dividend",
            "Dividend adjustment metadata is present and verified.",
        ),
        ("timezone", "All observations are normalized to UTC."),
        ("dataGaps", "The fixture range has complete daily coverage."),
    ]
    .into_iter()
    .map(|(name, detail)| BacktestGuardCheck {
        name: name.into(),
        state: BacktestGuardState::Passed,
        detail: detail.into(),
    })
    .collect()
}

pub fn validate_result(result: &BacktestResult, run: &BacktestRun) -> Result<(), TradeXError> {
    let invalid = || TradeXError::new("BACKTEST_RESULT_INVALID");
    if !result.historical_simulation
        || result.equity_curve.is_empty()
        || result.equity_curve.len() > 5_000
        || result.trades.len() > 5_000
        || result.limitations.is_empty()
        || result.limitations.len() > 8
        || result.metrics.trade_count as usize != result.trades.len()
        || result.manifest.strategy_version != run.strategy_version_id
        || result.manifest.strategy_hash != run.strategy_hash
        || result.manifest.dataset_id != run.dataset_id
        || result.manifest.start_at != run.start_at
        || result.manifest.end_at != run.end_at
        || result.manifest.starting_cash != run.starting_cash
        || result.manifest.commission != run.commission
        || result.manifest.slippage != run.slippage
        || result.manifest.engine_version != ENGINE_VERSION
        || result.manifest.seed != run.portfolio_seed.as_deref().unwrap_or("default")
        || result.manifest.parameters != run.parameters
        || result.manifest.guard_checks.len() != 6
    {
        return Err(invalid());
    }
    let expected_guards = [
        "lookAhead",
        "survivorship",
        "split",
        "dividend",
        "timezone",
        "dataGaps",
    ];
    if result.manifest.guard_checks.iter().any(|check| {
        check.state != BacktestGuardState::Passed
            || !expected_guards.contains(&check.name.as_str())
            || check.detail.trim().is_empty()
    }) || expected_guards.iter().any(|name| {
        result
            .manifest
            .guard_checks
            .iter()
            .filter(|check| check.name == *name)
            .count()
            != 1
    }) {
        return Err(invalid());
    }
    for value in [
        &result.metrics.return_pct,
        &result.metrics.sharpe,
        &result.metrics.sortino,
        &result.metrics.max_drawdown,
        &result.metrics.win_rate,
        &result.metrics.profit_factor,
        &result.metrics.turnover,
    ] {
        normalize_decimal(value, false).map_err(|_| invalid())?;
    }
    if result.manifest.dataset_hash != fixture_dataset_hash()
        || !valid_hash(&result.manifest.dataset_hash)
        || result.manifest.manifest_hash != manifest_hash(&result.manifest)?
        || result.result_hash != result_hash(result)?
        || result
            .equity_curve
            .first()
            .map(|point| point.equity.as_str())
            != Some(run.starting_cash.as_str())
        || result.equity_curve.iter().any(|point| {
            point.observed_at.trim().is_empty()
                || point.observed_at.len() > 64
                || point.observed_at.chars().any(char::is_control)
                || normalize_decimal(&point.equity, true).is_err()
                || normalize_decimal(&point.drawdown, false).is_err()
        })
        || result.trades.iter().any(|trade| {
            trade.trade_id.trim().is_empty()
                || trade.instrument_id != run.instrument_id
                || trade.side != "BUY"
                || normalize_decimal(&trade.quantity, true).is_err()
                || normalize_decimal(&trade.price, true).is_err()
                || normalize_decimal(&trade.gross_value, true).is_err()
                || normalize_decimal(&trade.commission, false).is_err()
                || normalize_decimal(&trade.slippage, false).is_err()
                || normalize_decimal(&trade.realized_pnl, false).is_err()
        })
    {
        return Err(invalid());
    }
    Ok(())
}

fn manifest_hash(manifest: &BacktestManifest) -> Result<String, TradeXError> {
    let mut canonical = manifest.clone();
    canonical.manifest_hash.clear();
    canonical_hash(&canonical)
}

fn result_hash(result: &BacktestResult) -> Result<String, TradeXError> {
    let mut canonical = result.clone();
    canonical.result_hash.clear();
    canonical_hash(&canonical)
}

fn canonical_hash<T: Serialize>(value: &T) -> Result<String, TradeXError> {
    let bytes =
        serde_json::to_vec(value).map_err(|_| TradeXError::new("BACKTEST_RESULT_INVALID"))?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

fn fixture_dataset_hash() -> String {
    canonical_hash(&serde_json::json!({
        "datasetId": "historical:fixture",
        "version": "1",
        "bars": ["2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z"]
    }))
    .expect("fixture dataset descriptor is serializable")
}

pub fn validate_failure(failure: &BacktestFailure) -> Result<(), TradeXError> {
    if failure.code.trim().is_empty()
        || failure.code.len() > 64
        || failure.code.chars().any(char::is_control)
        || failure.reason.trim().is_empty()
        || failure.reason.len() > 512
        || failure.reason.chars().any(char::is_control)
        || failure.remediation.len() > 4
        || failure.remediation.iter().any(|item| {
            item.trim().is_empty() || item.len() > 256 || item.chars().any(char::is_control)
        })
    {
        return Err(TradeXError::new("BACKTEST_FAILURE_INVALID"));
    }
    Ok(())
}

fn cancelled_failure() -> BacktestFailure {
    BacktestFailure {
        code: "BACKTEST_CANCELLED".into(),
        reason: "The backtest run was cancelled by the user.".into(),
        remediation: vec!["retry_backtest_run".into()],
    }
}

pub(crate) fn normalize_decimal(value: &str, positive: bool) -> Result<String, TradeXError> {
    let normalized = crate::provider_io::decimal(&Value::String(value.to_owned()))
        .map_err(|_| TradeXError::new("BACKTEST_DECIMAL_INVALID"))?;
    let fraction_digits = normalized
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    if fraction_digits > 18 || (positive && normalized == "0") || normalized.starts_with('-') {
        return Err(TradeXError::new("BACKTEST_DECIMAL_INVALID"));
    }
    Ok(normalized)
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime, TradeXError> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| TradeXError::new("BACKTEST_DATE_RANGE_INVALID"))
}

fn validate_parameter(parameter: &StrategyParameter) -> Result<(), TradeXError> {
    if parameter.name.trim().is_empty()
        || parameter.name.len() > 64
        || parameter.name.chars().any(char::is_control)
        || parameter.value.len() > 256
        || parameter.value.chars().any(char::is_control)
    {
        return Err(TradeXError::new("BACKTEST_PARAMETER_INVALID"));
    }
    Ok(())
}

fn valid_hash(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
