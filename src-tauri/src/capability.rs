use crate::protocol::{AgentMode, ExecutionContext, Result, ThreadContextRef, TradeXError};
use crate::providers::{AccountConnection, ConnectionState};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn present<'de, D, T>(value: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(value).map(Some)
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityLevel {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
}

impl CapabilityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::C0 => "C0",
            Self::C1 => "C1",
            Self::C2 => "C2",
            Self::C3 => "C3",
            Self::C4 => "C4",
            Self::C5 => "C5",
            Self::C6 => "C6",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolId {
    PublicMarketRead,
    AccountRead,
    HistoricalSimulation,
    PaperDemoTestnetExecution,
    LiveOrderProposal,
}

/// Data-plane tools are intentionally separate from financial authority IDs.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchToolId {
    PublicMarketRead,
    AccountRead,
    HistoricalSimulation,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchToolDefinition {
    pub id: ResearchToolId,
    #[schemars(length(min = 1, max = 120))]
    pub label: String,
    pub read_only: bool,
    #[schemars(length(min = 1, max = 256))]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityDecision {
    pub level: CapabilityLevel,
    pub allowed_tools: Vec<ToolId>,
    #[schemars(length(max = 3))]
    pub research_tools: Vec<ResearchToolDefinition>,
    pub execution_allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    pub agent_mode: AgentMode,
    pub execution_context: ExecutionContext,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", length(min = 1, max = 128))]
    pub account_id: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 32))]
    pub attached_contexts: Vec<ThreadContextRef>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String", length(min = 1, max = 64))]
    pub requested_tool: Option<String>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "CapabilityLevel")]
    pub requested_level: Option<CapabilityLevel>,
}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextCatalog {
    #[schemars(length(max = 256))]
    pub entries: Vec<ContextCatalogEntry>,
    #[schemars(length(max = 5))]
    pub empty_states: Vec<ContextCatalogEmptyState>,
}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextCatalogEntry {
    pub context_ref: ThreadContextRef,
    #[schemars(length(min = 1, max = 120))]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 32))]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 16))]
    pub environment: Option<String>,
    pub read_only: bool,
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "String", length(min = 1, max = 256))]
    pub availability_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextCatalogEmptyState {
    #[schemars(extend("enum" = ["instrument", "account", "strategy", "backtest", "artifact"]))]
    pub kind: String,
    #[schemars(length(min = 1, max = 256))]
    pub availability_reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountContext {
    pub provider_id: String,
    pub environment: String,
}

pub const MAX_CONTEXT_CATALOG_ENTRIES: usize = 256;

pub fn context_catalog(accounts: &[AccountConnection]) -> Result<ContextCatalog> {
    if accounts.len() > MAX_CONTEXT_CATALOG_ENTRIES {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let mut entries = Vec::with_capacity(accounts.len());
    for account in accounts {
        let (available, availability_reason) = account_availability(account);
        entries.push(ContextCatalogEntry {
            context_ref: account_context_ref(account),
            label: account.label.clone(),
            provider_id: Some(account.provider_id.clone()),
            environment: Some(account.environment.clone()),
            read_only: true,
            available,
            availability_reason,
        });
    }
    let mut empty_states = [
        (
            "instrument",
            "No instrument catalog is connected yet; S07 will populate it.",
        ),
        (
            "strategy",
            "No saved strategies are available yet; S14 will populate them.",
        ),
        (
            "backtest",
            "No backtest runs are available yet; S15 will populate them.",
        ),
        (
            "artifact",
            "No research artifacts are available yet; S12 will populate them.",
        ),
    ]
    .into_iter()
    .map(|(kind, availability_reason)| ContextCatalogEmptyState {
        kind: kind.into(),
        availability_reason: availability_reason.into(),
    })
    .collect::<Vec<_>>();
    if accounts.is_empty() {
        empty_states.insert(
            0,
            ContextCatalogEmptyState {
                kind: "account".into(),
                availability_reason:
                    "No persisted account connections are available in this workspace.".into(),
            },
        );
    }
    Ok(ContextCatalog {
        entries,
        empty_states,
    })
}

pub fn account_available(account: &AccountConnection) -> bool {
    account_availability(account).0
}

pub fn validate_catalog_refs(
    workspace_id: &str,
    contexts: &[ThreadContextRef],
    accounts: &[AccountConnection],
) -> Result<()> {
    validate_contexts(contexts)?;
    for context in contexts.iter().filter(|context| context.kind == "account") {
        let Some(account) = accounts
            .iter()
            .find(|account| account.connection_id == context.id)
        else {
            return Err(TradeXError::new("TURN_CONTEXT_INVALID"));
        };
        if account.workspace_id != workspace_id
            || account_context_ref(account).hash != context.hash
            || !account_available(account)
        {
            return Err(TradeXError::new("TURN_CONTEXT_INVALID"));
        }
    }
    Ok(())
}

pub fn account_context_ref(account: &AccountConnection) -> ThreadContextRef {
    let material = format!(
        "{}\0{}\0{}\0{}\0{}",
        account.workspace_id,
        account.connection_id,
        account.provider_id,
        account.environment,
        account.created_at,
    );
    let digest = Sha256::digest(material.as_bytes());
    ThreadContextRef {
        kind: "account".into(),
        id: account.connection_id.clone(),
        hash: format!("sha256:{}", hex::encode(digest)),
    }
}

fn account_availability(account: &AccountConnection) -> (bool, Option<String>) {
    if account.health.credential == "MISSING" {
        return (
            false,
            Some("The local credential is unavailable; reconnect this account.".into()),
        );
    }
    if account.health.credential == "DELETE_PENDING" {
        return (
            false,
            Some("Local credential cleanup is pending; remove or reconnect this account.".into()),
        );
    }
    match &account.connection_state {
        ConnectionState::Connected
            if account.health.connection == "ONLINE"
                && account.health.authentication == "VALID"
                && account.health.credential == "CONFIGURED" =>
        {
            (true, None)
        }
        ConnectionState::Connected => (
            false,
            Some("Account health is stale; re-test it before attaching this context.".into()),
        ),
        ConnectionState::Connecting => (
            false,
            Some("Account testing is still in progress; try again when it is connected.".into()),
        ),
        ConnectionState::ReviewRequired => (
            false,
            Some("Complete the account permission review before attaching it.".into()),
        ),
        ConnectionState::Failed => (
            false,
            Some("The last account test failed; reconnect before attaching it.".into()),
        ),
        ConnectionState::Disconnected => (
            false,
            Some("This account is disconnected; reconnect before attaching it.".into()),
        ),
    }
}

pub fn decide(
    mode: &AgentMode,
    context: &ExecutionContext,
    account: Option<&AccountContext>,
    attached_contexts: &[ThreadContextRef],
) -> Result<CapabilityDecision> {
    decide_policy(mode, context, account, attached_contexts)
}

pub fn decide_query(
    query: &CapabilityQuery,
    account: Option<&AccountContext>,
) -> Result<CapabilityDecision> {
    let decision = decide_policy(
        &query.agent_mode,
        &query.execution_context,
        account,
        &query.attached_contexts,
    )?;
    if let Some(level) = &query.requested_level
        && matches!(level, CapabilityLevel::C5 | CapabilityLevel::C6)
    {
        return Err(TradeXError::new("UNSUPPORTED_CAPABILITY"));
    }
    if let Some(tool) = query.requested_tool.as_deref() {
        if tool.trim().is_empty() || tool.chars().count() > 64 || tool.chars().any(char::is_control)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        let allowed = decision
            .allowed_tools
            .iter()
            .any(|candidate| tool_id(candidate) == tool);
        if !allowed {
            return Err(TradeXError::new("UNSUPPORTED_CAPABILITY"));
        }
    }
    if let Some(level) = &query.requested_level
        && level != &decision.level
    {
        return Err(TradeXError::new("UNSUPPORTED_CAPABILITY"));
    }
    Ok(decision)
}

fn decide_policy(
    mode: &AgentMode,
    context: &ExecutionContext,
    account: Option<&AccountContext>,
    attached_contexts: &[ThreadContextRef],
) -> Result<CapabilityDecision> {
    validate_contexts(attached_contexts)?;
    let attached_account = attached_contexts
        .iter()
        .any(|context| context.kind == "account");
    match mode {
        AgentMode::Ask | AgentMode::Research => {
            validate_read_context(context, account)?;
            let mut tools = vec![ToolId::PublicMarketRead];
            if account.is_some() || attached_account {
                tools.push(ToolId::AccountRead);
            }
            Ok(with_research_tools(CapabilityDecision {
                level: if account.is_some() || attached_account {
                    CapabilityLevel::C1
                } else {
                    CapabilityLevel::C0
                },
                research_tools: Vec::new(),
                allowed_tools: tools,
                execution_allowed: false,
                reason: if is_live(context) {
                    Some("LIVE_READ_ONLY".into())
                } else {
                    None
                },
            }))
        }
        AgentMode::Backtest => {
            validate_backtest_context(context, account)?;
            let mut tools = vec![ToolId::HistoricalSimulation];
            if account.is_some() || attached_account {
                tools.push(ToolId::AccountRead);
            }
            Ok(with_research_tools(CapabilityDecision {
                level: CapabilityLevel::C2,
                research_tools: Vec::new(),
                allowed_tools: tools,
                execution_allowed: false,
                reason: Some("HISTORICAL_SIMULATION_ONLY".into()),
            }))
        }
        AgentMode::Trade => {
            if matches!(
                context,
                ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation
            ) {
                return Err(TradeXError::new("TURN_CONTEXT_INVALID"));
            }
            let Some(account) = account else {
                if matches!(context, ExecutionContext::LocalPaper) {
                    return Ok(with_research_tools(CapabilityDecision {
                        level: CapabilityLevel::C3,
                        research_tools: Vec::new(),
                        allowed_tools: vec![
                            ToolId::PublicMarketRead,
                            ToolId::PaperDemoTestnetExecution,
                        ],
                        execution_allowed: true,
                        reason: Some("LOCAL_PAPER_SIMULATION".into()),
                    }));
                }
                return Err(TradeXError::new("TURN_ACCOUNT_REQUIRED"));
            };
            validate_account_environment(context, account)?;
            let live = is_live(context);
            Ok(with_research_tools(CapabilityDecision {
                level: if live {
                    CapabilityLevel::C4
                } else {
                    CapabilityLevel::C3
                },
                research_tools: Vec::new(),
                allowed_tools: if live {
                    vec![
                        ToolId::PublicMarketRead,
                        ToolId::AccountRead,
                        ToolId::LiveOrderProposal,
                    ]
                } else {
                    vec![
                        ToolId::PublicMarketRead,
                        ToolId::AccountRead,
                        ToolId::PaperDemoTestnetExecution,
                    ]
                },
                execution_allowed: !live,
                reason: Some(
                    if live {
                        "LIVE_PROPOSAL_REQUIRES_ARMING_APPROVAL"
                    } else {
                        "NON_LIVE_EXECUTION"
                    }
                    .into(),
                ),
            }))
        }
    }
}

fn with_research_tools(mut decision: CapabilityDecision) -> CapabilityDecision {
    decision.research_tools = decision
        .allowed_tools
        .iter()
        .filter_map(|tool| match tool {
            ToolId::PublicMarketRead => Some(ResearchToolDefinition {
                id: ResearchToolId::PublicMarketRead,
                label: "Public market read".into(),
                read_only: true,
                description: "Read-only public market context; no order or broker mutation.".into(),
            }),
            ToolId::AccountRead => Some(ResearchToolDefinition {
                id: ResearchToolId::AccountRead,
                label: "Account read".into(),
                read_only: true,
                description: "Read-only account context; credentials stay in native storage."
                    .into(),
            }),
            ToolId::HistoricalSimulation => Some(ResearchToolDefinition {
                id: ResearchToolId::HistoricalSimulation,
                label: "Historical simulation".into(),
                read_only: true,
                description:
                    "Historical-only simulation context; current-market execution is unavailable."
                        .into(),
            }),
            ToolId::PaperDemoTestnetExecution | ToolId::LiveOrderProposal => None,
        })
        .collect();
    decision
}

fn tool_id(tool: &ToolId) -> &'static str {
    match tool {
        ToolId::PublicMarketRead => "public_market_read",
        ToolId::AccountRead => "account_read",
        ToolId::HistoricalSimulation => "historical_simulation",
        ToolId::PaperDemoTestnetExecution => "paper_demo_testnet_execution",
        ToolId::LiveOrderProposal => "live_order_proposal",
    }
}

fn validate_read_context(
    context: &ExecutionContext,
    account: Option<&AccountContext>,
) -> Result<()> {
    if matches!(
        context,
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation
    ) {
        return Ok(());
    }
    let Some(account) = account else {
        return Err(TradeXError::new("TURN_ACCOUNT_REQUIRED"));
    };
    validate_account_environment(context, account)
}

fn validate_backtest_context(
    context: &ExecutionContext,
    account: Option<&AccountContext>,
) -> Result<()> {
    if matches!(context, ExecutionContext::HistoricalSimulation) {
        return Ok(());
    }
    if matches!(context, ExecutionContext::NoneReadOnly) {
        return Err(TradeXError::new("TURN_CONTEXT_INVALID"));
    }
    let Some(account) = account else {
        return Err(TradeXError::new("TURN_ACCOUNT_REQUIRED"));
    };
    validate_account_environment(context, account)
}

fn validate_account_environment(
    context: &ExecutionContext,
    account: &AccountContext,
) -> Result<()> {
    let expected = match context {
        ExecutionContext::AlpacaPaper => Some(("alpaca", "PAPER")),
        ExecutionContext::Trading212Demo => Some(("trading212", "DEMO")),
        ExecutionContext::Trading212Live => Some(("trading212", "LIVE")),
        ExecutionContext::BinanceTestnet => Some(("binance", "TESTNET")),
        ExecutionContext::BinanceLive => Some(("binance", "LIVE")),
        ExecutionContext::BitgetDemo => Some(("bitget", "DEMO")),
        ExecutionContext::BitgetLive => Some(("bitget", "LIVE")),
        ExecutionContext::NoneReadOnly | ExecutionContext::HistoricalSimulation => None,
        ExecutionContext::LocalPaper => return Err(TradeXError::new("TURN_ACCOUNT_INVALID")),
    };
    if let Some((provider, environment)) = expected
        && (account.provider_id != provider || account.environment != environment)
    {
        return Err(TradeXError::new("TURN_ACCOUNT_INVALID"));
    }
    Ok(())
}

fn is_live(context: &ExecutionContext) -> bool {
    matches!(
        context,
        ExecutionContext::Trading212Live
            | ExecutionContext::BinanceLive
            | ExecutionContext::BitgetLive
    )
}

pub fn validate_contexts(contexts: &[ThreadContextRef]) -> Result<()> {
    if contexts.len() > 32 {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    let mut seen = std::collections::HashSet::new();
    if contexts.iter().any(|context| {
        !matches!(
            context.kind.as_str(),
            "instrument" | "account" | "strategy" | "backtest" | "artifact"
        ) || context.id.trim().is_empty()
            || context.hash.trim().is_empty()
            || context.kind.chars().count() > 64
            || context.id.chars().count() > 256
            || context.hash.chars().count() > 256
            || context.kind.chars().any(char::is_control)
            || context.id.chars().any(char::is_control)
            || context.hash.chars().any(char::is_control)
            || !valid_hash(&context.hash)
            || !seen.insert((&context.kind, &context.id, &context.hash))
    }) {
        return Err(TradeXError::new("TURN_CONTEXT_INVALID"));
    }
    Ok(())
}

fn valid_hash(hash: &str) -> bool {
    let Some(digest) = hash.strip_prefix("sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(provider_id: &str, environment: &str) -> AccountContext {
        AccountContext {
            provider_id: provider_id.into(),
            environment: environment.into(),
        }
    }

    #[test]
    fn mode_context_matrix_keeps_live_proposal_below_execution() {
        let ask = decide(
            &AgentMode::Ask,
            &ExecutionContext::Trading212Live,
            Some(&account("trading212", "LIVE")),
            &[],
        )
        .unwrap();
        assert_eq!(ask.level, CapabilityLevel::C1);
        assert!(!ask.execution_allowed);
        assert_eq!(ask.reason.as_deref(), Some("LIVE_READ_ONLY"));

        let backtest = decide(
            &AgentMode::Backtest,
            &ExecutionContext::HistoricalSimulation,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(backtest.level, CapabilityLevel::C2);
        assert!(!backtest.execution_allowed);
        assert_eq!(backtest.allowed_tools, vec![ToolId::HistoricalSimulation]);
        assert_eq!(
            backtest
                .research_tools
                .iter()
                .map(|tool| tool.id.clone())
                .collect::<Vec<_>>(),
            vec![ResearchToolId::HistoricalSimulation]
        );

        let live = decide(
            &AgentMode::Trade,
            &ExecutionContext::Trading212Live,
            Some(&account("trading212", "LIVE")),
            &[],
        )
        .unwrap();
        assert_eq!(live.level, CapabilityLevel::C4);
        assert!(!live.execution_allowed);
        assert!(live.allowed_tools.contains(&ToolId::LiveOrderProposal));
        assert!(live.research_tools.iter().all(|tool| {
            matches!(
                tool.id,
                ResearchToolId::PublicMarketRead | ResearchToolId::AccountRead
            )
        }));
    }

    #[test]
    fn every_supported_environment_keeps_mode_boundaries() {
        let provider_contexts = [
            (ExecutionContext::AlpacaPaper, "alpaca", "PAPER"),
            (ExecutionContext::Trading212Demo, "trading212", "DEMO"),
            (ExecutionContext::Trading212Live, "trading212", "LIVE"),
            (ExecutionContext::BinanceTestnet, "binance", "TESTNET"),
            (ExecutionContext::BinanceLive, "binance", "LIVE"),
            (ExecutionContext::BitgetDemo, "bitget", "DEMO"),
            (ExecutionContext::BitgetLive, "bitget", "LIVE"),
        ];
        for mode in [AgentMode::Ask, AgentMode::Research] {
            assert_eq!(
                decide(&mode, &ExecutionContext::NoneReadOnly, None, &[])
                    .unwrap()
                    .level,
                CapabilityLevel::C0
            );
            for (context, provider, environment) in &provider_contexts {
                let account = account(provider, environment);
                let decision = decide(&mode, context, Some(&account), &[]).unwrap();
                assert_eq!(decision.level, CapabilityLevel::C1);
                assert!(!decision.execution_allowed);
                assert!(
                    !decision
                        .allowed_tools
                        .contains(&ToolId::PaperDemoTestnetExecution)
                );
                assert!(decide(&mode, context, None, &[]).is_err());
            }
        }

        let historical = decide(
            &AgentMode::Backtest,
            &ExecutionContext::HistoricalSimulation,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(historical.level, CapabilityLevel::C2);
        assert!(!historical.execution_allowed);
        for (context, provider, environment) in &provider_contexts {
            let account = account(provider, environment);
            let decision = decide(&AgentMode::Backtest, context, Some(&account), &[]).unwrap();
            assert_eq!(decision.level, CapabilityLevel::C2);
            assert!(!decision.execution_allowed);
            assert_eq!(
                decision.allowed_tools,
                vec![ToolId::HistoricalSimulation, ToolId::AccountRead]
            );
            assert!(decide(&AgentMode::Backtest, context, None, &[]).is_err());
        }

        let local_paper =
            decide(&AgentMode::Trade, &ExecutionContext::LocalPaper, None, &[]).unwrap();
        assert_eq!(local_paper.level, CapabilityLevel::C3);
        assert!(local_paper.execution_allowed);
        for (context, provider, environment) in &provider_contexts {
            let account = account(provider, environment);
            let decision = decide(&AgentMode::Trade, context, Some(&account), &[]);
            if matches!(
                context,
                ExecutionContext::Trading212Live
                    | ExecutionContext::BinanceLive
                    | ExecutionContext::BitgetLive
            ) {
                let decision = decision.unwrap();
                assert_eq!(decision.level, CapabilityLevel::C4);
                assert!(!decision.execution_allowed);
                assert!(decision.allowed_tools.contains(&ToolId::LiveOrderProposal));
            } else {
                let decision = decision.unwrap();
                assert_eq!(decision.level, CapabilityLevel::C3);
                assert!(decision.execution_allowed);
                assert!(
                    decision
                        .allowed_tools
                        .contains(&ToolId::PaperDemoTestnetExecution)
                );
            }
            assert!(decide(&AgentMode::Trade, context, None, &[]).is_err());
        }
    }

    #[test]
    fn invalid_matrix_rows_fail_closed() {
        assert_eq!(
            decide(
                &AgentMode::Trade,
                &ExecutionContext::NoneReadOnly,
                None,
                &[]
            )
            .unwrap_err()
            .code,
            "TURN_CONTEXT_INVALID"
        );
        assert_eq!(
            decide(
                &AgentMode::Trade,
                &ExecutionContext::Trading212Live,
                Some(&account("binance", "LIVE")),
                &[],
            )
            .unwrap_err()
            .code,
            "TURN_ACCOUNT_INVALID"
        );
        assert_eq!(
            decide(
                &AgentMode::Backtest,
                &ExecutionContext::NoneReadOnly,
                None,
                &[]
            )
            .unwrap_err()
            .code,
            "TURN_CONTEXT_INVALID"
        );
    }

    #[test]
    fn duplicate_or_unknown_context_refs_are_rejected() {
        let refs = vec![
            ThreadContextRef {
                kind: "artifact".into(),
                id: "a".into(),
                hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
            },
            ThreadContextRef {
                kind: "artifact".into(),
                id: "a".into(),
                hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
            },
        ];
        assert_eq!(
            decide(
                &AgentMode::Ask,
                &ExecutionContext::NoneReadOnly,
                None,
                &refs
            )
            .unwrap_err()
            .code,
            "TURN_CONTEXT_INVALID"
        );
        let unknown = [ThreadContextRef {
            kind: "prompt".into(),
            id: "x".into(),
            hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        }];
        assert_eq!(
            decide(
                &AgentMode::Ask,
                &ExecutionContext::NoneReadOnly,
                None,
                &unknown
            )
            .unwrap_err()
            .code,
            "TURN_CONTEXT_INVALID"
        );
    }

    #[test]
    fn requested_capabilities_fail_closed_for_unknown_or_privileged_grants() {
        let base = CapabilityQuery {
            workspace_id: "workspace".into(),
            agent_mode: AgentMode::Ask,
            execution_context: ExecutionContext::NoneReadOnly,
            account_id: None,
            attached_contexts: vec![],
            requested_tool: Some("order.submit".into()),
            requested_level: None,
        };
        assert_eq!(
            decide_query(&base, None).unwrap_err().code,
            "UNSUPPORTED_CAPABILITY"
        );
        for level in [CapabilityLevel::C5, CapabilityLevel::C6] {
            let query = CapabilityQuery {
                requested_tool: None,
                requested_level: Some(level),
                ..base_without_request()
            };
            assert_eq!(
                decide_query(&query, None).unwrap_err().code,
                "UNSUPPORTED_CAPABILITY"
            );
        }
        let allowed = CapabilityQuery {
            requested_tool: Some("public_market_read".into()),
            requested_level: Some(CapabilityLevel::C0),
            ..base_without_request()
        };
        assert_eq!(
            decide_query(&allowed, None).unwrap().level,
            CapabilityLevel::C0
        );
    }

    #[test]
    fn capability_query_rejects_nulls_and_unbounded_tool_probes() {
        let base = serde_json::json!({
            "workspaceId": "workspace",
            "agentMode": "ASK",
            "executionContext": "NONE_READ_ONLY",
            "attachedContexts": []
        });
        for field in ["accountId", "requestedTool", "requestedLevel"] {
            let mut value = base.clone();
            value[field] = serde_json::Value::Null;
            assert!(serde_json::from_value::<CapabilityQuery>(value).is_err());
        }

        for requested_tool in [" ".into(), "x".repeat(65), "bad\u{0000}".into()] {
            let query = CapabilityQuery {
                requested_tool: Some(requested_tool),
                ..base_without_request()
            };
            assert_eq!(
                decide_query(&query, None).unwrap_err().code,
                "IPC_PAYLOAD_INVALID"
            );
        }
    }

    fn base_without_request() -> CapabilityQuery {
        CapabilityQuery {
            workspace_id: "workspace".into(),
            agent_mode: AgentMode::Ask,
            execution_context: ExecutionContext::NoneReadOnly,
            account_id: None,
            attached_contexts: vec![],
            requested_tool: None,
            requested_level: None,
        }
    }

    fn connected_account() -> AccountConnection {
        let mut account = AccountConnection::new(
            "workspace".into(),
            "alpaca".into(),
            "PAPER".into(),
            "Research account".into(),
        )
        .unwrap();
        account.connection_state = ConnectionState::Connected;
        account.health.connection = "ONLINE".into();
        account.health.authentication = "VALID".into();
        account.health.credential = "CONFIGURED".into();
        account
    }

    #[test]
    fn account_context_ref_is_stable_and_secret_free() {
        let account = connected_account();
        let reference = account_context_ref(&account);
        assert_eq!(reference.kind, "account");
        assert_eq!(reference.id, account.connection_id);
        assert!(reference.hash.starts_with("sha256:"));
        assert_eq!(reference.hash.len(), 71);
        assert!(!reference.hash.contains(&account.label));

        let mut renamed = account.clone();
        renamed.label = "Renamed account".into();
        assert_eq!(account_context_ref(&account), account_context_ref(&renamed));

        let mut moved = account.clone();
        moved.environment = "LIVE".into();
        assert_ne!(account_context_ref(&account), account_context_ref(&moved));
    }

    #[test]
    fn context_catalog_lists_accounts_and_future_empty_states() {
        let account = connected_account();
        let catalog = context_catalog(std::slice::from_ref(&account)).unwrap();
        assert_eq!(catalog.entries.len(), 1);
        let entry = &catalog.entries[0];
        assert_eq!(entry.context_ref, account_context_ref(&account));
        assert_eq!(entry.label, "Research account");
        assert_eq!(entry.provider_id.as_deref(), Some("alpaca"));
        assert_eq!(entry.environment.as_deref(), Some("PAPER"));
        assert!(entry.read_only);
        assert!(entry.available);
        assert_eq!(catalog.empty_states.len(), 4);
        assert!(
            catalog
                .empty_states
                .iter()
                .all(|state| state.kind != "account")
        );

        let empty = context_catalog(&[]).unwrap();
        assert_eq!(empty.entries.len(), 0);
        assert!(
            empty
                .empty_states
                .iter()
                .any(|state| state.kind == "account")
        );
    }

    #[test]
    fn context_catalog_keeps_stale_and_cleanup_accounts_unavailable() {
        let mut stale = connected_account();
        stale.health.connection = "STALE".into();
        stale.health.authentication = "UNVERIFIED".into();
        stale.health.credential = "UNCHECKED".into();
        let stale_catalog = context_catalog(std::slice::from_ref(&stale)).unwrap();
        let stale_entry = &stale_catalog.entries[0];
        assert!(!stale_entry.available);
        assert!(
            stale_entry
                .availability_reason
                .as_deref()
                .unwrap()
                .contains("stale")
        );

        let mut cleanup = connected_account();
        cleanup.health.credential = "DELETE_PENDING".into();
        let cleanup_catalog = context_catalog(std::slice::from_ref(&cleanup)).unwrap();
        let cleanup_entry = &cleanup_catalog.entries[0];
        assert!(!cleanup_entry.available);
        assert!(
            cleanup_entry
                .availability_reason
                .as_deref()
                .unwrap()
                .contains("cleanup")
        );
    }

    #[test]
    fn context_catalog_rejects_unbounded_account_lists() {
        let accounts = vec![connected_account(); MAX_CONTEXT_CATALOG_ENTRIES + 1];
        assert_eq!(
            context_catalog(&accounts).unwrap_err().code,
            "IPC_PAYLOAD_INVALID"
        );
    }

    #[test]
    fn attached_account_context_adds_read_capability_without_trade_authority() {
        let account = connected_account();
        let context = account_context_ref(&account);
        let ask = decide(
            &AgentMode::Ask,
            &ExecutionContext::NoneReadOnly,
            None,
            std::slice::from_ref(&context),
        )
        .unwrap();
        assert_eq!(ask.level, CapabilityLevel::C1);
        assert!(ask.allowed_tools.contains(&ToolId::AccountRead));
        assert!(!ask.execution_allowed);

        let trade = decide(
            &AgentMode::Trade,
            &ExecutionContext::AlpacaPaper,
            None,
            std::slice::from_ref(&context),
        )
        .unwrap_err();
        assert_eq!(trade.code, "TURN_ACCOUNT_REQUIRED");
    }

    #[test]
    fn catalog_ref_validation_rejects_malformed_duplicate_and_unknown_refs() {
        let account = connected_account();
        let reference = account_context_ref(&account);
        assert_eq!(
            validate_catalog_refs(
                "workspace",
                &[reference.clone(), reference.clone()],
                std::slice::from_ref(&account)
            )
            .unwrap_err()
            .code,
            "TURN_CONTEXT_INVALID"
        );
        let mut malformed = reference.clone();
        malformed.hash = "sha256:abc".into();
        assert_eq!(
            validate_catalog_refs("workspace", &[malformed], std::slice::from_ref(&account))
                .unwrap_err()
                .code,
            "TURN_CONTEXT_INVALID"
        );
        let mut uppercase = reference.clone();
        uppercase.hash = uppercase
            .hash
            .to_uppercase()
            .replacen("SHA256:", "sha256:", 1);
        assert!(!valid_hash(&uppercase.hash));
        assert_eq!(
            validate_catalog_refs("workspace", &[uppercase], std::slice::from_ref(&account))
                .unwrap_err()
                .code,
            "TURN_CONTEXT_INVALID"
        );
        let unknown = ThreadContextRef {
            kind: "account".into(),
            id: "missing-account".into(),
            hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        };
        assert_eq!(
            validate_catalog_refs("workspace", &[unknown], std::slice::from_ref(&account))
                .unwrap_err()
                .code,
            "TURN_CONTEXT_INVALID"
        );
        let unsupported = ThreadContextRef {
            kind: "unknown".into(),
            id: "x".into(),
            hash: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
        };
        assert_eq!(
            validate_catalog_refs("workspace", &[unsupported], std::slice::from_ref(&account))
                .unwrap_err()
                .code,
            "TURN_CONTEXT_INVALID"
        );
    }
}
