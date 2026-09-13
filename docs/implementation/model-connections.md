# S03 / #12 模型连接与路由验证 Spec

状态：实施中，前置网关生命周期 #11 已关闭；本票只处理 ChatGPT OAuth、DeepSeek Keychain、精确路由验证和失败元数据。默认模型、回退政策及五步入门属于后继 #13–#14。

## 用户行为

- Settings 的 Models 面板列出 ChatGPT subscription 与 DeepSeek official API 两条来源。用户先启动固定 CLIProxyAPI，再可选择 Login/Re-login ChatGPT；DeepSeek key 可在网关停止时配置，替换已有 key 前需先停止网关，随后重新启动以渲染新配置。
- ChatGPT 登录只启动 pinned sidecar 的 `-codex-login` 浏览器流程。TradeX 不读取、解析或持久化 OAuth token；取消、超时、非零退出或 401 均保留既有状态并显示不可用。
- DeepSeek key 只在 macOS 原生 secure text dialog 输入，写入独立 OS Keychain service；renderer、SQLite、普通工作区、日志和事件只看到 `CONFIGURED`/health 元数据。取消不写入 Keychain；新 key 替换前保留旧 key，写入失败不覆盖旧状态。
- Verify 只允许 GPT-5.6 系列或 `deepseek-v4-flash` 的显式 `thinking.type=disabled|enabled`。调用受拥有 key 认证的 loopback `/v1/models`，再用固定 harmless prompt 对同一 provider/model/mode 做有界测试推理；目录存在但推理失败保持 UNVERIFIED。
- 每次 Verify/Login/重试写入新的 setup attempt，保留 provider、真实 model ID、thinking mode、开始/结束时间、结果、规范错误类别和已知 quota metadata；原 attempt 永不被新 attempt 覆盖。

## 公共契约

`model.get` 读取当前工作区 `ModelState`；`model.login_chatgpt`、`model.configure_deepseek` 与 `model.verify_route` 都要求当前 `stateVersion`。输入拒绝路径、token、key、URL、模型别名或 broker 引用。状态通过 `model.provider.changed` 和 `model.provider_attempt.changed` 事件重放。

`ModelState` 只保存：每个 provider 的 `configured`、`status`、允许路由、最近验证时间和规范错误；当前 route 只含 provider/model/mode/verifiedAt；attempt 列表最多 100 条；quota 只保存服务端明确给出的窗口/冷却元数据。OS Keychain reference 与 sidecar auth-dir 路径不进入该聚合。

## 失败与安全边界

- `401` → `OAUTH_EXPIRED`（ChatGPT）或 `MODEL_UNAVAILABLE`（DeepSeek）；`429` → `QUOTA_EXCEEDED`；连接、超时、空/伪造/未允许路由 → `MODEL_UNAVAILABLE`。
- 所有模型流量仍只到 `127.0.0.1:8317`；配置 DeepSeek 后由 Rust 重新渲染私有 sidecar config（0600）并在退出/失败清理。不得把 broker Keychain reference 或任何账户数据放进测试 prompt。
- ModelState 变化不修改账户、capability、risk、arming 或 approval。没有 verified route 时 `modelAvailable=false`，Send/Ready 保持阻断。

## 验证边界

- 当前 Rust tests 覆盖 schema、workspace scope、allowlist、attempt replay、失败类别、旧 stateVersion，以及目录/响应解析、状态码分类、配额 header bounds 和固定推理 payload；ignored macOS tests 已分别验证 disposable native Keychain item 与固定网关生命周期，并在结束时清理。原生窗口已验证 secure entry 的焦点与 Escape 取消；真实 ChatGPT OAuth 与 DeepSeek 两种模式的 credentialed 上游推理也已独立运行通过。
- Browser fixture 已检查 Login/Configure/Verify 的门控、错误保留、窄屏和秘密不进入 renderer；不使用用户 broker/API/OAuth 凭据。原生 OAuth 取消已验证为 `Configured · verification required` 并经 Verify 恢复；隔离 workspace 的 Tauri secure dialog Cmd-Return/Save 已完成写入、回读和清理；OAuth 超时 UI 已在隔离 workspace 验证并保持不可用，OAuth 401 过期 UI 仍需独立运行证据，不能用 fixture 标记 Ready。
