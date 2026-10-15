//! External loopback stock protocol for isolated browser QA; never compiled into desktop production.
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{io, net::TcpListener, thread, time::Duration};
use tradex::quote_source::hot::StockStreamConnector;
use tungstenite::Message;

pub fn start() -> io::Result<StockStreamConnector> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let drop_once = Arc::new(AtomicBool::new(
        std::env::var_os("TRADEX_QUOTE_STREAM_DROP_ONCE").is_some(),
    ));
    let wait_for_quote = std::env::var_os("TRADEX_QUOTE_STREAM_WAIT_FOR_QUOTE").is_some();
    thread::spawn(move || {
        for accepted in listener.incoming() {
            let Ok(tcp) = accepted else { break };
            // Each synthetic connection still exercises a real WebSocket handshake and frames.
            let drop_once = drop_once.clone();
            thread::spawn(move || {
                let _ = tcp.set_read_timeout(Some(Duration::from_secs(30)));
                let Ok(mut socket) = tungstenite::accept(tcp) else {
                    return;
                };
                let result = (|| -> Result<(), tungstenite::Error> {
                    socket.send(Message::text(r#"[{"T":"success","msg":"connected"}]"#))?;
                    let auth: Value =
                        serde_json::from_str(socket.read()?.to_text()?).unwrap_or(Value::Null);
                    if auth
                        != json!({"action":"auth","key":super::fixtures::KEY,"secret":super::fixtures::SECRET})
                    {
                        socket.send(Message::text(r#"[{"T":"error","code":402}]"#))?;
                        return Ok(());
                    }
                    socket.send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))?;
                    let subscription: Value =
                        serde_json::from_str(socket.read()?.to_text()?).unwrap_or(Value::Null);
                    let symbol = subscription["quotes"]
                        .as_array()
                        .filter(|items| items.len() == 1)
                        .and_then(|items| items[0].as_str());
                    let Some(symbol) = symbol.filter(|symbol| ["AAPL", "MSFT"].contains(symbol))
                    else {
                        return Ok(());
                    };
                    socket.send(Message::text(
                        json!([{"T":"subscription","quotes":[symbol],"trades":[],"bars":[]}])
                            .to_string(),
                    ))?;
                    if !wait_for_quote {
                        socket.send(Message::text(format!(r#"[{{"T":"q","S":"{symbol}","t":"2026-09-30T14:10:00.123456789Z","bx":"P","bp":250.7234567890123456789,"bs":211,"ax":"Q","ap":250.8234567890123456789,"as":365,"c":["R"],"z":"C"}}]"#)))?;
                    }
                    if drop_once.swap(false, Ordering::AcqRel) {
                        socket.close(None)?;
                        let _ = socket.read();
                        return Ok(());
                    }
                    loop {
                        match socket.read()? {
                            Message::Close(_) => break,
                            Message::Ping(bytes) => socket.send(Message::Pong(bytes))?,
                            _ => (),
                        }
                    }
                    Ok(())
                })();
                let _ = result;
                let _ = socket.close(None);
            });
        }
    });
    StockStreamConnector::for_loopback_test(&format!("ws://{address}/v2/sip"))
        .map_err(|_| io::Error::other("Invalid quote fixture address"))
}
