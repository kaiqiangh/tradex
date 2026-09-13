# S04 Thread 历史切片验收

状态：**#17 VERIFIED；S04 父 Spec #16 仍 OPEN**。本证据只覆盖 Thread 生命周期与历史恢复；Turn 流式运行、取消/重试和安全恢复由 #18/#19 负责。

## 固定实现

- 起始审查 SHA：`9e8365a`（S04 Spec 文档提交）。
- #17 实现固定 SHA：`508af56`。
- 实现提交：`2289c4f`（Thread projection/IPC/React history）、`8e05b28`（创建后立即刷新历史并切换工作区清除选中 Thread）、`b907b44`（即时历史浏览器断言）、`508af56`（schema v6 与 stale create 保护）。
- 协议生成物由 Rust `protocol.rs` 生成；`shared/ipc-v1.schema.json`、TypeScript 类型和 AJV validators 已同步。

## 交付范围

- `thread.create`、`thread.list`、`thread.get` 使用现有 v1 command/result envelope；创建支持 Agent Mode、Execution Context、可选账户/模型 metadata 和 canonical context refs，账户归属、路由格式、标题和 stale workspace state version 在可信控制面校验。
- SQLite schema v6 增加 `threads` projection；Thread projection 与 `thread.created`/`thread.updated` outbox event 在同一事务提交，严格按 aggregate sequence 重放；`domain.snapshot`/`domain.subscribe` 支持 `thread` aggregate，gap/conflict 继续走现有 reload 语义。
- React sidebar、Threads history、Thread composer 和 detail 使用真实 IPC。创建后历史列表立即失效刷新；选择历史恢复它自己的 title/mode/context/account/model metadata；切换 workspace 清除旧 Thread identity。
- Thread projection 预留 Turn/Item、provider attempt 和 provenance 类型，但本切片不启动 Codex App Server，Send 仍显示运行时未配置原因。

## 验证

- `npm run check`：通过。结果包括 schema/type agreement、Vite build、4 个前端 projection tests、Rust workspace 全部非 ignored tests（20 个单元/集成测试通过）、requirements traceability（201 requirements / 70 screens / 12 QA scenarios / 23 baseline files）。现有 bundle size warning 保持不变。
- `cargo fmt --all -- --check`：通过。
- `git diff --check`：通过。
- Rust Thread tests：`thread_create_list_snapshot_subscribe_and_reopen_are_persistent`、`thread_create_rejects_an_account_from_outside_the_workspace` 通过；覆盖 schema v6 migration、create/list/get、snapshot replay、live subscription、workspace reopen、stale state version 和 account boundary。
- 隔离 Rust-backed browser：`tests/thread-ui.mjs` 的 `checkThreadUI` 于 2026-09-13 在 `http://127.0.0.1:1420/` 通过。临时 workspace 由 integration bridge 创建；创建 `Earnings timeline`（RESEARCH / NONE_READ_ONLY），断言创建后 sidebar/history 立即可见、detail provenance 可读、renderer reload 后可选择并恢复，390px/768px 无横向溢出且焦点/语义控件可用；无浏览器 error 日志。
- 真实桌面工作区仅做只读检查；没有删除账户、写入用户凭据、启动真实模型 Turn 或触达任何外部金融执行。

## 未覆盖边界

- Codex App Server stdio handshake、真实 `thread/start`/`turn/start`、流式 item/delta、provider attempts、cancel/interrupt、retry 和 crash recovery 尚未验证；这些保持 `RUNTIME_PENDING`，由 #18/#19 实现。
- Full QA-01–12、S33 cross-page regression 和真实上游 authenticated inference 仍不能由本切片结论替代。
