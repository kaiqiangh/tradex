# S04 #19 Turn 取消、重试与安全恢复验收

状态：**#19 实现完成并关闭；S04 父 Spec #16 仍 OPEN**。实现固定 SHA：`e544a3a16f96ede907fcbba136e72dd2090f4a68`。

## 固定实现

- 主实现：`fcdbd08`（受控后台 supervisor、取消/重试 IPC、运行时终态、恢复协调、React 操作入口）。
- 边界修复：`79913ba`（有界帧轮询）、`16f81c5`（冲突序列重复 fail closed）、`f4c4cde`（取消请求优先于迟到完成）、`e544a3a`（重开后迟到完成不重复写模型 attempt）。
- Backend ARD §41 v1 envelope、§42 event envelope、SQLite Thread/outbox projection 和生成的 Rust/JSON Schema/TypeScript/AJV validators 保持同一版本。

## 已交付行为

- `turn.start` 先持久化 user message、`RUNNING` Turn、不可变 snapshot 和 provider attempt，再由 `RuntimeSupervisor` 在受控后台线程启动 Codex runtime；`turn.cancel`/`turn.retry` 通过同一 canonical envelope 校验 workspace、Thread、Turn 和 state version。
- 取消只接受 `RUNNING` Turn，持久化 `cancelRequestedAt`，信号运行时映射到 Codex `turn/interrupt`，最终落为 `CANCELLED` 或进程故障对应的 `INTERRUPTED`；取消请求后的 delta、completion 和旧 worker 事件不会重开或修改该 Turn。
- 上游 EOF、超时、非法/过大帧、协议错误、模型故障和有界队列溢出都保留脱敏 error item、结束 provider attempt 并释放子进程；冲突的上游 sequence 重复返回 `CODEX_EVENT_GAP`。
- Retry 只允许 `FAILED`、`CANCELLED` 或 `INTERRUPTED`，总是生成新的 Turn ID、snapshot 和 provider attempt；旧 timeline、错误和 attempt 不会被覆盖。重开 workspace 会把持久化的 `RUNNING` Turn 安全标为 `INTERRUPTED`，不恢复审批、预留、市场有效性或 ARMED 状态。

## 验证

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo clippy --workspace --all-targets --features integration-test -- -D warnings`：通过。
- `cargo test --workspace --features integration-test -- --test-threads=1`：34 个 library tests 和所有 integration target 的非 ignored tests 通过；原生 Keychain/真实 gateway 检查按仓库约定保持 ignored。
- `npm run check`：schema、TypeScript、Vite build、4 个前端单元测试、默认 Rust workspace tests 和 `python3 scripts/check_requirements.py` 通过；构建保留既有单 bundle 大于 500 kB warning。
- `tests/thread-ui.mjs` 通过最终 `e544a3a` 的隔离 Rust-backed browser：创建 Thread、观察 `RUNNING`、取消为 `CANCELLED`、错误 item/`CANCELLED` provider attempt、Retry 创建并完成新 Turn、重载后两轮历史保持，以及 768/390 无横向溢出和零浏览器 error。测试只使用隔离临时 workspace 与合成模型状态。
- Rust 回归覆盖取消完成竞态、重开后迟到 worker、Codex `turn/interrupt`、严格序列/重复事件、fake stream 持久化和 restart reconciliation；`codex-cli 0.154.0` 只完成隔离 `initialize`/`thread/start`/`turn/start` frame-shape smoke。

## 保留边界

- 当前证据没有把未认证的真实模型推理、跨进程 Codex session resume、完整 tool/result duration/summary 或 QA-01–12/S33 交叉回归标为通过。生产适配器仍按一次受限 stdio 会话运行，并将这些外部/后续范围留给对应切片。
- 没有读取、删除或写入真实账户/模型凭据，也没有触达外部金融执行；Ask/Research 失败路径不会进入 FinancialApproval、Order Gateway 或 broker。

## 串行审查

- **Standards：PASS。** `AGENTS.md`、`docs/agents/*`、格式、Clippy、schema、typecheck、build、unit、workspace 和 traceability 检查均通过；未发现硬性规范违规。
- **Spec：PASS（#19 已实现范围）。** 取消/中断/失败恢复/重试、终态守卫、版本冲突、序列边界、持久化与 UI 状态符合 PRD、UI Spec §14 和 Backend ARD §41–42；未覆盖边界已在上方明确保留。
