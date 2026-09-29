# S24 #95 独立 Privileged Live Order Gateway 验收

日期：2026-09-27。分支：`dev`。实现代码 SHA：`e35d42e3213466751fa4cbe2b61b577ad5c96c28`。固定审查基线：`c888648b78e3aa1967b40c21fb6e1d470fd7059d`。

## 实现边界

- `trade.execution.prepare` 是唯一公开的 PLACE/CANCEL authority entry。Control Plane 持有 SQLite 写权限，在准备结果持久化后启动单独构建和摘要校验的 `tradex-order-gateway` 子进程；私有继承双向通道完成有界版本握手与随机会话认证，没有 TCP listener 或 renderer/Agent Gateway command。
- Control Plane 对精确 `RESERVED` attempt 重新读取审批、提案/撤单意图、账户、风险和当前资格，发出绑定 attempt/account/operation/hash/approval/reservation/authority versions/session 的单次 grant。每账户串行化撤销、Disarm、Disable All 与 dispatch；`SUBMITTING` 和审计/outbox 持久化在 provider I/O 前提交。
- 支持 Trading 212 Live 的受限 PLACE/CANCEL provider routes。PLACE acceptance 显示 `ACCEPTED` 而非 fill；CANCEL acknowledgement 显示 `CANCEL_PENDING`、保留原订单 commitment，不伪称 `CANCELLED`。派发后无法核验的结果进入 `UNKNOWN_RECONCILING`，保留容量且不能重发；明确拒绝保留 `REJECTED`。
- 英文/中文 Backend ARD、Frontend ARD 和 UI Spec 同步描述子进程、wire/event/state 边界、Prepare 披露、状态语义及 accessible status；§41–42 仍是共享 IPC/event 契约来源。

## 验证

- `npm run check` **PASS**：schema 一致、TypeScript 与 Vite build、13 个 Node 单测、`cargo test --workspace`（200 个库测试及所有默认集成目标通过；仓库显式 ignored 项仍保持 ignored）、203 条需求/70 个页面/13 个 QA 场景/23 个基线文件的 traceability 检查。Vite 提示现有约 3.75 MB minified JavaScript chunk。
- `cargo test --manifest-path src-tauri/Cargo.toml --features 'integration-test order-gateway-runtime' --test order_gateway -- --nocapture` **PASS：13/13**。使用真实 Gateway 子进程与仅绑定 loopback 的假 provider；覆盖摘要校验、握手版本/认证/帧大小拒绝、准确 PLACE/CANCEL 身份和请求数、过期 grant 被拒绝且不触达 provider、明确拒绝、未知结果、重复激活及结果保存失败不重放。Control Plane 单测另验证到期 grant 阻止 `SUBMITTING` 并释放 PLACE reservation。
- Rust-backed CUA 浏览器验收 **PASS：14 个 PLACE 交互检查**。验证 Prepare 披露、键盘拒绝/显式批准、arm/账户/提案竞争 fail-closed、账户与提案重开恢复、一次 fake-provider POST、`ACCEPTED` / `REJECTED` / `UNKNOWN_RECONCILING` 的持久化与不可重发，以及 390/768/1280 px 和 accessible stale-status 呈现。浏览器状态与 provider responses 均为合成 fixture。
- 用 `synthetic-api-key` / `synthetic-api-secret` 验证 child 的 authenticated provider request；执行结果序列化中不包含这两个值。全程没有连接 provider host，也没有发送真实 provider POST/DELETE。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`、`git diff --check` 和三个变更/新增 Node 脚本的 `node --check` **PASS**。
- 严格 `cargo clippy --workspace --all-targets --all-features -- -D warnings` **未通过**：仓库现有 lint 包括广泛的 `TradeXError` 大型 Err 类型、既有 `ResultEnvelope` 大枚举、Binance 参数数量和 Bitget 测试模块顺序等。修复本切片新增的 Gateway 大枚举、冗余返回/闭包和复杂类型后，以这些现有 lint 类别显式 allow 的诊断运行通过；不将其记为全仓 Clippy PASS。

## 保留范围

真实 Trading 212 Live 读写和账户授权未在本项执行；本地 fake-provider 结果不代表 provider-hosted acceptance。S25 reconciliation/manual resolution、S26 完整 cancellation/fill convergence、S27 restart/sleep/session-lock recovery、S33 全页面回归和其他 provider Live execution 仍留给其各自工作项。
