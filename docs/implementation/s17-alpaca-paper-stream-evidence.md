# S17 #59 Alpaca Paper private-stream evidence

Implementation commits on `dev`: private-stream lifecycle `ac45a0250febe9ae7e1146a39297f6dac4d5be88`; Rust-backed disconnect UI coverage `70a088d9b41f4ccf08039c7c53002a93499222ba`. This records local acceptance evidence for [#59](https://github.com/kaiqiangh/tradex/issues/59), not the full S17 parent gate.

## Result

The #59 private-stream lifecycle and its local acceptance checks pass. A loopback WebSocket fixture exercises the Rust account worker's Paper authentication, `trade_updates` subscription, partial-fill event, disconnect, reconnect and supervisor restart. The worker reconciles REST orders and fill activities after recovery; the test verifies three connections, one deduplicated REST fill, three REST order refreshes and zero order POSTs.

A separate temporary-SQLite provider test persists the partial fill and disconnected health, reopens the workspace, confirms the saved order book is stale, reconciles provider order/activity truth, and restores `privateStream=CONNECTED` with `reconciliation=CURRENT`. The React helper uses the real `tradex-ipc` sidecar and isolated Rust fixture responses. It verifies stale/degraded text on Order Drafts and Account Health at 768px and 390px, alongside stream fill projection, duplicate/late-event handling, order reload, and cleanup. The disconnect command is compiled only with `integration-test` and calls the same ControlPlane health transition used by the stream worker.

The browser helper reported all ten observations passing. Provider credentials and HTTP/stream responses were fixtures in a temporary workspace; no real Alpaca credential, external provider request, or Paper order was used.

## Checks

- `cargo fmt --all -- --check` — passed.
- `cargo check --bin tradex-ipc --features integration-test` — passed.
- `cargo test --lib account_worker_reconnects_and_reconciles_after_private_stream_disconnect -- --nocapture` — passed, 1 test.
- `cargo test --test providers alpaca_private_stream_gap_reopens_stale_and_rest_reconciliation_dedupes_fills -- --nocapture` — passed, 1 test.
- `node --check tests/provider-ui.mjs` and `git diff --check` — passed.
- `checkProviderUI(testPage, browser, 'alpaca/PAPER')` against `npm run dev:browser` — passed, 10 observations including Order and Account Health disconnect projections.

## Remaining boundary

[#59 is closed](https://github.com/kaiqiangh/tradex/issues/59#issuecomment-5790711855) after its local acceptance evidence was published on `dev@1af7217`. S17 parent Spec #56 remains open: its real Alpaca Paper sandbox sequence (`submit → query → observe → cancel if open → verify`) has not been run. FR-015 and AC-009 therefore remain `IMPLEMENTED_UNVERIFIED`; S17 and the wider FR-033/UX-004 scope must not be marked verified from these fixture results. No S33 full-application regression is claimed here.
