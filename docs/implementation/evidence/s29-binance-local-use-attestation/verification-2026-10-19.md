# S29.11 Binance local-use attestation — implementation evidence

Date: 2026-10-19

Scope: sequential capability issue [#150](https://github.com/kaiqiangh/tradex/issues/150), implementing the user-approved local-use proposal for the owning S29.11 plan.

## Implemented contract

- A workspace may store an optional Binance local-use attestation alongside the saved ordinary Spot source. Older saved records remain readable. Attest and withdraw commands use the current source version, advance its generation, and do not perform provider I/O.
- The binding covers workspace, source generation, supported dataset and fee-symbol scope, declared jurisdiction, local decision-support purpose, local-only destination, retention boundary, policy version, and the displayed Binance terms URL/review date. Configure, disconnect, and withdrawal invalidate the prior confirmation.
- The recognized jurisdiction format is an ISO 3166-1 alpha-2 code. Unknown codes are refused. `US` is refused by the current local product policy; that is an application policy result, not a legal or provider determination.
- A current matching confirmation can pass only the Binance `MARKET_DATA_USE` risk check with `USER_ATTESTED_LOCAL_ONLY` provenance. Quote `dataUseRights` and provider entitlement remain `UNVERIFIED`; unresolved, stale, mismatched, unsupported, or locally restricted records refuse. All other risk checks remain independent. Captures retain bounded confirmation provenance/digest, not raw market-book data.
- Settings provides an unchecked-by-default explicit confirmation, jurisdiction input, current terms link/date, backend assessment/reason, a reusable status summary, and withdrawal. The summary is also visible in approval/risk review.

## Verification

The following focused checks passed during implementation:

- `cargo fmt --all`
- `cargo test -p tradex --lib binance_market::local_use_tests -- --nocapture` — 2 passed.
- `cargo test --workspace -p tradex --test binance_market --features integration-test -- --test-threads=1` — 4 passed.
- Focused public Hot-source local-use regression — 1 passed.
- `npm run test:binance-hot` — 60/60 scenarios passed in serial isolated processes.
- `npm run schema:check`, `npm run typecheck`, and `npm run build` passed. Vite reported the existing large-bundle warning.
- `python3 scripts/check_requirements.py` passed after correcting 203 stale PRD line references: 203 requirements, 70 screens, 13 QA scenarios, and 23 baseline files.
- Responsive browser verification covered 1280, 768, and 390 pixel widths, keyboard interaction, reload persistence, withdrawal, disconnect invalidation, and overflow/console checks. No horizontal overflow or application console errors/warnings were observed.

Final aggregate verification passed after correcting the stale requirement references: `RUST_TEST_THREADS=1 RUST_MIN_STACK=16777216 npm run check` exited 0. It covered generated schema agreement, TypeScript and production build, 35/35 Node tests, `cargo test --workspace` (579 passed, 33 existing ignored, 0 failed), secret scan, and traceability (203 requirements, 70 screens, 13 QA scenarios, 23 baseline files). `cargo fmt --all -- --check` and `git diff --check` also passed. The first aggregate run had stopped at traceability because 203 rows still referenced old PRD line numbers; those references were corrected before this final green run. Existing Rust dead-code warnings and the 5,956.37 kB Vite bundle warning remain.

Screenshots: [desktop](settings-desktop.png) · [mobile](settings-mobile.png).

## Boundaries and status

No real provider/account read, provider mutation, approval, Arm, consent, Prepare, dispatch, or financial action was performed. The external Binance terms page could not be fetched through the configured network proxy; the implementation records a dated terms pointer for the user's declaration and does not claim independent legal review or provider permission. Synthetic test fixtures establish implementation behavior only.

This completes the local implementation slice for #150; it does not satisfy the full owning ticket. #135 remains open, all its acceptance states remain unchanged, and its complete producer-to-Prepare, source-use, financial, reservation, settlement, and final review gates remain outstanding. Work stays on `dev`; no main PR was created.

---

# S29.11 Binance 本地用途确认 — 实现证据

日期：2026-10-19

范围：顺序能力票 [#150](https://github.com/kaiqiangh/tradex/issues/150)，实现用户已批准的 S29.11 owning plan 本地用途提案。

## 已实现契约

- 工作区可在已保存的普通 Spot 来源配置旁保存可选 Binance 本地用途确认；旧记录仍可读取。确认与撤回命令使用当前来源版本并推进 generation，不执行 provider I/O。
- 绑定内容覆盖工作区、来源 generation、受支持的数据集及费用币种范围、声明的辖区、本地交易决策支持目的、仅本地目的地、留存边界、策略版本，以及界面展示的 Binance 条款 URL/复核日期。重新配置、断开连接或撤回会使旧确认失效。
- 辖区格式按 ISO 3166-1 alpha-2 代码识别；未知代码会被拒绝。当前本地产品策略拒绝 `US`；这是应用策略结果，不构成法律或 provider 判定。
- 当前且匹配的确认仅可使 Binance `MARKET_DATA_USE` 风险检查以 `USER_ATTESTED_LOCAL_ONLY` provenance 通过。行情 `dataUseRights` 和 provider entitlement 仍为 `UNVERIFIED`；未解决、过期、错绑、不支持或本地限制状态均拒绝。所有其他风险检查保持独立。捕获记录仅保留有界确认 provenance/digest，不保存原始行情簿数据。
- Settings 提供默认未勾选的显式确认、辖区输入、当前条款链接/日期、后端评估状态/原因、可复用状态摘要及撤回操作；approval/risk review 中也展示相同摘要。

## 验证

实现过程中的以下定向检查通过：

- `cargo fmt --all`
- `cargo test -p tradex --lib binance_market::local_use_tests -- --nocapture`：2 项通过。
- `cargo test --workspace -p tradex --test binance_market --features integration-test -- --test-threads=1`：4 项通过。
- 定向 public Hot-source 本地用途回归：1 项通过。
- `npm run test:binance-hot`：串行隔离运行 60/60 场景通过。
- `npm run schema:check`、`npm run typecheck` 和 `npm run build` 通过；Vite 保留既有大型 bundle 告警。
- 修正 203 个过期 PRD 行号后，`python3 scripts/check_requirements.py` 通过：203 项需求、70 个屏幕、13 个 QA 场景、23 个基线文件。
- 响应式浏览器检查覆盖 1280、768、390 像素宽度、键盘交互、重载持久化、撤回、断开连接失效及溢出/console 检查。未观察到横向溢出或应用 console 错误/警告。

修正过期需求行号后，最终完整工作区复验通过：`RUST_TEST_THREADS=1 RUST_MIN_STACK=16777216 npm run check` 以 0 退出，覆盖生成 schema 一致性、TypeScript/生产构建、35/35 Node 单测、`cargo test --workspace`（579 项通过、33 项既有 ignored、0 失败）、secret scan，以及需求追踪（203 项需求、70 个屏幕、13 个 QA 场景、23 个基线文件）。`cargo fmt --all -- --check`及`git diff --check`也通过。首次聚合运行曾因203项需求引用旧PRD行号而停在traceability；修正后本次全量运行通过。保留既有Rust dead-code告警及5,956.37 kB Vite bundle告警。

截图：[桌面端](settings-desktop.png) · [移动端](settings-mobile.png)。

## 边界与状态

本次没有读取真实 provider/账户、修改 provider、执行 approval、Arm、consent、Prepare、dispatch 或金融操作。配置的网络代理无法获取 Binance 条款页面；实现为用户声明保存带日期的条款引用，不声称已独立完成法律审查或证明 provider 许可。合成测试夹具仅验证实现行为。

这完成 #150 的本地实现切片，但不满足完整 owning ticket。#135 仍为开放状态，所有验收项状态保持不变；完整 producer-to-Prepare、来源使用、金融、预留、结算和最终审查门禁仍待完成。工作留在 `dev`，没有创建 main PR。
