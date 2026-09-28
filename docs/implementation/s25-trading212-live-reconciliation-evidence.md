# S25.1 #97 Trading 212 Live Unknown Submission Reconciliation Evidence

Date: 2026-09-27. Branch: `dev`. Review base: `ad9e17fdfac4efe7cd71717e7bf5957c44207cc1`.

## Scope

This slice covers only a saved Trading 212 Live `PLACE` attempt and its exact account and immutable intent. The Control Plane owns provider identity, the five-minute trusted-time window, the read-only query, and the durable evidence ledger/outbox. The public renderer contract carries workspace, attempt, account, and attempt-state-version identities; it accepts no provider URL or credential. Reconciliation never retries or resends the order, and candidate orders are never auto-bound or treated as proof of absence.

The later Binance/Bitget reconciliation slices and narrowed Manual Resolution are recorded in the closed S25 parent [#96](https://github.com/kaiqiangh/tradex/issues/96#issuecomment-5863761675). This evidence remains scoped to Trading 212 reconciliation; provider-hosted acceptance and the clickable prototype remain outside this local slice.

## Verification

- `npm run check` **PASS**: generated IPC schema consistency, TypeScript check and Vite build, 13 Node unit tests, `cargo test --workspace` (202 Rust library tests plus the default integration targets), and requirement traceability (203 requirements, 70 screens, 13 QA cases, 23 baseline files). The existing Vite large-chunk advisory remains informational.
- `cargo fmt --all -- --check`, `git diff --check`, and `node --check tests/live-approval-ui.mjs` **PASS**. The schema v6/v8 migration fixtures now remove the v29-only evidence table before replaying the older migration chain; the future-schema fixture now uses version 30.
- Rust-backed browser fixture **PASS** with a synthetic Trading 212 Live account and a local fake provider. A `PLACE` result entered `UNKNOWN_RECONCILING`; read-only reconciliation persisted `INCONCLUSIVE` evidence with no candidates and explicit “absence is not proven” meaning. Navigation away from and back to Order Drafts restored the attempt and evidence while the reservation stayed `ACTIVE`; the account remained `DISARMED` and execution-blocked. The local Gateway log showed no extra submission `POST` during refresh or navigation.
- The expired-window fixture confirmed the trusted five-minute boundary: the attempt stayed `UNKNOWN_RECONCILING`, its reservation stayed `ACTIVE`, and only `KEEP_RECONCILING` was allowed. A refresh after expiry was rejected before provider access. The durable Keep action accepted only that backend-authorized decision, rejected `CONFIRMED_NOT_SUBMITTED`, preserved the attempt/reservation, and restored its audit record after workspace reopen.
- A deadline-tick regression advanced trusted time past the saved attempt's cutoff and called the native expiry pass without querying the evidence surface. It independently changed the account to `STALE`/`DISARMED` while retaining the unknown attempt and active reservation.
- The evidence region was visible without horizontal overflow at 1280, 768, and 390 CSS pixels. Measured document widths were 1265, 753, and 375 pixels; the evidence panel was fully within each viewport and had no local overflow. At 390 pixels, keyboard Tab navigation reached adjacent controls, while the evidence itself appeared in a labelled region with a polite live status; no resend control was exposed for the consumed unknown attempt.
- All browser data and provider responses were synthetic. No real Trading 212 credentials, provider-hosted requests, or provider writes were used. The existing clickable prototype was not changed or promoted to PASS.
