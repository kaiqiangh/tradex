# S33 70 屏现有证据总表

此表逐屏整理已有记录，没有重新执行 70 屏。69 屏有有限本地记录；K1 物理睡眠、唤醒按用户指示跳过。BOUNDED_RECORD 不是屏幕或 S33 验收通过；状态标签不构成运行证明。

来源快照：`8e9803c` 的 `surfaces.csv` 与双语 S33 报告。每项保留原状态；详细测试时间、fixture 边界、来源 SHA 和剩余变体以引用记录为准。[机器总表](evidence/s33/screen-inventory.json)。

## 所有行共同保留的门禁

最终 SHA 的代表状态及适用变体核对、原生 VoiceOver、完整弹窗与 Live 控件清单，以及当前桌面产物/Gateway pin 核验仍未完成。未把浏览器 ARIA 检查当作 VoiceOver。未改变任何规范、追踪状态或 issue 验收勾选。

## 逐屏清单

| ID / screen | 原追踪状态 | 已记录代表状态 | 证据 | 其余范围代码 |
|---|---|---|---|---|
| A1 Workspace | VERIFIED | 三种宽度的命名设置字段 | [记录](s33-ui-regression-evidence_zh.md); [primary-pages-result.json](evidence/s33/primary-pages-result.json) | VARIANTS |
| A2 Broker / Data Providers | IN_PROGRESS | 被动 provider 选择与只读说明 | [记录](s33-ui-regression-evidence_zh.md); [primary-pages-result.json](evidence/s33/primary-pages-result.json) | VARIANTS, PROVIDER |
| A3 Provider Connection / Permission Review | IN_PROGRESS | 四家 schema、UNVERIFIED 确认、提款权限阻止及重测 | [记录](s33-ui-regression-evidence_zh.md); [a3-provider-permission-result.json](evidence/s33/a3-provider-permission-result.json) | PERMISSIONS, PROVIDER |
| A4 LLM Providers (Model) | IN_PROGRESS | 模型卡片与 fallback OFF；合成路由推进引导 | [记录](s33-ui-regression-evidence_zh.md); [primary-pages-result.json](evidence/s33/primary-pages-result.json) | MODEL |
| A5 Risk Defaults | IMPLEMENTED_UNVERIFIED | 默认风控字段、硬规则与显式保存 | [记录](s33-ui-regression-evidence_zh.md); [primary-pages-result.json](evidence/s33/primary-pages-result.json) | VARIANTS |
| A6 Ready | IMPLEMENTED_UNVERIFIED | 合成就绪摘要与显式完成 | [记录](s33-ui-regression-evidence_zh.md); [primary-pages-result.json](evidence/s33/primary-pages-result.json) | MODEL, PROVIDER |
| B1 New Thread | IMPLEMENTED_UNVERIFIED | 建议提示选择后显式创建空 Thread | [记录](s33-ui-regression-evidence_zh.md); [b1-suggested-prompts-result.json](evidence/s33/b1-suggested-prompts-result.json); [qa12-narrow-navigation-result.json](evidence/s33/qa12-narrow-navigation-result.json) | VARIANTS, MODEL |
| B2 Thread History / Resume | IMPLEMENTED_UNVERIFIED | Today/Earlier 分组与准确只读恢复 | [记录](s33-ui-regression-evidence_zh.md); [b2-thread-history-result.json](evidence/s33/b2-thread-history-result.json); [qa12-narrow-navigation-result.json](evidence/s33/qa12-narrow-navigation-result.json) | HISTORY |
| B3 Ask Mode | IMPLEMENTED_UNVERIFIED | Ask 只读上下文与显式模式转换 | [记录](s33-ui-regression-evidence_zh.md); [b3-ask-mode-result.json](evidence/s33/b3-ask-mode-result.json); [s33-b3-ask-mode-playwright.js](../../tests/s33-b3-ask-mode-playwright.js) | MODEL, VARIANTS |
| B4 Composer Pickers | IMPLEMENTED_UNVERIFIED | 规范标的、上下文、账户与模型选择器 | [记录](s33-ui-regression-evidence_zh.md); [b4-composer-pickers-result.json](evidence/s33/b4-composer-pickers-result.json); [s33-qa06-immutable-turn-provenance-playwright.js](../../tests/s33-qa06-immutable-turn-provenance-playwright.js) | PROVIDER, VARIANTS |
| B5 Research Running | IMPLEMENTED_UNVERIFIED | 合成运行计划、工具流与取消 | [记录](s33-ui-regression-evidence_zh.md); [b5-research-running-result.json](evidence/s33/b5-research-running-result.json); [b5-thread-status-announcement-result.json](evidence/s33/b5-thread-status-announcement-result.json) | MODEL, VARIANTS |
| B6 Tool / Turn States | IMPLEMENTED_UNVERIFIED | 合成工具失败与 Turn 状态投影 | [记录](s33-ui-regression-evidence_zh.md); [b6-tool-turn-states-result.json](evidence/s33/b6-tool-turn-states-result.json); [s33-b6-tool-turn-states-playwright.js](../../tests/s33-b6-tool-turn-states-playwright.js) | MODEL, VARIANTS |
| B7 Research Result — Equities | IMPLEMENTED_UNVERIFIED | 合成股票 Research/Trade 结果卡 | [记录](s33-ui-regression-evidence_zh.md); [b7-b8-research-result-cards-result.json](evidence/s33/b7-b8-research-result-cards-result.json); [s33-research-result-cards-playwright.js](../../tests/s33-research-result-cards-playwright.js) | MODEL, PROVIDER |
| B8 Research Result — Crypto | IMPLEMENTED_UNVERIFIED | 合成 Crypto 场所 Research/Trade 结果卡 | [记录](s33-ui-regression-evidence_zh.md); [b7-b8-research-result-cards-result.json](evidence/s33/b7-b8-research-result-cards-result.json); [s33-research-result-cards-playwright.js](../../tests/s33-research-result-cards-playwright.js) | MODEL, PROVIDER |
| B9 Immutable Turn Provenance | IMPLEMENTED_UNVERIFIED | 后续 Turn 修改不改变已捕获 Turn/Artifact 来源 | [记录](s33-ui-regression-evidence_zh.md); [qa06-immutable-provenance-result.json](evidence/s33/qa06-immutable-provenance-result.json) | MODEL, PROVIDER |
| C1 Market Explorer | IMPLEMENTED_UNVERIFIED | 股票、Crypto、Screeners 目录与只读选择 | [记录](s33-ui-regression-evidence_zh.md); [c1-market-explorer-result.json](evidence/s33/c1-market-explorer-result.json) | PROVIDER, VARIANTS |
| C2 Natural-language Screener Builder | IMPLEMENTED_UNVERIFIED | 类型化解析、编辑、运行筛选器 | [记录](s33-ui-regression-evidence_zh.md); [c2-screener-builder-result.json](evidence/s33/c2-screener-builder-result.json) | MODEL, VARIANTS |
| C3 Screener Result | IMPLEMENTED_UNVERIFIED | 完成、空结果与失败状态播报 | [记录](s33-ui-regression-evidence_zh.md); [c3-screener-result-result.json](evidence/s33/c3-screener-result-result.json); [s33-screener-status-announcements-playwright.js](../../tests/s33-screener-status-announcements-playwright.js) | ANNOUNCEMENTS, VARIANTS |
| C4 Equity Instrument Detail | IMPLEMENTED_UNVERIFIED | 股票详情五视图、规范身份与数据不可用 | [记录](s33-ui-regression-evidence_zh.md); [c4-equity-instrument-detail-result.json](evidence/s33/c4-equity-instrument-detail-result.json) | PROVIDER, VARIANTS |
| C5 Crypto Instrument Detail | IMPLEMENTED_UNVERIFIED | 仅 Spot 详情与维护 fixture | [记录](s33-ui-regression-evidence_zh.md); [c5-crypto-instrument-detail-result.json](evidence/s33/c5-crypto-instrument-detail-result.json) | PROVIDER, VARIANTS |
| C6 Market Session / Corporate Actions | IMPLEMENTED_UNVERIFIED | 未可信 OPEN fixture 与已记录公司行动类型 | [记录](s33-ui-regression-evidence_zh.md); [c6-market-session-corporate-actions-result.json](evidence/s33/c6-market-session-corporate-actions-result.json) | PROVIDER, VARIANTS |
| D1 Watchlist Library | IMPLEMENTED_UNVERIFIED | 持久化 Watchlist 库与键盘选择 | [记录](s33-ui-regression-evidence_zh.md); [d1-watchlist-library-result.json](evidence/s33/d1-watchlist-library-result.json) | VARIANTS |
| D2 Watchlist Detail | IMPLEMENTED_UNVERIFIED | 准确的已保存列表、版本与成员详情 | [记录](s33-ui-regression-evidence_zh.md); [d2-watchlist-detail-result.json](evidence/s33/d2-watchlist-detail-result.json) | VARIANTS |
| D3 New Watchlist | IMPLEMENTED_UNVERIFIED | 空名称拒绝与显式本地创建 | [记录](s33-ui-regression-evidence_zh.md); [d3-new-watchlist-result.json](evidence/s33/d3-new-watchlist-result.json) | VARIANTS |
| D4 Add Instrument to Watchlist | IMPLEMENTED_UNVERIFIED | 规范 AAPL 搜索与显式添加 | [记录](s33-ui-regression-evidence_zh.md); [d4-add-instrument-to-watchlist-result.json](evidence/s33/d4-add-instrument-to-watchlist-result.json) | VARIANTS |
| E1 Accounts Overview | IN_PROGRESS | Local Paper 概览与模拟说明 | [记录](s33-ui-regression-evidence_zh.md); [e1-accounts-overview-result.json](evidence/s33/e1-accounts-overview-result.json); [primary-pages-result.json](evidence/s33/primary-pages-result.json) | PROVIDER, VARIANTS |
| E2 Provider-specific Account Detail | IN_PROGRESS | Local Paper 详情及合成启动、认证、私有流、限流状态 | [记录](s33-ui-regression-evidence_zh.md); [e2-local-paper-detail-result.json](evidence/s33/e2-local-paper-detail-result.json); [k2-startup-reconciliation-result.json](evidence/s33/k2-startup-reconciliation-result.json) | PROVIDER, VARIANTS |
| E3 Portfolio | IMPLEMENTED_UNVERIFIED | 本地模拟组合与 USD 身份路径 | [记录](s33-ui-regression-evidence_zh.md); [e3-local-paper-portfolio-result.json](evidence/s33/e3-local-paper-portfolio-result.json); [portfolio-ui.mjs](../../tests/portfolio-ui.mjs) | PROVIDER, VARIANTS |
| E4 Open Order | IMPLEMENTED_UNVERIFIED | 准确关联或未关联的部分成交订单与预留上下文 | [记录](s33-ui-regression-evidence_zh.md); [s33-open-order-linked-proposal-playwright.js](../../tests/s33-open-order-linked-proposal-playwright.js); [s33-open-order-provider-variants-playwright.js](../../tests/s33-open-order-provider-variants-playwright.js) | PROVIDER, LIFECYCLE, RECEIPT |
| F1 Account-scoped Live Arming | IMPLEMENTED_UNVERIFIED | 合成 Arm 弹窗身份与健康门禁拒绝 | [记录](s33-ui-regression-evidence_zh.md); [qa11-native-account-dialog-result.json](evidence/s33/qa11-native-account-dialog-result.json); [s33-account-delete-dialog-playwright.js](../../tests/s33-account-delete-dialog-playwright.js) | PROVIDER, PHYSICAL, VARIANTS |
| F2 Limit Order Approval — Trading 212 / AAPL | IMPLEMENTED_UNVERIFIED | 不可变合成 AAPL Limit 审批复核 | [记录](s33-ui-regression-evidence_zh.md); [qa11-live-approval-focus-result.json](evidence/s33/qa11-live-approval-focus-result.json); [s33-live-approval-focus-playwright.js](../../tests/s33-live-approval-focus-playwright.js) | PROVIDER, VARIANTS |
| F3 AAPL Submission / Monitoring | IMPLEMENTED_UNVERIFIED | 显式合成 PLACE 接受、活动预留与重载 | [记录](s33-ui-regression-evidence_zh.md); [s33-live-order-submission-monitoring-playwright.js](../../tests/s33-live-order-submission-monitoring-playwright.js) | LIFECYCLE, PROVIDER, RECEIPT |
| F4 Market Order Approval — Binance / BTC | IMPLEMENTED_UNVERIFIED | 合成 BTC 市价支出超出不可变上限，审批禁用 | [记录](s33-ui-regression-evidence_zh.md); [qa04-eligibility-expiry-result.json](evidence/s33/qa04-eligibility-expiry-result.json); [s33-market-order-approval-playwright.js](../../tests/s33-market-order-approval-playwright.js) | SPOT, PROVIDER |
| F5 Approval Invalidated — Market Changed / Stale | IMPLEMENTED_UNVERIFIED | 行情刷新使已签发同意失效 | [记录](s33-ui-regression-evidence_zh.md); [qa04-eligibility-expiry-result.json](evidence/s33/qa04-eligibility-expiry-result.json); [s33-market-refresh-invalidates-approval-playwright.js](../../tests/s33-market-refresh-invalidates-approval-playwright.js) | VARIANTS, PROVIDER |
| F6 Approval Expired | IMPLEMENTED_UNVERIFIED | 短期审批过期、本地拒绝与历史审计 | [记录](s33-ui-regression-evidence_zh.md); [f6-approval-expiry-result.json](evidence/s33/f6-approval-expiry-result.json); [f6-post-dispatch-expiry-result.json](evidence/s33/f6-post-dispatch-expiry-result.json) | RESTART, PROVIDER |
| F7 Risk Rejected | IMPLEMENTED_UNVERIFIED | 持久化策略拒绝且无提交副作用 | [记录](s33-ui-regression-evidence_zh.md); [f7-risk-decision-result.json](evidence/s33/f7-risk-decision-result.json); [s33-risk-decision-submit-blocks-playwright.js](../../tests/s33-risk-decision-submit-blocks-playwright.js) | VARIANTS |
| F8 Broker Rejected | IMPLEMENTED_UNVERIFIED | 本地 fake Gateway 拒绝与容量释放 | [记录](s33-ui-regression-evidence_zh.md); [s33-broker-rejected-screen-playwright.js](../../tests/s33-broker-rejected-screen-playwright.js) | LIFECYCLE, PROVIDER, RECEIPT |
| F9 Ambiguous Submission / Manual Resolution | IMPLEMENTED_UNVERIFIED | 未知提交、冻结容量与有限证据解决 | [记录](s33-ui-regression-evidence_zh.md); [qa02-unknown-order-timeline-result.json](evidence/s33/qa02-unknown-order-timeline-result.json); [s33-ambiguous-submission-screen-playwright.js](../../tests/s33-ambiguous-submission-screen-playwright.js) | RESOLUTION, PROVIDER |
| F10 Live Cancellation | IMPLEMENTED_UNVERIFIED | 合成 CANCEL Arm、审批、Prepare、待确认及成交竞态 | [记录](s33-ui-regression-evidence_zh.md); [qa01-cancel-through-arming-result.json](evidence/s33/qa01-cancel-through-arming-result.json); [s33-qa01-cancel-through-arming-playwright.js](../../tests/s33-qa01-cancel-through-arming-playwright.js) | LIFECYCLE, PROVIDER |
| F11 Reservation Conflict | IMPLEMENTED_UNVERIFIED | 容量不足、陈旧 UI 与原子 Prepare 竞态拒绝 | [记录](s33-ui-regression-evidence_zh.md); [f11-capacity-conflict-result.json](evidence/s33/f11-capacity-conflict-result.json) | RESTART, LIFECYCLE, PROVIDER |
| F12 Risk-policy Change Invalidation | IMPLEMENTED_UNVERIFIED | 工作区策略放宽使绑定 Proposal 失效 | [记录](s33-ui-regression-evidence_zh.md); [f12-risk-policy-invalidation-result.json](evidence/s33/f12-risk-policy-invalidation-result.json) | VARIANTS, PROVIDER |
| G1 Local Paper | IMPLEMENTED_UNVERIFIED | 本地完整及部分模拟成交与安全提交、撤单弹窗 | [记录](s33-ui-regression-evidence_zh.md); [s33-local-paper-playwright.js](../../tests/s33-local-paper-playwright.js) | LIFECYCLE, VARIANTS, RECEIPT |
| G2 Alpaca Paper | IMPLEMENTED_UNVERIFIED | 合成 ACK、重载、重测、待确认撤单、私有流、去重、断开 | [记录](s33-ui-regression-evidence_zh.md); [g2-alpaca-paper-result.json](evidence/s33/g2-alpaca-paper-result.json) | LIFECYCLE, PROVIDER |
| G3 Trading 212 Demo | IMPLEMENTED_UNVERIFIED | 合成 Demo ACK、部分成交、历史与待确认撤单 | [记录](s33-ui-regression-evidence_zh.md); [g3-trading212-demo-result.json](evidence/s33/g3-trading212-demo-result.json) | LIFECYCLE, PROVIDER |
| G4 Binance Testnet | IMPLEMENTED_UNVERIFIED | 准确 Testnet 复核；风控证据不足不创建 attempt | [记录](s33-ui-regression-evidence_zh.md); [g4-binance-testnet-result.json](evidence/s33/g4-binance-testnet-result.json) | LIFECYCLE, PROVIDER |
| G5 Bitget Demo | IMPLEMENTED_UNVERIFIED | 准确 Demo 复核；风控证据不足不创建 attempt | [记录](s33-ui-regression-evidence_zh.md); [g5-bitget-demo-result.json](evidence/s33/g5-bitget-demo-result.json) | LIFECYCLE, PROVIDER |
| H1 Strategy List | IMPLEMENTED_UNVERIFIED | 策略列表与准确选中版本 | [记录](s33-ui-regression-evidence_zh.md); [h1-strategy-list-result.json](evidence/s33/h1-strategy-list-result.json); [s33-strategy-screen-inventory-playwright.js](../../tests/s33-strategy-screen-inventory-playwright.js) | VARIANTS |
| H2 Editable Strategy Sandbox | IMPLEMENTED_UNVERIFIED | 参数校验与不可变策略版本 | [记录](s33-ui-regression-evidence_zh.md); [h2-strategy-sandbox-result.json](evidence/s33/h2-strategy-sandbox-result.json); [s33-strategy-sandbox-playwright.js](../../tests/s33-strategy-sandbox-playwright.js) | VARIANTS |
| H3 Backtest Running | IMPLEMENTED_UNVERIFIED | 回测运行、冻结输入身份与进度 | [记录](s33-ui-regression-evidence_zh.md); [h3-backtest-running-result.json](evidence/s33/h3-backtest-running-result.json); [s33-backtest-running-screen-playwright.js](../../tests/s33-backtest-running-screen-playwright.js) | VARIANTS, PROVIDER |
| H4 Backtest Failed | IMPLEMENTED_UNVERIFIED | 类型化失败运行与保留冻结输入 | [记录](s33-ui-regression-evidence_zh.md); [h4-backtest-failed-result.json](evidence/s33/h4-backtest-failed-result.json); [s33-backtest-failed-screen-playwright.js](../../tests/s33-backtest-failed-screen-playwright.js) | VARIANTS, PROVIDER |
| H5 Backtest Result | IMPLEMENTED_UNVERIFIED | 合成完成态指标、曲线、交易及完整 manifest | [记录](s33-ui-regression-evidence_zh.md); [h5-backtest-result-result.json](evidence/s33/h5-backtest-result-result.json); [s33-backtest-result-screen-playwright.js](../../tests/s33-backtest-result-screen-playwright.js) | PROVIDER, VARIANTS |
| H6 Backtest Compare | IMPLEMENTED_UNVERIFIED | 两个已完成合成运行及准确 manifest 链接 | [记录](s33-ui-regression-evidence_zh.md); [h6-backtest-compare-result.json](evidence/s33/h6-backtest-compare-result.json); [s33-backtest-compare-screen-playwright.js](../../tests/s33-backtest-compare-screen-playwright.js) | PROVIDER, VARIANTS |
| I1 Artifact Library | IMPLEMENTED_UNVERIFIED | 空及已填充 Artifact 库与显式保存 | [记录](s33-ui-regression-evidence_zh.md); [i1-artifact-library-result.json](evidence/s33/i1-artifact-library-result.json); [s33-artifact-library-screen-playwright.js](../../tests/s33-artifact-library-screen-playwright.js) | VARIANTS |
| I2 Artifact Detail | IMPLEMENTED_UNVERIFIED | 准确合成内容、hash 与不可变详情 | [记录](s33-ui-regression-evidence_zh.md); [i2-artifact-detail-result.json](evidence/s33/i2-artifact-detail-result.json); [s33-artifact-detail-screen-playwright.js](../../tests/s33-artifact-detail-screen-playwright.js) | PROVIDER, VARIANTS |
| I3 Provenance Drawer / Modal | IMPLEMENTED_UNVERIFIED | 来源身份、安全焦点、循环、关闭及返回 | [记录](s33-ui-regression-evidence_zh.md); [i3-artifact-provenance-dialog-result.json](evidence/s33/i3-artifact-provenance-dialog-result.json); [s33-artifact-provenance-dialog-playwright.js](../../tests/s33-artifact-provenance-dialog-playwright.js) | PROVIDER, VARIANTS |
| I4 Export | IMPLEMENTED_UNVERIFIED | 显式本地 JSON 导出与响应字节核对 | [记录](s33-ui-regression-evidence_zh.md); [i4-artifact-export-result.json](evidence/s33/i4-artifact-export-result.json); [s33-artifact-export-screen-playwright.js](../../tests/s33-artifact-export-screen-playwright.js) | VARIANTS |
| J1 Providers & Models | IN_PROGRESS | 被动 provider/model 页面与有限网关故障投影 | [记录](s33-ui-regression-evidence_zh.md); [j1-passive-provider-settings-result.json](evidence/s33/j1-passive-provider-settings-result.json) | MODEL, PROVIDER |
| J2 Risk & Limits | IMPLEMENTED_UNVERIFIED | 命名风控字段及工作区、账户策略流程 | [记录](s33-ui-regression-evidence_zh.md); [j2-risk-limits-result.json](evidence/s33/j2-risk-limits-result.json); [s33-risk-limits-screen-playwright.js](../../tests/s33-risk-limits-screen-playwright.js) | VARIANTS |
| J3 Data & Storage | IMPLEMENTED_UNVERIFIED | 存储职责及不可用、上下文说明 | [记录](s33-ui-regression-evidence_zh.md); [j3-data-storage-result.json](evidence/s33/j3-data-storage-result.json); [s33-data-storage-screen-playwright.js](../../tests/s33-data-storage-screen-playwright.js) | VARIANTS |
| J4 Import / Restore Workspace | IMPLEMENTED_UNVERIFIED | 浏览器导入弹窗、合成预览与选择 shim | [记录](s33-ui-regression-evidence_zh.md); [j4-workspace-import-result.json](evidence/s33/j4-workspace-import-result.json); [s33-workspace-import-dialog-playwright.js](../../tests/s33-workspace-import-dialog-playwright.js) | IMPORT |
| J5 Account Health | IN_PROGRESS | 独立合成账户健康维度 | [记录](s33-ui-regression-evidence_zh.md); [j5-account-health-result.json](evidence/s33/j5-account-health-result.json); [s33-account-health-screen-playwright.js](../../tests/s33-account-health-screen-playwright.js) | PROVIDER, VARIANTS |
| J6 Appearance | IN_PROGRESS | 当前 System/Light 与披露的未来 Dark 选项 | [记录](s33-ui-regression-evidence_zh.md); [j6-appearance-result.json](evidence/s33/j6-appearance-result.json); [s33-appearance-screen-playwright.js](../../tests/s33-appearance-screen-playwright.js) | VARIANTS |
| J7 About | IN_PROGRESS | 版本、运行时与 provider 目录只读信息 | [记录](s33-ui-regression-evidence_zh.md); [j7-about-result.json](evidence/s33/j7-about-result.json); [s33-about-screen-playwright.js](../../tests/s33-about-screen-playwright.js) | VARIANTS |
| K1 Resume after Sleep | NOT_STARTED | 没有物理睡眠、唤醒运行证据；按用户指示跳过 | [记录](s33-ui-regression-evidence_zh.md) | PHYSICAL |
| K2 Startup Reconciliation | IN_PROGRESS | 启动陈旧、未检查与未武装投影 | [记录](s33-ui-regression-evidence_zh.md); [k2-startup-reconciliation-result.json](evidence/s33/k2-startup-reconciliation-result.json); [s33-startup-reconciliation-screen-playwright.js](../../tests/s33-startup-reconciliation-screen-playwright.js) | RESTART, PROVIDER |
| K3 Authentication Failure | IN_PROGRESS | 合成认证失败与补救操作 | [记录](s33-ui-regression-evidence_zh.md); [k3-authentication-failure-result.json](evidence/s33/k3-authentication-failure-result.json); [k7-error-surface-result.json](evidence/s33/k7-error-surface-result.json) | PROVIDER, VARIANTS |
| K4 Private Stream Disconnect | IN_PROGRESS | 合成私有流降级与账户级阻止 | [记录](s33-ui-regression-evidence_zh.md); [k4-private-stream-disconnect-result.json](evidence/s33/k4-private-stream-disconnect-result.json); [s33-private-stream-disconnect-playwright.js](../../tests/s33-private-stream-disconnect-playwright.js) | PROVIDER, LIFECYCLE |
| K5 Rate Limiting | IMPLEMENTED_UNVERIFIED | 浏览器注入类型化限流与退避说明 | [记录](s33-ui-regression-evidence_zh.md); [k5-rate-limiting-result.json](evidence/s33/k5-rate-limiting-result.json); [s33-rate-limiting-playwright.js](../../tests/s33-rate-limiting-playwright.js) | RATE_LIMIT, PROVIDER |
| K6 Clock / Freshness Failure | IMPLEMENTED_UNVERIFIED | 未可信时钟阻止武装与审批 | [记录](s33-ui-regression-evidence_zh.md); [k6-clock-uncertain-result.json](evidence/s33/k6-clock-uncertain-result.json); [s33-untrusted-time-approval-playwright.js](../../tests/s33-untrusted-time-approval-playwright.js) | PHYSICAL, PROVIDER |
| K7 Canonical Error Surface Variants | IMPLEMENTED_UNVERIFIED | 二十种规范错误渲染、单测及部分浏览器变体 | [记录](s33-ui-regression-evidence_zh.md); [k7-error-surface-result.json](evidence/s33/k7-error-surface-result.json); [error-recovery.test.ts](../../tests/error-recovery.test.ts) | ERROR_BROWSER, VARIANTS |

## 范围代码

- `VARIANTS`: 仍需逐例核对适用的加载、空、陈旧、失败、阻止、禁用、完成及中断、竞态变体。
- `PROVIDER`: 真实 provider 真值及集成仍独立跳过、开放。
- `MODEL`: 合成、被动记录未建立真实模型、网关登录、恢复或推理验收。
- `PERMISSIONS`: 转账、托管、不支持的保证金或杠杆权限变体及原生凭据输入仍开放。
- `HISTORY`: 已填充 Saved、归档历史及加载、失败、重试仍开放。
- `ANNOUNCEMENTS`: 更广的 live-region 转换与流式噪声检查仍开放。
- `PHYSICAL`: 物理睡眠、唤醒、时钟证据仍跳过、开放；合成启动不能替代。
- `LIFECYCLE`: 完整终态、成交、结算、账户仓位刷新及中断、对账链仍开放。
- `SPOT`: 正向具备资格的 owning Spot 审批、Prepare 仍归 #135 与独立金融门禁。
- `RESTART`: 完整进程重启恢复与派发中断仍开放。
- `RESOLUTION`: 完整 provider 证据与充分缺失规则仍不可用、开放。
- `IMPORT`: 此浏览器 shim 不证明原生选择器、真实桌面入口或恢复发布。
- `RATE_LIMIT`: 浏览器注入不证明真实 producer Retry-After、冷却时序、退避或对账调度。
- `ERROR_BROWSER`: 尚未在浏览器中逐一运行全部 20 种规范错误。
- `RECEIPT`: 此索引没有该屏专用的当前运行收据；保留有限文字、脚本记录，最终回放需补收据。

## 下一步顺序

1. 历史快照之后的补充：八种已知 Binance 单 flag 权限响应在 `c768206` 通过（审查记录 `f25c6fc`）；B2 读取传输加载、失败及重试在 `cc16bd1` 通过。详见双语 S33 报告及机器收据。这不刷新其余 70 屏快照行。
2. 通过公开生命周期契约解决 B2 非空 Saved 缺口：ARCHIVED 有 wire/展示分支，但没有公开归档操作。保留真实存储故障、native 及真实 provider 缺口。
3. 按界面顺序逐项核对其余适用状态与交互；完成当前项验证、串行审查与记录后再推进。
4. 随后进行最终 SHA 集成回放、原生辅助功能及交付产物核验；本地证据不能关闭按用户指示跳过的真实/物理验收。
