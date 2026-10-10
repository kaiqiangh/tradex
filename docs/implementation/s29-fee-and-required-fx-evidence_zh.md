# S29.9 费用与真正所需执行FX来源证据

[English](s29-fee-and-required-fx-evidence.md)。源码`7b99f8e`，配对契约`00d4f14`，固定实现/复审基线`1f8e231de3fe67aec16db1ae7392f34981617330`。规范#130与实现#131仍OPEN，待精确远端/跟踪器交付。限定来源/拒绝片的最终本地验收PASS。

本零新增读取来源片报告账户maker/taker/buyer/seller原始费率、精确maker/taker比较基准，仅意图策略及BUY出资路径、绑定版本及唯一拥有方复核阻塞。缺失佣金如实描述；未知extension义务保留，字段名有界。当前费用币种/出处UNKNOWN、费用/换算金额缺失，必需USDT→工作区基准路径不支持。存在的已交付只读方向FX在缺少执行质量/成本时仍UNQUALIFIED，恒等无需汇率。比较基准不是完整费用估算。完整symbol/side费用及计入币种与真正所需USDT执行FX两项后续都必须建立；拒绝不能作为完整goal豁免或提供方能力缺失声明。

| #131 AC | 证据 |
|---|---|
| 1 | Rust/shared契约、schema检查及实际UI解码 |
| 2,9 | 原投影及公开commission extension场景、名称边界RED/GREEN |
| 3 | 公开拥有方路由收窄、排除组合目的、仅BUY出资 |
| 4 | 公开费率词法不同等值/maker/taker比较，无float |
| 5,6 | 公开route/fee事实、唯一追加阻塞及不变activation predicate |
| 7 | 真实外部佣金变化摘要、拒绝无审批/部分状态；既有已签发审批的缺FX Prepare测试不改 |
| 8 | 防御推导纯契约场景，明确不是公开产品资格 |
| 10 | 实际当前/捕获/pre-arm UI及refresh/reopen不变捕获，最终冻结UI PASS |
| 11 | 配对契约/计划/map、独立串行复审，精确远端/跟踪器交付待完成 |

#130 AC1–7对应上表契约/投影/身份/路由/基准/阻塞/拒绝/摘要；AC8对应实际UI/捕获；AC9对应明确可达性；AC10对应最终精确交付。当前来源陈述不建立完整费用或执行FX资格。

[最终检查](evidence/s29-fee-and-required-fx/checks-review-final.json)：8项exit0，169冻结源码/构建输入与源码提交一致；510Rust通过/0失败/39既有ignored跨34结果套件、17Node、49BinanceHot、19StockHot、23Gateway，schema/前端与203需求/70屏幕/13QA/23文件追溯。追溯只验证清单，不证明运行行为。[独立串行复审](evidence/s29-fee-and-required-fx/code-review-final.md)：最终Standards与Spec均PASS，0剩余可执行/实质发现。初次Spec两项缺陷已修复，复审者未运行验证。

[TDD记录](evidence/s29-fee-and-required-fx/tdd-current.json)区分2公开后端RED/GREEN、1实际UI披露周期及1纯保留契约汇率周期。首次即绿的公开回归与继承T02摘要单列，不制造RED或公开正向。首次桌面构建为Spec修复中断，不是PASS；最终构建/pin与UI日志单独记录。

父121、map1、S28、S17、物理S27、S33及dev→main仍开放。AC-035仍NOT_STARTED，仅目标拥有方及PRD行指针改变。未执行真实provider写/Arm/consent/派发，不声明原生/物理验收。原型代码未改，main未变。

最终普通桌面构建exit0，嵌入Gateway pin与恢复的普通Gateway一致（`5f2799d6…`）。[构建/输入记录](evidence/s29-fee-and-required-fx/desktop-build-review-final-inputs.json)绑定全部169输入。[最终实际React/Rust UI](evidence/s29-fee-and-required-fx/ui-review-final-report.json)通过缺失/声明/变化/捕获/pre-arm披露、不变历史及1280/768/390宽度检查，无控制台错误，未执行Arm或consent。拥有方1427主机与tab已关闭，viewport已恢复。桌面构建是构建证据，不是原生/真实提供方验收。
