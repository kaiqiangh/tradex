# S18 Trading 212 Demo 交易生命周期

日期：2026-09-23。实现分支：`dev`。规范基线：`8e5992c0546c57e984c6f84288fca5054cb4550d`。

Wayfinder 工作项：[S18 Spec #60](https://github.com/kaiqiangh/tradex/issues/60)。

## Problem Statement

S02 已可安全连接 Trading 212 Demo 并读取账户、持仓和开放订单；S13 已建立 OrderDraft / 不可变 OrderProposal；S16 已交付独立 Local Paper 模拟。但 TradeX 尚不能把被用户检查的 Proposal 安全提交到 Trading 212 Demo，也不能完整查询其订单状态、观察累计成交、确认撤单结果或把更新后的 provider 状态展示给用户。因此 FR-016、FR-033、FR-060、AC-010、AC-046 和 UX-004 中属于 Trading 212 Demo 的部分仍未完成。

Trading 212 Public API v0 的订单提交端点明确标注为非幂等，且 Demo 与 Live 使用不同 host。将超时直接作为失败重试可能创建重复订单；把 HTTP 接受响应当成成交或取消完成也会误导用户。TradeX 必须把 Demo 身份、provider acknowledgement、成交摘要和后续对账分开表达。

## Solution

在现有 AccountConnection、OrderDraft / OrderProposal、Control Plane、SQLite 与订单交互组件上增加仅面向 `trading212` / `DEMO` 的安全订单生命周期：用户通过 TradeX 主界面明确确认受支持 Proposal；后端固定路由到 Demo host，以精确数量和受支持字段发送；持久化尝试和 provider order identity；用受限 REST 查询观察状态；支持刷新、显式确认撤单和撤单/成交竞态处理。Demo 永不回退到 Live。官方 API 未提供 TradeX client order id，也未在当前 v0 订单参考中列出私有订单流，因此未知提交不重发，成交以 provider 返回的累计数量/金额摘要表示，状态更新通过受限 REST 查询完成。

## User Stories

1. 作为 TradeX 用户，我希望从已连接且已确认的 Trading 212 Demo 账户选择订单上下文，以便知道 Proposal 会提交到哪个模拟账户。
2. 作为 Demo 用户，我希望只提交绑定到当前 workspace、Demo connection、规范股票 instrument 和当前不可变 Proposal 的订单，以便账户、方向、数量和价格不能被 UI 或 payload 偷换。
3. 作为 Demo 用户，我希望下单前看到明确的 “Trading 212 Demo” 标识、股票、买卖方向、数量、订单类型、限价（如适用）与有效期，并通过单独的确认动作提交，以便清楚地区分 Proposal 生成与 provider 写操作。
4. 作为 Demo 用户，我希望系统仅允许 API 明确支持且 TradeX 当前订单模型可表达的数量型 Market / Limit 组合，以便报价金额、IOC/FOK、Stop 等不受支持输入不能被静默改写。
5. 作为卖出股票的 Demo 用户，我希望界面仍用正数量和明确的 Sell 方向，而 provider adapter 按 Trading 212 契约编码为负数量，以便避免方向符号错误。
6. 作为使用小数股数量的 Demo 用户，我希望 TradeX 精确保留数量、限价、累计成交数量和成交金额，而不经二进制浮点舍入，以便 broker observations 与 Proposal 一致。
7. 作为 Demo 用户，我希望收到 provider order ID 和“已接受/处理中”状态时看到它仍不等于成交，以便只有 provider 状态和累计成交观察能推进成交展示。
8. 作为 Demo 用户，我希望查看由 TradeX 创建及在该账户中由其他客户端创建的当前订单，并区分来源，以便订单簿忠实呈现账户状态。
9. 作为 Demo 用户，我希望刷新订单状态和历史订单，在 provider 支持时看到开放、部分成交、已成交、拒绝、取消和到期结果，以便核对实际模拟账户，而不是只看本地意图。
10. 作为 Demo 用户，我希望部分成交展示 provider 报告的累计 filled quantity/value 与剩余数量（可确定时），以便不会把一次快照误当成逐笔 fill。
11. 作为 Demo 用户，我希望只在账户身份、provider order ID 和当前开放状态经刷新确认后，才对确切订单发起显式撤单确认，以便撤单不会作用于另一账户、环境或已结束订单。
12. 作为 Demo 用户，我希望 Trading 212 接受撤单请求后看到 `CANCEL_PENDING`，并在查询观察到终态后才看到 `CANCELLED`；如果期间成交，我希望成交事实保留并优先显示。
13. 作为 Demo 用户，我希望提交或撤单发生超时、断线、429 或响应无法核验时看到未知/对账中状态，且 TradeX 不自动重复写请求，以便避免重复订单或重复撤单。
14. 作为 Demo 用户，我希望刷新控件遵守 provider 每账户速率限制，并显示下一次可用刷新时间和可恢复错误，以便使用不触发持续限流。
15. 作为多账户用户，我希望订单、尝试、provider order ID 和查询始终绑定 workspace、Demo connection、远端账户与 `DEMO` 环境，以便任何账户状态都不会跨账户或流入 Live。
16. 作为键盘、读屏或窄屏用户，我希望加载、空、过期、部分成交、拒绝、未知、限流和撤单确认状态均有清晰文字、焦点管理和可用操作，以便不用依赖颜色、鼠标或宽屏。
17. 作为 Local Paper 或 Live 用户，我希望 S18 不改变 Local Paper 仿真语义，也不会从 Trading 212 Demo 自动切换至 Trading 212 Live，以便不同执行环境保持不可变且可辨认。

## Implementation Decisions

- **Canonical shared wire contract:** Backend ARD §§41.21 and 42 own `trading212.demo.order.submit`, `trading212.demo.order.attempt.get`, `Trading212DemoOrderAttempt`, and `trading212.demo.order.attempt.changed`; UI behavior is paired with UI Spec §14.5 and Frontend ARD §13.10. Keep generated TypeScript schema aligned with the Rust contract.
- S18 复用已连接的 `providerId=trading212`、`environment=DEMO`、该连接自己的 OS Keychain 凭据、S13 的不可变 OrderProposal、workspace/session versioning、Rust Control Plane、SQLite 持久化和现有订单 UI 模式。renderer 不获得凭据、HTTP host、远端账户 ID、provider ticker 或 authority flags 的控制权。
- 所有 S18 provider I/O 固定为 `https://demo.trading212.com`，使用现有 API key + secret 的 HTTP Basic 凭据，设置 sensitive header 并在序列化前清理临时凭据。不得跟随重定向、接受调用方 URL、访问 Live host、因 Demo 错误回退到 Live，或把秘密、Authorization header、原始敏感响应写入 SQLite、事件或日志。
- 只有已连接且通过既有权限审阅的明确 Demo connection 可以进入 S18。provider 不公开完整 key scope/account subtype 时继续显示 `UNVERIFIED`；读取成功或单笔下单成功都不升级为 scope 已验证。Live `arming` 保持不适用；本切片不调用 Live Gateway、财务审批、reservation 或 Live authority。
- Proposal 必须再次由后端核对 workspace、connection、Demo 环境、远端账户身份、Proposal id/hash/state、账户与 instrument、下单字段、当前连接和 state version。每个写操作仅能由 TradeX 主 UI 明确用户动作启动；Agent、模型、research tools、strategy worker 和通用 broker tool 均不能提交或撤单。
- S18 只开放当前 TradeX `OrderDraft` 可表达的 Market 与 Limit 股票订单，且只接受 `BASE` 正数量。Buy 序列化为正数量，Sell 序列化为负数量。API 当前不支持按金额下单，因此 `QUOTE` 一律在 backend 拒绝；不支持的字段/组合给出可操作原因，不进行单位、TIF 或 order type 的隐式改写。
- Limit 的受支持有效期为 Trading 212 `DAY` 和 `GOOD_TILL_CANCEL`，映射自 TradeX `DAY` / `GTC`。Market 端点没有可由 TradeX 指定有效期的 request 字段，因此 S18 只接受 TradeX `DAY` Market，拒绝 `GTC` Market，固定发送 `extendedHours=false` 并在确认界面明确显示；还必须核对 provider 响应确认的有效期为 `DAY`，否则进入未知结果对账且不重发。`IOC`、`FOK`、Stop、Stop-Limit、extended-hours 开启选项和金额订单均不进入 S18。
- Provider ticker 必须来自 backend 管理的 canonical instrument/provider mapping，并按需要核对 Trading 212 instrument metadata；客户端不得直接传 ticker。instrument / account / order IDs 作为不透明字符串传递，int64 不得经过 JavaScript Number 舍入。数量、价格和累计成交字段使用精确 decimal string/wire conversion，不经 `f64`。
- Trading 212 当前 API 无官方幂等 client-order key；本地生成的 attempt/idempotency identity 只用于 TradeX 的持久化去重，绝不能声称 provider 幂等。写入 `SUBMITTING` attempt 后才发单。POST 明确拒绝可作为 `REJECTED`；网络错误、超时、未识别响应、不可核验身份或进程在响应前退出进入 `UNKNOWN_RECONCILING`。绝不自动重发。按可用 open/history 查询搜寻订单；相似行只作为候选证据展示。因 provider 不返回 TradeX client identity，候选不自动绑定到 attempt 或作为 ACK；零个/多个候选、一次空查询、分页不完整或身份不一致都保留未知并冻结该账户相关新提交，等待更强 provider 证据或未来的 S25 authority resolution。
- 成功 acknowledgement 只表示 provider 接收/返回了订单记录；映射后的 provider status 单独保留。`PARTIALLY_FILLED` / `FILLED` 必须有 provider status 或精确的累计 filled quantity/value 证据。S18 把这些当作累计 order fill summary；没有独立 execution ID、逐笔价格与时间时不合成 fill event。订单与 account/position observation 分开持久化并标注 provider time、TradeX observation time、来源和新鲜度。
- 订单簿支持当前 pending orders、按已知 provider order ID 查询，以及官方 paginated historical orders。分页最多使用 API `nextPagePath` 的 path/query，逐页验证固定 `/api/v0/equity/history/orders` 路由、游标前进和页数/行数/字节上限；重复游标、重复/冲突身份、未知状态、畸形数值或不完整读取标记 `DEGRADED`，不覆盖既有可信快照。Provider-returned order identity 作为账户内身份使用；不得依赖订单 ID 全局唯一。
- 状态归一到现有 normalized order lifecycle where possible，同时保留原始 provider status。ACKNOWLEDGED/ACCEPTED 不等于成交；部分成交状态保留累计 qty/value；CANCEL_PENDING 不是已撤销；终态只能来自 provider 观察。`LOCAL`、`UNCONFIRMED`、`CONFIRMED`、`NEW`、`CANCELLING`、`CANCELLED`、`PARTIALLY_FILLED`、`FILLED`、`REJECTED`、`REPLACING`、`REPLACED` 及届时官方明确的终态按契约映射；未知状态作为未知并阻断依赖该状态的写操作。
- 撤单前必须针对同一 Demo connection/provider order ID 刷新当前 pending order，检查 identity/status/filled/remaining/observed time，并让用户对精确账户和订单另行确认；详情超过 60 秒、账户/连接/订单簿版本变化、未知/终态状态或订单不再 pending 时拒绝。Backend 在 SQLite immediate transaction 中先持久化 `SUBMITTING`、本地幂等键与事件，再二次核验账户身份并最多调用一次固定 Demo DELETE 路由；本地撤单间隔为每账户 2 秒并展示 `cancelOrderRetryAt`。HTTP 200 仅映射为本地撤单 `PENDING`（UI 显示 `CANCEL_PENDING` 语义），不宣称已撤销；400/401/403/429 为有界明确失败并清理可重试状态；超时/transport/408/未识别响应和重开恢复均保持 pending/unknown 且不重发。重复命令不会重复 DELETE。随后只由显式订单详情/历史观察收敛；部分成交更新累计成交同时保留撤单 pending，provider `CANCELLED`、`FILLED`、`REJECTED`、`REPLACED` 或 `EXPIRED` 终态清除本地 pending，任何成交竞态以 provider 事实为准。禁止隐式 replace/new order 和后台轮询。
- 当前 Trading 212 v0 Order reference 提供 REST 查询，没有在其订单操作中声明私有 stream；此处据当前公开 reference 作保守推断，不新增 stream worker。订单状态通过用户请求或单次安全流程触发的 REST refresh 更新，不做高频后台轮询。鉴于列表、详情、历史、account 和 order-write 端点具有不同且按账户计算的额度，backend 负责实施每 connection/account + endpoint 的请求间隔/窗口、解析可用 `x-ratelimit-*` 响应头、遵守 reset、显示 `retryAfter`；429 和到限状态不自动重试写操作，也不清空最近可信快照。
- 成交观察后按 API 限制刷新账户/持仓；不可用、限流或不完整数据明确标为 stale/unavailable，不能把已接收订单、累计成交摘要或 fixture 投影直接当成最新账户余额。Provider observations 和 TradeX-origin attempts 保持来源/时间可追踪。
- UI 始终使用文字 `Trading 212 Demo` / `TRADING212_DEMO` 标识环境，清楚区分 Local Paper、Alpaca Paper 与 Live。显示 `origin`、provider order ID、原始/归一状态、累计 fill 摘要、剩余数量（仅在数据可确定时）、观测时间、新鲜度、错误与下一步；unknown/degraded/rejected 明确使用非成功文字与状态提示。布局和焦点行为遵循现有 Trade / Proposal / confirmation pattern。
- 不改变 S02 的 Live 只读连接行为，不实现 S28 Trading 212 Live，不接触 S19 Binance Testnet / S20 Bitget Demo，不增加新订单种类、不实现 amend/replace/自动策略执行、逐笔成交流、CSV export 或新的通用 provider framework。若 S19/S20 展示了真实重复结构，由对应规格决定后续共享重构。

## Testing Decisions

- **固定测试边界：**从 React Trade surface 经真实 versioned Rust IPC、真实临时 SQLite/outbox 与 provider job 到 T212 HTTP fixture，再观察 React projection。纯 parser/mock component 不替代此边界；已有最高行为 seam 优先，不另建服务或绕过 Control Plane。
- T212 provider contract/fault test 覆盖固定 Demo host 与 verb/path allowlist；Demo/Live 不串路由；HTTP Basic 敏感头；Market/Limit 精确 request body（买/卖符号、Decimal、大 int64 IDs、Limit DAY/GTC、Market DAY only）；unsupported Market GTC/Quote/IOC/FOK/order types；account/provider/ticker/order identity mismatch；200 acknowledgement 与 fill 分离；401/403、4xx 拒绝、408/429、redirect、timeout、超大/坏 JSON、未知状态、限流 reset、秘密扫描与分页重复/截断。
- Rust dispatcher/SQLite 测试覆盖 connected Demo account、跨 workspace/connection/环境/Proposal/hash/state-version 注入、写前持久化 attempt、重复 IPC 不触发第二次 POST、POST timeout→unknown/no retry、空/多候选无法释放 unknown、重开 workspace 后恢复、对账合并 provider-origin/external orders、分页不完整保留快照、account-level throttle 和不变的 Local Paper/Live guard。
- 订单与撤单负例覆盖：用户未确认或确认过期；非 pending/未知状态；远端账户变化；订单在确认前被部分/全部成交；撤单 DELETE 已接受但响应超时；查询返回 CANCELLING、CANCELLED、PARTIALLY_FILLED、FILLED、REJECTED/EXPIRED 或未知；每次只发一个 DELETE，恢复靠查询而非重发；状态和剩余数量按 provider truth 更新。
- Rust-backed responsive browser 验证 1280/768/390 窗口的 Draft/Proposal、Demo order list/history、acknowledgement、partial/full cumulative fills、unknown/degraded/rate limited、refresh disabled/retry countdown、cancel review/confirmation/fill race、键盘焦点/读屏标签/文字环境状态。复用现有浏览器 helper/fixture contract，避免只凭截图、静态 prototype 或 renderer mocks 宣称通过。
- macOS 原生 Keychain 运行时验证使用当前已连接 Demo credential reference 和 disposable fixture/test setup，不显示或记录真实 key；取消、失败、workspace reload 和隔离清理行为按 S02 契约。真实 provider API 写操作是单独外部验收：仅在明确展示一个测试 Proposal 和 Demo account 且用户在 TradeX UI 明确确认后执行；之后 query、观察状态/累计 fill、必要时取消仍开放订单并 verify。不得调用 Live host。若账户凭据、API scope、instrument 支持或用户确认不可用，记录 `BLOCKED_EXTERNAL` / `IMPLEMENTED_UNVERIFIED`，fixture 不能冒充真实 Demo 验收。
- 完成相关 schema/IPC contract generation checks、`cargo fmt --check`、Rust `check` / `clippy` / 相关 tests、TypeScript typecheck/build/UI tests、provider HTTPS boundary、requirements traceability、Rust-backed responsive browser tests、`git diff --check` 和串行 Standards→Spec review；保存起始 SHA、最终 SHA、实际命令和所有限制。

## Out of Scope

- Trading 212 Live 下单、Live credential、arming、financial approval、risk reservation、Privileged Live Order Gateway 和 S28 Live trusted execution。
- S17 Alpaca Paper sandbox gate（按用户指示跳过，保持 #56 开放）、S19 Binance Spot Testnet、S20 Bitget Spot Demo。
- T212 Stop / Stop-Limit（虽然官方 reference 列出，但当前 TradeX Draft/Proposal contract 不表达），Quote/notional order、IOC/FOK、broker amend/replace、订单策略自动执行、跨提供方 fallback、后台高频 polling、未被当前官方 reference 说明的 private stream。
- 以 API keys 无法检查的 scope 推断出读取/下单全部权限，或改变 Trading 212 account subtype、multi-currency 与 provider limitations 的既有 UNVERIFIED 状态。
- 将累计 order fill summary 扩写为不存在的逐笔 execution、推测 price/time、或以 TradeX simulated state 覆盖真实 provider observations。
- S33 全应用视觉/键盘/读屏完整回归、S34 签名安装包，以及 S21–S27 cross-provider risk/authority/reconciliation governance。

## Further Notes

本规范只把下列追溯项中 Trading 212 Demo 所属证据推进到可实现边界：FR-016、FR-033、FR-060、AC-010、AC-046、UX-004。跨 provider 或跨阶段需求须待所有拥有者切片通过后才能升级为 `VERIFIED`。规范本身与 fixtures 不构成实现或真实 broker 验收证据。

当前官方资料（核对日期 2026-09-23）：

- [Trading 212 Public API v0](https://docs.trading212.com/api) — Demo `https://demo.trading212.com/api/v0`、Live 独立 host、账户类型与订单限制。
- [Authentication / Basic header](https://docs.trading212.com/api/section/authentication/building-the-authorization-header) — API key + secret 的 Basic auth。
- [Market order](https://docs.trading212.com/api/orders/placemarketorder) — signed quantity、Market request fields、非幂等说明和 endpoint limit。
- [Limit order](https://docs.trading212.com/api/orders/placelimitorder) — signed quantity、DAY/GTC、非幂等说明和 endpoint limit。
- [Pending orders, by-ID query and cancellation](https://docs.trading212.com/api/orders/orders) — open order query、详情、取消 race / accepted semantics 和速率限制。
- [Historical orders](https://docs.trading212.com/api/historical-events/orders_1) 与 [cursor pagination](https://docs.trading212.com/api/section/pagination) — order history 与有界 cursor behavior。
- [Rate-limit headers](https://docs.trading212.com/api/section/rate-limiting/response-headers) — per-account limits 与 reset metadata。
- [Instrument metadata](https://docs.trading212.com/api/instruments/instruments) — provider ticker、instrument identity 与 metadata。
