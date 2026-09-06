# TradeX

Local desktop trading workspace, implemented serially against the [RevC product documents](docs/README.md). The complete [requirement inventory and delivery map](docs/implementation/README.md) covers 35 work items. Development stays on `dev`; the final `dev` → `main` PR is reserved for human review.

The current implementation is S01: a Tauri/React workspace shell, Rust command boundary, SQLite persistence and resumable domain events. Provider connections, model execution, research and trading are still pending. Empty pages and disabled controls do not count as implemented workflows.

## Run on macOS

Use Node 24.19.0, Rust 1.98.1 (pinned by `rust-toolchain.toml`), and Xcode command-line tools with the macOS SDK.

```sh
npm ci --ignore-scripts
npm run desktop
```

Select an absolute workspace directory, or use the default `~/.tradex/workspaces/default`. The workspace database contains non-secret metadata; credentials are not collected by this slice. Reopening an existing folder preserves its identity, name and base currency. A second writer, incompatible database or failed write produces an explicit error.

Build a local app with embedded frontend assets:

```sh
npx tauri build --features desktop --debug --bundles app
```

The result is `target/debug/bundle/macos/TradeX.app`. This development package is not the signed/notarized release required by S34.

## Verify

```sh
npm run check
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

`check` validates the Rust-derived wire schema, TypeScript/build, event projection, real SQLite/command behavior and planning inventory. The inventory check proves traceability only. Regenerate wire artifacts with `npm run schema:generate` after changing Rust protocol types and their paired Backend ARD contract.

For browser verification, `npm run dev:browser` serves the frontend on `127.0.0.1:1420` with an isolated temporary workspace and the same Rust dispatcher over inherited stdio. This development-only transport cannot replace the Tauri integration check. In the Codex CUA runtime, import `tests/workspace-ui.mjs` and call `checkWorkspaceUI(tab, browser)` with the selected local-app tab and browser bindings. It checks persistence, unavailable-model controls and all eight navigation destinations at 768/390 widths; a viewport-control failure is reported as unverified, never an application pass.

See [S01 evidence and remaining checks](docs/implementation/s01-evidence.md) for the current acceptance status.
