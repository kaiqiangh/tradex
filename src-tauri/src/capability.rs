use crate::protocol::{AgentMode, ExecutionContext, Result, ThreadContextRef, TradeXError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityDecision {
    pub level: CapabilityLevel,
    pub allowed_tools: Vec<ToolId>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountContext {
    pub provider_id: String,
    pub environment: String,
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
    match mode {
        AgentMode::Ask | AgentMode::Research => {
            validate_read_context(context, account)?;
            let mut tools = vec![ToolId::PublicMarketRead];
            if account.is_some() {
                tools.push(ToolId::AccountRead);
            }
            Ok(CapabilityDecision {
                level: if account.is_some() {
                    CapabilityLevel::C1
                } else {
                    CapabilityLevel::C0
                },
                allowed_tools: tools,
                execution_allowed: false,
                reason: if is_live(context) {
                    Some("LIVE_READ_ONLY".into())
                } else {
                    None
                },
            })
        }
        AgentMode::Backtest => {
            validate_backtest_context(context, account)?;
            let mut tools = vec![ToolId::HistoricalSimulation];
            if account.is_some() {
                tools.push(ToolId::AccountRead);
            }
            Ok(CapabilityDecision {
                level: CapabilityLevel::C2,
                allowed_tools: tools,
                execution_allowed: false,
                reason: Some("HISTORICAL_SIMULATION_ONLY".into()),
            })
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
                    return Ok(CapabilityDecision {
                        level: CapabilityLevel::C3,
                        allowed_tools: vec![
                            ToolId::PublicMarketRead,
                            ToolId::PaperDemoTestnetExecution,
                        ],
                        execution_allowed: true,
                        reason: Some("LOCAL_PAPER_SIMULATION".into()),
                    });
                }
                return Err(TradeXError::new("TURN_ACCOUNT_REQUIRED"));
            };
            validate_account_environment(context, account)?;
            let live = is_live(context);
            Ok(CapabilityDecision {
                level: if live {
                    CapabilityLevel::C4
                } else {
                    CapabilityLevel::C3
                },
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
            })
        }
    }
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
            || !seen.insert((&context.kind, &context.id, &context.hash))
    }) {
        return Err(TradeXError::new("TURN_CONTEXT_INVALID"));
    }
    Ok(())
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
                hash: "sha256:a".into(),
            },
            ThreadContextRef {
                kind: "artifact".into(),
                id: "a".into(),
                hash: "sha256:a".into(),
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
            hash: "sha256:x".into(),
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
}
