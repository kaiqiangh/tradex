# S28 经认证的 XNAS 日历 — 实施证据

日期：2026-10-01。子票 [#118](https://github.com/kaiqiangh/tradex/issues/118)，前置规范 [#117](https://github.com/kaiqiangh/tradex/issues/117)。固定审查基线：dev@`aa43091ea2eef732b5865b163cbeee3cac3e03c9`。状态：**IN_PROGRESS — 已审查实施检查点；真实原生读取待完成**。[英文配对](s28-calendar-evidence.md)。

用户显式选择符合条件、已安全保存的 Alpaca Paper 密钥引用，向固定 Paper 主机 GET `/v3/calendar/XNAS`，范围为昨天至今天+14天，参数 `timezone=UTC`。schema33 以 CAS 持久化选择/审计元数据；日历 SQLite 元数据和公开投影都不包含凭据或 vault reference。断开源保留账户/密钥。只有市场日历能力可以 AVAILABLE；OD-005 综合门禁仍为 UNVERIFIED，公司行为、停牌、历史调整各自保留状态。

生产者验证官方身份、UTC 区间、有界且有序的日期和成对扩展时段；解析前保存首次接收时间；绑定工作区、账户/源版本、进程会话、时钟代次和请求序号；在 Control Plane 锁外读取 vault/HTTP；随可信时间重新计算时段，不延长30秒执行年龄。失败或绑定变化取消资格。显式配置后禁止旧 Market/Live 合成 fixture 替代。既有风险、审批、派发消费同一共享投影；不会提升金融权限或其他前置依赖。

公开边界外部 HTTP/vault 测试覆盖保存选择、重开与断开；畸形/重复/空/不支持/过大/覆盖不足；认证、限流和传输失败；HTTP 期间源/账户/工作区/时钟变化；vault 期间账户断开；休市缺口、夏令时前后 UTC 边界、扩展交易和提前收盘；首次接收/过期和失败刷新；正常 UNVERIFIED/DISARMED Live 账户的预 Arm 日历风险检查及审批阻断；旧 fixture 隔离。实际 BrokerHttp loopback 测试还核对固定 GET/路径、拒绝 POST/DELETE/其他目的地、不跟随重定向、断链和12秒超时。这些都是合成外部提供方输入，不代表真实托管提供方通过。

真实 Rust/SQLite bridge 的公开 UI 验证已保存选择、可信时间错误及恢复、接收时间/版本/覆盖、独立能力，Settings 和 Markets 在390/768/1280px，Markets 使用同一 receipt，30秒后自动 Unknown/Unavailable 且 receipt 不续期，未保存选择禁用刷新，源替换清除旧目录观察，断开保留账户/密钥。没有 renderer 密码输入或浏览器错误。首次发现 IPC 存储版本上限32与实际33不一致，以及综合探测时间残留，均已修复并分开保存初次与复验结果。首次浏览器焦点/绑定失败属于工具尝试，不计为 PASS。

Standards 独立串行审查 PASS（0项硬性违规，2项非阻断维护建议）；Spec NOT PASS（0项行为缺陷，AC8 的实际认证读取与提交/推送验收未完成）。报告：[Standards](evidence/s28-calendar/standards-review.txt)、[Spec](evidence/s28-calendar/spec-review.txt)。维护建议未扩展为本票的非必要重构。

## 验证记录

- [当前 HTTP/vault 定向测试](evidence/s28-calendar/http-vault-recheck.txt)、[目录替换回归](evidence/s28-calendar/catalog-residue-green.txt)、[工作区投影版本回归](evidence/s28-calendar/workspace-projection-green.txt)。
- [首次公开 UI 的当前/过期观察](evidence/s28-calendar/ui-initial-results.json)、[新 IPC 的替换/断开复验](evidence/s28-calendar/ui-recheck-results.json)、[Settings 布局](evidence/s28-calendar/ui-layout.json)、[Markets 布局](evidence/s28-calendar/ui-market-layout.json)。
- [Settings 当前](evidence/s28-calendar/ui-settings-current.png)、[Markets 当前](evidence/s28-calendar/ui-market-current.png)、[Markets 过期](evidence/s28-calendar/ui-market-expired.png)。
- 全套测试中的旧迁移夹具把 user_version 降到6/8/30，却保留新日历表，且仍期望 schema32；产品正确拒绝重复建表。夹具现已同其他新增表一起移除日历表，并期望 schema33；未来版本拒绝使用34；[精确迁移复验](evidence/s28-calendar/artifact-migration-green.txt)通过。11项 watchlist/工作区迁移检查也通过。最终 `npm run check` 通过：17项 Node、348项 Rust，36项既有忽略/未验证项，schema/type/build 与203需求/70界面/13QA/23baseline追踪检查。完整 integration 通过382项 Rust，同样36项忽略/未验证。首次 Gateway binary unit 命令实际运行0项测试，不能证明 Gateway 测试覆盖；独立 `integration-test order-gateway-runtime --test order_gateway` target 通过23项测试。普通 `npm run desktop:build`（不含 integration-test feature）通过。[全套检查](evidence/s28-calendar/check-results.json)、[integration 输出](evidence/s28-calendar/full-integration.txt)、[Gateway 输出](evidence/s28-calendar/gateway-integration.txt)、[桌面构建输出](evidence/s28-calendar/desktop-build.txt)、[原生构建哈希](evidence/s28-calendar/native-build.json)。原生 schema32→33 迁移生成自动备份，保留4个账户，源 generation1/audit1 已持久化，见[脱敏元数据](evidence/s28-calendar/native-after-selection.json)。原生真实读取正等待受保护的 Keychain 认证；选择/存储迁移本身不是托管提供方通过证据。

原始 `.log` 为本地忽略文件。版本化 `.txt` 保留输出并统一清除行尾空格与文件末尾空白行；夹具或编译失败不当作产品 RED，除非实际公开行为失败。

原报价正向审批/Prepare/派发 [#116](https://github.com/kaiqiangh/tradex/issues/116)、Trading212 验收 #113、整体前置规范 #117 和 map #1 均保持 OPEN。S17 真实 Paper 和 S27 实际睡眠唤醒保持调度跳过/未验证。本日历切片未 Arm、未签发审批、未金融预留、未 broker 写操作。未提交 main PR 或合并。
