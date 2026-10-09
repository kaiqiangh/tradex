# Serial independent two-axis review

Baseline `2368e22122b2028e92f4c3352c2f08fe80e1344a`, planning HEAD `1d24523158674d2763d50bee49150e8ff9d808d6`, including frozen implementation worktree. Both reviewers independently matched all19 runtime/test/generated-file hashes in implementation-checkpoint.json. Standards completed before Spec started. No reviewer reran validation or performed provider/financial actions.

## Standards

PASS

Standards only: reviewed baseline `2368e22122b2028e92f4c3352c2f08fe80e1344a` through planning HEAD `1d24523158674d2763d50bee49150e8ff9d808d6`, including the implementation worktree. All 19 checkpoint file hashes match current bytes. Documented-standard breaches: **0**.

The change follows AGENTS.md:17–21 authority ownership and AGENTS.md:27–29 bilingual synchronization. English/Chinese PRD, UI and ARD additions preserve financial guards, wire fields and evidence boundaries. Backend ARD §41.1/§41.44 owns the new generated IPC; trusted-consumer/unknown-field/CAS checks, backend-derived intent, exact decimal arithmetic and unlocked public GETs are visible in `src-tauri/src/spot_proposal_rules.rs:16`, `:197`, `:331`, `:676` and `:798`. Captured-price redaction and independent risk checks are retained at `src-tauri/src/lib.rs:7992` and `:8018`. Current errors hide stale success, while captured rendering has no polling/refresh controller (`src/SpotProposalRules.tsx:8`, `:42`, `:54`). Reviewed public Rust/UI tests and supplied current/captured/390 screenshots; screenshots alone are not acceptance.

**Nonblocking judgement — possible Duplicated Code:** `src-tauri/src/spot_proposal_rules.rs:242` and `:291` repeat the QUOTE valuation shape: `if base { ... multiply(&fields.quantity.value, if market { rule_reference(rule, reference)? } else { fields.limit_price... }) ... } else { fields.quantity.value.clone() }`. Consider a shared exact QUOTE-amount helper so notional and quote-asset limits retain identical price-selection semantics. This is a Fowler heuristic, not a documented-rule violation or acceptance blocker.

Validation was **not rerun**: tests, builds, schema generation, UI and provider calls remain unexecuted by this reviewer. Existing evidence is hash-bound to this worktree; serial Spec review and delivery/tracker handoff remain pending. Parent financial gates, physical S27, main and prototype acceptance remain outside this PASS; prototype code is unchanged.

Root disposition: accept the nonblocking two-site QUOTE-amount duplication for this ticket. Both independently scoped evaluator families are covered by exact amount/unit regressions; no correctness defect is identified.

## Spec

PASS

Spec axis: no actionable missing implementation, scope creep, or incorrect behavior found against #125 AC1–AC8 and the paired S29.4 specification.

Reviewed the cumulative frozen worktree from `2368e22122b2028e92f4c3352c2f08fe80e1344a`, with planning HEAD `1d24523158674d2763d50bee49150e8ff9d808d6`. All 19 runtime/test/generated-file checkpoint hashes match. The committed comparison alone contains planning documentation, so this verdict covers the checkpointed implementation worktree.

Backend derives immutable intent and exact account/source bindings ([spot_proposal_rules.rs](/Users/kai/Desktop/my-repo/tradex/src-tauri/src/spot_proposal_rules.rs:331)), evaluates exact scoped static rules while retaining dynamic/unknown obligations (:190, :483), and enforces primary/null-only fallback, original timestamps, age, deadline and late-publication guards (:670). The owning risk check consumes these outcomes and captured references redact prices ([lib.rs](/Users/kai/Desktop/my-repo/tradex/src-tauri/src/lib.rs:7959)). Current and captured Trade/review/history surfaces retain units, purposes, recovery and capture labels ([SpotProposalRules.tsx](/Users/kai/Desktop/my-repo/tradex/src/SpotProposalRules.tsx:8), [OrderDrafts.tsx](/Users/kai/Desktop/my-repo/tradex/src/OrderDrafts.tsx:1512)). English/Chinese normative additions preserve the independent financial guards.

Reviewed public regression source, stored RED/GREEN/full/Hot/Gateway/build/UI evidence and narrow-layout capture. No tests, builds, UI scenarios, provider/account operations or financial actions were rerun. The preceding axis report was not read.

AC8's delivered SHA/remote/tracker handoff remains a delivery obligation after this review; this PASS does not close #125 or parent #121. Update the stale pending-build/UI introduction when recording delivery. Native/provider/financial/physical gates remain separately unfulfilled. 原型代码未改，main未变。

Standards: 0 hard breaches, 1 nonblocking heuristic (Duplicated Code). Spec: 0 actionable findings; SHA/remote/tracker handoff remains to be recorded. No cross-axis reranking.
