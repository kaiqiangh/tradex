# S29.1 — Binance Live 账户交易状态证据

实现票：[核验 Binance Live 账户交易状态](https://github.com/kaiqiangh/tradex/issues/122)。父规范：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。审查基线：`fe20d0aae1c0eea9df43c1e24f32199a92ff30b9`；实现：`ec9929ec8d449cad142e03da6acb19545a1ceb7d`。[English](s29-account-state-evidence.md)。

本子票提供账户诊断与失败关闭的安全消费。只有普通 Binance Live 的公开连接测试、验证和刷新读取固定域名的系统状态与签名精确账户交易锁定状态。可选类型化状态与账户数据分别持久化，保存保守的首个响应接收时间、Provider 锁定更新时间及可选计划恢复时间。Normal/unlocked 不表示金融就绪。维护、锁定、缺失、未来或不可解析的接收时间以及超过 30 秒的接收时间，均不能满足 Arm 或 PLACE 账户健康门禁。精确 CANCEL 与未知订单对账保留各自独立守卫。Binance Live PLACE 与私有流仍属于父项未完成工作。

## 验证

- 公开类型化 Control Plane → 一次性 SQLite/outbox → 外部假 vault/HTTP：新增五项测试覆盖正常类型化观察及重开、原始对象数字 token 拒绝、维护/锁定 Arm 与 proposal risk 拒绝、真实经过 30.1 秒后的过期、格式错误/缺失/未来 Provider 数据、密钥反射与配额故障、可信数据/成功同步时间保留、保持撤防的恢复及旧请求结果拒绝。没有新增隐藏金融权威种子，没有真实 Provider 请求或写操作。
- 真实 RED/GREEN 记录保留于[原始证据](evidence/s29-account-state/)。首次正常 GREEN 尝试暴露测试预期 UTC 换算错误，依据独立 datetime 换算修正，没有改动生产时间。原始对象数字 RED 暴露对象被接受为整数，通过独立于任意精度 Value 转换的原始字节类型化解析修复。历史完整检查失败来自旧合成生命周期及外部 HTTP 回归夹具缺少新诊断；兼容修正明确只属于测试，这些合成金融夹具不算 S29 正向验收。
- 修复后 `npm run check`：生成 schema、类型/构建检查、17 项 Node 与 393 项普通 Rust 测试通过；追溯检查覆盖 203 项需求、70 个画面、13 个 QA 场景及 23 个基线文件。integration-test workspace 432 项通过；限定真实子 Gateway 回归 23 项通过。39 项既有 ignored 测试仍未验证，这些结果不表示完整 map 验收。最终空白检查通过。
- [修复后 UI 记录](evidence/s29-account-state/remediation-ui.json)绑定最终实现 SHA 与实际 React → 真实 Rust Control Plane → 一次性工作区，仅使用外部 HTTP/vault 夹具。验证公开连接/确认、键盘刷新、维护、锁定、格式错误保留及正常重试/确认。1280/768/390px 文档宽度等于视口宽度，Arm 持续禁用。最终公开账户读取确认 CONNECTED/DISARMED/NORMAL/unlocked，`canArm=false`。控制台无 warning/error。[390px 截图](evidence/s29-account-state/remediation-maintenance-390.jpg)。两次初始 UI 记录作为历史另行保留，未代替最终修复后重放。
- 不包含 integration-test feature 的修复后普通 `npm run desktop:build` 成功。[构建输入及二进制哈希](evidence/s29-account-state/remediation-desktop-build-inputs.json)匹配最终实现；桌面程序内 Gateway 哈希与恢复后的独立固定 Gateway 相同。这是未打包构建，不是原生 UI/Provider、系统认证、签名公证包或物理唤醒验收。

第一轮 Standards 通过且无发现；Spec 发现一项 AC4 精确 CANCEL 持久化依赖问题。[审查记录](evidence/s29-account-state/review-round-1.md)。实际过期的底层兼容回归复现精确 CANCEL 成功观察持久化误触发撤防；按可信任务划分观察范围后通过。该测试使用既有合成生命周期夹具，不创建审批/Prepare/派发，不算公开金融验收。身份、权限、凭据和真实降级守卫保留；普通账户持久化仍会因过期诊断撤防。初始检查/UI/构建记录保留为历史。修复后检查/构建/UI 已通过；独立 Standards → Spec 串行复审通过，两轴均无发现。[最终审查](evidence/s29-account-state/review-final.md)及 [AC 核对](evidence/s29-account-state/acceptance-audit.md)绑定相同源码；源码/证据交付与 tracker 结票另记于 closure receipt。

## 剩余范围

S29 父项仍需已配置的精确 crypto 规则、真实报价/深度/来源权限、实际必要的金融 FX/费用输入、Live PLACE/私有流生命周期、故障恢复及具体授权的真实 Provider 验收。USDT 不等于 USD。S28 及其金融验收保持 OPEN；S17 真实 Paper 与用户跳过的物理 S27 唤醒仍未验证。点击式原型代码未改。没有提交 main PR 或合并 main。
