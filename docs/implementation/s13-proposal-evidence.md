# S13 #43 不可变 OrderProposal 生成验收证据

日期：2026-09-20  
开发分支：`dev`  
规范 SHA：`e8e41ffc14bb4a90b6885be886c65707b6432534`  
实现起始 SHA：`6141ce4`  
实现/复审 SHA：`c0d1f628be946e9f7a24a6ad2c6c103b2faf95f`  
Issue：[#43](https://github.com/kaiqiangh/tradex/issues/43)；后续刷新切片 [#44](https://github.com/kaiqiangh/tradex/issues/44) 保持 OPEN，父项 [#1](https://github.com/kaiqiangh/tradex/issues/1) 保持 OPEN。

## 实现范围

- 新增 `trade.generate_proposal`、`trade.proposal.list`、`trade.proposal.get` typed IPC；Rust、JSON Schema、TypeScript client/validator 保持一致。
- SQLite schema v11 保存 workspace-scoped immutable proposal projection 和 append-only history；相同 canonical hash 幂等返回已有 proposal，proposal identity 使用独立 opaque UUID。
- Proposal 快照包含 draft fields/version、canonical SHA-256、estimated notional、policy version/state/status/reference reason、market snapshot/status/reference reason 与 `NEEDS_APPROVAL` 状态；没有 approve、arm、reserve、Gateway 或 broker I/O。
- Draft 的 material fields 变更只追加 `DRAFT_CHANGED`，保持旧 projection/hash 不变并将读取状态显示为 `INVALIDATED`；client label 等非 material 字段不会误使快照失效。
- 规范 canonical Rust structs 显式序列化字段顺序和 optional/null 值；读取时验证 projection、row identity、hash、decimal、事件顺序和 history integrity。

## 自动验证

以下检查在实现/复审 SHA 通过：

- `npm run schema:check`：Rust / JSON Schema / TypeScript 一致。
- `npm run typecheck`、`npm run build`：通过；Vite 保留既有单 chunk 大于 500 kB 的非阻断 warning。
- `npm run test:unit`：6 个 UI projection/schema 测试通过。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --features integration-test -- -D warnings`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml --workspace --quiet`：93 个 Rust library tests 通过，全部非 ignored integration tests 通过；ignored native/gateway cases 按设计保留。
- 定向测试 `thread_tests::order_proposal_is_immutable_and_invalidates_after_material_draft_change`：通过，覆盖幂等、material invalidation、旧快照不可变、reopen、stale version、cross-workspace、unknown/forged fields、无副作用和损坏 history。
- `npm run desktop:build`：通过，生成 `target/release/tradex`；仅证明原生编译，不替代实际 Tauri 窗口运行验收。
- `node --check tests/order-draft-ui.mjs`、`python3 scripts/check_requirements.py`、`git diff --check`：通过；traceability 为 201 requirements、70 screens、12 QA scenarios、23 baseline files。

## 浏览器 CUA fixture 运行证据

2026-09-20 在 `npm run dev:browser` 新建的隔离临时 workspace 中运行真实 Rust-backed browser bridge，未输入或读取真实 OAuth、ChatGPT、DeepSeek key 或 broker credential：

- 保存 draft v1 后生成 `NEEDS_APPROVAL` proposal；页面展示独立 UUID proposal ID、SHA-256、`221.5 USD` estimated notional、`Policy: UNCONFIGURED · v1 · state ...` 和 `Market: BLOCKED_EXTERNAL · snapshot —`。
- 将 quantity 从 `1` 改为 `2` 并保存为 draft v2；原 proposal 保留 quantity `1` 的不可变快照，状态变为 `INVALIDATED`，history 追加 `DRAFT_CHANGED` 及原因。
- 浏览器 console error/warn 为空；1280、768、390 视口的 `scrollWidth` 分别为 1265、753、375，与 client width 一致，无横向溢出。
- 临时 workspace、Vite、integration runner 已停止；fixture 只证明 UI、typed IPC 与本地 SQLite 线路，不证明真实 provider truth、真实 OAuth/上游推理或原生窗口行为。

## Review 与边界

- Spec 复审：`PASS`，固定 `6141ce4...c0d1f62`；opaque identity、policy/market references、fixed canonical serialization 和负向/副作用/完整性验证已核对。
- Standards 复审：`PASS`，固定 `6141ce4...c0d1f62`；未发现 documented-standard hard breach 或新增高可信 code smell。
- 当前收口范围是 #43 的 Draft → immutable Proposal vertical slice。Proposal refresh/重新生成语义由 #44 处理；本项不声称 approval、arming、reservation、Gateway、broker execution 或 S33 全量回归完成。

`cargo test`、clippy、desktop build 和浏览器验证均在 `c0d1f62` 对应源码上完成；后续只允许在新 SHA 上追加变更并重新绑定证据。
