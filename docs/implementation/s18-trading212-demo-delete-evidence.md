# S18 #66 Trading 212 Demo local account deletion evidence

**Status:** Verified for the local application slice on `dev` at code commit `d093a124c59dcc1134122d70848b80e4ea586e14` (2026-09-24). Spec: [#65](https://github.com/kaiqiangh/tradex/issues/65). Implementation: [#66](https://github.com/kaiqiangh/tradex/issues/66).

## Scope

This evidence covers FR-081 / AC-067: deleting one eligible, credential-free Trading 212 Demo connection and its local account/order-book observations. The trusted command rechecks workspace, identity, state version, credentials and financial activity inside an immediate SQLite transaction. It refuses Live, other providers/environments, connected or credential-bearing accounts, open/pending orders, actionable proposals and unresolved attempts. Acknowledged attempts require their exact linked order to have a persisted recognized terminal status. Terminal proposal/attempt audit history and unrelated connections remain intact. The operation makes no provider request or Keychain call.

## Verification

- `npm run check` passed on the exact code commit: generated Rust/JSON Schema/TypeScript contracts agree; production typecheck and Vite build passed; 9 frontend tests, 132 Rust unit tests, and all workspace integration tests passed. The provider suite reported 40 passed, 0 failed, 1 explicit OS Keychain test ignored.
- `cargo check --bin tradex-ipc --features integration-test`, `cargo fmt --all -- --check`, `node --check tests/provider-ui.mjs`, `git diff --check`, and `python3 scripts/check_requirements.py` passed.
- Rust provider tests cover eligible deletion, transactional rollback after an injected storage failure, ineligible provider/environment/state/version/credential cases, open orders, actionable proposals, unresolved and reconciled attempts, no provider HTTP call, unrelated account preservation, and retained terminal audit.
- Rust-backed CUA browser verification used a disposable workspace and the Accounts UI. Cancel and Escape preserved the seeded record; explicit confirmation removed only that synthetic Demo record and its account/order-book snapshots. The dialog identified the exact record, initially focused Cancel, and fit 768 px and 390 px widths. A follow-up check confirmed success focus on the Account connections heading and that Local Paper remained in the account list.
- The disposable record was created by an `integration-test`-only fixture seed command. No user account was deleted.

## Remaining boundaries

The clickable prototype still fails QA-13; this runtime result does not change its status. S33 full application regression, the real Trading 212 Demo gate in #64, real Live connection/provider verification, and deletion of the user's three existing Demo records are not covered here.
