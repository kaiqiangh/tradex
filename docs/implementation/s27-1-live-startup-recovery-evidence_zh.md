# S27.1 #108 Live 启动恢复证据

日期：2026-09-29。分支：`dev`。审查基线：`8d007f5b02fe557d17847ea074e3ece7a13e484a`。

## 实现范围

工作区启动/重开后，为所有已连接的 Trading 212、Binance Spot 和 Bitget Spot Live 账户执行只读恢复，不依赖 UI 当前选中的账户。Live health 先变为 stale/disarmed，建立新的本地 TimeService 基线；未知 PLACE 的现有 S25 准确证据读取先于账户刷新派发，两类读取仍会并发重叠，且 P0 准确证据结果先于账户投影提交。UNKNOWN PLACE 或 CANCEL 都会继续阻止账户进入 CURRENT；未知 CANCEL 不会自动查询或重放，必须由用户触发现有准确订单刷新。失败或证据不完整时保留 stale/blocked reason 与现有账户观测，不 Arm，也不重放任何订单操作。

Trading 212 读取一页最多 50 行近期订单。Binance 对当前开放订单和已保存近期订单涉及的最多 10 个 symbol，各读取一页最多 1,000 行；尚未在本地观测过的 symbol 可能不在结果中。Bitget 复用现有订单/成交投影。Trading 212/Binance 的 `recentOrders` 与 `openOrders` 分开展示，不作为开放订单、撤单或容量的依据。

## 验证

- PASS：`reopening_the_same_workspace_stales_and_disarms_every_live_account`。
- PASS：`startup_revalidates_local_clock_without_connected_live_accounts`。
- PASS：`startup_recovery_covers_all_live_accounts_and_preserves_unknown_scope`。
- PASS：`startup_recovery_stays_blocked_for_an_unknown_live_cancel_attempt`；启动不安排准确 CANCEL 读取，账户刷新不能清除 stale/disarmed gate。
- PASS：`cancellation_approval_tests::startup_probe_refresh_fetches_and_persists_t212_recent_live_history`；覆盖 startup `Probe` job 的 Trading 212 有界近期订单读取。
- PASS：`t212_recent_history_is_bounded_and_rejects_duplicate_or_untrusted_pages`。
- PASS：`provider_io::binance::tests::live_recent_order_history_requires_the_requested_symbol_and_known_status`。
- PASS：`live_approval_tests::binance_live_account_refresh_restores_bounded_recent_order_history`；本地签名请求 fake 确认了有界 GET 和近期订单投影持久化。
- PASS：`cargo check --manifest-path src-tauri/Cargo.toml --bin tradex --features desktop,integration-test`。
- PASS：最近一次 `npm run check` 全量运行（schema check、typecheck/build、15 项前端单测、Rust workspace 测试和需求追溯）。
- 修复后的首次全量检查有两项 workspace 重开断言失败；分别单独重跑均通过，随后全量重跑也通过。
- PASS：`git diff --check`。

桌面 UI 成功恢复现有 default workspace，并显示 Trading 212 Live 为 stale/disarmed。Tauri 构建被放入临时 ad-hoc 签名 wrapper（`local.tradex.desktop`）；启动恢复和手动账户刷新都在 macOS `SecItemCopyMatching` 读取 Keychain 时阻塞，尚未到 provider HTTP。进程采样确认在等待 Keychain，且未观察到网络 socket 或 provider 请求，因此不能证明 `<5 s` 请求启动目标。主机没有有效代码签名 identity，也没有匹配的已安装 app bundle，无法验证可信签名包的 Keychain 行为。未改动 Keychain ACL 或凭据，也没有真实 provider 请求或订单写入。

Standards review 通过且无可操作问题。Spec review 未发现其他差异，但保留原生 `<5 s` 启动耗时验收项。签名包生命周期和完整 S27/S33 验收仍未验证；#108 保持打开，等待可信原生启动验证。
