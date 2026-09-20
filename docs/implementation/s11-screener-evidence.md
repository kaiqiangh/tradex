# S11 screener evidence

Spec start SHA: `e288898b6ca5b4938dec401069168281e725f0a3`; implementation completion SHA: `6a27b3b5ca6462f54fd7390d02566306181f0e49` on `dev`; final review-hardening SHA: `87f8877052f66926d04588fa55af599a5ec5933b`; browser-helper endpoint SHA: `e5e97f01d749cfc1de861fab6bf0be527c35c72f`. The hardening SHA shares the screener sequence lookup and the browser target assertion helper without changing the public contract. The fresh CUA recheck below was run against current `dev` `ad3b262`; the preceding S11 parse/run implementation is covered by the earlier commits listed in the issue history.

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

The complete automated command set above was rerun against final review-hardening SHA `87f8877052f66926d04588fa55af599a5ec5933b`; later commits only change documentation. The fresh CUA recheck below was run against current `dev` `ad3b262`.

The persistence integration test uses a temporary real SQLite workspace and covers first save, workspace-scoped listing, reopen, edited revision, stale update rejection, selected-only attach, duplicate selection rejection, canonical instrument validation, forged context hash rejection, and unchanged observable domain/runtime boundary state. The public IPC surface has no outbox, approval, arming, or credential read command; the test therefore compares the domain snapshot plus sanitized account, risk, Gateway, and model projections and documents that private/future boundaries have no mutation path in this slice.

## Runtime evidence

The earlier Rust-backed browser evidence covers `PARSE → revision → RUN`, typed FilterSpec/RankSpec display, source-gated `COMPLETED`/`EMPTY`/`FAILED` states, stale revision handling, retry input preservation, candidate identity, blocked external mode, and the existing 390/768/1280 helper assertions for the #37 flow.

The fresh isolated in-app browser CUA recheck on 2026-09-20 used a temporary Rust workspace and completed the #38 path with semantic keyboard actions: parse/review/run, save `Growth leaders`, reopen the reviewed definition, update it to a new revision, rerun a completed fixture result, attach only `equity:US:AAPL` to a new Thread, create that Thread, then attach only the distinct `equity:US:MSFT` to the current Thread's next Turn. The new-thread composer showed one context and the current Turn composer showed two; no Turn was started automatically. The same run covered EMPTY and FAILED retry with reviewed input preserved, and 390/768 viewport checks with `scrollWidth === clientWidth`; browser console warn/error count was zero. The fixture label remained `SYNTHETIC_SCREENER_FIXTURE` and no real credentials or provider calls were used.

The unlocked native TradeX window was also opened at `tauri://localhost` and navigated to Markets. Its real default workspace correctly showed `Market data is not connected` / `Provider connections are not available in this build`, so no native fixture result was claimed and no account or credential was touched. Native provider-backed screener truth remains outside this slice; the browser fixture is the responsive/keyboard evidence for the bounded UI contract.

## Evidence boundary

Saving and attaching write only the screener projection or return pending context references. They do not call external providers, persist candidate rows or provider payloads, restore Live authority, mutate account/risk/approval/arming/Gateway state, write credentials, create a Thread, or start a Turn. Reopened definitions must pass the current source gate before a new run. Fixtures cannot establish provider entitlement or Live authority; blocked external states remain explicit.

Related work: #36 (spec), #37 (source-gated implementation), #38 (persistence and selected candidate attachment). This slice is verified for the local Rust/SQLite/IPC/UI boundary; real provider entitlement, native provider-backed screener data and S33 remain future boundaries. No `dev → main` PR is created for this partial map.
