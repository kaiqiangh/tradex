# S29.5 精确账户开放订单与容量输入验收

[English](s29-spot-capacity-inputs-evidence.md)。[提供 Binance 精确账户开放订单与容量输入](https://github.com/kaiqiangh/tradex/issues/126)：来源实现VERIFIED，本记录时远端/追踪交付待完成。源码提交`b4d137a7b3064218da4a9e5e1147a980a7d0e810`，固定复审基线`4f06df633832aa2ed89bf675bcb219a15c41731d`。161个冻结源码/构建输入哈希均匹配提交字节；最终验证后无运行时变化。

不可变普通Binance Live意图现有显式可信容量get/CAS refresh，后端导出用途并使用固定signed account/openOrders/openOrderList。类型化观察保留精确单位、原始free/locked与BUY组成、有界计数/分类/列表覆盖、缺失腿、部分成交/外部资产不确定性、读取窗口/最早收据/原始update与transaction时间、摘要及非原子质量。刷新失败、过期、绑定、晚到和重开撤下当前观察；捕获保持有界且不可变。动态资格始终UNAVAILABLE，不审批订单或提升独立金融门禁。

最终68个公开规则/容量场景包含在17Node/466Rust中，39既有ignored保留。49个隔离BinanceHot、19StockHot、23Gateway通过。生成IPC/前端构建/追踪、普通桌面及准确Gateway pin通过；实际React→Rust current/captured/Arm前BASE/列表/原始transaction/错误/读取中/恢复/历史、键盘1280/768/390通过，控制台无警告/错误。[当前检查点](evidence/s29-spot-capacity-inputs/implementation-checkpoint.json)、[检查](evidence/s29-spot-capacity-inputs/checks-review-final.json)、[普通构建/pin](evidence/s29-spot-capacity-inputs/desktop-build-review-final-inputs.json)、[最终UI](evidence/s29-spot-capacity-inputs/ui-review-final-report.json)、[窄屏截图](evidence/s29-spot-capacity-inputs/ui-review-final-captured-390.png)。

独立串行[Standards再Spec](evidence/s29-spot-capacity-inputs/code-review-final.md)均PASS，0剩余/可执行发现。Standards初次发现双语进度不一致及重复数量验证；均修复，且针对复审重构刷新全部检查/构建/UI。早期阶段明确绑定before-review检查点；编译失败和重放配置/CAS尝试不作为最终PASS或产品RED。[验收映射](evidence/s29-spot-capacity-inputs/acceptance-before-handoff.json)区分已证明来源条件与待完成交付。

使用真实React/Rust/临时SQLite/outbox，仅外部HTTP/vault/WS为fake。未使用正向金融authority setter、真实私有凭据/服务商金融验收、原生启动、物理生命周期、订单mutation、Arm或同意。隔离1427 host/tab已清理，用户1421/原生应用未触及。仓库无CI workflows，不运行/声称远端CI。完整S29 #121、S28、S17、物理S27及map/main门禁仍OPEN，全局需求/界面证据未提升。原型和main未改。
