# TradeX RevC 需求清单与串行实现方案

日期：2026-09-06。基线：`9255feae39b646245acaf6db7db29fea0cb710c7`。开发分支：`dev`。

**这是完整应用的实施计划；S01、S02、S03 已完成，S04 的 #17 Thread 历史、#18 Turn 流式和 #19 取消/重试/安全恢复切片已完成；S05 的 #21 能力策略与可信模式上下文、#22 context catalog/picker、#23 typed research tool boundary 已完成验收；S06 的 #24 数据源规范与 #25 只读目录/探测已完成实现和隔离验证；S07 的 #27 市场目录、详情与历史边界、#28 Watchlists CRUD 已完成验收；S08 的 #30 可信时间门禁与 #31 市场时段、停牌与公司行为已实现并通过独立审查；S09 的 #32 组合与 FX 估值只读切片已通过 schema、Rust、前端和 Rust-backed 浏览器验证；S10 的 #34/#35 source-gated typed research 与股票/现货 evidence cards 已完成当前切片验证；S11 的 #36 规范、#37 source-gated screener 实现与 #38 持久化/附加候选项已完成验收；S12 研究产物保存、检索与导出已完成垂直验收，状态为 VERIFIED；S13 #42 Draft 保存/编辑、#43 不可变 Proposal 生成与 #44 Proposal 刷新已完成验收；S33 全量回归和真实 provider 边界仍待后续工作项；OD-001/OD-003/OD-005/OD-006 真实授权、S33 全量回归与后续 authority consumers 仍待完成；S14 已完成（实现 #46，当前证据见 [S14 strategy sandbox](s14-strategy-sandbox-evidence.md)）；S15 规范已写入 [S15 可复现历史回测](s15-backtest.md)，#48/#49 已完成当前生命周期与确定性结果切片（证据见 [S15 回测运行生命周期](s15-backtest-evidence.md)），#50（compare 与双入口结果收口）仍待实现；S15–S35 尚未完成。** S02 四张提供方票及整体验收已通过，S03 规范和串行实现票已发布并完成验收；#11 网关生命周期、#12 模型连接的真实 OAuth/上游推理、#13 默认路由与回退审计、#14 五步入门与风险默认值、#15 账户复用与隔离本地清理均已绑定实现、协议、桌面或 Rust-backed 浏览器证据。#17 的 Thread/history 证据见 [S04 Thread 历史验收](s04-thread-evidence.md)，#18 的 Turn 证据见 [S04 Turn 流式验收](s04-turn-evidence.md)，#19 的取消/重试/恢复证据见 [S04 #19 验收](s04-cancel-retry-evidence.md)，#21 的 capability 证据见 [S05 #21 能力策略验收](s05-capability-evidence.md)，#22 的 context catalog/picker 证据见 [S05 #22 上下文目录验收](s05-context-evidence.md)，#23 的 typed research 证据见 [S05 #23 typed research 验收](s05-typed-research-evidence.md)，#24/#25 的数据源证据见 [S06 数据源验收](s06-data-source-evidence.md)，#27 的市场证据见 [S07 #27 市场验收](s07-market-evidence.md)，#28 的 Watchlists 证据见 [S07 #28 Watchlists 验收](s07-watchlists-evidence.md)，#31 的市场状态与公司行为证据见 [S08 #31 验收](s08-market-evidence.md)，#32 的组合与 FX 估值证据见 [S09 验收](s09-portfolio-evidence.md)；S10 #34/#35 的 typed research 与 evidence card 证据见 [S10 research](s10-research-evidence.md) 和 [S10 evidence cards](s10-evidence-cards-evidence.md)；S11 规范与验收证据见 [S11 screener](s11-screener.md)、[S11 screener persistence](s11-screener-persistence.md) 和 [S11 screener evidence](s11-screener-evidence.md)；S13 #42 的 Draft 证据见 [S13 Draft 验收](s13-draft-evidence.md)，#43 的 Proposal 证据见 [S13 Proposal 验收](s13-proposal-evidence.md)，#44 的刷新证据见 [S13 Proposal refresh 验收](s13-proposal-refresh-evidence.md)。** 已逐一阅读基线下全部 23 个受版本控制的 docs 文件（含中英文文档、工作流说明及 HTML/CSS/JS 原型），不是只检索需求编号。原型 QA 的 9 个 FAILED、3 个 PARTIAL 保留为历史证据，不能升级为应用验收。当前证据见 [S01 verification](s01-evidence.md)。

- [逐条需求清单](requirements.csv)：201 条 FR/AC/NFR/SEC/DATA/OPS/UX 的原文、来源行、实施项、验证边界和状态；FR-041–043 按规范 DEFERRED，其余 198 条均需完成。
- [页面及原型回归清单](surfaces.csv)：UI Spec 全部页面与 QA-01–12 的负责工作项。
- [已阅读文件清单](sources.csv)：基线文件路径、行数、SHA-256。哈希只固定阅读来源，不证明行为通过。
- [S02 整体验收](s02-integration-evidence.md)：同一 dev SHA 上七个环境组合、四项原生 Keychain 检查和 11 条独占需求证据。
- [S02 Alpaca 验证进展](s02-alpaca-evidence.md)：精确实现 SHA、自动/原生验收、已修复问题与剩余提供方范围。
- [S02 Trading 212 验收](s02-trading212-evidence.md)：Demo/Live 隔离、精确数值、原生 Keychain/取消/失败路径及串行审查。
- [S02 Binance Spot 验收](s02-binance-evidence.md)：签名与服务端时间、密钥权限阻断、原币精度、Testnet/Live 原生输入及清理。
- [S02 Bitget 验收](s02-bitget-evidence.md)：三字段凭据、Classic 签名、权限/IP 变化、Demo 限制和三类订单分页。
- [S03 网关生命周期验收](s03-gateway-evidence.md)：#11 固定 CLIProxyAPI 生命周期、真实进程探测/冲突/退避/清理、公共 Settings 页面和串行 Standards/Spec 审查；#13 路由/回退/配额、#14 入门/风险与 #15 账户复用/清理验收均已完成，S03 已收口。
- [S03 模型连接验收](s03-model-evidence.md)：#12 ChatGPT OAuth / DeepSeek Keychain / 精确 Flash 路由验证的实现、协议/浏览器证据和真实 sidecar 失效 UI。
- [S03 入门与风险验收](s03-onboarding-evidence.md)：#14 五步入门、Risk Defaults、SQLite schema v5、Ready 门控、浏览器交互和公共协议负例；Ready 实际桌面与当前代码浏览器路径已重跑；S04 和 S33 的外部或完整回归门槛仍待补齐。
- [S03 账户复用验收](s03-account-evidence.md)：#15 权威账户读取、已有连接复用、版本化刷新、隔离本地清理、键盘与 390/768 布局证据；真实桌面账户只读检查，未删除真实凭据。
- [S04 Thread/Turn 运行时 Spec](thread-runtime.md)：#16 持久化 Thread、不可变 Turn 来源、Codex App Server 流式事件、取消/重试与安全恢复的规范和三张依赖有序实现票。
- [S05 Agent 能力与上下文 Spec](s05-capability-context.md)：#20 分离 Agent Mode / Execution Context，定义可信 capability policy、canonical context refs 和受限 typed tools 的串行实现边界。
- [S05 #21 能力策略验收](s05-capability-evidence.md)：固定 SHA `6d479024ac0a62592e71fbb9b49acc106215fb90` 上的 capability matrix、trusted `turn.start` 重算、schema/runtime 边界、Rust 与隔离浏览器证据；#22/#23 在该历史快照之后完成。
- [S05 #22 上下文目录验收](s05-context-evidence.md)：canonical `context.catalog`、非秘密账户 hash、可用性 gate、临时 Context/Account picker、显式空目录、空数组清空语义、焦点/inert/窄屏与 Rust-backed 浏览器证据。
- [S05 #23 typed research 验收](s05-typed-research-evidence.md)：typed data-plane registry、sanitized unavailable result、tamper/no-mutation guard、source/context/marker timeline provenance、隔离 bridge 负例与 390/768/accessibility 证据。
- [S04 Thread 历史切片验收](s04-thread-evidence.md)：#17 schema v6、Thread create/list/get、snapshot/replay/subscribe、历史恢复和 390/768 浏览器证据。
- [S04 Turn 流式验收](s04-turn-evidence.md)：#18 不可变 Turn 来源、受限 Codex stdio 适配器和流式 typed timeline；#19 取消/重试/安全恢复的逐项证据见 [S04 #19 验收](s04-cancel-retry-evidence.md)。
- [S02 提供方连接 Spec 与拆票](provider-connections.md)：四个提供方垂直切片、权限差异、原生凭据边界及官方资料。
- [首个工作项 Spec 和实现票](first-slice.md)：可审阅的测试边界与第一个垂直切片。
- [S15 可复现历史回测 Spec](s15-backtest.md)：冻结配置、确定性运行、结果 manifest 和 compare 的串行边界。
- [S15 回测运行生命周期证据](s15-backtest-evidence.md)：#48/#49 的 schema、SQLite、状态生命周期、deterministic result、manifest、guard、浏览器 fixture 与验证命令。

## 1. 文档分析与实施约束

1. 英文 PRD 为产品权威；UI Spec §14 定义目标交互；Backend ARD §41–42 定义唯一 IPC wire contract。中文翻译、覆盖矩阵和原型不能改变该顺序。
2. 原始阅读基线没有应用代码、数据库、测试或构建工程；S01 已建立首条真实工作区路径。采用规定的 Tauri + React/TypeScript + Rust Control Plane；复用原型视觉、信息架构和状态术语。原型全局变量、定时器金融状态和固定身份不能成为应用权限实现。
3. Rust 负责金融权限；Codex/研究/策略属于不可信区域。Order Gateway 必须是固定版本的独立子进程，通过继承私有双向通道认证；不开放 TCP，也不成为第二个 SQLite 写入者。不要按 ARD 推荐目录树预建空 crate；有实际边界需要时拆分。
4. SQLite WAL/事务保存权威领域状态和 outbox；DuckDB 保存历史行情、筛选和回测分析数据；文件保存策略、产物及备份。金融金额、数量、费用、FX 使用十进制定点/decimal，wire 使用规范字符串。
5. Model inference 只经过 `127.0.0.1:8317` CLIProxyAPI；ChatGPT OAuth 与 DeepSeek 官方模型是两个允许的 provider。Codex 与 CLIProxyAPI 的精确版本和协议须通过当前官方资料与实测固定。默认关闭跨 provider fallback，保留全部 attempt 审计。模型故障不停止可信控制面。
6. 密钥只进入受信原生输入通道和 OS Keychain。持久化/UI/日志/Agent 只有脱敏 metadata 与引用；sidecar 的 OAuth 目录独立，临时 DeepSeek config 0600 且退出清理。权限无法探测时显示 UNVERIFIED；危险权限阻止 Live readiness。
7. Mode、Execution Context、所选账户、arming 相互独立。Turn 开始快照不可变；provider attempts 在快照之外追加。恢复历史与变更选择器都不能恢复 arming 或既有审批。
8. 可编辑 Draft 生成新的不可变 Proposal 身份/hash；编辑/刷新重新同意。PLACE 与 CANCEL 是不同的审批意图。审批、预留、派发、broker ACK、fill 是不同事实。
9. Live 每次审批前、审批消费时和派发边界都校验当前资格；消费与预留共用数据库正确性边界。未知传输保留容量并查询，不能盲重试或超时释放。Disable All 的返回值区分派发前已停止与可能已提交。
10. 执行方案覆盖规范正文中的行为，不止 201 个编号：Watchlists、新闻/filings/fundamentals、归档校验、许可证元数据、审计保留、沙箱、键盘焦点、窗口布局等均在下方分配。新 IPC 操作先补双语契约，再生成/校验 Rust 与 TS 类型。
11. 发现中文 PRD §26.1 相比英文缺少字段敏感性/必填性与完整权限安全检查段落，§26.3 缺少部分路由披露文字；在对应 provider/model 项同步补齐并更新文档 manifest。其他地方已有相关 guard，不代表可以忽略配对差异。

## 2. 固定工作流

**当前项完成全部步骤后才开始下一项；不运行并行工作 agent。** 下表的依赖只表示真实技术前置条件；执行调度另加严格顺序，不把无技术依赖的项伪装为架构依赖。

1. 读取当前 Wayfinder map、前沿、相关已决策票和本项规范；认领本项。
2. `to-spec`：写用户行为、拒绝/故障路径、接口/存储决策和最高可用测试边界。沿用经确认的通用验证约定。
3. `to-tickets`：拆成一次可完成的垂直切片；每张票包括 schema/后端/UI/验证，建立原生 sub-issue 和真实 blocker；复杂 provider 项可在该项内继续串行拆分。
4. `implement`：先保留本项起始 SHA；从失败用例到实现，运行相关类型检查和行为测试；每个完成切片原子提交至 dev。绝不实施下一项来掩盖本项未通过。
5. `code-review`：对起始 SHA 到提交的 diff 依次做 Standards、Spec 独立审查，修复所有已确认问题；更改后的相关验证须重跑。用户的串行要求优先于技能默认并行审查。
6. 完成本项要求的整体验证，将精确 SHA、运行命令、通过/失败/未跑原因、运行时证据写入 issue resolution；仅证明完成后关闭该层 ticket/spec/map child 并更新 map 索引。
7. 所有项完成后在同一个 dev SHA 完成总验收，提交 dev → main PR 并保持 OPEN，用户自行 review/merge。

## 3. 有序工作项

所有实现项交付真实用户路径和持久化/IPC/测试。`Sxx` 只用于清单交叉引用，GitHub 以完整标题命名。

| 顺序 | 工作项标题 | 真实前置 | 完整交付与退出证据 |
|---|---|---|---|
| S01 | 打开并恢复本地桌面工作区 | 无 | 可运行 Tauri 窗口、Rust workspace.open、SQLite 迁移/事务、版本化 command/result/outbox/snapshot/replay；关闭后重开同一工作区。导航可达，错误显式，无模型时如实显示不可用。真实 IPC + 临时磁盘重启验证。 |
| S02 | 安全连接并查看提供方账户 | S01 | Schema 驱动原生敏感字段输入、Keychain 引用、账户/环境隔离、connect/probe/permissions/disconnect、危险权限 gate、账户详情/手动刷新；逐提供方以官方契约确认 schema，凭据不落普通存储。 |
| S03 | 配置模型网关并完成五步入门 | S01、S02 | 固定 CLIProxyAPI、8317 冲突/认证/探测/重启、OAuth/DeepSeek 测试推理、用户默认模型与 fallback、配额恢复；真实可用路由才允许 Ready；风险配置不隐式启用 Live。 |
| S04 | 持久化并流式恢复 Codex Thread | S01、S03 | 固定 Codex App Server 协议，Thread/Turn/Item、取消/中断/重试、工具状态与 provider attempts、历史恢复；不可变开始快照与真实流式事件早于 turn 完成。 |
| S05 | 按模式与精确上下文限制 Agent 工具 | S02、S04 | Composer pickers、canonical context refs/hash、完整 capability matrix、Ask/Research 只读、Backtest 仿真、Trade 非 Live/Live 分流；受限 typed research MCP 与金融命令隔离；历史不读当前默认值。 |
| S06 | 核实并选定所需外部数据与授权 | S02、S05 | 查阅官方资料、保留来源/日期/能力/许可和真实探测，逐项解决 OD-001–006；建立缺数行为。需要付费/账户授权的选择交用户，未解决项不冒充完成或实盘就绪。 |
| S07 | 浏览真实行情并管理 Watchlists | S01、S06 | canonical instruments、Market Explorer/详情、搜索、watchlist CRUD、Census/Warm/Hot/Cold、默认 Hot≤20、DuckDB 1m+历史、source/双时间戳/venue/entitlement/freshness。输入身份贯穿详情/附加上下文。 |
| S08 | 校验可信时间、市场时段与公司行为 | S06、S07 | wall+monotonic+provider offset；sleep/clock jump 失信；equity 日历/半日/停牌/公司行为和历史调整。UI 展示阻塞原因；Live 权限统一消费可信结果。 |
| S09 | 汇总账户组合并保留 FX 估值证据 | S02、S06、S07、S08 | 原币/账户/工作区基础货币、positions/open orders/fills、FX source/path/quality/depeg；不把 USDT 当 USD，不可信转换禁止依赖该值的 Live risk。 |
| S10 | 用可溯源研究工具分析股票与现货 | S05–S09 | 真实 market/portfolio/news/filings/fundamentals 工具、来源/时间/限制和结构化 Research 结果，Prompt injection 无金融权限；工具结果实际影响最终输出，缺失/篡改负例可观测。 |
| S11 | 审阅并运行自然语言筛选条件 | S07、S10 | NL→可编辑 FilterSpec/RankSpec→审阅版本→DuckDB 降低候选集→按需获取→仅附加勾选标的；修改使解析/结果失效，完整空/失败/重试状态。 |
| S12 | 保存、检索和导出研究产物溯源 | S04、S10 | Artifact library/detail/export、Turn/Item/provider/tool/source/dataset/order refs；历史模型和上下文独立于当前选择器，导出脱敏且可验证内容来源。 |
| S13 | 编辑订单草稿并生成不可变 Proposal | S02、S07–S09 | 全量 order 字段、tagged quantity/TIF/environment、规范 decimal、规则校验、draft version→新 proposal ID/hash；所有 material edits/refresh 生成新身份，后续审批绑定该对象。 |
| S14 | 保存策略版本并运行受限策略进程 | S01、S07 | 策略列表/编辑/保存版本/hash、独立 worker、受限文件/网络/Keychain/Gateway、只输出 signal；非法访问通过真实进程测试被拒绝。 |
| S15 | 配置并比较可复现历史回测 | S08、S14 | Composer 与 Editor 汇入同一配置；冻结策略/数据/hash/date/cost/slippage/seed；queued/running/cancelled/failed/completed，完整指标/曲线/trades/manifest，两次完成结果比较；无 lookahead，模型故障不停止确定性运行。 |
| S16 | 完成本地模拟交易与持仓更新 | S09、S13 | Local Paper 专属模拟器、标记 TradeX simulation、订单/fill/现金/持仓持久化与查询，永不穿过 Live Gateway；不把模拟结果称为 broker truth。 |
| S17 | 完成 Alpaca Paper 交易生命周期 | S02、S13、S16 | 官方 Paper endpoints/account/capability/symbol/order/cancel/query/private stream 适配，真实 sandbox 订单状态；不实现未授权的 Alpaca Live 范围。 |
| S18 | 完成 Trading 212 Demo 交易生命周期 | S02、S13、S16 | Provider-specific 认证、数量/TIF/取消/查询/限流/订单身份与权限能力，Demo 环境不可变；以真实官方支持能力验证，ACK 与 fill 分离。 |
| S19 | 完成 Binance Spot Testnet 交易生命周期 | S02、S13、S16 | Spot rules、签名/time offset、BASE/QUOTE、order/client identity、private stream/reconciliation，明确 Testnet；无 margin/futures/withdrawal 路径。 |
| S20 | 完成 Bitget Spot Demo 交易生命周期 | S02、S13、S16 | API key/secret/passphrase schema、官方 Demo 可用性与 headers/capabilities、query/cancel/fills/error/stream；不伪造不存在的 Demo API 支持。 |
| S21 | 配置并执行确定性风险政策 | S08、S09、S13 | 全部 PRD §21 风险字段和硬规则；无用户政策 Live disabled、市价单默认 off；policy version/多账户范围与失效原因；Agent 无修改权限。 |
| S22 | 显式 Arm 并批准精确金融意图 | S21 | 账户独立 arming/20 分钟超时；独立 PLACE/CANCEL approval 类型、单次/nonce/TTL/hash/版本；全量行情与最大授权支出展示，审批前与消费时校验；泛化 Codex approval 永不转成金融权限。 |
| S23 | 原子消费审批并预留账户容量 | S22 | SQLite immediate + per-account serialization、单次消费/幂等约束、现金/敞口/open orders/并发 Thread 容量；政策保存/消费竞态；实际冲突、回滚和释放金额证据。 |
| S24 | 经认证的独立 Gateway 派发执行 | S23 | 固定子进程、继承双向通道/协议/会话认证、狭窄 ID 请求、权威重载、一次性 grant、持久化 SUBMITTING 后 I/O；disarm 与 dispatch 串行、失败不重放变更；进程隔离与边界故障测试。 |
| S25 | 对账未知提交并核验人工处置证据 | S24、S17–S20 | canonical 全状态、broker truth 优先、unknown freeze/query-first/5 分钟后仍冻；backend evidence scope/window/pagination/identity/允许决策；状态版本防并发成交覆盖，处置后健康复核且仍 disarmed。 |
| S26 | 刷新并审批撤单与处理成交竞态 | S22、S24、S25 | 精确不可变 CANCEL intent、Arm 后回到撤单且刷新、不同撤单资格、approval/reject/expire、CANCEL_PENDING→provider truth；ACK 不释放，部分成交/费用/剩余容量恰好一次处理；修改仅确认撤单后新 proposal。 |
| S27 | 恢复崩溃、休眠、断流并中断未派发工作 | S24–S26 | 全 Live restart/sleep/lock/Disable All disarm，account fault 指定作用域、共享政策全部绑定账户；未知保留；P0/P1 优先于 research/backtest，bounded queues；模型故障下监控/撤单/对账仍有效。 |
| S28 | 验证 Trading 212 Live 可信执行 | S06、S08、S18、S21–S27 | Live adapter 与 trusted chain 贯通、最新 entitlement/calendar/账户能力门槛、真实查询与明确用户交易授权的验收；没有实际授权不发 Live order，保留该验收阻塞。 |
| S29 | 验证 Binance Spot Live 可信执行 | S06、S08、S19、S21–S27 | Live spot 单独连接、permissions/rules/time/FX、受信下单/撤单/对账全链，account-specific readiness；contract/fault 与授权外部验收分别记录。 |
| S30 | 验证 Bitget Spot Live 可信执行 | S06、S08、S20、S21–S27 | Live/Demo 绝不混路由，签名/身份/权限/成交/撤单证据与 trusted authority；只有该 provider 完整 gate 通过才开放对应能力。 |
| S31 | 验证工作区备份、导入、迁移与保留 | S12、S15、S25、S27 | 非秘密 export manifest、路径/归档/schema 校验、恢复前备份、事务迁移/integrity、Keychain 引用检查、恢复全 Live disarmed/reconcile；未解决金融证据不可自动清理。 |
| S32 | 完成设置、健康诊断与隐私控制 | S03、S27、S31 | Providers/Models/Risk/Data/Health/Appearance/About、版本来源/配额/日志/诊断、auto-update/crash-report 明确界面与 opt-in、默认无 telemetry；敏感字段序列化前脱敏。 |
| S33 | 完成全页面、键盘与窄屏回归 | S01–S32 | 所有页面与 QA-01–12 在实际应用重跑；native dialog 安全焦点/trap/inert/return、Enter 无隐式金融审批、semantic controls、screen reader、reduced motion；桌面/768/390 长内容保留导航与金融操作。基础可访问性从每项实现时即要求，本项负责完整交叉回归。 |
| S34 | 测量资源目标并构建签名桌面包 | S27、S31–S33 | IPC arrival→UI commit p95<100ms、重启→对账启动<5s、负载隔离/增长/Hot 上限；固定 runtime/schema/provider/engine 版本，可重复 macOS packaging/sign/notarization（按平台要求），缺少签名身份不能声称签名通过。 |
| S35 | 在同一 dev SHA 完成总验收并提交 PR | S01–S34 | 清单逐条绑定实现与行为证据；所有 provider gates、故障/安全/视觉/持久化/CI 达标，map/spec/task 层级无遗漏；dev→main PR 按仓库模板提交，保持 OPEN。 |

## 4. 开放决策的处理

| 决策 | 当前处理 | 必须取得的证据/后续动作 |
|---|---|---|
| OD-001 实时美股 | 尚未选择；不能宣称 Live equity ready | S06 比较官方数据权限、市场覆盖、许可/保留与价格；用户授权所需订阅，S07/S28 验证真实 snapshot entitlement。 |
| OD-002 历史数据 | 受影响 backtest 不可用 | S06 验证可下载范围、调整/时区/缺口与许可；S15 验证真实可复现数据。 |
| OD-003 基本面、OD-004 新闻/filings | 源专属工具缺数时 disabled | S06 选择合法官方来源与接口；S10 真实调用，不能用 fixture 结果冒充研究。 |
| OD-005 日历/公司行为 | 相关股票 Live blocked | S06/S08 覆盖 holiday/half-day/halt/split/dividend/symbol/delist 与更新频率。 |
| OD-006 FX | 依赖跨币值的 Live risk blocked | S06/S09 选择 source/path/timestamp/freshness/quality；stablecoin depeg 单独判断。 |
| OD-007 图表 | 按 PRD 可用最小内部图表 | S07/S15 优先 SVG/CSS，提供数据表/键盘可达；无需先引入商业图表库。 |
| OD-008 回测运行时 | 按 PRD 默认隔离 Python | S14 验证真实 OS 限制；若平台限制不满足则记录并解决，不能以 subprocess 名称代替沙箱。 |
| OD-010 数据库加密 | 依赖 OS 磁盘保护，不声称数据库加密 | Settings/diagnostics 如实说明；备份不包含 secrets。 |
| OD-011/012/013 | 用户政策前 Live disabled、市价单 off、20 分钟 inactivity | S21/S22 实现明确配置与超时；不得替用户选择风险额度。 |
| OD-014/016 | macOS first、English UI 且字符串可本地化 | S01/S33/S34 验证；中文规范配对独立于产品 UI 语言。 |
| 运行时/API 版本 | 未固定，必须当前资料验证 | 在负责工作项查询官方源与当前代码，保存版本/hash/compatibility 结果，禁止沿用历史仓库记忆中的旧运行时实现。 |
| 外部验收资源 | 尚未确认账户、数据订阅和 signing identity | 先完成可审阅实现、contract/fault 测试和具体验收步骤；缺少资源时明确请求必要输入。构建软件的授权不等于授权真实资金交易。 |

## 5. 统一验证边界（已用于 S01 Spec / implementation）

仓库尚无测试设施。默认最高公共边界为**桌面 UI → 真实版本化 Rust command dispatcher → 临时真实 SQLite/文件 → durable event → UI projection**。共用同一 command schema；renderer 的 mock transport 仅做组件错误/焦点/布局测试，不作为后端或金融权限证明。

| 证据类别 | 证明对象 | 最小反例 |
|---|---|---|
| IPC/持久化集成 | 实际命令、错误、版本、原子状态/outbox、重开/订阅/重放 | 不支持 schema、不合法 payload、陈旧版本不修改；event gap/conflict/compaction 不推进错误权限。 |
| 真实进程/安全边界 | Codex/CLIProxy/Gateway/strategy 生命周期与隔离 | 无 Gateway handle/secret 的进程无法获得权限；模型故障继续对账；错误协议/会话拒绝。 |
| Provider contract/fault | 每个环境真实序列化/解析/身份与失败语义 | ACK≠fill、空查询≠未提交、unknown 不重试释放、cancel/fill/disable/policy 竞态。 |
| 官方环境验收 | 当前 provider 实际可用性/permissions/数据授权/状态 | 真实 sandbox/demo/testnet 的 place/query/cancel；Live 使用用户批准的具体交易验收，不能由 mock 替代。 |
| 浏览器/桌面交互 | 用户动作、状态、可访问性、视觉 | QA-01–12；选择器不改历史、无隐式 Enter approve、全部页面窄屏可达。Tauri 原生行为另作实际桌面验证。 |
| Release/资源测量 | p95、恢复时限、包版本/签名、秘密零泄漏 | 代表性负载与故障数据；缺采样、跳过 credentialed gate 或缺签名都不能标记总验收通过。 |

每个非平凡 parser/状态分支/金融与安全路径保留能失败的行为检查。使用 Rust 自带 test、已需的 UI 测试工具和真实临时 DB；不为简单样式编辑单独制造镜像测试。完整测试在每项收尾运行一次；有新增修改/失败才重复扩大验证。

## 6. 完成判定与交接

需求表初始 `NOT_STARTED`；计划映射通过不是功能通过。每项验证后补 evidence 引用与 SHA，再更新为 `VERIFIED`。外部能力未证实使用 `BLOCKED_EXTERNAL`，已有代码无所需证据使用 `IMPLEMENTED_UNVERIFIED`；这两种状态都不能让总 map 关闭。

非 v1.0 的三条 FR 保留 `DEFERRED`，不能从总表删除。产品原型覆盖矩阵继续描述其自身基线，应用证据单独记录；只有重新验证原型自身时才能更新原型状态。

最终 PR 包含 Summary、Scope of Changes、Technical Details、Risk Assessment、Breaking Changes、Testing、Security Impact、Performance Impact、Checklist。创建前重新读取 dev SHA、完整 required CI、清单、map/spec/task 状态与 signed package 证据。PR 保持 OPEN，不自动 merge。
