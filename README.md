# TradeX

Local desktop trading workspace, implemented serially against the [RevC product documents](docs/README.md). The complete [requirement inventory and delivery map](docs/implementation/README.md) covers 35 work items. Development stays on `dev`; the final `dev` → `main` PR is reserved for human review.

The current implementation includes the S01 workspace shell and the Alpaca Paper and Trading 212 Demo/Live slices of S02: native credential entry, macOS Keychain storage, read-only provider connection testing, explicit permission review, persisted account details and resumable domain events. Binance and Bitget connections, model execution, research and trading remain pending. Empty pages and disabled controls do not count as implemented workflows.

## Run on macOS

Use Node 24.19.0, Rust 1.98.1 (pinned by `rust-toolchain.toml`), and Xcode command-line tools with the macOS SDK.

```sh
npm ci --ignore-scripts
npm run desktop
```

Select an absolute workspace directory, or use the default `~/.tradex/workspaces/default`. The workspace database contains non-secret metadata and credential references. Broker credentials are entered only in native secure fields and stored in macOS Keychain. Reopening an existing folder preserves its identity, name and base currency. A second writer, incompatible database or failed write produces an explicit error.

In Accounts, select Alpaca Paper and enter a connection label. The secure window accepts the Paper API key ID and secret; Command-Return tests the connection and Escape cancels. Review the observed account data and explicitly acknowledge the UNVERIFIED permission scope before confirming. A connection does not enable trading. Refresh reads account data; Disconnect removes local credential access without cancelling broker orders.

Trading 212 uses its API key and secret for the selected Demo or Live environment. Account values use primary currency; position prices retain their instrument currency. The exact Invest/ISA subtype and complete key scope are unavailable from the API. Allow five seconds between account refreshes; provider quota errors require a later manual retry. Live connections remain DISARMED with execution blocked.

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

`check` requires Python 3 and OpenSSL for a disposable loopback HTTPS server. It validates the Rust-derived wire schema, TypeScript/build, event projection, real SQLite/command behavior, HTTPS redirect/response-size/deadline boundaries and planning inventory. The TLS check adds its temporary certificate only to its own test client; it does not change OS trust or contact a broker. The inventory check proves traceability only. Regenerate wire artifacts with `npm run schema:generate` after changing Rust protocol types and their paired Backend ARD contract.

For browser verification, `npm run dev:browser` serves the frontend on `127.0.0.1:1420` with an isolated temporary workspace and the same Rust dispatcher over inherited stdio. Provider HTTP and secret entry are explicit fixtures in this mode; the production parser, state, database and events stay real. This development-only transport cannot replace the Tauri/Keychain integration check. In the Codex CUA runtime, import `tests/workspace-ui.mjs` and call `checkWorkspaceUI(tab, browser)` with the selected local-app tab and browser bindings. It checks persistence, unavailable-model controls and all eight navigation destinations at 768/390 widths; a viewport-control failure is reported as unverified, never an application pass.

Then import `tests/provider-ui.mjs` and call `checkProviderUI(tab, browser)` to verify account review, restore, refresh, narrow-window Settings and disconnect. Run the real OS storage boundary separately with `cargo test --test providers native_keychain_roundtrip_drives_the_real_connection_lifecycle -- --include-ignored`; this creates and deletes disposable synthetic Keychain credentials.

Run the same UI check with a third argument of `trading212/DEMO`, then `trading212/LIVE`, to verify those variants. Their separate real Keychain check is `cargo test --test trading212 native_keychain_keeps_demo_and_live_items_separate -- --include-ignored`.

See [S01 evidence](docs/implementation/s01-evidence.md) and [S02 Alpaca evidence](docs/implementation/s02-alpaca-evidence.md) and [S02 Trading 212 evidence](docs/implementation/s02-trading212-evidence.md) for acceptance scope and remaining work.
