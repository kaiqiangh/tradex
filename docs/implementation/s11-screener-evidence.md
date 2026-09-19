# S11 screener evidence

Implementation commits: `0d528f3c17a947cbaa2fea94739c0cf7ed7b9c8f`, `3ad37bac45c89031a1c00f31935f20486aa2f28d`, `cbaad10e01779633c10474e17618c9eeb30bff78`, `f7a0092029847a58588f7dc01e3f6b0bd931b38e`, `a27e5cf054101858c55ff5e34ad1bacc83dc42c3`, `b9b17964c6531dabd8ff11f05075ea89cf175bc7`, `a3eac717b6d6db22a77305bd2b4374e5950417dd`, `b6408e8542ec85bc1b65c7a0be59c9145f1a6883`, `ba0a4f154d3192bf0d7e716683b4a158bd476ccd`, `77ed3ec2bc582c979f4d9e5e2333371226ed9c03`, `d29f3fb0044b04afe10df2d1fb166b4676674a05`, `dffed8a6929c7d9b54c4ea93507cfe824637ec0c`, `b2d6234df32741e74b6a4393e7eac43852d0fb51`, `9c45cccd10674bb9c48771e664e19be2edc6290d`, `697387958144da8a6be8d4f01ddb141bd1cd7dd8`, `28268afcce8d4be385901a0355c0c96f7b739be5`, `e983c88ede302fd933729f05dbf1d777a1b2f44a`, `2dd4b10648949df676b3147c5cb662f3a32c437f`, and `b9f739982815f7682e4888668efd85824d1ffedd` on `dev`. The final hardening rejects unsupported universe/filter/rank/limit language with allowlisted universe and rank clauses, conjunction validation, repeated-field fail-closed behavior, filter/rank separation, threshold isolation, and complete integer-limit validation, closes the isolated bridge lifecycle, and adds a non-fixture browser bridge for the external block state.

The S11 `market.screen` slice now has a generated Rust/JSON Schema/TypeScript contract, a source-gated Rust dispatcher, a deterministic integration fixture, and a Markets screener builder. `PARSE` produces bounded `FilterSpec`/`RankSpec` values and a `sha256:` revision, reports field-level unsupported conditions and operators, and does not silently run a failed parse. Editing a condition clears the revision and disables `RUN` until the revision is recalculated. `RUN` rejects stale revisions, returns typed `BLOCKED_EXTERNAL` without provider calls when the source gate is unavailable, and returns deterministic `COMPLETED`/`EMPTY` fixture results with canonical instrument IDs and provenance.

## Automated checks

- `npm run schema:check` — PASS; Rust schema, generated JSON Schema and TypeScript agree.
- `npm run typecheck` — PASS.
- `npm run build` — PASS; Vite emitted the existing chunk-size warning for the 1.477 MB client bundle.
- `npm run test:unit` — PASS, 6 tests.
- `cargo test -p tradex screener --lib` — PASS, 5 screener unit tests.
- `cargo test -p tradex --test screener -- --test-threads=1` — PASS, 2 control-plane integration tests.
- `cargo test --workspace --features integration-test -- --test-threads=1` — PASS; 95 library tests plus integration targets, with only the repository's explicitly ignored native/provider checks skipped.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check`, `git diff --check`, `node --check tests/screener-ui.mjs` — PASS.
- `node --check tests/screener-ui.mjs` — PASS.

## Runtime evidence

Direct Rust IPC and the integration browser bridge both completed `PARSE → revision → RUN`. The default request parsed three predicates (revenue growth, estimate revision, RSI), and the fixture run ranked `MSFT` then `AAPL` by revision strength with source `FX-SCREENER` and fixture label `SYNTHETIC_SCREENER_FIXTURE`. A revised revenue-growth threshold of `0.20` reduced the candidate set to `AAPL`.

The browser path reached Markets → Open screener → Parse conditions, displayed the typed FilterSpec/RankSpec, disabled Run after a threshold edit, re-enabled it after Recalculate revision, and rendered `COMPLETED`, the canonical `equity:US:AAPL` identity, source/provenance timestamps, fixture label and the synthetic limitation. The CUA run asserted `EMPTY` and `FAILED`, stale messaging, retry input preservation, `aria-live`, no page overflow at `1280 × 900`, `768 × 900`, and `390 × 900`, and zero captured console `warn`/`error` entries. An unsupported `P/E` condition returned a field-level failure; opening the candidate entered the existing Market detail with the exact canonical ID. The final helper includes natural-language focus and teardown-safe blocked-mode assertions, and a direct browser-bridge smoke returned `BLOCKED_EXTERNAL` with zero candidates through the non-fixture Rust child.

## Evidence boundary

This slice is read-only. It does not call external providers, write SQLite or DuckDB, persist a saved filter, or attach candidates to a Thread. OD-001/OD-003 entitlement and adapter availability remain `BLOCKED_EXTERNAL`; both the control-plane integration test and the companion non-fixture browser bridge cover that state. The fixture cannot establish provider entitlement or Live authority. A later live CUA retry during this review was blocked by the local control plane being unavailable, so it adds no new browser evidence. Persistence and candidate attachment are tracked by #38.

Related work: #36 (spec), #37 (implementation), #38 (follow-on persistence/attachment).
