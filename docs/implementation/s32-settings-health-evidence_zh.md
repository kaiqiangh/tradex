# S32 设置、账户健康、隐私与恢复证据

**状态：** S32 实现和隔离原生 UI/IPC 验收已在 `dev` 完成。实现票为 [#139](https://github.com/kaiqiangh/tradex/issues/139)，关联规范 [#138](https://github.com/kaiqiangh/tradex/issues/138)。

- 源码提交：`0593ed78cca75a47bac35170916da8727f78a09b`
- 复审基线：`c1c146d015da7601a4a46d971433718ef0285fbe`
- 范围：RevC UI Spec J5–J7、K7；PRD FR-034、FR-064、AC-050、NFR-013、NFR-004、OPS-007；#138/#139。
- Provider 范围：按用户要求，Alpaca、Binance、Bitget、Trading 212 集成继续跳过。S27 物理睡眠/唤醒和 S33 全页面回归仍是独立开放门禁。

## 已交付行为

Settings → Account Health 只读取本地 `account.list` 投影，并分别显示连接、认证、私有流、对账、执行资格和 Arm 状态，同时显示最近同步与最近私有流事件时间。重新加载是明确的用户操作。Disable All Live 仍是独立且明确的 TradeX 本地撤防操作；打开此页不会刷新 provider。Tauri dispatcher 中的 `account.list` 是本地存储读取。

Appearance 如实显示原型中的 System/Light 表面，Dark 不可用，也不声称偏好已持久化。About 显示 Tauri 应用版本、原生运行时组件状态、RevC 基线及历史原型免责声明。隐私页面显示遥测已禁用、崩溃报告未配置、自动更新未配置。没有可用 collector 时不展示虚假的同意开关。

可复用恢复组件覆盖 RevC K7 的 20 个分类，将已知 model/provider 错误码归并为规范分类，并将未知或格式错误的数据回退到 `INTERNAL_ERROR`，不回显原始错误字段。通用命令错误改用规范恢复文本；现有结构化 reserved-capacity 错误仍保留其类型化金额上下文。测试诊断扫描会用合成 canary 调用真实隔离 IPC dispatcher，检查输出不含 canary，并验证扫描器能识别四种 canary 形式。

## 验收证据

- `npm run check` — 在源码提交上通过：schema 一致性、TypeScript 检查与生产构建、Node 测试 23/23、`cargo test --workspace`（库测试 257 项及所有未忽略 workspace fixture/集成测试）、代表性诊断秘密扫描，以及 203 项需求、70 个界面、13 个 QA 场景、23 个基线文件的追溯检查。
- `npm run desktop:build` — 通过，生成 `target/release/tradex`。前端仍有既有的大 bundle 警告。
- `cargo clippy --workspace --all-targets --all-features` — 退出码 0，存在既有警告，包括 `result_large_err` 和 fixture 警告。`cargo clippy --workspace --all-targets -- -D warnings` 会因未改动 Rust 文件中的既有警告失败；S32 未修改 Rust 源码。
- `cargo fmt --all -- --check` — 报告多个未改动既有 Rust 文件的格式差异；未运行格式化器改写无关文件。`git diff --check` 通过。
- 恢复组件定向测试 — 6/6 通过，覆盖全部 20 个分类、可访问 alert 标记、已知错误码映射、秘密安全的未知错误回退和通用命令错误脱敏。
- 原生 TradeX 集成测试构建 — 以隔离 Tauri 应用连接临时 SQLite 工作区 `48734a94-090a-4d87-9668-855766067a24`。Account Health 展示六个独立维度及两个时间戳；可访问性树中包含 Appearance、About、本地组件状态和遥测/崩溃/更新状态。临时工作区只有 TradeX Local Paper 模拟账户。
- 原生键盘/窄窗检查 — 按 Tab 可到达并聚焦“Reload health snapshot”，焦点环清晰可见。窗口平铺缩窄后，Settings 导航、账户健康卡片和 About 内容仍可读，未观察到横向溢出。S33 的全量 768/390 像素及跨页面回归仍待完成。
- 未输入 provider 凭据，未调用 provider 连接、刷新或订单命令，未进行金融写入。验收结束后已停止临时 UI 服务和原生测试应用。点击式原型未修改。

## 保留门禁

S32 不关闭 provider 集成、S27 物理睡眠/唤醒、S33 全页面无障碍/响应式回归、S34 性能与签名打包或 S35 最终验收。Wayfinder map 继续开放；所有剩余工作在 `dev` 验证完成前，不创建 `dev` → `main` PR。
