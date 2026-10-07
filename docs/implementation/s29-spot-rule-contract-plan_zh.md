# S29.2 — 普通 Spot 规则来源契约计划

当前单票：[读取 Binance Live 精确 Spot 规则与账户准入证据](https://github.com/kaiqiangh/tradex/issues/123)，属于未缩减的完整 [S29 规范](https://github.com/kaiqiangh/tradex/issues/121)。前置诊断子票 #122 已 CLOSED；S29 保持 OPEN。开始审查基线：`4388d9c4ae460ee5f4d8a6a21b0ced5cfa3fe374`。[English](s29-spot-rule-contract-plan.md)。

完整父项用户故事仍是目标。当前完整纵向切片选择已保存的普通 Live 账户，保存标准 BTC/USDT 或 ETH/USDT 规则来源选择，读取有界的认证账户及公开精确标的证据，再通过 Settings、Markets 和 Trade 投影类型化约束及负向/准入检查。选择持久化，活跃证据仅在运行期有效并绑定代次。Save 不读取 Provider；disconnect 保留借用账户/密钥；重开只恢复元数据。复用既有金融来源选择/存储/绑定、外部 vault/HTTP 适配器、调度器及独立首个接收时间时效；不另建密钥库或让页面选择端点。

既有 exchangeInfo 解析/读取属于历史 Testnet 提交路径，不能证明普通 Live 规则。当前官方资料要求分别读取公开 exchangeInfo/executionRules 与签名 account/myFilters。精确账户 permission sets 必须组内 OR、组间 AND；公开目录成员资格不足以准入。[General API](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/general)、[Account API](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account)、[Filters](https://developers.binance.com/en/docs/products/spot/filters)，核对于 2026-10-07。

类型化证据保留标准身份、标的状态、Spot/订单形式标志、原始整数/布尔/decimal 类型、独立约束范围及不支持义务。证据缺失、未知活跃约束和未满足的参考/动态输入不能升级为完整规则资格。不经过浮点 decimal 往返或自动调整 Proposal 数量/价格。提供方时钟采样、真实提供的时间及首个接收时间保持独立，均不等于报价时间；不暴露原始消息/响应正文或秘密。

读取/准入证据与逐 Proposal 执行资格分开。当前禁止/不支持权限、维护/锁定、HALT/BREAK、错误身份/权限成员资格、过期/未来证据以及来源/账户/材料/代次变化，均使新订单资格失效。精确 CANCEL/对账保持独立。报价/深度/使用权、实际所需 FX/费用、真实规则解释及 Gateway 即时复核、Live PLACE/私有流和单独授权的外部验收仍属于 S29 后续必要工作。USDT 不等于 USD；不虚构 crypto 股票日历或公司行为完整性。

沿用已确认公开 React → 真实类型化 Control Plane → 临时 SQLite/outbox → 外部假 vault/HTTP/WS 边界。先记录真实公开来源选择 RED 再 GREEN，然后补齐采集、解析、故障/当前任务/时效/恢复、真实响应式键盘 UI 和适当既有回归；不新增正向金融快照/权威种子。同步英文/中文 IPC 与 UI 契约，完成当前票后才处理下一实现项。独立 Standards → Spec 串行审查绑定本基线及实际交付；本切片不触发 main 操作或免除 S28/S17/物理 S27。
