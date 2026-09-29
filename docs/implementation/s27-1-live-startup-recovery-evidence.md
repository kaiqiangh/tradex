# S27.1 #108 Live startup recovery evidence

Status: implementation and full repository checks pass; standards review passes. Spec review keeps the native `<5 s` startup timing acceptance open, so #108 remains open. S27 parent #107 and the remaining recovery, queue, and model-failure tickets remain open.

## Delivered behavior

- Opening or reopening a workspace stales persisted account health, blocks execution eligibility, and keeps Live accounts disarmed. The native startup path begins recovery after the workspace-open response and does not depend on the selected UI account.
- Startup establishes a fresh local TimeService baseline even when no supported Live account is connected. Clock uncertainty keeps reconciliation stale and Live disarmed.
- The recovery plan includes every connected Trading 212, Binance Spot, and Bitget Spot Live account and each durable `UNKNOWN_RECONCILING` PLACE attempt scoped to those accounts.
- Exact S25 resolution-evidence reads for unknown PLACE attempts are dispatched before read-only `account.refresh` jobs; both read sets overlap, and every P0 result is committed before refreshed account projections. Unknown CANCEL attempts are not automatically queried or replayed and continue to block recovery until the existing user-triggered exact-order refresh resolves them.
- Trading 212 reads one bounded recent-orders page (at most 50 rows). Binance reads one page (at most 1,000 rows) for each of at most 10 symbols found in current open orders or the saved recent-orders projection; provider history for symbols never observed locally may be absent. Bitget reuses its existing order/fill projection. Trading 212/Binance `recentOrders` are displayed separately and never drive open-order, cancellation, or capacity decisions.
- A refreshed account becomes `CURRENT` only when fresh account data is present, the account remains connected and authenticated, local time is trusted, and no unknown PLACE or CANCEL attempt remains. Inconclusive PLACE attempts retain their active reservations; all unresolved attempts retain a persisted stale/blocked reason. Recovery never replays PLACE or CANCEL and never re-arms.
- Persisted account health and provider observations use the existing account projection/event path for restoration in the UI, including a separate recent-order table for Trading 212 and Binance Live.

## Verification

- PASS: `reopening_the_same_workspace_stales_and_disarms_every_live_account`.
- PASS: `startup_revalidates_local_clock_without_connected_live_accounts`.
- PASS: `startup_recovery_covers_all_live_accounts_and_preserves_unknown_scope`, including exact attempt selection, all three supported providers, retained active reservation, clock-drift refusal, and explicit disarmed/stale account state.
- PASS: `startup_recovery_stays_blocked_for_an_unknown_live_cancel_attempt`; startup does not queue exact CANCEL reads, and a fresh account response cannot clear the stale/disarmed gate.
- PASS: `cancellation_approval_tests::startup_probe_refresh_fetches_and_persists_t212_recent_live_history`, covering the startup `Probe` job and its bounded Trading 212 recent-history read.
- PASS: `t212_recent_history_is_bounded_and_rejects_duplicate_or_untrusted_pages` and `provider_io::binance::tests::live_recent_order_history_requires_the_requested_symbol_and_known_status`.
- PASS: `live_approval_tests::binance_live_account_refresh_restores_bounded_recent_order_history`, using a local signed-request fake to confirm the bounded GET and persisted recent-order projection.
- PASS: `cargo check --manifest-path src-tauri/Cargo.toml --bin tradex --features desktop,integration-test`.
- PASS: `npm run check` on the latest full run (schema check, typecheck/build, 15 frontend unit tests, Rust workspace tests, and requirement traceability).
- The first post-fix full-check run had two workspace-reopen assertions fail; each passed in isolation, and the subsequent full run passed.
- PASS: `git diff --check`.

The earlier native attempt used a temporary ad-hoc-signed wrapper (`local.tradex.desktop`) and blocked in macOS `SecItemCopyMatching` before provider HTTP. On 2026-09-29, a second run used the normal `npm run desktop` development app (`target/debug/tradex`) and reopened `/Users/kai/.tradex/workspaces/default` from the UI. macOS requested the login Keychain password for `com.tradex.broker.credentials`; no password was entered and no Allow/Always Allow action was taken. A process sample showed the startup `ProviderJob` waiting through `NativeVault::get` → `get_generic_password` → `SecItemCopyMatching`. Read-only inspection afterward found the Trading 212 Live projection still at sequence 14 with `lastSuccessfulSync` `2026-09-23T23:06:33.624426Z` and STALE/BLOCKED health. No provider socket or refreshed projection was observed. The socket check occurred well after workspace open, so it does not measure the `< 5 s` target; this native run remains unverified. No Keychain ACL or credential was changed, and no order mutation was attempted. The host has no valid code-signing identity or installed matching app bundle, so trusted-package Keychain behavior remains unverified.

The standards review passes with no actionable findings. The spec review found no additional mismatch but keeps native `< 5 s` startup timing open. Signed-package lifecycle and full S27/S33 acceptance remain unverified; #108 stays open pending a trusted native startup run.
