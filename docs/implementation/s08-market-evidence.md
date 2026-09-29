# S08 #31 Market session / corporate-action acceptance evidence

验证日期：2026-09-14
固定实现代码点：`90060eab99f26fe8218da4dc6666d6e061ee4314`（包含初始 market detail 边界 `813a75a00497d7746167c3876583f80e07b0193b`、权威门禁修复 `de7b5eee14c522bac374b81ddc4cf2b32fdd2165`）
桌面 Clippy 修复点：`dc8e8c2e4b2efa59e98320f2ecdf297108d0e199`（`ReplyData::MarketDetail` 装箱；wire schema 未变化）
证据记录提交点：ce15da5774f5a30288e929b67331ba4ac2df2730
浏览器地址：`http://127.0.0.1:1420/`（integration mode，隔离临时 workspace）

## Rust、协议与边界

- `MarketDetail` 新增 typed `MarketState`、有界 `CorporateAction[]` 和 `AdjustmentStatus`；枚举覆盖 equity session、crypto venue failure 与 split/dividend/symbol-change/delisting。
- `market.get` 读取 workspace-scoped TimeService confidence 和 OD-005 calendar/corporate-action source gate。当前 OD-005 为 `BLOCKED_EXTERNAL`，AAPL 明确返回 `UNKNOWN` / `BLOCKED_EXTERNAL` / `UNAVAILABLE`，crypto 明确返回 `UNKNOWN` / `UNAVAILABLE`；没有 source 时不推断 `OPEN`、next boundary 或 adjusted history。
- bounded market-state observation seam validates source/venue/time fields and covers equity regular/holiday/half-day/extended/closed/halted plus crypto maintenance/suspended/degraded fixtures; blocked sources still return UNKNOWN/UNAVAILABLE. The integration-only `TRADEX_MARKET_FIXTURE` seam routes deterministic synthetic states through `market.get` while production remains source-gated.
- corporate-action validation rejects unknown canonical instrument, duplicate action ID, control characters, invalid RFC 3339 time and more than 16 records, then orders accepted records by effective time/action ID; the integration fixture exposes split/dividend/symbol-change/delisting records with `UNKNOWN` adjustment and no SQLite, DuckDB, outbox, account, risk or thread writes occur.
- observations cannot claim `OPEN`/`CLOSED`/`HALTED` against a blocked or unverified OD-005 source; a negative test rejects the mismatched source status before projection.
- Markets uses the reusable `ErrorRecoveryPanel` for `MARKET_CLOSED` and `INSTRUMENT_HALTED`, with keyboard-reachable focus remediation and `aria-live` announcement.
- `market_execution_eligibility` first consumes `TimeService::require_trusted`, then requires an AVAILABLE source and known adjustment before allowing OPEN/EXTENDED_HOURS; CLOSED/HALTED map to `MARKET_CLOSED`/`INSTRUMENT_HALTED`, and maintenance/suspended/degraded/unknown remain fail closed. Current S08 does not implement order, approval, risk, reservation or gateway.

## 检查命令

- `npm run schema:check`、`npm run typecheck`：通过，Rust / JSON Schema / TypeScript 一致。
- `cargo test -p tradex --lib market::tests -- --nocapture`：9/9 通过（source gate、fixture session taxonomy、integration fixture detail、eligibility source/adjustment gate、duplicate/timestamp validation、既有历史边界）。
- `cargo test -p tradex --test market -- --nocapture`：2/2 通过（typed detail、workspace/domain read-only、unknown field fail closed）。
- `npm run check`：通过；schema、build、4 个前端单测、workspace Rust tests、requirements traceability 均通过。
- desktop `cargo check`、desktop clippy、`cargo fmt --all -- --check`、`git diff --check`：通过。

## Browser / Rust-backed UI

- Markets → AAPL detail 展示合约 fixture 的 `Open`、`Unverified` source status、next open/close、calendar version、provider time、observed time 与 `CLOCK_UNCERTAIN` 初始时间门禁；行情源仍显示 `BLOCKED_EXTERNAL`，没有伪造 quote。
- AAPL corporate actions 面板展示 4 条 synthetic split/dividend/symbol-change/delisting 记录，effective/announced/source/adjustment 字段可见，History adjustment 保持 `Unknown`。
- Markets → MSFT detail 展示 synthetic `Closed` 状态、`MARKET_CLOSED` ErrorRecoveryPanel 和 `View market state`；点击后焦点回到 session 标题。BTC detail 展示 synthetic `Maintenance` venue state。
- Account Health 的 `Synchronize time` 完成后返回 Markets，详情 `timeConfidence` 变为 `TRUSTED` 且没有隐式改写 session；生产/非 fixture 路径仍由 OD-005 gate 返回 `UNKNOWN`/`UNAVAILABLE`（Rust integration 2/2 负例保留）。
- 视口验证：1280×768、768×768、390×844；每个视口 `document.documentElement.scrollWidth === window.innerWidth`，状态与 corporate-action 文本仍可见。控制台 `error` / `warn` 为空。

## External gate and scope

真实 OD-005 日历/公司行为 entitlement、真实 provider session/halt/action payload、原生辅助技术读屏与 S33 全量回归仍未验证；它们属于外部授权或后续回归范围。任何 fixture 只证明 typed contract/rendering，不代表实时市场资格。
