# S29.11 — Complete owning Spot financial plan

Parent: [Verify trusted Binance Spot Live execution](https://github.com/kaiqiangh/tradex/issues/121). Published draft: [Complete the per-asset financial plan and genuinely required FX](https://github.com/kaiqiangh/tradex/issues/134), OPEN / needs-info / assigned, native child of the parent with completed dependencies133/131. Planning baseline: `dev@70ce33ec5aa0286eff82e924f281c4588e03b3be`. [中文](s29-owning-financial-plan-spec_zh.md).

**Status: specification draft, not implementation acceptance and not ready for implementation-ticket publication.** This is the next single owning item. It retains all twenty requirements in the [financial-input research handoff](s29-financial-inputs-research.md), rather than closing another always-unqualified source projection. Source-use evidence and the monetary authorization boundary below still need resolution. No implementation, source selection, rights grant, financial action or requirement-status upgrade is recorded by this document. Prototype code is unchanged.

## Problem Statement

The user can collect an exact-symbol commission declaration, but still cannot review a complete financial plan for the immutable Spot intent. The owning statement consumes historical account comparison rates, emits no monetary fee, and cannot qualify required USDT valuation. Principal capacity and settlement each currently track one asset. Consequently the complete live path lacks a consistent account/intent-bound explanation of fees, funding, workspace values and conditional BNB exposure, followed by atomic multi-asset reservations and exact settlement.

The research also exposed a financial contract ambiguity: a current estimate, an authorized allowance, a broker-enforced bound and a realized fee are different facts. A SELL limit constrains the minimum price, not maximum proceeds or fees. A separate local maximum-spend field is not an additional Binance order parameter. A spot quote is not proof of Binance's internal average BNB fee-conversion rate. Resolving these differences is part of this item, not a reason to waive the full goal or substitute permanent refusal for implementation.

## Solution

Provide one owning financial plan, derived in the trusted Control Plane from the unchanged Proposal and genuine current producers. It includes complete three-family fees and charging branches, exact BASE/QUOTE/BNB funds, the purpose-specific conversions actually consumed, native order bounds and the independently reviewed monetary authorization. The same plan supplies monetary risk, the immutable review and Prepare. Complete supported inputs must produce genuine positive financial qualification through the public product seam; incomplete or contradictory inputs produce a typed blocker without partial authority or reservation.

Prepare consumes the approval and reserves the complete native asset vector in one transaction. Gateway preflight revalidates the approved financial material before transmission. Real fills and fees settle each asset exactly once; pending and unknown work retains its capacity. This plan never grants Arm, market-data rights, account permissions, approval, dispatch or a future fill/fee guarantee on its own.

## User Stories

1. As a user, I want the exact saved account and immutable Proposal bound to every financial input, so that another account's rates or money cannot authorize mine.
2. As a user, I want standard, tax and special fees included, so that a maker/taker comparison cannot conceal other charges.
3. As a user, I want maker/taker and buyer/seller applied together correctly, so that the complete side-specific fee is visible.
4. As a user, I want ordinary limit orders to include possible taker and mixed fills, so that a limit is not described as maker-only.
5. As a user, I want the returned discount multiplier applied only to standard fees, so that tax and special fees are not discounted silently.
6. As a user, I want BUY fees expressed in received BASE and SELL fees in received QUOTE, so that the estimate retains the native asset.
7. As a user, I want the conditional BNB-payment and received-asset fallback branches retained, so that a flag does not promise the future charging asset.
8. As a user, I want total BNB sufficiency checked across all fee families, so that covering only discounted standard fees cannot select BNB payment.
9. As a user, I want original-asset amounts calculated with exact decimals, so that floating-point rounding cannot alter financial authority.
10. As a user, I want current estimate methodology and rounding/conversion uncertainty named, so that an estimate is not presented as a guaranteed future fee.
11. As a user, I want principal bounds, fee allowances and realized costs stated separately, so that I understand what my approval actually authorizes.
12. As a user, I want a SELL limit shown as a minimum price, so that estimated proceeds are not mislabeled as an upper bound.
13. As a user, I want the native submitted order parameters compared to my authorization, so that a local maximum is not represented as broker-enforced.
14. As a user, I want exact BASE, QUOTE and BNB free/locked observations and their receipts, so that missing funds do not become zero or available capacity.
15. As a user, I want broker locks and local reservations accounted for once, so that funds are neither overstated nor subtracted twice.
16. As a user, I want prefunded USDT treated as USDT funding, so that no unnecessary currency conversion or trading operation is invented.
17. As a user, I want a funding deficit to remain unavailable until I fund externally, so that a quote cannot pretend to supply wallet funds.
18. As a user, I want separate conditional BNB capacity reserved when needed, so that another thread cannot consume the same fee allowance.
19. As a user, I want received-asset fees to reduce receipts rather than duplicate principal debits, so that BUY BASE and SELL QUOTE accounting is correct.
20. As a user, I want only conversions actually consumed by this intent or enabled policy, so that unrelated portfolio currencies do not fabricate placement blockers.
21. As a user, I want the configured workspace base preserved, so that USD is not hard-coded and USDT is not treated as USD.
22. As a user, I want explicit source, direction, bid/ask, size, time, quality, cost purpose and depeg evidence, so that a numerical rate is not enough to pass live risk.
23. As a user, I want only permitted configured data sources used, so that public access does not grant market-data rights.
24. As a user, I want unsupported or denied routes named without automatic fallback, inversion or bridging, so that the application does not silently change the source or financial model.
25. As a user, I want supported complete producers to make the owning financial statement qualify positively, so that this item does not end as another permanent-unavailable display.
26. As a user, I want risk, review and Prepare to consume the same financial plan, so that displayed numbers and execution checks cannot diverge.
27. As a user, I want changed rates, balances, policy, source or intent to invalidate incompatible approval, so that old financial consent cannot authorize changed exposure.
28. As a user, I want fresh revalidation provenance retained without a receipt-only approval loop, so that unchanged financial material can be checked immediately before execution.
29. As a user, I want atomic multi-asset refusal, so that a rejected plan leaves no partially consumed approval or reservation.
30. As a user, I want exactly-once per-asset settlement and frozen unknown capacity, so that actual fees neither disappear nor release funds prematurely.
31. As a user, I want immutable current/captured/pre-arm disclosures at keyboard and narrow widths, so that history and deliberate consent remain readable.
32. As a reviewer, I want genuine public-producer positive and failure evidence, ordinary builds, serial independent review and exact dev handoff, so that a model-only fixture cannot close the financial capability.

## Implementation Decisions

### Fixed decisions from RevC and the completed research

- One owning financial plan extends the existing public Proposal/risk/review/Prepare/Gateway path. Reuse selected saved credentials and the existing external HTTP/vault/WS seam; no renderer financial inputs, authority setters, seeded positive snapshots, alternate gateway or second settlement authority.
- Consume the exact symbol commission declaration and its source/account/rule/Proposal binding. Account-level commission comparisons remain historical and retain their original meaning. Keep all twelve original strings; incomplete, duplicate, unknown-active or mismatched terms cannot qualify.
- For each role, add its side rate within each family. Compare complete maker and taker branch totals, rather than summing unrelated independently selected maxima. BUY applies the rate to modeled received BASE; SELL applies it to modeled received QUOTE. Ordinary LIMIT is not maker-only.
- Preserve the returned discount value as the standard-fee payment multiplier. Only standard is discounted; tax and special stay undiscounted. Preserve flags, asset and fallback. Unknown charging assets remain unavailable. Total BNB sufficiency concerns the sum of all three converted families.
- Keep native values exact and bounded; use existing decimal arithmetic, checked multiplication/division and outward model rounding only under the eventual explicit estimate policy. Do not infer fee rounding from quantity steps, derive a fill-count bound from LOT_SIZE or label aggregate ceiling as a guaranteed per-fill envelope.
- Explicitly separate `current fee estimate`, `authorized principal/fee allowance`, `broker-native bound` and `realized fee`. No source timestamp, precision, current price or snapshot range proves an unchanged future fee. The resolution of the authorization/rounding boundary below is mandatory before positive acceptance, rather than an invented zero or arbitrary safety factor.
- Native mapping must preserve the user's actual order type, quantity/notional and time-in-force. A fixed-BASE BUY LIMIT can bound principal QUOTE by quantity × limit. A SELL LIMIT supplies a minimum price and authorized BASE quantity, not maximum QUOTE proceeds. MARKET quantity and quoteOrderQty retain their actual broker semantics. Never change order shape automatically to manufacture a hard bound.
- Retain proposal-scoped exact-account BASE/QUOTE/BNB free and locked rows from authenticated producer reads with original receipts and generation. Validate required-row coverage and duplicates; a missing row is not a zero balance. Preserve non-atomicity and the absent snapshot timestamp instead of assigning provider transaction-update time as collection time.
- Reuse existing native principal capacity. Broker free amounts already exclude broker locks; subtract active local reservations once. A BUY with sufficient observed USDT needs USDT→USDT identity, not an actual conversion. SELL funds its submitted BASE. No Convert, transfer, withdrawal, new broker, or hidden financial operation is added.
- Received-asset fallback fees reduce the incoming BASE/QUOTE receipt, rather than creating a second outgoing principal claim. A consumed third-asset fee allowance creates its own native asset claim. Reserve all actually required outgoing claims atomically; retain branch identity and modeled/net receipt separately from debits.
- The reservation model becomes a bounded per-asset vector within the existing PLACE transaction and outbox ownership. Scope is account + asset + intent. Existing primary reservation history remains backward compatible. Consumed approval, attempt and every claim commit together, or none do. Concurrent principal and conditional fee claims share the same capacity predicate.
- Real fills/fees, not estimates, settle capacity by their exact commission asset and cumulative execution identity. Deduplicate replay and apply deltas once. Pending CANCEL and UNKNOWN_RECONCILING retain unresolved claims. Terminal broker truth releases only unused capacity. Missing/incomplete fee truth cannot invent a zero charge, early release or healthy account; realized exposure outside the modeled allowance remains visible and reconciled.
- Derive routes by consumed purpose: principal policy value, native funding identity, fallback-fee value, conditional fee conversion/value, and portfolio routes only when a selected risk policy actually consumes them. Preserve configurable workspace fiat. Required missing routes remain blockers; unrelated holdings do not add requirements.
- Widen asset/pair contracts coherently across Rust, generated schema and TypeScript. Each selected direct route or separately approved explicit multi-leg route retains its units, direction, original bid/ask and sizes, provider/first receipt times, generation, quality/depeg state, relevant cost methodology and every leg. No parity, automatic reciprocal, implicit bridge, denied-feed fallback or unobserved zero cost.
- Policy valuation and a real conversion operation are distinct. A route used only to value native funds must be labeled as a valuation, not as a funding transaction or guaranteed conversion offer. Identity requires no external rate. Actual conversion costs are required only for an operation that would actually occur; such operations are not added by this scope.
- A source profile must name the selected existing provider/dataset, permitted-use and actual entitlement evidence before it can qualify financial use. No source is selected by this draft. Coinbase public responses cannot substitute for its missing written permission. Existing Alpaca candidates are concrete candidates, not established rights/quality/freshness. A user selection or usage declaration is context, not a `VERIFIED` permission setter.
- Genuine positive owning qualification must be reachable from supported external producer responses and their legitimate source-use contract. It supplies the same native/workspace monetary inputs to existing risk checks and Prepare. Existing account/quote/rights/policy/Arm/approval/preflight guards remain independent; source-level success cannot grant them.
- Preserve financial material and collection provenance separately. Source/account/intent/policy changes and material rate/amount/condition changes invalidate incompatible consent. Fresh revalidation preserves its own immutable receipt without rewriting the old captured observation. Matching approved material is necessary, but original consent expiry still applies; reading cached history cannot renew either.
- Prepare and immediate Gateway preflight check exact identity, current generation/time/quality, the actual authorization boundary, required funds and current claims. Changed or unavailable inputs refuse atomically before a provider mutation. Later private lifecycle and separately authorized real-provider acceptance remain required.
- English PRD/UI/Backend contracts and Chinese translations change together when the draft is resolved into normative implementation decisions. Historical source-slice contracts/captures keep their as-of meanings. AC-035 and other global statuses are not upgraded by publication, model arithmetic or source observation.

### Open decision/evidence duties — not an implementation waiver

1. **Permitted configured FX source:** establish the actual use profile and applicable provider/account/dataset/jurisdiction terms, then bind an explicit selected profile and its entitlement/quality evidence. The October10 official Alpaca document library currently links Terms & Conditions that distinguish personal/non-commercial use, third-party content, distribution and U.S.-resident intent. The personal-use clause alone therefore cannot establish this user's complete applicable rights. Actual source/dataset choice and relevant account terms remain unproven. The current usage question is context gathering, not a grant of rights.
2. **Fee authorization and rounding:** choose and pin the honest current-estimate policy and its uncertainty; determine exactly which allowance the user approves, which part is enforced by native order parameters and how conditional charges outside the estimate are handled. The earlier full-goal wording that a rounded estimate can never be less than the future venue fee cannot be established from known precision alone. Do not silently weaken it, manufacture a rounding/fill-count theorem, or turn it into permanent prohibition of every useful estimate.
3. **BNB model:** pin a disclosed conservative current conversion methodology, exact source path and sufficient-funds test, while retaining the lack of internal-average equivalence. If an absolute third-asset debit cap is demanded, obtain evidence for actual enforceability instead of relabeling an estimate as that cap.

These duties do not block writing the known model, dependency and test plan. They do block marking this complete owning specification ready, publishing a supposedly ready implementation ticket, or counting positive financial acceptance before the contract is concrete. No separate source-only ticket is substituted for the owning item.

## Testing Decisions

Use the standing approved highest seam: actual React → typed Rust Control Plane → disposable SQLite/outbox, with only external HTTP/vault/WS producers fake. No further seam or granularity interview is needed. Preserve the existing source lifecycle, settlement and atomic Prepare public tests as prior art.

Record RED before GREEN, one public behavior at a time. Positive tests must create ordinary accounts/sources/drafts/Proposals through real public commands, consume genuine complete provider responses, reach the new owning qualified statement, and show its monetary-risk/Prepare consumption. A constructor test, injected financial row, manually verified rights flag or always-unqualified product path does not satisfy positive acceptance.

Cover both sides and complete maker/taker families, mixed role conservatism, multiplier values including 0/1, tax/special, disabled discount, sufficient/insufficient BNB, fallback, free/locked/local claims, missing/duplicate wallet rows, unsupported asset and source, amount/depth/time/depeg/cost failures, exact source/UID/symbol/generation/policy/intent mismatch, denied authentication, rate-limit/deadline/late completion, receipt-only versus material-change revalidation, expiry and workspace reopen. Check exact original units; no float, parity or hidden funding operation.

Observe public atomic refusal and concurrency: approval stays unconsumed when refusal precedes consumption; no partial claims/attempt/outbox state; no provider mutation. Verify per-asset fill/fee deltas, replays, terminal release, pending cancellation and unknown recovery. Actual UI current/captured/pre-arm, keyboard1280/768/390, immutable captures, no overflow or console errors are mandatory. Require generated schema, full/affected suites, ordinary desktop build/Gateway pin, independent serial Standards then Spec reviews and exact committed-source/remote/tracker evidence. Real hosted transactions, native/physical and release gates remain separate.

### Independent arithmetic oracle preparation

The pinned official FAQ's fictional SELL example contains an inconsistent printed special-rate sum: its component rates add to `0.06000000`, whereas its displayed product line prints `0.06000030`. Its worked BNB numbers also use differing displayed precision. Therefore do not copy the printed final example as a normative exact-decimal oracle. Recompute from the original declared component literals and record the chosen model rounding separately.

For its modeled received amount `17491.25 USDT`, exact pre-rounding family values are standard `0.01049475`, tax `0.040229875`, special `1049.475`; undiscounted fallback sum `1049.525724625 USDT`. With the example's multiplier `0.25` and assumed `260 USDT/BNB`, a **hypothetical model policy** that ceilings each converted family to eight decimals gives total `4.03660715 BNB`. This arithmetic was independently checked with decimal precision80 on2026-10-10. It is a planning oracle candidate, not an implemented test, selected rounding policy, current market value, native BNB precision proof or guaranteed venue charge. Genuine public TDD must still precede implementation.

## Acceptance Criteria — all pending

- [ ] AC1. Resolve and pin the permitted selected source profile, source-use/entitlement evidence and actual supported purpose-specific route contract without a rights setter or automatic fallback.
- [ ] AC2. Exact symbol/account/Proposal terms drive all role/side/family arithmetic; account comparisons never substitute.
- [ ] AC3. Pin current-estimate rounding, authorization and native-bound semantics, preserving explicit uncertainty and unsupported absolute guarantees.
- [ ] AC4. Both conditional BNB and received-asset branches have honest current monetary estimates, original units, net receipts and total-family sufficiency treatment.
- [ ] AC5. Exact BASE/QUOTE/BNB holding coverage, first receipts, free/locked amounts and local claims are consumed once with complete binding/retirement.
- [ ] AC6. Prefunded native principal and actually consumed fee claims produce one atomic account-scoped per-asset reservation vector; no hidden funding operation exists.
- [ ] AC7. Real per-asset fills/fees settle once, preserving pending/unknown safeguards and releasing only unused capacity after broker truth.
- [ ] AC8. Rust/schema/TypeScript support actual asset/pair/path/purpose/timestamp/quality/cost and backward-compatible captured/reservation records.
- [ ] AC9. Qualified genuine required valuation/fee routes preserve configurable base, explicit direction and all selected legs; identity and unavailable routes behave honestly without parity.
- [ ] AC10. Real supported external producer responses reach a positive owning financial statement and the same monetary-risk/Prepare consumer, without a seeded authority shortcut.
- [ ] AC11. Material changes invalidate incompatible consent; fresh matching revalidation retains original approval expiry and independent immutable collection provenance without receipt-only loops.
- [ ] AC12. Prepare/preflight enforce the actual authorization/funding boundary and reject changed or unavailable evidence without partial state or provider mutation.
- [ ] AC13. Genuine public TDD plus positive/error/lifecycle/concurrency/settlement scenarios and actual current/captured/pre-arm responsive UI pass.
- [ ] AC14. Paired normative contracts, complete checks, ordinary build/pin, serial independent reviews and frozen source/evidence/remote/tracker handoff pass; full parent/map/real/native/physical/release acceptance is not upgraded by this item alone.

## Twenty-requirement handoff coverage

All entries remain pending implementation; this is a completeness index, not PASS evidence.

| Research requirement | Owning acceptance |
| --- | --- |
| 1 Exact symbol/account/Proposal commission | AC2 |
| 2 Multiplier, complete families/roles/fallback | AC2, AC4 |
| 3 Exact native estimates and rounding | AC3, AC4 |
| 4 Estimate/allowance/native-bound/realized distinction | AC3, AC12 |
| 5 BNB methodology and uncertainty | AC3, AC4 |
| 6 Exact BASE/QUOTE/BNB funds | AC5 |
| 7 Principal funds and once-only deductions | AC5, AC6 |
| 8 Conditional claims and per-asset settlement | AC6, AC7 |
| 9 Actual consumed purposes | AC9 |
| 10 Prefunding, no hidden operations | AC6 |
| 11 Real asset/pair/path wire | AC8, AC9 |
| 12 Permitted-use and source entitlement | AC1 |
| 13 Directional valuation/model/depth/cost | AC3, AC9 |
| 14 Genuine positive producer/consumer | AC10 |
| 15 Material versus immutable provenance | AC11 |
| 16 Exact preflight and atomic refusal | AC12 |
| 17 Immutable responsive disclosures | AC13 |
| 18 Positive/fault/concurrency public TDD | AC13 |
| 19 Checks/build/pin/serial reviews/handoff | AC14 |
| 20 Independent whole-goal duties | AC14 |

## Out of Scope

Only for this owning item: new brokers, Convert/transfers/withdrawals, custody/leverage/derivatives, arbitrary trading instruments, automatic financial operations or real orders, license purchases, jurisdiction bypass, source-rights grants, private-stream lifecycle completion, unrelated S28/S17/physical-S27 acceptance, prototype repair and main merge. The complete S29 lifecycle, real-provider acceptance and full application goal remain mandatory subsequent serial work.

## Further Notes

Completed prerequisites are [exact-symbol commission declaration](https://github.com/kaiqiangh/tradex/issues/133), [owning fee/required-FX refusal and capture](https://github.com/kaiqiangh/tradex/issues/131), and the existing native principal/capacity/market/rule inputs actually consumed. Do not add scheduling-only blockers to S28 or physical acceptance. Publish one implementation ticket only after the open contract is resolved, retaining actual dependency edges and the standing single-ticket granularity.

Primary evidence: [pinned commission FAQ](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/faqs/commission_faq.md), [pinned REST contract](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/rest-api.md), [Alpaca document library](https://alpaca.markets/disclosures), [Alpaca Terms & Conditions](https://files.alpaca.markets/disclosures/library/TermsAndConditions.pdf), [Market Data API](https://docs.alpaca.markets/us/docs/about-market-data-api), [crypto datasets](https://docs.alpaca.markets/us/docs/real-time-crypto-pricing-data), and [Coinbase Market Data Terms](https://www.coinbase.com/legal/market_data). Checked2026-10-10; statements are planning evidence, not a legal rights determination or real financial acceptance.
