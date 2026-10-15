# S29.10 精确标的佣金来源验收

[English](s29-symbol-commission-evidence.md)。用户已确认实现/复审固定基线 `90745562fe19d1ceac4490a39311c306befc660e`。源码 `a3127a51ace0f8f71bfa8648f25b6a1f6306bd1f`、配对契约 `ba82e59d06000f991bd2ddefc9e36236bfde507c`。限定来源最终验收PASS；实现133/规范132均CLOSED，各8项AC全部勾选并回读。已核对dev源码/证据交付 `3d507c0d79c8a35fa0106382c0dce4c3e5eee80e` 与远端dev一致。

显式读取复用现有凭据与固定普通主机，先验证账户UID，再读取已保存规范标的声明。保留standard/tax/special三类maker/taker/buyer/seller共十二项原始费率字符串，以及两项折扣标志、币种和原始值。说明BUY收到BASE/SELL收到QUOTE及条件BNB/回退分支。来源有效性与金额费用/扣费币种和所需执行FX资格独立，不推导金额费用或换汇。未知活跃条款保持未解决；get/capture不更新原始回执，变化或过期来源使当前证据退役，历史捕获不变。

| 实现/规范AC | 证据 |
|---|---|
| 1 | 公开精确账户normal/UID/symbol/method/signature用例；固定普通signed GET白名单 |
| 2 | Rust/shared schema/client类型；schema检查与实际React解码 |
| 3 | 无效折扣RED/GREEN；缺失/numeric/duplicate/unknown/过界key公开用例 |
| 4 | BUY BASE/SELL QUOTE；启用/禁用/不支持币种及未解决余额/换汇公开说明 |
| 5 | 公开provider/auth/rate失败、迟到来源generation回调、不新增HTTP的过期、既有deadline/budget门禁 |
| 6 | 真实来源变化的review digest/capture RED/GREEN；无approval/Prepare状态；实际关闭/重开历史 |
| 7 | 最终实际React populated当前/捕获/pre-arm与键盘1280/768/390、console及截图 |
| 8 | 冻结完整检查、普通build/pin、串行独立源码复审、精确committed/remote/tracker交付 |

[TDD记录](evidence/s29-symbol-commission/tdd-current.json)区分三次真实公开后端RED/GREEN、一次实际UI循环及四组首次回归。首次IPC构建使用错误bin名称属于工具调用错误，不算产品RED。没有来源/authority setter，仅在已批准公开接缝改变外部HTTP/vault/WS生产者响应。

下方最终检查/构建/UI记录绑定已提交源码，精确关闭回读已在下方核对。[Standards→Spec](evidence/s29-symbol-commission/review-final.md)独立源码复审PASS，0遗留问题；复审者仅阅读源码，没有运行测试、构建、UI或修改tracker。

父121/map1/完整金融/native/provider/物理/release门禁保持OPEN。AC-035仍NOT_STARTED，需求/屏幕状态未升级。没有真实provider请求、order/test-order POST、Arm、consent、reservation或dispatch。原型源码与main未改。完整金额费用/扣费条件与真正必需USDT/BNB执行FX仍必须由后续拥有方实现。当前无CI workflow，本地检查不冒充CI证据。

[最终完整检查](evidence/s29-symbol-commission/checks-review-final.json)：八项exit0、172冻结输入未变；517Rust通过/0失败/39既有忽略、34结果套件，17Node、49BinanceHot、19StockHot、23Gateway，以及schema/frontend、203需求/70屏幕/13QA/23文件追溯。普通build/pin及冻结实际UI已通过，精确交付已在下方核对。

[普通桌面构建](evidence/s29-symbol-commission/desktop-build-review-final-inputs.json)exit0；desktop SHA681ac530…、恢复的普通Gateway SHA257f637c…、内嵌pin匹配，172输入未变。这是编译证据，不是native/真实provider验收。[最终实际React/Rust UI](evidence/s29-symbol-commission/ui-review-final-report.json)通过populated当前/捕获/pre-arm keyboard1280/768/390全部九组无溢出测量，历史不变，console无warning/error，无Arm/consent。截图已目视检查。integration IPC使用runtime Gateway pin，不适用普通桌面的embedded pin契约。工具编排/重复场景quota失败与独立最终PASS分开保留；没有修改源码或额度守卫。自建tab5/6及1428进程均关闭，viewport恢复，端口无监听。

[远端/提交字节交付](evidence/s29-symbol-commission/delivery-before-close.json)核对全部172已提交输入及57不可变证据文件、原生121→132→133/已完成123+131/0开放阻塞，原型/main未变。[Tracker关闭回读](evidence/s29-symbol-commission/tracker-closure-readback.json)确认132/133各8AC全勾选且CLOSED，父121/map1仍OPEN。[实现决议](https://github.com/kaiqiangh/tradex/issues/133#issuecomment-6098801386)、[规范决议](https://github.com/kaiqiangh/tradex/issues/132#issuecomment-6098802377)。原始stdout/stderr保留空白及字节哈希；代码/非日志空白检查通过。早期checkpoint为历史快照，不覆盖当前关闭回读。后续拥有方必须补齐完整金额费用上界/条件扣费与真正必需USDT/BNB执行FX，再进行preflight/私有生命周期/真实金融验收。
