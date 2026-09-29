# S04 — 持久化并流式恢复 Codex Thread

状态：**Spec 已完成，待按依赖串行实现**。父项：Wayfinder map #1。开发分支：`dev`。

## Problem Statement

TradeX 已经能保存工作区、账户、模型网关和模型连接，但还没有真实的 Thread/Turn/Item 运行时。用户无法创建独立对话、看到尚未完成的流式结果、在退出后恢复历史，或判断一轮请求使用了哪个模式、执行上下文、账户、模型、提供方和尝试记录。当前前端因此必须把发送入口标为不可用；若直接把当前选择器套到历史消息上，还会违反 RevC 要求的历史不可变来源和金融安全边界。

Codex App Server 是 S04 的上游运行时。TradeX 的 Renderer 不能直接接触其 JSON-RPC/JSONL、审批或进程；可信 Rust Control Plane 必须把它映射成 Backend ARD §41–42 的版本化 IPC 和持久化事件。上游协议持续演进，TradeX 的稳定 wire contract 仍由自己的 `thread.*`/`turn.*` 命令和 Thread/Turn/Item 事件拥有。

## Solution

用户可以创建一个独立 Thread，在 composer 中选择 Agent Mode、Execution Context、账户、模型和附加上下文后发送请求。发送时后端先验证模式与上下文兼容性，冻结不可变 Turn start snapshot，保存 user message，再启动受控 Codex App Server 会话。来自上游的 `thread/started`、`turn/started`、`item/*`、delta、`turn/completed` 或失败/中断通知被转换为严格递增的 Thread aggregate 事件；用户在一轮完成前就能看到状态和内容。

Thread history 在本地持久化。选择历史 Thread 会恢复它自己的 timeline 和默认上下文；恢复不会复用过期/已消费审批、旧的有效性判断或 ARMED 状态。当前 picker 的临时变化只影响下一轮，不能改写已完成 Turn。取消映射到上游 `turn/interrupt`，重试生成新 Turn 和新 provider attempt，并保留前一次失败事实。模型不可用时，控制面和历史仍可读，UI 显示可重试的明确原因。

## User Stories

1. As a TradeX user, I want to create a new Thread, so that each research conversation has an independent history.
2. As a TradeX user, I want the new Thread to appear immediately in local history, so that I can navigate back without waiting for a model response.
3. As a TradeX user, I want to send a natural-language request in Ask mode, so that I can obtain read-only assistance without granting trading authority.
4. As a TradeX user, I want to choose Research, Backtest, or Trade mode explicitly, so that capability follows my stated intent.
5. As a TradeX user, I want to choose an execution context separately from mode, so that a model change cannot silently change account permissions.
6. As a TradeX user, I want the selected account and its canonical environment shown before sending, so that Paper, Demo, Testnet, and Live are not confused.
7. As a TradeX user, I want the selected model and provider origin disclosed before sending, so that I know which route will answer.
8. As a TradeX user, I want attached context IDs and hashes visible in the turn provenance, so that research inputs can be audited later.
9. As a TradeX user, I want invalid mode/context/account combinations rejected before any upstream call, so that an unsafe request cannot reach a model.
10. As a TradeX user, I want a sent request to capture a complete immutable snapshot, so that later picker changes do not rewrite history.
11. As a TradeX user, I want my user message persisted before the model starts, so that a crash cannot erase my intent.
12. As a TradeX user, I want to see an explicit running Turn state, so that I know the request has been accepted.
13. As a TradeX user, I want agent text to stream into the timeline, so that I can read useful output before completion.
14. As a TradeX user, I want plans, tool calls, tool results, warnings, errors, and artifacts rendered as typed timeline items, so that I can distinguish work from commentary.
15. As a TradeX user, I want each item to show started, streaming, completed, or failed state, so that partial work is not mistaken for final work.
16. As a TradeX user, I want provider attempts recorded separately from the immutable start snapshot, so that retries and fallback decisions are auditable.
17. As a TradeX user, I want a model outage to leave the control plane and existing history usable, so that I can inspect or retry without losing state.
18. As a TradeX user, I want to cancel a running Turn, so that an unwanted read-only or research request stops promptly.
19. As a TradeX user, I want cancellation to show Cancelled or Interrupted explicitly, so that I do not mistake an interrupted response for a completed answer.
20. As a TradeX user, I want to retry a failed or interrupted Turn as a new Turn, so that the original failure remains evidence and the new attempt has a new identity.
21. As a TradeX user, I want duplicate or out-of-order stream events handled visibly, so that a protocol problem cannot silently corrupt my timeline.
22. As a TradeX user, I want an event gap to trigger a snapshot reload, so that the UI converges on the authoritative projection.
23. As a TradeX user, I want to reopen a previous Thread after restarting the app, so that local history survives process and window lifetimes.
24. As a TradeX user, I want resuming a Thread to restore its own defaults and context references, so that I can continue the same work.
25. As a TradeX user, I want resuming never to restore consumed or expired approvals, stale market validity, or ARMED state, so that history cannot grant execution authority.
26. As a TradeX user, I want a new Thread to leave all existing Threads intact, so that starting fresh is non-destructive.
27. As a TradeX user, I want the thread list to expose loading, empty, failed, and ready states, so that missing data is not presented as an empty account.
28. As a TradeX user, I want a selected Thread's identity to remain stable across navigation and reload, so that events cannot appear in the wrong conversation.
29. As a TradeX user, I want the composer to retain an unsent draft while browsing a Thread, so that navigation does not discard my text accidentally.
30. As a TradeX user, I want Enter to send only after normal validation and never to approve a financial action, so that keyboard use cannot bypass safety.
31. As a TradeX user, I want Escape to close temporary picker UI safely, so that dismissing a control cannot arm or submit anything.
32. As a keyboard or screen-reader user, I want focus, semantic state labels, and announced stream updates, so that the timeline remains understandable without a pointer.
33. As a narrow-window user, I want history, account context, and safety status retained in the drawer/composer, so that responsive layout does not hide essential controls.
34. As a TradeX user, I want generic Codex approval events kept distinct from FinancialApproval, so that a model confirmation cannot authorize a financial mutation.
35. As an operator, I want every upstream attempt and completion/error category persisted, so that failures can be diagnosed without storing secrets or raw provider bodies.
36. As an operator, I want the runtime to be cancellable and bounded, so that a crashed or overloaded sidecar cannot hold the control plane indefinitely.
37. As an operator, I want an unsupported upstream protocol or schema version surfaced as a compatibility error, so that upgrades fail closed and are reviewable.
38. As a maintainer, I want a deterministic fake App Server seam, so that stream ordering, gaps, cancellation, retry, and model outage can be tested without external network or credentials.
39. As a maintainer, I want the installed/pinned Codex version and generated schema checked at startup or compatibility test time, so that protocol drift is detected before acceptance.
40. As a TradeX user, I want read-only tools to remain read-only when a Turn fails, so that an error path cannot mutate financial state.

## Implementation Decisions

- **Stable boundary.** Keep the canonical Backend ARD §41 envelope (`schemaVersion`, `requestId`, command, payload, expected state version) and §42 event envelope (`eventId`, `eventType`, `schemaVersion`, timestamp, aggregate type/id, strict sequence, payload). Add `thread` as an explicitly schema'd aggregate; do not expose upstream JSON-RPC methods to React.
- **Commands.** Implement `thread.list`, `thread.get`, `thread.create`, `turn.start`, `turn.cancel`, and `turn.retry`. TradeX `turn.cancel` maps to current Codex `turn/interrupt`; the mapping is internal and can change with a pinned compatibility adapter.
- **Upstream session.** Rust owns a bounded stdio JSONL child process for the pinned `codex app-server` binary. A connection performs one `initialize` request followed by `initialized`; Thread creation/resume uses `thread/start`/`thread/resume`; a Turn uses `turn/start`. Shutdown, malformed frames, EOF, timeout, and unsupported method/schema become explicit runtime errors. No TCP listener or direct Renderer process is allowed.
- **Version discipline.** Record the installed Codex CLI version and the generated protocol/schema fingerprint used by the adapter. The candidate local version is accepted only after an isolated initialize/thread smoke and compatibility tests. Do not claim upstream `main` is a stable contract; fail closed on drift.
- **Thread projection.** Extend the existing SQLite JSON projection/outbox pattern with a `threads` projection keyed by `thread_id` and workspace. The projection stores Thread summary/default navigation context and bounded ordered Turn/Item history; the outbox is the authoritative event handoff. Use one transaction for projection plus outbox append and strict per-aggregate sequence. Reuse existing snapshot/replay and single-sink subscription machinery rather than adding another database writer.
- **Thread identity and history.** Thread fields include workspace ID, Codex thread ID when available, title, created/updated times, default Agent Mode, linked account/instrument/strategy/artifact references, and lifecycle status. `thread.list` returns summaries; `thread.get` returns the authoritative detail. Reload after a subscription gap or state-version conflict.
- **Immutable Turn snapshot.** At `turn.start`, backend validates the requested mode, Execution Context, account and attached canonical context refs; resolves the current verified model route using the existing model projection; then freezes turn ID, mode, context, account, capability level, model/provider, attached IDs/hashes, and start time. Picker changes affect only a later Turn. Provider attempts and completion metadata are appended outside the snapshot.
- **State vocabulary.** Turn states are `RUNNING`, `COMPLETED`, `CANCELLED`, `INTERRUPTED`, and `FAILED`. Item lifecycle is `STARTED`, `STREAMING`, `COMPLETED`, `FAILED`; item kinds cover at least user message, agent message, plan, tool call/result, warning, error, and artifact. Unknown upstream item kinds remain visible as compatibility items instead of being dropped.
- **Streaming.** Convert upstream notifications to typed Turn/Item events and persist every accepted sequence before publishing it. Deltas update the current streaming item; `turn/completed` closes the Turn only after all prior item events are committed. Duplicate events are idempotent; gaps/conflicts stop incremental application and request a snapshot.
- **Cancel/retry.** Cancel is available only for a running Turn and records the upstream interrupt outcome. Retry is a new Turn with a new immutable snapshot and provider attempt; it never mutates the failed Turn or revives a consumed approval. A cancelled/failed process is cleaned up before retry.
- **Capability and safety.** Ask and Research remain read-only; Backtest cannot invoke a current-market broker; Trade still requires later arming/FinancialApproval gates. Generic Codex approvals are stored and rendered separately. S04 does not implement order proposals, reservations, arming, or broker dispatch.
- **Frontend state.** Use query state for thread summaries/detail and event state for streaming Turn/Item updates; keep only ephemeral picker/draft UI in the existing lightweight store. Render explicit loading/empty/failed states, focusable controls, responsive drawer, and provenance cards. Thread selection restores its own defaults; model/mode/account switches apply next Turn.
- **Failure isolation.** Model outage, quota, OAuth expiry, malformed frame, process exit, backpressure, and timeout must leave persisted Thread history and the trusted control plane readable. Never log provider secrets, OAuth files, broker secrets, or raw diagnostic bodies.
- **No speculative layers.** Reuse current IPC decoding, generated validators, outbox, subscription, model route/attempt projection, and integration bridge. Add only the Thread/Turn/Item schema, projection, runtime adapter, commands, and UI surfaces required by the acceptance criteria.

## Testing Decisions

- Prefer the highest existing seam: public Rust command/result/event/snapshot/replay plus the same React/browser path used in production. Avoid tests that inspect private reducer calls.
- Add deterministic fake App Server frames for initialize, thread start/resume, streamed item deltas, completion, malformed/unknown frames, duplicate/gap sequence, interrupt, process exit, and retry. The fake must speak the same JSONL adapter boundary as the production child process.
- Add a compatibility smoke against the installed pinned `codex app-server` in an isolated HOME/config directory. Verify handshake and thread lifecycle only unless authenticated inference is independently available; no real user credentials or broker secrets enter the test.
- Exercise persistence across close/reopen and subscription replay. Assert that the timeline is present before completion, that strict sequences converge after a gap, and that unknown protocol versions produce an explicit compatibility error.
- Assert immutable provenance after changing current selectors, including model/provider, mode, execution context, account, and context hashes. Assert retry creates a new Turn and preserves all prior attempts.
- Assert cancellation states and cleanup, model outage independence, bounded timeout/backpressure, and no financial mutation from Ask/Research failure paths.
- Run narrow/desktop browser checks at 390px and 768px, keyboard/focus/semantic state assertions, and screen-reader-visible status text where the existing harness supports it.
- Use repository checks as the final gate: generated schema/type check, focused Rust/frontend tests, integration bridge, `npm run check`, `cargo test`, `cargo fmt --all -- --check`, and `git diff --check`. Record exact SHA and any unavailable native/runtime evidence; fixtures never upgrade runtime proof.

## Out of Scope

- S05 composer capability matrix beyond the S04 mode/context validation required to start a Turn.
- Research MCP/data providers, watchlists, artifacts library, strategies, backtests, order drafts, risk policy, arming, FinancialApproval, reservations, Order Gateway, broker dispatch, reconciliation, Live execution, and complete diagnostics.
- Automatic provider fallback or route changes beyond recording attempts from the already verified S03 model route; no new model provider.
- Direct browser/network model calls, a second persistence store, TCP App Server exposure, or storing secrets in Thread/Turn/Item records.
- Native signed packaging and full QA-01–12 cross-page regression; S33 owns the final regression after all feature slices.

## Further Notes

- Source authority is PRD §§11–12, 16, 58 and FR/AC-001/002/003/004/045/058/059/060/072/074/075; UI Spec B1–B9 and §14; Frontend ARD navigation/state/IPC sections; Backend ARD §§8–9 and 41–43. Coverage/QA currently marks the relevant runtime requirements `RUNTIME_PENDING` or `FAILED`; implementation must earn a fresh runtime result.
- Current local Codex CLI reports `codex-cli 0.154.0`. This is a candidate pin, not acceptance evidence. The adapter must record the actual binary/schema used and preserve the upstream command mapping.
- S04 begins only after S03 is closed. The work is deliberately split into dependency-ordered vertical slices so each slice can be demonstrated through schema, Rust, UI, persistence, and tests before the next starts.
