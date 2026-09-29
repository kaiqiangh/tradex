# S27.2 #109 Live 休眠恢复验收证据

审查基线：`3ca4d54`（S27.1 #108）。验证日期：2026-09-29。实现已可进入 Standards 与 Spec 审查。

## 实现行为

- macOS 系统唤醒和 TradeX 重新成为活动应用都会进入现有 Live 安全恢复流程。Tao 文档说明 `RunEvent::Resumed` 在 macOS 不受支持，因此由系统唤醒与应用激活通知驱动恢复。
- Resume 会 disarm 所有 Live 账户、重置 TimeService confidence，并在 provider recovery 开始前，将已连接 Live 账户持久化为 `STALE / UNVERIFIED / UNCHECKED / STALE / BLOCKED / DISARMED`。
- 现有 stream supervisor 会重启；随后复用启动恢复计划重新校验时间、读取已连接账户未知 PLACE attempt 的准确 evidence，并刷新所有已连接且受支持的 Trading 212、Binance Spot 和 Bitget Spot Live 账户。UI 账户选择不会缩小恢复范围。
- 刷新前账户保持 disarmed，stale 时无法 Arm。恢复不会重发 PLACE/CANCEL；账户投影通过已有 storage 路径使旧 account-bound approval 和派发前准备失效。

## 验证

- PASS：`sleep_and_resume_safety_triggers_persist_disarm_reasons` 覆盖两个已连接 Live 账户、持久化 freshness 失效、trusted-time 重置/重验证、恢复计划覆盖、刷新前 Arm 被阻断，以及恢复完成后仍需显式 Arm。
- PASS：`live_provider_auth_failure_disarms_only_the_affected_account` 验证认证失败只持久化到受影响 Live 账户，同时保持另一个健康 Live 账户原有的 ARMED/CURRENT 状态。
- PASS：`cargo check --manifest-path src-tauri/Cargo.toml` 在 macOS desktop 目标上编译通过。
- PASS：`cargo fmt --manifest-path src-tauri/Cargo.toml --check` 与 `git diff --check`。
- PASS：`RUST_TEST_THREADS=1 npm run check` 完成 schema 校验、前端 typecheck/build、15 项前端单测、227 项 Rust 核心单测与 workspace 集成测试，以及需求追溯（203 条需求、70 个页面、13 个 QA 场景）。
- 备注：此前默认并行的 `npm run check` 有两项 workspace reopen 测试失败；两项单独重跑均通过，完整检查在 Rust 测试串行执行时通过。
- PASS：运行中的原生开发版返回 TradeX 时触发 `NSApplicationDidBecomeActiveNotification`，并把 Trading 212 Live 持久化为 `SESSION_RESUMED / STALE / BLOCKED / DISARMED`；上次成功观测保持不变。
- BLOCKED：原生只读 provider refresh 停在 macOS SecurityAgent；系统要求输入 `com.tradex.broker.credentials` 对应的 `login` Keychain 密码。没有输入密码或替换凭据，因此原生 provider recovery 完成情况未验证；没有尝试订单写入。
- 未执行：实际 OS sleep/wake。Darwin 编译覆盖了 wake 通知路径；已直接验证原生应用重新激活。

生产恢复路径复用现有固定只读 provider adapter，不增加 provider 路由或金融权限。
