use crate::market;
use crate::protocol::{
    Result, StrategyDefinition, StrategyDirection, StrategyFailure, StrategyFixtureScenario,
    StrategyParameter, StrategyRunRequest, StrategySignal, StrategyVersion, TradeXError,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};
use std::{
    path::Path,
    process::{Child, Command, Stdio},
};

const MAX_WORKER_OUTPUT: usize = 16_384;
const WORKER_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(target_os = "macos")]
const MACOS_WORKER_SANDBOX: &str = r#"
(version 1)
(import "system.sb")
(deny process-exec*)
(allow process-exec (literal (param "WORKER_PATH")))
(deny network*)
(deny file-read* (subpath "/Users"))
(deny file-read* (subpath "/private"))
(deny file-read* (subpath "/etc"))
(deny file-read* (subpath "/tmp"))
(deny file-write* (subpath "/"))
"#;

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
    observed_at: &'a str,
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
    observed_at: &str,
) -> Result<String> {
    let input = CanonicalRun {
        strategy_version_id: &version.strategy_version_id,
        strategy_hash: &version.source_hash,
        instrument_id: &request.instrument_id,
        dataset_id: &request.dataset_id,
        start_at: &request.start_at,
        end_at: &request.end_at,
        observed_at,
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
    let mut parameter_names = std::collections::HashSet::new();
    for parameter in &request.parameters {
        validate_parameter(parameter)?;
        if !parameter_names.insert(parameter.name.as_str()) {
            return Err(TradeXError::new("STRATEGY_PARAMETER_INVALID"));
        }
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

fn valid_timestamp(value: &str) -> bool {
    value.len() <= 64
        && !value.trim().is_empty()
        && !value.chars().any(char::is_control)
        && time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
            .is_ok()
}

fn valid_hash(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn validate_failure(failure: &StrategyFailure) -> Result<()> {
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
        return Err(TradeXError::new("STRATEGY_WORKER_FAILED"));
    }
    Ok(())
}

pub(crate) fn validate_signal(
    signal: &StrategySignal,
    version: &StrategyVersion,
    request: &StrategyRunRequest,
    observed_at: &str,
) -> Result<()> {
    if signal.strategy_version_id != version.strategy_version_id
        || signal.source_ref != format!("strategy-version:{}", version.strategy_version_id)
        || signal.strategy_hash != version.source_hash
        || signal.instrument_id != request.instrument_id
        || signal.dataset_id != request.dataset_id
        || signal.observed_at != observed_at
        || !market::validate_instrument_id(&signal.instrument_id)
        || !valid_dataset_id(&signal.dataset_id)
        || !valid_hash(&signal.strategy_hash)
        || signal.source_ref.len() > 256
        || signal.source_ref.chars().any(char::is_control)
        || signal.desired_exposure.trim().is_empty()
        || signal.desired_exposure.len() > 64
        || signal.desired_exposure.chars().any(char::is_control)
        || crate::provider_io::decimal(&serde_json::Value::String(signal.desired_exposure.clone()))
            .is_err()
        || !valid_timestamp(&signal.observed_at)
    {
        return Err(TradeXError::new("STRATEGY_SIGNAL_INVALID"));
    }
    Ok(())
}

fn valid_dataset_id(value: &str) -> bool {
    matches!(value, "historical:fixture")
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
    cfg!(feature = "integration-test")
        && std::env::var("TRADEX_STRATEGY_FIXTURE").ok().as_deref() == Some("1")
}

pub fn execute(
    version: &StrategyVersion,
    request: &StrategyRunRequest,
    parameters: &[StrategyParameter],
    observed_at: &str,
    cancel: Option<&AtomicBool>,
) -> Result<RunOutcome> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Acquire)) {
        return Ok(RunOutcome::Cancelled(cancelled_failure()));
    }
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
                    source_ref: format!("strategy-version:{}", version.strategy_version_id),
                    strategy_hash: version.source_hash.clone(),
                    dataset_id: request.dataset_id.clone(),
                }),
                StrategyFixtureScenario::Failure => RunOutcome::Failed(StrategyFailure {
                    code: "STRATEGY_FIXTURE_FAILED".into(),
                    reason: "The integration strategy fixture returned a typed failure.".into(),
                    remediation: vec!["retry_strategy_run".into()],
                }),
                StrategyFixtureScenario::Cancelled => {
                    for _ in 0..500 {
                        if cancel.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                            return Ok(RunOutcome::Cancelled(cancelled_failure()));
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    RunOutcome::Cancelled(StrategyFailure {
                        code: "STRATEGY_CANCELLED".into(),
                        reason: "The integration strategy fixture was cancelled.".into(),
                        remediation: vec!["retry_strategy_run".into()],
                    })
                }
            },
        );
    }
    run_controlled_worker(version, request, parameters, observed_at, cancel)
}

fn run_controlled_worker(
    version: &StrategyVersion,
    request: &StrategyRunRequest,
    _parameters: &[StrategyParameter],
    observed_at: &str,
    cancel: Option<&AtomicBool>,
) -> Result<RunOutcome> {
    let worker = std::env::current_exe().ok().and_then(|path| {
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
    });
    let Some(worker) = worker else {
        return Ok(RunOutcome::Failed(worker_failure(
            "Strategy worker is unavailable.",
        )));
    };
    let token = uuid::Uuid::new_v4().to_string();
    let Some(mut command) = sandboxed_worker_command(&worker) else {
        return Ok(RunOutcome::Failed(StrategyFailure {
            code: "STRATEGY_SANDBOX_UNAVAILABLE".into(),
            reason: "No supported OS worker sandbox is available.".into(),
            remediation: vec!["configure_strategy_sandbox".into()],
        }));
    };
    let mut child = match command
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
    let started = Instant::now();
    let request_line = match serde_json::to_string(&WorkerRequest {
        protocol_version: 1,
        session_token: &token,
        operation: "run",
        strategy_hash: &version.source_hash,
        strategy_version_id: &version.strategy_version_id,
        instrument_id: &request.instrument_id,
        dataset_id: &request.dataset_id,
        observed_at,
    }) {
        Ok(line) => line,
        Err(_) => return worker_failed(&mut child, "Strategy worker request was invalid."),
    };
    let Some(mut stdin) = child.stdin.take() else {
        return worker_failed(&mut child, "Strategy worker input was unavailable.");
    };
    if writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "protocolVersion": 1,
            "sessionToken": token.clone(),
            "ok": true,
        })
    )
    .is_err()
        || writeln!(stdin, "{request_line}").is_err()
    {
        return worker_failed(&mut child, "Strategy worker input was unavailable.");
    }
    drop(stdin);
    let Some(stdout) = child.stdout.take() else {
        return worker_failed(&mut child, "Strategy worker output was unavailable.");
    };
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut stdout = stdout;
        let result = read_bounded_line(&mut stdout, MAX_WORKER_OUTPUT)
            .and_then(|line| {
                line.ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "missing handshake")
                })
            })
            .and_then(|handshake| {
                read_bounded_line(&mut stdout, MAX_WORKER_OUTPUT)
                    .and_then(|line| {
                        line.ok_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "missing result")
                        })
                    })
                    .map(|result| (handshake, result))
            });
        let _ = sender.send(result);
    });
    let result_lines = loop {
        if cancel.is_some_and(|flag| flag.load(Ordering::Acquire)) {
            reap_child(&mut child);
            return Ok(RunOutcome::Cancelled(cancelled_failure()));
        }
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(result) => match result {
                Ok(lines) => break lines,
                Err(_) => return worker_failed(&mut child, "Strategy worker output was invalid."),
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if started.elapsed() >= WORKER_TIMEOUT {
                    reap_child(&mut child);
                    return Ok(RunOutcome::Failed(worker_failure(
                        "Strategy worker exceeded its deadline.",
                    )));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return worker_failed(&mut child, "Strategy worker output was unavailable.");
            }
        }
    };
    let handshake: WorkerHandshake = match serde_json::from_str(&result_lines.0) {
        Ok(handshake) => handshake,
        Err(_) => return worker_failed(&mut child, "Strategy worker handshake was invalid."),
    };
    if !handshake.ok || handshake.protocol_version != 1 || handshake.session_token != token {
        return worker_failed(&mut child, "Strategy worker handshake was rejected.");
    }
    let result: WorkerResult = match serde_json::from_str(&result_lines.1) {
        Ok(result) => result,
        Err(_) => return worker_failed(&mut child, "Strategy worker response was invalid."),
    };
    loop {
        let exited = match child.try_wait() {
            Ok(status) => status.is_some(),
            Err(_) => return worker_failed(&mut child, "Strategy worker status was unavailable."),
        };
        if exited {
            break;
        }
        if cancel.is_some_and(|flag| flag.load(Ordering::Acquire)) {
            reap_child(&mut child);
            return Ok(RunOutcome::Cancelled(cancelled_failure()));
        }
        if started.elapsed() >= WORKER_TIMEOUT {
            reap_child(&mut child);
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker exceeded its deadline.",
            )));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    if result.protocol_version != 1 || result.session_token != token {
        return Ok(RunOutcome::Failed(worker_failure(
            "Strategy worker response was rejected.",
        )));
    }
    if result.ok {
        if result.failure.is_some() {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker response was inconsistent.",
            )));
        }
        let Some(signal) = result.signal else {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker returned no signal.",
            )));
        };
        if validate_signal(&signal, version, request, observed_at).is_err() {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker signal was rejected.",
            )));
        }
        Ok(RunOutcome::Completed(signal))
    } else {
        if result.signal.is_some() {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker response was inconsistent.",
            )));
        }
        let failure = result
            .failure
            .unwrap_or_else(|| worker_failure("Strategy worker returned a typed failure."));
        if validate_failure(&failure).is_err() {
            return Ok(RunOutcome::Failed(worker_failure(
                "Strategy worker failure was invalid.",
            )));
        }
        Ok(RunOutcome::Failed(failure))
    }
}

fn sandboxed_worker_command(worker: &Path) -> Option<Command> {
    #[cfg(target_os = "macos")]
    {
        let sandbox = Path::new("/usr/bin/sandbox-exec");
        let worker_path = worker.to_str()?;
        if !sandbox.is_file() || !worker.is_file() || worker_path.chars().any(char::is_control) {
            return None;
        }
        let mut command = Command::new(sandbox);
        command
            .arg("-D")
            .arg(format!("WORKER_PATH={worker_path}"))
            .arg("-p")
            .arg(MACOS_WORKER_SANDBOX)
            .arg(worker);
        Some(command)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = worker;
        None
    }
}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub fn worker_sandbox_profile() -> &'static str {
    MACOS_WORKER_SANDBOX
}

fn reap_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn worker_failed(child: &mut Child, reason: &str) -> Result<RunOutcome> {
    reap_child(child);
    Ok(RunOutcome::Failed(worker_failure(reason)))
}

fn read_bounded_line<R: Read>(reader: &mut R, limit: usize) -> std::io::Result<Option<String>> {
    let mut bytes = Vec::with_capacity(limit.min(1024));
    loop {
        let mut byte = [0_u8; 1];
        match reader.read(&mut byte)? {
            0 if bytes.is_empty() => return Ok(None),
            0 => break,
            _ if byte[0] == b'\n' => break,
            _ => {
                if bytes.len() >= limit {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "line too long",
                    ));
                }
                bytes.push(byte[0]);
            }
        }
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "line is not utf8"))
}

fn worker_failure(reason: &str) -> StrategyFailure {
    StrategyFailure {
        code: "STRATEGY_WORKER_FAILED".into(),
        reason: reason.into(),
        remediation: vec!["check_strategy_worker".into(), "retry_strategy_run".into()],
    }
}

fn cancelled_failure() -> StrategyFailure {
    StrategyFailure {
        code: "STRATEGY_CANCELLED".into(),
        reason: "The strategy run was cancelled before completion.".into(),
        remediation: vec!["retry_strategy_run".into()],
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
