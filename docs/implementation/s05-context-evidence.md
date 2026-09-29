# S05 / #22 上下文目录与 Composer 选择器验收

状态：**#22 VERIFIED（2026-09-13）**。实现固定 SHA：`b4454b62f04eef37e2af650e3d3965c4bb017397`。

## 交付范围

- Backend ARD §41.7 与 Frontend ARD §7.7 同步定义 `context.catalog`、canonical `account` ref、可用性和临时 picker 语义；中英文文档保持同一字段、限制和拒绝路径。
- Rust `context.catalog` 只读取当前 workspace 的已持久化账户，返回稳定的非秘密 `sha256:<64 lowercase hex>` hash、provider/environment、`readOnly`、`available` 和恢复原因；instrument/account/strategy/backtest/artifact 的空目录显式返回，账户列表上限为 256、空状态上限为 5。
- `turn.start` 在 schema 和运行时区分 omitted `attachedContexts`（沿用已保存 Thread refs）、显式 `[]`（清空本轮 refs）和显式 `null`（拒绝）；未知、重复、跨 workspace、不可用账户、unsupported kind 或 malformed hash 在持久化前 fail closed。
- Thread 与 Turn Composer 使用同一 catalog；`@ Context` 的 Attach/Cancel/remove 只改变临时 next-turn 列表。Live 账户在 Ask/Research 中显示 `LIVE · READ-ONLY` 并只提供 `account_read`，Trade 仍必须单独选择执行账户。
- picker 通过 portal 挂载，打开时将 `.app-shell` 设为 inert，标题先获得焦点，Tab/Shift+Tab 留在 dialog 内，Escape 关闭并把焦点还给触发按钮；账户操作会刷新 catalog，修复已保存账户不自动出现的问题。

## 验证

以下命令均在 `dev` 分支实现 SHA `b4454b62f04eef37e2af650e3d3965c4bb017397` 上执行并通过：

```text
npm run check
  schema/typecheck/build/unit/Rust workspace/traceability 全部通过；Vite 保留既有单 bundle >500 kB warning。
python3 scripts/check_requirements.py
  Traceability OK: 201 requirements, 70 screens, 12 QA scenarios, 23 baseline files.
npm run typecheck
npm run schema:check
  Rust / JSON Schema / TypeScript agree.
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features integration-test -- -D warnings
node --check tests/thread-ui.mjs
git diff --check
cargo test --workspace --features integration-test -- --test-threads=1
  49 library tests passed; every integration target's non-ignored tests passed。
```

隔离浏览器命令 `npm run dev:browser` 在 `http://127.0.0.1:1420/` 启动 Rust-backed bridge；`tests/thread-ui.mjs` 的 `checkThreadUI` 使用唯一临时 workspace 和合成 `binance/LIVE` fixture，完整通过以下可见路径：canonical account catalog、Live read-only disclosure、future empty state、Attach/Cancel/remove、非法 Trade + read-only 拦截、Thread create、RUNNING→COMPLETED、CANCELLED→Retry、reload/history，以及 768/390 无横向溢出。脚本新增并通过焦点断言：打开后 `h3` 为 active element、`.app-shell[inert]`、Shift+Tab 与 Tab wrap、Escape 关闭、关闭后 active element 为 picker trigger；浏览器 error 日志为零。

验证结束后已停止开发服务，并删除所有本次生成的 `tradex-browser-*` 临时 workspace；未读取、写入或删除真实 OAuth、DeepSeek、broker 凭据，也未触达真实金融执行。

## 保留边界

#23 typed research tool/result boundary、S33 全页面交叉回归、真实 provider/model 外部推理、真实账户数据和 Live 下单不属于 #22 证明范围，仍按 map 的后续串行票处理。

## 串行复审

- Standards：**PASS**。双语 ARD、生成 schema/validators、evidence 链接、安全边界和只读账户规则通过复审。
- Spec：**PASS**。catalog/hash/availability、workspace isolation、omitted/`[]`/`null`、picker 临时状态、Account gate、focus/inert/wrap/Escape/focus return 与 390/768 证据均通过复审。
