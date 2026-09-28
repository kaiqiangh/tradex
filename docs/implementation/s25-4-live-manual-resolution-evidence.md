# S25.4 Live Manual Resolution Evidence

Assessment date: 2026-09-28. Status: **Narrowed acceptance complete; Standards and Spec reviews passed serially; issues #100 and #101 closed. Parent #96 is open for final S25 acceptance.**

## Implemented boundary

- Binance Spot Live and Bitget Spot Live can expose `CONFIRMED_SUBMITTED` only when the latest evidence is fresh (30 seconds), complete, bound to the current account observation, and contains exactly one result from the attempt's persisted client-order-ID query. The result must match the saved attempt, account, immutable proposal, instrument, side, order type, supported quantity, and trusted submission window.
- The backend repeats these checks in an immediate SQLite transaction, links the observed provider order ID and status to the attempt, and appends the manual decision with both outbox events. The PLACE reservation remains active; no fill is inferred. The UI refreshes account health and the account remains `DISARMED`.
- Trading 212 similar-order candidates, empty results, and incomplete, delayed, stale, mismatched, or unsupported evidence expose only `KEEP_RECONCILING`. Keeping or dismissing preserves `UNKNOWN_RECONCILING` and the active reservation.
- `CONFIRMED_NOT_SUBMITTED` and reservation release are deliberately unavailable: no supported provider path supplies the required sufficient-absence proof. Empty queries remain inconclusive. The narrowed #100 scope covers only evidence-backed `CONFIRMED_SUBMITTED` and `KEEP_RECONCILING`; research #101 closed after finding no provider-published bounded absence rule. Any future provider-specific absence behavior requires separate evidence and implementation scope.

## Verification

- `npm run check` passed on the reviewed code snapshot committed as `dev@f67463d`: schema generation consistency, TypeScript production build, 13 frontend unit tests, 207 Rust unit tests, workspace integration suites, and requirement traceability (203 requirements, 70 screens, 13 QA scenarios, 23 baseline files). Repository-designated ignored tests remain ignored. The build reports the existing large-chunk advisory and fixture dead-code warnings.
- `cargo build --manifest-path src-tauri/Cargo.toml --features integration-test,order-gateway-runtime --bin tradex-ipc --bin tradex-order-gateway` passed.
- Rust-backed browser UI checks passed for Trading 212 Keep-only after timeout, Binance exact submitted-order confirmation, and Bitget exact submitted-order confirmation. They used temporary SQLite workspaces and synthetic provider fixtures, with no external provider requests or real accounts. Binance and Bitget fixture counters remained at zero order writes. The Trading 212 flow used a local fake gateway; manual Keep added no write and the existing synthetic POST count did not increase. Binance and Bitget retained active reservations and left accounts `DISARMED`; the evidence panel passed 1280/768/390 px checks.
- The Binance public IPC regression also submits `CONFIRMED_SUBMITTED` with stale attempt and evidence state versions before the valid decision; both are rejected without changing the unknown attempt or active reservation.
- Serial Standards and Spec reviews passed for `f7531640a69bd11fe8473d3f322ea4e348f69e7b..f67463d857850d69ba68050b2110612e3022394b`; the prior AC7 finding was resolved with the stale-version commit-path regression.
- No prototype code or fixture package was changed. These runtime checks do not upgrade prototype coverage or QA evidence.

## S25.4 acceptance

The narrowed #100 acceptance is satisfied: exact fresh Binance/Bitget order evidence authorizes only the submitted-order link; Trading 212 and all unsupported, incomplete, stale, delayed, or mismatched evidence authorize only Keep; stale state/evidence versions are rejected; reservations remain active; and successful submitted confirmation leaves the account DISARMED. There is no confirmed-absence transition or reservation release in this slice.

## Deferred from the parent S25 scope

Research issue #101 found no provider-published bounded sufficient-absence guarantee for the current Live interfaces. S25 therefore does not expose `CONFIRMED_NOT_SUBMITTED` or release capacity based on query absence. If a future provider contract adds such a rule, a separate implementation must still provide atomic exactly-once audit/reservation behavior and stale-version/fill-race coverage.
