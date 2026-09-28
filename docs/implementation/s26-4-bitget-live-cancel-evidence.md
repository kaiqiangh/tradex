# S26.4 #106 Bitget Classic Spot Live Cancellation and Fill-Race Verification

Date: 2026-09-28. Branch: `dev`. Review base: `a921ce233fc0b0c302445a79deb5b17b1173a434`.

## Status

Implementation, deterministic Rust/Gateway verification, the full automated check, and serial Standards/Spec reviews pass. Issue #106 remains open because the browser-driven Accounts history, keyboard, and responsive interaction checks have not completed. This record does not claim full acceptance or upgrade the clickable prototype from FAILED.

## Implemented scope

- Bitget Classic Spot Live cancellation is bound to the saved ordinary order identity (`normal:{orderId}`), account identity, symbol, state, and refreshed snapshot. The isolated Gateway sends only the exact signed `POST /api/v2/spot/trade/cancel-order` after durable `SUBMITTING`; acknowledgement remains `CANCEL_PENDING` and ambiguous results are never replayed.
- Refresh reads the exact order and bounded order-scoped fills, preserves trade/fee/timestamp facts, and passes complete evidence to S26.1 settlement. Uncertain evidence retains conservative capacity. A previously invalidated attempt can be reviewed again only when durable evidence proves it stopped before dispatch.
- Schema v31 preserves prior cancellation attempts while retaining one PLACE attempt per intent. IPC validation accepts the current workspace schema version.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --features integration-test,order-gateway-runtime bitget -- --nocapture` passed on the reviewed source: 9 Rust tests and 2 real-child Gateway tests, including one signed exact POST for `normal:200`, terminal rejection/lost-response behavior, and no replay after restart. The fixture observed `CANCEL_PENDING`; it used no real Bitget endpoint or credentials.
- Control Plane coverage exercises same-order review after `INVALIDATED` / `STOPPED_BEFORE_DISPATCH`, partial then full fill settlement across reopen, exact order-book evidence, and the Gateway frame-size boundary.
- `npm run check` passed, including schema generation consistency, production build/typecheck, 15 frontend unit tests, the Rust workspace suite, and requirement traceability. One earlier run had a transient provider-cleanup restart assertion; the isolated test and the subsequent full run passed.
- Standards and Spec reviews passed against base `a921ce233fc0b0c302445a79deb5b17b1173a434`. `git diff --check` and `cargo fmt --all --check` passed.
- Browser inspection reached the isolated TradeX setup page, but its POST to `/__integration/command` received an empty HTTP 403 before the Vite integration bridge. A direct host-side request to that same local bridge succeeds. The UI therefore did not establish workspace-open, Accounts history, keyboard, or responsive behavior in this run.
- No real Bitget request, credential, or order was used. The prototype package was not modified.

## Remaining acceptance

Run the Rust-backed browser Accounts path on a browser that can POST to the local integration bridge. Confirm persisted cancellation history and observation after reload/navigation, keyboard-operable review/refresh, and 390/768/1280 px layouts. Close #106 only after those checks, serial Standards and Spec reviews, and a final exact-SHA verification record.
