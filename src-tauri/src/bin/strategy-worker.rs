use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[cfg(feature = "integration-test")]
use std::{
    net::{SocketAddr, TcpStream},
    process::Command,
    time::Duration,
};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "integration-test")]
    probe: Option<ContainmentProbe>,
}

#[cfg(feature = "integration-test")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContainmentProbe {
    file_read_denied: bool,
    file_write_denied: bool,
    network_denied: bool,
    child_process_denied: bool,
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
    #[cfg(feature = "integration-test")]
    let operation_allowed = matches!(request.operation.as_str(), "run" | "containment_probe");
    #[cfg(not(feature = "integration-test"))]
    let operation_allowed = request.operation == "run";
    if request.protocol_version != 1
        || request.session_token != token
        || !operation_allowed
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
                failure: Some(failure),
                #[cfg(feature = "integration-test")]
                probe: None,
            })
            .unwrap()
        );
        return;
    }
    #[cfg(feature = "integration-test")]
    if request.operation == "containment_probe" {
        let read_path =
            std::env::var("TRADEX_STRATEGY_PROBE_PATH").unwrap_or_else(|_| "/Users".into());
        let file_read_denied = std::fs::read(read_path)
            .is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied);
        let file_write_denied = std::fs::write("/tmp/tradex-strategy-containment-probe", b"probe")
            .is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied);
        let network_denied = "127.0.0.1:9"
            .parse::<SocketAddr>()
            .ok()
            .is_some_and(|address| {
                TcpStream::connect_timeout(&address, Duration::from_millis(100))
                    .is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied)
            });
        let child_process_denied = Command::new("/bin/sh")
            .arg("-c")
            .arg("true")
            .status()
            .is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied);
        println!(
            "{}",
            serde_json::to_string(&WorkerResponse {
                protocol_version: 1,
                session_token: token,
                ok: false,
                signal: None,
                failure: Some(Failure {
                    code: "STRATEGY_CONTAINMENT_PROBE",
                    reason: "Integration-only containment probe completed.",
                    remediation: ["inspect_strategy_sandbox"],
                }),
                probe: Some(ContainmentProbe {
                    file_read_denied,
                    file_write_denied,
                    network_denied,
                    child_process_denied,
                }),
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
            failure: Some(failure),
            #[cfg(feature = "integration-test")]
            probe: None,
        })
        .unwrap()
    );
}
