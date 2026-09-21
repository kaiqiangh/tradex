import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import type { Artifact, ArtifactExport, ArtifactExportResult, ArtifactLibrary, ArtifactQuery, ArtifactSave, ChatgptLogin, ConfigureDeepseek, GatewayMutation, GatewayState, DomainEvent, AccountConnection, AccountMutation, AccountQuery, Accounts, Connect, ModelState, ModelQuery, PermissionReview, ProviderCatalog, ProviderDefinition, ProviderSelection, SetDefaultModel, SetFallbackPolicy, CompleteOnboarding, RiskPolicyState, RiskQuery, SaveRiskPolicy, SetOnboardingStep, VerifyRoute, WorkspaceQuery, Aggregate, EmptyPayload, OpenWorkspace, ResultEnvelope, RuntimeStatus, Snapshot, Subscribe, SubscriptionAck, TradeXError, Workspace, Thread, ThreadCreate, ThreadList, ThreadQuery, TurnCancel, TurnRetry, TurnStart, CapabilityDecision, CapabilityQuery, ContextCatalog, ResearchToolRequest, ResearchToolResult, DataSourceCatalog, DataSourceProbe, MarketCatalogQuery, MarketCatalog, MarketDetail, MarketGetQuery, PortfolioQuery, PortfolioSnapshot, Watchlist, Watchlists, WatchlistCreate, WatchlistRename, WatchlistDelete, WatchlistInstrumentMutation, TimeStatus, ScreenerRequest, ScreenerResult, ScreenerAttach, ScreenerAttachment, ScreenerLibrary, ScreenerSave, ScreenerUpdate, OrderDraft, OrderDraftLibrary, OrderDraftQuery, OrderDraftSave, OrderProposal, OrderProposalGenerate, OrderProposalQuery, OrderProposalLibrary, OrderProposalRefresh, OrderProposalRefreshResult, StrategyLibrary, StrategyQuery, StrategyRun, StrategyRunQuery, StrategyRunRequest, StrategySave, StrategyVersion, StrategyCancel, BacktestComparison, BacktestLibrary, BacktestRun, BacktestRunQuery, BacktestRunRequest, BacktestCompareRequest, BacktestCancel } from '../shared/ipc-types.ts';
import { decode } from './projection.ts';

interface Inputs {
  'model.get_gateway': WorkspaceQuery;
  'model.gateway': GatewayMutation;
  'model.get': ModelQuery;
  'model.login_chatgpt': ChatgptLogin;
  'model.configure_deepseek': ConfigureDeepseek;
  'model.verify_route': VerifyRoute;
  'model.set_default': SetDefaultModel;
  'model.set_fallback_policy': SetFallbackPolicy;
  'risk.get_policy': RiskQuery;
  'risk.save_policy': SaveRiskPolicy;
  'onboarding.set_step': SetOnboardingStep;
  'onboarding.complete': CompleteOnboarding;
  'provider.list_definitions': EmptyPayload;
  'provider.get_schema': ProviderSelection;
  'provider.connect': Connect;
  'provider.probe': AccountMutation;
  'provider.disconnect': AccountMutation;
  'provider.permissions': AccountQuery;
  'account.list': WorkspaceQuery;
  'account.get': AccountQuery;
  'account.refresh': AccountMutation;
  'workspace.open': OpenWorkspace;
  'runtime.status': EmptyPayload;
  'time.status': WorkspaceQuery;
  'time.revalidate': WorkspaceQuery;
  'domain.snapshot': Aggregate;
  'domain.subscribe': Subscribe;
  'thread.list': WorkspaceQuery;
  'thread.get': ThreadQuery;
  'thread.create': ThreadCreate;
  'turn.start': TurnStart;
  'turn.cancel': TurnCancel;
  'turn.retry': TurnRetry;
  'agent.capabilities': CapabilityQuery;
  'context.catalog': WorkspaceQuery;
  'research.run': ResearchToolRequest;
  'data.source.catalog': WorkspaceQuery;
  'data.source.probe': DataSourceProbe;
  'market.catalog': MarketCatalogQuery;
  'market.get': MarketGetQuery;
  'market.screen': ScreenerRequest;
  'screener.list': WorkspaceQuery;
  'screener.save': ScreenerSave;
  'screener.update': ScreenerUpdate;
  'screener.attach': ScreenerAttach;
  'trade.draft.list': WorkspaceQuery;
  'trade.draft.get': OrderDraftQuery;
  'trade.save_draft': OrderDraftSave;
  'trade.generate_proposal': OrderProposalGenerate;
  'trade.refresh_proposal': OrderProposalRefresh;
  'trade.proposal.list': WorkspaceQuery;
  'trade.proposal.get': OrderProposalQuery;
  'artifact.save': ArtifactSave;
  'artifact.list': WorkspaceQuery;
  'artifact.get': ArtifactQuery;
  'artifact.export': ArtifactExport;
  'portfolio.get': PortfolioQuery;
  'watchlist.list': WorkspaceQuery;
  'watchlist.create': WatchlistCreate;
  'watchlist.rename': WatchlistRename;
  'watchlist.delete': WatchlistDelete;
  'watchlist.add': WatchlistInstrumentMutation;
  'watchlist.remove': WatchlistInstrumentMutation;
  'strategy.list': WorkspaceQuery;
  'strategy.get': StrategyQuery;
  'strategy.get_run': StrategyRunQuery;
  'strategy.save_version': StrategySave;
  'strategy.run': StrategyRunRequest;
  'strategy.cancel': StrategyCancel;
  'backtest.get': BacktestRunQuery;
  'backtest.list': WorkspaceQuery;
  'backtest.compare': BacktestCompareRequest;
  'backtest.run': BacktestRunRequest;
  'backtest.cancel': BacktestCancel;
}
interface Outputs {
  'model.get_gateway': GatewayState;
  'model.gateway': GatewayState;
  'model.get': ModelState;
  'model.login_chatgpt': ModelState;
  'model.configure_deepseek': ModelState;
  'model.verify_route': ModelState;
  'model.set_default': ModelState;
  'model.set_fallback_policy': ModelState;
  'risk.get_policy': RiskPolicyState;
  'risk.save_policy': RiskPolicyState;
  'onboarding.set_step': RiskPolicyState;
  'onboarding.complete': RiskPolicyState;
  'provider.list_definitions': ProviderCatalog;
  'provider.get_schema': ProviderDefinition;
  'provider.connect': AccountConnection;
  'provider.probe': AccountConnection;
  'provider.disconnect': AccountConnection;
  'provider.permissions': PermissionReview;
  'account.list': Accounts;
  'account.get': AccountConnection;
  'account.refresh': AccountConnection;
  'workspace.open': Workspace;
  'runtime.status': RuntimeStatus;
  'time.status': TimeStatus;
  'time.revalidate': TimeStatus;
  'domain.snapshot': Snapshot;
  'domain.subscribe': SubscriptionAck;
  'thread.list': ThreadList;
  'thread.get': Thread;
  'thread.create': Thread;
  'turn.start': Thread;
  'turn.cancel': Thread;
  'turn.retry': Thread;
  'agent.capabilities': CapabilityDecision;
  'context.catalog': ContextCatalog;
  'research.run': ResearchToolResult;
  'data.source.catalog': DataSourceCatalog;
  'data.source.probe': DataSourceCatalog;
  'market.catalog': MarketCatalog;
  'market.get': MarketDetail;
  'market.screen': ScreenerResult;
  'screener.list': ScreenerLibrary;
  'screener.save': ScreenerLibrary;
  'screener.update': ScreenerLibrary;
  'screener.attach': ScreenerAttachment;
  'trade.draft.list': OrderDraftLibrary;
  'trade.draft.get': OrderDraft;
  'trade.save_draft': OrderDraft;
  'trade.generate_proposal': OrderProposal;
  'trade.refresh_proposal': OrderProposalRefreshResult;
  'trade.proposal.list': OrderProposalLibrary;
  'trade.proposal.get': OrderProposal;
  'artifact.save': Artifact;
  'artifact.list': ArtifactLibrary;
  'artifact.get': Artifact;
  'artifact.export': ArtifactExportResult;
  'portfolio.get': PortfolioSnapshot;
  'watchlist.list': Watchlists;
  'watchlist.create': Watchlist;
  'watchlist.rename': Watchlist;
  'watchlist.delete': Watchlists;
  'watchlist.add': Watchlist;
  'watchlist.remove': Watchlist;
  'strategy.list': StrategyLibrary;
  'strategy.get': StrategyVersion;
  'strategy.get_run': StrategyRun;
  'strategy.save_version': StrategyVersion;
  'strategy.run': StrategyRun;
  'strategy.cancel': StrategyRun;
  'backtest.get': BacktestRun;
  'backtest.list': BacktestLibrary;
  'backtest.compare': BacktestComparison;
  'backtest.run': BacktestRun;
  'backtest.cancel': BacktestRun;
}
const definitions = {
  'model.get_gateway': ['WorkspaceQuery', 'GatewayState'],
  'model.gateway': ['GatewayMutation', 'GatewayState'],
  'model.get': ['ModelQuery', 'ModelState'],
  'model.login_chatgpt': ['ChatgptLogin', 'ModelState'],
  'model.configure_deepseek': ['ConfigureDeepseek', 'ModelState'],
  'model.verify_route': ['VerifyRoute', 'ModelState'],
  'model.set_default': ['SetDefaultModel', 'ModelState'],
  'model.set_fallback_policy': ['SetFallbackPolicy', 'ModelState'],
  'risk.get_policy': ['RiskQuery', 'RiskPolicyState'],
  'risk.save_policy': ['SaveRiskPolicy', 'RiskPolicyState'],
  'onboarding.set_step': ['SetOnboardingStep', 'RiskPolicyState'],
  'onboarding.complete': ['CompleteOnboarding', 'RiskPolicyState'],
  'provider.list_definitions': ['EmptyPayload', 'ProviderCatalog'],
  'provider.get_schema': ['ProviderSelection', 'ProviderDefinition'],
  'provider.connect': ['Connect', 'AccountConnection'],
  'provider.probe': ['AccountMutation', 'AccountConnection'],
  'provider.disconnect': ['AccountMutation', 'AccountConnection'],
  'provider.permissions': ['AccountQuery', 'PermissionReview'],
  'account.list': ['WorkspaceQuery', 'Accounts'],
  'account.get': ['AccountQuery', 'AccountConnection'],
  'account.refresh': ['AccountMutation', 'AccountConnection'],
  'workspace.open': ['OpenWorkspace', 'Workspace'],
  'runtime.status': ['EmptyPayload', 'RuntimeStatus'],
  'time.status': ['WorkspaceQuery', 'TimeStatus'],
  'time.revalidate': ['WorkspaceQuery', 'TimeStatus'],
  'domain.snapshot': ['Aggregate', 'Snapshot'],
  'domain.subscribe': ['Subscribe', 'SubscriptionAck'],
  'thread.list': ['WorkspaceQuery', 'ThreadList'],
  'thread.get': ['ThreadQuery', 'Thread'],
  'thread.create': ['ThreadCreate', 'Thread'],
  'turn.start': ['TurnStart', 'Thread'],
  'turn.cancel': ['TurnCancel', 'Thread'],
  'turn.retry': ['TurnRetry', 'Thread'],
  'agent.capabilities': ['CapabilityQuery', 'CapabilityDecision'],
  'context.catalog': ['WorkspaceQuery', 'ContextCatalog'],
  'research.run': ['ResearchToolRequest', 'ResearchToolResult'],
  'data.source.catalog': ['WorkspaceQuery', 'DataSourceCatalog'],
  'data.source.probe': ['DataSourceProbe', 'DataSourceCatalog'],
  'market.catalog': ['MarketCatalogQuery', 'MarketCatalog'],
  'market.get': ['MarketGetQuery', 'MarketDetail'],
  'market.screen': ['ScreenerRequest', 'ScreenerResult'],
  'screener.list': ['WorkspaceQuery', 'ScreenerLibrary'],
  'screener.save': ['ScreenerSave', 'ScreenerLibrary'],
  'screener.update': ['ScreenerUpdate', 'ScreenerLibrary'],
  'screener.attach': ['ScreenerAttach', 'ScreenerAttachment'],
  'trade.draft.list': ['WorkspaceQuery', 'OrderDraftLibrary'],
  'trade.draft.get': ['OrderDraftQuery', 'OrderDraft'],
  'trade.save_draft': ['OrderDraftSave', 'OrderDraft'],
  'trade.generate_proposal': ['OrderProposalGenerate', 'OrderProposal'],
  'trade.refresh_proposal': ['OrderProposalRefresh', 'OrderProposalRefreshResult'],
  'trade.proposal.list': ['WorkspaceQuery', 'OrderProposalLibrary'],
  'trade.proposal.get': ['OrderProposalQuery', 'OrderProposal'],
  'artifact.save': ['ArtifactSave', 'Artifact'],
  'artifact.list': ['WorkspaceQuery', 'ArtifactLibrary'],
  'artifact.get': ['ArtifactQuery', 'Artifact'],
  'artifact.export': ['ArtifactExport', 'ArtifactExportResult'],
  'portfolio.get': ['PortfolioQuery', 'PortfolioSnapshot'],
  'watchlist.list': ['WorkspaceQuery', 'Watchlists'],
  'watchlist.create': ['WatchlistCreate', 'Watchlist'],
  'watchlist.rename': ['WatchlistRename', 'Watchlist'],
  'watchlist.delete': ['WatchlistDelete', 'Watchlists'],
  'watchlist.add': ['WatchlistInstrumentMutation', 'Watchlist'],
  'watchlist.remove': ['WatchlistInstrumentMutation', 'Watchlist'],
  'strategy.list': ['WorkspaceQuery', 'StrategyLibrary'],
  'strategy.get': ['StrategyQuery', 'StrategyVersion'],
  'strategy.get_run': ['StrategyRunQuery', 'StrategyRun'],
  'strategy.save_version': ['StrategySave', 'StrategyVersion'],
  'strategy.run': ['StrategyRunRequest', 'StrategyRun'],
  'strategy.cancel': ['StrategyCancel', 'StrategyRun'],
  'backtest.get': ['BacktestRunQuery', 'BacktestRun'],
  'backtest.list': ['WorkspaceQuery', 'BacktestLibrary'],
  'backtest.compare': ['BacktestCompareRequest', 'BacktestComparison'],
  'backtest.run': ['BacktestRunRequest', 'BacktestRun'],
  'backtest.cancel': ['BacktestCancel', 'BacktestRun'],
} as const;

export const browserIntegration = import.meta.env.MODE === 'integration';
export const desktop = isTauri();
export const transportAvailable = desktop || browserIntegration;

export class CommandError extends Error {
  detail: TradeXError;
  constructor(detail: TradeXError) { super(detail.message); this.name = 'CommandError'; this.detail = detail; }
}

export async function request<C extends keyof Inputs>(command: C, payload: Inputs[C], onEvent?: (event: unknown) => void): Promise<Outputs[C]> {
  decode(definitions[command][0], payload);
  const envelope = { requestId: crypto.randomUUID(), schemaVersion: 1 as const, command, payload };
  let raw: unknown;
  if (desktop) {
    const events = new Channel<unknown>();
    events.onmessage = onEvent ?? (() => {});
    raw = await invoke('control', { request: envelope, events });
  } else if (browserIntegration) {
    const response = await fetch('/__integration/command', {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
    });
    if (!response.ok) throw new Error('IPC_TRANSPORT_UNAVAILABLE');
    raw = await response.json();
  } else throw new Error('DESKTOP_REQUIRED');
  const result = decode<ResultEnvelope>('ResultEnvelope', raw);
  if (result.requestId !== envelope.requestId) throw new Error('IPC_REQUEST_MISMATCH');
  if (!result.ok) throw new CommandError(result.error);
  return decode<Outputs[C]>(definitions[command][1], result.data);
}

export async function subscribe(payload: Subscribe, onEvent: (event: unknown) => void, onError: (error: unknown) => void): Promise<() => void> {
  let active = true;
  if (desktop) {
    await request('domain.subscribe', payload, event => { if (active) onEvent(event); });
    return () => { active = false; };
  }
  if (!browserIntegration) throw new Error('DESKTOP_REQUIRED');
  const stream = new EventSource('/__integration/events');
  try {
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('IPC_TRANSPORT_UNAVAILABLE')), 5000);
      stream.onopen = () => { clearTimeout(timeout); resolve(); };
      stream.onerror = () => { clearTimeout(timeout); reject(new Error('IPC_TRANSPORT_UNAVAILABLE')); };
    });
    stream.onmessage = message => {
      if (active) {
        try { const event = decode<DomainEvent>('DomainEvent', JSON.parse(message.data)); if (event.aggregateType === payload.aggregateType && event.aggregateId === payload.aggregateId) onEvent(event); } catch (error) { onError(error); }
      }
    };
    stream.onerror = () => { if (active) { stream.close(); onError(new Error('IPC_TRANSPORT_UNAVAILABLE')); } };
    await request('domain.subscribe', payload);
    return () => { active = false; stream.close(); };
  } catch (error) { stream.close(); throw error; }
}

export function explainError(error: unknown): string {
  if (error instanceof CommandError) return error.message;
  if (error instanceof Error && error.message === 'IPC_SCHEMA_INCOMPATIBLE') return 'The application and runtime are incompatible. Update them together before continuing.';
  if (error instanceof Error && error.message.startsWith('IPC_SEQUENCE')) return 'Updates were interrupted. Reload the workspace to recover authoritative state.';
  return 'The local control plane is unavailable. Reopen TradeX or retry when it is available.';
}
