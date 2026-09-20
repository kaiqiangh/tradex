# S14 策略版本与受限策略进程证据

日期：2026-09-20

## 已实现边界

- `strategy.save_version` 在 workspace-scoped SQLite 中写入不可变 `StrategyVersion`；Rust 从规范化 definition 计算 `sha256:` hash，后续保存创建新 revision。
- `strategy.list`、`strategy.get`、`strategy.run`、`strategy.cancel` 使用 Backend ARD §41–42 的版本化 IPC；run 输入冻结 version/hash/instrument/dataset/time/parameters，结果只能是 `StrategySignal` 或 typed failure。
- 运行状态按 `QUEUED → RUNNING → COMPLETED|FAILED|CANCELLED` 持久化，取消只作用于 run；策略版本和已完成 run 不会被原地改写。
- production run 经过 `TimeService` trusted gate，并启动无继承环境的 `strategy-worker` duplex 子进程；握手校验协议版本和一次性 session token，stdout 有界，stderr 丢弃，父进程只接收身份匹配的 signal。
- worker 没有 Keychain、provider secret、Gateway、CLIProxyAPI、网络或 webview 句柄。`TRADEX_STRATEGY_FIXTURE=1` 只在 `integration-test` feature 下启用，并在结果中显式标记。
- Strategies 页面从后端读取版本/运行状态，提供键盘可达编辑、保存、运行、取消和 `aria-live` signal/failure；页面没有 Trade、approval、reservation 或 provider 操作。

## 验证命令与结果

| 命令 | 结果 |
|---|---|
| `cargo test --workspace --features integration-test --test strategy` | PASS：不可变 revision、workspace scope、hash/dataset/time fail-closed |
| `npm run typecheck` | PASS |
| `npm run schema:check` | PASS：Rust / JSON Schema / TypeScript 一致 |
| `npm run build` | PASS |
| `TRADEX_STRATEGY_FIXTURE=1 cargo build --bin tradex-ipc --bin strategy-worker --features integration-test` + 临时 SQLite IPC 脚本 | PASS：save → run → `COMPLETED`，signal-only，fixture label 显式 |
| trusted time + `strategy-worker` 真实 duplex 子进程 IPC 脚本 | PASS：握手、token、bounded signal identity 校验 |

## 未证明/后续边界

- 当前 worker seam 证明进程/协议/能力边界，未证明真实 Python 引擎、数值库和任意策略语义；这是 S15/runtime hardening 的后续证据。
- 尚未完成 390/768/1280 的自动浏览器截图与桌面原生窗口回归；本项 UI 契约已接入现有 shell，需在 Wayfinder 验证环境继续跑。
- integration fixture、worker 名称和本地脚本不构成真实外部数据、交易或 Live authority 证据。
