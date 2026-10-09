# S29.4 — Proposal 静态 Spot 规则与所需参考输入验收

单票 [#125](https://github.com/kaiqiangh/tradex/issues/125)，父项 [#121](https://github.com/kaiqiangh/tradex/issues/121)。[配对规范](s29-spot-proposal-rules-spec_zh.md)，[English](s29-spot-proposal-rules-evidence.md)。复审基线 `2368e22122b2028e92f4c3352c2f08fe80e1344a`。当前源码为规划 HEAD `1d24523158674d2763d50bee49150e8ff9d808d6` 之上的冻结工作树；运行时、测试和生成文件的逐文件哈希见 [源码检查点](evidence/s29-spot-proposal-rules/implementation-checkpoint.json)。实现验证及串行复审均 PASS；精确 commit/远端/tracker 交付仍待记录，票保持 OPEN。

## 已实现行为

公开 `trade.spot_rules.get` 和显式 CAS `trade.spot_rules.refresh` 从已存储的不可变 Proposal 推导意图、精确普通 Binance Live 账户、BTC/ETH canonical 标识、BASE/QUOTE 及来源/material/session/time/policy 绑定；renderer 不能传入 host、symbol、price 或 authority。支持原样 BASE Limit GTC/IOC/FOK、BASE Market DAY 和提供方允许的 BUY QUOTE Market。精确十进制字符串比较、取模和乘法覆盖价格、数量、按 side 百分比、适用 notional 与单订单原生资产限额；仅 PRICE_FILTER 的文档零字段可禁用其自身约束，不虚构缺失的可选 notional 或 resulting BASE。

保留 scope/origin 的逐规则 PASS/REJECT/UNAVAILABLE 进入 INSTRUMENT_RULES 和保存的 risk 输入摘要。已知违规拒绝；缺失动态订单计数、仓位容量、PRICE_RANGE 与未知活跃 schema 保留明确义务。只有全部实际适用义务已建立才允许所属规则检查 PASS；MARKET_DATA_USE、报价、账户健康/权限、费用/FX 和金融授权独立。

仅针对实际所需 purpose，通过固定普通 P3 公共 GET 读取，且不发送账户认证。非 null 主参考优先；仅合法显式 null 可回退到零区间的原始 last trade 或原始匹配区间平均价。错误、no-reference、无效和拒绝响应不选择回退。提供方原时间、首次本地接收和单调年龄分离；重复相同读取不能续期。未来/过期/区间不符/绑定变化/迟到/超时/共享限流均不能资格化。参考仅在运行时保留，持久化评估去除原始价格，只保存摘要和 captured 结果；重开不恢复当前资格。

实际 Trade 展示精确身份、单位、适用性、purpose、原区间及 provider time 与首次接收、显式刷新/恢复和 unresolved obligations。历史及 pre-arm/审批 review 展示保存评估，无参考刷新控制；换来源不改写历史。实际键盘 1280/768/390 验证无水平溢出。

## 接缝与边界

公开 React → 实际 typed Rust Control Plane → 自有临时 SQLite/outbox → 外部假 HTTP/vault；账户、来源和 Proposal 通过公开操作创建。没有正向 CP 金融/参考快照种子。HTTP fixture 仅改变外部响应；UI 使用自有 port1427 integration host、`TRADEX_SOURCE_ONLY_FIXTURE=1` 与外部规则 HTTP fixture，不修改用户桌面、工作区或真实账户。既有 Hot/Gateway 回归保留自己的历史 fixture 边界，不能作为 S29 正向金融验收。

独立规则与 Binance Hot 场景各自使用串行隔离应用进程，单场景生产 IP/UID 门禁不变；不声称全部场景共同进程的交换所公网 IP 资格。专门的跨账户/工作区配额和 418/429 共享 cooldown 场景验证实际共享边界，未重置配额。

## 验证记录

| 门禁 | 当前结果 | 证据 |
|---|---|---|
| 完整 `npm run check` | PASS：436 Rust、17 Node、39 既有 ignored；schema/build 及 203 requirements / 70 screens / 13 QA / 23 baseline files | [日志](evidence/s29-spot-proposal-rules/final-unified-check.txt) |
| 公开规则场景 | PASS：38 个独立场景，包含在完整检查中 | 同上 |
| Binance Hot | PASS：49 个串行隔离外部场景 | [日志](evidence/s29-spot-proposal-rules/binance-hot-final.txt) |
| Stock Hot | PASS：19 个外部场景 | [日志](evidence/s29-spot-proposal-rules/stock-hot-final.txt) |
| Gateway runtime | PASS：23 个真实子进程/外部假提供方场景 | [日志](evidence/s29-spot-proposal-rules/gateway-final.txt) |
| 普通桌面构建/pin | PASS；应用含匹配的 Gateway digest | [构建](evidence/s29-spot-proposal-rules/desktop-final.txt)、[配对](evidence/s29-spot-proposal-rules/desktop-build-inputs.json) |
| 最终重建 React→Rust UI | PASS：键盘1280/768/390、当前拒绝/冻结历史、captured pre-arm review，未 Arm/PLACE | [current](evidence/s29-spot-proposal-rules/ui-final.json)、[pre-arm](evidence/s29-spot-proposal-rules/ui-final-review.json)、[构建绑定](evidence/s29-spot-proposal-rules/ui-build-inputs.json)、[当前截图](evidence/s29-spot-proposal-rules/final-current.png)、[历史截图](evidence/s29-spot-proposal-rules/final-capture.png)、[390截图](evidence/s29-spot-proposal-rules/final-390.png) |
| 串行 Standards → Spec | PASS / PASS：0 条硬性违规、1 项不阻塞重复代码建议；0 项 Spec 问题 | [报告](evidence/s29-spot-proposal-rules/review-final.md) |
| 交付 SHA/远端/tracker | 待完成 | 尚未关闭 |

## RED/GREEN 与保留尝试

真实可观察公开 RED/GREEN 覆盖静态评估与 risk 拒绝、缺失覆盖、所需主参考/null 回退、Market notional/asset 与普通 forms、退休/单位、整体读取 deadline、首次接收不可续期/过期、last-trade/混合区间、未来时间、共享 418 ban、准确缺失理由、缺失可选 notional、risk 拒绝理由和区间不匹配。原日志保留在[证据目录](evidence/s29-spot-proposal-rules/)。UI RED 对应生成 reply union 缺失、窄屏溢出及 pre-arm captured 面板缺失；GREEN 使用实际 React/Rust。已有行为的公开回归不冒充新 RED。

初次 `full-check.txt` 失败于旧固定路由断言；其后的 `full-check-final.txt` 433 Rust/17 Node 通过但 PRD 插入造成行指针失败。最终完整检查修正断言及 source 行指针，保持 203 项状态与证据不变。初次并行规则全文件运行用尽未改生产共享 IP 配额，最终改用串行独立进程，并保留真实同进程配额场景；一次隔离 harness 设置尝试被主动停止，不计为通过。早期 GREEN 编译/设置尝试、30 位小数草稿拒绝 `ORDER_DECIMAL_INVALID` 和重开旧 owner 未退出导致 `WORKSPACE_BUSY` 均保留；修正后的 guard 使用合法 18 位小数且先释放旧工作区 owner，这些设置错误不作为产品 RED。

## 仍未完成的门禁

普通构建只证明编译及可执行配对，不证明原生 UI、Keychain、真实提供方、金融交易或许可。未使用真实订单、密钥或权益/许可批准。S29 父项动态容量、费用/FX、即时经认证 preflight、private lifecycle 与真实金融验收保持 OPEN；S28 金融依赖/T212 完整权限、S17 真实 Paper happy lifecycle 和 S27 实际 OS sleep/wake 均保持 OPEN。实际睡眠验证仅按调度跳过，未豁免。全局需求状态不因来源测试提高。原型代码未改，main 未改；全图在 dev 验证完成前不提交/合并 main。

存储文本日志仅去除行尾空白及多余末尾空行，命令内容和结果不变；原始与存储 SHA-256 见[日志格式记录](evidence/s29-spot-proposal-rules/log-format.json)。
