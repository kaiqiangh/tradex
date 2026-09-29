# S26.2 #104 Trading 212 Live 撤单与成交竞态验收证据

日期：2026-09-28。分支：`dev`。审查基线：`bb4112dcb994ca1ab31a87df993389d46c95edb6`。

## 范围

本切片覆盖 Trading 212 Live 账户范围内的耐久撤单历史，以及提供方确认收到撤单后订单竞态成交的情形。系统保留 `CANCEL_PENDING` 状态的 CANCEL attempt，保存最新的准确订单观测，并将撤单 acknowledgement 与终态 provider truth 分开呈现。

## 验证

- 定向 Rust 协议测试通过：`order_gateway::outcome_tests::cancel_ack_requires_the_preflight_provider_status`。测试接受与已审核订单一致的 `PARTIALLY_FILLED` 预检状态，并拒绝伪造的 `CANCEL_PENDING` provider status。
- 真实子进程 Gateway 测试通过：`real_child_sends_the_exact_live_cancel_and_records_only_pending_acknowledgement`。本地 loopback fixture 记录了账户摘要 GET 一次、准确订单 `9007199254740996` GET 一次，以及对 `/api/v0/equity/orders/9007199254740996` 的 DELETE 一次。
- Rust-backed 浏览器 fixture 中，用户审核并批准了准确的部分成交订单，然后显式准备撤单。TradeX 保留 `CANCEL_PENDING`，并说明 provider acknowledgement 不等于撤单已确认。
- 随后 fixture 返回竞态全额成交。点击 **Refresh exact order** 后，该订单从开放订单列表消失，已保存的撤单历史显示 `FILLED · TERMINAL`、订单数量 `1`、累计成交 `1`、剩余数量 `0` 和累计金额 `130`。耐久 execution preparation 查询仍保留 `CANCEL_PENDING`，并保存来自 `trading212.live.order-detail` 的准确 `FILLED` 观测。Gateway 日志仍只有原先一次 DELETE；刷新没有产生 provider 写入。
- 批准和准备操作后会失效账户范围的撤单历史缓存，因此新生成的批准会立即出现在当前页面的已保存历史列表中。
- 验证仅使用隔离临时 workspace、合成凭据和本地 fake provider。未使用真实 Trading 212 凭据、未请求真实 provider、未下单。点击式原型未修改，也未将其状态提升为 PASS。
