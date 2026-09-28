# S25.3 #99 Bitget Spot Live unknown PLACE reconciliation evidence

Date: 2026-09-28. Branch: `dev`. Review base: `0fa304f471e938a535aeef47b9bd5fd8af498afe`.

## Scope

This slice extends the shared S25 evidence ledger and Order Drafts panel to saved Bitget Classic Spot Live PLACE attempts in `UNKNOWN_RECONCILING`. The Control Plane binds the lookup to the exact saved account, proposal, attempt, and persisted `clientOid`. It uses the ordinary Bitget Live read routes only and preserves the unknown attempt and active reservation for every inconclusive result.

## Local verification

- Rust Control Plane fixture: a synthetic Bitget Live account and temporary SQLite/outbox persist an exact `orderInfo?clientOid=...` candidate, account mismatch, wrong side, empty response, malformed response, authentication failure, and transport failure. Every negative result leaves the attempt `UNKNOWN_RECONCILING`, keeps its reservation `ACTIVE`, and records no broker order identity. Replaying the outbox returns all seven evidence events; reopening SQLite restores all seven evidence rows.
- Browser verification: the Rust-backed integration bridge and isolated Vite page show the saved Bitget Live account and remote `userId`, exact client order ID, provider order ID/status, query scope, and candidate-only wording. Account mismatch, wrong side, empty result, authentication failure, and transport failure appear as inconclusive on the same panel through its scheduled refresh. The bridge reports zero provider order writes.
- Responsive verification: the evidence panel remains within 1280, 768, and 390 px viewports; the document and panel have no horizontal overflow. The browser reports no application console errors.
- Request boundary: the fixture verifies signed requests use `GET` on the ordinary Bitget Live endpoint, omit `paptrading`, and never use Demo/Testnet routes. No real credentials or provider requests were used.
- Prototype boundary: `docs/prototype/` was not changed; this runtime fixture does not certify the clickable prototype or provider-hosted account behavior.

The narrowed Manual Resolution and final S25 verification are recorded in the closed S25 parent [#96](https://github.com/kaiqiangh/tradex/issues/96#issuecomment-5863761675). This evidence remains scoped to Bitget Spot reconciliation.
