# TradeX Prototype QA Report — v1.0 Final Revision A

**Date:** 2026-09-04 (automated checks re-verified 2026-09-04)  
**PRD:** `TradeX_PRD_v1.0_RevA.md`  
**Coverage Matrix:** `TradeX_Prototype_Coverage_Matrix_v1.0_RevA.md`  
**Target:** `docs/prototype/` @ git tag `prototype-v1.0-reva-baseline` (commit `5af4ee5`, unchanged from the prior TradeX v1.0 Final release)  
**Purpose:** Separate verified prototype behavior from unverified requirements and document the new Revision A QA baseline.

---

# 1. QA Interpretation

This report distinguishes:

- **syntax/action smoke tests** — code can execute without missing functions/exceptions;
- **state assertions** — resulting product state is verified;
- **visual QA** — rendered layout/interaction is reviewed;
- **non-functional QA** — latency, restart timing, leakage, etc.;
- **security/architecture QA** — authority, keychain, sandbox, gateway boundaries.

A smoke-test PASS is not equivalent to requirement satisfaction.

---

# 2. Automated Checks Re-run for Revision A

## 2.1 JavaScript Syntax

```bash
node --check prototype/app.js
```

Result: **PASS**. Re-verified 2026-09-04 against the baseline tag (`node --check prototype/app.js`).

## 2.2 Click-handler Reference Audit

All inline `onclick` function names were compared with declared/global action functions.

Result:

- referenced action functions: **47**;
- missing implementations: **0**;
- status: **PASS** — re-verified 2026-09-04 against the baseline tag (47 unique inline `onclick` references, all resolved to defined functions).

## 2.3 Existing State-flow Smoke Evidence

The previous final package recorded a minimal DOM harness running **72 representative actions** without JavaScript exceptions.

That evidence remains useful as an action/smoke baseline but is **not** upgraded to state-assertion evidence in this revision. The harness source was not part of the delivered prototype package, so this result is recorded as **historical evidence that cannot currently be re-run**; it becomes re-runnable only when a state-assertion harness is added to the package (see §8).

Status: **PASS — smoke/action execution only**.

---

# 3. Targeted Source Audit for Documentation Gaps

A source-level audit of the current `prototype/app.js` was run while preparing Revision A.

| Target requirement | Current source result | Status |
|---|---|---|
| Ask mode in mode cycle | absent | **GAP** |
| Account-scoped arming state | absent; prototype uses one global `liveArmed` boolean | **GAP** |
| Live approval source field | absent | **GAP** |
| Live approval provider timestamp | absent | **GAP** |
| Workspace import action | absent | **GAP** |
| Sortino metric | absent | **GAP** |
| Profit Factor metric | absent | **GAP** |
| Turnover metric | absent | **GAP** |
| Market calendar / `MARKET_CLOSED` surface | absent | **GAP** |
| `INSTRUMENT_HALTED` surface | absent | **GAP** |
| Interactive reservation-conflict state | absent | **GAP** |

These gaps are reflected as `Partial` / `Not covered` in the Revision A Coverage Matrix rather than being reported as complete.

---

# 4. Safety-specific Prototype Evidence

| Check | Evidence level | Result |
|---|---|---|
| Selecting Live mode silently arms execution | source + smoke | PASS — does not occur |
| Live order while disarmed requires Arm action | source + smoke | PASS |
| AAPL / Trading 212 identity remains consistent in monitoring | prior smoke + source | PASS |
| Stale approval requires refresh | source + prior smoke | PASS |
| Expired approval cannot execute via represented flow | source + prior smoke | PASS at prototype-flow level |
| Risk rejection is non-overridable in represented UI | source | PASS at UI level |
| Ambiguous submission exposes no blind-retry action | source + prior smoke | PASS at UI level |
| Live cancellation requires approval | source + prior smoke | PASS |
| `CANCEL_PENDING` / `CANCELLED` represented | source | PASS |
| Market order shows max authorized spend | source | PASS |
| Live arming is per-account | source | **FAIL / GAP** — global boolean |
| Risk-policy save explicitly invalidates pending approvals | source | **PARTIAL** — disarms, but invalidation object/state not represented |
| Competing reservations prevent double-spend | source | **NOT TESTED / NOT REPRESENTED** |
| Live approval shows complete source/timestamp/entitlement provenance | source | **FAIL / GAP** |

---

# 5. Requirement-level QA Status

## 5.1 Prototype-verifiable but currently incomplete

- AC-008 / AC-043 — approval provenance: **PARTIAL**;
- AC-010 — T212/Binance/Bitget Demo/Testnet order lifecycle: **PARTIAL**;
- AC-013 — arbitrary material proposal mutation invalidation: **PARTIAL**;
- AC-014 / AC-044 — policy-change approval invalidation: **PARTIAL**;
- AC-017 / AC-045 — competing reservations: **NOT TESTED / NOT REPRESENTED**;
- AC-041 — Ask mode: **NOT REPRESENTED**;
- AC-042 — per-account live arming: **NOT REPRESENTED**;
- AC-047 — workspace import: **NOT REPRESENTED**;
- AC-048 — complete backtest metrics: **PARTIAL**;
- AC-049 — market calendar/halt/corporate-action UI: **NOT REPRESENTED**;
- AC-050 — complete canonical error variants: **PARTIAL**;
- AC-051 — provider-schema-driven credentials: **NOT REPRESENTED**;
- AC-052 — keyboard safety: **NOT TESTED**;
- AC-053 — FX provenance: **NOT REPRESENTED**;
- AC-054 — responsive visual safety: **QA PENDING**.

## 5.2 Architecture/security requirements not provable by standalone prototype

The following must not be marked PASS based on UI appearance alone:

- FR-001 Codex App Server integration;
- FR-007 trusted process boundary;
- FR-008 OS keychain storage;
- FR-025 privileged Order Gateway;
- SEC-001..006;
- AC-012 generic Codex approval isolation;
- AC-019 direct gateway isolation;
- AC-020 model secret isolation;
- AC-027 strategy sandbox credential isolation;
- AC-028 prompt-injection authority protection;
- AC-029 secret leakage across logs/storage.

Status: **NOT APPLICABLE TO STANDALONE PROTOTYPE QA — production security tests required**.

---

# 6. Non-functional Verification Status

| Requirement | Status | Evidence needed |
|---|---|---|
| NFR-001 UI event render p95 | NOT TESTED | runtime/browser timing |
| NFR-002 tool-card render p95 | NOT TESTED | runtime/browser timing |
| NFR-003 restart → reconciliation <5s | NOT TESTED | production/local runtime timing |
| NFR-004 secrets in logs = 0 | NOT TESTED | real secret/log scan |
| NFR-005 unapproved live submissions = 0 | prototype-flow evidence only | gateway integration tests |
| NFR-006 duplicate retries = 0 | prototype-flow evidence only | provider timeout fault injection |
| NFR-007 no silent ambiguous assumption | represented | backend state-machine tests |
| NFR-008 open orders reconciled after restart | represented | broker integration restart test |
| NFR-009 complete live snapshot provenance | FAIL / GAP | add fields + tests |
| NFR-010 healthy state required | represented | integration assertion |
| NFR-011 material change invalidates approval | PARTIAL | mutation state assertions |
| NFR-012 policy change invalidation/disarm | PARTIAL | approval-state assertions |
| NFR-013 telemetry default off | NOT TESTED | settings/runtime inspection |
| NFR-014 decimal arithmetic | NOT TESTED | unit/property tests |
| NFR-015 keyboard/focus safety | NOT TESTED | browser keyboard/accessibility QA |
| NFR-016 narrow layout safety | QA PENDING | desktop + narrow browser review |

---

# 7. Visual / Browser QA

The current prototype source contains responsive CSS, but the prior container could not complete reliable Chromium screenshot regression because of headless Chromium/DBus behavior.

Revision A therefore records:

- source-level responsive behavior: **PRESENT**;
- automated pixel-level regression: **NOT COMPLETED**;
- desktop manual visual review: **REQUIRED**;
- narrow-window manual visual review: **REQUIRED**;
- keyboard/focus live-approval review: **REQUIRED**.

The Coverage Matrix therefore uses `QA pending` instead of `Complete` for responsive/accessibility verification.

---

# 8. Required QA Additions Before Prototype Coverage Sign-off

The authoritative close-out checklist is Coverage Matrix §6 (Prototype Release Gate); this section must not duplicate it. The QA-specific additions on top of that gate are:

1. Add a reusable state-assertion harness to the package; until then the §2.3 smoke result remains historical, non-re-runnable evidence.
2. Complete desktop / narrow-window visual sign-off and keyboard / focus / accessibility review (covers the `QA pending` items in Coverage Matrix §3 and `AC-052` / `AC-054`).
3. Add NFR timing, decimal-arithmetic, and secret-leak evidence once the production runtime exists (NFR-001/002/003/004/013/014).

---

# 9. QA Conclusion

The current standalone prototype remains a strong product-review artifact and its core live-order safety story is materially improved versus earlier versions. However, Revision A explicitly removes the previous implication that all PRD requirements were proven `Complete` by the prototype.

Production readiness requires the additional UI coverage and QA evidence listed above, plus architecture/security/integration tests that a standalone prototype cannot provide.
