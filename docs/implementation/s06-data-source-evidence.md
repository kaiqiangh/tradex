# S06 / #24–#25 数据源与授权验收

状态：**已验收（2026-09-14）**。分支：`dev`。实现基线：`32aee84`（包含 `05d39c4` 与 `8247e06` 的研究 source fail-closed 修复）；文档收口提交：`10fb933`。Code-review 固定起点：`f172f81`。

## 交付

- `DataSourceCatalog` 是六项 OD-001–006 的单一静态政策目录；每项公开 provider、能力、覆盖、时效、entitlement、保留、再分发/商业使用/辖区限制、官方/条款 URL、审阅日期、探测类型、状态和脱敏原因。目录与 probe 不读取或返回 API key、secret、响应正文、账户余额或 Keychain 明文。
- Alpaca OD-001/002/005 在没有用户管理的 market-data/calendar entitlement 时保持 `BLOCKED_EXTERNAL`。OD-003 使用 SEC submissions 与固定 Apple XBRL Company Concept（`CIK0000320193/us-gaap/Revenues`）公开端点；OD-006 使用 ECB `EXR.D.USD.EUR.SP00.A` 日参考系列。公开 HTTP 成功只证明可达性，不授予交易、再分发或商业授权；OD-004 的 filings probe 成功仍因未选定 general-news provider 保持 `BLOCKED_EXTERNAL`。
- SEC 请求使用 identifying User-Agent、HTTPS/no-redirect、5 秒超时、1 MiB bounded body 和 10 req/s limiter；submissions 绑定固定 CIK、数组长度/日期，Company Concept 绑定 CIK/taxonomy/tag/entity/USD 且所有数据行有效；ECB 使用严格 UTF-8/CSV、固定 KEY/FREQ/CURRENCY、有效日期/有限数值且所有行有效。失败只返回脱敏原因。
- `research.run` 与 `turn.start` 在同一 Control Plane 目录快照上重算 gate：`public_market_read → OD-001`、`historical_simulation → OD-002`；非 `AVAILABLE` 时结果保持 `UNAVAILABLE`，`sourceId` 与状态写入 sanitized reason。`account_read` 继续使用 S02 的内部 `control-plane:account` 健康边界。研究结果仍带请求 hash、上下文 refs 和 marker，不能改变 Thread 或金融状态。
- 探测观察按 workspace/source 保存在 Control Plane 进程内存；同一进程 renderer reload/remount 保留，进程重启按架构回到静态 `UNVERIFIED`/`BLOCKED_EXTERNAL`，要求重新 probe。不会写 SQLite、日志、响应正文或凭据。

## 检查

以下命令在 `dev@32aee84` 通过：

- `RUST_TEST_THREADS=1 npm run check`：schema/TypeScript/Vite build、4 个前端单测、Rust workspace 单测与集成测试、requirements traceability（201 requirements / 70 screens / 12 QA scenarios / 23 baseline files）。
- `cargo clippy --workspace --all-targets --features integration-test -- -D warnings`、`cargo fmt --all -- --check`、`node --check tests/thread-ui.mjs`、`git diff --check`：通过。
- `cargo test --workspace --features integration-test typed_research_result_is_sanitized_and_tamper_evident -- --test-threads=1`、`cargo test --workspace data_sources -- --test-threads=1`：通过；wrong CIK/series/partial row、wrong series、invalid date/NaN、rate limit、研究 gate 和 no-mutation 负例均覆盖。

## 真实隔离 IPC 证据

使用新建临时 workspace 和 `target/debug/tradex-ipc`：

- `workspace.open` 后 `data.source.probe`：OD-003 返回 `AVAILABLE`（submissions + Company Concept shape 均通过）；OD-004 返回 `BLOCKED_EXTERNAL`（filings shape 通过但 news 未选定）；OD-006 返回 `AVAILABLE`；OD-001/002/005 返回 `BLOCKED_EXTERNAL` 且不读取凭据。
- 同一 workspace 的 `research.run(public_market_read)` 返回 `sourceId: OD-001`，reason 包含 `OD-001 is BLOCKED_EXTERNAL`；集成测试确认 probe/catalog/research 不改变 workspace/account/model/risk/thread state。
- 进程重启复核：首次 OD-006 为 `AVAILABLE` 且有 `checkedAt/observedAt`；使用同一目录重新打开后按约定回到 `UNVERIFIED` 且两项时间为空，workspace ID 保持不变。这是进程级观察缓存的明确边界，不是持久化数据源事实。

## 隔离浏览器证据

`npm run dev:browser` 启动 Rust-backed integration bridge 和唯一临时 workspace；没有输入、读取、修改或删除真实 ChatGPT OAuth、DeepSeek key 或 broker credential。CUA 在当前实现上验证：

- Settings → Data & Storage 渲染六张 OD 卡片、provider/coverage/entitlement/terms/status/retry；没有 password/secret 输入。OD-003 手动 probe 显示 `Available`；OD-004 显示 filings shape 成功但 general-news gate blocked；OD-006 显示 `Available`。
- `mod.checkDataSourceUI(tab, browser)` 返回：六项 policy 与 credentialed Alpaca gate 可见；768px/390px 下卡片、状态操作和 terms 链接可键盘到达且无横向溢出/浏览器错误。`tab.dev.logs({levels:['error']})` 返回 `[]`。
- 验证结束停止了该 Vite/临时 IPC 进程；没有启动用户的 TradeX 桌面应用，也没有执行真实下单或 entitlement 授权。

## 双轴审查

- **Standards：PASS** — `32aee84` 明确 Company Concept 来源、严格响应校验、研究 gate、workspace/source 隔离、进程缓存边界和生成 schema；未发现标准或 smell finding。
- **Spec：PASS** — 独立复审确认 S06 policy、probe、研究 gate、UI 与进程边界均有实现和证据。

S06 不提供真实行情、fundamentals/news/calendar/FX producer、付费订阅、key 编辑或任何 Live 授权；这些仍由后续 S07–S35 负责。下一前沿为 S07，当前不创建 dev→main PR。
