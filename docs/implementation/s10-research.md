# S10 可溯源研究工具与股票/现货结果

日期：2026-09-19  
前置：S05 capability/context/typed research、S06 data-source authorization、S07 canonical market、S09 portfolio/FX  
状态：**SPEC 已发布；#34 垂直切片已实现并留存证据**（真实 provider/native/S33 边界仍按 evidence 文档保留）

## Problem Statement

TradeX 已有受 capability 约束的 `research.run` 和 `turn.start` 结果复核，但 S05 只返回统一的 `UNAVAILABLE` 原因。用户无法在一个只读线程中把股票市场、账户组合和研究证据放在同一条可审阅链路上，也无法看到研究来源、provider/TradeX 时间、限制和结果是否真正进入最终 runtime 输出。研究内容必须始终是未受信数据，不能改变风险政策、账户授权、凭证或执行批准。

## Solution

在现有三项 typed research registry（`public_market_read`、`account_read`、`historical_simulation`）上扩展最小的结构化结果契约。研究请求继续携带 workspace、mode、execution context、account、context refs 和不可信 query；query 只用于受限检索和 request hash，永不回显为结果或权限。结果由现有 `market.get`、`portfolio.get`、data-source catalog/probe 的来源策略组合而成，未授权、外部阻塞、过期或缺字段时返回明确的 `UNAVAILABLE`/`BLOCKED_EXTERNAL`/`DEGRADED` 状态，不绕过 source gate，也不新增 provider 或直连网络客户端。

每个结果保持 `request_hash`、marker、context refs、source IDs、provider/received timestamps、freshness/quality、canonical instrument/account refs、结论、key findings、bounded scenarios、evidence 和 limitations。生产环境只暴露已授权的 typed fields；fixture 只能在测试/浏览器环境启用并标注为 fixture。多来源结果在一个 `research_result` timeline item 中呈现，`turn.start` 继续用请求重建和完整结果比较拒绝缺失/篡改；最终 runtime 至少收到真实结果 marker，不能只凭 UI 标签声称工具执行成功。

股票与现货结果卡复用同一结果契约：股票显示结论、关键发现、情景、证据与来源/新鲜度；现货在来源和权限可用时显示 Binance/Bitget 选定 venue、价差/深度和 quote age，否则显示受限原因。Trade CTA 只在 Trade mode 作为后续提案入口，S10 不新增下单、审批、arm 或 live risk 权限。

## User Stories

1. 在 Ask/Research 模式中，用户可以选择一个已授权 typed research tool，查看请求处于 Running、Queued、Done 或受限状态。
2. 用户可以在同一线程选择股票或现货 focus，并把 market、portfolio/account 和历史/研究上下文附加到同一次只读请求。
3. 用户看到的研究结果能区分 AVAILABLE、DEGRADED、UNAVAILABLE、BLOCKED_EXTERNAL 和 FAILED，并明确显示来源、provider 时间、TradeX 收到时间、新鲜度、质量和限制。
4. 用户看到 canonical instrument IDs、account refs 和 bounded numeric/text fields，而不是 provider 原始 JSON、无限 tick stream、原始 order book 或密钥。
5. 用户可以在研究结果中审阅股票结论、key findings、scenarios、evidence/provenance 和可用 artifact refs；任何不可用字段不会伪装成零值或成功数据。
6. 用户可以在现货结果中审阅已授权 venue 的 quote age、spread/depth 和来源；当 Binance/Bitget 数据被阻塞或未验证时，卡片显示原因而不是推测比较。
7. 用户可以在 390px 和 768px 宽度下通过键盘阅读状态、来源、限制和 marker；结果卡没有横向溢出，预览在 mode、context、account 或 context refs 变化时清除。
8. 用户可以验证研究结果真正进入最终 runtime：正常结果包含 marker，缺失或篡改 invocation/result 被 `RESEARCH_RESULT_INVALID` 拒绝。
9. 包含 prompt injection、`order.submit`、风险或凭证指令的外部研究文本仍只作为不可信 evidence，不能改变 capability、risk policy、account readiness、keychain 或 gateway。
10. workspace 隔离、account 选择和 source policy 在请求与结果两侧都保持一致；不存在跨 workspace 的 market、portfolio 或 research 泄漏。

## Implementation Decisions

1. **共享 IPC 契约**：在 `src-tauri/src/protocol.rs` 扩展现有 research 类型，保持 `deny_unknown_fields`、bounded lengths/array sizes、camelCase wire names 和 `ResearchToolId` 三项 ID 兼容。新增最小的 focus/section、result status、provenance/evidence/finding/scenario 类型；不要用 `serde_json::Value` 替代 typed fields。
2. **来源与生产边界**：`research.run` 通过现有 capability decision 和 source catalog 调用 typed producer。market/portfolio 只复用各自已有 read-only dispatcher 与 canonical/provenance 规则；filings/news/fundamentals 只在现有 OD-003/OD-004 source policy 允许时暴露受限 metadata，否则返回清洁的 unavailable/block reason。不得在 research 模块中新增 HTTP client 或 credential lookup。
3. **状态与安全**：结果 state 明确表达 source gate、freshness 和缺字段；任何不可信文本不得被解析为 tool ID、命令、风险等级、凭证或批准。研究结果不调用 gateway、不写 account/model/risk/thread state、不改变 outbox 或 keychain。
4. **请求完整性**：focus、tool、mode、execution context、account、attached contexts 和 query 都进入 canonical request hash；`turn.start` 重建同一请求并 exact-compare 完整结果。marker 由已验证结果计算，runtime 消费 marker 和结构化安全摘要，而非未清洗 query/provider payload。
5. **前端交互**：复用 Composer 的 research preview 生命周期和 `research_result` timeline item registry。增加 focus selector、Running/Queued 状态、股票/现货 evidence cards、来源/限制 panel 和明确的 Trade-mode-only CTA；Ask/Research 中不渲染可执行批准按钮。
6. **fixture 与真实边界**：浏览器/集成测试可用现有 fixture seams 产生固定的股票/现货/组合结果，并在结果中可见 fixture 标记；默认桌面运行不读取 fixture，不声称 provider truth。真实 provider entitlement、实时行情、交易级 FX、完整 filings/news/fundamentals 和 S33 端到端回归仍是后续证据边界。
7. **兼容性**：旧的 `UNAVAILABLE` 结果仍能按 schema 解码；S10 只增加可选/有默认值的结构化字段，除非请求显式选择新的 focus。未知字段、超长输入、非法 context/source、跨 workspace account 和伪造 provenance 均 fail closed。

## Testing Decisions

- Rust unit tests：状态映射、source/tool compatibility、canonical hash、query 不回显、canonical identity、bounded arrays/text、fixture/non-fixture 隔离、workspace/account isolation、prompt-injection no-authority、no-mutation。
- Rust integration tests：`research.run` 和 `turn.start` 的完整 result pair；正常 marker 进入 fake runtime；缺失/篡改 invocation/result 产生 `RESEARCH_RESULT_INVALID`；market/portfolio/source blocked 与 unavailable 输出稳定。
- Schema/type checks：Rust、JSON Schema、TypeScript 生成结果一致；旧 S05 unavailable fixture 仍通过。
- Browser checks：Composer focus/preview、timeline research result、股票/现货卡片和 source limitation；390/768/1280 viewport 无 overflow；键盘焦点、语义状态、console 无 warn/error。
- Evidence：每项声明记录精确 commit、命令、测试计数和外部边界；fixture、标签、截图或 requirement ID 单独不能替代 runtime 证据。

## Out of Scope

- 新 provider、真实 broker/exchange order submission、approval/arm/live-risk、keychain 或 gateway contract。
- A-share、完整新闻/公告/基本面内容抓取、未授权的 filings/news redistribution、无限历史/逐笔行情和原始 order book。
- 研究文本驱动的自动交易、策略 sandbox 权限、模型直连外部 endpoint。
- S33 全量 requirement/QA 回归及真实外部 entitlement 的最终验证。

## Further Notes

- Backend ARD §41–42 是共享 wire contract 的权威；Frontend ARD §7.8、§11、§12、§19.4 和 UI Spec B5/B7/B8/§14 约束预览、timeline、来源和安全交互。
- S10 完成后仍需保持 `FR-006`、`AC-007`、`AC-028`、`SEC-005` 的真实证据链；仅把状态从 `NOT_STARTED` 改为 `IMPLEMENTED` 不构成验收。
- 推荐 ticket 顺序：先完成一个可运行的 source-gated typed result vertical slice，再补股票/现货 evidence cards 与多来源限制路径；第二项依赖第一项的稳定 IPC 和 runtime marker 证据。
