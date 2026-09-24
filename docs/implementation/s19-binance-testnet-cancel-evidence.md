# S19 #71 Binance Spot Testnet exact-order cancellation evidence

Date: 2026-09-24. Branch: `dev`. Issue: [#71](https://github.com/kaiqiangh/tradex/issues/71).

## Scope and result

The exact-order cancellation slice is locally fixture-verified. In the main Trade UI, review first refreshes the selected Binance Spot Testnet order by exact symbol and provider order ID; it opens the confirmation only when the returned book is current and the exact order remains cancelable with a known, valid remaining quantity for the selected connection and remote account. An exact refresh of an unchanged order advances only its observation time, so review stays available without resetting any recorded cancellation intent. Explicit confirmation creates one durable cancellation intent and at most one signed Testnet DELETE. Reopen, duplicate, definite rejection, changed/terminal provider state, ambiguous transport, and concurrent private-stream fills cannot cause a blind replay or erase fills. The dialog supports safe dismissal and labels submission, pending, and terminal outcomes separately.

The test seam uses synthetic HTTP/WebSocket responses, temporary SQLite, and the integration-only Rust browser bridge. No real Binance credentials, external provider request, or order were used. This does not close the provider-hosted Testnet gate: parent Spec #67 and issue #72 remain open. The clickable prototype is unchanged.

## Verification

- `cargo test --test binance -- --test-threads=1` — 18 passed, 0 failed, 1 ignored (native Keychain).
- `testnet_cancel_is_single_exact_delete_and_reconciles_terminal_status` — one exact `DELETE /api/v3/order`; terminal state comes from follow-up detail.
- `testnet_cancel_definitive_rejection_is_retained_and_never_replayed` and `testnet_cancel_reopen_recovers_unknown_intent_without_delete_or_replay` — the idempotency key survives rejection/reopen, with no second DELETE.
- `testnet_cancel_rechecks_exact_order_and_never_resends_unknown_outcome` and `testnet_cancel_completion_merges_concurrent_partial_and_full_private_stream_fills` — changed/unknown states do not replay; both racing trade IDs and the final fill remain durable.
- Rust-backed browser fixture, final tree (2026-09-24) — the workspace fixture completes local onboarding and model readiness, then the full Trade UI flow passes: exact Detail refresh occurs before the dialog and honors the bounded read cooldown; stale observations block review; a provider-definite rejection renders `PROVIDER_CANCEL_REJECTED` without replay; an ambiguous outcome stays pending provider confirmation; a provider-terminal order stays visible with no cancellation action; an order without a usable remaining quantity offers no review; Escape sends no command; explicit confirmation records `CANCELED` while preserving an earlier partial fill; responsive checks passed at 390, 768, and 1280 px with zero console errors (`workspace-ui`, then `provider-ui` with `binance/TESTNET`).
- `npm run check` — passed: generated IPC schema check, TypeScript and production build, 9/9 frontend unit tests, Rust workspace tests, and requirement traceability (203 requirements, 70 screens, 13 QA scenarios, 23 baseline files). The pre-existing bundle-size warning remains; native Keychain/Gateway integration tests are explicitly ignored.
- `cargo fmt --all`, `node --check tests/provider-ui.mjs`, and `git diff --check` passed.
- `cargo check --features desktop --bin tradex` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — blocked by six pre-existing Clippy findings already in baseline `2ceaa61`: five library warnings in the signed-request/history/order merge/private-stream helpers and one private-stream test helper. The one new needless-borrow lint in this cancellation diff was corrected; no unrelated #70 cleanup was folded into this ticket.
