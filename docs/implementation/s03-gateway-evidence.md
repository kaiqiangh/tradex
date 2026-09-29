# S03 / #11 网关生命周期验收证据

状态：**VERIFIED（仅覆盖 S03 的 #11 网关生命周期票）**。S03 规范和 #12–#14 仍保持 OPEN；本证据不把模型 OAuth、DeepSeek 推理、配额回退或五步入门标记为完成。

实现范围：`d1cc6f1..83b691195829f7f4db94a036c1aa33544dd9c8ad`，分支 `dev`。`d1cc6f1` 是用户批准 DeepSeek Flash 契约后的工作区起点；`83b6911` 是本票最终代码提交。

## 固定运行时与边界

- CLIProxyAPI 固定为 `v7.2.155`，来源为 [官方 release](https://github.com/router-for-me/CLIProxyAPI/releases/tag/v7.2.155) 和 [官方配置示例](https://github.com/router-for-me/CLIProxyAPI/blob/v7.2.155/config.example.yaml)。Darwin arm64 发布包 SHA-256 为 `f90c503ce41a798c85b6f61dfe5fe8b812c1b889634f0c80d04ee376424fe305`，amd64 为 `198794a2fafb9fb8083476ac18232647c57d443422aa3f008d19ed7e75ca4604`；本机 arm64 可执行文件 SHA-256 为 `29d978064c49874a54126b161266b6b8a42b971a7d1ffcf742920761333d40d0`。
- 启动只允许 `127.0.0.1:8317`、私有随机下游 key、专用工作目录和专用 `HOME/TMPDIR`。下载包和可执行文件在启动前校验；不采用全局安装、全局配置或持久化 PID。
- 运行环境清除继承变量和代理，关闭管理面板、插件、请求日志、隐式重试和自动切换；配置文件为私有目录下的 `0600` 文件，停止、崩溃清理和新会话恢复都不信任旧监听者。
- 网关 I/O、进程等待和退避在 Control Plane 锁外执行。监控只重启自己拥有的子进程，退避为 1/2/4 秒，三次失败后进入 `FAILED`；账户控制面仍可响应。应用退出会终止自己启动的进程。

## #11 验收矩阵

| 验收点 | 证据 |
| --- | --- |
| 双语 Backend §41–42、公共命令/事件、SQLite 投影、Settings 面板 | `src-tauri/src/{gateway.rs,lib.rs,protocol.rs,storage.rs,main.rs}`、`src-tauri/src/bin/ipc.rs`、`src/Models.tsx`；EN/ZH ARD 同步 `model-gateway`、`model.gateway.changed`。Rust 集成测试覆盖 `model.get_gateway`、`domain.subscribe` replay 和 workspace 隔离。 |
| 固定发布物与身份校验 | `src-tauri/src/gateway_process.rs` 固定架构摘要、HTTPS release URL、归档/可执行文件摘要和 0700/0600 权限检查；失败时 `installed=false`，不会显示已验证。 |
| 端口冲突、认证和秘密边界 | 真实 pinned binary ignored test 验证已有 `127.0.0.1:8317` 只报告冲突且不附着、不终止、不生成秘密；独立 probe 验证无 key 为 401、有 key 才能访问。 |
| 运行/停止/重启/崩溃恢复/清理 | `cargo test --test gateway pinned_gateway_lifecycle_uses_public_commands_without_model_readiness -- --ignored --test-threads=1` 通过；测试覆盖空路由、停止清理、崩溃和 1/2/4 秒退避，最终三次失败进入 `FAILED`。 |
| 公共 UI、空路由和窄屏 | 当前提交启动 `npm run dev:browser` 后，在隔离临时工作区通过真实 Settings 页面确认 `Installing → Running`、`Pinned binary verified`、`Discovered models: 0`、停止后的“Launch the gateway before probing it.”；既有 390/768 视口检查分别为 `scrollWidth 375/753`，无横向溢出。随后在 `npm run desktop` 的真实 Tauri 窗口再次确认 `Starting / probing → Running → Stopped`、固定版本、二进制校验和空路由阻断。未输入任何 broker/model 凭据。 |
| Ready 与后续模型能力边界 | 运行中的真实空 `data` 数组只显示 `No verified model route`；没有 OAuth、DeepSeek key、测试推理、Codex Turn 或交易路径，因此不会伪造 Ready。 |

## 可重跑检查

以下命令在 `dev@83b691195829f7f4db94a036c1aa33544dd9c8ad` 通过：

```text
cargo fmt --all -- --check
cargo test --test gateway -- --nocapture
cargo test --test gateway pinned_gateway_lifecycle_uses_public_commands_without_model_readiness -- --ignored --test-threads=1
cargo clippy --workspace --all-targets --all-features -- -D warnings
npm run check
npm run desktop:build
```

`npm run check` 结果包含 schema/typecheck/Vite/unit/Rust workspace/requirements traceability：201 requirements、70 screens、12 QA scenarios、23 baseline files；所有 active checks 通过，明确的 OS Keychain 与 gateway native tests 仍按测试定义为 ignored。桌面构建产物为 `target/release/tradex`。

## 串行代码审查

- **Standards：PASS。** 已读取 `AGENTS.md`、`docs/agents/domain.md`、`docs/agents/issue-tracker.md`、`docs/agents/triage-labels.md`；无额外 CONTRIBUTING/CODING_STANDARDS/CONTEXT/ADR 文件。格式、Clippy、前端和仓库检查均通过。
- **Spec：PASS。** #11 的八项 acceptance criteria 均有代码、真实进程、公共协议或浏览器证据；发现的安装失败状态、陈旧配置清理顺序和 replay 覆盖问题已在同一提交修复并重跑相关验证。

未运行真实 OAuth、DeepSeek API key 或模型推理，因为这些属于后继 #12，且不能使用用户账户或凭据替代本票的生命周期验收。
