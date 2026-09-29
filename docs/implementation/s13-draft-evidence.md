# S13 #42 OrderDraft 保存与编辑验收证据

日期：2026-09-20  
开发分支：`dev`  
起始 SHA：`2e080b74651efe702fd31e429f39ea35eb6c37bd`  
实现/复审 SHA：`b61f5833c71df54633e53acece7da9875852075a`  
Issue：[#42](https://github.com/kaiqiangh/tradex/issues/42)（待关闭）；规范 [#41](https://github.com/kaiqiangh/tradex/issues/41) 保留到 #43/#44 完成，父项 [#1](https://github.com/kaiqiangh/tradex/issues/1) 保持 OPEN。

## 实现范围

- 新增 `trade.draft.list`、`trade.draft.get`、`trade.save_draft` typed IPC，JSON Schema、生成的 TypeScript client/validator 与 Rust dispatcher 保持一致。
- SQLite schema v10 保存 workspace-scoped OrderDraft projection；新建版本从 1 开始，更新要求当前 `stateVersion`，旧版本返回冲突，重开读取原值，跨 workspace 与未知字段拒绝。
- 编辑器覆盖 account、venue、environment、canonical instrument、side、order type、BASE/QUOTE quantity、规范 decimal、limit price、maximum spend、TIF 和 client label；renderer 不能写入 proposal/hash/policy/snapshot/approval/execution 字段。
- 后端校验空值、零/负数、18 位以上小数、非法组合、context/venue/instrument/provider 映射，并返回字段级错误；保存不创建 approval、reservation、execution/outbox，也不访问 provider network。
- 修正新草稿详情查询在 React Query 禁用状态下永久显示 Loading 的根因；新建表单、保存、重开和错误反馈均可操作。

## 自动验证

以下检查在实现/复审 SHA 通过：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p tradex`：92 个 Rust 单元测试、全部非 ignored 集成测试通过；原生 Keychain 与 pinned gateway 测试按设计 ignored。
- `cargo test -p tradex --test workspace -- --nocapture`：6 个 workspace 迁移/重开测试通过。
- `npm run schema:check`：Rust / JSON Schema / TypeScript 一致。
- `npm run typecheck`
- `npm run test:unit`：6 个 UI projection/schema 测试通过。
- `npm run build`：Vite 构建通过，保留既有单 chunk 大于 500 kB 的非阻断 warning。
- `npm run desktop:build`：当前源码 release 桌面编译通过，产物为 `target/release/tradex`。
- `node --check tests/order-draft-ui.mjs`
- `python3 scripts/check_requirements.py`：201 requirements、70 screens、12 QA scenarios、23 baseline files traceability 通过。
- `git diff --check`

## 浏览器 CUA fixture 运行证据

2026-09-20 在 `npm run dev:browser` 新建的隔离临时 workspace 中运行 `tests/order-draft-ui.mjs`，未输入或读取真实 OAuth、DeepSeek key、broker credential：

- `Order Drafts` 页面通过 typed Rust dispatcher 读取真实临时 SQLite；新建表单展示全部 S13 #42 字段。
- 数量设为 `0` 时 Save draft 被禁用并显示字段错误；有效草稿保存成功，列表和编辑器显示 `v1`。
- 新草稿输入 `1.1234567890123456789` 时后端拒绝，UI 显示 decimal 错误并将 `aria-describedby` 绑定到 `order-field-error`。
- 使用 Tab/Enter 完成保存；1280、768、390 宽度均保持页面可见且无横向溢出。
- helper 检查通过，浏览器 console error 数为 0；fixture 仅证明 UI、IPC 和本地持久化线路，不证明真实 provider truth。

## Review 与边界

Standards 复审没有发现硬性规范违例；重复的 Draft ID 校验已收敛到 storage helper，字段命名与迁移断言已修正。Spec 复审提出的字段级错误、decimal precision、TIF/order-type、provider/instrument、cross-workspace/unknown-field 与响应式证据均已补齐并重跑相关检查。

当前收口范围是 #42 的 Draft persistence/editor。`trade.generate_proposal`、不可变 proposal identity/history 与 stale refresh 仍由 #43/#44 按串行依赖实现；本项不声称 proposal、approval、arming、reservation、Gateway、broker execution 或 S33 全量回归完成。

当前 release bundle 原生窗口在本轮 CUA 启动检查中未返回可用 AX 状态，因此桌面证据仅记为编译通过；浏览器与 Rust dispatcher 证据不替代后续可用原生窗口的运行检查。
