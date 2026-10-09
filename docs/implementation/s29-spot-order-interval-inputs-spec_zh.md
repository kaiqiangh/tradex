# S29.6 — 精确账户时段订单额度输入

父项：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。已完成前置：[精确账户开放订单与容量输入](https://github.com/kaiqiangh/tradex/issues/126)。实现/复审起始基线`dev@91e2f2a99808a1a97004bef396a23c14b764ca4d`。[English](s29-spot-order-interval-inputs-spec.md)。

当前单票：[提供 Binance 精确账户时段订单额度输入](https://github.com/kaiqiangh/tradex/issues/127)，已由kaiqiangh领取，ready-for-agent/wayfinder:task；原生父项121、已完成依赖126、0 open blockers。[已发布规范](https://github.com/kaiqiangh/tradex/issues/121#issuecomment-6083562531)。

来源实现 VERIFIED，163个冻结输入与提交相同；最终回归/普通构建/Gateway pin/实际UI及独立串行Standards→Spec通过。精确远端/tracker交付待完成。[验收证据](s29-spot-order-interval-inputs-evidence_zh.md)。既有单票/公开接缝不变；完整父项/全图/原生/提供方/金融/物理/原型/main验收独立。

## 问题

Trade已有有界精确账户开放订单库存，却没有经认证的账户级未成交订单时段用量。开放库存、成功下单header及本地请求权重回答不同问题。用户需要实际时段定义和原始账户计数，供后续普通Market/Limit请求的拥有方判定；观察计数或旧采集不能被当作可用执行额度。

## 方案

对已有不可变普通Binance Live BTC/USDT或ETH/USDT Proposal，显式采集实际所需订单频率定义和经认证精确账户时段计数，并在当前/捕获/Arm前说明原始count/limit/interval单位、覆盖、读取窗口、来源、计龄、失败恢复及未解决执行时间资格。开放订单过滤器、时段用量、HTTP预算和本地预留始终独立。完整只读输入切片位于拥有方动态资格/即时认证preflight之前，不下单、探测、预留、Arm或审批。

## 用户故事

1. 作为用户，我希望时段输入绑定不可变Proposal与已保存普通账户，以避免其他账户/环境替代我的观察。
2. 作为用户，我希望后端导出UID/key归属和实际读取用途，以防renderer制造身份/计数/权限。
3. 作为用户，我希望读取实际ORDERS时段定义，以免记忆限制或其他类别成为当前额度。
4. 作为用户，我希望每个实际声明时段都有有界观察，以免缺失时段被当作无限额。
5. 作为用户，我希望说明跨keys/IPs/APIs的账户范围，以免换key/workspace重置用量。
6. 作为用户，我希望原始精确整数以字符串保留，以免大计数/限额在浏览器丢失精度。
7. 作为用户，我希望区分真实零用量与缺失用量，以免不完整数据显示零订单。
8. 作为用户，我希望达到/超过limit时仍保留原始count/limit，以免截断提供方事实或资格化剩余额度。
9. 作为用户，我希望区分开放库存与时段未成交计数，以免互相替代。
10. 作为用户，我希望只按官方契约说明成交/撤单影响，以免本地事件制造递减奖励或重置。
11. 作为用户，我希望未知时段/type/活动字段及限额矛盾保持未解决，以免猜测时间表产生资格。
12. 作为用户，我希望读取窗口/原始clock sample与counter快照时间独立，以免收据或clock冒充counter水位。
13. 作为用户，我希望过期及可能UTC边界跨越保守撤下当前观察，以免缓存冒充新时段计数。
14. 作为用户，我希望显式刷新与脱敏auth/time/quota恢复，以免失败泄漏key或保留旧成功。
15. 作为用户，我希望account/key/source/rule/market/policy/Proposal/workspace/session/time变化及晚到结果被拒绝，以免旧计数绑定新意图。
16. 作为用户，我希望已保存审核只保留获准有界观察/摘要，无刷新/续收据，以保持原评估。
17. 作为用户，我希望键盘及1280/768/390说明，以保留范围、时间不确定性和独立门禁可读性。
18. 作为复审者，我希望真实公开RED/GREEN、准确字节检查/构建/UI与串行Standards再Spec，以免隐藏金融快照seed或无关成功标签关闭前置项。

## 实现决策

- 复用不可变Proposal上下文、普通Binance已有连接、可信时钟/运行绑定、signed/vault/zeroization边界、P3调度及当前/捕获审核模块。可信本地get和显式CAS refresh只接收workspace、已保存Proposal、当前state version；无新account/source设置、renderer URL/symbol/UID/interval/count/limit/time/authority输入或后台扫描。
- 后端导出canonical BTC/ETH/USDT、精确已保存account/remote UID/credential version及workspace/source/rule-material/market/policy/Proposal/hash/session/clock generations。复用trusted main归属/发布检查，integration stdio仅为隔离构建接缝。本地get不调用vault或provider。
- 用固定普通公共exchangeInfo、按导出canonical symbol过滤读取实际rateLimits，不扫描标的全集。实际ORDERS定义确定所需账户时段覆盖；REQUEST_WEIGHT/RAW_REQUESTS属于传输约束，不是订单额度。定义缺失/未知保持明确，不推断无限额；无真实定义用途时不做多余私有读取。
- 固定普通公开clock，再signed USER_DATA account身份和rateLimit/order GET。原始数字Spot UID须匹配已保存精确账户；不为此读取余额/订单/列表/历史。当前拟定成本：公开clock1、filtered exchangeInfo20仅共享IP；认证account20、rateLimit/order40使用真实signed UID/IP。实施时复核官方契约。复用HMAC/recvWindow5000、固定host/no-redirect，公共读取不虚构UID。
- 对照声明与观察的原始时段tuple/limit。已知单位SECOND/MINUTE/HOUR/DAY，intervalNum为原始正整数，limit/count为原始非负整数，均限signed64-bit、以最长20位精确十进制字符串保留；duration乘法溢出检查，不用float。原始零limit不当作禁用，达到/超过limit仍保留count。重复字段/tuple、原始float/string伪整数、负数/溢出、格式/大小/矛盾失败或明确不完整，不丢行伪造完整。拟定512KiB/响应、32时段行、8临时Proposal slots。未知type/unit/活动字段为有界未解决证据。
- Counter是账户级时段用量，不是开放订单过滤器库存或symbol额度。本地成交/撤单/过期、header缺失、workspace/key/source变化不能递减或重置观察。遵守实际共享418/429 Retry-After及UID/IP预算，不混淆HTTP读取预算与下单时段限制；无order test、PLACE/CANCEL/amend/listen-key mutation或生命周期探测。
- 类型化观察展示定义/私有读取摘要和窗口、精确tuple/count/limit、覆盖/未知原因、最早收据、原始clock samples。未提供的汇总counter providerObservedAt保持缺失，各读取非原子。不制造provider resetAt、counter snapshot time、剩余slots或执行资格化时段身份。从clock/定义导出的时段关联须明确为derived且保留不确定性/可能边界保守撤下，不是原始counter时间或准入承诺。
- 总deadline30s，已有有界vault worker与12秒完成余量；CP锁不跨vault/network。刷新开始即撤下旧成功；失败/过期清空当前观察，仅保留脱敏解释。绑定/归属sequence变化、晚到、重开、clock/session reset不能发布/复活旧证据。缓存get/历史不续最早收据；计龄取30s与配置stale阈值较严者，可能时段rollover可更早撤下。
- 输入资格始终UNAVAILABLE及明确not-execution-qualified原因。后续拥有方须结合普通意图实际过滤器增量、实际所需时段准入、本地预留、即时认证preflight及全部金融义务。本票不提升完整规则、rights/quote/liquidity、permissions/health、funding/fees/FX、Arm、同意、Prepare/Gateway；精确CANCEL/对账独立。
- 当前Trade和捕获RiskDecision/history/Arm前说明单位、不同预算概念、原始与derived时间、覆盖/不确定性/计龄/失败恢复及资格不可用。只持久化获准有界聚合/摘要，不保存原始account/definition/counter响应、secret或运行cache；捕获无刷新/轮询控制器。同步中英文金融、IPC、交互和证据契约。

## 测试决策

沿用已批准最高公开接缝，仅外部producer为fake。公开get/refresh真实RED先于GREEN，再一次推进一个外部行为；通过公共操作建立account/source/Proposal并观察current/history，无内部parser测试或正向CP金融/额度snapshot setter。

核验准确host/method/UID/key/signature/成本、无写入或多余库存读取。覆盖多时段、原始零/大整数/达到或超过limit、错误原始类型、溢出/重复/上界、缺失/多余/未知tuple、声明/计数矛盾、实际共享账户用量及418/429不重置。验证库存/counter不互代、不制造本地递减/重置或窗口/快照时间，过期/缓存/重开/绑定/晚到/deadline撤下、无CP锁跨I/O及脱敏恢复；所有正向观察的source资格仍不可用。

用现有隔离外部provider host重放实际current/captured/Arm前、键盘1280/768/390、pending/错误/恢复及捕获不变。当前完整及受影响source/rule/capacity/Hot/Gateway、生成IPC/配对manifest/追踪、普通desktop/Gateway pin及独立串行Standards再Spec；绑定准确source/evidence/remote/tracker。原生/托管/金融/物理/原型证据单列。

## 范围外

拥有方动态过滤器/准入资格、本地预留分配/释放、隐式counter reset/decrement、PRICE_RANGE/完整金额意图/费用/所需执行FX、rights/permission grant、即时认证Gateway preflight、Live mutation/private lifecycle/真实交易、Testnet/Bitget/S28范围变化、物理睡眠豁免、原型修复和main合并。完整goal义务继续由后续拥有方完成，不豁免release门禁。

## 补充说明

2026-10-09复核：[Spot filters](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/filters.md)、[账户时段counter REST契约](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md#query-unfilled-order-count)、[未成交计数语义](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/faqs/order_count_decrement.md)。接口区分开放库存与账户时段用量，返回所有时段counter，signed读取weight40。保守时间不资格化属于本规范设计，不是provider快照保证。

规划时容量输入已完成，使区别可见；当时的代码仍未采集时段额度。本单票真实来源依赖可独立开始，不伪造正向本地预留/金融路径。它推进完整父项的综合拥有方判定，不替代终点；不制造S28或物理生命周期调度依赖。
