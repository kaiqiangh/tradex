# S28 所需路由与只读 FX 证据

实现票：[读取所需 FX 路由与 Alpaca 汇率证据](https://github.com/kaiqiangh/tradex/issues/120)。审查基线：`dev@d7ea48d1e5a5c2a5f95cc3937b02aa0202bc53d4`。配对报告：[English](s28-fx-evidence.md)。

## 交付边界

本切片推导实际货币需求并提供有界只读 FX 上下文，不授予换汇或金融执行资格。当前实现/原生/UI 验证已完成，包括余额路由及原始捕获时间整改；独立串行 Standards PASS0硬违规/1非阻塞建议、Spec PASS0确认问题；dev 证据交付仍待完成。初始/token 整改证据保留为历史。[验收审计](evidence/s28-fx/acceptance-audit.md)映射全部12项验收条件；[源码检查点](evidence/s28-fx/implementation-checkpoint.json)记录精确文件哈希及当前限制。

显式选择已有 Alpaca PAPER 来源，提供版本化 Save/Refresh/Disconnect/reload 及仅恢复元数据的重开。Save 不请求提供方；断开保留借用的账户和密钥。工作区、精确账户、已观察余额单位、金额消费者与不可变意图决定实际路由。BALANCE_WORKSPACE 保留已知钱包单位，不推断未知账户主 fiat 币种；共用 Portfolio 的2–16位大写字母/数字验证，包括不支持的 USDT/OP/1INCH。同币无需汇率，未知币种保持未知，USDT 与 USD 独立。Spot BUY 仅从实际观察到的匹配 quote asset 确定资金单位；SELL 不附加 BUY 资金换汇路由。Local Paper 与保护性 CANCEL 保留既有语义。

仅实际需要的 EURUSD/USDEUR 进入固定、经认证的最新汇率 GET。保留原始数字 token、精确十进制、bid/ask 方向、独立 provider mid 及原始 provider/receipt 时间。来源、账户、凭据、工作区、进程会话、可信时钟、序号或材料变化，以及 provider/receipt 任一独立过期，均撤销当前资格。原始 token 类型验证阻止 arbitrary_precision 将对象冒充数字；金融值不经过 f64 往返。

Settings、Portfolio、Trade 展示当前需求、选定来源、原始汇率/时间/质量及独立阻塞。pre-arm 与不可变审批窗口显示原始捕获日期时间，过期后仍与捕获证据一起保持冻结，不因轮询刷新原同意。读取观察不证明执行级 FX、券商资金/换汇费用或完整金额输入；实际需要但不支持的 FX 在 risk/Prepare/dispatch 保持不可用。

## 验证

- 公开接口真实 RED/GREEN 覆盖来源生命周期、需求、精确生产者、同币独立性、不可变意图、历史组合 fixture 隔离、捕获同意/材料、Spot 资金单位、SELL 独立性、原始 token 拒绝及实际钱包路由/支持的读取。保留三次历史 Spec P2 NOT PASS 与实际复现。
- 当前 `npm run check`：17Node/386普通Rust、生成 schema/构建及需求追溯 PASS；integration425Rust PASS；独立 Gateway23PASS；格式与空白 PASS。39项既有忽略测试仍未验证。错误的完整库 Gateway feature 组合命令保留为历史失败，与正确分层门禁分开记录。
- 真实 Rust-backed UI 验证390/768/1280px Settings/捕获窗口、键盘与未保存控制、最长64字符 bid/ask、实际 Proposal 反向路由、过期后的冻结窗口、外部 fixture403/error/Reload 及仅恢复元数据的重开。控制台无错误/警告，无 Arm、金融审批或提供方写入。Fixture UI 不冒充真实上游验收。
- 最终2026-10-07捕获时间修复后的普通原生构建用已有密钥读取后返回明确的端点访问拒绝，没有首个 receipt/汇率，控制已恢复。HTTP403 是基于审查过的映射推断，没有归档原始状态/正文。最终普通主程序/Gateway 固定哈希、实际父子关系及20项 runtime 输入与审查源码一致。另保留初始构建/拒绝证据。当前 capture-time-green 验证原始日期时间冻结，控制台无问题。不归档凭据、账户标识、余额、持仓或提供方正文。

## 剩余金融验收

所需执行级 FX、允许用途/授权、精确券商费用与完整金额输入仍不可用。日历读取及已知公司行为/精确账户元数据读取不证明完整行为/历史调整、权威身份、当前停牌/可交易性、完整密钥权限、SIP 或执行场所。报价及 Trading212 验收父项保持 OPEN。用户跳过的 S17 真实 Paper 与 S27 实际 OS 睡眠唤醒仍未验证。本切片不提交/合并 main PR，也不完成整张 map。

源码提交：`e82b65e396e91560debae914d407577b62a76db0`.
