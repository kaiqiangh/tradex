# S27.3 Provider 调度验收证据

实现票：[优先调度并限制 Provider 工作队列](https://github.com/kaiqiangh/tradex/issues/110)。审查基线：`4b813a9bb83e7fb458acb69f2242887ecbf64edb`。票的 resolution 记录随本证据提交的精确实现 SHA。验证日期：2026-09-29。[English](s27-3-provider-scheduling-evidence.md)。

## 交付行为

- Provider job 使用 P0 execution reconciliation、P1 账户安全刷新、P2 活跃监控、P3 research/history。符合条件的工作按 priority 排序，同一 priority 内 FIFO。
- 每个 provider 最多 4 个并发请求，非 P0 最多使用 3 个。等待工作最多 32 项，P2/P3 合计最多 24 项。溢出在发送请求前返回可重试的 `PROVIDER_BACKPRESSURE` / `RATE_LIMITED`。
- 账户冷却状态彼此独立；provider/IP 级冷却（包括 Binance request-weight 限制与无账户公共数据源）作用于全部账户。冷却账户不能阻塞其他账户符合条件的工作。
- 串行 Order Gateway 在预检/提交期间持有主进程 scheduler permit，并通过经过认证的私有 channel 返回脱敏冷却秒数。桌面 Gateway 忙碌时立即返回背压；持久化 SUBMITTING 前重新验证 grant eligibility。
- 等待 provider 时释放 scheduler mutex，不持有桌面或 stdio Control Plane/SQLite 锁。现有取消/current-job 检查在 scheduler mutex 外运行。
- 不合并金融状态转换与准确对账 attempt。当前 quote refresh 是本地 Paper 模拟，没有可合并的远程 quote/UI refresh producer；Backend ARD §36.2 保留未来 provider-backed feed 在 producer 侧 coalesce/sample 的要求。

## 验证

- PASS：7 项 scheduler 测试覆盖 priority 分配、P3 满载背压与 P0 进度、P1 优先、P0 保留并发槽、并发上限、冷却延迟、账户冷却隔离/取消。
- PASS：Gateway 忙碌准入、认证冷却元数据验证与共享回归。
- PASS：准确 Binance Live 429 对账保持 `UNKNOWN_RECONCILING` 与 active reservation；限流/缺失证据不转换成成功 resolution。
- PASS：最终源码的 `npm run check`，包括 schema 一致性、前端 typecheck/build、15 项前端测试、236 项 Rust library 测试、workspace integration suites 和需求清单。既有 ignored 测试仍为 ignored；清单覆盖 203 条需求、70 个页面、13 个 QA 场景，不证明运行时已完成。
- PASS：`cargo check -p tradex --features 'desktop integration-test order-gateway-runtime' --bins`、`cargo fmt --check`、`git diff --check`。
- PASS：全部 21 项测试，包括真实子进程 429 的单次 mutation/冷却回传回归：`cargo test -p tradex --features 'integration-test order-gateway-runtime' --test order_gateway -- --test-threads=1`。
- PASS：按固定基线独立完成 Standards 与 Spec 源码审查，均无剩余发现；审查者未运行测试。

沙箱最初以 EPERM 拒绝 loopback fixture 绑定端口，相关检查已在允许本地端口的环境重跑。中间版本的 GET wrapper 绕过旧自定义 `get` fixture，最终 metadata hook 已保持兼容。并发 default/feature Cargo 检查还共用了子进程可执行文件输出，最终整套检查改为串行。上述中间失败不作为验收证据。

只使用合成凭据、临时工作区与 loopback fake provider。没有真实 provider 订单写入，本票未重新验证原生账户恢复，也不声称 prototype/S33/全应用完成。用户已有的 workload dashboard/data 改动被排除。
