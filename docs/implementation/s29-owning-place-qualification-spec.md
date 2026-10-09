# S29.8 — Owning Binance Spot Live PLACE qualification from exact capacity and interval inputs

Parent: [Verify trusted Binance Spot Live execution](https://github.com/kaiqiangh/tradex/issues/121). Starting implementation/review baseline `dev@dd1d5c91020fb3c12b62e5cded5614760d8df11a`. [中文](s29-owning-place-qualification-spec_zh.md). Planning only; implementation and acceptance have not started.

## Problem Statement

Three read-only source slices now expose Binance Spot's exact-account capacity inventory, its declared ORDERS interval definitions and counts, and the immutable Proposal's execution rules. They are shown as evidence, but the owning Live PLACE path still derives capacity from the saved account balance's provider-declared available/committed figures alone. Nothing in the owning gate consumes the declared open-order inventory, the symbol-scoped exposure or the interval quota, so a user cannot tell whether an order would actually be admitted right now, and a Prepare can still succeed while the venue's own order-rate window is exhausted or while the capacity evidence is only partially complete. Reading the evidence pages does not change what the owning gate decides, which makes the delivered inputs decorative rather than authoritative.

## Solution

Make the existing owning Live PLACE qualification for an ordinary Binance Spot Live account consume exactly the delivered capacity and interval-quota evidence, with the same exactness and fail-closed discipline as the read-only slices. The user sees one precise, current statement of how much of their reviewed intent the venue would admit, which declared facts bound it, and which single blocker stops it; Prepare refuses when that statement is not currently available. Nothing here grants Arm, consent, dispatch or any new execution authority.

## User Stories

1. As a user, I want my Binance Spot Live capacity qualification derived from the venue's own declared figures, so that my own arithmetic can never invent headroom.
2. As a user, I want a Free/Locked balance read distinguished from the declared open-order inventory, so that order locks are not subtracted twice.
3. As a user, I want the qualification bound to the exact immutable Proposal, account, symbol, rule material and source generation, so that another intent's evidence cannot qualify mine.
4. As a user, I want incomplete or partially classified open-order coverage treated as unavailable, so that a truncated inventory never looks like free capacity.
5. As a user, I want the symbol-scoped open buy quantity used only for exposure completeness, so that it is never silently re-used as available balance.
6. As a user, I want the declared ORDERS interval limit and current count compared exactly, so that a decimal or integer representation never rounds an exhausted window into an available one.
7. As a user, I want an exhausted or unknown interval window to block Prepare, so that a venue-side rate limit is never discovered by a rejected order.
8. As a user, I want the interval counts left exactly as the provider reported them, so that no local decrement, reset or window-rollover assumption fabricates a free slot.
9. As a user, I want the local time association of those counters kept explicitly uncertain, so that an inferred instant is never shown as a provider counter timestamp.
10. As a user, I want the single binding blocker named when headroom is zero or unknown, so that I know what to fix instead of guessing.
11. As a user, I want stale, failed, retired or version-changed capacity and interval evidence to fail the qualification closed, so that yesterday's headroom cannot authorize today's order.
12. As a user, I want my own in-flight reservations to remain TradeX's business and the venue-declared figures to remain the venue's, so that a provider read never releases a local reservation and a local reservation never rewrites provider truth.
13. As a user, I want concurrent identical intents to be isolated per account, so that one attempt cannot consume another's headroom.
14. As a user, I want the existing percentage and notional risk checks, the exact stated-price checks and the PRICE_RANGE explanation preserved, so that this slice cannot weaken anything already enforced.
15. As a user, I want the owning qualification to reject exactly the same intent that the immutable approval would authorize, so that approval and admission cannot disagree.
16. As a user, I want the captured and pre-arm review to show the qualification that was actually relied on, without renewing it, so that a saved assessment stays honest.
17. As a user, I want a later get, refresh or reopen to be unable to renew a captured qualification, so that history cannot become fresh authority.
18. As a user, I want the exact capacity unit (QUOTE for buys, BASE for sells) stated with the numbers, so that USDT is never read as USD.
19. As a user, I want keyboard and 1280/768/390 review to keep the blocker as visible as the number, so that financial consent stays deliberate.
20. As a reviewer, I want genuine public RED/GREEN, exact-source checks, the ordinary build and independent serial Standards then Spec evidence, so that neither a label nor a fixture-only projection establishes this slice.

## Implementation Decisions

- Extend the existing owning Live PLACE path (risk evaluation → immutable approval → Prepare → authenticated child Gateway) for ordinary Binance Spot Live accounts only. No parallel authority model, no second capacity projection, no new account or source configuration, and no renderer-supplied amount, symbol, count, limit or qualification.
- The owning qualification consumes the already-delivered exact-account capacity evidence and interval-quota evidence by their bound identity (workspace, Proposal, Proposal hash, account, instrument, base/quote asset, rule material version, source generation and state version). A mismatch in any of them retires the evidence instead of being reconciled by assumption.
- Provider-declared `free` already excludes order locks. The declared open-order inventory is therefore used to prove coverage completeness and to cross-check the declared locked figure, never subtracted a second time from `free`. A disagreement between the two declared sources is unavailable, not a silent preference for either.
- The declared ORDERS interval limit and count are compared exactly with the existing decimal helpers. Remaining admission slots are derived, never stored; the counters are never decremented, reset or rolled forward locally, and a window boundary is never assumed to have passed.
- The interval counters carry no provider timestamp. The existing conservative local association remains the only time statement, and an association that cannot be shown to remain inside the original window makes the quota not evaluable.
- Any incomplete classification, incomplete order coverage, incomplete list coverage, unresolved obligation, failed read, non-atomic collection, missing provider time, stale evidence or version change fails the owning qualification closed with `RISK_EVIDENCE_UNAVAILABLE` semantics rather than a smaller number.
- TradeX's own reservation ledger stays authoritative for concurrent local attempts and provider-declared evidence stays authoritative for what the venue already holds. Neither may rewrite the other; the existing idempotent reservation release, disarm and policy-change behaviour is preserved unchanged.
- The existing capacity freshness window, response/rule/field/decimal/slot bounds, fixed ordinary public hosts, redirect refusal, IP budget and cooldown are reused without relaxation. No probe order, no source mutation, no HTTP weight reset and no Control Plane lock held across provider I/O.
- The capacity projection gains explicit availability and committed sources for this path so the UI can name the declared origin of each number, and the single binding blocker is a typed reason rather than free text. Existing providers keep their current sources unchanged.
- The owning qualification stays execution-unqualified even when every number is current: it is not a fill promise, not an admission guarantee for a future taker phase, and not a substitute for fees, required FX, authenticated preflight or private-stream readiness. Those remain separate owning work.
- Actual current, captured and pre-arm React views explain the qualification, its declared origin, its blocker and its future uncertainty with explicit recovery, preserving the frozen capture and the no-refresh-in-capture rule. Keyboard 1280/768/390 and the paired English/Chinese financial, wire and evidence contracts remain required. Prototype code remains unchanged.

## Testing Decisions

Use the existing public account/source/draft/Proposal/rules/capacity/interval/risk/approval/prepare flow and actual React, with only the external HTTP/vault producers fake. Record genuine public RED before GREEN for each new owned behaviour.

Verify, at the public seam: complete and current declared capacity accepting exactly the available amount; one unit beyond it rejected; incomplete classification, incomplete order/list coverage and unresolved obligations failing closed; a locked figure that contradicts the declared open-order inventory failing closed; BUY using QUOTE and SELL using BASE with the stated unit; an exhausted interval window blocking Prepare; a remaining slot of exactly one accepting one attempt and rejecting the second; unknown, malformed or non-integer counts and limits failing closed; an interval association that cannot be shown to remain in its original window making the quota not evaluable; per-account isolation under concurrent prepares; provider-declared evidence never releasing a local reservation; pending, failure, binding change, rule-material change, source-generation change, reopen, stale and newer-owner results retiring the qualification; a later get or refresh never renewing a captured qualification; and every positive case still lacking Arm, consent, dispatch or any new authority.

Replay the actual current, captured and pre-arm views plus the representative rejection and recovery paths at keyboard 1280/768/390 without overflow.

Require the current full and affected rule/capacity/interval/Hot/Gateway suites, generated IPC and paired traceability, the ordinary desktop build and pin, independent serial Standards then Spec review, and exact source/evidence/remote/tracker handoff. Native, provider, financial, physical, prototype and main evidence remain separately labelled and unclaimed.

## Out of Scope

Real Binance fee and genuinely required execution-grade FX inputs, reference-price streaming, immediate authenticated execution preflight, execution-expiry and private-stream lifecycle implementation, immutable financial intent changes beyond consuming the existing approval, actual orders/probes/Arm/consent beyond the existing approved guards, new accounts or source configuration, rights grants, S28/Testnet/Bitget scope changes, the physical S27 sleep waiver, prototype fixes and the main merge. These remain later owning work, not waived.

## Further Notes

Primary contracts rechecked 2026-10-09: [rate limits and order-count limits](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md#rate-limits), [current open orders](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md#current-open-orders-user_data), [account information](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md#account-information-user_data). Binance reports interval counts as original integers in the response envelope and documents `ORDERS` as a per-account order-rate limit; treating an unknown or incomplete read as available is this spec's explicit refusal, not a provider guarantee.

Actual technical dependencies are the completed exact-account capacity inputs and interval-quota inputs; the completed Proposal rule/reference slice supplies the instrument rules this qualification reports against. Full parent, map, S28, S17, physical S27 and main gates remain OPEN. Standing granularity and seam approvals apply; no repeated quiz.
