//! Private process boundary for live order execution.
//!
//! This process host is intentionally separate from `gateway_process`, which only owns the
//! model-provider sidecar.
use crate::{
    protocol::{
        CancellationIntent, ExecutionAttempt, ExecutionAttemptState, ExecutionDispatchGrant,
        ExecutionDispatchGrantStatus, ExecutionReservation, ExecutionReservationStatus,
        FinancialOperation, OrderProposal,
    },
    provider_io::LiveDispatchOutcome,
    providers::AccountConnection,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::{fs::MetadataExt, net::UnixStream, process::CommandExt},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};
use zeroize::Zeroizing;

const PROTOCOL_VERSION: u64 = 1;
const CHANNEL_FD: libc::c_int = 3;
const MAX_FRAME_BYTES: usize = 65_536;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);
const DISPATCH_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum GatewayDispatchIntent {
    Place(Box<OrderProposal>),
    Cancel(Box<CancellationIntent>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GatewayDispatchPackage {
    pub grant: ExecutionDispatchGrant,
    pub attempt: ExecutionAttempt,
    pub reservation: Option<ExecutionReservation>,
    pub account: AccountConnection,
    pub credential_reference: String,
    pub intent: GatewayDispatchIntent,
    #[cfg(feature = "integration-test")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_test_base_url: Option<String>,
}

impl GatewayDispatchPackage {
    pub fn validate(&self, attempt_id: &str, session_id: &str) -> Result<(), &'static str> {
        let grant = &self.grant;
        let attempt = &self.attempt;
        if grant.status != ExecutionDispatchGrantStatus::Issued
            || grant.gateway_session_id != session_id
            || grant.attempt_id != attempt_id
            || attempt.attempt_id != attempt_id
            || attempt.state != ExecutionAttemptState::Reserved
            || attempt.dispatch_disposition.is_some()
            || grant.workspace_id != attempt.workspace_id
            || grant.account_id != attempt.account_id
            || grant.operation != attempt.operation
            || grant.intent_id != attempt.intent_id
            || grant.intent_hash != attempt.intent_hash
            || grant.approval_id != attempt.approval_id
            || grant.reservation_id != attempt.reservation_id
            || grant.account_state_version != attempt.account_state_version
            || grant.intent_state_version != attempt.intent_state_version
            || grant.policy_version != attempt.policy_version
            || self.account.workspace_id != attempt.workspace_id
            || self.account.connection_id != attempt.account_id
            || self.account.state_version != attempt.account_state_version
            || self.account.environment != "LIVE"
            || !matches!(
                (
                    self.account.provider_id.as_str(),
                    attempt.environment.clone()
                ),
                (
                    "trading212",
                    crate::protocol::ExecutionContext::Trading212Live
                ) | ("binance", crate::protocol::ExecutionContext::BinanceLive)
                    | ("bitget", crate::protocol::ExecutionContext::BitgetLive)
            )
            || self.account.credential_ref() != self.credential_reference
            || self.account.connection_state != crate::providers::ConnectionState::Connected
        {
            return Err("GATEWAY_AUTH_FAILED");
        }
        match (&self.intent, attempt.operation) {
            (GatewayDispatchIntent::Place(proposal), FinancialOperation::PlaceOrder)
                if proposal.workspace_id == attempt.workspace_id
                    && proposal.proposal_id == attempt.intent_id
                    && proposal.proposal_hash == attempt.intent_hash
                    && proposal.fields.account_id.as_deref()
                        == Some(attempt.account_id.as_str())
                    && proposal.fields.environment
                        == crate::protocol::ExecutionContext::Trading212Live
                    && self.account.provider_id == "trading212"
                    && attempt.environment == crate::protocol::ExecutionContext::Trading212Live
                    && proposal.status == crate::protocol::OrderProposalStatus::Consumed
                    && self.reservation.as_ref().is_some_and(|reservation| {
                        reservation.status == ExecutionReservationStatus::Active
                            && Some(reservation.reservation_id.as_str())
                                == attempt.reservation_id.as_deref()
                            && reservation.attempt_id == attempt_id
                            && reservation.account_id == attempt.account_id
                    }) => {}
            (GatewayDispatchIntent::Cancel(intent), FinancialOperation::Cancel)
                if intent.workspace_id == attempt.workspace_id
                    && intent.cancellation_intent_id == attempt.intent_id
                    && intent.intent_hash == attempt.intent_hash
                    && intent.account_id == attempt.account_id
                    && intent.environment == attempt.environment
                    && Some(intent.provider_order_id.as_str())
                        == attempt.broker_order_id.as_deref()
                    && self.reservation.is_none() => {}
            _ => return Err("GATEWAY_AUTH_FAILED"),
        }
        Ok(())
    }
}

pub struct OrderGatewayHost {
    executable: PathBuf,
    pinned_sha256: String,
    child: Option<Child>,
    channel: Option<UnixStream>,
    session_credential: Option<Zeroizing<String>>,
}

impl OrderGatewayHost {
    pub fn try_acquire(
        host: &std::sync::Mutex<Self>,
    ) -> Result<std::sync::MutexGuard<'_, Self>, &'static str> {
        host.try_lock().map_err(|error| match error {
            std::sync::TryLockError::WouldBlock => "PROVIDER_BACKPRESSURE",
            std::sync::TryLockError::Poisoned(_) => "GATEWAY_UNAVAILABLE",
        })
    }
    pub fn new(executable: PathBuf, pinned_sha256: impl Into<String>) -> Self {
        Self {
            executable,
            pinned_sha256: pinned_sha256.into(),
            child: None,
            channel: None,
            session_credential: None,
        }
    }

    pub fn from_current_executable(pinned_sha256: impl Into<String>) -> Result<Self, &'static str> {
        let executable = std::env::current_exe().map_err(|_| "GATEWAY_BINARY_INVALID")?;
        let parent = executable.parent().ok_or("GATEWAY_BINARY_INVALID")?;
        Ok(Self::new(
            parent.join("tradex-order-gateway"),
            pinned_sha256,
        ))
    }

    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.is_running() {
            return Ok(());
        }
        self.stop();
        verify_executable(&self.executable, &self.pinned_sha256)?;

        let (mut parent_channel, child_channel) =
            UnixStream::pair().map_err(|_| "GATEWAY_PROCESS_FAILED")?;
        parent_channel
            .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
            .map_err(|_| "GATEWAY_PROCESS_FAILED")?;
        parent_channel
            .set_write_timeout(Some(HANDSHAKE_TIMEOUT))
            .map_err(|_| "GATEWAY_PROCESS_FAILED")?;

        let child_fd = child_channel.as_raw_fd();
        // Resolve this in the parent so `pre_exec` only performs async-signal-safe syscalls.
        let max_fd = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) }
            .clamp(0, libc::c_int::MAX as libc::c_long) as libc::c_int;
        if max_fd <= CHANNEL_FD + 1 {
            return Err("GATEWAY_PROCESS_FAILED");
        }
        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        // SAFETY: The child setup uses only async-signal-safe descriptor operations after fork.
        unsafe {
            command.pre_exec(move || {
                if child_fd != CHANNEL_FD && libc::dup2(child_fd, CHANNEL_FD) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::fcntl(CHANNEL_FD, libc::F_SETFD, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                for fd in (CHANNEL_FD + 1)..max_fd {
                    libc::close(fd);
                }
                Ok(())
            });
        }

        let mut child = command.spawn().map_err(|_| "GATEWAY_PROCESS_FAILED")?;
        drop(child_channel);

        let credential = Zeroizing::new(uuid::Uuid::new_v4().to_string());
        let hello = Zeroizing::new(
            serde_json::to_vec(&serde_json::json!({
                "kind": "hello",
                "protocolVersion": PROTOCOL_VERSION,
                "sessionCredential": credential.as_str()
            }))
            .map_err(|_| "GATEWAY_HANDSHAKE_FAILED")?,
        );
        if let Err(error) = write_frame(&mut parent_channel, &hello) {
            stop_child(&mut child);
            return Err(error);
        }
        let response = match read_frame(&mut parent_channel) {
            Ok(response) => response,
            Err(error) => {
                stop_child(&mut child);
                return Err(error);
            }
        };
        if let Err(error) = validate_ready(&response, &credential) {
            stop_child(&mut child);
            return Err(error);
        }
        parent_channel
            .set_read_timeout(Some(DISPATCH_TIMEOUT))
            .map_err(|_| "GATEWAY_PROCESS_FAILED")?;
        parent_channel
            .set_write_timeout(Some(DISPATCH_TIMEOUT))
            .map_err(|_| "GATEWAY_PROCESS_FAILED")?;

        self.child = Some(child);
        self.channel = Some(parent_channel);
        self.session_credential = Some(credential);
        Ok(())
    }

    pub fn dispatch_attempt(
        &mut self,
        attempt_id: &str,
        mut issue_grant: impl FnMut(&str, &str) -> std::result::Result<GatewayDispatchPackage, String>,
        mut begin_submission: impl FnMut(&str, &str) -> std::result::Result<(), String>,
        mut stop_before_dispatch: impl FnMut(&str, &str),
        mut complete_submission: impl FnMut(
            &str,
            &LiveDispatchOutcome,
        ) -> std::result::Result<(), String>,
    ) -> Result<(), &'static str> {
        if !self.is_running() || !valid_identity(attempt_id, 128) {
            return Err("GATEWAY_UNAVAILABLE");
        }
        let channel = self.channel.as_mut().ok_or("GATEWAY_UNAVAILABLE")?;
        let session = self
            .session_credential
            .as_ref()
            .ok_or("GATEWAY_UNAVAILABLE")?;
        send_authenticated(
            channel,
            session,
            "dispatch",
            json!({"attemptId":attempt_id}),
        )?;

        let grant_request = read_authenticated(channel, session)?;
        if request_kind(&grant_request) != Some("grant_request")
            || exact_keys(&grant_request, &["kind", "attemptId"]).is_err()
            || grant_request.get("attemptId").and_then(Value::as_str) != Some(attempt_id)
        {
            stop_before_dispatch(attempt_id, "GATEWAY_PROTOCOL_INVALID");
            return Err("GATEWAY_PROTOCOL_INVALID");
        }
        let package = match issue_grant(attempt_id, session.as_str()) {
            Ok(package) => package,
            Err(code) => {
                let code = safe_gateway_error(&code);
                stop_before_dispatch(attempt_id, code);
                let _ = send_authenticated(channel, session, "deny", json!({"errorCode":code}));
                return Ok(());
            }
        };
        if package.validate(attempt_id, session.as_str()).is_err() {
            stop_before_dispatch(attempt_id, "GATEWAY_AUTH_FAILED");
            let _ = send_authenticated(
                channel,
                session,
                "deny",
                json!({"errorCode":"GATEWAY_AUTH_FAILED"}),
            );
            return Err("GATEWAY_AUTH_FAILED");
        }
        let provider = match package.account.provider_id.as_str() {
            "trading212" => "trading212",
            "binance" => "binance",
            "bitget" => "bitget",
            _ => return Err("GATEWAY_AUTH_FAILED"),
        };
        let account_id = package.account.connection_id.clone();
        // Count the serialized child path in the parent's provider budget.
        // Grant eligibility is checked again before durable SUBMITTING.
        let _permit = match crate::provider_io::acquire_p0_provider_slot(provider, &account_id) {
            Ok(permit) => permit,
            Err(error) => {
                let code = safe_gateway_error(&error.code);
                stop_before_dispatch(attempt_id, code);
                let _ = send_authenticated(channel, session, "deny", json!({"errorCode":code}));
                return Ok(());
            }
        };
        let grant_id = package.grant.grant_id.clone();
        send_authenticated(channel, session, "grant", json!({"package":package}))?;

        let ready = read_authenticated(channel, session)?;
        if request_kind(&ready) == Some("pre_dispatch_failure") {
            if exact_keys(
                &ready,
                &[
                    "kind",
                    "attemptId",
                    "grantId",
                    "errorCode",
                    "retryAfterSeconds",
                ],
            )
            .is_err()
                || ready.get("attemptId").and_then(Value::as_str) != Some(attempt_id)
                || ready.get("grantId").and_then(Value::as_str) != Some(grant_id.as_str())
            {
                stop_before_dispatch(attempt_id, "GATEWAY_PROTOCOL_INVALID");
                return Err("GATEWAY_PROTOCOL_INVALID");
            }
            apply_gateway_cooldown(&ready, provider, &account_id)?;
            let code = ready
                .get("errorCode")
                .and_then(Value::as_str)
                .map(safe_gateway_error)
                .unwrap_or("GATEWAY_PROCESS_FAILED");
            stop_before_dispatch(attempt_id, code);
            send_authenticated(channel, session, "stopped", json!({"state":"INVALIDATED"}))?;
            return Ok(());
        }
        if request_kind(&ready) != Some("begin_request")
            || exact_keys(&ready, &["kind", "attemptId", "grantId"]).is_err()
            || ready.get("attemptId").and_then(Value::as_str) != Some(attempt_id)
            || ready.get("grantId").and_then(Value::as_str) != Some(grant_id.as_str())
        {
            stop_before_dispatch(attempt_id, "GATEWAY_PROTOCOL_INVALID");
            return Err("GATEWAY_PROTOCOL_INVALID");
        }
        if let Err(code) = begin_submission(&grant_id, session.as_str()) {
            let code = safe_gateway_error(&code);
            stop_before_dispatch(attempt_id, code);
            let _ = send_authenticated(channel, session, "deny", json!({"errorCode":code}));
            return Ok(());
        }
        let unknown = LiveDispatchOutcome {
            state: ExecutionAttemptState::UnknownReconciling,
            broker_order_id: None,
            provider_status: None,
            error_code: Some("ORDER_STATUS_UNKNOWN".into()),
        };
        if send_authenticated(
            channel,
            session,
            "submitting",
            json!({"attemptId":attempt_id}),
        )
        .is_err()
        {
            let _ = complete_submission(attempt_id, &unknown);
            return Err("GATEWAY_PROCESS_FAILED");
        }
        let outcome_request = match read_authenticated(channel, session) {
            Ok(request) => request,
            Err(error) => {
                let _ = complete_submission(attempt_id, &unknown);
                return Err(error);
            }
        };
        if request_kind(&outcome_request) != Some("mutation_result")
            || exact_keys(
                &outcome_request,
                &["kind", "attemptId", "outcome", "retryAfterSeconds"],
            )
            .is_err()
            || outcome_request.get("attemptId").and_then(Value::as_str) != Some(attempt_id)
        {
            let _ = complete_submission(attempt_id, &unknown);
            return Err("GATEWAY_PROTOCOL_INVALID");
        }
        if let Err(error) = apply_gateway_cooldown(&outcome_request, provider, &account_id) {
            let _ = complete_submission(attempt_id, &unknown);
            return Err(error);
        }
        let outcome: LiveDispatchOutcome = match serde_json::from_value(
            outcome_request
                .get("outcome")
                .cloned()
                .unwrap_or(Value::Null),
        ) {
            Ok(outcome) if valid_outcome(&package.intent, &outcome) => outcome,
            _ => {
                let _ = complete_submission(attempt_id, &unknown);
                return Err("GATEWAY_PROTOCOL_INVALID");
            }
        };
        complete_submission(attempt_id, &outcome).map_err(|_| "WORKSPACE_OPEN_FAILED")?;
        send_authenticated(
            channel,
            session,
            "completed",
            json!({"attemptId":attempt_id}),
        )?;
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        self.child
            .as_mut()
            .is_some_and(|child| child.try_wait().is_ok_and(|status| status.is_none()))
    }

    pub fn stop(&mut self) {
        if let (Some(channel), Some(credential)) =
            (self.channel.as_mut(), self.session_credential.as_ref())
        {
            let message = Zeroizing::new(
                serde_json::to_vec(&serde_json::json!({
                    "kind": "shutdown",
                    "protocolVersion": PROTOCOL_VERSION,
                    "sessionCredential": credential.as_str()
                }))
                .unwrap_or_default(),
            );
            let _ = write_frame(channel, &message);
        }
        self.channel.take();
        self.session_credential.take();
        if let Some(mut child) = self.child.take() {
            stop_child(&mut child);
        }
    }
}

impl Drop for OrderGatewayHost {
    fn drop(&mut self) {
        self.stop();
    }
}

fn verify_executable(path: &Path, pinned_sha256: &str) -> Result<(), &'static str> {
    if pinned_sha256.len() != 64
        || !pinned_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("GATEWAY_BINARY_INVALID");
    }
    let metadata = path
        .symlink_metadata()
        .map_err(|_| "GATEWAY_BINARY_INVALID")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > 200 * 1024 * 1024
        || metadata.mode() & 0o111 == 0
    {
        return Err("GATEWAY_BINARY_INVALID");
    }
    let mut file = File::open(path).map_err(|_| "GATEWAY_BINARY_INVALID")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "GATEWAY_BINARY_INVALID")?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    if hex::encode(hasher.finalize()) != pinned_sha256 {
        return Err("GATEWAY_BINARY_INVALID");
    }
    Ok(())
}

fn write_frame(stream: &mut UnixStream, payload: &[u8]) -> Result<(), &'static str> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err("GATEWAY_FRAME_TOO_LARGE");
    }
    let length = u32::try_from(payload.len()).map_err(|_| "GATEWAY_FRAME_TOO_LARGE")?;
    stream
        .write_all(&length.to_be_bytes())
        .and_then(|()| stream.write_all(payload))
        .map_err(|_| "GATEWAY_HANDSHAKE_FAILED")
}

fn read_frame(stream: &mut UnixStream) -> Result<Zeroizing<Vec<u8>>, &'static str> {
    let mut length = [0_u8; 4];
    stream
        .read_exact(&mut length)
        .map_err(|_| "GATEWAY_HANDSHAKE_FAILED")?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME_BYTES {
        return Err("GATEWAY_FRAME_TOO_LARGE");
    }
    let mut payload = Zeroizing::new(vec![0_u8; length]);
    stream
        .read_exact(&mut payload)
        .map_err(|_| "GATEWAY_HANDSHAKE_FAILED")?;
    Ok(payload)
}

fn validate_ready(frame: &[u8], expected_credential: &str) -> Result<(), &'static str> {
    let mut message: serde_json::Value =
        serde_json::from_slice(frame).map_err(|_| "GATEWAY_HANDSHAKE_FAILED")?;
    let object = message
        .as_object_mut()
        .filter(|object| object.len() == 3)
        .ok_or("GATEWAY_HANDSHAKE_FAILED")?;
    let credential = match object.remove("sessionCredential") {
        Some(serde_json::Value::String(value)) => Zeroizing::new(value),
        _ => return Err("GATEWAY_AUTH_FAILED"),
    };
    if object.get("kind").and_then(serde_json::Value::as_str) != Some("ready") {
        return Err("GATEWAY_HANDSHAKE_FAILED");
    }
    if object
        .get("protocolVersion")
        .and_then(serde_json::Value::as_u64)
        != Some(PROTOCOL_VERSION)
    {
        return Err("GATEWAY_PROTOCOL_MISMATCH");
    }
    if credential.as_str() != expected_credential {
        return Err("GATEWAY_AUTH_FAILED");
    }
    Ok(())
}

fn send_authenticated(
    channel: &mut UnixStream,
    credential: &str,
    kind: &str,
    mut payload: Value,
) -> Result<(), &'static str> {
    let object = payload.as_object_mut().ok_or("GATEWAY_PROTOCOL_INVALID")?;
    if object.contains_key("kind")
        || object.contains_key("protocolVersion")
        || object.contains_key("sessionCredential")
    {
        return Err("GATEWAY_PROTOCOL_INVALID");
    }
    object.insert("kind".into(), Value::String(kind.into()));
    object.insert("protocolVersion".into(), Value::from(PROTOCOL_VERSION));
    object.insert("sessionCredential".into(), Value::String(credential.into()));
    let frame =
        Zeroizing::new(serde_json::to_vec(&payload).map_err(|_| "GATEWAY_PROTOCOL_INVALID")?);
    write_frame(channel, &frame)
}

fn read_authenticated(
    channel: &mut UnixStream,
    expected_credential: &str,
) -> Result<Value, &'static str> {
    let frame = read_frame(channel)?;
    let mut message: Value =
        serde_json::from_slice(&frame).map_err(|_| "GATEWAY_PROTOCOL_INVALID")?;
    let object = message
        .as_object_mut()
        .filter(|object| object.len() >= 3)
        .ok_or("GATEWAY_PROTOCOL_INVALID")?;
    let credential = match object.remove("sessionCredential") {
        Some(Value::String(value)) => Zeroizing::new(value),
        _ => return Err("GATEWAY_AUTH_FAILED"),
    };
    if credential.as_str() != expected_credential {
        return Err("GATEWAY_AUTH_FAILED");
    }
    if object.get("protocolVersion").and_then(Value::as_u64) != Some(PROTOCOL_VERSION) {
        return Err("GATEWAY_PROTOCOL_MISMATCH");
    }
    object.remove("protocolVersion");
    Ok(message)
}

fn request_kind(value: &Value) -> Option<&str> {
    value.get("kind").and_then(Value::as_str)
}

fn exact_keys(value: &Value, expected: &[&str]) -> Result<(), ()> {
    let Some(object) = value.as_object() else {
        return Err(());
    };
    (object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key)))
        .then_some(())
        .ok_or(())
}

fn valid_identity(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn apply_gateway_cooldown(
    message: &Value,
    provider: &'static str,
    account_id: &str,
) -> Result<(), &'static str> {
    match message.get("retryAfterSeconds") {
        Some(Value::Null) => Ok(()),
        Some(value) => {
            let seconds = value.as_u64().ok_or("GATEWAY_PROTOCOL_INVALID")?;
            crate::provider_io::record_provider_retry_after(
                provider,
                if provider == "binance" {
                    None
                } else {
                    Some(account_id)
                },
                Some(seconds),
            );
            Ok(())
        }
        None => Err("GATEWAY_PROTOCOL_INVALID"),
    }
}

#[test]
fn gateway_cooldown_is_validated_and_shared_with_parent_requests() {
    assert_eq!(
        apply_gateway_cooldown(
            &json!({"retryAfterSeconds":"secret"}),
            "gateway-cooldown-test",
            "a"
        ),
        Err("GATEWAY_PROTOCOL_INVALID")
    );
    assert_eq!(
        apply_gateway_cooldown(
            &json!({"retryAfterSeconds":null}),
            "gateway-cooldown-test",
            "a"
        ),
        Ok(())
    );
    apply_gateway_cooldown(
        &json!({"retryAfterSeconds":2}),
        "gateway-cooldown-test",
        "a",
    )
    .unwrap();
    assert!(
        crate::provider_io::provider_retry_after_seconds("gateway-cooldown-test", "a")
            .is_some_and(|seconds| seconds >= 1)
    );
    assert_eq!(
        crate::provider_io::provider_retry_after_seconds("gateway-cooldown-test", "b"),
        None
    );
}

#[test]
fn busy_gateway_admission_returns_backpressure_without_waiting() {
    let host = std::sync::Arc::new(std::sync::Mutex::new(OrderGatewayHost::new(
        PathBuf::new(),
        String::new(),
    )));
    let _occupied = host.lock().unwrap();
    let waiting_host = std::sync::Arc::clone(&host);
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        tx.send(OrderGatewayHost::try_acquire(&waiting_host).err())
            .unwrap();
    });
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(1)).unwrap(),
        Some("PROVIDER_BACKPRESSURE")
    );
    worker.join().unwrap();
}

fn safe_gateway_error(code: &str) -> &'static str {
    match code {
        "CREDENTIAL_UNAVAILABLE"
        | "PROVIDER_UNAVAILABLE"
        | "PROVIDER_AUTH_FAILED"
        | "PROVIDER_PERMISSION_BLOCKED"
        | "PROVIDER_RATE_LIMITED"
        | "PROVIDER_BACKPRESSURE"
        | "PROVIDER_REVIEW_REQUIRED"
        | "PROVIDER_IDENTITY_CHANGED"
        | "PROVIDER_UNSUPPORTED"
        | "ORDER_PROPOSAL_NOT_ELIGIBLE"
        | "ORDER_AMOUNT_INVALID"
        | "ORDER_LIMIT_PRICE_REQUIRED"
        | "ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED"
        | "ORDER_VENUE_INVALID"
        | "ORDER_CAPABILITY_UNSUPPORTED"
        | "ORDER_CHANGED_REVIEW_AGAIN"
        | "STATE_VERSION_CONFLICT"
        | "CLOCK_SKEW"
        | "RISK_EVIDENCE_UNAVAILABLE" => match code {
            "CREDENTIAL_UNAVAILABLE" => "CREDENTIAL_UNAVAILABLE",
            "PROVIDER_UNAVAILABLE" => "PROVIDER_UNAVAILABLE",
            "PROVIDER_AUTH_FAILED" => "PROVIDER_AUTH_FAILED",
            "PROVIDER_PERMISSION_BLOCKED" => "PROVIDER_PERMISSION_BLOCKED",
            "PROVIDER_RATE_LIMITED" => "PROVIDER_RATE_LIMITED",
            "PROVIDER_BACKPRESSURE" => "PROVIDER_BACKPRESSURE",
            "PROVIDER_REVIEW_REQUIRED" => "PROVIDER_REVIEW_REQUIRED",
            "PROVIDER_IDENTITY_CHANGED" => "PROVIDER_IDENTITY_CHANGED",
            "PROVIDER_UNSUPPORTED" => "PROVIDER_UNSUPPORTED",
            "ORDER_PROPOSAL_NOT_ELIGIBLE" => "ORDER_PROPOSAL_NOT_ELIGIBLE",
            "ORDER_AMOUNT_INVALID" => "ORDER_AMOUNT_INVALID",
            "ORDER_LIMIT_PRICE_REQUIRED" => "ORDER_LIMIT_PRICE_REQUIRED",
            "ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED" => "ORDER_INSTRUMENT_PROVIDER_UNSUPPORTED",
            "ORDER_VENUE_INVALID" => "ORDER_VENUE_INVALID",
            "ORDER_CAPABILITY_UNSUPPORTED" => "ORDER_CAPABILITY_UNSUPPORTED",
            "ORDER_CHANGED_REVIEW_AGAIN" => "ORDER_CHANGED_REVIEW_AGAIN",
            "STATE_VERSION_CONFLICT" => "STATE_VERSION_CONFLICT",
            "CLOCK_SKEW" => "CLOCK_SKEW",
            _ => "RISK_EVIDENCE_UNAVAILABLE",
        },
        _ => "GATEWAY_PROCESS_FAILED",
    }
}

fn valid_outcome(intent: &GatewayDispatchIntent, outcome: &LiveDispatchOutcome) -> bool {
    match (&intent, outcome.state) {
        (GatewayDispatchIntent::Place(_), ExecutionAttemptState::Accepted) => {
            outcome
                .broker_order_id
                .as_deref()
                .is_some_and(crate::provider_io::valid_t212_order_id)
                && outcome.error_code.is_none()
        }
        (GatewayDispatchIntent::Cancel(intent), ExecutionAttemptState::CancelPending) => {
            outcome.broker_order_id.as_deref() == Some(intent.provider_order_id.as_str())
                && outcome.provider_status.as_deref() == Some(intent.provider_status.as_str())
                && outcome.error_code.is_none()
        }
        (_, ExecutionAttemptState::Rejected) => {
            outcome.error_code.as_deref().is_some_and(|code| {
                matches!(
                    code,
                    "PROVIDER_ORDER_REJECTED"
                        | "PROVIDER_AUTH_FAILED"
                        | "PROVIDER_PERMISSION_BLOCKED"
                        | "PROVIDER_RATE_LIMITED"
                )
            }) && (matches!(intent, GatewayDispatchIntent::Place(_))
                && outcome.broker_order_id.is_none()
                || matches!(intent, GatewayDispatchIntent::Cancel(cancel)
                    if outcome.broker_order_id.as_deref() == Some(cancel.provider_order_id.as_str())))
        }
        (_, ExecutionAttemptState::UnknownReconciling) => {
            outcome.broker_order_id.is_none()
                && outcome.provider_status.is_none()
                && outcome.error_code.as_deref() == Some("ORDER_STATUS_UNKNOWN")
        }
        _ => false,
    }
}

fn stop_child(child: &mut Child) {
    if child.try_wait().is_ok_and(|status| status.is_none()) {
        let _ = child.kill();
    }
    let _ = child.wait();
}

#[cfg(test)]
mod outcome_tests {
    use super::{GatewayDispatchIntent, valid_outcome};
    use crate::{
        protocol::{CancellationIntent, ExecutionAttemptState, ExecutionContext},
        provider_io::LiveDispatchOutcome,
    };

    fn cancel_intent() -> GatewayDispatchIntent {
        GatewayDispatchIntent::Cancel(Box::new(CancellationIntent {
            cancellation_intent_id: "cancel:1".into(),
            intent_hash: format!("sha256:{}", "a".repeat(64)),
            workspace_id: "workspace".into(),
            account_id: "account".into(),
            environment: ExecutionContext::Trading212Live,
            provider_order_id: "123".into(),
            instrument_id: "equity:US:MSFT".into(),
            symbol: "MSFT_US_EQ".into(),
            side: "BUY".into(),
            provider_status: "PARTIALLY_FILLED".into(),
            quantity: "1".into(),
            filled_quantity: "0.25".into(),
            remaining_quantity: "0.75".into(),
            created_at: "2026-09-28T10:00:00Z".into(),
        }))
    }

    #[test]
    fn cancel_ack_requires_the_preflight_provider_status() {
        let intent = cancel_intent();
        let outcome = |provider_status: &str| LiveDispatchOutcome {
            state: ExecutionAttemptState::CancelPending,
            broker_order_id: Some("123".into()),
            provider_status: Some(provider_status.into()),
            error_code: None,
        };

        assert!(valid_outcome(&intent, &outcome("PARTIALLY_FILLED")));
        assert!(!valid_outcome(&intent, &outcome("CANCEL_PENDING")));
    }
}
