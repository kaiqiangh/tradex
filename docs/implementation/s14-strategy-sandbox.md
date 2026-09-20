# S14 策略版本与受限策略沙箱规范

日期：2026-09-20
前置：S01 工作区、S07 canonical market/watchlists、S08 TimeService/market gate
状态：待实现

## Problem Statement

TradeX 当前能够保存市场、账户、研究和订单上下文，但没有受控的策略版本与策略 worker。用户无法把编辑中的策略冻结成可复现版本，也无法验证策略代码在没有 broker/model 秘密、Keychain、网络或 Order Gateway 的环境中运行。原型的 Strategy Editor、Strategy List 和 signal-only 边界尚未形成真实 IPC、持久化或进程隔离证据。

如果策略版本、源码 hash、运行输入和 worker 能力边界不由 Rust Control Plane 固定，后续 S15 回测或 S16–S30 交易路径就无法证明运行的是哪一个策略，也无法证明策略代码不能越权。策略输出必须是受限、可审计的 signal；它不能直接变成订单、approval、reservation 或 provider request。

## Solution

提供 workspace-scoped 的策略版本库与一次受限策略运行垂直切片：

- 用户可以创建、查看和保存策略版本。每个版本拥有不可变 version ID、strategy ID、revision、canonical source hash、参数 schema、创建时间、语言/runtime 标识和 bounded source metadata；保存后的版本不能被原地改写。
- 编辑器保存前显示 draft，保存后显示版本 identity/hash 和可比较的参数；当前选择器、模型或账户变化不会修改已保存版本。
- Rust Control Plane 通过已有的受控 inherited-duplex worker seam 启动策略 worker。worker 使用版本化握手和一次性 session credential，不能监听 TCP，也不能把句柄传给 webview、Codex、CLIProxyAPI 或 broker adapter。
- worker 只获得显式批准的历史数据集句柄、数值库、策略状态和 workspace 策略文件；拒绝 Keychain、broker/account secrets、Order Gateway、任意网络、任意文件路径和直接模型推理。环境、资源上限和超时由 Control Plane 固定。
- strategy.run 冻结 version ID/hash、canonical instrument/context refs、dataset ref、时间范围和参数，返回 QUEUED/RUNNING/COMPLETED/FAILED/CANCELLED 状态及有限的 StrategySignal/failure evidence。成功输出只有 signal（instrument、direction、desired exposure、observed time、strategy version/hash、source/dataset refs），不带 order ID、approval ID、reservation ID 或 gateway capability。
- UI 提供 Strategy list/editor/run surface：draft、saved versions、selected version/hash、运行状态、失败原因、取消/重试和 signal-only 结果使用文本、semantic status、键盘焦点和窄屏布局；策略页可从 Research/Backtest context 进入，不增加未经规范要求的金融主导航。
- 所有 persisted state、worker result 和错误均经过 Backend ARD §41–42 的 schema/size/control-character/canonical identity 验证。没有有效 version、dataset 或 trusted time 时运行被阻塞并给出 remediation；fixture 只能走 integration seam 并显式标记。

## User Stories

1. 作为研究用户，我想编辑策略草稿并保存不可变版本，以便后续回测能引用精确的策略身份。
2. 作为研究用户，我想看到源码 hash、revision、参数和创建时间，以便判断两个版本是否相同。
3. 作为研究用户，我想在保存前修改策略而不污染历史版本，以便旧结果保持可复现。
4. 作为研究用户，我想从策略列表重新打开已保存版本，以便继续编辑为新版本或运行原版本。
5. 作为用户，我想看到策略运行的 queued/running/completed/failed/cancelled 状态，以便知道 worker 是否仍在执行。
6. 作为用户，我想取消运行并在失败后重试冻结的输入，以便不必重新构造策略配置。
7. 作为用户，我想看到 signal-only 结果及其 instrument、direction、desired exposure、strategy hash、dataset/source refs 和 observed time，以便审计输出来源。
8. 作为用户，我想在无有效策略版本、数据集或可信时间时看到明确阻塞原因，而不是看到成功信号。
9. 作为安全审查者，我想确认策略代码不能读取 broker/model secrets、OS Keychain、私有账户数据或 Order Gateway。
10. 作为安全审查者，我想确认策略 worker 不能访问任意网络、路径或外部 LLM endpoint。
11. 作为用户，我想确认 strategy signal 不会自动创建 order、proposal、approval、reservation 或 provider request。
12. 作为用户，我想在 1280、768 和 390 宽度下查看版本、状态和失败 remediation，不发生横向溢出。
13. 作为键盘用户，我想使用语义控件编辑、保存、运行、取消和重试策略，并在状态变化时保留可见焦点。
14. 作为审计人员，我想让相同的 version/hash、dataset ref、参数和 engine/runtime 输入得到稳定的运行 identity 和有限结果。
15. 作为后续回测用户，我想让 S15 直接消费不可变 StrategyVersion，而不是复制 renderer 中的源码或当前编辑状态。

## Implementation Decisions

- 增加 typed StrategyDefinition、不可变 StrategyVersion、StrategyRun、StrategySignal、StrategyRunState、StrategyFailure 和 StrategyQuery/save/run/cancel/get/list IPC 契约；所有集合、文本、参数和 signal 数量有上限。
- strategy version hash 在 Rust Control Plane 从 canonical source、normalized parameter schema、language/runtime metadata 计算；renderer 不能提交或覆盖 hash、version ID、run state、observed time 或 authority fields。
- SQLite 保存 workspace、strategy metadata/version/run state、canonical hashes 和 bounded result metadata；大型历史数据仍属于 DuckDB/data-source boundary。保存 draft 不会改变已完成 run。
- worker 启动复用现有受控子进程/继承 duplex 设计：父进程保留唯一句柄，握手先验证 protocol version 和一次性 session credential，再接受有限请求；worker stdout/stderr、环境和退出原因不得泄露秘密。
- worker capability allowlist 只包括 approved dataset read、数值运算、strategy state 与 workspace strategy file；filesystem、network、Keychain、broker adapter、Order Gateway、CLIProxyAPI 和 arbitrary subprocess 全部拒绝。策略运行输出只能被 Control Plane 转换成 bounded signal 或 typed failure。
- strategy.run 在运行前重新验证 workspace、version/hash、canonical instrument/dataset refs、TimeService confidence 和 parameter bounds，并冻结 run input；运行中取消只影响该 run，不改变 version 或其他 workspace state。
- run identity/hash 包含 strategy version、dataset/source refs、instrument/context refs、parameters、time assumptions 和 engine/runtime version；相同输入必须得到相同 identity，但结果仍标记为 simulation/strategy signal。
- UI 通过 context-driven Strategies surface 和 React Query 读取 backend-owned library；版本列表、editor、run status、signal card 和 error recovery 不能自行推断 authority。Signal card 不提供 Trade/approval 操作。
- integration mode 可使用 TRADEX_STRATEGY_FIXTURE=1 产生固定成功、失败和取消场景，但 production Control Plane 没有 synthetic strategy source。fixture label、worker restriction 和 unavailable reason 必须显式显示。
- 中英文契约、生成的 JSON Schema/TypeScript、requirements/surfaces traceability 与 S14 evidence 文档同步更新；不把 fixture、截图或 label 计为真实 provider/runtime 证据。

## Testing Decisions

- Rust unit tests 验证 canonical source/parameter hashing、版本不可变性、长度/control-character/unknown-field 拒绝、bounded signal/failure、workspace isolation 和相同输入的稳定 identity。
- ControlPlane integration tests 验证 strategy list/save/get/run/cancel 的 schema、state transitions、no-mutation boundaries、version/hash tamper rejection、unknown workspace/dataset/instrument 和 untrusted TimeService fail-closed。
- Worker integration tests 使用真实受控子进程尝试读取 Keychain/broker-like secret paths、任意文件、网络、CLIProxyAPI 和 Gateway；每次都应被拒绝，并确认 secrets 不进入 args/env/logs/result。
- Frontend unit/browser tests 验证 draft→save version、reopen、version selection、run/cancel/retry/failed/signal states、keyboard focus、aria-live、390/768/1280 no-overflow 和空 console warnings/errors。
- 运行 schema/typecheck/build/unit、workspace Rust、integration feature、Clippy、fmt、diff check 和 requirements traceability；Rust-backed browser 使用隔离 SQLite/workspace fixture。
- 测试只证明本地契约、worker containment 和 signal-only authority。真实 Python/runtime hardening、外部 data license、完整 S15 backtest metrics、Live authority consumers 和 S33 cross-page regression 不由 S14 代替。

## Out of Scope

- 完整 deterministic backtest engine、OHLCV ingestion、equity curve、trade list、metrics、compare 和 reproducibility manifest（S15）。
- Local Paper 或 provider-hosted Paper/Demo/Testnet order lifecycle（S16–S20）。
- Risk policy、arming、approval、reservation、Order Gateway、reconciliation、cancel/fill race 和 Live strategy dispatch（S21–S30）。
- 真实 provider entitlement、付费数据许可、自动策略调度、云端同步、移动端执行和签名发布包。
- 让 agent 或策略代码直接发起 broker/model/network 请求；任何 signal 到交易的后续链路必须由独立工作项定义并再次 gate。

## Further Notes

- 权威来源：PRD §§39–41、40.1、41.1、67、69；UI Spec B4/H1–H6/§14.7；Backend ARD §§18、31.1–31.3、32、40.3、43；Frontend ARD §§18–20、FE-2；Coverage FR-031/032/037/054、AC-027/048/059、SEC-004/007、DATA-006、QA-09。
- S14 依赖 S01 workspace/process boundary 与 S07 canonical market/data context；S08 TimeService gate 作为 run freshness/TTL 输入，但 S14 不实现 Live approval。
- S15 必须消费不可变 StrategyVersion 和 S14 run identity；不能从当前 editor 状态或 renderer-supplied hash 重新解释策略。
- 任何真实 sandbox limitation、平台差异或 runtime dependency 都必须在 evidence 中标记为 RUNTIME_PENDING/IMPLEMENTED_UNVERIFIED，不能因 subprocess 名称或 fixture output 宣称完成。
