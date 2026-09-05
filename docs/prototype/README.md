# TradeX Clickable Prototype — v1.0 RevC

**Documentation revision: 2026-09-05. Current prototype handoff: NOT PASS.**

This directory is a no-build HTML/CSS/JS product prototype. Markets, accounts, orders, and models are fixtures. It contacts no broker, exchange, market-data service, Codex App Server, CLIProxyAPI, or LLM. This documentation revision does not change the prototype code.

## Run from the repository root

~~~bash
python3 -m http.server 8080 --bind 127.0.0.1 --directory docs/prototype
~~~

Open http://127.0.0.1:8080 . Directly opening [index.html](./index.html) is also possible, with browser-dependent behavior. Use fake credentials only for demonstrations.

## Target versus observed evidence

- [PRD](../TradeX_PRD_v1.0_RevC.md) owns scope and authority rules.
- [UI Spec](../TradeX_UI_Prototype_Spec_v1.0_RevC.md) owns A–K screens; §14 refines the required interactions.
- [Frontend ARD](../TradeX_Frontend_ARD_v1.0_RevC.md) / [Backend ARD](../TradeX_Backend_ARD_v1.0_RevC.md) own implementation boundaries and contracts.
- [Coverage Matrix](../TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md) separates failures, partial behavior, source presence, runtime pending, and deferred scope.
- [QA Report](../TradeX_Prototype_QA_Report_v1.0_RevC.md) records current defects, reproduction steps, and post-repair acceptance.

The prototype shows some target surfaces and states; it does not successfully demonstrate every safety rule. In particular, do not treat these observed behaviors as implementation requirements:

| Review entry | Known gap | Regression case |
|---|---|---|
| Cancel from a DISARMED Live account, then Arm | Opens new-order approval | QA-01 |
| Unknown order / Manual Resolution | Fabricated fill timeline; missing evidence verification | QA-02/QA-03 |
| Closed market, clock, Disable All, sleep | Incomplete blocking and global scope | QA-04/QA-05 |
| Change model/mode, inspect history | Historical provenance follows current selectors | QA-06 |
| Edit/regenerate proposal | Incomplete draft flow and reused identity | QA-07 |
| Screener / Context / Markets | Inputs and selected identity not propagated correctly | QA-08 |
| Backtest Send / LLM recovery | Incomplete entry and recovery flows | QA-09/QA-10 |
| Keyboard dialogs / narrow layout | Background focus; missing Thread/Provider actions | QA-11/QA-12 |

## Retained product scope

Agent Mode is Ask / Research / Backtest / Trade; execution context and account environment are separate. Non-live variants include Local Paper, Alpaca Paper, Trading 212 Demo, Binance Testnet, and Bitget Demo. Live actions additionally require account arming, immutable intent, specific consent, authority checks, and authoritative reconciliation.

The storage target is SQLite + DuckDB + filesystem, with Parquet optional in Phase 2+. This fixture performs no real database, keychain, or archive I/O. PRD open decisions still govern data sources/risk parameters. Full visual, screen-reader, runtime, and real-provider acceptance remain pending.
