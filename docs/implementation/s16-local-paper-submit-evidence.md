# S16 Local Paper Submit Evidence

- Scope: issue #53 full-fill Local Paper Proposal submission.
- Final implementation SHA: `df242b7627a0ae653b166a0e32e5bd411932d0b6`.
- Evidence boundary: this proves the Rust-backed browser/dispatcher path and the Local Paper storage boundary. It does not prove native Tauri/Keychain behavior; final isolation and desktop evidence remain in #55.

## Automated checks

All commands ran on `dev` at the SHA above:

- `cargo fmt --all --check` — PASS
- `cargo check -q --workspace --all-targets --all-features` — PASS
- `RUST_TEST_THREADS=1 cargo test --workspace --all-targets --all-features` — PASS (116 unit/integration tests; ignored native/provider tests remain explicitly ignored)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS
- `RUST_TEST_THREADS=1 npm run check` — PASS (schema, Vite build, 6 TypeScript unit tests, Rust workspace tests, 201 requirement traceability checks)
- `node --check tests/order-draft-ui.mjs` — PASS
- `git diff --check` — PASS

The Rust submit tests also cover full-fill idempotency/reopen, insufficient cash/position, foreign proposal rejection, duplicate key behavior, quote currency/freshness bounds, `prepare_provider(paper.order.submit) == None`, and unchanged gateway state.

## Rust-backed browser run

Command: `env -u TRADEX_PORTFOLIO_FIXTURE npm run dev:browser`.

Helper: `checkLocalPaperSubmitUI` from `tests/order-draft-ui.mjs`, run through the CUA browser bridge after opening the temporary workspace.

- Temporary workspace ID: `737f2d54-b598-4f67-947f-a18726d60493`
- Temporary workspace path: `/private/var/folders/pz/jpgkm5cd7bn8vj_klvtmvf700000gn/T/tradex-browser-SiNNRv/workspace`
- Observed full-fill result: Local Paper proposal submitted through the Rust dispatcher; UI rendered deterministic fill, quote, proposal hash, and `TRADEX_SIMULATION` / non-provider-truth disclosure.
- Observed portfolio result: refresh retained Local Paper provenance and `Live risk: Blocked` with `TRADEX_SIMULATION_NOT_LIVE`.
- Observed negative result: quantity `1001` at limit `100` produced the insufficient simulation cash alert and no `FILLED` status.
- Responsive result: no horizontal overflow at 1280px, 768px, or 390px.
- Browser console errors: `0`.

The integration bridge uses a temporary SQLite workspace and the Rust IPC binary; it does not use OAuth, DeepSeek, broker credentials, Keychain, network provider I/O, or the Live Gateway for this Local Paper path.

## Remaining scope

#53 covers the deterministic full-fill vertical slice and can be closed after review. Partial/resting/rejected/cancelled outcomes remain #54. Final native/browser isolation, ARD/traceability closeout, accessibility and complete evidence remain #55; parent #51 stays open until both tickets are complete.
