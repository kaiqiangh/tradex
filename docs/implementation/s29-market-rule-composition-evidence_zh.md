# S29.11 本地 BASE MARKET 规则组合——有界实现证据

**2026-10-19 · dev · 本地合成验证。** 用户以“批准，继续”批准[已评审范围](s29-instrument-rules-prepare-reachability-proposal_zh.md)。契约提交 `e07240d` 在运行时修改前同步 PRD §75、Backend §41.62、Frontend §13.49、UI §14.46。本工作仍属于[同一 owning ticket #135](https://github.com/kaiqiangh/tradex/issues/135)，十五项 AC 保留原勾选状态。完整 owning、provider、原生及 Wayfinder 验收仍开放。

## 已实现行为

- RiskDecision 在本地组合当前准确绑定 Proposal、账户、标的及来源的规则、容量和区间观察。独立来源查询保留 source-only 结果；评估不新增 provider/vault I/O。
- BUY MAX_POSITION 仅支持完整空账户库存，精确比较 free+locked+请求 BASE。SELL 与新建 PLACE 的改单义务按用途不适用。非空、部分成交或跨资产归属歧义保持不可用。
- 总订单数加一；完整 algo/iceberg/list 分类按已批准的保守零增量策略比较。准确超限拒绝；未知规则/字段/STP、缺覆盖或过期绑定不能通过。既有区间准入参与结果，保留来源限制。
- 可选且有界的比较/组合字段兼容旧捕获；摘要绑定所消费版本、比较、限额及固定策略。共享容量覆盖判断使 owning 准入和具体比较保持一致。
- 审批消费同一完整 MARKET 组合及已验证的当前 Spot 报价，修复旧通用 tradability 消费者不识别该来源的问题；独立来源可用不能直接授予资格。金融、权限、健康、Arm、同意、资金及派发检查仍独立。
- 当前/捕获/pre-arm 界面区分来源查询和组合评估，显示后端精确比较、版本与保留限制，并说明交易所执行时 PRICE_RANGE。受约束 LIMIT 保持不可用。刷新及重启保留历史捕获，不恢复当前资格。

## 验证

| 检查 | 结果与证据 |
| --- | --- |
| 公开 producer 到 risk TDD | 初始语义 [RED](evidence/s29-market-rule-composition/first-red.txt)；最终 [12 项核心场景](evidence/s29-market-rule-composition/core-final.txt)通过：BUY/SELL、精确仓位/计数/分类/范围、配额/STP、歧义、未知规则、不变 LIMIT、CAS/刷新/重启。 |
| 公开 producer → 审批 → Prepare | [两项隔离场景](evidence/s29-market-rule-composition/prepare-atomic.txt)通过：正常公开输入签发审批，原子预留 USDT 本金与 BNB 估算；重放仅消费一次；审批后库存不完整时拒绝 Prepare，不遗留部分 execution/reservation/consumption 行或 outbox 事件。最终 Hot 全集再次验证。 |
| 共享 wire | [三项测试](evidence/s29-market-rule-composition/wire.txt)通过：旧行可缺省、精确有界字符串、原因/限制/版本边界，以及拒绝 authority 字段。 |
| React → Rust → 隔离 SQLite | [最终结果](evidence/s29-market-rule-composition/ui-green.json)：当前/捕获/pre-arm，1280/768/390 六项溢出检查通过；禁止后端命令、页面错误、控制台错误及外部浏览器请求均为零；账户保持未 Arm，未签发审批。六张 PNG 是实际可滚动界面的视口截图。[UI RED](evidence/s29-market-rule-composition/ui-red.txt)先于实现。 |
| 默认配置全量回归 | [`npm run check`](evidence/s29-market-rule-composition/full-check.txt)通过：协议一致性、生产构建、38 项前端测试、Rust 汇总 591 通过和 33 项既有 ignored、日志检查、203 需求/70 屏幕/13 QA 场景/23 基线文件。ignored 或按 feature 排除的场景不构成验收。 |
| 特性全量回归 | [`npm run test:binance-hot`](evidence/s29-market-rule-composition/hot-final.txt)：64 项场景按独立进程顺序通过，包含两项 owning Prepare 场景。 |
| 独立审查 | 串行 [Standards](evidence/s29-market-rule-composition/standards-review.md)：无硬性违规，一项重复判断问题已修复；随后 [Spec](evidence/s29-market-rule-composition/spec-review.md)：无发现。 |
| 原生桌面检查 | [`cargo check -p tradex --features desktop --bin tradex`](evidence/s29-market-rule-composition/desktop-check.txt)受阻，exit 101：宿主缺少 pkg-config `glib-2.0 >= 2.70`。原生构建未验证。 |

文本回执仅规范化终端行末空白。命令使用 Rust 1.98.1、离线 Cargo、dev/test 关闭 debug information、单 Rust 测试线程及 16 MiB 测试栈。新增公开接缝测试仅替换 HTTP/vault/WS/clock 外部生产者，不 seed 应用金融 authority。未执行真实 provider 金融操作。合成 Alpaca Support 上下文不证明用户实际来源用途或 provider entitlement。

## 交接

继续同一 owning ticket；原生、真实来源激活及验收仍待完成。未升级需求状态或 AC 勾选，未创建 main PR，未使用 worktree。GitHub CLI issue-comment 写入返回 `Forbidden`；[本地决议记录](evidence/s29-market-rule-composition/decision-note.md)保留批准范围，不声称已发布 tracker。[需求与工作流快照](wayfinder-resume-2026-10-19.md)保留历史清单，本记录补充其后的实现进展。
