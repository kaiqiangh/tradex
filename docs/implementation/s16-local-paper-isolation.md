# S16 Local Paper isolation and end-to-end closeout

- Scope: issue #55, the final S16 acceptance slice after #53/#54.
- Authority: PRD Local Paper invariant, UI Spec G1/E3 and §9–10, Backend ARD §§30–32/41–42, Frontend ARD §§7.4/21–23.
- Boundary: Local Paper remains a TradeX-owned simulation. This slice does not add provider lifecycle, credentials, Live arming, approval, reservation, broker acknowledgement, reconciliation, or gateway authority.

## Acceptance matrix

| Requirement | Evidence to bind | Implementation decision |
| --- | --- | --- |
| Reopen preserves account/profile/orders/fills/cash/positions/open orders/P&L/events/portfolio | Rust dispatcher test plus Rust-backed browser flow on a disposable SQLite workspace | Rehydrate canonical order/fill/event tables, validate the rebuilt aggregate before every read or mutation, and read portfolio from the Local Paper projection. |
| Cross-workspace, foreign/invalidated proposal, unknown fields, stale versions, duplicate submit/cancel | Rust negative tests and generated schema check | Keep workspace identity and proposal status checks at the control-plane/storage boundary; all failed mutations occur before commit. |
| Tampered fill/cash/position/P&L | Rust persistence corruption tests | Recompute the deterministic Local Paper ledger from canonical fills and open-order reservations; reject any mismatch with `WORKSPACE_INTEGRITY_FAILED`. |
| No provider/Keychain/network/model/gateway or Live authority path | `prepare_provider` boundary assertion, gateway snapshot equality, account/provenance assertions, and explicit source audit | `paper.*` stays in the storage/paper modules; no adapter, credential, network, model, approval, arming, reservation, broker ACK, reconciliation, or readiness mutation is reachable. |
| Provenance and UI states | Rust-backed browser helper at 390/768/1280 with keyboard, modal focus, retry/error/empty/loading assertions | Preserve `local-paper`, `LOCAL`, `TRADEX_SIMULATION`, explicit disclosure, dialog semantics, focus restore, and fail-closed error surfaces. |
| Contract and traceability | English/Chinese Backend and Frontend ARD, requirements, surfaces, coverage, and this evidence file | Add the exact `paper.*` command/result/event contract and mark only S16 rows verified; keep S17–S30/S33 pending. |

## Ordered workflow

1. **to-spec** — this matrix fixes the acceptance boundary and the smallest semantic integrity rule.
2. **to-tickets** — #55 is the claimed S16 acceptance ticket; no new provider or Live ticket is created.
3. **implement** — add ledger validation and the missing negative/evidence checks, then synchronize generated/traceability documentation.
4. **code-review** — run the two-axis review against the final `dev` SHA, rerun the required checks, comment the exact evidence on #55, close #55, and update #51/#1 pointers without closing either parent prematurely.

## Explicit non-goals

S17–S30 provider and Live lifecycles, S31 import/export, S32 diagnostics, and S33 full cross-application regression remain incomplete and must remain marked as such.
