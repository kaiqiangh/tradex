use crate::protocol::{
    AssetClass, DataSourceEntry, DataSourceStatus, Instrument, InstrumentProviderMapping,
    MarketCatalog, MarketCatalogQuery, MarketDataStatus, MarketDetail, MarketEntitlement,
    MarketFreshness, MarketGetQuery, MarketTier, Result, TradeXError,
};
use duckdb::Connection;
use std::path::Path;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const MAX_HISTORY_ROWS: usize = 100_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalBar {
    pub instrument_id: String,
    pub interval_start: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub source: String,
    pub venue: Option<String>,
    pub provider_timestamp: String,
    pub received_timestamp: String,
    pub entitlement: MarketEntitlement,
    pub freshness: MarketFreshness,
}

pub fn ensure_history(path: &Path) -> Result<()> {
    let database = path.join("market.duckdb");
    if database
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(TradeXError::new("WORKSPACE_PATH_INVALID"));
    }
    let connection =
        Connection::open(&database).map_err(|_| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS ohlcv_1m (
                instrument_id VARCHAR NOT NULL,
                interval_start VARCHAR NOT NULL,
                open VARCHAR NOT NULL,
                high VARCHAR NOT NULL,
                low VARCHAR NOT NULL,
                close VARCHAR NOT NULL,
                volume VARCHAR NOT NULL,
                source VARCHAR NOT NULL,
                venue VARCHAR,
                provider_timestamp VARCHAR NOT NULL,
                received_timestamp VARCHAR NOT NULL,
                entitlement VARCHAR NOT NULL,
                freshness VARCHAR NOT NULL,
                PRIMARY KEY (instrument_id, interval_start, source)
            );",
        )
        .map_err(|_| TradeXError::new("WORKSPACE_OPEN_FAILED"))
}

pub fn insert_history(
    path: &Path,
    bar: &HistoricalBar,
    source: Option<&DataSourceEntry>,
) -> Result<()> {
    validate_bar(bar)?;
    if source
        .filter(|entry| {
            entry.source_id == bar.source && entry.status == DataSourceStatus::Available
        })
        .is_none()
    {
        return Err(TradeXError::new("MARKET_HISTORY_UNAVAILABLE"));
    }
    ensure_history(path)?;
    let database = path.join("market.duckdb");
    let connection =
        Connection::open(&database).map_err(|_| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM ohlcv_1m", [], |row| row.get(0))
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    if count < 0 || count as usize >= MAX_HISTORY_ROWS {
        return Err(TradeXError::new("MARKET_HISTORY_LIMIT"));
    }
    connection
        .execute(
            "INSERT OR REPLACE INTO ohlcv_1m (instrument_id, interval_start, open, high, low, close, volume, source, venue, provider_timestamp, received_timestamp, entitlement, freshness) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            duckdb::params![
                &bar.instrument_id,
                &bar.interval_start,
                &bar.open,
                &bar.high,
                &bar.low,
                &bar.close,
                &bar.volume,
                &bar.source,
                &bar.venue,
                &bar.provider_timestamp,
                &bar.received_timestamp,
                serde_json::to_string(&bar.entitlement).map_err(|_| TradeXError::new("WORKSPACE_OPEN_FAILED"))?.trim_matches('"'),
                serde_json::to_string(&bar.freshness).map_err(|_| TradeXError::new("WORKSPACE_OPEN_FAILED"))?.trim_matches('"'),
            ],
        )
        .map_err(|_| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
    Ok(())
}

pub fn history_count(path: &Path) -> Result<u64> {
    ensure_history(path)?;
    let connection = Connection::open(path.join("market.duckdb"))
        .map_err(|_| TradeXError::new("WORKSPACE_OPEN_FAILED"))?;
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM ohlcv_1m", [], |row| row.get(0))
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    u64::try_from(count).map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))
}

fn validate_bar(bar: &HistoricalBar) -> Result<()> {
    if !validate_instrument_id(&bar.instrument_id)
        || !valid_timestamp(&bar.interval_start, true)
        || !valid_timestamp(&bar.provider_timestamp, false)
        || !valid_timestamp(&bar.received_timestamp, false)
        || !matches!(bar.source.as_str(), "OD-001" | "OD-002")
        || bar
            .venue
            .as_deref()
            .is_some_and(|venue| !valid_identifier(venue, 32))
        || [&bar.open, &bar.high, &bar.low, &bar.close, &bar.volume]
            .iter()
            .any(|value| !valid_decimal(value))
    {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    Ok(())
}

fn valid_text(value: &str, max_len: usize) -> bool {
    !value.is_empty() && value.len() <= max_len && !value.chars().any(char::is_control)
}

fn valid_identifier(value: &str, max_len: usize) -> bool {
    valid_text(value, max_len)
        && value.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
}

fn valid_timestamp(value: &str, minute_boundary: bool) -> bool {
    if !valid_text(value, 64) {
        return false;
    }
    let Ok(timestamp) = OffsetDateTime::parse(value, &Rfc3339) else {
        return false;
    };
    !minute_boundary || (timestamp.second() == 0 && timestamp.nanosecond() == 0)
}

fn valid_decimal(value: &str) -> bool {
    let Ok(normalized) = crate::provider_io::decimal(&serde_json::Value::String(value.into()))
    else {
        return false;
    };
    !normalized.starts_with('-')
}

pub fn instruments() -> Vec<Instrument> {
    vec![
        Instrument {
            instrument_id: "equity:US:AAPL".into(),
            asset_class: AssetClass::Equity,
            symbol: "AAPL".into(),
            base: None,
            quote: None,
            exchange: Some("XNAS".into()),
            currency: "USD".into(),
            display_name: "Apple Inc.".into(),
            providers: vec![InstrumentProviderMapping {
                provider_id: "alpaca".into(),
                provider_symbol: "AAPL".into(),
            }],
        },
        Instrument {
            instrument_id: "equity:US:MSFT".into(),
            asset_class: AssetClass::Equity,
            symbol: "MSFT".into(),
            base: None,
            quote: None,
            exchange: Some("XNAS".into()),
            currency: "USD".into(),
            display_name: "Microsoft Corporation".into(),
            providers: vec![InstrumentProviderMapping {
                provider_id: "alpaca".into(),
                provider_symbol: "MSFT".into(),
            }],
        },
        Instrument {
            instrument_id: "crypto:BTC/USDT:spot".into(),
            asset_class: AssetClass::CryptoSpot,
            symbol: "BTC/USDT".into(),
            base: Some("BTC".into()),
            quote: Some("USDT".into()),
            exchange: None,
            currency: "USDT".into(),
            display_name: "Bitcoin / Tether spot".into(),
            providers: vec![
                InstrumentProviderMapping {
                    provider_id: "binance".into(),
                    provider_symbol: "BTCUSDT".into(),
                },
                InstrumentProviderMapping {
                    provider_id: "bitget".into(),
                    provider_symbol: "BTCUSDT".into(),
                },
            ],
        },
        Instrument {
            instrument_id: "crypto:ETH/USDT:spot".into(),
            asset_class: AssetClass::CryptoSpot,
            symbol: "ETH/USDT".into(),
            base: Some("ETH".into()),
            quote: Some("USDT".into()),
            exchange: None,
            currency: "USDT".into(),
            display_name: "Ethereum / Tether spot".into(),
            providers: vec![
                InstrumentProviderMapping {
                    provider_id: "binance".into(),
                    provider_symbol: "ETHUSDT".into(),
                },
                InstrumentProviderMapping {
                    provider_id: "bitget".into(),
                    provider_symbol: "ETHUSDT".into(),
                },
            ],
        },
    ]
}

pub fn validate_instrument_id(id: &str) -> bool {
    if id.len() > 128 || id.chars().any(char::is_control) {
        return false;
    }
    let Some(rest) = id.strip_prefix("equity:US:") else {
        let Some(pair) = id.strip_prefix("crypto:") else {
            return false;
        };
        let Some(pair) = pair.strip_suffix(":spot") else {
            return false;
        };
        let mut parts = pair.split('/');
        return parts.next().is_some_and(valid_token)
            && parts.next().is_some_and(valid_token)
            && parts.next().is_none();
    };
    !rest.is_empty()
        && rest.len() <= 32
        && rest
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '.' || c == '-')
}

fn valid_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= 16
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '.' || c == '-')
}

fn source_for_tier(tier: &MarketTier) -> &'static str {
    match tier {
        MarketTier::Cold => "OD-002",
        MarketTier::Census | MarketTier::Warm | MarketTier::Hot => "OD-001",
    }
}

pub fn source_id_for_instrument(instrument_id: &str, tier: &MarketTier) -> Option<&'static str> {
    if instrument_id.starts_with("equity:US:") {
        Some(source_for_tier(tier))
    } else {
        None
    }
}

fn source_gate(
    source: Option<&DataSourceEntry>,
    source_id: Option<&str>,
) -> (MarketDataStatus, String) {
    let Some(source_id) = source_id else {
        return (
            MarketDataStatus::Unavailable,
            "No crypto market-data source is selected in the S06 authorization catalog.".into(),
        );
    };
    let Some(source) = source else {
        return (
            MarketDataStatus::Unavailable,
            format!("Market source {source_id} is not present in the S06 authorization catalog."),
        );
    };
    match source.status {
        DataSourceStatus::Available => (
            MarketDataStatus::Unavailable,
            format!(
                "Market adapter for {source_id} is not configured in this build; no quote was returned."
            ),
        ),
        DataSourceStatus::BlockedExternal => (
            MarketDataStatus::BlockedExternal,
            source.availability_reason.clone(),
        ),
        DataSourceStatus::Unavailable => (
            MarketDataStatus::Unavailable,
            source.availability_reason.clone(),
        ),
        DataSourceStatus::Unverified => (
            MarketDataStatus::Unverified,
            source.availability_reason.clone(),
        ),
    }
}

pub fn catalog(
    input: &MarketCatalogQuery,
    source: Option<&DataSourceEntry>,
) -> Result<MarketCatalog> {
    validate_query(&input.workspace_id, &input.query)?;
    let source_id = source_for_tier(&input.tier);
    let (status, availability_reason) = source_gate(source, Some(source_id));
    let query = input.query.trim().to_ascii_lowercase();
    let instruments = instruments()
        .into_iter()
        .filter(|instrument| {
            query.is_empty()
                || instrument
                    .instrument_id
                    .to_ascii_lowercase()
                    .contains(&query)
                || instrument.symbol.to_ascii_lowercase().contains(&query)
                || instrument
                    .display_name
                    .to_ascii_lowercase()
                    .contains(&query)
        })
        .collect();
    Ok(MarketCatalog {
        workspace_id: input.workspace_id.clone(),
        query: input.query.trim().into(),
        tier: input.tier.clone(),
        source_id: Some(source_id.into()),
        status,
        availability_reason,
        instruments,
    })
}

pub fn detail(input: &MarketGetQuery, source: Option<&DataSourceEntry>) -> Result<MarketDetail> {
    if !valid_text(&input.workspace_id, 128) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    if !validate_instrument_id(&input.instrument_id) {
        return Err(TradeXError::new("MARKET_INSTRUMENT_INVALID"));
    }
    let instrument = instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == input.instrument_id)
        .ok_or_else(|| TradeXError::new("MARKET_INSTRUMENT_NOT_FOUND"))?;
    let source_id = source_id_for_instrument(&input.instrument_id, &input.tier);
    let source = source.filter(|entry| Some(entry.source_id.as_str()) == source_id);
    let (status, availability_reason) = source_gate(source, source_id);
    Ok(MarketDetail {
        workspace_id: input.workspace_id.clone(),
        instrument,
        tier: input.tier.clone(),
        source_id: source_id.map(str::to_owned),
        status,
        availability_reason,
        snapshot: None,
    })
}

fn validate_query(workspace_id: &str, query: &str) -> Result<()> {
    if !valid_text(workspace_id, 128) || query.len() > 120 || query.chars().any(char::is_control) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{DataSourceProbeKind, DataSourceStatus};
    use tempfile::tempdir;

    fn blocked_source(id: &str) -> DataSourceEntry {
        DataSourceEntry {
            source_id: id.into(),
            provider: "Test".into(),
            capabilities: vec!["market".into()],
            coverage: "test".into(),
            latency: "test".into(),
            entitlement: "test".into(),
            retention: "test".into(),
            redistribution: "test".into(),
            commercial_use: "test".into(),
            jurisdictions: "test".into(),
            official_url: "https://example.com".into(),
            terms_url: "https://example.com/terms".into(),
            reviewed_at: "2026-09-14".into(),
            checked_at: None,
            observed_at: None,
            probe_kind: DataSourceProbeKind::CredentialedMetadata,
            status: DataSourceStatus::BlockedExternal,
            configured: false,
            verified_at: None,
            availability_reason: "entitlement required".into(),
        }
    }

    #[test]
    fn canonical_catalog_is_searchable_and_blocked_source_is_explicit() {
        let input = MarketCatalogQuery {
            workspace_id: "w".into(),
            query: "aapl".into(),
            tier: MarketTier::Hot,
        };
        let catalog = catalog(&input, Some(&blocked_source("OD-001"))).unwrap();
        assert_eq!(catalog.instruments.len(), 1);
        assert_eq!(catalog.instruments[0].instrument_id, "equity:US:AAPL");
        assert_eq!(catalog.status, MarketDataStatus::BlockedExternal);
    }

    #[test]
    fn duckdb_history_is_separate_and_reopens() {
        let dir = tempdir().unwrap();
        ensure_history(dir.path()).unwrap();
        let blocked = blocked_source("OD-002");
        assert_eq!(
            insert_history(dir.path(), &history_bar(), Some(&blocked))
                .unwrap_err()
                .code,
            "MARKET_HISTORY_UNAVAILABLE"
        );
        assert_eq!(history_count(dir.path()).unwrap(), 0);
        let mut available = blocked_source("OD-002");
        available.status = DataSourceStatus::Available;
        available.configured = true;
        insert_history(dir.path(), &history_bar(), Some(&available)).unwrap();
        assert_eq!(history_count(dir.path()).unwrap(), 1);
        assert!(dir.path().join("market.duckdb").exists());
    }

    #[test]
    fn history_rejects_invalid_decimal_timestamp_and_venue() {
        let dir = tempdir().unwrap();
        let mut source = blocked_source("OD-002");
        source.status = DataSourceStatus::Available;
        let mut bar = history_bar();
        bar.open = "1e3".into();
        assert_eq!(
            insert_history(dir.path(), &bar, Some(&source))
                .unwrap_err()
                .code,
            "IPC_PAYLOAD_INVALID"
        );
        bar = history_bar();
        bar.interval_start = "2026-09-14T00:00:30Z".into();
        assert_eq!(
            insert_history(dir.path(), &bar, Some(&source))
                .unwrap_err()
                .code,
            "IPC_PAYLOAD_INVALID"
        );
        bar = history_bar();
        bar.venue = Some("XNAS\n".into());
        assert_eq!(
            insert_history(dir.path(), &bar, Some(&source))
                .unwrap_err()
                .code,
            "IPC_PAYLOAD_INVALID"
        );
    }

    fn history_bar() -> HistoricalBar {
        HistoricalBar {
            instrument_id: "equity:US:AAPL".into(),
            interval_start: "2026-09-14T00:00:00Z".into(),
            open: "1.00".into(),
            high: "2.00".into(),
            low: "0.50".into(),
            close: "1.50".into(),
            volume: "10".into(),
            source: "OD-002".into(),
            venue: Some("XNAS".into()),
            provider_timestamp: "2026-09-14T00:00:00Z".into(),
            received_timestamp: "2026-09-14T00:00:01Z".into(),
            entitlement: MarketEntitlement::Delayed,
            freshness: MarketFreshness::Healthy,
        }
    }
}
