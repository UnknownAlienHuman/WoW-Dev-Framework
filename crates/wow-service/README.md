# Product operations

Delivery state: [CURRENT_STATUS](../../docs/CURRENT_STATUS.md). Owner boundaries:
[IMPLEMENTATION](../../docs/IMPLEMENTATION.md#code-ownership).

The existing lower `addon_check` composes exact snapshots and returns scope,
completeness, diagnostics and limitations. It is not the frontend trust boundary.
Slice 1A adds one narrow public Service operation that owns Project preparation,
plan/input binding, requested-scope limitations, exact snapshot construction,
same-owner Emmy requirements, structured operation errors and invocation of the
lower check.

CLI/LSP/MCP call only that public operation. They never construct the lower
request or attach limitations. Keep observations out of requested-scope blockers,
preserve structured source origin, and do not introduce a generic backend,
registry, receipt, publication or result hierarchy. Exact contracts live in
[IMPLEMENTATION](../../docs/IMPLEMENTATION.md#first-implementation-slice) and the
[queue](../../docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check).
