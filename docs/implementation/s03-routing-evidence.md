# S03 / #13 模型路由、回退与配额恢复证据

状态：**IMPLEMENTED_UNVERIFIED（保持 OPEN）**。本票完成本地公共协议、持久化路由策略、回退规划、冷却门控和 Rust-backed UI 可验证范围；#12 的 credentialed OAuth/上游 route 已在独立原生运行中通过，但 secure-entry Save、OAuth 取消/过期/超时 UI 和 S04 的 Codex Thread/Turn 流式生命周期仍未开始。

实现提交：`f2603f8`（包含 `83e91c9`、`dfc8e16`、`22cff46`）；规范提交：`7d398ad`；开发分支：`dev`。

## 已实现

- `ModelState` 持久化 `defaultRoute`、`automaticFallback` 和单调 `fallbackPolicyVersion`；旧 SQLite projection/attempt 缺少新字段时使用安全默认值。重开会清除运行时 route health，但保留用户默认选择。
- 新增 `model.set_default` 与 `model.set_fallback_policy` 公共命令，均要求 workspace 与 `stateVersion`，只接受 allowlist 内、当前 provider `READY` 且带 `verifiedAt` 的 route；写入完整脱敏 `model.provider.changed` 事件。
- `ModelRequestPlan` 为后续 Thread 提供不可变 provider/model/thinking 快照；初始 `thread_plan` 只包含 primary。fallback 默认关闭；只有明确 consent、传入的原始 ChatGPT snapshot 合法且已验证、DeepSeek `deepseek-v4-flash` 当前 `READY` 且已验证时，eligible `MODEL_UNAVAILABLE` / `OAUTH_EXPIRED` / `QUOTA_EXCEEDED` 才由 `fallback_plan` 返回一个单向 DeepSeek fallback。设置变更不会改写 in-flight primary，不会自动回切 ChatGPT，也不触碰账户 capability、risk、arming 或 approval。
- `ModelAttemptKind` 区分 `SETUP` 与 `THREAD`；现有登录、配置和验证写入 `SETUP`。quota 的 `retryAfterSeconds` 限制为 0–86400；控制面在已知窗口结束前拒绝新的 route Verify，并返回 `MODEL_QUOTA_COOLDOWN`。未知或格式错误 quota 保持 unavailable。
- Settings 显示默认 route、fallback privacy disclosure/version、provider-specific cooldown、attempt kind、错误类别和已知 quota；Verify/Use as default 按真实健康状态禁用，显式 Switch to DeepSeek 只改变下一次选择。

## 自动化证据

在 `f2603f8` 上通过：

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --features desktop --bin tradex
cargo clippy --features desktop --bin tradex -- -D warnings
cargo test --features desktop --workspace --all-targets
npm run schema:check
npm run typecheck
npm run build
npm run test:unit
npm run check
git diff --check
```

行为测试覆盖：默认缺失/已验证 route、fallback OFF/ON 与未验证 DeepSeek 拒绝、单向 eligible error、provider health 变化、state-version 冲突、attempt 顺序/类型、bounded retry-after 和 cooldown 前后的 provider 隔离。`npm run check` 最终报告 `Traceability OK: 201 requirements, 70 screens, 12 QA scenarios, 23 baseline files.`；构建仅有既有 bundle size warning。代码审查期间新增的 fallback eligibility/snapshot 负例在 `f2603f8` 上重新通过模型单测与 Clippy。

## Rust-backed 浏览器证据

`npm run dev:browser` 使用临时 SQLite workspace，未使用用户凭据。CUA 在 Settings → Providers & Models 验证：

- 固定 CLIProxyAPI `7.2.155` 可 Install & Launch、Probe、Stop；运行时组件显示网关 `Available`，停止后恢复 `STOPPED`。
- Configure DeepSeek 使用 integration-only synthetic key，页面显示 `Configured · verification required`；没有 webview password input。随后验证 route 失败时页面保留 `MODEL_UNAVAILABLE`，attempt 列表显示 `SETUP · DEEPSEEK · deepseek-v4-flash · disabled · FAILED`，没有伪造 Ready/default。
- 初始状态显示 fallback checkbox OFF 且因无 verified target 禁用；Use as default 也因 route 未验证/health 不可用禁用。390px 与 768px viewport 的 `body.scrollWidth` 分别等于 `390` 与 `768`，四个 route controls 均可见；浏览器 error/warning 日志为空。
- 临时 workspace 的 SQLite/WAL/SHM 扫描未发现 `integration-test-key`、`api.deepseek.com` 或 key 值。测试结束已停止网关。

## 未验证边界

本票的 fixture 证据没有使用真实 ChatGPT/DeepSeek 账户、OAuth token、API key 或上游推理，且不能替代 #12 的外部验收；该段记录的是 credentialed route 运行之前的历史范围。当前实现提供后续 S04 可消费的 planner 与 attempt contract，但不声称已完成 Codex stream、真实 Thread attempt、跨 provider fallback 的上游成功或五步 Ready。#12 的 secure-entry Save 与 OAuth 取消/过期/超时 UI 补证后，仍需按串行顺序补 S04 运行时消费与完整 UI 回归。

## 串行代码审查

- Standards：本地格式、Clippy、schema、类型、构建、workspace/desktop 测试、单元测试和 traceability 均通过。
- Spec：与 PRD §16.3/§26.3、UI §14.8/J1、Backend §41–42 及 `model-routing.md` 一致；真实 credentialed acceptance 和 S04 流式证据明确保留为后续门槛。
