# S29.11 Binance InstrumentRules and Prepare reachability — specification proposal

> **Decision: scope approved by the user on 2026-10-19.** This document retains the decision rationale. Normative contracts are synchronized in PRD §75, Backend §41.62, Frontend §13.49, UI §14.46 and the paired owning plan. Approval authorizes implementation in the same owning ticket; it is not financial or provider acceptance.

## Problem

S29.11 #135 requires a supported-source, public-producer path through the owning RiskDecision and Prepare consumer. The current owning Binance fixture now passes the local Binance `MARKET_DATA_USE` attestation, quote freshness, fee/FX statement, notional, reserved capital, and account Arm checks. Its review is still ineligible because `INSTRUMENT_RULES` is `UNAVAILABLE`.

That result follows the current normative contracts:

- Backend ARD §41.44 says applicable dynamic position/count inputs and `PRICE_RANGE` inputs remain unavailable, and requires every applicable collected obligation to pass before `INSTRUMENT_RULES` can pass.
- Backend ARD §41.47 keeps an enforced `PRICE_RANGE` row unavailable even with a fresh reference and snapshot bounds. It is not placement authority. The current owning regression uses a LIMIT Proposal with an enforced `PRICE_RANGE` rule.
- `spot_proposal_rules.rs` maps `MAX_POSITION` and order-count rules to unavailable obligations. `spot_owning.rs` qualifies completeness and remaining declared order-rate slots from `SpotCapacityInputs` and `SpotOrderIntervalInputs`, but does not evaluate those concrete rule limits against the Proposal.
- The current exact-account fixture also omits `requireSelfTradePrevention`; the source correctly retains `ACCOUNT_SELF_TRADE_PREVENTION_REQUIREMENT_UNOBSERVED` rather than assuming false.

Therefore the current LIMIT scenario cannot reach complete `INSTRUMENT_RULES` PASS under §41.47. The exercised source also reports dynamic position/order-count constraints that §41.44 requires to stay unavailable. The static and owning observations cannot be composed into a positive Prepare review without clarifying the contract first.

## Approved decision

Permit a narrowly scoped **local Prepare eligibility** evaluation to compose the already collected, current, immutable-Proposal-bound Spot rules, capacity, and interval observations for supported ordinary Binance Spot **BASE MARKET** forms. A new order has no stated execution price, so the local stated-price placement check remains purpose-specific not-applicable under §41.47. MARKET orders remain subject to the venue's execution-time `PRICE_RANGE`; this proposal does not promise execution within a saved range or a fill. The composition can satisfy only the actual rule obligations for this unchanged Proposal; it does not turn `SpotCapacityInputs.qualification` or `SpotOrderIntervalInputs.qualification` into execution qualification.

The composed result may let the existing `INSTRUMENT_RULES` check and owning approval/Prepare eligibility pass only when all of the following hold:

1. The exact immutable Proposal, ordinary Binance Live account, source generation, rule material, instrument, side, quantity unit, policy, session, and freshness bindings match across all inputs.
2. Every applicable static order-form, quantity, notional, asset, and admission rule has complete known schema and exact units; known violations reject.
3. Applicable `MAX_POSITION`, account/exchange/symbol order-count, order-list-count, and order-rate obligations are evaluated from the existing exact-account capacity/interval observations against the actual rule limit and requested BASE quantity. The rule-specific formula, input coverage, and whether original or unfilled open-buy quantities apply must be pinned before implementation; no remaining-quantity inference or asset mapping is allowed.
4. Required account self-trade-prevention facts are present and compatible with the selected ordinary order form. Missing values remain unavailable.
5. Any missing, stale, genuinely incomplete coverage, partial/contradictory order inventory, unknown active field, or unsupported obligation remains `UNAVAILABLE`. The standing non-atomic and interval-association provenance labels retain §41.48's existing treatment: they stay visible and do not alone create a new coverage veto. A current exact violation is `REJECT`. The current immutable review digest binds every consumed input; refresh or change requires a new review.

The positive path is intentionally limited to MARKET forms with no stated execution price. An enforced LIMIT `PRICE_RANGE` remains unavailable, and its snapshot bounds remain non-authoritative. This decision does not authorize order submission or waive quote/data-use rights, fees/FX, wallet claims, permissions/health, Arm, consent, reservations, immediate authenticated Gateway preflight, lifecycle, or independent provider/source-use facts. The current synthetic Alpaca support context is excluded from acceptance; it is not evidence of the user's actual source-use facts.

## Approved rule semantics — source-backed scope clarification, 2026-10-19

The [bilingual first-party research](s29-instrument-rules-composition-research.md) pins Binance Spot documentation at `f0ee79c8ce24dbeda6645211b9504e237fddef19`. These local eligibility decisions are approved in scope; documentation facts and local restrictions remain distinct. Backend §41.62 defines the bounded wire and compatibility contract.

| Obligation | Proposed bounded local treatment |
| --- | --- |
| `MAX_POSITION` | BUY only: compare `free BASE + locked BASE + open BUY quantity + requested BASE` to the exact limit. Initially qualify only a complete empty all-account inventory, whose open BUY quantity is unambiguously zero. A nonempty inventory without established quantity/asset attribution, a partial fill, missing balances or contradictory coverage remains unavailable. Never infer `origQty-executedQty`, prevented quantity or cross-symbol BASE mapping. SELL does not add to the BUY-only obligation. |
| Symbol and exchange total-order counts | Use the exact account's corresponding complete scope. A standalone new MARKET contributes one: `count + 1 <= limit`. No predicted immediate-fill credit. |
| Algo, iceberg and order-list counts | The supported standalone MARKET contributes zero to these categories. Proposed conservative local policy still requires complete applicable category evidence and `count + 0 <= limit`; this stricter check is a TradeX eligibility policy, not an assertion of how the venue handles an unrelated pre-existing excess. Do not add one to every category or substitute total orders for lists. |
| `MAX_NUM_ORDER_AMENDS` | Purpose-specific not-applicable to a new standalone PLACE; this does not authorize amend/cancel-replace. |
| `ORDERS` interval quotas | Preserve the existing owning evaluation for every exact declared tuple: one available slot in each bucket, no local reset/decrement/rollover or expected-fill credit. An exact exhausted bucket rejects; `count > limit`, mismatched definitions, incomplete coverage or possible-boundary retirement remain unavailable. |
| Account STP and symbol default | Use the rule producer's observed account fact and actual symbol default/allowed modes. Missing account fact remains unavailable; required STP plus NONE retains the existing rejection. A default must have supported, understood semantics; membership in a string list alone cannot qualify an unknown mode. No renderer override or fabricated false value. |
| `PRICE_RANGE` | MARKET's local stated-price placement evaluation may be not-applicable; the execution-time venue rule and partial-fill/expiry exposure remain disclosed. Enforced LIMIT retains its existing unavailable result. |

The current input types retain source-only qualification and all standing limitations. A proposed consumer composes bounded per-rule outcomes at the existing RiskDecision seam and binds consumed evidence versions to the immutable review. It does not erase the standalone source projection's unavailable status or unknown obligations.

## Testing Decisions

Reuse the standing approved React → typed Rust Control Plane → disposable SQLite/outbox boundary from the complete owning-plan ticket. No additional test seam or ticket split is proposed. Fixtures replace only external HTTP/vault/WS/clock producers.

Work one public behavior at a time: complete empty-inventory MARKET BUY and SELL; exact total/category/interval limits; missing/unknown STP; nonempty ambiguous/partial/foreign inventory; stale or mismatched bindings and CAS retirement; unchanged enforced LIMIT refusal; current/captured/pre-arm 1280/768/390 disclosures. Each successful local rule evaluation must retain independent blockers; only complete owning facts may exercise positive approval/Prepare under the existing synthetic test plan. No fixture source-use fact establishes real-provider acceptance. The existing fifteen owning acceptance criteria remain on the same ticket.

## Implementation boundary

- Keep provider and vault I/O in the existing explicit refresh producers. Risk evaluation and `get` remain local-only.
- Add one bounded rule-specific composition at the existing owning RiskDecision seam; reuse the already captured observations and single approval blocker. Do not introduce renderer authority fields, another projection, provider mutations, test-order endpoints, or relaxed freshness.
- Preserve original limitations (`atomic: false`, no provider snapshot timestamp, uncertain interval-window association) as visible provenance. Passing local Prepare eligibility is not a fill, venue-admission, or execution guarantee.
- Synchronize PRD, Backend ARD English/Chinese, Frontend ARD, UI specification, and owning-plan contract before runtime implementation. Keep enforced LIMIT `PRICE_RANGE` refusal unless separately resolved.
- Add public-seam RED/GREEN coverage for complete MARKET BUY/SELL, each applicable rule family, known rejection, missing self-trade facts, stale/mismatched evidence, partial fills, cross-scope counts, CAS retirement, and Prepare refusing any incomplete case. Use fixtures only for external producers; do not count fixture source-use facts as real acceptance.

## Alternative

Retain current §41.44/§41.47 behavior and revise #135's positive AC6/AC10 objective to an explicitly unavailable/refusal-only result for these ordinary Binance forms. Do not weaken the runtime gate or claim the current positive producer-to-Prepare requirement is complete.

## Decision recorded

The user explicitly replied “批准，继续” (approved, continue). Synchronize the paired contracts and implement the MARKET-only scope above within the existing owning ticket. All #135 criteria and financial/provider gates remain independent; no acceptance checkbox changes follow from this decision.

## References

- [Backend ARD §41.44 and §41.47](../TradeX_Backend_ARD_v1.0_RevC.md)
- [Backend ARD §41.45 and §41.48](../TradeX_Backend_ARD_v1.0_RevC.md)
- [`spot_proposal_rules.rs`](../../src-tauri/src/spot_proposal_rules.rs)
- [`spot_owning.rs`](../../src-tauri/src/spot_owning.rs)
- [`spot_capacity.rs`](../../src-tauri/src/spot_capacity.rs)
- [`spot_order_intervals.rs`](../../src-tauri/src/spot_order_intervals.rs)
- [S29.11 owning plan](s29-owning-financial-plan-spec.md)
