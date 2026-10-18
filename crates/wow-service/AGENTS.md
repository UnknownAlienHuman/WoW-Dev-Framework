# `wow-service` contribution guidance

Follow repository [`AGENTS.md`](../../AGENTS.md),
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md),
[`docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md), and the shared
[`crates/AGENTS.md`](../AGENTS.md).

Service owns simple use cases that compose multiple owners, such as loading a
configured project and assembling diagnostics, search results, or context. It
validates user-facing options, orders owner calls, and returns useful results
or real errors. Keep parsers, analysis, ranking, graph algorithms, and SQL in
their owners. Do not add a wrapper for a cohesive single-owner operation when
the CLI/LSP/MCP app can call that owner directly.

Use plain typed inputs and outputs. Avoid cryptographic result envelopes,
generic operation registries, durable effect journals, lease/replay/recovery
protocols, expansive coverage taxonomies, and platform/release orchestration
without a concrete user path that needs them. Remove legacy variants and
callers together; derived project data can be rebuilt from source/configuration.

Resolve only the configuration needed for the selected request. Keep flavor,
root, and TOC selection explicit when ambiguous. Preserve cancellation and
bounded work through owner calls. An unavailable parser/source capability or
failed input should surface as a clear error or visibly incomplete result, not
as a clean empty answer.

Keep transport framing in the application, while allowing service results to
be projected simply into CLI output, stdio LSP, or MCP. Inspect real call sites
before changing a shared use case; do not treat E0-E7 plans, optional providers,
release acceptance, daemon lifecycle, or migration workflows as mandatory
scope for ordinary analysis/search work. Validate the narrow affected owner
chain with existing focused checks when useful.
