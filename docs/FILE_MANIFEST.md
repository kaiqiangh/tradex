# TradeX v1.0 RevC — File Manifest / 文件清单

**Generated / 生成日期:** 2026-10-10

Paths below are relative to the repository root. Hashes describe the current local file bytes after the S01/S02 IPC additions, user-approved S03 DeepSeek model clarification, S08 market-state contract additions, S18 Trading 212 Demo wire/UI and local deletion contract, S19 Binance Spot Testnet submission/recovery/private-stream/cancellation contracts, the 2026-09-25 account-interface scope update retiring dedicated Binance Testnet and Bitget Demo acceptance, S21 #82 workspace risk-policy change effects, S25.1–S25.4 Live reconciliation contracts, S26.2 #104 Trading 212 Live cancellation-history/fill-race contracts, and S27.1 #108 Live startup-recovery contracts, the S28 #118 authenticated XNAS calendar contract and S28 #119 known-event/exact-account metadata contract, plus the S28 #120 required-route/read-only FX contract and S29 account-state, exact-rule, Hot-source and #125 Proposal rule/reference plus #126 read-only capacity contracts, the #127 interval-quota contract, the #128 immutable proposal price-range execution-explanation contract and the #129 owning Spot Live admission-qualification contract, not a released ZIP or a passing execution system. The unchanged prototype sources were reviewed at main@6c4b267. Regenerate this manifest whenever a listed file changes.

以下路径相对仓库根目录。哈希描述 S01/S02 IPC 增补、用户批准的 S03 DeepSeek 模型澄清、S08 市场状态契约增补、S18 Trading 212 Demo wire/UI 与本地账户删除契约、S19 Binance Spot Testnet 提交/恢复/私有流/撤单契约，以及 2026-09-25 将专用 Binance Testnet 和 Bitget Demo 验收改为普通账户接口范围、S21 #82 工作区风险策略变更影响、S25.1–S25.4 Live 对账契约、S26.2 #104 Trading 212 Live 撤单历史/成交竞态契约及 S27.1 #108 Live 启动恢复契约以及 S28 #118 经认证的 XNAS 日历契约和 S28 #119 已知事件/精确账户元数据契约以及 S28 #120 所需路由/只读 FX 和 S29 账户状态、精确规则、Hot 来源及 #125 Proposal 规则/参考、#126 只读容量契约、#127 时段额度契约、#128 不变 Proposal 价格区间执行解释契约与 #129 拥有方 Spot Live 准入资格契约后的本地文件字节，不表示已打包发布或执行系统验收通过。未修改的原型源码审查基线为 main@6c4b267。清单中文件变化后须重新生成。

S29.9 source/refusal and S29.10 exact-symbol commission-source contracts are included. Source declarations do not establish full financial qualification; prototype sources remain unchanged.

包含 S29.9 来源/拒绝与 S29.10 精确标的佣金来源契约，来源声明不建立完整金融资格，原型源码未改。

This manifest excludes its own hash to avoid self-reference. Existing docs/agents workflow guides are outside this product-document manifest. The root README.md describes the application build; docs/implementation tracks delivery separately from this product-document manifest.

为避免自引用，本文件不计算自身哈希。既有 docs/agents 工作流说明不属于本产品文档清单。根 README.md 说明应用构建；docs/implementation 单独记录交付，不计入本产品文档清单。

| File / 文件 | Purpose / 用途 | Lines / 行数 | SHA-256 |
|---|---|---:|---|
| `AGENTS.md` | Agent entry point / Agent 文档入口 | 30 | `c3242039e90c7f6c684b0de2fd3de2ad16132839293b508facb06ecbd14be9dc` |
| `docs/README.md` | English documentation index | 64 | `a51095455aae2fd24a271f3892fc62934276a0c8a715c190a4c52ea36114c8a8` |
| `docs/zh/README.md` | 中文文档目录 | 59 | `f63e69db37f04aa7fe7e29a853da8c7165b9e7d6f48d719665cc92e2135afdc6` |
| `docs/TradeX_PRD_v1.0_RevC.md` | English PRD | 3683 | `00265a8e441efeac060ec94ba85b10113229e6cda5d32d05b6eef8831b676538` |
| `docs/zh/TradeX_PRD_v1.0_RevC_zh.md` | 中文 PRD | 3675 | `fccbae321656688637dd35df8823c6e040d14ddf88c758bf24b4319f34367dac` |
| `docs/TradeX_UI_Prototype_Spec_v1.0_RevC.md` | English UI Spec | 1339 | `b91a07a7dc3fb5e204e0775b8cac79c76549d4c3d63c9e475288c25441c396c9` |
| `docs/zh/TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md` | 中文 UI Spec | 1054 | `7ca7e54e0612e516039e9417fada9053dfa02ba73f9d6d7d09d67d0ca76115a7` |
| `docs/TradeX_Frontend_ARD_v1.0_RevC.md` | English Frontend ARD | 1215 | `75c234c28a9d70edb2ef1e061619b9909b8bd49e02eb2d2cec587425d57f0ba8` |
| `docs/zh/TradeX_Frontend_ARD_v1.0_RevC_zh.md` | 中文 Frontend ARD | 1213 | `9062222fc27e4909491fdd36c965280da2072a44411eae8ed60563b26f63f24b` |
| `docs/TradeX_Backend_ARD_v1.0_RevC.md` | English Backend ARD | 3465 | `661a5997a80bd9f505a86e021dbf75674acd9fe6f7ed984286d8771baa0a663b` |
| `docs/zh/TradeX_Backend_ARD_v1.0_RevC_zh.md` | 中文 Backend ARD | 3463 | `74e52ae4d8f4d89c4654132cb7d3704324afa87563f6b7b1c7be4a680726d87f` |
| `docs/TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md` | English Coverage Matrix | 214 | `3dcf4b201c135468a113c760d02e51ffac0268daaeca5595f09f08a281463f7c` |
| `docs/zh/TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md` | 中文 Coverage Matrix | 214 | `e9080e0dc8189d39815b1ac78518dad93f3319190a5783978e30cca7620049b8` |
| `docs/TradeX_Prototype_QA_Report_v1.0_RevC.md` | English QA Report | 188 | `5d1b2619f2fbe829a17684e1b2d32ca54c496386e5c32932d8c90c168178bc73` |
| `docs/zh/TradeX_Prototype_QA_Report_v1.0_RevC_zh.md` | 中文 QA Report | 188 | `b78ca742c8bb6ec8fdbc1a4716b15e9e9b0e6f23e4fd486f14457ee749e25c87` |
| `docs/prototype/README.md` | English prototype guide | 40 | `ee23e4cd6886a568d78205d1cb1dde5f1f441a998b7e15be8f4d515142f37795` |
| `docs/prototype/README_zh.md` | 中文原型说明 | 40 | `12939cb65e82bdc3eee153fc4719842d64a88d3b914c7a95a67d7e6b10b948bf` |
| `docs/prototype/index.html` | Clickable fixture HTML / 原型 HTML | 13 | `0d2d8341a2c0f0b24cabc131f6c5da68455b9885ffd56dac48b42fa23fb49a8b` |
| `docs/prototype/styles.css` | Fixture styles / 原型样式 | 57 | `68f40e45fcd6352f40f25d129edd86fa4312f90539150292cbceccd2c7610adc` |
| `docs/prototype/app.js` | Fixture state and interaction / 原型状态与交互 | 929 | `552e987172c171c64ea71fcb4bce2f1c40fe4944e093ccd575eb5db66701f96d` |

## Language pairs / 中英文配对

- PRD: [English](./TradeX_PRD_v1.0_RevC.md) ↔ [中文](./zh/TradeX_PRD_v1.0_RevC_zh.md)
- UI Spec: [English](./TradeX_UI_Prototype_Spec_v1.0_RevC.md) ↔ [中文](./zh/TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md)
- Frontend ARD: [English](./TradeX_Frontend_ARD_v1.0_RevC.md) ↔ [中文](./zh/TradeX_Frontend_ARD_v1.0_RevC_zh.md)
- Backend ARD: [English](./TradeX_Backend_ARD_v1.0_RevC.md) ↔ [中文](./zh/TradeX_Backend_ARD_v1.0_RevC_zh.md)
- Coverage Matrix: [English](./TradeX_Prototype_Coverage_Matrix_v1.0_RevC.md) ↔ [中文](./zh/TradeX_Prototype_Coverage_Matrix_v1.0_RevC_zh.md)
- QA Report: [English](./TradeX_Prototype_QA_Report_v1.0_RevC.md) ↔ [中文](./zh/TradeX_Prototype_QA_Report_v1.0_RevC_zh.md)
- Prototype guide: [English](./prototype/README.md) ↔ [中文](./prototype/README_zh.md)
- Index: [English](./README.md) ↔ [中文](./zh/README.md)
