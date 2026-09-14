# S07 行情浏览与 Watchlists 规范

日期：2026-09-14  
工作项：浏览真实行情并管理 Watchlists  
依赖：S01 本地工作区、S06 数据源授权目录  
状态：已规划，待实现

## Problem Statement

RevC 要求 Markets 和 Watchlists 使用与 provider symbol 无关的规范化 instrument ID，并能把行情来源、交易场所、provider 时间、TradeX 接收时间、entitlement 和 freshness 一起展示。当前应用只有空的 Markets/Watchlists 页面，研究工具也只能声明市场源不可用；没有目录、详情、搜索、列表持久化或历史数据存储。将来接入 Alpaca 时还必须保留数据市场与券商执行的边界，不能把账户连接误认为 market-data entitlement，也不能把 fixture 当成真实报价。

## Requirements

| ID | Requirement | Acceptance boundary |
|---|---|---|
| S07-R1 | Canonical instrument catalog | Search and list stable IDs such as `equity:US:AAPL` and `crypto:BTC/USDT:spot`; provider symbols stay in adapter mappings. |
| S07-R2 | Market Explorer and instrument detail | A selected row opens the exact canonical instrument; unsupported or unavailable data is explicit and retains identity. |
| S07-R3 | Market snapshot provenance | A snapshot carries source, venue, provider/received timestamps, entitlement (`REALTIME`/`DELAYED`/`UNKNOWN`) and freshness (`HEALTHY`/`STALE`/`CLOCK_UNCERTAIN`). |
| S07-R4 | Watchlist library and detail | Create, rename, delete and inspect watchlists; membership is by canonical instrument ID and survives workspace reopen. |
| S07-R5 | Add/remove instrument | Add/remove is version checked and idempotent; stale versions fail without changing the stored list. |
| S07-R6 | Census/Warm/Hot/Cold policy | The response exposes the selected tier. MVP allows Hot active detail and on-demand/coarse Census/Warm refresh; it never subscribes to the full universe. |
| S07-R7 | Historical persistence boundary | DuckDB stores 1-minute+ OHLCV/history metadata; SQLite remains the authority for workspace/watchlist state. No history is fabricated when OD-002 is blocked. |
| S07-R8 | Entitlement gate | OD-001/OD-002 must be `AVAILABLE` before a real quote/history is shown. A missing/blocked source returns a typed unavailable state with source ID and a next action. |
| S07-R9 | Accessibility/responsive behavior | Search, rows, watchlist actions, unavailable states and provenance remain keyboard reachable at 768px and 390px with no horizontal overflow. |

## Solution

### Domain and storage

Add typed Rust protocol objects:

- `Instrument` and `InstrumentProviderMapping` hold canonical identity, asset class, symbol/base/quote, venue/exchange, currency and adapter-only provider symbols.
- `MarketSnapshotProvenance` and `MarketSnapshot` carry source, venue, timestamps, entitlement and freshness. Quote fields are optional; unavailable data never gets a synthetic price.
- `MarketCatalog` returns the workspace, normalized query, catalog entries, selected tier, source ID, status and a sanitized availability reason.
- `MarketDetail` returns the exact `Instrument` plus an optional snapshot and the same source/status gate.
- `Watchlist`, `WatchlistItem` and `Watchlists` provide versioned local state. `watchlist.create`, `watchlist.rename`, `watchlist.delete`, `watchlist.add` and `watchlist.remove` use `expectedStateVersion` for mutations.

SQLite schema version 7 adds a `watchlists` projection table keyed by workspace and watchlist ID. Each projection includes ordered canonical members and a monotonic `stateVersion`; a uniqueness constraint prevents duplicate names in one workspace. Existing workspace, account and thread migrations remain transactional and back up before migration.

DuckDB lives beside `workspace.sqlite3` as `market.duckdb`. It is opened only by the market data layer and contains the bounded `ohlcv_1m`/historical tables with canonical `instrument_id`, source, venue, provider timestamp, received timestamp, entitlement and freshness columns. SQLite owns no analytical rows. When OD-002 is unavailable, the database stays empty and the UI explains why.

### Data access and gates

`market.catalog` uses the local canonical registry for search and identity. `market.get` resolves the selected ID, reads the current S06 source catalog, and calls a market adapter only when the mapped source is `AVAILABLE`. The first adapter is Alpaca Market Data with read-only snapshot/history paths; it must use the existing credential boundary and never expose key material to IPC or React. Until the user has a valid OD-001/002 entitlement, both commands return an explicit `UNAVAILABLE`/`BLOCKED_EXTERNAL` state, not fixture data.

The access tier is part of the response:

- `CENSUS`: broad coarse catalog/on-demand search;
- `WARM`: watchlist/candidate coarse refresh;
- `HOT`: the currently viewed instrument and a bounded short buffer;
- `COLD`: on-demand persisted historical data.

No command creates an always-on full-universe subscription. A market snapshot used later by an authority decision is identified by a persisted snapshot ID and includes the complete provenance block.

### UI behavior

Replace the current Markets/Watchlists placeholders with two read-only-first surfaces:

- Markets shows search, asset-class/venue chips, the result count, tier and source status. Selecting a row opens detail for that exact `instrumentId`; the detail displays quote/chart only when the source gate permits it, otherwise an unavailable card with source ID, entitlement reason and Settings/Data & Storage action.
- Watchlists shows the library, New Watchlist, rename/delete actions and the selected list’s canonical members. Add Instrument uses the Markets catalog; a duplicate add is a no-op, while a stale version prompts reload. Delete and rename require deliberate buttons and announce the result.

Rows use semantic buttons/links, full IDs remain inspectable, and status is text plus color. Dialogs follow the existing focus/inert/return pattern. Narrow layouts stack the catalog/detail and keep Watchlists reachable through the existing More navigation.

## IPC contract

| Command | Input | Output | Mutation |
|---|---|---|---|
| `market.catalog` | workspace, optional query, optional tier | `MarketCatalog` | none |
| `market.get` | workspace, canonical instrument ID, tier | `MarketDetail` | none |
| `watchlist.list` | workspace | `Watchlists` | none |
| `watchlist.create` | workspace, name | `Watchlist` | SQLite |
| `watchlist.rename` | workspace, ID, name, expected version | `Watchlist` | SQLite |
| `watchlist.delete` | workspace, ID, expected version | `Watchlists` | SQLite |
| `watchlist.add` | workspace, ID, instrument ID, expected version | `Watchlist` | SQLite |
| `watchlist.remove` | workspace, ID, instrument ID, expected version | `Watchlist` | SQLite |

All payloads use `deny_unknown_fields`, bounded strings/arrays and canonical ID validation. Unknown workspace, malformed ID, unknown watchlist and state-version conflicts fail with typed errors and no partial mutation.

## Testing Decisions

1. **Schema and contract:** generated JSON Schema/TypeScript validators cover every new command and output; unknown fields, invalid IDs, duplicate names and oversized query/member lists fail closed.
2. **Storage:** migration 6→7, workspace reopen, watchlist CRUD, ordered members, duplicate idempotence and stale-version no-mutation are tested against a temporary real SQLite workspace.
3. **Market gate:** OD-001/OD-002 blocked responses prove no quote/history is returned, source ID/reason is sanitized, and a source/catalog query cannot mutate workspace/account/model/risk/thread state. A synthetic adapter response may exercise parsing only and cannot produce an `AVAILABLE` entitlement.
4. **DuckDB boundary:** opening `market.duckdb`, writing/reading a bounded 1-minute row and reopening it proves the analytical file is separate from SQLite; blocked history leaves it empty.
5. **Browser/Rust-backed UI:** Markets search → exact detail identity, unavailable state, Watchlist create/add/remove/delete, keyboard path and 390/768 overflow/error-log checks. Browser fixtures are labelled unavailable and never claim real provider truth.
6. Run the repository checks, clippy, format, schema drift and requirement traceability after the implementation and record exact SHA and external-gate status in evidence.

## Out of Scope

- Purchasing Alpaca data plans or entering/deleting credentials; S07 consumes the existing secure provider boundary.
- Market session calendars, halts, splits/dividends and adjusted history (S08), portfolio/FX (S09), screener (S11), or any order/execution mutation.
- Full-universe persistent Warm/Census subscriptions, tick/order-book retention, Parquet interchange and commercial redistribution.
- Treating SEC/ECB probes, broker account balances or prototype fixtures as realtime/entitled market data.

## Further Notes

- Canonical identity and storage choices follow PRD §§28, 31–34, 54, 59–60; Frontend ARD §§13.1–13.2 and 18.2; Backend ARD §§7.1, 12–13 and 32; UI Spec §§4, 8, 9, 10 and §14.1.
- S06 remains the source of truth for OD-001/OD-002 status. If external entitlement is still absent during acceptance, the implementation can prove the blocked path and must record the real-data gate as `BLOCKED_EXTERNAL` rather than closing it as realtime-ready.
