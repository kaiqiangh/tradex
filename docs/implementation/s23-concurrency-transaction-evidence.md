# S23 #93 并发容量与事务回滚验收

日期：2026-09-27。分支：`dev`。实现 SHA：`1c2cf7a97d183e19c9769eb08dc4a5fce1c383ea`。审查基线：`dba02e1ab49d58610735ec1f2c8b4d0582a1a03c`。

## 实现范围

- barrier-controlled Rust Control Plane 调用验证同账户容量竞争最多只成功预留一次、不同账户容量互相隔离；策略保存先于 preparation 时旧审批不消费，preparation 先完成时后续策略更新会失效并释放未派发预留。
- 新申请仍执行当前 `maxReservedCapital` 风险检查。已绑定审批的可变 reservation 检查延迟到 SQLite preparation transaction；稳定风险检查、账户/提案/策略绑定及其他证据摘要仍须匹配。事务拒绝保存 `RESERVED_CAPACITY` 和 `WORKSPACE_RESERVED_CAPITAL` 后端上下文；相同幂等身份重放返回同一拒绝。
- 同币种工作区金额不依赖完整/新鲜的跨账户组合快照或 FX 路由；跨币种金额仍要求完整快照和近期、已验证 FX 路由。
- 注入 attempt outbox 写入故障后，关闭并重开 workspace，验证 approval、proposal、reservation、attempt、consumption/rejection audit 与相关 outbox 均无部分提交；移除故障后使用同一幂等身份重试，持久化一组一致状态。
- provider router 对 `trade.execution.prepare` 返回无 job；attempt 保持 `RESERVED`。全程使用合成数据与临时 SQLite，没有 Gateway 派发或 provider 写请求。

## 验证证据

- `RUST_TEST_THREADS=4 npm run check` 通过：IPC/schema、TypeScript、生产构建、11 个 Node 单测、191 个 Rust 单测、Rust 集成测试及需求追踪（203 条需求、70 个页面、13 个 QA 场景、23 个基线文件）。仓库既有的 Keychain、固定 Gateway 与外部 provider 门控测试保持 ignored。
- `cargo test --manifest-path src-tauri/Cargo.toml --lib live_approval_tests:: -- --test-threads=1`：22/22 通过；覆盖同账户容量竞争、账户隔离、两种 policy/preparation 次序、workspace reserved-capital 拒绝/重放、故障回滚/重开/重试。
- `cargo test --manifest-path src-tauri/Cargo.toml --lib risk::tests::reserved_capital_fx_conversion_requires_recent_verified_route -- --exact`：通过；覆盖跨币种过期 FX fail-closed 及降级快照中的同币种金额。
- `cargo fmt --all -- --check`、`node --check tests/live-approval-ui.mjs`、`git diff --check` 均通过。未运行浏览器行为回归：本票没有 UI 改动。
- 对固定基线 `dba02e1ab49d58610735ec1f2c8b4d0582a1a03c` 的 Standards 与 Spec 串行复审均 PASS。

## 保留边界

本证据仅关闭 #93 的并发、容量与事务原子性切片。父规范 #88 保持 OPEN，S27 生命周期恢复及 S33 全页面/键盘/窄屏回归仍由各自工作项负责；S24 Gateway dispatch、provider mutation、真实订单和 broker acknowledgement 未实现或验证。
