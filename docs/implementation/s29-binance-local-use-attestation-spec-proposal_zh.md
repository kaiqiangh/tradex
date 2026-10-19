# S29.11 Binance 来源绑定的本地用途确认——规范提案

> **状态：待人工决定的提案。** 本文尚非规范性契约，不改变运行时行为、S29.11 验收勾选或 #135 状态。只有该策略决定获批并准备好独立实现票后才能实施。

## 问题陈述

Backend ARD §41.43 与 `risk.rs` 当前对普通 Binance Spot Live 行情无条件将 `MARKET_DATA_USE` 设为 `UNAVAILABLE`。S29.11 AC6 和 AC10 要求真实 owning plan 能到达同一个 RiskDecision 和 Prepare consumer。Backend §41.60 中现有的 Alpaca 确认刻意仅限本地 `POLICY_VALUATION`，不能使 Binance 金融用途或 Prepare 合格。技术可用、成功采集以及现有 Alpaca 确认都无法消除这一冲突。

本提案评估增加一项绑定所选 Binance 公共来源、且用途严格受限的显式确认。它不声称 Binance 已授予许可，也不声称服务商已独立核实用户权利。

## 建议

为用户本人在本地、非商业地使用所选普通 Binance 公共 Spot 数据进行**交易决策支持**，增加独立且显式的 Control Plane 确认。范围限于 TradeX 本地报价/估值输入、owning plan 计算、风险评审、审批决策及本地 Prepare/预留流程。要求用户确认其已查看当前适用条款，并认为这些条款允许该用途。

且仅当当前 source profile 与确认绑定匹配时，owning `MARKET_DATA_USE` 检查才可返回 `PASS`，并附明确的 `USER_ATTESTED_LOCAL_ONLY` 原因/provenance。行情观察本身继续保持 `dataUseRights=UNVERIFIED`；不得改写 provider evidence，也不得将状态描述成经 provider 验证的 entitlement。未确认、范围过期、不匹配、不支持或被已知适用限制否定时，检查继续为 `UNAVAILABLE`（已知禁止用途时为 `REJECT`）。

这是产品决定：在限定的本地决策用途上依赖用户声明。它不是法律意见、服务商确认，也不是服务商条款允许此用途的证明。若产品不接受这种依赖，应保持当前 fail-closed 行为并修改 S29.11 正向 AC6/AC10 目标；不得增加仅测试可用的绕过。

## 用户故事

1. 选择普通 Binance Spot 公共数据时，用户可以查看适用来源条款，并明确确认本人在本地、非商业的决策支持用途。
2. 用户可以看到确认仅适用于选定来源、数据集、标的、用途和本地目的地，并明确标注为本人确认而非 Binance 确认。
3. 用户可在当前及捕获的评审中查看确认和 provenance，同时行情本身仍显示 provider rights 未验证。
4. 用户不能用此确认启用共享/商业用途、再分发、导出、model/tool 传输、其他来源或实际订单提交。
5. 来源、数据集、用途、目的地、条款/策略或法域变化时，旧确认会失效，之后 RiskDecision 或 Prepare 不得继续依赖它。

## 建议的实现决策

### 范围与证据分离

- 保持三个事实独立：技术采集、provider-observed entitlement、用户确认的本地用途策略决定。本提案只改变第三项。
- 新增 Binance 专属本地用途，例如 `LOCAL_TRADING_DECISION_SUPPORT`；不扩展 Alpaca 现有 `POLICY_VALUATION` 确认，也不复用其 `FX_DATA_USE_RIGHTS=VERIFIED` 投影。
- 确认绑定 workspace 和本地用户、`BINANCE_SPOT_PUBLIC`、provider `binance`、ordinary 环境、已审核的公开 Spot 数据集/profile、S29.11 当前使用的准确标的集合（BTC/USDT、ETH/USDT；仅在 owning 费用/估值模型实际消耗时包含 BNB/USDT）、声明法域、策略版本、适用的一手条款引用/修订、用途、`LOCAL_ONLY` 目的地及允许的本地保留/consumer 边界。Binance 公共行情没有远端账户绑定，不得虚构该字段。
- 由 Control Plane 根据已保存的来源配置和策略状态推导绑定。Renderer 输入只能表达显式肯定确认及预期 state version；不能提供 rights status、provenance、digest、source identity 或 binding。
- 确认仅记录用户声明。它不能覆盖已知禁令、超出范围的来源/数据集、不支持的法域或已审核 source policy 判定为阻断的未解决策略状态。确认前必须展示当前条款及其复核来源。

### 风险与 Prepare 行为

- 存在当前且匹配的确认时，仅所选普通 Binance 来源的本地 `MARKET_DATA_USE` 检查可通过；其原因/provenance 必须标明 `USER_ATTESTED_LOCAL_ONLY`。不得更改 `BinanceSpotQuoteEvidence.dataUseRights`、source entitlement 或其他来源/本地模拟行为。
- 无当前匹配确认时，保留 `DATA_USE_RIGHTS_UNVERIFIED` 与 `UNAVAILABLE`。已知适用限制无论是否确认都继续拒绝。
- 通过只移除此项可达性阻断，不表示整个 RiskDecision 合格。新鲜报价/深度、venue admission、instrument rules、账户健康/权限、source-use 要求、费用/FX 币种、余额、预留、审批及其他独立门禁继续必需。只有通过所有这些门禁的真实 producer-to-Prepare 路径才可满足 S29.11 AC6。
- 若其他独立门禁均通过，该确认可支持本地审批和耐久 Prepare/预留。它不授权 Arm、PLACE、取消/对账变更、provider mutation 或订单提交。后续金融操作仍需另行授权的用户流程。
- 仅持久化确认 binding/status/provenance 及现有有界风险/评审 digest。公开原始订单簿保持临时且仅在本地；风险历史不保存原始报价/订单簿。原始行情或派生来源值不得发送至云模型、工具、研究、导出或其他外部 consumer。

### 交互、失效与捕获

- 在已保存 Binance 来源设置中增加默认未选中的确认框和独立“记录我的本地用途确认”操作。用普通语言说明这是用户本人确认的本地用途，不是 Binance 核验；列出准确标的/用途和排除的目的地。不发起 provider 请求。
- 未保存符合条件的来源、设置尚未保存、来源/策略/条款 profile 不支持，或声明范围包含共享、商业、导出、外部用途时，禁用此操作。
- 显示 `由你确认：仅限本地决策支持`，并展示来源、数据集、用途、目的地、条款引用、法域、时间及策略版本。不得因该记录而将 provider entitlement 显示为 `VERIFIED`。
- 来源断开/重选、source/dataset/标的范围变化、策略或适用条款修订、法域变化、用途/目的地/保留变化、主动撤销或条款复核过期时，使确认失效。新的报价 receipt 或缓存读取不得创建或续期确认。
- 当前视图使用 Control Plane 当前投影。捕获的评审不可变保存当时决定及确认 provenance，且明确标记为捕获；不得借此续期当前资格。

## 后续实现票的验收标准

- [ ] 配对 PRD、Backend ARD、Frontend ARD、UI 规范及 S29.11 实现规范定义同一 Binance 用户确认的本地用途，并明确区分 provider entitlement 与仅限 Alpaca 的 `POLICY_VALUATION` 确认。
- [ ] 默认关闭的独立 Settings 确认由 Control Plane 绑定已保存来源、数据集、标的范围、本地用户/workspace、法域、当前策略/条款修订、用途、目的地及保留边界；renderer 无法设置 rights 或 binding 字段。
- [ ] 缺少确认、binding 过期/不符、profile 不支持或存在已知限制时，真实所选来源的 `MARKET_DATA_USE` 按准确原因拒绝。有效确认仅以用户确认 provenance 通过；provider evidence 仍未验证。
- [ ] 真实外部 producer evidence 经公开 React → typed Rust Control Plane → owning statement → RiskDecision → approval/Prepare consumer；不引入正向 finance snapshot、植入 approval、测试专用 source label 或 rights setter。
- [ ] RED/GREEN 覆盖 binding/失效、条款/策略变化、撤销、不支持或已知禁止范围、Save/confirm 不发网络请求、来源隔离、captured state 和无原始/派生行情外泄。
- [ ] 仅当 S29.11 所有独立门禁都通过时才验收正向 Prepare。此确认本身不得产生整体金融 PASS、approval、Arm、PLACE、provider mutation 或 execution authority。
- [ ] 键盘和响应式 Settings/current/captured review 在390/768/1280px验证；英文/简体中文契约与追踪同步。

## 测试决策

沿用现有公开 React → typed Rust Control Plane → disposable SQLite/outbox seam，仅伪造外部 HTTP/vault/WS/clock producer。按 RED 后 GREEN。测试有效确认、缺失确认、workspace/source/profile/instrument/purpose/destination/jurisdiction/terms version 错配、撤销、策略/条款修订、已知限制、来源变化/断开及并发 state-version 冲突。断言返回用户确认原因，同时 `dataUseRights=UNVERIFIED` 不变。让同一真实 producer 结果通过 owning statement、RiskDecision、approval 及 Prepare；另行验证其他输入缺失时所有独立门禁仍阻止。检查保存/捕获 UI 与外发边界，确认原始订单簿/报价不会进入持久化历史、model/tool 输入、research、export 或其他外部 consumer。

## 不在范围内

- 将 provider entitlement、法律/法域批准或 Binance 确认描述为已观察或经独立验证。
- 商业/共享用途、再分发、发布、云端/model/tool 使用、导出、超出现有本地策略的保留、其他 Binance 数据集、其他 venue 或未绑定标的。
- 修改 Alpaca 确认、交易 FX 资格、费用/资金、账户权限、新交易/订单类型、Arm、PLACE、订单提交、provider writes 或真实金融操作。
- 关闭 S29.11 或其 parent、修改验收勾选，或宣称整个应用/native/provider/physical/release 已完成。

## 实施前需决定

是否接受这一有限产品策略：允许用户显式绑定来源的声明用于本人本地、非商业的 Binance 交易决策支持并经过本地 Prepare，同时明确 provider entitlement 仍未验证且其他门禁保持独立？若批准，应创建独立实现票，并在实现前同步修改配对规范。若拒绝，则保留当前拒绝，并修改 S29.11 正向 AC6/AC10 预期，不应在代码中弱化门禁。

## 规范依据

- Backend ARD §41.43：普通 Binance Spot 行情来源及当前无条件 `MARKET_DATA_USE` 拒绝。
- Backend ARD §41.60 / Frontend ARD §13.46：现有仅适用于 Alpaca 本地 `POLICY_VALUATION` 的确认，明确不足以覆盖 Binance 或 Prepare。
- S29.11 #135 AC1、AC6、AC9、AC10、AC14：source-use 契约及 owning plan/Risk/Prepare 正向验收。
- 既有提案：[ #145](https://github.com/kaiqiangh/tradex/issues/145)。本提案是扩展决议，不修改 #145/#146。
