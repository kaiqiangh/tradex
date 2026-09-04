# TradeX High-Fidelity Prototype / UI Specification

**Version:** 1.0 Final — Revision B  
**Date:** 2026-09-04  
**Source of truth:** `TradeX_PRD_v1.0_RevB.md`  
**Prototype type:** Desktop-first interactive product prototype target specification  
**Current prototype implementation:** Standalone HTML / CSS / JavaScript  
**Traceability source:** `TradeX_Prototype_Coverage_Matrix_v1.0_RevB.md`

> This specification is normative for the target UI. The Coverage Matrix records which parts are already implemented in the current standalone prototype versus partial, visual-only, missing, implementation-only, or QA-pending.

---

# 1. Prototype Objective

TradeX is represented as:

```text
Intent
→ Persistent Agent Thread
→ Evidence / Typed Tools
→ Research / Strategy
→ Order Proposal
→ Deterministic Risk
→ Account-scoped Live Arming (when needed)
→ Transaction-specific User Approval
→ Reservation / Pre-execution Revalidation
→ Privileged Execution
→ Broker / Exchange State
→ Reconciliation / Audit
```

The persistent Agent Thread is the primary workspace object. TradeX must not degrade into a conventional trading dashboard with an AI chat sidebar.

---

# 2. Design System

## 2.1 Product Character

- Desktop-first
- Clean, restrained, implementation-oriented
- Dark left navigation + light primary workspace
- Medium information density
- Trading state colors are semantic, never decorative-only
- Research/agent work remains visually dominant; market context is secondary

## 2.2 Color and Mode Semantics

| Token | Value | Usage |
|---|---|---|
| App background | `#F6F8FB` | Main workspace |
| Surface | `#FFFFFF` | Panels, tables, modals |
| Sidebar | `#111827` | Navigation |
| Sidebar active | `#1F2937` | Selected navigation |
| Primary text | `#0F172A` | Main text |
| Muted | `#64748B` | Secondary information |
| Border | `#E2E8F0` | Dividers / panels |
| Blue | `#2563EB` | Research / primary action |
| Green | `#16A34A` | Paper/Demo/Testnet / healthy / completed |
| Red | `#DC2626` | Live / destructive / rejected |
| Amber | `#D97706` | stale / warning / degraded / reconciling |

Mode semantics:

- **Ask** → neutral / gray;
- **Research** → blue;
- **Backtest** → analytical blue / neutral;
- **Paper / Demo / Testnet** → green;
- **Live** → red;
- **Warning / stale / degraded / reconciling** → amber.

Paper/Demo/Testnet/Live must always use explicit text labels in addition to color.

---

# 3. Global Product Shell and Navigation

Desktop shell:

```text
┌───────────────────────────────────────────────────────────────────────┐
│ Header: Thread / View            Mode      Account + Arm State      │
├──────────────┬──────────────────────────────────┬─────────────────────┤
│ Left Rail    │ Primary Workspace                │ Dynamic Context     │
│              │                                  │                     │
│ New Thread   │ Agent timeline / market          │ Quote / provenance  │
│ Threads      │ table / editor / backtest        │ Position            │
│ Markets      │                                  │ Open orders         │
│ Watchlists   │                                  │ Evidence            │
│ Accounts     │                                  │ Risk / market state │
│ Strategies   │                                  │                     │
│ Artifacts    │                                  │                     │
│ Settings     │                                  │                     │
├──────────────┴──────────────────────────────────┴─────────────────────┤
│ Composer: @Context | Mode | Account | Model | Prompt | Send          │
└───────────────────────────────────────────────────────────────────────┘
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

Settings navigation:

- Providers & Models
- Risk & Limits
- Data & Storage
- Account Health
- Appearance
- About

Portfolio, Orders, Backtests, Account Detail, Instrument Detail, provenance, and recovery remain context-driven surfaces.

Thread history is grouped by recency and restores context but never restores a consumed/expired approval.

---

# 4. Modes, Composer, and Context

## 4.1 Modes

| Mode | UI intent | Execution capability |
|---|---|---|
| Ask | lightweight question / quick answer | none |
| Research | deeper market/portfolio investigation | none |
| Backtest | strategy research | historical simulation only |
| Paper | local/broker-hosted simulation | paper/demo/testnet only |
| Live | real account context | proposal + account arm + approval |

Ask mode must not surface paper/live execution actions.

## 4.2 Context Picker

`@ Context` can attach:

- instruments;
- accounts;
- strategies;
- backtest runs;
- artifacts.

## 4.3 Account Picker

Every row includes explicit environment:

```text
Alpaca · PAPER
Trading 212 · DEMO
Trading 212 · LIVE · DISARMED
Binance · TESTNET
Binance · LIVE · ARMED
Bitget · DEMO
Bitget · LIVE · DISARMED
```

## 4.4 Model Picker

Models are displayed in two provider groups (OD-015 resolved, PRD §16.1):

```text
CLIProxyAPI (local gateway)
  gpt-5.6-sol
  gpt-5.6-luna

DeepSeek (official API)
  deepseek-chat
  deepseek-reasoner
```

Each row carries a provider pill (CLIProxyAPI = green/local, DeepSeek = blue/API). Prototype fixture models follow the list above; production model inventory comes from the CLIProxyAPI `/v1/models` probe via the runtime/capability layer.

Selecting a model changes the reasoning model only — it takes effect on the next turn, never mutates an in-flight turn, and never changes trading authority or permissions. Model rows surface availability (probe state) so quota/OAuth degradation is visible before sending a prompt.

---

# 5. Complete Screen / State Inventory

> ID conventions: screen/state groups `A`–`K` below are the stable identifiers referenced by the Coverage Matrix; chapter numbers §1–§13 are this specification's own structure.

## A. Onboarding

### A1. Workspace

- Workspace name
- Base currency
- Local storage path

### A2. Providers

- Alpaca Paper
- Trading 212 Demo
- Trading 212 Live
- Binance Testnet
- Binance Live
- Bitget Demo
- Bitget Live

### A3. Provider Connection / Permission Review

The connection form is **provider-schema-driven**. It may contain API key, secret, passphrase, account ID, environment, or provider-specific fields; the spec must not assume a universal `key + secret` schema.

Flow:

```text
Provider/environment
→ provider-defined credential fields
→ trusted credential boundary
→ Test Connection
→ account type/capability discovery
→ permission review
→ Connection Success
```

UI explicitly excludes withdrawal, transfer, custody, margin borrowing, and leverage-management permissions.

### A4. LLM Providers (Model)

The Model step configures the LLM sources (PRD §26.3):

- CLIProxyAPI sidecar state: Running / Stopped / Port conflict / Unauthorized — a stopped or unauthorized sidecar offers launch and browser (`--codex-login`) authorization flows;
- DeepSeek: key entry → OS keychain storage → probe + test inference;
- probe `/v1/models` and model-list discovery for each source; provider rows show health pills;
- default model selection from the discovered inventory (§4.4 grouping);
- explicit note: model/provider choice does not change trading authority;
- onboarding gate: Ready is unreachable without at least one usable LLM provider (AC-055).

### A5. Risk Defaults

Initial user-configurable policy includes:

- max order notional;
- max single-instrument exposure;
- max daily traded notional;
- max daily realized loss;
- stale quote threshold;
- market-order policy;
- live inactivity timeout.

Advanced settings can expose additional user-configurable controls from PRD §21. System-enforced hard safety rules are shown read-only and cannot be weakened.

> Authoritative source: PRD §21.1 (user-configurable controls) and §21.2 (hard safety rules). This section summarizes them for onboarding; edit the PRD first, then sync here.

### A6. Ready

- workspace summary;
- providers;
- model;
- base currency;
- all live accounts = DISARMED.

---

## B. Agent Workspace

### B1. New Thread

Suggested prompts:

- Compare NVDA, AMD and AVGO
- Review my live portfolio risk
- Backtest BTC momentum strategy
- Compare BTC liquidity on Binance and Bitget

### B2. Thread History / Resume

Groups Today / Earlier / Saved. Resume restores context, not live approvals.

### B3. Ask Mode

A lightweight response state with optional read-only market context, no plan-heavy workflow by default, no paper/live execution action.

### B4. Composer Pickers

- Context picker
- Account picker
- Mode picker
- Model picker

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
```

### B7. Research Result — Equities

- conclusion;
- key findings;
- scenario analysis;
- evidence/provenance;
- artifacts;
- paper/live proposal actions where mode permits.

### B8. Research Result — Crypto

- Binance / Bitget venue comparison;
- spread;
- top-of-book depth;
- quote age;
- selected venue;
- market-order proposal action when allowed.

---

## C. Markets and Screening

### C1. Market Explorer

- search;
- US Stocks / Crypto / Screeners;
- broad-market metrics;
- top movers;
- saved screeners.

### C2. Natural-language Screener Builder

```text
Natural-language request
→ Parsed FilterSpec / RankSpec
→ Inspect/edit structured interpretation
→ Run
→ Reduced candidate set
```

### C3. Screener Result

Candidate table replaces broad market universe and can launch agent deep research.

### C4. Equity Instrument Detail

- quote/chart;
- Overview / Financials / News / Filings / Analysis;
- source/freshness metadata;
- linked thesis;
- watchlist action.

### C5. Crypto Instrument Detail

- venue quote/chart;
- bid/ask/spread;
- volume/book depth;
- quote age;
- Binance vs Bitget comparison;
- balance/position;
- market-order action when allowed.

### C6. Market Session / Corporate Actions

Equity detail also exposes:

```text
Market status: OPEN / CLOSED / EXTENDED / HALTED
Next open/close
Upcoming corporate action
Historical adjustment state
```

`MARKET_CLOSED` and `INSTRUMENT_HALTED` use deterministic blocking surfaces when execution is not permitted.

---

## D. Watchlists

### D1. Watchlist Library
### D2. Watchlist Detail
### D3. New Watchlist
### D4. Add Instrument to Watchlist

Watchlist rows may include symbol, price, change, market cap, thesis, and alert.

---

## E. Accounts and Portfolio

### E1. Accounts Overview

All MVP environments are distinct connections.

### E2. Provider-specific Account Detail

Show where supported:

- provider/environment/account type;
- health;
- balances/equity;
- positions;
- open orders;
- last sync;
- last reconciliation;
- permissions/capabilities;
- credential health;
- risk policy summary;
- optional IP allow-list state;
- per-account live arming state.

### E3. Portfolio

Shows:

- total value;
- daily P&L;
- cash;
- allocation;
- concentration;
- cross-account positions;
- **Workspace Base Currency** normalized value;
- FX source/timestamp/freshness.

Fixture data may use EUR; the specification does not hard-code EUR.

### E4. Open Order

Partially filled order may enter cancellation flow.

---

## F. Live Execution and Safety

### F1. Account-scoped Live Arming

```text
Prepare Live Order for Trading 212
→ Trading 212 DISARMED
→ Arm Trading 212 Live
→ Trading 212 ARMED
→ Binance/Bitget remain DISARMED
→ order approval may be shown
```

Header/composer shows `LIVE · Trading 212 · ARMED` or equivalent.

Global `Disable All Live Execution` disarms every live account.

### F2. Limit Order Approval — Trading 212 / AAPL

Immutable transaction plus Market Snapshot panel.

Mandatory Market Snapshot fields:

- quote;
- venue;
- provider/source;
- provider timestamp;
- TradeX received timestamp;
- realtime/delayed entitlement;
- quote age;
- freshness state.

Risk checks include reservation capacity.

### F3. AAPL Submission / Monitoring

```text
APPROVED
→ RESERVED
→ SUBMITTING
→ ACCEPTED
→ PARTIALLY_FILLED
→ FILLED
→ Reconciled
```

The exact account/instrument/side/quantity/order type remains identical to the approved proposal.

### F4. Market Order Approval — Binance / BTC

- expected spend;
- maximum authorized spend;
- bid/ask/spread;
- provider/source/timestamps;
- quote age;
- fee/slippage estimate;
- deterministic risk checks.

### F5. Approval Invalidated — Stale/Changed Market
### F6. Approval Expired
### F7. Risk Rejected
### F8. Broker Rejected

Machine mapping:

```text
Order state: REJECTED
Error category: SUBMISSION_REJECTED
UI label: Broker rejected order
```

### F9. Ambiguous Submission

`UNKNOWN_RECONCILING`, automatic retry blocked.

### F10. Live Cancellation

```text
PARTIALLY_FILLED
→ Cancel Remaining
→ approval
→ CANCEL_PENDING
→ CANCELLED
```

### F11. Reservation Conflict

A second concurrent proposal shows:

```text
Account cash            €10,000
Reserved by Thread A     €4,000
Risk-available           €6,000
Thread B requested       €7,000

RISK_REJECTED
Reason: insufficient unreserved account capacity
```

### F12. Risk-policy Change Invalidation

```text
Pending live approval exists
→ user saves relevant risk policy change
→ pending approval becomes INVALID
→ reason displayed
→ if policy weakened: affected live account DISARMED
```

---

## G. Paper / Demo / Testnet

### G1. Alpaca Paper Order
### G2. Alpaca Paper Fill

Paper warning explains simulation limitations.

### G3. Generic Broker-hosted Demo/Testnet Order Flow

Reusable normalized surface with explicit variants:

- Trading 212 Demo;
- Binance Testnet;
- Bitget Demo.

```text
Select environment
→ order proposal
→ simulated/provider acknowledgement
→ order update/fill
→ account/position update
```

No Demo/Testnet state may be visually mistaken for Live.

---

## H. Strategies and Backtesting

### H1. Strategy List
### H2. Editable Strategy Sandbox

Visible blocked capabilities:

- Network;
- Keychain;
- Order Gateway.

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

Provenance includes Thread, Turn, model, tools, sources, snapshot/dataset hashes, and related order where applicable.

---

## J. Settings

### J1. Providers & Models

Reuse provider-schema-driven connection/configuration flow.

RevB addition — LLM providers section: shows `CLIProxyAPI · local sidecar` (state: Running / Stopped / Port conflict / Unauthorized; pinned version; Open Auth / Re-login action) and `DeepSeek · official API` (Connected / Key missing; key managed in OS keychain). Each row exposes probe result and model availability; LLM providers are visually separated from broker/data providers and never grant trading capabilities (PRD §26.3).

### J2. Risk & Limits

Saving policy:

```text
persist policy
→ invalidate affected pending approvals
→ audit change
→ weakening policy disarms affected live account
→ show confirmation
```

Hard safety rules appear read-only.

### J3. Data & Storage

- SQLite / DuckDB / Parquet;
- artifacts/cache;
- retention;
- privacy boundaries;
- Export Workspace;
- Backup Now.

### J4. Import / Restore Workspace

Flow:

```text
Choose archive
→ validate manifest/schema
→ show import summary
→ confirm
→ restore non-secret workspace state
→ verify credential references
→ reconcile accounts
→ keep live execution disabled until healthy
```

Broker secrets are never imported from workspace archives.

---

## K. Recovery and Error States

### K1. Resume after Sleep
### K2. Startup Reconciliation
### K3. Authentication Failure
### K4. Private Stream Disconnect
### K5. Rate Limiting

### K6. Canonical Error Surface Variants

> Authoritative source: PRD §51 error taxonomy. The list below mirrors it for UI mapping; edit the PRD first, then sync here.

One reusable Error/Recovery component maps:

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

> Authoritative source: PRD §45 (Order State Model and UI Mapping). This table mirrors it for UI convenience; edit the PRD first, then sync here.

| Domain State | UI Surface |
|---|---|
| DRAFT | editable draft/composer |
| PROPOSED | OrderProposalCard |
| RISK_REJECTED | RiskRejectedPanel |
| NEEDS_APPROVAL | LiveApprovalModal |
| APPROVED | timeline/audit event |
| RESERVED | ReservationEvent / account-capacity detail |
| SUBMITTING | OrderTimeline pending state |
| ACCEPTED | acknowledged-not-filled state |
| PARTIALLY_FILLED | fill + remaining quantity |
| FILLED | authoritative fill summary |
| CANCEL_PENDING | cancellation pending + race warning |
| CANCELLED | authoritative cancellation |
| REJECTED | broker rejected UI; error category `SUBMISSION_REJECTED` |
| EXPIRED | expired approval/order surface |
| UNKNOWN_RECONCILING | ambiguous state / retry blocked |

---

# 7. Critical UX Invariants

> Authoritative source: PRD §62.5 (UX-001–007 UX Safety Requirements). The list below elaborates them for UI implementation; edit the PRD first, then sync here.

1. Live mode selection does not arm execution.
2. Live arming is explicit and account-scoped.
3. Arming one live account does not arm another.
4. Explicit live arming does not approve an order.
5. Every live order requires one-time transaction-specific approval.
6. Every live cancellation requires transaction-specific approval.
7. Relevant risk-policy changes invalidate affected pending approvals.
8. Weakening policy disarms the affected account.
9. Material proposal change invalidates approval.
10. Expired approval cannot be reused.
11. Stale market data blocks live execution.
12. Approval includes market-data provenance, not only quote age.
13. Approved order identity remains consistent through monitoring.
14. Broker acknowledgement is not a fill.
15. Ambiguous submission is not blindly retried.
16. Account reservations prevent double-spending across threads.
17. Risk policy cannot be modified by the agent.
18. Strategy code cannot access broker credentials or privileged Order Gateway.
19. Broker/exchange state is authoritative.
20. Paper / Demo / Testnet / Live are textually and structurally distinct.
21. Enter-key submission cannot approve a live transaction.
22. Model fallback or switching never changes capability levels, risk limits, or approval requirements; switches are written to the audit trail (PRD §16.3).
23. LLM unavailability (sidecar down / quota / OAuth) pauses agent turns only — approvals, execution, and reconciliation are never gated on model availability.

---

# 8. Component Inventory

- AppShell
- Sidebar
- SettingsSubnav
- ThreadHistory
- TopBar
- AccountLiveStateBadge
- DisableAllLiveControl
- LLMGatewayStatusBadge (sidecar state + provider availability)
- ModelProviderPill (CLIProxyAPI / DeepSeek origin on model rows)
- Composer
- ContextPicker
- AccountPicker
- ModePicker
- ModelPicker
- ContextPanel
- PlanCard
- ToolCard
- ToolErrorCard
- MarketMetric
- MarketSnapshotProvenance
- MarketStatusBanner
- CorporateActionPanel
- DataTable
- InstrumentHeader
- InstrumentTabs
- ScreenerBuilder
- WatchlistMenu
- ProviderConnectionModal
- ProviderCredentialSchemaForm
- AccountHealthPanel
- FXProvenancePanel
- OrderProposalCard
- RiskCheckPanel
- ReservationPanel
- LiveArmModal
- LiveApprovalModal
- MarketOrderApprovalModal
- ApprovalInvalidatedModal
- RiskRejectedModal
- OrderTimeline
- CancellationApprovalModal
- ErrorRecoveryPanel
- StrategyEditor
- StrategyInspector
- BacktestProgress
- BacktestMetrics
- ArtifactDetail
- ProvenanceModal
- WorkspaceImportModal
- RecoveryPanel
- SuccessToast / SuccessModal

---

# 9. Accessibility and Keyboard Interaction

Minimum rules:

- visible focus indicator on interactive controls;
- logical tab order;
- controls have accessible names;
- status updates are announced where appropriate;
- `Enter` never implicitly approves live order/cancel;
- `Escape` may dismiss/reject a dismissible approval but never approve it;
- touch/click targets remain usable in narrow layouts;
- state is never communicated by color alone;
- reduced-motion preference is respected.

---

# 10. Responsive Behavior

At widths below 900 px:

- desktop sidebar collapses;
- compact/bottom navigation may appear;
- multi-column layouts stack;
- context panel moves below primary content;
- strategy side panels may collapse;
- composer chips may scroll horizontally;
- live order identity, market snapshot provenance, risk checks, reject/approve, and disable-live actions remain reachable.

This is responsive desktop/tablet-window behavior. TradeX v1.0 does not define a native mobile live-execution product.

---

# 11. Prototype Fixture Data

Illustrative values such as AAPL `$221.42`, BTC/USDT `62,418.20`, and EUR portfolio values are fixture data only.

Fixture currency does not redefine the product requirement: portfolio aggregation uses the configured Workspace Base Currency.

---

# 12. Development Handoff Guidance

Production frontend replaces fixture state with:

- Codex App Server Thread / Turn / Item events;
- normalized market-data and snapshot-provenance models;
- account/broker adapter reads;
- trusted TradeX control-plane events;
- deterministic risk and reservation decisions;
- privileged Order Gateway events;
- local SQLite / DuckDB / Parquet persistence.

The frontend renders authority decisions but does not own financial authority.

---

# 13. Prototype Completion Standard

A product-review-complete prototype should let reviewers navigate:

```text
Onboard
→ schema-driven provider connection
→ choose model/risk defaults
→ Ask / Research / Backtest / Paper / Live modes
→ create/resume thread
→ attach context
→ screen/research markets
→ inspect equity/crypto market state
→ inspect account/portfolio + FX provenance
→ paper trade
→ Demo/Testnet trade
→ explicitly arm one live account
→ approve live limit/market order with market snapshot provenance
→ see RESERVED / SUBMITTING / ACCEPTED / partial/full fill
→ see reservation conflict
→ see risk-policy invalidation
→ inspect stale/expired/risk-rejected/rejected/ambiguous states
→ cancel open live order
→ develop/backtest/compare strategy with full metric set
→ inspect/export artifact provenance
→ export/import workspace
→ inspect startup/sleep/auth/stream/rate-limit/error recovery
→ verify narrow-window and keyboard safety
```

The Coverage Matrix is the authoritative record of which of these target states are already present in the current standalone prototype.
