# S13 #43 OrderProposal 生成与身份历史规范

日期：2026-09-20  
前置：S13 #42 OrderDraft persistence/editor；S02 provider/environment catalog、S07 canonical market identity、S08 TimeService/market state、S09 risk/account read  
范围：从已保存 Draft 生成后端权威的 immutable OrderProposal，并在 Draft 页面展示 Proposal detail/history。

## Problem Statement

TradeX 已能保存 workspace-scoped、带版本的 OrderDraft，但还没有后端生成的不可变订单身份。若 renderer 自己计算 hash、notional、policy 或 market snapshot，后续 approval 可能把旧字段误认为当前意图；原型还会按 instrument 复用固定 proposal identity，无法保留 revision 和失效原因。

## Solution

新增严格 typed 的 `trade.generate_proposal` 生成入口，以及只读的 Proposal library/detail 查询。Control Plane 重新读取当前 workspace 内的 Draft、canonical instrument/context、默认或已保存的 risk policy reference 和当前 market/time observation，规范化后端字段，计算 deterministic proposal hash，生成 opaque proposal ID，并以 `NEEDS_APPROVAL` 的 proposal-only 状态保存。

Proposal 的订单字段和生成依据只写入 immutable projection。Draft 的 material edit 不改写旧 Proposal，而是追加 `DRAFT_CHANGED` history event；重新生成得到新的 proposal identity。非 material 的 client label 变化不使金融身份复用旧字段之外的 proposal 失效。相同 Draft version、相同 canonical fields 和相同 authority references 的重复生成返回既有 Proposal，不产生重复记录。

没有可用的市场快照或已配置 policy 时，Proposal 仍明确保存 `UNAVAILABLE`/`UNCONFIGURED` reference status 和原因；它可以用于只读审阅，但不能被本切片解释为已通过风险或可以执行。浏览器 fixture 可以提供标记清楚的本地 observation，不伪造真实 provider truth。

## User Stories

1. 作为用户，我想从已保存的 Draft 生成 Proposal，以便在授权前查看冻结的订单意图。
2. 作为用户，我想看到由后端生成的 proposal ID 和 SHA-256 proposal hash，以便确认身份没有来自 renderer 的可变输入。
3. 作为用户，我想看到完整的 account、venue、environment、instrument、side、order type、quantity semantics、price/max spend 和 TIF，以便逐字段审阅。
4. 作为用户，我想看到 estimated notional 与 currency，以便理解该订单的规模；无法可靠计算时应看到 unavailable 原因。
5. 作为用户，我想看到 policy version/state reference 和 market snapshot reference/status，以便知道 Proposal 依据和缺失信息。
6. 作为用户，我想看到 Proposal 状态为 `NEEDS_APPROVAL` 且明确标注 proposal-only，以便不会把生成误认为批准或下单。
7. 作为用户，我想重复生成同一个 Draft version 时得到同一 Proposal，以便重试不会制造重复金融意图。
8. 作为用户，我想修改 Draft 的数量、instrument、venue、account、environment、side、order type、quantity type、price/max spend 或 TIF 后看到旧 Proposal 已失效，以便旧 consent 不会适用于新字段。
9. 作为用户，我想在修改后重新生成新的 Proposal identity/hash，以便 revision 可追溯。
10. 作为用户，我想查看旧 Proposal 的 immutable fields、draft version、创建时间和 invalidation reason，以便重建历史。
11. 作为并发用户，我想在 Draft version 过期时收到 `STATE_VERSION_CONFLICT`，以便不会从旧 Draft 生成身份。
12. 作为安全审查者，我想确认跨 workspace、未知字段、伪造 hash/notional/status、未知 Draft 和不匹配 parent version 都在 dispatcher 边界被拒绝。
13. 作为安全审查者，我想确认 Proposal 生成只写 Proposal/history projection，不修改 account、risk policy、approval、arming、reservation、outbox、Gateway 或 broker state。
14. 作为键盘用户，我想用 Enter 生成 Proposal、浏览历史并读取 detail，且不出现隐式 Approve/Arm/Place 操作。
15. 作为窄屏用户，我想在 390px、768px、1280px 下看到 Proposal identity、状态、字段和错误而不发生横向溢出。

## Implementation Decisions

- IPC 增加 `trade.generate_proposal({ workspaceId, draftId, expectedDraftVersion }) -> OrderProposal`；增加 workspace-scoped `trade.proposal.list` 和 `trade.proposal.get` 只读查询。输入仅允许 Draft identity/version；hash、notional、policy、snapshot、status、approval 字段属于后端生成字段，未知字段拒绝。
- Proposal projection 至少包含 workspace/draft identity、draft version、完整 `OrderDraftFields` 快照、proposal ID、proposal hash、estimated notional/currency（可空并带原因）、policy version/state reference/status、market snapshot ID/status/reason、`NEEDS_APPROVAL` 或 `INVALIDATED` 状态、created/state versions 和 optional invalidation reason。
- Hash 的 canonical input 使用固定 Rust struct 字段顺序、明确 optional/null 规则、enum 的规范大写、normalized decimal、account/instrument/environment/TIF、tagged quantity、draft ID/version 及当前 authority reference；client label 作为显示字段保存但不改变金融 identity。SHA-256 只由受信 Control Plane 计算。
- 相同 workspace、Draft ID/version、canonical financial fields 和 authority references 的请求复用现有 active Proposal。旧 Proposal 已被 material edit invalidation 后，后续 Draft version 通过新的 version-bound canonical input 生成新的 ID/hash，不能复活旧对象。
- SQLite 新增 Proposal immutable projection 与 append-only proposal history/event projection。Proposal 行的 order fields/hash 不做 UPDATE；Draft 保存事务在 material fields 变化时为关联 active Proposal 追加 `DRAFT_CHANGED` 事件。读取时由后端组合当前 status/history，按 workspace、draft 和 sequence 校验完整性。
- Material fields 固定为 instrument、venue、account、environment、side、order type、quantity type/value、limit price、maximum spend 和 TIF。client label 属于非 material display metadata，不触发旧 Proposal 失效。
- Estimated notional 使用现有 exact decimal seam：BASE + LIMIT 使用 quantity × limit price；QUOTE quantity 使用 quote amount；MARKET 只有 maximum spend 或可信 quote 时才计算，否则返回 nullable amount 与 explicit unavailable reason。禁止 binary floating point 和 provider network。
- Policy reference 来自当前 workspace 的 persisted risk state；尚未配置时使用受信默认版本并标出 `UNCONFIGURED`，不创建或修改 risk state。Market reference 来自现有 market/time seam；缺失、stale、blocked 或 fixture observation 都保留 status/reason，不能把 unavailable 转成 quote truth。
- Proposal UI 嵌入 Order Draft 页面：保存 Draft 后显示 Generate Proposal；生成后展示只读 ProposalCard/detail 和 Draft→Proposal history，旧 Proposal 显示 invalidation reason。页面不渲染 Approve、Arm、Place、Reserve 或 execution CTA。
- Live、Paper、Demo、Testnet 和 Local Paper 保留原始 execution context。Proposal 生成只建立审阅对象；Live arming、financial approval、risk evaluation、reservation、Gateway 和 provider adapter 由后续切片消费稳定的 proposal ID/hash。

## Testing Decisions

- Rust protocol/schema tests 只验证外部 wire behavior：严格 command/input/output、deny-unknown-fields、canonical enum/decimal、optional reference states 和 sanitized errors。
- Storage/dispatcher integration tests 使用真实临时 SQLite，覆盖 save Draft → generate → reopen、deterministic hash、duplicate generation idempotence、material edit invalidation、old projection immutability、history ordering、cross-workspace/unknown Draft/old version/forged fields rejection。
- Tests must assert that account/risk/approval/arming/reservation/outbox/provider observations are unchanged before and after generation; no HTTP or Keychain call is allowed on this path.
- UI tests use the existing Rust-backed browser bridge and semantic controls: Generate/Proposal detail/history, `NEEDS_APPROVAL` proposal-only text, invalidated old identity, keyboard Enter, 390/768/1280 overflow checks and zero console errors. Fixture labels remain explicit and do not establish provider truth.
- Native compilation is a build gate; a usable native window run is recorded separately from browser fixture evidence. Real provider/OAuth/API keys and broker submission are out of scope.

## Out of Scope

- `trade.refresh_proposal` stale market/policy revalidation (S13 #44).
- Financial approval/consent, policy evaluation, account arming, reservation, execution attempts, Gateway dispatch, broker submission/cancel/fill/reconciliation.
- Provider-specific quote/precision/order endpoints and external market-data entitlement; unavailable references remain explicit.
- Local Paper fills/positions/cash mutation, strategy/backtest, artifacts, cloud sync and auto-save.
- Full S33 accessibility/runtime regression, signed package release and final `dev → main` PR.

## Further Notes

- Normative sources: PRD §19 and §18; UI Spec §14.2 and §14.3; Backend ARD §§19.2–19.4 and §§41–42; Frontend ARD §§14.2–14.5; Coverage Matrix FR-019/FR-080/AC-032/AC-065; QA-07.
- The prototype’s fixed identity is a known FAILED baseline. Passing this slice requires different revision IDs/hashes, immutable old order fields, inspectable invalidation history and a real dispatcher-to-temporary-SQLite seam.
- This spec consumes #42 and must complete before #44 or any approval/execution ticket begins.
