# S29.11 FX source contract research

Checked **2026-10-10**, planning baseline `dev@3a6db06729ca0fb3f01c73fe1b4cc68756bc342a`.
This note researches the open source duty in [the owning financial plan](s29-owning-financial-plan-spec.md), following [the previous financial research](s29-financial-inputs-research.md).
It does not select a real user's feed, grant rights, change contracts, qualify an account, or close an issue. English findings govern the paired Chinese conclusions below.

**Finding:** conditional adapters and route eligibility can be specified now. The human has confirmed own local non-commercial use, with Alpaca raw data neither redistributed nor sent to cloud models. That context alone does not establish this user's permitted activation/acceptance; it does not make all engineering decisions unknowable. The checked APIs do not expose a universal licence-verification field. Technical authorization, contractual applicability, market quality and financial purpose must remain separate facts.

## 1. Alpaca crypto: exact documented contract

[Latest-quotes OpenAPI](https://docs.alpaca.markets/us/reference/cryptolatestquotes-1.md), metadata updated `2026-05-27T17:58:03Z`, defines ordinary production `GET https://data.alpaca.markets/v1beta3/crypto/{loc}/latest/quotes?symbols=...`.
Required response is `quotes`, a symbol-keyed object; every quote requires numeric `bp`, `bs`, `ap`, `as` and string `t` (RFC3339, nanosecond precision).
Price/size fields have OpenAPI `double` format; a TradeX exact-decimal adapter should preserve JSON numeric tokens rather than convert through binary floating point.
The security alternatives are key/secret headers together or Basic authentication. Documented errors include 400/401/403/429/500 and quota headers.
The enum lists `us`, `us-1`, `us-2`, `eu-1`, `bs-1`; descriptions explain only the first, second and fourth. Do not invent an origin/profile for the unexplained entries.

[Crypto-stream documentation](https://docs.alpaca.markets/us/docs/real-time-crypto-pricing-data) identifies `us` as Alpaca US, `us-1` as Kraken US, and `eu-1` as Kraken EU, with an explicit warning that those upstream providers can change.
The ordinary stream is `wss://stream.data.alpaca.markets/v1beta3/crypto/{loc}`; quote events add `T=q` and `S` to the five quote fields.
Orderbook events carry `t`, bid/ask `p,s` levels and reset flag `r`; reset replaces the book and subsequent quantities replace/remove levels.
It does not publish Binance-style update IDs for an independently proved gap detector. A disconnected/reconnected stream therefore needs a new subscription/reset before a usable book.

Unit evidence: the reference calls `bs/as` bid/ask sizes but does not explicitly name their currency. The [first-party SDK quote model](https://alpaca.markets/sdks/python/api_reference/data/models.html#quote) calls them quote sizes; its trade model describes trade size as quantity traded.
**Inference, not an explicit wire-unit field:** for `USDT/USD`, price dimension is USD per USDT and sizes naturally describe USDT quantity. Specify that mapping as the adapter's documented interpretation; do not claim the API carries a `size_currency` or use equity round-lot multiplication. If the chosen financial guard requires independently explicit size-unit proof, this checked reference alone does not provide it.

## 2. Authentication, entitlement and current USDT/USD observations

[Market Data overview](https://docs.alpaca.markets/us/docs/about-market-data-api) requires authentication except historical crypto; Trading API uses `APCA-API-KEY-ID` and `APCA-API-SECRET-KEY`.
It explicitly distinguishes personal trading applications from Broker API business integration. Basic is the default Paper/Live plan, with stock/options coverage limits; that table is not a complete crypto/forex entitlement declaration.

[Market Data FAQ](https://docs.alpaca.markets/us/docs/market-data-faq) shows actual subscription denials for requested stock feeds; subscription management is in the provider UI.
[Streaming authentication](https://docs.alpaca.markets/us/docs/streaming-market-data) exposes successful authentication/subscription responses and errors, including insufficient subscription.
These are genuine observable access outcomes for the requested dataset, not proof of every permitted downstream use.
[Trading account OpenAPI](https://docs.alpaca.markets/us/reference/getaccount-1.md) exposes account identity/status and trade-block flags; `ACTIVE` is account trading status, not a market-data copyright licence or redistribution grant.
No entitlement/accepted-agreement/financial-valuation licence endpoint was established by these checked references. Do not invent one or treat a renderer boolean as its substitute.

Three bounded anonymous documented GETs were performed serially. No credentials or account APIs were used. Their HTTP success contradicts an assumption that anonymous access is always rejected; it does **not** supersede the documented authentication contract. No market price or size is published here.

| Requested path | Status and exact observed shape | Timing (UTC, 2026-10-10) | Implication |
| --- | --- | --- | --- |
| `/v1beta3/crypto/us/latest/quotes?symbols=USDT%2FUSD` | 200; `quotes.USDT/USD`; `ap/as/bp/bs` JSON numbers, `t` string | Start `20:53:32.843960Z`; headers `20:53:33.215494Z`; first body byte `20:53:33.215518Z`; complete `20:53:33.215815Z`; provider `15:49:00.308691877Z` | Direct pair exists in this response; source timestamp is over five hours old. |
| `/v1beta3/crypto/eu-1/latest/quotes?symbols=USDT%2FUSD` | 200; same exact five quote fields/types | Start `20:53:33.216048Z`; headers `20:53:33.572382Z`; first body byte `20:53:33.572409Z`; complete `20:53:33.572738Z`; provider `20:53:21.584313416Z` | Direct pair exists in this response; about 11.988 seconds old at first body byte. |
| `/v1beta1/forex/latest/rates?currency_pairs=EURUSD%2CUSDEUR` | 401; non-JSON body, no rate observation | Start `20:53:33.572992Z`; headers `20:53:33.915857Z`; first body byte `20:53:33.915886Z`; complete `20:53:33.916047Z` | Authentication needed for this request; no conclusion about either pair's current availability. |

Body SHA256, respectively:

```text
us      e78902a3d9caeae8f54443d9e0394db7ae556901525b470dfad402237677f865
eu-1    5af87a4294ec47b29382da6189f91b719bdd55dd6ee1729968da28df184ec365
forex   9371176869a945e2958e43b349397210a1b72b83f11c67e02e0be1f950254ef2
```

The first-body-byte measurement is when the application received its first readable body byte, not a promised kernel/network arrival time. Receipt cannot replace the supplied market timestamp. Both successful quote samples fail the existing default 3-second age boundary; neither is positive financial evidence. A producer may later deliver an eligible fresh event; do not relax policy to pass this sample, silently switch location, or mislabel stale success as entitlement failure.

## 3. Forex latest rates: valuation versus conversion

[Forex-latest OpenAPI](https://docs.alpaca.markets/us/reference/latestrates-1.md) defines `GET /v1beta1/forex/latest/rates`, required comma-separated `currency_pairs`, response `rates[pair]`, required numeric `bp/mp/ap` and string date-time `t`.
Descriptions are last bid/mid/ask at the timeframe's end. Security supports Basic or key/secret headers together. Examples are USDJPY/USDMXN; the schema publishes no exhaustive pair list, depth, size, quote expiry, conversion fee or executable quote ID.
Its `info.license` refers to the API definition; do not promote that documentation licence to all underlying market data.

**Design inference:** a selected, permitted and fresh bid/ask rate can be evaluated as an explicitly approved **policy valuation** model. It cannot prove actual funding conversion, transferred USDT, amount-specific execution liquidity or broker costs. `mp` alone is not a conservative directional cost.
The owning specification must say whether a particular valuation guard requires sizes; if it requires executable amount coverage, this price-only endpoint cannot meet that guard. If policy valuation is intentionally price-only, mark execution depth/cost **not applicable to this purpose**, with its real reason, rather than fabricated zero fees/sizes or a transaction-grade liquidity claim.
Do not assume EURUSD implies USDEUR, assume any arbitrary workspace-fiat pair exists, or silently use a reciprocal/bridge. Unsupported/missing/stale required fiat legs remain unavailable; configurable workspace base remains intact.

## 4. Applicable use facts: what the terms actually say

[Alpaca Terms & Conditions](https://files.alpaca.markets/disclosures/library/TermsAndConditions.pdf), pp1–2, cover the API/content and supplement customer/account agreements. Personal/non-commercial use is stated; making an app available to others calls for advance notice, while publication/distribution/commercial copying requires prior written consent. The U.S.-resident statement expresses intended audience and non-solicitation. Third-party content is informational and supplied without accuracy/timeliness guarantees. No checked clause expressly forbids a personal local arithmetic valuation. This is a source reading, not a determination of this user's rights.

[Paper documentation](https://docs.alpaca.markets/us/docs/paper-trading) expressly offers Paper Only signup globally, but describes Paper Only API use as paper trading and its data entitlement as IEX. It is not an explicit universal crypto/forex licence for independent Binance Live risk. [International account documentation](https://docs.alpaca.markets/us/docs/international-accounts) also describes non-U.S. taxpayer onboarding. Thus neither a universal worldwide grant nor an absolute all-data non-U.S. ban follows from these pages.

**Concrete evidence for an actual local-use profile:** identify the provider account/product and selected dataset; inspect that customer's applicable agreement/plan/disclosures and explicit grant/limitations; combine them with the human's actual use description (own account versus others, commercial versus personal, local-only versus model transmission/export). Any necessary provider permission is real provider correspondence/agreement, not an agent-generated approval flag. Where ordinary personal use is covered by the applicable terms, do not demand a nonexistent bespoke “financial valuation licence.” Where terms remain conflicting/unclear for the actual product/use, do not assert permission from HTTP200.

The human answered on 2026-10-10: solely own local non-commercial use, with Alpaca raw market data neither redistributed nor sent to cloud models. This supplies actual-use context only. It cannot by itself prove a licence, account jurisdiction, provider acceptance, entitlement, or quality. Conversely, requiring actual personal/commercial details to design a parser would manufacture an engineering blocker.

Local Rust valuation, model transmission, external export and redistribution are different consumption paths. [Alpaca's own Trading API MCP documentation](https://docs.alpaca.markets/us/docs/trading-api-mcp) documents AI-client market-data integration, but does not replace this account/dataset's applicable terms or authorize every redistribution. Implement explicit consumption constraints and test outbound behavior; do not infer a general AI ban from an accuracy disclaimer or general permission from an integration example.

## 5. Binance BNBUSDT as a fee-only internal source

[Pinned Spot REST](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/rest-api.md#order-book) documents ordinary public `/api/v3/depth?symbol=BNBUSDT&limit=...`, memory-sourced `lastUpdateId`, bid/ask string price/quantity levels, maximum5000; snapshot has no event timestamp. Weight is5 for1–100,25 for101–500,50 for501–1000,250 for1001–5000. Exchange metadata establishes the pair's BASE/QUOTE mapping; use the actual metadata/status, not its ticker name alone.

[Pinned Spot streams](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/web-socket-streams.md#diff-depth-stream) defines `bnbusdt@depth` or `@depth@100ms` on ordinary `stream.binance.com`, event `e/E/s/U/u/b/a`; timestamps default milliseconds. Buffer events before the REST snapshot, join its update ID, apply absolute quantities and delete zero levels. Missing sequence requires discard/resnapshot. First-receipt time is local; `E` is source event time. Initial finite depth limits known liquidity; lack of more levels cannot prove unlimited coverage. Ping/pong, rate limits and24-hour reconnection are lifecycle requirements. Book continuity needs both HTTP and WebSocket evidence; a REST success or untimestamped bookTicker alone cannot provide it.

**Design inference:** an internal purpose-qualified BNBUSDT source can reuse that ordinary producer lifecycle while BNB remains absent from the trading catalogue. Instrument catalogue eligibility controls order creation; financial-source eligibility controls a narrowly bound fee estimate/valuation. Keep separate allowlists and prove that fee-source discovery never enables BNB orders or account mutations. This adds no trading broker, Convert, transfer or withdrawal operation.
For a conservative current replacement valuation, the BNB ask in USDT and sufficient explicitly modeled BASE depth can value a BNB liability. Compose USDT→workspace through the selected eligible leg and retain every side/unit/timestamp/receipt/source. A received-asset→BNB fee model is a separately specified direction; do not equate this book with Binance's internal commission average, a guaranteed fill, or a fee cap.

[Current global Terms linked by Binance's terms page](https://bin.bnbstatic.com/static/cms/cg08ou2ak0tn7mcplvfg/file/bf4879710c904b991848972ec4818ba2cf9e4ce314c09adae84fa2750d3477f7.pdf), effective21July2026: §27 grants necessary non-commercial personal/internal-business IP use; §14.1.2(b) makes account API access subject to separate API terms/approval; §§1.3–1.4 include applicable product/local/rulebook terms; §§33.6–33.7 discuss third-party/automated technology without guaranteeing operation. Eligibility/prohibited-use conditions still apply. These establish a concrete ordinary-use framework, not a universal open-data/redistribution grant or proof of this user's applicable entity/permissions. Region-specific Japan/Square terms are not substituted for the global Spot account contract.

No Binance market probe, alternate host, authenticated request, account read, order/test order, Arm/consent or mutation was performed in this research. Prior BNBUSDT metadata observation remains historical, not new continuous-depth acceptance.

## 6. Concrete conditional eligibility contract for the owning specification

The following are **proposed TradeX decisions**, not fields allegedly supplied by Alpaca/Binance. Root must adopt and pair them before implementation.

1. **Selection:** existing saved provider credential reference plus explicit dataset/location, ordinary host, upstream origin and fixed allowed route/purpose. No automatic region/feed fallback or unexplained-location profile. Selection is user configuration, never proof of permission.
2. **Applicable source-use basis:** bind the actual provider agreement/plan/document version and its supported consumption purposes. Facts obtained from the human/provider UI/correspondence are recorded as such; authenticated producer access cannot impersonate them. No `rights=VERIFIED` setter, fictional licence endpoint or manual attestation upgraded into provider verification. Changes/expiry retire dependent observations.
3. **Access observation:** actual authorized source request/subscription result, credential/source generation, original response digest, quota/denial and request/first-receipt evidence. Success establishes this dataset's technical access at that time. Do not require private trading permissions for a public fee-only book.
4. **Market observation:** exact asset/pair/direction, decimal originals, supplied source timestamp, first receipt, clock validity, spread, required amount coverage and status/continuity where applicable. Reject missing fields, crossed/nonpositive prices, future/stale time, depeg, invalid units, missing required depth and late/superseded responses. Getters cannot renew time.
5. **Purpose qualification:** `POLICY_VALUATION`/current `FEE_ESTIMATE` can use an explicitly stated conservative market model. `FUNDING` requires already observed native funds or a separately supported real operation; this item adds none. Any execution-depth/cost requirement must have actual evidence; a valuation-only route cannot invent it.
6. **Route set:** USDT/USD can use the explicitly selected Alpaca crypto dataset when every condition passes. BNB/USDT can use the ordinary internal continuous book only where the fee expression actually consumes it. USD→workspace requires an explicitly selected supported directional forex observation for non-USD base, or an approved fully recorded equivalent route; identity USD→USD consumes no external rate. Every required leg passes; unsupported configured fiat remains unavailable without changing the workspace default or overall goal.
7. **Financial consumer:** the same qualified source model reaches owning fee arithmetic, native claim vector, monetary risk and Prepare. Capture unchanged original audit provenance separately from normalized financial material; missing permission/access/market evidence refuses qualification without partial claims or provider mutation.

**Positive reachability:** under an applicable documented source-use basis, a real configured credential path with successful exact-dataset authorization plus fresh complete observations can satisfy these conditions. Public-seam tests fake only external HTTP/vault/WS inputs and use normal configuration commands, not seeded licence/FX authority. Tests prove the implemented conditional path; they do not prove this human's live-use rights or real market freshness. If the real source-use evidence cannot be obtained through existing supported product configuration, the owning implementation must genuinely add its evidence intake and policy evaluation; a synthetic shortcut is not acceptance.

**Next action distinction:** source/route contracts and unresolved branches can now be specified concretely; no need to wait before designing typed adapters, per-asset reservation/settlement or purpose-bound eligibility. Root decides whether this resolves development-design readiness under the current issue. Actual profile activation, source-use acceptance and complete S29 financial acceptance remain unproven until the real user's applicable facts and genuine runtime evidence arrive. Keep those gates explicit instead of declaring the complete item finished or publishing an always-unqualified substitute.

## 中文配对结论

本研究仅补齐 S29.11 来源契约依据，不授予权限、不选择实际账户的数据源、不升级验收；原型未改。用户已于2026-10-10确认仅本人本地非商业使用、Alpaca原始行情不分发/不发送云端模型；这是用途上下文，不能单独证明账户协议、数据权利或entitlement。

- 官方最新 crypto 契约有 `bp/bs/ap/as/t`，需要认证；匿名200仅证明该次技术响应。`us/us-1/eu-1`来源已解释，其他枚举不可臆造来源；切换地区必须显式选择。[最新报价定义](https://docs.alpaca.markets/us/reference/cryptolatestquotes-1.md)
- 本次 `us` 的 USDT/USD 已超过五小时；`eu-1` 约12秒。均不满足默认3秒新鲜度。保留原时间，不能用收取时间重置、放宽门槛或把陈旧响应说成授权失败。
- Forex latest 有 bid/mid/ask/time，没有深度、换汇报价编号或执行成本。它可以进入明确的政策估值模型；不能证明真实补资、金额级执行流动性或资金到账。[Forex定义](https://docs.alpaca.markets/us/reference/latestrates-1.md)
- 官方个人用途条款、全球 Paper 和国际账户文档不能单独证明本用户所有用途，也不能把“美国居民意向”扩大成所有非美数据使用禁令。实际产品、适用协议、用途和权限事实仍待取得；没有通用 licence API。[Alpaca条款](https://files.alpaca.markets/disclosures/library/TermsAndConditions.pdf)
- BNBUSDT 可作为内部费用来源，需快照加连续深度事件、原单位及真实生命周期；此用途不开放 BNB 下单，也不等于费用引擎平均换算。[连续深度](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/web-socket-streams.md#diff-depth-stream)
- 条件式生产实现可以继续设计；实际激活/验收仍需要适用来源使用依据、真实认证及满足门槛的观察。模型发送、导出、再分发和本地控制平面估值分别判断，不能用用户复选框或伪造字段变成 VERIFIED。
- 完整费用、资金向量、结算、配置货币及真实 S29/全应用义务仍保留；不加入 Coinbase、Convert、转账、提款、替代 Binance 主机或隐含金融操作。
