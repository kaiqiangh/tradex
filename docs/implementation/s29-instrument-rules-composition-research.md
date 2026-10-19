# S29 InstrumentRules composition research / InstrumentRules 组合研究

> Research aid for `to-spec`, not a normative contract, implementation, approval, or provider acceptance. The MARKET-only proposal remains **pending**; no user approval was supplied to this research task. / 本文是供 `to-spec` 使用的研究材料，不是规范、实现、批准或提供方验收。MARKET-only 提案仍为 **pending**；本研究任务未收到用户批准。

Research completed 2026-10-19 UTC. Local baseline: `dev` commit `825fa12c6f44192589b4e214f3592c511c547667`. Read the root `AGENTS.md`, domain instructions, the pending [proposal](s29-instrument-rules-prepare-reachability-proposal.md), Backend ARD §41.44–41.48, and the four named producers/consumer. Only first-party Binance GitHub documentation was fetched; no market/account endpoint, key, order, test order, or provider mutation was used. No tests were run for this research note.

研究日期为 2026-10-19 UTC，本地基线为 `dev` 提交 `825fa12c6f44192589b4e214f3592c511c547667`。已阅读根目录 `AGENTS.md`、领域说明、待决[提案](s29-instrument-rules-prepare-reachability-proposal.md)、Backend ARD §41.44–41.48 及指定的四个生产者/消费者文件。仅获取 Binance 第一方 GitHub 文档，未调用行情或账户端点、读取密钥、创建订单或测试订单，也未执行提供方写操作。本文未运行测试。

## 1. Source pin / 来源固定

All provider citations below refer to `binance/binance-spot-api-docs` commit **`f0ee79c8ce24dbeda6645211b9504e237fddef19`**, observed as repository HEAD on 2026-10-19. Its commit subject is “Restoring all Docs”, dated 2026-10-15. These are ordinary production Spot docs, not `testnet/` docs. SHA-256 hashes were calculated from the checked-out original bytes. A documentation example is not the current live exchange configuration or an observation of this user's account.

以下提供方引用均固定到 `binance/binance-spot-api-docs` 的提交 **`f0ee79c8ce24dbeda6645211b9504e237fddef19`**，这是 2026-10-19 获取时的仓库 HEAD；提交说明为 “Restoring all Docs”，日期为 2026-10-15。引用的是普通生产 Spot 文档，而非 `testnet/`。SHA-256 来自检出的原始文件字节。文档示例不代表交易所当前配置，也不代表用户账户事实。

| File / 文件 | SHA-256 |
| --- | --- |
| `filters.md` | `6dd26bcb117d08b3d4d0b95870a3781e0ae287c7773c7be3d196d5f809f9d23b` |
| `rest-api.md` | `26929b1e149ca3847ff073aa956115bc98db3d4320a8dcb4841addbc6067291a` |
| `enums.md` | `f031071d9299d35b846f0203bdc33e8a1ce7b6b2a987bfb12dae22582e4d4c83` |
| `errors.md` | `0c553c5c214837b302a55f9dfb32afda07f748f480c84503cd7ae43d4b886b2c` |
| `faqs/order_count_decrement.md` | `597e24ba9f053452d081f8922ff19a3d5e120e2575144ec716becea7de1ec3a6` |
| `faqs/stp_faq.md` | `4ae07ebbbf75cc59cd91b760ec7fdb6caa9b08b018b0ee8a8c81350bdbf265fa` |
| `faqs/price_range_execution_rules.md` | `af063834db2aa1021f637f347f0b0c2d35c485eb9d0eda160c81cd36aee2809a` |
| `faqs/spot_glossary.md` | `ea989af24ce0f73af407ea02f0627190661e59e5d008c0d5484720b476570dcb` |
| `faqs/market_orders_faq.md` | `d813e47d50d23e0ea14882e339d14a0daf8fbf2add0f4030cdaec149010dd2b6` |
| `CHANGELOG.md` | `9ecb94507618a12fb689660b292580b32b2b6195254e56b2fe45509fbb8a340b` |

Pinned citations / 固定引用:

- **P1 — position:** [filters.md L298–319](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/filters.md#L298-L319).
- **P2 — symbol total/algo/iceberg:** [filters.md L255–295](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/filters.md#L255-L295).
- **P3 — amendments/lists/exchange counts:** [filters.md L351–439](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/filters.md#L351-L439).
- **P4 — ordinary new order, increments, forms:** [rest-api.md L2135–2188](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L2135-L2188).
- **P5 — open-order inventory:** [rest-api.md L4406–4460](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L4406-L4460).
- **P6 — ORDERS declaration, scope and first fill:** [faqs/order_count_decrement.md L18–112](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/order_count_decrement.md#L18-L112); [cancellation, UTC and old-order fills L155–199](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/order_count_decrement.md#L155-L199).
- **P7 — counter response:** [rest-api.md L4788–4824](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L4788-L4824); [headers and failures L231–243](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L231-L243).
- **P8 — STP behavior:** [faqs/stp_faq.md L8–70](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/stp_faq.md#L8-L70); [prevented quantity L103–135](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/stp_faq.md#L103-L135); [defaults and allowed modes L137–168](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/stp_faq.md#L137-L168).
- **P9 — account example and mode enum:** [rest-api.md L4292–4329](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L4292-L4329); [enums.md L172–183](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/enums.md#L172-L183); [CHANGELOG.md L2490–2499](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/CHANGELOG.md#L2490-L2499).
- **P10 — PRICE_RANGE:** [faqs/price_range_execution_rules.md L8–38](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/price_range_execution_rules.md#L8-L38); [non-enforcement and taker rules L82–116](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/price_range_execution_rules.md#L82-L116).
- **P11 — quantity field definitions:** [faqs/spot_glossary.md L144–146](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/spot_glossary.md#L144-L146); [origQty L352–354](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/spot_glossary.md#L352-L354).
- **P12 — order-list contributions:** [rest-api.md OCO L3191–3205](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L3191-L3205); [OTO L3441–3452](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L3441-L3452); [OTOCO L3581–3592](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/rest-api.md#L3581-L3592).
- **P13 — filter error descriptions:** [errors.md L539–561](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/errors.md#L539-L561).
- **P14 — market execution and expiry:** [faqs/market_orders_faq.md L3–22](https://github.com/binance/binance-spot-api-docs/blob/f0ee79c8ce24dbeda6645211b9504e237fddef19/faqs/market_orders_faq.md#L3-L22).

## 2. MAX_POSITION: formula is documented; partial-fill mapping is not / 公式明确，部分成交字段映射不明确

**Provider fact:** P1 defines the position of the symbol's base asset as `free BASE + locked BASE + sum(qty of all open BUY orders)`. A BUY fails if the position is greater than `maxPosition`, or if the new order's `quantity` causes overflow. The natural arithmetic for a documented complete BASE BUY input is therefore `P + requested_BASE <= maxPosition`. P1 does not impose this BUY rejection on a SELL; a SELL must still satisfy its own funding and other applicable obligations. P13's shorter wording “has reached the maximum” does not supply a different exact formula and should not replace P1's explicit greater-than/overflow terms.

**提供方事实：** P1 将该交易对基础资产仓位定义为 `free BASE + locked BASE + 所有未结 BUY 订单 qty 之和`。仓位大于 `maxPosition`，或新订单 `quantity` 导致超限时，BUY 会失败。因此，对文档定义完整的 BASE BUY 输入，对应算式为 `P + requested_BASE <= maxPosition`。P1 未将这项 BUY 拒绝规则用于 SELL；SELL 的资金及其他实际适用义务仍需满足。P13 简写的 “has reached the maximum” 没有提供另一套精确公式，不应覆盖 P1 明确的“大于”和“导致超限”描述。

**Unresolved provider detail:** P11 defines `origQty` as the original submitted quantity and `executedQty` as quantity filled. P1/P13 do **not** say whether the `qty` of a partially filled open BUY means original, original minus executed, or another engine-maintained quantity. P8 separately states `original - executed - prevented = available for further execution`, but it does not connect that equation to MAX_POSITION. Thus neither `sum(origQty)` for every partially filled BUY nor `sum(origQty-executedQty)` is established as the MAX_POSITION formula by these sources. The latter also ignores possible prevented quantity. This research does not infer either mapping.

**提供方细节未决：** P11 将 `origQty` 定义为原始提交数量、`executedQty` 定义为已成交数量。P1/P13 **未说明** 部分成交未结 BUY 的 `qty` 指原始数量、原始数量减已成交量，还是撮合引擎维护的其他量。P8 虽另行给出 `original - executed - prevented = 可继续执行数量`，但没有把它与 MAX_POSITION 关联。因此，不能将全部部分成交 BUY 的 `sum(origQty)` 或 `sum(origQty-executedQty)` 宣称为这些来源证实的 MAX_POSITION 公式；后者还忽略潜在 prevented quantity。本研究不推定任何一种映射。

The formula is base-asset based. An opaque different symbol name does not prove its BASE/QUOTE or whether its BUY quantity contributes to the selected asset. No suffix parsing, BTC/ETH guess, foreign-symbol exclusion, or conversion follows from P1. The currently collected selected-symbol quantity aggregate cannot establish all account-wide shared-asset exposure if foreign rows are present and unattributed.

该公式以基础资产为对象。其他不透明 symbol 名称不能证明其 BASE/QUOTE，也不能证明其 BUY 数量是否计入选定资产。P1 不支持通过后缀解析、BTC/ETH 猜测、直接排除其他交易对或换算来补足事实。如果账户库存包含未归属的其他 symbol，当前仅聚合选定 symbol 的数量无法证明整个账户共享资产敞口完整。

**Current input boundary:** `SpotCapacityPosition` retains selected-symbol original and executed sums separately plus `assetExposureComplete`; `SpotCapacityBalance` retains original free/locked. The collector uses an all-account inventory for MAX_POSITION BUY, marks any foreign symbol as incomplete, and carries `PARTIAL_FILL_QUANTITY_SEMANTICS_UNRESOLVED` for **any** partially filled or executed open row. `spot_owning::qualify` treats either real obligation as fatal. This is deliberately stricter than proving that an unrelated SELL cannot affect MAX_POSITION; the present aggregates do not retain sufficient per-row context to selectively discharge that broad label. An empty all-account open inventory plus exact BASE free/locked avoids the open-BUY quantity ambiguity altogether. Supporting nonempty or partial-fill cases needs a separately specified evidence/formula boundary; no current positive partial-fill composition is established.

**当前输入边界：** `SpotCapacityPosition` 分别保留选定 symbol 的原始量与已成交量总和以及 `assetExposureComplete`，`SpotCapacityBalance` 保留原始 free/locked。MAX_POSITION BUY 会收集全账户库存；发现任意其他 symbol 即标为不完整；发现**任意**部分成交或已执行的未结订单即携带 `PARTIAL_FILL_QUANTITY_SEMANTICS_UNRESOLVED`。`spot_owning::qualify` 将这些真实未决义务视为阻断。这比证明某个无关 SELL 不影响 MAX_POSITION 更保守；现有聚合未保留足够逐行上下文来有选择地解除该广泛标签。全账户未结库存为空且 BASE free/locked 明确时，可完全避开 open-BUY 数量歧义。若要支持非空或部分成交库存，必须另行明确证据与公式边界；当前没有成立的部分成交正向组合。

## 3. Counts: distinguish scope, class and new contribution / 区分计数作用域、类别与新增贡献

Provider definitions P2/P3 describe limits on **one account**, not the exchange-wide total of every customer's orders. “Exchange” means this account across symbols. A symbol read supports symbol scope; an all-account read supports this account's exchange scope. Neither can be substituted for the other.

P2/P3 的限制均针对**一个账户**，不是所有客户订单的交易所全局总量。“Exchange” 表示该账户跨全部 symbol 的范围。单 symbol 查询支持 symbol 作用域；全账户查询支持该账户的 exchange 作用域，二者不能互换。

| Rule family / 规则族 | Provider definition / 提供方定义 | Ordinary standalone BASE MARKET BUY/SELL contribution / 普通独立 BASE MARKET 买卖贡献 |
| --- | --- | --- |
| `MAX_NUM_ORDERS` | Account's symbol open total; normal and algo both counted (P2). / 账户在该 symbol 的未结总数，普通与 algo 都计入。 | **1** new order (P4), even if expected to fill immediately. / 新增 **1**，不能因预计即刻成交而跳过。 |
| `EXCHANGE_MAX_NUM_ORDERS` | Same account's exchange open total (P3). / 同一账户跨交易所 symbol 的未结总数。 | **1** (P4; endpoint text calls it `EXCHANGE_MAX_ORDERS`, while filter schema uses `EXCHANGE_MAX_NUM_ORDERS`). / **1**；端点文字与过滤器 schema 名称存在上述差异。 |
| `MAX_NUM_ALGO_ORDERS`, `EXCHANGE_MAX_NUM_ALGO_ORDERS` | Only `STOP_LOSS`, `STOP_LOSS_LIMIT`, `TAKE_PROFIT`, `TAKE_PROFIT_LIMIT` (P2/P3). / 仅列出的四种止损/止盈类型。 | **0** for actual type `MARKET`; a stop order that later executes as MARKET is still an algo form (P4). / 实际类型为 `MARKET` 时为 **0**；日后执行成 MARKET 的 stop 订单仍属于 algo 形式。 |
| `MAX_NUM_ICEBERG_ORDERS`, `EXCHANGE_MAX_NUM_ICEBERG_ORDERS` | Iceberg means `icebergQty > 0` (P2/P3). P4 lists iceberg-capable limit forms. / `icebergQty > 0` 为 iceberg；P4 列出了可使用 iceberg 的 limit 形式。 | **0** for the bounded ordinary MARKET form with no iceberg quantity. / 无 iceberg 数量的限定普通 MARKET 为 **0**。 |
| `MAX_NUM_ORDER_LISTS`, `EXCHANGE_MAX_NUM_ORDER_LISTS` | Open lists, symbol/account-exchange scopes; one OTOCO counts as one **list** (P3). / 分别为 symbol 与账户跨 symbol 的未结列表数；一个 OTOCO 算一个**列表**。 | **0** lists for a standalone ordinary new order; existing list legs still contribute to applicable total-order inventory. / 独立普通新订单新增 **0** 个列表；已有列表的各 leg 仍计入适用的订单总库存。 |
| `MAX_NUM_ORDER_AMENDS` | Number of amendments to a single order (P3). / 单个订单被 amend 的次数。 | A new standalone `PLACE` is not an amendment. / 新建独立 `PLACE` 不属于 amend。 |

**Local policy still to specify:** For total counts, current complete `count + 1 <= declared limit` is the bounded local admission arithmetic supported by the documented maximum and P4's contribution. A count at the limit cannot be excused by anticipated fills. For zero-contribution families, the docs establish the order's classification but do not explicitly prescribe a new standalone MARKET request's behavior when some unrelated existing category already exceeds its present limit. The spec must say whether a known non-contributing rule is not-applicable to this exact request or is deliberately checked as a stricter account-wide condition. Do not silently assert a provider guarantee or evaluate `count+1` for algo/iceberg/lists. This note recommends recording the purpose-specific distinction before implementation; it does not approve that policy.

**仍需明确的本地策略：** 对总计数，文档最大值与 P4 新增贡献支持限定本地准入算式 `当前完整 count + 1 <= 已声明 limit`；不能以预计成交为由放过已达到 limit 的计数。对增量为零的类别，文档明确了订单分类，但未明确说明“某个不相关已有类别已超过当前 limit 时，新独立 MARKET 请求如何处理”。规范必须明确：该规则对当前不贡献数量的请求不适用，还是有意将它作为更严格的全账户条件。不能暗中承诺提供方一定放行，也不能将 algo/iceberg/list 都算为 `count+1`。本文建议在实现前记录用途边界，但不批准该策略。

P12 independently prevents a common count error: OCO contributes 2 total orders, OTO 2, OTOCO 3, while OTOCO is one list. Pending legs can exist before becoming working book orders. Missing/contradictory list legs therefore cannot be erased or counted as zero. Current collection checks list/leg correspondence; it intentionally returns incomplete instead of pretending one returned list equals all necessary order evidence.

P12 独立说明：OCO 为 2 个总订单、OTO 为 2 个、OTOCO 为 3 个，但 OTOCO 只有 1 个列表。Pending leg 可以在成为工作中盘口订单之前存在。因此不能将缺失或矛盾的 leg 删除或当成零。当前收集会检查 list/leg 对应关系；证据不足时标为不完整，不会将返回一个 list 等同于获得全部所需订单事实。

## 4. ORDERS is an account interval counter, not open-order inventory / ORDERS 是账户时间区间计数

P6 defines ORDERS as an unfilled-new-order rate limit, shared by the (sub)account across **all keys, IPs and APIs**. P7 exposes all interval counters. P4 declares a standalone ordinary new order's unfilled-order contribution as **1**, including MARKET BUY and SELL. A successful first partial or full fill subsequently decrements every interval by at least one; some maker/large fills receive additional reductions, and updates may be delayed. Cancelling or expiring an order does not decrement it. Old orders' first fills can decrease the current interval, the counter never becomes negative, and DAY uses UTC (P6).

P6 将 ORDERS 定义为未成交新订单速率限制，由同一（子）账户的**全部密钥、IP 与 API**共享。P7 返回全部区间计数。P4 将普通独立新订单的未成交计数贡献声明为 **1**，包括 MARKET BUY/SELL。之后首次部分或全部成交会使每个区间至少减一；部分 maker/大额成交可能有额外减量，且更新可能延迟。取消和过期不减少计数。历史订单首次成交也可减少当前区间计数；计数不变为负数；DAY 使用 UTC（P6）。

Therefore current open-order count, partial-fill totals, cancellation events, absence of a header or a locally observed time boundary do not reconstruct the authoritative ORDERS counter. Preserve provider counts. Compare every exact declared tuple `(rateLimitType, interval, intervalNum)` and matching limit; an absent/extra/unknown/contradictory tuple is a gap, not zero or unlimited. For a new single order, the local arithmetic is one available slot in **every** applicable ORDERS bucket. The existing owner reports the tightest exact `limit-count`; it rejects exact zero, and treats `count>limit` as unavailable inconsistency rather than manufacturing a negative slot count. No projected fill credit is allowed.

因此，未结订单数、部分成交总量、取消事件、响应头缺失或本地观察到时间边界，都不能重建权威 ORDERS 计数。应保留提供方原始计数，逐一匹配精确声明元组 `(rateLimitType, interval, intervalNum)` 及 limit；缺失、多出、未知或矛盾的元组属于缺口，而非零或无限制。单笔新订单的本地算式要求**每个**适用 ORDERS bucket 至少有一个槽位。现有 owning consumer 报告最紧 bucket 的精确 `limit-count`，在恰好为零时拒绝，在 `count>limit` 时标为事实不一致、不可用，不制造负槽位，也不提前计入预计成交减量。

P7's original counter response has no snapshot time, interval start/end or reset timestamp. An HTTP receipt or `/time` sample is not one. P6 gives fixed 10-second example windows and UTC DAY, but does not remove transport/read races. `SpotOrderIntervalInputs` accordingly keeps `atomic:false`, `providerObservedAt:null`, `DERIVED_UNCERTAIN`, separate read digests/times, and a conservative possible-boundary retirement. A local collection cannot guarantee another key will not place an order immediately afterward.

P7 的原始计数响应没有快照时间、区间开始/结束或重置时间；HTTP 接收时间与 `/time` 样本都不能替代。P6 的固定 10 秒示例和 UTC DAY 规则不会消除网络或查询竞态。故 `SpotOrderIntervalInputs` 保留 `atomic:false`、`providerObservedAt:null`、`DERIVED_UNCERTAIN`、独立读取摘要/时间，并在可能跨边界时保守退役。本地收集不能保证其他密钥随后不再下单。

## 5. STP account fact, default and modes / STP 账户事实、默认值与模式

P8 says self-trade means the same account or the same trade group; taker mode usually determines behavior, with the documented TRANSFER exception. P9 enumerates **six** current mode tokens: `NONE`, `EXPIRE_MAKER`, `EXPIRE_TAKER`, `EXPIRE_BOTH`, `DECREMENT`, `TRANSFER`. The FAQ introductory sentence still says “five possible modes” despite listing six; use the explicit current enum and descriptions, not that count. NONE exempts the order from STP. A symbol's `defaultSelfTradePreventionMode` applies if the request omits a mode; `allowedSelfTradePreventionModes` constrains explicit selection. MARKET is not exempt: P4 includes the optional mode on new orders and P8's taker rules apply.

P8 定义同一账户或同一 trade group 的交易为自成交；通常由 taker 模式决定行为，TRANSFER 有文档说明的例外。P9 目前列出**六个**模式 token：`NONE`、`EXPIRE_MAKER`、`EXPIRE_TAKER`、`EXPIRE_BOTH`、`DECREMENT`、`TRANSFER`。FAQ 开头仍写“五种模式”却列出六种，应以明确枚举和说明为准。NONE 使订单不受 STP 保护；请求未传模式时使用 symbol 的默认值，显式选择必须属于允许集合。MARKET 并无豁免：P4 为新订单列出可选 STP 模式，P8 的 taker 规则适用。

P9 demonstrates `requireSelfTradePrevention:false` on an authenticated account response and the changelog announces the field. In the pinned repository, its occurrences do **not** provide a fuller normative truth table for its interaction with every mode. The example is not a global default and omission does not prove false. TradeX currently imposes the explicit local compatibility rule: missing account fact ⇒ unavailable; `true` with default NONE ⇒ reject; otherwise the supported form can pass this part. This rule must be identified as TradeX's conservative compatibility policy rather than a complete provider-side contract derived from the example.

P9 在认证账户响应示例中显示 `requireSelfTradePrevention:false`，changelog 也说明新增该字段。但固定仓库中的相关内容**没有**提供该字段与所有模式组合的完整规范真值表。示例不是全局默认值，字段缺失也不等于 false。TradeX 目前明确实施本地兼容性规则：账户事实缺失 ⇒ 不可用；`true` 且默认 NONE ⇒ 拒绝；其余已支持订单形式可通过这一部分。该规则应标为 TradeX 的保守兼容性策略，不能宣称为仅凭示例推导出的完整提供方契约。

The current rule producer projects `accountRequiresSelfTradePrevention` from its own authenticated account observation and validates that the default occurs in the symbol's allowed-mode list. The capacity collector merely type-checks an optional `requireSelfTradePrevention` field; it does **not** project that field as a substitute fact. Therefore a capacity refresh cannot fill a missing field in the rule evidence by assumption. Also, the producer currently bounds mode text/list membership without restricting those tokens to the documented six; the new composition spec must explicitly retain fail-closed behavior for a newly unknown effective mode instead of treating any non-NONE token as known compatibility.

当前 rule producer 从自己的认证账户观察中投影 `accountRequiresSelfTradePrevention`，并校验默认模式在 symbol 允许集合中。capacity collector 仅对可选 `requireSelfTradePrevention` 做类型校验，**没有**将其投影成替代事实。因此不能靠 capacity refresh 的存在来补齐 rule evidence 的缺失字段。另一个边界是：当前 producer 限制模式文本长度并检查集合成员关系，但未将 token 限定在已文档化的六种；新的组合规范必须明确对未知有效模式保持不可用，不能将任意非 NONE token 当成已知兼容模式。

## 6. PRICE_RANGE: MARKET still has execution limits / MARKET 仍可能受执行价格约束

P10 describes an **execution-time** rule for orders entering the taker phase. The reference is continually changing and recalculated for that phase; an attempted out-of-range execution expires the taker order. It does not state “MARKET orders are exempt”. MARKET can be a taker and remains exposed to this execution outcome; P14 also documents partial fills and expiry when liquidity is exhausted.

P10 描述订单进入 taker 阶段时适用的**执行时**规则。参考价持续变化并在该阶段重新计算；尝试越界执行会使 taker 订单过期。它没有声称“MARKET 订单豁免”。MARKET 可以是 taker，仍可能遭遇该执行结果；P14 还明确说明流动性不足时可能部分成交并过期。

The **existing TradeX contract** §41.47 instead records a purpose-specific local `PRICE_RANGE_EXECUTION_TIME_ONLY` not-applicable PASS for MARKET's **absence of a stated execution price**. That means there is no submitted limit price to qualify in this placement check; it is not provider non-enforcement or a fill guarantee. For an enforced LIMIT side, a fresh local reference and exact bounds are still a snapshot only, and the existing row remains unavailable. No MARKET-only approval should rewrite this as a positive LIMIT execution/admission guarantee.

**现行 TradeX 合同** §41.47 对 MARKET **没有声明执行价格**这一事实，给出限定本地用途的 `PRICE_RANGE_EXECUTION_TIME_ONLY` 不适用 PASS。其含义是在该 placement 检查中没有待验证的提交限价，不代表提供方不执行规则，也不保证成交。对受约束的 LIMIT 方向，即使本地参考价新鲜、边界计算精确，也仍只是快照，现有行保持不可用。MARKET-only 批准不能被扩展为 LIMIT 的正向执行或准入保证。

Provider non-enforcement is separately documented when the correctly scoped rule is absent, the symbol has no reference price / an explicit null reference, or a relevant multiplier is omitted. A failed fetch, missing evidence or stale reference is not proof of these facts. P10's omission example uses `bidMultiplierDown` while its sample schema uses `bidLimitMultDown`; preserve the actual returned documented schema and do not silently accept this prose spelling as another active field. A reported zero is not a documented disabling sentinel for this execution rule.

提供方另行明确了不执行的情况：在正确作用域中确实没有规则、symbol 没有参考价或参考价显式为 null、相关方向的 multiplier 未配置。查询失败、证据缺失或过期不证明这些事实。P10 遗漏字段的文字示例使用 `bidMultiplierDown`，而示例 schema 是 `bidLimitMultDown`；应保留实际返回且已文档化的 schema，不应默默将文字拼写当成另一个活跃字段。对该执行规则，文档没有将数值零定义为禁用标记。

## 7. What the delivered inputs prove / 现有输入能证明什么

| Current seam / 现有接口 | Established locally when complete/current / 完整且当前时可建立的事实 | Still not established / 仍不能建立的事实 |
| --- | --- | --- |
| `spot_proposal_rules::get`, `static_result` | Exact immutable intent; order form, known static quantity/notional/asset rules; existing purpose-specific PRICE_RANGE state. / 精确不可变意图、订单形式、已知静态数量/名义值/资产约束及既有 PRICE_RANGE 用途状态。 | Dynamic MAX_POSITION/count rows currently return unavailable; `MAX_NUM_ORDER_AMENDS` also does so unconditionally; known static PASS is not whole INSTRUMENT_RULES PASS. / 动态仓位/计数行仍不可用；`MAX_NUM_ORDER_AMENDS` 当前也无条件不可用；静态通过不等于整个 INSTRUMENT_RULES 通过。 |
| `spot_capacity::get`, `execute_refresh` | Exact account/symbol inventory counts and classification flags, optional BASE free/locked and separate original/executed selected BUY sums, list completeness, read provenance. / 精确账户/symbol 库存计数、分类标记、可选 BASE free/locked、分别保留的 BUY 原始/已成交量、列表完整性和读取来源。 | Does not compare concrete filter limits; no foreign-symbol asset mapping, partial-fill MAX_POSITION semantics, atomic snapshot or authoritative snapshot timestamp; missing scope/classification stays null/unavailable. / 未比较具体过滤器上限；没有其他 symbol 资产映射、部分成交 MAX_POSITION 语义、原子快照或权威快照时刻；缺失作用域或分类仍为 null/不可用。 |
| `spot_order_intervals::get`, `execute_refresh` | Exact declared ORDERS tuple/limit/count match and complete bounded counter set, subject to freshness/boundary retirement. / 精确 ORDERS 元组/上限/计数匹配、完整有界计数集合，受新鲜度及边界退役限制。 | No provider counter timestamp/reset time, no actual atomic window certification, no execution qualification or inferred future fill credits. / 无提供方计数快照或重置时刻，无原子区间认证、执行资格或预计成交减量。 |
| `spot_owning::qualify` | Binding agreement, genuine coverage refusal and current exact tightest interval headroom; carries original inventory facts and limitations. / 绑定一致、真实覆盖缺口阻断、精确最紧区间余量，携带原始库存事实与限制。 | Does not evaluate requested quantity against MAX_POSITION or total/algo/iceberg/list filter maxima; its PASS cannot discharge the still-unavailable dynamic rule rows. / 未将请求数量代入 MAX_POSITION 或总量/algo/iceberg/list 上限；其 PASS 不能解除仍不可用的动态规则行。 |

**§41.48 is already an explicit local policy decision about provenance.** It carries `NON_ATOMIC_PROVIDER_SNAPSHOT`, `DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED`, `COUNTER_SNAPSHOT_TIME_UNAVAILABLE`, `DERIVED_WINDOW_ASSOCIATION_UNCERTAIN`, `INTERVAL_INPUTS_NOT_EXECUTION_QUALIFIED` as limitations rather than automatic admission vetoes. It still refuses actual unknown coverage, classification/list gaps, expired inputs or possible-boundary retirement. A proposal phrase such as “any non-atomic coverage gap remains unavailable” is compatible only if “gap” means missing required coverage; treating `atomic:false` itself as a new fatal condition would contradict §41.48 and make every existing collection ineligible. Conversely, suppressing its label would falsely improve provenance.

**§41.48 已明确决定如何在本地使用这些来源限制。** 它将上述五个标签保留为限制，而非自动准入否决；真实未知覆盖、分类/列表缺口、过期和可能跨边界退役仍会阻断。提案中的“非原子覆盖缺口不可用”只有在“缺口”指所需覆盖缺失时才与之兼容；若将 `atomic:false` 本身新增为致命条件，就会与 §41.48 冲突，并使现有全部收集失去资格。反之，删除该标签则会虚假提升来源质量。

Both inputs must describe the same workspace, Proposal/hash, account, instrument, BASE/QUOTE, source and rule material. Their individual state versions are separately enforced and hashed, not compared for equality across distinct input types. The delivered owner compares identity/material; each producer owns session/policy/time/currentness through its binding and retirement. These checks cannot manufacture atomicity, account facts, scope or arithmetic semantics.

两种输入必须绑定相同 workspace、Proposal/hash、账户、instrument、BASE/QUOTE、source 与 rule material。各自 state version 分别执行并纳入摘要，不能要求不同类型输入的摘要相等。现有 owning consumer 比较身份/material，生产者通过自身绑定与退役负责 session/policy/time/currentness。这些检查不能制造原子性、账户事实、作用域或算式语义。

## 8. Decisions required before positive local Prepare composition / 正向本地 Prepare 组合前需要的决议

1. **Approve or reject the MARKET-only scope.** This note records no approval. Update the pending proposal and paired normative contracts before changing a gate. Preserve the exact BASE-only supported form and no stated limit price. / **批准或拒绝 MARKET-only 范围。** 本文没有批准记录。修改 gate 前需更新提案与双语规范，保留精确 BASE-only 支持形式及无提交限价边界。
2. **Name the purpose and retained limitations.** Positive result is local eligibility based on current separately collected evidence; adopt §41.48's existing provenance treatment without claiming atomic or venue-certified admission. / **明确用途与保留限制。** 正向结果仅为基于当前独立收集证据的本地资格；沿用 §41.48 的来源处理，不声称原子或交易所认证准入。
3. **Pin rule applicability and increments.** Total counts and ORDERS need +1; separate non-contributing algo/iceberg/list rules, new PLACE versus amend, and BUY-only MAX_POSITION from SELL. Unknown active rules/modes remain unavailable. / **固定规则适用性与增量。** 总计数和 ORDERS 为 +1；区分不贡献数量的 algo/iceberg/list、新 PLACE 与 amend，以及仅 BUY 的 MAX_POSITION 与 SELL；未知活跃规则或模式仍不可用。
4. **Bound MAX_POSITION's positive evidence.** Empty complete all-account inventory gives an unambiguous open-BUY sum of zero. Partial fills and foreign asset attribution currently remain unresolved. Do not use original-minus-executed or reinterpret all original quantities without a source-backed contract; no mapping is established here. / **限定 MAX_POSITION 正向证据。** 完整全账户空库存的 open-BUY 总和明确为零；部分成交及其他 symbol 资产归属仍未决。不得未经有来源支撑的合同就采用原始减已成交量或重解释全部原始量；本文未建立此映射。
5. **Define exact per-row pass/reject/unavailable reasons and capture.** Bind actual constraints, scope, formula, all consumed evidence versions and precise units to the immutable review; any genuine change requires a new review. Current contradictory or missing inputs cannot become guessed zero. Preserve existing `count>limit` interval inconsistency handling unless explicitly deciding otherwise. / **明确逐行结果与捕获。** 将实际约束、作用域、算式、全部消费证据版本与精确单位绑定不可变审查；真实变化要求重新审查；矛盾或缺失输入不能猜为零。若无显式新决议，保留当前 interval `count>limit` 的不一致处理。
6. **Keep provider facts distinct from policy.** Missing account STP fact cannot be replaced with a fixture/default; unknown default mode cannot pass by being non-NONE. A MARKET placement N/A row does not erase PRICE_RANGE execution exposure, and an enforced LIMIT remains unavailable under §41.47. / **区分提供方事实与策略。** 缺失 STP 账户事实不可用 fixture/默认值替代；未知默认模式不能因非 NONE 就通过；MARKET placement 的不适用行不消除 PRICE_RANGE 执行风险，受约束 LIMIT 继续依 §41.47 不可用。
7. **Keep acceptance bounded.** Any later fixtures demonstrate local arithmetic and refusal only. They do not establish actual user's source-use facts, provider acceptance, fees/FX, funding, reservations, permissions, Arm/consent, immediate Gateway preflight, submission or fill. / **限制验收结论。** 后续 fixture 只能证明本地算式和拒绝路径，不能证明用户实际来源用途事实、提供方验收、费用/FX、资金、预留、权限、Arm/同意、即时 Gateway preflight、提交或成交。

## 9. Independent literal arithmetic examples / 独立字面量算例

These are hand-stated consequences of the cited definitions, not observations, fixtures, runs or provider acceptance. They deliberately avoid assigning any MAX_POSITION quantity to a partially filled open BUY.

以下仅是引用定义的人工字面量推演，不是账户观察、fixture、测试运行或提供方验收；特意不为部分成交未结 BUY 指定 MAX_POSITION 数量。

- P1, complete empty all-account inventory: free BASE `2`, locked BASE `3`, open BUY sum `0`, requested BASE BUY `5`, maxPosition `10` ⇒ `2+3+0+5=10`, no overflow. Requested `5.00000001` ⇒ `10.00000001>10`, known violation. / P1：完整全账户空库存时，`2+3+0+5=10` 不超限；请求 `5.00000001` 则超限。
- P2/P4, complete symbol total `24`, maxNumOrders `25`, one ordinary MARKET ⇒ `24+1=25`; total `25` ⇒ `25+1=26>25`. / P2/P4：symbol 总数 `24`、上限 `25` 时单笔 MARKET 后为 `25`；总数已有 `25` 时变为 `26`，超限。
- P4/P6, declared ORDERS bucket limit `100`, count `99` ⇒ one current slot; count `100` ⇒ zero slots. A predicted immediate MARKET fill supplies no pre-admission decrement. Every other bucket must independently have a slot. / P4/P6：ORDERS limit `100`、count `99` 剩一个当前槽位，count `100` 为零；预计即刻成交不提供准入前减量，其他每个 bucket 也各自需要槽位。

## 10. Local source anchors and reproducibility / 本地来源定位与可复现性

All local line anchors describe baseline commit `825fa12c6f44192589b4e214f3592c511c547667`, before this note. Relative links are repository navigation, not provider authority.

以下本地行号对应本文之前的基线提交 `825fa12c6f44192589b4e214f3592c511c547667`；相对链接用于仓库导航，不作为提供方规范。

- [Backend ARD](../TradeX_Backend_ARD_v1.0_RevC.md): §41.44 L3155, §41.45 L3165, §41.46 L3179, §41.47 L3193, §41.48 L3205.
- [spot_capacity.rs](../../src-tauri/src/spot_capacity.rs): structures L76–161; purpose selection `get` L198–247; `execute_refresh` L343; account STP type-check L587–620; counts/quantity classification L735–918; list/partial/foreign obligations and bounded observation L920–1131.
- [spot_order_intervals.rs](../../src-tauri/src/spot_order_intervals.rs): local clock association L26–47; bounded types L55–169; local get/retirement L171–213; explicit refresh L351; tuple/coverage/provenance construction L588–684.
- [spot_proposal_rules.rs](../../src-tauri/src/spot_proposal_rules.rs): static evaluator L294–431; PRICE_RANGE state/row L433–604; order-form/STP check L697–748; dynamic rows evaluated through `static_result` L758–804.
- [spot_owning.rs](../../src-tauri/src/spot_owning.rs): standing limitations L26–36; identity match L130–141; `qualify` L230–414; genuine capacity gaps L274–294; interval handling L300–410.
- [binance_rules.rs](../../src-tauri/src/financial_sources/binance_rules.rs): symbol STP fields L108–109, default-membership check L508–513, projected account requirement L702–709. This extra source was read only to trace where the consumer's STP account fact originates.

| Local file / 本地文件 | SHA-256 at research baseline / 研究基线摘要 |
| --- | --- |
| `docs/implementation/s29-instrument-rules-prepare-reachability-proposal.md` | `1cd6cc7295c174cb54aa5340f73fb5341d4d404ff5505343796a2d9b663935ba` |
| `docs/TradeX_Backend_ARD_v1.0_RevC.md` | `dfe6b9800d799f94d3f4dee58e52303b6d8f20ad86c0e10c26516b556dd03995` |
| `src-tauri/src/spot_capacity.rs` | `24b83a44d90432241e6aef854b615a5bc933312e07b0ac700be37d0deb27ef08` |
| `src-tauri/src/spot_order_intervals.rs` | `4df3cd3c032662c66a63f00dbc234c7e3740a379be61942239870cd268c75dfc` |
| `src-tauri/src/spot_proposal_rules.rs` | `ed6687b335c95a3c0824150c77298354b81a10e282ff12ec8cb4462962530bfc` |
| `src-tauri/src/spot_owning.rs` | `742f85c00dd267ef385fd615766a9e70e18a8b447df839a4f4100d724800e3b7` |
| `src-tauri/src/financial_sources/binance_rules.rs` | `de191c190e9a0307700a391d81d07334166cfc68a2b41f9251f55a118c33ac52` |
