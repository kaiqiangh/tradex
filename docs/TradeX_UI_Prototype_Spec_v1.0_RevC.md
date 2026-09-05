# TradeX High-Fidelity Prototype / UI Specification

**Version:** 1.0 Final — Revision C  
**Date:** 2026-09-05\
**Source of truth:** `TradeX_PRD_v1.0_RevC.md`  
**Prototype type:** Desktop-first interactive product prototype target specification  
**Prototype implementation:** `prototype/index.html`, `prototype/styles.css`, `prototype/app.js`  
**Traceability source:** `TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md`

> This specification is normative for user-visible product behavior. Architecture/security properties that a standalone prototype cannot prove remain implementation requirements in the PRD and are marked accordingly in the Coverage Matrix.

---

# 1. Prototype Objective

TradeX is represented as:

```text
Intent
→ Persistent Agent Thread
→ Immutable Turn Context Snapshot
→ Evidence / Typed Tools
→ Research / Strategy
→ OrderDraft (editable)
→ OrderProposal (immutable)
→ Deterministic Risk
→ Account-scoped Live Arming (Live only)
→ Transaction-specific User Approval
→ Reservation / Pre-execution Revalidation
→ Privileged Execution
→ Broker / Exchange State
→ Reconciliation / Audit
```

The persistent Agent Thread is the primary workspace object. TradeX must not become a conventional trading dashboard with an AI chat sidebar.

The prototype must make three authority boundaries obvious:

1. **Agent reasoning** — can research, explain, propose, and use permitted tools;
2. **TradeX Control Plane** — owns risk, arming, approval, reservations, reconciliation, and execution authority;
3. **Broker/Exchange** — authoritative source for live order/fill/cancel truth.

---

# 2. Design System

## 2.1 Product Character

- Desktop-first, local-first, implementation-oriented;
- Codex-style workbench rather than finance-dashboard-first;
- dark left navigation + light primary workspace;
- medium information density;
- market context secondary to the thread;
- financial states use explicit text labels as well as color;
- destructive/live authority controls are visually separated from ordinary agent actions.

## 2.2 Color and State Semantics

| Token | Value | Usage |
|---|---|---|
| App background | `#F6F8FB` | Main workspace |
| Surface | `#FFFFFF` | Panels / tables / modals |
| Sidebar | `#111827` | Navigation |
| Primary text | `#0F172A` | Main text |
| Muted | `#64748B` | Secondary information |
| Border | `#E2E8F0` | Dividers / panels |
| Blue | `#2563EB` | Research / primary action |
| Green | `#16A34A` | Healthy / simulated / completed |
| Red | `#DC2626` | Live authority / destructive / rejected |
| Amber | `#D97706` | Stale / degraded / reconciling / warning |

Agent Mode semantics:

- **Ask** → neutral;
- **Research** → blue;
- **Backtest** → analytical blue/neutral;
- **Trade** → neutral until execution context is known.

Execution Context semantics:

- **READ_ONLY / HISTORICAL_SIMULATION** → neutral/blue;
- **LOCAL_PAPER / PAPER / DEMO / TESTNET** → green;
- **LIVE** → red;
- **stale/degraded/reconciling** → amber.

Paper/Demo/Testnet/Live must always use explicit text labels; color is never the sole cue.

---

# 3. Global Product Shell and Navigation

Desktop shell:

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ Header: Thread/View | Agent Mode | Execution Context | Account State   │
├──────────────┬──────────────────────────────────────┬───────────────────┤
│ Left Rail    │ Primary Workspace                    │ Dynamic Context   │
│              │                                      │                   │
│ + New Thread │ Agent timeline / market table        │ Quote/provenance  │
│ Threads      │ strategy editor / backtest           │ Position          │
│ Markets      │ order activity / recovery            │ Open orders       │
│ Watchlists   │                                      │ Risk/market state │
│ Accounts     │                                      │ Evidence          │
│ Strategies   │                                      │                   │
│ Artifacts    │                                      │                   │
│ Settings     │                                      │                   │
├──────────────┴──────────────────────────────────────┴───────────────────┤
│ Composer: @Context | Agent Mode | Execution | Account | Model | Send   │
└─────────────────────────────────────────────────────────────────────────┘
```

Primary navigation:

- New Thread
- Threads
- Markets
- Watchlists
- Accounts
- Strategies
- Artifacts
- Settings

Settings uses an **internal sub-navigation**, not separate top-level left-rail items:

- Providers & Models
- Risk & Limits
- Data & Storage
- Account Health
- Appearance
- About

Portfolio, Orders, Backtests, Account Detail, Instrument Detail, provenance, and recovery are context-driven surfaces.

Thread history restores context/defaults but never restores consumed/expired approvals or `ARMED` state.

Narrow layouts use a compact bottom navigation with **More** for secondary surfaces so Artifacts and Settings remain reachable.

---

# 4. Agent Mode, Execution Context, Composer, and Context

## 4.1 Agent Mode

| Agent Mode | UI intent | Current-market execution |
|---|---|---|
| Ask | lightweight question / quick answer | none |
| Research | deeper market/portfolio investigation | none |
| Backtest | strategy research | none; historical simulation only |
| Trade | prepare/inspect a current-market transaction | depends on selected execution account + safety gates |

Selecting `Trade` does not arm a live account.

## 4.2 Execution Context

Execution Context is independent of Agent Mode and derives from the selected account/environment:

```text
READ_ONLY
HISTORICAL_SIMULATION
LOCAL_PAPER
ALPACA_PAPER
TRADING212_DEMO
TRADING212_LIVE
BINANCE_TESTNET
BINANCE_LIVE
BITGET_DEMO
BITGET_LIVE
```

The UI displays the context explicitly. A Live account in Ask/Research is shown as `LIVE · READ-ONLY`, not as live execution authority.

Compatibility rules:

| Agent Mode | Paper/Demo/Testnet account | Live account |
|---|---|---|
| Ask | read-only context | read-only context |
| Research | read-only context | read-only context |
| Backtest | optional portfolio seed; no broker execution | optional portfolio seed; no broker execution |
| Trade | simulated/provider-hosted execution | proposal + arming + approval + live execution |

Illegal combinations are disabled or explained rather than silently remapped.

## 4.3 Context Picker

`@ Context` can attach:

- instruments;
- accounts;
- strategies;
- backtest runs;
- artifacts.

## 4.4 Account Picker

Rows show explicit environment and account-scoped live state:

```text
Local Paper · LOCAL_PAPER
Alpaca · PAPER
Trading 212 · DEMO
Trading 212 · LIVE · DISARMED
Binance · TESTNET
Binance · LIVE · ARMED
Bitget · DEMO
Bitget · LIVE · DISARMED
```

## 4.5 Model Picker and Provider Disclosure

Models are grouped by provider path:

```text
CLIProxyAPI → ChatGPT
  gpt-5.6-sol
  gpt-5.6-luna

CLIProxyAPI → DeepSeek official API
  deepseek-chat
  deepseek-reasoner
```

Each row shows provider health and availability. The composer also shows the provider path for the next turn.

Rules:

- model changes apply to the next turn/attempt;
- model/provider choice never changes trading authority;
- cross-provider automatic fallback is **OFF by default**;
- Settings may expose an explicit `Allow automatic fallback to DeepSeek` opt-in;
- fallback/switches are visibly disclosed and audited.

---

# 5. Complete Screen / State Inventory

> Stable screen/state IDs `A`–`K` are referenced by the Coverage Matrix.

## A. Onboarding

### A1. Workspace

- Workspace name;
- Base currency;
- Local storage path.

### A2. Broker / Data Providers

- Local Paper is built-in and requires no external credential;
- Alpaca Paper;
- Trading 212 Demo / Live;
- Binance Testnet / Live;
- Bitget Demo / Live;
- market-data provider fixture.

### A3. Provider Connection / Permission Review

The connection form is **provider-schema-driven**.

Flow:

```text
Provider/environment
→ render provider credential schema
→ trusted credential entry
→ Test Connection
→ detect account/capabilities/permission scope
→ Permission Review
→ Ready / Blocked / Unverified
```

Schema fields may include:

- API key;
- secret;
- passphrase;
- account identifier;
- environment;
- provider-specific non-secret metadata;
- IP allow-list status.

Permission states:

- required: account/position/order reads; place/cancel only where execution is supported;
- forbidden: withdrawal / transfer / custody;
- unsupported/out of scope: margin borrowing / leverage management;
- `UNVERIFIED`: provider cannot introspect scope.

Detected forbidden permissions block live readiness until removed.

### A4. LLM Providers (Model)

CLIProxyAPI surface:

- sidecar Running / Stopped / Port conflict / Unauthorized;
- pinned version;
- ChatGPT OAuth state;
- `/v1/models` probe;
- discovered GPT models;
- Launch / Re-login actions.

DeepSeek surface:

- key missing / connected;
- OS keychain storage statement;
- probe + test inference;
- discovered models.

Fallback setting is shown but defaults OFF.

Onboarding Ready is blocked if no usable LLM provider exists.

### A5. Risk Defaults

Editable user policy:

- max order notional;
- max single-instrument exposure;
- max daily traded notional;
- max daily realized loss;
- stale quote threshold;
- market-order policy;
- live inactivity timeout.

Hard safety rules are read-only.

### A6. Ready

Summary includes:

- workspace + base currency;
- broker/data providers;
- LLM provider/model route;
- fallback policy;
- all live accounts = `DISARMED`.

---

## B. Agent Workspace

### B1. New Thread

Suggested prompts:

- Compare NVDA, AMD and AVGO;
- Review my live portfolio risk;
- Backtest BTC momentum strategy;
- Compare BTC liquidity on Binance and Bitget.

### B2. Thread History / Resume

Groups Today / Earlier / Saved. Resume restores context only.

### B3. Ask Mode

- lightweight answer;
- optional read-only attached account/instrument;
- no trade preparation action;
- no heavy plan by default;
- upgrade action to Research/Backtest/Trade.

### B4. Composer Pickers

- Context;
- Agent Mode;
- Execution Context badge;
- Account;
- Model/provider.

### B5. Research Running

- user request;
- plan;
- Done / Running / Queued;
- typed tools;
- interim result.

### B6. Tool / Turn States

```text
Tool Running
Tool Completed
Tool Failed
Tool Retrying
Turn Cancelled
Turn Interrupted
MODEL_UNAVAILABLE / QUOTA_EXCEEDED / OAUTH_EXPIRED
```

### B7. Research Result — Equities

- conclusion;
- key findings;
- scenarios;
- evidence/provenance;
- artifacts;
- Trade action appears only when Agent Mode is Trade.

### B8. Research Result — Crypto

- Binance / Bitget venue comparison;
- spread/depth;
- quote age + provenance;
- selected venue;
- market proposal only in Trade mode.

### B9. Immutable Turn Provenance

Each completed turn exposes:

- `agent_mode_at_start`;
- `execution_context_at_start`;
- selected account;
- model/provider;
- provider attempts;
- attached context identifiers/hashes.

---

## C. Markets and Screening

### C1. Market Explorer

- search;
- US Stocks / Crypto / Screeners;
- market metrics;
- movers;
- saved screeners.

### C2. Natural-language Screener Builder

```text
Natural-language request
→ Parsed FilterSpec / RankSpec
→ Inspect/edit structured interpretation
→ Run
→ Candidate set
```

### C3. Screener Result

Candidate table can launch agent research.

### C4. Equity Instrument Detail

- quote/chart;
- Overview / Financials / News / Filings / Analysis;
- market-data source/freshness;
- market session state;
- corporate-action panel;
- watchlist;
- `MARKET_CLOSED` / `INSTRUMENT_HALTED` deterministic execution block examples.

### C5. Crypto Instrument Detail

- venue quote/chart;
- bid/ask/spread/depth;
- source/provider timestamp/received timestamp/entitlement/freshness;
- spot-only warning;
- Trade action only when compatible.

### C6. Market Session / Corporate Actions

Dedicated surface or embedded panel for:

- Open / Closed / Pre-market / After-hours where supported;
- halted instrument;
- known split/dividend/corporate action fixture;
- unsupported execution remediation.

---

## D. Watchlists

### D1. Watchlist Library
### D2. Watchlist Detail
### D3. New Watchlist
### D4. Add Instrument to Watchlist

Watchlists use on-demand/coarse refresh in MVP; they do not imply persistent tick-level Warm subscriptions.

---

## E. Accounts and Portfolio

### E1. Accounts Overview

Every connection shows provider, environment, health, equity/balance, live arming where relevant, and last sync.

### E2. Provider-specific Account Detail

At minimum variants for:

- Local Paper;
- Alpaca Paper;
- Trading 212 Demo / Live;
- Binance Testnet / Live;
- Bitget Demo / Live.

Detail includes:

- environment + account type;
- connection/auth/stream/reconciliation/execution-eligibility state;
- balances/equity;
- `Available / Reserved / Effective Available` where relevant;
- positions/open orders;
- permissions/capabilities;
- credential health;
- risk summary;
- IP allow-list status when available;
- account-scoped ARMED/DISARMED for Live.

### E3. Portfolio

Cross-account base-currency view includes FX/stablecoin provenance:

- source/path;
- provider timestamp;
- TradeX received timestamp;
- freshness;
- quality/depeg state.

### E4. Open Order

- broker state;
- proposal/order identity;
- fill/remaining quantity;
- reservation amount;
- cancel action if live and eligible.

---

## F. Live Execution and Safety

### F1. Account-scoped Live Arming

Arm dialog binds to exactly one account.

```text
Trading 212 Live · ARMED
Binance Live · DISARMED
Bitget Live · DISARMED
```

Required controls:

- per-account `Arm Live Trading` / `Disable Live`;
- global `Disable All Live Execution`;
- restart/sleep/auth/reconciliation/risk weakening disarms affected account(s).

### F2. Limit Order Approval — Trading 212 / AAPL

Shows immutable proposal plus:

- proposal ID/hash;
- policy version;
- account / venue / environment;
- side / quantity / order type / limit / TIF;
- estimated notional;
- `Available / Reserved / Effective Available`;
- full MarketSnapshot provenance:
  - source;
  - provider timestamp;
  - TradeX received timestamp;
  - venue;
  - entitlement;
  - quote age;
  - freshness;
- risk results;
- explicit Reject / Approve & Place.

### F3. AAPL Submission / Monitoring

Visible sequence:

```text
APPROVED
→ RESERVED
→ SUBMITTING
→ ACCEPTED (not a fill)
→ PARTIALLY_FILLED or FILLED
→ reconciliation
```

Order identity remains unchanged.

### F4. Market Order Approval — Binance / BTC

Adds:

- expected spend;
- maximum authorized spend;
- bid/ask/spread;
- quote provenance;
- estimated fee/slippage where available;
- hard rejection if required spend exceeds authorization.

### F5. Approval Invalidated — Market Changed / Stale

Old approval cannot execute; refreshed proposal requires new approval.

### F6. Approval Expired

Expired approval is not reusable; reservation is released when it had not submitted.

### F7. Risk Rejected

Deterministic `RISK_REJECTED`; agent cannot override.

### F8. Broker Rejected

Order state `REJECTED`; error category `SUBMISSION_REJECTED`.

### F9. Ambiguous Submission / Manual Resolution

```text
SUBMITTING
→ UNKNOWN_RECONCILING
→ automatic query-first reconciliation
→ timeout: reservation remains frozen, account unhealthy/DISARMED
→ Manual Resolution
```

Manual Resolution actions:

- Confirmed not submitted (evidence required);
- Confirmed submitted + broker order identity;
- Keep reconciling.

There is no generic “release reservation and continue live trading” action.

### F10. Live Cancellation

Refresh broker state → approval → `CANCEL_PENDING` → `CANCELLED` or fill race.

v1.0 order modification is cancel + new proposal; no approval-skipping amend.

### F11. Reservation Conflict

Interactive fixture:

```text
Thread A reserves €4,000
Available = €10,000
Reserved = €4,000
Effective Available = €6,000
Thread B requires €7,000
→ RISK_REJECTED · RESERVED_CAPACITY
```

### F12. Risk-policy Change Invalidation

Saving a relevant policy update:

- increments policy version;
- invalidates affected pending approval;
- shows old/new version + reason;
- weakening also disarms affected live account.

---

## G. Local Paper / Broker Paper / Demo / Testnet

### G1. Local Paper

TradeX-managed simulation; explicit `LOCAL PAPER` label; never provider truth.

### G2. Alpaca Paper

Normalized proposal → provider paper acknowledgement → fill/update → account refresh.

### G3. Trading 212 Demo
### G4. Binance Testnet
### G5. Bitget Demo

All use one normalized interactive component with provider/environment fixture data:

```text
PROPOSED
→ ACCEPTED / provider acknowledgement
→ PARTIALLY_FILLED / FILLED
→ account/position refresh
```

No non-live state may be visually confused with Live.

---

## H. Strategies and Backtesting

### H1. Strategy List
### H2. Editable Strategy Sandbox

Visible blocked capabilities:

- broker credentials;
- OS keychain;
- privileged Order Gateway;
- unrestricted network.

### H3. Backtest Running
### H4. Backtest Failed
### H5. Backtest Result

Required metrics:

- return;
- Sharpe;
- Sortino;
- max drawdown;
- win rate;
- profit factor;
- turnover;
- equity curve;
- trade list;
- reproducibility manifest.

### H6. Backtest Compare

- metrics diff;
- parameter diff;
- manifest/version links.

---

## I. Artifacts

### I1. Artifact Library
### I2. Artifact Detail
### I3. Provenance Drawer / Modal
### I4. Export

Provenance includes Thread, Turn, immutable Turn snapshot, model/provider attempts, tools, sources, market/dataset hashes, and related order where applicable.

---

## J. Settings

Settings internal subnav is always reachable.

### J1. Providers & Models

Separate groups:

- Broker / Exchange / Market Data;
- LLM Gateway & Models.

Broker/provider rows use schema-driven configure/test flow and permission review.

LLM rows show:

- CLIProxyAPI state/version/OAuth/probe/models;
- DeepSeek key/probe/models;
- selected model/provider;
- `Allow automatic fallback to DeepSeek` toggle, default OFF;
- simulated `MODEL_UNAVAILABLE`, `QUOTA_EXCEEDED`, `OAUTH_EXPIRED` remediation.

### J2. Risk & Limits

Saving policy:

```text
persist policy
→ version policy
→ invalidate affected approvals
→ audit change
→ if weakening: disarm affected live account
→ show confirmation/reason
```

Hard safety rules are read-only.

### J3. Data & Storage

MVP storage wording must match PRD:

- SQLite — transactional/domain state;
- DuckDB — persistent 1-minute+ OHLCV + analytical/backtest data;
- Filesystem — artifacts/exports/backups;
- Parquet — optional/Phase 2+ large historical datasets.

Also show retention, privacy boundaries, Export Workspace, Backup Now.

### J4. Import / Restore Workspace

```text
Choose archive
→ validate manifest/schema
→ import summary
→ confirm
→ restore non-secret state
→ verify credential references
→ reconcile accounts
→ all live accounts remain DISARMED until healthy
```

Broker secrets are never imported.

### J5. Account Health

Orthogonal state display:

- connection;
- authentication;
- private stream;
- reconciliation;
- execution eligibility;
- arming.

### J6. Appearance

Prototype-only light/system preference surface.

### J7. About

Version, local runtime status, docs baseline, and prototype disclaimer.

---

## K. Recovery and Error States

### K1. Resume after Sleep
### K2. Startup Reconciliation
### K3. Authentication Failure
### K4. Private Stream Disconnect
### K5. Rate Limiting
### K6. Clock / Freshness Failure

TimeService/clock uncertainty maps to stale/reconciliation remediation; live authority fails closed.

### K7. Canonical Error Surface Variants

One reusable ErrorRecoveryPanel maps:

- `AUTH_ERROR`;
- `PERMISSION_ERROR`;
- `RATE_LIMITED`;
- `NETWORK_ERROR`;
- `UNSUPPORTED_CAPABILITY`;
- `MARKET_CLOSED`;
- `INSTRUMENT_HALTED`;
- `INVALID_ORDER`;
- `INSUFFICIENT_FUNDS`;
- `RISK_REJECTED`;
- `SUBMISSION_REJECTED`;
- `SUBMISSION_AMBIGUOUS`;
- `STREAM_DISCONNECTED`;
- `STATE_STALE`;
- `RECONCILIATION_REQUIRED`;
- `MODEL_UNAVAILABLE`;
- `QUOTA_EXCEEDED`;
- `OAUTH_EXPIRED`;
- `INTERNAL_ERROR`.

---

# 6. Order State → UI Mapping

| Domain State | UI Surface |
|---|---|
| DRAFT | editable OrderDraft |
| PROPOSED | immutable OrderProposalCard |
| RISK_REJECTED | RiskRejectedPanel |
| NEEDS_APPROVAL | LiveApprovalModal |
| APPROVED | timeline/audit event |
| RESERVED | ReservationEvent + capacity detail |
| SUBMITTING | pending provider submission |
| ACCEPTED | broker acknowledged, explicitly not a fill |
| PARTIALLY_FILLED | fill + remaining quantity |
| FILLED | authoritative fill summary |
| CANCEL_PENDING | cancellation pending + race warning |
| CANCELLED | authoritative cancellation |
| REJECTED | broker rejected; error category `SUBMISSION_REJECTED` |
| EXPIRED | expired approval/order; not reusable |
| UNKNOWN_RECONCILING | ambiguous state + query-first reconciliation + Manual Resolution |

---

# 7. Critical UX Invariants

1. Agent Mode and Execution Context are separate dimensions.
2. Ask/Research with a live account remains read-only.
3. Selecting Trade or a Live context does not arm execution.
4. Live arming is explicit and account-scoped.
5. Arming one live account does not arm another.
6. Global Disable All disarms every live account immediately for new submissions.
7. Arming does not approve any transaction.
8. Every live order/cancellation requires transaction-specific approval.
9. OrderDraft edits generate a new immutable proposal identity.
10. Material proposal changes invalidate prior approval.
11. Relevant risk-policy changes invalidate affected approvals; weakening disarms the account.
12. Stale market data or untrustworthy time blocks live authority.
13. Approval includes full market-data provenance, not quote age alone.
14. `RESERVED` is visible before submission when capacity is held.
15. Broker acknowledgement is not a fill.
16. Ambiguous submission is never blindly retried or casually released.
17. Manual Resolution is evidence-based and audited.
18. Reservations prevent cross-thread double-spend.
19. Risk policy cannot be modified by the agent.
20. Strategy code cannot access broker credentials/Order Gateway.
21. Broker/exchange state is authoritative.
22. Local Paper / Paper / Demo / Testnet / Live are textually distinct.
23. Generic Enter/form submission never implicitly approves a live transaction.
24. Cross-provider LLM fallback is OFF by default; switches/fallbacks are disclosed/audited.
25. LLM unavailability affects agent turns only, never control-plane authority/reconciliation.
26. No cloud/share-link semantics are implied by the local-first prototype; use local export/copy actions instead.

---

# 8. Component Inventory

- AppShell
- Sidebar
- SettingsSubnav
- CompactMoreNav
- ThreadHistory
- TopBar
- AgentModeBadge / AgentModePicker
- ExecutionContextBadge
- AccountLiveStateBadge
- DisableAllLiveControl
- LLMGatewayStatusBadge
- ModelProviderPill
- Composer
- ContextPicker
- AccountPicker
- ModelPicker
- TurnProvenancePanel
- ContextPanel
- PlanCard
- ToolCard / ToolErrorCard
- MarketMetric
- MarketSnapshotProvenance
- MarketStatusBanner
- CorporateActionPanel
- FXProvenancePanel
- DataTable
- InstrumentHeader / InstrumentTabs
- ScreenerBuilder
- WatchlistMenu
- ProviderConnectionModal
- ProviderCredentialSchemaForm
- PermissionReviewPanel
- AccountHealthPanel
- OrderDraftEditor
- OrderProposalCard
- RiskCheckPanel
- ReservationPanel
- LiveArmModal
- LiveApprovalModal
- MarketOrderApprovalModal
- ApprovalInvalidatedModal
- RiskRejectedModal
- ReservationConflictModal
- ManualResolutionModal
- OrderTimeline
- CancellationApprovalModal
- SimulatedOrderFlow
- ErrorRecoveryPanel
- StrategyEditor / StrategyInspector
- BacktestProgress / BacktestMetrics
- ArtifactDetail / ProvenanceModal
- WorkspaceImportModal
- RecoveryPanel
- SuccessToast / SuccessModal

---

# 9. Accessibility and Keyboard Interaction

Minimum prototype rules:

- every interactive control is keyboard reachable;
- visible `:focus-visible` styling;
- logical tab order;
- modal uses dialog semantics and accessible name;
- first focus in a live approval goes to a non-approval/reject-safe location, never directly to Approve;
- generic `Enter` in composer/forms does not approve live transactions;
- `Escape` may dismiss/reject a dismissible approval but never approve;
- status/toast/order changes are exposed via `aria-live` where appropriate;
- state is never color-only;
- reduced-motion preference is respected;
- narrow-layout actions remain reachable.

---

# 10. Responsive Behavior

Below 900 px:

- sidebar collapses;
- compact bottom nav appears;
- a drawer or `More` exposes New Thread / Thread History / Watchlists / Artifacts / Settings / Account Health;
- multi-column layouts stack;
- context panel moves below primary content;
- strategy side panels may collapse;
- composer chips scroll horizontally;
- live identity, arming state, provenance, risk, Reject/Approve, and Disable Live remain reachable.

TradeX v1.0 defines responsive desktop/tablet-window behavior, not a native mobile live-execution product.

---

# 11. Prototype Fixture Data

AAPL, BTC/USDT, balances, timestamps, risk numbers, permission schemas, FX routes, quota states, and model/provider health are illustrative fixtures only.

Fixture labels such as `US Market Data (fixture)` or `FX Provider (fixture)` intentionally avoid implying a resolved OD where the PRD still records one.

---

# 12. Development Handoff Guidance

Production frontend replaces fixture state with:

- Codex App Server Thread/Turn/Item events;
- immutable Turn snapshots;
- normalized market-data + TimeService provenance;
- account/broker adapter reads;
- provider-schema/capability metadata;
- TradeX Control Plane authority events;
- deterministic risk/reservation decisions;
- privileged Order Gateway events;
- reconciliation/manual-resolution events;
- SQLite + DuckDB persistence; optional Parquet for large data.

The frontend renders authority decisions but does not own financial authority.

---

# 13. Prototype Completion Standard

A product-review-complete Revision C prototype lets a reviewer navigate:

```text
Onboard
→ schema-driven broker provider connection + permission review
→ configure CLIProxyAPI / DeepSeek and fallback policy
→ set risk defaults
→ Ask / Research / Backtest / Trade Agent Modes
→ see separate Execution Context
→ create/resume thread + immutable turn provenance
→ attach account/context/model
→ screen/research markets
→ inspect market session/halt/corporate-action state
→ inspect account/portfolio + reservation/FX provenance
→ Local Paper / Alpaca Paper / T212 Demo / Binance Testnet / Bitget Demo
→ select Trade + one Live account
→ arm exactly that account
→ approve live limit/market order with full snapshot provenance
→ see RESERVED → SUBMITTING → ACCEPTED → fill
→ reservation conflict
→ risk-policy invalidation
→ stale/expired/risk-rejected/rejected/ambiguous states
→ evidence-based Manual Resolution
→ cancel open live order
→ backtest with full metrics
→ artifact provenance/export
→ workspace export/import
→ account health + complete error-remediation examples
→ LLM unavailable/quota/OAuth recovery
→ narrow-window + keyboard/focus safety
```

The Coverage Matrix classifies prototype evidence separately from runtime requirements; the QA Report owns observed results. This is a completion standard, not a claim that the current prototype meets it. See §14 for detailed interaction contracts and the remaining repair cases.

---

# 14. Interaction Contracts for Implementation and Prototype Repair

These contracts refine the existing A–K screen inventory; they do not add business scope or certify the current HTML implementation. [QA Report](./TradeX_Prototype_QA_Report_v1.0_RevC.md) records observed failures and regression cases. Backend authority remains defined by the PRD and Backend ARD.

## 14.1 Thread, picker, and instrument identity (B1/B2/B4/B9, C4/C5, I3)

- New Thread creates a distinct empty thread; selecting a history item restores that item's own timeline and saved next-turn defaults. Loading, empty history, and failed resume have explicit states with retry/back navigation.
- Context Picker edits a temporary selection. Cancel discards changes; Attach shows the exact selected account/instrument/strategy/artifact chips and preserves their canonical IDs. Removing a chip affects the next Turn only.
- Mode/account/model changes affect the next Turn. Switching to Backtest preserves the selected account as optional read-only portfolio seed; it does not silently replace it with No account. An incompatible selection displays a reason and blocks Send until the user chooses a supported context.
- At Send, the backend freezes mode, execution context, account, capability, model/provider route, and context IDs/hashes. Historical cards and Artifact provenance continue to show this snapshot. Later provider attempts append provenance events; they never rewrite the initial route.
- Clicking a market row uses the selected canonical instrument/venue, not a default AAPL/BTC mapping. An unsupported fixture displays a clearly identified unavailable state with Back; it must not show another instrument's identity as the selected one.

## 14.2 Editable OrderDraft and immutable OrderProposal (F2/F4/F5/F12)

The draft editor exposes account/environment, instrument, side, quantity semantics, order type, price or maximum spend as applicable, and TIF. Invalid/missing values show field errors and disable Generate Proposal. Switching account/instrument reloads applicable capabilities and marks incompatible draft fields for correction.

Generate Proposal freezes the displayed values into a new proposal ID/hash and policy/snapshot references. The approval summary is read-only. Edit returns to a draft, invalidates the prior proposal's approval, and requires Generate Proposal plus new consent; Refresh after stale market data or policy change also creates a new identity. Keep the old proposal and invalidation reason inspectable. A fixture may use clearly labelled synthetic revision IDs/hashes, but changed revisions must differ; production uses backend canonical serialization.

## 14.3 Eligibility, arming, and Disable All (F1–F4, J2/J5, K1/K2/K6)

| Action | Required state | Visible blocked behavior |
|---|---|---|
| Arm account | Exact Live account, healthy/reconciled, acceptable permissions, trusted time, supported Live capability | Explain failed checks and provide relevant recovery; Arm disabled |
| Prepare/approve new Live order | Trade mode, compatible account/instrument, account ARMED, eligible market/session/order type, fresh quote/FX where required, current proposal/policy and sufficient capacity | Show backend reason; Approve disabled; Refresh never silently approves |
| Dispatch reserved attempt | All authority checks still valid at the trusted dispatch boundary | Stop undispatched work; show invalidation/revalidation outcome |
| Cancel existing order | Exact provider order, cancellable remaining quantity, current broker state, account authority, cancellation-specific capability | Show why cancellation is unavailable; do not apply a new-order rule blindly |
| Disable All / restart / sleep / session lock | All Live accounts in scope | Disarm all; distinguish stopped-before-dispatch from possibly submitted attempts |

Account-specific faults affect that account; shared policy changes affect all bound accounts. The selected UI account never limits system-wide recovery. Disable All blocks new transmissions and invalidates undispatched consent; it does not cancel an already submitted broker order. Retain/reconcile capacity for attempts that may have left TradeX. Recovery and model switching never auto-arm accounts.

## 14.4 Expiry and cancellation continuation (F6/F10)

Approval TTL expiry uses the guarded PRD §45 release rules. The expiry screen shows proposal/account, expiry time/reason, whether submission may have begun, and the backend-provided reservation disposition. Refresh creates a new proposal requiring approval; a submitted/unknown attempt routes to order activity/reconciliation, not a replacement order.

Cancellation flow: select an open order → refresh its broker state → preserve an immutable CANCEL intent → Arm exact account if needed → refresh/revalidate cancellation intent → cancellation approval → CANCEL_PENDING → broker-confirmed CANCELLED or a fill race. The approval displays provider order ID, account/environment, instrument, observed filled/remaining quantity and timestamp. Its action reads Approve Cancellation; it never uses Approve & Place. Reject/Back dismisses consent without a mutation. A changed/filled order invalidates the old cancellation intent and explains the new state.

## 14.5 Unknown submission and Manual Resolution (F3/F9, K7)

UNKNOWN_RECONCILING uses an amber pending/uncertain treatment with explicit text; it is never a green success fallback. The activity timeline contains only observed events. ACCEPTED requires broker acknowledgement, and FILLED requires broker fill evidence. Pending broker identity cannot coexist with invented acknowledgement/fill events.

Layout: order/account identity and unknown-state banner first; observed timeline and frozen reservation next; evidence/reconciliation panel and actions last. The panel shows last query time, provider source, coverage window, results, errors, and next reconciliation action.

Manual Resolution starts by loading backend-owned evidence and allowed decisions. Confirmed submitted requires a verified broker order ID matched to the account/instrument/action. Confirmed not submitted requires sufficient provider absence evidence; an empty query or checkbox is insufficient. Show the evidence summary and intended reservation effect before final confirmation. Missing/stale/conflicting evidence disables confirmation and explains how to refresh. A concurrent fill invalidates stale manual input. Keep reconciling or closing the dialog preserves the frozen reservation. After valid resolution, show health revalidation progress and keep DISARMED until explicitly armed again.

## 14.6 Screener review and result flow (C2/C3)

Use explicit stages: Describe → Parse → Inspect/edit FilterSpec → Run → Results. Expose editable universe, predicates, thresholds, sort/rank, and result limit supported by the schema. Editing natural language after parsing marks the interpretation stale and requires Parse again; structured edits invalidate previous results. Run uses the displayed validated FilterSpec revision.

Show parse failure beside the input, unsupported filters beside the relevant field, and separate Running, Empty, Failed, and Completed result states. Retry preserves the reviewed inputs; Back to filters preserves edits. Results show applied conditions and count. Opening a row preserves its instrument ID; attaching checked candidates adds only those candidates to the selected/new Thread. Fixture data may be bounded to documented scenarios, but controls must either affect that scenario or disclose an unsupported operation.

## 14.7 Backtest entry and lifecycle (B4, H2–H6)

Backtest Send and the strategy editor converge on one run-configuration flow: select a saved strategy/version or explicitly save the edited strategy; choose instrument/dataset, date range, bar interval, initial capital, fees/slippage, and optional read-only portfolio seed. Missing/invalid inputs block Run with field-level messages. A request to create/revise a strategy may use the Agent, but must produce a selected saved version before deterministic execution begins.

Freeze inputs at Run and show a run ID plus Running progress. Cancel stops that run; Failed retains inputs and a concrete retry action; Completed shows metrics/trades and the reproducibility manifest for the frozen run. Editing strategy/configuration creates a new run and cannot change old results. Compare selects two completed run IDs. No Backtest action enters a broker order flow; an unavailable model blocks strategy-generation turns but does not by itself stop an already configured deterministic backtest.

## 14.8 Model setup and error recovery (A4, J1, K7)

| State | Required controls and outcome |
|---|---|
| CLIProxyAPI stopped | Launch → starting/probing → ready or specific failure |
| Port conflict / unauthorized | Show endpoint/status; Retry probe after configuration repair; no false Ready |
| ChatGPT OAuth missing/expired | Re-login and explicit provider switch; verify route health before enabling Send |
| DeepSeek key missing/invalid | Masked secure key entry, save to keychain, probe + test inference, error/success; no secret in logs/history |
| QUOTA_EXCEEDED | Show known quota/cooldown; explicit Retry when eligible or Switch provider |
| MODEL_UNAVAILABLE | Preserve draft/history, block affected Send, offer probe/restart guidance and explicit alternate route |
| Fallback OFF / opt-in ON | Default OFF; automatic cross-provider retry only after explicit opt-in, with attempt disclosure/audit and unchanged capabilities |

Onboarding Ready requires at least one verified usable route. Closing an error dialog does not restore availability. In-flight attempts retain their error/provenance; a retry/switch is an explicit new attempt and does not erase the failed attempt. Pure model outages leave account/reconciliation controls usable. Prototype credential examples must be visibly fake and must not collect real secrets.

## 14.9 Keyboard, narrow layout, and visual states (all screens)

- On dialog open, store the invoker, make the background inert, and focus the title/neutral safe control. Trap Tab/Shift+Tab within the dialog. Escape rejects/dismisses only where allowed; close restores focus to the invoker or a stable logical fallback. Generic Enter never approves a financial action.
- Use semantic links/buttons for clickable rows; icon-only controls have accessible names. Focus order follows visual reading order. Status transitions are announced without repeatedly announcing streaming noise.
- At widths below 900 px, provide New Thread and Thread History in a drawer or More, alongside Watchlists, Artifacts, Settings, and Account Health. Provider Configure/Models, retry controls, and table-row actions remain reachable by wrapping/stacking rather than hiding them.
- At 768 px and a smaller 390 px stress viewport, keep content inside the window; wide data tables may scroll internally. A tall dialog scrolls its body while identity, close/reject and approval controls remain reachable; do not cover actions with the bottom navigation.
- Apply the existing §2 tokens, type scale and spacing. Green indicates a confirmed healthy/success state; uncertain/reconciling is amber; rejection/blocking uses error treatment. Every color has a text label. Empty, loading, failed, stale, permission-limited, disabled and completed states must explain the next available action.
- Long account/instrument IDs may truncate in lists only if the full identity is available by keyboard/touch; approval and evidence views expose the full inspectable identity. Reduced motion removes nonessential transitions without hiding progress.

## 14.10 Acceptance evidence

The QA Report's QA-01–QA-12 scenarios define the minimum regression set for these refinements. Passing requires actual action/state assertions, including negative and interruption paths. A screenshot proves layout only; an interface or status label proves presence only. Runtime integration, provider truth, persistence, and assistive-technology checks retain separate gates. Existing FR/AC IDs remain stable, and both language editions must record the same case status.
