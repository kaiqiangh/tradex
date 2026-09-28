# S26.2 #104 Trading 212 Live Cancellation and Fill-Race Evidence

Date: 2026-09-28. Branch: `dev`. Review base: `bb4112dcb994ca1ab31a87df993389d46c95edb6`.

## Scope

This slice covers durable account-scoped Trading 212 Live cancellation history and the race where an order fills after the provider acknowledges cancellation. It preserves the CANCEL attempt as `CANCEL_PENDING`, stores the latest exact-order provider observation, and keeps cancellation acknowledgement separate from terminal provider truth.

## Verification

- The focused Rust protocol test passed: `order_gateway::outcome_tests::cancel_ack_requires_the_preflight_provider_status`. It accepts the exact reviewed `PARTIALLY_FILLED` preflight status and rejects a fabricated `CANCEL_PENDING` provider status.
- The real child Gateway test passed: `real_child_sends_the_exact_live_cancel_and_records_only_pending_acknowledgement`. The local loopback fixture recorded one GET of the account summary, one GET of order `9007199254740996`, and one DELETE to `/api/v0/equity/orders/9007199254740996`.
- In the Rust-backed browser fixture, the user reviewed and approved the exact partially filled order, then explicitly prepared the cancellation. TradeX retained `CANCEL_PENDING` and stated that the provider acknowledgement did not confirm cancellation.
- The fixture then reported a racing full fill. Clicking **Refresh exact order** removed the order from the open-order list while the saved cancellation history displayed `FILLED · TERMINAL`, quantity `1`, cumulative filled quantity `1`, remaining quantity `0`, and cumulative value `130`. A durable preparation read retained `CANCEL_PENDING` and the exact `FILLED` observation from `trading212.live.order-detail`. The Gateway log still contained only the original DELETE; refresh issued no provider write.
- The account-scoped history cache is invalidated after approval and preparation so a cancellation approval created in the open page appears immediately in the saved history list.
- Verification used an isolated temporary workspace, synthetic credentials, and the local fake provider only. No real Trading 212 credentials, provider requests, or orders were used. The clickable prototype was not changed and is not promoted to PASS.
