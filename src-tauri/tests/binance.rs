use serde_json::{Value, json};
use tradex::{
    ControlPlane,
    provider_io::{CredentialVault, ProviderHttp},
};
#[path = "support/provider_fixtures.rs"]
mod fixtures;
fn request(command: &str, payload: Value) -> Value {
    json!({"requestId":"binance-test","schemaVersion":1,"command":command,"payload":payload})
}
fn call(cp: &mut ControlPlane, command: &str, payload: Value) -> Value {
    cp.dispatch(request(command, payload))
}
fn mutation(a: &Value) -> Value {
    json!({"workspaceId":a["workspaceId"],"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]})
}
fn run(
    cp: &mut ControlPlane,
    req: Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let job = cp.prepare_provider_for(&req, "main").unwrap().unwrap();
    let result = job.run(
        vault,
        |_| fixtures::credentials(),
        http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, result);
    if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, vault) {
        cp.record_credential_cleanup(&job, cleanup);
    }
    reply
}
fn lifecycle(vault: &impl CredentialVault) {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let http = fixtures::Http::default();
    for env in ["TESTNET", "LIVE"] {
        let schema = call(
            &mut cp,
            "provider.get_schema",
            json!({"providerId":"binance","environment":env}),
        );
        assert_eq!(schema["ok"], true, "{schema}");
        let reply = run(
            &mut cp,
            request(
                "provider.connect",
                json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":env,"label":format!("Spot {env}")}),
            ),
            vault,
            &http,
        );
        assert_eq!(reply["ok"], true, "{reply}");
        let a = &reply["data"];
        assert_eq!(a["data"]["remoteAccountId"], "9007199254740993");
        assert_eq!(a["data"]["currency"], Value::Null);
        assert_eq!(a["data"]["balances"][0]["asset"], "USDT");
        assert_eq!(
            a["data"]["balances"][0]["total"],
            "100000000000000000000.0000000000000000001"
        );
        assert_eq!(a["data"]["positions"][0]["marketValue"], Value::Null);
        assert_eq!(a["data"]["openOrders"].as_array().unwrap().len(), 2);
        assert_eq!(
            a["data"]["openOrders"][0]["instrumentId"],
            "crypto:BTC/USDT:spot"
        );
        assert!(a["data"]["openOrders"][1]["instrumentId"].is_null());
        assert_ne!(
            a["data"]["openOrders"][0]["brokerOrderId"],
            a["data"]["openOrders"][1]["brokerOrderId"]
        );
        assert_eq!(
            a["permissions"]["scope"],
            if env == "LIVE" {
                "VERIFIED"
            } else {
                "UNVERIFIED"
            }
        );
        assert_eq!(
            a["permissions"]["forbidden"],
            json!([]),
            "account canWithdraw must not become key scope"
        );
        assert_eq!(a["health"]["executionEligibility"], "BLOCKED");
        if env == "LIVE" {
            assert_eq!(a["health"]["arming"], "DISARMED");
        }
        let mut confirm = mutation(a);
        confirm["step"] = "confirm".into();
        confirm["acknowledgeUnverified"] = true.into();
        let accepted = call(&mut cp, "provider.connect", confirm);
        assert_eq!(accepted["ok"], true, "{accepted}");
        let disconnected = run(
            &mut cp,
            request("provider.disconnect", mutation(&accepted["data"])),
            vault,
            &http,
        );
        assert_eq!(disconnected["data"]["health"]["credential"], "MISSING");
    }
}

fn connected_testnet(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
) -> Value {
    connected_testnet_labeled(cp, vault, http, workspace, "Testnet orders")
}

fn connected_testnet_labeled(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
    label: &str,
) -> Value {
    let tested = run(
        cp,
        request(
            "provider.connect",
            json!({"step":"test","workspaceId":workspace,"providerId":"binance","environment":"TESTNET","label":label}),
        ),
        vault,
        http,
    );
    assert_eq!(tested["ok"], true, "{tested}");
    let mut confirm = mutation(&tested["data"]);
    confirm["step"] = "confirm".into();
    confirm["acknowledgeUnverified"] = true.into();
    let accepted = call(cp, "provider.connect", confirm);
    assert_eq!(accepted["ok"], true, "{accepted}");
    accepted["data"].clone()
}

fn refresh_testnet_book(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
    account: &Value,
    action: &str,
    symbol: Option<&str>,
    provider_order_id: Option<&str>,
) -> Value {
    let mut payload = json!({
        "workspaceId":workspace,
        "connectionId":account["connectionId"],
        "expectedConnectionStateVersion":account["stateVersion"],
        "action":action
    });
    if let Some(symbol) = symbol {
        payload["symbol"] = json!(symbol);
    }
    if let Some(provider_order_id) = provider_order_id {
        payload["providerOrderId"] = json!(provider_order_id);
    }
    run(
        cp,
        request("binance.testnet.orders.refresh", payload),
        vault,
        http,
    )
}

fn cancel_testnet_order(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
    account: &Value,
    book: &Value,
    provider_order_id: &str,
    idempotency_key: &str,
) -> Value {
    run(
        cp,
        request(
            "binance.testnet.orders.cancel",
            json!({
                "workspaceId":workspace,
                "connectionId":account["connectionId"],
                "expectedConnectionStateVersion":account["stateVersion"],
                "symbol":"BTCUSDT",
                "providerOrderId":provider_order_id,
                "expectedBookStateVersion":book["stateVersion"],
                "idempotencyKey":idempotency_key,
                "confirmed":true
            }),
        ),
        vault,
        http,
    )
}

fn testnet_proposal(
    cp: &mut ControlPlane,
    workspace: &Value,
    account: &Value,
    order_type: &str,
    quantity_type: &str,
    quantity: &str,
    tif: &str,
) -> Value {
    testnet_proposal_side(
        cp,
        workspace,
        account,
        order_type,
        quantity_type,
        quantity,
        tif,
        "BUY",
    )
}

fn testnet_proposal_side(
    cp: &mut ControlPlane,
    workspace: &Value,
    account: &Value,
    order_type: &str,
    quantity_type: &str,
    quantity: &str,
    tif: &str,
    side: &str,
) -> Value {
    let mut fields = json!({
        "accountId":account["connectionId"],"venue":"BINANCE",
        "environment":"BINANCE_TESTNET","instrumentId":"crypto:BTC/USDT:spot",
        "side":side,"orderType":order_type,
        "quantity":{"type":quantity_type,"value":quantity},"timeInForce":tif
    });
    if order_type == "LIMIT" {
        fields["limitPrice"] = "100".into();
    }
    let draft = call(
        cp,
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":fields}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = call(
        cp,
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    call(
        cp,
        "trade.proposal.get",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}),
    )["data"]
        .clone()
}

fn testnet_submit_request(workspace: &Value, account: &Value, proposal: &Value) -> Value {
    request(
        "binance.testnet.order.submit",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "proposalId":proposal["proposalId"],
            "expectedProposalStateVersion":proposal["stateVersion"],
            "proposalHash":proposal["proposalHash"],
            "idempotencyKey":"testnet-order-1","confirmedTestnetOrder":true
        }),
    )
}

#[test]
fn testnet_market_quote_submit_persists_once_before_io_and_keeps_ack_separate_from_fill() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let proposal = testnet_proposal(
        &mut cp, &workspace, &account, "MARKET", "QUOTE", "25", "DAY",
    );
    let submit = testnet_submit_request(&workspace, &account, &proposal);

    let forbidden = cp.prepare_provider_for(&submit, "research");
    assert_eq!(forbidden.err().unwrap().code, "ORDER_SUBMIT_FORBIDDEN");
    let job = cp.prepare_provider_for(&submit, "main").unwrap().unwrap();
    let saved = call(
        &mut cp,
        "binance.testnet.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(saved["data"]["attempt"]["state"], "SUBMITTING");
    assert!(
        http.binance_posts.borrow().is_empty(),
        "the attempt must commit before provider I/O"
    );

    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "ACKNOWLEDGED");
    assert_eq!(reply["data"]["providerOrderId"], "9007199254740997");
    assert_eq!(
        reply["data"].get("fill"),
        None,
        "an acknowledgement is not a fill"
    );
    assert_eq!(http.binance_posts.borrow().len(), 1);
    let sent = http.binance_posts.borrow()[0].clone();
    assert!(sent.contains("quoteOrderQty=25"));
    assert!(sent.contains("newClientOrderId="));
    assert!(!sent.contains("quantity="));

    assert!(cp.prepare_provider_for(&submit, "main").unwrap().is_none());
    assert_eq!(
        http.binance_posts.borrow().len(),
        1,
        "duplicate submission must reuse its saved attempt"
    );
    let snapshot = call(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"binance-testnet-order-attempt","aggregateId":reply["data"]["attemptId"]}),
    );
    assert_eq!(snapshot["data"]["lastSequence"], 2);
    let encoded = serde_json::to_string(&reply).unwrap();
    assert!(!encoded.contains(fixtures::KEY));
    assert!(!encoded.contains(fixtures::SECRET));
}

#[test]
fn testnet_provider_rejection_is_persisted_redacted_and_never_retried() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let proposal = testnet_proposal(
        &mut cp, &workspace, &account, "MARKET", "QUOTE", "25", "DAY",
    );
    let submit = testnet_submit_request(&workspace, &account, &proposal);
    http.binance_post_status.set(Some(400));
    *http.binance_post_response_body.borrow_mut() = Some(
        serde_json::to_vec(&json!({
            "code":-2010,
            "msg":format!("provider reflected {} {}", fixtures::KEY, fixtures::SECRET)
        }))
        .unwrap(),
    );

    let job = cp.prepare_provider_for(&submit, "main").unwrap().unwrap();
    let saved_before_io = call(
        &mut cp,
        "binance.testnet.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(saved_before_io["data"]["attempt"]["state"], "SUBMITTING");

    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["state"], "REJECTED");
    assert_eq!(reply["data"]["errorCode"], "ORDER_PROVIDER_REJECTED");
    assert_eq!(http.binance_posts.borrow().len(), 1);
    assert!(cp.prepare_provider_for(&submit, "main").unwrap().is_none());
    assert_eq!(http.binance_posts.borrow().len(), 1);

    let attempt = call(
        &mut cp,
        "binance.testnet.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    );
    assert_eq!(attempt["data"]["attempt"]["state"], "REJECTED");
    assert_eq!(
        attempt["data"]["attempt"]["errorCode"],
        "ORDER_PROVIDER_REJECTED"
    );
    let snapshot = call(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"binance-testnet-order-attempt","aggregateId":reply["data"]["attemptId"]}),
    );
    assert_eq!(snapshot["data"]["lastSequence"], 2);
    let persisted = json!({"reply":reply,"attempt":attempt,"snapshot":snapshot}).to_string();
    assert!(!persisted.contains(fixtures::KEY));
    assert!(!persisted.contains(fixtures::SECRET));
    assert!(!persisted.contains("provider reflected"));
}

#[test]
fn unknown_testnet_submit_is_query_only_until_client_order_id_is_found() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let proposal = testnet_proposal(
        &mut cp, &workspace, &account, "MARKET", "QUOTE", "25", "DAY",
    );
    let submit = testnet_submit_request(&workspace, &account, &proposal);
    http.binance_post_timeout.set(true);
    http.binance_hide_order_lookup.set(true);

    let unknown = run(&mut cp, submit.clone(), &vault, &http);
    assert_eq!(unknown["ok"], true, "{unknown}");
    assert_eq!(unknown["data"]["state"], "UNKNOWN_RECONCILING");
    assert_eq!(http.binance_posts.borrow().len(), 1);
    assert!(cp.prepare_provider_for(&submit, "main").unwrap().is_none());
    assert_eq!(
        http.binance_posts.borrow().len(),
        1,
        "duplicate cannot repeat POST"
    );

    http.binance_hide_order_lookup.set(false);
    let reconcile = request(
        "binance.testnet.order.reconcile",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "proposalId":proposal["proposalId"]
        }),
    );
    let job = cp
        .prepare_provider_for(&reconcile, "main")
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let recovered = cp.complete_provider(&job, outcome);
    assert_eq!(recovered["ok"], true, "{recovered}");
    assert_eq!(recovered["data"]["state"], "ACKNOWLEDGED");
    assert_eq!(recovered["data"]["providerOrderId"], "9007199254740997");
    assert_eq!(
        http.binance_posts.borrow().len(),
        1,
        "reconciliation is GET-only"
    );
    assert!(
        http.calls
            .borrow()
            .iter()
            .all(|call| call.starts_with("https://testnet.binance.vision/"))
    );
    assert!(
        http.calls
            .borrow()
            .iter()
            .all(|call| !call.contains("/sapi/"))
    );
}

#[test]
fn testnet_filters_and_percent_price_rules_fail_closed_before_post() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let quote_proposal = testnet_proposal(
        &mut cp, &workspace, &account, "MARKET", "QUOTE", "25", "DAY",
    );
    *http.binance_exchange_info.borrow_mut() = Some(
        json!({"symbols":[{"symbol":"BTCUSDT","status":"TRADING","baseAsset":"BTC","quoteAsset":"USDT","filters":[{"filterType":"NOTIONAL","minNotional":"30","maxNotional":"0","applyMinToMarket":true,"applyMaxToMarket":false,"avgPriceMins":5}]}]}),
    );
    let quote_reply = run(
        &mut cp,
        testnet_submit_request(&workspace, &account, &quote_proposal),
        &vault,
        &http,
    );
    assert_eq!(quote_reply["data"]["state"], "REJECTED", "{quote_reply}");
    assert_eq!(quote_reply["data"]["errorCode"], "ORDER_FILTER_REJECTED");
    assert!(http.binance_posts.borrow().is_empty());

    *http.binance_exchange_info.borrow_mut() = Some(
        json!({"symbols":[{"symbol":"BTCUSDT","status":"TRADING","baseAsset":"BTC","quoteAsset":"USDT","filters":[{"filterType":"PRICE_FILTER","minPrice":"0.01","maxPrice":"1000000","tickSize":"0.01"},{"filterType":"LOT_SIZE","minQty":"0.03","maxQty":"9000","stepSize":"0.03"},{"filterType":"NOTIONAL","minNotional":"5","maxNotional":"0","applyMinToMarket":true,"applyMaxToMarket":false,"avgPriceMins":5}]}]}),
    );
    let limit_proposal =
        testnet_proposal(&mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC");
    let mut limit_submit = testnet_submit_request(&workspace, &account, &limit_proposal);
    limit_submit["payload"]["idempotencyKey"] = "testnet-order-filter-2".into();
    let limit_reply = run(&mut cp, limit_submit, &vault, &http);
    assert_eq!(limit_reply["data"]["state"], "REJECTED", "{limit_reply}");
    assert_eq!(limit_reply["data"]["errorCode"], "ORDER_FILTER_REJECTED");
    assert!(
        http.binance_posts.borrow().is_empty(),
        "misaligned base quantity is not rounded and posted"
    );

    let base_info = fixtures::default_binance_exchange_info();
    for (key, price_filter, error) in [
        (
            "testnet-order-percent",
            json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"0.9","avgPriceMins":5}),
            "ORDER_FILTER_REJECTED",
        ),
        (
            "testnet-order-percent-side",
            json!({"filterType":"PERCENT_PRICE_BY_SIDE","bidMultiplierDown":"0.8","bidMultiplierUp":"0.9","askMultiplierDown":"0.5","askMultiplierUp":"2","avgPriceMins":5}),
            "ORDER_FILTER_REJECTED",
        ),
        (
            "testnet-order-percent-interval",
            json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":1}),
            "ORDER_FILTERS_UNAVAILABLE",
        ),
    ] {
        let mut info = base_info.clone();
        info["symbols"][0]["filters"]
            .as_array_mut()
            .unwrap()
            .push(price_filter);
        *http.binance_exchange_info.borrow_mut() = Some(info);
        let proposal =
            testnet_proposal(&mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC");
        let mut submit = testnet_submit_request(&workspace, &account, &proposal);
        submit["payload"]["idempotencyKey"] = key.into();
        let reply = run(&mut cp, submit, &vault, &http);
        assert_eq!(reply["data"]["state"], "REJECTED", "{reply}");
        assert_eq!(reply["data"]["errorCode"], error, "{reply}");
    }
    let mut reference_info = base_info.clone();
    reference_info["symbols"][0]["filters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.9","multiplierUp":"1.1","avgPriceMins":5}));
    *http.binance_exchange_info.borrow_mut() = Some(reference_info);
    *http.binance_reference_price.borrow_mut() = Some(json!({
        "symbol":"BTCUSDT","referencePrice":"200","timestamp":1788849600000u64
    }));
    let avg_price_calls = http
        .calls
        .borrow()
        .iter()
        .filter(|call| call.ends_with("/api/v3/avgPrice?symbol=BTCUSDT"))
        .count();
    let proposal = testnet_proposal(&mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC");
    let mut submit = testnet_submit_request(&workspace, &account, &proposal);
    submit["payload"]["idempotencyKey"] = "testnet-order-reference-price".into();
    let reply = run(&mut cp, submit, &vault, &http);
    assert_eq!(reply["data"]["state"], "REJECTED", "{reply}");
    assert_eq!(reply["data"]["errorCode"], "ORDER_FILTER_REJECTED");
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|call| call.ends_with("/api/v3/avgPrice?symbol=BTCUSDT"))
            .count(),
        avg_price_calls,
        "a non-null reference price takes precedence over the average-price fallback"
    );

    http.binance_reference_price_status.set(Some(503));
    *http.binance_reference_price.borrow_mut() = Some(json!({"code":-1000,"msg":"unavailable"}));
    let avg_price_calls = http
        .calls
        .borrow()
        .iter()
        .filter(|call| call.ends_with("/api/v3/avgPrice?symbol=BTCUSDT"))
        .count();
    let proposal = testnet_proposal(&mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC");
    let mut submit = testnet_submit_request(&workspace, &account, &proposal);
    submit["payload"]["idempotencyKey"] = "testnet-order-reference-error".into();
    let reply = run(&mut cp, submit, &vault, &http);
    assert_eq!(reply["data"]["state"], "REJECTED", "{reply}");
    assert_eq!(reply["data"]["errorCode"], "ORDER_FILTERS_UNAVAILABLE");
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|call| call.ends_with("/api/v3/avgPrice?symbol=BTCUSDT"))
            .count(),
        avg_price_calls,
        "a reference endpoint error must not silently select another price source"
    );
    assert!(http.binance_posts.borrow().is_empty());

    let mut sell_info = base_info.clone();
    sell_info["symbols"][0]["filters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"filterType":"PERCENT_PRICE_BY_SIDE","bidMultiplierDown":"0.5","bidMultiplierUp":"2","askMultiplierDown":"0.8","askMultiplierUp":"0.9","avgPriceMins":0}));
    *http.binance_exchange_info.borrow_mut() = Some(sell_info);
    *http.binance_reference_price.borrow_mut() = None;
    http.binance_reference_price_status.set(None);
    let ticker_calls = http
        .calls
        .borrow()
        .iter()
        .filter(|call| call.ends_with("/api/v3/ticker/price?symbol=BTCUSDT"))
        .count();
    let proposal = testnet_proposal_side(
        &mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC", "SELL",
    );
    let mut submit = testnet_submit_request(&workspace, &account, &proposal);
    submit["payload"]["idempotencyKey"] = "testnet-order-percent-sell".into();
    let reply = run(&mut cp, submit, &vault, &http);
    assert_eq!(reply["data"]["state"], "REJECTED", "{reply}");
    assert_eq!(reply["data"]["errorCode"], "ORDER_FILTER_REJECTED");
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|call| call.ends_with("/api/v3/ticker/price?symbol=BTCUSDT"))
            .count(),
        ticker_calls + 1,
        "avgPriceMins=0 uses the last-price endpoint"
    );
    assert!(http.binance_posts.borrow().is_empty());

    let mut boundary_info = base_info;
    boundary_info["symbols"][0]["filters"]
        .as_array_mut()
        .unwrap()
        .push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"1","multiplierUp":"1","avgPriceMins":5}));
    *http.binance_exchange_info.borrow_mut() = Some(boundary_info);
    let proposal = testnet_proposal(&mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC");
    let mut submit = testnet_submit_request(&workspace, &account, &proposal);
    submit["payload"]["idempotencyKey"] = "testnet-order-percent-boundary".into();
    let reply = run(&mut cp, submit, &vault, &http);
    assert_eq!(reply["data"]["state"], "ACKNOWLEDGED", "{reply}");
    assert_eq!(http.binance_posts.borrow().len(), 1);
    assert!(
        http.calls
            .borrow()
            .iter()
            .any(|call| call.ends_with("/api/v3/avgPrice?symbol=BTCUSDT"))
    );
}

#[test]
fn testnet_order_book_keeps_exact_history_balances_and_account_boundaries() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let trade_x = connected_testnet_labeled(&mut cp, &vault, &http, &workspace, "History account");
    let proposal = testnet_proposal(&mut cp, &workspace, &trade_x, "LIMIT", "BASE", "0.1", "GTC");
    let submitted = run(
        &mut cp,
        testnet_submit_request(&workspace, &trade_x, &proposal),
        &vault,
        &http,
    );
    assert_eq!(submitted["data"]["state"], "ACKNOWLEDGED", "{submitted}");
    assert_eq!(submitted["data"].get("fill"), None);
    let attempt = call(
        &mut cp,
        "binance.testnet.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    )["data"]["attempt"]
        .clone();
    let client_order_id = attempt["clientOrderId"].as_str().unwrap();

    *http.binance_order_history.borrow_mut() = std::collections::HashMap::from([(
        "BTCUSDT".into(),
        vec![
            json!({
                "symbol":"BTCUSDT","orderId":"9007199254740997","clientOrderId":client_order_id,
                "side":"BUY","type":"LIMIT","timeInForce":"GTC","status":"PARTIALLY_FILLED",
                "price":"100","origQty":"0.1","origQuoteOrderQty":"0",
                "executedQty":"0.012345678901234567","cummulativeQuoteQty":"1.2345678901234567",
                "time":1788849500000u64,"updateTime":1788849510000u64
            }),
            json!({
                "symbol":"BTCUSDT","orderId":"9007199254741001","clientOrderId":"external-order-1",
                "side":"SELL","type":"LIMIT","timeInForce":"GTC","status":"FILLED",
                "price":"100","origQty":"2","origQuoteOrderQty":"0",
                "executedQty":"2","cummulativeQuoteQty":"200",
                "time":1788849400000u64,"updateTime":1788849500000u64
            }),
        ],
    )]);
    *http.binance_trade_history.borrow_mut() = std::collections::HashMap::from([(
        "BTCUSDT".into(),
        vec![json!({
            "symbol":"BTCUSDT","id":"9007199254741002","orderId":"9007199254740997",
            "price":"100","qty":"0.012345678901234567","quoteQty":"1.2345678901234567",
            "commission":"0.000000000000000003","commissionAsset":"BNB",
            "time":1788849510000u64,"isBuyer":true
        })],
    )]);

    let history = refresh_testnet_book(
        &mut cp,
        &vault,
        &http,
        &workspace,
        &trade_x,
        "HISTORY",
        Some("BTCUSDT"),
        None,
    );
    assert_eq!(history["ok"], true, "{history}");
    assert_eq!(history["data"]["status"], "CURRENT");
    let btc_history = history["data"]["history"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "BTCUSDT")
        .unwrap();
    assert_eq!(btc_history["complete"], true);
    assert!(btc_history["lastObservedAt"].is_string());
    assert_eq!(btc_history["pageCount"], 1);
    assert_eq!(btc_history["nextOrderId"], Value::Null);
    assert_eq!(btc_history["nextTradeId"], Value::Null);
    assert!(http.calls.borrow().iter().any(|path| {
        path.contains("/api/v3/allOrders?symbol=BTCUSDT&limit=1000&orderId=1&timestamp=")
    }));
    assert!(http.calls.borrow().iter().any(|path| {
        path.contains("/api/v3/myTrades?symbol=BTCUSDT&limit=1000&fromId=1&timestamp=")
    }));
    let local_order = history["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740997")
        .unwrap();
    assert_eq!(local_order["origin"], "TRADE_X");
    assert_eq!(local_order["attemptId"], attempt["attemptId"]);
    assert_eq!(local_order["filledQuantity"], "0.012345678901234567");
    assert_eq!(local_order["filledQuoteQuantity"], "1.2345678901234567");
    assert_eq!(local_order["remainingQuantity"], "0.087654321098765433");
    let external_order = history["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254741001")
        .unwrap();
    assert_eq!(external_order["origin"], "EXTERNAL");
    assert!(external_order.get("attemptId").is_none());
    assert_eq!(history["data"]["fills"][0]["tradeId"], "9007199254741002");
    assert_eq!(
        history["data"]["fills"][0]["commission"],
        "0.000000000000000003"
    );
    assert_eq!(history["data"]["fills"][0]["commissionAsset"], "BNB");

    let snapshot = call(
        &mut cp,
        "domain.snapshot",
        json!({"aggregateType":"binance-testnet-order-book","aggregateId":trade_x["connectionId"]}),
    );
    assert_eq!(snapshot["data"]["lastSequence"], 1);
    drop(cp);
    let mut reopened = ControlPlane::new(path);
    assert_eq!(call(&mut reopened, "workspace.open", json!({}))["ok"], true);
    let durable = call(
        &mut reopened,
        "binance.testnet.orders.get",
        json!({"workspaceId":workspace,"connectionId":trade_x["connectionId"]}),
    );
    assert_eq!(durable["data"]["book"]["status"], "STALE");
    assert_eq!(
        durable["data"]["book"]["orders"][0]["providerOrderId"],
        "9007199254740997"
    );
    assert_eq!(
        durable["data"]["book"]["fills"][0]["tradeId"],
        "9007199254741002"
    );

    http.binance_uid.set(9007199254740994);
    let balance_account =
        connected_testnet_labeled(&mut reopened, &vault, &http, &workspace, "Balance account");
    let balances = refresh_testnet_book(
        &mut reopened,
        &vault,
        &http,
        &workspace,
        &balance_account,
        "ACCOUNT",
        None,
        None,
    );
    assert_eq!(balances["data"]["status"], "CURRENT", "{balances}");
    assert!(balances["data"]["balancesObservedAt"].is_string());
    assert!(balances["data"]["pendingOrdersObservedAt"].is_null());
    let usdt = balances["data"]["balances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|balance| balance["asset"] == "USDT")
        .unwrap();
    assert_eq!(usdt["total"], "100000000000000000000.0000000000000000001");
    assert!(usdt.get("usdValue").is_none());
    let isolated_history = call(
        &mut reopened,
        "binance.testnet.orders.get",
        json!({"workspaceId":workspace,"connectionId":balance_account["connectionId"]}),
    );
    assert_eq!(isolated_history["data"]["book"]["orders"], json!([]));
    assert_eq!(
        isolated_history["data"]["book"]["balances"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn testnet_exact_order_refresh_accepts_any_saved_provider_symbol() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet_labeled(&mut cp, &vault, &http, &workspace, "Detail account");

    let pending = refresh_testnet_book(
        &mut cp, &vault, &http, &workspace, &account, "PENDING", None, None,
    );
    assert_eq!(pending["ok"], true, "{pending}");
    assert!(pending["data"]["pendingOrdersObservedAt"].is_string());
    assert!(pending["data"]["balancesObservedAt"].is_null());
    assert!(
        pending["data"]["orders"]
            .as_array()
            .unwrap()
            .iter()
            .any(|order| {
                order["symbol"] == "ODDCOINUSDT" && order["providerOrderId"] == "9007199254740996"
            })
    );

    let detail = refresh_testnet_book(
        &mut cp,
        &vault,
        &http,
        &workspace,
        &account,
        "DETAIL",
        Some("ODDCOINUSDT"),
        Some("9007199254740996"),
    );
    assert_eq!(detail["ok"], true, "{detail}");
    assert_eq!(detail["data"]["reason"], "PROVIDER_RATE_LIMITED");
}

#[test]
fn testnet_cancel_is_single_exact_delete_and_reconciles_terminal_status() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let pending = refresh_testnet_book(
        &mut cp, &vault, &http, &workspace, &account, "PENDING", None, None,
    );
    assert_eq!(pending["data"]["status"], "CURRENT", "{pending}");
    let cancelled = cancel_testnet_order(
        &mut cp,
        &vault,
        &http,
        &workspace,
        &account,
        &pending["data"],
        "9007199254740995",
        "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    );
    assert_eq!(cancelled["ok"], true, "{cancelled}");
    let order = cancelled["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(order["providerStatus"], "CANCELED");
    assert_eq!(order["cancelState"], "NONE");
    assert!(order["cancelIdempotencyKey"].is_null());
    assert_eq!(http.binance_cancel_calls.borrow().len(), 1);
    let path = http.binance_cancel_calls.borrow()[0].clone();
    assert!(path.starts_with("/api/v3/order?"));
    assert!(path.contains("symbol=BTCUSDT"));
    assert!(path.contains("orderId=9007199254740995"));
    assert!(path.contains("newClientOrderId=aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"));
    assert!(!path.contains("openOrders"));
}

#[test]
fn testnet_cancel_definitive_rejection_is_retained_and_never_replayed() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let pending = refresh_testnet_book(
        &mut cp, &vault, &http, &workspace, &account, "PENDING", None, None,
    );
    http.binance_cancel_status.set(Some(400));
    let rejected = cancel_testnet_order(
        &mut cp,
        &vault,
        &http,
        &workspace,
        &account,
        &pending["data"],
        "9007199254740995",
        "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee",
    );
    assert_eq!(rejected["ok"], true, "{rejected}");
    let order = rejected["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(order["providerStatus"], "NEW");
    assert_eq!(order["cancelState"], "NONE");
    assert_eq!(
        order["cancelIdempotencyKey"],
        "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"
    );
    assert_eq!(order["cancelError"], "PROVIDER_CANCEL_REJECTED");

    let replay = cp.prepare_provider_for(
        &request(
            "binance.testnet.orders.cancel",
            json!({
                "workspaceId":workspace,
                "connectionId":account["connectionId"],
                "expectedConnectionStateVersion":account["stateVersion"],
                "symbol":"BTCUSDT",
                "providerOrderId":"9007199254740995",
                "expectedBookStateVersion":rejected["data"]["stateVersion"],
                "idempotencyKey":"eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee",
                "confirmed":true
            }),
        ),
        "main",
    );
    let replay_error = match replay {
        Err(error) => error,
        Ok(_) => panic!("a rejected cancellation must not create a second provider job"),
    };
    assert_eq!(replay_error.code, "STATE_VERSION_CONFLICT");
    assert_eq!(http.binance_cancel_calls.borrow().len(), 1);
}

#[test]
fn testnet_cancel_reopen_recovers_unknown_intent_without_delete_or_replay() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let mut cp = ControlPlane::new(path.clone());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let pending = refresh_testnet_book(
        &mut cp, &vault, &http, &workspace, &account, "PENDING", None, None,
    );
    let cancel_key = "ffffffff-ffff-4fff-8fff-ffffffffffff";
    let job = cp
        .prepare_provider_for(
            &request(
                "binance.testnet.orders.cancel",
                json!({
                    "workspaceId":workspace,
                    "connectionId":account["connectionId"],
                    "expectedConnectionStateVersion":account["stateVersion"],
                    "symbol":"BTCUSDT",
                    "providerOrderId":"9007199254740995",
                    "expectedBookStateVersion":pending["data"]["stateVersion"],
                    "idempotencyKey":cancel_key,
                    "confirmed":true
                }),
            ),
            "main",
        )
        .unwrap()
        .unwrap();
    assert!(http.binance_cancel_calls.borrow().is_empty());
    drop(job);
    drop(cp);

    let mut reopened = ControlPlane::new(path);
    let opened = call(&mut reopened, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let recovered = call(
        &mut reopened,
        "binance.testnet.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(recovered["ok"], true, "{recovered}");
    assert_eq!(recovered["data"]["book"]["status"], "DEGRADED");
    let order = recovered["data"]["book"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(order["cancelState"], "PENDING");
    assert_eq!(order["cancelError"], "ORDER_CANCEL_STATUS_UNKNOWN");
    assert_eq!(order["cancelIdempotencyKey"], cancel_key);

    let account_after_open = call(
        &mut reopened,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    let probed = run(
        &mut reopened,
        request("provider.probe", mutation(&account_after_open["data"])),
        &vault,
        &http,
    );
    assert_eq!(probed["ok"], true, "{probed}");
    let replay = reopened.prepare_provider_for(
        &request(
            "binance.testnet.orders.cancel",
            json!({
                "workspaceId":workspace,
                "connectionId":account["connectionId"],
                "expectedConnectionStateVersion":probed["data"]["stateVersion"],
                "symbol":"BTCUSDT",
                "providerOrderId":"9007199254740995",
                "expectedBookStateVersion":recovered["data"]["book"]["stateVersion"],
                "idempotencyKey":cancel_key,
                "confirmed":true
            }),
        ),
        "main",
    );
    let replay_error = match replay {
        Err(error) => error,
        Ok(_) => panic!("a recovered cancellation must not create a second provider job"),
    };
    assert_eq!(replay_error.code, "STATE_VERSION_CONFLICT");
    assert!(http.binance_cancel_calls.borrow().is_empty());
}

#[test]
fn testnet_cancel_rechecks_exact_order_and_never_resends_unknown_outcome() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let pending = refresh_testnet_book(
        &mut cp, &vault, &http, &workspace, &account, "PENDING", None, None,
    );
    let mut remote_orders = vec![json!({
        "symbol":"BTCUSDT","orderId":9007199254740995u64,"clientOrderId":"fixture-open-btc",
        "side":"BUY","type":"LIMIT","timeInForce":"GTC","status":"NEW",
        "price":"100.2","origQty":"0.1","origQuoteOrderQty":"0","executedQty":"0",
        "cummulativeQuoteQty":"0","time":1788849500000u64,"updateTime":1788849500000u64
    })];
    remote_orders[0]["status"] = "PARTIALLY_FILLED".into();
    remote_orders[0]["executedQty"] = "0.02".into();
    remote_orders[0]["cummulativeQuoteQty"] = "2.004".into();
    remote_orders[0]["updateTime"] = 1788849502000u64.into();
    *http.binance_open_orders.borrow_mut() = Some(remote_orders);
    let changed = cancel_testnet_order(
        &mut cp,
        &vault,
        &http,
        &workspace,
        &account,
        &pending["data"],
        "9007199254740995",
        "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    );
    assert_eq!(changed["ok"], true, "{changed}");
    assert_eq!(http.binance_cancel_calls.borrow().len(), 0);
    let changed_order = changed["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(changed_order["providerStatus"], "PARTIALLY_FILLED");
    assert_eq!(changed_order["filledQuantity"], "0.02");
    assert_eq!(changed_order["cancelError"], "ORDER_CHANGED_REVIEW_AGAIN");

    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let uncertain_workspace =
        call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &uncertain_workspace);
    let pending = refresh_testnet_book(
        &mut cp,
        &vault,
        &http,
        &uncertain_workspace,
        &account,
        "PENDING",
        None,
        None,
    );
    http.binance_cancel_timeout.set(true);
    let uncertain = cancel_testnet_order(
        &mut cp,
        &vault,
        &http,
        &uncertain_workspace,
        &account,
        &pending["data"],
        "9007199254740995",
        "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    );
    assert_eq!(uncertain["ok"], true, "{uncertain}");
    let uncertain_order = uncertain["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(uncertain_order["cancelState"], "PENDING");
    assert_eq!(
        uncertain_order["cancelError"],
        "ORDER_CANCEL_STATUS_UNKNOWN"
    );
    http.binance_cancel_timeout.set(false);
    let replay = cp.prepare_provider_for(
        &request(
            "binance.testnet.orders.cancel",
            json!({
                "workspaceId":uncertain_workspace,
                "connectionId":account["connectionId"],
                "expectedConnectionStateVersion":account["stateVersion"],
                "symbol":"BTCUSDT",
                "providerOrderId":"9007199254740995",
                "expectedBookStateVersion":uncertain["data"]["stateVersion"],
                "idempotencyKey":"cccccccc-cccc-4ccc-8ccc-cccccccccccc",
                "confirmed":true
            }),
        ),
        "main",
    );
    let replay_error = match replay {
        Err(error) => error,
        Ok(_) => panic!("an uncertain cancellation must not create a second provider job"),
    };
    assert_eq!(replay_error.code, "STATE_VERSION_CONFLICT");
    assert_eq!(http.binance_cancel_calls.borrow().len(), 1);
}

#[test]
fn testnet_cancel_completion_merges_concurrent_partial_and_full_private_stream_fills() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let pending = refresh_testnet_book(
        &mut cp, &vault, &http, &workspace, &account, "PENDING", None, None,
    );
    let job = cp
        .prepare_provider_for(
            &request(
                "binance.testnet.orders.cancel",
                json!({
                    "workspaceId":workspace,
                    "connectionId":account["connectionId"],
                    "expectedConnectionStateVersion":account["stateVersion"],
                    "symbol":"BTCUSDT",
                    "providerOrderId":"9007199254740995",
                    "expectedBookStateVersion":pending["data"]["stateVersion"],
                    "idempotencyKey":"dddddddd-dddd-4ddd-8ddd-dddddddddddd",
                    "confirmed":true
                }),
            ),
            "main",
        )
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let secrets = vec![fixtures::KEY.to_owned(), fixtures::SECRET.to_owned()];
    let cancel_update_time = http
        .binance_open_orders
        .borrow()
        .as_ref()
        .unwrap()
        .iter()
        .find(|order| order["orderId"] == 9007199254740995u64)
        .unwrap()["updateTime"]
        .as_u64()
        .unwrap();
    let execution_report = |status: &str,
                            filled: &str,
                            quote: &str,
                            update_time: u64,
                            trade_id: u64,
                            last_quantity: &str| {
        json!({
            "subscriptionId":7,
            "event":{
                "e":"executionReport","E":update_time,"s":"BTCUSDT","c":"fixture-open-btc",
                "S":"BUY","o":"LIMIT","f":"GTC","q":"0.1","p":"100.2",
                "x":"TRADE","X":status,"i":9007199254740995u64,
                "l":last_quantity,"z":filled,"L":"100.2","n":"0.001","N":"USDT",
                "T":update_time,"t":trade_id,"O":1788849500000u64,"Q":"0","Y":quote,"Z":quote
            }
        })
    };
    let partial = execution_report(
        "PARTIALLY_FILLED",
        "0.04",
        "4.008",
        cancel_update_time + 1,
        9201,
        "0.04",
    );
    cp.apply_binance_private_stream_frame(
        account["connectionId"].as_str().unwrap(),
        account["stateVersion"].as_str().unwrap(),
        "9007199254740993",
        &partial,
        7,
        &secrets,
    )
    .unwrap();
    let interim = call(
        &mut cp,
        "binance.testnet.orders.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(interim["ok"], true, "{interim}");
    let interim_order = interim["data"]["book"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(interim_order["filledQuantity"], "0.04", "{interim}");
    assert_eq!(
        interim_order["providerStatus"], "PARTIALLY_FILLED",
        "{interim}"
    );

    let full = execution_report(
        "FILLED",
        "0.1",
        "10.02",
        cancel_update_time + 2,
        9202,
        "0.06",
    );
    cp.apply_binance_private_stream_frame(
        account["connectionId"].as_str().unwrap(),
        account["stateVersion"].as_str().unwrap(),
        "9007199254740993",
        &full,
        7,
        &secrets,
    )
    .unwrap();
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["reason"], "ORDER_BOOK_CHANGED_DURING_CANCEL");
    let order = reply["data"]["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740995")
        .unwrap();
    assert_eq!(order["providerStatus"], "FILLED", "{reply}");
    assert_eq!(order["filledQuantity"], "0.1");
    assert_eq!(order["cancelState"], "NONE", "{reply}");
    assert!(order["cancelIdempotencyKey"].is_null());
    let trade_ids = reply["data"]["fills"]
        .as_array()
        .unwrap()
        .iter()
        .map(|fill| fill["tradeId"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(trade_ids, ["9201", "9202"].into_iter().collect());
    assert_eq!(http.binance_cancel_calls.borrow().len(), 1);
}

#[test]
fn testnet_order_history_stops_after_one_provider_page_and_keeps_exact_next_id() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let workspace = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account = connected_testnet(&mut cp, &vault, &http, &workspace);
    let orders = (1u64..=1000)
        .map(|offset| {
            let id = 9007199254740000u64 + offset;
            json!({
                "symbol":"BTCUSDT","orderId":id.to_string(),"clientOrderId":format!("external-{offset}"),
                "side":"BUY","type":"LIMIT","timeInForce":"GTC","status":"FILLED",
                "price":"100","origQty":"1","origQuoteOrderQty":"0",
                "executedQty":"1","cummulativeQuoteQty":"100",
                "time":1788849500000u64 + offset,"updateTime":1788849500000u64 + offset
            })
        })
        .collect::<Vec<_>>();
    http.binance_order_history
        .borrow_mut()
        .insert("BTCUSDT".into(), orders);
    http.binance_trade_history
        .borrow_mut()
        .insert("BTCUSDT".into(), vec![]);

    let page = refresh_testnet_book(
        &mut cp,
        &vault,
        &http,
        &workspace,
        &account,
        "HISTORY",
        Some("BTCUSDT"),
        None,
    );
    assert_eq!(page["ok"], true, "{page}");
    assert_eq!(page["data"]["orders"].as_array().unwrap().len(), 1000);
    let btc_history = page["data"]["history"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "BTCUSDT")
        .unwrap();
    assert_eq!(btc_history["pageCount"], 1);
    assert_eq!(btc_history["complete"], false);
    assert_eq!(btc_history["nextOrderId"], "9007199254741001");
    assert_eq!(btc_history["nextTradeId"], Value::Null);
    assert_eq!(
        http.calls
            .borrow()
            .iter()
            .filter(|path| path.contains("/api/v3/allOrders?"))
            .count(),
        1
    );
}

#[test]
fn spot_connections_keep_native_assets_scope_and_environment_separate() {
    lifecycle(&fixtures::Vault::default());
}
#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit OS Keychain check with disposable Binance credentials"]
fn native_keychain_stores_and_removes_spot_credentials() {
    lifecycle(&tradex::provider_io::NativeVault);
}

#[derive(Default)]
struct Response {
    route: &'static str,
    body: Option<Value>,
    error: Option<&'static str>,
    delay_time: bool,
    calls: std::cell::RefCell<Vec<String>>,
}
impl ProviderHttp for Response {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        let route = path.split('?').next().unwrap();
        self.calls.borrow_mut().push(route.into());
        if route == self.route {
            if self.delay_time {
                std::thread::sleep(std::time::Duration::from_millis(2050));
            }
            if let Some(code) = self.error {
                return Err(tradex::protocol::TradeXError::new(code));
            }
            if let Some(body) = &self.body {
                return Ok(serde_json::to_vec(body).unwrap());
            }
        }
        fixtures::Http::default().get(endpoint, path, headers)
    }
}
fn restrictions() -> Value {
    json!({"ipRestrict":true,"enableReading":true,"enableWithdrawals":false,"enableInternalTransfer":false,"permitsUniversalTransfer":false,"enableMargin":false,"enableFutures":false,"enableVanillaOptions":false,"enablePortfolioMarginTrading":false,"enableFixApiTrade":false,"enableFixReadOnly":false,"enableSpotAndMarginTrading":true})
}
#[test]
fn scope_changes_and_forbidden_permissions_cannot_be_acknowledged_away() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = fixtures::Vault::default();
    let mut a=run(&mut cp,request("provider.connect",json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":"LIVE","label":"Permission gate"})),&vault,&fixtures::Http::default())["data"].clone();
    for flag in [
        "enableWithdrawals",
        "enableInternalTransfer",
        "permitsUniversalTransfer",
        "enableMargin",
        "enableFutures",
        "enableVanillaOptions",
        "enablePortfolioMarginTrading",
        "enableFixApiTrade",
        "enableReading",
    ] {
        let mut confirm = mutation(&a);
        confirm["step"] = "confirm".into();
        confirm["acknowledgeUnverified"] = true.into();
        let connected = call(&mut cp, "provider.connect", confirm);
        assert_eq!(connected["ok"], true, "{connected}");
        a = connected["data"].clone();
        let mut changed = restrictions();
        changed[flag] = (flag != "enableReading").into();
        // Incomplete inspection still retains every observed dangerous authority.
        changed.as_object_mut().unwrap().remove("enableFixReadOnly");
        let http = Response {
            route: "/sapi/v1/account/apiRestrictions",
            body: Some(changed),
            ..Default::default()
        };
        let probe = run(
            &mut cp,
            request("provider.probe", mutation(&a)),
            &vault,
            &http,
        );
        assert_eq!(probe["ok"], true, "{probe}");
        a = probe["data"].clone();
        assert_eq!(a["connectionState"], "REVIEW_REQUIRED");
        assert_eq!(a["permissions"]["scope"], "UNVERIFIED");
        assert_eq!(a["permissions"]["acknowledged"], false);
        let mut confirm = mutation(&a);
        confirm["step"] = "confirm".into();
        confirm["acknowledgeUnverified"] = true.into();
        assert_eq!(
            call(&mut cp, "provider.connect", confirm)["error"]["code"],
            "PROVIDER_PERMISSION_BLOCKED",
            "{flag}"
        );
        let recovered = run(
            &mut cp,
            request("provider.probe", mutation(&a)),
            &vault,
            &fixtures::Http::default(),
        );
        a = recovered["data"].clone();
    }
    let mut changed = restrictions();
    changed["futureUnknownPermission"] = true.into();
    changed["ipRestrict"] = false.into();
    let probe = run(
        &mut cp,
        request("provider.probe", mutation(&a)),
        &vault,
        &Response {
            route: "/sapi/v1/account/apiRestrictions",
            body: Some(changed),
            ..Default::default()
        },
    );
    assert_eq!(probe["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(
        probe["data"]["permissions"]["ipAllowListStatus"],
        "UNRESTRICTED"
    );
    let disconnected = run(
        &mut cp,
        request("provider.disconnect", mutation(&probe["data"])),
        &vault,
        &fixtures::Http::default(),
    );
    assert_eq!(disconnected["ok"], true);
}
#[test]
fn time_identity_and_incomplete_observations_fail_without_replacing_saved_data() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = fixtures::Vault::default();
    let original=run(&mut cp,request("provider.connect",json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":"LIVE","label":"Faults"})),&vault,&fixtures::Http::default())["data"].clone();
    let mut a = original.clone();
    for (route, body, code) in [
        (
            "/api/v3/account",
            json!({"uid":9007199254740993u64,"accountType":"SPOT","canTrade":true,"canWithdraw":true,"canDeposit":true,"balances":[{"asset":"BTC","free":"-0.01","locked":"0"}]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/account",
            json!({"uid":9007199254740993u64,"accountType":"SPOT","canTrade":true,"canWithdraw":true,"canDeposit":true,"balances":[{"asset":"BTC","free":"1","locked":"0"},{"asset":"BTC","free":"2","locked":"0"}]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/openOrders",
            json!([{"symbol":"BTCUSDT","orderId":1,"side":"BUY","status":"NEW","price":"2","origQty":"1","origQuoteOrderQty":"0","executedQty":"0"},{"symbol":"BTCUSDT","orderId":1,"side":"BUY","status":"NEW","price":"2","origQty":"1","origQuoteOrderQty":"0","executedQty":"0"}]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        ("/api/v3/time", json!({"serverTime":-1}), "CLOCK_SKEW"),
        (
            "/api/v3/time",
            json!({"serverTime":1788849600000000u64}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v3/time",
            json!({"serverTime":"1788849600000"}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v3/account",
            json!({"uid":123}),
            "PROVIDER_IDENTITY_CHANGED",
        ),
        (
            "/api/v3/account",
            json!({"uid":9007199254740993u64,"accountType":"MARGIN"}),
            "PROVIDER_UNSUPPORTED",
        ),
        (
            "/api/v3/openOrders",
            json!({"items":[],"next":"https://example.invalid"}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/openOrders",
            json!([{"symbol":fixtures::SECRET}]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/sapi/v1/account/apiRestrictions",
            json!({"enableWithdrawals":"false"}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            "/api/v3/openOrders",
            Value::Array(vec![json!({}); 10001]),
            "PROVIDER_DATA_INCOMPLETE",
        ),
    ] {
        let http = Response {
            route,
            body: Some(body),
            ..Default::default()
        };
        let reply = run(
            &mut cp,
            request("account.refresh", mutation(&a)),
            &vault,
            &http,
        );
        assert_eq!(reply["error"]["code"], code, "{reply}");
        if route == "/api/v3/time" {
            assert_eq!(http.calls.borrow().len(), 1);
            assert_eq!(reply["error"]["category"], "STATE_STALE");
        }
        if code == "PROVIDER_IDENTITY_CHANGED" {
            assert_eq!(http.calls.borrow().len(), 2);
        }
        a = call(
            &mut cp,
            "account.get",
            json!({"workspaceId":ws,"connectionId":a["connectionId"]}),
        )["data"]
            .clone();
        assert_eq!(a["data"], original["data"]);
        assert_eq!(a["lastSuccessfulSync"], original["lastSuccessfulSync"]);
    }
    for (route, error, delay_time) in [
        ("/api/v3/time", None, true),
        ("/api/v3/account", Some("CLOCK_SKEW"), false),
        (
            "/sapi/v1/account/apiRestrictions",
            Some("PROVIDER_RATE_LIMITED"),
            false,
        ),
        (
            "/sapi/v1/account/apiRestrictions",
            Some("PROVIDER_AUTH_FAILED"),
            false,
        ),
    ] {
        let http = Response {
            route,
            error,
            delay_time,
            ..Default::default()
        };
        let reply = run(
            &mut cp,
            request("provider.probe", mutation(&a)),
            &vault,
            &http,
        );
        assert_eq!(
            reply["error"]["code"],
            error.unwrap_or("CLOCK_SKEW"),
            "{reply}"
        );
        a = call(
            &mut cp,
            "account.get",
            json!({"workspaceId":ws,"connectionId":a["connectionId"]}),
        )["data"]
            .clone();
        assert_eq!(a["data"], original["data"]);
        assert_eq!(a["lastSuccessfulSync"], original["lastSuccessfulSync"]);
    }
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().into());
    call(&mut cp, "workspace.open", json!({}));
    a = call(
        &mut cp,
        "account.get",
        json!({"workspaceId":ws,"connectionId":a["connectionId"]}),
    )["data"]
        .clone();
    assert_eq!(a["health"]["connection"], "STALE");
    let refreshed = run(
        &mut cp,
        request("provider.probe", mutation(&a)),
        &vault,
        &fixtures::Http::default(),
    );
    assert_eq!(refreshed["ok"], true, "{refreshed}");
    let disconnected = run(
        &mut cp,
        request("provider.disconnect", mutation(&refreshed["data"])),
        &vault,
        &fixtures::Http::default(),
    );
    assert_eq!(disconnected["ok"], true);
    for entry in std::fs::read_dir(folder.path()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let bytes = std::fs::read(path).unwrap();
            for secret in [fixtures::KEY, fixtures::SECRET] {
                assert!(!bytes.windows(secret.len()).any(|b| b == secret.as_bytes()));
            }
        }
    }
}

#[test]
fn invalidation_during_server_time_prevents_any_signed_request_and_cleans_new_key() {
    struct Invalidated(std::cell::Cell<bool>);
    impl ProviderHttp for Invalidated {
        fn get(
            &self,
            endpoint: tradex::provider_io::ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            assert_eq!(
                path, "/api/v3/time",
                "No signed read may follow an invalidated job"
            );
            let response = fixtures::Http::default().get(endpoint, path, headers);
            self.0.set(true);
            response
        }
    }
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = call(&mut cp, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"binance","environment":"TESTNET","label":"Invalidated time sample"}),
    );
    let vault = fixtures::Vault::default();
    let http = Invalidated(std::cell::Cell::new(false));
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let reference = job.connection().credential_ref();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || !http.0.get() && cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["error"]["code"], "STATE_VERSION_CONFLICT");
    assert!(vault.get(&reference).is_err());
}
