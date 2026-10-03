# S28 公司行为与精确 Trading 212 标的证据

调研日期：2026-10-03。代码基线：dev `309419f946a3f82c43672ec08b694ab5942ae5fe`。
状态：**RESEARCH_COMPLETE；IMPLEMENTATION_NOT_STARTED；正向金融前置条件仍未验证**。
父规范：[补齐 S28 生产金融前置证据](https://github.com/kaiqiangh/tradex/issues/117)。
已批准的串行前项：[接入 Alpaca XNAS 日历与可信时段](https://github.com/kaiqiangh/tradex/issues/118)，CLOSED。
英文配对：[Corporate actions and exact Trading 212 instrument evidence](s28-actions-tradability-research.md)。英文版本为权威来源。
已发布规范：[第二项串行前置](https://github.com/kaiqiangh/tradex/issues/117#issuecomment-5966487911)。
已认领实现票：[读取已知公司行为与精确账户标的证据](https://github.com/kaiqiangh/tradex/issues/119)，OPEN。

本次只有公开一手资料研究与本地源码核对，没有读取凭据或真实账户、调用已认证 provider、测试、变更权限或证明 runtime 验收。10月1日原生日历是历史证据，不是现在30秒内的执行权威。原报价票、Trading 212 Live 验收与整个 map 均有独立门禁。

## 可据此执行的决定

分别实现显式选择、已认证的已知事件观察，以及精确所选账户的 broker 元数据。完整金融需求保持不变：事件可读或目录命中均不能证明行为覆盖完整、历史已调整、当前所选账户可不受限制交易或无停牌。下面的公开契约不能提供这些正向保证；代码可以展示并执行该限制，不能制造 provider 契约。

下一张单票可以交付可观察、可单独演示且失败关闭的完整纵向切片；关闭它不能关闭父前置规范或原审批/Prepare/派发正向验收。后者需要真正支持的权威证据，或明确批准的规范契约决定；本次没有批准契约降级。

## 本地要求与消费路径

权威顺序仍为英文 RevC PRD → UI Spec → ARD，共享 wire 由 Backend §41–42 定义。PRD §20 绑定不可变交易和当前市场证据；§34 要求明确 entitlement、保留、再分发和司法辖区；§35 要求拆股、分红、标的变更、退市、停牌及独立历史调整。UI C4/C6 与 Frontend §13.4 要求事件/时段与确定性修复提示。Backend §15.3、§41.13 定义现有公司行为载荷；§41.38 只补齐日历。

| 当前代码 | 调研结果与实现含义 |
|---|---|
| `src-tauri/src/market.rs:193` | canonical `equity:US:AAPL` / `equity:US:MSFT`，USD，上市 XNAS；Alpaca AAPL/MSFT，Trading 212 AAPL_US_EQ/MSFT_US_EQ。这是本地映射，不是当前 broker 确认。 |
| `src-tauri/src/protocol.rs:2277`、`market.rs:611` | `CorporateAction` 要求 RFC3339 `effectiveAt`，仅四类，可选 `announcedAt`/`sourceId`，列表最多16条；没有处理日期覆盖、分页完成或 provider 修订 provenance。基线不存在 `CorporateActionDetails` 类型。 |
| `src-tauri/src/protocol.rs:2260` | `InstrumentTradingObservation` 只有 provider/ticker/venue/status/单一 observedAt/source，缺少所选账户、版本、凭据/来源/会话绑定和 provider 时间与首次接收时间的区分。禁止把全局目录写成 `TRADABLE`。 |
| `src-tauri/src/provider_io.rs:350` | 当前 Live GET 白名单仅账户摘要、持仓、订单及受限历史；Alpaca 数据白名单仅报价。新增固定路径须显式校验，不能开放任意路径/host/method。 |
| `src-tauri/src/lib.rs:7560` | market 投影进入 risk、审批与后续复验。新事件/标的选择须沿用已配置生产日历/报价阻止历史合成 fixture 替代的原则。 |
| `src-tauri/src/risk.rs:2023` | InstrumentRules 匹配执行 provider/ticker/proposal venue、canonical 身份与新鲜度，但未精确绑定所选账户。HALTED 拒绝，缺失/不匹配阻止。 |
| `src-tauri/src/risk.rs:1026`、`lib.rs:8150` | market 材料进入决策 digest；读时间会被归一化移除。新增覆盖、首次接收、绑定版本与质量须保留在材料 digest 中；轮询不得延长权威。 |

日历 PASS 不是公司行为/调整检查。现有 `RiskCheckId` 没有独立公司行为或历史调整检查。旧 `market_execution_eligibility` 要求已知调整状态，也不能证明每个金融消费点独立执行行为完整性门禁；实现规范须明确这些检查与复验，并保留全部既有阻止条件。

## 当前官方契约

哈希只标识文档字节，不是 provider 响应证据：

| 文档 | 2026-10-03 身份 |
|---|---|
| [Alpaca REST Markdown/OpenAPI](https://docs.alpaca.markets/us/reference/corporateactions-1.md) | Market Data API 1.1；1项 GET；47,620字节；SHA-256 `b3c31fadd5a7ae89f8c0e4bd7d575bc5180faceda69ea1f115b4c1710d835803`。front matter updatedAt 2026-05-27 不代表最新 schema 变更日期。 |
| [Alpaca 公司行为 SSE Markdown/OpenAPI](https://docs.alpaca.markets/us/reference/subscribetocorporateactionseventssse.md) | Market Data API 1.1；1项 GET；96,409字节；SHA-256 `123a2f137025c09e8e616b38dc22725dc1edcb370f31251b604258a4cc6ab3c2`。 |
| [Trading 212 完整 OpenAPI](https://docs.trading212.com/_bundle/api.json?download=) | API v0；77,378字节；SHA-256 `a272f70a713fa9f2f906e9b4be12136c9d5d63d7f9f48562e939aa35c4fc041d`；22项操作、17条路径，与10月1日一致。 |

解析使用自有 `/tmp/tradex-{alpaca,t212}-actions-research-current.*` 与 `/tmp/tradex-alpaca-actions-sse-research-current.md`。一次 Python 公共获取返回403，随后公共文档浏览/普通 curl 成功；这是文档获取结果，不是已认证 entitlement 结果。

### Alpaca 已知公司行为

[REST 契约](https://docs.alpaca.markets/us/reference/corporateactions-1) 为 GET `https://data.alpaca.markets/v1/corporate-actions`；sandbox 数据 host 与 Paper trading host 不同。显式使用选定已存凭据，日历成功不证明数据 host 的 entitlement。`start`/`end` 包含边界，过滤的是 **process_date**，不是公告/经济生效时间。`data_quality=all` 包括早期不完整事件；默认 `complete` 也可能包含已处理但仍不完整的记录。上游采集/处理有延迟且不保证创建时间。limit 1–1000 是总记录数，必须遍历 next_page_token 到结束。返回分组 corporate_actions 与可空 token。200/400/401/403/429/500 须有明确不可用/限流处理。

[schema](https://docs.alpaca.markets/us/reference/corporateactions-1.md) 中 UUID id 是事件身份；symbol/old-new symbol 与可选安全标识用于 canonical 匹配；process_date 是处理日期；ex_date/effective_date 是日期精度经济字段；ratio/rate 保留精确十进制。name_change/worthless_removal 没有经济生效时间戳；REST 不定义创建、更新或公告时间。保留缺失/部分日期，禁止补零点 UTC、把处理日期写成公告时间、把 worthless_removal 写成退市。

当前16组是 forward/reverse/unit split、cash/stock dividend、spin-off、cash/stock/mixed merger、redemption、name change、worthless removal、rights distribution、partial call、reorganization、capital-gains distribution。未支持类别应可见或明确 unsupported，不能悄悄丢弃。官方[五月变更](https://docs.alpaca.markets/us/v1.1/changelog/2026-05-22-corporate-actions-5c87d2b)、[六月变更](https://docs.alpaca.markets/us/changelog/2026-06-03-market-data-9dddd18)、[八月变更](https://docs.alpaca.markets/us/changelog/2026-08-27-capital-gains-distributions-fce488a)分别补入部分新类型、安全标识/region/currency、capital-gains distribution。应扩展 typed wire，而不是假定旧四类/必填时间戳已覆盖官方数据。

**推论：**分页结束只证明返回的受限处理日期查询已遍历完成。空组不能证明没有刚公告/未来行为；处理延迟、窗口外变更仍可能存在。日期精度必须上 wire。历史调整继续独立 UNAVAILABLE/UNVERIFIED，直到真实 history provenance 证明它。

[SSE 契约](https://docs.alpaca.markets/us/reference/subscribetocorporateactionseventssse) 为 `stream.data.alpaca.markets/v1beta1/events/corporate-actions`，支持 insert/update/delete、region/type 与 timestamp/ULID/Last-Event-Id replay。event_id 是 mutation 身份，ca.id 是行为身份，at 是发出时间而非公告/经济生效时间；decimal 是字符串。它共享 REST 数据集，未来可处理修订/删除，不能消除上游延迟；订阅 ACK 不证明当前完整初始状态。不得把已批准有界 REST 工作悄悄替换为新 SSE 能力/entitlement。

### Trading 212 精确账户标的元数据

[官方概览](https://docs.trading212.com/api)限定 Invest/Stocks ISA、认证请求、独立 Live/Demo host、每账户限流和主账户币种执行。这里选 Live，禁止回落 Demo。

[完整 OpenAPI](https://docs.trading212.com/_bundle/api.json?download=)定义：

| 固定 Live GET | 内容与限额 |
|---|---|
| `/api/v0/equity/metadata/instruments` | 数组：ticker/type/ISIN/currencyCode/name/shortName/addedOn/maxOpenQuantity/workingScheduleId/extendedHours；10分钟更新，1次/50秒。 |
| `/api/v0/equity/metadata/exchanges` | exchange id/name，workingSchedules{id,timeEvents{date,type}}；OPEN/CLOSE/BREAK_START/BREAK_END/PRE_MARKET_OPEN/AFTER_HOURS_OPEN/AFTER_HOURS_CLOSE/OVERNIGHT_OPEN；10分钟更新，1次/30秒。 |

两项均定义200/401/403/408/429；403 是缺元数据 scope。操作 schema 未分页。没有 required 字段表、MIC、provider observation timestamp、当前 halt/restriction、account identity 或 discontinued 字段。maxOpenQuantity 没有语义描述；存在只证明认证可读元数据，不能证明完整当前可交易，也不从零/缺失推断买卖权限。

同一个所选凭据/版本先读取 account summary 的 id/currency，与保存账户比较。精确匹配唯一受支持 ticker、STOCK、USD、安全身份，唯一关联 workingScheduleId。名称/排期 ID 不能验证 MIC XNAS。本地 symbol mapping 需要冲突处理和安全身份佐证，symbol change 不自动重映射。

已弃用 Pie 详情的 InstrumentIssue 枚举包含 suspension/delisting/no-longer-tradable 等，但只覆盖已有 Pie 上下文；完整22项操作没有完整所选账户当前可交易/停牌 endpoint。paid dividend history 是历史账户活动，不是未来完整行为覆盖。不得为取证创建/修改 Pie 或探测订单。官方[暂停交易说明](https://helpcentre.trading212.com/hc/en-us/articles/360007293218-Why-might-trading-in-a-specific-instrument-be-suspended-unavailable)列举合规、监管与交易所原因。**推论：**broker 限制可不同于公开交易所状态；10分钟前的目录不能因现在接收而获得实时质量。

### 其他已公开能力及使用权限制

[Alpaca status channel](https://docs.alpaca.markets/us/docs/real-time-stock-pricing-data)含 symbol、status/reason、provider timestamp、tape；quotation resumption 不等于 trading resumption。真实 access/entitlement 要单独验证，官方 production 指引提到 sales。它可提供正确作用域的交易所状态，但不能证明 Trading 212 所选账户限制。既有 quote stream 没有完整 status producer；本研究不选择新付费计划/provider/source。

[已弃用 announcements](https://docs.alpaca.markets/us/reference/get-v2-corporate_actions-announcements-1)指向新 endpoint，不是完整性补救。官方[Market Data FAQ](https://docs.alpaca.markets/us/docs/market-data-faq)区分最新未调整与历史调整；下载事件不能改变 DuckDB history provenance。

OpenAPI 指向的 [Alpaca 条款](https://s3.amazonaws.com/files.alpaca.markets/disclosures/library/TermsAndConditions.pdf)有个人/非商业、额外 subscriber agreements 与发布/分发限制；本次没有建立公司行为特定保留期限或司法辖区 entitlement。[CUSIP notice](https://docs.alpaca.markets/us/v1.1/changelog/cusip)指出部分条目有额外 license。文档 license 不等于 market-data license。建议只持久化配置/审计哈希，观察在进程内，不宣称任意归档/导出/再分发权。

当前 [Trading 212 API Terms](https://www.trading212.com/legal-documentation/API-Terms_EN.pdf)标注17.10.2025，包含个人/测试使用、第三方分发及应用/自定义界面 consent 条件；§3.4 指明 API orders 经过 **Systematic Internaliser**，§7.1 指明提供的市场数据并非实时。**推论：**XNAS 是上市/日历身份，不是实际 broker 执行场所证明。下一规范须保留区别，不得擅改执行 venue 不变量。provider 使用授权/数据权利与账户权限验证分别处理。

## 下一张单票的验收草案

以下是实现验收；缺失正向权威仍在父规范、Trading 212 与报价验收中保持 OPEN。

1. Settings 显式复用合格已存 Alpaca Paper 凭据读取固定生产 data host；选择精确已存 Trading 212 Live 账户。Save/Refresh/Disconnect 有版本；断开不删除借用账户/key；无 secret 输入、host/key fallback、renderer authority、financial write 或自动 Arm。
2. typed wire 包含处理日期覆盖、原始日期精度/可空字段、16类、稳定事件 ID、精确 decimal、首次接收、材料版本与部分/分页完成状态；不造生效/公告/provider 时间或误分类；同步中英文与生成 schema/types。
3. 全类别 data_quality=all，明确 US canonical symbol 与受限 process-date window；建议 UTC今天−30至+30，是本地查询界限而非未来完整覆盖。规范须定义分页/记录/字节/时间/token 上限；全部完成后原子发布，错误/超限/token循环/冲突撤销资格；不截成16条后宣称查询完整。
4. 保留当前受限快照中的事件身份/修订、material version；重复/冲突、日期/比例/安全身份错误、新类别明确失败关闭。已知事件作为上下文，完整无事件、退市覆盖、history adjustment 保持 unsupported；Refresh 替换当前窗口，不是全局历史。
5. 同精确凭据核对 remote account identity，再唯一精确 ticker/安全身份/币种/类型/schedule join。记录10分钟质量/限流/首次接收；不造 provider time/MIC/tradability。metadata-only 可 AVAILABLE，执行可交易保持 UNVERIFIED/BLOCKED_EXTERNAL。
6. jobs/results 绑定 workspace、connection/account version、私有 credential ref、source generation、process session、clock generation、request sequence、canonical identity。vault/HTTP/quota 在 Control Plane lock 外；旧 job 无法发布；严格固定 GET/path/host/no-redirect、并发/取消受限。
7. 独立投影已知事件、查询完成、完整覆盖、broker identity/metadata、交易所 halt、账户当前 tradability、history adjustment。一个能力成功不能使 OD-005 Verified；准确说明能力缺口，不虚构换 key/人工确认补救。
8. 增加独立公司行为/调整与精确账户标的门禁；未知完整性/不支持的当前可交易不可 PASS。权限 UNVERIFIED/DISARMED、SIP/quote/calendar/FX 与其他门禁保持；Refresh 不能把负面/不完整证据变成正向。
9. risk preview、不可变审批材料、Prepare、dispatch 同源复验变化/陈旧/错绑定；轮询不改变首次 receipt/material version；配置生产来源阻止历史 fixture 替代；无隐藏 seed/注入 MarketSnapshot 制造正向测试。
10. 公开边界 RED/GREEN：普通账户设置 → typed 真实 Rust → 临时 SQLite/outbox → 外部 fake vault/HTTP。多页/部分/空/缺日期/类别/修订、缺失/重复/错身份目录、403/429/超时/超大/跳转、10分钟质量与30秒 receipt、时间/来源/账户/workspace 竞态、零泄密/零 broker mutation；普通权限仍未知、缺能力仍阻止。
11. Settings/Markets/Trade 展示已知/部分证据及独立 blocker；键盘、未保存选择禁止 Refresh、错误/reload/expiry、390/768/1280px。可行时普通 native hosted 只读只证明实际响应能力；证据分清 fake/hosted/stale/skipped。
12. 合适最终全套/schema/type/build、普通桌面构建、独立串行 Standards 再 Spec review，绑定起始基线和最终 dev SHA，提交/正常推送后才关闭。父正向完整性/当前账户可交易保持 OPEN；仅真实依赖建立 tracker edges，不把串行顺序伪装为依赖。

## 全 goal 仍需的外部证据

当前公开契约不能建立完整 Trading 212 key/IP scope、精确账户当前无限制可交易、未来公司行为/退市完整性、真实历史调整，或满足把上市 venue 当 broker execution venue 的断言。status entitlement、保留/分发权利也需要适用真实契约。这是证据清单，不是放松 RevC、购买计划或发送权限探测的请求。实际必需 FX 是随后已批准项，本次尚未开始。

父级完整验收保持不变。可做的只读实现继续串行，外部能力缺口保持可见，不能凭本地 PASS 关闭。
