# S29 conditional-fee contract — approved for AC6; AC5 remains waived

The user approved the bounded conditional-fee contract on 2026-10-15 for continued S29.11 AC6 work. The baseline RevC product documents remain unchanged; this records the scoped owning-plan decision and does not grant data-use rights or satisfy any independent financial gate. AC5 remains waived/unaccepted and its second-read active-claim acceptance sequence is not part of this work.

## Evidence and current conflict

The official Binance `account.commission` response exposes current standard, tax, and special rates plus the BNB discount flags, asset, and multiplier. It does not identify a unique asset for a future fill. Completed trade history carries the actual `commissionAsset`. Binance's current support explanation describes the conditional branches: default fees are deducted from the received asset; enabled BNB deduction uses discounted BNB when eligible; insufficient BNB uses the original fee; and the displayed amount is an estimate because final fills are not known before execution.

Sources:

- [Binance Account — Spot WebSocket API](https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-api/account)
- [How to Use BNB to Pay for Fees and Earn up to 25% Discount?](https://www.binance.com/en/support/faq/detail/115000583311)
- [What is Binance Spot Trading Fee and How to Calculate](https://www.binance.com/en-ZA/support/faq/detail/e85d6e703b874674840122196b89780a)

Before the 2026-10-15 decision, the S29.11 owning-plan rule treated an unknown future charging asset as making the owning statement unavailable. AC5 requires a genuine public current account read with at least one active same-account reservation so the read path can prove exact local-claim subtraction. The active reservation is created by public Prepare, which itself requires an eligible approval. Directly inserting a reservation would violate the no-seeded-finance acceptance rule. Therefore the two requirements formed a closed gate for a fresh synthetic end-to-end AC5 test; AC5 was later waived and that second-read sequence is not being pursued.

## Approved contract

Represent the fee asset as a bounded conditional outcome, not as a known future asset:

- Keep the actual fill's `feeCurrency` as `UNKNOWN_UNTIL_FILL` and retain the explicit no-guarantee limitation.
- Permit a complete same-intent/account current owning estimate to qualify review and Prepare only when the account-bound commission declaration, wallet coverage, required market references, configured source policy, and every independent risk/permission guard are complete and current.
- Bound the modeled outcome set to the documented BNB-discount branch and received-asset fallback. Do not infer any third asset, parity, zero cost, or guaranteed fill/fee amount.
- If the current BNB branch is selected, derive and atomically reserve its current native BNB allowance. If the received-asset fallback is selected, preserve it as an incoming-asset deduction and do not fabricate an outgoing fee claim.
- Show the selected branch, the alternative, exact wallet receipt and local claims, model limitations, and `ACTUAL_FEE_ASSET_PENDING` in approval review. Approval remains user-authorized and does not become a broker-enforced fee cap.
- Prepare and immediate Gateway preflight revalidate the same Proposal/account/rule/source bindings, wallet claim capacity, BNB reference and freshness. Any material change or unavailable input refuses before provider mutation; it does not silently switch branches or reuse a prior estimate.
- Settlement uses broker-reported `commissionAsset` and actual debit. A different asset, missing fee, or debit beyond the saved allowance remains frozen for reconciliation; it is never clipped or retroactively approved.
- Do not change Alpaca source-use, FX rights, account permissions, Arm, consent expiry, or any other independent gate. The user's local/non-commercial context and lack of a Support response remain unchanged.

This removes only the requirement for a unique pre-fill asset declaration. It does not turn a current model into a guaranteed fee bound. The user's approval supersedes the 2026-10-12 S29.11 choice to wait for a uniquely known pre-fill fee asset, within this feature's owning-plan implementation scope. It does not rewrite the RevC baseline documents or relax its other hard limits and independent guards.

## Proposed acceptance and test seam

Use one highest-level Rust Control Plane integration seam with disposable SQLite storage and public commands. Only external HTTP, vault, WebSocket, and clock producers may be synthetic. Do not seed reservations, approval rows, financial evidence, or `VERIFIED` rights.

1. Through ordinary public account/configuration, Proposal, evidence refresh, review, approval, and Prepare commands, create a first active reservation from a fully eligible conditional plan without dispatching an order.
2. Through a second Proposal and authenticated account read for the same account, assert exact BASE/QUOTE/BNB free/locked rows, original first receipts/digest, full binding, and per-asset local claims. Assert the first reservation is subtracted exactly once; released and other-account reservations are excluded.
3. Assert missing, duplicate, stale, mismatched, insufficient, or changed evidence removes the current proof or refuses Prepare atomically, leaving no partial approval consumption, reservation, or provider mutation.
4. Reopen the workspace and verify captured evidence retains its original receipts while a new account read is required for current eligibility.
5. Separately validate responsive review copy shows the conditional branch and pending actual fee asset without implying a guaranteed amount.

Positive source-use and market-data rights must come from the actual permitted source contract; no test-only report may claim the user's missing Alpaca Support response. If no permissible public source path can satisfy those independent gates, this proposed seam cannot claim full AC5/AC10 acceptance.

## Decision status

Approved by the user on 2026-10-15: “允许条件分支（推荐）：当前估算可进入 review/Prepare；实际币种成交后确认，偏差冻结核对”. This decision allows the bounded current estimate described above to qualify review and Prepare when every independent gate passes; settlement still waits for broker truth and freezes deviations for reconciliation. AC5 remains waived/unaccepted, AC6 remains pending implementation evidence, and AC1 source-use evidence remains unresolved. No RevC baseline document, issue checkbox, source-use status, or unrelated live financial gate is changed by this decision.

## Official support evidence rechecked — 2026-10-15

The user supplied a Binance Support screenshot. The linked official pages were opened and checked on 2026-10-15:

- [How to Calculate Binance Spot Trading Fees?](https://www.binance.com/en-NG/support/faq/detail/e85d6e703b874674840122196b89780a), updated 2026-01-06. It describes received-asset fees for ordinary Spot trades, names BNB discounts, and says the displayed estimate is for reference because the actual fee depends on the amount received by the executed trade.
- [How to Use BNB to Pay for Fees and Earn up to 25% Discount?](https://www.binance.com/en/support/faq/detail/115000583311), updated 2026-02-06. It requires the BNB deduction setting and sufficient Spot BNB; an insufficient BNB balance falls back to the original fee. It also notes that the pre-order filled amount is unknown, a displayed estimate remains in the received token, conversion uses a fill-time average rate, and some assets may not qualify for the BNB discount.

These official pages establish the general conditional policy and bound the ordinary modeled branches. They do not establish one unique actual asset/amount for a future fill, guarantee that the account's BNB setting or balance remains unchanged through execution, provide an account-specific agreement, or verify any Alpaca/source-use rights. They support the proposal's explicit `UNKNOWN_UNTIL_FILL` settlement result and conditional current estimate; they do not, by themselves, adopt the proposed change to the current owning-plan refusal rule.

The user's 2026-10-15 decision resolves the S29.11 fee-qualification question in favor of the bounded conditional contract. The existing acceptance checkboxes remain unchanged until their implementation evidence passes. AC5's second-read sequence is waived; AC6's public producer-to-Prepare evidence is still pending and must not fake source-use rights. No RevC baseline document or source-use status was changed.
