# S27.2 #109 Live resume recovery evidence

Review baseline: `3ca4d54` (`S27.1 #108`). Implementation commit: `f9e81ca`. Verification date: 2026-09-29. Standards and Spec reviews both PASS with no actionable findings.

## Delivered behavior

- macOS system wake and TradeX becoming active again both enter the existing Live safety-recovery path. This replaces reliance on Tauri `RunEvent::Resumed`, which Tao documents as unsupported on macOS.
- Resume disarms every Live account, resets TimeService confidence, and persists connected Live accounts as `STALE / UNVERIFIED / UNCHECKED / STALE / BLOCKED / DISARMED` before provider recovery begins.
- Existing stream supervisors restart, then the existing startup recovery plan revalidates time, reads exact evidence for connected accounts' unknown PLACE attempts, and refreshes every connected supported Trading 212, Binance Spot, and Bitget Spot Live account. Account selection does not narrow recovery scope.
- Consistent with Backend ARD §41.34, unknown CANCEL attempts remain stale/blocked until the existing user-triggered exact-order refresh resolves them; resume does not infer cancellation or add a second recovery route.
- A Live account stays disarmed and cannot be armed while its post-resume account state is stale. Recovery never resubmits PLACE/CANCEL; account projection changes invalidate prior account-bound approvals and pre-dispatch preparation through the existing storage path.

## Verification

- PASS: `sleep_and_resume_safety_triggers_persist_disarm_reasons` covers two connected Live accounts, durable freshness invalidation, trusted-time reset/revalidation, recovery-plan coverage, blocked Arm before refresh, and explicit Arm only after recovery completes.
- PASS: `live_provider_auth_failure_disarms_only_the_affected_account` verifies an authentication failure persists on the affected Live account while an unrelated healthy Live account stays armed and current.
- PASS: `cargo check --manifest-path src-tauri/Cargo.toml` compiled the macOS desktop target.
- PASS: `cargo fmt --manifest-path src-tauri/Cargo.toml --check` and `git diff --check`.
- PASS: `RUST_TEST_THREADS=1 npm run check` completed schema validation, frontend typecheck/build, all 15 frontend unit tests, all 227 Rust core unit tests and workspace integration tests, and requirement traceability (203 requirements, 70 screens, 13 QA scenarios).
- NOTE: An earlier default-parallel `npm run check` had two workspace-reopen test failures. Both exact tests passed in isolation, and the complete check passed with Rust tests serialized.
- PASS: On the running native development app, returning to TradeX triggered `NSApplicationDidBecomeActiveNotification` and persisted the connected Trading 212 Live account to `SESSION_RESUMED / STALE / BLOCKED / DISARMED`; its prior successful observation remained unchanged.
- PASS: Standards and Spec review of `3ca4d54..f9e81ca`; no actionable findings.
- BLOCKED on 2026-09-29 (cleared in the follow-up below): The native read-only provider refresh stopped at macOS SecurityAgent, which requested the `login` Keychain password for `com.tradex.broker.credentials`. No password or replacement credential was entered by the agent; no order write was attempted.
- Not exercised: actual OS sleep/wake. The wake notification path compiled on Darwin; native app reactivation was exercised directly.

The normal production recovery path uses the existing fixed read-only provider adapters and requires no new provider route or financial authority.

## Follow-up, 2026-09-30

The Keychain/provider-completion block is cleared in the ordinary native development run at `dev@b0750634d5e770fc5c955b061908ed4da66578aa`. Read-only outbox inspection recorded T212 `SESSION_RESUMED` at `08:03:12.085077Z`, a fresh successful provider observation at `08:03:23.244385Z`, and `CURRENT` reconciliation at `08:03:23.25236Z`. Live remained `DISARMED / BLOCKED` with explicit Arm required; the projection retained 3 positions, 0 open orders and 6 recent orders. No broker order write or agent credential/ACL mutation occurred.

Actual OS sleep/wake remains unverified. After the user's completion report, the approximately `08:15 UTC` and rechecked at `08:19 UTC` power-log check found no Sleep/Wake/DarkWake event that day; a pre-existing `caffeinate` process still held sleep assertions, and the account remained at sequence 83. Several resume triggers occurred and provider-start timing was not measured, so this proves provider completion after resume, not physical wake or a new `<5 s` result. See the paired [parent acceptance audit](s27-parent-acceptance-audit.md) for the remaining boundaries.
