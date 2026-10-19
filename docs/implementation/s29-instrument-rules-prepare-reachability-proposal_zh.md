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

允许一个严格限界的**本地 Prepare eligibility**评估，在受支持的普通 Binance Spot **BASE MARKET** 订单中，组合已有的当前 Spot 规则、容量和区间观察，并绑定到不可变 Proposal。新 MARKET 订单没有明示执行价格，因此 `PRICE_RANGE` 按 §41.47 保持用途专属的“不适用”。组合结果只能满足该 Proposal 实际适用的规则义务；不得把 `SpotCapacityInputs.qualification` 或 `SpotOrderIntervalInputs.qualification` 改成 execution-qualified。

只有同时满足以下条件，组合结果才可使现有 `INSTRUMENT_RULES` 检查及 owning approval/Prepare eligibility 通过：

1. 所有输入在不可变 Proposal、普通 Binance Live 账户、source generation、规则材料、instrument、side、数量单位、policy、session 和新鲜度绑定上完全一致。
2. 每项适用的静态订单形式、数量、notional、asset 和 admission 规则都有完整且已知的 schema 与精确单位；已知违规必须拒绝。
3. 适用的 `MAX_POSITION`、账户/交易所/币对订单计数、订单列表计数和订单速率义务，须使用现有精确账户容量/区间观察，按实际规则限额和请求的 BASE 数量进行评估。实现前须明确各规则公式、输入覆盖及采用 open buy 原始数量还是未成交数量；不得推断剩余数量或映射未验证的资产。
4. 必需的账户 self-trade-prevention 事实必须存在，且与所选普通订单形式兼容。缺失值继续不可用。
5. 任意缺失、过期、非原子覆盖缺口、部分/矛盾的订单清单、未知活动字段或不支持义务均继续 `UNAVAILABLE`。当前精确违规为 `REJECT`。当前不可变 review digest 绑定所有消费输入；刷新或变更后必须重新 review。

正向路径有意仅限没有明示执行价格的 MARKET 形式。受约束的 LIMIT `PRICE_RANGE` 仍保持不可用，其快照边界仍不具权威性。本决议不授权提交订单，也不豁免行情/数据用途权利、费用/FX、钱包 claims、权限/健康、Arm、consent、reservations、Gateway 即时认证 preflight、生命周期或独立 provider/source-use 事实。当前合成 Alpaca Support 上下文不计入验收，也不是用户真实来源用途事实的证据。

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
