# S25.3 #99 Bitget Spot Live 未知 PLACE 对账验收证据

日期：2026-09-28。分支：`dev`。审查基线：`0fa304f471e938a535aeef47b9bd5fd8af498afe`。

## 范围

本切片将共享 S25 证据账本和 Order Drafts 面板扩展到已保存的 Bitget Classic Spot Live PLACE attempt `UNKNOWN_RECONCILING`。Control Plane 将查询绑定到准确的已保存账户、proposal、attempt 和 `clientOid`。查询只使用普通 Bitget Live 读取路由；所有不确定结果都会保留未知 attempt 与 active reservation。

## 本地验证

- Rust Control Plane fixture：使用合成 Bitget Live 账户和临时 SQLite/outbox，持久化准确的 `orderInfo?clientOid=...` 候选，并覆盖账户不匹配、side 不匹配、空响应、格式错误、认证失败和传输失败。所有负例均使 attempt 保持 `UNKNOWN_RECONCILING`、reservation 保持 `ACTIVE`，且不写入券商订单 identity。重放 outbox 返回全部七条证据事件；重新打开 SQLite 后恢复全部七条证据。
- 浏览器验证：Rust-backed integration bridge 与隔离 Vite 页面展示已保存的 Bitget Live 账户/远端 `userId`、准确 client order ID、provider order ID/status、查询范围及候选专用说明。账户不匹配、side 不匹配、空结果、认证失败和传输失败通过同一面板的定时刷新显示为不确定。Bridge 报告券商订单写入数为 0。
- 响应式验证：证据面板在 1280、768、390 px 视口内完整显示；页面和面板均无横向溢出。浏览器没有应用控制台错误。
- 请求边界：fixture 验证签名请求只对普通 Bitget Live endpoint 发送 `GET`、省略 `paptrading`，且不使用 Demo/Testnet 路由。未使用真实凭据或发出真实 provider 请求。
- 原型边界：未修改 `docs/prototype/`；此运行时 fixture 不认证点击式原型或 provider-hosted 账户行为。

收窄后的人工处置和 S25 最终验收记录在已关闭的父项 [#96](https://github.com/kaiqiangh/tradex/issues/96#issuecomment-5863761675) 中。本证据仍仅覆盖 Bitget Spot 对账。
