# S13 #44 stale Proposal 刷新与失效安全回归规范

日期：2026-09-20  
前置：S13 #43 immutable OrderProposal；S08 TimeService/market state；S09 risk/account read  
范围：对当前 Proposal 做后端权威 Refresh，保留旧对象并生成新的 proposal identity。

## Problem Statement

S13 #43 已能从 Draft 生成不可变 Proposal，但 Proposal 生成后 policy、market reference 或可信时间状态可能变化。继续使用旧 Proposal 会把过期的依据带到后续 consent；浏览器也没有可审计的 Refresh 结果。刷新必须重新读取权威 Draft、policy、market 和 TimeService 状态，旧 Proposal 只追加失效事件，新对象仍保持 proposal-only。

## Solution

增加 typed `trade.refresh_proposal`。Control Plane 在 active workspace 内按 `proposalId + expectedStateVersion` 读取当前 Proposal，确认它仍为 `NEEDS_APPROVAL` 且关联 Draft 版本未被 material edit 失效，然后重新计算 policy/market/time references，创建新的 immutable Proposal，并把旧对象追加为 `REFRESHED`。旧字段、hash、created time 永不更新；结果同时返回旧对象、新对象、刷新状态和失效原因。

刷新结果的 `refreshStatus` 是后端权威状态：

- `REFRESHED`：当前 references 可重新读取且没有 blocked/stale/unavailable gate。
- `STALE`：TimeService 为 `CLOCK_UNCERTAIN` 或 `STALE`，或已有 reference 明确过期。
- `BLOCKED`：policy 未配置或 market entitlement/source 为 `BLOCKED_EXTERNAL`。
- `UNAVAILABLE`：policy/market/time reference 无法读取或返回 `UNAVAILABLE`/`UNVERIFIED`。

`STALE`、`BLOCKED`、`UNAVAILABLE` 仍可返回一个只读的新 Proposal 以保留用户审阅链，但不能被解释为可授权或可执行；UI 必须显示状态、原因和“需要新的 approval”语义。Refresh 不调用 provider network、Keychain、Gateway 或执行命令。

## IPC Contract

```text
trade.refresh_proposal({
  workspaceId,
  proposalId,
  expectedStateVersion
}) -> OrderProposalRefreshResult

OrderProposalRefreshResult {
  previousProposal: OrderProposal,   // REFRESHED / INVALIDATED history
  proposal: OrderProposal,            // new identity, NEEDS_APPROVAL
  refreshStatus: REFRESHED | STALE | BLOCKED | UNAVAILABLE,
  invalidationReason: string
}
```

输入只接受 workspace、opaque proposal ID 和当前 `stateVersion`，`previousProposal`、`proposal`、hash、notional、reference、status、reason 等输出字段不能由 renderer 提交。未知字段、跨 workspace、未知 proposal、旧 state version、空或过长 identity 必须在 dispatcher 边界失败。

## State and storage decisions

- Refresh 只允许当前事件流仍为 `NEEDS_APPROVAL` 的 Proposal。已被 `DRAFT_CHANGED` 或先前 `REFRESHED` 失效的对象返回 typed `ORDER_PROPOSAL_NOT_REFRESHABLE`，不追加部分事件。
- Refresh 重新读取 Proposal 的 Draft；若 Draft 不存在或版本已变化，返回 `STATE_VERSION_CONFLICT`/`ORDER_DRAFT_NOT_FOUND`，Proposal、Draft、risk、account、outbox、Gateway 与 provider 状态保持不变。
- 新 Proposal 复用 #43 的 immutable projection、fixed Rust canonical hash 和 exact decimal notional；刷新路径把新的 TimeService observation 纳入 market reference reason，使显式 Refresh 得到不同 hash/ID，即使外部 source 仍 blocked。
- 旧 Proposal 追加 `REFRESHED` event，reason 必须包含 replacement proposal ID；新 Proposal 只追加 `GENERATED`。读取时校验 event sequence、workspace、reason 长度、projection identity/hash 和 canonical hash。
- Transaction 使用 SQLite immediate boundary：读取/校验 expected state、追加旧事件、插入新 projection/event、commit 必须是一个原子操作。任一失败都不留下旧失效或孤立新行。
- Refresh 不修改 account、risk policy、workspace aggregate sequence、approval、arming、reservation、execution attempt、outbox、Gateway 或 broker state。

## UI contract

- Proposal detail 对 `NEEDS_APPROVAL` 提供 `Refresh proposal`；不显示 Approve、Arm、Place、Reserve 或执行 CTA。
- Refresh 成功后同时展示 refresh status/reason、旧/new proposal identity、旧对象 `INVALIDATED`、新对象 `NEEDS_APPROVAL`，并提示必须重新取得 approval。
- blocked/stale/unavailable 结果保留新 Proposal 的只读字段和 reference 原因，禁止显示“ready”或隐式恢复旧 consent。
- 旧 Proposal history 显示 `REFRESHED`、发生时间、replacement proposal ID/reason；选择器保留旧/new 两行。
- 语义按钮和焦点恢复可用；390/768/1280 下 identity、status、reason 和 history 不横向溢出，console error/warn 为零。

## Verification boundary

Rust dispatcher/storage tests 使用真实临时 SQLite，至少覆盖：

1. active Proposal refresh 返回不同 ID/hash、旧 projection 字段不变、新对象为 `NEEDS_APPROVAL`，history 有 `REFRESHED` 与 replacement ID；
2. policy version 变化、TimeService untrusted、market blocked/unavailable 分别得到 `REFRESHED`/`STALE`/`BLOCKED`/`UNAVAILABLE` 状态与显式 reason；
3. duplicate/stale state、cross-workspace、unknown fields、unknown/invalidated proposal、draft version mismatch 和损坏 history 均失败且无部分写入；
4. account/risk/workspace sequence/outbox 等 snapshot 在 refresh 前后不变；reopen 后旧/new identity 与 history 仍可读取；
5. browser bridge 通过 semantic action 验证 Refresh、旧/new detail、keyboard、390/768/1280 overflow 和 console clean。

运行收尾必须包含 schema check、TypeScript typecheck/build/unit、Rust workspace tests、clippy、fmt、diff check、requirements traceability、desktop compile；证据绑定 exact SHA，并明确 fixture、原生 runtime、provider/OAuth、approval/Gateway 与 S33 边界。

## Out of scope

Approval/consent、risk evaluation、arming、reservation、Gateway dispatch、provider quote/order/cancel/fill、Local Paper fills、真实 OAuth/API key、broker mutation、S33 全量回归和 `dev → main` PR。
