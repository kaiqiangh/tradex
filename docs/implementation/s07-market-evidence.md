# S07 #27 市场目录、详情与历史边界证据

验证日期：2026-09-14  
固定代码点：`fd5e0a2089fb70063561c09f5faafac04a496e33`  
浏览器地址：`http://127.0.0.1:1420/`（integration mode，隔离临时 workspace）

## Rust、协议与构建

- `npm run schema:check`：通过，Rust / JSON Schema / TypeScript 一致。
- `npm run build`：通过（包含 `npm run typecheck`；Vite 仅提示 bundle 大于 500 kB）。
- `npm run test:unit`：4/4 通过。
- `cargo test --workspace --all-targets`：55 个 Rust 单元测试通过；集成测试通过；仅显式标记的 native keychain / gateway 测试保持 ignored。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo fmt --all`：通过。
- `python3 scripts/check_requirements.py`：201 requirements、70 screens、12 QA、23 baseline 通过。
- DuckDB 边界测试：阻断的 OD-002 写入返回 `MARKET_HISTORY_UNAVAILABLE` 且行数保持 0；可用的测试源可写入并重开读取 1 行；crypto instrument 与 `REALTIME` entitlement 在历史写入边界拒绝；同路径 workspace reopen 会重建缺失的 `ohlcv_1m` 表。

## Browser / Rust-backed UI

- 首屏 Markets 目录显示 4 个规范 instrument ID，并展示资产类别与 venue facets；混合结果不返回单一 source ID，状态为 `UNAVAILABLE`。
- 搜索 `AAPL` 返回 1 行；选择或按 Enter 后打开精确 `equity:US:AAPL` / `HOT` 详情，状态为 `BLOCKED_EXTERNAL`、source `OD-001`，不显示 quote/chart。
- 搜索 `BTC/USDT` 返回精确 `crypto:BTC/USDT:spot`；详情状态为 `UNAVAILABLE`，source 为 `No source selected`，原因明确说明 S06 尚未选定 crypto market-data source，不显示 quote/chart。
- unavailable 详情的 `Open Data & Storage settings` 将 Settings 的 `Data & Storage` tab 置为 pressed，并显示 OD-001/OD-002 的阻断门禁。
- 768px CSS viewport：`innerWidth=768`、`scrollWidth=clientWidth=bodyScrollWidth=768`；键盘 Enter 能打开 AAPL 详情。
- 390px CSS viewport：`innerWidth=390`、`scrollWidth=clientWidth=bodyScrollWidth=390`；More 菜单可见且可展开，Primary navigation 可达。
- TradeX IAB `error` / `warn` console 日志为空。

## External gate

OD-001（Alpaca realtime）与 OD-002（Alpaca historical）在 S06 authorization catalog 中仍为 `BLOCKED_EXTERNAL`。本轮没有访问 provider、读取或写入凭据，也没有把 fixture 或测试行当作真实 entitlement；真实 quote/history 仍需后续 adapter 与 entitlement 验证。

Watchlists CRUD 属于 S07 子项 #28，按串行工作流暂不纳入本证据或 #27 的关闭范围。
