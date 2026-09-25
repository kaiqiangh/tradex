use serde_json::{Value, json};
use std::cell::RefCell;
use tradex::{
    protocol::Result,
    provider_io::{Credentials, ProviderEndpoint, ProviderHttp},
};
pub const KEY: &str = "S02-BITGET-KEY-463719502";
pub const SECRET: &str = "S02-BITGET-SECRET-593021847";
pub const PASSPHRASE: &str = "S02-BITGET-PASS-648103927";
pub fn credentials() -> Result<Credentials> {
    Credentials::new(vec![KEY.into(), SECRET.into(), PASSPHRASE.into()])
}
#[derive(Default)]
pub struct Http(
    pub RefCell<Vec<String>>,
    RefCell<Option<Value>>,
    RefCell<Vec<Value>>,
);

fn verify_signature(
    method: &str,
    path: &str,
    body: Option<&Value>,
    headers: &reqwest::header::HeaderMap,
) {
    use base64::Engine;
    use hmac::Mac;
    assert_eq!(headers["ACCESS-KEY"], KEY);
    assert_eq!(headers["ACCESS-PASSPHRASE"], PASSPHRASE);
    assert!(headers["ACCESS-KEY"].is_sensitive());
    assert!(headers["ACCESS-PASSPHRASE"].is_sensitive());
    assert!(headers["ACCESS-SIGN"].is_sensitive());
    let timestamp = headers["ACCESS-TIMESTAMP"].to_str().unwrap();
    let body = body
        .map(|value| serde_json::to_string(value).unwrap())
        .unwrap_or_default();
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(SECRET.as_bytes()).unwrap();
    mac.update(format!("{timestamp}{method}{path}{body}").as_bytes());
    mac.verify_slice(
        &base64::engine::general_purpose::STANDARD
            .decode(headers["ACCESS-SIGN"].as_bytes())
            .unwrap(),
    )
    .unwrap();
}

impl ProviderHttp for Http {
    fn get(
        &self,
        endpoint: ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        assert!(matches!(
            endpoint,
            ProviderEndpoint::BitgetDemo | ProviderEndpoint::BitgetLive
        ));
        self.0.borrow_mut().push(path.into());
        if path == "/api/v2/public/time" {
            assert!(headers.is_empty());
            return Ok(br#"{"code":"00000","data":{"serverTime":"1788849600000"}}"#.to_vec());
        }
        if endpoint == ProviderEndpoint::BitgetDemo {
            assert_eq!(headers["paptrading"], "1");
        } else {
            assert!(!headers.contains_key("paptrading"));
        }
        verify_signature("GET", path, None, &headers);
        let order = |id: u64, kind: &str| json!({"userId":"9007199254740993","orderId":id.to_string(),"symbol":"BTCUSDT","size":"0.1234567890123456789","orderType":"limit","side":"buy","status":"live","tpslType":kind,"priceAvg":"12345.67","triggerPrice":"12000","baseVolume":"0.01","quoteVolume":"123.4567"});
        let data = match path {
            "/api/v2/spot/account/info" => {
                json!({"userId":"9007199254740993","ips":"127.0.0.1","authorities":["stor","stow"]})
            }
            "/api/v2/spot/account/assets?assetType=all" => {
                json!([{"coin":"USDT","available":"999999999999999999.9999999999999999999","frozen":"0.0000000000000000001","locked":"2","limitAvailable":"7"}])
            }
            "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal" => {
                Value::Array((101..=200).rev().map(|i| order(i, "normal")).collect())
            }
            "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=normal&idLessThan=101" => {
                let mut o = order(100, "normal");
                o["orderType"] = "market".into();
                o["side"] = "sell".into();
                json!([o])
            }
            "/api/v2/spot/trade/unfilled-orders?limit=100&tpslType=tpsl" => {
                let mut value = order(99, "tpsl");
                value["orderType"] = "market".into();
                json!([value])
            }
            "/api/v2/spot/trade/current-plan-order?limit=100" => {
                json!({"nextFlag":true,"idLessThan":"200","orderList":[{"orderId":"200","symbol":"BTCUSDT","size":"20","executePrice":"1","triggerPrice":"2","status":"not_trigger","orderType":"limit","side":"buy","planType":"total"}]})
            }
            "/api/v2/spot/trade/current-plan-order?limit=100&idLessThan=200" => {
                json!({"nextFlag":false,"idLessThan":"0","orderList":[]})
            }
            "/api/v2/spot/public/symbols?symbol=BTCUSDT" => json!([
                {"symbol":"BTCUSDT","baseCoin":"BTC","quoteCoin":"USDT","status":"online","pricePrecision":"2","quantityPrecision":"6","quotePrecision":"8","minTradeUSDT":"1"}
            ]),
            "/api/v2/spot/market/tickers?symbol=BTCUSDT" => json!([
                {"symbol":"BTCUSDT","bidPr":"65000","askPr":"65001","lastPr":"65000"}
            ]),
            path if path.starts_with("/api/v2/spot/trade/orderInfo?clientOid=") => {
                let client_oid = path
                    .strip_prefix("/api/v2/spot/trade/orderInfo?clientOid=")
                    .unwrap();
                match self
                    .1
                    .borrow()
                    .as_ref()
                    .filter(|order| order["clientOid"] == client_oid)
                {
                    Some(order) => json!([order]),
                    None => json!([]),
                }
            }
            _ => panic!("Unexpected Bitget read path {path}"),
        };
        Ok(serde_json::to_vec(&json!({"code":"00000","data":data})).unwrap())
    }

    fn request(
        &self,
        endpoint: ProviderEndpoint,
        method: tradex::provider_io::ProviderHttpMethod,
        path: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<&Value>,
    ) -> Result<tradex::provider_io::ProviderHttpResponse> {
        use tradex::provider_io::{ProviderHttpMethod, ProviderHttpResponse};
        if method == ProviderHttpMethod::Get {
            return self
                .get(endpoint, path, headers)
                .map(|body| ProviderHttpResponse { status: 200, body });
        }
        if endpoint != ProviderEndpoint::BitgetDemo
            || method != ProviderHttpMethod::Post
            || path != "/api/v2/spot/trade/place-order"
            || headers
                .get("paptrading")
                .and_then(|value| value.to_str().ok())
                != Some("1")
        {
            return Err(tradex::protocol::TradeXError::new("PROVIDER_UNSUPPORTED"));
        }
        verify_signature("POST", path, body, &headers);
        let body =
            body.ok_or_else(|| tradex::protocol::TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
        assert_eq!(body["symbol"], "BTCUSDT");
        let order = json!({
            "userId":"9007199254740993",
            "symbol":body["symbol"],
            "orderId":"9876543210",
            "clientOid":body["clientOid"],
            "side":body["side"],
            "orderType":body["orderType"],
            "status":"live"
        });
        self.0.borrow_mut().push(path.into());
        self.2.borrow_mut().push(body.clone());
        *self.1.borrow_mut() = Some(order.clone());
        Ok(ProviderHttpResponse {
            status: 200,
            body: serde_json::to_vec(&json!({"code":"00000","data":{"orderId":order["orderId"],"clientOid":order["clientOid"]}})).unwrap(),
        })
    }
}
