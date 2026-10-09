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

// Independent external-provider scenarios each own one application process.
// The parent serializes them; production IP/UID quotas are unchanged inside a
// scenario. This also keeps a ban/fault scenario from poisoning unrelated cases.
fn isolated_rule_scenario(name: &str) -> bool {
    if std::env::var("TRADEX_RULE_SCENARIO").ok().as_deref() == Some(name) {
        return false;
    }
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([name, "--exact", "--nocapture", "--test-threads=1"])
        .env("TRADEX_RULE_SCENARIO", name)
        .output()
        .expect("Run isolated rule scenario");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "Isolated scenario failed: {name}");
    true
}

#[test]
fn immutable_spot_proposal_explains_static_rules_without_hiding_dynamic_obligations() {
    if isolated_rule_scenario(
        "immutable_spot_proposal_explains_static_rules_without_hiding_dynamic_obligations",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let result = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    assert_eq!(
        result["ok"], true,
        "Per-Proposal rule explanation is unavailable: {result}"
    );
    assert_eq!(result["data"]["proposalHash"], decision["proposalHash"]);
    assert_eq!(result["data"]["quoteAsset"], "USDT");
    let rows = result["data"]["rules"].as_array().unwrap();
    for kind in ["PRICE_FILTER", "LOT_SIZE", "MIN_NOTIONAL", "MAX_ASSET"] {
        assert_eq!(
            rows.iter().find(|r| r["ruleType"] == kind).unwrap()["outcome"],
            "PASS",
            "{result}"
        );
    }
    assert_eq!(
        rows.iter()
            .find(|r| r["ruleType"] == "PRICE_RANGE")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
    assert_eq!(result["data"]["outcome"], "UNAVAILABLE");
    assert_ne!(decision["status"], "ALLOWED");
}

#[test]
fn current_static_price_violation_rejects_risk_and_preserves_its_proposal_binding() {
    if isolated_rule_scenario(
        "current_static_price_violation_rejects_risk_and_preserves_its_proposal_binding",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"][0]["tickSize"] = json!("7");
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    assert_eq!(
        rule_check(&decision)["outcome"],
        "REJECT",
        "The exact 60000 limit is not a multiple of7: {decision}"
    );
    assert_eq!(
        decision["spotRules"]["proposalHash"],
        decision["proposalHash"]
    );
    assert_eq!(decision["spotRules"]["outcome"], "REJECT");
    assert_eq!(
        rule_check(&decision)["reasonCode"],
        "INSTRUMENT_RULES_REJECTED"
    );
}

#[test]
fn empty_collected_constraints_cannot_qualify_an_ordinary_spot_proposal() {
    if isolated_rule_scenario(
        "empty_collected_constraints_cannot_qualify_an_ordinary_spot_proposal",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| match route {
        "/api/v3/exchangeInfo" => body["symbols"][0]["filters"] = json!([]),
        "/api/v3/executionRules" => body["symbolRules"][0]["rules"] = json!([]),
        "/api/v3/myFilters" => {
            body["exchangeFilters"] = json!([]);
            body["symbolFilters"] = json!([]);
            body["assetFilters"] = json!([]);
        }
        _ => {}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    assert_eq!(
        rule_check(&decision)["outcome"],
        "UNAVAILABLE",
        "Metadata without required rule coverage is not qualification: {decision}"
    );
    assert!(
        decision["spotRules"]["unresolvedObligations"]
            .as_array()
            .is_some_and(|v| v.contains(&json!("REQUIRED_STATIC_RULE_COVERAGE_UNAVAILABLE")))
    );
}

#[test]
fn explicit_required_reference_read_qualifies_only_its_bound_percentage_rule() {
    if isolated_rule_scenario(
        "explicit_required_reference_read_qualifies_only_its_bound_percentage_rule",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"proposal-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        read["ok"], true,
        "Public required reference refresh is unavailable: {read}"
    );
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_eq!(read["data"]["references"][0]["kind"], "PROVIDER_REFERENCE");
    assert_eq!(read["data"]["references"][0]["price"], "60000");
    assert_eq!(
        read["data"]["outcome"], "UNAVAILABLE",
        "Dynamic rules remain required"
    );
}

#[test]
fn explicit_null_reference_uses_only_matching_average_interval() {
    if isolated_rule_scenario("explicit_null_reference_uses_only_matching_average_interval") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));
        }
        if route == "/api/v3/referencePrice" { body["referencePrice"] = Value::Null; }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"reference-null","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"][0]["kind"], "AVERAGE_PRICE",
        "Explicit null permits a genuine average, not an error fallback: {read}"
    );
    assert_eq!(read["data"]["references"][0]["intervalMinutes"], 5);
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE")
            .unwrap()["outcome"],
        "PASS"
    );
    assert_eq!(read["data"]["outcome"], "UNAVAILABLE");
}

#[test]
fn base_market_notional_requests_its_required_reference_and_keeps_base_units() {
    if isolated_rule_scenario(
        "base_market_notional_requests_its_required_reference_and_keeps_base_units",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision=rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":null,"timeInForce":"DAY"}));
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"market-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        read["ok"], true,
        "Market BASE requires notional valuation: {read}"
    );
    let rows = read["data"]["rules"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .find(|r| r["ruleType"] == "MIN_NOTIONAL")
            .unwrap()["outcome"],
        "PASS",
        "0.001 BTC at60000USDT is60USDT"
    );
    assert_eq!(
        rows.iter().find(|r| r["ruleType"] == "LOT_SIZE").unwrap()["outcome"],
        "PASS"
    );
    assert_eq!(
        rows.iter()
            .find(|r| r["ruleType"] == "PRICE_FILTER")
            .unwrap()["applicable"],
        false
    );
    assert_ne!(read["data"]["outcome"], "PASS");
}

#[test]
fn unsupported_provider_order_form_rejects_the_unchanged_proposal() {
    if isolated_rule_scenario("unsupported_provider_order_form_rejects_the_unchanged_proposal") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["orderTypes"] = json!(["MARKET"]);
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    assert_eq!(
        rule_check(&decision)["outcome"],
        "REJECT",
        "A LIMIT Proposal cannot use a MARKET-only symbol: {decision}"
    );
    assert!(
        decision["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["ruleType"] == "ORDER_FORM" && r["outcome"] == "REJECT")
    );
}

#[test]
fn refused_reference_read_retires_previous_price_and_never_calls_average_fallback() {
    if isolated_rule_scenario(
        "refused_reference_read_retires_previous_price_and_never_calls_average_fallback",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"reference-failure","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let good = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(good["data"]["references"][0]["price"], "60000");
    *external.raw.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        br#"{"code":-2043,"msg":"This symbol doesn't have a reference price."}"#.to_vec(),
    ));
    let failed = refresh(good["data"]["stateVersion"].clone());
    assert_eq!(
        failed["ok"], true,
        "A bounded unavailable projection permits UI recovery: {failed}"
    );
    assert_eq!(
        failed["data"]["references"],
        json!([]),
        "Failed refresh must not keep prior reference eligible"
    );
    assert_eq!(
        failed["data"]["referenceFailure"], "PROVIDER_RESPONSE_INVALID",
        "Refusal retains a sanitized specific reason: {failed}"
    );
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice")),
        "Error is not explicit null"
    );
    let risk = command(
        &mut rig.control.lock().unwrap(),
        "risk.evaluate_proposal",
        query,
    );
    assert_eq!(rule_check(&risk["data"])["outcome"], "UNAVAILABLE");
}

#[test]
fn quote_asset_market_limit_rejects_exact_notional_without_changing_base_quantity() {
    if isolated_rule_scenario(
        "quote_asset_market_limit_rejects_exact_notional_without_changing_base_quantity",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/myFilters" {
            body["assetFilters"] = json!([{"filterType":"MAX_ASSET","asset":"USDT","limit":"50"}]);
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision=rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":null,"timeInForce":"DAY"}));
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"asset-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let read = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "MAX_ASSET")
            .unwrap()["outcome"],
        "REJECT",
        "60USDT exceeds50USDT; no conversion toUSD: {read}"
    );
    assert_eq!(read["data"]["outcome"], "REJECT");
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "MAX_ASSET")
            .unwrap()["unit"],
        "USDT",
        "Asset constraint must retain its actual unit"
    );
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "LOT_SIZE")
            .unwrap()["unit"],
        "BTC"
    );
    let repeated = refresh(read["data"]["stateVersion"].clone());
    assert_eq!(
        repeated["ok"], true,
        "Required input remains refreshable after a complete read: {repeated}"
    );
}

#[test]
fn reference_deadline_retires_prior_input_without_blocking_control_plane_reads() {
    if isolated_rule_scenario(
        "reference_deadline_retires_prior_input_without_blocking_control_plane_reads",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"reference-deadline","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let good = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(good["data"]["references"][0]["price"], "60000");
    let cp = rig.control.clone();
    let q = query.clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        Box::new(move || {
            let during = command(
                &mut cp
                    .try_lock()
                    .expect("Network I/O must not hold the CP lock"),
                "trade.spot_rules.get",
                q.clone(),
            );
            assert_eq!(during["ok"], true);
            std::thread::sleep(std::time::Duration::from_millis(10020));
        }),
    ));
    let late = refresh(good["data"]["stateVersion"].clone());
    assert_eq!(
        late["ok"], true,
        "A same-binding expired read must retire prior eligibility and expose recovery: {late}"
    );
    assert_eq!(late["data"]["references"], json!([]));
    assert_eq!(late["data"]["referenceFailure"], "PROVIDER_UNAVAILABLE");
    let after = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query,
    );
    assert_eq!(after["data"]["references"], json!([]));
}

#[test]
fn identical_reference_cannot_renew_first_receipt_or_financial_freshness() {
    if isolated_rule_scenario(
        "identical_reference_cannot_renew_first_receipt_or_financial_freshness",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let raw = json!({"symbol":"BTCUSDT","referencePrice":"60000","timestamp":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64});
    *external.raw.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        serde_json::to_vec(&raw).unwrap(),
    ));
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"reference-age","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let first = refresh(before["data"]["stateVersion"].clone());
    let second = refresh(first["data"]["stateVersion"].clone());
    assert_eq!(
        first["data"]["references"], second["data"]["references"],
        "Identical provider material preserves its first receipt"
    );
    std::thread::sleep(std::time::Duration::from_millis(3100));
    let old = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    assert_eq!(
        old["data"]["references"],
        json!([]),
        "Default3-second financial freshness is stricter than30-second rule metadata: {old}"
    );
    assert_eq!(old["data"]["referenceFailure"], "REFERENCE_PRICE_STALE");
    let repeated = refresh(old["data"]["stateVersion"].clone());
    assert_eq!(
        repeated["data"]["references"],
        json!([]),
        "An unchanged cached original time cannot renew reference eligibility"
    );
}

#[test]
fn zero_interval_uses_original_last_trade_time_only_after_explicit_null() {
    if isolated_rule_scenario(
        "zero_interval_uses_original_last_trade_time_only_after_explicit_null",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":0})); }
        if route == "/api/v3/referencePrice" { body["referencePrice"]=Value::Null; }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"zero-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"][0]["kind"], "LAST_TRADE",
        "Zero interval needs original trade time, not a ticker receipt or5-minute average: {read}"
    );
    assert_eq!(read["data"]["references"][0]["intervalMinutes"], 0);
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE")
            .unwrap()["outcome"],
        "PASS"
    );
    assert!(
        external
            .calls
            .borrow()
            .iter()
            .any(|p| p == "/api/v3/trades?symbol=BTCUSDT&limit=1")
    );
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|p| p.starts_with("/api/v3/avgPrice") || p.starts_with("/api/v3/ticker"))
    );
}

#[test]
fn mixed_reference_intervals_qualify_only_their_own_rules() {
    if isolated_rule_scenario("mixed_reference_intervals_qualify_only_their_own_rules") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"].as_array_mut().unwrap().extend([
                json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":0}),
                json!({"filterType":"PERCENT_PRICE_BY_SIDE","bidMultiplierDown":"0.8","bidMultiplierUp":"1.2","askMultiplierDown":"0.8","askMultiplierUp":"1.2","avgPriceMins":5})]);
        }
        if route == "/api/v3/referencePrice" { body["referencePrice"] = Value::Null; }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"mixed-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"].as_array().unwrap().len(),
        2,
        "Different rule intervals cannot be substituted: {read}"
    );
    for kind in ["PERCENT_PRICE", "PERCENT_PRICE_BY_SIDE"] {
        assert_eq!(
            read["data"]["rules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["ruleType"] == kind)
                .unwrap()["outcome"],
            "PASS"
        );
    }
    assert_eq!(read["data"]["outcome"], "UNAVAILABLE");
}

#[test]
fn future_reference_time_is_not_accepted_as_a_new_receipt() {
    if isolated_rule_scenario("future_reference_time_is_not_accepted_as_a_new_receipt") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); }
        if route == "/api/v3/referencePrice" { body["timestamp"]=json!((time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64 +1000); }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"future-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["data"]["references"],
        json!([]),
        "Future provider time must not qualify: {read}"
    );
    assert_eq!(read["data"]["referenceFailure"], "REFERENCE_PRICE_FUTURE");
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice"))
    );
}

#[test]
fn public_reference_ip_ban_retires_price_and_honors_shared_cooldown() {
    if isolated_rule_scenario("public_reference_ip_ban_retires_price_and_honors_shared_cooldown") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"ban-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query,
    );
    let good = refresh(before["data"]["stateVersion"].clone());
    assert_eq!(good["data"]["references"].as_array().unwrap().len(), 1);
    *external.response.borrow_mut() = Some(("/api/v3/referencePrice".into(), 418, 1));
    let banned = refresh(good["data"]["stateVersion"].clone());
    assert_eq!(banned["data"]["references"], json!([]));
    *external.response.borrow_mut() = None;
    let started = std::time::Instant::now();
    let recovered = refresh(banned["data"]["stateVersion"].clone());
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(900),
        "An IP ban must delay the next shared-IP reference read"
    );
    assert_eq!(banned["data"]["referenceFailure"], "PROVIDER_IP_BANNED");
    assert_eq!(recovered["data"]["references"].as_array().unwrap().len(), 1);
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice"))
    );
}

#[test]
fn proposal_explanation_distinguishes_missing_reference_dynamic_and_unknown_rules() {
    if isolated_rule_scenario(
        "proposal_explanation_distinguishes_missing_reference_dynamic_and_unknown_rules",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" {
        body["symbols"][0]["filters"].as_array_mut().unwrap().extend([
            json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}),
            json!({"filterType":"FUTURE_FILTER","threshold":"10"})]);
    } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let rows = decision["spotRules"]["rules"].as_array().unwrap();
    for (kind, reason) in [
        ("PERCENT_PRICE", "REFERENCE_PRICE_MISSING"),
        (
            "PRICE_RANGE",
            "BOOK_REFERENCE_PRICE_RANGE_VALIDATION_REQUIRED",
        ),
        (
            "EXCHANGE_MAX_NUM_ORDERS",
            "CURRENT_ACCOUNT_EXCHANGE_ORDER_COUNTS_REQUIRED",
        ),
        ("FUTURE_FILTER", "UNSUPPORTED_ACTIVE_CONSTRAINT_SCHEMA"),
    ] {
        let row = rows.iter().find(|r| r["ruleType"] == kind).unwrap();
        assert_eq!(row["outcome"], "UNAVAILABLE");
        assert_eq!(row["reasonCode"], reason, "{row}");
    }
}

#[test]
fn fully_observed_applicable_static_rules_do_not_invent_an_absent_notional_filter() {
    if isolated_rule_scenario(
        "fully_observed_applicable_static_rules_do_not_invent_an_absent_notional_filter",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| match route {
        "/api/v3/account" => body["requireSelfTradePrevention"] = json!(false),
        "/api/v3/myFilters" => {
            body["exchangeFilters"] = json!([]);
            body["symbolFilters"] = json!([]);
            body["assetFilters"] = json!([]);
        }
        "/api/v3/executionRules" => body["symbolRules"][0]["rules"] = json!([]),
        _ => {}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let selected = command(
        &mut rig.control.lock().unwrap(),
        "data.binance_market.connection",
        json!({"workspaceId":rig.workspace}),
    );
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "data.binance_market.configure",
            json!({"workspaceId":rig.workspace,"expectedStateVersion":selected["data"]["stateVersion"]})
        )["ok"],
        true
    );
    let decision = rig.decision();
    assert_eq!(
        decision["spotRules"]["outcome"], "PASS",
        "Completeness evaluates actually returned applicable constraints: {decision}"
    );
    assert_eq!(
        decision["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["checkId"] == "MARKET_DATA_USE")
            .unwrap()["outcome"],
        "UNAVAILABLE",
        "Static success cannot grant public-market financial-use rights"
    );
    assert_eq!(rule_check(&decision)["outcome"], "PASS");
    assert_ne!(decision["status"], "ALLOWED");
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "account.get",
            json!({"workspaceId":rig.workspace,"connectionId":rig.account["connectionId"]})
        )["data"]["health"]["arming"],
        "DISARMED"
    );
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/referencePrice"))
    );
}

#[test]
fn proposal_rule_guards_keep_exact_intent_units_and_reference_faults_closed() {
    if isolated_rule_scenario(
        "proposal_rule_guards_keep_exact_intent_units_and_reference_faults_closed",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE_BY_SIDE","bidMultiplierDown":"0.9","bidMultiplierUp":"1.1","askMultiplierDown":"1.1","askMultiplierUp":"1.2","avgPriceMins":5})); } }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let mut fields = json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"});
    for (quantity, price, kind, outcome) in [
        ("0.0000011", "60000", "LOT_SIZE", "REJECT"),
        ("0.000001000000000001", "60000", "LOT_SIZE", "REJECT"),
        ("0.001", "60000.001", "PRICE_FILTER", "REJECT"),
        ("0.001", "1000", "MIN_NOTIONAL", "REJECT"),
        ("0.3", "60000", "MAX_ASSET", "REJECT"),
    ] {
        fields["quantity"]["value"] = json!(quantity);
        fields["limitPrice"] = json!(price);
        let decision = rig.decision_with(fields.clone());
        let row = decision["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == kind)
            .unwrap();
        assert_eq!(row["outcome"], outcome, "{row}");
    }
    fields["quantity"]["value"] = json!("0.001");
    fields["limitPrice"] = json!("60000");
    fields["side"] = json!("SELL");
    let decision = rig.decision_with(fields);
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let refresh = |version: Value| {
        tradex::financial_sources::execute_refresh(
            &rig.control,
            &json!({"requestId":"guards-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":version}}),
            "main",
            &rig.vault,
            &external,
        )
    };
    let get = || {
        command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_rules.get",
            query.clone(),
        )
    };
    let good = refresh(get()["data"]["stateVersion"].clone());
    assert_eq!(
        good["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PERCENT_PRICE_BY_SIDE")
            .unwrap()["outcome"],
        "REJECT",
        "SELL must use ask multipliers"
    );
    for body in [
        r#"{"symbol":"BTCUSDT","referencePrice":60000,"timestamp":1}"#,
        r#"{"symbol":"BTCUSDT","referencePrice":"60000","timestamp":1.5}"#,
        r#"{"symbol":"BTCUSDT","referencePrice":"60000"}"#,
        r#"{"symbol":"BTCUSDT","referencePrice":null,"referencePrice":"60000","timestamp":1}"#,
        r#"{"symbol":"ETHUSDT","referencePrice":"60000","timestamp":1}"#,
        r#"{"code":-2043,"msg":"NO_REFERENCE_PRICE"}"#,
    ] {
        *external.raw.borrow_mut() =
            Some(("/api/v3/referencePrice".into(), body.as_bytes().to_vec()));
        let failed = refresh(get()["data"]["stateVersion"].clone());
        assert_eq!(failed["ok"], true, "{failed}");
        assert_eq!(failed["data"]["references"], json!([]), "{failed}");
        assert!(failed["data"]["referenceFailure"].is_string());
    }
    assert!(
        !external
            .calls
            .borrow()
            .iter()
            .any(|c| c.starts_with("/api/v3/avgPrice"))
    );
    let mut untrusted = query.clone();
    untrusted["price"] = json!("60000");
    assert_eq!(
        command(
            &mut rig.control.lock().unwrap(),
            "trade.spot_rules.get",
            untrusted
        )["ok"],
        false
    );
    let forbidden=rig.control.lock().unwrap().dispatch_with_events(json!({"requestId":"foreign-consumer","schemaVersion":1,"command":"trade.spot_rules.get","payload":query}),"order-gateway",None);
    assert_eq!(forbidden["error"]["code"], "IPC_ACCESS_DENIED");
    let hash = command(
        &mut rig.control.lock().unwrap(),
        "trade.proposal.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    assert_eq!(hash["data"]["proposalHash"], decision["proposalHash"]);
}

#[test]
fn late_reference_selection_change_cannot_publish_into_new_rule_binding() {
    if isolated_rule_scenario(
        "late_reference_selection_change_cannot_publish_into_new_rule_binding",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {if route == "/api/v3/exchangeInfo" {body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));}}));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let control = rig.control.clone();
    let workspace = rig.workspace.clone();
    let account = rig.account["connectionId"].clone();
    *external.hook.borrow_mut() = Some((
        "/api/v3/referencePrice".into(),
        Box::new(move || {
            let mut cp = control
                .try_lock()
                .expect("Reference I/O must leave Control Plane available");
            let source = command(
                &mut cp,
                "data.binance_rules.connection",
                json!({"workspaceId":workspace}),
            );
            assert_eq!(
                command(
                    &mut cp,
                    "data.binance_rules.configure",
                    json!({"workspaceId":workspace,"connectionId":account,"instrumentId":"crypto:ETH/USDT:spot","expectedStateVersion":source["data"]["stateVersion"]})
                )["ok"],
                true
            );
        }),
    ));
    let late = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"late-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(late["error"]["code"], "STATE_VERSION_CONFLICT");
    let current = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query,
    );
    assert_eq!(current["data"]["references"], json!([]));
    assert!(current["data"]["materialVersion"].is_null());
}

#[test]
fn reopen_preserves_only_captured_digests_and_never_renews_public_references() {
    if isolated_rule_scenario(
        "reopen_preserves_only_captured_digests_and_never_renews_public_references",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {if route == "/api/v3/exchangeInfo" {body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5}));}}));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let query = json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]});
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        query.clone(),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"capture-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["data"]["references"].as_array().unwrap().len(), 1);
    let capture = command(
        &mut rig.control.lock().unwrap(),
        "risk.evaluate_proposal",
        query.clone(),
    );
    assert_eq!(capture["ok"], true);
    assert!(capture["data"]["spotRules"]["references"][0]["price"].is_null());
    assert_eq!(
        capture["data"]["spotRules"]["references"][0]["digest"],
        read["data"]["references"][0]["digest"]
    );
    let calls = external.calls.borrow().len();
    let RuleRig {
        _folder, control, ..
    } = rig;
    drop(control);
    let mut reopened = ControlPlane::new(_folder.path().into());
    let opened = command(&mut reopened, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let history = command(&mut reopened, "risk.decision.list", query.clone());
    assert_eq!(history["ok"], true, "{history}");
    assert_eq!(
        history["data"]["decisions"]
            .as_array()
            .unwrap()
            .last()
            .unwrap(),
        &capture["data"]
    );
    let current = command(&mut reopened, "trade.spot_rules.get", query);
    assert_eq!(current["data"]["references"], json!([]));
    assert_eq!(current["data"]["outcome"], "UNAVAILABLE");
    assert_eq!(external.calls.borrow().len(), calls);
}

#[test]
fn unmatched_average_is_explained_as_interval_mismatch_without_substitution() {
    if isolated_rule_scenario(
        "unmatched_average_is_explained_as_interval_mismatch_without_substitution",
    ) {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| { if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":1})); } if route == "/api/v3/referencePrice" {body["referencePrice"]=Value::Null;} }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision = rig.decision();
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":decision["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"mismatched-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":decision["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(read["data"]["references"][0]["intervalMinutes"], 5);
    let row = read["data"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "PERCENT_PRICE")
        .unwrap();
    assert_eq!(row["outcome"], "UNAVAILABLE");
    assert_eq!(
        row["reasonCode"], "REFERENCE_PRICE_INTERVAL_MISMATCH",
        "{row}"
    );
}

#[test]
fn notional_market_flags_and_quote_quantity_never_invent_base_amounts() {
    if isolated_rule_scenario("notional_market_flags_and_quote_quantity_never_invent_base_amounts")
    {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"NOTIONAL","minNotional":"5","maxNotional":"50","applyMinToMarket":false,"applyMaxToMarket":true,"avgPriceMins":5})); }
        if route == "/api/v3/myFilters" {body["symbolFilters"]=json!([]);}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let limit = rig.decision();
    assert_eq!(
        limit["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "NOTIONAL")
            .unwrap()["outcome"],
        "REJECT",
        "Limit notional60 exceeds50USDT"
    );
    let market = json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"MARKET","quantity":{"type":"BASE","value":"0.001"},"limitPrice":null,"maximumSpend":null,"timeInForce":"DAY"});
    let base = rig.decision_with(market.clone());
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":base["proposalId"]}),
    );
    let read = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"notional-reference","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":base["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(
        read["data"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "NOTIONAL")
            .unwrap()["outcome"],
        "REJECT",
        "Base0.001 at original60000 has notional60USDT"
    );
    let mut quote = market;
    quote["quantity"] = json!({"type":"QUOTE","value":"60"});
    let decision = rig.decision_with(quote.clone());
    let rows = decision["spotRules"]["rules"].as_array().unwrap();
    assert_eq!(
        rows.iter().find(|r| r["ruleType"] == "NOTIONAL").unwrap()["outcome"],
        "REJECT"
    );
    for kind in ["LOT_SIZE", "MAX_ASSET"] {
        assert_eq!(
            rows.iter().find(|r| r["ruleType"] == kind).unwrap()["outcome"],
            "UNAVAILABLE",
            "QUOTE60 is not a known BASE amount"
        );
    }
    assert_eq!(decision["spotRules"]["referencePurposes"], json!([]));
    external.edit.set(Some(|route,body| {
        if route == "/api/v3/exchangeInfo" { body["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"NOTIONAL","minNotional":"5","maxNotional":"50","applyMinToMarket":false,"applyMaxToMarket":false,"avgPriceMins":5})); }
        if route == "/api/v3/myFilters" {body["symbolFilters"]=json!([]);}
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let exempt = rig.decision_with(quote);
    let row = exempt["spotRules"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["ruleType"] == "NOTIONAL")
        .unwrap();
    assert_eq!(row["applicable"], false);
    assert_eq!(row["outcome"], "PASS");
    let before = command(
        &mut rig.control.lock().unwrap(),
        "trade.spot_rules.get",
        json!({"workspaceId":rig.workspace,"proposalId":exempt["proposalId"]}),
    );
    let calls = external.calls.borrow().len();
    let unnecessary = tradex::financial_sources::execute_refresh(
        &rig.control,
        &json!({"requestId":"not-required","schemaVersion":1,"command":"trade.spot_rules.refresh","payload":{"workspaceId":rig.workspace,"proposalId":exempt["proposalId"],"expectedStateVersion":before["data"]["stateVersion"]}}),
        "main",
        &rig.vault,
        &external,
    );
    assert_eq!(unnecessary["error"]["code"], "REFERENCE_PRICE_NOT_REQUIRED");
    assert_eq!(external.calls.borrow().len(), calls);
}

#[test]
fn only_documented_price_zero_fields_disable_an_obligation() {
    if isolated_rule_scenario("only_documented_price_zero_fields_disable_an_obligation") {
        return;
    }
    let rig = RuleRig::new();
    let external = RuleHttp::default();
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            for name in ["minPrice", "maxPrice", "tickSize"] {
                body["symbols"][0]["filters"][0][name] = json!("0.00000000");
            }
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let decision=rig.decision_with(json!({"accountId":rig.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000.000001","maximumSpend":null,"timeInForce":"GTC"}));
    assert_eq!(
        decision["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "PRICE_FILTER")
            .unwrap()["outcome"],
        "PASS"
    );
    external.edit.set(Some(|route, body| {
        if route == "/api/v3/exchangeInfo" {
            body["symbols"][0]["filters"][1]["stepSize"] = json!("0.00000000");
        }
    }));
    assert_eq!(rig.refresh(&external)["data"]["status"], "AVAILABLE");
    let zero_quantity = rig.decision();
    assert_eq!(
        zero_quantity["spotRules"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["ruleType"] == "LOT_SIZE")
            .unwrap()["outcome"],
        "UNAVAILABLE"
    );
}

#[test]
fn public_rule_source_selection_is_versioned_metadata_and_keeps_the_borrowed_key() {
    if isolated_rule_scenario(
        "public_rule_source_selection_is_versioned_metadata_and_keeps_the_borrowed_key",
    ) {
        return;
    }
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
    response: std::cell::RefCell<Option<(String, u16, u64)>>,
    hook: std::cell::RefCell<Option<(String, Box<dyn Fn()>)>>,
}
impl tradex::provider_io::ProviderHttp for RuleHttp {
    fn get_response_with_rate_limit(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<(
        tradex::provider_io::ProviderHttpResponse,
        Option<tradex::provider_io::ProviderRateLimit>,
    )> {
        let body = self.get(endpoint, path, headers)?;
        let (status, limit) = self
            .response
            .borrow()
            .as_ref()
            .filter(|(route, _, _)| path.split('?').next() == Some(route.as_str()))
            .map_or((200, None), |(_, status, seconds)| {
                (
                    *status,
                    Some(tradex::provider_io::ProviderRateLimit {
                        retry_after_seconds: Some(*seconds),
                        ..Default::default()
                    }),
                )
            });
        Ok((
            tradex::provider_io::ProviderHttpResponse { status, body },
            limit,
        ))
    }
    fn request_with_rate_limit(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        method: tradex::provider_io::ProviderHttpMethod,
        path: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<&Value>,
    ) -> tradex::protocol::Result<(
        tradex::provider_io::ProviderHttpResponse,
        Option<tradex::provider_io::ProviderRateLimit>,
    )> {
        assert_eq!(method, tradex::provider_io::ProviderHttpMethod::Get);
        assert!(body.is_none());
        self.get_response_with_rate_limit(endpoint, path, headers)
    }
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
                | "/api/v3/referencePrice"
                | "/api/v3/avgPrice"
                | "/api/v3/trades"
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
            "/api/v3/trades" => {
                assert_eq!(query, "symbol=BTCUSDT&limit=1");
                json!([{"id":9007199254740993u64,"price":"60000.00000000","qty":"0.001","quoteQty":"60","time":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64,"isBuyerMaker":false,"isBestMatch":true}])
            }
            "/api/v3/avgPrice" => {
                assert_eq!(query, "symbol=BTCUSDT");
                json!({"mins":5,"price":"60000.00000000","closeTime":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})
            }
            "/api/v3/referencePrice" => {
                assert_eq!(query, "symbol=BTCUSDT");
                json!({"symbol":"BTCUSDT","referencePrice":"60000.00000000","timestamp":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as u64})
            }
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
    if isolated_rule_scenario(
        "public_rule_refresh_collects_exact_account_and_scoped_constraints_without_execution_authority",
    ) {
        return;
    }
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
        self.decision_with(json!({"accountId":self.account["connectionId"],"venue":"BINANCE","environment":"BINANCE_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.001"},"limitPrice":"60000","maximumSpend":null,"timeInForce":"GTC"}))
    }
    fn decision_with(&self, fields: Value) -> Value {
        let mut control = self.control.lock().unwrap();
        let draft = command(
            &mut control,
            "trade.save_draft",
            json!({"workspaceId":self.workspace,"fields":fields}),
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
    if isolated_rule_scenario(
        "malformed_rule_range_retires_current_evidence_and_retains_the_last_good_observation",
    ) {
        return;
    }
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
    if isolated_rule_scenario(
        "actual_halt_evidence_rejects_new_order_rules_and_normal_recovery_never_grants_execution",
    ) {
        return;
    }
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
    if isolated_rule_scenario(
        "original_wire_types_and_duplicate_fields_cannot_replace_a_good_observation",
    ) {
        return;
    }
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
    if isolated_rule_scenario(
        "permission_sets_apply_or_within_and_between_sets_without_granting_per_order_qualification",
    ) {
        return;
    }
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
    if isolated_rule_scenario(
        "changing_the_selected_instrument_during_http_prevents_late_publication",
    ) {
        return;
    }
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
    if isolated_rule_scenario("elapsed_receipt_expires_rules_without_renewing_retained_evidence") {
        return;
    }
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
    if isolated_rule_scenario(
        "documented_price_and_asset_exponents_remain_original_bounded_integers",
    ) {
        return;
    }
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
    if isolated_rule_scenario("authentication_reflections_are_rejected_before_typed_projection") {
        return;
    }
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
    if isolated_rule_scenario("clock_revalidation_and_reopen_retire_active_rule_evidence") {
        return;
    }
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
    if isolated_rule_scenario("collected_constraints_name_the_unresolved_per_order_obligations") {
        return;
    }
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
    if isolated_rule_scenario(
        "rule_counts_preserve_large_integers_but_reject_values_outside_documented_int64",
    ) {
        return;
    }
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
    if isolated_rule_scenario("unpopulated_permission_sets_never_grant_account_admission") {
        return;
    }
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
    if isolated_rule_scenario(
        "original_order_form_flags_are_preserved_or_explicitly_unobserved_and_never_grant_advanced_forms",
    ) {
        return;
    }
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
