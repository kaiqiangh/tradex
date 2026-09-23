# S18 #63 Trading 212 Demo cancellation evidence

Date: 2026-09-23. Branch: `dev`. Implementation commit: `c1f6884f700d180534a05001ed12d3e6bfda5b81`.

## Scope and result

Issue [#63](https://github.com/kaiqiangh/tradex/issues/63) is implemented and locally verified. A connected Trading 212 Demo order must be re-read by exact ID and account before a separate confirmation. Rust persists the cancellation intent and event before one Demo-only DELETE. Accepted and ambiguous results remain pending; duplicate requests cannot resend; later provider terminal observations clear the local pending state, while partial-fill observations preserve the fill. The dialog identifies the exact account/order, supports safe dismissal, traps keyboard focus, and was checked at 768 px and 390 px.

The provider HTTP fixture used synthetic account/order identifiers. No Trading 212 API credential, external broker request, or real order was used. The real Demo API gate remains issue #64. S17 #56 remains open as requested.

## Verification

- `npm run check` — passed on the implementation commit: schema generation check; production build and TypeScript; 9/9 frontend unit tests; 132/132 Rust unit tests; all runnable workspace integration suites; and requirements traceability (201 requirements, 70 screens, 12 QA scenarios, 23 baseline files). The existing Vite bundle-size warning remains. Five explicit native Keychain tests and one isolated Gateway test are marked ignored by the suite.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo fmt --all -- --check`, `node --check tests/provider-ui.mjs`, and `git diff --check` — passed.
- `cargo test --test providers trading212_demo_cancel -- --nocapture` — 3/3 passed: expired/stale/unknown review rejection; account identity change and timeout with no repeat DELETE; persisted cancellation and fill race.
- Rust-backed browser fixture — exact-order review and explicit confirmation; dismissal produced no request; focus started on the safe action, remained trapped, and restored on close; the application shell became inert; no overflow at 768 px or 390 px. A synthetic provider acknowledgement stayed pending while the partial fill remained visible and the UI exposed only exact-order refresh.

The first full check encountered one workspace-reopen assertion failure. The exact test and full workspace integration binary passed on immediate reruns, and subsequent complete `npm run check` runs passed, including the final implementation commit. The final one-line UI correction suppresses a rejection message when the cancellation outcome is unknown; the accepted browser path was verified, and the final build/typecheck passed.

The provider contract is documented by [Trading 212's cancel-order API](https://docs.trading212.com/api/orders/orders): a successful response accepts the request, while cancellation can race with execution. This evidence does not claim a live Demo API test.
