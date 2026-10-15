# S31 workspace backup, restore, migration, and retention evidence

**Status:** Implementation, isolated native workflow verification, full repository checks, ordinary desktop build, and serial Standards/Spec reviews are complete on the feature branch. The S31 implementation issue is [#137](https://github.com/kaiqiangh/tradex/issues/137), linked to the open specification issue [#136](https://github.com/kaiqiangh/tradex/issues/136).

- Branch: `features/s31-workspace-backup-import-retention`
- Review baseline: `015f4ba19b0dcba138d2a48fafb42b6323821e83`
- Contract source: RevC PRD FR-039, FR-040, FR-063 and AC-047; Backend ARD §38 and §45.1; implementation scope in #136/#137.
- UI: Settings → Data & Storage.

## Delivered behavior

Backup Now and Export Workspace write a bounded `.tradex` archive with a versioned manifest, workspace identity, storage schema version, member size, and SHA-256. The archive currently contains one fixed-name SQLite workspace snapshot. Temporary output is private and published without replacing an existing file. Preview validates the archive and database before showing the workspace, compatibility, warnings, and account credential-reference checks. The native process checks only whether each matching Keychain item can be read; it returns a status, never the reference or secret. An available item remains unverified until fresh authentication and reconciliation.

The native Keychain result is bound to the validated manifest hash and is applied only if that same hash is observed by the preview or restore operation. A replaced archive cannot inherit credential status from an earlier read.

Restore checks the preview manifest hash again, extracts into a private staging directory, migrates supported schemas, invalidates issued approvals and current cancellation intents, stales and disarms accounts, stops the Gateway, resets the model session, and verifies SQLite integrity before publishing a new workspace. Missing Keychain entries persist as `MISSING` and failed/blocked; inaccessible checks remain `UNCHECKED`. Before publishing, TradeX writes a backup of the currently open workspace. Failed archive checks leave the current workspace untouched.

Market history and cache files are excluded from archives because their source-specific data rights are not verified. Retention can be saved without deleting data, then explicitly applied to unlinked ordinary artifacts, old local OHLCV rows, and regular files directly under `cache/`. Financial execution, reservation, settlement, approval, reconciliation, and audit records remain outside the deletion set. No retention period is selected by default.

The UI exposes native file dialogs, a preview and explicit restore confirmation, status/error feedback, focus trapping/return, and separate Save and Apply actions for retention. It identifies the data excluded from the archive and the fail-closed account state after restore.

## Verification evidence

- `cargo test --test workspace_archive` — passed: archive/export/preview/restore, tamper rejection before restore-directory creation, retention boundaries, and schema 37→38 migration.
- `cargo test workspace_restore_invalidates_issued_live_approval_and_keeps_accounts_disarmed --lib` — passed: issued approval is invalidated, Live remains disarmed, and a matching missing Keychain item is persisted as blocked.
- `cargo test --lib workspace_archive` — passed: credential status with a stale manifest hash remains `NOT_CHECKED` in preview and restore.
- `npm run typecheck` and `npm run schema:check` — passed after IPC generation.
- `npm run check` — passed: Rust/JSON/TypeScript IPC agreement, frontend production build, 17/17 Node tests, 257/257 library tests, all non-ignored workspace fixture/integration tests, and traceability for 203 requirements, 70 screens, 13 QA scenarios and 23 baseline files. Existing provider financial-prerequisite, explicit Keychain and time-boundary tests remain ignored by their documented gates; no user provider account was contacted.
- `cargo clippy --workspace --all-targets --all-features` — exited 0; it emitted existing `result_large_err`, fixture dead-code and other warnings.
- `npm run desktop:build` — passed and produced `target/release/tradex`; the frontend retains its existing 5.35 MB bundle-size warning.
- `git diff --check` — passed. Repository-wide `cargo fmt --all -- --check` reports formatting differences in unchanged files/hunks; changed S31 Rust hunks were formatted without rewriting unrelated code.
- Review baseline: `015f4ba19b0dcba138d2a48fafb42b6323821e83`. Standards review found no documented-rule violations and one non-blocking duplicated archive-validation/extraction heuristic. Spec review found no remaining acceptance gaps after binding native credential-check results to the validated manifest hash.
- Isolated native macOS TradeX build and UI flow — Backup Now wrote a `.tradex` file; Export Workspace saved `workspace.tradex` through the native Save panel; re-opening that file showed `Archive validated`, schema 38, one member, 725,682 bytes, **2 Accounts records**, and a synthetic missing-Keychain warning for the synthetic Live account. This verifies that workspace-local account rows without a `workspace_id` column are included in the preview count. A previous isolated restore succeeded into a new temporary workspace and left the synthetic account failed/disarmed with Refresh blocked until credentials are restored and reconciliation occurs. The UI warned that no provider request was made.
- Native retention settings saved successfully; Apply retention showed its confirmation and was cancelled, so no deletion ran. Import dialog keyboard focus returned to its trigger after cancellation. The desktop flow was exercised at 1280 px; isolated browser layout checks at 1280/768/390 px found no horizontal overflow. The browser did not substitute for native dialog or storage acceptance.
- All native actions used only `/private/tmp/tradex-s31-ui-default-safe`; `lsof` confirmed the running app's SQLite handles stayed under that isolated path. The fixture was explicitly synthetic, and no provider request, credential entry, account refresh, or financial write was made.

Alpaca, Binance, Bitget, and Trading 212 provider integrations are excluded by the user's current scope. No provider request or financial write was made. The S27 physical sleep/wake evidence remains skipped and unverified. The clickable prototype was not changed.

## Remaining delivery gates

Close only implementation issue #137 and merge this reviewed slice into `dev`. Keep parent spec #136 open. S27 physical verification remains skipped and unverified; provider integrations remain skipped at the user's direction. Continue the remaining non-provider Wayfinder tickets serially. S33 full-app accessibility, S34 performance/signed packaging, S35 final acceptance, and the final `dev`→`main` PR remain separate gates; do not create that PR until the entire map passes on `dev`.
