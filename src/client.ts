import type { FinancialSourceConnection, FinancialSourceConfigure } from '../shared/ipc-types.ts';
import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import { subscribeBrowserEvents } from './browserEvents.ts';
import type { Artifact, ArtifactExport, ArtifactExportResult, ArtifactLibrary, ArtifactQuery, ArtifactSave, ChatgptLogin, ConfigureDeepseek, GatewayMutation, GatewayState, DomainEvent, AccountConnection, AccountDeletionReceipt, AccountMutation, AccountArmingMutation, AccountQuery, Accounts, Connect, ModelState, ModelQuery, PermissionReview, ProviderCatalog, ProviderDefinition, ProviderSelection, SetDefaultModel, SetFallbackPolicy, CompleteOnboarding, RiskDecision, RiskDecisionEvaluate, RiskDecisionHistory, RiskDecisionQuery, RiskPolicyState, RiskQuery, SaveRiskPolicy, SetOnboardingStep, VerifyRoute, WorkspaceQuery, Aggregate, EmptyPayload, OpenWorkspace, ResultEnvelope, RuntimeStatus, Snapshot, Subscribe, SubscriptionAck, TradeXError, Workspace, Thread, ThreadCreate, ThreadList, ThreadQuery, TurnCancel, TurnRetry, TurnStart, CapabilityDecision, CapabilityQuery, ContextCatalog, ResearchToolRequest, ResearchToolResult, DataSourceCatalog, DataSourceProbe, DataSourceConnection, CalendarConnection, CalendarConfigure, DataSourceConfigure, DataSourceMutation, MarketCatalogQuery, MarketCatalog, MarketDetail, MarketGetQuery, HotQuoteAcquire, HotQuoteQuery, HotQuoteProjection, HotQuoteRelease, PortfolioQuery, PortfolioSnapshot, LocalPaperState, PaperOrderResult, PaperOrderSubmit, PaperOrderCancel, PaperQuoteRefresh, PaperScenarioSet, Trading212DemoOrderAttempt, Trading212DemoOrderAttemptQuery, Trading212DemoOrderAttemptQueryResult, Trading212DemoOrderSubmit, Trading212DemoOrderCancel, AlpacaPaperOrderAttempt, AlpacaPaperOrderAttemptQuery, AlpacaPaperOrderAttemptQueryResult, AlpacaPaperOrderReconcile, AlpacaPaperOrderSubmit, AlpacaPaperOrderBook, AlpacaPaperOrderBookQuery, AlpacaPaperOrderBookQueryResult, AlpacaPaperOrderBookRefresh, AlpacaPaperOrderReview, AlpacaPaperOrderCancel, BinanceTestnetOrderAttempt, BinanceTestnetOrderAttemptQuery, BinanceTestnetOrderAttemptQueryResult, BinanceTestnetOrderReconcile, BinanceTestnetOrderSubmit, BinanceTestnetOrderBook, BinanceTestnetOrderBookQuery, BinanceTestnetOrderBookQueryResult, BinanceTestnetOrderBookRefresh, BinanceTestnetOrderCancel, BitgetDemoOrderAttempt, BitgetDemoOrderAttemptQuery, BitgetDemoOrderAttemptQueryResult, BitgetDemoOrderReconcile, BitgetDemoOrderSubmit, Watchlist, Watchlists, WatchlistCreate, WatchlistRename, WatchlistDelete, WatchlistInstrumentMutation, TimeStatus, ScreenerRequest, ScreenerResult, ScreenerAttach, ScreenerAttachment, ScreenerLibrary, ScreenerSave, ScreenerUpdate, OrderDraft, OrderDraftLibrary, OrderDraftQuery, OrderDraftSave, OrderProposal, OrderProposalGenerate, OrderProposalQuery, OrderProposalLibrary, OrderProposalRefresh, OrderProposalRefreshResult, ApprovalAction, ApprovalReview, ApprovalReviewRequest, CancellationIntentRequest, CancellationReview, CancellationApprovalAction, CancellationApprovalHistoryQuery, CancellationApprovalHistory, LiveOrderRefreshRequest, FinancialApproval, FinancialApprovalHistory, FinancialApprovalHistoryQuery, ApprovalRejection, CancellationApprovalRejection, StrategyLibrary, StrategyQuery, StrategyRun, StrategyRunQuery, StrategyRunRequest, StrategySave, StrategyVersion, StrategyCancel, BacktestComparison, BacktestLibrary, BacktestRun, BacktestRunQuery, BacktestRunRequest, BacktestCompareRequest, BacktestCancel } from '../shared/ipc-types.ts';
import { decode } from './projection.ts';
import type { ExecutionPreparation, ExecutionPreparationQuery, ExecutionPreparationQueryResult, ExecutionPrepareRequest, ManualResolutionRequest, ResolutionEvidenceQuery, ResolutionEvidenceQueryResult, ResolutionEvidenceRefresh } from '../shared/ipc-types.ts';
import type { Trading212DemoOrderBook, Trading212DemoOrderBookQuery, Trading212DemoOrderBookQueryResult, Trading212DemoOrderBookRefresh } from '../shared/ipc-types.ts';

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
  'risk.evaluate_proposal': RiskDecisionEvaluate;
  'risk.decision.list': RiskDecisionQuery;
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
  'account.activity': WorkspaceQuery;
  'account.arm': AccountArmingMutation;
  'account.disarm': AccountMutation;
  'account.disable_all_live': WorkspaceQuery;
  'account.get': AccountQuery;
  'account.delete': AccountMutation;
  'account.refresh': AccountMutation;
  'paper.account.ensure': WorkspaceQuery;
  'paper.get': WorkspaceQuery;
  'paper.order.submit': PaperOrderSubmit;
  'paper.order.cancel': PaperOrderCancel;
  'paper.quote.refresh': PaperQuoteRefresh;
  'paper.scenario.set': PaperScenarioSet;
  'trading212.demo.order.submit': Trading212DemoOrderSubmit;
  'trading212.demo.order.attempt.get': Trading212DemoOrderAttemptQuery;
  'trading212.demo.orders.get': Trading212DemoOrderBookQuery;
  'trading212.demo.orders.refresh': Trading212DemoOrderBookRefresh;
  'trading212.demo.orders.cancel': Trading212DemoOrderCancel;
  'alpaca.paper.order.submit': AlpacaPaperOrderSubmit;
  'alpaca.paper.order.attempt.get': AlpacaPaperOrderAttemptQuery;
  'alpaca.paper.order.reconcile': AlpacaPaperOrderReconcile;
  'alpaca.paper.orders.get': AlpacaPaperOrderBookQuery;
  'alpaca.paper.orders.refresh': AlpacaPaperOrderBookRefresh;
  'alpaca.paper.order.review': AlpacaPaperOrderReview;
  'alpaca.paper.order.cancel': AlpacaPaperOrderCancel;
  'binance.testnet.order.submit': BinanceTestnetOrderSubmit;
  'binance.testnet.order.attempt.get': BinanceTestnetOrderAttemptQuery;
  'binance.testnet.order.reconcile': BinanceTestnetOrderReconcile;
  'binance.testnet.orders.get': BinanceTestnetOrderBookQuery;
  'binance.testnet.orders.refresh': BinanceTestnetOrderBookRefresh;
  'binance.testnet.orders.cancel': BinanceTestnetOrderCancel;
  'bitget.demo.order.submit': BitgetDemoOrderSubmit;
  'bitget.demo.order.attempt.get': BitgetDemoOrderAttemptQuery;
  'bitget.demo.order.reconcile': BitgetDemoOrderReconcile;
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
  'data.actions.connection': WorkspaceQuery;
  'data.actions.configure': FinancialSourceConfigure;
  'data.actions.disconnect': DataSourceMutation;
  'data.actions.refresh': DataSourceMutation;
  'data.instrument.connection': WorkspaceQuery;
  'data.instrument.configure': FinancialSourceConfigure;
  'data.instrument.disconnect': DataSourceMutation;
  'data.instrument.refresh': DataSourceMutation;
  'data.calendar.connection': WorkspaceQuery;
  'data.calendar.configure': CalendarConfigure;
  'data.calendar.disconnect': DataSourceMutation;
  'data.calendar.refresh': DataSourceMutation;
  'data.source.connection': WorkspaceQuery;
  'data.source.configure': DataSourceConfigure;
  'data.source.disconnect': DataSourceMutation;
  'data.source.cleanup': WorkspaceQuery;
  'data.source.catalog': WorkspaceQuery;
  'data.source.probe': DataSourceProbe;
  'market.catalog': MarketCatalogQuery;
  'market.get': MarketGetQuery;
  'market.hot.acquire': HotQuoteAcquire;
  'market.hot.get': HotQuoteQuery;
  'market.hot.release': HotQuoteQuery;
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
  'trade.request_approval': ApprovalReviewRequest;
  'trade.approve': ApprovalAction;
  'trade.execution.prepare': ExecutionPrepareRequest;
  'trade.execution.preparation.get': ExecutionPreparationQuery;
  'trade.resolution_evidence': ResolutionEvidenceQuery;
  'trade.resolution_evidence.refresh': ResolutionEvidenceRefresh;
  'trade.manual_resolution': ManualResolutionRequest;
  'trade.reject': ApprovalAction;
  'trade.approval.list': FinancialApprovalHistoryQuery;
  'trade.cancel_request': CancellationIntentRequest;
  'trade.cancel_approve': CancellationApprovalAction;
  'trade.cancel_reject': CancellationApprovalAction;
  'trade.cancel_approval.list': CancellationApprovalHistoryQuery;
  'trade.live_order.refresh': LiveOrderRefreshRequest;
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
  'risk.evaluate_proposal': RiskDecision;
  'risk.decision.list': RiskDecisionHistory;
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
  'account.activity': EmptyPayload;
  'account.arm': AccountConnection;
  'account.disarm': AccountConnection;
  'account.disable_all_live': Accounts;
  'account.get': AccountConnection;
  'account.delete': AccountDeletionReceipt;
  'account.refresh': AccountConnection;
  'paper.account.ensure': AccountConnection;
  'paper.get': LocalPaperState;
  'paper.order.submit': PaperOrderResult;
  'paper.order.cancel': PaperOrderResult;
  'paper.quote.refresh': LocalPaperState;
  'paper.scenario.set': LocalPaperState;
  'trading212.demo.order.submit': Trading212DemoOrderAttempt;
  'trading212.demo.order.attempt.get': Trading212DemoOrderAttemptQueryResult;
  'trading212.demo.orders.get': Trading212DemoOrderBookQueryResult;
  'trading212.demo.orders.refresh': Trading212DemoOrderBook;
  'trading212.demo.orders.cancel': Trading212DemoOrderBook;
  'alpaca.paper.order.submit': AlpacaPaperOrderAttempt;
  'alpaca.paper.order.attempt.get': AlpacaPaperOrderAttemptQueryResult;
  'alpaca.paper.order.reconcile': AlpacaPaperOrderAttempt;
  'alpaca.paper.orders.get': AlpacaPaperOrderBookQueryResult;
  'alpaca.paper.orders.refresh': AlpacaPaperOrderBook;
  'alpaca.paper.order.review': AlpacaPaperOrderBook;
  'alpaca.paper.order.cancel': AlpacaPaperOrderBook;
  'binance.testnet.order.submit': BinanceTestnetOrderAttempt;
  'binance.testnet.order.attempt.get': BinanceTestnetOrderAttemptQueryResult;
  'binance.testnet.order.reconcile': BinanceTestnetOrderAttempt;
  'binance.testnet.orders.get': BinanceTestnetOrderBookQueryResult;
  'binance.testnet.orders.refresh': BinanceTestnetOrderBook;
  'binance.testnet.orders.cancel': BinanceTestnetOrderBook;
  'bitget.demo.order.submit': BitgetDemoOrderAttempt;
  'bitget.demo.order.attempt.get': BitgetDemoOrderAttemptQueryResult;
  'bitget.demo.order.reconcile': BitgetDemoOrderAttempt;
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
  'data.actions.connection': FinancialSourceConnection;
  'data.actions.configure': FinancialSourceConnection;
  'data.actions.disconnect': FinancialSourceConnection;
  'data.actions.refresh': FinancialSourceConnection;
  'data.instrument.connection': FinancialSourceConnection;
  'data.instrument.configure': FinancialSourceConnection;
  'data.instrument.disconnect': FinancialSourceConnection;
  'data.instrument.refresh': FinancialSourceConnection;
  'data.calendar.connection': CalendarConnection;
  'data.calendar.configure': CalendarConnection;
  'data.calendar.disconnect': CalendarConnection;
  'data.calendar.refresh': CalendarConnection;
  'data.source.connection': DataSourceConnection;
  'data.source.configure': DataSourceConnection;
  'data.source.disconnect': DataSourceConnection;
  'data.source.cleanup': DataSourceConnection;
  'data.source.catalog': DataSourceCatalog;
  'data.source.probe': DataSourceCatalog;
  'market.catalog': MarketCatalog;
  'market.get': MarketDetail;
  'market.hot.acquire': HotQuoteProjection;
  'market.hot.get': HotQuoteProjection;
  'market.hot.release': HotQuoteRelease;
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
  'trade.request_approval': ApprovalReview;
  'trade.approve': FinancialApproval;
  'trade.execution.prepare': ExecutionPreparation;
  'trade.execution.preparation.get': ExecutionPreparationQueryResult;
  'trade.resolution_evidence': ResolutionEvidenceQueryResult;
  'trade.resolution_evidence.refresh': ResolutionEvidenceQueryResult;
  'trade.manual_resolution': ResolutionEvidenceQueryResult;
  'trade.reject': ApprovalRejection;
  'trade.approval.list': FinancialApprovalHistory;
  'trade.cancel_request': CancellationReview;
  'trade.cancel_approve': FinancialApproval;
  'trade.cancel_reject': CancellationApprovalRejection;
  'trade.cancel_approval.list': CancellationApprovalHistory;
  'trade.live_order.refresh': AccountConnection;
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
  'risk.evaluate_proposal': ['RiskDecisionEvaluate', 'RiskDecision'],
  'risk.decision.list': ['RiskDecisionQuery', 'RiskDecisionHistory'],
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
  'account.activity': ['WorkspaceQuery', 'EmptyPayload'],
  'account.arm': ['AccountArmingMutation', 'AccountConnection'],
  'account.disarm': ['AccountMutation', 'AccountConnection'],
  'account.disable_all_live': ['WorkspaceQuery', 'Accounts'],
  'account.get': ['AccountQuery', 'AccountConnection'],
  'account.delete': ['AccountMutation', 'AccountDeletionReceipt'],
  'account.refresh': ['AccountMutation', 'AccountConnection'],
  'paper.account.ensure': ['WorkspaceQuery', 'AccountConnection'],
  'paper.get': ['WorkspaceQuery', 'LocalPaperState'],
  'paper.order.submit': ['PaperOrderSubmit', 'PaperOrderResult'],
  'paper.order.cancel': ['PaperOrderCancel', 'PaperOrderResult'],
  'paper.quote.refresh': ['PaperQuoteRefresh', 'LocalPaperState'],
  'paper.scenario.set': ['PaperScenarioSet', 'LocalPaperState'],
  'trading212.demo.order.submit': ['Trading212DemoOrderSubmit', 'Trading212DemoOrderAttempt'],
  'trading212.demo.order.attempt.get': ['Trading212DemoOrderAttemptQuery', 'Trading212DemoOrderAttemptQueryResult'],
  'trading212.demo.orders.get': ['Trading212DemoOrderBookQuery', 'Trading212DemoOrderBookQueryResult'],
  'trading212.demo.orders.refresh': ['Trading212DemoOrderBookRefresh', 'Trading212DemoOrderBook'],
  'trading212.demo.orders.cancel': ['Trading212DemoOrderCancel', 'Trading212DemoOrderBook'],
  'alpaca.paper.order.submit': ['AlpacaPaperOrderSubmit', 'AlpacaPaperOrderAttempt'],
  'alpaca.paper.order.attempt.get': ['AlpacaPaperOrderAttemptQuery', 'AlpacaPaperOrderAttemptQueryResult'],
  'alpaca.paper.order.reconcile': ['AlpacaPaperOrderReconcile', 'AlpacaPaperOrderAttempt'],
  'alpaca.paper.orders.get': ['AlpacaPaperOrderBookQuery', 'AlpacaPaperOrderBookQueryResult'],
  'alpaca.paper.orders.refresh': ['AlpacaPaperOrderBookRefresh', 'AlpacaPaperOrderBook'],
  'alpaca.paper.order.review': ['AlpacaPaperOrderReview', 'AlpacaPaperOrderBook'],
  'alpaca.paper.order.cancel': ['AlpacaPaperOrderCancel', 'AlpacaPaperOrderBook'],
  'binance.testnet.order.submit': ['BinanceTestnetOrderSubmit', 'BinanceTestnetOrderAttempt'],
  'binance.testnet.order.attempt.get': ['BinanceTestnetOrderAttemptQuery', 'BinanceTestnetOrderAttemptQueryResult'],
  'binance.testnet.order.reconcile': ['BinanceTestnetOrderReconcile', 'BinanceTestnetOrderAttempt'],
  'binance.testnet.orders.get': ['BinanceTestnetOrderBookQuery', 'BinanceTestnetOrderBookQueryResult'],
  'binance.testnet.orders.refresh': ['BinanceTestnetOrderBookRefresh', 'BinanceTestnetOrderBook'],
  'binance.testnet.orders.cancel': ['BinanceTestnetOrderCancel', 'BinanceTestnetOrderBook'],
  'bitget.demo.order.submit': ['BitgetDemoOrderSubmit', 'BitgetDemoOrderAttempt'],
  'bitget.demo.order.attempt.get': ['BitgetDemoOrderAttemptQuery', 'BitgetDemoOrderAttemptQueryResult'],
  'bitget.demo.order.reconcile': ['BitgetDemoOrderReconcile', 'BitgetDemoOrderAttempt'],
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
  'data.actions.connection': ['WorkspaceQuery', 'FinancialSourceConnection'],
  'data.actions.configure': ['FinancialSourceConfigure', 'FinancialSourceConnection'],
  'data.actions.disconnect': ['DataSourceMutation', 'FinancialSourceConnection'],
  'data.actions.refresh': ['DataSourceMutation', 'FinancialSourceConnection'],
  'data.instrument.connection': ['WorkspaceQuery', 'FinancialSourceConnection'],
  'data.instrument.configure': ['FinancialSourceConfigure', 'FinancialSourceConnection'],
  'data.instrument.disconnect': ['DataSourceMutation', 'FinancialSourceConnection'],
  'data.instrument.refresh': ['DataSourceMutation', 'FinancialSourceConnection'],
  'data.calendar.connection': ['WorkspaceQuery', 'CalendarConnection'],
  'data.calendar.configure': ['CalendarConfigure', 'CalendarConnection'],
  'data.calendar.disconnect': ['DataSourceMutation', 'CalendarConnection'],
  'data.calendar.refresh': ['DataSourceMutation', 'CalendarConnection'],
  'data.source.connection': ['WorkspaceQuery', 'DataSourceConnection'],
  'data.source.configure': ['DataSourceConfigure', 'DataSourceConnection'],
  'data.source.disconnect': ['DataSourceMutation', 'DataSourceConnection'],
  'data.source.cleanup': ['WorkspaceQuery', 'DataSourceConnection'],
  'data.source.catalog': ['WorkspaceQuery', 'DataSourceCatalog'],
  'data.source.probe': ['DataSourceProbe', 'DataSourceCatalog'],
  'market.catalog': ['MarketCatalogQuery', 'MarketCatalog'],
  'market.get': ['MarketGetQuery', 'MarketDetail'],
  'market.hot.acquire': ['HotQuoteAcquire', 'HotQuoteProjection'],
  'market.hot.get': ['HotQuoteQuery', 'HotQuoteProjection'],
  'market.hot.release': ['HotQuoteQuery', 'HotQuoteRelease'],
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
  'trade.request_approval': ['ApprovalReviewRequest', 'ApprovalReview'],
  'trade.approve': ['ApprovalAction', 'FinancialApproval'],
  'trade.execution.prepare': ['ExecutionPrepareRequest', 'ExecutionPreparation'],
  'trade.execution.preparation.get': ['ExecutionPreparationQuery', 'ExecutionPreparationQueryResult'],
  'trade.resolution_evidence': ['ResolutionEvidenceQuery', 'ResolutionEvidenceQueryResult'],
  'trade.resolution_evidence.refresh': ['ResolutionEvidenceRefresh', 'ResolutionEvidenceQueryResult'],
  'trade.manual_resolution': ['ManualResolutionRequest', 'ResolutionEvidenceQueryResult'],
  'trade.reject': ['ApprovalAction', 'ApprovalRejection'],
  'trade.approval.list': ['FinancialApprovalHistoryQuery', 'FinancialApprovalHistory'],
  'trade.cancel_request': ['CancellationIntentRequest', 'CancellationReview'],
  'trade.cancel_approve': ['CancellationApprovalAction', 'FinancialApproval'],
  'trade.cancel_reject': ['CancellationApprovalAction', 'CancellationApprovalRejection'],
  'trade.cancel_approval.list': ['CancellationApprovalHistoryQuery', 'CancellationApprovalHistory'],
  'trade.live_order.refresh': ['LiveOrderRefreshRequest', 'AccountConnection'],
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
  let close = () => {};
  try {
    close = await subscribeBrowserEvents(data => {
      if (active) {
        try { const event = decode<DomainEvent>('DomainEvent', JSON.parse(data)); if (event.aggregateType === payload.aggregateType && event.aggregateId === payload.aggregateId) onEvent(event); } catch (error) { onError(error); }
      }
    }, error => { if (active) { active = false; onError(error); } });
    await request('domain.subscribe', payload);
    return () => { active = false; close(); };
  } catch (error) { active = false; close(); throw error; }
}

export function explainError(error: unknown): string {
  if (error instanceof CommandError) {
    const context = error.detail.capacityContext;
    if (error.detail.reason === 'RESERVED_CAPACITY' && context) {
      const source = context.source === 'BROKER_AVAILABLE'
        ? 'broker available'
        : 'workspace reserved-capital limit';
      const remediation = context.remediation === 'REDUCE_REQUEST_OR_REVIEW_WORKSPACE_LIMIT'
        ? 'Reduce the request or review the workspace reserved-capital limit.'
        : context.remediation === 'REDUCE_REQUEST_OR_WAIT_FOR_RESERVATIONS'
          ? 'Reduce the request or wait for earlier reservations to reconcile or complete.'
          : 'Refresh account evidence and review the request again.';
      return `RISK_REJECTED · RESERVED_CAPACITY — requested ${context.requestedAmount} ${context.unit}; ${source} ${context.capacityLimit} ${context.unit}; existing reservations ${context.existingReservations} ${context.unit}; effective capacity before this request ${context.effectiveAvailable} ${context.unit}. Next step: ${remediation}`;
    }
    return error.message;
  }
  if (error instanceof Error && error.message === 'IPC_SCHEMA_INCOMPATIBLE') return 'The application and runtime are incompatible. Update them together before continuing.';
  if (error instanceof Error && error.message.startsWith('IPC_SEQUENCE')) return 'Updates were interrupted. Reload the workspace to recover authoritative state.';
  return 'The local control plane is unavailable. Reopen TradeX or retry when it is available.';
}
