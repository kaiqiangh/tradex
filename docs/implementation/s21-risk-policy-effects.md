# S21 #82 Workspace policy-change effects

Status: the `d1d0518` baseline-to-working-tree implementation passes the serial Standards and Spec reviews, the full `npm run check` suite, and isolated Rust-backed browser verification. The required Clippy command still reports pre-existing warnings in unchanged Binance/Bitget provider paths and previously existing risk/account-stream functions; it reported no warning in the new S21 #82 logic.

## Delivered behavior

- `risk.save_policy` treats the persisted policy as workspace-shared and derives affected account IDs from all persisted workspace connections, not the selected UI account.
- The policy projection carries an optional `lastChange` summary with old/new versions, workspace scope, weakening classification/reason codes, affected account identities, affected pending proposals, and a safe invalidation reason.
- Policy relaxation is classified across scalar limits, allow/block lists, market-order permission, quote staleness, inactivity, and asset-class exposure. Any relaxation makes a mixed change weak; tightening-only changes do not.
- One SQLite `IMMEDIATE` transaction rechecks risk, account, and proposal versions; writes the policy and outbox; disarms any affected Live account that is armed; marks every pending proposal bound to the prior policy `POLICY_CHANGED`; and appends a current-policy RiskDecision containing `POLICY_VERSION_STALE`.
- The same policy-version predicate is part of ordinary RiskDecision evaluation for later S22 eligibility. No approval authority, provider mutation, broker cancellation, reservation, or order submission was added.
- Settings → Risk & Limits renders the persisted effect summary as a keyboard-readable `role="status"` polite live region with expandable account and proposal details.

## Evidence

- Rust unit test: `risk::tests::policy_weakening_classifier_detects_mixed_relaxation_and_pure_tightening`.
- Public-command tests: `risk_policy_weakening_invalidates_pending_proposals_and_rechecks_the_old_version`, `risk_policy_change_fans_out_to_workspace_accounts_and_proposals_only`, and `tightening_policy_change_invalidates_old_proposal_without_marking_weakening`.
- The public-command cases cover mixed/tightening classification, multiple persisted accounts, multiple proposals, isolated workspace state, stale state-version rejection without partial proposal mutation, durable stale decisions, proposal history, and reopen recovery.
- Account Live arming is currently invariantly `DISARMED`; the transaction includes the disarm write for any future persisted armed Live account state.
- Save-versus-approval-consume/reservation serialization remains S23. The prototype fixture package and its historical QA evidence are unchanged.

## Verification

- PASS: `npm run check` (schema parity, frontend build, 10 unit tests, Rust workspace and provider suites, requirements traceability).
- PASS: `cargo fmt --all -- --check`, `git diff --check`, manifest hashes/line counts, and generated `RiskPolicyChange` field-bound assertions.
- PASS: browser save of a tightening policy in an isolated Local Paper workspace showed v2 → v3, the affected account, the invalidated proposal and its stale-version reason. No provider request or order was sent.
- NOT PASS: `cargo clippy --workspace --all-targets --all-features -- -D warnings`; only pre-existing warnings remain outside the new policy-change logic.
