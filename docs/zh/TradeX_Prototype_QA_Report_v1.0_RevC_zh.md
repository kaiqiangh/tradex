# TradeX Prototype QA 报告 — v1.0 RevC

**修订日期：** 2026-09-05\
**证据基线：** 审查时 main@6c4b267 的 docs/prototype；本次仅修改文档，HTML/CSS/JS 未变\
**范围：** 文档一致性、源码、独立原型交互与有限浏览器检查

## 1. 结论

**原型交互/交付门槛：NOT PASS。** 2026-09-04 的整体 source-alignment PASS 和“适合直接交接”结论被本报告取代。目标规则已在中英文 PRD、前后端 ARD、UI Spec 中补齐；当前可点击原型仍有已复现缺陷，不能把规范修订当作原型修复。

当前状态为 9 个 FAILED、3 个 PARTIAL 场景。它们是回归场景计数，不是 FR/AC 覆盖率。规范可用于指导修复；原型交付须通过下面的关键场景以及独立 QA 门槛。

## 2. 证据方法与限制

- 阅读英文规范及中文对应段落，检查需求/页面 ID 集合与状态术语。
- 浏览器操作验证撤单分流、未知订单、人工处置、历史快照、焦点和 768 px 布局。
- 在隔离 Node VM 中执行当前 app.js 的相关状态函数，以模拟 DOM/定时器检查 CLOSED、UNTRUSTED、sleep、Disable All、proposal 身份、Backtest/模型失败路由；无真实提供方请求。
- 源码检查包含最后生效的函数覆盖，而非只读取旧实现。
- 上述浏览器/VM 结果是本次审查记录，不是已提交的自动回归套件；未归档完整截图。修复后必须重新执行并保存新证据。
- JavaScript 语法、文档链接、双语标识/状态配对、manifest 字节哈希由本轮本地检查确认；语法或 ID 存在不能证明交互通过。

## 3. 原型回归场景

每个场景列出实际观察与修复后的验收条件。下列源码行号对应未修改的原型基线；文档行号会随修订变化。目标细节见 [UI Spec §14](./TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md#14-开发与原型修复交互契约)。

<a id="qa-01"></a>

### QA-01 — Arming 后保留撤单意图

**状态：FAILED** · **方法：** 浏览器 + 源码 · **追溯：** FR-049; AC-033

- 操作：Accounts → Trading 212 Live DISARMED → Cancel remaining → Arm。
- 预期/修复后验收：返回含精确券商订单身份与剩余数量的撤单审批，再进入 CANCEL_PENDING 和券商确认结果；同时检查拒绝及审批前成交。
- 实际：打开 Approve live order / Approve & Place，订单字段出现 undefined。requestCancel 保存了 CANCEL，但 confirmArmLive 只按标的分支。
- 证据：[app.js:817,836](../prototype/app.js)；UI Spec §14.3–14.4。

<a id="qa-02"></a>

### QA-02 — 未知订单时间线

**状态：FAILED** · **方法：** 浏览器 + 源码 · **追溯：** FR-061; AC-021, AC-022, AC-034

- 操作：Order activity → Ambiguous submission + Manual Resolution；检查时间线和券商身份。
- 预期/修复后验收：UNKNOWN_RECONCILING 只显示已观察事件并采用不确定样式；不捏造确认/成交。逐个检查规范状态的显式呈现分支。
- 实际：未知状态默认展开至 FILLED 并使用绿色状态样式，而券商身份仍为 Pending。
- 证据：[app.js:718–731](../prototype/app.js)；UI Spec §14.5。

<a id="qa-03"></a>

### QA-03 — 基于证据的人工处置

**状态：FAILED** · **方法：** 浏览器 + 源码 · **追溯：** FR-023, FR-026, FR-076; AC-061

- 操作：分别重置场景后打开 Manual Resolution，选择 Confirmed submitted 和 Confirmed not submitted。
- 预期/修复后验收：加载可查后端证据，阻断缺失/陈旧/冲突证据。已提交要求核验券商身份，未提交要求充分证明。关闭/继续对账保留容量；有效处置后仍需健康校验和显式 arming。
- 实际：确认按钮直接修改状态；已提交直接设为 ACCEPTED 并显示固定身份，未提交无需证据步骤便清除 reservation.active。
- 证据：[app.js:793,829,920–926](../prototype/app.js)；UI Spec §14.5。

<a id="qa-04"></a>

### QA-04 — 执行资格与审批过期

**状态：FAILED** · **方法：** 隔离状态转换 + 源码 · **追溯：** FR-020, FR-021, FR-022, FR-024, FR-038, FR-062, FR-077; AC-015, AC-016, AC-049, AC-063

- 操作：设市场 CLOSED 或 timeHealth UNTRUSTED；请求 Live 订单 → Arm → Approve；检查审批过期入口和提交回调。
- 预期/修复后验收：审批和派发都阻断不支持订单及不可信时间下的权限。补测 HALTED、禁止权限、禁用市价单、陈旧行情/FX、策略变化、派发前过期和未知传输后过期；释放按 PRD §45。
- 实际：CLOSED 和 UNTRUSTED 均可进入 RESERVED；approveLive 只检查 arming。存在审批过期 helper，但未证明当前 UI 有完整入口/TTL 转换；所列其他拒绝场景需修复后执行回归。
- 证据：[app.js:817,823,835](../prototype/app.js)；UI Spec §14.3–14.4。

<a id="qa-05"></a>

### QA-05 — 全局 disarm 与派发中断

**状态：FAILED** · **方法：** 隔离状态转换 + 源码 · **追溯：** FR-027, FR-028, FR-051, FR-058; AC-014, AC-025, AC-037, AC-042, AC-044

- 操作：Arm Trading 212 和 Binance 后模拟 sleep；另一个场景将订单停在 RESERVED，Disable All 后推进提交回调。
- 预期/修复后验收：系统恢复 disarm 全部账户。派发前 Disable 阻止 I/O 并按条件释放；可能已传输后保留容量并对账。检查共享策略削弱对全部绑定账户的影响。
- 实际：Sleep 仅 disarm selectedAccount，另一个账户仍 armed；Disable All 后 RESERVED 订单仍进入 SUBMITTING。共享策略多账户行为仍需完整回归。
- 证据：[app.js:819,823,832,837](../prototype/app.js)；UI Spec §14.3。

<a id="qa-06"></a>

### QA-06 — 不可变历史溯源

**状态：FAILED** · **方法：** 浏览器 + 源码 · **追溯：** FR-002, FR-029, FR-055, FR-072, FR-075; AC-026, AC-058, AC-060

- 操作：打开已完成 US tech Research；不 Send，仅选择 DeepSeek，再切换到 Backtest；检查已有结果和 Artifact 溯源。
- 预期/修复后验收：历史模式/账户/模型/提供方/上下文保持不变；选择器只影响下一 Turn。Provider attempts 追加历史；恢复其他线程保持对应身份。
- 实际：已有 Immutable turn provenance 随当前模型/模式/账户选择器变化，Backtest 还清除所选账户。
- 证据：[app.js:651,767–768,811–813](../prototype/app.js)；UI Spec §14.1。

<a id="qa-07"></a>

### QA-07 — 草稿编辑与 proposal 重新生成

**状态：FAILED** · **方法：** 隔离身份检查 + 源码 · **追溯：** FR-019, FR-080; AC-032, AC-065

- 操作：生成 AAPL proposal 身份；变更数量或策略版本后重新生成；检查可编辑草稿入口。
- 预期/修复后验收：生成不同修订 ID/hash；旧 proposal 不变、旧审批失效、显式重新审批，并提供可用草稿编辑器。
- 实际：身份按标的固定，重新生成复用身份；可编辑 OrderDraft → Generate Proposal 交互不完整。
- 证据：[app.js:561–564,833](../prototype/app.js)；UI Spec §14.2。

<a id="qa-08"></a>

### QA-08 — 筛选器、上下文与标的选择

**状态：FAILED** · **方法：** 源码 · **追溯：** FR-009, FR-035, FR-046, FR-053; AC-039, AC-040

- 操作：编辑结构化筛选条件并 Run；改变 Context Picker 勾选项；从 Markets 打开 NVDA 或 AMD。
- 预期/修复后验收：展示输入驱动模拟结果，或明确提示未支持；保留精确选中上下文/标的身份。
- 实际：FilterSpec 静态且 Run 忽略输入；Attach 固定提示 AAPL + Trading 212 Live；非 BTC 标的映射到 AAPL。
- 证据：[app.js:366,398,403,406](../prototype/app.js)；UI Spec §14.1, 14.6。

<a id="qa-09"></a>

### QA-09 — 回测入口与冻结运行输入

**状态：PARTIAL** · **方法：** 隔离路由检查 + 源码 · **追溯：** FR-031, FR-054, FR-074; AC-048, AC-059

- 操作：选择 Backtest 模式并 Send；与策略编辑器运行入口比较。
- 预期/修复后验收：两个入口汇入已验证/冻结的回测配置和独立运行生命周期；不执行当前市场券商订单。
- 实际：存在回测结果/指标界面，但 Send 将全部非 Ask 模式导向 startResearch；完整配置 → 运行 → 取消/失败 → 比较行为尚未成立。
- 证据：[app.js:811,856](../prototype/app.js)；UI Spec §14.7。

<a id="qa-10"></a>

### QA-10 — 模型设置与修复

**状态：PARTIAL** · **方法：** 隔离可用性检查 + 源码 · **追溯：** FR-064, FR-068, FR-069, FR-070, FR-071; AC-038, AC-050, AC-055, AC-056, AC-057

- 操作：模拟 MODEL_UNAVAILABLE，关闭错误后 Send；检查 OAuth/配额恢复与设置控件。
- 预期/修复后验收：关闭错误不恢复 Send；设置/probe/重新登录/key 验证及显式重试/切换流程完整。Fallback 仍需 opt-in；控制面恢复保持可用。
- 实际：错误状态可与新 running research Turn 并存；修复控件不完整，标签不能证明 sidecar 启动、凭据验证或进行中尝试的重试策略。
- 证据：[app.js:797,847,856](../prototype/app.js)；UI Spec §14.8。

<a id="qa-11"></a>

### QA-11 — 键盘与弹窗焦点

**状态：FAILED** · **方法：** 浏览器 + 源码 · **追溯：** AC-052

- 操作：打开 Live 审批，按 Tab/Shift+Tab 和 Escape，并检查表格行语义。
- 预期/修复后验收：安全初始焦点、焦点限制、背景 inert、焦点恢复、表格行键盘可达，且不隐式 Enter 批准。
- 实际：对话框打开时 Tab 到达背景 + New Thread。存在弹窗 ARIA，但缺少焦点管理；可点击行依赖 onclick。读屏与完整 Enter 路径验证仍待执行。
- 证据：[app.js:118–119,737–739,859–865](../prototype/app.js)；UI Spec §14.9。

<a id="qa-12"></a>

### QA-12 — 窄屏导航与操作保留

**状态：PARTIAL** · **方法：** 768 px 浏览器检查 + 源码 · **追溯：** FR-004, FR-045; AC-039, AC-054

- 操作：在 768 px 检查紧凑导航；审查侧栏/历史和 Provider 动作 CSS；修复后补充 390 px 压力检查。
- 预期/修复后验收：新建/历史 Thread、次级页面、Provider 配置及全部 Live 控件可达，对话框/动作不被遮盖。
- 实际：侧栏/历史被隐藏，More 没有替代 Thread 操作；CSS 隐藏 Provider 按钮。尚未执行 390 px 和完整响应式回归。
- 证据：[styles.css:19,23](../prototype/styles.css), [app.js:798](../prototype/app.js)；UI Spec §14.9。

## 4. 文档决策与验证

| 项目 | 文档修订 | 验证边界 |
|---|---|---|
| 审批/订单过期 | PRD §18/§45 明确提交前释放、可能提交后保留及券商终态调整 | 运行时事务/崩溃测试待实现 |
| Gateway 隔离 | Backend ARD §5/§24 定义独立进程、私有通道、派发/撤销边界 | 进程/凭据隔离待运行时验证 |
| 前后端协议 | Backend ARD §41–42 为权威契约，Frontend ARD §10 引用 | 生成类型/兼容性测试待实现 |
| 双语语义 | 修正中文 OAuth 自动回退旧表述；新增契约完整配对 | ID 配对不替代逐段语义审查 |
| 文档入口 | RevC 权威顺序、ARD 配对、prototype 相对路径及 manifest 已更新 | manifest 描述当前文件，不代表发布认证 |

## 5. 独立且尚未完成的 QA 门槛

1. 修复后重跑 QA-01–QA-12，保存源码版本、初始状态、操作、预期/实际和截图/状态断言证据。
2. 完整桌面/768 px/390 px 视觉回归、长内容/滚动/空错误态，以及屏幕阅读器与全部键盘路径。
3. 真实 Codex App Server、CLIProxyAPI、IPC schema/顺序/重放兼容性，以及模型故障下控制面独立性。
4. Gateway 进程认证/凭据隔离；disarm 与派发、审批过期与提交、策略变更与消费、人工处置与成交的竞态。
5. SQLite 事务、预留恰好一次调整、真实重启/崩溃/休眠恢复，以及未知提交期间容量不被释放。
6. 提供方 sandbox/demo/testnet 的查询完整性、能力差异、订单身份、撤单成交竞态与证据规则。

只有对应门槛实际通过，才可更新其结论。原型通过也不等于生产交易系统已通过验收。
