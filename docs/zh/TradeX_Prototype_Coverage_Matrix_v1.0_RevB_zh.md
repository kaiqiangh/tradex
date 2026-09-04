# TradeX PRD / 原型需求可追溯性矩阵

**Version:** 1.0 Final — Revision B  
**Date:** 2026-09-04  
**PRD:** `TradeX_PRD_v1.0_RevB.md`  
**UI Specification:** `TradeX_UI_Prototype_Spec_v1.0_RevB.md`  
**当前评估的原型:** `docs/prototype/` @ git tag `prototype-v1.0-reva-baseline`(commit `5af4ee5`)以及 `867ec00` 中的 UI 一致性修复;在 `867ec00` 处重新验证:`node --check` PASS,47/47 唯一 `onclick` 引用已定义。RevB 的 LLM 提供商需求(FR-068–FR-073)为新增目标需求,尚未在原型中实现。

## 1. 状态图例

| Status | 含义 |
|---|---|
| **Covered** | 当前原型以原型保真度可见/可点击地展示了该需求。 |
| **Partial** | 展示了部分必需行为,但关键细节缺失。 |
| **Visual-only** | 概念被提及/渲染,但未作为完整的交互状态演示。 |
| **Not covered** | 当前原型中缺失所需的目标界面/流程。 |
| **N/A — implementation** | 需求属于架构/运行时/安全层面,无法由独立的 UI 原型证明。 |
| **N/A — future** | 明确在 v1.0 MVP 原型范围之外。 |
| **QA pending** | UI/源码已存在,但所需的视觉/非功能性/无障碍验证尚待完成。 |
| **Covered visually** | `Covered` 的变体,用于 §3/§4,其中证据仅为视觉/交互形式,无状态断言。 |
| **Visual-only / implementation** | §3 中使用的混合状态:需求的一部分在原型中渲染,一部分仅属实现层面。 |

**重要:** `Covered` 仅表示原型覆盖。它**不**意味着生产实现满足该需求。

**QA ↔ 矩阵状态映射(由 `TradeX_Prototype_QA_Report_v1.0_RevA.md` 使用):** `PASS` ↔ `Covered`;`PARTIAL` ↔ `Partial`;`NOT REPRESENTED` / `FAIL / GAP` ↔ `Not covered`;`NOT TESTED` ↔ `Partial` 或 `QA pending`(视上下文而定);`QA PENDING` ↔ `QA pending`;`NOT APPLICABLE` ↔ `N/A — implementation`。

---

## 2. 功能性需求可追溯性

| ID | 需求摘要 | UI 规范 | 当前原型证据 | 原型状态 | QA / 生产备注 |
|---|---|---|---|---|---|
| FR-001 | 固定的 Codex App Server | §12 交接 | 无 | N/A — implementation | 需运行时集成 |
| FR-002 | 持久化线程/回合/条目 UX | B2/B5 | 线程历史 + 时间线 | Covered | 生产持久化未证明 |
| FR-003 | 流式代理/工具活动 | B5/B6 | 运行/工具卡片模拟 | Visual-only | 真实运行时流式未证明 |
| FR-004 | 恢复线程 | B2 | 近期线程导航 | Covered | 持久化重启行为需实现 QA |
| FR-005 | 金融时间线卡片 | B5/F3 | 工具/订单时间线卡片 | Covered | 原型层面无 |
| FR-006 | 类型化领域工具 | B5 | 仅命名工具卡片 | Visual-only | 工具契约属实现工作 |
| FR-007 | 可信控制平面隔离 | §12 | 仅安全副本 | N/A — implementation | 需架构/安全测试 |
| FR-008 | OS 钥匙串凭据 | A3 | 钥匙串/可信边界副本 | Visual-only | 无法在原型中证明存储隔离 |
| FR-009 | 规范标的模型 | C4/C5 | 规范化 AAPL/BTC 标签 | Visual-only | 需实现领域模型 |
| FR-010 | 能力发现 | A3/E2 | 权限/能力结果 UI | Partial | 提供商模式/能力契约非真实 |
| FR-011 | 市场数据服务 | C1/C4/C5 | 测试桩市场数据 | Visual-only | 需真实提供商集成 |
| FR-012 | 鲜度元数据 | C4/C5/F2 | 显示报价时间 | Partial | 实时审批中缺失来源/接收/授权 |
| FR-013 | 组合聚合 | E3 | 跨账户组合 | Covered | 仅测试桩 |
| FR-014 | FX 规范化 | E3 | EUR 规范化测试桩 | Partial | 缺失 FX 来源/时间戳/鲜度 |
| FR-015 | Alpaca Paper 适配器 | G1/G2 | 仅 Paper UI | Visual-only | 适配器未实现 |
| FR-016 | Trading 212 Demo/Live 适配器 | A2/E1/F2 | 账户 + 实时订单 UI | Visual-only | 适配器未实现 |
| FR-017 | Binance Testnet/Live 适配器 | A2/E1/F4 | 账户 + 实时市价单 UI | Visual-only | 适配器未实现 |
| FR-018 | Bitget Demo/Live 适配器 | A2/E1 | 账户界面 | Visual-only | 未演示订单生命周期 |
| FR-019 | OrderProposal 模型 | F2/F4 | 提案卡片 | Covered | 不可变领域模型未证明 |
| FR-020 | 确定性风险引擎 | F2/F7/J2 | 风险检查/拒绝 | Visual-only | 确定性引擎未实现 |
| FR-021 | 审批前校验 | F2/F4 | 可见检查 | Covered | 仅测试桩行为 |
| FR-022 | 执行前校验 | F3 | 时间线阶段 | Visual-only | 实际重新校验逻辑未证明 |
| FR-023 | 原子预留 | F11 | 风险页提及预留 | Visual-only | 无交互式竞争线程流程 |
| FR-024 | 一次性实时审批 | F2/F5/F6 | 过期/失效流程 | Partial | 一次性令牌语义未做状态断言 |
| FR-025 | 特权订单网关 | F3/§12 | 仅描述 | N/A — implementation | 需安全边界 |
| FR-026 | 幂等性/对账 | F9/K2 | 模糊 + 恢复 UI | Partial | 提供商幂等行为非真实 |
| FR-027 | 账户级布防 | F1 | 当前原型使用全局 `liveArmed` 布尔值 | Partial | 布防非账户级 |
| FR-028 | 崩溃/启动对账 | K2 | 启动恢复界面 | Visual-only | 重启时序/状态非真实 |
| FR-029 | 本地执行审计追踪 | F3/I3 | 审计/溯源 UI | Visual-only | 持久化未证明 |
| FR-030 | 提供商限流 | K5 | 优先级降级 UI | Visual-only | 真实限流器未实现 |
| FR-031 | 本地回测 | H3-H6 | 模拟运行/结果 | Visual-only | 引擎未实现 |
| FR-032 | 策略沙箱 | H2 | 受限能力 UI | Visual-only | 沙箱隔离未证明 |
| FR-033 | Paper/Demo/Testnet 交易 | G | 仅 Alpaca paper 流程 | Partial | 缺失 T212 Demo/Binance Testnet/Bitget Demo 订单生命周期 |
| FR-034 | 错误分类法 | K6 | 子集:auth/network/rate/stream/risk/reject/ambiguous | Partial | 多个规范类别缺失 |
| FR-035 | 自然语言筛选器 | C2/C3 | 构建器/结果流程 | Covered | 测试桩查询 |
| FR-036 | 产物 | I1-I4 | 库/详情/导出 | Covered | 持久化/导出实现非真实 |
| FR-037 | 策略版本管理 | H1/H6 | 版本 + 对比 | Covered | 仅测试桩 |
| FR-038 | 市场日历/公司行为 | C6 | 当前原型无专用界面 | Not covered | 需添加市场收盘/停牌/行为状态 |
| FR-039 | 可配置保留 | J3 | 保留设置 UI | Covered | 存储行为未证明 |
| FR-040 | 工作区导出/导入 | J3/J4 | 存在导出/备份 | Partial | 缺失导入/恢复 |
| FR-041 | 期货 | future | 无 | N/A — future | P2 |
| FR-042 | A 股 | future | 无 | N/A — future | P2 |
| FR-043 | 其他券商 | future | 无 | N/A — future | P2 |
| FR-044 | 五步引导 | A1-A6 | 完整引导路径 | Covered | 原型层面无 |
| FR-045 | 线程历史/恢复 | B2 | 近期线程列表 | Covered | 会话外持久化未证明 |
| FR-046 | 上下文/账户/模型选择器 | B4 | 可点击选择器 | Covered | 原型层面无 |
| FR-047 | 提供商测试/权限审查 | A3 | 通用连接弹窗 | Covered | 凭据模式非提供商驱动 |
| FR-048 | 提供商特定账户详情 | E2 | 选中账户改变详情 | Partial | 所需的持仓/对账/凭据健康度未完整 |
| FR-049 | 实时取消审批 | F10 | 取消审批/待定/已取消 | Covered | 原型层面无 |
| FR-050 | 风险/拒绝/过期 UI | F5-F8 | 状态存在 | Covered | 仅文档措辞,无代码缺口;规范命名已在 RevA 中规范化 |
| FR-051 | 启动/认证/流恢复 | K1-K4 | 恢复状态 | Covered | 操作行为已模拟 |
| FR-052 | 市价单最大授权名义金额 | F4 | Binance BTC 审批 | Covered | 来源/时间戳不完整 |
| FR-053 | 完整筛选器构建器 | C2/C3 | 解析 → 结果 | Covered | 原型层面无 |
| FR-054 | 回测运行/失败/对比 | H3/H4/H6 | 状态存在 | Covered | 原型层面无 |
| FR-055 | 产物溯源 | I3 | 溯源弹窗 | Covered | 原型层面无 |
| FR-056 | Ask 模式 | B3 | cycleMode 省略 Ask | Not covered | 源码审计已确认 |
| FR-057 | 实时审批市场溯源 | F2/F4 | 仅报价 + 时间 | Partial | 缺失来源/提供商时间戳/接收/授权 |
| FR-058 | 风险策略失效生命周期 | F12/J2 | 保存解除全局实时 | Partial | 未建模显式待审批失效 |
| FR-059 | 预留冲突界面 | F11 | 无交互状态 | Not covered | 源码审计已确认 |
| FR-060 | Demo/Testnet 订单变体 | G3 | 仅账户变体 | Not covered | 无完整订单生命周期变体 |
| FR-061 | 完整订单状态 UI 映射(含 RESERVED) | §6 | 提交/成交/取消状态 | Partial | `RESERVED` 未作为状态可见呈现 |
| FR-062 | 市场时段/停牌/公司行为 UI | C6 | 缺失 | Not covered | 原型层面无 |
| FR-063 | 工作区导入/恢复 | J4 | 缺失 | Not covered | 源码审计已确认 |
| FR-064 | 完整错误修复界面 | K6 | 仅子集 | Partial | 缺失 permission/unsupported/closed/halted/invalid/funds/stale/reconcile/internal |
| FR-065 | 组合 FX 溯源 | E3 | 仅 EUR 测试桩 | Not covered | 无 FX 来源/时间戳 |
| FR-066 | 完整回测指标 | H5 | 收益/Sharpe/回撤/胜率 | Partial | 缺失 Sortino/profit factor/turnover |
| FR-067 | 提供商模式驱动的凭据表单 | A3 | 通用 API key/secret 表单 | Not covered | 缺失提供商表单模式 |
| FR-068 | LLM 提供商连接工作流(CLIProxyAPI OAuth / DeepSeek key) | A4/J1 | 缺失 | Not covered | RevB 需求;A4/J1 重设计待定 |
| FR-069 | CLIProxyAPI sidecar 生命周期 + 健康 UI | A4/J1 | 运行时卡片显示静态 "Codex runtime connected" | Not covered | sidecar 监管属实现工作 |
| FR-070 | 订阅配额显示 / 耗尽处理 | §4.4/§16.3 | 缺失 | Not covered | RevB 需求 |
| FR-071 | LLM 错误类别(MODEL_UNAVAILABLE/QUOTA_EXCEEDED/OAUTH_EXPIRED) | K6 | 规范错误变体中缺失 | Not covered | 分类法在 RevB 中扩展 |
| FR-072 | 每回合模型/提供商溯源 + 审计 | I3/B5 | 模型选择器存在;每回合溯源未渲染 | Not covered | RevB 需求 |
| FR-073 | 模型回退/路由策略界面 | §4.4 | 无回退 UI 与策略一致 | Not covered | 路由策略属 P1 实现 |

**源码审计附录(2026-09-04,基线 tag `prototype-v1.0-reva-baseline`):** PRD §11.6 要求六种工具/回合状态;基线原型实现了四种(`Tool: Running / Completed / Failed`、`Turn: Cancelled`)且缺失 `Tool: Retrying` 和 `Turn: Interrupted`。PRD §11.1 在上下文面板中列出 `Chart`;基线原型的上下文面板无图表。修订版 A 对此两类缺口均无专用 AC 行 —— 二者均在此处跟踪以供下个修订。

---

## 3. 非功能性 / 横切可追溯性

| ID | 原型状态 | QA 状态 / 备注 |
|---|---|---|
| NFR-001 | N/A — implementation | NOT TESTED:运行时事件延迟 |
| NFR-002 | N/A — implementation | NOT TESTED:工具渲染延迟 |
| NFR-003 | Visual-only | NOT TESTED:重启到对账时序 |
| NFR-004 | N/A — implementation | NOT TESTED:使用真实凭据/日志管道 |
| NFR-005 | Visual-only | 安全流程冒烟测试通过;生产授权不可测 |
| NFR-006 | Visual-only | 模糊重试 UI 已覆盖;提供商故障注入 NOT TESTED |
| NFR-007 | Covered visually | 模糊状态已显示;生产不变式 NOT TESTED |
| NFR-008 | Visual-only | 启动恢复已显示;真实券商对账 NOT TESTED |
| NFR-009 | Partial | 审批溯源字段不完整 |
| NFR-010 | Covered visually | 认证/流/启动状态在原型中禁用实时 |
| NFR-011 | Partial | 已显示过期/时效;通用实质编辑状态断言缺失 |
| NFR-012 | Partial | 风险保存解除布防;待审批失效未断言 |
| NFR-013 | N/A — implementation | NOT TESTED |
| NFR-014 | N/A — implementation | NOT TESTED |
| NFR-015 | QA pending | 键盘/焦点/实时 Enter 行为 NOT TESTED |
| NFR-016 | QA pending | 响应式 CSS 存在;浏览器视觉审查待定 |
| SEC-001 | N/A — implementation | 模型/钥匙串隔离需生产安全测试 |
| SEC-002 | N/A — implementation | 代理/网关隔离需进程边界测试 |
| SEC-003 | N/A — implementation | 通用 Codex 审批隔离需集成/安全测试 |
| SEC-004 | Visual-only / implementation | 沙箱限制已渲染;实际沙箱边界未测试 |
| SEC-005 | N/A — implementation | 提示注入授权边界需对抗性测试 |
| SEC-006 | Partial | 审批 UX 为提案特定;加密/一次性授权未证明 |
| DATA-001 | Visual-only / implementation | 规范化标签已显示;规范 ID 领域模型未证明 |
| DATA-002 | N/A — implementation | 十进制安全运算需单元/属性测试 |
| DATA-003 | Partial | 已显示报价时间;完整溯源不完整 |
| DATA-004 | Not covered | FX 来源/时间戳/鲜度缺失 |
| DATA-005 | Visual-only / implementation | 授权/保留策略为文档性,未强制执行 |
| DATA-006 | Covered visually | 可复现性清单已显示;持久化完整性未证明 |
| OPS-001 | Visual-only / implementation | UI 状态代理授权;真实券商对账未证明 |
| OPS-002 | Covered visually | 模糊重试在 UI 中已阻止;仍需提供商故障注入 |
| OPS-003 | Covered visually | 启动/恢复恢复状态已显示;需真实运行时测试 |
| OPS-004 | Visual-only | 限流优先级界面已显示;调度器未实现 |
| OPS-005 | Not covered | 无竞争预留交互状态 |
| OPS-006 | N/A — implementation | 需打包/代码签名管道 |
| OPS-007 | N/A — implementation | 自动更新 + 崩溃报告需生产运行时 |
| SEC-007 | N/A — implementation | 单模型退出(127.0.0.1:8317)需生产网络测试 |
| SEC-008 | N/A — implementation | sidecar 凭据隔离需进程边界测试 |
| UX-001 | Covered | 源码流程确认选择 Live 不会布防 |
| UX-002 | Partial | 当前原型布防为全局,非账户级 |
| UX-003 | Partial | 不可变订单已显示,溯源不完整 |
| UX-004 | Covered | 显式环境标签存在 |
| UX-005 | QA pending | 键盘安全性未测试 |
| UX-006 | Partial | 当前 UI 超出规范 IA 推广多个设置入口 |
| UX-007 | QA pending | 响应式源码存在;视觉审查待定 |

---

## 4. 验收标准可追溯性

| AC | 原型状态 | 证据 / 缺口 |
|---|---|---|
| AC-001 | Visual-only | 模拟的流式条目状态 |
| AC-002 | Partial | 线程恢复存在;进程重启持久化未证明 |
| AC-003 | Covered | 工具卡片 |
| AC-004 | Covered | 独立环境 |
| AC-005 | Partial | 能力审查 UI,无真实提供商校验 |
| AC-006 | Covered visually | 启动/认证/流状态禁用实时 |
| AC-007 | Covered | 研究 + 组合测试桩 |
| AC-008 | Partial | 审批中缺失来源/完整时间戳/授权 |
| AC-009 | Covered visually | Alpaca Paper 流程 |
| AC-010 | Partial | 账户变体存在;缺失 T212/Binance/Bitget Demo/Testnet 订单生命周期 |
| AC-011 | Covered visually | 审批门控 |
| AC-012 | N/A — implementation | 通用 Codex 授权隔离在独立原型中不可测 |
| AC-013 | Partial | 过期/刷新变体;缺失任意提案变更断言 |
| AC-014 | Partial | 风险保存解除布防;未建模待审批失效 |
| AC-015 | Covered | 过期审批已阻止 |
| AC-016 | Visual-only | 仅执行前检查时间线 |
| AC-017 | Not covered | 无竞争预留流程 |
| AC-018 | Visual-only | UI 显示代理无法更改策略 |
| AC-019 | N/A — implementation | 特权网关边界不可测 |
| AC-020 | N/A — implementation | 真实模型/密钥边界不可测 |
| AC-021 | Covered | ACCEPTED 与成交分离 |
| AC-022 | Covered | UNKNOWN_RECONCILING |
| AC-023 | Covered visually | 无重试 UI |
| AC-024 | Covered visually | 流断开 + REST 回退 |
| AC-025 | Covered visually | 启动对账 |
| AC-026 | Partial | 审计链已显示;持久化持久性未证明 |
| AC-027 | Visual-only | 沙箱限制已渲染 |
| AC-028 | N/A — implementation | 需提示注入安全测试 |
| AC-029 | N/A — implementation | 需使用真实管道的密钥扫描 |
| AC-030 | Covered | Live 模式不布防 |
| AC-031 | Covered | 显式布防弹窗 |
| AC-032 | Covered | AAPL/T212 身份连续性已修复 |
| AC-033 | Covered | 取消审批/待定/已取消 |
| AC-034 | Covered | 风险/拒绝/过期/模糊界面 |
| AC-035 | Covered | 市场最大支出 + 点差/时间/费用 |
| AC-036 | Covered visually | 权限/能力审查 |
| AC-037 | Covered visually | 认证/流禁用实时 |
| AC-038 | Covered | 五步引导 |
| AC-039 | Covered | 线程 + 选择器 |
| AC-040 | Covered | 已解析筛选器 |
| AC-041 | Not covered | 缺失 Ask 模式 |
| AC-042 | Partial | 布防为全局而非每账户 |
| AC-043 | Partial | 溯源不完整 |
| AC-044 | Partial | 策略保存未建模审批失效 |
| AC-045 | Not covered | 缺失预留冲突 |
| AC-046 | Partial | Demo/Testnet 账户存在,但缺失完整提供商订单生命周期变体 |
| AC-047 | Not covered | 缺失工作区导入 |
| AC-048 | Partial | 缺失 Sortino/profit factor/turnover |
| AC-049 | Not covered | 缺失日历/停牌/公司行为界面 |
| AC-050 | Partial | 规范错误变体不完整 |
| AC-051 | Not covered | 缺失提供商模式驱动表单 |
| AC-052 | QA pending | 键盘安全性未测试 |
| AC-053 | Not covered | 缺失 FX 溯源 |
| AC-054 | QA pending | 响应式视觉签核待定 |
| AC-055 | Not covered | 缺失引导 LLM 门控(无可用 LLM 提供商则无 Ready) |
| AC-056 | Not covered | 无法表示 sidecar fail-closed 行为 |
| AC-057 | Not covered | 缺失配额/OAuth 回退界面 |
| AC-058 | Not covered | 未渲染每回合模型/提供商溯源 |

---

## 5. 待定决策

待定的产品决策位于 PRD §72(`OD-001`–`OD-016`,其中 `OD-009` 与 `OD-015` 已在 RevB 解决),且为唯一事实来源。

> 维护说明(编辑者):请勿在本矩阵内重复或缩短该列表;仅引用 ID。

---

## 6. 原型发布门槛

当前原型仍适合产品方向评审,但文档基线不再将每个已呈现概念标记为 `Complete`。

在将原型本身称为相对于修订版 A **覆盖完整** 之前,至少关闭:

1. Ask 模式;
2. 账户级实时布防;
3. 完整实时审批市场数据溯源;
4. 显式风险策略审批失效;
5. 预留冲突状态;
6. Trading 212 Demo / Binance Testnet / Bitget Demo 订单生命周期变体;
7. `RESERVED` 状态界面;
8. 市场日历 / 停牌 / 公司行为;
9. 工作区导入/恢复;
10. 完整错误修复变体;
11. 组合 FX 溯源;
12. Sortino / profit factor / turnover;
13. 提供商模式驱动凭据表单;
14. 桌面 + 窄屏视觉 QA 及键盘/无障碍 QA;
15. 按 RevB 的 LLM 提供商界面(FR-068–FR-072):引导 LLM 门控、sidecar 健康 UI、配额/回退状态、每回合模型溯源。
