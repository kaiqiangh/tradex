# S03 / #14 五步入门与风险默认值证据

状态：**VERIFIED（2026-09-13 #14 垂直验收完成）**。本票的 Workspace → Providers → Model → Risk defaults → Ready 工作流、风险策略持久化、公共门控、实际桌面路径和 Rust-backed 浏览器路径均已取得证据。S21 完整风险执行、S04 Thread/Turn、S33 全页面交叉回归仍保留在各自工作项，不能由本票提前关闭。

实现提交：`9aa66f9`（Rust 风险状态、SQLite schema v5、公共 IPC）、`28e4979`（React 入门与 Risk & Limits 页面）、`9f048dd`（Ready 未知账户与恢复路径门控）、`75dfefa`（required policy payload、账户行身份校验、单步导航约束与生成 schema）、`a3bf8bc`（持久化 risk 完整性、required nullable 输出和 route provider 身份门控）、`54232de`（Ready 配置不变量与 fresh account health 门控）、`bc2608d`（已有工作区的 Workspace picker/switch flow）、`56fcecd`（持久化账户运行时校验、projection 错误恢复与证据 CSV 修正）、`11c0ac7`（Ready 后 New Thread 准确区分模型路由与 Codex 不可用原因）、`f55f81c`（Ready/完成命令要求 live gateway 与可用 model plan，并动态呈现 gateway 原因）及 `c812ad6`（App 持有唯一 model projection，Models 复用该状态并保持 Backend §41.2 重订阅契约）；规范提交：`66ed6de`、`75dfefa`；开发分支：`dev`。

## 已实现

- `RiskPolicy` 使用精确十进制字符串保存四个金额/敞口字段；金额默认留空，市场单默认关闭，stale quote 默认 3 秒，Live inactivity 默认 20 分钟。数值边界和非法科学计数法在 Rust trust boundary 拒绝。
- `risk_state` 在 SQLite schema v5 中持久化 workspace、policy version、onboarding step、完成标记和只读四条硬安全规则；`risk.policy.changed` 事件进入 outbox、snapshot、replay 和前端 projection。
- `risk.get_policy`、`risk.save_policy`、`onboarding.set_step`、`onboarding.complete` 都要求 workspace/state version。跳步、陈旧游标、未配置风险策略、未验证默认模型 route 或非 DISARMED Live 账户均 fail closed。
- `risk.save_policy` 的七个字段在公共 wire payload 中全部必填；四个金额/敞口字段必须显式使用十进制字符串或 `null`。账户 projection 同时校验数据库 row key、projection `connectionId` 与 active workspace，外来/损坏行返回 `WORKSPACE_INTEGRITY_FAILED`；入门进度只允许前进或后退一步。
- 重新加载 risk projection 时再次检查七个 policy key、十进制/边界语义、state version、policy version、步骤/完成关系和后端 hard rules；缺失或被篡改的持久化 JSON 在进入 Ready 前返回 `WORKSPACE_INTEGRITY_FAILED`。已验证 route 同时要求 route 与当前选择的 provider 身份一致。
- `onboardingCompleted` 或 step 5 没有已配置策略时，持久化读取失败；Ready 只有在账户查询成功且不处于 fetching（没有使用失败重取时的缓存账户数据）时才显示账户状态并启用完成按钮。
- 入门页面复用既有 Accounts / Models surfaces；Workspace、Providers、Model、Risk defaults、Ready 五步可恢复。Ready 汇总 provider/model/fallback/Live arming，并在 Codex App Server 未配置时明确保持 Send disabled。Risk & Limits 页面将字段和硬规则作为可审阅的表单/只读列表呈现。

## 自动化证据

在代码提交 `c812ad6`（包含此前 `56fcecd`、`11c0ac7`、`f55f81c` 的完整实现）上通过：

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

## 本轮垂直验收（2026-09-13）

- **当前代码桌面窗口**：真实 workspace `22acfa1d-b5da-420a-a754-f01ea0c43458` 在已配置账户上以 `c812ad6` 重启并完成 Model → Risk defaults → Ready。网关在 `2026-09-13T14:12:11.942333Z` 探测为 RUNNING，DeepSeek `deepseek-v4-flash` 非推理路由于 `2026-09-13T14:12:36.759766Z` 完成真实验证；风险页显示四个金额为空、stale quote 3 秒、市价单 OFF、inactivity 20 分钟及四条只读 hard rules。Ready 摘要显示 provider/account、`DEEPSEEK · deepseek-v4-flash · disabled`、fallback 状态和 `All DISARMED`；完成后主线程显示 `Model route ready · DEEPSEEK · deepseek-v4-flash`，Send 保持 disabled，并明确显示 `Codex App Server is not configured; Send remains disabled until its later runtime slice.`。
- **Supporting Rust-backed browser path**：隔离临时 workspace `c504d75b-6dd4-4843-8af9-c6c187dbd1b1` 在 `11c0ac7`/`f55f81c` 上重跑完整五步；`c812ad6` 仅移除同窗重复 model subscription，未改变浏览器协议。仅注入脱敏的 synthetic model projection；`1e3` 被拒绝并显示 `Enter valid risk defaults within the stated decimal and time bounds.`，清空后保存成功；Ready 摘要与完成/刷新恢复成功。New Thread 显示 `Model ready · CHATGPT`、`Model route ready · CHATGPT · gpt-5.6-luna`，Send 仍 disabled，并明确显示 Codex 未配置原因。
- **交互与边界**：同一浏览器回归已覆盖 390/768 viewport 无横向溢出、风险表单键盘顺序、刷新后的状态恢复和空/非法策略负例；临时 Vite/IPC 进程随后停止，未写入真实凭据或 API key。

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

## 未完成边界

浏览器验收使用 synthetic model projection 只证明 UI/协议路径；真实 ChatGPT/DeepSeek route、secure-entry Save、OAuth 取消、OAuth 超时和 sidecar OAuth expiry UI 的 credentialed 证据仍以 `s03-model-evidence.md` 为准。S04 尚未消费 Codex Thread/Turn，S21 尚未完成完整 policy enforcement，S33 尚未完成全页面桌面/读屏交叉回归；这些边界不影响 #14 五步入门垂直票关闭。

## 串行代码审查

- Standards：`c812ad6`（含 `07d9ac5` 对临时 fan-out 尝试的完整回退）完成串行审查；Backend §41.2 的重订阅替换/失败清理契约保持，格式、schema、typecheck/build、Rust workspace、前端单测和 traceability 均通过，未引入新的安全边界。
- Spec：风险字段、默认值、五步顺序、Ready/Send 门控、live gateway/model plan 门控、硬规则只读、required/nullable wire schema、账户完整性、单步导航、错误分类和 state-version 约束与 PRD/UI §14/Backend §41–42 及 `model-onboarding.md` 一致；单一 model projection 让模型已 ready 但 Codex 未配置时主壳、onboarding 与 Settings 显示一致，S04 和完整 S33 证据明确保留为后续门槛。
