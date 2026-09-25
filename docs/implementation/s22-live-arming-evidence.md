# S22 #84 Account-scoped Live Arming evidence

Status: **IMPLEMENTED_UNVERIFIED**. Review baseline: `dev@7f1648171728c158e99102a48de54bb8d2090798`. Implementation: `dev@5627905` (33 files; no provider write path). This evidence document is committed separately from the implementation.

## Delivered scope

- Rust Control Plane owns account-scoped Arm, Disable Live, Disable All, durable reason codes, and eligibility. Arm requires trusted time, a configured risk policy, healthy/reconciled account state, Live provider support, and `VERIFIED` credential-permission scope. Acknowledging `UNVERIFIED` scope only completes connection review and never enables Live Arm.
- A native deadline monitor checks armed accounts once per second and persists inactivity disarm even when the renderer sends no further IPC. Startup, sleep/resume, session loss, account/credential health changes, and risk-policy weakening also disarm with durable reasons.
- Accounts UI binds consent to the provider, label, environment, complete TradeX connection ID, and observed provider account ID. Per-account and global controls remain keyboard-operable; global Disable All acts across all Live accounts.
- English and Chinese PRD, UI Spec, and Frontend/Backend ARDs now state the verified-permission requirement and inactivity monitor behavior.

## Verification

- `npm run check` **PASS** on `5627905` in a single attempt: schema consistency, production build/typecheck, all 11 frontend unit tests, the full Rust workspace suite (including 160 `lib` tests and 27 `providers` tests), and requirements traceability (203 requirements, 70 screens, 13 QA scenarios, 23 baseline files). Existing explicitly ignored native Keychain, gateway, or evidence-gated provider tests remain ignored. Vite reports the existing large JavaScript chunk warning.
- The normative PRD grew above the requirement tables while the slice was implemented, so every `line` reference in `requirements.csv` had drifted by one and the traceability assertion failed. The references were re-derived from the same regexes `scripts/check_requirements.py` asserts against — not hand-edited — and the change is line numbers only, with no requirement text, status or evidence altered.
- `cargo check --manifest-path src-tauri/Cargo.toml --bin tradex --features desktop,integration-test` and the matching `cargo build` **PASS**.
- Rust regression tests **PASS**: `acknowledged_unverified_permissions_remain_blocked_for_live_arming` rejects both the displayed eligibility and the direct Arm command; `inactivity_deadline_tick_disarms_without_an_ipc_request` verifies persisted timeout state without dispatching another command. Existing sleep/resume, restart, per-account disable and Disable All tests also pass.
- `tests/live-arming-ui.mjs` previously verified the Arm identity dialog, Escape/Enter focus behavior, one-account Arm/Disable, Disable All, desktop/768/390 layouts, zero provider calls and zero browser console errors. A native Tauri fixture run on this build independently confirmed explicit Arm and inactivity disarm.
- Native idle-timeout observation used only a temporary SQLite workspace and synthetic Trading 212 identity. The test set the timeout to one minute, explicitly armed at `2026-09-25T20:05:01.391803Z`, then observed `account.arming.changed` with `DISARMED / INACTIVITY_TIMEOUT` at `2026-09-25T20:06:01.477505Z` (about 60 seconds later). The UI displayed the same persisted reason. No provider request was made.
- Native screen-lock observation used the latest embedded Tauri QA build and the same isolated workspace. The synthetic account was explicitly armed at `2026-09-25T21:04:02.730060Z`; the read-only macOS observer received `com.apple.screenIsLocked` at `2026-09-25 22:04:15.822` local time, and SQLite recorded `account.arming.changed` as `DISARMED / SESSION_INACTIVE` at `2026-09-25T21:04:15.822470Z`. The observer received `com.apple.screenIsUnlocked` at `22:04:17.305`; after unlock the UI still showed the persisted disarm reason. No provider request was made.

## Remaining acceptance gate

The first native lock attempt used the macOS lock shortcut while the isolated app showed a synthetic Live account as `ARMED`. The app process remained alive and its window became unavailable, but read-only SQLite samples still showed `ARMED / EXPLICIT_USER_ARM` at 20:17 and 20:21 UTC. The account only disarmed at 20:27:54 UTC with `INACTIVITY_TIMEOUT`; that attempt did not verify immediate lock disarm.

The public `NSWorkspaceSessionDidResignActiveNotification` is documented for a user session switching out; that contract does not establish that screen locking fires it ([Apple Developer Documentation](https://developer.apple.com/documentation/appkit/nsworkspace/sessiondidresignactivenotification)). The native host also observes loginwindow's `com.apple.screenIsLocked` distributed notification with `NSNotificationSuspensionBehaviorDeliverImmediately`. The native lock/unlock runtime check passed on this macOS host: the lock notification and durable `SESSION_INACTIVE` disarm event were observed in the same second, and the UI retained that state after unlock. This uses an undocumented loginwindow notification, and Apple's distributed-notification server may delay or drop messages under load; the result proves the tested host only, while session-loss and inactivity guards remain additional protections. S33 cross-surface regression remains pending, so surface requirement F1 stays `IMPLEMENTED_UNVERIFIED`.

All runtime arming in this evidence used synthetic local fixtures; no Live account, provider order, cancellation, or other provider write was used.

## Known pre-existing issue: `WORKSPACE_BUSY` flake under parallel test load

While re-running the gate, `providers.rs` intermittently failed with
`WORKSPACE_BUSY` ("This workspace is open in another process") returned by the
advisory `.tradex.lock` in `Store::open`, on `workspace.open` after an explicit
`drop`+reopen. Observed in `failed_initial_tests_remove_their_credential_and_keep_failed_cleanup_blocked_after_restart`
and `alpaca_cancel_identity_mismatch_and_workspace_reopen_never_delete` — neither
reaches any arming code path, and the failing call is an S01-era path.

Measured rates on this host:

- working tree (S22 #84): 4/24 standalone runs, 3/24 under three concurrent test
  processes;
- **baseline `dev@7f16481` in a detached worktree: 1/16** — the flake is
  pre-existing and is **not** introduced by this slice;
- `--test-threads=1`: 0/6 — it is a concurrency artefact, not a logic defect.

Consequence: the gate can require a retry under load; the recorded PASS was
obtained on the first attempt without other builds running, matching the
repository's documented practice of running the final check with no competing
cargo work. Two follow-ups are worth a separate ticket rather than a silent fix
inside this slice: `Store::open` currently maps *every* `flock` failure to
"another process holds this workspace", which misreports transient lock-resource
errors, and a bounded retry for the transient case would make the gate
deterministic. Tracked in the wayfinder map's **Not yet specified**, not resolved here.
