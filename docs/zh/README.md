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

S19 #68 在 Backend ARD §41.25、Frontend ARD §13.14 和 UI Spec §14.12 增加 Binance Spot Testnet 提交/恢复契约。Fixture 测试覆盖本地 adapter 与持久化状态转换，但不等于 provider-hosted Testnet runtime 验收。点击式原型保持未修改。

S19 #70 在 Backend ARD §41.27、Frontend ARD §13.15 和 UI Spec §14.13 增加 Binance Spot Testnet 签名私有流与 REST 恢复契约。实现已在 `dev` 使用 fixture 验证；未使用真实 Testnet 凭据、未调用 provider、未下单。父 Spec #67 仍等待 provider-hosted Testnet 验收，点击式原型保持未修改。

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
