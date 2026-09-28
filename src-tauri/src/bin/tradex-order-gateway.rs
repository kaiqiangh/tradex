use serde_json::{Value, json};
use std::{
    io::{self, Read, Write},
    os::fd::FromRawFd,
    os::unix::net::UnixStream,
};
use tradex::{
    order_gateway::{GatewayDispatchIntent, GatewayDispatchPackage},
    provider_io::{BrokerHttp, CredentialVault, Credentials, PrivilegedLiveOperation},
};
use zeroize::Zeroizing;

const PROTOCOL_VERSION: u64 = 1;
const CHANNEL_FD: i32 = 3;
const MAX_FRAME_BYTES: usize = 65_536;

fn main() {
    if run().is_err() {
        std::process::exit(2);
    }
}

fn run() -> io::Result<()> {
    // SAFETY: The parent passes a connected UnixStream at descriptor 3 before exec.
    let mut channel = unsafe { UnixStream::from_raw_fd(CHANNEL_FD) };
    let hello = read_frame(&mut channel)?;
    let mut message: Value = serde_json::from_slice(&hello)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid handshake"))?;
    let object = message
        .as_object_mut()
        .filter(|object| object.len() == 3)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid handshake"))?;
    let credential = match object.remove("sessionCredential") {
        Some(Value::String(value)) if !value.is_empty() && value.len() <= 128 => {
            Zeroizing::new(value)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "invalid session",
            ));
        }
    };
    if object.get("kind").and_then(Value::as_str) != Some("hello")
        || object.get("protocolVersion").and_then(Value::as_u64) != Some(PROTOCOL_VERSION)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incompatible protocol",
        ));
    }
    send_authenticated(&mut channel, &credential, "ready", json!({}))?;

    loop {
        let request = match read_authenticated(&mut channel, &credential) {
            Ok(request) => request,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error),
        };
        match request.get("kind").and_then(Value::as_str) {
            Some("shutdown") if exact_keys(&request, &["kind"]) => return Ok(()),
            Some("dispatch") if exact_keys(&request, &["kind", "attemptId"]) => {
                let attempt_id = request
                    .get("attemptId")
                    .and_then(Value::as_str)
                    .filter(|value| valid_identity(value, 128))
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid attempt"))?;
                dispatch(&mut channel, &credential, attempt_id)?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unsupported request",
                ));
            }
        }
    }
}

fn dispatch(channel: &mut UnixStream, credential: &str, attempt_id: &str) -> io::Result<()> {
    send_authenticated(
        channel,
        credential,
        "grant_request",
        json!({"attemptId":attempt_id}),
    )?;
    let grant_reply = read_authenticated(channel, credential)?;
    match grant_reply.get("kind").and_then(Value::as_str) {
        Some("deny") if exact_keys(&grant_reply, &["kind", "errorCode"]) => return Ok(()),
        Some("grant") if exact_keys(&grant_reply, &["kind", "package"]) => {}
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid grant reply",
            ));
        }
    }
    let package: GatewayDispatchPackage =
        serde_json::from_value(grant_reply.get("package").cloned().unwrap_or(Value::Null))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid grant"))?;
    if package.validate(attempt_id, credential).is_err() {
        pre_dispatch_failure(
            channel,
            credential,
            attempt_id,
            &package.grant.grant_id,
            "GATEWAY_AUTH_FAILED",
        )?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    let http = {
        #[cfg(feature = "integration-test")]
        if let Some(url) = package.local_test_base_url.as_deref() {
            BrokerHttp::for_loopback_test(url).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "invalid local test provider")
            })?
        } else {
            BrokerHttp::default()
        }
        #[cfg(not(feature = "integration-test"))]
        {
            BrokerHttp::default()
        }
    };
    #[cfg(not(target_os = "macos"))]
    let http = BrokerHttp::default();

    #[cfg(target_os = "macos")]
    let prepared = {
        match &package.intent {
            GatewayDispatchIntent::Place(proposal) => {
                tradex::provider_io::prepare_trading212_live_mutation(
                    &package.account,
                    &package.credential_reference,
                    PrivilegedLiveOperation::Place(proposal),
                    &GatewayVault,
                    &http,
                )
            }
            GatewayDispatchIntent::Cancel(intent)
                if package.account.provider_id == "trading212" =>
            {
                tradex::provider_io::prepare_trading212_live_mutation(
                    &package.account,
                    &package.credential_reference,
                    PrivilegedLiveOperation::Cancel(intent),
                    &GatewayVault,
                    &http,
                )
            }
            GatewayDispatchIntent::Cancel(intent) if package.account.provider_id == "binance" => {
                tradex::provider_io::prepare_binance_live_cancel_mutation(
                    &package.account,
                    &package.credential_reference,
                    intent,
                    &GatewayVault,
                    &http,
                )
            }
            _ => Err(tradex::protocol::TradeXError::new("PROVIDER_UNSUPPORTED")),
        }
    };
    #[cfg(not(target_os = "macos"))]
    let prepared: tradex::protocol::Result<tradex::provider_io::PreparedLiveMutation> =
        Err(tradex::protocol::TradeXError::new("PROVIDER_UNSUPPORTED"));

    let prepared = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            pre_dispatch_failure(
                channel,
                credential,
                attempt_id,
                &package.grant.grant_id,
                &error.code,
            )?;
            return Ok(());
        }
    };

    send_authenticated(
        channel,
        credential,
        "begin_request",
        json!({"attemptId":attempt_id,"grantId":package.grant.grant_id}),
    )?;
    let begin_reply = read_authenticated(channel, credential)?;
    match begin_reply.get("kind").and_then(Value::as_str) {
        Some("deny") if exact_keys(&begin_reply, &["kind", "errorCode"]) => return Ok(()),
        Some("submitting")
            if exact_keys(&begin_reply, &["kind", "attemptId"])
                && begin_reply.get("attemptId").and_then(Value::as_str) == Some(attempt_id) => {}
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid submission boundary",
            ));
        }
    }

    // A mutation is attempted once only after the Control Plane acknowledged durable SUBMITTING.
    let outcome = prepared.send(&http);
    send_authenticated(
        channel,
        credential,
        "mutation_result",
        json!({"attemptId":attempt_id,"outcome":outcome}),
    )?;
    let result = read_authenticated(channel, credential)?;
    if !exact_keys(&result, &["kind", "attemptId"])
        || result.get("kind").and_then(Value::as_str) != Some("completed")
        || result.get("attemptId").and_then(Value::as_str) != Some(attempt_id)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing completion acknowledgement",
        ));
    }
    Ok(())
}

#[cfg(feature = "integration-test")]
struct GatewayVault;

#[cfg(feature = "integration-test")]
impl CredentialVault for GatewayVault {
    fn put(&self, _reference: &str, _credentials: &Credentials) -> tradex::protocol::Result<()> {
        Err(tradex::protocol::TradeXError::new(
            "CREDENTIAL_STORE_FAILED",
        ))
    }
    fn get(&self, reference: &str) -> tradex::protocol::Result<Credentials> {
        if !reference.contains("/trading212/LIVE/") && !reference.contains("/binance/LIVE/") {
            return Err(tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        Credentials::new(vec![
            "synthetic-api-key".into(),
            "synthetic-api-secret".into(),
        ])
        .map_err(|_| tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"))
    }
    fn remove(&self, _reference: &str) -> tradex::protocol::Result<()> {
        Ok(())
    }
}

#[cfg(all(target_os = "macos", not(feature = "integration-test")))]
struct GatewayVault;

#[cfg(all(target_os = "macos", not(feature = "integration-test")))]
impl CredentialVault for GatewayVault {
    fn put(&self, reference: &str, credentials: &Credentials) -> tradex::protocol::Result<()> {
        tradex::provider_io::NativeVault.put(reference, credentials)
    }
    fn get(&self, reference: &str) -> tradex::protocol::Result<Credentials> {
        tradex::provider_io::NativeVault.get(reference)
    }
    fn remove(&self, reference: &str) -> tradex::protocol::Result<()> {
        tradex::provider_io::NativeVault.remove(reference)
    }
}

fn pre_dispatch_failure(
    channel: &mut UnixStream,
    credential: &str,
    attempt_id: &str,
    grant_id: &str,
    error_code: &str,
) -> io::Result<()> {
    let code = if valid_identity(error_code, 128) {
        error_code
    } else {
        "GATEWAY_PROCESS_FAILED"
    };
    send_authenticated(
        channel,
        credential,
        "pre_dispatch_failure",
        json!({"attemptId":attempt_id,"grantId":grant_id,"errorCode":code}),
    )?;
    let response = read_authenticated(channel, credential)?;
    if !exact_keys(&response, &["kind", "state"])
        || response.get("kind").and_then(Value::as_str) != Some("stopped")
        || response.get("state").and_then(Value::as_str) != Some("INVALIDATED")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing stop acknowledgement",
        ));
    }
    Ok(())
}

fn read_authenticated(stream: &mut UnixStream, expected: &str) -> io::Result<Value> {
    let frame = read_frame(stream)?;
    let mut message: Value = serde_json::from_slice(&frame)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid message"))?;
    let object = message
        .as_object_mut()
        .filter(|object| object.len() >= 3)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid message"))?;
    let credential = match object.remove("sessionCredential") {
        Some(Value::String(value)) => Zeroizing::new(value),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "invalid session",
            ));
        }
    };
    if credential.as_str() != expected {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "invalid session",
        ));
    }
    if object.get("protocolVersion").and_then(Value::as_u64) != Some(PROTOCOL_VERSION) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incompatible protocol",
        ));
    }
    object.remove("protocolVersion");
    Ok(message)
}

fn send_authenticated(
    stream: &mut UnixStream,
    credential: &str,
    kind: &str,
    mut payload: Value,
) -> io::Result<()> {
    let object = payload
        .as_object_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid message"))?;
    object.insert("kind".into(), Value::String(kind.into()));
    object.insert("protocolVersion".into(), Value::from(PROTOCOL_VERSION));
    object.insert("sessionCredential".into(), Value::String(credential.into()));
    let frame = Zeroizing::new(
        serde_json::to_vec(&payload)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid message"))?,
    );
    write_frame(stream, &frame)
}

fn exact_keys(value: &Value, expected: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
    })
}

fn valid_identity(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn read_frame(stream: &mut UnixStream) -> io::Result<Zeroizing<Vec<u8>>> {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut frame = Zeroizing::new(vec![0_u8; length]);
    stream.read_exact(&mut frame)?;
    Ok(frame)
}

fn write_frame(stream: &mut UnixStream, payload: &[u8]) -> io::Result<()> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let length = u32::try_from(payload.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "frame too large"))?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(payload)
}
