# S09 账户组合与 FX 估值证据规范

日期：2026-09-14
前置：S02 账户连接、S06 数据源目录、S07 canonical market、S08 TimeService/market gate
状态：IMPLEMENTED_UNVERIFIED；只读组合/估值垂直切片已实现并完成隔离验证，真实 OD-006 交易级 FX、Live risk 消费和 S33 仍待后续。实现与证据见 [S09 验收](s09-portfolio-evidence.md)。

## Problem Statement

TradeX 已能在 workspace 内保存 provider account，并在账户详情显示余额、持仓和 open orders，但没有跨账户的组合快照。不同账户和资产的原币、账户币种与 workspace base currency 没有统一表示，UI 不能审阅 FX 来源、路径、provider/TradeX 双时间戳、新鲜度或 stablecoin 质量。任何把 `USDT = USD` 的隐式假设都会污染组合分析并可能误导未来 Live risk。

## Requirements

| ID | Requirement | Acceptance boundary |
|---|---|---|
| S09-R1 | Read-only cross-account portfolio snapshot | Workspace-scoped `portfolio.get` aggregates stored account observations without changing SQLite domain state, outbox, account/model/risk/thread versions or credentials. |
| S09-R2 | Canonical identity and value layers | Each position/holding retains account/connection identity, adapter-resolved canonical instrument or explicit asset identity, optional provider-supplied venue, native amount/currency, account-currency value and workspace-currency value where available. Missing provider fields remain unavailable, never zero. |
| S09-R3 | Workspace base currency | Snapshot always returns the persisted workspace `baseCurrency`; no EUR hard-code and no renderer-side conversion. |
| S09-R4 | FX provenance | Every normalized value carries source, pair/path, provider timestamp, TradeX received timestamp, freshness and quality. OD-006 is the only current public FX source; its informational/reference nature remains visible. |
| S09-R5 | Stablecoin/depeg handling | USDT is a distinct currency. A USDT route must expose its explicit path and quality/depeg warning; it cannot silently become USD or workspace currency. |
| S09-R6 | Fail-closed authority | Missing/stale/degraded FX makes dependent workspace values unavailable or degraded and returns a typed `liveRisk` block reason. S09 never approves, reserves, submits or changes account readiness. |
| S09-R7 | Provider truth boundaries | Balances, positions, orders and fills are labelled provider observations with observed time/health. Unsupported fields (including fills when an adapter has none) are explicit unavailable states. |
| S09-R8 | Portfolio UI | Accounts context exposes a Portfolio view with totals, account/asset rows and an inspectable `FXProvenancePanel`; status uses text, semantic labels, keyboard access, `aria-live`, and 768/390 responsive layouts. |
| S09-R9 | Deterministic fixture seam | Integration/browser mode may set `TRADEX_PORTFOLIO_FIXTURE`; synthetic EUR/USD/USDT rows are visibly fixture-labelled, never claim provider truth, and never run in normal desktop Control Plane. |

## Implementation Plan

1. Add typed IPC `PortfolioQuery`, `PortfolioSnapshot`, bounded `PortfolioAccount`, `PortfolioHolding`, `PortfolioOrder`, `PortfolioFill`, `PortfolioValue` and `FxProvenance`/freshness/quality enums. Rows carry observed time and account health; provider symbols are normalized to canonical IDs at the adapter boundary. Generate schema and TypeScript types; reject unknown fields, invalid decimal strings, control characters, oversized arrays and foreign workspace IDs.
2. Add a read-only `portfolio.get` dispatcher path. It reads the current workspace and account projections plus OD-006 source observation and TimeService status. Production maps unavailable/stale account or FX fields to explicit typed states and rejects persisted observations that would exceed the snapshot's 256-account, 512-row or 128-route bounds, or an output identity field's length, with `PROVIDER_DATA_INCOMPLETE`. An integration-only fixture path supplies bounded balances, positions, open orders, fills, P&L and a USDT→USD→workspace route without writing storage.
3. Implement decimal-safe string aggregation for values that share a currency. Cross-currency values are produced only from a validated FX route; no binary floating point or implicit stablecoin parity. `liveRisk` remains blocked unless every dependent route is trusted and fresh.
4. Add a Portfolio subview reachable from Accounts (Portfolio is context/account-driven and not a new permanent primary navigation item). Render base currency, totals, account/position rows and FX/stablecoin provenance with unavailable/degraded states and a deterministic quality warning action.
5. Add Rust unit/integration tests for exact aggregation, canonical identity, workspace isolation, malformed/unknown payloads, source/freshness/depeg gates, no-mutation, fixture/non-fixture separation and schema drift. Add browser/Rust-backed checks for the Accounts→Portfolio path and 1280/768/390 overflow/console behavior.

## IPC Contract

| Command | Input | Output | Mutation |
|---|---|---|---|
| `portfolio.get` | `{ workspaceId }` | `PortfolioSnapshot` | none |

The output must keep native/account/workspace value layers separate. `FxProvenance` is attached to every conversion and the snapshot carries a summary status plus `liveRisk` eligibility/reason. Missing fields such as fills, cost basis, P&L, currency, venue or canonical identity remain explicit unavailable values; aggregation fails closed when a contributing native value cannot be normalized. The output is read-only and workspace-scoped.

## Evidence Boundary

OD-006 ECB reference rates may be probed as public metadata, but a successful HTTP response is not transaction-grade FX entitlement. Real provider positions/fills, execution-grade FX, stablecoin parity and S21+ Live risk consumption remain external/future gates. Browser fixtures prove contract and rendering only.
