//! Exact-symbol read-only commission declarations. Source validity is never fee qualification.
use crate::provider_io::{CredentialVault, ProviderEndpoint, ProviderHttp, ProviderHttpMethod};
use crate::{ControlPlane, protocol::*, risk::RiskCheckOutcome};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
    observation: Option<(SpotCommissionObservation, Instant)>,
    failure: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotCommissionStatus {
    NotObserved,
    Observed,
    Unavailable,
    Stale,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotCommissionQuality {
    ReadOnlySymbolCommission,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCommissionRates {
    #[schemars(length(min = 1, max = 64))]
    pub maker: String,
    #[schemars(length(min = 1, max = 64))]
    pub taker: String,
    #[schemars(length(min = 1, max = 64))]
    pub buyer: String,
    #[schemars(length(min = 1, max = 64))]
    pub seller: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCommissionDiscount {
    pub enabled_for_account: bool,
    pub enabled_for_symbol: bool,
    #[schemars(length(min = 1, max = 16))]
    pub discount_asset: String,
    #[schemars(length(min = 1, max = 64))]
    pub discount: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotCommissionReadKind {
    AccountIdentity,
    SymbolCommission,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCommissionReceipt {
    pub kind: SpotCommissionReadKind,
    #[schemars(length(min = 1, max = 64))]
    pub started_at: String,
    #[schemars(length(min = 1, max = 64))]
    pub received_at: String,
    #[schemars(length(min = 64, max = 64))]
    pub digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCommissionObservation {
    #[schemars(length(min = 1, max = 128))]
    pub collection_id: String,
    #[schemars(length(min = 1, max = 20))]
    pub remote_account_id: String,
    #[schemars(length(min = 1, max = 32))]
    pub symbol: String,
    pub quality: SpotCommissionQuality,
    pub standard_commission: SpotCommissionRates,
    pub tax_commission: SpotCommissionRates,
    pub special_commission: SpotCommissionRates,
    pub discount: SpotCommissionDiscount,
    pub terms_complete: bool,
    /// Received-asset branch, not a guaranteed fill-time fee currency.
    #[schemars(length(min = 1, max = 16))]
    pub received_asset: String,
    #[schemars(length(min = 1, max = 16))]
    pub conditional_discount_asset: Option<String>,
    pub charging_asset_qualified: bool,
    /// The database response supplies no provider observation timestamp.
    #[schemars(length(min = 1, max = 64))]
    pub provider_observed_at: Option<String>,
    #[schemars(length(min = 2, max = 2))]
    pub reads: Vec<SpotCommissionReceipt>,
    #[schemars(length(max = 32), inner(length(min = 1, max = 128)))]
    pub extension_keys: Vec<String>,
    #[schemars(length(max = 4), inner(length(min = 1, max = 128)))]
    pub unresolved_obligations: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpotCommissionInputs {
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
    pub side: OrderSide,
    #[schemars(length(min = 1, max = 128))]
    pub source_version: String,
    #[schemars(length(min = 64, max = 64))]
    pub rule_material_version: Option<String>,
    pub status: SpotCommissionStatus,
    pub qualification: RiskCheckOutcome,
    pub observation: Option<SpotCommissionObservation>,
    #[schemars(length(min = 1, max = 128))]
    pub failure: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub binding_version: String,
    #[schemars(length(min = 1, max = 128))]
    pub state_version: String,
}
fn invalid() -> TradeXError {
    TradeXError::new("PROVIDER_RESPONSE_INVALID")
}
fn digest(value: &impl Serialize) -> Result<String> {
    Ok(hex::encode(Sha256::digest(
        serde_json::to_vec(value).map_err(|_| invalid())?,
    )))
}

pub(crate) fn get(
    control: &mut ControlPlane,
    input: &crate::spot_proposal_rules::SpotRulesQuery,
) -> Result<SpotCommissionInputs> {
    let rules = crate::spot_proposal_rules::get(control, input)?;
    let store = control.store.as_ref().ok_or_else(invalid)?;
    let side = store.order_proposal(&input.proposal_id)?.fields.side;
    let max_age = store
        .risk_or_new()?
        .as_ref()
        .map_or(crate::risk::DEFAULT_STALE_QUOTE_THRESHOLD_SECONDS, |p| {
            p.policy.stale_quote_threshold_seconds
        })
        .min(30);
    let slot = control
        .spot_commission_runtime
        .slots
        .get(&input.proposal_id)
        .filter(|s| s.binding == rules.binding_version);
    let fresh = slot
        .and_then(|s| s.observation.as_ref())
        .filter(|(_, received)| received.elapsed() <= Duration::from_secs(max_age));
    let status = if fresh.is_some() {
        SpotCommissionStatus::Observed
    } else if slot.is_some_and(|s| s.failure.is_some()) {
        SpotCommissionStatus::Unavailable
    } else if slot.is_some_and(|s| s.observation.is_some()) {
        SpotCommissionStatus::Stale
    } else {
        SpotCommissionStatus::NotObserved
    };
    let mut result = SpotCommissionInputs {
        workspace_id: rules.workspace_id,
        proposal_id: rules.proposal_id,
        proposal_hash: rules.proposal_hash,
        account_id: rules.account_id,
        instrument_id: rules.instrument_id,
        base_asset: rules.base_asset,
        quote_asset: rules.quote_asset,
        side,
        source_version: rules.source_version,
        rule_material_version: rules.material_version,
        status,
        qualification: RiskCheckOutcome::Unavailable,
        observation: fresh.map(|(o, _)| o.clone()),
        failure: slot.and_then(|s| s.failure.clone()),
        binding_version: rules.binding_version,
        state_version: String::new(),
    };
    result.state_version = format!(
        "sha256:{}",
        digest(&json!([result, slot.map(|s| s.sequence)]))?
    );
    Ok(result)
}
fn original_decimal(value: &Value) -> Result<String> {
    let s = value
        .as_str()
        .filter(|s| !s.starts_with('-') && s.len() <= 64)
        .ok_or_else(invalid)?;
    crate::provider_io::decimal(value)?;
    Ok(s.into())
}
fn extensions(value: &Value, scope: &str, known: &[&str], keys: &mut Vec<String>) -> Result<()> {
    for key in value
        .as_object()
        .ok_or_else(invalid)?
        .keys()
        .filter(|key| !known.contains(&key.as_str()))
    {
        if !crate::valid_bounded_text(key, 64) || keys.len() >= 32 {
            return Err(invalid());
        }
        keys.push(format!("{scope}.{key}"));
    }
    Ok(())
}
fn rates(value: &Value, scope: &str, keys: &mut Vec<String>) -> Result<SpotCommissionRates> {
    extensions(value, scope, &["maker", "taker", "buyer", "seller"], keys)?;
    Ok(SpotCommissionRates {
        maker: original_decimal(&value["maker"])?,
        taker: original_decimal(&value["taker"])?,
        buyer: original_decimal(&value["buyer"])?,
        seller: original_decimal(&value["seller"])?,
    })
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
    request: &Value,
    consumer: &str,
    vault: &(impl CredentialVault + Clone + Send + 'static),
    http: &impl ProviderHttp,
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    let prepare = (|| {
        if !crate::provider_order_consumer_allowed(consumer) {
            return Err(TradeXError::new("IPC_ACCESS_DENIED"));
        }
        let envelope: CommandEnvelope = crate::payload(request.clone())?;
        if envelope.schema_version != 1 {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        if envelope.command != "trade.spot_commission.refresh"
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
        cp.spot_commission_runtime.sequence = cp
            .spot_commission_runtime
            .sequence
            .checked_add(1)
            .filter(|v| *v <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        let sequence = cp.spot_commission_runtime.sequence;
        if cp.spot_commission_runtime.slots.len() >= 8
            && !cp
                .spot_commission_runtime
                .slots
                .contains_key(&query.proposal_id)
        {
            if let Some(oldest) = cp
                .spot_commission_runtime
                .slots
                .iter()
                .min_by_key(|(_, s)| s.sequence)
                .map(|(id, _)| id.clone())
            {
                cp.spot_commission_runtime.slots.remove(&oldest);
            }
        }
        cp.spot_commission_runtime.slots.insert(
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
            cp.spot_commission_runtime
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
        let symbol = format!("{}{}", before.base_asset, before.quote_asset);
        let mut reads = Vec::new();
        let mut first_receipt = None;
        let mut collect =
            |route: &str, kind: SpotCommissionReadKind, params: &[(&str, &str)]| -> Result<Value> {
                if !remaining() {
                    return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
                }
                let started_at = wall(control, &query.workspace_id)?;
                crate::provider_io::binance_read_budget::reserve(Some(&remote), 20)?;
                let (response, limit) =
                    crate::provider_io::binance::signed_request_with_rate_limit_for(
                        ProviderEndpoint::BinanceLive,
                        &http,
                        ProviderHttpMethod::Get,
                        route,
                        params,
                        &secrets,
                        clock,
                        sampled,
                        &remaining,
                    )?;
                first_receipt.get_or_insert_with(Instant::now);
                match response.status {
                    200 => (),
                    418 => {
                        crate::provider_io::record_provider_retry_after(
                            "binance",
                            None,
                            limit.as_ref().and_then(|l| l.retry_after_seconds),
                        );
                        return Err(TradeXError::new("PROVIDER_IP_BANNED"));
                    }
                    429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
                    401 => return Err(TradeXError::new("PROVIDER_AUTHENTICATION_FAILED")),
                    403 | 451 => return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED")),
                    _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
                }
                if response.body.len() > 65_536 {
                    return Err(invalid());
                }
                let value: Value = crate::provider_json::strict_json(&response.body)?;
                reads.push(SpotCommissionReceipt {
                    kind,
                    started_at,
                    received_at: wall(control, &query.workspace_id)?,
                    digest: digest(&value)?,
                });
                Ok(value)
            };
        let account = collect(
            "/api/v3/account",
            SpotCommissionReadKind::AccountIdentity,
            &[],
        )?;
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
        let value = collect(
            "/api/v3/account/commission",
            SpotCommissionReadKind::SymbolCommission,
            &[("symbol", symbol.as_str())],
        )?;
        if value["symbol"] != symbol {
            return Err(TradeXError::new("PROVIDER_IDENTITY_CHANGED"));
        }
        let mut extension_keys = Vec::new();
        extensions(
            &value,
            "commission",
            &[
                "symbol",
                "standardCommission",
                "taxCommission",
                "specialCommission",
                "discount",
            ],
            &mut extension_keys,
        )?;
        let standard_commission = rates(
            &value["standardCommission"],
            "standardCommission",
            &mut extension_keys,
        )?;
        let tax_commission = rates(
            &value["taxCommission"],
            "taxCommission",
            &mut extension_keys,
        )?;
        let special_commission = rates(
            &value["specialCommission"],
            "specialCommission",
            &mut extension_keys,
        )?;
        let discount = &value["discount"];
        extensions(
            discount,
            "discount",
            &[
                "enabledForAccount",
                "enabledForSymbol",
                "discountAsset",
                "discount",
            ],
            &mut extension_keys,
        )?;
        let discount = SpotCommissionDiscount {
            enabled_for_account: discount["enabledForAccount"]
                .as_bool()
                .ok_or_else(invalid)?,
            enabled_for_symbol: discount["enabledForSymbol"].as_bool().ok_or_else(invalid)?,
            discount_asset: discount["discountAsset"]
                .as_str()
                .filter(|s| crate::valid_bounded_text(s, 16) && !s.chars().any(char::is_whitespace))
                .ok_or_else(invalid)?
                .into(),
            discount: original_decimal(&discount["discount"])?,
        };
        if crate::provider_io::decimal_cmp(&discount.discount, "1")? == std::cmp::Ordering::Greater
        {
            return Err(invalid());
        }
        extension_keys.sort();
        let mut unresolved_obligations = Vec::new();
        if !extension_keys.is_empty() {
            unresolved_obligations.push("UNKNOWN_ACTIVE_COMMISSION_FIELDS_UNRESOLVED".into());
        }
        if discount.discount_asset != "BNB" {
            unresolved_obligations.push("UNSUPPORTED_DISCOUNT_ASSET".into());
        }
        let terms_complete = unresolved_obligations.is_empty();
        unresolved_obligations.push("NON_ATOMIC_PROVIDER_SNAPSHOT".into());
        unresolved_obligations.push("FEE_AMOUNT_AND_CHARGING_ASSET_NOT_EXECUTION_QUALIFIED".into());
        let conditional_discount_asset = (discount.enabled_for_account
            && discount.enabled_for_symbol)
            .then(|| discount.discount_asset.clone());
        Ok((
            SpotCommissionObservation {
                collection_id: format!("commission:{sequence}"),
                remote_account_id: remote.clone(),
                symbol,
                quality: SpotCommissionQuality::ReadOnlySymbolCommission,
                standard_commission,
                tax_commission,
                special_commission,
                discount,
                terms_complete,
                received_asset: if before.side == OrderSide::Buy {
                    before.base_asset.clone()
                } else {
                    before.quote_asset.clone()
                },
                conditional_discount_asset,
                charging_asset_qualified: false,
                provider_observed_at: None,
                reads,
                extension_keys,
                unresolved_obligations,
            },
            first_receipt.ok_or_else(invalid)?,
        ))
    })();
    let finish = (|| {
        let mut cp = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if get(&mut cp, &query)?.binding_version != before.binding_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let slot = cp
            .spot_commission_runtime
            .slots
            .get_mut(&query.proposal_id)
            .filter(|s| s.sequence == sequence && s.binding == before.binding_version)
            .ok_or_else(|| TradeXError::new("STATE_VERSION_CONFLICT"))?;
        let read = if Instant::now() >= deadline {
            Err(TradeXError::new("PROVIDER_UNAVAILABLE"))
        } else {
            read
        };
        match read {
            Ok(o) => {
                slot.observation = Some(o);
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
