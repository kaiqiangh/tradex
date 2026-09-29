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

Account-interface scope update dated 2026-09-25: Binance Spot Testnet issues #67–#72 and Bitget Demo issues #73/#74/#76/#77/#78 are closed. Bitget Live read-only #75 is separate. This retires dedicated Testnet/Demo provider acceptance; it does not claim provider-hosted Testnet or Demo behavior was verified. Existing local fixture work remains local-only evidence. Future Binance/Bitget Live execution remains gated by S21–S30, including the S29/S30 provider gates.

S19 #68 adds a Binance Spot Testnet submission/recovery contract in Backend ARD §41.25, Frontend ARD §13.14, and UI Spec §14.12. Fixture tests exercise the local adapter and durable state transitions; they do not establish provider-hosted Testnet runtime evidence. The clickable prototype remains unchanged.

S19 #70 adds the Binance Spot Testnet signed private-stream and REST-recovery contract in Backend ARD §41.27, Frontend ARD §13.15, and UI Spec §14.13. The implementation is fixture-verified on `dev`; no real Testnet credentials, provider calls, or orders were used. Parent Spec #67 and provider acceptance #72 are closed under the updated ordinary-account scope; this fixture evidence is not provider-hosted Testnet acceptance. The clickable prototype remains unchanged.

S19 #71 adds exact-order Binance Spot Testnet cancellation in Backend ARD §41.28, Frontend ARD §13.15.1, and UI Spec §14.14. Rust-backed fixtures cover durable intent, one exact DELETE, ambiguity without replay, remote changes, and racing fills. [Local implementation evidence](./implementation/s19-binance-testnet-cancel-evidence.md). No real Testnet credentials, provider calls, or orders were used; parent Spec #67 and provider acceptance #72 are closed under the updated ordinary-account scope. The clickable prototype is unchanged.

S25.1 #97 implements and locally verifies query-only reconciliation evidence for a Trading 212 Live `PLACE` attempt that remains `UNKNOWN_RECONCILING`. The five-minute timeout keeps capacity frozen, and inconclusive evidence never proves absence or triggers a resend. Only the backend-authorized Keep Reconciling action is available after timeout; it records the decision without restarting provider reads. [Local evidence](./implementation/s25-trading212-live-reconciliation-evidence.md). No real provider requests were made and the clickable prototype is unchanged.

S26.2 #104 locally verifies durable Trading 212 Live cancellation history and a racing full fill. The exact cancellation was sent once through the isolated Gateway; a later exact-order refresh recorded `FILLED` while the cancellation attempt remained `CANCEL_PENDING`, removed the order from the open-order projection, and did not send another provider write. [Local evidence](./implementation/s26-2-trading212-live-cancel-evidence.md). Only synthetic credentials and a local fake provider were used; the clickable prototype remains unchanged.

S25.2 #98 adds the same query-only evidence path for Binance Spot Live. It verifies the saved SPOT account identity, queries only the exact saved `providerClientOrderId` on the ordinary Live endpoint, and keeps unmatched or incomplete results inconclusive with capacity frozen. [Local evidence](./implementation/s25-binance-live-reconciliation-evidence.md). No real provider requests or writes were made; the clickable prototype is unchanged.

S25.3 #99 extends the shared query-only evidence path to Bitget Classic Spot Live. It verifies the saved remote account and exact persisted `clientOid`, and keeps account/intent mismatches, empty or malformed responses, authentication failures, and transport failures inconclusive with capacity frozen. [Local evidence](./implementation/s25-bitget-live-reconciliation-evidence.md). Verification used synthetic credentials and local fixtures only; no real provider requests or writes were made, and the clickable prototype is unchanged.

S27.1 #108 implements startup/workspace-reopen recovery for connected Trading 212, Binance Spot, and Bitget Spot Live accounts. It stales and disarms accounts, refreshes fresh local time, dispatches exact PLACE evidence reads before overlapping account reads, commits exact evidence before account projections, keeps unknown PLACE/CANCEL attempts blocking, and exposes bounded recent-order history separately from open orders. [Local evidence](./implementation/s27-1-live-startup-recovery-evidence.md). `npm run check` passes and standards review passes. Spec review leaves the `<5 s` native timing acceptance open: both an ad-hoc wrapper and the standard development app waited at macOS Keychain before provider HTTP; the default workspace's Live projection remained unchanged. #108 remains open; the clickable prototype is unchanged.

S25.4 #100 is closed at reviewed `dev@f67463d` within the narrowed scope: fresh exact Binance/Bitget Live candidates can be confirmed as submitted; inconclusive evidence remains Keep-only, and reservations stay active. [Local evidence](./implementation/s25-4-live-manual-resolution-evidence.md) records the passing serial Standards/Spec reviews, full check, and [resolution comment](https://github.com/kaiqiangh/tradex/issues/100#issuecomment-5863195499). Local Rust-backed UI flows used synthetic fixtures; no real provider calls or writes were made, and the clickable prototype is unchanged. S25.5 research #101 is now closed: reviewed official docs for Trading 212, Binance Spot, and Bitget Classic Spot define no bounded order-visibility rule by which an empty exact query proves non-acceptance, so uncertain attempts stay Keep-only with frozen reservations. Binance's direct `-2010 NEW_ORDER_REJECTED` response is separate provider-rejection evidence only when durably tied to its one PLACE; it does not make `-2013`, a timeout, or query absence conclusive. [Research findings](./implementation/s25-live-absence-proof-research.md) and [resolution](https://github.com/kaiqiangh/tradex/issues/101#issuecomment-5863600151). S25 parent [#96 is closed after final acceptance](https://github.com/kaiqiangh/tradex/issues/96#issuecomment-5863761675); no real provider requests or order writes were used.

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
