# TradeX Documentation — v1.0 RevC

This directory is the Revision C product/design baseline.

## Normative reading order

1. [`TradeX_PRD_v1.0_RevC.md`](./TradeX_PRD_v1.0_RevC.md) — product, architecture constraints, safety model and acceptance criteria.
2. [`TradeX_UI_Prototype_Spec_v1.0_RevC.md`](./TradeX_UI_Prototype_Spec_v1.0_RevC.md) — target UI/state behavior.
3. [`TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md`](./TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md) — PRD/AC ↔ prototype traceability.
4. [`TradeX_Prototype_QA_Report_v1.0_RevC.md`](./TradeX_Prototype_QA_Report_v1.0_RevC.md) — source-level QA and remaining implementation QA gates.
5. [`../prototype/`](../prototype/) — clickable HTML/CSS/JS prototype.

## Chinese synchronized set

The `zh/` directory contains a synchronized Simplified Chinese edition with the same FR/AC identifiers and Revision C decisions:

- `zh/TradeX_PRD_v1.0_RevC_zh.md`
- `zh/TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md`
- `zh/TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md`
- `zh/TradeX_Prototype_QA_Report_v1.0_RevC_zh.md`
- `zh/README.md`

The English PRD remains the normative source for implementation interpretation; the Chinese version is maintained in semantic/ID parity rather than as a shortened summary.

## Revision C terminology

Keep these identifiers/terms stable across code and translations:

- Agent Mode: `Ask / Research / Backtest / Trade`
- Execution Context: `None / Read-only / Local Paper / Paper / Demo / Testnet / Live`
- Live arming: `DISARMED / ARMED`, scoped to account ID
- LLM route: `CLIProxyAPI → ChatGPT` or `CLIProxyAPI → DeepSeek`
- Proposal flow: `OrderDraft → OrderProposal`
- Core order states: `DRAFT / PROPOSED / RISK_REJECTED / NEEDS_APPROVAL / APPROVED / RESERVED / SUBMITTING / ACCEPTED / PARTIALLY_FILLED / FILLED / CANCEL_PENDING / CANCELLED / REJECTED / EXPIRED / UNKNOWN_RECONCILING`
- Recovery: `Manual Resolution`, not blind reservation release
- Storage: SQLite transactional/domain; DuckDB MVP 1m+ analytics; Filesystem artifacts; Parquet optional Phase 2+
- Traceability IDs: `FR-*`, `AC-*`, `NFR-*`, `SEC-*`, `DATA-*`, `OPS-*`, `UX-*`, `OD-*`
