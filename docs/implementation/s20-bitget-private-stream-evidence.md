# S20 #76 Bitget private-stream scope and verification evidence

**Status:** #76 is closed on the unsupported-for-current-adapter branch. This does not complete Bitget Demo acceptance (#73) or Bitget Live execution (S30).

- Scope: [#76 — 接收 Bitget Demo 私有流并恢复账户状态](https://github.com/kaiqiangh/tradex/issues/76)
- Baseline: `94ce8f2085c50baa24f6c35dfb14b2144a53bf73`
- Implementation: `3cdfbfcb6420df6515e226709575da12709757b7`
- Evidence date: 2026-09-25
- User boundary: use an ordinary Bitget Live account; do not create/use a Demo account or Demo API key. Current verification uses only local synthetic provider responses and sends no Bitget request or order write.

## Official documentation decision

Bitget's [Classic-to-UTA upgrade guide](https://www.bitget.com/docs/classic/uta-api-upgrade-guide) distinguishes Classic API v2 from UTA v3. Its private-channel matrix lists Classic v2 order updates for `USDT-FUTURES` and account updates for `SPOT`, but does not list a Classic Spot order-update channel. UTA v3 lists unified `order` and `account` topics and requires a different subscription structure after migrating to UTA.

Bitget's [UTA Demo WebSocket documentation](https://www.bitget.com/docs/uta/demo-trading/websocket) gives v3 Demo WebSocket endpoints and requires a Demo API key. The separate [Classic Demo REST documentation](https://www.bitget.com/zh-CN/docs/classic/demo-trading/rest-api) requires a Demo API key and the `paptrading: 1` request header. Neither document establishes an order-and-account private stream for the current TradeX Classic Spot v2 adapter. This decision is scoped to the current adapter; it does not claim Bitget globally lacks private WebSocket support.

TradeX therefore adds no private-stream worker for this Classic Spot v2 connection and does not fabricate stream health. Account/order observations update through explicit signed REST connection/refresh requests. The account detail displays `Private stream unavailable · REST reconciliation`, projects `privateStream` as `NOT_CONFIGURED`, and keeps reconciliation status distinct (`NOT_RUN` until an actual reconciliation is implemented and completed).

## Implementation and verification

- Rust account projection now explains the REST-only path; the Bitget Live fixture asserts `privateStream == NOT_CONFIGURED` and the limitation text.
- The Rust-backed browser fixture showed the Live account as `CONNECTED`, `READ ONLY`, and `DISARMED`, with the REST-only notice and `NOT_CONFIGURED` stream status. At 1280, 768, and 390 px, the page had no horizontal overflow.
- Serial code review: Standards PASS; Spec PASS. No findings requiring changes.
- `npm run check` — PASS: schema agreement, production build, 10 frontend unit tests, 142 Rust unit tests, provider integration tests including 15 Bitget passes (one native Keychain test ignored), and requirements traceability (203 requirements, 70 screens, 13 QA scenarios, 23 baseline files).
- `cargo fmt --all -- --check`, `node --check tests/provider-ui.mjs`, and `git diff --check` — PASS.

The default TradeX workspace has no Bitget connection. Real Bitget Live account reads remain pending a secure ordinary Live connection. No Demo account, Demo credential, real provider request, or Live write was used. The Demo lifecycle parent #73 and S30 remain open; this local result must not be represented as provider authentication or Demo lifecycle acceptance.
