use super::*;
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    time::Instant,
};

struct HttpsFixture {
    child: Child,
    directory: tempfile::TempDir,
    proxy: String,
}
impl HttpsFixture {
    fn new(mode: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let mut child = Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/provider_https.py"
            ))
            .arg(mode)
            .arg(directory.path())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Python 3 and OpenSSL are required for the local HTTPS boundary check");
        let mut port = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut port)
            .unwrap();
        let port: u16 = port
            .trim()
            .parse()
            .expect("Local TLS fixture did not start");
        Self {
            child,
            directory,
            proxy: format!("http://127.0.0.1:{port}"),
        }
    }
    fn http(&self) -> AlpacaHttp {
        let cert = std::fs::read(self.directory.path().join("cert.pem")).unwrap();
        // Only this test client trusts the ephemeral certificate and tunnels to the loopback fixture.
        let client = AlpacaHttp::client_builder()
            .add_root_certificate(reqwest::Certificate::from_pem(&cert).unwrap())
            .proxy(reqwest::Proxy::https(&self.proxy).unwrap())
            .build()
            .unwrap();
        AlpacaHttp(std::cell::OnceCell::from(Ok(client)))
    }
}
impl Drop for HttpsFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn real_https_transport_rejects_redirects_oversize_and_timeout_and_classifies_auth_and_quota() {
    for (mode, error) in [
        ("ok", None),
        ("redirect", Some("PROVIDER_UNAVAILABLE")),
        ("auth", Some("PROVIDER_AUTH_FAILED")),
        ("rate", Some("PROVIDER_RATE_LIMITED")),
        ("large", Some("PROVIDER_RESPONSE_INVALID")),
        ("timeout", Some("PROVIDER_UNAVAILABLE")),
    ] {
        let fixture = HttpsFixture::new(mode);
        let http = fixture.http();
        let mut headers = HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            HeaderValue::from_static("synthetic-network-test"),
        );
        let start = Instant::now();
        let result = http.get("/v2/account", headers);
        let elapsed = start.elapsed();
        match error {
            Some(expected) => assert_eq!(result.unwrap_err().code, expected, "{mode}"),
            None => assert_eq!(
                result.unwrap(),
                b"{}",
                "The fixture must complete a real verified TLS exchange"
            ),
        }
        assert_eq!(
            std::fs::read_to_string(fixture.directory.path().join("requests")).unwrap(),
            "1",
            "Redirects must never follow or repeat the request"
        );
        if mode == "timeout" {
            assert!(
                elapsed >= Duration::from_secs(10) && elapsed < Duration::from_secs(18),
                "Observed deadline: {elapsed:?}"
            );
        }
    }
}
