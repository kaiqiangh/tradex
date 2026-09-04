# TradeX 高保真原型 / UI 规范

**版本:** 1.0 Final — Revision B  
**日期:** 2026-09-04  
**权威来源:** `TradeX_PRD_v1.0_RevB.md`  
**原型类型:** Desktop-first 交互式产品原型目标规范  
**当前原型实现:** 独立 HTML / CSS / JavaScript  
**可追溯性来源:** `TradeX_Prototype_Coverage_Matrix_v1.0_RevB.md`

> 本规范对目标 UI 具有规范性。Coverage Matrix 记录了当前独立原型中哪些部分已实现、部分实现、仅视觉实现、缺失、仅实现层或待 QA。

---

# 1. 原型目标

TradeX 的运作可表示为:

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

持久的 Agent Thread 是主要的工作区对象。TradeX 不得退化为带有 AI 聊天侧边栏的传统交易仪表盘。

---

# 2. 设计系统

## 2.1 产品特性

- 桌面端优先
- 简洁、克制、面向实现
- 深色左侧导航 + 浅色主工作区
- 中等信息密度
- 交易状态颜色具有语义性,绝不仅仅用于装饰
- 研究 / 智能体工作保持视觉主导;市场环境为次要

## 2.2 颜色与模式语义

| 标记 | 取值 | 用途 |
|---|---|---|
| App background | `#F6F8FB` | 主工作区 |
| Surface | `#FFFFFF` | 面板、表格、弹窗 |
| Sidebar | `#111827` | 导航 |
| Sidebar active | `#1F2937` | 选中导航项 |
| Primary text | `#0F172A` | 主文本 |
| Muted | `#64748B` | 次要信息 |
| Border | `#E2E8F0` | 分隔线 / 面板 |
| Blue | `#2563EB` | 研究 / 主操作 |
| Green | `#16A34A` | Paper/Demo/Testnet / 健康 / 已完成 |
| Red | `#DC2626` | Live / 破坏性 / 已拒绝 |
| Amber | `#D97706` | 过期 / 警告 / 降级 / 对账中 |

模式语义:

- **Ask(轻问答)** → 中性 / 灰色;
- **Research(深度研究)** → 蓝色;
- **Backtest(回测)** → 分析蓝 / 中性;
- **Paper / Demo / Testnet** → 绿色;
- **Live** → 红色;
- **Warning / stale / degraded / reconciling** → 琥珀色.

Paper/Demo/Testnet/Live 除颜色外必须始终使用明确的文本标签。

---

# 3. 全局产品外壳与导航

桌面外壳:

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

主导航:

- New Thread
- Threads
- Markets
- Watchlists
- Accounts
- Strategies
- Artifacts
- Settings

设置导航:

- Providers & Models
- Risk & Limits
- Data & Storage
- Account Health
- Appearance
- About

投资组合、订单、回测、账户详情、标的详情、来源追溯以及恢复仍为上下文驱动的表面。

会话历史按时间近远分组,并恢复上下文,但绝不恢复已消费 / 已过期的审批。

---

# 4. 模式、输入区与上下文

## 4.1 模式

| 模式 | UI 意图 | 执行能力 |
|---|---|---|
| Ask | 轻量问答 / 快速回答 | 无 |
| Research | 更深入的市场 / 投资组合调研 | 无 |
| Backtest | 策略研究 | 仅历史模拟 |
| Paper | 本地 / 券商托管的模拟 | 仅 paper/demo/testnet |
| Live | 真实账户上下文 | 提案 + 账户布防 + 审批 |

Ask 模式不得暴露 paper/live 执行操作。

## 4.2 上下文选择器

`@ Context` 可附加:

- 标的;
- 账户;
- 策略;
- 回测运行;
- 产物。

## 4.3 账户选择器

每行都包含明确的环境标识:

```text
Alpaca · PAPER
Trading 212 · DEMO
Trading 212 · LIVE · DISARMED
Binance · TESTNET
Binance · LIVE · ARMED
Bitget · DEMO
Bitget · LIVE · DISARMED
```

## 4.4 模型选择器

模型以两个服务商分组展示(OD-015 已解决, PRD §16.1):

```text
CLIProxyAPI (local gateway)
  gpt-5.6-sol
  gpt-5.6-luna

DeepSeek (official API)
  deepseek-chat
  deepseek-reasoner
```

每行带有服务商标签(CLIProxyAPI = 绿色 / 本地,DeepSeek = 蓝色 / API)。原型 fixture 模型遵循上述列表;生产模型清单来自 CLIProxyAPI `/v1/models` 探测,经由运行时 / 能力层获取。

选择模型仅改变推理模型——它在下一次 turn 生效,绝不变更进行中的 turn,也绝不改变交易权限或许可。模型行展示可用性(探测状态),以便在发送提示前可见 quota/OAuth 降级情况。

---

# 5. 完整屏幕 / 状态清单

> ID 约定:下文 `A`–`K` 的屏幕 / 状态分组是 Coverage Matrix 引用的稳定标识符;章节编号 §1–§13 是本规范自身的结构。

## A. 引导

### A1. 工作区

- 工作区名称
- 基础货币
- 本地存储路径

### A2. 服务商

- Alpaca Paper
- Trading 212 Demo
- Trading 212 Live
- Binance Testnet
- Binance Live
- Bitget Demo
- Bitget Live

### A3. 服务商连接 / 权限审查

连接表单是**服务商模式驱动**的。它可能包含 API key、secret、passphrase、account ID、environment,或服务商特定字段;规范不得假设通用的 `key + secret` 模式。

流程:

```text
Provider/environment
→ provider-defined credential fields
→ trusted credential boundary
→ Test Connection
→ account type/capability discovery
→ permission review
→ Connection Success
```

UI 明确排除提现、转账、托管、保证金借贷以及杠杆管理权限。

### A4. LLM Provider(模型)配置

Model 步骤配置 LLM 来源(PRD §26.3):

- CLIProxyAPI sidecar 状态:Running / Stopped / 端口冲突 / 未授权——已停止或未授权的 sidecar 提供启动与浏览器(`--codex-login`)授权流程;
- DeepSeek:key 录入 → OS keychain 存储 → 探测 + 测试推理;
- 对每个来源探测 `/v1/models` 与模型列表发现;服务商行展示健康标签;
- 从发现的清单中选择默认模型(§4.4 分组);
- 明确说明:模型 / 服务商选择不改变交易权限;
- 引导门槛:若无至少一个可用 LLM 服务商,则无法进入 Ready(AC-055)。

### A5. 风险默认值

初始用户可配置策略包括:

- 单笔订单名义金额上限;
- 单一标的最大敞口;
- 每日成交名义金额上限;
- 每日已实现亏损上限;
- 过期报价阈值;
- 市价单策略;
- live 闲置超时。

高级设置可暴露来自 PRD §21 的额外用户可配置控件。系统强制的硬安全规则以只读形式展示,且不可被削弱。

> 权威来源:PRD §21.1(用户可配置控件)与 §21.2(硬安全规则)。本节为引导目的对其进行了摘要;请先编辑 PRD,再在此同步。

### A6. 就绪

- 工作区摘要;
- 服务商;
- 模型;
- 基础货币;
- 所有 live 账户 = DISARMED。

---

## B. 智能体工作区

### B1. 新建会话

建议提示:

- 对比 NVDA、AMD 与 AVGO
- 审视我的 live 投资组合风险
- 回测 BTC 动量策略
- 对比 Binance 与 Bitget 上的 BTC 流动性

### B2. 会话历史 / 恢复

分组为 Today / Earlier / Saved。恢复会还原上下文,而非 live 审批。

### B3. Ask 模式

一种轻量响应状态,可选只读市场环境,默认无重型计划工作流,无 paper/live 执行操作。

### B4. 输入区选择器

- 上下文选择器
- 账户选择器
- 模式选择器
- 模型选择器

### B5. 研究中

- 用户请求;
- 计划;
- Done / Running / Queued;
- 类型化工具;
- 中间结果。

### B6. 工具 / Turn 状态

```text
Tool Running
Tool Completed
Tool Failed
Tool Retrying
Turn Cancelled
Turn Interrupted
```

### B7. 研究结果 — 股票

- 结论;
- 关键发现;
- 情景分析;
- 证据 / 来源追溯;
- 产物;
- 在模式允许时的 paper/live 提案操作。

### B8. 研究结果 — 加密货币

- Binance / Bitget 场所对比;
- 价差;
- 盘口最优档深度;
- 报价时效;
- 所选场所;
- 允许时的市价单提案操作。

---

## C. 市场与筛选

### C1. 市场浏览器

- 搜索;
- 美股 / 加密货币 / 筛选器;
- 宽基市场指标;
- 领涨领跌;
- 已保存筛选器。

### C2. 自然语言筛选器构建器

```text
Natural-language request
→ Parsed FilterSpec / RankSpec
→ Inspect/edit structured interpretation
→ Run
→ Reduced candidate set
```

### C3. 筛选结果

候选表替换宽基市场全集,并可启动智能体深度研究。

### C4. 股票标详情

- 报价 / 图表;
- 概览 / 财务 / 新闻 / 公告文件 / 分析;
- 来源 / 时效性元数据;
- 关联论点;
- 自选列表操作。

### C5. 加密货币标详情

- 场所报价 / 图表;
- 买价 / 卖价 / 价差;
- 成交量 / 盘口深度;
- 报价时效;
- Binance 与 Bitget 对比;
- 余额 / 持仓;
- 允许时的市价单操作。

### C6. 市场时段 / 公司行为

股票详情还暴露:

```text
Market status: OPEN / CLOSED / EXTENDED / HALTED
Next open/close
Upcoming corporate action
Historical adjustment state
```

`MARKET_CLOSED` 与 `INSTRUMENT_HALTED` 在不允许执行时使用确定性阻断界面。

---

## D. 自选列表

### D1. 自选列表库
### D2. 自选列表详情
### D3. 新建自选列表
### D4. 向自选列表添加标的

自选列表行可包含代码、价格、涨跌幅、市值、论点与提醒。

---

## E. 账户与投资组合

### E1. 账户概览

所有 MVP 环境均为独立连接。

### E2. 服务商特定账户详情

在支持时展示:

- 服务商 / 环境 / 账户类型;
- 健康状态;
- 余额 / 权益;
- 持仓;
- 未平仓订单;
- 上次同步;
- 上次对账;
- 权限 / 能力;
- 凭证健康;
- 风险策略摘要;
- 可选 IP 允许列表状态;
- 每账户 live 布防状态。

### E3. 投资组合

展示:

- 总值;
- 每日盈亏;
- 现金;
- 配置;
- 集中度;
- 跨账户持仓;
- **Workspace Base Currency** 归一化值;
- 外汇来源 / 时间戳 / 时效性。

Fixture 数据可使用 EUR;规范不硬编码 EUR。

### E4. 未平仓订单

部分成交的订单可进入撤单流程。

---

## F. Live 执行与安全

### F1. 账户级 Live 布防

```text
Prepare Live Order for Trading 212
→ Trading 212 DISARMED
→ Arm Trading 212 Live
→ Trading 212 ARMED
→ Binance/Bitget remain DISARMED
→ order approval may be shown
```

头部 / 输入区展示 `LIVE · Trading 212 · ARMED` 或等价信息。

全局 `Disable All Live Execution` 会解除所有 live 账户的布防。

### F2. 限价单审批 — Trading 212 / AAPL

不可变交易加上 Market Snapshot(市场快照)面板。

必填 Market Snapshot 字段:

- 报价;
- 场所;
- 服务商 / 来源;
- 服务商时间戳;
- TradeX 接收时间戳;
- 实时 / 延迟 行情权限;
- 报价时效;
- 时效性状态。

风险检查包含预约容量。

### F3. AAPL 提交 / 监控

```text
APPROVED
→ RESERVED
→ SUBMITTING
→ ACCEPTED
→ PARTIALLY_FILLED
→ FILLED
→ Reconciled
```

精确的账户 / 标的 / 方向 / 数量 / 订单类型始终与已批准提案保持一致。

### F4. 市价单审批 — Binance / BTC

- 预期花费;
- 最大授权花费;
- 买价 / 卖价 / 价差;
- 服务商 / 来源 / 时间戳;
- 报价时效;
- 费用 / 滑点估算;
- 确定性风险检查。

### F5. 审批失效 — 过期 / 变化的市场
### F6. 审批过期
### F7. 风险拒绝
### F8. 券商拒绝

机器映射:

```text
Order state: REJECTED
Error category: SUBMISSION_REJECTED
UI label: Broker rejected order
```

### F9. 歧义提交

`UNKNOWN_RECONCILING`,自动重试被阻止。

### F10. Live 撤单

```text
PARTIALLY_FILLED
→ Cancel Remaining
→ approval
→ CANCEL_PENDING
→ CANCELLED
```

### F11. 预约冲突

第二个并发提案展示:

```text
Account cash            €10,000
Reserved by Thread A     €4,000
Risk-available           €6,000
Thread B requested       €7,000

RISK_REJECTED
Reason: insufficient unreserved account capacity
```

### F12. 风险策略变更失效

```text
Pending live approval exists
→ user saves relevant risk policy change
→ pending approval becomes INVALID
→ reason displayed
→ if policy weakened: affected live account DISARMED
```

---

## G. Paper / Demo / Testnet

### G1. Alpaca Paper 订单
### G2. Alpaca Paper 成交

Paper 警告说明模拟的局限性。

### G3. 通用券商托管 Demo/Testnet 订单流程

可复用的归一化界面,含明确变体:

- Trading 212 Demo;
- Binance Testnet;
- Bitget Demo。

```text
Select environment
→ order proposal
→ simulated/provider acknowledgement
→ order update/fill
→ account/position update
```

任何 Demo/Testnet 状态都不得在视觉上被误认为 Live。

---

## H. 策略与回测

### H1. 策略列表
### H2. 可编辑策略沙盒

可见的被阻断能力:

- 网络;
- Keychain;
- Order Gateway。

### H3. 回测运行中
### H4. 回测失败
### H5. 回测结果

必需指标:

- 收益;
- 夏普比率;
- 索提诺比率;
- 最大回撤;
- 胜率;
- 利润因子;
- 换手率;
- 权益曲线;
- 交易清单;
- 可复现性清单。

### H6. 回测对比

- 指标差异;
- 参数差异;
- 清单 / 版本链接。

---

## I. 产物

### I1. 产物库
### I2. 产物详情
### I3. 来源追溯抽屉 / 弹窗
### I4. 导出

来源追溯包含 Thread、Turn、模型、工具、来源、快照 / 数据集哈希,以及适用时的相关订单。

---

## J. 设置

### J1. 服务商与模型

复用服务商模式驱动的连接 / 配置流程。

RevB 新增 —— LLM 服务商部分:展示 `CLIProxyAPI · local sidecar`(状态:Running / Stopped / 端口冲突 / 未授权;固定版本;Open Auth / 重新登录操作)与 `DeepSeek · official API`(已连接 / 缺少 Key;Key 由 OS keychain 管理)。每行暴露探测结果与模型可用性;LLM 服务商在视觉上与券商 / 数据服务商分离,且绝不授予交易能力(PRD §26.3)。

### J2. 风险与限额

保存策略:

```text
persist policy
→ invalidate affected pending approvals
→ audit change
→ weakening policy disarms affected live account
→ show confirmation
```

硬安全规则以只读形式出现。

### J3. 数据与存储

- SQLite / DuckDB / Parquet;
- 产物 / 缓存;
- 保留期;
- 隐私边界;
- 导出工作区;
- 立即备份。

### J4. 导入 / 恢复工作区

流程:

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

券商密钥绝不从工作区归档中导入。

---

## K. 恢复与错误状态

### K1. 睡眠后恢复
### K2. 启动对账
### K3. 认证失败
### K4. 私有流断开
### K5. 限流

### K6. 规范错误界面变体

> 权威来源:PRD §51 错误分类法。下表为其 UI 映射镜像;请先编辑 PRD,再在此同步。

一个可复用的 Error/Recovery(错误 / 恢复)组件映射:

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
- `INTERNAL_ERROR`。

---

# 6. 订单状态 → UI 映射

> 权威来源:PRD §45(订单状态模型与 UI 映射)。本表为其 UI 便利镜像;请先编辑 PRD,再在此同步。

| 领域状态 | UI 界面 |
|---|---|
| DRAFT | 可编辑草稿 / 输入区 |
| PROPOSED | OrderProposalCard |
| RISK_REJECTED | RiskRejectedPanel |
| NEEDS_APPROVAL | LiveApprovalModal |
| APPROVED | 时间线 / 审计事件 |
| RESERVED | ReservationEvent / 账户容量详情 |
| SUBMITTING | OrderTimeline 待处理状态 |
| ACCEPTED | 已确认未成交状态 |
| PARTIALLY_FILLED | 成交 + 剩余数量 |
| FILLED | 权威成交摘要 |
| CANCEL_PENDING | 撤单待处理 + 竞态警告 |
| CANCELLED | 权威撤单 |
| REJECTED | 券商拒绝 UI;错误类别 `SUBMISSION_REJECTED` |
| EXPIRED | 过期审批 / 订单界面 |
| UNKNOWN_RECONCILING | 歧义状态 / 重试被阻止 |

---

# 7. 关键 UX 不变量

> 权威来源:PRD §62.5(UX-001–007 UX 安全要求)。下表为其 UI 实现展开;请先编辑 PRD,再在此同步。

1. Live 模式选择不会布防执行。
2. Live 布防是显式且账户级的。
3. 布防一个 live 账户不会布防另一个。
4. 显式 live 布防不批准订单。
5. 每笔 live 订单都需要一次性、交易特定的审批。
6. 每次 live 撤单都需要交易特定的审批。
7. 相关的风险策略变更会使受影响的待处理审批失效。
8. 削弱策略会解除受影响账户的布防。
9. 重大提案变更会使审批失效。
10. 过期的审批不可复用。
11. 过期市场数据会阻断 live 执行。
12. 审批包含市场数据来源追溯,而不仅仅是报价时效。
13. 已批准订单的标识在监控过程中保持一致。
14. 券商确认不等于成交。
15. 歧义提交不会被盲目重试。
16. 账户预约防止跨线程的双重花费。
17. 风险策略不能被智能体修改。
18. 策略代码无法访问券商凭证或特权 Order Gateway。
19. 券商 / 交易所状态是权威的。
20. Paper / Demo / Testnet / Live 在文本与结构上均明确区分。
21. 回车键提交不能批准 live 交易。
22. 模型回退或切换绝不改变能力等级、风险限额或审批要求;切换会被写入审计轨迹(PRD §16.3)。
23. LLM 不可用(sidecar 宕机 / quota / OAuth)仅暂停智能体 turn——审批、执行与对账绝不依赖模型可用性。

---

# 8. 组件清单

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

# 9. 无障碍与键盘交互

最低规则:

- 交互控件上有可见焦点指示;
- 逻辑 Tab 顺序;
- 控件具有可访问名称;
- 在适当处播报状态更新;
- `Enter` 绝不隐式批准 live 订单 / 撤单;
- `Escape` 可关闭 / 拒绝可解除了的审批,但绝不批准它;
- 触控 / 点击目标在窄布局下仍可用;
- 状态绝不单凭颜色传达;
- 尊重减少动效偏好。

---

# 10. 响应式行为

在宽度低于 900 px 时:

- 桌面侧边栏折叠;
- 可能出现紧凑 / 底部导航;
- 多列布局堆叠;
- 上下文面板移至主内容下方;
- 策略侧面板可能折叠;
- 输入区标签可能横向滚动;
- live 订单标识、市场快照来源追溯、风险检查、拒绝 / 批准以及禁用 live 操作仍可达。

这是响应式的桌面 / 平板窗口行为。TradeX v1.0 未定义原生的移动端 live 执行产品。

---

# 11. 原型 Fixture 数据

诸如 AAPL `$221.42`、BTC/USDT `62,418.20` 以及 EUR 投资组合值等示例值仅为 fixture 数据。

Fixture 货币不会重定义产品需求:投资组合聚合使用配置好的 Workspace Base Currency。

---

# 12. 开发交接指引

生产前端将 fixture 状态替换为:

- Codex App Server Thread / Turn / Item 事件;
- 归一化的市场数据与快照来源追溯模型;
- 账户 / 券商适配器读取;
- 可信的 TradeX 控制面事件;
- 确定性的风险与预约决策;
- 特权 Order Gateway 事件;
- 本地 SQLite / DuckDB / Parquet 持久化。

前端渲染权限决策,但不拥有金融权限。

---

# 13. 原型完成标准

一个产品评审完整的原型应让评审者能够导航:

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

Coverage Matrix 是哪些目标状态已存在于当前独立原型中的权威记录。
