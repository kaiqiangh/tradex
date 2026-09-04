# TradeX Prototype 覆盖矩阵 — v1.0 RevC

**状态：** RevC 对齐基线  
**PRD：** `TradeX_PRD_v1.0_RevC_zh.md`  
**UI Spec：** `TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md`  
**Prototype：** `../prototype/index.html + styles.css + app.js`  

本矩阵区分“prototype 可以直接证明的 UI/状态语义”和“必须在真实 Codex runtime / broker / storage implementation 中证明的行为”。因此不会把 standalone fixture 误标成真实交易集成完成。

## 1. RevC 关键对齐项

| 主题 | RevC 结论 | Prototype 证据 |
|---|---|---|
| 状态模型 | Agent Mode = Ask / Research / Backtest / Trade；Execution Context 独立并由账户/环境推导。 | app.js mode cycle + executionContext()；UI Spec §3–4 |
| Live 权限 | Arming 按账户管理；切换账户不继承 arming；Disable All 清空全部 armed account。 | armedAccounts Set；Account Health + 顶部 banner |
| LLM 路由 | CLIProxyAPI + ChatGPT OAuth 与 DeepSeek 明确；跨 provider 自动 fallback 默认 OFF。 | Onboarding Model；Settings → Providers & Models |
| Proposal/Approval | OrderDraft 可编辑；生成后的 OrderProposal 不可变，含 proposal ID/hash/policy version。 | Live approval + order audit |
| Reservation | 明确 APPROVED → RESERVED → SUBMITTING，并提供 reservation conflict。 | Order activity + reservation conflict fixture |
| 模糊提交 | UNKNOWN_RECONCILING 使用基于证据的 Manual Resolution，不提供 blind release。 | Manual Resolution modal |
| 行情 provenance | Live approval 展示 source/provider timestamp/TradeX timestamp/venue/entitlement/age/freshness。 | Market snapshot provenance card |
| 存储 | SQLite=transactional/domain；DuckDB=MVP 1m+ OHLCV/analytics/backtest；Filesystem=artifacts；Parquet 为 Phase 2+ 可选。 | Settings → Data & Storage |
| Market-data 生命周期 | MVP 不再宣称持久 Warm tier；watchlist 使用 on-demand/coarse refresh。 | Watchlists + PRD §32 |
| Prototype 范围 | 删除 cloud Share link；workspace portability 通过 Export / Workspace Import-Restore。 | Top bar + Data & Storage |

## 2. Functional requirement traceability

| ID | 需求 | 优先级 | Prototype 状态 | 证据 |
|---|---|---:|---|---|
| FR-001 | 在本地运行一个固定版本的 Codex App Server | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 运行时架构已在文档定义；独立 prototype 不实际启动 Codex App Server。 |
| FR-002 | 支持持久化的 Thread / Turn / Item 用户体验 | P0 | Prototype 已覆盖 | 提供 Thread 历史、恢复导航、Turn/Item 卡片和不可变 Turn provenance。 |
| FR-003 | 实时流式展示 agent 与工具活动 | P0 | Prototype 已覆盖 | Running plan/tool timeline 以 fixture 模拟流式 Item 状态。 |
| FR-004 | 恢复此前的 thread | P0 | Prototype 已覆盖 | Recent Threads 可选择并恢复到工作区。 |
| FR-005 | 渲染结构化的金融时间线卡片 | P0 | Prototype 已覆盖 | 研究、风险、订单、错误和 provenance 使用结构化 timeline/card。 |
| FR-006 | 实现 TradeX 类型化领域工具 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 已展示 typed tool-call card 与状态；独立 prototype 不执行真实 App Server tool protocol。 |
| FR-007 | 将可信控制平面与 agent 区域分离 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | PRD/UI 已明确进程/权限边界；HTML prototype 无法强制进程隔离。 |
| FR-008 | 实现 OS keychain 凭证存储 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 凭据 UI 明确 OS keychain 与 SECRET 禁止披露；prototype 不访问真实 keychain。 |
| FR-009 | 实现规范化标的模型 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 标的/provider/venue identity 已统一展示；canonical backend model 仍属实现项。 |
| FR-010 | 实现提供方能力发现 | P0 | Prototype 已覆盖 | Provider 设置和 Account Detail 都展示 schema/capability review。 |
| FR-011 | 实现市场数据服务 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 行情/OHLCV/order-book 为 fixture；不包含真实 market-data service。 |
| FR-012 | 实现市场数据新鲜度元数据 | P0 | Prototype 已覆盖 | Market snapshot 展示 source、provider time、TradeX receive time、venue、entitlement、age、freshness。 |
| FR-013 | 实现投资组合聚合 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Portfolio 以 fixture 表示跨账户聚合。 |
| FR-014 | 实现账户货币与 FX 归一化 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Portfolio 展示 EUR normalization、FX route、时间/新鲜度和 stablecoin quality fixture。 |
| FR-015 | 实现 Alpaca Paper 适配器 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Alpaca Paper connection 和 non-live lifecycle 为可交互 fixture。 |
| FR-016 | 实现 Trading 212 Demo/Live 适配器 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Trading 212 Demo/Live 独立连接，并有统一 non-live/live UI。 |
| FR-017 | 实现 Binance Testnet/Live 现货适配器 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Binance Testnet/Live 独立连接，并有统一 non-live/live UI。 |
| FR-018 | 实现 Bitget Demo/Live 现货适配器 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Bitget Demo/Live 独立连接，并有统一 non-live/live UI。 |
| FR-019 | 实现 OrderProposal 模型 | P0 | Prototype 已覆盖 | 可编辑 intent 生成不可变 proposal；approval/audit 显示 proposal ID/hash/policy version。 |
| FR-020 | 实现确定性风控引擎 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 展示 risk evaluation/rejection/policy version；不执行真实 deterministic engine。 |
| FR-021 | 实现审批前校验 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Approval UI 包含 risk state/validation evidence；后端校验为 fixture。 |
| FR-022 | 实现执行前校验 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Approval/order lifecycle 明示 pre-execution revalidation；不发真实 broker check。 |
| FR-023 | 实现账户作用域内的原子执行预留 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Available/Reserved/Effective、RESERVED 和 conflict fixture 表达 account-scoped reservation。 |
| FR-024 | 实现一次性实时审批 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 展示 single-use approval/expiry/invalidation；服务端 nonce enforcement 属运行时实现。 |
| FR-025 | 实现特权 Order Gateway | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 文档定义 privileged gateway 且 UI 不直接 submit；独立 prototype 无真实 gateway。 |
| FR-026 | 实现幂等性与对账 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 展示 idempotency/reconciliation/UNKNOWN_RECONCILING；后端协议待实现。 |
| FR-027 | 实现账户作用域内的实时布防/撤防 + 全局一键全部禁用 | P0 | Prototype 已覆盖 | Arming 通过 armedAccounts 按账户管理；Disable All 清空全部账户 arming。 |
| FR-028 | 实现崩溃/启动对账 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Startup/sleep/auth/stream recovery fixture 会进入 blocked/disarmed。 |
| FR-029 | 实现本地执行审计轨迹 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Order activity 包含 proposal、reservation、validation、execution attempt、broker state audit chain。 |
| FR-030 | 实现提供方限流 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 提供 rate-limit canonical error 与 remediation fixture。 |
| FR-031 | 实现本地回测 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 有 backtest metrics/manifest/compare；真实 engine 为运行时工作。 |
| FR-032 | 实现策略沙箱 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 有 strategy/sandbox UI；实际 sandbox 隔离属运行时工作。 |
| FR-033 | 实现 paper/demo/testnet 交易工作流 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Local Paper、Alpaca Paper、T212 Demo、Binance Testnet、Bitget Demo 使用统一 non-live fixture。 |
| FR-034 | 实现确定性错误分类与 UI 映射 | P1 | Prototype 已覆盖 | Canonical error categories 映射到可见 recovery panels。 |
| FR-035 | 实现自然语言筛选器 | P1 | Prototype 已覆盖 | 有 natural-language screener 和 parsed filter/result。 |
| FR-036 | 实现产物(artifacts) | P1 | Prototype 已覆盖 | 有 Artifacts 列表/详情/provenance。 |
| FR-037 | 实现策略版本管理 | P1 | UI/状态 fixture 已覆盖；真实运行时待实现 | 展示 strategy version/compare；真实持久化版本系统待实现。 |
| FR-038 | 实现市场日历与公司行为 | P1 | UI/状态 fixture 已覆盖；真实运行时待实现 | Market session/halt/corporate-action fixture 可见，并阻止不支持的 Trade flow。 |
| FR-039 | 实现可配置的数据保留 | P1 | UI/状态 fixture 已覆盖；真实运行时待实现 | Retention policy 可见；真实 pruning/protection 待实现。 |
| FR-040 | 实现工作区导出/导入 | P1 | UI/状态 fixture 已覆盖；真实运行时待实现 | Export/Workspace Import-Restore 可见；archive I/O 为模拟。 |
| FR-041 | 增加期货支持 | P2 | 延期（P2） | 明确 P2；RevC v1.0 prototype 不提供 futures execution。 |
| FR-042 | 增加 A 股研究/数据集成 | P2 | 延期（P2） | 明确 P2；A 股集成保留未来范围。 |
| FR-043 | 增加更多券商/交易所 | P2 | 延期（P2） | 明确 P2；RevC 不要求新增 adapter。 |
| FR-044 | 实现完整的五步入门引导流程 | P0 | Prototype 已覆盖 | Onboarding 实现 Workspace → Providers → Model → Risk defaults → Ready。 |
| FR-045 | 实现可交互的 thread 历史与恢复导航 | P0 | Prototype 已覆盖 | Sidebar Thread history/resume 可交互。 |
| FR-046 | 实现上下文/账户/模型选择器 | P0 | Prototype 已覆盖 | Context/account/model picker 可交互，并披露 Turn route。 |
| FR-047 | 实现提供方连接测试 + 权限/能力审查 | P0 | Prototype 已覆盖 | Provider form 有测试/权限/capability review，包括危险权限。 |
| FR-048 | 实现提供方特定的账户详情变体 | P0 | Prototype 已覆盖 | Paper/Demo/Testnet/Live Account Detail 区分环境和 execution eligibility。 |
| FR-049 | 实现实时订单取消审批生命周期 | P0 | Prototype 已覆盖 | Live open order cancel 进入 approval-gated cancel fixture。 |
| FR-050 | 实现明确的「风控拒绝 / 券商拒绝 / 审批过期」UI 状态 | P0 | Prototype 已覆盖 | Risk rejected、broker rejected、approval expired/invalidation 明确展示。 |
| FR-051 | 实现启动/认证/流断开的恢复状态 | P0 | Prototype 已覆盖 | 提供 startup/auth/stream/rate/clock recovery states。 |
| FR-052 | 实现市价单最大授权名义金额审批 UI | P0 | Prototype 已覆盖 | Market-order approval 展示 expected notional 与 maximum authorized spend。 |
| FR-053 | 实现完整的自然语言筛选器构建流程 | P1 | Prototype 已覆盖 | Screener builder 在执行前展示 parsed structured filter。 |
| FR-054 | 实现回测运行中/失败/对比状态 | P1 | Prototype 已覆盖 | Backtest running/result/failure/compare fixture 路径已覆盖。 |
| FR-055 | 实现产物溯源展示 | P1 | Prototype 已覆盖 | Artifact provenance 展示来源 Thread/Turn/context lineage。 |
| FR-056 | 实现无执行能力的 Ask 模式 | P0 | Prototype 已覆盖 | Ask 是独立 Agent Mode，并明确 read-only/no execution。 |
| FR-057 | 实现完整的实时审批市场快照溯源 | P0 | Prototype 已覆盖 | Live approval 展示完整 market snapshot provenance/freshness。 |
| FR-058 | 实现风控策略变更导致审批失效的生命周期 | P0 | Prototype 已覆盖 | Risk policy version 变化使 approval 失效；weakening 额外 disarm 对应账户。 |
| FR-059 | 实现预留冲突 UI/推理展示 | P0 | Prototype 已覆盖 | Reservation conflict 展示 reserved/effective capacity 和 deterministic rejection。 |
| FR-060 | 实现 Trading 212、Binance 与 Bitget 完整的 Demo/Testnet 订单生命周期变体 | P0 | Prototype 已覆盖 | T212 Demo、Binance Testnet、Bitget Demo 复用统一 non-live lifecycle。 |
| FR-061 | 实现完整的归一化订单状态 UI 映射(含 `RESERVED`) | P0 | Prototype 已覆盖 | Order timeline 明确 APPROVED → RESERVED → SUBMITTING → ACCEPTED → FILLED → RECONCILED。 |
| FR-062 | 实现市场时段 / 停牌 / 公司行为产品界面 | P1 | Prototype 已覆盖 | Instrument detail 展示 OPEN/CLOSED/HALTED/corporate action，并执行 Trade blocking。 |
| FR-063 | 实现工作区导入/恢复工作流 | P1 | Prototype 已覆盖 | Workspace Import/Restore 有 choose → validate → restore，恢复后 Live 全部 DISARMED。 |
| FR-064 | 实现完整的规范化错误修复界面 | P1 | Prototype 已覆盖 | Canonical error recovery modal 覆盖 trading/account/LLM/clock-freshness 类别。 |
| FR-065 | 实现投资组合 FX 溯源展示 | P1 | Prototype 已覆盖 | Portfolio 展示 FX source/route/timestamps/freshness。 |
| FR-066 | 实现完整的回测指标集(含 Sortino、盈利因子与换手率) | P1 | Prototype 已覆盖 | Backtest 增加 Sortino、Profit Factor、Turnover。 |
| FR-067 | 实现由提供方 schema 驱动的凭证/配置表单 | P1 | Prototype 已覆盖 | ProviderCredentialSchema 按 provider 动态定义 fields/permissions。 |
| FR-068 | 实现 LLM 提供方连接工作流(CLIProxyAPI OAuth / DeepSeek 密钥),基于 §26.3 的 schema 驱动 | P0 | Prototype 已覆盖 | Onboarding/Settings 展示 CLIProxyAPI ChatGPT OAuth 与 DeepSeek key/model。 |
| FR-069 | 实现 CLIProxyAPI sidecar 生命周期:启动/监管/健康检查(`/v1/models` 探测)/退避重启/端口冲突处理 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 可见 sidecar state/version/models/port-auth error fixture；真实 supervisor 待实现。 |
| FR-070 | 实现订阅配额展示与耗尽处理(provider 可报告的窗口;进行中 attempt 行为;显式切换/重试) | P0 | Prototype 已覆盖 | Quota exhaustion 默认要求显式 switch/retry，除非用户开启 fallback opt-in。 |
| FR-071 | 实现 LLM 错误类别与恢复状态(`MODEL_UNAVAILABLE` / `QUOTA_EXCEEDED` / `OAUTH_EXPIRED`)及修复 UI | P0 | Prototype 已覆盖 | 提供 MODEL_UNAVAILABLE / QUOTA_EXCEEDED / OAUTH_EXPIRED remediation。 |
| FR-072 | 记录每个 turn 的模型/提供方来源,并将模型切换/降级写入审计轨迹 | P0 | Prototype 已覆盖 | Composer/Turn provenance 披露 provider/model/attempts/route。 |
| FR-073 | 实现模型路由/fallback:跨 provider 自动 fallback 默认关闭、显式 opt-in、完整审计/披露且不改变 capability | P0 | Prototype 已覆盖 | 跨 provider 自动 fallback 默认 OFF；opt-in 明确且可审计。 |
| FR-074 | 将 Agent Mode 与 Execution Context 分离并强制执行兼容矩阵 | P0 | Prototype 已覆盖 | Agent Mode（Ask/Research/Backtest/Trade）与由账户/环境决定的 Execution Context 分离。 |
| FR-075 | 持久化不可变的每 turn mode/account/execution/model/provider/context 快照 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | 不可变 Turn snapshot 字段已展示；持久化属运行时工作。 |
| FR-076 | 为 `UNKNOWN_RECONCILING` 实现基于证据的 Manual Resolution;禁止盲目释放 reservation | P0 | Prototype 已覆盖 | UNKNOWN_RECONCILING 使用基于证据的 Manual Resolution，不存在 blind release。 |
| FR-077 | 实现 TimeService 的 clock-skew/freshness/TTL Live 安全门 | P0 | UI/状态 fixture 已覆盖；真实运行时待实现 | Clock skew/freshness recovery 阻断 Live authority；真实 TimeService 待实现。 |
| FR-078 | Live readiness 前阻断/审查危险 provider 权限(withdrawal/transfer/custody/margin/leverage) | P0 | Prototype 已覆盖 | Withdrawal/transfer/custody/margin/leverage 权限在 Live Ready 前被禁止或审查。 |
| FR-079 | 实现 FX/stablecoin 估值溯源与 depeg/quality 处理 | P1 | UI/状态 fixture 已覆盖；真实运行时待实现 | 有 USDT→USD→EUR provenance/depeg quality fixture；生产 valuation service 待实现。 |
| FR-080 | 实现可编辑 OrderDraft → 不可变 OrderProposal 的重新生成语义 | P0 | Prototype 已覆盖 | OrderDraft 修改/再生成会创建新的不可变 proposal identity，并使旧 approval 失效。 |

## 3. Acceptance-criterion traceability

| ID | 验收标准 | Prototype 状态 | Traceability 说明 |
|---|---|---|---|
| AC-001 | 用户可以创建 TradeX thread,并在完整 turn 结束前看到流式展示的 agent 项。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-002 | 用户可以关闭并重新打开 TradeX,恢复已持久化的 thread。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-003 | 工具调用展示名称、状态、持续时间与结构化摘要。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-039 | 用户可以从 composer 检查并恢复最近的 Thread,管理附加的上下文/账户/模型。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-041 | Ask 模式提供轻量分析/只读上下文,不暴露 paper 或 live 执行能力。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-004 | Demo 与 live 账户表示为独立的连接。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-005 | 账户连接时 TradeX 校验提供方能力。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-006 | 若账户对账不完整,实时执行被禁用。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-036 | 提供方连接界面在连接被视为就绪前,展示检测到的权限与能力集合。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-038 | 完整的首次运行工作流覆盖 Workspace → Providers → LLM (Model) → Risk Defaults → Ready,其中 Model 步骤配置 LLM 提供方:CLIProxyAPI sidecar 状态(运行中/OAuth 已授权)和/或经探测 + 测试推理校验的 DeepSeek 密钥(§26.3)。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-055 | 若无至少一个可用 LLM 提供方,入门流程无法到达 Ready;已停止、端口冲突或未授权的 sidecar 会阻断 Ready 并提供修复指引。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-051 | 提供方连接 UI 由提供方凭证/能力 schema 渲染,不假设每个提供方使用相同的凭证字段。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-007 | Agent 可在单个 thread 中结合市场数据、投资组合状态与研究。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-008 | 实时审批中展示的每个市场快照均含来源、提供方时间戳、TradeX 接收时间戳、场所、实时/延迟授权与新鲜度状态。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-040 | 自然语言筛选器在执行前展示已解析的结构化筛选条件,并渲染缩减后的候选集。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-049 | 股票标的详情暴露相关的市场时段/公司行为状态,且对不支持的已收盘/停牌执行确定性地阻止。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-053 | 跨账户投资组合值展示工作区基础货币,以及归一化值的 FX 来源/时间戳/新鲜度。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-009 | 用户可提交 Alpaca Paper 订单并查看结果券商状态。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-010 | 用户可在 Trading 212、Binance 与 Bitget 上执行受支持的 Demo/Testnet 订单工作流(在可用时),并在确认/成交状态中带有显式环境标签。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-011 | 无有效 TradeX 金融审批,实时订单无法提交。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-012 | 通用 Codex 审批无法授权实时金融执行。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-013 | 变更标的、方向、数量、账户、订单类型、价格或有效期使先前审批失效。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-014 | 相关风控策略变更使受影响的待审批项失效;策略弱化还额外撤防受影响的实时账户。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-015 | 陈旧的市场快照阻止执行,直至刷新。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-016 | 提交前立即重新校验风控状态。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-017 | 两个并发 thread 无法预留相同的现金或敞口。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-018 | Agent 无法修改风控策略。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-019 | Agent 无法直接访问特权 Order Gateway。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-020 | 模型绝不接收券商密钥。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-030 | 选择 Trade 模式或 Live Execution Context 不布防实时执行。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-031 | 在 DISARMED 状态下请求实时订单,需在展示交易审批前执行单独的显式 Arm 操作。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-032 | 订单监控期间展示的标的、账户、方向、数量与订单类型,与用户批准的不可变交易一致。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-033 | 未结实时订单唯有在券商状态刷新且经交易特定的用户审批后,方可进入取消流程;UI 呈现 `CANCEL_PENDING` 与 `CANCELLED`。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-034 | UI 呈现确定性的 `RISK_REJECTED`、券商 `REJECTED`、审批 `EXPIRED` 与 `UNKNOWN_RECONCILING` 状态,且不暗示执行成功。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-035 | 市价单审批展示预期支出、最大授权支出、买卖价差、报价年龄以及(在可用时)费用/滑点估计。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-042 | 实时布防为账户作用域:布防 Trading 212 不会布防 Binance 或 Bitget;全局「全部禁用」撤防所有实时账户。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-043 | 每个实时审批暴露由 AC-008 定义的完整市场快照溯源。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-044 | 保存风控策略使受影响的待审批项失效并传达失效原因;策略弱化撤防受影响的账户。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-045 | 在较早提案预留现金/敞口后,第二个并发提案按缩减后的账户容量评估,并在容量不足时被确定性拒绝。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-046 | Trading 212 Demo、Binance Testnet 与 Bitget Demo/Testnet 兼容工作流在提案、确认、成交/更新与账户持仓刷新全程保持明确的非实时环境标识。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-021 | 除非券商状态确认成交,否则券商确认绝不显示为成交。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-022 | 模糊的提交进入 `UNKNOWN_RECONCILING`。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-023 | TradeX 绝不在模糊超时后盲目重试非幂等的提供方订单 POST。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-024 | 丢失私有流连接触发降级状态与对账。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-025 | 重启应用后,在启用新实时提交前先对账未结实时订单。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-026 | 每个实时订单具备从提案 → 风控评估 → 审批 → 预留 → 执行尝试 → 提供方状态的持久链条。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-037 | 认证失败与私有流断开会禁用新实时执行,直至账户健康恢复。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-050 | 规范化错误类别渲染适当的修复措施,且不引入冲突的机器状态名称。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-027 | 策略代码无法访问券商凭证。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-028 | 外部研究内容无法授权交易。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-029 | 券商密钥绝不出现在日志、产物、thread 历史、SQLite、DuckDB 或 Parquet 中。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-047 | 工作区导入/恢复校验归档/schema 元数据,不导入券商密钥,且在重新启用实时执行前要求对账。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-048 | 回测结果包含收益、Sharpe、Sortino、最大回撤、胜率、盈利因子、换手率、权益曲线、交易清单与可复现清单。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-052 | 核心交互界面可通过键盘访问且具可见焦点,通用 `Enter` 提交无法批准实时订单或取消。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-054 | 窄桌面/平板级布局保留实时安全信息;v1.0 不提供原生移动端实时执行客户端。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-056 | 当 CLIProxyAPI sidecar 停止、端口冲突或未授权时,agent turn 暂停并提供修复指引,而审批、执行与对账继续运行(仅对 LLM 故障关闭)。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-057 | 订阅配额耗尽与 OAuth 过期呈现 `QUOTA_EXCEEDED` / `OAUTH_EXPIRED`,并提供 retry/re-login/显式 provider switch。只有用户显式开启时才允许跨 provider 自动 fallback;fallback 必须明显披露并写入审计,且绝不改变 capability 或绕过审批。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-058 | 每个 turn 记录产生/失败该 turn 的 model/provider 与全部 provider attempt;模型切换/降级写入审计轨迹(§57)。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-059 | Agent Mode 与 Execution Context 作为独立维度存储与呈现;Ask/Research + Live account 始终只读,Backtest 不能调用当前市场 broker execution,Trade + Live context 仍需 arming + approval。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-060 | 每个启动的 turn 持久化 Agent Mode、Execution Context、所选 account、capability、model/provider 与 attached context reference/hash 的不可变快照。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-061 | 模糊 Live submission 超过自动对账窗口后 reservation 继续冻结,仅暴露 §23 的 evidence-based Manual Resolution;不存在可恢复 Live readiness 的通用 release 操作。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-062 | Live broker credential 检测到 withdrawal/transfer/custody 权限时阻止 execution-ready 直至移除;无法 introspect 的权限持续标记 `UNVERIFIED`。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-063 | 当 TimeService 报告不可接受 clock uncertainty 时,quote age、approval expiry 与 reconciliation timer 对 Live 权限判断必须 fail closed。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-064 | Local Paper 明确标记为 TradeX simulation,不得与 provider-hosted Paper/Demo/Testnet 或 Live 混淆。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-065 | 编辑已生成订单会创建新的不可变 `OrderProposal` identity,并使旧 proposal 的 approval 失效。 | Prototype 可直接验证 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-066 | 涉及 stablecoin/FX 的跨账户估值展示 source/path/timestamps/freshness 与 quality/depeg 状态;不可靠转换不得静默驱动 Live 风控。 | UI/状态 fixture 对齐；真实运行时验收待实现 | 对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |

## 4. UI Spec 屏幕覆盖

| 区域 | RevC Prototype | 状态 |
|---|---|---|
| A Onboarding | Workspace / Providers / CLIProxyAPI+DeepSeek / Risk / Ready | Covered |
| B Thread Workspace | Ask/Research/Backtest/Trade、Execution Context、Turn provenance | Covered |
| C Markets | Screener、Instrument、market state、full provenance | Covered |
| D Watchlists | on-demand/coarse refresh，取消 Warm-tier 承诺 | Covered |
| E Accounts/Portfolio | FX provenance、account health、account-scoped arming | Covered |
| F Live Approval | proposal identity、policy、snapshot provenance、reservation、expiry/invalidation | Covered |
| G Non-live execution | Local Paper / Alpaca Paper / T212 Demo / Binance Testnet / Bitget Demo | Covered fixture |
| H Strategies/Backtest | run/result/failure/compare、Sortino/Profit Factor/Turnover | Covered fixture |
| I Artifacts | list/detail/provenance | Covered |
| J Settings | Providers & Models / Risk / Data / Health / Appearance / About | Covered |
| K Recovery | canonical errors、LLM errors、clock/freshness、Manual Resolution | Covered fixture |

## 5. 有意保留的 Prototype 限制

- 不连接真实 Codex App Server、CLIProxyAPI、broker/exchange 或 market-data backend。
- 不执行真实订单、撤单、OAuth、keychain、SQLite/DuckDB 写入或 workspace archive I/O。
- P2 的 Futures、A 股、额外 provider 不属于 RevC v1.0 prototype。
- 浏览器级 visual regression / keyboard screen-reader QA 仍属于实现前的独立 QA gate；源码已加入 `focus-visible`、dialog ARIA、`aria-live` 与 `prefers-reduced-motion` 基线。

这些限制是明确的 prototype 边界，不是 PRD ↔ prototype 的未对齐项。
