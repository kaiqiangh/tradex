# S29.11 — 完整 Spot 拥有方金融计划

父项：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。已发布规范：[完成 Spot 逐资产金融计划与真实必需 FX](https://github.com/kaiqiangh/tradex/issues/134)，OPEN / ready-for-agent / 已认领，父项native子项、已完成依赖133/131。规划基线：`dev@70ce33ec5aa0286eff82e924f281c4588e03b3be`。[英文权威](s29-owning-financial-plan-spec.md)。

**状态：完整单一拥有方实现规范已具体化；实现验收全部待完成。** 这是下一张单一拥有方工作项，保留[金融输入研究交接](s29-financial-inputs-research.md)的全部20项需求，不以另一个始终未资格化的来源投影代替目标。金额模型/授权与BNB估算决策已在下文具体化；条件来源选择/用途/质量及有限用途准确路由profile已在下文具体化。单独跟踪的#149实现是前置能力切片，不是拆分或替代完整的#135拥有方金融计划；全部15项验收条件及完整producer到Prepare正向路径仍由#135承接。真实账户/来源激活证据仍未证明。本文件不记录实现、来源选择、provider使用权授予、金融操作或需求状态升级。**原型代码未改**。

## 问题

用户可以读取精确标的佣金声明，却仍不能审阅不可变 Spot 意图的完整金融计划。拥有方语句仍消费历史账户比较费率，没有金额费用，也不能资格化必要的 USDT 估值；本金容量与结算目前各追踪一种资产，缺少一致的费用、资金、工作区本位币、条件 BNB 风险、逐资产原子预留及精确结算路径。

当前估算、授权额度、券商强制边界和实际成交费用是不同事实。SELL 限价约束最低价，不约束最高收入或费用；本地 maximum_spend 不是额外币安参数；现货报价不证明币安内部 BNB 平均费用换算价。解决这些差异属于本票，不豁免完整目标，也不以永久拒绝代替实现。

## 方案

在可信 Control Plane 中从不变 Proposal 与真实当前生产者推导一个拥有方金融计划，包含完整三族费用/扣费分支、精确 BASE/QUOTE/BNB 资金、真正消费的用途限定换算、原生订单边界及独立审阅的金额授权。货币风控、不可变 review 与 Prepare 消费同一计划。支持且完整的输入必须经公开产品接缝真正正向资格化；不完整或矛盾输入产生类型化阻塞，不留下部分权限或预留。

Prepare 在一个事务内消费审批并预留完整原币资产向量；Gateway preflight 在发送前重验已批准金融材料。真实成交/费用逐资产恰好结算一次；pending 与 unknown 工作保留容量。计划本身不授予 Arm、行情使用权、账户权限、审批、派发或未来成交/费用保证。

## 用户故事

1. 精确保存账户与不可变 Proposal 绑定所有金融输入，防止借用其他账户资金或费率。
2. 纳入 standard/tax/special，防止仅比较 maker/taker 隐藏收费。
3. 每族正确组合角色与 buyer/seller，显示完整方向费率。
4. 普通限价涵盖 taker 与混合成交，不宣称 maker-only。
5. 原始折扣乘数仅作用于 standard，不擅自折扣 tax/special。
6. BUY 费用保留收到的 BASE，SELL 保留收到的 QUOTE。
7. 保留条件 BNB 与收到资产 fallback，不由 flags 保证未来扣费币种。
8. BNB 充足性覆盖三族总额，不只覆盖折扣后的 standard。
9. 用精确十进制计算原币金额，不让浮点改变金融权限。
10. 显示估算方法与舍入/换算不确定性，不宣称未来费用保证。
11. 分开本金边界、费用额度与实际收费，明确审批授权内容。
12. SELL 限价显示最低价，不把估算收入写成上限。
13. 比较真正提交的参数与授权，不把本地 maximum 当作券商强制参数。
14. 展示 BASE/QUOTE/BNB free/locked、覆盖与原始回执，不把缺失解释为零或可用资金。
15. 券商锁定与本地预留各只计算一次。
16. 已足额 USDT 仍按 USDT 资金处理，不制造换算或金融操作。
17. 资金缺口等待用户外部补足，行情不供应钱包资金。
18. 真正需要的条件 BNB 额度另有原币预留，防止跨 Thread 重复占用。
19. 收到资产手续费减少收入，不重复计作支出本金。
20. 仅检查该意图或启用政策真正消费的换算，不引入无关组合货币。
21. 保留可配置工作区本位币，不写死 USD 或假定 USDT=USD。
22. 显示来源、方向、bid/ask、规模、时间、质量、成本用途和脱锚证据，单一数字不足以通过 Live 风控。
23. 只用获准配置的数据来源，公开可读不授予使用权。
24. 不支持或被拒绝的路由明确显示，不自动回退、取倒数或桥接。
25. 支持且完整的真实生产者能够正向资格化，不交付另一个永久不可用页面。
26. 风控、review、Prepare 消费同一计划，数字与门禁保持一致。
27. 费率、资金、政策、来源或意图的实质变化使不兼容审批失效。
28. 新鲜复验保留独立回执，不产生只因回执变化就无法完成的审批循环。
29. 原子拒绝全部资产，不留下部分消费的审批、attempt 或预留。
30. 真实费用逐资产恰好结算一次，unknown 容量继续冻结。
31. 当前/捕获/pre-arm、键盘与窄屏均保留完整披露及不可变历史。
32. 用真实公开生产者正向/失败、普通构建、串行独立审查和精确 dev 交付验收，不以模型 fixture 关闭金融能力。

## 固定实现决策

- 扩展既有 Proposal/风控/review/Prepare/Gateway 唯一拥有方。复用保存密钥及 HTTP/vault/WS 外部接缝；不接受 renderer 金融输入、权限 setter、正向权威快照注入、第二 Gateway 或第二结算拥有方。
- 消费精确 symbol 佣金声明及账户/来源/规则/Proposal 绑定。账户比较费率保留历史含义。保留12个原始字符串；缺失、重复、未知活跃扩展或矛盾不得资格化。
- 每个角色在各族加方向费率，再比较完整 maker/taker 分支总额；不把独立最大分量拼成虚假角色。BUY 基于模型收到 BASE，SELL 基于模型收到 QUOTE。普通 LIMIT 不等于 maker-only。
- 返回 discount 是 standard 支付乘数；tax/special 不折扣。保留 flags/asset/fallback。成交的实际费用资产直到券商结算后才确定。根据用户2026-10-15对S29.11作出的决定，只要所有独立的来源、钱包、行情、风险和权限门禁都通过，完整且绑定同一intent/账户的当前费用估算即可使review和Prepare具备资格。必须展示文档定义的BNB折扣与收到资产回退两种结果，披露`ACTUAL_FEE_ASSET_PENDING`，不得保证未来费用或资产。只预留当前选定分支实际会流出的额度：BNB分支预留原生BNB，收到资产回退分支不增加流出费用claim。Prepare和Gateway即时前置检查须重新验证绑定证据与额度，不得静默切换分支。结算以券商实际`commissionAsset`和扣款为准。未建模的币种/分支、费用缺失或扣款超过已保存额度时冻结容量并进入对账；不得截断或追认。当前分支不完整时仍不可用。BNB充足性覆盖三族换算总额。
- 用精确十进制及下文固定的向外有理数模型。LOT_SIZE 不能证明佣金舍入或成交次数；聚合 ceiling 不等于逐笔费用绝对上界。
- 区分当前费用估算、授权本金/费用额度、券商原生边界与真实费用。时间、精度、现价与快照均不保证未来费率不变；下文提交前授权检查保留RevC硬限，不伪造未来费用保证或任意安全系数。
- 保留用户真正的 order type、quantity/notional、TIF。固定 BASE 的 BUY LIMIT 可由 quantity×limit 约束 QUOTE 本金；SELL LIMIT 是最低价格与获准 BASE 数量，不是最高收入。MARKET quantity/quoteOrderQty 保留原生语义，不自动改订单形状来制造边界。
- 从认证生产者保留 proposal-scoped 精确 BASE/QUOTE/BNB free/locked 与原始回执/代次；必需行缺失不填零，重复失败。保留非原子性和缺少 snapshot timestamp，不把事务更新时间当作新鲜采集时间。规范钱包投影在不可变历史中保留完整receipt，但从审批material排除collection/读取receipt身份；准确金额、本地claims、binding和覆盖状态仍绑定审批。
- 复用原生本金容量。券商 free 已排除 locked；本地有效预留只扣一次。足额 USDT 的 BUY 是 USDT→USDT identity；SELL 使用提交 BASE。不得暗加 Convert、转账、提款、新券商或金融操作。
- fallback 费用减少收入，不另加一次支出本金；真正消费的第三资产费用额度有独立原币 claim。一次事务预留全部实际支出 claim，保留分支/模型净收入与支出差异。
- 既有 PLACE/outbox 事务扩展有界逐资产预留向量，以account+asset+intent为范围，兼容历史主预留。在预留旁以结算元数据保存不可变的已选费用处置分支、收到资产回退额度和净收款；这些值不占用流出钱包容量。须校验它与审批绑定的当前预览一致。审批消费、attempt及全部流出claim全部提交或全部不提交；本金与条件费用并发共享容量谓词。
- 根据真实累计成交/费用、实际资产和execution identity结算，回放去重，只应用delta一次。只接受已保存分支和原币额度内的费用；资产/分支不符或扣款超额时冻结全部剩余容量并进入对账。pending CANCEL/UNKNOWN_RECONCILING保留未解决claim；只有终态券商事实释放未用容量。费用真值缺失/不完整时不填零、不早释、不恢复healthy。
- 仅派生真正消费的本金政策估值、原生资金 identity、fallback 费用估值、条件扣费换算/估值；只有启用政策真实消费时才带入组合路由。保留配置的 fiat 本位币，必需缺失阻塞，无关资产不新增门禁。
- Rust/schema/TypeScript 协同放宽资产/币对。选定直接路由或另行明确批准的逐腿路径保留原始单位、方向、bid/ask/size、provider/first receipt、代次、质量/脱锚、成本方法及每腿。不得平价、隐含倒数/桥接、拒绝后回退或未知成本填零。
- 政策估值与真实资金换算不同。仅作原币估值的路由不是兑换交易或保证报价；identity 无需外部行情。仅实际会发生的兑换操作需要其真实成本，而本票不新增此类操作。
- 来源 profile 必须明确已有 provider/dataset，并分别保留许可用途、provider entitlement 与技术访问事实；草案不选择来源。Coinbase 的200不代替缺失书面许可；Alpaca 候选不等于已获使用权或新鲜度。用户选择或用途陈述不是 provider entitlement。既有Alpaca `USER_ATTESTED_LOCAL_ONLY`确认仍仅限本地`POLICY_VALUATION`及其范围受限的`FX_DATA_USE_RIGHTS=VERIFIED`能力。按已批准的#149决议，独立的Binance `USER_ATTESTED_LOCAL_ONLY`确认仅能支持用户本人本地、非商业的`LOCAL_TRADING_DECISION_SUPPORT`用途，包括本地owning-plan/risk review与Prepare；它只能通过所选Binance来源的`MARKET_DATA_USE`检查。该确认不改变provider entitlement或`BinanceSpotQuoteEvidence.dataUseRights=UNVERIFIED`，也不会单独创建approval、Arm、PLACE或执行权限。已知限制及过期/未解决的策略事实仍会阻塞。
- 支持且合法来源的真实外部响应必须使拥有方金融资格真实正向可达，并喂给同一货币风控/Prepare。账户、行情、使用权、政策、Arm、审批/preflight 门禁独立，来源成功不授予这些权限。
- 分开金融材料与采集来源。账户/意图/政策/来源及费率/金额/条件实质变化使不兼容审批失效；新复验有独立不可变回执，不改旧捕获。金融材料匹配仍须遵守原审批到期，缓存/history 不延寿。
- Prepare 与立即 Gateway preflight 检查准确身份、代次/时间/质量、真正授权边界、资金及 claims；变化/缺失在 provider mutation 前原子拒绝。后续私有生命周期及真实金融验收仍必需。
- 草案成为实现决策时同步英文 PRD/UI/Backend 与中文。历史来源切片/捕获保留当时语义；草案、模型或来源观察不升级 AC-035 等全局状态。

## 已明确金额模型与授权决策 — 尚未实现

根据权威PRD§44及既有UI§14.2关于TradeX授权/预留与券商强制的区分，解决此前金额/BNB设计问题。这些是模型决策，不证明来源使用权、不资格化当前观察、不改变规范wire或放松当前Live门禁。来源切片历史保留当时含义；实现暴露新金融资格前必须同步配对规范。

1. **支持意图。** 保留已有边界：BASE MARKET双向且沿用DAY语义；提供方元数据允许时的BUY-only QUOTE MARKET；BASE LIMIT GTC/IOC/FOK。不把QUOTE SELL等不支持形态暗中改成另一订单。BASE LIMIT按不可变limit建模；BASE MARKET按真实当前best ask/bid及既有显示规模/slippage门禁；BUY QUOTE MARKET以原生QUOTE金额建模，模型收到BASE是QUOTE/ask的有理数。这是全量成交情景，不是精确BASE成交预测；SELL限价情景为最低价收入，不是最高费用/收入包络。
2. **模型类别。** `CURRENT_ORDER_SCENARIO`，方法版本`SPOT_THREE_FAMILY_RATIONAL_CEIL18_V1`。按记录的数量/价格/角色/费率/换算假设精确计算，再向外舍入；不模拟交易所舍入，也不是未来手续费绝对上界。保留`VENUE_PER_FILL_ROUNDING_UNPROVEN`；实际消费BNB价格时另保留`BNB_INTERNAL_AVERAGE_EQUIVALENCE_UNPROVEN`。这些是已披露模型局限，不是宣称有未来保证的计算所缺的输入；本票不提供保证收费计算。真正必需模型输入缺失仍阻塞。
3. **有理数计算及输出。** 完整逐族表达式保留精确整数分子/分母，不提前截断BASE数量、倒数价或换算。每族最终原币支出向上舍入18小数位，再精确相加。收到资产扣费须从精确gross收入表达式减去完整已向上舍入模型费用，再将net收入向下舍入18位；显示净额与显示费用一致，不能另用未舍入费用。保留完整wire/显示值，标准化输出必须符合既有64字符金额上限，否则类型化不可用。零分母、负数/非法必需值及运算/资源越界均fail-closed。既有18位向下截断除法不能当作向上有理数除法，也不能在丢失余数后ceil来恢复。18位是明确TradeX模型尺度，不推断为原生佣金精度。
4. **每扣费分支独立选完整角色。** 每个允许角色分别计算并向外舍入三族的角色+方向费用；fallback选其完整舍入模型总额最大者，BNB另选其折扣后三族完整总额最大者。因为仅standard加权，选中的角色可能不同。保留每分支角色/原始族输入，不拼接各族最大值、不沿用fallback角色、不重复加方向。用精确十进制比较完整模型总额；相等以TAKER作显示tie-break，不宣称真实taker成交。保留舍入前表达式审计，不假定其胜者仍为逐族舍入后的胜者。普通LIMIT保留双角色；MARKET仅在固定原生MARKET契约下可用TAKER。混合LIMIT以最大完整角色情景表达，逐笔舍入仍明确未证明。
5. **BNB现价模型。** 选定profile必须明确收到资产→BNB估值路径。获准且共同报价货币的两个产品，模型比率是收到资产ask/BNB bid：取显示bid/ask矩形中的高成本比率，不宣称约束交易所内部平均价。记录每来源/方向/腿/规模与金额适用性/单位/时间。standard乘原始返回值，tax/special不折扣。不把avgPrice/现价称为精确内部价，不创造资金操作，也不因已有数字就启用倒数/桥接。准确可执行provider/path仍属下面的开放来源工作。
6. **条件资金选择。** BNB free减有效本地BNB claims，与完整当前模型BNB额度精确比较；不由flags推断持仓。折扣活跃、有效BNB余额已证明为零且折扣后费用表达式严格为正，则当前fallback无需BNB价格；不活跃BNB金额保持缺失，注明`ZERO_BNB_CURRENT_FALLBACK`依据，不填零换算。完整折扣表达式精确为零时记录代数事实，不推断支付资产或要求无用行情。其他正余额必须有有效换算才能判断充足性；缺失时不宣布充足或fallback。模型BNB不足选完整收到资产收费，足额选BNB claim。两分支都保留当前条件，不保证未来实际扣费币种。模型fallback超过收入时等待真实额外资金语义，不制造负收入或隐含额外支出。
7. **审批及本地硬限。** 金融审批通过既有不可变review digest冻结模型/必需路由/原币本金额度/实际支出BNB额度/费用值/预算单位/条件分支；renderer不能供应可信金额。BUY MARKET既有不可变QUOTE `maximumSpend`约束当前模型全部实际支出：本金及通过明确批准的QUOTE估值路径计入的支出费用。收到BASE的fallback费用减少BASE收入，不再加一次QUOTE支出。扣除已批准模型支出费用值，派生剩余QUOTE本金额度；当前本金/总额超任一额度即拒绝。BUY QUOTE MARKET原生本金claim为固定quoteOrderQty，不是更大的本地总成本上限。BUY BASE MARKET预留本地本金额度，并明确币安收到quantity，没有maximum-spend参数。BUY LIMIT的quantity×limit是原生本金边界；review另冻结模型BNB额度及总QUOTE值。若明确给本地maximum必须覆盖当前总额；未另给maximum则用户审阅派生本金边界及独立披露模型费用额度。既有显式审批动作之前不签发额度。
8. **SELL及政策单位。** BASE SELL批准并供给BASE quantity；QUOTE费用减少收入，真正消费BNB另有原币claim。保留既有MARKET maximum-value检查，明确其为TradeX提交前审阅交易价值上限；它不约束未来收入、不产生QUOTE现金支出、不成为券商价格参数。BNB额度同一审批单独可见，BASE资金claim不与QUOTE ceiling混比。订单/日交易notional消费gross交易价值；reserved-capital消费本金/费用支出向量一次，收到资产费用/收入不重复形成支出。其他启用持仓/风险/损失政策保留真实证据需求；当前价值门禁不保证未来更高SELL收入或未来费用。
9. **preflight与金融材料。** Prepare/Gateway重新计算当前必需原币/价值，与不变批准额度比较，并保留账户/来源/政策/时钟/行情有效性门禁。超本地硬限拒绝发送；缺输入则不可用。真实费率、报价、深度、条件、金额、来源/代次/有效性变化使不兼容审批失效。仅数值相同的decimal格式及采集回执/digest变化可从金融材料比较排除，原始审计来源独立保留；ID/symbol、政策/来源/许可版本和未知活跃义务绝不归一化成相等。材料匹配不延长审批/观察寿命、不重写旧捕获；不引入报价/深度变化容差。
10. **超模型的真实结算。** 券商事实超出已批准模型额度时记录全部原币支出及未覆盖差额，不截断或追认审批。通过既有路径degrade/disarm并对账该账户，退休当前金融观察。精确累计execution/fee identity驱动恰好一次结算。pending/unknown或费用不完整继续持有未解决claims；终态仅释未用授权容量，不制造负remaining、额外释放或未知费用零值。新权威账户/资金证据前不得恢复ready，不复用成交前过时资金。这是实际真值恢复，不保证未来成交cap，也不允许已知当前超额派发。
11. **预留证明范围。** 普通preview/Prepare考虑所有其他有效原币claims；已经Prepare的attempt复验必须由可信持久化attempt/Gateway上下文识别其自身已持有向量，比较所需金额与该向量及当前券商free，其他本地hold扣一次；不把自身claim加到券商free或再扣自身hold。不接受renderer传入ignore-claim/attempt列表。汇总locked或ACK本身不证明哪个本地claim已反映；确认重叠须有准确当前provider/account/order/asset证据。未知重叠继续持有并披露容量不确定性；只有保守下界时，可证明下界内足额，超下界不能误报为精确券商资金缺口。读取反映证据不释放/结算预留；资金负值或未证明重叠不制造新增容量。

S29.9早期“估算永不低于未来交易所收费”不是RevC、精度或只读声明已建立的定理。本票选定模型保留PRD§44估算费用及**提交前**硬支出检查，不暗改成未来精确收费保证；只有原生参数真正强制的更强边界才披露为券商保证。不以source-only验收宣称已实现。

## 来源与路由契约 — 开发决策，真实激活仍待验证

[第一方来源研究](s29-fx-source-contract-research.md)补齐适配器与路由设计。2026-10-10用户确认：**仅本人本地非商业使用，Alpaca原始行情不分发、不发送给云端模型**。记录为人类提供的Alpaca用途上下文。Alpaca独立Settings动作仍来源绑定且仅限`POLICY_VALUATION`，具范围受限的`FX_DATA_USE_RIGHTS=VERIFIED`；这不是provider entitlement或provider核实的许可，也不会关闭#135。2026-10-19用户批准[#149](https://github.com/kaiqiangh/tradex/issues/149)：独立Binance Settings动作可记录用户本人本地、非商业的`LOCAL_TRADING_DECISION_SUPPORT`用途，包括本地review与Prepare。该动作要求声明法域，并显式确认当前所引用的一手条款。其provenance为`USER_ATTESTED_LOCAL_ONLY`；它只能通过所选来源的`MARKET_DATA_USE` risk check，而provider entitlement及行情`dataUseRights`仍为`UNVERIFIED`。它不满足其他source/fee/FX/account/funding/approval/Arm/dispatch门禁。适用限制、准确所选数据集访问及当前观察仍属真实激活/验收事实；已知禁止用途或过期/未解决的source-policy状态仍会拒绝。

1. **显式选择及有限用途。** 普通保存来源配置扩展已有Alpaca凭据引用、显式crypto location（仅`us/us-1/eu-1`）、普通data host及准确用途专属API路由；初始未选择，本规范不替用户选定、不暗换地区/feed。个人本地profile的Alpaca观察仅供Rust `POLICY_VALUATION`；既有普通Binance Hot供标的价格，独立allowlist的内部`BNBUSDT`连续簿供`FEE_ESTIMATE`及BNB政策估值。发现来源不增加BNB可交易目录、不授权交易、不发金融mutation。不支持location/来源/路由形态类型化拒绝。
2. **使用依据、访问及质量独立。** 版本化source-policy catalog保留第一方条款/文档URL、原始hash、使用约束及判断适用所需account/product/dataset事实。Control Plane依据这些文档与实际用途派生评估；配置只收证据/上下文，不能收`verified/allowed`金融权威flag。人类/提供方UI协议证据保留声明来源，不伪装API事实。未识别/矛盾协议或缺少必需适用事实为`SOURCE_USE_UNRESOLVED`，已知限制为`SOURCE_USE_DENIED`。适用普通个人条款不要求虚构专用金融估值许可；HTTP200、ACTIVE或Basic/IEX不证明全部数据权利。准确认证dataset成功单独建立技术访问；拒绝/凭据失效/profile、协议、policy或来源代次改变均退休观察。必须实现真实证据收取及policy求值，不用构造器正向或永久不可用适配器代替。适用证据未解决前真实激活仍不可用；条件测试不授予本用户使用权。
3. **真实生产者及原值。** 认证Alpaca crypto latest请求选定location的准确`USDT/USD`，保留`bp/bs/ap/as`原始数字token及RFC3339 `t`；不用浮点、不用匿名成功改认证契约。其他独立forex消费者准确请求其方向性provider pair并保留原始`bp/mp/ap/t`。原数字token直接保存在既有`bid/mid/ask`字段；每条rate保留provider pair及timestamp，响应本地receipt保存在`FxRateEvidence.observedAt`。既有crypto quote详情还保留可用size、request path和response digest。归一化十进制仅供内部校验和计算；UI显示provider原token。缺币对不可用，不填零/推断倒数。认证仅用保存vault凭据。凭据/profile代次、配额/拒绝及首次可读body receipt为不可变来源；getter不更新时间。沿用有界deadline/配额/late fence；仅在原时钟有效期内缓存。
4. **估值路由及成本。** 对拥有方Spot `INTENT_POLICY`路由，直接使用选定API结果的原币对及计价币种。`USDT/USD`结果仍是USD：不得换算成EUR或其他工作区本位币，不加隐藏FX腿，不推断倒数、不回退历史、不将USDT按平价当作USD。原结果及provider精度/时间仍可见。只有结果币种与配置的工作区本位币完全相同时，货币RiskDecision/Prepare消费者才可使用该值，并且仍须通过所有独立门禁。不匹配时返回类型化`VALUATION_CURRENCY_MISMATCH` / `SPOT_VALUATION_CURRENCY_MISMATCH`阻塞，不产生owning金额、审批authority或reservation。此issue专属规则不改写独立的portfolio/account用途路由。另行规定的BNB费用模型仍按用途处理：逐条费用估值负债用ask、资产/收入用bid；BNB→USDT负债用内部Binance ask；收到BASE→BNB明确按标的USDT ask除以BNBUSDT bid，收到USDT→BNB除以BNBUSDT bid。这些表达式不证明交易所内部平均价，也不启用通用路由。外部负债按指定向外有理数舍入，流入净值按floor规则。价格quote仅为**政策估值**，执行深度及换汇交易成本为`NOT_APPLICABLE_TO_POLICY_VALUATION`，不伪造流动性/零交易成本，也不能授权资金转换或实际转账。必要的交易quote滑点/深度和费用簿覆盖仍是独立门禁。
5. **质量及金额范围。** 必需可信clock、真实provider time≤now、first receipt≥request、非负elapsed、正且不交叉bid/ask、同来源代次。每消费腿在review/Prepare/即时preflight的最大年龄为来源30秒与当前stale-quote policy最小值（默认3秒）；不把延迟/reference改实时。固定质量规则`USDT_USD_ONE_PERCENT_V1`：bid及ask均须含边界落在`[0.99,1.01] USD/USDT`；超界披露脱锚并阻塞依赖Live估值。这是明确保守来源质量决策，不是假定平价或替用户设金额偏好，版本/阈值进入金融材料及review。陈旧/12秒研究样本不通过。保留Alpaca size及文档解释，但价格估值不由单位解释宣称可执行规模。Binance费用模型要求真实best-level BASE数量：实际消费的标的ask覆盖完整收到资产换算额，BNB bid覆盖完整模型BNB费用，BNB ask覆盖实际估值BNB负债；不足阻塞，不造VWAP/无限深度/安全系数。完整费用总额消费size一次，不逐族重复声明足额。
6. **费用专用连续簿。** 复用普通Binance depth生命周期，仅`BNBUSDT`；准确当前metadata证明BASE=BNB/QUOTE=USDT及有效状态。REST前buffer diff、以`lastUpdateId`及`U/u`接续、绝对量更新/零量删除。REST无market time不能单独资格化；必需连续事件`E`、first receipt、簿/session代次及连续sequence。gap/reset/reconnect、陈旧/未来/非法/交叉、metadata/profile改变或断开均退休，真实重建后恢复。仅普通host、文档weight/ping/pong/reconnect及有界buffer；新增symbol仅内部费用，不可建订单。
7. **本地消费及单一拥有方。** 选定Alpaca policy-valuation与Binance local-decision-support profile的原始观察（含共享缓存）在model/tool序列化、research/export artifact或外部consumer前强制本地范围；本地UI可查原值，原始quote/rate/body不进入prompt/export。测试检查真正外发边界，不只查标签。来源材料供同一fee/funds/risk/review/Prepare，原值另存不可变审计。profile/source/policy/terms/intent、价格/深度、分支/金额仍绑定，仅沿用数值相同格式/receipt-only明确例外。新鲜相同证据不延长审批、不更新历史，也不豁免独立source-use、交易permission、Arm、审批/对账或真实/原生门禁。只有获批的Binance `MARKET_DATA_USE`检查提供本地确认路径；其他source-use要求保持独立。

**就绪区分：** 上述为完整拥有方实现决策，不是实际来源选择或资格化。#135单一拥有方实现票必须保留全部15项AC及20项交接，包含在受支持来源用途依据下producer→owning→risk/Prepare正向以及本用户仍未证明真实激活的拒绝路径。顺序执行的#149能力票只能实现获批的Binance确认前置能力，不能关闭或替代任何#135条件。不得以source-only结案或放松其他要求。

对于owning Spot Prepare，`ExecutionReservation.workspaceNotional`必须使用刚复验的Proposal范围`INTENT_POLICY`路线，与其`spotFeeFx`陈述使用同一份估值证据。保留本次选择的完整reservation notional（包括适用的独立本地授权上限），不得直接复制较小的expected-spend值。按准确BUY ask／SELL bid计算，校验workspace与结果币种、意图身份、requirements和route-material版本；缺失或变化时拒绝。该路径不得使用历史portfolio FX、平价、倒数或额外换汇腿。原生本金／费用claims及所有独立权限门禁分别继续校验。

已消费PLACE在派发前复验时，Control Plane只可在临时风险评估中复用准确attempt已保存的ACTIVE reservation。须对照耐久记录核对reservation ID、attempt、账户、Proposal/hash、标的、side和当前状态版本。只从竞争性本地claims、钱包／费用证据及容量汇总中排除这一条自身记录，所有其他reservation继续计入。持久化reservation仍保持ACTIVE，原始各项claim继续持有。新review、审批签发和Prepare仍统计全部既有reservation。缺失、已释放、错绑或借用其他意图的reservation必须拒绝派发；该评估不释放、结算、创建新同意或执行provider mutation。

## 测试决策

沿用已批准最高接缝：实际 React→类型化 Rust Control Plane→隔离 SQLite/outbox；仅 HTTP/vault/WS 外部生产者 fake。无需重问接缝或粒度；来源生命周期、结算及原子 Prepare 公开测试作为先例。

每个行为逐一公开 RED→GREEN。正向场景必须用正常命令建立账户/来源/Draft/Proposal，消费完整实际提供方格式，达到新拥有方正向语句并证明货币风控/Prepare 一致消费。构造器测试、注入金融行、人工 VERIFIED 权限或始终未资格化路径都不算正向验收。

覆盖双向/完整角色族/混合角色、乘数0/1、tax/special、禁用折扣、BNB充足/不足/fallback、free/locked/local claims、缺失/重复钱包、不支持来源/资产、规模/深度/时间/脱锚/成本、UID/symbol/代次/政策/意图矛盾、认证拒绝、配额/deadline/late、回执与材料变化区分、到期/reopen。检查原始单位，不用浮点/平价/暗中资金操作。

公开检查原子拒绝/并发：消费前拒绝则审批保持未消费，无部分claims/attempt/outbox/provider mutation。检查逐资产成交/费用 delta、重复、终态释放、pending撤单/unknown恢复。实际当前/捕获/pre-arm、键盘1280/768/390、不可变历史、无溢出/console错误必须通过。最终 schema/全量及受影响测试、普通desktop build/Gateway pin、独立串行Standards→Spec及精确源码/远端/tracker交付；真实交易、原生、物理及发布门禁独立。

## 验收条件 — 全部待完成

独立金额oracle准备：固定官方FAQ虚构SELL示例的special分量相加为`0.06000000`，乘法行却印`0.06000030`；BNB示例显示精度也不同，不能直接复制其最终数字作为精确oracle。按原始分量，模型收到`17491.25 USDT`时standard/tax/special的舍入前金额为`0.01049475`/`0.040229875`/`1049.475`，fallback合计`1049.525724625 USDT`。在示例`0.25`乘数、假定`260 USDT/BNB`下，先前ceil8的`4.03660715 BNB`仅保留初稿观察，不是选定政策。选定最终逐族ceil18给出standard`0.000010091105769231`、tax`0.000154730288461539`、special`4.036442307692307693`、合计`4.036607129086538463 BNB`，用decimal精度100独立核对。这些为合成规划oracle，不是已实现公开测试、当前行情、原生BNB精度证明或实际收费保证；实现前仍须真正公开TDD。

额外字面模型案例用独立精确Fraction及适用的高精度Decimal核对，供后续公开生产者断言。费率/价格/数量全为虚构且明确声明；零是已知输入，不替代缺失字段。

| 案例 | 模型事实 | 字面结果与处理 |
| --- | --- | --- |
| 折扣改变角色胜者 | 收到100USDT；standard maker0.01/taker0.001，tax maker0/taker0.005，special/side为0；乘数0.1；BNB260USDT | fallback MAKER=1USDT，TAKER=0.6USDT；BNB TAKER=`0.001961538461538463`，MAKER=`0.000384615384615385`。完整分支独立选择。 |
| 舍入改变角色胜者 | 收到1；maker standard0.0000000000000000009/tax0；taker standard0.0000000000000000004/tax0.0000000000000000004；special/side0 | 舍入前MAKER较大；逐族ceil18后MAKER=`0.000000000000000001`，TAKER=`0.000000000000000002`，模型选TAKER，不能沿用未舍入胜者。 |
| 极小正向换算 | 完整某族分子0.000000000000000001、正分母3 | upper18=`0.000000000000000001`，floor18为0；后续ceil不能恢复已丢弃正余数。 |
| 第三资产费用受本地硬限 | BUY本金500USDT；支出BNB额度2；假设获准BNB/USD ask50及明确USDT/USD bid0.99估值路径 | 费用QUOTE值ceil18=`101.010101010101010102`，总额=`601.010101010101010102 USDT`；maximum600拒绝，610留本金额度=`508.989898989898989898 USDT`。原币claims仍为USDT本金与BNB费用，不双扣USDT。 |
| 不重复扣locked | USDT free500/locked300，本地claims20 | 有效480USDT；本金450足额、490不足；不再次扣券商locked300。 |
| SELL更优成交非费用cap | BASE0.5、limit35000USDT/BASE、已知完整fallback rate0.06 | 最低价模型gross17500/fee1050USDT；后续40000成交gross20000/fee1200USDT；1050绝不写成最大可能费用。 |

这些仅建立模型决策/oracle准备，不取得产品资格、不验证真实FX/脱锚/使用权、不证明流动性，也不替代公开RED/GREEN。独立核对回读记为规划证据，单独保留旧初稿回读。

- [ ] AC1. 固定所选来源profile、provider-entitlement与技术访问证据、受支持的purpose-specific route contract及真实证据收取/policy求值，不含rights setter或自动回退。用户确认的本地用途策略必须保持独立；获批的Binance确认仅能通过所选来源的`MARKET_DATA_USE`检查，不能标记成provider entitlement。
- [ ] AC2. 精确symbol/account/Proposal驱动全部角色/方向/三族计算，无账户比较替代。
- [ ] AC3. 固定当前估算舍入、授权及原生边界，保留不确定性与不支持绝对保证。
- [ ] AC4. 条件BNB及收到资产分支保留实际消费的诚实当前估算、原币、净收入及三族足额；已证明零BNB的fallback不需无用换算，不伪造不活跃BNB金额。
- [ ] AC5. BASE/QUOTE/BNB持仓覆盖、first receipt、free/locked/local claims只消费一次并完整绑定/退休。
- [ ] AC6. 按已批准的条件费用契约，完整当前owning plan可在全部独立门禁通过时使review/Prepare具备资格；适用时，当前匹配的Binance本地用途确认仅可通过`MARKET_DATA_USE`。预先持有的原生本金及当前分支实际消费的费用claim在同一事务产生账户范围逐资产预留向量。Prepare还须持久化审批绑定的费用分支和回退额度供后续结算；收到资产扣费不作为流出容量claim。无隐藏资金操作。
- [ ] AC7. 逐资产成交/费用对照已保存分支与原币额度恰好结算一次；分支/币种不符、费用真值缺失或扣款超额时冻结并进入对账，保留pending/unknown，只按券商真值释放未用容量。
- [ ] AC8. Rust/schema/TypeScript支持真实资产/币对/路径/用途/时间/质量/成本，兼容历史捕获/预留。
- [ ] AC9. 必需估值/费用路由正向资格化并保留配置本位币/方向/逐腿，identity/不可用处理无平价。
- [ ] AC10. 支持的真实外部生产者响应在其受支持的来源用途依据下到达拥有方金融正向及同一风控/Prepare；适用时包括获批的Binance本地确认路径，且无植入权限捷径。
- [ ] AC11. 实质变化使不兼容审批失效；匹配复验保留原到期与独立来源，不制造回执审批循环。
- [ ] AC12. Prepare/preflight强制真正授权/资金边界，变化/缺失拒绝，无部分状态/provider mutation。
- [ ] AC13. 公开TDD、正向/错误/生命周期/并发/结算及实际当前/捕获/pre-arm响应式UI通过。
- [ ] AC14. 配对规范、完整检查、普通构建/pin、独立串行复审与冻结交付通过；本项不自动升级父项/全图/真实/原生/物理/发布验收。
- [ ] AC15. owning Spot `INTENT_POLICY`估值在当前、捕获和pre-Arm视图中保留精确提供方币对、原始数值文本、币种、提供方时间/本地回执时间及可用的size/status。本路由不增加欧元/其他币种换算，不使用历史FX，不推导倒数或平价。RiskDecision/Prepare仅消费结果币种与workspace本位币完全一致的估值；否则返回typed估值币种阻断，且不产生owning金额、审批权限或reservation。其他按purpose限定的FX路由及所有独立source-use门禁均保留；仅获批的Binance `MARKET_DATA_USE`检查可使用独立的本地确认。

## 不在本项范围

20项研究交接全部纳入，均仍待实现：1→AC2；2→AC2/4；3→AC3/4；4→AC3/12；5→AC3/4；6→AC5；7→AC5/6；8→AC6/7；9→AC9/15；10→AC6；11→AC8/9/15；12→AC1；13→AC3/9/15；14→AC10/15；15→AC11；16→AC12/15；17→AC13/15；18→AC13；19→AC14；20→AC14。完整性映射不是PASS证据。

新券商、Convert/转账/提款、托管/杠杆/衍生品、任意交易标的、自动金融操作/真实订单、购买许可、地区绕过或声称用户确认授予provider权利、私有流完整生命周期、无关S28/S17/物理S27验收、原型修复和main合并。完整S29生命周期/真实提供方及全应用仍为后续串行必需工作。

## 补充

真实已完成前置：[精确标的佣金声明](https://github.com/kaiqiangh/tradex/issues/133)、[拥有方费用/必需FX拒绝与捕获](https://github.com/kaiqiangh/tradex/issues/131)以及真正消费的现有本金/容量/行情/规则。不得为调度添加S28或物理阻塞边。按已明确的条件来源/路由契约只发布当前一个完整实现票，保留真实native依赖及已确认单票粒度。92544f5的旧规划回读是初稿三个问题的不可变当时记录，不是后续已明确模型决策的当前陈述。

主来源、日期及所有20项交接映射与[英文权威](s29-owning-financial-plan-spec.md)一致。本稿是规划证据，不是法律使用权结论或真实金融验收。
