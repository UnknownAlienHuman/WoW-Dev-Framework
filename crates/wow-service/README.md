# Product operations

Delivery state: [CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md). Owner boundaries:
[IMPLEMENTATION.md](../../docs/IMPLEMENTATION.md#code-ownership).

The existing lower `addon_check` composes exact snapshots and returns scope,
completeness, diagnostics and limitations without legacy platform receipts. It is
not a safe frontend boundary because its public request accepts independently
supplied snapshots and arbitrary limitations.

Slice 1A adds one narrow public Service operation that owns:

- Project target/scope preparation and plan/Main validation;
- held-root acquisition and same-file authority checks across Main/Libraries;
- standalone requested-scope limitations;
- compiled-pin Main/per-root Library snapshots;
- snapshot-ID -> configured-root mapping for structured input failures;
- construction of the lower request and invocation of existing `check_addon`.

CLI/LSP/MCP call only that operation. They never call `with_input_limitations`,
construct snapshots or combine a plan with unrelated Main bytes. Keep the lower
request/check crate-internal once callers move.

Only requested-scope blockers enter the result. Project observations remain
Project data. Remove unconditional chunk incompleteness. Preserve role, root
token/snapshot ID and relative path through errors; malformed caller input is
invalid input, not `ComponentUnavailable`. Ambient absolute paths are display
context, not result identity.
