# S27.1 #108 Live startup recovery evidence

Status: implementation and isolated Rust checks are complete for this ticket. S27 parent #107 and the remaining recovery, queue, and model-failure tickets remain open.

## Delivered behavior

- Opening or reopening a workspace stales persisted account health, blocks execution eligibility, and keeps Live accounts disarmed. The native startup path begins recovery after the workspace-open response and does not depend on the selected UI account.
- Startup establishes a fresh local TimeService baseline even when no supported Live account is connected. Clock uncertainty keeps reconciliation stale and Live disarmed.
- The recovery plan includes every connected Trading 212, Binance Spot, and Bitget Spot Live account and each durable `UNKNOWN_RECONCILING` PLACE attempt scoped to those accounts.
- Exact S25 resolution-evidence reads run before the existing read-only `account.refresh` jobs. Existing provider adapters retain their account-identity checks and supported bounded open-order, recent-history, and fill reads.
- A refreshed account becomes `CURRENT` only when fresh account data is present, the account remains connected and authenticated, local time is trusted, and no unknown PLACE attempt remains. Inconclusive attempts retain their active reservation and a persisted stale/blocked reason. Recovery never replays PLACE or CANCEL and never re-arms.
- Persisted account health and provider observations use the existing account projection/event path for restoration in the UI.

## Verification

- PASS: `reopening_the_same_workspace_stales_and_disarms_every_live_account`.
- PASS: `startup_revalidates_local_clock_without_connected_live_accounts`.
- PASS: `startup_recovery_covers_all_live_accounts_and_preserves_unknown_scope`, including exact attempt selection, all three supported providers, retained active reservation, clock-drift refusal, and explicit disarmed/stale account state.
- PASS: `cargo check --manifest-path src-tauri/Cargo.toml --bin tradex --features desktop,integration-test`.
- PASS: `git diff --check`.

The automated provider and persistence tests use synthetic workspaces. No real provider request or order mutation was made. Native desktop startup timing, the measured `< 5 s` target, signed-package lifecycle, and full S27/S33 acceptance remain unverified; this ticket does not close those broader requirements.
