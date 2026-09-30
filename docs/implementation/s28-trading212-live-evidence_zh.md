# S28 Trading 212 Live 进度证据

日期：2026-09-30。分支：`dev`。审查基线：`5f33334e8885e66afae542a239f23341821708a7`。[English](s28-trading212-live-evidence.md)。

父级：[验证 Trading 212 Live 可信执行](https://github.com/kaiqiangh/tradex/issues/113)。单张实现票：[贯通市价审批与可信派发验证](https://github.com/kaiqiangh/tradex/issues/114)。

**单张实现票 LOCAL_ACCEPTANCE_PASS；S28 父级保持 OPEN。** 用户为调度显式跳过 S27 实际 OS 睡眠唤醒验证；这不表示通过，也不移除最终验收要求。见 [S27 审计](s27-parent-acceptance-audit_zh.md)。

下文早期检查点描述历史观察；恢复继续执行后，其 `/private/tmp` 产物已不存在。当前可持久复核的证据在最终基线部分；不把缺失临时文件声称为当前可检查证据。

## 已交付修正

Live Market DAY 不可变 Proposal 必须携带正数、精确十进制 `maximumSpend`。共享 Trading 212 请求构建器此前连有效、已消费的 Live intent 也拒绝该字段；现在校验并保留本地金融额度，只向提供方编码官方 ticker、带符号的 BASE 数量和 `extendedHours: false`。Demo 与 Limit 的拒绝规则不变。审批界面说明：本地授权/预留金额不等于券商保证执行的市价价格或金额上限。中英文 Backend §41.36、Frontend §13.22、UI §14.2 同步此边界。

2026-09-30 核对官方 [Trading 212 API](https://docs.trading212.com/api) 与 OpenAPI；OpenAPI SHA-256 为 `a272f70a713fa9f2f906e9b4be12136c9d5d63d7f9f48562e939aa35c4fc041d`。没有 `maximumSpend` 请求字段。Sell 编码为负数；Limit 使用 DAY/GOOD_TILL_CANCEL；acknowledgement 不代表成交或撤单完成。

## 初始共享请求检查

- 有意义的 RED：真实子进程在 provider I/O 前以 `ORDER_PROPOSAL_NOT_ELIGIBLE` 停止已批准、带额度的 Market intent；修复后同一正例通过。日志：`/private/tmp/tradex-s28-red-domain.log`、`tradex-s28-green.log`。更早的假服务器超时仅是测试观测局限，不能作为唯一根因证明。
- 真实子进程 Gateway 全套：**23/23 PASS**，覆盖一次准确 Market POST、缺失/非法额度在 provider I/O 前拒绝、SELL 精确小数、Limit DAY/GTC、接受/未知/拒绝、身份/grant 边界、撤单及不重放。日志：`/private/tmp/tradex-s28-gateway.log`。使用合成派发包和 loopback 提供方，不能证明公开 Market 审批可用。
- 明确的 Demo 保持回归：**1/1 PASS**。公开生成的 Demo Proposal 携带 maximumSpend 时仍被拒绝，POST 为零且没有持久化提交 attempt。日志：`/private/tmp/tradex-s28-demo-bound-check.log`。
- 最终 `RUST_TEST_THREADS=1 npm run check`：**PASS**，schema、TypeScript/Vite、16 个 Node 测试、328 个 Rust 测试、36 个既有显式 ignored 项不变，以及 203 条需求/70 个页面/13 个 QA 场景/23 个基线文件清单。日志：`/private/tmp/tradex-s28-serial-final-check.log`。前一并行运行在既有 S27 模型恢复 workspace-reopen 断言失败；同一测试单独通过，最终串行全套通过。不能将并行运行报告为通过。
- Rust-backed CUA `checkTrading212BoundedMarketUI`：**仅所述边界 PASS**。Market 审阅保留不可变账户/proposal/hash、maximumSpend 51 和明确标为合成的行情溯源；缺少可执行报价时审批禁用且无新增 POST。独立 Limit GTC 审阅的 Enter 不产生同意；显式 Space 批准后再 Prepare，预留 50 USD、消费准确审批，并通过真实子进程向假提供方仅发送一次准确 POST。UI 明确 ACCEPTED 不代表成交。Workspace 重开保留同一已接受 attempt 与 ACTIVE reservation，不重放。Market 和 Limit 审阅在 390/768/1280px 无横向溢出，结束后重置 viewport。
- 浏览器结果：`/private/tmp/tradex-s28-ui-results.json`；截图：`/private/tmp/tradex-s28-limit-result.png`。执行的 helper 副本与 `tests/live-approval-ui.mjs` 字节一致。Chrome debugger 失败和首次 IAB 点击选择失败是工具局限，不作为应用 PASS/FAIL。最终成功运行使用文档支持的 IAB API，以及键盘选择 proposal/draft。
- `npm run desktop:build` **PASS**：普通 `desktop` release 应用和独立 Gateway 编译成功（非签名/打包验收）。`cargo fmt --all --check`、`node --check`、`git diff --check` 和需求清单检查通过。Standards 独立审查 PASS / 0 发现；Spec 对整票完整性为 NOT PASS，仍有 P1 公开 Market 滑点链路缺失。后续原生证据已解决原 P2 读取记录发现；Live readiness 健康状态仍未验证。共享请求修正本身未发现 Spec 缺陷；两票保持 OPEN。不声称严格全仓 Clippy PASS。

## 剩余实现与外部 gate

初始共享请求检查时，**公开 Live Market 审批尚未完成**：政策要求滑点，evaluator 始终返回 `EXECUTION_QUOTE_UNAVAILABLE`，snapshot 缺少 BASE 深度，review 没有估算。初始 UI 观察到 Market fail-closed，并独立验证 Limit。这是实现缺口，不能只归因于尚无交易同意。下文后续修改增加显示深度计算并验证原公开 Market 正例；尚未增加普通生产报价 producer，也不免除此 gate。

`2026-09-30T09:38:52.826214Z` 只读原生 SQLite 快照保留真实 T212 sequence 86：保存的 health 为 ONLINE/VALID/CONFIGURED/CURRENT/BLOCKED/DISARMED，3 个 position、0 个 open order、6 个 recent order，最近成功同步为 `08:25:40.072837Z`。该快照时 TradeX 未运行，因此是已保存提供方证据，不是新增或当前新鲜读取。权限 scope 仍为 UNVERIFIED，仅检测到账户/持仓/订单读取。快照：`/private/tmp/tradex-s28-native-read-only.json`。

普通桌面生产行情、日历、可交易性、所需 FX 和安全权限证明仍不可用/未验证。未对真实账户 Arm，未发送券商 POST/DELETE，未输入凭据或修改安全权限。Fixture 不授予普通原生 readiness。真实提供方下单/撤单须先满足这些 gate，再由用户对具体交易接管确认。FR-016/AC-010 和整个 map 不标为 VERIFIED；S17 与 S27 实际 OS gate 保持独立。

## 后续原生读取

普通 `npm run desktop` 重建并启动当前工作树，无 integration fixture（PID 59663；原生 SHA-256 `bdf9f850b2f7b8b020c64ac1ccea21af38c2e848b88d422b8e1e3590411fc98b`）。启动 sequence 87 处于 stale，一秒采样发现 worker 等待 `NativeVault::get / SecItemCopyMatching`；随后用户确认完成系统认证。持久化 sequence 97 在 `2026-09-30T10:23:03.856845Z` 记录了**新增提供方读取成功**，成功同步时间为 `10:23:03.856359Z`，ONLINE/VALID/CONFIGURED，但 reconciliation 仍为 STALE，执行为 BLOCKED/DISARMED。`10:31:00.863709Z` 当前快照为 sequence 101，提供方限流后变为 ERROR/UNVERIFIED/CONFIGURED/STALE/BLOCKED/DISARMED；保留该新增同步、3 个 position、0 个 open order、6 个 recent order，权限 scope 仍为 UNVERIFIED。这证明新增读取发生，不证明当前 reconciliation 健康、安全交易权限或实际 OS 睡眠唤醒。本次验证没有发起重试风暴或提供方 mutation。脱敏快照/outbox：`/private/tmp/tradex-s28-native-current.json`；此前等待采样：`/private/tmp/tradex-s28-native.sample`。未修改凭据或 Keychain 访问控制。

## 后续显示深度实现

沿用已同意的最高命令/UI 测试边界。[Spec 补充](https://github.com/kaiqiangh/tradex/issues/113#issuecomment-5909500902) 与[单票补充](https://github.com/kaiqiangh/tradex/issues/114#issuecomment-5909501452) 定义可选精确 BASE bidSize/askSize、对应方向完整深度、同一报价中点基准及精确交叉相乘政策比较。Risk 与 ApprovalReview 共用 estimator；审批、原子 Prepare、dispatch 的不可变输入包含新深度。UI 展示深度并标明百分比只是近似估算，不保证成交。中英文 ARD/UI 契约与生成 IPC schema/types/validators 同步。生产报价 producer/entitlement 仍缺失，普通原生 readiness 继续阻止执行。

- 有意义的公开命令 RED：修正 fixture Arm 和时钟后仅剩滑点/RiskDecision blocker，日志 `/private/tmp/tradex-s28-slippage-red.log`；同一带额度 Market 审阅/审批/Prepare 随后通过，日志 `/private/tmp/tradex-s28-slippage-green.log`。
- Market 相关检查 **10/10 PASS**，含四个新增公开命令测试：maximum51 预留、非法/缺失/交叉/不足/不匹配深度、仍足够但发生变化的深度使审批/Prepare/dispatch 失效、精确政策边界0.002与0.00199999，以及 BUY ask/SELL bid 深度。日志 `/private/tmp/tradex-s28-slippage-related.log`。中间故障测试发现比较 helper 会接受非法深度；现在共用 estimator 先显式校验十进制并拒绝。阈值测试改为在当前政策下生成 proposal，避免复用已失效 proposal。
- 更新后的 CUA Market UI **PASS**：Enter 不授予批准，显式 Space 后公开审批→maximum51 ACTIVE reservation→一次准确的真实子进程假提供方 Market POST `{ticker:"AAPL_US_EQ",quantity:0.001,extendedHours:false}`→ACCEPTED（非成交）。重开保留同一已接受 attempt/reservation，无重放；390/768/1280px 审阅无横向溢出。结果 `/private/tmp/tradex-s28-market-ui-results.json`，截图 `/private/tmp/tradex-s28-market-result.png`。已执行 helper/source SHA-256 为 `948bbc6372dbd030fc8e76f7a2b0dd2b0e9d9f8fcc1c8af60796bc9d7fa19c84`。首次导航早于 Vite ready，失败；成功运行使用已 ready 服务器和同一选定 IAB 的新标签页。随后停止本次拥有的浏览器服务器及原生开发 launcher。
- 此检查点仍待更新后全套检查、桌面构建、串行 Standards→Spec 审查。上面的更早构建/测试数量与审查发现只适用于初始共享请求修正，不构成新 estimator 的最终验收。两票保持 OPEN；未做真实 broker mutation、增加 entitlement、自动 Arm，也不声称 OS-wake PASS。

更新后的串行 `RUST_TEST_THREADS=1 npm run check` **PASS**：schema/type/Vite、16 Node、332 Rust、36 个既有 ignored、203 需求/70 页面/13 QA/23 基线文件。日志 `/private/tmp/tradex-s28-market-final-check.log`。更新后的真实子进程 Gateway **23/23 PASS**，日志 `/private/tmp/tradex-s28-market-gateway.log`。格式、helper 语法与 diff 检查通过。桌面构建及新 code-review 仍待完成。

最后补充的溯源校验还拒绝空来源/快照 ID 和错误报价 venue。此修改后 Market 相关检查 **10/10 PASS**（`/private/tmp/tradex-s28-market-final-related.log`），串行全套检查 **PASS**，16 Node / 332 Rust / 36 个既有 ignored（`/private/tmp/tradex-s28-market-final-recheck.log`）。当前源码集成正例在 `2026-09-30T11:21:12.281Z` 再次通过，所执行 helper hash 相同。整页截图为 `/private/tmp/tradex-s28-market-result.png`，可读结果视图为 `/private/tmp/tradex-s28-market-viewport.png`；两张截图均已目视核对。已停止本次截图服务器并恢复临时 viewport。

最终普通 `npm run desktop:build` **PASS**（`/private/tmp/tradex-s28-market-desktop-final.log`）。Release 应用 SHA-256 为 `030ed582d47b661bd1e1d4e160012e3889b88ca4514868e71446e27535d82ca8`；Gateway SHA-256 为 `bb4fc0d9c4ddd2b957f83179dc2e2a75a0238a5d90c34f2d48e685dd4f358cca`。这证明编译通过，不构成签名包或真实提供方验收。独立串行 Standards→Spec 审查仍待完成。

## 最终证据基线

当前 `RUST_TEST_THREADS=1 npm run check` **PASS**：16 Node / 332 Rust / 36 个既有 ignored，生成 schema/types、TypeScript/Vite 与 203 需求 / 70 页面 / 13 QA / 23 基线文件检查通过。[完整检查输出](evidence/s28/final-check.txt)。当前真实子进程 Gateway **23/23 PASS**，包括 Market 额度、SELL 精度、Limit DAY/GTC、精确 CANCEL、配额/身份故障与无重放。[Gateway 输出](evidence/s28/gateway-check.txt)。

`2026-09-30T16:58:37.955556+00:00` 的只读 SQLite 保留状态观察为 sequence103：connection FAILED，STALE/UNVERIFIED/UNCHECKED/STALE/BLOCKED/DISARMED，原因 SESSION_RESUMED。保留先前观察到的成功同步 `10:23:03.856359Z`、3 个持仓 / 0 个未完成 / 6 个历史订单；scope 仍为 UNVERIFIED，仅有读取能力。[脱敏观察](evidence/s28/native-retained.json)。这是当前保留状态快照，不是新增提供方读取，也不证明 Live readiness 健康。

当前普通 `npm run desktop:build` **PASS**：[构建输出](evidence/s28/desktop-build.txt)、[产物及所执行 helper 哈希](evidence/s28/artifact-hashes.json)。重建应用与 Gateway 的哈希与历史最终构建一致；编译不代表签名包或真实提供方验收。

当前主要 Market UI **2/2 PASS**，观察于 `2026-09-30T17:19:41.709Z`：maximum50 小于 expected50.001 时，在390/768/1280px 显示可读拒绝原因、禁用审批，approvals/attempts/POSTs 均为零。Escape→修改 maximum51→Save v2→Generate proposal 保留旧的不可变 maximum50 intent，并生成新的 proposal。Enter 不批准，显式 Space 后批准带额度的 Market DAY intent，预留51、通过真实子进程向 loopback 发送一次准确 Market POST，重开保留 ACCEPTED/ACTIVE，无重放。0.002% 来自合成深度，不保证成交。[结果](evidence/s28/market-ui.json)、[可读截图](evidence/s28/market-result-view.png)、[整页截图](evidence/s28/market-result.png)。

当前共享审批 UI **15/15 PASS**：Arm 身份/并发、隐式 Enter 拒绝、显式审批、Disarm 失效、独立 Prepare、一次准确 POST、重开、390/768/1280px 下 F11 额度不足/过期、政策变化、未知/不重放、明确拒绝/释放，以及五分钟后仅 Keep 的对账。[结果](evidence/s28/full-ui.json)、[最终截图](evidence/s28/full-ui-result.png)。所执行 helper SHA-256 为 `5c278730bb4ebe47c01021188efede045de43a6ed14b4419d9342e42a7227671`。临时 viewport 已重置，临时标签页已关闭，本次拥有的服务器已停止。

针对更早的审查/测试发现，helper 已补充 Market 拒绝与修复路径；规范化跨 realm 的 decision 数组再作严格比较；将时钟过期场景放到最后，避免令后续新报价过期；通过公开导航/重挂载取得当前对账状态，避免浏览器5秒等待依赖产品11秒轮询。未放宽任何产品 guard、政策或时钟新鲜度检查；中间失败不报告为通过。独立串行 Standards→Spec 复审仍待完成。

## 本地验收与提交绑定

实现提交：`b0fec527c892e3a99d760db94526fdaa785efd06`。基于捕获的基线，独立串行 [Standards 审查](evidence/s28/standards-review.txt) **PASS / 0 发现**，以及 [Spec 审查](evidence/s28/spec-review.txt) **PASS / 0 发现**；各轴最高严重度均无。审查读取工作树源码与保留产物，没有独立复跑测试。此前 Spec P1 滑点与 P2 Market 修复路径发现均已解决。此本地子票满足结案条件；S28 父级保持 OPEN，生产/外部门禁全部保留。

全套检查、真实子进程 Gateway、普通桌面构建及两套 UI helper 均基于此 SHA 的源码执行；随后仅记录证据与状态。[验证清单](evidence/s28/verification-manifest.json) 绑定当前源码/helper/产物哈希。当前持久结果替代早期检查点的待完成描述；历史观察仍显式保留为历史。未声称 CI 或签名包 PASS。
