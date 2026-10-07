# S28 production financial prerequisites

Date: 2026-10-01. Audit baseline: `aa43091ea2eef732b5865b163cbeee3cac3e03c9`.
Status: **READ_ONLY_CHILDREN_DELIVERED; positive financial acceptance remains unsupported/unverified**.
Specification: [补齐 S28 生产金融前置证据](https://github.com/kaiqiangh/tradex/issues/117).
Chinese pair: [金融前置依赖](s28-financial-prerequisites_zh.md).

The user explicitly authorized keeping [接入已配置 Alpaca 报价与 Hot 订阅](https://github.com/kaiqiangh/tradex/issues/116) OPEN while completing financial prerequisites serially. This changes sequencing only. Its positive approval/Prepare/dispatch criterion and the [Trading 212 Live acceptance parent](https://github.com/kaiqiangh/tradex/issues/113) remain unfulfilled.

## Current return-to-acceptance audit — 2026-10-07

Current source baseline: `dev@64b21a3df0ae56a4410604902ff14b636f6837aa`. The calendar, known company events/exact-account metadata and required-route/read-only FX implementation children are CLOSED with their own reviewed evidence. This does not complete this prerequisite specification or the quote/Trading212 positive acceptance parents.

| Current gate | Authoritative evidence / outcome | Required next evidence |
|---|---|---|
| Calendar | Calendar child118 delivered actual native read/expiry; current session requires explicit fresh refresh before consumption. | Current bound trusted coverage/session, not historical receipt. |
| Known actions / exact-account directory | Child119 delivered bounded reads and expiry. Source capabilities keep complete action coverage, current halts/account tradability and historical adjustment separate. | Complete prospective actions/adjustment and exact current unrestricted execution-account tradability/identity, not query exhaustion or directory membership. |
| Required FX | Child120 delivered actual balance/intent route derivation, exact original tokens/times and frozen consent. Current2026-10-07 native FX read returned the sanitized endpoint-denial reason, no rate or first receipt; HTTP403 is mapping inference. | Qualified permitted transaction use, explicit conservative direction/rounding, exact broker funding/costs and complete monetary inputs where actually required. Same-currency and protective CANCEL retain operation-specific semantics. |
| Quote source | Current ordinary native explicitly selected IEX: authenticated technical quote access verified; Hot authentication and subscription acknowledged, no actual current quote observed at sampling. | Actual fresh eligible quote/coverage/depth, and any required SIP entitlement. Acknowledgements and IEX connectivity are insufficient. |
| Trading212 account / permissions | The existing Live row was FAILED/STALE with UNVERIFIED scope and DISARMED. Explicit saved-key read-only Refresh is currently awaiting human macOS Keychain authentication. | Actual current account recovery; full authoritative permission evidence remains independently missing even if recovery succeeds. |
| Positive issued approval / Prepare / dispatch | Still unverified. Current normative scope guard rejects non-VERIFIED permissions; ordinary risk reports unavailable complete-action and exact-tradability/identity inputs; required unqualified FX remains unavailable. | All applicable production prerequisites before positive issued-consent/Prepare/Gateway acceptance. A fixture or disabled read-only preview cannot stand in for it. |

The complete Trading212 OpenAPI was freshly fetched2026-10-07:22operations/17paths, unchanged SHA256 `a272f70a713fa9f2f906e9b4be12136c9d5d63d7f9f48562e939aa35c4fc041d`; no full key/IP-scope introspection, current halt/tradability or FX quotation/conversion operation appears in that published contract. This is published-contract scope, not proof concerning undocumented interfaces. [Official API](https://docs.trading212.com/api) also retains primary-account-currency execution. [Alpaca corporate actions](https://docs.alpaca.markets/us/reference/corporateactions-1) still warns of upstream/processing delays and describes processing-date rather than prospective completeness. [Alpaca market-data plans](https://docs.alpaca.markets/us/docs/about-market-data-api) distinguish IEX coverage from all-US-exchange coverage.

Current20 runtime-input hashes, signed main/Gateway bytes and actual parent-child ownership match the validated ordinary build; no new build or test rerun is claimed for this unchanged-code audit. No CI workflow exists in this checkout, and no hosted CI PASS is claimed. Only safe source status, schema operation metadata and build/session booleans are retained in [return evidence](evidence/s28-return/); no secrets, account identifiers, balances, holdings, quote prices or raw provider bodies are archived. Keychain completion and subsequent account read are pending. No Arm, financial approval, Prepare, provider mutation or main action was performed. Existing S17 and actual OS sleep/wake skips remain unverified. The inventory and dated notes below describe the original audit and historical checkpoints.

## Dependency inventory

| Prerequisite | Production evidence at audit baseline | Required outcome / boundary |
|---|---|---|
| Complete API permission scope | Trading 212 read adapter detects three read capabilities; scope remains `UNVERIFIED`. | RevC requires `VERIFIED` for Live Arm. Published public API supplies no full key/IP-scope introspection. Record external capability gap; do not manufacture authority. |
| Trusted time | Existing process-scoped TimeService is consumed by financial gates. | Revalidate after discontinuity/reopen; untrusted time blocks every new observation. Physical OS sleep/wake acceptance remains separately scheduling-skipped. |
| XNAS market calendar | S08 supplies fail-closed projections and isolated fixtures; ordinary build has no producer. | Explicit saved Alpaca credential selection, fixed host, authenticated v3 XNAS/UTC response, bounded coverage, first receipt, material version, short-lived risk/approval/dispatch consumption. |
| Corporate actions / adjustment | Ordinary build returns unavailable/unknown. | Bounded canonical event provenance; processing-delay and completeness limits retained. Event metadata does not prove historical adjustment or absence of pending actions. |
| Trading 212 exact instrument state | Canonical AAPL/MSFT mappings exist; current execution tradability producer does not. | Evidence must match selected execution provider/account, provider symbol, canonical instrument and venue. Alpaca asset state or generic metadata membership is insufficient proof for Trading 212. |
| Required FX | ECB informational daily rates are available to existing valuation code; they are not transaction-grade execution quotes. | Derive required routes from actual intent/policy currencies; verify provider quality and both timestamps within execution freshness. Same-currency arithmetic remains exact without inventing a conversion. |
| Eligible production quotes | Configured selected-feed producer and Hot implementation delivered; actual native IEX access observed. | SIP entitlement/coverage, depth, freshness and current source binding remain separate; IEX connectivity is not eligible consolidated coverage. Existing quote ticket remains OPEN. |
| Positive financial path | Existing local Gateway/financial fixtures do not prove these production prerequisites. | Return to quote/Trading 212 acceptance only after genuine prerequisites; real money requires concrete separate order consent. No hidden authority seeding. |

## Current Trading 212 permission result

The [official API reference](https://docs.trading212.com/api) and [complete public OpenAPI bundle](https://docs.trading212.com/_bundle/api.json?download=) were fetched on 2026-10-01. Bundle SHA-256:

`a272f70a713fa9f2f906e9b4be12136c9d5d63d7f9f48562e939aa35c4fc041d`

The bundle contains **22 HTTP operations across 17 paths**. None reports full key permissions or IP scope. Its two authentication schemes are Basic key/secret authentication and legacy API-key header authentication, not permission introspection. This is a conclusion from the complete current published contract; it does not claim unpublished provider interfaces do not exist.

Successful summary/position/order reads show those capabilities were usable for those requests. They cannot prove place/cancel permission, absence of forbidden permissions, or complete scope. The production adapter therefore correctly leaves `PermissionReview.scope` as `UNVERIFIED`. RevC UI Spec Account Permissions and Backend §41.2 require `VERIFIED` for Arm; acknowledging an unknown review cannot change that.

Consequences:

- Entering another key, refreshing the account or manually reviewing a screenshot cannot establish `VERIFIED` under the current contract.
- No order/cancel probe, manual attestation endpoint, synthetic state update or automatic Arm is an acceptable implementation workaround.
- Implementable data prerequisites can proceed independently. Trading 212 positive financial acceptance stays blocked until an authoritative provider capability exists or the user explicitly approves a product-contract revision through the prescribed workflow. No such revision is approved here.

Local source evidence: the Trading 212 account adapter constructs only `account.read`, `positions.read`, `orders.read`; the Control Plane health blocker rejects any non-`VERIFIED` scope. The public wire rejects caller-supplied permission assertions.

## First implementable slice: calendar

The selected S06 calendar source has a current [MIC-specific v3 endpoint](https://docs.alpaca.markets/us/reference/calendar-2). Its accompanying `.md` OpenAPI specifies separate Paper and Live servers, required response `market` and `calendar`, required day `date`, `core_start`, `core_end`, optional pre/post/lunch bounds, and an explicit `timezone=UTC` parameter. `XNAS` is an allowed market identifier.

Use an explicitly selected eligible saved Alpaca Paper credential and its fixed Paper host for the initial slice. This is a read-only calendar request, independent of quote-feed entitlement and broker execution. Never try alternate hosts or keys automatically. Borrowed account keys remain owned by the account.

The producer must validate identity, interval completeness/order and coverage; preserve first receipt; distinguish scheduled timestamps from unavailable provider observation time; retire data after at most 30 trusted seconds for execution; and invalidate on credential/source/session/workspace changes or failed refresh. Determine OPEN/CLOSED/EXTENDED_HOURS from authenticated UTC boundaries. No local weekday/DST schedule or injected snapshot may substitute.

OD-005 capabilities must be independently typed and displayed: calendar availability cannot imply corporate-action, halt, adjustment or Live readiness. Inspect the current market-session risk consumer during this slice: it currently checks source/status/version/time confidence without an explicit receipt-age check. A short-lived producer must be coupled to fail-closed consumer freshness; periodically reprojecting a view must not renew receipt time.

## Subsequent decisions

2026-10-03: the paired [company-event / exact broker metadata research](s28-actions-tradability-research.md) refreshes the complete official contracts. The [second prerequisite specification](https://github.com/kaiqiangh/tradex/issues/117#issuecomment-5966487911) is published and the single [read-only implementation ticket](https://github.com/kaiqiangh/tradex/issues/119) is claimed. Date-only event semantics, pagination versus prospective completeness,10minute broker metadata quality, exact account bindings and listing versus broker execution venue remain distinct. This is implementation work still OPEN, not provider-hosted proof or a relaxation of the parent financial requirements. Required FX has not started.

The [current Alpaca corporate-actions contract](https://docs.alpaca.markets/us/reference/corporateactions-1) warns of provider/processing delays. Its date filters use `process_date`, and default `complete` quality can include already-processed records that would otherwise be incomplete. An empty response is not a proof that no new action was announced. Subsequent event/tradability implementation must respect those limits and investigate the exact broker contract before claiming a positive gate.

FX work begins with actual required routes, not a generic new rate service. Existing ECB data cannot be promoted from informational daily reference quality to verified execution quality. A new source/entitlement decision is needed if current supported providers lack the required observation contract.

The proposed sequence is calendar → corporate-action/exact broker tradability → required execution FX → return to existing positive financial acceptance. It is a **serial work order**, not a fabricated chain of technical blocking edges. Subsequent uncertain source choices remain explicit until researched.

## Evidence boundaries

This audit performed public documentation reads and local source inspection only. No vault read, real account refresh, Arm, approval, reservation, provider mutation or financial execution was performed. No runtime PASS is claimed. Existing S17 and physical S27 deferrals remain OPEN. Final map completion and dev→main PR are not authorized by this audit result.

Implementation follow-up: approved calendar child [#118](https://github.com/kaiqiangh/tradex/issues/118) has passed its bounded producer/UI/native calendar verification and final Standards/Spec reviews. Its verification is recorded separately in [calendar evidence](s28-calendar-evidence.md); the audit-only boundary above describes the initial research, not subsequent implementation work. No financial permission-contract revision is approved.

2026-10-03 implementation checkpoint: the company-event/exact-account metadata slice has passed automated checks, Rust-backed UI and ordinary hosted reads with saved Alpaca Paper / Trading 212 Live credentials. Natural expiry preserves original receipts/material versions; complete coverage, adjustment, canonical identity, current tradability/halts and permissions remain unsupported/unverified. Independent review and final dev delivery are pending; child119 and all acceptance parents remain OPEN. See [paired implementation evidence](s28-actions-evidence.md). Required FX has not started.
