# Crate guide

The active product and its intended workflows are defined in
[`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md). Choose the next bounded
change from [`../docs/REFACTOR_QUEUE.md`](../docs/REFACTOR_QUEUE.md), then read
the target crate README, its current implementation, and its real callers.
Follow repository [`../AGENTS.md`](../AGENTS.md) and shared
[`AGENTS.md`](AGENTS.md) guidance.

## Current owner map

| Crate | Responsibility |
|---|---|
| `wow-core` | Shared IDs, small common types and errors |
| `wow-store` | Local SQLite persistence, queries and transactions |
| `wow-reference` | Blizzard API/source facts and reference loading |
| `wow-annotations` | Native Ketho annotation pipeline and output |
| `wow-ketho-literals` | Shared native literal rendering algorithm |
| `wow-render-contract` | Small owner-neutral literal rendering types |
| `wow-emmy` | Adapter to upstream EmmyLua parsing and analysis |
| `wow-project` | Configured source roots, TOC/XML interpretation and project facts |
| `wow-graph` | Useful typed source relationships |
| `wow-recognizers` | Reusable recognizers over parsed/project facts |
| `wow-rules` | Actionable WoW diagnostics |
| `wow-search` | Local Blizzard/addon search and ranking |
| `wow-context` | Bounded, located source/API context |
| `wow-service` | Multi-owner use cases |

This map describes responsibilities, not a required crate count or activation
sequence. Cargo workspace membership and `Cargo.toml` dependencies are the
source of truth for what is compiled and which edges are real. The architecture
and queue describe product scope and work order; they are not duplicate package
status ledgers.

Keep algorithms cohesive in their current owner. Applications and tools may
call an owner directly for a single-owner operation; use Service when a use case
actually composes owners. Keep dependency direction acyclic and avoid adding a
crate or abstraction for forwarding alone. See
[`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md) for concise boundary guidance.
