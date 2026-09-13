use std::{io::Read, time::Duration};

use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::protocol::{
    DataSourceEntry, DataSourceProbeKind, DataSourceStatus, Result, TradeXError,
};

const SOURCE_REVIEWED_AT: &str = "2026-09-13";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RESPONSE_BYTES: usize = 1_048_576;
const USER_AGENT: &str = "TradeX-local-research/0.1 (local workspace)";
const SEC_SUBMISSIONS_URL: &str = "https://data.sec.gov/submissions/CIK0000320193.json";
const SEC_XBRL_FRAMES_URL: &str =
    "https://data.sec.gov/api/xbrl/frames/us-gaap/Revenues/USD/CY2024Q4.json";
const ECB_EXR_URL: &str = "https://data-api.ecb.europa.eu/service/data/EXR/D.USD.EUR.SP00.A?format=csvdata&lastNObservations=1";

pub fn entries() -> Vec<DataSourceEntry> {
    vec![
        DataSourceEntry {
            source_id: "OD-001".into(),
            provider: "Alpaca Market Data API".into(),
            capabilities: vec!["US equity realtime market data".into()],
            coverage: "US stocks and ETFs; Basic plan is IEX realtime and SIP delayed; full venue coverage depends on entitlement.".into(),
            latency: "Realtime or delayed according to the selected Alpaca plan; no default entitlement.".into(),
            entitlement: "Alpaca market-data API key and plan entitlement; broker account connection is not sufficient.".into(),
            retention: "Plan and local retention terms must be reviewed before caching or export.".into(),
            redistribution: "No redistribution or commercial-use grant is assumed.".into(),
            commercial_use: "UNVERIFIED — review the current Alpaca agreement for the intended use.".into(),
            jurisdictions: "US equities; market and customer jurisdiction restrictions apply.".into(),
            official_url: "https://docs.alpaca.markets/us/v1.1/docs/about-market-data-api".into(),
            terms_url: "https://alpaca.markets/legal".into(),
            reviewed_at: SOURCE_REVIEWED_AT.into(),
            checked_at: None,
            observed_at: None,
            probe_kind: DataSourceProbeKind::CredentialedMetadata,
            status: DataSourceStatus::BlockedExternal,
            configured: false,
            verified_at: None,
            availability_reason: "Market-data entitlement is not configured or verified in TradeX.".into(),
        },
        DataSourceEntry {
            source_id: "OD-002".into(),
            provider: "Alpaca Market Data API".into(),
            capabilities: vec!["US equity historical data".into()],
            coverage: "Historical bars, quotes and trades subject to plan history, adjustment and rate limits.".into(),
            latency: "Historical endpoint; latest window and history depth depend on entitlement.".into(),
            entitlement: "Alpaca market-data API key and historical-data plan entitlement.".into(),
            retention: "Persisted OHLCV must retain source, provider timestamp and plan/terms metadata.".into(),
            redistribution: "No redistribution or commercial-use grant is assumed.".into(),
            commercial_use: "UNVERIFIED — review the current Alpaca agreement for the intended use.".into(),
            jurisdictions: "US equities; source coverage is not a global exchange archive.".into(),
            official_url: "https://docs.alpaca.markets/us/v1.1/docs/about-market-data-api".into(),
            terms_url: "https://alpaca.markets/legal".into(),
            reviewed_at: SOURCE_REVIEWED_AT.into(),
            checked_at: None,
            observed_at: None,
            probe_kind: DataSourceProbeKind::CredentialedMetadata,
            status: DataSourceStatus::BlockedExternal,
            configured: false,
            verified_at: None,
            availability_reason: "Historical-data entitlement and usable range are not configured or verified.".into(),
        },
        DataSourceEntry {
            source_id: "OD-003".into(),
            provider: "SEC EDGAR data.sec.gov".into(),
            capabilities: vec!["Fundamentals and XBRL facts".into()],
            coverage: "SEC submissions and XBRL Company Facts/Frames for supported forms and filers.".into(),
            latency: "SEC publication and processing time; not a realtime market feed.".into(),
            entitlement: "Public API; every automated request needs an identifying User-Agent.".into(),
            retention: "Keep source URL, form/period and fetched timestamp with stored facts.".into(),
            redistribution: "Public access does not grant TradeX redistribution or commercial rights.".into(),
            commercial_use: "UNVERIFIED — follow SEC policy and review intended redistribution.".into(),
            jurisdictions: "US SEC filings; non-US issuers/forms may have different coverage.".into(),
            official_url: "https://www.sec.gov/search-filings/edgar-application-programming-interfaces".into(),
            terms_url: "https://www.sec.gov/about/developer-resources".into(),
            reviewed_at: SOURCE_REVIEWED_AT.into(),
            checked_at: None,
            observed_at: None,
            probe_kind: DataSourceProbeKind::PublicMetadata,
            status: DataSourceStatus::Unverified,
            configured: true,
            verified_at: None,
            availability_reason: "Public endpoint has not been probed in this workspace.".into(),
        },
        DataSourceEntry {
            source_id: "OD-004".into(),
            provider: "SEC EDGAR filings; general news provider unresolved".into(),
            capabilities: vec!["Filings".into(), "News (unresolved)".into()],
            coverage: "SEC filings are source-specific; no licensed general-news feed has been selected.".into(),
            latency: "Filing publication and processing timing follows SEC feeds; general news is unavailable.".into(),
            entitlement: "SEC public endpoint with User-Agent; licensed news entitlement still required.".into(),
            retention: "Filings retain accession/source metadata; no news content is cached.".into(),
            redistribution: "No general-news redistribution or commercial-use grant is assumed.".into(),
            commercial_use: "UNVERIFIED for filings; BLOCKED_EXTERNAL for news.".into(),
            jurisdictions: "SEC filing coverage only; news jurisdiction remains unselected.".into(),
            official_url: "https://www.sec.gov/search-filings/edgar-application-programming-interfaces".into(),
            terms_url: "https://www.sec.gov/about/developer-resources".into(),
            reviewed_at: SOURCE_REVIEWED_AT.into(),
            checked_at: None,
            observed_at: None,
            probe_kind: DataSourceProbeKind::PublicMetadata,
            status: DataSourceStatus::BlockedExternal,
            configured: false,
            verified_at: None,
            availability_reason: "Filings can be probed, but a licensed general-news provider is not selected; the combined OD gate remains blocked.".into(),
        },
        DataSourceEntry {
            source_id: "OD-005".into(),
            provider: "Alpaca Market Calendar and Corporate Actions".into(),
            capabilities: vec!["US equity sessions and corporate actions".into()],
            coverage: "Supported MIC calendars, early closes, splits, dividends and symbol/name actions; halts and full cross-market adjustment require S08.".into(),
            latency: "Provider calendar and action update timing; stale data blocks equity Live.".into(),
            entitlement: "Alpaca API key and applicable market-data/account entitlement.".into(),
            retention: "Persist event type, effective date, source and provider timestamp.".into(),
            redistribution: "No redistribution or commercial-use grant is assumed.".into(),
            commercial_use: "UNVERIFIED — review the current Alpaca agreement.".into(),
            jurisdictions: "Supported US market identifiers only until S08 adds cross-market sources.".into(),
            official_url: "https://docs.alpaca.markets/us/reference/calendar-2".into(),
            terms_url: "https://alpaca.markets/legal".into(),
            reviewed_at: SOURCE_REVIEWED_AT.into(),
            checked_at: None,
            observed_at: None,
            probe_kind: DataSourceProbeKind::CredentialedMetadata,
            status: DataSourceStatus::BlockedExternal,
            configured: false,
            verified_at: None,
            availability_reason: "Calendar and corporate-action credentials/coverage are not configured or freshly verified.".into(),
        },
        DataSourceEntry {
            source_id: "OD-006".into(),
            provider: "ECB Data Portal EXR / SDMX".into(),
            capabilities: vec!["Daily informational FX reference rates".into()],
            coverage: "Daily reference rates for 30 currencies quoted against EUR on working days.".into(),
            latency: "Published around 16:00 CET on working days; not transaction-grade intraday FX.".into(),
            entitlement: "Public SDMX REST endpoint; source disclosure travels with every value.".into(),
            retention: "Keep dataflow/series key, observation date, source URL and fetched timestamp.".into(),
            redistribution: "Use remains subject to ECB portal terms; information-only disclosure is mandatory.".into(),
            commercial_use: "UNVERIFIED — reference-rate information is not a transaction quote.".into(),
            jurisdictions: "EUR-denominated reference series; stablecoin parity is outside source coverage.".into(),
            official_url: "https://data.ecb.europa.eu/key-figures/ecb-interest-rates-and-exchange-rates/exchange-rates".into(),
            terms_url: "https://data.ecb.europa.eu/help/getting-data-web-services-sdmx-0".into(),
            reviewed_at: SOURCE_REVIEWED_AT.into(),
            checked_at: None,
            observed_at: None,
            probe_kind: DataSourceProbeKind::PublicMetadata,
            status: DataSourceStatus::Unverified,
            configured: true,
            verified_at: None,
            availability_reason: "Public EXR endpoint has not been probed in this workspace.".into(),
        },
    ]
}

pub fn probe(source_id: &str, mut source: DataSourceEntry) -> Result<DataSourceEntry> {
    if source.source_id != source_id {
        return Err(TradeXError::new("DATA_SOURCE_UNKNOWN"));
    }
    if source.probe_kind == DataSourceProbeKind::CredentialedMetadata {
        let checked_at = now()?;
        source.checked_at = Some(checked_at.clone());
        source.observed_at = Some(checked_at);
        source.status = DataSourceStatus::BlockedExternal;
        source.availability_reason = "A user-managed provider entitlement is required; TradeX did not read or infer credentials.".into();
        return Ok(source);
    }
    let observed_at = now()?;
    source.observed_at = Some(observed_at.clone());
    source.checked_at = Some(observed_at);
    let client = Client::builder()
        .https_only(true)
        .redirect(Policy::none())
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|_| TradeXError::new("DATA_SOURCE_PROBE_FAILED"))?;
    let endpoints: &[(&str, &str)] = match source_id {
        "OD-003" => &[
            ("SEC submissions", SEC_SUBMISSIONS_URL),
            ("SEC XBRL Frames", SEC_XBRL_FRAMES_URL),
        ],
        "OD-004" => &[("SEC filings", SEC_SUBMISSIONS_URL)],
        "OD-006" => &[("ECB EXR", ECB_EXR_URL)],
        _ => return Err(TradeXError::new("DATA_SOURCE_UNKNOWN")),
    };
    for (label, url) in endpoints {
        if let Err(reason) = fetch_public_endpoint(&client, source_id, label, url) {
            source.status = DataSourceStatus::Unavailable;
            source.availability_reason = reason;
            return Ok(source);
        }
    }
    if source_id == "OD-004" {
        source.status = DataSourceStatus::BlockedExternal;
        source.availability_reason =
            "SEC filings endpoint responded with the expected shape; a licensed general-news provider is still not selected."
                .into();
    } else {
        source.status = DataSourceStatus::Available;
        source.verified_at = source.observed_at.clone();
        source.availability_reason =
            "Public endpoints and response shapes verified; coverage, freshness and use restrictions still apply."
                .into();
    }
    Ok(source)
}

fn fetch_public_endpoint(
    client: &Client,
    source_id: &str,
    label: &str,
    url: &str,
) -> std::result::Result<(), String> {
    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::ACCEPT, "application/json, text/csv")
        .send()
        .map_err(|error| {
            format!(
                "{label} probe failed ({}). No response body was retained.",
                classify_probe_error(&error)
            )
        })?;
    if !response.status().is_success() {
        return Err(format!(
            "{label} endpoint returned HTTP {}; retry later. No response body was retained.",
            response.status().as_u16()
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(format!(
            "{label} response exceeded the bounded probe size; no response body was retained."
        ));
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let mut body = Vec::new();
    response
        .take((MAX_RESPONSE_BYTES as u64).saturating_add(1))
        .read_to_end(&mut body)
        .map_err(|_| {
            format!("{label} response could not be read. No response body was retained.")
        })?;
    if body.len() > MAX_RESPONSE_BYTES {
        return Err(format!(
            "{label} response exceeded the bounded probe size; no response body was retained."
        ));
    }
    if !validate_public_payload(source_id, label, &content_type, &body) {
        return Err(format!(
            "{label} response did not match the expected bounded protocol shape; no response body was retained."
        ));
    }
    Ok(())
}

fn validate_public_payload(
    source_id: &str,
    endpoint: &str,
    content_type: &str,
    body: &[u8],
) -> bool {
    match (source_id, endpoint) {
        ("OD-003", "SEC submissions") | ("OD-004", "SEC filings") => {
            content_type
                .to_ascii_lowercase()
                .contains("application/json")
                && serde_json::from_slice::<Value>(body).is_ok_and(|value| {
                    value.is_object()
                        && value.get("cik").is_some()
                        && value.get("filings").is_some()
                })
        }
        ("OD-003", "SEC XBRL Frames") => {
            content_type
                .to_ascii_lowercase()
                .contains("application/json")
                && serde_json::from_slice::<Value>(body).is_ok_and(|value| {
                    value.is_object()
                        && value.get("taxonomy").is_some()
                        && value.get("tag").is_some()
                        && value
                            .get("data")
                            .and_then(Value::as_array)
                            .is_some_and(|rows| !rows.is_empty())
                })
        }
        ("OD-006", "ECB EXR") => {
            if !content_type.to_ascii_lowercase().contains("text/csv") {
                return false;
            }
            let text = String::from_utf8_lossy(body);
            let mut lines = text.lines().filter(|line| !line.trim().is_empty());
            let Some(header) = lines.next() else {
                return false;
            };
            let fields: Vec<_> = header.split(',').map(str::trim).collect();
            fields.contains(&"TIME_PERIOD")
                && fields.contains(&"OBS_VALUE")
                && lines.next().is_some()
        }
        _ => false,
    }
}

fn classify_probe_error(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connection unavailable"
    } else {
        "request error"
    }
}

fn now() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| TradeXError::new("DATA_SOURCE_PROBE_FAILED"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_covers_each_open_decision_without_secrets() {
        let sources = entries();
        assert_eq!(sources.len(), 6);
        assert_eq!(
            sources
                .iter()
                .map(|source| source.source_id.as_str())
                .collect::<Vec<_>>(),
            vec!["OD-001", "OD-002", "OD-003", "OD-004", "OD-005", "OD-006"]
        );
        let encoded = serde_json::to_string(&sources).unwrap();
        assert!(!encoded.contains("secret"));
        assert!(!encoded.contains("api_key"));
        assert!(!encoded.contains("apikey"));
    }

    #[test]
    fn credentialed_probe_stays_blocked_without_reading_credentials() {
        let source = entries()
            .into_iter()
            .find(|item| item.source_id == "OD-001")
            .unwrap();
        let result = probe("OD-001", source).unwrap();
        assert_eq!(result.status, DataSourceStatus::BlockedExternal);
        assert!(result.observed_at.is_some());
        assert_ne!(result.checked_at.as_deref(), Some(SOURCE_REVIEWED_AT));
        assert!(!result.availability_reason.contains("key"));
    }

    #[test]
    fn public_probe_requires_all_sec_endpoint_shapes() {
        assert!(validate_public_payload(
            "OD-003",
            "SEC submissions",
            "application/json",
            br#"{"cik":"0000320193","filings":{}}"#
        ));
        assert!(!validate_public_payload(
            "OD-003",
            "SEC submissions",
            "application/json",
            br#"{"cik":"0000320193"}"#
        ));
        assert!(validate_public_payload(
            "OD-003",
            "SEC XBRL Frames",
            "application/json",
            br#"{"taxonomy":"us-gaap","tag":"Revenues","data":[{"val":1}]}"#
        ));
        assert!(!validate_public_payload(
            "OD-003",
            "SEC XBRL Frames",
            "application/json",
            br#"{"taxonomy":"us-gaap","tag":"Revenues"}"#
        ));
        assert!(validate_public_payload(
            "OD-006",
            "ECB EXR",
            "text/csv",
            b"KEY,FREQ,CURRENCY,TIME_PERIOD,OBS_VALUE\nD.USD.EUR.SP00.A,D,USD,2026-09-13,1.1\n"
        ));
        assert!(!validate_public_payload(
            "OD-006",
            "ECB EXR",
            "text/csv",
            b"KEY,FREQ,CURRENCY\nD.USD.EUR.SP00.A,D,USD\n"
        ));
    }
}
