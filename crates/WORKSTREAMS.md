# Crate work routing

Use [`../docs/REFACTOR_QUEUE.md`](../docs/REFACTOR_QUEUE.md) to select current
work and [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for product scope.
The queue is the task order; this page does not maintain a second roadmap,
completion ledger, or acceptance gate.

A useful implementation path generally connects these existing owners:

```text
Blizzard source and Ketho behavior
  -> wow-reference / wow-annotations
  -> ordinary generated annotation library
  -> wow-emmy for Lua parsing and analysis

Configured addon or Blizzard source
  -> wow-project and wow-emmy
  -> useful graph/rules/search/context as needed
  -> CLI or stdio LSP/MCP
```

The exact path for a task depends on its current caller. Keep data and
algorithms in their existing owners; let `wow-service` compose actual
multi-owner operations. A single-owner command may call that owner directly.
Use the current source/configuration as the authority and regenerate derived
indexes or annotation output when their format changes.

For each task, inspect the smallest relevant owner and caller chain, replace an
obsolete path along with its consumer, and run a focused existing check when it
clarifies a concrete behavior. Do not require E0-E7 stage completion, frozen
fixture inventories, package acceptance matrices, optional provider work, or
release/install infrastructure for these workflows.
