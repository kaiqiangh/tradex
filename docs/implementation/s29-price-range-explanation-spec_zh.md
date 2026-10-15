# S29.7 — Proposal PRICE_RANGE 执行规则解释

父项：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。实现/复审起始基线`dev@a555fe1de526ca380d782522a705b2055c2184c4`。[English](s29-price-range-explanation-spec.md)。仅规划，尚未实施或验收。

## 问题

普通 Binance Live 不变 Proposal 已能显示静态过滤器、容量与时段输入，但PRICE_RANGE尚无解释。用户不能判断哪些执行方向有实际边界、参考输入为何缺失/过期，以及当前范围是否保证未来成交。现有要求全部四个字段的元数据解析也不能表示官方的部分配置。

## 方案

沿用当前/捕获/pre-arm Proposal规则，完整展示实际配置、真正需要的提供方参考价、所选side精确快照边界及不实施/不可用原因，并解释未来执行不确定性。快照预览不成为下单资格或预期成交。

## 用户故事

1. 作为用户，我希望绑定不变Proposal/账户/symbol，避免借用其他意图配置。
2. 作为用户，我希望区分BUY/SELL实际配置方向，避免制造缺失配置。
3. 作为用户，我希望提供方参考价区别于book/trade/average，避免其他价格资格化执行预览。
4. 作为用户，我希望精确十进制乘法和QUOTE/BASE单位，避免精度或币种假设改变边界。
5. 作为用户，我希望原始零区别于字段缺省，避免制造disabled方向。
6. 作为用户，我希望实际absence/显式null区别于错误，避免失败表现为不实施。
7. 作为用户，我希望明确当前快照与未来不确定性，避免把预览当成交保证。
8. 作为用户，我希望只显式刷新真正需要的公开输入，避免页面读取产生不必要请求。
9. 作为用户，我希望配置/参考来源、首次收据/material可识别，便于评估和追溯。
10. 作为用户，我希望malformed/重复/未知项失败关闭，避免部分解析认证意图。
11. 作为用户，我希望pending/失败撤下当前预览，避免旧成功继续当前化。
12. 作为用户，我希望过期/全部绑定与拥有方变化撤下预览，避免get续收据。
13. 作为用户，我希望捕获历史在刷新/reopen后不变，避免历史变成新authority。
14. 作为用户，我希望百分比/notional的独立参考回退保留，避免执行purpose破坏静态规则。
15. 作为用户，我希望容量、金额及其他金融义务继续显示，避免单项预览审批订单。
16. 作为用户，我希望精确CANCEL/reconciliation保持既有边界，避免削弱保护动作。
17. 作为用户，我希望keyboard1280/768/390解释可读，使限制和值同样可见。
18. 作为reviewer，我希望公开RED/GREEN与精确源码检查/构建/UI/串行审查，避免标签或私有setter成为验收。

## 实现决策

- 沿用已确认单票、最高公开React→类型化Control Plane→所属临时SQLite/outbox→外部HTTP/vault/WS生产者接缝；扩展既有Proposal规则/元数据/参考/当前/捕获契约，不创建平行authority或新账户/source流程。Renderer仅workspace/Proposal/CAS，不接受URL/symbol/reference/bounds/multipliers/time/qualification。
- 绑定普通Binance Live BTC/USDT或ETH/USDT不变意图/hash、已保存账户/source/rule material、market/policy/workspace/session/time与刷新所有权。本地get无provider/vault读取；只在所选side实际配置需要时新增执行参考purpose，无universe扫描或private读取。
- 官方执行语义：此purpose只使用真正referencePrice响应。缺省multiplier对应方向不实施；无PRICE_RANGE或显式null参考亦按契约不实施。缺失/失败/未知lookup不是absence证明。交易所在taker阶段重算边界，越界执行可能使订单expire；当前快照不只因Limit意图的限价就拒绝订单，也不建立未来成交/准入。PRICE_RANGE不回退average/trade/book。
- 此规则仅将四字段必填改为官方可选字段语义；其余schema保持。实际字段保留精确十进制字符串，零是原始数值，不是disabled标记；重复/type/精度/range/未知active项拒绝或明确未解决。沿用512KiB响应、64规则、32字段、64字符decimal、8临时Proposal槽及既有有界参考；精确算术失败不可用，不经舍入形成成功。
- 提供有界类型化所选side配置、可选下/上快照界、USDT/BASE单位、真正reference/null/absence状态、原始provider时间/首次收据/material及预览/撤下/失败说明。明确absence需新鲜完整精确scope规则证据，不以漏项/截断推断。风险历史仅捕获批准aggregate/digest，无raw响应或runtime cache持久化。
- 沿用普通公开reference固定GET(weight2)、精确过滤规则来源及共享IP预算/cooldown。无key/UID伪造、redirect、source mutation、test order/probe、跨I/O的CP锁或weight reset。既有30秒deadline、12秒完成余量及首次收据/provider时间时效保持，不将localreceipt变成provider时间。其他静态purpose可用参考不能静默替代执行观察。
- 新预览pending/失败/过期/账户/key/source/rule/market/policy/Proposal/workspace/session/time变化、reopen及迟到/新拥有方结果撤下当前证据。捕获无poll/refresh或续收据/同意。百分比/notional显式null的独立回退及此purpose之外的既有行为保持。
- 即使配置/算术可观察，预览仍执行不资格化；不制造完整INSTRUMENT_RULES PASS、金额/风险authority、动态准入/预留、rights/quote/liquidity/permissions/health/feesFX/funding、Arm/consent/Prepare/Gateway或execution-phase timestamp。未来义务保持未解决；已证实不实施只是purpose事实，不是全面ready。独立精确CANCEL/reconciliation保持。
- 实际当前/捕获/pre-arm React解释身份/单位/配置/边界/null与fault/年龄/不实施/未来不确定性，提供显式恢复。keyboard1280/768/390及中英金融/wire/证据同步；原型代码未改。

## 测试决策

沿用公开account/source/draft/Proposal/rules refresh/get/risk与实际React，仅外部生产者fake。首先部分元数据与仅执行purpose真实公开RED，再GREEN后推进。覆盖完整/部分/empty、side/direction/原始零/十进制精度/边界包含、显式null与缺失/错误、不回退/不必要读取、未知/重复/malformed/overflow/bounds及错误symbol/identity；静态null回退回归独立。覆盖pending/失败/恢复/过期/reopen/绑定/迟到/新拥有方与不变捕获；正向预览仍无执行authority。实际当前/捕获/pre-arm keyboard1280/768/390。要求当前完整及受影响rule/capacity/interval/Hot/Gateway、IPC/配对追溯、普通构建/pin、独立串行Standards→Spec和精确源码/证据/远端/tracker交付；native/provider/financial/physical/prototype/main独立标记。

## 非本票范围

拥有方动态容量/准入/预留资格、金额意图/费用/所需FX、reference-price streaming、即时认证execution preflight、执行expiry/private-stream实现、真实订单/probe/Arm/consent、新账户/source、rights grant、S28/Testnet/Bitget范围变化、物理睡眠豁免、原型修复及main合并。这些完整goal义务由后续拥有方完成，不豁免。

## 补充说明

2026-10-09复核：[execution rules](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md#query-execution-rules)、[执行参考契约](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/faqs/price_range_execution_rules.md)。始终不资格化的快照解释与金融职责独立是本规范保守设计，不是provider保证。实际技术依赖为已完成元数据/Proposal参考解释；容量/时段完成只是串行调度，不制造技术阻塞。完整父项/map/S28/S17/物理S27保持OPEN。既有粒度/接缝批准持续有效，无重复问答。
