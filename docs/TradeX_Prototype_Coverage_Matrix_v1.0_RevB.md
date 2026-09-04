# TradeX PRD / Prototype Requirement Traceability Matrix

**Version:** 1.0 Final — Revision B  
**Date:** 2026-09-04  
**PRD:** `TradeX_PRD_v1.0_RevB.md`  
**UI Specification:** `TradeX_UI_Prototype_Spec_v1.0_RevB.md`  
**Current prototype assessed:** `docs/prototype/` @ git tag `prototype-v1.0-reva-baseline` (commit `5af4ee5`) plus the UI-consistency fixes in `867ec00`; re-verified at `867ec00`: `node --check` PASS, 47/47 unique `onclick` references defined. The RevB LLM-provider requirements (FR-068–FR-073) are new target requirements and are not yet implemented in the prototype.

## 1. Status Legend

| Status | Meaning |
|---|---|
| **Covered** | Current prototype visibly/clickably demonstrates the requirement at prototype fidelity. |
| **Partial** | Some required behavior is represented, but material detail is missing. |
| **Visual-only** | Concept is mentioned/rendered, but not demonstrated as a complete interactive state. |
| **Not covered** | Required target surface/flow is absent from the current prototype. |
| **N/A — implementation** | Requirement is architectural/runtime/security and cannot be proven by a standalone UI prototype. |
| **N/A — future** | Explicitly outside v1.0 MVP prototype scope. |
| **QA pending** | UI/source exists, but required visual/non-functional/accessibility verification is pending. |
| **Covered visually** | Variant of `Covered` used in §3/§4 where the evidence is visual/interactive only, with no state assertion. |
| **Visual-only / implementation** | Mixed status used in §3: part of the requirement is rendered in the prototype, part is an implementation-only concern. |

**Important:** `Covered` means prototype coverage only. It does **not** mean the production implementation satisfies the requirement.

**QA ↔ Matrix status mapping (used by `TradeX_Prototype_QA_Report_v1.0_RevA.md`):** `PASS` ↔ `Covered`; `PARTIAL` ↔ `Partial`; `NOT REPRESENTED` / `FAIL / GAP` ↔ `Not covered`; `NOT TESTED` ↔ `Partial` or `QA pending` depending on context; `QA PENDING` ↔ `QA pending`; `NOT APPLICABLE` ↔ `N/A — implementation`.

---

## 2. Functional Requirement Traceability

| ID | Requirement summary | UI Spec | Current prototype evidence | Prototype status | QA / production note |
|---|---|---|---|---|---|
| FR-001 | Pinned Codex App Server | §12 handoff | none | N/A — implementation | runtime integration required |
| FR-002 | Persistent Thread/Turn/Item UX | B2/B5 | thread history + timeline | Covered | production persistence not proven |
| FR-003 | Stream agent/tool activity | B5/B6 | running/tool cards simulated | Visual-only | real runtime streaming not proven |
| FR-004 | Resume threads | B2 | recent thread navigation | Covered | persistent restart behavior requires implementation QA |
| FR-005 | Financial timeline cards | B5/F3 | tool/order timeline cards | Covered | none at prototype level |
| FR-006 | Typed domain tools | B5 | named tool cards only | Visual-only | tool contracts are implementation work |
| FR-007 | Trusted control-plane separation | §12 | safety copy only | N/A — implementation | architecture/security test required |
| FR-008 | OS keychain credentials | A3 | keychain/trusted-boundary copy | Visual-only | cannot prove storage isolation in prototype |
| FR-009 | Canonical instrument model | C4/C5 | normalized AAPL/BTC labels | Visual-only | domain model implementation required |
| FR-010 | Capability discovery | A3/E2 | permission/capability result UI | Partial | provider schema/capability contract not real |
| FR-011 | Market-data service | C1/C4/C5 | fixture market data | Visual-only | real provider integration required |
| FR-012 | Freshness metadata | C4/C5/F2 | quote age shown | Partial | source/received/entitlement missing in live approval |
| FR-013 | Portfolio aggregation | E3 | cross-account portfolio | Covered | fixture-only |
| FR-014 | FX normalization | E3 | EUR-normalized fixture | Partial | FX source/timestamp/freshness missing |
| FR-015 | Alpaca Paper adapter | G1/G2 | paper UI only | Visual-only | adapter not implemented |
| FR-016 | Trading 212 Demo/Live adapter | A2/E1/F2 | account + live order UI | Visual-only | adapter not implemented |
| FR-017 | Binance Testnet/Live adapter | A2/E1/F4 | account + live market order UI | Visual-only | adapter not implemented |
| FR-018 | Bitget Demo/Live adapter | A2/E1 | account surfaces | Visual-only | order lifecycle not demonstrated |
| FR-019 | OrderProposal model | F2/F4 | proposal cards | Covered | immutable domain model not proven |
| FR-020 | Deterministic Risk Engine | F2/F7/J2 | risk checks/rejection | Visual-only | deterministic engine not implemented |
| FR-021 | Pre-approval validation | F2/F4 | visible checks | Covered | fixture behavior only |
| FR-022 | Pre-execution validation | F3 | timeline stage | Visual-only | actual revalidation logic not proven |
| FR-023 | Atomic reservations | F11 | risk page mentions reservations | Visual-only | no interactive competing-thread flow |
| FR-024 | Single-use live approval | F2/F5/F6 | stale/expiry flows | Partial | one-time token semantics not state-asserted |
| FR-025 | Privileged Order Gateway | F3/§12 | described only | N/A — implementation | security boundary required |
| FR-026 | Idempotency/reconciliation | F9/K2 | ambiguous + recovery UI | Partial | provider idempotency behavior not real |
| FR-027 | Account-scoped arming | F1 | current prototype uses global `liveArmed` boolean | Partial | arming is not account-scoped |
| FR-028 | Crash/startup reconciliation | K2 | startup recovery screen | Visual-only | restart timing/state not real |
| FR-029 | Local execution audit trail | F3/I3 | audit/provenance UI | Visual-only | persistence not proven |
| FR-030 | Provider rate limiting | K5 | priority degradation UI | Visual-only | real rate limiter not implemented |
| FR-031 | Local backtesting | H3-H6 | simulated runs/results | Visual-only | engine not implemented |
| FR-032 | Strategy sandbox | H2 | blocked-capability UI | Visual-only | sandbox isolation not proven |
| FR-033 | Paper/Demo/Testnet trading | G | Alpaca paper flow only | Partial | T212 Demo/Binance Testnet/Bitget Demo order lifecycle missing |
| FR-034 | Error taxonomy | K6 | subset: auth/network/rate/stream/risk/reject/ambiguous | Partial | several canonical categories absent |
| FR-035 | Natural-language screener | C2/C3 | builder/result flow | Covered | fixture query |
| FR-036 | Artifacts | I1-I4 | library/detail/export | Covered | persistence/export implementation not real |
| FR-037 | Strategy versioning | H1/H6 | versions + compare | Covered | fixture-only |
| FR-038 | Market calendar/corporate actions | C6 | no dedicated current prototype surface | Not covered | add market closed/halt/action states |
| FR-039 | Configurable retention | J3 | retention settings UI | Covered | storage behavior not proven |
| FR-040 | Workspace export/import | J3/J4 | export/backup present | Partial | import/restore missing |
| FR-041 | Futures | future | none | N/A — future | P2 |
| FR-042 | A-share | future | none | N/A — future | P2 |
| FR-043 | Additional brokers | future | none | N/A — future | P2 |
| FR-044 | Five-step onboarding | A1-A6 | complete onboarding path | Covered | none at prototype level |
| FR-045 | Thread history/resume | B2 | recent thread list | Covered | persistence beyond session not proven |
| FR-046 | Context/account/model pickers | B4 | clickable pickers | Covered | none at prototype level |
| FR-047 | Provider test/permission review | A3 | generic connection modal | Covered | credential schema is not provider-driven |
| FR-048 | Provider-specific account detail | E2 | selected account changes detail | Partial | required positions/reconciliation/credential health not complete |
| FR-049 | Live cancellation approval | F10 | cancel approval/pending/cancelled | Covered | none at prototype level |
| FR-050 | Risk/reject/expiry UI | F5-F8 | states present | Covered | doc wording only, no code gap; canonical naming already normalized in RevA |
| FR-051 | Startup/auth/stream recovery | K1-K4 | recovery states | Covered | operational behavior simulated |
| FR-052 | Market-order max authorized notional | F4 | Binance BTC approval | Covered | source/timestamps incomplete |
| FR-053 | Full screener builder | C2/C3 | parse → result | Covered | none at prototype level |
| FR-054 | Backtest running/failed/compare | H3/H4/H6 | states present | Covered | none at prototype level |
| FR-055 | Artifact provenance | I3 | provenance modal | Covered | none at prototype level |
| FR-056 | Ask mode | B3 | cycleMode omits Ask | Not covered | source audit confirmed |
| FR-057 | Live-approval market provenance | F2/F4 | quote + age only | Partial | source/provider timestamp/received/entitlement absent |
| FR-058 | Risk-policy invalidation lifecycle | F12/J2 | save disarms global live | Partial | explicit pending-approval invalidation not modeled |
| FR-059 | Reservation conflict surface | F11 | no interactive state | Not covered | source audit confirmed |
| FR-060 | Demo/Testnet order variants | G3 | account variants only | Not covered | no full order lifecycle variants |
| FR-061 | Full order-state UI mapping incl RESERVED | §6 | submission/fill/cancel states | Partial | `RESERVED` not visibly represented as state |
| FR-062 | Market-session/halt/corp action UI | C6 | absent | Not covered | none at prototype level |
| FR-063 | Workspace import/restore | J4 | absent | Not covered | source audit confirmed |
| FR-064 | Complete error-remediation surfaces | K6 | subset only | Partial | missing permission/unsupported/closed/halted/invalid/funds/stale/reconcile/internal |
| FR-065 | Portfolio FX provenance | E3 | EUR fixture only | Not covered | no FX source/timestamp |
| FR-066 | Full backtest metrics | H5 | return/Sharpe/drawdown/win rate | Partial | Sortino/profit factor/turnover absent |
| FR-067 | Provider-schema-driven credential form | A3 | generic API key/secret form | Not covered | provider form schema missing |
| FR-068 | LLM provider connection workflow (CLIProxyAPI OAuth / DeepSeek key) | A4/J1 | absent | Not covered | RevB requirement; A4/J1 redesign pending |
| FR-069 | CLIProxyAPI sidecar lifecycle + health UI | A4/J1 | runtime card shows static "Codex runtime connected" | Not covered | sidecar supervision is implementation work |
| FR-070 | Subscription quota display / exhaustion handling | §4.4/§16.3 | absent | Not covered | RevB requirement |
| FR-071 | LLM error categories (MODEL_UNAVAILABLE/QUOTA_EXCEEDED/OAUTH_EXPIRED) | K6 | absent from canonical error variants | Not covered | taxonomy extended in RevB |
| FR-072 | Per-turn model/provider provenance + audit | I3/B5 | model picker exists; per-turn provenance not rendered | Not covered | RevB requirement |
| FR-073 | Model fallback/routing policy surface | §4.4 | absence of fallback UI consistent with policy | Not covered | routing policy is P1 implementation |

**Source-audit addendum (2026-09-04, baseline tag `prototype-v1.0-reva-baseline`):** PRD §11.6 requires six tool/turn states; the baseline prototype implements four (`Tool: Running / Completed / Failed`, `Turn: Cancelled`) and lacks `Tool: Retrying` and `Turn: Interrupted`. PRD §11.1 lists `Chart` in the context panel; the baseline prototype's context panel has no chart. Revision A had no dedicated AC row for either gap — both are tracked here for the next revision.

---

## 3. Non-functional / Cross-cutting Traceability

| ID | Prototype status | QA status / note |
|---|---|---|
| NFR-001 | N/A — implementation | NOT TESTED: runtime-event latency |
| NFR-002 | N/A — implementation | NOT TESTED: tool-render latency |
| NFR-003 | Visual-only | NOT TESTED: restart-to-reconciliation timing |
| NFR-004 | N/A — implementation | NOT TESTED with real credentials/log pipeline |
| NFR-005 | Visual-only | safety flow smoke tested; production authority not testable |
| NFR-006 | Visual-only | ambiguous retry UI covered; provider fault injection NOT TESTED |
| NFR-007 | Covered visually | ambiguous state shown; production invariant NOT TESTED |
| NFR-008 | Visual-only | startup recovery shown; real broker reconciliation NOT TESTED |
| NFR-009 | Partial | approval provenance fields incomplete |
| NFR-010 | Covered visually | auth/stream/startup states disable live in prototype |
| NFR-011 | Partial | stale/expiry shown; general material-edit state assertion absent |
| NFR-012 | Partial | risk save disarms; pending approval invalidation not asserted |
| NFR-013 | N/A — implementation | NOT TESTED |
| NFR-014 | N/A — implementation | NOT TESTED |
| NFR-015 | QA pending | keyboard/focus/live-Enter behavior NOT TESTED |
| NFR-016 | QA pending | responsive CSS present; browser visual review pending |
| SEC-001 | N/A — implementation | model/keychain isolation requires production security test |
| SEC-002 | N/A — implementation | agent/gateway isolation requires process-boundary test |
| SEC-003 | N/A — implementation | generic Codex approval isolation requires integration/security test |
| SEC-004 | Visual-only / implementation | sandbox restrictions are rendered; actual sandbox boundary not tested |
| SEC-005 | N/A — implementation | prompt-injection authority boundary requires adversarial test |
| SEC-006 | Partial | approval UX is proposal-specific; cryptographic/single-use authority not proven |
| DATA-001 | Visual-only / implementation | normalized labels shown; canonical ID domain model not proven |
| DATA-002 | N/A — implementation | decimal-safe arithmetic requires unit/property tests |
| DATA-003 | Partial | quote age shown; full provenance incomplete |
| DATA-004 | Not covered | FX source/timestamp/freshness absent |
| DATA-005 | Visual-only / implementation | entitlement/retention policy is documentary, not enforced |
| DATA-006 | Covered visually | reproducibility manifest shown; persisted integrity not proven |
| OPS-001 | Visual-only / implementation | UI states broker authority; real broker reconciliation not proven |
| OPS-002 | Covered visually | ambiguous retry blocked in UI; provider fault injection still required |
| OPS-003 | Covered visually | startup/resume recovery states shown; real runtime test required |
| OPS-004 | Visual-only | rate-limit priority screen shown; scheduler not implemented |
| OPS-005 | Not covered | no competing-reservation interactive state |
| OPS-006 | N/A — implementation | packaging/code-signing pipeline required |
| OPS-007 | N/A — implementation | auto-update + crash reporting require production runtime |
| SEC-007 | N/A — implementation | single model exit (127.0.0.1:8317) requires production network test |
| SEC-008 | N/A — implementation | sidecar credential isolation requires process-boundary test |
| UX-001 | Covered | source flow confirms selecting Live does not arm |
| UX-002 | Partial | current prototype arming is global, not account-scoped |
| UX-003 | Partial | immutable order shown, provenance incomplete |
| UX-004 | Covered | explicit environment labels present |
| UX-005 | QA pending | keyboard safety not tested |
| UX-006 | Partial | current UI promotes multiple settings destinations beyond canonical IA |
| UX-007 | QA pending | responsive source exists; visual review pending |

---

## 4. Acceptance-Criteria Traceability

| AC | Prototype status | Evidence / gap |
|---|---|---|
| AC-001 | Visual-only | simulated streamed item states |
| AC-002 | Partial | thread resume exists; process restart persistence not proven |
| AC-003 | Covered | tool cards |
| AC-004 | Covered | separate environments |
| AC-005 | Partial | capability review UI, no real provider validation |
| AC-006 | Covered visually | startup/auth/stream states disable live |
| AC-007 | Covered | research + portfolio fixture |
| AC-008 | Partial | source/full timestamps/entitlement missing in approval |
| AC-009 | Covered visually | Alpaca Paper flow |
| AC-010 | Partial | account variants exist; T212/Binance/Bitget Demo/Testnet order lifecycles missing |
| AC-011 | Covered visually | approval gate |
| AC-012 | N/A — implementation | generic Codex authorization isolation not testable in standalone prototype |
| AC-013 | Partial | stale/refresh variant; arbitrary proposal mutation assertion absent |
| AC-014 | Partial | risk save disarms; pending approval invalidation not modeled |
| AC-015 | Covered | stale approval blocked |
| AC-016 | Visual-only | pre-execution check timeline only |
| AC-017 | Not covered | no competing reservation flow |
| AC-018 | Visual-only | UI says agent cannot change policy |
| AC-019 | N/A — implementation | privileged gateway boundary not testable |
| AC-020 | N/A — implementation | real model/secret boundary not testable |
| AC-021 | Covered | ACCEPTED separate from fill |
| AC-022 | Covered | UNKNOWN_RECONCILING |
| AC-023 | Covered visually | no-retry UI |
| AC-024 | Covered visually | stream-disconnect + REST fallback |
| AC-025 | Covered visually | startup reconciliation |
| AC-026 | Partial | audit chain shown; durable persistence not proven |
| AC-027 | Visual-only | sandbox restrictions rendered |
| AC-028 | N/A — implementation | prompt injection security test required |
| AC-029 | N/A — implementation | secret scan with real pipelines required |
| AC-030 | Covered | Live mode does not arm |
| AC-031 | Covered | explicit arm modal |
| AC-032 | Covered | AAPL/T212 identity continuity fixed |
| AC-033 | Covered | cancellation approval/pending/cancelled |
| AC-034 | Covered | risk/reject/expired/ambiguous surfaces |
| AC-035 | Covered | market max spend + spread/age/fee |
| AC-036 | Covered visually | permission/capability review |
| AC-037 | Covered visually | auth/stream disable live |
| AC-038 | Covered | five-step onboarding |
| AC-039 | Covered | threads + pickers |
| AC-040 | Covered | parsed screener |
| AC-041 | Not covered | Ask mode missing |
| AC-042 | Partial | arming is global not per-account |
| AC-043 | Partial | provenance incomplete |
| AC-044 | Partial | policy save does not model approval invalidation |
| AC-045 | Not covered | reservation conflict absent |
| AC-046 | Partial | Demo/Testnet accounts exist, but full provider order-lifecycle variants are absent |
| AC-047 | Not covered | workspace import absent |
| AC-048 | Partial | missing Sortino/profit factor/turnover |
| AC-049 | Not covered | calendar/halt/corporate action surfaces absent |
| AC-050 | Partial | canonical error variants incomplete |
| AC-051 | Not covered | provider schema-driven form absent |
| AC-052 | QA pending | keyboard safety not tested |
| AC-053 | Not covered | FX provenance absent |
| AC-054 | QA pending | responsive visual signoff pending |
| AC-055 | Not covered | onboarding LLM gate (no Ready without usable LLM provider) absent |
| AC-056 | Not covered | sidecar fail-closed behavior not representable |
| AC-057 | Not covered | quota/OAuth fallback surfaces absent |
| AC-058 | Not covered | per-turn model/provider provenance not rendered |

---

## 5. Open Decisions

Open product decisions live in PRD §72 (`OD-001`–`OD-016`, of which `OD-009` and `OD-015` are resolved as of RevB) and are the single source of truth.

> Maintenance note (editors): do not duplicate or shorten that list inside this matrix; reference IDs only.

---

## 6. Prototype Release Gate

The current prototype remains suitable for product-direction review, but the documentation baseline no longer labels every surfaced concept as `Complete`.

Before calling the prototype itself **coverage-complete** against Revision A, close at least:

1. Ask mode;
2. account-scoped live arming;
3. full live-approval market-data provenance;
4. explicit risk-policy approval invalidation;
5. reservation-conflict state;
6. Trading 212 Demo / Binance Testnet / Bitget Demo order lifecycle variants;
7. `RESERVED` state surface;
8. market calendar / halt / corporate actions;
9. workspace import/restore;
10. complete error-remediation variants;
11. portfolio FX provenance;
12. Sortino / profit factor / turnover;
13. provider-schema-driven credential form;
14. desktop + narrow visual QA and keyboard/accessibility QA;
15. LLM provider surfaces per RevB (FR-068–FR-072): onboarding LLM gate, sidecar health UI, quota/fallback states, per-turn model provenance.
