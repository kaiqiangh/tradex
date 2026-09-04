# TradeX UI Prototype QA Report — v1.0 RevC

**Baseline:** TradeX PRD / UI Spec / prototype Revision C  
**Prototype:** `prototype/index.html`, `prototype/styles.css`, `prototype/app.js`  
**Purpose:** source-level consistency, traceability, interaction-fixture and safety-state audit.

## 1. Executive result

**RevC source-alignment gate: PASS.** The prototype now expresses the same state model and safety semantics as the RevC PRD/UI Spec. Standalone fixture behavior is not claimed as real broker/runtime integration.

A separate browser/device accessibility and visual-regression pass is still required before treating the prototype as visually signed off.

## 2. Automated/static checks executed

| Check | Result | Evidence |
|---|---|---|
| JavaScript syntax | PASS | `node --check app.js` returns success |
| Inline action resolution | PASS | 64 referenced inline handlers; 0 unresolved |
| PRD functional-requirement parity | PASS | English and Chinese both contain FR-001–FR-080 (80 unique IDs) |
| PRD acceptance-criterion parity | PASS | English and Chinese contain the same AC-001–AC-066 set |
| Cross-cutting ID parity | PASS | NFR (19), SEC (9), DATA (8), OPS (9), UX (10), and OD (16) ID sets match |
| Prototype revision title | PASS | HTML title is `TradeX — High-Fidelity Prototype v1.0 RevC` |
| Agent Mode vocabulary | PASS | Ask / Research / Backtest / Trade |
| Execution Context separation | PASS | Separate context pill derived from selected account/environment |
| Account-scoped arming | PASS | `armedAccounts` Set and Disable All behavior are present |
| Full live market provenance | PASS | source/provider timestamp/TradeX received/venue/entitlement/age/freshness |
| LLM provider model | PASS | CLIProxyAPI ChatGPT OAuth + DeepSeek; fallback opt-in OFF by default |
| Reservation lifecycle | PASS | `RESERVED` and reservation-conflict fixture present |
| Ambiguous submission safety | PASS | Manual Resolution; no blind reservation-release action |
| Provider-schema form | PASS | provider-specific field/permission schema drives connection modal |
| Non-live variants | PASS | Local Paper, Alpaca Paper, T212 Demo, Binance Testnet, Bitget Demo |
| Storage terminology | PASS | SQLite + DuckDB + Filesystem; Parquet optional Phase 2+ |
| Accessibility source baseline | PASS | `:focus-visible`, dialog ARIA, `aria-live`, reduced-motion CSS |
| Prototype-only Share link | PASS | Removed from RevC topbar; export/import are local workspace operations |

## 3. Critical interaction-state audit

| Scenario | Expected RevC behavior | Result |
|---|---|---|
| Ask turn | Read-only analysis; no execution actions | PASS |
| Research turn | Research result + artifacts + provenance; no authority escalation | PASS |
| Trade + Live account | Trade mode alone grants no authority; exact account must be armed | PASS |
| Switch Live account | New account does not inherit prior account arming | PASS |
| Disable All | Clears every armed account | PASS |
| Live limit approval | Immutable proposal + policy + complete market snapshot + reservation data | PASS |
| Live market approval | Explicit maximum authorized spend | PASS |
| Approval expiry/stale quote | Approval becomes unusable and requires refresh/reapproval | PASS |
| Risk policy change | Approval invalidated; weakening additionally disarms affected account | PASS |
| Reservation conflict | Competing capacity is rejected using Available/Reserved/Effective values | PASS |
| Ambiguous submission | Account disarmed/unhealthy; reservation frozen; Manual Resolution required | PASS |
| Broker rejection | Distinct rejection state; acknowledgement never treated as fill | PASS |
| Non-live order | Environment label remains PAPER/DEMO/TESTNET through lifecycle | PASS |
| LLM quota/OAuth/unavailable | Recovery requires explicit action; auto cross-provider fallback stays off by default | PASS |
| Workspace restore | Archive validation excludes secrets; restored Live accounts remain DISARMED | PASS |
| Clock skew | Live authority blocked until TimeService recovers; re-arming remains explicit | PASS |
| Stablecoin depeg | Quality degraded; affected valuation/risk path is visibly blocked/degraded | PASS |
| Backtest metrics | Sortino, Profit Factor and Turnover are shown with reproducibility manifest | PASS |
| Narrow navigation | Primary 5-item nav includes More path to secondary destinations | PASS (source audit) |

## 4. Documentation consistency audit

- PRD and UI Spec use the same two-axis state model.
- Coverage Matrix is generated against the RevC FR/AC identifiers instead of reusing RevB claims.
- English and Chinese PRDs preserve the same requirement IDs and decision semantics.
- English and Chinese UI Specs describe the same screen inventory, safety invariants and storage/provider decisions.
- The prototype README labels all broker/model activity as fixture-only and uses Revision C terminology.

## 5. Open QA gates — not specification gaps

1. **Visual/browser regression:** execute desktop and narrow-width screenshots in the product browser harness.
2. **Keyboard/screen-reader QA:** verify focus order, modal focus trap/restore, labels and announcements with real assistive technology.
3. **Runtime integration QA:** once Codex App Server, CLIProxyAPI and provider adapters exist, validate the rows marked runtime-pending in the Coverage Matrix.
4. **Persistence/restart QA:** verify Thread/Turn/audit/reservation reconciliation across real process restart and sleep/resume.
5. **Real provider sandbox QA:** validate provider-specific capability/schema differences against official sandbox/demo/testnet APIs.

## 6. Release recommendation

The package is suitable as the **RevC product/design prototype baseline** and for implementation handoff. It should not be represented as a production execution system until the runtime-pending acceptance criteria have been implemented and tested against real provider environments.
