//! Explicit saved-account selections for read-only company events and broker metadata.
use crate::protocol::{
    BrokerInstrumentEvidence, BrokerInstrumentMetadata, BrokerScheduleEvent, CompanyEventCategory,
    CompanyEventDate, CompanyEventSecurity, CompanyEventTerm, CorporateActionEvidence,
    FinancialEvidenceBinding, FinancialEvidenceCapability as Capability,
    FinancialEvidenceCapabilityResult, FinancialEvidenceQuality, FinancialSourceEvidence,
    KnownCompanyEvent, TimeConfidence, TimeStatus,
};
use crate::provider_io::{CredentialVault, ProviderEndpoint, ProviderHttp};
use crate::{
    ControlPlane, payload,
    protocol::{
        CommandEnvelope, DataSourceAccountChoice, DataSourceMutation, DataSourceQuery,
        DataSourceStatus, FinancialSourceConfigure, FinancialSourceConnection, FinancialSourceKind,
        MAX_SEQUENCE, Result, TradeXError,
    },
    provider_order_consumer_allowed,
    providers::{AccountConnection, ConnectionState},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration as StdDuration, Instant},
};
use time::{
    Date, Duration, OffsetDateTime,
    format_description::{self, well_known::Rfc3339},
};

#[derive(Clone)]
struct Binding {
    workspace: String,
    kind: FinancialSourceKind,
    source_version: String,
    account: String,
    account_version: String,
    reference: String,
    session: String,
    runtime_epoch: String,
    clock_generation: String,
    sequence: u64,
}
impl Binding {
    fn current(&self, control: &ControlPlane) -> bool {
        self.session == control.session
            && self.runtime_epoch == control.financial_source_runtime.epoch
            && self.clock_generation == control.time.generation()
            && control.financial_source_runtime.sequences.get(&self.kind) == Some(&self.sequence)
            && control.require_workspace(&self.workspace).is_ok()
            && control.store.as_ref().is_some_and(|store| {
                store.financial_source(self.kind).is_ok_and(|source| {
                    source.version(self.kind, &self.workspace) == self.source_version
                }) && store.account(&self.account).is_ok_and(|account| {
                    self.kind.eligible(&account)
                        && account.state_version == self.account_version
                        && account.credential_ref() == self.reference
                })
            })
    }
    fn projection(&self) -> Result<FinancialEvidenceBinding> {
        Ok(FinancialEvidenceBinding {
            connection_id: self.account.clone(),
            account_version: self.account_version.clone(),
            source_version: self.source_version.clone(),
            binding_version: hash(&json!([
                self.workspace,
                self.kind,
                self.source_version,
                self.account,
                self.account_version,
                self.reference,
                self.session,
                self.runtime_epoch,
                self.clock_generation,
                self.sequence
            ]))?,
        })
    }
}
#[derive(Clone)]
struct Observation {
    binding: Binding,
    received: TimeStatus,
    evidence: FinancialSourceEvidence,
}
impl Observation {
    fn fresh(&self, control: &ControlPlane, now: &TimeStatus) -> bool {
        let wall_age = timestamp(&now.wall_clock)
            .ok()
            .zip(timestamp(&self.received.wall_clock).ok())
            .map(|(now, received)| now - received);
        self.binding.current(control)
            && now.confidence == TimeConfidence::Trusted
            && self.received.confidence == TimeConfidence::Trusted
            && now
                .monotonic_ms
                .checked_sub(self.received.monotonic_ms)
                .is_some_and(|age| age <= 30_000)
            && wall_age.is_some_and(|age| (Duration::ZERO..=Duration::seconds(30)).contains(&age))
    }
}
pub(crate) struct FinancialSourceRuntime {
    epoch: String,
    sequences: HashMap<FinancialSourceKind, u64>,
    observations: HashMap<FinancialSourceKind, Observation>,
    failures: HashMap<FinancialSourceKind, String>,
}
impl Default for FinancialSourceRuntime {
    fn default() -> Self {
        Self {
            epoch: uuid::Uuid::new_v4().to_string(),
            sequences: HashMap::new(),
            observations: HashMap::new(),
            failures: HashMap::new(),
        }
    }
}
fn invalid() -> TradeXError {
    TradeXError::new("PROVIDER_RESPONSE_INVALID")
}
fn hash(value: &impl Serialize) -> Result<String> {
    use sha2::{Digest, Sha256};
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(|_| invalid())?)
    ))
}
fn date(value: &str) -> Result<Date> {
    if value.len() != 10 {
        return Err(invalid());
    }
    Date::parse(
        value,
        &format_description::parse_borrowed::<2>("[year]-[month]-[day]").map_err(|_| invalid())?,
    )
    .map_err(|_| invalid())
}
fn timestamp(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| invalid())
}
fn capability(
    capability: Capability,
    status: DataSourceStatus,
    reason: &str,
) -> FinancialEvidenceCapabilityResult {
    FinancialEvidenceCapabilityResult {
        capability,
        status,
        reason: reason.into(),
    }
}

impl FinancialSourceKind {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::CorporateActions => "CORPORATE_ACTIONS",
            Self::BrokerInstruments => "BROKER_INSTRUMENTS",
        }
    }
    fn environment(self) -> &'static str {
        match self {
            Self::CorporateActions => "PAPER",
            Self::BrokerInstruments => "LIVE",
        }
    }
    fn provider(self) -> &'static str {
        match self {
            Self::CorporateActions => "alpaca",
            Self::BrokerInstruments => "trading212",
        }
    }
    fn eligible(self, account: &AccountConnection) -> bool {
        account.provider_id == self.provider()
            && account.environment == self.environment()
            && account.connection_state == ConnectionState::Connected
            && matches!(
                account.health.credential.as_str(),
                "CONFIGURED" | "UNCHECKED"
            )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SavedFinancialSource {
    pub generation: u64,
    pub connection_id: Option<String>,
}
impl Default for SavedFinancialSource {
    fn default() -> Self {
        Self {
            generation: 0,
            connection_id: None,
        }
    }
}
impl SavedFinancialSource {
    pub fn version(&self, kind: FinancialSourceKind, workspace: &str) -> String {
        format!("{}:{workspace}:{}", kind.key(), self.generation)
    }
    pub fn validate(&self) -> Result<()> {
        if self.generation > MAX_SEQUENCE
            || self
                .connection_id
                .as_ref()
                .is_some_and(|id| !crate::valid_bounded_text(id, 128))
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(())
    }
}

pub(crate) fn connection(
    control: &mut ControlPlane,
    workspace: &str,
    kind: FinancialSourceKind,
) -> Result<FinancialSourceConnection> {
    control.require_workspace(workspace)?;
    let store = control.store.as_ref().unwrap();
    let saved = store.financial_source(kind)?;
    let usable = saved.connection_id.as_ref().is_none_or(|id| {
        store
            .account(id)
            .is_ok_and(|account| kind.eligible(&account))
    });
    let now = control.time.status(workspace)?;
    let runtime = &control.financial_source_runtime;
    let observation = runtime.observations.get(&kind);
    let fresh = observation.is_some_and(|observation| observation.fresh(control, &now));
    let status = if saved.connection_id.is_none() {
        DataSourceStatus::BlockedExternal
    } else if usable && fresh {
        DataSourceStatus::Available
    } else if observation.is_some() || runtime.failures.contains_key(&kind) {
        DataSourceStatus::Unavailable
    } else if usable {
        DataSourceStatus::Unverified
    } else {
        DataSourceStatus::Unavailable
    };
    let reason = if saved.connection_id.is_none() {
        "Select an eligible saved account for this read-only source."
    } else if usable && fresh {
        match kind {
            FinancialSourceKind::CorporateActions => {
                "Authenticated known events are current as a bounded process-date query only. Complete prospective action coverage and historical adjustment remain unsupported."
            }
            FinancialSourceKind::BrokerInstruments => {
                "The selected account identity and instrument metadata have been read. Ten-minute metadata does not establish current account tradability, exchange halts or canonical security identity."
            }
        }
    } else if let Some(code) = runtime.failures.get(&kind) {
        match code.as_str() {
            "PROVIDER_AUTH_FAILED" => {
                "Authentication failed for the selected read-only source. No current observation was published."
            }
            "PROVIDER_PERMISSION_BLOCKED" => {
                "The provider denied access to the requested read-only endpoint. This response does not establish the account's broader permissions."
            }
            "PROVIDER_RATE_LIMITED" => {
                "Provider read quota is unavailable. Wait for the permitted endpoint window before refreshing the selected account."
            }
            "PROVIDER_BACKPRESSURE" => {
                "Source work could not be scheduled. Retry after current provider reads finish."
            }
            "CREDENTIAL_UNAVAILABLE" => {
                "The selected saved key is unavailable. Resolve its account connection before refreshing this source."
            }
            "CLOCK_SKEW" => {
                "System time is not trusted for this source observation. Revalidate time before refreshing."
            }
            "PROVIDER_RESPONSE_INVALID" => {
                "The provider response could not be validated. The prior current observation was retired."
            }
            _ => {
                "The provider read failed or exceeded its deadline. No current observation was published."
            }
        }
    } else if observation.is_some() {
        "This observation expired or no longer matches the account/time binding. Check account and time, then refresh the selected source."
    } else if usable {
        "Saved account selected. Authenticated source access has not been checked; financial evidence remains unverified."
    } else {
        "The selected saved account is unavailable. Reconnect it or explicitly select an eligible account."
    };
    Ok(FinancialSourceConnection {
        workspace_id: workspace.into(),
        kind,
        state_version: saved.version(kind, workspace),
        configured: saved.connection_id.is_some(),
        connection_id: saved.connection_id,
        environment: kind.environment().into(),
        status: status.clone(),
        availability_reason: reason.into(),
        eligible_accounts: store
            .accounts()?
            .into_iter()
            .filter(|account| kind.eligible(account))
            .map(|account| DataSourceAccountChoice {
                connection_id: account.connection_id,
                display_name: account.label,
            })
            .collect(),
        observed_at: observation.map(|observation| observation.received.wall_clock.clone()),
        evidence: observation.map(|observation| observation.evidence.clone()),
        capability_statuses: if kind == FinancialSourceKind::BrokerInstruments {
            vec![
                capability(Capability::BrokerAccountIdentity, status.clone(), reason),
                capability(Capability::BrokerInstrumentMetadata, status.clone(), reason),
                capability(
                    Capability::ExchangeHalts,
                    DataSourceStatus::BlockedExternal,
                    "Static exchange schedules do not supply current exchange halt evidence.",
                ),
                capability(
                    Capability::AccountTradability,
                    DataSourceStatus::BlockedExternal,
                    "Instrument directory membership and quantity caps do not establish current unrestricted account tradability.",
                ),
            ]
        } else {
            vec![
                capability(Capability::KnownActions, status.clone(), reason),
                capability(
                    Capability::ActionQueryCompletion,
                    status.clone(),
                    "Pagination exhaustion applies only to the returned processing-date query, not prospective event completeness.",
                ),
                capability(
                    Capability::CompleteActionCoverage,
                    DataSourceStatus::BlockedExternal,
                    "The provider documents processing delays and no immediate-announcement completeness guarantee. An empty query cannot certify no pending action or complete delisting coverage.",
                ),
                capability(
                    Capability::HistoricalAdjustment,
                    DataSourceStatus::Unverified,
                    "Known events do not prove historical prices were adjusted; actual history provenance is still required.",
                ),
            ]
        },
    })
}

pub(crate) fn selected_once(control: &ControlPlane) -> Result<bool> {
    let store = control
        .store
        .as_ref()
        .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?;
    Ok(store
        .financial_source(FinancialSourceKind::CorporateActions)?
        .generation
        > 0
        || store
            .financial_source(FinancialSourceKind::BrokerInstruments)?
            .generation
            > 0)
}

pub(crate) fn project(
    control: &mut ControlPlane,
    detail: &mut crate::protocol::MarketDetail,
) -> Result<()> {
    if !detail.instrument.instrument_id.starts_with("equity:") || !selected_once(control)? {
        return Ok(());
    }
    detail.financial_evidence = Some(crate::protocol::MarketFinancialEvidence {
        company_events: connection(
            control,
            &detail.workspace_id,
            FinancialSourceKind::CorporateActions,
        )?,
        broker_instruments: connection(
            control,
            &detail.workspace_id,
            FinancialSourceKind::BrokerInstruments,
        )?,
    });
    // Date-only company events and static instrument metadata cannot supply legacy action,
    // historical adjustment or current tradability authority.
    detail.corporate_actions.clear();
    detail.adjustment_status = crate::protocol::AdjustmentStatus::Unavailable;
    detail.instrument_state = None;
    Ok(())
}

pub(crate) fn catalog_entry(
    control: &ControlPlane,
    mut entry: crate::protocol::DataSourceEntry,
) -> crate::protocol::DataSourceEntry {
    match selected_once(control) {
        Ok(false) => (),
        Ok(true) => {
            entry.configured = control.store.as_ref().is_some_and(|store| {
                [
                    FinancialSourceKind::CorporateActions,
                    FinancialSourceKind::BrokerInstruments,
                ]
                .iter()
                .any(|kind| {
                    store
                        .financial_source(*kind)
                        .is_ok_and(|source| source.connection_id.is_some())
                })
            }) || entry.configured;
            entry.status = DataSourceStatus::Unverified;
            entry.checked_at = None;
            entry.verified_at = None;
            entry.availability_reason = "Inspect individual financial capabilities. A bounded company-event query or ten-minute instrument metadata does not certify complete action coverage, current halts, account tradability or history adjustment.".into();
        }
        Err(_) => {
            entry.status = DataSourceStatus::Unavailable;
            entry.availability_reason = "Financial source configuration could not be read. Reopen the workspace before using evidence.".into();
        }
    }
    entry
}

pub(crate) fn metadata(
    control: &mut ControlPlane,
    request: CommandEnvelope,
    consumer: &str,
) -> Result<(Value, Option<String>)> {
    if !provider_order_consumer_allowed(consumer) {
        return Err(TradeXError::new("IPC_ACCESS_DENIED"));
    }
    let kind = if request.command.starts_with("data.actions.") {
        FinancialSourceKind::CorporateActions
    } else {
        FinancialSourceKind::BrokerInstruments
    };
    if request.command.ends_with(".connection") {
        let input: DataSourceQuery = payload(request.payload)?;
        let source = connection(control, &input.workspace_id, kind)?;
        return Ok((json!(source), Some(source.state_version)));
    }
    let (workspace, expected, selected) = if request.command.ends_with(".configure") {
        let input: FinancialSourceConfigure = payload(request.payload)?;
        if !crate::valid_bounded_text(&input.connection_id, 128) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        (
            input.workspace_id,
            input.expected_state_version,
            Some(input.connection_id),
        )
    } else {
        let input: DataSourceMutation = payload(request.payload)?;
        (input.workspace_id, input.expected_state_version, None)
    };
    control.require_workspace(&workspace)?;
    let store = control.store.as_mut().unwrap();
    let mut saved = store.financial_source(kind)?;
    if saved.version(kind, &workspace) != expected {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    if let Some(id) = selected.as_ref() {
        if !kind.eligible(&store.account(id)?) {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
    }
    saved.connection_id = selected;
    store.save_financial_source(kind, saved)?;
    control.financial_source_runtime.observations.remove(&kind);
    control.financial_source_runtime.failures.remove(&kind);
    let source = connection(control, &workspace, kind)?;
    Ok((json!(source), Some(source.state_version)))
}

fn optional_text(
    row: &serde_json::Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<Option<String>> {
    match row.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if crate::valid_bounded_text(value, max) => {
            Ok(Some(value.clone()))
        }
        _ => Err(invalid()),
    }
}
fn required_text(row: &serde_json::Map<String, Value>, key: &str, max: usize) -> Result<String> {
    optional_text(row, key, max)?.ok_or_else(invalid)
}
fn decimal(value: &Value) -> Result<String> {
    let value = match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        _ => return Err(invalid()),
    };
    if value.is_empty() || value.len() > 64 {
        return Err(invalid());
    }
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix('-').unwrap_or(&value);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i32>().map_err(|_| invalid())?),
        None => (unsigned, 0),
    };
    let parts = mantissa.split('.').collect::<Vec<_>>();
    if parts.len() > 2
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
        || exponent.unsigned_abs() > 64
    {
        return Err(invalid());
    }
    let digits = parts.concat();
    let position = parts[0].len() as i32 + exponent;
    let expanded_length = if position <= 0 {
        2 + (-position) as usize + digits.len()
    } else if position as usize >= digits.len() {
        position as usize
    } else {
        digits.len() + 1
    };
    if expanded_length + usize::from(negative) > 64 {
        return Err(invalid());
    }
    let expanded = if position <= 0 {
        format!("0.{}{}", "0".repeat((-position) as usize), digits)
    } else if position as usize >= digits.len() {
        format!("{}{}", digits, "0".repeat(position as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..position as usize],
            &digits[position as usize..]
        )
    };
    let (integer, fraction) = expanded.split_once('.').unwrap_or((&expanded, ""));
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let fraction = fraction.trim_end_matches('0');
    let normalized = if fraction.is_empty() {
        integer.into()
    } else {
        format!("{integer}.{fraction}")
    };
    Ok(if negative && normalized != "0" {
        format!("-{normalized}")
    } else {
        normalized
    })
}
struct EventSchema {
    group: &'static str,
    category: CompanyEventCategory,
    fields: &'static str,
    required: &'static str,
}
// First-party REST fields. All-quality records can omit economic fields, never event identity/process date.
const EVENT_SCHEMAS: &[EventSchema] = &[
    EventSchema {
        group: "forward_splits",
        category: CompanyEventCategory::ForwardSplit,
        fields: "symbol,cusip,isin,new_rate,old_rate,ex_date,payable_date,record_date,due_bill_redemption_date",
        required: "symbol,cusip,new_rate,old_rate,ex_date",
    },
    EventSchema {
        group: "reverse_splits",
        category: CompanyEventCategory::ReverseSplit,
        fields: "symbol,old_cusip,old_isin,new_cusip,new_isin,new_symbol,new_rate,old_rate,ex_date,payable_date,record_date",
        required: "symbol,old_cusip,new_cusip,new_rate,old_rate,ex_date",
    },
    EventSchema {
        group: "unit_splits",
        category: CompanyEventCategory::UnitSplit,
        fields: "old_symbol,old_cusip,old_isin,old_rate,new_symbol,new_cusip,new_isin,new_rate,alternate_symbol,alternate_cusip,alternate_isin,alternate_rate,effective_date,payable_date",
        required: "old_symbol,old_cusip,old_rate,new_symbol,new_cusip,new_rate,alternate_symbol,alternate_cusip,alternate_rate,effective_date",
    },
    EventSchema {
        group: "cash_dividends",
        category: CompanyEventCategory::CashDividend,
        fields: "symbol,cusip,isin,rate,special,foreign,sub_type,ex_date,payable_date,record_date,due_bill_on_date,due_bill_off_date",
        required: "symbol,cusip,rate,special,foreign,ex_date",
    },
    EventSchema {
        group: "stock_dividends",
        category: CompanyEventCategory::StockDividend,
        fields: "symbol,cusip,isin,rate,ex_date,payable_date,record_date",
        required: "symbol,cusip,rate,ex_date",
    },
    EventSchema {
        group: "spin_offs",
        category: CompanyEventCategory::SpinOff,
        fields: "source_symbol,source_cusip,source_isin,source_rate,new_symbol,new_cusip,new_isin,new_rate,ex_date,payable_date,record_date,due_bill_redemption_date",
        required: "source_symbol,source_cusip,source_rate,new_symbol,new_cusip,new_rate,ex_date",
    },
    EventSchema {
        group: "cash_mergers",
        category: CompanyEventCategory::CashMerger,
        fields: "acquiree_symbol,acquiree_cusip,acquiree_isin,acquirer_symbol,acquirer_cusip,acquirer_isin,rate,effective_date,payable_date",
        required: "acquiree_symbol,acquiree_cusip,rate,effective_date",
    },
    EventSchema {
        group: "stock_mergers",
        category: CompanyEventCategory::StockMerger,
        fields: "acquiree_symbol,acquiree_cusip,acquiree_isin,acquiree_rate,acquirer_symbol,acquirer_cusip,acquirer_isin,acquirer_rate,effective_date,payable_date",
        required: "acquiree_symbol,acquiree_cusip,acquiree_rate,acquirer_symbol,acquirer_cusip,acquirer_rate,effective_date",
    },
    EventSchema {
        group: "stock_and_cash_mergers",
        category: CompanyEventCategory::StockAndCashMerger,
        fields: "acquiree_symbol,acquiree_cusip,acquiree_isin,acquiree_rate,acquirer_symbol,acquirer_cusip,acquirer_isin,acquirer_rate,cash_rate,effective_date,payable_date",
        required: "acquiree_symbol,acquiree_cusip,acquiree_rate,acquirer_symbol,acquirer_cusip,acquirer_rate,cash_rate,effective_date",
    },
    EventSchema {
        group: "redemptions",
        category: CompanyEventCategory::Redemption,
        fields: "symbol,cusip,isin,rate,payable_date",
        required: "symbol,cusip,rate",
    },
    EventSchema {
        group: "name_changes",
        category: CompanyEventCategory::NameChange,
        fields: "old_symbol,old_cusip,old_isin,new_symbol,new_cusip,new_isin",
        required: "old_symbol,old_cusip,new_symbol,new_cusip",
    },
    EventSchema {
        group: "worthless_removals",
        category: CompanyEventCategory::WorthlessRemoval,
        fields: "symbol,cusip,isin",
        required: "symbol,cusip",
    },
    EventSchema {
        group: "rights_distributions",
        category: CompanyEventCategory::RightsDistribution,
        fields: "source_symbol,source_cusip,source_isin,new_symbol,new_cusip,new_isin,rate,ex_date,payable_date,record_date,expiration_date",
        required: "source_symbol,source_cusip,new_symbol,new_cusip,rate,ex_date,payable_date",
    },
    EventSchema {
        group: "partial_calls",
        category: CompanyEventCategory::PartialCall,
        fields: "symbol,cusip,isin,price,dividend_rate,lottery_date,lottery_type,results_publication_date,payable_date,record_date",
        required: "symbol",
    },
    EventSchema {
        group: "reorganizations",
        category: CompanyEventCategory::Reorganization,
        fields: "symbol,cusip,isin,cash_rate,stock_movements,effective_date,payable_date",
        required: "symbol,cusip,effective_date",
    },
    EventSchema {
        group: "capital_gains_distributions",
        category: CompanyEventCategory::CapitalGainsDistribution,
        fields: "symbol,cusip,isin,long_term_rate,short_term_rate,ex_date,payable_date,record_date",
        required: "symbol,cusip,ex_date",
    },
];
fn event_schema(group: &str) -> Result<&'static EventSchema> {
    EVENT_SCHEMAS
        .iter()
        .find(|schema| schema.group == group)
        .ok_or_else(invalid)
}
fn security(
    row: &serde_json::Map<String, Value>,
    prefix: &str,
) -> Result<Option<CompanyEventSecurity>> {
    use crate::protocol::CompanyEventSecurityRole;
    let key = |field: &str| {
        if prefix.is_empty() {
            field.into()
        } else {
            format!("{prefix}_{field}")
        }
    };
    let symbol = optional_text(row, &key("symbol"), 32)?;
    let isin = optional_text(row, &key("isin"), 12)?;
    let cusip = optional_text(row, &key("cusip"), 9)?;
    if symbol.is_none() && isin.is_none() && cusip.is_none() {
        return Ok(None);
    }
    if symbol.as_ref().is_some_and(|value| {
        !value.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
    }) || isin.as_ref().is_some_and(|value| {
        value.len() != 12
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    }) || cusip.as_ref().is_some_and(|value| {
        value.len() != 9
            || !value.bytes().all(|byte| {
                byte.is_ascii_uppercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'*' | b'@' | b'#')
            })
    }) {
        return Err(invalid());
    }
    let role = match prefix {
        "" => CompanyEventSecurityRole::Subject,
        "old" => CompanyEventSecurityRole::Old,
        "new" => CompanyEventSecurityRole::New,
        "source" => CompanyEventSecurityRole::Source,
        "alternate" => CompanyEventSecurityRole::Alternate,
        "acquiree" => CompanyEventSecurityRole::Acquiree,
        "acquirer" => CompanyEventSecurityRole::Acquirer,
        _ => return Err(invalid()),
    };
    Ok(Some(CompanyEventSecurity {
        role,
        symbol,
        isin,
        cusip,
    }))
}
fn optional_boolean(row: &serde_json::Map<String, Value>, key: &str) -> Result<Option<bool>> {
    match row.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        _ => Err(invalid()),
    }
}
fn optional_decimal(row: &serde_json::Map<String, Value>, key: &str) -> Result<Option<String>> {
    match row.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => decimal(value).map(Some),
    }
}
fn parse_event(group: &str, value: Value, start: Date, end: Date) -> Result<KnownCompanyEvent> {
    use crate::protocol::{CompanyEventSecurityRole, CompanyStockMovement};
    let schema = event_schema(group)?;
    let row = value.as_object().ok_or_else(invalid)?;
    if row.keys().any(|key| {
        !matches!(key.as_str(), "id" | "process_date" | "currency")
            && !schema.fields.split(',').any(|field| field == key)
    }) {
        return Err(invalid());
    }
    let action_id = required_text(row, "id", 36)?;
    if action_id.len() != 36 {
        return Err(invalid());
    }
    let action_id = uuid::Uuid::parse_str(&action_id)
        .map_err(|_| invalid())?
        .hyphenated()
        .to_string();
    let process_date = required_text(row, "process_date", 10)?;
    if !(start..=end).contains(&date(&process_date)?) {
        return Err(invalid());
    }
    let mut securities = Vec::new();
    for prefix in [
        "",
        "old",
        "new",
        "alternate",
        "source",
        "acquiree",
        "acquirer",
    ] {
        if let Some(value) = security(row, prefix)? {
            securities.push(value);
        }
    }
    let instrument_ids = ["AAPL", "MSFT"]
        .into_iter()
        .filter(|symbol| {
            securities
                .iter()
                .any(|security| security.symbol.as_deref() == Some(*symbol))
        })
        .map(|symbol| format!("equity:US:{symbol}"))
        .collect::<Vec<_>>();
    if instrument_ids.is_empty() {
        return Err(invalid());
    }
    let mut dates = Vec::new();
    for name in [
        "ex_date",
        "effective_date",
        "payable_date",
        "record_date",
        "due_bill_on_date",
        "due_bill_off_date",
        "due_bill_redemption_date",
        "expiration_date",
        "lottery_date",
        "results_publication_date",
    ] {
        if let Some(value) = optional_text(row, name, 10)? {
            date(&value)?;
            dates.push(CompanyEventDate {
                name: serde_json::from_value(json!(name)).map_err(|_| invalid())?,
                value,
            });
        }
    }
    let mut terms = Vec::new();
    for name in [
        "rate",
        "old_rate",
        "new_rate",
        "alternate_rate",
        "source_rate",
        "acquiree_rate",
        "acquirer_rate",
        "cash_rate",
        "dividend_rate",
        "price",
        "long_term_rate",
        "short_term_rate",
    ] {
        if let Some(value) = optional_decimal(row, name)? {
            if name.ends_with("_rate")
                && matches!(
                    name,
                    "old_rate"
                        | "new_rate"
                        | "alternate_rate"
                        | "source_rate"
                        | "acquiree_rate"
                        | "acquirer_rate"
                )
                && (value.starts_with('-')
                    || !value.bytes().any(|byte| matches!(byte, b'1'..=b'9')))
            {
                return Err(invalid());
            }
            terms.push(CompanyEventTerm {
                name: serde_json::from_value(json!(name)).map_err(|_| invalid())?,
                value,
            });
        }
    }
    let mut partial = schema
        .required
        .split(',')
        .any(|key| row.get(key).is_none_or(Value::is_null));
    let mut stock_movements = Vec::new();
    if let Some(value) = row.get("stock_movements").filter(|value| !value.is_null()) {
        let rows = value.as_array().ok_or_else(invalid)?;
        if rows.len() > 16 {
            return Err(invalid());
        }
        for value in rows {
            let row = value.as_object().ok_or_else(invalid)?;
            if row.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "symbol" | "cusip" | "isin" | "new_rate" | "source_rate"
                )
            }) {
                return Err(invalid());
            }
            let mut security = security(row, "")?.ok_or_else(invalid)?;
            security.role = CompanyEventSecurityRole::Movement;
            let new_rate = optional_decimal(row, "new_rate")?;
            let source_rate = optional_decimal(row, "source_rate")?;
            if [new_rate.as_ref(), source_rate.as_ref()]
                .into_iter()
                .flatten()
                .any(|value| {
                    value.starts_with('-') || !value.bytes().any(|byte| matches!(byte, b'1'..=b'9'))
                })
            {
                return Err(invalid());
            }
            partial |= security.symbol.is_none()
                || security.cusip.is_none()
                || new_rate.is_none()
                || source_rate.is_none();
            stock_movements.push(CompanyStockMovement {
                security,
                new_rate,
                source_rate,
            });
        }
    }
    let currency = optional_text(row, "currency", 3)?;
    if currency.as_ref().is_some_and(|value| {
        value.len() != 3 || !value.bytes().all(|byte| byte.is_ascii_uppercase())
    }) {
        return Err(invalid());
    }
    Ok(KnownCompanyEvent {
        action_id,
        instrument_ids,
        category: schema.category,
        process_date,
        dates,
        securities,
        terms,
        stock_movements,
        currency,
        special: optional_boolean(row, "special")?,
        foreign: optional_boolean(row, "foreign")?,
        sub_type: optional_text(row, "sub_type", 64)?,
        lottery_type: optional_text(row, "lottery_type", 64)?,
        partial,
    })
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionPage {
    corporate_actions: serde_json::Map<String, Value>,
    next_page_token: Value,
}

// Validate duplicate object keys throughout the bounded provider response before typed parsing.
// Value alone would silently keep the last key, including contradictory event terms/tokens.
struct UniqueJson;
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UniqueJson;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("JSON with unique object fields")
            }
            fn visit_bool<E: serde::de::Error>(
                self,
                _: bool,
            ) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_i64<E: serde::de::Error>(self, _: i64) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_u64<E: serde::de::Error>(self, _: u64) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_string<E: serde::de::Error>(
                self,
                _: String,
            ) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_none<E: serde::de::Error>(self) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<UniqueJson, E> {
                Ok(UniqueJson)
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> std::result::Result<UniqueJson, A::Error> {
                while sequence.next_element::<UniqueJson>()?.is_some() {}
                Ok(UniqueJson)
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<UniqueJson, A::Error> {
                let mut keys = std::collections::HashSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key) {
                        return Err(serde::de::Error::custom("duplicate provider object field"));
                    }
                    map.next_value::<UniqueJson>()?;
                }
                Ok(UniqueJson)
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}
fn strict_json<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T> {
    serde_json::from_slice::<UniqueJson>(body).map_err(|_| invalid())?;
    serde_json::from_slice(body).map_err(|_| invalid())
}
fn token_encoded(token: &str) -> Result<String> {
    if !crate::valid_bounded_text(token, 2048) || !token.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(invalid());
    }
    Ok(token
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect())
}
pub(crate) fn allowed_actions_path(path: &str) -> bool {
    let Some(rest) = path
        .strip_prefix("/v1/corporate-actions?symbols=AAPL,MSFT&region=us&data_quality=all&start=")
    else {
        return false;
    };
    let Some((start, rest)) = rest.split_once("&end=") else {
        return false;
    };
    let Some((end, rest)) = rest.split_once("&limit=100") else {
        return false;
    };
    let (Ok(start), Ok(end)) = (date(start), date(end)) else {
        return false;
    };
    end - start == Duration::days(60)
        && (rest.is_empty()
            || rest.strip_prefix("&page_token=").is_some_and(|token| {
                !token.is_empty()
                    && token.len() <= 6144
                    && token.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric()
                            || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'%')
                    })
            }))
}
fn valid_isin(value: &str) -> bool {
    if value.len() != 12
        || !value.as_bytes()[..2].iter().all(|b| b.is_ascii_uppercase())
        || !value
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        || !value.as_bytes()[11].is_ascii_digit()
    {
        return false;
    }
    let digits: Vec<u8> = value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_digit() {
                vec![byte - b'0']
            } else {
                let n = byte - b'A' + 10;
                vec![n / 10, n % 10]
            }
        })
        .collect();
    digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, digit)| {
            let n = digit * if i % 2 == 1 { 2 } else { 1 };
            u32::from(n / 10 + n % 10)
        })
        .sum::<u32>()
        % 10
        == 0
}

fn positive_id(row: &serde_json::Map<String, Value>, key: &str) -> Result<u64> {
    row.get(key)
        .and_then(Value::as_u64)
        .filter(|id| (1..=MAX_SEQUENCE).contains(id))
        .ok_or_else(invalid)
}

fn broker_instruments(catalog: &[u8], exchanges: &[u8]) -> Result<Vec<BrokerInstrumentMetadata>> {
    let catalog: Vec<Value> = strict_json(catalog)?;
    let exchanges: Vec<Value> = strict_json(exchanges)?;
    if catalog.len() > 25_000 || exchanges.len() > 1_000 {
        return Err(invalid());
    }
    let mut tickers = std::collections::HashSet::new();
    let mut selected = Vec::new();
    for value in catalog {
        let row = value.as_object().ok_or_else(invalid)?;
        let ticker = required_text(row, "ticker", 128)?;
        if !tickers.insert(ticker.clone()) {
            return Err(invalid());
        }
        let instrument_id = match ticker.as_str() {
            "AAPL_US_EQ" => "equity:US:AAPL",
            "MSFT_US_EQ" => "equity:US:MSFT",
            _ => continue,
        };
        let isin = required_text(row, "isin", 12)?;
        if !valid_isin(&isin)
            || required_text(row, "type", 32)? != "STOCK"
            || required_text(row, "currencyCode", 3)? != "USD"
        {
            return Err(invalid());
        }
        let quantity = match row.get("maxOpenQuantity") {
            None | Some(Value::Null) => None,
            Some(value) => {
                let value = decimal(value)?;
                if value.starts_with('-') {
                    return Err(invalid());
                }
                Some(value)
            }
        };
        let extended_hours = match row.get("extendedHours") {
            None | Some(Value::Null) => None,
            Some(Value::Bool(value)) => Some(*value),
            _ => return Err(invalid()),
        };
        selected.push(BrokerInstrumentMetadata {
            instrument_id: instrument_id.into(),
            provider_symbol: ticker,
            isin,
            currency: "USD".into(),
            display_name: required_text(row, "name", 256)?,
            working_schedule_id: positive_id(row, "workingScheduleId")?,
            exchange_name: String::new(),
            schedule_events: Vec::new(),
            max_open_quantity: quantity,
            extended_hours,
            canonical_security_identity: DataSourceStatus::Unverified,
        });
    }
    if selected.len() != 2 || selected[0].isin == selected[1].isin {
        return Err(invalid());
    }
    let mut exchange_ids = std::collections::HashSet::new();
    let mut schedules = HashMap::new();
    let mut total_events = 0usize;
    for value in exchanges {
        let row = value.as_object().ok_or_else(invalid)?;
        if !exchange_ids.insert(positive_id(row, "id")?) {
            return Err(invalid());
        }
        let name = required_text(row, "name", 160)?;
        let entries = row
            .get("workingSchedules")
            .and_then(Value::as_array)
            .ok_or_else(invalid)?;
        if entries.len() > 1_000 || schedules.len() + entries.len() > 10_000 {
            return Err(invalid());
        }
        for entry in entries {
            let row = entry.as_object().ok_or_else(invalid)?;
            let id = positive_id(row, "id")?;
            let events = row
                .get("timeEvents")
                .and_then(Value::as_array)
                .ok_or_else(invalid)?;
            total_events = total_events.checked_add(events.len()).ok_or_else(invalid)?;
            if events.len() > 4_000 || total_events > 100_000 || schedules.contains_key(&id) {
                return Err(invalid());
            }
            let mut parsed = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for value in events {
                let row = value.as_object().ok_or_else(invalid)?;
                let date = required_text(row, "date", 64)?;
                let instant = timestamp(&date)?;
                let event_type = required_text(row, "type", 32)?;
                let typed = serde_json::from_value::<crate::protocol::BrokerScheduleEventType>(
                    json!(event_type),
                )
                .map_err(|_| invalid())?;
                if !seen.insert((instant.unix_timestamp_nanos(), event_type)) {
                    return Err(invalid());
                }
                parsed.push((
                    instant,
                    BrokerScheduleEvent {
                        date,
                        event_type: typed,
                    },
                ));
            }
            parsed.sort_by_key(|(instant, _)| *instant);
            validate_schedule_events(&parsed)?;
            schedules.insert(
                id,
                (
                    name.clone(),
                    parsed
                        .into_iter()
                        .map(|(_, event)| event)
                        .collect::<Vec<_>>(),
                ),
            );
        }
    }
    for instrument in &mut selected {
        let (name, events) = schedules
            .get(&instrument.working_schedule_id)
            .ok_or_else(invalid)?;
        if events.is_empty() {
            return Err(invalid());
        }
        instrument.exchange_name = name.clone();
        instrument.schedule_events = events.clone();
    }
    selected.sort_by(|a, b| a.instrument_id.cmp(&b.instrument_id));
    Ok(selected)
}

fn validate_schedule_events(events: &[(OffsetDateTime, BrokerScheduleEvent)]) -> Result<()> {
    use crate::protocol::BrokerScheduleEventType as Event;
    // Unknown initial states allow a query window to start inside a session or
    // break. Once a transition is observed, conflicting transitions are invalid.
    // An unfinished final session is also allowed; none of this establishes
    // current venue state or complete exchange-calendar coverage.
    let mut regular = None;
    let mut paused = None;
    let mut last_session = None;
    let mut last_regular_time = None;
    for (instant, event) in events {
        if matches!(
            event.event_type,
            Event::Open | Event::Close | Event::BreakStart | Event::BreakEnd
        ) {
            if last_regular_time == Some(*instant) {
                return Err(invalid());
            }
            last_regular_time = Some(*instant);
        }
        match event.event_type {
            Event::Open => {
                if regular == Some(true) {
                    return Err(invalid());
                }
                regular = Some(true);
                paused = Some(false);
                last_session = Some(Event::Open);
            }
            Event::Close => {
                if regular == Some(false) || paused == Some(true) {
                    return Err(invalid());
                }
                regular = Some(false);
                paused = Some(false);
                last_session = None;
            }
            Event::BreakStart => {
                if regular == Some(false) || paused == Some(true) {
                    return Err(invalid());
                }
                regular = Some(true);
                paused = Some(true);
            }
            Event::BreakEnd => {
                if regular == Some(false) || paused == Some(false) {
                    return Err(invalid());
                }
                regular = Some(true);
                paused = Some(false);
            }
            Event::PreMarketOpen | Event::AfterHoursOpen | Event::OvernightOpen => {
                if regular == Some(true) || last_session == Some(event.event_type) {
                    return Err(invalid());
                }
                regular = Some(false);
                paused = Some(false);
                last_session = Some(event.event_type);
            }
            Event::AfterHoursClose => {
                if regular == Some(true)
                    || matches!(
                        last_session,
                        Some(Event::PreMarketOpen | Event::OvernightOpen)
                    )
                {
                    return Err(invalid());
                }
                if last_session == Some(Event::AfterHoursClose) {
                    return Err(invalid());
                }
                regular = Some(false);
                paused = Some(false);
                last_session = Some(Event::AfterHoursClose);
            }
        }
    }
    Ok(())
}

const BROKER_READS: [(&str, u64); 3] = [
    ("/api/v0/equity/account/summary", 5),
    ("/api/v0/equity/metadata/instruments", 50),
    ("/api/v0/equity/metadata/exchanges", 30),
];
// Quotas are shared across workspaces, keys and source generations by authenticated account id.
// None is an unrepresentable provider deadline and remains blocked until process restart.
struct ReadQuota {
    until: Option<Instant>,
    in_flight: bool,
}
type ReadQuotas = HashMap<(String, &'static str), ReadQuota>;
static BROKER_READ_QUOTAS: OnceLock<Mutex<ReadQuotas>> = OnceLock::new();

// A reservation owns only its pending paths. Completed paths can be acquired by a
// later reader without an older guard's Drop releasing that reader's hold.
pub(crate) struct ReadReservation {
    remote: String,
    pending: Vec<&'static str>,
    wall_anchor: OffsetDateTime,
    started: Instant,
}

impl ReadReservation {
    pub(crate) fn complete(
        &mut self,
        path: &'static str,
        status: u16,
        limit: Option<&crate::provider_io::ProviderRateLimit>,
    ) -> Result<()> {
        let mut seconds = BROKER_READS
            .iter()
            .find(|(candidate, _)| *candidate == path)
            .map(|(_, seconds)| *seconds)
            .ok_or_else(invalid)?;
        let exhausted = status == 429 || limit.and_then(|limit| limit.remaining) == Some(0);
        if exhausted {
            seconds = seconds.max(
                limit
                    .and_then(|limit| limit.retry_after_seconds)
                    .unwrap_or(1),
            );
            if let Some(reset) = limit.and_then(|limit| limit.reset_at.as_ref()) {
                // Quota accounting remains independent of a workspace/clock change
                // while HTTP is in flight. This is not financial evidence time.
                let received = self.wall_anchor
                    + Duration::try_from(self.started.elapsed()).map_err(|_| invalid())?;
                let delta = timestamp(reset)? - received;
                if delta > Duration::ZERO {
                    seconds = seconds.max(delta.whole_seconds() as u64 + 1);
                }
            }
        }
        self.release(path, seconds)?;
        self.pending.retain(|candidate| *candidate != path);
        Ok(())
    }

    fn release(&self, path: &'static str, seconds: u64) -> Result<()> {
        let until = Instant::now().checked_add(StdDuration::from_secs(seconds));
        let mut quotas = BROKER_READ_QUOTAS
            .get_or_init(Default::default)
            .lock()
            .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
        let quota = quotas
            .get_mut(&(self.remote.clone(), path))
            .ok_or_else(invalid)?;
        quota.until = match (quota.until, until) {
            (Some(a), Some(b)) => Some(a.max(b)),
            _ => None,
        };
        quota.in_flight = false;
        Ok(())
    }
}

impl Drop for ReadReservation {
    fn drop(&mut self) {
        // Cancellation, transport failure, or invalid rate headers still consume
        // the minimum interval from completion. Unattempted reserved paths are
        // conservatively cooled down too; selection changes cannot reset quota.
        for path in &self.pending {
            if let Some((_, seconds)) = BROKER_READS.iter().find(|(candidate, _)| candidate == path)
            {
                let _ = self.release(path, *seconds);
            }
        }
    }
}

fn reserve_reads(
    remote: &str,
    paths: &[(&'static str, u64)],
    wall_anchor: OffsetDateTime,
) -> Result<ReadReservation> {
    let instant = Instant::now();
    let mut quotas = BROKER_READ_QUOTAS
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    quotas.retain(|_, quota| quota.in_flight || quota.until.is_none_or(|until| until > instant));
    if paths
        .iter()
        .any(|(path, _)| quotas.contains_key(&(remote.into(), *path)))
    {
        return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
    }
    if quotas.len() + paths.len() > 4096 {
        return Err(TradeXError::new("PROVIDER_BACKPRESSURE"));
    }
    for (path, seconds) in paths {
        quotas.insert(
            (remote.into(), *path),
            ReadQuota {
                until: instant.checked_add(StdDuration::from_secs(*seconds)),
                in_flight: true,
            },
        );
    }
    Ok(ReadReservation {
        remote: remote.into(),
        pending: paths.iter().map(|(path, _)| *path).collect(),
        wall_anchor,
        started: instant,
    })
}

fn reserve_broker_reads(account: &AccountConnection, now: &TimeStatus) -> Result<ReadReservation> {
    let data = account.data.as_ref().ok_or_else(invalid)?;
    if let Some(last) = account.last_successful_sync.as_ref() {
        let age = timestamp(&now.wall_clock)? - timestamp(last)?;
        if age < Duration::seconds(5) {
            return Err(TradeXError::new("PROVIDER_RATE_LIMITED"));
        }
    }
    reserve_reads(
        &data.remote_account_id,
        &BROKER_READS,
        timestamp(&now.wall_clock)?,
    )
}

// Every known account reader joins the shared summary quota, even before metadata is selected.
// Initial unknown-account authentication cannot guess an account id before its first response.
pub(crate) fn reserve_tracked_summary(remote_account_id: &str) -> Result<ReadReservation> {
    reserve_reads(
        remote_account_id,
        &BROKER_READS[..1],
        OffsetDateTime::now_utc(),
    )
}

fn read_broker_metadata(
    control: &Arc<Mutex<ControlPlane>>,
    binding: &Binding,
    account: &AccountConnection,
    credentials: &crate::provider_io::Credentials,
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
    deadline: Instant,
) -> Result<Observation> {
    let saved = account.data.as_ref().ok_or_else(invalid)?;
    let now = control
        .lock()
        .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
        .time
        .status(&binding.workspace)?;
    let mut reservation = reserve_broker_reads(account, &now)?;
    let headers = crate::provider_io::trading212_source_headers(credentials)?;
    let mut secrets = credentials.values()?;
    secrets.push(
        headers["Authorization"]
            .to_str()
            .map_err(|_| invalid())?
            .strip_prefix("Basic ")
            .ok_or_else(invalid)?
            .into(),
    );
    // The scheduler can wait. Recheck the entire fixed HTTP timeout before admission,
    // so an operation begun near the job deadline cannot overrun that deadline.
    let http_current = || current() && http_budget_remains(deadline);
    let http = crate::provider_io::p3_provider_http(http, &http_current, &saved.remote_account_id);
    let mut first_receipt = None;
    let mut get = |path, bound| -> Result<Vec<u8>> {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let outcome = http.request_with_rate_limit(
            ProviderEndpoint::Trading212Live,
            crate::provider_io::ProviderHttpMethod::Get,
            path,
            headers.clone(),
            None,
        );
        // Account for actual endpoint completion even on transport failure/cancellation.
        let (status, limit) = outcome
            .as_ref()
            .map(|(response, limit)| (response.status, limit.as_ref()))
            .unwrap_or((0, None));
        reservation.complete(path, status, limit)?;
        let (response, _) = outcome?;
        let received = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
            .time
            .status(&binding.workspace)?;
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if received.confidence != TimeConfidence::Trusted {
            return Err(TradeXError::new("CLOCK_SKEW"));
        }
        if first_receipt.is_none() {
            first_receipt = Some(received.clone());
        }
        match response.status {
            200 => (),
            401 => return Err(TradeXError::new("PROVIDER_AUTH_FAILED")),
            403 => return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED")),
            429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
            _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
        }
        if response.body.len() > bound {
            return Err(invalid());
        }
        let parsed: Value = strict_json(&response.body)?;
        if crate::provider_io::contains_secret(&parsed, &secrets) {
            return Err(invalid());
        }
        Ok(response.body)
    };
    let summary: Value = strict_json(&get("/api/v0/equity/account/summary", 2 * 1024 * 1024)?)?;
    let summary = summary.as_object().ok_or_else(invalid)?;
    let id = summary
        .get("id")
        .and_then(Value::as_u64)
        .filter(|id| *id > 0)
        .ok_or_else(invalid)?;
    let currency = required_text(summary, "currency", 3)?;
    if id.to_string() != saved.remote_account_id || saved.currency.as_deref() != Some(&currency) {
        return Err(invalid());
    }
    let catalog = get("/api/v0/equity/metadata/instruments", 8 * 1024 * 1024)?;
    let exchanges = get("/api/v0/equity/metadata/exchanges", 2 * 1024 * 1024)?;
    let instruments = broker_instruments(&catalog, &exchanges)?;
    let received = first_receipt.ok_or_else(invalid)?;
    let projected = binding.projection()?;
    let material_version = hash(&json!([
        projected,
        received.wall_clock,
        received.monotonic_ms,
        currency,
        instruments,
        FinancialEvidenceQuality::TenMinuteMetadata
    ]))?;
    Ok(Observation {
        binding: binding.clone(),
        received: received.clone(),
        evidence: FinancialSourceEvidence::BrokerInstruments(BrokerInstrumentEvidence {
            binding: projected,
            material_version,
            observed_at: received.wall_clock,
            provider_observed_at: None,
            provider_quality: FinancialEvidenceQuality::TenMinuteMetadata,
            account_currency: currency,
            instruments,
        }),
    })
}

fn http_budget_remains(deadline: Instant) -> bool {
    Instant::now()
        .checked_add(StdDuration::from_secs(12))
        .is_some_and(|latest_completion| latest_completion < deadline)
}

// Authentication may outlive a command because OS dialogs require human input.
// Keep at most one outstanding read per reference and32 in the process. A timed
// out caller drops its receiver; the worker can only return zeroizing credentials,
// never run HTTP or publish an observation. Holds end at actual vault completion.
fn pending_vault_reads() -> &'static Mutex<std::collections::HashSet<String>> {
    static READS: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    READS.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}
struct VaultReadHold(String);
impl VaultReadHold {
    fn acquire(reference: &str) -> Result<Self> {
        let mut reads = pending_vault_reads()
            .lock()
            .map_err(|_| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        if reads.len() >= 32 || !reads.insert(reference.to_owned()) {
            return Err(TradeXError::new("PROVIDER_BACKPRESSURE"));
        }
        Ok(Self(reference.to_owned()))
    }
}
impl Drop for VaultReadHold {
    fn drop(&mut self) {
        if let Ok(mut reads) = pending_vault_reads().lock() {
            reads.remove(&self.0);
        }
    }
}
fn read_credentials_before_deadline(
    vault: &(impl CredentialVault + Clone + Send + 'static),
    reference: &str,
    deadline: Instant,
) -> Result<crate::provider_io::Credentials> {
    if Instant::now() >= deadline {
        return Err(TradeXError::new("PROVIDER_UNAVAILABLE"));
    }
    let hold = VaultReadHold::acquire(reference)?;
    let vault = vault.clone();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("financial-source-vault".into())
        .spawn(move || {
            let result = vault.get(&hold.0);
            drop(hold);
            // Drop a late result (including its secrets) when the caller timed out.
            let _ = sender.send(result);
        })
        .map_err(|_| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .ok_or_else(|| TradeXError::new("PROVIDER_UNAVAILABLE"))?;
    receiver
        .recv_timeout(remaining)
        .map_err(|_| TradeXError::new("PROVIDER_UNAVAILABLE"))?
}

pub fn execute_refresh(
    control: &Arc<Mutex<ControlPlane>>,
    request: &Value,
    consumer: &str,
    vault: &(impl CredentialVault + Clone + Send + 'static),
    http: &impl ProviderHttp,
) -> Value {
    let deadline = Instant::now()
        + StdDuration::from_secs(if request["command"] == "data.instrument.refresh" {
            45
        } else {
            60
        });
    let prepare = (|| {
        if !provider_order_consumer_allowed(consumer) {
            return Err(TradeXError::new("IPC_ACCESS_DENIED"));
        }
        let envelope: CommandEnvelope = payload(request.clone())?;
        if envelope.schema_version != 1 {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        let kind = match envelope.command.as_str() {
            "data.actions.refresh" => FinancialSourceKind::CorporateActions,
            "data.instrument.refresh" => FinancialSourceKind::BrokerInstruments,
            _ => return Err(TradeXError::new("IPC_COMMAND_UNKNOWN")),
        };
        if !crate::valid_bounded_text(&envelope.request_id, 128) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let input: DataSourceMutation = payload(envelope.payload)?;
        let mut control = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        control.require_workspace(&input.workspace_id)?;
        let now = control.time.status(&input.workspace_id)?;
        if now.confidence != TimeConfidence::Trusted {
            return Err(TradeXError::new("CLOCK_SKEW"));
        }
        let store = control.store.as_ref().unwrap();
        let saved = store.financial_source(kind)?;
        if saved.version(kind, &input.workspace_id) != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let account = store.account(
            saved
                .connection_id
                .as_ref()
                .ok_or_else(|| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?,
        )?;
        if !kind.eligible(&account) {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        let today = timestamp(&now.wall_clock)?.date();
        let sequence = control
            .financial_source_runtime
            .sequences
            .get(&kind)
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .filter(|sequence| *sequence <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        let binding = Binding {
            workspace: input.workspace_id,
            kind,
            source_version: input.expected_state_version,
            account: account.connection_id.clone(),
            account_version: account.state_version.clone(),
            reference: account.credential_ref(),
            session: control.session.clone(),
            runtime_epoch: control.financial_source_runtime.epoch.clone(),
            clock_generation: control.time.generation().into(),
            sequence,
        };
        control
            .financial_source_runtime
            .sequences
            .insert(kind, sequence);
        control.financial_source_runtime.observations.remove(&kind);
        control.financial_source_runtime.failures.remove(&kind);
        Ok((
            binding,
            today - Duration::days(30),
            today + Duration::days(30),
            account,
        ))
    })();
    let (binding, start, end, account) = match prepare {
        Ok(value) => value,
        Err(error) => return crate::failure_reply(crate::request_id(request), error),
    };
    let current = || {
        Instant::now() < deadline
            && control
                .lock()
                .is_ok_and(|control| binding.current(&control))
    };
    let read = (|| {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let credentials = read_credentials_before_deadline(vault, &binding.reference, deadline)?;
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if binding.kind == FinancialSourceKind::BrokerInstruments {
            return read_broker_metadata(
                control,
                &binding,
                &account,
                &credentials,
                http,
                &current,
                deadline,
            );
        }
        let headers = crate::quote_source::source_headers(&credentials)?;
        let secrets = credentials.values()?;
        let http_current = || current() && http_budget_remains(deadline);
        let http = crate::provider_io::p3_provider_http(http, &http_current, &binding.reference);
        let base = format!(
            "/v1/corporate-actions?symbols=AAPL,MSFT&region=us&data_quality=all&start={start}&end={end}&limit=100"
        );
        let mut next: Option<String> = None;
        let mut seen_tokens = std::collections::HashSet::new();
        let mut seen_ids = std::collections::HashSet::new();
        let mut total_bytes = 0usize;
        let mut actions = Vec::new();
        let mut first_receipt = None;
        for _ in 0..10 {
            if !current() {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
            let path = if let Some(token) = next.as_ref() {
                format!("{base}&page_token={}", token_encoded(token)?)
            } else {
                base.clone()
            };
            let response = http.request(
                ProviderEndpoint::AlpacaMarketData,
                crate::provider_io::ProviderHttpMethod::Get,
                &path,
                headers.clone(),
                None,
            )?;
            let received = control
                .lock()
                .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
                .time
                .status(&binding.workspace)?;
            if received.confidence != TimeConfidence::Trusted || !current() {
                return Err(TradeXError::new("CLOCK_SKEW"));
            }
            if first_receipt.is_none() {
                first_receipt = Some(received);
            }
            match response.status {
                200 => (),
                401 => return Err(TradeXError::new("PROVIDER_AUTH_FAILED")),
                403 => return Err(TradeXError::new("PROVIDER_PERMISSION_BLOCKED")),
                429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
                _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
            }
            let body = response.body;
            total_bytes = total_bytes.checked_add(body.len()).ok_or_else(invalid)?;
            if body.len() > 512 * 1024 || total_bytes > 4 * 1024 * 1024 {
                return Err(invalid());
            }
            let parsed: Value = strict_json(&body)?;
            if crate::provider_io::contains_secret(&parsed, &secrets) {
                return Err(invalid());
            }
            let page: ActionPage = serde_json::from_value(parsed).map_err(|_| invalid())?;
            let mut page_count = 0;
            for (group, records) in page.corporate_actions {
                if event_schema(&group).is_err() {
                    return Err(invalid());
                }
                let records = records.as_array().ok_or_else(invalid)?;
                page_count += records.len();
                if page_count > 100 || actions.len() + records.len() > 1000 {
                    return Err(invalid());
                }
                for record in records {
                    let event = parse_event(&group, record.clone(), start, end)?;
                    if !seen_ids.insert(event.action_id.clone()) {
                        return Err(invalid());
                    }
                    actions.push(event);
                }
            }
            next = match page.next_page_token {
                Value::Null => None,
                Value::String(token) => Some(token),
                _ => return Err(invalid()),
            };
            if let Some(token) = next.as_ref() {
                token_encoded(token)?;
                if !seen_tokens.insert(token.clone()) {
                    return Err(invalid());
                }
            } else {
                actions.sort_by(|left, right| {
                    (&left.process_date, &left.action_id)
                        .cmp(&(&right.process_date, &right.action_id))
                });
                let received = first_receipt.ok_or_else(invalid)?;
                let projected = binding.projection()?;
                let material_version = hash(&json!([
                    projected,
                    received.wall_clock,
                    received.monotonic_ms,
                    start.to_string(),
                    end.to_string(),
                    FinancialEvidenceQuality::DelayedProcessDateQuery,
                    true,
                    actions
                ]))?;
                let evidence = FinancialSourceEvidence::CorporateActions(CorporateActionEvidence {
                    binding: projected,
                    material_version,
                    observed_at: received.wall_clock.clone(),
                    provider_observed_at: None,
                    provider_quality: FinancialEvidenceQuality::DelayedProcessDateQuery,
                    coverage_start: start.to_string(),
                    coverage_end: end.to_string(),
                    query_complete: true,
                    actions,
                });
                return Ok(Observation {
                    binding: binding.clone(),
                    received,
                    evidence,
                });
            }
        }
        Err(invalid())
    })();
    let finish = (|| {
        let mut control = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if !binding.current(&control) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        match read {
            Ok(observation) if Instant::now() < deadline => {
                control
                    .financial_source_runtime
                    .observations
                    .insert(binding.kind, observation);
            }
            Ok(_) => {
                control
                    .financial_source_runtime
                    .failures
                    .insert(binding.kind, "PROVIDER_UNAVAILABLE".into());
            }
            Err(error) => {
                control
                    .financial_source_runtime
                    .failures
                    .insert(binding.kind, error.code);
            }
        }
        connection(&mut control, &binding.workspace, binding.kind)
    })();
    match finish {
        Ok(source) => crate::success_reply(crate::request_id(request), json!(source), None),
        Err(error) => crate::failure_reply(crate::request_id(request), error),
    }
}
