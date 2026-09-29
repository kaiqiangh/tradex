# S10 #35 股票与现货 evidence cards 证据

日期：2026-09-20
规格：`b98c5e143c7931208f266d98d8723da6900faded`
实现：`bc3164f24ae85b3100ce51f84243cfcc072891fb`
状态：**IMPLEMENTED_UNVERIFIED**

## 已交付

- `ResearchToolPayload` 提供有界 `scenarios`、canonical `artifactRefs`、`spotVenues` 和 `fixtureLabel`；旧 payload 仍可解码。
- 股票 fixture 只接受 `public_market_read + EQUITY`，输出 `AVAILABLE`、canonical instrument、scenario 和显式 synthetic provenance；artifact ref 只从已附加的 canonical artifact context 复制。
- 现货只接受 `public_market_read + CRYPTO_SPOT` 的 integration fixture，输出固定的 Binance/Bitget typed rows；其他 tool/focus 组合不能取得 synthetic source。
- 不可用 venue 的 bid/ask/spread/depth/quoteAge 在 wire 上保留为 `null`，schema 接受 `null` 并拒绝 numeric zero；Trade CTA 仍是 disabled/read-only 且只在 Trade mode 出现。
- `ResearchCardBoundary` 同时包裹 preview 与 persisted card，单卡渲染异常降级为可聚焦的 alert，不影响 Thread 状态。
- `synthetic_research_fixture_enabled()` 同时约束 context catalog、artifact trust boundary 和 research source；它要求 `integration-test` feature 与显式 `TRADEX_RESEARCH_FIXTURE` 环境变量。
- artifact context 在可信边界按 artifact id、workspace id 和 canonical content hash 校验；未知 id、错误 hash、跨 workspace ref 均 fail closed。synthetic artifact 只在同一显式 fixture seam 中接受。

## 验证

| 检查 | 结果 |
|---|---|
| `npm run check` | PASS — schema、build、unit 6、默认 Rust library 96、integration targets、requirements 全部通过；Rust 默认测试无失败 |
| `cargo test --workspace --features integration-test -- --test-threads=1` | PASS — library 102 与 integration targets 全部通过；native provider/keychain、pinned gateway 等仓库标记的测试仍 ignored |
| `cargo clippy --workspace --features integration-test --all-targets -- -D warnings` | PASS |
| `cargo fmt --all -- --check`、`git diff --check`、`node --check tests/thread-ui.mjs`、`python3 scripts/check_requirements.py` | PASS — traceability 201 requirements、70 screens、12 QA scenarios、23 baseline files |
| targeted Rust fixture boundary tests | PASS — 环境变量缺失和 `TRADEX_RESEARCH_FIXTURE=1` 两种路径均验证 context catalog、policy matrix、typed result；unknown artifact、wrong hash、cross-workspace ref 拒绝 |
| CUA 浏览器 `http://127.0.0.1:1420/` | PASS — 当前隔离 workspace 验证股票/现货 card、synthetic label、artifact ref、scenario、Trade disabled CTA、card/marker 键盘焦点、Turn 发送后 reload 持久化和 warn/error 日志；此前完整 helper 已覆盖 1280/768/390 三种视口，未改动的样式路径保持该证据 |

CUA 使用 `npm run dev:browser` 的 Rust stdio/SQLite 临时 workspace 与明确的 provider/research fixture；没有读取或写入用户 ChatGPT OAuth、DeepSeek key 或 broker credential。fixture label 只证明契约和 UI 线路，不证明真实 Binance/Bitget entitlement、实时行情或 provider 事实。

## 边界

真实模型/native Keychain、provider entitlement、真实行情和完整 S33 回归仍待独立验证，因此本票保持 `IMPLEMENTED_UNVERIFIED`；本票的文档验收项已完成，可以关闭 GitHub #35。没有新增 provider client、credential lookup、网络行情、订单 proposal、approval、arming、Gateway 或 live-risk 调用；没有合并 `main`。
