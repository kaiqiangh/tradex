# S27.4 Model failure isolation evidence

Ticket: [Verify that model failure does not block the trusted Live control plane](https://github.com/kaiqiangh/tradex/issues/111). Review baseline: `495fc745e91a87095cccb00041ca902d8979c89a`. Verification date: 2026-09-29. The issue resolution records the exact delivery commit. [中文](s27-4-model-failure-isolation-evidence_zh.md).

## Verified behavior

- The deterministic Control Plane matrix covers a stopped model sidecar, unauthorized gateway, port conflict, quota failure, and OAuth expiry. Each failure records the canonical error/remediation on the affected turn. Another running turn, account health and Arm state, risk policy, capability decision, default/current model selection, and an unrelated account cooldown remain unchanged at that transition.
- While inference is unavailable, an independently eligible Trading 212 Live proposal can be inspected, approved, and prepared through the main consumer. An existing Binance `UNKNOWN_RECONCILING` attempt retains its active reservation.
- Exact P0 reconciliation runs through the real ProviderJob while three Binance P3 slots remain occupied. A synthetic HTTP 429 records `INCONCLUSIVE`, never successful resolution, and preserves the unknown attempt/reservation. Existing scheduler regressions cover P1 precedence, bounded queues, maximum concurrency, and per-account cooldown isolation.
- An exact Trading 212 CANCEL intent can be reviewed, explicitly armed/approved, and prepared with inference stopped. The trusted dispatch package and fresh account/order preflight precede durable SUBMITTING and exactly one synthetic DELETE for `/api/v0/equity/orders/123456`. Its acknowledgement persists only `CANCEL_PENDING`, with no PLACE reservation. Existing real-child Gateway tests independently verify the private-channel/one-write boundary.
- With inference stopped, a provider authentication failure disarms only its Live account. A subsequent stream-health degradation disarms the second affected account; a third healthy account remains armed/current. This uses the common persisted account-health boundary; it does not claim a real Live websocket outage.
- Resume and actual SQLite reopen retain unknown reservations and leave Live accounts disarmed. Restoring gateway availability never Arms them. Automatic fallback remains OFF unless explicitly enabled through the existing command; opting in permits an eligible DeepSeek fallback without changing the default route or financial capabilities.

## UI and integration transport

`tests/s27-model-recovery-ui.mjs` runs against an isolated Rust/SQLite integration server, with temporary workspaces and synthetic Binance/Trading 212 accounts. Its seed/status helpers use the test HTTP API; its UI checker uses the Codex CUA browser binding.

Chrome verified STOPPED, UNAUTHORIZED, PORT_CONFLICT, and RUNNING recovery at 390/768/1280px. Providers & Models remediation, Reload model state, Settings navigation, Account Health, and account selection work with Enter; the document has no horizontal overflow. No application console errors were observed. Two unrelated `chrome-extension://` import errors are explicitly excluded. A forced SSE disconnect displays a recovery error; keyboard Retry connection restores authoritative workspace/model state.

During verification, independent browser subscriptions exhausted the origin's HTTP slots and left model/account snapshots at Loading. The integration browser transport now shares one SSE connection, retaining per-aggregate schema validation and routing. One subscriber closing does not stop its siblings; disconnect invalidates subscribers and allows a new connection. The native Tauri Channel path is unchanged. An eight-subscriber regression covers this boundary.

The integration-only `model.gateway.fixture` accepts only STOPPED/UNAUTHORIZED/PORT_CONFLICT/RUNNING, publishes the real stored gateway projection, and never launches/stops a model sidecar. Production builds do not expose it. The in-app browser control became unresponsive during QA; its timeout is not a product PASS. Chrome supplied the completed UI evidence.

## Verification gate

- PASS: three targeted `s27_model` tests, including the five-case fault matrix, actual synthetic cancellation, account-scoped auth/stream failure, P0 progress under load, resume/reopen, and explicit fallback policy.
- PASS: browser event transport regression and frontend typecheck.
- PASS: all four gateway states across three browser widths, keyboard interactions, no application console errors, and SSE Retry recovery.
- PASS: final `npm run check`: schema consistency, typecheck/build, 16 frontend tests, 237 Rust library tests and workspace suites (328 Rust tests passed; 36 existing tests ignored), and the 203-requirement/70-screen/13-QA inventory. The inventory is traceability, not runtime proof.
- PASS: `cargo test -p tradex --features 'integration-test order-gateway-runtime' --test order_gateway`: 21/21 isolated real-child tests.
- The earlier full run failed once in the pre-existing workspace identity/reopen test without a successful result envelope. The complete workspace suite then passed 7/7 unchanged; the subsequent full repository run passed. This is recorded as observed test instability, not erased or treated as a production defect fixed by this ticket.
- PASS: desktop feature build (`cargo check -p tradex --features 'desktop integration-test order-gateway-runtime' --bins`), Cargo/Rust formatting, and `git diff --check`.
- PASS: serial independent Standards and Spec reviews against `495fc745e91a87095cccb00041ca902d8979c89a`, with zero actionable findings on each axis. The first reviewer launch was interrupted by an account usage limit; a fresh allowed-usage check preceded its successful resumption. Neither review reran tests.
- CI: no repository workflow files or dev runs were present at verification; the checks above are local evidence, not a CI PASS.

This proves deterministic/local recovery boundaries. It does not perform real broker mutations or modify the user's configured model/account credentials. S27 parent acceptance, signed-package lifecycle, real-provider gates, and S33 remain separate.
