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
    fn new(mode: &str, endpoint: ProviderEndpoint) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let mut child = Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/provider_https.py"
            ))
            .arg(mode)
            .arg(directory.path())
            .arg(endpoint.base_url().strip_prefix("https://").unwrap())
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
    fn http(&self) -> BrokerHttp {
        let cert = std::fs::read(self.directory.path().join("cert.pem")).unwrap();
        // Only this test client trusts the ephemeral certificate and tunnels to the loopback fixture.
        let client = BrokerHttp::client_builder()
            .add_root_certificate(reqwest::Certificate::from_pem(&cert).unwrap())
            .proxy(reqwest::Proxy::https(&self.proxy).unwrap())
            .build()
            .unwrap();
        BrokerHttp(std::cell::OnceCell::from(Ok(client)))
    }
}
impl Drop for HttpsFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn alpaca_asset_and_position_paths_allow_only_bounded_symbols() {
    for path in [
        "/v2/assets/AAPL",
        "/v2/positions/AAPL",
        "/v2/positions/BRK.B",
    ] {
        assert!(ProviderEndpoint::AlpacaPaper.allows(path), "{path}");
    }
    for path in [
        "/v2/assets/..",
        "/v2/positions/",
        "/v2/positions/../account",
        "/v2/positions/aapl",
        "/v2/positions/AAPL?status=open",
    ] {
        assert!(!ProviderEndpoint::AlpacaPaper.allows(path), "{path}");
    }
}

#[test]
fn real_https_transport_posts_only_to_the_fixed_alpaca_paper_order_route() {
    let endpoint = ProviderEndpoint::AlpacaPaper;
    let fixture = HttpsFixture::new("order", endpoint);
    let http = fixture.http();
    let mut headers = HeaderMap::new();
    let mut key = HeaderValue::from_static("synthetic-network-test");
    key.set_sensitive(true);
    headers.insert("APCA-API-KEY-ID", key);
    let mut secret = HeaderValue::from_static("synthetic-secret");
    secret.set_sensitive(true);
    headers.insert("APCA-API-SECRET-KEY", secret);
    let response = http
        .request(
            endpoint,
            ProviderHttpMethod::Post,
            "/v2/orders",
            headers,
            Some(&serde_json::json!({
                "symbol":"AAPL","side":"buy","type":"market",
                "time_in_force":"day","qty":"1","client_order_id":"tradex-synthetic"
            })),
        )
        .unwrap();
    assert_eq!(response.status, 201);
    assert!(response.body.starts_with(b"{\"id\":"));
    assert_eq!(
        std::fs::read_to_string(fixture.directory.path().join("requests")).unwrap(),
        "1"
    );
}

#[test]
fn real_https_transport_rejects_redirects_oversize_and_timeout_and_classifies_auth_and_quota() {
    for (mode, error, endpoint) in [
        ("ok", None, ProviderEndpoint::AlpacaPaper),
        ("ok", None, ProviderEndpoint::Trading212Demo),
        ("ok", None, ProviderEndpoint::Trading212Live),
        ("ok", None, ProviderEndpoint::BinanceTestnet),
        ("ok", None, ProviderEndpoint::BinanceLive),
        ("ok", None, ProviderEndpoint::BitgetDemo),
        ("ok", None, ProviderEndpoint::BitgetLive),
        (
            "bitget-clock",
            Some("CLOCK_SKEW"),
            ProviderEndpoint::BitgetLive,
        ),
        (
            "bitget-passphrase",
            Some("PROVIDER_AUTH_FAILED"),
            ProviderEndpoint::BitgetLive,
        ),
        (
            "bitget-demo",
            Some("PROVIDER_UNSUPPORTED"),
            ProviderEndpoint::BitgetDemo,
        ),
        (
            "bitget-false-success",
            Some("PROVIDER_RESPONSE_INVALID"),
            ProviderEndpoint::BitgetLive,
        ),
        ("clock", Some("CLOCK_SKEW"), ProviderEndpoint::BinanceLive),
        (
            "signature",
            Some("PROVIDER_AUTH_FAILED"),
            ProviderEndpoint::BinanceLive,
        ),
        (
            "banned",
            Some("PROVIDER_RATE_LIMITED"),
            ProviderEndpoint::BinanceLive,
        ),
        (
            "redirect",
            Some("PROVIDER_UNAVAILABLE"),
            ProviderEndpoint::AlpacaPaper,
        ),
        (
            "auth",
            Some("PROVIDER_AUTH_FAILED"),
            ProviderEndpoint::AlpacaPaper,
        ),
        (
            "rate",
            Some("PROVIDER_RATE_LIMITED"),
            ProviderEndpoint::AlpacaPaper,
        ),
        (
            "large",
            Some("PROVIDER_RESPONSE_INVALID"),
            ProviderEndpoint::AlpacaPaper,
        ),
        (
            "timeout",
            Some("PROVIDER_UNAVAILABLE"),
            ProviderEndpoint::AlpacaPaper,
        ),
    ] {
        let fixture = HttpsFixture::new(mode, endpoint);
        let http = fixture.http();
        let mut headers = HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            HeaderValue::from_static("synthetic-network-test"),
        );
        let start = Instant::now();
        let result = http.get(
            endpoint,
            if endpoint == ProviderEndpoint::AlpacaPaper {
                "/v2/account"
            } else if endpoint.is_bitget() {
                "/api/v2/public/time"
            } else if endpoint.is_binance() {
                "/api/v3/time"
            } else {
                "/api/v0/equity/account/summary"
            },
            headers,
        );
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
