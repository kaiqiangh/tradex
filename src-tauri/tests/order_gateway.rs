use sha2::{Digest, Sha256};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use tradex::order_gateway::OrderGatewayHost;

fn digest(path: &Path) -> String {
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

fn shell_gateway(script: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fake-order-gateway");
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    (directory, path)
}

fn response_script(response: &[u8]) -> String {
    let mut frame = (response.len() as u32).to_be_bytes().to_vec();
    frame.extend_from_slice(response);
    let escaped = frame
        .iter()
        .map(|byte| format!("\\{byte:03o}"))
        .collect::<String>();
    format!("#!/bin/sh\nprintf '%b' '{escaped}' >&3\n")
}

#[test]
fn launches_the_pinned_gateway_over_a_private_session_and_stops_it() {
    let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
    let mut gateway = OrderGatewayHost::new(executable.to_owned(), digest(executable));

    gateway.start().unwrap();
    assert!(gateway.is_running());

    gateway.stop();
    assert!(!gateway.is_running());
}

#[test]
fn refuses_to_launch_an_executable_that_does_not_match_its_pin() {
    let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
    let mut gateway = OrderGatewayHost::new(executable.to_owned(), "0".repeat(64));

    assert_eq!(gateway.start(), Err("GATEWAY_BINARY_INVALID"));
    assert!(!gateway.is_running());
}

#[test]
fn rejects_a_gateway_with_an_incompatible_protocol_version() {
    let response = br#"{"kind":"ready","protocolVersion":2,"sessionCredential":"unused"}"#;
    let (_directory, executable) = shell_gateway(&response_script(response));
    let mut gateway = OrderGatewayHost::new(executable.clone(), digest(&executable));

    assert_eq!(gateway.start(), Err("GATEWAY_PROTOCOL_MISMATCH"));
    assert!(!gateway.is_running());
}

#[test]
fn rejects_a_gateway_that_does_not_echo_the_current_session_credential() {
    let response = br#"{"kind":"ready","protocolVersion":1,"sessionCredential":"wrong"}"#;
    let (_directory, executable) = shell_gateway(&response_script(response));
    let mut gateway = OrderGatewayHost::new(executable.clone(), digest(&executable));

    assert_eq!(gateway.start(), Err("GATEWAY_AUTH_FAILED"));
    assert!(!gateway.is_running());
}

#[test]
fn rejects_an_oversized_handshake_frame_before_reading_its_body() {
    let script = "#!/bin/sh\nprintf '\\000\\001\\000\\001' >&3\n";
    let (_directory, executable) = shell_gateway(script);
    let mut gateway = OrderGatewayHost::new(executable.clone(), digest(&executable));

    assert_eq!(gateway.start(), Err("GATEWAY_FRAME_TOO_LARGE"));
    assert!(!gateway.is_running());
}

#[cfg(all(
    feature = "integration-test",
    feature = "order-gateway-runtime",
    target_os = "macos"
))]
mod local_provider_tests {
    use super::*;
    use hmac::{Hmac, Mac};
    use serde_json::{Value, json};
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::{Shutdown, TcpListener, TcpStream},
        sync::{Arc, Mutex},
        thread,
    };
    use tradex::{
        order_gateway::{GatewayDispatchIntent, GatewayDispatchPackage},
        protocol::{
            CancellationIntent, ExecutionAttempt, ExecutionAttemptState, ExecutionContext,
            ExecutionDispatchGrant, ExecutionDispatchGrantStatus, ExecutionReservation,
            ExecutionReservationStatus, FinancialOperation, MarketDataStatus, OrderDraftFields,
            OrderProposal, OrderProposalStatus, OrderQuantity, OrderQuantityType, OrderSide,
            OrderType, ProposalReferenceStatus, TimeInForce,
        },
        provider_io::LiveDispatchOutcome,
        providers::{
            AccountConnection, AccountData, AccountHealth, BitgetSpotOrder, BitgetSpotOrderBook,
            ConnectionState, OpenOrder, PermissionReview,
        },
    };

    #[derive(Clone, Debug)]
    struct CapturedRequest {
        method: String,
        path: String,
        authorization: Option<String>,
        api_key: Option<String>,
        access_key: Option<String>,
        access_passphrase: Option<String>,
        access_sign: Option<String>,
        access_timestamp: Option<String>,
        paptrading: Option<String>,
        body: Value,
    }

    type DispatchOutcomeSlot = Arc<Mutex<Option<LiveDispatchOutcome>>>;
    type ChildRun = (
        OrderGatewayHost,
        DispatchOutcomeSlot,
        Result<(), &'static str>,
    );

    fn fake_provider(
        calls_to_accept: usize,
        mutation_status: u16,
        mutation_body: &'static [u8],
        drop_mutation_response: bool,
    ) -> (
        String,
        Arc<Mutex<Vec<CapturedRequest>>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let captured = Arc::new(Mutex::new(Vec::new()));
        let capture = captured.clone();
        let thread = thread::spawn(move || {
            let expect_no_calls = calls_to_accept == 0;
            let expected_calls = calls_to_accept.max(1);
            let deadline = std::time::Instant::now()
                + std::time::Duration::from_secs(if expect_no_calls { 1 } else { 15 });
            for _ in 0..expected_calls {
                let (stream, _) = loop {
                    match listener.accept() {
                        Ok(connection) => break connection,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && std::time::Instant::now() < deadline =>
                        {
                            thread::sleep(std::time::Duration::from_millis(5));
                        }
                        Err(error)
                            if expect_no_calls
                                && error.kind() == std::io::ErrorKind::WouldBlock =>
                        {
                            return;
                        }
                        Err(error) => panic!("fake provider accept failed: {error}"),
                    }
                };
                let (request, mut stream) = read_request(stream);
                let is_mutation = request.method != "GET";
                let read_body = if request.path == "/api/v0/equity/account/summary" {
                    br#"{"id":777}"#.as_slice()
                } else if request.path == "/api/v0/equity/orders/123457" {
                    br#"{"id":123457,"ticker":"AAPL_US_EQ","side":"BUY","strategy":"QUANTITY","quantity":2,"filledQuantity":0,"status":"NEW"}"#.as_slice()
                } else if request.path == "/api/v0/equity/orders/123458" {
                    br#"{"id":123458,"ticker":"AAPL_US_EQ","side":"BUY","strategy":"QUANTITY","quantity":1,"filledQuantity":0,"status":"FILLED"}"#.as_slice()
                } else if request.path == "/api/v0/equity/orders/123459" {
                    br#"{"id":123459,"ticker":"AAPL_US_EQ","side":"BUY","strategy":"QUANTITY","quantity":1,"filledQuantity":0}"#.as_slice()
                } else {
                    br#"{"id":123456,"ticker":"AAPL_US_EQ","side":"BUY","strategy":"QUANTITY","quantity":1,"filledQuantity":0,"status":"NEW"}"#.as_slice()
                };
                capture.lock().unwrap().push(request);
                let (status, body) = if !is_mutation {
                    (200, read_body)
                } else if drop_mutation_response {
                    let _ = stream.shutdown(Shutdown::Both);
                    continue;
                } else {
                    (mutation_status, mutation_body)
                };
                let reason = if status == 200 { "OK" } else { "Rejected" };
                write!(
                    stream,
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
                .unwrap();
                stream.write_all(body).unwrap();
            }
        });
        (format!("http://{address}"), captured, thread)
    }

    fn fake_binance_provider(
        calls_to_accept: usize,
        mutation_status: u16,
        mutation_body: &'static [u8],
        drop_mutation_response: bool,
        remote_account_id: &'static str,
        order_body: &'static [u8],
    ) -> (
        String,
        Arc<Mutex<Vec<CapturedRequest>>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let captured = Arc::new(Mutex::new(Vec::new()));
        let capture = captured.clone();
        let thread = thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            for _ in 0..calls_to_accept {
                let (stream, _) = loop {
                    match listener.accept() {
                        Ok(connection) => break connection,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && std::time::Instant::now() < deadline =>
                        {
                            thread::sleep(std::time::Duration::from_millis(5));
                        }
                        Err(error) => panic!("fake Binance provider accept failed: {error}"),
                    }
                };
                let (request, mut stream) = read_request(stream);
                let is_mutation = request.method == "DELETE";
                let read_body = if is_mutation {
                    b"".as_slice()
                } else if request.path == "/api/v3/time" {
                    br#"{"serverTime":1788849600000}"#.as_slice()
                } else if request.path.starts_with("/api/v3/account?") {
                    if remote_account_id == "777" {
                        br#"{"accountType":"SPOT","uid":777}"#.as_slice()
                    } else {
                        br#"{"accountType":"SPOT","uid":778}"#.as_slice()
                    }
                } else if request.path.starts_with("/api/v3/order?") && request.method == "GET" {
                    order_body
                } else {
                    panic!(
                        "unexpected Binance loopback request: {} {}",
                        request.method, request.path
                    )
                };
                capture.lock().unwrap().push(request);
                if is_mutation && drop_mutation_response {
                    let _ = stream.shutdown(Shutdown::Both);
                    continue;
                }
                let (status, body) = if is_mutation {
                    (mutation_status, mutation_body)
                } else {
                    (200, read_body)
                };
                let reason = if status == 200 { "OK" } else { "Rejected" };
                write!(
                    stream,
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
                .unwrap();
                stream.write_all(body).unwrap();
            }
        });
        (format!("http://{address}"), captured, thread)
    }

    fn fake_bitget_provider(
        mutation_status: u16,
        mutation_body: &'static [u8],
        drop_mutation_response: bool,
    ) -> (
        String,
        Arc<Mutex<Vec<CapturedRequest>>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let captured = Arc::new(Mutex::new(Vec::new()));
        let capture = captured.clone();
        let thread = thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
            for _ in 0..5 {
                let (stream, _) = loop {
                    match listener.accept() {
                        Ok(connection) => break connection,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && std::time::Instant::now() < deadline =>
                        {
                            thread::sleep(std::time::Duration::from_millis(5));
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return,
                        Err(error) => panic!("fake Bitget provider accept failed: {error}"),
                    }
                };
                let (request, mut stream) = read_request(stream);
                let (status, body) = match (request.method.as_str(), request.path.as_str()) {
                    ("GET", "/api/v2/public/time") => (
                        200,
                        br#"{"code":"00000","data":{"serverTime":"1788849600000"}}"#.as_slice(),
                    ),
                    ("GET", "/api/v2/spot/account/info") => (
                        200,
                        br#"{"code":"00000","data":{"userId":"777","ips":"127.0.0.1","authorities":["stor","stow"]}}"#.as_slice(),
                    ),
                    ("GET", "/api/v2/spot/trade/orderInfo?orderId=12345") => (
                        200,
                        br#"{"code":"00000","data":[{"userId":"777","orderId":"12345","symbol":"BTCUSDT","price":"70000","size":"1","orderType":"limit","side":"buy","status":"live","priceAvg":"0","baseVolume":"0","quoteVolume":"0","quoteCoin":"USDT","tpslType":"normal","cTime":"1788849500000","uTime":"1788849600000"}]}"#.as_slice(),
                    ),
                    ("POST", "/api/v2/spot/trade/cancel-order") => {
                        capture.lock().unwrap().push(request.clone());
                        if drop_mutation_response {
                            let _ = stream.shutdown(Shutdown::Both);
                            continue;
                        }
                        (mutation_status, mutation_body)
                    }
                    _ => panic!(
                        "unexpected Bitget loopback request: {} {}",
                        request.method, request.path
                    ),
                };
                if request.method != "POST" {
                    capture.lock().unwrap().push(request);
                }
                write!(
                    stream,
                    "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
                .unwrap();
                stream.write_all(body).unwrap();
            }
        });
        (format!("http://{address}"), captured, thread)
    }

    fn assert_bitget_signature(request: &CapturedRequest) {
        use base64::Engine;
        assert_eq!(request.access_key.as_deref(), Some("synthetic-api-key"));
        assert_eq!(
            request.access_passphrase.as_deref(),
            Some("synthetic-api-passphrase")
        );
        assert!(request.paptrading.is_none());
        let timestamp = request.access_timestamp.as_deref().unwrap();
        let body = if request.body.is_null() {
            String::new()
        } else {
            serde_json::to_string(&request.body).unwrap()
        };
        let mut mac = Hmac::<Sha256>::new_from_slice(b"synthetic-api-secret").unwrap();
        mac.update(format!("{timestamp}{}{}{}", request.method, request.path, body).as_bytes());
        mac.verify_slice(
            &base64::engine::general_purpose::STANDARD
                .decode(request.access_sign.as_deref().unwrap())
                .unwrap(),
        )
        .unwrap();
    }

    fn assert_binance_signature(request: &CapturedRequest) {
        assert_eq!(request.api_key.as_deref(), Some("synthetic-api-key"));
        let (_, query) = request.path.split_once('?').unwrap();
        let (unsigned, signature) = query.split_once("&signature=").unwrap();
        let mut mac = Hmac::<Sha256>::new_from_slice(b"synthetic-api-secret").unwrap();
        mac.update(unsigned.as_bytes());
        assert_eq!(hex::encode(mac.finalize().into_bytes()), signature);
    }

    fn read_request(stream: TcpStream) -> (CapturedRequest, TcpStream) {
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        let mut authorization = None;
        let mut api_key = None;
        let mut access_key = None;
        let mut access_passphrase = None;
        let mut access_sign = None;
        let mut access_timestamp = None;
        let mut paptrading = None;
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" || line.is_empty() {
                break;
            }
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("authorization") {
                    authorization = Some(value.trim().to_owned());
                }
                if name.eq_ignore_ascii_case("x-mbx-apikey") {
                    api_key = Some(value.trim().to_owned());
                }
                if name.eq_ignore_ascii_case("access-key") {
                    access_key = Some(value.trim().to_owned());
                }
                if name.eq_ignore_ascii_case("access-passphrase") {
                    access_passphrase = Some(value.trim().to_owned());
                }
                if name.eq_ignore_ascii_case("access-sign") {
                    access_sign = Some(value.trim().to_owned());
                }
                if name.eq_ignore_ascii_case("access-timestamp") {
                    access_timestamp = Some(value.trim().to_owned());
                }
                if name.eq_ignore_ascii_case("paptrading") {
                    paptrading = Some(value.trim().to_owned());
                }
                if name.eq_ignore_ascii_case("content-length") {
                    content_length = value.trim().parse().unwrap_or_default();
                }
            }
        }
        let mut body = vec![0; content_length];
        reader.read_exact(&mut body).unwrap();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default().to_owned();
        let path = parts.next().unwrap_or_default().to_owned();
        (
            CapturedRequest {
                method,
                path,
                authorization,
                api_key,
                access_key,
                access_passphrase,
                access_sign,
                access_timestamp,
                paptrading,
                body: if body.is_empty() {
                    Value::Null
                } else {
                    serde_json::from_slice(&body).unwrap_or(Value::Null)
                },
            },
            reader.into_inner(),
        )
    }

    fn proposal() -> OrderProposal {
        OrderProposal {
            proposal_id: "proposal-1".into(),
            workspace_id: "integration-test".into(),
            draft_id: "draft-1".into(),
            draft_version: 1,
            proposal_hash: format!("sha256:{}", "a".repeat(64)),
            fields: OrderDraftFields {
                account_id: Some("account-1".into()),
                venue: "XNAS".into(),
                environment: ExecutionContext::Trading212Live,
                instrument_id: "equity:US:AAPL".into(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: OrderQuantity {
                    r#type: OrderQuantityType::Base,
                    value: "1".into(),
                },
                limit_price: None,
                maximum_spend: None,
                time_in_force: TimeInForce::Day,
                client_label: None,
            },
            estimated_notional: Some("200".into()),
            estimated_notional_currency: Some("USD".into()),
            estimated_notional_reason: None,
            policy_version: Some(1),
            policy_state_version: Some("risk-policy:1".into()),
            policy_status: ProposalReferenceStatus::Available,
            policy_reference_reason: "current".into(),
            market_snapshot_id: Some("quote-1".into()),
            market_status: MarketDataStatus::Available,
            market_reference_reason: "current".into(),
            status: OrderProposalStatus::Consumed,
            invalidation_reason: None,
            created_at: "2026-09-27T10:00:00Z".into(),
            state_version: "order-proposal:proposal-1:2".into(),
            history: vec![],
        }
    }

    fn package(attempt_id: &str, intent: GatewayDispatchIntent) -> GatewayDispatchPackage {
        let proposal = match &intent {
            GatewayDispatchIntent::Place(proposal) => Some(proposal),
            GatewayDispatchIntent::Cancel(_) => None,
        };
        let cancel = match &intent {
            GatewayDispatchIntent::Cancel(intent) => Some(intent),
            GatewayDispatchIntent::Place(_) => None,
        };
        let operation = if proposal.is_some() {
            FinancialOperation::PlaceOrder
        } else {
            FinancialOperation::Cancel
        };
        let intent_id = proposal
            .map(|proposal| proposal.proposal_id.clone())
            .or_else(|| cancel.map(|intent| intent.cancellation_intent_id.clone()))
            .unwrap();
        let intent_hash = proposal
            .map(|proposal| proposal.proposal_hash.clone())
            .or_else(|| cancel.map(|intent| intent.intent_hash.clone()))
            .unwrap();
        let provider_order_id = cancel.map(|intent| intent.provider_order_id.clone());
        let reservation_id = proposal.map(|_| "reservation-1".to_owned());
        let execution_context = cancel
            .map(|intent| intent.environment.clone())
            .unwrap_or(ExecutionContext::Trading212Live);
        let provider_id = match execution_context {
            ExecutionContext::BinanceLive => "binance",
            ExecutionContext::BitgetLive => "bitget",
            _ => "trading212",
        };
        let mut account = AccountConnection::new(
            "integration-test".into(),
            provider_id.into(),
            "LIVE".into(),
            "synthetic live account".into(),
        )
        .unwrap();
        account.connection_id = "account-1".into();
        account.state_version = "account:account-1:1".into();
        account.connection_state = ConnectionState::Connected;
        account.health = AccountHealth {
            connection: "ONLINE".into(),
            authentication: "VALID".into(),
            credential: "AVAILABLE".into(),
            private_stream: "NOT_CONFIGURED".into(),
            reconciliation: "CURRENT".into(),
            execution_eligibility: "ELIGIBLE".into(),
            arming: "ARMED".into(),
            arming_reason: "USER_ARMED".into(),
            reason: "ready".into(),
        };
        account.permissions = PermissionReview {
            scope: "VERIFIED".into(),
            detected: vec![
                "account.read".into(),
                "positions.read".into(),
                "orders.read".into(),
            ],
            forbidden: vec![],
            unsupported: vec![],
            acknowledged: true,
            ip_allow_list_status: "UNKNOWN".into(),
            ip_allow_list: None,
        };
        let bitget_live = provider_id == "bitget";
        account.data = Some(AccountData {
            remote_account_id: "777".into(),
            account_type: "INVEST".into(),
            currency: Some("USD".into()),
            buying_power: Some("1000".into()),
            balances: vec![],
            positions: vec![],
            open_orders: if bitget_live {
                vec![OpenOrder {
                    broker_order_id: "normal:12345".into(),
                    symbol: "BTCUSDT".into(),
                    instrument_id: Some("crypto:BTC/USDT:spot".into()),
                    side: "BUY".into(),
                    quantity: Some("1".into()),
                    notional: None,
                    filled_quantity: Some("0".into()),
                    filled_value: Some("0".into()),
                    currency: Some("USDT".into()),
                    status: "LIVE".into(),
                    limit_price: None,
                    kind: Some("NORMAL".into()),
                    trigger_price: None,
                }]
            } else {
                vec![]
            },
            bitget_order_book: bitget_live.then(|| BitgetSpotOrderBook {
                orders: vec![BitgetSpotOrder {
                    provider_order_id: "12345".into(),
                    kind: "NORMAL".into(),
                    symbol: "BTCUSDT".into(),
                    side: "BUY".into(),
                    quantity: Some("1".into()),
                    notional: None,
                    filled_quantity: Some("0".into()),
                    filled_value: Some("0".into()),
                    remaining_quantity: Some("1".into()),
                    currency: Some("USDT".into()),
                    provider_status: "live".into(),
                    normalized_status: "OPEN".into(),
                    origin: "external".into(),
                    created_at: Some("2026-09-08T06:38:20Z".into()),
                    updated_at: Some("2026-09-08T06:40:00Z".into()),
                }],
                fills: vec![],
                observed_at: "2026-09-08T06:40:00Z".into(),
            }),
            capabilities: vec![
                "account.read".into(),
                "positions.read".into(),
                "orders.read".into(),
            ],
            limitations: vec![],
        });
        let attempt = ExecutionAttempt {
            attempt_id: attempt_id.into(),
            workspace_id: "integration-test".into(),
            approval_id: "approval-1".into(),
            operation,
            intent_id: intent_id.clone(),
            intent_hash: intent_hash.clone(),
            proposal_id: proposal.map(|proposal| proposal.proposal_id.clone()),
            broker_order_id: provider_order_id.clone(),
            provider_client_order_id: None,
            provider_status: None,
            trading212_live_order_observation: None,
            binance_live_order_observation: None,
            bitget_live_order_observation: None,
            error_code: None,
            dispatch_disposition: None,
            account_id: "account-1".into(),
            environment: execution_context,
            policy_version: 1,
            risk_decision_id: "risk-1".into(),
            review_digest: format!("sha256:{}", "b".repeat(64)),
            account_state_version: account.state_version.clone(),
            intent_state_version: "intent:1".into(),
            reservation_id: reservation_id.clone(),
            state: ExecutionAttemptState::Reserved,
            invalidation_reason: None,
            created_at: "2026-09-27T10:00:00Z".into(),
            dispatch_started_at: None,
            state_version: format!("execution-attempt:{attempt_id}:1"),
        };
        let reservation = reservation_id
            .clone()
            .map(|reservation_id| ExecutionReservation {
                reservation_id,
                workspace_id: "integration-test".into(),
                account_id: "account-1".into(),
                attempt_id: attempt_id.into(),
                proposal_id: intent_id.clone(),
                proposal_hash: intent_hash.clone(),
                instrument_id: "equity:US:AAPL".into(),
                side: OrderSide::Buy,
                capacity_key: "account-1:USD".into(),
                amount: "200".into(),
                unit: "USD".into(),
                notional: "200".into(),
                notional_currency: "USD".into(),
                workspace_notional: Some("200".into()),
                workspace_currency: "USD".into(),
                broker_available: "1000".into(),
                existing_reservations: "0".into(),
                effective_available: "800".into(),
                capacity_projection: None,
                account_state_version: account.state_version.clone(),
                status: ExecutionReservationStatus::Active,
                created_at: "2026-09-27T10:00:00Z".into(),
                state_version: "execution-reservation:reservation-1:1".into(),
            });
        GatewayDispatchPackage {
            grant: ExecutionDispatchGrant {
                grant_id: format!("grant-{attempt_id}"),
                workspace_id: "integration-test".into(),
                attempt_id: attempt_id.into(),
                account_id: "account-1".into(),
                operation,
                intent_id,
                intent_hash,
                approval_id: "approval-1".into(),
                reservation_id,
                gateway_session_id: String::new(),
                account_state_version: account.state_version.clone(),
                intent_state_version: attempt.intent_state_version.clone(),
                policy_version: 1,
                issued_at: "2026-09-27T10:00:00Z".into(),
                expires_at: "2026-09-27T10:00:30Z".into(),
                status: ExecutionDispatchGrantStatus::Issued,
                state_version: "execution-dispatch-grant:1".into(),
            },
            attempt,
            reservation,
            credential_reference: account.credential_ref(),
            account,
            intent,
            local_test_base_url: None,
        }
    }

    fn cancel_intent(order_id: &str) -> CancellationIntent {
        CancellationIntent {
            cancellation_intent_id: format!("cancel-intent-{order_id}"),
            intent_hash: format!("sha256:{}", "c".repeat(64)),
            workspace_id: "integration-test".into(),
            account_id: "account-1".into(),
            environment: ExecutionContext::Trading212Live,
            provider_order_id: order_id.into(),
            instrument_id: "equity:US:AAPL".into(),
            symbol: "AAPL_US_EQ".into(),
            side: "BUY".into(),
            provider_status: "NEW".into(),
            quantity: "1".into(),
            filled_quantity: "0".into(),
            remaining_quantity: "1".into(),
            created_at: "2026-09-27T10:00:00Z".into(),
        }
    }

    fn binance_cancel_intent() -> CancellationIntent {
        CancellationIntent {
            cancellation_intent_id: "cancel-binance-12345".into(),
            intent_hash: format!("sha256:{}", "d".repeat(64)),
            workspace_id: "integration-test".into(),
            account_id: "account-1".into(),
            environment: ExecutionContext::BinanceLive,
            provider_order_id: "BTCUSDT:12345".into(),
            instrument_id: "crypto:BTC/USDT:spot".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            provider_status: "NEW".into(),
            quantity: "1".into(),
            filled_quantity: "0".into(),
            remaining_quantity: "1".into(),
            created_at: "2026-09-27T10:00:00Z".into(),
        }
    }

    fn bitget_cancel_intent() -> CancellationIntent {
        CancellationIntent {
            cancellation_intent_id: "cancel-bitget-12345".into(),
            intent_hash: format!("sha256:{}", "e".repeat(64)),
            workspace_id: "integration-test".into(),
            account_id: "account-1".into(),
            environment: ExecutionContext::BitgetLive,
            provider_order_id: "normal:12345".into(),
            instrument_id: "crypto:BTC/USDT:spot".into(),
            symbol: "BTCUSDT".into(),
            side: "BUY".into(),
            provider_status: "LIVE".into(),
            quantity: "1".into(),
            filled_quantity: "0".into(),
            remaining_quantity: "1".into(),
            created_at: "2026-09-08T06:38:20Z".into(),
        }
    }

    fn run_child(
        package: GatewayDispatchPackage,
        base_url: String,
    ) -> (OrderGatewayHost, DispatchOutcomeSlot) {
        let (gateway, result, dispatch) = run_child_with_persistence(package, base_url, true);
        dispatch.unwrap();
        (gateway, result)
    }

    fn run_child_recording_pre_dispatch(
        package: GatewayDispatchPackage,
        base_url: String,
    ) -> (
        OrderGatewayHost,
        DispatchOutcomeSlot,
        Arc<Mutex<Option<String>>>,
    ) {
        let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
        let mut gateway = OrderGatewayHost::new(executable.to_owned(), digest(executable));
        gateway.start().unwrap();
        let package = Arc::new(Mutex::new(package));
        let result = Arc::new(Mutex::new(None));
        let stopped = Arc::new(Mutex::new(None));
        let attempt_id = package.lock().unwrap().attempt.attempt_id.clone();
        let issue_package = package.clone();
        let begin_package = package.clone();
        let stopped_callback = stopped.clone();
        let result_callback = result.clone();
        gateway
            .dispatch_attempt(
                &attempt_id,
                move |_, session| {
                    let mut package = issue_package.lock().unwrap();
                    package.grant.gateway_session_id = session.into();
                    package.local_test_base_url = Some(base_url.clone());
                    Ok(package.clone())
                },
                move |grant_id, session| {
                    let package = begin_package.lock().unwrap();
                    if package.grant.grant_id == grant_id
                        && package.grant.gateway_session_id == session
                    {
                        Ok(())
                    } else {
                        Err("GATEWAY_AUTH_FAILED".into())
                    }
                },
                move |_, reason| *stopped_callback.lock().unwrap() = Some(reason.into()),
                move |_, outcome| {
                    *result_callback.lock().unwrap() = Some(outcome.clone());
                    Ok(())
                },
            )
            .unwrap();
        (gateway, result, stopped)
    }

    fn run_child_with_persistence(
        package: GatewayDispatchPackage,
        base_url: String,
        persist_result: bool,
    ) -> ChildRun {
        let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
        let mut gateway = OrderGatewayHost::new(executable.to_owned(), digest(executable));
        gateway.start().unwrap();
        let package = Arc::new(Mutex::new(package));
        let result = Arc::new(Mutex::new(None));
        let attempt_id = package.lock().unwrap().attempt.attempt_id.clone();
        let issue_package = package.clone();
        let begin_package = package.clone();
        let result_callback = result.clone();
        let dispatch = gateway.dispatch_attempt(
            &attempt_id,
            move |_, session| {
                let mut package = issue_package.lock().unwrap();
                package.grant.gateway_session_id = session.into();
                package.local_test_base_url = Some(base_url.clone());
                Ok(package.clone())
            },
            move |grant_id, session| {
                let package = begin_package.lock().unwrap();
                if package.grant.grant_id == grant_id && package.grant.gateway_session_id == session
                {
                    Ok(())
                } else {
                    Err("GATEWAY_AUTH_FAILED".into())
                }
            },
            |_, _| {},
            move |_, outcome| {
                *result_callback.lock().unwrap() = Some(outcome.clone());
                if persist_result {
                    Ok(())
                } else {
                    Err("WORKSPACE_OPEN_FAILED".into())
                }
            },
        );
        (gateway, result, dispatch)
    }

    #[test]
    fn real_child_does_not_contact_provider_after_expired_grant_is_rejected() {
        let (url, calls, provider) = fake_provider(0, 200, b"", false);
        let mut dispatch_package = package(
            "place-expired-grant",
            GatewayDispatchIntent::Place(Box::new(proposal())),
        );
        dispatch_package.grant.expires_at = "2026-09-27T09:59:59Z".into();
        let attempt_id = dispatch_package.attempt.attempt_id.clone();
        let package = Arc::new(Mutex::new(dispatch_package));
        let issue_package = package.clone();
        let begin_package = package.clone();
        let stopped = Arc::new(Mutex::new(false));
        let stopped_callback = stopped.clone();
        let result: DispatchOutcomeSlot = Arc::new(Mutex::new(None));
        let result_callback = result.clone();
        let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
        let mut gateway = OrderGatewayHost::new(executable.to_owned(), digest(executable));
        gateway.start().unwrap();

        let dispatch = gateway.dispatch_attempt(
            &attempt_id,
            move |_, session| {
                let mut package = issue_package.lock().unwrap();
                package.grant.gateway_session_id = session.into();
                package.local_test_base_url = Some(url.clone());
                Ok(package.clone())
            },
            move |grant_id, session| {
                let package = begin_package.lock().unwrap();
                if package.grant.grant_id != grant_id || package.grant.gateway_session_id != session
                {
                    return Err("GATEWAY_AUTH_FAILED".into());
                }
                let now = time::OffsetDateTime::parse(
                    "2026-09-27T10:00:00Z",
                    &time::format_description::well_known::Rfc3339,
                )
                .unwrap();
                let expires = time::OffsetDateTime::parse(
                    &package.grant.expires_at,
                    &time::format_description::well_known::Rfc3339,
                )
                .unwrap();
                if expires <= now {
                    Err("EXECUTION_DISPATCH_NOT_READY".into())
                } else {
                    Ok(())
                }
            },
            move |_, _| *stopped_callback.lock().unwrap() = true,
            move |_, outcome| {
                *result_callback.lock().unwrap() = Some(outcome.clone());
                Ok(())
            },
        );

        assert_eq!(dispatch, Ok(()));
        assert!(*stopped.lock().unwrap());
        assert!(result.lock().unwrap().is_none());
        gateway.stop();
        provider.join().unwrap();
        assert!(calls.lock().unwrap().is_empty());
    }

    #[test]
    fn real_child_sends_one_exact_live_place_to_the_loopback_fake_provider() {
        let (url, calls, provider) = fake_provider(
            2,
            200,
            br#"{"id":901,"ticker":"AAPL_US_EQ","status":"NEW"}"#,
            false,
        );
        let (mut gateway, result) = run_child(
            package(
                "place-1",
                GatewayDispatchIntent::Place(Box::new(proposal())),
            ),
            url,
        );
        provider.join().unwrap();
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].path, "/api/v0/equity/account/summary");
        assert_eq!(requests[1].method, "POST");
        assert_eq!(requests[1].path, "/api/v0/equity/orders/market");
        assert_eq!(
            requests[1].body,
            json!({"ticker":"AAPL_US_EQ","quantity":1,"extendedHours":false})
        );
        use base64::Engine;
        let expected_authorization = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD
                .encode("synthetic-api-key:synthetic-api-secret")
        );
        assert!(requests.iter().all(|request| {
            request.authorization.as_deref() == Some(expected_authorization.as_str())
        }));
        assert_eq!(
            result.lock().unwrap().as_ref().unwrap().state,
            ExecutionAttemptState::Accepted
        );
        assert_eq!(
            result
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .broker_order_id
                .as_deref(),
            Some("901")
        );
        let safe_result = serde_json::to_string(result.lock().unwrap().as_ref().unwrap()).unwrap();
        assert!(!safe_result.contains("synthetic-api-key"));
        assert!(!safe_result.contains("synthetic-api-secret"));
        drop(requests);

        let attempt_id = "place-1";
        let duplicate = gateway.dispatch_attempt(
            attempt_id,
            |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
            |_, _| panic!("duplicate activation must not cross SUBMITTING"),
            |_, _| {},
            |_, _| Ok(()),
        );
        assert_eq!(duplicate, Ok(()));
        assert_eq!(calls.lock().unwrap().len(), 2);
        gateway.stop();
    }

    #[test]
    fn real_child_sends_the_exact_live_cancel_and_records_only_pending_acknowledgement() {
        let intent = cancel_intent("123456");
        let (url, calls, provider) = fake_provider(3, 204, b"", false);
        let (mut gateway, result) = run_child(
            package("cancel-1", GatewayDispatchIntent::Cancel(Box::new(intent))),
            url,
        );
        provider.join().unwrap();
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].path, "/api/v0/equity/account/summary");
        assert_eq!(requests[1].method, "GET");
        assert_eq!(requests[1].path, "/api/v0/equity/orders/123456");
        assert_eq!(requests[2].method, "DELETE");
        assert_eq!(requests[2].path, "/api/v0/equity/orders/123456");
        assert!(requests[0].authorization.is_some());
        assert!(
            requests
                .iter()
                .all(|request| { request.authorization == requests[0].authorization })
        );
        let outcome = result.lock().unwrap().clone().unwrap();
        assert_eq!(outcome.state, ExecutionAttemptState::CancelPending);
        assert_eq!(outcome.broker_order_id.as_deref(), Some("123456"));
        assert_eq!(outcome.provider_status.as_deref(), Some("NEW"));
        drop(requests);
        let duplicate = gateway.dispatch_attempt(
            "cancel-1",
            |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
            |_, _| panic!("duplicate cancellation must not cross SUBMITTING"),
            |_, _| {},
            |_, _| Ok(()),
        );
        assert_eq!(duplicate, Ok(()));
        assert_eq!(calls.lock().unwrap().len(), 3);
        gateway.stop();
    }

    #[test]
    fn real_child_sends_one_signed_exact_binance_spot_live_cancel() {
        let order = br#"{"symbol":"BTCUSDT","orderId":12345,"orderListId":-1,"side":"BUY","status":"NEW","origQty":"1","executedQty":"0","cummulativeQuoteQty":"0","updateTime":1788849600000}"#;
        let acknowledgement =
            br#"{"symbol":"BTCUSDT","orderId":12345,"orderListId":-1,"status":"CANCELED"}"#;
        let (url, calls, provider) =
            fake_binance_provider(5, 200, acknowledgement, false, "777", order);
        let (mut gateway, result) = run_child(
            package(
                "binance-cancel-1",
                GatewayDispatchIntent::Cancel(Box::new(binance_cancel_intent())),
            ),
            url,
        );
        provider.join().unwrap();
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 5, "{requests:#?}");
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].path, "/api/v3/time");
        assert_eq!(requests[1].method, "GET");
        assert!(requests[1].path.starts_with("/api/v3/account?"));
        assert_eq!(requests[2].method, "GET");
        assert!(
            requests[2]
                .path
                .starts_with("/api/v3/order?symbol=BTCUSDT&orderId=12345&timestamp=")
        );
        assert_eq!(requests[3].path, "/api/v3/time");
        assert_eq!(requests[4].method, "DELETE");
        assert!(
            requests[4]
                .path
                .starts_with("/api/v3/order?symbol=BTCUSDT&orderId=12345&timestamp=")
        );
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.method == "DELETE")
                .count(),
            1
        );
        for request in [&requests[1], &requests[2], &requests[4]] {
            assert_binance_signature(request);
        }
        let outcome = result.lock().unwrap().clone().unwrap();
        assert_eq!(outcome.state, ExecutionAttemptState::CancelPending);
        assert_eq!(outcome.broker_order_id.as_deref(), Some("BTCUSDT:12345"));
        assert_eq!(outcome.provider_status.as_deref(), Some("NEW"));
        let safe_result = serde_json::to_string(&outcome).unwrap();
        assert!(!safe_result.contains("synthetic-api-key"));
        assert!(!safe_result.contains("synthetic-api-secret"));
        drop(requests);

        let duplicate = gateway.dispatch_attempt(
            "binance-cancel-1",
            |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
            |_, _| panic!("duplicate Binance cancellation must not cross SUBMITTING"),
            |_, _| {},
            |_, _| Ok(()),
        );
        assert_eq!(duplicate, Ok(()));
        assert_eq!(calls.lock().unwrap().len(), 5);
        gateway.stop();
    }

    #[test]
    fn real_child_sends_one_signed_exact_bitget_classic_spot_live_cancel() {
        let (url, calls, provider) = fake_bitget_provider(
            200,
            br#"{"code":"00000","data":{"orderId":"12345"}}"#,
            false,
        );
        let (mut gateway, result, stopped) = run_child_recording_pre_dispatch(
            package(
                "bitget-cancel-1",
                GatewayDispatchIntent::Cancel(Box::new(bitget_cancel_intent())),
            ),
            url,
        );
        provider.join().unwrap();
        assert_eq!(*stopped.lock().unwrap(), None);
        let outcome = result.lock().unwrap().clone().unwrap();
        assert_eq!(
            outcome.state,
            ExecutionAttemptState::CancelPending,
            "{outcome:?}"
        );
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 5);
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[0].path, "/api/v2/public/time");
        assert_eq!(requests[1].path, "/api/v2/spot/account/info");
        assert_eq!(
            requests[2].path,
            "/api/v2/spot/trade/orderInfo?orderId=12345"
        );
        assert_eq!(requests[3].path, "/api/v2/public/time");
        assert_eq!(requests[4].method, "POST");
        assert_eq!(requests[4].path, "/api/v2/spot/trade/cancel-order");
        assert_eq!(
            requests[4].body,
            json!({"symbol":"BTCUSDT","orderId":"12345"})
        );
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.method == "POST")
                .count(),
            1
        );
        for request in [&requests[1], &requests[2], &requests[4]] {
            assert_bitget_signature(request);
        }
        assert_eq!(outcome.broker_order_id.as_deref(), Some("normal:12345"));
        assert_eq!(outcome.provider_status.as_deref(), Some("LIVE"));
        let safe_result = serde_json::to_string(&outcome).unwrap();
        assert!(!safe_result.contains("synthetic-api-key"));
        assert!(!safe_result.contains("synthetic-api-secret"));
        assert!(!safe_result.contains("synthetic-api-passphrase"));
        drop(requests);

        let duplicate = gateway.dispatch_attempt(
            "bitget-cancel-1",
            |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
            |_, _| panic!("duplicate Bitget cancellation must not cross SUBMITTING"),
            |_, _| {},
            |_, _| Ok(()),
        );
        assert_eq!(duplicate, Ok(()));
        assert_eq!(calls.lock().unwrap().len(), 5);
        gateway.stop();
    }

    #[test]
    fn bitget_cancel_rejection_and_lost_response_are_terminal_without_replay_after_restart() {
        for (suffix, status, body, drop_response, expected_state, expected_error) in [
            (
                "rejected",
                400,
                br#"{"code":"40017","msg":"rejected"}"#.as_slice(),
                false,
                ExecutionAttemptState::Rejected,
                Some("PROVIDER_ORDER_REJECTED"),
            ),
            (
                "unknown",
                200,
                b"".as_slice(),
                true,
                ExecutionAttemptState::UnknownReconciling,
                Some("ORDER_STATUS_UNKNOWN"),
            ),
        ] {
            let attempt_id = format!("bitget-cancel-{suffix}");
            let (url, calls, provider) = fake_bitget_provider(status, body, drop_response);
            let (mut gateway, result, stopped) = run_child_recording_pre_dispatch(
                package(
                    &attempt_id,
                    GatewayDispatchIntent::Cancel(Box::new(bitget_cancel_intent())),
                ),
                url.clone(),
            );
            provider.join().unwrap();
            assert_eq!(*stopped.lock().unwrap(), None);
            let outcome = result.lock().unwrap().clone().unwrap();
            assert_eq!(outcome.state, expected_state, "{outcome:?}");
            assert_eq!(outcome.error_code.as_deref(), expected_error);
            let requests = calls.lock().unwrap();
            assert_eq!(requests.len(), 5, "{requests:#?}");
            assert_eq!(
                requests
                    .iter()
                    .filter(|request| request.method == "POST")
                    .count(),
                1
            );
            for request in [&requests[1], &requests[2], &requests[4]] {
                assert_bitget_signature(request);
            }
            drop(requests);
            gateway.stop();

            let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
            let mut restarted = OrderGatewayHost::new(executable.to_owned(), digest(executable));
            restarted.start().unwrap();
            let replay = restarted.dispatch_attempt(
                &attempt_id,
                |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
                |_, _| panic!("a restarted gateway cannot resubmit a terminal attempt"),
                |_, _| {},
                |_, _| Ok(()),
            );
            assert_eq!(replay, Ok(()));
            assert_eq!(calls.lock().unwrap().len(), 5);
            restarted.stop();
        }
    }

    #[test]
    fn real_child_fails_closed_on_binance_account_mismatch_and_unsupported_order() {
        let order = br#"{"symbol":"BTCUSDT","orderId":12345,"orderListId":-1,"side":"BUY","status":"NEW","origQty":"1","executedQty":"0","cummulativeQuoteQty":"0","updateTime":1788849600000}"#;
        for (attempt_id, remote_id, order_body, expected_reads) in [
            ("binance-account-mismatch", "778", order.as_slice(), 2),
            (
                "binance-oco-order",
                "777",
                br#"{"symbol":"BTCUSDT","orderId":12345,"orderListId":91,"side":"BUY","status":"NEW","origQty":"1","executedQty":"0","cummulativeQuoteQty":"0","updateTime":1788849600000}"#.as_slice(),
                3,
            ),
        ] {
            let (url, calls, provider) = fake_binance_provider(
                expected_reads,
                200,
                b"",
                false,
                remote_id,
                order_body,
            );
            let (mut gateway, result) = run_child(
                package(
                    attempt_id,
                    GatewayDispatchIntent::Cancel(Box::new(binance_cancel_intent())),
                ),
                url,
            );
            provider.join().unwrap();
            let requests = calls.lock().unwrap();
            assert_eq!(requests.len(), expected_reads);
            assert!(requests.iter().all(|request| request.method == "GET"));
            assert_eq!(requests.iter().filter(|request| request.method == "DELETE").count(), 0);
            assert!(result.lock().unwrap().is_none());
            drop(requests);
            gateway.stop();
        }
    }

    #[test]
    fn real_child_preserves_binance_cancel_rejection_and_unknown_without_replay() {
        let order = br#"{"symbol":"BTCUSDT","orderId":12345,"orderListId":-1,"side":"BUY","status":"NEW","origQty":"1","executedQty":"0","cummulativeQuoteQty":"0","updateTime":1788849600000}"#;
        for (attempt_id, status, body, drop_response, expected_state, expected_code) in [
            (
                "binance-cancel-rejected",
                400,
                br#"{"code":-2011,"msg":"Unknown order sent."}"#.as_slice(),
                false,
                ExecutionAttemptState::Rejected,
                Some("PROVIDER_ORDER_REJECTED"),
            ),
            (
                "binance-cancel-unknown",
                200,
                br#"{"symbol":"BTCUSDT","orderId":12345,"orderListId":-1,"status":"CANCELED"}"#
                    .as_slice(),
                true,
                ExecutionAttemptState::UnknownReconciling,
                Some("ORDER_STATUS_UNKNOWN"),
            ),
            (
                "binance-cancel-malformed-ack",
                200,
                br#"{"symbol":"ETHUSDT","orderId":12345,"orderListId":-1,"status":"CANCELED"}"#
                    .as_slice(),
                false,
                ExecutionAttemptState::UnknownReconciling,
                Some("ORDER_STATUS_UNKNOWN"),
            ),
        ] {
            let (url, calls, provider) =
                fake_binance_provider(5, status, body, drop_response, "777", order);
            let (mut gateway, result) = run_child(
                package(
                    attempt_id,
                    GatewayDispatchIntent::Cancel(Box::new(binance_cancel_intent())),
                ),
                url,
            );
            provider.join().unwrap();
            let requests = calls.lock().unwrap();
            assert_eq!(requests.len(), 5);
            assert_eq!(
                requests
                    .iter()
                    .filter(|request| request.method == "DELETE")
                    .count(),
                1
            );
            let outcome = result.lock().unwrap().clone().unwrap();
            assert_eq!(outcome.state, expected_state);
            assert_eq!(outcome.error_code.as_deref(), expected_code);
            drop(requests);
            gateway.stop();
            if expected_state == ExecutionAttemptState::UnknownReconciling {
                let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
                let mut restarted =
                    OrderGatewayHost::new(executable.to_owned(), digest(executable));
                restarted.start().unwrap();
                let replay = restarted.dispatch_attempt(
                    attempt_id,
                    |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
                    |_, _| panic!("unknown Binance cancellation must not cross SUBMITTING again"),
                    |_, _| {},
                    |_, _| Ok(()),
                );
                assert_eq!(replay, Ok(()));
                restarted.stop();
                assert_eq!(calls.lock().unwrap().len(), 5);
            }
        }
    }

    #[test]
    fn real_child_stops_before_submitting_for_changed_terminal_or_malformed_orders() {
        for order_id in ["123457", "123458", "123459"] {
            let (url, calls, provider) = fake_provider(2, 200, b"", false);
            let (mut gateway, result) = run_child(
                package(
                    &format!("cancel-preflight-{order_id}"),
                    GatewayDispatchIntent::Cancel(Box::new(cancel_intent(order_id))),
                ),
                url,
            );
            provider.join().unwrap();
            let requests = calls.lock().unwrap();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].path, "/api/v0/equity/account/summary");
            assert_eq!(requests[1].method, "GET");
            assert_eq!(
                requests[1].path,
                format!("/api/v0/equity/orders/{order_id}")
            );
            assert!(requests.iter().all(|request| request.method == "GET"));
            assert!(result.lock().unwrap().is_none());
            drop(requests);
            gateway.stop();
            assert!(
                calls
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|request| request.method != "DELETE")
            );
        }
    }

    #[test]
    fn real_child_rejects_a_different_live_account_before_reading_or_mutating_the_order() {
        let (url, calls, provider) = fake_provider(1, 200, b"", false);
        let mut package = package(
            "cancel-account-mismatch",
            GatewayDispatchIntent::Cancel(Box::new(cancel_intent("123456"))),
        );
        package.account.data.as_mut().unwrap().remote_account_id = "778".into();
        let (mut gateway, result) = run_child(package, url);
        provider.join().unwrap();
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].path, "/api/v0/equity/account/summary");
        assert!(result.lock().unwrap().is_none());
        assert!(requests.iter().all(|request| request.method == "GET"));
        drop(requests);
        gateway.stop();
    }

    #[test]
    fn real_child_records_place_rejection_after_exactly_one_post() {
        let (url, calls, provider) = fake_provider(2, 400, br#"{"error":"rejected"}"#, false);
        let (mut gateway, result) = run_child(
            package(
                "place-rejected",
                GatewayDispatchIntent::Place(Box::new(proposal())),
            ),
            url,
        );
        provider.join().unwrap();
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.method == "POST")
                .count(),
            1
        );
        assert_eq!(requests[1].path, "/api/v0/equity/orders/market");
        let outcome = result.lock().unwrap().clone().unwrap();
        assert_eq!(outcome.state, ExecutionAttemptState::Rejected);
        assert_eq!(
            outcome.error_code.as_deref(),
            Some("PROVIDER_ORDER_REJECTED")
        );
        gateway.stop();
    }

    #[test]
    fn real_child_keeps_a_lost_place_response_unknown_after_exactly_one_post() {
        let (url, calls, provider) = fake_provider(2, 200, b"", true);
        let (mut gateway, result) = run_child(
            package(
                "place-unknown",
                GatewayDispatchIntent::Place(Box::new(proposal())),
            ),
            url,
        );
        provider.join().unwrap();
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.method == "POST")
                .count(),
            1
        );
        let outcome = result.lock().unwrap().clone().unwrap();
        assert_eq!(outcome.state, ExecutionAttemptState::UnknownReconciling);
        assert_eq!(outcome.error_code.as_deref(), Some("ORDER_STATUS_UNKNOWN"));
        gateway.stop();
    }

    #[test]
    fn real_child_result_persistence_failure_cannot_replay_the_provider_post() {
        let (url, calls, provider) = fake_provider(
            2,
            200,
            br#"{"id":901,"ticker":"AAPL_US_EQ","status":"NEW"}"#,
            false,
        );
        let attempt_id = "place-result-save-failed";
        let (mut gateway, result, dispatch) = run_child_with_persistence(
            package(
                attempt_id,
                GatewayDispatchIntent::Place(Box::new(proposal())),
            ),
            url,
            false,
        );
        assert_eq!(dispatch, Err("WORKSPACE_OPEN_FAILED"));
        assert_eq!(
            result.lock().unwrap().as_ref().unwrap().state,
            ExecutionAttemptState::Accepted
        );
        gateway.stop();

        let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
        let mut restarted = OrderGatewayHost::new(executable.to_owned(), digest(executable));
        restarted.start().unwrap();
        let duplicate = restarted.dispatch_attempt(
            attempt_id,
            |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
            |_, _| panic!("recovered attempt must not cross SUBMITTING"),
            |_, _| {},
            |_, _| Ok(()),
        );
        assert_eq!(duplicate, Ok(()));
        restarted.stop();
        provider.join().unwrap();
        assert_eq!(calls.lock().unwrap().len(), 2);
        assert_eq!(
            calls
                .lock()
                .unwrap()
                .iter()
                .filter(|request| request.method == "POST")
                .count(),
            1
        );
    }

    #[test]
    fn real_child_treats_a_mismatched_place_identity_as_unknown_without_retry() {
        let (url, calls, provider) = fake_provider(
            2,
            200,
            br#"{"id":901,"ticker":"MSFT_US_EQ","status":"NEW"}"#,
            false,
        );
        let (mut gateway, result) = run_child(
            package(
                "place-mismatched",
                GatewayDispatchIntent::Place(Box::new(proposal())),
            ),
            url,
        );
        provider.join().unwrap();
        let requests = calls.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.method == "POST")
                .count(),
            1
        );
        let outcome = result.lock().unwrap().clone().unwrap();
        assert_eq!(outcome.state, ExecutionAttemptState::UnknownReconciling);
        assert_eq!(outcome.broker_order_id, None);
        gateway.stop();
    }

    #[test]
    fn real_child_keeps_cancel_rejection_and_lost_acknowledgement_distinct() {
        for (attempt_id, status, drop_response, expected_state, expected_code) in [
            (
                "cancel-rejected",
                400,
                false,
                ExecutionAttemptState::Rejected,
                Some("PROVIDER_ORDER_REJECTED"),
            ),
            (
                "cancel-unknown",
                200,
                true,
                ExecutionAttemptState::UnknownReconciling,
                Some("ORDER_STATUS_UNKNOWN"),
            ),
        ] {
            let (url, calls, provider) = fake_provider(3, status, b"", drop_response);
            let (mut gateway, result) = run_child(
                package(
                    attempt_id,
                    GatewayDispatchIntent::Cancel(Box::new(CancellationIntent {
                        cancellation_intent_id: format!("intent-{attempt_id}"),
                        intent_hash: format!("sha256:{}", "c".repeat(64)),
                        workspace_id: "integration-test".into(),
                        account_id: "account-1".into(),
                        environment: ExecutionContext::Trading212Live,
                        provider_order_id: "123456".into(),
                        instrument_id: "equity:US:AAPL".into(),
                        symbol: "AAPL_US_EQ".into(),
                        side: "BUY".into(),
                        provider_status: "NEW".into(),
                        quantity: "1".into(),
                        filled_quantity: "0".into(),
                        remaining_quantity: "1".into(),
                        created_at: "2026-09-27T10:00:00Z".into(),
                    })),
                ),
                url,
            );
            provider.join().unwrap();
            let requests = calls.lock().unwrap();
            assert_eq!(requests.len(), 3);
            assert_eq!(requests[0].path, "/api/v0/equity/account/summary");
            assert_eq!(requests[1].path, "/api/v0/equity/orders/123456");
            assert_eq!(
                requests
                    .iter()
                    .filter(|request| request.method == "DELETE")
                    .count(),
                1
            );
            assert_eq!(requests[2].path, "/api/v0/equity/orders/123456");
            let outcome = result.lock().unwrap().clone().unwrap();
            assert_eq!(outcome.state, expected_state);
            assert_eq!(outcome.error_code.as_deref(), expected_code);
            if expected_state == ExecutionAttemptState::Rejected {
                assert_eq!(outcome.broker_order_id.as_deref(), Some("123456"));
            } else {
                assert_eq!(outcome.broker_order_id, None);
            }
            drop(requests);
            gateway.stop();
            if expected_state == ExecutionAttemptState::UnknownReconciling {
                let executable = Path::new(env!("CARGO_BIN_EXE_tradex-order-gateway"));
                let mut restarted =
                    OrderGatewayHost::new(executable.to_owned(), digest(executable));
                restarted.start().unwrap();
                let replay = restarted.dispatch_attempt(
                    attempt_id,
                    |_, _| Err("EXECUTION_DISPATCH_NOT_READY".into()),
                    |_, _| panic!("reopened unknown cancel must not cross SUBMITTING"),
                    |_, _| {},
                    |_, _| Ok(()),
                );
                assert_eq!(replay, Ok(()));
                restarted.stop();
                assert_eq!(calls.lock().unwrap().len(), 3);
            }
        }
    }
}
