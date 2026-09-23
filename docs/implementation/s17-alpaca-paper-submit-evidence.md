# S17 Alpaca Paper submission evidence

Implementation SHA: `950c2b967ce027d132fdc56e11aaa1fa3539a359` on `dev` (review baseline `4ac0c44a37cab65bf274ffbc5e388bb8379cf7a6`). This records the acceptance evidence for [#57 安全提交 Alpaca Paper Proposal 并恢复未知结果](https://github.com/kaiqiangh/tradex/issues/57) only.

## Result

The #57 implementation and local acceptance checks pass. The serial Standards and Spec reviews found no blocking findings for this ticket. Typed IPC/schema, Rust persistence and provider transport, account and asset validation, Proposal confirmation, submit recovery, React projection, bilingual ARD updates, and focused regression coverage are included in the implementation commit.

The Rust-backed browser run used the isolated integration bridge, temporary SQLite workspace, fixture credential vault, and controlled HTTPS provider responses. `tests/provider-ui.mjs` passed the Alpaca Paper flow: account connection and reuse after reload, account refresh, exact Proposal confirmation and cancel-review path, acknowledgement distinct from fill evidence, responsive layouts at 390/768/1280px, reload without a second submit action, and local disconnect cleanup. No real Alpaca credential, external provider response, or Paper order was used.

The generic `tests/workspace-ui.mjs` bootstrap helper timed out at its `complementary Workspace` locator immediately after workspace creation; the current screen exposes onboarding's `Workspace` region at that point. The S17-specific provider helper then ran from the normal Accounts page and passed. The generic helper is outside #57 acceptance and remains a follow-up before using it as a full-map gate.

## Checks

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed, including 117 library tests and all runnable integration suites; explicit native Keychain and pinned-gateway tests remain ignored by their declared opt-in conditions.
- `npm run schema:check` — passed; Rust, JSON Schema, and TypeScript agree.
- `npm run typecheck` — passed.
- `npm run test:unit` — passed, 7/7.
- `npm run build` — passed; Vite reports a 2.6 MB JavaScript chunk size warning.
- `node --check tests/provider-ui.mjs` — passed.
- `python3 scripts/check_requirements.py` — passed (201 requirements, 70 screens, 12 QA scenarios, 23 baseline files; inventory check only).
- `git diff --check` — passed.

## Remaining boundary

The real external Alpaca Paper sandbox sequence was not attempted and is `BLOCKED_EXTERNAL`; no actual order was sent. Parent Spec #56 remains open: the #57–#59 local slices now have separate implementation and acceptance evidence, but the required external sandbox lifecycle still has no evidence.
