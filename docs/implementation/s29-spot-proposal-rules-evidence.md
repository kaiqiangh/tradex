# S29.4 — Proposal static Spot rules and required references

Ticket [#125](https://github.com/kaiqiangh/tradex/issues/125), parent [#121](https://github.com/kaiqiangh/tradex/issues/121). [Specification](s29-spot-proposal-rules-spec.md), [中文](s29-spot-proposal-rules-evidence_zh.md). Review baseline `2368e22122b2028e92f4c3352c2f08fe80e1344a`. Delivered runtime source is `ecc2860b197e3806d14f1eb9871c52246bd8378f` on dev/origin/dev. Checks/reviews ran against a frozen worktree above planning HEAD `1d24523158674d2763d50bee49150e8ff9d808d6`; every committed runtime/test/generated-file byte matches that checkpoint; exact runtime/test/generated-file hashes are in [implementation checkpoint](evidence/s29-spot-proposal-rules/implementation-checkpoint.json). All implementation validation and serial review gates PASS. Ticket #125 is CLOSED; [tracker receipt](https://github.com/kaiqiangh/tradex/issues/125#issuecomment-6075002255) and [exact-byte source/remote delivery](evidence/s29-spot-proposal-rules/source-delivery.json) record the final handoff.

## Delivered behavior

`trade.spot_rules.get` and explicit CAS `trade.spot_rules.refresh` derive saved immutable Proposal intent, exact selected ordinary Binance Live account, canonical BTC/ETH, BASE/QUOTE and source/material/session/time/policy bindings. Renderer payloads cannot supply host, symbol, price or authority. Supported BASE Limit GTC/IOC/FOK, BASE Market DAY and provider-permitted BUY QUOTE Market preserve the original intent. Exact string decimal comparisons, modulo and multiplication cover supported price/quantity/percentage/notional and single-order native-asset limits. Only documented PRICE_FILTER zero components disable their own fields. No absent optional notional filter or resulting BASE quantity is invented.

Scoped/origin-preserving per-rule PASS/REJECT/UNAVAILABLE feeds INSTRUMENT_RULES and captured risk input digests. Known violations reject. Missing dynamic order counts, position capacity, execution PRICE_RANGE and unknown active schemas remain named obligations. Complete static collection can PASS this owning check only if every actually applicable obligation is established; MARKET_DATA_USE, quotes, account health/permissions, fees/FX and financial authority remain independent.

Only genuine required public references are read through fixed ordinary P3 GET routes without account authentication. Primary non-null reference takes precedence; only well-formed explicit null permits original last trade for zero interval or average with its original matching interval. Error/no-reference/invalid/status responses never select fallback. Provider timestamps, first receipt and monotonic age stay separate; repeated identical reads cannot renew first receipt. Future/stale/wrong interval/binding/late/deadline/cooldown failures cannot qualify a Proposal. References remain transient; persisted decisions redact raw prices and keep digests/captured outcomes. Reopen never restores current reference eligibility.

Actual Trade exposes exact identity, units, applicability, purposes, original interval/time versus first receipt, explicit refresh/recovery and unresolved obligations. Captured history and pre-arm/approval reviews explain saved assessments without a refresh control. Source replacement cannot rewrite history. Keyboard-operated 1280/768/390 layouts have no horizontal overflow.

## Evidence seam and boundaries

Public React → actual typed Rust Control Plane → owned temporary SQLite/outbox → external fake HTTP/vault. Accounts, source selections and Proposals use public operations. No positive Control Plane financial/reference snapshot seed. HTTP fixtures change only external provider responses. UI verification uses an owned integration host on port1427, `TRADEX_SOURCE_ONLY_FIXTURE=1` and external rule HTTP fixtures; the user's existing desktop/workspace/accounts are untouched. Existing Hot/Gateway regressions have their own historical fixture boundaries; they are not S29 positive financial acceptance.

Each independent rule and Binance Hot scenario owns a serial isolated application process. Production IP/UID limits remain unchanged within a scenario. These runs do not establish aggregate same-process exchange-IP use across every case. Dedicated cross-account/workspace quota and 418/429 shared cooldown scenarios verify the shared production boundaries without quota resets.

## Verification ledger

| Gate | Current result | Evidence |
|---|---|---|
| Unified `npm run check` | PASS: 436 Rust, 17 Node, 39 existing ignored; schema/build and 203 requirements / 70 screens / 13 QA / 23 baseline files | [log](evidence/s29-spot-proposal-rules/final-unified-check.txt) |
| Public rule suite | PASS: 38 independent scenarios included in unified check | Same log |
| Binance Hot | PASS: 49 serial isolated external scenarios | [log](evidence/s29-spot-proposal-rules/binance-hot-final.txt) |
| Stock Hot | PASS: 19 external scenarios | [log](evidence/s29-spot-proposal-rules/stock-hot-final.txt) |
| Gateway runtime | PASS: 23 local child/fake-provider scenarios | [log](evidence/s29-spot-proposal-rules/gateway-final.txt) |
| Ordinary desktop build/pin | PASS; compiled application contains its pinned Gateway digest | [build](evidence/s29-spot-proposal-rules/desktop-final.txt), [pairing](evidence/s29-spot-proposal-rules/desktop-build-inputs.json) |
| Final rebuilt React→Rust UI | PASS: keyboard1280/768/390, current rejection/frozen history, captured pre-arm review, no Arm/PLACE | [current](evidence/s29-spot-proposal-rules/ui-final.json), [pre-arm](evidence/s29-spot-proposal-rules/ui-final-review.json), [build binding](evidence/s29-spot-proposal-rules/ui-build-inputs.json), [current image](evidence/s29-spot-proposal-rules/final-current.png), [captured image](evidence/s29-spot-proposal-rules/final-capture.png), [390 image](evidence/s29-spot-proposal-rules/final-390.png) |
| Serial Standards then Spec | PASS / PASS: 0 hard Standards breaches, 1 nonblocking duplication heuristic; 0 Spec findings | [reports](evidence/s29-spot-proposal-rules/review-final.md) |
| Delivered SHA/remote/tracker | PASS: source `ecc2860` matches all19 reviewed/tested files, remote dev verified, #125 CLOSED; GitHub workflows0 (no CI PASS claim) | [delivery](evidence/s29-spot-proposal-rules/source-delivery.json), [closure](evidence/s29-spot-proposal-rules/closure-receipt.json) |

## RED/GREEN and retained attempts

Genuine observable public RED/GREEN logs cover static evaluation/risk rejection, missing coverage, required primary/null fallback, Market notional/asset and supported forms, retirement/units, total reference deadline, immutable first receipt/expiry, last-trade/mixed intervals, future times, shared 418 ban, precise unresolved reasons, absent optional notional, risk rejection reason and interval mismatch. Logs retain their original outputs under [evidence directory](evidence/s29-spot-proposal-rules/). UI RED covered missing generated reply union, narrow overflow and absent pre-arm captured panel; GREEN used the real React/Rust path. Plain public regression guards are labelled regressions rather than fabricated RED.

Initial `full-check.txt` failed an old fixed-route assertion; `full-check-final.txt` subsequently passed 433 Rust/17 Node but failed shifted PRD line pointers. The final unified run fixes the assertion and updates only source line pointers, preserving all 203 requirement statuses/evidence. Initial parallel whole-rule run exhausted the unchanged shared IP limit; the final suite uses serial independent process ownership, with dedicated same-process quota tests retained. One isolated harness setup attempt was intentionally stopped; it is not a passed run. Early GREEN compile/setup attempts, unsupported 30-fraction draft (`ORDER_DECIMAL_INVALID`) and reopen `WORKSPACE_BUSY` fixture precondition errors remain visible. The corrected guards use accepted 18-fraction intent and drop the old workspace owner before reopen; neither setup error is claimed as a product RED.

## Unfulfilled gates

Ordinary build is compilation/pairing proof, not native UI, Keychain, hosted provider, financial mutation or licence proof. No real orders, credentials, rights or licence acceptance were used. Parent S29 dynamic capacities, fees/FX, authenticated immediate preflight, private lifecycle and real provider financial acceptance remain OPEN. S28 financial dependencies and Trading 212 complete permissions, S17 real Paper happy lifecycle and S27 actual OS sleep/wake remain OPEN. Physical sleep/wake was skipped for scheduling, not waived. Global requirements are not promoted from source tests. Prototype code and main remain unchanged; no main PR/merge is authorized until the full map is verified on dev.

Stored text logs normalize only trailing whitespace and surplus final blank lines; command content/outcomes are unchanged. Raw and stored SHA-256 are retained in [log format](evidence/s29-spot-proposal-rules/log-format.json).
