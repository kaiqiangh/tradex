# S28 后续——已配置 Alpaca 生产报价

状态：**IMPLEMENTATION_IN_PROGRESS**，2026-10-01。起点：`dev@dc4c350397bbe500983b47fdfd5d1c3af2f20455`。[English](s28-alpaca-production-quotes.md)。父级：[验证 Trading 212 Live 可信执行](https://github.com/kaiqiangh/tradex/issues/113)。此前本地 Market 审批子票已关闭，父级验收仍 OPEN。

## 问题

普通应用无法消费已选定的 OD-001：需要凭据的 probe 始终返回 BLOCKED_EXTERNAL，数据源配置无法保存，即使 source entry 为 AVAILABLE，普通 market detail 也没有报价；Market Explorer 的 Hot detail 没有生产订阅。已完成的合成审批链不能证明生产报价就绪。Alpaca Paper 券商连接不证明行情 entitlement 或覆盖。

## 方案

以单票贯通数据源配置至报价：显式配置独立范围的 Alpaca 行情访问，由提供方验证所选 feed，Census/Warm 按需获取、Hot 维持活动订阅，发布权威精确报价及溯源，并让既有行情和审批/风险界面消费同一观察。数据访问不授予金融权限；S28 其他前置条件分别保留。

## 用户故事

1. 作为用户，我希望 Settings 解释 OD-001 提供方，从而知道报价由哪个 feed 提供。
2. 作为用户，我希望显式选择已有 Alpaca 凭据引用，从而复用自己的已保存 key 而不展示或复制密钥。
3. 作为用户，我希望通过原生安全流程新增专用行情 key，从而不必为数据源创建交易账户。
4. 作为用户，我希望配置限于当前 workspace 且带版本，从而避免其他 workspace 或旧页面更改数据源。
5. 作为用户，我希望显式选择 IEX、SIP 或 delayed SIP，从而避免提供方默认 feed 静默改变覆盖。
6. 作为用户，我希望通过带认证的只读请求验证所选 feed，从而不以账户连接或勾选项捏造 entitlement。
7. 作为用户，我希望 SIP 拒绝时给出原因且不自动回退 IEX，从而不把有限 feed 展示为完整覆盖。
8. 作为用户，我希望延迟与单场所行情显式标注，从而不能批准其不支持的 Live 目标。
9. 作为用户，我希望保留来源、feed、覆盖、symbol、venue 及精确提供方/接收时间，从而能检查风险使用的报价。
10. 作为用户，我希望价格为精确小数且 BASE 深度单位有文档依据，从而避免浮点或 round-lot 假设改变审批。
11. 作为用户，我希望买卖报价场所与上市/执行目标分别保留，从而不把综合报价误称为单场所订单簿。
12. 作为用户，我希望拒绝非法、交叉、过期、未来或条件不支持的报价，从而避免坏数据成为可信证据。
13. 作为用户，我希望 Census/Warm 仅按需刷新，从而不订阅整个市场。
14. 作为用户，我希望 Hot 订阅当前查看的标的，从而在 Market Explorer 查看时获得实际更新。
15. 作为用户，我希望导航、断开和关闭 workspace 释放订阅，从而不跨界面泄露凭据或资源。
16. 作为用户，我希望认证/网络失败及配额冷却显示 degraded/stale，从而不让旧价格保持可执行。
17. 作为用户，我希望来源/feed/凭据变化及重连使旧报价同意失效，从而不能用旧审批授权新证据。
18. 作为用户，我希望重开保留来源选择但重新验证，从而不因保存配置恢复 Live 权限。
19. 作为用户，我希望网络工作有界且不持有 Control Plane 锁，从而不阻塞对账或可信命令。
20. 作为用户，我希望 Settings、Market Explorer 和审批理由在390/768/1280px 可读且支持键盘，从而能连接、检查和修复来源。
21. 作为用户，我希望 quote ticks 存在内存而不新增持久 tick archive，从而遵循 MVP 保留政策。
22. 作为用户，我希望 IPC、持久化和日志不含原始密钥或提供方错误正文，从而避免配置泄密。
23. 作为用户，我希望断开数据源保留复用的券商 key/账户，从而不误删其他连接。
24. 作为用户，我希望真实提供方读取与假提供方测试分开，从而不以 fixture 成功推断订阅已启用。
25. 作为用户，我希望权限/日历/可交易性/公司行为/FX 门禁持续执行，从而不因行情可用就 Arm 或发真实订单。

## 实现决策

- 复用既有安全 vault、数据源目录、版本化 command/job、provider 调度、market projection、TimeService、审批 digest 和界面。新增持久 source configuration 拥有 generation/version，仅保存元数据和不透明凭据引用；密钥保留在原生 vault。显式复用不重复拥有账户的 vault item。
- 增加 typed configure/disconnect 生命周期及脱敏连接/验证投影。Configure 显式选择凭据引用或调用原生专用凭据流程；普通 renderer 和 Agent 不能提交 market observation、entitlement attestation 或任意 URL。
- 仅固定 Alpaca 数据 HTTPS/WSS endpoint 与准确支持路径，禁用 redirect，显式 feed 与 canonical symbol mapping，限制 body/frame/timeout，仅只读 GET 和订阅流量。此 adapter 不涉及券商 POST/DELETE 或 privileged Gateway。
- 提供方确认显式请求的 feed 访问，仅证明该 feed 的技术可用性；新鲜度、覆盖、许可/保留限制和金融 readiness 分开。IEX 是单场所实时；delayed SIP 仍延迟；SIP 权限失败不自动切换 feed。不自动购买订阅、接受协议或扩大权限。
- Quote feed/coverage、bid/ask exchange 与上市/执行 venue 分开。在实现前同步双语权威 coverage-aware eligibility 契约；不把 IEX 改名为 XNAS，也不把综合 NBBO 称为单场所订单簿。未知覆盖/venue mapping 对执行保持 unavailable。
- 不经浮点转换解析小数 token。基于当前权威证据确认准确 HTTP/stream endpoint 的 size 单位；通用 stream 页面仍描述 round lots，但更晚的官方 CTA/UTP changelog 明确自2025-11-03起 quote size 为股数。当前 SIP CTA/UTP 报价直接保留精确股数为 BASE 深度，不乘100。这是 feed/date 范围明确的契约，不能外推至 IEX 或历史数据；其他支持路径的 BASE 深度须有已验证适用单位/转换，未解决时明确 unavailable，不捏造深度。价格仍可诚实展示；没有已验证深度不能声称金融正例验收通过。
- 校验 provider symbol、所请求 feed、支持的 quote conditions、时间精度、source/config generation 与标的身份。每次读取及审批阶段通过当前 TimeService 计算年龄；价格/深度/溯源/source generation 的变化通过共用 material 使同意失效，不放宽 S28 精确滑点比较。
- Census/Warm 按需请求，每个已配置 workspace/source 仅一个活动 Hot stream，遵守配额和连接限制；仅订阅请求的已支持 symbol。导航/重连通过 generation 防止旧帧替换其他标的或当前凭据的观察；重连有界，背压/冷却使观察变为非 current。
- SQLite 保存配置/审计元数据，报价及 Hot buffer 为有界内存。重开清除实时验证/新鲜度，不恢复审批或 ARMED；断开清除本数据源观察/订阅，保留不相关账户/vault ownership。
- 保留现有风险/review digest 对规范化 provider/receipt 时间的绑定。读取已接受的报价返回同一观察及原接收时间，重复同一提供方事件不刷新其新鲜度。新 provider 时间、quote material 或 source/connection generation 都是新证据，令旧同意失效；snapshot identity 包含这些权威维度，view collection time 不冒充 provider event。
- WebSocket upgrade/connected 不代表认证访问。Hot current 须先有 authenticated acknowledgement，再有请求 symbol 的 subscription acknowledgement。HTTP status 与 stream protocol error 分开：stream402/404 是认证失败/超时，405为 symbol limit，406为 connection limit，407为 slow client，409为 insufficient subscription；stream403为 already authenticated，不是 HTTP Forbidden。公开 test stream/FAKEPACA 不证明生产 entitlement。
- 原生进程为当前 workspace/source 维护单个 managed connection，限制 auth/subscription deadline、frame 和 buffer。其他应用造成的 connection limit 显式保留并进入冷却/手动修复，不停止他人的连接或并发重试；本切片不使用 wildcard symbol subscription。
- Hot update 使用 typed ephemeral subscription generation/sequence；不伪造 SQLite domain sequence，不为虚构 replay 持久化 quote ticks。重连先将旧观察标为 stale，再等待新认证/订阅流及实际 provider observation；迟到/旧响应不能替换当前证据，无序冲突观察保持 unavailable，不能猜测为 current。
- 同步共享 IPC 生成、中英文 Backend/Frontend/UI 契约、需求证据和运行说明。本地切片不将 FR-016/AC-010 或父级提升为 VERIFIED。

## 测试决策

沿用已同意的最高边界：公开 Settings→Market Explorer/Order Drafts UI→版本化 Rust Control Plane→临时 SQLite 与既有安全 vault 测试边界→固定 endpoint 的假 Alpaca HTTP/WebSocket→权威报价/风险投影。提供方字节必须经实际 adapter 进入，不以直接 fixture snapshot 或 renderer entitlement override 代替。只有一个外层集成测试边界，通过不同提供方响应覆盖故障。

验证 configure/reuse/专用 ownership、准确 feed/path/symbol/header 路由且不泄露值、提供方正例报价/Hot 更新/重开/UI 溯源、断开/凭据变化/迟到响应 generation、IEX/delayed 与 SIP 覆盖差别、已证明的 size 精确规范化，以及无法证明时深度 unavailable。覆盖401/403/429/冷却、非法/超大/缺失/交叉/非正/未来/过期报价、不支持 conditions、导航/stream 重连、不重放及不自动 Live Arm。断言外部可见 quote identity/source status/risk outcome/request count/持久 generation，不镜像内部 helper。

先通过公开边界取得当前 configured producer 缺失的有意义 RED，再运行定向检查、schema/type、最终串行全套、普通桌面构建、390/768/1280px UI，以及基于捕获起点 SHA 的独立串行 Standards→Spec。真实只读验证通过用户显式配置的既有数据服务；缺凭据/entitlement 或休市报价新鲜度保持 unverified/external。Fixture、provider 默认值、旧账户状态或订阅价格不意味着付费访问或金融权限。

## 范围外

此票不完成生产 calendar/halts/corporate actions、执行提供方可交易性、交易级 FX、安全券商权限证明或真实下单/撤单验收；这些仍是原 S28 要求和后续串行工作，不免除。OD-002 历史入库、全市场标的发现、crypto/news/fundamentals adapter、持久 Warm 订阅/全市场 Hot stream、签名打包与 S33–S35 独立处理。S17 Paper、S27实际睡眠唤醒仍 open。整张 map 完成前不提交 main PR。

## 补充

2026-09-30 核对权威资料：

- [Alpaca 订阅与认证](https://docs.alpaca.markets/us/docs/about-market-data-api)：Basic 美股实时为 IEX，完整全美交易所实时需要适用订阅。
- [Latest quotes](https://docs.alpaca.markets/us/reference/stocklatestquotes-1)：显式 feed/currency、固定 latest-quote endpoint，默认 feed 随订阅变化。
- [Market Data FAQ](https://docs.alpaca.markets/us/docs/market-data-faq)：latest SIP endpoint 需要订阅，symbol/时间戳事实与限额仍重要。
- [CTA/UTP size 单位变更](https://docs.alpaca.markets/us/changelog/marketdata-bid-and-ask-size-display-change)：发布于2025-10-30，2025-11-03生效，CTA/UTP quote size 从手数改为股数，在该 feed/date 范围替代通用页面的旧描述。
- [实时股票数据](https://docs.alpaca.markets/us/docs/real-time-stock-pricing-data)：固定 stream feed、认证/订阅和原始 bid/ask venue/round-lot 字段。
- [WebSocket 协议](https://docs.alpaca.markets/us/docs/streaming-market-data)：10秒内认证、订阅确认、有界连接/symbol 限额及 stream 专用错误代码；test stream 不证明生产 entitlement。
- [Exchange codes](https://docs.alpaca.markets/us/reference/stockmetaexchanges-1)、[quote conditions](https://docs.alpaca.markets/us/reference/stockmetaconditions-1)：提供方元数据须明确映射。

本实现方案沿用 S06 已选定来源与 PRD §§31–35/FR-011/FR-012/FR-057、UI §14、Backend §§41–42。实现票 #116 保持 OPEN。配置阶段正在工作树实现；生产报价/feed 就绪仍未交付或验证。

实现票：[接入已配置 Alpaca 报价与 Hot 订阅](https://github.com/kaiqiangh/tradex/issues/116)。沿用用户已确认的单票及测试边界；无新拆票决策。

当前配置进度：已有引用及独立原生凭据生命周期已在工作树实现，独占 key 清理状态可见。定向检查通过（13项 Node projection、18项 Rust source/workspace/watchlist），native cargo check 与 integration binary build 通过；6项公开 UI 观察覆盖两条路径及390/768/1280px。见[阶段证据](evidence/s28-source-progress/dedicated-key-manifest.json)。外部 vault/provider fixture 不证明实际 Keychain UI、生产报价、Hot 或 provider-hosted feed 访问；这些及最终整票检查/审查仍待完成。

HTTP 阶段进展（2026-09-30）：已在未提交工作树实现所选 feed 的认证访问与按需 latest-quote producer。精确 bid/ask、provider 纳秒时间戳、认证后的 exchange/condition metadata、feed/coverage、已证明的 BASE size 单位及 source/connection 绑定进入共享 Market projection。相同 event 保留首次接收时间/身份；读取失败与所复用账户更新使当前证据失效，后续读取轮换 connection generation。重开清除验证与报价内存。公开 Settings/Market UI 检查覆盖 390/768/1280px。当前检查：21 项默认 Rust source/workspace/watchlist/market 测试、10 项 integration source 测试（含实际 loopback HTTP client 请求）、13 项 Node projection 测试、desktop cargo check、integration build、schema/type/fmt/diff 及规划追溯均通过。见 [HTTP 阶段证据](evidence/s28-source-progress/http-quote-manifest.json)。Provider/vault bytes 使用外部合成边界，不是实际 hosted entitlement，也不是注入的 MarketSnapshot fixture。Hot 仍未激活，并明确显示此状态；coverage-aware risk/consent 集成、剩余对抗场景、普通原生/provider 读取证据、整票最终全套检查及串行 Standards/Spec review 仍待完成。#116、#115、#113 和 map 保持 OPEN；此阶段不代表发布或金融就绪。

Hot 租约阶段进展（2026-10-01）：工作树新增一个管理工作线程及至多一个待处理租约，使用显式 feed 的固定 WSS 地址；实际 TCP WebSocket 边界测试按阶段放行认证、完整 symbol 订阅及报价。欢迎消息或订阅确认均不能创建报价；旧连接先关闭再建立替代连接，迟到认证与旧 release 不能改变新租约。实际 stream 帧通过与 HTTP 相同的精度、metadata、时间、source/account generation 校验进入共享内存投影；另一次 HTTP 更新不能被改称为 Hot 订阅更新。原生 main 窗口接入 supervisor；公开 Market UI 持有并释放租约、读取本地投影，隐藏/卸载及迟到 acquire 均有清理。STREAMING 与报价 freshness、金融资格独立展示。公共结果封套漏列 Hot 类型的问题已通过失败测试复现并修复。见[Hot 阶段证据](evidence/s28-source-progress/hot-lease-manifest.json)。仅外部 fake vault/HTTP 与真实 loopback WebSocket 字节证明此阶段；有界自动重连、剩余 stream 故障/时钟/取消场景、coverage-aware risk/consent、普通原生/hosted provider 读取及最终整票检查和串行审查仍待完成。#116 及上级票保持 OPEN。

重连与安全阶段进展（2026-10-01）：同一页面持有的 Hot 租约现在支持最多三次有界自动重试，每次网络连接都会产生新的连接代次。旧报价立即变为非当前；只有完成认证、完整订阅确认并收到新的实际报价后才恢复 STREAMING。提供方认证、feed、连接上限、已认证错误为终态；提供方错误消息经过清理，超大或无效协议帧不会重试。公开 OS_SLEEP/会话安全处理入口会退役连接并保留来源选择；这是处理入口证据，不是实际 macOS Sleep/Wake 证据。七项实际 TCP WebSocket 边界测试、十项来源测试、21 项相关默认 Rust 测试、一项恢复安全回归测试、14 项 Node 投影测试，以及桌面检查、最终集成构建、schema/type/fmt/diff、规划可追溯检查通过。六项公开 UI 观察包括提供方主动关闭→自动重连第 1 次→恢复实际报价，以及 AAPL/结果/MSFT 切换和 390/768/1280px。页面仍将 CLOCK_UNCERTAIN/STALE 与 STREAMING 分别显示。参见[重连阶段证据](evidence/s28-source-progress/hot-reconnect-manifest.json)。这些外部合成边界不能证明托管提供方权限或金融就绪。其余连接取消、心跳、资源、元数据情景、覆盖范围感知的风险/同意、普通原生/真实提供方读取证据及整票最终检查/审阅仍待完成。#116/#115/#113/map 保持 OPEN，所有更改仍在 dev 未提交工作树。

Hot 资源/协议阶段进展（2026-10-01）：最后 owner 释放及显式终态 stop 现在会关闭实际提供方 socket 并退役公开 lease。被拒绝的 acquire 保留已退役 lease；首次 acquire 前停止的 manager 也拒绝新工作。未请求或未知订阅频道不可建立完整订阅。只有与本次 Ping 随机标识匹配的 Pong 才确认心跳；实际 30 秒 socket 测试证明不相关 Pong 不能取消 deadline 或阻止有界重连。DNS 通过单个未完成主机名解析许可限制工作量，字面 loopback IP 不消耗该许可。首版 guard 错误地将字面测试 IP 也串行化，终态错误的重试次数回归测试捕获此问题后已修正。11 项 Hot TCP 测试、10 项来源测试，以及桌面检查、typecheck、fmt/diff、规划可追溯和最终集成构建均通过。参见[资源/协议阶段证据](evidence/s28-source-progress/hot-resource-manifest.json)。真实 OS resolver 饱和有代码保护，但未做故障注入；不声称真实提供方权限、实际 Sleep/Wake 或金融验收通过。其余取消/元数据/时钟情景和风险/同意路径仍需完成，再执行整票最终验证与串行审阅。#116 保持 OPEN。

报价/风险阶段进展（2026-10-01）：已选择的仅报价 producer 现在进入共享风险与 approval review 路径，不虚构 last trade。当前实时 SIP 在 canonical 上市映射有效时可满足报价 freshness 与显示深度 slippage 检查。IEX、delayed SIP、非普通 condition、方向深度不足及超过策略报价时限仍为 unavailable；错误 venue 在创建草稿时被拒绝。已配置且依赖 last-trade 的限价偏离检查仍为 unavailable。公开读取与 review 保留同一精确 observation 及首次 receipt，BUY 预期支出为精确 ask × quantity，同一不可变 Proposal 在来源变化后会改变 review material。历史 opt-in Live fixture 曾能覆盖显式来源，隔离子进程 RED 复现后已修正为显式选择优先。因独立 arming/calendar/tradability 守卫未满足，公开 approve 仍被阻止，approval history 为空，未调用任何券商 POST/DELETE。本阶段通过 22 项默认来源/market/workspace/watchlist 测试、三项相关 library 风险/审批回归、23 项集成测试（12 来源 + 11 Hot）、桌面检查/构建、schema/type/fmt/diff、规划可追溯及最终集成构建。参见[报价/风险阶段证据](evidence/s28-source-progress/quote-risk-manifest.json)。本阶段没有新增审批 provenance UI 或成功 Prepare/dispatch consent 证据；这些、其余 stream 故障情景、整票最终套件与串行审阅仍待完成。当前普通原生构建恢复了保存的账户，但新读取正在等待本次实际观察到的 macOS Keychain 认证请求；保存记录显示不等于新鲜 provider truth。不声称真实报价权限、整票完成或金融就绪。#116 及父项保持 OPEN，无提交/main PR。

Hot 故障回归进展（2026-10-01）：14 项公开边界测试全部通过。新增覆盖证明：安全凭据读取取消不产生提供方 I/O；认证等待期间断开数据源或重新打开工作区会关闭连接，迟到确认无法订阅；格式错误、交叉报价、非正数、错误符号、未知条件、错误 tape、未来时间及订阅确认前报价共八种情况均不重试、不制造观察。这些是当前运行行为的回归测试，不声称新增功能 RED。Fmt/diff 检查通过。见[故障阶段证据](evidence/s28-source-progress/hot-fault-manifest.json)。真实提供方读取、其余验收、整票最终测试及串行审查仍需独立完成；#116 及父项保持 OPEN。

普通原生读取进展（2026-10-01）：用户完成本次系统认证后，普通桌面构建刷新了已有 Alpaca 账户。公开 Settings 独立复用保存凭据，明确保存 IEX 并取得真实托管接口的认证报价访问验证。实际 IEX 流认证及 AAPL 订阅确认已观察到，但本次窗口未收到流报价帧；Hot 报价验收仍未验证。显式 SIP 读取被拒绝，未发生自动降级；随后明确恢复并重新验证 IEX。脱敏证据不含金融账户数据或凭据：[原生读取阶段](evidence/s28-source-progress/native-read-manifest.json)。当前运行二进制尚不包含独立的等待报价原因修复。#116 及原始金融门禁保持 OPEN。

等待报价呈现修正（2026-10-01）：普通原生观察发现已配置且收到确认的数据源仍被旧的适配器未配置原因描述。公开 Hot 回归 RED 重现生命周期与 Market 详情不一致。缺少观察现在投影为 UNAVAILABLE，并显示匹配租约的有限原因；没有匹配租约时保留数据源验证或失败的修复提示。一项旧的关闭租约期望由 UNVERIFIED 更新为修正后的 UNAVAILABLE 契约，初次失败仍保留在检查日志。最终相关检查通过：22 项默认 Rust、26 项集成（12 项数据源及 14 项 Hot）、桌面 check、fmt/diff、JS 语法及最终集成构建。两项公开 UI 观察使用实际隔离 socket，只确认订阅而不发送报价；键盘和 390/768/1280px 验证确认等待原因准确，没有制造报价、溢出或浏览器错误。见[等待报价证据](evidence/s28-source-progress/hot-pending-manifest.json)。之前运行的普通原生二进制属于独立历史真实读取证据；最终原生构建、其余验收及最终串行审查仍待完成。#116 保持 OPEN。

控制格式与等待元数据进展（2026-10-01）：公开流 RED 证明订阅确认与报价被批量混合后可能创建观察，违反官方控制消息单元素数组契约。现在这类混合或批量控制消息在应用确认或报价之前被拒绝。新增外部边界回归暂停认证元数据读取：公开时间重验仍完成，数据源或 feed 替换成功，迟到旧响应关闭且不发布报价、不恢复验证。15 项 Hot TCP 测试、桌面 check、fmt/diff 及最终集成构建全部通过。见[控制与元数据阶段证据](evidence/s28-source-progress/hot-control-manifest.json)。未声称新增 UI、真实流报价或金融验收。其余审批呈现与绑定验收、最终测试及原生构建、串行审查仍待完成；#116 及父项保持 OPEN。

审批来源呈现进展（2026-10-01）：现有审批对话框现在显示已配置报价的 feed/覆盖范围、上市场所或提供方符号、认证买卖交易所名称、tape/条件、来源版本及连接代次。未验证的深度单位标记为不可用，不标记 BASE。类型及 diff 检查通过。本阶段仅为源码与类型证据：已配置生产数据源审批对话框的浏览器验收及正向 Prepare/dispatch 授权仍未验证，已同意的外部提供方边界被独立解锁交易前置条件阻止。现有公开风险或审阅测试证明报价材料进入后端，数据源切换改变摘要，且不授予审批。见[呈现阶段证据](evidence/s28-source-progress/approval-provenance-manifest.json)。尚未执行最终测试或审查，没有提交、关闭议题或主分支 PR。

解锁交易前只读证据说明（2026-10-01，story20）：现有 DISARMED PLACE 操作仍打开独立 Arm 确认。该视图另请求公开后端审批审阅投影，展示当前不可变 proposal 的报价来源及阻断原因，均为只读证据。核对返回 proposal 或账户身份，不将它呈现为已签发审批，也不改变 Arm 资格。缺少审阅或报价时明确显示不可用；后续显式 Arm 仍重新读取同一意图，并打开全新的独立审批审阅。继续使用已确认的公开 UI/Rust/一次性存储/外部提供方测试边界，报价来自提供方帧，假账户按正常连接保持 UNVERIFIED/DISARMED，不注入授权状态或报价 snapshot。

Arm 前只读 UI 进展（2026-10-01）：公共页面缺少证据区域的 RED 已转为 GREEN。通过正常外部假服务连接、仍为 UNVERIFIED/DISARMED 的 T212 账户，可以在 Arm 前查看实际 Alpaca 适配器产生的相同 SIP 报价标识及精确来源。通过公共设置显式替换为 IEX 后，原不可变提案显示 IEX 尚未验证，不再展示旧报价标识或价格。四项浏览器观察通过键盘和 390/768/1280px 检查，无溢出或浏览器错误。Arm 仍禁用，Approve 不出现；没有写入权限种子或修改经纪商订单。类型、schema、JS 语法、diff 及报价材料摘要回归通过。见 [Arm 前证据](evidence/s28-source-progress/prearm-manifest.json)。这是被阻断时的只读来源检查，不证明已发放审批、正向 Prepare/dispatch 或真实托管 tick。单票完整测试、最终原生构建和串行 review 待完成；#116 及父项保持 OPEN。

HTTP 与事件顺序回归进展（2026-10-01）：真实 loopback HTTP 401/403/429 使保留报价变为 STALE，保持所选 feed 且不泄露私有错误正文。请求计数与实际 Retry-After 截止时间证明按需冷却有界；网络读取不持有 Control Plane 锁，同 feed 恢复会改变 snapshot 标识。真实 TCP stream 404/405 不重试，407 最多重连三次；重复事件保留首次接收时间和标识，更旧或同时间材料冲突会终止并使旧报价为 STALE。定向检查通过。这是现有行为的回归证据，不是新增缺失功能 RED。见 [HTTP/顺序证据](evidence/s28-source-progress/http-fault-manifest.json)。下一步为完整测试、最终普通原生构建和串行 review；金融和真实托管 tick 验收仍未验证，所有父项保持 OPEN。

单票本地完整验证检查点（2026-10-01）：最终默认检查通过 17 Node / 340 Rust（36 项既有 ignored）；集成产物通过 369 Rust（36 项 ignored）；单独编译的 Order Gateway 通过 23 项测试。`npm run desktop:build` 生成普通 release 二进制，固定独立 Gateway 校验值且未启用 integration 特性。最终集成二进制通过六项公共浏览器观察：显式 feed 验证、实际适配器产生的 Arm 前精确报价，以及显式更换来源后同一不可变提案的不可用证据、Arm 禁用/无 Approve、键盘和 390/768/1280px 无错误或溢出。首次完整测试的旧 schema artifact 夹具在新增 v32 表后不一致；已复现、修正并通过完整默认重跑。额外混合 ordinary/Gateway 特性的整工作区诊断有两项普通客户端禁写断言失败，保留 FAILED；实际产物拆分检查通过。见 [最终验证](evidence/s28-source-progress/final-manifest.json)。独立串行 review 和交付待完成；ignored、真实托管 tick、金融正向证据及原父项要求仍未验证。没有关闭、提交或 main PR。

串行复审修复检查点（2026-10-01）：Standards 初审通过并给出三条建议，已用具名精确报价字段、共用敏感 headers 和溯源字段组件解决。独立 Spec 初审为 NOT PASS：接收时间采样过晚、整个握手无总截止时间、验收条件4证据不完整。公开 HTTP 慢元数据 RED 复现接收时间缺陷；adapter 现于元数据读取前记录原始 TimeService 接收时间，拒绝到达时仍在未来的报价，把接收时间绑定到快照身份，并保留重复事件的首次接收时间。Hot 同类慢元数据回归通过。实际 TCP 释放 RED 观察到3.660秒清理延迟；现用覆盖 DNS/TCP/TLS/升级的4秒总截止时间和一个作用域内取消看守线程，验证对端观察到释放后小于500毫秒关闭及慢响应尝试有界。相关14项数据源/19项 Hot 测试通过，Standards 复审通过。见[修复证据](evidence/s28-source-progress/review-fixes-manifest.json)及保留的[初次 Spec 报告](evidence/s28-source-progress/serial-spec-review.md)。Spec 复核和当前版本完整产品/构建/UI 验证待完成；正向已签发审批/Prepare/dispatch、托管实时帧和原父级要求仍未验证，未关闭 issue、提交或创建 main PR。

复审后当前源码验证（2026-10-01）：默认 `npm run check` 通过17项 Node/341项 Rust，36项既有 ignored；integration 产品通过372项 Rust，36项 ignored；独立 Order Gateway 指定测试目标通过23项。普通 `npm run desktop:build`、纯 integration IPC 构建及 fmt/diff 通过。六项新的公开 UI 观察通过390/768/1280px验证：显式已保存 feed 技术验证、实际 adapter 报价进入受阻的只读 Arm 前预览，以及显式替换数据源后同一不可变 Proposal 不再保留旧报价。自建测试页/服务和临时工作区已清理。见[当前验证](evidence/s28-source-progress/review-final-manifest.json)。独立 Standards 为 PASS；Spec 确认两项代码缺陷已修正，但仅因缺少正向已签发审批/Prepare/dispatch证据仍为 NOT PASS。此新 release 二进制未声称普通原生启动、托管实时帧或金融交易通过。#116 保持 OPEN；未提交或创建 main PR。已依据单票约束询问用户如何安排阻断的金融前置依赖。

普通原生托管报价跟进（2026-10-01）：保存的 IEX 数据源完成认证和订阅，真实 Hot 报价进入原生 Market UI。由此发现条件字典不匹配：Tape C 的 `R` 名称为 `Regular Two Sided Open`，而非旧夹具缩写 `Regular`。来源 producer 和风险验证现要求经认证的代码/名称精确配对；其他代码、缩短名称、混合及单边条件仍不能提供执行深度。公开 adapter RED/GREEN 和负面 provider 响应回归独立记录，不覆盖历史检查点。这项修复不扩展 IEX 覆盖或金融权限。当前构建及审查证据随后记录；#116 及所有父项验收保持 OPEN。[官方 Tape C 字典](https://github.com/alpacahq/alpaca-docs/blob/master/oas/data/openapi.yaml#L1531-L1546)。
