# Initial serial review — superseded by repair/review

Source `1d207478968db23e6753a7f5d243a7380df21af4`; baseline `4388d9c4ae460ee5f4d8a6a21b0ced5cfa3fe374`. Independent read-only Standards then Spec reviewers. No checks/UI/provider calls were rerun by reviewers.

## Standards

PASS. No hard documented-standard violations. One nonblocking heuristic: possible Primitive Obsession at binance_rules.rs:165, private static character field descriptors D/I/B/A. Suggested typed enum. Root disposition: this closed private descriptor cannot contain provider-selected tags, and no observed correctness failure was identified; retain it for this ticket and record the suggestion.

## Spec

NOT PASS. Two P2 findings:

1. AC2/AC4 require populated permission sets and no implication from missing/unproven tokens. binance_rules.rs:530 accepted an empty outer permissionSets and all([]) returned true, allowing account.permissions=[] to project AVAILABLE admission. Unconditional financial qualification still blocked execution.
2. AC3/AC6 require preserving and projecting order-form flags. Symbol and evidence omitted documented icebergAllowed, allowTrailingStop and list/amend/peg flags; missing or malformed flags disappeared without an unobserved/unsupported duty.

No other scope creep or implementation/spec mismatch found. These findings require public-refresh RED/GREEN repairs and fresh serial review. Ordinary-provider/native-read/financial acceptance and parent #121 remain OPEN; prototype code is unchanged.
