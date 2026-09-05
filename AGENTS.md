## Agent skills

### Issue tracker

Issues and specs are tracked in GitHub Issues using the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default five canonical triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository. See `docs/agents/domain.md`.

### Product docs

The TradeX product baseline lives in `docs/` and is read in this authority order:

1. `docs/TradeX_PRD_v1.0_RevC.md` — normative PRD (source of truth);
2. `docs/TradeX_UI_Prototype_Spec_v1.0_RevC.md` — UI / prototype target specification, including §14 interaction contracts;
3. `docs/TradeX_Frontend_ARD_v1.0_RevC.md` and `docs/TradeX_Backend_ARD_v1.0_RevC.md` — implementation architecture; Backend §41–42 own the shared IPC contract;
4. `docs/TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md` — requirement ↔ prototype evidence status;
5. `docs/TradeX_Prototype_QA_Report_v1.0_RevC.md` — observed defects, regression cases, and remaining QA gates;
6. `docs/prototype/` — clickable fixture package; current behavior does not override the normative documents.

Use `docs/README.md` for the current reading order and handoff status. The 2026-09-05 documentation clarification does not fix the prototype code; do not infer PASS from requirement IDs, screenshots, or status labels alone. Do not use the historical RevA tag as the current assessment baseline.

### Chinese translations

`docs/zh/` holds Simplified Chinese translations of the PRD, UI Specification, both ARDs, Coverage Matrix, and QA Report (RevC baseline). The English versions remain the source of truth; see `docs/zh/README.md` for terminology and pairing rules. Keep financial guards, wire schemas, requirement/screen IDs, and evidence statuses synchronized in both languages when either edition changes.
