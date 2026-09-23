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
fn alpaca_order_book_paths_and_delete_are_bounded_to_paper_uuid_routes() {
    let order = "/v2/orders/18c65e3e-feb0-4576-99e2-36e6f047d84d";
    assert!(ProviderEndpoint::AlpacaPaper.allows(order));
    assert!(ProviderEndpoint::AlpacaPaper.allows_method(ProviderHttpMethod::Delete, order));
    assert!(
        ProviderEndpoint::AlpacaPaper
            .allows("/v2/orders?status=all&limit=100&direction=desc&nested=false")
    );
    assert!(
        ProviderEndpoint::AlpacaPaper
            .allows("/v2/account/activities/FILL?page_size=100&direction=desc&page_token=abc-123")
    );
    assert!(ProviderEndpoint::AlpacaPaper.allows(
        "/v2/account/activities/FILL?page_size=100&direction=desc&page_token=20190801011955195::5f596936-6f23-4cef-bdf1-3806aae57dbf"
    ));
    for path in [
        "/v2/orders/../account",
        "/v2/orders/not-a-uuid",
        "/v2/orders?status=all&limit=1000&direction=desc&nested=false",
        "/v2/account/activities/FILL?page_size=100&direction=desc&page_token=abc&other=1",
    ] {
        assert!(!ProviderEndpoint::AlpacaPaper.allows(path), "{path}");
        assert!(
            !ProviderEndpoint::AlpacaPaper.allows_method(ProviderHttpMethod::Delete, path),
            "{path}"
        );
    }
    assert!(!ProviderEndpoint::Trading212Live.allows_method(ProviderHttpMethod::Delete, order));
}

#[test]
fn provider_order_remaining_quantity_uses_exact_decimal_subtraction() {
    assert_eq!(decimal_subtract("10", "2.5").unwrap(), "7.5");
    assert_eq!(decimal_subtract("0.05", "0.04").unwrap(), "0.01");
    assert_eq!(
        decimal_subtract(
            "999999999999999999999.0000000000000000001",
            "0.0000000000000000001"
        )
        .unwrap(),
        "999999999999999999999"
    );
    assert!(decimal_subtract("1", "1.01").is_err());
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
fn real_https_transport_deletes_only_the_fixed_alpaca_paper_order_route() {
    let endpoint = ProviderEndpoint::AlpacaPaper;
    let fixture = HttpsFixture::new("cancel", endpoint);
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
            ProviderHttpMethod::Delete,
            "/v2/orders/18c65e3e-feb0-4576-99e2-36e6f047d84d",
            headers,
            None,
        )
        .unwrap();
    assert_eq!(response.status, 204);
    assert!(response.body.is_empty());
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
