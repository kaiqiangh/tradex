# S01 verification record

Implementation/review baseline: `177edd3a91980a4686699d281345df231625a8ce`. Spec [#3](https://github.com/kaiqiangh/tradex/issues/3), implementation [#4](https://github.com/kaiqiangh/tradex/issues/4). Status: S01 accepted. Reviewed application commit: `2424dd5bdadb76a63fdf485caff4bc66d7466781`. This is acceptance of the workspace slice, not of the complete TradeX application.

## Executed evidence (2026-09-06)

- `npm run check`: generated Rust/JSON Schema/TypeScript agreement, production frontend build, two projection/envelope tests, five Rust public-dispatcher tests, and the planning inventory check passed. The inventory remains 201 requirements / 70 screens / 12 prototype QA cases / 23 original source files; those counts are not product acceptance.
- Rust checks exercise real on-disk reopen, stable workspace identity/configuration, invalid command/payload rejection, retained/live event continuity, replacement subscriptions, file lock conflicts, foreign/corrupt database preservation, outbox-write rollback and backup-before-upgrade/newer-schema rejection.
- Browser UI used the real Rust stdio dispatcher: saved `S01 Browser Research` / EUR, then reloaded and observed the same workspace ID and creation time. Send remained disabled; Settings showed the actual Control Plane and unconfigured model/runtime components.
- The final `tests/workspace-ui.mjs` CUA run passed all eight destinations with Enter at actual 768/390 widths, without horizontal overflow. The script verifies the applied width. An earlier stale viewport handle and a cross-runtime empty-array assertion were corrected before the passing run; zero browser console errors were observed.
- Local Tauri binary and `.app` builds succeeded. A missing `custom-protocol` feature was corrected so packaged assets are embedded. After the user unlocked macOS, native CUA verified the directory picker returning the isolated test folder, then opened `S01 Native Research` / USD. After Cmd-Q and stopping the development server (no listener on port 1420), relaunch at `tauri://localhost` restored ID `4c869041-be55-493e-8f6c-975660e49b38` and creation time `2026-09-06 16:50:03` (local display) unchanged. A native screenshot confirmed visible keyboard focus on Skip to content.
- The validator initially caused a blank development page because Vite served CommonJS directly. Generation now bundles static ES modules with the installed Vite toolchain, without runtime eval. A browser reload confirmed rendering. The integration bridge also now canonicalizes its temporary root, fixing macOS `/var` versus `/private/var` restore rejection.

## Visual comparison scope

Screenshots were captured and viewed from `.artifacts/s01/` (local, ignored output). Compared against the RevC prototype: navy navigation/brand treatment, blue primary controls, pale background/white bordered cards, compact type/control density, and five-step workspace labels. The S01 shell uses these tokens; complete onboarding composition and later steps belong to S03, and complete cross-page visual acceptance belongs to S33. No full prototype-parity claim is made here.

## Standards

PASS for `177edd3a91980a4686699d281345df231625a8ce...2424dd5bdadb76a63fdf485caff4bc66d7466781`. Reviewed serially under the user's no-parallel-work rule. No remaining documented-standard violations or actionable Fowler smell findings. The real authority lives in Rust/SQLite; the renderer projects versioned state, the browser test bridge is serve-only, and no unused authority/Gateway scaffolding was introduced. Bilingual IPC additions and the product manifest agree. Generated code and lockfiles account for most of the diff.

## Spec

PASS after one confirmed protocol-validation finding was repaired in `2424dd5`: Backend §41.2 requires non-empty IDs and optional string state versions, but the initial generated result validator accepted an empty workspace ID or `null` stateVersion. The added public decoder negative case failed before the fix and passes after Rust schema annotations and regenerated artifacts. No remaining S01 omissions or scope-expanding functionality were found against Spec #3 / task #4. All later model/account/research/Live features remain pending in the map.

Final exact-code checks passed in sequence: `npm run check`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `npx tauri build --features desktop --debug --bundles app`. The rebuilt package was launched again with no port-1420 listener and restored the same native workspace. The browser checks are repeatable via the retained CUA script; final screenshots are local test evidence, not normative documents. No CI workflow or signed/notarized release acceptance is claimed by S01.

Review totals: Standards 0 remaining findings; Spec 1 fixed / 0 remaining findings.

The browser fault-injection check closes the real event transport without modifying domain state. The UI removes its projection, then Retry obtains an unchanged snapshot and successfully re-subscribes. Unsupported/conflicting events are rejected by the retained generated-schema/projection tests. The UI publishes a projection only after the subscription acknowledges replay.

S02 has not started. The complete Wayfinder goal and all later application/Live/provider/release gates remain open.
