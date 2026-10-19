# S29.11 local BASE MARKET rule composition — bounded implementation evidence

**2026-10-19 · dev · local synthetic verification.** The user approved the [reviewed scope](s29-instrument-rules-prepare-reachability-proposal.md) with “批准，继续”. Contract commit `e07240d` synchronized PRD §75, Backend §41.62, Frontend §13.49 and UI §14.46 before runtime changes. This work belongs to [the existing owning ticket #135](https://github.com/kaiqiangh/tradex/issues/135); all fifteen acceptance criteria retain their prior states. Full owning/provider/native/Wayfinder acceptance remains OPEN.

## Delivered behavior

- RiskDecision composes the current exact Proposal/account/instrument/source-bound rules, capacity and interval observations locally. Original source queries remain source-only; no provider or vault I/O is added to evaluation.
- BUY MAX_POSITION qualifies only complete empty all-account inventory and compares exact free+locked+requested BASE. SELL and new-PLACE amendment obligations have purpose-specific inapplicability. Nonempty, partial-fill or foreign-attribution ambiguity remains unavailable.
- Total order counts add one; complete algo/iceberg/list categories use the approved conservative zero-increment policy. Exact violations reject; unknown rules/fields/STP, missing coverage and stale bindings cannot pass. Existing interval admission participates in the result, with its provenance retained.
- Optional bounded comparison/composition fields preserve old captures. Digests bind consumed versions, comparisons, limits and the fixed policy. A shared capacity-coverage predicate keeps owning admission and concrete comparisons consistent.
- Approval consumes the same complete MARKET composition and verified current Spot quote. This repairs the old generic-tradability consumer mismatch without treating standalone source availability as eligibility. Independent financial, permission, health, Arm, consent, funding and dispatch checks remain.
- Current/captured/pre-arm UI distinguishes source-only from composed assessment, displays exact backend comparisons and versions, and explains retained limitations and venue execution-time PRICE_RANGE. Enforced LIMIT remains unavailable. Refresh/reopen preserves captures and cannot restore current qualification.

## Verification

| Check | Result / evidence |
| --- | --- |
| Public producer-to-risk TDD | Initial semantic [RED](evidence/s29-market-rule-composition/first-red.txt); final [12 core scenarios](evidence/s29-market-rule-composition/core-final.txt) pass: BUY/SELL, exact position/count/category/scope boundaries, quotas/STP, ambiguity, unknown rules, unchanged LIMIT, CAS/refresh/reopen. |
| Public producer → approval → Prepare | [Two isolated scenarios](evidence/s29-market-rule-composition/prepare-atomic.txt) pass: normal public inputs issue approval and atomically reserve USDT principal plus BNB estimate; replay consumes once; after-approval incomplete inventory refuses Prepare without partial execution/reservation/consumption rows or outbox events. Reconfirmed in final Hot suite. |
| Shared wire | [Three tests](evidence/s29-market-rule-composition/wire.txt) pass for optional old rows, exact bounded strings, reason/limitation/version bounds and rejection of authority fields. |
| React → Rust → disposable SQLite | [Final result](evidence/s29-market-rule-composition/ui-green.json): current/captured/pre-arm; 1280/768/390, six overflow checks; zero forbidden backend commands, page errors, console errors or external browser requests. Account remains disarmed and no approval is issued. Six PNGs are viewport captures of the actual scrolling UI. [UI RED](evidence/s29-market-rule-composition/ui-red.txt) precedes implementation. |
| Full default regression | [`npm run check`](evidence/s29-market-rule-composition/full-check.txt) passes: schema consistency, production build, 38 frontend tests, 591 reported Rust passes and 33 existing ignored cases, log checks, 203 requirements / 70 screens / 13 QA scenarios / 23 baseline files. Ignored or feature-excluded cases are not acceptance. |
| Feature-specific full regression | [`npm run test:binance-hot`](evidence/s29-market-rule-composition/hot-final.txt): 64 serial isolated scenarios pass, including both owning Prepare cases. |
| Independent review | Serial [Standards](evidence/s29-market-rule-composition/standards-review.md): no hard violation, one duplication smell remediated; then [Spec](evidence/s29-market-rule-composition/spec-review.md): no findings. |
| Native desktop check | [`cargo check -p tradex --features desktop --bin tradex`](evidence/s29-market-rule-composition/desktop-check.txt) blocked, exit 101: host lacks pkg-config `glib-2.0 >= 2.70`. Native build is not verified. |

Captured text receipts normalize terminal trailing whitespace only. Commands used Rust 1.98.1, offline Cargo, debug information disabled for dev/test, one Rust test thread and a 16 MiB test stack. New public-seam tests fake only external HTTP/vault/WS/clock producers; they do not seed application financial authority. No real provider financial action occurred. The synthetic Alpaca support context is not the user's actual source-use or provider-entitlement evidence.

## Handoff

Work remains in the same owning ticket, with native and real source activation/acceptance outstanding. No requirement or AC checkbox was upgraded and no main PR was created. No worktree was used. GitHub CLI issue-comment writes returned `Forbidden`; the [local decision note](evidence/s29-market-rule-composition/decision-note.md) preserves the approved scope without claiming tracker publication. The [requirements/workflow snapshot](wayfinder-resume-2026-10-19.md) remains a historical inventory; this record supplies its later implementation update.
