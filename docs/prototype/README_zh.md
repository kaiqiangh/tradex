# TradeX 高保真可交互 Prototype — v1.0 RevC

该 standalone、无需 build 的可点击 prototype 与以下 RevC 文档对齐：

- `../docs/zh/TradeX_PRD_v1.0_RevC_zh.md`
- `../docs/zh/TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md`
- `../docs/zh/TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md`
- `../docs/zh/TradeX_Prototype_QA_Report_v1.0_RevC_zh.md`

这是 **产品/设计 review 与工程交接 prototype**，不是 production trading client。所有 market/account/order/model 行为均为 fixture，不会连接真实 broker、exchange、market-data service、Codex App Server、CLIProxyAPI 或 LLM provider。

## 运行

```bash
cd prototype
python3 -m http.server 8080
```

打开 `http://localhost:8080`。

多数浏览器也可直接打开 `index.html`。

## Revision C 产品模型

TradeX 不再把 Paper 和 Live 当作 Agent Mode。

**Agent Mode** 表示任务类型：

- Ask
- Research
- Backtest
- Trade

**Execution Context** 表示所选 execution/account 环境：

- None / Read-only
- Local Paper
- Alpaca Paper
- Trading 212 Demo / Live
- Binance Testnet / Live
- Bitget Demo / Live

选择 `Trade` 本身不会授予任何执行权限。Live transaction 还必须满足：精确 Live account 已 ARMED、deterministic validation、immutable proposal、transaction-specific approval、capacity reservation，以及 pre-execution revalidation。

## 主要 Review Flow

1. **Onboarding** — Workspace → Providers → Model → Risk defaults → Ready。
2. **LLM setup** — CLIProxyAPI/ChatGPT OAuth + DeepSeek；model discovery/health；跨 provider 自动 fallback 默认 OFF。
3. **Ask** — 只读轻量分析，不显示 execution action。
4. **Research** — plan/tool/result/artifact/Turn provenance。
5. **Backtest** — deterministic manifest + Sharpe、Sortino、drawdown、Profit Factor、Turnover。
6. **Trade context** — Agent Mode 与 Paper/Demo/Testnet/Live account 独立选择。
7. **Account-scoped Live arming** — arm 一个 Live account，切换账户不继承；可 Disable All。
8. **Live limit approval** — immutable proposal ID/hash、risk policy version、完整 market-data provenance、reservation、one-time approval。
9. **Market-order safety** — expected notional + maximum authorized spend。
10. **Reservation conflict** — Available / Reserved / Effective Available + deterministic rejection。
11. **Risk-policy invalidation** — 任何相关 policy change 都使 approval 失效；weakening 额外 disarm 对应 Live account。
12. **Ambiguous submission** — `UNKNOWN_RECONCILING`、frozen reservation、account unhealthy/disarmed、基于证据的 Manual Resolution。
13. **Broker rejection / cancellation** — acknowledgement 不等于 fill；cancel 同样需要 approval。
14. **Non-live execution** — Local Paper、Alpaca Paper、T212 Demo、Binance Testnet、Bitget Demo 全程保留 non-live identity。
15. **Markets** — 完整 provenance、market closed/halted/corporate-action fixture。
16. **Portfolio** — EUR normalization + FX/stablecoin route/source/timestamp/freshness/depeg-quality fixture。
17. **Settings** — Providers & Models / Risk & Limits / Data & Storage / Account Health / Appearance / About。
18. **Workspace portability** — 本地 Export / Workspace Import-Restore；没有 cloud Share-link 功能。
19. **Recovery** — auth、stream、rate、clock/freshness、LLM unavailable/quota/OAuth、startup/sleep reconciliation。
20. **Accessibility baseline** — visible focus、modal ARIA、live-region toast、reduced-motion CSS、带 More 的窄屏导航。

## Prototype 展示的安全 Invariant

- Live arming **按账户管理**，不是 global Boolean。
- 切换 account、Agent Mode、model/provider 不会继承或扩大 trading authority。
- 每个 Live order/cancel 都需要 transaction-specific approval。
- Approved proposal identity 不可变；material edit 必须生成新 proposal。
- Live approval 所使用的 market data 必须展示 source/provider timestamp/TradeX receive timestamp/venue/entitlement/freshness。
- 明确 `APPROVED → RESERVED → SUBMITTING`。
- Ambiguous submission 不允许 blind retry，也不允许 blind reservation release。
- 跨 provider LLM 自动 fallback 默认 OFF，必须显式 opt-in。
- withdrawal/transfer/custody/margin/leverage 等危险 broker permission 会在 Ready 前阻断/审查。
- Workspace restore 后全部 Live account 保持 DISARMED。

## Storage 术语

Revision C 统一使用：

- SQLite — transactional/domain state
- DuckDB — 1-minute+ OHLCV、analytics、backtests
- Filesystem — artifacts、exports、backups
- Parquet — Phase 2+ 大型 immutable dataset 的可选方案

MVP 默认不持续订阅或持久化 raw/tick/order-book 数据。
