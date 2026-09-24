# S19 #71 Binance Spot Testnet 准确订单撤销证据

日期：2026-09-24。分支：`dev`。Issue：[准确复核并撤销 Binance Testnet 开放订单 #71](https://github.com/kaiqiangh/tradex/issues/71)。

## 范围与结果

准确订单撤销切片已通过本地 fixture 验证。主 Trade UI 会先按准确 symbol 和 provider order ID 刷新所选 Binance Spot Testnet 订单；只有返回订单簿为 current，且准确订单对当前 connection 和远端账户仍可撤销、并具备已知有效剩余数量时，才打开确认框。对未变化订单的精确刷新只推进观测时间，复核保持可用，也不会重置已记录的撤销意图。用户明确确认后，系统先持久化一次撤销意图，最多发送一次签名 Testnet DELETE。重新打开、重复请求、确定拒绝、provider 状态变化/终结、传输结果不明及并发私有流成交均不会导致盲目重放或丢失成交。确认框支持安全关闭，并分别标明提交中、结果待确认和终态。

测试 seam 使用合成 HTTP/WebSocket 响应、临时 SQLite 和仅供集成测试的 Rust 浏览器桥接。未使用真实 Binance 凭据、未发送外部 provider 请求、未创建真实订单。这不能关闭 provider-hosted Testnet gate：父 Spec #67 和 issue #72 仍开放。点击式原型未修改。

## 验证

- `cargo test --test binance -- --test-threads=1`：18 项通过、0 项失败、1 项忽略（原生 Keychain）。
- `testnet_cancel_is_single_exact_delete_and_reconciles_terminal_status`：只向准确 `/api/v3/order` 发送一次 DELETE；后续 Detail 才确定终态。
- `testnet_cancel_definitive_rejection_is_retained_and_never_replayed` 与 `testnet_cancel_reopen_recovers_unknown_intent_without_delete_or_replay`：拒绝/重开后保留幂等键，不发生第二次 DELETE。
- `testnet_cancel_rechecks_exact_order_and_never_resends_unknown_outcome` 与 `testnet_cancel_completion_merges_concurrent_partial_and_full_private_stream_fills`：变化/结果不明不会重放；竞态中的两个 trade ID 和最终成交均被保留。
- Rust-backed 浏览器 fixture（最终树，2026-09-24）：workspace fixture 先在本地完成 onboarding 与模型就绪；随后完整 Trade UI 流程通过——先刷新准确 Detail（并遵守有界读冷却）再打开确认框；stale 观察阻止复核；provider 明确拒绝展示 `PROVIDER_CANCEL_REJECTED` 且不重放；结果不明保持等待 provider 确认；provider 终态订单保持可见且无撤销操作；无可用剩余数量的订单不提供复核；Escape 不发送 command；明确确认后记录 `CANCELED` 且保留之前的部分成交；390、768、1280 px 响应式检查通过且无 console error（先 `workspace-ui`，再 `provider-ui` 的 `binance/TESTNET`）。
- `npm run check` 已通过：IPC schema 生成检查、TypeScript 与生产构建、9/9 前端单测、Rust workspace tests 和 requirements traceability（203 项 requirement、70 个 screen、13 个 QA 场景、23 个基线文件）。保留现有 bundle-size 提示；原生 Keychain/Gateway integration tests 按设计忽略。
- `cargo fmt --all`、`node --check tests/provider-ui.mjs` 和 `git diff --check` 已通过。
- `cargo check --features desktop --bin tradex` 已通过。
- `cargo clippy --workspace --all-targets -- -D warnings` 被基线 `2ceaa61` 已存在的 6 项 Clippy finding 阻断：5 项 library warning 位于 signed request/history/order merge/private-stream helper，1 项位于 private-stream 测试 helper。此次撤销代码中新引入的多余引用已修复；没有把无关的 #70 清理并入本票。

## 后续验证 — 2026-09-25

撤销 guard 现在要求剩余数量严格大于零。Provider 十进制值会先规范化再比较，因此即使 provider 将已全部成交的订单标为 `PARTIALLY_FILLED`，其规范化剩余数量 `0` 也不会进入 DELETE。Renderer 和可信 storage command 都会拒绝该状态。`testnet_cancel_rejects_zero_remaining_quantity_before_delete` 已通过，并确认没有 provider 撤销调用；本次改动树上的 `npm run check`、`cargo check --features desktop --bin tradex`、格式、JavaScript 语法与空白检查均通过。

Rust-backed 浏览器的定向断言也已通过：零剩余 fixture 不显示撤销复核按钮，直接调用可信 command 返回 `ORDER_NOT_CANCELABLE`。但在这些断言之后，更广泛的 provider 浏览器运行因后续 Account 响应式断言失败而未能全绿；单独重放 Account 响应式/断开/删除尾段时通过。之后一次重跑更早在复用的 provider fixture 状态处受阻。最新的整体浏览器重跑应视为未定论；上文 2026-09-24 的完整流程结果早于这次零剩余修正。
