# S02 Binance Spot — accepted connection slice

- Ticket: [#8](https://github.com/kaiqiangh/tradex/issues/8); parent spec [#5](https://github.com/kaiqiangh/tradex/issues/5), Wayfinder [#1](https://github.com/kaiqiangh/tradex/issues/1).
- Implementation/review baseline: `dev@78e449a03fc5587a153d8dadb2dfe3dcf4b88ae5`, after #7 closed. Accepted implementation: `dev@feea542bf07be13b4f4cd98eea1e81a243c7b230`. This verifies the connection slice, not the full application release.
- Status: S02 ticket #8 passed implementation, automated checks, native Keychain, browser, bundled native interaction and serial Standards → Spec review. S02 remains incomplete until #9 and its integration acceptance finish.

## Implemented and checked

- Fixed Testnet/Live GET destinations, HMAC-SHA256 over exact transmitted query bytes, sensitive key header, no credentials on server-time reads. Fixed recvWindow 5000, bounded monotonic time sample and sanitized CLOCK_SKEW failure. RFC 4231 signature vector plus per-request signature verification and route rejection checks.
- Native-asset free/locked/exact total, unpriced Spot holdings, Unicode symbols, large numeric UID/order IDs, symbol-scoped order identity. No implicit USDT/USD or workspace-currency conversion. Complete all-symbol open orders, bounded arrays/body size and no mutation requests.
- Live key introspection separate from account capability; Testnet UNVERIFIED. Missing/unknown scope remains unverified; detected withdrawals, transfers, margin/derivatives and unsupported FIX trading block confirmation despite acknowledgement. Scope/IP changes require review; Live remains DISARMED and execution BLOCKED.
- Public protocol tests cover confirmation, failure preservation, restart/refresh, immutable identity, invalid time and slow sampling, invalidation before signed reads, negative/duplicate data, reflected secrets, incomplete arrays, forbidden permission changes and own-credential cleanup. Existing shared lifecycle tests continue covering stale versions, workspace/disconnect races, persistence failure and interrupted cleanup.
- Explicit macOS `NativeVault` test passed with disposable synthetic Binance credentials only. It writes, reads and deletes its own items; no real broker credential was read or requested.
- Real TLS transport passed 13 serial cases: verified exchanges for five fixed hosts plus redirect, authentication, rate limit, oversized body, timeout, Binance timestamp/signature rejection and HTTP 418 ban classification. Trust changes apply only to the disposable test client, not OS trust stores.
- `npm run check` passed (schema agreement, TypeScript/Vite build, Node projection checks, Rust suite and requirement inventory). Later added negative/duplicate and time-invalidation cases passed targeted tests. Clippy with all targets/features passed again after the final test additions; formatting and requirement-inventory checks also passed.
- Testnet and Live each passed five actual browser lifecycle/responsive groups at 1280/768/390 widths. A visible long-decimal card overflow was found and fixed; independent card containment checks passed at 768/390. Viewport screenshot was inspected. The full-page image showed stitching artifacts and is not acceptance evidence.
- The debug Tauri `.app` bundle built successfully with embedded `tauri://localhost` assets. Native interaction acceptance on this embedded build is recorded below.

## Bundled native acceptance — 2026-09-08

The user resolved macOS Desktop Folder authorization. The same rebuilt app then opened a dedicated QA workspace (`f2a224fe-5bdd-4cba-8b80-867d63e54bbd`), separate from existing user-configured accounts.

- Testnet secure dialog showed API key/secret with TESTNET help, initial key focus and Tab to secret. Synthetic invalid credentials produced an actual provider authentication rejection. The connection remained FAILED / authentication INVALID / credential MISSING, with no observation or successful sync; Refresh was disabled and focus returned to Connect.
- Live secure dialog showed LIVE help. Command-Return with empty fields stayed in the dialog with a required-field error and saved no credential. Tab moved between secure fields; Escape cancelled and returned focus to Connect. The record remained DISCONNECTED / MISSING / DISARMED, with no observation or successful sync.
- Both exact owned Keychain lookups returned exit 44 (item absent). Six files covering the dedicated SQLite/WAL and AX evidence, plus 646,418 retained process-log bytes, contained neither synthetic secret sentinel. SQLite account/outbox records contain only non-secret failed/cancelled projections.
- Screenshots of both native dialogs and the 390px browser viewport were inspected. The initial OS permission wait is retained as historical diagnostic evidence; it is resolved and does not count as an application pass by itself.

## Serial code review and limits

Standards: PASS, zero actionable findings against the fixed baseline, repository instructions and skill smell baseline. Spec: PASS for #8, zero remaining actionable findings after native validation. The actual screenshot exposed one long-decimal card overflow, corrected and rechecked before delivery. Reviews ran serially; no agents were spawned.

No successful credentialed Binance authentication or trading is claimed. The real provider test used invalid disposable Testnet values; deterministic contract tests establish success/scope/lifecycle behavior. Subsequent map slices own credentials-backed trading acceptance, execution, streams and instrument metadata. S02 and the full 35-item map remain incomplete; no main PR or merge is authorized yet.

Local diagnostic artifacts: `.artifacts/s02-binance/` (ignored); controlled QA evidence, not release artifacts.
