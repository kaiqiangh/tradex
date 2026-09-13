## Parent

Part of [Wayfinder] 完整构建 TradeX v1.0 RevC (#1).

## Question

如何让 Agent Mode、Execution Context、账户和附加上下文成为彼此独立、可审计且由可信 Control Plane 强制执行的工具权限边界，同时让 Composer 的 Context / Account / Mode / Model 选择真正作用于下一轮而不改写历史？

## Problem Statement

当前 Thread/Turn 已能保存 Agent Mode、Execution Context、账户、模型和通用 `ThreadContextRef`，但 composer 仍只提供一个账户下拉框，后端只做最小的模式/上下文校验，能力等级是字符串推导，研究和执行工具没有统一的可信决策入口。用户无法查看精确的工具能力、附加上下文的 canonical identity/hash 或非法组合的原因；Ask/Research、Backtest、Paper/Demo/Testnet、Live 的边界因此不能被完整证明。

## Solution

建立一个由 Control Plane 拥有的 capability policy seam。它以本轮即将冻结的 Agent Mode、Execution Context、账户健康/环境和 canonical context refs 为输入，返回 capability level、允许的 typed tool IDs、是否可执行和明确阻断原因。后端在 `turn.start` 前再次计算并拒绝不兼容或未授权的请求；`C5` 不作为 standing grant，`C6` 永远不支持。

Composer 增加独立的 Agent Mode、Execution Context、Account、Model/provider 和 `@ Context` 选择。临时选择只影响下一轮；Attach/Cancel 的 context 操作不修改 Thread 历史。可用账户以 canonical `account` refs 进入上下文目录；尚未由 S07/S14/S15/S12 提供的 instrument/strategy/backtest/artifact 目录显示明确的空/未连接状态，而不伪造实体。发送时把精确 ref IDs/hashes 传给后端，Turn snapshot 和 provenance 继续从后端读取。

## User Stories

1. As a TradeX user, I want Agent Mode and Execution Context shown as separate controls, so that intent and destination cannot be confused.
2. As a TradeX user, I want Ask to expose only lightweight read-only tools, so that asking a question cannot create a paper or live mutation.
3. As a TradeX user, I want Research to remain read-only even when a Live account is attached, so that account context never grants execution authority.
4. As a TradeX user, I want Backtest to use historical simulation only, so that a strategy run cannot reach a current-market broker tool.
5. As a TradeX user, I want Trade with Paper/Demo/Testnet to expose only the non-live execution capability available for that environment, so that the environment is explicit.
6. As a TradeX user, I want Trade with Live to stop at proposal capability until later arming and financial approval, so that selecting Live never arms an account.
7. As a TradeX user, I want illegal mode/context/account combinations disabled or explained, so that the UI never silently remaps my request.
8. As a TradeX user, I want to see the capability level and allowed tools for the pending selection, so that I can understand what the agent may do.
9. As a TradeX user, I want to attach instruments, accounts, strategies, backtest runs, and artifacts through `@ Context`, so that a turn has explicit research inputs.
10. As a TradeX user, I want a context picker to use stable canonical IDs and hashes, so that the same label cannot refer to a different object later.
11. As a TradeX user, I want Cancel in the context picker to discard temporary choices, so that browsing does not change the Thread.
12. As a TradeX user, I want Attach to show exact selected chips, so that I can review the next turn's inputs before sending.
13. As a TradeX user, I want removing a chip to affect only the next turn, so that prior Turn provenance remains immutable.
14. As a TradeX user, I want existing accounts listed with provider and environment, so that Paper, Demo, Testnet, and Live are not confused.
15. As a TradeX user, I want an attached Live account in Ask/Research labelled read-only, so that its presence cannot be mistaken for arming.
16. As a TradeX user, I want model/provider disclosure beside the capability summary, so that model selection never changes financial permission.
17. As a TradeX user, I want a failed capability lookup to block Send with a retry/reload action, so that missing authority is not treated as permission.
18. As a maintainer, I want one typed capability policy decision used by UI preflight and backend `turn.start`, so that the browser cannot widen authority.
19. As a maintainer, I want typed research tool IDs separated from financial command IDs, so that prompt-injected research text cannot call risk, approval, arming, or Order Gateway operations.
20. As a maintainer, I want deterministic policy tests for every matrix row and negative capability request, so that future modes/providers cannot silently widen tools.
21. As an operator, I want the immutable Turn snapshot to retain the capability decision inputs and context hashes, so that later defaults cannot rewrite historical authority.

## Implementation Decisions

- Keep Agent Mode and Execution Context as separate generated enums. Use the RevC matrix as the only compatibility source: Ask/Research are read-only; Backtest is historical simulation; Trade requires an execution account; Live reaches at most proposal capability in S05.
- Add generated, versioned capability types: `CapabilityLevel` (`C0`–`C6`), `ToolId`, and `CapabilityDecision { level, allowedTools, executionAllowed, reason }`. `C5` is represented only as a later control-plane approval result and `C6` returns unsupported.
- Add one canonical capability query seam for pending composer state. The query accepts workspace, mode, execution context, optional account ID, and canonical context refs; the backend reuses the same policy function in `turn.start`. A missing/unknown account or unavailable authoritative account state fails closed.
- Extend the v1 schema and bilingual contract documents before code generation. Preserve Backend ARD §41–42 envelope and exact command naming; no renderer-to-Codex or renderer-to-broker tool path is added.
- Add a typed context catalog response containing canonical ref, display label, kind, environment/read-only disclosure, and availability reason. Existing persisted account connections are the only populated catalog in S05; future instrument, strategy, backtest, and artifact providers add entries through the same response without changing the ref shape.
- An available attached account ref contributes only read-only `account_read` to Ask, Research, or Backtest; Trade still requires a separately selected execution account.
- Canonical account context hashes are derived only from stable non-secret account metadata and are never derived from credential material, raw provider bodies, or renderer labels. Ref validation accepts only the documented kinds and bounded IDs/hashes; duplicate refs are rejected.
- Composer picker state is ephemeral. `Attach` replaces the pending list, `Cancel` leaves it unchanged, and mode/account/model/context changes affect the next `turn.start` only. Existing Thread/Turn snapshots and provider attempts remain backend-owned.
- The UI renders capability level, allowed typed tools, environment, and explicit read-only/blocked reasons. A Live selection remains `LIVE · READ-ONLY` outside Trade and Trade Live remains `PROPOSAL_ONLY` until S22 adds arming/approval.
- Typed research tools are represented as data-plane IDs such as public market read, account read, and historical simulation. Financial command names, keychain access, risk policy mutation, approval, arming, and Order Gateway dispatch are never included in S05 allowed tools. Research content is untrusted and cannot promote a decision.
- Reuse current SQLite projection/event and TanStack Query seams. No second database writer, broker call, market-data provider, strategy worker, or artifact store is introduced. Empty future catalogs are explicit rather than fixture entities.
- Preserve S04’s immutable snapshot and add capability evidence to the same start-time projection. Historical rendering reads the stored snapshot, not current composer selectors.

## Testing Decisions

- Unit-test the pure capability matrix for every Agent Mode × account/environment row, including no account, missing account, Live read-only, Trade Live proposal-only, C5/C6 rejection, and unknown tool requests.
- Test canonical context validation and account hash stability with changed non-secret metadata, duplicate refs, unsupported kinds, control characters, and secret-like input. Assert no credential value enters the serialized ref or decision.
- Test the capability query and `turn.start` through the public versioned IPC seam. Assert the same decision is returned before send and enforced again at send; illegal payloads cause no upstream runtime or financial mutation.
- Test context picker external behavior through the existing browser integration path: open/cancel/attach/remove, account environment labels, explicit empty states for future catalogs, mode/context explanation, and no horizontal overflow at 390/768.
- Test immutable provenance by changing current picker selections after a completed Turn and reloading the Thread; the stored mode, context, account and ref hashes must remain unchanged.
- Test typed tool boundary with an allowed research request, a current-market execution request in Ask/Research/Backtest, and a financial command/prompt-injection negative case. The result must show a sanitized `UNSUPPORTED_CAPABILITY` or equivalent error and no state mutation.
- Run schema generation/check, Rust fmt/Clippy/tests with integration feature, frontend typecheck/unit tests, `npm run check`, `git diff --check`, and isolated Rust-backed browser checks. Real market/provider/tool data remains outside S05 unless independently available; fixtures cannot certify those later slices.

## Out of Scope

- Market/instrument data, watchlists, research providers, portfolio reads, news/filings/fundamentals, strategy storage, backtest execution, artifacts, order drafts, risk policy, arming, approvals, reservations, Order Gateway, broker mutations, and live reconciliation.
- Populating instrument/strategy/backtest/artifact catalogs before their owning S07/S12/S14/S15 slices.
- Granting C5 or implementing unattended C6; selecting Trade or Live never arms or approves anything.
- New model providers, automatic fallback, direct external LLM calls, raw Codex tool protocol exposure, or credentials in context records.
- Full QA-01–12 cross-page and signed packaging; S33/S34 own those gates.

## Further Notes

- Source authority: PRD §§11–14, 16, 17, 36, 41, 52, 61, 69; UI Spec §§3–5, B1–B9, §14.1 and §14.7; Frontend ARD §§7–12, 19, 21; Backend ARD §§8–10, 40–43.
- S04 already persists the immutable mode/context/account/model/provider/context snapshot. S05 closes the capability/tool decision and composer context seam without rewriting that history model.
- S05 implementation starts from current `dev` after S04 evidence commit `113edbe`; each child slice must be completed and reviewed before the next child begins.
