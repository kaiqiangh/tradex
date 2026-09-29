# S10 #35 股票与现货 evidence cards Spec

日期：2026-09-19  
前置：S10 #34 source-gated typed research result（`6adae6c`）  
状态：**SPEC**

## 目标

在同一个 `ResearchToolResult` 上提供股票与现货的可审阅证据卡。卡片只消费 Control Plane 已校验的 typed fields，不读取 provider 原始 JSON、凭证或网络；结果缺失时显示受限状态，不把缺失值渲染为零或成功。

## 契约

- `ResearchToolPayload` 增加有界 `scenarios`、`artifactRefs`、`spotVenues` 和可选 `fixtureLabel`；字段全部可选/有默认值，旧 S05 `state/reason` payload 仍可解码。
- `ResearchScenario` 只有 bounded `title`/`detail`；artifact ref 使用已附加 context 的 canonical artifact ID，不从 query 或 provider 文本推导。
- `ResearchSpotVenue` 包含 venue、typed state、selected、可选 bid/ask/spread/depth/quoteAge、完整 provenance 和 limitation。数值保持 decimal/string，缺失时保持 `null`；不会用 0 填充。
- 现货 venue 只允许 `BINANCE`、`BITGET`。普通桌面没有 crypto source 时生成两个带 `UNAVAILABLE`/`BLOCKED_EXTERNAL` 原因的安全受限 entry；集成 fixture 只能在 Rust integration feature 与显式环境变量下生成 synthetic entries，并带 `fixtureLabel`，不代表 provider truth。

## Producer 与安全边界

- 股票 producer 继续复用 canonical market catalog；只有 source gate 为 `AVAILABLE` 的集成 fixture 才生成 synthetic scenarios，真实 provider facts 仍由后续 market/data-source slice 提供。
- 现货 producer 不新增网络 client、provider credential 或 order path。Binance/Bitget positive fixture 使用独立 synthetic source gate；生产无该 gate 时只返回受限 provenance。
- artifact refs 只从 request 的 `artifact` context 复制；query、prompt injection、provider body、secret、risk/approval/order/gateway 状态不进入 card。
- `ResearchToolResult` 的 hash、marker、context refs、account scope 和 `turn.start` exact compare 保持不变；完整新增 payload 进入 timeline persistence 和 runtime marker 校验。

## UI 交互

- `ResearchResultCard` 按 focus 显示股票或现货分区。股票展示 conclusion、findings、scenarios、artifact refs、evidence/provenance 和 limitations。
- 现货展示仅有 provenance 的 Binance/Bitget rows、selected venue、spread/depth/quote age；blocked/unavailable/degraded rows 显示状态和 limitation，不显示伪造报价。
- Trade mode 才渲染 disabled/read-only 的 proposal entry 提示；Ask/Research 不渲染任何 Trade CTA，也不调用 order、approval、arm、gateway 或 live-risk。
- 状态、来源、限制、quote age、marker 使用语义元素并支持键盘阅读；卡片在 390/768/1280 viewport 不产生横向溢出。preview 生命周期继续在 mode、context、account、focus 和 refs 变化时清除。

## 验收

1. Rust/schema/TypeScript 生成物一致，旧 payload、unknown-field、bounded text/array 和 null-vs-zero 负例通过。
2. Rust integration 覆盖 equity scenarios/artifact refs、crypto blocked rows、synthetic fixture venue rows、workspace/account isolation、prompt-injection no-authority、no-mutation 和 tampered complete result。
3. Browser bridge 在隔离 workspace 验证股票/现货卡片、fixture label、Trade CTA gate、reload persistence、390/768/1280 overflow、keyboard semantics 和 zero console errors；真实 provider/entitlement 仍单独标记 pending。
4. 证据记录精确 SHA、测试计数和 fixture/真实 provider 边界；不提前关闭 S33 或宣称真实 Binance/Bitget facts。

## 不在本票

真实行情/深度订阅、新 provider、完整新闻/基本面、订单 proposal 生成或执行、审批/arm/live-risk、真实交易和 S33 全量回归。
