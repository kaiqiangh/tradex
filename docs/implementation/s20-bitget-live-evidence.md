# S20 #75 Bitget Live order and fill observation evidence

**Status:** Local implementation acceptance is complete with isolated provider fixtures. Real Bitget Live reads remain pending a secure ordinary Live connection in TradeX. This evidence does not complete the separate Bitget Demo gate in #73 or S30 Live execution.

- Branch: `dev`
- Review baseline: `59e3c2cb62dbd53ec67472067575e0c94e80df39`
- Implementation commit: `983e0518a82a044e967c661f03e35de8eed2ce94`
- Scope: [#75 — 查询 Bitget Live 订单与成交并刷新余额（只读）](https://github.com/kaiqiangh/tradex/issues/75)
- Contract: Backend ARD §41.29, Frontend ARD §13.16, UI Spec §14.15

## Delivered behavior

Explicit `account.refresh` for an ordinary `bitget` / `LIVE` connection reads current orders and balances together with bounded normal/TPSL history, plan-order history, and fills. It uses signed GET-only fixed-host routes, omits `paptrading: 1`, and has no Demo fallback or Live write path. Provider identities and decimals remain exact strings; currency, fill totals, remaining quantity, plan trigger outcomes, origin, and freshness are kept distinct when the provider data supports them. Duplicate, malformed, truncated, over-limit, identity-conflicting, or incomplete responses fail closed and retain the prior trusted account projection.

The Accounts detail labels the section `READ ONLY · DISARMED`, presents current and historical orders separately from recent fills, preserves the last successful observation time, and marks the snapshot `CURRENT`, `STALE`, or `DEGRADED`. Tables are keyboard reachable and remain within the viewport at the required widths. English and Chinese Backend ARD, Frontend ARD, and UI Spec sections were updated together.

The endpoint boundaries were checked against Bitget's [Classic Spot Trade API](https://www.bitget.com/docs/catalog/classic-spot-trade/classic-spot-trade) and [Get History Plan Orders API](https://www.bitget.com/api-doc/spot/plan/Get-History-Plan-Order): history-order pagination uses `orderId`, fill pagination uses `tradeId`, responses are capped at 100 records, recent records are limited to the documented 90-day window, and trader fill reads are spaced by at least one second.

## Verification

On implementation commit `983e0518a82a044e967c661f03e35de8eed2ce94`:

- `npm run check` — passed: generated schema/type agreement, TypeScript and production build, frontend unit tests, Rust workspace/integration tests, and requirement traceability (203 requirements, 70 screens, 13 QA scenarios).
- `cargo fmt --all -- --check`, `git diff --check`, `node --check tests/provider-ui.mjs`, and `npm run schema:check` — passed.
- Rust-backed browser `checkWorkspaceUI` — passed workspace reopen and reconnect paths, plus Enter-accessible navigation at 768 and 390 px.
- Rust-backed browser `checkProviderUI(..., "bitget/LIVE")` — passed exact large order/trade IDs and decimal rendering, balances/orders/fills, keyboard access, explicit refresh and disconnect behavior, and 390/768/1280 px layout checks without horizontal overflow.
- Standards review — PASS. Spec review — PASS; no actionable findings against the RevC English and Chinese contract sections.

The browser fixture used a fresh temporary SQLite workspace and synthetic provider responses. Read-only inspection of the user's default workspace account metadata found Alpaca Paper, Local Paper, and Trading 212 Live connections, with no Bitget connection. No Bitget API credential was read or used, no provider request was sent, and no Demo account or Live write was used. Connect an ordinary Bitget Live account to TradeX before claiming real provider-read verification.

The production build emitted the existing Vite bundle-size warning. The disposable native macOS Keychain credential integration test remains ignored and is not claimed as verified here.
