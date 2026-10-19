# S29.11 Binance InstrumentRules 与 Prepare 可达性——规范提案

> **决议：待定。** 本文件为非规范性提案。决议记录前，不修改配对契约或运行时门禁。

## 问题

S29.11 #135 要求受支持来源通过公开生产者进入 owning RiskDecision 和 Prepare 消费链路。当前 owning Binance fixture 已使本地 Binance `MARKET_DATA_USE`确认、行情新鲜度、费用/FX statement、notional、reserved capital 和账户 Arm 检查通过；review 仍因 `INSTRUMENT_RULES=UNAVAILABLE` 而不可参与。

该结果符合现行规范：

- Backend ARD §41.44 规定适用的动态仓位/计数输入和 `PRICE_RANGE` 输入保持不可用；`INSTRUMENT_RULES` 通过要求每项适用的已采集义务都通过。
- Backend ARD §41.47 规定，即使有新鲜 reference 和快照边界，受约束的 `PRICE_RANGE` 行也保持不可用。它不是下单权限。本次 owning 回归使用了存在受约束 `PRICE_RANGE` 的 LIMIT Proposal。
- `spot_proposal_rules.rs` 将 `MAX_POSITION` 和订单计数规则映射为不可用义务。`spot_owning.rs` 根据 `SpotCapacityInputs`、`SpotOrderIntervalInputs` 检查覆盖完整性和声明的剩余速率槽，但没有依据 Proposal 对这些具体规则限额作比较。
- 当前精确账户 fixture 也未提供 `requireSelfTradePrevention`；生产逻辑因此保留 `ACCOUNT_SELF_TRADE_PREVENTION_REQUIREMENT_UNOBSERVED`，不会推定为 false。

因此，当前 LIMIT 场景按 §41.47 无法达到完整 `INSTRUMENT_RULES PASS`。实际采集的账户规则还含 §41.44 要求继续不可用的动态仓位/订单计数。没有先澄清契约，就不能把静态规则与 owning 观察合并为正向 Prepare review。

## 提议决议

允许一个严格限界的**本地 Prepare eligibility**评估，在受支持的普通 Binance Spot **BASE MARKET** 订单中，组合已有的当前 Spot 规则、容量和区间观察，并绑定到不可变 Proposal。新 MARKET 订单没有明示执行价格，因此本地“明示价格 placement 检查”按 §41.47 保持用途专属的“不适用”。MARKET 仍受交易所执行时 `PRICE_RANGE` 约束；提案不保证成交价格落在已保存区间内，也不保证成交。组合结果只能满足该 Proposal 实际适用的规则义务；不得把 `SpotCapacityInputs.qualification` 或 `SpotOrderIntervalInputs.qualification` 改成 execution-qualified。

只有同时满足以下条件，组合结果才可使现有 `INSTRUMENT_RULES` 检查及 owning approval/Prepare eligibility 通过：

1. 所有输入在不可变 Proposal、普通 Binance Live 账户、source generation、规则材料、instrument、side、数量单位、policy、session 和新鲜度绑定上完全一致。
2. 每项适用的静态订单形式、数量、notional、asset 和 admission 规则都有完整且已知的 schema 与精确单位；已知违规必须拒绝。
3. 适用的 `MAX_POSITION`、账户/交易所/币对订单计数、订单列表计数和订单速率义务，须使用现有精确账户容量/区间观察，按实际规则限额和请求的 BASE 数量进行评估。实现前须明确各规则公式、输入覆盖及采用 open buy 原始数量还是未成交数量；不得推断剩余数量或映射未验证的资产。
4. 必需的账户 self-trade-prevention 事实必须存在，且与所选普通订单形式兼容。缺失值继续不可用。
5. 任意缺失、过期、真实覆盖缺口、部分/矛盾的订单清单、未知活动字段或不支持义务均继续 `UNAVAILABLE`。原有非原子与区间关联来源标签沿用 §41.48 的处理：保留可见限制，不仅因这些标签就增加新的覆盖否决。当前精确违规为 `REJECT`。当前不可变 review digest 绑定所有消费输入；刷新或变更后必须重新 review。

正向路径有意仅限没有明示执行价格的 MARKET 形式。受约束的 LIMIT `PRICE_RANGE` 仍保持不可用，其快照边界仍不具权威性。本决议不授权提交订单，也不豁免行情/数据用途权利、费用/FX、钱包 claims、权限/健康、Arm、consent、reservations、Gateway 即时认证 preflight、生命周期或独立 provider/source-use 事实。当前合成 Alpaca Support 上下文不计入验收，也不是用户真实来源用途事实的证据。

## 提议的逐规则语义——基于来源的范围澄清，2026-10-19

[双语第一方研究](s29-instrument-rules-composition-research.md)将 Binance Spot 文档固定到 `f0ee79c8ce24dbeda6645211b9504e237fddef19`。下列内容仍是本地资格策略提议，尚未成为批准的规范修订；提供方事实与本地限制保持区分。

| 义务 | 提议的有限本地处理 |
| --- | --- |
| `MAX_POSITION` | 仅 BUY：比较 `free BASE + locked BASE + open BUY 数量 + 请求 BASE` 与精确上限。首期仅对完整全账户空库存给出正向资格，此时 open BUY 数量明确为零。非空库存缺少已确立的数量/资产归属、部分成交、余额缺失或覆盖矛盾均不可用。不得推断 `origQty-executedQty`、prevented quantity 或跨 symbol 的 BASE 映射。SELL 不增加该 BUY 专属义务。 |
| Symbol 与 exchange 订单总数 | 使用准确账户对应的完整作用域；独立新 MARKET 增加一，比较 `count + 1 <= limit`。不提前计入预计即刻成交减量。 |
| Algo、iceberg 与订单列表计数 | 支持的独立 MARKET 对这些类别的新增量为零。提议的保守本地策略仍要求完整适用类别证据并比较 `count + 0 <= limit`；该更严格检查属于 TradeX 本地资格策略，不声称交易所会如何处理不相关类别既有超限。不能对每种类别都加一，也不能用订单总数代替列表数。 |
| `MAX_NUM_ORDER_AMENDS` | 对新建独立 PLACE 用途专属不适用；不因此支持 amend/cancel-replace。 |
| `ORDERS` 区间额度 | 保留现有 owning 评估：每个准确声明元组都需要至少一个槽位，不本地重置、减量、滚动窗口或提前记预计成交。准确耗尽时拒绝；`count > limit`、声明不匹配、覆盖不完整或可能跨窗口而退役时仍不可用。 |
| 账户 STP 与 symbol 默认值 | 使用 rule producer 观察到的账户事实、实际默认模式和允许集合。缺失账户事实继续不可用；要求 STP 却默认 NONE 时保留既有拒绝。默认模式语义必须受到支持且可理解，不能仅凭字符串集合成员关系让未知模式通过。无 renderer override，不伪造 false。 |
| `PRICE_RANGE` | MARKET 本地明示价格 placement 检查可不适用；交易所执行时规则、部分成交/过期风险继续披露。受约束 LIMIT 保留现有不可用结果。 |

原始输入类型继续保持 source-only qualification 及全部既有限制。提议中的消费者在现有 RiskDecision 接缝组合有界逐规则结果，将消费的证据版本绑定到不可变 review；不清除独立来源投影的 unavailable 状态或未知义务。

## 测试决策

沿用完整 owning-plan 票据已确认的 React → typed Rust Control Plane → 一次性 SQLite/outbox 接缝，不新增测试边界或拆票。Fixture 仅替代外部 HTTP/vault/WS/clock producer。

每次完成一个公开行为：完整空库存 MARKET BUY 与 SELL；准确总量/类别/区间限额；缺失/未知 STP；非空歧义/部分成交/其他 symbol 库存；过期或不匹配绑定与 CAS 退役；保持受约束 LIMIT 拒绝；1280/768/390 当前/captured/pre-arm 披露。每次本地规则通过仍须保留独立 blocker；只有完整 owning 事实才能按既有合成测试方案进入正向 approval/Prepare。Fixture 来源用途事实不建立真实提供方验收。同一票据继续持有原有全部十五项 owning AC。

## 获批后的实现边界

- Provider/vault I/O 保持在现有显式 refresh producer；RiskDecision 评估和 `get` 保持本地只读。
- 在现有 owning RiskDecision 接缝增加一个有界、按规则评估的组合逻辑；复用已捕获观察和单一 approval blocker。不新增 renderer authority 字段、第二份投影、provider mutation、test-order endpoint，也不放宽新鲜度。
- 保留原始限制（`atomic: false`、无 provider snapshot 时间、区间窗口关联不确定）并清晰展示。通过本地 Prepare eligibility 不代表成交、交易所 admission 或执行保证。
- 在运行时实现前同步 PRD、Backend ARD 英文/中文、Frontend ARD、UI 规范和 owning-plan 契约。受约束 LIMIT `PRICE_RANGE` 继续拒绝，除非另有决议。
- 增加公开接缝 RED/GREEN 覆盖：完整 MARKET BUY/SELL、每类适用规则、已知违规、self-trade 缺失、过期/不匹配证据、部分成交、跨范围计数、CAS 退休，以及任一不完整情况均由 Prepare 拒绝。外部生产者只用 fixture；fixture 来源用途事实不能计作真实验收。

## 替代方案

保留现行 §41.44/§41.47 行为，并将 #135 的 AC6/AC10 正向目标改成这些普通 Binance 订单形式明确不可用/仅拒绝。不得放宽运行时门禁，也不得声称现有正向 producer-to-Prepare 要求已完成。

## 待决议

请批准或拒绝上述仅限 MARKET 的本地 Prepare eligibility 组合方案。批准后可同步配对规范并实现；这不代表接受 #135 或任何金融/provider 门禁。拒绝则保留 fail-closed 代码，并需由 owning ticket 决定无法到达的正向验收项。

## 参考

- [Backend ARD §41.44、§41.47](../zh/TradeX_Backend_ARD_v1.0_RevC_zh.md)
- [Backend ARD §41.45、§41.48](../zh/TradeX_Backend_ARD_v1.0_RevC_zh.md)
- [`spot_proposal_rules.rs`](../../src-tauri/src/spot_proposal_rules.rs)
- [`spot_owning.rs`](../../src-tauri/src/spot_owning.rs)
- [`spot_capacity.rs`](../../src-tauri/src/spot_capacity.rs)
- [`spot_order_intervals.rs`](../../src-tauri/src/spot_order_intervals.rs)
- [S29.11 owning plan](s29-owning-financial-plan-spec_zh.md)
