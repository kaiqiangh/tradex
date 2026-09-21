use crate::protocol::{
    LocalPaperBalance, LocalPaperMoney, LocalPaperProfile, LocalPaperState, Result, TradeXError,
};
use crate::providers::{
    AccountConnection, AccountData, AccountHealth, Balance, ConnectionState, PermissionReview,
};

pub const PROVIDER_ID: &str = "local-paper";
pub const ENVIRONMENT: &str = "LOCAL";
pub const ACCOUNT_TYPE: &str = "TRADEX_SIMULATION";
pub const DEFAULT_STARTING_CASH: &str = "100000";
pub const QUOTE_SOURCE: &str = "LOCAL_DETERMINISTIC";
pub const SCENARIO_ID: &str = "default-v1";
pub const ENGINE_VERSION: &str = "s16-v1";
pub const DISCLOSURE: &str =
    "TRADEX_SIMULATION; TradeX simulation; not provider truth; not Live execution.";

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
        positions: vec![],
        open_orders: vec![],
        fills: vec![],
        event_cursor: 0,
        state_version: format!("paper:{workspace_id}:1"),
        updated_at,
        disclosure: DISCLOSURE.into(),
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
    }
}
