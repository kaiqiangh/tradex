# S19 #70 Binance Testnet private-stream evidence

**Status:** Local acceptance for implementation ticket #70 is complete. Parent Spec [#67](https://github.com/kaiqiangh/tradex/issues/67) remains open for provider-hosted Testnet verification; this fixture evidence does not satisfy that external gate.

- Branch: `dev`
- Review baseline: `ce4d20cfb57dd78c5d2cbd72cecf920674c31ae2`
- Implementation commit: `6da34af99a5a7ee1948b480de4160a8036c3b4e6`
- Scope: [#70 — 接收 Binance Testnet 私有流并恢复账户状态](https://github.com/kaiqiangh/tradex/issues/70)
- Contract: Backend ARD §41.27, Frontend ARD §13.15, UI Spec §14.13

## Delivered behavior

The desktop backend starts a signed private WebSocket worker only for the exact connected Binance Spot Testnet identity. It confines credentials and signed subscription data to the backend, validates the acknowledged subscription and each event identity, and bounds frames and event processing. Execution reports retain exact provider status and decimal quantities, deduplicate fills, and reject stale or conflicting updates. Changed account balances update only the named assets. Delta-only and unknown events keep reconciliation required.

Stream and recovery observations update the existing order-book and Account health projections with their outbox events in one SQLite transaction. Disconnect, workspace reopen, pause, and system resume preserve the last trusted rows while marking health degraded and the book stale. Fixed-route REST reconciliation verifies the account, refreshes open orders and balances, and advances bounded BTCUSDT/ETHUSDT order and trade history before health becomes current. No renderer stream command or Live, Local Paper, Agent, approval, reservation, or Order Gateway capability was added.

## Verification

On implementation commit `6da34af99a5a7ee1948b480de4160a8036c3b4e6`:

- `npm run check` — passed: generated schema/type agreement, TypeScript and production build, frontend tests (9/9), Rust unit tests (139/139), runnable workspace integration suites, and requirement traceability (203 requirements, 70 screens, 13 QA scenarios).
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — passed.
- `cargo check --manifest-path src-tauri/Cargo.toml --features desktop --bin tradex` — passed.
- `cargo build --manifest-path src-tauri/Cargo.toml --bin tradex-ipc --features integration-test` — passed on the implementation commit.
- `python3 scripts/check_requirements.py`, `node --check tests/provider-ui.mjs`, and `git diff ce4d20cfb57dd78c5d2cbd72cecf920674c31ae2..6da34af99a5a7ee1948b480de4160a8036c3b4e6 --check` — passed.
- Rust loopback WebSocket/HTTP and temporary-SQLite tests passed for signed subscription, bounded reconnect/backoff, reconciliation after stream termination, explicit pause/stale state, deduplication, and account/workspace binding.
- Rust-backed browser fixture validation passed. A synthetic `outboundAccountPosition` persisted USDT free/locked balances; health remained `REQUIRED` until reconciliation succeeded. A partial fill was stored once despite duplicate trade delivery, and a late `NEW` event did not roll back the partial fill. Disconnect marked the book stale and health degraded; bounded fixture reconciliation restored `CURRENT`. Accessible stream and reconciliation status remained visible with no horizontal overflow at 390, 768, and 1280 px.
- Standards and Spec reviews were completed serially against the fixed baseline; no findings remain.

The production build emitted the existing Vite bundle-size warning. Native Keychain/gateway tests marked ignored by the suite are not claimed as verified here.

## Evidence boundary

All provider credentials, WebSocket events, and REST responses were disposable local fixtures. No real Binance Testnet credential, external Binance request, or broker order was used. The native desktop target compiled, but this does not claim a real provider-hosted Testnet session. Parent Spec #67 remains open until its external acceptance is completed.
