# S27.1 #108 Live startup recovery evidence

Status: S27.1 implementation, the Trading 212 history-envelope correction, full repository checks, code review, and native development-app workspace-reopen acceptance pass. The saved Trading 212 Live account completed fresh read-only recovery about 0.682 seconds after workspace reopen; #108 is ready to close. Signed-package lifecycle and S33 remain outside this ticket and unverified. S27 parent #107 and the remaining recovery, queue, and model-failure tickets remain open.

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

The earlier native attempt used a temporary ad-hoc-signed wrapper (`local.tradex.desktop`) and blocked in macOS `SecItemCopyMatching` before provider HTTP. On 2026-09-29, the normal `npm run desktop` development app (`target/debug/tradex`) initially showed the same Keychain wait while reopening `/Users/kai/.tradex/workspaces/default`; at that sample, the Trading 212 Live projection remained at sequence 14 with `lastSuccessfulSync` `2026-09-23T23:06:33.624426Z` and STALE/BLOCKED health.

A later read-only check of that run found sequence 16, `FAILED / ERROR / STALE / DISARMED`, with the generic `PROVIDER_RESPONSE_INVALID` message and the same `lastSuccessfulSync`. The process started at 13:10:55 and the workspace WAL was last modified at 13:40:21. There is no per-request timestamp or endpoint detail, so this delayed persisted failure does not establish the `< 5 s` provider-read target. It does confirm that stale observations were preserved and Live stayed disarmed after a validation failure.

A controlled restart on 2026-09-29 started `target/debug/tradex` at 14:11:58. By 14:12:25, the Trading 212 Live row was sequence 17, `FAILED / UNCHECKED / STALE / DISARMED`, with its previous `lastSuccessfulSync`; the WAL mtime was 14:12:08. The only observed outbound port-443 socket resolved by reverse DNS to Google Cloud, and the sampled network stack showed an Alpaca stream, so this is not evidence of a Trading 212 request. The app was on its New Thread screen rather than a confirmed workspace-open view, and no Trading 212 request start was measured. The `< 5 s` native acceptance therefore remains open.

The Accounts UI also showed that selecting the saved `trading212 · LIVE` connection switches to “Use the stored local credential” and exposes “Use existing account” without asking for a new API key. The manual refresh was not activated during that UI check. No Keychain ACL or credential was changed, and no order mutation was attempted. The host has no valid code-signing identity or installed matching app bundle, so trusted-package Keychain behavior remains unverified.

A later same-workspace reopen at `2026-09-29T13:38:48Z` found the Trading 212 row at sequence 18 and `FAILED`; because startup recovery deliberately includes only `CONNECTED` accounts, that row was excluded and this reopen did not exercise a Trading 212 provider read. A subsequent read-only refresh of the saved row reached `NativeVault::get` and blocked in `SecItemCopyMatching` (`provider_io.rs:1597`). macOS SecurityAgent requested the `login` keychain password for `com.tradex.broker.credentials`. The user had authorized “Always Allow” and that button was selected, but the system dialog still requires the keychain password. No password or new API key was entered; the account sequence stayed 18 and no Trading 212 socket or provider HTTP request was observed. Native `<5 s` timing remains unverified.

The standards review passes with no actionable findings. The spec review found no additional mismatch but keeps native `< 5 s` startup timing open. Signed-package lifecycle and full S27/S33 acceptance remain unverified; #108 stays open pending a trusted native startup run.

## Follow-up: Trading 212 historical-order response shape

The official [Trading 212 OpenAPI bundle](https://docs.trading212.com/_bundle/api.json) defines each historical-order item as `{fill, order}`. The earlier `PROVIDER_RESPONSE_INVALID` history-stage failure came from passing that outer item to order parsers. Startup recent-order hydration, Live exact-order fallback and resolution, and Demo history pagination now read the nested `order`; Live projections collapse identical repeated snapshots and reject conflicting snapshots, while Demo history keeps its specified duplicate-identity rejection. Updated fixtures and `t212_recent_history_parses_official_order_fill_rows_and_rejects_conflicts` cover the contract. The focused regression and current `npm run check` passed. The native reopen recorded below then confirmed the corrected parser against the saved Trading 212 Live account and completed its read-only account/history recovery inside the five-second target.

## Native development-app workspace reopen

- PASS: Restarted the ordinary `npm run desktop` development app and reopened `/Users/kai/.tradex/workspaces/default` on 2026-09-29. Workspace `last_opened_at` was `2026-09-29T17:35:17.113204Z`; the T212 account first persisted `STALE / BLOCKED / DISARMED` at `17:35:17.110664Z`, retaining its prior successful observation.
- PASS: The provider account/history refresh completed at `17:35:17.790510Z`; the account reached `CURRENT` at `17:35:17.795093Z`, about `0.682 s` after workspace reopen. The persisted projection contained 3 positions, 0 open orders, and 6 recent orders. Execution eligibility remained `BLOCKED` and arming remained `DISARMED`.
- PASS: The reopened native Accounts view restored the T212 Live connection and its recovery reason/status, account and position projection, empty open-order view, and six-row recent-order table. The screen showed `ONLINE`, `VALID`, `CURRENT`, `DISARMED`, and `BLOCKED`; no Arm or order action was invoked.
- The native workspace had one connected supported Live account, Trading 212. Binance/Bitget and unresolved-attempt branches remain covered by the named Rust tests above; signed-package Keychain lifecycle and S33 are separate gates.
