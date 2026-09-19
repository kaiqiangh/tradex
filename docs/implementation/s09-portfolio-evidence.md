# S09 组合与 FX 估值验收

日期：2026-09-19
开发分支：`dev`
最终实现提交：`739f380` (`fix: validate portfolio account identities`)
前置 fixture/输出修复：`7e613e4` (`fix: harden S09 portfolio output and fixture totals`)
前置审查修复：`1d81509` (`fix: close S09 portfolio review findings`)
前置 provider identity 修复：`6dda93b` (`fix: normalize all provider portfolio identities`)
前置身份边界修复：`188209b` (`fix: guard portfolio identity lengths`)
前置输出上限修复：`7d754b6` (`fix: bound portfolio snapshot output`)
前置暴露修复：`dbbed1e` (`fix: exclude unavailable balances from exposure`)
前置实现修复：`2ac732fc244b8866514707187e90b7bdc04738a9` (`fix: close portfolio provenance gaps`)

## Delivered slice

- Added the versioned `portfolio.get` Control Plane command and generated Rust/JSON Schema/TypeScript contract, with the bilingual Backend ARD §41.14 contract.
- Aggregated stored account balances, positions and open orders without writing SQLite, outbox, account, model, risk or thread state. Provider fills, P&L and unsupported fields remain explicit unavailable values when an adapter does not expose them.
- Preserved native, account and workspace value layers, configured workspace base currency, decimal-safe string arithmetic, FX source/path, provider timestamp, TradeX received timestamp, freshness, quality and stablecoin warning fields.
- Normalized provider symbols to canonical instrument IDs at the adapter boundary; unknown mappings use the explicit `UNAVAILABLE` portfolio sentinel and raw provider symbols remain adapter-only. Retained per-row observed time/health, omitted fabricated venues and failed closed when any contributing FX conversion was unavailable.
- Enforced the generated snapshot limits in the production aggregator: more than 256 accounts, 512 rows or 128 FX routes returns `PROVIDER_DATA_INCOMPLETE` instead of emitting an oversized payload.
- Validated serialized account, balance, position and order identity fields for schema length/control-character bounds before emitting the snapshot.
- Added the Accounts-context Portfolio view. It is reachable from Accounts and is not a new primary navigation item. The view exposes totals, account/holding/order/fill tables, text status, `aria-live` status and inspectable FX/stablecoin provenance.
- Added an integration-only `TRADEX_PORTFOLIO_FIXTURE` seam. The seam is enabled only with the `integration-test` feature and labels all synthetic accounts as `(fixture)`; it is unavailable to a normal desktop Control Plane.

## Automated checks

The following source and frontend checks passed against implementation commit `739f380`:

```text
npm run build
npm run test:unit                 # 4 passed
node --check tests/portfolio-ui.mjs
cargo fmt --all -- --check
git diff --check
python3 scripts/check_requirements.py
```

`npm run schema:check` and focused/full Cargo tests were attempted after the implementation changes, but the local toolchain stopped before compilation assertions because the Xcode and Apple SDK license is not accepted (exit status 69). They are therefore not counted as passing checks.

The focused portfolio checks include exact signed decimal addition/multiplication, deterministic fixture routes and USDT depeg warning, decimal P&L aggregation, empty production behavior, malformed payload rejection, workspace scoping, unavailable native currency provenance, fail-closed aggregation, output identity bounds and a before/after domain snapshot equality assertion.

## Browser evidence

The earlier isolated browser spot check at `6dda93b` was run with a temporary in-app browser tab, but it is historical: later implementation commits changed fixture P&L and production identity validation. It showed:

- USD workspace base currency and degraded portfolio status;
- three fixture accounts (Trading 212 EUR, Binance USDT, Alpaca USD);
- canonical `equity:US:AAPL`, `crypto:BTC/USDT:spot`, `equity:US:MSFT` holdings plus an explicit `UNAVAILABLE` native balance, open orders and a fill;
- EUR → USD, USDT → USD and USD → USD routes with source, path, rate, provider timestamp, TradeX received timestamp, freshness and quality;
- visible `USDT is not USD` depeg warning and `Live risk: Blocked` reason.
- The historical isolated browser spot check at `6dda93b` showed the unavailable native USDT balance was excluded from the fixture exposure total (`55712 USD`). It exercised the three fixture accounts, Accounts → Open portfolio, canonical identities, provenance and responsive layout; no user-owned tab was opened or modified.

Current executable browser coverage is `tests/portfolio-ui.mjs`, including the corrected `3867.6 USD` P&L, `UNAVAILABLE` identity sentinel, provenance/depeg warning, blocked Live risk, keyboard navigation, 1280/768/390 layouts and warning/error console assertions. It remains `RUNTIME_PENDING` until the Rust integration bridge can be rebuilt after the local Xcode license is accepted; no current browser PASS is claimed.

The historical run measured document widths equal to the viewport at 1280, 768 and 390 pixels (`overflow: false`) and had no `warn` or `error` console entries. The temporary tab was closed, the viewport override reset, and the Vite/Rust bridge process stopped after verification. The user-owned browser tab was not opened or modified.

## Evidence boundary

The fixture proves the IPC contract, value layering, rendering and fail-closed UI state. The production path remains dependent on OD-006 ECB reference-rate availability and provider adapters; no transaction-grade FX entitlement, stablecoin parity or S21+ Live risk consumer is claimed. Full cross-page QA remains assigned to S33.
