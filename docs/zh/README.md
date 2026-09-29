# docs/zh — TradeX v1.0 RevC 中文文档

本目录是 Revision C 的简体中文同步版，2026-09-05 补充契约并校正验收证据，保留原业务范围和需求 ID。英文 PRD 为产品/安全语义权威；UI Spec 定义用户行为，前后端 ARD 定义实现架构。Backend ARD §41–42 是共享传输契约的唯一文档来源。

## 阅读顺序与中英文配对

| 顺序 | 中文版 | 英文版 |
|---|---|---|
| 1 | [PRD](./TradeX_PRD_v1.0_RevC_zh.md) | [PRD](../TradeX_PRD_v1.0_RevC.md) |
| 2 | [UI / Prototype Spec](./TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md) | [UI / Prototype Spec](../TradeX_UI_Prototype_Spec_v1.0_RevC.md) |
| 3 | [Frontend ARD](./TradeX_Frontend_ARD_v1.0_RevC_zh.md) | [Frontend ARD](../TradeX_Frontend_ARD_v1.0_RevC.md) |
| 3 | [Backend ARD](./TradeX_Backend_ARD_v1.0_RevC_zh.md) | [Backend ARD](../TradeX_Backend_ARD_v1.0_RevC.md) |
| 4 | [Coverage Matrix](./TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md) | [Coverage Matrix](../TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md) |
| 5 | [QA Report](./TradeX_Prototype_QA_Report_v1.0_RevC_zh.md) | [QA Report](../TradeX_Prototype_QA_Report_v1.0_RevC.md) |
| 6 | [Prototype 使用说明](../prototype/README_zh.md) | [Prototype guide](../prototype/README.md) |

[英文目录](../README.md) 定义完整权威顺序；[文件清单](../FILE_MANIFEST.md) 提供仓库相对路径、行数、字节哈希和语言配对。覆盖状态/原型表现不能覆盖规范要求，冲突应同步修正。

## 当前交付状态

本轮文档补齐：审批过期与预留释放、独立 Gateway 与派发边界、规范 IPC、撤单意图保留、证据型人工处置、不可变历史/proposal，以及具体交互要求；新增 S18 #66 本地 Demo 账户删除契约。

原型交互/交付仍为 **NOT PASS**：此次文档修订未修改 HTML/CSS/JS。QA Report 记录实际观察和 QA-01–QA-13 回归场景；QA-13 指出点击式原型没有符合条件账户的永久删除路径，S18 #66 运行时实现证据单独处理。UI Spec §14 定义修复目标。不得将目标写清楚当成原型已修复，也不得因此宣称其他运行时/券商集成通过。既有开放产品决策继续按 PRD 管理。

2026-09-25 账户接口范围更新：Binance Spot Testnet issues #67–#72 与 Bitget Demo issues #73/#74/#76/#77/#78 均已关闭；Bitget Live 只读 #75 属于独立范围。专用 Testnet/Demo provider 验收范围已撤销，但这不表示 provider-hosted Testnet 或 Demo 行为已验证。既有本地 fixture 工作仍只是本地证据。未来 Binance/Bitget Live 执行仍受 S21–S30 gate 约束，包括 S29/S30 provider gate。

S19 #68 在 Backend ARD §41.25、Frontend ARD §13.14 和 UI Spec §14.12 增加 Binance Spot Testnet 提交/恢复契约。Fixture 测试覆盖本地 adapter 与持久化状态转换，但不等于 provider-hosted Testnet runtime 验收。点击式原型保持未修改。

S19 #70 在 Backend ARD §41.27、Frontend ARD §13.15 和 UI Spec §14.13 增加 Binance Spot Testnet 签名私有流与 REST 恢复契约。实现已在 `dev` 使用 fixture 验证；未使用真实 Testnet 凭据、未调用 provider、未下单。父 Spec #67 和 provider acceptance #72 已按普通账户接口范围关闭；fixture 证据不等于 provider-hosted Testnet 验收，点击式原型保持未修改。

S19 #71 在 Backend ARD §41.28、Frontend ARD §13.15.1 和 UI Spec §14.14 增加 Binance Spot Testnet 准确订单撤销。Rust-backed fixture 覆盖持久化意图、单次准确 DELETE、结果不明时不重放、provider 订单变化和竞态成交。[本地实现证据](../implementation/s19-binance-testnet-cancel-evidence_zh.md)。未使用真实 Testnet 凭据、未调用 provider、未下单；父 Spec #67 和 provider acceptance #72 已按普通账户接口范围关闭。点击式原型未修改。

S25.1 #97 已实现并在本地验证 Trading 212 Live `PLACE` 进入 `UNKNOWN_RECONCILING` 后的只读对账证据。五分钟窗口超时后容量继续冻结；不确定证据不会证明订单不存在，也不会触发重发。窗口超时后只开放后端授权的 Keep Reconciling 操作；该操作记录决策，但不会重启 provider 查询。[本地验收证据](../implementation/s25-trading212-live-reconciliation-evidence_zh.md)。未发出真实 provider 请求，点击式原型未修改。

S26.2 #104 已在本地验证 Trading 212 Live 耐久撤单历史与竞态全额成交。准确撤单仅通过隔离 Gateway 派发一次；后续准确订单刷新保存 `FILLED` 观测，同时保留 `CANCEL_PENDING` attempt、将订单移出开放订单投影，且不再次写入 provider。[本地验收证据](../implementation/s26-2-trading212-live-cancel-evidence_zh.md)。仅使用合成凭据和本地 fake provider，点击式原型保持未修改。

S25.2 #98 为 Binance Spot Live 增加相同的只读证据路径。它会核验已保存的 SPOT 账户身份，只在普通 Live 端点按已保存的准确 `providerClientOrderId` 查询；未匹配或不完整结果保持不确定，容量继续冻结。[本地实现证据](../implementation/s25-binance-live-reconciliation-evidence_zh.md)。未发出真实 provider 请求或写入，点击式原型未修改。

S25.3 #99 将共享只读证据路径扩展到 Bitget Classic Spot Live。它核验已保存远端账户及准确持久化的 `clientOid`；账户/意图不匹配、空或格式错误的响应、认证失败和传输失败均保持不确定并冻结容量。[本地验收证据](../implementation/s25-bitget-live-reconciliation-evidence_zh.md)。验证仅使用合成凭据和本地 fixture；未发出真实 provider 请求或写入，点击式原型未修改。

S27.1 #108 为已连接的 Trading 212、Binance Spot 和 Bitget Spot Live 账户实现启动/workspace 重开恢复：将账户置为 stale 和 disarmed，刷新本地时间基线，先派发准确 PLACE evidence 读取，再与账户读取并发执行，并在提交账户投影前提交准确 evidence；含未知 PLACE/CANCEL attempt 的账户继续阻止恢复为 current，有界近期历史与开放订单分开展示。[本地证据](../implementation/s27-1-live-startup-recovery-evidence_zh.md)。`npm run check` 通过，Standards review 通过。Spec review 保留 `<5 s` 原生启动验收：临时 ad-hoc 签名应用在 macOS Keychain 读取处阻塞，provider HTTP 尚未开始；#108 保持打开，点击式原型未修改。

S25.4 #100 已在收窄范围内于已审查的 `dev@f67463d` 关闭：新鲜且准确的 Binance/Bitget Live 候选可以确认为已提交；不确定证据仍只能 Keep，reservation 保持 active。[本地验收证据](../implementation/s25-4-live-manual-resolution-evidence_zh.md)记录了串行 Standards/Spec 复审与全量检查通过，以及[结案评论](https://github.com/kaiqiangh/tradex/issues/100#issuecomment-5863195499)。Rust-backed UI 流程使用合成 fixture；未调用真实 provider 或写入，点击式原型未修改。S25.5 研究 #101 已关闭：Trading 212、Binance Spot 与 Bitget Classic Spot 的官方文档均未规定有时限的订单可见性保证，不能以准确查询为空证明订单未被接收；未决 attempt 因此继续只允许 Keep 并冻结 reservation。Binance 直接返回且耐久绑定到单次 PLACE 的 `-2010 NEW_ORDER_REJECTED` 属于独立的提供方拒单证据，不使 `-2013`、超时或查询无结果变为确定结论。[研究记录](../implementation/s25-live-absence-proof-research.md)与[#101 结案评论](https://github.com/kaiqiangh/tradex/issues/101#issuecomment-5863600151)。S25 父项 [#96 已在最终验收后关闭](https://github.com/kaiqiangh/tradex/issues/96#issuecomment-5863761675)；未发出真实 provider 请求或订单写入。

## 术语和同步规则

- Agent Mode 保留 Ask / Research / Backtest / Trade；Execution Context 区分只读/历史模拟与具体 Local Paper / Paper / Demo / Testnet / Live 环境。
- DISARMED / ARMED 按 account ID 管理；模型路由为 CLIProxyAPI → ChatGPT 或 DeepSeek，跨提供方自动回退必须 opt-in。
- OrderDraft 可编辑；OrderProposal 不可变；撤单绑定独立不可变 CANCEL 意图。
- UNKNOWN_RECONCILING 保持容量冻结，Manual Resolution 必须基于证据。“预留”对应 reservation，不使用暗示日程预约的译法。
- SQLite 保存事务/领域状态，DuckDB 保存 MVP 1m+ 分析数据，文件系统保存产物；Parquet 为 Phase 2+ 可选。
- 保留命令名、model/provider ID、错误/状态枚举及 FR/AC/NFR/SEC/DATA/OPS/UX/OD 标识。
- 中英文必须具有相同 FR/AC、A–K 页面及 QA case ID 集合；Coverage 和 QA 的状态逐项一致。
- 状态使用共同 token：FAILED / PARTIAL / SOURCE_ONLY / RUNTIME_PENDING / DEFERRED；SOURCE_ONLY 不等于通过。
- ID 配对检查只证明结构；金融条件、权限作用域、失败恢复和动作语义必须逐段同步，英文变更在同次修改中更新中文。
