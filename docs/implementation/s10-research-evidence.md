# S10 #34 source-gated typed research evidence

日期：2026-09-20

实现提交：`6eaa12149ca87fd50b8b1d1c5eb8c63faf264e47`、`49c57ae3b4e7016a01880c4c94a34523959ba395`、`4f68deb468c87061d0cbd1a1012283b17fa38845`、`6adae6c09e383fd3ce7a21a679d9b1df03745da1`、`aecd3b79f30b7d4fdd8310f09631579ff7fcf0f1`、`5e6f85bd07ee7be3f98e6edbde2b98bcdb40347`、`8caa6eb269429f6b3ee966f3be265c35bcf50381`、`b879243f57568e6acfa307cc1b6e202005e9ea52`

验证 SHA：`b879243f57568e6acfa307cc1b6e202005e9ea52`（`dev`）
范围：S10 第一条垂直切片；股票/现货 evidence cards 见 [`s10-evidence-cards-evidence.md`](s10-evidence-cards-evidence.md)。
状态：**IMPLEMENTED_UNVERIFIED**（#34 的 typed contract、source gate、隔离 bridge、fake runtime 和目标 UI 路径已通过；真实 provider facts、native Keychain/runtime 和 S33 回归仍未宣称完成）

## Delivered behavior

- `ResearchToolRequest`/`ResearchToolInvocation` 可携带可选 `focus`（GENERAL/EQUITY/CRYPTO_SPOT），focus 进入 canonical request hash；旧请求省略 focus 时仍按 GENERAL 兼容。
- `ResearchToolPayload` 从单一 reason 扩展为 typed state、focus、conclusion、bounded findings、bounded evidence/provenance、instrument refs 和 limitations。结果保留 source/provider/status、provider/received timestamp、freshness/quality；没有 provider received time 时明确为 `UNAVAILABLE`，保证 `turn.start` 重算结果稳定。
- source-gated 状态遵循 Backend ARD §41.9：任何非 `AVAILABLE` source（包括 `UNVERIFIED`）以及尚无 provider facts 的 `AVAILABLE` catalog source 都返回 sanitized `UNAVAILABLE`，fixture 也不能绕过 gate；证据 entry 保留 `BLOCKED_EXTERNAL`/`UNVERIFIED` 等来源状态。integration-only `TRADEX_RESEARCH_FIXTURE` 只在明确的 public-market + EQUITY/CRYPTO_SPOT request 与 synthetic source 同时匹配时产生显式 fixture-labelled `AVAILABLE` result，正常桌面不读取该 seam。
- `research.run` 复用现有 `market::catalog` 与 `portfolio::get` producer seams，输出 bounded canonical instrument refs 或 producer 状态摘要，不新增 provider、credential 或网络客户端。
- Composer 增加 authorized Research tool selector、focus selector 和结构化 preview；preview 在 selected tool、mode、execution、account、context refs 或 focus 变化时清除。preview 展示 conclusion、instrument refs、findings、source/provider/status/received time、limitations 和 marker。
- 研究 preview 独立于模型/运行时 ready gate，允许在 runtime/model 不可用时查看只读结果，并明确显示 `READY`、`RUNNING`、`DONE` 或 `UNAVAILABLE` 生命周期。
- `research_result` timeline item 持久化完整有界 typed result（旧摘要仍保留），reload 后继续展示 evidence card；`turn.start` 仍重建并 exact-compare invocation/result，真实 marker 进入 runtime，缺失/篡改 pair 继续返回 `RESEARCH_RESULT_INVALID`，失败前不写 Thread 状态。
- 研究 query 仍只用于受限输入和 hash；prompt injection、authority command、provider body、凭证和模型 secret 不进入 typed result，也不取得 capability/risk/keychain/gateway 权限。

## Verification

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `npm run schema:check` | PASS — Rust / JSON Schema / TypeScript agree |
| `npm run typecheck` | PASS |
| `npm run build` | PASS；保留既有大 chunk warning |
| `cargo check --workspace --features integration-test` | PASS |
| `cargo test --workspace research::tests --features integration-test -- --test-threads=1` | PASS — 10 research tests |
| `cargo test --workspace typed_research_result_is_sanitized_and_tamper_evident --features integration-test -- --test-threads=1` | PASS — marker, missing/tampered pair, no-mutation and sanitized evidence |
| `cargo test --workspace --features integration-test -- --test-threads=1` | PASS — 101 library tests and all workspace integration targets; only repository-marked native checks ignored |
| `cargo clippy --workspace --features integration-test --all-targets -- -D warnings` | PASS |
| `python3 scripts/check_requirements.py` | PASS — 201 requirements, 70 screens, 12 QA scenarios, 23 baseline files |
| `node --check tests/thread-ui.mjs`、`git diff --check` | PASS |
| Rust-backed `/__integration/command` source gate | PASS — GENERAL with fixture enabled remains `UNAVAILABLE` / `OD-001` / `BLOCKED_EXTERNAL`, while explicit EQUITY and CRYPTO_SPOT fixture requests are `AVAILABLE` only with `control-plane:market`; prompt-injection text is absent; missing and tampered result pairs return `RESEARCH_RESULT_INVALID` without changing thread state |
| Rust-backed fake runtime | PASS — valid `EQUITY` result reaches `research_result` timeline and completed runtime item with the exact marker; provider attempt `SUCCEEDED` |
| Fresh CUA browser path | PASS — runtime/model-unavailable preview stays enabled; lifecycle moves `READY` → `DONE`; unavailable preview, focus/mode invalidation, synthetic EQUITY and CRYPTO_SPOT cards, disabled Trade CTA, card/marker keyboard focus, renderer reload persistence, 390/768/1280 widths with no horizontal overflow, and zero warn/error logs |

The browser run used `npm run dev:browser` with a fresh temporary Rust/SQLite workspace and explicit integration fixtures. It did not read or write ChatGPT OAuth, DeepSeek keys, broker credentials, or external provider responses. The synthetic `AVAILABLE` rows therefore prove the typed contract and UI/runtime wiring only; they do not prove provider entitlement, real-time quotes, native Keychain behavior, or S33 coverage. The native desktop/provider boundary remains pending and is not upgraded by this fixture run.

## Remaining boundary

#34 does not claim real market/portfolio/news/filings/fundamentals data, venue comparison, trade proposal, model inference or execution authority. #35 consumes this stable typed result for the follow-on evidence-card slice; S33 and real external entitlements remain separate evidence boundaries.
