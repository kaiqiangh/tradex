# TradeX Documentation — v1.0 RevC

Revision C clarification dated 2026-09-05. Product scope and existing requirement IDs are retained. The documents define the target; the current clickable prototype has known gaps recorded in the QA Report.

## Authority and reading order

1. [PRD](./TradeX_PRD_v1.0_RevC.md) — source of truth for scope, financial authority, state semantics, and acceptance.
2. [UI / Prototype Spec](./TradeX_UI_Prototype_Spec_v1.0_RevC.md) — target screens and interactions; §14 defines detailed repair contracts.
3. [Frontend ARD](./TradeX_Frontend_ARD_v1.0_RevC.md) and [Backend ARD](./TradeX_Backend_ARD_v1.0_RevC.md) — implementation architecture under the PRD/UI requirements. Backend §41–42 own the shared wire contract; the frontend references it.
4. [Coverage Matrix](./TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md) — requirement-to-evidence status; ID presence does not mean acceptance.
5. [QA Report](./TradeX_Prototype_QA_Report_v1.0_RevC.md) — observed defects, QA-01–QA-13 regression cases, and incomplete runtime/browser gates.
6. [Clickable prototype](./prototype/README.md) — fixture evidence, subordinate to the normative documents.
7. [File manifest](./FILE_MANIFEST.md) — repository-relative paths, line counts, byte hashes, and language pairs.

The PRD wins on product/safety meaning. The UI Spec owns user-visible behavior, and the ARDs specify its implementation. A prototype observation or coverage status cannot override a requirement. Correct conflicting documents together rather than choosing a convenient interpretation.

## Current handoff status

The 2026-09-05 documentation revision clarifies expiry/reservation rules, Gateway process isolation and dispatch, canonical IPC, operation-preserving cancellation, evidence-based resolution, immutable history/proposals, and complete interaction expectations.

Prototype interaction/handoff remains **NOT PASS**: HTML/CSS/JS were not changed in this documentation revision. QA-13 records that the clickable prototype has no permanent eligible-account removal path; S18 #66 runtime implementation evidence is separate. Use the QA Report for observations and the UI Spec for intended behavior. This does not certify other broker/runtime integrations or close existing open product decisions.

S19 #68 adds a Binance Spot Testnet submission/recovery contract in Backend ARD §41.25, Frontend ARD §13.14, and UI Spec §14.12. Fixture tests exercise the local adapter and durable state transitions; they do not establish provider-hosted Testnet runtime evidence. The clickable prototype remains unchanged.

S19 #70 adds the Binance Spot Testnet signed private-stream and REST-recovery contract in Backend ARD §41.27, Frontend ARD §13.15, and UI Spec §14.13. The implementation is fixture-verified on `dev`; no real Testnet credentials, provider calls, or orders were used. Parent Spec #67 remains open for provider-hosted Testnet acceptance, and the clickable prototype remains unchanged.

## Chinese synchronized set

[Chinese index](./zh/README.md) links all six English/Chinese document pairs: PRD, UI Spec, frontend/backend ARDs, Coverage Matrix, and QA Report. [Chinese prototype guide](./prototype/README_zh.md) accompanies the same fixture.

Both languages preserve requirement IDs, A–K screen IDs, QA case IDs/statuses, command names, error/state enums, financial guards, and scope decisions. An English normative change requires the corresponding Chinese semantic update in the same change.

## Stable terminology

- Agent Mode: Ask / Research / Backtest / Trade.
- Execution Context: read-only/historical simulation or an explicit Local Paper / Paper / Demo / Testnet / Live environment.
- Arming: DISARMED / ARMED, keyed by account ID.
- LLM: CLIProxyAPI → ChatGPT or CLIProxyAPI → DeepSeek; cross-provider fallback opt-in.
- Proposal: OrderDraft → immutable OrderProposal; cancellation binds a distinct immutable CANCEL intent.
- UNKNOWN_RECONCILING: frozen capacity and evidence-based Manual Resolution.
- Storage: SQLite transactional/domain, DuckDB MVP 1m+ analytics, filesystem artifacts; Parquet optional Phase 2+.
- Coverage status: FAILED / PARTIAL / SOURCE_ONLY / RUNTIME_PENDING / DEFERRED.
