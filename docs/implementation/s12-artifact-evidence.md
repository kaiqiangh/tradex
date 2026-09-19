# S12 研究产物保存、检索与导出验收证据

日期：2026-09-20  
开发分支：`dev`  
起始 SHA：`131b7a164b15f5983240768806c4c3c14e43c599`  
实现/审查 SHA：`ac3b18e28936e373621a9f9500bb8f3c078ab91a`  
Issue：[#40](https://github.com/kaiqiangh/tradex/issues/40)（OPEN）；规范 [#39](https://github.com/kaiqiangh/tradex/issues/39) 与父项 [#1](https://github.com/kaiqiangh/tradex/issues/1) 保持 OPEN。

## 实现范围

- `artifact.save/list/get/export` 已接入 Rust IPC、JSON Schema、生成的 TypeScript client/validator、SQLite schema 9 和 Artifacts UI。
- 保存前重新读取 workspace-scoped Thread/Turn/Item，要求 Turn/Item 已完成，并只复制 typed research producer 的 provenance；renderer context 不会被改写为 market snapshot、dataset 或 order identity。
- 导出使用脱敏投影、canonical hash、UTF-8 manifest、用户明确点击和 Unix `openat`/`O_NOFOLLOW`/`linkat` 不覆盖写入；路径越界、祖先符号链接、目标存在、敏感键和写入失败均 fail closed。
- provenance modal 提供不可变 Turn snapshot、provider attempts、研究来源和键盘焦点 trap/恢复；原生 `dialog:allow-save` 已加入能力配置。

## 自动验证

以下命令在实现/审查 SHA 的工作树通过：

- `cargo fmt --all -- --check`
- `cargo clippy -p tradex --all-targets -- -D warnings`
- `cargo test --workspace --features integration-test -- --test-threads=1`：97 个 Rust 单元测试通过；集成测试通过，原生 Keychain 与 pinned gateway 测试按设计 ignored。
- `cargo test -p tradex artifact_round_trip_preserves_provenance_and_exports_safely -- --nocapture`
- `cargo test -p tradex renderer_contexts_are_not_reinterpreted_as_producer_refs -- --nocapture`
- `npm run schema:check`
- `npm run typecheck`
- `npm run test:unit`：6 个 UI projection/schema 测试通过。
- `npm run build`：Vite 构建通过；保留既有单 chunk 大于 500 kB 的非阻断 warning。
- `node --check tests/thread-ui.mjs`
- `python3 scripts/check_requirements.py`：201 requirements、70 screens、12 QA scenarios、23 baseline files traceability 通过。
- `git diff --check`

Standards 与 Spec 两轴复审均为 `PASS`。最后一次复审覆盖 producer/context 边界、双语 ARD 可选字段、敏感键、迁移、符号链接和 no-follow 原子导出。

## 未验证边界

- 当前 CUA 状态报告 macOS 仍锁定，无法执行真实 Tauri/native save chooser、桌面导出或原生 focus-trap；这些证据保持 `UNVERIFIED`，不能由 fixture 或静态检查升级为 PASS。
- 隔离浏览器能打开 TradeX，但当前 workspace 显示 CLIProxyAPI discovered models 为 0、ChatGPT/DeepSeek 未配置且没有 default model route；因此没有声称真实 provider/OAuth/DeepSeek 路由验证通过。
- `tests/thread-ui.mjs` 只完成语法检查；真实浏览器交互、原生 Keychain 和 pinned gateway 仍待桌面解锁并提供可用模型路由后重跑。

S12 因此标记为 `IMPLEMENTED_UNVERIFIED`。S13–S35、S33 全量回归、OD-001/OD-003/OD-005/OD-006 真实授权和最终 `dev → main` PR 仍未完成。
