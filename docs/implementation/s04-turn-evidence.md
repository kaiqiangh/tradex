# S04 Turn 流式运行与不可变来源验收

状态：**#18/#19 实现完成并关闭；S04 父 Spec #16 仍 OPEN**。取消、重试与安全恢复的逐项证据见 [S04 #19 验收](s04-cancel-retry-evidence.md)。

## 固定实现

- 起始审查 SHA：`45501e9`（#17 已完成后的 S04 固定点）。
- #18 实现 SHA：`09122d2`（Thread/Turn/Item IPC、Rust 控制面、Codex 适配器、React 时间线）、`f3cebe9`（无换行 JSONL 长帧的有界读取）和 `f3d574f`（Clippy 清理）。
- #19 实现固定 SHA：`e544a3a16f96ede907fcbba136e72dd2090f4a68`；取消/重试/恢复证据见 [S04 #19 验收](s04-cancel-retry-evidence.md)。
- Rust schema、JSON Schema、TypeScript 类型和 AJV validators 已由同一生成流程同步。

## 交付范围

- `turn.start` 只接受 Backend ARD §41 v1 envelope；Rust 在启动上游进程前校验 Thread、state version、模式/上下文、账户/环境、canonical context refs 和已验证模型路由。
- `codex_runtime` 以单次受限 stdio JSONL 会话执行一次 `initialize`/`initialized`，再调用 `thread/start` 或 `thread/resume` 与 `turn/start`。子进程环境清空，只保留临时 0700 HOME/CODEX_HOME、loopback `127.0.0.1:8317/v1` 和短生命周期 gateway key；Renderer 不接触上游 RPC。
- User message 与 `RUNNING` Turn 先写入 SQLite Thread projection/outbox。上游 `thread/started`、`turn/started`、item start/delta/completed 和 completion 被映射为严格递增的 Thread 更新；重复事件幂等，序列跳跃、错误、EOF、超时、非法/过大帧和不同 Thread/Turn 的通知 fail closed。
- `turn.cancel` 只接受 `RUNNING` Turn，持久化取消请求并向受控运行时发送 `turn/interrupt`；取消、进程退出、超时、背压和协议错误都有明确终态及脱敏 error item。`turn.retry` 生成新的 Turn/snapshot/provider attempt，工作区重开会安全中断遗留的 `RUNNING` Turn，迟到旧事件不能更新新 Turn。
- Turn snapshot 冻结 mode、Execution Context、account/environment、capability、model/provider、context IDs/hashes 和时间；provider attempt、item 状态和 completion/error 在快照外追加。通用 Codex approval 只成为 `codex_approval` timeline item，不进入金融审批链。
- Threads 页面显示运行中状态、typed timeline、来源与 provider attempt；已有 subscription/replay/reload 机制继续用于历史恢复和 gap 收敛。

## 验证

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo test --workspace`：通过；默认 Rust library/workspace tests 全部非 ignored 项通过，原生 Keychain/真实 gateway 测试按仓库约定保持 ignored。
- `cargo test --workspace --features integration-test -- --test-threads=1`：通过，34 个 library tests 和所有 integration target 的非 ignored tests 通过；取消完成竞态、重开迟到 worker、Codex interrupt、retry 和 reconciliation 回归均通过。
- `npm run schema:check`、`npm run typecheck`、`npm run build`、`npm run test:unit`、`python3 scripts/check_requirements.py`：通过。构建保留既有单 bundle >500 kB warning。
- 隔离 Rust-backed browser `tests/thread-ui.mjs` 于 2026-09-13 在实现固定 `e544a3a` 的 `http://127.0.0.1:1420/` 通过：Thread create、RUNNING、fake streaming、取消为 `CANCELLED`、Retry 新 Turn、provider attempt、reload 后历史，以及 768px/390px 无横向溢出、焦点/语义状态和零浏览器 error。
- 当前本机 `codex-cli 0.154.0` 的隔离 smoke 已完成 `initialize`/`initialized`、`thread/start`、`turn/start` frame shape 检查；请求只走 loopback gateway。smoke 中上游 websocket 因无认证推理而拒绝，因此没有把它记录为真实模型 inference 通过。

## 未覆盖边界

- 上游跨进程 Thread resume 所需的持久 Codex session 目录、真实 authenticated inference、完整 tool/result duration/summary 和 QA-01–12/S33 交叉回归仍未声称完成。
- 没有删除真实账户、写入用户凭据或触达外部金融执行；Trade/Live authority、FinancialApproval、Order Gateway 仍由后续切片负责。
