use crate::{
    ControlPlane,
    protocol::TradeXError,
    provider_io::{
        BrokerHttp, CredentialVault, NativeVault, ProviderHttp,
        binance_testnet_private_stream_subscription, reconcile_binance_testnet_private_stream,
        verify_binance_testnet_private_stream_account,
    },
    providers::AccountConnection,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    io,
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{RecvTimeoutError, SyncSender, sync_channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tungstenite::{
    Error as WebSocketError, Message, WebSocket, protocol::WebSocketConfig, stream::MaybeTlsStream,
};
use zeroize::Zeroizing;

const STREAM_URL: &str = "wss://ws-api.testnet.binance.vision/ws-api/v3";
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
pub struct BinancePrivateStreamSupervisor(Arc<Mutex<State>>);

impl BinancePrivateStreamSupervisor {
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
            Ok(control) => control
                .binance_private_stream_accounts()
                .unwrap_or_default(),
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
        for (connection_id, active) in &state.workers {
            let keep = desired.get(connection_id).is_some_and(|(account, remote)| {
                account.state_version == active.state_version && remote == &active.remote_account_id
            });
            if !keep {
                active.cancel.store(true, Ordering::Release);
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
                .name(format!("binance-testnet-{connection_id}"))
                .spawn(move || worker_runner(worker_engine, account, worker_cancel))
            else {
                continue;
            };
            state.workers.insert(
                connection_id,
                Worker {
                    state_version,
                    remote_account_id,
                    cancel,
                    thread: worker_thread,
                },
            );
        }
    }
}

pub fn mark_connected_accounts_degraded(engine: &Arc<Mutex<ControlPlane>>) {
    let Ok(mut control) = engine.lock() else {
        return;
    };
    let Ok(accounts) = control.binance_private_stream_accounts() else {
        return;
    };
    for account in accounts {
        let _ = control.update_binance_private_stream_health(
            &account.connection_id,
            &account.state_version,
            "DEGRADED",
            "DEGRADED",
            "Binance Spot Testnet private stream paused; REST reconciliation is required.",
        );
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
    account: AccountConnection,
    cancel: Arc<AtomicBool>,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    connect: impl Fn(
        &AccountConnection,
        &AtomicBool,
    ) -> Result<(Socket, Zeroizing<Vec<String>>, u64), TradeXError>,
) {
    let mut retry = Duration::from_secs(1);
    loop {
        if cancelled(&cancel) || !current_account(&engine, &account) {
            return;
        }
        update_health(
            &engine,
            &account,
            "CONNECTING",
            "REQUIRED",
            "Connecting to the fixed Binance Spot Testnet private stream.",
        );
        let (socket, secrets, subscription_id) = match connect(&account, &cancel) {
            Ok(value) => value,
            Err(error) => {
                let stream = if matches!(
                    error.code.as_str(),
                    "PROVIDER_AUTHENTICATION_FAILED" | "PROVIDER_AUTH_FAILED"
                ) {
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
            "Binance Spot Testnet private stream is connected; REST reconciliation is running.",
        );
        let (sender, receiver) = sync_channel(32);
        let reader_cancel = cancel.clone();
        let reader = match thread::Builder::new()
            .name(format!("binance-testnet-read-{}", account.connection_id))
            .spawn(move || read_stream(socket, reader_cancel, sender))
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
        let mut reconnect = !reconcile_order_book(&engine, &account, &cancel, vault, http);
        while !reconnect && !cancelled(&cancel) {
            match receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(Ok(frame)) => {
                    let outcome = engine.lock().map(|mut control| {
                        if cancelled(&cancel) {
                            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
                        }
                        control.apply_binance_private_stream_frame(
                            &account.connection_id,
                            &account.state_version,
                            &account.data.as_ref().unwrap().remote_account_id,
                            &frame,
                            subscription_id,
                            &secrets,
                        )
                    });
                    match outcome {
                        Ok(Ok(needs_reconciliation)) => {
                            if needs_reconciliation
                                && !reconcile_order_book(&engine, &account, &cancel, vault, http)
                            {
                                reconnect = true;
                            }
                        }
                        Ok(Err(error)) => {
                            if !current_account(&engine, &account) {
                                drop(secrets);
                                let _ = reader.join();
                                return;
                            }
                            mark_degraded(&engine, &account, "DEGRADED", &error.code);
                            reconnect = true;
                        }
                        Err(_) => {
                            mark_degraded(
                                &engine,
                                &account,
                                "DEGRADED",
                                "IPC_CONTROL_PLANE_UNAVAILABLE",
                            );
                            reconnect = true;
                        }
                    }
                }
                Ok(Err(code)) => {
                    mark_degraded(&engine, &account, "DEGRADED", &code);
                    reconnect = true;
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
        drop(secrets);
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

fn connect_stream(
    account: &AccountConnection,
    cancel: &AtomicBool,
) -> Result<(Socket, Zeroizing<Vec<String>>, u64), TradeXError> {
    let values = NativeVault.get(&account.credential_ref())?.values()?;
    let remote_account_id = account
        .data
        .as_ref()
        .map(|data| data.remote_account_id.as_str())
        .ok_or_else(|| TradeXError::new("PROVIDER_REVIEW_REQUIRED"))?;
    let http = BrokerHttp::default();
    let current = || !cancel.load(Ordering::Acquire);
    verify_binance_testnet_private_stream_account(&http, &values, remote_account_id, &current)?;
    if cancel.load(Ordering::Acquire) {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_STREAM_MESSAGE))
        .max_frame_size(Some(MAX_STREAM_MESSAGE));
    let (mut socket, _) = tungstenite::client::connect_with_config(STREAM_URL, Some(config), 0)
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    match socket.get_mut() {
        MaybeTlsStream::NativeTls(stream) => stream
            .get_mut()
            .set_read_timeout(Some(READ_TIMEOUT))
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?,
        _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
    }
    let subscription_id = authenticate_and_subscribe(&mut socket, &values, &http, cancel)?;
    Ok((socket, values, subscription_id))
}

fn authenticate_and_subscribe(
    socket: &mut Socket,
    secrets: &[String],
    http: &impl ProviderHttp,
    cancel: &AtomicBool,
) -> Result<u64, TradeXError> {
    let current = || !cancel.load(Ordering::Acquire);
    let (request_id, request) =
        binance_testnet_private_stream_subscription(http, secrets, &current)?;
    let request = Zeroizing::new(request.to_string());
    socket
        .send(Message::text(request.as_str()))
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    wait_for_subscription(socket, &request_id, cancel)
}

fn wait_for_subscription(
    socket: &mut Socket,
    request_id: &str,
    cancel: &AtomicBool,
) -> Result<u64, TradeXError> {
    let deadline = Instant::now() + ACK_TIMEOUT;
    while Instant::now() < deadline {
        if cancel.load(Ordering::Acquire) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let frame = match read_message(socket, cancel).map_err(|code| TradeXError::new(&code))? {
            StreamRead::Json(frame) => frame,
            StreamRead::Pong | StreamRead::PeerActivity | StreamRead::Idle => continue,
        };
        if frame.get("id").and_then(Value::as_str) != Some(request_id) {
            continue;
        }
        if frame["status"].as_u64() != Some(200) {
            return Err(TradeXError::new(
                if matches!(frame["status"].as_u64(), Some(401 | 403)) {
                    "PROVIDER_AUTHENTICATION_FAILED"
                } else {
                    "PROVIDER_RESPONSE_INVALID"
                },
            ));
        }
        return frame["result"]["subscriptionId"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or_else(|| TradeXError::new("PROVIDER_RESPONSE_INVALID"));
    }
    Err(TradeXError::new("PROVIDER_UNAVAILABLE"))
}

enum StreamRead {
    Json(Value),
    Pong,
    PeerActivity,
    Idle,
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
) {
    let mut next_ping = Instant::now() + PING_INTERVAL;
    let mut ping_sent_at = None;
    while !cancelled(&cancel) {
        match read_message(&mut socket, &cancel) {
            Ok(StreamRead::Json(frame)) => {
                ping_sent_at = None;
                next_ping = Instant::now() + PING_INTERVAL;
                if sender.send(Ok(frame)).is_err() {
                    return;
                }
            }
            Ok(StreamRead::Pong | StreamRead::PeerActivity) => {
                ping_sent_at = None;
                next_ping = Instant::now() + PING_INTERVAL;
            }
            Ok(StreamRead::Idle) => {
                let now = Instant::now();
                if ping_sent_at.is_some_and(|sent| now.duration_since(sent) >= PONG_TIMEOUT) {
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
) -> bool {
    if cancelled(cancel) || !current_account(engine, account) {
        return false;
    }
    let mut book = match engine.lock() {
        Ok(control) => match control.binance_private_stream_order_book(&account.connection_id) {
            Ok(book) => book,
            Err(_) => return false,
        },
        Err(_) => return false,
    };
    let secret = match vault
        .get(&account.credential_ref())
        .and_then(|secret| secret.values())
    {
        Ok(secret) if secret.len() == 2 => secret,
        _ => {
            book.status = crate::protocol::BinanceTestnetOrderBookStatus::Degraded;
            book.reason = Some("CREDENTIAL_UNAVAILABLE".into());
            mark_reconciliation_result(engine, account, book, "DEGRADED", "CREDENTIAL_UNAVAILABLE");
            return false;
        }
    };
    let current = || !cancelled(cancel) && current_account(engine, account);
    let result = reconcile_binance_testnet_private_stream(&mut book, &secret, http, &current);
    match result {
        Ok(()) => {
            mark_reconciliation_result(
                engine,
                account,
                book,
                "CURRENT",
                "Binance Spot Testnet private stream is connected and REST reconciliation completed.",
            );
            true
        }
        Err(error) => {
            let reconciliation = if error.code == "STATE_VERSION_CONFLICT" {
                "REQUIRED"
            } else {
                "DEGRADED"
            };
            mark_reconciliation_result(engine, account, book, reconciliation, &error.code);
            false
        }
    }
}

fn mark_reconciliation_result(
    engine: &Arc<Mutex<ControlPlane>>,
    account: &AccountConnection,
    book: crate::protocol::BinanceTestnetOrderBook,
    reconciliation: &str,
    reason: &str,
) {
    if let Ok(mut control) = engine.lock() {
        let _ = control.persist_binance_private_stream_state(
            &account.connection_id,
            &account.state_version,
            Some(book),
            "CONNECTED",
            reconciliation,
            reason,
            None,
        );
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
        let _ = control.update_binance_private_stream_health(
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
    private_stream: &str,
    code: &str,
) {
    let reason = format!(
        "Binance Spot Testnet private stream needs recovery ({code}); saved orders are not current."
    );
    update_health(engine, account, private_stream, "DEGRADED", &reason);
}

fn current_account(engine: &Arc<Mutex<ControlPlane>>, account: &AccountConnection) -> bool {
    let Some(remote_account_id) = account
        .data
        .as_ref()
        .map(|data| data.remote_account_id.as_str())
    else {
        return false;
    };
    engine.lock().is_ok_and(|control| {
        control.binance_private_stream_account_current(
            &account.connection_id,
            &account.state_version,
            remote_account_id,
        )
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
    use crate::{
        protocol::Result,
        provider_io::{
            CredentialVault, Credentials, ProviderEndpoint, ProviderHttpMethod,
            ProviderHttpResponse,
        },
        providers::{AccountConnection, AccountData, ConnectionState},
    };
    use hmac::{Hmac, Mac};
    use serde_json::json;
    use sha2::Sha256;
    use tungstenite::client::connect;

    const API_KEY: &str = "TESTNET-FIXTURE-KEY";
    const API_SECRET: &str = "TESTNET-FIXTURE-SECRET";

    #[test]
    fn signed_subscription_completes_over_a_loopback_websocket() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut socket = tungstenite::accept(stream).unwrap();
            let request = socket.read().unwrap().into_text().unwrap();
            let request: Value = serde_json::from_str(&request).unwrap();
            assert_eq!(request["method"], "userDataStream.subscribe.signature");
            assert_eq!(request["params"]["apiKey"], API_KEY);
            assert_eq!(request["params"]["recvWindow"], 5000);
            assert!(!request.to_string().contains(API_SECRET));

            let timestamp = request["params"]["timestamp"].as_u64().unwrap();
            let payload = format!("apiKey={API_KEY}&recvWindow=5000&timestamp={timestamp}");
            let mut mac = Hmac::<Sha256>::new_from_slice(API_SECRET.as_bytes()).unwrap();
            mac.update(payload.as_bytes());
            mac.verify_slice(
                &hex::decode(request["params"]["signature"].as_str().unwrap()).unwrap(),
            )
            .unwrap();

            socket
                .send(Message::text(
                    json!({"id":"another-request","status":200,"result":{"subscriptionId":1}})
                        .to_string(),
                ))
                .unwrap();
            socket
                .send(Message::text(
                    json!({"id":request["id"],"status":200,"result":{"subscriptionId":19}})
                        .to_string(),
                ))
                .unwrap();
        });

        let (mut socket, _) = connect(format!("ws://{address}")).unwrap();
        let cancel = AtomicBool::new(false);
        let subscription_id = authenticate_and_subscribe(
            &mut socket,
            &[API_KEY.into(), API_SECRET.into()],
            &StreamHttp,
            &cancel,
        )
        .unwrap();
        assert_eq!(subscription_id, 19);
        server.join().unwrap();
    }

    #[test]
    fn supervisor_backoff_is_bounded_and_doubles() {
        assert_eq!(next_retry(Duration::from_secs(1)), Duration::from_secs(2));
        assert_eq!(next_retry(Duration::from_secs(16)), Duration::from_secs(30));
        assert_eq!(next_retry(Duration::from_secs(30)), Duration::from_secs(30));
    }

    struct StreamVault;

    impl CredentialVault for StreamVault {
        fn put(&self, _: &str, _: &Credentials) -> Result<()> {
            Ok(())
        }

        fn get(&self, _: &str) -> Result<Credentials> {
            Credentials::new(vec![API_KEY.into(), API_SECRET.into()])
        }

        fn remove(&self, _: &str) -> Result<()> {
            Ok(())
        }
    }

    struct StreamHttp;

    impl ProviderHttp for StreamHttp {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> Result<Vec<u8>> {
            assert_eq!(endpoint, ProviderEndpoint::BinanceTestnet);
            assert!(headers.is_empty());
            assert_eq!(path, "/api/v3/time");
            Ok(br#"{"serverTime":1788849600000}"#.to_vec())
        }

        fn request(
            &self,
            endpoint: ProviderEndpoint,
            method: ProviderHttpMethod,
            path: &str,
            _: reqwest::header::HeaderMap,
            body: Option<&Value>,
        ) -> Result<ProviderHttpResponse> {
            assert_eq!(endpoint, ProviderEndpoint::BinanceTestnet);
            assert_eq!(method, ProviderHttpMethod::Get);
            assert!(body.is_none());
            let response = match path.split('?').next().unwrap_or(path) {
                "/api/v3/time" => json!({"serverTime":1788849600000u64}),
                "/api/v3/account" => json!({
                    "uid":9007199254740993u64,
                    "accountType":"SPOT",
                    "balances":[{"asset":"USDT","free":"10","locked":"0"}]
                }),
                "/api/v3/openOrders" | "/api/v3/allOrders" | "/api/v3/myTrades" => {
                    json!([])
                }
                route => panic!("Unexpected Binance Testnet fixture route: {route}"),
            };
            Ok(ProviderHttpResponse {
                status: 200,
                body: response.to_string().into_bytes(),
            })
        }
    }

    fn connected_testnet_control(path: &std::path::Path) -> (ControlPlane, AccountConnection) {
        let mut control = ControlPlane::new(path.into());
        let opened = control.dispatch(json!({
            "requestId":"stream-open","schemaVersion":1,
            "command":"workspace.open","payload":{}
        }));
        assert_eq!(opened["ok"], true, "{opened}");
        let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
        let mut account = AccountConnection::new(
            workspace_id,
            "binance".into(),
            "TESTNET".into(),
            "Loopback stream fixture".into(),
        )
        .unwrap();
        account.connection_state = ConnectionState::Connected;
        account.health.connection = "CONNECTED".into();
        account.health.authentication = "VERIFIED".into();
        account.health.credential = "CONFIGURED".into();
        account.data = Some(AccountData {
            remote_account_id: "9007199254740993".into(),
            account_type: "SPOT".into(),
            currency: None,
            buying_power: None,
            balances: vec![],
            positions: vec![],
            open_orders: vec![],
            bitget_order_book: None,
            capabilities: vec![],
            limitations: vec![],
        });
        let connection_id = account.connection_id.clone();
        control
            .store
            .as_mut()
            .unwrap()
            .save_account(account)
            .unwrap();
        let account = control
            .store
            .as_ref()
            .unwrap()
            .account(&connection_id)
            .unwrap();
        (control, account)
    }

    #[test]
    fn terminated_stream_reconnects_and_restores_current_book() {
        let workspace = tempfile::tempdir().unwrap();
        let (control, account) = connected_testnet_control(workspace.path());
        let connection_id = account.connection_id.clone();
        let engine = Arc::new(Mutex::new(control));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (connect_tx, connect_rx) = std::sync::mpsc::channel();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            for attempt in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut socket = tungstenite::accept(stream).unwrap();
                let request: Value =
                    serde_json::from_str(&socket.read().unwrap().into_text().unwrap()).unwrap();
                socket
                    .send(Message::text(
                        json!({
                            "id":request["id"],"status":200,
                            "result":{"subscriptionId":19}
                        })
                        .to_string(),
                    ))
                    .unwrap();
                connect_tx.send(()).unwrap();
                if attempt == 0 {
                    socket
                        .send(Message::text(
                            json!({
                                "subscriptionId":19,
                                "event":{"e":"eventStreamTerminated","E":1788849600000u64}
                            })
                            .to_string(),
                        ))
                        .unwrap();
                } else {
                    stop_rx.recv_timeout(Duration::from_secs(8)).unwrap();
                    let _ = socket.close(None);
                }
            }
        });

        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let worker_engine = engine.clone();
        let worker_http = StreamHttp;
        let worker = thread::spawn(move || {
            run_account_stream_with(
                worker_engine,
                account,
                worker_cancel,
                &StreamVault,
                &worker_http,
                move |_, cancelled| {
                    let (mut socket, _) = connect(format!("ws://{address}"))
                        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
                        stream
                            .set_read_timeout(Some(READ_TIMEOUT))
                            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                    }
                    let subscription_id = authenticate_and_subscribe(
                        &mut socket,
                        &[API_KEY.into(), API_SECRET.into()],
                        &StreamHttp,
                        cancelled,
                    )?;
                    Ok((
                        socket,
                        Zeroizing::new(vec![API_KEY.into(), API_SECRET.into()]),
                        subscription_id,
                    ))
                },
            );
        });

        connect_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        connect_rx.recv_timeout(Duration::from_secs(7)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let control = engine.lock().unwrap();
            let restored = control
                .store
                .as_ref()
                .unwrap()
                .account(&connection_id)
                .unwrap();
            let book = control
                .binance_private_stream_order_book(&connection_id)
                .unwrap();
            if restored.health.private_stream == "CONNECTED"
                && restored.health.reconciliation == "CURRENT"
                && book.status == crate::protocol::BinanceTestnetOrderBookStatus::Current
            {
                assert_eq!(book.balances[0].free, "10");
                break;
            }
            drop(control);
            assert!(
                Instant::now() < deadline,
                "stream did not reconcile after reconnect"
            );
            thread::sleep(Duration::from_millis(20));
        }

        cancel.store(true, Ordering::Release);
        stop_tx.send(()).unwrap();
        worker.join().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn pause_marks_connected_order_books_stale_until_reconciliation() {
        let workspace = tempfile::tempdir().unwrap();
        let (mut control, account) = connected_testnet_control(workspace.path());
        let book = crate::empty_binance_testnet_order_book(&account).unwrap();
        control
            .persist_binance_private_stream_state(
                &account.connection_id,
                &account.state_version,
                Some(book),
                "CONNECTED",
                "CURRENT",
                "Fixture stream reconciled.",
                None,
            )
            .unwrap();
        let connection_id = account.connection_id.clone();
        let engine = Arc::new(Mutex::new(control));

        mark_connected_accounts_degraded(&engine);

        let control = engine.lock().unwrap();
        let account = control
            .store
            .as_ref()
            .unwrap()
            .account(&connection_id)
            .unwrap();
        let book = control
            .binance_private_stream_order_book(&connection_id)
            .unwrap();
        assert_eq!(account.health.private_stream, "DEGRADED");
        assert_eq!(account.health.reconciliation, "DEGRADED");
        assert_eq!(
            book.status,
            crate::protocol::BinanceTestnetOrderBookStatus::Stale
        );
        assert_eq!(book.reason.as_deref(), Some("PRIVATE_STREAM_DISCONNECTED"));
    }
}
