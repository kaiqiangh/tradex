# S23 #92 派发前安全释放验收

日期：2026-09-27。分支：`dev`。审查固定基线：`031bfdb3456f6f05e3d66997d8a370416bafcd15`。

## 实现范围

- 风险策略保存、Live 账户撤防、Disable All 与可信 TTL 扫描均在 SQLite immediate transaction 中停止仍为 `RESERVED` 的 attempt。策略先保存时，旧 approval 失效且不能创建 attempt 或 reservation；准备先完成时，后续相关策略/账户授权变化或 TTL 到期使 attempt 失效。
- 只将状态确认为派发前的 PLACE attempt 从 `RESERVED` 变为 `INVALIDATED`，并将唯一 active reservation 转为 `RELEASED`。CANCEL 不新增 reservation；本切片不处理可能已经跨过派发边界的状态。
- consumed approval 保留原审计状态；相同幂等键重放返回相同的已停止 attempt。attempt 与 reservation 的状态事件/快照重开可恢复；重复停止不再写第二次释放事件。
- Order Drafts 在 `RESERVED` 期间刷新耐久状态；失效卡片显示原因、释放金额与未发送 provider 请求说明，不再把释放金额显示为占用容量。

## 验证证据

- Rust 公共命令用例覆盖策略先保存与准备先完成两种顺序、单账户撤防、Disable All、trusted TTL、幂等重放、reservation create/release outbox 顺序及 workspace reopen。
- `npm run check` 通过：IPC schema 同步、TypeScript 检查与生产构建、11 个 Node 单测、187 个 Rust library tests、workspace 集成测试及需求追溯（203 requirements、70 screens、13 QA scenarios、23 baseline files）。集成套件中原生 Keychain、固定 Gateway 与外部证据门控的测试按现有规则 ignored。
- `cargo fmt --all -- --check`、`git diff --check` 和 `node --check tests/live-approval-ui.mjs` 通过。
- `tests/live-approval-ui.mjs` 的 Rust-backed 隔离浏览器运行通过 13 项观测：合成账户、行情及容量 fixture 覆盖显式 approval/preparation、历史恢复、容量冲突、键盘操作、1280/768/390 宽度、策略失效/释放和同页刷新。断言确认没有发送 provider 请求。
- 固定审查基线 `031bfdb3456f6f05e3d66997d8a370416bafcd15` 的 Spec 与 Standards 复审通过。实现提交为 `b790c095f268b78b3ef954faac8dd615d2cb4877`；最终证据索引提交和 `dev` HEAD 将写入关闭 #92 前的 resolution comment。

## 保留边界

S23 父规范 #88 仍保持 OPEN：父规范要求的 barrier-controlled 并发准备/策略保存竞态与事务中断回滚证据尚未由本切片提供。S27 重启/睡眠/恢复生命周期、S24 Gateway 派发以及 S33 全量回归也仍归各自工作项；此处不声称真实 provider 写入、订单或 broker acknowledgement。
