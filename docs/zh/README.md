# docs/zh — TradeX v1.0 RevC 中文文档

本目录是 TradeX Revision C 文档的**简体中文同步版**。不是摘要版：PRD 的 FR/AC ID、关键产品决策、安全 invariant、UI screen/state inventory 与英文版保持语义对齐。

英文 PRD 仍作为 implementation interpretation 的规范源；如存在翻译歧义，应以英文 PRD 的技术含义为准并同步修正中文，而不是允许两版长期分叉。

| 中文版 | 英文版 |
|---|---|
| [TradeX_PRD_v1.0_RevC_zh.md](./TradeX_PRD_v1.0_RevC_zh.md) | `../TradeX_PRD_v1.0_RevC.md` |
| [TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md](./TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md) | `../TradeX_UI_Prototype_Spec_v1.0_RevC.md` |
| [TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md](./TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md) | `../TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md` |
| [TradeX_Prototype_QA_Report_v1.0_RevC_zh.md](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md) | `../TradeX_Prototype_QA_Report_v1.0_RevC.md` |
| [`../../prototype/README_zh.md`](../../prototype/README_zh.md) | `../../prototype/README.md` |

## RevC 术语约定

以下标识保留英文原文或稳定 token：

- Agent Mode：`Ask / Research / Backtest / Trade`
- Execution Context：`None / Read-only / Local Paper / Paper / Demo / Testnet / Live`
- Live arming：`DISARMED / ARMED`，按 account ID 管理
- LLM route：`CLIProxyAPI → ChatGPT`、`CLIProxyAPI → DeepSeek`
- Proposal：`OrderDraft → OrderProposal`
- Order states：`DRAFT / PROPOSED / RISK_REJECTED / NEEDS_APPROVAL / APPROVED / RESERVED / SUBMITTING / ACCEPTED / PARTIALLY_FILLED / FILLED / CANCEL_PENDING / CANCELLED / REJECTED / EXPIRED / UNKNOWN_RECONCILING`
- Recovery：`Manual Resolution`，不翻译成或实现为 blind reservation release
- Storage：SQLite transactional/domain；DuckDB MVP 1m+ analytics；Filesystem artifacts；Parquet Phase 2+ optional
- 错误 token、model IDs、provider names、FR/AC/NFR/SEC/DATA/OPS/UX/OD IDs 保留原样。

## 对齐规则

1. PRD 中英文必须包含相同的 FR 与 AC ID 集合。
2. UI Spec 必须包含相同的 A–K Screen / State Inventory。
3. Coverage Matrix 的中英文 status 对同一 ID 必须语义一致。
4. QA Report 的 PASS/Pending 结论必须一致。
5. Prototype 中的 product-state vocabulary 必须以 RevC 为准，不再把 Paper/Live 当作 Agent Mode。
