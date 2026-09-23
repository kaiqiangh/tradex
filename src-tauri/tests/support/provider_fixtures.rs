#[path = "bitget_fixtures.rs"]
pub mod bitget;
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
};
use tradex::{
    protocol::{Result, TradeXError},
    provider_io::{
        CredentialVault, Credentials, ProviderEndpoint, ProviderHttp, ProviderHttpMethod,
        ProviderHttpResponse,
    },
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
        if reference.split('/').nth(1) == Some("bitget") {
            bitget::credentials()
        } else {
            credentials()
        }
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
    pub alpaca_posts: RefCell<Vec<Value>>,
    pub alpaca_order: RefCell<Option<Value>>,
    pub alpaca_order_history: RefCell<Vec<Value>>,
    pub alpaca_fills: RefCell<Vec<Value>>,
    pub alpaca_repeat_order_cursor: Cell<bool>,
    pub alpaca_delete_calls: RefCell<Vec<String>>,
    pub alpaca_delete_status: Cell<Option<u16>>,
    pub alpaca_delete_confirms_cancel: Cell<bool>,
    pub alpaca_delete_order_status: RefCell<Option<String>>,
    pub alpaca_lookup_misses: Cell<u32>,
    pub alpaca_post_timeout: Cell<bool>,
    pub alpaca_order_currency: RefCell<String>,
    pub alpaca_asset: RefCell<Value>,
    pub alpaca_position: RefCell<Option<Value>>,
    pub alpaca_post_status: Cell<Option<u16>>,
}
impl Default for Http {
    fn default() -> Self {
        Self {
            fail: Cell::new(false),
            identity: "81161e77-bafd-44bb-b2a0-60b9055e3cd4".into(),
            calls: RefCell::new(vec![]),
            alpaca_posts: RefCell::new(vec![]),
            alpaca_order: RefCell::new(None),
            alpaca_order_history: RefCell::new(vec![]),
            alpaca_fills: RefCell::new(vec![]),
            alpaca_repeat_order_cursor: Cell::new(false),
            alpaca_delete_calls: RefCell::new(vec![]),
            alpaca_delete_status: Cell::new(None),
            alpaca_delete_confirms_cancel: Cell::new(false),
            alpaca_delete_order_status: RefCell::new(None),
            alpaca_lookup_misses: Cell::new(0),
            alpaca_post_timeout: Cell::new(false),
            alpaca_order_currency: RefCell::new("USD".into()),
            alpaca_asset: RefCell::new(json!({
                "id":"b0b6dd9d-8b9b-48a9-ba46-b9d54906e15b",
                "class":"us_equity","exchange":"NASDAQ","symbol":"AAPL",
                "status":"active","tradable":true,"fractionable":true
            })),
            alpaca_position: RefCell::new(Some(json!({
                "asset_id":"b0b6dd9d-8b9b-48a9-ba46-b9d54906e15b",
                "symbol":"AAPL","asset_class":"us_equity","side":"long",
                "qty":"10","qty_available":"10"
            }))),
            alpaca_post_status: Cell::new(None),
        }
    }
}
impl ProviderHttp for Http {
    fn request(
        &self,
        endpoint: ProviderEndpoint,
        method: ProviderHttpMethod,
        path: &str,
        headers: reqwest::header::HeaderMap,
        body: Option<&Value>,
    ) -> Result<ProviderHttpResponse> {
        if endpoint != ProviderEndpoint::AlpacaPaper {
            return match (method, body) {
                (ProviderHttpMethod::Get, None) => self
                    .get(endpoint, path, headers)
                    .map(|body| ProviderHttpResponse { status: 200, body }),
                _ => Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
            };
        }
        assert_eq!(headers["APCA-API-KEY-ID"], KEY);
        assert_eq!(headers["APCA-API-SECRET-KEY"], SECRET);
        assert!(headers["APCA-API-KEY-ID"].is_sensitive());
        assert!(headers["APCA-API-SECRET-KEY"].is_sensitive());
        assert!(!headers.contains_key("Authorization"));
        self.calls
            .borrow_mut()
            .push(format!("{}{path}", endpoint.base_url()));
        match (method, path, body) {
            (ProviderHttpMethod::Get, "/v2/account", None) => {
                let body = self.get(endpoint, path, headers)?;
                let mut account: Value = serde_json::from_slice(&body).unwrap();
                account["currency"] = self.alpaca_order_currency.borrow().clone().into();
                Ok(ProviderHttpResponse {
                    status: 200,
                    body: serde_json::to_vec(&account).unwrap(),
                })
            }
            (ProviderHttpMethod::Get, "/v2/assets/AAPL", None) => Ok(ProviderHttpResponse {
                status: 200,
                body: serde_json::to_vec(&self.alpaca_asset.borrow().clone()).unwrap(),
            }),
            (ProviderHttpMethod::Get, "/v2/positions/AAPL", None) => {
                match self.alpaca_position.borrow().clone() {
                    Some(position) => Ok(ProviderHttpResponse {
                        status: 200,
                        body: serde_json::to_vec(&position).unwrap(),
                    }),
                    None => Ok(ProviderHttpResponse {
                        status: 404,
                        body: Vec::new(),
                    }),
                }
            }
            (ProviderHttpMethod::Get, path, None)
                if path == "/v2/orders?status=all&limit=100&direction=desc&nested=false"
                    || path.starts_with("/v2/orders?status=all&limit=100&direction=desc&nested=false&before_order_id=") =>
            {
                let mut orders = self.alpaca_order_history.borrow().clone();
                if let Some(order) = self.alpaca_order.borrow().clone()
                    && !orders.iter().any(|item| item["id"] == order["id"])
                {
                    orders.push(order);
                }
                orders.sort_by(|left, right| right["submitted_at"].as_str().cmp(&left["submitted_at"].as_str()));
                let start = path
                    .split_once("before_order_id=")
                    .and_then(|(_, cursor)| {
                        if self.alpaca_repeat_order_cursor.get() {
                            Some(0)
                        } else {
                            orders.iter().position(|order| order["id"] == cursor).map(|index| index + 1)
                        }
                    })
                    .unwrap_or(0);
                let values = orders.into_iter().skip(start).take(100).collect::<Vec<_>>();
                Ok(ProviderHttpResponse { status: 200, body: serde_json::to_vec(&values).unwrap() })
            }
            (ProviderHttpMethod::Get, path, None) if path.starts_with("/v2/orders/") => {
                let order_id = path.trim_start_matches("/v2/orders/");
                let order = self.alpaca_order_history.borrow().iter().find(|order| order["id"] == order_id).cloned()
                    .or_else(|| self.alpaca_order.borrow().clone().filter(|order| order["id"] == order_id));
                match order {
                    Some(order) => Ok(ProviderHttpResponse { status: 200, body: serde_json::to_vec(&order).unwrap() }),
                    None => Ok(ProviderHttpResponse { status: 404, body: Vec::new() }),
                }
            }
            (ProviderHttpMethod::Get, path, None)
                if path == "/v2/account/activities/FILL?page_size=100&direction=desc"
                    || path.starts_with("/v2/account/activities/FILL?page_size=100&direction=desc&page_token=") =>
            {
                let fills = self.alpaca_fills.borrow().clone();
                let start = path
                    .split_once("page_token=")
                    .and_then(|(_, cursor)| fills.iter().position(|fill| fill["id"] == cursor).map(|index| index + 1))
                    .unwrap_or(0);
                let values = fills.into_iter().skip(start).take(100).collect::<Vec<_>>();
                Ok(ProviderHttpResponse { status: 200, body: serde_json::to_vec(&values).unwrap() })
            }
            (ProviderHttpMethod::Get, path, None)
                if path.starts_with("/v2/orders:by_client_order_id?client_order_id=") =>
            {
                if self.alpaca_lookup_misses.get() > 0 {
                    self.alpaca_lookup_misses
                        .set(self.alpaca_lookup_misses.get() - 1);
                    return Ok(ProviderHttpResponse {
                        status: 404,
                        body: Vec::new(),
                    });
                }
                match self.alpaca_order.borrow().clone() {
                    Some(order) => Ok(ProviderHttpResponse {
                        status: 200,
                        body: serde_json::to_vec(&order).unwrap(),
                    }),
                    None => Ok(ProviderHttpResponse {
                        status: 404,
                        body: Vec::new(),
                    }),
                }
            }
            (ProviderHttpMethod::Post, "/v2/orders", Some(request)) => {
                self.alpaca_posts.borrow_mut().push(request.clone());
                if let Some(status) = self.alpaca_post_status.get() {
                    return Ok(ProviderHttpResponse {
                        status,
                        body: Vec::new(),
                    });
                }
                let order = json!({
                    "id":"18c65e3e-feb0-4576-99e2-36e6f047d84d",
                    "asset_id":"b0b6dd9d-8b9b-48a9-ba46-b9d54906e15b",
                    "client_order_id":request["client_order_id"],
                    "symbol":request["symbol"],
                    "asset_class":"us_equity","side":request["side"],
                    "type":request["type"],"time_in_force":request["time_in_force"],
                    "qty":request.get("qty"),"notional":request.get("notional"),
                    "limit_price":request.get("limit_price"),"status":"accepted",
                    "filled_qty":"0","submitted_at":"2026-09-23T10:00:00Z","created_at":"2026-09-23T10:00:00Z"
                });
                *self.alpaca_order.borrow_mut() = Some(order);
                if self.alpaca_post_timeout.get() {
                    Err(TradeXError::new("PROVIDER_UNAVAILABLE"))
                } else {
                    Ok(ProviderHttpResponse {
                        status: 201,
                        body: serde_json::to_vec(self.alpaca_order.borrow().as_ref().unwrap())
                            .unwrap(),
                    })
                }
            }
            (ProviderHttpMethod::Delete, path, None) if path.starts_with("/v2/orders/") => {
                let order_id = path.trim_start_matches("/v2/orders/").to_owned();
                self.alpaca_delete_calls.borrow_mut().push(order_id.clone());
                let status = self.alpaca_delete_status.get().unwrap_or(204);
                if status == 204 && (self.alpaca_delete_confirms_cancel.get() || self.alpaca_delete_order_status.borrow().is_some()) {
                    let provider_status = self.alpaca_delete_order_status.borrow().clone().unwrap_or_else(|| "canceled".into());
                    if let Some(order) = self.alpaca_order.borrow_mut().as_mut().filter(|order| order["id"] == order_id) {
                        order["status"] = provider_status.clone().into();
                        if provider_status == "filled" {
                            order["filled_qty"] = order["qty"].clone();
                        }
                    }
                    if let Some(order) = self.alpaca_order_history.borrow_mut().iter_mut().find(|order| order["id"] == order_id) {
                        order["status"] = provider_status.clone().into();
                        if provider_status == "filled" {
                            order["filled_qty"] = order["qty"].clone();
                        }
                    }
                }
                Ok(ProviderHttpResponse { status, body: Vec::new() })
            }
            _ => Err(TradeXError::new("PROVIDER_UNSUPPORTED")),
        }
    }

    fn get(
        &self,
        endpoint: tradex::provider_io::ProviderEndpoint,
        path: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<Vec<u8>> {
        if matches!(
            endpoint,
            tradex::provider_io::ProviderEndpoint::BitgetDemo
                | tradex::provider_io::ProviderEndpoint::BitgetLive
        ) {
            return bitget::Http(RefCell::new(vec![])).get(endpoint, path, headers);
        }
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
                json!({"id":self.identity,"currency":"USD","status":"ACTIVE","cash":"001000.2500","equity":"1200.5500","buying_power":"1100.9876543210123456789","account_blocked":false,"trading_blocked":false,"trade_suspended_by_user":false,"shorting_enabled":true})
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
