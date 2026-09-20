# S09 组合与 FX 估值验收

日期：2026-09-19
开发分支：`dev`
状态：**VERIFIED（#32 垂直验收完成）**。生产 OD-006 交易级授权、后续 Live risk consumers 和 S33 全量回归仍保留在外部边界。
最终实现提交：`e645089` (`fix: remove stale portfolio account field`)
复核基线：`522bf33`；该提交之后没有 S09 运行时行为变化，`src-tauri/src/portfolio.rs` 仅将已验证的 `decimal_mul` 提升为模块可见性。
前置实现提交：`739f380` (`fix: validate portfolio account identities`)
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

The following source, protocol and frontend checks passed against implementation commit `e645089`:

```text
npm run build
npm run test:unit                 # 4 passed
node --check tests/portfolio-ui.mjs
cargo test --manifest-path src-tauri/Cargo.toml # 77 unit + 40 integration passed; 6 explicit native/gateway tests ignored
cargo fmt --all -- --check
git diff --check
python3 scripts/check_requirements.py
npm run schema:check
```

The Xcode and Apple SDK license was accepted before the runtime rerun; schema generation and Cargo compilation/tests completed successfully.

The focused portfolio checks include exact signed decimal addition/multiplication, deterministic fixture routes and USDT depeg warning, decimal P&L aggregation, empty production behavior, malformed payload rejection, workspace scoping, unavailable native currency provenance, fail-closed aggregation, output identity bounds and a before/after domain snapshot equality assertion.

## Browser evidence

The current isolated Rust-backed browser run at `e645089` used a fresh temporary workspace and verified:

- USD workspace base currency and degraded portfolio status;
- three fixture accounts (Trading 212 EUR, Binance USDT, Alpaca USD);
- canonical `equity:US:AAPL`, `crypto:BTC/USDT:spot`, `equity:US:MSFT` holdings plus an explicit `UNAVAILABLE` native balance, open orders and a fill;
- EUR → USD, USDT → USD and USD → USD routes with source, path, rate, provider timestamp, TradeX received timestamp, freshness and quality;
- visible `USDT is not USD` depeg warning and `Live risk: Blocked` reason.
- The current run measured document widths equal to the viewport at 768 and 390 pixels, with no page overflow and no `warn` or `error` console entries. It exercised Accounts → Open portfolio, canonical identities, provenance, blocked Live risk and the corrected `3867.6 USD` P&L. The viewport override was reset after verification; the temporary workspace was isolated and no user-owned account or credential was opened or modified.

The executable browser coverage remains in `tests/portfolio-ui.mjs`; it now has current Rust-backed runtime evidence rather than `RUNTIME_PENDING`.

## Evidence boundary

The fixture proves the IPC contract, value layering, rendering and fail-closed UI state. The production path remains dependent on OD-006 ECB reference-rate availability and provider adapters; no transaction-grade FX entitlement, stablecoin parity or S21+ Live risk consumer is claimed. Full cross-page QA remains assigned to S33.

The current `dev` recheck also passed `npm run check`, the integration-feature Rust suite, Clippy, schema/type/build/unit checks, `cargo fmt --all -- --check`, `git diff --check`, `node --check tests/portfolio-ui.mjs`, and `python3 scripts/check_requirements.py`; the current diff contains no S09 behavior change after `e645089`.
