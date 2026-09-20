use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn worker_rejects_operations_outside_the_allowlist() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_strategy-worker"))
        .env_clear()
        .env("TRADEX_STRATEGY_WORKER", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(
        stdin,
        r#"{{"protocolVersion":1,"sessionToken":"test-token","ok":true}}"#
    )
    .unwrap();
    writeln!(stdin, r#"{{"protocolVersion":1,"sessionToken":"test-token","operation":"read_secret","strategyHash":"sha256:test","strategyVersionId":"version","instrumentId":"equity:US:AAPL","datasetId":"historical:test","observedAt":"2026-01-01T00:00:00Z"}}"#).unwrap();
    drop(stdin);
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let _handshake = lines.next().unwrap().unwrap();
    let result = lines.next().unwrap().unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["failure"]["code"], "STRATEGY_WORKER_DENIED");
    assert!(child.wait().unwrap().success());
}

#[test]
fn worker_fails_closed_without_a_production_runtime() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_strategy-worker"))
        .env_clear()
        .env("TRADEX_STRATEGY_WORKER", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(
        stdin,
        r#"{{"protocolVersion":1,"sessionToken":"test-token","ok":true}}"#
    )
    .unwrap();
    writeln!(stdin, r#"{{"protocolVersion":1,"sessionToken":"test-token","operation":"run","strategyHash":"sha256:test","strategyVersionId":"version","instrumentId":"equity:US:AAPL","datasetId":"historical:fixture","observedAt":"2026-01-01T00:00:00Z"}}"#).unwrap();
    drop(stdin);
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let _handshake = lines.next().unwrap().unwrap();
    let result: serde_json::Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["failure"]["code"], "STRATEGY_RUNTIME_UNAVAILABLE");
    assert!(child.wait().unwrap().success());
}
