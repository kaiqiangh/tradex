//! Owned CLIProxyAPI process. No renderer-supplied paths, arguments or credentials.
use crate::gateway::{GatewayAction, GatewayJob, GatewayState, GatewayStatus};
use crate::model::ModelQuota;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::TcpListener,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[cfg(not(target_arch = "x86_64"))]
const EXECUTABLE_SHA: &str = "29d978064c49874a54126b161266b6b8a42b971a7d1ffcf742920761333d40d0";
#[cfg(not(target_arch = "x86_64"))]
const ARCHIVE_SHA: &str = "f90c503ce41a798c85b6f61dfe5fe8b812c1b889634f0c80d04ee376424fe305";
#[cfg(not(target_arch = "x86_64"))]
const ARCHIVE_URL: &str = "https://github.com/router-for-me/CLIProxyAPI/releases/download/v7.2.155/CLIProxyAPI_7.2.155_darwin_aarch64.tar.gz";

#[cfg(target_arch = "x86_64")]
const EXECUTABLE_SHA: &str = "059ee564515d362f7ea233327fd92307378a48f62017d0964ff7f6813a1a511a";
#[cfg(target_arch = "x86_64")]
const ARCHIVE_SHA: &str = "198794a2fafb9fb8083476ac18232647c57d443422aa3f008d19ed7e75ca4604";
#[cfg(target_arch = "x86_64")]
const ARCHIVE_URL: &str = "https://github.com/router-for-me/CLIProxyAPI/releases/download/v7.2.155/CLIProxyAPI_7.2.155_darwin_amd64.tar.gz";

type Result<T> = std::result::Result<T, &'static str>;

pub struct GatewayHost {
    root: PathBuf,
    child: Option<Child>,
    runtime_lock: Option<File>,
    workspace: Option<String>,
    config: Option<PathBuf>,
    key: Zeroizing<String>,
    deepseek_key: Zeroizing<String>,
    deepseek_key_workspace: Option<String>,
    deepseek_key_reload_pending: bool,
    last_quota: Option<ModelQuota>,
    owner_session: Option<String>,
    next_probe: Instant,
    stable_since: Option<Instant>,
    retry_at: Option<Instant>,
    restart_attempts: u32,
    retry_pending: bool,
}

fn private_dir(path: &Path) -> Result<()> {
    if path.exists() {
        let metadata = path
            .symlink_metadata()
            .map_err(|_| "GATEWAY_STORAGE_FAILED")?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.permissions().mode() & 0o077 != 0
        {
            return Err("GATEWAY_STORAGE_FAILED");
        }
    } else {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(|_| "GATEWAY_STORAGE_FAILED")?;
    }
    Ok(())
}

fn digest(path: &Path) -> Result<String> {
    let metadata = path
        .symlink_metadata()
        .map_err(|_| "GATEWAY_NOT_INSTALLED")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > 200 * 1024 * 1024
    {
        return Err("GATEWAY_BINARY_INVALID");
    }
    let mut file = File::open(path).map_err(|_| "GATEWAY_BINARY_INVALID")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "GATEWAY_BINARY_INVALID")?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn quota_metadata(headers: &reqwest::header::HeaderMap) -> Option<ModelQuota> {
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .filter(|value| {
                !value.is_empty() && value.len() <= 64 && !value.chars().any(char::is_control)
            })
    };
    let remaining = header("x-ratelimit-remaining")
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|remaining| *remaining <= 1_000_000_000);
    let retry_after = header("retry-after")
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds <= 86_400);
    let reset_at = header("x-ratelimit-reset").map(str::to_owned);
    if remaining.is_none() && retry_after.is_none() && reset_at.is_none() {
        return None;
    }
    Some(ModelQuota {
        window: retry_after.map(|seconds| format!("retry-after:{seconds}s")),
        reset_at,
        retry_after_seconds: retry_after,
        remaining,
    })
}

fn parse_model_catalog(body: &[u8]) -> Result<Vec<String>> {
    if body.len() > 1024 * 1024 {
        return Err("GATEWAY_PROBE_FAILED");
    }
    let data: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| "GATEWAY_PROBE_FAILED")?;
    let models = data
        .get("data")
        .and_then(|value| value.as_array())
        .filter(|models| models.len() <= 1000)
        .ok_or("GATEWAY_PROBE_FAILED")?;
    models
        .iter()
        .map(|model| {
            let id = model
                .get("id")
                .and_then(|value| value.as_str())
                .ok_or("GATEWAY_PROBE_FAILED")?;
            if id.is_empty() || id.len() > 128 || id.chars().any(char::is_control) {
                return Err("GATEWAY_PROBE_FAILED");
            }
            Ok(id.to_owned())
        })
        .collect()
}

fn inference_payload(model_id: &str, thinking_type: Option<&str>) -> Result<Vec<u8>> {
    if model_id.is_empty() || model_id.len() > 128 || model_id.chars().any(char::is_control) {
        return Err("MODEL_ROUTE_INVALID");
    }
    let mut body = serde_json::json!({
        "model": model_id,
        "messages": [{"role":"user","content":"Reply with exactly: OK"}],
        "max_tokens": 16,
        "temperature": 0,
        "stream": false
    });
    if let Some(mode) = thinking_type {
        body["thinking"] = serde_json::json!({"type": mode});
    }
    serde_json::to_vec(&body).map_err(|_| "MODEL_TEST_INFERENCE_FAILED")
}

fn classify_inference_status(status: u16, model_id: &str) -> Result<()> {
    match status {
        401 => Err(if model_id == "deepseek-v4-flash" {
            "MODEL_UNAVAILABLE"
        } else {
            "MODEL_OAUTH_EXPIRED"
        }),
        429 => Err("MODEL_QUOTA_EXCEEDED"),
        404 => Err("MODEL_UNAVAILABLE"),
        200..=299 => Ok(()),
        _ => Err("MODEL_TEST_INFERENCE_FAILED"),
    }
}

fn classify_inference_response(status: u16, model_id: &str, body: &[u8]) -> Result<()> {
    if status == 503 && model_id != "deepseek-v4-flash" {
        let lower = String::from_utf8_lossy(body).to_ascii_lowercase();
        if lower.contains("auth_unavailable")
            || lower.contains("token_expired")
            || lower.contains("could not validate your token")
            || lower.contains("invalid or expired token")
        {
            return Err("MODEL_OAUTH_EXPIRED");
        }
    }
    classify_inference_status(status, model_id)
}

fn parse_inference_response(body: &[u8]) -> Result<()> {
    if body.len() > 1024 * 1024 {
        return Err("MODEL_TEST_INFERENCE_FAILED");
    }
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| "MODEL_TEST_INFERENCE_FAILED")?;
    if value
        .get("choices")
        .and_then(|choices| choices.as_array())
        .is_none_or(|choices| choices.is_empty())
    {
        return Err("MODEL_TEST_INFERENCE_FAILED");
    }
    Ok(())
}

impl GatewayHost {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            child: None,
            runtime_lock: None,
            workspace: None,
            config: None,
            key: Zeroizing::new(String::new()),
            deepseek_key: Zeroizing::new(String::new()),
            deepseek_key_workspace: None,
            deepseek_key_reload_pending: false,
            last_quota: None,
            owner_session: None,
            next_probe: Instant::now(),
            stable_since: None,
            retry_at: None,
            restart_attempts: 0,
            retry_pending: false,
        }
    }

    pub fn set_deepseek_key(&mut self, workspace_id: &str, key: Option<&str>) {
        self.deepseek_key = Zeroizing::new(key.unwrap_or_default().to_owned());
        self.deepseek_key_workspace = key.map(|_| workspace_id.to_owned());
        self.deepseek_key_reload_pending = false;
    }

    pub fn needs_deepseek_key_reload(&self) -> bool {
        self.deepseek_key_reload_pending
    }

    pub fn codex_runtime_access(
        &self,
        workspace_id: &str,
    ) -> Option<crate::codex_runtime::RuntimeAccess> {
        (self.workspace.as_deref() == Some(workspace_id) && self.child.is_some())
            .then(|| crate::codex_runtime::RuntimeAccess::for_gateway(self.key.as_str()))
            .flatten()
    }

    fn install(&mut self) -> Result<PathBuf> {
        if !cfg!(all(
            target_os = "macos",
            any(target_arch = "aarch64", target_arch = "x86_64")
        )) {
            return Err("GATEWAY_PLATFORM_UNSUPPORTED");
        }
        private_dir(&self.root)?;
        if self.runtime_lock.is_none() {
            let path = self.root.join(".gateway.lock");
            if path
                .symlink_metadata()
                .is_ok_and(|m| m.file_type().is_symlink())
            {
                return Err("GATEWAY_STORAGE_FAILED");
            }
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .mode(0o600)
                .open(path)
                .map_err(|_| "GATEWAY_STORAGE_FAILED")?;
            lock.try_lock().map_err(|_| "GATEWAY_BUSY")?;
            self.runtime_lock = Some(lock);
        }
        let binary = self.root.join("cli-proxy-api-7.2.155");
        if binary.exists() {
            return if digest(&binary)? == EXECUTABLE_SHA {
                Ok(binary)
            } else {
                Err("GATEWAY_BINARY_INVALID")
            };
        }
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .https_only(true)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() < 4
                    && attempt.url().scheme() == "https"
                    && matches!(
                        attempt.url().host_str(),
                        Some("github.com" | "release-assets.githubusercontent.com")
                    )
                {
                    attempt.follow()
                } else {
                    attempt.stop()
                }
            }))
            .build()
            .map_err(|_| "GATEWAY_INSTALL_FAILED")?;
        let response = client
            .get(ARCHIVE_URL)
            .send()
            .map_err(|_| "GATEWAY_INSTALL_FAILED")?;
        if !response.status().is_success() {
            return Err("GATEWAY_INSTALL_FAILED");
        }
        let mut archive = Vec::new();
        response
            .take(100 * 1024 * 1024 + 1)
            .read_to_end(&mut archive)
            .map_err(|_| "GATEWAY_INSTALL_FAILED")?;
        if archive.len() > 100 * 1024 * 1024 || hex::encode(Sha256::digest(&archive)) != ARCHIVE_SHA
        {
            return Err("GATEWAY_BINARY_INVALID");
        }
        let staging = self.root.join(format!("install-{}", uuid::Uuid::new_v4()));
        private_dir(&staging)?;
        let result = (|| {
            let path = staging.join("archive.tar.gz");
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&path)
                .and_then(|mut f| f.write_all(&archive))
                .map_err(|_| "GATEWAY_INSTALL_FAILED")?;
            let extracted = staging.join("binary");
            let output = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o700)
                .open(&extracted)
                .map_err(|_| "GATEWAY_INSTALL_FAILED")?;
            let mut tar = Command::new("/usr/bin/tar")
                .args(["-xOf"])
                .arg(&path)
                .arg("cli-proxy-api")
                .env_clear()
                .stdin(Stdio::null())
                .stdout(output)
                .stderr(Stdio::null())
                .spawn()
                .map_err(|_| "GATEWAY_INSTALL_FAILED")?;
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if let Some(status) = tar.try_wait().map_err(|_| "GATEWAY_INSTALL_FAILED")? {
                    if !status.success() {
                        return Err("GATEWAY_INSTALL_FAILED");
                    }
                    break;
                }
                if Instant::now() >= deadline {
                    let _ = tar.kill();
                    let _ = tar.wait();
                    return Err("GATEWAY_INSTALL_FAILED");
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            if digest(&extracted)? != EXECUTABLE_SHA {
                return Err("GATEWAY_BINARY_INVALID");
            }
            fs::rename(&extracted, &binary).map_err(|_| "GATEWAY_INSTALL_FAILED")?;
            Ok(binary)
        })();
        let _ = fs::remove_dir_all(staging);
        result
    }

    pub fn stop(&mut self) -> bool {
        if let Some(mut child) = self.child.take() {
            // Only signal a retained, unreaped child, never a persisted or caller-supplied PID.
            if child.try_wait().ok().flatten().is_none() {
                let _ = Command::new("/bin/kill")
                    .arg("-TERM")
                    .arg(child.id().to_string())
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                let deadline = Instant::now() + Duration::from_secs(2);
                while Instant::now() < deadline && child.try_wait().ok().flatten().is_none() {
                    std::thread::sleep(Duration::from_millis(20));
                }
                let _ = child.kill();
            }
            let _ = child.wait();
        }
        let cleaned = if let Some(path) = self.config.take() {
            match fs::remove_file(&path) {
                Ok(()) => true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
                Err(_) => {
                    self.config = Some(path);
                    false
                }
            }
        } else {
            true
        };
        self.key = Zeroizing::new(String::new());
        self.deepseek_key_reload_pending = !self.deepseek_key.is_empty();
        self.deepseek_key = Zeroizing::new(String::new());
        self.deepseek_key_workspace = None;
        self.last_quota = None;
        self.workspace = None;
        if cleaned {
            self.runtime_lock = None;
        }
        cleaned
    }

    fn owned_listener(&mut self) -> Result<bool> {
        let child = self.child.as_mut().ok_or("GATEWAY_STOPPED")?;
        if child
            .try_wait()
            .map_err(|_| "GATEWAY_PROCESS_FAILED")?
            .is_some()
        {
            return Err("GATEWAY_PROCESS_FAILED");
        }
        let output = Command::new("/usr/sbin/lsof")
            .args(["-nP", "-a", "-p"])
            .arg(child.id().to_string())
            .args(["-iTCP:8317", "-sTCP:LISTEN", "-Fn"])
            .env_clear()
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|_| "GATEWAY_PROCESS_FAILED")?;
        Ok(output.status.success()
            && String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line == "n127.0.0.1:8317"))
    }

    pub fn discover_models(&mut self) -> Result<Vec<String>> {
        if !self.owned_listener()? {
            return Err("GATEWAY_PROCESS_FAILED");
        }
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|_| "GATEWAY_PROBE_FAILED")?;
        let response = client
            .get("http://127.0.0.1:8317/v1/models")
            .bearer_auth(self.key.as_str())
            .send()
            .map_err(|_| "GATEWAY_PROBE_FAILED")?;
        if response.status().as_u16() == 401 {
            return Err("GATEWAY_UNAUTHORIZED");
        }
        if !response.status().is_success() {
            return Err("GATEWAY_PROBE_FAILED");
        }
        let mut body = Zeroizing::new(Vec::new());
        response
            .take(1024 * 1024 + 1)
            .read_to_end(&mut body)
            .map_err(|_| "GATEWAY_PROBE_FAILED")?;
        parse_model_catalog(&body)
    }

    fn probe(&mut self) -> Result<u32> {
        Ok(self.discover_models()?.len() as u32)
    }

    pub fn test_inference(&mut self, model_id: &str, thinking_type: Option<&str>) -> Result<()> {
        if !self.owned_listener()? {
            return Err("GATEWAY_PROCESS_FAILED");
        }
        let request_body = inference_payload(model_id, thinking_type)?;
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| "MODEL_TEST_INFERENCE_FAILED")?;
        let response = client
            .post("http://127.0.0.1:8317/v1/chat/completions")
            .bearer_auth(self.key.as_str())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(request_body)
            .send()
            .map_err(|_| "MODEL_TEST_INFERENCE_FAILED")?;
        self.last_quota = quota_metadata(response.headers());
        let status = response.status().as_u16();
        let mut bytes = Zeroizing::new(Vec::new());
        response
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "MODEL_TEST_INFERENCE_FAILED")?;
        classify_inference_response(status, model_id, &bytes)?;
        parse_inference_response(&bytes)
    }

    pub fn take_last_quota(&mut self) -> Option<ModelQuota> {
        self.last_quota.take()
    }

    pub fn login_chatgpt(&mut self, current: impl Fn() -> bool) -> Result<()> {
        if !current() {
            return Err("STATE_VERSION_CONFLICT");
        }
        let binary = self.root.join("cli-proxy-api-7.2.155");
        if digest(&binary)? != EXECUTABLE_SHA {
            return Err("GATEWAY_BINARY_INVALID");
        }
        let config = self.config.clone().ok_or("GATEWAY_STOPPED")?;
        if self.child.is_none() {
            return Err("GATEWAY_STOPPED");
        }
        let runtime = config.parent().ok_or("GATEWAY_STORAGE_FAILED")?;
        let config_arg = config.to_string_lossy().into_owned();
        let mut login = Command::new(binary)
            .args(["-config", config_arg.as_str(), "-codex-login"])
            .current_dir(runtime)
            .env_clear()
            .env("HOME", runtime.join("home"))
            .env("TMPDIR", runtime)
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "MODEL_LOGIN_FAILED")?;
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            if !current() {
                let _ = login.kill();
                let _ = login.wait();
                return Err("STATE_VERSION_CONFLICT");
            }
            if let Some(status) = login.try_wait().map_err(|_| "MODEL_LOGIN_FAILED")? {
                return if status.success() {
                    Ok(())
                } else {
                    Err("MODEL_LOGIN_FAILED")
                };
            }
            if Instant::now() >= deadline {
                let _ = login.kill();
                let _ = login.wait();
                return Err("MODEL_LOGIN_TIMEOUT");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn run(&mut self, job: &GatewayJob, current: impl Fn() -> bool) -> GatewayState {
        if job.action != GatewayAction::Probe {
            self.restart_attempts = 0;
            self.retry_at = None;
            self.retry_pending = false;
            self.stable_since = None;
        }
        self.owner_session = Some(job.session.clone());
        self.execute(job, current)
    }

    pub fn monitor(
        &mut self,
        job: &GatewayJob,
        current: impl Fn() -> bool,
    ) -> Option<GatewayState> {
        if self.owner_session.as_deref() != Some(&job.session) {
            self.stop();
            self.retry_at = None;
            self.retry_pending = false;
            self.owner_session = Some(job.session.clone());
            return None;
        }
        if !current() {
            return None;
        }
        let mut state = job.state.clone();
        if !state.desired_running {
            self.stop();
            self.deepseek_key_reload_pending = false;
            self.retry_pending = false;
            self.retry_at = None;
            return None;
        }
        if self.retry_pending && state.status == GatewayStatus::Starting {
            self.retry_pending = false;
            let restart = GatewayJob {
                action: GatewayAction::Launch,
                request_id: job.request_id.clone(),
                session: job.session.clone(),
                state: state.clone(),
            };
            state = self.execute(&restart, &current);
            state.restart_attempts = self.restart_attempts;
            if state.status == GatewayStatus::Running {
                return Some(state);
            }
        } else if state.status == GatewayStatus::Backoff {
            if self.retry_at.is_some_and(|at| Instant::now() >= at) {
                self.restart_attempts += 1;
                self.retry_pending = true;
                self.retry_at = None;
                state.restart_attempts = self.restart_attempts;
                state.next_retry_at = None;
                state.status = GatewayStatus::Starting;
                return Some(state);
            }
            return None;
        } else if state.status == GatewayStatus::Running {
            let alive = self
                .child
                .as_mut()
                .is_some_and(|child| child.try_wait().ok().is_some_and(|status| status.is_none()));
            if alive && Instant::now() < self.next_probe {
                return None;
            }
            self.next_probe = Instant::now() + Duration::from_secs(5);
            let result = if alive {
                self.probe()
            } else {
                Err("GATEWAY_PROCESS_FAILED")
            };
            match result {
                Ok(count) => {
                    if self
                        .stable_since
                        .is_some_and(|since| since.elapsed() >= Duration::from_secs(60))
                    {
                        self.restart_attempts = 0;
                    }
                    if count == state.discovered_model_count
                        && self.restart_attempts == state.restart_attempts
                    {
                        return None;
                    }
                    state.discovered_model_count = count;
                    state.restart_attempts = self.restart_attempts;
                    state.last_probe_at = crate::storage::timestamp().ok();
                    return Some(state);
                }
                Err(code) => {
                    state.error_code = Some(code.into());
                }
            }
        } else {
            return None;
        }
        self.stop();
        self.stable_since = None;
        state.model_available = false;
        state.last_probe_at = None;
        state.discovered_model_count = 0;
        if self.restart_attempts >= 3
            || matches!(
                state.error_code.as_deref(),
                Some("GATEWAY_UNAUTHORIZED" | "GATEWAY_PORT_CONFLICT")
            )
        {
            state.status = if state.error_code.as_deref() == Some("GATEWAY_UNAUTHORIZED") {
                GatewayStatus::Unauthorized
            } else if state.error_code.as_deref() == Some("GATEWAY_PORT_CONFLICT") {
                GatewayStatus::PortConflict
            } else {
                GatewayStatus::Failed
            };
            state.desired_running = false;
            state.next_retry_at = None;
        } else {
            let delay = 1 << self.restart_attempts;
            self.retry_at = Some(Instant::now() + Duration::from_secs(delay));
            state.next_retry_at = (time::OffsetDateTime::now_utc()
                + time::Duration::seconds(delay as i64))
            .format(&time::format_description::well_known::Rfc3339)
            .ok();
            state.status = GatewayStatus::Backoff;
        }
        Some(state)
    }

    fn execute(&mut self, job: &GatewayJob, current: impl Fn() -> bool) -> GatewayState {
        let mut state = job.state.clone();
        let result = (|| {
            if !current() {
                return Err("STATE_VERSION_CONFLICT");
            }
            if uuid::Uuid::parse_str(&state.workspace_id).is_err() {
                return Err("GATEWAY_STORAGE_FAILED");
            }
            if job.action == GatewayAction::Stop {
                if !self.stop() {
                    return Err("GATEWAY_CLEANUP_FAILED");
                }
                state.status = GatewayStatus::Stopped;
                state.last_probe_at = None;
                state.discovered_model_count = 0;
                return Ok(());
            }
            let restart_or_switch = job.action == GatewayAction::Restart
                || self.workspace.as_deref() != Some(&state.workspace_id);
            let retained_deepseek_key = restart_or_switch
                .then(|| {
                    (self.deepseek_key_workspace.as_deref() == Some(state.workspace_id.as_str())
                        && !self.deepseek_key.is_empty())
                    .then(|| self.deepseek_key.clone())
                })
                .flatten();
            if restart_or_switch {
                if !self.stop() {
                    return Err("GATEWAY_CLEANUP_FAILED");
                }
                if let Some(key) = retained_deepseek_key.as_ref() {
                    self.set_deepseek_key(&state.workspace_id, Some(key.as_str()));
                }
            }
            if job.action == GatewayAction::Probe && self.child.is_none() {
                return Err("GATEWAY_STOPPED");
            }
            if self.child.is_none() {
                let binary = match self.install() {
                    Ok(binary) => binary,
                    Err(code) => {
                        state.installed = false;
                        return Err(code);
                    }
                };
                state.installed = true;
                if !current() {
                    return Err("STATE_VERSION_CONFLICT");
                }
                let runtime = self.root.join(&state.workspace_id);
                private_dir(&runtime)?;
                private_dir(&runtime.join("auth"))?;
                private_dir(&runtime.join("home"))?;
                let config = runtime.join("gateway.json");
                if config.exists() {
                    fs::remove_file(&config).map_err(|_| "GATEWAY_STORAGE_FAILED")?;
                }
                let listener =
                    TcpListener::bind("127.0.0.1:8317").map_err(|_| "GATEWAY_PORT_CONFLICT")?;
                self.key = Zeroizing::new(format!(
                    "{}{}",
                    uuid::Uuid::new_v4().simple(),
                    uuid::Uuid::new_v4().simple()
                ));
                let mut settings = serde_json::json!({"host":"127.0.0.1","port":8317,"auth-dir":runtime.join("auth"),"api-keys":[self.key.as_str()],
                    "remote-management":{"allow-remote":false,"secret-key":"","disable-control-panel":true,"disable-auto-update-panel":true},
                    "commercial-mode":true,"logging-to-file":false,"debug":false,"request-log":false,"usage-statistics-enabled":false,
                    "plugins":{"enabled":false},"request-retry":0,"max-retry-credentials":1,"max-retry-interval":0,
                    "quota-exceeded":{"switch-project":false,"switch-preview-model":false},"pprof":{"enable":false}});
                if !self.deepseek_key.is_empty() {
                    settings["openai-compatibility"] = serde_json::json!([{
                        "name":"deepseek",
                        "base-url":"https://api.deepseek.com/v1",
                        "request-retry":0,
                        "api-key-entries":[{"api-key":self.deepseek_key.as_str(),"proxy-url":"direct"}],
                        "models":[{"name":"deepseek-v4-flash","alias":"deepseek-v4-flash"}]
                    }]);
                }
                let encoded = Zeroizing::new(
                    serde_json::to_vec(&settings).map_err(|_| "GATEWAY_STORAGE_FAILED")?,
                );
                self.config = Some(config.clone());
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&config)
                    .and_then(|mut f| {
                        f.write_all(&encoded)?;
                        f.sync_all()
                    })
                    .map_err(|_| "GATEWAY_STORAGE_FAILED")?;
                drop(listener);
                self.child = Some(
                    Command::new(binary)
                        .arg("-config")
                        .arg(config)
                        .arg("-local-model")
                        .current_dir(&runtime)
                        .env_clear()
                        .env("HOME", runtime.join("home"))
                        .env("TMPDIR", &runtime)
                        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .map_err(|_| "GATEWAY_PROCESS_FAILED")?,
                );
                self.workspace = Some(state.workspace_id.clone());
                let deadline = Instant::now() + Duration::from_secs(5);
                while !self.owned_listener()? {
                    if !current() {
                        return Err("STATE_VERSION_CONFLICT");
                    }
                    if Instant::now() >= deadline {
                        return Err("GATEWAY_PROBE_FAILED");
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            if !current() {
                return Err("STATE_VERSION_CONFLICT");
            }
            state.discovered_model_count = self.probe()?;
            state.last_probe_at =
                Some(crate::storage::timestamp().map_err(|_| "GATEWAY_PROCESS_FAILED")?);
            state.status = GatewayStatus::Running;
            state.error_code = None;
            state.next_retry_at = None;
            self.next_probe = Instant::now() + Duration::from_secs(5);
            if self.stable_since.is_none() {
                self.stable_since = Some(Instant::now());
            }
            Ok(())
        })();
        if let Err(code) = result {
            let code = if self.stop() {
                code
            } else {
                "GATEWAY_CLEANUP_FAILED"
            };
            state.status = match code {
                "GATEWAY_PORT_CONFLICT" => GatewayStatus::PortConflict,
                "GATEWAY_UNAUTHORIZED" => GatewayStatus::Unauthorized,
                "GATEWAY_STOPPED" => GatewayStatus::Stopped,
                _ => GatewayStatus::Failed,
            };
            state.error_code = Some(code.into());
            state.discovered_model_count = 0;
            state.last_probe_at = None;
        }
        state.model_available = false;
        state
    }
}

impl Drop for GatewayHost {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_catalog_and_inference_responses_fail_closed() {
        assert_eq!(
            parse_model_catalog(br#"{"data":[]}"#).unwrap(),
            Vec::<String>::new()
        );
        assert!(parse_model_catalog(br#"{}"#).is_err());
        assert!(parse_model_catalog(br#"{"data":[{"id":"bad\nroute"}]}"#).is_err());
        assert!(parse_inference_response(br#"{"choices":[]}"#).is_err());
        assert!(parse_inference_response(br#"not-json"#).is_err());
        assert!(parse_inference_response(br#"{"choices":[{}]}"#).is_ok());
    }

    #[test]
    fn inference_request_and_status_keep_provider_contract_explicit() {
        let body: serde_json::Value = serde_json::from_slice(
            &inference_payload("deepseek-v4-flash", Some("enabled")).unwrap(),
        )
        .unwrap();
        assert_eq!(body["model"], "deepseek-v4-flash");
        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(body["max_tokens"], 16);
        assert_eq!(body["temperature"], 0);
        assert_eq!(body["stream"], false);
        assert!(inference_payload("bad\nmodel", None).is_err());
        assert_eq!(
            classify_inference_status(401, "gpt-5.6-sol"),
            Err("MODEL_OAUTH_EXPIRED")
        );
        assert_eq!(
            classify_inference_status(401, "deepseek-v4-flash"),
            Err("MODEL_UNAVAILABLE")
        );
        assert_eq!(
            classify_inference_status(429, "gpt-5.6-sol"),
            Err("MODEL_QUOTA_EXCEEDED")
        );
        assert_eq!(classify_inference_status(200, "gpt-5.6-sol"), Ok(()));
        assert_eq!(
            classify_inference_status(503, "gpt-5.6-sol"),
            Err("MODEL_TEST_INFERENCE_FAILED")
        );
        assert_eq!(
            classify_inference_response(
                503,
                "gpt-5.6-sol",
                br#"{"error":{"code":"auth_unavailable","message":"token_expired"}}"#,
            ),
            Err("MODEL_OAUTH_EXPIRED")
        );
        assert_eq!(
            classify_inference_response(
                503,
                "deepseek-v4-flash",
                br#"{"error":{"code":"auth_unavailable"}}"#,
            ),
            Err("MODEL_TEST_INFERENCE_FAILED")
        );
    }

    #[test]
    fn quota_metadata_is_bounded_and_unknown_stays_unavailable() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-ratelimit-remaining", "42".parse().unwrap());
        headers.insert("retry-after", "60".parse().unwrap());
        headers.insert("x-ratelimit-reset", "2026-09-12T21:00:00Z".parse().unwrap());
        let quota = quota_metadata(&headers).unwrap();
        assert_eq!(quota.remaining, Some(42));
        assert_eq!(quota.window.as_deref(), Some("retry-after:60s"));
        assert_eq!(quota.reset_at.as_deref(), Some("2026-09-12T21:00:00Z"));

        let mut bounded = reqwest::header::HeaderMap::new();
        bounded.insert("x-ratelimit-remaining", "1000000001".parse().unwrap());
        bounded.insert("retry-after", "86401".parse().unwrap());
        assert!(quota_metadata(&bounded).is_none());
        assert!(quota_metadata(&reqwest::header::HeaderMap::new()).is_none());
    }
}
