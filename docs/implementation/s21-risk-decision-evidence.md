# S21 #81 RiskDecision 实现与验证

日期：2026-09-25。分支：`dev`。审查固定点：`96381f3`（RiskDecision 实现前）。

实现与本地验证已完成；最终 Standards/Spec 审查结论和 issue 关闭状态以 GitHub #81 记录为准。

## 实现范围

- 由 Rust Control Plane 按 workspace/proposal ID 求值；renderer 不提供 policy、账户、行情、检查结果或资格输入。
- 每次求值追加独立 `RiskDecision` 及 durable event，绑定 proposal hash、精确账户/环境、policy version、证据摘要和逐项检查；重开 workspace 后可读完整历史。
- 缺失或不可信证据返回 `UNAVAILABLE`。提交路径经相同 evaluator 阻止 `REJECTED` / `UNAVAILABLE`，且不产生模拟器或 provider I/O。
- Bitget 使用普通 `BITGET_LIVE` 身份；此切片不访问 Bitget Demo、Binance Testnet 或任何真实下单接口。

## 自动检查

- `cargo fmt --all -- --check`、`node scripts/ipc-schema.mjs --check`、`npm run typecheck`、`npm run test:unit`、`npm run build`、`python3 scripts/check_requirements.py`、`git diff --check`：通过。
- `cargo test --workspace --no-fail-fast -- --test-threads=1`：通过，146 个库级测试通过，workspace 集成测试无失败。
- 将状态优先级收敛为共享 helper 后，`cargo test --workspace risk_decision -- --test-threads=1`：3 个 RiskDecision 单元/提交守卫集成测试通过。
- 30 个 provider lifecycle 集成用例标记为 S21 忽略项：可信行情、市场日历或 instrument-rule evidence 尚未由所属数据工作项提供时，submit 保持 fail-closed。它们没有被计为通过。额外的 OS Keychain/native 环境测试按各自注释跳过。
- IPC schema 检查确认 Rust / JSON Schema / TypeScript 一致；需求追踪检查覆盖 203 条需求、70 个屏幕、13 个 QA 场景和 23 个基线文件。
- Vite 构建成功；其 bundle 超过 500 kB 的提示仍存在，不影响构建结果。

## 隔离浏览器验证

在临时 SQLite workspace 和 Rust-backed browser bridge 中验证：

- 打开 storage schema v23 workspace，生成 Local Paper proposal 并执行求值；页面显示 `UNAVAILABLE`、`POLICY_UNCONFIGURED`、绑定的 proposal hash 及检查/证据原因。
- 刷新页面后仍显示历史决策；使用键盘激活“Evaluate again”后新增第二条历史记录。
- 1280、768、390 像素下风险面板均可见，页面无横向溢出；未点击 submit。
- Workspace v23 与其 Snapshot 均通过 IPC validator。此前 validator 上限仍为 22 的不兼容已修正为 23，并重新生成协议产物。

该验证证明的是本地风险求值、持久化和界面行为，不证明 provider 行情授权、交易准备就绪或真实账户交易能力。验证期间未调用真实交易所 API，未创建 Bitget Demo 账户，也未发送订单。
