# S29.4 — Proposal static Spot rules and required references

Parent: [Verify trusted Binance Spot Live execution](https://github.com/kaiqiangh/tradex/issues/121). Source prerequisites [Exact Spot rules/admission](https://github.com/kaiqiangh/tradex/issues/123) and [Hot quote/continuous depth](https://github.com/kaiqiangh/tradex/issues/124) are CLOSED. Planning baseline `dev@2368e22122b2028e92f4c3352c2f08fe80e1344a`. [中文](s29-spot-proposal-rules-spec_zh.md).

Current single implementation ticket: [Verify Proposal static Spot rules and required reference inputs](https://github.com/kaiqiangh/tradex/issues/125), claimed by kaiqiangh, ready-for-agent. [Published specification](https://github.com/kaiqiangh/tradex/issues/121#issuecomment-6069315992). Implementation/review baseline is `2368e22122b2028e92f4c3352c2f08fe80e1344a`. Planning and native completed dependencies are recorded; implementation and acceptance are not yet performed.

## Problem Statement

Trade shows exact-account Spot metadata and genuine bid/ask, but cannot explain whether an immutable Proposal satisfies the collected filters. A supported symbol or a visible quote does not establish quantity steps, limit-price bounds, percentage/notional inputs or dynamic account limits. Users need exact per-rule results and visible missing obligations without invented execution readiness.

## Solution

Provide one complete read-only vertical slice: select an existing ordinary Binance Live Proposal, explicitly refresh only genuinely required reference inputs, evaluate supported static rules exactly, and display bound per-rule results and unresolved obligations in Trade. Negative rules reject; missing, unknown or unsupported applicable inputs remain unavailable. This advances parent stories8–11,14,17 and27; full dynamic qualification, fees/FX, authenticated preflight and financial acceptance remain required.

## User Stories

1. As a user, I want rules evaluated against my saved immutable Proposal, so that the checked amount, side, order type and venue cannot differ from my intent.
2. As a user, I want exact saved account and canonical BTC/USDT or ETH/USDT identity, so that another account's rules cannot qualify this intent.
3. As a user, I want quantity and price units retained, so that BASE and QUOTE are never substituted or USDT relabelled USD.
4. As a user, I want supported order forms checked, so that unsupported time-in-force, quantity modes or advanced forms remain blocked.
5. As a user, I want every applicable static price/quantity constraint evaluated exactly, so that rounding cannot silently change my Proposal.
6. As a user, I want disabled fields distinguished from missing or malformed fields, so that a zero value cannot disable an unrelated obligation.
7. As a user, I want side-specific percentage and notional applicability explained, so that BUY/SELL and Market/Limit use their actual constraints.
8. As a user, I want a genuine required reference collected only when necessary, so that quote midpoints or receipt times cannot impersonate a provider reference.
9. As a user, I want unavailable/error reference responses to fail closed, so that fallback is never selected to evade a refusal.
10. As a user, I want original interval and provider time shown separately from first local receipt, so that cached reads do not renew eligibility.
11. As a user, I want stale, foreign or changed-generation inputs retired, so that refreshing elsewhere cannot preserve old authority.
12. As a user, I want all rule scopes and unsupported constraints retained, so that account/exchange obligations are never dropped after symbol checks pass.
13. As a user, I want dynamic account counts, positions and execution-rule inputs shown as unresolved when absent, so that partial static success is never a complete rules PASS.
14. As a user, I want rights, quote, health, FX, fees and permission checks kept independent, so that a reference read cannot Arm or approve a trade.
15. As a user, I want read-only refresh and recovery accessible by keyboard and at narrow widths, so that blockers are visible before any consent.
16. As a user, I want saved risk/review evidence capture-labelled, so that history is not displayed as a renewed current observation.
17. As a reviewer, I want public external-protocol proof, exact-SHA checks and serial independent reviews, so that a seeded financial fixture cannot close this ticket or the parent.

## Implementation Decisions

- Reuse existing exact-account financial-source selection, runtime evidence bindings, Hot source generations, shared read scheduler/budgets, immutable Proposal and Trade risk/review surfaces. No new configuration, vault, renderer host/symbol/price/authority input or background universe scan. The read-only request identifies a stored Proposal and expected current state; backend derives all intent, account, canonical symbol and required inputs.
- Return bounded typed per-rule results: origin/scope/kind, applicable/not applicable, PASS/REJECT/UNAVAILABLE, precise reason and evidence references; include overall completeness and unresolved obligations. Distinguish static component PASS from the owning INSTRUMENT_RULES outcome. A known current violation may reject; full rules PASS requires every actually applicable obligation established. This slice does not fabricate dynamic evidence, so current dynamic/unsupported obligations continue to prevent full qualification.
- Evaluate supported ordinary forms only: BASE Market with DAY semantics, BUY-only QUOTE Market when provider permits it, and BASE Limit with exact GTC/IOC/FOK. Preserve current user intent; reject unsupported combinations, advanced flags and implicit conversions. Precision is not tick/step permission. Interpret each known active field according to its own rule; do not blindly reuse the metadata disabled marker. Exact decimal comparisons/modulo/multiplication only, no f64, rounding or Testnet writer reuse.
- Own static PRICE_FILTER, LOT_SIZE/MARKET_LOT_SIZE, applicable MIN_NOTIONAL/NOTIONAL, PERCENT_PRICE/PERCENT_PRICE_BY_SIDE and single-order MAX_ASSET amount evaluations when their complete inputs are genuinely available. Preserve each scope/origin without silent precedence or dropping unknown fields. QUOTE Market is not an invented exact BASE amount; unavailable resulting BASE remains an unresolved quantity/asset obligation. Bounded local maximum spend checks do not establish fees, funding or approval.
- Reference selection is purpose-specific and backend-derived. Recheck official current contracts before implementing each family. Non-null genuine provider reference takes its documented role; only a well-formed explicit-null response permits the documented fallback. All errors, including documented no-reference error responses, remain conservatively unavailable under the existing accepted fail-closed policy. A matching average interval is required where applicable. Zero-interval last-trade use needs an original genuine trade/provider timestamp; timestamp-less ticker/REST receipt or Hot midpoint cannot qualify it. If no sufficiently bounded authentic path exists, keep that case explicitly unavailable rather than introduce guessed data.
- Do not reinterpret the provider's rule reference as the policy's last-trade deviation input or a bid/ask. Keep separate purpose, canonical/venue/account/source/Proposal hash, workspace/session/time generation, material and original/first-receipt age. Source references remain transient; saved reviews contain approved bounded references/digests and captured outcomes, not raw tick/book caches. Reopen restores configuration/history only; repeated reads do not renew current reference age. Reference changes or retirement invalidate any bound consent under existing rules.
- Use existing bounded provider HTTP with fixed ordinary host, P3 public-IP costs and no public authenticated UID fiction, strict original types/duplicates/size/deadline/redirect/cooldown guards and sanitized failure. No global CP lock across I/O; state-conflict, cancellation/source/workspace/time changes reject late publication. Public reference collection does not transmit saved execution credentials.
- Dynamic order-count/position capacities, unresolved execution PRICE_RANGE inputs and unknown applicable constraints remain explicitly UNAVAILABLE until their owning later evidence work. Missing dynamic inputs are not NOT_APPLICABLE merely because this application requests a plain order. Protective exact CANCEL/reconciliation remain independent. Fee/FX and immediate authenticated Gateway preflight remain separate duties.
- MARKET_DATA_USE and user-specific data rights remain unavailable/unverified. No read, configuration checkbox or acknowledgement creates licensing, regional eligibility, overall financial PASS, Arm, financial approval, reservation or provider mutation. No changes to Testnet hosted scope, S28/S17/physical S27 or main. Synchronize English/Chinese normative IPC/financial/UI contracts during implementation.

## Testing Decisions

- Retain the previously approved single public React→actual typed Rust Control Plane→temporary SQLite/outbox→external fake vault/HTTP/WS seam. Start with an observable public RED before GREEN. Create accounts/sources/Proposal via existing public operations; no hidden positive financial authority or reference snapshot seed.
- External fake provider supplies actual HTTP responses; real Hot worker produces any used quote. Cover exact valid/invalid price/step/notional/side/applicability, disabled versus malformed fields, BASE/QUOTE forms, MAX_ASSET units, scoped duplicates/unknown obligations, reference value/null/error/interval/provider-time faults, precision extremes, source/account/Proposal/time changes, first receipt/expiry/reopen and late read cancellation. Successful static components may coexist with overall UNAVAILABLE; explicitly assert no overall approval/PLACE eligibility.
- Actual Trade and captured-history UI demonstrates per-rule success/rejection/missing input, explicit refresh/recovery, keyboard1280/768/390 and truthful current versus captured labels. No external financial action or licensing acceptance. Check affected rule-source/Hot/Gateway regressions, full local checks, generated IPC/bilingual manifest and ordinary production build; independent Standards then Spec against the fixed implementation baseline. Label ordinary-native/provider/financial proof separately.

## Out of Scope

Dynamic account/execution-input collection, fee/FX qualification, authenticated immediate Gateway preflight, new PLACE/private-stream lifecycle, actual provider transactions, data-rights grants, general text DLP, margin/derivatives/advanced orders, new assets/hosts, licence purchases and main merge. These remain parent work where in scope; none is waived by this prerequisite.

## Further Notes

Official sources rechecked2026-10-08: [Spot filters](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/filters.md) and [Spot REST reference/average endpoints](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md). They distinguish price-field disabling, side/Market applicability and reference value/null/error with original timestamp or average interval/closeTime. Implementation must not infer undocumented semantics or financial-use rights. Only this new ticket is planned; do not implement another slice in parallel.
