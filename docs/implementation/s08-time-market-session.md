# S08 可信时间、市场时段与公司行为规范

日期：2026-09-14
前置：S06 数据源授权目录、S07 canonical market/Watchlists
状态：已拆为 #30（可信时间门禁）与 #31（市场时段、停牌与公司行为）；#30 已完成，#31 已实现并待审查

## Problem Statement

TradeX 已能用 canonical instrument ID 浏览 Markets 和管理 Watchlists，但当前控制面没有统一的可信时间语义，也没有把交易日历、半日、延长时段、停牌和公司行为接入 instrument detail。市场详情只能说明行情源是否可用，不能说明当前市场状态、下一个开收盘边界或历史调整状态。原型中 CLOSED 和 UNTRUSTED time 仍能走到 RESERVED，QA-04 将其列为失败路径。

这会让 quote age、approval TTL、reconciliation deadline 和审计排序依赖各处的 wall clock；系统休眠、恢复或时钟跳变后可能继续把旧时间当成可信时间。没有可信的 session/corporate-action 状态时，模型也可能错误推断 OPEN、可交易或未调整历史。S06 的 OD-005 仍是 BLOCKED_EXTERNAL，任何 fixture 只能展示状态契约，不能冒充真实日历或实时资格。

## Solution

建立一个由 Rust Control Plane 持有的轻量 TimeService 和 market-state service，并以最高现有 seam 供未来风险/审批/派发调用：

- TimeService 同时记录 wall-clock 与 monotonic reading，按单一配置容差比较两者的经过时间，识别 material wall-clock jump、resume discontinuity 和 provider/server offset 失真。它返回明确的 TRUSTED、CLOCK_UNCERTAIN、STALE 状态、观测时间、单调时钟读数、offset 和 remediation reason。
- 工作区打开、进程重启或显式 resume 会重新初始化时间信任；重新采样前时间状态保持阻塞。time.status 读取当前状态，time.revalidate 只建立新的基准并在样本正常时恢复信任。时间状态是进程会话事实，不写 financial DomainProjection、outbox 或普通工作区文件。
- market-state service 为 canonical instrument 返回 venue/source、OPEN、CLOSED、EXTENDED_HOURS、HALTED、MAINTENANCE、SUSPENDED、DEGRADED 或 UNKNOWN，以及 next open/close、calendar version、observed time 和 sanitized reason。Equity 使用明确的 holiday/half-day/regular/extended/halt fixture seam；Crypto 使用 venue maintenance/suspension/degraded seam。OD-005 未授权时返回 UNAVAILABLE/BLOCKED_EXTERNAL 和未知状态，不伪造当前开市。
- Corporate actions 返回受限的 split、dividend、symbol change、delisting 记录、effective time、公告/来源标识和历史 adjustmentStatus。没有可信 action source 时显示 UNKNOWN/UNAVAILABLE；历史数据不得因为 fixture 自动变成已调整。
- MarketDetail 携带 market-state/corporate-action panel 数据，C4/C6 在同一 canonical detail 中显示；K6 使用同一 TimeService 状态和 ErrorRecoveryPanel。MARKET_CLOSED、INSTRUMENT_HALTED、CLOCK_SKEW 是可枚举、可测试的阻塞结果。
- 暂无 live order/approval command，因此 S08 只提供一个可复用的 authority eligibility gate：时间不可信、市场 CLOSED/HALTED、历史调整未知或来源不满足时返回阻塞原因。S21–S27 必须调用这个 gate，不能在 renderer 或模型中重新推断。

## User Stories

1. 作为投资者，我想在 equity detail 看到当前市场 session 状态，以便知道详情是否来自 OPEN、CLOSED 或 EXTENDED HOURS。
2. 作为投资者，我想看到下一次 open/close 时间，以便安排研究和后续操作。
3. 作为投资者，我想看到 holiday、half-day 和 extended-hours 的来源与 calendar version，以便判断时间边界是否可信。
4. 作为投资者，我想看到某个 instrument 是否 HALTED，以及解除前的 remediation，以便不会误以为它可交易。
5. 作为 crypto 用户，我想看到 venue maintenance、instrument suspension 或 degraded trading service 状态，以便区分全天市场与 venue 故障。
6. 作为研究用户，我想看到 split、dividend、symbol change 和 delisting 事件及其 effective time，以便正确解释价格和公司身份变化。
7. 作为研究用户，我想看到历史数据的 adjustment status，以便知道历史 OHLCV 是否可用于比较或回测。
8. 作为用户，我想看到市场数据源、calendar version、provider time 和 TradeX received time，以便区分真实来源、旧 fixture 和未知状态。
9. 作为用户，我想在没有 OD-005 entitlement 时看到 BLOCKED_EXTERNAL/UNKNOWN 和下一步，而不是看到一个看似真实的开市状态。
10. 作为用户，我想看到 TimeService 的 wall-clock、monotonic、provider offset 和 confidence，以便判断 quote age 是否可信。
11. 作为用户，我想在系统 sleep/resume 后看到时间重新校验中的阻塞状态，以便旧的 approval TTL 和 freshness 不会继续有效。
12. 作为用户，我想在系统时钟发生跳变时获得 CLOCK_SKEW remediation，并在同步成功前保持 Live 阻塞。
13. 作为用户，我想通过明确的重新同步动作恢复可信时间，而不是由打开某个页面隐式恢复。
14. 作为未来 Live 用户，我想让 approval、TTL、reconciliation 和 dispatch 都使用同一时间信任结果，以便不同页面不会出现互相矛盾的资格判断。
15. 作为未来 Live 用户，我想让 CLOSED/HALTED/CLOCK_UNCERTAIN 在审批前和派发前都阻塞，以便模型无法绕过确定性门禁。
16. 作为用户，我想看到状态变化通过文本和 aria-live 告知，并能用键盘访问所有 remediation 控件。
17. 作为窄窗口用户，我想在 768px 与 390px 仍看到 session、halt、corporate-action、clock blocking 信息而不水平溢出。
18. 作为用户，我想在重开同一 workspace 后看到旧的市场事实被标为需要重新验证，而不是把旧进程时间当作当前可信时间。
19. 作为审计人员，我想让时间状态和 market-state provenance 具有有限长度、稳定枚举和可复现的测试输入，以便故障能被回放和审查。
20. 作为安全审查者，我想确认 TimeService 和 market-state query 不读取或返回 broker/model secrets，也不修改账户、风险、审批或 outbox 状态。

## Implementation Decisions

- 在现有 Rust Control Plane 增加 TimeService；所有当前时间读取集中经过该 seam。生产读数使用标准库 wall clock 与 monotonic Instant，测试使用注入的有限 reading，不增加新的时间库或后台线程。
- 使用固定、文档化的容差（以毫秒表示）比较 wall elapsed 与 monotonic elapsed；超过容差、monotonic 回退、resume reset 或 provider offset 超界即为 CLOCK_UNCERTAIN/STALE。状态恢复必须显式 revalidate，且第一次正常采样建立新基准。
- 添加 time.status 和 time.revalidate 两个 workspace-scoped query command；输出 typed TimeStatus，不提供任意 renderer 传入时间覆盖生产读数的入口。时间状态不进入 DomainProjection/outbox；未来 authority gate 只消费 service 的结果。
- 为 MarketDetail 增加可选的 typed MarketState 数据，包括 session enum、venue、source、next boundaries、calendar version、observed timestamp、time confidence 和 reason；保持 S07 的 canonical instrument/snapshot/entitlement 字段兼容。
- 为 CorporateAction 定义有界 action type、instrument ID、effective/announced timestamp、source ID、description 和 adjustment status；列表有上限并拒绝未知字段、控制字符、重复事件和不可信时间。
- 以当前 canonical registry 的 AAPL/MSFT 和 BTC/USDT、ETH/USDT 建立纯本地 fixture seam，明确标注 fixture；真实 OD-005 仍走 data-source catalog 的 status gate。没有可用来源时状态为 UNKNOWN/UNAVAILABLE，不用当前系统时间推断 exchange 是否 OPEN。
- 将 MARKET_CLOSED、INSTRUMENT_HALTED、CLOCK_SKEW 纳入稳定错误 remediation；关闭/停牌错误分别指向查看下一 session/等待权威恢复，clock 错误指向 time revalidate。市场详情仍可只读显示状态。
- 在 Markets 的 equity detail 增加 Market Status、next boundary、Corporate Actions、Adjustment Status 区域；crypto detail 显示 venue state。Time/Health 设置或可复用 recovery panel 显示 K6；所有状态使用文本、颜色之外的语义和 live announcement。
- 提供单一 market_execution_eligibility/等价 gate，供未来 approval/risk/dispatch 调用；S08 不实现订单、风险政策或外部日历付费授权，也不把 UI 选择当 authority。
- 保留 S06/S07 的 source licensing boundary、SQLite domain authority 与 DuckDB history boundary；S08 的时间和 market-state 观察是有限运行时/详情投影，不新建第二个数据库或跨窗口 event stream。

## Testing Decisions

- 用纯 Rust service tests 验证正常 wall/monotonic elapsed、material jump、monotonic 回退、sleep/resume、provider offset 超界、首次基准、显式 revalidate 和状态恢复；同一输入必须得到稳定枚举和 remediation。
- 用 market-state tests 验证 equity regular/holiday/half-day/extended/closed/halted、crypto maintenance/suspended/degraded、next boundary、corporate action 类型/排序/adjustment status、未知/重复/越界输入以及 MARKET_CLOSED/INSTRUMENT_HALTED gate。
- 用真实 ControlPlane 临时 workspace 测试 time.status/time.revalidate 和 market.get 的 typed payload、workspace isolation、unknown fields、无秘密、无 SQLite/outbox/account/risk/thread mutation；验证重开会使时间重新需要 revalidate。
- 重新生成并检查 Rust/JSON Schema/TypeScript；运行 targeted Rust、workspace Rust、clippy、fmt、frontend typecheck/build/unit 和 traceability。
- 用 Rust-backed integration/browser 验证 AAPL detail 的 C4/C6 状态面板、BLOCKED_EXTERNAL/UNKNOWN 文案、K6 clock recovery、键盘焦点、aria-live 和 1280/768/390 无水平溢出；控制台 error/warn 必须为空。
- 任何 fixture、source label 或截图只证明 rendering/contract。OD-005 没有 entitlement 时不做 provider 网络请求，不把日历或公司行为标为真实已验证。

## Out of Scope

- 购买或验证 Alpaca OD-005、任何付费交易日历/公司行为许可，以及真实 provider entitlement。
- 实时 quote/history adapter、portfolio/FX、screener、research tools、order draft、risk engine、approval、reservation、Order Gateway、reconciliation 和 Live 交易；这些由后续 S09–S30 负责消费 S08 gate。
- 完整历史 corporate-action ingestion、自动回填或批量调整 DuckDB 数据；S08 只呈现受限 metadata 和明确的 adjustment status。
- 全市场订阅、后台定时刷新、云端同步、移动端 Live execution 和新的 charting/licensing 决策。

## Further Notes

- 权威来源：PRD §§33、35、45、51、62.1、62.4、62.5、63；UI Spec C4–C6、K1–K7、§9–§14；Backend ARD §§14–15、17、43–44；Frontend ARD market snapshot/error/accessibility seams；Coverage/QA FR-038、FR-062、FR-077、AC-049、AC-063、QA-04。
- S06 的 OD-005 状态仍是 BLOCKED_EXTERNAL；这份规范明确分离“状态契约/fixture 验证”和真实数据授权。
- S07 的 MarketDetail 继续保留原有 snapshot optional 语义；新增 market state 不会把 unavailable source 变成 quote。
- #31 实现固定 SHA 为 `813a75a00497d7746167c3876583f80e07b0193b`；本轮证据记录锚点为 `ece04c5de3413aa13bf7437729ad9b701a31b4f3`（[S08 #31 验收](s08-market-evidence.md)）。C6、C4/C5 的 S08 子范围已更新为 `IMPLEMENTED_UNVERIFIED`；K6 的可信时间记录沿用 [S08 #30 证据](s08-time-evidence.md)，QA-04 和 S33 仍待后续完整回归。
