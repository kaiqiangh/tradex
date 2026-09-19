# S11 screener evidence

Spec start SHA: `e288898b6ca5b4938dec401069168281e725f0a3`; implementation completion SHA: `6a27b3b5ca6462f54fd7390d02566306181f0e49` on `dev`; final review-hardening SHA: `87f8877052f66926d04588fa55af599a5ec5933b`; browser-helper endpoint SHA: `e5e97f01d749cfc1de861fab6bf0be527c35c72f`. The hardening SHA shares the screener sequence lookup and the browser target assertion helper without changing the public contract. The helper endpoint adds distinct new/current target assertions; its syntax check passed, while fresh CUA execution remains unverified because the native Mac window was locked. This evidence amendment is documentation-only. GitHub #38 remains OPEN until the user closes the ticket after review. The preceding S11 parse/run implementation is covered by the earlier commits listed in the issue history.

The #38 slice adds a workspace-scoped SQLite screener library with `screener.list`, `screener.save`, and `screener.update`, including bounded reviewed inputs, canonical revision recalculation, reopen persistence, and optimistic state-version checks. `screener.attach` accepts only explicitly selected canonical instrument IDs and returns workspace-bound `ThreadContextRef` values. The Rust dispatcher, generated JSON Schema/TypeScript contract, frontend Markets flow, Backend ARD §41.15, and the Chinese contract notes are updated together.

## Automated checks

- `npm run schema:check` — PASS; Rust schema, generated JSON Schema and TypeScript agree.
- `npm run typecheck` — PASS.
- `npm run build` — PASS; Vite emits the existing large-client chunk warning.
- `npm run test:unit` — PASS, 6 tests.
- `cargo test -p tradex --all-targets -- --test-threads=1` — PASS; 89 library tests and all package integration targets, with only the repository's explicitly ignored native/provider checks skipped.
- `cargo test --workspace --features integration-test -- --test-threads=1` — PASS; 95 library tests and all workspace integration targets, with only the repository's explicitly ignored native/provider checks skipped.
- `cargo clippy -p tradex --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check`, `git diff --check`, and `node --check tests/screener-ui.mjs` — PASS.

The complete automated command set above was rerun against final review-hardening SHA `87f8877052f66926d04588fa55af599a5ec5933b`; `3027c6a5754a6819e8a69e84280c0f4bf29f4cb0` and this evidence update only change documentation. Executing the helper against the browser remains unverified.

The persistence integration test uses a temporary real SQLite workspace and covers first save, workspace-scoped listing, reopen, edited revision, stale update rejection, selected-only attach, duplicate selection rejection, canonical instrument validation, forged context hash rejection, and unchanged observable domain/runtime boundary state. The public IPC surface has no outbox, approval, arming, or credential read command; the test therefore compares the domain snapshot plus sanitized account, risk, Gateway, and model projections and documents that private/future boundaries have no mutation path in this slice.

## Runtime evidence

The earlier Rust-backed browser evidence covers `PARSE → revision → RUN`, typed FilterSpec/RankSpec display, source-gated `COMPLETED`/`EMPTY`/`FAILED` states, stale revision handling, retry input preservation, candidate identity, blocked external mode, and the existing 390/768/1280 helper assertions for the #37 flow.

The current isolated in-app browser run additionally completed this #38 path against a temporary workspace: open the screener, parse and run a fixture result, save `Growth leaders`, reopen the saved definition, edit it into a new revision, observe an empty result at the fixture's `0.25` threshold, recalculate at `0.18`, rerun to `COMPLETED`, select `equity:US:AAPL`, and attach it to a new Thread; a separate manual run attached the selected result to the current Thread's next Turn. The resulting Thread showed the canonical context reference and no automatically started Turn. The helper at `e5e97f0` now re-runs the fixture with unlinked `equity:US:MSFT` to exercise current-target routing and asserts the new composer remains empty while the Turn composer gains the second context; only `node --check` was run for that updated helper. The run was partial in the desktop sense: the in-app browser remained at its constrained narrow viewport and the CUA native-window helper could not establish fresh 390/768/1280 or keyboard evidence during this pass. Those widths remain unverified for the #38 additions and are not marked PASS.

## Evidence boundary

Saving and attaching write only the screener projection or return pending context references. They do not call external providers, persist candidate rows or provider payloads, restore Live authority, mutate account/risk/approval/arming/Gateway state, write credentials, create a Thread, or start a Turn. Reopened definitions must pass the current source gate before a new run. Fixtures cannot establish provider entitlement or Live authority; blocked external states remain explicit.

Related work: #36 (spec), #37 (source-gated implementation), #38 (persistence and selected candidate attachment; implemented but OPEN). No `dev → main` PR is created for this partial map.
