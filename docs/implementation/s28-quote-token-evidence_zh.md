# S28 原始报价数字 token 修复

当前票：[接入已配置 Alpaca 报价与 Hot 订阅](https://github.com/kaiqiangh/tradex/issues/116)。审查起点：dev@213d63953187439cce366faa327d153f3c19bb1d。配对：[English](s28-quote-token-evidence.md)。

公开HTTP market.get复现对象类型bid通过serde_json arbitrary_precision私有Number标记成为AVAILABLE/HEALTHY快照。单独的真实loopback WebSocket/公开Hot路径复现同类错误类型成为STREAMING快照；该Hot RED时钟仍CLOCK_UNCERTAIN，不能证明金融资格。两次真实失败均保留。

HTTP在Value反序列化前检查bp/ap/bs/as原始RawValue；Hot保留原始帧token，在重序列化/发布前共用该边界，不能让此前Value类型转换抹去原始类型证据。精确十进制、报价方向、数量单位、时间、材料绑定、额度/取消和来源/金融门禁不变；不增加端点、schema、权限或规范契约。市场/账户/FX/权限等仍失败关闭。

公开命令回归覆盖四字段、普通与转义标记共8HTTP/8Hot对象用例；既有有效精确数字及无效时间/条件/订阅/传输用例通过。相关35测试通过。完整检查17Node/387普通Rust、schema/类型/构建/追溯通过；integration426Rust；实际独立Gateway23通过。39忽略项仍未验证。Rust用例串行执行，专门竞态/socket内部并发仍保留；格式/空白通过。首个Gateway目标运行0测试，仅保留为未覆盖尝试，不算PASS。

真实隔离Rust界面通过已选feed键盘/未保存控制，以及实际Hot报价/来源在390/768/1280显示，控制台错误/警告0。browser-hot截图/JSON为假提供方证据。两次旧按需UI尝试不证明该路径：一次环境开启Hot；另一次模式刻意拒绝market.hot.acquire而当前已配置详情要求Hot。保留为场景/覆盖限制，不记产品RED或UI通过。HTTP正向行为由真实公开生产者接口验证。

普通桌面重建已通过，编译Gateway固定哈希/本地签名/实际父子关系及75项构建输入一致；本人完成Keychain认证后，当前普通构建明确确认已有IEX源认证技术读取通过。首次尝试以STATE_CURSOR_EXPIRED结束；使用公开Reload quote source后重新Verify selected feed得到成功终态，仅证明技术访问。界面操作后的进程采样无匹配进程；此前实际父子关系保留独立时间，不改为当前进程声明。native-read.json记录脱敏结果；75项输入及两个二进制哈希仍一致。独立串行Standards通过（0缺陷/建议），Spec对本次修复通过（0缺陷/范围扩张），但原票Spec仍NOT PASS且保留1项独立正向验收缺口。修复源码已提交953cbacc99d3f0662dba69ba1baf0cf14f8bef71；冻结证据已在a3370be7d510d392bf9b18229ce1aca7523f96c3正常提交/推送，23项冻结文件哈希与远端dev核对通过；后续delivery.json记录此交付完成，checkpoint.json故意保留此前证据推送前状态。此前213d639账户恢复是旧二进制的历史证据，不改成当前修复结果。原AC4正向签发审批→Prepare→Gateway仍未通过，116/115/113/117/map保持OPEN；无真实金融操作、mainPR/merge或总map完成。

[当前检查点](evidence/s28-quote-token/checkpoint.json)记录明细；Standards首次因用量限制未能执行；最新只读查询允许普通工作后，原审查代理完成。重复重试已中断，不使用其报告。复审不重复运行检查；双轴报告分别归档，不声称托管CI或整票交付PASS。75项普通构建输入仍绑定当前源码提交，未重复构建未变化源码。

后续普通构建真实Hot跟进基线dev@88ff1a5：公开Back to results→AAPL重新获取过期租约。IEX认证/订阅及实际提供方报价到达STREAMING，随后流返回PROVIDER_RESPONSE_INVALID，保留报价变为REALTIME / STALE。同一签名二进制、75输入及新采样的实际父子关系一致。这补充真实生产者证据，不证明稳定ready、合并覆盖、授权/许可或正向金融AC4；未捕获原始无效帧/具体原因，不据通用错误推断解析缺陷。见[后续审计](evidence/s28-quote-token/native-hot-followup.md)及脱敏JSON。独立串行Standards PASS0缺陷/建议；Spec审计PASS0缺陷/扩张，但原票NOTPASS1正向AC4缺口。本段保留提交前观测；后续限定dev交付记于原票进度评论。此前快照保留各自历史状态。
