# S15 可复现历史回测规范

日期：2026-09-20  
前置：S08 TimeService/market calendar、S14 不可变 StrategyVersion 与受限 worker
状态：#48 已实现；#49/#50 待实现

## Problem Statement

TradeX 已能保存不可变策略版本并在受限 worker 边界上运行 signal-only 策略，但 Backtest Composer、策略编辑器和后端还没有共同的回测配置、冻结输入、持久化运行状态或比较结果。当前原型只有结果表面，Backtest Send 仍可能走研究路径，不能证明运行使用了哪个策略、数据、成本假设或时间边界。

没有冻结的配置会让用户修改编辑器、数据选择器或默认账户后改变旧结果；没有 reproducibility manifest 会让结果无法复核；没有明确的失败、取消和模型不可用边界会把确定性本地回测错误地绑定到 LLM 或 broker execution。回测必须保持本地、确定性和只读模拟边界，不能产生 proposal、approval、reservation、gateway 或 provider request。

## Solution

建立一个由 Rust Control Plane 所有的 typed backtest run flow：

- Backtest Send 与 Strategies editor 共用同一份 run configuration。用户必须选择已保存的 StrategyVersion，或先明确保存当前 draft；配置包含 canonical instrument、历史 dataset、日期范围、bar interval、starting cash、commission、slippage，以及可选的只读 portfolio seed。
- Run 前执行字段级校验和数据门禁；缺少 version、dataset、时间范围、有效金额/费率、可用历史覆盖或可信时间时阻塞，并保留用户输入。Agent 可以生成或修订策略，但 deterministic execution 之前必须落成并选择一个已保存版本。
- 点击 Run 时由后端计算 canonical input identity，冻结策略/version/hash、dataset/hash/provider、日期/时区、日历/调整假设、成本/滑点、starting cash、seed 和 engine/runtime version，并分配不可变 run ID。后续编辑只能创建新 run。
- 运行状态为 `QUEUED → RUNNING → COMPLETED | FAILED | CANCELLED`；终态不可回写。Cancel 只影响目标 run；Failed 保留冻结配置并提供 Retry，Retry 创建新 run identity，不覆盖失败记录。
- Completed 保存 return、Sharpe、Sortino、max drawdown、win rate、profit factor、turnover、equity curve、trade list 和 reproducibility manifest。金额、价格、数量和指标使用规范 decimal 字符串或有明确精度的序列化值；结果标记为历史模拟，不能显示为预期 live performance。
- Compare 只接受两个 `COMPLETED` run ID，展示 parameter/input differences、核心指标、曲线/交易数量和 dataset/manifest 差异；未知、非终态、跨 workspace 或同一 run 的请求被拒绝。
- 回测引擎消费 S14 不可变 StrategyVersion 和受控历史数据边界。模型不可用只阻止策略生成/修订 turn；已经冻结且配置有效的 deterministic backtest 仍可运行或如实报告数据/运行时不可用。任何 backtest command 都不进入 broker order flow。

## User Stories

1. 作为研究用户，我想从 Backtest Composer 或策略编辑器进入同一套配置，以免两条路径产生不同的运行语义。
2. 作为研究用户，我想选择已保存的策略版本并看到 version/hash，以便确认执行对象不是当前未保存草稿。
3. 作为研究用户，我想明确选择 instrument、dataset、日期范围和 bar interval，以便回测边界可复核。
4. 作为研究用户，我想设置 starting cash、commission、slippage 和可选只读 portfolio seed，以便结果包含完整成本假设。
5. 作为用户，我想在字段旁看到缺失、格式错误、覆盖不足、时区或可信时间阻塞原因，以便修正后再运行。
6. 作为用户，我想在 Run 后看到不可变 run ID、冻结摘要和 queued/running 进度，以便知道系统运行的确切输入。
7. 作为用户，我想取消指定的运行，以便长任务停止而不影响其他 run 或历史结果。
8. 作为用户，我想在失败后用原始冻结配置重试，以便失败记录仍可审计且重试不会偷偷换参数。
9. 作为用户，我想查看完整指标、equity curve、trade list 和 manifest，以便判断结果是否可复现。
10. 作为用户，我想比较两个已完成 run 的参数、指标、交易数量和数据身份，以便评估策略或成本变化。
11. 作为用户，我想确认修改策略、数据或任何 run 字段后会生成新 run，而不会改写旧结果。
12. 作为审计人员，我想看到 strategy/data hash、provider、retrieved_at、adjustment、timezone、calendar、commission、slippage、starting cash 和 engine version，以便重建运行条件。
13. 作为安全审查者，我想确认回测只能读取批准的历史数据和策略状态，不能读取 Keychain、broker secret、任意路径、网络或 Order Gateway。
14. 作为用户，我想在模型不可用时仍能执行已配置的 deterministic backtest，并在模型只负责生成策略时看到清晰边界。
15. 作为用户，我想看到 Backtest 明确是历史模拟，不提供下单、审批、reservation 或 provider action。
16. 作为键盘和窄屏用户，我想在 1280、768、390 宽度下配置、取消、重试和比较回测，状态变化有可访问文本和焦点回归。

## Implementation Decisions

- 使用 Backend ARD §41–42 的版本化 command/result envelope，增加或落实 `backtest.run`、`backtest.cancel`、`backtest.get`、`backtest.compare`；实现前先更新 schema、Rust 类型、JSON Schema 和 TypeScript client，禁止 renderer 自行生成 run ID、hash、manifest 或状态。
- 定义 bounded `BacktestConfig`、`BacktestRun`、`BacktestManifest`、`BacktestMetrics`、`EquityPoint`、`TradeRecord` 和 `BacktestComparison`。未知字段、过长文本、非法 decimal、反序列化 NaN/Infinity、跨 workspace refs 和重复 run refs 在持久化前拒绝。
- SQLite 保存 workspace-scoped 的 config snapshot、run identity、状态、错误、manifest 和小型指标投影；DuckDB 保持历史 dataset、equity/trade 分析和可扩展结果。Control Plane 是状态与权限的唯一 owner，前端只消费投影。
- run identity 对 canonical config、StrategyVersion/hash、dataset/hash、time/calendar/adjustment、cost/slippage、seed 和 engine/runtime version 做稳定 hashing；同一输入必须得到同一 identity，终态结果和 manifest 不可原地修改。
- 状态转移在事务中校验当前 version，Cancel 使用 run-scoped CAS；`QUEUED/RUNNING` 在 workspace 重开时按既定恢复策略收敛为可解释的 `CANCELLED` 或 `FAILED`，不得留下假运行。
- 引擎只读取经过 S06/S07/S08 边界确认的历史数据。未解决的 provider entitlement、许可、历史覆盖、调整、calendar 或数据缺口必须返回显式 unavailable/blocked reason，不能用 fixture 标签当作真实数据证据。
- look-ahead、survivorship、split、dividend、timezone mismatch 和 data gap 作为运行前或运行中的明确 guard；不满足 guard 时返回 typed failure，并保留冻结输入。
- Strategies 页面和 Backtest Thread 使用同一 query/mutation projection；Completed 页面提供 manifest、metrics、curve、trades 和 Compare 入口，但不提供 Trade、approval 或 provider action。
- integration fixture 仅用于稳定覆盖 success/failure/cancel/compare 的本地协议路径，并显式标记 `fixture`；production 仍依赖真实配置、批准数据集和可用 engine，缺失即 fail closed。
- 中文实现文档与英文权威 requirement/screen/QA IDs 保持同步；S15 不修改 PRD/UI/ARD 的规范含义，也不把原型覆盖状态升级为通过。

## Testing Decisions

- Rust unit tests 覆盖 canonical config/hash、decimal/field bounds、manifest completeness、look-ahead/timezone/gap guards、stable identity 和 immutable terminal rows。
- Control Plane integration tests 使用隔离临时 workspace/SQLite 验证 run/get/cancel/compare schema、workspace isolation、state CAS、重启收敛、失败重试、跨 run/非完成 compare 拒绝及 no-mutation 边界。
- Backtest engine tests 使用一个受控、固定的历史 dataset/strategy 输入，重复运行必须得到同一 manifest、identity、metrics、equity curve 和 trade list；negative cases 覆盖 dataset hash mismatch、survivorship/look-ahead、split/dividend、timezone 和 data gap。
- Worker/authority tests 证明回测不能获得 Keychain、broker secret、任意网络/文件、CLIProxyAPI 或 Order Gateway；模型不可用时已冻结 run 不会调用模型且仍能完成或给出非模型失败。
- Rust-backed browser/desktop tests 覆盖 editor→save/select→Run、Backtest Send 汇合、field errors、queued/running/cancelled/failed/retry/completed、manifest/compare、signal-only/无 broker action、aria-live、焦点恢复、1280/768/390 no-overflow 和空 console。
- 收尾运行 `npm run schema:check`、typecheck、build、unit、`cargo test --workspace`、integration feature、`cargo check --workspace --all-targets --all-features`、Clippy、fmt、`git diff --check` 和 `python3 scripts/check_requirements.py`；浏览器 fixture 不替代真实数据授权或 provider 验收。

## Out of Scope

- S16 Local Paper、S17–S20 provider Paper/Demo/Testnet 订单生命周期，以及任何 live broker order、approval、arming、reservation、Gateway 或 reconciliation。
- 外部数据订阅/许可的选择与购买、真实 provider entitlement、自动下载调度、云同步和移动端。
- 参数优化、walk-forward、Monte Carlo、组合优化、ML 训练、实时行情回测和未在 PRD MVP 中要求的订单类型。
- S14 策略版本与 OS sandbox 的重新实现；S15 只消费其不可变版本和受控 worker 边界。
- S33 全页面交叉回归、S34 资源目标/签名发布和 S35 总验收；本项只提供可被后续工作项复用的回测证据。

## Further Notes

- 权威来源：PRD §§35–40、§41.1、FR-031/054/066/074、AC-048/059、DATA-006；UI Spec §14.7；Backend ARD §§31.1–31.3、§32、§41–42；Frontend ARD FE-2/§10；Coverage FR-031/054/066/074、AC-048/059、QA-09。
- S15 的最低可用数据边界必须由 S06/S07/S08 的现有 source、calendar、adjustment 和 timezone 证据决定；没有真实数据授权时只能显示 blocked/unavailable，不能宣称完整 backtest ready。
- `strategy_version`、`strategy_hash`、`dataset_hash`、`data_provider`、`retrieved_at`、`adjustment_method`、`timezone`、`market_calendar_version`、`commission_model`、`slippage_model`、`starting_cash` 和 `engine_version` 是 manifest 的必需身份字段。
- 回测结果是历史模拟证据，不等价于 live performance；任何后续 signal→proposal→risk→approval→reservation→gateway 链路必须在 S16+ 重新定义和 gate。
