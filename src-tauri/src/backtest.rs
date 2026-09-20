use crate::market;
use crate::protocol::{
    BacktestFailure, BacktestFixtureScenario, BacktestRunRequest, StrategyParameter,
    StrategyVersion, TradeXError,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub const ENGINE_VERSION: &str = "tradex-backtest-engine-v1";

#[derive(Debug)]
pub enum RunOutcome {
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
    observed_at: &'a str,
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
        return Err(TradeXError::new("BACKTEST_INSTRUMENT_NOT_FOUND"));
    }
    if request.dataset_id != "historical:fixture" {
        return Err(TradeXError::new("BACKTEST_DATASET_NOT_FOUND"));
    }
    if !matches!(
        request.bar_interval.as_str(),
        "1m" | "5m" | "15m" | "30m" | "1h" | "1d"
    ) {
        return Err(TradeXError::new("BACKTEST_BAR_INTERVAL_INVALID"));
    }
    let start = parse_timestamp(&request.start_at)?;
    let end = parse_timestamp(&request.end_at)?;
    if start > end {
        return Err(TradeXError::new("BACKTEST_DATE_RANGE_INVALID"));
    }
    let _starting_cash = decimal(&request.starting_cash, true)?;
    let _commission = decimal(&request.commission, false)?;
    let _slippage = decimal(&request.slippage, false)?;
    if let Some(seed) = &request.portfolio_seed
        && (seed.trim().is_empty() || seed.len() > 128 || seed.chars().any(char::is_control))
    {
        return Err(TradeXError::new("BACKTEST_PORTFOLIO_SEED_INVALID"));
    }
    if request.parameters.len() > 32 {
        return Err(TradeXError::new("BACKTEST_PARAMETER_INVALID"));
    }
    for parameter in &request.parameters {
        validate_parameter(parameter)?;
    }
    if let Some(hash) = request.expected_strategy_hash.as_deref()
        && !valid_hash(hash)
    {
        return Err(TradeXError::new("BACKTEST_STRATEGY_HASH_INVALID"));
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
    observed_at: &str,
) -> Result<String, TradeXError> {
    let starting_cash = decimal(&request.starting_cash, true)?;
    let commission = decimal(&request.commission, false)?;
    let slippage = decimal(&request.slippage, false)?;
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
        observed_at,
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

pub fn execute(request: &BacktestRunRequest, cancel: Option<&AtomicBool>) -> RunOutcome {
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
    }
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

fn decimal(value: &str, positive: bool) -> Result<String, TradeXError> {
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
