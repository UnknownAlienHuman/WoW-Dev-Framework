# Crates

The [implementation guide](../docs/IMPLEMENTATION.md#code-ownership) owns the
code map and workflow paths. Cargo manifests define actual compiled dependencies;
the existing crate count is not a target to preserve.

Read root [AGENTS.md](../AGENTS.md), the [manifesto](../docs/ARCHITECTURE.md),
the selected Issue and the relevant owner code/instructions. Keep algorithms
with their owner and remove layers that only forward or copy their values.
