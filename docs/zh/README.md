# docs/zh — TradeX 文档中文版

本目录是 TradeX 文档的**简体中文翻译版**。**英文版为权威版本(source of truth)**,两者不一致时以英文版为准。

| 中文版 | 英文权威版 |
|---|---|
| [TradeX_PRD_v1.0_RevB_zh.md](./TradeX_PRD_v1.0_RevB_zh.md) | `docs/TradeX_PRD_v1.0_RevB.md` |
| [TradeX_UI_Prototype_Spec_v1.0_RevB_zh.md](./TradeX_UI_Prototype_Spec_v1.0_RevB_zh.md) | `docs/TradeX_UI_Prototype_Spec_v1.0_RevB.md` |
| [TradeX_Prototype_Coverage_Matrix_v1.0_RevB_zh.md](./TradeX_Prototype_Coverage_Matrix_v1.0_RevB_zh.md) | `docs/TradeX_Prototype_Coverage_Matrix_v1.0_RevB.md` |

翻译基线:RevB(2026-09-04,commit `2bb13c9`)。英文版更新后中文版需同步重译对应章节。

## 术语约定

以下术语/标识在中文版中**保留英文原文**:

- 产品/组件名:Order Gateway、Codex App Server、Codex Harness、CLIProxyAPI、model_provider、Tauri、SQLite、DuckDB、Parquet、OS keychain、JSON-RPC/JSONL;
- 模型:`gpt-5.6-sol`、`gpt-5.6-luna`、`deepseek-chat`、`deepseek-reasoner`;
- 订单状态机(§45):DRAFT、PROPOSED、RISK_REJECTED、NEEDS_APPROVAL、APPROVED、RESERVED、SUBMITTING、ACCEPTED、PARTIALLY_FILLED、FILLED、CANCEL_PENDING、CANCELLED、REJECTED、EXPIRED、UNKNOWN_RECONCILING;
- 账户武装态:DISARMED / ARMED;环境名:Paper / Demo / Testnet / Live;模式名:Ask / Research / Backtest;
- 错误分类(§51):AUTH_ERROR、PERMISSION_ERROR、RATE_LIMITED、NETWORK_ERROR、UNSUPPORTED_CAPABILITY、MARKET_CLOSED、INSTRUMENT_HALTED、INVALID_ORDER、INSUFFICIENT_FUNDS、RISK_REJECTED、SUBMISSION_REJECTED、SUBMISSION_AMBIGUOUS、STREAM_DISCONNECTED、STATE_STALE、RECONCILIATION_REQUIRED、MODEL_UNAVAILABLE、QUOTA_EXCEEDED、OAUTH_EXPIRED、INTERNAL_ERROR;
- 全部追溯 ID:FR-xxx、AC-xxx、NFR-xxx、SEC-xxx、DATA-xxx、OPS-xxx、UX-xxx、OD-xxx;章节引用(§16.1);优先级(P0/P1/P2);状态值(Covered / Partial / Visual-only / Not covered / QA pending 等)。

## 阅读顺序(同英文权威链)

1. PRD(产品与安全模型,源头)
2. UI Prototype Spec(UI 目标规范)
3. Prototype Coverage Matrix(需求 ↔ 原型追溯)
4. QA Report(仅英文:`docs/TradeX_Prototype_QA_Report_v1.0_RevA.md`,暂无中文版)
5. `docs/prototype/` 可点击原型
