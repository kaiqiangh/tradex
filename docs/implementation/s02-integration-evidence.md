# S02 — provider connection integration acceptance

Accepted on 2026-09-09 at `dev@3eb784664a79c34a9d82db624d2ad3920c890c4d`. Parent Spec [#5](https://github.com/kaiqiangh/tradex/issues/5); sequential implementation tickets #6 → #7 → #8 → #9 are closed. Review baseline is S01 delivery `62c6ec0ea50d5ed67c8f784256b7737e8667b8e7`. This accepts S02 only; S03–S35 and the complete application remain unfinished.

## Combined verification

- `npm run check` passed on the accepted SHA: schema agreement, TypeScript/Vite build, two Node checks, 27 Rust tests and requirement inventory. The four explicit Keychain tests are intentionally excluded from the default run.
- `cargo test --workspace -- --ignored --test-threads=1` then passed all four native Keychain lifecycle tests serially on that SHA. Every test uses isolated workspaces and disposable values; it does not access configured user broker credentials.
- The browser runner rebuilt the integration executable from that SHA. All seven selections passed sequentially: Alpaca Paper, Trading 212 Demo/Live, Binance Testnet/Live, Bitget Demo/Live. Each passed setup/review/confirmation, reload/refresh, 768px Account Health, 390px Account Health and disconnect: 35 groups total, with exact amount assertions, separate health/arming states and no console errors. HTTP and native secret capture are explicitly replaced only in this browser harness; Rust, SQLite and IPC remain real.
- Native secure entry and embedded Tauri interaction evidence is retained per provider: [Alpaca](s02-alpaca-evidence.md), [Trading 212](s02-trading212-evidence.md), [Binance](s02-binance-evidence.md), [Bitget](s02-bitget-evidence.md). The final Bitget implementation bundle built and reopened its dedicated workspace; it includes every provider. Strict Clippy and formatting passed during that final implementation delivery.
- Combined public tests cover immutable provider/environment/workspace identity, exact state-version confirmation, permission changes, failure preservation, restart/recovery, disconnect/late completion, pagination and credential cleanup. Network transport tests use actual isolated TLS and reject redirect, timeout, quota, oversized body and provider-specific signing errors. Successful credentialed broker authentication and order execution are not inferred from these tests.

## Requirement audit

| S02-owned requirement | Evidence establishing completion |
|---|---|
| FR-008 | Production macOS NativeVault and all four explicitly executed Keychain round-trip/cleanup lifecycles; native entry and sentinel scans in provider evidence |
| FR-010, AC-005 | Fixed provider/environment definitions; successful read operations determine observed read capabilities; unsupported/malformed provider responses cannot produce a ready snapshot |
| FR-047, AC-036 | Public connection test → exact-version permission review → confirmation; unknown scope requires acknowledgement, dangerous scope blocks; seven UI flows |
| FR-048 | Provider-specific identity/currency/asset/position/order projections with exact decimals, native-asset distinctions and unavailable values; seven UI variants |
| FR-067, AC-051 | Backend field schema drives secure native input; two-field and three-field providers; ordinary IPC rejects secrets; native keyboard/validation/cancel evidence |
| FR-078, AC-062 | Binance restriction flags and Bitget authorities gate readiness; unknown introspection remains UNVERIFIED; permission/IP changes invalidate confirmation; Live remains DISARMED and execution BLOCKED |
| AC-004 | Distinct immutable environment selections and Keychain references; fixed hosts or Demo-only header; public, native-store and browser environment isolation |

These 11 exclusively S02 requirements may move to VERIFIED. Requirements also owned by S03/S05/S21/S22/S24/S32/S35 remain IN_PROGRESS. All six S02-contributing surfaces remain IN_PROGRESS because their other mapped owners (including S33 visual/interaction parity) are incomplete. Prototype QA FAILED/PARTIAL labels remain historical and unchanged.

## Serial review and remaining scope

Standards: PASS, no remaining actionable findings after sequential provider reviews and integration review. Provider-specific parsing/signing reuses the existing native store, command lifecycle, response boundary, exact decimal helpers and UI projections. No raw-secret renderer path, unsupported execution mutation, unused adapter framework or new Bitget dependency was introduced.

Spec: PASS for parent #5. Its four vertical slices and 15 user stories are supported by the combined protocol/native/browser evidence above. Last-sync preservation, explicit limitations, unknown/dangerous scope, connection review and recoverable cleanup are implemented. Local Paper is explicitly built-in and credential-free; its simulation engine remains assigned to S16. Later model routing, full onboarding, market metadata/FX, trading, private streams, reconciliation, Live authority and release gates remain open.

Next is S03, model gateway configuration and five-step onboarding, beginning with to-spec before ticketing or implementation. No dev→main PR is due until all 35 map items pass. Local combined browser evidence is `.artifacts/s02-integration/browser-results.json` (ignored, fixture evidence).
