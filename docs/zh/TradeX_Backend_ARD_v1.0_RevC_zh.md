# TradeX 后端架构需求与设计（ARD）

**契约澄清日期：** 2026-09-05；原型行为仅为证据，以 QA Report 记录的缺陷和待验证门槛为准。

**版本：** v1.0 Revision C (RevC)  
**状态：** 工程基线  
**范围：** 本地后端 / Control Plane / Agent Runtime 集成 / Domain Services / Execution Boundary  
**主要实现：** 以 Rust Control Plane 为核心，并在合理场景使用本地 Sidecar / Process  
**来源基线：** `TradeX_PRD_v1.0_RevC_zh.md`、`TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md`；原型观察另见 QA Report\
**语言：** 简体中文

> 本 ARD 将 RevC 产品需求落实为可实施的后端架构。安全语义、产品状态、Provider 范围和存储职责以 RevC PRD 为准；下文中的进程边界、服务拆分、持久化模式、Command/Event Contract、并发控制和 Adapter Pattern 属于工程实现层面的架构决策。

---

## 1. 目的

TradeX 后端是桌面 Workspace 背后的可信本地 Control Plane。它集成 Codex App Server 负责 Agent 执行，使用 CLIProxyAPI 负责模型路由，使用本地域服务完成研究/回测，并通过隔离的券商/交易所 Adapter 访问账户和执行能力。

后端最核心的职责是：允许 Agent 进行研究并提出金融操作，但绝不能让 Agent 自身成为金融执行权限主体。

后端必须：

- 监督本地 runtime dependency；
- 持久化 Thread / Turn / Item provenance；
- 向 Agent 暴露 typed domain tools；
- 统一 instrument、account、order、market data 与 provider capabilities；
- 管理 account-scoped live arming；
- 运行 deterministic risk；
- 签发和验证 TradeX financial approval；
- 按账户原子预留执行 capacity；
- 隔离 privileged Order Gateway 与 credentials；
- 在 ambiguity、restart、sleep、disconnect 和 rate limit 下安全 reconciliation/recovery；
- 保持本地 auditability 与 data provenance。

---

## 2. 架构驱动因素

### 2.1 安全驱动

1. Model 不能访问原始 broker secrets。
2. Codex/Agent runtime 不能直接调用 privileged Order Gateway。
3. Generic Codex approval 不能授权金融执行。
4. Live approval 必须绑定 proposal/account/action、短期且 single-use。
5. Live orders、positions、fills 以 broker/exchange state 为权威。
6. 对 ambiguous non-idempotent submission 禁止 blind retry。
7. Live account arming 按账户隔离，并在安全 trigger 时 reset。
8. Reservation 在 concurrent threads 间按账户原子化。
9. stale market data 或不可接受的 clock uncertainty 对 live authority decision fail closed。
10. Dangerous credential permission 必须阻止 live readiness。

### 2.2 Runtime 驱动

- Codex App Server 固定版本并进行 compatibility test。
- CLIProxyAPI 固定版本、由本地 supervise，并且是唯一 LLM egress。
- 即使模型不可用，Control Plane 的 trading/reconciliation 仍继续运行。
- 默认 local persistence；external telemetry 仅 opt-in。

### 2.3 数据驱动

- SQLite 负责 transactional/domain state 与 financial audit。
- DuckDB 负责 MVP analytical/historical data。
- Parquet 仅用于可选的大型 immutable dataset/interchange。
- Filesystem 保存 artifacts、strategies、exports、backups、datasets。
- Secret 排除在所有普通 Workspace storage 之外。

---

## 3. 后端系统上下文

```mermaid
flowchart TD
    UI[TradeX React/Tauri UI]

    subgraph CP[Trusted TradeX Control Plane - Rust]
      API[IPC Command/Event API]
      ORCH[Runtime Orchestrator]
      CAP[Capability Policy]
      RISK[Risk Engine]
      APPR[Approval Authority]
      RES[Reservation Service]
      REC[Reconciliation Coordinator]
      TIME[TimeService]
      PROV[Provider Registry]
    end

    subgraph AGENT[Untrusted Agent Zone]
      COD[Codex App Server]
      MCP[TradeX Research MCP]
      SBX[Strategy Sandbox]
    end

    subgraph MODEL[Model Credential Zone]
      CLIP[CLIProxyAPI :8317]
      CHAT[ChatGPT OAuth]
      DS[DeepSeek API]
    end

    subgraph DOMAIN[Domain Services]
      MKT[Market Data]
      PORT[Portfolio]
      SCR[Screener]
      BT[Backtest]
      PAPER[Local Paper]
      INST[Instrument Rules]
      CAL[Calendar / Corporate Actions]
    end

    subgraph EXEC[Privileged Execution Zone]
      GW[Order Gateway]
      KEY[OS Keychain]
      ADP[Provider Adapters]
    end

    subgraph STORE[Local Storage]
      SQL[(SQLite)]
      DUCK[(DuckDB)]
      PQ[(Parquet optional)]
      FS[Filesystem]
    end

    UI <--> API
    API <--> ORCH
    ORCH <--> COD
    COD --> MCP
    COD --> CLIP
    CLIP --> CHAT
    CLIP --> DS

    MCP --> DOMAIN
    CP --> DOMAIN

    APPR --> RISK
    RISK --> RES
    RES --> GW
    GW --> KEY
    GW --> ADP
    REC --> ADP

    CP --> SQL
    DOMAIN --> SQL
    DOMAIN --> DUCK
    DOMAIN --> PQ
    DOMAIN --> FS
```

---

## 4. 信任区与进程边界

TradeX 定义四个安全相关区域。

### 4.1 Untrusted Agent Zone

包含：

- Codex App Server；
- TradeX research MCP/tool process；
- strategy sandbox；
- workspace research files。

该区域可以读取获准 context 并创建 proposal/signal，但不得获得 broker credentials 或直接 execution capability。

### 4.2 Trusted Control Plane

包含金融权限逻辑：

- capability policy；
- account arming state；
- risk engine；
- approval authority；
- reservations；
- reconciliation coordination；
- provider capability/health state；
- TimeService；
- persistence transactions。

### 4.3 Privileged Execution Zone

包含：

- OS keychain access；
- provider request signing/authentication；
- ExecutionAdapter 实现；
- Order Gateway。

只有经过验证、批准并成功 reservation 的 execution intent 才能进入该区域。

### 4.4 Model Credential Zone

仅包含 CLIProxyAPI 与 model credentials：

- ChatGPT OAuth token 位于 sidecar auth directory；
- DeepSeek API key 由 Rust 在 launch 时渲染到 sidecar config。

Broker credentials 绝不能进入该区域。

---

## 5. 本地进程拓扑

推荐 v1.0：

```text
tradex-desktop (Tauri/Rust main process)
 ├─ React webview
 ├─ embedded/control-plane Rust services
 ├─ child: order-gateway [privileged execution; private IPC only]
 ├─ child: codex-app-server [pinned]
 ├─ child: cliproxyapi [pinned, localhost:8317]
 ├─ child: research-mcp (Rust or Python, restricted contract)
 └─ child: strategy/backtest worker(s) [restricted]
```

### 5.1 Process supervision

Rust main process 负责：

- deterministic startup order；
- version verification；
- health checks；
- capped exponential backoff restart；
- stdout/stderr redaction；
- shutdown coordination；
- crash event persistence。

### 5.2 Startup order

推荐：

```text
open/migrate SQLite
→ load settings + provider metadata
→ initialize TimeService
→ initialize domain services + durable event/authority store
→ start/authenticate Order Gateway (new mutations disabled)
→ restore account metadata
→ reconcile all live/open-order accounts
→ expose live execution readiness only for healthy accounts
→ require explicit per-account arming before new Live mutations
```

领域服务初始化后，以独立且有界退避的分支执行 CLIProxyAPI 启动 → `/v1/models` 探测 → Codex App Server 启动。模型启动失败不得阻塞订单查询、对账、账户 disarm 或执行控制面。只有所选模型路由健康后才能开始 Agent Turn；此前工作区可显示历史与恢复入口。

### 5.3 Order Gateway 进程边界

独立子进程是强制要求（PRD OD-009），桌面主进程内的模块不能代替该边界。“Privileged”表示独占交易/凭据能力，不表示以 root/管理员身份运行。父进程固定并验证二进制版本，负责健康检查、有界重启、日志脱敏和关闭。

使用专属继承式双向 IPC 通道，以消息长度分帧，并在启动时握手协议版本。父进程持有唯一对端句柄；不得开放监听 TCP 端口，也不得把该句柄传给 webview、Codex、CLIProxyAPI、研究或策略 worker。每次子进程会话通过继承通道传递随机会话凭证完成认证；凭证不能放入命令行参数、日志或普通工作区文件。协议不兼容时，在接受任何请求前拒绝连接。

Gateway 经认证的控制面通道读取权威不可变对象，不成为第二个 SQLite 写入者。只有其提供方签名层解析执行凭据引用。模型凭据和任意前端/Agent 订单字段不得进入 Gateway 请求。Gateway 故障使受影响 Live 账户 disarm，停止未派发工作，并对所有可能到达提供方的尝试完成对账后才允许显式重新 arming。重启子进程绝不自动重放订单修改请求。

Agent workspace 可以在 broker reconciliation 尚未结束时部分可用，但受影响 account 的 Live execution 必须保持 blocked。

---

## 6. 后端模块拆分

推荐 Rust workspace：

```text
crates/
  tradex-app/
  tradex-api/
  tradex-domain/
  tradex-runtime/
  tradex-thread/
  tradex-capability/
  tradex-market/
  tradex-instruments/
  tradex-portfolio/
  tradex-risk/
  tradex-approval/
  tradex-reservation/
  tradex-execution/
  tradex-reconciliation/
  tradex-provider-core/
  tradex-provider-alpaca/
  tradex-provider-t212/
  tradex-provider-binance/
  tradex-provider-bitget/
  tradex-storage/
  tradex-observability/
  tradex-security/
```

Quant/scientific workload 可以使用 Python worker，但金融 authority 必须留在 Rust。

---

## 7. Canonical Domain Model

### 7.1 Identity 规则

Domain logic 使用稳定 canonical IDs：

```text
workspace_id
thread_id
turn_id
account_id
instrument_id
proposal_id
approval_id
reservation_id
execution_attempt_id
provider_order_id
market_snapshot_id
policy_version
```

Provider-specific symbol/ID 只存在于 adapter mapping。

### 7.2 Instrument 示例

```yaml
instrument_id: equity:US:AAPL
asset_class: EQUITY
exchange: XNAS
currency: USD
providers:
  alpaca: AAPL
  trading212: provider_specific_id
```

```yaml
instrument_id: crypto:BTC/USDT:spot
asset_class: CRYPTO_SPOT
base: BTC
quote: USDT
providers:
  binance: BTCUSDT
  bitget: BTCUSDT
```

### 7.3 Decimal arithmetic

Price、quantity、notional、fees、FX conversion 与 risk calculation 必须使用 decimal-safe type。金融权限逻辑禁止 binary floating point。

### 7.4 Quantity semantics

```rust
enum OrderQuantity {
    Base(Decimal),
    Quote(Decimal),
    Notional(Decimal),
}
```

Adapter 根据 provider capability 与 instrument rules 显式转换。

---

## 8. Thread / Turn / Item Runtime 架构

### 8.1 Persistent Thread

只保存导航/default context：

```text
workspace_id
codex_thread_id
title
created_at
updated_at
default_agent_mode
linked_accounts[]
linked_instruments[]
linked_strategies[]
linked_artifacts[]
```

### 8.2 Immutable Turn snapshot

Turn 开始时持久化：

```text
turn_id
agent_mode_at_start
selected_account_id?
execution_context_at_start
capability_level_at_start
model_id
model_provider
attached_context_ids[]
attached_context_hashes[]
started_at
```

Turn 启动后该 snapshot 不可修改。Provider attempts 和完成时间属于启动快照之外只追加的 Turn 生命周期事件；历史模型/上下文不能从当前工作区默认值读取。

### 8.3 Runtime adapter

`tradex-runtime` 将 Codex protocol event 转换为 TradeX domain item，避免产品逻辑直接依赖不稳定 protocol detail。

```text
Codex JSON-RPC/JSONL
→ RuntimeProtocolAdapter
→ TradeX TurnEvent / ItemEvent
→ persistence
→ UI domain event
```

### 8.4 Generic approval 隔离

Codex approval event 可以暂停/恢复 Turn，但必须与 `FinancialApproval` 分开存储，任何代码路径都不能相互强转。

---

## 9. Agent Mode、Execution Context 与 Capability Policy

### 9.1 Agent Mode

```text
ASK
RESEARCH
BACKTEST
TRADE
```

### 9.2 Execution Context

```text
NONE_READ_ONLY
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

### 9.3 Capability Policy Service

根据具体 Turn snapshot 返回允许的 tool capability。

```rust
struct CapabilityDecision {
    level: CapabilityLevel,
    allowed_tools: Vec<ToolId>,
    execution_allowed: bool,
    reason: Option<String>,
}
```

规则：

- Ask/Research 不获得 current-market execution tools；
- Backtest 只有 historical simulation；
- Trade + Paper/Demo/Testnet 可获得 C3 execution tools；
- Trade + Live 可以达到 proposal capability C4；
- C5 不是常驻 tool grant，只在 Control Plane 内消费有效 transaction-specific approval 时临时成立；
- C6 不支持。

---

## 10. LLM Runtime 架构

### 10.1 Single egress

所有 model inference 都经过：

```text
127.0.0.1:8317 → CLIProxyAPI
```

TradeX、Codex、research tool、strategy code 禁止直接连接外部 LLM endpoint。

### 10.2 CLIProxyAPI supervision

后端负责：

- 验证 pinned binary/version；
- 占用/验证 8317 port；
- 启动 sidecar；
- 只注入 model credentials/config；
- probe `/v1/models`；
- 分类 stopped/port-conflict/unauthorized；
- 适用时 backoff restart；
- 退出时清理渲染的 DeepSeek configuration。

### 10.3 Provider routing

V1.0：

- ChatGPT subscription OAuth → GPT-5.6 series；
- DeepSeek official API → `deepseek-v4-flash`，显式使用 `thinking.type: disabled` 或 `thinking.type: enabled`。

选择的普通/推理模式与真实模型 ID 一起进入不可变路由/尝试快照和审计。必须通过 CLIProxyAPI 显式发送模式，不依赖上游默认值，也不将任一模式伪装为已弃用模型名。本次用户批准的模型名称澄清不增加提供方，不放宽单一出口或金融安全规则。

Cross-provider automatic fallback 默认 OFF。

### 10.4 Provider attempt audit

每次 attempt 持久化：

```text
turn_id
attempt_no
model_id
provider
started_at
ended_at
result
error_category?
quota_metadata?
```

若发生用户 opt-in 的 automatic fallback，两个 attempt 都必须保留并可审计。Provider switch 不得改变金融 capability state。

### 10.5 Model failure independence

`MODEL_UNAVAILABLE`、`OAUTH_EXPIRED`、`QUOTA_EXCEEDED` 可以阻止新的 Agent turn，但不得停止：

- live order monitoring；
- 已进入可信 control-plane flow 的 cancellation；
- reconciliation；
- account health processing；
- audit persistence。

---

## 11. Provider Registry 与 Schema-driven Connection Model

### 11.1 Provider definition

每个 integration 暴露：

```rust
struct ProviderDefinition {
    provider_id: ProviderId,
    display_name: String,
    environments: Vec<Environment>,
    credential_schema: CredentialSchema,
    capability_schema: CapabilitySchema,
    permission_rules: PermissionRules,
}
```

### 11.2 Credential schema

Schema 描述：

- fields；
- sensitivity；
- required/optional；
- environment applicability；
- help text；
- validation behavior。

后端不得假定所有 broker 都使用相同 `API Key + Secret`。

### 11.3 Secure connection sequence

```text
UI submits secret fields through privileged Tauri command
→ Rust validates shape
→ write secret to OS keychain
→ persist only credential reference metadata
→ adapter probes provider
→ discover permissions/capabilities
→ apply dangerous-permission safety gate
→ persist non-secret health/capability metadata
→ return sanitized account connection result
```

### 11.4 Permission gate

检测到以下权限时阻止 Live readiness：

- withdrawal；
- transfer；
- custody authority；
- unsupported margin/leverage-management authority。

无法 introspect permission 时标记 `UNVERIFIED`，要求用户 acknowledge，并持续在 Account Health 中可见。

---

## 12. Broker Adapter 架构

采用小型、按 capability 拆分的 interface，避免 oversized abstraction。

```rust
trait AccountAdapter { /* balances, positions, orders */ }
trait ExecutionAdapter { /* place/cancel/query execution */ }
trait MarketDataAdapter { /* quotes/bars/book + instrument metadata */ }
trait AccountStreamAdapter { /* private account/order stream */ }
```

### 12.1 Capability discovery

```rust
struct BrokerCapabilities {
    account_read: bool,
    position_read: bool,
    public_market_data: bool,
    private_account_stream: bool,
    order_types: Vec<OrderType>,
    fractional_quantity: bool,
    notional_orders: bool,
    extended_hours: bool,
    client_order_id: bool,
    paper_environment: bool,
    live_environment: bool,
}
```

在 provider 支持时，额外记录 TIF、min notional、post-only/venue constraints、cancellation behavior 和 provider-specific limit。

### 12.2 Provider-specific adapters

V1.0：

- Alpaca Paper；
- Trading 212 Demo / Live；
- Binance Testnet / Spot Live；
- Bitget Demo / Spot Live。

Adapter 将 canonical order 映射成 provider request，并把 provider state 映射回 normalized TradeX state。

### 12.3 Adapter error normalization

Provider error 映射到 canonical taxonomy，同时保留经过 redaction 的 provider raw code/message 作为诊断 metadata。

---

## 13. Market Data 架构

### 13.1 与 Execution 分离

Market data 与 broker execution 是不同 service contract。Broker adapter 即便能提供 market data，domain 也不能假设其数据完整或可用于 execution-grade 决策。

### 13.2 Subscription/access tiers

```text
Census — broad coarse universe/on-demand
Warm   — watchlists/candidates periodic refresh
Hot    — currently viewed/monitored active stream
Cold   — persisted historical/backtest data
```

MVP 使用 Hot active subscription 加 watchlist/universe 的 on-demand/coarse refresh，不对全部 universe 维持 always-on tick subscription。

### 13.3 Market snapshot

```rust
struct MarketSnapshot {
    id: MarketSnapshotId,
    instrument_id: InstrumentId,
    source: String,
    venue: Option<String>,
    provider_timestamp: DateTime<Utc>,
    received_timestamp: DateTime<Utc>,
    entitlement: Entitlement,
    freshness: FreshnessState,
    payload: MarketPayload,
}
```

所有 Live authority decision 都引用 persisted snapshot ID。

### 13.4 Entitlement metadata

每个 market-data integration 定义：

- realtime/delayed；
- local retention limit；
- redistribution restriction；
- commercial constraint；
- 适用 jurisdiction。

---

## 14. TimeService

TimeService 为以下能力提供一致时间语义：

- quote age；
- approval TTL；
- reconciliation deadline；
- event ordering；
- provider timestamp offset。

### 14.1 Data model

同时跟踪 wall-clock 与 monotonic time，检测：

- significant wall-clock jump；
- system resume discontinuity；
- provider/server offset 超 tolerance；
- quote age 不确定。

### 14.2 Fail-closed 规则

TimeService 认为 timing confidence 低于 Live threshold 时：

- 不签发/接受新的有效 Live approval；
- pre-execution freshness/TTL check 失败；
- account/execution surface 收到 clock/freshness blocking state；
- monitoring/reconciliation 继续运行。

---

## 15. Instrument Rules、Calendar 与 Corporate Actions

### 15.1 InstrumentRulesService

维护 venue/provider-specific constraints：

- tick size；
- price/quantity precision；
- minimum/maximum quantity；
- minimum/maximum notional；
- allowed order types；
- market-order constraints；
- trading status。

验证顺序：

```text
normalized order
→ InstrumentRulesService
→ deterministic risk
→ approval
→ pre-execution revalidation
→ provider adapter
```

### 15.2 MarketCalendarService

Equity：

- holidays；
- half days；
- sessions；
- extended-hours state；
- open/close timestamps；
- halts。

### 15.3 CorporateActionsService

追踪：

- splits；
- dividends；
- symbol changes；
- delistings；
- historical adjustment metadata。

在 execution 不被允许时，`MARKET_CLOSED` 与 `INSTRUMENT_HALTED` 是 deterministic blocking state。

---

## 16. Portfolio 与 Valuation 架构

### 16.1 Broker truth

Balance、position、open order、fill 来源于 provider state，并归一化成本地 projection。

### 16.2 Workspace base currency

Portfolio aggregation 使用配置的 base currency。

### 16.3 FX/stablecoin conversion

Conversion 记录：

```text
source
pair/path
provider timestamp
TradeX received timestamp
freshness
quality/depeg state
```

不得假设 `USDT = USD` 或 stablecoin 永远锚定。

如果 conversion quality 不可靠，且 Live risk 依赖该 normalized value，则执行必须 fail closed。

---

## 17. Deterministic Risk Engine

Risk Engine 与模型独立。

### 17.1 Inputs

Risk evaluation 消费 immutable snapshot/reference：

- account state；
- normalized order proposal；
- instrument rules；
- current market snapshot；
- policy version；
- portfolio/exposure state；
- open orders；
- active reservations；
- daily execution counters；
- market/calendar status；
- 必要时的 valuation provenance。

### 17.2 User-configurable policy

支持：

- maximum order notional/quantity；
- position/concentration limits；
- asset-class exposure；
- daily traded notional/loss；
- max open orders；
- max reserved capital；
- allowed/blocked instruments/venues/accounts；
- market-order enablement/slippage；
- price deviation；
- stale-price threshold；
- environment constraints。

Agent 无法修改 policy。

### 17.3 Hard safety rules

系统强制且不可 bypass：

- approval binding；
- duplicate-order protection；
- decimal/precision validation；
- instrument-rule validation；
- authoritative reconciliation；
- unhealthy-account block；
- stale snapshot block；
- reservation correctness；
- ambiguity 后禁止 blind retry；
- Order Gateway isolation。

### 17.4 Risk result

```rust
struct RiskDecision {
    proposal_id: ProposalId,
    policy_version: PolicyVersion,
    allowed: bool,
    checks: Vec<RiskCheckResult>,
    evaluated_at: DateTime<Utc>,
}
```

所有 Live risk decision 都要持久化。

---

## 18. Risk Policy Versioning 与 Serialization

Risk policy 对受影响 scope/account 进行 versioning。

Save flow：

```text
begin per-account single-writer transaction
→ persist new policy version
→ invalidate affected pending approvals
→ re-evaluate pending proposals
→ if policy weakened: DISARM affected live account
→ append audit events
→ commit
```

同账户的 approval consumption 进入同一 serialization boundary，确保 policy-save 与 approval-consume race 无法绕过 revalidation。

---

## 19. Order Draft 与 Immutable Proposal Service

### 19.1 Draft

`OrderDraft` 可以自由修改，不具备 authority。

### 19.2 Proposal generation

```text
OrderDraft
→ normalize canonical order
→ validate instrument/provider capability
→ capture market_snapshot_id where needed
→ bind policy_version
→ generate proposal_id
→ canonical serialize
→ SHA-256 proposal_hash
→ persist immutable OrderProposal
```

### 19.3 Material edit

任何 material edit 生成新的 proposal ID/hash。旧 approval 不可使用，但保留 audit history。

### 19.4 Canonical serialization

Hashing 使用 deterministic serialization，并明确 decimal/string normalization、field ordering、instrument/account identity、environment、TIF 与 quantity semantics。

---

## 20. Financial Approval Authority

`FinancialApproval` 与 Codex approval 完全分离。

### 20.1 Approval payload

```rust
struct FinancialApproval {
    approval_id: ApprovalId,
    intent: ApprovedFinancialIntent,
    account_id: AccountId,
    operation: FinancialOperation,
    policy_version: PolicyVersion,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    nonce: String,
    consumed_at: Option<DateTime<Utc>>,
}
```

ApprovedFinancialIntent 是带类型标签的联合：PLACE_ORDER 绑定 proposal_id/proposal_hash；CANCEL 绑定 cancellation_intent_id/intent_hash。operation 必须匹配标签、账户及不可变意图。撤单审批不能授权新建订单。

### 20.2 属性

- proposal-bound；
- account-bound；
- operation-bound；
- short-lived；
- single-use；
- material order change 时 invalidated；
- relevant policy change 时 invalidated；
- market snapshot/clock condition 不再满足时不可执行。

### 20.3 Approval consumption

Consumption 与 pre-execution validation、reservation creation 处于同一事务流程。已 consumed approval 不能复用。

---

## 21. Account-scoped Live Arming

Live arming 在 Control Plane 中按账户持久化：

```text
account A: DISARMED/ARMED
account B: DISARMED/ARMED
account C: DISARMED/ARMED
```

### 21.1 Arm requirements

接受 `ARM` 前：

- account connection healthy；
- reconciliation complete；
- credential/permission acceptable；
- provider capability 支持 Live；
- 不存在 blocking recovery state；
- 用户动作明确指向该账户。

### 21.2 Automatic disarm triggers

发生以下情况时 disarm 受影响账户：

- application restart；
- OS sleep/session lock；
- credential change；
- account health degradation；
- reconciliation failure；
- relevant pre-execution failure；
- risk-policy weakening；
- inactivity timeout。

### 21.3 Global Disable All

单个 Control Plane operation 原子地把全部 live account 设为 DISARMED，并追加 audit event。

应用重启、OS 休眠/锁屏和 Disable All 影响全部 Live 账户。凭据/健康故障影响指定账户；共享策略变更影响绑定该策略版本的全部账户。UI 当前选择不能决定作用范围。

Disable All 同时撤销尚未派发的执行许可。在受信派发边界之前停止的 `RESERVED` 尝试失效，并按 PRD §45 释放预留。已经进入提供方 I/O 的尝试保留预留并对账；Disable All 不等于隐式券商撤单。重新 arming 不恢复旧审批或派发许可。

---

## 22. Execution Reservation Service

Reservation 防止多个 Thread 重复消费同一 capacity。

### 22.1 Effective capacity

Risk 使用：

```text
broker available state
- open-order committed capacity
- active reservations
- submitted-but-unconfirmed exposure
± pending cancellation rules
= effective available capacity
```

### 22.2 Atomicity model

使用 per-account serialization：SQLite immediate transaction + application-level per-account async mutex/single-writer queue。

Database transaction 是 correctness boundary；in-memory lock 只是降低争用，不是唯一安全机制。

### 22.3 Reservation lifecycle

```text
APPROVED
→ RESERVED
→ SUBMITTING
→ ACCEPTED / REJECTED / UNKNOWN_RECONCILING
→ adjust/release only from authoritative resolution
```

### 22.4 Unknown state

`UNKNOWN_RECONCILING` 冻结 capacity，超时后也不能自动 release。

以 PRD §45 的过期/释放条件表为准。可能已经传输后的审批过期仅修改审批记录。提供方终态证据应先计入累计成交/费用，再释放未使用部分；仅收到撤单请求确认不能释放。证据引用、原状态、释放金额及最终账户状态须与预留处置在同一事务中持久化。

---

## 23. Pre-approval 与 Pre-execution Validation Pipeline

### 23.1 Pre-approval

```text
proposal immutable?
→ account/environment compatibility
→ account health
→ arming state for Live
→ provider capability
→ instrument rules
→ market/calendar state
→ market snapshot freshness/entitlement
→ TimeService confidence
→ deterministic risk
→ reservation capacity preview
→ approval request eligibility
```

### 23.2 Pre-execution

提交 broker 前立即执行：

```text
approval valid + unconsumed?
→ proposal hash matches?
→ policy version unchanged?
→ account still ARMED?
→ account healthy/reconciled?
→ quote still fresh?
→ TimeService trusted?
→ cash/positions/open orders refreshed as policy requires?
→ reservation created atomically
→ consume approval
→ transition RESERVED
→ call Order Gateway
```

Material failure 会 invalidate/reject flow，并在需要时要求新的用户同意。

---

## 24. Privileged Order Gateway

### 24.1 职责

Order Gateway 是唯一允许执行 Live provider mutation 的组件。

它只接收狭窄 internal request，不接收自由形式 Agent input。

```rust
struct GatewayExecutionRequest {
    execution_attempt_id: ExecutionAttemptId,
    intent_id: FinancialIntentId,
    operation: FinancialOperation,
    approval_id: ApprovalId,
    reservation_id: Option<ReservationId>,
    account_id: AccountId,
}
```

Gateway 在 privileged boundary 内重新加载权威 proposal/account data，不信任 caller 复制的 mutable fields。

intent_id 在 PLACE_ORDER 时解析为不可变 OrderProposal，在 CANCEL 时解析为 CancellationIntent。新建订单必须有 reservation_id；撤单可以引用已有预留，但不能仅为了撤单而新增购买预留。

### 24.1.1 派发权威与故障边界

1. 控制面在账户/策略串行化边界内创建预留、消费审批，并将执行尝试持久化为 `RESERVED`。
2. Gateway 经私有通道为该尝试申请一次性派发许可。控制面使用与 disarm/策略保存相同的串行化边界，重新校验 arming、健康、权限、策略、proposal 身份、行情/时钟/FX 可执行性及当前预留；先持久化许可，再回复。
3. Gateway 按账户串行处理派发与撤销，确认许可仍有效，并在提供方 I/O 前通过控制面持久化 `SUBMITTING` 意图。在此边界前已确认的 disable 阻止 I/O；跨过边界后，取消本地工作不能证明提供方没有收到请求。
4. 许可发出后若交付、子进程健康或传输确认不确定，必须保留容量并优先查询提供方。控制面不能根据 Gateway 未回复推断“未提交”。只有持久化的派发/撤销证据证明传输从未开始，或后续提供方证据解决了尝试，才允许释放。

Disable All 返回各账户 disarm 状态，以及各尝试的 STOPPED_BEFORE_DISPATCH 或 MAY_HAVE_SUBMITTED 处置。前者要求 Gateway 在派发边界前确认撤销；Gateway 不可达时按后者处理并保留容量。这些是派发处置，不是新增券商订单状态。

许可绑定执行尝试、账户、操作、proposal/hash、审批、适用的预留以及权限版本。沿用 execution attempt ID 去重；传输 request ID 本身不是金融幂等保证。Gateway 或控制面重启使许可失效，同时保留尝试用于对账。

### 24.2 Keychain access

Credential 只在 provider signing/execution layer 内读取，绝不返回调用方。

### 24.3 Network isolation

只有 provider adapter 需要高权限 broker/exchange outbound access。Agent/strategy process 不获得该 capability。

---

## 25. Idempotency 与 Submission Semantics

### 25.1 Internal identity

每次 execution 有 durable `execution_attempt_id`；provider 支持时使用由稳定 TradeX identity 派生的 `client_order_id`。

### 25.2 Safe retry classes

- query/read：安全时 bounded backoff retry；
- idempotent provider mutation：仅按 provider contract retry；
- ambiguous timeout 后的 non-idempotent order POST：**绝不 blind retry**。

### 25.3 Ambiguous timeout

如果网络结果未知：

```text
SUBMITTING
→ SUBMISSION_AMBIGUOUS
→ UNKNOWN_RECONCILING
→ query provider using client order ID / account orders / time-symbol-side fingerprints
```

Reservation 继续冻结，直到 evidence 解决状态。

---

## 26. Reconciliation 架构

Reconciliation 使本地 projection 收敛到 provider truth。

### 26.1 Triggers

- application startup；
- OS resume；
- private stream disconnect/reconnect；
- ambiguous submission；
- periodic health cycle；
- user-requested refresh；
- restore/import；
- detected state mismatch。

### 26.2 Priority

Rate-limit budget 优先：

1. ambiguous-order resolution；
2. open live orders；
3. execution safety 所需 fills/positions/balances；
4. private stream recovery；
5. research/history traffic。

### 26.3 Reconciliation algorithm

```text
load local unresolved/open execution records
→ fetch authoritative provider state
→ map provider orders/fills to canonical IDs
→ detect missing/changed state
→ append reconciliation events
→ update local projection
→ adjust/release reservations only from resolved truth
→ recompute account health
```

### 26.4 Stream handling

Private WebSocket/account stream 是低延迟 signal，不是唯一真相。REST/query reconciliation 用来修复 missed event。

---

## 27. Manual Resolution

超过 bounded automatic reconciliation window（PRD 默认 5 分钟）后，未解决 submission 仍冻结，account 保持 unhealthy/disarmed。

允许：

### 27.1 Confirmed not submitted

需要足够 provider/account evidence。之后：

- 标记 execution attempt resolved-not-submitted；
- transactionally release reservation；
- 执行 account reconciliation；
- 只有 health check 通过后才恢复 readiness。

### 27.2 Confirmed submitted

要求/绑定 broker order identity，再根据 provider truth reconciliation 并调整 reservation。

### 27.3 Keep reconciling

保持 reservation frozen，并继续 query-first resolution。

仅用户口头/本地 assertion 不能恢复 account healthy/live-ready。

### 27.4 证据契约

Manual Resolution 提交 `{execution_attempt_id, account_id, decision, evidence_ids, broker_order_id?, expected_state_version}`。decision 为 `CONFIRMED_NOT_SUBMITTED`、`CONFIRMED_SUBMITTED` 或 `KEEP_RECONCILING`；这些是处置决策，不是新增订单状态。后端持有脱敏证据记录：提供方/账户、查询范围、查询时间与覆盖窗口、提供方请求/引用、结果及关联券商身份。凭据缺失、分页不完整、提供方可见性延迟或单次空查询均不能证明未提交。

确认已提交必须提供与目标标的/操作相符、已核验且属于该账户的券商订单身份；确认未提交必须满足适配器专属的充分缺席证据规则。无法证明缺席时，只允许 Keep reconciling。后端在提交处置时再次校验证据与预期状态版本；期间到达的成交优先于陈旧人工输入。UI 可以查看证据，但不能自行宣告证据已验证。处置成功后重新校验健康，且保持 DISARMED，直至另一次显式用户操作。

---

## 28. Order State Machine

Normalized backend order states：

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

### 28.1 Transition ownership

- Draft/Proposal：proposal service；
- Risk rejected：Risk Engine；
- Needs approval/Approved 与审批过期：Approval Authority；
- Reserved：Reservation Service；
- Submitting：Order Gateway；
- Accepted/Partial/Filled/Rejected/Cancelled 与券商订单过期：adapter/reconciliation 获得的 provider truth；
- Unknown reconciling：submission/reconciliation coordinator。

UI 或 Agent text 不得直接写入这些 state。

---

## 29. Cancellation 架构

Cancellation 是 transaction-specific。

撤单意图不可变，绑定 `operation=CANCEL`、账户/环境、标的、提供方订单 ID、最新观察订单状态、累计成交/剩余数量以及提供方快照时间。Arming 是前置条件，完成后必须返回同一撤单意图，绝不进入新建订单审批。Arming 后再次刷新提供方状态；剩余数量/状态变化时，刷新撤单意图并重新获得同意。已成交/终态订单不能撤销。使用撤单专属可执行性规则：股票市场不接受新订单，不自动等于提供方禁止撤单。

流程：

```text
refresh provider order state
→ verify cancellable state/capability
→ create cancellation intent
→ deterministic safety checks
→ request user cancellation approval
→ consume cancellation approval
→ Order Gateway cancel
→ CANCEL_PENDING
→ reconcile
→ CANCELLED or authoritative terminal state
```

如果取消前已经 partial fill，需要根据剩余 exposure 调整 reservation。

V1.0 不要求 broker-native amend/replace。修改 open order 使用 confirmed cancel 后再创建新 proposal。

---

## 30. Paper / Demo / Testnet 架构

### 30.1 Local Paper

Local Paper 是 TradeX 自有 simulation engine，必须与 provider-hosted environment 明确区分。

尽可能复用 normalized order/event domain，但永远不穿过 Privileged Live Order Gateway。

### 30.2 Provider-hosted non-live environment

Alpaca Paper、Trading 212 Demo、Binance Testnet、Bitget Demo 使用 provider adapter，并带明确 non-live environment metadata。

它们尽可能映射到同一 order-state model，但不使用 Live arming/financial approval 作为 Live authority gate。

### 30.3 Environment invariant

Execution attempt 的 environment 不可修改。Demo/Testnet order 不能因为 adapter routing 或 UI state 改变而变成 Live。

---

## 31. Backtest 与 Strategy 架构

### 31.1 Backtest engine

Backtest local + deterministic。持久化：

- inputs/parameters；
- strategy version/hash；
- dataset hash/provider；
- adjustment/calendar/timezone assumptions；
- commission/slippage model；
- engine version；
- 完整 metrics/trades/equity curve。

### 31.2 Sandbox

Strategy worker 可以访问 approved historical data 与 numerical library，但不能访问：

- keychain；
- broker credentials；
- arbitrary network；
- privileged Order Gateway；
- unrestricted filesystem。

### 31.3 Live strategy output

Strategy 只输出 signal，不输出 executable provider request。Signal → proposal → risk → approval → reservation → gateway。

---

## 32. Storage 架构

### 32.1 SQLite

权威 transactional/domain state：

- workspace metadata；
- Thread/Turn/Item mapping；
- account metadata/capabilities/health；
- watchlists；
- risk policies/versions；
- Local Paper state；
- order drafts/proposals；
- risk decisions；
- approvals；
- reservations；
- execution attempts；
- reconciliation events；
- portfolio snapshots；
- provider connection metadata；
- settings/memory；
- audit log。

### 32.2 DuckDB

Analytical/historical：

- persistent 1-minute+ OHLCV；
- screener materializations；
- features；
- portfolio analytics；
- historical joins；
- backtest datasets/results。

### 32.3 Parquet

可选大型 immutable historical/interchange layer。V1.0 correctness 不依赖 Parquet。

### 32.4 Filesystem

```text
~/.tradex/
  workspaces/
  artifacts/
  strategies/
  datasets/
  logs/
  broker-cache/
  backups/
  exports/
```

Secret 不进入这些普通文件目录。

---

## 33. SQLite Transaction 与 Concurrency Model

### 33.1 Single-writer safety domains

关键金融 mutation 按账户 serialization：

- risk policy save；
- approval consumption；
- reservation create/release；
- execution attempt creation；
- 会改变 execution capacity 的 reconciliation update。

### 33.2 Transaction rules

使用 explicit transaction 与 foreign-key constraints。推荐：

- 金融状态 mutation 使用 `BEGIN IMMEDIATE`；
- 非关键 editable metadata 使用 optimistic version；
- financial event 尽可能 append-only；
- 对 single-use approval consumption 和 idempotency identity 设置 unique constraint。

### 33.3 推荐 uniqueness constraints

```text
UNIQUE(proposal_hash)
UNIQUE(approval_id)
UNIQUE(reservation_id)
UNIQUE(execution_attempt_id)
UNIQUE(account_id, provider_client_order_id) where supported
```

Approval table 必须 transactionally enforce consumed-at-most-once。

---

## 34. Event 与 Audit 架构

### 34.1 Domain events

关键 state change append immutable event：

- TurnStarted/Completed/Failed；
- ProviderAttemptStarted/Failed/Completed；
- AccountArmed/Disarmed；
- RiskPolicyChanged；
- ProposalGenerated；
- RiskEvaluated；
- ApprovalIssued/Invalidated/Consumed/Expired；
- ReservationCreated/Adjusted/Released/Frozen；
- ExecutionStarted；
- BrokerAcknowledged；
- FillObserved；
- SubmissionBecameAmbiguous；
- ReconciliationStarted/Resolved/Failed；
- ManualResolutionRecorded。

### 34.2 Tamper evidence

建议 financial audit event 增加：

```text
sequence
previous_event_hash
event_hash
```

提供本地 tamper detection，但不宣称达到受监管 immutable ledger 的法律保证。

### 34.3 Secret redaction

Structured logging 在 serialize 前做 field-level redaction，不能只依赖下游 log scrubber。

---

## 35. Error Taxonomy

后端返回 canonical categories：

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

每个 error 包含：

- category；
- stable internal code；
- human-safe message；
- blocking/non-blocking；
- remediation actions；
- related aggregate ID；
- 适用时经过 redaction 的 provider code/detail。

---

## 36. Rate-limit 与 Backpressure 架构

### 36.1 Provider budgets

维护 per-provider/per-account rate-limit state 与 request class。

Priority：

```text
P0 execution reconciliation
P1 account/order safety refresh
P2 active market/context
P3 research/history/background
```

### 36.2 Backpressure

高流量 stream 经过 bounded channel：

- durable processing 前不得丢失 financial order/fill event；
- 可替代 quote/UI update 可以 coalesce；
- 高频 market data 按 tier backpressure/sample；
- provider 支持时保存 account stream sequence/checkpoint metadata。

### 36.3 Codex backpressure

Codex queue overload 只能影响 Agent turn，不能挤占 control-plane reconciliation/execution task。

---

## 37. Recovery 架构

### 37.1 Application restart

启动时：

- 所有 live accounts 初始 DISARMED；
- 加载 unresolved/open live executions；
- 在 NFR 目标内开始 reconciliation；
- account health 恢复前保持 Live execution disabled。

### 37.2 Sleep/resume

Sleep/session lock 时：

- disarm live accounts；
- 安全时持久化 runtime checkpoint；
- resume 后重新建立 TimeService confidence；
- 重连 private streams；
- 新 Live execution 前先 reconciliation。

### 37.3 Stream disconnect

Private-stream loss 将 account 标记 degraded，阻止新的 Live execution，并触发 query-based reconciliation/reconnect。

### 37.4 Corrupted local projections

Order/position projection 可以从 broker truth 重建。Local audit/proposal history 有价值，但不能覆盖 provider truth。

---

## 38. Workspace Export / Import / Backup

### 38.1 Export

Archive 包含 non-secret workspace state、schemas/manifests、artifacts、strategy files 与允许的数据/metadata。

### 38.2 Import

```text
validate archive manifest/schema
→ backup current state
→ migrate/restore non-secret state
→ verify credential references exist
→ mark live accounts DISARMED
→ reconcile account state
→ enable readiness only after health checks
```

Raw broker secrets 绝不从 Workspace archive 导入。

### 38.3 Retention

普通 artifact/cache 可以遵循用户 retention。Unresolved execution/reconciliation records 禁止自动删除。

---

## 39. Observability

Local metrics/logging 追踪：

- Codex turn/tool latency；
- token/model provider usage；
- CLIProxyAPI health；
- market-data freshness；
- WebSocket reconnects；
- broker REST latency；
- rate-limit state；
- order acknowledgement latency；
- fill convergence latency；
- reconciliation failures；
- unknown-order count；
- risk rejection count；
- storage growth。

External telemetry 默认关闭；开启后也必须进行 explicit redaction，并排除 broker secret。

---

## 40. Security Controls

### 40.1 Credentials

- broker secrets 只进入 OS credential storage；
- SQLite credential reference 不含 secret material；
- ChatGPT OAuth 保留在 CLIProxyAPI auth-dir；
- DeepSeek API key 保存在 OS keychain，由 Rust 渲染到严格文件权限的 sidecar config；
- 退出时清除临时 rendered model config；
- logs redact auth headers/signatures/tokens。

### 40.2 Prompt-injection boundary

Research content 是 untrusted data。Tool contract 必须分离 data 与 authority。任何 research source 返回文本都不能：

- 修改 risk policy；
- Arm account；
- 签发 financial approval；
- 访问 keychain；
- 调用 Order Gateway。

### 40.3 Sandbox

Strategy/research subprocess 使用 least privilege、restricted filesystem/environment，且不能访问 privileged broker credential。

### 40.4 Dependency pinning

固定并测试：

- Codex App Server；
- CLIProxyAPI；
- provider SDK/API schema assumptions；
- database migrations。

升级要求 compatibility/schema diff test。

---

## 41. Backend API / IPC Surface

前端使用的规范命令名（版本 1，见 §41.1）：

### Workspace/runtime

```text
workspace.open
workspace.export
workspace.import
runtime.status
runtime.restart_sidecar
time.status
time.revalidate
```

### Agent capability

```text
agent.capabilities
```

### Threads

```text
thread.list
thread.get
thread.create
turn.start
turn.cancel
turn.retry
```

### Markets/accounts

```text
market.catalog
market.get
market.snapshot
market.history
market.screen
screener.list
screener.save
screener.update
screener.attach
watchlist.list
watchlist.create
watchlist.rename
watchlist.delete
watchlist.add
watchlist.remove
account.list
account.get
account.refresh
account.arm
account.disarm
account.disable_all_live
portfolio.get
```

### Data-source policy

```text
data.source.catalog
data.source.probe
```

### Providers

```text
provider.list_definitions
provider.get_schema
provider.connect
provider.disconnect
provider.probe
provider.permissions
model.login_chatgpt
model.configure_deepseek
model.set_default
model.set_fallback_policy
```

### Risk/trading

```text
risk.get_policy
risk.save_policy
trade.save_draft
trade.generate_proposal
trade.refresh_proposal
trade.request_approval
trade.approve
trade.reject
trade.cancel_request
trade.cancel_approve
trade.manual_resolution
trade.resolution_evidence
```

### Local Paper simulation（S16）

```text
paper.account.ensure
paper.get
paper.order.submit
paper.order.cancel
paper.quote.refresh
paper.scenario.set
```

### Alpaca Paper 订单提交（S17）

```text
alpaca.paper.order.submit
alpaca.paper.order.attempt.get
alpaca.paper.order.reconcile
```

### Strategy/backtest

```text
strategy.list
strategy.save_version
backtest.run
backtest.cancel
backtest.get
backtest.list
backtest.compare
```

### Artifacts

```text
artifact.save
artifact.list
artifact.get
artifact.export
```

每个 command 都使用明确 schema version 和 sanitized error。

---


### 41.1 权威传输契约（版本 1）

本节和 §42 是前端/控制面传输契约的权威来源，Frontend ARD §10 引用它们；概念模块分组不是另一套命令名。本节命令为精确传输名称，不能由两端独立改名。新增操作必须先定义 schema，再实现。

~~~ts
interface CommandEnvelope<T> {
  requestId: string;
  command: string;
  schemaVersion: 1;
  payload: T;
}
type ResultEnvelope<T> =
  | { requestId: string; schemaVersion: 1; ok: true; data: T; stateVersion?: string }
  | { requestId: string; schemaVersion: 1; ok: false; error: TradeXError };
interface TradeXError {
  category: string; // PRD §51 canonical error category
  code: string;     // stable operation-specific reason, not an order state
  message: string;  // sanitized user-facing explanation
  retryable: boolean; // not permission to retry a financial mutation
  blocking: boolean;
  remediationActions: Array<{ id: string; label: string }>;
  aggregateId?: string;
  providerCode?: string; // sanitized, optional
}
~~~

不支持的 schema 版本必须在派发前以 category INTERNAL_ERROR、code IPC_SCHEMA_UNSUPPORTED 失败；UI 提供运行时兼容性/重连指引。未知命令返回 IPC_COMMAND_UNKNOWN；传输 payload 格式错误返回 INTERNAL_ERROR、code IPC_PAYLOAD_INVALID，订单值无效使用 INVALID_ORDER。状态版本不匹配时返回 STATE_STALE、code STATE_VERSION_CONFLICT 且不发生修改；客户端重新读取权威对象后才能请求新的同意。

| 前端职责 | 精确命令 | 最小 payload 契约 |
|---|---|---|
| 读取 Agent capability | agent.capabilities | workspace_id、agent_mode、execution_context、可选 account_id、attached_contexts，以及可选 requested_tool/requested_level 探针；返回 capability level、allowed tools、executionAllowed 和阻断原因；不允许或未知工具以及 C5/C6 返回 UNSUPPORTED_CAPABILITY |
| 保存可编辑草稿 | trade.save_draft | 更新时含 draft_id、草稿字段和 expected_state_version；不授予权限 |
| 生成 proposal | trade.generate_proposal | draft_id、expected_draft_version；后端生成不可变身份/hash |
| 刷新陈旧 proposal | trade.refresh_proposal | proposal_id、expected_state_version；返回新 proposal 并使旧同意失效 |
| 请求审批 | trade.request_approval | proposal_id、expected_state_version；返回可执行性与不可变审批摘要 |
| 显式批准 | trade.approve | proposal_id、proposal_hash、approval_id、expected_state_version；后端重新校验后才能消费 |
| 拒绝审批 | trade.reject | approval_id、expected_state_version；不执行券商操作 |
| 准备撤单 | trade.cancel_request | account_id、broker_order_id、expected_state_version；查询提供方状态并返回不可变撤单意图 |
| 批准撤单 | trade.cancel_approve | cancellation_intent_id、approval_id、expected_state_version；operation 始终为 CANCEL |
| 查看处置证据 | trade.resolution_evidence | execution_attempt_id、account_id；返回后端持有的证据与允许的决策 |
| 处置未知提交 | trade.manual_resolution | §27.4 payload；提交处置时再次核验 decision/evidence |

状态版本是后端生成、限于返回聚合对象的不透明 token。Decimal 金额使用规范化字符串；ID、枚举、时间表示及必填/可选字段属于命令的版本化 schema。request ID 只关联一次交互，不能替代 proposal/approval/execution 身份。改变权限的命令超时后必须先查询状态再决定重试；不得把传输重试变成重复同意。

#### 41.1.1 回测生命周期 payload（S15）

`backtest.run`、`backtest.get`、`backtest.list`、`backtest.compare` 和 `backtest.cancel` 使用版本 1 且限定在当前 workspace。`backtest.run` 只接受以下有界冻结配置；renderer 不能提交 run ID、state、request hash、观测时间、引擎版本或 state version。后端解析已保存的 strategy hash 和可信时间，并在 worker 启动前持久化 QUEUED run。

```ts
interface BacktestRunRequest {
  workspaceId: string;
  strategyVersionId: string;
  expectedStrategyHash?: string;
  instrumentId: string;
  datasetId: string;
  startAt: string; // UTC RFC 3339
  endAt: string; // UTC RFC 3339，end >= start
  barInterval: "1m" | "5m" | "15m" | "30m" | "1h" | "1d";
  startingCash: string; // 规范化非负 decimal，且 > 0
  commission: string; // 规范化非负 decimal
  slippage: string; // 规范化非负 decimal
  portfolioSeed?: string;
  parameters?: StrategyParameter[]; // 省略时使用已保存版本的参数
  fixtureScenario?: "SUCCESS" | "FAILURE" | "CANCELLED" | "LOOKAHEAD" | "SURVIVORSHIP" | "SPLIT" | "DIVIDEND" | "TIMEZONE" | "DATA_GAP" | "DATASET_HASH_MISMATCH"; // 仅集成测试 fixture；生产环境禁止
}
interface BacktestCancel {
  workspaceId: string;
  runId: string;
  expectedStateVersion: string;
}
interface BacktestRunSummary {
  runId: string;
  strategyVersionId: string;
  strategyHash: string;
  instrumentId: string;
  datasetId: string;
  startAt: string;
  endAt: string;
  barInterval: string;
  state: "QUEUED" | "RUNNING" | "COMPLETED" | "FAILED" | "CANCELLED";
  requestHash: string;
  updatedAt: string;
  resultHash?: string;
  tradeCount?: number;
}
interface BacktestLibrary {
  workspaceId: string;
  stateVersion: string;
  runs: BacktestRunSummary[];
}
interface BacktestCompareRequest {
  workspaceId: string;
  leftRunId: string;
  rightRunId: string;
}
interface BacktestComparison {
  workspaceId: string;
  left: BacktestRun;
  right: BacktestRun;
  leftCurve: {pointCount: number; startAt: string; endAt: string; startEquity: string; endEquity: string; maxDrawdown: string};
  rightCurve: {pointCount: number; startAt: string; endAt: string; startEquity: string; endEquity: string; maxDrawdown: string};
  inputDifferences: Array<{field: string; left: string; right: string}>;
  manifestDifferences: Array<{field: string; left: string; right: string}>;
  metrics: {
    return: {left: string; right: string; difference: string};
    sharpe: {left: string; right: string; difference: string};
    sortino: {left: string; right: string; difference: string};
    maxDrawdown: {left: string; right: string; difference: string};
    winRate: {left: string; right: string; difference: string};
    profitFactor: {left: string; right: string; difference: string};
    turnover: {left: string; right: string; difference: string};
    tradeCount: {left: string; right: string; difference: string};
  };
  historicalSimulation: boolean;
  limitations: string[];
}
interface BacktestManifest {
  strategyVersion: string;
  strategyHash: string;
  datasetId: string;
  datasetHash: string;
  dataProvider: string;
  retrievedAt: string;
  startAt: string;
  endAt: string;
  adjustmentMethod: string;
  timezone: string;
  marketCalendarVersion: string;
  commissionModel: string;
  commission: string;
  slippageModel: string;
  slippage: string;
  startingCash: string;
  seed: string;
  parameters: StrategyParameter[];
  engineVersion: string;
  runtimeVersion: string;
  guardChecks: Array<{name: string; state: "PASSED"; detail: string}>; // look-ahead、survivorship、split、dividend、timezone、缺口
  manifestHash: string;
}
interface BacktestResult {
  metrics: {
    return: string;
    sharpe: string;
    sortino: string;
    maxDrawdown: string;
    winRate: string;
    profitFactor: string;
    turnover: string;
    tradeCount: number;
  };
  equityCurve: Array<{observedAt: string; equity: string; drawdown: string}>;
  trades: Array<{
    tradeId: string;
    instrumentId: string;
    side: string;
    quantity: string;
    price: string;
    grossValue: string;
    commission: string;
    slippage: string;
    realizedPnl: string;
    observedAt: string;
  }>;
  manifest: BacktestManifest;
  historicalSimulation: true;
  limitations: string[];
  resultHash: string;
}
interface BacktestRun {
  // 冻结配置和生命周期字段在此省略
  result?: BacktestResult; // state=COMPLETED 时必需；其他状态不得存在
}
```

省略 `parameters` 或传入空数组时，后端使用已保存 strategy version 的参数。`fixtureScenario` 仅由集成测试 fixture 接受，生产运行时不得使用。

`backtest.get` 接受 `{workspaceId, runId}`，返回完整冻结配置、`runId`、`requestHash`、`state`、适用时的 failure/remediation 以及不透明 `stateVersion`。`COMPLETED` 响应包含经过 hash 校验的 `BacktestResult`，涵盖完整指标集、equity curve、trade list、六项数据 guard、限制说明和可复现 manifest。字段缺失、hash 被篡改、strategy/dataset identity 不匹配或结果不是历史模拟时必须 fail closed。稳定 request identity 是 strategy/version/hash、dataset hash、instrument、dataset、日期范围、时区、calendar/adjustment 假设、bar interval、规范化成本、starting cash、portfolio seed、parameters、runtime 和 engine version 的 canonical SHA-256；观测时间是 metadata，不参与 identity。同一配置的 retry 保持 request identity，但每次产生新的 run identity。

`backtest.list` 接受 {workspaceId}，返回按最新优先排序的、有界、workspace 作用域 `BacktestLibrary` 持久化摘要。后端在返回前重新校验每个 projection；该目录只用于 compare 选择，不授予执行权限。`backtest.compare` 恰好接受两个不同 run ID，并在构造 `BacktestComparison` 前重新加载两个 projection。两个 run 必须属于当前 workspace、均为 `COMPLETED`，且 request/result/manifest identity 有效并满足 `historicalSimulation=true`；未知、跨 workspace、同一 run、未完成或被篡改的 identity 必须 fail closed 且不产生修改。响应保留两侧完整 run identity 以及有界 input/manifest/metric/curve 差异。Compare 只读历史分析，永远不调用 broker、order、approval、reservation、gateway 或 provider mutation。模型不可用时，不得清空该持久化目录或已完成结果。

持久化状态机为 `QUEUED -> RUNNING -> COMPLETED|FAILED|CANCELLED`（QUEUED 可在启动前 FAILED 或 CANCELLED）。每次迁移都在 SQLite immediate transaction 中执行并进行 state-version compare-and-swap。过期的 cancel token 返回 `STATE_STALE / STATE_VERSION_CONFLICT` 且不修改状态；缺失 token 会在派发前被版本 1 payload schema 拒绝。terminal projection 不可变。重新打开 workspace 时，QUEUED/RUNNING run 会被对账为带类型的 `CANCELLED`。回测 worker 永不提交券商订单，也不调用 execution account。

模型推理不可用时，运行时状态、账户查询、事件订阅/重放和对账仍必须可用。纯前端导航/草稿输入不需要后端命令。

---

### 41.2 工作区启动 payload（S01）

以下版本 1 payload 定义首个桌面垂直切片。所有对象拒绝未声明字段。ID 是非空不透明字符串；sequence 是 0 到 JavaScript 安全整数上限之间的整数。时间为 UTC RFC 3339 字符串。下述命令均不授予金融权限。

| 命令 | Payload | 成功 data |
|---|---|---|
| workspace.open | `{path?: string, name?: string, baseCurrency?: string}`；省略时使用应用默认工作区目录；提供的 path 必须为绝对目录路径 | Workspace 投影：`{workspaceId, name, baseCurrency, path, createdAt, lastOpenedAt, storageSchemaVersion: 5}`，result envelope 附不透明 `stateVersion` |
| runtime.status | `{}` | `{components: [{id, status, message}], modelAvailable: boolean, liveExecutionAvailable: boolean}`；初始 Codex/CLIProxyAPI 为 `NOT_CONFIGURED`，不得推断健康 |
| domain.snapshot | `{aggregateType: "workspace", aggregateId: string}` | `{aggregateType, aggregateId, projection: Workspace, lastSequence}` |
| domain.subscribe | `{aggregateType: "workspace", aggregateId: string, afterSequence: number}` | 在交付完确认游标之前全部保留事件后返回 `{aggregateType, aggregateId, afterSequence, lastSequence, replayedCount}`；后续事件沿用同一传输通道 |

桌面传输在规范命令封装之外提供 Tauri Channel。控制面从原生 webview 获取 consumer 身份，不使用调用方提供的身份。重新订阅替换该 consumer 对此聚合体的订阅；交付失败移除订阅，客户端重新加载快照并订阅。切换活动工作区撤销旧订阅。重放到实时交付的交接与命令修改共享同一串行化边界，确保并发提交事件不会落入交接缺口。无界面集成 runner 通过继承 stdio 的逐行 JSON 分帧通信，不监听网络端口。

不存在/未知聚合体返回 `STATE_STALE / IPC_AGGREGATE_NOT_FOUND`。超前游标返回 `STATE_STALE / STATE_VERSION_CONFLICT`。保留事件存在缺口时返回 `STATE_STALE / IPC_REPLAY_UNAVAILABLE`，要求重载快照。缺少/失败订阅交付返回 `INTERNAL_ERROR / IPC_SUBSCRIPTION_CHANNEL_REQUIRED` 或 `IPC_SUBSCRIPTION_DELIVERY_FAILED`。存储打开失败返回脱敏的 `INTERNAL_ERROR` 代码 `WORKSPACE_PATH_INVALID`、`WORKSPACE_BUSY`、`WORKSPACE_OPEN_FAILED`、`WORKSPACE_SCHEMA_UNSUPPORTED` 或 `WORKSPACE_INTEGRITY_FAILED`；保留原活动工作区，不覆盖损坏/外来数据库，不返回原始文件系统/SQL 诊断。

name（1–120 个字符，不含控制字符）与 baseCurrency（三个大写字母）仅用于新建；重开返回持久化配置。默认名称为目录名、默认基础货币为 USD。不支持的换算货币对在组合/数据集成提供前保持不可用。打开操作在数据库不存在时创建非秘密工作区库，否则重开既有身份。每次成功 open 事务更新 `lastOpenedAt` 并追加一个 `workspace.opened` 领域事件，payload 为所得 Workspace 投影；workspace ID 与 `createdAt` 不变。新库初始化在事务中完成；升级已识别的既有 schema 前必须生成一致备份，迁移后校验完整性。进程级 OS lock 防止两个控制面同时把同一工作区当作活动写入者。未知较新 schema 和外来 SQLite 库均拒绝迁移。浏览器投影/渲染检查不替代实际 Tauri 与持久化存储验证。

---

### 41.3 券商连接载荷（S02）

版本 1 增加以下精确操作。所有输入对象拒绝额外字段和显式 null；字符串及集合长度受限，opaque ID/state version 不得为空。连接绑定活动 `workspaceId`，创建后 provider/environment 不可变。wire 不接受原始秘密、HTTP 目标地址、调用方声明的权限、arming 标志或自行指定的 Keychain 引用。

| Command | Payload | Success data |
|---|---|---|
| provider.list_definitions | `{}` | `{providers: ProviderDefinition[]}`，包含提供方/环境可用性、字段敏感性/必填性/验证/帮助、请求权限规则及支持能力 |
| provider.get_schema | `{providerId, environment}` | 该受支持组合的 ProviderDefinition；不支持的组合在秘密输入前拒绝 |
| provider.connect | `{step: "test", workspaceId, providerId, environment, label}` | 原生安全输入及只读认证后的 AccountConnection，状态 REVIEW_REQUIRED；秘密字段不经过 webview command envelope |
| provider.connect | `{step: "confirm", workspaceId, connectionId, expectedStateVersion, acknowledgeUnverified: boolean}` | AccountConnection；仅确认已成功测试的精确审阅版本；未知权限须显式确认且继续标为 UNVERIFIED |
| provider.probe / account.refresh | `{workspaceId, connectionId, expectedStateVersion}` | 实际读取结果/健康状态；读取失败保留旧观察和上次成功同步时间，标记 stale/error 并返回脱敏错误 |
| provider.permissions / account.get | `{workspaceId, connectionId}` | 分别为 PermissionReview / AccountConnection |
| account.list | `{workspaceId}` | `{accounts: AccountConnection[]}`；包含待处理、失败及断开记录以供恢复/历史查询 |
| provider.disconnect | `{workspaceId, connectionId, expectedStateVersion}` | 删除凭据前先持久化 DISCONNECTED；删除失败保持 DELETE_PENDING，可使用新版本重试；不修改提供方、不取消外部订单 |

ProviderDefinition 包含 `providerId`、`displayName`、`environment`、`available`、`helpText`、`fields`（`id`、`label`、`inputType`、`required`、`secret`、`maxLength`、`helpText`、适用环境）及权限要求。仅已实现的受支持组合可连接；不可用的目录项说明原因。Local Paper 内置且无需凭据，不代表外部探测成功。

AccountConnection 包含不可变的 `connectionId`、`workspaceId`、`providerId`、`environment`、`createdAt`；`label`、opaque `stateVersion`、`updatedAt`、`connectionState`（CONNECTING / REVIEW_REQUIRED / CONNECTED / FAILED / DISCONNECTED）；分开的连接/认证/凭据/私有流/对账/执行资格/arming 健康状态；可选的既有账户数据、上次成功同步及 PermissionReview。数据包含远端身份/类型、可用时的币种、规范 decimal 字符串余额、可选账户级购买力、持仓/未完成订单、已观察能力及明确限制。缺失值不可用，不能默认零。Alpaca Paper 的 `buyingPower` 必须是提供方账户币种下的精确 decimal，不能从 cash 或 equity 推算。PermissionReview 区分 VERIFIED/UNVERIFIED、已检测权限、禁止/不支持权限、确认记录及 IP 限制状态。读取成功不代表完整密钥权限或金融授权。所有 Live 账户保持 DISARMED；S02 不授予执行资格。

账户观察在 balance 上增加可选 decimal 字符串 `reserved`、`inPies`；在 position 上增加可选 `instrumentCurrency`、`marketValueCurrency`；在 open order 上增加可选 `currency`、`filledValue`。提供方金额型订单的 `filledQuantity` 可缺失/null。旧持久化投影缺少字段时保持不可用。显示的金额单位来自对应观察币种，不将工作区币种视为隐式换算。Trading 212 summary 不提供账户子类型，应明确显示不可观测。提供方 JSON number 必须无二进制浮点转换地规范为精确 decimal wire 字符串。

控制面在原生输入前分配不可变连接及私有引用；仅受信提供方层采集/保存/解析 Keychain 值，普通存储只有 metadata/reference。取消使待处理连接失效并只清理自身凭据；初次测试失败保持明确失败并清理自身凭据；清理失败显示 DELETE_PENDING，可重试断开，期间禁止探测。原生弹窗和网络 I/O 不持有领域状态锁。提交结果前再次校验工作区会话、连接身份及期望状态；断开或切换工作区使进行中的结果失效。受信层在继续 I/O 前检查有效性，并清理放弃的新凭据。重启时中断的 CONNECTING 连接转为 DISCONNECTED / DELETE_PENDING，须清理后重新连接；旧观察保持 stale，直至引用检查及新探测成功；不得自动确认审阅或恢复 arming。

连接/健康变更与 `account.health.changed` 事件在同一 SQLite 事务提交。`account` aggregate 使用 `connectionId`、独立连续序列及 AccountConnection projection。`domain.snapshot` / `domain.subscribe` 接受该 aggregate，复用 §41.2 的恢复及 replay-to-live 保证。同一 consumer 可订阅不同 aggregate；替换只作用于 consumer + aggregate。

错误使用 PRD §51 类别与稳定 code：`PROVIDER_UNSUPPORTED`、`PROVIDER_NATIVE_ENTRY_REQUIRED`、`PROVIDER_ENTRY_CANCELLED`、`PROVIDER_ENTRY_BUSY`、`PROVIDER_ALREADY_CONNECTED`、`PROVIDER_AUTH_FAILED`、`PROVIDER_UNAVAILABLE`、`PROVIDER_RATE_LIMITED`、`CLOCK_SKEW`（`STATE_STALE`）、`PROVIDER_RESPONSE_INVALID`、`PROVIDER_DATA_INCOMPLETE`、`PROVIDER_IDENTITY_CHANGED`、`PROVIDER_REVIEW_REQUIRED`、`PROVIDER_PERMISSION_BLOCKED`、`CREDENTIAL_UNAVAILABLE`、`CREDENTIAL_STORE_FAILED`、`CREDENTIAL_DELETE_FAILED`，以及既有 payload/state/storage 错误。不返回原始 provider body、带签名 URL、认证 header 或原生诊断；request correlation 不能替代连接/同意身份。

Binance Spot 的余额 `available` / `reserved` 分别为原币 free / locked，`total` 为精确相加；非零余额形成未估值的现货持有量。订单身份包含 symbol 与 orderId，因为订单 ID 按交易对限定；缺失报价币种保持 unavailable。Testnet 密钥权限保持 UNVERIFIED，Live 使用独立密钥权限接口，账户 `canWithdraw` 不代表密钥提款权限。签名时间无效、采样过慢或服务端拒绝时间戳返回 `STATE_STALE / CLOCK_SKEW`；本次观察不更新。


Bitget Classic Spot 原生凭据 schema 精确包含三个敏感字段：API key、secret、passphrase。Demo 与 Live 是不可变的独立连接，共用固定 Bitget REST 主机；每个 Demo 私有请求携带 `paptrading: 1`，Live 请求省略该头。Demo 账户读取不受支持时明确返回 `PROVIDER_UNSUPPORTED`；禁止回退到 Live 或制造空账户成功观测。

Bitget 余额的 `reserved` 表示冻结资产。Balance 新增可选十进制字符串字段 `locked` 和 `restrictedAvailable`；其他提供方及旧持久化记录可保持不可用。分别保留可用、冻结、锁仓和受限可用资产。若展示由 available + frozen + locked 计算的资产总量/持仓数量，必须在账户限制中明确组成依据；提供方未说明受限可用是否重叠，因此不得另行相加。这不是组合权益或 FX 估值。缺失字段保持不可用，不替换为零。

OpenOrder 新增可选 `kind`（NORMAL / TPSL / PLAN）与十进制字符串 `triggerPrice`。Bitget 分别读取普通单、止盈止损当前单及待触发计划委托，按各接口分页契约完整读取后才发布账户快照。计划委托身份与普通订单身份分开。基础币数量、计价币名义金额、基础币已成交数量和计价币已成交金额保持区分。市价买单 size 为计价币金额，限价单和市价卖单 size 为基础币数量；计划单的 planType amount/total 决定数量/金额。币种或成交观测不可用时保持 null。这些只读观测不授权创建或执行触发订单。旧记录缺少这些新增可选字段时仍须可读。

PermissionReview 新增可选 `ipAllowList: string[] | null`：提供方返回合法白名单时，保留规范化、排序、去重后的 IP 地址；空数组表示明确未限制，null 表示不可用。该字段参与权限范围比较，即使状态仍为 RESTRICTED，地址变化也必须使先前确认失效。

Bitget authorities 与 IP 内省独立于 REST 读取成功决定凭据权限范围。未知权限码保持可见且为 UNVERIFIED；划转、提现、账户管理及不支持的非现货写权限，即使已确认未知权限也必须阻止连接确认。权限范围变化使先前确认失效。所有 Live 连接继续保持 DISARMED，执行为 BLOCKED。

---

### 41.4 受管模型网关载荷（S03 生命周期）

所有版本 1 输入对象拒绝未声明字段和 null。工作区身份必须匹配当前工作区；变更必须携带当前网关状态版本。载荷不得接收秘密、路径、可执行参数、端口、远程 URL 或进程 ID。宿主提供工作区存储之外的私有运行目录。

| Command | Payload | Success data |
|---|---|---|
| model.get_gateway | `{workspaceId: string}` | `GatewayState` |
| model.gateway | `{workspaceId: string, expectedStateVersion: string, action: "LAUNCH" \| "PROBE" \| "RESTART" \| "STOP"}` | `GatewayState` and opaque `stateVersion` |

~~~ts
interface GatewayState {
  workspaceId: string;
  stateVersion: string;
  pinnedVersion: "7.2.155";
  endpoint: "http://127.0.0.1:8317";
  status: "STOPPED" | "INSTALLING" | "STARTING" | "RUNNING" |
          "PORT_CONFLICT" | "UNAUTHORIZED" | "BACKOFF" | "FAILED" | "STOPPING";
  desiredRunning: boolean;
  installed: boolean;
  modelAvailable: boolean;
  discoveredModelCount: number;
  lastProbeAt: string | null;
  nextRetryAt: string | null;
  restartAttempts: number;
  errorCode: string | null;
  updatedAt: string;
}
~~~

`model-gateway` 聚合以 `workspaceId` 为 aggregate ID；snapshot/subscribe/replay 沿用 §41.2，投影为 `GatewayState`。新进程会话恢复非秘密配置，但将进程/探测观察视为未验证：STOPPED、无可用模型、发现模型数为零、无当前探测时间。新会话绝不信任持久化 PID 或端口监听者。时间为 UTC RFC 3339，计数为有界非负整数。installed 表示固定可执行文件摘要验证通过，不是文件存在。

LAUNCH 可先安装固定发布物，再启动并探测自己拥有的进程。STOP 取消退避并只停止自己的子进程；RESTART 先停止再启动；PROBE 仅检查自己的进程，不附着到已有端点。慢 I/O 前提交并发送过渡状态；I/O 不持有 Control Plane mutex。切换工作区使旧操作失效，迟到结果不能更新新工作区。发送下游秘密前验证进程所有权；限制请求和输出、禁止重定向/代理，并隐藏原始进程/网络诊断。

`/v1/models` 探测成功只表示 RUNNING，不表示 modelAvailable。生命周期票保持 modelAvailable=false，直到后续精确授权路由通过推理验证。端口冲突、未授权、停止、启动/探测失败及重启预算耗尽采用 MODEL_UNAVAILABLE 类别及具体脱敏 GATEWAY_* 原因码。崩溃自动恢复依次等待 1、2、4 秒，三次重启失败后停止，并展示下一重试时间。显式 STOP 或切换工作区取消待执行重试；持续稳定运行 60 秒后才可重置预算。普通模型/提供方故障不改变金融状态。

每次已提交状态变更在 §42 的 `model.gateway.changed` 事件中携带完整 `GatewayState`，聚合序号严格递增，事件与投影原子提交。无变化的轮询不重复发事件。Domain replay 与工作区/账户一样严格校验事件类型及聚合/载荷身份。

### 41.5 模型提供方连接 payload（S03 连接）

`model` 聚合以 `workspaceId` 为 aggregate ID，只包含非秘密的提供方健康、允许路由元数据和有界 setup-attempt 溯源。绝不包含 OAuth token、DeepSeek key、Keychain 字节、sidecar 路径、broker reference、原始响应 body 或 prompt 内容。新进程会话会把已保存的提供方配置标为 UNVERIFIED，直到新的路由验证成功。

| 命令 | Payload | 成功数据 |
|---|---|---|
| model.get | `{workspaceId: string}` | `ModelState` |
| model.login_chatgpt | `{workspaceId: string, expectedStateVersion: string, action: "LOGIN" \| "RELOGIN"}` | `ModelState` 和不透明 `stateVersion` |
| model.configure_deepseek | `{workspaceId: string, expectedStateVersion: string}` | `ModelState` 和不透明 `stateVersion` |
| model.verify_route | `{workspaceId: string, expectedStateVersion: string, provider: "CHATGPT" \| "DEEPSEEK", modelId: string, thinkingType: "disabled" \| "enabled" \| null}` | `ModelState` 和不透明 `stateVersion` |

`model.login_chatgpt` 在专用 auth directory 中启动固定 sidecar 的 `-codex-login` 流程，只观察有界退出/状态；TradeX 不读取或解析 OAuth 文件。`model.configure_deepseek` 只接受可信原生安全输入，将 key 存入仅供模型使用的 OS Keychain service，并在网关（重新）启动时将其渲染至私有 0600 sidecar config。取消或写入失败会保留旧 key 和状态。

`model.verify_route` 先认证自有 loopback 网关的 `/v1/models` 响应，再使用固定、无害且有界的 prompt 对同一精确 provider/model 路由进行测试推理。只有 GPT-5.6 系列 ID，或带显式 `thinking.type: disabled|enabled` 的 `deepseek-v4-flash` 才允许。目录命中而推理不成功仍为 UNVERIFIED。Setup attempt 只追加，保存真实 provider/model/mode、时间、结果、规范错误（`MODEL_UNAVAILABLE`、`OAUTH_EXPIRED`、`QUOTA_EXCEEDED`）和服务端明确给出的 quota window/cooldown；丢弃原始 body 和 prompt。

`model.provider.changed` 携带完整脱敏 `ModelState`；`model.provider_attempt.changed` 携带追加 attempt 后的同一状态。两者使用连续的 `model` 聚合序列，并在 replay 时严格校验聚合/载荷身份。这些变更绝不修改账户 capability、risk、arming 或 approval。只有验证通过的路由才能令 `modelAvailable` 或 onboarding Ready 为 true；默认选择和跨提供方 fallback consent 持久化在 model aggregate，入门和风险契约见 §41.6。

### 41.6 风险策略与入门 payload（S03 onboarding）

`risk` aggregate 使用 `workspaceId` 作为 aggregate ID，是入门进度和设置风险默认值的唯一权威。版本 1 增加以下精确命令：

| 命令 | Payload | 成功数据 |
|---|---|---|
| risk.get_policy | `{workspaceId: string}` | `RiskPolicyState` 及其不透明 `stateVersion` |
| risk.save_policy | `{workspaceId: string, expectedStateVersion: string, policy: RiskPolicyInput}` | 新 `RiskPolicyState`、递增的 `policyVersion` 以及 `risk.policy.changed` |
| onboarding.set_step | `{workspaceId: string, expectedStateVersion: string, step: 1 \| 2 \| 3 \| 4 \| 5}` | 带请求进度步骤的新 `RiskPolicyState` |
| onboarding.complete | `{workspaceId: string, expectedStateVersion: string}` | `onboardingCompleted: true` 的新 `RiskPolicyState` |

~~~ts
interface RiskPolicyInput {
  maxOrderNotional: string | null;
  maxSingleInstrumentExposurePercent: string | null;
  maxDailyTradedNotional: string | null;
  maxDailyRealizedLoss: string | null;
  staleQuoteThresholdSeconds: number;
  marketOrdersEnabled: boolean;
  liveInactivityTimeoutMinutes: number;
}
interface RiskPolicy {
  maxOrderNotional: string | null;
  maxSingleInstrumentExposurePercent: string | null;
  maxDailyTradedNotional: string | null;
  maxDailyRealizedLoss: string | null;
  staleQuoteThresholdSeconds: number;
  marketOrdersEnabled: boolean;
  liveInactivityTimeoutMinutes: number;
}
interface RiskPolicyState {
  workspaceId: string;
  stateVersion: string;
  policyVersion: number;
  configured: boolean;
  onboardingStep: 1 | 2 | 3 | 4 | 5;
  onboardingCompleted: boolean;
  policy: RiskPolicy;
  hardRules: Array<{id: string; description: string}>;
  updatedAt: string;
}
~~~

金额和敞口使用十进制字符串，绝不使用 JSON number 或浮点。每个 `RiskPolicyInput` 字段在 wire 上都必须存在；四个金额/敞口额度在用户选择前可显式设为 `null`。新策略默认报价过期阈值 3 秒、市价单 `false`、Live inactivity timeout 20 分钟。时间边界为 1–86,400 秒和 1–1,440 分钟；敞口最多 100%；空值、零值、科学计数法、格式错误或超长小数返回 `POLICY_ERROR / RISK_POLICY_INVALID`。`hardRules` 由后端拥有且只读：Live 默认 DISARMED、仍需单独 approval、过期数据阻断 Live、Agent 不能修改策略。本设置记录不实现完整 S21 risk engine。

所有变更都要求当前 workspace 和精确的 `stateVersion`；陈旧游标返回 `STATE_STALE / STATE_VERSION_CONFLICT` 且不修改状态。进度只能前进或后退一步；越级返回 `POLICY_ERROR / ONBOARDING_STEP_INVALID`。步骤 5 和完成都要求已配置风险默认值及 §41.5 的当前已验证默认模型路由。完成还要检查每个 Live 账户为 `DISARMED`；任何入门命令都不会 arm 账户或启用 Send/Live execution。模型会话重置会使已完成设置失效并回到 Model（步骤 3）。`risk.policy.changed` 与 risk projection/outbox 在同一事务提交，`risk` snapshot/subscribe/replay 遵循 §41.2 的工作区规则和连续序列。新工作区在 storage schema version 5 迁移时初始化风险表；已识别的旧工作区迁移前先备份。

脱敏 remediation code 为 `RISK_POLICY_INVALID`、`RISK_POLICY_NOT_CONFIGURED`、`ONBOARDING_STEP_INVALID` 和 `ONBOARDING_BLOCKED`；不返回原始存储、账户或模型诊断。前端在拿到权威账户 projection 前必须把未知/加载中的账户状态视为不可用，并在 Ready 展示当前 provider/model/fallback、currency 与 Live arming 事实。这些命令仅供 renderer 入门设置；Agent/Thread 执行没有修改策略的能力。

---

### 41.7 上下文目录载荷（S05）

上下文目录是只读的工作区查询，为 Composer 暴露非秘密 canonical account refs，并为未来上下文类别返回明确的空状态；绝不返回凭据、提供方响应或权限秘密。

| Command | Payload | Success data |
|---|---|---|
| context.catalog | `{workspaceId: string}` | `ContextCatalog`，包含有界账户条目和明确空状态 |

~~~ts
interface ContextCatalog {
  entries: ContextCatalogEntry[];
  emptyStates: ContextCatalogEmptyState[];
}
interface ContextCatalogEntry {
  contextRef: { kind: "account"; id: string; hash: string };
  label: string;
  providerId?: string;
  environment?: string;
  readOnly: boolean;
  available: boolean;
  availabilityReason?: string;
}
interface ContextCatalogEmptyState {
  kind: "instrument" | "account" | "strategy" | "backtest" | "artifact";
  availabilityReason: string;
}
~~~

账户 hash 使用 `sha256:<64 个小写十六进制字符>`，输入为 canonical 非秘密元组 `workspaceId\0connectionId\0providerId\0environment\0createdAt`。label 与凭据不参与 hash。DISCONNECTED、缺少凭据和清理待处理连接仍可见但 `available: false`；未来 instrument、strategy、backtest、artifact 类别返回明确空状态而非 fixture。`thread.create` 和 `turn.start` 只接受有界的受支持 refs；account ref 必须匹配活动工作区目录及派生 hash。重复、格式错误、未知或跨工作区 ref 在持久化或 runtime 前失败。`turn.start` 省略 `attachedContexts` 时为兼容性沿用已保存 Thread refs，显式空数组会清除本轮上下文，显式 `null` 无效。

可用的附加 account 上下文会为 Ask、Research 和 Backtest 增加只读 `account_read` 能力，但不会满足 Trade 所需的独立执行账户条件。

### 41.8 Typed research tool/result 边界（S05）

`CapabilityDecision.researchTools` 是 Control Plane 从 `allowedTools` 派生的 data-plane registry，只能包含 `public_market_read`、`account_read` 和 `historical_simulation`；`paper_demo_testnet_execution`、`live_order_proposal` 以及 risk、approval、arming、keychain 和 Order Gateway 命令永远不会成为 registry entry。总能力列表仍是模式/环境的权威决策：Trade Paper/Demo/Testnet 是唯一非 Live execution 路径，Trade Live 始终停在 proposal-only。

| Command | Payload | Success data |
|---|---|---|
| research.run | `ResearchToolRequest` | `ResearchToolResult` |

~~~ts
type ResearchToolId = "public_market_read" | "account_read" | "historical_simulation";
type ResearchResultState = "AVAILABLE" | "DEGRADED" | "UNAVAILABLE" | "BLOCKED_EXTERNAL" | "FAILED";
type ResearchFocus = "GENERAL" | "EQUITY" | "CRYPTO_SPOT";
type ResearchSpotVenueId = "BINANCE" | "BITGET";
type ResearchFreshness = "HEALTHY" | "STALE" | "UNAVAILABLE";
type ResearchQuality = "VERIFIED" | "DEGRADED" | "UNKNOWN" | "UNAVAILABLE";
interface ResearchToolDefinition { id: ResearchToolId; label: string; readOnly: boolean; description: string; }
interface ResearchFinding { title: string; detail: string; } // title <=120，detail <=512
interface ResearchScenario { title: string; detail: string; } // title <=120，detail <=512
interface ResearchProvenance {
  sourceId: string; provider: string; status: DataSourceStatus;
  providerTimestamp?: string; receivedTimestamp: string;
  freshness: ResearchFreshness; quality: ResearchQuality; limitation?: string;
}
interface ResearchSpotVenue {
  venue: ResearchSpotVenueId; state: ResearchResultState; selected: boolean;
  bid?: string; ask?: string; spread?: string; depth?: string; quoteAge?: string;
  provenance: ResearchProvenance; limitation?: string;
}
interface ResearchToolPayload {
  state: ResearchResultState; reason: string; focus?: ResearchFocus;
  conclusion?: string; findings: ResearchFinding[]; scenarios: ResearchScenario[];
  evidence: ResearchProvenance[]; limitations: string[]; instrumentRefs: string[];
  artifactRefs: string[]; marketSnapshotRefs?: string[]; datasetRefs?: string[];
  orderRefs?: string[]; spotVenues: ResearchSpotVenue[]; fixtureLabel?: string;
}
interface ResearchToolRequest {
  workspaceId: string; agentMode: AgentMode; executionContext: ExecutionContext;
  accountId?: string; focus?: ResearchFocus; attachedContexts: ThreadContextRef[];
  toolId: ResearchToolId; query: string;
}
interface ResearchToolInvocation { toolId: ResearchToolId; focus?: ResearchFocus; query: string; }
interface ResearchToolResult {
  resultId: string; toolId: ResearchToolId; sourceId: string; accountId?: string;
  requestHash: string; marker: string; contextRefs: ThreadContextRef[];
  payload: ResearchToolPayload;
}
~~~

`research.run` 是只读命令，在拥有 market/account/history 的 slice 接入 provider 前返回受 source gate 约束的 typed payload。focus、tool、mode、execution context、account、attached refs 和 query 都进入 request hash；query 文本有界并只参与 hash，不会回显到 result。旧的仅含 `state/reason` 的 S05 payload 仍可按默认字段解码。`turn.start` 只能成对提交 `ResearchToolInvocation` 和 result；Control Plane 从 Turn 输入重建完整 request，再次运行同一 registry 并逐字段比较，确认后才写入 `research_result` item 或启动 runtime。缺失、错配或篡改 pair 返回 `RESEARCH_RESULT_INVALID`，且不修改 projection。持久化 item 与非秘密 source/context refs、marker 一起保留完整的有界 typed result；runtime 只接收清洗后的 marker。broker credential、model secret 和 direct external LLM endpoint 不会跨越该边界。

payload 的 `scenarios` 和 `artifactRefs` 是有界 typed 字段；`artifactRefs` 只能复制请求中已经附加的 canonical artifact context ID。`spotVenues` 仅允许 Binance 和 Bitget 条目，每条都带 typed state 和完整 provenance；来源不可用时 bid/ask/spread/depth/quoteAge 为可空字段并序列化为 `null`，不得用零替代。集成 bridge 只有在 integration feature 和显式 fixture 环境变量同时启用时才可生成 synthetic 股票情景或 venue 条目，结果必须显示 `fixtureLabel`；这不构成 provider entitlement 或实时事实。Trade 卡片只能提供只读 proposal 入口，不能调用 order、approval、arming、Gateway 或 live-risk 命令。

### 41.9 数据源目录与探测载荷（S06）

数据源目录是只读的策略投影。它为 OD-001–006 记录所选来源、能力覆盖、时效、entitlement、保留、再分发/商业/辖区限制、官方/条款 URL 及来源审阅日期。它不表示已连接的券商账户拥有市场数据 entitlement。`data.source.probe` 是有界只读操作：公开 HTTP 响应只能证明可达性；Alpaca credentialed source 在用户管理的 entitlement 被单独验证前始终为 `BLOCKED_EXTERNAL`。

| Command | Payload | Success data |
|---|---|---|
| data.source.catalog | `{workspaceId: string}` | `DataSourceCatalog` |
| data.source.probe | `{workspaceId: string, sourceId: string, expectedStateVersion: string}` | `DataSourceCatalog`，以探测后的条目替换目标项 |

~~~ts
type DataSourceStatus = "AVAILABLE" | "UNAVAILABLE" | "BLOCKED_EXTERNAL" | "UNVERIFIED";
type DataSourceProbeKind = "PUBLIC_METADATA" | "CREDENTIALED_METADATA";
interface DataSourceEntry {
  sourceId: string; provider: string; capabilities: string[];
  coverage: string; latency: string; entitlement: string;
  retention: string; redistribution: string; commercialUse: string;
  jurisdictions: string; officialUrl: string; termsUrl: string;
  reviewedAt: string; checkedAt?: string; observedAt?: string;
  probeKind: DataSourceProbeKind; status: DataSourceStatus;
  configured: boolean; verifiedAt?: string; availabilityReason: string;
}
interface DataSourceCatalog { workspaceId: string; stateVersion: string; sources: DataSourceEntry[]; }
interface DataSourceQuery { workspaceId: string; }
interface DataSourceProbe { workspaceId: string; sourceId: string; expectedStateVersion: string; }
~~~

初始策略将 Alpaca Market Data 映射到 OD-001/002，将 SEC EDGAR 映射到 OD-003/004 的基本面与 filings，将 Alpaca Calendar/Corporate Actions 映射到 OD-005，将 ECB EXR/SDMX 信息性参考汇率映射到 OD-006。通用新闻、完整跨市场事件、交易级盘中 FX 和稳定币 parity 保持 `BLOCKED_EXTERNAL`。公开 SEC/ECB 探测保留来源 URL、checked/observed 时间和脱敏 HTTP 结果，但绝不保留响应正文或凭据。未知 source ID 返回 `DATA_SOURCE_UNKNOWN`；过期 workspace 游标返回 `STATE_STALE / STATE_VERSION_CONFLICT`；两个命令都不写 SQLite、不改变 account/model/risk/thread 版本，也不启用 Live。探测观察按 workspace/source 保存在 Control Plane 进程内存中，同一进程的 renderer reload/remount 会保留；进程重启后恢复静态 `UNVERIFIED`/`BLOCKED_EXTERNAL` 并要求重新探测。source 状态不是 `AVAILABLE` 时，typed research 必须返回 sanitized unavailable，直到所属数据切片解除 gate。

### 41.10 行情目录、详情与历史载荷（S07）

S07 增加两个只读行情命令。它们在执行任何 provider adapter 之前解析规范化 instrument 身份，绝不把券商账户连接当作行情 entitlement。

| Command | Payload | 成功 data |
|---|---|---|
| market.catalog | `{workspaceId: string, query?: string, tier?: MarketTier}` | `MarketCatalog` |
| market.get | `{workspaceId: string, instrumentId: string, tier: MarketTier}` | `MarketDetail` |

~~~ts
type MarketTier = "CENSUS" | "WARM" | "HOT" | "COLD";
type MarketDataStatus = "AVAILABLE" | "UNAVAILABLE" | "BLOCKED_EXTERNAL" | "UNVERIFIED";
type MarketEntitlement = "REALTIME" | "DELAYED" | "UNKNOWN";
type MarketFreshness = "HEALTHY" | "STALE" | "CLOCK_UNCERTAIN";
interface InstrumentProviderMapping { providerId: string; providerSymbol: string; }
interface Instrument {
  instrumentId: string; assetClass: "EQUITY" | "CRYPTO_SPOT"; symbol: string;
  base?: string; quote?: string; exchange?: string; currency: string;
  displayName: string; providers: InstrumentProviderMapping[];
}
interface MarketSnapshotProvenance {
  marketSnapshotId: string; source: string; venue?: string;
  providerTimestamp: string; receivedTimestamp: string;
  entitlement: MarketEntitlement; freshness: MarketFreshness;
}
interface MarketSnapshot {
  instrumentId: string; provenance: MarketSnapshotProvenance;
  lastPrice?: string; bid?: string; ask?: string;
}
interface MarketCatalog {
  workspaceId: string; query: string; tier: MarketTier; sourceId?: string;
  status: MarketDataStatus; availabilityReason: string; instruments: Instrument[];
}
interface MarketDetail {
  workspaceId: string; instrument: Instrument; tier: MarketTier; sourceId?: string;
  status: MarketDataStatus; availabilityReason: string; snapshot?: MarketSnapshot;
}
~~~

`market.catalog` 省略 `tier` 时默认为 `CENSUS`；两个 payload 都拒绝未知字段、控制字符和超长值。Instrument ID 使用规范形式（`equity:US:AAPL` 或 `crypto:BTC/USDT:spot`），provider symbol 只存在于 adapter 映射中。Equity tier 选择 OD-001（Census/Warm/Hot）或 OD-002（Cold）。只有当所有匹配结果都解析到同一个 source 时 catalog 才返回 source ID；混合 equity/crypto 结果省略它，避免为 crypto 行显示 Alpaca source。Crypto 映射为未来 Binance/Bitget adapter 保留，但 S06 授权目录目前没有选定 crypto source，因此 `market.get` 返回 `UNAVAILABLE`、不返回 source ID 和 snapshot。source 状态不是 `AVAILABLE` 时返回脱敏的状态/原因，绝不制造报价。

行情历史是内部数据层操作，不是 renderer mutation。DuckDB 与 workspace SQLite 并列存放于 `market.duckdb`，保存按规范 instrument、分钟和 source 键控的有界 `ohlcv_1m` 行。只有规范 instrument 在 registry 中、source ID 映射到 equity 的 OD-001/OD-002 adapter 且匹配条目为 `AVAILABLE` 时才接收行；crypto 行和 `REALTIME` entitlement 在历史写入边界直接拒绝。OHLCV 是精确的非负 decimal 字符串，时间是 RFC 3339（interval start 必须落在整分钟），venue/source 是有界标识符。source 缺失、阻断或不匹配时在任何写入前返回 `MARKET_HISTORY_UNAVAILABLE`；不改变 SQLite 领域投影或 state version。重开 workspace 时按需创建表，不导入 synthetic 或 blocked history。

### 41.11 Watchlist library 与 membership payload（S07）

Watchlists 是 workspace-scoped 的本地 collection projection。它们是有序成员关系的 SQLite 权威状态，但不是 financial-authority aggregate。V1 暴露以下精确命令：

| Command | Payload | 成功 data |
|---|---|---|
| watchlist.list | `{workspaceId: string}` | `Watchlists` |
| watchlist.create | `{workspaceId: string, name: string}` | `Watchlist` |
| watchlist.rename | `{workspaceId: string, watchlistId: string, name: string, expectedStateVersion: string}` | `Watchlist` |
| watchlist.delete | `{workspaceId: string, watchlistId: string, expectedStateVersion: string}` | `Watchlists` |
| watchlist.add | `{workspaceId: string, watchlistId: string, instrumentId: string, expectedStateVersion: string}` | `Watchlist` |
| watchlist.remove | `{workspaceId: string, watchlistId: string, instrumentId: string, expectedStateVersion: string}` | `Watchlist` |

~~~ts
interface WatchlistItem { instrumentId: string; }
interface Watchlist {
  watchlistId: string; workspaceId: string; name: string;
  stateVersion: string; items: WatchlistItem[];
}
interface Watchlists {
  workspaceId: string; stateVersion: string; watchlists: Watchlist[];
}
~~~

每个 payload 都拒绝未声明字段、控制字符和超长值。Name 会 trim、有界并在 workspace 内大小写不敏感地唯一；membership 只使用 canonical instrument ID 并保留插入顺序。mutation 必须携带该 list 精确的 `expectedStateVersion`；陈旧游标返回 `STATE_STALE / STATE_VERSION_CONFLICT` 且不修改状态。对已存在/不存在成员的 add/remove 是幂等的。上限为每 workspace 128 个 list、每 list 256 个成员。脱敏错误包括 `WATCHLIST_NAME_CONFLICT`、`WATCHLIST_NOT_FOUND`、`MARKET_INSTRUMENT_INVALID`、`MARKET_INSTRUMENT_NOT_FOUND`、`STATE_VERSION_CONFLICT` 和 `IPC_PAYLOAD_INVALID`。

每个 mutation 在一个 immediate SQLite transaction 中同时提交 projection 与表元数据。它绝不存储 credential 或 quote，也不改变 account、model、risk 或 thread 版本。mutation 和 workspace reopen 后，`watchlist.list` 是权威读取。V1.0 中 Watchlists 有意不使用 `DomainProjection`、outbox 或 `domain.snapshot`/`domain.subscribe`；§42 的 event/replay 适用于 financial 和 agent-authority aggregate。如果未来需要跨窗口实时同步，应先将该 collection 提升为 evented aggregate，并补充 replay/snapshot 契约后再启用。

### 41.12 Trusted time payload（S08）

`time.status` 与 `time.revalidate` 是 workspace-scoped、只读的运行时查询。两者都接收 `{workspaceId: string}`，返回 `TimeStatus`：

~~~ts
type TimeConfidence = "TRUSTED" | "CLOCK_UNCERTAIN" | "STALE";
interface TimeStatus {
  workspaceId: string; confidence: TimeConfidence; wallClock: string;
  monotonicMs: number; providerOffsetMs?: number; observedAt: string;
  reason: string; remediation: { id: string; label: string };
}
~~~

Rust Control Plane 使用文档化的 2,000 ms 容差比较 UTC wall-clock 与 monotonic elapsed，并将 provider/server offset 限制在 5,000 ms。workspace open、进程重启和 resume 都会重置 trust；首次 status 在显式执行 `time.revalidate` 建立有效基准前保持 `CLOCK_UNCERTAIN`。wall-clock 重大偏离、monotonic 回退或超界 provider offset 返回 `CLOCK_UNCERTAIN` 或 `STALE`，并以 `CLOCK_SKEW`/`time_revalidate` 提供 remediation。时间读数只存在进程内，绝不写入 SQLite、DomainProjection、outbox、account、risk、approval 或 thread 状态。未来 freshness/TTL/approval/dispatch 路径必须消费 `TimeService::require_trusted`；renderer 不能传入时钟覆盖。

### 41.13 Market state 与 corporate-action payload（S08）

`market.get` 仍是只读 canonical-instrument query，并携带 typed 的 market-session 与 corporate-action metadata：

~~~ts
type MarketSession = "OPEN" | "CLOSED" | "EXTENDED_HOURS" | "HALTED" | "MAINTENANCE" | "SUSPENDED" | "DEGRADED" | "UNKNOWN";
type AdjustmentStatus = "ADJUSTED" | "UNADJUSTED" | "UNKNOWN" | "UNAVAILABLE";
type CorporateActionType = "SPLIT" | "DIVIDEND" | "SYMBOL_CHANGE" | "DELISTING";
interface MarketState {
  session: MarketSession; venue: string; sourceId?: string;
  sourceStatus: MarketDataStatus; nextOpen?: string; nextClose?: string;
  calendarVersion?: string; providerTime?: string; observedAt: string;
  timeConfidence: TimeConfidence; reason: string;
}
interface CorporateAction {
  actionId: string; instrumentId: string; actionType: CorporateActionType;
  effectiveAt: string; announcedAt?: string; sourceId?: string;
  description: string; adjustmentStatus: AdjustmentStatus;
}
interface MarketDetail {
  /* S07 的既有字段保持不变 */
  marketState: MarketState; corporateActions: CorporateAction[];
  adjustmentStatus: AdjustmentStatus;
}
~~~

Equity 状态和 action 数据使用 OD-005 calendar/corporate-action gate；在选择授权 source 前，crypto venue state 保持 `UNKNOWN`/`UNAVAILABLE`。任何 source 状态都不能展示成 `OPEN`，任何 fixture 都不能把 DuckDB history 标为已调整。有限 observation seam 为 contract tests 提供确定性的 regular/holiday/half-day/extended/halt 以及 crypto maintenance/suspension/degraded fixture；blocked 或 unavailable source 继续显示 `UNKNOWN`/`UNAVAILABLE`。Payload 有界、按 canonical instrument 归属，拒绝未知/控制字符字段和无效 RFC 3339 timestamp，并拒绝重复 action ID；corporate-action record 按 effective time 与 action ID 规范排序。共享 `market_execution_eligibility` seam 先消费 `TimeService::require_trusted`，再要求 source 可用且 adjustment status 已知，才允许 `OPEN`/`EXTENDED_HOURS`；对 source 阻断、CLOSED/HALTED 返回确定性的 `MARKET_CLOSED` 或 `INSTRUMENT_HALTED` remediation；本节不实现 order、approval、risk、reservation 或 gateway command。

### 41.14 只读组合载荷（S09）

版本 1 增加一个按 workspace 作用域的只读操作：

| 命令 | 载荷 | 成功数据 | 写入/事件行为 |
|---|---|---|---|
| `portfolio.get` | `{workspaceId}` | `PortfolioSnapshot` | 无；不写入 SQLite、outbox、account/model/risk/thread 版本或凭据，也不产生 domain event |

`PortfolioSnapshot` 返回持久化 workspace `baseCurrency`、TimeService `observedAt`、typed `status`（`AVAILABLE`、`DEGRADED`、`UNAVAILABLE` 或 `BLOCKED_EXTERNAL`）、清理后的 `availabilityReason`、`PortfolioTotals`、有界的账户/持仓/订单行、可选 `fills`、有界 `fxRoutes`，以及本只读切片中始终 `eligible=false` 的 `liveRisk`。请求拒绝额外字段、控制字符、外部 workspace 以及超出 schema 限制的集合。

每条账户/持仓/订单/成交观察都携带 provider connection identity、该观察的 `observedAt` 和账户 `health`。适配器在账户投影进入组合逻辑前把 provider symbol 解析为 canonical `instrumentId`；未知映射使用明确的 `asset: "UNAVAILABLE"` sentinel。只有 provider 提供 venue 证据时才返回可选 `venue`。余额 asset 可以保留为明确的原生资产/币种 identity。

`PortfolioValue` 分开保存 `nativeValue/nativeCurrency`、`accountValue/accountCurrency` 与 `workspaceValue/workspaceCurrency`。每个归一化值都必须带有 `FxProvenance`，包含 source、pair path、可选 rate、provider timestamp、TradeX received timestamp、freshness、quality 和可选 depeg warning。缺失币种、汇率、成交、成本基础、已实现/未实现 P&L 或其他 provider 字段必须为 `null`/unavailable，不能填零。聚合采用 fail-closed：任何参与的原生值缺少可信 workspace 转换时，依赖的 workspace 总额不可用，snapshot status 保持 degraded/blocked。provider 不提供成交时 `fillsCount` 为可选字段。组合载荷不授予 arming、approval、reservation 或 gateway 权限；后续 risk consumer 必须重新验证全部账户、市场、时间和 FX gate。

### 41.15 只读自然语言筛选器载荷（S11）

版本 1 增加受 source gate 约束的 `market.screen` 操作、按 workspace 作用域的已审阅定义 projection，以及 canonical 附加操作：

| 命令 | 载荷 | 成功数据 | 写入/事件行为 |
|---|---|---|---|
| `market.screen` | `{workspaceId, operation, naturalLanguage, focus?, filterSpec?, rankSpec?, revision?, limit?}` | `ScreenerResult` | 无；不调用 provider、不写 SQLite/DuckDB、不产生 domain event，也不授予 execution authority |
| `screener.list` | `{workspaceId}` | `ScreenerLibrary` | 只读查询 SQLite projection；不产生 domain event、不调用 provider、不授予 execution authority |
| `screener.save` / `screener.update` | `{workspaceId, name, definition, state, expectedStateVersion[, screenerId]}` | `ScreenerLibrary` | 仅原子写入 workspace SQLite projection；不写 outbox/domain event，也不修改 DuckDB/provider/account/risk/approval/arming/Gateway/credentials |
| `screener.attach` | `{workspaceId, revision, selectedInstrumentIds[]}` | `ScreenerAttachment` | 不持久化、不调用 provider；只返回所选 canonical `ThreadContextRef` |

`operation` 为 `PARSE` 或 `RUN`。`PARSE` 把有界自然语言转换为 typed `FilterSpec`、`RankSpec` 和确定性的 `sha256:` revision。Renderer 必须在 `RUN` 前显示已解析条件；编辑后必须生成新 revision。`RUN` 拒绝缺失或过期 revision，并返回带有界候选行、精确 canonical instrument ID、feature 值、source ID、provider/received timestamp、freshness、quality 和 limitation 的 `COMPLETED`、`EMPTY`、`BLOCKED_EXTERNAL` 或 `FAILED` 结果。`RUN` 不得静默丢弃不支持的 predicate：typed result 必须以 `SCREENER_FILTER_UNSUPPORTED` 失败。

首个实现支持 US equities、US large-cap technology 和 crypto spot universe，支持 revenue growth、estimate revision、RSI、price change predicate，以及 quality/revision-strength/momentum 排序。真实 provider adapter 仍受 OD-001/OD-003 source gate 约束，在 entitlement 和 adapter 配置完成前返回 `BLOCKED_EXTERNAL`。仅 integration 使用的 `SYNTHETIC_SCREENER_FIXTURE` 是确定性的，不能证明 provider entitlement、quote authority 或 Live execution。

`ScreenerLibrary` 只在当前 workspace 保存有界的已审阅定义（`naturalLanguage`、`FilterSpec`、`RankSpec`、`revision`、`limit`、状态和时间戳）。每个 workspace 内名称唯一；未知 workspace、重名、越界、revision 不匹配和陈旧 `expectedStateVersion` 都 fail closed。重开只恢复输入，不恢复 Live authority 或 provider payload；编辑生成新 revision 并使旧结果失效。

`screener.attach` 只接受用户明确选择的 canonical instrument ID，重新验证它们仍属于当前 market catalog，并返回 workspace-bound `ThreadContextRef`。Renderer 可以把这些 ref 传给 `thread.create.linkedContexts` 或下一次 `turn.start`，两条路径都会重新验证。未选候选、完整 universe、provider payload、credentials 和自然语言上下文不会被附加，也不会自动启动 Turn。

### 41.16 研究产物载荷（S12）

版本 1 增加按 workspace 作用域的、对金融状态只读的产物 projection，以及显式的本地导出：

| 命令 | 载荷 | 成功数据 | 写入/事件行为 |
|---|---|---|---|
| `artifact.save` | `{workspaceId, threadId, turnId, itemId, kind, title}` | `Artifact` | 仅在 SQLite 写入有界脱敏 projection；不写 outbox/domain event，不修改 account/model/risk/approval/arming/reservation/Gateway/credentials |
| `artifact.list` | `{workspaceId}` | `ArtifactLibrary` | 仅按 workspace 查询 SQLite projection |
| `artifact.get` | `{workspaceId, artifactId}` | `Artifact` | 仅按 workspace 查询 projection；保存的 Turn snapshot 是权威来源 |
| `artifact.export` | `{workspaceId, artifactId, fileName?, destinationPath?}` | `ArtifactExportResult` | 在 workspace `exports/` 目录或原生选择的本地路径执行显式本地原子 JSON 导出；不上传云端、不生成分享链接 |

`artifact.save` 会重新读取持久化 Thread，并要求当前 workspace 中的 Turn 和 Item 均已完成。此切片生成的 `Artifact` 不可变，包含有界文本/typed research 字段和 `ArtifactProvenance`：workspace/Thread/Turn/Item ID、不可变 `TurnSnapshot`、追加式 provider attempts、research tool/result/source provenance，以及可选的 market snapshot、dataset 和 order 引用。每个产物使用版本 `1`、规范 `sha256:` 内容 hash 和 opaque ID；重复保存会创建新的 identity。

当 typed research producer 提供这些信息时，`ResearchToolResult.payload` 会携带有界的 `marketSnapshotRefs`、`datasetRefs` 和 `orderRefs`；`artifact.save` 将 producer 自有值复制到对应 provenance 字段。Renderer 的 context reference 不会被重新解释为 market snapshot、dataset 或 order identity。

Projection 拒绝未知或跨 workspace 引用、不支持的 kind、超限集合、控制字符和敏感 marker。它不会保存 broker credentials、model keys、Keychain bytes、Authorization header、原始 provider response 或完整账户/订单 payload。`artifact.export` 写入包含 schema version、artifact/version/hash、导出时间和 provenance 引用的 manifest，以及脱敏后的 artifact JSON；路径穿越、符号链接目标、已存在文件、脱敏失败和部分写入都会被拒绝，返回的 manifest hash 与 content hash 用于后续完整性检查。产物操作不授予执行权限，也不改变金融状态。

### 41.17 Local Paper payload（S16）

Local Paper 使用版本 1 envelope，且始终是按 workspace 作用域的 TradeX projection。它无 credential，不调用 provider adapter、Keychain/native credential input、network、model gateway、approval authority、arming service、reservation service、broker gateway 或 reconciliation service。

| 命令 | 载荷 | 成功 data | 写入/事件行为 |
|---|---|---|---|
| `paper.account.ensure` | `{workspaceId}` | 内置 `AccountConnection`，包含 `providerId: "local-paper"`、`environment: "LOCAL"`、`TRADEX_SIMULATION` 与 `SIMULATION_ONLY` eligibility | 幂等创建 Local Paper account 和初始 SQLite projection；不读取 credential、不做 provider probe |
| `paper.get` | `{workspaceId}` | `LocalPaperState` | 读取权威 SQLite；重建 orders、fills、events，校验确定性 ledger，不产生 event |
| `paper.order.submit` | `{workspaceId, proposalId, expectedProposalStateVersion, idempotencyKey}` | `PaperOrderResult`，包含 `LocalPaperOrder`、可选 `LocalPaperFill`、确定性 `LocalPaperQuote` 和更新后的 `LocalPaperState` | 只消费当前已选择、绑定 workspace 且为 `NEEDS_APPROVAL` 的 Local Paper proposal；在一个 immediate transaction 中持久化 order/fill/event/projection；同一 idempotency key 只重放结果、不产生修改 |
| `paper.order.cancel` | `{workspaceId, orderId, expectedStateVersion, idempotencyKey}` | `PaperOrderResult` | 只取消 open Local Paper order，释放模拟现金 reservation，保留 fills，并在同一事务持久化一次 cancellation event；重复取消只重放、不产生修改 |
| `paper.quote.refresh` | `{workspaceId, expectedStateVersion}` | `LocalPaperState` | 仅 Trade surface 可调用，刷新有界确定性 quote 时间；有 open order 或 Agent consumer 时拒绝 |
| `paper.scenario.set` | `{workspaceId, expectedStateVersion, profile}` | `LocalPaperState` | 仅 Trade surface 可在有界 S16 scenario 中切换；有 open order、policy 字段被篡改、版本过期、未知字段或 Agent consumer 时拒绝 |

`LocalPaperState` 保留 `local-paper` / `LOCAL` / `TRADEX_SIMULATION` identity、确定性 scenario 和 quote provenance、规范化现金/reservation/position/P&L projection，以及有序 event 列表（`ACCEPTED`、`PARTIALLY_FILLED`、`FILLED`、`REJECTED`、`CANCELLED`、`SCENARIO_CHANGED`、`QUOTE_REFRESHED`）。Storage boundary 拒绝外部 workspace/proposal、格式错误或未知字段、过期版本、已 invalidated proposal、被篡改的 fill/cash/position/P&L projection，以及任何无法从 canonical fills 与 open-order reservation 重新计算的状态，返回 `WORKSPACE_INTEGRITY_FAILED` 且不产生部分修改。重新打开同一 workspace 后，account、profile、orders、fills、cash、positions、open orders、P&L、events 和只读 portfolio aggregate 必须一致。

Local Paper result 是 simulation observation，不是 provider order ID、broker acknowledgement、reconciliation truth、approval state、arming state、Live readiness 或 Live execution authority。这些命令不发布 provider 或 Live domain event；内嵌的 Local Paper event list 是权威模拟时间线。

### 41.18 Alpaca Paper 提交与恢复（S17 #57）

这些版本 1 命令仅供主 UI consumer 使用现有 `alpaca` / `PAPER` 连接和 Keychain 凭据。Provider transport 将 host 固定为 `https://paper-api.alpaca.markets`，只允许有界的必需路径与方法；renderer 不能提供 host、symbol、远端 account ID、credential 或 authority 字段。

| 命令 | 载荷 | 成功 data | 写入/事件行为 |
|---|---|---|---|
| `alpaca.paper.order.submit` | `{workspaceId, connectionId, expectedConnectionStateVersion, proposalId, expectedProposalStateVersion, proposalHash, idempotencyKey, confirmedPaperOrder}` | `AlpacaPaperOrderAttempt` | 重新读取并将不可变 Alpaca Paper Proposal 绑定到当前连接/账户；provider I/O 前持久化一次 attempt 和稳定 `clientOrderId`；重复提交返回已保存 attempt，不再 POST |
| `alpaca.paper.order.attempt.get` | `{workspaceId, proposalId}` | `{attempt?: AlpacaPaperOrderAttempt}` | 仅按 workspace 读取 |
| `alpaca.paper.order.reconcile` | `{workspaceId, connectionId, expectedConnectionStateVersion, proposalId}` | `AlpacaPaperOrderAttempt` | 仅 `UNKNOWN_RECONCILING` attempt 可对账；按已保存 client order ID 查询前会重新核对 provider account identity；空查询或失败仍保持 unknown，绝不因此再次 POST |

`AlpacaPaperOrderAttempt` 是带版本的 workspace projection，并附有追加式状态事件；状态为 `SUBMITTING`、`ACKNOWLEDGED`、`UNKNOWN_RECONCILING` 或 `REJECTED`。重启会将未解决的 `SUBMITTING` 转为 `UNKNOWN_RECONCILING`；provider acknowledgement 与 fill evidence 保持区分。载荷经过 schema 校验，拒绝未知字段，且仅主 Trade surface 可提交或对账。Provider job 在 POST 前重新核对返回的 account identity、账户交易状态、资产类别/状态/可交易/可 fractional 能力、精确 Proposal identity 及受支持的订单组合。Fractional equity `qty` 与 `notional` 仅支持 Market/Day；当前契约无法验证的组合 fail closed。

Attempt 状态变化通过 aggregate type `alpaca-paper-order-attempt` 发布 `alpaca.paper.order.attempt.changed`，包含稳定的 attempt identity 和单调递增序号。秘密、Authorization header 和原始 provider payload 不进入 attempt projection 或 event。订单列表、fills、撤单与私有流恢复属于后续 S17 子票；这些命令不增加 Live authority，也不改变 Local Paper 行为。

## 42. Backend-to-Frontend Event Surface

代表性 events：

```text
runtime.health.changed
model.gateway.changed
model.provider_attempt.changed
thread.updated
turn.started
turn.item.started
turn.item.delta
turn.item.completed
turn.failed
market.snapshot.updated
account.health.changed
account.arming.changed
risk.policy.changed
trade.proposal.created
trade.proposal.invalidated
trade.approval.issued
trade.approval.invalidated
trade.reservation.created
trade.order.state_changed
trade.fill.observed
trade.reconciliation.changed
trade.manual_resolution.required
provider.health.changed
alpaca.paper.order.attempt.changed
```

Event payload 使用 canonical IDs 和 versioned schemas。

---


### 42.1 事件封装、顺序与恢复

~~~ts
interface DomainEvent<T> {
  eventId: string;
  eventType: string;
  schemaVersion: 1;
  occurredAt: string;
  aggregateType: string;
  aggregateId: string;
  sequence: number;
  payload: T;
}
~~~

对同一 aggregateType/aggregateId，sequence 是从 1 开始、持久化且严格递增的整数；封装不隐含跨聚合对象的全局顺序。领域变更与 outbox 事件在同一事务中持久化；重放保留原 eventId、sequence、schemaVersion 和 occurredAt。流式 UI delta 使用所属 Turn 的 sequence；瞬时动画帧不是领域事件。

IPC 传输提供精确命令 domain.subscribe({aggregateType, aggregateId, afterSequence}) 与 domain.snapshot({aggregateType, aggregateId})。Subscribe 重放游标之后保留的事件，再无缝接续实时事件；afterSequence=0 表示从头开始。Snapshot 返回一致投影和 lastSequence；新订阅从该游标之后继续。

前端忽略相同 eventId/sequence 的重复事件，不无限缓存缺口，也不推测更新金融权限；sequence 缺口或冲突重复触发快照重载。重放范围已被压缩时，返回 STATE_STALE、code IPC_REPLAY_UNAVAILABLE，并要求读取快照。快照丢失/schema 未知时，金融控件保持不可用，直至加载受支持的权威投影。未知事件类型/版本必须显式提示兼容性错误，不能静默越过并推进游标。

审批、预留、券商订单状态、账户就绪及模型路由健康保留为不同 payload。Agent item 完成、收到请求确认或未知状态均不能合成券商成交。实现 IPC 模块时，前端类型与 Rust serde 类型必须从同一实现 schema 生成或对其校验，避免维护独立漂移的定义。

---

## 43. 测试策略

### 43.1 Unit tests

- capability matrix；
- decimal arithmetic；
- instrument rules；
- risk checks；
- policy version invalidation；
- approval single-use；
- reservation accounting；
- order transition validation；
- error normalization；
- TimeService skew decision。

### 43.2 Adapter contract tests

每个 provider/environment 验证：

- symbol mapping；
- capability discovery；
- credential/permission detection；
- account read normalization；
- order request mapping；
- acknowledgement 与 fill 区分；
- cancellation semantics；
- error mapping；
- idempotency/client-order-ID；
- private stream event mapping。

### 43.3 Fault-injection tests

模拟：

- provider accept 前后 timeout；
- private stream disconnect；
- duplicate provider event；
- out-of-order event；
- RESERVED 与 SUBMITTING 之间 crash；
- provider accept 后、本地 acknowledgement 前 crash；
- SQLite transaction interruption；
- clock jump；
- quota/OAuth/model sidecar failure；
- rate-limit exhaustion。

### 43.4 Security tests

- 尝试向 Codex 暴露 keychain value；
- strategy 尝试访问 gateway/network/secret path；
- prompt-injection 请求 execution；
- log secret scanning；
- dangerous provider permission gate；
- generic Codex approval 无法进入 FinancialApproval consumption path。

### 43.5 End-to-end state assertions

验证：

- 没有 unapproved Live submission；
- TradeX retry 不产生 duplicate submission；
- broker acknowledgement 不等于 fill；
- restart 后 disarm + reconcile；
- 两个 Thread 不能 reserve 同一 capital；
- policy change 使 pending approval invalid；
- stale/clock-uncertain snapshot block execution；
- ambiguous reservation 在 evidence-based resolution 前冻结；
- model outage 不影响 reconciliation。

---

## 44. 性能与资源目标

后端实现遵循 RevC local-resource model：

- 避免 full-universe tick subscription；
- 使用 bounded queue；
- 默认持久化 1-minute+ OHLCV，而不是无限 raw tick history；
- DuckDB analytical write 批处理；
- execution safety 的 SQLite transaction 优先；
- child-process restart loop 有上限；
- reconciliation latency 与重型 backtest/research job 隔离。

Backtest 和大型 analytics 使用 worker thread/process，确保 Control Plane 保持 responsive。

---

## 45. Release 与 Migration 架构

### 45.1 Database migration

- 每个 schema versioning；
- migration 前 backup；
- 支持时 transactional migration；
- migration 后 integrity check；
- migration/reconciliation 成功前不启用 Live readiness。

### 45.2 Runtime compatibility

Release artifact 记录 pinned versions：

- TradeX app；
- Codex App Server；
- CLIProxyAPI；
- backend IPC schema；
- provider adapter schema/version；
- backtest engine。

### 45.3 Code signing 与 packaging

Tauri desktop 与 bundled/managed sidecar 进入可重复的 signed release pipeline。About/diagnostics 中展示 binary provenance/version。

---

## 46. 后端交付阶段

### Phase BE-0 — Core Control Plane

- Rust app/bootstrap；
- SQLite/DuckDB stores；
- versioned IPC schemas；
- Thread/Turn/Item persistence；
- Codex supervision；
- CLIProxyAPI supervision；
- keychain abstraction；
- Agent Mode/Execution Context capability service；
- provider schema registry；
- account-scoped arming model。

### Phase BE-1 — Research/Data Domain

- canonical instruments；
- market data tiers；
- TimeService；
- portfolio/FX/stablecoin provenance；
- screener/research MCP；
- calendars/corporate actions；
- artifact provenance。

### Phase BE-2 — Backtest 与 Non-live Execution

- deterministic backtest engine；
- strategy sandbox；
- Local Paper；
- Alpaca Paper；
- T212 Demo；
- Binance Testnet；
- Bitget Demo；
- normalized order lifecycle。

### Phase BE-3 — Trusted Live Execution

- Risk Engine；
- policy versioning；
- Approval Authority；
- Reservations；
- Order Gateway；
- T212/Binance/Bitget Live adapters；
- cancellation；
- reconciliation；
- ambiguous-state Manual Resolution；
- restart/sleep/stream recovery。

### Phase BE-4 — Hardening

- fault injection；
- audit tamper detection；
- export/import；
- telemetry controls；
- migration hardening；
- accessibility-support event semantics；
- performance/resource profiling；
- adapter capability contract coverage。

---

## 47. 后端 Definition of Done

后端 v1.0 达到 architecture-complete 的条件：

1. 全部金融 authority 位于 Agent/Model zone 之外；
2. broker credentials 只有 privileged provider code 能访问；
3. 每笔 live execution 都有 durable proposal → risk → approval → reservation → execution → broker-state provenance；
4. approval consumption 与 reservation creation transactional 且 account-serialized；
5. ambiguous non-idempotent submission 绝不 blind retry，并冻结 capacity；
6. restart/disconnect/ambiguity 后通过 reconciliation 让 provider state 成为权威；
7. account arming 按账户隔离，并在全部 RevC safety trigger 下 reset；
8. model outage 不影响 trusted order monitoring/reconciliation；
9. 所有 live market-data authority decision 使用完整 provenance 与 trusted time semantics；
10. adapter/provider capability 通过 discovery/normalization，而不是硬编码假设；
11. SQLite/DuckDB/filesystem 职责与 RevC 一致，secret 不进入普通 store；
12. 所有启用的 Live provider 都通过 end-to-end safety 与 fault-injection test。

---

## 48. 与 RevC 的后端追溯

本 ARD 主要实现：

- **FR:** FR-001–080，重点后端 ownership 为 FR-006–013、FR-019–029、FR-036–039、FR-046–047、FR-057–080；
- **NFR:** NFR-003–014、NFR-017–019；
- **SEC:** SEC-001–009；
- **DATA:** DATA-001–008；
- **OPS:** OPS-001–009；
- **UX:** 为 UX-001–010 提供后端 enforcement。

前端消费并展示这些决策，但不取代后端 authority。
