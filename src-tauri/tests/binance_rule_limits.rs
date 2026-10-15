use serde_json::{Value, json};
use std::sync::{Arc, Condvar, Mutex};
use tradex::{
    ControlPlane,
    protocol::Result,
    provider_io::{CredentialVault, Credentials},
};
#[path = "support/provider_fixtures.rs"]
#[allow(dead_code)]
mod fixtures;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch_with_events(
        json!({"requestId":"rule-limits","schemaVersion":1,"command":name,"payload":payload}),
        "main",
        None,
    )
}
struct ReadSource {
    _folder: tempfile::TempDir,
    control: Arc<Mutex<ControlPlane>>,
    workspace: Value,
    vault: fixtures::Vault,
    http: fixtures::Http,
}
impl ReadSource {
    fn new(uid: u64) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let mut control = ControlPlane::new(folder.path().into());
        let workspace =
            command(&mut control, "workspace.open", json!({}))["data"]["workspaceId"].clone();
        let vault = fixtures::Vault::default();
        let mut http = fixtures::Http::default();
        http.binance_rules_ui = true;
        http.binance_uid.set(uid);
        let request = json!({"requestId":"limited-account","schemaVersion":1,"command":"provider.connect","payload":{"step":"test","workspaceId":workspace,"providerId":"binance","environment":"LIVE","label":"Limited read source"}});
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
        let connected = command(
            &mut control,
            "provider.connect",
            json!({"workspaceId":workspace,"step":"confirm","connectionId":tested["data"]["connectionId"],"expectedStateVersion":tested["data"]["stateVersion"],"acknowledgeUnverified":false}),
        );
        assert_eq!(connected["ok"], true, "{connected}");
        let source = command(
            &mut control,
            "data.binance_rules.connection",
            json!({"workspaceId":workspace}),
        );
        assert_eq!(
            command(
                &mut control,
                "data.binance_rules.configure",
                json!({"workspaceId":workspace,"expectedStateVersion":source["data"]["stateVersion"],"connectionId":connected["data"]["connectionId"],"instrumentId":"crypto:BTC/USDT:spot"})
            )["ok"],
            true
        );
        assert_eq!(
            command(
                &mut control,
                "time.revalidate",
                json!({"workspaceId":workspace})
            )["data"]["confidence"],
            "TRUSTED"
        );
        Self {
            _folder: folder,
            control: Arc::new(Mutex::new(control)),
            workspace,
            vault,
            http,
        }
    }
    fn request(&self) -> Value {
        let source = command(
            &mut self.control.lock().unwrap(),
            "data.binance_rules.connection",
            json!({"workspaceId":self.workspace}),
        );
        json!({"requestId":"limited-read","schemaVersion":1,"command":"data.binance_rules.refresh","payload":{"workspaceId":self.workspace,"expectedStateVersion":source["data"]["stateVersion"]}})
    }
    fn refresh(&self) -> Value {
        tradex::financial_sources::execute_refresh(
            &self.control,
            &self.request(),
            "main",
            &self.vault,
            &self.http,
        )
    }
}

#[test]
fn account_and_ip_read_budgets_survive_selection_and_workspace_changes() {
    let first = ReadSource::new(9007199254751001);
    let mut successes = 0;
    loop {
        let response = first.refresh();
        if response["data"]["status"] == "UNAVAILABLE" {
            assert!(
                response["data"]["availabilityReason"]
                    .as_str()
                    .unwrap()
                    .contains("quota"),
                "{response}"
            );
            break;
        }
        assert_eq!(response["data"]["status"], "AVAILABLE", "{response}");
        successes += 1;
        assert!(successes < 20, "The saved account budget was bypassed");
    }
    assert!(successes > 0);
    let selected = command(
        &mut first.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":first.workspace}),
    );
    let changed = command(
        &mut first.control.lock().unwrap(),
        "data.binance_rules.configure",
        json!({"workspaceId":first.workspace,"expectedStateVersion":selected["data"]["stateVersion"],"connectionId":selected["data"]["connectionId"],"instrumentId":"crypto:ETH/USDT:spot"}),
    );
    assert_eq!(changed["ok"], true, "{changed}");
    assert_eq!(
        first.refresh()["data"]["status"],
        "UNAVAILABLE",
        "Selection must not reset the account budget"
    );
    let second = ReadSource::new(9007199254751002);
    let mut more = 0;
    loop {
        let response = second.refresh();
        if response["data"]["status"] == "UNAVAILABLE" {
            break;
        }
        assert_eq!(response["data"]["status"], "AVAILABLE", "{response}");
        more += 1;
        assert!(more < 20, "The shared IP budget was bypassed");
    }
    assert!(
        more > 0,
        "The separate authenticated UID has its own account allowance"
    );
    let third = ReadSource::new(9007199254751003);
    let exhausted = third.refresh();
    assert_eq!(
        exhausted["data"]["status"], "UNAVAILABLE",
        "A third workspace/UID must inherit consumed IP weight: {exhausted}"
    );
    assert!(
        exhausted["data"]["availabilityReason"]
            .as_str()
            .unwrap()
            .contains("quota")
    );
}

#[derive(Clone)]
struct WaitingVault {
    inner: fixtures::Vault,
    gate: Arc<(Mutex<bool>, Condvar)>,
    entered: Arc<std::sync::atomic::AtomicBool>,
}
impl CredentialVault for WaitingVault {
    fn put(&self, reference: &str, value: &Credentials) -> Result<()> {
        self.inner.put(reference, value)
    }
    fn get(&self, reference: &str) -> Result<Credentials> {
        self.entered
            .store(true, std::sync::atomic::Ordering::Release);
        let (mutex, signal) = &*self.gate;
        let mut ready = mutex.lock().unwrap();
        while !*ready {
            ready = signal.wait(ready).unwrap();
        }
        self.inner.get(reference)
    }
    fn remove(&self, reference: &str) -> Result<()> {
        self.inner.remove(reference)
    }
}
#[test]
fn vault_deadline_finishes_without_http_or_late_publication() {
    let source = ReadSource::new(9007199254752001);
    let request = source.request();
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let entered = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let vault = WaitingVault {
        inner: source.vault.clone(),
        gate: gate.clone(),
        entered: entered.clone(),
    };
    let control = source.control.clone();
    let started = std::time::Instant::now();
    let worker = std::thread::spawn(move || {
        let http = fixtures::Http::default();
        let response =
            tradex::financial_sources::execute_refresh(&control, &request, "main", &vault, &http);
        (response, http.calls.borrow().len())
    });
    while !entered.load(std::sync::atomic::Ordering::Acquire) {
        assert!(started.elapsed().as_secs() < 5, "Vault read never started");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        command(
            &mut source.control.lock().unwrap(),
            "data.binance_rules.connection",
            json!({"workspaceId":source.workspace})
        )["ok"],
        true,
        "A pending protected read must not hold the Control Plane lock"
    );
    let (response, calls) = worker.join().unwrap();
    *gate.0.lock().unwrap() = true;
    gate.1.notify_all();
    assert!(
        started.elapsed().as_secs() < 35,
        "Read did not finish at its 30-second caller deadline"
    );
    assert_eq!(calls, 0, "Late credentials must never start HTTP");
    assert_eq!(response["data"]["status"], "UNAVAILABLE", "{response}");
    std::thread::sleep(std::time::Duration::from_millis(50));
    let after = command(
        &mut source.control.lock().unwrap(),
        "data.binance_rules.connection",
        json!({"workspaceId":source.workspace}),
    );
    assert!(
        after["data"]["evidence"].is_null(),
        "Late vault completion published an observation: {after}"
    );
}
