# S28 已知公司行为与精确账户标的元数据 — 验证证据

日期：2026-10-03。[实现票](https://github.com/kaiqiangh/tradex/issues/119)，[完整原始规范](https://github.com/kaiqiangh/tradex/issues/117#issuecomment-5966487911)。状态：**已实现；独立审查和交付待完成**。[英文权威版本](s28-actions-evidence.md)。

Settings 两个独立版本化来源复用已保存且符合条件的 Alpaca Paper、Trading 212 Live 密钥。Save 只持久化引用与审计元数据；Refresh 在控制面锁外执行固定认证 GET；Disconnect 保留借用账户和密钥。schema34 只迁移元数据。30秒进程观察绑定 workspace、来源/账户版本、凭据引用、进程会话、时钟代次和请求序列。失败、绑定变化或过期都会撤销当前证据资格，不更新原始读取时间或材料版本。

Alpaca 返回有限 AAPL/MSFT 处理日期查询，不代表完整前瞻公司行为或历史复权。保留官方16种分类、原始仅日期字段、精确小数、部分记录和完整分页。Trading 212 先核对选定账户远端身份/币种，再读取标的及交易所元数据。精确 ticker/ISIN/日程校验不证明权威证券身份、当前账户可交易性、执行场所、停牌或完整权限。即使刚收到响应，质量仍是延迟处理事件或10分钟元数据。独立覆盖/复权/账户绑定门禁保留其他金融检查、Local Paper 和保护性撤单语义；普通来源历史在断开后也屏蔽旧集成 fixture 替代。

## 验证

| 证据 | 结果及边界 |
|---|---|
| [完整检查](evidence/s28-actions/full-check.txt) | PASS：17项 Node、365项 Rust，以及 schema、类型、前端构建和追溯。38项 ignored 保留，不能声称全部通过。 |
| [完整集成](evidence/s28-actions/full-integration.txt) | PASS：402项 Rust，38项 ignored。外部 provider/vault fixture 配合真实控制面/SQLite，无权威状态注入。 |
| [Gateway 目标](evidence/s28-actions/gateway-integration.txt) | PASS：实际23项 Gateway 集成测试。 |
| [普通桌面构建](evidence/s28-actions/desktop-build.txt)、[二进制记录](evidence/s28-actions/native-build.json) | PASS，未启用 integration feature。仅本地临时签名验证，分发签名另行验收。 |
| [实际 HTTP 适配器](evidence/s28-actions/http-adapter-characterization.txt) | loopback HTTP 的401/403/429、不跟随重定向、超大响应和12秒超时撤销旧证据；无重试或更换 host/凭据。 |
| [Broker 迟到结果边界](evidence/s28-actions/broker-races-characterization.txt)、[最新本票测试](evidence/s28-actions/feature-characterization-final.txt) | PASS：身份/标的/日程三个读取阶段分别改变来源、账户、工作区或时钟，共12组；旧结果拒绝、后续读取停止。最新21项 PASS、2项默认忽略（慢测试此前单独通过）。这项检查晚于完整测试检查点；生产代码未变。 |
| [分页边界](evidence/s28-actions/pagination-bounds-characterization.txt) | 验证10页/1000条；单页或总分页溢出原子失败。 |
| [慢响应后配额](evidence/s28-actions/quota-receipt-green.txt)、[总期限余量](evidence/s28-actions/deadline-budget-green.txt) | 两项默认忽略的慢测试已单独执行并 PASS；49秒 vault 等待后不得在60秒任务内再启动12秒 HTTP。另36项 ignored 仍未验证。 |
| [公开 UI](evidence/s28-actions/ui-workflow-green-final.txt)、[启用前上下文](evidence/s28-actions/ui-trade-review.txt) | 真实 Rust：保存/CAS、26条按25+1分页、部分记录和精确小数、明确键盘动作、未保存禁止刷新、配额错误/Reload、过期、重开及断开保留密钥。Settings/Markets/Trade 在390/768/1280px 验证；启用前审阅保留原始时间/材料及独立阻塞，审批数为0。 |
| [普通原生真实读取](evidence/s28-actions/native-hosted.json)、[来源截图](evidence/s28-actions/native-metadata.png) | 已有密钥成功读取实际 Alpaca 生产数据及 Trading 212 Live；自然过期保留原始时间/版本。只证明实际响应能力；未归档私有余额、持仓、订单、远端账户 ID 或凭据。 |

Alpaca 首次读取 `2026-10-03T10:42:01.706Z`，处理日期范围2026-09-03至2026-11-02内有一条已知 MSFT 现金分红。Trading 212 首次读取 `2026-10-03T10:43:57.066Z`，与选定 EUR 账户身份一致，返回 AAPL/MSFT USD 元数据及各126条日程事件。两者自然过期后为 UNAVAILABLE，原始时间与材料哈希不变；provider 观察时间仍缺失。原生后台 Hot Market 显示 PAUSED/加载中，此尝试不证明原生详情；已返回 Settings 并结束详情订阅，真实 Rust 浏览器 Market 检查单独记载。既有 Live 账户刷新恢复认证读取，但未知权限确认没有将 scope 提升至 VERIFIED，也未启用交易。

原始 RED/GREEN 和未成功的工具/编译/导航尝试保留在 `evidence/s28-actions/`，不能算作产品 PASS。UUID 大小写身份别名、HTTP 总期限余量、启用前缺少上下文的真实失败已修复。最早 Market 截图早于最终哈希/期限修复，仅证明展示；最终 Settings/启用前记录使用重建桥接。完整检查早于仅 feature 的 HTTP 测试及最后的启用前面板添加，后续完整集成、类型检查和普通桌面构建分别覆盖这些改动。

## 审查修复检查点

用户确认的审查基线为 `2f42df60407ef695f6393c84bc24dcaff7e83332`，对比捕获工作树。独立 Standards 和 Spec 审查串行执行，初始均为 NOT PASS；报告及哈希快照保留于 evidence/s28-actions/review-*-initial.md 和 review-initial-snapshot.json。最终独立验收、commit 和 dev push 仍待完成。

串行修复：(1) 每个已知账户普通 summary 读取均原子加入端点额度，包括首次元数据读取前；请求进行中保持占用，按实际完成/失败和 exhausted/reset/Retry-After header 计冷却；(2) 原子拒绝矛盾日程转换，同时允许有界窗口初始状态未知和末尾未结束；(3) 不可变 pre-arm/审批面板明确标注捕获时间/状态/评估，并说明不报告当前资格。当前投影轮询仍由后端拥有，不修改 review 捕获内容。

公开边界 RED/GREEN 证据覆盖普通 summary 竞态、普通/扩展时段冲突日程和捕获 review 文案。完成时序测试覆盖成功响应耗尽额度、provider reset/Retry-After 和慢速失败传输。进行中 summary 竞态、完成时序、慢元数据接收后额度（56秒）及全部事件类型/有界窗口用例均通过。浏览器捕获过期及响应式检查通过，未 Arm/审批/金融修改；重开 review 捕获后端 UNAVAILABLE 评估，保留原始 receipt/material。

额度修复后的首次完整检查发现两项既有 synthetic 对账测试回归：它们同一账户重复 summary 读取未遵守新增最小间隔。既有命名 fixture 现等待真实冷却期，不新增权限或额度绕过。Trading 212 library 定向回归通过12项。失败完整检查输出保留。修复后完整检查通过（17 Node / 369 Rust / 38 ignored，schema/type/前端构建及追溯），完整 integration 通过（406 Rust / 38 ignored）。Gateway 全集重复在 Bitget pre-dispatch 失败且 fixture 无请求记录，定向复现通过。诊断记录模拟 provider 的6秒接收窗口在零请求时结束。fixture 生命周期现与其他 loopback provider 一致为15秒，包含 executable pin 校验/child 启动；生产 HTTP deadline 未改变。完整 Gateway 通过23项，失败及诊断输出全部保留。普通原生 release 重建已通过，未启用 integration feature。当前读取等待 macOS Keychain 系统认证（已观察 SecItemCopyMatching）；新 hosted 读取及独立复审仍待完成。此前原生观测是历史证据，不能验证修复后的 producer binary。


首次串行复审两轴仍为 NOT PASS：隔离 Gateway 的身份 summary 绕过主进程端点额度，包括保护性 CANCEL。报告和146文件快照保留为 review-*-remediation-1.md 及 review-remediation-1-snapshot.json。既有命名 legacy 子进程撤单 fixture 复现第二次 preflight 发出 summary 而未返回 PROVIDER_RATE_LIMITED（RED）。现由主进程在发送 grant 前预留额度，并在 begin_request 前验证有界认证完成回执及确认。传输/子进程失败和未使用预留保守保留完成时冷却。定向 GREEN 和成功响应耗尽 header 测试通过；正常5秒最小间隔结束后，30秒 Retry-After 仍阻止第二个子进程。首次 Gateway 全量回归随后正确暴露原本独立 loopback fixture 都使用 remote id777；这些外部 provider 现返回独有监听端口为账户身份，helper 保留刻意身份不匹配。未增加额度 reset 或新权限 fixture。完整 Gateway 通过23项。这次 Gateway 改动前的 release 不作为最终来源验收；最终检查、原生重建/重读及下一轮串行审查仍待完成。

Gateway 修复后的完整检查首次仅在未修改的 Thread reopen 用例失败；定向复现及完整重跑通过（369 Rust /17 Node /38 ignored）。该孤立失败仍未解释，两份输出保留。完整 integration 通过406/38ignored，最新 Gateway 全集通过23项，含成功响应额度耗尽 header 测试。普通 release 编译随后成功，但实际配对检查发现：Tauri 在主桌面写入 pin 后重建并覆盖了 Gateway；当前 Gateway 哈希不在桌面字节中，而此前 pin 哈希存在。失败配对 JSON/构建日志保留。构建脚本现于普通构建后原子恢复 Tauri 前已固定的精确 Gateway 字节；实际普通重建、pin 配对、签名及新进程 Gateway 启动均通过。最终来源普通进程的公司事件 Refresh 仍在等 Keychain；16:10:56Z 观察到 Working，而该刷新发生在16:01:57Z 可信时间同步后，超过60秒 job 上限。代码在同步 vault.get 返回后才检验 deadline，尚未将该真实读取计为通过；此现象交给下一轮独立审查判断。此打包修复不带来金融写操作或权限放宽。

## 验收与交付边界

上述记录覆盖 AC1–AC11 的实现与可取得本地/原生观察。AC12 串行独立 Standards、Spec 审查、最终来源绑定、提交和正常 dev push 尚待完成，本检查点不宣称关票。

OD-005 整体、完整前瞻行为覆盖、历史复权、权威证券身份、当前账户可交易性/停牌、完整权限、合格 SIP 报价/深度及执行级 FX 保持独立门禁。报价 #116、Trading 212 #113、前置规范 #117、map #1 保持 OPEN。本票审查交付后才进入下一个 FX 前置项。S17 真实 Paper 和 S27 实际睡眠唤醒仍为用户跳过/未验证。此切片没有 Arm、已签发审批、reservation、下单/撤单写操作、main PR 或 main merge。

### 第二次复审与受保护凭据截止修复

第二次串行复审核对167项哈希，两轴均 NOT PASS：同步 vault 获取可超过45/60秒来源任务上限。报告及冻结快照保存在 review-*-remediation-2.md 和 review-remediation-2-snapshot.json。沿用已确认的公开来源/外部 vault 测试边界，RED 复现 Refresh 等待63秒。修复后凭据获取在拥有自身数据的有界 worker 中进行，Control Plane 锁外执行，进程最多32项未结束读取，同一私有引用实际完成前最多一项。来源在期限到达时返回经过清理的 UNAVAILABLE；已超时的接收方丢弃会清零的晚到凭据，不执行 HTTP 或发布证据，仍要求用户完成系统认证。定向 GREEN 与等待期间重复刷新检查通过：命令60.007秒返回，没有 HTTP，重复等待立即拒绝，晚到凭据未恢复证据，实际获取结束后新的明确刷新成功。这是本地假 vault 证据；最新全套检查、普通桌面构建与独立复审仍待完成。

截止修复后的默认完整检查通过369 Rust/17 Node/39 ignored。首次集成检查只有未改行为的完整风险策略重开测试失败，原断言只显示 ok=false；定向复现通过。现已为两处重开断言保留公开错误结果，未改变行为；完整诊断版集成重跑通过406/39 ignored。最初风险重开失败仍未明确原因，早前 Thread 重开失败同样保持原因未知，失败与复现记录均保留。Gateway、普通桌面重建/原生读取和最终串行复审仍待完成。

最终默认检查再次通过369 Rust/17 Node/39 ignored，Gateway23项通过。普通桌面构建通过，捕获的运行源码输入未漂移；Gateway 字节配对、父程序编译 pin、本地 ad-hoc 签名及新父/子进程实际启动均通过。新普通进程早前采样观察到专属 vault worker 等待 SecItemCopyMatching；稍后的 UI 观察确认公司 Refresh 已返回经过清理的 UNAVAILABLE 并恢复刷新按钮，没有发布公司来源接收记录。再次采样已未找到该 worker，因此不能独立证明终态观察时仍同时等待认证。用户完成认证后，公开面板仍无来源接收记录，晚到凭据未补发布证据。原生终态在启动128秒后观察到，因此不能独立证明精确60秒边界；时间证明由公开 Rust/假 vault 测试提供。当前公司与所选 Trading 212 真实读取仍需用户完成 Keychain 认证；未归档秘密、私密账户数值或完整堆栈。最终串行复审和 dev 交付仍待完成。

用户已完成 Keychain 认证。相同普通二进制上的明确公司重读成功：新鲜 AVAILABLE 有界查询保留1条已知事件、原始接收时间、material identity 和仅日期精度，完整覆盖/历史调整仍不支持；前一查询自然过期。native-hosted-company-auth-deadline.json 只归档经过清理的数量/quality/接收时间/hash，不归档事件正文。已有 Trading212 Live 重测进入 REVIEW_REQUIRED，没有输入新 key 或提升权限范围。随后普通进程重开，观察按契约撤销；源码输入仍匹配构建，但桌面点击发生屏幕捕获 -3811 错误，公开 Expand 动作也未改变 popup。真实 Live 元数据仍待桌面控制恢复与连接确认，这只是工具/原生验证限制，不代表来源 PASS 或产品失败。

最终独立 Standards → Spec 串行实现复审均 PASS：每轴0项可操作发现；Standards 保留1项非阻断重复代码建议。两轴审查前后185项哈希全部匹配，报告及冻结快照保存在 review-*-remediation-3.md 与 review-remediation-3-snapshot.json。实现提交 `32c2a0b99e52b7f7c6e2da667141fa1b62d64eb6` 的36项源码/测试/产品契约字节匹配该快照和普通构建输入。当前 Live 元数据仍待验证；#119及所有验收父票保持 OPEN。随后提交证据并正常推送 dev，不代表关闭或主分支交付。
