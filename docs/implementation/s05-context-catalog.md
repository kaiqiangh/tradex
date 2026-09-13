# S05 / #22 Canonical Context Catalog and Composer Picker

## Scope

Provide one versioned `context.catalog` query for the active workspace and use it to drive temporary Composer context/account selection. The backend remains the authority for canonical refs; the picker never creates or edits Thread history until `thread.create` or `turn.start` receives the selected list.

## Wire contract

`context.catalog` accepts the existing `{ workspaceId }` `WorkspaceQuery` payload and returns:

```ts
interface ContextCatalog {
  entries: ContextCatalogEntry[];
  emptyStates: ContextCatalogEmptyState[];
}

interface ContextCatalogEntry {
  contextRef: { kind: "account"; id: string; hash: string };
  label: string;
  providerId?: string;
  environment?: string;
  readOnly: boolean;
  available: boolean;
  availabilityReason?: string;
}

interface ContextCatalogEmptyState {
  kind: "instrument" | "account" | "strategy" | "backtest" | "artifact";
  availabilityReason: string;
}
```

S05 populates only persisted account connections. Account hashes use the canonical non-secret serialization `workspaceId\0connectionId\0providerId\0environment\0createdAt`, encoded as `sha256:<64 lowercase hex characters>`. Labels, credentials, remote response bodies and permission details never enter the hash or catalog response. Disconnected, missing-credential and cleanup-pending rows stay visible for recovery but are `available: false` with a concrete reason. Future kinds return explicit empty states and no fixture entities.

Every supplied `ThreadContextRef` is bounded, uses a supported kind, and has a canonical SHA-256 hash. Account refs must match the active workspace's catalog entry and derived hash. Unknown account refs, duplicate refs, unsupported kinds and malformed hashes fail closed before persistence or runtime work.

## Composer behavior

- `@ Context` and `Account` are keyboard-reachable buttons/selectors. Opening the picker copies the current pending refs into a temporary draft.
- `Attach` replaces pending refs with the selected chips. `Cancel` closes the picker without changing pending refs. Removing a chip changes only the pending next-turn list.
- Account rows show label, provider, environment and availability. A Live account selected in Ask/Research is explicitly `LIVE · READ-ONLY`; Backtest keeps it as an optional read-only seed. The capability query remains the source of legal/blocked explanations.
- An available attached account context contributes `account_read` in Ask/Research/Backtest, while the separate Account picker remains the only execution-account input for Trade.
- New Thread sends the attached refs in `linkedContexts`. An existing Thread keeps its historical snapshot; picker changes are passed only on the next `turn.start`. Reload restores stored refs, not a pending draft.
- `turn.start` distinguishes an omitted `attachedContexts` field (reuse the saved Thread refs for compatibility) from an explicit empty array (clear every context for this Turn); explicit `null` is rejected at the wire boundary.
- Empty future catalogs explain which owning slice will populate them. No account or context is silently substituted.

## Verification boundary

Rust tests cover stable account hashes, account catalog isolation, malformed/duplicate/unknown refs, unavailable rows and no mutation on rejected create/start requests. The isolated browser path covers open/cancel/attach/remove, provider/environment/read-only labels, history preservation after reload, keyboard focus return and 390/768 no-overflow. Real broker credentials and external instrument/research data remain outside #22.
