# S18 #62 Trading 212 Demo order-book evidence

**Status:** locally verified with provider and UI fixtures; #62 acceptance is complete. This does not close the S18 parent specification or establish a real Trading 212 account result.

- Branch: `dev`
- Implementation baseline: `f6a96feae975c929900f18d7b7d8e050ca82fc95`
- Implementation commit: `715abd770600bf1f0ec071e7c770700e00b2657a` (`feat(trading212): add Demo order book reads`)
- Scope: GitHub issue [#62](https://github.com/kaiqiangh/tradex/issues/62), pending/detail/history reads and cumulative order-fill summaries for a connected Trading 212 Demo account.

## Delivered behavior

- The primary Trade UI can restore a saved order book and explicitly refresh pending orders, one known pending-order detail, or one bounded history page. Requests remain on the fixed Demo host and read through the versioned Rust IPC, provider job, and SQLite projection.
- Orders remain scoped to workspace, connection, remote account, and Demo environment. TradeX versus external origin, raw and normalized provider status, exact decimal quantities and cumulative fill values, observation times, and a provider-reported currency when available are preserved. A TradeX attempt links only on an exact recorded provider order identity.
- Cursor bounds, endpoint throttles and provider reset headers, malformed values, unexpected statuses, duplicate/conflicting identities, and incomplete reads are handled without replacing the last trusted snapshot. Acknowledgements remain separate from fills; no synthetic execution rows are created.
- The UI presents saved, loading, empty, current, stale/degraded, and retry states with responsive order summaries. Currency stays attached to the provider-reported cumulative value and is not inferred or converted.

## Verification

On the implementation tree committed as `715abd770600bf1f0ec071e7c770700e00b2657a`:

- `npm run check` — passed: generated IPC contract check, TypeScript check/build, frontend tests (9/9), Rust library tests (131/131), runnable workspace integration tests, and requirements traceability.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `node --check tests/provider-ui.mjs` — passed.
- `git diff --check` — passed.
- Rust-backed browser validation with `checkProviderUI(tab, browser, 'trading212/DEMO')` — passed across nine observation groups: permission review and exact decimals; connection reuse after reload; pending and external historical orders with cumulative GBP fills and retained rate-limit state; explicit Demo submit confirmation and acknowledgement/fill separation; persisted attempt recovery without a second POST; exact order detail refresh and responsive 768/390 layouts; account controls at 768 and 390 (separate groups); and isolated local disconnect persistence.
- Serial Standards and Spec reviews completed with no remaining findings. The review fixes include carrying optional provider currency with cumulative value, presenting an expired retry deadline as `ready`, and classifying unexpected 4xx responses as invalid responses rather than provider unavailability.

The full check reports the existing Vite bundle-size warning. Repository-declared native Keychain and pinned-gateway checks are ignored by that check and are not claimed here as #62 evidence.

## Evidence boundary and next work

All provider responses and credentials used for these checks were disposable fixtures. No real Trading 212 API request, real account credential, or broker order was used. Real Demo acceptance remains a separate gate in #64 after #63 cancellation and fill-race handling. The next serial implementation item is #63. S17 parent issue #56 remains open as requested; S18 parent issue #60 remains open until its remaining child work is complete.
