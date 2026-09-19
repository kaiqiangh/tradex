# S11 筛选器持久化与候选附加规范

日期：2026-09-19  
前置：S11 #37 source-gated `market.screen` parse/run 已完成实现；S07 canonical market/ThreadContextRef、S10 typed research  
状态：SPEC；本规范对应 GitHub #38，保存、重开和候选附加仍需实现与验证

## Problem Statement

S11 已能把自然语言转换为可审阅的 `FilterSpec`/`RankSpec` 并运行受限候选筛选，但用户关闭 Markets 或刷新后会失去筛选定义。候选结果也只能打开 Market detail，不能把明确勾选的 canonical instrument 安全地带入当前或新 Thread。把完整 universe、provider payload 或未勾选候选带入 Agent 会扩大上下文和权限边界。

## Solution

在现有 `market.screen` parse/run 结果之上增加 workspace-scoped Screener library。用户保存 bounded natural language、结构化 spec、revision、limit 和可见状态；重新打开只恢复已审阅输入，不恢复 Live 数据授权或旧候选结果。编辑保存的定义会产生新的 revision，并使旧运行结果失效。

结果表为每个候选保留 exact canonical `instrumentId`。用户通过复选框选择候选后，可以把所选项作为 canonical `ThreadContextRef` 预填到当前 Thread 的下一 Turn 或新 Thread；此动作不会自动发送 Turn、改变 capability、写金融状态或调用 Gateway。未勾选候选、完整 universe、provider 原始 payload 和凭据永远不进入附加 context。

## User Stories

1. 作为研究用户，我想按 workspace 查看已保存的筛选器，以便继续上次的研究。
2. 作为研究用户，我想保存自然语言、FilterSpec、RankSpec、limit 和 revision，以便重开时仍能审阅同一组输入。
3. 作为研究用户，我想为筛选器使用唯一且有界的名称，以便快速区分定义并避免覆盖错误对象。
4. 作为研究用户，我想编辑已保存筛选器并得到新 revision，以便旧结果不会被误认为对应新条件。
5. 作为研究用户，我想在重开后重新运行筛选器，以便重新经过当前的数据源门禁，而不是恢复过期的 Live authority。
6. 作为研究用户，我想在 SQLite projection 发生并发修改时看到 stale version 错误，以便不会静默覆盖其他修改。
7. 作为研究用户，我想在数据为空、运行失败或外部数据被阻断时保留已审阅输入，以便可以安全重试。
8. 作为研究用户，我想勾选一个或多个候选，以便只把我明确选择的 canonical instruments 带入后续研究。
9. 作为研究用户，我想把所选候选预填到新 Thread，以便先检查上下文再开始 Turn。
10. 作为研究用户，我想把所选候选附加到当前 Thread 的下一 Turn，以便继续当前研究而不改变历史 Turn。
11. 作为研究用户，我想看到候选的 exact identity 和选择状态，以便避免 symbol 重名或错误映射。
12. 作为安全审查者，我想确认保存和附加不写账户、风险、approval、arming、reservation、Gateway 或 credentials。
13. 作为安全审查者，我想确认 Thread 创建和 Turn 开始仍会在后端重验 context ref，以便伪造 instrument hash 或 workspace 不会被接受。
14. 作为窄屏和键盘用户，我想用键盘完成保存、重开、选择和附加，并在 390px、768px、1280px 看到清晰状态。

## Implementation Decisions

- 保留 `market.screen` 作为 parse/run 命令；新增 `screener.list`、`screener.save`、`screener.update` 和 `screener.attach`。这些命令共享一个 Rust screener/storage seam，前端不复制持久化或 identity 规则。
- Screener library 是 workspace-scoped SQLite projection，不是金融 domain aggregate，也不产生 outbox/domain event；DuckDB analytical/materialization 边界保持独立。保存的 projection 不含候选行、provider 原始响应、账户、凭据或 Live entitlement。
- Library 返回 `{ workspaceId, stateVersion, screeners }`。每个 saved screener 至少包含 `screenerId`、`workspaceId`、唯一名称、bounded natural language、`FilterSpec`、`RankSpec`、revision、limit、最近可见状态和创建/更新时间。名称、文本、predicate 数量和 limit 复用 S11 的既有上限。
- 写入请求带 `expectedStateVersion`。未知 workspace、名称冲突、越界输入、revision 与规范化 spec 不一致以及 stale version 都 fail closed，并返回 typed error；失败 mutation 不产生部分 projection。
- 更新保存定义时重新计算 canonical revision，清空旧候选结果并保留 reviewed inputs。重开只加载 definition/state badge；下一次 run 必须重新通过当前 source gate，不能从保存记录恢复 Live authority。
- `screener.attach` 接收 workspace、结果 revision 和明确选中的 canonical instrument IDs，限制选择数量，重新校验这些 IDs 属于当前 bounded market catalog，并返回带 workspace-bound hash 的 `ThreadContextRef[]`。它不接受或返回完整 ScreenerResult、provider payload 或自然语言文本。
- `ThreadContextRef` 继续作为唯一跨页面身份载体。新 Thread 复用 `thread.create.linkedContexts`；当前 Thread 复用已有 Turn composer 的 pending contexts，并在下一次 `turn.start` 由后端再次执行 context catalog/capability 校验。没有当前 Thread 时，UI 只提供新 Thread 路径。
- 结果行的选中状态只存在于当前 UI 结果 revision；重跑、编辑、空结果、失败或 stale 后清空选择。未勾选行绝不写入 composer pending context。
- 保存、重开和附加都不改变 Agent mode、ExecutionContext、account readiness、risk policy、arming 或 approval；也不自动启动 Turn。外部来源继续是不可信数据。
- UI 复用 Markets、New Thread、Threads 和现有 context chips/keyboard navigation，不增加一级导航。保存库和附加控件必须有明确 disabled/error/empty 状态，并保留 `aria-live` 状态播报。

### IPC shape

```text
screener.list({ workspaceId }) -> ScreenerLibrary
screener.save({ workspaceId, name, definition, expectedStateVersion? }) -> ScreenerLibrary
screener.update({ workspaceId, screenerId, name, definition, expectedStateVersion }) -> ScreenerLibrary
screener.attach({ workspaceId, revision, selectedInstrumentIds }) -> ScreenerAttachment
```

`definition` 只包含 S11 已审阅的 bounded natural language、`FilterSpec`、`RankSpec` 和 limit；`ScreenerAttachment` 只包含 workspace ID、revision 和 canonical `ThreadContextRef[]`。Wire schema、Rust 类型和 TypeScript client 必须由同一 contract 生成/校验。

## Testing Decisions

- 测试外部行为和安全边界，不测试 SQLite 查询实现细节。最高 seam 是真实版本化 IPC → 临时 SQLite projection → Thread/Turn context validation；纯 React 测试只覆盖键盘、状态和布局。
- Rust storage/service checks 覆盖首次保存、按 workspace 隔离、重名、未知 workspace、名称/文本/limit/predicate 越界、stale version、revision mismatch、更新后旧结果失效和重开只恢复 definition。
- IPC integration checks 覆盖 list/save/update/attach 的 typed success/error、空/失败重试输入保持、selected-only attachment、重复/未知/伪造 instrument identity，以及保存/附加前后 domain snapshot、outbox、account、risk、approval、arming、Gateway 和 credentials 不变。
- Capability checks 覆盖合法 instrument ref、workspace/hash 篡改、未列入当前 catalog 的 instrument 和超出选择上限；Thread create 与 Turn start 的既有后端重验必须继续生效。
- Rust-backed browser checks 覆盖保存库、重开、stale mutation、编辑后 stale、empty/failed retry、明确勾选后的 selected-only attach、新 Thread/current Thread 两条入口、键盘 focus/aria-live 和 390/768/1280 布局。真实 provider 不可用时只能记录 `BLOCKED_EXTERNAL`，fixture 不能升级为 Live 证据。
- 交付前运行 schema check、typecheck/build、相关 Rust/unit/integration、workspace feature suite、clippy、fmt、diff check、UI helper syntax 和 S33 全回归；记录起始 SHA、完成 SHA 及未能运行的桌面/CUA证据。

## Out of Scope

- 删除或跨 workspace 共享筛选器、后台定时运行、收藏候选结果、保存 provider payload、恢复旧 Live 数据或把 screener 变成金融 domain aggregate。
- 自动创建/发送 Turn、自动改变 Agent mode、账户/风险/approval/arming/Gateway/订单状态，或让 screener 绕过现有 capability/context validation。
- 全量 universe、任意 SQL、真实 provider entitlement、历史特征摄取、S12 artifact library 和 S33 之外的全应用新回归范围。

## Further Notes

- 本切片沿用 #37 的 deterministic revision 和 source-gated state；保存成功不等于真实数据可用，`BLOCKED_EXTERNAL`、`EMPTY` 和 `FAILED` 必须继续可见。
- #38 已经是完整的 tracer-bullet ticket，覆盖 schema、projection、IPC、UI 和验证；不再创建水平拆分子票。它的唯一技术 blocker 是 #37 的实现（已完成但按工作流保留 OPEN）。
- 实现完成后需同步 Backend ARD §41.15、中文对应段落、schema manifest 和 S11 evidence；在同一 `dev` SHA 上完成 code review 与全套验证后再更新 issue resolution，不能提前关闭 #38 或创建 `dev -> main` PR。
