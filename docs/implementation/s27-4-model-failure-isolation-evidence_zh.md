# S27.4 模型故障隔离验收证据

工作票：[验证模型故障不阻塞可信 Live 控制面](https://github.com/kaiqiangh/tradex/issues/111)。审查基线：`495fc745e91a87095cccb00041ca902d8979c89a`。验证日期：2026-09-29。精确交付提交记录在 issue resolution 中。[English](s27-4-model-failure-isolation-evidence.md)。

## 已验证行为

- 确定性的 Control Plane 矩阵覆盖模型 sidecar 停止、网关未授权、端口冲突、额度故障和 OAuth 过期。每种故障仅在受影响 Turn 记录规范错误及修复动作；在该转移时，另一运行中 Turn、账户健康和 Arm 状态、风险策略、能力决策、默认/当前模型选择以及另一账户的冷却预算均保持不变。
- 推理不可用时，独立且符合资格的 Trading 212 Live Proposal 仍可通过 main consumer 查看、审批和准备执行。已有 Binance `UNKNOWN_RECONCILING` Attempt 保留有效 Reservation。
- 三个 Binance P3 槽位仍被占用时，精确 P0 对账通过真实 ProviderJob 继续运行。合成 HTTP 429 仅记录 `INCONCLUSIVE`，不会成为成功结论，未知 Attempt/Reservation 均保留。现有调度回归覆盖 P1 优先、队列上限、最大并发及账户冷却隔离。
- 推理停止时，精确 Trading 212 CANCEL Intent 仍可审阅、显式 Arm/审批及准备。可信 dispatch package 和新鲜账户/订单预检先于持久化 SUBMITTING，随后仅向合成 `/api/v0/equity/orders/123456` 发送一次 DELETE。确认只持久化为 `CANCEL_PENDING`，不创建 PLACE Reservation。现有真实子进程 Gateway 测试另行验证私有通道及单次写入边界。
- 推理停止时，Provider 认证故障仅使对应 Live 账户 Disarm；之后 stream 健康降级仅使第二个受影响账户 Disarm；第三个健康账户仍为 Armed/Current。此项经过共同账户健康持久化边界，不声称发生了真实 Live websocket 断线。
- Resume 和实际 SQLite 重开保留未知 Reservation，Live 账户保持 Disarmed。恢复网关可用性不会自动 Arm。Automatic fallback 默认 OFF，只有通过既有命令显式启用后才允许符合条件的 DeepSeek 回退；默认路由及金融能力保持不变。

## UI 与集成传输

`tests/s27-model-recovery-ui.mjs` 使用隔离 Rust/SQLite 集成服务、临时工作区和合成 Binance/Trading 212 账户。准备/状态 helper 使用测试 HTTP API，UI checker 使用 Codex CUA 浏览器绑定。

Chrome 在 390/768/1280px 验证 STOPPED、UNAUTHORIZED、PORT_CONFLICT 和 RUNNING 恢复状态。Providers & Models 修复提示、Reload model state、Settings 导航、Account Health 和账户选择均支持 Enter；页面无横向溢出。未观察到应用控制台错误。两条无关的 `chrome-extension://` 导入错误明确排除。强制 SSE 断线显示恢复错误，通过键盘 Retry connection 可恢复权威工作区/模型状态。

验证中发现独立浏览器订阅耗尽同源 HTTP 连接槽位，使模型/账户快照停在 Loading。集成浏览器传输现在共用一个 SSE 连接，保留逐 aggregate 的 schema 校验和路由。关闭一个订阅不影响其他订阅；断线使订阅失效并允许建立新连接。原生 Tauri Channel 路径保持不变。八订阅回归验证此边界。

仅集成构建提供的 `model.gateway.fixture` 只接受 STOPPED/UNAUTHORIZED/PORT_CONFLICT/RUNNING，发布真实存储的网关投影，不启动/停止模型 sidecar；生产构建不暴露该入口。QA 中内置浏览器控制无响应，其超时不作为产品 PASS。完整 UI 证据来自 Chrome。

## 验证门禁

- PASS：三项定向 `s27_model` 测试，涵盖五种故障矩阵、实际合成撤单、账户级 auth/stream 隔离、负载下 P0 进展、resume/reopen 和显式 fallback 策略。
- PASS：浏览器事件传输回归和前端 typecheck。
- PASS：四种网关状态 × 三种浏览器宽度、键盘交互、无应用控制台错误和 SSE Retry 恢复。
- PASS：最终 `npm run check`，包括 schema 一致性、typecheck/build、16 项前端测试、237 项 Rust 库测试和 workspace 测试（共 328 项 Rust 通过，36 项既有测试忽略），以及 203 requirements / 70 screens / 13 QA 清单。清单仅证明可追踪性，不是运行时证据。
- PASS：`cargo test -p tradex --features 'integration-test order-gateway-runtime' --test order_gateway`，21/21 项隔离真实子进程测试。
- 较早一次完整运行中，既有工作区身份/重开测试未得到成功 result envelope 而失败；整个工作区套件未改代码复跑 7/7 通过，随后完整仓库运行通过。保留此测试波动记录，不抹去失败，也不声称本票修复了生产缺陷。
- PASS：桌面 feature 构建（`cargo check -p tradex --features 'desktop integration-test order-gateway-runtime' --bins`）、Cargo/Rust 格式检查及 `git diff --check`。
- PASS：基于 `495fc745e91a87095cccb00041ca902d8979c89a` 的串行独立 Standards 与 Spec 审查，两轴均无可执行发现。首次启动审查因账户使用额度中断；重新检查确认可使用后恢复并完成审查。两次审查均未复跑测试。
- CI：验收时仓库没有 workflow 文件，dev 也没有运行记录；上述检查为本地证据，不是 CI PASS。

这些证据证明确定性/本地恢复边界，未执行真实券商写操作，也未修改用户配置的模型/账户凭据。S27 父级验收、签名包生命周期、真实 Provider 门禁及 S33 分别处理。
