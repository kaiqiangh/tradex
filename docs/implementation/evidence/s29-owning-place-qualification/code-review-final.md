PASS

Independent serial review of #129 (Owning Binance Spot Live PLACE qualification from exact
capacity and interval inputs), Standards axis then Spec axis, each run by a separate read-only
reviewer over the uncommitted working tree on top of `dev@7307787`. No scope creep and no
fail-open was found on either axis. Every finding below was re-verified against the source by me
before it was acted on; the dispositions are recorded with the reason, not just the action.

## Axis 1 — Standards

HARD FINDING (fixed). `docs/FILE_MANIFEST.md` recorded stale SHA-256 values for four rows:
`docs/TradeX_UI_Prototype_Spec_v1.0_RevC.md`, `docs/zh/TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md`,
`docs/TradeX_Frontend_ARD_v1.0_RevC.md` and `docs/zh/TradeX_Frontend_ARD_v1.0_RevC_zh.md`. The
row line counts were already correct, so the rows were not generated from the delivered bytes.
Cause: those four files were edited after the manifest was regenerated. I re-hashed every row in
the manifest rather than only the four reported ones; exactly those four were stale and the other
sixteen matched. All four were corrected and all twenty rows now verify against the file bytes.
The manifest is not part of `npm run check`, so this was an integrity defect rather than a gate
failure.

JUDGEMENT-LEVEL SMELLS (recorded, not all acted on). The reviewer listed repeated reason-code
literal pairs, the two standing-limitation constant arrays duplicating the consumed slices'
labels, a duplicated `Some("POSSIBLE_INTERVAL_BOUNDARY")`, the two `matches!` helpers on one enum,
three tests repeating the same `external.edit.set` closure, the `carried`/`skeleton` names, and
`SpotOwningExplanation` not matching its sibling components' naming. These are stylistic. The
repository's own conventions override the heuristic baseline, and each is a local readability
question rather than a correctness or contract risk, so they are recorded here and deliberately
left unchanged rather than churned at close.

## Axis 2 — Spec

A1 (resolved, doc + surface). The implementation decisions said the capacity projection would gain
explicit availability and committed sources for this path, so the UI could name the declared origin
of each number. The consumed `spot_capacity.rs` was in fact left unchanged. The PRD and Backend
§41.48 require this statement to derive from the already-delivered read-only observations and
never from a second projection, and this spec itself forbids a parallel projection and requires the
consumed slices to be inherited unchanged, so the projection was deliberately not reshaped. The
correct disposition was therefore two-sided: the spec and the Backend ARD now state the delivered
design and its reason, and the statement itself was made to attribute every reported figure to the
venue read that declared it. `SpotOwningAdmission.tsx` now renders two explicit origin rows
(open-order and balance inventory; order-rate counters) instead of one combined line, so the
published "declared origin of each number" is literally true on the real surface. The change is
renderer-only — no command, payload, generated validator or Control Plane value changed, so no IPC
regeneration was required. `ui-green.txt`, `ui.json` and the four region screenshots were re-earned
on a freshly restarted server, and the re-run and its reason are recorded as an amendment.

A2 (resolved, doc). The bound-identity list read as if the two inputs' state versions had to match
each other. `same_intent` does not compare them, and that is correct: each state version is
`sha256` over `[that observation struct, sequence]`, so two different endpoints' observations
cannot produce equal digests, and requiring equality would block even a fully current qualified
intent — the opposite of fail-closed. The RED payload archived in this directory shows the two
values differing (`d75337af…` for capacity, `b0c2c2a5…` for intervals). Each input's own state
version is enforced against its own expected cursor by the slice that produced it, and both are
folded into the two evidence-version digests this qualification emits. The wording is now corrected
in the spec and the Backend ARD, in both languages, to say exactly that.

C1 (resolved, doc). Backend §41.48 claimed the approval review digest deliberately does not cover
Spot evidence, so a time-sensitive statement could not invalidate an issued approval binding. That
is false. `RiskDecisionInputKind` includes `SpotCapacity` and `SpotOrderIntervals`
(`risk.rs:846-847`), `normalize_input_material` is a no-op for both (`risk.rs:1084-1085`),
`is_approval_bound_risk_input` excludes only `Reservations` (`lib.rs:2093-2095`), and
`approval_review_digest` binds every remaining input's digest (`lib.rs:2167-2178`). The RED payload
in this directory independently confirms it: both kinds appear in the decision's `inputs`. Prepare
revalidation refuses when the digest differs (`lib.rs:8741-8752`). The behaviour is nevertheless
sound and stays as it is: `build_approval_review` derives a fresh evaluation (`lib.rs:8426-8427`)
and the owning gate reads `current_decision.spot_owning`, so no stale admission can be replayed,
and a genuine change to the evidence correctly requires a newly reviewed approval. The ARD sentence
was the error and now states the actual behaviour, including that the owning gate is evaluated
against a freshly derived evaluation.

## Method

Both axes were run as separate independent read-only reviewers. The Standards reviewer's hard
finding was re-verified by re-hashing all twenty manifest rows; the Spec findings were re-verified
by reading `spot_owning.rs`, `risk.rs` and `lib.rs` at the cited lines and by inspecting the
archived RED payload. Dispositions above were applied by me. Reviewer note: independent source
recheck made no edits/tests/build/browser/network calls.

Reviewed tree: uncommitted working changes on `dev@7307787`. Gates re-run after the dispositions:
`npm run check` EXIT=0 (Rust 492 passed / 0 failed / 39 pre-existing ignored over 34 binaries;
Node 17/17; Rust, JSON Schema and TypeScript agree; Traceability OK: 203 requirements, 70 screens,
13 QA scenarios, 23 baseline files), recorded in `full-check.txt`. The owning UI replay was
re-earned green on a freshly restarted integration server; the three consumed slices are unaffected
because no consumed test references the owning statement. This is source acceptance for this slice;
it is not parent, map, S28, S17, physical S27, financial or main acceptance, and prototype code is
unchanged.
