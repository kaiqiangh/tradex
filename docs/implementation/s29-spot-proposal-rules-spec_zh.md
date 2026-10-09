# S29.4 — Proposal 静态 Spot 规则与所需参考输入

父项：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。[精确规则/准入](https://github.com/kaiqiangh/tradex/issues/123)与[Hot报价/连续深度](https://github.com/kaiqiangh/tradex/issues/124)已 CLOSED。规划起点 `dev@2368e22122b2028e92f4c3352c2f08fe80e1344a`。[English](s29-spot-proposal-rules-spec.md)。

当前单实现票：[核验 Proposal 静态 Spot 规则与所需参考输入](https://github.com/kaiqiangh/tradex/issues/125)，已由kaiqiangh领取并标记ready-for-agent。[已发布规范](https://github.com/kaiqiangh/tradex/issues/121#issuecomment-6069315992)。实现/复审基线 `2368e22122b2028e92f4c3352c2f08fe80e1344a`。只读实现、完整本地/Hot/Gateway 回归、普通构建/pin、重建 React→Rust UI 及串行 Standards→Spec 复审均 PASS。精确源码/远端/tracker 交付仍待记录，票保持 OPEN。见[验收记录](s29-spot-proposal-rules-evidence_zh.md)。

## 问题

Trade已有精确账户元数据与真实买卖盘，却不能解释不可变 Proposal 是否符合已收集的过滤器。支持的标的或可见报价不证明数量步长、限价、百分比/名义额输入或动态账户上限。用户需要逐规则结果与缺失义务，不能虚构执行就绪。

## 方案

单张只读纵向票：选中既有普通 Binance Live Proposal，显式刷新实际需要的参考输入，精确求值支持的静态规则，在 Trade 展示绑定的逐规则结果与未完成义务。违反规则拒绝；缺失、未知或不支持的适用输入保持不可用。推进父故事8–11、14、17、27；完整动态资格、费用/FX、已认证预检与金融验收仍须后续完成。

## 用户故事

1. 用户希望针对保存的不可变 Proposal 求值，避免金额/方向/类型/venue偏离意图。
2. 用户希望绑定保存账户与canonical BTC/USDT或ETH/USDT，避免借用别的账户规则。
3. 用户希望保留BASE/QUOTE及币种，避免互换或USDT=USD。
4. 用户希望校验订单形式，阻止不支持的TIF、数量模式或高级形式。
5. 用户希望精确求值适用价格/数量约束，不自动舍入Proposal。
6. 用户希望区分停用、缺失与畸形字段，避免零值关闭无关义务。
7. 用户希望显示方向和名义额适用性，BUY/SELL、Market/Limit使用实际规则。
8. 用户希望仅按需读取真实参考，不把midpoint/receipt冒充参考。
9. 用户希望参考缺失/错误失败关闭，不以回退绕过拒绝。
10. 用户希望区分原始区间/提供方时间/首次本地receipt，缓存不续期。
11. 用户希望过期/外来/改变代次输入退役，不保留旧权威。
12. 用户希望保留全部scope及未知约束，不因symbol成功丢弃账户/交易所义务。
13. 用户希望缺失动态计数、仓位、执行输入显示未完成，静态成功不是完整规则PASS。
14. 用户希望权利/报价/健康/FX/费用/权限独立，参考读取不能Arm或审批。
15. 用户希望键盘和窄屏能只读刷新/恢复，授权前可读阻断。
16. 用户希望历史风险/复核标记为capture，不显示成续期当前观察。
17. 审阅者希望公开外部协议证据、精确SHA检查和串行独立复审，不用注入夹具关闭票/父项。

## 实现决策

- 复用账户金融来源、运行时绑定、Hot来源代次、共享读取调度/预算、不可变Proposal与Trade风险/复核。没有新配置/密钥库、renderer主机/标的/价格/权威输入或全市场后台扫描。只读请求只给存储Proposal和期望状态；后端派生意图、账户、标的和所需输入。
- 有界类型化逐规则结果包含origin/scope/kind、适用/不适用、PASS/REJECT/UNAVAILABLE、原因和证据引用，并显示整体完整性与未完成义务。静态component PASS不同于INSTRUMENT_RULES。已知当前违反可拒绝；完整PASS要求所有实际适用义务已证明。本票不虚构动态证据，当前动态/未知义务继续阻止完整资格。
- 只支持BASE Market/DAY、提供方允许的BUY-only QUOTE Market、BASE Limit/GTC/IOC/FOK；不改意图，拒绝不支持组合/高级标志/隐式转换。precision不等于tick/step许可；按各规则解释字段，不能盲用元数据disabled标记。精确decimal比较/取余/乘法；无f64/舍入/Testnet写路径复用。
- 完成输入真实可用时，求值静态PRICE_FILTER、LOT_SIZE/MARKET_LOT_SIZE、适用MIN_NOTIONAL/NOTIONAL、PERCENT_PRICE/PERCENT_PRICE_BY_SIDE、单笔MAX_ASSET。各scope/origin保留，不静默覆盖/丢弃未知字段。QUOTE Market不虚构精确BASE量；实际结果未知时数量/资产义务仍未完成。最大支出局部守卫不证明费用/资金/审批。
- 参考由后端按用途派生；各规则实施前重核官方契约。真实非空提供方参考使用其文档角色；只有良好显式null响应才允许文档回退。所有错误（包括无参考错误）按既有保守策略保持不可用。适用平均值区间须精确相符；零区间最后成交须真实原始成交/提供方时间，无时间ticker、REST receipt或Hot midpoint不能合格。若没有有界可信路径，明确不可用，不猜数据。
- 规则参考不是政策最后成交偏离输入或买卖盘。独立绑定用途、canonical/venue/账户/来源/Proposal hash、workspace/session/time代次、材料及原始/首次receipt年龄。参考暂态；保存复核只留已允许的有界引用/hash和capture结果，不落原始tick/book。重开只恢复配置/历史，读取不续期；材料变化/退役按既有机制使绑定同意失效。
- 固定普通主机，复用有界HTTP/P3公共IP预算，不虚构公共UID消耗；严格原始类型/重复/大小/期限/redirect/冷却及净化错误。全局CP锁不覆盖I/O；状态冲突、取消、来源/工作区/时间变化拒绝迟到发布。公共参考不传执行凭据。
- 动态订单计数/仓位容量、执行PRICE_RANGE输入及未知适用约束在后续所属工作前明确UNAVAILABLE。不能因只提交普通订单把缺失动态义务记NOT_APPLICABLE；准确保护性CANCEL/对账独立。费用/FX与即时认证Gateway预检独立。
- MARKET_DATA_USE及用户数据权利仍不可用/未验证。读取/checkbox/确认不能产生许可/区域资格/整体金融PASS/Arm/审批/预留/写请求。不改Testnet范围、S28/S17/物理S27或main；实施时同步双语规范IPC/金融/UI契约。

## 测试决策

- 沿用已确认单一公开React→真实typed Rust CP→临时SQLite/outbox→外部fake vault/HTTP/WS边界。首个新外部行为真实RED→GREEN；账户/来源/Proposal用公开操作创建，不注入正向金融权威/参考快照。
- 实际外部HTTP响应和真实Hot报价；覆盖精确合法/非法price/step/notional/side/applicability、停用与畸形、BASE/QUOTE、MAX_ASSET单位、scope重复/未知、参考值/null/错误/区间/提供方时间、精度边界、来源/账户/Proposal/时间变化、首次receipt/过期/重开与迟到取消。静态组件成功仍可整体UNAVAILABLE，明确断言无整体审批/PLACE资格。
- 实际Trade/capture历史展示逐项成功/拒绝/缺口、显式刷新/恢复、键盘1280/768/390、当前与capture真实标签。没有真实金融/法律动作。规则源/Hot/Gateway相关回归、完整检查、生成IPC/双语清单与普通生产构建；固定基线Standards→Spec独立串行复审，原生/提供方/金融证据单列。

## 不在本票范围

动态账户/执行输入收集、费用/FX资格、即时认证Gateway预检、新PLACE/私有流、真实交易、数据权利授权、通用文本DLP、margin/衍生/高级订单、新资产/主机、购买许可与合并main。属于父项的后续工作不因本前置票免除。

## 补充

2026-10-08核对[官方Spot过滤器](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/filters.md)与[REST参考/平均值](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md)：区分价格字段停用、方向/Market适用性、参考值/null/错误及原始timestamp或平均区间/closeTime。实施不得推断未文档化语义或金融用途权利。只规划当前新票，不并行实施其他切片。
