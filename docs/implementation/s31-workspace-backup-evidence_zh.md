# S31 工作区备份、恢复、迁移与保留证据

**状态：** 功能、隔离原生工作流验证、完整仓库检查、普通桌面构建以及串行 Standards/Spec 复审均已在 feature 分支完成。S31 实现票为 [#137](https://github.com/kaiqiangh/tradex/issues/137)，关联的规范票 [#136](https://github.com/kaiqiangh/tradex/issues/136) 保持开放。

- 分支：`features/s31-workspace-backup-import-retention`
- 复审基线：`015f4ba19b0dcba138d2a48fafb42b6323821e83`
- 契约来源：RevC PRD FR-039、FR-040、FR-063、AC-047；Backend ARD §38、§45.1；实现范围见 #136/#137。
- 页面：Settings → Data & Storage。

## 已实现行为

Backup Now 与 Export Workspace 生成有界 `.tradex` 归档，包含版本化 manifest、工作区身份、存储 schema 版本、成员大小和 SHA-256。当前归档只有一个固定名称的 SQLite 工作区快照。临时文件权限仅当前用户可读写，完成后以不覆盖已有目标的方式发布。预览会先验证归档和数据库，再展示工作区、兼容性、警告及账户凭据引用检查。原生进程只检查对应 Keychain 项能否读取，并只返回状态；引用和密钥都不会返回界面。存在的项仍需重新认证和对账后才能使用。

原生 Keychain 检查结果绑定到已校验归档的 manifest 摘要，只有 preview 或 restore 观察到相同摘要时才应用。归档若在两次读取之间被替换，不会继承先前的凭据状态。

恢复时再次核对预览 manifest 摘要，在私有暂存目录中解包并迁移受支持的旧 schema；撤销已签发审批和当前撤单意图，将账户标为 stale 且 disarmed，停止 Gateway，重置模型会话，并在发布前验证 SQLite 完整性。缺失的 Keychain 项会持久化为 `MISSING` 且保持失败/阻断；无法检查时仍为 `UNCHECKED`。发布恢复工作区前，会先备份当前工作区。归档校验失败不会修改当前工作区。

由于尚未核实各数据源的权利，归档排除行情历史和缓存文件。保留设置可以先保存、不删除数据；用户明确 Apply 后，才清理未关联的普通产物、过期本地 OHLCV 行和 `cache/` 目录直接包含的普通文件。金融执行、预留、结算、审批、对账及审计记录不在清理范围内。没有默认清理期限。

界面提供原生文件选择、归档预览与明确恢复确认、状态/错误反馈、焦点限制与返回，以及彼此独立的保留设置保存和应用操作。页面说明归档排除项和恢复后的 fail-closed 账户状态。

## 验证证据

- `cargo test --test workspace_archive` — 通过：归档导出/预览/恢复、在创建恢复目录前拒绝篡改归档、保留边界、schema 37→38 迁移。
- `cargo test workspace_restore_invalidates_issued_live_approval_and_keeps_accounts_disarmed --lib` — 通过：已签发审批被撤销，Live 仍撤防，摘要匹配的缺失 Keychain 项被持久化为阻断状态。
- `cargo test --lib workspace_archive` — 通过：带旧 manifest 摘要的凭据状态在预览和恢复中均保持 `NOT_CHECKED`。
- IPC 生成后 `npm run typecheck` 与 `npm run schema:check` — 通过。
- `npm run check` — 通过：Rust/JSON/TypeScript IPC 一致，前端生产构建，Node 测试 17/17，库测试 257/257，所有未忽略的 workspace fixture/集成测试通过；需求追踪覆盖 203 项需求、70 个界面、13 个 QA 场景及 23 个基线文件。按既有门禁忽略了 provider 金融前置条件、显式 Keychain 和时间边界测试；未连接用户的 provider 账户。
- `cargo clippy --workspace --all-targets --all-features` — 退出码 0；输出既有的 `result_large_err`、fixture 未使用代码等警告。
- `npm run desktop:build` — 通过，生成 `target/release/tradex`；前端保留原有 5.35 MB bundle 体积警告。
- `git diff --check` — 通过。仓库范围的 `cargo fmt --all -- --check` 报告未修改文件/代码区段的既有格式差异；本次 S31 Rust 改动已格式化，未批量改写无关代码。
- 复审基线：`015f4ba19b0dcba138d2a48fafb42b6323821e83`。Standards 复审未发现书面规则违规，有一项非阻断的归档校验/解包重复逻辑启发式。将原生凭据检查绑定到已校验 manifest 摘要后，Spec 复审未发现剩余验收缺口。
- 隔离的原生 macOS TradeX 构建与界面流程：Backup Now 写入 `.tradex` 文件；Export Workspace 通过原生 Save 面板保存 `workspace.tradex`；重新打开该文件后显示 `Archive validated`、schema 38、单成员、725,682 字节、**2 条 Accounts 记录**，并针对合成 Live 账户显示 Keychain 缺失警告。这确认了没有 `workspace_id` 列的工作区本地账户行仍计入预览。此前一次隔离恢复已成功创建新的临时工作区；合成账户保持 failed/disarmed，恢复凭据并对账前 Refresh 仍被阻断。界面明确说明未发出 provider 请求。
- 原生保留设置成功保存；Apply retention 显示确认并被取消，没有执行删除。取消导入对话框后，键盘焦点返回触发按钮。原生界面在 1280 px 宽度验证；隔离浏览器在 1280/768/390 px 的布局检查均无横向溢出。浏览器检查没有替代原生文件对话框或存储验收。
- 所有原生操作仅使用 `/private/tmp/tradex-s31-ui-default-safe`；`lsof` 确认运行中应用的 SQLite 文件句柄保持在该隔离路径下。fixture 明确为合成数据；未输入凭据、刷新账户、发出 provider 请求或执行金融写入。

按用户当前范围，Alpaca、Binance、Bitget 与 Trading 212 提供方集成不在本票内。未发出 provider 请求或金融写入。S27 实际睡眠/唤醒证据仍为用户跳过、未验证。点击式原型未修改。

## 剩余交付门禁

只关闭实现票 #137，并将已复审切片合并回 `dev`；父规范 #136 保持开放。S27 实际睡眠/唤醒仍按用户指示跳过并标记未验证；provider 集成按用户指示跳过。继续按顺序处理其余非 provider Wayfinder 票。S33 全应用无障碍、S34 性能/签名打包、S35 总验收及最终 `dev`→`main` PR 保持为独立门禁；整个 map 在 `dev` 上通过前不要创建该 PR。
