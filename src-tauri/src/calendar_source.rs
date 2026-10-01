use crate::{
    ControlPlane, failure_reply, payload,
    protocol::{
        CalendarCapability, CalendarCapabilityResult, CalendarConfigure, CalendarConnection,
        CommandEnvelope, DataSourceAccountChoice, DataSourceEntry, DataSourceMutation,
        DataSourceQuery, DataSourceStatus, MAX_SEQUENCE, MarketDataStatus, MarketDetail,
        MarketSession, Result, TimeConfidence, TimeStatus, TradeXError,
    },
    provider_io::{CredentialVault, ProviderEndpoint, ProviderHttp},
    provider_order_consumer_allowed,
    providers::{AccountConnection, ConnectionState},
    request_id, success_reply,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use time::{
    Date, Duration, OffsetDateTime,
    format_description::{self, well_known::Rfc3339},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SavedCalendarSource {
    pub generation: u64,
    pub connection_id: Option<String>,
}
impl SavedCalendarSource {
    pub fn version(&self, workspace: &str) -> String {
        format!("calendar:{workspace}:{}", self.generation)
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
fn eligible(account: &AccountConnection) -> bool {
    account.provider_id == "alpaca"
        && account.environment == "PAPER"
        && account.connection_state == ConnectionState::Connected
        && matches!(
            account.health.credential.as_str(),
            "CONFIGURED" | "UNCHECKED"
        )
}
#[derive(Clone)]
struct Binding {
    workspace: String,
    source_version: String,
    account: String,
    account_version: String,
    reference: String,
    session: String,
    clock_generation: String,
    sequence: u64,
}
impl Binding {
    fn current(&self, control: &ControlPlane) -> bool {
        control.session == self.session
            && control.time.generation() == self.clock_generation
            && control.require_workspace(&self.workspace).is_ok()
            && control.calendar_request_sequence == self.sequence
            && control.store.as_ref().is_some_and(|store| {
                store
                    .calendar_source()
                    .is_ok_and(|source| source.version(&self.workspace) == self.source_version)
                    && store.account(&self.account).is_ok_and(|account| {
                        eligible(&account)
                            && account.state_version == self.account_version
                            && account.credential_ref() == self.reference
                    })
            })
    }
}
#[derive(Clone)]
pub(crate) struct CalendarObservation {
    binding: Binding,
    received: TimeStatus,
    start: Date,
    end: Date,
    version: String,
    days: Vec<Day>,
}
impl CalendarObservation {
    fn fresh(&self, control: &ControlPlane, now: &TimeStatus) -> bool {
        let age = timestamp(&now.wall_clock)
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
            && age.is_some_and(|age| age >= Duration::ZERO && age <= Duration::seconds(30))
    }
}
fn capability(
    capability: CalendarCapability,
    status: DataSourceStatus,
    reason: &str,
) -> CalendarCapabilityResult {
    CalendarCapabilityResult {
        capability,
        status,
        reason: reason.into(),
    }
}
pub(crate) fn connection(
    control: &mut ControlPlane,
    workspace: &str,
) -> Result<CalendarConnection> {
    control.require_workspace(workspace)?;
    let now = control.time.status(workspace)?;
    let store = control.store.as_ref().unwrap();
    let saved = store.calendar_source()?;
    let accounts = store.accounts()?;
    let configured = saved.connection_id.is_some();
    let usable = saved
        .connection_id
        .as_ref()
        .is_none_or(|id| store.account(id).is_ok_and(|account| eligible(&account)));
    let observation = control.calendar_observation.as_ref();
    let fresh = observation.is_some_and(|observation| observation.fresh(control, &now));
    let (status, reason) = if !configured {
        (
            DataSourceStatus::BlockedExternal,
            "Select a saved Alpaca Paper key for read-only XNAS calendar access.",
        )
    } else if !usable {
        (
            DataSourceStatus::Unavailable,
            "The selected saved Paper key is unavailable. Reconnect or select another eligible account.",
        )
    } else if fresh {
        (
            DataSourceStatus::Available,
            "Authenticated XNAS calendar is current. Corporate actions, halts, adjustment and all financial guards remain separate.",
        )
    } else if observation.is_some() || control.calendar_failure.is_some() {
        (
            DataSourceStatus::Unavailable,
            "Calendar evidence is stale or the last refresh failed. Revalidate time if needed, then refresh the selected calendar.",
        )
    } else {
        (
            DataSourceStatus::Unverified,
            "Saved calendar key selected; authenticated XNAS calendar access has not been checked.",
        )
    };
    let capabilities = vec![
        capability(CalendarCapability::MarketCalendar, status.clone(), reason),
        capability(
            CalendarCapability::CorporateActions,
            DataSourceStatus::BlockedExternal,
            "Corporate-action production evidence is not configured.",
        ),
        capability(
            CalendarCapability::Halts,
            DataSourceStatus::BlockedExternal,
            "Execution-provider halt/tradability evidence is not configured.",
        ),
        capability(
            CalendarCapability::HistoricalAdjustment,
            DataSourceStatus::Unverified,
            "Calendar access does not prove historical adjustment.",
        ),
    ];
    Ok(CalendarConnection {
        workspace_id: workspace.into(),
        source_id: "OD-005".into(),
        state_version: saved.version(workspace),
        configured,
        connection_id: saved.connection_id,
        environment: "PAPER".into(),
        status,
        availability_reason: reason.into(),
        eligible_accounts: accounts
            .into_iter()
            .filter(eligible)
            .map(|account| DataSourceAccountChoice {
                connection_id: account.connection_id,
                display_name: account.label,
            })
            .collect(),
        observed_at: observation.map(|o| o.received.wall_clock.clone()),
        calendar_version: observation.map(|o| o.version.clone()),
        coverage_start: observation.map(|o| o.start.to_string()),
        coverage_end: observation.map(|o| o.end.to_string()),
        capability_statuses: capabilities,
    })
}
pub(crate) fn metadata(
    control: &mut ControlPlane,
    request: CommandEnvelope,
    consumer: &str,
) -> Result<(Value, Option<String>)> {
    if !provider_order_consumer_allowed(consumer) {
        return Err(TradeXError::new("IPC_ACCESS_DENIED"));
    }
    if request.command == "data.calendar.connection" {
        let input: DataSourceQuery = payload(request.payload)?;
        let source = connection(control, &input.workspace_id)?;
        return Ok((json!(source), Some(source.state_version)));
    }
    let (workspace, expected, selected) = if request.command == "data.calendar.configure" {
        let input: CalendarConfigure = payload(request.payload)?;
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
    let mut saved = store.calendar_source()?;
    if saved.version(&workspace) != expected {
        return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
    }
    if let Some(id) = selected.as_ref() {
        let account = store.account(id)?;
        if !eligible(&account) {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
    }
    saved.connection_id = selected;
    store.save_calendar_source(saved)?;
    control.calendar_observation = None;
    control.calendar_failure = None;
    control
        .data_source_observations
        .remove(&(workspace.clone(), "OD-005".into()));
    let source = connection(control, &workspace)?;
    Ok((json!(source), Some(source.state_version)))
}

fn invalid() -> TradeXError {
    TradeXError::new("PROVIDER_RESPONSE_INVALID")
}
fn timestamp(value: &str) -> Result<OffsetDateTime> {
    if !crate::valid_bounded_text(value, 64) {
        return Err(invalid());
    }
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| invalid())
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
fn utc(value: &str) -> Result<OffsetDateTime> {
    let value = timestamp(value)?;
    if !value.offset().is_utc() {
        return Err(invalid());
    }
    Ok(value)
}
#[derive(Clone)]
struct Day {
    date: Date,
    core_start: OffsetDateTime,
    core_end: OffsetDateTime,
    pre: Option<(OffsetDateTime, OffsetDateTime)>,
    post: Option<(OffsetDateTime, OffsetDateTime)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    market: FrameMarket,
    calendar: Vec<FrameDay>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameMarket {
    mic: Option<String>,
    acronym: String,
    name: String,
    timezone: String,
    bic: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameDay {
    date: String,
    core_start: String,
    core_end: String,
    pre_start: Option<String>,
    pre_end: Option<String>,
    post_start: Option<String>,
    post_end: Option<String>,
    lunch_start: Option<String>,
    lunch_end: Option<String>,
    settlement_date: Option<String>,
}
fn interval(
    start: Option<String>,
    end: Option<String>,
) -> Result<Option<(OffsetDateTime, OffsetDateTime)>> {
    match (start, end) {
        (None, None) => Ok(None),
        (Some(start), Some(end)) => {
            let start = utc(&start)?;
            let end = utc(&end)?;
            if start >= end {
                return Err(invalid());
            }
            Ok(Some((start, end)))
        }
        _ => Err(invalid()),
    }
}
fn parse(body: &[u8], start: Date, end: Date) -> Result<Vec<Day>> {
    if body.len() > 256 * 1024 {
        return Err(invalid());
    }
    let frame: Frame = serde_json::from_slice(body).map_err(|_| invalid())?;
    if frame.market.mic.as_deref() != Some("XNAS")
        || frame.market.acronym != "NASDAQ"
        || frame.market.timezone != "America/New_York"
        || !crate::valid_bounded_text(&frame.market.name, 160)
        || frame
            .market
            .bic
            .as_ref()
            .is_some_and(|value| !crate::valid_bounded_text(value, 11))
        || frame.calendar.is_empty()
        || frame.calendar.len() > 16
    {
        return Err(invalid());
    }
    let mut days = Vec::new();
    let mut last = None;
    for day in frame.calendar {
        let day_date = date(&day.date)?;
        let core_start = utc(&day.core_start)?;
        let core_end = utc(&day.core_end)?;
        let pre = interval(day.pre_start, day.pre_end)?;
        let post = interval(day.post_start, day.post_end)?;
        let lunch = interval(day.lunch_start, day.lunch_end)?;
        if day_date < start
            || day_date > end
            || last.is_some_and(|date| date >= day_date)
            || core_start >= core_end
            || core_start.date() != day_date
            || core_end.date() != day_date
            || pre.is_some_and(|(start, end)| start.date() != day_date || end != core_start)
            || post
                .is_some_and(|(start, end)| start != core_end || end - start > Duration::hours(12))
            || lunch.is_some()
            || day
                .settlement_date
                .as_ref()
                .is_some_and(|value| date(value).is_err())
        {
            return Err(invalid());
        }
        last = Some(day_date);
        days.push(Day {
            date: day_date,
            core_start,
            core_end,
            pre,
            post,
        });
    }
    Ok(days)
}
/// Fixed read-only calendar destination; renderer never supplies a path or host.
pub(crate) fn allowed_path(path: &str) -> bool {
    let Some(query) = path.strip_prefix("/v3/calendar/XNAS?start=") else {
        return false;
    };
    let Some((start, rest)) = query.split_once("&end=") else {
        return false;
    };
    let Some(end) = rest.strip_suffix("&timezone=UTC") else {
        return false;
    };
    let (Ok(start), Ok(end)) = (date(start), date(end)) else {
        return false;
    };
    end - start == Duration::days(15)
}

pub fn execute_refresh(
    control: &Arc<Mutex<ControlPlane>>,
    request: &Value,
    consumer: &str,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    let prepare = (|| {
        if !provider_order_consumer_allowed(consumer) {
            return Err(TradeXError::new("IPC_ACCESS_DENIED"));
        }
        let envelope: CommandEnvelope = payload(request.clone())?;
        if envelope.schema_version != 1 {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        if envelope.command != "data.calendar.refresh" {
            return Err(TradeXError::new("IPC_COMMAND_UNKNOWN"));
        }
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
        let saved = store.calendar_source()?;
        if saved.version(&input.workspace_id) != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let account_id = saved
            .connection_id
            .ok_or_else(|| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
        let account = store.account(&account_id)?;
        if !eligible(&account) {
            return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
        }
        let today = timestamp(&now.wall_clock)?.date();
        let start = today - Duration::days(1);
        let end = today + Duration::days(14);
        control.calendar_request_sequence = control
            .calendar_request_sequence
            .checked_add(1)
            .filter(|value| *value <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        let binding = Binding {
            workspace: input.workspace_id,
            source_version: input.expected_state_version,
            account: account_id,
            account_version: account.state_version.clone(),
            reference: account.credential_ref(),
            session: control.session.clone(),
            clock_generation: control.time.generation().into(),
            sequence: control.calendar_request_sequence,
        };
        control.calendar_observation = None;
        control.calendar_failure = None;
        Ok((binding, start, end))
    })();
    let (binding, start, end) = match prepare {
        Ok(value) => value,
        Err(error) => return failure_reply(request_id(request), error),
    };
    let current = || {
        control
            .lock()
            .is_ok_and(|control| binding.current(&control))
    };
    let read = (|| {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let credentials = vault.get(&binding.reference)?;
        let headers = crate::quote_source::source_headers(&credentials)?;
        let http = crate::provider_io::p3_provider_http(http, &current, &binding.reference);
        let body = http.get(
            ProviderEndpoint::AlpacaPaper,
            &format!("/v3/calendar/XNAS?start={start}&end={end}&timezone=UTC"),
            headers,
        )?;
        let received = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
            .time
            .status(&binding.workspace)?;
        let days = parse(&body, start, end)?;
        let now = timestamp(&received.wall_clock)?;
        if received.confidence != TimeConfidence::Trusted
            || !days.iter().any(|day| day.core_end > now)
        {
            return Err(invalid());
        }
        use sha2::{Digest, Sha256};
        let schedule=days.iter().map(|day| {
            let fmt=|value:OffsetDateTime|value.format(&Rfc3339).map_err(|_|invalid());
            let pair=|value:Option<(OffsetDateTime,OffsetDateTime)>|value.map(|(start,end)|Ok::<_,TradeXError>((fmt(start)?,fmt(end)?))).transpose();
            Ok::<_,TradeXError>(json!({"date":day.date.to_string(),"coreStart":fmt(day.core_start)?,"coreEnd":fmt(day.core_end)?,"pre":pair(day.pre)?,"post":pair(day.post)?}))
        }).collect::<Result<Vec<_>>>()?;
        let version = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(
                    binding.source_version.as_str(),
                    binding.account.as_str(),
                    binding.account_version.as_str(),
                    binding.session.as_str(),
                    binding.clock_generation.as_str(),
                    start.to_string(),
                    end.to_string(),
                    schedule
                ))
                .map_err(|_| invalid())?
            )
        );
        Ok(CalendarObservation {
            binding: binding.clone(),
            received,
            start,
            end,
            version,
            days,
        })
    })();
    let finish = (|| {
        let mut control = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if !binding.current(&control) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        match read {
            Ok(observation) => control.calendar_observation = Some(observation),
            Err(error) => control.calendar_failure = Some(error.code),
        }
        connection(&mut control, &binding.workspace)
    })();
    match finish {
        Ok(source) => success_reply(request_id(request), json!(source), None),
        Err(error) => failure_reply(request_id(request), error),
    }
}

pub(crate) fn project(control: &ControlPlane, now: &TimeStatus, detail: &mut MarketDetail) {
    if detail.instrument.exchange.as_deref() != Some("XNAS")
        || !matches!(detail.instrument.symbol.as_str(), "AAPL" | "MSFT")
    {
        return;
    }
    let configured = control
        .store
        .as_ref()
        .and_then(|store| store.calendar_source().ok())
        .is_some_and(|source| source.connection_id.is_some());
    if !configured {
        return;
    }
    let Some(observation) = control.calendar_observation.as_ref() else {
        if control.calendar_failure.is_some() {
            detail.market_state.source_status = MarketDataStatus::Unavailable;
            detail.market_state.session = MarketSession::Unknown;
            detail.market_state.reason="The last selected XNAS calendar refresh failed. No prior calendar remains eligible; refresh after resolving the source error.".into();
        }
        return;
    };
    let state = &mut detail.market_state;
    state.source_id = Some("OD-005".into());
    state.observed_at = observation.received.wall_clock.clone();
    state.calendar_version = Some(observation.version.clone());
    state.provider_time = None;
    state.time_confidence = now.confidence;
    if !observation.fresh(control, now) {
        state.session = MarketSession::Unknown;
        state.source_status = MarketDataStatus::Unavailable;
        state.reason="Calendar evidence is stale or its source/account/clock binding changed. Refresh before execution.".into();
        return;
    }
    let Ok(current) = timestamp(&now.wall_clock) else {
        return;
    };
    state.session = MarketSession::Closed;
    state.source_status = MarketDataStatus::Available;
    for day in &observation.days {
        if current >= day.core_start && current < day.core_end {
            state.session = MarketSession::Open;
        } else if day
            .pre
            .is_some_and(|(start, end)| current >= start && current < end)
            || day
                .post
                .is_some_and(|(start, end)| current >= start && current < end)
        {
            state.session = MarketSession::ExtendedHours;
        }
    }
    state.next_open = observation
        .days
        .iter()
        .find(|day| day.core_start > current)
        .and_then(|day| day.core_start.format(&Rfc3339).ok());
    state.next_close = observation
        .days
        .iter()
        .find(|day| day.core_end > current)
        .and_then(|day| day.core_end.format(&Rfc3339).ok());
    state.reason="Authenticated XNAS UTC schedule; calendar only. Corporate actions, halts, tradability, adjustment and financial authority remain separate.".into();
}
pub(crate) fn catalog_entry(
    control: &ControlPlane,
    workspace: &str,
    mut source: DataSourceEntry,
) -> DataSourceEntry {
    if !control.require_workspace(workspace).is_ok() {
        return source;
    }
    if let Some(saved) = control
        .store
        .as_ref()
        .and_then(|store| store.calendar_source().ok())
        && saved.connection_id.is_some()
    {
        source.configured = true;
        source.status = DataSourceStatus::Unverified;
        source.checked_at = None;
        source.availability_reason="Calendar selection configured separately. Inspect its individual capability status; corporate actions, halts and historical adjustment remain unverified.".into();
        source.observed_at = control
            .calendar_observation
            .as_ref()
            .map(|observation| observation.received.wall_clock.clone());
    }
    source
}
