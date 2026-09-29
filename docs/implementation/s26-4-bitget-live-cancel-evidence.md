# S26.4 #106 Bitget Classic Spot Live Cancellation and Fill-Race Verification

Date: 2026-09-28. Branch: `dev`. Full implementation review base: `a921ce233fc0b0c302445a79deb5b17b1173a434`. Final fixture-only review base: `e3a067e36a85e414f11d4d37c1877a3675214ff5`.

## Status

All local #106 acceptance criteria are verified with deterministic Rust/Gateway tests and the isolated Rust-backed browser path. The implementation and the final fixture-only delta passed serial Standards and Spec reviews. The issue can be closed after its resolution comment records the exact final `dev` SHA. This is synthetic local evidence; it does not claim real-account acceptance or upgrade the clickable prototype from FAILED.

## Implemented scope

- Bitget Classic Spot Live cancellation is bound to the saved ordinary order identity (`normal:{orderId}`), account identity, symbol, state, and refreshed snapshot. The isolated Gateway sends only the exact signed `POST /api/v2/spot/trade/cancel-order` after durable `SUBMITTING`; acknowledgement remains `CANCEL_PENDING` and ambiguous results are never replayed.
- Refresh reads the exact order and bounded order-scoped fills, preserves trade/fee/timestamp facts, and passes complete evidence to S26.1 settlement. Uncertain evidence retains conservative capacity. A previously invalidated attempt can be reviewed again only when durable evidence proves it stopped before dispatch.
- Schema v31 preserves prior cancellation attempts while retaining one PLACE attempt per intent. IPC validation accepts the current workspace schema version.
- The synthetic cancellation order `normal:200` now uses quantity `0.123456`, matching the Bitget symbol's six-digit quantity precision and the supported decimal boundary. The list and exact-order fixtures agree; other high-precision orders remain unchanged.

## Verification

- `npm run check` passed on the final implementation source: schema generation consistency, production build/typecheck, all 15 frontend unit tests, the Rust workspace suite, and requirement traceability (203 requirements, 70 screens, 13 QA scenarios, 23 baseline files). The build emitted the existing large-chunk advisory; checks passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --features integration-test,order-gateway-runtime bitget -- --nocapture` passed: 9 Rust tests and 2 real-child Gateway tests, including a signed exact POST for `normal:200`, rejection/lost-response handling, and no replay after restart. `bitget_live_cancel_refresh_settles_partial_then_full_fills_across_reopen` also passed for S26.1 linked-capacity settlement. No real Bitget endpoint or credentials were used.
- In a clean isolated Chrome/CDP run against the local Rust-backed fixture, the same immutable intent ID/hash survived the required post-Arm refresh. Explicit approval followed by separate Prepare/send produced exactly one Gateway write: signed `POST /api/v2/spot/trade/cancel-order` with `{orderId:"200",symbol:"BTCUSDT"}`. The durable state remained `CANCEL_PENDING` / `MAY_HAVE_SUBMITTED`; the UI did not claim cancellation.
- The Accounts flow refreshed exact order 200 and its Spot fill after navigation and reload. Persisted facts included provider status `live`, disposition `WORKING`, order quantity `0.123456`, filled quantity `0.01`, remaining quantity `0.113456`, filled quote value `123.4567`, fee `0.123 USDT`, and provider observation time. The order is external, so the UI correctly showed no linked TradeX PLACE settlement. Linked PLACE partial/full settlement and capacity recovery across reopen were verified by the Rust Control Plane test above.
- Keyboard safe-close via Enter dismissed the review without approving and restored focus to the review action. At 1280×900, 768×640, and 390×844, the dialog remained within the viewport, content did not overflow horizontally, and its actions remained reachable by dialog scrolling.
- Standards and Spec reviews passed serially: the original S26.4 diff from `a921ce233fc0b0c302445a79deb5b17b1173a434`, followed by the fixture-only delta from `e3a067e36a85e414f11d4d37c1877a3675214ff5`. The fixture delta changes no production behavior and keeps the exact refreshed-order payload consistent with the ordinary-order fixture.
- `cargo fmt --all --check`, `git diff --check`, and `python3 scripts/check_requirements.py` passed. No real Bitget request, credential, or order was used. The prototype package was not modified, and its existing FAILED evidence remains unchanged.
