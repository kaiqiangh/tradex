use crate::protocol::{Result, TradeXError};
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
#[derive(Default)]
struct ReadBudget {
    calls: VecDeque<(Instant, Option<String>, u32)>,
}
static READ_BUDGET: OnceLock<Mutex<ReadBudget>> = OnceLock::new();
static STREAM_CONNECTIONS: OnceLock<Mutex<VecDeque<Instant>>> = OnceLock::new();

/// Client-owned ordinary-host headroom, shared by every Spot symbol, source
/// generation and workspace. Local wall-clock changes cannot reset the window.
pub(crate) fn reserve_stream_connection() -> Result<()> {
    let now = Instant::now();
    let mut attempts = STREAM_CONNECTIONS
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .map_err(|_| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
    while attempts
        .front()
        .is_some_and(|at| now.saturating_duration_since(*at) >= Duration::from_secs(300))
    {
        attempts.pop_front();
    }
    // Leave most of the documented shared-IP allowance for other clients.
    if attempts.len() >= 30 {
        return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
    }
    attempts.push_back(now);
    Ok(())
}
pub(crate) fn reserve(account: Option<&str>, weight: u32) -> Result<()> {
    let now = Instant::now();
    let mut b = READ_BUDGET
        .get_or_init(|| Mutex::new(ReadBudget::default()))
        .lock()
        .map_err(|_| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
    while b
        .calls
        .front()
        .is_some_and(|(at, _, _)| now.duration_since(*at) >= Duration::from_secs(60))
    {
        b.calls.pop_front();
    }
    let total: u32 = b.calls.iter().map(|(_, _, w)| w).sum();
    let own: u32 = b
        .calls
        .iter()
        .filter(|(_, a, _)| account.is_some() && a.as_deref() == account)
        .map(|(_, _, w)| w)
        .sum();
    // Reserve ordinary-host P3 metadata headroom rather than assuming the whole
    // provider/IP allowance is ours. Account/source/workspace changes cannot reset it.
    if total.saturating_add(weight) > 3000
        || (account.is_some() && own.saturating_add(weight) > 1500)
    {
        return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
    }
    b.calls.push_back((now, account.map(str::to_owned), weight));
    Ok(())
}
