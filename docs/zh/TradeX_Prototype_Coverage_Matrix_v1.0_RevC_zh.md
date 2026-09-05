# TradeX Prototype 覆盖矩阵 — v1.0 RevC

**修订日期：** 2026-09-05\
**状态：** 按证据重新建立基线；原型交付 NOT PASS\
**权威文档：** [PRD](./TradeX_PRD_v1.0_RevC_zh.md)、[UI Spec](./TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md)、[Frontend ARD](./TradeX_Frontend_ARD_v1.0_RevC_zh.md)、[Backend ARD](./TradeX_Backend_ARD_v1.0_RevC_zh.md)\
**Prototype：** [docs/prototype](../prototype/README_zh.md)，审查源码 main@6c4b267

保留全部 80 个 FR、66 个 AC 的追踪。标识存在只证明结构覆盖，不等于功能验收。2026-09-05 审查取代先前未限定的 Covered/可直接验证结论。

| 状态 | 含义 |
|---|---|
| FAILED | 已发现具体原型行为与目标冲突；关联 QA 场景记录证据 |
| PARTIAL | 存在界面，但所需交互/拒绝路径缺失或尚未成立 |
| SOURCE_ONLY | 源码包含相关界面，不宣称完整交互通过 |
| RUNTIME_PENDING | 需要真实进程/提供方/持久化权限实现，fixture 存在不能证明通过 |
| DEFERRED | 显式不属于 v1.0 范围 |

FAILED/PARTIAL 原型项修复后仍可能需要运行时验证。SOURCE_ONLY 不等于 PASS。[QA Report](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md) 保存详细观察，[UI Spec §14](./TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md#14-开发与原型修复交互契约) 定义目标交互。

## 1. RevC 关键对齐项

| 主题 | 目标 | 当前证据 |
|---|---|---|
| 模式/上下文与历史 | 独立维度；不可变 Turn 快照 | QA-06/QA-09：历史值变化，Backtest 入口不完整 |
| Live 权限 | 账户级 arming、全局 disarm、派发检查 | QA-04/QA-05：阻断状态与中断未一致落实 |
| LLM 路由 | CLIProxyAPI、ChatGPT OAuth/DeepSeek；fallback opt-in | QA-10：存在标签/设置，设置与恢复流程不完整 |
| 草稿/proposal | 可编辑草稿 → 新不可变身份 → 新同意 | QA-07：身份复用，编辑器流程不完整 |
| 预留/未知状态 | 按条件取得权威处置前保留容量 | QA-02/QA-03：捏造成交，缺少证据工作流 |
| 撤单 | Arming 和提供方刷新后保留 CANCEL 意图 | QA-01：disarmed 撤单进入新建订单审批 |
| 行情/FX 溯源 | 完整来源/时间/场所/授权/路径展示 | SOURCE_ONLY；生产新鲜度/FX 权威验证仍待完成 |
| 存储与范围 | SQLite + DuckDB + 文件；Parquet 可选；明确非 Live 变体 | SOURCE_ONLY；独立 HTML 没有存储/提供方执行 |
| 键盘/窄屏布局 | 焦点限制与操作保留 | QA-11/QA-12：焦点进入背景，操作被隐藏 |

## 2. Functional requirement traceability

| ID | 需求 | 优先级 | Prototype 状态 | 证据 |
|---|---|---:|---|---|
| FR-001 | 在本地运行一个固定版本的 Codex App Server | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-002 | 支持持久化的 Thread / Turn / Item 用户体验 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| FR-003 | 实时流式展示 agent 与工具活动 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Running plan/tool timeline 以 fixture 模拟流式 Item 状态。 |
| FR-004 | 恢复此前的 thread | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-12](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-12)；修复后重新验证，不以界面存在代替通过。 |
| FR-005 | 渲染结构化的金融时间线卡片 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：研究、风险、订单、错误和 provenance 使用结构化 timeline/card。 |
| FR-006 | 实现 TradeX 类型化领域工具 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-007 | 将可信控制平面与 agent 区域分离 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-008 | 实现 OS keychain 凭证存储 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-009 | 实现规范化标的模型 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-08](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-08)；修复后重新验证，不以界面存在代替通过。 |
| FR-010 | 实现提供方能力发现 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Provider 设置和 Account Detail 都展示 schema/capability review。 |
| FR-011 | 实现市场数据服务 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-012 | 实现市场数据新鲜度元数据 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Market snapshot 展示 source、provider time、TradeX receive time、venue、entitlement、age、freshness。 |
| FR-013 | 实现投资组合聚合 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-014 | 实现账户货币与 FX 归一化 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-015 | 实现 Alpaca Paper 适配器 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-016 | 实现 Trading 212 Demo/Live 适配器 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-017 | 实现 Binance Testnet/Live 现货适配器 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-018 | 实现 Bitget Demo/Live 现货适配器 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-019 | 实现 OrderProposal 模型 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-07](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-07)；修复后重新验证，不以界面存在代替通过。 |
| FR-020 | 实现确定性风控引擎 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-021 | 实现审批前校验 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-022 | 实现执行前校验 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-023 | 实现账户作用域内的原子执行预留 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-03](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-03)；修复后重新验证，不以界面存在代替通过。 |
| FR-024 | 实现一次性实时审批 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-025 | 实现特权 Order Gateway | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-026 | 实现幂等性与对账 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-03](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-03)；修复后重新验证，不以界面存在代替通过。 |
| FR-027 | 实现账户作用域内的实时布防/撤防 + 全局一键全部禁用 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| FR-028 | 实现崩溃/启动对账 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| FR-029 | 实现本地执行审计轨迹 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| FR-030 | 实现提供方限流 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-031 | 实现本地回测 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-09](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-09)；修复后重新验证，不以界面存在代替通过。 |
| FR-032 | 实现策略沙箱 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-033 | 实现 paper/demo/testnet 交易工作流 | P0 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-034 | 实现确定性错误分类与 UI 映射 | P1 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Canonical error categories 映射到可见 recovery panels。 |
| FR-035 | 实现自然语言筛选器 | P1 | FAILED | 关联场景的实际观察与验收条件见 [QA-08](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-08)；修复后重新验证，不以界面存在代替通过。 |
| FR-036 | 实现产物(artifacts) | P1 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| FR-037 | 实现策略版本管理 | P1 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-038 | 实现市场日历与公司行为 | P1 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-039 | 实现可配置的数据保留 | P1 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-040 | 实现工作区导出/导入 | P1 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| FR-041 | 增加期货支持 | P2 | DEFERRED | 显式未来范围；v1.0 不要求实现。 |
| FR-042 | 增加 A 股研究/数据集成 | P2 | DEFERRED | 显式未来范围；v1.0 不要求实现。 |
| FR-043 | 增加更多券商/交易所 | P2 | DEFERRED | 显式未来范围；v1.0 不要求实现。 |
| FR-044 | 实现完整的五步入门引导流程 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Onboarding 实现 Workspace → Providers → Model → Risk defaults → Ready。 |
| FR-045 | 实现可交互的 thread 历史与恢复导航 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-12](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-12)；修复后重新验证，不以界面存在代替通过。 |
| FR-046 | 实现上下文/账户/模型选择器 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-08](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-08)；修复后重新验证，不以界面存在代替通过。 |
| FR-047 | 实现提供方连接测试 + 权限/能力审查 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-048 | 实现提供方特定的账户详情变体 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-049 | 实现实时订单取消审批生命周期 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-01](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-01)；修复后重新验证，不以界面存在代替通过。 |
| FR-050 | 实现明确的「风控拒绝 / 券商拒绝 / 审批过期」UI 状态 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-051 | 实现启动/认证/流断开的恢复状态 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| FR-052 | 实现市价单最大授权名义金额审批 UI | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Market-order approval 展示 expected notional 与 maximum authorized spend。 |
| FR-053 | 实现完整的自然语言筛选器构建流程 | P1 | FAILED | 关联场景的实际观察与验收条件见 [QA-08](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-08)；修复后重新验证，不以界面存在代替通过。 |
| FR-054 | 实现回测运行中/失败/对比状态 | P1 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-09](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-09)；修复后重新验证，不以界面存在代替通过。 |
| FR-055 | 实现产物溯源展示 | P1 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| FR-056 | 实现无执行能力的 Ask 模式 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Ask 是独立 Agent Mode，并明确 read-only/no execution。 |
| FR-057 | 实现完整的实时审批市场快照溯源 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Live approval 展示完整 market snapshot provenance/freshness。 |
| FR-058 | 实现风控策略变更导致审批失效的生命周期 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| FR-059 | 实现预留冲突 UI/推理展示 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Reservation conflict 展示 reserved/effective capacity 和 deterministic rejection。 |
| FR-060 | 实现 Trading 212、Binance 与 Bitget 完整的 Demo/Testnet 订单生命周期变体 | P0 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：T212 Demo、Binance Testnet、Bitget Demo 复用统一 non-live lifecycle。 |
| FR-061 | 实现完整的归一化订单状态 UI 映射(含 `RESERVED`) | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-02](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-02)；修复后重新验证，不以界面存在代替通过。 |
| FR-062 | 实现市场时段 / 停牌 / 公司行为产品界面 | P1 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-063 | 实现工作区导入/恢复工作流 | P1 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Workspace Import/Restore 有 choose → validate → restore，恢复后 Live 全部 DISARMED。 |
| FR-064 | 实现完整的规范化错误修复界面 | P1 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| FR-065 | 实现投资组合 FX 溯源展示 | P1 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Portfolio 展示 FX source/route/timestamps/freshness。 |
| FR-066 | 实现完整的回测指标集(含 Sortino、盈利因子与换手率) | P1 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：Backtest 增加 Sortino、Profit Factor、Turnover。 |
| FR-067 | 实现由提供方 schema 驱动的凭证/配置表单 | P1 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：ProviderCredentialSchema 按 provider 动态定义 fields/permissions。 |
| FR-068 | 实现 LLM 提供方连接工作流(CLIProxyAPI OAuth / DeepSeek 密钥),基于 §26.3 的 schema 驱动 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| FR-069 | 实现 CLIProxyAPI sidecar 生命周期:启动/监管/健康检查(`/v1/models` 探测)/退避重启/端口冲突处理 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| FR-070 | 实现订阅配额展示与耗尽处理(provider 可报告的窗口;进行中 attempt 行为;显式切换/重试) | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| FR-071 | 实现 LLM 错误类别与恢复状态(`MODEL_UNAVAILABLE` / `QUOTA_EXCEEDED` / `OAUTH_EXPIRED`)及修复 UI | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| FR-072 | 记录每个 turn 的模型/提供方来源,并将模型切换/降级写入审计轨迹 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| FR-073 | 实现模型路由/fallback:跨 provider 自动 fallback 默认关闭、显式 opt-in、完整审计/披露且不改变 capability | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| FR-074 | 将 Agent Mode 与 Execution Context 分离并强制执行兼容矩阵 | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-09](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-09)；修复后重新验证，不以界面存在代替通过。 |
| FR-075 | 持久化不可变的每 turn mode/account/execution/model/provider/context 快照 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| FR-076 | 为 `UNKNOWN_RECONCILING` 实现基于证据的 Manual Resolution;禁止盲目释放 reservation | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-03](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-03)；修复后重新验证，不以界面存在代替通过。 |
| FR-077 | 实现 TimeService 的 clock-skew/freshness/TTL Live 安全门 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-078 | Live readiness 前阻断/审查危险 provider 权限(withdrawal/transfer/custody/margin/leverage) | P0 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-079 | 实现 FX/stablecoin 估值溯源与 depeg/quality 处理 | P1 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| FR-080 | 实现可编辑 OrderDraft → 不可变 OrderProposal 的重新生成语义 | P0 | FAILED | 关联场景的实际观察与验收条件见 [QA-07](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-07)；修复后重新验证，不以界面存在代替通过。 |

## 3. Acceptance-criterion traceability

| ID | 验收标准 | Prototype 状态 | Traceability 说明 |
|---|---|---|---|
| AC-001 | 用户可以创建 TradeX thread,并在完整 turn 结束前看到流式展示的 agent 项。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-002 | 用户可以关闭并重新打开 TradeX,恢复已持久化的 thread。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-003 | 工具调用展示名称、状态、持续时间与结构化摘要。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-039 | 用户可以从 composer 检查并恢复最近的 Thread,管理附加的上下文/账户/模型。 | FAILED | 关联场景的实际观察与验收条件见 [QA-08](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-08), [QA-12](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-12)；修复后重新验证，不以界面存在代替通过。 |
| AC-041 | Ask 模式提供轻量分析/只读上下文,不暴露 paper 或 live 执行能力。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-004 | Demo 与 live 账户表示为独立的连接。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-005 | 账户连接时 TradeX 校验提供方能力。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-006 | 若账户对账不完整,实时执行被禁用。 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| AC-036 | 提供方连接界面在连接被视为就绪前,展示检测到的权限与能力集合。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-038 | 完整的首次运行工作流覆盖 Workspace → Providers → LLM (Model) → Risk Defaults → Ready,其中 Model 步骤配置 LLM 提供方:CLIProxyAPI sidecar 状态(运行中/OAuth 已授权)和/或经探测 + 测试推理校验的 DeepSeek 密钥(§26.3)。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| AC-055 | 若无至少一个可用 LLM 提供方,入门流程无法到达 Ready;已停止、端口冲突或未授权的 sidecar 会阻断 Ready 并提供修复指引。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| AC-051 | 提供方连接 UI 由提供方凭证/能力 schema 渲染,不假设每个提供方使用相同的凭证字段。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-007 | Agent 可在单个 thread 中结合市场数据、投资组合状态与研究。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-008 | 实时审批中展示的每个市场快照均含来源、提供方时间戳、TradeX 接收时间戳、场所、实时/延迟授权与新鲜度状态。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-040 | 自然语言筛选器在执行前展示已解析的结构化筛选条件,并渲染缩减后的候选集。 | FAILED | 关联场景的实际观察与验收条件见 [QA-08](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-08)；修复后重新验证，不以界面存在代替通过。 |
| AC-049 | 股票标的详情暴露相关的市场时段/公司行为状态,且对不支持的已收盘/停牌执行确定性地阻止。 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| AC-053 | 跨账户投资组合值展示工作区基础货币,以及归一化值的 FX 来源/时间戳/新鲜度。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-009 | 用户可提交 Alpaca Paper 订单并查看结果券商状态。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-010 | 用户可在 Trading 212、Binance 与 Bitget 上执行受支持的 Demo/Testnet 订单工作流(在可用时),并在确认/成交状态中带有显式环境标签。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-011 | 无有效 TradeX 金融审批,实时订单无法提交。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-012 | 通用 Codex 审批无法授权实时金融执行。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-013 | 变更标的、方向、数量、账户、订单类型、价格或有效期使先前审批失效。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-014 | 相关风控策略变更使受影响的待审批项失效;策略弱化还额外撤防受影响的实时账户。 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| AC-015 | 陈旧的市场快照阻止执行,直至刷新。 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| AC-016 | 提交前立即重新校验风控状态。 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| AC-017 | 两个并发 thread 无法预留相同的现金或敞口。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-018 | Agent 无法修改风控策略。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-019 | Agent 无法直接访问特权 Order Gateway。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-020 | 模型绝不接收券商密钥。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-030 | 选择 Trade 模式或 Live Execution Context 不布防实时执行。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-031 | 在 DISARMED 状态下请求实时订单,需在展示交易审批前执行单独的显式 Arm 操作。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-032 | 订单监控期间展示的标的、账户、方向、数量与订单类型,与用户批准的不可变交易一致。 | FAILED | 关联场景的实际观察与验收条件见 [QA-07](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-07)；修复后重新验证，不以界面存在代替通过。 |
| AC-033 | 未结实时订单唯有在券商状态刷新且经交易特定的用户审批后,方可进入取消流程;UI 呈现 `CANCEL_PENDING` 与 `CANCELLED`。 | FAILED | 关联场景的实际观察与验收条件见 [QA-01](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-01)；修复后重新验证，不以界面存在代替通过。 |
| AC-034 | UI 呈现确定性的 `RISK_REJECTED`、券商 `REJECTED`、审批 `EXPIRED` 与 `UNKNOWN_RECONCILING` 状态,且不暗示执行成功。 | FAILED | 关联场景的实际观察与验收条件见 [QA-02](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-02)；修复后重新验证，不以界面存在代替通过。 |
| AC-035 | 市价单审批展示预期支出、最大授权支出、买卖价差、报价年龄以及(在可用时)费用/滑点估计。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-042 | 实时布防为账户作用域:布防 Trading 212 不会布防 Binance 或 Bitget;全局「全部禁用」撤防所有实时账户。 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| AC-043 | 每个实时审批暴露由 AC-008 定义的完整市场快照溯源。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-044 | 保存风控策略使受影响的待审批项失效并传达失效原因;策略弱化撤防受影响的账户。 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| AC-045 | 在较早提案预留现金/敞口后,第二个并发提案按缩减后的账户容量评估,并在容量不足时被确定性拒绝。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-046 | Trading 212 Demo、Binance Testnet 与 Bitget Demo/Testnet 兼容工作流在提案、确认、成交/更新与账户持仓刷新全程保持明确的非实时环境标识。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-021 | 除非券商状态确认成交,否则券商确认绝不显示为成交。 | FAILED | 关联场景的实际观察与验收条件见 [QA-02](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-02)；修复后重新验证，不以界面存在代替通过。 |
| AC-022 | 模糊的提交进入 `UNKNOWN_RECONCILING`。 | FAILED | 关联场景的实际观察与验收条件见 [QA-02](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-02)；修复后重新验证，不以界面存在代替通过。 |
| AC-023 | TradeX 绝不在模糊超时后盲目重试非幂等的提供方订单 POST。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-024 | 丢失私有流连接触发降级状态与对账。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-025 | 重启应用后,在启用新实时提交前先对账未结实时订单。 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| AC-026 | 每个实时订单具备从提案 → 风控评估 → 审批 → 预留 → 执行尝试 → 提供方状态的持久链条。 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| AC-037 | 认证失败与私有流断开会禁用新实时执行,直至账户健康恢复。 | FAILED | 关联场景的实际观察与验收条件见 [QA-05](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-05)；修复后重新验证，不以界面存在代替通过。 |
| AC-050 | 规范化错误类别渲染适当的修复措施,且不引入冲突的机器状态名称。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| AC-027 | 策略代码无法访问券商凭证。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-028 | 外部研究内容无法授权交易。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-029 | 券商密钥绝不出现在日志、产物、thread 历史、SQLite、DuckDB 或 Parquet 中。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-047 | 工作区导入/恢复校验归档/schema 元数据,不导入券商密钥,且在重新启用实时执行前要求对账。 | RUNTIME_PENDING | 按 PRD/ARD 验证真实进程、提供方或持久化行为；独立 fixture 不证明本项完成。 |
| AC-048 | 回测结果包含收益、Sharpe、Sortino、最大回撤、胜率、盈利因子、换手率、权益曲线、交易清单与可复现清单。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-09](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-09)；修复后重新验证，不以界面存在代替通过。 |
| AC-052 | 核心交互界面可通过键盘访问且具可见焦点,通用 `Enter` 提交无法批准实时订单或取消。 | FAILED | 关联场景的实际观察与验收条件见 [QA-11](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-11)；修复后重新验证，不以界面存在代替通过。 |
| AC-054 | 窄桌面/平板级布局保留实时安全信息;v1.0 不提供原生移动端实时执行客户端。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-12](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-12)；修复后重新验证，不以界面存在代替通过。 |
| AC-056 | 当 CLIProxyAPI sidecar 停止、端口冲突或未授权时,agent turn 暂停并提供修复指引,而审批、执行与对账继续运行(仅对 LLM 故障关闭)。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| AC-057 | 订阅配额耗尽与 OAuth 过期呈现 `QUOTA_EXCEEDED` / `OAUTH_EXPIRED`,并提供 retry/re-login/显式 provider switch。只有用户显式开启时才允许跨 provider 自动 fallback;fallback 必须明显披露并写入审计,且绝不改变 capability 或绕过审批。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-10](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-10)；修复后重新验证，不以界面存在代替通过。 |
| AC-058 | 每个 turn 记录产生/失败该 turn 的 model/provider 与全部 provider attempt;模型切换/降级写入审计轨迹(§57)。 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| AC-059 | Agent Mode 与 Execution Context 作为独立维度存储与呈现;Ask/Research + Live account 始终只读,Backtest 不能调用当前市场 broker execution,Trade + Live context 仍需 arming + approval。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-09](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-09)；修复后重新验证，不以界面存在代替通过。 |
| AC-060 | 每个启动的 turn 持久化 Agent Mode、Execution Context、所选 account、capability、model/provider 与 attached context reference/hash 的不可变快照。 | FAILED | 关联场景的实际观察与验收条件见 [QA-06](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-06)；修复后重新验证，不以界面存在代替通过。 |
| AC-061 | 模糊 Live submission 超过自动对账窗口后 reservation 继续冻结,仅暴露 §23 的 evidence-based Manual Resolution;不存在可恢复 Live readiness 的通用 release 操作。 | FAILED | 关联场景的实际观察与验收条件见 [QA-03](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-03)；修复后重新验证，不以界面存在代替通过。 |
| AC-062 | Live broker credential 检测到 withdrawal/transfer/custody 权限时阻止 execution-ready 直至移除;无法 introspect 的权限持续标记 `UNVERIFIED`。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| AC-063 | 当 TimeService 报告不可接受 clock uncertainty 时,quote age、approval expiry 与 reconciliation timer 对 Live 权限判断必须 fail closed。 | FAILED | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |
| AC-064 | Local Paper 明确标记为 TradeX simulation,不得与 provider-hosted Paper/Demo/Testnet 或 Live 混淆。 | SOURCE_ONLY | 仅源码界面存在性记录；完整交互仍待验证。原记录：对应 UI/状态已在 RevC prototype 与 UI Spec 中实现；涉及真实 broker、持久化或运行时隔离的部分按状态列保留为实现阶段验收。 |
| AC-065 | 编辑已生成订单会创建新的不可变 `OrderProposal` identity,并使旧 proposal 的 approval 失效。 | FAILED | 关联场景的实际观察与验收条件见 [QA-07](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-07)；修复后重新验证，不以界面存在代替通过。 |
| AC-066 | 涉及 stablecoin/FX 的跨账户估值展示 source/path/timestamps/freshness 与 quality/depeg 状态;不可靠转换不得静默驱动 Live 风控。 | PARTIAL | 关联场景的实际观察与验收条件见 [QA-04](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md#qa-04)；修复后重新验证，不以界面存在代替通过。 |


## 4. 页面覆盖概览

| 区域 | 页面 | 状态 | 证据 |
|---|---|---|---|
| A | Onboarding / 模型设置 | PARTIAL | QA-10 |
| B | Thread 工作区 / 历史 / 溯源 | FAILED | QA-06, QA-08, QA-09, QA-12 |
| C | Markets / 筛选器 / 标的 | FAILED | QA-04, QA-08 |
| D | Watchlists | SOURCE_ONLY | 仅源码存在性 |
| E | 账户 / 组合 / 就绪 | PARTIAL | QA-04, QA-05 |
| F | Live 审批 / 撤单 / 处置 | FAILED | QA-01–QA-07 |
| G | 非 Live 执行变体 | SOURCE_ONLY | 仅源码存在性 |
| H | 策略 / 回测 | PARTIAL | QA-09 |
| I | 产物 / 溯源 | FAILED | QA-06 |
| J | 设置 / 模型恢复 / 窄屏操作 | PARTIAL | QA-10, QA-12 |
| K | 恢复 / 时钟 / 未知提交 | FAILED | QA-03–QA-05, QA-10 |

## 5. 边界与更新规则

- 独立原型不连接真实 Codex、CLIProxyAPI、券商/交易所或行情后端，也不执行真实凭据/归档/数据库 I/O。
- Futures、A-share 和额外提供方属于既有未来范围。
- 这些运行时边界与上方已发现的交互缺陷分别记录，不能用 fixture 限制解释 FAILED。
- 更新任一状态时，关联新的源码版本与可复现证据，同步英文表；不得仅因规范补充而升级为通过。
