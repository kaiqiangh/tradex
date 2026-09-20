# S15 回测运行生命周期证据

日期：2026-09-21
规范起点：`cb378e5`  
#48 实现 SHA：`d5af7ba49f7e4c6bf5cf039dd293bd62a2562961`（`dev`）

## 本票范围

- Rust Control Plane 新增版本化 `backtest.run`、`backtest.get`、`backtest.cancel`，renderer 只能提交有界配置；run ID、状态、identity、observed time 和 state version 由后端生成。
- SQLite 以 workspace 为边界保存冻结的 StrategyVersion/hash、instrument/dataset、日期、bar interval、starting cash、commission、slippage、seed、状态和 typed failure；读取会重新校验 hash、配置、时间、参数和状态不变量。
- 运行生命周期覆盖 `QUEUED → RUNNING → FAILED|CANCELLED`，run-scoped cancel、失败后以原请求 retry（分配新 run ID）以及 workspace 重开时将遗留 `QUEUED/RUNNING` 收敛为 `CANCELLED`。
- Backtest Thread 与 Strategies 页面复用同一 `BacktestRunPanel`，要求用户明确选择已保存 strategy version，提供 instrument/dataset、日期范围、bar interval、成本/滑点、seed、状态、run ID、request hash、失败 remediation、retry/cancel 和 `aria-live`；结果从后端投影渲染完整 Frozen configuration，字段和后端错误均显示 `aria-invalid`/field message，页面明确历史模拟且不提供 broker/order/approval/reservation 操作。
- 后端在 request identity、worker input 和 SQLite projection 三个边界统一使用规范化 decimal；负向测试覆盖未知字段、stale cancel CAS 不变更、跨 workspace 读取隔离和重开收敛。
- Run 前检查历史覆盖范围（fixture 为 2000–2100 UTC）和 OD-002 entitlement；不可用或越界日期在保存前返回带 `startAt`/`endAt`/`datasetId` 的 `MARKET_HISTORY_UNAVAILABLE`，不会创建 run。省略参数会先解析为保存版本的默认参数，再同时冻结到 projection 和 worker input。
- 浏览器自动脚本收集 warn/error console 并断言为空；手动观察与自动断言保持同一条隔离 Rust bridge 线路。
- Backend ARD §41.1.1 及中文对应段落定义了版本 1 的 backtest payload。request identity 排除观测时间；SQLite transition 使用 run-scoped state-version CAS，终态 projection 不可变，cancel 只接受返回的 `expectedStateVersion`。
- integration fixture 只在 `integration-test` feature 且 `TRADEX_BACKTEST_FIXTURE=1` 时启用，并在结果显示 `TRADEX_BACKTEST_FIXTURE`。生产路径先在没有可用历史 entitlement/覆盖时返回 `MARKET_HISTORY_UNAVAILABLE`；通过数据门禁后若没有回测引擎才返回 `BACKTEST_RUNTIME_UNAVAILABLE`，不伪造完成结果。

## 验证结果

| 检查 | 结果 |
|---|---|
| `cargo test --workspace --all-features` | PASS：105 个库测试及全部 workspace 集成测试通过；backtest 测试覆盖输入拒绝、规范化 decimal、历史覆盖/entitlement 门禁、保存版本默认参数、未知字段、typed fixture failure、SQLite 保存/get、stale cancel CAS 无变更、跨 workspace 隔离、终态不可取消、重开收敛 |
| `cargo check --workspace --all-targets --all-features` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo fmt --all -- --check`、`git diff --check` | PASS |
| `npm run schema:check`、`npm run typecheck`、`npm run build` | PASS；Rust / JSON Schema / TypeScript 一致 |
| `npm run test:unit` | PASS：6 个 projection/schema 测试 |
| `python3 scripts/check_requirements.py` | PASS：201 requirements、70 screens、12 QA scenarios、23 baseline files |
| `node --check tests/strategy-ui.mjs` | PASS；脚本包含 warn/error console 为空断言 |

## 隔离浏览器证据

2026-09-20 在 `npm run dev:browser` 的真实 Rust stdio/SQLite bridge 中使用临时 workspace `9fdc9dea-5f0f-400b-a7c5-2bdbef8c6ffe`，通过 Thread 与 Strategies 两个入口保存版本并验证 backtest：

- Thread Backtest panel 明确选择 saved version 后提交 `backtest.run`，失败显示 `FAILED / BACKTEST_FIXTURE_FAILED`；同一面板的取消使用 state-version CAS 并得到 `CANCELLED / BACKTEST_CANCELLED`。
- `FAILURE` 返回 `FAILED / BACKTEST_FIXTURE_FAILED`、冻结配置和 remediation；Retry 保留原配置并生成新的 run ID。
- 手动字段校验清空 Instrument 后显示字段错误并设置 `aria-invalid=true`；恢复输入后可继续运行。
- `CANCELLED` 进入 `RUNNING`，随后只取消目标 run，得到 `CANCELLED / BACKTEST_CANCELLED`；终态焦点恢复到 `Retry backtest`。
- 结果显示 run ID、request hash 和 fixture label；页面没有 Trade、Approve、Reserve 或 provider 操作。
- 1280、768、390 视口的 `scrollWidth` 分别为 `1265/1265`、`753/753`、`375/375`；浏览器 error 日志为零。

该 fixture 只证明本地 schema、状态、持久化、UI 和取消线路；它不证明真实历史数据授权、回测引擎指标、equity curve、trade list、manifest 或 compare。这些属于 #49 和 #50。原生 macOS 窗口本票未重复运行，S14 的原生 Strategies 证据仍单独记录。
