# S29 条件费用契约 — 已批准用于AC6；AC5仍豁免

用户已于2026-10-15批准此有界条件费用契约，以继续S29.11 AC6工作。RevC基线产品文档保持不变；本文记录owning plan范围内的决定，不授予数据使用权，也不满足任何独立金融门禁。AC5仍豁免/未验收，不执行其第二次读取活动claim的验收流程。

## 证据与现行冲突

Binance 官方 `account.commission` 响应包含当前 standard、tax、special 费率，以及 BNB 折扣标志、币种和乘数；它不提供未来成交唯一的实际扣费币种。已完成成交的历史记录才包含实际 `commissionAsset`。Binance 当前帮助说明描述了条件分支：默认从收到的资产扣费；启用 BNB 扣费且符合条件时使用折扣 BNB；BNB 不足时收取原费；因成交前不知道最终成交数量，页面金额只是估算。

来源：

- [Binance Account — Spot WebSocket API](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-api/account)
- [How to Use BNB to Pay for Fees and Earn up to 25% Discount?](https://www.binance.com/en/support/faq/detail/115000583311)
- [What is Binance Spot Trading Fee and How to Calculate](https://www.binance.com/en-ZA/support/faq/detail/e85d6e703b874674840122196b89780a)

在2026-10-15决定之前，S29.11 owning-plan规则把未来实际收费币种未知视为owning statement不可用。AC5还要求通过真实公开账户读取路径验证至少一个活动同账户reservation的精确本地claim扣减。活动reservation由公开Prepare创建，而Prepare又要求符合资格的审批。直接插入reservation会违反“不种入金融记录”的验收规则。因此，新的合成端到端AC5测试当时形成了闭环门禁；随后AC5被豁免，不再执行该第二次读取流程。

## 已批准的契约

把收费币种表示为有界条件结果，而不是已知的未来实际币种：

- 成交后实际费用的 `feeCurrency` 保持 `UNKNOWN_UNTIL_FILL`，并保留显式的“非费用保证”限制。
- 只有当账户绑定的佣金声明、钱包覆盖、所需行情引用、已配置的数据源策略，以及每项独立风险/权限门禁均完整且新鲜时，才允许完整且绑定同一intent/账户的当前 owning estimate 使review和Prepare具备资格。
- 把模型允许的结果范围限定为文档描述的 BNB 折扣分支与收到资产回退分支。不推断第三种币、平价、零成本或受保证的成交/费用金额。
- 当前选择 BNB 分支时，推导并原子预留当前原生 BNB 额度；选择收到资产回退时，将它保留为流入资产扣减，不虚构流出费用 claim。
- 审批复核需展示已选分支、备选分支、精确钱包回执与本地 claims、模型限制，以及 `ACTUAL_FEE_ASSET_PENDING`。审批仍由用户授权，不能成为券商执行的费用上限。
- Prepare 和 Gateway 即时前置检查重新验证同一 Proposal/账户/规则/来源绑定、钱包 claim 容量、BNB 引用与新鲜度。任何实质变化或不可用输入都必须在 provider mutation 前拒绝；不得静默切换分支或复用旧估算。
- 结算以券商报告的 `commissionAsset` 和实际扣费为准。币种不同、费用缺失或扣费超过已保存额度时，保持冻结并进入对账；不能截断或追认。
- 不更改 Alpaca source-use、FX rights、账户权限、Arm、consent expiry 或其他独立门禁。用户本地/非商业用途以及 Support 未回复的状态保持不变。

此方案只取消“成交前必须知道唯一实际费用币种”的要求；不会把当前模型变成保证费用上限。用户的批准仅在此功能owning plan实现范围内，取代2026-10-12等待成交前唯一收费资产的S29.11决定，不重写RevC基线文档，也不放宽其他硬限或独立门禁。

## 建议验收与测试接口

使用最高层 Rust Control Plane 集成测试接口，并以临时 SQLite 存储和公开命令运行。只允许合成外部 HTTP、vault、WebSocket 和 clock 提供方；不得种入 reservation、approval、金融证据或 `VERIFIED` rights。

1. 通过普通公开账户配置、Proposal、证据刷新、复核、审批与 Prepare 命令，使用完全符合资格的条件计划创建第一个活动 reservation，但不 dispatch 订单。
2. 对同一账户创建第二个 Proposal 并执行认证账户读取，断言精确 BASE/QUOTE/BNB free/locked 行、原始首次回执/digest、完整绑定及逐币本地 claims。断言第一个 reservation 只扣减一次；已释放及其他账户的 reservation 不计入。
3. 断言缺失、重复、陈旧、绑定不符、额度不足或已变化的证据会撤销当前 proof 或原子拒绝 Prepare，且不产生部分审批消费、reservation 或 provider mutation。
4. 重开工作区，验证捕获证据保留原始回执，而当前资格仍需新的账户读取。
5. 单独验证响应式复核文案展示条件分支和待定实际费用币种，但不暗示费用金额受保证。

正向 source-use 与行情数据权利必须来自真实适用的数据源契约；测试专用报告不得声称用户已收到尚未收到的 Alpaca Support 回复。若没有合规公开路径能满足这些独立门禁，则该测试方案不能宣称完整验收 AC5/AC10。

## 决策状态

用户已于2026-10-15批准：“允许条件分支（推荐）：当前估算可进入 review/Prepare；实际币种成交后确认，偏差冻结核对”。该决定允许上述有界当前估算在所有独立门禁均通过时使review和Prepare具备资格；结算仍以券商实际记录为准，偏差冻结并进入对账。AC5仍豁免/未验收，AC6仍待实现证据，AC1来源使用依据仍未解决。此决定不修改RevC基线文档、issue勾选、source-use状态或无关实时金融门禁。

## 官方支持材料复核 — 2026-10-15

用户提供了一张 Binance Support 截图。2026-10-15 已打开并核对对应官方页面：

- [How to Calculate Binance Spot Trading Fees?](https://www.binance.com/en-NG/support/faq/detail/e85d6e703b874674840122196b89780a)，更新于2026-01-06。页面说明一般现货交易从收到的资产扣费、介绍BNB折扣，并说明成交页面的预估仅供参考，实际费用取决于已成交交易收到的资产数量。
- [How to Use BNB to Pay for Fees and Earn up to 25% Discount?](https://www.binance.com/en/support/faq/detail/115000583311)，更新于2026-02-06。页面要求开启BNB抵扣且现货账户有足够BNB；BNB不足时退回原费用。页面还说明下单时实际成交数量未知、页面仍显示收到资产的估算、BNB换算使用成交时平均汇率，且部分资产可能不符合BNB折扣条件。

这些官方页面确定了一般条件性规则，也限定了普通情形的模型分支；但它们不能确定未来成交的唯一实际币种/金额，不能保证账户BNB设置或余额一直保持到执行时，也不是账户专属协议，更不能验证任何Alpaca/来源使用权。它们支持方案中的`UNKNOWN_UNTIL_FILL`结算结果和条件性当前估算；仅凭这些页面并未自动采纳当前owning plan拒绝门禁的变更。

用户已于2026-10-15决定采用上述S29.11有界条件契约。验收勾选仍须等待实现证据通过。AC5第二次读取流程已豁免；AC6公开生产者到Prepare证据仍待完成，且不得伪造source-use权限。未修改RevC基线文档或source-use状态。

## 已核验 Binance Spot API 收费资产证据 — 2026-10-15

仓库引用的固定版本[Binance Spot 官方佣金 FAQ](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/faqs/commission_faq.md)明确给出收费资产分支：关闭BNB支付时，standard、tax、special三族费用合计后从收到的资产扣除；账户和标的均启用折扣且BNB余额充足时，这三族费用会换算为BNB，其中折扣只减少standard，不减少tax或special；BNB不足时，全部费用从收到的资产扣除。固定版本的[Spot REST 契约](https://github.com/binance/binance-spot-api-docs/blob/263ac1aa96556af0b4da824c82a1d8cd9a0edda9/rest-api.md#account-trade-list-user_data)说明，`GET /api/v3/myTrades`会为每笔已实现成交返回`commission`和`commissionAsset`。

这份 Binance 一手 API 证据确认了成交前估算可覆盖的通用分支，支持已批准的条件费用估算；它不能提前确定未来某笔成交的唯一币种或金额。实际`commissionAsset`须以成交后交易记录为准。继续保留`UNKNOWN_UNTIL_FILL`、条件估算和偏差冻结，不修改任何 RevC 基线文档。用户已要求跳过的数据使用依据仍未解决；在所有独立门禁都有有效证据之前，公开正向 Prepare 与 AC6 仍未验收。
