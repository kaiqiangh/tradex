//! External public HTTP/WS protocol for browser QA; no Control Plane state writes.
use serde_json::{Value, json};
use std::{
    io::{self, Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};
use tradex::{
    binance_market::stream::BinanceStreamConnector,
    protocol::Result,
    provider_io::{BrokerHttp, CredentialVault, Credentials},
};

pub struct NoTradingKey;
impl CredentialVault for NoTradingKey {
    fn put(&self, _: &str, _: &Credentials) -> Result<()> {
        panic!("Public market source wrote a trading key")
    }
    fn get(&self, _: &str) -> Result<Credentials> {
        panic!("Public market source read a trading key")
    }
    fn remove(&self, _: &str) -> Result<()> {
        panic!("Public market source deleted a trading key")
    }
}

#[derive(Default)]
struct Ledger {
    connections: AtomicU64,
    closed: AtomicU64,
    frames: AtomicU64,
    snapshot_base: AtomicU64,
    requests: Mutex<Vec<String>>,
    streams: Mutex<Vec<String>>,
}
pub struct Fixture {
    pub http: Arc<BrokerHttp>,
    ws_address: String,
    ledger: Arc<Ledger>,
}
impl Fixture {
    pub fn connector(&self, instrument: &str) -> Result<BinanceStreamConnector> {
        let symbol = match instrument {
            "crypto:BTC/USDT:spot" => "btcusdt",
            "crypto:ETH/USDT:spot" => "ethusdt",
            _ => {
                return Err(tradex::protocol::TradeXError::new(
                    "MARKET_INSTRUMENT_INVALID",
                ));
            }
        };
        BinanceStreamConnector::for_loopback_test(&format!(
            "ws://{}/ws/{symbol}@depth@100ms",
            self.ws_address
        ))
    }
    pub fn inspect(&self) -> Value {
        json!({"connections":self.ledger.connections.load(Ordering::Acquire),"closedConnections":self.ledger.closed.load(Ordering::Acquire),"frames":self.ledger.frames.load(Ordering::Acquire),"requests":*self.ledger.requests.lock().unwrap(),"streams":*self.ledger.streams.lock().unwrap()})
    }
}

pub fn start() -> io::Result<Fixture> {
    const START: u64 = 9_007_199_254_740_992;
    let rest = TcpListener::bind("127.0.0.1:0")?;
    let stream = TcpListener::bind("127.0.0.1:0")?;
    let http = Arc::new(
        BrokerHttp::for_loopback_test(&format!("http://{}", rest.local_addr()?))
            .map_err(|_| io::Error::other("Invalid public fixture address"))?,
    );
    let ws_address = stream.local_addr()?.to_string();
    let ledger = Arc::new(Ledger::default());
    ledger.snapshot_base.store(START, Ordering::Release);
    let observed = ledger.clone();
    thread::spawn(move || {
        for accepted in rest.incoming() {
            let Ok(mut socket) = accepted else { break };
            let _ = socket.set_read_timeout(Some(Duration::from_millis(500)));
            let mut bytes = Vec::new();
            while !bytes.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                if socket.read_exact(&mut byte).is_err() {
                    break;
                }
                bytes.push(byte[0]);
                assert!(bytes.len() < 16_384);
            }
            let request = String::from_utf8(bytes).unwrap();
            assert!(!request.to_ascii_lowercase().contains("x-mbx-apikey"));
            assert!(!request.contains("signature="));
            let mut line = request.lines().next().unwrap().split_whitespace();
            assert_eq!(line.next(), Some("GET"));
            let path = line.next().unwrap();
            let mut paths = observed.requests.lock().unwrap();
            if paths.len() < 64 {
                paths.push(path.into());
            }
            drop(paths);
            let base = observed.snapshot_base.load(Ordering::Acquire);
            let body = match path {
                "/api/v3/time" => {
                    json!({"serverTime":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64})
                }
                "/api/v3/depth?symbol=BTCUSDT&limit=1000" => {
                    json!({"lastUpdateId":base,"bids":[["60000","1"],["59900","2"]],"asks":[["60001","1"],["60100","2"]]})
                }
                "/api/v3/depth?symbol=ETHUSDT&limit=1000" => {
                    json!({"lastUpdateId":base,"bids":[["3500","1"],["3499","2"]],"asks":[["3501","1"],["3502","2"]]})
                }
                _ => panic!("Unexpected public market route"),
            };
            let body = serde_json::to_vec(&body).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = socket
                .write_all(response.as_bytes())
                .and_then(|_| socket.write_all(&body));
        }
    });
    let observed = ledger.clone();
    thread::spawn(move || {
        for accepted in stream.incoming() {
            let Ok(tcp) = accepted else { break };
            let _ = tcp.set_read_timeout(Some(Duration::from_secs(2)));
            let mut symbol = "";
            let Ok(mut socket) = tungstenite::accept_hdr(
                tcp,
                |request: &tungstenite::handshake::server::Request,
                 response: tungstenite::handshake::server::Response| {
                    assert!(!request.headers().contains_key("x-mbx-apikey"));
                    symbol = match request.uri().path() {
                        "/ws/btcusdt@depth@100ms" => "BTCUSDT",
                        "/ws/ethusdt@depth@100ms" => "ETHUSDT",
                        _ => panic!("Unexpected public market stream"),
                    };
                    observed
                        .streams
                        .lock()
                        .unwrap()
                        .push(request.uri().path().into());
                    Ok(response)
                },
            ) else {
                continue;
            };
            let connection = observed.connections.fetch_add(1, Ordering::AcqRel) + 1;
            let base = START + connection * 1_000_000;
            observed.snapshot_base.store(base, Ordering::Release);
            let _ = socket
                .get_mut()
                .set_read_timeout(Some(Duration::from_millis(200)));
            let (bid, ask) = if symbol == "BTCUSDT" {
                ("60000", "60001")
            } else {
                ("3500", "3501")
            };
            let mut cursor = base;
            loop {
                let event = json!({"e":"depthUpdate","E":(time::OffsetDateTime::now_utc().unix_timestamp_nanos()/1_000_000) as i64,"s":symbol,"U":cursor,"u":cursor+1,"b":[[bid,"0.5"]],"a":[[ask,"0.75"]]});
                if socket
                    .send(tungstenite::Message::Text(event.to_string().into()))
                    .is_err()
                {
                    break;
                }
                observed.frames.fetch_add(1, Ordering::AcqRel);
                cursor += 2;
                match socket.read() {
                    Ok(tungstenite::Message::Close(_)) => {
                        let _ = socket.flush();
                        break;
                    }
                    Ok(tungstenite::Message::Ping(bytes)) => {
                        let _ = socket.send(tungstenite::Message::Pong(bytes));
                    }
                    Err(tungstenite::Error::Io(e))
                        if matches!(
                            e.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) =>
                    {
                        ()
                    }
                    Err(_) => break,
                    _ => (),
                }
            }
            observed.closed.fetch_add(1, Ordering::AcqRel);
        }
    });
    Ok(Fixture {
        http,
        ws_address,
        ledger,
    })
}
