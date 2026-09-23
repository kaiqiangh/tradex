# S18 #61 Trading 212 Demo 提交与未知结果恢复验收

日期：2026-09-23。分支：`dev`。实现提交：`873d42f68377f0b7df8d1c35bdcaabfbc1f63791`。起始基线：`8e5992c0546c57e984c6f84288fca5054cb4550d`。

## 已验证范围

- Trade 页面展示 Trading 212 Demo、不可变 Proposal 的订单细节，并要求单独确认；Market 订单明确显示 extended-hours 关闭。确认绑定当前 workspace、连接、账户、环境和 Proposal 版本。
- Rust IPC 与 SQLite/outbox 在网络写入前持久化提交尝试；重复提交复用原尝试。提供方固定使用 Demo host 和该 Demo 连接的 Keychain 凭据；不支持字段、身份变化及 Live 路由均失败关闭。
- Market-DAY 与 Limit-DAY/GTC 保留精确十进制数量和卖出方向编码。成功响应只作为 acknowledgement，不当作成交；明确拒绝与未知结果分类分开。
- 超时、派发后传输错误、408、损坏或身份不匹配的响应以及进程重开后的 `SUBMITTING` 均进入不可重发的 `UNKNOWN_RECONCILING`。attempt 可经只读查询恢复；账户冻结不会因空结果或相似订单自动解除。

## 检查结果

- `npm run check`：通过；包含 schema、构建、类型检查、前端测试和 Rust workspace 测试。Rust library 128 项通过；provider 集成 31 项通过、1 项显式忽略；Trading 212 读取集成 2 项通过、1 项原生 Keychain 检查显式忽略。需求追溯工具报告 201 条需求、70 个页面、12 个 QA 场景、23 个基线文件；该清单检查不替代行为测试。
- `cargo fmt --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `git diff --check`：通过。
- Rust-backed browser fixture 手动检查覆盖 1280/768/390 宽度、独立确认、取消确认后不提交、acknowledgement 不等于 fill、重载恢复未知尝试并隐藏重复提交入口；浏览器控制台无错误。此交互检查发生在最后的响应 ID helper 命名清理和损坏 JSON 负例补充之前；这两项后续改动已包含于实现 SHA 并由上述最终全量检查验证，浏览器交互未在提交后重跑。

## 验收边界

本票使用合成 IPC/provider fixture，没有调用 Trading 212 API，没有使用真实凭据，也没有创建真实 Demo 订单。成功真实 Demo 写入、查询和必要的撤单保留给依赖票 #64，在用户于 TradeX 中确认具体 Demo Proposal 后验收。Trading 212 的官方 Market reference 将成功响应作为订单资源返回；实现按响应的 provider order identity 记录 acknowledgement，并独立等待后续状态查询，不将接收响应标作成交：[Market order reference](https://docs.trading212.com/api/orders/placemarketorder)。
