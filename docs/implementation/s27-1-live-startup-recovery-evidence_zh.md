# S27.1 #108 Live 启动恢复证据

日期：2026-09-29。分支：`dev`。审查基线：`4f3fe18`。

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

此前一次原生尝试使用临时 ad-hoc 签名 wrapper（`local.tradex.desktop`），并在 macOS `SecItemCopyMatching` 处阻塞，尚未到 provider HTTP。2026-09-29，普通 `npm run desktop` 开发应用（`target/debug/tradex`）重开 `/Users/kai/.tradex/workspaces/default` 时也先显示相同的 Keychain 等待；该次采样时，Trading 212 Live 投影仍为序号 14，`lastSuccessfulSync` 为 `2026-09-23T23:06:33.624426Z`，健康状态为 STALE/BLOCKED。

稍后的只读检查发现该次投影已到序号 16，状态为 `FAILED / ERROR / STALE / DISARMED`，原因是通用 `PROVIDER_RESPONSE_INVALID`，且 `lastSuccessfulSync` 未变化。进程于 13:10:55 启动，工作区 WAL 最后修改时间为 13:40:21。没有逐请求时间戳或 endpoint 详情，因此这次延迟持久化的失败不能证明 provider 读取在 `<5 s` 内启动；它确认验证失败后保留了旧观测并继续 disarm Live。

2026-09-29 的一次受控重启中，`target/debug/tradex` 于 14:11:58 启动。14:12:25 前，Trading 212 Live 行已到序号 17，状态为 `FAILED / UNCHECKED / STALE / DISARMED`，`lastSuccessfulSync` 仍是旧值；WAL 修改时间为 14:12:08。唯一观察到的 443 端口外连反向 DNS 指向 Google Cloud，进程采样显示 Alpaca stream，因此这不是 Trading 212 请求的证据。应用当时停留在 New Thread 页面，而不是已确认打开工作区的界面；没有测得 Trading 212 请求的启动时间。`<5 s` 原生验收仍未验证。

Accounts 页面还确认：选择已保存的 `trading212 · LIVE` 连接后，表单显示“使用已存储的本地凭据”和“Use existing account”，不会要求重新输入 API key。本次界面检查没有触发手动刷新。未更改 Keychain ACL 或凭据，也未尝试订单写入。主机没有有效代码签名 identity 或匹配的已安装 app bundle，因此可信签名包的 Keychain 行为仍未验证。

后续于 `2026-09-29T13:38:48Z` 重开同一工作区时，Trading 212 行仍为 `FAILED`，序号 18。启动恢复计划只纳入 `CONNECTED` 账户，因此该失败账户被正确排除；这次重开没有覆盖 Trading 212 provider 读取。之后对已保存账户点击只读刷新，进程到达 `NativeVault::get` 后阻塞于 `SecItemCopyMatching`（`provider_io.rs:1597`）。macOS SecurityAgent 要求输入 `com.tradex.broker.credentials` 项对应的 `login` Keychain 密码。用户已授权“Always Allow”，我也点击了该按钮，但系统仍等待 Keychain 密码。没有输入密码或新 API key；账户序号仍为 18，没有观察到 Trading 212 socket 或 provider HTTP 请求。因此原生 `<5 s` 启动耗时仍未验证。

## 2026-09-29 原生开发版重开验证

- PASS：通过普通 `npm run desktop` 重启并重开 `/Users/kai/.tradex/workspaces/default`。工作区 `last_opened_at` 为 `2026-09-29T17:35:17.113204Z`；Trading 212 Live 账户先于刷新写入 `STALE / BLOCKED / DISARMED`（`17:35:17.110664Z`），保留旧的成功观测。
- PASS：只读 provider 账户/订单历史读取于 `17:35:17.790510Z` 完成，账户于 `17:35:17.795093Z` 恢复为 `CURRENT`，距工作区重开约 `0.682 秒`。持久化投影包含 3 个持仓、0 个开放订单和 6 条近期订单；执行资格仍为 `BLOCKED`，账户保持 `DISARMED`。
- PASS：重开后的原生 Accounts 页面恢复了 Trading 212 Live 账户、恢复原因/状态、账户与持仓投影、空的开放订单列表及 6 条近期订单。界面显示 `ONLINE`、`VALID`、`CURRENT`、`DISARMED`、`BLOCKED`。没有触发 Arm 或任何订单操作。
- 此原生工作区只有一个已连接的受支持 Live 账户（Trading 212）；Binance/Bitget 和未知 attempt 分支由上文 Rust 测试覆盖。签名包 Keychain 生命周期与 S33 属于独立门禁。

Standards review 通过且无可操作问题。Spec review 未发现其他差异。上述原生开发版重开已验证本票的 `<5 s` 恢复验收；签名包生命周期和完整 S27/S33 验收仍未验证，分别由后续门禁负责。
