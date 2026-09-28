# S25.4 Live 人工处置验证证据

评估日期：2026-09-28。状态：**收窄范围内验收完成；Standards 与 Spec 已串行复审通过；#100 和 #101 已关闭。父项 #96 等待 S25 最终验收。**

## 已实现边界

- Binance Spot Live 和 Bitget Spot Live 只有在最新证据新鲜（30 秒）、完整、绑定当前账户观测，且准确查询 attempt 已持久化的 client-order ID 并恰好返回一条记录时，才开放 `CONFIRMED_SUBMITTED`。记录还必须匹配已保存的 attempt、账户、不可变 proposal、标的、方向、订单类型、支持的数量及可信提交时间窗口。
- 后端在 SQLite immediate transaction 中再次校验这些条件，将观察到的 provider order ID/status 关联到 attempt，并与人工决策及两个 outbox event 一起写入。PLACE reservation 仍保持 active；不会推断或合成 fill。UI 会刷新账户健康状态，账户仍为 `DISARMED`。
- Trading 212 相似订单候选，以及空、不完整、延迟、过期、不匹配或不支持的证据，都只开放 `KEEP_RECONCILING`。Keep 或关闭界面都会保留 `UNKNOWN_RECONCILING` 与 active reservation。
- 当前不开放 `CONFIRMED_NOT_SUBMITTED` 或 reservation 释放：受支持的 provider 路径均不能提供所需的充分未提交证明，空查询仍是不确定结果。收窄后的 #100 只覆盖有证据支持的 `CONFIRMED_SUBMITTED` 与 `KEEP_RECONCILING`；研究 #101 已关闭，未发现 provider 发布的有界缺席规则。未来如 provider 契约新增专属缺席规则，须另行制定证据与实现范围。

## 验证结果

- `npm run check` 在已审查并提交为 `dev@f67463d` 的代码快照通过：schema 一致性、TypeScript 生产构建、13 个前端单测、207 个 Rust 单测、workspace 集成测试及需求追踪（203 条需求、70 个 screen、13 个 QA 场景、23 个基线文件）。仓库明确标记为 ignored 的测试保持跳过。构建仍报告既有的大 bundle 提示和 fixture dead-code warning。
- `cargo build --manifest-path src-tauri/Cargo.toml --features integration-test,order-gateway-runtime --bin tradex-ipc --bin tradex-order-gateway` 通过。
- Rust-backed 浏览器 UI 验收通过：Trading 212 超时后仅 Keep、Binance 精确已提交订单确认、Bitget 精确已提交订单确认。测试使用临时 SQLite workspace 和合成 provider fixture，不请求外部 provider，也不触碰真实账户。Binance/Bitget fixture 的订单写入计数均为 0。Trading 212 流程使用本地 fake gateway；人工 Keep 没有新增写入，原有合成 POST 计数保持不变。Binance/Bitget 保留 active reservation，账户保持 `DISARMED`；证据面板通过 1280/768/390 px 检查。
- Binance 公共 IPC 回归还会先以过期的 attempt 与 evidence state version 提交 `CONFIRMED_SUBMITTED`；两次请求都被拒绝，且 attempt 仍为 unknown、reservation 仍为 active，随后正确版本才能成功确认。
- Standards 与 Spec 已按 `f7531640a69bd11fe8473d3f322ea4e348f69e7b..f67463d857850d69ba68050b2110612e3022394b` 串行复审通过；此前 AC7 问题已由 stale-version 提交路径回归测试解决。
- 未修改 prototype 代码或 fixture package。这些运行时检查不改变 prototype coverage 或 QA 证据状态。

## S25.4 验收

收窄后的 #100 验收已满足：新鲜且准确的 Binance/Bitget 订单证据仅授权关联已提交订单；Trading 212 以及所有不支持、不完整、过期、延迟或不匹配的证据都只授权 Keep；过期的 attempt/evidence 版本会被拒绝；reservation 保持 active；确认为已提交后账户仍为 DISARMED。本切片不提供已确认未提交状态转换或 reservation 释放。

## S25 父范围的后续工作

研究 issue #101 未发现当前 Live 接口存在 provider 发布的有界充分缺席保证。因此 S25 不开放 `CONFIRMED_NOT_SUBMITTED`，也不根据查询缺失释放容量。若未来 provider 契约增加此类规则，后续实现仍须保证审计与 reservation 原子、最多一次提交，并覆盖受支持 provider 的版本冲突/fill 竞态。
