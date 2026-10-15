use serde_json::{Value, json};
use tradex::ControlPlane;
#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod provider_fixtures;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch_with_events(
        json!({"requestId":"calendar-test","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}

#[test]
fn user_selects_saved_paper_calendar_without_authority_and_reopens_metadata_only() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let http = provider_fixtures::Http::default();
    let tested_request = json!({"requestId":"calendar-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Calendar Paper key"}});
    let job = control.prepare_provider(&tested_request).unwrap().unwrap();
    let outcome = job.run(
        &vault,
        |_| provider_fixtures::credentials(),
        &http,
        || control.provider_job_current(&job),
    );
    let tested = control.complete_provider(&job, outcome);
    assert_eq!(tested["ok"], true, "{tested}");
    let account = command(
        &mut control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":true}),
    );
    assert_eq!(account["data"]["permissions"]["scope"], "UNVERIFIED");
    let source = command(
        &mut control,
        "data.calendar.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(source["ok"], true, "{source}");
    assert_eq!(source["data"]["configured"], false);
    assert_eq!(
        source["data"]["eligibleAccounts"][0]["connectionId"],
        account["data"]["connectionId"]
    );
    let before = http.calls.borrow().clone();
    let saved = command(
        &mut control,
        "data.calendar.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["data"]["connectionId"]}),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(saved["data"]["configured"], true);
    assert_eq!(saved["data"]["status"], "UNVERIFIED");
    assert_eq!(
        *http.calls.borrow(),
        before,
        "Saving selection does not read the provider"
    );
    let stale = command(
        &mut control,
        "data.calendar.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["data"]["connectionId"]}),
    );
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    for (field, value) in [
        ("secret", json!(provider_fixtures::SECRET)),
        ("providerUrl", json!("https://example.invalid")),
        ("permissionScope", json!("VERIFIED")),
        ("environment", json!("LIVE")),
    ] {
        let mut payload = json!({"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"],"connectionId":account["data"]["connectionId"]});
        payload[field] = value;
        let rejected = command(&mut control, "data.calendar.configure", payload);
        assert_eq!(
            rejected["error"]["code"], "IPC_PAYLOAD_INVALID",
            "Caller-supplied authority/destinations/secrets must be rejected: {rejected}"
        );
        assert!(!rejected.to_string().contains(provider_fixtures::SECRET));
    }
    assert_eq!(
        *http.calls.borrow(),
        before,
        "Rejected selection requests do not read any provider"
    );
    drop(control);
    let mut reopened = ControlPlane::new(path);
    assert_eq!(
        command(&mut reopened, "workspace.open", json!({}))["ok"],
        true
    );
    let restored = command(
        &mut reopened,
        "data.calendar.connection",
        json!({"workspaceId":workspace}),
    );
    assert_eq!(
        restored["data"]["connectionId"],
        account["data"]["connectionId"]
    );
    assert_eq!(restored["data"]["status"], "UNVERIFIED");
    let disconnected = command(
        &mut reopened,
        "data.calendar.disconnect",
        json!({"workspaceId":workspace,"expectedStateVersion":restored["data"]["stateVersion"]}),
    );
    assert_eq!(disconnected["data"]["configured"], false);
    assert_eq!(
        vault.present.borrow().len(),
        1,
        "Calendar disconnect keeps the borrowed account key"
    );
    let listed = command(
        &mut reopened,
        "account.get",
        json!({"workspaceId":workspace,"connectionId":account["data"]["connectionId"]}),
    );
    assert_eq!(listed["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(listed["data"]["health"]["arming"], "NOT_APPLICABLE");
    for reply in [saved, restored, disconnected] {
        for secret in [provider_fixtures::KEY, provider_fixtures::SECRET] {
            assert!(!reply.to_string().contains(secret));
        }
    }
}

#[test]
fn authenticated_calendar_reaches_market_without_making_other_capabilities_ready() {
    use reqwest::header::HeaderMap;
    use std::sync::{Arc, Mutex};
    use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
    use tradex::protocol::Result;
    use tradex::provider_io::{ProviderEndpoint, ProviderHttp};
    struct CalendarHttp {
        body: std::cell::RefCell<Vec<u8>>,
        calls: std::cell::Cell<usize>,
    }
    impl ProviderHttp for CalendarHttp {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: HeaderMap,
        ) -> Result<Vec<u8>> {
            assert_eq!(endpoint, ProviderEndpoint::AlpacaPaper);
            assert!(path.starts_with("/v3/calendar/XNAS?start="));
            assert!(path.ends_with("&timezone=UTC"));
            assert_eq!(headers["APCA-API-KEY-ID"], provider_fixtures::KEY);
            self.calls.set(self.calls.get() + 1);
            Ok(self.body.borrow().clone())
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let workspace =
        command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
    let vault = provider_fixtures::Vault::default();
    let account_http = provider_fixtures::Http::default();
    let request = json!({"requestId":"calendar-account-read","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Calendar only"}});
    let job = control.prepare_provider(&request).unwrap().unwrap();
    let result = job.run(
        &vault,
        |_| provider_fixtures::credentials(),
        &account_http,
        || control.provider_job_current(&job),
    );
    let tested = control.complete_provider(&job, result);
    let account = command(
        &mut control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":true}),
    );
    let live_request = json!({"requestId":"calendar-live-read","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"trading212","environment":"LIVE","label":"Read-only Live account"}});
    let live_job = control.prepare_provider(&live_request).unwrap().unwrap();
    let live_result = live_job.run(
        &vault,
        |_| provider_fixtures::credentials(),
        &account_http,
        || control.provider_job_current(&live_job),
    );
    let live_tested = control.complete_provider(&live_job, live_result);
    assert_eq!(live_tested["ok"], true, "{live_tested}");
    let live = command(
        &mut control,
        "provider.connect",
        json!({"step":"confirm","workspaceId":workspace,"connectionId":live_tested["data"]["connectionId"],"expectedStateVersion":live_tested["data"]["stateVersion"],"acknowledgeUnverified":true}),
    );
    assert_eq!(live["data"]["permissions"]["scope"], "UNVERIFIED");
    assert_eq!(live["data"]["health"]["arming"], "DISARMED");
    let source = command(
        &mut control,
        "data.calendar.connection",
        json!({"workspaceId":workspace}),
    );
    let saved = command(
        &mut control,
        "data.calendar.configure",
        json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["data"]["connectionId"]}),
    );
    let now = command(
        &mut control,
        "time.revalidate",
        json!({"workspaceId":workspace}),
    );
    let now = OffsetDateTime::parse(now["data"]["wallClock"].as_str().unwrap(), &Rfc3339).unwrap();
    let midnight = now.date().midnight().assume_utc();
    let last_in_day =
        (now.date() + Duration::days(1)).midnight().assume_utc() - Duration::nanoseconds(1);
    let core_start = (now - Duration::seconds(10))
        .max(midnight)
        .format(&Rfc3339)
        .unwrap();
    let core_end = (now + Duration::seconds(20))
        .min(last_in_day)
        .format(&Rfc3339)
        .unwrap();
    let next_date = now.date() + Duration::days(1);
    let next_start = next_date
        .with_hms(13, 30, 0)
        .unwrap()
        .assume_utc()
        .format(&Rfc3339)
        .unwrap();
    let next_end = next_date
        .with_hms(20, 0, 0)
        .unwrap()
        .assume_utc()
        .format(&Rfc3339)
        .unwrap();
    let http=CalendarHttp {body:std::cell::RefCell::new(serde_json::to_vec(&json!({"market":{"mic":"XNAS","acronym":"NASDAQ","name":"NASDAQ","timezone":"America/New_York"},"calendar":[{"date":now.date().to_string(),"core_start":core_start,"core_end":core_end},{"date":(now+Duration::days(1)).date().to_string(),"core_start":next_start,"core_end":next_end}]})).unwrap()), calls:std::cell::Cell::new(0)};
    let control = Arc::new(Mutex::new(control));
    let read = tradex::calendar_source::execute_refresh(
        &control,
        &json!({"requestId":"calendar-refresh","schemaVersion":1,"command":"data.calendar.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(read["data"]["status"], "AVAILABLE", "{read}");
    let market = command(
        &mut control.lock().unwrap(),
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
    );
    assert_eq!(market["data"]["marketState"]["session"], "OPEN", "{market}");
    assert_eq!(market["data"]["marketState"]["nextClose"], core_end);
    assert_eq!(
        market["data"]["marketState"]["observedAt"],
        read["data"]["observedAt"]
    );
    assert!(market["data"]["marketState"]["providerTime"].is_null());
    assert_eq!(market["data"]["adjustmentStatus"], "UNAVAILABLE");
    assert!(market["data"]["instrumentState"].is_null());
    assert_eq!(http.calls.get(), 1);
    let repeated = command(
        &mut control.lock().unwrap(),
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"equity:US:MSFT","tier":"CENSUS"}),
    );
    assert_eq!(
        repeated["data"]["marketState"]["observedAt"],
        market["data"]["marketState"]["observedAt"]
    );
    let catalog = command(
        &mut control.lock().unwrap(),
        "data.source.catalog",
        json!({"workspaceId":workspace}),
    );
    let source = catalog["data"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["sourceId"] == "OD-005")
        .unwrap();
    assert_eq!(
        source["status"], "UNVERIFIED",
        "Only calendar capability is checked; combined source must stay unverified"
    );
    assert!(
        market["data"]["snapshot"].is_null(),
        "Explicit calendar configuration cannot inject a legacy quote fixture"
    );
    let draft = command(
        &mut control.lock().unwrap(),
        "trade.save_draft",
        json!({"workspaceId":workspace,"fields":{"accountId":live["data"]["connectionId"],"venue":"XNAS","environment":"TRADING212_LIVE","instrumentId":"equity:US:AAPL","side":"BUY","orderType":"LIMIT","quantity":{"type":"BASE","value":"1"},"limitPrice":"200","timeInForce":"DAY"}}),
    );
    assert_eq!(draft["ok"], true, "{draft}");
    let proposal = command(
        &mut control.lock().unwrap(),
        "trade.generate_proposal",
        json!({"workspaceId":workspace,"draftId":draft["data"]["draftId"],"expectedDraftVersion":1}),
    );
    assert_eq!(proposal["ok"], true, "{proposal}");
    let review = command(
        &mut control.lock().unwrap(),
        "trade.request_approval",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert_eq!(review["ok"], true, "{review}");
    assert_eq!(
        review["data"]["market"]["marketState"]["observedAt"],
        read["data"]["observedAt"]
    );
    let checks = review["data"]["riskDecision"]["checks"].as_array().unwrap();
    let calendar_check = checks
        .iter()
        .find(|check| check["checkId"] == "MARKET_SESSION")
        .unwrap();
    assert_eq!(
        calendar_check["outcome"], "PASS",
        "Only the matching calendar gate should pass: {review}"
    );
    assert_eq!(review["data"]["eligible"], false);
    let blocked = command(
        &mut control.lock().unwrap(),
        "trade.approve",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"],"proposalHash":proposal["data"]["proposalHash"],"reviewedRiskDecisionId":review["data"]["riskDecision"]["decisionId"],"reviewDigest":review["data"]["reviewDigest"],"expectedStateVersion":proposal["data"]["stateVersion"]}),
    );
    assert_eq!(
        blocked["ok"], false,
        "Calendar does not create financial authority: {blocked}"
    );
    let approvals = command(
        &mut control.lock().unwrap(),
        "trade.approval.list",
        json!({"workspaceId":workspace,"proposalId":proposal["data"]["proposalId"]}),
    );
    assert!(
        approvals["data"]["approvals"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(account_http.trading212_posts.borrow().is_empty());
    assert!(account_http.trading212_delete_calls.borrow().is_empty());

    #[cfg(feature = "integration-test")]
    {
        control
            .lock()
            .unwrap()
            .advance_test_clock_fixture(workspace.as_str().unwrap(), 20_001)
            .unwrap();
        let closed = command(
            &mut control.lock().unwrap(),
            "market.get",
            json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
        );
        assert_eq!(
            closed["data"]["marketState"]["session"], "CLOSED",
            "The provider's early-close boundary is exclusive: {closed}"
        );
        assert_eq!(
            closed["data"]["marketState"]["observedAt"],
            read["data"]["observedAt"]
        );
        control
            .lock()
            .unwrap()
            .advance_test_clock_fixture(workspace.as_str().unwrap(), 11_000)
            .unwrap();
        let expired = command(
            &mut control.lock().unwrap(),
            "market.get",
            json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
        );
        assert_eq!(expired["data"]["marketState"]["session"], "UNKNOWN");
        assert_eq!(
            expired["data"]["marketState"]["sourceStatus"],
            "UNAVAILABLE"
        );
        assert_eq!(
            expired["data"]["marketState"]["observedAt"], read["data"]["observedAt"],
            "Projection does not refresh calendar age"
        );
        command(
            &mut control.lock().unwrap(),
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let old = command(
            &mut control.lock().unwrap(),
            "data.calendar.connection",
            json!({"workspaceId":workspace}),
        );
        assert_eq!(
            old["data"]["status"], "UNAVAILABLE",
            "Clock revalidation cannot revive previous evidence"
        );
        let refreshed = tradex::calendar_source::execute_refresh(
            &control,
            &json!({"requestId":"calendar-new-clock","schemaVersion":1,"command":"data.calendar.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
            "main",
            &vault,
            &http,
        );
        assert_eq!(refreshed["data"]["status"], "AVAILABLE", "{refreshed}");
        assert_ne!(
            refreshed["data"]["calendarVersion"], read["data"]["calendarVersion"],
            "New clock-bound evidence cannot preserve old review identity"
        );
    }

    let mut wrong: Value = serde_json::from_slice(&http.body.borrow()).unwrap();
    wrong["market"]["mic"] = json!("XNYS");
    *http.body.borrow_mut() = serde_json::to_vec(&wrong).unwrap();
    let failed = tradex::calendar_source::execute_refresh(
        &control,
        &json!({"requestId":"wrong-calendar","schemaVersion":1,"command":"data.calendar.refresh","payload":{"workspaceId":workspace,"expectedStateVersion":saved["data"]["stateVersion"]}}),
        "main",
        &vault,
        &http,
    );
    assert_eq!(failed["data"]["status"], "UNAVAILABLE", "{failed}");
    let rejected = command(
        &mut control.lock().unwrap(),
        "market.get",
        json!({"workspaceId":workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
    );
    assert_eq!(rejected["data"]["marketState"]["session"], "UNKNOWN");
    assert_eq!(
        rejected["data"]["marketState"]["sourceStatus"], "UNAVAILABLE",
        "A failed calendar read must not look merely unchecked: {rejected}"
    );
}

#[test]
#[cfg(feature = "integration-test")]
fn legacy_opt_ins_cannot_override_explicit_calendar_or_grant_live_authority() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "authenticated_calendar_reaches_market_without_making_other_capabilities_ready",
            "--exact",
        ])
        .env("TRADEX_LIVE_APPROVAL_FIXTURE", "1")
        .env("TRADEX_MARKET_FIXTURE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "The public calendar/risk/read-only account scenario failed with legacy fixture opt-ins: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

struct Scenario {
    _directory: tempfile::TempDir,
    path: std::path::PathBuf,
    control: std::sync::Arc<std::sync::Mutex<ControlPlane>>,
    vault: provider_fixtures::Vault,
    workspace: Value,
    account: Value,
    source: Value,
    now: time::OffsetDateTime,
}
impl Scenario {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace");
        let mut control = ControlPlane::new(path.clone());
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = provider_fixtures::Vault::default();
        let http = provider_fixtures::Http::default();
        let request = json!({"requestId":"normal-paper","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"alpaca","environment":"PAPER","label":"Calendar test account"}});
        let job = control.prepare_provider(&request).unwrap().unwrap();
        let result = job.run(
            &vault,
            |_| provider_fixtures::credentials(),
            &http,
            || control.provider_job_current(&job),
        );
        let tested = control.complete_provider(&job, result);
        assert_eq!(tested["ok"], true, "{tested}");
        let account = command(&mut control, "provider.connect", json!({"step":"confirm","workspaceId":workspace,"connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":true}))["data"].clone();
        let source = command(
            &mut control,
            "data.calendar.connection",
            json!({"workspaceId":workspace}),
        );
        let source = command(&mut control, "data.calendar.configure", json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":account["connectionId"]}))["data"].clone();
        let now = command(
            &mut control,
            "time.revalidate",
            json!({"workspaceId":workspace}),
        );
        let now = time::OffsetDateTime::parse(
            now["data"]["wallClock"].as_str().unwrap(),
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        Self {
            _directory: directory,
            path,
            control: std::sync::Arc::new(std::sync::Mutex::new(control)),
            vault,
            workspace,
            account,
            source,
            now,
        }
    }
    fn refresh(&self, http: &impl tradex::provider_io::ProviderHttp) -> Value {
        tradex::calendar_source::execute_refresh(
            &self.control,
            &json!({"requestId":"calendar-case","schemaVersion":1,"command":"data.calendar.refresh","payload":{"workspaceId":self.workspace,"expectedStateVersion":self.source["stateVersion"]}}),
            "main",
            &self.vault,
            http,
        )
    }
    fn market(&self) -> Value {
        command(
            &mut self.control.lock().unwrap(),
            "market.get",
            json!({"workspaceId":self.workspace,"instrumentId":"equity:US:AAPL","tier":"CENSUS"}),
        )["data"]
            .clone()
    }
}
fn frame(days: Vec<Value>) -> Value {
    json!({"market":{"mic":"XNAS","acronym":"NASDAQ","name":"NASDAQ","timezone":"America/New_York"},"calendar":days})
}
fn day(date: time::Date, start: &str, end: &str) -> Value {
    json!({"date":date.to_string(),"core_start":format!("{date}T{start}Z"),"core_end":format!("{date}T{end}Z")})
}
struct ResponseHttp {
    body: Vec<u8>,
    error: Option<&'static str>,
}
impl ResponseHttp {
    fn json(value: &Value) -> Self {
        Self {
            body: serde_json::to_vec(value).unwrap(),
            error: None,
        }
    }
}
impl tradex::provider_io::ProviderHttp for ResponseHttp {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        assert_eq!(endpoint, tradex::provider_io::ProviderEndpoint::AlpacaPaper);
        assert!(path.starts_with("/v3/calendar/XNAS?start="));
        assert!(path.ends_with("&timezone=UTC"));
        assert!(headers["APCA-API-KEY-ID"].is_sensitive());
        assert!(headers["APCA-API-SECRET-KEY"].is_sensitive());
        match self.error {
            Some(code) => Err(tradex::protocol::TradeXError::new(code)),
            None => Ok(self.body.clone()),
        }
    }
}

#[test]
fn malformed_and_failed_provider_reads_retire_prior_calendar() {
    let scenario = Scenario::new();
    let tomorrow = scenario.now.date() + time::Duration::days(1);
    let valid = frame(vec![day(tomorrow, "13:30:00", "20:00:00")]);
    let mut faults = Vec::new();
    for (path, value) in [
        ("/market/mic", json!("XNYS")),
        ("/market/timezone", json!("UTC")),
        (
            "/calendar/0/core_end",
            json!(format!("{tomorrow}T13:30:00Z")),
        ),
        (
            "/calendar/0/core_start",
            json!(format!("{tomorrow}T13:30:00-04:00")),
        ),
        ("/calendar/0/date", json!("2026-02-30")),
        (
            "/calendar/0/core_start",
            json!(format!("{}T13:30:00Z", tomorrow + time::Duration::days(1))),
        ),
    ] {
        let mut fault = valid.clone();
        *fault.pointer_mut(path).unwrap() = value;
        faults.push(ResponseHttp::json(&fault));
    }
    for (field, value) in [
        ("pre_start", json!(format!("{tomorrow}T08:00:00Z"))),
        ("post_start", json!(format!("{tomorrow}T20:00:00Z"))),
        ("settlement_date", json!("invalid")),
        ("unknown_authority", json!("VERIFIED")),
    ] {
        let mut fault = valid.clone();
        fault["calendar"][0][field] = value;
        faults.push(ResponseHttp::json(&fault));
    }
    let duplicate = frame(vec![
        valid["calendar"][0].clone(),
        valid["calendar"][0].clone(),
    ]);
    faults.push(ResponseHttp::json(&duplicate));
    faults.push(ResponseHttp::json(&frame(vec![])));
    faults.push(ResponseHttp::json(&frame(vec![day(
        scenario.now.date() - time::Duration::days(2),
        "13:30:00",
        "20:00:00",
    )])));
    faults.push(ResponseHttp::json(&frame(vec![day(
        scenario.now.date() - time::Duration::days(1),
        "13:30:00",
        "20:00:00",
    )])));
    faults.push(ResponseHttp {
        body: vec![b' '; 256 * 1024 + 1],
        error: None,
    });
    faults.push(ResponseHttp {
        body: b"{broken".to_vec(),
        error: None,
    });
    for code in [
        "PROVIDER_AUTH_FAILED",
        "PROVIDER_RATE_LIMITED",
        "PROVIDER_UNAVAILABLE",
    ] {
        faults.push(ResponseHttp {
            body: vec![],
            error: Some(code),
        });
    }
    for (index, fault) in faults.iter().enumerate() {
        assert_eq!(
            scenario.refresh(&ResponseHttp::json(&valid))["data"]["status"],
            "AVAILABLE"
        );
        let failed = scenario.refresh(fault);
        assert_eq!(
            failed["data"]["status"], "UNAVAILABLE",
            "fault {index}: {failed}"
        );
        assert!(failed["data"]["calendarVersion"].is_null());
        let market = scenario.market();
        assert_eq!(
            market["marketState"]["session"], "UNKNOWN",
            "fault {index}: {market}"
        );
        assert_eq!(market["marketState"]["sourceStatus"], "UNAVAILABLE");
        assert!(market["marketState"]["calendarVersion"].is_null());
    }
}

#[test]
fn public_source_account_workspace_and_clock_changes_discard_late_http_result() {
    struct RacingHttp<'a> {
        response: ResponseHttp,
        action: std::cell::RefCell<Box<dyn FnMut() + 'a>>,
    }
    impl tradex::provider_io::ProviderHttp for RacingHttp<'_> {
        fn get(
            &self,
            endpoint: tradex::provider_io::ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> tradex::protocol::Result<Vec<u8>> {
            (self.action.borrow_mut())();
            self.response.get(endpoint, path, headers)
        }
    }
    for boundary in ["source", "account", "workspace", "clock"] {
        let scenario = Scenario::new();
        let value = frame(vec![day(
            scenario.now.date() + time::Duration::days(1),
            "13:30:00",
            "20:00:00",
        )]);
        let http = RacingHttp {
            response: ResponseHttp::json(&value),
            action: std::cell::RefCell::new(Box::new(|| {
                let mut control = scenario
                    .control
                    .try_lock()
                    .expect("Provider IO must release the Control Plane lock");
                match boundary {
                    "source" => {
                        let reply = command(
                            &mut control,
                            "data.calendar.disconnect",
                            json!({"workspaceId":scenario.workspace,"expectedStateVersion":scenario.source["stateVersion"]}),
                        );
                        assert_eq!(reply["ok"], true);
                    }
                    "account" => {
                        let job = control.prepare_provider(&json!({"requestId":"disconnect-race","schemaVersion":1,"command":"provider.disconnect","payload":{"workspaceId":scenario.workspace,"connectionId":scenario.account["connectionId"],"expectedStateVersion":scenario.account["stateVersion"]}})).unwrap();
                        assert!(job.is_some());
                    }
                    "workspace" => {
                        let reply = command(
                            &mut control,
                            "workspace.open",
                            json!({"path":scenario.path}),
                        );
                        assert_eq!(reply["ok"], true);
                    }
                    "clock" => {
                        let reply = command(
                            &mut control,
                            "time.revalidate",
                            json!({"workspaceId":scenario.workspace}),
                        );
                        assert_eq!(reply["ok"], true);
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
            "data.calendar.connection",
            json!({"workspaceId":scenario.workspace}),
        );
        assert_ne!(source["data"]["status"], "AVAILABLE");
        assert!(
            source["data"]["calendarVersion"].is_null(),
            "{boundary}: {source}"
        );
        assert_ne!(
            scenario.market()["marketState"]["sourceStatus"],
            "AVAILABLE"
        );
    }
}

#[test]
#[cfg(feature = "integration-test")]
fn actual_utc_bounds_drive_holiday_extended_early_close_and_dst_without_weekday_rules() {
    use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
    fn advance(scenario: &Scenario, target: OffsetDateTime) {
        let mut control = scenario.control.lock().unwrap();
        let now = command(
            &mut control,
            "time.status",
            json!({"workspaceId":scenario.workspace}),
        );
        let now =
            OffsetDateTime::parse(now["data"]["wallClock"].as_str().unwrap(), &Rfc3339).unwrap();
        let mut elapsed = (target - now).whole_milliseconds().max(0) as u64;
        while elapsed > 0 {
            let step = elapsed.min(600_000);
            control
                .advance_test_clock_fixture(scenario.workspace.as_str().unwrap(), step)
                .unwrap();
            elapsed -= step;
        }
    }
    let scenario = Scenario::new();
    // Deliberately no Saturday row; Monday's authenticated bounds are after the DST fallback.
    // These fixture dates are external time/HTTP inputs, never internal session projections.
    let fallback = |year| {
        let mut sunday = time::Date::from_calendar_date(year, time::Month::November, 1).unwrap();
        while sunday.weekday() != time::Weekday::Sunday {
            sunday += Duration::days(1);
        }
        (sunday - Duration::days(1))
            .with_hms(12, 0, 0)
            .unwrap()
            .assume_utc()
    };
    let mut saturday = fallback(scenario.now.year());
    if scenario.now >= saturday {
        saturday = fallback(scenario.now.year() + 1);
    }
    advance(&scenario, saturday);
    let monday = saturday.date() + Duration::days(2);
    let mut early = day(monday, "14:30:00", "18:00:00");
    early["pre_start"] = json!(format!("{monday}T09:00:00Z"));
    early["pre_end"] = json!(format!("{monday}T14:30:00Z"));
    early["post_start"] = json!(format!("{monday}T18:00:00Z"));
    early["post_end"] = json!(format!("{monday}T22:00:00Z"));
    let external = ResponseHttp::json(&frame(vec![
        day(saturday.date() - Duration::days(1), "13:30:00", "20:00:00"),
        early.clone(),
        day(monday + Duration::days(1), "14:30:00", "21:00:00"),
    ]));
    assert_eq!(scenario.refresh(&external)["data"]["status"], "AVAILABLE");
    let market = scenario.market();
    assert_eq!(market["marketState"]["session"], "CLOSED");
    assert_eq!(
        market["marketState"]["nextOpen"],
        format!("{monday}T14:30:00Z")
    );
    for (clock, expected) in [
        ("09:00:00", "EXTENDED_HOURS"),
        ("14:29:59", "EXTENDED_HOURS"),
        ("14:30:00", "OPEN"),
        ("17:59:59", "OPEN"),
        ("18:00:00", "EXTENDED_HOURS"),
        ("22:00:00", "CLOSED"),
    ] {
        advance(
            &scenario,
            OffsetDateTime::parse(&format!("{monday}T{clock}Z"), &Rfc3339).unwrap(),
        );
        let external = ResponseHttp::json(&frame(vec![
            early.clone(),
            day(monday + Duration::days(1), "14:30:00", "21:00:00"),
        ]));
        assert_eq!(scenario.refresh(&external)["data"]["status"], "AVAILABLE");
        let market = scenario.market();
        assert_eq!(
            market["marketState"]["session"], expected,
            "{clock}: {market}"
        );
        if expected == "OPEN" {
            assert_eq!(
                market["marketState"]["nextClose"],
                format!("{monday}T18:00:00Z")
            );
        }
    }
}

#[test]
fn vault_read_releases_control_lock_and_account_disconnect_prevents_http() {
    use tradex::provider_io::{CredentialVault, Credentials};
    struct RacingVault<'a> {
        scenario: &'a Scenario,
    }
    impl CredentialVault for RacingVault<'_> {
        fn put(&self, _: &str, _: &Credentials) -> tradex::protocol::Result<()> {
            unreachable!()
        }
        fn remove(&self, _: &str) -> tradex::protocol::Result<()> {
            unreachable!()
        }
        fn get(&self, reference: &str) -> tradex::protocol::Result<Credentials> {
            let mut control = self
                .scenario
                .control
                .try_lock()
                .expect("Vault IO must release the Control Plane lock");
            control.prepare_provider(&json!({"requestId":"vault-race","schemaVersion":1,"command":"provider.disconnect","payload":{"workspaceId":self.scenario.workspace,"connectionId":self.scenario.account["connectionId"],"expectedStateVersion":self.scenario.account["stateVersion"]}}))?.unwrap();
            self.scenario.vault.get(reference)
        }
    }
    let scenario = Scenario::new();
    let http = provider_fixtures::Http::default();
    let reply = tradex::calendar_source::execute_refresh(
        &scenario.control,
        &json!({"requestId":"read-vault-race","schemaVersion":1,"command":"data.calendar.refresh","payload":{"workspaceId":scenario.workspace,"expectedStateVersion":scenario.source["stateVersion"]}}),
        "main",
        &RacingVault {
            scenario: &scenario,
        },
        &http,
    );
    assert_eq!(reply["error"]["code"], "STATE_VERSION_CONFLICT");
    assert!(
        http.calls.borrow().is_empty(),
        "Stale vault results must not reach HTTP"
    );
    assert_ne!(
        scenario.market()["marketState"]["sourceStatus"],
        "AVAILABLE"
    );
}

#[test]
#[cfg(feature = "integration-test")]
fn real_http_boundary_preserves_fixed_read_destination_and_fails_closed() {
    use std::io::{BufRead, BufReader, Write};
    use tradex::provider_io::{BrokerHttp, ProviderEndpoint, ProviderHttp, ProviderHttpMethod};
    let scenario = Scenario::new();
    let tomorrow = scenario.now.date() + time::Duration::days(1);
    let valid = serde_json::to_vec(&frame(vec![day(tomorrow, "13:30:00", "20:00:00")])).unwrap();
    for mode in [
        "success",
        "auth",
        "quota",
        "redirect",
        "oversize",
        "disconnect",
        "timeout",
    ] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let destination = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        destination.set_nonblocking(true).unwrap();
        let redirect = destination.local_addr().unwrap();
        let body = valid.clone();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(socket.try_clone().unwrap());
            let mut request = String::new();
            reader.read_line(&mut request).unwrap();
            assert!(request.starts_with("GET /v3/calendar/XNAS?start="));
            assert!(request.ends_with("&timezone=UTC HTTP/1.1\r\n"));
            let mut headers = String::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                headers.push_str(&line);
            }
            assert!(headers.contains(provider_fixtures::KEY));
            assert!(headers.contains(provider_fixtures::SECRET));
            let status = match mode {
                "auth" => 401,
                "quota" => 429,
                "redirect" => 302,
                _ => 200,
            };
            if mode == "disconnect" {
                return;
            }
            let length = if mode == "oversize" {
                3 * 1024 * 1024
            } else {
                body.len()
            };
            let location = if mode == "redirect" {
                format!("Location: http://{redirect}/stolen\r\n")
            } else {
                String::new()
            };
            write!(socket, "HTTP/1.1 {status} Response\r\nContent-Length: {length}\r\nConnection: close\r\n{location}\r\n").unwrap();
            if mode == "timeout" {
                std::thread::sleep(std::time::Duration::from_secs(13));
                return;
            }
            if mode != "oversize" {
                socket.write_all(&body).unwrap();
            }
        });
        let http = BrokerHttp::for_loopback_test(&format!("http://{address}")).unwrap();
        let reply = scenario.refresh(&http);
        server.join().unwrap();
        assert_eq!(
            reply["data"]["status"],
            if mode == "success" {
                "AVAILABLE"
            } else {
                "UNAVAILABLE"
            },
            "{mode}: {reply}"
        );
        assert!(
            destination.accept().is_err(),
            "Redirect target must never receive headers"
        );
        for method in [ProviderHttpMethod::Post, ProviderHttpMethod::Delete] {
            let path = format!(
                "/v3/calendar/XNAS?start={}&end={}&timezone=UTC",
                scenario.now.date() - time::Duration::days(1),
                scenario.now.date() + time::Duration::days(14)
            );
            assert_eq!(
                http.request(
                    ProviderEndpoint::AlpacaPaper,
                    method,
                    &path,
                    Default::default(),
                    None
                )
                .err()
                .unwrap()
                .code,
                "PROVIDER_UNSUPPORTED"
            );
        }
        for path in [
            "https://evil.invalid/v3/calendar/XNAS",
            "/v3/calendar/XNYS?start=2026-10-01&end=2026-10-16&timezone=UTC",
            "/v3/calendar/XNAS?start=2026-10-01&end=2026-10-17&timezone=UTC",
            "/v3/calendar/XNAS?start=2026-10-01&end=2026-10-16&timezone=UTC&host=evil",
        ] {
            assert_eq!(
                http.get(ProviderEndpoint::AlpacaPaper, path, Default::default())
                    .unwrap_err()
                    .code,
                "PROVIDER_UNSUPPORTED"
            );
        }
    }
}

#[test]
fn calendar_replacement_removes_combined_source_probe_residue() {
    let scenario = Scenario::new();
    let mut control = scenario.control.lock().unwrap();
    let catalog = command(
        &mut control,
        "data.source.catalog",
        json!({"workspaceId":scenario.workspace}),
    );
    let probed = command(
        &mut control,
        "data.source.probe",
        json!({"workspaceId":scenario.workspace,"sourceId":"OD-005","expectedStateVersion":catalog["data"]["stateVersion"]}),
    );
    assert_eq!(probed["ok"], true, "{probed}");
    let replaced = command(
        &mut control,
        "data.calendar.configure",
        json!({"workspaceId":scenario.workspace,"expectedStateVersion":scenario.source["stateVersion"],"connectionId":scenario.account["connectionId"]}),
    );
    assert_eq!(replaced["ok"], true, "{replaced}");
    let catalog = command(
        &mut control,
        "data.source.catalog",
        json!({"workspaceId":scenario.workspace}),
    );
    let source = catalog["data"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|source| source["sourceId"] == "OD-005")
        .unwrap();
    assert!(
        source["checkedAt"].is_null(),
        "Source replacement cannot retain a previous combined probe: {source}"
    );
    assert!(source["observedAt"].is_null());
    assert_eq!(source["status"], "UNVERIFIED");
    assert!(replaced["data"]["observedAt"].is_null());
}
