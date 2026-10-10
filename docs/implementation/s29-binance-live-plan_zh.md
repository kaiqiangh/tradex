# S29 — Binance Spot Live 串行计划

父规范：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。当前票：[精确 Spot 规则来源与账户准入](https://github.com/kaiqiangh/tradex/issues/123)。诊断 #122 已 CLOSED；规则来源 #123 为 IN_PROGRESS。起点：dev@fe20d0aae1c0eea9df43c1e24f32199a92ff30b9。

用户于 2026-10-07 选择先做 S29，S28 验收仍开放；仅变更调度。S17、物理 S27 及 S28 全部金融门禁保持此前记录的未验证状态。最终目标仍是完整 RevC 应用，在 dev 完整验证后提交 dev→main PR，由本人 review，禁止自动合并。

父规范包含完整需求与实现决策。沿用已确认的公开 UI → 真实类型化 Control Plane → 临时 SQLite/outbox → 外部假 vault/HTTP/WS 边界。不得用隐藏权限种子构造新的金融正向路径；后续 PLACE 验收必须经过真实认证子 Gateway 与 loopback 假提供方。既有底层合成生命周期回归夹具仍明确为合成，不计作 S29 验收证据。

每次执行一张完整实现票：

1. **账户交易状态观察与门禁消费**（#122）：公开连接/探测/刷新 → 固定普通 Binance HTTP/签名/当前任务适配 → 类型化持久化状态 → Accounts 投影及 Arm/PLACE 健康门禁。无开放技术阻塞。真实金融验收不属于该子票，仍由父项要求。
2. **配置交易所/规则/行情证据**：精确 canonical BASE/QUOTE、状态/权限组/过滤器、真实 bid/ask 与深度、时间戳/序列/时效、来源代际与明确数据权利。#123 精确规则来源/准入已完成；报价/深度/权利归下一所属来源切片，先规范后实现；不得以 ticker/REST 接收时间替代当前报价，不虚构 crypto 股票日历。
3. **Live PLACE 与私有流生命周期**：真实账户能力、临发预检及实际需要的执行级 FX、不可变金融意图/审批/容量、认证 Gateway、单次变更、精确 client/order 身份、未知先查询、精确撤单/成交/费用及恢复。只依赖实际消费的证据门禁。
4. **普通原生与授权外部验收**：先安全复用已有密钥只读；真实金融操作须另外批准具体交易/撤单计划并由本人执行。不使用 Testnet/Demo 外部验收替代，不绕过地区访问限制，不自动合并 main。

父项完成须满足完整规范的每项用户故事以及 provider/金融/UX/架构门禁；关闭只读前置票不能关闭 S29。

2026-10-07 核对的官方接口参考：[账户交易状态与 API 权限](https://developers.binance.com/en/docs/catalog/core-trading-wallet/api/rest-api/account)、[系统状态](https://developers.binance.com/en/docs/catalog/core-trading-wallet/api/rest-api/others)、[Spot 规则](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/general)、[Spot 流](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)。后续端点核查归所属切片；这些参考不证明用户的数据许可、地区访问或真实就绪。

已完成串行票为 [#123 精确 Spot 规则来源与账户准入](https://github.com/kaiqiangh/tradex/issues/123)，状态 CLOSED；诊断 #122 已 CLOSED。#123 只完成规则来源纵向切片，报价/深度/权利与完整金融生命周期继续由后续所属工作承担，父项 #121 保持 OPEN。

S29.2 [收口](https://github.com/kaiqiangh/tradex/issues/123#issuecomment-6052549325)：源码 `b726bb38df9280cb290d8efedc72cc2f2dbbff95`，证据交付 `21a665919d540caf730e5b6c1b2a35d72af826b0`；当前完整检查、普通构建、真实 Rust-backed UI 和独立串行复审通过。父项 #121 保持 OPEN。

已完成单来源票：[提供 Binance Spot Hot 报价与连续深度来源](https://github.com/kaiqiangh/tradex/issues/124#issuecomment-6069229563)已 CLOSED。[配对验收](s29-binance-hot-source-evidence_zh.md)：运行时d9abe24、复审4898850、交付dc51bd7已核对 origin/dev；九项来源标准、当前检查、独立串行复审通过。父项保持 OPEN；下一步先细化逐 Proposal 精确规则/参考/动态输入及实际需要的费用/FX，再完成所属审批/Prepare/Gateway/私有生命周期验收；不并行实施或跳过权威门禁。

当前下一张单票：[核验 Proposal 静态 Spot 规则与所需参考输入](https://github.com/kaiqiangh/tradex/issues/125)，[规范](s29-spot-proposal-rules-spec_zh.md)。已领取/ready-for-agent，S29原生子票，记录已关闭规则源与Hot来源依赖；无开放阻塞。实现/复审起点2368e22，仅交付规划，未实施/验收。


已完成单票：[提供 Binance 精确账户时段订单额度输入](https://github.com/kaiqiangh/tradex/issues/127)；[配对规范](s29-spot-order-interval-inputs-spec_zh.md)、[已发布规范](https://github.com/kaiqiangh/tradex/issues/121#issuecomment-6083562531)。容量输入126已CLOSED，不能由开放库存推断时段用量。原生completed dependency126、0 open blocker、基线91e2f2a；源码5d7c597的163冻结输入、最终回归/构建/pin/UI与串行复审通过，来源实现VERIFIED；[配对验收](s29-spot-order-interval-inputs-evidence_zh.md)，交付22646b3及8AC关闭已回读。完整拥有方动态资格、PRICE_RANGE/费用/所需FX、即时preflight/私有生命周期与真实验收继续必需，无authority捷径或并行实施。

已完成单票：[解释 Proposal 价格区间执行范围](https://github.com/kaiqiangh/tradex/issues/128)；[配对规范](s29-price-range-explanation-spec_zh.md)、[已发布规范](https://github.com/kaiqiangh/tradex/issues/121#issuecomment-6087692513)、[配对验收](s29-price-range-explanation-evidence_zh.md)。ready-for-agent、S29原生子票、原生completed dependency125、0 open blocker、实现/复审基线 ec10e99（规划提交不改源码）。源码622e34c的160冻结输入与已提交字节一致；最终487Rust/0失败/39既有ignored跨34二进制、17Node、schema/前端构建/追溯、普通实际React当前/捕获/pre-arm与独立串行Standards→Spec复审均通过；交付7f4a301及7项AC关闭已回读，远端/跟踪器交付完成。读取该视图本身不新增提供方请求；提供方显式 null 是「已验证不实施」的事实而非失败查找；任何可用预览都保持执行未获资格。费用/所需FX、拥有方动态资格、即时 preflight/私有生命周期与真实金融验收仍属后续拥有方工作。

已完成单票：[以精确容量与时段额度履行 Spot Live PLACE 拥有方资格](https://github.com/kaiqiangh/tradex/issues/129)；[配对规范](s29-owning-place-qualification-spec_zh.md)、[已发布规范](https://github.com/kaiqiangh/tradex/issues/121#issuecomment-6088873723)、[配对验收](evidence/s29-owning-place-qualification/README.md)。ready-for-agent、S29原生子票、原生completed dependency126/127/128、0 open blocker、实现/复审基线 dd1d5c9。已交付的容量与时段额度输入此前只是无人据以判定的只读证据；拥有方 Live PLACE 门禁现已消费它们，且被复核的判定携带该资格。源码375e1c0已VERIFIED；全量门禁 EXIT=0（492Rust/0失败/39既有ignored跨34二进制、17Node、schema/前端构建/追溯），并在仅渲染层的「声明出处」改动后于全新重启的服务器上重新跑绿实际 React 当前/捕获/pre-arm；独立串行 Standards→Spec 复审记录于 [code-review-final.md](evidence/s29-owning-place-qualification/code-review-final.md)；交付7246062及11项AC关闭已回读。两个公开接缝各自携带真实 RED/GREEN 对；浏览器那一对发现了一个真实契约缺陷——对 71 字符的 `sha256:` 值声明了 64 字符上限——`cargo test` 与 `schema:check` 都无法检出，因为 React 会先用生成的校验器解码每个响应。复审中更正了两处契约陈述以与已交付行为一致：绑定身份不再读作两个输入的 state version 必须互相相等，审批复核段落也不再声称摘要不覆盖 Spot 证据。费用/所需FX、即时认证 preflight/私有生命周期与真实金融验收仍属后续拥有方工作。

已交付来源/拒绝片单票 [S29.9 #131](https://github.com/kaiqiangh/tradex/issues/131)，规范门禁 [#130](https://github.com/kaiqiangh/tradex/issues/130)：[配对规范](s29-fee-and-required-fx-spec_zh.md)、[设计](s29-fee-and-required-fx-design.md)，固定基线 `dev@1f8e231`。原生层级121→130→131，completed dependency129，0开放阻塞。规范130与实现131已CLOSED。源码7b99f8e、契约00d4f14及核对的源码/证据交付2c7dad1绑定最终检查、普通构建/pin、冻结实际UI及独立串行Standards→Spec PASS；[配对证据](s29-fee-and-required-fx-evidence_zh.md)记录关闭回读。本零新增读取的来源陈述在当前已交付输入下不可用：全部允许意图需要不支持的 `USDT → base`，账户层费率无法建立完整symbol/side费用或计入币种。maker/taker比较不是完整费额估计。普通主机已有symbol佣金API；真正费用/计入币种证据与真正所需USDT执行FX两项都必须由后续拥有方完成，才可进入完整金融资格。不接受提供方能力缺失或完整goal永久拒绝的结论。父121及map1保持OPEN；原生、物理、真实金融、原型与main验收保持独立。
