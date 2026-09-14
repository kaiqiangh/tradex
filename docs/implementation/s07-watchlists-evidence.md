# S07 #28 Watchlists CRUD 验收证据

验证日期：2026-09-14  
固定实现代码点：`1e157ae`
前端播报代码点：`318936a9888d5f1c8500352b488001b12d81f1cd`  
浏览器地址：`http://127.0.0.1:1420/`（integration mode，隔离临时 workspace）

## Rust、协议与持久化

- `npm run schema:check`：通过，watchlist 输入/输出、错误和 Rust / JSON Schema / TypeScript 一致。
- `cargo test --test watchlists -- --nocapture`：3/3 通过。
  - `watchlists_are_versioned_ordered_idempotent_and_persistent` 覆盖空列表、create、大小写不敏感重名、canonical AAPL/MSFT 有序 add、重复 add/remove 幂等、stale remove 无变更、rename、未知 instrument、删除及删除后 not-found；删除前关闭并重开 workspace，确认重开的非空列表仍含 `equity:US:MSFT`。
  - `watchlist_schema_rejects_unknown_fields_and_invalid_names` 还验证 Unicode `Ångström`/`ångström` 重名被拒绝，避免 SQLite ASCII-only `NOCASE` 漏洞。
  - `watchlist_schema_rejects_unknown_fields_and_invalid_names` 覆盖未知字段、控制字符名称，以及 SQLite 表 `name` 与 JSON projection 名称篡改时的 `WORKSPACE_INTEGRITY_FAILED`。
  - `schema_six_workspaces_migrate_watchlists_transactionally` 覆盖 schema 6→7 迁移和重开。
- `cargo fmt --all`：通过；`git diff --check`：通过。
- Watchlist projection 只包含 list 元数据与 canonical member IDs，不包含 credentials、quotes 或 provider response。每次 mutation 在一个 immediate SQLite transaction 中执行 per-list CAS；不写账户、模型、风险、Thread 或 outbox/event 状态。该 v1.0 collection boundary 与双语 Backend ARD §41.11 一致。

## Browser / Rust-backed UI

隔离 workspace：`e1a7ad58-7457-46ec-80ef-ae3658bcd964`，路径由 integration runner 临时生成，provider response 明确标记为 test fixture。

- Watchlists 页面只渲染一次；D1 library、D2 detail、D3 New Watchlist、D4 Add Instrument 均可达。
- UI create `QA evidence list` 后出现 `role="status"` live message `Created “QA evidence list”.`；添加 `equity:US:AAPL` 后出现 `Instrument added.`；rename 为 `QA evidence renamed` 后出现 `Watchlist renamed.`；remove AAPL 后出现 `Instrument removed.`。
- Delete 按钮先进入 `Delete this list?` / `Confirm delete` / `Cancel` 两步确认状态；本轮只验证可达与可取消。最终删除由 Rust-backed `watchlist.delete` 测试验证，避免在图形界面执行本地 destructive delete。
- 重载页面后，先前已存在的 `Core + Tech` 列表和 `equity:US:MSFT` 成员仍显示；Rust 重开测试同时证明非空成员持久化。
- stale/error 边界由 Rust-backed command 验证：旧 `expectedStateVersion` 返回 `STATE_VERSION_CONFLICT` 且列表不变；未知 instrument 返回 `MARKET_INSTRUMENT_NOT_FOUND`；重复删除返回 `WATCHLIST_NOT_FOUND`。UI 显示错误并提供 Reload。
- 键盘路径验证了 Primary navigation、More 折叠/展开、Watchlists catalog 搜索，以及 canonical `crypto:BTC/USDT:spot` 结果可见；所有操作控件是 semantic button/input。
- 768px CSS viewport：`innerWidth=768`、`scrollWidth=clientWidth=bodyScrollWidth=768`。
- 390px CSS viewport：`innerWidth=390`、`scrollWidth=clientWidth=bodyScrollWidth=390`；More 菜单与 Watchlists 路由可达，无水平溢出。
- 当前最终 worktree（1280px）DOM 确认 `status: Instrument removed.`，`Delete this list?` 可进入并取消；TradeX app console `error`/`warn`（过滤 `127.0.0.1`）为空。

## Checks

- `npm run check`：EXIT 0；包括 `npm run schema:check`、Vite build、TypeScript、4/4 frontend unit tests、Rust workspace tests（55 unit + integration；显式 native Keychain/gateway tests保持 ignored）和 `python3 scripts/check_requirements.py`（201 requirements、70 screens、12 QA、23 baseline）。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- Vite 仅提示 bundle 大于 500 kB；不影响本 slice 行为。

## External gate and scope

OD-001（Alpaca realtime）与 OD-002（Alpaca historical）在 S06 authorization catalog 中仍是 `BLOCKED_EXTERNAL`。本票没有访问 provider、读取凭据或把 fixture 当作真实行情；Watchlists 保存 canonical identity，真实 quote/history entitlement 继续由 S08+ 数据切片负责。S33 全应用跨页面回归仍未完成。
