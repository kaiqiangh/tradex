use serde_json::json;
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
};
use tradex::{
    protocol::{Result, TradeXError},
    provider_io::{CredentialVault, Credentials, ProviderHttp},
};
pub const KEY: &str = "S02-FAKE-KEY-594791453";
pub const SECRET: &str = "S02-FAKE-SECRET-704556921";

#[derive(Default)]
pub struct Vault {
    pub present: RefCell<HashSet<String>>,
    pub fail_remove: Cell<bool>,
}
impl CredentialVault for Vault {
    fn put(&self, reference: &str, _: &Credentials) -> Result<()> {
        self.present.borrow_mut().insert(reference.into());
        Ok(())
    }
    fn get(&self, reference: &str) -> Result<Credentials> {
        if !self.present.borrow().contains(reference) {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        credentials()
    }
    fn remove(&self, reference: &str) -> Result<()> {
        if self.fail_remove.get() {
            return Err(TradeXError::new("CREDENTIAL_DELETE_FAILED"));
        }
        self.present.borrow_mut().remove(reference);
        Ok(())
    }
}
pub fn credentials() -> Result<Credentials> {
    Credentials::new(vec![KEY.into(), SECRET.into()])
}

pub struct Http {
    pub fail: Cell<bool>,
    pub identity: String,
    pub calls: RefCell<Vec<String>>,
}
impl Default for Http {
    fn default() -> Self {
        Self {
            fail: Cell::new(false),
            identity: "81161e77-bafd-44bb-b2a0-60b9055e3cd4".into(),
            calls: RefCell::new(vec![]),
        }
    }
}
impl ProviderHttp for Http {
    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        if matches!(
            endpoint,
            tradex::provider_io::ProviderEndpoint::BinanceTestnet
                | tradex::provider_io::ProviderEndpoint::BinanceLive
        ) {
            self.calls
                .borrow_mut()
                .push(format!("{}{path}", endpoint.base_url()));
            if path == "/api/v3/time" {
                assert!(headers.is_empty());
                return Ok(br#"{"serverTime":1788849600000}"#.to_vec());
            }
            assert_eq!(headers["X-MBX-APIKEY"], KEY);
            assert!(headers["X-MBX-APIKEY"].is_sensitive());
            assert_eq!(headers.len(), 1);
            let (route, query) = path.split_once('?').unwrap();
            let (params, sig) = query.split_once("&signature=").unwrap();
            use hmac::Mac;
            let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(SECRET.as_bytes()).unwrap();
            mac.update(params.as_bytes());
            mac.verify_slice(&hex::decode(sig).unwrap()).unwrap();
            let timestamp = params
                .strip_prefix("timestamp=")
                .unwrap()
                .strip_suffix("&recvWindow=5000")
                .unwrap()
                .parse::<u64>()
                .unwrap();
            assert!((1788849600000..1788849660000).contains(&timestamp));
            if self.fail.get() {
                return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
            }
            return Ok(serde_json::to_vec(&match route {
                "/api/v3/account"=>json!({"uid":9007199254740993u64,"accountType":"SPOT","canTrade":true,"canWithdraw":true,"canDeposit":true,"permissions":["SPOT"],"balances":[{"asset":"USDT","free":"99999999999999999999.9999999999999999999","locked":"0.0000000000000000002"},{"asset":"测试币","free":"0.1","locked":"0.2"}]}),
                "/api/v3/openOrders"=>json!([{"symbol":"BTCUSDT","orderId":9007199254740995u64,"side":"BUY","status":"NEW","price":"100.2","origQty":"0.1","executedQty":"0","origQuoteOrderQty":"0"},{"symbol":"测试币USDT","orderId":9007199254740995u64,"side":"SELL","status":"PARTIALLY_FILLED","price":"2","origQty":"0.5","executedQty":"0.1","origQuoteOrderQty":"0"}]),
                "/sapi/v1/account/apiRestrictions"=>{
                    assert_eq!(endpoint,tradex::provider_io::ProviderEndpoint::BinanceLive);
                    json!({"ipRestrict":true,"createTime":1623840271000u64,"enableReading":true,"enableWithdrawals":false,"enableInternalTransfer":false,"enableMargin":false,"enableFutures":false,"permitsUniversalTransfer":false,"enableVanillaOptions":false,"enableFixApiTrade":false,"enableFixReadOnly":false,"enableSpotAndMarginTrading":true,"enablePortfolioMarginTrading":false})
                },
                _=>panic!("Unexpected Binance operation"),
            }).unwrap());
        }
        if endpoint != tradex::provider_io::ProviderEndpoint::AlpacaPaper {
            assert_eq!(
                headers["Authorization"],
                "Basic UzAyLUZBS0UtS0VZLTU5NDc5MTQ1MzpTMDItRkFLRS1TRUNSRVQtNzA0NTU2OTIx"
            );
            assert!(headers["Authorization"].is_sensitive());
            assert!(!headers.contains_key("APCA-API-KEY-ID"));
            self.calls
                .borrow_mut()
                .push(format!("{}{path}", endpoint.base_url()));
            if self.fail.get() {
                return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
            }
            return Ok(match path {
                "/api/v0/equity/account/summary" => br#"{"id":9007199254740993,"currency":"GBP","cash":{"availableToTrade":1000.1234567890123456789,"reservedForOrders":20.50,"inPies":3.2},"totalValue":1300.25}"#.to_vec(),
                "/api/v0/equity/positions" => br#"[{"instrument":{"ticker":"AAPL_US_EQ","currency":"USD"},"quantity":1.2e-7,"averagePricePaid":150.25,"walletImpact":{"currency":"GBP","currentValue":200.34}}]"#.to_vec(),
                "/api/v0/equity/orders" => br#"[{"id":9007199254740995,"ticker":"MSFT_US_EQ","strategy":"VALUE","side":"BUY","status":"PARTIALLY_FILLED","currency":"GBP","value":10.50,"filledValue":1.23}]"#.to_vec(),
                _ => panic!("Unexpected Trading 212 operation"),
            });
        }
        assert_eq!(headers["APCA-API-KEY-ID"], KEY);
        assert_eq!(headers["APCA-API-SECRET-KEY"], SECRET);
        assert!(headers["APCA-API-SECRET-KEY"].is_sensitive());
        self.calls.borrow_mut().push(path.into());
        if self.fail.get() {
            return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
        }
        let response = match path {
            "/v2/account" => {
                json!({"id":self.identity,"currency":"USD","status":"ACTIVE","cash":"001000.2500","equity":"1200.5500","account_blocked":false,"trading_blocked":false,"shorting_enabled":true})
            }
            "/v2/positions" => {
                json!([{"symbol":"AAPL","qty":"1.2500","market_value":"200.3000","avg_entry_price":"150.0"}])
            }
            "/v2/orders?status=open&limit=500&direction=asc&nested=false" => {
                json!([{"id":"e9124a88-3e5e-45d0-a08f-2b21a8665a91","symbol":"MSFT","side":"buy","qty":null,"notional":"10.5000","filled_qty":"0","status":"new","limit_price":null}])
            }
            _ => panic!("Unexpected provider operation"),
        };
        Ok(serde_json::to_vec(&response).unwrap())
    }
}
