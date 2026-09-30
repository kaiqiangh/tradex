# S27 父级验收审计

父项：[恢复 Live 执行并协调中断工作](https://github.com/kaiqiangh/tradex/issues/107)。审计基线：`f5ed6d114a6b7e5cc905f8ab3e3cf015fc5191cf`。日期：2026-09-29 UTC。[English](s27-parent-acceptance-audit.md)。

**IN_PROGRESS：父项继续 OPEN。** 四张实现票已关闭，但子票结案不证明父项或整张 map 的全部门禁完成。

## 需求审计

| 父项 User Stories | 权威证据 | 剩余边界 |
|---|---|---|
| 1–4：重启/重开、持久工作、及时恢复、就绪门禁 | [S27.1](s27-1-live-startup-recovery-evidence_zh.md)：SQLite 重开、选择所有受支持 Live 账户、恢复前 stale/disarmed；2026-09-29 原生 T212 只读恢复耗时 0.682 s | 原生运行仅有一个已连接 Live 账户且无开放订单；尚未证明完整 Provider/开放订单及代表性崩溃计时验收。 |
| 5：休眠/恢复 | [S27.2](s27-2-live-resume-recovery-evidence_zh.md)：确定性失效/恢复和原生应用重新激活；macOS wake observer 已编译 | 实际 OS 睡眠唤醒及其后的 Provider 恢复完成仍未验证。 |
| 6–7：账户故障与政策作用域 | S27.2 与 [S27.4](s27-4-model-failure-isolation-evidence_zh.md) 的账户级 auth/stream 故障；[S23 政策顺序](s23-pre-dispatch-release-evidence.md)和[并发](s23-concurrency-transaction-evidence.md) | stream 健康注入证明持久化边界，不代表真实 Live websocket 中断。 |
| 8–11：未知 Attempt、派发区别、成交与幂等 | S27.1/S27.4 的持久未知 Reservation 与 fail-closed 证据；[S24](s24-live-order-gateway-evidence.md)、[S25](s25-4-live-manual-resolution-evidence.md)、[S26 结算](https://github.com/kaiqiangh/tradex/issues/103#issuecomment-5864907639) | 完整 Provider 外部恢复场景独立验收；未知 CANCEL 保留既有用户触发的精确订单恢复边界。 |
| 12–14：优先级与有界负载 | [S27.3](s27-3-provider-scheduling-evidence_zh.md) 的 P0–P3、并发/队列上限和账户/IP 冷却；S27.4 的真实 ProviderJob 在三个 P3 槽位占用时继续 P0 | 当前没有可合并的远程 quote/UI producer；Backend ARD §36.2 保留 producer 端要求，fixture 不证明实际 Provider 工作负载。 |
| 15：模型故障隔离 | S27.4 的五种规范故障、可信审批/准备/撤单/对账和显式 fallback；真实子进程 Gateway 回归 | 使用合成 broker 响应及模型网关投影；未进行真实 broker 写操作或真实 sidecar 与金融控制联合验收。 |
| 16–18：健康/修复、独立 Arm、键盘/布局 | S27.4 的四种 gateway 状态 × 390/768/1280px、Enter、账户健康与不自动 Arm；S27.1 原生账户恢复投影 | S33 全应用可访问性/回归和签名包生命周期独立验收。 |

仅按已命名证据更新需求清单；本审计不将任何 S27 需求提升为 VERIFIED。本地实现、真实物理生命周期、真实 Provider 证据及签名包验收分别记录。

## 原生尝试，2026-09-29

- 在审计基线和已有默认工作区运行普通 `npm run desktop`。原生构建成功，`target/debug/tradex` 在原始启动 session 中运行；未使用集成 fixture 或替代凭据路径。
- 工作区在 `2026-09-29T22:35:14.836325Z` 重开。只读 SQLite 检查显示已连接 T212 Live 账户 sequence 68，STALE/UNVERIFIED/UNCHECKED/STALE/BLOCKED/DISARMED，保留 `lastSuccessfulSync=2026-09-29T17:35:17.79051Z`。
- 约 `22:58 UTC`，只读进程采样显示两条 Provider 凭据 worker 停在 `NativeVault::get` / `SecItemCopyMatching`；未证明本次尝试产生新鲜 T212 观察。
- 桌面工具无法按开发二进制选择应用；Finder 调用耗时超过 20 分钟。工具延迟和应用选择失败均不作为产品 PASS/FAIL，未触发 OS wake 事件。
- 已请求用户切到 TradeX，在本机系统弹窗完成可能出现的 Keychain 认证，然后实际睡眠唤醒并解锁 Mac。完成必须由真实生命周期事件后的新鲜 Provider 观察证明，仅口头确认不足。

未进行 broker 下单/撤单、Arm、凭据输入、Keychain ACL 修改或直接 SQLite 写入。系统认证可用后，运行中的应用可能继续只读恢复。父项 #107 和 map #1 保持 OPEN；该门禁未完成时不启动 S28。

## 审计文档验证

基于审计基线的串行独立 Standards 与 Spec 审查均 PASS，无可执行发现。本地证据链接、14 行作用域/无 VERIFIED 提升、需求清单及 `git diff --check` 检查通过。此项仅验证文档增量，不关闭待完成的原生验收门禁，也未复跑应用测试套件。交付 SHA 记录在 issue 进度评论中。

## 后续：原生 Provider 恢复完成，2026-09-30

较早的启动以 exit code 0 结束。确认没有 TradeX 进程后，在 `dev@b0750634d5e770fc5c955b061908ed4da66578aa` 重新运行普通 `npm run desktop`。开发程序 SHA-256 为 `34fdfcd760e56a6eec53a31c4383d5f123ad63e1bef2007decb24e61afdee30d`。凭据 worker 最初停在 `SecItemCopyMatching`；桌面工具明确禁止访问 SecurityAgent。代理未更改凭据或安全权限。

只读 SQLite/outbox 检查证明真实 T212 恢复：sequence 78 在 `08:03:12.085077Z` 持久化 `SESSION_RESUMED / STALE / BLOCKED / DISARMED`；sequence 79–81 记录了更多 resume 触发。sequence 82 在 `08:03:23.245049Z` 持久化 `ONLINE / VALID / CONFIGURED`，`lastSuccessfulSync=08:03:23.244385Z`，保留 3 个持仓、0 个开放订单、6 个近期订单。sequence 83 在 `08:03:23.25236Z` 恢复 `CURRENT`，但保持 `BLOCKED / DISARMED` 并明确要求重新 Arm。之后进程采样不再出现 `SecItemCopyMatching` 等待。本次凭据/Provider 完成阻塞已解除。未进行 broker 订单写操作或 Arm。

此记录不证明真实 OS wake 或新的 `<5 s` 计时结果：存在多次 resume 触发，未测量 Provider 请求开始时间。用户报告睡眠唤醒并解锁后，约 `08:15 UTC` 捕获并于 `08:19 UTC` 复查的电源日志仍没有 2026-09-30 Sleep/Wake/DarkWake 事件；账户仍为 sequence 83，没有后续恢复事件。已有 `caffeinate -s -d -i` 进程仍持有睡眠/显示断言，代理未停止或修改该进程。已请求用户暂时停止自己的进程，实际睡眠唤醒并解锁后切到 TradeX。物理生命周期验收仍待完成；表中更广泛的证据边界分别保留。父项 #107 保持 OPEN，不启动 S28。

这五份文件的后续增量基于 `b0750634d5e770fc5c955b061908ed4da66578aa` 完成串行独立 Standards 与 Spec 审查，两个轴均为 0 发现。双语证据标记、本地链接、程序 hash、需求清单与 `git diff --check` 均通过。此次仅更新文档，未复跑应用测试套件。

## 用户明确延期验证，2026-09-30

用户明确要求跳过当前实际 OS 睡眠唤醒验证并继续 S28。本门禁记录为 `SKIPPED_BY_USER / 未验证`：不再阻止 S28 的串行调度，但不代表 PASS，也不从最终验收范围移除。父项 #107 保持 OPEN，不将任何需求提升为 VERIFIED；整张 map 验收前复核物理生命周期证据。本调度例外不授予 Arm 或真实资金交易权限。
