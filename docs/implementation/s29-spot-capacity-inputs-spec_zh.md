# S29.5 — 精确账户开放订单与容量输入

父项：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。前项[Proposal 静态规则/参考输入](https://github.com/kaiqiangh/tradex/issues/125)已 CLOSED。规划/实现起始基线 `dev@4f06df633832aa2ed89bf675bcb219a15c41731d`。[English](s29-spot-capacity-inputs-spec.md)。

当前单票：[提供 Binance 精确账户开放订单与容量输入](https://github.com/kaiqiangh/tradex/issues/126)，已由kaiqiangh领取，ready-for-agent。原生父项121、已完成依赖125；实时 open blockers0。

[已发布规范](https://github.com/kaiqiangh/tradex/issues/121#issuecomment-6075145431)。仅完成规划，尚未开始实现/验收。

## 问题

Trade 已解释缺失的动态订单计数/仓位义务，却没有可见的当前、有界、完整账户输入。单标的订单、旧余额或区间 rate-limit 计数不能建立交易所级开放订单容量或不可变 Proposal 的 BASE 敞口。用户需要先看到观测、覆盖与剩余资格缺口。

## 方案

完成一个只读纵向切片：选中既有普通 Binance Live BTC/USDT 或 ETH/USDT Proposal，通过显式刷新和固定认证 GET，仅收集其实际所需的精确账户容量输入，在 Trade 的 current/captured review 显示 typed inventory/count/position 观测、原始来源和未完成完整性。本前置来源票不资格化动态金融规则、资金、Arm、同意或派发；后续 owning evaluator/preflight 须建立执行资格，不引入正向 authority 种子。

## 用户故事

1. 作为用户，我希望输入绑定保存的不可变 Proposal 与精确账户，以免借用另一账户库存。
2. 作为用户，我希望实际 active scoped rules 推导所需输入，以免读取无关账户数据。
3. 作为用户，我希望区分单标的与全交易所计数，以免忽略其他标的订单。
4. 作为用户，我希望区间 unfilled counter 与 open-order filter inventory 分离，以免错用 API 计数。
5. 作为用户，我希望普通/algo/iceberg/list 的实际覆盖可见，以免未知分类被当作零。
6. 作为用户，我希望 list 身份与未见 pending legs 明确，以免外部高级订单隐藏容量占用。
7. 作为用户，我希望原始 BASE free/locked 与 open BUY 敞口分开，以免混淆持有与待买。
8. 作为用户，我希望保留相关 orig/executed quantity，以免部分成交被转换成猜测容量。
9. 作为用户，我希望外部 symbol/asset 的归属不确定性可见，以免 ticker 前缀虚构身份。
10. 作为用户，我希望响应覆盖与边界可见，以免截断、重复、无效库存被称完整。
11. 作为用户，我希望 provider update time 与 collection receipt 分开，以免最后状态变更冒充快照新鲜度。
12. 作为用户，我希望多个 HTTP 观测标明非原子，以免接收时间靠近冒充提供方快照保证。
13. 作为用户，我希望显式刷新和脱敏恢复，以使 auth/quota/deadline/缺失退休当前输入并保留解释。
14. 作为用户，我希望账户/凭据/来源/Proposal/session/time 变化拒绝迟到结果，以免挂到新意图。
15. 作为用户，我希望 captured review 保留结果/摘要且无原始私人库存，以使历史不冒充当前重读。
16. 作为用户，我希望键盘/窄屏展示单位、未资格化和独立门禁，以免观测容量冒充批准。
17. 作为审查者，我希望真实公开 RED/GREEN、精确字节本地/构建/UI/串行复审证据，以免隐藏快照种子关闭来源票或父项。

## 实现决定

- 沿用已确认的单票公开 React → 实际 typed Rust CP → 自有临时 SQLite/outbox → 外部假 vault/HTTP/WS 接缝。复用不可变 Proposal、精确账户/密钥所有权、当前规则元数据、可信时间、共享读调度/配额及 current/captured Trade 解释。不增加账户配置、renderer credential/host/symbol/count/balance 输入、后台扫描或独立金融 authority 模型。
- 新增受信 `trade.spot_capacity.get` / 显式 CAS `trade.spot_capacity.refresh`，仅识别 workspace/已存 Proposal/current state。后端推导实际 purpose、精确账户/remote UID/credential version、BTC/ETH/USDT、rule scope/material 与 workspace/source/policy/session/time 绑定。无实际所需 purpose 时不做无关私人读取。有界并发所有权/slots/generation/deadline，在发布前拒绝 foreign/变化状态。
- 仅固定普通 Binance USER_DATA 认证 GET，复用签名/vault/zeroization。account 提供精确身份和相关 BASE free/locked；交易所级或共享资产覆盖须全账户 openOrders，单标的子集不能替代完整性。仅实际 list/leg 完整性需要时读取 openOrderList。提议成本 clock2/account20/full openOrders80/openOrderList6，实现前再次核对官方契约。使用实际 signed-account UID/IP 配额，不重置，不虚构 public UID；共享 cooldown 延迟、剩余 deadline 及 vault/network I/O 不持 CP 全局锁。
- 区分 interval unfilled counter、请求/ORDERS rate limits、open-order filter inventory 和本地 reservation。rateLimit/order 不是 open inventory 替代，本票不调用；不进行 order test/probe、PLACE/CANCEL/amend/list 创建、listen-key mutation 或金融授权。
- 投影前严格解析有界原始 JSON 类型/重复字段、UID/symbol/order/list 身份、精确 decimal、status/type/side/iceberg 与时间。提议硬边界：每响应512KiB、open-order1000行、open-list256行、balance1024行；超过边界须拒绝/明确不完整，不能截断。oversized/ambiguous/duplicate/non-open/malformed 不得伪装完整；未知 active fields/status/classification 保留 unresolved，不能默认零/普通。int64 identity/count 跨 IPC 保留精确字符串。
- 仅投影有界 coverage/counts 与所选 BASE balance/exposure，不投影 raw private inventory。区分确切观测、保守 bound 与资产归属不可用；不推断外部 symbol BASE、不假设 USDT/USD、不猜高级 list pending leg 或 remaining quantity。部分成交 open BUY 语义不确定保持明确，待 owning execution 契约；不能用仅所选标的 BUY 订单认证所有共享 BASE 的 MAX_POSITION。
- 完整返回数组只证明其自身 endpoint 契约下的有界响应完整性。account/orders/lists 无此处已文档化的共同原子 watermark：展示读取窗口、真实 account/order/list update times、缺失的 aggregate provider observation time 与 READ_ONLY_SPOT_CAPACITY 质量。provider clock sample 仅是时间观测；不能把 last update 改写成当前快照时间或认证原子容量。list/leg/identity/time 矛盾保留 unresolved。
- get/cached history 不更新接收时间；一次实际、显式、认证重读创建独立 collection observation，保持该观测的首次 receipts 与 material digest。过期、account/credential/source/rule/market/policy/Proposal/workspace/session/clock 变化退休当前资格；失败刷新清除当前输入，仅保留脱敏最后解释。重开只恢复配置/历史，不隐式私人收集。
- 本来源票即使输入完整，owning dynamic financial evaluation 仍为 UNAVAILABLE/DYNAMIC_INPUTS_NOT_EXECUTION_QUALIFIED。不升级 MARKET_DATA_USE/rights、key permissions/health、quote/liquidity、fees/FX、funding/reservations、Arm、同意或即时 Gateway preflight。保护独立 exact CANCEL/reconciliation；后续同步契约再区分普通意图的实际适用容量和认证执行资格，不使用隐藏正向快照预解。
- current/captured UI 展示身份、所需 purpose、单位、确切/不完整 coverage/classification、非原子质量、原时间/receipts、age/failure 和显式只读恢复。risk/review 历史仅保存允许的有界 aggregate observations/digests，不保存 raw account/balance/order/list 响应或运行时缓存；captured 无刷新/poll 控制。实现中同步英文/中文金融、IPC、交互契约。

## 测试决定

- 先取得真实公开命令/UI RED 再 GREEN。通过公开操作创建账户/来源/Proposal，仅外部 vault/HTTP 为 fake；复用实际 React/Rust/source-only host，不添加正向 CP capacity/reference setters 或来源快照种子。
- 覆盖精确 scope/UID/key/signature/原字段、完整/空/外标的 inventory、duplicate/数组或body过界、错误 integer/decimal/boolean 类型、未知 status/type/classification、list/pending legs 矛盾、部分成交、foreign/missing asset attribution、原时间；验证不必要读为零，interval counter 不替代 open inventory；每个正向来源场景金融规则资格仍不可用。
- 覆盖 account/credential/source/rule/Proposal/policy/workspace/session/time 变化、失败刷新/过期/重开、迟到/deadline/cancellation、共享 UID/IP 和418/429 cooldown，不重置。断言 I/O 不持 CP 锁、无 order probe/mutation、raw private 持久化、overall approval/reservation。
- 实际 Trade/current/captured review、显式刷新/失败恢复及键盘1280/768/390无溢出；换来源不改历史。受影响 source/static-reference/Hot/Gateway 回归、完整本地检查、IPC/双语manifest、普通桌面构建/pin及 Standards→Spec 独立串行复审通过；记录精确字节 source/remote/tracker 与独立 native/hosted/financial/prototype 边界。

## 范围外

动态金融资格化、order rate-limit 容量资格、新 metadata universe 扫描/外资产交易、猜测 partial-fill/list 语义、fees/FX/funding、即时认证 Gateway preflight、private lifecycle/mutation/真实交易、许可授予、Testnet/Bitget/S28 范围变化、物理睡眠豁免、原型改动及 main 合并。完整 goal 所需的内容仍由后续 owning 工作负责。

## 补充说明

2026-10-09 核对官方：[Spot filters](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/filters.md)、[REST账户/open orders/open lists/unfilled count](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/rest-api.md)、[unfilled count规则](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/faqs/order_count_decrement.md)。它们区分 filter count、BASE balance/open BUY、endpoint scope/cost 与 interval counter；把组合读取标为非原子是本规范保守设计，非声称提供方保证。

仅规划这一前置来源票，沿用已确认单票/接缝。实际依赖为已完成的 Proposal/source explanation 契约，不虚构 S28/物理生命周期调度边。S29 父项、S28/S17/物理S27及全图发布门禁均 OPEN。规划不代表运行时交付。
