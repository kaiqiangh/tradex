# S03 / #15 账户复用、自动读取与本地移除证据

状态：**VERIFIED（2026-09-13 #15 垂直验收完成）**。账户列表恢复、已有连接复用、版本化刷新、schema 驱动的新账户入口、断开后的本地清理和窄屏/键盘路径均已取得证据。S02 其他提供方范围、S27 健康恢复和 S33 全页面交叉回归仍由各自工作项负责。

实现提交：`f647453`（已有连接选择/复用、权威列表刷新、清理 UX）、`fc493cf`（浏览器回归等待真实 state-version 转换）；开发分支：`dev`。

## 已实现

- Accounts 与 Settings 在挂载、重连和 workspace 切换时读取权威 `account.list`，并自动选择已持久化连接以恢复详情。
- `Connection source` 同时提供 `New account — enter credentials securely` 和已有本地连接。已有连接路径只调用带 `expectedStateVersion` 的 `account.refresh`，不打开凭据窗口，也不把 secret 送入 renderer 或 IPC。
- `DISCONNECTED` 且 credential 为 `MISSING` 的连接保留脱敏审计行，显示幂等的 `Remove local connection`；说明本地访问已停止、provider key 与外部订单不变。`DELETE_PENDING` 仍显示可重试的 Keychain cleanup。
- 新账户继续使用 provider credential schema 和原生安全输入；provider/environment/label 在已有连接路径不可编辑。

## 自动化与协议证据

- `cargo test --test providers`：7 passed，1 ignored（原生 Keychain 测试按平台跳过）。覆盖连接、刷新、断开、`DELETE_PENDING`、丢失 credential、重开恢复和 workspace 隔离。
- `npm run check`、`cargo fmt --all -- --check`、`git diff --check` 已在当前 `dev` 基线通过；没有引入 renderer secret input。
- `tests/provider-ui.mjs` 的导出 `checkProviderUI` 在 Rust-backed 浏览器上完整通过，函数返回 5 条观察结论；测试进程随后停止且工作区保持干净。

## 本轮隔离 UI 验收（2026-09-13）

使用临时 workspace `a7c0f3bf-c575-4937-a207-aaf26661a08a`，路径为 `/private/var/folders/pz/jpgkm5cd7bn8vj_klvtmvf700000gn/T/tradex-browser-nU5U3s/accounts-1789309539980`，仅使用 synthetic Alpaca Paper fixture，不接触真实账户或密钥。

- 页面重载后自动恢复已保存连接；source selector 列出已有连接，选择后 provider/environment/label 变为只读。
- `Use existing account` 复用路径和 `Refresh account` 均等待真实 `data-state-version` 变化；没有凭据对话框或 renderer secret 字段。
- 768px 与 390px 均无横向溢出，Account Health、详情和操作保持可达；键盘 Enter 可触发复用/刷新/清理按钮，浏览器无 alert、error 或 warning。
- 用户确认后在该隔离 fixture 执行 `Remove local connection`。UI 读回 `DISCONNECTED`、`credential:MISSING`、`execution:BLOCKED`，Refresh disabled、Remove enabled，并显示 `Local access stopped. External orders and the provider API key are unchanged.`。审计行和余额/持仓观察仍保留。

当前真实桌面 workspace `22acfa1d-b5da-420a-a754-f01ea0c43458` 只做了只读复用检查，未执行任何 disconnect 或清理；因此不存在真实 credential 删除证据，也没有修改用户账户。

## 串行审查与边界

Standards 和 Spec 审查均已对 `4f0bbfb..fc493cf` 返回 PASS；本票后续只增加证据文档，没有改变实现。账户清理只删除 TradeX 自己持有的 Keychain 引用并保留非秘密历史；不会撤销 provider key、取消外部订单或扩大账户能力。完整 S02/S27/S33 验收仍未由本票代替。
