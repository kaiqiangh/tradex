NOT PASS

Baseline: user-approved `2f42df60407ef695f6393c84bc24dcaff7e83332` versus the captured working tree. Independent reviewer: `s28_actions_spec_review`; ran serially after Standards, before remediation. All 111 hashes matched. Existing malformed broker test passed (1 passed, 22 filtered); it does not cover these findings.

1. P2 — `financial_sources.rs:1240-1250`, `provider_io.rs:2291,2842`: ordinary known-account readers are tracked only after a metadata reservation already exists, and admission-time intervals can expire during slow reads. AC5 and Backend §41.39 require atomic per-account endpoint quotas and completion accounting. Reserve every known-account summary and retain its hold through completion/failure, including provider headers.
2. P2 — `financial_sources.rs:1158`: date/type/duplicate validation accepts contradictory transitions such as consecutive OPEN events. Originating spec requires coherent dated time events; AC5 requires conflicting schedules to be unavailable. Reject incoherent transitions atomically while allowing bounded-window edge truncation.

Commit/push acceptance remains pending. Native Markets presentation remains unverified; unsupported positive financial capabilities remain blocked.

This report records the initial review, not final acceptance.
