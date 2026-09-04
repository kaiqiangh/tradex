# TradeX High-Fidelity Interactive Prototype — v1.0 RevC

Standalone, no-build clickable prototype aligned to:

- `../docs/TradeX_PRD_v1.0_RevC.md`
- `../docs/TradeX_UI_Prototype_Spec_v1.0_RevC.md`
- `../docs/TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md`
- `../docs/TradeX_Prototype_QA_Report_v1.0_RevC.md`

This is a **product/design and engineering-handoff prototype**, not a production trading client. All market/account/order/model behavior is fixture data. It does not contact a broker, exchange, market-data service, Codex App Server, CLIProxyAPI, or LLM provider.

## Run

```bash
cd prototype
python3 -m http.server 8080
```

Open `http://localhost:8080`.

Opening `index.html` directly also works in most browsers.

## Revision C product model

TradeX no longer treats Paper and Live as Agent Modes.

**Agent Mode** describes the task:

- Ask
- Research
- Backtest
- Trade

**Execution Context** describes the selected execution/account environment:

- None / Read-only
- Local Paper
- Alpaca Paper
- Trading 212 Demo / Live
- Binance Testnet / Live
- Bitget Demo / Live

Selecting `Trade` never grants authority. A Live transaction additionally requires the exact Live account to be ARMED, deterministic validation, an immutable proposal, transaction-specific approval, capacity reservation, and pre-execution revalidation.

## Primary review flows

1. **Onboarding** — Workspace → Providers → Model → Risk defaults → Ready.
2. **LLM setup** — CLIProxyAPI/ChatGPT OAuth + DeepSeek; model discovery/health; automatic cross-provider fallback OFF by default.
3. **Ask** — read-only lightweight analysis with no execution action.
4. **Research** — plan/tool/result/artifact/Turn provenance.
5. **Backtest** — deterministic result manifest plus Sharpe, Sortino, drawdown, Profit Factor and Turnover.
6. **Trade context** — select a Paper/Demo/Testnet/Live account independently from Agent Mode.
7. **Account-scoped Live arming** — arm one Live account; switch accounts and verify arming is not inherited; use Disable All.
8. **Live limit approval** — immutable proposal ID/hash, risk policy version, complete market-data provenance, reservation and one-time approval.
9. **Market-order safety** — expected notional plus maximum authorized spend.
10. **Reservation conflict** — Available / Reserved / Effective Available and deterministic rejection.
11. **Risk-policy invalidation** — any relevant policy change invalidates approval; a weakening additionally disarms the affected Live account.
12. **Ambiguous submission** — `UNKNOWN_RECONCILING`, frozen reservation, account unhealthy/disarmed, evidence-based Manual Resolution.
13. **Broker rejection / cancellation** — acknowledgement is not a fill; cancel remains approval-gated.
14. **Non-live execution** — Local Paper, Alpaca Paper, Trading 212 Demo, Binance Testnet and Bitget Demo remain visibly non-live through lifecycle.
15. **Markets** — full provenance, market closed/halted/corporate-action fixtures.
16. **Portfolio** — EUR normalization with FX/stablecoin route, source, timestamp/freshness and depeg-quality fixture.
17. **Settings** — Providers & Models / Risk & Limits / Data & Storage / Account Health / Appearance / About.
18. **Workspace portability** — local Export / Workspace Import-Restore; no cloud Share-link feature.
19. **Recovery** — auth, stream, rate, clock/freshness, LLM unavailable/quota/OAuth, startup/sleep reconciliation.
20. **Accessibility baseline** — visible focus, modal ARIA, live-region toast, reduced-motion CSS, narrow navigation with More.

## Safety invariants demonstrated

- Live arming is **account-scoped**, not a global Boolean.
- Switching account, Agent Mode, or model/provider does not inherit or increase trading authority.
- Every Live order/cancel requires transaction-specific approval.
- Approved proposal identity is immutable; material edits create a new proposal.
- Market data used for Live approval exposes source/provider timestamp/TradeX receive timestamp/venue/entitlement/freshness.
- `APPROVED → RESERVED → SUBMITTING` is explicit.
- Ambiguous submission never uses blind retry or blind reservation release.
- Cross-provider LLM automatic fallback is OFF by default and requires explicit opt-in.
- Dangerous broker permissions such as withdrawal/transfer/custody/margin/leverage are blocked/reviewed.
- Workspace restore leaves all Live accounts DISARMED.

## Storage terminology

Revision C uses one consistent MVP model:

- SQLite — transactional/domain state
- DuckDB — 1-minute+ OHLCV, analytics and backtests
- Filesystem — artifacts, exports and backups
- Parquet — optional for large immutable datasets in Phase 2+

Raw/tick/order-book data is not persistently subscribed or retained by default in the MVP.
