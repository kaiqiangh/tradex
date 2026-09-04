# TradeX High-Fidelity Prototype / UI Specification

**Version:** 1.0 Final — Revision C  
**Date:** 2026-09-04  
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
- `More` exposes Watchlists / Artifacts / Settings / Account Health;
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

The Coverage Matrix is authoritative for prototype evidence versus implementation-only requirements.
