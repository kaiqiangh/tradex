# S32 Settings, account health, privacy, and recovery evidence

**Status:** S32 implementation and isolated native UI/IPC verification are complete on `dev`. The implementation ticket is [#139](https://github.com/kaiqiangh/tradex/issues/139), linked to specification [#138](https://github.com/kaiqiangh/tradex/issues/138).

- Source commit: `0593ed78cca75a47bac35170916da8727f78a09b`
- Review baseline: `c1c146d015da7601a4a46d971433718ef0285fbe`
- Scope: RevC UI Spec J5–J7 and K7; PRD FR-034, FR-064, AC-050, NFR-013, NFR-004, OPS-007; #138/#139.
- Provider scope: Alpaca, Binance, Bitget, and Trading 212 integrations remain skipped at the user's direction. S27 physical sleep/wake and S33 full-page regression remain separate open gates.

## Delivered behavior

Settings → Account Health reads only the local `account.list` projection and presents connection, authentication, private stream, reconciliation, execution eligibility, and arming independently, with last-sync and last-private-stream timestamps. Reload is explicit. Disable All Live remains a separate explicit local TradeX arming action; opening the page does not refresh a provider. `account.list` is a local store read in the Tauri dispatcher.

Appearance reports the prototype-only System/Light surface and leaves Dark unavailable without claiming a persisted preference. About reports the Tauri application version, native runtime component state, RevC baseline, and the historical prototype disclaimer. Privacy reports telemetry disabled, crash reporting not configured, and automatic updates not configured. No consent toggle is shown without a collector.

The reusable recovery surface maps the 20 RevC K7 categories, converts known model/provider error codes to canonical categories, and maps unknown or malformed errors to `INTERNAL_ERROR` without echoing raw error fields. Generic command errors now use canonical remediation text; the existing structured reserved-capacity message retains its typed monetary context. The test-only diagnostic scan executes the real isolated IPC dispatcher with synthetic canaries, verifies that its output contains none, and proves the scanner detects all four canary forms.

## Verification evidence

- `npm run check` — passed at the source commit: schema agreement, TypeScript check and production build, 23/23 Node tests, `cargo test --workspace` (257 library tests plus the non-ignored workspace fixture/integration tests), representative diagnostic secret scan, and traceability for 203 requirements, 70 screens, 13 QA scenarios, and 23 baseline files.
- `npm run desktop:build` — passed; produced `target/release/tradex`. The frontend emitted the existing large-bundle warning.
- `cargo clippy --workspace --all-targets --all-features` — exited 0 with existing warnings, including `result_large_err` and fixture warnings. `cargo clippy --workspace --all-targets -- -D warnings` fails on warnings in untouched Rust files; S32 changes no Rust source.
- `cargo fmt --all -- --check` — reports formatting differences in untouched existing Rust files; no formatter was run to rewrite unrelated files. `git diff --check` passed.
- Focused recovery tests — 6/6 passed, including all 20 categories, accessible alert markup, known-code mapping, secret-safe fallback, and sanitized generic command errors.
- Native TradeX integration-test build — launched as an isolated Tauri app against temporary SQLite workspace `48734a94-090a-4d87-9668-855766067a24`. The Account Health view showed six independent dimensions and both timestamps; Appearance, About, local component status, telemetry/crash/update status were present in the accessibility tree. The local workspace contained only the TradeX Local Paper simulation.
- Native keyboard/narrow-window check — keyboard Tab reached the visible “Reload health snapshot” control with a visible focus ring. A tiled narrow desktop window kept Settings navigation, account-health cards, and About facts readable without observed horizontal overflow. S33's full 768/390-pixel and cross-page regression remains pending.
- No provider credentials were entered, no provider refresh/connect or order command was invoked, and no financial write was made. The temporary UI server and native test app were stopped after verification. The clickable prototype was not changed.

## Remaining gates

S32 does not close the provider integrations, S27 physical sleep/wake, S33 full-page accessibility/responsive regression, S34 performance and signed packaging, or S35 final acceptance. The map remains open; do not create a `dev` → `main` PR until all remaining map work is verified on `dev`.
