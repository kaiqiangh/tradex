## Agent skills

### Issue tracker

Issues and specs are tracked in GitHub Issues using the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default five canonical triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository. See `docs/agents/domain.md`.

### Product docs

The TradeX product baseline lives in `docs/` and is read in this authority order:

1. `docs/TradeX_PRD_v1.0_RevB.md` — normative PRD (source of truth);
2. `docs/TradeX_UI_Prototype_Spec_v1.0_RevB.md` — UI / prototype target specification;
3. `docs/TradeX_Prototype_Coverage_Matrix_v1.0_RevB.md` — requirement ↔ prototype traceability;
4. `docs/TradeX_Prototype_QA_Report_v1.0_RevA.md` — QA evidence baseline;
5. `docs/prototype/` — clickable prototype package (assessment baseline @ git tag `prototype-v1.0-reva-baseline`).
