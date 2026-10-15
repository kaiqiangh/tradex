# S27 parent acceptance audit

Parent: [Recover Live execution and coordinate interrupted work](https://github.com/kaiqiangh/tradex/issues/107). Audit baseline: `f5ed6d114a6b7e5cc905f8ab3e3cf015fc5191cf`. Date: 2026-09-29 UTC. [中文](s27-parent-acceptance-audit_zh.md).

**IN_PROGRESS: the parent remains OPEN.** The four implementation tickets are closed; their closure does not prove every parent or full-map acceptance gate.

## Requirement audit

| Parent user stories | Authoritative evidence | Remaining boundary |
|---|---|---|
| 1–4: restart/reopen, durable work, timely recovery, readiness | [S27.1](s27-1-live-startup-recovery-evidence.md): SQLite reopen, all supported Live accounts selected, stale/disarmed before recovery; native T212 read-only recovery completed 0.682 s after reopen on 2026-09-29 | Native run had one connected Live account and no open orders. Complete provider/open-order and representative crash timing acceptance is not established. |
| 5: sleep/resume | [S27.2](s27-2-live-resume-recovery-evidence.md): deterministic invalidation/recovery and native application reactivation; macOS wake observer source compiled | Actual OS sleep/wake followed by completed provider recovery remains unverified. |
| 6–7: account faults and policy scope | S27.2 and [S27.4](s27-4-model-failure-isolation-evidence.md) account-scoped auth/stream faults; [S23 policy ordering](s23-pre-dispatch-release-evidence.md) and [concurrency](s23-concurrency-transaction-evidence.md) | Stream-health injection proves the persisted boundary, not a real Live websocket outage. |
| 8–11: unknown attempts, dispatch distinction, fills, idempotence | S27.1/S27.4 durable unknown reservations and fail-closed evidence; [S24](s24-live-order-gateway-evidence.md), [S25](s25-4-live-manual-resolution-evidence.md), [S26 settlement](https://github.com/kaiqiangh/tradex/issues/103#issuecomment-5864907639) | Complete provider-hosted recovery scenarios remain separate. Unknown CANCEL keeps the established user-triggered exact-order recovery boundary. |
| 12–14: priority and bounded load | [S27.3](s27-3-provider-scheduling-evidence.md) P0–P3 priority, concurrency/queue limits, account/IP cooldowns; S27.4 actual ProviderJob P0 progress while three P3 slots remain occupied | No remote quote/UI producer currently exists to coalesce. The producer-side requirement remains in Backend ARD §36.2; actual provider workload is not proven by fixtures. |
| 15: model failure isolation | S27.4 five canonical faults, trusted approval/preparation/cancel/reconciliation and explicit fallback; real-child Gateway regressions | Synthetic broker responses and model gateway projections; no real broker mutation or combined real-sidecar financial acceptance. |
| 16–18: health/remediation, separate Arm, keyboard/layout | S27.4 four gateway states × 390/768/1280px, Enter, account health, no auto-arm; native S27.1 account restoration | S33 full-app accessibility/regression and signed-package lifecycle remain separate. |

The requirement inventory is updated only where named evidence exists. No S27 requirement is promoted to VERIFIED by this audit. Local implementation, physical lifecycle, real-provider evidence and signed-package acceptance remain distinct.

## Native attempt, 2026-09-29

- Started ordinary `npm run desktop` against the audit baseline and existing default workspace. The native build succeeded; `target/debug/tradex` is running under the original launch session. No integration fixture or alternative credential path is used.
- The workspace reopened at `2026-09-29T22:35:14.836325Z`. Read-only SQLite inspection found the connected T212 Live account at sequence 68, STALE/UNVERIFIED/UNCHECKED/STALE/BLOCKED/DISARMED, retaining `lastSuccessfulSync=2026-09-29T17:35:17.79051Z`.
- At approximately `22:58 UTC`, read-only process sampling found both provider credential workers in `NativeVault::get` / `SecItemCopyMatching`. No completed fresh T212 observation is established for this attempt.
- The desktop tool could not resolve the development binary as an application. A Finder call took over 20 minutes; neither that tool delay nor failed application selection is a product PASS/FAIL. The OS wake event was not induced.
- The user was asked to switch to TradeX, complete any system Keychain authentication locally, then sleep/wake and unlock the Mac. Completion requires fresh recorded provider observations after the actual lifecycle event; an acknowledgement alone is insufficient.

No broker order placement/cancellation, Arm action, credential entry, Keychain ACL change, or direct SQLite mutation was performed. The running app may complete read-only recovery once system authentication is available. Parent #107 and map #1 remain OPEN; S28 is not started while this gate is pending.

## Audit-document verification

Serial independent Standards and Spec reviews against the audit baseline both PASS, with zero actionable findings. Local evidence-link, 14-row scope/no-VERIFIED-promotion, requirement inventory and `git diff --check` checks pass. This verifies the documentation delta only; it does not close the pending native acceptance gate or rerun the application test suites. The issue progress comment records the delivery SHA.

## Follow-up: native provider recovery completed, 2026-09-30

The earlier launch exited with code 0. After confirming no TradeX process remained, ordinary `npm run desktop` was restarted at `dev@b0750634d5e770fc5c955b061908ed4da66578aa`. The development executable SHA-256 was `34fdfcd760e56a6eec53a31c4383d5f123ad63e1bef2007decb24e61afdee30d`. Its credential workers initially waited in `SecItemCopyMatching`; the desktop tool explicitly prohibited access to SecurityAgent. No credential or security permission was changed by the agent.

Read-only SQLite/outbox inspection established real T212 recovery: sequence 78 at `08:03:12.085077Z` persisted `SESSION_RESUMED / STALE / BLOCKED / DISARMED`; sequences 79–81 recorded further resume triggers. Sequence 82 at `08:03:23.245049Z` persisted `ONLINE / VALID / CONFIGURED` with `lastSuccessfulSync=08:03:23.244385Z`, 3 positions, 0 open orders and 6 recent orders. Sequence 83 at `08:03:23.25236Z` became `CURRENT` while remaining `BLOCKED / DISARMED` and explicitly requiring a new Arm. A subsequent process sample no longer found `SecItemCopyMatching` waits. The credential/provider-completion blocker is cleared for this run. No broker order write or Arm was performed.

This does not establish physical OS wake or another `<5 s` timing result: multiple resume triggers occurred, and the provider request start was not measured. After the user reported sleep/wake/unlock, the power log captured at approximately `08:15 UTC` and rechecked at `08:19 UTC` still contained no 2026-09-30 Sleep/Wake/DarkWake events; the account remained at sequence 83 without a later recovery event. A pre-existing `caffeinate -s -d -i` process still held sleep/display assertions. It was not stopped or modified. The user was asked to temporarily stop their process and perform actual sleep/wake/unlock followed by activating TradeX. Physical lifecycle acceptance remains pending; the broader evidence boundaries in the table remain separate. Parent #107 stays OPEN and S28 is not started.

The five-file follow-up delta passed serial independent Standards and Spec reviews against `b0750634d5e770fc5c955b061908ed4da66578aa`, with 0 findings on each axis. Paired evidence tokens, local links, executable hash, requirement inventory and `git diff --check` passed. No application test suite was rerun for this documentation-only update.

## User-directed verification deferral, 2026-09-30

The user explicitly requested skipping the current physical OS sleep/wake validation and continuing to S28. Record this gate as `SKIPPED_BY_USER / unverified`: it no longer blocks serial scheduling of S28, but is not a PASS or removal from the final acceptance scope. Parent #107 remains OPEN; no requirement is promoted to VERIFIED. Revisit physical lifecycle evidence before full-map acceptance. This scheduling exception grants no Arm or real-money transaction authority.
