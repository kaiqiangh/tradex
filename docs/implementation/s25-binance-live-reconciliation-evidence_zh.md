# S25.2 Binance Spot Live 对账验收证据

## 范围

Issue #98 为已保存且处于 `UNKNOWN_RECONCILING` 的 Binance Spot Live PLACE attempt 增加只读对账。共享界面会核验已保存的账户和 proposal 身份，只调用 Binance 普通 Live 账户与准确订单查询路由；没有准确观测时继续保留不确定状态和 active reservation。

## 本地验证

- Rust-backed 集成 fixture：使用合成数字型 Binance SPOT 账户和临时 SQLite workspace 创建未知 attempt，并持久化 `tx-{去掉连字符的 attempt UUID}` client order ID。fixture 返回准确候选；Binance `-2013`、认证失败及订单字段不匹配均保持不确定。窗口超时会 disarm 账户但保留 attempt 和 reservation；Keep Reconciling 只记录决策，不重启 provider 查询。
- 浏览器验证：Rust-backed 集成桥接与本地 Vite 界面显示已保存的 Binance attempt、准确 client order ID、仅作候选的订单状态及已完成的准确订单查询；reservation 仍为 active。
- 响应式检查：证据面板和页面在 1280、768、390 px 下均无横向溢出；浏览器没有应用错误。
- Provider 写入保护：隔离 fixture 报告 provider order POST 写入数为 0；Rust mock 另外验证对账请求仅使用 GET。未使用真实 Binance 凭据，也未请求真实 provider。
- 原型边界：未修改 `docs/prototype/`；运行时 fixture 不证明点击式原型或 provider-hosted 账户行为通过。

父 S25 issue #96 仍开放，用于跟踪其他 provider 及确认已提交/未提交的处置范围。
