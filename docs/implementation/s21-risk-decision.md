# S21 #81 RiskDecision 求值与审计规范

日期：2026-09-25  
前置：#80 Risk Policy 配置与持久化；S08 TimeService、S09 Portfolio/FX、S13 immutable OrderProposal。

## Problem Statement

TradeX 已持久化完整、版本化的 Risk Policy 和 immutable OrderProposal，但尚未对提案执行确定性风险求值。用户看不到哪些限制通过、拒绝或缺少证据，也无法在重开工作区后检查当时的决策依据。

## Solution

可信 Rust Control Plane 读取指定 workspace 中的 immutable OrderProposal、当前 Risk Policy、精确账户、workspace Portfolio、market/time evidence 和可用的执行活动/预留记录，产生新的 append-only RiskDecision。Renderer 只能请求求值并显示结果，不能传入或覆盖任何政策、账户、市场输入、检查结果或资格状态。

RiskDecision 与 OrderProposal 分开持久化。重求值新增决策记录，不改变 Proposal 字段、hash 或历史；每条决策都绑定当时的 Proposal hash、workspace、目标账户/环境、当前 policy version、评估时间及输入引用/摘要。缺少或不可信的必需证据产生可审计的 `UNAVAILABLE` 检查，不合成空组合、零计数或汇率平价。

## User Stories

1. 作为用户，我想让 Control Plane 对指定 Proposal 运行风险求值，以便看到确定性的 `ALLOWED`、`REJECTED` 或 `UNAVAILABLE` 结果。
2. 作为用户，我想逐项查看稳定的检查名称、结果和原因，以便知道哪个限额拒绝提案或缺少什么可信数据。
3. 作为用户，我想让结果绑定不可变 Proposal hash、workspace、普通账户/环境、policy version 和当时输入，以便审计时确认评估对象。
4. 作为用户，我想在重新评估后仍能查看之前每一次决策，以便还原政策或账户证据变化前后的结果。
5. 作为用户，我想在缺少行情、账户、规则、组合、FX、完整成交计数或预留记录时看到 `UNAVAILABLE`，以便缺数不会变成批准。
6. 作为多账户用户，我想让 workspace 级政策的组合限制使用 workspace Portfolio，而账户白名单、环境和健康检查仍绑定精确目标账户，以便屏幕当前选择不缩小共享限制范围。
7. 作为用户，我想让 `RISK_REJECTED` 和 `UNAVAILABLE` 在 Proposal 详情中展示原因，且不能从 Agent、普通确认或 Renderer 覆盖。
8. 作为用户，我想确认风险检查通过只代表政策求值通过，不等于 Arm、金融审批、预留或下单授权。
9. 作为 Bitget 用户，我想直接使用普通 `BITGET_LIVE` 账户身份，保留只读边界，不创建或使用 Demo 账户。
10. 作为键盘或窄屏用户，我想在 1280、768 和 390 像素布局中读取每项结果并操作重新评估。

## Implementation Decisions

- 求值请求只接收 workspace ID 和 proposal ID；Control Plane 从可信存储/服务读取所有输入。求值与历史读取使用公开版本化 IPC 命令。
- RiskDecision 是独立的不可变审计记录，含唯一 ID、绑定身份、`ALLOWED`/`REJECTED`/`UNAVAILABLE` 汇总状态、评估时间、每项 `PASS`/`REJECT`/`UNAVAILABLE` 结果和稳定 reason code。拒绝优先于不可用；没有拒绝但至少一项不可用时，总体为 `UNAVAILABLE`。
- 每次显式重求值都追加新记录和持久化事件，不重用旧决策、不更新 Proposal、不覆盖历史。输入身份包括当时的政策/账户/组合/市场/时钟版本或内容摘要；历史查询返回所有已保存决策。
- workspace 持有共享 Risk Policy。账户允许/阻止、环境、连接身份与健康状态按 Proposal 的精确 `accountId` 求值；组合规模、资产类别暴露、workspace 级日活动和 open-order 计数使用完整 workspace Portfolio，不按当前 UI 选择缩小。缺行、未知状态或不完整数据使依赖该数据的检查不可用。
- 对 PRD §21 每项配置给出稳定检查：订单名义金额/数量按精确 decimal 与 Proposal 比较；仓位与暴露依赖完整可信 Portfolio 和必要的 FX；日成交额/已实现损失依赖完整日计数；open-order 限额依赖可分类的权威订单状态；reserved-capital 依赖有效预留。限制为 null 时明确记录“未配置”并按无该用户限制处理；设置了限制但缺其输入时为 `UNAVAILABLE`。
- 允许/阻止 instrument、venue、account 与 environment 使用 canonical ID 精确匹配；空 allow-list 不增加限制，任一匹配的 block-list 优先。Bitget 普通账户的 `BITGET_LIVE` 映射到 `LIVE`；不经 Demo 适配器或 Demo 凭据求值。
- 禁用的市价单产生 `REJECT`。启用市价单但缺少覆盖请求数量的可信执行价格估算时，滑点检查为 `UNAVAILABLE`。最大价格偏差以可信 snapshot 的 last price 为基准，与 limit price 做 exact-decimal 百分比比较。实时场景的 delayed/stale/unentitled 行情及不可信 TimeService 均不能通过依赖行情的检查。
- `live_inactivity_timeout_minutes` 属于后续账户 Arm 会话时限；RiskDecision 明确不代表 Arm 或金融审批。当前尚未存在的预留/成交计数/提供方规则能力不伪装为通过，依赖它们的检查会带稳定原因返回 `UNAVAILABLE`。
- 风险拒绝显示为 `RISK_REJECTED`；证据不可用与政策拒绝保持可区分。所有可执行 submit 命令在 Rust 侧使用同一评估器阻止 `REJECTED`/`UNAVAILABLE` Proposal，不能依赖 Renderer 的禁用按钮作为安全边界；`ALLOWED` 本身不授予金融审批、Arm 或 Gateway 权限。
- 普通 Bitget Live 只用于账户身份及已有只读观察；本项评估不主动请求任何 provider endpoint，也不发送 Live/Demo/Testnet 订单。

## Testing Decisions

- 最高 seam：真实版本化 Rust Control Plane 命令、可信本地输入投影、临时 SQLite、decision event/outbox、查询与 workspace 重开。直接 evaluator 单元测试只辅助验证 exact-decimal 边界和逐项规则。
- 测试每项已配置限额的恰好等于/最小可表示超限值；允许/阻止优先级、环境映射、精确账户身份、Proposal hash、policy version 和 workspace 范围。
- 测试缺失/过期/延迟/不健康/不支持/不可信输入均变为带 reason code 的 `UNAVAILABLE`；未知输入不得变成零、空集合或 parity。
- 测试重复求值追加新 decision、不修改 Proposal/hash；按提案查询完整历史并在 workspace reopen 后逐条读取。
- 测试直接调用 submit IPC 时，`REJECTED`/`UNAVAILABLE` 无 provider/simulator side effect，证明绕过 UI 无效；验证期间不执行任何真实 provider 写操作。
- Rust-backed browser 展示 `RISK_REJECTED` 与不可用原因、键盘焦点与重新求值，并检查 1280/768/390 布局；Bitget 覆盖使用 `BITGET_LIVE` 只读语境，不创建或使用 Demo 账户。

## Out of Scope

- S22 Live Arm 和 PLACE/CANCEL 财务审批；S23 原子执行预留；S24 Order Gateway；S25–27 对账、撤单和恢复生命周期。
- 新的实时行情/FX/提供方规则/成交历史/预留采集适配器；这些证据缺失时返回不可用，后续对应数据工作项再补齐。
- Bitget Demo 账户或 Demo 订单路径验收，以及任何 Bitget Live 下单/撤单、转账或提现。
- 以本地 fixture 证明真实账户可交易、市场数据有授权或 Live execution ready。
- S33 全应用回归和 S35 dev→main 交付。

## Further Notes

Normative authority: PRD §21；UI Spec F7、F12、J2；Backend ARD §§17–19、§41.6；Frontend ARD risk/policy projection；parent spec #79。同步维护英文/中文 ARD、UI Spec 和 requirement evidence。
