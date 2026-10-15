# S29.8 evidence boundaries

Owning Binance Spot Live PLACE qualification from the delivered exact capacity and interval
inputs (#129, parent #121).

Public RED/GREEN originals: `red.txt` is the five public Rust tests against a detached worktree
at the planning baseline `dev@7307787`, where the slice implementation is absent, so the
failures are observable behaviour rather than hand-written expectations. `green.txt` is the same
five tests plus the whole `binance_rules` regression after implementation.

The browser is a separate public seam, so it carries its own pair. `ui-red.txt` records what the
real React surface did before the corrections: the generated validator rejected the payload and
the whole owning statement was undecodable in the app, and no external producer state could
qualify the delivered inputs at all. `ui-contract-red.txt` names the exact cause — a declared
64-character bound on a 71-character `sha256:` value — and why `cargo test` and
`npm run schema:check` both passed while it was present. `ui-green.txt` records the passing
replay, and `ui.json` plus the four `ui-*-region-*.png` visuals are its archived report. The GREEN
was re-run on a freshly restarted server after the independent Spec review; the renderer-only
declared-origin change and the reason for it are recorded as an amendment at the end of
`ui-green.txt`. `ui-harness-setup-notes.txt` separates my own instrumentation mistakes from those
two genuine public-surface findings.

`full-check.txt` is the whole repository gate run with the integration dev server stopped. The
external producer fixture gained two states so the positive case and the single typed blocker
exist at that seam; nothing in the product, the assertions or the Control Plane was relaxed to
reach either.

Not claimed here: no native launch, no provider-hosted financial acceptance, no physical
lifecycle, no prototype fix and no main delivery. The parent, map, S28, S17 and physical S27
gates remain OPEN, and source acceptance is not full product acceptance.
