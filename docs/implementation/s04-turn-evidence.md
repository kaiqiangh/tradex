# S04 Turn 流式运行与不可变来源验收

状态：**#18 实现完成；#19 取消/重试已解除阻塞；S04 父 Spec #16 仍 OPEN**。

## 固定实现

- 起始审查 SHA：`45501e9`（#17 已完成后的 S04 固定点）。
- #18 实现 SHA：`09122d2`（Thread/Turn/Item IPC、Rust 控制面、Codex 适配器、React 时间线）、`f3cebe9`（无换行 JSONL 长帧的有界读取）和 `f3d574f`（Clippy 清理）。
- Rust schema、JSON Schema、TypeScript 类型和 AJV validators 已由同一生成流程同步。

## 交付范围

- `turn.start` 只接受 Backend ARD §41 v1 envelope；Rust 在启动上游进程前校验 Thread、state version、模式/上下文、账户/环境、canonical context refs 和已验证模型路由。
- `codex_runtime` 以单次受限 stdio JSONL 会话执行一次 `initialize`/`initialized`，再调用 `thread/start` 或 `thread/resume` 与 `turn/start`。子进程环境清空，只保留临时 0700 HOME/CODEX_HOME、loopback `127.0.0.1:8317/v1` 和短生命周期 gateway key；Renderer 不接触上游 RPC。
- User message 与 `RUNNING` Turn 先写入 SQLite Thread projection/outbox。上游 `thread/started`、`turn/started`、item start/delta/completed 和 completion 被映射为严格递增的 Thread 更新；重复事件幂等，序列跳跃、错误、EOF、超时、非法/过大帧和不同 Thread/Turn 的通知 fail closed。
- Turn snapshot 冻结 mode、Execution Context、account/environment、capability、model/provider、context IDs/hashes 和时间；provider attempt、item 状态和 completion/error 在快照外追加。通用 Codex approval 只成为 `codex_approval` timeline item，不进入金融审批链。
- Threads 页面显示运行中状态、typed timeline、来源与 provider attempt；已有 subscription/replay/reload 机制继续用于历史恢复和 gap 收敛。

## 验证

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo test --workspace`：通过，29 个 Rust library tests、全部非 ignored workspace integration tests 通过；原生 Keychain/真实 gateway 测试按仓库约定保持 ignored。
- `cargo test --lib --features integration-test turn_runtime_tests::fake_app_server_stream_is_persisted_before_completion -- --exact`：通过，验证 fake stream 在 completion 前持久化，含 context ID/hash、typed item、provider attempt 和严格事件序列。
- `npm run schema:check`、`npm run typecheck`、`npm run build`、`npm run test:unit`、`python3 scripts/check_requirements.py`：通过。构建保留既有单 bundle >500 kB warning。
- 隔离 Rust-backed browser `tests/thread-ui.mjs` 于 2026-09-13 在 `http://127.0.0.1:1420/` 通过：Thread create、RUNNING、fake streaming、provider attempt、reload 后历史，以及 768px/390px 无横向溢出、焦点/语义状态和零浏览器 error。
- 当前本机 `codex-cli 0.154.0` 的隔离 smoke 已完成 `initialize`/`initialized`、`thread/start`、`turn/start` frame shape 检查；请求只走 loopback gateway。smoke 中上游 websocket 因无认证推理而拒绝，因此没有把它记录为真实模型 inference 通过。

## 未覆盖边界

- `turn.cancel`、`turn.retry`、`turn/interrupt`、取消后的状态和 crash recovery 由已解除阻塞的 #19 继续实现。
- 上游跨进程 Thread resume 所需的持久 Codex session 目录、真实 authenticated inference、完整 tool/result duration/summary 和 QA-01–12/S33 交叉回归仍未声称完成。
- 没有删除真实账户、写入用户凭据或触达外部金融执行；Trade/Live authority、FinancialApproval、Order Gateway 仍由后续切片负责。
