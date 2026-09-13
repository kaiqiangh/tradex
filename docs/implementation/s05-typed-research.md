# S05 / #23 Typed research tool/result boundary

状态：**SPEC READY（2026-09-13）**。本票从 #22 验收 SHA `23bd587d8bf3e511dad0213e206f9e63898e1e3b` 开始。

## Problem

S05 已经能依据 Agent Mode、Execution Context、账户健康和 canonical context refs 计算能力等级，但 research tool 仍只是展示在总能力列表中的字符串。这样无法证明研究请求经过 data-plane 白名单、结果带有来源与上下文身份，也无法证明结果被篡改时不会进入 Turn 或升级为金融能力。

## Solution

在 Control Plane 增加独立的 typed research registry。registry 只投影 `public_market_read`、`account_read`、`historical_simulation` 三个 data-plane IDs；`paper_demo_testnet_execution` 与 `live_order_proposal` 继续存在于总 capability decision，但永远不出现在 research registry。

新增 `research.run` 只读 IPC。它重新计算当前 workspace/mode/context/account/attached refs 的 capability decision，确认请求的 research tool 已被 registry 授权，然后返回固定结构的、无凭据且无外部 provider 数据的 unavailable result。结果包含 typed tool ID、canonical context refs、非秘密 source ID、request hash、result marker 和明确 payload 状态。研究 query 只用于 bounded hash，不回显到 result，避免 prompt injection 被当作指令。

`turn.start` 可携带成对的 research invocation/result。Control Plane 会从本轮快照字段重建 invocation，重新执行 registry 并逐字段比较结果；缺失、孤立或 tampered result 在任何 thread projection/runtime 之前返回 `RESEARCH_RESULT_INVALID`。通过校验的 result 作为 `research_result` timeline item 持久化，并把 marker 传入 runtime；fake 与真实本地 Codex seam 都只接收这个已清洗的 marker/summary，不获得 broker credential、model secret 或 direct external LLM endpoint。

## User stories

1. 作为研究用户，我希望只看到与当前模式匹配的 typed data-plane tools，这样研究请求不能变成下单、审批、arming 或 keychain 操作。
2. 作为研究用户，我希望 interim result 保留 source/context identity 和 marker，这样最终 Turn 能审计结果来自哪个 canonical context。
3. 作为维护者，我希望 unknown、current-market execution、prompt-injected financial text 和 tampered result 在公共 IPC 边界 fail closed，且不会产生 domain mutation。
4. 作为维护者，我希望 runtime 沿用本地 loopback gateway 与 sanitized environment，研究子流程不能携带 broker/model secrets 或直接访问外部 LLM。

## Implementation decisions

- 在 `capability.rs` 新增 `ResearchToolId`、`ResearchToolDefinition` 和 `CapabilityDecision.research_tools`；由总 `allowed_tools` 派生 registry，金融 authority IDs 永不映射。
- 在 `protocol.rs` 新增 `ResearchToolRequest`、`ResearchToolInvocation`、`ResearchToolPayload`、`ResearchToolResult`，并把 `research.run` 加入 generated IPC schema。所有字符串和向量有明确长度上限，result payload 使用 tagged typed fields，不接受 arbitrary JSON。
- 新增纯 `research` module。`run` 只读取已校验的 account/context 状态，返回 `UNAVAILABLE` payload；request hash/marker 由稳定的 workspace、mode、context、account、tool、query hash tuple 派生，不使用 credential material。
- `turn.start` 只接受 invocation/result 成对字段。Control Plane 在写入 Thread 前用同一纯函数重算并比较，成功后写入 `research_result` item（`source_id` 为 result source），runtime request 仅携带 marker。
- Composer 提供一个明确的 “Preview typed research result” action；它调用 `research.run`，显示结构化状态和 marker，并在下一次 Send 时提交成对 invocation/result。模式、执行上下文、账户或 context refs 改变时清除 preview。
- 不新增 market-data provider、research worker、broker command、risk/approval/arming/gateway mutation、credential path 或 direct network egress；真实市场事实由后续 S07/S12/S14/S15 slice 提供。

## Testing decisions

- Rust capability tests：每个 mode/context row 的 `research_tools` 仅含三种 data-plane IDs；Trade Paper/Demo/Testnet 与 Live 的 financial IDs 不会进入 registry，Live 仍 proposal-only。
- Public IPC tests：允许的 `research.run` 返回 bounded structured unavailable result；financial/unknown tool payload、current-market intent 和 prompt-injected query 均 fail closed 或返回无害固定 payload，且 snapshot sequence 不变。
- Turn tests：合法 result marker 出现在 persisted `research_result` item 和最终 fake Turn output；missing、mismatched request hash、marker 或 context refs 返回 `RESEARCH_RESULT_INVALID` 且不写入 thread。
- Runtime tests：marker 只通过 sanitized RuntimeRequest 进入 fake/local process；现有 no-secret/no-direct-egress assertions 继续通过。
- Browser integration：在 390/768 viewport 验证 registry disclosure、preview/result marker、final timeline provenance、denied action/error isolation、retry state、focus/accessibility 和 no horizontal overflow。
- 运行 schema generation/check、typecheck/build/unit、Rust fmt/Clippy/tests、requirements traceability、`node --check tests/thread-ui.mjs`、`git diff --check`。

## Out of scope

真实 market data、provider research fetch、portfolio facts、strategy/backtest/artifact producers、order draft、risk policy、arming/approval、Order Gateway、live reconciliation、additional model providers 和 full S33/S34 cross-page/packaging gates。

## Acceptance mapping

| #23 acceptance | Evidence target |
| --- | --- |
| Typed registry excludes authority | `CapabilityDecision.researchTools`, capability unit tests |
| Mode/context restrictions | `research.run` policy recheck and IPC tests |
| Sanitized unsupported/negative paths | public dispatch tests and `RESEARCH_RESULT_INVALID` |
| Source/context/marker reaches output | persisted `research_result` item plus runtime fake output |
| No secrets/direct egress | `RuntimeRequest` marker-only seam and process environment tests |
| Browser/provenance/accessibility | isolated `tests/thread-ui.mjs` run and evidence doc |
