use super::*;
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    time::Instant,
};

#[test]
fn ordinary_live_cancellation_review_has_read_routes_but_no_provider_write_route() {
    for (endpoint, read_path) in [
        (ProviderEndpoint::Trading212Live, "/api/v0/equity/orders"),
        (
            ProviderEndpoint::BinanceLive,
            "/api/v3/openOrders?timestamp=1788849600000&recvWindow=5000&signature=0000000000000000000000000000000000000000000000000000000000000000",
        ),
        (
            ProviderEndpoint::BitgetLive,
            "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal",
        ),
    ] {
        assert!(endpoint.allows_method(ProviderHttpMethod::Get, read_path));
        assert!(!endpoint.allows_method(ProviderHttpMethod::Post, read_path));
        assert!(!endpoint.allows_method(ProviderHttpMethod::Delete, read_path));
    }
    assert!(!ProviderEndpoint::Trading212Live.allows_method(
        ProviderHttpMethod::Delete,
        "/api/v0/equity/orders/9007199254740995"
    ));
    assert!(
        !ProviderEndpoint::BitgetLive
            .allows_method(ProviderHttpMethod::Post, "/api/v2/spot/trade/cancel-order")
    );
}

#[test]
fn binance_live_delete_is_limited_to_one_exact_spot_order() {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let path = format!(
        "/api/v3/order?symbol=BTCUSDT&orderId=123456&timestamp={timestamp}&recvWindow=5000&signature={}",
        "0".repeat(64)
    );
    assert_eq!(
        ProviderEndpoint::BinanceLive.allows_method(ProviderHttpMethod::Delete, &path),
        cfg!(feature = "order-gateway-runtime")
    );
    assert!(!ProviderEndpoint::BinanceTestnet.allows_method(ProviderHttpMethod::Delete, &path));
    for invalid in [
        path.replace("/api/v3/order?", "/api/v3/openOrders?"),
        path.replace("orderId=123456&", "orderId=123456&origClientOrderId=x&"),
        path.replace("symbol=BTCUSDT", "symbol=BTCUSDT&symbol=ETHUSDT"),
        path.replace("orderId=123456", "orderId=0"),
    ] {
        assert!(!ProviderEndpoint::BinanceLive.allows_method(ProviderHttpMethod::Delete, &invalid));
    }
}

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
        let client = BrokerHttp::client_builder(true)
            .add_root_certificate(reqwest::Certificate::from_pem(&cert).unwrap())
            .proxy(reqwest::Proxy::https(&self.proxy).unwrap())
            .build()
            .unwrap();
        BrokerHttp {
            client: std::sync::OnceLock::from(Ok(client)),
            #[cfg(feature = "integration-test")]
            local_test_base_url: None,
        }
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
fn bitget_order_submission_is_exactly_demo_only_and_requires_paptrading() {
    let path = "/api/v2/spot/trade/place-order";
    assert!(ProviderEndpoint::BitgetDemo.allows_method(ProviderHttpMethod::Post, path));
    assert!(!ProviderEndpoint::BitgetLive.allows_method(ProviderHttpMethod::Post, path));
    assert!(
        !ProviderEndpoint::BitgetDemo
            .allows_method(ProviderHttpMethod::Post, "/api/v2/spot/trade/cancel-order")
    );
    assert!(
        ProviderEndpoint::BitgetDemo
            .allows("/api/v2/spot/trade/orderInfo?clientOid=tx-0123456789abcdef")
    );
    assert!(
        !ProviderEndpoint::BitgetDemo
            .allows("/api/v2/spot/trade/orderInfo?clientOid=tx-0123456789abcdef&orderId=1")
    );
    for path in [
        "/api/v2/spot/trade/history-orders?limit=100",
        "/api/v2/spot/trade/history-orders?limit=100&tpslType=tpsl&idLessThan=9007199254740997",
        "/api/v2/spot/trade/history-plan-order?limit=100&idLessThan=9007199254740997",
        "/api/v2/spot/trade/fills?limit=100&idLessThan=9223372036854775808",
    ] {
        assert!(ProviderEndpoint::BitgetLive.allows(path), "{path}");
        assert!(!ProviderEndpoint::BitgetDemo.allows(path), "{path}");
    }
    for path in [
        "/api/v2/spot/trade/history-orders?limit=100&tpslType=plan",
        "/api/v2/spot/trade/history-plan-order?limit=100&idLessThan=0",
    ] {
        assert!(!ProviderEndpoint::BitgetLive.allows(path), "{path}");
    }
    assert!(
        !ProviderEndpoint::BitgetLive
            .allows_method(ProviderHttpMethod::Post, "/api/v2/spot/trade/place-order")
    );

    let error = BrokerHttp::default()
        .request(
            ProviderEndpoint::BitgetDemo,
            ProviderHttpMethod::Post,
            path,
            HeaderMap::new(),
            Some(&json!({"symbol":"BTCUSDT"})),
        )
        .err()
        .expect("a write without paptrading must be rejected locally");
    assert_eq!(error.code, "PROVIDER_UNSUPPORTED");
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

#[test]
fn fx_https_uses_only_fixed_get_pairs_with_bounded_body_timeout_and_no_redirect() {
    let endpoint = ProviderEndpoint::AlpacaMarketData;
    let path = "/v1beta1/forex/latest/rates?currency_pairs=EURUSD";
    for allowed in [
        path,
        "/v1beta1/forex/latest/rates?currency_pairs=USDEUR",
        "/v1beta1/forex/latest/rates?currency_pairs=EURUSD,USDEUR",
    ] {
        assert!(endpoint.allows_method(ProviderHttpMethod::Get, allowed));
        assert!(!endpoint.allows_method(ProviderHttpMethod::Post, allowed));
        assert!(!endpoint.allows_method(ProviderHttpMethod::Delete, allowed));
    }
    for forbidden in [
        "/v1beta1/forex/rates?currency_pairs=EURUSD",
        "/v1beta1/forex/latest/rates?currency_pairs=GBPUSD",
        "/v1beta1/forex/latest/rates?currency_pairs=EURUSD,EURUSD",
        "/v1beta1/forex/latest/rates?currency_pairs=EURUSD&url=https://example.com",
        "/v1beta1/forex/latest/rates?currency_pairs=USDEUR,EURUSD",
        "/v1beta1/forex/latest/rates?currency_pairs=EURUSD%2CUSDEUR",
    ] {
        assert!(!endpoint.allows_method(ProviderHttpMethod::Get, forbidden));
    }
    for (mode, status, error) in [
        ("ok", Some(200), None),
        ("auth", Some(401), None),
        ("forbidden", Some(403), None),
        ("rate", Some(429), None),
        ("redirect", Some(302), None),
        ("large", None, Some("PROVIDER_RESPONSE_INVALID")),
        ("timeout", None, Some("PROVIDER_UNAVAILABLE")),
    ] {
        let fixture = HttpsFixture::new(mode, endpoint);
        let http = fixture.http();
        let mut headers = HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            HeaderValue::from_static("synthetic-network-test"),
        );
        headers.insert(
            "APCA-API-SECRET-KEY",
            HeaderValue::from_static("synthetic-network-secret"),
        );
        let start = Instant::now();
        let response = http.request(endpoint, ProviderHttpMethod::Get, path, headers, None);
        if let Some(expected) = error {
            assert_eq!(
                response.err().expect("Expected transport failure").code,
                expected,
                "{mode}"
            );
        } else {
            assert_eq!(response.unwrap().status, status.unwrap(), "{mode}");
        }
        assert_eq!(
            std::fs::read_to_string(fixture.directory.path().join("requests")).unwrap(),
            "1",
            "No redirects or authentication fallback may repeat the request"
        );
        if mode == "timeout" {
            assert!(
                start.elapsed() >= Duration::from_secs(10)
                    && start.elapsed() < Duration::from_secs(18)
            );
        }
    }
}
