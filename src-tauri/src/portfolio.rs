use crate::market;
use crate::protocol::{
    DataSourceEntry, DataSourceStatus, FxFreshness, FxProvenance, FxQuality, PortfolioAccount,
    PortfolioFill, PortfolioHolding, PortfolioLiveRisk, PortfolioOrder, PortfolioSnapshot,
    PortfolioStatus, PortfolioTotals, PortfolioValue, Result, TimeStatus, TradeXError,
};
use crate::providers::{AccountConnection, AccountHealth, Balance, ConnectionState, Position};
use serde_json::Value;

const IDENTITY_SOURCE: &str = "IDENTITY";

pub fn get(
    workspace_id: &str,
    base_currency: &str,
    accounts: &[AccountConnection],
    fx_source: Option<&DataSourceEntry>,
    time_status: &TimeStatus,
    fixture: bool,
) -> Result<PortfolioSnapshot> {
    if !valid_workspace_id(workspace_id) || !valid_currency(base_currency) {
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
    )
}

fn actual_snapshot(
    workspace_id: &str,
    base_currency: &str,
    accounts: &[AccountConnection],
    fx_source: Option<&DataSourceEntry>,
    time_status: &TimeStatus,
) -> Result<PortfolioSnapshot> {
    if accounts.len() > 256
        || accounts
            .iter()
            .any(|account| account.workspace_id != workspace_id)
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    let mut portfolio_accounts = Vec::with_capacity(accounts.len());
    let mut holdings = Vec::new();
    let mut open_orders = Vec::new();
    let mut fx_routes = Vec::new();
    let mut equity_values = Vec::new();
    let mut cash_values = Vec::new();
    let mut exposure_values = Vec::new();
    let mut any_observation = false;
    let mut conversion_missing = false;
    for account in accounts {
        let Some(data) = account.data.as_ref() else {
            portfolio_accounts.push(account_row(account, base_currency, None, None, 0, 0, 0));
            continue;
        };
        any_observation = true;
        let account_currency = data.currency.as_deref();
        let cash_balance = data
            .balances
            .iter()
            .find(|balance| account_currency.is_some_and(|currency| balance.asset == currency));
        let cash = cash_balance
            .and_then(|balance| {
                balance
                    .total
                    .as_deref()
                    .or(Some(balance.available.as_str()))
            })
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
            .transpose()?;
        let equity = cash_balance
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
            .transpose()?;
        if let Some(value) = equity.as_ref() {
            conversion_missing |= value.workspace_value.is_none() && value.native_value.is_some();
            collect_fx(value, &mut fx_routes);
            equity_values.push(value.clone());
        }
        if let Some(value) = cash.as_ref() {
            conversion_missing |= value.workspace_value.is_none() && value.native_value.is_some();
            collect_fx(value, &mut fx_routes);
            cash_values.push(value.clone());
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
                collect_fx(&value, &mut fx_routes);
                holdings.push(holding_from_balance(account, balance, value));
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
                collect_fx(value, &mut fx_routes);
                exposure_values.push(value.clone());
            }
            holdings.push(holding_from_position(
                account,
                position,
                value,
                base_currency,
            ));
        }
        for order in &data.open_orders {
            open_orders.push(PortfolioOrder {
                connection_id: account.connection_id.clone(),
                account_label: account.label.clone(),
                broker_order_id: order.broker_order_id.clone(),
                instrument_id: market_instrument_id(&order.symbol, &account.provider_id),
                asset: order.symbol.clone(),
                side: order.side.clone(),
                quantity: order.quantity.clone(),
                notional: order.notional.clone(),
                currency: order
                    .currency
                    .clone()
                    .or_else(|| account_currency.map(str::to_owned)),
                status: order.status.clone(),
            });
        }
        portfolio_accounts.push(account_row(
            account,
            base_currency,
            equity,
            cash,
            data.positions.len(),
            data.open_orders.len(),
            0,
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
    } else {
        PortfolioStatus::Available
    };
    let reason = match status {
        PortfolioStatus::Available => {
            "Provider observations are shown with workspace-currency values where the route is trusted."
        }
        PortfolioStatus::Degraded => {
            "Some FX routes are unavailable or stale; affected workspace values remain unavailable."
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
            unrealized_pnl: unavailable_value(base_currency),
            realized_pnl: unavailable_value(base_currency),
            exposure: sum_values(&exposure_values, base_currency, time_status)?,
        },
        accounts: portfolio_accounts,
        holdings,
        open_orders,
        fills: None,
        fx_routes,
        live_risk: PortfolioLiveRisk {
            eligible: false,
            reason: "S09 is read-only; Live risk requires a later consumer to revalidate every account, market, time and FX gate.".into(),
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
            fills_count: 1,
        });
    }
    let mut holdings = vec![
        fixture_holding(
            FixtureHoldingSpec {
                connection_id: "fixture:t212-live",
                account_label: "Trading 212 Live (fixture)",
                provider_id: "trading212",
                environment: "LIVE",
                asset: "AAPL",
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
                asset: "BTC/USDT",
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
                asset: "MSFT",
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
            asset: "USDT",
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
            asset: "AAPL".into(),
            side: "BUY".into(),
            quantity: Some("10".into()),
            notional: Some("1800".into()),
            currency: Some("USD".into()),
            status: "PENDING".into(),
        },
        PortfolioOrder {
            connection_id: "fixture:binance-live".into(),
            account_label: "Binance Live (fixture)".into(),
            broker_order_id: "fixture-order-btc".into(),
            instrument_id: Some("crypto:BTC/USDT:spot".into()),
            asset: "BTC/USDT".into(),
            side: "SELL".into(),
            quantity: Some("0.05".into()),
            notional: None,
            currency: Some("USDT".into()),
            status: "NEW".into(),
        },
    ];
    let fills = Some(vec![PortfolioFill {
        connection_id: "fixture:t212-live".into(),
        account_label: "Trading 212 Live (fixture)".into(),
        fill_id: "fixture-fill-aapl".into(),
        instrument_id: Some("equity:US:AAPL".into()),
        asset: "AAPL".into(),
        quantity: "100".into(),
        value: fixture_value("16800", "USD", base_currency, time_status)?,
        observed_at: time_status.observed_at.clone(),
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
        collect_fx(value, &mut fx_routes);
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
        .filter(|holding| holding.asset != "USDT")
        .map(|holding| holding.value.clone())
        .collect();
    let degraded = fx_routes
        .iter()
        .any(|route| route.quality == FxQuality::Degraded);
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
            "Synthetic fixture includes a stablecoin quality warning; workspace analytics are degraded.".into()
        } else {
            "Synthetic fixture values are labelled for contract and rendering verification only."
                .into()
        },
        totals: PortfolioTotals {
            equity: sum_values(&equity_values, base_currency, time_status)?,
            cash: sum_values(&cash_values, base_currency, time_status)?,
            unrealized_pnl: fixture_value("3900", "USD", base_currency, time_status)?,
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
            reason:
                "Stablecoin quality is degraded in this fixture; S09 never authorizes Live risk."
                    .into(),
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
        reason: "Synthetic fixture account; no provider connection was made.".into(),
    }
}

struct FixtureHoldingSpec<'a> {
    connection_id: &'a str,
    account_label: &'a str,
    provider_id: &'a str,
    environment: &'a str,
    asset: &'a str,
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
        venue: Some(spec.provider_id.into()),
        instrument_id: spec.instrument_id.map(str::to_owned),
        asset: spec.asset.into(),
        quantity: Some(normalize_decimal(spec.quantity)?),
        value: fixture_value(spec.value, spec.currency, base_currency, time_status)?,
        unrealized_pnl: Some(fixture_value(
            spec.pnl,
            spec.currency,
            base_currency,
            time_status,
        )?),
    })
}

fn holding_from_balance(
    account: &AccountConnection,
    balance: &Balance,
    value: PortfolioValue,
) -> PortfolioHolding {
    PortfolioHolding {
        connection_id: account.connection_id.clone(),
        account_label: account.label.clone(),
        provider_id: account.provider_id.clone(),
        environment: account.environment.clone(),
        venue: Some(account.provider_id.clone()),
        instrument_id: None,
        asset: balance.asset.clone(),
        quantity: balance
            .total
            .clone()
            .or_else(|| Some(balance.available.clone())),
        value,
        unrealized_pnl: None,
    }
}

fn holding_from_position(
    account: &AccountConnection,
    position: &Position,
    value: Option<PortfolioValue>,
    base_currency: &str,
) -> PortfolioHolding {
    PortfolioHolding {
        connection_id: account.connection_id.clone(),
        account_label: account.label.clone(),
        provider_id: account.provider_id.clone(),
        environment: account.environment.clone(),
        venue: Some(account.provider_id.clone()),
        instrument_id: market_instrument_id(&position.symbol, &account.provider_id),
        asset: position.symbol.clone(),
        quantity: Some(position.quantity.clone()),
        value: value.unwrap_or_else(|| unavailable_value(base_currency)),
        unrealized_pnl: None,
    }
}

fn account_row(
    account: &AccountConnection,
    base_currency: &str,
    equity: Option<PortfolioValue>,
    cash: Option<PortfolioValue>,
    positions_count: usize,
    open_orders_count: usize,
    fills_count: usize,
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
        fills_count: fills_count as u32,
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

fn collect_fx(value: &PortfolioValue, routes: &mut Vec<FxProvenance>) {
    if let Some(route) = &value.fx_provenance
        && !routes.iter().any(|existing| existing == route)
    {
        routes.push(route.clone());
    }
}

fn sum_values(
    values: &[PortfolioValue],
    base_currency: &str,
    time_status: &TimeStatus,
) -> Result<PortfolioValue> {
    let mut total: Option<String> = None;
    let mut route: Option<FxProvenance> = None;
    for value in values {
        let Some(workspace_value) = value.workspace_value.as_deref() else {
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
        workspace_value: total,
        workspace_currency: base_currency.into(),
        fx_provenance: route
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

fn market_instrument_id(symbol: &str, provider: &str) -> Option<String> {
    market::instruments().into_iter().find_map(|instrument| {
        (instrument.symbol == symbol
            || instrument.providers.iter().any(|mapping| {
                mapping.provider_id == provider && mapping.provider_symbol == symbol
            }))
        .then_some(instrument.instrument_id)
    })
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

fn normalize_decimal(value: &str) -> Result<String> {
    crate::provider_io::decimal(&Value::String(value.into()))
}

fn decimal_add(left: &str, right: &str) -> Result<String> {
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

fn decimal_mul(left: &str, right: &str) -> Result<String> {
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
        assert!(!snapshot.live_risk.eligible);
    }

    #[test]
    fn empty_production_snapshot_is_unavailable_without_mutation() {
        let status = time_status();
        let snapshot = get("w", "EUR", &[], None, &status, false).unwrap();
        assert_eq!(snapshot.status, PortfolioStatus::Unavailable);
        assert!(snapshot.totals.equity.workspace_value.is_none());
        assert!(snapshot.fills.is_none());
    }
}
