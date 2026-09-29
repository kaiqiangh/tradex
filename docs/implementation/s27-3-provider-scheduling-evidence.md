# S27.3 Provider scheduling evidence

Ticket: [Prioritize and bound Provider work queues](https://github.com/kaiqiangh/tradex/issues/110). Review baseline: `4b813a9bb83e7fb458acb69f2242887ecbf64edb`. The issue resolution records the exact accompanying implementation commit. Verification date: 2026-09-29. [中文](s27-3-provider-scheduling-evidence_zh.md).

## Delivered behavior

- Provider jobs use P0 execution reconciliation, P1 account safety refresh, P2 active monitoring, and P3 research/history. Eligible work runs by priority, then FIFO within a priority.
- Each provider has at most four active requests, with non-P0 work limited to three. Waiting work is capped at 32; P2/P3 together may occupy only 24 slots. Overflow returns retryable `PROVIDER_BACKPRESSURE` / `RATE_LIMITED` before the request is sent.
- Account cooldowns are independent. Provider/IP cooldowns, including Binance request-weight limits and accountless public sources, apply across accounts. A cooled account cannot block another account's eligible work.
- The serialized Order Gateway holds a parent scheduler permit throughout preflight/submission and returns sanitized cooldown seconds over its authenticated private channel. Busy desktop admission returns immediate backpressure. Grant eligibility is rechecked before durable SUBMITTING.
- Provider waits release the scheduler mutex and do not hold the desktop or stdio Control Plane/SQLite lock. Existing cancellation/current-job checks run outside the scheduler mutex.
- Financial transitions and exact reconciliation attempts are not coalesced. Current quote refresh is local Paper simulation; there is no remote quote/UI refresh producer to coalesce. Backend ARD §36.2 retains the producer-side coalescing/sampling requirement for a future provider-backed feed.

## Verification

- PASS: seven scheduler tests cover priority assignment, saturated P3 backpressure and P0 progress, P1 precedence, the reserved P0 slot, maximum concurrency, cooldown delay, and account cooldown isolation/cancellation.
- PASS: busy Gateway admission and authenticated cooldown validation/sharing regressions.
- PASS: exact Binance Live 429 reconciliation retains `UNKNOWN_RECONCILING` and the active reservation; limited/missing evidence never becomes successful resolution.
- PASS: `npm run check` on the final source: schema consistency, frontend typecheck/build, 15 frontend tests, 236 Rust library tests, workspace integration suites, and requirement inventory. Existing ignored tests remain ignored. The inventory covers 203 requirements, 70 screens, and 13 QA scenarios; it does not prove runtime completion.
- PASS: `cargo check -p tradex --features 'desktop integration-test order-gateway-runtime' --bins`, `cargo fmt --check`, and `git diff --check`.
- PASS: all 21 tests, including the real-child 429 single-mutation/cooldown propagation regression: `cargo test -p tradex --features 'integration-test order-gateway-runtime' --test order_gateway -- --test-threads=1`.
- PASS: independent Standards and Spec source reviews against the fixed baseline; neither has remaining findings. Reviewers did not run tests.

Sandboxed loopback fixture binds initially failed with EPERM; those checks were rerun with local-port access. An intermediate GET wrapper bypassed existing custom `get` fixtures; the final metadata hook preserves them. Concurrent default/feature Cargo runs also shared the child executable output; final suite runs are serialized. These intermediate failures are not acceptance evidence.

Only synthetic credentials, temporary workspaces, and loopback fake providers were used. No provider-hosted order was sent, native account recovery was not revalidated here, and prototype/S33/full-application completion is not claimed. The user-owned workload dashboard/data files are excluded.
