use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Clone, Copy)]
pub enum MutationResult {
    Accepted,
    Rejected,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct CapturedRequest {
    pub method: String,
    pub path: String,
    pub body: Value,
    pub authorization_present: bool,
}

#[derive(Default)]
struct State {
    remote_account_id: Option<String>,
    next_results: VecDeque<MutationResult>,
    requests: Vec<CapturedRequest>,
}

pub struct FakeLiveProvider {
    base_url: String,
    state: Arc<Mutex<State>>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl FakeLiveProvider {
    pub fn start() -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let base_url = format!("http://{}", listener.local_addr()?);
        let state = Arc::new(Mutex::new(State::default()));
        let server_state = state.clone();
        let running = Arc::new(AtomicBool::new(true));
        let server_running = running.clone();
        let thread = thread::spawn(move || {
            while server_running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => handle(stream, &server_state),
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            base_url,
            state,
            running,
            thread: Some(thread),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn set_remote_account_id(&self, remote_account_id: String) {
        self.state.lock().unwrap().remote_account_id = Some(remote_account_id);
    }

    pub fn set_next_result(&self, result: MutationResult) {
        self.state.lock().unwrap().next_results.push_back(result);
    }

    pub fn requests(&self) -> Vec<CapturedRequest> {
        self.state.lock().unwrap().requests.clone()
    }
}

impl Drop for FakeLiveProvider {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn handle(stream: TcpStream, state: &Arc<Mutex<State>>) {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let (request, body, mut stream) = match read_request(stream) {
        Ok(request) => request,
        Err(_) => return,
    };
    let body = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body).unwrap_or(Value::Null)
    };
    let mut state = state.lock().unwrap();
    state.requests.push(CapturedRequest {
        method: request.method.clone(),
        path: request.path.clone(),
        body: body.clone(),
        authorization_present: request.authorization_present,
    });
    let response = match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/api/v0/equity/account/summary") => Some(
            state
                .remote_account_id
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .map(|id| (200, json!({"id":id}).to_string().into_bytes()))
                .unwrap_or_else(|| (500, b"{}".to_vec())),
        ),
        ("GET", "/api/v0/equity/orders/9007199254740996") => Some((
            200,
            json!({
                "id":9007199254740996u64,
                "ticker":"MSFT_US_EQ",
                "strategy":"QUANTITY",
                "side":"BUY",
                "type":"LIMIT",
                "timeInForce":"DAY",
                "status":"PARTIALLY_FILLED",
                "quantity":1,
                "filledQuantity":0.25,
                "filledValue":32.5
            })
            .to_string()
            .into_bytes(),
        )),
        ("POST", "/api/v0/equity/orders/market" | "/api/v0/equity/orders/limit") => {
            mutation_response(&mut state, &body, None)
        }
        ("DELETE", path) if path.starts_with("/api/v0/equity/orders/") => {
            mutation_response(&mut state, &Value::Null, Some(path))
        }
        _ => Some((404, b"{}".to_vec())),
    };
    let Some(response) = response else {
        let _ = stream.shutdown(Shutdown::Both);
        return;
    };
    let reason = if response.0 == 200 { "OK" } else { "Rejected" };
    let _ = write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.0,
        reason,
        response.1.len()
    );
    let _ = stream.write_all(&response.1);
    let _ = stream.shutdown(Shutdown::Both);
}

struct Request {
    method: String,
    path: String,
    authorization_present: bool,
}

fn read_request(stream: TcpStream) -> io::Result<(Request, Vec<u8>, TcpStream)> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let path = parts.next().unwrap_or_default().to_owned();
    let mut authorization_present = false;
    let mut content_length = 0;
    loop {
        line.clear();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("authorization") {
                authorization_present = !value.trim().is_empty();
            } else if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(usize::MAX);
            }
        }
    }
    if content_length > 16 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "request too large",
        ));
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    Ok((
        Request {
            method,
            path,
            authorization_present,
        },
        body,
        reader.into_inner(),
    ))
}

fn mutation_response(
    state: &mut State,
    body: &Value,
    path: Option<&str>,
) -> Option<(u16, Vec<u8>)> {
    match state
        .next_results
        .pop_front()
        .unwrap_or(MutationResult::Accepted)
    {
        MutationResult::Accepted => {
            let value = if let Some(path) = path {
                json!({"acknowledged":true,"path":path})
            } else {
                json!({
                    "id":901,
                    "ticker":body.get("ticker").and_then(Value::as_str).unwrap_or_default(),
                    "status":"NEW"
                })
            };
            Some((200, value.to_string().into_bytes()))
        }
        MutationResult::Rejected => Some((400, br#"{"error":"synthetic rejection"}"#.to_vec())),
        MutationResult::Unknown => None,
    }
}
