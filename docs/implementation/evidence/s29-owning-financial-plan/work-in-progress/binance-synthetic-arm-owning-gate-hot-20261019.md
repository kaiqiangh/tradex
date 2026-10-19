# Binance synthetic Arm and owning-gate check — 2026-10-19

The Hot integration suite now exercises public Live-account eligibility through the production startup-reconciliation routine. A loopback-only provider fixture refreshes the disposable Binance account. Before reconciliation, `account.list` reports `canArm=false` with `RECONCILIATION_REQUIRED`, and the public `account.arm` command refuses. After `finish_startup_live_reconciliation` accepts the trusted clock and current fixture observation, eligibility becomes ready; explicit Arm sets `ARMED/EXPLICIT_USER_ARM`, and the test disarms the temporary account afterward.

The owning BNB fee-model scenario also follows this public account-health sequence before collecting its quote, fee terms, capacity, and policy-valuation inputs. The synthetic current decision passes `MARKET_DATA_USE`, `QUOTE_FRESHNESS`, `SPOT_FEE_FX`, `ORDER_NOTIONAL`, `RESERVED_CAPITAL`, and `ACCOUNT_ARMING`. The full decision remains `UNAVAILABLE` because `INSTRUMENT_RULES` is still `UNAVAILABLE`; the public approval review remains ineligible and the approval list stays empty. The Alpaca Support context and all provider responses are synthetic fixtures. This does not establish the user's source-use facts, provider entitlement, complete Live approval, or the positive owning Prepare path.

Verification:

- Focused account-arm scenario: 1/1 passed.
- Focused owning BNB scenario: 1/1 passed.
- Complete serial `binance_market_hot` integration suite: 62/62 passed.
- `cargo fmt --all -- --check` and `git diff --check` passed.

No production behavior changed. No real account/provider read, network request, approval artifact, dispatch, order, or financial action occurred. The test-only fixture's public `account.arm` state was disarmed before the temporary workspace was discarded. #135 remains open and no acceptance state changed.

## 中文

# Binance 合成账户 Arm 与 owning gate 检查 — 2026-10-19

Hot 集成套件现通过生产启动对账例程验证公开 Live 账户 eligibility。loopback-only provider fixture 在隔离工作区刷新合成 Binance 账户。对账前，`account.list` 返回 `canArm=false`、`RECONCILIATION_REQUIRED`，公开 `account.arm` 命令拒绝。生产 `finish_startup_live_reconciliation` 确认可信时钟和当前 fixture observation 后，eligibility 才就绪；显式 Arm 将账户置为 `ARMED/EXPLICIT_USER_ARM`，测试随后撤防临时账户。

owning BNB 费用模型场景也在收集行情、费用条款、容量及 policy-valuation 输入前执行这套公开账户健康顺序。合成当前决策中的`MARKET_DATA_USE`、`QUOTE_FRESHNESS`、`SPOT_FEE_FX`、`ORDER_NOTIONAL`、`RESERVED_CAPITAL`和`ACCOUNT_ARMING`均通过。整体决策仍为`UNAVAILABLE`，因为`INSTRUMENT_RULES`仍为`UNAVAILABLE`；公开 approval review 仍不可参与，approval 列表为空。Alpaca Support 上下文及所有provider响应均为合成 fixture。本检查没有证明用户真实 source-use facts、provider entitlement、完整 Live approval 或正向 owning Prepare 路径。

验证结果：

- 账户 Arm 定向场景：1/1 通过。
- owning BNB 定向场景：1/1 通过。
- 完整串行`binance_market_hot`集成套件：62/62通过。
- `cargo fmt --all -- --check`及`git diff --check`通过。

未更改生产行为。未读取真实账户/provider、发出网络请求、创建 approval artifact、派发订单或执行金融操作。测试 fixture 的公开 `account.arm` 状态已在临时工作区丢弃前撤防。#135 继续 OPEN，验收状态未更改。
