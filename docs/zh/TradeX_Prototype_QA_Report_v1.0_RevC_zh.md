# TradeX UI Prototype QA 报告 — v1.0 RevC

**基线：** TradeX PRD / UI Spec / prototype Revision C  
**Prototype：** `prototype/index.html`、`prototype/styles.css`、`prototype/app.js`  
**目的：** 源码一致性、traceability、交互 fixture 与安全状态审计。

## 1. 总体结果

**RevC 源码对齐 Gate：PASS。** Prototype 现在使用与 RevC PRD/UI Spec 相同的状态模型和安全语义。Standalone fixture 不会被描述为已经完成真实 broker/runtime 集成。

在把 prototype 视为视觉验收完成之前，仍需独立执行浏览器/设备 accessibility 与 visual-regression QA。

## 2. 已执行自动/静态检查

| 检查 | 结果 | 证据 |
|---|---|---|
| JavaScript syntax | PASS | `node --check app.js` 成功 |
| Inline action resolution | PASS | 64 个 inline handler reference；0 unresolved |
| PRD FR 中英文一致 | PASS | 中英文均包含 FR-001–FR-080，共 80 个唯一 ID |
| PRD AC 中英文一致 | PASS | 中英文包含相同 AC-001–AC-066 集合 |
| Cross-cutting ID 对齐 | PASS | NFR (19)、SEC (9)、DATA (8)、OPS (9)、UX (10)、OD (16) ID 集合一致 |
| Prototype revision title | PASS | HTML title = `TradeX — High-Fidelity Prototype v1.0 RevC` |
| Agent Mode vocabulary | PASS | Ask / Research / Backtest / Trade |
| Execution Context separation | PASS | 独立 context pill，由 selected account/environment 推导 |
| Account-scoped arming | PASS | 使用 `armedAccounts` Set，并有 Disable All |
| 完整 Live market provenance | PASS | source/provider timestamp/TradeX received/venue/entitlement/age/freshness |
| LLM provider model | PASS | CLIProxyAPI ChatGPT OAuth + DeepSeek；fallback opt-in 默认 OFF |
| Reservation lifecycle | PASS | 有 `RESERVED` 与 reservation conflict fixture |
| Ambiguous submission 安全 | PASS | 使用 Manual Resolution；没有 blind release action |
| Provider-schema form | PASS | connection modal 由 provider-specific field/permission schema 驱动 |
| Non-live variants | PASS | Local Paper、Alpaca Paper、T212 Demo、Binance Testnet、Bitget Demo |
| Storage terminology | PASS | SQLite + DuckDB + Filesystem；Parquet 为 Phase 2+ 可选 |
| Accessibility 源码基线 | PASS | `:focus-visible`、dialog ARIA、`aria-live`、reduced-motion CSS |
| Prototype-only Share link | PASS | RevC topbar 已删除；workspace portability 使用本地 export/import |

## 3. 关键交互/状态审计

| 场景 | RevC 预期 | 结果 |
|---|---|---|
| Ask turn | Read-only analysis；无执行 action | PASS |
| Research turn | Research result + artifacts + provenance；不提升 authority | PASS |
| Trade + Live account | Trade mode 本身不授予权限；必须 arm 精确账户 | PASS |
| 切换 Live account | 新账户不继承之前账户 arming | PASS |
| Disable All | 清空所有 armed account | PASS |
| Live limit approval | Immutable proposal + policy + 完整 market snapshot + reservation | PASS |
| Live market approval | 明确 maximum authorized spend | PASS |
| Approval expiry/stale quote | Approval 不再可用，必须 refresh/reapprove | PASS |
| Risk policy change | Approval invalidated；weakening 额外 disarm 对应账户 | PASS |
| Reservation conflict | 使用 Available/Reserved/Effective 做 deterministic reject | PASS |
| Ambiguous submission | Account disarmed/unhealthy；reservation frozen；必须 Manual Resolution | PASS |
| Broker rejection | 独立 rejection state；acknowledgement 不等于 fill | PASS |
| Non-live order | PAPER/DEMO/TESTNET label 贯穿 lifecycle | PASS |
| LLM quota/OAuth/unavailable | 默认需要显式操作；跨 provider auto fallback 默认关闭 | PASS |
| Workspace restore | 验证 archive 不含 secrets；恢复后 Live 全部 DISARMED | PASS |
| Clock skew | 阻断 Live authority，TimeService 恢复后仍需显式 re-arm | PASS |
| Stablecoin depeg | Quality degraded；对应 valuation/risk 路径明确 degraded/blocked | PASS |
| Backtest metrics | Sortino、Profit Factor、Turnover + reproducibility manifest | PASS |
| Narrow navigation | 5 个 primary item + More 到 secondary destinations | PASS（源码审计） |

## 4. 文档一致性审计

- PRD 与 UI Spec 使用相同的双轴状态模型。
- Coverage Matrix 直接基于 RevC FR/AC ID 生成，不复用 RevB 的 Complete 声明。
- 中英文 PRD 保留相同 requirement IDs 和决策语义。
- 中英文 UI Spec 描述相同 screen inventory、安全 invariant、storage/provider 决策。
- Prototype README 明确所有 broker/model 行为都是 fixture，并统一使用 Revision C 术语。

## 5. 尚未执行的 QA Gate——不是规格缺口

1. **Visual/browser regression：** 在正式浏览器 harness 中执行 desktop/narrow-width screenshots。
2. **Keyboard/screen-reader QA：** 用真实辅助技术验证 focus order、modal focus trap/restore、label、announcement。
3. **Runtime integration QA：** Codex App Server、CLIProxyAPI、provider adapter 实现后，验证 Coverage Matrix 中 runtime-pending 行。
4. **Persistence/restart QA：** 验证真实 process restart 与 sleep/resume 后 Thread/Turn/audit/reservation reconciliation。
5. **Real provider sandbox QA：** 使用官方 sandbox/demo/testnet API 验证 provider-specific capability/schema 差异。

## 6. Release 建议

该包可以作为 **RevC 产品/设计 prototype baseline** 和 implementation handoff。只有在 runtime-pending acceptance criteria 完成真实实现和 provider 环境测试后，才能将其描述为 production execution system。
