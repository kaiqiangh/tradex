# S29.7 — final independent serial review (issue #128)

Fixed point: `dev@ec10e99` (planning handoff). Reviewed change: the S29.7 source delivery, now committed as `622e34c9e2d6c344943a0b3aca761b85cf959c1d`. Full detail in `review.md`.

Two independent sub-agents ran serially over the same frozen diff — **Standards first, then Spec** — and their findings were aggregated without merging or reranking.

## Standards — PASS

0 hard violations. Three conformance checks passed with reasoning (bilingual plus generated-artifact sync, test conventions, public-test boundary: no production setter or test-only hook added).

Five judgement-call smells were dispositioned: **2 fixed** (duplicated provider-timestamp derivation extracted into `provider_timestamp` with a shared `reference_age` window replacing a hardcoded `30`; the first new test inlined `execute_refresh` instead of the file's `rules_refresh` helper) and **3 accepted with reasons** (the complete eight-state match arms, the reason-string literals shared between `explanation` and the row reason code, and the three bounded predicate scans). The cross-axis note about `EXECUTION_PRICE_RANGE_UNQUALIFIED` covering verified non-enforcement is accepted: spec line 43 requires the unresolved future duty to stay explicit, and an over-inclusive obligation can only withhold authority.

## Spec — PASS after fixes

5 findings. **2 correctness/truthfulness defects fixed:**

1. An expired execution reference still surfaced its **price** in the current preview, contradicting spec line 42 and Story 12. The stale path now keeps provenance only and nulls the price, covered by the new retirement test.
2. The captured view claimed the snapshot bounds were "retained as digests only" while no bound digest existed. `boundsDigest` is now computed over the reference digest, selected side, unit and exact bounds, survives capture, and is named by the React surface and the UI assertion.

**2 evidence/coverage items closed by this run** (recorded GREEN plus the build/pin and Rust-backed UI evidence) and **1 accepted** (`docs/FILE_MANIFEST.md` regeneration absorbed the previously missing `#127` note alongside the new `#128` contract while the code change stayed PRICE_RANGE-only).

No scope creep: every touched production site is PRICE_RANGE-specific and no other rule schema or reference behaviour changed.

## Result

Both axes PASS against the same frozen source that produced the final checks, the ordinary frontend build and the actual React acceptance run. No remaining source finding.

Closure: the source child is delivered at `7f4a3016f4e5d7e2c06396776907913dfe50f97c` and closed `2026-10-09T20:31:53Z`; see `acceptance-final.json` and `delivery-final.json`. This closes this source ticket only — the S29 parent, S28, S17, physical S27, S33 and the final main gates remain OPEN.
