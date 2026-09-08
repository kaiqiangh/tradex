# S02 — 安全连接并查看提供方账户

状态：Spec [#5](https://github.com/kaiqiangh/tradex/issues/5) 已发布；实现票 [#6](https://github.com/kaiqiangh/tradex/issues/6) → [#7](https://github.com/kaiqiangh/tradex/issues/7) → [#8](https://github.com/kaiqiangh/tradex/issues/8) → [#9](https://github.com/kaiqiangh/tradex/issues/9) 严格串行。当前 #6 已在 `dev@c36296ba0d121e54703555430542bb573146b2c4` 完成并通过公共协议、真实 HTTPS 边界、Keychain、浏览器、原生窗口与串行双轴审查；详见 [Alpaca evidence](s02-alpaca-evidence.md)。#7 也已在 `dev@5f487ec10c48eea6616dac91472abe38a8297b19` 完成并验收，见 [Trading 212 evidence](s02-trading212-evidence.md)。下一张票为 #8；S02 未完成。起点 `dev@62c6ec0ea50d5ed67c8f784256b7737e8667b8e7`。沿用 S01 已确认的公共测试边界和逐票实现、验证、串行双轴审查约定。

## Problem Statement

用户能恢复本地工作区，但尚不能连接自己的 broker/exchange 账户。统一密钥表单会遗漏提供方差异；仅凭认证成功标记 Ready 会隐藏未知或危险权限。账户、环境和凭据必须形成可恢复且不可混用的连接身份。

## Solution

从 Accounts 或 Settings 选择提供方及环境，依据后端 schema 在 macOS 原生安全输入界面录入凭据，进行只读认证/账户探测，再审阅实际能力、权限及限制。用户确认后保存连接。账户详情显示原币余额、持仓、未完成订单、同步时间和分开的健康状态；可以手动刷新、重新测试或断开。秘密只保存在 OS Keychain，由受信提供方层读取，任何 renderer/IPC/result/event/SQLite 均没有原始凭据。

## User Stories

1. As a user, I want each provider's exact fields and environment, so that I can supply the right credentials without guessing.
2. As a user, I want secure native entry with clear field labels, help, validation and cancellation, so that secrets do not enter the webview.
3. As a user, I want authentication and read capabilities tested before confirmation, so that a saved connection is backed by observed results.
4. As a user, I want requested permissions separated from detected permissions, so that successful reads do not imply trading authority.
5. As a user, I want unknown scope labeled UNVERIFIED and explicitly acknowledged, so that its limitation remains visible after connecting.
6. As a user, I want forbidden or unsupported permissions to block Live readiness, so that acknowledgement cannot bypass the safety gate.
7. As a user, I want Demo/Testnet and Live represented as immutable distinct connections, so that refresh never changes the destination of my credentials.
8. As a user, I want provider-specific account identity, balances, positions and open orders, so that I can inspect what the provider actually returned.
9. As a user, I want last successful sync retained when a refresh fails, with stale/error status, so that old data cannot appear freshly verified.
10. As a user, I want authentication, connection, private stream, reconciliation and execution eligibility shown separately, so that a REST response cannot imply a healthy stream or completed reconciliation.
11. As a user, I want safe disconnect and credential cleanup, so that the app stops accessing the connection without canceling external orders.
12. As a returning user, I want stable persisted account identity and reference health checks, so that missing Keychain items lead to reconnection instead of fake success.
13. As a user, I want clear rate-limit, offline, invalid-key, unsupported-account and malformed-response failures without secret diagnostics, so that I can recover safely.
14. As a keyboard or narrow-window user, I want configure, review, refresh and disconnect reachable with visible focus, so that account management remains usable.
15. As a user, I want Local Paper clearly identified as built-in and credential-free, so that it cannot be confused with a broker account or invented broker balances.

## Implementation Decisions

- Four sequential vertical tickets, one provider family each. Alpaca establishes the complete native-entry → Keychain → read-only adapter → persisted connection → review/detail flow. Later tickets reuse that path with their actual schemas and response contracts; no prebuilt empty adapter framework.
- Define new exact payloads in bilingual Backend ARD §41 and events in §42 before code. Provider definitions, schema, connect (test/review confirmation), disconnect, probe, permissions, account list/get/refresh use canonical operation names. Renderer never submits a raw key or a caller-chosen credential reference. Unknown fields, environments, IDs and stale versions fail before side effects.
- Secret fields include API key identifiers as well as secrets/passphrases. Use platform secure text fields and Keychain APIs. Native capture and privileged signing own secret values; diagnostics expose only stable sanitized errors. HTTP destinations and read operations are fixed by the selected adapter, with TLS, redirects disabled, bounded responses and deadlines.
- Authentication creates a review-required connection. Its credential reference and non-secret observation may be durable while review is pending, but it is not confirmed/ready until explicit confirmation. Unverified permission acknowledgement binds to that exact review version. A changed scope requires fresh review. Blocked scope cannot be cleared by a checkbox.
- Non-secret connection state and per-aggregate events commit atomically. Workspace/provider/environment/remote-account identity are immutable. A refresh must return the same remote identity. A workspace switch, disconnect or competing change invalidates an in-flight completion. Provider I/O and native dialogs run outside the control-plane state lock so local runtime and account queries remain available.
- Credentials are individually scoped by workspace, connection and environment. Failed/cancelled attempts and disconnect clean up their own Keychain items; failed cleanup stays visible/retryable. No filesystem credential fallback. Persisted projections never contain response dumps or arbitrary provider messages.
- Decimal source values retain exact decimal strings; missing values remain unavailable. Queries follow documented pagination to a bounded complete result or report incomplete data explicitly. A partial/error read cannot create a complete snapshot or fresh successful-sync marker.
- Separate supported provider capabilities, detected credential permissions, and current TradeX execution eligibility. Every Live account stays DISARMED; no policy, stream or reconciliation success is invented. S02 performs no order/transfer/withdrawal or account-setting mutation.
- Restore leaves observations visibly stale until refreshed and verifies credential references. Disconnect removes local access and preserves non-secret history; it does not imply provider key revocation or cancel external orders.
- Synchronize the missing Chinese PRD §26.1 field/permission rules and regenerate the product manifest. Existing prototype defect labels remain unchanged.

## Testing Decisions

- Reuse the real public command boundary with isolated on-disk SQLite and actual outbox/reopen checks. Substitute only external HTTP responses and native secret-entry input for deterministic contract/fault cases; the production parser, state transition and persistence stay real.
- Test the real macOS native input and OS Keychain write/read/delete with disposable synthetic secrets, including cancellation. Scan result/event/error/log/storage corpora for those exact sentinels. A browser fixture is not Keychain/Tauri proof.
- Authentication success, pending review, required acknowledgement, confirmation, refresh, stale version, wrong identity/environment/workspace, disconnect and reopen must work through the public lifecycle. Simulate auth failure, timeout, redirect, overlarge/malformed JSON, partial pagination, permission downgrade, deleted Keychain reference and persistence failure.
- Verify dangerous permissions are still blocking after acknowledgement; missing introspection is never converted to a verified scope. Successful reads never grant execution capability or arming.
- Exercise Accounts/Settings flow, permission review and details at desktop/768/390 widths and by keyboard in the actual UI; native dialog cancellation returns focus to its trigger.
- Provider contract tests and OS integration evidence are recorded separately from credentialed provider acceptance. Real sandbox order lifecycle is owned by S17–S20; real Live execution acceptance by S28–S30. Missing credentials cannot be represented as successful provider authentication.
- Capture the actual dev SHA before each ticket's first implementation change; review Standards then Spec serially after its checks, fix confirmed findings, then commit/resolve before the next ticket.

## Out of Scope

Order placement/cancellation, simulation, stream/reconciliation engines, arming/approval/reservations/Gateway, FX valuation, market-data source selection, model credentials and complete onboarding are assigned to later map items. The account view explains their current availability; S02 does not claim those requirements complete. Alpaca Live and derivatives are outside this baseline.

## Further Notes

Owned requirements: FR-008/010/047/048/067/078; AC-004/005/036/051/062. Contributions to FR-044, AC-020/029/038, NFR-004, SEC-001/008/009 remain partial until all their mapped slices pass. Screens A2/A3/E1/E2/J1/J5 also have later cross-feature owners.

## Sequential tickets

1. **安全连接 Alpaca Paper 并审阅账户** — blocked by completed S01. Complete shared flow plus Alpaca Paper reads; Local Paper is shown as built-in without simulated broker state.
2. **隔离连接 Trading 212 Demo 与 Live 账户** — blocked by ticket 1's working connection flow. Add Basic-auth schema, environment isolation, supported account types, exact numeric handling and provider-specific views.
3. **连接 Binance Spot 并核验 API 权限** — blocked by ticket 1's working connection flow. Add signed Spot Testnet/Live reads, server-time checks, separate key-scope discovery, IP restriction and danger gates. Scheduler still waits for ticket 2's completion.
4. **连接 Bitget Spot 并处理 Demo 能力限制** — blocked by ticket 1's working connection flow. Add passphrase/signature, Classic Spot permission mapping, Demo header isolation and explicit unsupported/unavailable Demo behavior. Scheduler still waits for ticket 3's completion.

These are user-visible vertical slices; schema, database, adapter and UI are not separate horizontal tickets. No next-map item starts until all four are complete and S02 is reviewed.

## Official contract research — checked 2026-09-06

| Provider | Connection contract and limits | Primary sources |
|---|---|---|
| Alpaca Paper | Separate paper credentials and `paper-api.alpaca.markets`; headers `APCA-API-KEY-ID` / `APCA-API-SECRET-KEY`. Read account, positions, open orders; follow the documented `after_order_id` cursor in pages of 500. A 10,000-order bound or repeated IDs yields explicit incomplete data, never a truncated success. Account flags are not comprehensive API-key permission introspection; show that scope as UNVERIFIED. Never route this connection to Live. | [Authentication](https://docs.alpaca.markets/us/docs/authentication), [Account](https://docs.alpaca.markets/us/reference/getaccount-1), [Account state](https://docs.alpaca.markets/us/docs/working-with-account), [Order pagination](https://docs.alpaca.markets/us/reference/getallorders-1) |
| Trading 212 | Official v0 uses API key plus secret in HTTP Basic auth; separate demo/live hosts. Invest/Stocks ISA only, primary account currency, endpoint/account limits and optional IP restrictions. No documented scope-introspection operation was found in the public reference: UNVERIFIED rather than inferred verified. | [Public API](https://docs.trading212.com/) |
| Binance Spot | Separate Testnet/Live hosts and HMAC credentials. Account `canWithdraw` describes account capability, not key scope. Live key scope comes from `/sapi/v1/account/apiRestrictions`; transfer/withdrawal/margin-related flags feed the gate. Testnet has only `/api`, so `/sapi` discovery is unavailable and scope stays UNVERIFIED. | [Key permissions](https://developers.binance.com/en/docs/catalog/core-trading-wallet/api/rest-api/account), [Spot account](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/account), [Testnet limits](https://developers.binance.com/en/docs/products/spot/testnet/general-info) |
| Bitget Spot | Classic v2 uses API key, secret and passphrase; signed read requests. Account info returns `authorities` and `ips`. Map write transfer/withdrawal and unsupported margin/loan/derivative authorities explicitly, retain unknown scope as unverified. Demo requires separate Demo key and `paptrading: 1`; this general Demo documentation alone does not prove all Classic Spot endpoints available. Probe/failure must remain truthful; no Live fallback. | [Classic account](https://www.bitget.com/api-doc/classic/spot/account/Get-Account-Info), [Authentication](https://www.bitget.com/api-doc/classic/quickStart/intro), [Demo](https://www.bitget.com/api-doc/classic/demotrading/restapi) |

macOS uses [NSSecureTextField](https://developer.apple.com/documentation/appkit/nssecuretextfield) for secret capture and [Keychain password items](https://developer.apple.com/documentation/security/adding-a-password-to-the-keychain) for storage. Native Keychain calls run off the UI thread. Existing objc2/AppKit and reqwest dependencies are reused where applicable; any additional dependency must serve an actual required platform/crypto boundary.

## Trading 212 implementation clarification — 2026-09-07 / #7

Baseline: `907c510` on `dev`, after #6 closed. The current official [OpenAPI](https://docs.trading212.com/_bundle/api.yaml), [summary](https://docs.trading212.com/api/accounts/getaccountsummary.md), [positions](https://docs.trading212.com/api/positions/getpositions.md) and [pending orders](https://docs.trading212.com/api/orders/orders.md) refine this slice:

- Three fixed GETs: `/api/v0/equity/account/summary`, `/api/v0/equity/positions`, `/api/v0/equity/orders`; Demo and Live keep separate hosts and Keychain references. No history/order mutation or provider-supplied URL is followed. Positions and pending orders return arrays without pagination; an unexpected page envelope or more than 10,000 records fails incomplete/invalid. Historical cursor pagination is outside this slice.
- Summary/pending-order limits are 1 request per 5 seconds per broker account; positions allow 1 per second. Reads are user initiated, serial within each probe, with no automatic retry. Provider 429 remains explicit, preserves last successful data and requires a later manual retry. No key/IP rotation to evade account quotas.
- Use Basic `base64(API_KEY:API_SECRET)` with no newline; reject a colon in the key ID. Extend the existing secret-reflection check to the encoded authorization value as well as raw secrets.
- Preserve JSON number lexemes using the existing serde_json arbitrary-precision feature, including exact exponent expansion, without a binary-float conversion. Numerical IDs remain decimal strings beyond JavaScript's safe-integer range. Missing optional observations remain unavailable.
- Summary has no account subtype field. Successful use is limited by the provider to Invest/Stocks ISA, but the precise subtype is displayed as unavailable, not guessed. Unsupported-account HTTP rejection fails the connection. Permission/IP scope stays UNVERIFIED/UNKNOWN. Live stays DISARMED and execution BLOCKED.
- Show primary-currency cash available, reserved cash, cash in Pies and account value. Effective available remains unavailable until TradeX reservations exist. Positions retain instrument-price currency separately from wallet-value currency. Pending quantity and value orders preserve their respective filled units; never substitute a missing quantity with zero.
- Reuse the established public lifecycle/native/Keychain seam. Add Demo/Live isolation, exact Basic bytes, exponent/large-ID precision, missing currency, value-order, unobservable type, hostile reflection/page and 429 cases. Verify both provider selections and restored account details in the real UI, then Standards and Spec serially.
