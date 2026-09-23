use serde_json::{Value, json};
use tradex::ControlPlane;
use tradex::protocol::Result;
use tradex::provider_io::{CredentialVault, ProviderHttp, ProviderRateLimit};

#[path = "support/provider_fixtures.rs"]
mod fixtures;
use fixtures::*;

fn envelope(command: &str, payload: Value) -> Value {
    json!({"requestId":"provider-test","schemaVersion":1,"command":command,"payload":payload})
}
fn begin(workspace: &Value) -> Value {
    envelope(
        "provider.connect",
        json!({"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Paper research"}),
    )
}
fn query(account: &Value) -> Value {
    json!({"workspaceId":account["workspaceId"],"connectionId":account["connectionId"]})
}
fn mutation(account: &Value) -> Value {
    let mut p = query(account);
    p["expectedStateVersion"] = account["stateVersion"].clone();
    p
}
fn alpaca_order(id: usize, quantity: &str, filled: &str, status: &str) -> Value {
    json!({
        "id":format!("00000000-0000-4000-8000-{id:012}"),
        "client_order_id":format!("manual-{id}"),
        "symbol":"AAPL","asset_class":"us_equity","side":"buy",
        "type":"limit","time_in_force":"day","qty":quantity,
        "filled_qty":filled,"status":status,
        "submitted_at":"2026-09-23T10:00:00Z","created_at":"2026-09-23T10:00:00Z"
    })
}
fn alpaca_fill(id: usize, order_id: &str, quantity: &str, price: &str) -> Value {
    json!({
        "id":format!("20260923100000000::00000000-0000-4000-8000-{id:012}"),
        "order_id":order_id,"symbol":"AAPL","side":"buy","qty":quantity,
        "price":price,"transaction_time":"2026-09-23T10:00:00Z"
    })
}
fn refresh_alpaca_orders(workspace: &Value, account: &Value) -> Value {
    envelope(
        "alpaca.paper.orders.refresh",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"]
        }),
    )
}
fn review_alpaca_order(workspace: &Value, account: &Value, order_id: &str) -> Value {
    envelope(
        "alpaca.paper.order.review",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "providerOrderId":order_id
        }),
    )
}
fn cancel_alpaca_order(
    workspace: &Value,
    account: &Value,
    book: &Value,
    order_id: &str,
    key: &str,
) -> Value {
    envelope(
        "alpaca.paper.order.cancel",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "providerOrderId":order_id,"expectedBookStateVersion":book["stateVersion"],
            "idempotencyKey":key,"confirmed":true
        }),
    )
}
fn execute(
    cp: &mut ControlPlane,
    input: Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let job = cp.prepare_provider(&input).unwrap().unwrap();
    let outcome = job.run(
        vault,
        |_| credentials(),
        http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, vault) {
        cp.record_credential_cleanup(&job, cleanup);
    }
    reply
}

fn execute_main(
    cp: &mut ControlPlane,
    input: Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let job = cp
        .prepare_provider_for(&input, "main")
        .unwrap_or_else(|error| panic!("{}: {error:?}", input["command"]))
        .unwrap();
    let outcome = job.run(
        vault,
        |_| credentials(),
        http,
        || cp.provider_job_current(&job),
    );
    cp.complete_provider(&job, outcome)
}

fn command(cp: &mut ControlPlane, command: &str, payload: Value) -> Value {
    cp.dispatch(
        json!({"requestId":"provider-test","schemaVersion":1,"command":command,"payload":payload}),
    )
}

fn command_main(cp: &mut ControlPlane, command: &str, payload: Value) -> Value {
    cp.dispatch_with_events(envelope(command, payload), "main", None)
}

fn lifecycle(vault: &impl CredentialVault) {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let http = Http::default();
    let connected = execute(&mut cp, begin(&workspace), vault, &http);
    assert_eq!(connected["ok"], true, "{connected}");
    let pending = connected["data"].clone();
    assert_eq!(pending["connectionState"], "REVIEW_REQUIRED");
    assert_eq!(pending["data"]["balances"][0]["available"], "1000.25");
    assert_eq!(pending["data"]["openOrders"][0]["notional"], "10.5");
    assert_eq!(pending["data"]["buyingPower"], "1100.9876543210123456789");
    assert_eq!(pending["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(pending["health"]["privateStream"], "NOT_CONFIGURED");
    assert_eq!(pending["health"]["executionEligibility"], "BLOCKED");
    let mut confirm = mutation(&pending);
    confirm["step"] = "confirm".into();
    confirm["acknowledgeUnverified"] = false.into();
    assert_eq!(
        command(&mut cp, "provider.connect", confirm.clone())["error"]["code"],
        "PROVIDER_REVIEW_REQUIRED"
    );
    confirm["acknowledgeUnverified"] = true.into();
    let confirmed = command(&mut cp, "provider.connect", confirm.clone());
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    assert_eq!(confirmed["data"]["connectionState"], "CONNECTED");
    assert_eq!(
        command(&mut cp, "provider.connect", confirm)["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );

    let snapshot = command(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"account","aggregateId":pending["connectionId"]}),
    );
    assert_eq!(snapshot["data"]["lastSequence"], 3);
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = events.clone();
    let ack=cp.dispatch_with_events(envelope("domain.subscribe",json!({"aggregateType":"account","aggregateId":pending["connectionId"],"afterSequence":0})),"test-window",Some(std::sync::Arc::new(move|event|{sink.lock().unwrap().push(serde_json::to_value(event).unwrap());true})));
    assert_eq!(ack["data"]["replayedCount"], 3);
    assert_eq!(events.lock().unwrap()[2]["payload"], confirmed["data"]);

    http.fail.set(true);
    let failed = execute(
        &mut cp,
        envelope("account.refresh", mutation(&confirmed["data"])),
        vault,
        &http,
    );
    assert_eq!(failed["error"]["code"], "PROVIDER_UNAVAILABLE");
    let stale = command(&mut cp, "account.get", query(&pending))["data"].clone();
    assert_eq!(stale["data"], pending["data"]);
    assert_eq!(stale["lastSuccessfulSync"], pending["lastSuccessfulSync"]);
    assert_eq!(stale["health"]["connection"], "ERROR");
    assert_eq!(events.lock().unwrap().last().unwrap()["payload"], stale);
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    command(&mut cp, "workspace.open", json!({}));
    let restored = command(&mut cp, "account.get", query(&pending))["data"].clone();
    assert_eq!(restored["connectionId"], pending["connectionId"]);
    assert_eq!(restored["health"]["connection"], "STALE");
    assert_eq!(restored["health"]["credential"], "UNCHECKED");
    http.fail.set(false);
    let refreshed = execute(
        &mut cp,
        envelope("provider.probe", mutation(&restored)),
        vault,
        &http,
    );
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    let disconnected = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&refreshed["data"])),
        vault,
        &http,
    );
    assert_eq!(disconnected["data"]["connectionState"], "DISCONNECTED");
    assert_eq!(disconnected["data"]["health"]["credential"], "MISSING");
    let reference = serde_json::from_value::<tradex::providers::AccountConnection>(pending.clone())
        .unwrap()
        .credential_ref();
    assert!(
        vault.get(&reference).is_err(),
        "Disconnect must remove the actual Keychain item"
    );
    let serialized = serde_json::to_vec(&json!([
        pending,
        confirmed,
        stale,
        restored,
        refreshed,
        disconnected,
        *events.lock().unwrap()
    ]))
    .unwrap();
    for secret in [KEY, SECRET] {
        assert!(
            !serialized
                .windows(secret.len())
                .any(|w| w == secret.as_bytes())
        );
        for entry in std::fs::read_dir(folder.path()).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }
            let bytes = std::fs::read(path).unwrap();
            assert!(!bytes.windows(secret.len()).any(|w| w == secret.as_bytes()));
        }
    }
}

#[test]
fn trading212_demo_order_book_reads_are_scoped_bounded_and_event_persisted() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    http.trading212_identity.set(9007199254741101);
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let never_synced = command(
        &mut cp,
        "trading212.demo.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(never_synced["ok"], true, "{never_synced}");
    assert_eq!(never_synced["data"]["book"]["status"], "NEVER_SYNCED");
    assert_eq!(
        never_synced["data"]["book"]["remoteAccountId"],
        "9007199254741101"
    );
    assert!(
        never_synced["data"]["book"]["orders"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let proposal = trading212_demo_proposal(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "LIMIT",
        json!({"type":"BASE","value":"5"}),
        "DAY",
    );
    let submitted = execute_main(
        &mut cp,
        trading212_submit_request(&workspace, &account, &proposal, "t212-book-attempt"),
        &vault,
        &http,
    );
    assert_eq!(submitted["ok"], true, "{submitted}");
    let attempt_id = submitted["data"]["attemptId"].clone();
    let provider_order_id = submitted["data"]["providerOrderId"].clone();
    assert_eq!(provider_order_id, "9007199254740995");

    *http.trading212_order_list.borrow_mut() = Some(vec![
        serde_json::from_str::<Value>(
            r#"{
            "id":9007199254740995,"ticker":"AAPL_US_EQ","side":"BUY",
            "type":"LIMIT","timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
            "filledQuantity":1.25,"filledValue":25.1234567890123456789,"currency":"GBP",
            "status":"PARTIALLY_FILLED","createdAt":"2026-09-23T10:00:00Z"
        }"#,
        )
        .unwrap(),
    ]);
    *http.trading212_rate_limit.borrow_mut() = Some(ProviderRateLimit {
        remaining: Some(0),
        reset_at: Some("2099-01-01T00:00:00Z".into()),
    });
    let refresh = |action: &str, order_id: Option<&str>| {
        let mut payload = json!({
            "workspaceId":workspace,
            "connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "action":action
        });
        if let Some(order_id) = order_id {
            payload["providerOrderId"] = json!(order_id);
        }
        envelope("trading212.demo.orders.refresh", payload)
    };
    let pending = execute_main(&mut cp, refresh("PENDING", None), &vault, &http);
    assert_eq!(pending["ok"], true, "{pending}");
    assert_eq!(pending["data"]["status"], "CURRENT");
    assert_eq!(pending["data"]["remoteAccountId"], "9007199254741101");
    assert_eq!(pending["data"]["environment"], "DEMO");
    assert_eq!(
        pending["data"]["orders"][0]["providerOrderId"],
        "9007199254740995"
    );
    assert_eq!(pending["data"]["orders"][0]["filledQuantity"], "1.25");
    assert_eq!(
        pending["data"]["orders"][0]["filledValue"],
        "25.1234567890123456789"
    );
    assert_eq!(pending["data"]["orders"][0]["currency"], "GBP");
    assert_eq!(pending["data"]["orders"][0]["remainingQuantity"], "3.75");
    assert_eq!(pending["data"]["orders"][0]["origin"], "TRADE_X");
    assert_eq!(pending["data"]["orders"][0]["attemptId"], attempt_id);
    assert_eq!(
        pending["data"]["rateLimits"]["pendingOrdersRetryAt"],
        "2099-01-01T00:00:00Z"
    );
    let saved_book = command(
        &mut cp,
        "trading212.demo.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(saved_book["data"]["book"]["status"], "STALE");
    assert_eq!(
        saved_book["data"]["book"]["stateVersion"],
        pending["data"]["stateVersion"]
    );
    assert_eq!(
        saved_book["data"]["book"]["orders"][0]["filledQuantity"],
        "1.25"
    );

    let unknown_detail = cp
        .prepare_provider_for(&refresh("DETAIL", Some("55")), "main")
        .err()
        .unwrap();
    assert_eq!(unknown_detail.code, "ORDER_STATUS_UNKNOWN");
    let details: Value = serde_json::from_str(
        r#"{
        "id":9007199254740995,"ticker":"AAPL_US_EQ","side":"BUY",
        "type":"LIMIT","timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
        "filledQuantity":2.5,"filledValue":50.9876543210987654321,
        "status":"PARTIALLY_FILLED","createdAt":"2026-09-23T10:00:00Z"
    }"#,
    )
    .unwrap();
    http.trading212_order_details
        .borrow_mut()
        .insert("9007199254740995".into(), details);
    let detailed = execute_main(
        &mut cp,
        refresh("DETAIL", Some("9007199254740995")),
        &vault,
        &http,
    );
    assert_eq!(detailed["ok"], true, "{detailed}");
    assert_eq!(detailed["data"]["orders"][0]["filledQuantity"], "2.5");
    assert_eq!(
        detailed["data"]["orders"][0]["filledValue"],
        "50.9876543210987654321"
    );
    assert_eq!(detailed["data"]["orders"][0]["remainingQuantity"], "2.5");

    let history_order: Value = serde_json::from_str(
        r#"{
        "id":8001,"ticker":"MSFT_US_EQ","side":"SELL","type":"MARKET",
        "timeInForce":"DAY","strategy":"QUANTITY","quantity":-2,
        "filledQuantity":-2,"filledValue":201.234567890123456789,"currency":"GBP",
        "status":"FILLED","createdAt":"2026-09-22T12:00:00Z"
    }"#,
    )
    .unwrap();
    http.trading212_history_pages.borrow_mut().insert(
        "/api/v0/equity/history/orders?limit=50".into(),
        json!({"items":[history_order],"nextPagePath":"/api/v0/equity/history/orders?limit=50&cursor=123"}),
    );
    let next_history_order: Value = serde_json::from_str(
        r#"{
        "id":8002,"ticker":"NVDA_US_EQ","side":"BUY","type":"LIMIT",
        "timeInForce":"GOOD_TILL_CANCEL","strategy":"QUANTITY","quantity":3,
        "filledQuantity":0,"filledValue":0,
        "status":"CANCELLED","createdAt":"2026-09-22T11:00:00Z"
    }"#,
    )
    .unwrap();
    http.trading212_history_pages.borrow_mut().insert(
        "/api/v0/equity/history/orders?limit=50&cursor=123".into(),
        json!({"items":[next_history_order],"nextPagePath":null}),
    );
    let history = execute_main(&mut cp, refresh("HISTORY", None), &vault, &http);
    assert_eq!(history["ok"], true, "{history}");
    assert_eq!(history["data"]["historyPageCount"], 1);
    assert_eq!(
        history["data"]["nextPagePath"],
        "/api/v0/equity/history/orders?limit=50&cursor=123"
    );
    let external = history["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "8001")
        .unwrap();
    assert_eq!(external["origin"], "EXTERNAL");
    assert_eq!(external["normalizedStatus"], "FILLED");
    assert_eq!(external["remainingQuantity"], "0");
    assert_eq!(external["filledValue"], "201.234567890123456789");
    assert_eq!(external["currency"], "GBP");

    std::thread::sleep(std::time::Duration::from_millis(10_100));
    let second_history_page = execute_main(&mut cp, refresh("HISTORY", None), &vault, &http);
    assert_eq!(second_history_page["ok"], true, "{second_history_page}");
    assert_eq!(
        second_history_page["data"]["historyPageCount"], 2,
        "{second_history_page}"
    );
    assert_eq!(second_history_page["data"]["historyComplete"], true);
    assert_eq!(second_history_page["data"]["nextPagePath"], Value::Null);
    assert!(
        second_history_page["data"]["orders"]
            .as_array()
            .unwrap()
            .iter()
            .any(|order| order["providerOrderId"] == "8002")
    );
    assert!(
        http.calls
            .borrow()
            .iter()
            .any(|path| { path.ends_with("/api/v0/equity/history/orders?limit=50&cursor=123") })
    );

    let filled_details: Value = serde_json::from_str(
        r#"{
        "id":9007199254740995,"ticker":"AAPL_US_EQ","side":"BUY",
        "type":"LIMIT","timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
        "filledQuantity":5,"filledValue":101.234567890123456789,
        "status":"FILLED","createdAt":"2026-09-23T10:00:00Z"
    }"#,
    )
    .unwrap();
    http.trading212_order_details
        .borrow_mut()
        .insert("9007199254740995".into(), filled_details);
    let fully_filled = execute_main(
        &mut cp,
        refresh("DETAIL", Some("9007199254740995")),
        &vault,
        &http,
    );
    assert_eq!(fully_filled["ok"], true, "{fully_filled}");
    let trade_x_order = fully_filled["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(trade_x_order["providerStatus"], "FILLED");
    assert_eq!(trade_x_order["pending"], false);
    assert_eq!(trade_x_order["filledQuantity"], "5");
    assert_eq!(trade_x_order["filledValue"], "101.234567890123456789");
    assert_eq!(trade_x_order["remainingQuantity"], "0");
    assert_eq!(trade_x_order["origin"], "TRADE_X");
    assert_eq!(trade_x_order["attemptId"], attempt_id);
    assert_eq!(
        cp.prepare_provider_for(&refresh("DETAIL", Some("9007199254740995")), "main")
            .err()
            .unwrap()
            .code,
        "ORDER_STATUS_UNKNOWN"
    );

    let history_calls_before_retry = http
        .calls
        .borrow()
        .iter()
        .filter(|path| path.contains("/api/v0/equity/history/orders?limit=50"))
        .count();
    let rate_limited = execute_main(&mut cp, refresh("HISTORY", None), &vault, &http);
    assert_eq!(rate_limited["ok"], true, "{rate_limited}");
    assert_eq!(rate_limited["data"]["status"], "DEGRADED");
    assert_eq!(rate_limited["data"]["reason"], "PROVIDER_RATE_LIMITED");
    assert_eq!(rate_limited["data"]["orders"].as_array().unwrap().len(), 3);
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|path| path.contains("/api/v0/equity/history/orders?limit=50"))
            .count(),
        history_calls_before_retry,
        "a refresh within the endpoint window must not reach the provider"
    );

    let book_snapshot = command(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"trading212-demo-order-book","aggregateId":account["connectionId"]}),
    );
    assert_eq!(book_snapshot["ok"], true, "{book_snapshot}");
    assert_eq!(book_snapshot["data"]["lastSequence"], 6);
    drop(cp);
    let mut reopened = ControlPlane::new(path);
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let durable = command(
        &mut reopened,
        "trading212.demo.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(durable["data"]["book"]["status"], "DEGRADED");
    assert_eq!(durable["data"]["book"]["orders"][0]["filledQuantity"], "5");
}

#[test]
fn trading212_demo_submit_is_persisted_before_io_and_duplicate_never_posts_twice() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let proposal = trading212_demo_proposal(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "MARKET",
        json!({"type":"BASE","value":"1.234567890123456789"}),
        "DAY",
    );
    let request = trading212_submit_request(&workspace, &account, &proposal, "t212-attempt-1");

    let job = cp.prepare_provider_for(&request, "main").unwrap();
    assert!(
        job.is_some(),
        "the confirmed Demo proposal must produce a provider job"
    );
    let job = job.unwrap();
    let before_io = command(
        &mut cp,
        "trading212.demo.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(before_io["data"]["attempt"]["state"], "SUBMITTING");
    assert!(http.trading212_posts.borrow().is_empty());

    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "ACKNOWLEDGED");
    assert_eq!(reply["data"]["providerOrderId"], "9007199254740995");
    assert_eq!(reply["data"]["providerStatus"], "NEW");
    assert_eq!(http.trading212_posts.borrow().len(), 1);
    let (path, body) = &http.trading212_posts.borrow()[0];
    assert_eq!(path, "/api/v0/equity/orders/market");
    assert_eq!(body["ticker"], "AAPL_US_EQ");
    assert_eq!(body["quantity"].to_string(), "1.234567890123456789");
    assert_eq!(body["extendedHours"], false);
    assert!(body.get("idempotencyKey").is_none());
    assert!(body.get("clientOrderId").is_none());

    let duplicate = trading212_submit_request(&workspace, &account, &proposal, "t212-attempt-2");
    assert!(
        cp.prepare_provider_for(&duplicate, "main")
            .unwrap()
            .is_none()
    );
    assert_eq!(http.trading212_posts.borrow().len(), 1);
    let saved = command(
        &mut cp,
        "trading212.demo.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(saved["data"]["attempt"]["state"], "ACKNOWLEDGED");
    let duplicate_reply = command_main(
        &mut cp,
        "trading212.demo.order.submit",
        request["payload"].clone(),
    );
    assert_eq!(duplicate_reply["ok"], true, "{duplicate_reply}");
    assert_eq!(duplicate_reply["data"]["state"], "ACKNOWLEDGED");
    let mut mismatched = request["payload"].clone();
    mismatched["proposalHash"] = format!("sha256:{}", "0".repeat(64)).into();
    assert_eq!(
        command_main(&mut cp, "trading212.demo.order.submit", mismatched)["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    assert_eq!(http.trading212_posts.borrow().len(), 1);
}

#[test]
fn trading212_demo_sell_limit_preserves_signed_decimal_and_good_till_cancel() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let proposal = trading212_demo_proposal(
        &mut cp,
        &workspace,
        &account,
        "SELL",
        "LIMIT",
        json!({"type":"BASE","value":"0.000000000000000001"}),
        "GTC",
    );
    let reply = execute_main(
        &mut cp,
        trading212_submit_request(&workspace, &account, &proposal, "t212-limit-sell"),
        &vault,
        &http,
    );
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "ACKNOWLEDGED");
    let (path, body) = &http.trading212_posts.borrow()[0];
    assert_eq!(path, "/api/v0/equity/orders/limit");
    assert_eq!(body["quantity"].to_string(), "-0.000000000000000001");
    assert_eq!(body["limitPrice"].to_string(), "150.123456789012345678");
    assert_eq!(body["timeValidity"], "GOOD_TILL_CANCEL");
    assert_eq!(body.as_object().unwrap().len(), 4);
}

#[test]
fn trading212_demo_timeout_freezes_account_and_never_posts_again() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let first = trading212_demo_proposal(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "MARKET",
        json!({"type":"BASE","value":"1"}),
        "DAY",
    );
    http.trading212_post_timeout.set(true);
    let reply = execute_main(
        &mut cp,
        trading212_submit_request(&workspace, &account, &first, "t212-timeout"),
        &vault,
        &http,
    );
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "UNKNOWN_RECONCILING");
    assert_eq!(reply["data"]["errorCode"], "ORDER_STATUS_UNKNOWN");
    assert_eq!(http.trading212_posts.borrow().len(), 1);
    let duplicate_ipc = command_main(
        &mut cp,
        "trading212.demo.order.submit",
        trading212_submit_request(&workspace, &account, &first, "t212-timeout-ipc-retry")["payload"].clone(),
    );
    assert_eq!(duplicate_ipc["ok"], true, "{duplicate_ipc}");
    assert_eq!(duplicate_ipc["data"]["state"], "UNKNOWN_RECONCILING");
    assert!(
        cp.prepare_provider_for(
            &trading212_submit_request(&workspace, &account, &first, "t212-timeout-retry"),
            "main"
        )
        .unwrap()
        .is_none()
    );
    let second = trading212_demo_proposal(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "MARKET",
        json!({"type":"BASE","value":"2"}),
        "DAY",
    );
    let error = match cp.prepare_provider_for(
        &trading212_submit_request(&workspace, &account, &second, "t212-account-freeze"),
        "main",
    ) {
        Err(error) => error,
        Ok(_) => panic!("an unresolved T212 attempt must freeze later account orders"),
    };
    assert_eq!(error.code, "ORDER_STATUS_UNKNOWN");
    assert_eq!(http.trading212_posts.borrow().len(), 1);
}

#[test]
fn trading212_demo_reopen_recovers_interrupted_submit_as_query_only_unknown() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let proposal = trading212_demo_proposal(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "MARKET",
        json!({"type":"BASE","value":"1"}),
        "DAY",
    );
    let job = cp
        .prepare_provider_for(
            &trading212_submit_request(&workspace, &account, &proposal, "t212-interrupted"),
            "main",
        )
        .unwrap()
        .unwrap();
    let before_restart = command(
        &mut cp,
        "trading212.demo.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    )["data"]["attempt"]
        .clone();
    assert_eq!(before_restart["state"], "SUBMITTING");
    drop(job);
    drop(cp);

    let mut reopened = ControlPlane::new(path);
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let recovered = command(
        &mut reopened,
        "trading212.demo.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    )["data"]["attempt"]
        .clone();
    assert_eq!(recovered["state"], "UNKNOWN_RECONCILING");
    assert_eq!(recovered["attemptId"], before_restart["attemptId"]);
    assert_eq!(http.trading212_posts.borrow().len(), 0);

    let snapshot = command(
        &mut reopened,
        "domain.snapshot",
        json!({"aggregateType":"trading212-demo-order-attempt","aggregateId":recovered["attemptId"]}),
    );
    assert_eq!(snapshot["ok"], true, "{snapshot}");
    assert_eq!(snapshot["data"]["lastSequence"], 2);
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = events.clone();
    let ack = reopened.dispatch_with_events(
        envelope(
            "domain.subscribe",
            json!({"aggregateType":"trading212-demo-order-attempt","aggregateId":recovered["attemptId"],"afterSequence":0}),
        ),
        "main",
        Some(std::sync::Arc::new(move |event| {
            sink.lock().unwrap().push(serde_json::to_value(event).unwrap());
            true
        })),
    );
    assert_eq!(ack["ok"], true, "{ack}");
    assert_eq!(ack["data"]["replayedCount"], 2);
    assert_eq!(
        events.lock().unwrap()[1]["eventType"],
        "trading212.demo.order.attempt.changed"
    );
    assert_eq!(
        events.lock().unwrap()[1]["payload"]["state"],
        "UNKNOWN_RECONCILING"
    );
}

#[test]
fn trading212_demo_rejected_or_mismatched_response_never_allows_a_resubmit() {
    for (status, response_body, expected_state) in [
        (Some(400), None, "REJECTED"),
        (Some(401), None, "REJECTED"),
        (Some(403), None, "REJECTED"),
        (Some(429), None, "REJECTED"),
        (Some(408), None, "UNKNOWN_RECONCILING"),
        (
            None,
            Some(br#"{"#.to_vec()),
            "UNKNOWN_RECONCILING",
        ),
        (
            None,
            Some(br#"{"id":9007199254740995,"ticker":"MSFT_US_EQ","quantity":1,"side":"BUY","strategy":"QUANTITY","type":"MARKET","timeInForce":"DAY","extendedHours":false}"#.to_vec()),
            "UNKNOWN_RECONCILING",
        ),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let account = connected_trading212(&mut cp, &vault, &http, &workspace);
        let proposal = trading212_demo_proposal(
            &mut cp,
            &workspace,
            &account,
            "BUY",
            "MARKET",
            json!({"type":"BASE","value":"1"}),
            "DAY",
        );
        http.trading212_post_status.set(status);
        *http.trading212_post_response_body.borrow_mut() = response_body;
        let request = trading212_submit_request(&workspace, &account, &proposal, "t212-response-case");
        let reply = execute_main(&mut cp, request.clone(), &vault, &http);
        assert_eq!(reply["data"]["state"], expected_state, "{reply}");
        assert_eq!(http.trading212_posts.borrow().len(), 1);
        assert!(cp.prepare_provider_for(&request, "main").unwrap().is_none());
        assert_eq!(http.trading212_posts.borrow().len(), 1);
    }
}

#[test]
fn trading212_demo_refuses_to_submit_after_remote_account_identity_changes() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    http.trading212_identity.set(9007199254740994);
    let proposal = trading212_demo_proposal(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "MARKET",
        json!({"type":"BASE","value":"1"}),
        "DAY",
    );
    let reply = execute_main(
        &mut cp,
        trading212_submit_request(&workspace, &account, &proposal, "t212-identity-change"),
        &vault,
        &http,
    );
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "REJECTED");
    assert_eq!(reply["data"]["errorCode"], "PROVIDER_IDENTITY_CHANGED");
    assert!(http.trading212_posts.borrow().is_empty());
}

#[test]
fn trading212_demo_rejects_unsupported_order_fields_before_persisting_or_sending() {
    for (quantity_type, time_in_force, expected_error) in [
        ("BASE", "GTC", "ORDER_CAPABILITY_UNSUPPORTED"),
        ("QUOTE", "DAY", "ORDER_PROPOSAL_NOT_ELIGIBLE"),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let account = connected_trading212(&mut cp, &vault, &http, &workspace);
        let proposal = trading212_demo_proposal(
            &mut cp,
            &workspace,
            &account,
            "BUY",
            "MARKET",
            json!({"type":quantity_type,"value":"1"}),
            time_in_force,
        );
        let error = match cp.prepare_provider_for(
            &trading212_submit_request(&workspace, &account, &proposal, "t212-unsupported"),
            "main",
        ) {
            Err(error) => error,
            Ok(_) => panic!("unsupported Trading 212 order was accepted"),
        };
        assert_eq!(error.code, expected_error);
        assert!(http.trading212_posts.borrow().is_empty());
        let saved = command(
            &mut cp,
            "trading212.demo.order.attempt.get",
            json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
        );
        assert_eq!(saved["data"]["attempt"], Value::Null);
    }
}

fn connected_alpaca(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
) -> Value {
    let tested = execute(cp, begin(workspace), vault, http);
    assert_eq!(tested["ok"], true, "{tested}");
    let mut confirm = mutation(&tested["data"]);
    confirm["step"] = "confirm".into();
    confirm["acknowledgeUnverified"] = true.into();
    let confirmed = command(cp, "provider.connect", confirm);
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    confirmed["data"].clone()
}

fn alpaca_paper_proposal(cp: &mut ControlPlane, workspace: &Value, account: &Value) -> Value {
    alpaca_paper_proposal_with_order(
        cp,
        workspace,
        account,
        "BUY",
        "BASE",
        "1",
        ("MARKET", "DAY"),
    )
}

fn alpaca_paper_proposal_with_order(
    cp: &mut ControlPlane,
    workspace: &Value,
    account: &Value,
    side: &str,
    quantity_type: &str,
    quantity: &str,
    (order_type, time_in_force): (&str, &str),
) -> Value {
    let mut fields = json!({
        "accountId":account["connectionId"],"venue":"XNAS",
        "environment":"ALPACA_PAPER","instrumentId":"equity:US:AAPL",
        "side":side,"orderType":order_type,
        "quantity":{"type":quantity_type,"value":quantity},"timeInForce":time_in_force
    });
    if order_type == "LIMIT" {
        fields["limitPrice"] = "100".into();
    }
    let saved = command(
        cp,
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":fields}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let generated = command(
        cp,
        "trade.generate_proposal",
        json!({
            "workspaceId":workspace,"draftId":saved["data"]["draftId"],
            "expectedDraftVersion":1
        }),
    );
    assert_eq!(generated["ok"], true, "{generated}");
    let selected = command(
        cp,
        "trade.proposal.get",
        json!({"workspaceId":workspace,"proposalId":generated["data"]["proposalId"]}),
    );
    assert_eq!(selected["ok"], true, "{selected}");
    selected["data"].clone()
}

fn alpaca_submit_request(workspace: &Value, account: &Value, proposal: &Value) -> Value {
    envelope(
        "alpaca.paper.order.submit",
        json!({
            "workspaceId":workspace,
            "connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "proposalId":proposal["proposalId"],
            "expectedProposalStateVersion":proposal["stateVersion"],
            "proposalHash":proposal["proposalHash"],
            "idempotencyKey":"paper-attempt-qa-1",
            "confirmedPaperOrder":true
        }),
    )
}

fn connected_trading212(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
) -> Value {
    let tested = execute(
        cp,
        envelope(
            "provider.connect",
            json!({"step":"test","workspaceId":workspace,"providerId":"trading212","environment":"DEMO","label":"Demo test"}),
        ),
        vault,
        http,
    );
    assert_eq!(tested["ok"], true, "{tested}");
    let mut confirm = mutation(&tested["data"]);
    confirm["step"] = "confirm".into();
    confirm["acknowledgeUnverified"] = true.into();
    let confirmed = command(cp, "provider.connect", confirm);
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    confirmed["data"].clone()
}

fn trading212_demo_proposal(
    cp: &mut ControlPlane,
    workspace: &Value,
    account: &Value,
    side: &str,
    order_type: &str,
    quantity: Value,
    time_in_force: &str,
) -> Value {
    let mut fields = json!({
        "accountId":account["connectionId"],"venue":"XNAS",
        "environment":"TRADING212_DEMO","instrumentId":"equity:US:AAPL",
        "side":side,"orderType":order_type,
        "quantity":quantity,"timeInForce":time_in_force
    });
    if order_type == "LIMIT" {
        fields["limitPrice"] = "150.123456789012345678".into();
    }
    let saved = command(
        cp,
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":fields}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let generated = command(
        cp,
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":saved["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(generated["ok"], true, "{generated}");
    let selected = command(
        cp,
        "trade.proposal.get",
        json!({"workspaceId":workspace,"proposalId":generated["data"]["proposalId"]}),
    );
    assert_eq!(selected["ok"], true, "{selected}");
    selected["data"].clone()
}

fn trading212_submit_request(
    workspace: &Value,
    account: &Value,
    proposal: &Value,
    key: &str,
) -> Value {
    envelope(
        "trading212.demo.order.submit",
        json!({
            "workspaceId":workspace,
            "connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "proposalId":proposal["proposalId"],
            "expectedProposalStateVersion":proposal["stateVersion"],
            "proposalHash":proposal["proposalHash"],
            "idempotencyKey":key,
            "confirmedDemoOrder":true
        }),
    )
}

const T212_TEST_ORDER_ID: &str = "9007199254740995";

fn trading212_test_order(status: &str, filled: &str, filled_value: &str) -> Value {
    let filled = serde_json::from_str::<Value>(filled).unwrap();
    let filled_value = serde_json::from_str::<Value>(filled_value).unwrap();
    json!({
        "id":9007199254740995u64,"ticker":"AAPL_US_EQ","side":"BUY",
        "type":"LIMIT","timeInForce":"DAY","strategy":"QUANTITY","quantity":5,
        "filledQuantity":filled,"filledValue":filled_value,"currency":"GBP",
        "status":status,"createdAt":"2026-09-23T10:00:00Z"
    })
}

fn refresh_trading212_orders(
    cp: &mut ControlPlane,
    workspace: &Value,
    account: &Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    action: &str,
    order_id: Option<&str>,
) -> Value {
    let mut payload = json!({
        "workspaceId":workspace,"connectionId":account["connectionId"],
        "expectedConnectionStateVersion":account["stateVersion"],"action":action
    });
    if let Some(order_id) = order_id {
        payload["providerOrderId"] = json!(order_id);
    }
    execute_main(
        cp,
        envelope("trading212.demo.orders.refresh", payload),
        vault,
        http,
    )
}

fn ready_trading212_cancel_order(
    cp: &mut ControlPlane,
    workspace: &Value,
    account: &Value,
    vault: &impl CredentialVault,
    http: &Http,
) -> Value {
    let order = trading212_test_order("PARTIALLY_FILLED", "1", "25");
    *http.trading212_order_list.borrow_mut() = Some(vec![order.clone()]);
    http.trading212_order_details
        .borrow_mut()
        .insert(T212_TEST_ORDER_ID.into(), order);
    let pending = refresh_trading212_orders(cp, workspace, account, vault, http, "PENDING", None);
    assert_eq!(pending["ok"], true, "{pending}");
    let reviewed = refresh_trading212_orders(
        cp,
        workspace,
        account,
        vault,
        http,
        "DETAIL",
        Some(T212_TEST_ORDER_ID),
    );
    assert_eq!(reviewed["ok"], true, "{reviewed}");
    reviewed["data"].clone()
}

fn trading212_cancel_request(
    workspace: &Value,
    account: &Value,
    book: &Value,
    confirmed: bool,
    key: &str,
) -> Value {
    envelope(
        "trading212.demo.orders.cancel",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "providerOrderId":T212_TEST_ORDER_ID,
            "expectedBookStateVersion":book["stateVersion"],
            "idempotencyKey":key,"confirmed":confirmed
        }),
    )
}

fn edit_trading212_book_projection(
    workspace_path: &std::path::Path,
    connection_id: &Value,
    edit: impl FnOnce(&mut Value),
) {
    let connection = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    let (workspace_id, raw): (String, String) = connection
        .query_row(
            "SELECT workspace_id,projection FROM trading212_demo_order_books WHERE connection_id=?1",
            [connection_id.as_str().unwrap()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let mut book: Value = serde_json::from_str(&raw).unwrap();
    edit(&mut book);
    connection
        .execute(
            "UPDATE trading212_demo_order_books SET projection=?1 WHERE workspace_id=?2 AND connection_id=?3",
            rusqlite::params![serde_json::to_string(&book).unwrap(), workspace_id, connection_id.as_str().unwrap()],
        )
        .unwrap();
}

fn saved_trading212_order(workspace_path: &std::path::Path, connection_id: &Value) -> Value {
    let connection = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    let raw: String = connection
        .query_row(
            "SELECT projection FROM trading212_demo_order_books WHERE connection_id=?1",
            [connection_id.as_str().unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    serde_json::from_str::<Value>(&raw).unwrap()["orders"][0].clone()
}

#[test]
fn trading212_demo_cancel_is_persisted_once_and_a_racing_fill_wins() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    http.trading212_identity.set(9007199254741301);
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let book = ready_trading212_cancel_order(&mut cp, &workspace, &account, &vault, &http);
    assert_eq!(book["status"], "CURRENT");
    assert_eq!(book["orders"][0]["cancelState"], "NONE");

    let request = trading212_cancel_request(&workspace, &account, &book, true, "t212-cancel-1");
    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    assert_eq!(
        saved_trading212_order(folder.path(), &account["connectionId"])["cancelState"],
        "SUBMITTING",
        "the cancel intent must commit before provider I/O"
    );
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let accepted = cp.complete_provider(&job, outcome);
    assert_eq!(accepted["ok"], true, "{accepted}");
    assert_eq!(
        http.trading212_delete_calls.borrow().as_slice(),
        ["/api/v0/equity/orders/9007199254740995"]
    );
    assert_eq!(accepted["data"]["status"], "STALE");
    assert_eq!(
        accepted["data"]["orders"][0]["providerStatus"],
        "PARTIALLY_FILLED"
    );
    assert_eq!(accepted["data"]["orders"][0]["cancelState"], "PENDING");
    assert_eq!(accepted["data"]["orders"][0]["filledQuantity"], "1");

    let duplicate = trading212_cancel_request(
        &workspace,
        &account,
        &accepted["data"],
        true,
        "t212-cancel-2",
    );
    assert!(cp.prepare_provider_for(&duplicate, "main").is_err());
    assert_eq!(http.trading212_delete_calls.borrow().len(), 1);

    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let still_partial = refresh_trading212_orders(
        &mut cp,
        &workspace,
        &account,
        &vault,
        &http,
        "DETAIL",
        Some(T212_TEST_ORDER_ID),
    );
    assert_eq!(still_partial["ok"], true, "{still_partial}");
    assert_eq!(still_partial["data"]["status"], "CURRENT");
    assert_eq!(still_partial["data"]["orders"][0]["cancelState"], "PENDING");
    let duplicate_pending = trading212_cancel_request(
        &workspace,
        &account,
        &still_partial["data"],
        true,
        "t212-cancel-3",
    );
    assert_eq!(
        cp.prepare_provider_for(&duplicate_pending, "main")
            .err()
            .unwrap()
            .code,
        "STATE_VERSION_CONFLICT"
    );
    assert_eq!(http.trading212_delete_calls.borrow().len(), 1);

    http.trading212_order_details.borrow_mut().insert(
        T212_TEST_ORDER_ID.into(),
        trading212_test_order("FILLED", "5", "125"),
    );
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let reconciled = refresh_trading212_orders(
        &mut cp,
        &workspace,
        &account,
        &vault,
        &http,
        "DETAIL",
        Some(T212_TEST_ORDER_ID),
    );
    assert_eq!(reconciled["ok"], true, "{reconciled}");
    let order = &reconciled["data"]["orders"][0];
    assert_eq!(order["providerStatus"], "FILLED");
    assert_eq!(order["filledQuantity"], "5");
    assert_eq!(order["remainingQuantity"], "0");
    assert_eq!(order["pending"], false);
    assert_eq!(order["cancelState"], "NONE");
    assert_eq!(http.trading212_delete_calls.borrow().len(), 1);
    let terminal_cancel = trading212_cancel_request(
        &workspace,
        &account,
        &reconciled["data"],
        true,
        "t212-cancel-3",
    );
    assert_eq!(
        cp.prepare_provider_for(&terminal_cancel, "main")
            .err()
            .unwrap()
            .code,
        "ORDER_NOT_CANCELABLE"
    );
    assert_eq!(http.trading212_delete_calls.borrow().len(), 1);
}

#[test]
fn trading212_demo_cancel_rejects_unconfirmed_expired_stale_and_unknown_review() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    http.trading212_identity.set(9007199254741302);
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let book = ready_trading212_cancel_order(&mut cp, &workspace, &account, &vault, &http);

    let unconfirmed = trading212_cancel_request(&workspace, &account, &book, false, "missing");
    assert_eq!(
        cp.prepare_provider_for(&unconfirmed, "main")
            .err()
            .unwrap()
            .code,
        "ORDER_CONFIRMATION_REQUIRED"
    );

    let observed_at = book["orders"][0]["observedAt"].clone();
    edit_trading212_book_projection(folder.path(), &account["connectionId"], |persisted| {
        persisted["orders"][0]["observedAt"] = json!("2000-01-01T00:00:00Z");
    });
    let expired = trading212_cancel_request(&workspace, &account, &book, true, "expired");
    assert_eq!(
        cp.prepare_provider_for(&expired, "main")
            .err()
            .unwrap()
            .code,
        "ORDER_CONFIRMATION_EXPIRED"
    );

    edit_trading212_book_projection(folder.path(), &account["connectionId"], |persisted| {
        persisted["orders"][0]["observedAt"] = observed_at;
        persisted["status"] = json!("STALE");
    });
    let stale = trading212_cancel_request(&workspace, &account, &book, true, "stale");
    assert_eq!(
        cp.prepare_provider_for(&stale, "main").err().unwrap().code,
        "STATE_VERSION_CONFLICT"
    );

    edit_trading212_book_projection(folder.path(), &account["connectionId"], |persisted| {
        persisted["status"] = json!("CURRENT");
    });
    http.trading212_order_details.borrow_mut().insert(
        T212_TEST_ORDER_ID.into(),
        trading212_test_order("FUTURE_STATUS", "1", "25"),
    );
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let unknown = refresh_trading212_orders(
        &mut cp,
        &workspace,
        &account,
        &vault,
        &http,
        "DETAIL",
        Some(T212_TEST_ORDER_ID),
    );
    assert_eq!(unknown["ok"], true, "{unknown}");
    assert_eq!(unknown["data"]["status"], "DEGRADED");
    assert_eq!(unknown["data"]["reason"], "PROVIDER_DATA_INCOMPLETE");
    let cancel_unknown =
        trading212_cancel_request(&workspace, &account, &unknown["data"], true, "unknown");
    assert_eq!(
        cp.prepare_provider_for(&cancel_unknown, "main")
            .err()
            .unwrap()
            .code,
        "STATE_VERSION_CONFLICT"
    );
    assert!(http.trading212_delete_calls.borrow().is_empty());
}

#[test]
fn trading212_demo_cancel_account_change_and_timeout_never_repeat_delete() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    http.trading212_identity.set(9007199254741303);
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let book = ready_trading212_cancel_order(&mut cp, &workspace, &account, &vault, &http);

    http.trading212_identity.set(9007199254741304);
    let changed = execute_main(
        &mut cp,
        trading212_cancel_request(&workspace, &account, &book, true, "changed-account"),
        &vault,
        &http,
    );
    assert_eq!(changed["ok"], true, "{changed}");
    assert_eq!(changed["data"]["status"], "DEGRADED");
    assert_eq!(changed["data"]["reason"], "PROVIDER_REVIEW_REQUIRED");
    assert_eq!(changed["data"]["orders"][0]["cancelState"], "NONE");
    assert!(http.trading212_delete_calls.borrow().is_empty());

    let timeout_folder = tempfile::tempdir().unwrap();
    let mut timeout_cp = ControlPlane::new(timeout_folder.path().to_path_buf());
    let timeout_workspace =
        command(&mut timeout_cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let timeout_http = Http::default();
    timeout_http.trading212_identity.set(9007199254741305);
    let timeout_account =
        connected_trading212(&mut timeout_cp, &vault, &timeout_http, &timeout_workspace);
    let timeout_book = ready_trading212_cancel_order(
        &mut timeout_cp,
        &timeout_workspace,
        &timeout_account,
        &vault,
        &timeout_http,
    );
    timeout_http.trading212_delete_timeout.set(true);
    let timeout = execute_main(
        &mut timeout_cp,
        trading212_cancel_request(
            &timeout_workspace,
            &timeout_account,
            &timeout_book,
            true,
            "timeout",
        ),
        &vault,
        &timeout_http,
    );
    assert_eq!(timeout["ok"], true, "{timeout}");
    assert_eq!(timeout["data"]["orders"][0]["cancelState"], "PENDING");
    assert_eq!(
        timeout["data"]["orders"][0]["cancelError"],
        "ORDER_CANCEL_STATUS_UNKNOWN"
    );
    assert_eq!(timeout_http.trading212_delete_calls.borrow().len(), 1);
    let retry = trading212_cancel_request(
        &timeout_workspace,
        &timeout_account,
        &timeout["data"],
        true,
        "timeout-retry",
    );
    assert!(timeout_cp.prepare_provider_for(&retry, "main").is_err());
    assert_eq!(timeout_http.trading212_delete_calls.borrow().len(), 1);
}

#[test]
fn trading212_demo_reopen_recovers_interrupted_cancel_as_unknown_without_delete() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    http.trading212_identity.set(9007199254741306);
    let account = connected_trading212(&mut cp, &vault, &http, &workspace);
    let book = ready_trading212_cancel_order(&mut cp, &workspace, &account, &vault, &http);
    let request = trading212_cancel_request(&workspace, &account, &book, true, "interrupted");
    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    assert_eq!(
        saved_trading212_order(&path, &account["connectionId"])["cancelState"],
        "SUBMITTING"
    );
    drop(job);
    drop(cp);

    let mut reopened = ControlPlane::new(path.clone());
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let recovered = saved_trading212_order(&path, &account["connectionId"]);
    assert_eq!(recovered["cancelState"], "PENDING");
    assert_eq!(recovered["cancelError"], "ORDER_CANCEL_STATUS_UNKNOWN");
    assert_eq!(http.trading212_delete_calls.borrow().len(), 0);
}

#[test]
fn alpaca_paper_order_submit_is_persisted_before_io_and_never_posts_twice() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    assert_eq!(account["data"]["currency"], "USD");
    assert_eq!(account["data"]["balances"][0]["available"], "1000.25");
    assert_eq!(account["data"]["balances"][0]["total"], "1200.55");
    assert_eq!(account["data"]["buyingPower"], "1100.9876543210123456789");
    let proposal = alpaca_paper_proposal(&mut cp, &workspace, &account);
    let request = alpaca_submit_request(&workspace, &account, &proposal);

    let forbidden = match cp.prepare_provider_for(&request, "research") {
        Err(error) => error.code,
        Ok(_) => panic!("research consumers cannot submit provider orders"),
    };
    assert_eq!(forbidden, "ORDER_SUBMIT_FORBIDDEN");

    let mut unconfirmed = request.clone();
    unconfirmed["payload"]["confirmedPaperOrder"] = json!(false);
    let unconfirmed_error = match cp.prepare_provider_for(&unconfirmed, "main") {
        Err(error) => error.code,
        Ok(_) => panic!("unconfirmed Alpaca Paper orders must be rejected"),
    };
    assert_eq!(unconfirmed_error, "IPC_PAYLOAD_INVALID");
    let absent_attempt = command(
        &mut cp,
        "alpaca.paper.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(absent_attempt["data"]["attempt"], Value::Null);
    assert!(http.alpaca_posts.borrow().is_empty());

    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    let attempt = command(
        &mut cp,
        "alpaca.paper.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    )["data"]["attempt"]
        .clone();
    assert_eq!(attempt["state"], "SUBMITTING");
    assert!(attempt["clientOrderId"].as_str().unwrap().len() <= 128);
    assert!(
        http.alpaca_posts.borrow().is_empty(),
        "POST must happen after durable attempt creation"
    );

    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "ACKNOWLEDGED");
    assert_eq!(reply["data"]["providerStatus"], "accepted");
    assert!(
        reply["data"].get("fill").is_none(),
        "HTTP acknowledgement is not fill evidence"
    );
    assert_eq!(http.alpaca_posts.borrow().len(), 1);
    let sent = http.alpaca_posts.borrow()[0].clone();
    assert_eq!(sent["client_order_id"], attempt["clientOrderId"]);
    assert_eq!(sent["symbol"], "AAPL");
    assert_eq!(sent["qty"], "1");
    assert_eq!(sent["time_in_force"], "day");
    assert!(
        http.calls
            .borrow()
            .iter()
            .any(|url| url.starts_with("https://paper-api.alpaca.markets/"))
    );
    assert!(
        http.calls
            .borrow()
            .iter()
            .all(|url| !url.contains("live-api"))
    );

    assert!(cp.prepare_provider_for(&request, "main").unwrap().is_none());
    let duplicate = cp.dispatch_with_events(request.clone(), "main", None);
    assert_eq!(duplicate["data"]["attemptId"], attempt["attemptId"]);
    assert_eq!(
        http.alpaca_posts.borrow().len(),
        1,
        "same immutable proposal must not create another POST"
    );

    let snapshot = command(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"alpaca-paper-order-attempt","aggregateId":attempt["attemptId"]}),
    );
    assert_eq!(snapshot["ok"], true, "{snapshot}");
    assert_eq!(snapshot["data"]["lastSequence"], 2);
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = events.clone();
    let ack = cp.dispatch_with_events(
        envelope(
            "domain.subscribe",
            json!({"aggregateType":"alpaca-paper-order-attempt","aggregateId":attempt["attemptId"],"afterSequence":0}),
        ),
        "main",
        Some(std::sync::Arc::new(move |event| {
            sink.lock().unwrap().push(serde_json::to_value(event).unwrap());
            true
        })),
    );
    assert_eq!(ack["data"]["replayedCount"], 2);
    assert_eq!(
        events.lock().unwrap()[1]["eventType"],
        "alpaca.paper.order.attempt.changed"
    );
    let encoded =
        serde_json::to_string(&json!([reply, duplicate, *events.lock().unwrap()])).unwrap();
    assert!(!encoded.contains(KEY));
    assert!(!encoded.contains(SECRET));
}

#[test]
fn alpaca_paper_timeout_stays_unknown_after_empty_lookup_and_reconcile_never_reposts() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let proposal = alpaca_paper_proposal(&mut cp, &workspace, &account);
    let request = alpaca_submit_request(&workspace, &account, &proposal);
    http.alpaca_post_timeout.set(true);
    http.alpaca_lookup_misses.set(1);

    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "UNKNOWN_RECONCILING");
    assert_eq!(reply["data"]["errorCode"], "ORDER_STATUS_UNKNOWN");
    assert_eq!(http.alpaca_posts.borrow().len(), 1);
    assert!(cp.prepare_provider_for(&request, "main").unwrap().is_none());
    assert_eq!(http.alpaca_posts.borrow().len(), 1);

    http.alpaca_post_timeout.set(false);
    let reconcile = envelope(
        "alpaca.paper.order.reconcile",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "proposalId":proposal["proposalId"]
        }),
    );
    let reconcile_job = cp
        .prepare_provider_for(&reconcile, "main")
        .unwrap()
        .unwrap();
    let outcome = reconcile_job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&reconcile_job),
    );
    let recovered = cp.complete_provider(&reconcile_job, outcome);
    assert_eq!(recovered["ok"], true, "{recovered}");
    assert_eq!(recovered["data"]["state"], "ACKNOWLEDGED");
    assert_eq!(
        recovered["data"]["providerOrderId"],
        "18c65e3e-feb0-4576-99e2-36e6f047d84d"
    );
    assert_eq!(
        http.alpaca_posts.borrow().len(),
        1,
        "reconciliation is query-only"
    );
}

#[test]
fn alpaca_paper_reconcile_never_reads_an_order_from_a_different_account() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let mut http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let proposal = alpaca_paper_proposal(&mut cp, &workspace, &account);
    let request = alpaca_submit_request(&workspace, &account, &proposal);
    http.alpaca_post_timeout.set(true);
    http.alpaca_lookup_misses.set(1);

    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["data"]["state"], "UNKNOWN_RECONCILING");
    let lookup_count = http
        .calls
        .borrow()
        .iter()
        .filter(|path| path.contains("/v2/orders:by_client_order_id?client_order_id="))
        .count();

    http.identity = "b1a3c22f-4ad7-47aa-912b-cda43b22ce44".into();
    let reconcile = envelope(
        "alpaca.paper.order.reconcile",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "proposalId":proposal["proposalId"]
        }),
    );
    let reconcile_job = cp
        .prepare_provider_for(&reconcile, "main")
        .unwrap()
        .unwrap();
    let outcome = reconcile_job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&reconcile_job),
    );
    let unchanged = cp.complete_provider(&reconcile_job, outcome);
    assert_eq!(unchanged["data"]["state"], "UNKNOWN_RECONCILING");
    assert_eq!(unchanged["data"]["errorCode"], "PROVIDER_IDENTITY_CHANGED");
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|path| path.contains("/v2/orders:by_client_order_id?client_order_id="))
            .count(),
        lookup_count,
        "a changed provider account must be rejected before querying its orders"
    );
    assert_eq!(http.alpaca_posts.borrow().len(), 1);
}

#[test]
fn alpaca_paper_reopen_recovers_submitting_attempt_to_query_only_unknown_state() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let proposal = alpaca_paper_proposal(&mut cp, &workspace, &account);
    let request = alpaca_submit_request(&workspace, &account, &proposal);
    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    let before_restart = command(
        &mut cp,
        "alpaca.paper.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    )["data"]["attempt"]
        .clone();
    assert_eq!(before_restart["state"], "SUBMITTING");
    assert!(http.alpaca_posts.borrow().is_empty());
    drop(job);
    drop(cp);

    let mut reopened = ControlPlane::new(path);
    let opened = command(&mut reopened, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    assert_eq!(opened["data"]["workspaceId"], workspace);
    let recovered = command(
        &mut reopened,
        "alpaca.paper.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    )["data"]["attempt"]
        .clone();
    assert_eq!(recovered["state"], "UNKNOWN_RECONCILING");
    assert_eq!(recovered["clientOrderId"], before_restart["clientOrderId"]);
    let current_account = command(
        &mut reopened,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    )["data"]
        .clone();

    let reconcile = envelope(
        "alpaca.paper.order.reconcile",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":current_account["stateVersion"],
            "proposalId":proposal["proposalId"]
        }),
    );
    let reconcile_job = reopened
        .prepare_provider_for(&reconcile, "main")
        .unwrap()
        .unwrap();
    let outcome = reconcile_job.run(
        &vault,
        |_| credentials(),
        &http,
        || reopened.provider_job_current(&reconcile_job),
    );
    let still_unknown = reopened.complete_provider(&reconcile_job, outcome);
    assert_eq!(still_unknown["data"]["state"], "UNKNOWN_RECONCILING");
    assert_eq!(
        still_unknown["data"]["clientOrderId"],
        before_restart["clientOrderId"]
    );
    assert!(
        http.alpaca_posts.borrow().is_empty(),
        "restart recovery must never resubmit"
    );
}

#[test]
fn alpaca_paper_rejects_notional_above_fresh_buying_power_before_order_io() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let proposal = alpaca_paper_proposal_with_order(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "QUOTE",
        "1200",
        ("MARKET", "DAY"),
    );
    let request = alpaca_submit_request(&workspace, &account, &proposal);
    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "REJECTED");
    assert_eq!(
        reply["data"]["errorCode"],
        "ORDER_BUYING_POWER_INSUFFICIENT"
    );
    assert_eq!(http.alpaca_posts.borrow().len(), 0);
    assert!(
        !http
            .calls
            .borrow()
            .iter()
            .any(|path| path.starts_with("/v2/assets/"))
    );
}

#[test]
fn alpaca_paper_definitive_http_rejections_are_persisted_without_resubmission() {
    for (status, error_code) in [
        (400, "PROVIDER_ORDER_REJECTED"),
        (401, "PROVIDER_AUTH_FAILED"),
        (403, "PROVIDER_ORDER_REJECTED"),
        (422, "PROVIDER_ORDER_REJECTED"),
        (429, "PROVIDER_RATE_LIMITED"),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
        let proposal = alpaca_paper_proposal(&mut cp, &workspace, &account);
        let request = alpaca_submit_request(&workspace, &account, &proposal);
        http.alpaca_post_status.set(Some(status));

        let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
        let outcome = job.run(
            &vault,
            |_| credentials(),
            &http,
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, outcome);
        assert_eq!(reply["ok"], true, "HTTP {status}: {reply}");
        assert_eq!(reply["data"]["state"], "REJECTED", "HTTP {status}");
        assert_eq!(
            reply["data"]["errorCode"], error_code,
            "HTTP {status}: {reply}"
        );
        assert_eq!(http.alpaca_posts.borrow().len(), 1, "HTTP {status}");
        assert!(cp.prepare_provider_for(&request, "main").unwrap().is_none());
        assert_eq!(
            http.alpaca_posts.borrow().len(),
            1,
            "replaying a definitively rejected Proposal cannot issue a second POST"
        );
    }
}

#[test]
fn alpaca_paper_classifies_known_403_reasons_without_persisting_provider_text() {
    for (message, side, error_code) in [
        (
            "Buying power or shares is not sufficient.",
            "BUY",
            "ORDER_BUYING_POWER_INSUFFICIENT",
        ),
        (
            "Buying power or shares is not sufficient.",
            "SELL",
            "ORDER_INSUFFICIENT_POSITION",
        ),
        (
            "account not authorized to trade",
            "BUY",
            "PROVIDER_PERMISSION_BLOCKED",
        ),
        (
            "potential wash trade detected",
            "BUY",
            "PROVIDER_ORDER_REJECTED",
        ),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
        let proposal = alpaca_paper_proposal_with_order(
            &mut cp,
            &workspace,
            &account,
            side,
            "BASE",
            "1",
            ("MARKET", "DAY"),
        );
        let request = alpaca_submit_request(&workspace, &account, &proposal);
        http.alpaca_post_status.set(Some(403));
        *http.alpaca_post_error_body.borrow_mut() =
            Some(serde_json::to_vec(&json!({"code":40310000,"message":message})).unwrap());

        let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
        let outcome = job.run(
            &vault,
            |_| credentials(),
            &http,
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, outcome);
        assert_eq!(reply["ok"], true, "{reply}");
        assert_eq!(reply["data"]["state"], "REJECTED", "{reply}");
        assert_eq!(reply["data"]["errorCode"], error_code, "{message}: {reply}");
        assert!(!reply.to_string().contains(message));
    }
}

#[test]
fn alpaca_paper_ioc_and_fok_without_verified_account_capability_never_post() {
    for time_in_force in ["IOC", "FOK"] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
        let proposal = alpaca_paper_proposal_with_order(
            &mut cp,
            &workspace,
            &account,
            "BUY",
            "BASE",
            "1",
            ("LIMIT", time_in_force),
        );
        let request = alpaca_submit_request(&workspace, &account, &proposal);
        let error = match cp.prepare_provider_for(&request, "main") {
            Err(error) => error,
            Ok(_) => panic!("unverified {time_in_force} capability must fail closed"),
        };
        assert_eq!(
            error.code, "ORDER_CAPABILITY_UNSUPPORTED",
            "{time_in_force}: {error:?}"
        );
        assert!(http.alpaca_posts.borrow().is_empty(), "{time_in_force}");
        assert!(
            !http
                .calls
                .borrow()
                .iter()
                .any(|url| url.ends_with("/v2/assets/AAPL")),
            "unsupported TIF must be rejected before asset lookup: {time_in_force}"
        );
    }
}

#[test]
fn alpaca_paper_fractional_equity_orders_require_market_day() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let proposal = alpaca_paper_proposal_with_order(
        &mut cp,
        &workspace,
        &account,
        "BUY",
        "BASE",
        "1.25",
        ("LIMIT", "DAY"),
    );
    let request = alpaca_submit_request(&workspace, &account, &proposal);
    let calls_before_submit = http.calls.borrow().len();
    let error = match cp.prepare_provider_for(&request, "main") {
        Err(error) => error,
        Ok(_) => panic!("fractional Alpaca limit orders must fail before provider I/O"),
    };
    assert_eq!(error.code, "ORDER_CAPABILITY_UNSUPPORTED");
    assert_eq!(http.calls.borrow().len(), calls_before_submit);
    assert!(http.alpaca_posts.borrow().is_empty());
}

#[test]
fn alpaca_paper_sell_cannot_open_or_increase_a_short_position() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    *http.alpaca_position.borrow_mut() = Some(json!({
        "asset_id":"b0b6dd9d-8b9b-48a9-ba46-b9d54906e15b",
        "symbol":"AAPL","asset_class":"us_equity","side":"long",
        "qty":"1","qty_available":"0.5"
    }));
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let proposal = alpaca_paper_proposal_with_order(
        &mut cp,
        &workspace,
        &account,
        "SELL",
        "BASE",
        "1",
        ("MARKET", "DAY"),
    );
    let request = alpaca_submit_request(&workspace, &account, &proposal);
    let job = cp.prepare_provider_for(&request, "main").unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["data"]["state"], "REJECTED");
    assert_eq!(reply["data"]["errorCode"], "ORDER_INSUFFICIENT_POSITION");
    assert!(http.alpaca_posts.borrow().is_empty());
}

#[test]
fn public_connection_lifecycle_preserves_scope_events_decimal_values_and_secret_boundaries() {
    lifecycle(&Vault::default());
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit native OS Keychain integration; creates and deletes disposable synthetic credentials"]
fn native_keychain_roundtrip_drives_the_real_connection_lifecycle() {
    lifecycle(&tradex::provider_io::NativeVault);
}

#[test]
fn disconnected_or_switched_workspaces_reject_late_results_and_cleanup_can_be_retried() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().join("one"));
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let job = cp.prepare_provider(&begin(&workspace)).unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let pending = serde_json::to_value(job.connection()).unwrap();
    vault.fail_remove.set(true);
    let disconnected = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&pending)),
        &vault,
        &http,
    );
    assert_eq!(
        disconnected["data"]["health"]["credential"],
        "DELETE_PENDING"
    );
    assert_eq!(
        cp.complete_provider(&job, outcome)["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    vault.fail_remove.set(false);
    let removed = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&disconnected["data"])),
        &vault,
        &http,
    );
    assert_eq!(removed["data"]["health"]["credential"], "MISSING");
    assert!(vault.present.borrow().is_empty());

    let late = cp.prepare_provider(&begin(&workspace)).unwrap().unwrap();
    let outcome = late.run(&vault, |_| credentials(), &http, || true);
    command(
        &mut cp,
        "workspace.open",
        json!({"path":folder.path().join("two")}),
    );
    assert_eq!(
        command(&mut cp, "runtime.status", json!({}))["data"]["modelAvailable"],
        false
    );
    let result = cp.complete_provider(&late, outcome);
    assert_eq!(result["error"]["code"], "STATE_VERSION_CONFLICT");
    if let Some(cleanup) = late.cleanup_after_failed_commit(&result, &vault) {
        cp.record_credential_cleanup(&late, cleanup);
    }
    assert!(vault.present.borrow().is_empty());
}

#[test]
fn duplicate_and_interrupted_connections_keep_owned_credentials_recoverable() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let first = execute(&mut cp, begin(&workspace), &vault, &http)["data"].clone();
    let duplicate = execute(&mut cp, begin(&workspace), &vault, &http);
    assert_eq!(duplicate["error"]["code"], "PROVIDER_ALREADY_CONNECTED");
    assert_eq!(
        command(&mut cp, "account.get", query(&first))["data"],
        first
    );
    assert_eq!(vault.present.borrow().len(), 1);
    let list = command(&mut cp, "account.list", json!({"workspaceId":workspace}));
    let abandoned = list["data"]["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["connectionId"] != first["connectionId"])
        .unwrap();
    assert_eq!(abandoned["connectionState"], "DISCONNECTED");
    assert_eq!(abandoned["health"]["credential"], "MISSING");

    let interrupted = cp.prepare_provider(&begin(&workspace)).unwrap().unwrap();
    let _outcome = interrupted.run(&vault, |_| credentials(), &http, || true);
    let pending = serde_json::to_value(interrupted.connection()).unwrap();
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    command(&mut cp, "workspace.open", json!({}));
    let restored = command(&mut cp, "account.get", query(&pending))["data"].clone();
    assert_eq!(restored["connectionState"], "DISCONNECTED");
    assert_eq!(restored["health"]["credential"], "DELETE_PENDING");
    assert!(
        cp.prepare_provider(&envelope("provider.probe", mutation(&restored)))
            .is_err()
    );
    let removed = execute(
        &mut cp,
        envelope("provider.disconnect", mutation(&restored)),
        &vault,
        &http,
    );
    assert_eq!(removed["data"]["health"]["credential"], "MISSING");
    assert_eq!(
        vault.present.borrow().len(),
        1,
        "Cleanup must retain the original connection's credential"
    );
}

struct Override {
    base: Http,
    path: &'static str,
    body: Vec<u8>,
}

struct Fault(&'static str);
impl ProviderHttp for Fault {
    fn get(
        &self,
        _: tradex::provider_io::ProviderEndpoint,
        _: &str,
        _: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        Err(tradex::protocol::TradeXError::new(self.0))
    }
}

#[test]
fn failed_initial_tests_remove_their_credential_and_keep_failed_cleanup_blocked_after_restart() {
    for code in [
        "PROVIDER_AUTH_FAILED",
        "PROVIDER_UNAVAILABLE",
        "PROVIDER_RATE_LIMITED",
    ] {
        for cleanup_fails in [false, true] {
            let folder = tempfile::tempdir().unwrap();
            let mut cp = ControlPlane::new(folder.path().to_path_buf());
            let workspace =
                command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
            let vault = Vault::default();
            vault.fail_remove.set(cleanup_fails);
            let result = execute(&mut cp, begin(&workspace), &vault, &Fault(code));
            assert_eq!(result["error"]["code"], code);
            let failed = command(&mut cp, "account.list", json!({"workspaceId":workspace}))["data"]
                ["accounts"][0]
                .clone();
            assert_eq!(failed["connectionState"], "FAILED");
            assert!(failed["lastSuccessfulSync"].is_null());
            assert!(failed["data"].is_null());
            assert_eq!(
                failed["health"]["credential"],
                if cleanup_fails {
                    "DELETE_PENDING"
                } else {
                    "MISSING"
                }
            );
            assert!(
                cp.prepare_provider(&envelope("provider.probe", mutation(&failed)))
                    .is_err()
            );
            drop(cp);
            let mut cp = ControlPlane::new(folder.path().to_path_buf());
            command(&mut cp, "workspace.open", json!({}));
            let restored = command(&mut cp, "account.get", query(&failed))["data"].clone();
            assert_eq!(
                restored["health"]["credential"],
                failed["health"]["credential"]
            );
            assert!(
                cp.prepare_provider(&envelope("provider.probe", mutation(&restored)))
                    .is_err()
            );
            vault.fail_remove.set(false);
            let cleaned = execute(
                &mut cp,
                envelope("provider.disconnect", mutation(&restored)),
                &vault,
                &Http::default(),
            );
            assert_eq!(cleaned["data"]["health"]["credential"], "MISSING");
            assert!(vault.present.borrow().is_empty());
        }
    }
}
impl ProviderHttp for Override {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        let normal = self.base.get(
            endpoint,
            if path.contains("&after_order_id=") {
                "/v2/orders?status=open&limit=500&direction=asc&nested=false"
            } else {
                path
            },
            headers,
        )?;
        Ok(if path == self.path {
            self.body.clone()
        } else {
            normal
        })
    }
}

#[test]
fn provider_faults_cannot_publish_unvalidated_or_cross_account_observations() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let result = execute(&mut cp, begin(&workspace), &vault, &Http::default());
    let original = result["data"].clone();
    let mut current = original.clone();
    let wrong = Http {
        identity: "04d1f763-caaa-43d0-a06f-cb20dd65d145".into(),
        ..Http::default()
    };
    let result = execute(
        &mut cp,
        envelope("account.refresh", mutation(&current)),
        &vault,
        &wrong,
    );
    assert_eq!(result["error"]["code"], "PROVIDER_IDENTITY_CHANGED");
    assert_eq!(
        wrong.calls.borrow().len(),
        1,
        "Stop reading when identity differs"
    );
    current = command(&mut cp, "account.get", query(&current))["data"].clone();
    for body in [
        b"not json".to_vec(),
        vec![b' '; 2 * 1024 * 1024 + 1],
        format!("{{\"reflected\":\"{SECRET}\"}}").into_bytes(),
        format!(
            "[{{\"symbol\":\"{}\",\"qty\":\"1\"}}]",
            KEY.replace('-', "\\u002d")
        )
        .into_bytes(),
        serde_json::to_vec(&json!([{"symbol":"AAPL","qty":"NaN"}])).unwrap(),
    ] {
        let http = Override {
            base: Http::default(),
            path: "/v2/positions",
            body,
        };
        let result = execute(
            &mut cp,
            envelope("provider.probe", mutation(&current)),
            &vault,
            &http,
        );
        assert_eq!(
            result["error"]["code"], "PROVIDER_RESPONSE_INVALID",
            "{result}"
        );
        current = command(&mut cp, "account.get", query(&current))["data"].clone();
        assert_eq!(current["data"], original["data"]);
        assert_eq!(
            current["lastSuccessfulSync"],
            original["lastSuccessfulSync"]
        );
        assert_ne!(current["health"]["connection"], "ONLINE");
    }
    let successful = execute(
        &mut cp,
        envelope("provider.probe", mutation(&current)),
        &vault,
        &Http::default(),
    );
    assert_eq!(successful["ok"], true);
    let job = cp
        .prepare_provider(&envelope("account.refresh", mutation(&successful["data"])))
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &Http::default(),
        || true,
    );
    let fault = rusqlite::Connection::open(folder.path().join("workspace.sqlite3")).unwrap();
    fault.execute_batch("CREATE TRIGGER fail_account_event BEFORE INSERT ON outbox WHEN NEW.aggregate_type='account' BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    let failed = cp.complete_provider(&job, outcome);
    assert_eq!(failed["error"]["code"], "WORKSPACE_OPEN_FAILED");
    assert_eq!(
        command(&mut cp, "account.get", query(&current))["data"],
        successful["data"],
        "Projection and outbox roll back together"
    );
    fault
        .execute_batch("DROP TRIGGER fail_account_event")
        .unwrap();
    let reference = job.connection().credential_ref();
    vault.remove(&reference).unwrap();
    let missing = execute(
        &mut cp,
        envelope("provider.probe", mutation(&successful["data"])),
        &vault,
        &Http::default(),
    );
    assert_eq!(missing["error"]["code"], "CREDENTIAL_UNAVAILABLE");
}

struct Pages {
    base: Http,
    repeated: bool,
    calls: std::cell::Cell<usize>,
}
impl ProviderHttp for Pages {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        if !path.starts_with("/v2/orders?") {
            return self.base.get(endpoint, path, headers);
        }
        self.calls.set(self.calls.get() + 1);
        let start = if path.contains("&after_order_id=") {
            assert!(path.ends_with("00000000-0000-4000-8000-000000000500"));
            if self.repeated { 0 } else { 500 }
        } else {
            0
        };
        let count = if start == 0 { 500 } else { 1 };
        let orders=(start+1..start+count+1).map(|n|json!({"id":format!("00000000-0000-4000-8000-{n:012}"),"symbol":"AAPL","side":"buy","qty":"1","filled_qty":"0","status":"new"})).collect::<Vec<_>>();
        Ok(serde_json::to_vec(&orders).unwrap())
    }
}

#[test]
fn order_id_pagination_is_complete_and_repeated_pages_fail_closed() {
    for repeated in [false, true] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let http = Pages {
            base: Http::default(),
            repeated,
            calls: std::cell::Cell::new(0),
        };
        let result = execute(&mut cp, begin(&workspace), &Vault::default(), &http);
        assert_eq!(http.calls.get(), 2);
        if repeated {
            assert_eq!(result["error"]["code"], "PROVIDER_DATA_INCOMPLETE");
        } else {
            assert_eq!(
                result["data"]["data"]["openOrders"]
                    .as_array()
                    .unwrap()
                    .len(),
                501
            );
        }
    }
    let http = tradex::provider_io::BrokerHttp::default();
    assert_eq!(
        http.get(
            tradex::provider_io::ProviderEndpoint::AlpacaPaper,
            "https://example.invalid/",
            reqwest::header::HeaderMap::new()
        )
        .unwrap_err()
        .code,
        "PROVIDER_UNSUPPORTED"
    );
    for value in [
        json!(1.2),
        json!("1e3"),
        json!("NaN"),
        json!("1."),
        json!("+1"),
        json!("1.2.3"),
    ] {
        assert!(tradex::provider_io::decimal(&value).is_err());
    }
    assert_eq!(
        tradex::provider_io::decimal(&json!("-000.000")).unwrap(),
        "0"
    );
}

#[test]
fn provider_schema_precedes_native_entry_and_never_accepts_renderer_secrets() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let schema = command(
        &mut cp,
        "provider.get_schema",
        json!({"providerId":"alpaca","environment":"PAPER"}),
    );
    assert_eq!(schema["ok"], true, "{schema}");
    let fields = schema["data"]["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 2);
    assert!(
        fields
            .iter()
            .all(|f| f["secret"] == true && f["required"] == true)
    );
    assert_eq!(
        command(
            &mut cp,
            "provider.get_schema",
            json!({"providerId":"alpaca","environment":"LIVE"})
        )["error"]["code"],
        "PROVIDER_UNSUPPORTED"
    );
    let mut input = json!({"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Paper research"});
    input["apiKey"] = "synthetic-renderer-secret-must-be-rejected".into();
    assert_eq!(
        command(&mut cp, "provider.connect", input)["error"]["code"],
        "IPC_PAYLOAD_INVALID"
    );
    let accounts = command(&mut cp, "account.list", json!({"workspaceId":workspace}));
    assert_eq!(accounts["data"]["accounts"].as_array().unwrap().len(), 1);
    assert_eq!(accounts["data"]["accounts"][0]["providerId"], "local-paper");
}

#[test]
fn alpaca_order_book_reads_max_complete_pages_exactly_and_survives_reopen() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let orders = (1..=500)
        .map(|id| {
            alpaca_order(
                id,
                if id == 1 { "3.5000" } else { "1" },
                if id == 1 { "1.2500" } else { "0" },
                if id == 1 { "partially_filled" } else { "new" },
            )
        })
        .collect::<Vec<_>>();
    let fills = (1..=1000)
        .map(|id| {
            alpaca_fill(
                id,
                &format!("00000000-0000-4000-8000-{:012}", ((id - 1) % 500) + 1),
                "0.1",
                "100.25",
            )
        })
        .collect::<Vec<_>>();
    *http.alpaca_order_history.borrow_mut() = orders;
    *http.alpaca_fills.borrow_mut() = fills;

    let refreshed = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    let book = &refreshed["data"];
    assert_eq!(book["status"], "CURRENT");
    assert_eq!(book["orders"].as_array().unwrap().len(), 500);
    assert_eq!(book["fills"].as_array().unwrap().len(), 1000);
    let partial = book["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "00000000-0000-4000-8000-000000000001")
        .unwrap();
    assert_eq!(partial["quantity"], "3.5");
    assert_eq!(partial["filledQuantity"], "1.25");
    assert_eq!(partial["remainingQuantity"], "2.25");
    assert_eq!(partial["origin"], "EXTERNAL");
    assert_eq!(partial["instrumentId"], "equity:US:AAPL");
    assert!(
        book["fills"][0]["activityId"]
            .as_str()
            .unwrap()
            .contains("::")
    );
    assert!(
        http.calls
            .borrow()
            .iter()
            .any(|path| path.contains("page_token=20260923100000000::"))
    );
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|path| path.contains("/v2/orders?status=all"))
            .count(),
        6
    );
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|path| path.contains("/v2/account/activities/FILL"))
            .count(),
        11
    );
    let repeated = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(repeated["data"]["orders"].as_array().unwrap().len(), 500);
    assert_eq!(repeated["data"]["fills"].as_array().unwrap().len(), 1000);
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|path| path.contains("/v2/orders?status=all"))
            .count(),
        12
    );
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|path| path.contains("/v2/account/activities/FILL"))
            .count(),
        22
    );

    let snapshot = command(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"alpaca-paper-order-book","aggregateId":account["connectionId"]}),
    );
    assert_eq!(snapshot["ok"], true, "{snapshot}");
    assert_eq!(snapshot["data"]["lastSequence"], 2);
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = events.clone();
    let subscribed = cp.dispatch_with_events(
        envelope(
            "domain.subscribe",
            json!({"aggregateType":"alpaca-paper-order-book","aggregateId":account["connectionId"],"afterSequence":0}),
        ),
        "main",
        Some(std::sync::Arc::new(move |event| {
            sink.lock()
                .unwrap()
                .push(serde_json::to_value(event).unwrap());
            true
        })),
    );
    assert_eq!(subscribed["ok"], true, "{subscribed}");
    assert_eq!(subscribed["data"]["replayedCount"], 2);
    assert_eq!(
        events.lock().unwrap()[1]["eventType"],
        "alpaca.paper.order.book.changed"
    );
    drop(cp);

    let mut reopened = ControlPlane::new(path);
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(
        &mut reopened,
        "alpaca.paper.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(restored["ok"], true, "{restored}");
    assert_eq!(restored["data"]["book"]["status"], "STALE");
    assert_eq!(
        restored["data"]["book"]["orders"].as_array().unwrap().len(),
        500
    );
    assert_eq!(
        restored["data"]["book"]["fills"].as_array().unwrap().len(),
        1000
    );
}

#[test]
fn alpaca_private_stream_gap_reopens_stale_and_rest_reconciliation_dedupes_fills() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let connection_id = account["connectionId"].as_str().unwrap();
    let remote_account_id = account["data"]["remoteAccountId"].as_str().unwrap();
    let order = alpaca_order(1, "1", "0", "new");
    let order_id = order["id"].as_str().unwrap();
    let client_order_id = order["client_order_id"].as_str().unwrap();
    let execution_id = "00000000-0000-4000-8000-000000000001";
    let observed_at = "2026-09-23T10:00:00Z";
    let mut current_order = order.clone();
    current_order["status"] = "partially_filled".into();
    current_order["filled_qty"] = "0.5".into();
    current_order["updated_at"] = observed_at.into();
    *http.alpaca_order_history.borrow_mut() = vec![order.clone()];
    let initial = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(initial["data"]["status"], "CURRENT", "{initial}");

    let frame = json!({
        "stream":"trade_updates",
        "data":{
            "event":"partial_fill",
            "execution_id":execution_id,
            "qty":"0.5",
            "price":"10.25",
            "timestamp":observed_at,
            "order":{
                "id":order_id,
                "account_id":remote_account_id,
                "client_order_id":client_order_id,
                "symbol":"AAPL",
                "side":"buy",
                "type":"limit",
                "time_in_force":"day",
                "status":"partially_filled",
                "qty":"1",
                "filled_qty":"0.5",
                "submitted_at":observed_at,
                "updated_at":observed_at
            }
        }
    });
    cp.apply_alpaca_private_stream_frame(
        connection_id,
        account["stateVersion"].as_str().unwrap(),
        remote_account_id,
        &frame,
        &[KEY.into(), SECRET.into()],
    )
    .unwrap();
    cp.mark_alpaca_private_stream_degraded(
        connection_id,
        account["stateVersion"].as_str().unwrap(),
        "DEGRADED",
        "Alpaca Paper private stream disconnected; saved orders are not current.",
    )
    .unwrap();
    let stale = command(
        &mut cp,
        "alpaca.paper.orders.get",
        json!({"workspaceId":workspace,"connectionId":connection_id}),
    );
    assert_eq!(stale["data"]["book"]["status"], "STALE", "{stale}");
    assert_eq!(stale["data"]["book"]["fills"].as_array().unwrap().len(), 1);
    let disconnected = command(&mut cp, "account.get", query(&account));
    assert_eq!(disconnected["data"]["health"]["privateStream"], "DEGRADED");
    assert_eq!(disconnected["data"]["health"]["reconciliation"], "DEGRADED");
    assert!(disconnected["data"]["lastPrivateStreamEventAt"].is_string());
    drop(cp);

    let mut reopened = ControlPlane::new(path);
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let restored_accounts = reopened.alpaca_private_stream_accounts().unwrap();
    assert_eq!(restored_accounts.len(), 1);
    assert_eq!(restored_accounts[0].connection_id, connection_id);
    let restored_account = command(
        &mut reopened,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":connection_id}),
    )["data"]
        .clone();
    assert_eq!(
        restored_account["connectionState"], "CONNECTED",
        "{restored_account}"
    );
    let revalidated = execute_main(
        &mut reopened,
        envelope("account.refresh", mutation(&restored_account)),
        &vault,
        &http,
    );
    assert_eq!(revalidated["ok"], true, "{revalidated}");
    let current_account =
        command(&mut reopened, "account.get", query(&restored_account))["data"].clone();
    let restored = command(
        &mut reopened,
        "alpaca.paper.orders.get",
        json!({"workspaceId":workspace,"connectionId":connection_id}),
    );
    assert_eq!(restored["data"]["book"]["status"], "STALE", "{restored}");

    *http.alpaca_order_history.borrow_mut() = vec![current_order];
    *http.alpaca_fills.borrow_mut() = vec![alpaca_fill(1, order_id, "0.5", "10.25")];
    let reconciled = execute_main(
        &mut reopened,
        refresh_alpaca_orders(&workspace, &current_account),
        &vault,
        &http,
    );
    assert_eq!(reconciled["ok"], true, "{reconciled}");
    assert_eq!(reconciled["data"]["status"], "CURRENT");
    assert_eq!(
        reconciled["data"]["orders"][0]["providerStatus"],
        "partially_filled"
    );
    assert_eq!(reconciled["data"]["fills"].as_array().unwrap().len(), 1);
    assert_eq!(reconciled["data"]["fills"][0]["source"], "REST_ACTIVITY");
    assert!(
        reconciled["data"]["fills"][0]["activityId"]
            .as_str()
            .unwrap()
            .contains("::")
    );
    reopened
        .update_alpaca_private_stream_health(
            connection_id,
            current_account["stateVersion"].as_str().unwrap(),
            "CONNECTED",
            "CURRENT",
            "Alpaca Paper private stream reconnected and REST reconciliation completed.",
        )
        .unwrap();
    let recovered = command(&mut reopened, "account.get", query(&account));
    assert_eq!(recovered["data"]["health"]["privateStream"], "CONNECTED");
    assert_eq!(recovered["data"]["health"]["reconciliation"], "CURRENT");
    assert_eq!(http.alpaca_posts.borrow().len(), 0);
}

#[test]
fn alpaca_order_book_degrades_without_replacing_last_complete_data() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let mut order = alpaca_order(1, "3.5", "1", "partially_filled");
    let fill = alpaca_fill(1, order["id"].as_str().unwrap(), "1", "100");
    *http.alpaca_order_history.borrow_mut() = vec![order.clone()];
    *http.alpaca_fills.borrow_mut() = vec![fill.clone()];
    let first = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(first["data"]["status"], "CURRENT");
    let last_successful_sync = first["data"]["lastSuccessfulSyncAt"].clone();
    let prior_order = first["data"]["orders"][0].clone();
    let prior_fill = first["data"]["fills"][0].clone();

    order["status"] = "filled".into();
    order["filled_qty"] = "3.5".into();
    *http.alpaca_order_history.borrow_mut() = vec![order];
    let mut conflicting_fill = fill;
    conflicting_fill["price"] = "101".into();
    *http.alpaca_fills.borrow_mut() = vec![conflicting_fill];
    let degraded = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(degraded["ok"], true, "{degraded}");
    assert_eq!(degraded["data"]["status"], "DEGRADED");
    assert_eq!(degraded["data"]["reason"], "PROVIDER_RESPONSE_INCOMPLETE");
    assert_eq!(degraded["data"]["orders"][0], prior_order);
    assert_eq!(degraded["data"]["fills"][0], prior_fill);
    assert_eq!(
        degraded["data"]["lastSuccessfulSyncAt"],
        last_successful_sync
    );

    let mut conflicting_order = alpaca_order(1, "3.5", "3.5", "filled");
    conflicting_order["client_order_id"] = "changed-provider-identity".into();
    *http.alpaca_order_history.borrow_mut() = vec![conflicting_order];
    *http.alpaca_fills.borrow_mut() = vec![alpaca_fill(
        1,
        "00000000-0000-4000-8000-000000000001",
        "1",
        "100",
    )];
    let identity_conflict = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(identity_conflict["data"]["status"], "DEGRADED");
    assert_eq!(identity_conflict["data"]["orders"][0], prior_order);
    assert_eq!(identity_conflict["data"]["fills"][0], prior_fill);

    let mut reflected_secret = alpaca_order(1, "3.5", "3.5", "filled");
    reflected_secret["status"] = KEY.into();
    *http.alpaca_order_history.borrow_mut() = vec![reflected_secret];
    *http.alpaca_fills.borrow_mut() = Vec::new();
    let redacted = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(redacted["data"]["status"], "DEGRADED");
    assert_eq!(redacted["data"]["reason"], "PROVIDER_RESPONSE_INVALID");
    assert_eq!(redacted["data"]["orders"][0], prior_order);
    let encoded = serde_json::to_string(&redacted).unwrap();
    assert!(!encoded.contains(KEY));
    assert!(!encoded.contains(SECRET));
}

#[test]
fn alpaca_order_book_rejects_repeated_cursor_and_over_cap_without_false_empty() {
    for mode in ["repeat", "over-orders", "over-fills"] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
        *http.alpaca_order_history.borrow_mut() = if mode == "over-fills" {
            Vec::new()
        } else {
            (1..=if mode == "repeat" { 100 } else { 501 })
                .map(|id| alpaca_order(id, "1", "0", "new"))
                .collect()
        };
        if mode == "over-fills" {
            *http.alpaca_fills.borrow_mut() = (1..=1001)
                .map(|id| alpaca_fill(id, "00000000-0000-4000-8000-000000000001", "0.1", "100"))
                .collect();
        }
        http.alpaca_repeat_order_cursor.set(mode == "repeat");

        let result = execute_main(
            &mut cp,
            refresh_alpaca_orders(&workspace, &account),
            &vault,
            &http,
        );
        assert_eq!(result["ok"], true, "{result}");
        assert_eq!(result["data"]["status"], "DEGRADED");
        assert_eq!(result["data"]["reason"], "PROVIDER_RESPONSE_INCOMPLETE");
        assert!(result["data"]["orders"].as_array().unwrap().is_empty());
        assert!(
            http.calls
                .borrow()
                .iter()
                .filter(|path| path.contains("/v2/orders?status=all"))
                .count()
                <= 6
        );
        assert!(
            http.calls
                .borrow()
                .iter()
                .filter(|path| path.contains("/v2/account/activities/FILL"))
                .count()
                <= 11
        );
    }
}

#[test]
fn alpaca_cancel_review_records_intent_before_io_and_waits_for_provider_truth() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().to_path_buf());
    let workspace = command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let http = Http::default();
    let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
    let order = alpaca_order(1, "3.5", "1.25", "partially_filled");
    let order_id = order["id"].as_str().unwrap().to_owned();
    *http.alpaca_order_history.borrow_mut() = vec![order];
    let initial = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(initial["data"]["status"], "CURRENT");

    let reviewed = execute_main(
        &mut cp,
        review_alpaca_order(&workspace, &account, &order_id),
        &vault,
        &http,
    );
    assert_eq!(reviewed["ok"], true, "{reviewed}");
    let review_book = &reviewed["data"];
    let reviewed_order = review_book["orders"].as_array().unwrap().first().unwrap();
    assert_eq!(review_book["status"], "STALE");
    assert_eq!(reviewed_order["providerStatus"], "partially_filled");
    assert_eq!(reviewed_order["filledQuantity"], "1.25");
    assert_eq!(reviewed_order["remainingQuantity"], "2.25");
    assert!(http.alpaca_delete_calls.borrow().is_empty());

    let cancel = cancel_alpaca_order(&workspace, &account, review_book, &order_id, "cancel-qa-1");
    let job = cp.prepare_provider_for(&cancel, "main").unwrap().unwrap();
    let submitting = command(
        &mut cp,
        "alpaca.paper.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    let order_state = &submitting["data"]["book"]["orders"][0];
    assert_eq!(order_state["cancelState"], "SUBMITTING");
    assert_eq!(order_state["providerStatus"], "partially_filled");
    assert_eq!(order_state["remainingQuantity"], "2.25");
    assert!(
        http.alpaca_delete_calls.borrow().is_empty(),
        "durable intent precedes DELETE"
    );
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let accepted = cp.complete_provider(&job, outcome);
    assert_eq!(accepted["ok"], true, "{accepted}");
    assert_eq!(accepted["data"]["orders"][0]["cancelState"], "PENDING");
    assert_eq!(
        accepted["data"]["orders"][0]["providerStatus"],
        "partially_filled"
    );
    assert_eq!(http.alpaca_delete_calls.borrow().len(), 1);

    let persisted = command(
        &mut cp,
        "alpaca.paper.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    )["data"]["book"]
        .clone();
    let repeated = cp
        .prepare_provider_for(
            &cancel_alpaca_order(&workspace, &account, &persisted, &order_id, "cancel-qa-1"),
            "main",
        )
        .unwrap();
    assert!(repeated.is_none(), "pending cancel is idempotent");
    assert_eq!(http.alpaca_delete_calls.borrow().len(), 1);

    http.alpaca_order_history.borrow_mut()[0]["status"] = "canceled".into();
    let reconciled = execute_main(
        &mut cp,
        refresh_alpaca_orders(&workspace, &account),
        &vault,
        &http,
    );
    assert_eq!(reconciled["data"]["status"], "CURRENT");
    assert_eq!(
        reconciled["data"]["orders"][0]["providerStatus"],
        "canceled"
    );
    assert_eq!(reconciled["data"]["orders"][0]["cancelState"], "NONE");
    assert_eq!(
        reconciled["data"]["orders"][0]["cancelIdempotencyKey"],
        Value::Null
    );
}

#[test]
fn alpaca_cancel_rejection_and_fill_race_preserve_provider_truth() {
    for (delete_status, with_fill) in [(Some(422), false), (None, true)] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().to_path_buf());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let http = Http::default();
        let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
        let order = alpaca_order(1, "3.5", "1.25", "partially_filled");
        let order_id = order["id"].as_str().unwrap().to_owned();
        *http.alpaca_order_history.borrow_mut() = vec![order];
        if with_fill {
            *http.alpaca_fills.borrow_mut() = vec![alpaca_fill(1, &order_id, "2.25", "100.25")];
            http.alpaca_delete_order_status
                .borrow_mut()
                .replace("filled".into());
        } else {
            http.alpaca_delete_status.set(delete_status);
        }
        let initial = execute_main(
            &mut cp,
            refresh_alpaca_orders(&workspace, &account),
            &vault,
            &http,
        );
        let reviewed = execute_main(
            &mut cp,
            review_alpaca_order(&workspace, &account, &order_id),
            &vault,
            &http,
        );
        let cancel = cancel_alpaca_order(
            &workspace,
            &account,
            &reviewed["data"],
            &order_id,
            "cancel-qa-race",
        );
        let result = execute_main(&mut cp, cancel, &vault, &http);
        assert_eq!(result["ok"], true, "{result}; initial={initial}");
        let current = result["data"]["orders"]
            .as_array()
            .unwrap()
            .first()
            .unwrap();
        assert_eq!(http.alpaca_delete_calls.borrow().len(), 1);
        if with_fill {
            assert_eq!(current["providerStatus"], "filled");
            assert_eq!(current["filledQuantity"], "3.5");
            assert_eq!(current["remainingQuantity"], "0");
            assert_eq!(current["cancelState"], "NONE");
            assert_eq!(result["data"]["fills"].as_array().unwrap().len(), 1);
            assert_eq!(result["data"]["fills"][0]["quantity"], "2.25");
        } else {
            assert_eq!(current["providerStatus"], "partially_filled");
            assert_eq!(current["cancelState"], "NONE");
            assert_eq!(current["cancelError"], "PROVIDER_CANCEL_REJECTED");
        }
    }
}

#[test]
fn alpaca_cancel_identity_mismatch_and_workspace_reopen_never_delete() {
    for reopen in [false, true] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().to_path_buf();
        let mut cp = ControlPlane::new(path.clone());
        let workspace =
            command(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = Vault::default();
        let mut http = Http::default();
        let account = connected_alpaca(&mut cp, &vault, &http, &workspace);
        let order = alpaca_order(1, "1", "0", "new");
        let order_id = order["id"].as_str().unwrap().to_owned();
        *http.alpaca_order_history.borrow_mut() = vec![order];
        let _ = execute_main(
            &mut cp,
            refresh_alpaca_orders(&workspace, &account),
            &vault,
            &http,
        );
        let reviewed = execute_main(
            &mut cp,
            review_alpaca_order(&workspace, &account, &order_id),
            &vault,
            &http,
        );
        let input = cancel_alpaca_order(
            &workspace,
            &account,
            &reviewed["data"],
            &order_id,
            "cancel-qa-reopen",
        );
        let job = cp.prepare_provider_for(&input, "main").unwrap().unwrap();
        if reopen {
            drop(job);
            drop(cp);
            let mut reopened = ControlPlane::new(path);
            assert_eq!(
                command(&mut reopened, "workspace.open", json!({}))["ok"],
                true
            );
            let restored = command(
                &mut reopened,
                "alpaca.paper.orders.get",
                json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
            );
            assert_eq!(
                restored["data"]["book"]["orders"][0]["cancelState"],
                "PENDING"
            );
            assert_eq!(
                restored["data"]["book"]["orders"][0]["cancelError"],
                "ORDER_CANCEL_STATUS_UNKNOWN"
            );
            assert!(http.alpaca_delete_calls.borrow().is_empty());
        } else {
            http.identity = "b1a3c22f-4ad7-47aa-912b-cda43b22ce44".into();
            let outcome = job.run(
                &vault,
                |_| credentials(),
                &http,
                || cp.provider_job_current(&job),
            );
            let failed_closed = cp.complete_provider(&job, outcome);
            assert_eq!(failed_closed["ok"], true, "{failed_closed}");
            assert_eq!(failed_closed["data"]["status"], "DEGRADED");
            assert_eq!(failed_closed["data"]["orders"][0]["cancelState"], "NONE");
            assert_eq!(
                failed_closed["data"]["orders"][0]["cancelError"],
                "PROVIDER_REVIEW_REQUIRED"
            );
            assert!(http.alpaca_delete_calls.borrow().is_empty());
        }
    }
}
