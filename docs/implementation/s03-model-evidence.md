# S03 / #12 模型连接验收证据

状态：**IMPLEMENTED_UNVERIFIED（保持 OPEN）**。模型连接的协议、受信边界、状态机和本地检查已完成；macOS secure entry、ChatGPT OAuth 和 DeepSeek 上游推理仍未取得验收证据，因此不关闭 #12 或其父项 #10，也不开始 #13。

实现代码提交：`8c4a87043f1bd379732809a1a1a2eba914eafbf8` + 边界修复 `fdf9fdc465619eab3cfbfddfca83f84c46077cbd` + 最终网关 key 生命周期修复 `753f273d4315ff68bdfa95c6f7cc784ec0da8053`（`dev`）；取消安全边界测试 `a94038545b7ffc3096fd58289324ac14fcf2f0da`。基线源清单修正提交：`612ef61b4eb6f5327943dfdd4349e23e6f1598c8`。用户已批准 DeepSeek 使用官方现行 `deepseek-v4-flash`，并显式区分 `thinking.type=disabled|enabled`。

## 已实现的边界

- 模型聚合状态按 workspace 持久化到 SQLite，事件只保存 provider、route、mode、状态、错误类别、quota 元数据和 attempt 结果；key、OAuth token、Authorization header 和完整上游响应不进入 IPC、SQLite、日志或普通文件。
- ChatGPT 只允许 `gpt-5.6` 前缀路由，登录通过固定 sidecar 的 `-codex-login`，不读取其他应用 auth-dir；发现结果去重并限制为最多 100 个允许模型。
- DeepSeek 只允许精确 `deepseek-v4-flash`，普通/推理模式必须分别携带显式 `thinking.type=disabled|enabled`。key 只经原生安全输入进入 macOS Keychain service `com.tradex.model.credentials`，受信 Rust 在网关启动时注入私有 0600 临时配置。
- Verify 先通过拥有 key 的 loopback `/v1/models` 确认精确路由，再以固定 harmless prompt、`max_tokens=16`、`temperature=0`、非流式请求做有界测试推理；空目录、错误路由、认证失败、超限、网络/响应错误都保持不可用并记录规范错误类别。
- Configure DeepSeek 在网关运行时禁用，避免已运行子进程继续使用旧 key；界面明确要求停止、配置后重新启动。模型 Ready 还要求网关处于 `RUNNING` 且存在当前 route 的成功验证时间。
- 验证失败不会伪装为已配置成功：已配置模型保持 `UNVERIFIED`、清除当前可用 route 并保留可审计的失败类别；ChatGPT `/v1/models` 的 401 映射为 `OAUTH_EXPIRED`。网关自动重启前从原生 Keychain 重新装载 DeepSeek key，公共 headless 命令先执行 payload/allowlist 校验；quota 元数据有界且在模型卡片中展示。
- 显式 Restart 与 workspace 切换会在清理旧进程后只保留当前 workspace 对应的 DeepSeek key；正常 STOP 不会留下重复 reload 标记，STOPPING 状态也阻断 key 替换。
- DeepSeek 原生录入取消路径通过 `MemoryModelVault` 安全边界测试：取消 attempt 追加为 `CANCELLED`，状态保持 `NOT_CONFIGURED`，不写入任何 key。

## 验收矩阵

| 验收点 | 当前证据 | 结论 |
| --- | --- | --- |
| 公共 IPC、SQLite 重开、事件重放和 workspace 隔离 | `src-tauri/src/{model.rs,lib.rs,protocol.rs,storage.rs}`、`src-tauri/tests/model.rs`、`src/projection.ts`、`shared/ipc-*`；模型 provider/attempt 事件只接受 `Model` aggregate | PASS（本地） |
| 固定版本网关真实生命周期 | macOS ignored test `pinned_gateway_lifecycle_uses_public_commands_without_model_readiness` 以 `.artifacts/s03-planning/upstream/cli-proxy-api` 运行通过：端口冲突、启动/探测、停止清理、stale config、4 次崩溃退避及最终失败 | PASS（直接进程） |
| 凭据安全边界和原生 Keychain API | `src-tauri/src/model_credentials.rs` 的 `ModelVault`；renderer 没有 password input；取消录入测试 `cancelled_deepseek_entry_does_not_write_keychain_double` 通过；macOS ignored test `native_model_keychain_roundtrip_is_workspace_scoped` 以 disposable synthetic key 运行通过（`cargo test --test model native_model_keychain_roundtrip_is_workspace_scoped -- --ignored --nocapture`） | PASS（取消不写入 + 直接 Keychain round-trip）；secure entry UI 仍未验证 |
| 精确 provider/model/mode allowlist | Rust allowlist tests 覆盖 ChatGPT `gpt-5.6*` 和 DeepSeek `deepseek-v4-flash` 两个显式模式，拒绝别名/未知模式 | PASS（本地） |
| 认证 probe、测试推理与错误分类 | `gateway_process.rs` bounded `/v1/models`、`/v1/chat/completions`、401/404/429/非 JSON/空 choices 分支；protocol canonical mapping 和 model failure tests | PASS（代码/本地分支）；未做 credentialed upstream run |
| 浏览器交互与窄屏 | 最终代码 `753f273d4315ff68bdfa95c6f7cc784ec0da8053` 上重跑 `npm run dev:browser` 隔离工作区：Settings 显示两张模型卡；网关停止时 Login/Verify disabled；Configure DeepSeek synthetic fixture 后显示 `Configured · verification required`；未发现 renderer 密码输入。390/768 视口检查 `scrollWidth 375/753`，临时 SQLite/WAL/SHM 未发现 `integration-test-key`、`api.deepseek.com` 或 key 值 | PASS（浏览器 fixture） |
| 真实 OAuth、上游 DeepSeek API 与桌面 secure dialog | 需要用户账户/凭据和解锁的 macOS 原生窗口；本轮 `cua.getState()` 返回 `The Mac is locked and automatic unlock could not unlock it` | BLOCKED_EXTERNAL |

## 可重跑检查

以下检查在最终代码 `dev@753f273d4315ff68bdfa95c6f7cc784ec0da8053` 通过：

```text
git diff --check
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
npm run schema:check
npm run typecheck
npm run test:unit
npm run check
```

`npm run check` 包含 schema、TypeScript、Vite build、前端 3 个单元测试、Rust workspace 和需求追踪；最终输出为 `Traceability OK: 201 requirements, 70 screens, 12 QA scenarios, 23 baseline files.`。模型专用集成测试为 5 个通过，另以显式 `--ignored` 运行原生 Keychain round-trip 1 个通过；固定版本网关生命周期 ignored test 也以显式 `--ignored` 运行并通过 1 个。lib 内模型状态/失败边界测试为 6 个通过。其它 native broker 测试仍按测试定义 ignored。构建只有既有的 bundle size warning，没有失败。

## 秘密与外部门槛

浏览器 fixture 的临时 SQLite/outbox/model_state 扫描没有 `integration-test-key`、`api.deepseek.com` 或 `key` 值；模型测试断言序列化 payload 不含这些值。真实 key 没有写入仓库、普通工作区文件或日志。

本轮不能完成 macOS secure text field、Cmd-Return 保存/Escape 取消，因为 CUA 仍报告桌面锁定；直接原生 Keychain round-trip 已用 disposable synthetic key 运行通过。没有使用用户 ChatGPT/DeepSeek 账户、OAuth token、真实 API key 或真实推理请求；fixture 成功不会标记 Ready。解锁桌面并提供允许的独立测试资源后，重跑 secure entry、OAuth 取消/过期/超时和 DeepSeek 两模式上游推理，才能把本票提升为 VERIFIED。

## 串行代码审查

- **Standards：PASS。** 已按仓库 `AGENTS.md` 和 `docs/agents/*` 检查；格式、Clippy、schema、typecheck、build、unit、workspace 和 traceability 均通过。
- **Spec：PASS（已实现范围）。** Backend §41–42、公共模型事件、精确 Flash 模式、秘密边界、错误分类和 UI 状态与 RevC/用户批准契约一致；外部 credentialed acceptance 仍明确为未验证。
