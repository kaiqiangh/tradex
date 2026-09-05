# TradeX 可点击原型 — v1.0 RevC

**文档修订：2026-09-05。当前原型交付：NOT PASS。**

此目录是无需构建的 HTML/CSS/JS 产品原型。行情、账户、订单和模型均为模拟数据，不连接券商、交易所、行情服务、Codex App Server、CLIProxyAPI 或 LLM。此次文档修订未改变原型代码。

## 从仓库根目录运行

~~~bash
python3 -m http.server 8080 --bind 127.0.0.1 --directory docs/prototype
~~~

打开 http://127.0.0.1:8080 。也可直接打开 [index.html](./index.html)，但浏览器行为可能不同。演示只使用假凭据。

## 目标与实际证据

- [PRD](../zh/TradeX_PRD_v1.0_RevC_zh.md) 定义业务范围与权限规则。
- [UI Spec](../zh/TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md) 定义 A–K 页面；§14 细化所需交互。
- [Frontend ARD](../zh/TradeX_Frontend_ARD_v1.0_RevC_zh.md) / [Backend ARD](../zh/TradeX_Backend_ARD_v1.0_RevC_zh.md) 定义实现边界与协议。
- [Coverage Matrix](../zh/TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md) 区分失败、部分、源码存在性、运行时待验证和未来范围。
- [QA Report](../zh/TradeX_Prototype_QA_Report_v1.0_RevC_zh.md) 记录当前缺陷、重现步骤及修复后验收。

原型展示目标的部分页面与状态，不代表所有安全规则均已演示成功。尤其不要把以下现状作为实现规范：

| 检查入口 | 已知缺口 | 回归场景 |
|---|---|---|
| DISARMED Live 账户撤单后 Arm | 错入新建订单审批 | QA-01 |
| 未知订单 / Manual Resolution | 捏造成交时间线，缺少证据核验 | QA-02/QA-03 |
| 休市、时钟、Disable All、sleep | 部分阻断和全局作用域未落实 | QA-04/QA-05 |
| 改模型/模式后查看历史 | 历史溯源随当前选择变化 | QA-06 |
| 编辑/重新生成 proposal | 草稿流程不完整，身份复用 | QA-07 |
| Screener / Context / Markets | 输入和标的选择未正确传递 | QA-08 |
| Backtest Send / LLM recovery | 入口与错误恢复不完整 | QA-09/QA-10 |
| 弹窗键盘 / 窄屏 | 背景可获焦点，线程/Provider 操作缺失 | QA-11/QA-12 |

## 保留的产品范围

Agent Mode 是 Ask / Research / Backtest / Trade；执行上下文与账户环境单独选择。非 Live 环境包括 Local Paper、Alpaca Paper、Trading 212 Demo、Binance Testnet、Bitget Demo；Live 操作另需账户 arming、不可变意图、特定审批、权限校验和权威对账。

存储目标为 SQLite + DuckDB + 文件系统，Parquet 为 Phase 2+ 可选；当前原型不执行真实数据库、keychain 或归档 I/O。数据源/风险参数等开放决策以 PRD 为准。完整视觉、读屏、运行时和真实提供方验收尚待完成。
