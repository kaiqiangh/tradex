# S25.1 #97 Trading 212 Live 未知提交对账验收证据

日期：2026-09-27。分支：`dev`。审查基线：`ad9e17fdfac4efe7cd71717e7bf5957c44207cc1`。

## 范围

本切片仅覆盖已保存的 Trading 212 Live `PLACE` attempt、其准确账户及不可变意图。Control Plane 负责提供方身份、五分钟可信时间窗口、只读查询，以及持久化证据台账/outbox。公开 renderer 契约只传递 workspace、attempt、account 和 attempt 状态版本身份，不接受提供方 URL 或凭据。对账不会重试或重发订单；候选订单不会自动绑定，也不会被当作不存在订单的证明。

后续 Binance/Bitget 对账切片和收窄后的人工处置记录在已关闭的 S25 父项 [#96](https://github.com/kaiqiangh/tradex/issues/96#issuecomment-5863761675) 中。本证据仍仅覆盖 Trading 212 对账；provider-hosted 验收和点击式原型不属于此本地切片。

## 验证

- `npm run check` **PASS**：生成的 IPC schema 一致性、TypeScript 检查和 Vite 构建、13 项 Node 单测、`cargo test --workspace`（202 项 Rust 库测试及默认集成目标），以及需求追踪检查（203 条需求、70 个页面、13 个 QA 场景、23 个基线文件）。Vite 现有大 chunk 提示仅为信息性提示。
- `cargo fmt --all -- --check`、`git diff --check` 和 `node --check tests/live-approval-ui.mjs` **PASS**。schema v6/v8 迁移夹具会在重放旧迁移链之前移除仅属于 v29 的证据表；future-schema 夹具现使用版本 30。
- Rust-backed 浏览器 fixture **PASS**，使用合成 Trading 212 Live 账户和本地假提供方。`PLACE` 结果进入 `UNKNOWN_RECONCILING`；只读对账持久化 `INCONCLUSIVE` 证据，候选列表为空，并明确表达“未观察到相似订单不代表已证明订单不存在”。离开后重新打开 Order Drafts，attempt 和证据恢复，reservation 仍为 `ACTIVE`；账户保持 `DISARMED` 且执行被阻止。本地 Gateway 日志在刷新或导航期间没有新增提交 `POST`。
- 过期窗口 fixture 确认可信的五分钟边界：attempt 继续保持 `UNKNOWN_RECONCILING`，reservation 保持 `ACTIVE`，仅允许 `KEEP_RECONCILING`。窗口过期后的刷新在访问提供方之前被拒绝。持久 Keep 操作只接受后端授权的该决策，拒绝 `CONFIRMED_NOT_SUBMITTED`，保持 attempt/reservation 不变，并在重新打开 workspace 后恢复审计记录。
- deadline-tick 回归测试将可信时间推进到已保存 attempt 截止时间之后，并在不查询证据页面的情况下调用原生 expiry pass。它独立地将账户置为 `STALE`/`DISARMED`，同时保留未知 attempt 和 active reservation。
- 证据区域在 1280、768 和 390 CSS px 下均完整显示且无横向溢出。页面测得宽度分别为 1265、753 和 375 px；证据面板完整位于各视口范围内且本地无溢出。390 px 下用 Tab 验证了相邻控件的键盘焦点；证据通过带标签的 region 和 polite live status 呈现，已消费的未知 attempt 不显示重发控件。
- 浏览器数据和提供方响应均为合成 fixture。未使用真实 Trading 212 凭据、未访问 provider-hosted 接口、未发送真实提供方写入。现有点击式原型未修改，也未升级为 PASS。
