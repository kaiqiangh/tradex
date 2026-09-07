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
    fn get(&self, path: &str, headers: reqwest::header::HeaderMap) -> Result<Vec<u8>> {
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
