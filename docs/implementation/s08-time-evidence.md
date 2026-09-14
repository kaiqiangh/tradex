# S08 #30 TimeService 与可信时间门禁验收证据

验证日期：2026-09-14
固定实现代码点：`c659d2b0a9e766abd41a69ec2318df0033016fdf`
证据记录提交点：`cf8e02edfd6ecc4514731f557fc8155545b3d022`
浏览器地址：`http://127.0.0.1:1420/`（integration mode，隔离临时 workspace）

## Rust、协议与边界

- 新增 Rust Control Plane `TimeService`，使用 UTC wall clock 与进程内 monotonic `Instant`；公开 2,000 ms wall/monotonic 容差和 5,000 ms provider/server offset 上限。
- `time.status` 与 `time.revalidate` 使用现有 `WorkspaceQuery` 输入，输出生成的 `TimeStatus`；未知 workspace、未知字段和 renderer `wallClock` 覆盖均返回 `IPC_AGGREGATE_NOT_FOUND` 或 `IPC_PAYLOAD_INVALID`。
- workspace open、同 workspace reopen、进程新建和 Tauri `RunEvent::Resumed` 都清除既有基准。首次 status 保持 `CLOCK_UNCERTAIN`；只有显式 revalidate 建立 `TRUSTED` 基准。
- 注入式单测覆盖首次基准、正常 elapsed、material wall jump、monotonic rollback、provider offset、resume 和 `require_trusted` 门禁。失信返回 `CLOCK_UNCERTAIN` 或 `STALE`，并使用 `CLOCK_SKEW` / `time_revalidate` remediation。
- `time.status` / `time.revalidate` 只读进程状态，不写 SQLite、outbox、DomainProjection、account、risk、approval 或 thread；ControlPlane 集成测试验证 workspace 隔离、reopen 失信和状态快照不变。
- 双语 Backend ARD §41.12、Frontend ARD §13.3、FILE_MANIFEST 与生成的 JSON Schema/TypeScript 已同步。

## 检查命令

- `npm run schema:check`：通过，Rust / JSON Schema / TypeScript 一致。
- `npm run build`：通过；Vite 保留既有单 bundle 大于 500 kB 提示。
- `npm run test:unit`：4/4 通过。
- `cargo test --workspace`：全部非 ignored library/integration tests 通过；新增 TimeService 6 个 Rust 单测和 1 个 ControlPlane 集成测试通过；原有 native Keychain/Gateway 测试按定义保持 ignored。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo fmt --all -- --check`、`git diff --check`：通过。
- `python3 scripts/check_requirements.py`：201 requirements、70 screens、12 QA、23 baseline 通过。

## Browser / Rust-backed UI

- Settings → Account Health 的 Trusted time 区域展示 `CLOCK_UNCERTAIN`、wall clock、monotonic reading、provider offset、observed time、阻塞 reason 与 `Synchronize time`。
- 显式点击 `Synchronize time` 后状态变为 `TRUSTED`，reason 更新为已同步；动作使用 semantic button、`role="status"`、`aria-live="polite"`，操作完成后焦点保持在按钮上。
- 视口验证：1280×768、768×768、390×844；三者 `document.documentElement.scrollWidth === window.innerWidth`，无水平溢出。390px 页面仍可读到完整阻塞原因和恢复控件。
- 浏览器控制台 `error` / `warn` 为空。

## External gate and scope

本票没有访问 provider、读取凭据或修改用户 workspace；OD-005 日历/公司行为 entitlement 仍由 #31 处理并保持外部阻断。#30 只交付可信时间状态与可复用门禁，不实现订单、风险、审批、预留、派发或真实 Live freshness 消费；S08 父票、QA-04 和 S33 仍未关闭。
