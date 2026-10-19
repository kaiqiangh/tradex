# S29.11 Binance InstrumentRules and Prepare reachability — specification proposal

> **Decision: pending.** This is a non-normative proposal. Do not change the paired contracts or runtime gate until the decision is recorded.

## Problem

S29.11 #135 requires a supported-source, public-producer path through the owning RiskDecision and Prepare consumer. The current owning Binance fixture now passes the local Binance `MARKET_DATA_USE` attestation, quote freshness, fee/FX statement, notional, reserved capital, and account Arm checks. Its review is still ineligible because `INSTRUMENT_RULES` is `UNAVAILABLE`.

That result follows the current normative contracts:

- Backend ARD §41.44 says applicable dynamic position/count inputs and `PRICE_RANGE` inputs remain unavailable, and requires every applicable collected obligation to pass before `INSTRUMENT_RULES` can pass.
- Backend ARD §41.47 keeps an enforced `PRICE_RANGE` row unavailable even with a fresh reference and snapshot bounds. It is not placement authority. The current owning regression uses a LIMIT Proposal with an enforced `PRICE_RANGE` rule.
- `spot_proposal_rules.rs` maps `MAX_POSITION` and order-count rules to unavailable obligations. `spot_owning.rs` qualifies completeness and remaining declared order-rate slots from `SpotCapacityInputs` and `SpotOrderIntervalInputs`, but does not evaluate those concrete rule limits against the Proposal.
- The current exact-account fixture also omits `requireSelfTradePrevention`; the source correctly retains `ACCOUNT_SELF_TRADE_PREVENTION_REQUIREMENT_UNOBSERVED` rather than assuming false.

Therefore the current LIMIT scenario cannot reach complete `INSTRUMENT_RULES` PASS under §41.47. The exercised source also reports dynamic position/order-count constraints that §41.44 requires to stay unavailable. The static and owning observations cannot be composed into a positive Prepare review without clarifying the contract first.

## Proposed decision

Permit a narrowly scoped **local Prepare eligibility** evaluation to compose the already collected, current, immutable-Proposal-bound Spot rules, capacity, and interval observations for supported ordinary Binance Spot **BASE MARKET** forms. A new order has no stated execution price, so `PRICE_RANGE` remains purpose-specific not-applicable under §41.47. The composition can satisfy only the actual rule obligations for this unchanged Proposal; it does not turn `SpotCapacityInputs.qualification` or `SpotOrderIntervalInputs.qualification` into execution qualification.

The composed result may let the existing `INSTRUMENT_RULES` check and owning approval/Prepare eligibility pass only when all of the following hold:

1. The exact immutable Proposal, ordinary Binance Live account, source generation, rule material, instrument, side, quantity unit, policy, session, and freshness bindings match across all inputs.
2. Every applicable static order-form, quantity, notional, asset, and admission rule has complete known schema and exact units; known violations reject.
3. Applicable `MAX_POSITION`, account/exchange/symbol order-count, order-list-count, and order-rate obligations are evaluated from the existing exact-account capacity/interval observations against the actual rule limit and requested BASE quantity. The rule-specific formula, input coverage, and whether original or unfilled open-buy quantities apply must be pinned before implementation; no remaining-quantity inference or asset mapping is allowed.
4. Required account self-trade-prevention facts are present and compatible with the selected ordinary order form. Missing values remain unavailable.
5. Any missing, stale, non-atomic coverage gap, partial/contradictory order inventory, unknown active field, or unsupported obligation remains `UNAVAILABLE`. A current exact violation is `REJECT`. The current immutable review digest binds every consumed input; refresh or change requires a new review.

The positive path is intentionally limited to MARKET forms with no stated execution price. An enforced LIMIT `PRICE_RANGE` remains unavailable, and its snapshot bounds remain non-authoritative. This decision does not authorize order submission or waive quote/data-use rights, fees/FX, wallet claims, permissions/health, Arm, consent, reservations, immediate authenticated Gateway preflight, lifecycle, or independent provider/source-use facts. The current synthetic Alpaca support context is excluded from acceptance; it is not evidence of the user's actual source-use facts.

## Implementation boundary if approved

- Keep provider and vault I/O in the existing explicit refresh producers. Risk evaluation and `get` remain local-only.
- Add one bounded rule-specific composition at the existing owning RiskDecision seam; reuse the already captured observations and single approval blocker. Do not introduce renderer authority fields, another projection, provider mutations, test-order endpoints, or relaxed freshness.
- Preserve original limitations (`atomic: false`, no provider snapshot timestamp, uncertain interval-window association) as visible provenance. Passing local Prepare eligibility is not a fill, venue-admission, or execution guarantee.
- Synchronize PRD, Backend ARD English/Chinese, Frontend ARD, UI specification, and owning-plan contract before runtime implementation. Keep enforced LIMIT `PRICE_RANGE` refusal unless separately resolved.
- Add public-seam RED/GREEN coverage for complete MARKET BUY/SELL, each applicable rule family, known rejection, missing self-trade facts, stale/mismatched evidence, partial fills, cross-scope counts, CAS retirement, and Prepare refusing any incomplete case. Use fixtures only for external producers; do not count fixture source-use facts as real acceptance.

## Alternative

Retain current §41.44/§41.47 behavior and revise #135's positive AC6/AC10 objective to an explicitly unavailable/refusal-only result for these ordinary Binance forms. Do not weaken the runtime gate or claim the current positive producer-to-Prepare requirement is complete.

## Decision required

Approve or reject the narrow MARKET-only local Prepare eligibility composition above. Approval authorizes a paired normative-contract update and implementation; it does not accept #135 or any financial/provider gate. Rejection keeps the fail-closed code and requires an owning-ticket decision on the unreachable positive criterion.

## References

- [Backend ARD §41.44 and §41.47](../TradeX_Backend_ARD_v1.0_RevC.md)
- [Backend ARD §41.45 and §41.48](../TradeX_Backend_ARD_v1.0_RevC.md)
- [`spot_proposal_rules.rs`](../../src-tauri/src/spot_proposal_rules.rs)
- [`spot_owning.rs`](../../src-tauri/src/spot_owning.rs)
- [`spot_capacity.rs`](../../src-tauri/src/spot_capacity.rs)
- [`spot_order_intervals.rs`](../../src-tauri/src/spot_order_intervals.rs)
- [S29.11 owning plan](s29-owning-financial-plan-spec.md)
