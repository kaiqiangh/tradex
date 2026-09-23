# S17 Alpaca Paper order, fill, and cancellation evidence

Implementation SHA: `b7c33b9a94d107f763b51b060e738ef7939fa80a` on `dev` (review baseline `1618018d171c3be87fadc93bfa18a022b0b4f99f`). This records the acceptance evidence for [#58 查询 Alpaca Paper 订单成交并安全撤单](https://github.com/kaiqiangh/tradex/issues/58) only.

## Result

The #58 implementation and local acceptance checks pass. The serial Standards and Spec reviews found no blocking findings for this ticket. Versioned IPC/schema, fixed-Paper REST pagination, durable account-isolated order/fill observations and outbox events, exact-order cancellation review, pending-versus-provider-truth semantics, React open/history/fill projection, bilingual ARD updates, and focused security/fault regressions are included in the implementation commit.

The Rust-backed browser verification used an isolated integration bridge, temporary SQLite workspace, fixture credential vault, and controlled HTTPS provider responses. The fixture flow covered account-scoped open/history orders and fills, explicit refresh, exact account/order/environment and filled/remaining review before cancellation, dismissing confirmation without sending, HTTP 204 remaining `CANCEL_PENDING`, reload persistence, and responsive 390/768 layouts. After the final UI refresh adjustment, explicit full refresh showed `CURRENT`; a targeted single-order review correctly left the complete book `STALE` until refreshed. No real Alpaca credential, external provider response, or Paper order was used.

The repository's `checkProviderUI` assertions were rerun on 2026-09-23 at dev HEAD `8506400378b28193b202cfbb0f19e6fa80c54f19` against `npm run dev:browser`, using a Playwright adapter with an explicit workspace-ready wait. All 10 observation groups passed, including Proposal confirmation and acknowledgement separation, order refresh and cancellation review, stream fill deduplication/degraded recovery, reload, and 390/768 responsive checks. The isolated Rust/SQLite bridge used provider and credential-entry fixtures; no real Alpaca request, user credential, or Paper order was used. The browser runner excluded the development server's missing `/favicon.ico` from its console-error assertion; no application JavaScript errors were observed.

## Checks

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `cargo test --workspace -- --test-threads=1` — passed: 120 library tests and all runnable integration suites; six opt-in native Keychain/pinned-gateway tests remain ignored by declaration. Provider suite: 22 passed, 1 ignored.
- `npm run schema:check` — passed; Rust, JSON Schema, and TypeScript agree.
- `npm run typecheck` — passed.
- `npm run test:unit` — passed, 7/7.
- `npm run build` — passed; Vite reports the existing minified chunk-size warning (~2.7 MB, over its 500 KB threshold).
- `node --check tests/provider-ui.mjs` — passed.
- `python3 scripts/check_requirements.py` — passed (201 requirements, 70 screens, 12 QA scenarios, 23 baseline files; inventory check only).
- `git diff --check 1618018d171c3be87fadc93bfa18a022b0b4f99f..b7c33b9a94d107f763b51b060e738ef7939fa80a` — passed.

## Remaining boundary

S17 parent Spec #56 remains open. The #59 private stream, reconnect/restart reconciliation, and IPC/SQLite/UI evidence are now recorded in [S17 #59 stream evidence](s17-alpaca-paper-stream-evidence.md). The real external Alpaca Paper sandbox lifecycle was not attempted, so it remains `BLOCKED_EXTERNAL`; no live or Paper order was sent. This evidence must not be used to mark the broader PRD requirements or S17 as verified.
