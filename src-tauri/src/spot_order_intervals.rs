//! Account-wide interval usage inputs never grant execution authority.
use crate::provider_io::{CredentialVault, ProviderEndpoint, ProviderHttp, ProviderHttpMethod};
use crate::{ControlPlane, protocol::*, risk::RiskCheckOutcome};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Default)]
pub(crate) struct Runtime {
    sequence: u64,
    slots: HashMap<String, Slot>,
}
struct Slot {
    sequence: u64,
    binding: String,
    observation: Option<(SpotOrderIntervalObservation, Instant, ClockAssociation)>,
    failure: Option<String>,
}

// This is a conservative local association, never a provider counter timestamp.
struct ClockAssociation {
    server_ms: u64,
    sampled: Instant,
    round_trip_bound_ms: u64,
    order_durations_ms: Vec<u64>,
}
impl ClockAssociation {
    fn remains_in_original_window(&self) -> bool {
        let lower = self.server_ms.saturating_sub(self.round_trip_bound_ms);
        let Some(upper) = u64::try_from(self.sampled.elapsed().as_millis())
            .ok()
            .and_then(|elapsed| self.server_ms.checked_add(elapsed))
            .and_then(|v| v.checked_add(self.round_trip_bound_ms))
        else {
            return false;
        };
        self.order_durations_ms
            .iter()
            .all(|duration| lower / duration == upper / duration)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotOrderIntervalTimeAssociation {
    DerivedUncertain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotOrderIntervalDefinition {
    #[schemars(length(min = 1, max = 64))]
    pub rate_limit_type: String,
    #[schemars(length(min = 1, max = 64))]
    pub interval: String,
    #[schemars(length(min = 1, max = 20))]
    pub interval_num: String,
    #[schemars(length(min = 1, max = 20))]
    pub limit: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotOrderIntervalCounter {
    #[serde(flatten)]
    pub definition: SpotOrderIntervalDefinition,
    #[schemars(length(min = 1, max = 20))]
    pub count: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotOrderIntervalReadKind {
    IntervalDefinitions,
    AccountIdentity,
    AccountIntervalCounters,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotOrderIntervalRead {
    pub kind: SpotOrderIntervalReadKind,
    #[schemars(length(min = 1, max = 64))]
    pub started_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub received_at: String,
    #[schemars(length(min = 64, max = 64))]
    pub digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotOrderIntervalClock {
    #[schemars(length(min = 1, max = 20))]
    pub server_time_ms: String,
    #[schemars(length(min = 1, max = 20))]
    pub local_round_trip_bound_ms: String,
    #[schemars(length(min = 1, max = 64))]
    pub started_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub received_at: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotOrderIntervalObservation {
    #[schemars(length(min = 1, max = 128))]
    pub collection_id: String,
    #[schemars(extend("const" = false))]
    pub atomic: bool,
    #[schemars(length(min = 1, max = 64))]
    pub provider_observed_at: Option<String>,
    #[schemars(length(max = 32))]
    pub declarations: Vec<SpotOrderIntervalDefinition>,
    #[schemars(length(max = 32))]
    pub counters: Vec<SpotOrderIntervalCounter>,
    pub coverage_complete: bool,
    #[schemars(length(max = 16), inner(length(min = 1, max = 128)))]
    pub unresolved_obligations: Vec<String>,
    pub clock: SpotOrderIntervalClock,
    pub time_association: SpotOrderIntervalTimeAssociation,
    #[schemars(length(min = 1, max = 3))]
    pub reads: Vec<SpotOrderIntervalRead>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotOrderIntervalScope {
    AccountAllKeysIpsApis,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotOrderIntervalInputs {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub proposal_id: String,
    #[schemars(length(min = 71, max = 71))]
    pub proposal_hash: String,
    #[schemars(length(min = 1, max = 128))]
    pub account_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub instrument_id: String,
    #[schemars(length(min = 1, max = 16))]
    pub base_asset: String,
    #[schemars(length(min = 1, max = 16))]
    pub quote_asset: String,
    #[schemars(length(min = 1, max = 128))]
    pub source_version: String,
    #[schemars(length(min = 64, max = 64))]
    pub rule_material_version: Option<String>,
    pub scope: SpotOrderIntervalScope,
    pub status: crate::spot_capacity::SpotCapacityStatus,
    pub qualification: RiskCheckOutcome,
    #[schemars(length(min = 1, max = 128))]
    pub qualification_reason: String,
    pub observation: Option<SpotOrderIntervalObservation>,
    #[schemars(length(min = 1, max = 128))]
    pub failure: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub retirement_reason: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub provider_wait_seconds: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub binding_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub state_version: String,
}

pub(crate) fn get(
    control: &mut ControlPlane,
    input: &crate::spot_proposal_rules::SpotRulesQuery,
) -> Result<SpotOrderIntervalInputs> {
    let rules = crate::spot_proposal_rules::get(control, input)?;
    let max_age = control
        .store
        .as_ref()
        .ok_or_else(invalid)?
        .risk_or_new()?
        .as_ref()
        .map_or(crate::risk::DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS, |p| {
            p.policy.stale_quote_threshold_seconds
        })
        .min(30);
    let slot = control
        .spot_order_interval_runtime
        .slots
        .get(&input.proposal_id)
        .filter(|s| s.binding == rules.binding_version);
    let retained = slot.and_then(|s| s.observation.as_ref());
    let retirement_reason = retained.and_then(|(_, received, association)| {
        if received.elapsed() > Duration::from_secs(max_age) {
            Some("FIRST_RECEIPT_EXPIRED".to_string())
        } else if !association.remains_in_original_window() {
            Some("POSSIBLE_INTERVAL_BOUNDARY".to_string())
        } else {
            None
        }
    });
    let fresh = retained.filter(|_| retirement_reason.is_none());
    let status = if fresh.is_some() {
        crate::spot_capacity::SpotCapacityStatus::Observed
    } else if slot.is_some_and(|s| s.failure.is_some()) {
        crate::spot_capacity::SpotCapacityStatus::Unavailable
    } else if slot.is_some_and(|s| s.observation.is_some()) {
        crate::spot_capacity::SpotCapacityStatus::Stale
    } else {
        crate::spot_capacity::SpotCapacityStatus::NotObserved
    };
    let sequence = slot.map(|s| s.sequence);
    let provider_wait_seconds = control
        .store
        .as_ref()
        .ok_or_else(invalid)?
        .account(&rules.account_id)?
        .data
        .as_ref()
        .and_then(|d| {
            crate::provider_io::provider_retry_after_seconds("binance", &d.remote_account_id)
        })
        .map(|seconds| seconds.to_string());
    let mut result = SpotOrderIntervalInputs {
        workspace_id: rules.workspace_id,
        proposal_id: rules.proposal_id,
        proposal_hash: rules.proposal_hash,
        account_id: rules.account_id,
        instrument_id: rules.instrument_id,
        base_asset: rules.base_asset,
        quote_asset: rules.quote_asset,
        source_version: rules.source_version,
        rule_material_version: rules.material_version,
        scope: SpotOrderIntervalScope::AccountAllKeysIpsApis,
        status,
        qualification: RiskCheckOutcome::Unavailable,
        qualification_reason: "INTERVAL_INPUTS_NOT_EXECUTION_QUALIFIED".into(),
        observation: fresh.map(|(o, _, _)| o.clone()),
        failure: slot.and_then(|s| s.failure.clone()),
        retirement_reason,
        provider_wait_seconds,
        binding_version: rules.binding_version,
        state_version: String::new(),
    };
    result.state_version = format!(
        "sha256:{}",
        hex::encode(Sha256::digest(
            serde_json::to_vec(&json!([result, sequence]))
                .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?
        ))
    );
    Ok(result)
}

fn invalid() -> TradeXError {
    TradeXError::new("PROVIDER_RESPONSE_INVALID")
}
fn original_integer(value: &serde_json::Value, positive: bool) -> Result<String> {
    value
        .as_i64()
        .filter(|n| if positive { *n > 0 } else { *n >= 0 })
        .map(|n| n.to_string())
        .ok_or_else(invalid)
}
fn bounded_rows(value: &serde_json::Value) -> Result<&Vec<serde_json::Value>> {
    value
        .as_array()
        .filter(|rows| rows.len() <= 32)
        .ok_or_else(invalid)
}
fn has_unknown_fields(rows: &[serde_json::Value], permitted: &[&str]) -> bool {
    rows.iter().any(|row| {
        row.as_object()
            .is_some_and(|fields| fields.keys().any(|key| !permitted.contains(&key.as_str())))
    })
}
fn unique_definitions<'a>(
    rows: impl Iterator<Item = &'a SpotOrderIntervalDefinition>,
) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for row in rows {
        if !seen.insert((&row.rate_limit_type, &row.interval, &row.interval_num)) {
            return Err(invalid());
        }
    }
    Ok(())
}
fn definition(row: &serde_json::Value) -> Result<SpotOrderIntervalDefinition> {
    let text = |v: &serde_json::Value| {
        v.as_str()
            .filter(|s| crate::valid_bounded_text(s, 64))
            .map(String::from)
            .ok_or_else(invalid)
    };
    let result = SpotOrderIntervalDefinition {
        rate_limit_type: text(&row["rateLimitType"])?,
        interval: text(&row["interval"])?,
        interval_num: original_integer(&row["intervalNum"], true)?,
        limit: original_integer(&row["limit"], false)?,
    };
    interval_duration_ms(&result)?;
    Ok(result)
}
fn interval_duration_ms(row: &SpotOrderIntervalDefinition) -> Result<Option<u64>> {
    let unit = match row.interval.as_str() {
        "SECOND" => 1000,
        "MINUTE" => 60_000,
        "HOUR" => 3_600_000,
        "DAY" => 86_400_000,
        _ => return Ok(None),
    };
    row.interval_num
        .parse::<u64>()
        .map_err(|_| invalid())?
        .checked_mul(unit)
        .filter(|d| *d <= i64::MAX as u64)
        .map(Some)
        .ok_or_else(invalid)
}
fn wall(control: &Arc<Mutex<ControlPlane>>, workspace: &str) -> Result<String> {
    Ok(control
        .lock()
        .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
        .time
        .status(workspace)?
        .wall_clock)
}
fn parse_response(
    response: crate::provider_io::ProviderHttpResponse,
    rate_limit: Option<crate::provider_io::ProviderRateLimit>,
) -> Result<serde_json::Value> {
    match response.status {
        200 => (),
        418 => {
            crate::provider_io::record_provider_retry_after(
                "binance",
                None,
                rate_limit.and_then(|l| l.retry_after_seconds),
            );
            return Err(TradeXError::new("PROVIDER_IP_BANNED"));
        }
        429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
        401 => return Err(TradeXError::new("PROVIDER_AUTHENTICATION_FAILED")),
        403 | 451 => return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED")),
        _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
    }
    if response.body.len() > 512 * 1024 {
        return Err(invalid());
    }
    crate::provider_json::strict_json(&response.body)
}
pub(crate) fn execute_refresh(
    control: &Arc<Mutex<ControlPlane>>,
    request: &serde_json::Value,
    consumer: &str,
    vault: &(impl CredentialVault + Clone + Send + 'static),
    http: &impl ProviderHttp,
) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    let prepare = (|| {
        if !crate::provider_order_consumer_allowed(consumer) {
            return Err(TradeXError::new("IPC_ACCESS_DENIED"));
        }
        let envelope: CommandEnvelope = crate::payload(request.clone())?;
        if envelope.schema_version != 1 {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        if envelope.command != "trade.spot_order_intervals.refresh"
            || !crate::valid_bounded_text(&envelope.request_id, 128)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let input: crate::spot_proposal_rules::SpotRulesRefresh = crate::payload(envelope.payload)?;
        let query = crate::spot_proposal_rules::SpotRulesQuery {
            workspace_id: input.workspace_id,
            proposal_id: input.proposal_id,
        };
        let mut cp = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        let before = get(&mut cp, &query)?;
        if before.state_version != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if before.rule_material_version.is_none() {
            return Err(TradeXError::new("ORDER_FILTERS_UNAVAILABLE"));
        }
        let account = cp
            .store
            .as_ref()
            .ok_or_else(invalid)?
            .account(&before.account_id)?;
        let remote = account
            .data
            .as_ref()
            .ok_or_else(invalid)?
            .remote_account_id
            .clone();
        cp.spot_order_interval_runtime.sequence = cp
            .spot_order_interval_runtime
            .sequence
            .checked_add(1)
            .filter(|v| *v <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        let sequence = cp.spot_order_interval_runtime.sequence;
        if cp.spot_order_interval_runtime.slots.len() >= 8
            && !cp
                .spot_order_interval_runtime
                .slots
                .contains_key(&query.proposal_id)
        {
            if let Some(oldest) = cp
                .spot_order_interval_runtime
                .slots
                .iter()
                .min_by_key(|(_, s)| s.sequence)
                .map(|(id, _)| id.clone())
            {
                cp.spot_order_interval_runtime.slots.remove(&oldest);
            }
        }
        cp.spot_order_interval_runtime.slots.insert(
            query.proposal_id.clone(),
            Slot {
                sequence,
                binding: before.binding_version.clone(),
                observation: None,
                failure: None,
            },
        );
        Ok((query, before, sequence, account.credential_ref(), remote))
    })();
    let (query, before, sequence, reference, remote) = match prepare {
        Ok(v) => v,
        Err(e) => return crate::failure_reply(crate::request_id(request), e),
    };
    let owned = || {
        control.lock().is_ok_and(|mut cp| {
            cp.spot_order_interval_runtime
                .slots
                .get(&query.proposal_id)
                .is_some_and(|s| s.sequence == sequence && s.binding == before.binding_version)
                && get(&mut cp, &query).is_ok_and(|r| r.binding_version == before.binding_version)
        })
    };
    let current = || Instant::now() < deadline && owned();
    let read = (|| {
        let remaining = || current() && crate::financial_sources::http_budget_remains(deadline);
        let http = crate::provider_io::p3_provider_http(http, &remaining, &remote);
        let clock_started = wall(control, &query.workspace_id)?;
        let clock_request_started = Instant::now();
        crate::provider_io::binance_read_budget::reserve(None, 1)?;
        let (clock, sampled) = crate::provider_io::binance::server_time_for(
            ProviderEndpoint::BinanceLive,
            &http,
            &remaining,
        )?;
        let round_trip_bound_ms = u64::try_from(clock_request_started.elapsed().as_millis())
            .map_err(|_| invalid())?
            .checked_add(1)
            .ok_or_else(invalid)?;
        let clock_received = wall(control, &query.workspace_id)?;
        let local = ::time::OffsetDateTime::parse(
            &clock_received,
            &::time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| invalid())?
        .unix_timestamp_nanos()
            / 1_000_000;
        if (i128::from(clock) - local).abs() > i128::from(crate::time::MAX_PROVIDER_OFFSET_MS) {
            return Err(TradeXError::new("CLOCK_SKEW"));
        }
        let started = wall(control, &query.workspace_id)?;
        crate::provider_io::binance_read_budget::reserve(None, 20)?;
        let (response, limit) = http.request_with_rate_limit(
            ProviderEndpoint::BinanceLive,
            ProviderHttpMethod::Get,
            &format!(
                "/api/v3/exchangeInfo?symbol={}USDT&showPermissionSets=true",
                before.base_asset
            ),
            reqwest::header::HeaderMap::new(),
            None,
        )?;
        let first_receipt = Instant::now();
        let received = wall(control, &query.workspace_id)?;
        let definitions = parse_response(response, limit)?;
        let symbols = definitions["symbols"]
            .as_array()
            .filter(|rows| rows.len() == 1)
            .ok_or_else(invalid)?;
        if symbols[0]["symbol"] != format!("{}USDT", before.base_asset)
            || symbols[0]["baseAsset"] != before.base_asset
            || symbols[0]["quoteAsset"] != "USDT"
        {
            return Err(invalid());
        }

        let declarations = bounded_rows(&definitions["rateLimits"])?
            .iter()
            .map(definition)
            .collect::<Result<Vec<_>>>()?;
        unique_definitions(declarations.iter())?;
        if !declarations.iter().any(|d| d.rate_limit_type == "ORDERS") {
            return Err(TradeXError::new("INTERVAL_DEFINITIONS_UNAVAILABLE"));
        }
        let mut reads = vec![SpotOrderIntervalRead {
            kind: SpotOrderIntervalReadKind::IntervalDefinitions,
            started_at: started,
            received_at: received,
            digest: hex::encode(Sha256::digest(
                serde_json::to_vec(&definitions).map_err(|_| invalid())?,
            )),
        }];
        let credentials = crate::financial_sources::read_credentials_before_deadline(
            vault, &reference, deadline,
        )?;
        let secrets = zeroize::Zeroizing::new(credentials.values()?);
        if secrets.len() != 2 {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        if crate::provider_io::contains_secret(&definitions, &secrets) {
            return Err(invalid());
        }
        let mut collect = |route: &str,
                           kind: SpotOrderIntervalReadKind,
                           weight: u32|
         -> Result<serde_json::Value> {
            if !remaining() {
                return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
            }
            let started = wall(control, &query.workspace_id)?;
            crate::provider_io::binance_read_budget::reserve(Some(&remote), weight)?;
            let (response, limit) =
                crate::provider_io::binance::signed_request_with_rate_limit_for(
                    ProviderEndpoint::BinanceLive,
                    &http,
                    ProviderHttpMethod::Get,
                    route,
                    &[],
                    &secrets,
                    clock,
                    sampled,
                    &remaining,
                )?;
            let received = wall(control, &query.workspace_id)?;
            let value = parse_response(response, limit)?;
            reads.push(SpotOrderIntervalRead {
                kind,
                started_at: started,
                received_at: received,
                digest: hex::encode(Sha256::digest(
                    serde_json::to_vec(&value).map_err(|_| invalid())?,
                )),
            });
            Ok(value)
        };
        let account = collect(
            "/api/v3/account",
            SpotOrderIntervalReadKind::AccountIdentity,
            20,
        )?;
        if account["uid"]
            .as_i64()
            .filter(|n| *n > 0)
            .map(|n| n.to_string())
            .as_deref()
            != Some(remote.as_str())
            || account["accountType"] != "SPOT"
        {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
        }
        let value = collect(
            "/api/v3/rateLimit/order",
            SpotOrderIntervalReadKind::AccountIntervalCounters,
            40,
        )?;
        let counters = bounded_rows(&value)?
            .iter()
            .map(|row| {
                Ok(SpotOrderIntervalCounter {
                    definition: definition(row)?,
                    count: original_integer(&row["count"], false)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        unique_definitions(counters.iter().map(|c| &c.definition))?;
        let mut unresolved = Vec::new();
        let definition_extensions = has_unknown_fields(
            bounded_rows(&definitions["rateLimits"])?,
            &["rateLimitType", "interval", "intervalNum", "limit"],
        );
        let counter_extensions = has_unknown_fields(
            bounded_rows(&value)?,
            &["rateLimitType", "interval", "intervalNum", "limit", "count"],
        );
        if definition_extensions || counter_extensions {
            unresolved.push("UNKNOWN_ACTIVE_INTERVAL_FIELDS_UNRESOLVED".into());
        }
        if declarations
            .iter()
            .chain(counters.iter().map(|c| &c.definition))
            .any(|d| !matches!(d.interval.as_str(), "SECOND" | "MINUTE" | "HOUR" | "DAY"))
        {
            unresolved.push("UNKNOWN_INTERVAL_UNIT_UNRESOLVED".into());
        }
        if declarations.iter().any(|d| {
            !matches!(
                d.rate_limit_type.as_str(),
                "ORDERS" | "REQUEST_WEIGHT" | "RAW_REQUESTS"
            )
        }) {
            unresolved.push("UNKNOWN_DECLARED_TYPE_UNRESOLVED".into());
        }
        if counters
            .iter()
            .any(|c| c.definition.rate_limit_type != "ORDERS")
        {
            unresolved.push("UNKNOWN_COUNTER_TYPE_UNRESOLVED".into());
        }
        let same_tuple = |a: &SpotOrderIntervalDefinition, b: &SpotOrderIntervalDefinition| {
            a.rate_limit_type == b.rate_limit_type
                && a.interval == b.interval
                && a.interval_num == b.interval_num
        };
        let expected = declarations
            .iter()
            .filter(|d| d.rate_limit_type == "ORDERS")
            .collect::<Vec<_>>();
        if expected
            .iter()
            .any(|d| !counters.iter().any(|c| same_tuple(d, &c.definition)))
        {
            unresolved.push("DECLARED_INTERVALS_MISSING".into());
        }
        if counters
            .iter()
            .any(|c| !expected.iter().any(|d| same_tuple(d, &c.definition)))
        {
            unresolved.push("UNDECLARED_COUNTER_INTERVALS".into());
        }
        if expected.iter().any(|d| {
            counters
                .iter()
                .any(|c| same_tuple(d, &c.definition) && d.limit != c.definition.limit)
        }) {
            unresolved.push("INTERVAL_LIMIT_CONTRADICTION".into());
        }
        let coverage_complete = unresolved.is_empty();
        unresolved.push("COUNTER_SNAPSHOT_TIME_UNAVAILABLE".into());
        unresolved.push("DERIVED_WINDOW_ASSOCIATION_UNCERTAIN".into());
        unresolved.push("INTERVAL_INPUTS_NOT_EXECUTION_QUALIFIED".into());
        let order_durations_ms = declarations
            .iter()
            .chain(counters.iter().map(|c| &c.definition))
            .filter(|d| d.rate_limit_type == "ORDERS")
            .map(interval_duration_ms)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok((
            SpotOrderIntervalObservation {
                collection_id: uuid::Uuid::new_v4().to_string(),
                atomic: false,
                provider_observed_at: None,
                declarations,
                counters,
                coverage_complete,
                unresolved_obligations: unresolved,
                clock: SpotOrderIntervalClock {
                    server_time_ms: clock.to_string(),
                    local_round_trip_bound_ms: round_trip_bound_ms.to_string(),
                    started_at: clock_started,
                    received_at: clock_received,
                },
                time_association: SpotOrderIntervalTimeAssociation::DerivedUncertain,
                reads,
            },
            first_receipt,
            ClockAssociation {
                server_ms: clock,
                sampled,
                round_trip_bound_ms,
                order_durations_ms,
            },
        ))
    })();
    let finish = (|| {
        if !owned() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let mut cp = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if get(&mut cp, &query)?.binding_version != before.binding_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let slot = cp
            .spot_order_interval_runtime
            .slots
            .get_mut(&query.proposal_id)
            .filter(|s| s.sequence == sequence)
            .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?;
        let read = if Instant::now() >= deadline {
            Err(TradeXError::new("PROVIDER_UNAVAILABLE"))
        } else {
            read
        };
        match read {
            Ok(observation) => {
                slot.observation = Some(observation);
                slot.failure = None;
            }
            Err(e) => {
                slot.observation = None;
                slot.failure = Some(e.code);
            }
        }
        get(&mut cp, &query)
    })();
    match finish {
        Ok(r) => crate::success_reply(crate::request_id(request), json!(r), None),
        Err(e) => crate::failure_reply(crate::request_id(request), e),
    }
}
