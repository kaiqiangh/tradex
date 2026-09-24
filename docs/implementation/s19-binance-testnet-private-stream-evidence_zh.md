# S19 #70 Binance Testnet 私有流验收证据

**状态：**实现票 #70 的本地验收已完成。父 Spec [#67](https://github.com/kaiqiangh/tradex/issues/67) 仍等待 provider-hosted Testnet 验收；本地 fixture 证据不替代该外部 gate。

- 分支：`dev`
- 审查基线：`ce4d20cfb57dd78c5d2cbd72cecf920674c31ae2`
- 实现提交：`6da34af99a5a7ee1948b480de4160a8036c3b4e6`
- 范围：[#70 — 接收 Binance Testnet 私有流并恢复账户状态](https://github.com/kaiqiangh/tradex/issues/70)
- 契约：Backend ARD §41.27、Frontend ARD §13.15、UI Spec §14.13

## 已实现行为

桌面后端仅为准确身份已连接的 Binance Spot Testnet 账户启动签名私有 WebSocket worker。凭据和签名订阅数据留在后端；worker 校验订阅确认及每个事件身份，并限制帧大小和事件处理队列。Execution report 保留准确 provider 状态和十进制数量，对 fill 去重，并拒绝过时或冲突更新。账户余额事件只更新其明确指出的资产。仅有 delta 的余额事件及未知事件会让 reconciliation 保持 required。

流与恢复观察在一个 SQLite transaction 中更新现有订单簿、账户健康投影及其 outbox event。断流、workspace reopen、pause 和系统 resume 保留最后可信记录，同时将健康状态标记为 degraded、订单簿标记为 stale。固定 REST 路由会先核验账户，再刷新开放订单和余额，并推进 BTCUSDT/ETHUSDT 有界订单及成交历史；全部所需读取完成后才恢复 current。没有增加 renderer 流命令，也没有增加 Live、Local Paper、Agent、审批、reservation 或 Order Gateway 权限。

## 验证

以下检查针对实现提交 `6da34af99a5a7ee1948b480de4160a8036c3b4e6`：

- `npm run check`：通过，包含生成 schema/type 一致性、TypeScript 与生产构建、前端测试（9/9）、Rust 单元测试（139/139）、可运行的 workspace 集成套件及需求追溯（203 项需求、70 个界面、13 个 QA 场景）。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml --features desktop --bin tradex`：通过。
- `cargo build --manifest-path src-tauri/Cargo.toml --bin tradex-ipc --features integration-test`：在实现提交上通过。
- `python3 scripts/check_requirements.py`、`node --check tests/provider-ui.mjs` 和 `git diff ce4d20cfb57dd78c5d2cbd72cecf920674c31ae2..6da34af99a5a7ee1948b480de4160a8036c3b4e6 --check`：通过。
- Rust loopback WebSocket/HTTP 与临时 SQLite 测试验证了签名订阅、有界重连/退避、断流后对账、显式 pause/stale 状态、去重及账户/workspace 绑定。
- Rust-backed 浏览器 fixture 验收通过。合成的 `outboundAccountPosition` 持久化了 USDT 可用/锁定余额；reconciliation 成功前健康状态保持 `REQUIRED`。部分成交重复投递仍只存一条，较晚的 `NEW` event 不会回滚部分成交。断流后订单簿为 stale、健康状态为 degraded；有界 fixture 对账后恢复 `CURRENT`。390、768、1280 px 下，无障碍流状态与 reconciliation 文案保持可见且无横向溢出。
- Standards 与 Spec 按串行流程基于固定基线完成复核；无遗留问题。

生产构建有既存 Vite bundle size warning。测试套件标记为 ignored 的原生 Keychain/gateway 测试不计为本票已验证项。

## 证据边界

Provider 凭据、WebSocket event 和 REST 响应均为本地一次性 fixture。没有使用真实 Binance Testnet 凭据、没有请求 Binance 外部 API，也没有下单。原生桌面目标已编译，但这不代表建立了真实 provider-hosted Testnet session。父 Spec #67 仍须完成外部验收。
