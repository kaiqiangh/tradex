# S06 数据源与授权规范

日期：2026-09-13
工作项：核实并选定所需外部数据与授权
依赖：S02 提供方账户边界、S05 typed research 结果边界
状态：已验收（2026-09-14）

## Problem Statement

TradeX 的 RevC 要求在进入行情、研究、组合估值和 Live 安全链之前，明确每个外部数据能力的来源、覆盖、时效、授权、保留和缺数行为（PRD §31–36、§72 OD-001–006）。现有账户连接只证明 broker/account 身份，不能证明市场数据 entitlement；S05 的 research tool 目前能安全返回结构化 `UNAVAILABLE`，但没有展示数据源决策或可审计的授权门槛。若把 provider 名称、fixture 或公开网页当成可用数据，会错误开启研究或 Live readiness。

本项只建立数据源目录、授权/许可元数据、只读探测契约和 UI 可见的 availability gate。实际行情、基本面、新闻、公司行为和 FX 数据拉取分别在 S07–S10/S09 实现；本项不绕过 S02 的 Keychain 或 S05 的 typed research boundary。

## Solution

### Source decisions

每个 OD 项只有在“来源选定 + 能力边界记录 + 当前探测或明确外部配置缺失”后才可交付。目录中的 `status` 采用 `AVAILABLE`、`UNAVAILABLE`、`BLOCKED_EXTERNAL`、`UNVERIFIED`；`AVAILABLE` 只表示该公开接口或已配置 entitlement 的探测通过，不表示任何交易权限。

| ID | 初始来源与范围 | 时效/覆盖边界 | 授权、许可和保留 | 未满足时的行为 |
|---|---|---|---|---|
| `OD-001` | Alpaca Market Data API（美股/ETF；Market Data 与 broker execution 分开） | Basic 仅 IEX 实时、SIP 15 分钟延迟且 websocket 有限；更完整的 SIP 实时与历史范围依赖订阅/entitlement。当前默认只允许研究读取。 | API key/secret 由用户在既有 provider 连接中配置；不把 Alpaca broker 连接等同于 market-data entitlement。商业使用、Redistribution、辖区和本地保留依 Alpaca 当前条款复核，不在代码中默认授予。 | 没有有效 entitlement 时显示 `BLOCKED_EXTERNAL`，禁用实时美股；Research 可声明延迟/有限覆盖，不能显示为 Live ready。 |
| `OD-002` | Alpaca Market Data API 历史 bars/quotes/trades | 历史起始时间、调整参数、限速和最新窗口随计划变化；不能假设无限历史或完整交易所覆盖。 | 与 OD-001 相同；下载、缓存和导出需保留计划、来源、provider 时间戳及条款快照。 | 受影响 backtest/历史筛选返回 `UNAVAILABLE`，不以本地 fixture 或账户余额替代。 |
| `OD-003` | SEC EDGAR `data.sec.gov` submissions 与 Apple XBRL Company Concept（`CIK0000320193/us-gaap/Revenues`） | filings/XBRL 的更新时间由 SEC 发布；字段适用范围以 SEC schema/form 为准，Company Concept 校验固定 CIK、taxonomy/tag/entity 和 USD 数据行。 | 当前公开 JSON API 不要求 API key；每次自动请求必须带可识别 User-Agent 并遵守 fair-access（≤10 req/s）和隐私/使用政策。公开可读不等于 TradeX 获得再分发或商业许可，导出须保留来源与法律状态。 | 未能读取或字段不完整时，基本面字段标记 unavailable/stale，相关 Research 卡片禁用或降级为带原因的 `UNAVAILABLE`。 |
| `OD-004` filings/news | filings 先采用 SEC EDGAR；通用新闻 provider 尚未选定 | SEC filings 不是一般新闻，延迟、RSS/提交处理和覆盖按 SEC 资料；通用新闻没有授权来源。 | EDGAR User-Agent/fair-access 约束；通用新闻只有在后续选定有再分发许可的 provider 后才能接入。 | filings provider 失败时禁用 filings 工具；news 始终 `BLOCKED_EXTERNAL`，不得用搜索结果、提示词或 fixture 充当新闻源。 |
| `OD-005` | Alpaca Market Calendar（NYSE/NASDAQ 等支持的 MIC/market）与 Corporate Actions API | Calendar 提供交易日、开收盘和 early close；Corporate Actions 覆盖 provider 支持的 split/dividend/name/symbol 等事件。停牌、退市、跨市场和完整调整需要 S08 的交叉校验。 | 需要 Alpaca API key/secret；provider 计划、市场覆盖和商业/再分发限制按当前条款复核。 | 没有新鲜日历/公司行为时，相关 equity Live gate 为 blocked；UI 展示旧/未知状态与下一次更新时间，不能继续派发。 |
| `OD-006` | ECB Data Portal EXR/SDMX 日参考汇率 | 工作日约 16:00 CET 发布 30 个币种对 EUR 的参考汇率；信息用途，不代表实际成交价，不是交易级 intraday FX。 | SDMX REST 可公开读取；请求保留 dataflow/series key、获取时间和原始 source URL。ECB 的信息用途声明必须随值传播；稳定币 parity 或脱锚没有被该源覆盖。 | 仅允许带新鲜度/质量声明的研究和组合估值；跨币 Live risk、稳定币换算和执行级 FX 在无独立源时 blocked。 |

### Read-only probe contract

Control Plane 增加只读 `data.source.catalog` 查询和 `data.source.probe` 命令。查询返回不含密钥的 `DataSourceCatalog`：

- `sourceId`、`capabilities`（OD ID 列表）、`provider`、`coverage`、`latency`、`entitlement`、`retention`、`redistribution`、`commercialUse`、`jurisdictions`；
- `officialUrl`、`termsUrl`、`checkedAt`、`probeKind`、`status`、`availabilityReason`；
- `configured`、`verifiedAt` 和可选的 `observedAt`，只记录时间与 HTTP/协议结果，不记录响应正文、API key、secret、User-Agent 联系信息或账户余额。

`data.source.probe` 接受 workspace、source ID 和 `expectedStateVersion`，只执行公开 endpoint 的 bounded metadata/health request，或在缺少用户配置时返回可解释的 `BLOCKED_EXTERNAL`。它不能写 domain state、改变 account/model/risk、启动 order gateway、读取 Keychain 明文或把 200 响应变成 entitlement。Alpaca probe 在没有已连接且允许的 source credential 时必须明确返回 `BLOCKED_EXTERNAL`；SEC/ECB 的无密钥 probe 可记录公开 HTTP 成功，但仍保留数据质量和许可限制。

所有 `ResearchToolResult` 继续通过 S05 的 source/context/request marker 校验。Control Plane 在 `research.run` 和 `turn.start` 成对校验前读取当前目录：`public_market_read` 映射 `OD-001`，`historical_simulation` 映射 `OD-002`，`account_read` 保留 S02 的内部 `control-plane:account` 健康来源。若所属 source status 不是 `AVAILABLE`，typed research 结果只能是 sanitized `UNAVAILABLE`，原因必须指向 source ID 和 gate，不得拼接外部响应文本；即使 source 可用，所属数据切片尚未实现时仍保持 `UNAVAILABLE`。

### UI and interaction

在 Settings → Data & Storage 增加 “Data sources” 区域。每个 OD 项展示 provider、capability chips、checked time、latency/coverage、entitlement/terms link 和状态；`BLOCKED_EXTERNAL`/`UNAVAILABLE` 显示简洁原因和后续动作（连接/订阅/稍后重试），不提供 secret 输入框。用户可以刷新单个公开 probe，按钮在运行中禁用并显示 `Checking…`；连续失败保留上一次观察但将状态标为 stale。探测观察按 workspace/source 保存在当前 Control Plane 进程的内存中，renderer reload/remount 会保留，进程重启会回到静态 `UNVERIFIED`/`BLOCKED_EXTERNAL` 并要求重新探测；不会写入 SQLite、日志、响应正文或凭据。键盘顺序、语义按钮、焦点返回和 390/768 窄屏遵循 UI Spec §14 与 S05 的 picker 约定。

### User stories and acceptance

1. 作为研究用户，我能看到每个 OD 项选定的官方来源、覆盖和授权限制，因此知道“可研究”不等于“可实盘”。
2. 作为没有 Alpaca data entitlement 的用户，我看到明确的 `BLOCKED_EXTERNAL` 和订阅/连接动作，实时行情与 Live gate 不会被错误打开。
3. 作为用户，我可以刷新 SEC/ECB 的只读公开 probe，并看到 checked time、source URL 和 stale/failed 原因；响应正文和敏感 header 永远不进入 UI、SQLite 或日志。
4. 作为审计者，我能从 IPC response 和 evidence 复现 source ID、probe kind、状态版本和失败原因；同一请求不能改账户、模型、风险或线程状态。
5. 在窄屏和键盘操作下，目录卡片、状态、条款链接和 retry 都可达；加载/错误/空目录均有显式状态。

## Implementation Decisions

- 在现有 `protocol.rs` 添加 source catalog/probe 的双向 JSON schema，并运行 schema generator；不通过 `serde_json::Value` 绕过类型。
- 目录使用代码内的常量政策表（少量静态决策无需新表）；`checkedAt/verifiedAt` 作为查询结果元数据，观察值只在 Control Plane 进程内按 workspace/source 缓存，不新增第二个权威数据库；进程重启后按上述静态默认值重新开始。
- 使用现有 `ControlPlane` dispatcher、`request`/runtime validator 和 Settings tab；不新增网络 client 依赖。真实 provider fetch 留给 S07–S10 的 provider adapter。
- 公开 probe 使用现有 Rust stdlib/HTTP 边界或返回 `BLOCKED_EXTERNAL`；测试默认采用确定性 fixture response，fixture 不能产生 `AVAILABLE` 的 entitlement 结论。
- 所有 source URLs/terms URLs 在英文规范和中文配对文档中同步；官方页面、访问日期和真实探测摘要写入本项 evidence。

## Testing Decisions

最小最高验证边界：

1. Rust/schema：目录包含 OD-001–006，`deny_unknown_fields`、状态/理由枚举和空/超长/未知 source ID 校验；probe 不能改变 workspace/account/model/risk/thread stateVersion。
2. Negative contract：缺 Alpaca credential、未知 source、错误 expected version、篡改 status/response 都返回 `BLOCKED_EXTERNAL`/schema error；不会把 HTTP fixture 或公开 200 误判为 market entitlement。
3. Public probes：在隔离环境记录 SEC 200（带 User-Agent）、ECB EXR 200 和无 credential Alpaca 401；外网不可用时证据标为 unavailable/flaky，不升级为 PASS。
4. Browser/Rust-backed UI：Settings Data & Storage 目录的 loading/error/retry、terms link、键盘路径和 390/768 无横向溢出；浏览器日志为零。验证 source catalog 的响应实际影响状态文案。
5. Existing full checks：`npm run check`、Rust integration tests、`cargo clippy --workspace --all-targets --features integration-test -- -D warnings`、format/schema/traceability；不运行真实下单或写入用户凭据。

## Out of Scope

- S07–S10 的行情、Watchlist、市场时段、组合、新闻、filings、fundamentals 和研究工具实现。
- 购买数据订阅、接受商业/再分发合同、录入或删除任何用户 key；不存在数据 source 的账户不能被本项自动创建。
- 交易级 FX、稳定币脱锚模型、完整跨市场交易日历、期货/A 股/新增券商（FR-041–043 仍 DEFERRED）。
- 把 Alpaca market-data entitlement、SEC/ECB 公开访问或任何 fixture 解释为 Live execution authorization。

## Further Notes

- 官方来源（访问日 2026-09-13）：
  - Alpaca Market Data API：[about-market-data-api](https://docs.alpaca.markets/us/v1.1/docs/about-market-data-api)、[Get Market Calendar](https://docs.alpaca.markets/us/reference/calendar-2)、[corporate actions](https://docs.alpaca.markets/us/reference/corporate-actions)。
  - SEC：[EDGAR APIs](https://www.sec.gov/search-filings/edgar-application-programming-interfaces)、[Developer Resources / fair access](https://www.sec.gov/about/developer-resources)、[EDGAR API toolkit User-Agent](https://api.edgarfiling.sec.gov/docs/index.html)。
  - ECB：[exchange rates and information-only disclosure](https://data.ecb.europa.eu/key-figures/ecb-interest-rates-and-exchange-rates/exchange-rates)、[SDMX web services](https://data.ecb.europa.eu/help/getting-data-web-services-sdmx-0)。
- 2026-09-14 隔离只读探测摘要：SEC `data.sec.gov/submissions/CIK0000320193.json` 与 `data.sec.gov/api/xbrl/companyconcept/CIK0000320193/us-gaap/Revenues.json` 均返回 HTTP 200 并通过固定身份/字段校验；ECB `EXR/D.USD.EUR.SP00.A` CSV 返回 HTTP 200 并通过固定 series/完整行校验；Alpaca calendar 在无 API headers 时返回 HTTP 401。请求带 identifying User-Agent，响应正文未写入仓库。详见 [S06 evidence](s06-data-source-evidence.md)。
- S06 完成后，S07 只能消费目录中 `AVAILABLE` 且具有适用 coverage/quality 的 source；OD 仍 blocked 时，S07–S10 必须在 UI 与 typed result 中保留该原因。
