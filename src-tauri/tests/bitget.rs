use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};
use tradex::{
    ControlPlane,
    protocol::{Result, TradeXError},
    provider_io::{
        CredentialVault, Credentials, ProviderEndpoint, ProviderHttp, ProviderHttpMethod,
        ProviderHttpResponse,
    },
};

#[path = "support/bitget_fixtures.rs"]
mod bitget_fixtures;
use bitget_fixtures::{Http as LivePages, KEY, PASSPHRASE, SECRET, credentials};
#[derive(Default)]
struct Vault(RefCell<Option<String>>);
impl CredentialVault for Vault {
    fn put(&self, reference: &str, _: &Credentials) -> Result<()> {
        assert!(self.0.borrow().is_none());
        *self.0.borrow_mut() = Some(reference.into());
        Ok(())
    }
    fn get(&self, reference: &str) -> Result<Credentials> {
        if self.0.borrow().as_deref() != Some(reference) {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        credentials()
    }
    fn remove(&self, reference: &str) -> Result<()> {
        if self.0.borrow().as_deref() == Some(reference) {
            self.0.borrow_mut().take();
        }
        Ok(())
    }
}
struct DemoUnsupported(RefCell<Vec<String>>);
impl ProviderHttp for DemoUnsupported {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        assert_eq!(endpoint.base_url(), "https://api.bitget.com");
        self.0.borrow_mut().push(path.into());
        if path == "/api/v2/public/time" {
            assert!(headers.is_empty());
            return Ok(br#"{"code":"00000","data":{"serverTime":"1788849600000"}}"#.to_vec());
        }
        assert_eq!(path, "/api/v2/spot/account/info");
        assert_eq!(headers["paptrading"], "1");
        assert_eq!(headers["ACCESS-KEY"], KEY);
        assert_eq!(headers["ACCESS-PASSPHRASE"], PASSPHRASE);
        for name in ["ACCESS-KEY", "ACCESS-PASSPHRASE", "ACCESS-SIGN"] {
            assert!(headers[name].is_sensitive());
        }
        let timestamp = headers["ACCESS-TIMESTAMP"].to_str().unwrap();
        assert!((1788849600000..1788849660000).contains(&timestamp.parse::<u64>().unwrap()));
        use base64::Engine;
        use hmac::Mac;
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(SECRET.as_bytes()).unwrap();
        mac.update(format!("{timestamp}GET{path}").as_bytes());
        mac.verify_slice(
            &base64::engine::general_purpose::STANDARD
                .decode(headers["ACCESS-SIGN"].as_bytes())
                .unwrap(),
        )
        .unwrap();
        Ok(
            br#"{"code":"40081","msg":"synthetic Demo account endpoint restriction","data":null}"#
                .to_vec(),
        )
    }
}
fn request(command: &str, payload: Value) -> Value {
    static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(0);
    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    json!({"requestId":format!("bitget-test-{request_id}"),"schemaVersion":1,"command":command,"payload":payload})
}

#[derive(Clone, Copy, Default)]
enum DemoPostMode {
    #[default]
    Ack,
    Timeout,
    RateLimit,
    Rejected,
    Redirect,
    BadJson,
    WrongClientOid,
}

struct DemoOrders {
    calls: RefCell<Vec<(ProviderEndpoint, ProviderHttpMethod, String, bool)>>,
    posts: RefCell<Vec<Value>>,
    last_order: RefCell<Option<Value>>,
    account_info_override: RefCell<Option<Value>>,
    post_mode: Cell<DemoPostMode>,
}

impl Default for DemoOrders {
    fn default() -> Self {
        Self {
            calls: RefCell::new(vec![]),
            posts: RefCell::new(vec![]),
            last_order: RefCell::new(None),
            account_info_override: RefCell::new(None),
            post_mode: Cell::new(DemoPostMode::Ack),
        }
    }
}

impl DemoOrders {
    fn respond(
        &self,
        endpoint: ProviderEndpoint,
        method: ProviderHttpMethod,
        path: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<&Value>,
    ) -> Result<ProviderHttpResponse> {
        assert_eq!(endpoint, ProviderEndpoint::BitgetDemo);
        let demo_header = headers
            .get("paptrading")
            .and_then(|value| value.to_str().ok())
            == Some("1");
        self.calls
            .borrow_mut()
            .push((endpoint, method, path.into(), demo_header));
        let envelope = |data: Value| json!({"code":"00000","data":data});
        let response = match (method, path) {
            (ProviderHttpMethod::Get, "/api/v2/public/time") => {
                assert!(headers.is_empty());
                json!({"code":"00000","data":{"serverTime":"1788849600000"}})
            }
            (ProviderHttpMethod::Get, "/api/v2/spot/account/info") => {
                assert!(demo_header);
                assert_signed("GET", path, None, &headers);
                envelope(self.account_info_override.borrow().clone().unwrap_or_else(|| {
                    json!({"userId":"9007199254740993","authorities":["coor","cpor","stor"],"ips":""})
                }))
            }
            (ProviderHttpMethod::Get, "/api/v2/spot/account/assets?assetType=all") => {
                assert!(demo_header);
                assert_signed("GET", path, None, &headers);
                envelope(json!([]))
            }
            (ProviderHttpMethod::Get, p)
                if p.starts_with("/api/v2/spot/trade/unfilled-orders?") =>
            {
                assert!(demo_header);
                assert_signed("GET", path, None, &headers);
                envelope(json!([]))
            }
            (ProviderHttpMethod::Get, "/api/v2/spot/trade/current-plan-order?limit=100") => {
                assert!(demo_header);
                assert_signed("GET", path, None, &headers);
                envelope(json!({"orderList":[],"nextFlag":false}))
            }
            (ProviderHttpMethod::Get, "/api/v2/spot/public/symbols?symbol=BTCUSDT") => {
                envelope(json!([
                    {"symbol":"BTCUSDT","baseCoin":"BTC","quoteCoin":"USDT","status":"online","pricePrecision":"2","quantityPrecision":"6","quotePrecision":"8","minTradeUSDT":"1"}
                ]))
            }
            (ProviderHttpMethod::Get, "/api/v2/spot/market/tickers?symbol=BTCUSDT") => {
                envelope(json!([
                    {"symbol":"BTCUSDT","bidPr":"65000","askPr":"65001","lastPr":"65000"}
                ]))
            }
            (ProviderHttpMethod::Get, p)
                if p.starts_with("/api/v2/spot/trade/orderInfo?clientOid=") =>
            {
                assert!(demo_header);
                assert_signed("GET", path, None, &headers);
                envelope(json!([self
                    .last_order
                    .borrow()
                    .clone()
                    .expect("submitted order fixture")]))
            }
            (ProviderHttpMethod::Post, "/api/v2/spot/trade/place-order") => {
                assert!(demo_header);
                let body = body.expect("Bitget order body");
                assert_signed("POST", path, Some(body), &headers);
                self.posts.borrow_mut().push(body.clone());
                let order = json!({
                    "userId":"9007199254740993","symbol":body["symbol"],"orderId":"123456789",
                    "clientOid":body["clientOid"],"side":body["side"],"orderType":body["orderType"],"status":"live"
                });
                *self.last_order.borrow_mut() = Some(order.clone());
                let mode = self.post_mode.get();
                return match mode {
                    DemoPostMode::Timeout => Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
                    DemoPostMode::RateLimit => Ok(ProviderHttpResponse {
                        status: 429,
                        body: br#"{"code":"429","msg":"rate limited"}"#.to_vec(),
                    }),
                    DemoPostMode::Rejected => Ok(ProviderHttpResponse {
                        status: 400,
                        body: br#"{"code":"40017","msg":"order rejected"}"#.to_vec(),
                    }),
                    DemoPostMode::Redirect => Ok(ProviderHttpResponse {
                        status: 302,
                        body: vec![],
                    }),
                    DemoPostMode::BadJson => Ok(ProviderHttpResponse {
                        status: 200,
                        body: b"not json".to_vec(),
                    }),
                    DemoPostMode::WrongClientOid => Ok(ProviderHttpResponse {
                        status: 200,
                        body: serde_json::to_vec(&envelope(json!({
                            "orderId":"123456789","clientOid":"tx-wrong"
                        })))
                        .unwrap(),
                    }),
                    DemoPostMode::Ack => Ok(ProviderHttpResponse {
                        status: 200,
                        body: serde_json::to_vec(&envelope(json!({
                            "orderId":order["orderId"],"clientOid":order["clientOid"]
                        })))
                        .unwrap(),
                    }),
                };
            }
            _ => panic!(
                "unexpected Bitget fixture request: {} {path}",
                if method == ProviderHttpMethod::Get {
                    "GET"
                } else {
                    "POST"
                }
            ),
        };
        Ok(ProviderHttpResponse {
            status: 200,
            body: serde_json::to_vec(&response).unwrap(),
        })
    }
}

fn assert_signed(
    method: &str,
    path: &str,
    body: Option<&Value>,
    headers: &reqwest::header::HeaderMap,
) {
    assert_eq!(headers["ACCESS-KEY"], KEY);
    assert_eq!(headers["ACCESS-PASSPHRASE"], PASSPHRASE);
    assert!(headers["ACCESS-KEY"].is_sensitive());
    assert!(headers["ACCESS-PASSPHRASE"].is_sensitive());
    assert!(headers["ACCESS-SIGN"].is_sensitive());
    let timestamp = headers["ACCESS-TIMESTAMP"].to_str().unwrap();
    let body = body
        .map(|body| serde_json::to_string(body).unwrap())
        .unwrap_or_default();
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(SECRET.as_bytes()).unwrap();
    use base64::Engine;
    use hmac::Mac;
    mac.update(format!("{timestamp}{method}{path}{body}").as_bytes());
    mac.verify_slice(
        &base64::engine::general_purpose::STANDARD
            .decode(headers["ACCESS-SIGN"].as_bytes())
            .unwrap(),
    )
    .unwrap();
}

impl ProviderHttp for DemoOrders {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        Ok(self
            .respond(endpoint, ProviderHttpMethod::Get, path, headers, None)?
            .body)
    }

    fn request(
        &self,
        endpoint: ProviderEndpoint,
        method: ProviderHttpMethod,
        path: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<&Value>,
    ) -> Result<ProviderHttpResponse> {
        self.respond(endpoint, method, path, headers, body)
    }
}

fn run_provider(
    cp: &mut ControlPlane,
    req: Value,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let job = cp.prepare_provider_for(&req, "main").unwrap().unwrap();
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

fn connected_demo(
    cp: &mut ControlPlane,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    workspace: &Value,
) -> Value {
    let tested = run_provider(
        cp,
        request(
            "provider.connect",
            json!({"step":"test","workspaceId":workspace,"providerId":"bitget","environment":"DEMO","label":"S20 fixture"}),
        ),
        vault,
        http,
    );
    assert_eq!(tested["ok"], true, "{tested}");
    let confirmed = cp.dispatch(request(
        "provider.connect",
        json!({
            "step":"confirm","workspaceId":workspace,
            "connectionId":tested["data"]["connectionId"],
            "expectedStateVersion":tested["data"]["stateVersion"],
            "acknowledgeUnverified":false
        }),
    ));
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    confirmed["data"].clone()
}

fn demo_proposal(
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
        "accountId":account["connectionId"],"venue":"BITGET",
        "environment":"BITGET_DEMO","instrumentId":"crypto:BTC/USDT:spot",
        "side":side,"orderType":order_type,
        "quantity":{"type":quantity_type,"value":quantity},"timeInForce":tif
    });
    if order_type == "LIMIT" {
        fields["limitPrice"] = "65000".into();
    }
    let draft = cp.dispatch(request(
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":fields}),
    ));
    assert_eq!(draft["ok"], true, "{draft}");
    let generated = cp.dispatch(request(
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    ));
    assert_eq!(generated["ok"], true, "{generated}");
    cp.dispatch(request(
        "trade.proposal.get",
        json!({"workspaceId":workspace,"proposalId":generated["data"]["proposalId"]}),
    ))["data"]
        .clone()
}

fn demo_submit_request(workspace: &Value, account: &Value, proposal: &Value) -> Value {
    request(
        "bitget.demo.order.submit",
        json!({
            "workspaceId":workspace,"connectionId":account["connectionId"],
            "expectedConnectionStateVersion":account["stateVersion"],
            "proposalId":proposal["proposalId"],
            "expectedProposalStateVersion":proposal["stateVersion"],
            "proposalHash":proposal["proposalHash"],
            "idempotencyKey":"bitget-demo-order-1","confirmedDemoOrder":true
        }),
    )
}

#[test]
fn demo_proposal_is_persisted_before_one_signed_paptrading_submit_and_ack_is_not_fill() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = Vault::default();
    let http = DemoOrders::default();
    let workspace =
        cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let account = connected_demo(&mut cp, &vault, &http, &workspace);
    let proposal = demo_proposal(
        &mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC", "BUY",
    );
    let submit = demo_submit_request(&workspace, &account, &proposal);

    assert_eq!(
        cp.prepare_provider_for(&submit, "research")
            .err()
            .unwrap()
            .code,
        "ORDER_SUBMIT_FORBIDDEN"
    );
    let job = cp.prepare_provider_for(&submit, "main").unwrap().unwrap();
    let saved = cp.dispatch(request(
        "bitget.demo.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    ));
    assert_eq!(saved["data"]["attempt"]["state"], "SUBMITTING");
    assert!(
        http.posts.borrow().is_empty(),
        "persist before provider I/O"
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
    assert_eq!(reply["data"]["providerOrderId"], "123456789");
    assert_eq!(
        reply["data"].get("fill"),
        None,
        "acknowledgement is not fill"
    );
    assert_eq!(http.posts.borrow().len(), 1);
    let body = http.posts.borrow()[0].clone();
    assert_eq!(body["symbol"], "BTCUSDT");
    assert_eq!(body["side"], "buy");
    assert_eq!(body["orderType"], "limit");
    assert_eq!(body["force"], "gtc");
    assert_eq!(body["size"], "0.1");
    assert_eq!(body["price"], "65000");
    assert!(body["clientOid"].as_str().unwrap().starts_with("tx-"));
    assert!(body["clientOid"].as_str().unwrap().len() <= 50);
    assert!(cp.prepare_provider_for(&submit, "main").unwrap().is_none());
    assert_eq!(
        http.posts.borrow().len(),
        1,
        "duplicate must not POST twice"
    );
    assert!(
        http.calls
            .borrow()
            .iter()
            .all(|(endpoint, method, path, paptrading)| {
                *endpoint == ProviderEndpoint::BitgetDemo
                    && (!(*method == ProviderHttpMethod::Post
                        || path.starts_with("/api/v2/spot/account/")
                        || path.starts_with("/api/v2/spot/trade/"))
                        || *paptrading)
            })
    );
    let encoded = serde_json::to_string(&reply).unwrap();
    for secret in [KEY, SECRET, PASSPHRASE] {
        assert!(!encoded.contains(secret));
    }
}

#[test]
fn changed_remote_permission_scope_blocks_demo_write_before_post() {
    for (account_info, expected_error) in [
        (
            json!({"userId":"9007199254740993","authorities":["coor","cpor","stor"],"ips":"127.0.0.1"}),
            "PROVIDER_REVIEW_REQUIRED",
        ),
        (
            json!({"userId":"9007199254740993","authorities":["coor","cpor","stor","wtow"],"ips":""}),
            "PROVIDER_PERMISSION_BLOCKED",
        ),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let vault = Vault::default();
        let http = DemoOrders::default();
        let workspace =
            cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let account = connected_demo(&mut cp, &vault, &http, &workspace);
        let proposal = demo_proposal(
            &mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC", "BUY",
        );
        *http.account_info_override.borrow_mut() = Some(account_info);

        let reply = run_provider(
            &mut cp,
            demo_submit_request(&workspace, &account, &proposal),
            &vault,
            &http,
        );
        assert_eq!(reply["ok"], true, "{reply}");
        assert_eq!(reply["data"]["state"], "REJECTED");
        assert_eq!(reply["data"]["errorCode"], expected_error);
        assert!(
            http.posts.borrow().is_empty(),
            "permission drift must not POST"
        );
    }
}

#[test]
fn limit_and_market_orders_use_bitget_units_and_time_in_force() {
    let cases = [
        (
            "LIMIT",
            "BASE",
            "0.1",
            "GTC",
            "BUY",
            "limit",
            "0.1",
            Some("gtc"),
        ),
        (
            "LIMIT",
            "BASE",
            "0.1",
            "IOC",
            "BUY",
            "limit",
            "0.1",
            Some("ioc"),
        ),
        (
            "LIMIT",
            "BASE",
            "0.1",
            "FOK",
            "BUY",
            "limit",
            "0.1",
            Some("fok"),
        ),
        (
            "MARKET", "QUOTE", "100", "DAY", "BUY", "market", "100", None,
        ),
        (
            "MARKET", "BASE", "0.1", "DAY", "SELL", "market", "0.1", None,
        ),
    ];
    for (order_type, quantity_type, quantity, tif, side, expected_type, expected_size, force) in
        cases
    {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let vault = Vault::default();
        let http = DemoOrders::default();
        let workspace =
            cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let account = connected_demo(&mut cp, &vault, &http, &workspace);
        let proposal = demo_proposal(
            &mut cp,
            &workspace,
            &account,
            order_type,
            quantity_type,
            quantity,
            tif,
            side,
        );
        let reply = run_provider(
            &mut cp,
            demo_submit_request(&workspace, &account, &proposal),
            &vault,
            &http,
        );
        assert_eq!(reply["data"]["state"], "ACKNOWLEDGED", "{reply}");
        let body = http.posts.borrow()[0].clone();
        assert_eq!(body["side"], side.to_ascii_lowercase());
        assert_eq!(body["orderType"], expected_type);
        assert_eq!(body["size"], expected_size);
        assert_eq!(body.get("force").and_then(Value::as_str), force);
        if order_type == "LIMIT" {
            assert_eq!(body["price"], "65000");
        } else {
            assert!(body.get("price").is_none());
        }
    }
}

#[test]
fn unsupported_order_combinations_and_exchange_filters_block_submission() {
    {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let vault = Vault::default();
        let http = DemoOrders::default();
        let workspace =
            cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let account = connected_demo(&mut cp, &vault, &http, &workspace);
        let unsupported = demo_proposal(
            &mut cp, &workspace, &account, "MARKET", "BASE", "0.1", "GTC", "BUY",
        );
        let unsupported_error = match cp.prepare_provider_for(
            &demo_submit_request(&workspace, &account, &unsupported),
            "main",
        ) {
            Ok(_) => panic!("unsupported Market/BASE/GTC must fail closed"),
            Err(error) => error,
        };
        assert_eq!(unsupported_error.code, "ORDER_CAPABILITY_UNSUPPORTED");
        assert!(http.posts.borrow().is_empty());
    }

    for quantity in ["0.1234567", "0.000001"] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let vault = Vault::default();
        let http = DemoOrders::default();
        let workspace =
            cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let account = connected_demo(&mut cp, &vault, &http, &workspace);
        let proposal = demo_proposal(
            &mut cp, &workspace, &account, "LIMIT", "BASE", quantity, "GTC", "BUY",
        );
        let reply = run_provider(
            &mut cp,
            demo_submit_request(&workspace, &account, &proposal),
            &vault,
            &http,
        );
        assert_eq!(reply["data"]["state"], "REJECTED", "{reply}");
        assert_eq!(reply["data"]["errorCode"], "ORDER_FILTER_REJECTED");
        assert!(http.posts.borrow().is_empty());
    }
}

#[test]
fn timeout_recovers_after_workspace_restart_by_client_oid_without_a_second_post() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = Vault::default();
    let http = DemoOrders::default();
    http.post_mode.set(DemoPostMode::Timeout);
    let workspace =
        cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let account = connected_demo(&mut cp, &vault, &http, &workspace);
    let proposal = demo_proposal(
        &mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC", "BUY",
    );
    let submit = demo_submit_request(&workspace, &account, &proposal);
    let unknown = run_provider(&mut cp, submit.clone(), &vault, &http);
    assert_eq!(unknown["data"]["state"], "UNKNOWN_RECONCILING", "{unknown}");
    assert_eq!(http.posts.borrow().len(), 1);

    drop(cp);
    let mut cp = ControlPlane::new(folder.path().into());
    assert_eq!(
        cp.dispatch(request("workspace.open", json!({})))["ok"],
        true
    );
    let saved = cp.dispatch(request(
        "bitget.demo.order.attempt.get",
        json!({"workspaceId":workspace,"proposalId":proposal["proposalId"]}),
    ));
    assert_eq!(saved["data"]["attempt"]["state"], "UNKNOWN_RECONCILING");
    let accounts = cp.dispatch(request("account.list", json!({"workspaceId":workspace})));
    let current_account = accounts["data"]["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["connectionId"] == account["connectionId"])
        .unwrap()
        .clone();
    let mut retry = submit.clone();
    retry["payload"]["expectedConnectionStateVersion"] = current_account["stateVersion"].clone();
    assert!(cp.prepare_provider_for(&retry, "main").unwrap().is_none());

    let reconciled = run_provider(
        &mut cp,
        request(
            "bitget.demo.order.reconcile",
            json!({
                "workspaceId":workspace,"connectionId":account["connectionId"],
                "expectedConnectionStateVersion":current_account["stateVersion"],
                "proposalId":proposal["proposalId"]
            }),
        ),
        &vault,
        &http,
    );
    assert_eq!(reconciled["data"]["state"], "ACKNOWLEDGED", "{reconciled}");
    assert_eq!(reconciled["data"]["providerOrderId"], "123456789");
    assert_eq!(http.posts.borrow().len(), 1, "reconcile is query-only");
    assert!(
        http.calls
            .borrow()
            .iter()
            .any(|(_, method, path, paptrading)| {
                *method == ProviderHttpMethod::Get
                    && path.starts_with("/api/v2/spot/trade/orderInfo?clientOid=tx-")
                    && *paptrading
            })
    );
}

#[test]
fn ambiguous_post_responses_stay_unknown_and_do_not_retry() {
    for mode in [
        DemoPostMode::RateLimit,
        DemoPostMode::Redirect,
        DemoPostMode::BadJson,
        DemoPostMode::WrongClientOid,
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let vault = Vault::default();
        let http = DemoOrders::default();
        http.post_mode.set(mode);
        let workspace =
            cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let account = connected_demo(&mut cp, &vault, &http, &workspace);
        let proposal = demo_proposal(
            &mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC", "BUY",
        );
        let submit = demo_submit_request(&workspace, &account, &proposal);
        let reply = run_provider(&mut cp, submit.clone(), &vault, &http);
        assert_eq!(reply["data"]["state"], "UNKNOWN_RECONCILING", "{reply}");
        assert_eq!(http.posts.borrow().len(), 1);
        assert!(cp.prepare_provider_for(&submit, "main").unwrap().is_none());
        assert_eq!(http.posts.borrow().len(), 1);
    }
}

#[test]
fn explicit_http_rejection_is_terminal_and_cannot_be_resubmitted() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let vault = Vault::default();
    let http = DemoOrders::default();
    http.post_mode.set(DemoPostMode::Rejected);
    let workspace =
        cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let account = connected_demo(&mut cp, &vault, &http, &workspace);
    let proposal = demo_proposal(
        &mut cp, &workspace, &account, "LIMIT", "BASE", "0.1", "GTC", "BUY",
    );
    let submit = demo_submit_request(&workspace, &account, &proposal);
    let reply = run_provider(&mut cp, submit.clone(), &vault, &http);
    assert_eq!(reply["data"]["state"], "REJECTED", "{reply}");
    assert_eq!(reply["data"]["errorCode"], "PROVIDER_ORDER_REJECTED");
    assert!(cp.prepare_provider_for(&submit, "main").unwrap().is_none());
    assert_eq!(http.posts.borrow().len(), 1);
}

#[test]
fn demo_account_rejection_keeps_three_secrets_native_and_never_falls_back_to_live() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let schema = cp.dispatch(request(
        "provider.get_schema",
        json!({"providerId":"bitget","environment":"DEMO"}),
    ));
    assert_eq!(schema["ok"], true, "{schema}");
    let fields = schema["data"]["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 3);
    assert_eq!(
        fields
            .iter()
            .map(|f| f["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["apiKey", "secret", "passphrase"]
    );
    assert!(
        fields
            .iter()
            .all(|f| f["secret"] == true && f["required"] == true)
    );
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"DEMO","label":"Disposable Demo rejection"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let vault = Vault::default();
    let http = DemoUnsupported(RefCell::new(vec![]));
    let outcome = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, &vault) {
        cp.record_credential_cleanup(&job, cleanup);
    }
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(reply["error"]["code"], "PROVIDER_UNSUPPORTED", "{reply}");
    assert!(vault.0.borrow().is_none());
    assert_eq!(
        *http.0.borrow(),
        ["/api/v2/public/time", "/api/v2/spot/account/info"]
    );
    let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
    let account = &list["data"]["accounts"][0];
    assert_eq!(account["environment"], "DEMO");
    assert_eq!(account["connectionState"], "FAILED");
    assert_eq!(account["health"]["credential"], "MISSING");
    assert!(account["data"].is_null());
    assert!(account["lastSuccessfulSync"].is_null());
    for secret in [KEY, SECRET, PASSPHRASE] {
        assert!(!format!("{schema}{reply}{list}").contains(secret));
    }
}

fn live_lifecycle(vault: &impl CredentialVault) {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Live contract fixture"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let http = LivePages::default();
    let outcome = job.run(
        vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    let a = &reply["data"];
    assert_eq!(a["permissions"]["scope"], "VERIFIED");
    assert_eq!(a["health"]["arming"], "DISARMED");
    assert_eq!(a["health"]["executionEligibility"], "BLOCKED");
    assert_eq!(a["data"]["balances"][0]["total"], "1000000000000000002");
    assert_eq!(a["data"]["balances"][0]["locked"], "2");
    assert_eq!(a["data"]["balances"][0]["restrictedAvailable"], "7");
    let orders = a["data"]["openOrders"].as_array().unwrap();
    assert_eq!(orders.len(), 103);
    assert!(
        orders
            .iter()
            .all(|order| order["instrumentId"] == "crypto:BTC/USDT:spot")
    );
    assert_eq!(orders[100]["quantity"], "0.1234567890123456789");
    assert!(orders[100]["notional"].is_null());
    assert_eq!(orders[101]["kind"], "TPSL");
    assert!(orders[101]["quantity"].is_null());
    assert_eq!(orders[101]["notional"], "0.1234567890123456789");
    assert_eq!(orders[101]["triggerPrice"], "12000");
    assert_eq!(orders[102]["kind"], "PLAN");
    assert_eq!(orders[102]["notional"], "20");
    assert_ne!(orders[0]["brokerOrderId"], orders[102]["brokerOrderId"]);
    let book = &a["data"]["bitgetOrderBook"];
    assert_eq!(book["orders"].as_array().unwrap().len(), 107);
    assert_eq!(book["fills"].as_array().unwrap().len(), 1);
    let historical = book["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254740997")
        .unwrap();
    assert_eq!(historical["providerStatus"], "filled");
    assert_eq!(historical["normalizedStatus"], "FILLED");
    assert_eq!(historical["origin"], "external");
    assert_eq!(historical["filledQuantity"], "0.0002");
    assert_eq!(historical["filledValue"], "14.000025");
    let trigger = book["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254741002")
        .unwrap();
    assert_eq!(trigger["kind"], "PLAN");
    assert_eq!(trigger["normalizedStatus"], "TRIGGERED");
    assert!(trigger["filledQuantity"].is_null());
    let trigger_failed = book["orders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|order| order["providerOrderId"] == "9007199254741000")
        .unwrap();
    assert_eq!(trigger_failed["normalizedStatus"], "TRIGGER_FAILED");
    assert!(trigger_failed["filledValue"].is_null());
    assert_eq!(book["fills"][0]["providerTradeId"], "9223372036854775808");
    assert_eq!(book["fills"][0]["quantity"], "0.0002");
    assert_eq!(book["fills"][0]["value"], "14.000025");
    assert_eq!(book["fills"][0]["currency"], "USDT");
    let paths = http.0.borrow();
    assert!(
        paths
            .iter()
            .any(|path| path == "/api/v2/spot/trade/history-orders?limit=100")
    );
    assert!(
        paths
            .iter()
            .any(|path| path == "/api/v2/spot/trade/history-orders?limit=100&tpslType=tpsl")
    );
    assert!(
        paths
            .iter()
            .any(|path| path == "/api/v2/spot/trade/history-plan-order?limit=100")
    );
    assert!(paths.iter().any(|path| path
        == "/api/v2/spot/trade/history-plan-order?limit=100&idLessThan=9007199254741001"));
    assert!(
        paths
            .iter()
            .any(|path| path == "/api/v2/spot/trade/fills?limit=100")
    );
    let confirm=cp.dispatch(request("provider.connect",json!({"step":"confirm","workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"],"acknowledgeUnverified":false})));
    assert_eq!(confirm["ok"], true, "{confirm}");
    drop(cp);
    let mut cp = ControlPlane::new(folder.path().into());
    assert_eq!(
        cp.dispatch(request("workspace.open", json!({})))["ok"],
        true
    );
    let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
    let a = &list["data"]["accounts"][0];
    assert_eq!(a["data"]["openOrders"].as_array().unwrap().len(), 103);
    assert_eq!(
        a["data"]["bitgetOrderBook"]["orders"]
            .as_array()
            .unwrap()
            .len(),
        107
    );
    let req = request(
        "provider.disconnect",
        json!({"workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let outcome = job.run(
        vault,
        |_| panic!("Disconnect must not capture credentials"),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, outcome);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["health"]["credential"], "MISSING");
}

#[test]
fn live_connection_preserves_all_order_pages_asset_buckets_and_review() {
    let vault = Vault::default();
    live_lifecycle(&vault);
    assert!(vault.0.borrow().is_none());
}
#[test]
#[cfg(target_os = "macos")]
#[ignore = "Explicit native Keychain integration with three disposable Bitget credential fields"]
fn native_keychain_stores_and_removes_three_bitget_fields() {
    live_lifecycle(&tradex::provider_io::NativeVault);
}

struct ChangedResponse {
    path: &'static str,
    data: Value,
}
impl ProviderHttp for ChangedResponse {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        let bytes = LivePages::default().get(endpoint, path, headers)?;
        if path == self.path {
            Ok(serde_json::to_vec(&json!({"code":"00000","data":self.data})).unwrap())
        } else {
            Ok(bytes)
        }
    }
}
#[test]
fn dangerous_or_unknown_scope_and_failed_refresh_do_not_gain_authority() {
    for authority in [
        "wtow",
        "wwow",
        "chow",
        "coow",
        "cpow",
        "smow",
        "ttow",
        "p2p",
        "pllw",
        "taxw",
        "future_permission",
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let req = request(
            "provider.connect",
            json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Scope fixture"}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let vault = Vault::default();
        let http = ChangedResponse {
            path: "/api/v2/spot/account/info",
            data: json!({"userId":"9007199254740993","ips":"127.0.0.1","authorities":["stor",authority]}),
        };
        let outcome = job.run(
            &vault,
            |_| credentials(),
            &http,
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, outcome);
        assert_eq!(reply["ok"], true, "{reply}");
        let a = &reply["data"];
        let req = request(
            "provider.connect",
            json!({"step":"confirm","workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"],"acknowledgeUnverified":true}),
        );
        let confirmed = cp.dispatch(req);
        assert_eq!(
            confirmed["ok"],
            authority == "future_permission",
            "{authority}: {confirmed}"
        );
        if authority == "future_permission" {
            assert_eq!(confirmed["data"]["permissions"]["scope"], "UNVERIFIED");
        }
        let a = if confirmed["ok"] == true {
            &confirmed["data"]
        } else {
            a
        };
        let previous = a["data"].clone();
        let stamp = a["lastSuccessfulSync"].clone();
        let req = request(
            "account.refresh",
            json!({"workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let http = if authority == "future_permission" {
            ChangedResponse {
                path: "/api/v2/spot/account/info",
                data: json!({"userId":"2","ips":"127.0.0.1","authorities":["stor"]}),
            }
        } else {
            ChangedResponse {
                path: "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal&idLessThan=101",
                data: json!({"orders":[]}),
            }
        };
        let outcome = job.run(
            &vault,
            |_| panic!("Refresh must use existing credentials"),
            &http,
            || cp.provider_job_current(&job),
        );
        let failed = cp.complete_provider(&job, outcome);
        assert_eq!(failed["ok"], false);
        if authority == "future_permission" {
            assert_eq!(failed["error"]["code"], "PROVIDER_IDENTITY_CHANGED");
        }
        let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
        let a = &list["data"]["accounts"][0];
        assert_eq!(a["data"], previous);
        assert_eq!(a["lastSuccessfulSync"], stamp);
        assert_eq!(a["health"]["executionEligibility"], "BLOCKED");
    }
}

#[test]
fn overlapping_live_current_and_history_rows_keep_the_previous_account_snapshot() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let connect = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Live history fixture"}),
    );
    let job = cp.prepare_provider(&connect).unwrap().unwrap();
    let vault = Vault::default();
    let result = job.run(
        &vault,
        |_| credentials(),
        &LivePages::default(),
        || cp.provider_job_current(&job),
    );
    let connected = cp.complete_provider(&job, result);
    assert_eq!(connected["ok"], true, "{connected}");
    let account = &connected["data"];
    let confirmed = cp.dispatch(request(
        "provider.connect",
        json!({"step":"confirm","workspaceId":ws,"connectionId":account["connectionId"],"expectedStateVersion":account["stateVersion"],"acknowledgeUnverified":false}),
    ));
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    let account = &confirmed["data"];
    let previous = account["data"].clone();
    let stamp = account["lastSuccessfulSync"].clone();
    let refresh = request(
        "account.refresh",
        json!({"workspaceId":ws,"connectionId":account["connectionId"],"expectedStateVersion":account["stateVersion"]}),
    );
    let job = cp.prepare_provider(&refresh).unwrap().unwrap();
    let duplicate = json!({
        "userId":"9007199254740993","orderId":"200","symbol":"BTCUSDT",
        "price":"1","size":"1","orderType":"limit","side":"buy","status":"filled",
        "priceAvg":"1","baseVolume":"1","quoteVolume":"1","quoteCoin":"USDT","tpslType":"normal"
    });
    let result = job.run(
        &vault,
        |_| panic!("Refresh must use stored credentials"),
        &ChangedResponse {
            path: "/api/v2/spot/trade/history-orders?limit=100",
            data: json!([duplicate]),
        },
        || cp.provider_job_current(&job),
    );
    let failed = cp.complete_provider(&job, result);
    assert_eq!(failed["ok"], false, "{failed}");
    assert_eq!(failed["error"]["code"], "PROVIDER_DATA_INCOMPLETE");
    let accounts = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
    let account = &accounts["data"]["accounts"][0];
    assert_eq!(account["data"], previous);
    assert_eq!(account["lastSuccessfulSync"], stamp);
}

#[test]
fn provider_schema_field_count_is_checked_before_storage_and_http() {
    for (provider, environment, values) in [
        (
            "alpaca",
            "PAPER",
            vec![KEY.into(), SECRET.into(), PASSPHRASE.into()],
        ),
        ("bitget", "LIVE", vec![KEY.into(), SECRET.into()]),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let req = request(
            "provider.connect",
            json!({"step":"test","workspaceId":ws,"providerId":provider,"environment":environment,"label":"Invalid field count"}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let vault = Vault::default();
        let http = LivePages::default();
        let outcome = job.run(
            &vault,
            |_| Credentials::new(values),
            &http,
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, outcome);
        assert_eq!(reply["ok"], false, "{reply}");
        assert_eq!(reply["error"]["code"], "IPC_PAYLOAD_INVALID");
        assert!(vault.0.borrow().is_none());
        assert!(http.0.borrow().is_empty());
    }
}

#[test]
fn changing_restricted_ip_addresses_requires_a_new_permission_review() {
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let vault = Vault::default();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"IP review fixture"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let http = LivePages::default();
    let result = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, result);
    assert_eq!(reply["ok"], true, "{reply}");
    let a = &reply["data"];
    let confirmed = cp.dispatch(request("provider.connect", json!({"step":"confirm","workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"],"acknowledgeUnverified":true})));
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    let a = &confirmed["data"];
    let req = request(
        "account.refresh",
        json!({"workspaceId":ws,"connectionId":a["connectionId"],"expectedStateVersion":a["stateVersion"]}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let changed = ChangedResponse {
        path: "/api/v2/spot/account/info",
        data: json!({"userId":"9007199254740993","ips":"127.0.0.2","authorities":["stor","stow"]}),
    };
    let result = job.run(
        &vault,
        |_| panic!("No capture on refresh"),
        &changed,
        || cp.provider_job_current(&job),
    );
    let reply = cp.complete_provider(&job, result);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["connectionState"], "REVIEW_REQUIRED");
    assert_eq!(reply["data"]["permissions"]["acknowledged"], false);
}

#[test]
fn malformed_identity_time_assets_pages_and_reflected_secrets_fail_without_a_snapshot() {
    let info = "/api/v2/spot/account/info";
    let assets = "/api/v2/spot/account/assets?assetType=all";
    let plan = "/api/v2/spot/trade/current-plan-order?limit=100";
    let normal = "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal";
    let history = "/api/v2/spot/trade/history-orders?limit=100";
    let plan_history = "/api/v2/spot/trade/history-plan-order?limit=100";
    let fills = "/api/v2/spot/trade/fills?limit=100";
    let coin = json!({"coin":"BTC","available":"1","frozen":"0","locked":"0","limitAvailable":"0"});
    let mut negative = coin.clone();
    negative["locked"] = "-1".into();
    let bad_order = json!({"userId":"9007199254740993","orderId":"1","symbol":"BTCUSDT","size":"1","orderType":"limit","side":"buy","status":"live","tpslType":"normal","priceAvg":"1","baseVolume":"0","quoteVolume":"0"});
    let history_order = json!({"userId":"9007199254740993","orderId":"2","symbol":"BTCUSDT","size":"1","orderType":"limit","side":"buy","status":"filled","tpslType":"normal","price":"1","priceAvg":"1","baseVolume":"1","quoteVolume":"1","quoteCoin":"USDT"});
    let mut wrong_history_user = history_order.clone();
    wrong_history_user["userId"] = "3".into();
    let fill = json!({"userId":"9007199254740993","orderId":"2","tradeId":"4","symbol":"BTCUSDT","side":"buy","size":"1","amount":"1"});
    let mut wrong_fill_user = fill.clone();
    wrong_fill_user["userId"] = "3".into();
    let mut wrong_user = bad_order.clone();
    wrong_user["userId"] = "2".into();
    let cases = vec![
        (
            "/api/v2/public/time",
            json!({"serverTime":"-1"}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v2/public/time",
            json!({"serverTime":1788849600000u64}),
            "CLOCK_SKEW",
        ),
        (
            "/api/v2/public/time",
            json!({"serverTime":"1788849600000000"}),
            "CLOCK_SKEW",
        ),
        (
            info,
            json!({"userId":"../1","authorities":[],"ips":""}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            info,
            json!({"userId":"1","authorities":true,"ips":""}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            info,
            json!({"userId":"1","authorities":["stor","stor"],"ips":""}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            info,
            json!({"userId":"1","authorities":["stor"],"ips":"","unexpected":PASSPHRASE}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            assets,
            json!([coin.clone(), coin.clone()]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (assets, json!([negative]), "PROVIDER_RESPONSE_INVALID"),
        (
            assets,
            json!({"list":[coin.clone()]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            assets,
            Value::Array(vec![coin; 10_001]),
            "PROVIDER_DATA_INCOMPLETE",
        ),
        (normal, json!([wrong_user]), "PROVIDER_IDENTITY_CHANGED"),
        (
            normal,
            json!([bad_order.clone(), bad_order]),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            plan,
            json!({"nextFlag":true,"idLessThan":"1","orderList":[]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (
            plan,
            json!({"nextFlag":"false","idLessThan":"1","orderList":[]}),
            "PROVIDER_RESPONSE_INVALID",
        ),
        (plan, json!([]), "PROVIDER_RESPONSE_INVALID"),
        (
            history,
            json!([wrong_history_user]),
            "PROVIDER_IDENTITY_CHANGED",
        ),
        (
            history,
            json!([history_order.clone(), history_order.clone()]),
            "PROVIDER_DATA_INCOMPLETE",
        ),
        (
            history,
            Value::Array(
                (1..=101)
                    .map(|order_id| {
                        let mut row = history_order.clone();
                        row["orderId"] = order_id.to_string().into();
                        row
                    })
                    .collect(),
            ),
            "PROVIDER_DATA_INCOMPLETE",
        ),
        (
            plan_history,
            json!({"nextFlag":true,"idLessThan":"3","orderList":[{
                "orderId":"2","symbol":"BTCUSDT","size":"1","executePrice":"1",
                "triggerPrice":"1","status":"executed","orderType":"limit",
                "side":"buy","planType":"amount"
            }]}),
            "PROVIDER_DATA_INCOMPLETE",
        ),
        (fills, json!([wrong_fill_user]), "PROVIDER_IDENTITY_CHANGED"),
    ];
    for (path, data, expected) in cases {
        let folder = tempfile::tempdir().unwrap();
        let mut cp = ControlPlane::new(folder.path().into());
        let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
        let req = request(
            "provider.connect",
            json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Rejected observation"}),
        );
        let job = cp.prepare_provider(&req).unwrap().unwrap();
        let vault = Vault::default();
        let result = job.run(
            &vault,
            |_| credentials(),
            &ChangedResponse { path, data },
            || cp.provider_job_current(&job),
        );
        let reply = cp.complete_provider(&job, result);
        if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, &vault) {
            cp.record_credential_cleanup(&job, cleanup);
        }
        assert_eq!(reply["ok"], false, "{path}: {reply}");
        assert_eq!(reply["error"]["code"], expected, "{path}: {reply}");
        assert!(vault.0.borrow().is_none());
        let list = cp.dispatch(request("account.list", json!({"workspaceId":ws})));
        let a = &list["data"]["accounts"][0];
        assert!(a["data"].is_null());
        assert!(a["lastSuccessfulSync"].is_null());
        assert_eq!(a["health"]["credential"], "MISSING");
        for secret in [KEY, SECRET, PASSPHRASE] {
            assert!(!format!("{reply}{list}").contains(secret));
        }
    }
}

#[test]
fn invalidation_during_a_page_stops_further_io_and_cleans_the_new_key() {
    struct Invalidated {
        current: std::cell::Cell<bool>,
        calls: RefCell<Vec<String>>,
    }
    impl ProviderHttp for Invalidated {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> Result<Vec<u8>> {
            assert!(self.current.get(), "No read may follow invalidation");
            self.calls.borrow_mut().push(path.into());
            let result = LivePages::default().get(endpoint, path, headers);
            if path == "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal" {
                self.current.set(false);
            }
            result
        }
    }
    let folder = tempfile::tempdir().unwrap();
    let mut cp = ControlPlane::new(folder.path().into());
    let ws = cp.dispatch(request("workspace.open", json!({})))["data"]["workspaceId"].clone();
    let req = request(
        "provider.connect",
        json!({"step":"test","workspaceId":ws,"providerId":"bitget","environment":"LIVE","label":"Invalidated page"}),
    );
    let job = cp.prepare_provider(&req).unwrap().unwrap();
    let vault = Vault::default();
    let http = Invalidated {
        current: std::cell::Cell::new(true),
        calls: RefCell::new(vec![]),
    };
    let result = job.run(
        &vault,
        |_| credentials(),
        &http,
        || cp.provider_job_current(&job) && http.current.get(),
    );
    let reply = cp.complete_provider(&job, result);
    assert_eq!(reply["error"]["code"], "STATE_VERSION_CONFLICT", "{reply}");
    assert_eq!(http.calls.borrow().len(), 3);
    assert!(vault.0.borrow().is_none());
}
