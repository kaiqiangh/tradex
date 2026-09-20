# S15 回测运行生命周期证据

日期：2026-09-20  
规范起点：`cb378e5`  
#48 实现 SHA：`edbf197d1f93af3256114d896d286d80d1846fba`（`dev`）

## 本票范围

- Rust Control Plane 新增版本化 `backtest.run`、`backtest.get`、`backtest.cancel`，renderer 只能提交有界配置；run ID、状态、identity、observed time 和 state version 由后端生成。
- SQLite 以 workspace 为边界保存冻结的 StrategyVersion/hash、instrument/dataset、日期、bar interval、starting cash、commission、slippage、seed、状态和 typed failure；读取会重新校验 hash、配置、时间、参数和状态不变量。
- 运行生命周期覆盖 `QUEUED → RUNNING → FAILED|CANCELLED`，run-scoped cancel、失败后以原请求 retry（分配新 run ID）以及 workspace 重开时将遗留 `QUEUED/RUNNING` 收敛为 `CANCELLED`。
- Strategies 页面提供与策略版本共用的 instrument/dataset 和 backtest 配置字段、日期范围、成本/滑点、seed、状态、run ID、request hash、失败 remediation、retry/cancel 和 `aria-live`；页面明确历史模拟且不提供 broker/order/approval/reservation 操作。
- integration fixture 只在 `integration-test` feature 且 `TRADEX_BACKTEST_FIXTURE=1` 时启用，并在结果显示 `TRADEX_BACKTEST_FIXTURE`。生产路径没有回测引擎时返回 `BACKTEST_RUNTIME_UNAVAILABLE`，不伪造完成结果。

## 验证结果

| 检查 | 结果 |
|---|---|
| `cargo test --workspace --all-features` | PASS：105 个库测试及全部 workspace 集成测试通过；新增 backtest 测试覆盖输入拒绝、typed runtime failure、SQLite 保存/get、终态不可取消、重开收敛 |
| `cargo check --workspace --all-targets --all-features` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo fmt --all -- --check`、`git diff --check` | PASS |
| `npm run schema:check`、`npm run typecheck`、`npm run build` | PASS；Rust / JSON Schema / TypeScript 一致 |
| `python3 scripts/check_requirements.py` | PASS：201 requirements、70 screens、12 QA scenarios、23 baseline files |
| `node --check tests/strategy-ui.mjs` | PASS |

## 隔离浏览器证据

2026-09-20 在 `npm run dev:browser` 的真实 Rust stdio/SQLite bridge 中使用临时 workspace `914d5407-c075-43ec-ba5b-ac7f9b7d64b9`，通过 Strategies 页面保存版本并验证 backtest：

- `FAILURE` 返回 `FAILED / BACKTEST_FIXTURE_FAILED`、冻结配置和 remediation；Retry 保留原配置并生成新的 run ID。
- `CANCELLED` 进入 `RUNNING`，随后只取消目标 run，得到 `CANCELLED / BACKTEST_CANCELLED`；终态焦点恢复到 `Retry backtest`。
- 结果显示 run ID、request hash 和 fixture label；页面没有 Trade、Approve、Reserve 或 provider 操作。
- 1280、768、390 视口的 `scrollWidth` 分别为 `1265/1265`、`753/753`、`375/375`；浏览器 error 日志为零。

该 fixture 只证明本地 schema、状态、持久化、UI 和取消线路；它不证明真实历史数据授权、回测引擎指标、equity curve、trade list、manifest 或 compare。这些属于 #49 和 #50。原生 macOS 窗口本票未重复运行，S14 的原生 Strategies 证据仍单独记录。
