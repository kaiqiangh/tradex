//! Bounded quote-stream transport shared by explicit provider adapters.
use crate::protocol::{Result, TradeXError};
use std::{
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use tungstenite::{WebSocket, protocol::WebSocketConfig, stream::MaybeTlsStream};
pub(crate) type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

/// Interrupt an admitted socket even when a peer keeps an individual read alive
/// by trickling bytes. One scoped guard is joined before handing the worker on.
pub(crate) fn guarded_read<T>(
    socket: &mut Socket,
    current: &(impl Fn() -> bool + Sync),
    work: impl FnOnce(&mut Socket) -> Result<T>,
) -> Result<T> {
    let tcp = match socket.get_ref() {
        MaybeTlsStream::Plain(tcp) => tcp,
        MaybeTlsStream::NativeTls(tls) => tls.get_ref(),
        _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
    };
    let interrupt = tcp
        .try_clone()
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    thread::scope(|scope| {
        let (finished, completion) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("tradex-quote-read".into())
            .spawn_scoped(scope, move || {
                let mut retired_at = None;
                loop {
                    if !current() {
                        // Ordinary reads time out within200ms and can send the
                        // existing closing handshake. Force only a stuck read.
                        let at = retired_at.get_or_insert_with(Instant::now);
                        if at.elapsed() >= Duration::from_millis(250) {
                            let _ = interrupt.shutdown(std::net::Shutdown::Both);
                            break;
                        }
                    } else {
                        retired_at = None;
                    }
                    if !matches!(
                        completion.recv_timeout(Duration::from_millis(50)),
                        Err(mpsc::RecvTimeoutError::Timeout)
                    ) {
                        break;
                    }
                }
            })
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        let result = work(socket);
        let _ = finished.send(());
        result
    })
}
static RESOLUTION_IN_FLIGHT: AtomicBool = AtomicBool::new(false);
struct ResolutionPermit;
impl Drop for ResolutionPermit {
    fn drop(&mut self) {
        RESOLUTION_IN_FLIGHT.store(false, Ordering::Release);
    }
}

pub(crate) fn connect(
    url: &str,
    maximum_frame_bytes: usize,
    current: &(impl Fn() -> bool + Sync),
) -> Result<Socket> {
    let deadline = Instant::now() + Duration::from_secs(4);
    let remaining = || {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() || !current() {
            Err(TradeXError::new("PROVIDER_UNAVAILABLE"))
        } else {
            Ok(remaining)
        }
    };
    let parsed = reqwest::Url::parse(url).map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| TradeXError::new("PROVIDER_UNAVAILABLE"))?
        .to_owned();
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    // One deadline covers resolution, TCP, TLS and upgrade; no redirects/proxies.
    // An OS resolver may outlive our wait. Limit it to one thread across all lease attempts.
    let addresses = if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        vec![std::net::SocketAddr::new(ip, port)]
    } else {
        RESOLUTION_IN_FLIGHT
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        let permit = ResolutionPermit;
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("tradex-quote-dns".into())
            .spawn(move || {
                let _permit = permit;
                let addresses = (host.as_str(), port)
                    .to_socket_addrs()
                    .map(|it| it.take(2).collect::<Vec<_>>());
                let _ = sender.send(addresses);
            })
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        loop {
            match receiver.recv_timeout(remaining()?.min(Duration::from_millis(50))) {
                Ok(result) => {
                    break result.map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(_) => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
            }
        }
    };
    let tcp = addresses
        .iter()
        .find_map(|address| TcpStream::connect_timeout(address, remaining().ok()?).ok())
        .ok_or_else(|| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    tcp.set_read_timeout(Some(remaining()?))
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    tcp.set_write_timeout(Some(remaining()?))
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    let config = WebSocketConfig::default()
        .write_buffer_size(8192)
        .max_write_buffer_size(524288)
        .max_message_size(Some(maximum_frame_bytes))
        .max_frame_size(Some(maximum_frame_bytes));
    // Closing a cloned descriptor interrupts both native TLS and HTTP upgrade even
    // when the peer supplies bytes often enough to reset per-operation timeouts.
    // The scoped guard is joined before returning; at most one belongs to our worker.
    let interrupt = tcp
        .try_clone()
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    let (mut socket, _) = thread::scope(|scope| {
        let (finished, completion) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("tradex-quote-handshake".into())
            .spawn_scoped(scope, move || {
                loop {
                    if Instant::now() >= deadline || !current() {
                        let _ = interrupt.shutdown(std::net::Shutdown::Both);
                        break;
                    }
                    if !matches!(
                        completion.recv_timeout(Duration::from_millis(50)),
                        Err(mpsc::RecvTimeoutError::Timeout)
                    ) {
                        break;
                    }
                }
            })
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        let result =
            tungstenite::client_tls_with_config(url, tcp, Some(config), None).map_err(|error| {
                match error {
                    tungstenite::HandshakeError::Failure(tungstenite::Error::Http(response)) => {
                        TradeXError::new(match response.status().as_u16() {
                            401 => "PROVIDER_AUTH_FAILED",
                            403 => "DATA_SOURCE_FEED_DENIED",
                            418 => "PROVIDER_IP_BANNED",
                            429 => "PROVIDER_RATE_LIMITED",
                            _ => "PROVIDER_UNAVAILABLE",
                        })
                    }
                    _ => TradeXError::new("PROVIDER_UNAVAILABLE"),
                }
            });
        let _ = finished.send(());
        remaining()?;
        result
    })?;
    let tcp = match socket.get_mut() {
        MaybeTlsStream::Plain(tcp) => tcp,
        MaybeTlsStream::NativeTls(tls) => tls.get_mut(),
        _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
    };
    tcp.set_read_timeout(Some(Duration::from_millis(200)))
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    Ok(socket)
}
