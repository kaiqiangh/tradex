# S16 Local Paper outcomes and cancellation evidence

- Scope: issue #54, Local Paper partial/resting/rejected/cancelled outcomes and UI integration.
- Implementation SHA: `5861d8498d2e04b0d634fa8aea866d502bd40c7b`.
- Predecessor: `5b08bcc3871498e00dca5bd5f965cc7fff86d446`.
- Evidence boundary: this closes the Local Paper outcome slice. Native Keychain/OAuth, external provider truth, Live Gateway authority and full isolation closeout remain issue #55.

## Contract and implementation coverage

- Added typed `paper.order.cancel` and `paper.scenario.set` IPC commands, generated JSON Schema/TypeScript validators, and Backend ARD-compatible result/event vocabulary.
- The Rust engine now provides deterministic full, partial, resting, rejected and cancelled outcomes, exact decimal cash reservation/release, fills, positions, open orders, event cursors, state-version conflicts and idempotent submit/cancel replay.
- SQLite persistence rehydrates canonical orders, fills and events after reopen; scenario changes are rejected while open orders exist. Local Paper remains `local-paper` / `LOCAL` / `TRADEX_SIMULATION` and never creates provider order IDs or Live authority records.
- Order Drafts and Accounts expose scenario selection, fill/history, state text, cancellation and simulation disclosure. Submit reselects the immutable Proposal before each attempt and retains submit/cancel idempotency keys for retry semantics.

## Automated verification at `5861d84`

- `cargo fmt --all --check` — PASS
- `cargo check -q --workspace --all-targets --all-features` — PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS
- `RUST_TEST_THREADS=1 cargo test -q --workspace --all-targets --all-features` — PASS
- `npm run schema:check` — PASS
- `npm run typecheck` — PASS
- `npm run build` — PASS (existing >500 kB Vite chunk warning only)
- `npm run test:unit` — PASS (6 tests)
- `node --check tests/order-draft-ui.mjs` — PASS
- `python3 scripts/check_requirements.py` — PASS (201 requirements, 70 screens, 12 QA scenarios, 23 baseline files)
- `git diff --check` — PASS

## Rust-backed browser evidence

Command: `npm run dev:browser`.

Helper: `checkLocalPaperSubmitUI` from `tests/order-draft-ui.mjs`.

- Temporary workspace ID: `e6d067ce-bc87-481d-b6c8-167517903020`
- Temporary workspace path: `/private/var/folders/pz/jpgkm5cd7bn8vj_klvtmvf700000gn/T/tradex-browser-dBPLam/workspace`
- Observed full fill: deterministic `FILLED`, quote, Proposal hash and `TRADEX_SIMULATION` disclosure.
- Observed portfolio: Local Paper provenance and `Live risk: Blocked · TRADEX_SIMULATION_NOT_LIVE` remained visible.
- Observed partial fill: `PARTIALLY_FILLED`, `2 filled · 2 remaining`, then `CANCELLED` with the fill retained.
- Observed resting limit: `ACCEPTED`, `0 filled · 1 remaining`, then cancellation from Accounts order history.
- Observed rejection: `REJECTED`, `0 filled · 1 remaining`, with no false fill text.
- Observed insufficient cash: typed simulation-cash error and no `FILLED` result.
- Responsive/interaction: helper used keyboard Enter and found no horizontal overflow at 1280px, 768px and 390px; browser console error count was `0`.

The integration bridge used a temporary SQLite tree and the Rust IPC binary. The onboarding model route was seeded only in that disposable tree to pass the unrelated model gate; no OAuth, DeepSeek key, broker credential, Keychain or real workspace was read or written. Local Paper order/scenario/cancel calls themselves went through the real Rust dispatcher and SQLite transactions.

## Remaining boundary

Issue #54 is ready to close after the two-axis review. Issue #55 remains open for native desktop/isolation instrumentation, cross-workspace/tamper evidence aggregation, bilingual ARD/traceability closeout and the final release boundary; parent #51 and map #1 remain open until #55 is complete.
