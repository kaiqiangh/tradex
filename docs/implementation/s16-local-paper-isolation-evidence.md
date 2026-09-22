# S16 Local Paper isolation and end-to-end evidence

- Scope: issue #55, the S16 acceptance ticket after #53 and #54.
- Branch: `dev`.
- Implementation SHA: `306275b` (`fix: reject tampered Local Paper projections`).
- Verification SHA: `5c2575e55af23f3c94a7e70fe9e15de2dc76a828` (final `dev` commit before this evidence-only amendment).
- Boundary: Local Paper is a TradeX-owned simulation. This evidence does not claim provider Paper/Demo/Testnet, Live authority, native credential, or S33 completion.

## Acceptance evidence

| Acceptance item | Evidence | Result |
| --- | --- | --- |
| Reopen preserves account/profile/orders/fills/cash/positions/open orders/P&L/events/portfolio | `local_paper_outcomes_cancel_and_reopen_are_authoritative`, `local_paper_state_reopens_without_duplication`, and the full-fill reopen assertions in `local_paper_submit_full_fill_is_idempotent_and_reopenable` use disposable SQLite workspaces and the real dispatcher. | PASS for the Local Paper Rust path. |
| Cross-workspace, foreign/invalidated Proposal, unknown fields, stale version, duplicate submit/cancel | Paper dispatcher tests reject foreign proposals, unknown/foreign queries, stale scenario versions, consumed proposals, and duplicate mutations return the original event cursor/order. Failed mutations are checked before the storage transaction commits. | PASS for covered Rust paths. |
| Tampered fill/cash/position/P&L fail closed without accepting a partial projection | `validate_local_paper_ledger` rebuilds cash, reservations, positions, realized/unrealized P&L, exposure, equity, and base balance from canonical fills/orders. `local_paper_rejects_tampered_cash_projection` and `local_paper_rejects_tampered_fill_position_and_pnl_without_mutation` reopen corrupted SQLite projections and require `WORKSPACE_INTEGRITY_FAILED`. | PASS for covered projection corruption paths. |
| No provider, Keychain, network, model gateway, or Privileged Live Order Gateway path | `local_paper_is_provisioned_and_exposed_without_provider_io` rejects provider connect/probe/refresh/disconnect. `local_paper_submit_full_fill_is_idempotent_and_reopenable` requires `prepare_provider(paper.order.submit) == None` and unchanged model gateway state before/after submit and reopen. | PASS for the instrumented Rust boundary; native/provider integrations remain outside S16. |
| Provenance stays explicit | Rust results/events/account/portfolio assertions and the browser helper require `local-paper`, `LOCAL`, `LOCAL_PAPER`, `TRADEX_SIMULATION`, simulation disclosure, and `TRADEX_SIMULATION_NOT_LIVE`; no provider order identity is created. | PASS for the Local Paper path. |
| Contracts and traceability are synchronized | English/Chinese Backend ARD §41.17 and Frontend ARD §13.6, `requirements.csv`, `surfaces.csv`, both Coverage Matrix files, and this evidence file were updated together. S17–S30 and S33 remain pending. | PASS for the S16 documentation scope. |

## Rust checks

The new semantic integrity rule is in `src-tauri/src/storage.rs` at `validate_local_paper_state` / `validate_local_paper_ledger`. It rejects a projection whose derived values do not match the canonical order/fill ledger, while preserving the existing bounded shape, identity, sequence, and decimal checks.

The focused paper suite passed after the change:

```text
RUST_TEST_THREADS=1 cargo test -q paper_tests --workspace --all-targets --all-features
11 passed
```

The complete checks were rerun at verification SHA `5c2575e55af23f3c94a7e70fe9e15de2dc76a828`:

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | PASS |
| `cargo check -q --workspace --all-targets --all-features` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `RUST_TEST_THREADS=1 cargo test -q --workspace --all-targets --all-features` | PASS — 121 core tests passed; all remaining workspace test binaries passed with no failures |
| `npm run schema:check` | PASS — Rust / JSON Schema / TypeScript agree |
| `npm run typecheck` | PASS |
| `npm run build` | PASS — Vite build completed; existing chunk-size warning only |
| `npm run test:unit` | PASS — 6/6 |
| `node --check tests/order-draft-ui.mjs` | PASS |
| `python3 scripts/check_requirements.py` | PASS — 201 requirements, 70 screens, 12 QA scenarios, 23 baseline files |
| `git diff --check` | PASS |

The evidence-only documentation commit after this verification does not change implementation code. The closing issue comment records both this exact verification SHA and the final documentation commit SHA.

## Rust-backed browser evidence

Command: `npm run dev:browser`.

Helper: `checkLocalPaperSubmitUI` in `tests/order-draft-ui.mjs`, executed through the Rust stdio/SQLite browser bridge in a disposable workspace.

The current Local Paper browser path covers:

- deterministic full fill, quote, Proposal hash, and `TRADEX_SIMULATION` disclosure;
- quote refresh and portfolio `Live risk: Blocked · TRADEX_SIMULATION_NOT_LIVE`;
- partial fill, remaining quantity, explicit cancel confirmation, reserved-cash release, and persisted fill/event history;
- resting `ACCEPTED` order, history cancellation, rejection, and insufficient simulation cash;
- keyboard `Enter`, submit/cancel dialog focus restoration, 1280/768/390 no-overflow, and zero browser console errors.

The disposable browser workspace `39950613-fb34-4b18-8422-fdaa28f45beb` was manually exercised through the same Rust-backed route: full fill, partial fill, explicit cancellation, persisted account history, simulation provenance, and the no-provider boundary were observed. The remaining scenario branches are covered by `local_paper_outcomes_cancel_and_reopen_are_authoritative` and the helper flow above. A separate native Tauri package/signing run and the S33 cross-application regression remain pending.

### 2026-09-22 browser state and accessibility follow-up

The Rust-backed browser flow was rechecked in disposable workspace `d8dfe5f5-6107-4ba3-a5c0-83baa2cf9a0c` at `/private/var/folders/pz/jpgkm5cd7bn8vj_klvtmvf700000gn/T/tradex-browser-LdHuy6/workspace`. The browser bridge contains only fixture setup; no OAuth token, DeepSeek key, broker credential, Keychain entry, or user workspace was read or changed.

- After a full-page reload cleared query cache, a delayed `paper.get` displayed the semantic `Loading Local Paper simulation…` status; the resolved empty state exposed the labelled `Local Paper order history` region and `No Local Paper orders yet.`
- A controlled retryable `paper.get` failure displayed a semantic alert and enabled `Reload account state`; restoring the bridge and activating the button cleared the alert and restored the history region.
- Re-entering Order Drafts with an existing proposal now shows `Choose a proposal to inspect its details.` instead of leaving a disabled query in a permanent loading state. Selecting the proposal loads its details.
- Keyboard submit and cancellation dialogs expose `aria-modal="true"`; focus returns to the proposal panel after submit and the Local Paper summary after cancellation. A partial fill remained visible after the confirmed cancellation.
- At 1280, 768, and 390 CSS pixels, `document.documentElement.scrollWidth` equalled `clientWidth`. The browser console had no errors.

The direct CUA run reproduced the state assertions in `checkLocalPaperSubmitUI`; that helper remains a manually invoked browser scenario, not part of `npm run test:unit`.

The browser bridge uses a temporary SQLite tree and the Rust IPC binary. It does not read or write ChatGPT OAuth, DeepSeek keys, broker credentials, Keychain data, or a real user workspace. The model route used by the integration bridge is disposable setup data and is not Local Paper authority.

Known evidence boundary: the browser helper proves the Rust-backed desktop/browser contract. A separate native Tauri package/signing/Keychain run and the full S33 cross-application regression are not required to claim a provider or Live result and remain explicitly pending.

## Final checks and remaining scope

The final check list is:

- `cargo fmt --all --check`
- `cargo check -q --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUST_TEST_THREADS=1 cargo test -q --workspace --all-targets --all-features`
- `npm run schema:check`
- `npm run typecheck`
- `npm run build`
- `npm run test:unit`
- `node --check tests/order-draft-ui.mjs`
- `python3 scripts/check_requirements.py`
- `git diff --check`

S17–S20 provider lifecycle, S21–S30 Live authority/reconciliation, S31–S32 later operational slices, S33 full regression, S34 packaging/performance/signing, and the final `dev` → `main` PR remain open work. This evidence must not be used to close those scopes or the Wayfinder map.
