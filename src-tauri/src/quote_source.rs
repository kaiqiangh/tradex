pub mod hot;
pub(crate) mod transport;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    ControlPlane, failure_reply, payload,
    protocol::{
        AlpacaFeed, CommandEnvelope, DataSourceConfigure, DataSourceCredential, DataSourceMutation,
        DataSourceQuery, MAX_SEQUENCE, Result, TradeXError,
    },
    provider_io::{
        CredentialVault, Credentials, ProviderEndpoint, ProviderHttp, ProviderHttpMethod,
    },
    provider_order_consumer_allowed,
    providers::ConnectionState,
    request_id, success_reply,
};

fn feed_name(feed: &AlpacaFeed) -> &'static str {
    match feed {
        AlpacaFeed::Iex => "iex",
        AlpacaFeed::Sip => "sip",
        AlpacaFeed::DelayedSip => "delayed_sip",
    }
}

fn describe_feed(source: &mut crate::protocol::DataSourceEntry, feed: &AlpacaFeed) {
    source.configured = true;
    source.coverage = match feed {
        AlpacaFeed::Iex => "IEX: single-venue US equity quotes; not consolidated US coverage.",
        AlpacaFeed::Sip => "SIP: consolidated US equity quotes; listing and bid/ask venues remain distinct.",
        AlpacaFeed::DelayedSip => "Delayed SIP: consolidated US equity quotes with a 15 minute delay; not execution-ready.",
    }.into();
    source.latency = match feed {
        AlpacaFeed::DelayedSip => "15 minute delayed quotes.",
        _ => "Realtime feed; actual provider timestamps and freshness still require validation.",
    }
    .into();
}

pub(crate) fn configured_entry(
    control: &ControlPlane,
    workspace_id: &str,
) -> Result<crate::protocol::DataSourceEntry> {
    control.require_workspace(workspace_id)?;
    let store = control.store.as_ref().unwrap();
    let saved = store.quote_source()?;
    let mut source = crate::data_sources::entries()
        .into_iter()
        .find(|source| source.source_id == "OD-001")
        .unwrap();
    let Some(feed) = saved.feed.as_ref() else {
        return Ok(source);
    };
    describe_feed(&mut source, feed);
    source.status = crate::protocol::DataSourceStatus::Unverified;
    source.availability_reason = format!(
        "Selected {} feed is configured; technical access has not been verified.",
        feed_name(feed)
    );
    let previous = control
        .data_source_observations
        .get(&(workspace_id.to_owned(), "OD-001".into()));
    if let Some(id) = saved.account_id()
        && !store.account(id).is_ok_and(|account| {
            account.provider_id == "alpaca"
                && account.connection_state == ConnectionState::Connected
                && matches!(
                    account.health.credential.as_str(),
                    "CONFIGURED" | "UNCHECKED"
                )
        })
    {
        source.status = crate::protocol::DataSourceStatus::Unavailable;
        source.availability_reason="The reused Alpaca account key is unavailable. Reconnect that account or select another data credential.".into();
        return Ok(crate::merge_data_source_observation(previous, source));
    }
    let access_current = control
        .quote_source_access_binding
        .as_ref()
        .is_some_and(|binding| binding.current_source(control));
    Ok(if access_current {
        previous.cloned().unwrap_or(source)
    } else {
        source
    })
}

pub(crate) fn allowed_latest_path(path: &str) -> bool {
    if matches!(
        path,
        "/v2/stocks/meta/exchanges"
            | "/v2/stocks/meta/conditions/quote?tape=A"
            | "/v2/stocks/meta/conditions/quote?tape=B"
            | "/v2/stocks/meta/conditions/quote?tape=C"
    ) {
        return true;
    }
    let Some(parameters) = path.strip_prefix("/v2/stocks/quotes/latest?symbols=") else {
        return false;
    };
    let Some((symbol, remainder)) = parameters.split_once("&feed=") else {
        return false;
    };
    let Some(feed) = remainder.strip_suffix("&currency=USD") else {
        return false;
    };
    matches!(feed, "iex" | "sip" | "delayed_sip")
        && !symbol.is_empty()
        && symbol.len() <= 32
        && symbol.as_bytes()[0].is_ascii_uppercase()
        && symbol
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || matches!(c, b'.' | b'-'))
}

#[derive(Clone)]
pub(crate) struct SourceReadBinding {
    workspace_id: String,
    source_version: String,
    session: String,
    epoch: String,
    probe_sequence: u64,
    connection_generation: String,
    saved: SavedQuoteSource,
    reference: Option<String>,
    account_version: Option<(String, String)>,
}

impl SourceReadBinding {
    fn capture(control: &mut ControlPlane, workspace_id: &str) -> Result<Self> {
        control.require_workspace(workspace_id)?;
        let store = control.store.as_ref().unwrap();
        let saved = store.quote_source()?;
        let mut account_version = None;
        let reference = match &saved.credential {
            Some(SavedSourceCredential::Dedicated { reference }) => Some(reference.clone()),
            Some(SavedSourceCredential::ExistingAccount { connection_id }) => {
                let account = store.account(connection_id)?;
                if account.provider_id != "alpaca"
                    || account.connection_state != ConnectionState::Connected
                    || !matches!(
                        account.health.credential.as_str(),
                        "CONFIGURED" | "UNCHECKED"
                    )
                {
                    return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
                }
                account_version = Some((connection_id.clone(), account.state_version.clone()));
                Some(account.credential_ref())
            }
            None => None,
        };
        if control
            .quote_source_access_binding
            .as_ref()
            .is_some_and(|binding| !binding.current_source(control))
        {
            control.quote_connection_generation = uuid::Uuid::new_v4().to_string();
        }
        control.quote_source_probe_sequence = control
            .quote_source_probe_sequence
            .checked_add(1)
            .filter(|sequence| *sequence <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("PROVIDER_BACKPRESSURE"))?;
        Ok(Self {
            workspace_id: workspace_id.into(),
            source_version: saved.state_version(workspace_id),
            session: control.session.clone(),
            epoch: control.quote_source_epoch.clone(),
            probe_sequence: control.quote_source_probe_sequence,
            connection_generation: control.quote_connection_generation.clone(),
            saved,
            reference,
            account_version,
        })
    }

    fn current(&self, control: &ControlPlane) -> bool {
        self.probe_sequence == control.quote_source_probe_sequence
            && self.connection_generation == control.quote_connection_generation
            && self.current_source(control)
    }

    fn current_source(&self, control: &ControlPlane) -> bool {
        if self.session != control.session
            || self.epoch != control.quote_source_epoch
            || control.require_workspace(&self.workspace_id).is_err()
        {
            return false;
        }
        let store = control.store.as_ref().unwrap();
        if !store
            .quote_source()
            .is_ok_and(|source| source.state_version(&self.workspace_id) == self.source_version)
        {
            return false;
        }
        self.account_version.as_ref().is_none_or(|(id, version)| {
            store.account(id).is_ok_and(|account| {
                &account.state_version == version
                    && account.connection_state == ConnectionState::Connected
                    && matches!(
                        account.health.credential.as_str(),
                        "CONFIGURED" | "UNCHECKED"
                    )
            })
        })
    }
}

/// Authenticated read-only verification of the explicit source, never an account permission attestation.
pub fn execute_probe(
    control: &Arc<Mutex<ControlPlane>>,
    request: &Value,
    consumer: &str,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    if !provider_order_consumer_allowed(consumer) {
        return failure_reply(request_id(request), TradeXError::new("IPC_ACCESS_DENIED"));
    }
    let prepared = (|| {
        let mut control = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        let job = control
            .prepare_data_source_probe(request)?
            .ok_or_else(|| TradeXError::new("IPC_COMMAND_UNKNOWN"))?;
        let binding = if job.input.source_id == "OD-001" {
            Some(SourceReadBinding::capture(
                &mut control,
                &job.input.workspace_id,
            )?)
        } else {
            None
        };
        Ok((job, binding))
    })();
    let (job, binding) = match prepared {
        Ok(value) => value,
        Err(error) => return failure_reply(request_id(request), error),
    };
    let outcome = if let Some(binding) = &binding
        && binding.reference.is_some()
    {
        let current = || {
            control
                .lock()
                .is_ok_and(|control| binding.current(&control))
        };
        probe_selected(binding, job.source.clone(), vault, http, &current)
    } else {
        crate::data_sources::probe(&job.input.source_id, job.source.clone())
    };
    match control.lock() {
        Ok(mut control) => {
            if binding
                .as_ref()
                .is_some_and(|binding| !binding.current(&control))
            {
                return failure_reply(
                    request_id(request),
                    TradeXError::new("STATE_VERSION_CONFLICT"),
                );
            }
            if outcome.as_ref().is_ok_and(|source| {
                source.source_id == "OD-001"
                    && source.status != crate::protocol::DataSourceStatus::Available
            }) {
                invalidate_quotes(&mut control, "DATA_SOURCE_READ_FAILED");
            }
            let previous_access = control.quote_source_access_binding.clone();
            if let Some(binding) = binding {
                control.quote_source_access_binding = Some(binding);
            }
            let result = control.complete_data_source_probe(&job, outcome);
            if result["ok"] != true {
                control.quote_source_access_binding = previous_access;
            }
            result
        }
        Err(_) => failure_reply(
            request_id(request),
            TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"),
        ),
    }
}

fn probe_selected(
    binding: &SourceReadBinding,
    mut source: crate::protocol::DataSourceEntry,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
    current: &impl Fn() -> bool,
) -> Result<crate::protocol::DataSourceEntry> {
    let feed = binding
        .saved
        .feed
        .as_ref()
        .ok_or_else(|| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
    let name = feed_name(feed);
    describe_feed(&mut source, feed);
    let read = (|| {
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let credentials = vault.get(binding.reference.as_deref().unwrap())?;
        let headers = source_headers(&credentials)?;
        let http = crate::provider_io::p3_provider_http(
            http,
            current,
            binding.reference.as_deref().unwrap(),
        );
        let response = http.request(
            ProviderEndpoint::AlpacaMarketData,
            ProviderHttpMethod::Get,
            &format!("/v2/stocks/quotes/latest?symbols=AAPL&feed={name}&currency=USD"),
            headers,
            None,
        )?;
        match response.status {
            200 => (),
            401 => return Err(TradeXError::new("PROVIDER_AUTH_FAILED")),
            403 => return Err(TradeXError::new("DATA_SOURCE_FEED_DENIED")),
            429 => return Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
            _ => return Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
        }
        validate_latest_response(&response.body, "AAPL")?;
        if !current() {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        Ok(())
    })();
    let checked = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| TradeXError::new("DATA_SOURCE_PROBE_FAILED"))?;
    source.checked_at = Some(checked.clone());
    match read {
        Ok(()) => {
            source.status = crate::protocol::DataSourceStatus::Available;
            source.observed_at = Some(checked.clone());
            source.verified_at = Some(checked);
            source.availability_reason = format!(
                "Technical access to the selected {name} feed verified by an authenticated quote read. Quote freshness, coverage, licensing and every financial guard still apply."
            );
        }
        Err(error) => {
            source.status = crate::protocol::DataSourceStatus::Unavailable;
            source.observed_at = None;
            source.verified_at = None;
            source.availability_reason = format!(
                "Selected {name} feed could not be verified ({}). No feed fallback or response-body disclosure occurred.",
                error.code
            );
        }
    }
    Ok(source)
}

#[derive(Clone, Serialize, PartialEq, Eq)]
struct ParsedQuote {
    timestamp: String,
    bid: String,
    ask: String,
    bid_size: String,
    ask_size: String,
    bid_exchange: String,
    ask_exchange: String,
    tape: String,
    conditions: Vec<String>,
}

pub(crate) struct AcceptedQuote {
    binding: SourceReadBinding,
    snapshot: crate::protocol::MarketSnapshot,
    failure: Option<String>,
}

pub(crate) fn source_headers(credentials: &Credentials) -> Result<HeaderMap> {
    let secrets = credentials.values()?;
    if secrets.len() != 2 {
        return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
    }
    let mut headers = HeaderMap::new();
    for (header, value) in [
        ("APCA-API-KEY-ID", &secrets[0]),
        ("APCA-API-SECRET-KEY", &secrets[1]),
    ] {
        let mut value =
            HeaderValue::from_str(value).map_err(|_| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
        value.set_sensitive(true);
        headers.insert(header, value);
    }
    Ok(headers)
}

fn read_data(http: &impl ProviderHttp, path: &str, headers: HeaderMap) -> Result<Vec<u8>> {
    let response = http.request(
        ProviderEndpoint::AlpacaMarketData,
        ProviderHttpMethod::Get,
        path,
        headers,
        None,
    )?;
    match response.status {
        200 if response.body.len() <= 2 * 1024 * 1024 => Ok(response.body),
        401 => Err(TradeXError::new("PROVIDER_AUTH_FAILED")),
        403 => Err(TradeXError::new("DATA_SOURCE_FEED_DENIED")),
        429 => Err(TradeXError::new("PROVIDER_RATE_LIMITED")),
        200 => Err(TradeXError::new("PROVIDER_RESPONSE_INVALID")),
        _ => Err(TradeXError::new("PROVIDER_UNAVAILABLE")),
    }
}

fn metadata(body: &[u8], secrets: &[String]) -> Result<std::collections::HashMap<String, String>> {
    let invalid = || TradeXError::new("PROVIDER_RESPONSE_INVALID");
    if body.len() > 65_536 {
        return Err(invalid());
    }
    let value: Value = serde_json::from_slice(body).map_err(|_| invalid())?;
    let object = value
        .as_object()
        .filter(|v| !v.is_empty() && v.len() <= 128)
        .ok_or_else(invalid)?;
    object
        .iter()
        .map(|(code, name)| {
            let name = name.as_str().ok_or_else(invalid)?;
            if code.is_empty()
                || code.len() > 4
                || !code.bytes().all(|b| b.is_ascii_graphic())
                || !crate::valid_bounded_text(name, 128)
                || secrets
                    .iter()
                    .any(|secret| name.contains(secret) || code.contains(secret))
            {
                return Err(invalid());
            }
            Ok((code.clone(), name.into()))
        })
        .collect()
}

/// Native/public read orchestration. Only the fixed external data/vault boundaries run outside the lock.
pub fn execute_market(
    control: &Arc<Mutex<ControlPlane>>,
    request: &Value,
    consumer: &str,
    vault: &impl CredentialVault,
    http: &impl ProviderHttp,
) -> Value {
    use crate::protocol::{MarketGetQuery, MarketTier};
    let prepared = (|| {
        if !provider_order_consumer_allowed(consumer) {
            return Err(TradeXError::new("IPC_ACCESS_DENIED"));
        }
        let envelope: CommandEnvelope = payload(request.clone())?;
        if envelope.schema_version != 1 {
            return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
        }
        if envelope.command != "market.get" {
            return Err(TradeXError::new("IPC_COMMAND_UNKNOWN"));
        }
        if !crate::valid_bounded_text(&envelope.request_id, 128) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let input: MarketGetQuery = payload(envelope.payload)?;
        let mut locked = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        // Validate canonical identity/tier before any credential or network read.
        let detail = locked.market_detail_for(
            &input.workspace_id,
            &input.instrument_id,
            &input.tier,
            false,
        )?;
        let binding = if detail.source_id.as_deref() == Some("OD-001")
            && input.tier != MarketTier::Cold
            && locked
                .store
                .as_ref()
                .unwrap()
                .quote_source()?
                .feed
                .is_some()
        {
            Some(SourceReadBinding::capture(
                &mut locked,
                &input.workspace_id,
            )?)
        } else {
            None
        };
        Ok((input, detail.instrument, binding))
    })();
    let (input, instrument, binding) = match prepared {
        Ok(value) => value,
        Err(error) => return failure_reply(request_id(request), error),
    };
    let Some(binding) = binding else {
        return control
            .lock()
            .map(|mut control| control.dispatch(request.clone()))
            .unwrap_or_else(|_| {
                failure_reply(
                    request_id(request),
                    TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"),
                )
            });
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
        let credentials = vault.get(
            binding
                .reference
                .as_deref()
                .ok_or_else(|| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?,
        )?;
        let headers = source_headers(&credentials)?;
        let mapping = instrument
            .providers
            .iter()
            .find(|mapping| mapping.provider_id == "alpaca")
            .ok_or_else(|| TradeXError::new("MARKET_INSTRUMENT_NOT_FOUND"))?;
        let feed = binding.saved.feed.as_ref().unwrap();
        let http = crate::provider_io::p3_provider_http(
            http,
            &current,
            binding.reference.as_deref().unwrap(),
        );
        let body = read_data(
            &http,
            &format!(
                "/v2/stocks/quotes/latest?symbols={}&feed={}&currency=USD",
                mapping.provider_symbol,
                feed_name(feed)
            ),
            headers.clone(),
        )?;
        let received = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?
            .time
            .status(&input.workspace_id)?;
        let quote = parse_latest_response(&body, &mapping.provider_symbol)?;
        if instrument.exchange.as_deref() != Some("XNAS") || quote.tape != "C" {
            return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
        }
        let conditions = metadata(
            &read_data(
                &http,
                &format!("/v2/stocks/meta/conditions/quote?tape={}", quote.tape),
                headers.clone(),
            )?,
            &credentials.values()?,
        )?;
        let exchanges = metadata(
            &read_data(&http, "/v2/stocks/meta/exchanges", headers)?,
            &credentials.values()?,
        )?;
        let evidence = quote_evidence(&binding, &instrument, &quote, &conditions, &exchanges)?;
        Ok((quote, evidence, received))
    })();
    let result = (|| {
        let mut control = control
            .lock()
            .map_err(|_| TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"))?;
        if !binding.current(&control) {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        let now = control.time.status(&input.workspace_id)?;
        let accepted = read.and_then(|(quote, evidence, received)| {
            accept_snapshot(
                &control,
                &binding,
                &input.instrument_id,
                quote,
                evidence,
                &received,
            )
        });
        let mut source = configured_entry(&control, &input.workspace_id)?;
        source.checked_at = Some(now.wall_clock.clone());
        control.quote_source_access_binding = Some(binding.clone());
        match accepted {
            Ok(snapshot) => {
                control.quote_observations.insert(
                    input.instrument_id.clone(),
                    AcceptedQuote {
                        binding,
                        snapshot,
                        failure: None,
                    },
                );
                source.status = crate::protocol::DataSourceStatus::Available;
                source.observed_at = Some(now.wall_clock.clone());
                source.verified_at = Some(now.wall_clock);
                source.availability_reason="Selected feed technical access verified by an authenticated quote and metadata read; freshness and every financial guard remain separate.".into();
            }
            Err(error) => {
                invalidate_quotes(&mut control, &error.code);
                source.status = crate::protocol::DataSourceStatus::Unavailable;
                source.availability_reason = format!(
                    "Selected feed quote read failed ({}). No feed fallback occurred.",
                    error.code
                );
                source.verified_at = None;
            }
        }
        control
            .data_source_observations
            .insert((input.workspace_id.clone(), "OD-001".into()), source);
        control.market_detail_for(
            &input.workspace_id,
            &input.instrument_id,
            &input.tier,
            false,
        )
    })();
    match result {
        Ok(detail) => success_reply(request_id(request), json!(detail), None),
        Err(error) => failure_reply(request_id(request), error),
    }
}

fn invalidate_quotes(control: &mut ControlPlane, code: &str) {
    control.quote_connection_generation = uuid::Uuid::new_v4().to_string();
    for previous in control.quote_observations.values_mut() {
        previous.failure = Some(code.into());
    }
}

pub(crate) fn project_quote(
    control: &ControlPlane,
    now: &crate::protocol::TimeStatus,
    detail: &mut crate::protocol::MarketDetail,
) {
    use crate::protocol::{MarketDataStatus, MarketFreshness, TimeConfidence};
    let Some(accepted) = control
        .quote_observations
        .get(&detail.instrument.instrument_id)
        .filter(|quote| quote.binding.current_source(control))
    else {
        detail.status = MarketDataStatus::Unavailable;
        let source_reason = configured_entry(control, &detail.workspace_id)
            .ok()
            .filter(|source| source.status != crate::protocol::DataSourceStatus::Available)
            .map(|source| source.availability_reason);
        detail.availability_reason = if detail.tier == crate::protocol::MarketTier::Hot {
            control
                .hot_quote
                .as_ref()
                .and_then(|lease| {
                    lease.missing_quote_reason(&detail.instrument.instrument_id)
                })
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    source_reason.unwrap_or_else(|| "Selected Alpaca quote source is configured; acquire a Hot subscription and wait for an actual provider quote.".into())
                })
        } else {
            source_reason.unwrap_or_else(|| "Selected Alpaca quote source is configured; no validated quote observation is available. Refresh the selected feed.".into())
        };
        return;
    };
    let mut snapshot = accepted.snapshot.clone();
    let parse = |timestamp: &str| OffsetDateTime::parse(timestamp, &Rfc3339).ok();
    let fresh = match (
        parse(&now.wall_clock),
        parse(&snapshot.provenance.provider_timestamp),
        parse(&snapshot.provenance.received_timestamp),
    ) {
        (Some(now), Some(provider), Some(received)) => [now - provider, now - received]
            .iter()
            .all(|age| *age >= time::Duration::ZERO && *age <= time::Duration::seconds(30)),
        _ => false,
    };
    let source_ok = configured_entry(control, &detail.workspace_id)
        .is_ok_and(|source| source.status == crate::protocol::DataSourceStatus::Available);
    let same_connection = snapshot.provenance.alpaca.as_ref().is_some_and(|evidence| {
        evidence.connection_generation == control.quote_connection_generation
    });
    snapshot.provenance.freshness = if accepted.failure.is_some() || !source_ok || !same_connection
    {
        MarketFreshness::Stale
    } else if now.confidence != TimeConfidence::Trusted {
        MarketFreshness::ClockUncertain
    } else if fresh && source_ok && accepted.failure.is_none() {
        MarketFreshness::Healthy
    } else {
        MarketFreshness::Stale
    };
    let regular = snapshot
        .provenance
        .alpaca
        .as_ref()
        .is_some_and(|evidence| evidence.regular_conditions);
    detail.status = if snapshot.provenance.freshness == MarketFreshness::Healthy && regular {
        MarketDataStatus::Available
    } else {
        MarketDataStatus::Unavailable
    };
    detail.availability_reason = if !regular {
        "Quote conditions are not supported as regular execution evidence.".into()
    } else if accepted.failure.is_some() || !source_ok {
        "Retained quote is stale after a source read failure. Refresh the selected source.".into()
    } else if snapshot.provenance.freshness == MarketFreshness::ClockUncertain {
        "Quote received; synchronize TimeService before freshness can be trusted.".into()
    } else if !fresh {
        "Retained quote is stale; refresh the selected feed. No current financial evidence is available.".into()
    } else {
        "Authenticated selected-feed quote available. Coverage, depth, calendar and all financial guards remain separate.".into()
    };
    detail.snapshot = Some(snapshot);
    if detail.tier == crate::protocol::MarketTier::Hot
        && !control.hot_quote.as_ref().is_some_and(|lease| {
            lease.streaming_for(
                &detail.instrument.instrument_id,
                &control.quote_connection_generation,
                control,
            )
        })
    {
        detail.status = MarketDataStatus::Unavailable;
        detail.availability_reason.push_str(
            " No active Hot lease supplied this snapshot; an on-demand read is not a subscription update.",
        );
    }
}

fn validate_latest_response(body: &[u8], symbol: &str) -> Result<()> {
    parse_latest_response(body, symbol).map(|_| ())
}

#[derive(Deserialize)]
struct QuotePriceTokens {
    bp: Box<serde_json::value::RawValue>,
    ap: Box<serde_json::value::RawValue>,
    bs: Box<serde_json::value::RawValue>,
    #[serde(rename = "as")]
    ask_size: Box<serde_json::value::RawValue>,
}

#[derive(Deserialize)]
struct LatestQuoteTokens {
    quotes: std::collections::HashMap<String, Box<serde_json::value::RawValue>>,
}

fn validate_quote_numeric_tokens(body: &[u8]) -> Result<()> {
    let invalid = || TradeXError::new("PROVIDER_RESPONSE_INVALID");
    let tokens: QuotePriceTokens = serde_json::from_slice(body).map_err(|_| invalid())?;
    for number in [&tokens.bp, &tokens.ap, &tokens.bs, &tokens.ask_size] {
        // arbitrary_precision can deserialize its private sentinel object as Number.
        // Validate the original JSON token before it can become monetary evidence.
        if !number
            .get()
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_digit() || *byte == b'-')
        {
            return Err(invalid());
        }
    }
    Ok(())
}

fn parse_latest_response(body: &[u8], symbol: &str) -> Result<ParsedQuote> {
    let invalid = || TradeXError::new("PROVIDER_RESPONSE_INVALID");
    if body.len() > 2 * 1024 * 1024 {
        return Err(invalid());
    }
    let tokens: LatestQuoteTokens = serde_json::from_slice(body).map_err(|_| invalid())?;
    let quote_tokens = tokens.quotes.get(symbol).ok_or_else(invalid)?;
    validate_quote_numeric_tokens(quote_tokens.get().as_bytes())?;
    let response: Value = serde_json::from_slice(body).map_err(|_| invalid())?;
    if !response
        .get("quotes")
        .and_then(Value::as_object)
        .is_some_and(|quotes| quotes.len() == 1)
    {
        return Err(invalid());
    }
    let quote = response
        .get("quotes")
        .and_then(|quotes| quotes.get(symbol))
        .and_then(Value::as_object)
        .ok_or_else(invalid)?;
    let timestamp = quote
        .get("t")
        .and_then(Value::as_str)
        .filter(|v| v.len() <= 64)
        .ok_or_else(invalid)?;
    OffsetDateTime::parse(timestamp, &Rfc3339).map_err(|_| invalid())?;
    let exact_decimal = |field: &str| {
        let number = quote
            .get(field)
            .and_then(Value::as_number)
            .ok_or_else(invalid)?
            .to_string();
        if number.len() > 64 {
            return Err(invalid());
        }
        let number = crate::provider_io::decimal(&Value::String(number))?;
        if number.starts_with('-') || ((field == "bp" || field == "ap") && number == "0") {
            return Err(invalid());
        }
        Ok(number)
    };
    let bid = exact_decimal("bp")?;
    let ask = exact_decimal("ap")?;
    let bid_size = exact_decimal("bs")?;
    let ask_size = exact_decimal("as")?;
    if crate::provider_io::decimal_cmp(&bid, &ask)?.is_gt() {
        return Err(invalid());
    }
    for field in ["bx", "ax"] {
        if !quote
            .get(field)
            .and_then(Value::as_str)
            .is_some_and(|s| s.len() == 1 && s.as_bytes()[0].is_ascii_uppercase())
        {
            return Err(invalid());
        }
    }
    if !quote
        .get("z")
        .and_then(Value::as_str)
        .is_some_and(|s| matches!(s, "A" | "B" | "C"))
    {
        return Err(invalid());
    }
    let conditions = quote
        .get("c")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?;
    if conditions.is_empty()
        || conditions.len() > 16
        || conditions.iter().any(|c| {
            !c.as_str().is_some_and(|s| {
                !s.is_empty() && s.len() <= 4 && s.bytes().all(|b| b.is_ascii_graphic())
            })
        })
    {
        return Err(invalid());
    }
    Ok(ParsedQuote {
        timestamp: timestamp.into(),
        bid,
        ask,
        bid_size,
        ask_size,
        bid_exchange: quote["bx"].as_str().unwrap().into(),
        ask_exchange: quote["ax"].as_str().unwrap().into(),
        tape: quote["z"].as_str().unwrap().into(),
        conditions: conditions
            .iter()
            .map(|c| c.as_str().unwrap().to_owned())
            .collect(),
    })
}

/// Durable selection only. Verification and quotes are never restored from this record.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SavedQuoteSource {
    pub generation: u64,
    pub feed: Option<AlpacaFeed>,
    pub credential: Option<SavedSourceCredential>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum SavedSourceCredential {
    ExistingAccount { connection_id: String },
    Dedicated { reference: String },
}

impl SavedQuoteSource {
    pub fn state_version(&self, workspace_id: &str) -> String {
        format!("data:{workspace_id}:{}", self.generation)
    }
    pub fn account_id(&self) -> Option<&str> {
        match &self.credential {
            Some(SavedSourceCredential::ExistingAccount { connection_id }) => Some(connection_id),
            _ => None,
        }
    }
    pub fn owned_reference(&self) -> Option<&str> {
        match &self.credential {
            Some(SavedSourceCredential::Dedicated { reference }) => Some(reference),
            _ => None,
        }
    }
    pub fn validate(&self, workspace_id: &str) -> Result<()> {
        if self.generation > MAX_SEQUENCE
            || self.feed.is_some() != self.credential.is_some()
            || self
                .account_id()
                .is_some_and(|id| !crate::valid_bounded_text(id, 128))
            || self
                .owned_reference()
                .is_some_and(|reference| !owned_reference_matches(reference, workspace_id))
        {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        Ok(())
    }
}

pub(crate) fn owned_reference_matches(reference: &str, workspace_id: &str) -> bool {
    let prefix = format!("{workspace_id}/market-data/OD-001/");
    reference
        .strip_prefix(&prefix)
        .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
}

struct ConfigurationJob {
    request_id: String,
    workspace_id: String,
    session: String,
    epoch: String,
    prior_version: String,
    saved: SavedQuoteSource,
    account_version: Option<(String, String)>,
    new_reference: Option<String>,
    changes_selection: bool,
}

fn prepare(control: &mut ControlPlane, value: &Value, consumer: &str) -> Result<ConfigurationJob> {
    if !provider_order_consumer_allowed(consumer) {
        return Err(TradeXError::new("IPC_ACCESS_DENIED"));
    }
    let request: CommandEnvelope = payload(value.clone())?;
    if request.schema_version != 1 {
        return Err(TradeXError::new("IPC_SCHEMA_UNSUPPORTED"));
    }
    if !crate::valid_bounded_text(&request.request_id, 128) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let (workspace_id, expected, selection) = match request.command.as_str() {
        "data.source.configure" => {
            let input: DataSourceConfigure = payload(request.payload)?;
            (
                input.workspace_id,
                Some(input.expected_state_version),
                Some((input.feed, input.credential)),
            )
        }
        "data.source.disconnect" => {
            let input: DataSourceMutation = payload(request.payload)?;
            (input.workspace_id, Some(input.expected_state_version), None)
        }
        "data.source.cleanup" => {
            let input: DataSourceQuery = payload(request.payload)?;
            (input.workspace_id, None, None)
        }
        _ => return Err(TradeXError::new("IPC_COMMAND_UNKNOWN")),
    };
    control.require_workspace(&workspace_id)?;
    let store = control.store.as_mut().unwrap();
    let mut saved = store.quote_source()?;
    let prior_version = saved.state_version(&workspace_id);
    if let Some(expected) = &expected {
        if !crate::valid_bounded_text(expected, 256) {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        if expected != &prior_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
    }
    let mut account_version = None;
    let mut new_reference = None;
    let changes_selection = request.command != "data.source.cleanup";
    if let Some((feed, credential)) = selection {
        saved.feed = Some(feed);
        saved.credential = Some(match credential {
            DataSourceCredential::ExistingAccount { connection_id } => {
                if !crate::valid_bounded_text(&connection_id, 128) {
                    return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
                }
                let account = store.account(&connection_id)?;
                if account.provider_id != "alpaca"
                    || account.connection_state != ConnectionState::Connected
                    || account.health.credential != "CONFIGURED"
                {
                    return Err(TradeXError::new("CREDENTIAL_UNAVAILABLE"));
                }
                account_version = Some((connection_id.clone(), account.state_version));
                SavedSourceCredential::ExistingAccount { connection_id }
            }
            DataSourceCredential::Dedicated {} => {
                let reference =
                    format!("{workspace_id}/market-data/OD-001/{}", uuid::Uuid::new_v4());
                store.register_quote_source_credential(&reference)?;
                new_reference = Some(reference.clone());
                SavedSourceCredential::Dedicated { reference }
            }
        });
    } else if changes_selection {
        saved.feed = None;
        saved.credential = None;
    }
    Ok(ConfigurationJob {
        request_id: request.request_id,
        workspace_id,
        session: control.session.clone(),
        epoch: control.quote_source_epoch.clone(),
        prior_version,
        saved,
        account_version,
        new_reference,
        changes_selection,
    })
}

fn complete(control: &mut ControlPlane, job: &ConfigurationJob, outcome: Result<()>) -> Value {
    let result = (|| {
        outcome?;
        if control.session != job.session || control.quote_source_epoch != job.epoch {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        control.require_workspace(&job.workspace_id)?;
        let store = control.store.as_mut().unwrap();
        if store.quote_source()?.state_version(&job.workspace_id) != job.prior_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if let Some((id, version)) = &job.account_version {
            let account = store.account(id)?;
            if &account.state_version != version
                || account.connection_state != ConnectionState::Connected
                || account.health.credential != "CONFIGURED"
            {
                return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
            }
        }
        if job.changes_selection {
            store.save_quote_source(job.saved.clone())?;
            control.quote_observations.clear();
            control.hot_quote = None;
            control.quote_source_access_binding = None;
            control.quote_source_epoch = uuid::Uuid::new_v4().to_string();
            control.quote_connection_generation = uuid::Uuid::new_v4().to_string();
            control
                .data_source_observations
                .remove(&(job.workspace_id.clone(), "OD-001".into()));
        }
        control.quote_source_connection(&job.workspace_id)
    })();
    match result {
        Ok(connection) => success_reply(
            job.request_id.clone(),
            json!(connection),
            Some(connection.state_version),
        ),
        Err(error) => failure_reply(job.request_id.clone(), error),
    }
}

/// Native transport entry point: all Keychain and secure-entry work happens outside the domain lock.
/// The renderer sends selection metadata only, never a secret or a vault reference.
pub fn execute_configuration(
    control: &Arc<Mutex<ControlPlane>>,
    request: &Value,
    consumer: &str,
    vault: &impl CredentialVault,
    capture: impl FnOnce() -> Result<Credentials>,
) -> Value {
    let job = match control.lock() {
        Ok(mut control) => match prepare(&mut control, request, consumer) {
            Ok(job) => job,
            Err(error) => return failure_reply(request_id(request), error),
        },
        Err(_) => {
            return failure_reply(
                request_id(request),
                TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"),
            );
        }
    };
    let outcome = if let Some(reference) = &job.new_reference {
        (|| {
            let credentials = capture()?;
            if credentials.values()?.len() != 2 {
                return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
            }
            vault.put(reference, &credentials)
        })()
    } else {
        Ok(())
    };
    let mut reply = match control.lock() {
        Ok(mut control) => {
            let reply = complete(&mut control, &job, outcome);
            if reply["ok"] != true
                && let Some(reference) = &job.new_reference
                && control.require_workspace(&job.workspace_id).is_ok()
            {
                let _ = control
                    .store
                    .as_mut()
                    .unwrap()
                    .queue_quote_source_credential_cleanup(reference);
            }
            reply
        }
        Err(_) => failure_reply(
            job.request_id.clone(),
            TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE"),
        ),
    };
    // A rejected job owns only its newly registered UUID. The former active/reused item is untouched.
    if reply["ok"] != true
        && let Some(reference) = &job.new_reference
    {
        let removed = vault.remove(reference).is_ok();
        if removed
            && let Ok(mut control) = control.lock()
            && control.require_workspace(&job.workspace_id).is_ok()
        {
            let _ = control
                .store
                .as_mut()
                .unwrap()
                .acknowledge_quote_source_cleanup(reference);
        }
    }
    let pending = control
        .lock()
        .ok()
        .and_then(|control| {
            if control.require_workspace(&job.workspace_id).is_err() {
                return None;
            }
            control
                .store
                .as_ref()
                .unwrap()
                .quote_source_pending_cleanup()
                .ok()
        })
        .unwrap_or_default();
    for reference in pending {
        if reply["ok"] != true && job.new_reference.as_deref() == Some(reference.as_str()) {
            continue;
        }
        if vault.remove(&reference).is_ok()
            && let Ok(mut control) = control.lock()
            && control.require_workspace(&job.workspace_id).is_ok()
        {
            let _ = control
                .store
                .as_mut()
                .unwrap()
                .acknowledge_quote_source_cleanup(&reference);
        }
    }
    if reply["ok"] == true
        && let Ok(control) = control.lock()
        && control.require_workspace(&job.workspace_id).is_ok()
        && let Ok(connection) = control.quote_source_connection(&job.workspace_id)
        && reply["data"]["stateVersion"] == connection.state_version
    {
        reply["data"] = json!(connection);
    }
    reply
}

/// Pure metadata callers can reuse an existing key when there is no owned Keychain work.
/// Native transport routes every mutation through execute_configuration instead.
pub(crate) fn metadata_only(
    control: &mut ControlPlane,
    request: CommandEnvelope,
    consumer: &str,
) -> Result<(Value, Option<String>)> {
    if !provider_order_consumer_allowed(consumer) {
        return Err(TradeXError::new("IPC_ACCESS_DENIED"));
    }
    let workspace_id = request
        .payload
        .get("workspaceId")
        .and_then(Value::as_str)
        .ok_or_else(|| TradeXError::new("IPC_PAYLOAD_INVALID"))?;
    control.require_workspace(workspace_id)?;
    let store = control.store.as_ref().unwrap();
    if request
        .payload
        .get("credential")
        .and_then(|c| c.get("kind"))
        .and_then(Value::as_str)
        == Some("DEDICATED")
        || store.quote_source()?.owned_reference().is_some()
        || !store.quote_source_pending_cleanup()?.is_empty()
    {
        return Err(TradeXError::new("PROVIDER_NATIVE_ENTRY_REQUIRED"));
    }
    let encoded = json!({"requestId":request.request_id,"schemaVersion":request.schema_version,"command":request.command,"payload":request.payload});
    let job = prepare(control, &encoded, consumer)?;
    let reply = complete(control, &job, Ok(()));
    if reply["ok"] != true {
        return Err(TradeXError::new(
            reply["error"]["code"]
                .as_str()
                .unwrap_or("IPC_PAYLOAD_INVALID"),
        ));
    }
    Ok((
        reply["data"].clone(),
        reply["stateVersion"].as_str().map(str::to_owned),
    ))
}

fn quote_evidence(
    binding: &SourceReadBinding,
    instrument: &crate::protocol::Instrument,
    quote: &ParsedQuote,
    conditions: &std::collections::HashMap<String, String>,
    exchanges: &std::collections::HashMap<String, String>,
) -> Result<crate::protocol::AlpacaQuoteEvidence> {
    let mapping = instrument
        .providers
        .iter()
        .find(|mapping| mapping.provider_id == "alpaca")
        .ok_or_else(|| TradeXError::new("MARKET_INSTRUMENT_NOT_FOUND"))?;
    let feed = binding
        .saved
        .feed
        .as_ref()
        .ok_or_else(|| TradeXError::new("CREDENTIAL_UNAVAILABLE"))?;
    let invalid = || TradeXError::new("PROVIDER_RESPONSE_INVALID");
    let names = quote
        .conditions
        .iter()
        .map(|code| {
            Ok(crate::protocol::QuoteCondition {
                code: code.clone(),
                name: conditions.get(code).ok_or_else(invalid)?.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let regular = names
        .iter()
        .all(|condition| condition.code == "R" && condition.name == "Regular Two Sided Open");
    let evidence = crate::protocol::AlpacaQuoteEvidence {
        feed: feed.clone(),
        coverage: if *feed == AlpacaFeed::Iex {
            crate::protocol::QuoteCoverage::Iex
        } else {
            crate::protocol::QuoteCoverage::UsSip
        },
        provider_symbol: mapping.provider_symbol.clone(),
        listing_venue: instrument.exchange.clone().ok_or_else(invalid)?,
        bid_exchange: quote.bid_exchange.clone(),
        ask_exchange: quote.ask_exchange.clone(),
        bid_exchange_name: exchanges
            .get(&quote.bid_exchange)
            .ok_or_else(invalid)?
            .clone(),
        ask_exchange_name: exchanges
            .get(&quote.ask_exchange)
            .ok_or_else(invalid)?
            .clone(),
        tape: quote.tape.clone(),
        conditions: names,
        regular_conditions: regular,
        depth_unit: if *feed != AlpacaFeed::Iex
            && OffsetDateTime::parse(&quote.timestamp, &Rfc3339).map_err(|_| invalid())?
                >= OffsetDateTime::parse("2025-11-03T00:00:00Z", &Rfc3339).unwrap()
            && regular
        {
            crate::protocol::QuoteDepthUnit::Base
        } else {
            crate::protocol::QuoteDepthUnit::Unavailable
        },
        source_version: binding.source_version.clone(),
        connection_generation: binding.connection_generation.clone(),
    };
    Ok(evidence)
}

fn accept_snapshot(
    control: &ControlPlane,
    binding: &SourceReadBinding,
    instrument_id: &str,
    quote: ParsedQuote,
    evidence: crate::protocol::AlpacaQuoteEvidence,
    receipt: &crate::protocol::TimeStatus,
) -> Result<crate::protocol::MarketSnapshot> {
    use crate::protocol::{
        MarketEntitlement, MarketFreshness, MarketSnapshot, MarketSnapshotProvenance,
    };
    use sha2::{Digest, Sha256};
    let provider_time = OffsetDateTime::parse(&quote.timestamp, &Rfc3339)
        .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
    let received = OffsetDateTime::parse(&receipt.wall_clock, &Rfc3339)
        .map_err(|_| TradeXError::new("CLOCK_SKEW"))?;
    if provider_time > received {
        return Err(TradeXError::new("PROVIDER_RESPONSE_INVALID"));
    }
    let identity = |received: &str| -> Result<String> {
        let fingerprint = serde_json::to_vec(&(
            &binding.workspace_id,
            instrument_id,
            &quote,
            &evidence,
            received,
        ))
        .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
        Ok(format!(
            "alpaca:{}",
            hex::encode(Sha256::digest(fingerprint))
        ))
    };
    if let Some(previous) = control
        .quote_observations
        .get(instrument_id)
        .filter(|quote| quote.binding.current_source(&control))
    {
        let previous_time =
            OffsetDateTime::parse(&previous.snapshot.provenance.provider_timestamp, &Rfc3339)
                .map_err(|_| TradeXError::new("PROVIDER_RESPONSE_INVALID"))?;
        let same_connection =
            previous
                .snapshot
                .provenance
                .alpaca
                .as_ref()
                .is_some_and(|evidence| {
                    evidence.connection_generation == binding.connection_generation
                });
        if provider_time < previous_time {
            return Err(TradeXError::new("MARKET_QUOTE_CONFLICT"));
        }
        if same_connection && provider_time == previous_time {
            if previous.snapshot.provenance.market_snapshot_id
                == identity(&previous.snapshot.provenance.received_timestamp)?
            {
                return Ok(previous.snapshot.clone());
            }
            return Err(TradeXError::new("MARKET_QUOTE_CONFLICT"));
        }
    }
    let snapshot_id = identity(&receipt.wall_clock)?;
    let depth = evidence.depth_unit == crate::protocol::QuoteDepthUnit::Base;
    Ok(MarketSnapshot {
        instrument_id: instrument_id.to_owned(),
        provenance: MarketSnapshotProvenance {
            market_snapshot_id: snapshot_id,
            source: "OD-001".into(),
            venue: Some(
                if evidence.coverage == crate::protocol::QuoteCoverage::UsSip {
                    "US_SIP"
                } else {
                    "IEX"
                }
                .into(),
            ),
            provider_timestamp: quote.timestamp,
            received_timestamp: receipt.wall_clock.clone(),
            entitlement: if evidence.feed == AlpacaFeed::DelayedSip {
                MarketEntitlement::Delayed
            } else {
                MarketEntitlement::Realtime
            },
            freshness: MarketFreshness::ClockUncertain,
            alpaca: Some(evidence),
            binance: None,
        },
        last_price: None,
        bid: Some(quote.bid),
        ask: Some(quote.ask),
        bid_size: depth.then_some(quote.bid_size),
        ask_size: depth.then_some(quote.ask_size),
    })
}
