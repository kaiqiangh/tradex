# TradeX High-Fidelity Interactive Prototype — v1.0 Final

Standalone, no-build clickable prototype aligned to `TradeX_PRD_v1.0_Final.md` and the final UI specification.

## Run

```bash
cd prototype
python3 -m http.server 8080
```

Open `http://localhost:8080`.

You can also open `index.html` directly in a browser, although serving it locally is recommended.

## Primary review flows

1. **Complete onboarding** — Workspace → Providers → Model → Risk defaults → Ready.
2. **Agent thread** — New Thread → Research Running → Tool failure/retry → Research Result.
3. **Context controls** — `@ Context` → Account picker → Model picker → Mode switch.
4. **Screener** — Markets → Screeners → natural-language FilterSpec → results.
5. **Equity detail** — AAPL → tabs → add to Watchlist.
6. **Crypto detail** — BTC/USDT → cross-venue context → Binance market-order preview.
7. **Accounts** — all Paper/Demo/Testnet/Live account variants → provider-specific Account Detail.
8. **Explicit live arming** — Prepare Live Order while DISARMED → Arm Live Trading → order approval.
9. **AAPL live order** — Trading 212 Live → BUY 2 AAPL → approval → submitting → fills → reconciliation.
10. **Market-order safety** — Binance Live → BTC market buy → expected/max spend → approval.
11. **Approval invalidation** — stale quote / approval expiry → fresh proposal required.
12. **Risk rejection** — deterministic `RISK_REJECTED`; agent cannot override.
13. **Broker rejection / ambiguous state** — `BROKER_REJECTED` and `UNKNOWN_RECONCILING`.
14. **Cancel order** — partial fill → cancel remaining → approval → `CANCEL_PENDING` → `CANCELLED`.
15. **Strategy** — list → editable sandbox → running → result / failed → v11-v12 compare.
16. **Artifacts** — list → detail → provenance → export.
17. **Settings** — provider configure/test, editable risk limits, workspace export/backup.
18. **Recovery** — sleep/resume, startup reconciliation, auth error, stream disconnect, rate limiting.

## Safety behavior demonstrated

- Selecting `Live` mode does **not** arm live execution.
- Preparing a live order while DISARMED requires a separate explicit **Arm Live Trading** action.
- Every live order and live cancellation still requires transaction-specific approval.
- The approved AAPL / Trading 212 proposal remains the same order throughout submission and fill monitoring.
- Risk checks run before approval and are described as running again immediately before submission.
- Stale or expired approvals cannot be reused.
- Ambiguous provider submission never triggers blind retry.
- Broker/exchange state is treated as authoritative.
- Strategy code is visually isolated from broker secrets and the Order Gateway.

## Prototype notes

This is a product-review and engineering-handoff prototype, not a production trading client. Market values and account data are illustrative fixture data.
