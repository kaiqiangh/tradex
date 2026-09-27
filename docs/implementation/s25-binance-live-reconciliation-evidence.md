# S25.2 Binance Spot Live reconciliation evidence

## Scope

Issue #98 adds read-only reconciliation for a saved Binance Spot Live PLACE attempt in `UNKNOWN_RECONCILING`. The shared surface checks the saved account and proposal identity, queries Binance's ordinary Live account and exact-order routes, and preserves uncertainty and the active reservation when no exact observation is available.

## Local verification

- Rust-backed integration fixture: a synthetic numeric Binance SPOT account and temporary SQLite workspace create an unknown attempt with its persisted `tx-{attempt UUID without hyphens}` client order ID. The fixture returns an exact candidate; Binance `-2013`, authentication failure, and a mismatched order remain inconclusive. Timeout disarms the account while retaining the attempt and reservation; Keep Reconciling is recorded without restarting provider reads.
- Browser verification: the Rust-backed integration bridge and local Vite surface show the saved Binance attempt, exact client order ID, candidate-only order status, and completed exact-order lookup. The reservation remains active.
- Responsive check: the evidence panel and document have no horizontal overflow at 1280, 768, and 390 px. The browser console has no application errors.
- Provider-write guard: the isolated fixture reports zero provider order POST writes, and the Rust mock verifies that reconciliation requests use GET only. No real Binance credentials or provider requests were used.
- Prototype boundary: `docs/prototype/` was not changed; this runtime fixture does not certify the clickable prototype or provider-hosted account behavior.

The parent S25 issue #96 remains open for the remaining provider and confirmed-submitted/not-submitted resolution scope.
