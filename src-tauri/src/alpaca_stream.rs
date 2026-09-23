use crate::{
    ControlPlane,
    protocol::TradeXError,
    provider_io::{
        BrokerHttp, CredentialVault, NativeVault, ProviderHttp, alpaca_headers,
        verify_alpaca_paper_account,
    },
    providers::AccountConnection,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io,
    net::TcpStream,
    sync::mpsc::{RecvTimeoutError, SyncSender, sync_channel},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tungstenite::{
    Error as WebSocketError, Message, WebSocket, protocol::WebSocketConfig, stream::MaybeTlsStream,
};
use zeroize::Zeroizing;

const STREAM_URL: &str = "wss://paper-api.alpaca.markets/stream";
const MAX_STREAM_MESSAGE: usize = 256 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(1);
const ACK_TIMEOUT: Duration = Duration::from_secs(10);
const PING_INTERVAL: Duration = Duration::from_secs(30);
const PONG_TIMEOUT: Duration = Duration::from_secs(10);

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;
type Workers = HashMap<String, Worker>;

struct Worker {
    state_version: String,
    remote_account_id: String,
    cancel: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

#[derive(Default)]
struct State {
    paused: bool,
    workers: Workers,
}

#[derive(Clone, Default)]
pub struct AlpacaPrivateStreamSupervisor(Arc<Mutex<State>>);

impl AlpacaPrivateStreamSupervisor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pause(&self) {
        if let Ok(mut state) = self.0.lock() {
            state.paused = true;
            for worker in state.workers.values() {
                worker.cancel.store(true, Ordering::Release);
            }
        }
    }

    pub fn resume(&self) {
        if let Ok(mut state) = self.0.lock() {
            state.paused = false;
        }
    }

    pub fn restart_all(&self) {
        if let Ok(mut state) = self.0.lock() {
            for worker in state.workers.values() {
                worker.cancel.store(true, Ordering::Release);
            }
            state.paused = false;
        }
    }

    pub fn stop_all(&self) {
        self.pause();
    }

    pub fn sync(&self, engine: Arc<Mutex<ControlPlane>>) {
        self.sync_with(engine, run_account_stream);
    }

    fn sync_with(
        &self,
        engine: Arc<Mutex<ControlPlane>>,
        worker: impl Fn(Arc<Mutex<ControlPlane>>, AccountConnection, Arc<AtomicBool>)
        + Send
        + Sync
        + 'static,
    ) {
        let worker = Arc::new(worker);
        let accounts = match engine.lock() {
            Ok(control) => control.alpaca_private_stream_accounts().unwrap_or_default(),
            Err(_) => return,
        };
        let desired: HashMap<_, _> = accounts
            .into_iter()
            .filter_map(|account| {
                let remote_account_id = account.data.as_ref()?.remote_account_id.clone();
                Some((account.connection_id.clone(), (account, remote_account_id)))
            })
            .collect();
        let mut state = match self.0.lock() {
            Ok(state) => state,
            Err(_) => return,
        };
        if state.paused {
            return;
        }
        state
            .workers
            .retain(|_, worker| !worker.thread.is_finished());
        for (connection_id, worker) in &state.workers {
            let keep = desired.get(connection_id).is_some_and(|(account, remote)| {
                account.state_version == worker.state_version && remote == &worker.remote_account_id
            });
            if !keep {
                worker.cancel.store(true, Ordering::Release);
            }
        }
        for (connection_id, (account, remote_account_id)) in desired {
            if state.workers.contains_key(&connection_id) {
                continue;
            }
            let cancel = Arc::new(AtomicBool::new(false));
            let worker_cancel = cancel.clone();
            let worker_engine = engine.clone();
            let worker_runner = worker.clone();
            let state_version = account.state_version.clone();
            let Ok(worker_thread) = thread::Builder::new()
                .name(format!("alpaca-paper-{}", connection_id))
                .spawn(move || worker_runner(worker_engine, account, worker_cancel))
            else {
                continue;
            };
            state.workers.insert(
                connection_id,
                Worker {
                    state_version,
                    remote_account_id,
                    cancel: cancel.clone(),
                    thread: worker_thread,
                },
            );
        }
    }
}

fn run_account_stream(
    engine: Arc<Mutex<ControlPlane>>,
    account: AccountConnection,
    cancel: Arc<AtomicBool>,
) {
    run_account_stream_with(
        engine,
        account,
        cancel,
        &NativeVault,
        &BrokerHttp::default(),
        connect_stream,
    );
}

fn run_account_stream_with(
    engine: Arc<Mutex<ControlPlane>>,
    mut account: AccountConnection,
    cancel: Arc<AtomicBool>,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    connect: impl Fn(
        &AccountConnection,
        &AtomicBool,
    ) -> Result<(Socket, Zeroizing<Vec<String>>), TradeXError>,
) {
    let mut retry = Duration::from_secs(1);
    loop {
        if cancelled(&cancel) || !current_account(&engine, &account) {
            return;
        }
        if account.health.connection != "ONLINE"
            || account.health.authentication != "VALID"
            || account.health.credential != "CONFIGURED"
        {
            match refresh_account(&engine, &account, &cancel, vault, http) {
                Ok(refreshed) => {
                    if refreshed.state_version != account.state_version {
                        return;
                    }
                    account = refreshed;
                }
                Err(code) => {
                    update_health(
                        &engine,
                        &account,
                        "DEGRADED",
                        "DEGRADED",
                        &format!(
                            "Account recovery is waiting for Alpaca Paper ({}).",
                            code.code
                        ),
                    );
                    if !wait_retry(&cancel, retry) {
                        return;
                    }
                    retry = next_retry(retry);
                    continue;
                }
            }
        }
        if cancelled(&cancel) || !current_account(&engine, &account) {
            return;
        }
        update_health(
            &engine,
            &account,
            "CONNECTING",
            "REQUIRED",
            "Connecting to the fixed Alpaca Paper private stream.",
        );
        let (socket, secrets) = match connect(&account, &cancel) {
            Ok(value) => value,
            Err(error) => {
                let stream = if error.code == "PROVIDER_AUTH_FAILED" {
                    "AUTH_FAILED"
                } else {
                    "DEGRADED"
                };
                mark_degraded(&engine, &account, stream, &error.code);
                if !wait_retry(&cancel, retry) {
                    return;
                }
                retry = next_retry(retry);
                continue;
            }
        };
        retry = Duration::from_secs(1);
        update_health(
            &engine,
            &account,
            "CONNECTED",
            "RUNNING",
            "Alpaca Paper trade_updates is connected; REST reconciliation is running.",
        );
        let reconciliation = reconcile_order_book(&engine, &account, &cancel, vault, http);
        let reconciliation_status = if reconciliation == "CURRENT" {
            "CURRENT"
        } else {
            "DEGRADED"
        };
        update_health(
            &engine,
            &account,
            "CONNECTED",
            reconciliation_status,
            if reconciliation_status == "CURRENT" {
                "Alpaca Paper private stream is connected and REST reconciliation completed."
            } else {
                "Alpaca Paper private stream is connected; REST reconciliation is incomplete."
            },
        );
        let (sender, receiver) = sync_channel(32);
        let reader_cancel = cancel.clone();
        let reader = match thread::Builder::new()
            .name(format!("alpaca-paper-read-{}", account.connection_id))
            .spawn(move || read_stream(socket, reader_cancel, sender, PING_INTERVAL, PONG_TIMEOUT))
        {
            Ok(reader) => reader,
            Err(_) => {
                mark_degraded(&engine, &account, "DEGRADED", "PROVIDER_UNAVAILABLE");
                if !wait_retry(&cancel, retry) {
                    return;
                }
                retry = next_retry(retry);
                continue;
            }
        };
        let mut reconnect = false;
        while !cancelled(&cancel) {
            match receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(Ok(value)) => match value.get("stream").and_then(Value::as_str) {
                    Some("trade_updates") => {
                        let current = engine.lock().is_ok_and(|mut control| {
                            control
                                .apply_alpaca_private_stream_frame(
                                    &account.connection_id,
                                    &account.state_version,
                                    &account.data.as_ref().unwrap().remote_account_id,
                                    &value,
                                    &secrets,
                                )
                                .is_ok()
                        });
                        if !current {
                            if !current_account(&engine, &account) {
                                return;
                            }
                            mark_degraded(
                                &engine,
                                &account,
                                "DEGRADED",
                                "PROVIDER_RESPONSE_INVALID",
                            );
                            reconnect = true;
                            break;
                        }
                    }
                    Some("error") => {
                        let code = if value
                            .to_string()
                            .to_ascii_lowercase()
                            .contains("unauthorized")
                        {
                            "PROVIDER_AUTH_FAILED"
                        } else {
                            "PROVIDER_UNAVAILABLE"
                        };
                        mark_degraded(
                            &engine,
                            &account,
                            if code == "PROVIDER_AUTH_FAILED" {
                                "AUTH_FAILED"
                            } else {
                                "DEGRADED"
                            },
                            code,
                        );
                        reconnect = true;
                        break;
                    }
                    _ => (),
                },
                Ok(Err(code)) => {
                    mark_degraded(&engine, &account, "DEGRADED", &code);
                    reconnect = true;
                    break;
                }
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => {
                    if !cancelled(&cancel) {
                        mark_degraded(&engine, &account, "DEGRADED", "PROVIDER_UNAVAILABLE");
                        reconnect = true;
                    }
                    break;
                }
            }
        }
        drop(receiver);
        let _ = reader.join();
        if cancelled(&cancel) || !current_account(&engine, &account) {
            return;
        }
        if reconnect {
            if !wait_retry(&cancel, retry) {
                return;
            }
            retry = next_retry(retry);
        }
    }
}

fn refresh_account(
    engine: &Arc<Mutex<ControlPlane>>,
    account: &AccountConnection,
    cancel: &AtomicBool,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Result<AccountConnection, TradeXError> {
    let request = json!({
        "requestId": uuid::Uuid::new_v4().to_string(),
        "schemaVersion": 1,
        "command": "account.refresh",
        "payload": {
            "workspaceId": account.workspace_id,
            "connectionId": account.connection_id,
            "expectedStateVersion": account.state_version
        }
    });
    let job = engine
        .lock()
        .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
        .prepare_provider_for(&request, "main")?
        .ok_or_else(|| TradeXError::new("IPC_COMMAND_UNKNOWN"))?;
    let outcome = job.run(
        vault,
        |_| Err(TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED")),
        http,
        || {
            !cancel.load(Ordering::Acquire)
                && engine
                    .lock()
                    .is_ok_and(|control| control.provider_job_current(&job))
        },
    );
    if let Some(error) = outcome.error.as_ref()
        && error.code == "PROVIDER_IDENTITY_CHANGED"
    {
        let reply = engine
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
            .complete_provider(&job, outcome);
        return Err(reply["error"]["code"]
            .as_str()
            .map(TradeXError::new)
            .unwrap_or_else(|| TradeXError::new("PROVIDER_IDENTITY_CHANGED")));
    }
    if let Some(error) = outcome.error.as_ref() {
        return Err(TradeXError::new(&error.code));
    }
    let reply = engine
        .lock()
        .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
        .complete_provider(&job, outcome);
    if reply["ok"] != true {
        return Err(reply["error"]["code"]
            .as_str()
            .map(TradeXError::new)
            .unwrap_or_else(|| TradeXError::new("PROVIDER_UNAVAILABLE")));
    }
    serde_json::from_value(reply["data"].clone())
        .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))
}

fn connect_stream(
    account: &AccountConnection,
    cancel: &AtomicBool,
) -> Result<(Socket, Zeroizing<Vec<String>>), TradeXError> {
    let secret = NativeVault.get(&account.credential_ref())?;
    let values = secret.values()?;
    if values.len() != 2 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let remote_account_id = account
        .data
        .as_ref()
        .map(|data| data.remote_account_id.as_str())
        .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
    let http = BrokerHttp::default();
    let auth = alpaca_headers(&values)?;
    verify_alpaca_paper_account(&http, &auth, remote_account_id, &values)?;
    if cancel.load(Ordering::Acquire) {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_STREAM_MESSAGE))
        .max_frame_size(Some(MAX_STREAM_MESSAGE));
    let (mut socket, _) = tungstenite::client::connect_with_config(STREAM_URL, Some(config), 0)
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    let read_timeout = match socket.get_mut() {
        MaybeTlsStream::NativeTls(stream) => stream.get_mut().set_read_timeout(Some(READ_TIMEOUT)),
        _ => Err(io::Error::other("unexpected Alpaca TLS transport")),
    };
    read_timeout.map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    authenticate_and_subscribe(&mut socket, &values, cancel)?;
    Ok((socket, values))
}

fn authenticate_and_subscribe(
    socket: &mut Socket,
    credentials: &[String],
    cancel: &AtomicBool,
) -> Result<(), TradeXError> {
    if credentials.len() != 2 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let authentication = Zeroizing::new(
        json!({"action":"auth", "key":credentials[0], "secret":credentials[1]}).to_string(),
    );
    socket
        .send(Message::text(authentication.as_str()))
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    wait_for_ack(socket, cancel, Ack::Authorized)?;
    let subscription = Zeroizing::new(
        "{\"action\":\"listen\",\"data\":{\"streams\":[\"trade_updates\"]}}".to_owned(),
    );
    socket
        .send(Message::text(subscription.as_str()))
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    wait_for_ack(socket, cancel, Ack::TradeUpdates)?;
    Ok(())
}

#[derive(Clone, Copy)]
enum Ack {
    Authorized,
    TradeUpdates,
}

enum StreamRead {
    Json(Value),
    Pong,
    PeerActivity,
    Idle,
}

fn wait_for_ack(
    socket: &mut Socket,
    cancel: &AtomicBool,
    expected: Ack,
) -> Result<(), TradeXError> {
    let deadline = Instant::now() + ACK_TIMEOUT;
    while Instant::now() < deadline {
        if cancel.load(Ordering::Acquire) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let value = match read_message(socket, cancel).map_err(|code| TradeXError::new(&code))? {
            StreamRead::Json(value) => value,
            StreamRead::Pong | StreamRead::PeerActivity | StreamRead::Idle => continue,
        };
        let items = value.as_array().cloned().unwrap_or_else(|| vec![value]);
        for item in items {
            let stream = item
                .get("stream")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let data = item.get("data").unwrap_or(&Value::Null);
            if stream == "error" {
                return Err(TradeXError::new("PROVIDER_AUTH_FAILED"));
            }
            match expected {
                Ack::Authorized if stream == "authorization" => {
                    return if data.get("status").and_then(Value::as_str) == Some("authorized") {
                        Ok(())
                    } else {
                        Err(TradeXError::new("PROVIDER_AUTH_FAILED"))
                    };
                }
                Ack::TradeUpdates if stream == "listening" => {
                    let subscribed =
                        data.get("streams")
                            .and_then(Value::as_array)
                            .is_some_and(|streams| {
                                streams
                                    .iter()
                                    .any(|name| name.as_str() == Some("trade_updates"))
                            });
                    return if subscribed {
                        Ok(())
                    } else {
                        Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"))
                    };
                }
                _ => (),
            }
        }
    }
    Err(TradeXError::new("PROVIDER_UNAVAILABLE"))
}

fn read_message(socket: &mut Socket, cancel: &AtomicBool) -> Result<StreamRead, String> {
    if cancel.load(Ordering::Acquire) {
        return Ok(StreamRead::Idle);
    }
    match socket.read() {
        Ok(Message::Text(text)) => parse_stream_json(text.as_bytes()).map(StreamRead::Json),
        Ok(Message::Binary(bytes)) => parse_stream_json(bytes.as_ref()).map(StreamRead::Json),
        Ok(Message::Ping(bytes)) => socket
            .send(Message::Pong(bytes))
            .map(|()| StreamRead::PeerActivity)
            .map_err(|_| "PROVIDER_UNAVAILABLE".into()),
        Ok(Message::Pong(_)) => Ok(StreamRead::Pong),
        Ok(Message::Frame(_)) => Ok(StreamRead::PeerActivity),
        Ok(Message::Close(_)) => Err("PROVIDER_UNAVAILABLE".into()),
        Err(WebSocketError::Io(error))
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
            ) =>
        {
            Ok(StreamRead::Idle)
        }
        Err(_) => Err("PROVIDER_UNAVAILABLE".into()),
    }
}

fn read_stream(
    mut socket: Socket,
    cancel: Arc<AtomicBool>,
    sender: SyncSender<Result<Value, String>>,
    ping_interval: Duration,
    pong_timeout: Duration,
) {
    let mut next_ping = Instant::now() + ping_interval;
    let mut ping_sent_at = None;
    while !cancelled(&cancel) {
        match read_message(&mut socket, &cancel) {
            Ok(StreamRead::Json(value)) => {
                ping_sent_at = None;
                next_ping = Instant::now() + ping_interval;
                if sender.send(Ok(value)).is_err() {
                    return;
                }
            }
            Ok(StreamRead::Pong | StreamRead::PeerActivity) => {
                ping_sent_at = None;
                next_ping = Instant::now() + ping_interval;
            }
            Ok(StreamRead::Idle) => {
                let now = Instant::now();
                if ping_sent_at.is_some_and(|sent_at| now.duration_since(sent_at) >= pong_timeout) {
                    let _ = sender.send(Err("PROVIDER_UNAVAILABLE".into()));
                    return;
                }
                if ping_sent_at.is_none() && now >= next_ping {
                    if socket.send(Message::Ping(Vec::new().into())).is_err() {
                        let _ = sender.send(Err("PROVIDER_UNAVAILABLE".into()));
                        return;
                    }
                    ping_sent_at = Some(now);
                }
            }
            Err(error) => {
                let _ = sender.send(Err(error));
                return;
            }
        }
    }
}

fn parse_stream_json(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > MAX_STREAM_MESSAGE {
        return Err("PROVIDER_RESPONSE_INVALID".into());
    }
    serde_json::from_slice(bytes).map_err(|_| "PROVIDER_RESPONSE_INVALID".into())
}

fn reconcile_order_book(
    engine: &Arc<Mutex<ControlPlane>>,
    account: &AccountConnection,
    cancel: &AtomicBool,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> String {
    let request = json!({
        "requestId": uuid::Uuid::new_v4().to_string(),
        "schemaVersion": 1,
        "command": "alpaca.paper.orders.refresh",
        "payload": {
            "workspaceId": account.workspace_id,
            "connectionId": account.connection_id,
            "expectedConnectionStateVersion": account.state_version
        }
    });
    let job = match engine.lock() {
        Ok(mut control) => match control.prepare_provider_for(&request, "main") {
            Ok(Some(job)) => job,
            _ => return "DEGRADED".into(),
        },
        Err(_) => return "DEGRADED".into(),
    };
    let outcome = job.run(
        vault,
        |_| Err(TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED")),
        http,
        || {
            !cancel.load(Ordering::Acquire)
                && engine
                    .lock()
                    .is_ok_and(|control| control.provider_job_current(&job))
        },
    );
    let reply = match engine.lock() {
        Ok(mut control) => control.complete_provider(&job, outcome),
        Err(_) => return "DEGRADED".into(),
    };
    if reply["ok"] == true && reply["data"]["status"] == "CURRENT" {
        "CURRENT".into()
    } else {
        "DEGRADED".into()
    }
}

fn update_health(
    engine: &Arc<Mutex<ControlPlane>>,
    account: &AccountConnection,
    private_stream: &str,
    reconciliation: &str,
    reason: &str,
) {
    if let Ok(mut control) = engine.lock() {
        let _ = control.update_alpaca_private_stream_health(
            &account.connection_id,
            &account.state_version,
            private_stream,
            reconciliation,
            reason,
        );
    }
}

fn mark_degraded(
    engine: &Arc<Mutex<ControlPlane>>,
    account: &AccountConnection,
    state: &str,
    code: &str,
) {
    if let Ok(mut control) = engine.lock() {
        let reason = format!(
            "Alpaca Paper private stream needs recovery ({code}); saved orders are not current."
        );
        let _ = control.mark_alpaca_private_stream_degraded(
            &account.connection_id,
            &account.state_version,
            state,
            &reason,
        );
    }
}

fn current_account(engine: &Arc<Mutex<ControlPlane>>, account: &AccountConnection) -> bool {
    engine.lock().is_ok_and(|control| {
        control
            .alpaca_private_stream_accounts()
            .is_ok_and(|accounts| {
                accounts.iter().any(|current| {
                    current.connection_id == account.connection_id
                        && current.state_version == account.state_version
                        && current
                            .data
                            .as_ref()
                            .map(|data| data.remote_account_id.as_str())
                            == account
                                .data
                                .as_ref()
                                .map(|data| data.remote_account_id.as_str())
                })
            })
    })
}

fn cancelled(cancel: &AtomicBool) -> bool {
    cancel.load(Ordering::Acquire)
}

fn wait_retry(cancel: &AtomicBool, delay: Duration) -> bool {
    let end = Instant::now() + delay;
    while Instant::now() < end {
        if cancelled(cancel) {
            return false;
        }
        thread::sleep(Duration::from_millis(100));
    }
    !cancelled(cancel)
}

fn next_retry(current: Duration) -> Duration {
    (current * 2).min(Duration::from_secs(30))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[allow(dead_code)]
    mod fixtures {
        include!("../tests/support/provider_fixtures.rs");
    }

    #[derive(Clone)]
    struct SharedVault(Arc<Mutex<fixtures::Vault>>);

    impl CredentialVault for SharedVault {
        fn put(
            &self,
            reference: &str,
            credentials: &crate::provider_io::Credentials,
        ) -> crate::protocol::Result<()> {
            self.0
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
                .put(reference, credentials)
        }

        fn get(&self, reference: &str) -> crate::protocol::Result<crate::provider_io::Credentials> {
            self.0
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
                .get(reference)
        }

        fn remove(&self, reference: &str) -> crate::protocol::Result<()> {
            self.0
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
                .remove(reference)
        }
    }

    #[derive(Clone)]
    struct SharedHttp(Arc<Mutex<fixtures::Http>>);

    impl ProviderHttp for SharedHttp {
        fn get(
            &self,
            endpoint: crate::provider_io::ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> crate::protocol::Result<Vec<u8>> {
            self.0
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
                .get(endpoint, path, headers)
        }

        fn request(
            &self,
            endpoint: crate::provider_io::ProviderEndpoint,
            method: crate::provider_io::ProviderHttpMethod,
            path: &str,
            headers: reqwest::header::HeaderMap,
            body: Option<&Value>,
        ) -> crate::protocol::Result<crate::provider_io::ProviderHttpResponse> {
            self.0
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
                .request(endpoint, method, path, headers, body)
        }
    }

    fn local_worker(
        vault: SharedVault,
        http: SharedHttp,
        address: String,
        connections: Arc<std::sync::atomic::AtomicUsize>,
    ) -> impl Fn(Arc<Mutex<ControlPlane>>, AccountConnection, Arc<AtomicBool>) + Send + Sync + 'static
    {
        move |engine, account, cancel| {
            let vault = vault.clone();
            let http = http.clone();
            let connector_vault = vault.clone();
            let address = address.clone();
            let connections = connections.clone();
            run_account_stream_with(
                engine,
                account,
                cancel,
                &vault,
                &http,
                move |account, cancel| {
                    let secret = connector_vault.get(&account.credential_ref())?;
                    let values = secret.values()?;
                    let (mut socket, _) = tungstenite::client::connect(format!("ws://{address}"))
                        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
                        stream
                            .set_read_timeout(Some(READ_TIMEOUT))
                            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                    }
                    authenticate_and_subscribe(&mut socket, &values, cancel)?;
                    connections.fetch_add(1, Ordering::AcqRel);
                    Ok((socket, values))
                },
            );
        }
    }

    #[test]
    fn stream_authenticates_and_subscribes_only_to_trade_updates() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            server_stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut socket = tungstenite::accept(server_stream).unwrap();
            let auth: Value =
                serde_json::from_str(socket.read().unwrap().into_text().unwrap().as_str()).unwrap();
            assert_eq!(auth["action"], "auth");
            assert_eq!(auth["key"], "fixture-key");
            assert_eq!(auth["secret"], "fixture-secret");
            socket
                .send(Message::text(
                    r#"{"stream":"authorization","data":{"status":"authorized"}}"#,
                ))
                .unwrap();
            let subscription: Value =
                serde_json::from_str(socket.read().unwrap().into_text().unwrap().as_str()).unwrap();
            assert_eq!(subscription["action"], "listen");
            assert_eq!(subscription["data"]["streams"], json!(["trade_updates"]));
            socket
                .send(Message::text(
                    r#"{"stream":"listening","data":{"streams":["trade_updates"]}}"#,
                ))
                .unwrap();
        });
        let (mut socket, _) = tungstenite::client::connect(format!("ws://{address}")).unwrap();
        let credentials = vec!["fixture-key".into(), "fixture-secret".into()];
        authenticate_and_subscribe(&mut socket, &credentials, &AtomicBool::new(false)).unwrap();
        server.join().unwrap();
    }

    #[test]
    fn stream_rejects_an_unverified_authentication_ack() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            server_stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut socket = tungstenite::accept(server_stream).unwrap();
            let _ = socket.read().unwrap();
            socket
                .send(Message::text(
                    r#"{"stream":"authorization","data":{"status":"unauthorized"}}"#,
                ))
                .unwrap();
        });
        let (mut socket, _) = tungstenite::client::connect(format!("ws://{address}")).unwrap();
        let credentials = vec!["fixture-key".into(), "fixture-secret".into()];
        let error = authenticate_and_subscribe(&mut socket, &credentials, &AtomicBool::new(false))
            .unwrap_err();
        assert_eq!(error.code, "PROVIDER_AUTH_FAILED");
        server.join().unwrap();
    }

    #[test]
    fn stream_reader_reports_a_missing_pong_as_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            server_stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut socket = tungstenite::accept(server_stream).unwrap();
            assert!(matches!(socket.read().unwrap(), Message::Ping(_)));
            thread::sleep(Duration::from_millis(200));
        });
        let (mut socket, _) = tungstenite::client::connect(format!("ws://{address}")).unwrap();
        match socket.get_mut() {
            MaybeTlsStream::Plain(stream) => stream
                .set_read_timeout(Some(Duration::from_millis(10)))
                .unwrap(),
            _ => unreachable!(),
        }
        let (sender, receiver) = sync_channel(1);
        let reader = thread::spawn(move || {
            read_stream(
                socket,
                Arc::new(AtomicBool::new(false)),
                sender,
                Duration::from_millis(10),
                Duration::from_millis(50),
            )
        });

        assert_eq!(
            receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
            Err("PROVIDER_UNAVAILABLE".into())
        );
        reader.join().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn account_worker_reconnects_and_reconciles_after_private_stream_disconnect() {
        use fixtures::{Http, Vault, credentials};

        fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
            control.dispatch(json!({
                "requestId":"alpaca-worker-test",
                "schemaVersion":1,
                "command":name,
                "payload":payload
            }))
        }

        fn execute(
            control: &mut ControlPlane,
            request: Value,
            vault: &impl CredentialVault,
            http: &impl ProviderHttp,
        ) -> Value {
            let job = control.prepare_provider(&request).unwrap().unwrap();
            let outcome = job.run(
                vault,
                |_| credentials(),
                http,
                || control.provider_job_current(&job),
            );
            control.complete_provider(&job, outcome)
        }

        let folder = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let tested = execute(
            &mut control,
            json!({
                "requestId":"alpaca-worker-test",
                "schemaVersion":1,
                "command":"provider.connect",
                "payload":{"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Paper worker test"}
            }),
            &vault,
            &http,
        );
        assert_eq!(tested["ok"], true, "{tested}");
        let pending = &tested["data"];
        let confirmed = command(
            &mut control,
            "provider.connect",
            json!({
                "step":"confirm",
                "workspaceId":workspace,
                "connectionId":pending["connectionId"],
                "expectedStateVersion":pending["stateVersion"],
                "acknowledgeUnverified":true
            }),
        );
        assert_eq!(confirmed["ok"], true, "{confirmed}");
        let account: AccountConnection = serde_json::from_value(confirmed["data"].clone()).unwrap();
        let connection_id = account.connection_id.clone();
        let remote_account_id = account.data.as_ref().unwrap().remote_account_id.clone();
        let order_id = "00000000-0000-4000-8000-000000000001";
        let execution_id = order_id;
        let timestamp = "2026-09-23T10:00:00Z";
        let order = json!({
            "id":order_id,"client_order_id":"manual-1","symbol":"AAPL",
            "asset_class":"us_equity","side":"buy","type":"limit",
            "time_in_force":"day","qty":"1","filled_qty":"0.5",
            "status":"partially_filled","submitted_at":timestamp,"created_at":timestamp,
            "updated_at":timestamp
        });
        http.alpaca_order_history.borrow_mut().push(order.clone());
        http.alpaca_fills.borrow_mut().push(json!({
            "id":format!("20260923100000000::{execution_id}"),
            "order_id":order_id,"symbol":"AAPL","side":"buy","qty":"0.5",
            "price":"10.25","transaction_time":timestamp
        }));
        let frame = json!({
            "stream":"trade_updates",
            "data":{
                "event":"partial_fill","execution_id":execution_id,
                "qty":"0.5","price":"10.25","timestamp":timestamp,
                "order":{
                    "id":order_id,"account_id":remote_account_id,
                    "client_order_id":"manual-1","symbol":"AAPL",
                    "side":"buy","type":"limit","time_in_force":"day",
                    "status":"partially_filled","qty":"1","filled_qty":"0.5",
                    "submitted_at":timestamp,"updated_at":timestamp
                }
            }
        });

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server_frame = frame.clone();
        let server_connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let server_ready = server_connections.clone();
        let watcher_server_ready = server_connections.clone();
        let server = thread::spawn(move || {
            for index in 0..3 {
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(4)))
                    .unwrap();
                let mut socket = tungstenite::accept(stream).unwrap();
                let auth: Value =
                    serde_json::from_str(socket.read().unwrap().into_text().unwrap().as_str())
                        .unwrap();
                assert_eq!(auth["action"], "auth");
                socket
                    .send(Message::text(
                        r#"{"stream":"authorization","data":{"status":"authorized"}}"#,
                    ))
                    .unwrap();
                let subscription: Value =
                    serde_json::from_str(socket.read().unwrap().into_text().unwrap().as_str())
                        .unwrap();
                assert_eq!(subscription["data"]["streams"], json!(["trade_updates"]));
                socket
                    .send(Message::text(
                        r#"{"stream":"listening","data":{"streams":["trade_updates"]}}"#,
                    ))
                    .unwrap();
                server_ready.fetch_add(1, Ordering::AcqRel);
                if index == 0 {
                    socket
                        .send(Message::text(server_frame.to_string()))
                        .unwrap();
                    socket.send(Message::Close(None)).unwrap();
                } else {
                    loop {
                        match socket.read() {
                            Ok(Message::Close(_)) | Err(_) => break,
                            _ => (),
                        }
                    }
                }
            }
        });

        let engine = Arc::new(Mutex::new(control));
        let shared_vault = SharedVault(Arc::new(Mutex::new(vault)));
        let shared_http = SharedHttp(Arc::new(Mutex::new(http)));
        let supervisor = AlpacaPrivateStreamSupervisor::new();
        let connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let worker_started = Arc::new(AtomicBool::new(false));
        let watcher_engine = engine.clone();
        let watcher_connections = connections.clone();
        let watcher_supervisor = supervisor.clone();
        let watcher_worker_started = worker_started.clone();
        let watcher = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(8);
            loop {
                if !watcher_worker_started.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                watcher_supervisor.sync(watcher_engine.clone());
                let recovered = watcher_engine
                    .lock()
                    .ok()
                    .and_then(|control| control.alpaca_private_stream_accounts().ok())
                    .and_then(|accounts| {
                        accounts
                            .into_iter()
                            .find(|item| item.connection_id == connection_id)
                    });
                if watcher_connections.load(Ordering::Acquire) >= 2
                    && watcher_server_ready.load(Ordering::Acquire) >= 2
                    && recovered.is_some_and(|item| {
                        item.health.private_stream == "CONNECTED"
                            && item.health.reconciliation == "CURRENT"
                            && item.last_private_stream_event_at.is_some()
                    })
                {
                    watcher_supervisor.pause();
                    return true;
                }
                if Instant::now() >= deadline {
                    watcher_supervisor.pause();
                    return false;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        supervisor.sync_with(
            engine.clone(),
            local_worker(
                shared_vault.clone(),
                shared_http.clone(),
                address.to_string(),
                connections.clone(),
            ),
        );
        worker_started.store(true, Ordering::Release);
        assert!(
            watcher.join().unwrap(),
            "worker did not reconnect and reconcile"
        );
        let workers = supervisor
            .0
            .lock()
            .unwrap()
            .workers
            .drain()
            .map(|(_, worker)| worker.thread)
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }

        supervisor.restart_all();
        supervisor.sync_with(
            engine.clone(),
            local_worker(
                shared_vault.clone(),
                shared_http.clone(),
                address.to_string(),
                connections.clone(),
            ),
        );
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut resumed = false;
        while Instant::now() < deadline {
            supervisor.sync(engine.clone());
            let recovered = engine
                .lock()
                .ok()
                .and_then(|control| control.alpaca_private_stream_accounts().ok())
                .and_then(|accounts| {
                    accounts
                        .into_iter()
                        .find(|item| item.connection_id == account.connection_id)
                });
            if connections.load(Ordering::Acquire) >= 3
                && server_connections.load(Ordering::Acquire) >= 3
                && recovered.is_some_and(|item| {
                    item.health.private_stream == "CONNECTED"
                        && item.health.reconciliation == "CURRENT"
                })
            {
                resumed = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        supervisor.pause();
        assert!(resumed, "worker did not restart and reconcile after resume");
        let workers = supervisor
            .0
            .lock()
            .unwrap()
            .workers
            .drain()
            .map(|(_, worker)| worker.thread)
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
        server.join().unwrap();

        let recovered = engine
            .lock()
            .unwrap()
            .alpaca_private_stream_accounts()
            .unwrap()
            .into_iter()
            .find(|item| item.connection_id == account.connection_id)
            .unwrap();
        assert_eq!(recovered.health.private_stream, "CONNECTED");
        assert_eq!(recovered.health.reconciliation, "CURRENT");
        assert!(recovered.last_private_stream_event_at.is_some());
        assert_eq!(connections.load(Ordering::Acquire), 3);
        let book = engine.lock().unwrap().dispatch(json!({
            "requestId":"alpaca-worker-test",
            "schemaVersion":1,
            "command":"alpaca.paper.orders.get",
            "payload":{"workspaceId":account.workspace_id,"connectionId":account.connection_id}
        }));
        assert_eq!(book["data"]["book"]["fills"].as_array().unwrap().len(), 1);
        assert_eq!(book["data"]["book"]["fills"][0]["source"], "REST_ACTIVITY");
        assert_eq!(
            shared_http
                .0
                .lock()
                .unwrap()
                .calls
                .borrow()
                .iter()
                .filter(|path| path.contains("/v2/orders?status=all"))
                .count(),
            3
        );
        assert_eq!(shared_http.0.lock().unwrap().alpaca_posts.borrow().len(), 0);
    }
}
