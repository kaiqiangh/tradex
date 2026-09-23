use crate::protocol::{
    LocalPaperBalance, LocalPaperEvent, LocalPaperEventKind, LocalPaperFill, LocalPaperMoney,
    LocalPaperOrder, LocalPaperOrderState, LocalPaperPosition, LocalPaperProfile, LocalPaperQuote,
    LocalPaperState, OrderProposal, OrderProposalStatus, OrderQuantityType, OrderSide, OrderType,
    PaperOrderResult, Result, TimeInForce, TradeXError,
};
use crate::providers::{
    AccountConnection, AccountData, AccountHealth, Balance, ConnectionState, PermissionReview,
};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

pub const PROVIDER_ID: &str = "local-paper";
pub const ENVIRONMENT: &str = "LOCAL";
pub const ACCOUNT_TYPE: &str = "TRADEX_SIMULATION";
pub const DEFAULT_STARTING_CASH: &str = "100000";
pub const QUOTE_SOURCE: &str = "LOCAL_DETERMINISTIC";
pub const SCENARIO_ID: &str = "default-v1";
pub const SCENARIO_PARTIAL: &str = "partial-v1";
pub const SCENARIO_RESTING: &str = "resting-v1";
pub const SCENARIO_REJECTED: &str = "rejected-v1";
pub const ENGINE_VERSION: &str = "s16-v1";
pub const SCENARIO_SEED: &str = "s16-default";
pub const SCENARIO_VERSION: &str = "s16-v1";
pub const FEE_POLICY: &str = "ZERO";
pub const SLIPPAGE_POLICY: &str = "NONE";
pub const FILL_POLICY: &str = "BOUNDED_V1";
pub const DISCLOSURE: &str =
    "TRADEX_SIMULATION; TradeX simulation; not provider truth; not Live execution.";

pub fn scenario_supported(value: &str) -> bool {
    matches!(
        value,
        SCENARIO_ID | SCENARIO_PARTIAL | SCENARIO_RESTING | SCENARIO_REJECTED
    )
}

pub fn account(workspace_id: &str, base_currency: &str) -> Result<AccountConnection> {
    if workspace_id.is_empty()
        || workspace_id.len() > 128
        || workspace_id.chars().any(char::is_control)
        || !valid_currency(base_currency)
    {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let now = crate::storage::timestamp()?;
    let account_id = format!("{PROVIDER_ID}:{workspace_id}");
    Ok(AccountConnection {
        connection_id: account_id,
        workspace_id: workspace_id.into(),
        provider_id: PROVIDER_ID.into(),
        environment: ENVIRONMENT.into(),
        label: "Local Paper · LOCAL PAPER · TradeX simulation".into(),
        created_at: now.clone(),
        updated_at: now.clone(),
        state_version: String::new(),
        connection_state: ConnectionState::Connected,
        health: AccountHealth {
            connection: "ONLINE".into(),
            authentication: "NOT_REQUIRED".into(),
            credential: "NOT_REQUIRED".into(),
            private_stream: "NOT_CONFIGURED".into(),
            reconciliation: "LOCAL_PROJECTION".into(),
            execution_eligibility: "SIMULATION_ONLY".into(),
            arming: "NOT_APPLICABLE".into(),
            reason: DISCLOSURE.into(),
        },
        permissions: PermissionReview {
            scope: "VERIFIED".into(),
            detected: vec!["tradex.simulation".into()],
            forbidden: vec![],
            unsupported: vec![],
            acknowledged: true,
            ip_allow_list_status: "NOT_APPLICABLE".into(),
            ip_allow_list: None,
        },
        data: Some(AccountData {
            remote_account_id: format!("tradex-simulation:{workspace_id}"),
            account_type: ACCOUNT_TYPE.into(),
            currency: Some(base_currency.into()),
            buying_power: None,
            balances: vec![Balance {
                asset: base_currency.into(),
                available: DEFAULT_STARTING_CASH.into(),
                total: Some(DEFAULT_STARTING_CASH.into()),
                reserved: Some("0".into()),
                in_pies: None,
                locked: None,
                restricted_available: None,
            }],
            positions: vec![],
            open_orders: vec![],
            capabilities: vec!["simulation.execute".into(), "portfolio.read".into()],
            limitations: vec![
                DISCLOSURE.into(),
                "No external provider or network I/O.".into(),
            ],
        }),
        last_successful_sync: Some(now),
        last_private_stream_event_at: None,
    })
}

pub fn initial_state(
    workspace_id: &str,
    account_id: &str,
    account_label: &str,
    base_currency: &str,
    updated_at: String,
) -> Result<LocalPaperState> {
    if workspace_id.is_empty()
        || account_id.is_empty()
        || account_label.is_empty()
        || !valid_currency(base_currency)
    {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let money = |value: &str| LocalPaperMoney {
        value: value.into(),
        currency: base_currency.into(),
    };
    Ok(LocalPaperState {
        workspace_id: workspace_id.into(),
        account_id: account_id.into(),
        provider_id: PROVIDER_ID.into(),
        environment: ENVIRONMENT.into(),
        account_label: account_label.into(),
        profile: LocalPaperProfile {
            base_currency: base_currency.into(),
            starting_cash: DEFAULT_STARTING_CASH.into(),
            quote_source: QUOTE_SOURCE.into(),
            scenario_id: SCENARIO_ID.into(),
            engine_version: ENGINE_VERSION.into(),
            scenario_seed: SCENARIO_SEED.into(),
            scenario_version: SCENARIO_VERSION.into(),
            fee_policy: FEE_POLICY.into(),
            slippage_policy: SLIPPAGE_POLICY.into(),
            fill_policy: FILL_POLICY.into(),
            quote_price: Some("100".into()),
            quote_freshness: Some("FRESH".into()),
            quote_observed_at: Some(updated_at.clone()),
        },
        cash: money(DEFAULT_STARTING_CASH),
        reserved_cash: money("0"),
        equity: money(DEFAULT_STARTING_CASH),
        realized_pnl: money("0"),
        unrealized_pnl: money("0"),
        exposure: money("0"),
        balances: vec![LocalPaperBalance {
            asset: base_currency.into(),
            available: DEFAULT_STARTING_CASH.into(),
            total: DEFAULT_STARTING_CASH.into(),
            reserved: "0".into(),
        }],
        orders: vec![],
        positions: vec![],
        open_orders: vec![],
        fills: vec![],
        events: vec![],
        event_cursor: 0,
        state_version: format!("paper:{workspace_id}:0"),
        updated_at,
        disclosure: DISCLOSURE.into(),
    })
}

pub fn validate_profile(
    profile: &LocalPaperProfile,
    base_currency: &str,
    starting_cash: &str,
) -> Result<()> {
    if profile.base_currency != base_currency
        || profile.starting_cash != starting_cash
        || profile.quote_source != QUOTE_SOURCE
        || profile.engine_version != ENGINE_VERSION
        || profile.scenario_seed != SCENARIO_SEED
        || profile.scenario_version != SCENARIO_VERSION
        || profile.fee_policy != FEE_POLICY
        || profile.slippage_policy != SLIPPAGE_POLICY
        || profile.fill_policy != FILL_POLICY
        || !scenario_supported(&profile.scenario_id)
        || profile.quote_freshness.as_deref() != Some("FRESH")
        || !profile
            .quote_price
            .as_deref()
            .is_some_and(|value| normalize_positive(value).is_ok())
        || profile.quote_observed_at.as_deref().is_none_or(|value| {
            time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
                .is_err()
        })
    {
        return Err(TradeXError::new("PAPER_SCENARIO_INVALID"));
    }
    Ok(())
}

pub fn submit(
    state: &mut LocalPaperState,
    proposal: &OrderProposal,
    idempotency_key: &str,
    proposal_state_version: &str,
    now: String,
) -> Result<PaperOrderResult> {
    if proposal.status != OrderProposalStatus::NeedsApproval
        || proposal.workspace_id != state.workspace_id
        || proposal.fields.account_id.as_deref() != Some(state.account_id.as_str())
        || proposal.fields.environment != crate::protocol::ExecutionContext::LocalPaper
        || proposal.fields.venue != "TRADEX_SIM"
        || idempotency_key.is_empty()
        || idempotency_key.len() > 128
        || idempotency_key.chars().any(char::is_control)
    {
        return Err(TradeXError::new("PAPER_PROPOSAL_INVALID"));
    }
    if let Some(existing) = state
        .orders
        .iter()
        .find(|order| order.proposal_id == proposal.proposal_id)
        .cloned()
    {
        if existing.idempotency_key.as_deref() == Some(idempotency_key) {
            return result_for_order(state, &existing, proposal_state_version);
        }
        return Err(TradeXError::new("PAPER_PROPOSAL_CONSUMED"));
    }
    if state
        .orders
        .iter()
        .any(|order| order.idempotency_key.as_deref() == Some(idempotency_key))
    {
        return Err(TradeXError::new("PAPER_IDEMPOTENCY_CONFLICT"));
    }

    let quote = deterministic_quote(state, &proposal.fields.instrument_id, &now)?;
    let requested_quantity = match proposal.fields.quantity.r#type {
        OrderQuantityType::Base => normalize_positive(&proposal.fields.quantity.value)?,
        OrderQuantityType::Quote => {
            let value = normalize_positive(&proposal.fields.quantity.value)?;
            let quantity = crate::portfolio::decimal_div(&value, &quote.price)?;
            if quantity == "0" || crate::portfolio::decimal_mul(&quantity, &quote.price)? != value {
                return Err(TradeXError::new("PAPER_QUANTITY_UNREPRESENTABLE"));
            }
            quantity
        }
    };
    let requested_value = crate::portfolio::decimal_mul(&requested_quantity, &quote.price)?;
    if let Some(maximum_spend) = proposal.fields.maximum_spend.as_deref()
        && compare_nonnegative(&requested_value, maximum_spend)? == Ordering::Greater
    {
        return Err(TradeXError::new("PAPER_MAXIMUM_SPEND_EXCEEDED"));
    }
    if matches!(proposal.fields.side, OrderSide::Sell) {
        ensure_sell_capacity(state, &proposal.fields.instrument_id, &requested_quantity)?;
    }

    let crossing = if matches!(proposal.fields.order_type, OrderType::Limit) {
        let limit = proposal
            .fields
            .limit_price
            .as_deref()
            .ok_or_else(|| TradeXError::new("PAPER_PROPOSAL_INVALID"))?;
        match proposal.fields.side {
            OrderSide::Buy => compare_nonnegative(limit, &quote.price)? != Ordering::Less,
            OrderSide::Sell => compare_nonnegative(limit, &quote.price)? != Ordering::Greater,
        }
    } else {
        true
    };
    let scenario = state.profile.scenario_id.as_str();
    if !scenario_supported(scenario) {
        return Err(TradeXError::new("PAPER_SCENARIO_INVALID"));
    }
    let force_reject = scenario == SCENARIO_REJECTED
        || (!crossing
            && matches!(
                proposal.fields.time_in_force,
                TimeInForce::Ioc | TimeInForce::Fok
            ));
    let resting = scenario == SCENARIO_RESTING || !crossing;
    let requested_fill = if force_reject || resting {
        "0".into()
    } else if scenario == SCENARIO_PARTIAL {
        crate::portfolio::decimal_div(&requested_quantity, "2")?
    } else {
        requested_quantity.clone()
    };
    if scenario == SCENARIO_PARTIAL && requested_fill == "0" {
        return Err(TradeXError::new("PAPER_QUANTITY_UNREPRESENTABLE"));
    }
    let has_fill = requested_fill != "0";
    if matches!(proposal.fields.time_in_force, TimeInForce::Fok)
        && requested_fill != requested_quantity
    {
        return build_order_result(
            state,
            proposal,
            idempotency_key,
            OrderOutcome {
                quote: &quote,
                requested_quantity: &requested_quantity,
                remaining_quantity: "0",
                fill: None,
                state: LocalPaperOrderState::Rejected,
            },
            proposal_state_version,
            now,
        );
    }

    if has_fill {
        let value = crate::portfolio::decimal_mul(&requested_fill, &quote.price)?;
        apply_fill(
            state,
            &proposal.fields.side,
            &proposal.fields.instrument_id,
            &requested_fill,
            &quote.price,
            &value,
        )?;
    }
    let remaining =
        crate::portfolio::decimal_add(&requested_quantity, &format!("-{requested_fill}"))?;
    let final_state = if force_reject {
        LocalPaperOrderState::Rejected
    } else if !has_fill {
        LocalPaperOrderState::Accepted
    } else if remaining == "0" {
        LocalPaperOrderState::Filled
    } else if matches!(proposal.fields.time_in_force, TimeInForce::Ioc) {
        LocalPaperOrderState::Cancelled
    } else {
        LocalPaperOrderState::PartiallyFilled
    };
    if matches!(
        final_state,
        LocalPaperOrderState::Accepted | LocalPaperOrderState::PartiallyFilled
    ) && matches!(proposal.fields.side, OrderSide::Buy)
    {
        let reservation_price = proposal
            .fields
            .limit_price
            .as_deref()
            .unwrap_or(&quote.price);
        let reserve = crate::portfolio::decimal_mul(&remaining, reservation_price)?;
        reserve_cash(state, &reserve)?;
    }
    let fill = if has_fill {
        let value = crate::portfolio::decimal_mul(&requested_fill, &quote.price)?;
        Some(LocalPaperFill {
            fill_id: format!("paper-fill:{}", uuid::Uuid::new_v4()),
            order_id: String::new(),
            instrument_id: proposal.fields.instrument_id.clone(),
            side: proposal.fields.side,
            quantity: requested_fill.clone(),
            price: quote.price.clone(),
            value,
            currency: quote.currency.clone(),
            observed_at: now.clone(),
        })
    } else {
        None
    };
    build_order_result(
        state,
        proposal,
        idempotency_key,
        OrderOutcome {
            quote: &quote,
            requested_quantity: &requested_quantity,
            remaining_quantity: &remaining,
            fill,
            state: final_state,
        },
        proposal_state_version,
        now,
    )
}

pub fn cancel(
    state: &mut LocalPaperState,
    order_id: &str,
    idempotency_key: &str,
    proposal_state_version: &str,
    now: String,
) -> Result<PaperOrderResult> {
    if order_id.is_empty()
        || order_id.len() > 128
        || order_id.chars().any(char::is_control)
        || idempotency_key.is_empty()
        || idempotency_key.len() > 128
        || idempotency_key.chars().any(char::is_control)
    {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    if state.orders.iter().any(|order| {
        order.order_id != order_id
            && order.cancel_idempotency_key.as_deref() == Some(idempotency_key)
    }) {
        return Err(TradeXError::new("PAPER_IDEMPOTENCY_CONFLICT"));
    }
    let index = state
        .orders
        .iter()
        .position(|order| order.order_id == order_id)
        .ok_or_else(|| TradeXError::new("PAPER_ORDER_NOT_FOUND"))?;
    let mut order = state.orders[index].clone();
    if order.cancel_idempotency_key.as_deref() == Some(idempotency_key) {
        return result_for_order(state, &order, proposal_state_version);
    }
    if !matches!(
        order.state,
        LocalPaperOrderState::Accepted
            | LocalPaperOrderState::PartiallyFilled
            | LocalPaperOrderState::CancelPending
    ) {
        return Err(TradeXError::new("PAPER_ORDER_NOT_CANCELLABLE"));
    }
    if matches!(
        order.state,
        LocalPaperOrderState::Accepted | LocalPaperOrderState::PartiallyFilled
    ) && matches!(order.side, OrderSide::Buy)
    {
        let reserve_price = order
            .limit_price
            .as_deref()
            .or_else(|| order.quote.as_ref().map(|quote| quote.price.as_str()))
            .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        let release = crate::portfolio::decimal_mul(&order.remaining_quantity, reserve_price)?;
        release_cash_reservation(state, &release)?;
    }
    order.cancel_idempotency_key = Some(idempotency_key.into());
    order.state = LocalPaperOrderState::Cancelled;
    order.updated_at = now.clone();
    append_event(
        state,
        LocalPaperEventKind::Cancelled,
        order.order_id.clone(),
        None,
        now,
    )?;
    order.event_sequence = Some(state.event_cursor);
    state.orders[index] = order.clone();
    state
        .open_orders
        .retain(|candidate| candidate.order_id != order_id);
    state.updated_at = order.updated_at.clone();
    result_for_order(state, &order, proposal_state_version)
}

pub fn set_scenario(
    state: &mut LocalPaperState,
    profile: LocalPaperProfile,
    now: String,
) -> Result<()> {
    validate_profile(
        &profile,
        &state.profile.base_currency,
        &state.profile.starting_cash,
    )?;
    if !state.open_orders.is_empty() {
        return Err(TradeXError::new("PAPER_SCENARIO_ORDER_OPEN"));
    }
    state.profile = profile;
    state.updated_at = now.clone();
    append_event(
        state,
        LocalPaperEventKind::ScenarioChanged,
        format!("paper-scenario:{}", state.workspace_id),
        None,
        now,
    )?;
    Ok(())
}

pub fn refresh_quote(state: &mut LocalPaperState, now: String) -> Result<()> {
    if !state.open_orders.is_empty() {
        return Err(TradeXError::new("PAPER_SCENARIO_ORDER_OPEN"));
    }
    state.profile.quote_freshness = Some("FRESH".into());
    state.profile.quote_observed_at = Some(now.clone());
    state.updated_at = now.clone();
    append_event(
        state,
        LocalPaperEventKind::QuoteRefreshed,
        format!("paper-quote-refresh:{}", state.workspace_id),
        None,
        now,
    )?;
    Ok(())
}

struct OrderOutcome<'a> {
    quote: &'a LocalPaperQuote,
    requested_quantity: &'a str,
    remaining_quantity: &'a str,
    fill: Option<LocalPaperFill>,
    state: LocalPaperOrderState,
}

fn build_order_result(
    state: &mut LocalPaperState,
    proposal: &OrderProposal,
    idempotency_key: &str,
    outcome: OrderOutcome<'_>,
    proposal_state_version: &str,
    now: String,
) -> Result<PaperOrderResult> {
    let order_id = format!("paper-order:{}", uuid::Uuid::new_v4());
    let mut fill = outcome.fill;
    if let Some(fill) = fill.as_mut() {
        fill.order_id = order_id.clone();
        fill.fill_id = format!("paper-fill:{}", uuid::Uuid::new_v4());
    }
    let fill_sequence = fill.as_ref().map(|_| LocalPaperEventKind::PartiallyFilled);
    append_event(
        state,
        LocalPaperEventKind::Accepted,
        order_id.clone(),
        None,
        now.clone(),
    )?;
    if let Some(kind) = fill_sequence {
        append_event(
            state,
            if matches!(outcome.state, LocalPaperOrderState::Filled) {
                LocalPaperEventKind::Filled
            } else {
                kind
            },
            order_id.clone(),
            fill.as_ref().map(|fill| fill.fill_id.clone()),
            now.clone(),
        )?;
    } else if matches!(outcome.state, LocalPaperOrderState::Rejected) {
        append_event(
            state,
            LocalPaperEventKind::Rejected,
            order_id.clone(),
            None,
            now.clone(),
        )?;
    }
    if matches!(outcome.state, LocalPaperOrderState::Cancelled) {
        append_event(
            state,
            LocalPaperEventKind::Cancelled,
            order_id.clone(),
            fill.as_ref().map(|fill| fill.fill_id.clone()),
            now.clone(),
        )?;
    }
    let average_fill_price = fill.as_ref().map(|fill| fill.price.clone());
    let order = LocalPaperOrder {
        order_id: order_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_hash: proposal.proposal_hash.clone(),
        instrument_id: proposal.fields.instrument_id.clone(),
        side: proposal.fields.side,
        order_type: proposal.fields.order_type,
        state: outcome.state,
        requested_quantity: outcome.requested_quantity.into(),
        filled_quantity: fill
            .as_ref()
            .map(|fill| fill.quantity.clone())
            .unwrap_or_else(|| "0".into()),
        remaining_quantity: outcome.remaining_quantity.into(),
        quantity_type: Some(proposal.fields.quantity.r#type),
        time_in_force: Some(proposal.fields.time_in_force),
        limit_price: proposal.fields.limit_price.clone(),
        average_fill_price,
        quote: Some(outcome.quote.clone()),
        idempotency_key: Some(idempotency_key.into()),
        cancel_idempotency_key: None,
        event_sequence: Some(state.event_cursor),
        created_at: now.clone(),
        updated_at: now,
    };
    if let Some(fill) = fill.clone() {
        state.fills.push(fill);
    }
    state.orders.push(order.clone());
    state.updated_at = order.updated_at.clone();
    state.open_orders = state
        .orders
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.state,
                LocalPaperOrderState::Accepted
                    | LocalPaperOrderState::PartiallyFilled
                    | LocalPaperOrderState::CancelPending
            )
        })
        .cloned()
        .collect();
    result_for_order(state, &order, proposal_state_version)
}

pub(crate) fn result_for_order(
    state: &LocalPaperState,
    order: &LocalPaperOrder,
    proposal_state_version: &str,
) -> Result<PaperOrderResult> {
    let quote = order
        .quote
        .clone()
        .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    let fill = state
        .fills
        .iter()
        .find(|fill| fill.order_id == order.order_id)
        .cloned();
    Ok(PaperOrderResult {
        workspace_id: state.workspace_id.clone(),
        account_id: state.account_id.clone(),
        environment: ENVIRONMENT.into(),
        proposal_id: order.proposal_id.clone(),
        proposal_hash: order.proposal_hash.clone(),
        order: order.clone(),
        fill,
        quote,
        proposal_state_version: proposal_state_version.into(),
        state_version: state.state_version.clone(),
        event_sequence: order.event_sequence.unwrap_or(state.event_cursor),
        disclosure: state.disclosure.clone(),
        paper_state: Box::new(state.clone()),
    })
}

fn append_event(
    state: &mut LocalPaperState,
    kind: LocalPaperEventKind,
    order_id: String,
    fill_id: Option<String>,
    occurred_at: String,
) -> Result<()> {
    let sequence = state
        .event_cursor
        .checked_add(1)
        .filter(|sequence| *sequence <= crate::protocol::MAX_SEQUENCE)
        .ok_or_else(|| TradeXError::new("PAPER_STATE_LIMIT"))?;
    state.event_cursor = sequence;
    state.state_version = format!("paper:{}:{sequence}", state.workspace_id);
    state.events.push(LocalPaperEvent {
        event_id: format!("paper-event:{}", uuid::Uuid::new_v4()),
        sequence,
        kind,
        order_id,
        fill_id,
        occurred_at,
        state_version: state.state_version.clone(),
    });
    Ok(())
}

fn ensure_sell_capacity(
    state: &LocalPaperState,
    instrument_id: &str,
    quantity: &str,
) -> Result<()> {
    let position = state
        .positions
        .iter()
        .find(|position| position.instrument_id == instrument_id)
        .ok_or_else(|| TradeXError::new("PAPER_INSUFFICIENT_POSITION"))?;
    if compare_nonnegative(&position.quantity, quantity)? == Ordering::Less {
        return Err(TradeXError::new("PAPER_INSUFFICIENT_POSITION"));
    }
    Ok(())
}

fn reserve_cash(state: &mut LocalPaperState, amount: &str) -> Result<()> {
    if compare_nonnegative(&state.cash.value, amount)? == Ordering::Less {
        return Err(TradeXError::new("PAPER_INSUFFICIENT_CASH"));
    }
    state.cash.value = crate::portfolio::decimal_add(&state.cash.value, &format!("-{amount}"))?;
    state.reserved_cash.value = crate::portfolio::decimal_add(&state.reserved_cash.value, amount)?;
    sync_cash_projection(state)
}

fn release_cash_reservation(state: &mut LocalPaperState, amount: &str) -> Result<()> {
    if compare_nonnegative(&state.reserved_cash.value, amount)? == Ordering::Less {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    state.reserved_cash.value =
        crate::portfolio::decimal_add(&state.reserved_cash.value, &format!("-{amount}"))?;
    state.cash.value = crate::portfolio::decimal_add(&state.cash.value, amount)?;
    sync_cash_projection(state)
}

fn sync_cash_projection(state: &mut LocalPaperState) -> Result<()> {
    state.equity.value = crate::portfolio::decimal_add(
        &crate::portfolio::decimal_add(&state.cash.value, &state.reserved_cash.value)?,
        &state.exposure.value,
    )?;
    if let Some(balance) = state
        .balances
        .iter_mut()
        .find(|balance| balance.asset == state.cash.currency)
    {
        balance.available = state.cash.value.clone();
        balance.total =
            crate::portfolio::decimal_add(&state.cash.value, &state.reserved_cash.value)?;
        balance.reserved = state.reserved_cash.value.clone();
    }
    Ok(())
}

fn deterministic_quote(
    state: &LocalPaperState,
    instrument_id: &str,
    observed_at: &str,
) -> Result<LocalPaperQuote> {
    let instrument_currency = crate::market::instruments()
        .into_iter()
        .find(|instrument| instrument.instrument_id == instrument_id)
        .map(|instrument| instrument.currency)
        .ok_or_else(|| TradeXError::new("PAPER_QUOTE_UNAVAILABLE"))?;
    let quote_observed_at = state
        .profile
        .quote_observed_at
        .as_deref()
        .ok_or_else(|| TradeXError::new("PAPER_QUOTE_UNAVAILABLE"))?;
    let observed =
        time::OffsetDateTime::parse(observed_at, &time::format_description::well_known::Rfc3339)
            .map_err(|_| TradeXError::new("PAPER_QUOTE_UNAVAILABLE"))?;
    let quote_time = time::OffsetDateTime::parse(
        quote_observed_at,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| TradeXError::new("PAPER_QUOTE_UNAVAILABLE"))?;
    let age = observed - quote_time;
    if instrument_id.is_empty()
        || instrument_id.len() > 128
        || instrument_id.chars().any(char::is_control)
        || state.profile.quote_source != QUOTE_SOURCE
        || state.profile.quote_freshness.as_deref() != Some("FRESH")
        || instrument_currency != state.profile.base_currency
        || age.is_negative()
        || age > time::Duration::seconds(300)
    {
        return Err(TradeXError::new("PAPER_QUOTE_UNAVAILABLE"));
    }
    let price = state
        .profile
        .quote_price
        .as_deref()
        .ok_or_else(|| TradeXError::new("PAPER_QUOTE_UNAVAILABLE"))
        .and_then(normalize_positive)
        .map_err(|_| TradeXError::new("PAPER_QUOTE_UNAVAILABLE"))?;
    let mut hasher = Sha256::new();
    hasher.update(state.profile.scenario_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(state.profile.scenario_seed.as_bytes());
    hasher.update(b"\0");
    hasher.update(state.profile.scenario_version.as_bytes());
    hasher.update(b"\0");
    hasher.update(state.profile.fee_policy.as_bytes());
    hasher.update(b"\0");
    hasher.update(state.profile.slippage_policy.as_bytes());
    hasher.update(b"\0");
    hasher.update(state.profile.fill_policy.as_bytes());
    hasher.update(b"\0");
    hasher.update(instrument_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(price.as_bytes());
    hasher.update(b"\0");
    hasher.update(quote_observed_at.as_bytes());
    let quote_id = format!("quote:sha256:{}", hex::encode(hasher.finalize()));
    Ok(LocalPaperQuote {
        quote_id,
        instrument_id: instrument_id.into(),
        price,
        currency: state.profile.base_currency.clone(),
        observed_at: quote_observed_at.into(),
        scenario_id: state.profile.scenario_id.clone(),
        source: state.profile.quote_source.clone(),
        freshness: state.profile.quote_freshness.clone().unwrap_or_default(),
    })
}

fn apply_fill(
    state: &mut LocalPaperState,
    side: &OrderSide,
    instrument_id: &str,
    quantity: &str,
    price: &str,
    value: &str,
) -> Result<()> {
    match side {
        OrderSide::Buy => {
            if compare_nonnegative(&state.cash.value, value)? == Ordering::Less {
                return Err(TradeXError::new("PAPER_INSUFFICIENT_CASH"));
            }
            state.cash.value =
                crate::portfolio::decimal_add(&state.cash.value, &format!("-{value}"))?;
        }
        OrderSide::Sell => {
            let index = state
                .positions
                .iter()
                .position(|position| position.instrument_id == instrument_id)
                .ok_or_else(|| TradeXError::new("PAPER_INSUFFICIENT_POSITION"))?;
            let position = &state.positions[index];
            if compare_nonnegative(&position.quantity, quantity)? == Ordering::Less {
                return Err(TradeXError::new("PAPER_INSUFFICIENT_POSITION"));
            }
            let realized = crate::portfolio::decimal_mul(
                &crate::portfolio::decimal_add(
                    price,
                    &format!("-{}", position.average_entry_price),
                )?,
                quantity,
            )?;
            state.realized_pnl.value =
                crate::portfolio::decimal_add(&state.realized_pnl.value, &realized)?;
            state.cash.value = crate::portfolio::decimal_add(&state.cash.value, value)?;
        }
    }

    let existing_index = state
        .positions
        .iter()
        .position(|position| position.instrument_id == instrument_id);
    match side {
        OrderSide::Buy => {
            if let Some(index) = existing_index {
                let position = &mut state.positions[index];
                let old_cost = crate::portfolio::decimal_mul(
                    &position.quantity,
                    &position.average_entry_price,
                )?;
                let new_cost = crate::portfolio::decimal_add(&old_cost, value)?;
                position.quantity = crate::portfolio::decimal_add(&position.quantity, quantity)?;
                position.average_entry_price =
                    crate::portfolio::decimal_div(&new_cost, &position.quantity)?;
            } else {
                state.positions.push(LocalPaperPosition {
                    instrument_id: instrument_id.into(),
                    quantity: quantity.into(),
                    average_entry_price: price.into(),
                    market_value: None,
                    currency: state.profile.base_currency.clone(),
                    unrealized_pnl: None,
                });
            }
        }
        OrderSide::Sell => {
            let index =
                existing_index.ok_or_else(|| TradeXError::new("PAPER_INSUFFICIENT_POSITION"))?;
            let position = &mut state.positions[index];
            position.quantity =
                crate::portfolio::decimal_add(&position.quantity, &format!("-{quantity}"))?;
            if position.quantity == "0" {
                state.positions.remove(index);
            }
        }
    }

    let mut exposure = "0".to_owned();
    let mut unrealized = "0".to_owned();
    for position in &mut state.positions {
        position.market_value = Some(crate::portfolio::decimal_mul(&position.quantity, price)?);
        position.unrealized_pnl = Some(crate::portfolio::decimal_mul(
            &crate::portfolio::decimal_add(price, &format!("-{}", position.average_entry_price))?,
            &position.quantity,
        )?);
        exposure = crate::portfolio::decimal_add(
            &exposure,
            position.market_value.as_deref().unwrap_or("0"),
        )?;
        unrealized = crate::portfolio::decimal_add(
            &unrealized,
            position.unrealized_pnl.as_deref().unwrap_or("0"),
        )?;
    }
    state.exposure.value = exposure;
    state.unrealized_pnl.value = unrealized;
    sync_cash_projection(state)
}

fn normalize_positive(value: &str) -> Result<String> {
    let normalized = crate::provider_io::decimal(&serde_json::Value::String(value.into()))
        .map_err(|_| TradeXError::new("PAPER_QUOTE_UNAVAILABLE"))?;
    if normalized == "0" || normalized.starts_with('-') {
        return Err(TradeXError::new("PAPER_QUOTE_UNAVAILABLE"));
    }
    Ok(normalized)
}

fn compare_nonnegative(left: &str, right: &str) -> Result<Ordering> {
    let difference = crate::portfolio::decimal_add(left, &format!("-{right}"))?;
    Ok(if difference == "0" {
        Ordering::Equal
    } else if difference.starts_with('-') {
        Ordering::Less
    } else {
        Ordering::Greater
    })
}

fn valid_currency(currency: &str) -> bool {
    currency.len() == 3 && currency.bytes().all(|byte| byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_account_and_state_are_explicit_simulation() {
        let account = account("workspace", "USD").unwrap();
        assert_eq!(account.provider_id, PROVIDER_ID);
        assert_eq!(account.environment, ENVIRONMENT);
        assert_eq!(account.health.execution_eligibility, "SIMULATION_ONLY");
        assert_eq!(account.data.unwrap().account_type, ACCOUNT_TYPE);

        let state = initial_state(
            "workspace",
            "local-paper:workspace",
            "Local Paper · TradeX simulation",
            "USD",
            "2026-09-21T00:00:00Z".into(),
        )
        .unwrap();
        assert_eq!(state.cash.value, DEFAULT_STARTING_CASH);
        assert_eq!(state.positions.len(), 0);
        assert!(state.disclosure.contains("not provider truth"));
        assert_eq!(state.profile.quote_price.as_deref(), Some("100"));
        assert_eq!(state.profile.quote_freshness.as_deref(), Some("FRESH"));
        assert_eq!(state.profile.scenario_seed, SCENARIO_SEED);
        assert_eq!(state.profile.scenario_version, SCENARIO_VERSION);
        assert_eq!(state.profile.fee_policy, FEE_POLICY);
        assert_eq!(state.profile.slippage_policy, SLIPPAGE_POLICY);
        assert_eq!(state.profile.fill_policy, FILL_POLICY);
    }

    #[test]
    fn quote_requires_fresh_bounded_profile_input() {
        let mut state = initial_state(
            "workspace:quote",
            "local-paper:quote",
            "Local Paper · TradeX simulation",
            "USD",
            "2026-01-01T00:00:00Z".into(),
        )
        .unwrap();
        assert!(deterministic_quote(&state, "equity:US:AAPL", "2026-01-01T00:00:00Z").is_ok());
        state.profile.quote_freshness = Some("STALE".into());
        assert_eq!(
            deterministic_quote(&state, "equity:US:AAPL", "2026-01-01T00:00:00Z")
                .unwrap_err()
                .code,
            "PAPER_QUOTE_UNAVAILABLE"
        );
        state.profile.quote_freshness = Some("FRESH".into());
        state.profile.quote_price = None;
        assert_eq!(
            deterministic_quote(&state, "equity:US:AAPL", "2026-01-01T00:00:00Z")
                .unwrap_err()
                .code,
            "PAPER_QUOTE_UNAVAILABLE"
        );
        state.profile.quote_price = Some("100".into());
        state.profile.quote_observed_at = Some("2025-12-31T23:00:00Z".into());
        assert_eq!(
            deterministic_quote(&state, "equity:US:AAPL", "2026-01-01T00:00:00Z")
                .unwrap_err()
                .code,
            "PAPER_QUOTE_UNAVAILABLE"
        );
        state.profile.quote_observed_at = Some("2026-01-01T00:00:00Z".into());
        assert_eq!(
            deterministic_quote(&state, "crypto:BTC/USDT:spot", "2026-01-01T00:00:00Z")
                .unwrap_err()
                .code,
            "PAPER_QUOTE_UNAVAILABLE"
        );
        assert_eq!(
            deterministic_quote(&state, "equity:US:AAPL", "2026-01-01T00:05:00.001Z")
                .unwrap_err()
                .code,
            "PAPER_QUOTE_UNAVAILABLE"
        );
        state.profile.fill_policy = "UNBOUNDED".into();
        assert_eq!(
            validate_profile(&state.profile, "USD", DEFAULT_STARTING_CASH)
                .unwrap_err()
                .code,
            "PAPER_SCENARIO_INVALID"
        );
    }
}
