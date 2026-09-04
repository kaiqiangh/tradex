# TradeX 高保真原型 / UI 规范

**版本:** 1.0 Final — Revision C(中文版)  
**日期:** 2026-09-04  
**英文权威版:** `TradeX_UI_Prototype_Spec_v1.0_RevC.md`  
**PRD:** `TradeX_PRD_v1.0_RevC_zh.md`  
**原型实现:** `prototype/index.html`、`prototype/styles.css`、`prototype/app.js`  
**追溯:** `TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md`

> 本规范定义用户可见产品行为。独立原型无法证明的架构/安全属性仍以 PRD 为实现要求,并在 Coverage Matrix 中标记为 implementation-only。

---

# 1. 原型目标

TradeX 通过以下链条表达:

```text
Intent
→ Persistent Agent Thread
→ Immutable Turn Context Snapshot
→ Evidence / Typed Tools
→ Research / Strategy
→ OrderDraft (editable)
→ OrderProposal (immutable)
→ Deterministic Risk
→ Account-scoped Live Arming (仅 Live)
→ Transaction-specific User Approval
→ Reservation / Pre-execution Revalidation
→ Privileged Execution
→ Broker / Exchange State
→ Reconciliation / Audit
```

Persistent Agent Thread 是主工作对象,不能退化成传统交易 dashboard + AI chat sidebar。

原型必须清晰表达三个 authority boundary:

1. **Agent reasoning**——研究、解释、提案与 permitted tools;
2. **TradeX Control Plane**——risk、arming、approval、reservation、reconciliation 与 execution authority;
3. **Broker/Exchange**——Live order/fill/cancel 的权威事实来源。

---

# 2. 设计系统

## 2.1 产品特征

- Desktop-first、local-first、implementation-oriented;
- Codex-style workbench,而非 finance-dashboard-first;
- 深色左导航 + 浅色主工作区;
- 中等信息密度;
- thread 优先,market context 次级;
- 金融状态同时使用文字 + 颜色;
- destructive/live authority control 与普通 agent action 清晰分离。

## 2.2 颜色与状态语义

| Token | Value | 用途 |
|---|---|---|
| App background | `#F6F8FB` | 主工作区 |
| Surface | `#FFFFFF` | Panels / tables / modals |
| Sidebar | `#111827` | 导航 |
| Primary text | `#0F172A` | 主文本 |
| Muted | `#64748B` | 次级信息 |
| Border | `#E2E8F0` | 分隔/面板 |
| Blue | `#2563EB` | Research / primary action |
| Green | `#16A34A` | Healthy / simulated / completed |
| Red | `#DC2626` | Live authority / destructive / rejected |
| Amber | `#D97706` | stale / degraded / reconciling / warning |

Agent Mode:

- **Ask** → neutral;
- **Research** → blue;
- **Backtest** → analytical blue/neutral;
- **Trade** → 在 Execution Context 确定前保持 neutral。

Execution Context:

- **READ_ONLY / HISTORICAL_SIMULATION** → neutral/blue;
- **LOCAL_PAPER / PAPER / DEMO / TESTNET** → green;
- **LIVE** → red;
- stale/degraded/reconciling → amber。

Paper/Demo/Testnet/Live 必须始终有显式文字标签,不能只靠颜色。

---

# 3. 全局产品外壳与导航

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ Header: Thread/View | Agent Mode | Execution Context | Account State   │
├──────────────┬──────────────────────────────────────┬───────────────────┤
│ Left Rail    │ Primary Workspace                    │ Dynamic Context   │
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

主导航:

- New Thread
- Threads
- Markets
- Watchlists
- Accounts
- Strategies
- Artifacts
- Settings

Settings 使用**内部 sub-navigation**,不再把子设置放成左栏一级入口:

- Providers & Models
- Risk & Limits
- Data & Storage
- Account Health
- Appearance
- About

Portfolio、Orders、Backtests、Account Detail、Instrument Detail、provenance、recovery 保持 context-driven。

恢复 thread 不恢复已消费/过期 approval 或 `ARMED`。

窄窗口使用 compact bottom nav,并通过 **More** 保证 Artifacts 与 Settings 可达。

---

# 4. Agent Mode、Execution Context、Composer 与 Context

## 4.1 Agent Mode

| Agent Mode | UI 意图 | 当前市场执行 |
|---|---|---|
| Ask | 轻量问题/快速回答 | 无 |
| Research | 深度市场/组合研究 | 无 |
| Backtest | 策略研究 | 无;仅历史模拟 |
| Trade | 准备/检查当前市场交易 | 取决于 execution account + safety gates |

选择 `Trade` 不会布防 Live account。

## 4.2 Execution Context

Execution Context 与 Agent Mode 独立,由所选账户/环境推导:

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

Live account 在 Ask/Research 中显示 `LIVE · READ-ONLY`,绝不表示 Live execution authority。

兼容规则:

| Agent Mode | Paper/Demo/Testnet account | Live account |
|---|---|---|
| Ask | 只读 context | 只读 context |
| Research | 只读 context | 只读 context |
| Backtest | 可作 portfolio seed;无 broker execution | 可作 portfolio seed;无 broker execution |
| Trade | simulated/provider-hosted execution | proposal + arming + approval + live execution |

非法组合应 disabled 或解释原因,不得静默 remap。

## 4.3 Context Picker

`@ Context` 可附加 instrument、account、strategy、backtest run、artifact。

## 4.4 Account Picker

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

## 4.5 Model Picker 与 Provider Disclosure

```text
CLIProxyAPI → ChatGPT
  gpt-5.6-sol
  gpt-5.6-luna

CLIProxyAPI → DeepSeek official API
  deepseek-chat
  deepseek-reasoner
```

每行显示 provider health/availability;composer 同时显示下一 turn 的 provider path。

规则:

- model change 作用于下一 turn/attempt;
- model/provider 不改变 trading authority;
- 跨 provider 自动 fallback **默认关闭**;
- Settings 可显式 opt-in `Allow automatic fallback to DeepSeek`;
- fallback/switch 必须披露并写入 audit。

---

# 5. 完整 Screen / State Inventory

> `A`–`K` 为 Coverage Matrix 使用的稳定 screen/state ID。

## A. Onboarding

### A1. Workspace

Workspace name、Base currency、Local storage path。

### A2. Broker / Data Providers

- Local Paper(内置,无外部 credential);
- Alpaca Paper;
- Trading 212 Demo / Live;
- Binance Testnet / Live;
- Bitget Demo / Live;
- market-data provider fixture。

### A3. Provider Connection / Permission Review

Provider connection 必须 **schema-driven**:

```text
Provider/environment
→ render provider credential schema
→ trusted credential entry
→ Test Connection
→ detect account/capabilities/permission scope
→ Permission Review
→ Ready / Blocked / Unverified
```

Schema 可包含 API key、secret、passphrase、account ID、environment、provider-specific metadata、IP allow-list status。

Permission:

- required: account/position/order read;支持 execution 时 place/cancel;
- forbidden: withdrawal / transfer / custody;
- out-of-scope: margin borrowing / leverage management;
- `UNVERIFIED`:provider 无法 introspect。

检测到 forbidden permission 时阻止 Live readiness。

### A4. LLM Providers (Model)

CLIProxyAPI:

- Running / Stopped / Port conflict / Unauthorized;
- pinned version;
- ChatGPT OAuth;
- `/v1/models` probe;
- discovered GPT models;
- Launch / Re-login。

DeepSeek:

- key missing / connected;
- OS keychain statement;
- probe + test inference;
- discovered models。

Fallback 设置可见但默认 OFF。没有可用 LLM provider 时 Ready 被阻断。

### A5. Risk Defaults

max order notional、max exposure、max daily traded notional、max daily realized loss、stale quote threshold、market-order policy、live inactivity timeout。Hard rules read-only。

### A6. Ready

展示 workspace/base currency、broker/data providers、LLM route、fallback policy、全部 Live account=`DISARMED`。

---

## B. Agent Workspace

### B1. New Thread

建议 prompt 与英文版一致。

### B2. Thread History / Resume

Today / Earlier / Saved;仅恢复 context。

### B3. Ask Mode

轻量回答、可选只读 context、默认无重 plan、无 trade action、可提升到 Research/Backtest/Trade。

### B4. Composer Pickers

Context、Agent Mode、Execution Context badge、Account、Model/provider。

### B5. Research Running

request、plan、Done/Running/Queued、typed tools、interim result。

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

结论、key findings、scenario、evidence/provenance、artifacts;Trade action 仅在 Trade mode 出现。

### B8. Research Result — Crypto

Binance/Bitget comparison、spread/depth、quote provenance、selected venue;market proposal 仅在 Trade mode。

### B9. Immutable Turn Provenance

展示 `agent_mode_at_start`、`execution_context_at_start`、selected account、model/provider、provider attempts、attached context IDs/hashes。

---

## C. Markets and Screening

### C1. Market Explorer

search、US Stocks/Crypto/Screeners、metrics、movers、saved screeners。

### C2. Natural-language Screener Builder

```text
Natural-language request
→ Parsed FilterSpec / RankSpec
→ Inspect/edit
→ Run
→ Candidate set
```

### C3. Screener Result

Candidate table 可启动 Agent research。

### C4. Equity Instrument Detail

quote/chart、tabs、market-data provenance、market session、corporate action、watchlist、`MARKET_CLOSED` / `INSTRUMENT_HALTED` block examples。

### C5. Crypto Instrument Detail

venue quote/chart、bid/ask/spread/depth、source/provider timestamp/received/entitlement/freshness、spot-only warning、Trade action。

### C6. Market Session / Corporate Actions

Open/Closed/Pre-market/After-hours(适用时)、halt、split/dividend fixture、unsupported execution remediation。

---

## D. Watchlists

### D1. Watchlist Library
### D2. Watchlist Detail
### D3. New Watchlist
### D4. Add Instrument to Watchlist

MVP watchlist 使用按需/粗粒度刷新,不暗示常驻 tick-level Warm subscription。

---

## E. Accounts and Portfolio

### E1. Accounts Overview

展示 provider、environment、health、equity/balance、Live arming、last sync。

### E2. Provider-specific Account Detail

Local Paper、Alpaca Paper、T212 Demo/Live、Binance Testnet/Live、Bitget Demo/Live。

展示 environment/account type、connection/auth/stream/reconciliation/execution eligibility、balances、`Available / Reserved / Effective Available`、positions/open orders、permissions/capabilities、credential health、risk summary、IP status、Live ARMED/DISARMED。

### E3. Portfolio

跨账户 base-currency view 展示 FX/stablecoin provenance:source/path、provider timestamp、TradeX received、freshness、quality/depeg。

### E4. Open Order

broker state、proposal/order identity、fill/remaining、reservation、cancel action。

---

## F. Live Execution and Safety

### F1. Account-scoped Live Arming

Arm dialog 绑定单一 account:

```text
Trading 212 Live · ARMED
Binance Live · DISARMED
Bitget Live · DISARMED
```

提供 per-account Arm/Disable + global Disable All。Restart/sleep/auth/reconciliation/risk weakening 仅对受影响账户 disarm。

### F2. Limit Order Approval — Trading 212 / AAPL

展示 immutable proposal、proposal ID/hash、policy version、account/venue/environment、side/qty/type/limit/TIF、notional、`Available / Reserved / Effective Available`、完整 MarketSnapshot provenance(source/provider timestamp/TradeX received/venue/entitlement/age/freshness)、risk、Reject/Approve。

### F3. AAPL Submission / Monitoring

```text
APPROVED
→ RESERVED
→ SUBMITTING
→ ACCEPTED (not a fill)
→ PARTIALLY_FILLED or FILLED
→ reconciliation
```

Identity 不变。

### F4. Market Order Approval — Binance / BTC

expected spend、maximum authorized、bid/ask/spread、provenance、fee/slippage;超过授权直接拒绝。

### F5. Approval Invalidated — Market Changed / Stale

旧 approval 不可执行;需新 proposal + approval。

### F6. Approval Expired

不可复用;未提交时释放 reservation。

### F7. Risk Rejected

确定性 `RISK_REJECTED`,Agent 不可 override。

### F8. Broker Rejected

Order state `REJECTED`;error category `SUBMISSION_REJECTED`。

### F9. Ambiguous Submission / Manual Resolution

```text
SUBMITTING
→ UNKNOWN_RECONCILING
→ query-first reconciliation
→ timeout: reservation frozen + account unhealthy/DISARMED
→ Manual Resolution
```

动作只有 Confirmed not submitted(需证据)、Confirmed submitted + broker order identity、Keep reconciling。没有通用 release-and-continue。

### F10. Live Cancellation

refresh state → approval → `CANCEL_PENDING` → `CANCELLED` 或 fill race。修改订单=cancel + new proposal,无跳过 approval 的 amend。

### F11. Reservation Conflict

```text
Thread A reserves €4,000
Available = €10,000
Reserved = €4,000
Effective Available = €6,000
Thread B requires €7,000
→ RISK_REJECTED · RESERVED_CAPACITY
```

### F12. Risk-policy Change Invalidation

更新 policy → version increment → affected approval invalidated → old/new version + reason;若 weakening 同时 disarm account。

---

## G. Local Paper / Broker Paper / Demo / Testnet

### G1. Local Paper

TradeX-managed simulation,显式 `LOCAL PAPER`,不是 provider truth。

### G2. Alpaca Paper

proposal → provider paper acknowledgement → fill/update → account refresh。

### G3. Trading 212 Demo
### G4. Binance Testnet
### G5. Bitget Demo

复用统一 normalized component:

```text
PROPOSED
→ ACCEPTED / acknowledgement
→ PARTIALLY_FILLED / FILLED
→ account/position refresh
```

非 Live 环境不可与 Live 混淆。

---

## H. Strategies and Backtesting

### H1. Strategy List
### H2. Editable Strategy Sandbox

显示 blocked:broker credentials、OS keychain、Order Gateway、unrestricted network。

### H3. Backtest Running
### H4. Backtest Failed
### H5. Backtest Result

return、Sharpe、Sortino、max drawdown、win rate、profit factor、turnover、equity curve、trade list、reproducibility manifest。

### H6. Backtest Compare

metrics diff、parameter diff、manifest/version links。

---

## I. Artifacts

### I1. Artifact Library
### I2. Artifact Detail
### I3. Provenance Drawer / Modal
### I4. Export

Provenance 包含 Thread、Turn、immutable Turn snapshot、model/provider attempts、tools、sources、market/dataset hashes、related order。

---

## J. Settings

Settings internal subnav 始终可达。

### J1. Providers & Models

分组 Broker/Exchange/Market Data 与 LLM Gateway & Models。Broker 使用 schema-driven configure/test + permission review。

LLM 展示 CLIProxyAPI state/version/OAuth/probe/models、DeepSeek key/probe/models、selected route、`Allow automatic fallback to DeepSeek`(默认 OFF)、LLM error remediation fixture。

### J2. Risk & Limits

```text
persist policy
→ version policy
→ invalidate affected approvals
→ audit
→ if weakening: disarm affected account
→ confirmation/reason
```

Hard safety rules read-only。

### J3. Data & Storage

与 PRD 一致:

- SQLite — transactional/domain;
- DuckDB — persistent 1-minute+ OHLCV + analytical/backtest;
- Filesystem — artifacts/exports/backups;
- Parquet — optional/Phase 2+ large datasets。

含 retention、privacy、Export Workspace、Backup Now。

### J4. Import / Restore Workspace

```text
Choose archive
→ validate manifest/schema
→ import summary
→ confirm
→ restore non-secret state
→ verify credential references
→ reconcile accounts
→ all live accounts DISARMED until healthy
```

不导入 broker secrets。

### J5. Account Health

connection、authentication、private stream、reconciliation、execution eligibility、arming 六个正交状态。

### J6. Appearance

prototype 级 light/system preference。

### J7. About

version、local runtime、docs baseline、prototype disclaimer。

---

## K. Recovery and Error States

### K1. Resume after Sleep
### K2. Startup Reconciliation
### K3. Authentication Failure
### K4. Private Stream Disconnect
### K5. Rate Limiting
### K6. Clock / Freshness Failure

TimeService/clock uncertainty → stale/reconciliation remediation;Live authority fail closed。

### K7. Canonical Error Surface Variants

统一 ErrorRecoveryPanel 映射:

`AUTH_ERROR`、`PERMISSION_ERROR`、`RATE_LIMITED`、`NETWORK_ERROR`、`UNSUPPORTED_CAPABILITY`、`MARKET_CLOSED`、`INSTRUMENT_HALTED`、`INVALID_ORDER`、`INSUFFICIENT_FUNDS`、`RISK_REJECTED`、`SUBMISSION_REJECTED`、`SUBMISSION_AMBIGUOUS`、`STREAM_DISCONNECTED`、`STATE_STALE`、`RECONCILIATION_REQUIRED`、`MODEL_UNAVAILABLE`、`QUOTA_EXCEEDED`、`OAUTH_EXPIRED`、`INTERNAL_ERROR`。

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
| ACCEPTED | broker acknowledged, not fill |
| PARTIALLY_FILLED | fill + remaining |
| FILLED | authoritative fill |
| CANCEL_PENDING | pending + race warning |
| CANCELLED | authoritative cancel |
| REJECTED | broker rejected;`SUBMISSION_REJECTED` |
| EXPIRED | expired;not reusable |
| UNKNOWN_RECONCILING | ambiguous + query-first + Manual Resolution |

---

# 7. Critical UX Invariants

1. Agent Mode 与 Execution Context 独立。
2. Ask/Research + Live account 始终只读。
3. 选择 Trade 或 Live Context 不会布防。
4. Live arming 显式且 account-scoped。
5. 布防一个账户不影响另一个。
6. Global Disable All 立即解除全部 Live account 对新提交的布防。
7. Arming 不等于 approval。
8. 每个 Live order/cancel 都需要 transaction-specific approval。
9. OrderDraft edit 会生成新 immutable proposal identity。
10. Material change invalidates approval。
11. Risk-policy relevant change invalidates approval;weakening disarms。
12. Stale data 或 untrustworthy time blocks Live authority。
13. Approval 展示完整 market provenance。
14. Capacity 被占用时 `RESERVED` 可见。
15. Broker acknowledgement 不是 fill。
16. Ambiguous submission 不 blind retry / casual release。
17. Manual Resolution 基于证据并审计。
18. Reservation 防止 cross-thread double spend。
19. Agent 不能改 risk policy。
20. Strategy 无 broker credential/Order Gateway。
21. Broker/exchange state authoritative。
22. Local Paper/Paper/Demo/Testnet/Live 文字上明确区分。
23. Generic Enter/form submit 不隐式 approve Live transaction。
24. Cross-provider LLM fallback 默认 OFF;switch/fallback 披露并审计。
25. LLM unavailable 只影响 agent turn,不影响 control-plane/reconciliation。
26. Local-first prototype 不暗示 cloud share-link;使用 local export/copy。

---

# 8. Component Inventory

与英文版一一对应:AppShell、Sidebar、SettingsSubnav、CompactMoreNav、ThreadHistory、TopBar、AgentModeBadge/Picker、ExecutionContextBadge、AccountLiveStateBadge、DisableAllLiveControl、LLMGatewayStatusBadge、ModelProviderPill、Composer、ContextPicker、AccountPicker、ModelPicker、TurnProvenancePanel、ContextPanel、PlanCard、ToolCard/ToolErrorCard、MarketSnapshotProvenance、MarketStatusBanner、CorporateActionPanel、FXProvenancePanel、ProviderCredentialSchemaForm、PermissionReviewPanel、AccountHealthPanel、OrderDraftEditor、OrderProposalCard、RiskCheckPanel、ReservationPanel、LiveArmModal、LiveApprovalModal、MarketOrderApprovalModal、ApprovalInvalidatedModal、RiskRejectedModal、ReservationConflictModal、ManualResolutionModal、OrderTimeline、CancellationApprovalModal、SimulatedOrderFlow、ErrorRecoveryPanel、StrategyEditor/Inspector、BacktestProgress/Metrics、ArtifactDetail/ProvenanceModal、WorkspaceImportModal、RecoveryPanel、SuccessToast/Modal。

---

# 9. 可访问性与键盘交互

- 所有互动 control keyboard reachable;
- visible `:focus-visible`;
- logical tab order;
- modal 使用 dialog semantics + accessible name;
- Live approval 初始 focus 不直接落在 Approve;
- generic `Enter` 不 approve Live transaction;
- `Escape` 可 dismiss/reject,绝不 approve;
- status/toast/order change 使用适当 `aria-live`;
- 状态不只靠颜色;
- respects reduced-motion;
- narrow layout 保持关键 action 可达。

---

# 10. 响应式行为

低于 900px:

- sidebar collapse;
- compact bottom nav;
- `More` 暴露 Watchlists/Artifacts/Settings/Account Health;
- multi-column stack;
- context panel 下移;
- strategy side panels 可 collapse;
- composer chips 横向 scroll;
- Live identity/arming/provenance/risk/Reject/Approve/Disable Live 仍可达。

v1.0 是 responsive desktop/tablet-window,不是 native mobile Live execution product。

---

# 11. 原型 Fixture Data

AAPL、BTC/USDT、balance、timestamp、risk number、permission schema、FX route、quota/model health 均为 fixture。`US Market Data (fixture)` / `FX Provider (fixture)` 等标签避免把尚未解决的 OD 假装成已选 provider。

---

# 12. 开发交接

Production frontend 用真实 Codex Thread/Turn/Item event、immutable Turn snapshot、market data + TimeService provenance、broker adapter read、provider schema/capability metadata、Control Plane authority event、risk/reservation、Order Gateway、reconciliation/manual-resolution、SQLite + DuckDB 替换 fixture。Parquet 仅在大型数据集需要时引入。

Frontend 只渲染 authority decision,不拥有金融 authority。

---

# 13. 原型完成标准

Revision C product-review-complete prototype 必须可点击覆盖:

```text
Onboard
→ schema-driven provider + permission review
→ CLIProxyAPI / DeepSeek + fallback policy
→ risk defaults
→ Ask / Research / Backtest / Trade
→ separate Execution Context
→ create/resume thread + immutable turn provenance
→ context/account/model
→ market research/screener
→ market session/halt/corporate action
→ account/portfolio + reservation/FX provenance
→ Local Paper / Alpaca Paper / T212 Demo / Binance Testnet / Bitget Demo
→ Trade + one Live account
→ arm exactly that account
→ live limit/market approval + full provenance
→ RESERVED → SUBMITTING → ACCEPTED → fill
→ reservation conflict
→ risk-policy invalidation
→ stale/expired/risk-rejected/rejected/ambiguous
→ evidence-based Manual Resolution
→ cancel live order
→ full backtest metrics
→ artifact provenance/export
→ workspace export/import
→ account health + canonical errors
→ LLM unavailable/quota/OAuth recovery
→ narrow-window + keyboard/focus safety
```

Coverage Matrix 是 prototype evidence 与 implementation-only requirement 的权威记录。
