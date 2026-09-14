# S08 #31 Market session / corporate-action acceptance evidence

验证日期：2026-09-14
固定实现代码点：`813a75a00497d7746167c3876583f80e07b0193b`
桌面 Clippy 修复点：`dc8e8c2e4b2efa59e98320f2ecdf297108d0e199`（`ReplyData::MarketDetail` 装箱；wire schema 未变化）
证据记录提交点：ece04c5de3413aa13bf7437729ad9b701a31b4f3
浏览器地址：`http://127.0.0.1:1420/`（integration mode，隔离临时 workspace）

## Rust、协议与边界

- `MarketDetail` 新增 typed `MarketState`、有界 `CorporateAction[]` 和 `AdjustmentStatus`；枚举覆盖 equity session、crypto venue failure 与 split/dividend/symbol-change/delisting。
- `market.get` 读取 workspace-scoped TimeService confidence 和 OD-005 calendar/corporate-action source gate。当前 OD-005 为 `BLOCKED_EXTERNAL`，AAPL 明确返回 `UNKNOWN` / `BLOCKED_EXTERNAL` / `UNAVAILABLE`，crypto 明确返回 `UNKNOWN` / `UNAVAILABLE`；没有 source 时不推断 `OPEN`、next boundary 或 adjusted history。
- corporate-action validation 拒绝未知 canonical instrument、重复 action ID、控制字符、无效 RFC 3339 时间和超过 16 条记录；不写 SQLite、DuckDB、outbox、account、risk 或 thread。
- `market_execution_eligibility` 先消费 `TimeService::require_trusted`，再将 `CLOSED` / `HALTED` 映射为 `MARKET_CLOSED` / `INSTRUMENT_HALTED`；maintenance、suspended、degraded、unknown 继续 fail closed。当前 S08 不实现 order、approval、risk、reservation 或 gateway。

## 检查命令

- `npm run schema:check`、`npm run typecheck`：通过，Rust / JSON Schema / TypeScript 一致。
- `cargo test -p tradex --lib market::tests -- --nocapture`：6/6 通过（source gate、session gate、duplicate/timestamp validation、既有历史边界）。
- `cargo test -p tradex --test market -- --nocapture`：2/2 通过（typed detail、workspace/domain read-only、unknown field fail closed）。
- `npm run check`：通过；schema、build、4 个前端单测、workspace Rust tests、requirements traceability 均通过。
- desktop `cargo check`、desktop clippy、`cargo fmt --all -- --check`、`git diff --check`：通过。

## Browser / Rust-backed UI

- Markets → AAPL detail 展示 `Market session: Unknown`、`CLOCK_UNCERTAIN` / `TRUSTED`、venue、OD-005 source status、next open/close、calendar version、provider time、observed time 与恢复入口。
- Corporate actions 面板展示 `History adjustment: Unavailable`，并明确没有 authoritative records、历史数据不标记为 adjusted。
- `Review calendar source` 是键盘可达语义 button，并跳转 Data & Storage 的 OD-005 gate；Account Health 的 `Synchronize time` 完成后返回 Markets，详情 `timeConfidence` 变为 `TRUSTED` 且没有隐式改写 session。
- 视口验证：1280×768、768×768、390×844；每个视口 `document.documentElement.scrollWidth === window.innerWidth`，状态与 corporate-action 文本仍可见。控制台 `error` / `warn` 为空。

## External gate and scope

真实 OD-005 日历/公司行为 entitlement、真实 provider session/halt/action payload、原生辅助技术读屏与 S33 全量回归仍未验证；它们属于外部授权或后续回归范围。任何 fixture 只证明 typed contract/rendering，不代表实时市场资格。
