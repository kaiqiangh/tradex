# S02 Bitget Classic Spot — accepted connection slice

- Ticket [#9](https://github.com/kaiqiangh/tradex/issues/9), parent [#5](https://github.com/kaiqiangh/tradex/issues/5), Wayfinder [#1](https://github.com/kaiqiangh/tradex/issues/1).
- Review baseline: `7963c94b56a09100dc1094dfe0b1b04db836d508`; implementation: `7b5f32929cef26a784aadd44833793b5f569d330` on dev. Acceptance date: 2026-09-09. S02 integration acceptance remains separate; this does not certify the full application.

## Implementation and automated evidence

Three schema-defined native secret fields share the established Keychain and connection lifecycle. Exact provider field counts are checked before storage and after retrieval. Fixed Classic v2 read destinations, HMAC-SHA256/base64 over transmitted timestamp/method/path/query, sensitive headers, bounded monotonic server time and Demo-only paptrading isolate environments. Unsupported Demo reads fail without a Live fallback or invented snapshot. HTTP 400 cannot become success even if its body claims a successful business code.

Balances preserve available/frozen/locked/restricted availability and explain the total's component basis. Ordinary, TPSL and plan orders use their distinct paginated response contracts; plan identity, trigger price, market-buy notional, market-sell quantity and filled quote values remain distinct. No FX, cost basis, execution, stream or reconciliation success is inferred. Known dangerous authorities block confirmation; unknown authorities remain visible and UNVERIFIED. Canonical IP lists participate in scope comparison, so changing an address while remaining RESTRICTED requires review again.

- Seven Bitget public-protocol tests passed, including 103-order pagination, confirm/reopen/disconnect, Demo rejection cleanup, exact signatures/headers, field-count rejection, all dangerous authority classes, unknown scope, IP-list changes, failed-page and changed-identity preservation, malformed time/assets/IDs/envelopes, reflected secrets and in-flight page invalidation. The established shared lifecycle tests cover stale versions, invalid IPC, workspace/disconnect races and failed/interrupted cleanup through the same control-plane path.
- Explicit `cargo test --test bitget native_keychain_stores_and_removes_three_bitget_fields -- --ignored --exact` passed with disposable synthetic credentials. It exercised the production macOS store through successful connection, reopen and disconnect. The ordinary suite intentionally skips this OS-mutating check unless explicitly selected.
- Real HTTPS transport passed 19 serial scenarios: the existing 13 cases plus both Bitget environments and HTTP 400 clock/passphrase/Demo/false-success cases. Certificate trust is confined to the test client.
- `npm run check` passed: Rust/JSON Schema/TypeScript agreement, production frontend build, two Node checks, Rust suite and requirement inventory. Final market-buy fixture and error-copy changes passed targeted Bitget tests and all-target/all-feature Clippy with warnings denied; the final embedded Tauri app build passed. Formatting and product-manifest hashes also passed.

## Browser and native evidence

Both Bitget Live and Demo passed five browser lifecycle groups using the real Rust/SQLite/IPC path with explicit synthetic HTTP/credential boundaries: setup/review/confirm, reload/refresh, 768px controls, 390px controls and disconnect. Account Health, per-asset values, plan/TPSL labels, IP list and separate execution/stream/reconciliation status remained visible. A 390px viewport screenshot was inspected; this is fixture-based UI evidence, not real provider authentication.

Native validation used a new dedicated workspace `426d1b08-680b-4257-a134-2b8e0b6770c9`; existing user-configured accounts were not probed or changed. The embedded application displayed all three NSSecureTextFields with environment help, initial API-key focus and Tab access to secret then passphrase. Empty Command-Return submission stayed in validation without saving; Escape returned focus to Connect. Synthetic invalid Live credentials produced a real authentication rejection, with FAILED / MISSING / DISARMED and no data or successful sync.

Both owned cancelled/failed connection Keychain lookups returned exit 44 (absent). Six dedicated workspace/AX files and 2,607,507 process-log bytes contained none of the three synthetic sentinels. The final bundle was reopened successfully into the same dedicated workspace after the final schema constraint and error-copy changes. No successful authentication with real Bitget credentials or trading is claimed.

## Serial review

Standards: PASS, zero remaining actionable findings against the fixed baseline and repository/skill standards. Existing native UI, store, control-plane lifecycle, decimal helpers and installed crypto dependencies were reused; no new dependency or execution adapter framework was introduced.

Spec: PASS for #9, zero remaining actionable findings. Review corrected an inaccurate unsupported-operation message. Earlier regression testing exposed and fixed unchanged RESTRICTED status masking an IP-list change. Both corrections are included in the implementation SHA. Additive wire fields are paired in English/Chinese Backend §41.3 and generated schemas. Reviews were serial; no sub-agents ran.

Ignored diagnostics: `.artifacts/s02-bitget/`. S02 parent integration review must finish before S03. The full 35-item map remains incomplete; no dev→main PR or merge is part of this acceptance.
