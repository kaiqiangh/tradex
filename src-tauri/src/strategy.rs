use crate::market;
use crate::protocol::{
    Result, StrategyDefinition, StrategyDirection, StrategyFailure, StrategyFixtureScenario,
    StrategyParameter, StrategyRunRequest, StrategySignal, StrategyVersion, TradeXError,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

const MAX_WORKER_OUTPUT: usize = 16_384;

#[derive(Debug)]
pub enum RunOutcome {
    Completed(StrategySignal),
    Failed(StrategyFailure),
    Cancelled(StrategyFailure),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalDefinition<'a> {
    name: &'a str,
    source: &'a str,
    language: &'a str,
    runtime: &'a str,
    parameters: &'a [StrategyParameter],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalRun<'a> {
    strategy_version_id: &'a str,
    strategy_hash: &'a str,
    instrument_id: &'a str,
    dataset_id: &'a str,
    start_at: &'a str,
    end_at: &'a str,
    parameters: &'a [StrategyParameter],
    engine_version: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkerRequest<'a> {
    protocol_version: u32,
    session_token: &'a str,
    operation: &'static str,
    strategy_hash: &'a str,
    strategy_version_id: &'a str,
    instrument_id: &'a str,
    dataset_id: &'a str,
    observed_at: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerHandshake {
    protocol_version: u32,
    session_token: String,
    ok: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerResult {
    protocol_version: u32,
    session_token: String,
    ok: bool,
    signal: Option<StrategySignal>,
    failure: Option<StrategyFailure>,
}

pub fn canonical_version_hash(definition: &StrategyDefinition) -> Result<String> {
    validate_definition(definition)?;
    let input = CanonicalDefinition {
        name: definition.name.trim(),
        source: &definition.source,
        language: definition.language.trim(),
        runtime: definition.runtime.trim(),
        parameters: &definition.parameters,
    };
    let bytes = serde_json::to_vec(&input).map_err(|_| TradeXError::new("STRATEGY_HASH_FAILED"))?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

pub fn run_identity_hash(
    version: &StrategyVersion,
    request: &StrategyRunRequest,
    parameters: &[StrategyParameter],
) -> Result<String> {
    let input = CanonicalRun {
        strategy_version_id: &version.strategy_version_id,
        strategy_hash: &version.source_hash,
        instrument_id: &request.instrument_id,
        dataset_id: &request.dataset_id,
        start_at: &request.start_at,
        end_at: &request.end_at,
        parameters,
        engine_version: "tradex-strategy-engine-v1",
    };
    let bytes = serde_json::to_vec(&input).map_err(|_| TradeXError::new("STRATEGY_HASH_FAILED"))?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

pub fn validate_definition(definition: &StrategyDefinition) -> Result<()> {
    if definition.name.trim().is_empty()
        || definition.name.chars().count() > 120
        || definition.name.chars().any(char::is_control)
        || definition.source.is_empty()
        || definition.source.chars().count() > 100_000
        || definition
            .source
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        || !valid_token(&definition.language, 32)
        || !valid_token(&definition.runtime, 64)
        || definition.parameters.len() > 32
    {
        return Err(TradeXError::new("STRATEGY_DEFINITION_INVALID"));
    }
    let mut names = std::collections::HashSet::new();
    for parameter in &definition.parameters {
        validate_parameter(parameter)?;
        if !names.insert(parameter.name.trim().to_owned()) {
            return Err(TradeXError::new("STRATEGY_PARAMETER_INVALID"));
        }
    }
    Ok(())
}

pub fn validate_run_request(request: &StrategyRunRequest) -> Result<()> {
    for value in [
        request.instrument_id.as_str(),
        request.dataset_id.as_str(),
        request.start_at.as_str(),
        request.end_at.as_str(),
    ] {
        if value.trim().is_empty() || value.chars().any(char::is_control) || value.len() > 128 {
            return Err(TradeXError::new("STRATEGY_RUN_INVALID"));
        }
    }
    if request.strategy_version_id.trim().is_empty()
        || request.strategy_version_id.len() > 128
        || !market::validate_instrument_id(&request.instrument_id)
        || !market::instruments()
            .iter()
            .any(|instrument| instrument.instrument_id == request.instrument_id)
        || !valid_dataset_id(&request.dataset_id)
        || request.parameters.len() > 32
    {
        return Err(if valid_dataset_id(&request.dataset_id) {
            TradeXError::new("STRATEGY_RUN_INVALID")
        } else {
            TradeXError::new("STRATEGY_DATASET_NOT_FOUND")
        });
    }
    if request.start_at > request.end_at {
        return Err(TradeXError::new("STRATEGY_RUN_INVALID"));
    }
    for parameter in &request.parameters {
        validate_parameter(parameter)?;
    }
    if request
        .expected_strategy_hash
        .as_deref()
        .is_some_and(|hash| hash.len() > 80 || hash.chars().any(char::is_control))
    {
        return Err(TradeXError::new("STRATEGY_RUN_INVALID"));
    }
    Ok(())
}

fn valid_dataset_id(value: &str) -> bool {
    ["historical:", "dataset:", "fixture:"]
        .iter()
        .any(|prefix| value.starts_with(prefix))
}

fn validate_parameter(parameter: &StrategyParameter) -> Result<()> {
    if parameter.name.trim().is_empty()
        || parameter.name.chars().count() > 64
        || parameter.name.chars().any(char::is_control)
        || parameter.value.chars().count() > 256
        || parameter.value.chars().any(char::is_control)
    {
        return Err(TradeXError::new("STRATEGY_PARAMETER_INVALID"));
    }
    Ok(())
}

fn valid_token(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

pub fn fixture_enabled() -> bool {
    cfg!(feature = "integration-test") && std::env::var_os("TRADEX_STRATEGY_FIXTURE").is_some()
}

pub fn execute(
    version: &StrategyVersion,
    request: &StrategyRunRequest,
    parameters: &[StrategyParameter],
    observed_at: &str,
) -> Result<RunOutcome> {
    if fixture_enabled() {
        return Ok(
            match request
                .fixture_scenario
                .clone()
                .unwrap_or(StrategyFixtureScenario::Success)
            {
                StrategyFixtureScenario::Success => RunOutcome::Completed(StrategySignal {
                    instrument_id: request.instrument_id.clone(),
                    direction: StrategyDirection::Hold,
                    desired_exposure: "0".into(),
                    observed_at: observed_at.into(),
                    strategy_version_id: version.strategy_version_id.clone(),
                    strategy_hash: version.source_hash.clone(),
                    dataset_id: request.dataset_id.clone(),
                }),
                StrategyFixtureScenario::Failure => RunOutcome::Failed(StrategyFailure {
                    code: "STRATEGY_FIXTURE_FAILED".into(),
                    reason: "The integration strategy fixture returned a typed failure.".into(),
                    remediation: vec!["retry_strategy_run".into()],
                }),
                StrategyFixtureScenario::Cancelled => RunOutcome::Cancelled(StrategyFailure {
                    code: "STRATEGY_CANCELLED".into(),
                    reason: "The integration strategy fixture was cancelled.".into(),
                    remediation: vec!["retry_strategy_run".into()],
                }),
            },
        );
    }
    run_controlled_worker(version, request, parameters, observed_at)
}

fn run_controlled_worker(
    version: &StrategyVersion,
    request: &StrategyRunRequest,
    _parameters: &[StrategyParameter],
    observed_at: &str,
) -> Result<RunOutcome> {
    let worker = std::env::var_os("TRADEX_STRATEGY_WORKER_PATH")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent().map(|parent| {
                    let direct = parent.join("strategy-worker");
                    if direct.exists() {
                        direct
                    } else {
                        parent
                            .parent()
                            .map(|root| root.join("strategy-worker"))
                            .unwrap_or(direct)
                    }
                })
            })
        });
    let Some(worker) = worker else {
        return Ok(RunOutcome::Failed(worker_failure(
            "Strategy worker is unavailable.",
        )));
    };
    let token = uuid::Uuid::new_v4().to_string();
    let mut child = match Command::new(worker)
        .env_clear()
        .env("TRADEX_STRATEGY_WORKER", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker could not start.",
            )));
        }
    };
    let request_line = serde_json::to_string(&WorkerRequest {
        protocol_version: 1,
        session_token: &token,
        operation: "run",
        strategy_hash: &version.source_hash,
        strategy_version_id: &version.strategy_version_id,
        instrument_id: &request.instrument_id,
        dataset_id: &request.dataset_id,
        observed_at,
    })
    .map_err(|_| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "protocolVersion": 1,
            "sessionToken": token.clone(),
            "ok": true,
        })
    )
    .map_err(|_| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    writeln!(stdin, "{request_line}").map_err(|_| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    drop(stdin);
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    let mut lines = BufReader::new(stdout).lines();
    let handshake = lines
        .next()
        .transpose()
        .map_err(|_| TradeXError::new("STRATEGY_WORKER_FAILED"))?
        .ok_or_else(|| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    let handshake: WorkerHandshake =
        serde_json::from_str(&handshake).map_err(|_| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    if !handshake.ok || handshake.protocol_version != 1 || handshake.session_token != token {
        let _ = child.kill();
        return Ok(RunOutcome::Failed(worker_failure(
            "Strategy worker handshake was rejected.",
        )));
    }
    let result_line = lines
        .next()
        .transpose()
        .map_err(|_| TradeXError::new("STRATEGY_WORKER_FAILED"))?
        .ok_or_else(|| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    if result_line.len() > MAX_WORKER_OUTPUT {
        let _ = child.kill();
        return Ok(RunOutcome::Failed(worker_failure(
            "Strategy worker output exceeded its limit.",
        )));
    }
    let result: WorkerResult = serde_json::from_str(&result_line)
        .map_err(|_| TradeXError::new("STRATEGY_WORKER_FAILED"))?;
    let _ = child.wait();
    if result.protocol_version != 1 || result.session_token != token {
        return Ok(RunOutcome::Failed(worker_failure(
            "Strategy worker response was rejected.",
        )));
    }
    if result.ok {
        let Some(signal) = result.signal else {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker returned no signal.",
            )));
        };
        if signal.strategy_version_id != version.strategy_version_id
            || signal.strategy_hash != version.source_hash
            || signal.instrument_id != request.instrument_id
            || signal.dataset_id != request.dataset_id
        {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker signal identity was rejected.",
            )));
        }
        Ok(RunOutcome::Completed(signal))
    } else {
        Ok(RunOutcome::Failed(result.failure.unwrap_or_else(|| {
            worker_failure("Strategy worker returned a typed failure.")
        })))
    }
}

fn worker_failure(reason: &str) -> StrategyFailure {
    StrategyFailure {
        code: "STRATEGY_WORKER_FAILED".into(),
        reason: reason.into(),
        remediation: vec!["check_strategy_worker".into(), "retry_strategy_run".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(source: &str) -> StrategyDefinition {
        StrategyDefinition {
            name: "Example".into(),
            source: source.into(),
            language: "python".into(),
            runtime: "sandbox-v1".into(),
            parameters: vec![StrategyParameter {
                name: "window".into(),
                value: "20".into(),
            }],
        }
    }

    #[test]
    fn canonical_hash_is_stable_and_source_sensitive() {
        let first = canonical_version_hash(&definition("return 1")).unwrap();
        assert_eq!(
            first,
            canonical_version_hash(&definition("return 1")).unwrap()
        );
        assert_ne!(
            first,
            canonical_version_hash(&definition("return 2")).unwrap()
        );
    }

    #[test]
    fn source_allows_bounded_multiline_code() {
        assert!(validate_definition(&definition("return 1\nreturn 2")).is_ok());
        assert!(validate_definition(&definition("return 1\u{0000}")).is_err());
    }

    #[test]
    fn validation_rejects_duplicate_parameters() {
        let mut value = definition("return 1");
        value.parameters.push(StrategyParameter {
            name: "window".into(),
            value: "30".into(),
        });
        assert_eq!(
            validate_definition(&value).unwrap_err().code,
            "STRATEGY_PARAMETER_INVALID"
        );
    }
}
