# Product operations

Delivery state: [CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md). Owner boundaries:
[IMPLEMENTATION.md](../../docs/IMPLEMENTATION.md#code-ownership).

`addon_check` composes exact already-acquired Main, Library and optional virtual
snapshots through `ConfiguredAnalyzerSession` and returns one direct result:

- `LuaTypes` scope;
- completeness;
- exact diagnostics;
- scoped limitations with optional source location.

The public result contains no snapshot, analyzer, Project, report, proof,
component or presentation receipts. Do not wrap it in legacy local/graph/replay
result models.

Current `check_addon` still adds addon-chunk incompleteness and returns `Partial`
unconditionally. That is transitional. Completeness must be derived from facts
required by selected units and scope; unused chunk arguments or unavailable
unrequested WoW features do not make every type-only run partial.

Project owns ordinary configuration/acquisition, exact user-path projection,
selected order/context and XML/binding maps. Service orchestrates those inputs and
runs only explicitly selected WoW checks. CLI/LSP/MCP are thin adapters to the
same owners.
