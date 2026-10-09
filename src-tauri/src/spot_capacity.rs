//! Exact-account read-only capacity observations are not execution qualification.
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
    observation: Option<(SpotCapacityObservation, Instant)>,
    failure: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotCapacityPurpose {
    OpenOrdersAccount,
    OpenOrdersSymbol,
    BaseBalances,
    OpenOrderLists,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotCapacityStatus {
    NotObserved,
    Observed,
    Unavailable,
    Stale,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotCapacityQuality {
    ReadOnlySpotCapacity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotCapacityReadKind {
    Account,
    OpenOrdersAccount,
    OpenOrdersSymbol,
    OpenOrderLists,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCapacityRead {
    pub kind: SpotCapacityReadKind,
    #[schemars(length(min = 1, max = 64))]
    pub started_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub received_at: String,
    #[schemars(length(min = 64, max = 64))]
    pub digest: String,
    #[schemars(length(min = 1, max = 20))]
    pub oldest_provider_update_time_ms: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub latest_provider_update_time_ms: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub oldest_provider_transaction_time_ms: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub latest_provider_transaction_time_ms: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCapacityBalance {
    #[schemars(length(min = 1, max = 16))]
    pub asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub free: String,
    #[schemars(length(min = 1, max = 64))]
    pub locked: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCapacityPosition {
    #[schemars(length(min = 1, max = 64))]
    pub selected_symbol_open_buy_original_quantity: String,
    #[schemars(length(min = 1, max = 64))]
    pub selected_symbol_open_buy_executed_quantity: String,
    pub asset_exposure_complete: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCapacityCounts {
    #[schemars(length(min = 1, max = 20))]
    pub account_open_orders: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub symbol_open_orders: String,
    #[schemars(length(min = 1, max = 20))]
    pub account_algo_orders: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub symbol_algo_orders: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub account_iceberg_orders: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub symbol_iceberg_orders: String,
    pub classifications_complete: bool,
    pub order_coverage_complete: bool,
    #[schemars(length(min = 1, max = 20))]
    pub account_open_order_lists: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub symbol_open_order_lists: Option<String>,
    #[schemars(length(min = 1, max = 20))]
    pub missing_list_legs: Option<String>,
    pub list_coverage_complete: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCapacityObservation {
    #[schemars(length(min = 1, max = 128))]
    pub collection_id: String,
    pub quality: SpotCapacityQuality,
    #[schemars(extend("const" = false))]
    pub atomic: bool,
    #[schemars(length(min = 1, max = 64))]
    pub provider_observed_at: Option<String>,
    pub counts: SpotCapacityCounts,
    pub base_balance: Option<SpotCapacityBalance>,
    pub position: Option<SpotCapacityPosition>,
    #[schemars(length(max = 16), inner(length(min = 1, max = 128)))]
    pub unresolved_obligations: Vec<String>,
    #[schemars(length(min = 2, max = 3))]
    pub reads: Vec<SpotCapacityRead>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCapacityInputs {
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
    #[schemars(length(max = 3))]
    pub purposes: Vec<SpotCapacityPurpose>,
    pub status: SpotCapacityStatus,
    pub qualification: RiskCheckOutcome,
    #[schemars(length(min = 1, max = 128))]
    pub qualification_reason: String,
    pub observation: Option<SpotCapacityObservation>,
    #[schemars(length(min = 1, max = 128))]
    pub failure: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub binding_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub state_version: String,
}

pub(crate) fn get(
    control: &mut ControlPlane,
    input: &crate::spot_proposal_rules::SpotRulesQuery,
) -> Result<SpotCapacityInputs> {
    let rules = crate::spot_proposal_rules::get(control, input)?;
    let proposal = control
        .store
        .as_ref()
        .ok_or_else(invalid)?
        .order_proposal(&input.proposal_id)?;
    let position_required = proposal.fields.side == OrderSide::Buy
        && rules.rules.iter().any(|r| r.rule_type == "MAX_POSITION");
    let list_required = rules.rules.iter().any(|r| {
        matches!(
            r.rule_type.as_str(),
            "MAX_NUM_ORDER_LISTS" | "EXCHANGE_MAX_NUM_ORDER_LISTS"
        )
    });
    let account_required = position_required
        || list_required
        || rules.rules.iter().any(|r| {
            matches!(
                r.rule_type.as_str(),
                "EXCHANGE_MAX_NUM_ORDERS"
                    | "EXCHANGE_MAX_NUM_ALGO_ORDERS"
                    | "EXCHANGE_MAX_NUM_ICEBERG_ORDERS"
            )
        });
    let symbol_required = rules.rules.iter().any(|r| {
        matches!(
            r.rule_type.as_str(),
            "MAX_NUM_ORDERS" | "MAX_NUM_ALGO_ORDERS" | "MAX_NUM_ICEBERG_ORDERS"
        )
    });
    let mut purposes = if account_required {
        vec![SpotCapacityPurpose::OpenOrdersAccount]
    } else if symbol_required {
        vec![SpotCapacityPurpose::OpenOrdersSymbol]
    } else {
        vec![]
    };
    if position_required {
        purposes.push(SpotCapacityPurpose::BaseBalances);
    }
    if list_required {
        purposes.push(SpotCapacityPurpose::OpenOrderLists);
    }
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
        .spot_capacity_runtime
        .slots
        .get(&input.proposal_id)
        .filter(|s| s.binding == rules.binding_version);
    let fresh = slot
        .and_then(|s| s.observation.as_ref())
        .filter(|(_, received)| received.elapsed() <= Duration::from_secs(max_age));
    let status = if fresh.is_some() {
        SpotCapacityStatus::Observed
    } else if slot.is_some_and(|s| s.failure.is_some()) {
        SpotCapacityStatus::Unavailable
    } else if slot.is_some_and(|s| s.observation.is_some()) {
        SpotCapacityStatus::Stale
    } else {
        SpotCapacityStatus::NotObserved
    };
    let sequence = slot.map(|s| s.sequence);
    let mut result = SpotCapacityInputs {
        workspace_id: rules.workspace_id,
        proposal_id: rules.proposal_id,
        proposal_hash: rules.proposal_hash,
        account_id: rules.account_id,
        instrument_id: rules.instrument_id,
        base_asset: rules.base_asset,
        quote_asset: rules.quote_asset,
        source_version: rules.source_version,
        rule_material_version: rules.material_version,
        purposes,
        status,
        qualification: RiskCheckOutcome::Unavailable,
        qualification_reason: "DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED".into(),
        observation: fresh.map(|(o, _)| o.clone()),
        failure: slot.and_then(|s| s.failure.clone()),
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
fn hash(value: &serde_json::Value) -> Result<String> {
    Ok(hex::encode(Sha256::digest(
        serde_json::to_vec(value).map_err(|_| invalid())?,
    )))
}
fn original_decimal(value: &serde_json::Value) -> Result<String> {
    let original = value
        .as_str()
        .filter(|v| v.len() <= 64 && !v.starts_with('-'))
        .ok_or_else(invalid)?;
    crate::provider_io::decimal(value)?;
    Ok(original.into())
}
fn original_update(value: &serde_json::Value) -> Result<u64> {
    value
        .as_u64()
        .filter(|v| *v <= i64::MAX as u64)
        .ok_or_else(invalid)
}
// Foreign inventory identifiers are opaque provider text, including documented
// non-ASCII assets/symbols. They never become HTTP parameters or inferred BASEs.
fn inventory_identity(value: &serde_json::Value) -> Result<&str> {
    value
        .as_str()
        .filter(|text| {
            crate::valid_bounded_text(text, 64) && !text.chars().any(char::is_whitespace)
        })
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
        if envelope.command != "trade.spot_capacity.refresh"
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
        if before.purposes.is_empty() {
            return Err(TradeXError::new("CAPACITY_INPUTS_NOT_REQUIRED"));
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
        cp.spot_capacity_runtime.sequence = cp
            .spot_capacity_runtime
            .sequence
            .checked_add(1)
            .filter(|v| *v <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        let sequence = cp.spot_capacity_runtime.sequence;
        if cp.spot_capacity_runtime.slots.len() >= 8
            && !cp
                .spot_capacity_runtime
                .slots
                .contains_key(&query.proposal_id)
        {
            if let Some(oldest) = cp
                .spot_capacity_runtime
                .slots
                .iter()
                .min_by_key(|(_, s)| s.sequence)
                .map(|(id, _)| id.clone())
            {
                cp.spot_capacity_runtime.slots.remove(&oldest);
            }
        }
        cp.spot_capacity_runtime.slots.insert(
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
            cp.spot_capacity_runtime
                .slots
                .get(&query.proposal_id)
                .is_some_and(|s| s.sequence == sequence && s.binding == before.binding_version)
                && get(&mut cp, &query).is_ok_and(|r| r.binding_version == before.binding_version)
        })
    };
    let current = || Instant::now() < deadline && owned();
    let read = (|| {
        let credentials = crate::financial_sources::read_credentials_before_deadline(
            vault, &reference, deadline,
        )?;
        let secrets = zeroize::Zeroizing::new(credentials.values()?);
        if secrets.len() != 2 {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        if !crate::financial_sources::http_budget_remains(deadline) {
            return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
        }
        let remaining = || current() && crate::financial_sources::http_budget_remains(deadline);
        let http = crate::provider_io::p3_provider_http(http, &remaining, &remote);
        crate::provider_io::binance_read_budget::reserve(None, 2)?;
        let (clock, sampled) = crate::provider_io::binance::server_time_for(
            ProviderEndpoint::BinanceLive,
            &http,
            &remaining,
        )?;
        let local = ::time::OffsetDateTime::parse(
            &wall(control, &query.workspace_id)?,
            &::time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| invalid())?
        .unix_timestamp_nanos()
            / 1_000_000;
        if (i128::from(clock) - local).abs() > i128::from(crate::time::MAX_PROVIDER_OFFSET_MS) {
            return Err(TradeXError::new("CLOCK_SKEW"));
        }
        let symbol = format!("{}USDT", before.base_asset);
        let account_scope = before
            .purposes
            .contains(&SpotCapacityPurpose::OpenOrdersAccount);
        let mut reads = Vec::new();
        let mut first_receipt = None;
        let mut collect = |route: &str,
                           kind: SpotCapacityReadKind,
                           weight: u32|
         -> Result<serde_json::Value> {
            if !remaining() {
                return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
            }
            let started = wall(control, &query.workspace_id)?;
            crate::provider_io::binance_read_budget::reserve(Some(&remote), weight)?;
            let symbol_params = [("symbol", symbol.as_str())];
            let (response, rate_limit) =
                crate::provider_io::binance::signed_request_with_rate_limit_for(
                    ProviderEndpoint::BinanceLive,
                    &http,
                    ProviderHttpMethod::Get,
                    route,
                    if kind == SpotCapacityReadKind::OpenOrdersSymbol {
                        &symbol_params
                    } else {
                        &[]
                    },
                    &secrets,
                    clock,
                    sampled,
                    &remaining,
                )?;
            // Freshness starts with the oldest private response, before parsing
            // or collecting a later non-atomic endpoint.
            first_receipt.get_or_insert_with(Instant::now);
            match response.status {
                200 => (),
                418 => {
                    crate::provider_io::record_provider_retry_after(
                        "binance",
                        None,
                        rate_limit
                            .as_ref()
                            .and_then(|limit| limit.retry_after_seconds),
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
            let value: serde_json::Value = crate::provider_json::strict_json(&response.body)?;
            let updates = if let Some(rows) = value.as_array() {
                rows.iter()
                    .filter_map(|r| r.get("updateTime"))
                    .map(original_update)
                    .collect::<Result<Vec<_>>>()?
            } else {
                value
                    .get("updateTime")
                    .map(original_update)
                    .transpose()?
                    .into_iter()
                    .collect()
            };
            let transactions = if kind == SpotCapacityReadKind::OpenOrderLists {
                value
                    .as_array()
                    .ok_or_else(invalid)?
                    .iter()
                    .map(|row| original_update(&row["transactionTime"]))
                    .collect::<Result<Vec<_>>>()?
            } else {
                vec![]
            };
            let latest = clock
                .checked_add(sampled.elapsed().as_millis() as u64)
                .and_then(|v| v.checked_add(crate::time::MAX_PROVIDER_OFFSET_MS as u64))
                .ok_or_else(invalid)?;
            if updates
                .iter()
                .chain(transactions.iter())
                .any(|timestamp| *timestamp > latest)
            {
                return Err(invalid());
            }
            reads.push(SpotCapacityRead {
                kind,
                started_at: started,
                received_at: wall(control, &query.workspace_id)?,
                digest: hash(&value)?,
                oldest_provider_update_time_ms: updates.iter().min().map(u64::to_string),
                latest_provider_update_time_ms: updates.iter().max().map(u64::to_string),
                oldest_provider_transaction_time_ms: transactions.iter().min().map(u64::to_string),
                latest_provider_transaction_time_ms: transactions.iter().max().map(u64::to_string),
            });
            Ok(value)
        };
        let account = collect("/api/v3/account", SpotCapacityReadKind::Account, 20)?;
        if account["uid"]
            .as_u64()
            .filter(|uid| *uid > 0 && *uid <= i64::MAX as u64)
            .map(|uid| uid.to_string())
            .as_deref()
            != Some(remote.as_str())
            || account["accountType"] != "SPOT"
        {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
        }
        let position_required = before.purposes.contains(&SpotCapacityPurpose::BaseBalances);
        if account["canTrade"].as_bool().is_none() {
            return Err(invalid());
        }
        let mut account_extensions = account.as_object().ok_or_else(invalid)?.keys().any(|key| {
            !matches!(
                key.as_str(),
                "uid"
                    | "accountType"
                    | "canTrade"
                    | "canWithdraw"
                    | "canDeposit"
                    | "brokered"
                    | "requireSelfTradePrevention"
                    | "preventSor"
                    | "updateTime"
                    | "balances"
                    | "permissions"
                    | "makerCommission"
                    | "takerCommission"
                    | "buyerCommission"
                    | "sellerCommission"
                    | "commissionRates"
            )
        });
        for field in [
            "canWithdraw",
            "canDeposit",
            "brokered",
            "requireSelfTradePrevention",
            "preventSor",
        ] {
            if account
                .get(field)
                .is_some_and(|value| value.as_bool().is_none())
            {
                return Err(invalid());
            }
        }
        if let Some(permissions) = account.get("permissions") {
            let permissions = permissions
                .as_array()
                .filter(|values| values.len() <= 64)
                .ok_or_else(invalid)?;
            let mut seen = std::collections::HashSet::new();
            for value in permissions {
                let permission = value
                    .as_str()
                    .filter(|text| crate::valid_bounded_text(text, 64))
                    .ok_or_else(invalid)?;
                if !seen.insert(permission) {
                    return Err(invalid());
                }
            }
        }
        for field in [
            "makerCommission",
            "takerCommission",
            "buyerCommission",
            "sellerCommission",
        ] {
            if let Some(value) = account.get(field) {
                original_update(value)?;
            }
        }
        if let Some(rates) = account.get("commissionRates") {
            let rates = rates.as_object().ok_or_else(invalid)?;
            for field in ["maker", "taker", "buyer", "seller"] {
                original_decimal(rates.get(field).ok_or_else(invalid)?)?;
            }
            account_extensions |= rates
                .keys()
                .any(|key| !matches!(key.as_str(), "maker" | "taker" | "buyer" | "seller"));
        }
        let mut balance_extensions = false;
        let mut selected_balance = None;
        if let Some(balances) = account.get("balances") {
            let rows = balances
                .as_array()
                .filter(|rows| rows.len() <= 1024)
                .ok_or_else(invalid)?;
            let mut assets = std::collections::HashSet::new();
            for row in rows {
                balance_extensions |= row
                    .as_object()
                    .ok_or_else(invalid)?
                    .keys()
                    .any(|key| !matches!(key.as_str(), "asset" | "free" | "locked"));
                let asset = inventory_identity(&row["asset"])?;
                if !assets.insert(asset) {
                    return Err(invalid());
                }
                let free = original_decimal(&row["free"])?;
                let locked = original_decimal(&row["locked"])?;
                if position_required && asset == before.base_asset {
                    selected_balance = Some(SpotCapacityBalance {
                        asset: asset.into(),
                        free,
                        locked,
                    });
                }
            }
        }
        let base_balance = if position_required {
            Some(selected_balance.ok_or_else(|| TradeXError::new("PROVIDER_DATA_INCOMPLETE"))?)
        } else {
            None
        };
        let orders = collect(
            "/api/v3/openOrders",
            if account_scope {
                SpotCapacityReadKind::OpenOrdersAccount
            } else {
                SpotCapacityReadKind::OpenOrdersSymbol
            },
            if account_scope { 80 } else { 6 },
        )?;
        let rows = orders
            .as_array()
            .filter(|rows| rows.len() <= 1000)
            .ok_or_else(invalid)?;
        let mut ids = std::collections::HashSet::new();
        let mut original_quantity = "0".to_string();
        let mut executed_quantity = "0".to_string();
        let mut asset_exposure_complete = true;
        let mut unresolved = vec![
            "NON_ATOMIC_PROVIDER_SNAPSHOT".into(),
            "DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED".into(),
        ];
        if account_extensions {
            unresolved.push("UNKNOWN_ACTIVE_ACCOUNT_FIELDS_UNRESOLVED".into());
        }
        if balance_extensions {
            unresolved.push("UNKNOWN_ACTIVE_BALANCE_FIELDS_UNRESOLVED".into());
        }
        let mut account_algo = 0u64;
        let mut symbol_algo = 0u64;
        let mut account_iceberg = 0u64;
        let mut symbol_iceberg = 0u64;
        let mut classifications_complete = true;
        let mut unknown_fields = false;
        let mut partial_quantity = false;
        let mut missing_order_times = false;
        let mut unresolved_order_flags = false;
        for row in rows {
            let id = row["orderId"]
                .as_u64()
                .filter(|id| *id > 0 && *id <= i64::MAX as u64)
                .ok_or_else(invalid)?;
            let row_symbol = inventory_identity(&row["symbol"])?;
            if (!account_scope && row_symbol != symbol) || !ids.insert((row_symbol, id)) {
                return Err(invalid());
            }
            if !matches!(row["side"].as_str(), Some("BUY" | "SELL"))
                || row["isWorking"].as_bool().is_none()
                || !matches!(
                    row["status"].as_str(),
                    Some("NEW" | "PARTIALLY_FILLED" | "PENDING_NEW")
                )
            {
                return Err(invalid());
            }
            let original = original_decimal(&row["origQty"])?;
            for field in [
                "price",
                "stopPrice",
                "origQuoteOrderQty",
                "cummulativeQuoteQty",
            ] {
                original_decimal(&row[field])?;
            }
            row["orderListId"]
                .as_i64()
                .filter(|id| *id >= -1)
                .ok_or_else(invalid)?;
            row["clientOrderId"]
                .as_str()
                .filter(|text| crate::valid_bounded_text(text, 36))
                .ok_or_else(invalid)?;
            let time_in_force = row["timeInForce"]
                .as_str()
                .filter(|text| crate::valid_bounded_text(text, 16))
                .ok_or_else(invalid)?;
            unresolved_order_flags |= !matches!(time_in_force, "GTC" | "IOC" | "FOK")
                || row["isWorking"] == false
                || row["status"] == "PENDING_NEW";
            if let Some(value) = row.get("selfTradePreventionMode") {
                let mode = value
                    .as_str()
                    .filter(|text| crate::valid_bounded_text(text, 32))
                    .ok_or_else(invalid)?;
                unresolved_order_flags |= !matches!(
                    mode,
                    "NONE"
                        | "EXPIRE_TAKER"
                        | "EXPIRE_MAKER"
                        | "EXPIRE_BOTH"
                        | "DECREMENT"
                        | "TRANSFER"
                );
            }
            for field in ["time", "updateTime"] {
                if let Some(value) = row.get(field) {
                    let timestamp = original_update(value)?;
                    let latest = clock
                        .checked_add(sampled.elapsed().as_millis() as u64)
                        .and_then(|v| v.checked_add(crate::time::MAX_PROVIDER_OFFSET_MS as u64))
                        .ok_or_else(invalid)?;
                    if timestamp == 0 || timestamp > latest {
                        return Err(invalid());
                    }
                } else {
                    missing_order_times = true;
                }
            }
            if row["time"]
                .as_u64()
                .zip(row["updateTime"].as_u64())
                .is_some_and(|(created, updated)| created > updated)
            {
                return Err(invalid());
            }
            if let Some(value) = row.get("workingTime") {
                let timestamp = value
                    .as_i64()
                    .filter(|time| *time >= -1)
                    .ok_or_else(invalid)?;
                let latest = clock
                    .checked_add(sampled.elapsed().as_millis() as u64)
                    .and_then(|v| v.checked_add(crate::time::MAX_PROVIDER_OFFSET_MS as u64))
                    .ok_or_else(invalid)?;
                if timestamp >= 0 && timestamp as u64 > latest {
                    return Err(invalid());
                }
            }
            let executed = original_decimal(&row["executedQty"])?;
            if crate::provider_io::decimal_cmp(&executed, &original)? == std::cmp::Ordering::Greater
            {
                return Err(invalid());
            }
            partial_quantity |= row["status"] == "PARTIALLY_FILLED"
                || crate::provider_io::decimal_cmp(&executed, "0")? == std::cmp::Ordering::Greater;
            unknown_fields |= row.as_object().ok_or_else(invalid)?.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "symbol"
                        | "orderId"
                        | "orderListId"
                        | "clientOrderId"
                        | "price"
                        | "origQty"
                        | "executedQty"
                        | "cummulativeQuoteQty"
                        | "status"
                        | "timeInForce"
                        | "type"
                        | "side"
                        | "stopPrice"
                        | "icebergQty"
                        | "time"
                        | "updateTime"
                        | "isWorking"
                        | "workingTime"
                        | "origQuoteOrderQty"
                        | "selfTradePreventionMode"
                )
            });
            let algo = match row["type"]
                .as_str()
                .filter(|text| crate::valid_bounded_text(text, 32))
                .ok_or_else(invalid)?
            {
                "STOP_LOSS" | "STOP_LOSS_LIMIT" | "TAKE_PROFIT" | "TAKE_PROFIT_LIMIT" => true,
                "LIMIT" | "MARKET" | "LIMIT_MAKER" => false,
                _ => {
                    classifications_complete = false;
                    false
                }
            };
            let iceberg =
                crate::provider_io::decimal_cmp(&original_decimal(&row["icebergQty"])?, "0")?
                    == std::cmp::Ordering::Greater;
            account_algo += u64::from(algo);
            symbol_algo += u64::from(algo && row_symbol == symbol);
            account_iceberg += u64::from(iceberg);
            symbol_iceberg += u64::from(iceberg && row_symbol == symbol);
            if position_required {
                if row_symbol != symbol {
                    asset_exposure_complete = false;
                }
                if row_symbol == symbol && row["side"] == "BUY" {
                    original_quantity =
                        crate::portfolio::decimal_add(&original_quantity, &original)?;
                    executed_quantity =
                        crate::portfolio::decimal_add(&executed_quantity, &executed)?;
                    if original_quantity.len() > 64 || executed_quantity.len() > 64 {
                        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
                    }
                }
            }
        }
        let list_required = before
            .purposes
            .contains(&SpotCapacityPurpose::OpenOrderLists)
            || rows
                .iter()
                .any(|row| row["orderListId"].as_i64().is_some_and(|id| id >= 0));
        let mut account_lists = None;
        let mut symbol_lists = None;
        let mut missing_legs = None;
        let mut list_complete = None;
        if list_required {
            let lists = collect(
                "/api/v3/openOrderList",
                SpotCapacityReadKind::OpenOrderLists,
                6,
            )?;
            let lists = lists
                .as_array()
                .filter(|rows| rows.len() <= 256)
                .ok_or_else(invalid)?;
            let mut list_ids = std::collections::HashSet::new();
            let mut leg_ids = std::collections::HashSet::new();
            let mut missing = 0u64;
            let mut complete = true;
            let mut unknown_list_fields = false;
            let mut unknown_leg_fields = false;
            let mut identity_contradiction = false;
            let mut unknown_contingency = false;
            let mut unresolved_leg_shape = false;
            for list in lists {
                unknown_list_fields |= list.as_object().ok_or_else(invalid)?.keys().any(|key| {
                    !matches!(
                        key.as_str(),
                        "orderListId"
                            | "contingencyType"
                            | "listStatusType"
                            | "listOrderStatus"
                            | "listClientOrderId"
                            | "transactionTime"
                            | "symbol"
                            | "orders"
                    )
                });
                let id = original_update(&list["orderListId"])?;
                if !list_ids.insert(id) {
                    return Err(invalid());
                }
                let list_symbol = inventory_identity(&list["symbol"])?;
                if list["listOrderStatus"] != "EXECUTING"
                    || !matches!(
                        list["listStatusType"].as_str(),
                        Some("EXEC_STARTED" | "UPDATED")
                    )
                {
                    return Err(invalid());
                }
                list["listClientOrderId"]
                    .as_str()
                    .filter(|text| crate::valid_bounded_text(text, 36))
                    .ok_or_else(invalid)?;
                let contingency = list["contingencyType"]
                    .as_str()
                    .filter(|text| crate::valid_bounded_text(text, 32))
                    .ok_or_else(invalid)?;
                if !matches!(contingency, "OCO" | "OTO") {
                    complete = false;
                    unknown_contingency = true;
                }
                if original_update(&list["transactionTime"])? == 0 {
                    return Err(invalid());
                }
                let legs = list["orders"]
                    .as_array()
                    .filter(|rows| !rows.is_empty() && rows.len() <= 3)
                    .ok_or_else(invalid)?;
                if matches!(contingency, "OCO" | "OTO") && legs.len() != 2 {
                    complete = false;
                    unresolved_leg_shape = true;
                }
                for leg in legs {
                    unknown_leg_fields |=
                        leg.as_object().ok_or_else(invalid)?.keys().any(|key| {
                            !matches!(key.as_str(), "symbol" | "orderId" | "clientOrderId")
                        });
                    let client_id = leg["clientOrderId"]
                        .as_str()
                        .filter(|text| crate::valid_bounded_text(text, 36))
                        .ok_or_else(invalid)?;
                    let leg_id = original_update(&leg["orderId"])?;
                    if leg_id == 0
                        || leg["symbol"] != list_symbol
                        || !leg_ids.insert((list_symbol, leg_id))
                    {
                        return Err(invalid());
                    }
                    if let Some(order) = rows.iter().find(|row| {
                        row["symbol"] == list_symbol && row["orderId"].as_u64() == Some(leg_id)
                    }) {
                        if order["orderListId"].as_u64() != Some(id)
                            || order["clientOrderId"].as_str() != Some(client_id)
                        {
                            complete = false;
                            identity_contradiction = true;
                        }
                    } else {
                        missing += 1;
                        complete = false;
                    }
                }
            }
            for order in rows {
                let id = order["orderListId"]
                    .as_i64()
                    .filter(|id| *id >= -1)
                    .ok_or_else(invalid)?;
                if id >= 0
                    && (!list_ids.contains(&(id as u64))
                        || !leg_ids.contains(&(
                            order["symbol"].as_str().ok_or_else(invalid)?,
                            order["orderId"].as_u64().ok_or_else(invalid)?,
                        )))
                {
                    complete = false;
                }
            }
            if unknown_list_fields {
                complete = false;
                unresolved.push("UNKNOWN_ACTIVE_LIST_FIELDS_UNRESOLVED".into());
            }
            if unknown_leg_fields {
                complete = false;
                unresolved.push("UNKNOWN_ACTIVE_LIST_LEG_FIELDS_UNRESOLVED".into());
            }
            if identity_contradiction {
                unresolved.push("LIST_LEG_IDENTITY_CONTRADICTION_UNRESOLVED".into());
            }
            if unknown_contingency {
                unresolved.push("LIST_CONTINGENCY_SEMANTICS_UNRESOLVED".into());
            }
            if unresolved_leg_shape {
                unresolved.push("LIST_LEG_SHAPE_UNRESOLVED".into());
            }
            account_lists = Some(lists.len().to_string());
            symbol_lists = Some(
                lists
                    .iter()
                    .filter(|list| list["symbol"] == symbol)
                    .count()
                    .to_string(),
            );
            missing_legs = Some(missing.to_string());
            list_complete = Some(complete);
            if !complete {
                unresolved.push("PENDING_OR_MISSING_LIST_LEGS_UNRESOLVED".into());
            }
        }
        if partial_quantity {
            unresolved.push("PARTIAL_FILL_QUANTITY_SEMANTICS_UNRESOLVED".into());
        }
        if missing_order_times {
            unresolved.push("ORIGINAL_ORDER_TIMES_INCOMPLETE".into());
        }
        if unresolved_order_flags {
            unresolved.push("ACTIVE_ORDER_FLAGS_UNRESOLVED".into());
        }
        if unknown_fields {
            unresolved.push("UNKNOWN_ACTIVE_ORDER_FIELDS_UNRESOLVED".into());
        }
        if !classifications_complete {
            unresolved.push("ORDER_TYPE_CLASSIFICATION_UNAVAILABLE".into());
        }
        if position_required && !asset_exposure_complete {
            unresolved.push("FOREIGN_SYMBOL_ASSET_ATTRIBUTION_UNAVAILABLE".into());
        }
        Ok((
            SpotCapacityObservation {
                collection_id: uuid::Uuid::new_v4().to_string(),
                quality: SpotCapacityQuality::ReadOnlySpotCapacity,
                atomic: false,
                provider_observed_at: None,
                counts: SpotCapacityCounts {
                    account_open_order_lists: account_lists,
                    symbol_open_order_lists: symbol_lists,
                    missing_list_legs: missing_legs,
                    list_coverage_complete: list_complete,
                    account_algo_orders: (account_scope && classifications_complete)
                        .then(|| account_algo.to_string()),
                    symbol_algo_orders: classifications_complete.then(|| symbol_algo.to_string()),
                    account_iceberg_orders: account_scope.then(|| account_iceberg.to_string()),
                    symbol_iceberg_orders: symbol_iceberg.to_string(),
                    classifications_complete,
                    order_coverage_complete: classifications_complete
                        && !unknown_fields
                        && !unresolved_order_flags,
                    account_open_orders: account_scope.then(|| rows.len().to_string()),
                    symbol_open_orders: rows
                        .iter()
                        .filter(|r| r["symbol"] == symbol)
                        .count()
                        .to_string(),
                },
                base_balance,
                position: position_required.then_some(SpotCapacityPosition {
                    selected_symbol_open_buy_original_quantity: original_quantity,
                    selected_symbol_open_buy_executed_quantity: executed_quantity,
                    asset_exposure_complete,
                }),
                unresolved_obligations: unresolved,
                reads,
            },
            first_receipt.ok_or_else(invalid)?,
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
            .spot_capacity_runtime
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
