use crate::protocol::{
    DataSourceEntry, DataSourceStatus, FxFreshness, FxProvenance, FxQuality, LocalPaperOrderState,
    LocalPaperState, OrderSide, PortfolioAccount, PortfolioFill, PortfolioHolding,
    PortfolioLiveRisk, PortfolioOrder, PortfolioSnapshot, PortfolioStatus, PortfolioTotals,
    PortfolioValue, Result, TimeStatus, TradeXError,
};
use crate::providers::{
    AccountConnection, AccountData, AccountHealth, Balance, ConnectionState, OpenOrder, Position,
};
use serde_json::Value;

const IDENTITY_SOURCE: &str = "IDENTITY";
const MAX_PORTFOLIO_ACCOUNTS: usize = 256;
const MAX_PORTFOLIO_ROWS: usize = 512;
const MAX_PORTFOLIO_FX_ROUTES: usize = 128;
const FIXTURE_SCENARIO_ID: &str = "portfolio-fixture-v1";

pub fn get(
    workspace_id: &str,
    base_currency: &str,
    accounts: &[AccountConnection],
    fx_source: Option<&DataSourceEntry>,
    time_status: &TimeStatus,
    fixture: bool,
) -> Result<PortfolioSnapshot> {
    get_with_paper_state(
        workspace_id,
        base_currency,
        accounts,
        fx_source,
        time_status,
        None,
        fixture,
    )
}

pub fn get_with_paper_state(
    workspace_id: &str,
    base_currency: &str,
    accounts: &[AccountConnection],
    fx_source: Option<&DataSourceEntry>,
    time_status: &TimeStatus,
    paper_state: Option<&LocalPaperState>,
    fixture: bool,
) -> Result<PortfolioSnapshot> {
    if !valid_workspace_id(workspace_id) || !valid_base_currency(base_currency) {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    if time_status.workspace_id != workspace_id {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    if fixture {
        return fixture_snapshot(workspace_id, base_currency, time_status);
    }
    actual_snapshot(
        workspace_id,
        base_currency,
        accounts,
        fx_source,
        time_status,
        paper_state,
    )
}

fn local_paper_data(account: &AccountConnection, state: &LocalPaperState) -> AccountData {
    let mut balances = state
        .balances
        .iter()
        .map(|balance| Balance {
            asset: balance.asset.clone(),
            available: balance.available.clone(),
            total: Some(balance.total.clone()),
            reserved: Some(balance.reserved.clone()),
            in_pies: None,
            locked: None,
            restricted_available: None,
        })
        .collect::<Vec<_>>();
    if let Some(balance) = balances
        .iter_mut()
        .find(|balance| balance.asset == state.cash.currency)
    {
        balance.available = state.cash.value.clone();
        balance.reserved = Some(state.reserved_cash.value.clone());
    } else {
        balances.push(Balance {
            asset: state.cash.currency.clone(),
            available: state.cash.value.clone(),
            total: Some(state.cash.value.clone()),
            reserved: Some(state.reserved_cash.value.clone()),
            in_pies: None,
            locked: None,
            restricted_available: None,
        });
    }
    AccountData {
        remote_account_id: account
            .data
            .as_ref()
            .map(|data| data.remote_account_id.clone())
            .unwrap_or_else(|| format!("tradex-simulation:{}", state.workspace_id)),
        account_type: "TRADEX_SIMULATION".into(),
        currency: Some(state.cash.currency.clone()),
        buying_power: None,
        balances,
        positions: state
            .positions
            .iter()
            .map(|position| Position {
                symbol: position.instrument_id.clone(),
                instrument_id: Some(position.instrument_id.clone()),
                quantity: position.quantity.clone(),
                market_value: position.market_value.clone(),
                average_entry_price: Some(position.average_entry_price.clone()),
                instrument_currency: Some(position.currency.clone()),
                market_value_currency: Some(position.currency.clone()),
            })
            .collect(),
        open_orders: state
            .open_orders
            .iter()
            .map(|order| OpenOrder {
                broker_order_id: order.order_id.clone(),
                symbol: order.instrument_id.clone(),
                instrument_id: Some(order.instrument_id.clone()),
                side: match order.side {
                    OrderSide::Buy => "BUY".into(),
                    OrderSide::Sell => "SELL".into(),
                },
                quantity: Some(order.remaining_quantity.clone()),
                notional: None,
                filled_quantity: Some(order.filled_quantity.clone()),
                filled_value: None,
                currency: Some(state.cash.currency.clone()),
                status: match order.state {
                    LocalPaperOrderState::Proposed => "PROPOSED",
                    LocalPaperOrderState::Accepted => "ACCEPTED",
                    LocalPaperOrderState::PartiallyFilled => "PARTIALLY_FILLED",
                    LocalPaperOrderState::Filled => "FILLED",
                    LocalPaperOrderState::Rejected => "REJECTED",
                    LocalPaperOrderState::CancelPending => "CANCEL_PENDING",
                    LocalPaperOrderState::Cancelled => "CANCELLED",
                }
                .into(),
                limit_price: None,
                kind: None,
                trigger_price: None,
            })
            .collect(),
        bitget_order_book: None,
        capabilities: vec!["simulation.execute".into(), "portfolio.read".into()],
        limitations: vec![
            state.disclosure.clone(),
            "No external provider or network I/O.".into(),
        ],
    }
}

fn actual_snapshot(
    workspace_id: &str,
    base_currency: &str,
    accounts: &[AccountConnection],
    fx_source: Option<&DataSourceEntry>,
    time_status: &TimeStatus,
    paper_state: Option<&LocalPaperState>,
) -> Result<PortfolioSnapshot> {
    if accounts.len() > MAX_PORTFOLIO_ACCOUNTS
        || accounts
            .iter()
            .any(|account| account.workspace_id != workspace_id)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    if accounts.iter().any(|account| {
        !valid_output_identity(&account.connection_id, 128)
            || !valid_output_identity(&account.label, 120)
            || !valid_output_identity(&account.provider_id, 32)
            || !valid_output_identity(&account.environment, 16)
            || account
                .data
                .as_ref()
                .and_then(|data| data.currency.as_deref())
                .is_some_and(|currency| !valid_output_identity(currency, 16))
    }) {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    let mut portfolio_accounts = Vec::with_capacity(accounts.len());
    let mut holdings = Vec::new();
    let mut open_orders = Vec::new();
    let mut fx_routes = Vec::new();
    let mut equity_values = Vec::new();
    let mut cash_values = Vec::new();
    let mut exposure_values = Vec::new();
    let mut realized_pnl_values = Vec::new();
    let mut unrealized_pnl_values = Vec::new();
    let mut paper_fills = Vec::new();
    let mut any_observation = false;
    let mut conversion_missing = false;
    let mut data_incomplete = false;
    let has_local_paper = accounts.iter().any(AccountConnection::is_local_paper);
    for account in accounts {
        let local_state = if account.is_local_paper() {
            let state =
                paper_state.ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
            if state.workspace_id != workspace_id
                || state.account_id != account.connection_id
                || state.provider_id != account.provider_id
                || state.environment != account.environment
            {
                return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
            }
            Some(state)
        } else {
            None
        };
        let local_data = local_state.map(|state| local_paper_data(account, state));
        let Some(data) = local_data.as_ref().or(account.data.as_ref()) else {
            portfolio_accounts.push(account_row(account, base_currency, None, None, 0, 0, None));
            continue;
        };
        if holdings
            .len()
            .saturating_add(data.balances.len())
            .saturating_add(data.positions.len())
            > MAX_PORTFOLIO_ROWS
            || open_orders.len().saturating_add(data.open_orders.len()) > MAX_PORTFOLIO_ROWS
            || data
                .balances
                .iter()
                .any(|balance| !valid_output_identity(&balance.asset, 64))
            || data.positions.iter().any(|position| {
                !valid_output_identity(&position.symbol, 64)
                    || position
                        .instrument_id
                        .as_deref()
                        .is_some_and(|id| !valid_output_identity(id, 128))
            })
            || data.open_orders.iter().any(|order| {
                !valid_output_identity(&order.broker_order_id, 128)
                    || !valid_output_identity(&order.symbol, 64)
                    || order
                        .instrument_id
                        .as_deref()
                        .is_some_and(|id| !valid_output_identity(id, 128))
            })
        {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        any_observation = true;
        if !account.is_local_paper() {
            data_incomplete = true;
        }
        let account_currency = local_state
            .map(|state| state.cash.currency.as_str())
            .or(data.currency.as_deref());
        let cash_balance = data
            .balances
            .iter()
            .find(|balance| account_currency.is_some_and(|currency| balance.asset == currency));
        let cash = if let Some(state) = local_state {
            Some(value_from_parts(
                Some(state.cash.value.as_str()),
                Some(state.cash.currency.as_str()),
                account_currency,
                base_currency,
                fx_source,
                time_status,
                false,
            )?)
        } else {
            cash_balance
                .map(|balance| balance.available.as_str())
                .map(|value| {
                    value_from_parts(
                        Some(value),
                        account_currency,
                        account_currency,
                        base_currency,
                        fx_source,
                        time_status,
                        false,
                    )
                })
                .transpose()?
        };
        let equity = if let Some(state) = local_state {
            Some(value_from_parts(
                Some(state.equity.value.as_str()),
                Some(state.equity.currency.as_str()),
                account_currency,
                base_currency,
                fx_source,
                time_status,
                false,
            )?)
        } else {
            cash_balance
                .and_then(|balance| balance.total.as_deref())
                .map(|value| {
                    value_from_parts(
                        Some(value),
                        account_currency,
                        account_currency,
                        base_currency,
                        fx_source,
                        time_status,
                        false,
                    )
                })
                .transpose()?
        };
        if let Some(value) = equity.as_ref() {
            conversion_missing |= value.workspace_value.is_none() && value.native_value.is_some();
            collect_fx(value, &mut fx_routes)?;
            equity_values.push(value.clone());
        }
        if let Some(value) = cash.as_ref() {
            conversion_missing |= value.workspace_value.is_none() && value.native_value.is_some();
            collect_fx(value, &mut fx_routes)?;
            cash_values.push(value.clone());
        }
        if let Some(state) = local_state {
            for (money, target) in [
                (&state.realized_pnl, &mut realized_pnl_values),
                (&state.unrealized_pnl, &mut unrealized_pnl_values),
                (&state.exposure, &mut exposure_values),
            ] {
                let value = value_from_parts(
                    Some(money.value.as_str()),
                    Some(money.currency.as_str()),
                    account_currency,
                    base_currency,
                    fx_source,
                    time_status,
                    false,
                )?;
                conversion_missing |=
                    value.workspace_value.is_none() && value.native_value.is_some();
                collect_fx(&value, &mut fx_routes)?;
                target.push(value);
            }
            for fill in &state.fills {
                let value = value_from_parts(
                    Some(fill.value.as_str()),
                    Some(fill.currency.as_str()),
                    account_currency,
                    base_currency,
                    fx_source,
                    time_status,
                    false,
                )?;
                conversion_missing |=
                    value.workspace_value.is_none() && value.native_value.is_some();
                collect_fx(&value, &mut fx_routes)?;
                paper_fills.push(PortfolioFill {
                    connection_id: account.connection_id.clone(),
                    account_label: account.label.clone(),
                    fill_id: fill.fill_id.clone(),
                    instrument_id: Some(fill.instrument_id.clone()),
                    asset: fill.instrument_id.clone(),
                    quantity: fill.quantity.clone(),
                    value,
                    observed_at: fill.observed_at.clone(),
                    health: account.health.clone(),
                });
            }
        }
        for balance in &data.balances {
            if let Some(total) = balance
                .total
                .as_deref()
                .or(Some(balance.available.as_str()))
            {
                let value = value_from_parts(
                    Some(total),
                    Some(balance.asset.as_str()),
                    account_currency,
                    base_currency,
                    fx_source,
                    time_status,
                    false,
                )?;
                conversion_missing |=
                    value.workspace_value.is_none() && value.native_value.is_some();
                collect_fx(&value, &mut fx_routes)?;
                holdings.push(holding_from_balance(account, balance, value, time_status));
            }
        }
        for position in &data.positions {
            let value = position
                .market_value
                .as_deref()
                .map(|amount| {
                    value_from_parts(
                        Some(amount),
                        position
                            .market_value_currency
                            .as_deref()
                            .or(position.instrument_currency.as_deref())
                            .or(account_currency),
                        account_currency,
                        base_currency,
                        fx_source,
                        time_status,
                        false,
                    )
                })
                .transpose()?;
            if let Some(value) = value.as_ref() {
                conversion_missing |=
                    value.workspace_value.is_none() && value.native_value.is_some();
                collect_fx(value, &mut fx_routes)?;
                if local_state.is_none() {
                    exposure_values.push(value.clone());
                }
            }
            holdings.push(holding_from_position(
                account,
                position,
                value,
                base_currency,
                time_status,
            ));
        }
        for order in &data.open_orders {
            open_orders.push(PortfolioOrder {
                connection_id: account.connection_id.clone(),
                account_label: account.label.clone(),
                broker_order_id: order.broker_order_id.clone(),
                instrument_id: order.instrument_id.clone(),
                asset: order
                    .instrument_id
                    .clone()
                    .unwrap_or_else(|| "UNAVAILABLE".into()),
                side: order.side.clone(),
                quantity: order.quantity.clone(),
                notional: order.notional.clone(),
                currency: order.currency.clone(),
                status: order.status.clone(),
                observed_at: observation_timestamp(account, time_status),
                health: account.health.clone(),
            });
        }
        portfolio_accounts.push(account_row(
            account,
            base_currency,
            equity,
            cash,
            data.positions.len(),
            data.open_orders.len(),
            None,
        ));
    }
    let status = if !any_observation {
        PortfolioStatus::Unavailable
    } else if conversion_missing {
        match fx_source.map(|source| source.status.clone()) {
            Some(DataSourceStatus::BlockedExternal) | Some(DataSourceStatus::Unverified) | None => {
                PortfolioStatus::BlockedExternal
            }
            Some(DataSourceStatus::Unavailable) => PortfolioStatus::Unavailable,
            Some(DataSourceStatus::Available) => PortfolioStatus::Degraded,
        }
    } else if data_incomplete {
        PortfolioStatus::Degraded
    } else {
        PortfolioStatus::Available
    };
    let reason = match status {
        PortfolioStatus::Available => {
            if has_local_paper {
                "TRADEX_SIMULATION; Local Paper is TradeX-managed simulation; its state is not provider truth or Live execution."
            } else {
                "Provider observations are shown with workspace-currency values where the route is trusted."
            }
        }
        PortfolioStatus::Degraded => {
            "Some provider fields or FX routes are unavailable; affected values remain unavailable."
        }
        PortfolioStatus::Unavailable => {
            "No complete provider or FX observations are available for portfolio normalization."
        }
        PortfolioStatus::BlockedExternal => {
            "OD-006 FX entitlement or verification is required before cross-currency values can be normalized."
        }
    };
    Ok(PortfolioSnapshot {
        workspace_id: workspace_id.into(),
        base_currency: base_currency.into(),
        observed_at: time_status.observed_at.clone(),
        status,
        availability_reason: reason.into(),
        totals: PortfolioTotals {
            equity: sum_values(&equity_values, base_currency, time_status)?,
            cash: sum_values(&cash_values, base_currency, time_status)?,
            unrealized_pnl: if unrealized_pnl_values.is_empty() {
                unavailable_value(base_currency)
            } else {
                sum_values(&unrealized_pnl_values, base_currency, time_status)?
            },
            realized_pnl: if realized_pnl_values.is_empty() {
                unavailable_value(base_currency)
            } else {
                sum_values(&realized_pnl_values, base_currency, time_status)?
            },
            exposure: sum_values(&exposure_values, base_currency, time_status)?,
        },
        accounts: portfolio_accounts,
        holdings,
        open_orders,
        fills: if has_local_paper {
            Some(paper_fills)
        } else {
            None
        },
        fx_routes,
        live_risk: PortfolioLiveRisk {
            eligible: false,
            reason: if has_local_paper {
                "TRADEX_SIMULATION_NOT_LIVE; simulation is never Live risk or provider truth."
                    .into()
            } else {
                "S09 is read-only; Live risk requires a later consumer to revalidate every account, market, time and FX gate.".into()
            },
        },
    })
}

fn fixture_snapshot(
    workspace_id: &str,
    base_currency: &str,
    time_status: &TimeStatus,
) -> Result<PortfolioSnapshot> {
    let account_specs = [
        (
            "fixture:t212-live",
            "Trading 212 Live (fixture)",
            "trading212",
            "LIVE",
            "EUR",
        ),
        (
            "fixture:binance-live",
            "Binance Live (fixture)",
            "binance",
            "LIVE",
            "USDT",
        ),
        (
            "fixture:alpaca-paper",
            "Alpaca Paper (fixture)",
            "alpaca",
            "PAPER",
            "USD",
        ),
    ];
    let mut accounts = Vec::new();
    for (id, label, provider, environment, currency) in account_specs {
        let equity = fixture_value("80000", currency, base_currency, time_status)?;
        let cash = fixture_value(
            match currency {
                "EUR" => "20000",
                "USDT" => "10000",
                _ => "2118",
            },
            currency,
            base_currency,
            time_status,
        )?;
        accounts.push(PortfolioAccount {
            connection_id: id.into(),
            label: label.into(),
            provider_id: provider.into(),
            environment: environment.into(),
            connection_state: ConnectionState::Connected,
            health: fixture_health(),
            account_currency: Some(currency.into()),
            equity,
            cash,
            positions_count: 1,
            open_orders_count: 1,
            fills_count: Some(1),
        });
    }
    let mut holdings = vec![
        fixture_holding(
            FixtureHoldingSpec {
                connection_id: "fixture:t212-live",
                account_label: "Trading 212 Live (fixture)",
                provider_id: "trading212",
                environment: "LIVE",
                instrument_id: Some("equity:US:AAPL"),
                quantity: "100",
                value: "18000",
                currency: "USD",
                pnl: "1200",
            },
            base_currency,
            time_status,
        )?,
        fixture_holding(
            FixtureHoldingSpec {
                connection_id: "fixture:binance-live",
                account_label: "Binance Live (fixture)",
                provider_id: "binance",
                environment: "LIVE",
                instrument_id: Some("crypto:BTC/USDT:spot"),
                quantity: "0.25",
                value: "16000",
                currency: "USDT",
                pnl: "1800",
            },
            base_currency,
            time_status,
        )?,
        fixture_holding(
            FixtureHoldingSpec {
                connection_id: "fixture:alpaca-paper",
                account_label: "Alpaca Paper (fixture)",
                provider_id: "alpaca",
                environment: "PAPER",
                instrument_id: Some("equity:US:MSFT"),
                quantity: "50",
                value: "22000",
                currency: "USD",
                pnl: "900",
            },
            base_currency,
            time_status,
        )?,
    ];
    holdings.push(fixture_holding(
        FixtureHoldingSpec {
            connection_id: "fixture:binance-live",
            account_label: "Binance Live (fixture)",
            provider_id: "binance",
            environment: "LIVE",
            instrument_id: None,
            quantity: "10000",
            value: "10000",
            currency: "USDT",
            pnl: "0",
        },
        base_currency,
        time_status,
    )?);
    let open_orders = vec![
        PortfolioOrder {
            connection_id: "fixture:t212-live".into(),
            account_label: "Trading 212 Live (fixture)".into(),
            broker_order_id: "fixture-order-aapl".into(),
            instrument_id: Some("equity:US:AAPL".into()),
            asset: "equity:US:AAPL".into(),
            side: "BUY".into(),
            quantity: Some("10".into()),
            notional: Some("1800".into()),
            currency: Some("USD".into()),
            status: "PENDING".into(),
            observed_at: time_status.observed_at.clone(),
            health: fixture_health(),
        },
        PortfolioOrder {
            connection_id: "fixture:binance-live".into(),
            account_label: "Binance Live (fixture)".into(),
            broker_order_id: "fixture-order-btc".into(),
            instrument_id: Some("crypto:BTC/USDT:spot".into()),
            asset: "crypto:BTC/USDT:spot".into(),
            side: "SELL".into(),
            quantity: Some("0.05".into()),
            notional: None,
            currency: Some("USDT".into()),
            status: "NEW".into(),
            observed_at: time_status.observed_at.clone(),
            health: fixture_health(),
        },
    ];
    let fills = Some(vec![PortfolioFill {
        connection_id: "fixture:t212-live".into(),
        account_label: "Trading 212 Live (fixture)".into(),
        fill_id: "fixture-fill-aapl".into(),
        instrument_id: Some("equity:US:AAPL".into()),
        asset: "equity:US:AAPL".into(),
        quantity: "100".into(),
        value: fixture_value("16800", "USD", base_currency, time_status)?,
        observed_at: time_status.observed_at.clone(),
        health: fixture_health(),
    }]);
    let mut fx_routes = Vec::new();
    for value in accounts
        .iter()
        .flat_map(|account| [&account.equity, &account.cash])
        .chain(holdings.iter().map(|holding| &holding.value))
        .chain(
            fills
                .iter()
                .flat_map(|items| items.iter().map(|fill| &fill.value)),
        )
    {
        collect_fx(value, &mut fx_routes)?;
    }
    let equity_values: Vec<_> = accounts
        .iter()
        .map(|account| account.equity.clone())
        .collect();
    let cash_values: Vec<_> = accounts
        .iter()
        .map(|account| account.cash.clone())
        .collect();
    let exposure_values: Vec<_> = holdings
        .iter()
        .filter(|holding| holding.instrument_id.is_some())
        .map(|holding| holding.value.clone())
        .collect();
    let unrealized_pnl_values: Vec<_> = holdings
        .iter()
        .filter_map(|holding| holding.unrealized_pnl.clone())
        .collect();
    let degraded = fx_routes.iter().any(|route| {
        matches!(
            route.quality,
            FxQuality::Degraded | FxQuality::Unknown | FxQuality::Unavailable
        )
    });
    Ok(PortfolioSnapshot {
        workspace_id: workspace_id.into(),
        base_currency: base_currency.into(),
        observed_at: time_status.observed_at.clone(),
        status: if degraded {
            PortfolioStatus::Degraded
        } else {
            PortfolioStatus::Available
        },
        availability_reason: if degraded {
            format!(
                "TRADEX_SIMULATION; scenario={FIXTURE_SCENARIO_ID}; synthetic fixture includes degraded or unavailable FX routes; workspace analytics are not authority."
            )
        } else {
            format!(
                "TRADEX_SIMULATION; scenario={FIXTURE_SCENARIO_ID}; synthetic fixture values are labelled for contract and rendering verification only."
            )
        },
        totals: PortfolioTotals {
            equity: sum_values(&equity_values, base_currency, time_status)?,
            cash: sum_values(&cash_values, base_currency, time_status)?,
            unrealized_pnl: sum_values(&unrealized_pnl_values, base_currency, time_status)?,
            realized_pnl: fixture_value("750", "USD", base_currency, time_status)?,
            exposure: sum_values(&exposure_values, base_currency, time_status)?,
        },
        accounts,
        holdings,
        open_orders,
        fills,
        fx_routes,
        live_risk: PortfolioLiveRisk {
            eligible: false,
            reason: format!(
                "TRADEX_SIMULATION; scenario={FIXTURE_SCENARIO_ID}; stablecoin quality is degraded in this fixture; S09 never authorizes Live risk."
            ),
        },
    })
}

fn fixture_health() -> AccountHealth {
    AccountHealth {
        connection: "ONLINE".into(),
        authentication: "VALID".into(),
        credential: "AVAILABLE".into(),
        private_stream: "NOT_CONFIGURED".into(),
        reconciliation: "NOT_RUN".into(),
        execution_eligibility: "BLOCKED".into(),
        arming: "DISARMED".into(),
        arming_reason: "NOT_ARMED".into(),
        reason: format!(
            "TRADEX_SIMULATION; scenario={FIXTURE_SCENARIO_ID}; synthetic fixture account; no provider connection was made."
        ),
    }
}

struct FixtureHoldingSpec<'a> {
    connection_id: &'a str,
    account_label: &'a str,
    provider_id: &'a str,
    environment: &'a str,
    instrument_id: Option<&'a str>,
    quantity: &'a str,
    value: &'a str,
    currency: &'a str,
    pnl: &'a str,
}

fn fixture_holding(
    spec: FixtureHoldingSpec<'_>,
    base_currency: &str,
    time_status: &TimeStatus,
) -> Result<PortfolioHolding> {
    Ok(PortfolioHolding {
        connection_id: spec.connection_id.into(),
        account_label: spec.account_label.into(),
        provider_id: spec.provider_id.into(),
        environment: spec.environment.into(),
        venue: None,
        instrument_id: spec.instrument_id.map(str::to_owned),
        asset: spec.instrument_id.unwrap_or("UNAVAILABLE").into(),
        quantity: Some(normalize_decimal(spec.quantity)?),
        value: fixture_value(spec.value, spec.currency, base_currency, time_status)?,
        unrealized_pnl: Some(fixture_value(
            spec.pnl,
            spec.currency,
            base_currency,
            time_status,
        )?),
        observed_at: time_status.observed_at.clone(),
        health: fixture_health(),
    })
}

fn holding_from_balance(
    account: &AccountConnection,
    balance: &Balance,
    value: PortfolioValue,
    time_status: &TimeStatus,
) -> PortfolioHolding {
    PortfolioHolding {
        connection_id: account.connection_id.clone(),
        account_label: account.label.clone(),
        provider_id: account.provider_id.clone(),
        environment: account.environment.clone(),
        venue: None,
        instrument_id: None,
        asset: balance.asset.clone(),
        quantity: balance
            .total
            .clone()
            .or_else(|| Some(balance.available.clone())),
        value,
        unrealized_pnl: None,
        observed_at: observation_timestamp(account, time_status),
        health: account.health.clone(),
    }
}

fn holding_from_position(
    account: &AccountConnection,
    position: &Position,
    value: Option<PortfolioValue>,
    base_currency: &str,
    time_status: &TimeStatus,
) -> PortfolioHolding {
    PortfolioHolding {
        connection_id: account.connection_id.clone(),
        account_label: account.label.clone(),
        provider_id: account.provider_id.clone(),
        environment: account.environment.clone(),
        venue: None,
        instrument_id: position.instrument_id.clone(),
        asset: position
            .instrument_id
            .clone()
            .unwrap_or_else(|| "UNAVAILABLE".into()),
        quantity: Some(position.quantity.clone()),
        value: value.unwrap_or_else(|| unavailable_value(base_currency)),
        unrealized_pnl: None,
        observed_at: observation_timestamp(account, time_status),
        health: account.health.clone(),
    }
}

fn observation_timestamp(account: &AccountConnection, time_status: &TimeStatus) -> String {
    account
        .last_successful_sync
        .clone()
        .unwrap_or_else(|| time_status.observed_at.clone())
}

fn account_row(
    account: &AccountConnection,
    base_currency: &str,
    equity: Option<PortfolioValue>,
    cash: Option<PortfolioValue>,
    positions_count: usize,
    open_orders_count: usize,
    fills_count: Option<u32>,
) -> PortfolioAccount {
    PortfolioAccount {
        connection_id: account.connection_id.clone(),
        label: account.label.clone(),
        provider_id: account.provider_id.clone(),
        environment: account.environment.clone(),
        connection_state: account.connection_state.clone(),
        health: account.health.clone(),
        account_currency: account.data.as_ref().and_then(|data| data.currency.clone()),
        equity: equity.unwrap_or_else(|| unavailable_value(base_currency)),
        cash: cash.unwrap_or_else(|| unavailable_value(base_currency)),
        positions_count: positions_count as u32,
        open_orders_count: open_orders_count as u32,
        fills_count,
    }
}

fn value_from_parts(
    native_value: Option<&str>,
    native_currency: Option<&str>,
    account_currency: Option<&str>,
    base_currency: &str,
    fx_source: Option<&DataSourceEntry>,
    time_status: &TimeStatus,
    fixture: bool,
) -> Result<PortfolioValue> {
    let native_value = native_value.map(normalize_decimal).transpose()?;
    let native_currency = native_currency
        .map(str::to_owned)
        .filter(|currency| valid_currency(currency));
    let account_currency = account_currency
        .map(str::to_owned)
        .filter(|currency| valid_currency(currency));
    let account_value = match (&native_value, &native_currency, &account_currency) {
        (Some(value), Some(native), Some(account)) if native == account => Some(value.clone()),
        _ => None,
    };
    let (workspace_value, fx_provenance) = match (&native_value, native_currency.as_deref()) {
        (Some(value), Some(currency)) if currency == base_currency => (
            Some(value.clone()),
            Some(identity_fx(currency, &time_status.observed_at)),
        ),
        (Some(value), Some(currency)) if fixture => {
            fixture_conversion(value, currency, base_currency, time_status)?
        }
        (Some(_), Some(currency)) => (
            None,
            Some(unavailable_fx(
                currency,
                base_currency,
                fx_source,
                time_status,
            )),
        ),
        (Some(_), None) => (
            None,
            Some(unavailable_fx(
                "UNKNOWN",
                base_currency,
                fx_source,
                time_status,
            )),
        ),
        _ => (None, None),
    };
    Ok(PortfolioValue {
        native_value,
        native_currency,
        account_value,
        account_currency,
        workspace_value,
        workspace_currency: base_currency.into(),
        fx_provenance,
    })
}

fn fixture_value(
    native: &str,
    currency: &str,
    base_currency: &str,
    time_status: &TimeStatus,
) -> Result<PortfolioValue> {
    value_from_parts(
        Some(native),
        Some(currency),
        Some(currency),
        base_currency,
        None,
        time_status,
        true,
    )
}

fn fixture_conversion(
    value: &str,
    currency: &str,
    base_currency: &str,
    time_status: &TimeStatus,
) -> Result<(Option<String>, Option<FxProvenance>)> {
    if currency == base_currency {
        return Ok((
            Some(value.into()),
            Some(identity_fx(currency, &time_status.observed_at)),
        ));
    }
    let (rate, path, quality, warning) = match (currency, base_currency) {
        ("EUR", "USD") => ("1.08", "EUR -> USD", FxQuality::Verified, None),
        ("USD", "EUR") => ("0.92", "USD -> EUR", FxQuality::Verified, None),
        ("USDT", "USD") => (
            "0.982",
            "USDT -> USD",
            FxQuality::Degraded,
            Some("USDT is not USD; 1.8% synthetic depeg warning."),
        ),
        ("USDT", "EUR") => (
            "0.90344",
            "USDT -> USD -> EUR",
            FxQuality::Degraded,
            Some("USDT is not USD; synthetic depeg warning is carried into the EUR route."),
        ),
        _ => {
            return Ok((
                None,
                Some(FxProvenance {
                    source_id: "FX-FIXTURE".into(),
                    pair_path: format!("{currency} -> {base_currency}"),
                    rate: None,
                    provider_timestamp: None,
                    received_timestamp: time_status.observed_at.clone(),
                    freshness: FxFreshness::Unavailable,
                    quality: FxQuality::Unavailable,
                    depeg_warning: Some(
                        "No deterministic fixture route exists for this base currency.".into(),
                    ),
                }),
            ));
        }
    };
    Ok((
        Some(decimal_mul(value, rate)?),
        Some(FxProvenance {
            source_id: "FX-FIXTURE".into(),
            pair_path: path.into(),
            rate: Some(rate.into()),
            provider_timestamp: Some("2026-09-13T16:00:00Z".into()),
            received_timestamp: time_status.observed_at.clone(),
            freshness: FxFreshness::Healthy,
            quality,
            depeg_warning: warning.map(str::to_owned),
        }),
    ))
}

fn identity_fx(currency: &str, received: &str) -> FxProvenance {
    FxProvenance {
        source_id: IDENTITY_SOURCE.into(),
        pair_path: format!("{currency} -> {currency}"),
        rate: Some("1".into()),
        provider_timestamp: None,
        received_timestamp: received.into(),
        freshness: FxFreshness::Healthy,
        quality: FxQuality::Verified,
        depeg_warning: None,
    }
}

fn unavailable_fx(
    currency: &str,
    base_currency: &str,
    source: Option<&DataSourceEntry>,
    time_status: &TimeStatus,
) -> FxProvenance {
    FxProvenance {
        source_id: source.map_or_else(|| "OD-006".into(), |source| source.source_id.clone()),
        pair_path: format!("{currency} -> {base_currency}"),
        rate: None,
        provider_timestamp: source.and_then(|source| source.observed_at.clone()),
        received_timestamp: time_status.observed_at.clone(),
        freshness: if source.is_some_and(|source| source.observed_at.is_some()) {
            FxFreshness::Stale
        } else {
            FxFreshness::Unavailable
        },
        quality: FxQuality::Unavailable,
        depeg_warning: (currency == "USDT")
            .then(|| "USDT parity is not assumed without a trusted route.".into()),
    }
}

fn collect_fx(value: &PortfolioValue, routes: &mut Vec<FxProvenance>) -> Result<()> {
    if let Some(route) = &value.fx_provenance
        && !routes.iter().any(|existing| existing == route)
    {
        if routes.len() >= MAX_PORTFOLIO_FX_ROUTES {
            return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
        }
        routes.push(route.clone());
    }
    Ok(())
}

fn sum_values(
    values: &[PortfolioValue],
    base_currency: &str,
    time_status: &TimeStatus,
) -> Result<PortfolioValue> {
    let mut total: Option<String> = None;
    let mut route: Option<FxProvenance> = None;
    let mut missing_route: Option<FxProvenance> = None;
    let mut missing_value = false;
    for value in values {
        let Some(workspace_value) = value.workspace_value.as_deref() else {
            missing_value |= value.native_value.is_some();
            if missing_route.is_none() {
                missing_route = value.fx_provenance.clone();
            }
            continue;
        };
        total = Some(match total {
            Some(existing) => decimal_add(&existing, workspace_value)?,
            None => workspace_value.into(),
        });
        if route.is_none() {
            route = value.fx_provenance.clone();
        }
    }
    Ok(PortfolioValue {
        native_value: None,
        native_currency: None,
        account_value: None,
        account_currency: None,
        workspace_value: if missing_value { None } else { total },
        workspace_currency: base_currency.into(),
        fx_provenance: missing_route
            .or(route)
            .or_else(|| Some(unavailable_fx("UNKNOWN", base_currency, None, time_status))),
    })
}

fn unavailable_value(currency: &str) -> PortfolioValue {
    PortfolioValue {
        native_value: None,
        native_currency: None,
        account_value: None,
        account_currency: None,
        workspace_value: None,
        workspace_currency: currency.into(),
        fx_provenance: None,
    }
}

fn valid_workspace_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

fn valid_currency(value: &str) -> bool {
    (2..=16).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn valid_base_currency(value: &str) -> bool {
    value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase())
}

fn valid_output_identity(value: &str, max_len: usize) -> bool {
    !value.is_empty() && value.len() <= max_len && !value.chars().any(char::is_control)
}

fn normalize_decimal(value: &str) -> Result<String> {
    crate::provider_io::decimal(&Value::String(value.into()))
}

pub(crate) fn decimal_add(left: &str, right: &str) -> Result<String> {
    let (left_negative, left_digits, left_scale) = decimal_parts(left)?;
    let (right_negative, right_digits, right_scale) = decimal_parts(right)?;
    let scale = left_scale.max(right_scale);
    let left_digits = pad_scale(&left_digits, left_scale, scale);
    let right_digits = pad_scale(&right_digits, right_scale, scale);
    let (negative, digits) = if left_negative == right_negative {
        (left_negative, add_abs(&left_digits, &right_digits))
    } else {
        match cmp_abs(&left_digits, &right_digits) {
            std::cmp::Ordering::Equal => (false, "0".into()),
            std::cmp::Ordering::Greater => (left_negative, sub_abs(&left_digits, &right_digits)),
            std::cmp::Ordering::Less => (right_negative, sub_abs(&right_digits, &left_digits)),
        }
    };
    format_decimal(negative, &digits, scale)
}

pub(crate) fn decimal_mul(left: &str, right: &str) -> Result<String> {
    let (left_negative, left_digits, left_scale) = decimal_parts(left)?;
    let (right_negative, right_digits, right_scale) = decimal_parts(right)?;
    let mut digits = vec![0u8; left_digits.len() + right_digits.len()];
    for (left_index, left_byte) in left_digits.bytes().rev().enumerate() {
        for (right_index, right_byte) in right_digits.bytes().rev().enumerate() {
            let index = digits.len() - 1 - (left_index + right_index);
            let product = (left_byte - b'0') * (right_byte - b'0') + digits[index];
            digits[index] = product % 10;
            let mut carry = product / 10;
            let mut carry_index = index;
            while carry > 0 && carry_index > 0 {
                carry_index -= 1;
                let sum = digits[carry_index] + carry;
                digits[carry_index] = sum % 10;
                carry = sum / 10;
            }
        }
    }
    let text: String = digits
        .into_iter()
        .map(|digit| char::from(b'0' + digit))
        .collect();
    format_decimal(
        left_negative != right_negative,
        &text,
        left_scale + right_scale,
    )
}

pub(crate) fn decimal_div(left: &str, right: &str) -> Result<String> {
    let (left_negative, left_digits, left_scale) = decimal_parts(left)?;
    let (right_negative, right_digits, right_scale) = decimal_parts(right)?;
    if right_digits.chars().all(|digit| digit == '0') {
        return Err(TradeXError::new("PAPER_QUOTE_UNAVAILABLE"));
    }
    let target_scale = 18usize;
    let shift = target_scale
        .checked_add(right_scale)
        .and_then(|value| value.checked_sub(left_scale))
        .ok_or_else(|| TradeXError::new("PAPER_QUANTITY_UNREPRESENTABLE"))?;
    let numerator = format!("{left_digits}{}", "0".repeat(shift));
    let quotient = div_abs(&numerator, &right_digits)?;
    format_decimal(left_negative != right_negative, &quotient, target_scale)
}

fn decimal_parts(value: &str) -> Result<(bool, String, usize)> {
    let normalized = normalize_decimal(value)?;
    let (negative, unsigned) = normalized
        .strip_prefix('-')
        .map_or((false, normalized.as_str()), |value| (true, value));
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    Ok((negative, format!("{whole}{fraction}"), fraction.len()))
}

fn pad_scale(digits: &str, current: usize, target: usize) -> String {
    format!("{digits}{}", "0".repeat(target.saturating_sub(current)))
}

fn add_abs(left: &str, right: &str) -> String {
    let mut output = Vec::with_capacity(left.len().max(right.len()) + 1);
    let mut carry = 0u8;
    let mut left_iter = left.bytes().rev();
    let mut right_iter = right.bytes().rev();
    loop {
        let left_digit = left_iter.next().map_or(0, |byte| byte - b'0');
        let right_digit = right_iter.next().map_or(0, |byte| byte - b'0');
        if left_digit == 0
            && right_digit == 0
            && carry == 0
            && left_iter.len() == 0
            && right_iter.len() == 0
        {
            break;
        }
        let sum = left_digit + right_digit + carry;
        output.push(b'0' + sum % 10);
        carry = sum / 10;
        if left_iter.len() == 0 && right_iter.len() == 0 && carry == 0 {
            break;
        }
    }
    output.reverse();
    String::from_utf8(output).unwrap_or_else(|_| "0".into())
}

fn cmp_abs(left: &str, right: &str) -> std::cmp::Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn sub_abs(left: &str, right: &str) -> String {
    let mut output = Vec::with_capacity(left.len());
    let mut borrow = 0i8;
    let mut left_iter = left.bytes().rev();
    let mut right_iter = right.bytes().rev();
    for _ in 0..left.len() {
        let left_digit = left_iter.next().map_or(0, |byte| (byte - b'0') as i8);
        let right_digit = right_iter.next().map_or(0, |byte| (byte - b'0') as i8);
        let mut value = left_digit - borrow - right_digit;
        if value < 0 {
            value += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        output.push(b'0' + value as u8);
    }
    while output.len() > 1 && output.last() == Some(&b'0') {
        output.pop();
    }
    output.reverse();
    String::from_utf8(output).unwrap_or_else(|_| "0".into())
}

fn div_abs(numerator: &str, denominator: &str) -> Result<String> {
    let mut remainder = String::from("0");
    let mut quotient = String::new();
    for digit in numerator.bytes() {
        if !digit.is_ascii_digit() {
            return Err(TradeXError::new("PAPER_QUANTITY_UNREPRESENTABLE"));
        }
        remainder = if remainder == "0" {
            char::from(digit).to_string()
        } else {
            format!("{remainder}{}", char::from(digit))
        };
        let mut count = 0u8;
        while cmp_abs(&remainder, denominator) != std::cmp::Ordering::Less {
            remainder = sub_abs(&remainder, denominator);
            count = count
                .checked_add(1)
                .ok_or_else(|| TradeXError::new("PAPER_QUANTITY_UNREPRESENTABLE"))?;
        }
        quotient.push(char::from(b'0' + count));
    }
    let quotient = quotient.trim_start_matches('0');
    Ok(if quotient.is_empty() {
        "0".into()
    } else {
        quotient.into()
    })
}

fn format_decimal(negative: bool, digits: &str, scale: usize) -> Result<String> {
    if digits.len() > 128 {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let mut whole = if scale >= digits.len() {
        "0".to_owned()
    } else {
        digits[..digits.len() - scale].to_owned()
    };
    let mut fraction = if scale == 0 {
        String::new()
    } else if scale >= digits.len() {
        format!("{}{}", "0".repeat(scale - digits.len()), digits)
    } else {
        digits[digits.len() - scale..].to_owned()
    };
    while whole.len() > 1 && whole.starts_with('0') {
        whole.remove(0);
    }
    while fraction.ends_with('0') {
        fraction.pop();
    }
    let result = if fraction.is_empty() {
        whole
    } else {
        format!("{whole}.{fraction}")
    };
    Ok(if negative && result != "0" {
        format!("-{result}")
    } else {
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{TimeConfidence, TimeStatus};
    use crate::providers::AccountData;

    fn time_status() -> TimeStatus {
        TimeStatus {
            workspace_id: "w".into(),
            confidence: TimeConfidence::Trusted,
            wall_clock: "2026-09-14T00:00:00Z".into(),
            monotonic_ms: 1,
            provider_offset_ms: None,
            observed_at: "2026-09-14T00:00:00Z".into(),
            reason: "fixture".into(),
            remediation: crate::protocol::Remediation {
                id: "none".into(),
                label: "none".into(),
            },
        }
    }

    #[test]
    fn decimal_operations_are_exact_and_signed() {
        assert_eq!(decimal_add("10.25", "2.5").unwrap(), "12.75");
        assert_eq!(decimal_add("10", "-2.5").unwrap(), "7.5");
        assert_eq!(decimal_add("-10", "2.5").unwrap(), "-7.5");
        assert_eq!(decimal_mul("100", "0.982").unwrap(), "98.2");
        assert_eq!(decimal_div("200", "100").unwrap(), "2");
        assert_eq!(decimal_div("1.25", "100").unwrap(), "0.0125");
    }

    #[test]
    fn fixture_preserves_usdt_warning_and_base_currency() {
        let status = time_status();
        let snapshot = get("w", "USD", &[], None, &status, true).unwrap();
        assert_eq!(snapshot.base_currency, "USD");
        assert_eq!(snapshot.status, PortfolioStatus::Degraded);
        assert!(
            snapshot
                .fx_routes
                .iter()
                .any(|route| route.pair_path == "USDT -> USD" && route.depeg_warning.is_some())
        );
        assert_eq!(
            snapshot.totals.unrealized_pnl.workspace_value.as_deref(),
            Some("3867.6")
        );
        assert!(!snapshot.live_risk.eligible);
    }

    #[test]
    fn fixture_with_unmapped_base_currency_degrades_instead_of_claiming_availability() {
        let snapshot = get("w", "GBP", &[], None, &time_status(), true).unwrap();
        assert_eq!(snapshot.status, PortfolioStatus::Degraded);
        assert!(snapshot.totals.equity.workspace_value.is_none());
    }

    #[test]
    fn invalid_base_currency_cannot_escape_snapshot_schema() {
        let error = get("w", "USDT", &[], None, &time_status(), false).unwrap_err();
        assert_eq!(error.code, "IPC_PAYLOAD_INVALID");
    }

    #[test]
    fn fixture_exposure_excludes_unavailable_balance_identity() {
        let snapshot = get("w", "USD", &[], None, &time_status(), true).unwrap();
        assert_eq!(
            snapshot.totals.exposure.workspace_value.as_deref(),
            Some("55712")
        );
    }

    #[test]
    fn empty_production_snapshot_is_unavailable_without_mutation() {
        let status = time_status();
        let snapshot = get("w", "EUR", &[], None, &status, false).unwrap();
        assert_eq!(snapshot.status, PortfolioStatus::Unavailable);
        assert!(snapshot.totals.equity.workspace_value.is_none());
        assert!(snapshot.fills.is_none());
    }

    #[test]
    fn missing_equity_total_stays_unavailable() {
        let mut account = AccountConnection::new(
            "w".into(),
            "alpaca".into(),
            "PAPER".into(),
            "cash-only".into(),
        )
        .unwrap();
        account.connection_state = ConnectionState::Connected;
        account.data = Some(AccountData {
            remote_account_id: "remote".into(),
            account_type: "PAPER".into(),
            currency: Some("USD".into()),
            buying_power: None,
            balances: vec![Balance {
                asset: "USD".into(),
                available: "10".into(),
                total: None,
                reserved: None,
                in_pies: None,
                locked: None,
                restricted_available: None,
            }],
            positions: vec![],
            open_orders: vec![],
            bitget_order_book: None,
            capabilities: vec![],
            limitations: vec![],
        });

        let snapshot = get("w", "USD", &[account], None, &time_status(), false).unwrap();
        assert_eq!(
            snapshot.accounts[0].cash.workspace_value.as_deref(),
            Some("10")
        );
        assert!(snapshot.accounts[0].equity.workspace_value.is_none());
        assert!(snapshot.totals.equity.workspace_value.is_none());
        assert_eq!(snapshot.totals.cash.workspace_value.as_deref(), Some("10"));
    }

    #[test]
    fn unmapped_provider_symbols_remain_unavailable() {
        let mut account = AccountConnection::new(
            "w".into(),
            "alpaca".into(),
            "PAPER".into(),
            "unmapped".into(),
        )
        .unwrap();
        account.connection_state = ConnectionState::Connected;
        account.data = Some(AccountData {
            remote_account_id: "remote".into(),
            account_type: "PAPER".into(),
            currency: Some("USD".into()),
            buying_power: None,
            balances: vec![],
            positions: vec![Position {
                symbol: "UNKNOWN".into(),
                instrument_id: None,
                quantity: "1".into(),
                market_value: Some("10".into()),
                average_entry_price: None,
                instrument_currency: Some("USD".into()),
                market_value_currency: None,
            }],
            open_orders: vec![crate::providers::OpenOrder {
                broker_order_id: "order-unknown".into(),
                symbol: "UNKNOWN".into(),
                instrument_id: None,
                side: "BUY".into(),
                quantity: Some("1".into()),
                notional: Some("10".into()),
                filled_quantity: None,
                filled_value: None,
                currency: Some("USD".into()),
                status: "NEW".into(),
                limit_price: None,
                kind: None,
                trigger_price: None,
            }],
            bitget_order_book: None,
            capabilities: vec![],
            limitations: vec![],
        });

        let snapshot = get("w", "USD", &[account], None, &time_status(), false).unwrap();
        let holding = snapshot
            .holdings
            .iter()
            .find(|holding| holding.instrument_id.is_none())
            .unwrap();
        assert_eq!(holding.asset, "UNAVAILABLE");
        assert_eq!(snapshot.open_orders[0].instrument_id, None);
        assert_eq!(snapshot.open_orders[0].asset, "UNAVAILABLE");
    }

    #[test]
    fn unknown_native_currency_keeps_an_unavailable_fx_route() {
        let value =
            value_from_parts(Some("12.5"), None, None, "USD", None, &time_status(), false).unwrap();
        assert!(value.workspace_value.is_none());
        assert_eq!(
            value.fx_provenance.as_ref().unwrap().pair_path,
            "UNKNOWN -> USD"
        );
    }

    #[test]
    fn aggregation_fails_closed_when_one_native_value_lacks_conversion() {
        let status = time_status();
        let known = value_from_parts(
            Some("5"),
            Some("USD"),
            Some("USD"),
            "USD",
            None,
            &status,
            false,
        )
        .unwrap();
        let missing = value_from_parts(
            Some("3"),
            Some("EUR"),
            Some("EUR"),
            "USD",
            None,
            &status,
            false,
        )
        .unwrap();
        let total = sum_values(&[known, missing], "USD", &status).unwrap();
        assert!(total.workspace_value.is_none());
        assert_eq!(
            total.fx_provenance.as_ref().unwrap().pair_path,
            "EUR -> USD"
        );
    }

    #[test]
    fn production_snapshot_rejects_rows_and_routes_over_wire_limits() {
        let mut account = AccountConnection::new(
            "w".into(),
            "alpaca".into(),
            "PAPER".into(),
            "bounded".into(),
        )
        .unwrap();
        account.connection_state = ConnectionState::Connected;
        account.data = Some(AccountData {
            remote_account_id: "remote".into(),
            account_type: "PAPER".into(),
            currency: Some("USD".into()),
            buying_power: None,
            balances: (0..=MAX_PORTFOLIO_ROWS)
                .map(|index| Balance {
                    asset: format!("A{index}"),
                    available: "1".into(),
                    total: Some("1".into()),
                    reserved: None,
                    in_pies: None,
                    locked: None,
                    restricted_available: None,
                })
                .collect(),
            positions: vec![],
            open_orders: vec![],
            bitget_order_book: None,
            capabilities: vec![],
            limitations: vec![],
        });
        let error = get("w", "USD", &[account], None, &time_status(), false).unwrap_err();
        assert_eq!(error.code, "PROVIDER_DATA_INCOMPLETE");

        let mut oversized_asset =
            AccountConnection::new("w".into(), "alpaca".into(), "PAPER".into(), "asset".into())
                .unwrap();
        oversized_asset.connection_state = ConnectionState::Connected;
        oversized_asset.data = Some(AccountData {
            remote_account_id: "remote".into(),
            account_type: "PAPER".into(),
            currency: Some("USD".into()),
            buying_power: None,
            balances: vec![Balance {
                asset: "A".repeat(65),
                available: "1".into(),
                total: Some("1".into()),
                reserved: None,
                in_pies: None,
                locked: None,
                restricted_available: None,
            }],
            positions: vec![],
            open_orders: vec![],
            bitget_order_book: None,
            capabilities: vec![],
            limitations: vec![],
        });
        let error = get("w", "USD", &[oversized_asset], None, &time_status(), false).unwrap_err();
        assert_eq!(error.code, "PROVIDER_DATA_INCOMPLETE");

        let mut oversized_position = AccountConnection::new(
            "w".into(),
            "alpaca".into(),
            "PAPER".into(),
            "position".into(),
        )
        .unwrap();
        oversized_position.connection_state = ConnectionState::Connected;
        oversized_position.data = Some(AccountData {
            remote_account_id: "remote".into(),
            account_type: "PAPER".into(),
            currency: Some("USD".into()),
            buying_power: None,
            balances: vec![],
            positions: vec![Position {
                symbol: "S".repeat(65),
                instrument_id: None,
                quantity: "1".into(),
                market_value: None,
                average_entry_price: None,
                instrument_currency: None,
                market_value_currency: None,
            }],
            open_orders: vec![],
            bitget_order_book: None,
            capabilities: vec![],
            limitations: vec![],
        });
        let error = get(
            "w",
            "USD",
            &[oversized_position],
            None,
            &time_status(),
            false,
        )
        .unwrap_err();
        assert_eq!(error.code, "PROVIDER_DATA_INCOMPLETE");

        let mut oversized_account = AccountConnection::new(
            "w".into(),
            "alpaca".into(),
            "PAPER".into(),
            "account".into(),
        )
        .unwrap();
        oversized_account.label = "L".repeat(121);
        let error = get(
            "w",
            "USD",
            &[oversized_account],
            None,
            &time_status(),
            false,
        )
        .unwrap_err();
        assert_eq!(error.code, "PROVIDER_DATA_INCOMPLETE");

        let mut routed =
            AccountConnection::new("w".into(), "alpaca".into(), "PAPER".into(), "routes".into())
                .unwrap();
        routed.connection_state = ConnectionState::Connected;
        routed.data = Some(AccountData {
            remote_account_id: "remote".into(),
            account_type: "PAPER".into(),
            currency: None,
            buying_power: None,
            balances: vec![],
            positions: (0..=MAX_PORTFOLIO_FX_ROUTES)
                .map(|index| Position {
                    symbol: format!("S{index}"),
                    instrument_id: None,
                    quantity: "1".into(),
                    market_value: Some("1".into()),
                    average_entry_price: None,
                    instrument_currency: Some(format!("C{index}")),
                    market_value_currency: None,
                })
                .collect(),
            open_orders: vec![],
            bitget_order_book: None,
            capabilities: vec![],
            limitations: vec![],
        });
        let error = get("w", "USD", &[routed], None, &time_status(), false).unwrap_err();
        assert_eq!(error.code, "PROVIDER_DATA_INCOMPLETE");
    }
}
