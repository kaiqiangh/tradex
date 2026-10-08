use serde_json::{Value, json};
use tradex::ControlPlane;
#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod provider_fixtures;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch_with_events(
        json!({"requestId":"financial-source","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}

fn connect(
    control: &mut ControlPlane,
    workspace: &Value,
    provider: &str,
    environment: &str,
    vault: &provider_fixtures::Vault,
) -> Value {
    let http = provider_fixtures::Http::default();
    connect_using(control, workspace, provider, environment, vault, &http)
}
fn connect_using(
    control: &mut ControlPlane,
    workspace: &Value,
    provider: &str,
    environment: &str,
    vault: &provider_fixtures::Vault,
    http: &impl tradex::provider_io::ProviderHttp,
) -> Value {
    let request = json!({"requestId":"source-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":provider,"environment":environment,"label":"Read-only source"}});
    let job = control.prepare_provider(&request).unwrap().unwrap();
    let result = job.run(
        vault,
        |_| {
            if provider == "bitget" {
                provider_fixtures::bitget::credentials()
            } else {
                provider_fixtures::credentials()
            }
        },
        http,
        || control.provider_job_current(&job),
    );
    let tested = control.complete_provider(&job, result);
    assert_eq!(tested["ok"], true, "{tested}");
    let account = command(
        control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":true}),
    );
    assert_eq!(account["ok"], true, "{account}");
    assert_eq!(account["data"]["permissions"]["scope"], "UNVERIFIED");
    account["data"].clone()
}

#[test]
fn source_selection_reopens_as_metadata_and_disconnect_keeps_the_account_key() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let source = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        source["ok"], true,
        "Known-event source selection is unavailable: {source}"
    );
    assert_eq!(source["data"]["configured"], false);
    assert_eq!(
        source["data"]["eligibleAccounts"][0]["connectionId"],
        account["connectionId"]
    );
    let saved = command(
        &mut control,
        "data.actions.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["configured"], true);
    assert_eq!(saved["data"]["status"], "UNVERIFIED");
    assert!(saved["data"]["observedAt"].is_null());
    let stale = command(
        &mut control,
        "data.actions.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    drop(control);
    let mut control = ControlPlane::new(path);
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(restored["data"]["connectionId"], account["connectionId"]);
    assert_eq!(restored["data"]["status"], "UNVERIFIED");
    let disconnected = command(
        &mut control,
        "data.actions.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":restored["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["data"]["configured"], false);
    let retained = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(retained["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(retained["data"]["health"]["arming"], "NOT_APPLICABLE");
    assert_eq!(vault.present.borrow().len(), 1);
    for projection in [saved, restored, disconnected] {
        for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
            assert!(!projection.to_string().contains(secret));
        }
    }
}

#[test]
fn known_event_query_keeps_date_precision_and_never_certifies_complete_coverage() {
    use reqwest::header::HeaderMap;
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    struct EventsHttp {
        paths: RefCell<Vec<String>>,
    }
    impl ProviderHttp for EventsHttp {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            assert_eq!(endpoint, ProviderEndpoint::AlpacaMarketData);
            assert_eq!(headers["APCA-API-KEY-ID"], provider_fixtures::KEY);
            assert!(path.starts_with(
                "/v1/corporate-actions?symbols=AAPL,MSFT&region=us&data_quality=all&start="
            ));
            assert!(path.contains("&limit=100"));
            self.paths.borrow_mut().push(path.into());
            let today = time::OffsetDateTime::now_utc().date().to_string();
            let response = if path.contains("&page_token=page-two") {
                json!({"corporate_actions":{"cash_dividends":[{"id":"5bf09d38-114c-4db7-84b0-b07f25ff9002","symbol":"MSFT","cusip":null,"isin":null,"process_date":today,"ex_date":null,"rate":0.1234567890123456789,"special":false,"foreign":false}]},"next_page_token":null})
            } else {
                json!({"corporate_actions":{"name_changes":[{"id":"5bf09d38-114c-4db7-84b0-b07f25ff9001","old_symbol":"AAPL","new_symbol":"AAPL","old_cusip":"037833100","new_cusip":"037833100","process_date":today}]},"next_page_token":"page-two"})
            };
            let mut bytes = serde_json::to_vec(&response).unwrap();
            // Preserve an independently supplied exact lexical provider decimal.
            if path.contains("&page_token=page-two") {
                let text = String::from_utf8(bytes).unwrap();
                bytes = text
                    .replace("0.12345678901234568", "0.1234567890123456789")
                    .into_bytes();
            }
            Ok(bytes)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    assert_eq!(
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace})
        )["ok"],
        true
    );
    let source = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.actions.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let http = EventsHttp {
        paths: RefCell::new(Vec::new()),
    };
    let refreshed = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"known-events","schemaVersion":1,"command":"data.actions.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(
        refreshed["ok"], true,
        "Known-event refresh is not implemented: {refreshed}"
    );
    assert_eq!(refreshed["data"]["status"], "AVAILABLE", "{refreshed}");
    let evidence = &refreshed["data"]["evidence"];
    assert_eq!(evidence["queryComplete"], true);
    assert_eq!(evidence["actions"].as_array().unwrap().len(), 2);
    let name = evidence["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["category"] == "NAME_CHANGE")
        .unwrap();
    assert!(name["effectiveAt"].is_null());
    assert!(name["effectiveDate"].is_null());
    assert!(name["announcedAt"].is_null());
    let dividend = evidence["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["category"] == "CASH_DIVIDEND")
        .unwrap();
    assert_eq!(dividend["terms"][0]["value"], "0.1234567890123456789");
    assert_eq!(dividend["partial"], true);
    assert!(evidence["providerObservedAt"].is_null());
    for (capability, status) in [
        ("KNOWN_ACTIONS", "AVAILABLE"),
        ("ACTION_QUERY_COMPLETION", "AVAILABLE"),
        ("COMPLETE_ACTION_COVERAGE", "BLOCKED_EXTERNAL"),
        ("HISTORICAL_ADJUSTMENT", "UNVERIFIED"),
    ] {
        assert_eq!(
            refreshed["data"]["capabilityStatuses"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["capability"] == capability)
                .unwrap()["status"],
            status
        );
    }
    assert_eq!(http.paths.borrow().len(), 2);
    let projected = command(
        &mut control.lock().unwrap(),
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        projected["data"]["observedAt"],
        refreshed["data"]["observedAt"]
    );
    assert_eq!(projected["data"]["evidence"], *evidence);
    let retained = command(
        &mut control.lock().unwrap(),
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(retained["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(retained["data"]["health"]["arming"], "NOT_APPLICABLE");
    assert!(!refreshed.to_string().contains(provider_fixtures::SECRET));
}

#[test]
fn every_official_company_event_category_is_preserved_without_legacy_reclassification() {
    use reqwest::header::HeaderMap;
    use std::sync::{Arc, Mutex};
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    let today = time::OffsetDateTime::now_utc().date().to_string();
    // Cases are the current first-party REST taxonomy, not the implementation's table.
    let cases = [
        (
            "forward_splits",
            "FORWARD_SPLIT",
            json!({"symbol":"AAPL","cusip":"037833100","new_rate":4,"old_rate":1,"ex_date":today}),
        ),
        (
            "reverse_splits",
            "REVERSE_SPLIT",
            json!({"symbol":"AAPL","old_cusip":"037833100","new_cusip":"037833100","new_rate":1,"old_rate":4,"ex_date":today}),
        ),
        (
            "unit_splits",
            "UNIT_SPLIT",
            json!({"old_symbol":"AAPL","old_cusip":"037833100","old_rate":1,"new_symbol":"AAPL","new_cusip":"037833100","new_rate":1,"alternate_symbol":"OTHER","alternate_cusip":"594918104","alternate_rate":2,"effective_date":today}),
        ),
        (
            "cash_dividends",
            "CASH_DIVIDEND",
            json!({"symbol":"AAPL","cusip":"037833100","rate":0.25,"special":false,"foreign":false,"ex_date":today}),
        ),
        (
            "stock_dividends",
            "STOCK_DIVIDEND",
            json!({"symbol":"AAPL","cusip":"037833100","rate":0.25,"ex_date":today}),
        ),
        (
            "spin_offs",
            "SPIN_OFF",
            json!({"source_symbol":"AAPL","source_cusip":"037833100","source_rate":1,"new_symbol":"OTHER","new_cusip":"594918104","new_rate":2,"ex_date":today}),
        ),
        (
            "cash_mergers",
            "CASH_MERGER",
            json!({"acquiree_symbol":"AAPL","acquiree_cusip":"037833100","rate":5,"effective_date":today}),
        ),
        (
            "stock_mergers",
            "STOCK_MERGER",
            json!({"acquiree_symbol":"AAPL","acquiree_cusip":"037833100","acquiree_rate":1,"acquirer_symbol":"OTHER","acquirer_cusip":"594918104","acquirer_rate":2,"effective_date":today}),
        ),
        (
            "stock_and_cash_mergers",
            "STOCK_AND_CASH_MERGER",
            json!({"acquiree_symbol":"AAPL","acquiree_cusip":"037833100","acquiree_rate":1,"acquirer_symbol":"OTHER","acquirer_cusip":"594918104","acquirer_rate":2,"cash_rate":5,"effective_date":today}),
        ),
        (
            "redemptions",
            "REDEMPTION",
            json!({"symbol":"AAPL","cusip":"037833100","rate":1}),
        ),
        (
            "name_changes",
            "NAME_CHANGE",
            json!({"old_symbol":"AAPL","old_cusip":"037833100","new_symbol":"OTHER","new_cusip":"594918104"}),
        ),
        (
            "worthless_removals",
            "WORTHLESS_REMOVAL",
            json!({"symbol":"AAPL","cusip":"037833100"}),
        ),
        (
            "rights_distributions",
            "RIGHTS_DISTRIBUTION",
            json!({"source_symbol":"AAPL","source_cusip":"037833100","new_symbol":"OTHER","new_cusip":"594918104","rate":2,"ex_date":today,"payable_date":today}),
        ),
        (
            "partial_calls",
            "PARTIAL_CALL",
            json!({"symbol":"AAPL","price":100,"lottery_type":"RANDOM","lottery_date":today,"results_publication_date":today,"dividend_rate":0.25}),
        ),
        (
            "reorganizations",
            "REORGANIZATION",
            json!({"symbol":"AAPL","cusip":"037833100","effective_date":today,"cash_rate":0.25,"stock_movements":[{"symbol":"OTHER","cusip":"594918104","new_rate":2,"source_rate":1}]}),
        ),
        (
            "capital_gains_distributions",
            "CAPITAL_GAINS_DISTRIBUTION",
            json!({"symbol":"AAPL","cusip":"037833100","ex_date":today,"long_term_rate":0.25,"short_term_rate":0.125}),
        ),
    ];
    let mut groups = serde_json::Map::new();
    for (index, (group, _, fields)) in cases.iter().enumerate() {
        let mut event = fields.clone();
        event["id"] = json!(format!(
            "5bf09d38-114c-4db7-84b0-b07f25ff{:04}",
            index + 100
        ));
        event["process_date"] = json!(today);
        groups.insert((*group).into(), json!([event]));
    }
    struct AllEvents(Vec<u8>);
    impl ProviderHttp for AllEvents {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            _path: &str,
            _headers: HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            assert_eq!(endpoint, ProviderEndpoint::AlpacaMarketData);
            Ok(self.0.clone())
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.actions.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    let http = AllEvents(
        serde_json::to_vec(&json!({"corporate_actions":groups,"next_page_token":null})).unwrap(),
    );
    let control = Arc::new(Mutex::new(control));
    let observed = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"all-events","schemaVersion":1,"command":"data.actions.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(
        observed["data"]["status"], "AVAILABLE",
        "Official categories were discarded or reclassified: {observed}"
    );
    let rows = observed["data"]["evidence"]["actions"].as_array().unwrap();
    assert_eq!(rows.len(), 16);
    for (_, category, _) in cases {
        let row = rows.iter().find(|row| row["category"] == category).unwrap();
        assert_eq!(row["partial"], false, "{row}");
        assert_eq!(row["instrumentIds"], json!(["equity:US:AAPL"]));
        if category == "REORGANIZATION" {
            assert_eq!(row["stockMovements"][0]["newRate"], "2");
            assert_eq!(row["stockMovements"][0]["sourceRate"], "1");
        }
        if category == "PARTIAL_CALL" {
            assert!(
                row["dates"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value["name"] == "lottery_date")
            );
        }
    }
    assert!(
        !rows
            .iter()
            .any(|row| row["category"] == "DELISTING" || row["category"] == "SYMBOL_CHANGE")
    );
}

#[test]
fn malformed_or_ambiguous_query_completion_retires_the_previous_snapshot() {
    use reqwest::header::HeaderMap;
    use std::{
        cell::RefCell,
        sync::{Arc, Mutex},
    };
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    struct Response(RefCell<Vec<u8>>);
    impl ProviderHttp for Response {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            _path: &str,
            _headers: HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            assert_eq!(endpoint, ProviderEndpoint::AlpacaMarketData);
            Ok(self.0.borrow().clone())
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.actions.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let request = json!({"requestId":"query-completeness","schemaVersion":1,"command":"data.actions.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
    let http = Response(RefCell::new(Vec::new()));
    for malformed in [
        r#"{"corporate_actions":{}}"#,
        r#"{"corporate_actions":{"name_changes":[],"name_changes":[]},"next_page_token":null}"#,
        r#"{"corporate_actions":{},"next_page_token":null,"next_page_token":null}"#,
        r#"{"corporate_actions":{},"next_page_token":42}"#,
        r#"{"corporate_actions":{"unpublished_category":[]},"next_page_token":null}"#,
    ] {
        *http.0.borrow_mut() = br#"{"corporate_actions":{},"next_page_token":null}"#.to_vec();
        let valid =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(valid["data"]["status"], "AVAILABLE", "{valid}");
        assert_eq!(valid["data"]["evidence"]["actions"], json!([]));
        assert_eq!(
            valid["data"]["capabilityStatuses"][2]["status"],
            "BLOCKED_EXTERNAL"
        );
        *http.0.borrow_mut() = malformed.as_bytes().to_vec();
        let invalid =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(
            invalid["data"]["status"], "UNAVAILABLE",
            "Ambiguous completion was accepted: {malformed}: {invalid}"
        );
        assert!(invalid["data"]["evidence"].is_null());
        assert!(invalid["data"]["observedAt"].is_null());
    }
}

#[test]
fn provider_exponent_terms_are_exact_and_invalid_ratios_retire_the_query() {
    use reqwest::header::HeaderMap;
    use std::{
        cell::RefCell,
        sync::{Arc, Mutex},
    };
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    struct Response(RefCell<Vec<u8>>);
    impl ProviderHttp for Response {
        fn get(
            &self,
            _endpoint: ProviderEndpoint,
            _path: &str,
            _headers: HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            Ok(self.0.borrow().clone())
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.actions.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let request = json!({"requestId":"exact-terms","schemaVersion":1,"command":"data.actions.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
    let today = time::OffsetDateTime::now_utc().date().to_string();
    let http = Response(RefCell::new(Vec::new()));
    for (raw, expected) in [
        ("2.5e-6", "0.0000025"),
        ("1e+20", "100000000000000000000"),
        ("-0.25", "-0.25"),
    ] {
        *http.0.borrow_mut()=format!(r#"{{"corporate_actions":{{"cash_dividends":[{{"id":"5bf09d38-114c-4db7-84b0-b07f25ff9010","symbol":"AAPL","cusip":"037833100","process_date":"{today}","ex_date":"{today}","special":false,"foreign":false,"rate":{raw}}}]}},"next_page_token":null}}"#).into_bytes();
        let observed =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(
            observed["data"]["status"], "AVAILABLE",
            "A valid exact provider number was lost: {raw}: {observed}"
        );
        assert_eq!(
            observed["data"]["evidence"]["actions"][0]["terms"][0]["value"],
            expected
        );
    }
    for raw in ["0", "-1", "1e100000"] {
        *http.0.borrow_mut()=format!(r#"{{"corporate_actions":{{"forward_splits":[{{"id":"5bf09d38-114c-4db7-84b0-b07f25ff9011","symbol":"AAPL","cusip":"037833100","process_date":"{today}","ex_date":"{today}","old_rate":1,"new_rate":{raw}}}]}},"next_page_token":null}}"#).into_bytes();
        let observed =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(
            observed["data"]["status"], "UNAVAILABLE",
            "Invalid ratio accepted: {raw}: {observed}"
        );
        assert!(observed["data"]["evidence"].is_null());
    }
}

#[test]
fn exact_live_account_metadata_is_readable_without_current_tradability_authority() {
    use reqwest::header::HeaderMap;
    use std::{
        cell::RefCell,
        sync::{Arc, Mutex},
    };
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    struct MetadataHttp {
        paths: RefCell<Vec<String>>,
        account: provider_fixtures::Http,
    }
    impl ProviderHttp for MetadataHttp {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            assert_eq!(endpoint, ProviderEndpoint::Trading212Live);
            assert_eq!(
                headers["Authorization"],
                "Basic UzAyLUZBS0UtS0VZLTU5NDc5MTQ1MzpTMDItRkFLRS1TRUNSRVQtNzA0NTU2OTIx"
            );
            assert!(headers["Authorization"].is_sensitive());
            assert!(!headers.contains_key("APCA-API-KEY-ID"));
            self.paths.borrow_mut().push(path.into());
            match path {
                "/api/v0/equity/account/summary"=>self.account.get(endpoint,path,headers),
                "/api/v0/equity/metadata/instruments"=>Ok(serde_json::to_vec(&json!([
                    {"ticker":"AAPL_US_EQ","type":"STOCK","isin":"US0378331005","currencyCode":"USD","name":"Apple","shortName":"Apple","workingScheduleId":101,"maxOpenQuantity":0,"extendedHours":false},
                    {"ticker":"MSFT_US_EQ","type":"STOCK","isin":"US5949181045","currencyCode":"USD","name":"Microsoft","shortName":"Microsoft","workingScheduleId":101,"maxOpenQuantity":10,"extendedHours":true}
                ])).unwrap()),
                "/api/v0/equity/metadata/exchanges"=>Ok(serde_json::to_vec(&json!([
                    {"id":1,"name":"NASDAQ","workingSchedules":[{"id":101,"timeEvents":[{"date":"2026-10-05T13:30:00Z","type":"OPEN"},{"date":"2026-10-05T20:00:00Z","type":"CLOSE"}]}]}
                ])).unwrap()),
                _=>panic!("A metadata source attempted an unapproved route: {path}"),
            }
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "trading212", "LIVE", &vault);
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.instrument.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.instrument.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["environment"], "LIVE");
    let control = Arc::new(Mutex::new(control));
    let http = MetadataHttp {
        paths: RefCell::new(Vec::new()),
        account: provider_fixtures::Http::default(),
    };
    std::thread::sleep(std::time::Duration::from_millis(5100));
    let read = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"broker-metadata","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(
        read["ok"], true,
        "Broker metadata source is unavailable: {read}"
    );
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    assert_eq!(
        *http.paths.borrow(),
        vec![
            "/api/v0/equity/account/summary",
            "/api/v0/equity/metadata/instruments",
            "/api/v0/equity/metadata/exchanges"
        ]
    );
    let evidence = &read["data"]["evidence"];
    assert_eq!(evidence["kind"], "BROKER_INSTRUMENTS");
    assert_eq!(evidence["providerQuality"], "TEN_MINUTE_METADATA");
    assert_eq!(evidence["accountCurrency"], "GBP");
    assert_eq!(evidence["binding"]["connectionId"], account["connectionId"]);
    assert_eq!(
        evidence["binding"]["accountVersion"],
        account["stateVersion"]
    );
    assert_eq!(evidence["instruments"].as_array().unwrap().len(), 2);
    let apple = &evidence["instruments"][0];
    assert_eq!(apple["providerSymbol"], "AAPL_US_EQ");
    assert_eq!(apple["isin"], "US0378331005");
    assert_eq!(apple["maxOpenQuantity"], "0");
    assert!(apple["providerObservedAt"].is_null());
    assert!(
        apple["venue"].is_null(),
        "An exchange display name must not invent MIC/routing evidence."
    );
    assert!(
        apple["tradable"].is_null(),
        "A metadata member or quantity cap must not invent current tradability."
    );
    for (name, status) in [
        ("BROKER_ACCOUNT_IDENTITY", "AVAILABLE"),
        ("BROKER_INSTRUMENT_METADATA", "AVAILABLE"),
        ("EXCHANGE_HALTS", "BLOCKED_EXTERNAL"),
        ("ACCOUNT_TRADABILITY", "BLOCKED_EXTERNAL"),
    ] {
        assert_eq!(
            read["data"]["capabilityStatuses"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["capability"] == name)
                .unwrap()["status"],
            status
        );
    }
    let retained = command(
        &mut control.lock().unwrap(),
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(retained["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(retained["data"]["health"]["arming"], "DISARMED");
    assert!(!read.to_string().contains(provider_fixtures::SECRET));
}

struct BrokerResponses {
    account: Value,
    instruments: Value,
    exchanges: Value,
    paths: std::cell::RefCell<Vec<String>>,
}

#[test]
fn ordinary_account_summary_holds_metadata_quota_before_the_first_metadata_read() {
    use std::{
        cell::RefCell,
        sync::{Arc, Mutex},
    };
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    struct AccountRead<'a> {
        account: provider_fixtures::Http,
        during_summary: RefCell<Box<dyn FnMut() + 'a>>,
    }
    impl ProviderHttp for AccountRead<'_> {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            if path == "/api/v0/equity/account/summary" {
                // A slow response must not let the admission-time interval expire in flight.
                std::thread::sleep(std::time::Duration::from_millis(6100));
                (self.during_summary.borrow_mut())();
            }
            self.account.get(endpoint, path, headers)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account_http = provider_fixtures::Http::default();
    let identity = 9007199254741800u64;
    account_http.trading212_identity.set(identity);
    let account = connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &account_http,
    );
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.instrument.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.instrument.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    std::thread::sleep(std::time::Duration::from_millis(5100));
    let control = Arc::new(Mutex::new(control));
    let request = json!({"requestId":"ordinary-summary","schemaVersion":1,"command":"account.refresh","payload":{"workspaceId":workspace,"connectionId":account["connectionId"],"expectedStateVersion":account["stateVersion"]}});
    let job = control
        .lock()
        .unwrap()
        .prepare_provider(&request)
        .unwrap()
        .unwrap();
    let mut metadata = BrokerResponses::default();
    metadata.account["id"] = json!(identity);
    let refresh = json!({"requestId":"metadata-during-summary","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
    let http = AccountRead {
        account: account_http,
        during_summary: RefCell::new(Box::new(|| {
            let read = tradex::financial_sources::execute_refresh(
                &control, &refresh, "main", &vault, &metadata,
            );
            assert_eq!(
                read["data"]["status"], "UNAVAILABLE",
                "An ordinary summary must hold the shared quota while in flight: {read}"
            );
            assert!(
                metadata.paths.borrow().is_empty(),
                "Metadata must not issue a second summary request"
            );
        })),
    };
    let result = job.run(
        &vault,
        |_| provider_fixtures::credentials(),
        &http,
        || control.lock().unwrap().provider_job_current(&job),
    );
    let completed = control.lock().unwrap().complete_provider(&job, result);
    assert_eq!(completed["ok"], true, "{completed}");
    let read =
        tradex::financial_sources::execute_refresh(&control, &refresh, "main", &vault, &metadata);
    assert_eq!(
        read["data"]["status"], "UNAVAILABLE",
        "Completion must start a fresh summary interval: {read}"
    );
    assert!(metadata.paths.borrow().is_empty());
}
impl Default for BrokerResponses {
    fn default() -> Self {
        Self {
            account: json!({"id":9007199254740993u64,"currency":"GBP"}),
            instruments: json!([
                {"ticker":"AAPL_US_EQ","type":"STOCK","isin":"US0378331005","currencyCode":"USD","name":"Apple","workingScheduleId":101},
                {"ticker":"MSFT_US_EQ","type":"STOCK","isin":"US5949181045","currencyCode":"USD","name":"Microsoft","workingScheduleId":101}
            ]),
            exchanges: json!([{ "id":1,"name":"NASDAQ","workingSchedules":[{"id":101,"timeEvents":[{"date":"2026-10-05T13:30:00Z","type":"OPEN"}]}]}]),
            paths: Default::default(),
        }
    }
}

#[test]
fn ordinary_summary_completion_preserves_exhausted_headers_and_transport_cooldown() {
    use std::sync::{Arc, Mutex};
    use tradex::provider_io::{
        ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse, ProviderRateLimit,
    };
    struct AccountRead {
        account: provider_fixtures::Http,
        mode: &'static str,
    }
    impl ProviderHttp for AccountRead {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            self.account.get(endpoint, path, headers)
        }
        fn request_with_rate_limit(
            &self,
            endpoint: ProviderEndpoint,
            method: ProviderHttpMethod,
            path: &str,
            headers: reqwest::header::HeaderMap,
            body: Option<&Value>,
        ) -> tradex::protocol::Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
            assert_eq!(method, ProviderHttpMethod::Get);
            assert!(body.is_none());
            assert_eq!(path, "/api/v0/equity/account/summary");
            if self.mode == "transport" {
                std::thread::sleep(std::time::Duration::from_millis(6100));
                return Err(tradex::protocol::TradeXError::new("PROVIDER_UNAVAILABLE"));
            }
            let reset = (time::OffsetDateTime::now_utc() + time::Duration::seconds(30))
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap();
            Ok((
                ProviderHttpResponse {
                    status: 200,
                    body: self.get(endpoint, path, headers)?,
                },
                Some(ProviderRateLimit {
                    remaining: Some(0),
                    retry_after_seconds: (self.mode == "retry").then_some(30),
                    reset_at: (self.mode == "reset").then_some(reset),
                }),
            ))
        }
    }
    for (index, mode) in ["retry", "reset", "transport"].into_iter().enumerate() {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = provider_fixtures::Vault::default();
        let account_http = provider_fixtures::Http::default();
        let identity = 9007199254741810u64 + index as u64;
        account_http.trading212_identity.set(identity);
        let account = connect_using(
            &mut control,
            &workspace,
            "trading212",
            "LIVE",
            &vault,
            &account_http,
        );
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let source = command(
            &mut control,
            "data.instrument.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.instrument.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
        );
        assert_eq!(saved["ok"], true, "{saved}");
        std::thread::sleep(std::time::Duration::from_millis(5100));
        let control = Arc::new(Mutex::new(control));
        let job = control.lock().unwrap().prepare_provider(&json!({"requestId":"quota-headers","schemaVersion":1,"command":"account.refresh","payload":{"workspaceId":workspace,"connectionId":account["connectionId"],"expectedStateVersion":account["stateVersion"]}})).unwrap().unwrap();
        let http = AccountRead {
            account: account_http,
            mode,
        };
        let result = job.run(
            &vault,
            |_| provider_fixtures::credentials(),
            &http,
            || control.lock().unwrap().provider_job_current(&job),
        );
        if mode != "transport" {
            assert_eq!(
                control.lock().unwrap().complete_provider(&job, result)["ok"],
                true
            );
            std::thread::sleep(std::time::Duration::from_millis(5100));
        }
        let mut metadata = BrokerResponses::default();
        metadata.account["id"] = json!(identity);
        let refresh = json!({"requestId":"completion-quota","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
        let read = tradex::financial_sources::execute_refresh(
            &control, &refresh, "main", &vault, &metadata,
        );
        assert_eq!(
            read["data"]["status"], "UNAVAILABLE",
            "{mode} must preserve its completion quota: {read}"
        );
        assert!(
            metadata.paths.borrow().is_empty(),
            "{mode} must block before HTTP"
        );
        if mode == "transport" {
            std::thread::sleep(std::time::Duration::from_millis(5100));
            let read = tradex::financial_sources::execute_refresh(
                &control, &refresh, "main", &vault, &metadata,
            );
            assert_eq!(
                read["data"]["status"], "AVAILABLE",
                "A finite transport cooldown must reopen: {read}"
            );
        }
    }
}
impl tradex::provider_io::ProviderHttp for BrokerResponses {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        assert_eq!(
            endpoint,
            tradex::provider_io::ProviderEndpoint::Trading212Live
        );
        assert!(headers["Authorization"].is_sensitive());
        self.paths.borrow_mut().push(path.into());
        Ok(serde_json::to_vec(match path {
            "/api/v0/equity/account/summary" => &self.account,
            "/api/v0/equity/metadata/instruments" => &self.instruments,
            "/api/v0/equity/metadata/exchanges" => &self.exchanges,
            _ => panic!("Unexpected metadata read: {path}"),
        })
        .unwrap())
    }
}
#[test]
fn malformed_broker_identity_or_schedule_is_unavailable_without_panicking() {
    let directory = tempfile::tempdir().unwrap();
    let vault = provider_fixtures::Vault::default();
    let mut bad = Vec::new();
    let mut unicode = BrokerResponses::default();
    unicode.instruments[0]["isin"] = json!("€037833100");
    bad.push(unicode);
    let mut wrong_id = BrokerResponses::default();
    wrong_id.account["id"] = json!(42);
    bad.push(wrong_id);
    let mut wrong_currency = BrokerResponses::default();
    wrong_currency.account["currency"] = json!("USD");
    bad.push(wrong_currency);
    let mut checksum = BrokerResponses::default();
    checksum.instruments[0]["isin"] = json!("US0378331006");
    bad.push(checksum);
    let mut duplicate_ticker = BrokerResponses::default();
    duplicate_ticker.instruments[1]["ticker"] = json!("AAPL_US_EQ");
    bad.push(duplicate_ticker);
    let mut duplicate_isin = BrokerResponses::default();
    duplicate_isin.instruments[1]["isin"] = json!("US0378331005");
    bad.push(duplicate_isin);
    let mut wrong_type = BrokerResponses::default();
    wrong_type.instruments[0]["type"] = json!("ETF");
    bad.push(wrong_type);
    let mut missing_schedule = BrokerResponses::default();
    missing_schedule.instruments[0]["workingScheduleId"] = json!(102);
    bad.push(missing_schedule);
    let mut ambiguous_schedule = BrokerResponses::default();
    ambiguous_schedule.exchanges[0]["workingSchedules"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":101,"timeEvents":[]}));
    bad.push(ambiguous_schedule);
    let mut bad_event = BrokerResponses::default();
    bad_event.exchanges[0]["workingSchedules"][0]["timeEvents"][0]["type"] = json!("TRADABLE");
    bad.push(bad_event);
    for event_types in [
        vec!["OPEN", "OPEN", "CLOSE"],
        vec!["OPEN", "CLOSE", "CLOSE"],
        vec!["OPEN", "BREAK_END", "CLOSE"],
        vec!["CLOSE", "BREAK_START"],
        vec!["OPEN", "BREAK_START", "BREAK_START", "BREAK_END", "CLOSE"],
        vec!["PRE_MARKET_OPEN", "PRE_MARKET_OPEN", "OPEN", "CLOSE"],
        vec!["PRE_MARKET_OPEN", "BREAK_END"],
        vec!["OVERNIGHT_OPEN", "BREAK_START"],
        vec!["AFTER_HOURS_CLOSE", "CLOSE"],
        vec!["OPEN", "BREAK_START", "AFTER_HOURS_OPEN"],
        vec!["OPEN", "AFTER_HOURS_OPEN", "AFTER_HOURS_OPEN"],
        vec!["OPEN", "AFTER_HOURS_OPEN", "BREAK_START"],
    ] {
        let mut conflicting = BrokerResponses::default();
        conflicting.exchanges[0]["workingSchedules"][0]["timeEvents"] = json!(event_types.into_iter().enumerate().map(|(minute, kind)| json!({"date":format!("2026-10-05T13:{:02}:00Z",30+minute),"type":kind})).collect::<Vec<_>>());
        bad.push(conflicting);
    }
    let mut reflected_secret = BrokerResponses::default();
    reflected_secret.instruments[0]["name"] = json!(provider_fixtures::SECRET);
    bad.push(reflected_secret);
    let mut cases = Vec::new();
    for (index, mut response) in bad.into_iter().enumerate() {
        let identity = 9007199254741293u64 + index as u64;
        let account_http = provider_fixtures::Http::default();
        account_http.trading212_identity.set(identity);
        let mut control = ControlPlane::new(directory.path().join(format!("case-{index}")));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let account = connect_using(
            &mut control,
            &workspace,
            "trading212",
            "LIVE",
            &vault,
            &account_http,
        );
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let source = command(
            &mut control,
            "data.instrument.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.instrument.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
        );
        let request = json!({"requestId":"broker-invalid","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
        if response.account["id"] == 9007199254740993u64 {
            response.account["id"] = json!(identity);
        }
        cases.push((
            std::sync::Arc::new(std::sync::Mutex::new(control)),
            request,
            response,
            identity,
        ));
    }
    std::thread::sleep(std::time::Duration::from_millis(5100));
    for (control, request, response, identity) in cases {
        let failed = tradex::financial_sources::execute_refresh(
            &control, &request, "main", &vault, &response,
        );
        assert_eq!(failed["ok"], true, "{failed}");
        assert!(
            !failed.to_string().contains(provider_fixtures::SECRET),
            "A provider echo must not expose the credential"
        );
        assert_eq!(failed["data"]["status"], "UNAVAILABLE", "{failed}");
        assert!(failed["data"]["evidence"].is_null(), "{failed}");
        if response.account["id"] != identity || response.account["currency"] != "GBP" {
            assert_eq!(
                response.paths.borrow().len(),
                1,
                "Identity must be checked before directory reads"
            );
        } else if response.instruments[0]["name"] == provider_fixtures::SECRET {
            assert_eq!(
                response.paths.borrow().len(),
                2,
                "Reject an echoed secret before another read"
            );
        } else {
            assert_eq!(
                response.paths.borrow().len(),
                3,
                "The malformed metadata must actually be consumed, not hidden behind quota failure"
            );
        }
    }
}

#[test]
fn coherent_broker_schedule_keeps_all_event_types_and_truncated_window_edges() {
    let directory = tempfile::tempdir().unwrap();
    let vault = provider_fixtures::Vault::default();
    let sequences = [
        vec![
            "CLOSE",
            "AFTER_HOURS_OPEN",
            "AFTER_HOURS_CLOSE",
            "OVERNIGHT_OPEN",
            "PRE_MARKET_OPEN",
            "OPEN",
            "BREAK_START",
            "BREAK_END",
            "CLOSE",
            "AFTER_HOURS_CLOSE",
            "PRE_MARKET_OPEN",
            "OPEN",
        ],
        vec!["BREAK_END", "CLOSE", "OPEN"],
        vec!["BREAK_START", "BREAK_END", "CLOSE", "OPEN"],
    ];
    let mut cases = Vec::new();
    for (index, sequence) in sequences.into_iter().enumerate() {
        let mut control = ControlPlane::new(directory.path().join(format!("schedule-{index}")));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let account_http = provider_fixtures::Http::default();
        let identity = 9007199254741900u64 + index as u64;
        account_http.trading212_identity.set(identity);
        let account = connect_using(
            &mut control,
            &workspace,
            "trading212",
            "LIVE",
            &vault,
            &account_http,
        );
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let source = command(
            &mut control,
            "data.instrument.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.instrument.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
        );
        assert_eq!(saved["ok"], true, "{saved}");
        let mut metadata = BrokerResponses::default();
        metadata.account["id"] = json!(identity);
        // Provider array order does not establish event chronology.
        let events: Vec<_> = sequence.iter().enumerate().rev().map(|(minute, kind)| json!({"date":format!("2026-10-05T13:{:02}:00Z",30+minute),"type":kind})).collect();
        metadata.exchanges[0]["workingSchedules"][0]["timeEvents"] = json!(events);
        let refresh = json!({"requestId":"schedule-window","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
        cases.push((
            std::sync::Arc::new(std::sync::Mutex::new(control)),
            refresh,
            metadata,
            sequence,
        ));
    }
    std::thread::sleep(std::time::Duration::from_millis(5100));
    for (control, refresh, metadata, sequence) in cases {
        let read = tradex::financial_sources::execute_refresh(
            &control, &refresh, "main", &vault, &metadata,
        );
        assert_eq!(
            read["data"]["status"], "AVAILABLE",
            "A coherent bounded schedule must be retained: {read}"
        );
        let events = read["data"]["evidence"]["instruments"][0]["scheduleEvents"]
            .as_array()
            .unwrap();
        assert_eq!(
            events
                .iter()
                .map(|event| event["eventType"].as_str().unwrap())
                .collect::<Vec<_>>(),
            sequence
        );
        assert_eq!(
            read["data"]["evidence"]["instruments"][0]["canonicalSecurityIdentity"],
            "UNVERIFIED"
        );
    }
}

#[test]
fn after_hours_open_transitions_from_regular_hours_without_fabricating_a_close() {
    let directory = tempfile::tempdir().unwrap();
    let vault = provider_fixtures::Vault::default();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account_http = provider_fixtures::Http::default();
    let identity = 9007199254741999u64;
    account_http.trading212_identity.set(identity);
    let account = connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &account_http,
    );
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.instrument.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.instrument.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let mut metadata = BrokerResponses::default();
    metadata.account["id"] = json!(identity);
    // Production metadata uses an after-hours opening as the phase transition,
    // without a separate CLOSE. These are external fixture dates, not venue authority.
    let events = json!([
        {"date":"2026-10-05T08:00:00Z","type":"PRE_MARKET_OPEN"},
        {"date":"2026-10-05T13:30:00Z","type":"OPEN"},
        {"date":"2026-10-05T20:00:00Z","type":"AFTER_HOURS_OPEN"},
        {"date":"2026-10-06T00:00:00Z","type":"AFTER_HOURS_CLOSE"},
        {"date":"2026-10-06T00:01:00Z","type":"OVERNIGHT_OPEN"},
        {"date":"2026-10-06T08:00:00Z","type":"PRE_MARKET_OPEN"},
        {"date":"2026-10-06T13:30:00Z","type":"OPEN"},
        {"date":"2026-10-06T20:00:00Z","type":"AFTER_HOURS_OPEN"}
    ]);
    metadata.exchanges[0]["workingSchedules"][0]["timeEvents"] = events.clone();
    let control = std::sync::Arc::new(std::sync::Mutex::new(control));
    std::thread::sleep(std::time::Duration::from_millis(5100));
    let read = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"after-hours-phase","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &metadata,
    );
    assert_eq!(
        read["data"]["status"], "AVAILABLE",
        "Actual regular-to-after-hours transitions must remain usable as metadata: {read}"
    );
    let evidence = &read["data"]["evidence"];
    assert_eq!(evidence["providerQuality"], "TEN_MINUTE_METADATA");
    let returned = &evidence["instruments"][0]["scheduleEvents"];
    assert_eq!(
        returned.as_array().unwrap().len(),
        8,
        "Do not fabricate a CLOSE"
    );
    assert_eq!(returned[2]["eventType"], "AFTER_HOURS_OPEN");
    assert_eq!(returned[2]["date"], "2026-10-05T20:00:00Z");
    assert_eq!(
        evidence["instruments"][0]["canonicalSecurityIdentity"],
        "UNVERIFIED"
    );
    assert!(evidence["instruments"][0]["tradable"].is_null());
    let account_now = command(
        &mut control.lock().unwrap(),
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(account_now["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(account_now["data"]["health"]["arming"], "DISARMED");
}

#[test]
fn metadata_quota_is_shared_by_remote_account_and_selection_cannot_reset_it() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let identity = 9007199254741093u64;
    let account_http = provider_fixtures::Http::default();
    account_http.trading212_identity.set(identity);
    let first = connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &account_http,
    );
    let mut other_control = ControlPlane::new(directory.path().join("other-workspace"));
    let other_workspace =
        command(&mut other_control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let second = connect_using(
        &mut other_control,
        &other_workspace,
        "trading212",
        "LIVE",
        &vault,
        &account_http,
    );
    assert_ne!(first["connectionId"], second["connectionId"]);
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.instrument.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.instrument.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":first["connectionId"]}),
    );
    std::thread::sleep(std::time::Duration::from_millis(5100));
    let control = std::sync::Arc::new(std::sync::Mutex::new(control));
    let mut http = BrokerResponses::default();
    http.account["id"] = json!(identity);
    let request = |version: &Value| json!({"requestId":"broker-quota","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":version}});
    let prime = tradex::financial_sources::execute_refresh(
        &control,
        &request(&saved["data"]["stateVersion"]),
        "main",
        &vault,
        &http,
    );
    assert_eq!(prime["data"]["status"], "AVAILABLE", "{prime}");
    let repeated = tradex::financial_sources::execute_refresh(
        &control,
        &request(&saved["data"]["stateVersion"]),
        "main",
        &vault,
        &http,
    );
    assert_eq!(repeated["data"]["status"], "UNAVAILABLE", "{repeated}");
    assert!(repeated["data"]["evidence"].is_null());
    assert_eq!(
        http.paths.borrow().len(),
        3,
        "An immediate refresh must not spend another endpoint request"
    );
    let swapped = command(
        &mut control.lock().unwrap(),
        "data.instrument.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"],"connectionId":first["connectionId"]}),
    );
    let retry = tradex::financial_sources::execute_refresh(
        &control,
        &request(&swapped["data"]["stateVersion"]),
        "main",
        &vault,
        &http,
    );
    assert_eq!(retry["data"]["status"], "UNAVAILABLE", "{retry}");
    assert_eq!(
        http.paths.borrow().len(),
        3,
        "Saving a selection cannot reset quota"
    );
    command(
        &mut other_control,
        "time.revalidate",
        json!({"workspaceId":other_workspace}),
    );
    let other_source = command(
        &mut other_control,
        "data.instrument.connection",
        json!({"workspaceId":other_workspace}),
    );
    let other_saved = command(
        &mut other_control,
        "data.instrument.configure",
        json!({"workspaceId":other_workspace,"expectedStateVersion":other_source["data"]["stateVersion"],"connectionId":second["connectionId"]}),
    );
    let other_request = json!({"requestId":"broker-quota-other","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":other_workspace,"expectedStateVersion":other_saved["data"]["stateVersion"]}});
    let other_control = std::sync::Arc::new(std::sync::Mutex::new(other_control));
    let other_retry = tradex::financial_sources::execute_refresh(
        &other_control,
        &other_request,
        "main",
        &vault,
        &http,
    );
    assert_eq!(
        other_retry["data"]["status"], "UNAVAILABLE",
        "{other_retry}"
    );
    assert_eq!(
        http.paths.borrow().len(),
        3,
        "A different workspace/credential for the same remote account cannot reset quota"
    );
}

struct ActionScenario {
    _directory: tempfile::TempDir,
    control: std::sync::Arc<std::sync::Mutex<ControlPlane>>,
    workspace: Value,
    source: Value,
    account: Value,
    vault: provider_fixtures::Vault,
    process_date: String,
}
impl ActionScenario {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = provider_fixtures::Vault::default();
        let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
        let clock = command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let process_date = clock["data"]["wallClock"].as_str().unwrap()[..10].to_owned();
        let source = command(
            &mut control,
            "data.actions.connection",
            json!({"workspaceId":workspace}),
        );
        let source=command(&mut control,"data.actions.configure",json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}))["data"].clone();
        Self {
            _directory: directory,
            control: std::sync::Arc::new(std::sync::Mutex::new(control)),
            workspace,
            source,
            account,
            vault,
            process_date,
        }
    }
    fn request(&self) -> Value {
        json!({"requestId":"company-observation","schemaVersion":1,"command":"data.actions.refresh","payload":{"workspaceId":self.workspace,"expectedStateVersion":self.source["stateVersion"]}})
    }
    fn page(&self, sub_type: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"corporate_actions":{"cash_dividends":[{
            "id":"01234567-89ab-4cde-8f01-234567890123","symbol":"AAPL","cusip":"037833100","rate":"0.25","special":false,"foreign":false,"sub_type":sub_type,"ex_date":self.process_date,"process_date":self.process_date
        }]},"next_page_token":null})).unwrap()
    }
    fn refresh(&self, http: &impl tradex::provider_io::ProviderHttp) -> Value {
        tradex::financial_sources::execute_refresh(
            &self.control,
            &self.request(),
            "main",
            &self.vault,
            http,
        )
    }
}
struct ActionBody(Vec<u8>);
impl tradex::provider_io::ProviderHttp for ActionBody {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        assert_eq!(
            endpoint,
            tradex::provider_io::ProviderEndpoint::AlpacaMarketData
        );
        assert!(path.starts_with(
            "/v1/corporate-actions?symbols=AAPL,MSFT&region=us&data_quality=all&start="
        ));
        assert!(headers["APCA-API-KEY-ID"].is_sensitive());
        assert!(headers["APCA-API-SECRET-KEY"].is_sensitive());
        Ok(self.0.clone())
    }
}
#[test]
fn provider_echoed_company_event_credentials_retire_the_previous_query() {
    let scenario = ActionScenario::new();
    let prime = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
    assert_eq!(prime["data"]["status"], "AVAILABLE", "{prime}");
    let failed = scenario.refresh(&ActionBody(scenario.page(provider_fixtures::SECRET)));
    assert!(
        !failed.to_string().contains(provider_fixtures::SECRET),
        "Company event strings must not reflect credentials"
    );
    assert_eq!(failed["data"]["status"], "UNAVAILABLE", "{failed}");
    assert!(failed["data"]["evidence"].is_null());
}

#[test]
fn company_query_reaches_market_with_its_original_binding_without_adjustment_authority() {
    let scenario = ActionScenario::new();
    let read = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let market = command(
        &mut scenario.control.lock().unwrap(),
        "market.get",
        json!({"workspaceId":scenario.workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
    );
    assert_eq!(market["ok"], true, "{market}");
    assert_eq!(
        market["data"]["financialEvidence"]["companyEvents"]["evidence"], read["data"]["evidence"],
        "Markets must consume the original producer evidence, not mint a receipt or substitute a fixture"
    );
    assert_eq!(market["data"]["adjustmentStatus"], "UNAVAILABLE");
    assert!(
        market["data"]["corporateActions"]
            .as_array()
            .unwrap()
            .is_empty(),
        "Date-only events cannot be reclassified as legacy timestamped actions"
    );
    assert!(market["data"]["snapshot"].is_null());
    assert!(market["data"]["instrumentState"].is_null());
    let sources = command(
        &mut scenario.control.lock().unwrap(),
        "data.source.catalog",
        json!({"workspaceId":scenario.workspace}),
    );
    let combined = sources["data"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["sourceId"] == "OD-005")
        .unwrap();
    assert_eq!(combined["status"], "UNVERIFIED");
    let disconnected = command(
        &mut scenario.control.lock().unwrap(),
        "data.actions.disconnect",
        json!({"workspaceId":scenario.workspace,"expectedStateVersion":scenario.source["stateVersion"]}),
    );
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    let market = command(
        &mut scenario.control.lock().unwrap(),
        "market.get",
        json!({"workspaceId":scenario.workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
    );
    assert_eq!(
        market["data"]["financialEvidence"]["companyEvents"]["configured"],
        false
    );
    assert!(
        market["data"]["snapshot"].is_null(),
        "Disconnect cannot reactivate synthetic market data"
    );
    assert_eq!(market["data"]["adjustmentStatus"], "UNAVAILABLE");
}
#[test]
#[cfg(feature = "integration-test")]
fn legacy_opt_ins_cannot_replace_a_selected_or_disconnected_company_source() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "company_query_reaches_market_with_its_original_binding_without_adjustment_authority",
            "--exact",
        ])
        .env("TRADEX_MARKET_FIXTURE", "1")
        .env("TRADEX_LIVE_APPROVAL_FIXTURE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
#[ignore = "Real monotonic quota boundary; execute explicitly when changing quota accounting"]
fn metadata_quota_does_not_open_early_after_a_slow_identity_read() {
    use tradex::provider_io::ProviderHttp;
    struct SlowIdentity {
        responses: BrokerResponses,
        slow: std::cell::Cell<bool>,
    }
    impl ProviderHttp for SlowIdentity {
        fn get(
            &self,
            endpoint: tradex::provider_io::ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            if path == "/api/v0/equity/account/summary" && self.slow.replace(false) {
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
            self.responses.get(endpoint, path, headers)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let identity = 9007199254741593u64;
    let account_http = provider_fixtures::Http::default();
    account_http.trading212_identity.set(identity);
    let account = connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &account_http,
    );
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.instrument.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.instrument.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    std::thread::sleep(std::time::Duration::from_millis(5100));
    let control = std::sync::Arc::new(std::sync::Mutex::new(control));
    let mut responses = BrokerResponses::default();
    responses.account["id"] = json!(identity);
    let http = SlowIdentity {
        responses,
        slow: std::cell::Cell::new(true),
    };
    let request = json!({"requestId":"slow-metadata","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
    let read =
        tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    // The initial 50-second reservation has expired, but fewer than 50 seconds have
    // elapsed since the actual instrument request following the slow summary.
    std::thread::sleep(std::time::Duration::from_secs(46));
    let retry =
        tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
    assert_eq!(retry["data"]["status"], "UNAVAILABLE", "{retry}");
    assert_eq!(
        http.responses.paths.borrow().len(),
        3,
        "The instrument quota starts from actual endpoint work, not job preparation"
    );
    assert!(retry["data"]["evidence"].is_null());
}

#[test]
fn known_events_do_not_satisfy_independent_equity_approval_guards() {
    let scenario = ActionScenario::new();
    let read = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let mut control = scenario.control.lock().unwrap();
    let live = connect(
        &mut control,
        &scenario.workspace,
        "trading212",
        "LIVE",
        &scenario.vault,
    );
    let draft = command(
        &mut control,
        "trade.save_draft",
        json!({"workspaceId":scenario.workspace,"fields":{"accountId":live["connectionId"],"venue":"XNAS","environment":"TRADING212_LIVE","instrumentId":"equity:US:AAPL","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"1"},"limitPrice":"200","timeInForce":"DAY"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &mut control,
        "trade.generate_proposal",
        json!({"workspaceId":scenario.workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let request =
        json!({"workspaceId":scenario.workspace,"proposalId":proposal["data"]["proposalId"]});
    let review = command(&mut control, "trade.request_approval", request.clone());
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(
        review["data"]["market"]["financialEvidence"]["companyEvents"]["evidence"],
        read["data"]["evidence"]
    );
    let checks = review["data"]["riskDecision"]["checks"].as_array().unwrap();
    for name in [
        "CORPORATE_ACTION_COVERAGE",
        "HISTORICAL_ADJUSTMENT",
        "INSTRUMENT_RULES",
    ] {
        let check = checks.iter().find(|row| row["checkId"] == name);
        assert!(
            check.is_some(),
            "A required independent {name} guard is missing: {review}"
        );
        assert_eq!(check.unwrap()["outcome"], "UNAVAILABLE", "{review}");
    }
    assert_eq!(review["data"]["eligible"], false);
    let repeated = command(&mut control, "trade.request_approval", request);
    assert_eq!(
        repeated["data"]["market"]["financialEvidence"]["companyEvents"]["evidence"],
        read["data"]["evidence"],
        "Reviewing again cannot renew the producer receipt or material identity"
    );
    let market_digest = |response: &Value| {
        response["data"]["riskDecision"]["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|input| input["kind"] == "MARKET")
            .unwrap()["digest"]
            .clone()
    };
    assert_eq!(
        market_digest(&repeated),
        market_digest(&review),
        "Unchanged producer material must keep the same market decision input digest"
    );
    // Each explicit review binds a new decision id; source polling only projects evidence.
    assert_ne!(
        repeated["data"]["riskDecision"]["decisionId"],
        review["data"]["riskDecision"]["decisionId"]
    );
    let approved = command(
        &mut control,
        "trade.approve",
        json!({"workspaceId":scenario.workspace,"proposalId":proposal["data"]["proposalId"],"proposalHash":proposal["data"]["proposalHash"],"reviewedRiskDecisionId":repeated["data"]["riskDecision"]["decisionId"],"reviewDigest":repeated["data"]["reviewDigest"],"expectedStateVersion":proposal["data"]["stateVersion"]}),
    );
    assert_eq!(approved["ok"], false, "{approved}");
    let approvals = command(
        &mut control,
        "trade.approval.list",
        json!({"workspaceId":scenario.workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert!(
        approvals["data"]["approvals"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let retained = command(
        &mut control,
        "account.get",
        json!({"workspaceId":scenario.workspace,"connectionId":live["connectionId"]}),
    );
    assert_eq!(retained["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(retained["data"]["health"]["arming"], "DISARMED");
}

#[test]
fn source_read_errors_distinguish_auth_permission_and_quota_without_echoing_the_body() {
    use tradex::provider_io::{
        ProviderHttp, ProviderHttpMethod, ProviderHttpResponse, ProviderRateLimit,
    };
    struct RejectedHttp {
        status: u16,
        reads: std::cell::Cell<usize>,
    }
    impl ProviderHttp for RejectedHttp {
        fn get(
            &self,
            _: tradex::provider_io::ProviderEndpoint,
            _: &str,
            _: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            panic!("Status-aware read expected")
        }
        fn request_with_rate_limit(
            &self,
            endpoint: tradex::provider_io::ProviderEndpoint,
            method: ProviderHttpMethod,
            path: &str,
            headers: reqwest::header::HeaderMap,
            body: Option<&Value>,
        ) -> tradex::protocol::Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
            assert_eq!(
                endpoint,
                tradex::provider_io::ProviderEndpoint::Trading212Live
            );
            assert_eq!(method, ProviderHttpMethod::Get);
            assert_eq!(path, "/api/v0/equity/account/summary");
            assert!(headers["Authorization"].is_sensitive());
            assert!(body.is_none());
            self.reads.set(self.reads.get() + 1);
            Ok((
                ProviderHttpResponse {
                    status: self.status,
                    body: provider_fixtures::SECRET.as_bytes().to_vec(),
                },
                None,
            ))
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let vault = provider_fixtures::Vault::default();
    let mut cases = Vec::new();
    for (index, status, expected) in [
        (0, 401, "Authentication"),
        (1, 403, "denied access"),
        (2, 429, "quota"),
    ] {
        let mut control = ControlPlane::new(directory.path().join(format!("case-{index}")));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let account_http = provider_fixtures::Http::default();
        account_http
            .trading212_identity
            .set(9007199254741793u64 + index);
        let account = connect_using(
            &mut control,
            &workspace,
            "trading212",
            "LIVE",
            &vault,
            &account_http,
        );
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let source = command(
            &mut control,
            "data.instrument.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.instrument.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
        );
        let request = json!({"requestId":"rejected-metadata","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
        cases.push((
            std::sync::Arc::new(std::sync::Mutex::new(control)),
            request,
            RejectedHttp {
                status,
                reads: Default::default(),
            },
            expected,
        ));
    }
    std::thread::sleep(std::time::Duration::from_millis(5100));
    for (control, request, http, expected) in cases {
        let read =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(read["data"]["status"], "UNAVAILABLE", "{read}");
        assert!(
            read["data"]["availabilityReason"]
                .as_str()
                .unwrap()
                .contains(expected),
            "The source should explain its actual read failure: {read}"
        );
        assert!(read["data"]["evidence"].is_null());
        assert!(!read.to_string().contains(provider_fixtures::SECRET));
        assert_eq!(
            http.reads.get(),
            1,
            "No fallback or automatic retry is permitted"
        );
    }
}

#[test]
fn company_event_forbidden_response_is_an_access_failure_without_an_alternate_read() {
    use tradex::provider_io::{
        ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse, ProviderRateLimit,
    };
    struct Forbidden(std::cell::Cell<usize>);
    impl Forbidden {
        fn response(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
            assert_eq!(endpoint, ProviderEndpoint::AlpacaMarketData);
            assert!(path.starts_with(
                "/v1/corporate-actions?symbols=AAPL,MSFT&region=us&data_quality=all&start="
            ));
            assert!(headers["APCA-API-SECRET-KEY"].is_sensitive());
            self.0.set(self.0.get() + 1);
            Ok((
                ProviderHttpResponse {
                    status: 403,
                    body: provider_fixtures::SECRET.as_bytes().to_vec(),
                },
                None,
            ))
        }
    }
    impl ProviderHttp for Forbidden {
        fn get(
            &self,
            _: ProviderEndpoint,
            _: &str,
            _: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            panic!("Status-aware response required")
        }
        fn get_response_with_rate_limit(
            &self,
            e: ProviderEndpoint,
            p: &str,
            h: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
            self.response(e, p, h)
        }
        fn request_with_rate_limit(
            &self,
            e: ProviderEndpoint,
            m: ProviderHttpMethod,
            p: &str,
            h: reqwest::header::HeaderMap,
            b: Option<&Value>,
        ) -> tradex::protocol::Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
            assert_eq!(m, ProviderHttpMethod::Get);
            assert!(b.is_none());
            self.response(e, p, h)
        }
    }
    let scenario = ActionScenario::new();
    let http = Forbidden(Default::default());
    let reply = scenario.refresh(&http);
    assert_eq!(reply["data"]["status"], "UNAVAILABLE", "{reply}");
    assert!(
        reply["data"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("denied access"),
        "A forbidden data read does not establish invalid credentials: {reply}"
    );
    assert!(reply["data"]["evidence"].is_null());
    assert!(!reply.to_string().contains(provider_fixtures::SECRET));
    assert_eq!(
        http.0.get(),
        1,
        "Do not retry using another key, host or data quality"
    );
}

#[test]
fn public_source_account_workspace_and_clock_changes_discard_late_company_result() {
    struct RacingHttp<'a> {
        body: ActionBody,
        change: std::cell::RefCell<Box<dyn FnMut() + 'a>>,
    }
    impl tradex::provider_io::ProviderHttp for RacingHttp<'_> {
        fn get(
            &self,
            e: tradex::provider_io::ProviderEndpoint,
            p: &str,
            h: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            (self.change.borrow_mut())();
            self.body.get(e, p, h)
        }
    }
    for boundary in ["source", "account", "workspace", "clock", "sequence"] {
        let scenario = ActionScenario::new();
        let http = RacingHttp {
            body: ActionBody(scenario.page("REGULAR")),
            change: std::cell::RefCell::new(Box::new(|| {
                if boundary == "sequence" {
                    let newer = scenario.refresh(&ActionBody(scenario.page("UPDATED")));
                    assert_eq!(newer["data"]["status"], "AVAILABLE", "{newer}");
                    return;
                }
                let mut control = scenario
                    .control
                    .try_lock()
                    .expect("Provider IO must release the Control Plane lock");
                match boundary {
                    "source" => {
                        let reply = command(
                            &mut control,
                            "data.actions.disconnect",
                            json!({"workspaceId":scenario.workspace,"expectedStateVersion":scenario.source["stateVersion"]}),
                        );
                        assert_eq!(reply["ok"], true, "{reply}");
                    }
                    "account" => {
                        let job=control.prepare_provider(&json!({"requestId":"account-change","schemaVersion":1,"command":"provider.disconnect","payload":{"workspaceId":scenario.workspace,"connectionId":scenario.account["connectionId"],"expectedStateVersion":scenario.account["stateVersion"]}})).unwrap();
                        assert!(job.is_some());
                    }
                    "workspace" => {
                        let reply = command(
                            &mut control,
                            "workspace.open",
                            json!({"path":scenario._directory.path().join("workspace")}),
                        );
                        assert_eq!(reply["ok"], true, "{reply}");
                    }
                    "clock" => {
                        let reply = command(
                            &mut control,
                            "time.revalidate",
                            json!({"workspaceId":scenario.workspace}),
                        );
                        assert_eq!(reply["ok"], true, "{reply}");
                    }
                    _ => unreachable!(),
                }
            })),
        };
        let reply = scenario.refresh(&http);
        assert_eq!(
            reply["error"]["code"], "STATE_VERSION_CONFLICT",
            "{boundary}: {reply}"
        );
        let source = command(
            &mut scenario.control.lock().unwrap(),
            "data.actions.connection",
            json!({"workspaceId":scenario.workspace}),
        );
        if boundary == "sequence" {
            assert_eq!(source["data"]["status"], "AVAILABLE");
            assert_eq!(
                source["data"]["evidence"]["actions"][0]["subType"], "UPDATED",
                "An older job cannot replace the newer result"
            );
        } else {
            assert!(source["data"]["evidence"].is_null(), "{boundary}: {source}");
            assert_ne!(source["data"]["status"], "AVAILABLE");
        }
    }
}

#[test]
fn broker_read_boundaries_stop_late_results_and_remaining_metadata_requests() {
    use std::sync::{Arc, Mutex};
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};

    struct RacingHttp<'a> {
        body: BrokerResponses,
        phase: &'a str,
        change: std::cell::RefCell<Box<dyn FnMut() + 'a>>,
    }
    impl ProviderHttp for RacingHttp<'_> {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            let result = self.body.get(endpoint, path, headers);
            if path == self.phase {
                (self.change.borrow_mut())();
            }
            result
        }
    }

    let phases = [
        "/api/v0/equity/account/summary",
        "/api/v0/equity/metadata/instruments",
        "/api/v0/equity/metadata/exchanges",
    ];
    let boundaries = ["source", "account", "workspace", "clock"];
    let mut scenarios = Vec::new();
    for phase in phases {
        for boundary in boundaries {
            let directory = tempfile::tempdir().unwrap();
            let mut control = ControlPlane::new(directory.path().join("workspace"));
            let workspace =
                command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
            let vault = provider_fixtures::Vault::default();
            let account_http = provider_fixtures::Http::default();
            let identity = 9007199254741600u64 + scenarios.len() as u64;
            account_http.trading212_identity.set(identity);
            let account = connect_using(
                &mut control,
                &workspace,
                "trading212",
                "LIVE",
                &vault,
                &account_http,
            );
            command(
                &mut control,
                "time.revalidate",
                json!({"workspaceId":workspace}),
            );
            let source = command(
                &mut control,
                "data.instrument.connection",
                json!({"workspaceId":workspace}),
            );
            let saved = command(
                &mut control,
                "data.instrument.configure",
                json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
            );
            assert_eq!(saved["ok"], true, "{saved}");
            scenarios.push((
                directory,
                Arc::new(Mutex::new(control)),
                workspace,
                vault,
                account,
                saved,
                identity,
                phase,
                boundary,
            ));
        }
    }
    // Normal connection reads consume the real per-account summary interval.
    std::thread::sleep(std::time::Duration::from_millis(5100));
    for (directory, control, workspace, vault, account, saved, identity, phase, boundary) in
        scenarios
    {
        let mut body = BrokerResponses::default();
        body.account["id"] = json!(identity);
        let http = RacingHttp {
            body,
            phase,
            change: std::cell::RefCell::new(Box::new(|| {
                let mut engine = control
                    .try_lock()
                    .expect("Broker HTTP must release the Control Plane lock");
                match boundary {
                    "source" => {
                        let changed = command(
                            &mut engine,
                            "data.instrument.disconnect",
                            json!({"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}),
                        );
                        assert_eq!(changed["ok"], true, "{changed}");
                    }
                    "account" => {
                        let job = engine.prepare_provider(&json!({"requestId":"broker-account-change","schemaVersion":1,"command":"provider.disconnect","payload":{"workspaceId":workspace,"connectionId":account["connectionId"],"expectedStateVersion":account["stateVersion"]}})).unwrap();
                        assert!(job.is_some());
                    }
                    "workspace" => {
                        let changed = command(
                            &mut engine,
                            "workspace.open",
                            json!({"path":directory.path().join("workspace")}),
                        );
                        assert_eq!(changed["ok"], true, "{changed}");
                    }
                    "clock" => {
                        let changed = command(
                            &mut engine,
                            "time.revalidate",
                            json!({"workspaceId":workspace}),
                        );
                        assert_eq!(changed["ok"], true, "{changed}");
                    }
                    _ => unreachable!(),
                }
            })),
        };
        let reply = tradex::financial_sources::execute_refresh(
            &control,
            &json!({"requestId":"broker-late-read","schemaVersion":1,"command":"data.instrument.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
            "main",
            &vault,
            &http,
        );
        assert_eq!(
            reply["error"]["code"], "STATE_VERSION_CONFLICT",
            "{phase}/{boundary}: {reply}"
        );
        let last = phases
            .iter()
            .position(|candidate| *candidate == phase)
            .unwrap();
        assert_eq!(
            *http.body.paths.borrow(),
            phases[..=last],
            "No later metadata request after {phase}/{boundary}"
        );
        let source = command(
            &mut control.lock().unwrap(),
            "data.instrument.connection",
            json!({"workspaceId":workspace}),
        );
        assert!(
            source["data"]["evidence"].is_null(),
            "{phase}/{boundary}: {source}"
        );
        assert_ne!(source["data"]["status"], "AVAILABLE");
    }
}
#[test]
#[cfg(feature = "integration-test")]
fn company_query_expires_without_renewing_its_first_receipt_or_material_version() {
    let scenario = ActionScenario::new();
    let first = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
    assert_eq!(first["data"]["status"], "AVAILABLE");
    let mut control = scenario.control.lock().unwrap();
    control
        .advance_test_clock_fixture(scenario.workspace.as_str().unwrap(), 20_000)
        .unwrap();
    let fresh = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":scenario.workspace}),
    );
    assert_eq!(fresh["data"]["status"], "AVAILABLE");
    assert_eq!(fresh["data"]["evidence"], first["data"]["evidence"]);
    control
        .advance_test_clock_fixture(scenario.workspace.as_str().unwrap(), 10_001)
        .unwrap();
    let stale = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":scenario.workspace}),
    );
    assert_eq!(stale["data"]["status"], "UNAVAILABLE");
    assert_eq!(
        stale["data"]["evidence"], first["data"]["evidence"],
        "Retained context stays bound to its original receipt and hash"
    );
    let market = command(
        &mut control,
        "market.get",
        json!({"workspaceId":scenario.workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
    );
    assert_eq!(
        market["data"]["financialEvidence"]["companyEvents"]["status"],
        "UNAVAILABLE"
    );
    assert!(market["data"]["instrumentState"].is_null());
}

#[test]
fn uuid_case_aliases_cannot_duplicate_a_company_action() {
    let scenario = ActionScenario::new();
    let mut page: Value = serde_json::from_slice(&scenario.page("REGULAR")).unwrap();
    let mut duplicate = page["corporate_actions"]["cash_dividends"][0].clone();
    duplicate["id"] = json!(duplicate["id"].as_str().unwrap().to_uppercase());
    page["corporate_actions"]["cash_dividends"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let read = scenario.refresh(&ActionBody(serde_json::to_vec(&page).unwrap()));
    assert_eq!(
        read["data"]["status"], "UNAVAILABLE",
        "UUID aliases must not create two known actions: {read}"
    );
    assert!(read["data"]["evidence"].is_null());
}

#[test]
fn company_pagination_enforces_bounds_without_truncating_or_publishing_partial_queries() {
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    struct Pages {
        template: Value,
        count: std::cell::Cell<usize>,
        unfinished: bool,
    }
    impl ProviderHttp for Pages {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            _: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            assert_eq!(endpoint, ProviderEndpoint::AlpacaMarketData);
            let page = self.count.get();
            self.count.set(page + 1);
            if page > 0 {
                assert!(path.ends_with(&format!("page_token=page-{page}")));
            }
            let rows = (0..100)
                .map(|index| {
                    let mut row = self.template.clone();
                    row["id"] = json!(format!(
                        "01234567-89ab-4cde-8f01-{:012}",
                        page * 100 + index
                    ));
                    row
                })
                .collect::<Vec<_>>();
            Ok(serde_json::to_vec(&json!({"corporate_actions":{"cash_dividends":rows},"next_page_token":if page == 9 && !self.unfinished { None } else { Some(format!("page-{}",page+1)) }})).unwrap())
        }
    }
    let scenario = ActionScenario::new();
    let parsed: Value = serde_json::from_slice(&scenario.page("REGULAR")).unwrap();
    let template = parsed["corporate_actions"]["cash_dividends"][0].clone();
    let pages = Pages {
        template: template.clone(),
        count: Default::default(),
        unfinished: false,
    };
    let read = scenario.refresh(&pages);
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    assert_eq!(
        read["data"]["evidence"]["actions"]
            .as_array()
            .unwrap()
            .len(),
        1000
    );
    assert_eq!(pages.count.get(), 10);
    let unfinished = Pages {
        template,
        count: Default::default(),
        unfinished: true,
    };
    let failed = scenario.refresh(&unfinished);
    assert_eq!(failed["data"]["status"], "UNAVAILABLE");
    assert!(failed["data"]["evidence"].is_null());
    assert_eq!(
        unfinished.count.get(),
        10,
        "An eleventh request is prohibited"
    );

    let mut excessive: Value = serde_json::from_slice(&scenario.page("REGULAR")).unwrap();
    excessive["corporate_actions"]["cash_dividends"] = json!(vec![
            excessive["corporate_actions"]["cash_dividends"][0]
                .clone();
            101
        ]);
    let oversized_page = scenario.refresh(&ActionBody(serde_json::to_vec(&excessive).unwrap()));
    assert_eq!(oversized_page["data"]["status"], "UNAVAILABLE");
    let mut raw = scenario.page("REGULAR");
    raw.resize(512 * 1024 + 1, b' ');
    let oversized_body = scenario.refresh(&ActionBody(raw));
    assert_eq!(oversized_body["data"]["status"], "UNAVAILABLE");
    assert!(oversized_body["data"]["evidence"].is_null());
}

#[test]
fn revised_and_removed_events_replace_current_material_without_certifying_complete_coverage() {
    let scenario = ActionScenario::new();
    let original = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
    let mut revised: Value = serde_json::from_slice(&scenario.page("REGULAR")).unwrap();
    revised["corporate_actions"]["cash_dividends"][0]["rate"] = json!("0.5");
    let changed = scenario.refresh(&ActionBody(serde_json::to_vec(&revised).unwrap()));
    assert_eq!(changed["data"]["status"], "AVAILABLE");
    assert_ne!(
        changed["data"]["evidence"]["materialVersion"],
        original["data"]["evidence"]["materialVersion"]
    );
    assert_eq!(
        changed["data"]["evidence"]["actions"][0]["terms"][0]["value"],
        "0.5"
    );
    let empty = scenario.refresh(&ActionBody(
        br#"{"corporate_actions":{},"next_page_token":null}"#.to_vec(),
    ));
    assert_eq!(empty["data"]["status"], "AVAILABLE");
    assert_eq!(empty["data"]["evidence"]["queryComplete"], true);
    assert!(
        empty["data"]["evidence"]["actions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_ne!(
        empty["data"]["evidence"]["materialVersion"],
        changed["data"]["evidence"]["materialVersion"]
    );
    for capability in empty["data"]["capabilityStatuses"].as_array().unwrap() {
        if ["COMPLETE_ACTION_COVERAGE", "HISTORICAL_ADJUSTMENT"]
            .iter()
            .any(|name| capability["capability"] == *name)
        {
            assert_ne!(capability["status"], "AVAILABLE");
        }
    }
}

#[test]
#[ignore = "Uses 49 seconds of real elapsed time to verify the HTTP timeout reserve"]
fn a_late_vault_read_cannot_start_http_without_the_full_timeout_budget() {
    use tradex::provider_io::{CredentialVault, Credentials, ProviderEndpoint, ProviderHttp};
    let scenario = ActionScenario::new();
    #[derive(Clone)]
    struct LateVault {
        inner: provider_fixtures::Vault,
        control: std::sync::Arc<std::sync::Mutex<ControlPlane>>,
    }
    impl CredentialVault for LateVault {
        fn put(&self, _: &str, _: &Credentials) -> tradex::protocol::Result<()> {
            panic!("No source owns credentials")
        }
        fn remove(&self, _: &str) -> tradex::protocol::Result<()> {
            panic!("No source deletes credentials")
        }
        fn get(&self, reference: &str) -> tradex::protocol::Result<Credentials> {
            assert!(
                self.control.try_lock().is_ok(),
                "Protected vault waits must not hold the control plane"
            );
            let credentials = self.inner.get(reference)?;
            std::thread::sleep(std::time::Duration::from_secs(49));
            Ok(credentials)
        }
    }
    struct ReadCount {
        body: Vec<u8>,
        reads: std::cell::Cell<usize>,
    }
    impl ProviderHttp for ReadCount {
        fn get(
            &self,
            _: ProviderEndpoint,
            _: &str,
            _: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            self.reads.set(self.reads.get() + 1);
            Ok(self.body.clone())
        }
    }
    let http = ReadCount {
        body: scenario.page("REGULAR"),
        reads: Default::default(),
    };
    let vault = LateVault {
        inner: scenario.vault.clone(),
        control: scenario.control.clone(),
    };
    let started = std::time::Instant::now();
    let reply = tradex::financial_sources::execute_refresh(
        &scenario.control,
        &scenario.request(),
        "main",
        &vault,
        &http,
    );
    assert_eq!(
        http.reads.get(),
        0,
        "A 12-second HTTP operation cannot fit within the remaining 11-second job budget"
    );
    assert_eq!(reply["data"]["status"], "UNAVAILABLE");
    assert!(started.elapsed() < std::time::Duration::from_secs(60));
}

#[test]
#[ignore = "Uses63seconds of real elapsed time to verify protected authentication deadline"]
fn protected_authentication_returns_by_the_job_deadline_and_discards_late_credentials() {
    use std::sync::{Arc, Mutex, mpsc};
    use tradex::provider_io::{CredentialVault, Credentials, ProviderEndpoint, ProviderHttp};
    let scenario = ActionScenario::new();
    let prior = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
    assert_eq!(prior["data"]["status"], "AVAILABLE");
    #[derive(Clone)]
    struct BlockedVault {
        control: Arc<Mutex<ControlPlane>>,
        completed: mpsc::Sender<()>,
    }
    impl CredentialVault for BlockedVault {
        fn put(&self, _: &str, _: &Credentials) -> tradex::protocol::Result<()> {
            panic!("Read-only source must not store credentials")
        }
        fn remove(&self, _: &str) -> tradex::protocol::Result<()> {
            panic!("Read-only source must not delete credentials")
        }
        fn get(&self, _: &str) -> tradex::protocol::Result<Credentials> {
            assert!(
                self.control.try_lock().is_ok(),
                "Authentication cannot hold the Control Plane"
            );
            let credentials = provider_fixtures::credentials()?;
            std::thread::sleep(std::time::Duration::from_secs(63));
            self.completed.send(()).unwrap();
            Ok(credentials)
        }
    }
    struct NoHttp;
    impl ProviderHttp for NoHttp {
        fn get(
            &self,
            _: ProviderEndpoint,
            _: &str,
            _: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            panic!("Expired authentication must never start provider HTTP")
        }
    }
    let (completed, completion) = mpsc::channel();
    let vault = BlockedVault {
        control: scenario.control.clone(),
        completed,
    };
    let started = std::time::Instant::now();
    let reply = tradex::financial_sources::execute_refresh(
        &scenario.control,
        &scenario.request(),
        "main",
        &vault,
        &NoHttp,
    );
    assert!(
        started.elapsed() < std::time::Duration::from_secs(62),
        "Refresh waited beyond its60-second job deadline: {:?}",
        started.elapsed()
    );
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["data"]["status"], "UNAVAILABLE", "{reply}");
    assert!(
        reply["data"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("deadline")
    );
    assert!(reply["data"]["observedAt"].is_null());
    assert!(reply["data"]["evidence"].is_null());
    eprintln!(
        "Protected authentication Refresh returned after {:?}",
        started.elapsed()
    );
    let retry_started = std::time::Instant::now();
    let pending_retry = tradex::financial_sources::execute_refresh(
        &scenario.control,
        &scenario.request(),
        "main",
        &vault,
        &NoHttp,
    );
    assert!(
        retry_started.elapsed() < std::time::Duration::from_secs(1),
        "An already pending vault read must reject a repeated attempt immediately"
    );
    assert_eq!(pending_retry["data"]["status"], "UNAVAILABLE");
    assert!(
        pending_retry["data"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("could not be scheduled")
    );
    completion
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let source = command(
        &mut scenario.control.lock().unwrap(),
        "data.actions.connection",
        json!({"workspaceId":scenario.workspace}),
    );
    assert_eq!(source["data"]["status"], "UNAVAILABLE");
    assert!(
        source["data"]["evidence"].is_null(),
        "Late credentials cannot republish retired evidence"
    );
    let retry = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
    assert_eq!(
        retry["data"]["status"], "AVAILABLE",
        "A fresh explicit retry after authentication ends may read: {retry}"
    );
}

#[test]
#[cfg(feature = "integration-test")]
fn actual_http_adapter_retires_company_evidence_on_redirect_timeout_and_oversize() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use tradex::provider_io::BrokerHttp;
    for mode in [
        "auth",
        "permission",
        "quota",
        "redirect",
        "oversize",
        "timeout",
    ] {
        let scenario = ActionScenario::new();
        let prior = scenario.refresh(&ActionBody(scenario.page("REGULAR")));
        assert_eq!(prior["data"]["status"], "AVAILABLE");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 2048];
            while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                let read = stream.read(&mut buffer).unwrap();
                assert!(read > 0);
                request.extend_from_slice(&buffer[..read]);
                assert!(request.len() < 16384);
            }
            assert!(request.starts_with(
                b"GET /v1/corporate-actions?symbols=AAPL,MSFT&region=us&data_quality=all&start="
            ));
            if mode == "timeout" {
                std::thread::sleep(std::time::Duration::from_secs(13));
            } else {
                let (status, extra, body) = match mode {
                    "auth" => (
                        401,
                        String::new(),
                        br#"{"secret":"S02-FAKE-SECRET-704556921"}"#.to_vec(),
                    ),
                    "permission" => (403, String::new(), b"denied".to_vec()),
                    "quota" => (429, "Retry-After: 1\r\n".into(), b"quota".to_vec()),
                    "redirect" => (
                        302,
                        format!("Location: http://{address}/unapproved-redirect\r\n"),
                        vec![],
                    ),
                    "oversize" => (200, String::new(), vec![b' '; 512 * 1024 + 1]),
                    _ => unreachable!(),
                };
                let header = format!(
                    "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n",
                    body.len()
                );
                stream.write_all(header.as_bytes()).unwrap();
                let _ = stream.write_all(&body);
            }
            drop(stream);
            listener.set_nonblocking(true).unwrap();
            assert!(
                listener.accept().is_err(),
                "No redirect, retry, alternate endpoint or mutation is allowed"
            );
        });
        let adapter = BrokerHttp::for_loopback_test(&format!("http://{address}")).unwrap();
        let started = std::time::Instant::now();
        let failed = scenario.refresh(&adapter);
        assert_eq!(failed["data"]["status"], "UNAVAILABLE", "{mode}: {failed}");
        assert!(
            failed["data"]["evidence"].is_null(),
            "{mode} retained prior authority"
        );
        assert!(!failed.to_string().contains(provider_fixtures::SECRET));
        if mode == "timeout" {
            assert!(started.elapsed() < std::time::Duration::from_secs(13));
        }
        worker.join().unwrap();
    }
}

#[test]
fn fx_source_reuses_explicit_saved_key_and_reopens_without_authority() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let initial = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId": workspace}),
    );
    assert_eq!(
        initial["ok"], true,
        "FX source selection is unavailable: {initial}"
    );
    assert_eq!(initial["data"]["configured"], false);
    assert_eq!(
        initial["data"]["eligibleAccounts"][0]["connectionId"],
        account["connectionId"]
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["kind"], "FX");
    assert_eq!(saved["data"]["status"], "UNVERIFIED");
    assert!(saved["data"]["observedAt"].is_null());
    assert!(saved["data"]["evidence"].is_null());
    assert_eq!(
        saved["data"]["capabilityStatuses"][1]["capability"],
        "TRANSACTION_FX_QUALIFICATION"
    );
    assert_eq!(
        saved["data"]["capabilityStatuses"][1]["status"],
        "BLOCKED_EXTERNAL"
    );
    assert_eq!(
        command(
            &mut control,
            "data.fx.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":initial["data"]["stateVersion"],"connectionId":account["connectionId"]})
        )["error"]["code"],
        "STATE_VERSION_CONFLICT"
    );
    drop(control);
    let mut control = ControlPlane::new(path);
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(restored["data"]["connectionId"], account["connectionId"]);
    assert_eq!(restored["data"]["status"], "UNVERIFIED");
    let disconnected = command(
        &mut control,
        "data.fx.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":restored["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["data"]["configured"], false);
    let retained = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["connectionId"]}),
    );
    assert_eq!(retained["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(retained["data"]["health"]["arming"], "NOT_APPLICABLE");
    assert_eq!(vault.present.borrow().len(), 1);
    for projection in [saved, restored, disconnected] {
        assert!(!projection.to_string().contains(provider_fixtures::KEY));
        assert!(!projection.to_string().contains(provider_fixtures::SECRET));
    }
}

#[test]
fn fx_requirements_distinguish_identity_from_actual_cross_currency_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let alpaca = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let same = command(
        &mut control,
        "data.fx.requirements",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        same["ok"], true,
        "Actual FX requirements are unavailable: {same}"
    );
    assert_eq!(same["data"]["baseCurrency"], "USD");
    let rows = same["data"]["requirements"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|row| {
            row["connectionId"] == alpaca["connectionId"] && row["purpose"] == "ACCOUNT_WORKSPACE"
        })
        .unwrap();
    assert_eq!(row["need"], "IDENTITY");
    assert_eq!(row["providerPair"], Value::Null);
    let live = connect(&mut control, &workspace, "trading212", "LIVE", &vault);
    let changed = command(
        &mut control,
        "data.fx.requirements",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(changed["ok"], true, "{changed}");
    let row = changed["data"]["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["connectionId"] == live["connectionId"] && row["purpose"] == "ACCOUNT_WORKSPACE"
        })
        .unwrap();
    assert_eq!(row["fromCurrency"], "GBP");
    assert_eq!(row["toCurrency"], "USD");
    assert_eq!(row["need"], "EXTERNAL_RATE");
    assert_eq!(
        row["providerPair"],
        Value::Null,
        "An unsupported route must not be silently replaced with EUR/USD"
    );
    assert_ne!(
        same["data"]["materialVersion"],
        changed["data"]["materialVersion"]
    );
    let repeated = command(
        &mut control,
        "data.fx.requirements",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        repeated["data"]["materialVersion"],
        changed["data"]["materialVersion"]
    );
    let account = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":live["connectionId"]}),
    );
    assert_eq!(account["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(account["data"]["health"]["arming"], "DISARMED");
}

struct FxReadHttp {
    account: provider_fixtures::Http,
    body: Vec<u8>,
    paths: std::cell::RefCell<Vec<String>>,
}

#[test]
fn observed_wallet_units_require_named_portfolio_routes_and_actual_supported_reads() {
    use std::sync::{Arc, Mutex};
    use tradex::provider_io::{
        ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse,
    };
    struct WalletHttp(FxReadHttp);
    impl ProviderHttp for WalletHttp {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            let bytes = self.0.get(endpoint, path, headers)?;
            if matches!(
                path,
                "/api/v2/spot/account/info" | "/api/v2/spot/account/assets?assetType=all"
            ) {
                let mut body: Value = serde_json::from_slice(&bytes).unwrap();
                if path == "/api/v2/spot/account/info" {
                    body["data"].as_object_mut().unwrap().remove("authorities");
                } else {
                    for coin in ["EUR", "OP", "1INCH"] {
                        body["data"].as_array_mut().unwrap().push(json!({
                            "coin":coin,"available":"12.34","frozen":"0","locked":"0","limitAvailable":"12.34"
                        }));
                    }
                }
                return Ok(serde_json::to_vec(&body).unwrap());
            }
            Ok(bytes)
        }
        fn request(
            &self,
            endpoint: ProviderEndpoint,
            method: ProviderHttpMethod,
            path: &str,
            headers: reqwest::header::HeaderMap,
            body: Option<&Value>,
        ) -> tradex::protocol::Result<ProviderHttpResponse> {
            assert_eq!(
                method,
                ProviderHttpMethod::Get,
                "Wallet/FX context must never write"
            );
            assert!(body.is_none());
            self.get(endpoint, path, headers)
                .map(|body| ProviderHttpResponse { status: 200, body })
        }
    }
    let t = (time::OffsetDateTime::now_utc() - time::Duration::seconds(1))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let http = WalletHttp(FxReadHttp {
        account: provider_fixtures::Http::default(),
        body: format!(r#"{{"rates":{{"EURUSD":{{"bp":1.01,"ap":1.12,"mp":1.07,"t":"{t}"}}}}}}"#)
            .into_bytes(),
        paths: Default::default(),
    });
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let source_account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let wallet = connect_using(&mut control, &workspace, "bitget", "LIVE", &vault, &http);
    let account = command(
        &mut control,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":wallet["connectionId"]}),
    );
    assert!(account["data"]["data"]["currency"].is_null());
    assert_eq!(account["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(account["data"]["health"]["arming"], "DISARMED");
    let requirements = command(
        &mut control,
        "data.fx.requirements",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(requirements["ok"], true, "{requirements}");
    let rows = requirements["data"]["requirements"].as_array().unwrap();
    assert!(
        rows.iter()
            .any(|r| r["connectionId"] == wallet["connectionId"]
                && r["purpose"] == "ACCOUNT_WORKSPACE"
                && r["need"] == "UNKNOWN_CURRENCY")
    );
    for (unit, pair) in [
        ("USDT", Value::Null),
        ("EUR", json!("EURUSD")),
        ("OP", Value::Null),
        ("1INCH", Value::Null),
    ] {
        let row = rows
            .iter()
            .find(|r| {
                r["connectionId"] == wallet["connectionId"]
                    && r["purpose"] == "BALANCE_WORKSPACE"
                    && r["fromCurrency"] == unit
            })
            .unwrap_or_else(|| {
                panic!("Actual {unit} wallet monetary input needs its named route: {requirements}")
            });
        assert_eq!(row["toCurrency"], "USD");
        assert_eq!(row["need"], "EXTERNAL_RATE");
        assert_eq!(row["providerPair"], pair);
    }
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":source_account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let read = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"wallet-fx","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    assert_eq!(
        &*http.0.paths.borrow(),
        &["/v1beta1/forex/latest/rates?currency_pairs=EURUSD"]
    );
    assert_eq!(
        read["data"]["capabilityStatuses"][1]["status"],
        "BLOCKED_EXTERNAL"
    );
    assert!(http.0.account.trading212_posts.borrow().is_empty());
    assert!(http.0.account.trading212_delete_calls.borrow().is_empty());
}
impl tradex::provider_io::ProviderHttp for FxReadHttp {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        if matches!(
            path,
            "/v1beta1/forex/latest/rates?currency_pairs=EURUSD"
                | "/v1beta1/forex/latest/rates?currency_pairs=EURUSD,USDEUR"
        ) {
            assert_eq!(
                endpoint,
                tradex::provider_io::ProviderEndpoint::AlpacaMarketData
            );
            assert_eq!(headers["APCA-API-KEY-ID"], provider_fixtures::KEY);
            assert_eq!(headers["APCA-API-SECRET-KEY"], provider_fixtures::SECRET);
            assert!(!headers.contains_key("authorization"));
            self.paths.borrow_mut().push(path.into());
            return Ok(self.body.clone());
        }
        let bytes = self.account.get(endpoint, path, headers)?;
        if matches!(
            path,
            "/api/v0/equity/account/summary" | "/api/v0/equity/positions"
        ) {
            return Ok(String::from_utf8(bytes)
                .unwrap()
                .replace("GBP", "EUR")
                .into_bytes());
        }
        Ok(bytes)
    }
}

#[test]
fn required_fx_read_preserves_exact_bid_ask_mid_and_separate_qualification() {
    use std::sync::{Arc, Mutex};
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let source_account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let provider_time = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let http = FxReadHttp {
        account: provider_fixtures::Http::default(),
        body: format!(r#"{{"rates":{{"EURUSD":{{"bp":1.01234567890123456789,"ap":1.11234567890123456789,"mp":1.07,"t":"{provider_time}"}}}}}}"#).into_bytes(),
        paths: Default::default(),
    };
    let _live = connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &http,
    );
    assert_eq!(
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace})
        )["ok"],
        true
    );
    let source = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":source_account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let result = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"fx-read","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(result["ok"], true, "FX read is unavailable: {result}");
    assert_eq!(result["data"]["status"], "AVAILABLE", "{result}");
    let evidence = &result["data"]["evidence"];
    assert_eq!(evidence["kind"], "FX");
    assert_eq!(evidence["providerQuality"], "UNQUALIFIED_FX_RATE");
    assert_eq!(evidence["rates"][0]["providerPair"], "EURUSD");
    assert_eq!(evidence["rates"][0]["bid"], "1.01234567890123456789");
    assert_eq!(evidence["rates"][0]["ask"], "1.11234567890123456789");
    assert_eq!(evidence["rates"][0]["mid"], "1.07");
    assert_eq!(evidence["rates"][0]["providerTimestamp"], provider_time);
    assert_eq!(
        result["data"]["capabilityStatuses"][1]["status"],
        "BLOCKED_EXTERNAL"
    );
    let mut control = control.lock().unwrap();
    let repeated = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(repeated["data"]["observedAt"], result["data"]["observedAt"]);
    assert_eq!(
        repeated["data"]["evidence"]["materialVersion"],
        evidence["materialVersion"]
    );
    let accounts = command(
        &mut control,
        "account.list",
        json!({"workspaceId":workspace}),
    );
    assert!(
        accounts["data"]["accounts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| a["providerId"] != "local-paper")
            .all(|a| a["permissions"]["scope"] != "VERIFIED" && a["health"]["arming"] != "ARMED")
    );
    assert_eq!(http.paths.borrow().len(), 1);
    assert!(http.account.trading212_posts.borrow().is_empty());
    assert!(http.account.trading212_delete_calls.borrow().is_empty());
}

#[derive(Clone, Default)]
struct CountingUnavailableVault(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl tradex::provider_io::CredentialVault for CountingUnavailableVault {
    fn put(&self, _: &str, _: &tradex::provider_io::Credentials) -> tradex::protocol::Result<()> {
        Err(tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"))
    }
    fn get(&self, _: &str) -> tradex::protocol::Result<tradex::provider_io::Credentials> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Err(tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"))
    }
    fn remove(&self, _: &str) -> tradex::protocol::Result<()> {
        Err(tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"))
    }
}

#[test]
fn identity_currency_refresh_does_not_request_a_key_or_external_rate() {
    use std::sync::{Arc, Mutex, atomic::Ordering};
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let account_vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &account_vault);
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    assert!(
        source["data"]["fxRequirements"]["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["need"] == "IDENTITY")
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    let vault = CountingUnavailableVault::default();
    let http = provider_fixtures::Http::default();
    let control = Arc::new(Mutex::new(control));
    let result = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"identity-fx","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(
        vault.0.load(Ordering::SeqCst),
        0,
        "Same-currency inputs must not request Keychain authentication: {result}"
    );
    assert!(http.calls.borrow().is_empty());
    assert_eq!(result["ok"], true, "{result}");
    assert!(result["data"]["evidence"].is_null());
    assert!(
        result["data"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("No external currency rate is required"),
        "{result}"
    );
}

#[test]
fn immutable_intent_adds_only_its_actual_funding_direction_to_fx_read() {
    use std::sync::{Arc, Mutex};
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let source_account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let t = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let http = FxReadHttp {
        account: provider_fixtures::Http::default(),
        body: format!(r#"{{"rates":{{"EURUSD":{{"bp":1.01,"ap":1.12,"mp":1.07,"t":"{t}"}},"USDEUR":{{"bp":0.88,"ap":0.99,"mp":0.93,"t":"{t}"}}}}}}"#).into_bytes(),
        paths: Default::default(),
    };
    let live = connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &http,
    );
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let draft = command(
        &mut control,
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":{
            "accountId":live["connectionId"],"venue":"XNAS","environment":"TRADING212_LIVE","instrumentId":"equity:US:AAPL","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"1"},"limitPrice":"221.50","maximumSpend":null,"timeInForce":"DAY","clientLabel":"FX route read only"
        }}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &mut control,
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    assert_eq!(
        proposal["data"]["estimatedNotionalCurrency"], "USD",
        "{proposal}"
    );
    let requirements = command(
        &mut control,
        "data.fx.requirements",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert_eq!(requirements["ok"], true, "{requirements}");
    assert!(
        requirements["data"]["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["purpose"] == "INTENT_FUNDING"
                && row["fromCurrency"] == "USD"
                && row["toCurrency"] == "EUR"
                && row["providerPair"] == "USDEUR")
    );
    let source = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":source_account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let result = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"intent-fx","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"],"proposalId":proposal["data"]["proposalId"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(
        result["ok"], true,
        "Immutable intent-scoped FX read is unavailable: {result}"
    );
    assert_eq!(result["data"]["status"], "AVAILABLE", "{result}");
    assert_eq!(
        *http.paths.borrow(),
        vec!["/v1beta1/forex/latest/rates?currency_pairs=EURUSD,USDEUR"]
    );
    assert_eq!(
        result["data"]["evidence"]["requirements"],
        requirements["data"]
    );
    let reread = command(
        &mut control.lock().unwrap(),
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(reread["data"]["status"], "AVAILABLE", "{reread}");
    assert_eq!(
        reread["data"]["evidence"]["rates"][1]["fromCurrency"],
        "USD"
    );
    assert_eq!(reread["data"]["evidence"]["rates"][1]["bid"], "0.88");
    let review_payload =
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]});
    let review = command(
        &mut control.lock().unwrap(),
        "trade.request_approval",
        review_payload.clone(),
    );
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(
        review["data"]["currencyEvidence"]["source"]["evidence"], result["data"]["evidence"],
        "The immutable review must capture the original FX observation: {review}"
    );
    assert_eq!(review["data"]["eligible"], false);
    assert!(
        review["data"]["riskDecision"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["checkId"] == "CURRENCY_CONVERSION" && row["outcome"] == "UNAVAILABLE")
    );
    let fx_digest = |value: &Value| {
        value["data"]["riskDecision"]["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["kind"] == "CURRENCY_RATES")
            .unwrap()["digest"]
            .clone()
    };
    let repeated = command(
        &mut control.lock().unwrap(),
        "trade.request_approval",
        review_payload.clone(),
    );
    assert_eq!(
        fx_digest(&review),
        fx_digest(&repeated),
        "Polling or review cannot renew captured FX material"
    );
    let disconnected = command(
        &mut control.lock().unwrap(),
        "data.fx.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":reread["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["ok"], true, "{disconnected}");
    let changed = command(
        &mut control.lock().unwrap(),
        "trade.request_approval",
        review_payload,
    );
    assert_ne!(
        fx_digest(&review),
        fx_digest(&changed),
        "Changed FX selection must change the risk and consent binding"
    );
    assert_eq!(changed["data"]["eligible"], false);
    assert!(http.account.trading212_posts.borrow().is_empty());
    assert!(http.account.trading212_delete_calls.borrow().is_empty());
}

#[test]
fn selected_or_disconnected_fx_source_cannot_restore_synthetic_portfolio_authority() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let live = connect(&mut control, &workspace, "trading212", "LIVE", &vault);
    let source = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    for disconnect in [false, true] {
        if disconnect {
            let result = command(
                &mut control,
                "data.fx.disconnect",
                json!({"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}),
            );
            assert_eq!(result["ok"], true, "{result}");
        }
        let portfolio = command(
            &mut control,
            "portfolio.get",
            json!({"workspaceId":workspace}),
        );
        assert_eq!(portfolio["ok"], true, "{portfolio}");
        assert!(
            portfolio["data"]["accounts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["connectionId"] == live["connectionId"]),
            "A selected producer must expose actual accounts: {portfolio}"
        );
        assert!(
            portfolio["data"]["accounts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| !row["connectionId"]
                    .as_str()
                    .unwrap()
                    .starts_with("fixture:"))
        );
        assert_eq!(portfolio["data"]["liveRisk"]["eligible"], false);
    }
}

#[test]
#[cfg(feature = "integration-test")]
fn legacy_portfolio_opt_in_cannot_override_explicit_fx_selection() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "selected_or_disconnected_fx_source_cannot_restore_synthetic_portfolio_authority",
            "--exact",
        ])
        .env("TRADEX_PORTFOLIO_FIXTURE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn schema_34_source_selections_survive_fx_migration_and_metadata_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let source = command(
        &mut control,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.actions.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    drop(control);
    // External historical storage fixture: restore only the v34 table constraints.
    // No permissions, balances, rates or financial authority are inserted.
    let database = rusqlite::Connection::open(path.join("workspace.sqlite3")).unwrap();
    database.execute_batch("BEGIN;
        CREATE TABLE financial_source_config_old (kind TEXT PRIMARY KEY CHECK(kind IN ('CORPORATE_ACTIONS','BROKER_INSTRUMENTS')), generation INTEGER NOT NULL CHECK(generation>0), projection TEXT NOT NULL);
        INSERT INTO financial_source_config_old SELECT * FROM financial_source_config;
        CREATE TABLE financial_source_config_audit_old (kind TEXT NOT NULL CHECK(kind IN ('CORPORATE_ACTIONS','BROKER_INSTRUMENTS')), generation INTEGER NOT NULL CHECK(generation>0), occurred_at TEXT NOT NULL, projection TEXT NOT NULL, PRIMARY KEY(kind,generation));
        INSERT INTO financial_source_config_audit_old SELECT * FROM financial_source_config_audit;
        DROP TABLE financial_source_config;
        DROP TABLE financial_source_config_audit;
        ALTER TABLE financial_source_config_old RENAME TO financial_source_config;
        ALTER TABLE financial_source_config_audit_old RENAME TO financial_source_config_audit;
        DROP TABLE binance_market_source_config; DROP TABLE binance_market_source_config_audit; PRAGMA user_version=34; COMMIT;").unwrap();
    drop(database);
    let mut reopened = ControlPlane::new(path);
    let opened = command(&mut reopened, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    assert_eq!(opened["data"]["storageSchemaVersion"], 37);
    let retained = command(
        &mut reopened,
        "data.actions.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        retained["data"]["connectionId"],
        saved["data"]["connectionId"]
    );
    assert_eq!(
        retained["data"]["stateVersion"],
        saved["data"]["stateVersion"]
    );
    assert_eq!(retained["data"]["status"], "UNVERIFIED");
    let fx = command(
        &mut reopened,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    let selected = command(
        &mut reopened,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":fx["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    assert_eq!(
        selected["ok"], true,
        "New FX selection must work after migration: {selected}"
    );
    assert!(selected["data"]["evidence"].is_null());
    assert_eq!(vault.present.borrow().len(), 1);
}

#[test]
fn malformed_fx_or_old_provider_time_retires_current_read_without_authority() {
    use std::sync::{Arc, Mutex};
    let now = time::OffsetDateTime::now_utc();
    let timestamp = |t: time::OffsetDateTime| {
        t.format(&time::format_description::well_known::Rfc3339)
            .unwrap()
    };
    let t = timestamp(now);
    let valid = format!(r#"{{"rates":{{"EURUSD":{{"bp":1.01,"ap":1.12,"mp":1.07,"t":"{t}"}}}}}}"#);
    let faults = vec![
        ("missing pair", "{\"rates\":{}}".to_owned()),
        ("wrong pair", valid.replace("EURUSD", "USDEUR")),
        (
            "extra pair",
            valid.replace("\"EURUSD\":", "\"USDEUR\":{},\"EURUSD\":"),
        ),
        (
            "duplicate pair",
            valid.replace("\"EURUSD\":", "\"EURUSD\":{},\"EURUSD\":"),
        ),
        (
            "duplicate field",
            valid.replace("\"bp\":1.01", "\"bp\":1.01,\"bp\":1.02"),
        ),
        ("missing ask", valid.replace("\"ap\":1.12,", "")),
        (
            "unknown field",
            valid.replace("\"bp\":1.01", "\"other\":true,\"bp\":1.01"),
        ),
        (
            "unknown root",
            valid.replace("{\"rates\":", "{\"other\":true,\"rates\":"),
        ),
        ("crossed bid ask", valid.replace("\"bp\":1.01", "\"bp\":2")),
        ("zero bid", valid.replace("\"bp\":1.01", "\"bp\":0")),
        ("negative ask", valid.replace("\"ap\":1.12", "\"ap\":-1")),
        (
            "string mid",
            valid.replace("\"mp\":1.07", "\"mp\":\"1.07\""),
        ),
        (
            "object bid",
            valid.replace(
                "\"bp\":1.01",
                r#""bp":{"$serde_json::private::Number":"1.01"}"#,
            ),
        ),
        (
            "object ask",
            valid.replace(
                "\"ap\":1.12",
                r#""ap":{"$serde_json::private::Number":"1.12"}"#,
            ),
        ),
        (
            "object mid",
            valid.replace(
                "\"mp\":1.07",
                r#""mp":{"$serde_json::private::Number":"1.07"}"#,
            ),
        ),
        (
            "escaped object bid",
            valid.replace(
                "\"bp\":1.01",
                r#""bp":{"\u0024serde_json::private::Number":"1.01"}"#,
            ),
        ),
        (
            "escaped object ask",
            valid.replace(
                "\"ap\":1.12",
                r#""ap":{"\u0024serde_json::private::Number":"1.12"}"#,
            ),
        ),
        (
            "escaped object mid",
            valid.replace(
                "\"mp\":1.07",
                r#""mp":{"\u0024serde_json::private::Number":"1.07"}"#,
            ),
        ),
        (
            "oversized decimal",
            valid.replace("\"mp\":1.07", "\"mp\":1e1000"),
        ),
        ("invalid timestamp", valid.replace(&t, "2026-99-99")),
        (
            "stale provider time",
            valid.replace(&t, &timestamp(now - time::Duration::seconds(31))),
        ),
        (
            "future provider time",
            valid.replace(&t, &timestamp(now + time::Duration::minutes(1))),
        ),
        ("oversized body", " ".repeat(512 * 1024 + 1)),
        (
            "secret reflection",
            valid.replace(
                "\"bp\":1.01",
                &format!("\"secret\":\"{}\",\"bp\":1.01", provider_fixtures::KEY),
            ),
        ),
    ];
    for (name, body) in faults {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = provider_fixtures::Vault::default();
        let source_account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
        let mut http = FxReadHttp {
            account: provider_fixtures::Http::default(),
            body: valid.as_bytes().to_vec(),
            paths: Default::default(),
        };
        connect_using(
            &mut control,
            &workspace,
            "trading212",
            "LIVE",
            &vault,
            &http,
        );
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let source = command(
            &mut control,
            "data.fx.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.fx.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":source_account["connectionId"]}),
        );
        let request = json!({"requestId":"fx-invalid","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
        let control = Arc::new(Mutex::new(control));
        let read =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(
            read["data"]["status"], "AVAILABLE",
            "Initial read for {name}: {read}"
        );
        http.body = body.into_bytes();
        let rejected =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(rejected["ok"], true, "{name}: {rejected}");
        assert_eq!(
            rejected["data"]["status"], "UNAVAILABLE",
            "{name}: {rejected}"
        );
        assert!(rejected["data"]["evidence"].is_null(), "{name}: {rejected}");
        assert!(rejected["data"]["observedAt"].is_null());
        assert_eq!(
            rejected["data"]["capabilityStatuses"][1]["status"],
            "BLOCKED_EXTERNAL"
        );
        assert!(http.account.trading212_posts.borrow().is_empty());
        assert!(http.account.trading212_delete_calls.borrow().is_empty());
        assert!(!rejected.to_string().contains(provider_fixtures::KEY));
    }
}

#[test]
#[cfg(feature = "integration-test")]
fn provider_age_expires_fx_before_a_recent_receipt_and_reopen_keeps_only_selection() {
    use std::sync::{Arc, Mutex};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let t = (time::OffsetDateTime::now_utc() - time::Duration::seconds(20))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let http = FxReadHttp {
        account: provider_fixtures::Http::default(),
        body: format!(r#"{{"rates":{{"EURUSD":{{"bp":1.01,"ap":1.12,"mp":1.07,"t":"{t}"}}}}}}"#)
            .into_bytes(),
        paths: Default::default(),
    };
    connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &http,
    );
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let read = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"fx-dual-age","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let mut locked = control.lock().unwrap();
    locked
        .advance_test_clock_fixture(workspace.as_str().unwrap(), 11_000)
        .unwrap();
    let stale = command(
        &mut locked,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        stale["data"]["status"], "UNAVAILABLE",
        "The provider rate is over30seconds even though the receipt is only11seconds old: {stale}"
    );
    assert_eq!(stale["data"]["evidence"], read["data"]["evidence"]);
    assert_eq!(stale["data"]["observedAt"], read["data"]["observedAt"]);
    drop(locked);
    drop(control);
    let mut reopened = ControlPlane::new(path);
    command(&mut reopened, "workspace.open", json!({}));
    let source = command(
        &mut reopened,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        source["data"]["connectionId"],
        saved["data"]["connectionId"]
    );
    assert_eq!(source["data"]["status"], "UNVERIFIED");
    assert!(source["data"]["evidence"].is_null());
    assert!(source["data"]["observedAt"].is_null());
}

#[test]
fn changed_fx_source_account_workspace_clock_or_sequence_discards_late_results() {
    use std::sync::{Arc, Mutex};
    struct RacingHttp<'a> {
        body: &'a FxReadHttp,
        change: std::cell::RefCell<Box<dyn FnMut() + 'a>>,
    }
    impl tradex::provider_io::ProviderHttp for RacingHttp<'_> {
        fn get(
            &self,
            e: tradex::provider_io::ProviderEndpoint,
            p: &str,
            h: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            (self.change.borrow_mut())();
            self.body.get(e, p, h)
        }
    }
    for boundary in ["source", "account", "workspace", "clock", "sequence"] {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = provider_fixtures::Vault::default();
        let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
        let t = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let http = FxReadHttp {
            account: provider_fixtures::Http::default(),
            body: format!(
                r#"{{"rates":{{"EURUSD":{{"bp":1.01,"ap":1.12,"mp":1.07,"t":"{t}"}}}}}}"#
            )
            .into_bytes(),
            paths: Default::default(),
        };
        connect_using(
            &mut control,
            &workspace,
            "trading212",
            "LIVE",
            &vault,
            &http,
        );
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let source = command(
            &mut control,
            "data.fx.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.fx.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
        );
        let control = Arc::new(Mutex::new(control));
        let request = json!({"requestId":"fx-race","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
        let racing = RacingHttp {
            body: &http,
            change: std::cell::RefCell::new(Box::new(|| {
                if boundary == "sequence" {
                    let newer = tradex::financial_sources::execute_refresh(
                        &control, &request, "main", &vault, &http,
                    );
                    assert_eq!(newer["data"]["status"], "AVAILABLE", "{newer}");
                    return;
                }
                let mut locked = control
                    .try_lock()
                    .expect("FX HTTP must not hold the Control Plane lock");
                match boundary {
                "source" => assert_eq!(command(&mut locked, "data.fx.disconnect", json!({"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}))["ok"], true),
                "account" => assert!(locked.prepare_provider(&json!({"requestId":"fx-account-change","schemaVersion":1,"command":"provider.disconnect","payload":{"workspaceId":workspace,"connectionId":account["connectionId"],"expectedStateVersion":account["stateVersion"]}})).unwrap().is_some()),
                "workspace" => assert_eq!(command(&mut locked, "workspace.open", json!({"path":directory.path().join("workspace")}))["ok"], true),
                "clock" => assert_eq!(command(&mut locked, "time.revalidate", json!({"workspaceId":workspace}))["ok"], true),
                _ => unreachable!(),
            }
            })),
        };
        let late =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &racing);
        assert_eq!(
            late["error"]["code"], "STATE_VERSION_CONFLICT",
            "{boundary}: {late}"
        );
        let source = command(
            &mut control.lock().unwrap(),
            "data.fx.connection",
            json!({"workspaceId":workspace}),
        );
        if boundary == "sequence" {
            assert_eq!(source["data"]["status"], "AVAILABLE");
        } else {
            assert_ne!(source["data"]["status"], "AVAILABLE");
            assert!(source["data"]["evidence"].is_null());
        }
    }
}

#[test]
fn fx_access_denials_stay_read_only_and_retry_respects_actual_provider_quota() {
    use std::sync::{Arc, Mutex};
    use tradex::provider_io::{
        ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse, ProviderRateLimit,
    };
    struct Responses {
        account: FxReadHttp,
        status: std::cell::Cell<u16>,
        reads: std::cell::Cell<usize>,
    }
    impl ProviderHttp for Responses {
        fn get(
            &self,
            e: ProviderEndpoint,
            p: &str,
            h: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            self.account.get(e, p, h)
        }
        fn request_with_rate_limit(
            &self,
            e: ProviderEndpoint,
            m: ProviderHttpMethod,
            p: &str,
            h: reqwest::header::HeaderMap,
            b: Option<&Value>,
        ) -> tradex::protocol::Result<(ProviderHttpResponse, Option<ProviderRateLimit>)> {
            if e != ProviderEndpoint::AlpacaMarketData {
                return self.account.request_with_rate_limit(e, m, p, h, b);
            }
            assert_eq!(e, ProviderEndpoint::AlpacaMarketData);
            assert_eq!(m, ProviderHttpMethod::Get);
            assert_eq!(p, "/v1beta1/forex/latest/rates?currency_pairs=EURUSD");
            assert!(b.is_none());
            assert_eq!(h["APCA-API-KEY-ID"], provider_fixtures::KEY);
            assert!(!h.contains_key("authorization"));
            self.reads.set(self.reads.get() + 1);
            let status = self.status.get();
            let t = (time::OffsetDateTime::now_utc() - time::Duration::seconds(1))
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap();
            let body = if status == 200 {
                format!(r#"{{"rates":{{"EURUSD":{{"bp":1.01,"ap":1.12,"mp":1.07,"t":"{t}"}}}}}}"#)
                    .into_bytes()
            } else {
                format!("untrusted diagnostic {}", provider_fixtures::SECRET).into_bytes()
            };
            Ok((
                ProviderHttpResponse { status, body },
                if status == 429 {
                    Some(ProviderRateLimit {
                        retry_after_seconds: Some(2),
                        ..Default::default()
                    })
                } else {
                    None
                },
            ))
        }
    }
    for (status, reason) in [
        (401, "Authentication failed"),
        (403, "provider denied access"),
        (429, "Provider read quota is unavailable"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(directory.path().join("workspace"));
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = provider_fixtures::Vault::default();
        let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
        let http = Responses {
            account: FxReadHttp {
                account: provider_fixtures::Http::default(),
                body: vec![],
                paths: Default::default(),
            },
            status: std::cell::Cell::new(status),
            reads: std::cell::Cell::new(0),
        };
        connect_using(
            &mut control,
            &workspace,
            "trading212",
            "LIVE",
            &vault,
            &http,
        );
        command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let source = command(
            &mut control,
            "data.fx.connection",
            json!({"workspaceId":workspace}),
        );
        let saved = command(
            &mut control,
            "data.fx.configure",
            json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
        );
        let control = Arc::new(Mutex::new(control));
        let request = json!({"requestId":"fx-denied","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}});
        let denied =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        assert_eq!(
            denied["data"]["status"], "UNAVAILABLE",
            "{status}: {denied}"
        );
        assert!(denied["data"]["evidence"].is_null());
        assert!(
            denied["data"]["availabilityReason"]
                .as_str()
                .unwrap()
                .contains(reason),
            "{status}: {denied}"
        );
        assert!(!denied.to_string().contains(provider_fixtures::SECRET));
        assert_eq!(http.reads.get(), 1);
        if status == 429 {
            http.status.set(200);
            let start = std::time::Instant::now();
            let retry = tradex::financial_sources::execute_refresh(
                &control, &request, "main", &vault, &http,
            );
            assert!(
                start.elapsed() >= std::time::Duration::from_millis(1900),
                "A retry must wait for actual Retry-After"
            );
            assert_eq!(retry["data"]["status"], "AVAILABLE", "{retry}");
            assert_eq!(
                retry["data"]["capabilityStatuses"][1]["status"],
                "BLOCKED_EXTERNAL"
            );
            assert_eq!(http.reads.get(), 2);
        }
        assert!(http.account.account.trading212_posts.borrow().is_empty());
        assert!(
            http.account
                .account
                .trading212_delete_calls
                .borrow()
                .is_empty()
        );
    }
}

#[test]
fn spot_buy_uses_actual_quote_asset_without_inventing_portfolio_currency_or_usdt_parity() {
    use std::sync::{Arc, Mutex, atomic::Ordering};
    struct PartialKeyScope(provider_fixtures::Http);
    impl tradex::provider_io::ProviderHttp for PartialKeyScope {
        fn get(
            &self,
            e: tradex::provider_io::ProviderEndpoint,
            p: &str,
            h: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            let bytes = self.0.get(e, p, h)?;
            if p == "/api/v2/spot/account/info" {
                let mut value: Value = serde_json::from_slice(&bytes).unwrap();
                value["data"].as_object_mut().unwrap().remove("authorities");
                return Ok(serde_json::to_vec(&value).unwrap());
            }
            Ok(bytes)
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account = connect(&mut control, &workspace, "alpaca", "PAPER", &vault);
    let spot = connect_using(
        &mut control,
        &workspace,
        "bitget",
        "LIVE",
        &vault,
        &PartialKeyScope(provider_fixtures::Http::default()),
    );
    let draft = command(
        &mut control,
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":{
            "accountId":spot["connectionId"],"venue":"BITGET","environment":"BITGET_LIVE","instrumentId":"crypto:BTC/USDT:spot","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"0.01"},"limitPrice":"20000","timeInForce":"GTC"
        }}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &mut control,
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    assert_eq!(
        proposal["data"]["estimatedNotionalCurrency"], "USDT",
        "{proposal}"
    );
    let requirements = command(
        &mut control,
        "data.fx.requirements",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    let rows = requirements["data"]["requirements"].as_array().unwrap();
    let policy = rows
        .iter()
        .find(|row| row["purpose"] == "INTENT_POLICY")
        .unwrap();
    assert_eq!(policy["fromCurrency"], "USDT");
    assert_eq!(policy["toCurrency"], "USD");
    assert_eq!(policy["need"], "EXTERNAL_RATE");
    assert!(policy["providerPair"].is_null());
    let funding = rows
        .iter()
        .find(|row| row["purpose"] == "INTENT_FUNDING")
        .unwrap();
    assert_eq!(funding["need"], "IDENTITY");
    assert_eq!(funding["toCurrency"], "USDT");
    assert!(rows.iter().any(|row| row["purpose"] == "ACCOUNT_WORKSPACE"
        && row["connectionId"] == spot["connectionId"]
        && row["need"] == "UNKNOWN_CURRENCY"));
    command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let source = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.fx.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}),
    );
    let control = Arc::new(Mutex::new(control));
    let no_key = CountingUnavailableVault::default();
    let http = provider_fixtures::Http::default();
    let result = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"unsupported-fx","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"],"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &no_key,
        &http,
    );
    assert_eq!(result["ok"], true, "{result}");
    assert_eq!(no_key.0.load(Ordering::SeqCst), 0);
    assert!(http.calls.borrow().is_empty());
    assert!(
        result["data"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("unknown or unsupported"),
        "{result}"
    );
    assert!(result["data"]["evidence"].is_null());
}

#[test]
fn sell_intent_does_not_require_buy_funding_fx_for_asset_capacity() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let http = FxReadHttp {
        account: provider_fixtures::Http::default(),
        body: vec![],
        paths: Default::default(),
    };
    let account = connect_using(
        &mut control,
        &workspace,
        "trading212",
        "LIVE",
        &vault,
        &http,
    );
    let draft = command(
        &mut control,
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":{
            "accountId":account["connectionId"],"venue":"XNAS","environment":"TRADING212_LIVE","instrumentId":"equity:US:AAPL","side":"SELL","orderType":"LIMIT","quantity":{"type":"BASE","value":"1"},"limitPrice":"200","timeInForce":"DAY"
        }}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &mut control,
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let result = command(
        &mut control,
        "data.fx.requirements",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert_eq!(result["ok"], true, "{result}");
    let rows = result["data"]["requirements"].as_array().unwrap();
    assert!(
        rows.iter().all(|row| row["purpose"] != "INTENT_FUNDING"),
        "Sell capacity is in the base asset; it must not acquire a Buy funding-currency route: {result}"
    );
    assert!(
        rows.iter()
            .any(|row| row["purpose"] == "INTENT_POLICY" && row["need"] == "IDENTITY")
    );
    assert!(
        rows.iter()
            .any(|row| row["purpose"] == "ACCOUNT_WORKSPACE" && row["providerPair"] == "EURUSD"),
        "Actually required portfolio conversions remain explicit"
    );
    assert!(http.account.trading212_posts.borrow().is_empty());
    assert!(http.account.trading212_delete_calls.borrow().is_empty());
}

#[test]
fn fx_commands_reject_renderer_assertions_before_vault_or_provider_work() {
    use std::sync::{Arc, Mutex, atomic::Ordering};
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let before = command(
        &mut control,
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    for (name, extra) in [
        (
            "data.fx.connection",
            json!({"connectionId":"caller-account"}),
        ),
        (
            "data.fx.requirements",
            json!({"fromCurrency":"EUR","toCurrency":"USD"}),
        ),
        (
            "data.fx.configure",
            json!({"expectedStateVersion":before["data"]["stateVersion"],"connectionId":"caller-account","endpoint":"https://example.com"}),
        ),
        (
            "data.fx.disconnect",
            json!({"expectedStateVersion":before["data"]["stateVersion"],"deleteCredentials":true}),
        ),
    ] {
        let mut input = json!({"workspaceId":workspace});
        input
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let rejected = command(&mut control, name, input);
        assert_eq!(
            rejected["error"]["code"], "IPC_PAYLOAD_INVALID",
            "{name}: {rejected}"
        );
    }
    let control = Arc::new(Mutex::new(control));
    let vault = CountingUnavailableVault::default();
    let http = provider_fixtures::Http::default();
    for extra in [
        json!({"pairs":["EURUSD"]}),
        json!({"rate":"1"}),
        json!({"providerTimestamp":"2026-10-04T12:00:00Z"}),
        json!({"quality":"VERIFIED"}),
        json!({"connectionId":"caller-account"}),
        json!({"credentialRef":"caller-reference"}),
        json!({"endpoint":"https://example.com"}),
    ] {
        let mut input =
            json!({"workspaceId":workspace,"expectedStateVersion":before["data"]["stateVersion"]});
        input
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let rejected = tradex::financial_sources::execute_refresh(
            &control,
            &json!({"requestId":"fx-caller-claims","schemaVersion":1,"command":"data.fx.refresh","payload":input}),
            "main",
            &vault,
            &http,
        );
        assert_eq!(
            rejected["error"]["code"], "IPC_PAYLOAD_INVALID",
            "{rejected}"
        );
    }
    let rejected = tradex::financial_sources::execute_refresh(
        &control,
        &json!({"requestId":"fx-untrusted-consumer","schemaVersion":1,"command":"data.fx.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":before["data"]["stateVersion"]}}),
        "untrusted-renderer",
        &vault,
        &http,
    );
    assert_eq!(rejected["error"]["code"], "IPC_ACCESS_DENIED");
    assert_eq!(vault.0.load(Ordering::SeqCst), 0);
    assert!(http.calls.borrow().is_empty());
    let after = command(
        &mut control.lock().unwrap(),
        "data.fx.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(before["data"], after["data"]);
}
