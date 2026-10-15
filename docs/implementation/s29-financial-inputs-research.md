# S29 financial inputs research — complete fees and required FX

## English findings

Research checked **2026-10-10**, against clean `dev@8ad62d7d02f84a34ee47c460d4911eb829e94322`. This is research evidence, not a readiness upgrade. English [RevC PRD §21.4, §22, §38.1 and AC-035](../TradeX_PRD_v1.0_RevC.md), [Backend §41.49–41.50](../TradeX_Backend_ARD_v1.0_RevC.md), [S29.9](s29-fee-and-required-fx-spec.md) and [S29.10](s29-symbol-commission-spec.md) govern the proposed work. S29.10 establishes a complete known declaration, not a monetary fee, future charging asset or execution-FX qualification. Full S29 and financial acceptance remain pending; prototype code is unchanged.

The Binance-owned Spot documentation repository was pinned through its [first-party commit API](https://api.github.com/repos/binance/binance-spot-api-docs/commits/master) at **`263ac1aa96556af0b4da824c82a1d8cd9a0edda9`**, commit date `2026-10-07T00:26:57Z`, observed `2026-10-10T18:08:39.074013Z`. Pinned source links below are reproducible; the developer portal and other official pages were checked on the research date.

**Finding:** a complete, technically implementable **conditional current fee estimate** is supported by the official model. Direct public USDT/USD and BNB/USD source candidates also exist. Qualification still needs purpose-specific source/rights/freshness evidence, fee rounding and BNB-conversion semantics, exact-account holdings, and an owning reservation model. These are concrete prerequisites, not evidence that the complete goal is permanently impossible. A current conservative estimate, an authorized allowance, a broker-enforced bound and a realized fee are different facts. Treating all four as a guaranteed future conversion would create a stronger requirement than RevC; treating an estimate as a hard bound would weaken its financial guards.

## 1. Commission model and the discount question

The [pinned commission FAQ](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/faqs/commission_faq.md) explicitly marks its numbers fictional. It applies role plus side rates for each standard/tax/special family to the received amount: BUY receives BASE quantity; SELL receives QUOTE proceeds. BNB conversion discounts **only standard** commission; all three converted components contribute to total BNB sufficiency. Insufficient BNB invokes the undiscounted received-asset fallback. Its formula multiplies by the returned discount value. The [pinned REST account/commission example](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/rest-api.md#query-commission-rates-user_data) supplies `0.75000000`, while the FAQ uses `0.25000000`. Different example values do not establish competing operators: pin `d = original returned payment multiplier`, never `1-d` or a hard-coded promotional rate. No private actual-account rate was obtained in this research.

Derived arithmetic specification, before venue rounding, for side `s`, role `r` and received amount `X`:

```text
A(r,s) = standard[r] + standard[s]
T(r,s) = tax[r]      + tax[s]
H(r,s) = special[r]  + special[s]
received-asset branch = X × (A + T + H)
conditional BNB branch = X × K(received asset → BNB) × (d × A + T + H)
```

Here `X` is filled BASE for BUY and actual filled QUOTE for SELL; an expected statement substitutes its explicitly bound model quantity/proceeds. Preserve the components and calculate the maximum **complete role sum** over maker/taker when role is uncertain. An ordinary limit may cross immediately and have mixed fills; maker-only pricing requires a separately supported order contract. The [official trade contract](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/trade) defines `LIMIT_MAKER` as rejecting an immediate taker match and exposes `quantity` or `quoteOrderQty` for market orders. Do not double-add buyer/seller to a hypothetical order-specific response that already includes side; no test order was called.

The [Binance BNB-payment support FAQ, questions 2–4](https://www.binance.com/en/support/faq/detail/115000583311) says the conversion updates at fills and, for non-BNB pairs, uses an average rate rather than the spot rate. It describes a token→chosen quote→BNB calculation. It does **not** identify an API response as the exact internal fee-conversion rate/window. The public [average-price endpoint](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/market#current-average-price) returns `mins`, `price`, `closeTime`; those fields alone do not prove equality to that internal average. A BNBUSDT quote can support a disclosed current estimate after source qualification, but cannot be labeled the guaranteed fee-conversion rate. The support FAQ itself cautions that it is informational and may be outdated; any exact engine-equivalence claim needs additional official evidence.

## 2. Rounding, bounded estimates and actual limits

[Exchange information](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/general#exchange-information) supplies `baseCommissionPrecision` and `quoteCommissionPrecision`; the [official glossary](https://developers.binance.com/en/docs/products/spot/faqs/spot_glossary) defines these as commission calculation decimals. They are distinct from asset quantity precision and `LOT_SIZE`. The checked contracts do not establish a complete rounding operator, family aggregation order, per-fill versus aggregate rounding, or the BNB fee precision mapping for BTCUSDT/ETHUSDT. BNBUSDT metadata with precision 8 is a useful candidate, not proof that every third-asset fee uses that field. Convert's asset quantity precision is also a different contract.

Implementation inference: keep arbitrary-precision decimal coefficients and units throughout; round displayed/authorized costs outward under an explicit, justified estimate policy. A ceiling of one aggregate is not automatically a hard bound on separately rounded fills. If a claimed hard bound needs an allowance per fill, it also needs a justified bound on fill count; an order quantity step does not by itself prove a minimum trade size. Missing rounding detail must remain an explicit model uncertainty, not an invented zero cost or a permanent prohibition on useful conservative estimates.

| Fact | What can be established | What must not be inferred |
| --- | --- | --- |
| Current conservative estimate | Fresh, bound terms plus a stated price/quantity/conversion model; complete roles, sides, families and conditional assets | Future unchanged rates, price or charging asset |
| Authorized fee/funding allowance | User-reviewed native asset amounts and purpose-qualified workspace values | An automatic broker fee cap or BNB-payment preference |
| Fixed BASE BUY limit | Submitted quantity and limit can bound principal QUOTE outlay by `Q × limit`; received-BASE fee estimate uses `Q` | This principal bound also bounds BNB conversion or future fee-schedule changes |
| Fixed BASE SELL limit | Submitted quantity bounds BASE sold; limit constrains the minimum acceptable price | `Q × limit` caps actual proceeds, USDT fees or BNB fees under price improvement |
| Market amount | Native submitted quantity/quote quantity is a real order parameter | A separate TradeX `maximum_spend` becomes a broker parameter; quote-quantity execution is an exact full-fill promise |
| Realized fee | Trade facts with exact commission amount, asset, role and execution identity | A pre-trade estimate settles a fee or releases capacity |

The SELL result above is a mathematical consequence of a minimum selling price, not a new product rule. A fee percentage can be described as a function of realized proceeds without inventing a finite absolute fee ceiling. Qualification of such a **current estimate** remains distinct from any guard that genuinely requires an absolute authorized debit bound.

[PRICE_RANGE execution rules](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/faqs/price_range_execution_rules.md) do not repair that distinction: the reference is recalculated when the order enters its taker phase, then applies through that phase. Missing direction multipliers or a null reference can mean non-enforcement. A checked snapshot upper price is useful conditional envelope evidence, not a user-fixed maximum for a later phase. An admission filter on submitted order notional likewise must not be repurposed into a guaranteed fill-proceeds cap. Post-fill [account trades](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account#account-trade-list-user_data) supply commission/commissionAsset and execution facts; future truth stays there.

## 3. Exactly which conversions are required

This table proposes purpose distinctions for the next specification; it does not change current §41.49 semantics or promote an existing source quality.

| Purpose | Required input/path | Complete owning treatment |
| --- | --- | --- |
| Principal policy valuation | USDT→configured workspace fiat for BTCUSDT/ETHUSDT, on BUY and SELL | A selected live-risk-qualified directional source; original bid/ask, provider/receipt times, quality/depeg and conservative valuation methodology. Workspace currency is configurable; USD is only the creation default. |
| Existing BUY funding | Sufficient exact-account free USDT minus local USDT reservations | USDT→USDT identity needs no external conversion. An observed USDT row names the unit; amount, completeness, freshness and commitments still need proof. |
| Missing BUY funding | An actual operation to acquire the deficit in USDT | A price feed cannot execute it or prove funds on Binance. Keep funding unavailable until the exact permitted operation, costs, wallet and lifecycle exist. |
| BUY fallback fee | BTC or ETH received | Native fee reduces received BASE; it is not extra USDT principal. Policy valuation still needs explicit BASE→workspace evidence or a validated, fully recorded valuation path. |
| SELL fallback fee | USDT received | Native fee reduces proceeds; include conservative net proceeds and policy fee value. Do not add the same fee again to sold BASE. |
| Conditional BNB payment | Received-asset→BNB estimate plus BNB holding/allowance; BNB→workspace where policy consumes it | Include total standard-discounted + tax + special, sufficiency, conversion uncertainty, fallback and extra fee funding capacity. A zero BNB balance does not imply missing USDT funding. |

Derived directional policy: for a direct product quoted as fiat per USDT/BNB, use the conservative **ask** when valuing a liability/replacement cost and **bid** when valuing liquidation proceeds. Converting a fiat allowance into a maximum token amount divides by the applicable conservative cost and rounds the allowance down. This is an explicit prospective policy, not an existing broker conversion. An observed spread is measurable; execution commissions, transfers and other unobserved costs cannot be invented or silently set to zero.

Treat same-product reverse use as an explicit direction with sides/units, never a free inferred reciprocal. A multi-leg path must preserve every selected source, timestamp, side, spread, cost and rounding stage and pass its own approved route contract. The currently separate EURUSD/USDEUR observations are not interchangeable. No USDT/USD parity, BNB/USD bridge or unsupported workspace-fiat route is created by this report. An unrelated portfolio asset is not a placement dependency; a fee asset consumed by this exact intent is.

For real funding, [Binance Convert market data](https://developers.binance.com/en/docs/catalog/core-trading-convert/api/rest-api/market-data) documents pair/amount limits through `/sapi/v1/convert/exchangeInfo`. [Convert trade APIs](https://developers.binance.com/en/docs/catalog/core-trading-convert/api/rest-api/trade) document signed TRADE `POST /sapi/v1/convert/getQuote`, an offered `quoteId`, exact returned amounts and expiry, then `POST /sapi/v1/convert/acceptQuote` and order-status/history. These paths show what a genuine funding operation would require; they neither establish USD/USDT convert availability nor reproduce automatic Spot BNB fee conversion. No Convert endpoint was called. Acceptance is a separate financial action with independent authority and settlement, never a concealed step in reading fees.

## 4. Official documented / observed / unproven evidence

All market probes below were bounded anonymous GETs to the documented first-party endpoints. No private credential, account API, order, test-order, financial operation, alternate Binance host or access bypass was used. Every abbreviated time below is UTC on **2026-10-10**. Recorded receipt is response-read completion; first-byte receipt was not separately measured, so these probes are not production first-receipt evidence. Book `time` and Alpaca quote `t` are supplied market timestamps; HTTP `Date`, request start and Binance metadata `serverTime` cannot replace them. `avgPrice.closeTime` would be last-trade time, not a fresh collection time. HTTP 200 proves technical availability, not data-use rights, sufficient liquidity, account permission or execution qualification.

| Source/path | Documented | Observed in this research | Still unproven |
| --- | --- | --- | --- |
| Binance exact-symbol [commission GET](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account#query-commission-rates-user_data) | Signed USER_DATA, weight 20, Database; all rate families and discount declaration | Official pinned example inspected; no actual-account request | Actual current account terms, rounding and fill-time charging |
| Binance ordinary [BNBUSDT exchangeInfo](https://api.binance.com/api/v3/exchangeInfo?symbol=BNBUSDT) | Public [exchange metadata](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/general#exchange-information) | HTTP 200 at `18:10:05.925820Z`; `TRADING`, BASE BNB / QUOTE USDT; commission precision 8/8 | Fee engine average/precision equivalence; account eligibility/rights |
| Alpaca [latest crypto quotes](https://docs.alpaca.markets/reference/cryptolatestquotes-1), `/v1beta3/crypto/{loc}/latest/quotes` | Locations include `us`, `us-1`, `eu-1`; bid/ask quote API | Fixed `us` request for USDT/USD,BNB/USD: HTTP 200 at `18:08:39.594672Z`; only USDT/USD returned | BNB/USD omitted in this response: no permanent unsupported conclusion. Other locations were not probed. |
| Alpaca [observed direct quote](https://data.alpaca.markets/v1beta3/crypto/us/latest/quotes?symbols=USDT%2FUSD,BNB%2FUSD) | Quote presence is stronger than generic supported quote-currency documentation | USDT/USD bid/ask fields and provider `15:49:00.308691877Z`; observed prices omitted | Over two hours old at receipt; cannot qualify current live risk. Rights and fee/funding execution are independent. |
| Coinbase [product metadata](https://docs.cdp.coinbase.com/api-reference/exchange-api/rest-api/products/get-single-product) | Public `/products/{product_id}` | USDT-USD and BNB-USD HTTP 200, online, trading_disabled=false at `18:08:39.794791Z` / `18:08:40.122155Z` | Selected-source configuration, user/account permissions and rights |
| Coinbase [USDT-USD level-2 book](https://api.exchange.coinbase.com/products/USDT-USD/book?level=2) | [Aggregated book](https://docs.cdp.coinbase.com/api-reference/exchange-api/rest-api/products/get-product-book): price, total size, sequence and time | HTTP 200; bid/ask/size/sequence/time fields observed; receipt `18:08:39.910943Z`; market values omitted | No amount-specific liquidity qualification or fill/funding promise |
| Coinbase [BNB-USD level-2 book](https://api.exchange.coinbase.com/products/BNB-USD/book?level=2) | Same documented public book contract | HTTP 200; bid/ask/size/sequence/time fields observed; receipt `18:10:05.217658Z`; market values omitted | Same limitations; not Binance's BNB commission conversion |
| Coinbase [USDT-EUR](https://api.exchange.coinbase.com/products/USDT-EUR) / [BNB-EUR](https://api.exchange.coinbase.com/products/BNB-EUR) metadata | Same product endpoint | USDT-EUR HTTP 200 online at `18:10:05.431517Z`; BNB-EUR HTTP 404 at `18:10:05.615450Z` | USDT-EUR book not probed. BNB-EUR response does not establish permanent absence or an alternative route. Other fiat coverage untested. |
| ECB OD-006 | Repository Backend §41.9/§41.40 retain informational reference-rate purpose | No probe; no new execution source | Intraday execution qualification; unchanged `BLOCKED_EXTERNAL` boundary |

Observation-body SHA-256 values, retained here without additional evidence files:

```text
Alpaca combined quotes  e78902a3d9caeae8f54443d9e0394db7ae556901525b470dfad402237677f865
Coinbase USDT-USD metadata  d9aec6e775857172e6ad3c8f3af390b95af78dd4cb4d27dcc548fe899e15ea42
Coinbase USDT-USD book  bc9621875a45c45cd6a3b27f3d146784583f36450720151f447c0b9efe94da29
Coinbase BNB-USD metadata  88153524807b6879953c2ea39d0a5c119d28ef76be6687d2f51e6de6e1c75198
Coinbase BNB-USD book  cb96673cefe0d2ea634fefcd9f7a81c0e0069b3f3b86c461d5e54d3ae88108b8
Coinbase USDT-EUR metadata  b29bbc98ee22a652d69d7b83c613afcc19dc34c7254ed94eff67c1554ef12d57
Coinbase BNB-EUR 404  78b49965ff92e946ac350d326d6f398777ec3da2e89ddacf20d04ece20efff1e
Binance BNBUSDT metadata  c33787228fc14f61353e1c8b7b4087846c1fd535460874673798518a2c55cecb
```

The source documents and public records establish candidates. No provider was selected, saved connection added, rights granted or recommendation to purchase a license made.

Parent validation addendum, checked2026-10-10: [Coinbase Market Data Terms](https://www.coinbase.com/legal/market_data), updated2026-08-07, §§3.3/3.5 require prior written consent for specified valuation and AI/automated-system uses. No such permission was obtained. Thus its public books are technical candidates only and cannot be adopted by default for TradeX live risk/AI use. This is a documented source-use dependency, not an API or full-goal impossibility. No further Coinbase market probes or integration are authorized by this report.

Scope clarification: Convert documentation is contrast evidence only. S29 does not add funding conversions, transfers, withdrawal or a new broker. Binance principal funding requires sufficient existing USDT; deficits remain unavailable until the user funds externally and TradeX observes it. Any future in-app conversion would require an independently approved scope and financial lifecycle.


## 5. Exact current implementation dependencies

Pointers refer to the pinned source SHA, not any user database or actual account. Code inspection only:

| Current code | Owning consequence for the next complete slice |
| --- | --- |
| [spot_commission.rs:76, 417](../../src-tauri/src/spot_commission.rs) | Complete three-family declaration, discount strings/flags and two UID/symbol-bound receipts; provider observation time absent. No balances, fee arithmetic or guarantee. Consume its exact original binding; do not reclassify source-only success. |
| [spot_fee_fx.rs:251, 362, 402](../../src-tauri/src/spot_fee_fx.rs) | Owning derivation still takes account comparison rates from capacity, not S29.10 symbol terms. `FeeSituation` has no qualified arm; expected fees remain absent. Own the genuine positive derivation here, replacing the comparison-only input without bypassing identity/expiry gates. |
| [spot_capacity.rs:141, 676](../../src-tauri/src/spot_capacity.rs) | Account balance rows are validated, but only selected BASE is retained when MAX_POSITION needs it. No owning QUOTE/BNB fee-funding projection. Reuse account identity/receipts; do not pretend an existing BASE row proves all holdings. |
| [financial_sources.rs:162, 224, 288, 660, 1364](../../src-tauri/src/financial_sources.rs) | Proposal funding becomes identity when a USDT balance row exists; that is a unit choice, not sufficiency. Producer supports EURUSD/USDEUR only. Transaction qualification and broker conversion costs remain separate blocked capabilities. |
| [protocol.rs:1895, 2349, 3797, 3944, 4433](../../src-tauri/src/protocol.rs) | Required-route purposes omit fee conversion/fee valuation; FX pair is six characters and currencies three, so USDT/BNB routes need a synchronized schema change. One `EstimatedFee` and one reservation amount/unit cannot express mutually conditional fee assets and separate BNB allowance. |
| [lib.rs:1841, 2060, 8623, 9000](../../src-tauri/src/lib.rs) | BUY capacity uses free QUOTE; SELL uses available BASE. Local reservations sharing account/capacity key are subtracted once. Limit expected/max value is `Q × limit`; market uses quote/maximum_spend. Prepare creates one capacity reservation, with no BNB leg. |
| [binance.rs:2573, 2988](../../src-tauri/src/binance.rs) | `maximum_spend` locally checks modeled notional; submit serializes quantity/quoteOrderQty and limit parameters, not that separate maximum. A true principal bound must match the actual native payload. |
| [risk.rs:1572, 1800, 2313](../../src-tauri/src/risk.rs) | Monetary guards consume workspace values. `trusted_workspace_notional` requires complete portfolio plus healthy verified, timely, non-depegged FX; qualifying a fee/FX statement alone would not fix this second owning consumer. Keep common provenance and consistent values, including reserved-capital policy. |
| [storage.rs:15400](../../src-tauri/src/storage.rs) | Settlement preserves all fees but reduces commitment using only the fee matching its reservation unit. Third-asset reservation accounting must preserve exactly-once trade identity, cumulative evidence, no premature release and unknown reconciliation. |
| [storage.rs:425, 2018](../../src-tauri/src/storage.rs) / [protocol.rs:51](../../src-tauri/src/protocol.rs) | Workspace base is user-configured three-letter fiat, default USD. Do not constrain the full product to USD or silently invent coverage for other currencies. |

## 6. Recommended next complete owning vertical slice

Take one **complete conditional Spot fee and purpose-qualified financial-input slice** through `to-spec → to-tickets → implement → code-review`, with one owning implementation ticket. Its purpose is to make positive, approval-bound per-asset fee/required-route qualification genuinely reachable from real external producer responses once the prerequisites below hold; another permanent UNQUALIFIED source holder would not complete this work.

1. Specify one fee model for both BTCUSDT/ETHUSDT BUY/SELL, preserving all families, role/side, returned multiplier, expected amount in each possible charging asset, received-asset fallback, BNB total sufficiency and per-asset rounding uncertainty. Separate current conservative estimate, stated envelope, native principal bound, authorized fee allowance and realized truth. Synchronize the English/Chinese owning contracts before changing their wire meaning; do not silently weaken an absolute bound that an existing guard actually needs.
2. Extend the trusted holdings/reservation consumer to exact QUOTE/BASE/BNB evidence with original identities, free/locked units, receipts and local commitments. Atomically consume approval and reserve the actual funding legs needed by the complete conditional model; do not double-subtract broker locks. Do not present a local BNB allowance as a venue-enforced automatic fee cap. Revalidate changes and reconcile each asset from authoritative fills.
3. Define a route per actual purpose, with direct selected-provider metadata, permission/rights and live-risk-quality evidence, directional book/quote provenance, amount/depth relevance, conservative cost methodology, depeg checks and expiry. The new producer must be able to return **qualified** evidence when these conditions are met; success must feed both the statement and existing monetary-risk/Prepare consumers. Add fee routes and widened exact asset/pair units through generated Rust/schema/TypeScript contracts. Reject only actually required missing routes; pre-funded USDT needs no invented funding conversion.
4. Use the public producer seam for positive normal scenarios with complete fee/holding/route responses, both charging branches and unchanged native bounds, then real missing/stale/contradictory/changed-source failures. Verify review digest, atomic refusal/no partial reservations, immutable captures, multidimensional settlement and actual React disclosures. Later authorized provider acceptance still has to verify account terms, data rights, funding, immediate preflight and fill lifecycle; fixtures do not provide that acceptance.

**Prerequisite evidence to obtain, not assumptions to encode:** the user's selected permitted source and actual live-risk-use rights/quality for each required route; actual account holdings/terms; a pinned BNB conversion estimate methodology plus honest uncertainty or stronger official engine-equivalence evidence; per-asset rounding semantics or a justified conservative estimate envelope; an explicit mapping of any absolute authorized debit bound to enforceable native behavior. For a real funding deficit, exact permitted conversion availability and its authorized lifecycle are additionally required. A fresh third-party quote alone satisfies none of those funding-operation obligations.

Research establishes where those inputs can come from and what can be implemented now at the specification/model boundary. It does not authorize new account/source configuration, obtain rights, guarantee conversion or certify fee funding. Keep unavailable combinations explicit while completing supported combinations positively; unsupported combinations do not excuse leaving the full owning capability as an always-unqualified projection. Unresolved full-goal fee/BNB/required-FX branches remain pending and must not be counted as completed by acceptance of a supported branch.

The research agent added only this file and performed no commit or issue mutation. Parent validation then added the license/scope/Alpaca EU findings and planning handoff. No production code, prototype or normative contract was changed; no app, test, build or UI was run; no credentials, private account calls, Arm, consent or financial operations were used. Findings are subsequently recorded on dev as research only.

## 中文配对结论（简版）

本研究以 `dev@8ad62d7d02f84a34ee47c460d4911eb829e94322` 为基线，核查日期为 **2026-10-10**；币安官方 Spot 文档固定在 `263ac1aa96556af0b4da824c82a1d8cd9a0edda9`。它是研究证据，不升级 S29、金融验收或发布状态；**原型代码未改**。

1. **手续费模型可实现。** 保留 standard/tax/special 的 maker/taker 与 buyer/seller 全部声明。BUY 按收到的 BASE 数量、SELL 按实际收到的 QUOTE 金额计算。官方 FAQ 的 `0.25` 与 REST 的 `0.75` 是不同示例值；原始返回值作为标准手续费的支付乘数，仅折扣 standard。BNB 需覆盖全部转换后费用，不足时保留收到资产的全额收费分支；本研究未读取真实账户费率。[官方手续费 FAQ](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/faqs/commission_faq.md)
2. **估算、授权额度、券商强制上限、实际费用必须分别表达。** SELL 限价不能作为成交收入或手续费的绝对上限；PRICE_RANGE 在 taker 阶段重新计算。现有 `maximum_spend` 没有作为独立参数发送给币安。佣金精度不等于完整舍入规则；现价或 `/avgPrice` 也未被证明等同于币安内部 BNB 平均换算价。缺失证据应明确保留，不应永久否定符合 RevC 的保守当前估算。[PRICE_RANGE](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/faqs/price_range_execution_rules.md)、[BNB 换算说明](https://www.binance.com/en/support/faq/detail/115000583311)
3. **真实来源候选已存在。** Alpaca HTTP 200 返回 USDT/USD，但报价已过时两小时以上；同一响应缺少 BNB/USD，不证明永久不支持。Coinbase 的 USDT-USD、BNB-USD 元数据和订单簿均返回 200；USDT-EUR 元数据返回 200，BNB-EUR 当次返回 404。币安普通主机 BNBUSDT 元数据返回 200/TRADING。详细时间、路径和摘要见英文证据表。200 仅证明技术可访问，不授予数据权利、账户权限、流动性或执行资格。[Alpaca API](https://docs.alpaca.markets/reference/cryptolatestquotes-1)、[Coinbase API](https://docs.cdp.coinbase.com/api-reference/exchange-api/rest-api/products/get-product-book)
4. **只检查该意图真正需要的换算。** 已有足额 USDT 的 BUY 资金路径是 USDT→USDT；USDT→工作区本位币是政策估值。BASE/BNB 费用另有相应估值或收费换算需求。真实资金不足需要独立受权的资金操作，行情不等于资金兑换。工作区本位币可配置，USD 仅为默认值；ECB OD-006 仍仅供参考，不假定 USDT=USD，也不自动拼接路径。
5. **下一步应交付一个完整 owning 切片。** 将三族条件费用、BASE/QUOTE/BNB 持仓与逐资产预留、用途限定且能真实正向资格化的 FX、现有货币风控/Prepare 消费端一起规范并实现。前提是实际来源权利与质量、账户事实、舍入/BNB 换算方法和原生授权边界得到证据支持；通过公共外部生产者接缝验证正向与失败场景。不能再以始终 UNQUALIFIED 的来源容器替代完整目标。这里只新增本研究文件，未选择新供应商、配置账户、运行应用/测试或执行任何金融操作。


父代理复核补充：Coinbase [市场数据条款](https://www.coinbase.com/legal/market_data)（2026-08-07版）§3.3/§3.5对特定估值及AI/自动系统用途要求事先书面同意，本任务未取得；其200记录仅为技术候选，不默认接入金融风控或AI。此为真实使用权依赖，不是市场/API永久不可用。Convert仅作对照，S29不新增兑换、转账、提款或券商；USDT不足仍须用户在外部补足后由应用观察。


## Parent handoff — next owning requirements, not implementation acceptance

Additional anonymous, bounded first-party [Alpaca EU dataset probe](https://data.alpaca.markets/v1beta3/crypto/eu-1/latest/quotes?symbols=USDT%2FUSD,BNB%2FUSD) used the [documented location parameter](https://docs.alpaca.markets/us/docs/real-time-crypto-pricing-data), with no credential, selection or automatic fallback. Request started2026-10-10T18:28:09.484369Z; response-read completed18:28:09.879320Z, HTTP200. Only USDT/USD was returned, with provider timestamp18:27:59.219702983Z, roughly10.66seconds old at response-read completion. BNB/USD omission is not permanent capability proof. Body SHA25644d4a99c8f35288484b5300705d68e2806b04c5520aaa7a07932359ac0c55af3. This establishes another concrete existing-provider candidate, not current policy eligibility, permitted-use proof, first-byte receipt or production financial acceptance. Do not increase an existing age guard merely to pass this sample.

The next single owning specification must resolve and implement all of these requirements at the accepted public seam. This list is a planning handoff, not a published ready implementation ticket:

1. Consume the exact symbol declaration, including all role/side/family rates and its original account/source/Proposal identity; keep account comparison historical.
2. Pin the returned discount multiplier and preserve non-discounted tax/special components, mixed maker/taker roles and received-asset fallback.
3. Compute original-asset current fee estimates with checked exact decimals; make estimate policy, native precision and rounding uncertainty visible.
4. Separate estimated fees, authorized principal/fee allowances, broker-native quantity/price bounds and realized fill/fee truth; never describe SELL limit proceeds as an upper bound.
5. Resolve conditional BNB conversion methodology and allowance semantics explicitly. Unknown internal-average equivalence cannot be replaced by a spot-price guarantee.
6. Retain current exact BASE/QUOTE/BNB holdings, original receipts, available/locked units and generation; absent or duplicate required data remains unavailable.
7. Consume one native principal funding proof; existing free amounts already exclude broker locks, and local reservations are subtracted once.
8. Add actual required conditional fee-funding reservations and exactly-once per-asset settlement without changing pending/unknown recovery safeguards.
9. Derive only intent-policy, actual native funding and actually consumed fee purposes; keep unrelated portfolio routes separate unless a specific selected policy consumes them.
10. Use pre-funded USDT for Spot principal; do not add Convert, transfers, withdrawals or a new broker as an implicit dependency.
11. Widen FX asset/pair wire contracts coherently, preserving supported fiat base choices, explicit sides, original bid/ask/size/timestamp/cost, route and depeg provenance.
12. Establish actual permitted live-risk-use source/entitlement before positive qualification. Coinbase requires specific written permission for relevant uses; API200 alone is insufficient. Existing Alpaca candidates still need actual configured-source/rights/freshness validation.
13. Define every conservative valuation/fee model, liquidity and genuinely relevant cost; no parity, zero unknown costs, unapproved inverse/bridge or snapshot funding promise.
14. Make a genuine positive owning qualification reachable from real supported producer responses when all required inputs exist, and feed the same evidence to monetary risk and Prepare. Do not close this by retaining an always-unqualified projection.
15. Distinguish approval-material financial changes from immutable collection provenance; fresh private revalidation must neither silently renew old consent nor create a receipt-only approval loop.
16. Revalidate exact identity/policy/generation/time/source and the actual authorized debit envelope before Prepare/Gateway; reject a changed or unavailable input without partial state.
17. Preserve current/captured/pre-arm disclosures and immutable histories, including independent unresolved duties; keyboard1280/768/390 and no overflow/errors remain mandatory.
18. Public producer-seam TDD must demonstrate positive supported fees/routes and both conditional funding outcomes, plus wrong/incomplete/stale/changed/late/permission/cost/depeg cases and atomic refusal.
19. Final schema/complete/affected checks, ordinary build/pin/frozen UI, serial Standards→Spec and exact committed-source/remote/tracker handoff bind acceptance; no new financial authority setter.
20. Keep the entire S29/map/real-provider/physical/release duties open until their own evidence is obtained. Supporting one legitimate branch does not waive unresolved full-goal branches.

Standing human approval for the existing public external-producer seam and one-ticket serial granularity applies; no repeated seam/granularity interview is needed. Source choice, financial boundary or permission remains an evidence duty where not yet established. No feature or source profile was implemented/selected by this research.

中文交接：补充Alpaca `eu-1` 匿名探测HTTP200，只返回USDT/USD，回读时约10.66秒旧；BNB/USD缺失不证明永久不支持，未选择来源、取得使用权或升级资格。下一完整拥有方规范须覆盖精确三族费额、折扣乘数、估算/授权原币/原生边界区分、BNB平均价不确定性、BASE/QUOTE/BNB持仓及逐资产预留结算、真正必要且有使用权与质量证据的FX、货币风控/Prepare一致消费及真实正向公开验证。预先足额USDT仍是Spot资金前提，不暗加兑换/转账/提款/券商。上述20项为待落实清单，不是已实现、ready票或金融PASS；沿用已批准单票串行与现有公开接缝，完整父项/全图/真实/物理验收仍开放。
