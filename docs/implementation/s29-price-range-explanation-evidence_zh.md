# S29.7 Proposal 价格区间执行规则解释

[English](s29-price-range-explanation-evidence.md)。[解释 Proposal 价格区间执行范围](https://github.com/kaiqiangh/tradex/issues/128)：来源实现 VERIFIED；本记录时远端/跟踪器交付仍待完成。源码提交 `622e34c9e2d6c344943a0b3aca761b85cf959c1d`，固定复审基线 `ec10e99`（规划交付点；规划提交不改动源码）。全部160份最终源码/构建输入哈希与已提交字节一致。

不可变的普通 Binance Live BTC/USDT 或 ETH/USDT Proposal 现在通过既有的当前/捕获/pre-arm 规则视图解释其真实的 PRICE_RANGE 执行规则。官方文档允许的逐方向可选乘数被如实表示而非被拒绝，因此部分配置的规则按原样呈现，不会被臆造为完整。已呈现字段保持精确十进制字符串（包括实际上报的零），而重复字段、畸形小数、超出文档整数范围的值与未知的生效字段仍失败关闭。只有真实的执行参考价 `referencePrice` 能为该用途定性：不存在订单簿、成交或均价的回退，且读取未刷新的视图本身不产生任何提供方请求。精确的所选侧边界以 QUOTE 每 BASE 报出；已证明的规则缺失与提供方显式 null 都作为「已验证不实施」的事实呈现，绝不作为执行资格。过期或失效的参考价被退役而非被续期，只保留其溯源信息；捕获时记录边界摘要，使已保存的复核在价格被剥离后仍能指认它所依据的边界。任何可用预览都保持执行未获资格：此处不批准、不提交、不承诺成交，成交方在订单进入 taker 阶段时仍会重算参考价，越界执行会使订单失效，且所有未决义务持续可见。

最终验证通过487项 Rust（0失败，39项既有 ignored 单独保留）跨34个测试二进制，其中89项公开规则用例包含4项新增 PRICE_RANGE 用例；另有17项 Node 通过。IPC 生成校验（`schema:check`）、普通前端构建与203需求/70页面/13 QA 场景的可追溯性回读通过。独立串行的 Standards → Spec 复审均 PASS，0剩余来源发现；Spec 轴发现的两项正确性缺陷已在最终运行前通过公开边界 RED/GREEN 修复。[最终检查](evidence/s29-price-range-explanation/checks-review-final.txt)、[计数](evidence/s29-price-range-explanation/verification-counts.json)、[独立复审](evidence/s29-price-range-explanation/code-review-final.md)、[最终 UI](evidence/s29-price-range-explanation/ui-review-final-report.json)。

实现之前有真实的公开 RED：三项新增 PRICE_RANGE 用例在规划交付点的回退切片状态下失败，退役用例随后针对过期参考价缺陷补入。[RED](evidence/s29-price-range-explanation/red.txt)、[GREEN](evidence/s29-price-range-explanation/green.txt)、[首次规则全档运行](evidence/s29-price-range-explanation/ordinary-rust-initial.txt)、[中间态全量检查](evidence/s29-price-range-explanation/checks-intermediate.txt)。实际 React 验收的当前/捕获/pre-arm、部分/空/null/快照状态、冻结捕获、剥离价格，以及键盘1280/768/390 无溢出检查均通过，控制台0警告/0错误；6处 harness 配置与断言错误单独记为仪器问题，既非产品 RED 也非最终失败。[UI 报告](evidence/s29-price-range-explanation/ui-review.json)、[1280](evidence/s29-price-range-explanation/ui-1280.png)、[390](evidence/s29-price-range-explanation/ui-390.png)、[setup 说明](evidence/s29-price-range-explanation/ui-harness-setup-notes.txt)。

仅在已批准的公开接缝上把外部 HTTP/vault 生产者置为假；未使用任何正向金融 authority setter、真实凭据或提供方请求、订单变更、Arm、consent、原生启动或物理休眠证据。自有的 dev server 与浏览器标签页已停止，视口已复位。仓库无 CI 工作流，故不运行也不声明远端 CI。完整 S29 父项、S28、S17、物理 S27、S33 与最终 main 门禁保持 OPEN。全局需求与页面状态不提高；嵌套门禁不变：映射到 S29 的只有 `FR-017`（IN_PROGRESS）与 `UX-004`（IMPLEMENTED_UNVERIFIED），二者在 S33 与 OD-005/OD-006 授权前都不会推进。原型代码未改；远端 main 未动。

验收映射与冻结的未声明事项见 [acceptance-before-handoff.json](evidence/s29-price-range-explanation/acceptance-before-handoff.json) 与 [delivery-before-handoff.json](evidence/s29-price-range-explanation/delivery-before-handoff.json)。


最终交付：正常 dev 推送已核对 `7f4a3016f4e5d7e2c06396776907913dfe50f97c`；7项AC已核对关闭，票于 `2026-10-09T20:31:53Z` 关闭。[关闭结论](https://github.com/kaiqiangh/tradex/issues/128#issuecomment-6088765600)、[最终验收映射](evidence/s29-price-range-explanation/acceptance-final.json)、[交付回读](evidence/s29-price-range-explanation/delivery-final.json)。父项与 Wayfinder 指针已回读；剩余8个开放 issue。此前的待完成表述是明确的关闭前记录，并非当前状态。本元数据跟进不改动160份源码输入。
