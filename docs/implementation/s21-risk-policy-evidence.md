# S21 #80 Risk policy configuration and persistence evidence

Status: **VERIFIED for #80 only**. Code and UI review baseline: `dev@20113b0c203e8409f49db8c4ce7df2cc751b5c36` (implementation `1292c853a08bb84e0219bdfc68d8f1a1754449ae`, save-state fix `20113b0c203e8409f49db8c4ce7df2cc751b5c36`).

## Delivered scope

- Settings → Risk & Limits edits every PRD §21 policy field with explicit units, unset monetary/exposure defaults, exact decimal strings, allowed/blocked identifiers and environment constraints. Onboarding retains its seven setup fields.
- The public versioned `risk.save_policy` command rejects malformed, out-of-range, unknown-field and stale-version input before mutation. A valid policy increments its version and persists the policy projection and `risk.policy.changed` outbox event in one SQLite transaction. Reopening restores values and version; recognized pre-S21 projections receive unset defaults for new fields.
- Hard rules remain backend-owned and read-only. No Agent policy-write command was added. A save disables the complete form until its result arrives.
- English and Chinese Backend ARD §41.6 and UI Spec J2 define matching wire fields, units, validation, defaults, migration, list semantics and interaction requirements.

The complete deterministic evaluator and policy-change effects on approvals/arming are later S21 work. This evidence does not close the S21 map item or full Risk Engine requirement.

## Verification

- `npm run check` **PASS** at the code baseline above: schema agreement, TypeScript check, production build, all 10 frontend tests, Rust workspace tests (144 library tests and integration suites), and requirements traceability (203 requirements, 70 screens, 13 QA scenarios, 23 baseline files).
- `cargo test -p tradex --lib full_risk_policy_round_trips_through_the_public_command_after_reopen -- --nocapture` **PASS**. The public-command test covers every new field, invalid values, unknown fields, no mutation/event-sequence advance on rejection, one sequence advance on valid save, policy-version persistence, reopen and stale-version rejection. Rust also covers pre-S21 persisted-policy compatibility.
- Rust-backed browser fixture at `http://127.0.0.1:1420/`: Settings → Risk & Limits saved a complete policy and reloaded the exact values. The interface showed the market-order slippage guard, keyboard traversal, visible hard rules, and no horizontal overflow at 1280, 768 and 390 px. The saved environment selection was Live only.
- `python3 scripts/check_requirements.py` and `git diff --check 9f3b8ac HEAD` **PASS**.
- Standards and Spec reviews **PASS** after disabling all policy inputs during save.

One earlier full run transiently failed an unrelated Local Paper reopen assertion; that test passed by itself, `cargo test --workspace` passed on rerun, and the final `npm run check` passed. Six explicit native Keychain / pinned-gateway integration checks were skipped by their existing environment guards. The production build reports its existing 3.21 MB JavaScript chunk-size warning.

The browser used an isolated temporary workspace with provider responses supplied by fixtures. No provider order, cancellation, transfer, withdrawal, live API request, or Bitget Demo account was used. `BITGET` appeared only as a venue in the local risk-policy fixture.
