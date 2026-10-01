# S28 生产金融前置依赖

日期：2026-10-01。审计基线：`aa43091ea2eef732b5865b163cbeee3cac3e03c9`。
状态：**IMPLEMENTATION_IN_PROGRESS；日历切片已批准**。
规范：[补齐 S28 生产金融前置证据](https://github.com/kaiqiangh/tradex/issues/117)。
英文配对：[Production financial prerequisites](s28-financial-prerequisites.md)。

用户明确授权在串行补齐金融前置依赖期间保持[接入已配置 Alpaca 报价与 Hot 订阅](https://github.com/kaiqiangh/tradex/issues/116) OPEN。本决定只调整执行顺序，不免除当前票正向审批/Prepare/dispatch 验收；[Trading 212 Live 父级验收](https://github.com/kaiqiangh/tradex/issues/113)同样尚未完成。

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

[Alpaca 当前公司行为契约](https://docs.alpaca.markets/us/reference/corporateactions-1)警告 provider/处理延迟；日期过滤按 `process_date`，默认 `complete` 也可能包含已经处理但字段不完整的事件。空响应不能证明没有新公告。后续事件/可交易实现须保留这些限制，研究精确 broker 契约后才能声称正向门禁通过。

FX 首先确定实际必需路径，不预设新建通用服务。ECB 日参考数据不能升级为执行级 Verified；若现有 provider 没有所需观察契约，必须另作来源/授权决策。

拟议顺序：日历 → 公司行为/精确 broker tradability → 必需执行 FX → 回到已有正向金融验收。这是**串行工作顺序**，不伪造技术依赖链；后续不确定来源继续明确记录并研究。

## 证据边界

本次仅公开文档读取和本地源码核对，没有读取 vault、刷新真实账户、Arm、审批、reservation、provider mutation 或金融执行；不宣称 runtime PASS。既有 S17 与真实 S27 延后项保持 OPEN。本次审计不构成完整 map 验收或 dev→main PR 条件。

实现续项：已批准的日历子票 [#118](https://github.com/kaiqiangh/tradex/issues/118) 正在实施。生产者、界面与原生验证单独记录在[日历证据](s28-calendar-evidence_zh.md)；上面的仅审计边界描述初始调研，不描述后续实现。未批准金融权限契约变更。
