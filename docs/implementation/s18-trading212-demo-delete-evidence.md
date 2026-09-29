# S18 #66 Trading 212 Demo local account deletion evidence

**Status:** Verified for the local application slice on `dev` at code commit `383e4c60dad4164f6fc7aab1d8869e2fe6d2b8a3` (2026-09-24). Spec: [#65](https://github.com/kaiqiangh/tradex/issues/65). Implementation: [#66](https://github.com/kaiqiangh/tradex/issues/66).

## Scope

This evidence covers FR-081 / AC-067: deleting one eligible, credential-free Trading 212 Demo connection and its local account/order-book observations. The trusted command rechecks workspace, identity, state version, credentials and financial activity inside an immediate SQLite transaction. It refuses Live, other providers/environments, connected or credential-bearing accounts, open/pending orders, actionable proposals and unresolved attempts. Acknowledged attempts require their exact linked order to have a persisted recognized terminal status. Terminal proposal/attempt audit history and unrelated connections remain intact. The operation makes no provider request or Keychain call.

## Verification

- `npm run check` passed on the exact code commit: generated Rust/JSON Schema/TypeScript contracts agree; production typecheck and Vite build passed; 9 frontend tests, 132 Rust unit tests, all workspace integration tests, and 203 requirement / 70 screen / 13 QA traceability checks passed. The provider suite reported 40 passed, 0 failed, 1 explicit OS Keychain test ignored.
- `cargo check --bin tradex-ipc --features integration-test`, `cargo fmt --all -- --check`, `node --check tests/provider-ui.mjs`, `git diff --check`, and `python3 scripts/check_requirements.py` passed.
- Rust provider tests cover eligible deletion, transactional rollback after an injected storage failure, ineligible provider/environment/state/version/credential cases, open orders, actionable proposals, unresolved and reconciled attempts, no provider HTTP call, unrelated account preservation, and retained terminal audit.
- Rust-backed CUA browser verification used a disposable workspace with three same-label, same-environment Demo fixtures. The confirmation named the selected full connection ID; Cancel and Escape preserved all three. Confirming one synthetic record removed only that account and its account/order-book snapshots; the other two same-label records and Local Paper remained. Focus returned to Account connections, and the dialog fit 768 px and 390 px widths.
- Built and opened the native debug app bundle from the exact code commit. In the existing workspace, the eligible FAILED Demo record exposed “Delete local account”; its confirmation showed the selected record's full connection ID, local-only effects, and initial Cancel focus. The real record was left at the final confirmation button and was not deleted.
- The disposable record was created by an `integration-test`-only fixture seed command. No user account was deleted.

## Remaining boundaries

The clickable prototype still fails QA-13; this runtime result does not change its status. S33 full application regression and the real Trading 212 Demo order-lifecycle gate in #64 remain open. This implementation evidence does not cover provider writes or actual removal of the user's three existing Demo records; each final deletion remains an operator-confirmed action.
