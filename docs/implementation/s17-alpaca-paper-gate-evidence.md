# S17 Alpaca Paper external gate evidence

Implementation: `4b8a07bc5060fa6a50bbebdf136f626d12945373` on `dev`.

## Result

Status: `BLOCKED_EXTERNAL` / `IMPLEMENTED_UNVERIFIED`. The user-confirmed Paper test on 2026-09-23 reached a persisted `REJECTED` attempt. The test used the existing Alpaca Paper connection and an `AAPL BUY 1 BASE LIMIT DAY` proposal. The attempt has no provider order identity, and a subsequent read-only order/fill refresh showed no new order or fill. No cancel was applicable, and no Live endpoint was used.

The running desktop build stored the rejection as `PROVIDER_PERMISSION_BLOCKED`. Its HTTP response body was not retained, consistent with the secret/raw-payload boundary, so the exact reason for this historical rejection cannot be reconstructed. The account observation at test time did not support the requested buy. The existing rejected attempt remains historical evidence; it was not resubmitted.

Alpaca documents that an order can be rejected when an account is unauthorized or its tradable balance is insufficient, and describes HTTP 403 as insufficient buying power or shares. Alpaca also documents account-protection rejections that use HTTP 403. Therefore the prior blanket mapping of order HTTP 403 to a permission error was not reliable. Commit `4b8a07b` classifies only recognized bounded response-message patterns into permission, buying-power, or available-share errors; unknown 403s remain generic provider rejections. The raw message is not persisted or displayed. Tests cover those paths, but the new classifier has not yet been exercised against a real Alpaca response.

## Verification

- `cargo test --workspace -- --test-threads=1` — passed: 127 library tests and 69 integration tests; 6 opt-in tests ignored by default.
- `cargo test --test providers -- --test-threads=1` — passed: 24 passed, 1 opt-in test ignored.
- `cargo test --test providers native_keychain_roundtrip_drives_the_real_connection_lifecycle -- --ignored --test-threads=1` — passed with disposable synthetic Keychain credentials.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `npm run schema:check`, `npm run typecheck`, `npm run test:unit` — passed; unit tests 7/7.
- `npm run build` — passed; Vite emitted the existing 500 KB chunk-size warning.
- The repository's `checkProviderUI` assertions from `tests/provider-ui.mjs` ran on 2026-09-23 at dev HEAD `8506400378b28193b202cfbb0f19e6fa80c54f19` against `npm run dev:browser`, using a Playwright adapter with an explicit workspace-ready wait. All 10 observation groups passed. The isolated Rust/SQLite bridge used provider and credential-entry fixtures; no real Alpaca request or Paper order was made. The browser runner excluded the development server's missing `/favicon.ico` from the console-error assertion; no application JavaScript errors were observed.
- `node --check tests/provider-ui.mjs` — passed.
- `python3 scripts/check_requirements.py` — passed inventory/traceability check; this does not prove runtime behavior.
- `git diff --check` — passed.
- Manual Standards and Spec review of `4173fef..4b8a07b` — PASS on both axes for this error-classification change.

## Remaining gate

S17 parent [#56](https://github.com/kaiqiangh/tradex/issues/56) remains open. To complete its external lifecycle, the Paper account needs enough available buying power for a newly reviewed order, and the user must explicitly authorize that specific new order because the previous proposal is consumed. Then run `submit → query by client order ID → observe provider state/fill/update → cancel if still open → refresh and verify`. Until that succeeds, FR-015 and AC-009 remain `IMPLEMENTED_UNVERIFIED`; broader FR-033 and UX-004 are not complete.

References: [Alpaca Create an Order](https://docs.alpaca.markets/us/v1.1/reference/postorder), [Alpaca User Protection](https://docs.alpaca.markets/us/v1.1/docs/user-protection).
