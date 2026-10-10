# TradeX 前端架构需求与设计（ARD）

**契约澄清日期：** 2026-09-05（RevC）；S18 本地账户删除契约于 2026-09-24 澄清为通过完整 connection ID 识别同名记录。原型行为仅为证据，以 QA Report 记录的缺陷和待验证门槛为准。

**版本：** v1.0 Revision C (RevC)  
**状态：** 工程基线  
**范围：** 仅桌面前端  
**目标技术栈：** Tauri + React + TypeScript  
**来源基线：** `TradeX_PRD_v1.0_RevC_zh.md`、`TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md`；原型观察另见 QA Report\
**语言：** 简体中文

> 本 ARD 将 RevC 产品需求落实为可实施的前端架构。产品语义和安全不变量以 RevC PRD / UI Spec 为准；下文中的模块边界、状态存储拆分、IPC contract、目录结构和测试结构属于工程实现层面的架构决策。

---

## 1. 目的

TradeX 前端是一个 local-first 桌面工作台，用于持久化 Agent Thread、市场研究、回测、模拟交易以及经审批门控的真实交易。其交互体验应类似 Codex Agent Workspace，但金融执行必须始终作为独立的高权限工作流处理。

前端负责：

- 渲染持久化的 Thread / Turn / Item 时间线；
- 将 Agent Mode 与 Execution Context 作为两个独立维度展示；
- 展示市场、账户、组合、策略、Artifact 与执行状态；
- 展示后端产生的可信 Risk、Approval、Reservation 与 Reconciliation 决策；
- 收集明确的用户操作，例如账户 Arm、真实交易审批、撤单审批、Provider 配置和 Manual Resolution；
- 持续展示模型/provider provenance 与 market-data provenance；
- 在窄窗口、键盘操作、断线、数据过期和后端失败时仍保持安全。

前端**不是金融权限主体**。前端不能自行判断真实订单是否有效，不能持有券商凭据，也不能把通用 Agent/Codex approval 转换为交易授权。

---

## 2. 架构驱动因素

### 2.1 产品驱动

1. Codex 风格的持久工作台，而不是无状态聊天页面。
2. Local-first 桌面体验与快速事件渲染。
3. Agent-native 的结构化时间线卡片。
4. Research-first，并允许用户显式进入 Trade。
5. Agent Mode 与 Execution Context 明确分离。
6. 真实交易 Arm 必须按账户隔离。
7. 每笔真实交易采用 transaction-specific、single-use approval。
8. 每个 Live approval 必须完整展示 market snapshot provenance。
9. Paper / Demo / Testnet / Live 必须有明确文字身份。
10. 对恢复与 reconciliation 使用显式状态，而不是乐观猜测。

### 2.2 质量驱动

- IPC 事件到达后至 UI 可见提交：代表性交互 p95 < 100 ms。
- 核心 UI 可通过键盘访问，并具有明显 focus。
- `Enter` 绝不能隐式批准真实订单或撤单。
- 窄桌面/平板窗口必须保留真实交易安全控制。
- 状态不能只靠颜色表达。
- 外部 telemetry 默认关闭且需 opt-in；前端日志不得包含秘密信息。

### 2.3 信任边界驱动

前端只能通过类型化后端命令请求高权限操作。它不能访问 OS keychain、provider signing code、Order Gateway 或原始 broker credentials。

---

## 3. 前端系统上下文

```mermaid
flowchart LR
    U[用户]
    UI[React / TypeScript UI]
    FE[Frontend Application Layer]
    IPC[Tauri IPC Client]
    CP[Trusted Rust Control Plane]
    COD[Codex App Server]
    MD[Market / Portfolio Services]
    EX[Risk / Approval / Reservation / Execution]

    U --> UI
    UI --> FE
    FE <--> IPC
    IPC <--> CP
    CP <--> COD
    CP <--> MD
    CP <--> EX
```

### 架构规则

UI 只渲染后端权威状态；不得根据 Agent 文本、网络超时或客户端乐观状态推断金融操作成功。

---

## 4. 技术决策

| 领域 | 决策 | 原因 |
|---|---|---|
| Desktop shell | Tauri | 本地桌面集成、较小体积、Rust Control Plane 安全边界 |
| UI framework | React + TypeScript | 适合组件化 Codex 风格 Workspace，强类型 |
| Server-state cache | TanStack Query | Query 生命周期、失效与必要的轮询/重验证 |
| UI/session state | Zustand 或等价轻量 store | 管理明确的本地 UI 状态，不把 server state 变成客户端权威状态 |
| Runtime transport | Typed Tauri commands/events | 所有高权限操作都经过 Rust 边界 |
| Agent stream mapping | 将 Codex JSON-RPC/JSONL 转成 typed frontend events 的 adapter | 避免协议细节扩散到全部 UI 组件 |
| Charts | 轻量金融图表库 | 本地快速渲染；图表层不拥有权限语义 |
| Styling | CSS modules/tokens 或等价方案 | 可控 design system、focus 和 reduced-motion |
| Schema validation | Zod 或生成式 runtime validator | 在 IPC/event 边界拒绝格式错误的数据 |

React meta-framework 不在本 ARD 中强制指定。TradeX v1.0 是桌面应用，不依赖 SSR。

---

## 5. 前端进程与分层模型

```text
React Presentation
    ↓
Feature Controllers / View Models
    ↓
Frontend Domain Selectors
    ↓
Query + Event Stores
    ↓
Typed IPC Client
    ↓
Rust Control Plane
```

### 5.1 Presentation layer

只包含可复用视觉组件，例如：

- `AppShell`
- `Sidebar`
- `TopBar`
- `ThreadTimeline`
- `Composer`
- `ContextPanel`
- `MarketSnapshotProvenance`
- `OrderProposalCard`
- `RiskCheckPanel`
- `ReservationPanel`
- `LiveArmModal`
- `LiveApprovalModal`
- `CancellationApprovalModal`
- `ErrorRecoveryPanel`
- `WorkspaceImportModal`

Presentation component 接收明确的 state 和 callback，不直接调用 broker API，也不能读取 secret。

### 5.2 Feature-controller layer

负责协调：

- 创建/恢复 Thread；
- 开始/取消/重试 Turn；
- 切换 Agent Mode；
- 选择 execution account/context；
- 对单个 live account Arm/Disarm；
- 生成/刷新 live proposal；
- 提交批准/拒绝；
- 批准撤单；
- 单独准备精确获批的 Live CANCEL，并恢复已保存的 `RESERVED` attempt；不得将其显示为提供方已撤单；
- 处理 ambiguous submission；
- 配置 provider/model；
- 导入/恢复 Workspace。

### 5.3 Frontend domain-selector layer

只生成展示所需的派生状态，例如：

- `canShowLiveApproval`
- `executionContextLabel`
- `isSelectedLiveAccountArmed`
- `effectiveAvailableDisplay`
- `isMarketSnapshotFresh`
- `isTurnExternalProcessorChanged`

Selector 可帮助渲染，但不能产生执行权限。`can_execute`、risk、freshness eligibility、reservation validity 与 account health 始终以后端为权威。

---

## 6. 应用导航架构

RevC 固定一级导航：

```text
New Thread
Threads
Markets
Watchlists
Accounts
Strategies
Artifacts
Settings
```

Portfolio 与 Orders 是 account/context surface，不是永久一级导航。

### 6.1 Route 模型

推荐逻辑路由：

```text
/
/thread/:threadId
/markets
/markets/:instrumentId
/watchlists
/watchlists/:watchlistId
/accounts
/accounts/:accountId
/strategies
/strategies/:strategyId
/artifacts
/artifacts/:artifactId
/settings/:section?
```

这些是本地桌面导航状态，不是公开 Web URL。

### 6.2 窄窗口导航

进入窄布局后：

- sidebar 折叠；
- 主 Workspace 仍然可访问；
- secondary inspector 堆叠到主内容下方；
- 侧栏折叠后，在抽屉或 `More` 保留 New Thread、Thread History、Watchlists、Artifacts、Settings、Account Health；Provider Configure/Models 与表格行动作通过换行/堆叠保留；
- live account identity、arming status、Reject、Approve、Cancel、Manual Resolution 和 Disable All Live 均必须无需 hover 即可访问。

---

## 7. 核心前端 Domain Model

前端镜像后端 Domain Object，但不拥有其权威生命周期。

### 7.1 Thread

```ts
interface ThreadSummary {
  workspaceId: string;
  threadId: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  defaultAgentMode: AgentMode;
  linkedAccountIds: string[];
  linkedInstrumentIds: string[];
  linkedStrategyIds: string[];
  linkedArtifactIds: string[];
}
```

### 7.2 不可变 Turn start snapshot

```ts
interface TurnSnapshot {
  turnId: string;
  agentModeAtStart: AgentMode;
  selectedAccountId?: string;
  executionContextAtStart: ExecutionContext;
  capabilityLevelAtStart: CapabilityLevel;
  modelId: string;
  modelProvider: ModelProvider;
  attachedContextIds: string[];
  attachedContextHashes: string[];
  startedAt: string;
}
```

Turn 启动后的 picker 修改不得改变该对象。

不可变快照只保存启动时输入。Provider attempts 是只追加的 Turn 事件，完成时间属于 Turn 生命周期记录。历史 Turn/Artifact 溯源读取已保存快照和事件历史，不能读取当前 composer 选择器。

### 7.3 Agent Mode

```ts
type AgentMode = "ASK" | "RESEARCH" | "BACKTEST" | "TRADE";
```

### 7.4 Execution Context

```ts
type ExecutionContext =
  | "NONE_READ_ONLY"
  | "HISTORICAL_SIMULATION"
  | "LOCAL_PAPER"
  | "ALPACA_PAPER"
  | "TRADING212_DEMO"
  | "TRADING212_LIVE"
  | "BINANCE_TESTNET"
  | "BINANCE_LIVE"
  | "BITGET_DEMO"
  | "BITGET_LIVE";
```

### 7.5 Live account state

```ts
type ArmingState = "DISARMED" | "ARMED";

interface LiveAccountStatus {
  accountId: string;
  environment: "LIVE";
  armingState: ArmingState;
  health: AccountHealth;
  lastReconciledAt?: string;
  capabilitySummary: string[];
}
```

禁止使用 workspace 级 `liveArmed: boolean`。

### 7.6 Capability 预检

```ts
type CapabilityLevel = "C0" | "C1" | "C2" | "C3" | "C4" | "C5" | "C6";
type ToolId = "public_market_read" | "account_read" | "historical_simulation" | "paper_demo_testnet_execution" | "live_order_proposal";

interface CapabilityDecision {
  level: CapabilityLevel;
  allowedTools: ToolId[];
  executionAllowed: boolean;
  reason?: string;
}
```

Composer 可以通过 `agent.capabilities` 查询待发送状态，但 `turn.start` 必须接收相同输入并在可信边界重新计算 decision。C5/C6 以及未知或不允许的工具始终不可用。

### 7.7 上下文目录与临时 picker 状态

```ts
interface ContextCatalogEntry {
  contextRef: { kind: "account"; id: string; hash: string };
  label: string;
  providerId?: string;
  environment?: string;
  readOnly: boolean;
  available: boolean;
  availabilityReason?: string;
}
interface ContextCatalog {
  entries: ContextCatalogEntry[];
  emptyStates: { kind: "instrument" | "account" | "strategy" | "backtest" | "artifact"; availabilityReason: string }[];
}
```

`context.catalog` 由后端拥有，返回已持久化的 account refs 以及未来目录的明确空状态。picker 保持临时 pending 列表：Attach 替换列表，Cancel 不改变列表，移除 chip 只影响下一 Turn。Ask/Research 中的 Live account 显示 `LIVE · READ-ONLY`；Backtest 只将其保留为可选 read-only seed。创建 Thread 或开始 Turn 时，后端重新校验 ref ID 和 hash。`turn.start` 省略 `attachedContexts` 时兼容性沿用已保存 Thread refs，显式空数组清除本轮 refs，`null` 无效。

可用的附加 account 上下文会为 Ask、Research 和 Backtest 暴露只读账户工具；Trade 需要执行账户时仍必须使用独立的 Account picker。

### 7.8 Typed research registry 与 result preview

Capability summary 必须渲染独立的 `researchTools` data-plane registry。它只包含 read-only IDs `public_market_read`、`account_read` 和 `historical_simulation`；`allowedTools` 中的金融 authority IDs 不是 research tool。Composer 提供已授权 research tool selector 和 focus selector（`GENERAL`、`EQUITY`、`CRYPTO_SPOT`），然后可以调用 `research.run`，展示 typed state、结论、canonical instrument refs、findings、有界 scenarios、artifact refs、source/provider 时间、新鲜度、质量、限制和 marker。`CRYPTO_SPOT` 卡片只渲染带 provenance 的 Binance/Bitget typed venue 条目、选定 venue、spread/depth 和 quote age；数据缺失时保持缺失并展示 blocked/unavailable/degraded 原因。query 文本始终是不可信输入，不得被写入 result payload。

Send 之前，Composer 可以成对附加 `ResearchToolInvocation` 与 `ResearchToolResult`。`turn.start` 在可信边界重新校验 pair；缺失或篡改的 pair 显示 `RESEARCH_RESULT_INVALID` 且 Thread 保持不变。合法 result 会渲染为 `research_result` timeline item，并将完整的有界 typed result 与 item 一起持久化，使 reload 后仍保留 evidence card；其 marker 在最终 Turn 输出可见。selected tool、focus、mode、execution context、account 或附加 refs 改变时，preview 临时状态会清除。Trade mode 卡片可以显示 disabled/read-only proposal 入口；Ask 和 Research 不渲染 Trade CTA，卡片也不会调用 order、approval、arming、Gateway 或 live-risk。Synthetic fixture 条目必须可见标记，不能作为 provider 事实。

---

## 8. Agent Mode × Execution Context UX 状态机

前端必须显式表达合法组合。

| Agent Mode | 无账户 | Paper/Demo/Testnet | Live account |
|---|---|---|---|
| Ask | read-only | read-only context | read-only context |
| Research | read-only | read-only context | read-only context |
| Backtest | historical simulation | 可作为 portfolio seed | 可作为 portfolio seed |
| Trade | 必须选择 execution account | 模拟/provider-hosted execution | live proposal flow，但仍受安全门控 |

### UI 行为

- 非法组合 disable 并解释原因；
- 切换 Agent Mode 不得静默替换 account/environment；
- Ask/Research 中选择 Live account 仍然是 read-only；
- 选择 Trade 不得自动 Arm；
- mode/account/model 的变化只作用于下一 Turn，不修改 in-flight turn。

---

## 9. 状态管理策略

TradeX 必须区分后端权威状态与临时 UI 状态。

### 9.1 TanStack Query 管理

用于后端拥有的 snapshot：

- thread summaries / detail；
- account detail / health；
- portfolio snapshot；
- watchlists；
- market/instrument snapshot；
- provider/model health；
- risk-policy summary；
- 按 workspace/proposal 查询的 RiskDecision history；
- strategy/backtest metadata；
- artifacts；
- open orders 与 reconciliation view。

### 9.2 Event-driven state

订阅后端事件：

- Codex turn/item streaming；
- tool lifecycle；
- order state change；
- fills/position updates；
- private-stream degradation；
- reconciliation progress；
- account arming change；
- risk-policy invalidation；
- CLIProxyAPI health/provider-attempt change。

事件更新 normalized cache 或追加 immutable timeline item。

### 9.3 Zustand 管理

只保存：

- 当前 workspace/thread route；
- composer draft；
- drawer/inspector 开关；
- 展开的 timeline item IDs；
- 发送前的 picker 临时选择；
- 本地 theme/appearance；
- transient modal stack。

不得只在 Zustand 中保存 approval validity、broker order truth、reservation truth 或 account health。

---

## 10. IPC Contract 架构

命令/结果/事件封装、精确命令名、payload 要求、schema 版本规则和重放协议统一由 [Backend ARD §41–42](./TradeX_Backend_ARD_v1.0_RevC_zh.md#41-backend-api--ipc-surface) 定义。前端 IPC 使用该契约，不得为既定命令另造 trade.draft.*、trade.approval.* 等别名。

### 10.1 客户端职责

- 命令使用 schemaVersion 1，并验证结果/事件中的版本。
- 匹配 requestId；成功与错误使用互斥结果类型。
- 修改请求携带预期权威状态版本；状态陈旧时重新加载并取得新的同意，不能盲目重试。
- 按聚合身份和 sequence 订阅/重放；重复事件去重，sequence 缺口、冲突重复或重放范围不可用时重新加载一致快照。
- 权威投影缺失/不兼容时，金融控件保持不可用。纯模型故障禁用受影响 Agent Turn，同时保留账户/对账操作。
- 导航、尚未提交的选择器/草稿变更属于本地 UI 状态。类型化的 account/trade/model/domain 命令只通过受信 Tauri 控制面边界。

### 10.2 安全规则

前端不能获得 keychain secret。凭据输入只使用专用 secure-entry handoff，后续命令引用凭据 ID。UI 不签名提供方请求，也不直接调用 Gateway。金融命令携带规范 account/proposal/approval/cancellation-intent ID，不能从通用 Agent 审批取得权限。客户端检查权威契约要求的安全/错误字段，显示后端修复措施，不另造订单状态。

---

## 11. Thread / Turn / Item 渲染架构

### 11.1 Timeline item registry

采用 typed renderer registry，避免巨型条件组件。

```ts
const itemRenderers: Record<ItemType, React.ComponentType<any>> = {
  user_message: UserMessageItem,
  agent_message: AgentMessageItem,
  plan: PlanItem,
  tool_call: ToolCallItem,
  tool_result: ToolResultItem,
  market_snapshot: MarketSnapshotItem,
  research_evidence: ResearchEvidenceItem,
  portfolio_snapshot: PortfolioSnapshotItem,
  screener_result: ScreenerResultItem,
  backtest_result: BacktestResultItem,
  order_draft: OrderDraftItem,
  order_proposal: OrderProposalItem,
  risk_check: RiskCheckItem,
  approval_request: ApprovalRequestItem,
  reservation: ReservationItem,
  order_update: OrderUpdateItem,
  fill: FillItem,
  error: ErrorItem,
  artifact: ArtifactItem,
};
```

### 11.2 Item lifecycle

UI 支持：

```text
started → streaming → completed
                 ↘ failed
```

Turn 包含 running、cancelled、interrupted、completed、failed 等状态。

### 11.3 Resume 行为

恢复 Thread 可以恢复导航/default context，但不得把以下内容恢复为有效权威状态：

- 已消费/过期 approval；
- stale market snapshot；
- restart 前的 `ARMED`；
- 用当前 picker 值覆盖历史 in-flight turn 的 provider/model。

---

## 12. Composer 架构

Composer controls 是一级产品状态：

```text
@ Context | Account | Agent Mode | Model | Send
```

### 12.1 Send 流程

```text
validate local draft
→ request backend compatibility validation
→ freeze TurnSnapshot inputs
→ persist/start turn
→ render streaming items
```

### 12.2 Provider disclosure

发送前展示下一 Turn 的处理路径：

```text
CLIProxyAPI → ChatGPT
或
CLIProxyAPI → DeepSeek
```

如果用户显式启用了 fallback 且 provider 在 turn 中变化，timeline 必须记录并可见地展示变化。

### 12.3 Ask mode

Ask 是轻量 read-only workflow，不暴露当前市场交易动作，也不默认生成 research artifact。

---

## 13. Market 与 Portfolio UI 架构

### 13.1 Canonical instrument identity

前端 route 与 cache key 使用 canonical `instrument_id`，不使用 provider symbol 作为 domain identity。

### 13.2 Market snapshot model

```ts
interface MarketSnapshotProvenance {
  marketSnapshotId: string;
  instrumentId: string;
  source: string;
  venue?: string;
  providerTimestamp: string;
  receivedTimestamp: string;
  entitlement: "REALTIME" | "DELAYED" | "UNKNOWN";
  freshness: "HEALTHY" | "STALE" | "CLOCK_UNCERTAIN";
  quoteAgeMs?: number;
}
```

只要 market data 参与 live authority decision，approval view 就必须展示完整 provenance。

### 13.3 Trusted time health

Settings / Account Health 展示 workspace-scoped 的 `time.status`，以语义状态呈现 wall-clock、monotonic reading、provider offset、observed timestamp、有限长度 reason；当 confidence 为 `CLOCK_UNCERTAIN` 或 `STALE` 时提供键盘可达的 `time.revalidate` 动作。面板使用 `role="status"` 与 `aria-live="polite"`，保留可见焦点，并在 768 px 与 390 px 保持同样的阻塞原因。renderer 不能覆盖后端读数；只有 Control Plane 报告 `TRUSTED` 后 Live eligibility 才能恢复。

### 13.4 Market session 与 corporate actions

`MarketDetail` 在既有 source/snapshot panel 旁展示后端持有的 `marketState`、`corporateActions` 与 `adjustmentStatus`。session 文本覆盖 `OPEN`、`CLOSED`、`EXTENDED_HOURS`、`HALTED`、`MAINTENANCE`、`SUSPENDED`、`DEGRADED`、`UNKNOWN`；venue、source status、next boundary、calendar version、provider time、observed time 和 `timeConfidence` 保持可见。OD-005 缺失或 blocked 时维持 `UNKNOWN`/`UNAVAILABLE`，fixture 不得被呈现成实时 session 或已调整历史。

面板使用语义 heading、`role="status"`/`aria-live="polite"` 宣布 session 变化，提供可见焦点、键盘可达的 source remediation，并在颜色之外使用文本标签。`MARKET_CLOSED` 与 `INSTRUMENT_HALTED` 作为确定性的 blocking state 展示；renderer 不计算 execution eligibility。Corporate-action 行有界，并展示 action type、description、effective/announced time、source 与 adjustment status。

### 13.5 FX/stablecoin provenance

跨账户归一化必须展示：

- workspace base currency；
- conversion pair/path；
- source；
- timestamps；
- freshness；
- quality/depeg warning。

前端不得在没有后端 conversion state 的情况下把 USDT 静默展示为 USD 等价物。

### 13.6 Local Paper surface（S16）

Local Paper account 必须显示为 `Local Paper · LOCAL_PAPER` 与 `TradeX simulation · TRADEX_SIMULATION`。Accounts surface 读取 `paper.get`，展示确定性 scenario/quote 状态、cash、reserved cash、positions、open orders、fills 与 event history，并只提供 simulation action。Order Drafts surface 通过 `paper.order.submit` 提交已选择的 immutable Local Paper proposal；resting 或 partial order 必须先经过显式 cancellation dialog，再调用 `paper.order.cancel`。`paper.quote.refresh` 与 `paper.scenario.set` 只能由 Trade surface 显式触发，不能成为 Agent action。

Renderer 在 reload/reopen 后把 Local Paper projection 作为权威状态。Loading、empty、error 与 retry 使用 status/alert 语义；submit/cancel dialog 使用 `role="dialog"`、`aria-modal`、accessible name，仅显式 action 接受键盘 Enter，并把焦点恢复到触发 proposal 或 Local Paper summary。每个 simulation result 保留 `LOCAL`、`TRADEX_SIMULATION` 与 `not provider truth` 文本披露。Portfolio 展示 simulation provenance 与 `Live risk: Blocked`；Local Paper control 不渲染 approval、arming、reservation、broker acknowledgement 或 Live readiness。

### 13.7 Alpaca Paper 订单提交（S17 #57）

Accounts surface 只从 provider observation 展示 Alpaca Paper buying power，并同时标注账户币种；缺失时显示 `Unavailable`。Order Drafts surface 在显式的 Paper-only 确认 dialog 前展示不可变的 `ALPACA_PAPER` Proposal 及其绑定账户、环境、canonical instrument、方向、数量/金额、订单类型、有效期和限价。用户必须确认审阅过的精确 Proposal；此路径与 Local Paper simulation 和 Live approval 分离。

提交后 renderer 从持久化 attempt projection 读取状态，并区分 `SUBMITTING`、`ACKNOWLEDGED`、`UNKNOWN_RECONCILING` 与 `REJECTED`。Acknowledgement 标记为 provider 已接受，不能称为成交证据。Reload 后恢复已保存 attempt；unknown attempt 只允许按已保存 client order ID 查询式对账；重复提交展示已有 attempt，不能再发 POST。错误保留有界的 remediation 文本。Loading/error 使用 status/alert 语义，确认 dialog 会恢复焦点；在 390、768、1280 px 下账户与订单的 Paper identity 均保持可读。Alpaca Paper control 不授予 Live 或 Agent 提交权限。

### 13.8 Alpaca Paper 订单簿与撤单审阅（S17 #58）

Order Drafts surface 读取当前 Alpaca Paper account 持久化的 `alpaca-paper-order-book`，并提供显式刷新。界面标记 `ALPACA_PAPER` 环境、`TRADE_X` 或 `EXTERNAL` 来源、provider order status、filled/remaining quantity、provider 时间、TradeX observation 时间，以及最近一次完整读取为 current、stale、degraded 或尚未读取。Open orders、订单历史和 `FILL` activities 分开显示；读取不完整时保留最近一次完整数据并说明 degraded 原因。

只有当前仍 open 的 provider order 才显示撤单操作。弹出确认框前，前端先请求后端重新读取该订单。确认框显示 account、Paper 环境、provider order、标的、filled quantity 与 remaining quantity。只有用户明确确认后才调用 typed cancel command。Provider acknowledgement 显示为 `CANCEL_PENDING`；后续 provider observation 确认终态后才显示 `CANCELLED`。订单 identity/status 已变化时要求重新审阅。键盘关闭确认框不会发送请求，焦点返回触发撤单的订单控件。这些操作不会撤销 Local Paper 或 Live 订单。

### 13.9 Alpaca Paper 实时更新与流健康状态（S17 #59）

Accounts surface 显示文字形式的私有流与 reconciliation 状态，以及最近成功接收 trade update 的时间。Order Drafts surface 在已保存的 Paper order book 旁显示相同健康状态；account aggregate 报告 stream 更新后重新读取订单簿，并将 fill observation 标记为 `trade_updates` 或 REST `FILL` activity。部分成交、完全成交、拒绝、撤单、过期和未知 provider 状态保持清晰区分。断流或对账不完整时显示 stale/degraded 文本并保留已保存订单/成交；不会将其呈现为 current。健康状态不授予 Local Paper、Live、Agent、approval 或 arming capability。

### 13.10 Trading 212 Demo 订单提交（S18 #61）

Order Drafts surface 仅在独立确认后，通过 `trading212.demo.order.submit` 提交所选的不可变 `TRADING212_DEMO` Proposal。确认界面标识 Demo 环境并准确展示标的、方向、数量、订单类型、限价和有效期。只支持 BASE 数量型 Market-DAY 与 Limit-DAY/GTC；Market 确认还显示 extended-hours 已关闭。Reload 后通过 `trading212.demo.order.attempt.get` 恢复已保存 attempt。UI 区分 `SUBMITTING`、`ACKNOWLEDGED`、`UNKNOWN_RECONCILING` 和 `REJECTED`；不能把 acknowledgement 称作成交。重复提交只读取已保存 attempt，不再发第二个 POST。未知结果禁用重试并保持冻结，因为 provider 不公开 TradeX client-order identity。所有操作只能由主 Trade UI 触发；Agent 与 Live 均不能进入此路径。

### 13.11 Trading 212 Demo 订单簿（S18 #62）

Order Drafts 允许用户选择一个已连接的 Trading 212 Demo 账户，并在选择/重新加载时读取其已保存订单簿。显式控件可刷新待处理订单、刷新某个准确且已保存的待处理订单详情，或每次加载一个有界历史页。面板标明 `TRADING212_DEMO`、provider order ID、`TRADE_X` / `EXTERNAL` 来源、原始与归一化 provider 状态、订单类型/有效期、提交/观察时间、精确累计成交数量/金额，以及可确定时的剩余数量。只有已记录 provider order ID 完全匹配时才显示 TradeX attempt；不关联相似订单。Provider 缺失值保持为 `Unavailable`。

UI 区分尚未同步、加载、空、当前、stale/degraded 和限流读取，显示最近成功读取与 endpoint 重试时间，并在读取不完整后保留已保存观察。历史控件显示页数/是否完成，且不能触发自动轮询。只有已保存观察仍标记订单待处理时才提供详情控件；详情返回终态后会移除该操作。列表和控件支持键盘及窄屏阅读。累计成交金额在币种可用时与 provider 报告的币种一起显示；UI 不换算或推断单位。不合成 fill 行，也不增加 stream 或 Live/Agent 操作。

### 13.12 Trading 212 Demo 撤单（S18 #63）

对当前待处理且 provider 状态在允许列表内的 Demo 订单，“复核撤单”会先刷新该准确订单详情。只有返回的订单簿与准确订单均为当前状态、仍待处理/可撤单，并且属于已选 Demo connection 与远端账户时，才打开确认框。对话框展示捕获的准确账户/环境和完整 provider order ID、原始/归一状态、精确已成交与剩余数量、可用时的成交金额/币种及观察时间；文案明确 provider 接受不代表撤单已完成。点击“继续检查”或按 Escape 只关闭，不会写入；键盘焦点从安全的关闭操作开始，在框内循环，关闭后回到触发控件或逻辑替代位置。确认仅针对已捕获的 connection 与订单簿版本发送一次 `trading212.demo.orders.cancel`。

收到 HTTP 200 后显示撤单待确认，同时保留 provider 原始状态；超时或未知结果也保持 pending/unknown，不能重新提交。继续保留准确订单详情刷新用于对账，隐藏再次撤单控件，并让后续 provider 事实优先处理部分/全部成交竞态。只有 provider 终态才显示最终结果。将撤单 endpoint 重试时间与其他每账户门控一同展示。这些操作仅限 Demo；陈旧、断开、未知或终态订单不提供撤单，且须在 390/768 px 验证布局与键盘行为。

### 13.13 Trading 212 Demo 本地账户删除（S18 #66）

Accounts 仅在所选 Trading 212 Demo connection 处于 FAILED 或 DISCONNECTED 且 credential health 为 MISSING 时提供“删除本地账户”。原生确认框通过完整 connection ID、账户 label、provider 和 environment 标识捕获的准确记录，明确说明将删除 TradeX 本地账户及账户/订单簿观察，并说明 provider key/order 与其他所有连接保持不变。Cancel/Escape 为只读操作。确认只调用 `account.delete`；后端资格校验仍是权威边界。`ACKNOWLEDGED` attempt 在其准确关联订单获得持久化且已识别的终态观察、`pending: false` 前仍属未解决。成功后失效 account list/detail/context 查询，移除所选详情，播报完成，并将焦点返回触发控件或 `Account connections`。若状态陈旧、存在未解决活动或存储失败，显示 accessible 错误并保留账户视图供恢复。已有连接选择与安全的新账户输入仍可使用。

---

### 13.14 Binance Spot Testnet Proposal 提交与恢复（S19 #68）

Order Drafts 仅对绑定已连接 Binance `TESTNET` 账户的不可变 `BINANCE_TESTNET` Proposal 提供提交。确认前捕获准确 connection ID、账户 label 和远端账户 ID，并与 Proposal hash、instrument/venue、方向、数量类型/数值、订单类型、有效期、限价和 maximum spend 一同展示。用户确认后重新读取 Proposal 和账户；若 Proposal 已变化或捕获的 Testnet 账户 identity 不再匹配，则停止操作并要求重新审阅。只有主 Trade UI 可调用有类型的 Testnet submit command。

选择/重新打开 Proposal 时读取已保存 attempt。UI 区分 `SUBMITTING`、`ACKNOWLEDGED`、`UNKNOWN_RECONCILING` 与 `REJECTED`；acknowledgement 仅表示 Binance 已接受，绝不展示为 fill。重复提交只读取已保存 attempt，不能再发 POST。`UNKNOWN_RECONCILING` 禁用重新提交，只提供按已保存 `clientOrderId` 显式查询；无结果时 attempt 保持 unknown。Loading/error/reload 使用 status/alert 语义。确认框限制焦点，Escape/Keep reviewing 安全关闭，关闭后恢复焦点。在 390、768、1280 px 检查身份与状态标签。不暴露 Binance Live 或 Agent 写入控件。

### 13.15 Binance Spot Testnet 私有更新与恢复（S19 #70）

在所选且已连接的 `BINANCE_TESTNET` 账户下，Order Drafts 展示已保存订单簿、文字形式的私有流/reconciliation 健康状态及最近事件时间。`executionReport` 更新通过现有账户 aggregate event 刷新已保存订单查询。有效账户持仓更新只替换明确变更的资产；stale/degraded/reconciling 状态保留最后可信记录。仅提供 delta 的余额事件或未知事件会让 reconciliation 保持可见的 required，直至固定 REST 路由读取成功。未知订单状态继续显示且非终态。此界面不增加 user-data stream 控件、Live 或 Local Paper 路径或任何权限；保留显式 pending/history/detail 读取控件，并验证键盘操作以及 390/768/1280 px 布局。

### 13.15.1 Binance Spot Testnet 准确订单撤销（S19 #71）

仅当 `BINANCE_TESTNET` 账户已连接，且准确 `BTCUSDT` / `ETHUSDT` 订单的已保存观察为 `CURRENT`、不超过 60 秒、状态是 `NEW` 或 `PARTIALLY_FILLED`、剩余数量已知、有效且严格大于零并且未在撤销时，显示“复核撤销”。激活后先执行现有的准确订单 Detail 刷新；仅当返回订单簿为 `CURRENT`，且准确订单对当前 connection 和远端账户仍可撤销时，才打开复核框。显示账户/订单身份、状态、已成交和剩余数量以及观察时间；剩余数量缺失、无效或为零时保持复核不可用。用户确认后，后端会独立校验捕获的账户/订单簿版本、远端身份和新鲜度，并在写入前重新读取准确账户和订单。订单变化时必须重新复核；已终结订单只记录状态，不发送 DELETE。

仅通过主 Trade UI 发送有类型的 `binance.testnet.orders.cancel` command。`SUBMITTING` 与 `PENDING` 分开呈现；请求已被接受不等于撤销完成，结果不明时保持 pending 且不得再次请求。后续准确订单读取或私有流 event 决定终态并保留竞态成交。不提供 Live、撤销全部、Agent 或 Order Gateway 控件。

### 13.16 Bitget Spot Live 账户订单与成交（#75）

选中 `bitget` / `LIVE` connection 后，Accounts 详情页显示 provider 余额，并在用户显式刷新后显示已保存的当前订单及近期历史订单/成交观测。该区域明确标注 `READ ONLY · DISARMED`。订单表展示 provider order ID；只有关联到持久化 TradeX identity 时 origin 才显示 `TRADEX`，否则显示 `external`；同时显示标的/类型、方向、准确数量或名义金额、累计基础币/计价币成交数量、仅在已知时显示剩余数量、原始/归一 provider 状态与 provider 时间。近期 fills 以 provider trade/order ID 单独识别。缺失字段显示为不可用。

展示 TradeX 观测时间及 `CURRENT`、`STALE`（超过五分钟）或 `DEGRADED` 新鲜度。读取失败时保留最后保存的订单簿与账户余额；账户健康状态显示脱敏的失败/限流状态与重试状态。不进行自动 provider 轮询。宽订单/成交/余额表可用键盘滚动，并在 390、768、1280 px 检查账户详情。没有 Bitget Demo 回退或 Live 写操作。

TradeX 不为 Bitget Classic Spot v2 connection 维护私有流。账户详情显示 `Private stream unavailable · REST reconciliation`；`privateStream` 保持 `NOT_CONFIGURED`，且仅通过用户显式连接或 REST 刷新读取更新观测。

### 13.17 Live 未知 PLACE 对账（S25.1 #97、S25.2 #98、S25.3 #99、S25.4 #100）

Order Drafts 针对准确的 Trading 212、Binance 或 Bitget Live `UNKNOWN_RECONCILING` PLACE attempt 加载已保存的对账证据；只有后端时间可信且五分钟窗口未关闭时，才通过 `trade.resolution_evidence.refresh` 刷新。有界 provider 查询至少间隔 11 秒，每次请求只前进一个历史 cursor 页。导航或重新打开 workspace 后恢复 ledger；时钟不可信会暂停 provider 刷新，但已保存观测仍须可见。超时后只显示后端授权的操作。新鲜且准确的 Binance/Bitget client-order-ID 匹配可提供 Confirm submitted，并预览其证据和预留影响；提交后关联观察到的券商 ID/status，容量仍保持 active，且不代表成交。Trading 212 相似订单及不确定/过期证据只能 Keep；Keep、关闭或导航均保留未知 attempt 和 active reservation。账户保持 DISARMED，直至用户显式执行 Arm。

对于 Binance Spot Live，使用相同的 evidence IPC 和面板，展示 Binance provider、已保存账户/attempt、查询范围、provider order/client ID、状态及可信窗口。只查询普通 Live 账户身份及 attempt 已保存的 `providerClientOrderId`（`tx-{去掉连字符的 execution-attempt UUID}`）指定的准确订单（`origClientOrderId`）。只有 client ID、symbol、side、type、base 或 quote quantity 和窗口时间均准确匹配后，provider row 才是候选；LIMIT 订单还必须匹配 limit price 和 time-in-force。Binance `-2013` 及格式错误、不完整、不匹配、未认证、限流或失败的读取都保持 `INCONCLUSIVE`；不能处置 attempt 或释放 reservation。

对于 Bitget Classic Spot Live，复用相同的 evidence IPC 和面板，显示已保存 Live 账户及远端 `userId`、attempt、查询范围、返回的 Bitget order ID/clientOid/status 和可信窗口。只通过普通 Classic Spot `orderInfo` GET 查询已保存的准确 `clientOid`（`tx-{去掉连字符的 execution_attempt_id}`）。仅当账户、clientOid、symbol、side、order type、size 和时间完全匹配时才列为候选；LIMIT 订单还须匹配 price/force 且 `tpslType=normal`。空、无关、不完整、格式错误、延迟、未认证、限流或失败读取均保持不确定，不能证明不存在。明确区分 provider 订单观测与成交证据；沿用共享超时和冻结 reservation。S25.4 下，新鲜且准确的候选可提供 Confirm submitted；不确定证据仍只能 Keep。不得开放 Live 写入或 Demo/Testnet 回退。

可信窗口过期后，通过 `trade.manual_resolution` 和当前 attempt/证据版本、已保存 evidence ID 提交后端授权的操作。不确定或不支持的证据只能使用 Keep Reconciling；它保持 attempt 与 reservation 冻结，不会重启 provider 查询。新鲜且准确的 Binance/Bitget 候选可提供 Confirm submitted；它会关联观察到的券商 ID/status、保持容量 active，且不代表成交。刷新或重新打开后显示已保存决策。本 provider 切片不得显示确认未提交、释放或重发操作。

展示已保存的账户/attempt 身份、可信窗口、最近查询、覆盖范围和查询范围、分页/完成状态、候选 provider ID/status、不确定/错误状态及后续动作。候选订单不得标成已关联的 TradeX 订单或提交证明。空结果或不完整结果必须明确表示无法证明不存在。窗口超时后显示账户 DISARMED/STALE，继续展示未知 attempt 和 active reservation；只显示后端授权的操作：不确定证据使用 Keep Reconciling，新鲜且准确的 Binance/Bitget 候选使用 Confirm submitted，并显示已保存审计记录；两者均不会重启 provider 查询。验证键盘操作及 390/768/1280 px 布局。此界面没有确认未提交、provider 写入、重试或释放 reservation 的操作。

### 13.18 Trading 212 Live 撤单历史与成交竞态（S26.2 #104）

Accounts 在账户范围列出已保存的 Trading 212 Live 撤单审批，因此准确订单离开开放订单投影后，已消费 attempt 仍可见。导航或重新打开 workspace 后重新读取审批列表及每个已消费审批的 execution preparation。将已保存 attempt/acknowledgement 与最新准确 provider 观测，以及按准确 provider order ID 关联的 S26.1 settlement 分开展示。

“刷新准确订单”只对捕获的账户、已消费 approval、准确 provider order ID 和当前账户状态版本发送 `trade.live_order.refresh`。这是显式只读操作；账户已连接时可用；连接需复核时，只有账户仍在线、认证有效且凭据可用时才可刷新。此只读恢复不使写入或 Arm 变为可用。显示原始状态、归一 disposition、准确可用的数量/成交/剩余/金额、来源、TradeX 观测时间，以及仅在提供时显示 provider 时间。若缺少 provider fee/trade facts，标为不可用并保留关联 settlement 的完整性、未解决原因和剩余容量。竞态成交应成为最新 provider 事实，同时撤单 attempt 继续保持 `CANCEL_PENDING`；不得将 provider acknowledgement 标成撤单已确认。使用 status/alert 语义、键盘可操作的刷新按钮，并验证 390/768/1280 px 布局；不自动轮询。

### 13.19 Binance Spot Live 撤单历史与成交竞态（S26.3 #105）

Accounts 按账户列出已保存的 Binance Spot Live CANCEL approval，并在导航或重新打开 workspace 后恢复每个已消费 attempt。将已保存 acknowledgement 与最新准确订单观测及其准确关联的 S26.1 PLACE settlement 分开展示。

“刷新准确订单”使用现有 `trade.live_order.refresh` command，传入捕获的账户、已消费 approval、准确 `symbol:orderId` 和当前账户状态版本。这是只读操作；仅在 Binance Spot Live 账户已连接时可用；若状态为 `REVIEW_REQUIRED`，只有账户仍在线、认证有效且凭据可用时才可刷新。显示 Binance 原始状态、归一 disposition、准确订单/已成交/剩余数量与累计 quote value、完整时的 provider trade ID 及 commission 资产/金额、来源、TradeX 观测时间、可选 provider 时间，以及关联 settlement 的完整性、未解决原因和剩余容量。成交/手续费证据缺失或不完整时继续显示不可用，并保守保留容量。

DELETE 期间发生的成交成为最新保存的订单事实，而撤单 attempt 仍保持 `CANCEL_PENDING`；acknowledgement 永不表示撤单完成。PLACE 关联必须匹配同一账户、准确的 Binance 数字订单 ID，以及已保存 proposal 中的规范 symbol/instrument；外部订单或仅相似的订单绝不关联。使用 status/alert 语义、键盘可操作的审阅和刷新控件，并验证 390/768/1280 px 布局。不自动轮询，也不提供撤销全部或 Testnet 回退控件。

### 13.20 Bitget Classic Spot Live 撤单历史与成交竞态（S26.4 #106）

Accounts 按账户列出 Bitget Classic Spot Live CANCEL approval，包括订单离开开放订单投影后的已消费 attempt，并在导航或重新打开 workspace 后重新读取。将已保存 acknowledgement 与最新准确订单观测及其准确关联的 S26.1 PLACE settlement 分开展示。

用户先刷新并审阅普通 `normal` Spot 订单，批准不可变 CANCEL intent，然后通过隔离的 Order Gateway 单独准备并发送。刷新使用捕获的账户、已消费 approval、准确 `normal:{orderId}` 和当前账户状态版本。这是只读操作；账户已连接时可用；状态为 `REVIEW_REQUIRED` 时，只有认证和已保存凭据仍有效才可读取。显示原始状态、归一 disposition、准确 base/已成交/剩余 quantity 与累计 quote value、完整时的 trade ID 和手续费资产/金额、来源、provider 时间与 TradeX 观测时间。Bitget 有符号 `totalFee` 以非负手续费成本显示。证据缺失或不完整时继续标为不可用，并保守保留容量。

收到 acknowledgement 后以及成交竞态发生后，撤单 attempt 都保持 `CANCEL_PENDING`；只有准确的 provider 终态证据才能更新订单和关联 settlement。外部或相似订单绝不关联。提供可访问的状态/错误提示、键盘可操作的审阅与刷新控件，并验证 390/768/1280 px 布局。不自动轮询，也不显示 Demo/Testnet、批量撤单或撤单替换控件。

已保存 attempt 若为 `INVALIDATED` 且 `STOPPED_BEFORE_DISPATCH`，应保留在历史中，并允许用户再次刷新和审阅同一准确订单。任何可能已派发到 provider 的 attempt 都必须继续对账，不能重试。

### 13.21 Live 启动恢复与近期订单展示（S27.1 #108）

workspace 打开后，恢复每个受支持 Live 账户的后端恢复状态和可见 health reason。当刷新状态 stale 或 blocked 时，保留并显示最近一次可信账户/开放订单观测。Trading 212 和 Binance Live 在单独的 Recent orders 区域展示有界 `recentOrders` 投影；不得将这些行混入开放订单表，也不得从近期历史行提供撤单入口。Bitget 继续显示既有有界订单/成交历史。

启动流程在不依赖当前选中账户的情况下刷新所有已连接且受支持的 Live 账户。它只会自动为未解决 PLACE attempt 请求准确 S25 evidence；未解决 CANCEL 保持可见，并要求用户显式触发已有准确订单刷新。只有后端报告 reconciliation 与 eligibility 均为 current 时才允许 Arm，并在恢复后保留单独的显式 Arm 确认。展示提供方读取、时间失败和后端 reason；不得根据空近期历史页面推断恢复完成，也不得重试提供方修改操作。

### 13.22 Trading 212 Live 市价审批额度（S28 #114）

不可变的 Market 审批审阅将正数 `maximumSpend` 显示为最高授权金额，并在后端容量预览中保留它。在金额旁说明：它约束 TradeX 的审批与容量预留；Trading 212 不会将其作为市价成交价格或金额上限执行。将预期支出/收入分开展示，显示完整的账户、proposal 和行情溯源，并保留独立、显式的审批及 Prepare 操作。提供方 acknowledgement 仍仅表示接受，不能代表成交。

同时展示 bid/ask 的 BASE 显示深度；滑点标明相对于报价中点、仅覆盖显示深度的近似估算，不保证成交。无受信深度或数量超过深度时保留 unavailable，审批继续禁用。

### 13.23 XNAS 日历来源与独立能力状态（S28 #118）

使用 Backend §41.38 typed `data.calendar.*` 契约。Settings 明确选择合格的已有 Alpaca Paper 账户，保存 metadata，分别刷新/断开；不删除借用 key。分别显示 receipt/version/requested coverage 和 MARKET_CALENDAR/CORPORATE_ACTIONS/HALTS/HISTORICAL_ADJUSTMENT；日历成功不标记综合 OD-005 或 Live Ready。本地只读投影轮询可更新过期状态，但不能请求 provider 或延长接收时间。来源变化刷新 Settings/Markets 投影。市场详情保留 source/time/session 边界、缺失 provider timestamp 和其他独立门禁；保持键盘/错误处理及390/768/1280px 布局。此目标不证明原型行为或真实 provider 验收。

### 13.24 已知事件与精确 broker 元数据（S28 #119）

使用 Backend §41.39 生成的 `data.actions.*` / `data.instrument.*` 命令及可选 `MarketDetail.financialEvidence`。Settings 分别提供账户选择/CAS Save/Refresh/Disconnect；不允许 renderer 秘密或调用方权限断言。未保存选择时禁用 Refresh；Disconnect 保留借用账户/key；显示脱敏访问/额度/错误/reload 状态。本地投影轮询只更新过期，不请求 provider、不延长 receipt。重开 workspace 保留选择并撤销进程证据，重新读取前显示 unverified。

显示原始 receipt、provider quality、缺失 provider timestamp、精确账户/source/material binding 和八项独立能力。公司事件保留16类 typed category、date-only process/可选日期、security 身份/role、精确小数、可选属性及 partial 状态。每页25条访问完整有界查询，不使用16条摄取截断；空且穷尽的查询不能证明没有待处理事件或历史已调整。Broker 行保留精确 ticker/ISIN/currency/schedule join 和有界 schedule 翻页；十分钟元数据、unverified 规范身份、缺失停牌/可交易性及执行 venue 不确定性保持可见。Markets 和审批 review 使用原始 backend 证据；不可用的保留内容不能变成 readiness。保留 Local Paper/CANCEL 和所有既有门禁、键盘操作、错误及390/768/1280px 布局。此为目标契约，不是原型或 provider-hosted PASS。

冻结的审批及 pre-arm 面板将金融状态/原因标为捕获值，显示 review 捕获时间，并说明不可变 review 不报告当前资格。不得用轮询覆盖或修改捕获内容。重新打开获取新的后端评估；受保护操作独立重新核验当前证据。

### 13.25 所需币种上下文与只读汇率（S28 #120）

消费 Backend §41.40 生成的 data.fx.* commands。Settings → Data & Storage 提供“Alpaca currency rates”，明确保存合格已有 Paper 账户；Save、Refresh、Disconnect、reload 分离。未保存选择不能刷新，Save 不读 provider，Disconnect 保留借用账户/key，reopen 仅恢复 unverified 元数据。Renderer 不提供 pair、endpoint、rates、quality 或金融权威。显示脱敏 entitlement、quota、缺失/不支持币种和无需汇率原因。

Portfolio 显示后端导出的金额币种需求，包括已观察余额单位的 BALANCE_WORKSPACE 具名路由；未知账户主币种仍独立保持未知；Trade 加入所选不可变 Proposal 的 policy/funding 路由及明确的“Refresh rates for this proposal”读取。区分所选 intent 上下文与已观察读取上下文。显示原始 bid/ask/provider-mid、方向、各 provider timestamp、首次 receipt/material/source binding、UNQUALIFIED_FX_RATE 和独立 transaction qualification、broker costs、monetary completeness 阻塞。不得把上下文变成组合金额权威、funding bounds 或 consent。仅轮询本地投影更新过期状态，不读 provider、不续 receipt。过期证据保留但不可用。预 Arm review 与不可变审批窗口均仅显示捕获的 currencyEvidence、capture time 和捕获状态，不能把轮询写进这些捕获 review。预 Arm 证据保持只读，不能批准 Proposal。保留所有既有门禁、Local Paper 与保护性 CANCEL。以真实 Rust 验证键盘/status/errors/reload 及390/768/1280px。此目标契约不是 runtime 或 prototype PASS。

### 13.26 Binance Live 交易状态展示（S29.1 #122）

Accounts 显示选中 Binance Live 连接在 Backend §41.41 定义的可选观察：采样系统状态、API 交易锁、保守的首个响应接收时间、提供方锁更新时间及可空预计恢复时间。历史状态缺失时显示不可用，并引导用户使用现有 Refresh。正常/未锁定只表示最近一次读取，明确提示 30 秒过期及不完整读取保留旧观察。不得断言标的流动性、数据权利或 Live-ready。Arm/PLACE 使用后端健康门禁；浏览器时间或这些标签不能赋权。保留独立受限撤单/对账、键盘导航及窄屏换行。S29 实际金融验收另行完成。

### 13.27 精确 Binance Spot 规则来源展示（S29.2 #123）

Settings Data & Storage 选择已有普通 Live 账户和 BTC/USDT 或 ETH/USDT，提供显式 Save、Refresh、Disconnect 和恢复控件。未保存选择禁用 Refresh；展示观察仍标明属于已保存选择。Disconnect 保留账户/密钥。分开展示后端采集/准入/执行/权利能力、原始/保留接收时间、类型化分层约束、禁用项、不支持字段和逐项剩余义务。Settings 元数据轮询不请求提供方。Markets 使用后端精确标的 `spotRuleEvidence`，轮询投影反映过期；主 Trade Proposal 视图要求精确账户及标的匹配。捕获的审批/武装前证据保留捕获时间边界。重开只有元数据。保留股票来源面板、键盘控件及 1280/768/390 布局。Renderer 时间、成功读取或 TRADING 永不授予 Arm/审批/PLACE 或报价/深度/许可权限。参见 Backend §41.42；原型 fixture 代码未改。


逐项展示后端 `orderFormFlags` 为 true/false/未提供，并展示高级订单形式不支持边界；不得从缺失推断支持。空权限集合不表示无需权限即可访问。


### 13.28 普通公开 Spot 行情来源（S29.3 #124，IN_PROGRESS）

通过生成 IPC 消费 Backend §41.43。Settings Data & Storage 显式 Save/Disconnect，不输入账户/密钥，不请求提供方或创建租约。分开展示已保存选择、技术采集及 UNVERIFIED 的金融用途/保留/再分发/商业/地区权利；来源目录保留官方/条款链接和审阅日期。本来源没有通用 PUBLIC_METADATA probe：采集属于可见 Hot 生产者，不由按钮暗中制造连通/许可。CAS 失败重载权威配置。重开/重新挂载只恢复选择，保留 Alpaca 设置与账户密钥。

UI §14.27 完整目标仍 IN_PROGRESS。Markets 与所选 Trade Proposal 必须分别使用同一有界 acquire/get/release 页面生命周期并绑定精确后端代次；Trade 不得借用已释放 Markets 租约。展示 BINANCE 来源、精确最佳价格/BASE 数量、已知区间/显示层级、首次接收/提供方事件时间、连续性/失败/恢复和不可变材料身份。流连接不表示认证、报价权威或权利。捕获评审保留捕获标签；本地投影轮询不读提供方、不续年龄、不授予金融权限。导航/隐藏/工作区/来源/标的变更及迟到 acquire 必须终止/释放所属租约。保持键盘和1280/768/390 元数据/阻断可读，保留股票 Hot 行为。Settings 与 Markets 聚焦公开 React→Rust/外部 HTTP/WS 证明覆盖 BTC/ETH 所属 Hot 页面、精确深度/来源、导航与标的替换，并通过原股票 Hot 回归；所选不可变 Proposal 现通过同一控制器持有独立租约，以工作区/Proposal 身份/hash 为 key；不支持的执行上下文不创建租约。预 Arm/审批深度面板只消费捕获的后端评审数据和捕获时间，没有自己的轮询或武装权限。公开 UI 证明覆盖 DISARMED 的只读预 Arm 捕获在来源替换产生新当前盘口时保持不变。实际已武装审批/普通原生/提供方/金融、其余生命周期/故障及完整来源验收仍待完成。原型代码未改。


### 13.29 Proposal 专属静态 Spot 规则（S29.4 #125）

通过生成的 IPC 消费 Backend §41.44。所选已保存 Binance Live Proposal 持有当前只读规则投影及显式所需参考 Refresh；本地轮询只反映过期，不读取提供方、不续首次接收。展示精确 Proposal/hash/账户/canonical 单位、source/material/接收时间、每项保留 scope/origin 的结果和明确原因、所需用途、原始区间/提供方时间/首次接收/digest、失败恢复及未解决义务。投影错误时不得继续显示当前成功。捕获风险/评审/历史展示已保存评估，隐去价格，没有刷新/轮询权限。保持键盘及1280/768/390可读，保留 BASE/QUOTE 和 USDT 单位，区分静态分项 PASS、完整规则与独立金融门禁。参见 UI §14.28。原型代码未改；文档本身不等于运行时验收。

### 13.30 Proposal 专属只读容量输入（S29.5 #126，IN_PROGRESS）

CurrentSpotCapacity 为所选不可变 Proposal 使用类型化本地 `trade.spot_capacity.get`，通过显式 CAS `trade.spot_capacity.refresh` 读取实际所需经认证输入。本地轮询只反映过期/绑定变化，不采集服务商。显示精确身份/BASE/QUOTE 单位、用途、独立账户/标的 total/algo/iceberg/list 计数、不可用与零的区别、覆盖/部分成交/外部资产/列表腿缺口、原始余额/BUY 组成、非原子质量、读取窗口、原始 update/transaction 时间与收据及摘要。说明脱敏失败/恢复及独立执行资格。规则材料/用途不可用时禁用私有刷新；busy 或投影错误不能保留当前成功。捕获 RiskDecision/历史/Arm 前只展示已保存 `spotCapacity`，无刷新或轮询，不能续证据或同意。核验键盘及1280/768/390，保留独立金融门禁。Backend §41.45 与 UI §14.29 定义契约；原型代码未改，实施验收仍待完成。

### 13.31 Proposal 专属账户时段输入（S29.6 #127，SOURCE_VERIFIED / FULL_ACCEPTANCE_PENDING）

CurrentSpotOrderIntervals 使用生成的本地 `trade.spot_order_intervals.get` 和显式 CAS `trade.spot_order_intervals.refresh`；本地1s投影轮询不调用 provider。展示不可变身份/BASE/QUOTE、跨 key/IP/API 账户范围、实际定义与原始精确字符串 count/limit/intervalNum、完整/未解决 tuple 覆盖、非原子读取窗口/digest、原始 provider clock 与缺失 counter snapshot time、导出的不确定关联/撤下、失败和请求冷却。开放订单 filter 库存、ORDERS 时段用量、REQUEST_WEIGHT/RAW_REQUESTS 与 reservation 分开。Busy/error/已撤下输入不能仍展示当前成功。捕获 RiskDecision/历史/pre-arm 展示保存的 `spotOrderIntervals`，不刷新/轮询或续收据/同意。保留 keyboard 与1280/768/390；完整 dynamic/financial/Arm/同意/Prepare/Gateway 门禁独立。Backend §41.46 与 UI §14.30 定义 wire/交互；原型代码未改，来源验收不意味着完整产品验收。

### 13.32 Proposal 专属价格区间执行预览（S29.7 #128，IN_PROGRESS）

既有 Current Proposal Spot rules 与 Captured Proposal Spot rules 展示所选不变普通 Binance Live Proposal 的生成 `priceRangePreview`。以白话展示精确状态、选定 side 与 QUOTE-per-BASE 单位、BID(BUY) 与 ASK(SELL) 两条配置条目（缺省乘数写作「未设置」而非零）、各方向实施/不实施的事实、真正当前参考价限定的选定 side 快照边界、执行参考价的 provider 原始时间/首收据/digest、精确解释码，以及固定说明：交易所在 taker 阶段重算参考价、越界执行会使订单过期，本面板不审批、不下单、不承诺。

「不实施」、显式 null、参考价缺失/失败/过期、不支持配置与市价形态保持视觉区分；失败读取绝不渲染为不实施，快照预览绝不渲染为资格。过期参考价保留其来源 digest 与收据时间，但价格以「非当前」为由不予显示，且绝不渲染为提供方显式 null。既有可键盘操作的「Refresh required rule references」是唯一 provider 读取，当意图无需参考价时禁用，且不新增其他控件。捕获 risk/历史/pre-arm 保留已保存预览，无刷新控制器、无轮询、不续收据/同意，并把被剥离的快照边界显示为「仅保留 digest」，同时仍指认该边界 digest。仍需 keyboard 与1280/768/390 可读；权利、报价、权限、健康、费用/FX、资金、Arm、同意、Prepare、Gateway 与精确 CANCEL/reconciliation 保持独立。Backend §41.47 与 UI §14.31 定义本契约；原型代码未改，来源验收不意味着完整产品验收。

### 13.33 拥有方 Spot Live 准入陈述（S29.8 #129，IN_PROGRESS）

既有的 Live 批准复核、pre-arm 捕获证据与 RiskDecision 历史界面在已交付的容量与时段面板旁渲染生成的 `spotOwning` 陈述。用平实语言给出精确原因码与原因，随后在同一行给出交易所自身声明的剩余下单额度及其绑定声明桶，或在无法依赖任何额度时给出唯一类型化阻塞，使数字与其阻塞无法被分开阅读。渲染所选单位（买单 QUOTE、卖单 BASE）、每个数字的声明出处、声明的开放订单数、声明的标的开放买单数量并明确标注它只是声明敞口、绝不等于可用余额、声明的 BASE free/locked 数字并说明提供方 `free` 已排除订单锁定且绝不二次扣减、任何既有提供方请求冷却并明确它不是本门禁、以及两个观测时间并明确时段计数时间戳缺失。把 `carriedLimitations` 列为其证据如何取得，而非覆盖缺口，并明确保留交易所 taker 阶段重算、可能过期以及不存在任何成交、准入或批准承诺。合格陈述绝不渲染为权威：Arm、consent、派发、费用、所需 FX、即时认证 preflight、私有流就绪与 exact CANCEL/对账保持独立。捕获 risk/history/pre-arm 保持为已保存评估，无刷新控件、无轮询、不续期收据或同意；拥有方陈述不存在「当前」形态，因为它只会在已保存的 RiskDecision 内派生。键盘与 1280/768/390 可读性仍为必需。Backend §41.48 拥有线缆契约；UI §14.32 拥有交互契约。原型代码未改；源码验收不代表完整产品验收。

## 14. Live Execution UI 架构

### 14.1 原则

前端是**决策与用户同意界面**，不是 execution engine。

### 14.2 Order lifecycle 展示

UI 至少支持：

```text
DRAFT
PROPOSED
RISK_REJECTED
NEEDS_APPROVAL
APPROVED
RESERVED
SUBMITTING
ACCEPTED
PARTIALLY_FILLED
FILLED
CANCEL_PENDING
CANCELLED
REJECTED
EXPIRED
UNKNOWN_RECONCILING
```

### 14.3 Editable draft 与 immutable proposal

- `OrderDraft` 可编辑；
- `Generate Proposal` 返回新的不可变 `proposal_id + proposal_hash`；
- 之后的任何 material edit 都生成新的 proposal identity；
- 旧 approval UI 必须明确变为 invalidated，不能静默替换字段。

### 14.4 Account-scoped arming

流程：

```text
select exact live account
→ inspect account health + permission state
→ explicit Arm action
→ backend confirms ARMED for that account
→ UI updates account badge
```

Global Disable All 调用一次后端动作，再渲染后端返回的每个账户状态。

确认框必须显示 provider、账户标签、`LIVE` 环境、完整 TradeX connection ID 和已观察到的 provider account ID。当前资格缺失或被阻断时禁用 Arm，并展示后端 remediation reason；权限范围必须为 `VERIFIED`，即使连接审核时已确认 `UNVERIFIED` 范围。`Disable Live` 只影响所选连接；`Disable All Live Execution` 始终作用于工作区内全部 Live 连接。Keep reviewing/Escape 取消且不写入，初始焦点位于 Keep reviewing，关闭后焦点返回触发控件或账户标题。应用只通过 `account.activity` 报告经过节流的受信指针、键盘和触控输入；后台轮询不延长 inactivity deadline。

### 14.5 Approval modal

必须包含：

- 准确 account/environment；
- proposal ID/hash 或可审计 immutable identity；
- instrument、side、quantity/notional、type、price、TIF；
- estimated notional；
- 完整 market snapshot provenance；
- 必要时的 available / reserved / effective available；
- deterministic risk checks；
- policy version；
- 显式 Reject 与 Approve。

`Enter` 永远不能作为默认 approval 动作。

Approve 只签发短时、单次 approval。唯一公开且明确的 `Prepare and send approved PLACE` 操作调用 `trade.execution.prepare`。界面说明它先提交 `RESERVED`，随后在 Control Plane 内部启动隔离 Order Gateway；只有之后持久化 `SUBMITTING` 才可能开始 provider request。展示后端当前 attempt，以及精确 reservation 金额/单位、provider available 与 committed capacity、TradeX reserved capacity、effective available capacity、证据来源/新鲜度/account state version。若停止在 `SUBMITTING` 前获胜，应显示 `INVALIDATED`/`STOPPED_BEFORE_DISPATCH`、后端原因和精确 `RELEASED` 金额，并说明未发送 provider mutation。`SUBMITTING` 持久化后，显示 request 可能已发送，并保持容量预留直至权威结果解决。`ACCEPTED` 仅表示 provider 接受，不得显示为成交；分别显示 `REJECTED` 与 `UNKNOWN_RECONCILING`，并禁用已消费 approval/attempt 的重发。若 provider 已返回扣除其订单承诺后的 available，则仅解释 provider committed 金额，不得再次扣减。打开 proposal 历史时按 approval ID 读取 preparation 与脱敏容量拒绝历史，使响应丢失或重启后能恢复已保存状态。`RISK_REJECTED · RESERVED_CAPACITY` 显示后端原因、容量来源/上限、本次需求、已有预留、请求前有效容量和 remediation。陈旧或不可用证据必须明确标注，不得把旧金额当作当前金额。不能把 `ALLOWED` RiskDecision 当作拒绝原因重新读取或展示。容量拒绝保留 approval 与 proposal；后续显式重试使用新的幂等键。每个持久化状态都使用无障碍状态文本，并验证键盘焦点及 390/768/1280 px 布局。

### 14.6 Reservation conflict

如果 proposal 因 effective capacity 降低而失败，直接展示后端原因、reservation context、观测时间和 remediation。浏览器端不得重新计算金融结论。账户或 reservation 更新后重新读取后端投影；若 review 绑定较旧的 account state version，则使其失效。

### 14.7 Ambiguous submission

`UNKNOWN_RECONCILING` 打开 Manual Resolution，只允许后端授权的选项：

- Confirmed not submitted（要求 evidence）；
- Confirmed submitted（绑定 broker identity）；
- Keep reconciling。

不得存在通用 “release reservation and continue”。

---

### 14.8 保留操作类型的审批与撤单

待处理意图按 PLACE_ORDER 或 CANCEL 区分，并保存其不可变后端身份。打开 Arm 不替换意图、不修改账户、不填入默认订单。Arming 后请求后端刷新可执行性，并返回对应审批。撤单必须展示券商订单 ID 与剩余数量；提供方快照变化使旧同意失效。

### 14.9 按状态呈现与控制可执行性

显式处理各订单状态；无法识别或 UNKNOWN_RECONCILING 状态不得回退到成功成交。只显示已持久化的观察事件，并区分账户健康、订单状态、预留状态和审批过期。Manual Resolution 先通过 trade.resolution_evidence 读取后端证据及允许的决策，再通过 trade.manual_resolution 提交证据引用/版本；本地文字或复选框不能验证证据。

按钮反映精确操作的后端可执行性。账户、模式、行情、时钟、权限、策略、proposal 或预留变化时重新评估。Disable All 等待结果说明哪些账户已 disarm、哪些尝试已停止/可能提交；不能乐观删除预留或暗示已券商撤单。UI Spec §14 定义所需交互与无障碍场景。

---

### 14.10 RiskDecision 复核与历史（S21 #81）

Order Drafts 按 workspace/proposal 查询不可变 RiskDecision history，并提供显式 evaluate/reevaluate 操作。Renderer 仅发送这两个 identity；Control Plane 负责策略、账户、证据、检查与持久化。界面分别显示 `ALLOWED`、`REJECTED` 和 `UNAVAILABLE`，并展示 proposal hash、准确账户/环境、策略版本、输入摘要与检查原因。Submit 失败后刷新查询，以展示已持久化的阻断决策。`ALLOWED` 结果及其 UI 状态均不授予审批、Arm、预留或发送权限。`RISK_EVIDENCE_UNAVAILABLE` 是独立于 `RISK_REJECTED` 的规范错误。普通 Bitget `LIVE` 保持只读。

### 14.11 Workspace 风险策略变更摘要（S21 #82）

Settings → Risk & Limits 在策略保存后以及 risk projection 恢复时展示 `RiskPolicyState.lastChange`。`role="status"` 的 polite live region 播报作用范围、新旧版本、是否存在任一放宽，以及受影响的已持久化账户数和待处理 proposal 数。键盘可操作的详情展示每个账户身份/环境，以及每个失效 proposal 和其策略版本原因。Renderer 不得从当前选中账户推导作用范围；失效和撤防由后端决定。

## 15. Provider 与 Model 配置 UI

### 15.1 Schema-driven forms

Provider 配置由后端 schema metadata 驱动。

```ts
interface ProviderFieldSchema {
  id: string;
  label: string;
  inputType: "text" | "password" | "select" | "boolean";
  required: boolean;
  secret: boolean;
  environment?: string[];
  helpText?: string;
}
```

前端不得假定所有 provider 都使用 `API Key + Secret`。

### 15.2 Secret-entry 规则

Secret field 对 UI 来说是 write-only。安全提交完成后，UI 只保存 “configured”、允许暴露的 keychain reference identity、permission/capability result 和 health 等 metadata。

### 15.3 LLM provider surface

展示：

- CLIProxyAPI sidecar state；
- pinned version；
- `/v1/models` health；
- ChatGPT OAuth login/re-login；
- DeepSeek key configured/invalid；
- discovered models；
- default provider/model；
- 可选 `Allow automatic fallback to DeepSeek`，默认 OFF。

LLM Provider 永远不能表现为交易 capability。

---

## 16. Error 与 Recovery 架构

使用一个由 canonical error 驱动的通用 `ErrorRecoveryPanel`。

```ts
interface TradeXError {
  category: ErrorCategory;
  code?: string;
  title: string;
  detail?: string;
  blocking: boolean;
  remediation: RemediationAction[];
  relatedEntity?: { type: string; id: string };
}
```

Canonical categories：

```text
AUTH_ERROR
PERMISSION_ERROR
RATE_LIMITED
NETWORK_ERROR
UNSUPPORTED_CAPABILITY
MARKET_CLOSED
INSTRUMENT_HALTED
INVALID_ORDER
INSUFFICIENT_FUNDS
RISK_REJECTED
RISK_EVIDENCE_UNAVAILABLE
SUBMISSION_REJECTED
SUBMISSION_AMBIGUOUS
STREAM_DISCONNECTED
STATE_STALE
RECONCILIATION_REQUIRED
MODEL_UNAVAILABLE
QUOTA_EXCEEDED
OAUTH_EXPIRED
INTERNAL_ERROR
```

前端不得自行创造新的 machine-state 名称。

---

## 17. Accessibility 架构

强制工程规则：

- 使用语义化 button/input/landmark；
- 明确 `:focus-visible`；
- 合理 tab order；
- icon-only control 有 accessible label；
- modal 有 focus trap 与关闭后的 focus return；
- 重要订单状态变化使用 `aria-live`/status；
- Paper/Demo/Testnet/Live 使用明确文字；
- generic `Enter` 不绑定 approval；
- 非必要动画支持 reduced-motion；
- 安全信息不能 hover-only。

Live approval modal 初始 focus 应放在 neutral/reject-safe 控件，而不是 Approve。

---

## 18. 性能架构

### 18.1 Rendering strategy

- 长 Thread timeline 使用 virtualization；
- streaming item 增量追加；
- 对复杂 structured card memoize；
- chart render 与主 timeline tree 隔离；
- cache key 基于 canonical ID；
- 高频 market event 在超过实际可视刷新需求时先 batch，再触发 React commit。

### 18.2 Market stream strategy

前端不消费所有 instrument 的所有 tick，只接收后端选择的 Hot subscription，以及其他 context 的 coarse/on-demand update。

### 18.3 Failure containment

图表或 research card 的 rendering error 不能遮蔽 live execution state。关键 execution/status surface 与非关键 visualization 使用独立 error boundary。

---

## 19. Security 架构

前端安全要求：

1. broker secrets 不进入 React state、browser storage、日志、route、analytics 或 crash payload；
2. 不直接请求外部 LLM；
3. 不直接请求 broker/exchange 的高权限操作；
4. research/web content 视为 untrusted，不能触发 privileged command；
5. HTML/Markdown 渲染必须 sanitize；
6. 对敏感 account data 的 clipboard/export 在适用时要求明确用户动作；
7. Trade approval state 只从后端 authority object 渲染；
8. UI 不能伪造 approval ID、reservation ID 或 execution eligibility。

---

## 20. 前端项目结构

推荐：

```text
src/
  app/
    App.tsx
    routes.tsx
    providers/
  components/
    shell/
    timeline/
    market/
    account/
    trading/
    strategy/
    artifact/
    settings/
    recovery/
  features/
    threads/
    composer/
    markets/
    watchlists/
    accounts/
    trading/
    strategies/
    backtests/
    artifacts/
    providers/
  domain/
    types/
    selectors/
    schemas/
  ipc/
    client.ts
    commands.ts
    events.ts
    generated/
  state/
    uiStore.ts
  query/
    keys.ts
    hooks/
  accessibility/
  styles/
  test/
```

Generated IPC/domain schema binding 与手写 UI logic 分离维护。

---

## 21. 测试策略

### 21.1 Unit tests

覆盖：

- Agent Mode × Execution Context compatibility；
- state selectors；
- immutable proposal rendering；
- policy invalidation display；
- provider disclosure；
- canonical error mapping；
- safe keyboard behavior。

### 21.2 Component tests

通过 fixture 测试：

- LiveArmModal；
- LiveApprovalModal；
- ReservationPanel；
- CancellationApprovalModal；
- ManualResolution panel；
- ProviderCredentialSchemaForm；
- ErrorRecoveryPanel。

### 21.3 Integration tests

使用 fake IPC backend 验证：

- turn streaming；
- restart/resume 后全部 live accounts 为 DISARMED；
- policy save 使 approval 失效；
- reservation conflict；
- ambiguous submission；
- model OAuth/quota error 和 explicit switch；
- Demo/Testnet 永远不会被视觉误认为 Live。

### 21.4 End-to-end safety tests

必须断言：

- 选择 Trade/Live 不会自动 Arm；
- Arm Trading 212 不会 Arm Binance/Bitget；
- generic Enter 无法批准 live order/cancel；
- materially changed proposal 不能复用旧 approval；
- stale/clock-uncertain quote 不能显示可执行 approval；
- `UNKNOWN_RECONCILING` 不存在不安全 release button；
- restart 后 UI 不恢复 ARMED。

### 21.5 Accessibility tests

尽可能自动化语义检查，并手动验证 focus order、screen-reader status、narrow-window live flow 与 reduced motion。

---

## 22. 前端交付阶段

### Phase FE-0 — Shell 与 Contracts

- Tauri/React shell；
- typed IPC client；
- 一级导航；
- design tokens；
- Thread/Turn/Item renderer foundation；
- Agent Mode/Execution Context 类型；
- account-scoped arming display model。

### Phase FE-1 — Agent Research Workspace

- composer/pickers；
- thread history/resume；
- market/instrument/context panels；
- provider/model provenance；
- watchlists/accounts/artifacts。

### Phase FE-2 — Backtest 与 Simulated Trading

- strategy/backtest surfaces；
- Local Paper；
- Alpaca Paper；
- T212 Demo / Binance Testnet / Bitget Demo lifecycle variants。

### Phase FE-3 — Live Execution Safety

- live arming；
- approval/provenance；
- reservation；
- cancellation；
- risk-policy invalidation；
- ambiguous state / Manual Resolution；
- startup/reconnect recovery。

### Phase FE-4 — Hardening

- 完整 canonical errors；
- workspace import/restore；
- accessibility；
- narrow-layout QA；
- performance profiling；
- release telemetry controls。

---

## 23. 前端 Definition of Done

前端 v1.0 达到 architecture-complete 的条件：

1. 每个 RevC UI state 都对应 typed backend contract，或被明确标记为 local-only UI state；
2. Thread/Turn/Item replay 不依赖当前 picker 值；
3. Agent Mode 与 Execution Context 在 UI/state/API type 中始终独立；
4. account-scoped arming 不存在 global boolean shortcut；
5. 所有 live approval 展示 immutable order identity + 完整 provenance + risk/reservation 信息；
6. frontend 无法访问 secret 或 privileged execution API；
7. canonical error/recovery 不伪造 authority；
8. keyboard/narrow-layout 保持全部 live safety invariants；
9. generated schema compatibility test 与 pinned backend version 全部通过。

---

## 24. 与 RevC 的前端追溯

本 ARD 主要落实以下 requirement groups：

- **FR:** FR-002–006、FR-014–018、FR-030–035、FR-040–045、FR-048–080 中所有 UI-facing 部分；
- **NFR:** NFR-001、NFR-002、NFR-009、NFR-015–019；
- **SEC:** SEC-001–009 作为前端 boundary constraints；
- **DATA:** DATA-001、DATA-003–008；
- **OPS:** OPS-003、OPS-007–009 的 recovery/visibility 部分；
- **UX:** UX-001–010。

Risk、Reservation、Broker state、Approval validity、Reconciliation、Credential handling 与 Execution 的最终权威均在后端。

## 已配置生产报价（S28 #116）

通过生成的 IPC 消费 Backend §41.37 数据源配置和权威行情 projection；renderer/Agent 不得提交 entitlement、行情 observation、任意 URL 或凭据。遵循 UI §14.21 的 Settings 与 Market Explorer 交互目标，在导航/workspace 变化时释放 active Hot lease，准确显示 feed/coverage/depth/time/failure 状态。缓存渲染不得更新 provider/receipt freshness 或恢复 Live authority。保持不可变证据/同意绑定及其他全部金融门禁。此实现目标仍待运行时验证。

Settings 渲染生成的 credentialKind 与 cleanupPending 字段。专用安全输入仅以元数据调用，renderer 不含密码字段或 Keychain 引用。失败/取消保存后重新读取权威选择及清理状态；独占 key 删除待处理时提供独立显式重试。复用券商 key 仍由账户生命周期持有。

纯报价 adapter 不提供最后成交价。渲染精确 bid/ask、已验证 BASE 深度及生成的 `alpaca` 溯源，不用推断价格替代最后成交价。缓存导航保持 provider/receipt 时间戳与 snapshot 身份。初始 HTTP observation 不证明 Hot stream 已激活；明确显示此区别及报价/时间失败。

#### 已配置股票 Hot 视图生命周期

已配置 OD-001 的股票详情持有一个临时 `market.hot.acquire` 租约，绑定 workspace、canonical instrument 与来源连接的 `stateVersion`。视图渲染 `market.hot.get` 投影，renderer 不直接拉取提供方报价。本地投影可每 500 ms 刷新，最多一个请求进行中；它不能替代后端 WebSocket producer。认证/订阅标志及 stream status 与 `MarketDetail.status`、报价 freshness 分别展示。导航、切换标的、打开 screener、workspace 变化、卸载及 document 隐藏均释放租约；cleanup 后才返回的 acquire 也必须释放其租约。Failed/stale 投影提供显式所选 feed 重试。旧 effect 的迟到响应不能覆盖新视图。未配置股票与其他资产类别沿用原有受保护详情路径。仅 HTTP 刷新不能展示为 Hot 订阅更新。

自动重连时，视图仍用其 `leaseId` / lease `generation` 读取和释放；单独展示轮换的 `connectionGeneration` 与 `reconnectAttempt`（0–3）。`RECONNECTING` 清除认证/订阅标志，旧报价非 current；视图不得自行 acquire 第二条连接或将 ACK 当作恢复的报价。

### 13.34 捕获的拥有方费用与所需执行 FX 解释（S29.9 #131，IN_PROGRESS）

渲染既有保存RiskDecision/approval/pre-arm的spotFeeFx：精确账户/Proposal/hash/instrument/BASE/QUOTE、原始声明rate/比较基准、实际意图notional/明确声明的maximum、fee currency/origin、所需direction、provider pair/quality/time/首次收据/cost、两个evidence version、carried limitations、一项绑定reason及共存fee事实。缺失佣金不称已声明，缺失费用/换汇仍不可用。无refresh/polling或历史续期；USDT≠USD及keyboard1280/768/390无溢出。陈述成功不代表费用/FX资格，当前Spot输入仍阻断。Backend §41.49/UI §14.33为权威；完整父项/原生/金融/物理/原型/main门禁独立。
