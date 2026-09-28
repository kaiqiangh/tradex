# S25.4 Live 人工处置验证证据

评估日期：2026-09-28。状态：**部分完成；issue #100 保持 OPEN。**

## 已实现边界

- Binance Spot Live 和 Bitget Spot Live 只有在最新证据新鲜（30 秒）、完整、绑定当前账户观测，且准确查询 attempt 已持久化的 client-order ID 并恰好返回一条记录时，才开放 `CONFIRMED_SUBMITTED`。记录还必须匹配已保存的 attempt、账户、不可变 proposal、标的、方向、订单类型、支持的数量及可信提交时间窗口。
- 后端在 SQLite immediate transaction 中再次校验这些条件，将观察到的 provider order ID/status 关联到 attempt，并与人工决策及两个 outbox event 一起写入。PLACE reservation 仍保持 active；不会推断或合成 fill。UI 会刷新账户健康状态，账户仍为 `DISARMED`。
- Trading 212 相似订单候选，以及空、不完整、延迟、过期、不匹配或不支持的证据，都只开放 `KEEP_RECONCILING`。Keep 或关闭界面都会保留 `UNKNOWN_RECONCILING` 与 active reservation。
- 当前不开放 `CONFIRMED_NOT_SUBMITTED` 或 reservation 释放：现有 provider 路径均不能提供所需的充分未提交证明。空查询仍是不确定结果，因此 #100 的未提交确认/释放验收标准尚未满足。

## 验证结果

- `npm run check` 通过：schema 一致性、TypeScript 生产构建、13 个前端单测、205 个 Rust 单测、workspace 集成测试及需求追踪（203 条需求、70 个 screen、13 个 QA 场景、23 个基线文件）。仓库明确标记为 ignored 的测试保持跳过。构建仍报告既有的大 bundle 提示和 fixture dead-code warning。
- `cargo build --manifest-path src-tauri/Cargo.toml --features integration-test,order-gateway-runtime --bin tradex-ipc --bin tradex-order-gateway` 通过。
- Rust-backed 浏览器 UI 验收通过：Trading 212 超时后仅 Keep、Binance 精确已提交订单确认、Bitget 精确已提交订单确认。测试使用临时 SQLite workspace 和合成 provider fixture，不请求外部 provider，也不触碰真实账户。Binance/Bitget fixture 的订单写入计数均为 0。Trading 212 流程使用本地 fake gateway；人工 Keep 没有新增写入，原有合成 POST 计数保持不变。Binance/Bitget 保留 active reservation，账户保持 `DISARMED`；证据面板通过 1280/768/390 px 检查。
- 未修改 prototype 代码或 fixture package。这些运行时检查不改变 prototype coverage 或 QA 证据状态。

## 尚未满足的验收

在为具体 provider 定义安全的未提交证明、准确且最多一次释放 reservation，并覆盖 Trading 212、Binance Spot Live、Bitget Spot Live 的版本冲突和 fill 竞态前，#100 应保持 OPEN。
