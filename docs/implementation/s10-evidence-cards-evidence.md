# S10 #35 股票与现货 evidence cards 证据

日期：2026-09-19  
规格：`b98c5e1`  
实现：`8b41df1`、`d142842`、`db32dd3` 及本次审查修正
状态：**IMPLEMENTED_UNVERIFIED**

## 已交付

- `ResearchToolPayload` 增加有界 `scenarios`、canonical `artifactRefs`、`spotVenues` 和 `fixtureLabel`；旧 payload 仍可解码。
- 股票 fixture 只接受 `public_market_read + EQUITY`，输出 `AVAILABLE`、canonical instrument、scenario 和显式 synthetic provenance；artifact ref 只从已附加的 `artifact` context 复制。
- 现货只接受 `public_market_read + CRYPTO_SPOT` 的 integration fixture，输出固定的 Binance/Bitget typed rows；其他 tool/focus 组合不能取得 synthetic source。
- 不可用 venue 的 bid/ask/spread/depth/quoteAge 在 wire 上保留为 `null`，schema 接受 `null` 并拒绝 numeric zero；Trade CTA 仍是 disabled/read-only 且只在 Trade mode 出现。
- integration-only context catalog 暴露一个有界 synthetic artifact，便于验证 canonical artifact ref 从 picker 到 research card 的完整线路；evidence card 与 marker 可通过键盘聚焦。

## 验证

| 检查 | 结果 |
|---|---|
| `npm run schema:check` | PASS — Rust / JSON Schema / TypeScript 一致 |
| `npm run typecheck`、`npm run build`、`npm run test:unit` | PASS — unit 6 |
| `cargo test --workspace --features integration-test -- --test-threads=1` | PASS — library 90（含 8 research tests），integration targets 无失败；native provider/gateway tests 仍按仓库标记 ignored |
| `cargo clippy --workspace --all-targets --features integration-test -- -D warnings` | PASS |
| `cargo fmt --all -- --check`、`git diff --check`、`node --check tests/thread-ui.mjs` | PASS |
| Rust-backed bridge `research.run` | PASS — EQUITY 返回 `AVAILABLE` + `equity:US:AAPL` + scenario + `artifact-1`；CRYPTO_SPOT 返回 Binance/Bitget；`account_read + CRYPTO_SPOT` 无 fixture label 且五个 quote 字段为 `null` |
| CUA 浏览器 `http://127.0.0.1:1420/` | PASS（目标路径）— 股票/现货卡片、`SYNTHETIC_INTEGRATION_FIXTURE` label、artifact ref、scenario、Trade disabled CTA；card/marker 键盘焦点与 picker Tab/Shift+Tab 已验证；此前 1280/768/390 的 `scrollWidth` 为 1265/753/375，目标路径 warn/error 日志为空。完整 helper 在取消竞态处停止：fake runtime 在点击 Cancel 前完成 Turn 2，因此不把该次 helper 运行宣称为全绿 |

浏览器运行使用 `npm run dev:browser` 的 Rust stdio/SQLite 临时 workspace 与明确的 provider/research fixture；没有读取或写入用户 ChatGPT OAuth、DeepSeek key 或 broker credential。fixture label 仅证明契约和 UI 线路，不证明真实 Binance/Bitget entitlement、实时行情或 provider 事实。真实模型/native Keychain、provider entitlement 和完整 S33 回归仍待独立验证，因此本票保持 `IMPLEMENTED_UNVERIFIED`，GitHub #35 保持 OPEN。

## 边界

本票没有新增 provider client、credential lookup、网络行情、订单 proposal、approval、arming、Gateway 或 live-risk 调用；没有合并 `main`，也没有关闭 issue。
