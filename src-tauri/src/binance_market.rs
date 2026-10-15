//! Public Spot source selection; configuration performs no provider I/O.
mod book;
pub mod stream;
use crate::protocol::{
    BinanceMarketSourceConnection, CommandEnvelope, DataSourceEntry, DataSourceMutation,
    DataSourceProbeKind, DataSourceQuery, DataSourceStatus, MAX_SEQUENCE, Result, TradeXError,
};
use crate::{ControlPlane, payload, provider_order_consumer_allowed};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const SOURCE_ID: &str = "BINANCE_SPOT_PUBLIC";

/// Only configuration persists. No credential, quote, book or lease belongs here.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SavedMarketSource {
    pub generation: u64,
    pub configured: bool,
}
impl SavedMarketSource {
    pub fn state_version(&self, workspace: &str) -> String {
        format!("binance-market:{workspace}:{}", self.generation)
    }
    pub fn validate(&self) -> Result<()> {
        if self.generation > MAX_SEQUENCE {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(())
    }
}

pub(crate) fn policy() -> DataSourceEntry {
    DataSourceEntry {
        source_id: SOURCE_ID.into(),
        provider: "Binance ordinary public Spot market data".into(),
        capabilities: vec!["BTC/USDT and ETH/USDT Hot quotes and bounded depth".into()],
        coverage: "Only canonical BTC/USDT and ETH/USDT on BINANCE; BASE depth, with explicit known price bands. USDT is not USD.".into(),
        latency: "Realtime diff-depth events; usable continuity and actual timestamps must be proven.".into(),
        entitlement: "Public endpoints require no trading credentials. Access alone does not prove permitted financial use.".into(),
        retention: "Ephemeral in-memory book only; permitted durable retention remains UNVERIFIED. No raw tick/book export is granted.".into(),
        redistribution: "UNVERIFIED; no redistribution permission is inferred from public access.".into(),
        commercial_use: "UNVERIFIED; no user-specific commercial or financial-use determination.".into(),
        jurisdictions: "UNVERIFIED for this user; current agreement, local terms and regional access restrictions apply. No host fallback or bypass.".into(),
        official_url: "https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~".into(),
        terms_url: "https://www.binance.com/en/terms".into(),
        reviewed_at: "2026-10-08".into(),
        checked_at: None,
        observed_at: None,
        verified_at: None,
        probe_kind: DataSourceProbeKind::PublicMetadata,
        status: DataSourceStatus::BlockedExternal,
        configured: false,
        availability_reason: "Select the ordinary public Binance Spot source. Saving does not start a network read or lease.".into(),
    }
}

pub(crate) fn connection(
    control: &ControlPlane,
    workspace: &str,
) -> Result<BinanceMarketSourceConnection> {
    control.require_workspace(workspace)?;
    let saved = control.store.as_ref().unwrap().binance_market_source()?;
    let version = saved.state_version(workspace);
    let mut source = policy();
    source.configured = saved.configured;
    if saved.configured {
        source.status = DataSourceStatus::Unverified;
        source.availability_reason = "Public Spot source selected; no continuous Hot book is verified. Open a supported Hot view to start a lease.".into();
        stream::project_source(control, &version, &mut source);
    }
    Ok(BinanceMarketSourceConnection {
        workspace_id: workspace.into(),
        state_version: version,
        configured: saved.configured,
        source,
    })
}

pub(crate) fn selected_once(control: &ControlPlane) -> Result<bool> {
    Ok(control
        .store
        .as_ref()
        .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?
        .binance_market_source()?
        .generation
        > 0)
}

pub(crate) fn metadata(
    control: &mut ControlPlane,
    request: CommandEnvelope,
    consumer: &str,
) -> Result<(Value, Option<String>)> {
    if !provider_order_consumer_allowed(consumer) {
        return Err(TradeXError::new("IPC_ACCESS_DENIED"));
    }
    let workspace = if request.command == "data.binance_market.connection" {
        let input: DataSourceQuery = payload(request.payload)?;
        input.workspace_id
    } else {
        let input: DataSourceMutation = payload(request.payload)?;
        control.require_workspace(&input.workspace_id)?;
        let store = control.store.as_mut().unwrap();
        let mut saved = store.binance_market_source()?;
        if saved.state_version(&input.workspace_id) != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        saved.configured = request.command == "data.binance_market.configure";
        store.save_binance_market_source(saved)?;
        input.workspace_id
    };
    let projected = connection(control, &workspace)?;
    let version = projected.state_version.clone();
    Ok((json!(projected), Some(version)))
}
