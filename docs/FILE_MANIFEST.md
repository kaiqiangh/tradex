# TradeX v1.0 RevC — File Manifest / 文件清单

**Generated / 生成日期:** 2026-10-08

Paths below are relative to the repository root. Hashes describe the current local file bytes after the S01/S02 IPC additions, user-approved S03 DeepSeek model clarification, S08 market-state contract additions, S18 Trading 212 Demo wire/UI and local deletion contract, S19 Binance Spot Testnet submission/recovery/private-stream/cancellation contracts, the 2026-09-25 account-interface scope update retiring dedicated Binance Testnet and Bitget Demo acceptance, S21 #82 workspace risk-policy change effects, S25.1–S25.4 Live reconciliation contracts, S26.2 #104 Trading 212 Live cancellation-history/fill-race contracts, and S27.1 #108 Live startup-recovery contracts, the S28 #118 authenticated XNAS calendar contract and S28 #119 known-event/exact-account metadata contract, plus the S28 #120 required-route/read-only FX contract, not a released ZIP or a passing execution system. The unchanged prototype sources were reviewed at main@6c4b267. Regenerate this manifest whenever a listed file changes.

以下路径相对仓库根目录。哈希描述 S01/S02 IPC 增补、用户批准的 S03 DeepSeek 模型澄清、S08 市场状态契约增补、S18 Trading 212 Demo wire/UI 与本地账户删除契约、S19 Binance Spot Testnet 提交/恢复/私有流/撤单契约，以及 2026-09-25 将专用 Binance Testnet 和 Bitget Demo 验收改为普通账户接口范围、S21 #82 工作区风险策略变更影响、S25.1–S25.4 Live 对账契约、S26.2 #104 Trading 212 Live 撤单历史/成交竞态契约及 S27.1 #108 Live 启动恢复契约以及 S28 #118 经认证的 XNAS 日历契约和 S28 #119 已知事件/精确账户元数据契约以及 S28 #120 所需路由/只读 FX 契约后的本地文件字节，不表示已打包发布或执行系统验收通过。未修改的原型源码审查基线为 main@6c4b267。清单中文件变化后须重新生成。

This manifest excludes its own hash to avoid self-reference. Existing docs/agents workflow guides are outside this product-document manifest. The root README.md describes the application build; docs/implementation tracks delivery separately from this product-document manifest.

为避免自引用，本文件不计算自身哈希。既有 docs/agents 工作流说明不属于本产品文档清单。根 README.md 说明应用构建；docs/implementation 单独记录交付，不计入本产品文档清单。

| File / 文件 | Purpose / 用途 | Lines / 行数 | SHA-256 |
|---|---|---:|---|
| `AGENTS.md` | Agent entry point / Agent 文档入口 | 30 | `c3242039e90c7f6c684b0de2fd3de2ad16132839293b508facb06ecbd14be9dc` |
| `docs/README.md` | English documentation index | 60 | `02db9e68854620ab068c3907713643452096569f46b7afa13e9a52fe0605a5e2` |
| `docs/zh/README.md` | 中文文档目录 | 57 | `223863b6afc0ae0084e0eaaafa945b6e3f5f785516e01968b6b2f5ed2959ef44` |
| `docs/TradeX_PRD_v1.0_RevC.md` | English PRD | 3669 | `dd0a7737604ce0bdc39dde33c52a7ca760483b84567267ed00ff0a2a65169cea` |
| `docs/zh/TradeX_PRD_v1.0_RevC_zh.md` | 中文 PRD | 3661 | `32cf80c6bca84df0f68eb68da704aab158c980dbea978019f7eaf92e44ac6e11` |
| `docs/TradeX_UI_Prototype_Spec_v1.0_RevC.md` | English UI Spec | 1303 | `521c19fc84eb07ca76cd32f094814bb198e400db20d245dc3156e604a9bc206c` |
| `docs/zh/TradeX_UI_Prototype_Spec_v1.0_RevC_zh.md` | 中文 UI Spec | 1018 | `80416f8c371945f6854a0a519dd4b0ae7a49351006e6b24ffc58f075363d0c37` |
| `docs/TradeX_Frontend_ARD_v1.0_RevC.md` | English Frontend ARD | 1178 | `942920a24c331bec396cde38901f9ed2bb5183ce48ece6392dd2e53c7bdbe95d` |
| `docs/zh/TradeX_Frontend_ARD_v1.0_RevC_zh.md` | 中文 Frontend ARD | 1176 | `cf3c9811c23619ae5b2033671cb6faff1d0d78cb4212c032ca97577f61e07b6d` |
| `docs/TradeX_Backend_ARD_v1.0_RevC.md` | English Backend ARD | 3369 | `63ff53249b5c600a83e4dd3edd3b8f724558861571b625c73d51e3b1f508be98` |
| `docs/zh/TradeX_Backend_ARD_v1.0_RevC_zh.md` | 中文 Backend ARD | 3367 | `e12ebb3491f68b9fdc2c6d12032f9059a5bb3c25c73fbfc615726301373a79e1` |
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
