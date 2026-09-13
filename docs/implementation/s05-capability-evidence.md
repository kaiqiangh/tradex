# S05 / #21 能力策略与可信模式上下文验收

状态：**#21 VERIFIED（2026-09-13）；S05 父 Spec #20 仍 OPEN**。本证据只覆盖能力策略和可信 `turn.start` 边界；canonical context catalog/picker 由 #22 负责，typed research tool registry/result boundary 由 #23 负责，S33 全页面回归仍待后续工作项。

## 固定实现

- Spec 固定起始 SHA：`79a1c2e5da261c9eb2e9a57a6356b2bb8e12b38f`。
- 当前 `dev` 验收 SHA：`6d479024ac0a62592e71fbb9b49acc106215fb90`。
- 实现提交：`9ccd6ebdd6de7992d70b4cbddbce974ceb2a2793`（policy seam、IPC/UI）、`20645cb`（显式 null 与 probe 边界）、`356c8df`（query→turn parity 与未知 account/context 公共测试）、`6d47902`（默认测试与 integration-test parity 分离）。
- 生成物已由 `src-tauri/src/protocol.rs` 与 `src-tauri/src/capability.rs` 同步生成：`shared/ipc-v1.schema.json`、`shared/ipc-types.ts`、`shared/ipc-validators.js`。

## 交付范围

- Control Plane 提供版本化 `agent.capabilities` 查询，输入独立的 Agent Mode、Execution Context、可选账户和 canonical context refs，输出 `CapabilityLevel`（C0–C6）、typed `ToolId`、`executionAllowed` 和阻断原因。
- 策略矩阵覆盖 Ask/Research 只读、Backtest 历史仿真、Trade Local Paper、Alpaca Paper、Trading 212 Demo/Live、Binance Testnet/Live、Bitget Demo/Live。Live 最多返回 C4 proposal capability；C5/C6、未知 tool、非法账户/上下文组合均 fail closed。
- `turn.start` 在任何模型/runtime 工作前重新计算同一策略，并把 capability level 与模式、上下文、账户、环境和 refs 写入不可变 Turn snapshot。拒绝请求不写入 domain state。
- Composer 将 Agent Mode 与 Execution Context 分开显示，展示 capability level、允许工具、provider/model disclosure 和非法组合原因；能力查询失败或缺失时阻止 Create/Send。
- `CapabilityQuery` 的可选字段可省略但拒绝显式 `null`；`requested_tool` 在后端再次执行非空、64 字符和控制字符边界检查。

## 自动验证

以下命令均在当前 SHA 运行并通过：

```text
npm run check
cargo test --workspace --features integration-test --lib -- --test-threads=1
cargo clippy --workspace --all-targets --features integration-test -- -D warnings
node --check tests/thread-ui.mjs
git diff --check
```

`npm run check` 包含 schema/type agreement、TypeScript、Vite build、4 个前端单元测试、默认 Rust workspace **37 passed / 0 failed** 和需求追踪：**201 requirements, 70 screens, 12 QA scenarios, 23 baseline files**。integration-test Rust lib 为 **42 passed / 0 failed**，其中新增 capability matrix、null/boundary、未知 account/context 和 query→`turn.start`→snapshot parity 均通过。构建只有既有 bundle size warning；未执行被定义为 ignored 的真实 Keychain/真实网关测试。

## 隔离浏览器验证

使用 `npm run dev:browser` 的真实 Rust stdio/SQLite bridge，在新建隔离 workspace `013e2e49-3dd2-4289-9998-45e3f4b7cb85`（`S05 Capability QA`）中通过：

- `TRADE + NONE_READ_ONLY` 显示 `This mode and execution context cannot be used together.`，Create disabled；没有静默重映射。
- `RESEARCH + NONE_READ_ONLY` 显示 `Capability: C0` 和 `Public market read`，创建 Thread 后 capability summary 在详情和 reload 后仍存在。
- 390px 与 768px viewport 的 `scrollWidth` 分别为 375/753，与 client width 相等；无横向溢出，页面无 alert/error。
- 当前隔离 bridge 未配置模型网关，因此未在浏览器发送真实 inference Turn；Send 的不可用状态按契约显示。Rust integration-test parity 覆盖了可信 `turn.start` 快照路径。

验证期间未读取、写入或删除真实 ChatGPT OAuth、DeepSeek key 或 broker credentials；验证服务、临时目录和调试日志已清理，`tests/integration-bridge.ts` 无工作区差异。

## 串行代码审查

- **Standards：PASS。** `AGENTS.md`、`docs/agents/domain.md`、`issue-tracker.md`、`triage-labels.md` 和 Backend §41–42 边界已复核；显式 null、运行时长度校验和生成物均闭环。
- **Spec（#21）：PASS。** 六项 acceptance 均有实现或公共测试证据；#22/#23 的 catalog/picker 与 typed tool/result 边界按 blocker 保留，不提前宣称完成。

## 未覆盖边界

- #22 仍负责账户/上下文 catalog、稳定非秘密 hash、`@ Context` Attach/Cancel/remove 和完整 picker 可访问性；当前 S05 只校验已传入 refs。
- #23 仍负责真实 typed research tool registry、结构化结果 marker、prompt injection/financial command/gateway/keychain 负例和工具子进程边界。
- S33 的 QA-01–12 全页面回归、真实外部行情/研究数据和 configured gateway 的浏览器 inference 不由 #21 证据替代。
