use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch_with_events(
        json!({"requestId":"spot-market","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}

#[test]
fn public_spot_market_selection_is_versioned_metadata_without_trading_credentials() {
    let folder = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(folder.path().into());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true);
    let workspace = opened["data"]["workspaceId"].clone();
    let stock = command(
        &mut control,
        "data.source.connection",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        source["ok"], true,
        "Public ordinary Spot market-source selection is missing: {source}"
    );
    assert_eq!(source["data"]["configured"], false);
    assert_eq!(source["data"]["source"]["sourceId"], "BINANCE_SPOT_PUBLIC");
    let payload =
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"]});
    let saved = command(
        &mut control,
        "data.binance_market.configure",
        payload.clone(),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["configured"], true);
    assert_eq!(saved["data"]["source"]["configured"], true);
    assert_eq!(saved["data"]["source"]["status"], "UNVERIFIED");
    assert!(saved["data"]["source"]["observedAt"].is_null());
    assert!(saved["data"]["source"]["checkedAt"].is_null());
    let conflict = command(&mut control, "data.binance_market.configure", payload);
    assert_eq!(conflict["error"]["code"], "STATE_VERSION_CONFLICT");
    assert_eq!(
        command(
            &mut control,
            "data.source.connection",
            json!({"workspaceId":workspace})
        )["data"],
        stock["data"],
        "Public Spot selection must not change the stock source"
    );
    drop(control);
    let mut control = ControlPlane::new(folder.path().into());
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let reopened = command(
        &mut control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(reopened["data"]["configured"], true);
    assert_eq!(
        reopened["data"]["stateVersion"],
        saved["data"]["stateVersion"]
    );
    assert_eq!(reopened["data"]["source"]["status"], "UNVERIFIED");
    assert!(reopened["data"]["source"]["observedAt"].is_null());
    let disconnected = command(
        &mut control,
        "data.binance_market.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":reopened["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    assert_eq!(disconnected["data"]["configured"], false);
    assert_eq!(disconnected["data"]["source"]["status"], "BLOCKED_EXTERNAL");
}

#[test]
fn binance_local_use_confirmation_is_strict_versioned_and_user_reported() {
    let folder = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(folder.path().into());
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace = opened["data"]["workspaceId"].clone();
    let initial = command(
        &mut control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        initial["data"]["localUseAssessment"]["state"],
        "SOURCE_NOT_SELECTED"
    );
    let preselected = command(
        &mut control,
        "data.binance_market.attest_local_use",
        json!({"workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"],"declaredJurisdiction":"CA","confirmLocalTradingDecisionSupport":true}),
    );
    assert_eq!(
        preselected["error"]["code"],
        "BINANCE_LOCAL_USE_SOURCE_UNAVAILABLE"
    );

    let selected = command(
        &mut control,
        "data.binance_market.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"]}),
    );
    assert_eq!(selected["ok"], true, "{selected}");
    assert_eq!(
        selected["data"]["localUseAssessment"]["state"],
        "NOT_RECORDED"
    );

    for (payload, code) in [
        (
            json!({"workspaceId":workspace,"expectedStateVersion":selected["data"]["stateVersion"],"declaredJurisdiction":"ca","confirmLocalTradingDecisionSupport":true}),
            "IPC_PAYLOAD_INVALID",
        ),
        (
            json!({"workspaceId":workspace,"expectedStateVersion":selected["data"]["stateVersion"],"declaredJurisdiction":"CA","confirmLocalTradingDecisionSupport":false}),
            "IPC_PAYLOAD_INVALID",
        ),
        (
            json!({"workspaceId":workspace,"expectedStateVersion":selected["data"]["stateVersion"],"declaredJurisdiction":"CA","confirmLocalTradingDecisionSupport":true,"providerId":"binance"}),
            "IPC_PAYLOAD_INVALID",
        ),
        (
            json!({"workspaceId":workspace,"expectedStateVersion":selected["data"]["stateVersion"],"declaredJurisdiction":"ZZ","confirmLocalTradingDecisionSupport":true}),
            "BINANCE_LOCAL_USE_JURISDICTION_UNSUPPORTED",
        ),
    ] {
        let rejected = command(
            &mut control,
            "data.binance_market.attest_local_use",
            payload,
        );
        assert_eq!(rejected["error"]["code"], code, "{rejected}");
    }
    let restricted = command(
        &mut control,
        "data.binance_market.attest_local_use",
        json!({"workspaceId":workspace,"expectedStateVersion":selected["data"]["stateVersion"],"declaredJurisdiction":"US","confirmLocalTradingDecisionSupport":true}),
    );
    assert_eq!(restricted["error"]["code"], "BINANCE_LOCAL_USE_RESTRICTED");

    let recorded = command(
        &mut control,
        "data.binance_market.attest_local_use",
        json!({"workspaceId":workspace,"expectedStateVersion":selected["data"]["stateVersion"],"declaredJurisdiction":"CA","confirmLocalTradingDecisionSupport":true}),
    );
    assert_eq!(recorded["ok"], true, "{recorded}");
    assert_ne!(
        recorded["data"]["stateVersion"],
        selected["data"]["stateVersion"]
    );
    assert_eq!(recorded["data"]["localUseAssessment"]["state"], "CURRENT");
    assert!(
        recorded["data"]["localUseAssessment"]["reason"]
            .as_str()
            .unwrap()
            .contains("provider entitlement remains UNVERIFIED")
    );
    let attestation = &recorded["data"]["localUseAttestation"];
    assert_eq!(attestation["provenance"], "USER_ATTESTED_LOCAL_ONLY");
    assert_eq!(attestation["scope"], "LOCAL_TRADING_DECISION_SUPPORT");
    assert_eq!(attestation["workspaceId"], workspace);
    assert_eq!(attestation["sourceGeneration"], 2);
    assert_eq!(attestation["sourceId"], "BINANCE_SPOT_PUBLIC");
    assert_eq!(attestation["providerId"], "binance");
    assert_eq!(attestation["environment"], "ORDINARY");
    assert_eq!(
        attestation["datasetProfile"],
        "ORDINARY_PUBLIC_SPOT_QUOTES_AND_BOUNDED_DEPTH"
    );
    assert_eq!(attestation["symbols"], json!(["BTCUSDT", "ETHUSDT"]));
    assert_eq!(attestation["declaredJurisdiction"], "CA");
    assert_eq!(attestation["destination"], "LOCAL_ONLY");
    assert_eq!(
        attestation["retentionBoundary"],
        "EPHEMERAL_LOCAL_EVIDENCE_NO_QUOTE_OR_BOOK_EXPORT"
    );
    assert_eq!(attestation["termsUrl"], "https://www.binance.com/en/terms");
    assert_eq!(attestation["termsReviewedAt"], "2026-10-08");
    assert_eq!(attestation["bindingDigest"].as_str().unwrap().len(), 64);
    assert!(attestation["confirmedAt"].as_str().unwrap().contains('T'));
    assert!(recorded["data"]["source"]["observedAt"].is_null());
    assert!(recorded["data"]["source"]["checkedAt"].is_null());

    drop(control);
    let mut control = ControlPlane::new(folder.path().into());
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(
        &mut control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(restored["data"]["localUseAttestation"], attestation.clone());
    assert_eq!(restored["data"]["localUseAssessment"]["state"], "CURRENT");

    let conflict = command(
        &mut control,
        "data.binance_market.withdraw_local_use",
        json!({"workspaceId":workspace,"expectedStateVersion":selected["data"]["stateVersion"]}),
    );
    assert_eq!(conflict["error"]["code"], "STATE_VERSION_CONFLICT");
    let withdrawn = command(
        &mut control,
        "data.binance_market.withdraw_local_use",
        json!({"workspaceId":workspace,"expectedStateVersion":restored["data"]["stateVersion"]}),
    );
    assert_eq!(withdrawn["ok"], true, "{withdrawn}");
    assert_eq!(withdrawn["data"]["localUseAttestation"], Value::Null);
    assert_eq!(
        withdrawn["data"]["localUseAssessment"]["state"],
        "NOT_RECORDED"
    );
    assert_ne!(
        withdrawn["data"]["stateVersion"],
        restored["data"]["stateVersion"]
    );

    let reattested = command(
        &mut control,
        "data.binance_market.attest_local_use",
        json!({"workspaceId":workspace,"expectedStateVersion":withdrawn["data"]["stateVersion"],"declaredJurisdiction":"CA","confirmLocalTradingDecisionSupport":true}),
    );
    assert_eq!(reattested["data"]["localUseAssessment"]["state"], "CURRENT");
    let reconfigured = command(
        &mut control,
        "data.binance_market.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":reattested["data"]["stateVersion"]}),
    );
    assert_eq!(reconfigured["ok"], true, "{reconfigured}");
    assert_eq!(
        reconfigured["data"]["localUseAssessment"]["state"],
        "NOT_RECORDED"
    );
    assert_eq!(reconfigured["data"]["localUseAttestation"], Value::Null);

    let reattested = command(
        &mut control,
        "data.binance_market.attest_local_use",
        json!({"workspaceId":workspace,"expectedStateVersion":reconfigured["data"]["stateVersion"],"declaredJurisdiction":"CA","confirmLocalTradingDecisionSupport":true}),
    );
    let disconnected = command(
        &mut control,
        "data.binance_market.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":reattested["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    assert_eq!(
        disconnected["data"]["localUseAssessment"]["state"],
        "SOURCE_NOT_SELECTED"
    );
    assert_eq!(disconnected["data"]["localUseAttestation"], Value::Null);
}

#[test]
fn public_catalog_keeps_spot_policy_and_selection_separate_from_stock_access() {
    let folder = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(folder.path().into());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let catalog = command(
        &mut control,
        "data.source.catalog",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(catalog["ok"], true);
    let source = catalog["data"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["sourceId"] == "BINANCE_SPOT_PUBLIC");
    assert!(
        source.is_some(),
        "The public registry must expose the separate ordinary Spot source: {catalog}"
    );
    let source = source.unwrap();
    assert_eq!(source["configured"], false);
    assert_eq!(source["status"], "BLOCKED_EXTERNAL");
    assert!(
        source["entitlement"]
            .as_str()
            .unwrap()
            .contains("no trading credentials")
    );
    assert!(
        source["redistribution"]
            .as_str()
            .unwrap()
            .contains("UNVERIFIED")
    );
    let stock = catalog["data"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["sourceId"] == "OD-001")
        .unwrap()
        .clone();
    let selection = command(
        &mut control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        command(
            &mut control,
            "data.binance_market.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":selection["data"]["stateVersion"]})
        )["ok"],
        true
    );
    let catalog = command(
        &mut control,
        "data.source.catalog",
        json!({"workspaceId":workspace}),
    );
    let rows = catalog["data"]["sources"].as_array().unwrap();
    assert_eq!(
        rows.iter().find(|s| s["sourceId"] == "OD-001").unwrap(),
        &stock
    );
    let selected = rows
        .iter()
        .find(|s| s["sourceId"] == "BINANCE_SPOT_PUBLIC")
        .unwrap();
    assert_eq!(selected["configured"], true);
    assert_eq!(selected["status"], "UNVERIFIED");
    assert!(selected["observedAt"].is_null());
}

#[test]
fn spot_source_metadata_rejects_untrusted_consumers_and_credential_or_endpoint_overrides() {
    let folder = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(folder.path().into());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let before = command(
        &mut control,
        "data.binance_market.connection",
        json!({"workspaceId":workspace}),
    );
    let denied = control.dispatch_with_events(json!({"requestId":"untrusted-source","schemaVersion":1,"command":"data.binance_market.configure","payload":{"workspaceId":workspace,"expectedStateVersion":before["data"]["stateVersion"]}}),"agent",None);
    assert_eq!(denied["error"]["code"], "IPC_ACCESS_DENIED");
    for field in [
        "apiKey",
        "apiSecret",
        "endpoint",
        "connectionId",
        "providerSymbol",
    ] {
        let mut payload =
            json!({"workspaceId":workspace,"expectedStateVersion":before["data"]["stateVersion"]});
        payload[field] = json!("untrusted-override");
        let rejected = command(&mut control, "data.binance_market.configure", payload);
        assert_eq!(
            rejected["error"]["code"], "IPC_PAYLOAD_INVALID",
            "{field}: {rejected}"
        );
    }
    assert_eq!(
        command(
            &mut control,
            "data.binance_market.connection",
            json!({"workspaceId":workspace})
        )["data"],
        before["data"]
    );
}
