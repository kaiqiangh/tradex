# S29.10 — Exact-symbol commission source and conditional fee-asset evidence

[中文](s29-symbol-commission-spec_zh.md). [Parent](https://github.com/kaiqiangh/tradex/issues/121), [specification gate](https://github.com/kaiqiangh/tradex/issues/132), [single implementation ticket](https://github.com/kaiqiangh/tradex/issues/133). Planning published; implementation NOT_STARTED. Native121→132→133 hierarchy, completed123/131 dependencies,0 open blockers.

## Problem Statement

A Binance Spot Live review currently reports account-level commission rates and UNKNOWN fee currency. These inputs omit symbol-specific tax/special commissions and discount conditions. Users cannot inspect the complete current symbol commission declaration or distinguish an unconditional received-asset fee from a conditional BNB payment. This prevents later owning fee qualification; the delivered S29.9 statement must not be treated as that qualification.

## Solution

Provide one explicit, read-only collection for the saved Proposal's exact connected ordinary Binance account and canonical Spot symbol. Show the complete declared commission families, discount conditions, original receipts and the conditional charging-asset branches. A valid source observation is useful evidence, never an approval or a fill-time fee promise. The source can be valid while the owning fee/required-FX review remains unavailable. Complete monetary fee bounds and genuinely required USDT execution FX remain mandatory later owning work.

## User Stories

1. As a user, I want commission terms for my exact account and symbol, so that generic account rates are not mistaken for my symbol's full fees.
2. As a user, I want all standard, tax and special maker/taker/buyer/seller rates, so that no applicable family or side is silently omitted.
3. As a user, I want original decimal strings and declared units, so that display rounding cannot reduce a fee or change its meaning.
4. As a user, I want account and symbol discount flags, asset and original discount value, so that a discount setting is visible without being treated as guaranteed payment.
5. As a user, I want BUY received-BASE and SELL received-QUOTE charging branches explained, so that USDT or quote currency is not assigned to every fee.
6. As a user, I want eligible BNB payment described as conditional on sufficient balance and conversion evidence, so that a flag alone cannot choose the fee asset.
7. As a user, I want incomplete or unknown active fee terms reported as unresolved, so that the application cannot silently zero them.
8. As a user, I want an explicit read using my existing connection, so that inspecting a saved assessment does not trigger authenticated collection.
9. As a user, I want the exact Proposal hash, source generation and account identity bound to the read, so that edits or connection changes retire its evidence.
10. As a user, I want original local start/first receipt and absent provider timestamp disclosed, so that a database response is not presented as current execution proof.
11. As a user, I want failed refresh, expiration and late completions to stay unavailable, so that an older success cannot repair changed inputs.
12. As a user, I want current, captured and pre-arm views to show the same terms relied on, so that history stays immutable after another collection or reopening.
13. As a user, I want missing or changed terms to invalidate their review binding, so that a later approval cannot rely on a prior collection by assumption.
14. As a user, I want bounded reads and understandable rate/authentication failures, so that repeated clicks cannot cause uncontrolled provider requests.
15. As a user, I want source success separated from fee/FX eligibility, so that observing terms never arms, approves, reserves or sends an order.
16. As a user, I want keyboard use and narrow-screen inspection, so that long rates, hashes and charging conditions remain readable.

## Implementation Decisions

- Reuse the accepted public React → typed Rust Control Plane → disposable SQLite/outbox → external fake vault/HTTP seam. No renderer rate, fee, asset, qualification or financial authority setter; no repeated seam/granularity quiz because the same seam and single-ticket serial granularity already have standing human approval.
- Reuse the existing connected Binance Spot Live credentials and fixed ordinary host. One user-triggered Proposal collection verifies account UID with signed account information, then reads signed `GET /api/v3/account/commission` for the exact canonical symbol. No test-order POST, account setting change, new saved host/source or credential. Reserve the shared read budget for provider time and each documented request; commission weight20. Respect the existing deadline, ownership, retry/backoff and credential guards.
- The source owns a bounded typed observation, not a replacement for capacity or account-rate projection. Bind workspace, account/remote identity, credential/connection generation, Proposal/hash, instrument/BASE/QUOTE and current rule-source/material. A current version token guards refresh; obsolete jobs never publish to changed identity. Unknown or contradictory identity fails closed.
- Preserve all twelve commission rate strings and the two discount flags, discount asset and value. Validate bounded exact nonnegative decimals, duplicate/missing fields, unknown active extensions and exact symbol. Unknown active fee semantics are retained as unresolved and cannot be treated as zero or complete.
- Report source states separately from execution eligibility. The commission API has a database data source and no supplied response observation timestamp. Preserve original local read start/receipt, monotonic expiry and material version; never invent provider time or renew a cached get/capture. The two private reads are non-atomic.
- Explain the documented received-asset branch using BUY BASE / SELL QUOTE. When discount can apply, list BNB as conditional together with the received-asset fallback; neither account free balance nor flags alone establish which asset a future fill will charge. Keep balances and conversions unqualified until their owning evidence exists.
- This ticket does not compute or qualify a numeric total fee or discounted BNB amount. The official examples use differing discount numbers; preserve the value verbatim and require a pinned interpretation before later arithmetic. Account-level max(maker,taker) stays comparison-only. SELL limit quantity×limit price is not a guaranteed maximum proceeds/fee under price improvement; later monetary bounds must account for the actual permitted execution range.
- Carry the source observation/version into current risk/approval review and captured evidence. Genuine rate/discount/source changes alter the review digest and leave full financial eligibility blocked under the existing owning gates. No standalone source PASS is promoted into `SPOT_FEE_FX_QUALIFIED` or permission.
- Keep the existing S29.9 source/refusal semantics and its historical captures. Extend the source display additively, showing the source-only boundary and both missing owning financial dependencies. Synchronize English/Chinese contracts, evidence and traceability. AC-035 remains NOT_STARTED until its full acceptance is earned.

## Testing Decisions

Use the highest accepted product seam. External fixtures change HTTP/vault responses; the Control Plane receives normal typed commands and manages its own storage. Genuine public RED before implementation must cover complete normal terms, absence/incompleteness/duplicate fields, symbol/UID mismatch, unknown active obligations, discount enabled/disabled, expiry, changed Proposal/source/connection, late completion, provider/auth/rate failures and unchanged captured/reopened evidence. Existing capacity/fee and source-lifecycle tests are prior art. Observe exact GET methods/symbol/signature budget; assert no POST/Arm/consent/reservation/dispatch. Actual React current/captured/pre-arm and keyboard1280/768/390 must retain complete disclosures without overflow or console errors. Ordinary/schema/affected/full checks, ordinary desktop build/Gateway pin and independent Standards then Spec reviews bind final source bytes and exact dev/tracker delivery. Contract-only cases do not establish public financial qualification.

## Acceptance Criteria

- [ ] AC1. The public explicit collection traverses the real typed Control Plane and external producer seam, verifies exact remote account identity and reads only the pinned ordinary account/commission GETs for the saved exact symbol using existing credentials.
- [ ] AC2. Rust/schema/TypeScript agree on a bounded source observation containing all three four-rate families, original discount declaration, exact identity, source/version, original receipts, absent provider timestamp and source-only state/quality.
- [ ] AC3. Missing, malformed, duplicate, contradictory or unknown active terms never become zero/complete; obligations remain visible, with no float comparison or rounding and no account-rate substitute.
- [ ] AC4. BUY BASE / SELL QUOTE received-asset branches and conditional BNB/fallback are disclosed correctly; flags never select a guaranteed future charging asset, and no fee/BNB arithmetic is fabricated.
- [ ] AC5. Deadlines, current-job/generation/version guards, shared read budget, authentication/backoff, bounded observations and monotonic expiry retire failures/obsolete completions without renewing original receipts.
- [ ] AC6. Current risk/review captures bind genuine fee-source changes in their digest, preserve old captures and keep financial approval/Prepare unavailable through the existing independent owning gates; source validity creates no authority or partial financial state.
- [ ] AC7. Actual React current/captured/pre-arm and keyboard1280/768/390 show complete declared origin/units/conditions and independent unresolved owning gaps, with immutable refresh/reopen history, no overflow or console errors.
- [ ] AC8. Public normal/error/incomplete/contradictory/lifecycle RED/GREEN, current checks/schema/ordinary build/pin/frozen UI and independent serial Standards then Spec review pass at the committed source bytes. Paired evidence and verified dev/tracker handoff close only this prerequisite; parent/map/full financial gates remain OPEN.

## Out of Scope

For this prerequisite only: numeric total fee or BNB-discount amount qualification, execution-grade USDT/BNB conversion and monetary upper bounds, fee-change private subscription, immediate authenticated preflight, private-stream lifecycle, actual orders/test orders/Arm/consent, new connections/rights or saved sources, prototype repair, physical Sleep/Wake and main merge. These required owning capabilities are deferred to later serial tickets, not waived from the complete goal.

## Further Notes

Part of the full S29 parent. Planning starts at dev `cb5c69a25552797e8e71d36f125a863af0565d44`, after verified S29.9 closure. Technical dependency is the delivered exact-account/Proposal rule identity and S29.9 capture/digest contract; no artificial edge to S28 or physical gates. Schedule one end-to-end implementation ticket. Specification and implementation issue states must not upgrade the parent or requirement matrix.

Primary contracts checked2026-10-10: [commission API](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md) and [commission/charging conditions](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/faqs/commission_faq.md). PRD §21.4/§38.1/AC-035 and Backend §41.49 remain authoritative product boundaries. A source observation does not establish future fees or execution freshness.
