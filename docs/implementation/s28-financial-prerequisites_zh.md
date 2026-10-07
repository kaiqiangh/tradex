# S28 生产金融前置依赖

日期：2026-10-01。审计基线：`aa43091ea2eef732b5865b163cbeee3cac3e03c9`。
状态：**READ_ONLY_CHILDREN_DELIVERED；正向金融验收仍不支持/未验证**。
规范：[补齐 S28 生产金融前置证据](https://github.com/kaiqiangh/tradex/issues/117)。
英文配对：[Production financial prerequisites](s28-financial-prerequisites.md)。

用户明确授权在串行补齐金融前置依赖期间保持[接入已配置 Alpaca 报价与 Hot 订阅](https://github.com/kaiqiangh/tradex/issues/116) OPEN。本决定只调整执行顺序，不免除当前票正向审批/Prepare/dispatch 验收；[Trading 212 Live 父级验收](https://github.com/kaiqiangh/tradex/issues/113)同样尚未完成。

## 当前返回验收审计 — 2026-10-07

初始返回审计：`dev@64b21a3df0ae56a4410604902ff14b636f6837aa`，于6a81d2e交付；恢复跟进基线：`dev@6a81d2ed63f9fba5f774b2abd8900cca776d7b67`。日历、已知公司行为/精确账户元数据、所需路由/只读 FX 三张实现子票均已带独立审查证据关闭；本前置规范及报价/Trading212 正向验收父项尚未完成。

| 当前门禁 | 权威证据 / 结果 | 后续所需证据 |
|---|---|---|
| 日历 | 日历子票118已交付实际原生读取/过期证据；本会话显式刷新并观察到经认证的当前XNAS日历，消费时仍需校验实际年龄/绑定。 | 当前绑定且可信的覆盖/时段，不能用历史 receipt。 |
| 已知行为 / 精确账户目录 | 子票119已交付有界读取和过期；当前已知事件查询完成，采样时保留观察已不可用。账户恢复后精确元数据成为AVAILABLE，随后保持原始首次receipt2026-10-07T09:21:18.724Z而转为UNAVAILABLE，原因是过期/绑定失效。完整行为/当前停牌/可交易性/调整仍独立。 | 完整前瞻行为/调整与执行账户当前不受限可交易性/权威身份，不能用遍历完查询或目录成员资格代替。 |
| 所需 FX | 子票120已交付实际余额/意图路由、原始精确数字/时间及冻结同意；2026-10-07账户恢复后的原生FX重读返回脱敏端点拒绝原因，没有汇率或首次 receipt；HTTP403仅由映射推断。 | 实际需要时须有合格金融用途/授权、保守方向/舍入、精确券商资金/费用及完整金额输入；同币及保护性 CANCEL 保留操作级语义。 |
| 报价来源 | 当前普通原生显式选择 IEX：经认证报价读取验证技术可达；Hot认证和订阅已确认，采样时尚未观察到实际当前报价。 | 实际新鲜且合格的报价/覆盖/深度及真正需要的 SIP 授权；确认消息和 IEX 连通性不足。 |
| Trading212账户 / 权限 | 既有Live记录原为FAILED/STALE、UNVERIFIED且DISARMED；显式已有密钥只读Refresh已恢复实际读取，保留同一scope的既有确认后，本地确认恢复CONNECTED/UNVERIFIED/DISARMED；没有新权限被验证。 | 实际只读账户恢复已观察；完整权威权限仍独立缺失。 |
| 已签发审批 / Prepare / dispatch | 仍未验证；规范权限守卫拒绝非VERIFIED；普通风险对完整公司行为及精确可交易/权威身份输入报告不可用，所需未合格FX仍不可用。 | 所有适用生产前置条件先满足，再验证实际签发同意/Prepare/Gateway；fixture或禁用的只读预览不能替代。 |

2026-10-07重新取得完整Trading212 OpenAPI：22操作/17路径，SHA256仍为 `a272f70a713fa9f2f906e9b4be12136c9d5d63d7f9f48562e939aa35c4fc041d`；公布的契约中没有完整key/IP scope、当前停牌/可交易性或FX报价/换汇操作。这是公开契约范围，不声称不存在未公开接口。[官方API](https://docs.trading212.com/api)仍限制主账户币种执行。[Alpaca公司行为](https://docs.alpaca.markets/us/reference/corporateactions-1)仍提醒上游/处理延迟，使用处理日期而非前瞻完整性。[Alpaca行情计划](https://docs.alpaca.markets/us/docs/about-market-data-api)区分IEX与美国全交易所覆盖。

当前20项runtime输入、已签名主程序/Gateway字节及实际父子关系与已验证普通构建一致；本次源码未变，不声称重新构建或运行测试。当前checkout没有CI工作流，不声称上游CI通过。[返回证据](evidence/s28-return/)仅归档安全来源状态、schema操作元数据和构建/进程布尔值，不含密钥、账户标识、余额、持仓、报价价格或提供方原始正文。此前Keychain等待为历史：当前账户及精确账户元数据读取已完成，最新原生采样无SecItemCopyMatching等待；早期审计冻结于6a81d2e，[恢复证据](evidence/s28-return/recovery-audit.md)更新其账户待完成状态。没有执行Arm、金融审批、Prepare、提供方写入或main操作；S17及实际OS睡眠唤醒跳过项仍未验证。以下清单和带日期段落保留最初审计及历史检查点语义。

## 依赖清单

| 前置项 | 审计基线时的生产证据 | 所需结果 / 边界 |
|---|---|---|
| 完整 API 权限范围 | Trading 212 adapter 只检测三项读取能力，scope 为 `UNVERIFIED`。 | RevC 要求 Live Arm 使用 `VERIFIED`；当前公开接口无法查询完整 key/IP scope。保留外部能力缺口，不制造权限。 |
| 可信时间 | 已有进程级 TimeService，金融门禁消费其结果。 | 重开/时间不连续后重新校验；时间不可信时阻止新观察。真实 OS 睡眠唤醒验收仍按用户安排跳过。 |
| XNAS 交易日历 | S08 有失败关闭投影和隔离 fixture；普通构建无生产 producer。 | 明确选择已有 Alpaca 凭据、固定 host、认证 v3 XNAS/UTC 数据、有界覆盖、首次接收时间、内容版本与短时风险/审批/派发消费。 |
| 公司行为 / 调整 | 普通构建为 unavailable/unknown。 | 受限 canonical 事件来源，保留处理延迟和完整性限制。事件元数据不证明历史已调整或没有待公告事件。 |
| Trading 212 精确标的状态 | 已有 AAPL/MSFT canonical mapping；没有当前执行可交易状态 producer。 | 证据必须匹配执行 provider/account、provider symbol、canonical instrument 和 venue。Alpaca asset flag 或普通 metadata 成员资格不能证明 Trading 212 可交易。 |
| 必需 FX | 现有估值可使用 ECB 日参考数据，但不是执行级报价。 | 从实际 intent/policy 币种确定路径；需要可信质量、执行新鲜度和双时间戳。同币精确计算不凭空引入转换。 |
| 合格生产报价 | 已交付选定 feed producer 和 Hot；观察到真实原生 IEX 可达。 | SIP entitlement/coverage、depth、新鲜度及来源绑定分别验证；IEX 连通性不能代替合格的合并覆盖。当前报价票保持 OPEN。 |
| 正向金融链路 | 既有 Gateway/金融 fixture 不能证明这些生产依赖。 | 真正满足依赖后返回报价/Trading 212 验收；真实资金另需具体交易同意。禁止隐藏 authority seeding。 |

## Trading 212 权限核对

2026-10-01 获取了[官方 API 文档](https://docs.trading212.com/api)与[完整公开 OpenAPI](https://docs.trading212.com/_bundle/api.json?download=)。SHA-256：

`a272f70a713fa9f2f906e9b4be12136c9d5d63d7f9f48562e939aa35c4fc041d`

完整 bundle 包含 **17 个 path、22 项 HTTP operation**，没有完整 key 权限或 IP scope 查询。两种认证方案是 Basic key/secret 和旧 API-key header，并非权限 introspection。这是对当前完整公开契约的结论，不声称不存在未公开的 provider 接口。

summary/position/order GET 成功只证明这些请求当时具备读取能力，不能证明 place/cancel 权限、没有危险权限或完整 scope。生产 adapter 因此正确保留 `UNVERIFIED`。RevC UI Spec Account Permissions 与 Backend §41.2 要求 Arm 使用 `VERIFIED`；确认未知权限审核不会改变 scope。

- 换 key、刷新账户或人工审核截图均不能在当前契约下建立 `VERIFIED`。
- 禁止订单/撤单探测、人工 attestation endpoint、合成状态更新或自动 Arm。
- 可实现的数据依赖能独立推进。Trading 212 正向金融验收继续阻塞，直到取得权威 provider 能力，或用户通过既定流程明确批准产品契约修订。本次没有批准此类修订。

本地代码证据：Trading 212 只构建 `account.read`、`positions.read`、`orders.read`；Control Plane health blocker 拒绝任何非 `VERIFIED` scope；公开 wire 拒绝调用方权限声明。

## 首个可实现切片：日历

S06 已选来源当前提供[按 MIC 查询的 v3 接口](https://docs.alpaca.markets/us/reference/calendar-2)。配套 `.md` OpenAPI 明确区分 Paper/Live server，响应要求 `market`、`calendar`，日数据要求 `date`、`core_start`、`core_end`，允许可选 pre/post/lunch 时段，支持 `timezone=UTC` 与 `XNAS`。

首票明确选择合格的已有 Alpaca Paper 凭据及固定 Paper host，仅 GET 日历。此能力与报价 feed entitlement、broker execution 独立；不自动尝试其他 host/key；借用的账户 key 保留账户所有权。

producer 校验 identity、时段完整性/顺序及覆盖，保留首次接收时间，不将计划开市时间或接收时间冒充缺失的 provider observation time。执行证据最多存活 30 个可信秒；凭据、来源、会话、工作区变化及读取失败均撤销资格。OPEN/CLOSED/EXTENDED_HOURS 来源于认证 UTC 边界，不使用本地 weekday/DST 推算或注入 snapshot。

OD-005 能力必须分别提供 typed 状态和 UI：日历可用不表示公司行为、停牌、历史调整或 Live Ready。当前市场时段 risk consumer 只检查 source/status/version/time confidence，没有显式 receipt-age 检查；首票必须将短时 producer 与失败关闭的新鲜度消费一并实现，读取投影不能延长接收时间。

## 后续决策

2026-10-03：[公司行为 / 精确 broker 元数据中英文研究](s28-actions-tradability-research_zh.md)重新核对完整官方契约。[第二项前置规范](https://github.com/kaiqiangh/tradex/issues/117#issuecomment-5966487911)已发布，单张[只读实现票](https://github.com/kaiqiangh/tradex/issues/119)已认领。日期精度、分页完成与未来完整性、10分钟 broker 元数据质量、精确账户绑定及上市/实际 broker 执行 venue 分别保留。这是仍 OPEN 的实现工作，不是 hosted 验证，也不放宽父级完整金融要求。必需 FX 尚未开始。

[Alpaca 当前公司行为契约](https://docs.alpaca.markets/us/reference/corporateactions-1)警告 provider/处理延迟；日期过滤按 `process_date`，默认 `complete` 也可能包含已经处理但字段不完整的事件。空响应不能证明没有新公告。后续事件/可交易实现须保留这些限制，研究精确 broker 契约后才能声称正向门禁通过。

FX 首先确定实际必需路径，不预设新建通用服务。ECB 日参考数据不能升级为执行级 Verified；若现有 provider 没有所需观察契约，必须另作来源/授权决策。

拟议顺序：日历 → 公司行为/精确 broker tradability → 必需执行 FX → 回到已有正向金融验收。这是**串行工作顺序**，不伪造技术依赖链；后续不确定来源继续明确记录并研究。

## 证据边界

本次仅公开文档读取和本地源码核对，没有读取 vault、刷新真实账户、Arm、审批、reservation、provider mutation 或金融执行；不宣称 runtime PASS。既有 S17 与真实 S27 延后项保持 OPEN。本次审计不构成完整 map 验收或 dev→main PR 条件。

实现续项：已批准的日历子票 [#118](https://github.com/kaiqiangh/tradex/issues/118) 的有限生产者、界面与真实原生日历验证及最终 Standards/Spec 审查已通过；验证单独记录在[日历证据](s28-calendar-evidence_zh.md)；上面的仅审计边界描述初始调研，不描述后续实现。未批准金融权限契约变更。

2026-10-03 实现检查点：公司行为/精确账户元数据已通过自动检查、真实 Rust UI、使用已保存 Alpaca Paper / Trading 212 Live 凭据的普通原生真实读取。自然过期保留原始时间/材料版本；完整覆盖、复权、权威身份、当前可交易性/停牌及权限仍不受支持或未验证。独立审查与最终 dev 交付待完成；子票119和全部验收父级保持 OPEN。见[双语实现证据](s28-actions-evidence_zh.md)。必需 FX 尚未开始。
