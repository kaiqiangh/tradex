# S29.3 source use/persistence boundary audit

Audit baseline: dev@45f0594, plus the external fault-fixture/test worktree. SOURCE_ONLY code-path findings complement the earlier public review/history/reopen runtime proof; they do not grant a licence or establish all provider/native/financial acceptance.

| Boundary | Current authoritative code/evidence | Finding |
|---|---|---|
| Producer | `src-tauri/src/binance_market/stream.rs` publish stores only `ControlPlane.binance_market_observations`; lease/book/buffer are runtime values | No Store/outbox/history/export call in public source publication; bounded buffer/book and projected depth remain ephemeral |
| Reset/reopen | `src-tauri/src/lib.rs` observations HashMap initialized empty and cleared on workspace open/reset; source selection persists separately | Configuration can restore, current raw quote cannot; runtime proof in producer/consumer checkpoints independently verifies this |
| Risk persistence | `src-tauri/src/storage.rs:14308` append_risk_decision_tx saves typed RiskDecision; references/digests/times/checks only | Earlier public approval/risk-history/reopen test verifies no raw snapshot/bids/asks/provenance are restored |
| Proposal | `src-tauri/src/lib.rs:9042` builds OrderProposalReferences; storage proposal schema contains snapshot ID/status/reference reason | Reference/intent persistence is not a raw quote/book cache |
| Approval review/digest | `src-tauri/src/lib.rs:2100` hashes transient serialized MarketDetail; shared FinancialApproval type stores intent/decision/hash/times/status | Full review book is returned ephemerally; digest calculation itself does not persist the raw input. Actual armed approval path remains separately pending |
| Research producer | `src-tauri/src/lib.rs:5328` PublicMarketRead calls canonical catalog and returns at most8 instrument refs/count summary | It does not call market detail or pass live bid/ask/depth into model research/turn/artifact input |
| Artifact creation/export | `src-tauri/src/lib.rs:5239` saves completed turn item text/research result and typed provenance; storage validates these and exports that artifact | There is no source-book-to-artifact/export path in this producer. Arbitrary author/model text is not proof of permitted source redistribution; numeric-text DLP or user-specific rights are not established by this audit |
| Historical cache | `src-tauri/src/market.rs:64` test-only insert_history rejects REALTIME and non-equity source mapping; production exposes no raw Spot tick/book/history insertion operation | Public source does not enable a durable crypto realtime cache or historical authority |
| UI/capture | Markets/Trade source panels render backend projections; capture panel receives immutable review/time and has no query | UI proof verifies captured text remains unchanged when current source/book identity changes; no rights/Arm/approval is created |

No automatic raw-book persistence/export path was found in the audited producer and its consumers. This is a scoped source call-path audit with concrete paths, not a universal guarantee against arbitrary user text, external copies or future features. Source catalog and MARKET_DATA_USE continue UNVERIFIED/UNAVAILABLE for user-specific financial use, retention, redistribution/commercial and regional rights. Official URLs/review date are metadata, not legal acceptance. Full acceptance must retain current runtime checks/reviews and the independent real-provider/native/financial gates.

中文：按45f0594 与当前外部故障测试差异审计，生产者只存运行时 HashMap/租约/盘口/缓冲，没有 Store/outbox/历史/导出调用；重开清空当前原始观察，仅恢复配置。风险保存类型化引用/摘要/时间/结果，Proposal 保存 snapshot ID/状态/原因；此前公开运行时证明不恢复原始盘口。审批 digest 仅临时序列化哈希，完整评审盘口临时返回，实际已武装审批仍独立待验。PublicMarketRead 只返回最多8个目录标的引用/数量摘要，不把原始报价送入研究/turn/artifact。Artifact 保存已完成文本/结果/出处并导出该内容，没有本生产者的盘口导出路径；任意文本或数字 DLP/用户再分发许可不由本审计建立。历史写入仅测试入口并拒绝 REALTIME/crypto 映射，生产没有原始 Spot 历史写入操作。捕获面板无自身查询，公开 UI 证明材料更新后捕获全文不变。

中文：这些是有具体路径的来源/消费者边界审计，不是对任意用户文本、外部复制或未来功能的绝对保证。用户金融用途/保留/再分发/商业/地区权利始终未验证，MARKET_DATA_USE 不可用；官方 URL/审阅日期不是法律接受。完整检查/复审与普通原生/提供方/金融门禁仍须保留。
