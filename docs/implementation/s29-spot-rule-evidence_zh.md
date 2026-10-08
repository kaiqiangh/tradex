# S29.2 — 精确 Spot 规则来源证据

单票 [#123](https://github.com/kaiqiangh/tradex/issues/123)，父项 [#121](https://github.com/kaiqiangh/tradex/issues/121)。基线 `4388d9c4ae460ee5f4d8a6a21b0ced5cfa3fe374`；源码 `b726bb38df9280cb290d8efedc72cc2f2dbbff95`。[English](s29-spot-rule-evidence.md)。

状态 LOCAL_ACCEPTANCE_PASS；远端交付与 tracker 收口待完成。当前源码已本地提交。完整检查、普通桌面构建、实际 Rust-backed UI 复测及独立 Standards → Spec 串行复审通过。父项和金融门禁保持 OPEN。

公开规则来源借用符合条件的已保存普通 Binance Live 账户/密钥；无 I/O 保存精确 BTC/USDT 或 ETH/USDT，然后有界读取固定主机的签名/公开观察，经真实 Control Plane 返回类型化证据。配置持久化；运行时资格不随重开恢复。负向准入进入 InstrumentRules 拒绝；成功元数据不授予逐 Proposal PASS。契约见配对 Backend §41.42、Frontend §13.27、UI §14.26。此前诊断章节重复编号在两语言修正为 UI §14.25；原型代码未改。

| AC | 实现边界与当前证据 |
|---|---|
| 1 | 公开 configure/connection/disconnect CAS、保存选择、无提供方 I/O、保留借用账户/密钥、元数据重开：`binance_rules.rs` 选择/时钟/重开测试。 |
| 2 | 外部 HTTP fake 验证普通 Live 端点、精确标的、敏感 header、原始 HMAC；账户/IP 限额跨选择/workspace 保留；真实 vault 截止时间无 HTTP 或迟到发布：`binance_rule_limits.rs`。 |
| 3 | 原始布尔/整数字符串/精确十进制；身份/范围/重复/哨兵/区间失败关闭。合法大于 2^53 计数无损，越界 int64 拒绝；可选文档指数受限；未知有效规则/字段显式未决。 |
| 4 | 权限组内 OR/组间 AND，HALT/BREAK、canTrade/类型/Spot/密钥/系统/锁分别诊断；恢复仍解除武装，InstrumentRules UNAVAILABLE。精确撤单有独立既有回归。 |
| 5 | 实际 30.1 秒过期、失败保留首次接收/材料、HTTP 中改来源不得发布、时钟重校验/未来时钟/重开使资格失效。 |
| 6 | Settings/Markets/Trade 已实现；中间真实 UI 发现 schema-36 与 Markets 缓存过期缺口并修复。最终当前构建 Settings/Markets/Trade 的 1280/768/390 无溢出、Enter 操作、失败保留、BTC/ETH 精确绑定、错误标的拒绝、HALT 风险拒绝、不匹配、自动过期及断开/重新选择/读取恢复记录于 final-ui.json。 |
| 7 | 精确后端 market/proposal 账户绑定与负向检查；逐单参考/网格/名义金额/计数/仓位/资产/PRICE_RANGE 义务逐项未决。无正向金融种子。 |
| 8 | 公开边界 RED/GREEN 与故障测试；最终真实 React→Rust 记录于 final-ui.json。普通提供方/原生读取/金融写操作分开，未声称通过。 |
| 9 | 生成 IPC 与中英文契约同步；完整检查/Gateway 已通过，普通桌面构建及匹配固定 Gateway 字节已通过；审查/远程交付待完成。 |

不变源码树检查：schema/typecheck/前端构建、17 项 Node、408 项普通 Rust、203 条需求追踪；447 项 integration-feature Rust；真实子 Gateway 专项 23 项。普通和集成各保留 39 项既有 ignored，不计 PASS。见[原始证据](evidence/s29-spot-rules/)。历史完整检查失败来自固定迁移测试期望（35→36、未来 36→37），未发现数据丢失。初始限额测试错误认为两个 UID 不能在 IP 余量耗尽前各自耗尽；修正后的第三 UID 验证共享 IP 耗尽。失败尝试明确保留，未伪装通过。

真实 RED/GREEN 包括公开选择缺失、采集、精确市场投影、数量范围倒置、负向准入、可选指数、逐项未决义务、int64 越界。UI RED/GREEN 包括 schema 兼容及 Markets 自动过期。密钥/签名反射、原始类型拒绝、截止时间/限额和重开是公开边界回归。未使用用户真实秘密或真实金融账户观察。

执行资格与 Spot 数据使用权保持 UNVERIFIED。S29 后续仍负责真实报价/深度/时效/权利、实际所需 FX/费用、逐 Proposal 规则解释、Gateway 即时复核、Live PLACE/私有流/恢复及具体另行授权的提供方/金融验收。S28、S17、物理 S27 门禁不变；全 goal 满足前禁止 main 操作。

最终 UI 使用新的临时数据目录与外部 HTTP/vault 假响应，无金融写操作。首次缓存路径属于上轮测试 bridge，被新的 bridge 按设计拒绝；公开 Workspace/Open 成功恢复。ETH 错误标的读取发生于外部 fake 仍返回 BTC HALT 内容时；改回正确外部响应后精确 ETH 恢复。原生 select 选择使用文档化 selectOption UI 动作；已执行 Enter 操作，IAB ArrowDown 尝试不声称为平台原生方向键选择证明。账户保持 CONNECTED、credential CONFIGURED、DISARMED、execution BLOCKED、private stream NOT_CONFIGURED；浏览器 warning/error 捕获为空。临时标签页和 QA server 已停止，用户的 1421 标签页未动。


初次 Standards 为 PASS，含 1 条非阻断私有描述符启发式建议；独立 Spec 为 NOT PASS：空权限集合被隐含视为满足成员要求，订单形式标志遗漏。现已逐项通过公开刷新边界的真实 RED/GREEN 修复。空外层/内层集合采集失败，并保留不可用旧观察。8 个原始可空布尔订单形式标志得到保留；提供类型畸形则失败，缺失保持未观察，高级形式保持明确不支持。双语契约同步。初次报告见 `review-initial.md`；修复日志见 `review-permission-red-actual.txt` / `review-permission-green.txt` 与 `review-order-flags-red.txt` / `review-order-flags-green.txt`。

当前源码 `b726bb3`：完整本地检查通过（410 Rust / 17 Node，39 既有 ignored），集成模式 449 Rust 通过（39 既有 ignored），Gateway 23 通过，公开规则来源 15 用例通过。普通桌面构建与修复后的针对性 UI 复测已通过；串行独立复审及远端交付仍待完成。`review-permission-before-test-write.txt` 执行 0 用例，不构成 RED；`review-typecheck-before-generation.txt` 检查尚未生成的类型，由成功 `review-typecheck.txt` 取代。此前 `final-ui.json` 仍绑定 `1d20747`，不得重标为当前源码证据。

当前普通桌面构建通过；`review-desktop-build-inputs.json` 确认程序包含交付 Gateway 文件的固定 SHA-256。`b726bb3` 的 `review-ui.json` 复测 Settings/Markets/Trade 八项标志和高级形式不支持边界，各页 1280/768/390 均无溢出；公开 Enter 保存/刷新/草稿/提案操作及空 console 捕获通过。临时页面/服务停止、viewport 复位。此证据补充此前 `1d20747` 更广的故障/恢复 UI 证据，不将旧证据重标。

最终[串行双轴复审](evidence/s29-spot-rules/review-final.md)：Standards PASS（0 硬性违规、1 非阻断私有描述符建议）；Spec PASS（0 可执行问题）。[验收清单](evidence/s29-spot-rules/acceptance-manifest.json)记录源码、检查、各类未验证边界。
