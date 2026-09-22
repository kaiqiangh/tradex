# S16 本地模拟交易与持仓更新规范

日期：2026-09-21  
前置：S09 组合与 FX 只读切片、S13 Draft/Proposal 切片  
状态：S16 实现与父 Spec 验收已完成。代码检查 SHA：`100db3d0fa0e7718174e1f335e095517833a3efc`；证据索引 SHA：`4ad4d4a581b428dbb80dadb4f7f18eda5d4f86c5`。验证细节见 [S16 Local Paper 隔离验收](s16-local-paper-isolation-evidence.md)。S17–S20 provider lifecycle、S21+ financial authority 与 S33 全量回归仍属于后续范围。

## Problem Statement

S16 开始实施前，TradeX 已能够把订单草稿保存为不可变 Proposal，也能在能力层把 `LOCAL_PAPER` 识别为 C3 simulation context，但 Local Paper 仍是不可用的 provider catalog 占位。账户页明确写着 simulation engine 未配置，`portfolio.get` 只能读取 provider 观察结果或 fixture，Control Plane 没有本地订单、成交、现金、持仓、未完成订单和填充记录的权威持久化模型。

这在 S16 开始时造成三个产品缺口：用户不能从一个明确标为 TradeX simulation 的 Local Paper 账户执行已审阅 Proposal；重开 workspace 后没有可恢复的模拟状态；任何原型中的状态或固定 broker-like 数值都可能被误读成 provider execution truth。S16 补齐了由 TradeX 管理的可重复本地执行边界，同时保持它与 broker Paper/Demo/Testnet 和 Live Gateway 的架构隔离。

Local Paper 不是真实市场、券商账户或预期 Live 结果的代理。它只消费 Local Paper Proposal 和本地可审计的 simulation input，所有结果都标记为 `TRADEX_SIMULATION`，并在 UI、IPC、持久化投影和 portfolio 汇总中保留该身份。

## Solution

为每个 workspace 提供一个无需凭据的 Local Paper simulation account。账户使用固定的 provider/environment identity：`providerId=local-paper`、`environment=LOCAL`、execution context `LOCAL_PAPER`。首次打开 Local Paper surface 或提交 Local Paper Proposal 时由 Rust Control Plane 以幂等方式确保该账户存在；账户初始现金来自 workspace base currency 的可见 simulation profile，不能从外部 provider 推断或伪造。

建立一个受 Control Plane 所有的确定性 paper engine。它只在 SQLite 事务中读取权威 Proposal、simulation profile 和本地 quote/scenario，按规范 decimal 计算订单状态、成交数量、现金、持仓、未完成订单、已实现/未实现 P&L 和事件。所有变化都产生 append-only execution/fill/portfolio events，并更新可查询的 current projection。engine 不读取 Keychain、不调用 provider adapter、不发网络请求、不创建 Gateway grant，也不进入 Privileged Live Order Gateway。

订单执行必须绑定一个完整且不可变的 Local Paper Proposal。提交使用幂等 key 和 proposal identity；重复请求返回同一权威订单，不重复扣现金或生成成交。订单生命周期复用 Backend ARD 的归一化状态：`PROPOSED → ACCEPTED → PARTIALLY_FILLED/FILLED`，失败为 `REJECTED`，未成交限价单可以保持 `ACCEPTED`，取消为 `CANCEL_PENDING → CANCELLED`。Local Paper 的 `ACCEPTED` 表示 TradeX simulation engine 接受，不是 provider acknowledgement；投影始终显示这一区别。

报价和故障场景必须可复现。生产路径只接受受约束的本地 `SimulationQuote`/scenario identity，禁止浏览器提交任意成交结果；集成测试可以通过同一 typed seam 注入 full fill、partial fill、resting、reject、cancel race、duplicate request 和 restart 场景。缺少或过期的本地 simulation input 会阻塞提交并保留 Proposal，不得用 fixture 或 broker 数据冒充成功。

## User Stories

1. 作为用户，我想在账户列表看到一个无需凭据的 `Local Paper · LOCAL PAPER · TradeX simulation` 账户，以便不会把它误认为 Alpaca Paper、Demo/Testnet 或 Live。
2. 作为用户，我想从现有 Local Paper Proposal 进入提交操作，以便执行对象只能来自已保存、不可变、属于当前 workspace 的 Proposal。
3. 作为用户，我想在提交前看到 Proposal identity、instrument、side、quantity、order type、TIF、simulation quote、预计现金影响和明确的 simulation disclosure，以便知道这是本地模拟。
4. 作为用户，我想提交 Market/Limit 订单并看到明确的 `ACCEPTED`、`PARTIALLY_FILLED`、`FILLED`、`REJECTED` 或 `CANCELLED` 状态，以便理解 engine 的实际结果。
5. 作为用户，我想让 Limit/GTC 未达到成交条件的订单保留为 open order，以便稍后在新的本地 quote/scenario 下刷新，而不是被伪造为成交。
6. 作为用户，我想取消仍可取消的 Local Paper 订单，并看到剩余数量、取消状态和最终结果，以便取消不会抹掉已经发生的部分成交。
7. 作为用户，我想在发生部分成交时看到每一笔 fill 的数量、价格、价值、费用（如 profile 配置）和时间，以便审计现金与持仓变化。
8. 作为用户，我想在成交后看到现金、reserved cash、持仓数量、平均成本、已实现 P&L、未实现 P&L 和 exposure 更新，以便组合视图与订单历史保持一致。
9. 作为用户，我想关闭并重新打开 workspace 后仍能看到同一 Local Paper 订单、fills、cash、positions 和 event history，以便模拟结果不会依赖页面内存。
10. 作为用户，我想在 portfolio 页面看到 Local Paper 的账户、持仓、open orders 和 fills 汇总，并保留 `TRADEX_SIMULATION` provenance，以便组合汇总不掩盖环境身份。
11. 作为用户，我想看到没有 quote、无效 quantity、现金不足、卖出数量不足、超出 profile 限制和不支持 order/TIF 组合的字段级错误，以便失败不会改变任何金融状态。
12. 作为用户，我想看到 Local Paper 提交失败后的 Retry/Reload 入口，并知道是否已经创建了订单，以便网络错误语义不会被套用到本地事务。
13. 作为并发用户，我想在同一账户同时提交相同 Proposal 时只得到一个 order 和一组影响，以便幂等与账户级事务不会重复扣款。
14. 作为并发用户，我想在提交、取消、quote refresh 和 workspace reopen 竞争时收到 state-version conflict 或权威当前状态，以便不会覆盖更新后的持仓。
15. 作为安全审查者，我想确认任何 Local Paper 命令都无法触发 Live Gateway、provider adapter、Keychain 或网络 I/O，以便 C3 simulation 不越权。
16. 作为安全审查者，我想确认 Agent/strategy/model 只能看到脱敏的 simulation projection，不能提交未经用户选择的 Proposal，也不能注入成交价格、fill 数量或账户余额。
17. 作为安全审查者，我想确认 Local Paper 结果不会把 `brokerOrderId`、provider ACK、Live eligibility 或 broker truth 写入 projection，以便后续 provider slices 不会复用错误身份。
18. 作为审计者，我想沿着 Proposal → paper order → fill → portfolio event 追踪 workspace、account、scenario、sequence 和 state version，以便重建每次模拟变更。
19. 作为测试人员，我想用固定 scenario 重复运行相同 Proposal 得到相同 order/fill/portfolio 结果，以便验证 deterministic simulation。
20. 作为测试人员，我想注入 full fill、partial fill、resting、reject、cancel race、duplicate request、malformed quote 和 restart，且每种结果都有明确的 typed state 和无数据丢失保证。
21. 作为键盘用户，我想通过语义控件打开提交/取消对话框、查看 fill/history、关闭并恢复焦点，以便整个 Local Paper 流程不依赖鼠标。
22. 作为窄屏用户，我想在 390px、768px 和 1280px 下看到环境标识、状态、订单字段、错误与取消结果，避免横向溢出或状态只靠颜色表达。
23. 作为用户，我想看到“TradeX simulation；not provider truth；not Live execution”固定文案和可访问状态文本，以便截图、导出和 portfolio 汇总也不会丢失边界。
24. 作为后续 provider slice 的实现者，我想消费稳定的 normalized order/fill/portfolio projection，而不需要把 Local Paper 的事件改写成 broker 事件，以便 S17–S20 复用组件但保留 environment identity。

## Implementation Decisions

- Local Paper 是 workspace-scoped 的单一内置账户。使用稳定的 `local-paper` provider、`LOCAL` environment、`LOCAL_PAPER` execution context 和固定的 simulation-only health/capability 文案；它不接受 Connect 表单、外部凭据或 provider probe。`account.list` 与 `context.catalog` 必须能读取它，首次 ensure 是幂等的并写入本地账户 metadata。
- 复用现有 `OrderDraftFields`、`OrderProposal`、`PortfolioValue`、`PortfolioHolding`、`PortfolioOrder`、`PortfolioFill` 和 Backend ARD §28 的 order state vocabulary。不要为 Local Paper 复制一套 renderer-only order model；新增类型只承载 paper-specific identity、scenario、execution/fill metadata 与 state projection。
- 新增明确的 canonical IPC 命令，具体 wire schema 必须在 Backend ARD §41–42 及中英文生成类型中同步：
  - `paper.account.ensure({workspaceId}) -> LocalPaperAccount`；重复调用返回同一 account/state version。
  - `paper.get({workspaceId, expectedStateVersion?}) -> LocalPaperState`，返回 simulation profile、cash/balances、positions、openOrders、fills、P&L、event cursor、state version 和 disclosure。
  - `paper.order.submit({workspaceId, proposalId, expectedProposalStateVersion, idempotencyKey}) -> PaperOrderResult`。
  - `paper.order.cancel({workspaceId, orderId, expectedStateVersion, idempotencyKey}) -> PaperOrderResult`。
  - `paper.quote.refresh({workspaceId, expectedStateVersion}) -> LocalPaperState`：在没有 open order 时刷新受约束的本地 quote，写入新的 quote identity/observed time 与 `QUOTE_REFRESHED` event。
  - `paper.scenario.set({workspaceId, expectedStateVersion, profile}) -> LocalPaperState` 只修改 Local Paper simulation profile，不产生 provider/account mutation；profile 字段有严格 bounds，且不能由 Agent 工具调用。
- `paper.order.submit` 只接受 `fields.environment=LOCAL_PAPER`、Local Paper venue/account identity 和当前 workspace 的 Proposal；Live/Paper/Demo/Testnet、跨 workspace、invalidated Proposal、重复已消费 Proposal、旧 state version 和不支持的 order/TIF 组合 fail closed。Renderer 不得提交 order state、fill、cash、position、quote、provider order ID 或 P&L。
- `PaperOrderResult` 至少包含 `orderId`、`workspaceId`、`accountId`、`proposalId`/`proposalHash`、`environment=LOCAL`、normalized fields、state、requested/filled/remaining quantity、average fill price、simulation scenario/quote identity、created/updated time、state version、event sequence、explicit disclosure 和 typed error/remediation（如适用）。所有 decimal 使用规范字符串，禁止 binary floating point 参与 canonical validation 或 arithmetic。
- Local Paper engine 使用现有 SQLite single-writer/`BEGIN IMMEDIATE` 事务边界。最小持久化模型包括 `paper_accounts`/profile、`paper_orders`、`paper_fills`、`paper_positions`、`paper_balances`、`paper_events`/outbox；每个 projection 带 workspace/account identity、sequence、state version 和 bounded JSON。唯一约束覆盖 `workspace_id` 单账户、`proposal_id`、`idempotency_key`、`order_id`/`fill_id` 与事件 sequence。迁移必须先备份旧 workspace，失败时 fail closed。
- Order submission、fill application、cash reservation/release、position average-cost update、realized/unrealized P&L update、open-order transition 和 event append 必须在同一个账户级事务中完成。部分成交只减少 remaining quantity；取消不会删除 fills，终态订单不能重复取消或提交。
- Simulation profile 只允许确定性本地输入：base currency、starting cash/initial balance、quote source identity、scenario seed/version、fee/slippage policy 和 bounded fill policy。quote 必须带 instrument、bid/ask/last 或可用价格、currency、observed time、scenario hash 和 freshness；缺少、过期、负数/非规范 decimal 或 currency 不匹配时返回 `PAPER_QUOTE_UNAVAILABLE`/typed equivalent，不读取外部 provider。
- MVP order rules 只承诺现有 `Market`/`Limit`、`BASE`/`QUOTE` quantity、`Day`/`Gtc`/`Ioc`/`Fok` 与 profile 明确支持的 instruments。Market 使用确定性 reference price；Limit 只在 quote crossing 时成交，否则保持 `ACCEPTED`；IOC/FOK 的剩余量按 profile 规则终止。复杂 stop/trigger、margin、short、derivatives、borrow、withdrawal、FX conversion 和 corporate actions 不在 S16 中实现。
- `paper.get` 作为 Local Paper portfolio 的权威查询，并将同一 projection 合并进 `portfolio.get` 的 accounts/holdings/open_orders/fills；账户 environment、simulation provenance、base/account/native currency 和 FX quality 必须显式保留。Local Paper 的 `liveRisk.eligible` 永远为 false，原因明确为 `TRADEX_SIMULATION_NOT_LIVE`；模拟 P&L 不得被用作 Live readiness 或 broker truth。
- `paper.*` 不经过 `Privileged Live Order Gateway`，不创建 approval、arming、reservation、provider execution attempt、provider client order ID、broker ACK 或 reconciliation record。S21–S30 可以消费 normalized paper events，但不能把它们当成 Live authority evidence。
- UI 复用 Accounts、Portfolio 和 Order Draft/Proposal 页面，增加 Local Paper simulator panel、submit/cancel confirmation、scenario/profile disclosure、order/fill/history 和 empty/loading/error/stale/retry states。所有状态使用文本 + 状态标识；modal 关闭后恢复触发控件焦点，390/768/1280 不能隐藏 environment 或 remediation。
- Local Paper actions 只能从明确选中的 Local Paper Proposal 发起。Agent mode/context picker 可以展示 C3 capability，但 typed research/model/thread tools 不获得 `paper.order.submit`；只有用户在 Trade surface 明确触发，Control Plane 才能消费 Proposal。
- 任何 integration fixture、synthetic quote 或 seeded scenario 都必须在 response/UI/event 中标记 `TRADEX_SIMULATION` 与 scenario identity。固定 ID、浏览器 fixture、截图或 React local state 不能作为完成证据。
- 中文/英文 Backend ARD §30、§32、§41–42，Frontend ARD 的 order/portfolio surfaces，requirements.csv（FR-033、AC-064、UX-004）、surfaces.csv（G1）和 coverage/evidence 文档在 S16 实现验收时同步；实现状态、SHA 和验证边界见 [S16 Local Paper 隔离验收](s16-local-paper-isolation-evidence.md)。S16 的局部验收不代表 S17–S20 provider lifecycle、S21+ financial authority、S33 全量回归或跨阶段需求状态已完成。

## Testing Decisions

- Rust protocol/schema tests 验证所有 `paper.*` command/response 的 camelCase、`deny_unknown_fields`、ID/length/vector bounds、environment enum、decimal string、scenario/profile bounds、unknown field 和 cross-workspace rejection；schema check 必须同时验证生成的 TypeScript/JSON Schema。
- Engine unit tests 验证 canonical decimal arithmetic、market/limit crossing、BASE/QUOTE conversion、Day/GTC/IOC/FOK、full/partial/resting/reject/cancel、fee/slippage、average cost、realized/unrealized P&L、cash reservation/release、position close/open 和 invariant checks。
- SQLite integration tests 验证 migration from current schema, ensure idempotency, submit/retry idempotency, same-account serialization, state-version conflict, unique proposal/order/fill/event constraints, workspace isolation, reopen persistence, outbox/replay and corrupted projection fail-closed。
- Boundary tests spy on provider adapter, gateway launcher, Keychain/native credential path and network layer; every Local Paper submit/cancel/query must prove zero calls. Tests also assert no approval, arming, reservation, broker order ID or provider ACK is written.
- Dispatcher tests verify Proposal identity and hash are reloaded from SQLite, renderer-supplied fill/cash/position/P&L/quote is rejected or ignored, invalidated/non-Local/foreign Proposal is blocked, and duplicate requests return the original authoritative result.
- Rust-backed browser tests run real stdio IPC with a temporary workspace: ensure account → save/generate Local Paper Proposal → submit full fill → query/reopen → submit partial/resting → cancel → inspect portfolio/fills/events. Negative paths cover empty state, insufficient cash/position, stale quote, rejected order, state conflict, duplicate submit, model gateway unavailable, 390/768 overflow, focus restore and no console error.
- Fixture/browser-only tests may cover copy, keyboard and responsive layout, but cannot prove persistence, simulation arithmetic, no-gateway boundary or broker truth. External provider sandbox, OAuth/API key, Live arming, approvals, reservations, reconciliation and S33 full regression remain future evidence.
- S16 closeout runs schema check, Rust fmt/check/clippy, unit/integration tests, TypeScript typecheck/build/unit, UI helper syntax, requirements traceability and `git diff --check`; runtime claims bind to exact SHA and distinguish native desktop, Rust-backed browser, fixture, provider and unavailable evidence.

## Out of Scope

- Alpaca Paper, Trading 212 Demo, Binance Spot Testnet、Bitget Demo 的认证、签名、能力探测、order/cancel/query/private stream/reconciliation（S17–S20）。
- Risk policy、arm/disarm、financial approval/consent、reservation、Privileged Live Order Gateway、UNKNOWN_RECONCILING、broker ACK、Live order/cancel/fill and manual resolution（S21–S30）。
- Real market quotes, provider balances, broker account truth, external network, margin/short/derivatives/borrow/withdrawal/custody and FX conversion/corporate-action semantics.
- Stop/trigger/bracket/OCO/amend-replace、complex order rules、multi-currency settlement and tax/fee reporting beyond the bounded profile.
- Strategy worker access, model routing, automatic trading, background scheduler, cloud sync, sharing, mobile UI, exports and full artifact provenance beyond the Local Paper event chain.
- S33 full-page/accessibility regression, S34 packaging/performance/signing and final `dev → main` PR.

## Further Notes

- 权威来源：PRD §§38、42.1、50、67.3、FR-033、AC-064、UX-004；UI Spec §G/G1、§14.9–14.10；Backend ARD §§28–32、41–42；Frontend ARD 的 Accounts/Portfolio/OrderProposal surfaces；Coverage Matrix FR-033/AC-064/UX-004；QA 中 Local Paper 与 non-live lifecycle 的相关回归项。
- S09 的组合 projection 只提供 read-only aggregation 和 FX provenance；S16 必须让 Local Paper domain state 成为新的权威来源，不把 provider fixture 复制成模拟余额。
- S13 的 Proposal 仍是不可变审阅对象；S16 只消费它并追加 paper order/fill/events。编辑 Draft、刷新 Proposal 或切换 environment 后必须重新生成并重新选择 Proposal。
- `ACCEPTED`、`PARTIALLY_FILLED`、`FILLED` 只描述 TradeX engine 的 simulation state。任何 UI、event、portfolio 或导出字段都不得使用“broker acknowledged/filled”或 Live quality 语义。
- 若实现需要扩大 order types、quote sources、multi-account Local Paper 或 live-compatible risk semantics，应先更新本规范、双语 ARD 和依赖关系，再创建独立工作项；不要在 S16 中隐式扩张边界。
