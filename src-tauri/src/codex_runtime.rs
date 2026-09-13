use crate::protocol::{Result, ThreadModel, TradeXError};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

const MAX_FRAME_BYTES: usize = 65_536;
const DEFAULT_TIMEOUT_SECS: u64 = 20;
const LOOPBACK_ENDPOINT: &str = "http://127.0.0.1:8317/v1";

#[derive(Clone, Debug)]
pub struct RuntimeRequest {
    pub codex_thread_id: Option<String>,
    pub model: ThreadModel,
    pub message: String,
}

pub struct RuntimeAccess {
    api_key: Zeroizing<String>,
}

impl RuntimeAccess {
    pub(crate) fn for_gateway(api_key: &str) -> Option<Self> {
        (!api_key.is_empty()).then(|| Self {
            api_key: Zeroizing::new(api_key.to_owned()),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    ThreadStarted {
        key: String,
        thread_id: String,
    },
    TurnStarted {
        key: String,
    },
    ItemStarted {
        key: String,
        item_id: String,
        item_type: String,
    },
    ItemDelta {
        key: String,
        item_id: String,
        item_type: String,
        delta: String,
    },
    ItemCompleted {
        key: String,
        item_id: String,
        item_type: String,
        content: Option<String>,
    },
    TurnCompleted {
        key: String,
    },
}

#[derive(Clone, Debug)]
pub struct AppServerConfig {
    executable: PathBuf,
    timeout: Duration,
}

impl AppServerConfig {
    pub fn from_environment() -> Self {
        let timeout = env::var("TRADEX_CODEX_APP_SERVER_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|seconds| (1..=300).contains(seconds))
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        Self {
            executable: env::var_os("TRADEX_CODEX_APP_SERVER")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("codex")),
            timeout: Duration::from_secs(timeout),
        }
    }

    pub fn available() -> bool {
        if cfg!(feature = "integration-test") && env::var_os("TRADEX_CODEX_APP_SERVER").is_none() {
            return true;
        }
        let config = Self::from_environment();
        Command::new(config.executable)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}

pub fn run(
    request: &RuntimeRequest,
    access: Option<&RuntimeAccess>,
    emit: &mut impl FnMut(RuntimeEvent) -> Result<()>,
) -> Result<()> {
    if cfg!(feature = "integration-test") && env::var_os("TRADEX_CODEX_APP_SERVER").is_none() {
        return run_fake(request, emit);
    }
    let access = access.ok_or_else(|| TradeXError::new("CODEX_RUNTIME_NOT_CONFIGURED"))?;
    run_process(&AppServerConfig::from_environment(), request, access, emit)
}

fn run_fake(
    request: &RuntimeRequest,
    emit: &mut impl FnMut(RuntimeEvent) -> Result<()>,
) -> Result<()> {
    let scenario = env::var("TRADEX_FAKE_APP_SERVER_SCENARIO").unwrap_or_default();
    if scenario == "malformed" {
        return Err(TradeXError::new("CODEX_FRAME_INVALID"));
    }
    if scenario == "unsupported" {
        return Err(TradeXError::new("CODEX_PROTOCOL_UNSUPPORTED"));
    }
    let codex_thread_id = request
        .codex_thread_id
        .clone()
        .unwrap_or_else(|| format!("codex-thread-{}", &uuid::Uuid::new_v4().to_string()[..8]));
    let frames = [
        RuntimeEvent::ThreadStarted {
            key: "thread/started:1".into(),
            thread_id: codex_thread_id,
        },
        RuntimeEvent::TurnStarted {
            key: "turn/started:1".into(),
        },
        RuntimeEvent::ItemStarted {
            key: "item/started:1".into(),
            item_id: "item-agent-1".into(),
            item_type: "agent_message".into(),
        },
        RuntimeEvent::ItemDelta {
            key: "item/delta:1".into(),
            item_id: "item-agent-1".into(),
            item_type: "agent_message".into(),
            delta: format!("Read-only response for: {}", request.message),
        },
        RuntimeEvent::ItemCompleted {
            key: "item/completed:1".into(),
            item_id: "item-agent-1".into(),
            item_type: "agent_message".into(),
            content: None,
        },
        RuntimeEvent::TurnCompleted {
            key: "turn/completed:1".into(),
        },
    ];
    for (index, event) in frames.into_iter().enumerate() {
        if scenario == "duplicate" && index == 4 {
            emit(event.clone())?;
        }
        if scenario == "gap" && index == 3 {
            return Err(TradeXError::new("CODEX_EVENT_GAP"));
        }
        if scenario == "outage" && index == 3 {
            return Err(TradeXError::new("MODEL_UNAVAILABLE"));
        }
        emit(event)?;
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

fn run_process(
    config: &AppServerConfig,
    request: &RuntimeRequest,
    access: &RuntimeAccess,
    emit: &mut impl FnMut(RuntimeEvent) -> Result<()>,
) -> Result<()> {
    let mut process = Process::spawn(config, access)?;
    let mut id = 1_u64;
    process.send(json!({
        "id": id,
        "method": "initialize",
        "params": {
            "clientInfo": {"name": "tradex", "title": "TradeX", "version": "0.1.0"}
        }
    }))?;
    process.response(id)?;
    process.send(json!({"method": "initialized", "params": {}}))?;

    id += 1;
    let method = if request.codex_thread_id.is_some() {
        "thread/resume"
    } else {
        "thread/start"
    };
    let params = if let Some(thread_id) = &request.codex_thread_id {
        json!({"threadId": thread_id})
    } else {
        json!({
            "model": request.model.model_id,
            "approvalPolicy": "never",
            "sandbox": "read-only"
        })
    };
    process.send(json!({"id": id, "method": method, "params": params}))?;
    let thread_response = process.response(id)?;
    let thread_id = thread_id_from_response(&thread_response)
        .ok_or_else(|| TradeXError::new("CODEX_PROTOCOL_UNSUPPORTED"))?;
    let turn_thread_id = thread_id.clone();
    emit(RuntimeEvent::ThreadStarted {
        key: format!("thread/started:{thread_id}"),
        thread_id: thread_id.clone(),
    })?;

    id += 1;
    let mut turn_params = json!({
        "threadId": turn_thread_id,
        "input": [{"type": "text", "text": request.message}],
        "model": request.model.model_id,
        "approvalPolicy": "never",
        "sandboxPolicy": {"type": "readOnly", "networkAccess": false}
    });
    if request.model.provider == "DEEPSEEK"
        && let Some(thinking_type) = request.model.thinking_type.as_deref()
    {
        turn_params["effort"] = json!(match thinking_type {
            "disabled" => "none",
            "enabled" => "high",
            _ => return Err(TradeXError::new("CODEX_PROTOCOL_UNSUPPORTED")),
        });
    }
    process.send(json!({
        "id": id,
        "method": "turn/start",
        "params": turn_params
    }))?;
    let mut turn_id = None;
    let mut last_upstream_sequence = None;
    let mut seen_sequences = std::collections::HashSet::new();
    loop {
        let frame = process.next()?;
        if frame.get("id").and_then(Value::as_u64) == Some(id) {
            response_error(&frame)?;
            turn_id = turn_id_from_response(&frame);
            continue;
        }
        if let Some(sequence) = upstream_sequence(&frame) {
            if !seen_sequences.contains(&sequence) {
                if last_upstream_sequence.is_some_and(|previous| sequence != previous + 1) {
                    return Err(TradeXError::new("CODEX_EVENT_GAP"));
                }
                seen_sequences.insert(sequence);
                last_upstream_sequence = Some(sequence);
            }
        }
        validate_notification_scope(&frame, &thread_id, turn_id.as_deref())?;
        if let Some(event) = notification(&frame)? {
            let approval = matches!(
                &event,
                RuntimeEvent::ItemStarted { item_type, .. } if item_type == "codex_approval"
            );
            let completed = matches!(event, RuntimeEvent::TurnCompleted { .. });
            emit(event)?;
            if approval {
                return Err(TradeXError::new("CODEX_UPSTREAM_ERROR"));
            }
            if completed {
                break;
            }
        }
    }
    if turn_id.is_none() {
        return Err(TradeXError::new("CODEX_PROTOCOL_UNSUPPORTED"));
    }
    process.finish();
    Ok(())
}

struct Process {
    child: Child,
    stdin: ChildStdin,
    frames: Receiver<std::result::Result<String, String>>,
    deadline: Instant,
    runtime_home: PathBuf,
}

impl Process {
    fn spawn(config: &AppServerConfig, access: &RuntimeAccess) -> Result<Self> {
        let runtime_home = env::temp_dir().join(format!("tradex-codex-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&runtime_home)
            .map_err(|_| TradeXError::new("CODEX_RUNTIME_START_FAILED"))?;
        #[cfg(unix)]
        if fs::set_permissions(
            &runtime_home,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        )
        .is_err()
        {
            let _ = fs::remove_dir(&runtime_home);
            return Err(TradeXError::new("CODEX_RUNTIME_START_FAILED"));
        }
        let path = env::var_os("PATH").unwrap_or_default();
        let config_override = format!("openai_base_url=\"{LOOPBACK_ENDPOINT}\"");
        let mut command = Command::new(&config.executable);
        let mut child = command
            .args(["-c", config_override.as_str(), "app-server", "--stdio"])
            .env_clear()
            .env("PATH", path)
            .env("HOME", &runtime_home)
            .env("CODEX_HOME", &runtime_home)
            .env("TMPDIR", &runtime_home)
            .env("OPENAI_API_KEY", access.api_key.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| {
                let _ = fs::remove_dir_all(&runtime_home);
                TradeXError::new("CODEX_RUNTIME_START_FAILED")
            })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            let _ = fs::remove_dir_all(&runtime_home);
            TradeXError::new("CODEX_RUNTIME_START_FAILED")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            let _ = fs::remove_dir_all(&runtime_home);
            TradeXError::new("CODEX_RUNTIME_START_FAILED")
        })?;
        let (sender, frames) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = Vec::new();
                match reader.read_until(b'\n', &mut line) {
                    Ok(0) => break,
                    Ok(_) if line.len() > MAX_FRAME_BYTES => {
                        let _ = sender.send(Err("oversized".into()));
                        break;
                    }
                    Ok(_) => {
                        let line = String::from_utf8_lossy(&line).trim().to_owned();
                        if !line.is_empty() && sender.send(Ok(line)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = sender.send(Err(error.to_string()));
                        break;
                    }
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            frames,
            deadline: Instant::now() + config.timeout,
            runtime_home,
        })
    }

    fn send(&mut self, frame: Value) -> Result<()> {
        serde_json::to_writer(&mut self.stdin, &frame)
            .map_err(|_| TradeXError::new("CODEX_RUNTIME_START_FAILED"))?;
        self.stdin
            .write_all(b"\n")
            .and_then(|_| self.stdin.flush())
            .map_err(|_| TradeXError::new("CODEX_RUNTIME_START_FAILED"))
    }

    fn next(&mut self) -> Result<Value> {
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(TradeXError::new("CODEX_RUNTIME_TIMEOUT"));
        }
        let line = self
            .frames
            .recv_timeout(remaining)
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => TradeXError::new("CODEX_RUNTIME_TIMEOUT"),
                mpsc::RecvTimeoutError::Disconnected => TradeXError::new("CODEX_PROCESS_EXITED"),
            })?
            .map_err(|_| TradeXError::new("CODEX_FRAME_INVALID"))?;
        let frame: Value =
            serde_json::from_str(&line).map_err(|_| TradeXError::new("CODEX_FRAME_INVALID"))?;
        if !frame.is_object() {
            return Err(TradeXError::new("CODEX_FRAME_INVALID"));
        }
        if frame.get("jsonrpc").is_some()
            && frame.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        {
            return Err(TradeXError::new("CODEX_PROTOCOL_UNSUPPORTED"));
        }
        Ok(frame)
    }

    fn response(&mut self, id: u64) -> Result<Value> {
        loop {
            let frame = self.next()?;
            if frame.get("id").and_then(Value::as_u64) == Some(id) {
                response_error(&frame)?;
                return Ok(frame);
            }
            if frame.get("method").is_some() {
                continue;
            }
            return Err(TradeXError::new("CODEX_PROTOCOL_UNSUPPORTED"));
        }
    }

    fn finish(&mut self) {
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.runtime_home);
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.runtime_home);
    }
}

fn response_error(frame: &Value) -> Result<()> {
    if frame.get("error").is_some() {
        return Err(TradeXError::new("CODEX_UPSTREAM_ERROR"));
    }
    if frame.get("result").is_none() {
        return Err(TradeXError::new("CODEX_PROTOCOL_UNSUPPORTED"));
    }
    Ok(())
}

fn thread_id_from_response(frame: &Value) -> Option<String> {
    frame["result"]["thread"]["id"]
        .as_str()
        .or_else(|| frame["result"]["threadId"].as_str())
        .or_else(|| frame["result"]["id"].as_str())
        .map(str::to_owned)
}

fn turn_id_from_response(frame: &Value) -> Option<String> {
    frame["result"]["turn"]["id"]
        .as_str()
        .or_else(|| frame["result"]["turnId"].as_str())
        .or_else(|| frame["result"]["id"].as_str())
        .map(str::to_owned)
}

fn validate_notification_scope(
    frame: &Value,
    expected_thread_id: &str,
    expected_turn_id: Option<&str>,
) -> Result<()> {
    let Some(params) = frame.get("params") else {
        return Ok(());
    };
    let thread_id = string_field(params, &["threadId"]).or_else(|| {
        params
            .get("thread")
            .and_then(|value| string_field(value, &["id"]))
    });
    if thread_id.is_some_and(|id| id != expected_thread_id) {
        return Err(TradeXError::new("CODEX_FRAME_INVALID"));
    }
    let turn_id = string_field(params, &["turnId"]).or_else(|| {
        params
            .get("turn")
            .and_then(|value| string_field(value, &["id"]))
    });
    if turn_id.is_some_and(|id| expected_turn_id.is_some_and(|expected| id != expected)) {
        return Err(TradeXError::new("CODEX_FRAME_INVALID"));
    }
    Ok(())
}

fn notification(frame: &Value) -> Result<Option<RuntimeEvent>> {
    let Some(method) = frame.get("method").and_then(Value::as_str) else {
        return Ok(None);
    };
    let params = frame.get("params").unwrap_or(&Value::Null);
    let frame_key =
        serde_json::to_string(frame).map_err(|_| TradeXError::new("CODEX_FRAME_INVALID"))?;
    let item = params.get("item").unwrap_or(params);
    let item_id =
        string_field(item, &["id", "itemId"]).or_else(|| string_field(params, &["id", "itemId"]));
    let item_type = string_field(item, &["type", "itemType"])
        .or_else(|| string_field(params, &["type", "itemType"]))
        .unwrap_or_else(|| "compatibility".into())
        .replace('/', "_");
    let item_type = normalize_item_type(&item_type);
    match method {
        "thread/started" => string_field(params, &["threadId", "id"])
            .or_else(|| {
                params
                    .get("thread")
                    .and_then(|thread| string_field(thread, &["threadId", "id"]))
            })
            .or_else(|| string_field(item, &["threadId", "id"]))
            .map(|thread_id| {
                Some(RuntimeEvent::ThreadStarted {
                    key: format!("thread/started:{thread_id}"),
                    thread_id,
                })
            })
            .ok_or_else(|| TradeXError::new("CODEX_FRAME_INVALID")),
        "turn/started" => Ok(Some(RuntimeEvent::TurnStarted { key: frame_key })),
        "turn/completed" => {
            if let Some(status) = string_field(params, &["status"]).or_else(|| {
                params
                    .get("turn")
                    .and_then(|turn| string_field(turn, &["status"]))
            }) && !matches!(
                status.to_ascii_lowercase().as_str(),
                "completed" | "complete" | "success" | "succeeded"
            ) {
                return Err(TradeXError::new("CODEX_UPSTREAM_ERROR"));
            }
            Ok(Some(RuntimeEvent::TurnCompleted { key: frame_key }))
        }
        "item/started" => Ok(Some(RuntimeEvent::ItemStarted {
            key: frame_key,
            item_id: item_id.ok_or_else(|| TradeXError::new("CODEX_FRAME_INVALID"))?,
            item_type,
        })),
        "item/completed" => Ok(Some(RuntimeEvent::ItemCompleted {
            key: frame_key,
            item_id: item_id.ok_or_else(|| TradeXError::new("CODEX_FRAME_INVALID"))?,
            item_type,
            content: text_field(item, &["text", "content"]),
        })),
        method if method.starts_with("item/") && method.ends_with("/delta") => {
            Ok(Some(RuntimeEvent::ItemDelta {
                key: frame_key,
                item_id: item_id.ok_or_else(|| TradeXError::new("CODEX_FRAME_INVALID"))?,
                item_type,
                delta: text_field(params, &["delta", "text", "content"]).unwrap_or_default(),
            }))
        }
        method if method.ends_with("/error") || method == "error" => {
            Err(TradeXError::new("CODEX_UPSTREAM_ERROR"))
        }
        method if method.to_ascii_lowercase().contains("approval") => {
            Ok(Some(RuntimeEvent::ItemStarted {
                key: frame_key,
                item_id: string_field(params, &["approvalId", "requestId", "id"])
                    .unwrap_or_else(|| "codex-approval".into()),
                item_type: "codex_approval".into(),
            }))
        }
        _ => Ok(None),
    }
}

fn string_field(value: &Value, fields: &[&str]) -> Option<String> {
    fields
        .iter()
        .find_map(|field| value.get(*field).and_then(Value::as_str).map(str::to_owned))
}

fn text_field(value: &Value, fields: &[&str]) -> Option<String> {
    string_field(value, fields).or_else(|| {
        value
            .get("content")
            .and_then(|content| content.get("text"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    })
}

fn normalize_item_type(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for (index, character) in value.chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            normalized.push('_');
        }
        normalized.push(character.to_ascii_lowercase());
    }
    normalized
}

fn upstream_sequence(frame: &Value) -> Option<u64> {
    frame
        .get("params")
        .and_then(|params| params.get("sequence").or_else(|| params.get("seq")))
        .and_then(Value::as_u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typed_stream_notifications_and_preserves_unknown_item_type() {
        let started = notification(
            &json!({"method":"item/started","params":{"item":{"id":"i1","type":"mystery"}}}),
        )
        .unwrap();
        assert_eq!(
            started,
            Some(RuntimeEvent::ItemStarted {
                key: r#"{"method":"item/started","params":{"item":{"id":"i1","type":"mystery"}}}"#
                    .into(),
                item_id: "i1".into(),
                item_type: "mystery".into()
            })
        );
        let delta = notification(
            &json!({"method":"item/agentMessage/delta","params":{"itemId":"i1","delta":"hello"}}),
        )
        .unwrap();
        assert!(
            matches!(delta, Some(RuntimeEvent::ItemDelta { item_id, delta, .. }) if item_id == "i1" && delta == "hello")
        );
        assert!(notification(&json!({"method":"item/agentMessage/error","params":{}})).is_err());
    }

    #[test]
    fn rejects_bad_response_and_unsupported_jsonrpc_version() {
        assert!(response_error(&json!({"id":1,"result":{}})).is_ok());
        assert_eq!(
            response_error(&json!({"id":1,"error":{}}))
                .unwrap_err()
                .code,
            "CODEX_UPSTREAM_ERROR"
        );
        assert_eq!(
            serde_json::from_str::<Value>(r#"{"jsonrpc":"1.0"}"#).unwrap()["jsonrpc"],
            "1.0"
        );
    }

    #[test]
    fn rejects_non_successful_turn_completion() {
        let error =
            notification(&json!({"method":"turn/completed","params":{"turn":{"status":"failed"}}}))
                .unwrap_err();
        assert_eq!(error.code, "CODEX_UPSTREAM_ERROR");
    }

    #[test]
    fn normalizes_upstream_item_types_and_reads_nested_thread_ids() {
        let started = notification(&json!({
            "method":"thread/started",
            "params":{"thread":{"id":"thread-1"}}
        }))
        .unwrap();
        assert!(
            matches!(started, Some(RuntimeEvent::ThreadStarted { thread_id, .. }) if thread_id == "thread-1")
        );
        let item = notification(&json!({
            "method":"item/started",
            "params":{"item":{"id":"item-1","type":"agentMessage"}}
        }))
        .unwrap();
        assert!(
            matches!(item, Some(RuntimeEvent::ItemStarted { item_type, .. }) if item_type == "agent_message")
        );
    }

    #[test]
    fn approval_requests_are_distinct_from_financial_actions() {
        let event = notification(&json!({
            "method": "item/commandExecution/requestApproval",
            "params": {"approvalId": "approval-1", "itemId": "item-1"}
        }))
        .unwrap();
        assert!(matches!(
            event,
            Some(RuntimeEvent::ItemStarted { item_id, item_type, .. })
                if item_id == "approval-1" && item_type == "codex_approval"
        ));
    }

    #[test]
    fn rejects_events_for_a_different_thread_or_turn() {
        let error = validate_notification_scope(
            &json!({"params":{"threadId":"other","turnId":"turn-1"}}),
            "thread-1",
            Some("turn-1"),
        )
        .unwrap_err();
        assert_eq!(error.code, "CODEX_FRAME_INVALID");
        let error =
            notification(&json!({"method":"item/started","params":{"item":{}}})).unwrap_err();
        assert_eq!(error.code, "CODEX_FRAME_INVALID");
    }

    #[cfg(unix)]
    #[test]
    fn process_adapter_uses_one_bounded_jsonl_session() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("fake-codex");
        fs::write(
            &executable,
            r#"#!/bin/sh
initialize_count=0
thread_count=0
turn_count=0
while IFS= read -r line; do
  [ "${OPENAI_API_KEY:-}" = "test-key" ] || exit 10
  case "$line" in
    *'"method":"initialize"'*)
      initialize_count=$((initialize_count + 1)); [ "$initialize_count" -eq 1 ] || exit 11
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{}}' ;;
    *'"method":"thread/start"'*)
      thread_count=$((thread_count + 1)); [ "$thread_count" -eq 1 ] || exit 12
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"threadId":"fake-thread"}}' ;;
    *'"method":"turn/start"'*)
      turn_count=$((turn_count + 1)); [ "$turn_count" -eq 1 ] || exit 13
      case "$line" in *'"sandboxPolicy"'*) ;; *) exit 14 ;; esac
      case "$line" in *'"type":"readOnly"'*) ;; *) exit 15 ;; esac
      case "$line" in *'"effort":"high"'*) ;; *) exit 16 ;; esac
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"turnId":"fake-turn"}}'
      printf '%s\n' '{"method":"turn/started","params":{}}'
      printf '%s\n' '{"method":"item/started","params":{"item":{"id":"item-1","type":"agent_message"}}}'
      printf '%s\n' '{"method":"item/agent_message/delta","params":{"itemId":"item-1","delta":"OK"}}'
      printf '%s\n' '{"method":"item/completed","params":{"item":{"id":"item-1","type":"agent_message","text":"OK"}}}'
      printf '%s\n' '{"method":"turn/completed","params":{}}' ;;
  esac
done
"#,
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        let config = AppServerConfig {
            executable,
            timeout: Duration::from_secs(3),
        };
        let access = RuntimeAccess::for_gateway("test-key").unwrap();
        let request = RuntimeRequest {
            codex_thread_id: None,
            model: ThreadModel {
                provider: "DEEPSEEK".into(),
                model_id: "deepseek-v4-flash".into(),
                thinking_type: Some("enabled".into()),
            },
            message: "hello".into(),
        };
        let mut events = Vec::new();
        run_process(&config, &request, &access, &mut |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();
        assert!(
            matches!(events.first(), Some(RuntimeEvent::ThreadStarted { thread_id, .. }) if thread_id == "fake-thread")
        );
        assert!(matches!(
            events.last(),
            Some(RuntimeEvent::TurnCompleted { .. })
        ));
        assert!(
            events.iter().any(
                |event| matches!(event, RuntimeEvent::ItemDelta { delta, .. } if delta == "OK")
            )
        );
    }
}
