# S14 策略版本与受限策略进程证据

日期：2026-09-20

## 已实现边界

- `strategy.save_version` 在 workspace-scoped SQLite 中写入不可变 `StrategyVersion`；Rust 从规范化 definition 计算 `sha256:` hash，后续保存创建新 revision。
- `strategy.list`、`strategy.get`、`strategy.run`、`strategy.cancel` 使用 Backend ARD §41–42 的版本化 IPC；run 输入冻结 version/hash/instrument/dataset/time/parameters，结果只能是 `StrategySignal` 或 typed failure。
- 运行状态按 `QUEUED → RUNNING → COMPLETED|FAILED|CANCELLED` 持久化，取消只作用于 run；策略版本和已完成 run 不会被原地改写。
- 持久化读取会重新验证关联版本/hash、冻结请求、运行 identity、时间和 signal/failure 状态；工作区切换或重启会把遗留的 `QUEUED/RUNNING` run 收敛为带 remediation 的 `CANCELLED`，避免孤儿运行。
- production run 经过 `TimeService` trusted gate，并启动无继承环境的 `strategy-worker` duplex 子进程；握手校验协议版本和一次性 session token，stdout/超时有界，stderr 丢弃。当前 worker 没有真实 Python runtime 时返回 `STRATEGY_RUNTIME_UNAVAILABLE`，不会伪造 signal。
- 当前固定 worker 不执行用户源码，因此不会获得 Keychain、provider secret、Gateway、CLIProxyAPI、网络或 webview 句柄；真实 Python runtime 接入前生产路径始终 fail-closed。`TRADEX_STRATEGY_FIXTURE=1` 只在 `integration-test` feature 且值严格为 `1` 时启用，并在结果中显式标记。
- Strategies 页面从后端读取版本/运行状态，提供键盘可达编辑、保存、运行、取消、从历史记录恢复冻结请求后的重试和 `aria-live` signal/failure；页面没有 Trade、approval、reservation 或 provider 操作。

## 验证命令与结果

| 命令 | 结果 |
|---|---|
| `cargo test --workspace --features integration-test --test strategy` | PASS：不可变 revision、workspace scope、hash/dataset/time fail-closed、持久化 tamper rejection、active-run reconciliation |
| `cargo check --workspace --all-targets --all-features` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS：desktop/stdin worker 全特性编译和 lint |
| `npm run typecheck` | PASS |
| `npm run schema:check` | PASS：Rust / JSON Schema / TypeScript 一致 |
| `npm run build` | PASS |
| `TRADEX_STRATEGY_FIXTURE=1 cargo build --bin tradex-ipc --bin strategy-worker --features integration-test` + 临时 SQLite IPC 脚本 | PASS：save → run → `COMPLETED`，signal-only，fixture label 显式 |
| trusted time + `strategy-worker` 真实 duplex 子进程 IPC 脚本 | PASS：握手、token、bounded failure identity 校验；无 runtime 时 fail-closed |
| `npx tauri build --features desktop --bundles app --no-sign --config '{"bundle":{"active":true}}'` + 当前 SHA 的 macOS release bundle CUA | PASS：原生 Strategies 保存 immutable v1、重新选择并运行；同步 trusted time 后终态为 `FAILED / STRATEGY_RUNTIME_UNAVAILABLE`，未伪造 signal 或金融操作 |

## 未证明/后续边界

- 当前 worker seam 证明固定进程/协议/能力边界，macOS 正式启动链路使用 `/usr/bin/sandbox-exec` 拒绝用户/临时文件路径、网络和后续子进程；真实 Python 引擎、数值库和任意策略语义仍未接入，非 macOS 没有受支持的 OS sandbox 时生产路径 fail-closed 并返回 `STRATEGY_SANDBOX_UNAVAILABLE`。
- Strategies 已可从 Research/Backtest Thread context 进入并保留 canonical refs；浏览器脚本覆盖保存、成功 signal-only、失败、重试、取消、终态焦点、aria-live、390/768/1280 no-overflow 和空 console；当前 SHA 的 macOS release bundle 已完成原生保存、选择、trusted-time gate 和 fail-closed run 回归。
- integration fixture、worker 名称和本地脚本不构成真实外部数据、交易或 Live authority 证据。
