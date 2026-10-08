use crate::{
    protocol::{Result, SpotDepthLevel, TradeXError},
    provider_json::strict_json,
};
use serde::Deserialize;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
};
fn invalid() -> TradeXError {
    TradeXError::new("PROVIDER_RESPONSE_INVALID")
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Price(String);
impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        let (a, af) = self.0.split_once('.').unwrap_or((&self.0, ""));
        let (b, bf) = other.0.split_once('.').unwrap_or((&other.0, ""));
        a.len().cmp(&b.len()).then(a.cmp(b)).then(af.cmp(bf))
    }
}
impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
fn decimal(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 64 || !s.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return Err(invalid());
    }
    let parts = s.split('.').collect::<Vec<_>>();
    if parts.len() > 2 || parts.iter().any(|p| p.is_empty()) {
        return Err(invalid());
    }
    crate::portfolio::decimal_add(s, "0").map_err(|_| invalid())
}
fn levels(input: Vec<(String, String)>, zero: bool, limit: usize) -> Result<Vec<(Price, String)>> {
    if input.len() > limit {
        return Err(invalid());
    }
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (p, q) in input {
        let p = decimal(&p)?;
        let q = decimal(&q)?;
        if p == "0" || (!zero && q == "0") || !seen.insert(p.clone()) {
            return Err(invalid());
        }
        out.push((Price(p), q));
    }
    Ok(out)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSnapshot {
    #[serde(rename = "lastUpdateId")]
    id: i64,
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvent {
    pub e: String,
    #[serde(rename = "E")]
    pub time: i64,
    pub s: String,
    #[serde(rename = "U")]
    pub first: i64,
    pub u: i64,
    b: Vec<(String, String)>,
    a: Vec<(String, String)>,
}
pub(super) struct Event {
    pub time: i64,
    pub first: i64,
    pub u: i64,
    b: Vec<(Price, String)>,
    a: Vec<(Price, String)>,
}
impl Event {
    pub(super) fn parse(bytes: &[u8], symbol: &str) -> Result<Self> {
        if bytes.len() > 524_288 {
            return Err(invalid());
        }
        let event: RawEvent = strict_json(bytes)?;
        if event.e != "depthUpdate"
            || event.s != symbol
            || event.time <= 0
            || event.first <= 0
            || event.u < event.first
        {
            return Err(invalid());
        }
        Ok(Self {
            time: event.time,
            first: event.first,
            u: event.u,
            b: levels(event.b, true, 5000)?,
            a: levels(event.a, true, 5000)?,
        })
    }
}
pub(super) struct Book {
    pub id: i64,
    bids: BTreeMap<Price, String>,
    asks: BTreeMap<Price, String>,
    bid_floor: Price,
    ask_ceiling: Price,
}
impl Book {
    pub fn snapshot(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 524_288 {
            return Err(invalid());
        }
        let s: RawSnapshot = strict_json(bytes)?;
        if s.id <= 0 {
            return Err(invalid());
        }
        let bids = levels(s.bids, false, 1000)?;
        let asks = levels(s.asks, false, 1000)?;
        if bids.is_empty() || asks.is_empty() {
            return Err(invalid());
        }
        let bids = BTreeMap::from_iter(bids);
        let asks = BTreeMap::from_iter(asks);
        let bid_floor = bids.first_key_value().unwrap().0.clone();
        let ask_ceiling = asks.last_key_value().unwrap().0.clone();
        let book = Self {
            id: s.id,
            bids,
            asks,
            bid_floor,
            ask_ceiling,
        };
        book.best()?;
        Ok(book)
    }
    pub fn bridge(&self, e: &Event) -> bool {
        e.u > self.id && e.first <= self.id && self.id <= e.u
    }
    /// None means obsolete; Some(false) advances only the stream cursor.
    pub fn apply(&mut self, e: Event) -> Result<Option<bool>> {
        if e.u <= self.id {
            return Ok(None);
        }
        if e.first > self.id.checked_add(1).ok_or_else(invalid)? {
            return Err(TradeXError::new("PROVIDER_STREAM_GAP"));
        }
        let bids = e.b;
        let asks = e.a;
        let mut changed = false;
        for (p, q) in bids {
            if p < self.bid_floor {
                continue;
            }
            if q == "0" {
                changed |= self.bids.remove(&p).is_some();
            } else {
                changed |= self.bids.get(&p) != Some(&q);
                self.bids.insert(p, q);
            }
        }
        for (p, q) in asks {
            if p > self.ask_ceiling {
                continue;
            }
            if q == "0" {
                changed |= self.asks.remove(&p).is_some();
            } else {
                changed |= self.asks.get(&p) != Some(&q);
                self.asks.insert(p, q);
            }
        }
        if self.bids.len() > 5000 || self.asks.len() > 5000 {
            return Err(TradeXError::new("PROVIDER_BACKPRESSURE"));
        }
        self.id = e.u;
        self.best()?;
        Ok(Some(changed))
    }
    pub fn best(&self) -> Result<(SpotDepthLevel, SpotDepthLevel)> {
        let (b, bq) = self.bids.last_key_value().ok_or_else(invalid)?;
        let (a, aq) = self.asks.first_key_value().ok_or_else(invalid)?;
        if b > a {
            return Err(invalid());
        }
        Ok((
            SpotDepthLevel {
                price: b.0.clone(),
                quantity: bq.clone(),
            },
            SpotDepthLevel {
                price: a.0.clone(),
                quantity: aq.clone(),
            },
        ))
    }
    pub fn material(&self, symbol: &str) -> (u32, u32, String) {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(format!(
            "BINANCE_SPOT_PUBLIC:{symbol}:BASE\n{}:{}\n",
            self.bid_floor.0, self.ask_ceiling.0
        ));
        for (p, q) in self.bids.iter().rev() {
            hash.update(format!("B:{}:{q}\n", p.0));
        }
        for (p, q) in &self.asks {
            hash.update(format!("A:{}:{q}\n", p.0));
        }
        (
            self.bids.len() as u32,
            self.asks.len() as u32,
            format!("{:x}", hash.finalize()),
        )
    }
    pub fn projection(&self) -> (String, String, Vec<SpotDepthLevel>, Vec<SpotDepthLevel>) {
        let bids = self
            .bids
            .iter()
            .rev()
            .take(20)
            .map(|(p, q)| SpotDepthLevel {
                price: p.0.clone(),
                quantity: q.clone(),
            })
            .collect();
        let asks = self
            .asks
            .iter()
            .take(20)
            .map(|(p, q)| SpotDepthLevel {
                price: p.0.clone(),
                quantity: q.clone(),
            })
            .collect();
        (
            self.bid_floor.0.clone(),
            self.ask_ceiling.0.clone(),
            bids,
            asks,
        )
    }
}
