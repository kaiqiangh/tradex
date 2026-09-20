use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerRequest {
    protocol_version: u32,
    session_token: String,
    operation: String,
    strategy_hash: String,
    strategy_version_id: String,
    instrument_id: String,
    dataset_id: String,
    observed_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkerResponse<'a> {
    protocol_version: u32,
    session_token: &'a str,
    ok: bool,
    signal: Option<serde_json::Value>,
    failure: Option<Failure<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Failure<'a> {
    code: &'a str,
    reason: &'a str,
    remediation: [&'a str; 1],
}

fn main() {
    if std::env::var("TRADEX_STRATEGY_WORKER").ok().as_deref() != Some("1") {
        std::process::exit(2);
    }
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let Some(Ok(handshake)) = lines.next() else {
        std::process::exit(3)
    };
    let Ok(handshake) = serde_json::from_str::<serde_json::Value>(&handshake) else {
        std::process::exit(3)
    };
    let protocol_version = handshake.get("protocolVersion").and_then(|v| v.as_u64());
    let Some(token) = handshake.get("sessionToken").and_then(|v| v.as_str()) else {
        std::process::exit(3)
    };
    if protocol_version != Some(1) || token.is_empty() {
        std::process::exit(3);
    }
    println!(
        "{}",
        serde_json::json!({"protocolVersion":1,"sessionToken":token,"ok":true})
    );
    let _ = io::stdout().flush();
    let Some(Ok(line)) = lines.next() else {
        std::process::exit(3)
    };
    let Ok(request) = serde_json::from_str::<WorkerRequest>(&line) else {
        std::process::exit(3)
    };
    if request.protocol_version != 1
        || request.session_token != token
        || request.operation != "run"
        || request.strategy_hash.is_empty()
        || request.strategy_version_id.is_empty()
        || request.instrument_id.is_empty()
        || request.dataset_id.is_empty()
        || request.observed_at.is_empty()
    {
        let failure = Failure {
            code: "STRATEGY_WORKER_DENIED",
            reason: "The strategy worker request was outside its allowlist.",
            remediation: ["check_strategy_worker"],
        };
        println!(
            "{}",
            serde_json::to_string(&WorkerResponse {
                protocol_version: 1,
                session_token: token,
                ok: false,
                signal: None,
                failure: Some(failure)
            })
            .unwrap()
        );
        return;
    }
    let failure = Failure {
        code: "STRATEGY_RUNTIME_UNAVAILABLE",
        reason: "No production strategy runtime is configured for this worker.",
        remediation: ["configure_strategy_runtime"],
    };
    println!(
        "{}",
        serde_json::to_string(&WorkerResponse {
            protocol_version: 1,
            session_token: token,
            ok: false,
            signal: None,
            failure: Some(failure)
        })
        .unwrap()
    );
}
