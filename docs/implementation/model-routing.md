# S03 / #13 模型默认路由、回退与配额恢复 Spec

状态：**IMPLEMENTED LOCALLY（保持 OPEN，#12 的 OAuth 401 过期 UI 门槛仍待验收）**。本票建立可供后续 Turn 使用的路由与 attempt 边界；#12 的真实 OAuth/上游 route 证据已记录在 `s03-model-evidence.md`，本票仍不实现 S04 的 Codex 流式生命周期。

## Problem Statement

模型连接成功后，用户仍需要一个可持久化的默认 provider/model/mode，明确知道失败是否可以重试，以及是否愿意让模型输入交给另一个外部 provider。当前状态只记录已验证 route 和 setup attempt，无法保存默认偏好、fallback consent、Thread attempt 类型或 cooldown 门控。

## Solution

在现有 `model` aggregate 上增加持久化 routing preference、fallback consent/version、latest bounded quota metadata 和明确的 attempt kind。提供 `model.set_default` 与 `model.set_fallback_policy` 公共命令；两者都使用 workspace 与 state version 条件写入，并发布完整 sanitized model projection。路由规划器为后续 Codex 请求生成 immutable provider/model/mode snapshot：默认只返回当前默认 route，只有用户显式开启 fallback、DeepSeek 目标 route 已验证且原始错误属于 eligible `MODEL_UNAVAILABLE`/`OAUTH_EXPIRED`/`QUOTA_EXCEEDED` 时才附加一个 DeepSeek fallback。永不自动切回 ChatGPT，永不修改账户 capability、risk、arming 或 approval。

失败 attempt 保留在 append-only history 中。已知 cooldown 未到期时，Retry/Verify 被拒绝并保留原状态；未知 quota 只显示 unavailable，不推断为零或无限。显式 Switch/Retry 都创建新的 attempt 或 routing event，UI 始终披露 provider change、fallback consent 和可用 quota。

## User Stories

1. As a user, I want to choose a verified route as my default, so that the next model request has a clear provider and model.
2. As a user, I want my default route to survive restart while health becomes unverified, so that preference is not mistaken for live availability.
3. As a user, I want an explicit privacy explanation before enabling automatic DeepSeek fallback, so that I control the external processor receiving model input.
4. As a user, I want fallback consent and its version persisted, so that a restart cannot silently change my choice.
5. As a user, I want automatic fallback OFF by default, so that a provider failure never changes destination implicitly.
6. As a user, I want fallback to require a currently verified DeepSeek route, so that an unavailable alternate cannot be presented as recovery.
7. As a user, I want an in-flight request to retain its original provider/model/mode snapshot, so that changing Settings affects only the next attempt.
8. As a user, I want setup attempts and future Thread attempts distinguished, so that onboarding probes cannot be mistaken for productive turns.
9. As a user, I want provider, model, thinking mode, timestamps, result and canonical error recorded for every attempt, so that routing is auditable.
10. As a user, I want a failed primary attempt preserved when fallback runs, so that provider changes are visible rather than erased.
11. As a user, I want no automatic fallback back to ChatGPT, so that recovery follows the explicit one-way policy.
12. As a user, I want a known retry cooldown enforced by the control plane, so that Retry cannot hammer a throttled provider.
13. As a user, I want unknown quota fields shown as unavailable, so that the UI never invents a limit or reset time.
14. As a user, I want explicit Retry to create a new attempt, so that closing an error cannot restore health or erase evidence.
15. As a user, I want explicit Switch to DeepSeek to affect the next attempt only, so that current financial and execution context remains unchanged.
16. As a user, I want `OAUTH_EXPIRED`, `QUOTA_EXCEEDED` and `MODEL_UNAVAILABLE` to offer distinct remediation, so that I know whether to re-login, wait, retry or switch.
17. As a user, I want model settings to leave account capability, risk policy, arming and approvals unchanged, so that routing never grants financial authority.
18. As a user, I want defaults, consent, attempts and fallback events to survive SQLite reopen and replay, so that the audit trail is durable.
19. As a narrow-window or keyboard user, I want default, fallback and recovery controls to remain reachable at 768 px and 390 px, so that no safety choice is hidden.
20. As a user, I want the composer and Settings to show the provider path for the next request, so that external processing is disclosed before Send.

## Implementation Decisions

- Extend the schema-driven model aggregate with a non-secret `ModelSelection` (provider, real model ID, explicit thinking mode), optional `defaultRoute`, `automaticFallback` consent and monotonic `fallbackPolicyVersion`. Preserve the existing `currentRoute` as the latest verified route used for health evidence.
- Add `ModelAttemptKind` with `SETUP` and `THREAD`; existing setup actions write `SETUP`. A reusable route planner exposes a `THREAD` snapshot without opening a new IPC path or sending workspace data. S04 will consume this planner when the Codex stream exists.
- Add `model.set_default` accepting only a route already present with a non-null verification timestamp and a provider in `READY`; reject aliases, unverified routes, stale workspace versions and unknown thinking modes. Setting a default never changes financial state.
- Add `model.set_fallback_policy` with an explicit boolean and state-version precondition. Enabling requires a verified `deepseek-v4-flash` route in either explicit mode and increments the consent version; disabling is always allowed and also increments it.
- Define fallback eligibility in one trusted planner: the primary snapshot is immutable; fallback is offered only for the three canonical model failures, only when consent is enabled, only when the target DeepSeek route is verified, and only once. No fallback is attempted for cancellation, payload errors or in-flight state conflicts.
- Persist bounded quota fields with the attempt. A relative `retry-after` is retained as bounded seconds and paired with the attempt end timestamp; the control plane rejects a new Verify/Retry until that cooldown expires. Missing or malformed quota remains `None`.
- Add canonical `MODEL_QUOTA_COOLDOWN` remediation and expose Retry, Re-login and Switch actions according to error category. Buttons use public commands and preserve the failed attempt.
- Keep all model traffic on the existing loopback gateway. The route planner carries metadata only; it never accepts a URL, key, token, prompt or broker reference.

## Testing Decisions

- Exercise public command/result/event/snapshot/replay behavior through the existing temporary SQLite Control Plane seam. Assert state versions, append-only attempt order, stale-operation rejection and unchanged gateway/account/risk projections.
- Test route selection with verified ChatGPT and DeepSeek fixtures: default OFF/ON, consent version increments, unverified and missing fallback target rejection, eligible versus ineligible errors, one-way fallback and immutable primary snapshot.
- Test quota boundaries with known/unknown metadata, cooldown before/after expiry, malformed timestamps and bounded retry-after values. Tests must assert no retry command is issued while blocked.
- Test frontend behavior against the Rust-backed browser integration: default selection, explicit privacy consent, remediation actions, attempt disclosure, quota unavailable/known states and 768/390 layout. Renderer fixtures must use visibly synthetic metadata and no secrets.
- Reuse existing model aggregate replay, state-version and native boundary tests; do not claim #12 credentialed acceptance or S04 streaming acceptance from this ticket.

## Out of Scope

The native secure-dialog Save path and OAuth timeout UI are evidenced in isolated workspaces; OAuth 401 expiry UI remains outside this ticket. OAuth cancellation evidence is recorded in `s03-model-evidence.md`. Codex App Server streaming, full Turn persistence, automatic model invocation transport, and complete five-step onboarding also remain outside this ticket. Credentialed ChatGPT/DeepSeek route evidence is recorded in `s03-model-evidence.md`; #12 remains open until its remaining independent OAuth 401 expiry UI evidence is supplied.

## Further Notes

Authority: PRD §16.3, §26.3, FR-070/FR-073, AC-057/AC-058 and UX-009; UI Spec §14.8 and J1; Backend ARD §10.3–10.5, §41–42. This spec records the user's instruction to defer #12 while continuing the map with #13.
