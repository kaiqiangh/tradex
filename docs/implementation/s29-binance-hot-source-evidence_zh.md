# S29.3 — Spot 来源验收

所属票：[提供 Binance Spot Hot 报价与连续深度来源](https://github.com/kaiqiangh/tradex/issues/124)。父项[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)保持 OPEN。[English](s29-binance-hot-source-evidence.md)。

运行时源码 `d9abe2470ed0b5d69b08c645625a4aa30d988511`；公开边界回归源码 `0ea2b96c8c3d95b44ad0eb591fc90743b87a4898`；完整检查树 `776efe0095dffb21493d10feeb83fe1e51a33ac4`。独立 Standards→Spec 串行复审覆盖固定基线 `5edf18d01122d1871b6c94aeafefe708435b7f38` 至 `4898850e55c904f43072a32bdd05d5c3651b41d0`，已核对与 origin/dev 相同。随后收口仅修改文档/证据；发布后记录实际 tracker 关闭和交付回执，本证据本身不改变 tracker 状态。

| 标准 | 已验收行为与证据 |
| --- | --- |
| AC1 | 公开来源 CAS/保存/断开、只恢复配置、canonical BTC/ETH 与凭据隔离；[选择 RED/GREEN 与拒绝](evidence/s29-binance-hot/selection-progress.md)、[实际 Settings](evidence/s29-binance-hot/settings-ui-progress.md)。保存/重开不产生网络权威或合成回退。 |
| AC2 | 精确所属 Hot 代次、实际 worker 释放与迟到隔离；[生产者](evidence/s29-binance-hot/producer-progress.md)、[Markets/选中 Trade 原生隐藏](evidence/s29-binance-hot/native-hidden-progress.md)。隐藏关闭连接、旧材料 UNAVAILABLE，返回产生新租约/连接/材料。 |
| AC3 | 实际外部 HTTP 快照+WS diff 对齐、原始提供方时间/无损 ID、桥接/重叠/过时/缺口/删除；[生产者 RED/GREEN](evidence/s29-binance-hot/producer-progress.md)、[最终49来源场景](evidence/s29-binance-hot/final-source-49-regression.txt)。生产普通主机固定，loopback 不证明真实服务就绪。 |
| AC4 | 精确 decimal/BASE、原始类型、双边不交叉、有界已知范围、容量失效与最多20层投影；[深度范围](evidence/s29-binance-hot/depth-scope-progress.md)、[准入/深度](evidence/s29-binance-hot/admission-depth-progress.md)、最终49场景。没有虚构最后成交/参考价格/日历或 USD 平价。 |
| AC5 | 首次 receipt/材料不可变，重复/缓存不续期，可信 wall/provider/单调时效及更严金融 TTL、代次退役；[消费者](evidence/s29-binance-hot/consumer-progress.md)、最终49场景、[冻结 Trade 复核](evidence/s29-binance-hot/trade-ui-frozen-regression.json)。 |
| AC6 | 共享 IP HTTP 配额/冷却/期限、有界传输/取消、控制余量、最多3次重连、终态封禁及计划轮换；[传输](evidence/s29-binance-hot/transport-progress.md)、[轮换/HTTP RED/GREEN](evidence/s29-binance-hot/rotation-http-progress.md)、最终49场景。每个隔离串行场景内保留生产配额。 |
| AC7 | 实际 Settings/Markets/Trade 出处、BASE 深度和独立权利，键盘1280/768/390及显式恢复；[Markets](evidence/s29-binance-hot/markets-ui-progress.md)、[Trade](evidence/s29-binance-hot/trade-ui-progress.md)、[缺口/Retry](evidence/s29-binance-hot/recovery-ui-progress.md)、原生隐藏、[股票 Hot19](evidence/s29-binance-hot/rotation-stock-hot-current-regression.txt)。 |
| AC8 | 只满足所属报价/hash检查与精确账户准入；独立 MARKET_DATA_USE/rights仍为 UNAVAILABLE/UNVERIFIED。[消费者守卫](evidence/s29-binance-hot/consumer-progress.md)、[当前研究/turn/artifact/导出构造器论证](evidence/s29-binance-hot/current-source-use-acceptance.md)、[公开负向回归](evidence/s29-binance-hot/source-boundary-public-regression.txt)、[风险/历史/重开](evidence/s29-binance-hot/consumer-captured-risk-history-regression.txt)。没有整体金融 PASS、Arm/审批/PLACE 或法律接受。 |
| AC9 | 公开 React→typed Rust→临时存储/outbox→外部 HTTP/WS；真实局部 RED/GREEN 与单列既有行为回归；[完整检查](evidence/s29-binance-hot/final-full-check.txt)、最终49、[Gateway23](evidence/s29-binance-hot/rotation-order-gateway-current-regression.txt)、[普通构建绑定](evidence/s29-binance-hot/rotation-ordinary-release-binding.json)、[Standards PASS](evidence/s29-binance-hot/final-standards-review.md)、[Spec PASS](evidence/s29-binance-hot/final-spec-review.md)。双语契约与20项产品清单一致。 |

当前完整检查通过413 Rust、17 Node；39既有 ignored 仍未验证。49来源场景为独立应用进程逐个串行，每个场景内执行生产配额，不是同进程全集或真实共享 IP 验收。股票19、Gateway23、普通发布构建绑定未变化运行时 d9abe24，不冒充文档/测试新增后的重跑。[最终绑定](evidence/s29-binance-hot/final-current-verification.json)保留区别；没有配置远端 CI，不声明远端 CI PASS；既有包体积提示保留。

Standards0硬违规/1非阻断重复逻辑建议；Spec0剩余阻断/范围扩张/确认实现错误。初始 NOT PASS 与所有失败/部分尝试均保留。原生 Chrome 隐藏属于隔离 React/Rust 证据，没有直接采样隐藏 DOM；实际原生动作与外部 worker 效果证明生命周期，不表示普通打包应用或真实金融验收。

来源使用论证证明当前生产者没有自动向研究/turn/artifact/导出传递原始盘口的路径，不证明通用文本 DLP、实际成功模型/导出往返、用户许可或人工复制权利。没有 CP 正向权威注入、真实凭据/写请求或法律接受。自建测试进程/临时根已清理，原用户标签已恢复；保留可恢复的关闭测试组记录，同步删除对话框已取消。

父项全部故事、逐 Proposal 规则/参考/动态输入、实际需要的 FX/费用、即时已认证 Gateway 预检、私有流/订单生命周期及单独授权真实金融验收仍须完成。S28/S17/物理 S27 保持此前未完成状态。原型代码未改。完整 goal 活跃，main 未改变。
