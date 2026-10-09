# S29.7 — Proposal Price Range execution-rule explanation

Parent: [Verify trusted Binance Spot Live execution](https://github.com/kaiqiangh/tradex/issues/121). Starting implementation/review baseline `dev@a555fe1de526ca380d782522a705b2055c2184c4`. [中文](s29-price-range-explanation-spec_zh.md). Planning only; implementation and acceptance have not started.

## Problem Statement

An immutable ordinary Binance Live Proposal can show static filters, capacity and interval inputs, but PRICE_RANGE remains unexplained. Users cannot tell which execution directions have an actual configured boundary, why its reference is missing or stale, or whether a displayed range promises their order's eventual execution. The existing all-four-required metadata parsing also cannot represent documented partial configurations.

## Solution

Provide one complete read-only vertical slice through the existing current/captured/pre-arm Proposal rules: show the actual execution-rule configuration, required genuine provider reference, exact selected-side snapshot bounds and precise non-enforcement/unavailable reasons. Explain the future execution uncertainty. Do not turn a snapshot preview into placement eligibility or an expected fill.

## User Stories

1. As a user, I want this explanation bound to my immutable Proposal/account/symbol, so that it cannot borrow another intent's configuration.
2. As a user, I want configured BUY and SELL directions distinguished, so that missing configuration is not invented.
3. As a user, I want the genuine provider reference distinguished from book, trade and average inputs, so that another price cannot qualify this execution preview.
4. As a user, I want exact decimal multiplication and quote/base units, so that precision or currency assumptions cannot alter the displayed bounds.
5. As a user, I want actual zero values distinguished from omitted fields, so that a disabled direction is not fabricated.
6. As a user, I want actual provider absence or explicit null distinguished from errors, so that failures never look like non-enforcement.
7. As a user, I want the preview labelled as a current snapshot with future uncertainty, so that I do not mistake it for a fill guarantee.
8. As a user, I want explicit refresh of only genuinely required public inputs, so that reading the view performs no unnecessary provider request.
9. As a user, I want configured metadata and reference provenance/first receipts/material identified, so that the result can be assessed and traced.
10. As a user, I want malformed/duplicate/unknown terms fail closed, so that partial parsing cannot certify the intent.
11. As a user, I want pending and failed refresh retire the current preview, so that old success does not remain current.
12. As a user, I want expiry and all binding/ownership changes retire it, so that cached input cannot be renewed by get.
13. As a user, I want captured history remain unchanged through refresh and reopen, so that saved review does not become new authority.
14. As a user, I want existing percentage/notional reference fallback preserved independently, so that an execution-purpose change does not break static rules.
15. As a user, I want all unresolved capacities, monetary and other financial duties still visible, so that one preview cannot approve an order.
16. As a user, I want exact CANCEL/reconciliation retain its existing boundary, so that this read-only work cannot weaken protective actions.
17. As a user, I want keyboard and1280/768/390 explanations readable, so that limitations are as visible as values.
18. As a reviewer, I want genuine public RED/GREEN and exact-source checks/build/UI/serial review evidence, so that neither labels nor private setters establish acceptance.

## Implementation Decisions

- Retain the standing approved single-ticket highest public React→typed Control Plane→owned disposable SQLite/outbox→external HTTP/vault/WS producer seam. Extend the existing Proposal rules/metadata/reference/current/captured contract, without a parallel authority model or new account/source setup. Renderer payload remains only trusted workspace/Proposal/CAS identity; never accept URL/symbol/reference/bounds/multipliers/time/qualification.
- Bind ordinary Binance Live BTC/USDT or ETH/USDT immutable intent/hash, saved account/source/rule material, market/policy/workspace/session/time and refresh ownership. Local get makes no provider/vault read. Add execution-reference purpose only when actual current selected-side configuration needs it; no universe scan or private collection for this preview.
- Official execution semantics: use only the genuine referencePrice response for this purpose. Omitted multipliers leave the corresponding direction unenforced; absence of a PRICE_RANGE rule or an explicit null reference also means no enforcement under the documented contract. Missing/failed/unknown lookup is not proven absence. Taker-phase limits are recalculated by the venue; out-of-range execution may expire the order. This snapshot neither rejects a Limit intent solely by its stated limit nor establishes future fill/admission. No average/trade/book fallback for PRICE_RANGE.
- Preserve optional original execution fields with bounded strict parsing; all-four-required validation must become documented optional-field handling only for this rule. Present fields remain exact decimal strings, zero stays a real reported number rather than a disabled marker, duplicates/types/precision/range/unknown active fields remain rejected or explicitly unresolved. Reuse512KiB response,64 rule and32 field bounds,64-character decimals,8 transient Proposal slots and existing bounded references; exact arithmetic failure is unavailable, never rounded into success. Preserve all other filter schemas.
- Expose bounded typed selected-side configuration, optional lower/upper snapshot bounds, unit USDT/BASE, genuine reference/null/absence state, provider original timestamp/first receipt/material and precise preview/retirement/failure explanations. Explicit absence must be supported by fresh complete correctly scoped rule evidence, not omitted or truncated collections. Capture only this approved aggregate/digests with the existing risk history; no raw responses or runtime cache persistence.
- Reuse fixed ordinary public reference GET(weight2), exact filtered rule source and shared IP budget/cooldown. No key/UID fabrication, redirects, source mutation, test order/probe, CP lock over I/O or HTTP weight reset. Existing30-second deadline,12-second completion margin and strict first-receipt/provider-time freshness remain; never replace the provider time with local receipt. A reference valid for other static purposes must not silently replace the execution-purpose observation.
- New preview pending/failure/expiry/account/key/source/rule/market/policy/Proposal/workspace/session/time changes, reopen and stale/late/newer-owner results retire its current evidence. Captured review has no polling/refresh or receipt/consent renewal. Preserve percentage/notional null fallback separately and all current public reference behavior outside this purpose.
- PRICE_RANGE preview is execution-unqualified even when its arithmetic/configuration is available. Do not manufacture complete INSTRUMENT_RULES PASS, monetary/risk snapshot authority, dynamic admission/reservation, rights/quote/liquidity/permissions/health/feesFX/funding, Arm/consent/Prepare/Gateway or execution-phase timestamp. Represent its unresolved future duty explicitly, while verified non-enforcement is a purpose-specific fact rather than broad readiness. Preserve independent exact CANCEL/reconciliation.
- Actual current/captured/pre-arm React explains identity/units/configuration/bounds/null versus fault/age/non-enforcement/future uncertainty with explicit recovery. Keyboard1280/768/390 and paired English/Chinese financial/wire/evidence contracts remain required. Prototype code remains unchanged.

## Testing Decisions

Use the existing public account/source/draft/Proposal/rules refresh/get/risk flow and actual React, with only external producers fake. First record genuine public RED for partial metadata and execution-only reference purpose, then GREEN before the next behavior. Verify full/partial/empty configuration, side/direction/original zero/decimal precision/inclusive bounds, explicit null versus missing/error, no execution fallback or unnecessary reads, unknown/duplicate/malformed/overflow bounds and wrong symbol/identity. Keep static null-fallback regression separate. Cover pending/failure/recovery/expiry/reopen/binding/late/newer owner and immutable captured history; all positive preview cases must still lack execution authority. Replay current/captured/pre-arm keyboard1280/768/390. Require current full/affected rule/capacity/interval/Hot/Gateway checks, IPC/paired traceability, ordinary build/pin, independent serial Standards→Spec and exact source/evidence/remote/tracker handoff. Native/provider/financial/physical/prototype/main evidence remains separately labelled.

## Out of Scope

Owning dynamic capacity/admission and reservation qualification, monetary intent/fees/required FX, reference-price streaming, immediate authenticated execution preflight, execution expiry/private-stream implementation, actual orders/probes/Arm/consent, new accounts/source configuration, rights grants, S28/Testnet/Bitget scope changes, physical sleep waiver, prototype fixes and main merge. These included full-goal duties remain later owning work, not waived.

## Further Notes

Primary contracts rechecked2026-10-09: [execution rules](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md#query-execution-rules), [execution reference semantics](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/faqs/price_range_execution_rules.md). Always-unqualified snapshot explanation and retaining independent financial duties are this spec's conservative design, not provider guarantees. Actual technical dependency is the completed existing metadata/Proposal reference explanation; capacity/interval completion determines serial scheduling only and is not an invented technical blocker. Full parent/map/S28/S17/physicalS27 remain OPEN. Standing granularity/seam approvals apply; no repeated quiz.
