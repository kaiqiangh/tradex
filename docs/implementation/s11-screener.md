# S11 自然语言筛选器规范

日期：2026-09-19
前置：S07 canonical market/Watchlists、S08 TimeService/market gate、S09 portfolio read、S10 typed research
状态：SPEC；先完成 source-gated parse/run 垂直切片，再扩展保存与候选附加路径

## Problem Statement

Markets 已经能返回 canonical instrument catalog，但 TradeX 还没有把用户的自然语言筛选意图转换成可检查、可修改、可重跑的结构化筛选。原型中的 FilterSpec 是静态文本，编辑阈值不会改变结果，候选身份也不能可靠地进入后续 Thread。这样用户无法知道筛选器究竟解释了什么，也无法区分没有候选、数据不可用和执行失败。

S11 必须保持研究数据与金融权限分离：筛选器只降低候选集，不能创建订单、审批、arm、reservation 或 Gateway 请求。真实 OD-001/002/003 数据授权未具备时，生产路径要明确阻断；integration fixture 只能提供有限、可见标记的候选和 feature 值。

## Solution

在现有 canonical market 与 DuckDB analytical boundary 上增加 `market.screen`。命令有两个显式操作：`PARSE` 将自然语言转换成 bounded `FilterSpec`/`RankSpec` 和可审阅 revision；`RUN` 只接受当前显示且校验过的 spec revision，返回候选集及其 canonical identity、应用条件、来源和限制。解析失败、unsupported filter、blocked source、empty、failed 和 completed 是不同的 typed state。

前端在 Markets 的 Screeners 子视图中使用 Describe → Parse → Inspect/edit → Run → Results 五个阶段。自然语言重新编辑会使 parsed revision stale；结构化字段修改会使旧结果失效；Run 始终发送屏幕上可见的 revision。候选表保留 exact `instrumentId`，勾选结果只作为后续 Thread 的 context refs 入口，S11 不实现执行。

## User Stories

1. 作为研究用户，我想用自然语言描述 universe、条件和排序，以便快速缩小候选集。
2. 作为研究用户，我想在运行前看到 parsed `FilterSpec` 和 `RankSpec`，以便发现模型解释错误。
3. 作为研究用户，我想编辑 universe、predicate、threshold、rank 和 limit，以便控制实际筛选条件。
4. 作为研究用户，我想在自然语言被修改后看到 stale 状态，以便不会误用旧解释。
5. 作为研究用户，我想在结构化条件被修改后看到旧结果失效，以便结果始终对应可见 revision。
6. 作为研究用户，我想看到 parse failure 和字段级 unsupported reason，以便知道怎样修改请求。
7. 作为研究用户，我想区分 Running、Empty、Failed、Blocked external 和 Completed，以便按状态采取下一步。
8. 作为研究用户，我想重试失败运行并保留已审阅的输入，以便不必重新开始。
9. 作为研究用户，我想看到 applied conditions 和 candidate count，以便理解结果规模。
10. 作为研究用户，我想打开候选并保留 canonical instrument ID，以便进入精确的 Market detail。
11. 作为研究用户，我想只附加勾选的候选到当前或新 Thread，以便不会把完整 universe 发送给 Agent。
12. 作为研究用户，我想看到每个候选的 source、provider time、TradeX received time、freshness 和 quality，以便判断数据是否可用。
13. 作为研究用户，我想在缺少市场/feature entitlement 时看到明确的 `BLOCKED_EXTERNAL`，而不是伪造结果。
14. 作为研究用户，我想看到 fixture 明确标记为 synthetic，以便不把测试候选当成 provider truth。
15. 作为安全审查者，我想确认筛选文本、provider payload 和 prompt injection 不会改变 capability、risk、account readiness 或 execution authority。
16. 作为窄窗口和键盘用户，我想在 390px、768px、1280px 使用全部阶段和结果控件，并读到状态播报。

## Implementation Decisions

- `market.screen` 是 Backend ARD §41 的新增 canonical command，先更新英文/中文 Backend ARD §41 的操作与 payload 表，再生成 Rust/JSON Schema/TypeScript。
- `ScreenOperation` 仅允许 `PARSE` 和 `RUN`。`PARSE` 接收 bounded natural-language request、workspace ID、focus/universe hint；`RUN` 接收完整 `FilterSpec`、`RankSpec`、revision 和 result limit。未知字段、空字符串、控制字符、过长文本、过多 predicates/candidates 或不匹配 workspace 都 fail closed。
- `FilterSpec` 采用有限 enum 字段：universe（US equities、US large-cap technology、crypto spot）、predicate field（revenue growth、estimate revision、RSI、price change）、operator（GT/GTE/LT/LTE）、规范 decimal threshold；`RankSpec` 采用有限 rank field（quality、revision strength、momentum）和 direction。不得执行任意 SQL 或把 natural-language text 当作 SQL。
- Parse 是确定性、可审阅的结构化解释器；不能把不可识别的过滤条件默默丢弃。不能解析或不支持时返回 `FAILED`/`UNAVAILABLE` 并指出 field-level reason。未来模型解析可以复用同一 output contract，但不增加权限。
- Result revision 是对 canonical request、mode/focus、FilterSpec、RankSpec 和 limit 的规范化 hash；`RUN` 要求 exact revision match。自然语言或结构化编辑只在前端清除本地结果，不改 SQLite financial projection。
- Candidate rows 只包含 canonical instrument、bounded feature display values、rank、source IDs、provider/received timestamps、freshness、quality 和 optional limitation。provider symbol、原始 payload、query text beyond the bounded review field 和秘密不会进入结果或 Thread context。
- `market.screen` 通过现有 S06 data-source catalog gate：生产缺 OD-001/002/003 时返回 `BLOCKED_EXTERNAL`/`UNAVAILABLE`，不发网络请求；不能用账户连接或 portfolio fixture 充当 market entitlement。integration-test 且 `TRADEX_SCREENER_FIXTURE` 开启时，使用四个 canonical instruments 的静态 feature rows，并在结果携带 `fixtureLabel`。
- DuckDB 仍是 analytical boundary。筛选 materialization 只能写入 workspace 的 market DuckDB，不写 SQLite/outbox/account/model/risk/thread；查询行数和 fixture candidates 有上限。第一切片允许在 fixture 中使用 bounded in-memory rows，但必须保留同一 typed result seam，后续历史 feature ingestion 复用该 seam。
- UI 不把 Screener 变成新的一级导航；复用 Markets 页面与现有 More/keyboard navigation。结果行打开 exact canonical market detail；Attach 只提交勾选的 canonical refs 到 Composer，不能自动启动 Agent 或触发交易。
- `Run` 不进入 Trade mode，也不调用 `order.*`、approval、arming、risk、reservation 或 Gateway。研究结果可以供 S10 typed research 使用，但外部文本是 untrusted data。

## IPC Contract

```text
market.screen({
  workspaceId,
  operation: "PARSE" | "RUN",
  naturalLanguage,
  focus,
  filterSpec?,
  rankSpec?,
  revision?,
  limit?
}) -> ScreenerResult
```

`ScreenerResult` 至少包含 workspace ID、operation、state (`PARSED`, `RUNNING`, `EMPTY`, `COMPLETED`, `BLOCKED_EXTERNAL`, `FAILED`)、bounded natural-language review text、optional FilterSpec/RankSpec、revision、applied conditions、candidate count、candidates、source provenance、availability reason、limitations 和 optional fixture label。`RUN` 只在 `revision` 与规范化 spec exact match 时执行；不匹配返回 `SCREENER_REVISION_STALE`，没有 partial result。

## Testing Decisions

- Rust protocol tests 验证 enum/schema bounds、unknown fields、control characters、invalid decimal/empty predicate、unsupported parse、revision mismatch、workspace isolation 和 candidate/result limits。
- Screener service tests 验证同一个 parse input 生成稳定 spec/revision；每个 predicate 和 rank edit 都改变 revision/候选；empty 与 blocked source 不伪造成功；fixture rows 只在 integration flag 出现；运行前后 SQLite domain snapshot/outbox/account/model/risk/thread version 完全相同。
- DuckDB boundary test 使用临时 workspace，确认 materialization 与 SQLite 分离、上限生效、重开可重新查询；没有 entitlement 时不写 provider response 或凭据。
- Rust-backed browser test 验证五个阶段、parse/unsupported/blocked/empty/failed/completed 文案、结构化编辑 stale、retry 保留输入、候选 exact identity、键盘 focus/aria-live 和 1280/768/390 无横向溢出；console warn/error 必须为空。
- 运行 `npm run schema:check`、typecheck/build/unit、Cargo workspace/integration、clippy、fmt、diff check 和 requirements traceability，并记录 exact SHA。Fixture/label 只能证明 contract/rendering，不证明真实 market entitlement。

## Out of Scope

- 购买或验证 OD-001/002/003 entitlement、真实 provider feature/history ingestion、新闻/filings/fundamentals 抓取和完整跨市场 universe。
- 任意 SQL、无限候选、全量 provider dataset 传给模型、后台全市场订阅、tick/order-book 或回测数据生产。
- Screener 驱动订单、审批、arm、risk、reservation、Gateway 或 Live execution。
- S12 artifact library、S14 strategy、S15 backtest、真实 Thread persistence beyond attaching bounded canonical refs，以及 S33 全页面回归。

## Further Notes

- 权威来源：PRD §§36–37、61 FR-035/FR-053、63 AC-040；UI Spec C1–C3、§14.6；Backend ARD §§32、41–42；Frontend ARD §11 timeline registry；Coverage AC-040；QA-08。
- 该规范把“可编辑解释”和“可运行结果”绑定到同一个 revision，修复原型中静态 FilterSpec、Run 忽略输入和错误 instrument mapping 的问题。
- S06 OD-001/002/003 仍可能是 `BLOCKED_EXTERNAL`；实现必须保留该状态，不能用 fixture 或成功解析宣告 real-data 完成。
