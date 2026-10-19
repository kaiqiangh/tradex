# Binance local-use / fresh-quote integration check — 2026-10-19

Added the public Hot-source integration scenario `local_use_confirmation_precedes_collection_and_only_qualifies_its_bound_market_gate` under #135's sequential owning-plan work. Also extended `owning_bnb_fee_estimate_uses_continuous_market_and_compares_the_complete_discounted_branch` to use the same pre-collection attestation and check it in the owning fee-model decision.

The disposable workspace selects the ordinary Binance Spot source, records an explicit test-user local-use declaration before acquiring the quote lease, then reads a fresh continuous BTC/USDT book through a loopback-only peer. A synthetic account is connected through the public Control Plane fixture. The resulting RiskDecision proves:

- `MARKET_DATA_USE=PASS` with `DATA_USE_USER_ATTESTED_LOCAL_ONLY`;
- `QUOTE_FRESHNESS=PASS` for the quote collected under the attested source generation;
- quote `dataUseRights=UNVERIFIED`, with the reason explicitly retaining provider entitlement as `UNVERIFIED`;
- the complete decision is not `ALLOWED`, review is ineligible, captured review contains no raw book, and the account remains `DISARMED`.

Both focused commands passed 1/1. In the existing owning model, `SPOT_FEE_FX`, `MARKET_DATA_USE`, `ORDER_NOTIONAL`, and `RESERVED_CAPITAL` pass together; `ACCOUNT_ARMING` remains `UNAVAILABLE`, so the full decision is still unavailable. That path uses a synthetic Alpaca source-support input, which is excluded from acceptance. The complete `npm run test:binance-hot` runner passed 61/61 serial isolated scenarios. Rust emitted the repository's existing dead-code warnings. There was no failing RED before these tests: they add regressions for the already implemented #150 policy and do not claim a production fix.

Only loopback HTTP/WebSocket peers and synthetic account fixtures were used. No real provider/account read, external provider request, provider mutation, approval, Arm, consent, Prepare, dispatch, order, or financial action occurred. This proves the local-use gate and fresh-quote gate can independently pass together; it does not prove a complete positive owning financial statement, eligible approval, AC6/AC10, or Prepare. AC statuses remain unchanged.
