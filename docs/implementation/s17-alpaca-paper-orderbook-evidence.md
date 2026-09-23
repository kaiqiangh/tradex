# S17 Alpaca Paper order, fill, and cancellation evidence

Implementation SHA: `b7c33b9a94d107f763b51b060e738ef7939fa80a` on `dev` (review baseline `1618018d171c3be87fadc93bfa18a022b0b4f99f`). This records the acceptance evidence for [#58 查询 Alpaca Paper 订单成交并安全撤单](https://github.com/kaiqiangh/tradex/issues/58) only.

## Result

The #58 implementation and local acceptance checks pass. The serial Standards and Spec reviews found no blocking findings for this ticket. Versioned IPC/schema, fixed-Paper REST pagination, durable account-isolated order/fill observations and outbox events, exact-order cancellation review, pending-versus-provider-truth semantics, React open/history/fill projection, bilingual ARD updates, and focused security/fault regressions are included in the implementation commit.

The Rust-backed browser verification used an isolated integration bridge, temporary SQLite workspace, fixture credential vault, and controlled HTTPS provider responses. The fixture flow covered account-scoped open/history orders and fills, explicit refresh, exact account/order/environment and filled/remaining review before cancellation, dismissing confirmation without sending, HTTP 204 remaining `CANCEL_PENDING`, reload persistence, and responsive 390/768 layouts. After the final UI refresh adjustment, explicit full refresh showed `CURRENT`; a targeted single-order review correctly left the complete book `STALE` until refreshed. No real Alpaca credential, external provider response, or Paper order was used.

`tests/provider-ui.mjs` passed syntax validation, but the full helper was not rerun after the final UI refresh adjustment. The final adjustment was manually checked in the Rust-backed browser. This ticket's executable Rust provider suite passed after the final security guard; the limitation is recorded rather than treating helper syntax as a full browser run.

## Checks

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `cargo test --workspace -- --test-threads=1` — passed: 120 library tests and all runnable integration suites; six opt-in native Keychain/pinned-gateway tests remain ignored by declaration. Provider suite: 22 passed, 1 ignored.
- `npm run schema:check` — passed; Rust, JSON Schema, and TypeScript agree.
- `npm run typecheck` — passed.
- `npm run test:unit` — passed, 7/7.
- `npm run build` — passed; Vite reports the existing minified chunk-size warning (~2.7 MB, over its 500 KB threshold).
- `node --check tests/provider-ui.mjs` — passed; see the browser-helper limitation above.
- `python3 scripts/check_requirements.py` — passed (201 requirements, 70 screens, 12 QA scenarios, 23 baseline files; inventory check only).
- `git diff --check 1618018d171c3be87fadc93bfa18a022b0b4f99f..b7c33b9a94d107f763b51b060e738ef7939fa80a` — passed.

## Remaining boundary

S17 parent Spec #56 remains open. #59 still needs the authenticated private `trade_updates` stream, reconnect/restart reconciliation, and its IPC/SQLite/UI evidence. The real external Alpaca Paper sandbox lifecycle was not attempted, so it remains `BLOCKED_EXTERNAL`; no live or Paper order was sent. This evidence must not be used to mark the broader PRD requirements or S17 as verified.
