//! Ephemeral, fixed BNB/USDT fee input. It never enters the tradable catalogue.
use crate::protocol::{HotQuoteStatus, MarketSnapshot};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(crate) const INTERNAL_ID: &str = "internal:BNB/USDT:fee";

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotFeeMarketAcquire {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_source_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeeMarketMetadata {
    #[schemars(length(min = 1, max = 32))]
    pub provider_symbol: String,
    #[schemars(length(min = 1, max = 16))]
    pub base_asset: String,
    #[schemars(length(min = 1, max = 16))]
    pub quote_asset: String,
    #[schemars(length(min = 1, max = 32))]
    pub market_status: String,
    #[schemars(length(min = 1, max = 64))]
    pub received_at: String,
    #[schemars(length(min = 64, max = 64))]
    pub original_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotFeeMarketProjection {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub source_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub lease_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub generation: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_generation: String,
    #[schemars(length(min = 1, max = 256))]
    pub source_version: String,
    #[schemars(range(min = 0, max = 9007199254740991u64))]
    pub sequence: u64,
    #[schemars(range(min = 0, max = 3))]
    pub reconnect_attempt: u32,
    pub status: HotQuoteStatus,
    #[schemars(length(min = 1, max = 1024))]
    pub reason: String,
    pub metadata: Option<FeeMarketMetadata>,
    /// Current/raw fee inputs are protected ephemeral reads, never saved risk history.
    pub snapshot: Option<MarketSnapshot>,
}
