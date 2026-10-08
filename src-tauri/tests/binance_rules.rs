use serde_json::{Value, json};
use tradex::{
    ControlPlane,
    provider_io::{CredentialVault, ProviderHttp},
};
#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod fixtures;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch_with_events(
        json!({"requestId":"spot-rules","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}

#[test]
fn public_rule_source_selection_is_versioned_metadata_and_keeps_the_borrowed_key() {
    let folder = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(folder.path().into());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = fixtures::Vault::default();
    let http = fixtures::Http::default();
    let request = json!({"requestId":"rules-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"binance","environment":"LIVE","label":"Spot rules source"}});
    let job = control
        .prepare_provider_for(&request, "main")
        .unwrap()
        .unwrap();
    let outcome = job.run(
        &vault,
        |_| fixtures::credentials(),
        &http,
        || control.provider_job_current(&job),
    );
    let tested = control.complete_provider(&job, outcome);
    assert_eq!(tested["ok"], true, "{tested}");
    let confirmed = command(
        &mut control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":false}),
    );
    assert_eq!(confirmed["ok"], true, "{confirmed}");
    assert_eq!(confirmed["data"]["health"]["arming"], "DISARMED");
    let account: tradex::providers::AccountConnection =
        serde_json::from_value(confirmed["data"].clone()).unwrap();
    let before = http.calls.borrow().len();
    let source = command(
        &mut control,
        "data.binance_rules.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        source["ok"], true,
        "Public Binance rule-source selection is unavailable: {source}"
    );
    assert_eq!(source["data"]["configured"], false);
    assert_eq!(
        source["data"]["eligibleAccounts"][0]["connectionId"],
        account.connection_id
    );
    let payload = json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account.connection_id,"instrumentId":"crypto:BTC/USDT:spot"});
    let saved = command(
        &mut control,
        "data.binance_rules.configure",
        payload.clone(),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(saved["data"]["status"], "UNVERIFIED");
    assert_eq!(saved["data"]["kind"], "BINANCE_SPOT_RULES");
    assert!(saved["data"]["evidence"].is_null());
    assert_eq!(
        http.calls.borrow().len(),
        before,
        "Save must not read the provider"
    );
    let stale = command(&mut control, "data.binance_rules.configure", payload);
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    drop(control);
    let mut control = ControlPlane::new(folder.path().into());
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let reopened = command(
        &mut control,
        "data.binance_rules.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(reopened["data"]["connectionId"], account.connection_id);
    assert_eq!(reopened["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(reopened["data"]["status"], "UNVERIFIED");
    assert!(reopened["data"]["evidence"].is_null());
    let disconnected = command(
        &mut control,
        "data.binance_rules.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":reopened["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["data"]["configured"], false);
    assert!(vault.get(&account.credential_ref()).is_ok());
    let retained = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account.connection_id}),
    );
    assert_eq!(retained["data"]["connectionState"], "CONNECTED");
    assert_eq!(retained["data"]["health"]["arming"], "DISARMED");
}

#[derive(Default)]
struct RuleHttp {
    calls: std::cell::RefCell<Vec<String>>,
    uid: std::cell::Cell<u64>,
    reflect_signature: std::cell::Cell<bool>,
    reflect_key: std::cell::Cell<bool>,
    edit: std::cell::Cell<Option<fn(&str, &mut Value)>>,
    raw: std::cell::RefCell<Option<(String, Vec<u8>)>>,
    hook: std::cell::RefCell<Option<(String, Box<dyn Fn()>)>>,
}
impl tradex::provider_io::ProviderHttp for RuleHttp {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        assert_eq!(endpoint, tradex::provider_io::ProviderEndpoint::BinanceLive);
        let (route, query) = path.split_once('?').unwrap_or((path, ""));
        self.calls.borrow_mut().push(path.into());
        let public = matches!(
            route,
            "/api/v3/time"
                | "/sapi/v1/system/status"
                | "/api/v3/exchangeInfo"
                | "/api/v3/executionRules"
        );
        if public {
            assert!(headers.is_empty());
        } else {
            assert_eq!(headers["X-MBX-APIKEY"], fixtures::KEY);
            assert!(headers["X-MBX-APIKEY"].is_sensitive());
            let (params, signature) = query.split_once("&signature=").unwrap();
            use hmac::Mac;
            let mut mac =
                hmac::Hmac::<sha2::Sha256>::new_from_slice(fixtures::SECRET.as_bytes()).unwrap();
            mac.update(params.as_bytes());
            mac.verify_slice(&hex::decode(signature).unwrap()).unwrap();
            assert!(params.contains("recvWindow=5000"));
        }
        let mut body = match route {
            "/api/v3/time" => {
                json!({"serverTime":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})
            }
            "/api/v3/account" => {
                json!({"uid":self.uid.get(),"accountType":"SPOT","canTrade":true,"permissions":["SPOT"]})
            }
            "/sapi/v1/account/apiRestrictions" => {
                json!({"enableReading":true,"enableWithdrawals":false,"enableInternalTransfer":false,"permitsUniversalTransfer":false,"enableMargin":false,"enableFutures":false,"enableVanillaOptions":false,"enablePortfolioMarginTrading":false,"enableFixApiTrade":false,"enableFixReadOnly":false,"enableSpotAndMarginTrading":true,"ipRestrict":true})
            }
            "/sapi/v1/system/status" => json!({"status":0}),
            "/sapi/v1/account/apiTradingStatus" => {
                json!({"data":{"isLocked":false,"updateTime":1547630471725u64,"plannedRecoverTime":0}})
            }
            "/api/v3/exchangeInfo" => {
                assert_eq!(query, "symbol=BTCUSDT&showPermissionSets=true");
                json!({"exchangeFilters":[],"symbols":[{"symbol":"BTCUSDT","status":"TRADING","baseAsset":"BTC","quoteAsset":"USDT","baseAssetPrecision":8,"quoteAssetPrecision":8,"isSpotTradingAllowed":true,"quoteOrderQtyMarketAllowed":true,"orderTypes":["LIMIT","MARKET"],"defaultSelfTradePreventionMode":"NONE","allowedSelfTradePreventionModes":["NONE"],"permissionSets":[["SPOT","MARGIN"]],"filters":[{"filterType":"PRICE_FILTER","minPrice":"0.00000000","maxPrice":"999999.00000000","tickSize":"0.01000000"},{"filterType":"LOT_SIZE","minQty":"0.00000100","maxQty":"100.00000000","stepSize":"0.00000100"}]}]})
            }
            "/api/v3/executionRules" => {
                assert_eq!(query, "symbol=BTCUSDT");
                json!({"symbolRules":[{"symbol":"BTCUSDT","rules":[{"ruleType":"PRICE_RANGE","bidLimitMultUp":"1.0001","bidLimitMultDown":"0.9999","askLimitMultUp":"1.0001","askLimitMultDown":"0.9999"}]}]})
            }
            "/api/v3/myFilters" => {
                assert!(query.starts_with("symbol=BTCUSDT&timestamp="));
                json!({"exchangeFilters":[{"filterType":"EXCHANGE_MAX_NUM_ORDERS","maxNumOrders":1000}],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":"5.00000000","applyToMarket":true,"avgPriceMins":5}],"assetFilters":[{"filterType":"MAX_ASSET","asset":"BTC","limit":"0.25000000"}]})
            }
            _ => return Err(tradex::protocol::TradeXError::new("PROVIDER_UNSUPPORTED")),
        };
        if !public && self.reflect_signature.get() {
            body["message"] = json!(query.split_once("&signature=").unwrap().1);
        }
        if self.reflect_key.get() {
            body["message"] = json!(fixtures::KEY);
        }
        let run_hook = self
            .hook
            .borrow()
            .as_ref()
            .is_some_and(|(target, _)| target == route);
        if run_hook {
            if let Some((_, hook)) = self.hook.borrow_mut().take() {
                hook();
            }
        }
        if let Some(edit) = self.edit.get() {
            edit(route, &mut body);
        }
        if let Some((target, bytes)) = self.raw.borrow().as_ref() {
            if target == route {
                return Ok(bytes.clone());
            }
        }
        Ok(serde_json::to_vec(&body).unwrap())
    }
}

#[test]
fn public_rule_refresh_collects_exact_account_and_scoped_constraints_without_execution_authority() {
    let rig = RuleRig::new();
    let control = &rig.control;
    let workspace = &rig.workspace;
    let account = &rig.account;
    let external = RuleHttp::default();
    let read = rig.refresh(&external);
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let evidence = &read["data"]["evidence"];
    assert_eq!(evidence["kind"], "BINANCE_SPOT_RULES");
    assert_eq!(evidence["providerQuality"], "READ_ONLY_SPOT_RULES");
    assert_eq!(
        evidence["remoteAccountId"],
        account["data"]["remoteAccountId"]
    );
    assert_eq!(evidence["instrumentId"], "crypto:BTC/USDT:spot");
    assert_eq!(evidence["providerSymbol"], "BTCUSDT");
    assert_eq!(evidence["baseAsset"], "BTC");
    assert_eq!(evidence["quoteAsset"], "USDT");
    assert_eq!(evidence["permissionSetsSatisfied"], true);
    assert!(
        evidence["constraints"]
            .as_array()
            .is_some_and(|rows| rows.len() >= 6)
    );
    let qualifications = read["data"]["capabilityStatuses"].as_array().unwrap();
    assert_eq!(
        qualifications
            .iter()
            .find(|c| c["capability"] == "SPOT_EXECUTION_QUALIFICATION")
            .unwrap()["status"],
        "UNVERIFIED"
    );
    assert_eq!(
        qualifications
            .iter()
            .find(|c| c["capability"] == "SPOT_DATA_USE_RIGHTS")
            .unwrap()["status"],
        "UNVERIFIED"
    );
    let mut control = control.lock().unwrap();
    let retained = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(retained["data"]["health"]["arming"], "DISARMED");
    let market = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"crypto:BTC/USDT:spot","tier":"WARM"}),
    );
    assert_eq!(market["ok"], true, "{market}");
    assert!(
        market["data"]["snapshot"].is_null(),
        "Rule reads cannot manufacture a quote: {market}"
    );
    assert_eq!(
        market["data"]["spotRuleEvidence"]["evidence"]["providerSymbol"], "BTCUSDT",
        "Markets must expose the real exact-symbol rule evidence: {market}"
    );
    let other = command(
        &mut control,
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"crypto:ETH/USDT:spot","tier":"WARM"}),
    );
    assert!(
        other["data"]["spotRuleEvidence"].is_null(),
        "BTC rules must not qualify ETH: {other}"
    );
    for path in [
        "/api/v3/account?",
        "/api/v3/exchangeInfo?symbol=BTCUSDT&showPermissionSets=true",
        "/api/v3/executionRules?symbol=BTCUSDT",
        "/api/v3/myFilters?symbol=BTCUSDT&",
    ] {
        assert!(
            external
                .calls
                .borrow()
                .iter()
                .any(|call| call.starts_with(path)),
            "Missing {path}"
        );
    }
}

struct RuleRig {
    _folder: tempfile::TempDir,
    control: std::sync::Arc<std::sync::Mutex<ControlPlane>>,
    workspace: Value,
    account: Value,
    vault: fixtures::Vault,
}
impl RuleRig {
    fn new() -> Self {
        let folder = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(folder.path().into());
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = fixtures::Vault::default();
        let http = fixtures::Http::default();
        static NEXT_UID: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(9007199254740993);
        http.binance_uid
            .set(NEXT_UID.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        let request = json!({"requestId":"collect-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"binance","environment":"LIVE","label":"Collect rules"}});
        let job = control
            .prepare_provider_for(&request, "main")
            .unwrap()
            .unwrap();
        let observed = job.run(
            &vault,
            |_| fixtures::credentials(),
            &http,
            || control.provider_job_current(&job),
        );
        let tested = control.complete_provider(&job, observed);
        assert_eq!(tested["ok"], true, "{tested}");
        let account = command(&mut control,"provider.connect",json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":false}))["data"].clone();
        let source = command(
            &mut control,
            "data.binance_rules.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.binance_rules.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"],"instrumentId":"crypto:BTC/USDT:spot"}),
        );
        assert_eq!(saved["ok"], true, "{saved}");
        assert_eq!(
            command(
                &mut control,
                "time.revalidate",
                json!({"workspaceId":workspace})
            )["data"]["confidence"],
            "TRUSTED"
        );
        let control = std::sync::Arc::new(std::sync::Mutex::new(control));
        Self {
            _folder: folder,
            control,
            workspace,
            account,
            vault,
        }
    }
    fn decision(&self) -> Value {
        let mut control = self.control.lock().unwrap();
        let draft = command(
            &mut control,
            "trade.save_draft",
            json!({"workspaceId":self.workspace,"fields":{"accountId":self.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}}),
        );
        assert_eq!(draft["ok"], true, "{draft}");
        let proposal = command(
            &mut control,
            "trade.generate_proposal",
            json!({"workspaceId":self.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
        );
        assert_eq!(proposal["ok"], true, "{proposal}");
        let decision = command(
            &mut control,
            "risk.evaluate_proposal",
            json!({"workspaceId":self.workspace,"proposalId":proposal["data"]["proposalId"]}),
        );
        assert_eq!(decision["ok"], true, "{decision}");
        decision["data"].clone()
    }
    fn refresh(&self, external: &RuleHttp) -> Value {
        external.uid.set(
            self.account["data"]["remoteAccountId"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
        );
        let saved = command(
            &mut self.control.lock().unwrap(),
            "data.binance_rules.connection",
            json!({"workspaceId":self.workspace}),
        );
        tradex::financial_sources::execute_refresh(
            &self.control,
            &json!({"requestId":"read-rules","schemaVersion":1,"command":"data.binance_rules.refresh","payload":{"workspaceId":self.workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
            "main",
            &self.vault,
            external,
        )
    }
}

#[test]
fn malformed_rule_range_retires_current_evidence_and_retains_the_last_good_observation() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"][1]["minQty"] = json!("200");
        }
    }));
    let failed = rig.refresh(&external);
    assert_eq!(
        failed["data"]["status"], "UNAVAILABLE",
        "Inverted quantity limits must fail closed: {failed}"
    );
    assert_eq!(failed["data"]["evidence"], good["data"]["evidence"]);
    assert_eq!(failed["data"]["observedAt"], good["data"]["observedAt"]);
    external.edit.set(None);
    let recovered = rig.refresh(&external);
    assert_eq!(recovered["data"]["status"], "AVAILABLE", "{recovered}");
    let account = command(
        &mut rig.control.lock().unwrap(),
        "account.get",
        json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]}),
    );
    assert_eq!(account["data"]["health"]["arming"], "DISARMED");
}

fn rule_check(decision: &Value) -> &Value {
    decision["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["checkId"] == "INSTRUMENT_RULES")
        .unwrap()
}
#[test]
fn actual_halt_evidence_rejects_new_order_rules_and_normal_recovery_never_grants_execution() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["status"] = json!("HALT");
        }
    }));
    let read = rig.refresh(&external);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    assert_eq!(
        read["data"]["evidence"]["admissionBlockers"],
        json!(["SYMBOL_NOT_TRADING"])
    );
    let halted = rig.decision();
    assert_eq!(
        rule_check(&halted)["outcome"],
        "REJECT",
        "Actual HALT must reach Trade's protected decision: {halted}"
    );
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => {
            body["symbols"][0]["status"] = json!("BREAK");
            body["symbols"][0]["isSpotTradingAllowed"] = json!(false);
        }
        "/api/v3/account" => {
            body["accountType"] = json!("MARGIN");
            body["canTrade"] = json!(false);
        }
        "/sapi/v1/account/apiRestrictions" => body["enableWithdrawals"] = json!(true),
        "/sapi/v1/system/status" => body["status"] = json!(1),
        "/sapi/v1/account/apiTradingStatus" => body["data"]["isLocked"] = json!(true),
        _ => (),
    }));
    let blocked = rig.refresh(&external);
    assert_eq!(blocked["data"]["status"], "AVAILABLE", "{blocked}");
    assert_eq!(blocked["data"]["evidence"]["symbolStatus"], "BREAK");
    let blockers = blocked["data"]["evidence"]["admissionBlockers"]
        .as_array()
        .unwrap();
    for expected in [
        "ACCOUNT_NOT_SPOT",
        "ACCOUNT_CANNOT_TRADE",
        "SPOT_NOT_ALLOWED",
        "SYMBOL_NOT_TRADING",
        "KEY_PERMISSION_BLOCKED",
        "SYSTEM_MAINTENANCE",
        "API_TRADING_LOCKED",
    ] {
        assert!(
            blockers.contains(&json!(expected)),
            "Missing distinct admission diagnosis {expected}: {blocked}"
        );
    }
    assert_eq!(rule_check(&rig.decision())["outcome"], "REJECT");
    external.edit.set(None);
    let normal = rig.refresh(&external);
    assert_eq!(normal["data"]["status"], "AVAILABLE", "{normal}");
    let recovered = rig.decision();
    assert_eq!(
        rule_check(&recovered)["outcome"],
        "UNAVAILABLE",
        "Normal metadata is not complete per-Proposal qualification: {recovered}"
    );
    assert_ne!(recovered["status"], "ALLOWED");
}

#[test]
fn original_wire_types_and_duplicate_fields_cannot_replace_a_good_observation() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    for (route, bytes) in [
        ("/api/v3/account", br#"{"uid":7,"accountType":"SPOT","canTrade":true,"permissions":["SPOT"]}"#.to_vec()),
        ("/api/v3/account", br#"{"uid":9007199254740993,"accountType":"SPOT","canTrade":1,"permissions":["SPOT"]}"#.to_vec()),
        ("/api/v3/account", br#"{"uid":{"$serde_json::private::Number":"9007199254740993"},"accountType":"SPOT","canTrade":true,"permissions":["SPOT"]}"#.to_vec()),
        ("/sapi/v1/account/apiRestrictions", br#"{"enableReading":true,"enableSpotAndMarginTrading":{"$serde_json::private::Number":"1"}}"#.to_vec()),
        ("/sapi/v1/system/status", br#"{"status":0,"status":1}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":5,"applyToMarket":true,"avgPriceMins":5}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":"5","applyToMarket":true,"avgPriceMins":5.0}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"LOT_SIZE","minQty":"-0.01","maxQty":"10","stepSize":"0.01"}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"LOT_SIZE","minQty":"0.01","maxQty":"10","stepSize":"0.01"},{"filterType":"LOT_SIZE","minQty":"0.01","maxQty":"10","stepSize":"0.01"}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[],"symbolFilters":[{"filterType":"MIN_NOTIONAL","minNotional":"5","applyToMarket":true,"avgPriceMins":{"$serde_json::private::Number":"5"}}],"assetFilters":[]}"#.to_vec()),
        ("/api/v3/myFilters", br#"{"exchangeFilters":[{"filterType":"LOT_SIZE","minQty":"0.01","maxQty":"10","stepSize":"0.01"}],"symbolFilters":[],"assetFilters":[]}"#.to_vec()),
    ] {
        *external.raw.borrow_mut() = Some((route.into(), bytes));
        let failed = rig.refresh(&external);
        assert_eq!(failed["data"]["status"], "UNAVAILABLE", "Invalid original types/scopes at {route}: {failed}");
        assert_eq!(failed["data"]["evidence"], good["data"]["evidence"], "Prior evidence must remain explicitly retained, not refreshed");
        assert_ne!(rule_check(&rig.decision())["outcome"], "PASS");
    }
}

#[test]
fn permission_sets_apply_or_within_and_between_sets_without_granting_per_order_qualification() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["permissionSets"] = json!([["SPOT", "MARGIN"], ["MARGIN"]]);
        }
    }));
    let unmet = rig.refresh(&external);
    assert_eq!(unmet["data"]["status"], "AVAILABLE", "{unmet}");
    assert_eq!(unmet["data"]["evidence"]["permissionSetsSatisfied"], false);
    assert_eq!(rule_check(&rig.decision())["outcome"], "REJECT");
    external.edit.set(None);
    let accepted = rig.refresh(&external);
    assert_eq!(
        accepted["data"]["evidence"]["permissionSetsSatisfied"], true,
        "OR permits SPOT without MARGIN: {accepted}"
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/executionRules" {
            body["symbolRules"][0]["rules"] =
                json!([{"ruleType":"FUTURE_ACTIVE_RULE","limit":"1"}]);
        }
    }));
    let unknown = rig.refresh(&external);
    assert_eq!(unknown["data"]["status"], "AVAILABLE", "{unknown}");
    assert!(
        unknown["data"]["evidence"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA"))
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn changing_the_selected_instrument_during_http_prevents_late_publication() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let cp = rig.control.clone();
    let workspace = rig.workspace.clone();
    let account = rig.account.clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/account".into(),
        Box::new(move || {
            let mut control = cp.lock().unwrap();
            let source = command(
                &mut control,
                "data.binance_rules.connection",
                json!({"workspaceId":workspace}),
            );
            let changed = command(
                &mut control,
                "data.binance_rules.configure",
                json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"],"instrumentId":"crypto:ETH/USDT:spot"}),
            );
            assert_eq!(changed["ok"], true, "{changed}");
        }),
    ));
    let late = rig.refresh(&external);
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT", "{late}");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(current["data"]["instrumentId"], "crypto:ETH/USDT:spot");
    assert_eq!(current["data"]["status"], "UNVERIFIED");
    assert!(current["data"]["evidence"].is_null());
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|p| p.starts_with("/api/v3/myFilters"))
    );
}

#[test]
fn elapsed_receipt_expires_rules_without_renewing_retained_evidence() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    std::thread::sleep(std::time::Duration::from_millis(30_100));
    let expired = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(expired["data"]["status"], "UNAVAILABLE", "{expired}");
    assert_eq!(expired["data"]["evidence"], good["data"]["evidence"]);
    assert_eq!(expired["data"]["observedAt"], good["data"]["observedAt"]);
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn documented_price_and_asset_exponents_remain_original_bounded_integers() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => body["symbols"][0]["filters"][0]["priceExponent"] = json!(8),
        "/api/v3/myFilters" => body["assetFilters"][0]["qtyExponent"] = json!(8),
        _ => (),
    }));
    let read = rig.refresh(&external);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    for (kind, name) in [
        ("PRICE_FILTER", "priceExponent"),
        ("MAX_ASSET", "qtyExponent"),
    ] {
        let rule = read["data"]["evidence"]["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == kind)
            .unwrap();
        assert_eq!(
            rule["knownSchema"], true,
            "Documented exponent must be typed: {rule}"
        );
        let field = rule["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == name)
            .unwrap();
        assert_eq!(field["value"], json!({"type":"INTEGER","value":"8"}));
    }
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["assetFilters"][0]["qtyExponent"] = json!(65);
        }
    }));
    let invalid = rig.refresh(&external);
    assert_eq!(invalid["data"]["status"], "UNAVAILABLE", "{invalid}");
}

#[test]
fn authentication_reflections_are_rejected_before_typed_projection() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    external.reflect_signature.set(true);
    let reflected = rig.refresh(&external);
    assert_eq!(reflected["data"]["status"], "UNAVAILABLE", "{reflected}");
    assert_eq!(reflected["data"]["evidence"], good["data"]["evidence"]);
    external.reflect_signature.set(false);
    external.reflect_key.set(true);
    let leaked = rig.refresh(&external);
    assert_eq!(leaked["data"]["status"], "UNAVAILABLE", "{leaked}");
    assert!(!leaked.to_string().contains(fixtures::KEY));
    assert!(!leaked.to_string().contains(fixtures::SECRET));
}

#[test]
fn clock_revalidation_and_reopen_retire_active_rule_evidence() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    let clock = command(
        &mut rig.control.lock().unwrap(),
        "time.revalidate",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(clock["data"]["confidence"], "TRUSTED");
    let retired = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(retired["data"]["status"], "UNAVAILABLE");
    assert_eq!(retired["data"]["evidence"], good["data"]["evidence"]);
    let fresh = rig.refresh(&external);
    assert_eq!(fresh["data"]["status"], "AVAILABLE", "{fresh}");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/time" {
            body["serverTime"] = json!(
                (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64
                    + 60_000
            );
        }
    }));
    let future = rig.refresh(&external);
    assert_eq!(future["data"]["status"], "UNAVAILABLE", "{future}");
    assert_eq!(future["data"]["evidence"], fresh["data"]["evidence"]);
    let RuleRig {
        _folder,
        control,
        workspace,
        account,
        vault,
    } = rig;
    drop(control);
    let mut reopened = ControlPlane::new(_folder.path().into());
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let source = command(
        &mut reopened,
        "data.binance_rules.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(source["data"]["status"], "UNVERIFIED");
    assert_eq!(source["data"]["connectionId"], account["connectionId"]);
    assert_eq!(source["data"]["instrumentId"], "crypto:BTC/USDT:spot");
    assert!(source["data"]["evidence"].is_null());
    assert!(source["data"]["observedAt"].is_null());
    let stored: tradex::providers::AccountConnection = serde_json::from_value(account).unwrap();
    assert!(vault.get(&stored.credential_ref()).is_ok());
}

#[test]
fn collected_constraints_name_the_unresolved_per_order_obligations() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let read = rig.refresh(&external);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let obligations = read["data"]["evidence"]["unresolvedObligations"]
        .as_array()
        .unwrap();
    for expected in [
        "PRICE_TICK_AND_BOUND_VALIDATION_REQUIRED",
        "BASE_QUANTITY_GRID_VALIDATION_REQUIRED",
        "NOTIONAL_AND_MARKET_REFERENCE_VALIDATION_REQUIRED",
        "CURRENT_ACCOUNT_EXCHANGE_ORDER_COUNTS_REQUIRED",
        "EXACT_ORDER_ASSET_AMOUNT_VALIDATION_REQUIRED",
        "BOOK_REFERENCE_PRICE_RANGE_VALIDATION_REQUIRED",
    ] {
        assert!(
            obligations.contains(&json!(expected)),
            "Unresolved constraint is not individually represented: {expected}: {obligations:?}"
        );
    }
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn rule_counts_preserve_large_integers_but_reject_values_outside_documented_int64() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["exchangeFilters"][0]["maxNumOrders"] = json!(9007199254740993u64);
        }
    }));
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    let constraint = good["data"]["evidence"]["constraints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "EXCHANGE_MAX_NUM_ORDERS")
        .unwrap();
    assert_eq!(
        constraint["fields"][0]["value"],
        json!({"type":"INTEGER","value":"9007199254740993"})
    );
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["exchangeFilters"][0]["maxNumOrders"] = json!(u64::MAX);
        }
    }));
    let failed = rig.refresh(&external);
    assert_eq!(
        failed["data"]["status"], "UNAVAILABLE",
        "Out-of-range int64 must fail closed: {failed}"
    );
    assert_eq!(failed["data"]["evidence"], good["data"]["evidence"]);
    assert_ne!(rule_check(&rig.decision())["outcome"], "PASS");
}

#[test]
fn unpopulated_permission_sets_never_grant_account_admission() {
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE", "{good}");
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => body["symbols"][0]["permissionSets"] = json!([]),
        "/api/v3/account" => body["permissions"] = json!([]),
        _ => (),
    }));
    let empty = rig.refresh(&external);
    assert_eq!(
        empty["data"]["status"], "UNAVAILABLE",
        "Unpopulated permissions are not admission evidence: {empty}"
    );
    assert_eq!(empty["data"]["evidence"], good["data"]["evidence"]);
    assert!(
        empty["data"]["capabilityStatuses"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] != "AVAILABLE")
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["permissionSets"] = json!([[]]);
        }
    }));
    let empty_inner = rig.refresh(&external);
    assert_eq!(empty_inner["data"]["status"], "UNAVAILABLE");
    external.edit.set(None);
    let recovered = rig.refresh(&external);
    assert_eq!(recovered["data"]["status"], "AVAILABLE", "{recovered}");
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}

#[test]
fn original_order_form_flags_are_preserved_or_explicitly_unobserved_and_never_grant_advanced_forms()
{
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            for (name, enabled) in [
                ("icebergAllowed", true),
                ("ocoAllowed", false),
                ("otoAllowed", true),
                ("opoAllowed", false),
                ("allowTrailingStop", true),
                ("cancelReplaceAllowed", false),
                ("amendAllowed", true),
                ("pegInstructionsAllowed", false),
            ] {
                body["symbols"][0][name] = json!(enabled);
            }
        }
    }));
    let good = rig.refresh(&external);
    assert_eq!(good["data"]["status"], "AVAILABLE");
    assert_eq!(
        good["data"]["evidence"]["orderFormFlags"],
        json!({"icebergAllowed":true,"ocoAllowed":false,"otoAllowed":true,"opoAllowed":false,"allowTrailingStop":true,"cancelReplaceAllowed":false,"amendAllowed":true,"pegInstructionsAllowed":false})
    );
    assert!(
        good["data"]["evidence"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("ADVANCED_ORDER_FORMS_UNSUPPORTED"))
    );
    let raw = external
        .get(
            tradex::provider_io::ProviderEndpoint::BinanceLive,
            "/api/v3/exchangeInfo?symbol=BTCUSDT&showPermissionSets=true",
            reqwest::header::HeaderMap::new(),
        )
        .unwrap();
    for name in [
        "icebergAllowed",
        "ocoAllowed",
        "otoAllowed",
        "opoAllowed",
        "allowTrailingStop",
        "cancelReplaceAllowed",
        "amendAllowed",
        "pegInstructionsAllowed",
    ] {
        let mut malformed: Value = serde_json::from_slice(&raw).unwrap();
        malformed["symbols"][0][name] = json!(1);
        *external.raw.borrow_mut() = Some((
            "/api/v3/exchangeInfo".into(),
            serde_json::to_vec(&malformed).unwrap(),
        ));
        let rejected = rig.refresh(&external);
        assert_eq!(
            rejected["data"]["status"], "UNAVAILABLE",
            "Nonboolean {name} must fail closed: {rejected}"
        );
        assert_eq!(rejected["data"]["evidence"], good["data"]["evidence"]);
    }
    *external.raw.borrow_mut() = None;
    external.edit.set(None);
    let absent = rig.refresh(&external);
    assert_eq!(absent["data"]["status"], "AVAILABLE");
    assert!(
        absent["data"]["evidence"]["orderFormFlags"]
            .as_object()
            .unwrap()
            .values()
            .all(Value::is_null)
    );
    assert!(
        absent["data"]["evidence"]["unresolvedObligations"]
            .as_array()
            .unwrap()
            .contains(&json!("ORDER_FORM_FLAGS_UNOBSERVED"))
    );
    assert_eq!(rule_check(&rig.decision())["outcome"], "UNAVAILABLE");
}
