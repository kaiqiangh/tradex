# S12 研究产物保存、检索与可验证导出规范

日期：2026-09-19
前置：S04 Thread/Turn/Item 持久化与不可变快照、S10 source-gated typed research 结果
状态：VERIFIED（S12 垂直切片完成；S33 全量回归和真实 provider 边界仍待后续工作项）

## Problem Statement

TradeX 已经能够把研究结果作为带来源的 `research_result` timeline item 写入 Thread，但用户无法把一次经过审阅的结论保存为 workspace 产物，也无法从 Artifacts 导航重新打开、检查其不可变 Turn 快照或导出一份可复核的本地文件。当前 Artifacts 页面仍是空状态，历史产物若依赖当前 Composer 的账户、模式或模型选择器会产生错误归因。

## Solution

增加 workspace-scoped 的 Research/Decision artifact library。用户从一个已完成的研究 Item 显式保存产物，随后可以在 Artifacts 列表按时间查看、打开详情、展开 Provenance drawer/modal，并执行一次明确的本地导出。Artifact 保存的是内容的有界、脱敏投影和创建时的 immutable Turn snapshot；导出同时携带稳定 artifact/version/hash、Thread/Turn/Item、provider attempts、research tool/source provenance、market/dataset/order refs（存在时）和 manifest 校验信息，因此可以重新连接到来源而不会读取当前选择器。

所有读写都经过版本化 Rust IPC 和 SQLite projection；导出写入 workspace `exports/` 或用户选择的本地文件，不提供云端上传、分享链接或秘密归档。保存、查看和导出永远不改变账户、风险政策、arming、approval、reservation、Gateway 或凭据。

## User Stories

1. 作为研究用户，我想从已完成的研究 Item 显式保存一个带标题的产物，以便把可审阅结论留在 workspace 中。
2. 作为研究用户，我想看到当前 workspace 的 Artifact library，以便按标题、类型、创建时间和来源 Thread 区分历史结果。
3. 作为研究用户，我想打开 Artifact detail，以便查看保存时的摘要、结论、限制、来源和可验证标识。
4. 作为研究用户，我想查看 Thread、Turn、Item 和 immutable Turn snapshot，以便知道产物是在什么 Agent Mode、Execution Context、账户、模型和附加 context 下产生的。
5. 作为研究用户，我想查看所有 provider attempts，以便看到模型路由、切换或失败尝试，而不是只看到最终模型。
6. 作为研究用户，我想查看 research tool、source/provider 时间、新鲜度、质量和限制，以便判断证据是否仍可用。
7. 作为研究用户，我想查看 market snapshot、dataset/backtest manifest 和 related order refs（存在时），以便把研究结论连接到其数据或后续交易事实。
8. 作为研究用户，我想在关闭并重新打开 workspace 后继续访问同一 Artifact，以便历史检索不依赖临时 UI 状态。
9. 作为研究用户，我想导出单个 Artifact 到本地文件，以便在 TradeX 外部保存或审阅可验证副本。
10. 作为研究用户，我想让导出文件包含 manifest、artifact hash 和 provenance refs，以便发现内容或来源被改动。
11. 作为隐私审查者，我想确认导出和普通 Artifact 永不包含 broker credentials、model keys、Keychain bytes、Authorization header、raw provider payload 或原始敏感账户数据。
12. 作为隐私审查者，我想确认研究文本按不可信内容处理并经过边界脱敏，以便 prompt injection 不会变成金融命令或凭据。
13. 作为安全审查者，我想确认 artifact workspace、Thread、Turn 和 Item 关系在受信后端重验，以便伪造或跨 workspace 引用无法保存。
14. 作为并发用户，我想在 Artifact 被其他操作改变时收到 stale/version 错误，以便不会静默覆盖历史内容。
15. 作为用户，我想看到 unknown artifact、source unavailable、invalid export path、redaction failure 和 integrity failure 的明确状态与下一步操作。
16. 作为键盘用户，我想用语义按钮打开详情、展开 provenance、导出并返回列表，且焦点在 modal 关闭后恢复到调用控件。
17. 作为窄屏用户，我想在 390px、768px 和 1280px 窗口阅读长 provenance 和导出状态，不丢失 Artifacts 导航或主要操作。
18. 作为安全审查者，我想确认保存、读取和导出前后账户、risk、approval、arming、reservation、Gateway、outbox 与 credentials 投影不变。

## Implementation Decisions

- Artifact library 是 workspace-scoped SQLite projection；它不是金融 domain aggregate，不写金融 outbox，也不授予任何 capability。普通 Artifact 文件和导出文件位于现有 filesystem artifact/export 边界内。
- S12 只实现 `RESEARCH` 与 `DECISION` 两种研究产物来源。策略、回测、图表、报告批量导出、Workspace import/restore 和 retention policy 继续由后续工作项负责。
- 保存使用显式 `artifact.save`：请求带 workspace、Thread、Turn、Item、标题和 kind。Control Plane 重新加载 Thread，要求来源 Turn/Item 属于当前 workspace、已完成、Item 可序列化且存在 source-gated research result 或有界完成内容；不接受前端自造 provenance 或正文。
- 读取使用 `artifact.list` 与 `artifact.get`。列表只返回有界摘要和稳定 `artifactId`/`version`/`contentHash`；详情返回保存时的 content projection 与完整非秘密 provenance。结果不读取当前 Composer selectors，也不重新计算历史 capability。
- 导出使用 `artifact.export`，目标只能是 workspace `exports/` 下的生成文件名或由原生文件选择器确认的本地路径。导出结构为 UTF-8 JSON manifest + sanitized artifact payload；manifest 记录 schema version、artifact/version/content hash、exportedAt 和 provenance refs。不会提供 cloud/share-link 语义。
- Artifact provenance 至少包含 workspace、Thread、Turn、Item、immutable `TurnSnapshot`、provider attempts、research tool/result/source/provider timestamps、freshness/quality、market/dataset hashes、related order IDs（若来源明确提供）。模型/provider attempts 追加保存，不覆盖 Turn snapshot。
- Content projection 只保留 bounded textual/typed research fields（结论、findings、scenarios、evidence、limitations、instrument/artifact refs 和 marker）。敏感凭据、原始 provider response、Authorization headers、Keychain bytes、完整账户余额/订单 payload 和未验证的自由格式命令不会进入 projection 或 export。
- 为保持 Backend ARD §41–42 单一 wire contract，新增命令和 payload/result 类型必须同时加入 Rust protocol、JSON Schema、生成的 TypeScript validators/client definitions，并拒绝未知字段、跨 workspace ID、超长字符串/数组和不支持的 kind。
- Artifact identity 使用 opaque ID、单调版本和 canonical content hash；同一来源重复保存创建新的 artifact identity，历史记录不就地改写。读取/导出使用 workspace 与 artifact version 校验，失败不产生部分文件。
- 导出在目标目录的临时文件中完成后以不可覆盖的原子链接创建目标文件；路径越界、符号链接、无法写入、hash 不匹配或 redaction 失败均 fail closed 并清理临时文件。导出动作需要显式用户点击，不由页面加载或 Enter 隐式触发。
- UI 实现 Artifacts library、detail 和 ProvenanceModal，复用现有 AppShell、thread/context identity 和语义状态模式。详情中的 Thread/Turn/Item/context 全部使用保存快照；当前模式、账户、模型选择变化不影响历史展示。
- 最高验证 seam 为真实桌面 UI → 版本化 Rust command dispatcher → 临时 SQLite/filesystem → 重新打开后的 UI。浏览器 fixture 只能辅助键盘/布局断言，不能证明持久化、来源真实性或真实 provider entitlement。

## Testing Decisions

- Rust storage/service tests 覆盖 schema migration、workspace 隔离、保存来源关系、重复保存、新旧 version/hash、未知/跨 workspace/未完成 Item、超长/未知 kind、corrupt projection 和 atomic export cleanup。
- IPC integration tests 通过真实 dispatcher 验证 `artifact.save/list/get/export` 的 typed success/error；重开 workspace 后仍可 list/get；stale/unknown/path/redaction 错误不会产生部分 projection 或文件。
- Provenance tests 从现有 persisted Thread/Turn/Item 构造 artifact，断言 immutable snapshot、provider attempts、research result/source/tool refs、market/dataset/order refs 与 content hash；改变当前 Composer defaults 后详情仍保持原值。
- Redaction tests 使用代表性 key/secret/token/header/prompt-injection 文本，断言普通 projection、详情和导出均无秘密且保留可解释的 limitation/error；结果中的 untrusted text 不调用任何金融命令。
- Isolation tests 在保存/读取/导出前后比较 workspace、accounts、risk、model/gateway、approval/arming/reservation、outbox 和 credential boundary 的 sanitized snapshot，确认 S12 是只读金融边界。
- Browser helper 覆盖保存入口、Artifacts 导航、empty/loading/failed/success、detail/provenance modal、export status、键盘 focus/escape/return 和 390/768/1280 无横向溢出。真实原生 file chooser/CUA 若不可用需明确记录为 UNVERIFIED，不能用 fixture 标签升级为 PASS。
- 收尾运行 schema check、TypeScript typecheck/build、相关 Rust unit/integration、workspace feature suite、clippy、fmt、diff check 和 UI helper syntax；证据记录起始 SHA、完成 SHA、命令计数和未运行边界。

## Out of Scope

- 自动把 Ask/Research 结果创建为 Artifact、后台定时保存、跨 workspace 共享、删除/编辑已保存历史、全文检索、云同步、分享链接和远程上传。
- Strategy、Backtest、Chart、Workspace export/import/restore、retention/backup policy 的完整产物流程。
- 任何 broker/exchange 下单、审批、arming、risk、reservation、Gateway、账户刷新或凭据读写；Artifact 只能引用已有非秘密 order/proposal IDs。
- 新市场/新闻/filings/fundamentals provider、真实 entitlement、原始 provider payload 或未授权数据复制。
- S13 订单 proposal、S31 workspace restore、S33 全量页面/辅助技术回归和 S34 性能/签名包验收。

## Further Notes

- 本规范以 PRD §55.1、FR-036/FR-055、NFR-017、DATA-007、UX-006、AC-028/AC-029/AC-047 为产品约束；UI Spec §I/§14.1/§14.9 约束 Artifacts、Provenance、local export、焦点和窄屏交互；Backend ARD §32.4、§38、§41–42 约束 filesystem、secrets 和新 IPC schema。
- `workspace.export` 是后续完整工作区归档命令；S12 的 `artifact.export` 只导出一个经过脱敏和 hash 校验的研究产物，不能把二者混为一谈。
- 单一 tracer-bullet ticket 应覆盖 schema、Rust projection/validation、IPC/client、Artifacts UI 和验证；完成后才进入 S13。Issue resolution 必须绑定精确 SHA 和真实验证状态，未完成的 native CUA/provider 证据保留为 UNVERIFIED。
