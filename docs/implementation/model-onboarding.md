# S03 — 配置模型网关并完成五步入门

状态：Spec [配置模型网关并完成五步入门](https://github.com/kaiqiangh/tradex/issues/10) 已发布，未验收。实现顺序：[网关生命周期](https://github.com/kaiqiangh/tradex/issues/11) → [模型连接](https://github.com/kaiqiangh/tradex/issues/12) → [路由恢复](https://github.com/kaiqiangh/tradex/issues/13) → [五步入门](https://github.com/kaiqiangh/tradex/issues/14)。起点 `dev@c6c193fa50848168086377d782c82bfd4fb1ec4a`；S01/S02 已完成。沿用已确认的公共协议测试边界、纵向拆票粒度与严格串行交付约定。

## Problem Statement

用户已经能打开工作区和连接账户，但无法配置模型、验证真实可用路由或完成五步入门。进程运行、模型目录存在与实际可推理是不同事实；将任一目录响应当作 Ready，会让用户在首次使用时遇到未授权、配额耗尽或错误路由。模型故障也不能阻断独立的账户控制。

## Solution

在入门 Model 步骤和 Settings 共用模型配置界面，由 Rust 管理固定版本 CLIProxyAPI。用户可启动网关、完成 ChatGPT OAuth，或通过原生安全输入配置 DeepSeek，然后验证模型、选择默认路由，并显式决定是否允许自动回退。五步入门保留 Workspace → Providers → Model → Risk Defaults → Ready 的顺序、返回编辑和重开恢复；Ready 摘要仅使用当前验证证据，所有 Live 账户仍为 DISARMED。

## User Stories

1. As a user, I want the installed gateway version and endpoint displayed, so that I can identify the exact runtime serving my requests.
2. As a user, I want Launch to install or locate the verified pinned artifact and start the gateway, so that I do not need to run shell commands.
3. As a user, I want an occupied port reported without attaching to or stopping its owner, so that another local service remains intact.
4. As a user, I want Starting, Running, Stopped, Port conflict and Unauthorized distinguished, so that recovery reflects the actual failure.
5. As a user, I want bounded restart with visible backoff, so that a crash does not cause an invisible restart loop.
6. As a user, I want ChatGPT login and re-login launched through the sidecar's browser OAuth flow, so that TradeX never handles OAuth tokens.
7. As a user, I want cancellation and failed OAuth to preserve the previous configuration without inventing authorization, so that I can retry safely.
8. As a user, I want provider model discovery restricted to the RevC sources and model families, so that unrelated configured models cannot enter my route picker.
9. As a user, I want masked native DeepSeek key entry with Keychain storage and explicit cancel, so that secrets never enter the renderer.
10. As a user, I want DeepSeek probe plus a bounded test inference, so that a configured key or static model list alone cannot mark it connected.
11. As a user, I want the selected ChatGPT route verified before use, so that expired authorization or an unavailable model blocks Ready.
12. As a user, I want test inference to use a fixed harmless prompt without workspace/account content, so that setup does not export my data.
13. As a user, I want my default provider/model preserved across restarts while its health becomes unverified until rechecked, so that saved preference is not confused with live evidence.
14. As a user, I want automatic cross-provider fallback OFF initially and an explicit privacy explanation before opting in, so that I control which external processor receives model input.
15. As a user, I want an explicit switch or retry to create a new recorded attempt, so that the failed attempt remains visible.
16. As a user, I want every attempt's provider/model, times, outcome and sanitized quota metadata retained, so that routing is auditable.
17. As a user, I want provider-reported quota windows and cooldown shown only when known, so that unknown limits are not rendered as zero or unlimited.
18. As a user, I want quota exhaustion, expired OAuth and unavailable gateway to offer the appropriate retry, re-login or explicit alternate route, so that closing an error cannot restore health.
19. As a user, I want model changes to leave account selection, capability, risk policy, arming and approvals untouched, so that routing never grants financial authority.
20. As a user, I want account inspection and trusted control-plane commands to remain responsive during model startup, network failure and backoff, so that model availability is not a dependency of account safety.
21. As a user, I want forward/back navigation through the five setup steps without losing entered non-secret settings, so that I can review setup before finishing.
22. As a user, I want the Providers step to reuse the existing schema-driven connection/review flow, so that setup and Settings have identical account semantics.
23. As a user, I want editable risk defaults with explicit units, decimal validation and no preselected financial limits, so that the app does not silently choose my risk appetite.
24. As a user, I want hard safety rules read-only and market orders initially disabled, so that setup cannot bypass system guards.
25. As a user, I want the Ready summary to show workspace, currency, providers, verified model route, fallback choice and all Live accounts DISARMED, so that completion has a concrete meaning.
26. As a returning user, I want unfinished onboarding restored and invalidated model evidence to block completion, so that restart cannot skip setup gates.
27. As a keyboard or narrow-window user, I want every setup, login, retry, provider switch and cancel control reachable, so that the flow works at 768 px and 390 px with visible focus and status announcements.
28. As a user, I want model disconnect and application exit to clear rendered secret configuration and stop owned children, so that credentials and orphan processes do not remain unintentionally active.

## Implementation Decisions

- Reuse the Rust Control Plane, SQLite atomic event/outbox, public command/result/snapshot/replay, React domain projection and existing native secure-entry pattern. Add exact commands/events and schemas to bilingual Backend ARD §41–42 before implementation. No raw key, caller-selected URL, arbitrary command, process ID or credential reference is accepted through renderer commands.
- Only CLIProxyAPI receives external inference traffic. TradeX talks to fixed `127.0.0.1:8317` with an unpredictable downstream secret, no proxy or redirects. The supervisor verifies ownership rather than trusting a successful response from an occupied port. Gateway I/O and process waits occur outside the Control Plane mutex.
- Pin CLIProxyAPI `v7.2.155` as the initial compatibility candidate, published 2026-09-08. Verify official release archive SHA-256 and executable identity before launch; a version change requires explicit compatibility review. Do not adopt a user's global gateway configuration or auth directory. Compatibility failures remain failures rather than silently selecting another version.
- Render an allowlisted configuration: loopback host, no remote management/control-panel downloads, no plugins/debug/request logging, no inherited proxy or unrelated provider configuration. Disable gateway retry rounds and implicit route changes; TradeX owns observable retry/fallback decisions. Test the pinned upstream's actual behavior, including failure logging and shutdown, before acceptance.
- OAuth tokens remain solely in a dedicated sidecar auth directory; TradeX does not parse its token files. DeepSeek uses a separate model-only OS Keychain service. Render launch configuration outside workspace storage with mode 0600 in a private directory; remove it on exit and recover stale rendered files after crashes. Never inject broker secrets or broker Keychain references into the sidecar.
- Route identifiers preserve actual discovered IDs and provider identity. Permit GPT-5.6 series for ChatGPT and `deepseek-chat` / `deepseek-reasoner` for DeepSeek as the normative baseline; do not relabel another upstream model to fake compatibility. If the official service no longer provides a required model, report the verified mismatch and preserve the requirement for resolution.
- A usable route requires successful authenticated discovery and test inference against that exact route, with bounded size/time and sanitized error classification. Setup test attempts are explicitly identified separately from future Thread/Turn attempts. S04 consumes the same routing/attempt behavior when its actual Codex lifecycle exists; S03 cannot claim S04's in-flight Turn acceptance.
- Persist routing preference, explicit fallback consent/version and attempt provenance, never response secrets or raw diagnostic bodies. A manual retry/switch creates a new immutable attempt; optional eligible fallback targets DeepSeek only, requires a verified target and explicit consent, and exposes both attempts. No automatic switch back to ChatGPT. Missing quota metadata stays unavailable; cooldown uses observed bounds and cannot be cleared by dismissing UI.
- Risk Defaults here owns the UI-specified seven fields: maximum order notional, single-instrument exposure, daily traded notional, daily realized loss, stale quote threshold, market-order policy and live inactivity timeout. Store explicitly chosen defaults with units/version; money remains exact decimal, market orders default OFF and inactivity defaults to the PRD's 20 minutes. No default monetary appetite is invented. S21 still owns full per-account deterministic enforcement and all additional §21 fields; this setup record does not become execution eligibility.
- Ready means onboarding completed with current usable model evidence and explicitly reviewed setup, not that future trading/runtime slices are implemented. Keep Send unavailable until S04 supplies an operational Codex runtime, with an accurate separate reason; keep Live execution unavailable until its full gates exist.

## Testing Decisions

- Highest existing seam: real public Control Plane command/result/event/snapshot/replay and temporary SQLite reopen. Drive model operations through the same production orchestration used by Tauri; substitute only external process/network/credential boundaries for deterministic faults.
- Verify actual pinned binary in an isolated model directory: startup/probe, downstream unauthorized access, port conflict with a separate owned listener, exit/restart/backoff, cleanup, malformed/incomplete discovery and time/size bounds. Actual successful upstream inference and OAuth remain separate acceptance evidence and cannot be replaced by fixtures.
- Exercise immutable route attempts, default-OFF/no silent switch, explicit opt-in, cooldown eligibility, 401/429/network failures, stale operation completion, restart and concurrent independent account commands. Check original failed attempts and unchanged financial state through public projections.
- Scan SQLite, events, normal files, logs and screenshots/accessibility output for disposable secret sentinels; validate real native Keychain lifecycle separately and never use user broker credentials.
- Use real Rust-backed browser tests for all setup steps, reload/back navigation, exact validation, error recovery, fallback disclosure and 768/390 layout. Verify native secure entry/OAuth handoff and focus/cancel in the actual desktop bundle. Run required repository checks and serial Standards then Spec review per completed ticket.

## Out of Scope

S04 Codex Thread/Turn/tool execution, S21 complete deterministic risk enforcement, S22 arming/financial approvals, S27 full process recovery matrix, S32 complete diagnostics, and S34–35 signed release remain later mandatory map items. No extra model provider, direct external LLM client, automatic main merge or substitution of fixture success for actual authenticated inference.

## Further Notes

Authority: PRD §16, §21, §26.3, §27 and AC-038/055–057; UI A1–A6 and §14.8–14.9; Frontend §15.3; Backend §10 and §41–42. FR-068–071/073 and AC-055/057 are S03-owned; FR-044, AC-038/056, SEC-007/008 and UX-009 remain cross-stage until their other owners pass.

Official compatibility evidence: [pinned release](https://github.com/router-for-me/CLIProxyAPI/releases/tag/v7.2.155), [pinned configuration](https://github.com/router-for-me/CLIProxyAPI/blob/v7.2.155/config.example.yaml). Darwin aarch64 archive SHA-256: `f90c503ce41a798c85b6f61dfe5fe8b812c1b889634f0c80d04ee376424fe305`; Darwin amd64: `198794a2fafb9fb8083476ac18232647c57d443422aa3f008d19ed7e75ca4604`. These are release metadata, not proof of runtime compatibility.

## Gateway compatibility observations — 2026-09-09

The downloaded aarch64 archive matches the official digest above. Its executable SHA-256 is `29d978064c49874a54126b161266b6b8a42b971a7d1ffcf742920761333d40d0`; `--help` reports version `7.2.155`, commit `7fac6b15`, build time `2026-09-08T17:38:53Z`. The executable supports `-config`, `-local-model` and `-codex-login`. No authenticated provider was configured in this check.

An isolated real process using a private directory, empty environment except dedicated HOME/TMPDIR and system PATH, explicit loopback/secret configuration and `-local-model` produced:

- an owned listener at `127.0.0.1:8317`, confirmed against the child PID;
- HTTP 401 without downstream authorization;
- HTTP 200 with authorization and an empty model array; this is **not** a usable model route;
- HTTP 404 for the disabled management configuration endpoint;
- configuration mode 0600 and exit code 0 after termination;
- only the captured process log remaining after caller cleanup, with no generated downstream secret reflected in it.

Local diagnostic evidence: `.artifacts/s03-planning/runtime-probe/results.json` and `process.log`. This does not verify application integration, restart supervision, successful OAuth/inference, complete secret containment or ticket acceptance.

Pinned source inspection changes the launch requirements: startup reads `.env` from its current working directory, so use a private working directory as well as cleared inherited environment. `commercial-mode: true` omits request-logging middleware, while disabling ordinary request logging alone can still permit error files. Disable `quota-exceeded.switch-project` and `switch-preview-model` explicitly as well as retry rounds. Use `-local-model` to prevent automatic remote catalog updates; discovered authorization remains required despite an embedded catalog. References: [entrypoint](https://github.com/router-for-me/CLIProxyAPI/blob/v7.2.155/cmd/server/main.go), [server middleware](https://github.com/router-for-me/CLIProxyAPI/blob/v7.2.155/internal/api/server.go), and the pinned configuration above.

### Pending product clarification for the later connection ticket

The [current DeepSeek quick start](https://api-docs.deepseek.com/) lists `deepseek-v4-flash` / `deepseek-v4-pro`, whereas the [official indexed deprecation notice](https://api-docs.deepseek.com/guides/function_calling/) names 2026-07-24 as the deprecation date for `deepseek-chat` / `deepseek-reasoner` and describes their former mapping to Flash non-thinking/thinking modes. This conflicts with RevC's exact model names. A user clarification is pending on updating both language editions to the current Flash model's two modes. No model substitution or normative edit is authorized by this observation alone; the gateway lifecycle ticket can proceed independently.
