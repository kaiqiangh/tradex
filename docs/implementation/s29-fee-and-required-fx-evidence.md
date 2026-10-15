# S29.9 fee and required execution-FX source evidence

[中文](s29-fee-and-required-fx-evidence_zh.md). Source commit `7b99f8e`, paired contracts `00d4f14`, fixed implementation/review baseline `1f8e231de3fe67aec16db1ae7392f34981617330`. Specification #130 and implementation #131 are CLOSED with every current-slice AC checked and read back. Source/evidence delivery `2c7dad1540358b00dc3d33dbb39e3c84ea68be9d` matches remote dev. Final acceptance is PASS for the bounded source/refusal slice. [Implementation resolution](https://github.com/kaiqiangh/tradex/issues/131#issuecomment-6097896018); [specification resolution](https://github.com/kaiqiangh/tradex/issues/130#issuecomment-6097897385).

This zero-new-read source slice reports original account maker/taker/buyer/seller rates, exact maker/taker comparison basis, only intent-policy and BUY funding routes, bound versions and one owning review blocker. Missing commission is described honestly; unknown extension obligations remain intact with bounded names. Current fee currency/origin UNKNOWN, fee/conversion amounts absent, and required USDT→workspace-base unsupported. Present delivered read-only directional FX is UNQUALIFIED without execution quality/cost; identity needs no rate. The comparison is not an all-in fee estimate. BOTH complete symbol/side fees and charging currency AND required USDT execution FX remain mandatory owning work; this refusal is no full-goal waiver or provider-capability absence claim.

| #131 AC | Evidence |
|---|---|
| 1 | Rust/shared contract; schema check; actual UI decoder |
| 2,9 | original projection and public commission extension cases; extension-name RED/GREEN |
| 3 | owning public route scope; excludes portfolio purposes; BUY-only funding |
| 4 | public declared-basis lexical equality/maker/taker cases, no float |
| 5,6 | public route/fee facts, single additive blocker, unchanged activation predicate |
| 7 | public actual commission-change digest, refusal/no approval or partial state; inherited issued-approval missing-FX Prepare test unchanged |
| 8 | defensive derivation-only contract cases, explicitly no public qualification |
| 10 | actual current/captured/pre-arm UI and immutable refresh/reopen captures; final frozen UI PASS |
| 11 | paired contracts/plan/map, serial independent reviews; exact remote/tracker handoff verified |

#130 AC1–7 map to the contract/projection/identity/route/basis/blocker/refusal/digest rows above; AC8 maps to actual UI/capture; AC9 to explicit reachability; AC10 to exact final delivery. A current source statement does not establish complete fee or execution-FX qualification.

[Final checks](evidence/s29-fee-and-required-fx/checks-review-final.json): all8 exit0,169 frozen source/build inputs matching source commit;510 Rust passed/0failed/39pre-existing ignored across34 result suites,17Node,49BinanceHot,19StockHot,23Gateway, schema/frontend and203-requirement/70-screen/13-QA/23-file traceability. Traceability verifies inventory, not runtime behavior. [Independent serial review](evidence/s29-fee-and-required-fx/code-review-final.md): Standards and Spec final PASS,0 remaining actionable/material findings. Initial Spec's two defects were corrected; reviewer ran no validation.

[TDD record](evidence/s29-fee-and-required-fx/tdd-current.json) distinguishes two public backend RED/GREEN cycles, one actual UI disclosure cycle and one retained-contract-only unqualified-rate cycle. First-pass public regressions and inherited T02 summary records are separately labeled; no fabricated RED or public positive. The first desktop build was interrupted for Spec correction and is not PASS; final build/pin and UI logs follow separately.

Parent121,map1,S28,S17,physicalS27,S33 and dev→main remain open. AC-035 NOT_STARTED; only target ownership and PRD line pointers changed. No real provider write/Arm/consent/dispatch or native/physical acceptance. Prototype code unchanged; main unchanged.

Final ordinary desktop build exited0; its embedded Gateway pin equals the restored ordinary Gateway (`5f2799d6…`). [Build/input record](evidence/s29-fee-and-required-fx/desktop-build-review-final-inputs.json) binds all169 inputs. [Final actual React/Rust UI](evidence/s29-fee-and-required-fx/ui-review-final-report.json) passes missing/declared/changed/captured/pre-arm disclosure, immutable history and1280/768/390 widths, with no console errors. No Arm or consent was performed. The owned1427 host and tab were closed and the viewport reset. The desktop build is build evidence, not native/real-provider acceptance.

[Remote handoff](evidence/s29-fee-and-required-fx/delivery-before-close.json) verifies169 committed source/build inputs, committed evidence hashes, native121→130→131 hierarchy, completed dependency129 and0 open blockers. [Tracker readback](evidence/s29-fee-and-required-fx/tracker-closure-readback.json) verifies131 CLOSED/11 checked AC,130 CLOSED/10 checked AC, parent121 and map1 OPEN. No CI workflow was present; local checks are not represented as CI.
