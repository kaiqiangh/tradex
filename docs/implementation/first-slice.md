# 首个垂直切片：打开并恢复本地桌面工作区

状态：已发布为 Spec #3 / implementation #4，S01 已完成实现、验证及串行审查。计划起点为 `9255feae39b646245acaf6db7db29fea0cb710c7`；实际 dev 审查起始 SHA 为 `177edd3a91980a4686699d281345df231625a8ce`。验收进度见 [S01 evidence](s01-evidence.md)。

## Problem Statement

用户目前只能点击无持久化的 HTML 原型，无法启动真实 TradeX 桌面工作区、保留本地身份或辨别应用运行时状态。后续研究与金融操作需要一个可验证、失败时状态明确的本地应用入口。

## Solution

提供可启动的 Tauri 应用，以真实 Rust Control Plane 打开或重新打开本地工作区。界面展示权威工作区信息和尚未配置的模型状态；主导航通过键盘和窄窗口可达。业务状态经唯一版本化 IPC 契约读取，保存到真实 SQLite，事件可恢复。

## User Stories

1. As a TradeX user, I want to open a local workspace, so that I can begin work in an actual desktop application.
2. As a returning user, I want the same workspace identity after restarting, so that subsequent work belongs to a stable workspace.
3. As a user, I want a clear workspace error if storage cannot be opened, so that I can recover without believing data was saved.
4. As a user, I want to see that the model runtime is not configured, so that I understand why Agent Send and onboarding Ready are unavailable.
5. As a keyboard user, I want visible focus and reachable primary navigation, so that I can use the application without a pointer.
6. As a user with a narrow window, I want access to New Thread, Threads, Markets, Watchlists, Accounts, Strategies, Artifacts and Settings, so that navigation remains available.
7. As a user, I want empty feature areas to explain their current availability, so that absent accounts and runtime data are not represented as successful fixture results.
8. As a user reconnecting the UI, I want its state to recover from a consistent snapshot and retained events, so that disconnected state cannot appear authoritative.
9. As a user, I want incompatible runtime versions to show an actionable error, so that the UI cannot silently misinterpret state.
10. As a user, I want application data stored separately from broker/model credentials, so that opening a workspace does not expose secrets.

## Implementation Decisions

- Tauri hosts React/TypeScript and Rust. One application crate is sufficient at this stage; the later mandatory independent Gateway is not represented by a placeholder authority module.
- Introduce the canonical version-1 command/result/error/event envelopes defined by Backend ARD. Define concrete workspace/runtime/domain command payloads in the authoritative bilingual contract before implementing them; derive or validate both language types from the same schema.
- Implement only the operations used by this slice: workspace.open, runtime.status, domain.snapshot and domain.subscribe. Unknown commands/schema/payloads fail explicitly. Pure UI navigation does not create backend operations.
- Open/create one workspace at a user-selected directory with a stable generated identity, schema version and timestamps. SQLite uses foreign keys, WAL and explicit migrations; backup/integrity behavior precedes changes to an existing schema. Only non-secret metadata is stored.
- Persist domain changes and per-aggregate sequential outbox events atomically. Subscribe replays original envelopes and continues without a snapshot/live gap. Snapshot returns a consistent projection and lastSequence. UI deduplicates exact events, reloads on gaps/conflicts and fails visibly on unsupported event schemas.
- runtime.status returns observed unconfigured/stopped components without launching unimplemented sidecars or claiming health. First-run workspace progress is real; later provider/model/risk steps remain explicitly incomplete until their own slices land.
- Match the PRD primary navigation and prototype visual tokens. Use native semantic controls, visible focus, scrolling and responsive navigation. Feature areas outside this slice show honest empty/unavailable states.
- No fake trading data, no automatically armed account, no financial mutations or external telemetry in this slice.

## Testing Decisions

- Primary seam: invoke the actual Rust command dispatcher against an isolated on-disk workspace, capture real outbox events and reopen the store to prove persisted identity. This is new because the repository has no application or tests.
- Exercise unsupported schema/command, malformed payload, inaccessible/corrupt storage, unknown aggregate and replay recovery through that public boundary; verify no mutation on rejection.
- Exercise a persisted change/outbox transaction, preserved replay identity/sequence and snapshot cursor consistency; tests must fail if persistence or event ordering breaks, rather than assert internal helper names.
- Run the built desktop application to verify the real Tauri invoke bridge and open/reopen flow. Browser-driven UI checks may use the same dispatcher through a test transport for automation; that transport alone is not proof of Tauri integration.
- UI behavior checks cover accessible navigation at desktop/768/390 widths and runtime-unavailable Send/Ready behavior. No claim of final full-application accessibility/performance acceptance.
- Proposed review fixed point: the captured dev SHA immediately before this ticket's first implementation change. Standards then Spec review run serially after relevant checks pass.

## Out of Scope

This first slice does not implement account connections, actual model runtime, full Thread workflow, market data, simulation or Live execution. Those remain in the complete map and are not considered complete by this slice. It does not introduce an alternative web-only application architecture.

## Further Notes

The English RevC documents remain normative. Related requirements are partial contributions to FR-007/034/044/064, UX-006/007 and NFR-015/016; broader requirements close only after their full mapped work is verified. Backend IPC/storage contracts are also explicit unnumbered architecture requirements.

## Proposed ticket breakdown

1. **打开并恢复本地桌面工作区** — Blocked by: the completed requirements/serial-plan decision. Delivers the complete narrow path above: schema → Rust/SQLite → Tauri UI → recovery/event checks. One implementation ticket; no separate horizontal schema/UI/database tickets.

用户确认点：上述测试边界、首票粒度，以及按各票实现前的 dev SHA 做串行双轴审查。其他工作项在进入时依此约定继续细化，避免一次发布未经研究的提供方实现细节。
