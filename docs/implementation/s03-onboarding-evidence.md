# S03 / #14 五步入门与风险默认值证据

状态：**IMPLEMENTED_UNVERIFIED（保持 OPEN）**。本票已完成可恢复的 Workspace → Providers → Model → Risk defaults → Ready 工作流、风险策略持久化与公共门控。#12 的 credentialed OAuth/上游路由、原生 secure entry 取消和 OAuth 取消已取得独立证据，但 secure-entry Save、OAuth 过期/超时 UI、Ready 重跑和完整 S33 桌面回归仍未完成，因此本票不把 fixture 失败或无凭据路径升级为 Ready 通过。

实现提交：`9aa66f9`（Rust 风险状态、SQLite schema v5、公共 IPC）、`28e4979`（React 入门与 Risk & Limits 页面）、`9f048dd`（Ready 未知账户与恢复路径门控）、`75dfefa`（required policy payload、账户行身份校验、单步导航约束与生成 schema）、`a3bf8bc`（持久化 risk 完整性、required nullable 输出和 route provider 身份门控）、`54232de`（Ready 配置不变量与 fresh account health 门控）、`bc2608d`（已有工作区的 Workspace picker/switch flow）及 `56fcecd`（持久化账户运行时校验、projection 错误恢复与证据 CSV 修正）；规范提交：`66ed6de`、`75dfefa`；开发分支：`dev`。

## 已实现

- `RiskPolicy` 使用精确十进制字符串保存四个金额/敞口字段；金额默认留空，市场单默认关闭，stale quote 默认 3 秒，Live inactivity 默认 20 分钟。数值边界和非法科学计数法在 Rust trust boundary 拒绝。
- `risk_state` 在 SQLite schema v5 中持久化 workspace、policy version、onboarding step、完成标记和只读四条硬安全规则；`risk.policy.changed` 事件进入 outbox、snapshot、replay 和前端 projection。
- `risk.get_policy`、`risk.save_policy`、`onboarding.set_step`、`onboarding.complete` 都要求 workspace/state version。跳步、陈旧游标、未配置风险策略、未验证默认模型 route 或非 DISARMED Live 账户均 fail closed。
- `risk.save_policy` 的七个字段在公共 wire payload 中全部必填；四个金额/敞口字段必须显式使用十进制字符串或 `null`。账户 projection 同时校验数据库 row key、projection `connectionId` 与 active workspace，外来/损坏行返回 `WORKSPACE_INTEGRITY_FAILED`；入门进度只允许前进或后退一步。
- 重新加载 risk projection 时再次检查七个 policy key、十进制/边界语义、state version、policy version、步骤/完成关系和后端 hard rules；缺失或被篡改的持久化 JSON 在进入 Ready 前返回 `WORKSPACE_INTEGRITY_FAILED`。已验证 route 同时要求 route 与当前选择的 provider 身份一致。
- `onboardingCompleted` 或 step 5 没有已配置策略时，持久化读取失败；Ready 只有在账户查询成功且不处于 fetching（没有使用失败重取时的缓存账户数据）时才显示账户状态并启用完成按钮。
- 入门页面复用既有 Accounts / Models surfaces；Workspace、Providers、Model、Risk defaults、Ready 五步可恢复。Ready 汇总 provider/model/fallback/Live arming，并在 Codex App Server 未配置时明确保持 Send disabled。Risk & Limits 页面将字段和硬规则作为可审阅的表单/只读列表呈现。

## 自动化证据

在最终代码 HEAD `56fcecd` 上通过：

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --features desktop --bin tradex
cargo clippy --features desktop --bin tradex -- -D warnings
npm run check
```

`npm run check` 包含 schema:check、TypeScript、Vite build、3 个前端 projection 单测、Rust workspace 和 `check_requirements.py`；最终 traceability 为 `201 requirements, 70 screens, 12 QA scenarios, 23 baseline files`。构建只有已有的 bundle size warning。风险专用单测覆盖默认值、精确小数/边界、非法输入、缺少 policy 字段、版本冲突、外来账户行、持久化 provider/environment/arming 校验、单步前后导航、Ready 模型门控、完成和重开后恢复；projection 错误 banner 提供重新加载路径。

## Rust-backed 浏览器证据

`npm run dev:browser` 使用隔离临时 SQLite workspace 和 fixture provider，不接触用户凭据。以下页面证据是在初始实现提交 `28e4979` 的 Rust-backed 浏览器运行中采集；最终 review 修复由公共协议、schema 与自动化检查覆盖，需在可用桌面窗口中重新跑一遍完整 S33 才能升级为最终 UI 证据。CUA 通过真实页面路径验证：

- Open workspace 后可依次进入 Workspace、Providers、Model；无已验证 route 时 Continue to Risk defaults 保持 disabled，并显示需要验证默认模型的原因。
- 通过 Settings → Risk & Limits 检查四个可留空的金额/敞口字段、3 秒 stale quote、20 分钟 inactivity、OFF 市价单和四条 read-only hard rules；保存返回 `Saved policy v2/v3`。
- 进入 Risk defaults 后用空金额保存，页面显示 `Saved policy`，尝试 Ready 返回 `Ready requires a verified default model route and saved risk defaults.`；没有伪造 Ready 或 Send 可用。
- 390px 与 768px viewport 均满足 `bodyScrollWidth == clientWidth == innerWidth`，风险表单和保存按钮可见。点击首个输入后按 Tab，焦点移动到下一个风险字段，键盘顺序可达。浏览器 error/warning 日志为空。

## 公共协议负例

在同一隔离 workspace 上，真实 `/__integration/command` 返回：

| 场景 | 结果 |
| --- | --- |
| `risk.save_policy` 使用 `1e3` | `RISK_POLICY_INVALID` |
| 从 step 1 直接请求 step 3 | `ONBOARDING_STEP_INVALID` |
| step 4 请求 Ready，缺少 verified default route | `ONBOARDING_BLOCKED` |
| 使用旧 `expectedStateVersion` 保存 | `STATE_VERSION_CONFLICT` |
| 使用旧 `expectedStateVersion` 设置 onboarding step | `STATE_VERSION_CONFLICT` |

失败请求不改变策略或 onboarding 权限。测试后已停止 Vite/`tradex-ipc`，进程扫描无残留；临时 workspace 未发现 `integration-test-key`、DeepSeek endpoint 或 key 值。

## 未验证边界

本票的初始浏览器/协议证据没有使用真实 ChatGPT/DeepSeek OAuth、API key 或上游推理，且没有声称 Codex Thread/Turn 已可用；该段是 #12 credentialed 运行之前的历史记录。后续独立证据已在 `s03-model-evidence.md` 记录真实 route 成功以及 OAuth 取消；仍需重跑 Ready、完成 secure-entry Save 与 OAuth 过期/超时 UI，并通过 S04 消费和 S33 全页面桌面回归，才能把相关需求提升为 VERIFIED。

## 串行代码审查

- Standards：格式、Clippy、schema、typecheck/build、Rust workspace、前端单测和 traceability 均通过。
- Spec：风险字段、默认值、五步顺序、Ready/Send 门控、硬规则只读、required/nullable wire schema、账户完整性、单步导航、错误分类和 state-version 约束与 PRD/UI §14/Backend §41–42 及 `model-onboarding.md` 一致；credentialed #12、S04 和完整 S33 证据明确保留为后续门槛。
