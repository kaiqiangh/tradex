# TradeX Prototype QA Report — v1.0 RevC

**Revision date:** 2026-09-05\
**Evidence baseline:** docs/prototype reviewed at main@6c4b267; this revision changes documentation only, not HTML/CSS/JS\
**Scope:** document consistency, source inspection, standalone fixture interaction, and limited browser checks

## 1. Result

**Prototype interaction/handoff gate: NOT PASS.** This report supersedes the 2026-09-04 aggregate source-alignment PASS and direct-handoff recommendation. The target rules are clarified in both languages of the PRD, frontend/backend ARDs and UI Spec; the clickable prototype still has reproduced defects. A specification revision is not a prototype repair.

There are 9 FAILED and 3 PARTIAL scenarios below. These count regression scenarios, not FR/AC coverage percentages. The specification can guide repair; prototype handoff requires these critical scenarios and the independent QA gates to pass.

## 2. Evidence method and limitations

- Read English specifications and corresponding Chinese passages; check requirement/screen ID sets and state terminology.
- Browser interactions verified cancellation routing, unknown order state, Manual Resolution, historical snapshots, focus, and a 768 px layout.
- Execute selected current app.js state functions in an isolated Node VM with simulated DOM/timers for CLOSED, UNTRUSTED, sleep, Disable All, proposal identity, Backtest/model-failure routing; no real provider requests.
- Source inspection includes the final effective function overrides, not just earlier definitions.
- Browser/VM observations are review records, not a committed automated regression suite; a complete screenshot archive was not retained. Re-execute and retain new evidence after repair.
- JavaScript syntax, document links, bilingual ID/status parity, and manifest byte hashes are checked locally in this revision. Syntax validity or ID presence does not prove interaction correctness.

## 3. Prototype regression scenarios

Each case separates observation from post-repair acceptance. Source line references refer to the unchanged prototype baseline; documentation line numbers will move with edits. Target details are in [UI Spec §14](./TradeX_UI_Prototype_Spec_v1.0_RevC.md#14-interaction-contracts-for-implementation-and-prototype-repair).

<a id="qa-01"></a>

### QA-01 — Preserve cancellation through arming

**Status: FAILED** · **Method:** Browser + source · **Traceability:** FR-049; AC-033

- Steps: Accounts → Trading 212 Live DISARMED → Cancel remaining → Arm.
- Expected/post-repair acceptance: Return to cancellation approval with the exact broker order identity and remaining quantity; then CANCEL_PENDING and broker-confirmed outcome. Also check rejection and a fill arriving before approval.
- Observed: Opens Approve live order / Approve & Place with undefined order fields. requestCancel records CANCEL, but confirmArmLive branches only on instrument.
- Evidence: [app.js:817,836](./prototype/app.js); UI Spec §14.3–14.4.

<a id="qa-02"></a>

### QA-02 — Unknown order timeline

**Status: FAILED** · **Method:** Browser + source · **Traceability:** FR-061; AC-021, AC-022, AC-034

- Steps: Order activity → Ambiguous submission + Manual Resolution; inspect the timeline and broker identity.
- Expected/post-repair acceptance: UNKNOWN_RECONCILING displays only observed events and an uncertain treatment; no fabricated acknowledgement/fill. Verify all normalized states with explicit render branches.
- Observed: Unknown status defaults to the FILLED timeline rank and green status treatment, while broker identity is Pending.
- Evidence: [app.js:718–731](./prototype/app.js); UI Spec §14.5.

<a id="qa-03"></a>

### QA-03 — Evidence-based Manual Resolution

**Status: FAILED** · **Method:** Browser + source · **Traceability:** FR-023, FR-026, FR-076; AC-061

- Steps: Open Manual Resolution; choose Confirmed submitted and Confirmed not submitted from separate reset scenarios.
- Expected/post-repair acceptance: Load inspectable backend evidence; block missing/stale/conflicting evidence. Submission requires verified broker identity; absence requires sufficient proof. Close/Keep reconciling preserves capacity; valid resolution still requires health revalidation and explicit arming.
- Observed: Confirmation buttons mutate state directly; submitted sets ACCEPTED with fixed identity presentation, and not-submitted clears reservation.active without an evidence step.
- Evidence: [app.js:793,829,920–926](./prototype/app.js); UI Spec §14.5.

<a id="qa-04"></a>

### QA-04 — Execution eligibility and approval expiry

**Status: FAILED** · **Method:** Isolated state transitions + source · **Traceability:** FR-020, FR-021, FR-022, FR-024, FR-038, FR-062, FR-077; AC-015, AC-016, AC-049, AC-063

- Steps: Set market CLOSED or timeHealth UNTRUSTED; request a Live order → Arm → Approve. Inspect approval-expiry entry and submission callbacks.
- Expected/post-repair acceptance: Block unsupported orders and untrusted-time authority at approval and dispatch. Add cases for HALTED, forbidden permissions, disabled market orders, stale quote/FX, policy change, expiry before dispatch and expiry after uncertain transmission. PRD §45 controls release.
- Observed: CLOSED and UNTRUSTED both reach RESERVED; approveLive checks arming only. Approval-expiry helper exists but no complete current UI entry/TTL transition was demonstrated. Additional listed negative cases require regression execution after repair.
- Evidence: [app.js:817,823,835](./prototype/app.js); UI Spec §14.3–14.4.

<a id="qa-05"></a>

### QA-05 — Global disarm and dispatch interruption

**Status: FAILED** · **Method:** Isolated state transitions + source · **Traceability:** FR-027, FR-028, FR-051, FR-058; AC-014, AC-025, AC-037, AC-042, AC-044

- Steps: Arm Trading 212 and Binance, then simulate sleep. Separately pause an order at RESERVED, Disable All, and advance the submission callback.
- Expected/post-repair acceptance: System recovery disarms all accounts. Disable before dispatch stops I/O and applies the guarded release rule; after possible transmission it retains/reconciles capacity. Check shared-policy weakening across all bound accounts.
- Observed: Sleep disarms only selectedAccount; the other account remains armed. A RESERVED order advances to SUBMITTING after Disable All. Shared-policy multi-account behavior still needs complete regression coverage.
- Evidence: [app.js:819,823,832,837](./prototype/app.js); UI Spec §14.3.

<a id="qa-06"></a>

### QA-06 — Immutable historical provenance

**Status: FAILED** · **Method:** Browser + source · **Traceability:** FR-002, FR-029, FR-055, FR-072, FR-075; AC-026, AC-058, AC-060

- Steps: Open completed US tech Research; select DeepSeek without Send, then change mode to Backtest; inspect existing result and Artifact provenance.
- Expected/post-repair acceptance: Historical mode/account/model/provider/context remains unchanged; only the next Turn uses picker changes. Provider attempts append history. Resume another thread preserves that thread's identity.
- Observed: Existing 'Immutable turn provenance' changes with current model/mode/account selectors; Backtest also clears the selected account.
- Evidence: [app.js:651,767–768,811–813](./prototype/app.js); UI Spec §14.1.

<a id="qa-07"></a>

### QA-07 — Draft editing and proposal regeneration

**Status: FAILED** · **Method:** Isolated identity check + source · **Traceability:** FR-019, FR-080; AC-032, AC-065

- Steps: Generate AAPL proposal identity; change quantity or policy version and regenerate. Inspect the editable-draft entry.
- Expected/post-repair acceptance: Different revision IDs/hashes, immutable old proposal, invalidated old approval, explicit new approval, and a usable draft editor.
- Observed: Identity is fixed per instrument; regeneration reuses it. The editable OrderDraft → Generate Proposal interaction is incomplete.
- Evidence: [app.js:561–564,833](./prototype/app.js); UI Spec §14.2.

<a id="qa-08"></a>

### QA-08 — Screener, picker and instrument selection

**Status: FAILED** · **Method:** Source · **Traceability:** FR-009, FR-035, FR-046, FR-053; AC-039, AC-040

- Steps: Edit parsed filters and Run; vary checked Context Picker items; open NVDA or AMD from Markets.
- Expected/post-repair acceptance: Displayed inputs drive fixture outcomes or show explicit unsupported state; exact selected context/instrument identity is preserved.
- Observed: FilterSpec is static and Run ignores input; Attach always reports AAPL + Trading 212 Live; non-BTC instruments map to AAPL.
- Evidence: [app.js:366,398,403,406](./prototype/app.js); UI Spec §14.1, 14.6.

<a id="qa-09"></a>

### QA-09 — Backtest entry and frozen run inputs

**Status: PARTIAL** · **Method:** Isolated routing check + source · **Traceability:** FR-031, FR-054, FR-074; AC-048, AC-059

- Steps: Select Backtest mode and Send; compare the route with the strategy-editor run entry.
- Expected/post-repair acceptance: Both paths converge on validated/frozen backtest configuration and a distinct run lifecycle. No current-market broker execution.
- Observed: Backtest result/metric surfaces exist, but Send routes all non-Ask modes to startResearch. Full configuration → run → cancellation/failure → comparison behavior is not established.
- Evidence: [app.js:811,856](./prototype/app.js); UI Spec §14.7.

<a id="qa-10"></a>

### QA-10 — Model setup and remediation

**Status: PARTIAL** · **Method:** Isolated availability check + source · **Traceability:** FR-064, FR-068, FR-069, FR-070, FR-071; AC-038, AC-050, AC-055, AC-056, AC-057

- Steps: Simulate MODEL_UNAVAILABLE, close the error, and Send; inspect OAuth/quota recovery and setup controls.
- Expected/post-repair acceptance: Closing does not restore Send; setup/probe/re-login/key-validation and explicit retry/switch paths are complete. Fallback remains opt-in; control-plane recovery remains usable.
- Observed: Error state can coexist with a new running research turn. Recovery controls are incomplete; labels do not prove sidecar launch, credentials validation, or in-flight retry policy.
- Evidence: [app.js:797,847,856](./prototype/app.js); UI Spec §14.8.

<a id="qa-11"></a>

### QA-11 — Keyboard and modal focus

**Status: FAILED** · **Method:** Browser + source · **Traceability:** AC-052

- Steps: Open a Live approval, press Tab/Shift+Tab and Escape; inspect row semantics.
- Expected/post-repair acceptance: Safe initial focus, trapped focus, inert background, restored focus, keyboard-accessible rows, and no implicit Enter approval.
- Observed: Tab reaches background + New Thread while dialog is open. Modal ARIA exists, but focus management is absent; clickable rows rely on onclick. Screen-reader and full Enter-path verification remain pending.
- Evidence: [app.js:118–119,737–739,859–865](./prototype/app.js); UI Spec §14.9.

<a id="qa-12"></a>

### QA-12 — Narrow navigation and retained actions

**Status: PARTIAL** · **Method:** 768 px browser inspection + source · **Traceability:** FR-004, FR-045; AC-039, AC-054

- Steps: At 768 px inspect compact navigation; audit sidebar/history and Provider action CSS. Add a 390 px stress pass after repair.
- Expected/post-repair acceptance: New/history Thread, secondary pages, Provider configuration, and all Live controls remain reachable; dialogs/actions are not covered.
- Observed: Sidebar/history are hidden without replacement Thread actions in More; CSS hides Provider buttons. 390 px and complete responsive regression were not executed.
- Evidence: [styles.css:19,23](./prototype/styles.css), [app.js:798](./prototype/app.js); UI Spec §14.9.

## 4. Documentation decisions and validation

| Topic | Documentation repair | Verification boundary |
|---|---|---|
| Approval/order expiry | PRD §18/§45 distinguish pre-dispatch release, possible transmission, and provider terminal adjustment | Runtime transaction/crash tests pending |
| Gateway isolation | Backend ARD §5/§24 specify separate process, private channel, dispatch/revocation boundary | Process/credential isolation requires runtime proof |
| Frontend/backend protocol | Backend ARD §41–42 own the contract; Frontend ARD §10 references it | Generated types/compatibility tests pending implementation |
| Bilingual semantics | Correct Chinese OAuth fallback drift; pair the new contracts in full | ID parity does not replace semantic review |
| Entry points | RevC authority order, ARD pairs, prototype relative paths, and manifest updated | Manifest describes current files, not release certification |

## 5. Independent incomplete QA gates

1. Re-run QA-01–QA-12 after repair, retaining source revision, initial state, steps, expected/observed result, and screenshot/state-assertion evidence.
2. Complete desktop/768 px/390 px visual regression, long content/scrolling/empty/error states, screen readers, and all keyboard paths.
3. Real Codex App Server, CLIProxyAPI, IPC schema/order/replay compatibility, and model-outage independence of the control plane.
4. Gateway process authentication/credential isolation; races between disarm and dispatch, expiry and submission, policy save and consumption, manual resolution and fill.
5. SQLite transactions, exactly-once reservation adjustment, real restart/crash/sleep recovery, and no premature release during unknown submission.
6. Provider sandbox/demo/testnet query completeness, capabilities, broker identity, cancel/fill races, and evidence rules.

Update each gate only after its own verification passes. Prototype success does not certify a production trading system.
