# S29.3 — Binance Spot Hot 报价与连续深度来源

父项：[验证 Binance Spot Live 可信执行](https://github.com/kaiqiangh/tradex/issues/121)。实现/复审起点：dev@5edf18d01122d1871b6c94aeafefe708435b7f38。前票 #123 规则来源/准入已 CLOSED；S29 保持 OPEN。此规范推进父项故事 9–12、27，不移除完整执行及真实外部验收要求。[English](s29-binance-hot-source-spec.md)。

当前单实现票：[提供 Binance Spot Hot 报价与连续深度来源](https://github.com/kaiqiangh/tradex/issues/124)。已领取、ready-for-agent、IN_PROGRESS。已实现选择和首个真实 loopback HTTP/WS 生产者；其余传输/生命周期、受保护消费者、UI 以及完整验收/复审/交付仍须完成。不表示当前票或父项关闭。

## 问题

普通 Binance 账户和精确 Spot 规则来源不能提供当前买卖盘流动性。生产 Hot 来源仅处理 Alpaca 股票。账户连接、最后成交价、公开 bookTicker 或未对齐的 REST 快照不能构成有提供方时间且连续的盘口。用户需要精确交易所/来源出处，以及明确生命周期、故障和权利状态，才能供后续金融检查消费真实流动性证据。

## 方案

将普通 Binance 公开 Spot 行情来源与执行凭据独立配置。可见 Hot BTC/USDT 或 ETH/USDT 视图取得一个有界、版本化租约，把提供方深度快照与缓冲实时增量对齐，维护可证实连续的已知盘口，投影精确价格、BASE 数量、覆盖价格带、原始提供方事件时间及首次 TradeX 接收时间。技术采集与用户特定允许的数据使用分开；断序或来源/时间/生命周期变化使证据失效，并通过明确有界操作恢复。复用类型化 Control Plane、来源目录、Hot 归属和受保护消费者。

## 用户故事

1. 用户可保存普通 Binance Spot 行情来源，避免静默转向 Testnet 或其他主机。
2. 来源设置与执行凭据分开，公开行情不借用或传输交易密钥。
3. Save/Disconnect 不发网络请求，选择来源不隐式订阅。
4. 明确 canonical BTC/USDT、ETH/USDT 覆盖，不支持资产保持不可用。
5. 可见 Hot 视图拥有一个有界租约，不向后台全市场订阅 tick。
6. 区分 socket 已连接与连续盘口可用，握手不直接生成报价。
7. 独立保留提供方事件时间、接收时间和更新身份，缓存不能制造新鲜度。
8. 显示精确 bid/ask 与 BASE 数量，不把报价金额虚构为 USD。
9. 显示有界深度和已知覆盖价格带，不把初始部分快照当全盘口。
10. 缺口、畸形/交叉盘口与时间故障撤销可用证据，后续检查失败关闭。
11. 重复/旧增量不更新旧材料的新鲜度，重放不能续期同意。
12. 断连/重连完成新代次 bootstrap 后才能显示当前流动性。
13. stop、导航、隐藏视图、来源变更与关闭工作区释放归属，过时 worker 不向别的上下文发布。
14. 明确重试上限、冷却与终止访问故障，不反复请求被拒端点。
15. 实时技术延迟与允许使用、保留、再分发、商业/地区资格分开，公开访问不隐含许可。
16. 来源代次和不可变报价/深度身份进入后端 review，材料变化使旧证据失效。
17. 所属报价检查消费真实 Binance 出处，不要求虚构最后成交价。
18. crypto 交易所/准入消费当前准确账户规则，不伪造股票日历。
19. 规则、账户健康、费用/FX、允许使用、Arm、审批和 Gateway 临发预检独立，行情访问不授权下单。
20. Settings/Markets/Trade 的恢复与深度证据支持键盘及窄屏。
21. 实际 React/Rust 与外部 HTTP/WS 证据绑定源码，合成金融快照不算生产者验收。
22. 普通原生/真实提供方/金融/物理睡眠门禁单独标记，本地来源交付不关闭完整父项。

## 实现决策

- 沿用已接受的单票串行、公开 UI → 真实类型化 Control Plane → 临时 SQLite/outbox → 外部假 vault/HTTP/WS 边界和 Hot acquire/get/release 归属模式。起始 dev SHA 为固定复审基线，不重复询问已确认边界/粒度。
- 来源目录/Settings 增加普通 Binance 公开行情选择，CAS 持久化选择；临时盘口/租约与金融权威不随重开恢复。公开读取不传交易凭据，保持 Alpaca 来源、coverage、密钥生命周期和股票行为正确。
- 固定普通 `api.binance.com` REST、`stream.binance.com:9443` WSS 和精确 registry 标的。无页面自选 URL、替代地区主机、Testnet 或 market-data-only 主机回退。公开 depth/time 读取遵守既有 P3、共享 IP、响应大小/期限/重定向/冷却边界；不包含变更或账户私有流端点。
- 一个 raw 精确标的 diff-depth 流及有界 REST bootstrap。按当前官方 snapshot/buffer 桥接确认后才可用；后续允许重叠、丢弃旧增量、缺范围则失败关闭，零数量删除层级。整数 ID 无损保留，十进制精确原始字符串，不经 f64 或舍入。
- 初始 snapshot 每侧 limit1000；保留其及连续增量建立的完整已知价格带。最多每侧5000已知层级，bootstrap 缓冲最多256帧/4MiB，单帧512KiB。容量耗尽、价格带耗尽或任一最佳侧未证实则撤销盘口、有界重新同步，不静默丢层却声称完整。投影最佳两侧、每侧最多20已知层级及覆盖边界；不证明全盘口流动性。
- 总连接/bootstrap 期限有界、及时检查取消，I/O/解析不占全局 Control Plane 锁。重连最多3次且退避；被拒、封禁、限流和不支持分别呈现。遵守提供方控制消息上限、复制 ping 负载 pong、计划/实际关闭；新 socket/bootstrap 在新可用报价前建立新代次。
- 材料有独立首次接收/提供方事件时间和不可变身份；重复/旧/相同材料与缓存读取不续期。流 cursor/健康可单独推进。新鲜度使用可信提供方/墙钟/单调时钟；来源投影不超过30秒，金融消费者使用更严格已配置阈值。无事件时间的 REST/bookTicker/partial-depth 不能用本地接收时间代替。
- Binance 出处记录规范标的/BASE/QUOTE、准确 BINANCE venue、来源/连接/session/time 代次、材料更新 ID、接收/事件时间、有界已知深度、当前连续性/质量。不虚构 last trade、加权/参考价、股票日历/公司行为完整性、全盘口、稳定币平价或 quote age。
- 可见当前 Trade Proposal 和 Markets 沿同一归属生命周期取得自己的 Hot 租约；从 Markets 导航不能让 Trade 借用已释放/过期租约。两者使用同一后端生产者投影和捕获边界。真实流动性只可满足所属技术 coverage/freshness/spread/displayed-depth 检查；缺权利或其他义务不能形成整体金融 PASS。已配置 last-trade 偏离检查缺真实参考则不可用。规则/权限/交易所负项复用 #123 当前准确账户证据，不当成完整逐单规则。数量遍历、费用/FX 和派发归后续所属解释/预检工作。
- 来源政策明确公开技术访问、实时语义、内存/短缓冲、官方/条款 URL 与审阅日期。未确立的用户特定金融使用、再分发/商业权利、允许保留和地区资格保持 UNVERIFIED；勾选或读取成功不升级 VERIFIED。不得建立超出已确立权利的持久化原始 tick/book 缓存或导出，不代用户接受协议。
- 同步双语金融/wire/UI 契约。历史合成回归保持隔离，不直接播种新正向报价/规则/金融权威快照。

## 测试决策

使用已确认最高公开边界，只在外部替身。先真实公开来源选择 RED，再实现 GREEN，随后逐行为推进。验证 CAS/无 I/O/重开、真实 HTTP+WS bootstrap 与报价、原始无损类型、覆盖价格带/删除/重叠/重复/缺口、错误来源/标的/环境、未来/过期提供方/接收/单调时间、缓存不续期、乱序/过时完成、资源上限、ping/close/拒绝/冷却/有界恢复、释放/导航/工作区/来源退休及精确消费者绑定。传输/生命周期相关时使用真实 loopback socket。实际 React Settings/Markets/Trade 对接 Rust 及外部模拟响应，键盘与1280/768/390验证；不播种 CP 金融快照/权威。生成 schema/typecheck、完整本地检查、相关 Hot/Gateway 回归及普通桌面构建，最后独立 Standards → Spec 串行复审，并记录源码/证据/远端及 tracker 收口。

## 此票范围外

Live PLACE/账户私有流、完整逐单 filter/reference-price/FX/费用、全盘口流动性、历史 bars、常驻 Warm/Census tick、购买权利/地区绕过、真实金融验收、Bitget S30、S28/S17/物理 S27 免除、原型修复和 main 操作不属于此来源票；属于完整目标的要求仍由后续父项/发布工作完成。

## 后续说明

2026-10-08 核对：[REST depth](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/rest-api/market)、[行情流 schema](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~)、[官方连续性步骤](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/web-socket-streams.md)、[Spot 条款入口](https://raw.githubusercontent.com/binance/binance-spot-api-docs/master/PROD-TERMS-OF-USE.md)、[条款页面](https://www.binance.com/en/terms)。当前嵌入[生效2026-07-21的 ADGM 文本](https://bin.bnbstatic.com/static/cms/cg08ou2ak0tn7mcplvfg/file/bf4879710c904b991848972ec4818ba2cf9e4ce314c09adae84fa2750d3477f7.pdf)，Local Terms 可补充；本核对不认定用户的协议、行情数据允许使用或地区资格。

完整父项 #121 仍是目标。下一实现为一个完整纵向来源切片，无开放技术阻塞；准确交易所/消费者检查复用已完成规则/准入契约，不并行推进其他实现票。
