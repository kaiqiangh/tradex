# S28 required FX observations — implementation checkpoint

Ticket: [读取所需 FX 路由与 Alpaca 汇率证据](https://github.com/kaiqiangh/tradex/issues/120), OPEN. Review baseline: dev@d7ea48d1e5a5c2a5f95cc3937b02aa0202bc53d4. Current changes are uncommitted, unreviewed and unpushed. Source hashes: implementation-checkpoint.json. No main PR or merge.

Implemented saved-source CAS selection/disconnect/reopen, schema35 migration, actual account/monetary/immutable-proposal route requirements, bounded exact-decimal latest rates, dual-age binding, source/account/workspace/clock/sequence cancellation, immutable currency risk/review capture and unavailable conversion guard. Legacy synthetic portfolio data cannot override an explicit FX choice, including after disconnect. Supported Spot BUY uses an observed matching quote-asset balance solely for the funding unit; portfolio fiat currency remains unknown. SELL does not acquire a BUY funding-conversion route. Local Paper/CANCEL semantics are separate.

## Recorded local verification

- Genuine RED/GREEN: selection, requirements, precision-preserving producer, identity/no key read, immutable intent pairs, portfolio fixture shield, captured consent/material, Spot BUY funding unit and SELL route independence. The corresponding `*-red.txt` / `*-green.txt` files retain actual failing and passing outcomes.
- Financial-source file checkpoint:31passed/3ignored before the subsequent Spot/Sell additions. This is not a final current-source full-suite receipt.
- Parser faults:18malformed, conflicting, time or size cases; current evidence retired without authority.
- Late-result races: source/account/workspace/clock/newer sequence; no Control Plane lock during external HTTP.
- Dual-age/reopen: provider expiry independently retires a recent receipt; restart keeps selection only.
- HTTPS boundary: actual disposable verified TLS, fixed GET pairs, no redirect follow, bounded body and12second timeout.
- Access/quota:401/403/429 classified independently; actual2second Retry-After is respected before the next request. Read200 still does not qualify conversion.
- Schema34 selected source survives schema35 metadata migration; workspace regression file8passed.
- Current full npm check passed: schema/build,17 Node tests, all ordinary Rust tests and requirement traceability. Typecheck, formatting and diff whitespace passed. Generated schema/types include the new commands and captured review contract.

The `*-attempt.txt` files record unsuccessful compilation or external-fixture setup/timestamp assertions; they are not purposeful product REDs or passing receipts. A synthetic response timestamp is deliberately prior to the millisecond receipt in the quota fixture; production future-time rejection is unchanged.

## Still required before closure

Human completion of current native Keychain authentication, followed by an explicit saved-source hosted FX reread and sanitized current/expired or exact-denial evidence; final scope audit; serial independent Standards then Spec reviews; scoped commit, normal dev push and exact source/evidence receipt. Review has not started while implementation's native read verification remains pending. No issue was closed, no commit/push/PR was made, and main was untouched.

Current full checks passed:17Node/385ordinaryRust,424integrationRust and23Gateway tests;39existing ignored tests remain unverified. Ordinary desktop build passed. The combined full-library invocation with order-gateway-runtime failed two pre-existing ordinary-application no-write assertions because that feature intentionally enables Gateway write routes; the unsuccessful command is retained as integration-gateway-feature-scope-attempt.txt. Correctly scoped ordinary integration and Gateway target checks both passed; no assertion or production route was relaxed.

No hosted FX current/expired/access-denied PASS, CI, review or delivery is claimed. Transaction-grade source qualification, explicit financial conversion/rounding, exact broker funding/costs and complete monetary evidence remain unavailable. Parent acceptance issues#117/#116/#115/#113/map#1 remain OPEN. No Arm, financial approval, reservation, order/cancel provider mutation or funds conversion occurred.

## Browser and contract regression checkpoint

Real Rust UI passed Settings and disabled pre-arm review layouts at390/768/1280px, keyboard selection, exact numeric context, proposal-scoped reverse route, retained expiry, a real external-fixture403 read with sanitized error/Reload, reopen and disconnect. Permissions remained UNVERIFIED, Live DISARMED, approval count0, no provider financial mutation. Browser console errors/warnings0. browser-ui.json and browser-fx-rates.png are fixture evidence only. UI verification exposed and fixed the Workspace wire maximum remaining at34 after storage35 migration, and missing captured FX in the pre-arm window. Existing projection/watchlist current-schema expectations now35; their earlier failure receipts remain *-attempt logs. The schema-ui-red/green logs used a temporary focused contract assertion; equivalent ongoing coverage lives in tests/projection.test.ts. Caller assertion coverage passed before vault/HTTP work. Ordinary native release build passed. The locally signed validation bundle has the current main executable and unchanged Gateway bytes matching its compiled pin; actual parent/child images and ownership were checked. Saved-source FX Refresh returned UNAVAILABLE with no first receipt and controls reenabled; process sampling confirms protected Keychain and the owned financial-source vault worker. Human authentication was requested; hosted access/rate freshness remain unverified. No native current/expired/denial PASS is inferred from this attempt.

## Maximum-decimal UI boundary and acceptance audit

The external FX HTTP fixture and public browser scenario now exercise the maximum64character exact bid/ask strings. The real Rust-backed source and captured review passed390/768/1280px with complete numeric text, wrapped390px screenshot, no console errors/warnings, immutable expiry and actual external-fixture403 error/Reload. Initial run passed; no product RED is claimed. Existing global dd wrapping already handles this boundary; no production CSS/runtime change was needed. browser-max64.json/browser-max64-390.png retain the evidence. acceptance-audit.md maps all12ACs to code, receipts and unresolved gates. Native authentication was freshly rechecked on the live ordinary process and remains protected. No new hosted refresh, review, commit, push or issue closure occurred.

## Current ordinary hosted denial and review handoff

Fresh2026-10-04T09:03 process sampling found the protected/source-vault workers ended. Revalidated time in the ordinary native UI and explicitly refreshed the saved source. It returned UNAVAILABLE with the exact provider-denied-access classification, no first receipt or rate, and reenabled Refresh/Reload. native-hosted-fx.json archives only safe source metadata; reviewed HTTP403 mapping is explicitly an inference, not an archived raw response. Account-wide permissions, current rate/age and transaction-grade source qualification remain unverified. No key/identifier/balance/holding/body or provider financial mutation was archived/performed. AC11 negative hosted evidence is now complete for this read-only slice; the earlier timeout/authentication receipts remain historical unsuccessful attempts. Implementation is ready for serial independent Standards then Spec review against d7ea48d. Issue120 and acceptance parents remain OPEN until scoped delivery/relevant gates, respectively.
