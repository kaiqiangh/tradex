# S29.11 owning reservation revalidation — local implementation evidence

2026-10-19. Baseline `78d7450`, branch `dev`. Continue the existing [complete Spot financial-plan ticket](https://github.com/kaiqiangh/tradex/issues/135); do not create another owning ticket. The current map and all fifteen AC states were read through the configured GitHub connector after `gh issue view` returned `Forbidden`. Existing source, fee-model and MARKET composition decisions remain approved.

## Requirements and sequential implementation

1. Verify the current native build blocker. GLib/GTK/WebKit development metadata is absent; `apt-get --assume-no install` exits 100 because the agent cannot acquire the root-owned dpkg lock, and `sudo` is absent. Native verification remains blocked.
2. Extend the existing public external-producer → risk → approval → Prepare scenario through the trusted dispatch-grant and immediate preflight interfaces. The exact attempt must reuse its own durable principal/BNB vector without double counting, while retaining every claim. Preserve approval expiry and immutable review evidence. No provider POST or real dispatch is allowed.
3. Verify changed financial evidence refuses preflight and preserves the documented invalidation/release semantics. Fix only behavior demonstrated by a failing public regression. Existing Backend §41.57, owning-plan decisions 9/11 and AC6/11/12 govern; no new wire authority or financial policy is proposed.
4. Run affected and complete checks, then independent Standards and Spec review in that order. Record exact outcomes and commit to `dev`; full source/provider/native/physical/Wayfinder acceptance remains open.

Use the standing approved seam: typed Control Plane with disposable SQLite and external HTTP/vault/WS/clock fakes. Read-only ledger checks may additionally prove absence of partial state; no financial authority rows are seeded. Preserve any initial semantic RED before the minimum production fix. No new seam or ticket-granularity decision is needed.

## Implemented correction

The first public producer→approval→Prepare→dispatch-grant regression [failed](evidence/s29-owning-dispatch-revalidation/first-red.txt). Temporary diagnostics identified an unavailable required policy-valuation route: hashing the entire stored Proposal made atomic consumption's status/version/history changes retire its unchanged FX observation. The diagnostics were removed.

`FxRequirements.materialVersion` now excludes only Proposal lifecycle `status`, `stateVersion`, `history` and `invalidationReason`. Every remaining Proposal field, workspace/base and complete account-version-bound route vector stays bound. The paired Backend §41.57 and owning-plan spec state this financial identity rule. No schema or renderer changes are needed. Source generation, observation age/session/clock and durable approval/attempt/reservation checks remain intact.

Three focused public regressions [pass](evidence/s29-owning-dispatch-revalidation/focused.txt): (1) the unchanged exact-account, two-asset reservation passes dispatch-grant and immediate Control Plane preflight without changing claims, review digest or approval expiry; a BNB balance of exactly `0.0006120102` proves its own fee claim is not charged twice, and grant replay cannot start another attempt; (2) a genuine current quote change after the grant refuses and releases once; (3) source reconfiguration retires the old FX observation and refuses preflight. Prepare's lifecycle transition leaves the complete public FX requirements projection unchanged.

The BNB literal is independent of production arithmetic: `0.001 × 60001 × 0.001 × 0.1 ÷ 500 = 0.0000120002` discounted standard plus `0.001 × 60001 × 0.005 ÷ 500 = 0.00060001` tax; special is zero. Both family values are already exact at the approved scale.

These calls exercise trusted Control Plane interfaces, not a Binance Gateway package, child process, authenticated provider preflight or real provider submission. Ordinary Binance PLACE Gateway support remains outside this correction; no provider support check was changed. The synthetic Alpaca Support context is not actual entitlement/account evidence. All provider POST collections stay empty. Full AC6/10/12, actual source activation, settlement/native/physical/release and map acceptance remain open.

## Final verification and handoff — 2026-10-20

| Check | Result / receipt |
| --- | --- |
| Complete default check | `npm run check` [exit0](evidence/s29-owning-dispatch-revalidation/full-check-status.json), exact source `46b1ec9`: [full log](evidence/s29-owning-dispatch-revalidation/full-check.txt),591 reported Rust passes,33 existing ignored,38 frontend passes; schema, production frontend build, secret/log and203/70/13/23 traceability checks pass. |
| Binance-specific regression | [67 serial isolated scenarios](evidence/s29-owning-dispatch-revalidation/hot-final.txt), [exit0](evidence/s29-owning-dispatch-revalidation/hot-status.json), including all three new public preflight regressions. |
| Actual React → Rust → SQLite | [Result](evidence/s29-owning-dispatch-revalidation/ui-result.json), current/captured/pre-arm at1280/768/390, six overflow assertions;104 observed commands,0 forbidden commands/page errors/console errors/external browser requests. Account stays disarmed with no issued approval. Six capture/prearm PNGs are actual viewport screenshots;390 capture and1280 pre-arm were visually inspected. |
| Gateway pin/handshake | [5 tests pass](evidence/s29-owning-dispatch-revalidation/gateway-regression.txt), [exit0](evidence/s29-owning-dispatch-revalidation/gateway-status.json). macOS-only child/provider tests are excluded on Linux and are not counted as verified. |
| Independent review | Serial [Standards](evidence/s29-owning-dispatch-revalidation/standards-review.md):0 hard violations,1 nonblocking maintenance advisory; then [Spec](evidence/s29-owning-dispatch-revalidation/spec-review.md):0 actionable correction defects. |
| Native desktop | [Environment blocker](evidence/s29-owning-dispatch-revalidation/environment-blocker.txt): missing GLib/GTK/WebKit metadata; package installation exit100, root-owned dpkg/no sudo. Native build remains unverified. |
| Frozen source | [264 build/test/application/contract inputs](evidence/s29-owning-dispatch-revalidation/source-inputs.json), exact source commit/tree; all matched source `46b1ec9` when frozen. |

The environment restarted between runs. The earlier completed check log is retained as `full-check-before-resume.txt`; the final full check was rerun against the exact source and saved its exit code independently of the terminal session. Rust1.98.1, offline Cargo, dev/test debug information disabled, one Rust test thread and16MiB test stack were used. Text receipts normalize trailing whitespace only. Existing ignored, platform-excluded and unexecuted real/native cases remain outside acceptance. No test timing or production quota was shortened. Existing compiler dead-code and Vite chunk-size warnings remain.

The attempted [tracker handoff](evidence/s29-owning-dispatch-revalidation/tracker-handoff.md) could not be published: `gh issue comment` [returned Forbidden](evidence/s29-owning-dispatch-revalidation/tracker-write.txt). The connector fallback is read-only, so no alternate tracker write was used and no AC state changed. Source and evidence remain on `dev`; no worktree or main PR was created. Full owning/parent/map acceptance remains open.

## 中文接续计划

基线 `78d7450`，分支 `dev`。继续同一完整 Spot 金融计划票；CLI 读取被拒绝后，已用配置的只读 GitHub 连接器核对地图与全部15项验收状态，不新建拥有方票。

依次执行：①记录原生构建依赖安装的权限阻塞，原生验证仍未完成；②从现有公开 producer→risk→approval→Prepare 场景继续验证受信 dispatch grant 与即时 preflight，只复用本 attempt 的持久本金/BNB 预留，保留全部 claims、原审批到期时间及不可变审阅证据；③验证金融证据变化时拒绝 preflight，并遵循既有失效/释放语义，仅修复公开回归证明的缺陷；④完成受影响与完整检查，串行 Standards→Spec 审查后提交 `dev`。

沿用已批准的 typed Control Plane、临时 SQLite 与外部 HTTP/vault/WS/clock 模拟边界，不植入金融权限行，不执行真实 provider POST/派发。既有 Backend §41.57、owning-plan 决策9/11及AC6/11/12为依据，不改变金融政策或线协议。完整真实来源、provider、原生、物理及Wayfinder验收继续开放。

## 中文实现记录

公开 producer→审批→Prepare→dispatch grant 回归先失败：系统把整个已保存Proposal计入FX需求摘要，原子消费新增的生命周期状态、版本及历史误使未变化的必需估值观察失效。临时诊断已删除。金融身份摘要现仅排除`status`、`stateVersion`、`history`和`invalidationReason`，其余Proposal字段、workspace/本位币与完整账户版本绑定的路线仍全部绑定；Backend §41.57及owning规范双语同步，无schema或renderer改动。

三项重点回归已通过：①完整外部模拟producer生成的两资产预留可通过受信grant与即时Control Plane preflight；BNB余额恰为独立算例`0.0006120102`时不重复扣除自身claim，预留、review digest及approval到期时间保持不变，grant不可重放；②grant后真实报价变化拒绝并仅释放一次；③来源重配退休旧FX观察并拒绝preflight。Prepare前后公开FX需求投影完全一致。

这些调用只验证受信Control Plane接口，不验证Binance Gateway package、子进程、provider认证预检或真实提交。未更改普通Binance PLACE支持门禁；合成Alpaca Support声明不构成真实账户/来源权限证据，provider POST记录全部为空。AC6/10/12完整验收、真实来源激活、结算、原生、物理、发布与整张map继续开放。

## 中文最终验证与交接——2026-10-20

最终`npm run check`绑定源码`46b1ec9`并以0退出：591项Rust通过、33项既有ignored、38项前端通过，schema/前端生产构建/secret与日志/203需求、70屏幕、13QA场景、23基线文件追踪通过。67项隔离Binance专项全部通过；实际React当前/捕获/pre-arm三种宽度、六项溢出检查通过，观察104条命令，无禁用命令/页面错误/控制台错误/外部浏览器请求。UI账户保持撤防且未签发审批。Gateway pin/握手5/5通过，macOS专属子进程/provider场景不在Linux验收范围。Standards先完成（0硬违规、1条非阻断建议），随后Spec完成（0项本修复缺陷）。264份源码/构建/测试/规范输入已冻结并核对准确source commit/tree。

环境重启后已针对准确源码重新运行完整检查并独立保存退出码，之前完整日志另留备份。使用Rust1.98.1、离线Cargo、dev/test关闭调试信息、单Rust测试线程及16MiB测试栈；文字收据仅规范行尾空白。ignored、平台排除及未执行的真实/原生用例不算验收，原有编译dead-code和Vite chunk-size警告保留。

原生依赖缺失且无root/sudo安装权限，原生构建仍未验证。`gh issue comment`以Forbidden拒绝发布，交接正文和失败收据已本地保留；连接器fallback仅允许读，因此未绕过tracker写入约定，也未修改任何AC状态。源码与证据均在`dev`，没有worktree或main PR，完整owning/parent/map验收继续开放。
