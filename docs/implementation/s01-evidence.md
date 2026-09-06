# S01 verification record

Implementation/review baseline: `177edd3a91980a4686699d281345df231625a8ce`. Spec [#3](https://github.com/kaiqiangh/tradex/issues/3), implementation [#4](https://github.com/kaiqiangh/tradex/issues/4). Status: implementation and focused behavior checks passed; final fixed-baseline review is in progress.

## Executed evidence (2026-09-06)

- `npm run check`: generated Rust/JSON Schema/TypeScript agreement, production frontend build, two projection/envelope tests, five Rust public-dispatcher tests, and the planning inventory check passed. The inventory remains 201 requirements / 70 screens / 12 prototype QA cases / 23 original source files; those counts are not product acceptance.
- Rust checks exercise real on-disk reopen, stable workspace identity/configuration, invalid command/payload rejection, retained/live event continuity, replacement subscriptions, file lock conflicts, foreign/corrupt database preservation, outbox-write rollback and backup-before-upgrade/newer-schema rejection.
- Browser UI used the real Rust stdio dispatcher: saved `S01 Browser Research` / EUR, then reloaded and observed the same workspace ID and creation time. Send remained disabled; Settings showed the actual Control Plane and unconfigured model/runtime components.
- The final `tests/workspace-ui.mjs` CUA run passed all eight destinations with Enter at actual 768/390 widths, without horizontal overflow. The script verifies the applied width. An earlier stale viewport handle and a cross-runtime empty-array assertion were corrected before the passing run; zero browser console errors were observed.
- Local Tauri binary and `.app` builds succeeded. A missing `custom-protocol` feature was corrected so packaged assets are embedded. After the user unlocked macOS, native CUA verified the directory picker returning the isolated test folder, then opened `S01 Native Research` / USD. After Cmd-Q and stopping the development server (no listener on port 1420), relaunch at `tauri://localhost` restored ID `4c869041-be55-493e-8f6c-975660e49b38` and creation time `2026-09-06 16:50:03` (local display) unchanged. A native screenshot confirmed visible keyboard focus on Skip to content.
- The validator initially caused a blank development page because Vite served CommonJS directly. Generation now bundles static ES modules with the installed Vite toolchain, without runtime eval. A browser reload confirmed rendering. The integration bridge also now canonicalizes its temporary root, fixing macOS `/var` versus `/private/var` restore rejection.

## Visual comparison scope

Screenshots were captured and viewed from `.artifacts/s01/` (local, ignored output). Compared against the RevC prototype: navy navigation/brand treatment, blue primary controls, pale background/white bordered cards, compact type/control density, and five-step workspace labels. The S01 shell uses these tokens; complete onboarding composition and later steps belong to S03, and complete cross-page visual acceptance belongs to S33. No full prototype-parity claim is made here.

## Remaining S01 gates

1. Finish serial Standards and Spec review against the fixed baseline; fix confirmed findings, rerun affected checks, record the final exact SHA, then resolve task and spec in order.

The browser fault-injection check closes the real event transport without modifying domain state. The UI removes its projection, then Retry obtains an unchanged snapshot and successfully re-subscribes. Unsupported/conflicting events are rejected by the retained generated-schema/projection tests. The UI publishes a projection only after the subscription acknowledges replay.

S02 has not started. The complete Wayfinder goal and all later application/Live/provider/release gates remain open.
