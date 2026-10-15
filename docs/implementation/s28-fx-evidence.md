# S28 required routes and read-only FX evidence

Ticket: [Read required FX routes and Alpaca rate evidence](https://github.com/kaiqiangh/tradex/issues/120). Review baseline: `dev@d7ea48d1e5a5c2a5f95cc3937b02aa0202bc53d4`. Paired report: [中文](s28-fx-evidence_zh.md).

## Delivery boundary

This slice derives actual currency requirements and supplies bounded read-only FX context. It does not authorize conversion or financial execution. Current implementation/native/UI verification is complete, including balance routes and original captured review time. Independent serial Standards PASS0hard/1advisory and Spec PASS0confirmed. Source/evidence delivery verified on dev and ticket120 closed; initial/token-remediation records stay historical. [Acceptance audit](evidence/s28-fx/acceptance-audit.md) maps all12 acceptance criteria; [source checkpoint](evidence/s28-fx/implementation-checkpoint.json) records exact file hashes and current limits.

The explicit saved Alpaca PAPER selection has versioned Save/Refresh/Disconnect/reload and metadata-only reopen. Save makes no provider read; disconnect preserves the borrowed account and key. Actual workspace, account, observed balance unit, monetary consumer and immutable intent determine route needs. BALANCE_WORKSPACE preserves known wallet units without inferring unknown primary fiat currency; the shared Portfolio validator accepts2–16uppercase/digit units, including unsupported USDT/OP/1INCH. Identity needs no rate; unknown currency remains unknown; USDT is distinct from USD. Spot BUY uses an observed matching quote asset only for its funding unit; SELL does not acquire a BUY funding route. Local Paper and protective CANCEL retain their existing semantics.

Only required EURUSD/USDEUR pairs reach the fixed authenticated latest-rate GET. Original numeric tokens, exact decimals, bid/ask direction, independent provider mid and original provider/receipt times are preserved. Source/account/credential/workspace/session/trusted-clock/sequence/material changes or independently stale provider/receipt age retire eligibility. Raw token shape validation prevents arbitrary_precision objects from masquerading as numbers; no f64 round trip supplies financial values.

Settings, Portfolio and Trade expose current requirements, selected source, original rates/time/quality and independent blockers. Pre-arm and immutable approval review display the original capture datetime and retain it with captured evidence after expiry, without polling it into new consent. A read observation cannot establish transaction-grade FX, broker funding/conversion fees or complete monetary inputs; required unsupported FX remains unavailable in risk/Prepare/dispatch.

## Verification

- Genuine public-seam RED/GREEN slices cover source lifecycle, requirements, exact producer, identity, immutable intent, legacy portfolio fixture isolation, captured consent/material, Spot funding, SELL independence, original-token rejection and actual wallet routes/supported read. All three historical Spec P2 failures and their real reproductions are retained.
- Current `npm run check`:17Node/386ordinaryRust, generated schema/build and traceability PASS. Integration:425Rust PASS. Scoped Gateway:23PASS. Formatting/whitespace PASS.39existing ignored tests remain unverified. The invalid combined full-library Gateway-feature invocation remains a failed historical attempt, separate from valid scoped gates.
- Real Rust-backed UI verifies Settings/captured review at390/768/1280px, keyboard and unsaved controls, maximum64character bid/ask, actual proposal reverse route, frozen review after expiry, external-fixture403/error/Reload and metadata-only reopen. No console warnings/errors, Arm, financial approval or provider mutation. Fixture UI is not hosted acceptance.
- Final2026-10-07 capture-time-remediation ordinary native saved-key read returned explicit provider-access-denied classification, with no first receipt/rate and reenabled controls. HTTP403 is an inference from the reviewed mapping; no raw status/body was archived. Final ordinary main/Gateway pin, actual ownership and20 runtime-input hashes match reviewed source. Initial bundle/denial evidence is retained separately. Current capture-time-green verifies frozen original datetime with0console issues. No credentials, account identifiers, balances, holdings or provider bodies are archived.

## Remaining financial acceptance

Required transaction-grade FX and permissible usage/entitlement, exact broker costs and complete monetary inputs remain unavailable. Calendar read and known company-event/exact-account metadata reads do not establish complete actions/history adjustment, canonical identity, current halts/tradability, full key permissions, SIP or execution venue. Quote and Trading212 acceptance parents remain OPEN. S17 real Paper and S27 actual OS sleep/wake were skipped by the user and remain unverified. No main PR/merge or full-map completion follows this slice.

Source commit: `e82b65e396e91560debae914d407577b62a76db0`.

Delivered evidence: `d4a943cba31a948fae54b365bce4e91697c358bf`. [Closure receipt](evidence/s28-fx/closure-receipt.json) distinguishes final delivery from frozen pre-delivery checkpoints.
