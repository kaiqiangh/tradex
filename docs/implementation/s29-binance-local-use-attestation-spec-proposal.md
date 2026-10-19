# S29.11 Binance source-bound local-use attestation — specification proposal

> **Decision: approved on 2026-10-19; this remains the rationale/proposal record.** The current normative contract is in [Backend ARD §41.43](../TradeX_Backend_ARD_v1.0_RevC.md), [Frontend ARD §13.48](../TradeX_Frontend_ARD_v1.0_RevC.md), [UI §14.45](../TradeX_UI_Prototype_Spec_v1.0_RevC.md), the paired PRD source-use section, and the [S29.11 owning-plan contract](s29-owning-financial-plan-spec.md). Approval does not change runtime behavior, #135 status, or any S29.11 acceptance checkbox; implementation proceeds under a separate sequential ticket.

## Problem statement

Backend ARD §41.43 and `risk.rs` currently make `MARKET_DATA_USE` unconditionally `UNAVAILABLE` for ordinary Binance Spot Live data. S29.11 AC6 and AC10 require a genuine owning plan to reach the same RiskDecision and Prepare consumer. The existing Alpaca attestation in Backend §41.60 is deliberately limited to local `POLICY_VALUATION`; it cannot qualify Binance market use or Prepare. Technical availability, successful collection, and the existing Alpaca confirmation do not resolve this conflict.

This proposal evaluates an explicit confirmation bound to the selected Binance public source and to a narrow local use. It does not claim that Binance has granted a licence or independently verified the user's rights.

## Recommendation

Add a distinct, explicit Control Plane attestation for the user's own local, non-commercial **trading decision support** using the selected ordinary Binance public Spot dataset. Scope it to the local TradeX quote/valuation inputs, owning-plan calculation, risk review, approval decision, and local Prepare/reservation workflow. Require the user to confirm that they reviewed the applicable current terms and believe those terms permit this use.

When, and only when, the current source profile and attestation binding match, the owning `MARKET_DATA_USE` check may return `PASS` with an explicit `USER_ATTESTED_LOCAL_ONLY` reason/provenance. The market-data observation itself remains `dataUseRights=UNVERIFIED`; do not rewrite provider evidence or describe the state as provider-verified entitlement. Without the confirmation, or when its scope is stale, mismatched, unsupported, or contradicted by a known applicable restriction, the check remains `UNAVAILABLE` (or `REJECT` for a known prohibition).

This is a product policy decision to rely on the user's scoped declaration for local decision support. It is not legal advice, provider confirmation, or proof that provider terms permit the use. If this reliance is not acceptable, retain the current fail-closed behavior and revise S29.11's positive AC6/AC10 target; do not create a test-only bypass.

## User stories

1. As a user selecting ordinary Binance Spot public data, I can review the applicable source terms and explicitly confirm my own local, non-commercial decision-support use.
2. As a user, I can see that my confirmation applies only to the selected source, dataset, instruments, purpose, and local destination, and is marked as confirmed by me rather than Binance.
3. As a user, I can see the confirmation and its provenance in current and captured reviews, while the quote itself continues to show provider rights as unverified.
4. As a user, I cannot use this confirmation to enable shared/commercial use, redistribution, export, model/tool transmission, another source, or an actual order submission.
5. As a user, a source, dataset, purpose, destination, terms-policy, or jurisdiction change retires the old confirmation before another RiskDecision or Prepare can rely on it.

## Proposed implementation decisions

### Scope and evidence separation

- Keep three facts separate: technical collection, provider-observed entitlement, and a user-attested local-use policy decision. This proposal changes only the third.
- Introduce a Binance-specific local scope such as `LOCAL_TRADING_DECISION_SUPPORT`; do not broaden Alpaca's existing `POLICY_VALUATION` attestation or reuse its `FX_DATA_USE_RIGHTS=VERIFIED` projection.
- Bind the attestation to the workspace and local user, `BINANCE_SPOT_PUBLIC`, provider `binance`, ordinary environment, the reviewed public Spot dataset/profile, the exact currently supported instrument set used by S29.11 (BTC/USDT, ETH/USDT, and BNB/USDT only where the owning fee/valuation model consumes it), declared jurisdiction, policy version, applicable first-party terms reference/revision, purpose, `LOCAL_ONLY` destination, and the allowed local retention/consumer boundary. Binance public market data has no remote account binding; do not invent one.
- Require the Control Plane to derive the binding from saved source configuration and policy state. Renderer input may carry the user-reported declared jurisdiction, explicit affirmative confirmation, and expected state version; the jurisdiction remains declared context, not a support or entitlement result. Renderer cannot provide a rights status, provenance, digest, source identity, or binding.
- The attestation records the user's declaration. It cannot override a known prohibition, an out-of-profile source/dataset, unsupported jurisdiction, or unresolved policy state that the reviewed source policy treats as blocking. Current terms and their review provenance must be visible before confirmation.

### Risk and Prepare behavior

- With a current matching attestation, only the selected ordinary Binance source's local `MARKET_DATA_USE` check may pass, and its reason/provenance must identify `USER_ATTESTED_LOCAL_ONLY`. Do not change `BinanceSpotQuoteEvidence.dataUseRights`, source entitlement, or unrelated sources/local simulation.
- Without a current matching attestation, retain `DATA_USE_RIGHTS_UNVERIFIED` and `UNAVAILABLE`. A known applicable restriction remains a refusal regardless of confirmation.
- A pass removes this one reachability blocker. It does not assert that the whole RiskDecision is eligible. Fresh quote/depth, venue admission, instrument rules, account health/permissions, source-use requirements, fee/FX denomination, balances, reservations, approval and every other independent gate remain mandatory. S29.11 AC6 is satisfied only by a real producer-to-Prepare path that passes all of them.
- The attestation may support local approval and durable Prepare/reservation when all independent gates pass. It does not authorize Arm, PLACE, cancellation/reconciliation changes, provider mutation, or an order submission. A subsequent financial action still needs the separately authorized user flow.
- Persist only the attestation binding/status/provenance and existing bounded risk/review digests. Keep raw public books ephemeral and local; retain no raw quote/book in risk history. No raw market data or derived source values may be sent to cloud models, tools, research, exports, or another external consumer.

### Interaction, invalidation, and capture

- Add a default-unchecked confirmation and a separate “Record my local-use confirmation” action to the saved Binance source settings. Explain in plain language that the user is confirming their own permitted local use, not receiving Binance verification; list the exact instruments/purposes and excluded destinations. No provider request occurs.
- Disable the action when no eligible source is saved, settings are dirty, the source/policy/terms profile is unsupported, or the declared scope includes shared, commercial, exported, or external use.
- Display `Confirmed by you for local-only decision support` with its source, dataset, purpose, destination, terms reference, jurisdiction, time, and policy version. Never display provider entitlement as `VERIFIED` because of this record.
- Retire the attestation on source disconnect/reselection, source/dataset/instrument-scope change, policy or applicable-terms revision, jurisdiction change, purpose/destination/retention change, explicit withdrawal, or an out-of-date terms review. A new quote receipt or cached read must not create or renew it.
- Current views use the current Control Plane projection. Captured reviews preserve the exact decision and attestation provenance immutably, clearly labelled as captured; they cannot renew current eligibility.

## Acceptance criteria for a follow-up implementation ticket

- [x] Paired PRD, Backend ARD, Frontend ARD, UI specification, and S29.11 implementation specification define the same Binance-specific user-attested local purpose and explicitly distinguish it from provider entitlement and the Alpaca-only `POLICY_VALUATION` attestation.
- [ ] A default-off, separate Settings confirmation is bound by the Control Plane to the saved source, dataset, instrument scope, local user/workspace, jurisdiction, current policy/terms revision, purpose, destination, and retention boundary; renderer cannot set rights or binding fields.
- [ ] With no confirmation, stale/mismatched binding, unsupported profile, or known restriction, the real selected-source `MARKET_DATA_USE` check refuses with an exact reason. With a valid confirmation it passes only with user-attestation provenance; provider evidence remains unverified.
- [ ] Real external-producer evidence traverses the public React → typed Rust Control Plane → owning statement → RiskDecision → approval/Prepare consumer. No positive finance snapshot, seeded approval, test-only source label, or rights setter is introduced.
- [ ] RED/GREEN coverage proves binding and invalidation, terms/policy changes, withdrawal, unsupported/known-prohibited scope, no network on Save/confirm, source isolation, captured-state behavior, and no raw/derived market-data egress.
- [ ] A positive Prepare case is accepted only if every independent S29.11 gate passes. This attestation alone never creates overall financial PASS, approval, Arm, PLACE, provider mutation, or execution authority.
- [ ] Keyboard and responsive Settings/current/captured review behavior is verified at 390/768/1280px; English and Simplified Chinese contracts and traceability are synchronized.

## Testing decisions

Use the existing public React → typed Rust Control Plane → disposable SQLite/outbox seam, with only external HTTP/vault/WS/clock producers faked. Record RED before GREEN. Test valid confirmation, absent confirmation, wrong workspace/source/profile/instrument/purpose/destination/jurisdiction/terms version, withdrawal, policy and terms revision, known restriction, source change/disconnect, and concurrent state-version conflict. Assert both the returned user-attestation reason and the unchanged `dataUseRights=UNVERIFIED` evidence. Exercise the same real producer result through the owning statement, RiskDecision, approval, and Prepare path; separately prove all other gates still block when their inputs are missing. Verify saved/captured UI and outbound boundaries, with raw book/quote values absent from persisted history, model/tool inputs, research, exports, and other external consumers.

## Out of scope

- Provider entitlement, legal/regional approval, or Binance confirmation represented as observed or independently verified.
- Commercial/shared use, redistribution, publication, cloud/model/tool use, export, retention beyond the existing local policy, another Binance dataset, another venue, or any unbound instrument.
- Alpaca attestation changes, transaction FX qualification, fees/funding, account permissions, new trading/order forms, Arm, PLACE, order submission, provider writes, or real financial operations.
- Closing S29.11 or its parent, changing acceptance checkboxes, or claiming full application/native/provider/physical/release completion.

## Decision required before implementation

Approve or reject the narrow product policy: may the user's explicit, source-bound declaration be relied on for their own local, non-commercial Binance-based trading decision support through local Prepare, while the provider entitlement remains explicitly unverified and all other gates remain independent? If approved, create a separate implementation ticket and revise the paired normative contracts before implementation. If rejected, preserve the current refusal and revise the S29.11 positive AC6/AC10 expectation rather than weakening the guard in code.

## Normative references

- Backend ARD §41.43 — ordinary Binance Spot market source and unconditional current `MARKET_DATA_USE` refusal.
- Backend ARD §41.60 / Frontend ARD §13.46 — existing Alpaca-only, local `POLICY_VALUATION` attestation, expressly insufficient for Binance or Prepare.
- S29.11 #135 AC1, AC6, AC9, AC10 and AC14 — source-use contract and positive owning-plan/Risk/Prepare acceptance.
- Existing proposal record: [#145](https://github.com/kaiqiangh/tradex/issues/145). This proposal is an extension decision, not a change to #145/#146.
