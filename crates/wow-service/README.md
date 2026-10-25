# Product operations

Current status: [delivered owners and open handoffs](../../docs/CURRENT_STATUS.md).

Delivered: `addon_check.rs` composes exact already-acquired Main, Library and
optional virtual snapshots through `ConfiguredAnalyzerSession` and returns a
plain diagnostic/limitation result without Reference, Rules, Project publication,
graph, store or presentation construction. It is intentionally partial until a
validated Project order/context handoff, addon chunks and exact XML/binding
projection plus handler/callback context are connected.

Remaining work is ordinary configuration/acquisition orchestration, selected WoW
checks, coherent updates, and thin CLI/LSP/MCP adapters to the same owners. The
complete first-command checklist is queue item 1; this README does not narrow it.
The legacy local/graph/replay/lifecycle/result routes remain removal work as
their real callers move; do not wrap the direct result in those models.

See [Implementation](../../docs/IMPLEMENTATION.md#code-ownership),
[frontends](../../docs/FRONTENDS.md) and the
[trace register](../../docs/IMPLEMENTATION_TRACE_REGISTER.md).
