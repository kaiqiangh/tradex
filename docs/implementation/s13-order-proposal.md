# S13 编辑订单草稿并生成不可变 Proposal 规范

日期：2026-09-20  
前置：S02 provider/environment catalog、S07 canonical market identity、S08 TimeService/market state、S09 account/portfolio read  
状态：SPEC（待拆票与实现）

## Problem Statement

TradeX 当前能够展示只读的 Trade proposal 入口和原型订单卡片，但没有可持久化的 OrderDraft 编辑流程，也没有由受信后端生成并保存的不可变 OrderProposal。原型中的 proposal identity 会按 instrument 固定并在刷新时复用，数量或 policy 变化不能可靠地形成新 revision；用户无法检查订单字段、规范化后的 decimal、适用的 environment/TIF 和新旧 proposal 的关系。后续审批、reservation、Gateway 若绑定这种可变对象，会把旧批准错误地带到新订单上。

S13 需要建立 draft → proposal 的唯一身份边界。它必须支持完整的订单字段、tagged quantity、规范 decimal、draft version、canonical proposal hash 和旧 proposal 的审计可见性，同时保持金融权限隔离：生成 proposal 不等于批准、arming、预留或提交订单。

## Solution

在现有 Control Plane、版本化 Backend ARD IPC 和 workspace SQLite projection 上增加 OrderDraft/OrderProposal 服务。用户可保存或更新一个有版本的 editable OrderDraft；Generate Proposal 对当前 draft 做 canonical validation，冻结其字段，绑定可用的 policy version 与 market snapshot reference，生成新的 opaque proposal ID 和 SHA-256 proposal hash，并把 proposal 保存为不可变记录。任何 material edit、刷新或 stale revalidation 都只产生新的 proposal identity，并把旧 proposal 标记为历史/失效原因，绝不就地改写。

Trade UI 提供 Draft editor、Generate Proposal、Proposal detail、Edit/Refresh 和明确的 blocked/stale/error 状态。Proposal detail 只读地展示 account、environment、venue、instrument、side、order type、quantity semantics、price/max spend、TIF、estimated notional、policy/snapshot references 和 proposal identity。S13 结束时 proposal 仍停留在 `NEEDS_APPROVAL` 或等价的待授权状态；任何执行相关按钮都不可用，后续 S21–S24 才能消费该身份。

## User Stories

1. 作为研究或交易用户，我想创建一个 workspace-scoped OrderDraft，以便在未授权的情况下逐步填写订单。
2. 作为用户，我想编辑 account、environment、venue 和 canonical instrument，以便订单目标清楚且不会被默认 AAPL/BTC 替换。
3. 作为用户，我想选择 BUY 或 SELL，以便订单方向明确。
4. 作为用户，我想选择 MARKET 或 LIMIT order type，以便价格字段按订单类型显示和校验。
5. 作为用户，我想使用 BASE 或 QUOTE tagged quantity，以便明确数量语义，不把 quote notional 当作 base quantity。
6. 作为用户，我想输入规范 decimal 数值，以便前导零、尾随零、指数格式和负数都按统一规则处理。
7. 作为用户，我想填写 limit price 或适用的 maximum spend，以便生成的 proposal 反映真实授权边界。
8. 作为用户，我想选择受支持的 time-in-force，以便 Draft 不携带 provider 无法识别的 TIF。
9. 作为用户，我想在字段缺失、格式错误、精度超限、数量为零或字段组合不兼容时看到字段级错误，以便在生成前修正订单。
10. 作为用户，我想看到 account/environment/instrument 的不兼容原因，以便不会把 Demo、Testnet 和 Live 混淆。
11. 作为用户，我想保存 Draft 并看到 draft version，以便知道下次更新针对的是哪一个状态。
12. 作为用户，我想点击 Generate Proposal，以便把当前可见且已校验的 Draft 冻结为独立对象。
13. 作为用户，我想看到新 proposal ID、proposal hash、policy version 和 market snapshot reference，以便审阅生成依据。
14. 作为用户，我想在 Proposal detail 中看到完整 immutable order fields，以便后续授权前检查每一项。
15. 作为用户，我想看到 estimated notional 与 currency，以便知道数量和价格形成的规模。
16. 作为用户，我想在生成 Proposal 后继续查看原 Draft，以便知道 Proposal 来自哪一版编辑内容。
17. 作为用户，我想修改一个 material field 后看到旧 Proposal 已失效，以便旧 approval 不会继续适用于新订单。
18. 作为用户，我想修改 instrument、side、quantity、account、order type、price、TIF、environment 或 quantity type 时得到新的 proposal identity，以便所有实质变化都可追溯。
19. 作为用户，我想修改非 material 的展示字段时不产生隐藏的金融变化，以便 identity 规则可解释。
20. 作为用户，我想点击 Refresh Proposal 处理 stale market/policy 状态，以便刷新产生新 hash 而不是悄悄改写旧 Proposal。
21. 作为用户，我想查看旧 Proposal 的 invalidation reason、draft version 和创建时间，以便审计为什么它不能继续使用。
22. 作为用户，我想在重新生成后看到需要新的 approval 的提示，以便不会把旧 consent 当作新 consent。
23. 作为用户，我想在当前 account、market snapshot 或 policy reference 不可用时看到明确的 blocked/stale 状态，以便不会把缺失数据当成成功校验。
24. 作为用户，我想在 Live context 下看到 proposal-only 的状态，以便 Trade mode 或 Live selection 不会自动 arm 或 submit。
25. 作为用户，我想在 Local Paper、Paper、Demo、Testnet 等非 Live context 下保留明确 environment identity，以便模拟/沙盒订单不会被称作 broker Live truth。
26. 作为用户，我想在切换 mode 或 execution context 后看到不兼容 Draft 被标记并要求修正，以便旧 context 不会污染新 proposal。
27. 作为用户，我想在 workspace 重新打开后继续看到自己的 Draft、Proposal 和 revision history，以便身份不依赖页面内存。
28. 作为并发用户，我想在 Draft version 过期时收到 state-version conflict 并重新加载权威状态，以便不会静默覆盖另一个编辑。
29. 作为安全审查者，我想确认 proposal 只能在所属 workspace、account 和 draft scope 内读取，以便跨 workspace 引用不能生成金融对象。
30. 作为安全审查者，我想确认 renderer 不能自造 proposal hash、policy version、market snapshot 或审批状态，以便权威字段由后端生成。
31. 作为安全审查者，我想确认 Proposal generation 不写 approval、reservation、execution attempt、Gateway 或 broker mutation，以便 S13 保持无权限边界。
32. 作为安全审查者，我想确认 secret、API key、Authorization header、Keychain bytes 和原始 provider payload 不进入 Draft/Proposal projection，以便订单身份不泄露凭据。
33. 作为键盘用户，我想用语义表单完成编辑、生成、查看和返回，并在 modal 关闭后恢复焦点，以便无需鼠标。
34. 作为窄屏用户，我想在 390px、768px 和 1280px 下看到全部必填字段、identity 和错误操作，以便不会因布局隐藏订单约束。
35. 作为审计者，我想看到 Draft revision、Proposal identity、hash、invalidation event 和 source references 的完整链条，以便重建用户看到的对象。
36. 作为后续 approval 实现者，我想使用稳定的 proposal_id/proposal_hash 和 state_version，以便 approval 能严格绑定一次不可变对象。

## Implementation Decisions

- OrderDraft 和 OrderProposal 都是 workspace-scoped domain projections。Draft 可更新并有单调 draft version；Proposal 是 append-only immutable projection，旧对象只追加状态/失效事件，不原地修改订单字段。
- 复用现有 Control Plane 和 Backend ARD §41–42 的严格 wire contract，提供三个 canonical command：`trade.save_draft`、`trade.generate_proposal`、`trade.refresh_proposal`。前者接收 workspace、可选 draft ID、draft fields 和 expected draft/state version；后两者分别接收 draft ID/version 或 proposal ID/state version，并返回权威 Draft/Proposal projection。
- Proposal 生成必须重新从 workspace 读取 Draft、account/context identity、已知 instrument/venue catalog、TimeService 状态和可用 policy/snapshot reference；前端提交的 hash、notional、policy、snapshot、status 或 approval 字段一律忽略或拒绝。
- Draft 的 canonical fields 至少包括 account ID、venue、environment、instrument ID、side、order type、tagged quantity（`BASE` 或 `QUOTE` 加规范 decimal string）、limit price 或 maximum spend、time-in-force、可选 client label，以及 workspace-scoped context references。只允许已定义 enum/opaque ID；不接受任意 provider payload。
- Proposal 的 immutable fields 至少包括 proposal ID、proposal hash、draft ID/version、account/venue/environment/instrument、side、order type、tagged quantity、price/max spend、TIF、estimated notional/currency、market snapshot ID（如适用）、policy version/reference、created time、status 和 invalidation metadata。状态起点是 `NEEDS_APPROVAL` 或等价待授权状态；S13 不创建 approval。
- 所有 financial decimal 以规范字符串传输和持久化，禁止 binary floating point 参与 canonical validation/hash。canonical serialization 使用固定字段顺序、明确 null/optional 规则、enum 规范大小写、quantity type/value、account/instrument/environment/TIF 和 decimal normalization；SHA-256 的输入只来自后端 canonical projection。
- Material edit 集合固定包含 instrument、venue、account、environment、side、order type、quantity type/value、limit price、maximum spend、TIF 和会改变 market/policy/snapshot 依据的 refresh。material edit 或 refresh 生成新的 proposal ID/hash；旧 proposal 与旧 approval（若未来存在）保持审计记录并变为不可用。
- `trade.refresh_proposal` 不在浏览器更新字段；它重新读取权威 proposal/draft、重新获取已存在的 market/time/policy references，生成新的 immutable identity，并返回旧 proposal 的 invalidation reason。外部 provider 无法读取时返回明确的 stale/unavailable result，不伪造刷新成功。
- Provider-specific precision/order-rule checks 只使用已有 capability/catalog seam。S13 可以阻止已知不兼容组合；真实 provider endpoint、签名、账户刷新和订单提交由 S17–S20 负责。缺少所需外部证据时返回 typed blocked/unavailable 状态，而不是把 fixture 当 provider truth。
- Proposal generation 在所有 execution contexts 保留原始 environment identity。Live context 仅可生成 proposal 预览并保持 DISARMED；Local Paper/Paper/Demo/Testnet 也只保存 proposal，不能绕过后续 simulator/provider authority。
- SQLite migration 增加 Draft、immutable Proposal 和 proposal event/history projection，约束 workspace scope、唯一 proposal hash、draft version/state version 与 parent identity。打开旧 workspace 时迁移失败要 fail closed，不得丢失现有 workspace。
- UI 复用现有 AppShell、Trade context/capability decision 和 typed error rendering，新增 Draft editor、ProposalCard/Detail、revision history 与 stale/blocked states。Approval/Arm/Place controls 不在本切片启用；任何只读入口都明确标注 proposal-only。
- 原型中的 synthetic proposal ID/hash 仅可用于 labelled fixture UI；生产 UI 必须显示后端 canonical identity，不能凭固定 fixture ID 宣告完成。
- 最高验证 seam 是真实桌面 UI → 版本化 Rust dispatcher → 临时 workspace SQLite → 重开后的 Draft/Proposal UI。浏览器 fixture 只辅助键盘、响应式和文案断言，不证明 proposal persistence 或 provider truth。

## IPC Contract

逻辑契约如下，具体 schema 版本由 Backend ARD §41–42 定义：

```text
trade.save_draft({
  workspaceId,
  draftId?,
  expectedStateVersion?,
  draft
}) -> OrderDraft

trade.generate_proposal({
  workspaceId,
  draftId,
  expectedDraftVersion
}) -> OrderProposal

trade.refresh_proposal({
  workspaceId,
  proposalId,
  expectedStateVersion
}) -> {
  previousProposal,
  proposal,
  invalidationReason
}
```

`OrderDraft` 至少返回 workspace ID、draft ID/version、state version、完整 canonical draft fields、updatedAt 和 validation status/errors。`OrderProposal` 至少返回 workspace ID、proposal ID/hash、draft ID/version、immutable order fields、policy/snapshot references、status、createdAt 和 optional invalidation metadata。未知字段、跨 workspace identity、空/超长字符串、非法 decimal、未来 state version 和重复/不匹配 parent identity 必须在 dispatch 前失败。

## Testing Decisions

- Rust protocol/schema tests 验证 command names、camelCase wire fields、deny-unknown-fields、enum bounds、ID/length limits、decimal normalization、BASE/QUOTE quantity、TIF/order-type combinations、state version and workspace checks。
- Draft service tests 验证 create/update/version conflict、完整字段 round-trip、material vs non-material edit、workspace isolation、reopen persistence、malformed/empty values 和不泄露秘密。
- Proposal service tests 验证 canonical serialization 稳定性、相同 canonical Draft 遵守唯一 hash 的幂等/复用策略、material edit/refresh 产生不同 ID/hash、旧 Proposal 字段不变、旧 proposal/approval 不可用、invalidation event 可检索、unique hash 与父版本约束生效。
- Market/time/policy seam tests 验证 snapshot/policy reference 缺失、stale、clock uncertain、unsupported instrument/environment 时返回 typed blocked/stale 结果；不调用 provider network，也不写 accounts、risk decisions、approval、reservation、outbox 或 Gateway state。
- IPC integration tests 通过真实 dispatcher 验证 save → generate → edit → generate、refresh、state-version conflict、cross-workspace rejection、reopen 和 unknown-field errors；超时后先查询权威状态，不能把 retry 变成重复 consent。
- Rust-backed browser/native UI tests 验证 Draft editor 的字段级错误、proposal read-only identity、Edit/Refresh invalidation、history、Trade/Live proposal-only gating、keyboard focus trap/restore、390/768/1280 layout 和无 console error。
- S13 不使用真实 provider/OAuth/API key 或 broker submission 作为通过条件；provider truth、approval TTL、risk evaluation、reservation conflict、Gateway dispatch、fills/reconciliation 由后续切片验证。
- 收尾运行 schema check、TypeScript typecheck/build、Rust unit/integration、clippy、fmt、diff check、requirements traceability 和 UI helper syntax；证据必须绑定 exact SHA，并区分 fixture、desktop runtime、provider 和 S33 全量回归边界。

## Out of Scope

- Financial approval/consent、account arming/disarming、risk decision、policy editing、capacity reservation、execution attempt、Order Gateway、broker submission/cancel/fill/reconciliation。
- Alpaca、Trading 212、Binance、Bitget 的真实认证、签名、能力探测、订单 endpoint、private stream 和 provider-specific execution；这些由 S17–S20 实现。
- Local Paper simulator 的 fill/position/cash mutation（S16）。
- Strategy、Backtest、workspace restore/export、artifact creation、full history search、cloud sync、sharing、background draft autosave。
- Broker-native amend/replace；v1.0 的修改路径仍是取消确认后创建新 proposal，属于后续 cancellation/execution slices。
- 真实市场数据 entitlement、完整 risk policy semantics、approval market snapshot display 和 S22/S23/S24 authority consumers。
- 将模型输出、浏览器状态或固定 prototype fixture 直接提升为 proposal identity、policy authority 或 execution permission。
- S33 全量页面/辅助技术回归、S34 性能/签名包和最终 dev → main PR。

## Further Notes

- 权威来源是 PRD §§18–23、§43、FR-019/FR-080、AC-013/AC-032/AC-065、NFR-011/NFR-014、DATA-002、UX-003；UI Spec §§14.2–14.5、14.9–14.10；Backend ARD §§19、20、23、41–42；Frontend ARD §14 与其 OrderProposalCard/command registry；Coverage Matrix FR-019/FR-080/AC-032；QA Report QA-07。
- QA-07 当前状态是 FAILED：原型会复用 identity，且 editable OrderDraft → Generate Proposal 流程不完整。S13 的验收必须使用不同 revision IDs/hashes、不可变旧对象、显式 approval invalidation 和可用 Draft editor 的行为断言。
- S13 只解决 proposal identity boundary；后续票据必须以 proposal_id/proposal_hash 为输入，不能新造 draft/approval aliases 或把 proposal generation 当作 execution capability。
- 该规范应先拆成一个可运行的 tracer-bullet ticket，优先打通 schema、Rust storage/service、canonical IPC、Draft/Proposal UI 和最小桌面/浏览器验证；完成并记录真实证据后再进入 S14。
