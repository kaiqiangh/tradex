# S17 完成 Alpaca Paper 交易生命周期

日期：2026-09-22

前置：S02 提供方连接、S13 OrderDraft/OrderProposal、S16 Local Paper

状态：S17 规范阶段；实现与验收尚未开始。S17 仅覆盖 Alpaca Paper；S18–S20、S21+ financial authority 和 S33 全量回归仍属于后续工作项。

## Problem Statement

TradeX 已能安全连接 Alpaca Paper 并读取账户、持仓和未完成订单，也能生成不可变 OrderProposal 和执行 Local Paper simulation，但用户尚不能把已审阅的 Proposal 提交给 Alpaca Paper、核对成交或撤销未完成订单。当前账户投影也没有显示 Alpaca 的 buying power 或 fills。FR-015、FR-033 和 AC-009 因此尚未完成。

如果用户只能看到 Local Paper 结果，便无法确认券商 Paper 环境实际接受或拒绝了什么；若把本地模拟状态展示成 broker 状态，或把 Paper 与 Live 路由混用，又会造成错误的账户和订单认知。

## Solution

在现有 TradeX 账户、OrderDraft/OrderProposal、Portfolio 和 Control Plane 边界上补齐 Alpaca Paper 的可验证生命周期：

- 复用用户已经确认的 `providerId=alpaca`、`environment=PAPER` 连接和 Keychain 凭据；展示真实 buying power、持仓、开放订单、订单历史和 fills。
- 将 `ALPACA_PAPER` 的不可变 Proposal 绑定到一个精确的、已连接的 Paper account 和规范 instrument。调用 Paper 专属资产能力查询，确认股票可交易以及订单字段组合受支持；缺数据或不支持时明确阻止提交并说明原因。
- 只有用户在 Trade surface 对当前 Proposal 明确确认后，Control Plane 才提交订单。界面显示 Paper 环境、账户、股票、方向、数量/金额、订单类型、限价和有效期。模型、Agent 和研究工具不能直接提交或撤单。
- 对每个请求先持久化 TradeX attempt 和稳定 client order identity，再访问 Alpaca Paper Trading API。记录 provider order identity 和规范状态。超时或连接中断可能发生在 provider 已接单之后；此时进入 `UNKNOWN_RECONCILING`，按 client order ID 先查询，绝不自动重发 POST，也不把空的一次查询当成未提交证明。
- 提供按账户查看/刷新订单和 fills、显式确认撤单、接收 Alpaca `trade_updates` 私有流，以及断线/重启后的 REST 对账。流事件是低延迟信号；provider REST 状态和活动记录用于修复遗漏。HTTP acknowledgement 表示 provider 接收了请求，不等于成交；HTTP 接受撤单也不等于订单已经取消。
- 在账户和所有订单/成交 surfaces 保留 `ALPACA_PAPER` 环境标识及 Paper 模拟限制说明。Paper 使用 Alpaca 固定 Paper endpoint 和单独凭据；Live endpoint、Live credentials、Live Gateway、arming、TradeX financial approval 和 reservation 均不进入 S17。

## User Stories

1. 作为 TradeX 用户，我希望选择已有 Alpaca Paper account，而不用重新输入 API key，以便在正确账户上继续操作。
2. 作为 Paper 用户，我希望账户页显示 Alpaca 报告的 cash、equity、buying power 和币种，以便知道订单依赖的真实 Paper 账户观测值；缺字段时显示 unavailable，不推算成 cash。
3. 作为 Paper 用户，我希望账户页和 Portfolio 能查看规范 instrument 对应的持仓、未完成订单和成交记录，并看到 provider 与 TradeX 收到时间，以便区分当前 broker 状态和历史状态。
4. 作为 Paper 用户，我希望 TradeX 用 canonical instrument 与 Alpaca 资产能力查询核对股票代码、资产类别、active/tradable 状态及 fractionable 能力，以便避免把股票代码误路由到其它资产或环境。
5. 作为 Paper 用户，我希望在 OrderDraft 中看到当前 Alpaca Paper account 的可用订单能力，并在不支持的 order type、TIF、数量语义或 asset 上看到明确原因，以便不能通过绕过 UI 的 payload 创建不支持的订单。
6. 作为 Paper 用户，我希望从与账户、环境、股票、数量/金额和 order details 绑定的不可变 Proposal 发起提交，以便发送的内容与我审阅的一致。
7. 作为 Paper 用户，我希望在发送前看到明确的 Alpaca Paper 提交确认和模拟环境披露，以便把提交意图与生成 Proposal、Local Paper simulation 和 Live financial approval 区分开。
8. 作为 Paper 用户，我希望 provider 拒绝、账户不可交易、购买力不足、资产不可交易、限流和认证失败都有准确且可恢复的提示，以便修正输入或账户状态。
9. 作为 Paper 用户，我希望网络超时后看到“状态未知/正在核对”，TradeX 先按稳定 client order ID 查询而不是再发一笔订单，以便避免重复 Paper 订单。
10. 作为 Paper 用户，我希望只在收到 Alpaca acknowledgement 后看到 broker-accepted 状态，并在收到 fill evidence 后看到成交，以便 acknowledgement 不会被误认为成交。
11. 作为 Paper 用户，我希望 TradeX 的订单列表包含从该 Alpaca account 读取到的未完成订单，即使订单最初不是由 TradeX 创建，以便完整查看 Paper account 当前状态。
12. 作为 Paper 用户，我希望撤单前刷新选中订单、查看当前 filled/remaining quantity 并显式确认，以便撤销确切的 broker order。
13. 作为 Paper 用户，我希望撤单请求接受后看到 `CANCEL_PENDING`，并在 broker 确认后才看到 `CANCELLED`；如期间发生 fill 或 provider 拒绝撤单，我希望看到该事实而不丢失成交。
14. 作为 Paper 用户，我希望实时看到 Alpaca `trade_updates` 中的新单、部分成交、完全成交、拒绝、撤单、过期及其它已支持状态，以便无需手工刷新才能观察生命周期。
15. 作为 Paper 用户，我希望私有流断开时健康状态转为 degraded，TradeX 在重连后以 REST 状态和 fill activities 对账，以便不把断流期间的状态当成最新事实。
16. 作为 Paper 用户，我希望关闭并重开 workspace 后，TradeX 仍能读取本地 attempt、broker order ID、fills 与 unresolved state，并重新查询 provider，以便恢复过程中不会重发订单。
17. 作为多账户用户，我希望每一笔订单和 stream 都绑定准确的 workspace、connection、remote account 和 PAPER environment，以便一个 Paper account 的数据不会串到另一账户或 Live。
18. 作为使用键盘或窄屏的用户，我希望确认弹窗、错误、loading、empty、stale 和 retry 状态可访问，且状态同时使用文字与标识，以便无需依赖颜色或鼠标理解和操作订单。
19. 作为 Paper 用户，我希望看到 Paper Trading 是模拟环境且成交结果不能代表 Live 表现，以便不把 simulated fills 当作真实市场执行保证。

## Implementation Decisions

- 继续使用既有 workspace、AccountConnection、OrderDraft/OrderProposal、canonical instrument、Portfolio 和 versioned Control Plane。S17 不建立 renderer-owned order model，也不把 `paper.*` Local Paper commands 改作 provider orders。
- 所有操作固定到 `alpaca` / `PAPER` connection 和 `https://paper-api.alpaca.markets`；客户端不能提交 HTTP host、provider symbol、credentials、remote account identity 或 authority flags。Alpaca Paper 以外的环境、其它 provider 和所有 Live endpoint 在本切片拒绝。
- 只接受当前 OrderDraft 已表达且 Alpaca 支持的 US equity Market/Limit order。`BASE` quantity 映射为 `qty`；`QUOTE` 映射为 `notional`，并只在 Alpaca 支持的 market/day 组合启用。限价字段、fractional eligibility 和其余 quantity/TIF 组合均由后端能力和官方约束校验；不因 S17 扩充全局 order form 来暴露未规划的 order types。
- 通过 Alpaca Paper 的资产查询获得 canonical instrument 对应的 asset metadata。只有响应 identity 与请求 symbol 一致、class 为 US equity、状态和 tradable/fractionable 能力均可验证时才允许相应订单。未知字段、未知组合或 provider 状态不可被当作成功能力。
- account observation 保留精确 decimal `buying_power` 与其币种；fills 是有稳定 provider identity、order/account/instrument identity、精确 qty/price、execution timestamp 和观测来源的持久化 provider observations。缺失 buying power、position 或 fill 字段保持 unavailable，不补零。
- 使用只允许固定 Paper endpoint、受限 HTTP method/path 和有界响应的 Rust provider transport；凭据仍只经 native secure entry / OS Keychain 读取，HTTP headers 使用 sensitive 标记。secret、Authorization header、完整 raw payload 不进入 SQLite、UI、domain events 或日志。
- 添加 typed、schema-validated provider order/asset IPC。submit 必须核对 workspace、connection、PAPER environment、连接版本、proposal identity/hash/state、proposal account/instrument/quantity、idempotency key 和已有 unresolved attempt。所有写入只由主 UI 对用户明确动作调用；Agent/研究/策略 runtime 没有提交或撤单能力。
- 写入提交 attempt 和稳定、最多 128 字符的 `client_order_id` 后才进行 provider I/O；成功响应须核对远端 account/order/client identity，持久化 normalized order 与事件。相同 TradeX attempt 重放不得生成第二个 POST。超时、无法判定的响应或 stream gap 转 `UNKNOWN_RECONCILING`，按 client order ID 查询优先恢复；一次空查询不能证明未提交。明确的 provider validation/account rejection 显示为拒绝并要求用户修正；rate limit 展示 provider retry guidance，不自动重发尚未核实的 submission。
- 支持 provider 返回的 HTTP order identity/client identity 查询、open/history order query 与 cursor pagination。响应数、页数、decimal、字符串、timestamps 和 JSON size 均有界；重复页/token、重复或冲突 ID、不完整读取返回明确的 incomplete/degraded 状态。
- Paper order lifecycle 复用 Backend ARD 的 normalized order state/event model，并保留 provider status。REST、`trade_updates` 和活动 fill 同时到达时按 provider identity 去重；过期、done-for-day、replaced、suspended、held 等状态不能被错误映射为 filled 或 cancelled。未映射状态显式显示并阻止依赖该状态的操作。
- 撤单必须定位当前 account 下的 provider order，刷新最新状态及 remaining quantity，显示不可变的 provider order/account/environment identity 后请求用户确认。HTTP 204 只记作取消已受理 / `CANCEL_PENDING`；只有 provider stream 或 REST 确认后才记录 terminal cancel。fill/cancel race、cancel rejection 和重复撤单均以 provider order truth 收敛。
- 在 `wss://paper-api.alpaca.markets/stream` 通过 Alpaca Paper credentials 认证并订阅 JSON `trade_updates`。Stream listener 按 account 生命周期启动/停止，bounded queue 与幂等 event processing 持久化 provider updates；重连、workspace reopen 和 sleep/resume 后先恢复订阅并 REST reconcile。Stream 断开不停止 account/order/fill 的 query/reconciliation。
- 从 Alpaca Trading API 的 Paper account activity `FILL` 读取历史成交并分页对账，结合 `trade_updates` 恢复 stream 缺口。新的 beta Activity SSE 不作为 S17 的必需依赖。
- `AccountConnection.health.privateStream`、`reconciliation`、`executionEligibility` 和 `lastSuccessfulSync` 反映实际的验证结果。账户身份变化、credentials unavailable、incomplete pagination、未解决提交和不可信状态各有独立提示；这些 Paper 状态不改变 Live arming (`NOT_APPLICABLE`)。
- Order / Portfolio UI 明确标注 `ALPACA_PAPER`；broker acknowledgement、pending/cancel/fill、provider observation timestamp、origin、error/retry 和 paper simulation disclosure 分开显示。用户可查看 provider 订单、成交与 TradeX attempt/history，并在明确确认后撤销支持撤单的订单。
- 不新增 Alpaca Live adapter、Live Gateway 路由、risk policy、arming、financial approval、reservation 或 manual resolution authority。后续 Live 交易和通用人工处置仍由 S21+ / S25 定义。

## Testing Decisions

- 最高行为 seam 是 Trade surface → 真实 versioned Rust IPC/provider job → 临时真实 SQLite/outbox → React projection。测试在 provider HTTP 与 private-stream transport 边界注入确定响应和事件；同一路径验证产品行为，不以纯组件 mock 或直接 parser 测试替代端到端控制面证据。
- Provider contract/fault cases 覆盖固定 Paper host、verb/path allowlist、敏感 header、资产能力、equity order request serialization、200/204/4xx/422/429、无效/超大 JSON、redirect、timeout、限流、provider auth failure、重复页、坏 cursor、身份变化、unknown status 和 secret redaction。
- SQLite/dispatcher cases 覆盖 workspace/account/proposal/env 不匹配、stale state version、duplicate idempotency、client order ID 恢复、POST timeout 后 query-first、空查询不被当作 absence、partial/full/rejected/done-for-day/expired/cancel-pending/cancelled、fill dedup、cancel/fill race、stream 重复/乱序/断线/重连、pagination incomplete、restart/reopen 和不变的 Local Paper/Live 隔离。
- Rust-backed UI integration 覆盖连接选择、能力阻止、精确 Proposal review/submit confirmation、provider acknowledgement 与 fill 分离、unknown/reconciliation/degraded、撤单确认/拒绝/race、错误和 retry、account summary 与 fills 投影、键盘/焦点/读屏语义，以及 390/768/1280 布局。
- 真实 Tauri/native credential flow 确认 Paper 连接只取现有 Keychain 引用、取消/失败不泄漏秘密，并在断流/重启后继续以同一 PAPER account 对账。单元 fixture、schema 标识或截图不能替代这项运行时证据。
- 最终 external sandbox gate 使用当前官方 Alpaca Paper credentials 执行一笔经过明确确认的可撤回 Paper-only 测试：submit → query by broker/client ID → observe state/fill/update → cancel (if still open) → verify terminal/current account state. 不调用 Live endpoint，不将 fixture 冒充真实 sandbox。创建该外部模拟订单前须获得用户对具体 Paper account/test order 的授权；若凭据或授权不可用，按 `BLOCKED_EXTERNAL` / `IMPLEMENTED_UNVERIFIED` 记录，不标 VERIFIED。
- 完成 IPC/schema contract check、`cargo fmt --check`、workspace `cargo check`/`clippy`/tests、TS typecheck/build/unit tests、helper syntax、requirements traceability、diff check 和 Rust-backed responsive/browser tests。完整命令及精确实现 SHA 写入 resolution evidence；native package/signing 与 S33 全量回归不得由 S17 通过声明完成。

## Out of Scope

- Alpaca Live, real-money orders, live credentials, order arming/financial approval, live reservations, Privileged Live Order Gateway and generalized Manual Resolution.
- Alpaca options, crypto, multileg/bracket/OCO/OTO, short-sale enablement, margin/leverage, withdrawals/transfers/custody and broker amend/replace. 订单修改沿用 cancel + 新 Proposal 约束。
- S18 Trading 212 Demo、S19 Binance Spot Testnet、S20 Bitget Spot Demo 的 provider lifecycle，以及 S21+ cross-provider risk/authority/recovery work。
- 选择实时/历史 market-data vendor、改变 OD-001 entitlement/licensing 状态、或把 Alpaca paper-market data 作为其他数据源的替代品。
- Alpaca Paper simulation 作为 Live 成交、回测收益或未来真实交易表现的保证；S33 全应用视觉/键盘/读屏回归和 S34 签名发布包。

## Further Notes

本规范映射 PRD FR-015、FR-033 和 AC-009；UX-004 的跨环境文字状态由 S16–S20/S33 持续满足，S17 不把它提前标记为完整 VERIFIED。现有 `requirements.csv` 将 S17 条目保持 `NOT_STARTED`，实现验收后再依据跨阶段范围更新 FR-033 与 UX-004 状态。

当前依据 Alpaca 官方个人 Trading API 文档，而不是 Alpaca Broker API：

- [Authentication and separate Paper credentials](https://docs.alpaca.markets/us/v1.1/docs/authentication-1)
- [Create an Order](https://docs.alpaca.markets/us/reference/postorder)
- [Get All Orders and pagination](https://docs.alpaca.markets/us/reference/getallorders-1)
- [Get Order by ID](https://docs.alpaca.markets/us/reference/getorderbyorderid-1) and [Get Order by Client Order ID](https://docs.alpaca.markets/us/reference/getorderbyclientorderid)
- [Delete Order by ID](https://docs.alpaca.markets/us/reference/deleteorderbyorderid-1)
- [Get an Asset by ID or Symbol](https://docs.alpaca.markets/us/reference/get-v2-assets-symbol_or_asset_id)
- [Account Activities and FILL records](https://docs.alpaca.markets/us/docs/account-activities)
- [Paper trade_updates WebSocket](https://docs.alpaca.markets/us/docs/websocket-streaming)
- [Order lifecycle and status definitions](https://docs.alpaca.markets/us/docs/orders-at-alpaca)
- [Paper Trading assumptions and limitations](https://docs.alpaca.markets/us/v1.4.2/docs/paper-trading)
