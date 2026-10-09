# S29.7 — serial two-axis review (issue #128)

Fixed point: `dev@ec10e99` (planning handoff). The implementation under review is the
uncommitted S29.7 working tree.
Diff command: `git diff ec10e99` (the untracked `docs/implementation/evidence/s29-price-range-explanation/` directory is part of the change).
Standards sources: `AGENTS.md`, `README.md`, `docs/agents/*`, `docs/implementation/README.md`,
the normative product docs. There is no `CONTEXT.md`, `CODING_STANDARDS.md`, `CONTRIBUTING.md`
or `docs/adr/` in this repo, so the Fowler smell baseline applied on top.
The two axes ran as two independent sub-agents over the same frozen diff, then were aggregated
here without merging or reranking findings.

`shared/ipc-validators.js` was excluded from detailed reading on both axes: it is a
230 000-line generated Ajv bundle whose validator numbering shifts globally on every
regeneration (historical regenerations produce 15k–37k line diffs), and `npm run schema:check`
already enforces byte equality against the Rust schema.

## Standards

Hard violations: **none**. Three conformance checks passed with reasoning:

1. **English/Chinese plus generated-artifact sync — PASS.** New sections exist in both languages
   with matching IDs and status tokens (Backend ARD §41.47, Frontend ARD §13.32, UI Spec §14.31,
   PRD §21) and `docs/FILE_MANIFEST.md` line counts and SHA-256 now match the working-tree bytes.
2. **Test conventions — PASS.** The new `rules_get`/`rules_refresh` helpers mirror the existing
   `interval_get`/`interval_refresh` shape; the added tests use the existing `RuleRig`/`RuleHttp`
   fixtures, the `isolated_rule_scenario` guard and descriptive snake_case names.
3. **Public-test boundary — PASS.** No production setter or test-only hook was added. The new
   fixture overrides live in `src-tauri/tests/support/provider_fixtures.rs` and `src/bin/ipc.rs`
   (dev browser transport).

Judgement-call smells and dispositions:

| Smell | Site | Disposition |
|---|---|---|
| Duplicated Code | `capture` / `capture_absent` both re-derived the provider RFC3339 instant | **Fixed** — extracted `provider_timestamp`, and both now use the shared `reference_age` window instead of a hardcoded `30` |
| Duplicated Code | first new test inlined `execute_refresh` while the test file now offers `rules_refresh` | **Fixed** — the test now calls the helper |
| Speculative Generality | `price_range_row` `NoRule` / `UnsupportedConfiguration` arms are unreachable from the guarded call site | **Accepted** — the match documents the complete eight-state space, and the guard is the thing that could legitimately change; a `_` fallback would be equally dead and less honest |
| Duplicated literals | three reason strings appear in both `preview.explanation` and `price_range_row` | **Accepted** — the repo uses string literals for wire reason codes throughout; the two roles (explanation vs reason code) are deliberately distinct fields |
| Repeated predicate scans | `execute_refresh` walks `e.constraints` three times with `requires_reference`, `execution_reference_required` and an inline shared filter | **Accepted** — the three predicates answer different questions (static reference need, execution reference need, shared-reference need) and the collection is bounded by the documented 64-rule cap |

Cross-axis note raised by the Standards agent: `src/financial_sources/binance_rules.rs` emits
`EXECUTION_PRICE_RANGE_UNQUALIFIED` for any `PRICE_RANGE` rule, including a verified non-enforcement.
**Accepted** — the spec (line 43) requires the unresolved future duty to stay explicit, and an
over-inclusive unresolved obligation can only withhold authority, never grant it.

## Spec

Findings against `docs/implementation/s29-price-range-explanation-spec.md`:

1. **Stale or expired execution reference was not retired — fixed.** The preview assigned
   `observation.map(|(reference, _)| reference.clone())` on the not-fresh path, so an expired
   observation still surfaced its **price** in the current preview. That contradicted spec line 42
   ("stale/late/newer-owner results retire its current evidence") and Story 12 ("expiry … retire it,
   so that cached input cannot be renewed by get"). The stale path now keeps only provenance
   (digest, provider time, first receipt) and nulls the price, matching the captured-view treatment.
   Covered by the new `proposal_price_range_retires_a_stale_execution_reference_instead_of_renewing_it`.
2. **"Retained as digests only" was unbacked for the bounds — fixed.** `SpotPriceRangePreview` had
   no bound digest, so `capture` simply dropped the bounds while the UI claimed they were retained as
   digests. Added `boundsDigest`, computed over the reference digest, selected side, unit and exact
   bounds; it survives capture exactly like `SpotRuleReference::digest`. The React surface now names
   the digest instead of asserting an empty claim, and the UI test asserts the digest is present
   after capture.
3. **Evidence incomplete — addressed.** `green.txt` now records the focused and whole-file runs and
   this record; the build/pin and Rust-backed UI evidence are produced by the same slice run before
   closure.
4. **Test matrix partial — partially addressed.** "Empty configuration" is now covered by the added
   `NO_RULE` case (no PRICE_RANGE rule, no reference read, no invented per-rule row) and expiry by
   the new stale test. Duplicate/malformed/overflow bounds, wrong symbol/identity and
   pending/failure/reopen/newer-owner remain covered by the shared, rule-type-agnostic parser and
   scope tests in the same file (`original_wire_types_and_duplicate_fields_cannot_replace_a_good_observation`,
   `malformed_rule_range_retires_current_evidence_and_retains_the_last_good_observation`,
   `rule_counts_preserve_large_integers_but_reject_values_outside_documented_int64`,
   `proposal_explanation_distinguishes_missing_reference_dynamic_and_unknown_rules`) rather than by
   duplicated PRICE_RANGE-specific tests.
5. **Manifest description scope — accepted.** `docs/FILE_MANIFEST.md` had not yet absorbed the
   already-closed S29.6 `#127` interval-quota contract, so the regeneration names both the previously
   missing `#127` and the new `#128` contract. The code change itself stays PRICE_RANGE-only.

No scope creep in the implementation: every touched production site is PRICE_RANGE-specific and no
non-PRICE_RANGE rule schema or reference behaviour changed.

## Summary

- Standards axis: 0 hard violations; 2 judgement-call smells fixed, 3 accepted with reasons.
- Spec axis: 5 findings; 2 correctness/truthfulness defects fixed (stale-price retirement, unbacked
  bound-digest claim), 2 evidence/test-coverage items closed by this run, 1 accepted.
