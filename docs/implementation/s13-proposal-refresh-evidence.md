# S13 #44 stale Proposal 刷新验收证据

日期：2026-09-20  
开发分支：`dev`  
规范 SHA：`92d86fd3c962812368e7b8a88524f9f1be0849f8`  
实现/复审 SHA：`90ede1fad310a821d7e493beac733af536a72ab`  
Issue：[#44](https://github.com/kaiqiangh/tradex/issues/44)；父项 [#41](https://github.com/kaiqiangh/tradex/issues/41) 与总图 [#1](https://github.com/kaiqiangh/tradex/issues/1) 在本证据提交前保持 OPEN。

## 实现范围

- 新增 typed `trade.refresh_proposal` IPC；Rust、JSON Schema、TypeScript client/validator 保持一致。
- Refresh 在 SQLite immediate transaction 内校验 workspace、proposal 状态、draft version 和 expected state version，原子追加旧对象 `REFRESHED` history、插入新 immutable proposal 与 `GENERATED` event；失败不留下部分写入。
- 旧 proposal 的 projection、hash、created time 和 immutable fields 不变；新 proposal 使用不同 identity 并回到 `NEEDS_APPROVAL`。canonical hash 不再包含随机 proposal ID，刷新后的 TimeService observation 进入 market reference reason/hash。
- `REFRESHED` history 严格校验 replacement ID、workspace、projection identity/hash 和首个 `GENERATED` event；篡改 history 会返回 workspace integrity error。
- `REFRESHED`、`STALE`、`BLOCKED`、`UNAVAILABLE` 状态均有 typed mapping；UI 显示状态、reason、完整旧/新 proposal ID 与 hash，并保留旧对象 `INVALIDATED` 和新对象 `NEEDS_APPROVAL`。Refresh 不调用 approval、arming、reservation、execution、provider 或 Gateway。

## 自动验证

以下检查在实现/复审 SHA 通过：

- `npm run schema:check`：Rust / JSON Schema / TypeScript 一致。
- `npm run build`：通过，包含 TypeScript typecheck；Vite 仅保留既有单 chunk 大于 500 kB 的非阻断 warning。
- `npm run test:unit`：6 个 UI projection/schema 测试通过。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --features integration-test -- -D warnings`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml --workspace --quiet`：94 个 Rust library tests 通过，全部非 ignored integration tests 通过；ignored native/gateway cases 按设计保留。
- 定向测试覆盖全部 refresh status、stale/invalidated 无部分写入、account snapshot 不变、旧/新对象重开、篡改 replacement reason/首事件拒绝，以及 draft/risk/workspace sequence 不变。
- `node --check tests/order-draft-ui.mjs`、`python3 scripts/check_requirements.py`、`git diff --check`：通过；traceability 为 201 requirements、70 screens、12 QA scenarios、23 baseline files。
- `npm run desktop:build`：通过，生成 `/Users/kai/Desktop/my-repo/tradex/target/release/tradex`；仅证明原生编译，不替代实际签名或窗口验收。

## 浏览器 CUA fixture 运行证据

2026-09-20 在 `npm run dev:browser` 创建的隔离临时 workspace 中运行真实 Rust-backed browser bridge，未输入或读取真实 OAuth、ChatGPT、DeepSeek key 或 broker credential：

- 保存 draft、生成 proposal 后执行 Refresh；页面显示 `Proposal refreshed (STALE)`、完整旧/new proposal ID、reason 和“a new approval is required”。旧 proposal 保留 `REFRESHED` history，新 proposal 为 `NEEDS_APPROVAL`。
- 1280、768、390 视口的 proposal selector 均为 `scrollWidth === clientWidth`（分别 1265、753、375），完整 identity/hash 不横向溢出。
- 浏览器 console `error`/`warn` 为空；临时 workspace、Vite 和 integration runner 已停止。

## Review 与边界

- Spec 复审：`PASS`，固定 `92d86fd3...90ede1f`；canonical hash、replacement integrity、status matrix、旧/新 identity 和 history 证据已核对。
- Standards 复审：`PASS`，固定 `92d86fd3...90ede1f`；未发现 documented-standard hard breach 或阻塞性 code smell。
- 当前证据是本地 Rust、SQLite、typed IPC 与浏览器 fixture 证据。真实 provider/OAuth/DeepSeek/Gateway、approval/arming/reservation、broker mutation、原生签名运行、S33 全量回归和 `dev → main` PR 仍由后续工作项验证。

