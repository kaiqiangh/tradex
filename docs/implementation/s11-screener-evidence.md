# S11 screener evidence

Implementation SHA: `0d528f3c17a947cbaa2fea94739c0cf7ed7b9c8f` on `dev`.

The S11 `market.screen` slice now has a generated Rust/JSON Schema/TypeScript contract, a source-gated Rust dispatcher, a deterministic integration fixture, and a Markets screener builder. `PARSE` produces bounded `FilterSpec`/`RankSpec` values and a `sha256:` revision. Editing a condition clears the revision and disables `RUN` until the revision is recalculated. `RUN` rejects stale revisions, returns typed `BLOCKED_EXTERNAL` without provider calls when the source gate is unavailable, and returns deterministic `COMPLETED`/`EMPTY` fixture results with canonical instrument IDs and provenance.

## Automated checks

- `npm run schema:check` — PASS; Rust schema, generated JSON Schema and TypeScript agree.
- `npm run typecheck` — PASS.
- `npm run build` — PASS; Vite emitted the existing chunk-size warning for the 1.477 MB client bundle.
- `npm run test:unit` — PASS, 6 tests.
- `cargo test -p tradex --test screener -- --test-threads=1` — PASS, 2 control-plane integration tests.
- `cargo test --workspace --features integration-test -- --test-threads=1` — PASS; 94 library tests plus integration targets, with only the repository's explicitly ignored native/provider checks skipped.
- `node --check tests/screener-ui.mjs` — PASS.

## Runtime evidence

Direct Rust IPC and the integration browser bridge both completed `PARSE → revision → RUN`. The default request parsed three predicates (revenue growth, estimate revision, RSI), and the fixture run ranked `MSFT` then `AAPL` by revision strength with source `FX-SCREENER` and fixture label `SYNTHETIC_SCREENER_FIXTURE`. A revised revenue-growth threshold of `0.20` reduced the candidate set to `AAPL`.

The browser path reached Markets → Open screener → Parse conditions, displayed the typed FilterSpec/RankSpec, disabled Run after a threshold edit, re-enabled it after Recalculate revision, and rendered `COMPLETED`, the canonical `equity:US:AAPL` identity, source/provenance and the synthetic limitation. At the mobile viewport override `390 × 844`, the builder remained visible without page overflow; captured console `warn`/`error` entries were zero.

## Evidence boundary

This slice is read-only. It does not call external providers, write SQLite or DuckDB, persist a saved filter, or attach candidates to a Thread. OD-001/OD-003 entitlement and adapter availability therefore remain `BLOCKED_EXTERNAL`; the fixture cannot establish provider entitlement or Live authority. Persistence and candidate attachment are tracked by #38.

Related work: #36 (spec), #37 (implementation), #38 (follow-on persistence/attachment).
