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
