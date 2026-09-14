# S09 组合与 FX 估值验收

日期：2026-09-14
开发分支：`dev`
实现提交：`2ac732fc244b8866514707187e90b7bdc04738a9` (`fix: close portfolio provenance gaps`)

## Delivered slice

- Added the versioned `portfolio.get` Control Plane command and generated Rust/JSON Schema/TypeScript contract, with the bilingual Backend ARD §41.14 contract.
- Aggregated stored account balances, positions and open orders without writing SQLite, outbox, account, model, risk or thread state. Provider fills, P&L and unsupported fields remain explicit unavailable values when an adapter does not expose them.
- Preserved native, account and workspace value layers, configured workspace base currency, decimal-safe string arithmetic, FX source/path, provider timestamp, TradeX received timestamp, freshness, quality and stablecoin warning fields.
- Normalized provider symbols to canonical instrument IDs at the adapter boundary, retained per-row observed time/health, omitted fabricated venues and failed closed when any contributing FX conversion was unavailable.
- Added the Accounts-context Portfolio view. It is reachable from Accounts and is not a new primary navigation item. The view exposes totals, account/holding/order/fill tables, text status, `aria-live` status and inspectable FX/stablecoin provenance.
- Added an integration-only `TRADEX_PORTFOLIO_FIXTURE` seam. The seam is enabled only with the `integration-test` feature and labels all synthetic accounts as `(fixture)`; it is unavailable to a normal desktop Control Plane.

## Automated checks

The following checks passed during implementation and are repeated after the final docs anchor:

```text
npm run schema:check
npm run build
npm run test:unit                 # 4 passed
cargo test --workspace            # full workspace run recorded after the final docs anchor
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
python3 scripts/check_requirements.py
```

The focused portfolio checks include exact signed decimal addition/multiplication, deterministic fixture routes and USDT depeg warning, empty production behavior, malformed payload rejection, workspace scoping, unavailable native currency provenance, fail-closed aggregation and a before/after domain snapshot equality assertion.

## Browser evidence

`npm run dev:browser` was run with a temporary in-app browser tab. The tab opened an isolated temporary workspace and exercised Accounts → Open portfolio. The rendered fixture showed:

- USD workspace base currency and degraded portfolio status;
- three fixture accounts (Trading 212 EUR, Binance USDT, Alpaca USD);
- canonical `equity:US:AAPL`, `crypto:BTC/USDT:spot`, `equity:US:MSFT` holdings plus an explicit `UNAVAILABLE` native balance, open orders and a fill;
- EUR → USD, USDT → USD and USD → USD routes with source, path, rate, provider timestamp, TradeX received timestamp, freshness and quality;
- visible `USDT is not USD` depeg warning and `Live risk: Blocked` reason.

Measured document widths were equal to the viewport at 1280, 768 and 390 pixels (`overflow: false`). The browser console contained no `warn` or `error` entries. The temporary tab was closed, the viewport override reset, and the Vite/Rust bridge process stopped after verification. The user-owned browser tab was not opened or modified.

## Evidence boundary

The fixture proves the IPC contract, value layering, rendering and fail-closed UI state. The production path remains dependent on OD-006 ECB reference-rate availability and provider adapters; no transaction-grade FX entitlement, stablecoin parity or S21+ Live risk consumer is claimed. Full cross-page QA remains assigned to S33.
