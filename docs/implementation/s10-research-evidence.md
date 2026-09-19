# S10 #34 source-gated typed research evidence

日期：2026-09-19  
实现提交：`6eaa121`（`feat: add structured typed research results`）  
范围：S10 第一条垂直切片；股票/现货 evidence cards 仍由 #35 负责。  
状态：**IMPLEMENTED_UNVERIFIED**（契约、Rust 负例和隔离 bridge 已通过；真实 provider facts、完整 UI 研究卡和 S33 回归仍未宣称完成）

## Delivered behavior

- `ResearchToolRequest`/`ResearchToolInvocation` 可携带可选 `focus`（GENERAL/EQUITY/CRYPTO_SPOT），focus 进入 canonical request hash；旧请求省略 focus 时仍按 GENERAL 兼容。
- `ResearchToolPayload` 从单一 reason 扩展为 typed state、focus、conclusion、bounded findings、bounded evidence/provenance 和 limitations。结果保留 source/provider/status、provider/received timestamp、freshness/quality；没有 provider received time 时明确为 `UNAVAILABLE`，保证 `turn.start` 重算结果稳定。
- source-gated 状态遵循 Backend ARD §41.9：OD-001/002 等非 AVAILABLE source 返回 sanitized `UNAVAILABLE`；证据 entry 保留 `BLOCKED_EXTERNAL`/`UNVERIFIED` 等来源状态。integration-only `TRADEX_RESEARCH_FIXTURE` 可产生显式 fixture-labelled `AVAILABLE` result，正常桌面不读取该 seam。
- Composer 增加 Research focus selector 和结构化 preview；preview 在 mode、execution、account、context refs 或 focus 变化时清除。preview 展示 conclusion、findings、source/provider/status/received time、limitations 和 marker。
- `research_result` timeline item 继续只保存清洗摘要、限制和 marker；`turn.start` 仍重建并 exact-compare invocation/result，真实 marker 进入 runtime，缺失/篡改 pair 继续返回 `RESEARCH_RESULT_INVALID`，失败前不写 Thread 状态。
- 研究 query 仍只用于受限输入和 hash；prompt injection、authority command、provider body、凭证和模型 secret 不进入 typed result，也不取得 capability/risk/keychain/gateway 权限。

## Verification

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `npm run schema:check` | PASS — Rust / JSON Schema / TypeScript agree |
| `npm run typecheck` | PASS |
| `npm run build` | PASS；保留既有大 chunk warning |
| `cargo check --workspace --features integration-test` | PASS |
| `cargo test --workspace research::tests --features integration-test` | PASS — 3 targeted tests |
| `cargo test --workspace typed_research_result_is_sanitized_and_tamper_evident --features integration-test` | PASS — marker, missing/tampered pair, no-mutation and sanitized evidence |
| Rust-backed `/__integration/command` source gate | PASS — focus `EQUITY`，state `UNAVAILABLE`，source `OD-001`，evidence status `BLOCKED_EXTERNAL`，received `UNAVAILABLE`，injection text absent |
| `git diff --check` | PASS |

The native desktop window check could not be rerun in this pass because macOS became locked again. This is an environment boundary, not runtime success evidence. Full UI preview/timeline browser interaction and authenticated provider facts remain pending for #35/S33; this document does not upgrade them from `RUNTIME_PENDING`.

## Remaining boundary

#34 does not claim real market/portfolio/news/filings/fundamentals data, venue comparison, trade proposal, model inference or execution authority. #35 must consume this stable typed result to render the stock/spot evidence cards and rerun 390/768/1280, keyboard, console and security negative checks.
