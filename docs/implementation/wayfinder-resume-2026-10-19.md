# Wayfinder 需求清单与串行恢复 — 2026-10-19

> Later update, 2026-10-19: the user approved the MARKET scope and its bounded local implementation is now verified. See the [implementation evidence](s29-market-rule-composition-evidence.md). The inventory, tracker readback and tables below preserve the earlier snapshot; #135 and the full map remain open.
> 后续更新：MARKET 范围已获批准，并完成有界本地实现验证，详见[配对证据](s29-market-rule-composition-evidence_zh.md)。下方清单、tracker 读回及表格保留早先快照；#135 与完整地图仍开放。

本记录恢复已有 [完整构建 TradeX v1.0 RevC](https://github.com/kaiqiangh/tradex/issues/1) 地图。固定本地基线为 `dev@825fa12c6f44192589b4e214f3592c511c547667`；这是计划和状态快照，不是新的应用验收。本轮沿用已有规范、拆票和公开验证边界。

English scope: this is a dated requirements and workflow snapshot. It inventories the existing documentation and preserves its recorded acceptance states. The current owning Spot financial-plan ticket remains open; the proposed MARKET-only rule composition is pending a separate scope decision. No runtime contract or financial authority changes are made by this record.

## Skill 来源及执行方式

已从用户指定的 `mattpocock/skills` 仓库读取并固定提交 `5beb35b040e3de97e54fa9e386d7e42d15c295c2`。实际路径位于 `skills/engineering/`：

- [wayfinder](https://github.com/mattpocock/skills/blob/5beb35b040e3de97e54fa9e386d7e42d15c295c2/skills/engineering/wayfinder/SKILL.md)：恢复已有地图、读取当前前沿、保留决策出处。
- [to-spec](https://github.com/mattpocock/skills/blob/5beb35b040e3de97e54fa9e386d7e42d15c295c2/skills/engineering/to-spec/SKILL.md)：将本轮发现的规范冲突收敛为可审阅方案。现有最高公开测试边界已在 owning ticket 确认，沿用此约定。
- [to-tickets](https://github.com/mattpocock/skills/blob/5beb35b040e3de97e54fa9e386d7e42d15c295c2/skills/engineering/to-tickets/SKILL.md)：沿用已批准的单一完整 owning ticket；本轮未发布重复票据或改动其十五项 AC。
- [implement](https://github.com/mattpocock/skills/blob/5beb35b040e3de97e54fa9e386d7e42d15c295c2/skills/engineering/implement/SKILL.md)：规范决策完成后，逐个公开行为做 RED→GREEN，再做检查与提交。
- [code-review](https://github.com/mattpocock/skills/blob/5beb35b040e3de97e54fa9e386d7e42d15c295c2/skills/engineering/code-review/SKILL.md)：按固定起点依次做独立 Standards 和 Spec 审查。用户要求的串行顺序覆盖 skill 的默认并行。
- [research](https://github.com/mattpocock/skills/blob/5beb35b040e3de97e54fa9e386d7e42d15c295c2/skills/engineering/research/SKILL.md)：本轮仅运行一个研究代理，期间主代理等待；依据固定版本的 Binance 第一方文档整理[双语研究](s29-instrument-rules-composition-research.md)。

## 全目录索引与阅读证据边界

基线包含 2,002 个受 Git 跟踪的 docs 文件，共 47,298,211 字节。全部文件都已逐一读取字节并计算 SHA-256；文本文件另做 UTF-8 扫描。结果见 [逐文件索引](evidence/wayfinder-resume-20261019/document-inventory.csv)，其 SHA-256 为 `6bd6ffb10e7156b304b2b6f5336e52cdbcb22bb43c443122c97d15e1d2e09c5d`。

| 文档职责 | 文件数 | 本轮处理方式 |
| --- | ---: | --- |
| 根目录基线及索引 | 8 | 权威顺序及当前交接核对 |
| agents 工作约定 | 3 | 读取 tracker、领域与标签约定 |
| 实现方案与追踪 | 142 | 继承需求/阶段交叉引用，核对当前 owning 规范、进度与提案 |
| 历史证据 | 1,789 | 全量字节索引，按当前项读取相关证据；历史结论不改写为新鲜验收 |
| 中文配对及索引 | 55 | 对当前金融契约和交接内容核对配对语义 |
| 点击式原型 | 5 | 继承其 fixture 身份和历史阅读记录；原型行为不覆盖规范 |

目录索引不是 2,002 份文件逐份语义复审或 224 张图像逐张视觉验收的声明。[原始 sources.csv](sources.csv) 仍保留最初 23 份 `READ` 记录及原 SHA；其中 15 份文档此后已有修改，原记录不代表其新内容已重新完整审阅。本轮具体语义核对集中在当前地图、需求/阶段清单、已完成的 Binance 本地用途能力、owning plan、Backend §41.44–41.48、待决提案、对应生产者/消费者及固定第一方规则文档。完整运行时、原生和真实提供方验收仍按各项证据执行。

权威顺序为英文 PRD → UI Spec（尤其 §14）→ Frontend/Backend ARD（Backend §41–42 为 IPC 权威）→ Coverage → QA → prototype。当前目录的最新英文规范仍是产品语义依据，原型和历史 PASS 不能更改它。

## 完整需求清单

下表逐项转录基线 [requirements.csv](requirements.csv) 的全部 203 行；验证方法和证据仍以该 CSV 为完整记录。状态是追踪表原有状态，本轮没有升级。每行的 Sxx 对应后文方案表中的交付路径。

| 状态 | 项数 |
| --- | ---: |
| VERIFIED | 28 |
| IMPLEMENTED_UNVERIFIED | 74 |
| IN_PROGRESS | 55 |
| NOT_STARTED | 43 |
| DEFERRED | 3 |

| ID | 需求 | 优先级/目标 | 实现路径 | 原状态 |
| --- | --- | --- | --- | --- |
| FR-001 | Run a pinned Codex App Server locally | P0 | S04 | IMPLEMENTED_UNVERIFIED |
| FR-002 | Support persistent Thread / Turn / Item UX | P0 | S04 | IMPLEMENTED_UNVERIFIED |
| FR-003 | Stream agent and tool activity in real time | P0 | S04 | IMPLEMENTED_UNVERIFIED |
| FR-004 | Resume prior threads | P0 | S04 | IMPLEMENTED_UNVERIFIED |
| FR-005 | Render structured financial timeline cards | P0 | S04 | NOT_STARTED |
| FR-006 | Implement TradeX typed domain tools | P0 | S05,S10 | IMPLEMENTED_UNVERIFIED |
| FR-007 | Separate trusted control plane from agent zone | P0 | S01,S05,S24 | IN_PROGRESS |
| FR-008 | Implement OS keychain credential storage | P0 | S02 | VERIFIED |
| FR-009 | Implement canonical instrument model | P0 | S07 | IMPLEMENTED_UNVERIFIED |
| FR-010 | Implement provider capability discovery | P0 | S02 | VERIFIED |
| FR-011 | Implement market-data service | P0 | S07 | IMPLEMENTED_UNVERIFIED |
| FR-012 | Implement market-data freshness metadata | P0 | S07 | IMPLEMENTED_UNVERIFIED |
| FR-013 | Implement portfolio aggregation | P0 | S09 | IMPLEMENTED_UNVERIFIED |
| FR-014 | Implement account-currency and FX normalization | P0 | S09 | IMPLEMENTED_UNVERIFIED |
| FR-015 | Implement Alpaca Paper adapter | P0 | S17 | IMPLEMENTED_UNVERIFIED |
| FR-016 | Implement Trading 212 Demo/Live adapter | P0 | S18,S28 | IN_PROGRESS |
| FR-017 | Implement Binance Testnet/Live Spot adapter | P0 | S19,S29 | IN_PROGRESS |
| FR-018 | Implement Bitget Demo/Live Spot adapter | P0 | S20,S30 | IMPLEMENTED_UNVERIFIED |
| FR-019 | Implement OrderProposal model | P0 | S13 | IN_PROGRESS |
| FR-020 | Implement deterministic Risk Engine | P0 | S21 | IN_PROGRESS |
| FR-021 | Implement pre-approval validation | P0 | S22 | NOT_STARTED |
| FR-022 | Implement pre-execution validation | P0 | S23,S24 | VERIFIED |
| FR-023 | Implement account-scoped atomic execution reservations | P0 | S23,S25 | NOT_STARTED |
| FR-024 | Implement single-use live approval | P0 | S22 | NOT_STARTED |
| FR-025 | Implement privileged Order Gateway | P0 | S24 | VERIFIED |
| FR-026 | Implement idempotency and reconciliation | P0 | S25,S27 | IN_PROGRESS |
| FR-027 | Implement account-scoped live arming/disarming + global disable-all | P0 | S22 | IN_PROGRESS |
| FR-028 | Implement crash/startup reconciliation | P0 | S27 | IN_PROGRESS |
| FR-029 | Implement local execution audit trail | P0 | S12,S25 | NOT_STARTED |
| FR-030 | Implement provider rate limiting | P0 | S27 | IMPLEMENTED_UNVERIFIED |
| FR-031 | Implement local backtesting | P0 | S15 | IMPLEMENTED_UNVERIFIED |
| FR-032 | Implement strategy sandbox | P0 | S14 | NOT_STARTED |
| FR-033 | Implement paper/demo/testnet trading workflows | P0 | S16,S17,S18,S19,S20 | IMPLEMENTED_UNVERIFIED |
| FR-034 | Implement deterministic error taxonomy and UI mapping | P1 | S01,S32 | VERIFIED |
| FR-035 | Implement natural-language screener | P1 | S11 | IMPLEMENTED_UNVERIFIED |
| FR-036 | Implement artifacts | P1 | S12 | IMPLEMENTED_UNVERIFIED |
| FR-037 | Implement strategy versioning | P1 | S14 | NOT_STARTED |
| FR-038 | Implement market calendar and corporate actions | P1 | S08 | IMPLEMENTED_UNVERIFIED |
| FR-039 | Implement configurable data retention | P1 | S31 | IMPLEMENTED_UNVERIFIED |
| FR-040 | Implement workspace export/import | P1 | S31 | IMPLEMENTED_UNVERIFIED |
| FR-041 | Add futures support | P2 | DEFERRED | DEFERRED |
| FR-042 | Add A-share research/data integration | P2 | DEFERRED | DEFERRED |
| FR-043 | Add additional brokers/exchanges | P2 | DEFERRED | DEFERRED |
| FR-044 | Implement complete five-step onboarding flow | P0 | S01,S02,S03,S21 | IMPLEMENTED_UNVERIFIED |
| FR-045 | Implement interactive thread history and resume navigation | P0 | S04 | IMPLEMENTED_UNVERIFIED |
| FR-046 | Implement context/account/model pickers | P0 | S05 | IMPLEMENTED_UNVERIFIED |
| FR-047 | Implement provider connection test + permission/capability review | P0 | S02 | VERIFIED |
| FR-048 | Implement provider-specific account-detail variants | P0 | S02 | VERIFIED |
| FR-049 | Implement live order cancellation approval lifecycle | P0 | S26 | NOT_STARTED |
| FR-050 | Implement explicit Risk Rejected / Broker Rejected / Approval Expired UI states | P0 | S22,S25 | NOT_STARTED |
| FR-051 | Implement startup/auth/stream-disconnected recovery states | P0 | S27 | IN_PROGRESS |
| FR-052 | Implement market-order maximum-authorized-notional approval UI | P0 | S22 | NOT_STARTED |
| FR-053 | Implement complete natural-language screener builder flow | P1 | S11 | IMPLEMENTED_UNVERIFIED |
| FR-054 | Implement backtest running/failed/compare states | P1 | S15 | IMPLEMENTED_UNVERIFIED |
| FR-055 | Implement artifact provenance surface | P1 | S12 | IMPLEMENTED_UNVERIFIED |
| FR-056 | Implement Ask mode with no execution capability | P0 | S05 | NOT_STARTED |
| FR-057 | Implement complete live-approval market snapshot provenance | P0 | S22 | NOT_STARTED |
| FR-058 | Implement risk-policy-change approval invalidation lifecycle | P0 | S21,S23,S27 | IN_PROGRESS |
| FR-059 | Implement reservation-conflict UI/reasoning surface | P0 | S23 | IMPLEMENTED_UNVERIFIED |
| FR-060 | Implement complete Demo/Testnet order lifecycle variants for Trading 212, Binance, and Bitget | P0 | S18,S19,S20 | IN_PROGRESS |
| FR-061 | Implement complete normalized order-state UI mapping including `RESERVED` | P0 | S22,S25 | IN_PROGRESS |
| FR-062 | Implement market-session / halt / corporate-action product surfaces | P1 | S08 | IMPLEMENTED_UNVERIFIED |
| FR-063 | Implement workspace import/restore workflow | P1 | S31 | IMPLEMENTED_UNVERIFIED |
| FR-064 | Implement complete canonical error-remediation surfaces | P1 | S01,S32 | VERIFIED |
| FR-065 | Implement portfolio FX provenance display | P1 | S09 | IMPLEMENTED_UNVERIFIED |
| FR-066 | Implement full backtest metric set including Sortino, profit factor, and turnover | P1 | S15 | IMPLEMENTED_UNVERIFIED |
| FR-067 | Implement provider-schema-driven credential/configuration forms | P1 | S02 | VERIFIED |
| FR-068 | Implement LLM provider connection workflow (CLIProxyAPI OAuth / DeepSeek key), schema-driven per §26.3 | P0 | S03 | VERIFIED |
| FR-069 | Implement CLIProxyAPI sidecar lifecycle: launch/supervise/health-check (`/v1/models` probe)/backoff restart/port-conflict handling | P0 | S03 | VERIFIED |
| FR-070 | Implement subscription quota display and exhaustion handling (provider-reported windows where available; in-flight attempt behavior; explicit switch/retry) | P0 | S03 | IN_PROGRESS |
| FR-071 | Implement LLM error categories and recovery states (`MODEL_UNAVAILABLE` / `QUOTA_EXCEEDED` / `OAUTH_EXPIRED`) with remediation UI | P0 | S03 | IN_PROGRESS |
| FR-072 | Record per-turn model/provider provenance and write model switches/degradations to the audit trail | P0 | S04 | IMPLEMENTED_UNVERIFIED |
| FR-073 | Implement model routing/fallback policy: cross-provider automatic fallback OFF by default, explicit opt-in, full audit/disclosure, and no capability change | P0 | S03 | IMPLEMENTED_UNVERIFIED |
| FR-074 | Separate Agent Mode from Execution Context and enforce the compatibility matrix | P0 | S05 | IMPLEMENTED_UNVERIFIED |
| FR-075 | Persist immutable per-turn mode/account/execution/model/provider/context snapshots | P0 | S04 | IMPLEMENTED_UNVERIFIED |
| FR-076 | Implement evidence-based Manual Resolution for `UNKNOWN_RECONCILING`; no blind reservation release | P0 | S25 | IN_PROGRESS |
| FR-077 | Implement TimeService clock-skew/freshness/TTL safeguards for live authority decisions | P0 | S08 | IMPLEMENTED_UNVERIFIED |
| FR-078 | Block/review dangerous provider permissions (withdrawal/transfer/custody/margin/leverage) before live readiness | P0 | S02 | VERIFIED |
| FR-079 | Implement FX/stablecoin valuation provenance and depeg/quality handling | P1 | S09 | IMPLEMENTED_UNVERIFIED |
| FR-080 | Implement editable OrderDraft → immutable OrderProposal regeneration semantics | P0 | S13 | IN_PROGRESS |
| AC-001 | User can create a TradeX thread and see streamed agent items before the complete turn finishes. | AC | S04 | IMPLEMENTED_UNVERIFIED |
| AC-002 | User can close and reopen TradeX and resume a persisted thread. | AC | S04 | IMPLEMENTED_UNVERIFIED |
| AC-003 | Tool calls display name, state, duration, and structured summary. | AC | S04 | NOT_STARTED |
| AC-004 | Demo and live accounts are represented as distinct connections. | AC | S02 | VERIFIED |
| AC-005 | TradeX validates provider capabilities when an account connects. | AC | S02 | VERIFIED |
| AC-006 | Live execution is disabled if account reconciliation is incomplete. | AC | S22,S27 | IMPLEMENTED_UNVERIFIED |
| AC-007 | Agent can combine market data, portfolio state, and research in one thread. | AC | S10 | NOT_STARTED |
| AC-008 | Every market snapshot shown in a live approval has source, provider timestamp, TradeX received timestamp, venue, realtime/delayed entitlement, and freshness state. | AC | S07,S22 | NOT_STARTED |
| AC-009 | User can submit an Alpaca Paper order and see resulting broker state. | AC | S17 | IMPLEMENTED_UNVERIFIED |
| AC-010 | User can execute supported Demo/Testnet order workflows on Trading 212, Binance, and Bitget where available, with explicit environment labels through acknowledgement/fill state. | AC | S18,S19,S20 | IN_PROGRESS |
| AC-011 | A live order cannot be submitted without a valid TradeX financial approval. | AC | S22,S23,S24 | IN_PROGRESS |
| AC-012 | A generic Codex approval cannot authorize live financial execution. | AC | S22,S23,S24 | IN_PROGRESS |
| AC-013 | Changing instrument, side, quantity, account, order type, price, or time-in-force invalidates prior approval. | AC | S13,S22 | NOT_STARTED |
| AC-014 | A relevant risk-policy change invalidates affected pending approvals; weakening policy additionally disarms the affected live account. | AC | S21,S23,S27 | IN_PROGRESS |
| AC-015 | A stale market snapshot prevents execution until refreshed. | AC | S08,S22,S24 | IN_PROGRESS |
| AC-016 | Risk state is revalidated immediately before submission. | AC | S08,S22,S24 | IN_PROGRESS |
| AC-017 | Two concurrent threads cannot reserve the same cash or exposure. | AC | S23 | VERIFIED |
| AC-018 | Agent cannot modify risk policy. | AC | S05,S21 | NOT_STARTED |
| AC-019 | Agent cannot access the privileged Order Gateway directly. | AC | S05,S24 | NOT_STARTED |
| AC-020 | Model never receives broker secrets. | AC | S02,S05,S24,S32 | IN_PROGRESS |
| AC-021 | Broker acknowledgement is never displayed as a fill unless broker state confirms fill. | AC | S25 | NOT_STARTED |
| AC-022 | Ambiguous submission moves to `UNKNOWN_RECONCILING`. | AC | S25 | IMPLEMENTED_UNVERIFIED |
| AC-023 | TradeX never blindly retries a non-idempotent provider order POST after ambiguous timeout. | AC | S25 | NOT_STARTED |
| AC-024 | Lost private-stream connectivity triggers degraded state and reconciliation. | AC | S27 | IN_PROGRESS |
| AC-025 | Restarting the app reconciles open live orders before enabling new live submissions. | AC | S27 | IN_PROGRESS |
| AC-026 | Every live order has a durable chain from proposal → risk evaluation → approval → reservation → execution attempt → provider state. | AC | S12,S25 | NOT_STARTED |
| AC-027 | Strategy code cannot access broker credentials. | AC | S14 | NOT_STARTED |
| AC-028 | External research content cannot authorize a trade. | AC | S05,S10 | IMPLEMENTED_UNVERIFIED |
| AC-029 | Broker secrets never appear in logs, artifacts, thread history, SQLite, DuckDB, or Parquet. | AC | S02,S05,S24,S32 | IN_PROGRESS |
| AC-030 | Selecting Trade mode or a Live execution context does not arm live execution. | AC | S05,S22 | IN_PROGRESS |
| AC-031 | Requesting a live order while DISARMED requires a separate explicit Arm action before transaction approval is shown. | AC | S22 | NOT_STARTED |
| AC-032 | The instrument, account, side, quantity, and order type displayed during order monitoring are the same immutable transaction approved by the user. | AC | S13,S22 | NOT_STARTED |
| AC-033 | An open live order can enter cancellation flow only after broker state refresh and transaction-specific user approval; UI represents `CANCEL_PENDING` and `CANCELLED`. | AC | S26 | NOT_STARTED |
| AC-034 | The UI represents deterministic `RISK_REJECTED`, broker `REJECTED`, approval `EXPIRED`, and `UNKNOWN_RECONCILING` states without implying successful execution. | AC | S22,S25 | IN_PROGRESS |
| AC-035 | Market-order approval displays expected spend, maximum authorized spend, bid/ask/spread, quote age, and fees/slippage estimate where available. | AC | S22,S29 | NOT_STARTED |
| AC-036 | Provider connection surfaces detected permissions and capability set before the connection is treated as ready. | AC | S02 | VERIFIED |
| AC-037 | Authentication failure and private-stream disconnection disable new live execution until account health is restored. | AC | S27 | IN_PROGRESS |
| AC-038 | The complete first-run workflow covers Workspace → Providers → LLM (Model) → Risk Defaults → Ready, where the Model step configures the LLM provider: CLIProxyAPI sidecar state (running/OAuth-authorized) and/or DeepSeek key validated by probe + test inference (§26.3). | AC | S01,S02,S03,S21 | IMPLEMENTED_UNVERIFIED |
| AC-039 | The user can inspect and resume recent Threads and manage attached context/account/model from the composer. | AC | S04,S05,S33 | IN_PROGRESS |
| AC-040 | The natural-language screener shows the parsed structured filter before execution and renders the reduced candidate set. | AC | S11 | IMPLEMENTED_UNVERIFIED |
| AC-041 | Ask mode provides lightweight analysis/read-only context and does not expose paper or live execution capability. | AC | S05 | IN_PROGRESS |
| AC-042 | Live arming is account-scoped: arming Trading 212 does not arm Binance or Bitget; global Disable All disarms all live accounts. | AC | S22 | IMPLEMENTED_UNVERIFIED |
| AC-043 | Every live approval exposes complete market snapshot provenance defined by AC-008. | AC | S07,S22 | NOT_STARTED |
| AC-044 | Saving a risk policy invalidates affected pending approvals and communicates the invalidation reason; weakening policy disarms the affected account. | AC | S21,S23,S27 | IN_PROGRESS |
| AC-045 | A second concurrent proposal evaluates against reduced account capacity after an earlier proposal reserves cash/exposure and is deterministically rejected when insufficient capacity remains. | AC | S23 | VERIFIED |
| AC-046 | Trading 212 Demo, Binance Testnet, and Bitget Demo/Testnet-compatible workflows preserve their explicit non-live environment identity through proposal, acknowledgement, fill/update, and account-position refresh. | AC | S18,S19,S20 | IN_PROGRESS |
| AC-047 | Workspace import/restore validates archive/schema metadata, does not import broker secrets, and requires reconciliation before live execution is re-enabled. | AC | S31 | IMPLEMENTED_UNVERIFIED |
| AC-048 | Backtest result includes return, Sharpe, Sortino, max drawdown, win rate, profit factor, turnover, equity curve, trade list, and reproducibility manifest. | AC | S15 | IMPLEMENTED_UNVERIFIED |
| AC-049 | Equity instrument detail exposes relevant market session/corporate-action state, and unsupported closed/halted execution is blocked deterministically. | AC | S08 | IMPLEMENTED_UNVERIFIED |
| AC-050 | Canonical error categories render appropriate remediation without introducing conflicting machine-state names. | AC | S32 | VERIFIED |
| AC-051 | Provider connection UI is rendered from provider credential/capability schema and does not assume every provider uses identical credential fields. | AC | S02 | VERIFIED |
| AC-052 | Core interactive surfaces are keyboard reachable with visible focus, and generic `Enter` submission cannot approve a live order or cancellation. | AC | S22,S26,S33 | IN_PROGRESS |
| AC-053 | Cross-account portfolio values show the workspace base currency plus FX source/timestamp/freshness for normalized values. | AC | S09 | IMPLEMENTED_UNVERIFIED |
| AC-054 | Narrow desktop/tablet-class layouts preserve live safety information; v1.0 exposes no native mobile live-execution client. | AC | S33 | IN_PROGRESS |
| AC-055 | Onboarding cannot reach Ready without at least one usable LLM provider; a sidecar that is stopped, port-conflicted, or unauthorized blocks Ready with remediation guidance. | AC | S03 | IN_PROGRESS |
| AC-056 | When the CLIProxyAPI sidecar is stopped, port-conflicted, or unauthorized, agent turns pause with remediation guidance while approvals, execution, and reconciliation continue to function (fail-closed for LLM only). | AC | S03,S27 | IMPLEMENTED_UNVERIFIED |
| AC-057 | Subscription quota exhaustion and OAuth expiry surface `QUOTA_EXCEEDED` / `OAUTH_EXPIRED` with retry/re-login/explicit provider-switch remediation. Cross-provider automatic fallback occurs only when the user explicitly enabled it, is visibly disclosed/audited, and never changes capability levels or bypasses approvals. | AC | S03 | IMPLEMENTED_UNVERIFIED |
| AC-058 | Every turn records the model/provider and all provider attempts that produced or failed that turn; model switches/degradations are written to the audit trail (§57). | AC | S04 | IMPLEMENTED_UNVERIFIED |
| AC-059 | Agent Mode and Execution Context are stored and rendered as separate dimensions; Ask/Research with a live account remains read-only, Backtest cannot invoke current-market broker execution, and Trade with a Live context still requires arming + approval. | AC | S05 | IN_PROGRESS |
| AC-060 | Every started turn persists an immutable snapshot of Agent Mode, execution context, selected account, capability level, model/provider, and attached context references/hashes. | AC | S04 | IMPLEMENTED_UNVERIFIED |
| AC-061 | An ambiguous live submission that exceeds the automatic reconciliation window keeps its reservation frozen and exposes only the evidence-based Manual Resolution options in §23; no generic release action restores live readiness. | AC | S25 | IMPLEMENTED_UNVERIFIED |
| AC-062 | Detected withdrawal/transfer/custody permission on a live broker credential blocks execution readiness until removed; permission scope that cannot be inspected is labeled `UNVERIFIED` and remains visible. | AC | S02 | VERIFIED |
| AC-063 | Quote age, approval expiry, and reconciliation timers fail closed for live authority decisions when TimeService reports unacceptable clock uncertainty. | AC | S08 | IMPLEMENTED_UNVERIFIED |
| AC-064 | Local Paper is explicitly identified as TradeX simulation and cannot be confused with provider-hosted Paper/Demo/Testnet or Live execution. | AC | S16 | VERIFIED |
| AC-065 | Editing a generated order creates a new immutable `OrderProposal` identity and invalidates any approval tied to the prior proposal. | AC | S13,S22 | IN_PROGRESS |
| AC-066 | Cross-account valuation involving stablecoins/FX shows source/path/timestamps/freshness and a quality/depeg state; unreliable conversion cannot silently drive live risk calculations. | AC | S09 | IMPLEMENTED_UNVERIFIED |
| NFR-001 | UI render after received runtime event | p95 < 100 ms (measured from IPC event arrival to UI commit; sampled over representative interactions) | S34 | NOT_STARTED |
| NFR-002 | Tool-card render after result received | p95 < 100 ms (same measurement boundary as NFR-001) | S34 | NOT_STARTED |
| NFR-003 | Crash restart to reconciliation start | < 5 s | S27,S34 | IMPLEMENTED_UNVERIFIED |
| NFR-004 | Secret leakage into logs | 0 (CI runs a secret scanner over representative log corpora; zero findings is the acceptance gate) | S02,S24,S32,S35 | IN_PROGRESS |
| NFR-005 | Unapproved live order submissions | 0 | S22,S24,S35 | IN_PROGRESS |
| NFR-006 | Duplicate live submissions caused by TradeX retry | 0 | S24,S25,S35 | IN_PROGRESS |
| NFR-007 | Silent assumption of ambiguous broker state | 0 | S25 | IN_PROGRESS |
| NFR-008 | Open live orders reconciled after restart | 100% | S27 | IN_PROGRESS |
| NFR-009 | Live approval market snapshots include source, provider timestamp, TradeX received timestamp, venue, entitlement, and freshness | 100% | S07,S22 | NOT_STARTED |
| NFR-010 | Live execution requires healthy account state | 100% | S22,S24,S27 | IN_PROGRESS |
| NFR-011 | Material order change invalidates approval | 100% | S13,S22 | NOT_STARTED |
| NFR-012 | Any relevant risk-policy change invalidates affected pending approvals; weakening policy disarms the affected live account | 100% | S21,S23,S27 | IN_PROGRESS |
| NFR-013 | External telemetry is disabled by default and requires explicit opt-in | 100% | S32 | VERIFIED |
| NFR-014 | Financial price/quantity/notional calculations use decimal-safe arithmetic | 100% | S09,S13,S21,S23 | IMPLEMENTED_UNVERIFIED |
| NFR-015 | Core desktop UI is keyboard reachable with visible focus; `Enter` never implicitly approves a live transaction | 100% | S33 | NOT_STARTED |
| NFR-016 | Narrow-window layouts preserve live safety information; v1.0 ships no native mobile live-execution client | 100% | S33 | NOT_STARTED |
| NFR-017 | Turn/provider provenance is complete for every agent turn | 100% | S04,S12 | NOT_STARTED |
| NFR-018 | Live freshness/TTL checks use synchronized time semantics and fail closed on unacceptable clock uncertainty | 100% | S08,S22,S24 | IMPLEMENTED_UNVERIFIED |
| NFR-019 | Ambiguous submissions cannot be locally released into a healthy/live-ready state without authoritative/evidence-based resolution | 100% | S25 | NOT_STARTED |
| SEC-001 | Model cannot access raw broker secrets or OS-keychain values. |  | S02,S05,S24 | IN_PROGRESS |
| SEC-002 | Agent/Codex runtime cannot directly call the privileged Order Gateway. |  | S05,S24 | VERIFIED |
| SEC-003 | Generic Codex approvals cannot authorize financial execution. |  | S05,S22 | NOT_STARTED |
| SEC-004 | Strategy sandbox cannot access broker credentials, keychain, privileged gateway, or unrestricted network. |  | S14 | NOT_STARTED |
| SEC-005 | Untrusted external content cannot alter risk policy, authority, credentials, or approve execution. |  | S05,S10,S21 | IMPLEMENTED_UNVERIFIED |
| SEC-006 | Live approvals are account/action/proposal-bound, short-lived, and single-use. |  | S22,S23 | NOT_STARTED |
| SEC-007 | All model inference traffic has exactly one exit: the local CLIProxyAPI endpoint (127.0.0.1:8317). TradeX, Codex, and the strategy sandbox must not connect to any external LLM endpoint directly. |  | S03,S04,S14 | IN_PROGRESS |
| SEC-008 | The CLIProxyAPI sidecar holds only model credentials (ChatGPT OAuth, rendered DeepSeek key); it must never hold or receive broker credentials, keychain broker items, or Order Gateway capabilities. |  | S02,S03,S24 | IN_PROGRESS |
| SEC-009 | A Live broker connection with withdrawal/transfer/custody permissions detected must not become execution-ready until those permissions are removed; TradeX must not use unsupported margin/leverage permissions. |  | S02,S22 | IN_PROGRESS |
| DATA-001 | Domain logic uses canonical instrument IDs; provider symbols remain adapter mappings. |  | S07 | IMPLEMENTED_UNVERIFIED |
| DATA-002 | Price, quantity, and notional calculations use decimal-safe representations. |  | S09,S13,S21,S23 | IMPLEMENTED_UNVERIFIED |
| DATA-003 | Market snapshots retain source, provider timestamp, TradeX received timestamp, venue, entitlement, and freshness. |  | S07 | IMPLEMENTED_UNVERIFIED |
| DATA-004 | Cross-account valuation records workspace-base currency, FX source, timestamp, and freshness. |  | S09 | IMPLEMENTED_UNVERIFIED |
| DATA-005 | Market-data provider metadata includes entitlement, retention, and redistribution constraints. |  | S07 | IMPLEMENTED_UNVERIFIED |
| DATA-006 | Backtests persist reproducibility manifests including strategy/data hashes and adjustment/calendar assumptions. |  | S15 | IMPLEMENTED_UNVERIFIED |
| DATA-007 | Every Turn persists an immutable Agent Mode, Execution Context, account, model/provider, and attached-context provenance snapshot. |  | S04,S05 | IMPLEMENTED_UNVERIFIED |
| DATA-008 | FX/stablecoin normalized valuation persists source/path/timestamps/freshness and quality/depeg state. |  | S09 | IMPLEMENTED_UNVERIFIED |
| OPS-001 | Broker/exchange state is authoritative for live positions, orders, and fills. |  | S25 | NOT_STARTED |
| OPS-002 | Ambiguous non-idempotent submissions are reconciled before any retry. |  | S25 | IN_PROGRESS |
| OPS-003 | Startup/resume/reconnect performs reconciliation before new live execution. |  | S27 | IN_PROGRESS |
| OPS-004 | Provider rate limiting prioritizes execution reconciliation above research/history traffic. |  | S27 | IMPLEMENTED_UNVERIFIED |
| OPS-005 | Execution reservations are atomic and account-scoped across concurrent threads. |  | S23 | VERIFIED |
| OPS-006 | Desktop builds are packaged and code-signed through a repeatable release pipeline. |  | S34 | NOT_STARTED |
| OPS-007 | Auto-update and crash reporting are explicit product surfaces; crash reports contain no broker secrets and telemetry follows NFR-013 opt-in. |  | S32 | VERIFIED |
| OPS-008 | An `UNKNOWN_RECONCILING` reservation remains frozen after automated timeout and may only be resolved through audited Manual Resolution. |  | S25 | IN_PROGRESS |
| OPS-009 | TimeService detects material wall-clock jumps/skew and blocks Live freshness/TTL authority decisions until trusted time is restored. |  | S08 | IMPLEMENTED_UNVERIFIED |
| UX-001 | Selecting Trade mode or a Live execution context never arms execution. |  | S05,S22 | IN_PROGRESS |
| UX-002 | Live arming is explicit and account-scoped; arming one broker account does not arm another. |  | S22,S27 | IMPLEMENTED_UNVERIFIED |
| UX-003 | Live approval displays immutable order identity and market-data provenance. |  | S13,S22 | NOT_STARTED |
| UX-004 | Paper / Demo / Testnet / Live states use explicit text in addition to color. |  | S16,S17,S18,S19,S20,S28,S29,S30 | IMPLEMENTED_UNVERIFIED |
| UX-005 | Keyboard interaction cannot implicitly approve live order or cancellation. |  | S22,S26,S33 | NOT_STARTED |
| UX-006 | Primary navigation is New Thread / Threads / Markets / Watchlists / Accounts / Strategies / Artifacts / Settings. |  | S01,S33 | VERIFIED |
| UX-007 | Responsive behavior is for narrow desktop/tablet-class windows; native mobile execution is out of v1.0 scope. |  | S01,S33 | IN_PROGRESS |
| UX-008 | Agent Mode and Execution Context are visibly distinct; illegal combinations are disabled or explained. |  | S05 | IMPLEMENTED_UNVERIFIED |
| UX-009 | Cross-provider model fallback is visibly disclosed before/when it occurs and is not enabled by default. |  | S03,S04 | IN_PROGRESS |
| UX-010 | Manual Resolution never presents an unsafe generic “release reservation and continue” action. |  | S25 | NOT_STARTED |
| FR-081 | Implement confirmed permanent local removal of eligible Trading 212 Demo account and order-book observations | P0 | S18 | VERIFIED |
| AC-067 | Only a credential-free `trading212` / `DEMO` connection in `FAILED` or `DISCONNECTED` state can be permanently deleted after explicit confirmation naming the selected record by its full connection ID, label, provider, and environment. The confirmation says that provider keys, provider orders and every other connection are unchanged. The backend rechecks the active workspace, identity, state version, credential health, open orders, actionable proposals and unresolved attempts, then atomically removes the account projection plus its account and Demo order-book observations. `SUBMITTING` and `UNKNOWN_RECONCILING` attempts remain unresolved; an `ACKNOWLEDGED` attempt is resolved for deletion only after its exact linked order has a durable recognized terminal observation with `pending: false`. Terminal proposal/attempt audit history remains retained. Cancellation makes no change; success refreshes the list/context and announces completion; focus returns to the trigger or a logical fallback; failure leaves all target rows intact. | AC | S18 | VERIFIED |

## 实现方案与依赖

下表继承 [原实施方案](README.md#3-有序工作项) 的 35 项路径。技术依赖与串行调度分开；已批准的 S29 优先及物理验证暂缓只影响调度，不能关闭它们的验收。当前实施项以地图的最新决策及本节后的具体前沿为准。

| 顺序 | 工作项标题 | 真实前置 | 完整交付与退出证据 |
|---|---|---|---|
| S01 | 打开并恢复本地桌面工作区 | 无 | 可运行 Tauri 窗口、Rust workspace.open、SQLite 迁移/事务、版本化 command/result/outbox/snapshot/replay；关闭后重开同一工作区。导航可达，错误显式，无模型时如实显示不可用。真实 IPC + 临时磁盘重启验证。 |
| S02 | 安全连接并查看提供方账户 | S01 | Schema 驱动原生敏感字段输入、Keychain 引用、账户/环境隔离、connect/probe/permissions/disconnect、危险权限 gate、账户详情/手动刷新；逐提供方以官方契约确认 schema，凭据不落普通存储。 |
| S03 | 配置模型网关并完成五步入门 | S01、S02 | 固定 CLIProxyAPI、8317 冲突/认证/探测/重启、OAuth/DeepSeek 测试推理、用户默认模型与 fallback、配额恢复；真实可用路由才允许 Ready；风险配置不隐式启用 Live。 |
| S04 | 持久化并流式恢复 Codex Thread | S01、S03 | 固定 Codex App Server 协议，Thread/Turn/Item、取消/中断/重试、工具状态与 provider attempts、历史恢复；不可变开始快照与真实流式事件早于 turn 完成。 |
| S05 | 按模式与精确上下文限制 Agent 工具 | S02、S04 | Composer pickers、canonical context refs/hash、完整 capability matrix、Ask/Research 只读、Backtest 仿真、Trade 非 Live/Live 分流；受限 typed research MCP 与金融命令隔离；历史不读当前默认值。 |
| S06 | 核实并选定所需外部数据与授权 | S02、S05 | 查阅官方资料、保留来源/日期/能力/许可和真实探测，逐项解决 OD-001–006；建立缺数行为。需要付费/账户授权的选择交用户，未解决项不冒充完成或实盘就绪。 |
| S07 | 浏览真实行情并管理 Watchlists | S01、S06 | canonical instruments、Market Explorer/详情、搜索、watchlist CRUD、Census/Warm/Hot/Cold、默认 Hot≤20、DuckDB 1m+历史、source/双时间戳/venue/entitlement/freshness。输入身份贯穿详情/附加上下文。 |
| S08 | 校验可信时间、市场时段与公司行为 | S06、S07 | wall+monotonic+provider offset；sleep/clock jump 失信；equity 日历/半日/停牌/公司行为和历史调整。UI 展示阻塞原因；Live 权限统一消费可信结果。 |
| S09 | 汇总账户组合并保留 FX 估值证据 | S02、S06、S07、S08 | 原币/账户/工作区基础货币、positions/open orders/fills、FX source/path/quality/depeg；不把 USDT 当 USD，不可信转换禁止依赖该值的 Live risk。 |
| S10 | 用可溯源研究工具分析股票与现货 | S05–S09 | 真实 market/portfolio/news/filings/fundamentals 工具、来源/时间/限制和结构化 Research 结果，Prompt injection 无金融权限；工具结果实际影响最终输出，缺失/篡改负例可观测。 |
| S11 | 审阅并运行自然语言筛选条件 | S07、S10 | NL→可编辑 FilterSpec/RankSpec→审阅版本→DuckDB 降低候选集→按需获取→仅附加勾选标的；修改使解析/结果失效，完整空/失败/重试状态。 |
| S12 | 保存、检索和导出研究产物溯源 | S04、S10 | Artifact library/detail/export、Turn/Item/provider/tool/source/dataset/order refs；历史模型和上下文独立于当前选择器，导出脱敏且可验证内容来源。 |
| S13 | 编辑订单草稿并生成不可变 Proposal | S02、S07–S09 | 全量 order 字段、tagged quantity/TIF/environment、规范 decimal、规则校验、draft version→新 proposal ID/hash；所有 material edits/refresh 生成新身份，后续审批绑定该对象。 |
| S14 | 保存策略版本并运行受限策略进程 | S01、S07 | 策略列表/编辑/保存版本/hash、独立 worker、受限文件/网络/Keychain/Gateway、只输出 signal；非法访问通过真实进程测试被拒绝。 |
| S15 | 配置并比较可复现历史回测 | S08、S14 | Composer 与 Editor 汇入同一配置；冻结策略/数据/hash/date/cost/slippage/seed；queued/running/cancelled/failed/completed，完整指标/曲线/trades/manifest，两次完成结果比较；无 lookahead，模型故障不停止确定性运行。 |
| S16 | 完成本地模拟交易与持仓更新 | S09、S13 | Local Paper 专属模拟器、标记 TradeX simulation、订单/fill/现金/持仓持久化与查询，永不穿过 Live Gateway；不把模拟结果称为 broker truth。 |
| S17 | 完成 Alpaca Paper 交易生命周期 | S02、S13、S16 | 官方 Paper endpoints/account/capability/symbol/order/cancel/query/private stream 适配，真实 sandbox 订单状态；不实现未授权的 Alpaca Live 范围。 |
| S18 | 完成 Trading 212 Demo 交易生命周期 | S02、S13、S16 | Provider-specific 认证、数量/TIF/取消/查询/限流/订单身份与权限能力，Demo 环境不可变；以真实官方支持能力验证，ACK 与 fill 分离。 |
| S19 | 使用 Binance Spot 普通账户接口 | S02、S13、S16 | 保留账户身份、权限、Spot rules、签名/time offset、BASE/QUOTE、order/client identity 与观察边界；专用 Testnet 交易生命周期票已关闭，不声称 Live 下单验收通过，完整 Live gate 留给 S29。 |
| S20 | 使用 Bitget Spot 普通账户接口 | S02、S13、S16 | 保留 API key/secret/passphrase、官方能力与 REST-only 边界；不创建/使用 Demo 账户，不声称 Demo 行为已验证，完整 Live 执行 gate 留给 S30。 |
| S21 | 配置并执行确定性风险政策 | S08、S09、S13 | 全部 PRD §21 风险字段和硬规则；无用户政策 Live disabled、市价单默认 off；policy version/多账户范围与失效原因；Agent 无修改权限。 |
| S22 | 显式 Arm 并批准精确金融意图 | S21 | 账户独立 arming/20 分钟超时；独立 PLACE/CANCEL approval 类型、单次/nonce/TTL/hash/版本；全量行情与最大授权支出展示，审批前与消费时校验；泛化 Codex approval 永不转成金融权限。 |
| S23 | 原子消费审批并预留账户容量 | S22 | #88 父规范与 #89–#93 子项均已完成：PLACE/CANCEL preparation、容量冲突、派发前失效/释放、并发隔离与事务回滚。S27/S33 属于独立后续范围，跨阶段需求状态保持未完成。 |
| S24 | 经认证的独立 Gateway 派发执行 | S23 | 固定子进程、继承双向通道/协议/会话认证、狭窄 ID 请求、权威重载、一次性 grant、持久化 SUBMITTING 后 I/O；disarm 与 dispatch 串行、失败不重放变更；进程隔离与边界故障测试。 |
| S25 | 对账未知提交并核验人工处置证据 | S24、S17–S20 | canonical 全状态、broker truth 优先、unknown freeze/query-first/5 分钟后仍冻；backend evidence scope/window/pagination/identity/允许决策；状态版本防并发成交覆盖，处置后健康复核且仍 disarmed。 |
| S26 | 刷新并审批撤单与处理成交竞态 | S22、S24、S25 | 精确不可变 CANCEL intent、Arm 后回到撤单且刷新、不同撤单资格、approval/reject/expire、CANCEL_PENDING→provider truth；ACK 不释放，部分成交/费用/剩余容量恰好一次处理；修改仅确认撤单后新 proposal。 |
| S27 | 恢复崩溃、休眠、断流并中断未派发工作 | S24–S26 | 全 Live restart/sleep/lock/Disable All disarm，account fault 指定作用域、共享政策全部绑定账户；未知保留；P0/P1 优先于 research/backtest，bounded queues；模型故障下监控/撤单/对账仍有效。 |
| S28 | 验证 Trading 212 Live 可信执行 | S06、S08、S18、S21–S27 | Live adapter 与 trusted chain 贯通、最新 entitlement/calendar/账户能力门槛、真实查询与明确用户交易授权的验收；没有实际授权不发 Live order，保留该验收阻塞。 |
| S29 | 验证 Binance Spot Live 可信执行（父规范 #121；#122–#127 已关闭，完整金融验收仍开放） | S06、S08、S19、S21–S27 | Live spot 单独连接、permissions/rules/time/FX、受信下单/撤单/对账全链，account-specific readiness；contract/fault 与授权外部验收分别记录。 |
| S30 | 验证 Bitget Spot Live 可信执行 | S06、S08、S20、S21–S27 | Live/Demo 绝不混路由，签名/身份/权限/成交/撤单证据与 trusted authority；只有该 provider 完整 gate 通过才开放对应能力。 |
| S31 | 验证工作区备份、导入、迁移与保留 | S12、S15、S25、S27 | 非秘密 export manifest、路径/归档/schema 校验、恢复前备份、事务迁移/integrity、Keychain 引用检查、恢复全 Live disarmed/reconcile；未解决金融证据不可自动清理。 |
| S32 | 完成设置、健康诊断与隐私控制 | S03、S27、S31 | Providers/Models/Risk/Data/Health/Appearance/About、版本来源/配额/日志/诊断、auto-update/crash-report 明确界面与 opt-in、默认无 telemetry；敏感字段序列化前脱敏。 |
| S33 | 完成全页面、键盘与窄屏回归 | S01–S32 | 所有页面与 QA-01–12 在实际应用重跑；native dialog 安全焦点/trap/inert/return、Enter 无隐式金融审批、semantic controls、screen reader、reduced motion；桌面/768/390 长内容保留导航与金融操作。基础可访问性从每项实现时即要求，本项负责完整交叉回归。 |
| S34 | 测量资源目标并构建签名桌面包 | S27、S31–S33 | IPC arrival→UI commit p95<100ms、重启→对账启动<5s、负载隔离/增长/Hot 上限；固定 runtime/schema/provider/engine 版本，可重复 macOS packaging/sign/notarization（按平台要求），缺少签名身份不能声称签名通过。 |
| S35 | 在同一 dev SHA 完成总验收并提交 PR | S01–S34 | 清单逐条绑定实现与行为证据；所有 provider gates、故障/安全/视觉/持久化/CI 达标，map/spec/task 层级无遗漏；dev→main PR 按仓库模板提交，保持 OPEN。 |

架构执行约束：React/TypeScript 展示权威快照与事件投影；Rust Control Plane 推导能力、风险、审批和预留；SQLite 事务与 outbox 同步保存领域变化；DuckDB 承担历史分析；模型经固定 CLIProxyAPI 接入，金融写入只经认证独立 Gateway。密钥留在原生安全输入/OS Keychain。不可变 Turn、Proposal 和 captured review 保留自身身份与来源。缺少行情、用途、费用、FX、权限或账户事实时，界面显示准确阻断原因并由 Rust 拒绝相关行为。

## 当前前沿与冲突

通过 `gh issue view` 和 `gh api` 查询均收到 `Forbidden`；随后 GitHub 连接器的只读查询成功。本轮以实际读回状态核对。Standards 审查指出原约定仅列 CLI；现已在 [tracker 约定](../agents/issue-tracker.md)补充环境阻止 CLI 时的已授权只读回退，`gh` 仍是读取首选及 tracker 写入工作流：

1. [Binance 来源绑定本地用途确认与 MARKET_DATA_USE 检查](https://github.com/kaiqiangh/tradex/issues/150) 已 CLOSED，八项 AC 勾选完成；[批准的来源绑定用途决议](https://github.com/kaiqiangh/tradex/issues/149) 同样 CLOSED。
2. 当前串行项为 [完整 Spot 金融计划、逐资产预留与所需估值](https://github.com/kaiqiangh/tradex/issues/135)，已认领、OPEN，十五项 AC 中四项勾选。其原生前置 [精确标的佣金来源](https://github.com/kaiqiangh/tradex/issues/133) 与 [费用/必需 FX 陈述](https://github.com/kaiqiangh/tradex/issues/131) 已 CLOSED；已完成的前置不等于当前项通过。
3. [owning 规范](s29-owning-financial-plan-spec.md) 要求正向 producer→RiskDecision→Prepare；Backend §41.44 保留动态规则不可用，§41.47 保留受约束 LIMIT 的 PRICE_RANGE 不可用。现有 LIMIT 场景不能满足正向 AC6/AC10；不得通过删除实际规则或伪造账户事实获得 PASS。
4. [MARKET 范围提案](s29-instrument-rules-prepare-reachability-proposal_zh.md) 仍待用户决策。其开头明确要求决议记录前不修改配对契约或运行时 gate。此前对 MARKET_DATA_USE 用途确认的批准不包含此项 InstrumentRules 资格变更。
5. 第一方研究确认：MARKET 仍受交易所执行价格范围约束；本地“无明示价格”的用途不适用只限 placement 检查。MAX_POSITION 的部分成交数量语义未获文档明确，仍不可推断。总订单数及 ORDERS 的新增贡献为一；algo/iceberg/list 不应机械加一。

[完整 financial-plan 进度](s29-owning-financial-plan-progress.md) 中已有这些未完成边界。根目录和地图 Notes 中仍指向已关闭前置的交接文字需要更新，但不改变任何 AC。

## 严格有序的后续步骤

1. 当前 `to-spec`：取得 MARKET 范围的具体决策；按固定研究明确公式、适用性、缺失/矛盾输入及保留的本地用途限制。决议前只完善非规范提案与证据。
2. 决议后同步中英文 PRD/UI/ARD 与 owning 规范，记录当前票据的决定。沿用既有单一 owning ticket，不将完整目标拆成可永久拒绝的来源容器。
3. `implement`：在已确认的实际 React→typed Rust Control Plane→隔离 SQLite/outbox 接缝，一次完成一个真实行为的 RED→GREEN。只替代外部 HTTP/vault/WS/clock 生产者；不植入金融状态或手动 VERIFIED。
4. 当前项完成正向、拒绝、时效/绑定、并发、预留/结算和当前/captured/pre-arm UI 证据后，执行相应 schema、类型、行为、完整仓库、普通构建/pin 检查。
5. 固定起点至当前提交，依序独立 Standards→Spec 审查，修复并重验；精确 SHA、证据和 tracker 交接完成后，才允许处理下一项。
6. 所有地图功能与未豁免验收完成，并在同一 `dev` SHA 完整验证后，提交 `dev`→`main` PR；由用户 review/merge。

仍开放的 S17 真实 Paper、S27 物理生命周期、S28/S29 金融、S31 父验收、S33 全页面/原生辅助功能以及 S34/S35 不能由本轮文档核对关闭。最近的 B2 代码/浏览器证据保留其局部范围；其独立审查及普通桌面构建限制继续记录在 [B2 收据](evidence/s33/b2-thread-archive-result.json)。

本轮为文档/计划核对，未发出真实提供方请求、金融操作或运行时行为变更。

固定起点至 `7f4de79` 的独立 Standards → Spec 串行复审均通过，零剩余发现。文档摘要、需求/阶段导出、研究来源及 tracker 勾选完整性、结构追踪和 diff 检查通过；该结论仅针对文档，不代表提案批准或应用验收。详见[复审与检查记录](evidence/wayfinder-resume-20261019/review.md)。
